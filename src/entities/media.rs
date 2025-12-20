use std::{
    collections::{BTreeMap, HashMap},
    ops::Range,
    path::PathBuf,
    sync::Arc,
};

use chrono::NaiveDate;
use gpui::*;
use uuid::Uuid;

use crate::web::api::models::cloud::res::FileMetaReponse;

pub const MEDIA_HEIGHT: Pixels = px(176.);

pub struct FetchMedia {
    pub file: Arc<FileMetaReponse>,
}

impl EventEmitter<FetchMedia> for MediaState {}

#[derive(Debug, Clone)]
pub enum PreviewAssetType {
    Loading,
    Preview(PathBuf),
    VideoUrl(String),
}

#[derive(Debug, Clone)]
pub enum MediaAssetState {
    Idle,
    Queued,
    Loaded {
        thumbnail_path: PathBuf,
        preview_asset_ty: PreviewAssetType,
    },
    Error,
}

#[derive(Debug, Clone)]
pub enum ElementType {
    File(Arc<FileMetaReponse>),
    Section(Arc<NaiveDate>),
}

#[derive(Debug, Clone)]
pub struct MediaState {
    view_list: Vec<ElementType>,
    asset_states: HashMap<Uuid, MediaAssetState>,
    visible_grid: Vec<(Pixels, Range<usize>)>,
}

impl MediaState {
    pub fn new() -> Self {
        Self {
            view_list: Vec::new(),
            asset_states: HashMap::new(),
            visible_grid: Vec::new(),
        }
    }

    pub fn view_list(&self) -> &Vec<ElementType> {
        &self.view_list
    }

    pub fn get_file(&self, index: usize) -> Option<Arc<FileMetaReponse>> {
        self.view_list.get(index).and_then(|el| match el {
            ElementType::File(file_meta_reponse) => Some(file_meta_reponse.clone()),
            _ => None,
        })
    }

    pub fn asset_mut(&mut self, asset_id: &Uuid) -> Option<&mut MediaAssetState> {
        self.asset_states.get_mut(asset_id)
    }

    pub fn asset(&self, asset_id: &Uuid) -> Option<&MediaAssetState> {
        self.asset_states.get(asset_id)
    }

    pub fn set_files(&mut self, files: Vec<FileMetaReponse>) {
        self.asset_states = files.iter().fold(HashMap::new(), |mut acc, file| {
            acc.insert(file.id.clone(), MediaAssetState::Idle);
            acc
        });

        let mut grouped_files = files.into_iter().fold(
            BTreeMap::<NaiveDate, Vec<FileMetaReponse>>::new(),
            |mut acc, file| {
                let date = file.updated_at.date_naive();
                match acc.get_mut(&date) {
                    Some(list) => {
                        list.push(file);
                    }
                    None => {
                        acc.insert(date, vec![file]);
                    }
                };

                acc
            },
        );

        grouped_files.iter_mut().for_each(|(_, files)| {
            files.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        });

        for (date, files) in grouped_files.into_iter().rev() {
            self.view_list.push(ElementType::Section(Arc::new(date)));
            self.view_list.extend(
                files
                    .into_iter()
                    .rev()
                    .map(|file| ElementType::File(Arc::new(file))),
            );
        }
    }

    pub fn compute_visible_grid(&mut self, bounds: &super::bounds::RenderBounds) {
        if self.view_list.is_empty() {
            return;
        }

        self.visible_grid = Vec::new();

        let mut start = 0;
        let mut used_height = px(0.);
        let mut used_width = 16f32;

        for (i, element) in self.view_list.iter().enumerate() {
            match element {
                ElementType::File(file) => {
                    let fw = file.width as f32;

                    if px(fw + 4. + used_width) > bounds.width {
                        used_height += MEDIA_HEIGHT + px(4.);
                        self.visible_grid.push((used_height, start..i));

                        start = i;
                        used_width = 16. + fw + 4.;
                    } else {
                        used_width += fw + 4.;
                    }
                }
                ElementType::Section(_) => {
                    used_height += px(38.);
                    if i > 0 {
                        used_height += MEDIA_HEIGHT + px(4.);
                    }
                    self.visible_grid.push((used_height, start..i));
                    start = i + 1;
                    used_width = 16.;
                }
            };
        }

        used_height += MEDIA_HEIGHT + px(4.);
        self.visible_grid
            .push((used_height, start..self.view_list.len()));
    }

    pub fn get_initial_visible_range(&self, bounds: &super::bounds::RenderBounds) -> Range<usize> {
        let visible_row_count = self
            .visible_grid
            .len()
            .min((bounds.height / MEDIA_HEIGHT).ceil() as usize + 3);

        let start = self.visible_grid.first().map(|(_, r)| r.start).unwrap_or(0);
        let end = self
            .visible_grid
            .get(visible_row_count)
            .map(|(_, r)| r.end)
            .unwrap_or(0);

        start..end
    }

    pub fn get_visible_state(&self, scroll_offset: Pixels, end_offset: Pixels) -> Range<usize> {
        let (start_index, start) = self
            .visible_grid
            .iter()
            .enumerate()
            .find(|(_, (o, _))| &scroll_offset < o)
            .map(|(i, (_, r))| (i, r.start))
            .unwrap_or((0, 0));

        let end = self
            .visible_grid
            .iter()
            .skip(start_index)
            .find(|(o, _)| o > &end_offset)
            .map(|(_, r)| r.end)
            .unwrap_or(self.visible_grid.last().map(|(_, r)| r.end).unwrap_or(0));

        start..end
    }

    pub fn clear(&mut self) {
        self.view_list.clear();
        self.asset_states.clear();
        self.visible_grid.clear();
    }
}

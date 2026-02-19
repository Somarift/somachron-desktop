use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    ops::Range,
    path::PathBuf,
    sync::Arc,
};

use chrono::NaiveDate;
use futures::FutureExt;
use gpui::*;
use uuid::Uuid;

use crate::{err::AppError, web::api::models::cloud::res::FileMetaReponse};

pub const MEDIA_HEIGHT: Pixels = px(176.);
pub const MEDIA_GAP: Pixels = px(4.);
pub const SECTION_HEIGHT: Pixels = px(36.);

pub struct FetchMedia {
    pub file: Arc<FileMetaReponse>,
}

impl EventEmitter<FetchMedia> for MediaState {}

#[derive(Debug, Clone)]
pub enum MediaAssetState {
    Idle,
    Queued,
    Loaded {
        thumbnail_path: PathBuf,
        preview_path: PathBuf,
    },
    Error(AppError),
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
    sections: Arc<Vec<(Arc<NaiveDate>, Pixels)>>,
}

impl MediaState {
    pub fn new() -> Self {
        Self {
            view_list: Vec::new(),
            asset_states: HashMap::new(),
            visible_grid: Vec::new(),
            sections: Arc::new(Vec::new()),
        }
    }

    pub fn view_list(&self) -> &Vec<ElementType> {
        &self.view_list
    }

    pub fn sections(&self) -> Arc<Vec<(Arc<NaiveDate>, Pixels)>> {
        self.sections.clone()
    }

    pub fn get_file(&self, index: usize) -> Option<Arc<FileMetaReponse>> {
        self.view_list.get(index).and_then(|el| match el {
            ElementType::File(file_meta_reponse) => Some(file_meta_reponse.clone()),
            _ => None,
        })
    }

    pub fn get_next_file(&self, mut index: usize) -> Option<(usize, Arc<FileMetaReponse>)> {
        let mut iter = self.view_list.iter();
        match iter.nth(index) {
            Some(ElementType::File(file)) => Some((index, file.clone())),
            _ => {
                for element in iter {
                    index += 1;
                    if let ElementType::File(file) = element {
                        return Some((index, file.clone()));
                    }
                }

                None
            }
        }
    }

    pub fn get_prev_file(&self, mut index: usize) -> Option<(usize, Arc<FileMetaReponse>)> {
        let mut iter = self.view_list.iter();
        match iter.nth(index) {
            Some(ElementType::File(file)) => Some((index, file.clone())),
            _ => {
                while let Some(element) = iter.next_back() {
                    index = index.checked_sub(1).unwrap_or(index);
                    if let ElementType::File(file) = element {
                        return Some((index, file.clone()));
                    }
                }

                None
            }
        }
    }

    pub fn asset_mut(&mut self, asset_id: &Uuid) -> Option<&mut MediaAssetState> {
        self.asset_states.get_mut(asset_id)
    }

    pub fn asset(&self, asset_id: &Uuid) -> Option<&MediaAssetState> {
        self.asset_states.get(asset_id)
    }

    pub fn set_files(&mut self, files: Vec<FileMetaReponse>) {
        self.asset_states = files.iter().fold(HashMap::new(), |mut acc, file| {
            acc.insert(file.id, MediaAssetState::Idle);
            acc
        });

        let mut grouped_files =
            files
                .into_iter()
                .fold(BTreeMap::<NaiveDate, Vec<FileMetaReponse>>::new(), |mut acc, file| {
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
                });

        grouped_files.iter_mut().for_each(|(_, files)| {
            files.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        });

        for (date, files) in grouped_files.into_iter().rev() {
            self.view_list.push(ElementType::Section(Arc::new(date)));
            self.view_list
                .extend(files.into_iter().map(|file| ElementType::File(Arc::new(file))));
        }
    }

    pub fn compute_visible_grid(&mut self, bounds: &super::bounds::RenderBounds) {
        if self.view_list.is_empty() {
            return;
        }

        self.visible_grid = Vec::new();
        let mut sections = Vec::new();
        for element in self.view_list.iter().take(1) {
            if let ElementType::Section(date) = element {
                sections.push((date.clone(), px(0.)));
            }
        }

        let mut start = 0;
        let mut used_height = SECTION_HEIGHT + MEDIA_GAP;
        let mut used_width = px(0.);
        let bounded_width = bounds.width - px(16.);

        for (i, element) in self.view_list.iter().enumerate().skip(1) {
            match element {
                ElementType::File(file) => {
                    let fw = px(file.width as f32);

                    if (MEDIA_GAP + fw + used_width) > bounded_width {
                        used_height += MEDIA_HEIGHT + MEDIA_GAP;
                        self.visible_grid.push((used_height, start..i));

                        start = i;
                        used_width = fw;
                    } else if used_width.is_zero() {
                        used_width += fw;
                    } else {
                        used_width += MEDIA_GAP + fw;
                    }
                }
                ElementType::Section(date) => {
                    used_height += MEDIA_HEIGHT + MEDIA_GAP;
                    self.visible_grid.push((used_height, start..i));

                    sections.push((date.clone(), used_height));
                    used_height += SECTION_HEIGHT + MEDIA_GAP;

                    start = i + 1;
                    used_width = px(0.);
                }
            };
        }

        used_height += MEDIA_HEIGHT + MEDIA_GAP;
        self.visible_grid.push((used_height, start..self.view_list.len()));

        self.sections = Arc::new(sections);
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
        if self.visible_grid.is_empty() {
            return 0..0;
        }

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
        self.sections = Arc::new(Vec::new());
        self.view_list.clear();
        self.asset_states.clear();
        self.visible_grid.clear();
    }
}

pub struct MediaCacher {
    max_items: usize,
    deque: VecDeque<u64>,
    cache: HashMap<u64, ImageCacheItem>,
}

impl MediaCacher {
    pub fn new(max_items: usize, cx: &mut Context<Self>) -> Self {
        assert_ne!(max_items, 0);

        cx.on_release(|cache, cx| {
            for (_, mut item) in std::mem::take(&mut cache.cache) {
                if let Some(Ok(image)) = item.get() {
                    cx.drop_image(image, None);
                }
            }
        })
        .detach();

        Self {
            max_items,
            deque: VecDeque::with_capacity(max_items),
            cache: HashMap::with_capacity(max_items),
        }
    }

    pub fn update_max_items(&mut self, max_items: usize) {
        self.max_items = max_items;
    }
}

impl ImageCache for MediaCacher {
    fn load(
        &mut self,
        resource: &Resource,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<std::result::Result<Arc<RenderImage>, ImageCacheError>> {
        let hash = hash(resource);

        // we have cache image, return
        if let Some(item) = self.cache.get_mut(&hash) {
            return item.get();
        }

        // trim cache
        while self.deque.len() == self.max_items {
            let oldest = self
                .deque
                .pop_front()
                .expect("Wait.. how does pop fails if length is max_items ?");
            let mut image = self.cache.remove(&oldest).expect("cache not in sync ??");
            if let Some(Ok(image)) = image.get() {
                cx.drop_image(image, Some(window));
            }
        }

        // spawn loader task
        let fut = AssetLogger::<ImageAssetLoader>::load(resource.clone(), cx);
        let task = cx.background_executor().spawn(fut).shared();

        // cache image
        self.cache.insert(hash, ImageCacheItem::Loading(task.clone()));
        self.deque.push_back(hash);

        // notify
        let entity_id = window.current_view();
        window
            .spawn(cx, async move |cx| {
                let _ = task.await;
                cx.on_next_frame(move |_window, cx| {
                    cx.notify(entity_id);
                });
            })
            .detach();

        None
    }
}

use std::{path::PathBuf, sync::Arc};

use gpui::*;
use uuid::Uuid;

use crate::{
    err::AppError,
    web::api::models::cloud::res::{FolderResponse, InitiateUploadResponse},
};

#[derive(Debug, Clone)]
pub enum UploadUrlState {
    Queued,
    Uploading(InitiateUploadResponse),
    Done,
    Error(AppError),
}

pub struct UploadState {
    pub path: PathBuf,
    pub url_state: UploadUrlState,
}

pub enum UploadJobStatus {
    Queued,
    InProgress,
    Done,
}

pub struct UploadJob {
    pub uploads: Vec<UploadState>,
    pub space_id: Uuid,
    pub folder: FolderResponse,
    pub status: UploadJobStatus,
}

impl UploadJob {
    pub fn new(
        paths: Arc<Vec<PathBuf>>,
        space_id: Uuid,
        folder: FolderResponse,
        cx: &mut App,
    ) -> Entity<Self> {
        let uploads = paths
            .iter()
            .map(|path| UploadState {
                path: path.clone(),
                url_state: UploadUrlState::Queued,
            })
            .collect();

        cx.new(|_cx| Self {
            uploads,
            space_id,
            folder,
            status: UploadJobStatus::Queued,
        })
    }
}

pub struct UploadEvent;

impl EventEmitter<UploadEvent> for UploadManager {}

pub struct UploadManager {
    jobs: Vec<Entity<UploadJob>>,
}

impl UploadManager {
    pub fn new() -> Self {
        Self { jobs: Vec::new() }
    }

    pub fn jobs(&self) -> &Vec<Entity<UploadJob>> {
        &self.jobs
    }

    pub fn next(&self, cx: &App) -> Option<&Entity<UploadJob>> {
        self.jobs
            .iter()
            .find(|j| matches!(j.read(cx).status, UploadJobStatus::Queued))
    }

    pub fn trim_completed(&mut self, cx: &App) {
        self.jobs = self
            .jobs
            .iter()
            .cloned()
            .filter(|j| matches!(j.read(cx).status, UploadJobStatus::Queued))
            .collect();
    }

    pub fn push<T: EventEmitter<UploadEvent>>(
        &mut self,
        job: Entity<UploadJob>,
        cx: &mut Context<T>,
    ) {
        self.jobs.push(job);
        cx.emit(UploadEvent {});
        cx.notify();
    }
}

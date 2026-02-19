use std::{collections::HashMap, path::PathBuf, sync::Arc};

use futures::StreamExt;
use gpui::*;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    auth::InnerAuth,
    err::AppError,
    rt,
    util::MapAsync,
    web::api::{
        self, UploadRet,
        models::cloud::res::{DownloadUrlResponse, FolderResponse, InitiateUploadResponse},
    },
};

#[derive(Debug, Clone)]
pub enum UrlState<T> {
    Queued,
    Transferring(T),
    Processing,
    Done,
    Error(AppError),
}

pub struct UploadState {
    pub path: PathBuf,
    pub url_state: UrlState<InitiateUploadResponse>,
}

pub struct DownloadState {
    pub file_id: Uuid,
    pub path: PathBuf,
    pub url_state: UrlState<DownloadUrlResponse>,
}

#[derive(Debug, Clone, Copy)]
pub enum JobStatus {
    Queued,
    InProgress,
    Done,
    Failed,
}

pub struct UploadJob {
    pub uploads: Vec<UploadState>,
    pub space_id: Uuid,
    pub folder: FolderResponse,
    pub status: JobStatus,
    pub completed: usize,
}

impl UploadJob {
    pub async fn execute(
        inner: Arc<InnerAuth>,
        job: Entity<Self>,
        cx: &mut AsyncWindowContext,
    ) -> Result<(), AppError> {
        let (tx, mut rx) = mpsc::unbounded_channel::<(usize, UrlState<InitiateUploadResponse>)>();

        let task = job
            .read_with(cx, |this, cx| {
                let folder_id = this.folder.id;
                let space_id = this.space_id;

                let files: Vec<(Arc<str>, PathBuf)> = this
                    .uploads
                    .iter()
                    .map(|state| {
                        (
                            state
                                .path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .map_or(Arc::from(""), Arc::from),
                            state.path.clone(),
                        )
                    })
                    .collect();

                rt::spawn(cx, async move {
                    let init_uploads = files.into_iter().enumerate().map(|(i, (file_name, path))| {
                        let _tx = tx.clone();
                        let inner = inner.clone();

                        async move {
                            let token = match inner.get_token().await {
                                Ok(token) => token,
                                Err(err) => {
                                    _tx.send((i, UrlState::Error(err))).unwrap();
                                    return;
                                }
                            };

                            let result = api::cloud::init_file_upload(&token, &space_id, &folder_id, &file_name).await;
                            let data = match result {
                                Ok(data) => {
                                    _tx.send((i, UrlState::Transferring(data.clone()))).unwrap();
                                    data
                                }
                                Err(err) => {
                                    _tx.send((i, UrlState::Error(err))).unwrap();
                                    return;
                                }
                            };

                            let result = api::upload(data.url.as_str(), path).await;
                            let UploadRet {
                                file_size,
                                updated_millis,
                            } = match result {
                                Ok(data) => {
                                    _tx.send((i, UrlState::Processing)).unwrap();
                                    data
                                }
                                Err(err) => {
                                    _tx.send((i, UrlState::Error(err))).unwrap();
                                    return;
                                }
                            };

                            let token = match inner.get_token().await {
                                Ok(token) => token,
                                Err(err) => {
                                    _tx.send((i, UrlState::Error(err))).unwrap();
                                    return;
                                }
                            };
                            let result = api::cloud::queue_media(
                                &token,
                                &space_id,
                                &folder_id,
                                &file_name,
                                file_size,
                                updated_millis,
                            )
                            .await;
                            match result {
                                Ok(_) => _tx.send((i, UrlState::Done)).unwrap(),
                                Err(err) => _tx.send((i, UrlState::Error(err))).unwrap(),
                            };
                        }
                    });

                    let _ = futures::stream::iter(init_uploads)
                        .buffer_unordered(8)
                        .collect::<Vec<_>>()
                        .await;

                    Ok(())
                })
            })
            .map_err(|err| AppError::err(err.root_cause()))?;

        cx.spawn(async move |cx| {
            while let Some((index, result)) = rx.recv().await {
                let _ = job.update(cx, |job, cx| {
                    if let UrlState::Done = result {
                        job.completed += 1;
                    }
                    if let Some(u) = job.uploads.get_mut(index) {
                        u.url_state = result;
                    }
                    cx.notify();
                });
            }
        })
        .detach();

        let _ = task.await.flatten()?;

        Ok(())
    }
}

pub struct DownloadJob {
    pub dst_path: PathBuf,
    pub downloads: Vec<DownloadState>,
    pub space_id: Uuid,
    pub status: JobStatus,
}
impl DownloadJob {
    pub async fn initialize_downloads(inner: Arc<InnerAuth>, job: Entity<Self>, cx: &mut AsyncWindowContext) {
        let tasks = job
            .read_with(cx, |this, cx| {
                let job = job.clone();
                let space_id = this.space_id;

                this.downloads
                    .iter()
                    .enumerate()
                    .map(|(i, state)| {
                        let job = job.clone();
                        let inner = inner.clone();
                        let file_id = state.file_id;

                        let task = rt::spawn(cx, async move {
                            inner
                                .get_token()
                                .await
                                .map_async(async move |token| {
                                    api::cloud::get_download_stream_url(&token, &space_id, &file_id).await
                                })
                                .await
                        });

                        cx.spawn(async move |cx| {
                            let result = task.await.flatten();

                            let _ = job.update(cx, |job, cx| {
                                if let Some(u) = job.downloads.get_mut(i) {
                                    match result {
                                        Ok(data) => u.url_state = UrlState::Transferring(data),
                                        Err(err) => u.url_state = UrlState::Error(err),
                                    };
                                }
                                cx.notify();
                            });
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        run_batched(cx, tasks).await
    }

    pub async fn download_files(job: Entity<Self>, cx: &mut AsyncWindowContext) {
        let tasks = job
            .read_with(cx, |this, cx| {
                this.downloads
                    .iter()
                    .enumerate()
                    .filter_map(|(i, state)| {
                        let job = job.clone();
                        match &state.url_state {
                            UrlState::Transferring(data) => {
                                let url = data.url.clone();
                                let path = state.path.clone();
                                let task = rt::spawn(cx, async move { api::download(url, path).await });

                                Some(cx.spawn(async move |cx| {
                                    let result = task.await.flatten();
                                    let _ = job.update(cx, |job, cx| {
                                        if let Some(u) = job.downloads.get_mut(i) {
                                            match result {
                                                Ok(_) => u.url_state = UrlState::Done,
                                                Err(err) => u.url_state = UrlState::Error(err),
                                            };
                                        }
                                        cx.notify();
                                    });
                                }))
                            }
                            _ => None,
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        run_batched(cx, tasks).await
    }
}

async fn run_batched<T: 'static>(cx: &mut AsyncWindowContext, tasks: Vec<Task<T>>) {
    cx.spawn(async move |_cx| {
        let _ = futures::stream::iter(tasks)
            .buffer_unordered(8)
            .collect::<Vec<_>>()
            .await;
    })
    .await
}

#[derive(Debug, Clone)]
pub enum TransferJob {
    Upload(Entity<UploadJob>),
    Download(Entity<DownloadJob>),
}

impl TransferJob {
    pub fn upload(paths: Arc<Vec<PathBuf>>, space_id: Uuid, folder: FolderResponse, cx: &mut App) -> Self {
        let uploads = paths
            .iter()
            .map(|path| UploadState {
                path: path.clone(),
                url_state: UrlState::Queued,
            })
            .collect();

        Self::Upload(cx.new(|_cx| UploadJob {
            uploads,
            space_id,
            folder,
            status: JobStatus::Queued,
            completed: 0,
        }))
    }

    pub fn download(
        dst_path: PathBuf,
        checked_files: HashMap<Uuid, SharedString>,
        space_id: Uuid,
        cx: &mut App,
    ) -> Self {
        let downloads = checked_files
            .iter()
            .map(|(id, file_name)| {
                let path = dst_path.clone();
                let path = path.join(file_name.as_str());

                DownloadState {
                    file_id: *id,
                    path,
                    url_state: UrlState::Queued,
                }
            })
            .collect();

        Self::Download(cx.new(|_cx| DownloadJob {
            dst_path,
            downloads,
            space_id,
            status: JobStatus::Queued,
        }))
    }

    pub fn status(&self, cx: &App) -> JobStatus {
        match self {
            TransferJob::Upload(entity) => entity.read(cx).status,
            TransferJob::Download(entity) => entity.read(cx).status,
        }
    }

    pub fn summary(&self, cx: &App) -> SharedString {
        let summary = match self {
            TransferJob::Upload(entity) => entity.read_with(cx, |this, _cx| {
                format!(
                    "{}: {}",
                    match this.status {
                        JobStatus::Queued => "Queued",
                        JobStatus::InProgress => "Uploading",
                        JobStatus::Done => "Completed",
                        JobStatus::Failed => "Failed",
                    },
                    this.folder.path,
                )
            }),
            TransferJob::Download(entity) => entity.read_with(cx, |this, _cx| {
                format!(
                    "{}: {}",
                    match this.status {
                        JobStatus::Queued => "Queued",
                        JobStatus::InProgress => "Downloading to",
                        JobStatus::Done => "Completed",
                        JobStatus::Failed => "Failed",
                    },
                    this.dst_path.display(),
                )
            }),
        };

        SharedString::new(summary)
    }
}

pub struct NewJobEvent;

impl EventEmitter<NewJobEvent> for TransferManager {}

pub struct TransferManager {
    jobs: Vec<TransferJob>,
}

impl TransferManager {
    pub fn new() -> Self {
        Self { jobs: Vec::new() }
    }

    pub fn jobs(&self) -> &Vec<TransferJob> {
        &self.jobs
    }

    pub fn next(&self, cx: &App) -> Option<&TransferJob> {
        self.jobs.iter().find(|j| matches!(j.status(cx), JobStatus::Queued))
    }

    pub fn trim_completed(&mut self, cx: &App) {
        self.jobs = self
            .jobs
            .iter()
            .filter(|j| !matches!(j.status(cx), JobStatus::Done))
            .cloned()
            .collect();
    }

    pub fn push<T: EventEmitter<NewJobEvent>>(&mut self, job: TransferJob, cx: &mut Context<T>) {
        self.jobs.push(job);
        cx.emit(NewJobEvent {});
        cx.notify();
    }
}

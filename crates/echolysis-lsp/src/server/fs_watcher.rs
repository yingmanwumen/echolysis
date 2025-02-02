use echolysis_core::utils::hash::FxDashSet;
use notify::{Config, Watcher};
use parking_lot::Mutex;
use std::{path::PathBuf, sync::Arc, time::Duration};
use tower_lsp::lsp_types;

use super::Server;

pub struct FsWatcher {
    watcher: Arc<Mutex<Box<dyn Watcher + Send>>>,
    folders: FxDashSet<PathBuf>,
}

impl FsWatcher {
    pub fn new<T: Watcher + 'static + Send>(
        interval: Duration,
        event_handler: Box<dyn Fn(Result<notify::Event, notify::Error>) + Send>,
    ) -> Self {
        let watcher = Box::new(
            T::new(
                event_handler,
                Config::default().with_poll_interval(interval),
            )
            .unwrap(),
        );

        Self {
            watcher: Arc::new(Mutex::new(watcher)),
            folders: FxDashSet::default(),
        }
    }

    fn unwatch(&self, folders: &[PathBuf]) {
        let mut watcher = self.watcher.lock();

        for folder in folders {
            let _ = watcher.unwatch(folder);
            self.folders.remove(folder);
        }
    }

    fn watch(&self, folders: &[PathBuf]) {
        let mut watcher = self.watcher.lock();

        for folder in folders {
            let _ = watcher.watch(folder, notify::RecursiveMode::Recursive);
            self.folders.insert(folder.clone());
        }
    }

    fn clear(&self) {
        let mut watcher = self.watcher.lock();
        for folder in self.folders.iter() {
            let _ = watcher.unwatch(&folder);
        }
        self.folders.clear();
    }
}

impl Server {
    pub(super) async fn watch(&self, folders: &[lsp_types::WorkspaceFolder]) {
        let folders: Vec<_> = folders
            .iter()
            .filter_map(|f| f.uri.to_file_path().ok())
            .collect();

        if folders.is_empty() {
            return;
        }
        self.log_info(format!("watching folders: {:?}", folders))
            .await;
        self.fs_watcher.watch(&folders);
        let files: Vec<_> = folders
            .into_iter()
            .flat_map(|folder| get_all_files_under_folder(&folder))
            .zip(std::iter::repeat(None))
            .collect();
        self.on_insert(files).await;
    }

    pub(super) async fn unwatch(&self, folders: &[lsp_types::WorkspaceFolder]) {
        let folders: Vec<_> = folders
            .iter()
            .filter_map(|f| f.uri.to_file_path().ok())
            .collect();

        if folders.is_empty() {
            return;
        }
        self.log_info(format!("unwatching folders: {:?}", folders))
            .await;
        self.fs_watcher.unwatch(&folders);
        let files: Vec<_> = folders
            .into_iter()
            .flat_map(|folder| get_all_files_under_folder(&folder))
            .collect();
        self.on_remove(files).await;
    }

    pub(super) async fn clear(&self) {
        self.log_info("unwatching all folders").await;
        self.fs_watcher.clear();
        self.on_remove_all().await;
    }

    pub(super) async fn handle_fs_event(&self, event: Result<notify::Event, notify::Error>) {
        match event {
            Err(e) => {
                self.log_error(format!("fs event error: {e:?}")).await;
                // self.fs_watcher.unwatch(&e.paths);
            }
            Ok(event) => {
                self.do_handle_fs_event(event).await;
            }
        }
    }

    async fn do_handle_fs_event(&self, event: notify::Event) {
        use notify::{event::*, EventKind};

        let paths: Vec<_> = event
            .paths
            .iter()
            .filter(|path| {
                (path.is_file()
                    && path
                        .extension()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default()
                        != "log")
                    || self.file_map.contains_key(path.as_path())
            })
            .cloned()
            .collect();
        if paths.is_empty() {
            return;
        }

        match event.kind {
            // Full update
            EventKind::Create(_)
            | EventKind::Modify(ModifyKind::Data(_))
            | EventKind::Modify(ModifyKind::Metadata(MetadataKind::WriteTime)) => {
                self.on_insert(paths.into_iter().zip(std::iter::repeat(None)).collect())
                    .await;
            }
            EventKind::Remove(_) => {
                self.on_remove(paths).await;
            }
            _ => (),
        }
    }
}

fn get_all_files_under_folder(folder: &PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(folder).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            files.extend(get_all_files_under_folder(&path));
        } else if path.is_file() {
            files.push(path);
        }
    }
    files
}

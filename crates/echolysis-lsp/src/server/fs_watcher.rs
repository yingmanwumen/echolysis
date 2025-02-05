use dashmap::DashSet;
use notify::{Config, Watcher};
use std::path::PathBuf;
use tower_lsp::lsp_types;

use crate::server::utils::should_ignore;

use super::{
    utils::{get_all_files_under_folder, git_root},
    Server,
};

pub struct FsWatcher {
    watcher: parking_lot::Mutex<Box<dyn Watcher + Send>>,
    folders: DashSet<PathBuf, ahash::RandomState>,
}

impl FsWatcher {
    pub fn new<T: Watcher + 'static + Send>(
        interval: std::time::Duration,
        event_handler: Box<dyn Fn(Result<notify::Event, notify::Error>) + Send>,
    ) -> Result<Self, notify::Error> {
        let watcher = Box::new(T::new(
            event_handler,
            Config::default().with_poll_interval(interval),
        )?);

        Ok(Self {
            watcher: parking_lot::Mutex::new(watcher),
            folders: DashSet::with_hasher(ahash::RandomState::default()),
        })
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

    #[allow(unused)]
    fn clear(&self) {
        let mut watcher = self.watcher.lock();
        for folder in self.folders.iter() {
            let _ = watcher.unwatch(&folder);
        }
        self.folders.clear();
    }
}

impl Server {
    fn collect_folder_files(folders: Vec<PathBuf>) -> Vec<lsp_types::Url> {
        folders
            .into_iter()
            .flat_map(|folder| {
                get_all_files_under_folder(&folder)
                    .into_iter()
                    .filter_map(|path| lsp_types::Url::from_file_path(&path).ok())
            })
            .collect()
    }

    pub(super) async fn watch(&self, folders: &[lsp_types::WorkspaceFolder]) {
        let folders: Vec<_> = folders
            .iter()
            .filter_map(|f| f.uri.to_file_path().ok().and_then(|f| git_root(&f)))
            .collect();

        if folders.is_empty() {
            return;
        }
        self.fs_watcher.watch(&folders);
        let files = Self::collect_folder_files(folders)
            .into_iter()
            .zip(std::iter::repeat(None))
            .collect::<Vec<_>>();
        self.on_insert(&files).await;
    }

    pub(super) async fn unwatch(&self, folders: &[lsp_types::WorkspaceFolder]) {
        let folders: Vec<_> = folders
            .iter()
            .filter_map(|f| f.uri.to_file_path().ok())
            .collect();

        if folders.is_empty() {
            return;
        }
        self.fs_watcher.unwatch(&folders);
        let files = Self::collect_folder_files(folders);
        self.on_remove(&files).await;
    }

    #[allow(unused)]
    pub(super) async fn clear(&self) {
        self.fs_watcher.clear();
        self.on_remove_all().await;
    }

    pub(super) async fn handle_fs_event(&self, event: Result<notify::Event, notify::Error>) {
        match event {
            Err(e) => {
                self.fs_watcher.unwatch(&e.paths);
            }
            Ok(event) => {
                self.do_handle_fs_event(event).await;
            }
        }
    }

    // Handle filesystem events for code duplication analysis
    async fn do_handle_fs_event(&self, event: notify::Event) {
        use notify::{event::*, EventKind};

        // Early return for unsupported event types
        if !matches!(
            event.kind,
            EventKind::Create(_)
                | EventKind::Modify(ModifyKind::Data(_))
                | EventKind::Modify(ModifyKind::Metadata(MetadataKind::WriteTime))
                | EventKind::Remove(_)
        ) {
            return;
        }

        // Filter relevant paths (files that are not logs and tracked files)
        let uris: Vec<_> = event
            .paths
            .iter()
            .filter_map(|path| {
                if should_ignore(path) {
                    return None;
                }
                let uri = lsp_types::Url::from_file_path(path).ok()?;
                if path.is_file() || self.file_map.contains_key(&uri) {
                    Some(uri)
                } else {
                    None
                }
            })
            .collect();

        if uris.is_empty() {
            return;
        }

        // Process the event based on its type
        match event.kind {
            EventKind::Create(_)
            | EventKind::Modify(ModifyKind::Data(_))
            | EventKind::Modify(ModifyKind::Metadata(MetadataKind::WriteTime)) => {
                self.on_insert(
                    &uris
                        .into_iter()
                        .zip(std::iter::repeat(None))
                        .collect::<Vec<_>>(),
                )
                .await;
            }
            EventKind::Remove(_) => {
                self.on_remove(&uris).await;
            }
            _ => (),
        }
    }
}

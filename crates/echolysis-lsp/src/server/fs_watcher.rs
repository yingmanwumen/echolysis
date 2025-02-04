use dashmap::DashSet;
use notify::{Config, Watcher};
use std::path::PathBuf;
use tower_lsp::lsp_types;

use super::{
    utils::{get_all_files_under_folder, get_git_top_root},
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
            .filter_map(|f| f.uri.to_file_path().ok().and_then(|f| get_git_top_root(&f)))
            .collect();

        if folders.is_empty() {
            return;
        }
        self.fs_watcher.watch(&folders);
        let files = Self::collect_folder_files(folders)
            .into_iter()
            .filter(|f| {
                if let Ok(path) = f.to_file_path() {
                    !path.components().any(|c| {
                        matches!(
                            c.as_os_str().to_str(),
                            Some("venv" | "node_modules" | "target")
                        )
                    })
                } else {
                    false
                }
            })
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
        let should_process = matches!(
            event.kind,
            EventKind::Create(_)
                | EventKind::Modify(ModifyKind::Data(_))
                | EventKind::Modify(ModifyKind::Metadata(MetadataKind::WriteTime))
                | EventKind::Remove(_)
        );

        if !should_process {
            return;
        }

        // Filter relevant paths (files that are not logs and tracked files)
        let uris: Vec<_> = event
            .paths
            .iter()
            .filter_map(|path| {
                let uri = lsp_types::Url::from_file_path(path).ok()?;
                // TODO: need add a global path filter here
                // language specific path filters are also considered
                if path.components().any(|c| {
                    matches!(
                        c.as_os_str().to_str(),
                        Some("venv" | "node_modules" | "target")
                    )
                }) {
                    return None;
                }
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

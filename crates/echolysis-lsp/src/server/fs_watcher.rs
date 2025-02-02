use echolysis_core::utils::hash::FxDashSet;
use notify::{Config, Watcher};
use parking_lot::Mutex;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tower_lsp::lsp_types;

use super::{
    utils::{get_all_files_under_folder, get_git_root},
    Server,
};

/// File system watcher for monitoring workspace folder changes
///
/// Provides functionality to:
/// - Watch workspace folders for file changes
/// - Handle file system events (create/modify/delete)
/// - Manage watched folder lifecycle
pub struct FsWatcher {
    /// The underlying file system watcher implementation
    watcher: Arc<Mutex<Box<dyn Watcher + Send>>>,
    /// Set of currently watched folder paths
    folders: FxDashSet<PathBuf>,
}

impl FsWatcher {
    /// Creates a new file system watcher instance
    ///
    /// # Arguments
    /// * `interval` - Polling interval for the watcher
    /// * `event_handler` - Callback for handling file system events
    ///
    /// # Type Parameters
    /// * `T` - The concrete watcher implementation type
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
            .filter_map(|f| get_git_root(&f))
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

    pub(super) async fn try_watch_path(&self, path: &Path) {
        if !self.fs_watcher.folders.is_empty() {
            return;
        }
        if let Some(root) = get_git_root(path) {
            if let Ok(uri) = lsp_types::Url::from_directory_path(root) {
                self.watch(&[lsp_types::WorkspaceFolder {
                    uri,
                    name: String::new(),
                }])
                .await;
            }
        }
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
        let paths: Vec<_> = event
            .paths
            .iter()
            .filter(|path| {
                let is_non_log_file = path.is_file();
                // TODO: configurable file filter, ignore some file patterns here

                is_non_log_file || self.file_map.contains_key(path.as_path())
            })
            .cloned()
            .collect();

        if paths.is_empty() {
            return;
        }

        // Process the event based on its type
        match event.kind {
            EventKind::Create(_)
            | EventKind::Modify(ModifyKind::Data(_))
            | EventKind::Modify(ModifyKind::Metadata(MetadataKind::WriteTime)) => {
                let files = paths.into_iter().map(|p| (p, None)).collect();
                self.on_insert(files).await;
            }
            EventKind::Remove(_) => {
                self.on_remove(paths).await;
            }
            _ => unreachable!(),
        }
    }
}

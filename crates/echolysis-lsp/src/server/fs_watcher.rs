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
        // TODO
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
        // TODO
    }

    pub(super) async fn clear(&self) {
        self.log_info("unwatching all folders").await;
        self.fs_watcher.clear();
        // TODO
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

        self.log_info(format!("fs event: {event:?}")).await;

        match event.kind {
            // Full update
            EventKind::Create(_)
            | EventKind::Modify(ModifyKind::Data(_))
            | EventKind::Modify(ModifyKind::Metadata(MetadataKind::WriteTime)) => {
                let paths: Vec<_> = event.paths.iter().filter(|path| path.is_file()).collect();
                if paths.is_empty() {
                    return;
                }
                self.log_info(format!("insert files: {:?}", paths)).await;
            }
            EventKind::Remove(_) => {
                self.log_info(format!("remove files: {:?}", event.paths))
                    .await;
            }
            _ => (),
        }
    }
}

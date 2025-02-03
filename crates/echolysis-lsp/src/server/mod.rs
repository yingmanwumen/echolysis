use std::{sync::Arc, time::Duration};

use dashmap::{DashMap, DashSet};
use fs_watcher::FsWatcher;
use router::Router;
use tower_lsp::lsp_types;

mod diagnostic;
mod fs_watcher;
mod language_server;
mod on_event;
mod router;
mod utils;

pub struct Server {
    client: tower_lsp::Client,
    router: Router,

    diagnostics_record: DashSet<lsp_types::Url>,
    duplicate_locations: parking_lot::Mutex<Vec<Vec<lsp_types::Location>>>,

    /// K: file path, V: language id
    file_map: DashMap<lsp_types::Url, &'static str>,
    fs_watcher: FsWatcher,
}

impl Server {
    fn create_fs_watcher() -> (
        FsWatcher,
        tokio::sync::mpsc::Receiver<Result<notify::Event, notify::Error>>,
    ) {
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        let tx_clone = tx.clone();
        let watcher = FsWatcher::new::<notify::RecommendedWatcher>(
            Duration::from_millis(500),
            Box::new(move |evt| {
                let _ = tx_clone.blocking_send(evt);
            }),
        );
        (watcher, rx)
    }

    fn run_fs_event_loop(
        server: Arc<Server>,
        mut rx: tokio::sync::mpsc::Receiver<Result<notify::Event, notify::Error>>,
    ) {
        tokio::spawn(async move {
            while let Some(evt) = rx.recv().await {
                server.handle_fs_event(evt).await;
            }
        });
    }

    pub fn new(client: tower_lsp::Client) -> Arc<Self> {
        let (fs_watcher, fs_evt_rx) = Self::create_fs_watcher();

        let server = Arc::new(Self {
            client,
            fs_watcher,
            router: Router::new(),
            file_map: DashMap::default(),
            diagnostics_record: DashSet::default(),
            duplicate_locations: parking_lot::Mutex::new(vec![]),
        });

        Self::run_fs_event_loop(server.clone(), fs_evt_rx);

        server
    }
}

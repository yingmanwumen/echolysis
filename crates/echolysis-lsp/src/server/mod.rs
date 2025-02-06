use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use dashmap::{DashMap, DashSet};
use fs_watcher::FsWatcher;
use router::Router;
use tower_lsp::lsp_types::{self, MessageType};

mod diagnostic;
mod fs_watcher;
mod language_server;
mod on_event;
mod router;
mod utils;

pub struct Server {
    client: tower_lsp::Client,
    router: Router,

    diagnostics_uri_record: DashSet<lsp_types::Url>,
    duplicate_locations: parking_lot::Mutex<Vec<Vec<lsp_types::Location>>>,

    /// K: file path, V: language id
    file_map: DashMap<lsp_types::Url, &'static str>,
    fs_watcher: FsWatcher,

    stopped: AtomicBool,
}

impl Server {
    fn create_fs_watcher() -> Result<
        (
            FsWatcher,
            tokio::sync::mpsc::Receiver<Result<notify::Event, notify::Error>>,
        ),
        notify::Error,
    > {
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        let tx_clone = tx.clone();
        let watcher = FsWatcher::new::<notify::RecommendedWatcher>(
            Duration::from_millis(500),
            Box::new(move |evt| {
                let _ = tx_clone.blocking_send(evt);
            }),
        )?;
        Ok((watcher, rx))
    }

    fn run_fs_event_loop(
        server: Arc<Server>,
        mut rx: tokio::sync::mpsc::Receiver<Result<notify::Event, notify::Error>>,
    ) {
        tokio::spawn(async move {
            while let Some(evt) = rx.recv().await {
                if !server.is_stopped() {
                    server.handle_fs_event(evt).await;
                }
            }
        });
    }

    pub fn new(client: tower_lsp::Client) -> Arc<Self> {
        let (fs_watcher, fs_evt_rx) = match Self::create_fs_watcher() {
            Ok((watcher, rx)) => (watcher, rx),
            Err(e) => {
                futures::executor::block_on(client.show_message(
                    MessageType::ERROR,
                    format!("Failed to create fs watcher: {}", e),
                ));
                panic!();
            }
        };

        let server = Arc::new(Self {
            client,
            fs_watcher,
            router: Router::new(),
            file_map: DashMap::default(),
            diagnostics_uri_record: DashSet::default(),
            duplicate_locations: parking_lot::Mutex::new(vec![]),
            stopped: AtomicBool::new(false),
        });

        Self::run_fs_event_loop(server.clone(), fs_evt_rx);

        server
    }

    pub async fn clear(&self) {
        self.fs_watcher.clear();
        self.file_map.clear();
        self.router.clear(); // Clear router before clear diagnostics
        self.push_diagnostic().await;
    }

    pub async fn stop(&self) {
        self.stopped
            .store(true, std::sync::atomic::Ordering::SeqCst);
        self.clear().await;
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped.load(std::sync::atomic::Ordering::SeqCst)
    }
}

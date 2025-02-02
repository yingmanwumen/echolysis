use std::{
    hash::{Hash, Hasher},
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use echolysis_core::utils::hash::{ADashMap, FxDashSet};
use fs_watcher::FsWatcher;
use router::Router;
use tower_lsp::lsp_types;

mod diagnostic;
mod fs_watcher;
mod language_server;
mod log;
mod on_event;
mod router;
mod utils;

#[derive(Debug, Clone, Eq)]
struct LocationRange {
    uri: lsp_types::Url,
    range: lsp_types::Range,
}

impl PartialEq for LocationRange {
    fn eq(&self, other: &Self) -> bool {
        self.uri == other.uri && self.range == other.range
    }
}

impl Hash for LocationRange {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uri.as_str().hash(state);
        self.range.start.line.hash(state);
        self.range.start.character.hash(state);
        self.range.end.line.hash(state);
        self.range.end.character.hash(state);
    }
}

pub struct Server {
    client: tower_lsp::Client,
    fs_watcher: FsWatcher,
    stopped: AtomicBool,
    #[allow(unused)]
    router: Router,
    file_map: ADashMap<PathBuf, &'static str>,
    diagnostics_record: FxDashSet<lsp_types::Url>,
    duplicate_locations: ADashMap<LocationRange, Vec<lsp_types::Location>>,
}

// Helper struct to hold position information
#[derive(Debug)]
struct TSRange {
    start: echolysis_core::tree_sitter::Point,
    end: echolysis_core::tree_sitter::Point,
}

impl Server {
    fn create_fs_watcher() -> (
        FsWatcher,
        tokio::sync::mpsc::Receiver<Result<notify::Event, notify::Error>>,
    ) {
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        let tx_clone = tx.clone();

        let watcher = FsWatcher::new::<notify::PollWatcher>(
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
                if server.is_stopped() {
                    break;
                }
                server.handle_fs_event(evt).await;
            }
        });
    }

    pub fn new(client: tower_lsp::Client) -> Arc<Self> {
        let (fs_watcher, fs_evt_rx) = Self::create_fs_watcher();

        let server = Arc::new(Self {
            client,
            fs_watcher,
            stopped: AtomicBool::new(false),
            router: Router::new(),
            file_map: ADashMap::default(),
            diagnostics_record: FxDashSet::default(),
            duplicate_locations: ADashMap::default(),
        });

        Self::run_fs_event_loop(server.clone(), fs_evt_rx);

        server
    }

    fn is_stopped(&self) -> bool {
        self.stopped.load(std::sync::atomic::Ordering::SeqCst)
    }

    async fn stop(&self) {
        self.stopped
            .store(true, std::sync::atomic::Ordering::SeqCst);
        self.clear().await;
    }
}

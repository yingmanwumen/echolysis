use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use fs_watcher::FsWatcher;

mod fs_watcher;
mod language_server;
mod log;

pub struct Server {
    client: tower_lsp::Client,
    fs_watcher: FsWatcher,
    stopped: AtomicBool,
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

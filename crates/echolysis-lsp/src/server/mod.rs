use std::{
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use echolysis_core::utils::hash::{ADashMap, FxDashSet};
use fs_watcher::FsWatcher;
use rayon::prelude::*;
use router::Router;
use tower_lsp::lsp_types;

mod fs_watcher;
mod language_server;
mod log;
mod on_event;
mod router;

pub struct Server {
    client: tower_lsp::Client,
    fs_watcher: FsWatcher,
    stopped: AtomicBool,
    #[allow(unused)]
    router: Router,
    file_map: ADashMap<PathBuf, &'static str>,
    diagnostics_record: FxDashSet<lsp_types::Url>,
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

    async fn push_diagnostic(&self) {
        let duplicates: Vec<_> = self
            .router
            .engines()
            .par_iter()
            .map(|engine| {
                let duplicates = engine.detect_duplicates();
                (
                    engine.key().clone(),
                    duplicates
                        .into_iter()
                        .map(|group| {
                            group
                                .into_iter()
                                .filter_map(|id| {
                                    let node = engine.get_node_by_id(id)?;
                                    let start = node.start_position();
                                    let end = node.end_position();
                                    Some((engine.get_path_by_id(id)?, (start, end)))
                                })
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
        self.log_info(format!("publishing: {:?}", duplicates)).await;

        let mut diagnostics_map: std::collections::HashMap<
            lsp_types::Url,
            Vec<lsp_types::Diagnostic>,
        > = std::collections::HashMap::new();

        for (_, groups) in duplicates {
            for group in groups {
                for (file, (start, end)) in group {
                    self.log_info(format!(
                        "{:?}",
                        lsp_types::Url::from_file_path(file.as_str())
                    ))
                    .await;
                    if let Ok(uri) = lsp_types::Url::from_file_path(file.as_str()) {
                        let diagnostics = diagnostics_map.entry(uri).or_default();
                        diagnostics.push(lsp_types::Diagnostic {
                            range: lsp_types::Range {
                                start: lsp_types::Position::new(
                                    start.row as u32,
                                    start.column as u32,
                                ),
                                end: lsp_types::Position::new(end.row as u32, end.column as u32),
                            },
                            severity: Some(lsp_types::DiagnosticSeverity::WARNING),
                            source: Some("echolysis".to_string()),
                            message: format!(
                                "Duplicated code fragment found, {} lines",
                                end.row - start.row + 1
                            ),
                            ..Default::default()
                        });
                    }
                }
            }
        }

        if diagnostics_map.is_empty() {
            for item in self.diagnostics_record.iter() {
                self.client
                    .publish_diagnostics(item.key().clone(), vec![], None)
                    .await;
            }
            self.diagnostics_record.clear();
            return;
        }

        for (uri, diagnostics) in diagnostics_map {
            self.diagnostics_record.insert(uri.clone());
            self.client
                .publish_diagnostics(uri, diagnostics, None)
                .await;
        }
    }
}

use std::sync::Arc;

use tower_lsp::{
    jsonrpc,
    lsp_types::{self, GotoDefinitionParams, GotoDefinitionResponse},
    LanguageServer,
};

use super::Server;

#[tower_lsp::async_trait]
impl LanguageServer for Server {
    async fn initialize(
        &self,
        params: lsp_types::InitializeParams,
    ) -> jsonrpc::Result<lsp_types::InitializeResult> {
        self.stopped
            .store(false, std::sync::atomic::Ordering::SeqCst);

        self.watch(&params.workspace_folders.unwrap_or_default())
            .await;

        Ok(lsp_types::InitializeResult {
            capabilities: lsp_types::ServerCapabilities {
                text_document_sync: Some(lsp_types::TextDocumentSyncCapability::Options(
                    lsp_types::TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(lsp_types::TextDocumentSyncKind::FULL),
                        ..Default::default()
                    },
                )),
                definition_provider: Some(lsp_types::OneOf::Left(true)),
                workspace: Some(lsp_types::WorkspaceServerCapabilities {
                    workspace_folders: Some(lsp_types::WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(lsp_types::OneOf::Left(true)),
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            server_info: Some(lsp_types::ServerInfo {
                name: "echolysis-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        self.stop().await;
        Ok(())
    }

    async fn did_open(&self, params: lsp_types::DidOpenTextDocumentParams) {
        self.on_insert(&[(
            params.text_document.uri,
            Some(Arc::new(params.text_document.text)),
        )])
        .await;
    }

    async fn did_close(&self, params: lsp_types::DidCloseTextDocumentParams) {
        self.clear_diagnostic(&[params.text_document.uri], None)
            .await;
    }

    async fn did_change(&self, params: lsp_types::DidChangeTextDocumentParams) {
        static LAST_CHANGE: parking_lot::Mutex<Option<lsp_types::DidChangeTextDocumentParams>> =
            parking_lot::Mutex::new(None);
        static LAST_CHANGE_TIME: parking_lot::Mutex<Option<std::time::Instant>> =
            parking_lot::Mutex::new(None);
        let uri = params.text_document.uri.clone();
        let last = LAST_CHANGE.lock().replace(params);
        match last {
            Some(mut last) if last.text_document.uri != uri => {
                self.on_insert(&[(
                    last.text_document.uri,
                    Some(Arc::new(last.content_changes.swap_remove(0).text)),
                )])
                .await;
            }
            _ => (),
        }
        LAST_CHANGE_TIME.lock().replace(std::time::Instant::now());

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let last_time = *LAST_CHANGE_TIME.lock();
        match last_time {
            None => (),
            Some(last_time) if last_time.elapsed().as_millis() < 500 => (),
            _ => {
                let last = LAST_CHANGE.lock().take();
                if let Some(mut last) = last {
                    self.on_insert(&[(
                        last.text_document.uri,
                        Some(Arc::new(last.content_changes.swap_remove(0).text)),
                    )])
                    .await;
                }
            }
        }
    }

    async fn did_change_workspace_folders(
        &self,
        params: lsp_types::DidChangeWorkspaceFoldersParams,
    ) {
        self.unwatch(&params.event.removed).await;
        self.watch(&params.event.added).await;
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> jsonrpc::Result<Option<GotoDefinitionResponse>> {
        if self.is_stopped() {
            return Err(jsonrpc::Error::invalid_request());
        }

        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let is_inside = |location: &lsp_types::Location,
                         uri: &lsp_types::Url,
                         position: &lsp_types::Position|
         -> bool {
            location.uri == *uri
                && location.range.start.line <= position.line
                && position.line <= location.range.end.line
                && (position.line != location.range.start.line
                    || position.character >= location.range.start.character)
                && (position.line != location.range.end.line
                    || position.character <= location.range.end.character)
        };

        // Find a range that contains the clicked position
        let matching_locations = self
            .duplicate_locations
            .lock()
            .iter()
            .find_map(|locations| {
                let from = locations
                    .iter()
                    .find(|location| is_inside(location, &uri, &position))?;
                Some(
                    locations
                        .iter()
                        .filter(|to| *to != from)
                        .cloned()
                        .collect::<Vec<_>>(),
                )
            });

        if let Some(locations) = matching_locations {
            if !locations.is_empty() {
                return Ok(Some(GotoDefinitionResponse::Array(locations.clone())));
            }
        }
        Ok(None)
    }
}

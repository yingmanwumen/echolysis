use tower_lsp::{jsonrpc, lsp_types, LanguageServer};

use super::Server;

#[tower_lsp::async_trait]
impl LanguageServer for Server {
    async fn initialize(
        &self,
        params: lsp_types::InitializeParams,
    ) -> jsonrpc::Result<lsp_types::InitializeResult> {
        self.log_info(format!(
            "root_dir: {:?}, workspace_dirs: {:?}",
            params.root_uri, params.workspace_folders
        ))
        .await;

        Ok(lsp_types::InitializeResult {
            capabilities: lsp_types::ServerCapabilities {
                text_document_sync: Some(lsp_types::TextDocumentSyncCapability::Options(
                    lsp_types::TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(lsp_types::TextDocumentSyncKind::INCREMENTAL),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            },
            server_info: Some(lsp_types::ServerInfo {
                name: "echolysis-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: lsp_types::DidOpenTextDocumentParams) {
        self.log_info(format!("textDocument/didOpen: {params:?}"))
            .await;
    }

    async fn did_change(&self, params: lsp_types::DidChangeTextDocumentParams) {
        self.log_info(format!("textDocument/didChange: {params:?}"))
            .await;
    }

    async fn diagnostic(
        &self,
        params: lsp_types::DocumentDiagnosticParams,
    ) -> jsonrpc::Result<lsp_types::DocumentDiagnosticReportResult> {
        self.log_info(format!("textDocument/diagnostics: {params:?}"))
            .await;
        Err(jsonrpc::Error::method_not_found())
    }

    async fn workspace_diagnostic(
        &self,
        params: lsp_types::WorkspaceDiagnosticParams,
    ) -> jsonrpc::Result<lsp_types::WorkspaceDiagnosticReportResult> {
        self.log_info(format!("workspace/diagnostics: {params:?}"))
            .await;
        Err(jsonrpc::Error::method_not_found())
    }

    async fn did_change_workspace_folders(
        &self,
        params: lsp_types::DidChangeWorkspaceFoldersParams,
    ) {
        self.log_info(format!("workspace/didChangeWorkspaceFolders: {params:?}"))
            .await;
    }

    async fn did_create_files(&self, params: lsp_types::CreateFilesParams) {
        self.log_info(format!("workspace/didCreateFiles: {params:?}"))
            .await;
    }

    async fn did_rename_files(&self, params: lsp_types::RenameFilesParams) {
        self.log_info(format!("workspace/didRenameFiles: {params:?}"))
            .await;
    }

    async fn did_delete_files(&self, params: lsp_types::DeleteFilesParams) {
        self.log_info(format!("workspace/didDeleteFiles: {params:?}"))
            .await;
    }
}

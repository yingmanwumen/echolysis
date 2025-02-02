use tower_lsp::lsp_types;

use super::Server;

#[allow(unused)]
impl Server {
    pub(super) async fn log_info<M: std::fmt::Display>(&self, message: M) {
        self.client
            .log_message(
                lsp_types::MessageType::INFO,
                format!("[echolysis] {message}"),
            )
            .await;
    }

    pub(super) async fn log_error<M: std::fmt::Display>(&self, message: M) {
        self.client
            .log_message(
                lsp_types::MessageType::ERROR,
                format!("[echolysis] {message}"),
            )
            .await;
    }

    pub(super) async fn log_warn<M: std::fmt::Display>(&self, message: M) {
        self.client
            .log_message(
                lsp_types::MessageType::WARNING,
                format!("[echolysis] {message}"),
            )
            .await;
    }
}

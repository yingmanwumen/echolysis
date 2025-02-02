#![allow(unused)]

use tower_lsp::lsp_types;

use super::Server;

impl Server {
    /// Logs an info message to the LSP client
    ///
    /// # Arguments
    /// * `message` - The message to log
    pub(super) async fn log_info<M: std::fmt::Display>(&self, message: M) {
        self.client
            .log_message(
                lsp_types::MessageType::INFO,
                format!("[echolysis] {message}"),
            )
            .await;
    }

    /// Logs an error message to the LSP client
    ///
    /// # Arguments
    /// * `message` - The error message to log
    pub(super) async fn log_error<M: std::fmt::Display>(&self, message: M) {
        self.client
            .log_message(
                lsp_types::MessageType::ERROR,
                format!("[echolysis] {message}"),
            )
            .await;
    }

    /// Logs a warning message to the LSP client
    ///
    /// # Arguments
    /// * `message` - The warning message to log
    pub(super) async fn log_warn<M: std::fmt::Display>(&self, message: M) {
        self.client
            .log_message(
                lsp_types::MessageType::WARNING,
                format!("[echolysis] {message}"),
            )
            .await;
    }
}

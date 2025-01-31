mod language_server;

use tower_lsp::lsp_types;

pub struct Server {
    client: tower_lsp::Client,
}

#[allow(unused)]
impl Server {
    pub fn new(client: tower_lsp::Client) -> Self {
        Self { client }
    }

    async fn log_info<M: std::fmt::Display>(&self, message: M) {
        self.client
            .log_message(
                lsp_types::MessageType::INFO,
                format!("[echolysis] {message}"),
            )
            .await;
    }

    async fn log_error<M: std::fmt::Display>(&self, message: M) {
        self.client
            .log_message(
                lsp_types::MessageType::ERROR,
                format!("[echolysis] {message}"),
            )
            .await;
    }

    async fn log_warn<M: std::fmt::Display>(&self, message: M) {
        self.client
            .log_message(
                lsp_types::MessageType::WARNING,
                format!("[echolysis] {message}"),
            )
            .await;
    }
}

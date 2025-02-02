use server::Server;

mod server;

#[global_allocator]
static GLOBAL: rpmalloc::RpMalloc = rpmalloc::RpMalloc;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = tower_lsp::LspService::new(Server::new);
    tower_lsp::Server::new(stdin, stdout, socket)
        .serve(service)
        .await;
}

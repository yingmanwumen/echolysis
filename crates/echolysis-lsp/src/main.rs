use server::Server;

mod server;

#[global_allocator]
static GLOBAL: rpmalloc::RpMalloc = rpmalloc::RpMalloc;

#[tokio::main]
async fn main() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(8) // TODO: Make it configurable
        .build_global()
        .unwrap();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = tower_lsp::LspService::new(Server::new);
    tower_lsp::Server::new(stdin, stdout, socket)
        .serve(service)
        .await;
}

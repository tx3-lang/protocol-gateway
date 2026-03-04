mod config;
mod registry;
mod rpc;
mod server;

use std::sync::Arc;

use config::Config;
use registry::TiiRegistry;
use rpc::handler::AppState;

#[tokio::main]
async fn main() {
    let config = Config::from_env();

    let registry = TiiRegistry::load_dir(&config.protocols_dir).unwrap_or_else(|e| {
        eprintln!("failed to load protocols: {e}");
        std::process::exit(1);
    });

    let state = Arc::new(AppState {
        registry,
        trp_override: config.trp_override,
        trp_headers: config.trp_headers,
        network: config.network,
    });

    let router = server::build_router(state);

    if let Err(e) = server::serve(router, config.port).await {
        eprintln!("server error: {e}");
        std::process::exit(1);
    }
}

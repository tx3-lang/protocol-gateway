mod config;
mod registry;
mod rpc;
mod server;

use std::sync::Arc;

use config::{Config, trp_options_for_network};
use registry::TiiRegistry;
use rpc::handler::AppState;
use tx3_sdk::trp::Client;

#[tokio::main]
async fn main() {
    let config = Config::from_env();

    let registry = TiiRegistry::load_dir(&config.protocols_dir).unwrap_or_else(|e| {
        eprintln!("failed to load protocols: {e}");
        std::process::exit(1);
    });

    let trp_options =
        trp_options_for_network(&config.network, &config.trp_override, &config.trp_headers)
            .unwrap_or_else(|| {
                eprintln!(
                    "no TRP endpoint configured for network '{}'",
                    config.network
                );
                std::process::exit(1);
            });

    let trp_client = Client::new(trp_options);

    let state = Arc::new(AppState {
        registry,
        trp_client,
        network: config.network,
    });

    let router = server::build_router(state);

    if let Err(e) = server::serve(router, config.port).await {
        eprintln!("server error: {e}");
        std::process::exit(1);
    }
}

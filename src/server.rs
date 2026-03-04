use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

use crate::rpc::discover;
use crate::rpc::dispatcher;
use crate::rpc::handler::AppState;

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(dispatcher::dispatch))
        .route("/openrpc.json", get(discover::openrpc_handler))
        .route("/docs", get(discover::docs_redirect))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
}

pub async fn serve(router: Router, port: u16) -> Result<(), std::io::Error> {
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;
    println!("listening on {addr}");
    axum::serve(listener, router).await
}

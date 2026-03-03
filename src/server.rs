use axum::routing::post;
use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::rpc::dispatcher;
use crate::rpc::handler::AppState;

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(dispatcher::dispatch))
        .with_state(state)
}

pub async fn serve(router: Router, port: u16) -> Result<(), std::io::Error> {
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;
    println!("listening on {addr}");
    axum::serve(listener, router).await
}

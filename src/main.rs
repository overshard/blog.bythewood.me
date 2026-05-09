mod app;
mod error;
mod markdown;
mod middleware;
mod pdf;
mod posts;
mod render;
mod routes;
mod templates;

use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let state = app::AppState::from_env();
    let router = app::router(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    eprintln!("blog listening on http://{addr}");
    axum::serve(listener, router).await.unwrap();
}

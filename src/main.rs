use axum::{
    Router,
    routing::{get, post},
};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod converter;
mod handlers;

use handlers::{convert_handler, health_handler, root_handler};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    // 1. Initialize Logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tex2pdf=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 2. Create the router
    let app = Router::new()
        .route("/root", get(root_handler))
        .route("/healthz", get(health_handler))
        .route("/convert", post(convert_handler))
        .layer(TraceLayer::new_for_http());

    let addr = "0.0.0.0:3000";
    tracing::info!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

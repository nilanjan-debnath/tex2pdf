use axum::{
    routing::get,
    Router,
    response::{Json, IntoResponse},
    http::StatusCode,
};
use serde_json::json;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    // 1. Initialize Logging
    // This sets up "env_filter" which reads the RUST_LOG environment variable.
    // If RUST_LOG is not set, it defaults to "tex2pdf=debug,tower_http=debug".
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tex2pdf=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 2. Create the router and define the endpoints
    let app = Router::new()
        .route("/root", get(root_handler))
        .route("/healthz", get(health_handler))
        // 3. Add the middleware layer to log requests
        .layer(TraceLayer::new_for_http());

    // 4. Define the address to listen on (localhost:3000)
    let addr = "0.0.0.0:3000";
    
    // We can now use tracing::info! instead of println!
    tracing::info!("Server running on http://{}", addr);

    // 5. Create a TCP listener
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    // 6. Start the server
    axum::serve(listener, app).await.unwrap();
}

// Handler for GET /root
async fn root_handler() -> impl IntoResponse {
    // You can also add manual logs inside handlers if you want specific details
    tracing::debug!("Root handler accessed");

    Json(json!({
        "name": "tex2pdf API",
        "version": "0.1.0",
        "description": "API for converting Tex to PDF"
    }))
}

// Handler for GET /healthz
async fn health_handler() -> impl IntoResponse {
    tracing::debug!("Health handler accessed");
    (StatusCode::OK, Json(json!({
        "status": "ok",
    })))
}
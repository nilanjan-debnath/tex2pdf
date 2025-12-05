use axum::{
    routing::get,
    Router,
    response::{Json, IntoResponse},
    http::StatusCode,
};
use serde_json::json;

#[tokio::main]
async fn main() {
    // 1. Create the router and define the endpoints
    let app = Router::new()
        .route("/root", get(root_handler))
        .route("/healthz", get(health_handler));

    // 2. Define the address to listen on (localhost:3000)
    let addr = "0.0.0.0:3000";
    println!("Server running on http://{}", addr);

    // 3. Create a TCP listener
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    // 4. Start the server
    axum::serve(listener, app).await.unwrap();
}

// Handler for GET /root
async fn root_handler() -> impl IntoResponse {
    // Returns a simple JSON welcome message
    Json(json!({
        "name": "tex2pdf API",
        "version": "0.1.0",
        "description": "API for converting Tex to PDF"
    }))
}

// Handler for GET /healthz
// This is commonly used by load balancers (like Kubernetes or AWS) to check if the app is alive
async fn health_handler() -> impl IntoResponse {
    // Returns a 200 OK status code with a simple json message
    (StatusCode::OK, Json(json!({
        "status": "ok",
    })))
}
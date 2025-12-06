use axum::{
    extract::Multipart,
    routing::{get, post},
    Router,
    response::{Json, IntoResponse, Response},
    http::{StatusCode, header},
    body::Body,
};
use serde_json::json;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod converter;

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

// --- HANDLERS ---

async fn root_handler() -> impl IntoResponse {
    tracing::debug!("Root handler accessed");
    Json(json!({
        "name": "tex2pdf API",
        "version": "0.1.0",
        "description": "API for converting Tex to PDF"
    }))
}

async fn health_handler() -> impl IntoResponse {
    tracing::debug!("Health handler accessed - running TeX compilation check");
    
    // Simple test LaTeX document to verify tectonic is working
    let test_tex = r#"\documentclass{article}
\begin{document}
Health check passed.
\end{document}"#.to_string();
    
    let start = std::time::Instant::now();
    
    // Run conversion in a blocking thread
    let result = tokio::task::spawn_blocking(move || {
        converter::convert_tex_to_pdf(test_tex)
    }).await;
    
    let elapsed_ms = start.elapsed().as_millis();
    
    match result {
        Err(e) => {
            tracing::error!("Health check failed - task error: {}", e);
            (StatusCode::SERVICE_UNAVAILABLE, Json(json!({
                "status": "unhealthy",
                "error": format!("Task execution failed: {}", e),
                "check": "tex_compilation"
            })))
        },
        Ok(converter_result) => match converter_result {
            Ok(pdf_bytes) => {
                tracing::info!("Health check passed - compiled {} bytes in {}ms", pdf_bytes.len(), elapsed_ms);
                (StatusCode::OK, Json(json!({
                    "status": "ok",
                    "check": "tex_compilation",
                    "pdf_size_bytes": pdf_bytes.len(),
                    "conversion_time_ms": elapsed_ms
                })))
            },
            Err(err_msg) => {
                tracing::error!("Health check failed - conversion error: {}", err_msg);
                (StatusCode::SERVICE_UNAVAILABLE, Json(json!({
                    "status": "error",
                    "error": err_msg,
                    "check": "tex_compilation"
                })))
            }
        }
    }
}

async fn convert_handler(mut multipart: Multipart) -> Response {
    tracing::debug!("Convert handler accessed");

    let mut tex_content = String::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.text().await {
            Ok(text) => {
                tex_content = text;
                break; 
            }
            Err(e) => {
                tracing::error!("Failed to read field text: {}", e);
                return (StatusCode::BAD_REQUEST, "Failed to read upload").into_response();
            }
        }
    }

    if tex_content.is_empty() {
        return (StatusCode::BAD_REQUEST, "No content found in upload").into_response();
    }

    // Run conversion in a blocking thread
    let result = tokio::task::spawn_blocking(move || {
        converter::convert_tex_to_pdf(tex_content)
    }).await;

    match result {
        Err(e) => {
            tracing::error!("Conversion task failed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal processing error").into_response()
        },
        Ok(converter_result) => match converter_result {
            Ok(pdf_bytes) => {
                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "application/pdf")
                    .header(header::CONTENT_DISPOSITION, "attachment; filename=\"output.pdf\"")
                    .body(Body::from(pdf_bytes))
                    .unwrap()
            },
            Err(err_msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, err_msg).into_response()
            }
        }
    }
}
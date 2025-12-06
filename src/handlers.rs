use axum::{
    body::Body,
    extract::Multipart,
    http::{StatusCode, header},
    response::{IntoResponse, Json, Response},
};
use serde_json::json;

use crate::converter;

pub async fn root_handler() -> impl IntoResponse {
    tracing::debug!("Root handler accessed");
    Json(json!({
        "name": "tex2pdf API",
        "version": "0.1.0",
        "description": "API for converting Tex to PDF"
    }))
}

pub async fn health_handler() -> impl IntoResponse {
    tracing::debug!("Health handler accessed - running TeX compilation check");

    // Simple test LaTeX document to verify tectonic is working
    let test_tex = r#"\documentclass{article}
\begin{document}
Health check passed.
\end{document}"#
        .to_string();

    let start = std::time::Instant::now();

    // Run conversion in a blocking thread
    let result = tokio::task::spawn_blocking(move || converter::convert_tex_to_pdf(test_tex)).await;

    let elapsed_ms = start.elapsed().as_millis();

    match result {
        Err(e) => {
            tracing::error!("Health check failed - task error: {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "unhealthy",
                    "error": format!("Task execution failed: {}", e),
                    "check": "tex_compilation"
                })),
            )
        }
        Ok(converter_result) => match converter_result {
            Ok(pdf_bytes) => {
                tracing::info!(
                    "Health check passed - compiled {} bytes in {}ms",
                    pdf_bytes.len(),
                    elapsed_ms
                );
                (
                    StatusCode::OK,
                    Json(json!({
                        "status": "ok",
                        "check": "tex_compilation",
                        "pdf_size_bytes": pdf_bytes.len(),
                        "conversion_time_ms": elapsed_ms
                    })),
                )
            }
            Err(err_msg) => {
                tracing::error!("Health check failed - conversion error: {}", err_msg);
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(json!({
                        "status": "error",
                        "error": err_msg,
                        "check": "tex_compilation"
                    })),
                )
            }
        },
    }
}

pub async fn convert_handler(mut multipart: Multipart) -> Response {
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
    let result =
        tokio::task::spawn_blocking(move || converter::convert_tex_to_pdf(tex_content)).await;

    match result {
        Err(e) => {
            tracing::error!("Conversion task failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal processing error",
            )
                .into_response()
        }
        Ok(converter_result) => match converter_result {
            Ok(pdf_bytes) => Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/pdf")
                .header(
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"output.pdf\"",
                )
                .body(Body::from(pdf_bytes))
                .unwrap(),
            Err(err_msg) => (StatusCode::INTERNAL_SERVER_ERROR, err_msg).into_response(),
        },
    }
}

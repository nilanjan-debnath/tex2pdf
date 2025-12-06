use tectonic::latex_to_pdf;

/// Converts a LaTeX string into PDF bytes.
/// Returns a Result containing the PDF bytes or an error message.
pub fn convert_tex_to_pdf(tex_source: String) -> Result<Vec<u8>, String> {
    tracing::debug!("Starting Tectonic conversion...");

    // tectonic::latex_to_pdf is a high-level helper that compiles the string
    // and returns the binary PDF data.
    match latex_to_pdf(tex_source) {
        Ok(pdf_data) => {
            tracing::debug!(
                "Tectonic conversion successful. PDF size: {} bytes",
                pdf_data.len()
            );
            Ok(pdf_data)
        }
        Err(e) => {
            tracing::error!("Tectonic conversion failed: {}", e);
            Err(e.to_string())
        }
    }
}

use std::process::Command;
use tempfile::Builder;

/// Converts a LaTeX string into PDF bytes using tectonic CLI.
/// This approach is compatible with Alpine/musl by avoiding the
/// embedded tectonic library which has tokio runtime conflicts.
pub fn convert_tex_to_pdf(tex_source: String) -> Result<Vec<u8>, String> {
    tracing::debug!("Starting Tectonic CLI conversion...");

    // Create temp directory for both input and output
    let temp_dir = Builder::new()
        .prefix("tex2pdf_")
        .tempdir()
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;

    // Create input file with .tex extension (required by tectonic)
    let input_path = temp_dir.path().join("input.tex");
    std::fs::write(&input_path, tex_source.as_bytes())
        .map_err(|e| format!("Failed to write tex source: {}", e))?;

    // Run tectonic CLI - using simple mode which outputs PDF to same directory
    let output = Command::new("tectonic")
        .arg(&input_path)
        .arg("--outdir")
        .arg(temp_dir.path())
        .output()
        .map_err(|e| format!("Failed to run tectonic: {}", e))?;

    // Read the output PDF (same name as input but with .pdf extension)
    let output_pdf = temp_dir.path().join("input.pdf");

    // Check if PDF was created - this is the true success indicator
    // Tectonic writes notes/warnings to stderr even on success
    if output_pdf.exists() {
        let pdf_data =
            std::fs::read(&output_pdf).map_err(|e| format!("Failed to read output PDF: {}", e))?;

        tracing::debug!(
            "Tectonic CLI conversion successful. PDF size: {} bytes",
            pdf_data.len()
        );

        return Ok(pdf_data);
    }

    // If PDF doesn't exist, it's a real error
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Filter out "note:" lines which are just informational
    let error_lines: Vec<&str> = stderr
        .lines()
        .chain(stdout.lines())
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with("note:") && !trimmed.starts_with("warning:")
        })
        .collect();

    let error_msg = if error_lines.is_empty() {
        // No real errors, but PDF still not created - check if exit code was non-zero
        if !output.status.success() {
            format!(
                "Tectonic exited with code {:?} but no error message",
                output.status.code()
            )
        } else {
            "Tectonic completed but no PDF was generated".to_string()
        }
    } else {
        error_lines.join("\n")
    };

    tracing::error!("Tectonic CLI failed: {}", error_msg);
    Err(format!("Tectonic compilation failed: {}", error_msg))
}

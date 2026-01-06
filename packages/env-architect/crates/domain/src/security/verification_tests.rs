#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[tokio::test]
    async fn test_verify_binary_insecure_mocks() -> Result<()> {
        let verifier = VerificationService::new();
        // Create a temporary dummy file to verify
        let mut temp = tempfile::NamedTempFile::new()?;
        writeln!(temp, "test binary content")?;
        let path = temp.path();

        // Use the CLI mock strings
        let mock_sig = "SGVsbG8gV29ybGQ="; // "Hello World"
        let mock_cert = "SGVsbG8gQ2VydGlmaWNhdGU="; // "Hello Certificate"
        let mock_identity = "developer@architect.io";

        // This should return Ok(false) because verify_blob will error on invalid PEM format,
        // and we catch it to prevent crashes in the CLI flow.
        let result = verifier
            .verify_binary(path, mock_sig, mock_cert, mock_identity)
            .await?;

        assert_eq!(result, false, "Verification should fail for invalid mocks");

        Ok(())
    }

    #[tokio::test]
    async fn test_verify_binary_empty_inputs() {
        let verifier = VerificationService::new();
        let path = Path::new("does_not_exist"); // Shouldn't reach file read if args check works

        // Should bail early
        let result = verifier.verify_binary(path, "", "", "id").await;
        assert!(result.is_err());
    }
}

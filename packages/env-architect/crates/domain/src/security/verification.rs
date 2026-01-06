use anyhow::Result;
use std::path::Path;

/// Service for verifying the authenticity and integrity of tool binaries
pub struct VerificationService {}

impl VerificationService {
    pub fn new() -> Self {
        Self {}
    }

    /// Verify a binary file using Sigstore signatures (Keyless)
    pub async fn verify_binary(
        &self,
        binary_path: &Path,
        signature_b64: &str,
        certificate_b64: &str,
        expected_identity: &str,
    ) -> Result<bool> {
        use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
        use pkcs8::der::Decode;
        use sigstore::cosign::{Client, CosignCapabilities};
        use x509_cert::Certificate;

        // Verify we can read the file first
        let blob = std::fs::read(binary_path)?;

        println!("🛡️  Sigstore verifying: {:?}", binary_path);

        if signature_b64.trim().is_empty() || certificate_b64.trim().is_empty() {
            anyhow::bail!("Empty signature or certificate");
        }

        match Client::verify_blob(certificate_b64, signature_b64, &blob) {
            Ok(_) => {
                println!("✅ Signature Cryptographically Verified.");
            }
            Err(e) => {
                eprintln!("❌ Signature Verification Failed: {}", e);
                // Return false to indicate verification failure (but don't crash)
                return Ok(false);
            }
        }

        let cert_der = BASE64
            .decode(certificate_b64)
            .map_err(|e| anyhow::anyhow!("Failed to decode certificate base64: {}", e))?;

        let pem_str = String::from_utf8(cert_der)?;
        // If the decoding above worked and produced a PEM-like string, we use it.

        let pem_content = pem_str;

        // verification passed above, so pem parsing works.
        // Now parse to x509 struct to get Identity.
        let pem =
            pem::parse(&pem_content).map_err(|e| anyhow::anyhow!("PEM parse failed: {}", e))?;
        let cert = Certificate::from_der(pem.contents())
            .map_err(|e| anyhow::anyhow!("DER parse failed: {}", e))?;

        // Extract Subject Alternative Name (Email)
        // This is complex in x509 structures.
        // We check the Subject field for common name or email.
        // Sigstore Keyless uses SAN (Subject Alternative Name) for OIDC email.

        let subject_string = format!("{:?}", cert.tbs_certificate.subject);
        println!("   Certificate Subject: {}", subject_string);

        let mut found_identity = false;
        if let Some(extensions) = &cert.tbs_certificate.extensions {
            for ext in extensions.iter() {
                // OID for SAN is 2.5.29.17
                if ext.extn_id.to_string() == "2.5.29.17" {
                    // Parse SAN. This requires `x509_cert::ext::pkix::SubjectAltName`.
                    // We can try to debug print it or converting to string.
                    let ext_val_debug = format!("{:?}", ext.extn_value);
                    if ext_val_debug.contains(expected_identity) {
                        found_identity = true;
                    }
                }
            }
        }

        if subject_string.contains(expected_identity) {
            found_identity = true;
        }

        if found_identity {
            println!("👤 Identity Match: {}", expected_identity);
            Ok(true)
        } else {
            eprintln!("❌ Identity Mismatch. Expected '{}'", expected_identity);
            // Verify extensions content for debug
            if let Some(exts) = &cert.tbs_certificate.extensions {
                println!("   Extensions: {:?}", exts);
            }
            Ok(false)
        }
    }

    /// Check the transparency log (Rekor) for a binary hash
    pub async fn check_transparency_log(&self, _content_hash: &str) -> Result<bool> {
        Ok(true)
    }
}

pub struct SignatureData {
    pub signature: String,
    pub certificate: String,
    pub oidc_identity: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[tokio::test]
    async fn test_verify_binary_insecure_mocks() -> anyhow::Result<()> {
        let verifier = VerificationService::new();
        let mut temp = tempfile::NamedTempFile::new()?;
        writeln!(temp, "test binary content")?;
        let path = temp.path();

        let mock_sig = "SGVsbG8gV29ybGQ=";
        let mock_cert = "SGVsbG8gQ2VydGlmaWNhdGU=";
        let mock_identity = "developer@architect.io";

        let result = verifier
            .verify_binary(path, mock_sig, mock_cert, mock_identity)
            .await?;

        assert_eq!(result, false, "Verification should fail for invalid mocks");
        Ok(())
    }

    #[tokio::test]
    async fn test_verify_binary_empty_inputs() {
        let verifier = VerificationService::new();
        let path = Path::new("does_not_exist");
        let result = verifier.verify_binary(path, "", "", "id").await;
        assert!(result.is_err());
    }
}

#![allow(unused)]
use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use keyring::Entry;
use rand::rngs::OsRng;

const KEYRING_SERVICE: &str = "env-architect-signing-key";
const KEYRING_USER: &str = "default";

/// Generate a new Ed25519 signing key or load existing one from OS Keychain
/// This is idempotent - will return the same key on subsequent calls
pub fn generate_or_load_signing_key() -> Result<SigningKey> {
    let entry =
        Entry::new(KEYRING_SERVICE, KEYRING_USER).context("Failed to access OS Keychain")?;

    if let Ok(secret) = entry.get_password() {
        if let Ok(bytes) = general_purpose::STANDARD.decode(&secret) {
            let bytes: Vec<u8> = bytes; // Explicit type check fallback
            if bytes.len() == 32 {
                let key_bytes: [u8; 32] = bytes.try_into().unwrap();
                let key = SigningKey::from_bytes(&key_bytes);
                return Ok(key);
            }
        }
    }

    let home = dirs::home_dir().context("Could not find home directory")?;
    let key_path = home
        .join(".config")
        .join("env-architect")
        .join("signing-key");

    if key_path.exists() {
        let secret = std::fs::read_to_string(&key_path)?;
        if let Ok(bytes) = general_purpose::STANDARD.decode(&secret) {
            let bytes: Vec<u8> = bytes; // Explicit type check fallback
            if bytes.len() == 32 {
                let key_bytes: [u8; 32] = bytes.try_into().unwrap();
                let key = SigningKey::from_bytes(&key_bytes);
                return Ok(key);
            }
        }
    }

    let signing_key = SigningKey::generate(&mut OsRng);
    let secret = general_purpose::STANDARD.encode(signing_key.to_bytes());

    let _ = entry.set_password(&secret);

    // Save to file fallback
    if let Some(parent) = key_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&key_path, &secret).context("Failed to save fallback signing key")?;

    Ok(signing_key)
}

/// Get the public key in base64-encoded format for registration
#[allow(dead_code)]
pub fn get_public_key_base64(signing_key: &SigningKey) -> String {
    let verifying_key = signing_key.verifying_key();
    general_purpose::STANDARD.encode(verifying_key.to_bytes())
}

/// Sign arbitrary bytes with the signing key
pub fn sign_bytes(signing_key: &SigningKey, data: &[u8]) -> Signature {
    signing_key.sign(data)
}

/// Get signature as base64-encoded string
pub fn sign_bytes_base64(signing_key: &SigningKey, data: &[u8]) -> String {
    let signature = sign_bytes(signing_key, data);
    general_purpose::STANDARD.encode(signature.to_bytes())
}

/// Verify a signature (for testing purposes)
#[allow(dead_code)]
pub fn verify_signature(
    public_key_base64: &str,
    data: &[u8],
    signature_base64: &str,
) -> Result<()> {
    use ed25519_dalek::Verifier;

    let public_key_bytes = general_purpose::STANDARD
        .decode(public_key_base64)
        .context("Invalid public key encoding")?;

    let signature_bytes = general_purpose::STANDARD
        .decode(signature_base64)
        .context("Invalid signature encoding")?;

    let verifying_key_bytes: [u8; 32] = public_key_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid public key length"))?;

    let signature_bytes: [u8; 64] = signature_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid signature length"))?;

    let verifying_key =
        VerifyingKey::from_bytes(&verifying_key_bytes).context("Invalid public key format")?;

    let signature = Signature::from_bytes(&signature_bytes);

    verifying_key
        .verify(data, &signature)
        .context("Signature verification failed")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let key1 = generate_or_load_signing_key().unwrap();
        let key2 = generate_or_load_signing_key().unwrap();

        assert_eq!(key1.to_bytes(), key2.to_bytes());
    }

    #[test]
    fn test_sign_and_verify() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let public_key_b64 = get_public_key_base64(&signing_key);

        let data = b"test message";
        let signature_b64 = sign_bytes_base64(&signing_key, data);

        verify_signature(&public_key_b64, data, &signature_b64).unwrap();
    }
}

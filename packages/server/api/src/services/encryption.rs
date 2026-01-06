use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Result};
use rand::RngCore;
use std::env;

#[allow(dead_code)]
pub fn encrypt(plaintext: &str) -> Result<String> {
    let key_hex = env::var("ENCRYPTION_KEY").map_err(|_| anyhow!("ENCRYPTION_KEY not set"))?;
    let key_bytes =
        hex::decode(key_hex.trim()).map_err(|_| anyhow!("Invalid hex in ENCRYPTION_KEY"))?;

    if key_bytes.len() != 32 {
        return Err(anyhow!("ENCRYPTION_KEY must be 32 bytes (64 hex chars)"));
    }

    let cipher =
        Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| anyhow!("Cipher init failed: {}", e))?;

    let mut nonce_bytes = [0u8; 12];
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| anyhow!("Encryption failed: {}", e))?;

    // Prepend nonce to ciphertext
    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);

    Ok(hex::encode(combined))
}

#[allow(dead_code)]
pub fn decrypt(hex_ciphertext: &str) -> Result<String> {
    let key_hex = env::var("ENCRYPTION_KEY").map_err(|_| anyhow!("ENCRYPTION_KEY not set"))?;
    let key_bytes =
        hex::decode(key_hex.trim()).map_err(|_| anyhow!("Invalid hex in ENCRYPTION_KEY"))?;

    let combined =
        hex::decode(hex_ciphertext.trim()).map_err(|_| anyhow!("Invalid hex ciphertext"))?;
    if combined.len() < 12 {
        return Err(anyhow!("Ciphertext too short"));
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher =
        Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| anyhow!("Cipher init failed: {}", e))?;
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| anyhow!("Invalid UTF-8 after decryption: {}", e))
}

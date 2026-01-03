use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use crc32fast::Hasher;
use rand::Rng;

pub const KEY_PREFIX_LIVE: &str = "env_live_";
pub const KEY_PREFIX_TEST: &str = "env_test_";

/// Generates a new API Key with embedded checksum.
/// Format: prefix_entropy(32)_checksum(6)
pub fn generate_api_key(is_live: bool) -> (String, String) {
    let prefix = if is_live {
        KEY_PREFIX_LIVE
    } else {
        KEY_PREFIX_TEST
    };

    // 1. Generate 24 bytes of random data for entropy (results in ~32 base62 chars)
    // Actually, let's stick to simple alphanumeric for entropy to avoid confusion.
    // 32 chars of random alphanumeric.
    let entropy: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let partial_key = format!("{}{}", prefix, entropy);

    // 2. Compute Checksum (CRC32 of the entropy part)
    // Why entropy only? Or partial key?
    // Let's checksum the entropy.
    // Format: ..._entropychecksum
    let checksum = compute_checksum(&entropy);

    // 3. Final Key
    let full_key = format!("{}{}", partial_key, checksum); // e.g. env_live_abc...123456

    // 4. Hash it (Argon2)
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(full_key.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string();

    (full_key, password_hash)
}

/// Validates the format and checksum of a key WITHOUT database access.
/// Returns Ok(true) if valid, or Error if format is wrong.
pub fn validate_key_format(key: &str) -> bool {
    // 1. Check Prefix
    if !key.starts_with(KEY_PREFIX_LIVE) && !key.starts_with(KEY_PREFIX_TEST) {
        return false;
    }

    // 2. Extract Parts
    // Pattern: prefix(9) + entropy(32) + checksum(8 hex chars for CRC32?)
    // Wait, CRC32 is 4 bytes. Hex encoded is 8 chars.
    // Length check: 9 + 32 + 8 = 49 chars.
    // If we used a different encoding for checksum in generate, adjust here.
    // `compute_checksum` below needs to output fixed width.

    let parts: Vec<&str> = key.split('_').collect();
    if parts.len() < 3 {
        return false;
    } // env, live, [entropy+checksum]

    // Actually, `env_live_` is the prefix.
    // Let's strip prefix.
    let content = if let Some(stripped) = key.strip_prefix(KEY_PREFIX_LIVE) {
        stripped
    } else if let Some(stripped) = key.strip_prefix(KEY_PREFIX_TEST) {
        stripped
    } else {
        return false;
    };

    // Content is `entropy` + `checksum`
    // If entropy is 32 chars, and checksum is 8 chars (hex crc32).
    if content.len() != 40 {
        return false;
    }

    let (entropy, checksum) = content.split_at(32);

    let expected = compute_checksum(entropy);
    checksum == expected
}

fn compute_checksum(data: &str) -> String {
    let mut hasher = Hasher::new();
    hasher.update(data.as_bytes());
    let checksum = hasher.finalize();
    format!("{:08x}", checksum) // 8 char hex
}

/// Verifies a raw key against a stored hash.
pub fn verify_key_hash(raw_key: &str, stored_hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(stored_hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(raw_key.as_bytes(), &parsed_hash)
        .is_ok()
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthDeviceResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishPayload {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub ecosystem: String,
    pub purl: String,
    pub dependencies: Vec<DependencyPayload>,
    pub oci_reference: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DependencyPayload {
    pub purl: String,
    pub kind: String,
    pub req: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterKeyRequest {
    pub public_key: String, // base64-encoded Ed25519 public key (32 bytes)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterKeyResponse {
    pub success: bool,
    pub message: String,
}

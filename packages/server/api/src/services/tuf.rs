use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signer, SigningKey};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::collections::HashMap;

// TUF metadata structures - used for serialization
#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub struct SignedMetadata<T> {
    pub signatures: Vec<SignatureObj>,
    pub signed: T,
}

#[derive(Serialize, Deserialize)]
pub struct SignatureObj {
    pub keyid: String,
    pub sig: String,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub struct Root {
    pub _type: String,
    pub spec_version: String,
    pub version: i32,
    pub expires: DateTime<Utc>,
    pub keys: HashMap<String, Key>,
    pub roles: HashMap<String, Role>,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub struct Key {
    pub keytype: String, // "ed25519"
    pub scheme: String,  // "ed25519"
    pub keyval: KeyVal,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub struct KeyVal {
    pub public: String,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub struct Role {
    pub keyids: Vec<String>,
    pub threshold: i32,
}

#[derive(Serialize, Deserialize)]
pub struct Targets {
    pub _type: String,
    pub spec_version: String,
    pub version: i32,
    pub expires: DateTime<Utc>,
    pub targets: HashMap<String, TargetFile>,
}

#[derive(Serialize, Deserialize)]
pub struct TargetFile {
    pub length: i64,
    pub hashes: HashMap<String, String>, // "sha256": "..."
    pub custom: Option<HashMap<String, String>>, // e.g. "purl"
}

#[derive(Serialize, Deserialize)]
pub struct Snapshot {
    pub _type: String,
    pub spec_version: String,
    pub version: i32,
    pub expires: DateTime<Utc>,
    pub meta: HashMap<String, MetaFile>,
}

#[derive(Serialize, Deserialize)]
pub struct MetaFile {
    pub version: i32,
    pub length: Option<i64>,
    pub hashes: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize)]
pub struct Timestamp {
    pub _type: String,
    pub spec_version: String,
    pub version: i32,
    pub expires: DateTime<Utc>,
    pub meta: HashMap<String, MetaFile>,
}

pub struct TufService {
    db: Pool<Postgres>,
    signing_key: SigningKey,
    key_id: String,
}

impl TufService {
    pub fn new(db: Pool<Postgres>, _signing_key_pem: &str) -> Self {
        let signing_key = SigningKey::from_bytes(&[0u8; 32]); // TODO: Real key load

        Self {
            db,
            signing_key,
            key_id: "root-key".to_string(),
        }
    }

    pub async fn generate_targets(&self) -> Result<SignedMetadata<Targets>, anyhow::Error> {
        // Query DB
        let rows = sqlx::query!(
            r#"
            SELECT name, version, sha256, size_bytes 
            FROM components 
            WHERE sha256 IS NOT NULL AND size_bytes IS NOT NULL
            "#
        )
        .fetch_all(&self.db)
        .await?;

        let mut targets_map = HashMap::new();
        for row in rows {
            let filename = format!("{}-{}.wasm", row.name, row.version);
            let mut hashes = HashMap::new();
            hashes.insert("sha256".to_string(), row.sha256.unwrap());

            targets_map.insert(
                filename,
                TargetFile {
                    length: row.size_bytes.unwrap(),
                    hashes,
                    custom: None,
                },
            );
        }

        let targets = Targets {
            _type: "targets".to_string(),
            spec_version: "1.0".to_string(),
            version: 1,
            // TUF requires strictly increasing versions.
            expires: Utc::now() + chrono::Duration::days(1),
            targets: targets_map,
        };

        Ok(self.sign(targets))
    }

    pub fn generate_snapshot(&self, targets: &SignedMetadata<Targets>) -> SignedMetadata<Snapshot> {
        let targets_bytes = serde_json::to_vec(targets).unwrap();
        // SHA256 of the targets.json (signed wrapper)
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&targets_bytes);
        let hash = format!("{:x}", hasher.finalize());
        let length = targets_bytes.len() as i64;

        let mut meta = HashMap::new();
        let mut hashes = HashMap::new();
        hashes.insert("sha256".to_string(), hash);

        meta.insert(
            "targets.json".to_string(),
            MetaFile {
                version: targets.signed.version,
                length: Some(length),
                hashes: Some(hashes),
            },
        );

        // Add root.json if we had it dynamic, but for now just targets.

        let snapshot = Snapshot {
            _type: "snapshot".to_string(),
            spec_version: "1.0".to_string(),
            version: (Utc::now().timestamp() % 2147483647) as i32,
            expires: Utc::now() + chrono::Duration::days(1),
            meta,
        };
        self.sign(snapshot)
    }

    pub fn generate_timestamp(
        &self,
        snapshot: &SignedMetadata<Snapshot>,
    ) -> SignedMetadata<Timestamp> {
        let snapshot_bytes = serde_json::to_vec(snapshot).unwrap();
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&snapshot_bytes);
        let hash = format!("{:x}", hasher.finalize());
        let length = snapshot_bytes.len() as i64;

        let mut meta = HashMap::new();
        let mut hashes = HashMap::new();
        hashes.insert("sha256".to_string(), hash);

        meta.insert(
            "snapshot.json".to_string(),
            MetaFile {
                version: snapshot.signed.version,
                length: Some(length),
                hashes: Some(hashes),
            },
        );

        let timestamp = Timestamp {
            _type: "timestamp".to_string(),
            spec_version: "1.0".to_string(),
            version: (Utc::now().timestamp() % 2147483647) as i32,
            expires: Utc::now() + chrono::Duration::minutes(15), // Short lived
            meta,
        };
        self.sign(timestamp)
    }

    pub fn sign<T: Serialize>(&self, data: T) -> SignedMetadata<T> {
        let json_bytes = serde_json::to_vec(&data).unwrap();
        let signature_bytes = self.signing_key.sign(&json_bytes);
        let sig_base64 = general_purpose::STANDARD.encode(signature_bytes.to_bytes());

        SignedMetadata {
            signatures: vec![SignatureObj {
                keyid: self.key_id.clone(),
                sig: sig_base64,
            }],
            signed: data,
        }
    }
}

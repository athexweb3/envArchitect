use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub github_id: i64,
    pub username: String,
    pub email: Option<String>,
    pub role: String, // 'user' | 'admin'
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Package {
    pub id: Uuid,
    pub name: String,
    pub owner_id: Uuid,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub is_archived: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PackageVersion {
    pub id: Uuid,
    pub package_id: Uuid,

    // SemVer
    pub version_major: i32,
    pub version_minor: i32,
    pub version_patch: i32,
    pub version_prerelease: Option<String>,
    pub version_raw: String,

    // Artifact
    pub oci_reference: String,
    pub integrity_hash: String,

    // Managed Lifecycle
    pub approval_status: String, // 'PENDING', 'APPROVED', 'REJECTED'
    pub is_yanked: Option<bool>,
    pub yanked_reason: Option<String>,

    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Dependency {
    pub id: Uuid,
    pub dependent_version_id: Uuid,
    pub dependency_package_id: Uuid,
    pub version_req: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Signature {
    pub id: Uuid,
    pub version_id: Uuid,
    pub signer_type: String, // 'DEVELOPER', 'PLATFORM'
    pub signer_id: Option<Uuid>,
    pub signature_content: String,
    pub public_key: Option<String>,
    pub certificate: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub actor_id: Option<Uuid>,
    pub event_type: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

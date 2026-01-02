-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- 1. USERS TABLE
-- Stores developers and admins. Authenticated via GitHub.
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    github_id BIGINT UNIQUE NOT NULL,
    username TEXT NOT NULL,
    email TEXT,
    role TEXT NOT NULL DEFAULT 'user', -- 'user', 'admin'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 2. PACKAGES TABLE
-- The high-level "crate" or "plugin" entity.
CREATE TABLE packages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT UNIQUE NOT NULL,
    owner_id UUID NOT NULL REFERENCES users(id),
    description TEXT,
    repository_url TEXT,
    is_archived BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 3. PACKAGE VERSIONS TABLE
-- Immutable release artifacts. SemVer is split for easy querying.
CREATE TABLE package_versions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    package_id UUID NOT NULL REFERENCES packages(id),
    
    -- SemVer Columns (e.g. 1.2.3-alpha.1)
    version_major INT NOT NULL,
    version_minor INT NOT NULL,
    version_patch INT NOT NULL,
    version_prerelease TEXT, -- 'alpha.1', NULL for stable
    version_raw TEXT NOT NULL, -- Full string "1.2.3-alpha.1" for display
    
    -- OCI Artifact Ref (Where the Wasm blob lives)
    oci_reference TEXT NOT NULL, 
    integrity_hash TEXT NOT NULL, -- SHA256 of the .wasm file
    
    -- Managed Lifecycle
    approval_status TEXT NOT NULL DEFAULT 'PENDING', -- 'PENDING', 'APPROVED', 'REJECTED'
    is_yanked BOOLEAN DEFAULT FALSE, -- Soft delete for breaking bugs
    yanked_reason TEXT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraint: No duplicate versions for a package
    UNIQUE (package_id, version_major, version_minor, version_patch, version_prerelease)
);

-- 4. DEPENDENCIES TABLE
-- The Graph. Version ranges are stored as text (e.g. "^1.0").
CREATE TABLE dependencies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    dependent_version_id UUID NOT NULL REFERENCES package_versions(id), -- The consumer
    dependency_package_id UUID NOT NULL REFERENCES packages(id),        -- The target package
    version_req TEXT NOT NULL, -- "^1.2.0"
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 5. SIGNATURES TABLE (The Double-Lock)
-- Stores both Developer signatures and the Official Platform signature.
CREATE TABLE signatures (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    version_id UUID NOT NULL REFERENCES package_versions(id),
    signer_type TEXT NOT NULL, -- 'DEVELOPER', 'PLATFORM'
    signer_id UUID, -- NULL for PLATFORM (system identity)
    
    -- The Signature Blob (Cosign/Sigstore format)
    signature_content TEXT NOT NULL,
    public_key TEXT, -- If key-based (legacy)
    certificate TEXT, -- If keyless (Sigstore OIDC)
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 6. AUDIT LOGS TABLE
-- Immutable security trail.
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    actor_id UUID REFERENCES users(id), -- NULL for system actions
    event_type TEXT NOT NULL, -- 'PUBLISH', 'APPROVE', 'YANK', 'LOGIN'
    resource_type TEXT NOT NULL, -- 'PACKAGE', 'VERSION', 'USER'
    resource_id UUID,
    
    ip_address TEXT,
    user_agent TEXT,
    payload JSONB, -- Context (e.g. { "old_status": "PENDING", "new_status": "APPROVED" })
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for Speed
CREATE INDEX idx_packages_name ON packages(name);
CREATE INDEX idx_versions_package ON package_versions(package_id);
CREATE INDEX idx_versions_semver ON package_versions(version_major, version_minor, version_patch);
CREATE INDEX idx_audit_created_at ON audit_logs(created_at);

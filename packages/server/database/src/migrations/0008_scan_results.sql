-- 0008_scan_results
-- Description: Stores results of static analysis (malware scanning) for package versions.

CREATE TYPE scan_status AS ENUM ('pending', 'safe', 'suspicious', 'malicious');

CREATE TABLE IF NOT EXISTS scan_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Link to the package version being scanned
    version_id UUID NOT NULL REFERENCES package_versions(id) ON DELETE CASCADE,
    
    status scan_status NOT NULL DEFAULT 'pending',
    
    -- JSON report containing details (e.g. "detected_imports": ["env.exec"], "score": 85)
    report JSONB NOT NULL DEFAULT '{}'::jsonb,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Ensure one scan result per version
    CONSTRAINT uq_scan_results_version_id UNIQUE (version_id)
);

CREATE INDEX idx_scan_results_status ON scan_results(status);

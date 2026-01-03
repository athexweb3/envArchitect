-- 0003_registry_foundation
-- Description: Sets up the PURL Component Graph and Search Foundation.

-- 1. CLEANUP (Drop legacy dependencies table)
DROP TABLE IF EXISTS dependencies;

-- 2. EXTENSIONS
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- 3. COMPONENTS TABLE (The "Nodes" of the Graph)
-- Every node in the dependency graph (Internal Plugin or External Lib) is a component.
CREATE TABLE components (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    purl TEXT UNIQUE NOT NULL, -- e.g. pkg:cargo/tokio@1.0.0
    ecosystem TEXT NOT NULL,   -- 'env', 'npm', 'cargo'
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 4. DEPENDENCIES TABLE (The "Edges" of the Graph)
-- Replaces the old table. Uses Component IDs.
CREATE TABLE dependencies (
    source_id UUID NOT NULL REFERENCES components(id) ON DELETE CASCADE,
    target_id UUID NOT NULL REFERENCES components(id) ON DELETE CASCADE,
    version_req TEXT NOT NULL, -- e.g. "^1.0.0"
    kind TEXT NOT NULL,        -- 'runtime', 'dev', 'build'
    
    PRIMARY KEY (source_id, target_id)
);

-- 5. INDEXES
-- Fast Reverse Lookup ("Used By")
CREATE INDEX idx_deps_target ON dependencies(target_id);
-- Fast PURL Lookup
CREATE INDEX idx_components_purl ON components(purl);
-- Fast Search Helper (e.g. Find all versions of 'react')
CREATE INDEX idx_components_ecosystem_name ON components(ecosystem, name);

-- 6. SEARCH UPGRADE (Phase 1)
-- Add a Search Vector to the main Packages table for Weighted FTS.
ALTER TABLE packages ADD COLUMN IF NOT EXISTS search_vector tsvector;
CREATE INDEX IF NOT EXISTS idx_packages_search ON packages USING GIN(search_vector);

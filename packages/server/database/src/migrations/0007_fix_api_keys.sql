-- 0007_fix_api_keys
-- Description: Re-creates api_keys table to ensure schema consistency (fixing 'prefix column missing' error).

DROP TABLE IF EXISTS api_keys CASCADE;

CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    prefix VARCHAR(32) NOT NULL, 
    hash VARCHAR(255) NOT NULL, 
    scopes TEXT[] NOT NULL DEFAULT '{}', 
    lookup_hash VARCHAR(64) NOT NULL,
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_api_keys_user_id ON api_keys(user_id);
CREATE UNIQUE INDEX idx_api_keys_lookup_hash ON api_keys(lookup_hash);

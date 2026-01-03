-- 0006_api_keys
-- Description: Stores hashed API keys for CI/CD automation.

CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    
    -- "env_live_" or "env_test_"
    prefix VARCHAR(32) NOT NULL, 
    
    -- Argon2 hash of the full key (env_live_entropy_checksum)
    -- We never store the raw key.
    hash VARCHAR(255) NOT NULL, 
    
    -- Scopes: "package:publish", "package:yank", etc.
    scopes TEXT[] NOT NULL DEFAULT '{}', 
    
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Fast Index for looking up keys by user (UI list)
CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys(user_id);

-- Fast Index for looking up keys by prefix (Auth check optimization?)
-- Actually, strict lookup happens by finding the key via ID or just iterating user keys? 
-- No, usually we look up by HASH? No, you can't look up by hash with Argon2 (salted).
-- Strategy:
-- 1. Client sends `env_live_abc123...checksum`
-- 2. We CANNOT look up by hash directly efficiently if unique salt per row.
-- 3. We typically need an ID... OR we accept that we must scan?
--
-- FIX: Stripe approach: The Access Token often HAS an ID embedded or we store a "lookup hash" (SHA256) AND a "verification hash" (Argon2).
-- OR: We just trust the key is correct, but how do we find WHICH row matches?
--
-- REVISION:
-- To allow fast lookup, we should store a `lookup_hash` (Fast, e.g. SHA256 of the key) 
-- AND a `secure_hash` (Slow, e.g. Argon2). 
-- OR, we store the `prefix + first 8 chars` as an indexable column?
-- 
-- Let's stick to the simplest "V1" that works:
-- The `api_keys` table is usually small per user.
-- BUT wait, the Request only has the Key. We don't know the User.
-- We cannot iterate ALL keys in DB.
--
-- SOLUTION: 
-- We will embed the `kid` (Key ID) into the key itself? 
-- `env_live_[base62_id]_[entropy][checksum]`?
-- Stripe keys are opaque.
--
-- Alternative: Store `token_hash` (SHA256) as the primary lookup. 
-- SHA256 is fast and unique. If we treat the Key itself as high entropy, SHA256 is safe enough for LOOKUP, 
-- but for "Password" equivalent, Argon2 is better.
--
-- Let's use `lookup_hash` (SHA256) for the index.

ALTER TABLE api_keys ADD COLUMN lookup_hash VARCHAR(64) NOT NULL;
CREATE UNIQUE INDEX idx_api_keys_lookup_hash ON api_keys(lookup_hash);

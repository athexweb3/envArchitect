-- 0002: Auth Extensions for Device Flow and Secure Tokens

-- 1. DEVICE CODES TABLE (OAuth 2.0 Device Authorization Grant)
-- Tracks the handshake between CLI and Web during `login`.
CREATE TABLE device_codes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    device_code TEXT UNIQUE NOT NULL,
    user_code TEXT UNIQUE NOT NULL, -- The 8-character human-readable code
    user_id UUID REFERENCES users(id), -- Populated after user authorizes in browser
    scopes TEXT[] DEFAULT '{}',
    expires_at TIMESTAMPTZ NOT NULL,
    last_polled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 2. REFRESH TOKENS TABLE
-- Used to issue new access tokens without re-authenticating.
-- These are rotated on every use for maximum security.
CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id),
    token_hash TEXT NOT NULL, -- SHA256 of the token
    expires_at TIMESTAMPTZ NOT NULL,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 3. API KEYS TABLE
-- For CI/CD and programmatic access.
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id),
    name TEXT NOT NULL, -- Friendly name e.g. "GitHub Actions"
    key_prefix TEXT NOT NULL, -- To help user identify the key
    key_hash TEXT NOT NULL, -- SHA256 of the key
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_device_codes_device_code ON device_codes(device_code);
CREATE INDEX idx_device_codes_user_code ON device_codes(user_code);
CREATE INDEX idx_refresh_tokens_hash ON refresh_tokens(token_hash);
CREATE INDEX idx_api_keys_hash ON api_keys(key_hash);

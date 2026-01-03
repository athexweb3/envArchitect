-- Add metadata for TUF / Integrity checks
ALTER TABLE components ADD COLUMN sha256 TEXT;
ALTER TABLE components ADD COLUMN size_bytes BIGINT;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='users' AND column_name='ghcr_pat') THEN
        ALTER TABLE users ADD COLUMN ghcr_pat TEXT;
    END IF;
END
$$;

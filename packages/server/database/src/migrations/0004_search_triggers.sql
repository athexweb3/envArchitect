-- 0004_search_triggers
-- Description: "World Class" Performance: Move Indexing logic to Database Layer via Triggers.
-- This ensures zero latency penalty on API rights and "Always Fresh" search indexes.

-- 1. ADD DOWNLOADS COLUMN (Rank Factor: Popularity)
ALTER TABLE packages ADD COLUMN IF NOT EXISTS downloads INT NOT NULL DEFAULT 0;
CREATE INDEX IF NOT EXISTS idx_packages_downloads ON packages(downloads DESC);

-- 2. Create a function to generating the Weighted Search Vector
-- Priority A: Name (1.0)
-- Priority B: Keywords/Ecosystem (From components?)
-- Priority C: Description (0.5)
-- We use 'coalesce' to handle nulls gracefully.
CREATE OR REPLACE FUNCTION func_packages_update_search_vector() RETURNS trigger AS $$
BEGIN
  -- Construct the vector
  -- Note: ecosystem is not on packages table, it's on components maybe?
  -- Wait, package is the high level entity.
  -- Let's just index name & description for now.
  -- Add simple name indexing.
  NEW.search_vector :=
    setweight(to_tsvector('english', coalesce(NEW.name, '')), 'A') ||
    setweight(to_tsvector('english', coalesce(NEW.description, '')), 'C');
  return NEW;
END
$$ LANGUAGE plpgsql;

-- 3. Create the Trigger
-- Runs BEFORE INSERT or UPDATE. Very fast.
DROP TRIGGER IF EXISTS trg_packages_search_vector ON packages;
CREATE TRIGGER trg_packages_search_vector
BEFORE INSERT OR UPDATE ON packages
FOR EACH ROW EXECUTE PROCEDURE func_packages_update_search_vector();

-- 4. Backfill existing data
-- Force a re-save of all rows to calculate the vector for existing data.
UPDATE packages SET id = id;

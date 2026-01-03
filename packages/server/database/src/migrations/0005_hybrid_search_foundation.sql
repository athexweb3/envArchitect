-- 0005_hybrid_search_foundation
-- Description: Enables Vector Search (Semantic) and Q-P-M Scoring Columns.

-- 1. Enable pgvector extension (if available)
-- Note: User needs to have pgvector installed on their Postgres server.
CREATE EXTENSION IF NOT EXISTS vector;

-- 2. Add Embeddings to Packages (The "Semantic" Signal)
-- We use 384 dimensions (standard for all-MiniLM-L6-v2) for local efficiency.
-- If switching to OpenAI (1536), we will need an ALTER column later.
ALTER TABLE packages 
ADD COLUMN IF NOT EXISTS embedding vector(384);

-- Index for fast cosine similarity search
-- ivfflat is good for speed, hnsw is better for recall. HNSW is "World Class".
CREATE INDEX IF NOT EXISTS idx_packages_embedding 
ON packages 
USING hnsw (embedding vector_cosine_ops);

-- 3. Add Ranking Signals (The "Q-P-M" Scorecard)
-- All scores are normalized 0.0 to 1.0
ALTER TABLE packages 
ADD COLUMN IF NOT EXISTS score_quality real DEFAULT 0.0,
ADD COLUMN IF NOT EXISTS score_popularity real DEFAULT 0.0,
ADD COLUMN IF NOT EXISTS score_maintenance real DEFAULT 0.0,
ADD COLUMN IF NOT EXISTS score_authority real DEFAULT 0.0, -- DependencyRank
ADD COLUMN IF NOT EXISTS score_trending real DEFAULT 0.0;    -- Install Velocity

-- 4. Create "Hybrid Score" generated column or view?
-- A generated column is faster for sorting than computing on fly.
-- But weights might change. Let's keep it compute-on-fly for now in the SQL query, 
-- or use a pre-calculated column updated by a background worker.
-- Let's add a `final_score` column for ultra-fast "Order By".
ALTER TABLE packages
ADD COLUMN IF NOT EXISTS final_score real DEFAULT 0.0;
CREATE INDEX IF NOT EXISTS idx_packages_final_score ON packages(final_score DESC);

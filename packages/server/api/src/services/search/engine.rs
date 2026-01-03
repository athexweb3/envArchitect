use super::embedding::LocalEmbedder;
use anyhow::Result;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
// use super::signals::SearchSignals; // Used implicitly in query

pub struct SearchEngine {
    embedder: LocalEmbedder,
}

#[derive(FromRow, Debug, serde::Serialize)]
pub struct SearchResult {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub score: f32, // Changed to match "real" return
    pub downloads: i32,
}

impl SearchEngine {
    pub fn new() -> Result<Self> {
        let embedder = LocalEmbedder::new()?;
        Ok(Self { embedder })
    }

    pub fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        self.embedder.generate(text)
    }

    pub async fn search(&self, pool: &PgPool, q: &str, limit: i64) -> Result<Vec<SearchResult>> {
        if q.is_empty() {
            return Ok(vec![]);
        }

        // 1. Generate Embedding (CPU intensive, spawn blocking?)
        // For now, simple call. In prod, move to task::spawn_blocking.
        let embedding = self.embedder.generate(q)?;
        // pgvector requires Vector type or simply &[f32] for bind?
        // sqlx-pgvector maps Vec<f32> automatically if feature enabled.

        let limit = limit.min(50);

        // 2. Hybrid Query
        // We use a CTE to combine:
        // A. Semantic Rank (1 - cosine_distance)
        // B. Keyword Rank (ts_rank)
        // C. Signals (Popularity, Quality, etc)
        //
        // Final Score = (0.3 * Keyword) + (0.3 * Vector) + (0.4 * Signals)

        // Using runtime query_as instead of macro to bypass type inference issues with vectors
        let results = sqlx::query_as::<_, SearchResult>(
            r#"
            WITH vector_scores AS (
                SELECT id, 1 - (embedding <=> $1::real[]::vector) as v_score
                FROM packages
                ORDER BY embedding <=> $1::real[]::vector
                LIMIT 100
            ),
            keyword_scores AS (
                SELECT id, ts_rank(search_vector, websearch_to_tsquery('english', $2)) as k_score
                FROM packages
                WHERE search_vector @@ websearch_to_tsquery('english', $2)
                LIMIT 100
            )
            SELECT 
                p.id,
                p.name,
                p.description,
                v.version_raw as version,
                p.downloads,
                (
                    (COALESCE(k.k_score, 0) * 0.30) + 
                    (COALESCE(vec.v_score, 0) * 0.25) +
                    (COALESCE(p.score_authority, 0) * 0.20) +
                    (COALESCE(p.score_trending, 0) * 0.15) +
                    (COALESCE(p.score_quality, 0) * 0.10)
                )::real as score
            FROM packages p
            LEFT JOIN (SELECT package_id, version_raw FROM package_versions ORDER BY created_at DESC) v ON v.package_id = p.id
            LEFT JOIN vector_scores vec ON vec.id = p.id
            LEFT JOIN keyword_scores k ON k.id = p.id
            WHERE k.id IS NOT NULL OR vec.id IS NOT NULL
            ORDER BY score DESC
            LIMIT $3
            "#)
            .bind(&embedding)
            .bind(q)
            .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e: sqlx::Error| anyhow::anyhow!("Hybrid search failed: {}", e))?;

        Ok(results)
    }
}

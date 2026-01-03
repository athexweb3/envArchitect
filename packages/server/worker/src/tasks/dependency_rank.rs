use anyhow::Result;
use sqlx::PgPool;
use tracing::info;

/// Calculates the "Authority Score" (PageRank) for all packages.
///
/// Algorithm: Standard Iterative PageRank using Recursive CTE.
/// Damping Factor: 0.85
/// Iterations: 20 (Hard coded limit for performance safety)
///
/// This runs entirely in the database to avoid fetching the large graph into memory.
pub async fn calculate_authority(pool: &PgPool) -> Result<()> {
    info!("Starting DependencyRank (PageRank) calculation...");

    // We use a CTE to perform the iteration.
    // Note: writing a true recursive loop in SQL for PageRank is complex because
    // strictly speaking, standard Recursive CTEs traverse graphs, they don't easily allow
    // "Update all nodes based on previous iteration of all nodes" in a loop syntax.
    //
    // However, for PageRank, we can approximate or use a fixed iterative approach if supported,
    // OR we can use a PL/pgSQL function.
    //
    // For raw SQL compatibility without stored procs, we can simulate N iterations
    // by simply running the update N times in a loop here in Rust.
    // This is actually safer and easier to debug than a massive recursive SQL query.

    let damping_factor: f32 = 0.85;
    let iterations = 20;

    // 1. Initialize all scores to 1.0 (or 1/N, but 1.0 is easier to read, relative rank matters)
    sqlx::query!(
        r#"
        UPDATE packages 
        SET score_authority = 1.0
        "#
    )
    .execute(pool)
    .await?;

    // 2. Run Iterations
    for i in 0..iterations {
        // formula: PR(A) = (1-d) + d * Sum(PR(T)/C(T))
        // where T are pages pointing to A, and C(T) is out-degree of T.

        // We do this in a single massive UPDATE using a calculated table.
        // To avoid "updating while reading", we might need a temp table or just let Postgres MVCC handle it?
        // Actually, updating in place implies we use the "latest" values as we go (Gauss-Seidel) or "old" values (Jacobi).
        // Standard SQL UPDATE uses the snapshot at start of query (Jacobi). This is fine for PageRank.

        let affected = sqlx::query!(
            r#"
            WITH outbound_counts AS (
                -- Calculate C(T): number of dependencies OF standard packages
                -- We join components-dependencies-components
                -- but wait, our graph is packages.
                -- dependencies link `components`.
                -- We need to map component->package.
                -- For simplicity V1, let's assume 1 package = 1 component (latest version).
                -- Actually, dependencies link specific versions, but Authority is a Package-level metric.
                -- STRATEGY: Collapse the graph. Package A depends on Package B if ANY version of A depends on B.
                
                SELECT source_c.name as source_pkg, COUNT(DISTINCT target_c.name) as out_degree
                FROM dependencies d
                JOIN components source_c ON d.source_id = source_c.id
                JOIN components target_c ON d.target_id = target_c.id
                GROUP BY source_c.name
            ),
            incoming_scores AS (
                -- Calculate Sum(PR(T)/C(T)) for each target
                SELECT 
                    target_c.name as target_pkg, 
                    SUM(p_source.score_authority / GREATEST(oc.out_degree, 1)) as inbound_sum
                FROM dependencies d
                JOIN components source_c ON d.source_id = source_c.id
                JOIN components target_c ON d.target_id = target_c.id
                JOIN packages p_source ON p_source.name = source_c.name
                JOIN outbound_counts oc ON oc.source_pkg = source_c.name
                GROUP BY target_c.name
            )
            UPDATE packages p
            SET score_authority = (1.0 - $1::real) + ($1::real * COALESCE(inc.inbound_sum, 0))
            FROM incoming_scores inc
            WHERE p.name = inc.target_pkg
            "#,
            damping_factor
        )
        .execute(pool)
        .await?
        .rows_affected();

        if i % 5 == 0 {
            info!("PageRank Iteration {}: updated {} packages", i, affected);
        }
    }

    // 3. Normalize to 0.0 - 1.0 range for the final search formula
    // Find Max, then Divide.
    sqlx::query!(
        r#"
        WITH stats AS (SELECT MAX(score_authority) as max_score FROM packages)
        UPDATE packages 
        SET score_authority = score_authority / GREATEST(stats.max_score, 1.0)
        FROM stats
        "#
    )
    .execute(pool)
    .await?;

    info!("DependencyRank calculation complete.");
    Ok(())
}

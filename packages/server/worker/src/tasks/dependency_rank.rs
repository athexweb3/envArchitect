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

    let damping_factor: f32 = 0.85;
    let iterations = 20;

    sqlx::query!(
        r#"
        UPDATE packages 
        SET score_authority = 1.0
        "#
    )
    .execute(pool)
    .await?;

    for i in 0..iterations {
        let affected = sqlx::query!(
            r#"
            WITH outbound_counts AS (

                
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

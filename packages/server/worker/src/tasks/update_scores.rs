use anyhow::{Context, Result};
use sqlx::PgPool;
use tracing::info;

/// Updates "Trending" and "Maintenance" scores for all packages.
///
/// Signals:
/// 1. Trending (Velocity): (Recent Downloads / Total Downloads) * Age Penalty
///    - Identifies "Rising Stars".
/// 2. Maintenance (Health): Time Decay since last update.
///    - Penalizes abandonware.
pub async fn update_scores(pool: &PgPool) -> Result<()> {
    info!("Starting ScoreUpdater (Trending + Maintenance)...");

    sqlx::query!(
        r#"
        UPDATE packages
        SET score_trending = (
            -- Log10(downloads + 1) to damp massive numbers
            LOG(GREATEST(downloads, 0) + 1.0) 
            / 
            -- Divide by Age (Days + 1). Younger = Higher Score.
            GREATEST(EXTRACT(EPOCH FROM (NOW() - created_at)) / 86400.0, 1.0)
        )

        "#
    )
    .execute(pool)
    .await
    .context("Failed to update trending scores")?;

    sqlx::query!(
        r#"
        UPDATE packages
        SET score_maintenance = EXP(
            -0.0019 * (EXTRACT(EPOCH FROM (NOW() - updated_at)) / 86400.0)
        )
        "#
    )
    .execute(pool)
    .await
    .context("Failed to update maintenance scores")?;

    info!("ScoreUpdater complete.");
    Ok(())
}

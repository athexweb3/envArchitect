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

    // 1. Calculate Trending Score (Velocity)
    // Formula: (downloads_last_7d) / (total_downloads + 1)
    // Note: We don't track 7d downloads explicitly yet in this schema,
    // so we'll approximate using `updated_at` Recency as a proxy for "Freshness" for now,
    // OR we assume `downloads` is "all time".
    //
    // REALITY CHECK: We don't have a `downloads_history` table yet.
    // So "Trending" in V1 will be synonymous with "Recently Created/Updated with High Downloads".
    //
    // Let's implement the "Rising Star" proxy:
    // Score = Log(Downloads + 1) / (Age_In_Days + 1)
    // This boosts new packages with downloads over old packages with same downloads.

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
        -- Normalization happen later or we assume this raw score is useful enough for sorting relative.
        -- Let's normalize to 0.0-1.0 roughly by clamping? 
        -- Or just update and let the ranker normalize.
        "#
    )
    .execute(pool)
    .await
    .context("Failed to update trending scores")?;

    // 2. Calculate Maintenance Score (Health)
    // Formula: Exponential Decay based on `updated_at`.
    // 1.0 = Just updated.
    // 0.5 = 1 year old.
    // 0.1 = 3 years old.
    // Decay Constant (lambda) for Half-Life of 365 days:
    // N(t) = N0 * e^(-lambda * t)
    // 0.5 = e^(-lambda * 365) => ln(0.5) = -lambda * 365 => lambda = -ln(0.5)/365 ~= 0.0019

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

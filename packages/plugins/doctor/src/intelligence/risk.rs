use std::collections::HashMap;

/// Calculate Risk Score based on Centrality, Age, and Vulnerabilities
pub struct RiskCalculator;

impl RiskCalculator {
    pub fn calculate(centrality: u32, days_since_release: u32, vuln_count: u32) -> f64 {
        let w_centrality = 1.5;
        let w_age = 0.5;
        let w_vuln = 10.0;

        let age_factor = if days_since_release > 730 {
            (days_since_release as f64 / 365.0).log10().max(0.0)
        } else {
            0.0
        };

        (centrality as f64 * w_centrality) + (age_factor * w_age) + (vuln_count as f64 * w_vuln)
    }
}

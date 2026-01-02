use std::collections::HashMap;

/// Simplified Bayesian Inference for Root Cause Analysis
pub struct CausalEngine {
    // Map of RootCause -> Prior Probability
    priors: HashMap<String, f64>,
    // Map of (Symptom, RootCause) -> Conditional Probability P(E|H)
    conditionals: HashMap<(String, String), f64>,
}

impl CausalEngine {
    pub fn new() -> Self {
        Self {
            priors: HashMap::new(),
            conditionals: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, cause: &str, symptom: &str, probability: f64) {
        self.conditionals
            .insert((symptom.to_string(), cause.to_string()), probability);
        // Ensure prior exists, default to uniform if unknown (simplified)
        self.priors.entry(cause.to_string()).or_insert(0.1);
    }

    pub fn infer(&self, observed_symptoms: &[&str]) -> Vec<(String, f64)> {
        let mut posterior_probs: Vec<(String, f64)> = Vec::new();

        for (cause, p_cause) in &self.priors {
            let mut likelihood = 1.0;
            for symptom in observed_symptoms {
                if let Some(p_symptom_given_cause) =
                    self.conditionals.get(&(symptom.to_string(), cause.clone()))
                {
                    likelihood *= p_symptom_given_cause;
                } else {
                    // If symptom not associated with cause, likelihood drops
                    likelihood *= 0.01;
                }
            }

            // P(H|E) ~ P(E|H) * P(H)  (Ignoring P(E) normalization for ranking)
            let raw_posterior = likelihood * p_cause;
            posterior_probs.push((cause.clone(), raw_posterior));
        }

        posterior_probs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        posterior_probs
    }
}

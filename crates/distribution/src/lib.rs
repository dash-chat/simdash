use rand::distr::Uniform;
use rand::prelude::*;
use rand_distr::Normal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Distribution {
    Fixed { value: f64 },
    Uniform { min: f64, max: f64 },
    Normal { mean: f64, std_dev: f64 },
}

impl Distribution {
    pub fn fixed(value: f64) -> Self {
        Distribution::Fixed { value }
    }

    pub fn uniform(min: f64, max: f64) -> Self {
        Distribution::Uniform { min, max }
    }

    pub fn normal(mean: f64, std_dev: f64) -> Self {
        Distribution::Normal { mean, std_dev }
    }

    /// Scale the distribution parameters by a factor.
    pub fn scale(&self, factor: f64) -> Self {
        match self {
            Distribution::Fixed { value } => Distribution::fixed(value * factor),
            Distribution::Uniform { min, max } => {
                Distribution::uniform(min * factor, max * factor)
            }
            Distribution::Normal { mean, std_dev } => {
                Distribution::normal(mean * factor, std_dev * factor)
            }
        }
    }

    pub fn sample(&self, rng: &mut impl Rng) -> u64 {
        let value = match self {
            Distribution::Fixed { value } => *value,
            Distribution::Uniform { min, max } => {
                rng.sample(Uniform::new(*min, *max).expect("invalid uniform range"))
            }
            Distribution::Normal { mean, std_dev } => {
                rng.sample(Normal::new(*mean, *std_dev).expect("invalid normal parameters"))
            }
        };
        value.round().max(0.0) as u64
    }
}

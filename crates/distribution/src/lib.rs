use rand::distr::Uniform;
use rand::prelude::*;
use rand_distr::Normal;

#[derive(Debug, Clone)]
pub enum Distribution {
    Fixed(f64),
    Uniform { min: f64, max: f64, distr: Uniform<f64> },
    Normal { mean: f64, std_dev: f64, distr: Normal<f64> },
}

impl Distribution {
    pub fn fixed(value: f64) -> Self {
        Distribution::Fixed(value)
    }

    pub fn uniform(min: f64, max: f64) -> Self {
        Distribution::Uniform {
            min,
            max,
            distr: Uniform::new(min, max).expect("invalid uniform range"),
        }
    }

    pub fn normal(mean: f64, std_dev: f64) -> Self {
        Distribution::Normal {
            mean,
            std_dev,
            distr: Normal::new(mean, std_dev).expect("invalid normal parameters"),
        }
    }

    /// Scale the distribution parameters by a factor.
    pub fn scale(&self, factor: f64) -> Self {
        match self {
            Distribution::Fixed(v) => Distribution::fixed(v * factor),
            Distribution::Uniform { min, max, .. } => {
                Distribution::uniform(min * factor, max * factor)
            }
            Distribution::Normal { mean, std_dev, .. } => {
                Distribution::normal(mean * factor, std_dev * factor)
            }
        }
    }

    pub fn sample(&self, rng: &mut impl Rng) -> u64 {
        let value = match self {
            Distribution::Fixed(v) => *v,
            Distribution::Uniform { distr, .. } => rng.sample(distr),
            Distribution::Normal { distr, .. } => rng.sample(distr),
        };
        value.round().max(0.0) as u64
    }
}

use rand::distr::Uniform;
use rand::prelude::*;
use rand_distr::Normal;

#[derive(Debug, Clone)]
pub enum Distribution {
    Fixed(f64),
    Uniform(Uniform<f64>),
    Normal(Normal<f64>),
}

impl Distribution {
    pub fn fixed(value: f64) -> Self {
        Distribution::Fixed(value)
    }

    pub fn uniform(min: f64, max: f64) -> Self {
        Distribution::Uniform(Uniform::new(min, max).expect("invalid uniform range"))
    }

    pub fn normal(mean: f64, std_dev: f64) -> Self {
        Distribution::Normal(Normal::new(mean, std_dev).expect("invalid normal parameters"))
    }

    pub fn sample(&self, rng: &mut impl Rng) -> u64 {
        let value = match self {
            Distribution::Fixed(v) => *v,
            Distribution::Uniform(d) => rng.sample(d),
            Distribution::Normal(d) => rng.sample(d),
        };
        value.round().max(0.0) as u64
    }
}

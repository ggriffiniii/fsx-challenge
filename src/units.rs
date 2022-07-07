use serde::{Deserialize, Serialize};

const YARDS_PER_METER: f64 = 1.09361;

#[derive(Clone, Copy, Debug)]
pub struct Meters(usize);
impl Meters {
    pub fn from_float(x: f64) -> Self {
        Meters((x * 1000.0) as usize)
    }
    pub fn to_yards(self) -> Yards {
        Yards((self.0 as f64 * YARDS_PER_METER) as usize)
    }
    pub fn as_float(self) -> f64 {
        self.0 as f64 / 1000.0
    }
}
impl<'de> Deserialize<'de> for Meters {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Meters::from_float(Deserialize::deserialize(deserializer)?))
    }
}

impl Serialize for Meters {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Serialize::serialize(&self.as_float(), serializer)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Yards(usize);
impl Yards {
    pub fn new(val: usize) -> Self {
        Yards(val * 1000)
    }
    pub fn from_float(x: f64) -> Self {
        Yards((x * 1000.0) as usize)
    }
    pub fn as_float(self) -> f64 {
        self.0 as f64 / 1000.0
    }
    pub fn to_meters(self) -> Meters {
        Meters((self.0 as f64 / YARDS_PER_METER) as usize)
    }
    pub fn abs_diff(self, other: Self) -> Self {
        Yards(self.0.abs_diff(other.0))
    }
}
impl<'de> Deserialize<'de> for Yards {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Yards::from_float(Deserialize::deserialize(deserializer)?))
    }
}

impl Serialize for Yards {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Serialize::serialize(&self.as_float(), serializer)
    }
}
use rand::distributions::uniform::{SampleBorrow, SampleUniform, UniformInt, UniformSampler};

pub struct UniformYards(UniformInt<usize>);
impl UniformSampler for UniformYards {
    type X = Yards;
    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        UniformYards(UniformInt::new(low.borrow().0, high.borrow().0))
    }
    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        UniformYards(UniformInt::new_inclusive(low.borrow().0, high.borrow().0))
    }
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        Yards(self.0.sample(rng))
    }
}

impl SampleUniform for Yards {
    type Sampler = UniformYards;
}

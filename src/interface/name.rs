use super::sample::Sample;

pub trait SampleNamerTrait: Fn(&Sample, usize, &str) -> String + Send + Sync {}

impl<T: Fn(&Sample, usize, &str) -> String + Send + Sync> SampleNamerTrait for T {}

// impl SampleNamerTrait for &dyn SampleNamerTrait {}
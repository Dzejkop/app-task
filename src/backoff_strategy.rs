use std::marker::PhantomData;
use std::time::Duration;

pub mod constant_time;
pub mod threshold_buckets;

pub trait StrategyFactory: Send + Sync {
    type Strategy: BackoffStrategy;

    fn create_strategy(&self) -> Self::Strategy;
}

pub trait BackoffStrategy: Send + Sync + 'static {
    fn add_failure(&mut self);

    fn next_backoff(&self) -> Duration;
}

pub struct DefaultStrategyFactory<S> {
    _strategy: PhantomData<S>,
}

impl<S> Default for DefaultStrategyFactory<S> {
    fn default() -> Self {
        Self {
            _strategy: PhantomData,
        }
    }
}

impl<S> DefaultStrategyFactory<S> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<S> StrategyFactory for DefaultStrategyFactory<S>
where
    S: Default + BackoffStrategy,
{
    type Strategy = S;

    fn create_strategy(&self) -> Self::Strategy {
        Self::Strategy::default()
    }
}

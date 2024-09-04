use std::time::Duration;

use super::{BackoffStrategy, StrategyFactory};

pub struct ContantTimeFactory {
    pub backoff: Duration,
}

impl StrategyFactory for ContantTimeFactory {
    type Strategy = ConstantTimeBackoff;

    fn create_strategy(&self) -> Self::Strategy {
        ConstantTimeBackoff {
            backoff: self.backoff,
        }
    }
}

pub struct ConstantTimeBackoff {
    backoff: Duration,
}

impl Default for ConstantTimeBackoff {
    fn default() -> Self {
        Self {
            backoff: Duration::from_secs(5),
        }
    }
}

impl BackoffStrategy for ConstantTimeBackoff {
    fn add_failure(&mut self) {}

    fn next_backoff(&self) -> Duration {
        self.backoff
    }
}

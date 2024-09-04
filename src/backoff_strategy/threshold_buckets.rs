use std::time::{Duration, Instant};

use super::{BackoffStrategy, StrategyFactory};

pub struct ThresholdBucketsFactory {
    pub buckets: Vec<(usize, Duration)>,
    pub monitoring_period: Duration,
}

impl Default for ThresholdBucketsFactory {
    fn default() -> Self {
        Self {
            buckets: vec![
                (1, Duration::from_secs(5)),
                (5, Duration::from_secs(10)),
                (10, Duration::from_secs(30)),
            ],
            monitoring_period: Duration::from_secs(60),
        }
    }
}

impl StrategyFactory for ThresholdBucketsFactory {
    type Strategy = ThresholdBucketsBackoff;

    fn create_strategy(&self) -> Self::Strategy {
        ThresholdBucketsBackoff {
            buckets: self.buckets.clone(),
            monitoring_period: self.monitoring_period,
            failures: Vec::new(),
        }
    }
}

pub struct ThresholdBucketsBackoff {
    buckets: Vec<(usize, Duration)>,
    monitoring_period: Duration,
    failures: Vec<Instant>,
}

impl BackoffStrategy for ThresholdBucketsBackoff {
    fn add_failure(&mut self) {
        self.failures.push(Instant::now());
        self.prune_failures();
    }

    fn next_backoff(&self) -> Duration {
        let mut backoff = Duration::from_secs(0);

        for (threshold, duration) in &self.buckets {
            if self.failures.len() >= *threshold {
                backoff = *duration;
            }
        }

        backoff
    }
}

impl ThresholdBucketsBackoff {
    fn prune_failures(&mut self) {
        self.failures
            .retain(|instant| instant.elapsed() < self.monitoring_period);
    }
}

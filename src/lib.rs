use std::any::Any;
use std::borrow::Cow;
use std::fmt::Display;
use std::panic::UnwindSafe;
use std::sync::Arc;

use backoff_strategy::constant_time::ConstantTimeBackoff;
use backoff_strategy::{BackoffStrategy, DefaultStrategyFactory, StrategyFactory};
use futures::{Future, FutureExt};
use tokio::task::JoinHandle;

pub mod backoff_strategy;

pub struct TaskRunner<T, SF = DefaultStrategyFactory<ConstantTimeBackoff>> {
    app: Arc<T>,
    backoff_strategy: SF,
}

impl<T> TaskRunner<T> {
    pub fn new(app: Arc<T>) -> Self {
        Self {
            app,
            backoff_strategy: DefaultStrategyFactory::new(),
        }
    }
}

impl<T, SF> TaskRunner<T, SF>
where
    T: Send + Sync + 'static,
    SF: StrategyFactory,
{
    pub fn with_default_strategy<NS>(self) -> TaskRunner<T, DefaultStrategyFactory<NS>>
    where
        NS: StrategyFactory,
    {
        TaskRunner {
            app: self.app,
            backoff_strategy: DefaultStrategyFactory::new(),
        }
    }

    pub fn with_strategy<NSF>(self, backoff_strategy: NSF) -> TaskRunner<T, NSF> {
        TaskRunner {
            app: self.app,
            backoff_strategy,
        }
    }
}

impl<T, SF> TaskRunner<T, SF>
where
    T: Send + Sync + 'static,
    SF: StrategyFactory,
{
    /// Spawns a task that will run until it returns Ok(()) or panics.
    /// If the task returns an error, it will be logged and the task will be retried with a backoff.
    ///
    /// If the task panics, the panic output will be returned as an error.
    pub fn spawn_task<S, C, F, E>(
        &self,
        label: S,
        task: C,
    ) -> JoinHandle<Result<(), Box<dyn Any + Send>>>
    where
        S: ToString,
        C: Fn(Arc<T>) -> F + Send + Sync + 'static,
        F: Future<Output = Result<(), E>> + Send + 'static + UnwindSafe,
        E: Display + Send + Sync,
    {
        let app = self.app.clone();
        let label = label.to_string();

        let mut backoff_strategy = self.backoff_strategy.create_strategy();

        tokio::spawn(async move {
            loop {
                tracing::info!(task_label = label, "Running task");

                let result = task(app.clone()).catch_unwind().await;

                match result {
                    Ok(Ok(())) => {
                        tracing::info!(task_label = label, "Task finished");
                        break;
                    }
                    Ok(Err(err)) => {
                        tracing::error!(task_label = label, error = %err, "Task failed");
                        backoff_strategy.add_failure();
                        tokio::time::sleep(backoff_strategy.next_backoff()).await;
                    }
                    Err(err) => {
                        let reason = panic_helper(&err);
                        tracing::error!(task_label = label, error = %reason, "Task panicked");
                        return Err(err);
                    }
                }
            }

            Ok(())
        })
    }
}

pub fn panic_helper(err: &Box<dyn Any + Send>) -> Cow<'_, str> {
    if let Some(err) = err.downcast_ref::<&str>() {
        Cow::Borrowed(*err)
    } else if let Some(err) = err.downcast_ref::<String>() {
        Cow::Owned(err.clone())
    } else {
        Cow::Borrowed("unknown panic reason")
    }
}

use core::panic;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures::stream::FuturesUnordered;
use futures::StreamExt;

struct State {
    counter: AtomicUsize,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = Arc::new(State {
        counter: AtomicUsize::new(0),
    });

    let runner = app_task::TaskRunner::new(state);

    let mut handles = FuturesUnordered::new();

    handles.push(runner.spawn_task("Basic task", task));
    handles.push(runner.spawn_task("Fails once task", fails_once));
    handles.push(runner.spawn_task("Panics task", panics));

    while let Some(handle) = handles.next().await {
        handle.expect("A task panicked");
    }
}

async fn task(state: Arc<State>) -> eyre::Result<()> {
    let v = state.counter.load(Ordering::Relaxed);

    tracing::info!(v, "Task");

    Ok(())
}

async fn fails_once(state: Arc<State>) -> eyre::Result<()> {
    let counter = state.counter.fetch_add(1, Ordering::Relaxed);

    if counter == 0 {
        eyre::bail!("Task failed");
    } else {
        tracing::info!("Task succeeded");
    }

    Ok(())
}

async fn panics(_state: Arc<State>) -> eyre::Result<()> {
    tokio::time::sleep(Duration::from_secs(7)).await;

    panic!("Task panicked");
}

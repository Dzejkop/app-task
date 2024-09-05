use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

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

    let basic_handle = runner.spawn_task("Basic task", task);
    let fails_once_handle = runner.spawn_task("Fails once task", fails_once);

    basic_handle.await.unwrap().unwrap();
    fails_once_handle.await.unwrap().unwrap();
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

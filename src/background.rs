//! The process's periodic jobs, and the switch that stops them.
//!
//! Every sweep, sync and poller here has the same shape: wait for the next tick, run, log
//! the failure if there was one, go round again. [`spawn_every`] is that loop, written once.
//! A tick that comes due while the job is still running is skipped rather than queued, so
//! a slow pass never turns into a burst of catch-up passes.
//!
//! On shutdown [`shutdown`] stops the loops from starting another pass and waits a moment
//! for the ones mid-pass to finish, so a sweep is not cut off halfway through a transaction
//! when the container is stopped. [`stopped`] is the same signal for the long-lived tasks
//! that are not periodic, such as open WebSockets.

use std::fmt::Display;
use std::future::Future;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::time::{MissedTickBehavior, interval};

static SHUTDOWN: LazyLock<watch::Sender<bool>> = LazyLock::new(|| watch::channel(false).0);
static JOBS: Mutex<Vec<JoinHandle<()>>> = Mutex::new(Vec::new());

/// Run `job` now and then every `period` until shutdown. A failure is logged under `name`
/// and the loop carries on; the next tick is the retry.
pub fn spawn_every<F, Fut, E>(name: &'static str, period: Duration, mut job: F)
where
    F: FnMut() -> Fut + Send + 'static,
    Fut: Future<Output = Result<(), E>> + Send + 'static,
    E: Display,
{
    let mut stop = SHUTDOWN.subscribe();
    let handle = tokio::spawn(async move {
        let mut ticker = interval(period);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                _ = ticker.tick() => {}
                _ = stop.wait_for(|stopped| *stopped) => break,
            }
            let started = std::time::Instant::now();
            let result = job().await;
            let took = started.elapsed();
            crate::telemetry::metrics::job_pass(name, took, result.is_err());
            // The interval drops the ticks that came due while this pass ran.
            crate::telemetry::metrics::ticks_skipped(
                name,
                (took.as_nanos() / period.as_nanos().max(1)) as u64,
            );
            if let Err(err) = result {
                tracing::error!("{name} failed: {err}");
            }
        }
    });
    JOBS.lock().expect("jobs lock").push(handle);
}

/// Resolves once shutdown has begun. For the tasks that hold something open, so they can
/// say goodbye properly instead of being dropped mid-frame.
pub async fn stopped() {
    let mut stop = SHUTDOWN.subscribe();
    let _ = stop.wait_for(|stopped| *stopped).await;
}

/// Tell everything watching [`stopped`] that the process is on its way out.
pub fn begin_shutdown() {
    SHUTDOWN.send_replace(true);
}

/// Stop every loop from starting another pass, and give the ones mid-pass up to `grace` to
/// finish. Whatever is still running after that is dropped with the runtime.
pub async fn shutdown(grace: Duration) {
    begin_shutdown();
    let jobs = std::mem::take(&mut *JOBS.lock().expect("jobs lock"));
    let all = async {
        for job in jobs {
            let _ = job.await;
        }
    };
    if tokio::time::timeout(grace, all).await.is_err() {
        tracing::warn!("background jobs still running after {grace:?}, stopping anyway");
    }
}

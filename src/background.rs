//! The process's periodic jobs, and the switch that stops them.
//!
//! Every sweep, sync and poller here has the same shape: wait for the next tick, run, log
//! the failure if there was one, go round again. [`spawn_every`] is that loop, written once.
//! A tick that comes due while the job is still running is skipped rather than queued, so
//! a slow pass never turns into a burst of catch-up passes.
//!
//! A panic is a failed pass like any other: [`guarded`] keeps it inside the pass, logs it
//! and the loop goes on. The loops that are not periodic (the killmail feed, the alert
//! workers) run under [`supervise`], which starts one again after it panicked.
//!
//! On shutdown [`shutdown`] stops the loops from starting another pass and waits a moment
//! for the ones mid-pass to finish, so a sweep is not cut off halfway through a transaction
//! when the container is stopped. [`stopped`] is the same signal for the long-lived tasks
//! that are not periodic, such as open WebSockets.

use std::any::Any;
use std::fmt::Display;
use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

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
    E: Display + Send + 'static,
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
            let started = Instant::now();
            // `None` is a panic, already logged.
            let result = guarded(name, &mut job).await;
            let took = started.elapsed();
            crate::telemetry::metrics::job_pass(name, took, !matches!(result, Some(Ok(()))));
            // The interval drops the ticks that came due while this pass ran.
            crate::telemetry::metrics::ticks_skipped(
                name,
                (took.as_nanos() / period.as_nanos().max(1)) as u64,
            );
            if let Some(Err(err)) = result {
                tracing::error!("{name} failed: {err}");
            }
        }
    });
    JOBS.lock().expect("jobs lock").push(handle);
}

/// Run one pass of `name` on a task of its own, so a panic in it ends the pass and not
/// whoever is waiting on it. The panic is logged at error under `name`, which is also what
/// sends it to the error reports, and comes back as `None`.
pub async fn guarded<F, Fut>(name: &str, start: F) -> Option<Fut::Output>
where
    F: FnOnce() -> Fut,
    Fut: Future + Send + 'static,
    Fut::Output: Send + 'static,
{
    // Building the pass can panic too, before there is a future to spawn.
    let panic = match std::panic::catch_unwind(AssertUnwindSafe(start)) {
        Ok(pass) => match tokio::spawn(pass).await {
            Ok(output) => return Some(output),
            Err(err) => match err.try_into_panic() {
                Ok(panic) => panic,
                // Cancelled: the runtime is going away, and the pass with it.
                Err(_) => return None,
            },
        },
        Err(panic) => panic,
    };
    tracing::error!("{name} panicked: {}", panic_message(&*panic));
    None
}

/// What a panic said, when it said it with a string, as nearly all do.
fn panic_message(panic: &(dyn Any + Send)) -> &str {
    panic
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| panic.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("(no message)")
}

/// Tells a bad moment from a stuck job. Upstream services fail now and then and the next
/// try usually works, so a failure is a warning until the job has failed every time for
/// `limit`, which is worth one error.
pub struct Streak {
    limit: Duration,
    /// Since when every try has failed, and whether that has been reported yet.
    state: Mutex<Option<(Instant, bool)>>,
}

impl Streak {
    pub fn new(limit: Duration) -> Streak {
        Streak {
            limit,
            state: Mutex::new(None),
        }
    }

    /// Note one try's outcome. True once per streak, when it first reaches the limit.
    pub fn record(&self, failed: bool, now: Instant) -> bool {
        let mut state = self.state.lock().expect("streak lock");
        if !failed {
            *state = None;
            return false;
        }
        let (since, reported) = state.get_or_insert((now, false));
        if *reported || now.duration_since(*since) < self.limit {
            return false;
        }
        *reported = true;
        true
    }
}

/// How long a loop that panicked waits before [`supervise`] starts it again, the first
/// time. Doubles while it keeps panicking soon after starting, up to [`RESTART_MAX`], so a
/// panic that comes back every time is not a tight loop of error reports.
const RESTART_FIRST: Duration = Duration::from_secs(5);
const RESTART_MAX: Duration = Duration::from_secs(300);

/// Keep a long-lived loop going: should it panic, the panic is logged under `name` and the
/// loop started again a little later, until shutdown. A loop that returns is done.
pub fn supervise<F, Fut>(name: &'static str, start: F)
where
    F: FnMut() -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    supervise_with(name, RESTART_FIRST, start);
}

/// [`supervise`] with a first delay of the caller's. For tests.
#[doc(hidden)]
pub fn supervise_with<F, Fut>(name: &'static str, first: Duration, mut start: F)
where
    F: FnMut() -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    tokio::spawn(async move {
        let mut delay = first;
        loop {
            let started = Instant::now();
            if guarded(name, &mut start).await.is_some() {
                return;
            }
            // A run that lasted a while before it panicked starts the count over.
            if started.elapsed() > RESTART_MAX {
                delay = first;
            }
            tracing::info!("{name}: starting again in {delay:?}");
            tokio::select! {
                _ = tokio::time::sleep(delay) => {}
                _ = stopped() => return,
            }
            delay = (delay * 2).min(RESTART_MAX.max(first));
        }
    });
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_streak_is_reported_once_when_it_reaches_the_limit() {
        let limit = Duration::from_secs(900);
        let streak = Streak::new(limit);
        let start = Instant::now();
        assert!(!streak.record(true, start));
        assert!(!streak.record(true, start + limit / 2));
        assert!(streak.record(true, start + limit));
        assert!(!streak.record(true, start + limit * 2));
        // A success ends the streak, and the next failure starts the clock again.
        assert!(!streak.record(false, start + limit * 2));
        assert!(!streak.record(true, start + limit * 3));
        assert!(streak.record(true, start + limit * 4));
    }
}

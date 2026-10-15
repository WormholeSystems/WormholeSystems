//! A panic in a background job is a failed pass, not the end of the job.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use wormholesystems::background::{guarded, spawn_every, supervise_with};

async fn wait_for(done: impl Fn() -> bool) -> bool {
    for _ in 0..200 {
        if done() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    false
}

#[tokio::test]
async fn a_job_that_panicked_runs_again_on_the_next_tick() {
    let passes = Arc::new(AtomicUsize::new(0));
    let counted = passes.clone();
    spawn_every("panics once", Duration::from_millis(20), move || {
        let pass = counted.fetch_add(1, Ordering::SeqCst);
        async move {
            if pass == 0 {
                panic!("the first pass falls over");
            }
            Ok::<_, std::convert::Infallible>(())
        }
    });
    assert!(wait_for(|| passes.load(Ordering::SeqCst) >= 3).await);
}

#[tokio::test]
async fn a_job_that_panicked_before_its_future_runs_again() {
    let passes = Arc::new(AtomicUsize::new(0));
    let counted = passes.clone();
    spawn_every("panics building", Duration::from_millis(20), move || {
        if counted.fetch_add(1, Ordering::SeqCst) == 0 {
            panic!("no future to run");
        }
        std::future::ready(Ok::<_, std::convert::Infallible>(()))
    });
    assert!(wait_for(|| passes.load(Ordering::SeqCst) >= 3).await);
}

#[tokio::test]
async fn a_panic_comes_back_as_none_and_an_answer_as_some() {
    assert_eq!(guarded("fine", || async { 7 }).await, Some(7));
    assert_eq!(
        guarded("falls over", || async { panic!("on purpose") }).await,
        None::<()>
    );
}

#[tokio::test]
async fn a_supervised_loop_is_started_again_after_a_panic_and_left_once_it_returns() {
    let starts = Arc::new(AtomicUsize::new(0));
    let counted = starts.clone();
    supervise_with("listener", Duration::from_millis(10), move || {
        let start = counted.fetch_add(1, Ordering::SeqCst);
        async move {
            if start < 2 {
                panic!("the feed fell over");
            }
        }
    });
    assert!(wait_for(|| starts.load(Ordering::SeqCst) == 3).await);
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        starts.load(Ordering::SeqCst),
        3,
        "a loop that returned stays done"
    );
}

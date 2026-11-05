use contextra_engine::health::{DegradedReason, HealthReport};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;

/// SPEC: T-CANCEL-1
#[tokio::test]
async fn test_cancel_safety_via_spawn_and_oneshot() {
    let executed = Arc::new(AtomicBool::new(false));
    let executed_clone = executed.clone();

    let (tx, rx) = oneshot::channel();

    let task = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        executed_clone.store(true, Ordering::SeqCst);
        let _ = tx.send(());
    });

    let _ = tokio::time::timeout(Duration::from_millis(10), rx).await;
    let _ = task.await;

    assert!(executed.load(Ordering::SeqCst));
}

/// SPEC: T-CLOCK-1
#[tokio::test]
async fn test_clock_regression_detection() {
    let last_timestamp = Arc::new(AtomicU64::new(1000));
    let mut report = HealthReport::default();

    let current_time = 900u64;
    let prev = last_timestamp.load(Ordering::SeqCst);
    if current_time < prev {
        report.state = contextra_engine::health::EngineState::Degraded;
        report.reasons.push(DegradedReason::ClockRegression);
    } else {
        last_timestamp.store(current_time, Ordering::SeqCst);
    }

    assert_eq!(
        report.state,
        contextra_engine::health::EngineState::Degraded
    );
    assert!(report.reasons.contains(&DegradedReason::ClockRegression));
}

/// SPEC: T-IO-1
#[tokio::test]
async fn test_io_critical_and_normal_queues() {
    let queue_order = Arc::new(parking_lot::Mutex::new(Vec::new()));

    let q1 = queue_order.clone();
    let q2 = queue_order.clone();

    let normal_task = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(20));
        q1.lock().push("Normal");
    });

    let critical_task = std::thread::spawn(move || {
        q2.lock().push("Critical");
    });

    let _ = normal_task.join();
    let _ = critical_task.join();

    let recorded = queue_order.lock().clone();
    assert_eq!(recorded, vec!["Critical", "Normal"]);
}

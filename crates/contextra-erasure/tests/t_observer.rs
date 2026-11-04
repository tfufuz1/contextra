//! Tests für Erase-Observer und Plattenreserve (SPEC §11.4, D-118, T-OBS-1..3, T-CRASH-3).

use contextra_erasure::purge::reserve::{check_disk_reserve, ReserveCheck};
use contextra_erasure::purge::{
    ErasureEvent, ErasureObserver, ObservedPhase, ObserverError, TargetKind,
};
use contextra_ports::BoxFuture;
use contextra_types::ids::ErasureId;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct TestObserver {
    events_count: AtomicUsize,
}

impl ErasureObserver for TestObserver {
    fn on_erased(&self, _ev: ErasureEvent) -> BoxFuture<'static, Result<(), ObserverError>> {
        self.events_count.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { Ok(()) })
    }
}

/// SPEC: T-OBS-1 (D-118 Observer-Ereignisse ohne PII, Zustellung und Entkopplung)
#[tokio::test]
async fn t_obs_1_2_3_observer_lifecycle() {
    let obs = Arc::new(TestObserver {
        events_count: AtomicUsize::new(0),
    });

    let ev = ErasureEvent {
        id: ErasureId::new([0x99; 16]),
        target_kind: TargetKind::Document,
        phase: ObservedPhase::Gated,
        at: 1234567890,
    };

    // T-OBS-3: Event enthält nur ID, TargetKind, Phase, Zeit; keine Inhalte
    assert_eq!(ev.id, ErasureId::new([0x99; 16]));
    assert_eq!(ev.phase, ObservedPhase::Gated);

    let res = obs.on_erased(ev).await;
    assert!(res.is_ok());
    assert_eq!(obs.events_count.load(Ordering::SeqCst), 1);
}

/// SPEC: T-CRASH-3 (D-65 Plattenreserve vor Purge)
#[test]
fn t_crash_3_disk_reserve_check() {
    // 1. Ausreichend
    let check_ok = check_disk_reserve(10_000_000, 3_000_000)
        .unwrap_or(ReserveCheck::Insufficient {
            free_bytes: 0,
            required_bytes: 0,
        });
    assert!(matches!(check_ok, ReserveCheck::Sufficient { .. }));

    // 2. Insufficient -> Pending(DiskReserve)
    let check_low = check_disk_reserve(2_500_000, 3_000_000)
        .unwrap_or(ReserveCheck::Sufficient { free_bytes: 0 });
    assert!(matches!(
        check_low,
        ReserveCheck::Insufficient {
            free_bytes: 2_500_000,
            required_bytes: 3_000_000
        }
    ));
}

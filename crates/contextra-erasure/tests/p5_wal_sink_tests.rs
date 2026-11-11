//! Integrationstests für AUFGABE P5: WAL-Entsorgung nach Scope-Purge und Port WalSink (WP-7, INV-ERA-6).

#[path = "../src/journal/wal_sink.rs"]
mod wal_sink;

#[path = "../src/journal/scope_purge.rs"]
mod scope_purge;

use scope_purge::dispose_scope_wal;
use wal_sink::{InMemoryWalSink, WalDisposalMethod, WalEntryId, WalScope, WalSink, WalSinkError};

use contextra_erasure::types_local::Clock;
use std::sync::atomic::{AtomicU64, Ordering};

struct MockClock {
    time_nanos: AtomicU64,
}

impl MockClock {
    fn new(start_nanos: u64) -> Self {
        Self {
            time_nanos: AtomicU64::new(start_nanos),
        }
    }

    fn advance(&self, delta: u64) {
        self.time_nanos.fetch_add(delta, Ordering::SeqCst);
    }
}

impl Clock for MockClock {
    fn now_unix_nanos(&self) -> u64 {
        self.time_nanos.load(Ordering::SeqCst)
    }

    fn monotonic_nanos(&self) -> u64 {
        self.time_nanos.load(Ordering::SeqCst)
    }
}

/// Invariante INV-ERA-6:
/// WAL-Einträge zu einem gelöschten Scope werden entsorgt oder unlesbar gemacht.
/// Nach dem Purge darf kein Klartext zu dem Scope im WAL lesbar sein.
#[test]
fn inv_era_6_wal_read_after_scope_purge_has_no_plaintext() {
    let sink = InMemoryWalSink::new();
    let clock = MockClock::new(1_000_000_000);

    let scope_a = WalScope::with_prefix([1u8; 16], b"tenant_1/".to_vec());
    let scope_b = WalScope::new([2u8; 16]);

    assert_eq!(scope_a.prefix(), b"tenant_1/");

    sink.set_integrity_key(*scope_a.scope_id(), [42u8; 32]);
    sink.set_integrity_key(*scope_b.scope_id(), [99u8; 32]);

    sink.add_entry(1, scope_a.clone(), b"secret_user_data_a1".to_vec());
    sink.add_entry(2, scope_a.clone(), b"secret_user_data_a2".to_vec());
    sink.add_entry(3, scope_b.clone(), b"other_scope_data_b1".to_vec());

    // Vor dem Purge: Klartext lesbar
    let initial_entries = sink.read_entries_for_scope(&scope_a);
    assert_eq!(initial_entries.len(), 2);
    assert_eq!(initial_entries[0], b"secret_user_data_a1");
    assert_eq!(initial_entries[1], b"secret_user_data_a2");

    // Purge ausführen
    let receipt = dispose_scope_wal(&sink, &scope_a, &clock).unwrap();
    assert_eq!(receipt.disposed_entries_count, 2);
    assert_eq!(receipt.method, WalDisposalMethod::PhysicalPurge);
    assert_eq!(receipt.timestamp_nanos, 1_000_000_000);

    // INV-ERA-6: Nach dem Scope-Purge existiert kein Klartext mehr im WAL
    let after_entries = sink.read_entries_for_scope(&scope_a);
    assert!(
        after_entries.is_empty(),
        "INV-ERA-6 verletzt: Klartext nach Purge noch im WAL lesbar!"
    );

    // Krypto-Schlüssel für Scope A muss verworfen sein
    assert!(
        sink.get_integrity_key(scope_a.scope_id()).is_none(),
        "Integritätsschlüssel von Scope A muss verworfen sein"
    );
}

/// Invariante INV-ERA-2:
/// Purge ist idempotent und wiederaufnehmbar. Wiederholte Aufrufe vollenden
/// halb entsorgte Scopes und bleiben ohne Seiteneffekt stabil.
#[test]
fn dispose_is_idempotent_and_resumable() {
    let sink = InMemoryWalSink::new();
    let clock = MockClock::new(2_000_000_000);

    let scope = WalScope::new([7u8; 16]);
    sink.set_integrity_key(*scope.scope_id(), [11u8; 32]);
    sink.add_entry(10, scope.clone(), b"payload_1".to_vec());
    sink.add_entry(11, scope.clone(), b"payload_2".to_vec());

    // Erster Purge-Durchlauf
    let receipt1 = dispose_scope_wal(&sink, &scope, &clock).unwrap();
    assert_eq!(receipt1.disposed_entries_count, 2);
    assert_eq!(receipt1.method, WalDisposalMethod::PhysicalPurge);
    assert_eq!(receipt1.timestamp_nanos, 2_000_000_000);

    // Scope gilt als entsorgt
    assert!(sink.is_scope_disposed(&scope).unwrap());

    // Zeit vergeht
    clock.advance(500_000_000);

    // Zweiter Aufruf (Idempotenz / Wiederaufnahme)
    let receipt2 = dispose_scope_wal(&sink, &scope, &clock).unwrap();
    assert_eq!(receipt2.disposed_entries_count, 0);
    assert_eq!(receipt2.method, WalDisposalMethod::AlreadyDisposed);
    assert_eq!(receipt2.timestamp_nanos, 2_500_000_000);

    // Weiterhin kein Klartext vorhanden
    assert!(sink.read_entries_for_scope(&scope).is_empty());
}

/// Sicherstellen, dass andere Scopes nicht berührt werden (strikte Scope-Isolation).
#[test]
fn other_scopes_wal_untouched() {
    let sink = InMemoryWalSink::new();
    let clock = MockClock::new(3_000_000_000);

    let scope_target = WalScope::new([10u8; 16]);
    let scope_other = WalScope::new([20u8; 16]);

    sink.set_integrity_key(*scope_target.scope_id(), [101u8; 32]);
    sink.set_integrity_key(*scope_other.scope_id(), [202u8; 32]);

    sink.add_entry(1, scope_target.clone(), b"target_data".to_vec());
    sink.add_entry(2, scope_other.clone(), b"preserve_this_data".to_vec());

    // Target purgen
    let receipt = dispose_scope_wal(&sink, &scope_target, &clock).unwrap();
    assert_eq!(receipt.disposed_entries_count, 1);

    // Target ist leer
    assert!(sink.read_entries_for_scope(&scope_target).is_empty());

    // Scope_other ist unberührt und Klartext vollständig erhalten
    let other_entries = sink.read_entries_for_scope(&scope_other);
    assert_eq!(other_entries.len(), 1);
    assert_eq!(other_entries[0], b"preserve_this_data");

    // Schlüssel von scope_other ist weiterhin intakt
    assert_eq!(
        sink.get_integrity_key(scope_other.scope_id()),
        Some([202u8; 32])
    );
}

/// Invariante INV-VAULT-FAILCLOSED-1 / Fail-Closed:
/// Bei E/A-Fehler oder Fehler bei der Schlüsselvernichtung darf der Scope NICHT als entsorgt gemeldet werden.
#[test]
fn sink_error_is_fail_closed() {
    let sink = InMemoryWalSink::new();
    let clock = MockClock::new(4_000_000_000);

    let scope = WalScope::new([55u8; 16]);
    sink.set_integrity_key(*scope.scope_id(), [77u8; 32]);
    sink.add_entry(1, scope.clone(), b"sensitive_fail_test".to_vec());

    let entry_id = WalEntryId::new(1);
    assert_eq!(entry_id.as_u64(), 1);
    assert_eq!(entry_id.to_string(), "WalEntryId(1)");

    // 1. Simuliere Fehler beim Enumerieren
    sink.set_fail_on_enumerate(true);
    let err_enum = dispose_scope_wal(&sink, &scope, &clock);
    assert!(err_enum.is_err(), "Muss fail-closed fehlschlagen");
    assert!(!sink.is_scope_disposed(&scope).unwrap());

    sink.set_fail_on_enumerate(false);

    // 2. Simuliere Fehler beim Entsorgen der Einträge
    sink.set_fail_on_dispose(true);
    let err_disp = dispose_scope_wal(&sink, &scope, &clock);
    assert!(err_disp.is_err(), "Muss fail-closed fehlschlagen");
    assert!(!sink.is_scope_disposed(&scope).unwrap());

    sink.set_fail_on_dispose(false);

    // 3. Simuliere Fehler bei Schlüsselvernichtung
    sink.set_fail_on_key_discard(true);
    let err_discard = dispose_scope_wal(&sink, &scope, &clock);
    assert!(err_discard.is_err(), "Muss fail-closed fehlschlagen");
    assert!(
        !sink.is_scope_disposed(&scope).unwrap(),
        "Scope darf nach Fehler NICHT als entsorgt markiert sein"
    );

    // In Schritt 3 wurden die Einträge bereits getilgt, bevor der Key-Discard fehlschlug.
    // Beim erneuten Aufruf (Wiederaufnahme) sind keine neuen unbereinigten Einträge vorhanden,
    // daher vollendet der Aufruf die Schlüsselvernichtung via CryptoShredding.
    sink.set_fail_on_key_discard(false);

    let receipt = dispose_scope_wal(&sink, &scope, &clock).unwrap();
    assert_eq!(receipt.method, WalDisposalMethod::CryptoShredding);
    assert!(sink.is_scope_disposed(&scope).unwrap());
    assert!(sink.read_entries_for_scope(&scope).is_empty());
}

/// Testet die Fehler- und Display-Repräsentationen
#[test]
fn wal_sink_error_variants_and_display() {
    let err_corrupt = WalSinkError::Corrupt {
        phase: "header_check",
        detail: "magic mismatch".to_string(),
    };
    assert!(err_corrupt.to_string().contains("magic mismatch"));

    let err_unavailable = WalSinkError::SinkUnavailable {
        detail: "storage disconnected".to_string(),
    };
    assert!(err_unavailable.to_string().contains("storage disconnected"));

    assert_eq!(
        WalDisposalMethod::PhysicalPurge.to_string(),
        "PhysicalPurge"
    );
    assert_eq!(
        WalDisposalMethod::CryptoShredding.to_string(),
        "CryptoShredding"
    );
    assert_eq!(
        WalDisposalMethod::AlreadyDisposed.to_string(),
        "AlreadyDisposed"
    );
}

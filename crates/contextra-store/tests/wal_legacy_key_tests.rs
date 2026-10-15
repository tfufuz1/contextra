use contextra_core::{ContextraError, TxId};
use contextra_store::wal::{Wal, WalEntry, WalOp, WAL_V3_HEADER};
use std::sync::{Arc, Mutex};
use tempfile::tempdir;
use tokio::fs;

struct LogCollector(Arc<Mutex<Vec<String>>>);

struct MessageVisitor(String);
impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" || field.name() == "1" {
            self.0.push_str(&format!("{:?}", value));
        } else {
            self.0.push_str(&format!(" {}={:?}", field.name(), value));
        }
    }
}

impl tracing::Subscriber for LogCollector {
    fn enabled(&self, _metadata: &tracing::Metadata<'_>) -> bool {
        true
    }
    fn new_span(&self, _span: &tracing::span::Attributes<'_>) -> tracing::Id {
        tracing::Id::from_u64(1)
    }
    fn record(&self, _span: &tracing::Id, _values: &tracing::span::Record<'_>) {}
    fn record_follows_from(&self, _span: &tracing::Id, _follows: &tracing::Id) {}
    fn event(&self, event: &tracing::Event<'_>) {
        let mut visitor = MessageVisitor(String::new());
        event.record(&mut visitor);
        self.0.lock().unwrap().push(visitor.0);
    }
    fn enter(&self, _span: &tracing::Id) {}
    fn exit(&self, _span: &tracing::Id) {}
}

#[tokio::test]
async fn test_regular_open_rejects_legacy_integrity_key() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("legacy_signed.wal");

    // Manually construct a WAL segment signed exclusively with the legacy integrity key
    let op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"secret_key".to_vec(),
        value: b"secret_val".to_vec(),
    };
    let legacy_key = Wal::legacy_integrity_key_for_test();
    let legacy_entry = WalEntry::try_new(op, 1, &legacy_key, [0u8; 32]).expect("legacy entry");

    let mut wal_bytes = Vec::new();
    wal_bytes.extend_from_slice(&WAL_V3_HEADER);
    wal_bytes.extend_from_slice(&legacy_entry.to_bytes().expect("to_bytes"));

    fs::write(&wal_path, wal_bytes)
        .await
        .expect("write legacy WAL");

    // Regular Wal::open (WalConfig::default()) MUST reject the legacy-signed segment
    let open_res = Wal::open(&wal_path).await;
    assert!(
        open_res.is_err(),
        "Regular Wal::open must reject WAL segment signed exclusively with legacy key"
    );
    let err = open_res.unwrap_err();
    assert!(
        matches!(err, ContextraError::WalCorruption { .. }),
        "Expected WalCorruption error, got: {:?}",
        err
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_open_for_legacy_migration_accepts_and_logs_warning() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("legacy_migration.wal");

    // Construct legacy WAL segment
    let op = WalOp::Put {
        tx_id: TxId::new(10),
        key: b"legacy_user_key".to_vec(),
        value: b"legacy_user_val".to_vec(),
    };
    let legacy_key = Wal::legacy_integrity_key_for_test();
    let legacy_entry = WalEntry::try_new(op, 10, &legacy_key, [0u8; 32]).expect("legacy entry");

    let mut wal_bytes = Vec::new();
    wal_bytes.extend_from_slice(&WAL_V3_HEADER);
    wal_bytes.extend_from_slice(&legacy_entry.to_bytes().expect("to_bytes"));

    fs::write(&wal_path, wal_bytes)
        .await
        .expect("write legacy WAL");

    // Set up log capture
    let logs = Arc::new(Mutex::new(Vec::<String>::new()));
    let collector = LogCollector(logs.clone());

    let _guard = tracing::subscriber::set_default(collector);
    tracing::callsite::rebuild_interest_cache();

    let wal = Wal::open_for_legacy_migration(&wal_path, None)
        .await
        .expect("open_for_legacy_migration must accept legacy segment");

    let entries = wal.replay().await.expect("replay legacy segment");

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].1.seq_no, 10);
    if let WalOp::Put { key, value, .. } = &entries[0].1.op {
        assert_eq!(key, b"legacy_user_key");
        assert_eq!(value, b"legacy_user_val");
    } else {
        panic!("Expected Put op");
    }

    // Verify expected warning log entry was generated
    let captured_logs = logs.lock().unwrap();
    let warning_found = captured_logs.iter().any(|msg| {
        msg.contains("Legacy-WAL-Integritätsschlüssel aktiv für Segment")
            && msg.contains("öffentlich im Quellcode liegt")
    });

    assert!(
        warning_found,
        "Expected tracing::warn log on legacy key fallback usage. Captured logs: {:?}",
        *captured_logs
    );
}

#[tokio::test]
async fn test_fallback_remains_inactive_without_explicit_migration_call() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("legacy_inactive.wal");

    let op = WalOp::Put {
        tx_id: TxId::new(42),
        key: b"unmigrated_key".to_vec(),
        value: b"unmigrated_val".to_vec(),
    };
    let legacy_key = Wal::legacy_integrity_key_for_test();
    let legacy_entry = WalEntry::try_new(op, 42, &legacy_key, [0u8; 32]).expect("entry");

    let mut wal_bytes = Vec::new();
    wal_bytes.extend_from_slice(&WAL_V3_HEADER);
    wal_bytes.extend_from_slice(&legacy_entry.to_bytes().expect("to_bytes"));
    fs::write(&wal_path, wal_bytes).await.expect("write wal");

    // Standard open MUST fail without explicit migration call
    let open_res = Wal::open(&wal_path).await;
    assert!(
        open_res.is_err(),
        "Standard Wal::open must fail on unmigrated legacy WAL file"
    );

    // Read-only handle replay MUST fail without explicit migration call
    let read_only_wal = Wal::open_read_only(&wal_path, None)
        .await
        .expect("open_read_only creates read-only handle");
    let read_only_replay = read_only_wal.replay().await;
    assert!(
        read_only_replay.is_err(),
        "Replay on unmigrated legacy WAL file without explicit migration must fail"
    );

    let marker_path = dir.path().join("legacy_inactive.wal.rekeyed");
    assert!(
        !marker_path.exists(),
        ".rekeyed marker must not be written without explicit migration"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_explicit_migrate_legacy_wal_produces_structured_audit_log_and_v3_segment() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("legacy_structured_mig.wal");

    let op = WalOp::Put {
        tx_id: TxId::new(100),
        key: b"mig_key".to_vec(),
        value: b"mig_val".to_vec(),
    };
    let legacy_key = Wal::legacy_integrity_key_for_test();
    let legacy_entry = WalEntry::try_new(op, 100, &legacy_key, [0u8; 32]).expect("entry");

    let mut wal_bytes = Vec::new();
    wal_bytes.extend_from_slice(&WAL_V3_HEADER);
    wal_bytes.extend_from_slice(&legacy_entry.to_bytes().expect("to_bytes"));
    fs::write(&wal_path, wal_bytes).await.expect("write wal");

    let logs = Arc::new(Mutex::new(Vec::<String>::new()));
    let collector = LogCollector(logs.clone());

    let _guard = tracing::subscriber::set_default(collector);
    tracing::callsite::rebuild_interest_cache();

    // Explicit operator invocation
    let migrated = Wal::migrate_legacy_wal(&wal_path, None)
        .await
        .expect("migrate_legacy_wal must succeed");
    assert!(
        migrated,
        "migrate_legacy_wal must return true on legacy segment"
    );

    // Check structured audit log
    let captured_logs = logs.lock().unwrap();
    let audit_found = captured_logs.iter().any(|msg| {
        msg.contains("legacy_integrity_key_fallback_used")
            || msg.contains("Legacy-WAL-Integritätsschlüssel aktiv für Segment")
    });

    assert!(
        audit_found,
        "Expected structured audit log for legacy key fallback. Captured logs: {:?}",
        *captured_logs
    );

    // Re-opening with standard Wal::open MUST now succeed without legacy fallback
    let std_wal = Wal::open(&wal_path)
        .await
        .expect("Standard open must succeed on migrated WAL");

    assert!(
        !std_wal.legacy_key_used_for_test(),
        "Legacy key path must NOT be taken for post-migration WAL segment"
    );
    assert!(
        !std_wal.allow_legacy_fallback_for_test(),
        "Legacy fallback flag must be false after migration"
    );

    let entries = std_wal.replay().await.expect("replay migrated wal");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].1.seq_no, 100);
}

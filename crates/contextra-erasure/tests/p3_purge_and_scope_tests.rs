//! P3 Integrationstests: PurgePhase (WP-2) und dauerhafter Scope mit Single-Flight (WP-3, INV-ERA-3).

use contextra_erasure::error::ErasureError;
use contextra_erasure::purge::{PurgePhase, PurgePhaseError, PurgeProgressTracker};
use contextra_erasure::scope::codec::{decode_scope, encode_scope};
use contextra_erasure::scope::set::{LayerScopeSet, ResolvedLayerScope};
use contextra_erasure::scope::single_flight::{ScopeLockError, ScopeLockManager};
use contextra_erasure::types_local::{Clock, Vfs};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[cfg(feature = "async")]
use contextra_erasure::scope::file::{load_scope_file, store_scope_file};
#[cfg(feature = "async")]
use contextra_erasure::types_local::{FileId, FileKey, FileKeyring, FileKind};
#[cfg(feature = "async")]
use futures_util::future::BoxFuture;

/// Mock Clock für deterministische Zeittests (keine Wanduhr).
#[derive(Debug, Default)]
struct MockClock {
    now_monotonic_nanos: AtomicU64,
}

impl MockClock {
    fn new(start_nanos: u64) -> Self {
        Self {
            now_monotonic_nanos: AtomicU64::new(start_nanos),
        }
    }

    fn advance(&self, delta_nanos: u64) {
        self.now_monotonic_nanos
            .fetch_add(delta_nanos, Ordering::SeqCst);
    }
}

impl Clock for MockClock {
    fn now_unix_nanos(&self) -> u64 {
        1_700_000_000_000_000_000 + self.monotonic_nanos()
    }

    fn monotonic_nanos(&self) -> u64 {
        self.now_monotonic_nanos.load(Ordering::SeqCst)
    }
}

/// In-Memory Vfs für simulierte Crash- und E/A-Tests.
#[derive(Default)]
struct MemoryVfs {
    files: RwLock<BTreeMap<PathBuf, Vec<u8>>>,
}

impl Vfs for MemoryVfs {
    fn read(&self, path: &Path) -> Result<Vec<u8>, ErasureError> {
        let guard = self.files.read();
        guard.get(path).cloned().ok_or_else(|| {
            ErasureError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "not found",
            ))
        })
    }

    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<(), ErasureError> {
        self.files.write().insert(path.to_path_buf(), data.to_vec());
        Ok(())
    }

    fn create_dir_all(&self, _path: &Path) -> Result<(), ErasureError> {
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> Result<(), ErasureError> {
        let mut guard = self.files.write();
        if guard.remove(path).is_some() {
            Ok(())
        } else {
            Err(ErasureError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "not found",
            )))
        }
    }

    fn exists(&self, path: &Path) -> Result<bool, ErasureError> {
        Ok(self.files.read().contains_key(path))
    }

    fn len(&self, path: &Path) -> Result<u64, ErasureError> {
        let guard = self.files.read();
        let bytes = guard.get(path).ok_or_else(|| {
            ErasureError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "not found",
            ))
        })?;
        Ok(bytes.len() as u64)
    }

    fn list_dir(&self, _path: &Path) -> Result<Vec<String>, ErasureError> {
        Ok(Vec::new())
    }

    fn sync_dir(&self, _path: &Path) -> Result<(), ErasureError> {
        Ok(())
    }
}

/// In-Memory Keyring für Scope-Dateien.
#[cfg(feature = "async")]
struct MemoryKeyring {
    keys: RwLock<BTreeMap<u64, Vec<u8>>>,
}

#[cfg(feature = "async")]
impl MemoryKeyring {
    fn new() -> Self {
        Self {
            keys: RwLock::new(BTreeMap::new()),
        }
    }
}

#[cfg(feature = "async")]
impl FileKeyring for MemoryKeyring {
    fn create(&self, _kind: FileKind, id: FileId) -> BoxFuture<'_, Result<FileKey, String>> {
        Box::pin(async move {
            let key = vec![0x33u8; 32];
            self.keys.write().insert(id.as_u64(), key.clone());
            Ok(FileKey::new(key, id))
        })
    }

    fn open(&self, id: FileId) -> BoxFuture<'_, Result<FileKey, String>> {
        Box::pin(async move {
            let guard = self.keys.read();
            let key = guard
                .get(&id.as_u64())
                .cloned()
                .ok_or_else(|| "key not found".to_string())?;
            Ok(FileKey::new(key, id))
        })
    }

    fn destroy(&self, id: FileId) -> BoxFuture<'_, Result<(), String>> {
        Box::pin(async move {
            self.keys.write().remove(&id.as_u64());
            Ok(())
        })
    }
}

#[test]
fn purge_phase_transition_matrix_is_exhaustive() {
    let all_phases = [
        PurgePhase::Admitted,
        PurgePhase::Fenced,
        PurgePhase::Flushed,
        PurgePhase::Rewriting,
        PurgePhase::Disposing,
        PurgePhase::WalRetiring,
        PurgePhase::Witnessing,
        PurgePhase::Finalizing,
        PurgePhase::Released,
    ];

    assert_eq!(all_phases.len(), 9);

    for (from_idx, &from) in all_phases.iter().enumerate() {
        for (to_idx, &to) in all_phases.iter().enumerate() {
            let allowed = from.can_transition_to(to);

            // Gültig genau dann wenn:
            // 1. Derselbe Zustand (Idempotenz)
            // 2. Genau der nächste lineare Schritt (from_idx + 1 == to_idx)
            let expected = (from_idx == to_idx) || (from_idx + 1 == to_idx);

            assert_eq!(
                allowed, expected,
                "Transition von {:?} nach {:?} sollte {} sein",
                from, to, expected
            );

            // Prüfe transition_to Verhalten
            let mut state = from;
            let res = state.transition_to(to);
            if expected {
                assert!(
                    res.is_ok(),
                    "Expected Ok for transition {:?} -> {:?}",
                    from,
                    to
                );
                assert_eq!(state, to);
            } else {
                assert!(
                    matches!(res, Err(PurgePhaseError::InvalidTransition { .. })),
                    "Expected InvalidTransition for {:?} -> {:?}",
                    from,
                    to
                );
                assert_eq!(state, from, "State must not change on invalid transition");
            }
        }
    }

    // Zusätzliche Tests für Phasen-Codes und Fehler
    for code in 1..=9 {
        let phase = PurgePhase::try_from(code).expect("valid code");
        assert_eq!(phase.code(), code);
    }
    assert!(PurgePhase::try_from(0).is_err());
    assert!(PurgePhase::try_from(10).is_err());
}

#[test]
fn inv_era_3_single_flight_after_simulated_crash_between_two_starts() {
    let vfs = Arc::new(MemoryVfs::default());
    let clock = Arc::new(MockClock::new(1_000_000));
    let lease_nanos = 5_000_000; // 5 ms Lease

    let manager = ScopeLockManager::new(vfs.as_ref(), clock.as_ref(), lease_nanos);

    let scope_id = [0xAA; 16];
    let owner1 = [0x11; 16];
    let owner2 = [0x22; 16];

    // 1. Erster Purge-Prozess erwirbt Lock auf Scope
    let guard1 = manager
        .acquire(scope_id, owner1)
        .expect("First acquire must succeed");
    assert_eq!(guard1.epoch(), 1);
    assert_eq!(guard1.owner_id(), &owner1);

    // 2. Zweiter Purge-Prozess versucht zeitgleich denselben Scope zu purgen -> schlägt fehl (Single Flight, INV-ERA-3)
    let err = manager.acquire(scope_id, owner2);
    assert!(
        matches!(
            err,
            Err(ScopeLockError::ScopeLocked {
                scope_id: s,
                owner_id: o,
                epoch: 1
            }) if s == scope_id && o == owner1
        ),
        "Expected ScopeLocked, got {:?}",
        err
    );

    // 3. Simulierter Crash von owner1: guard1 wird ohne release() fallen gelassen.
    drop(guard1);

    // Unmittelbar nach dem Crash ist das Lock noch nicht abgelaufen:
    clock.advance(1_000); // 1 µs vergangen
    let err_after_crash = manager.acquire(scope_id, owner2);
    assert!(matches!(
        err_after_crash,
        Err(ScopeLockError::ScopeLocked { .. })
    ));

    // 4. Nach Ablauf der Lease (stale lock) startet owner2 erneut:
    clock.advance(lease_nanos + 1);
    let guard2 = manager
        .acquire(scope_id, owner2)
        .expect("Acquiring stale lock must succeed with incremented epoch");

    // Epoche wurde sauber inkrementiert
    assert_eq!(guard2.epoch(), 2);
    assert_eq!(guard2.owner_id(), &owner2);

    // Zu jedem Zeitpunkt lief genau ein einziger Purge auf diesem Scope (INV-ERA-3)
    guard2.release().expect("Release must succeed");
    assert!(!manager
        .is_locked(&scope_id)
        .expect("Lock check must succeed"));
}

#[cfg(feature = "async")]
#[tokio::test]
async fn scope_file_roundtrip_and_corruption_fails_closed() {
    let vfs = Arc::new(MemoryVfs::default());
    let keyring = Arc::new(MemoryKeyring::new());
    let erasure_id = [0x42; 16];
    let file_id = FileId::new(1001);

    let mut scope_set = LayerScopeSet::new();
    let layer1 = ResolvedLayerScope::with_units([1u8; 16], vec![10, 20, 30]);
    let layer2 = ResolvedLayerScope::with_units([2u8; 16], vec![100, 200]);
    scope_set.push(layer1);
    scope_set.push(layer2);

    // 1. Schreiben der persistenten Scope-Datei
    store_scope_file(
        vfs.as_ref(),
        keyring.as_ref(),
        &erasure_id,
        file_id,
        &scope_set,
    )
    .await
    .expect("Store scope file must succeed");

    // 2. Erfolgreiches Laden
    let loaded = load_scope_file(vfs.as_ref(), keyring.as_ref(), &erasure_id, file_id)
        .await
        .expect("Load scope file must succeed");
    assert_eq!(loaded.total_units(), 5);
    assert_eq!(loaded.scopes.len(), 2);
    assert_eq!(loaded.scopes[0].units, vec![10, 20, 30]);

    // 3. Korruptionstest: Bitflip im Body/Envelope -> Fail-Closed
    let path = contextra_erasure::scope::file::scope_file_path(&erasure_id);
    let mut raw_bytes = vfs.read(&path).expect("File must exist");
    let last_idx = raw_bytes.len() - 1;
    raw_bytes[last_idx] ^= 0xFF;
    vfs.write_atomic(&path, &raw_bytes)
        .expect("Write tampered file");

    let tampered_res = load_scope_file(vfs.as_ref(), keyring.as_ref(), &erasure_id, file_id).await;
    assert!(
        matches!(tampered_res, Err(ErasureError::Corrupt)),
        "Tampered envelope must fail-closed with ErasureError::Corrupt, got {:?}",
        tampered_res
    );

    // 4. Trunkierungstest: Datei auf 20 Bytes gekürzt -> Fail-Closed
    let truncated_bytes = &raw_bytes[0..20];
    vfs.write_atomic(&path, truncated_bytes)
        .expect("Write truncated file");

    let truncated_res =
        load_scope_file(vfs.as_ref(), keyring.as_ref(), &erasure_id, file_id).await;
    assert!(
        matches!(truncated_res, Err(ErasureError::Corrupt)),
        "Truncated file must fail-closed with ErasureError::Corrupt, got {:?}",
        truncated_res
    );

    // 5. Codec-Direkttest: Beschädigung der Prüfsumme
    let valid_encoded = encode_scope(&scope_set).expect("Encode must succeed");
    let decoded = decode_scope(&valid_encoded).expect("Decode must succeed");
    assert_eq!(decoded.total_units(), 5);

    let mut tampered_codec = valid_encoded.clone();
    tampered_codec[20] ^= 0xEE; // Bitflip in Prüfsumme oder Scopes
    let codec_err = decode_scope(&tampered_codec);
    assert!(
        matches!(codec_err, Err(ErasureError::Corrupt)),
        "Tampered scope codec bytes must fail closed, got {:?}",
        codec_err
    );
}

#[cfg(not(feature = "async"))]
#[test]
fn scope_file_roundtrip_and_corruption_fails_closed() {
    let mut scope_set = LayerScopeSet::new();
    let layer1 = ResolvedLayerScope::with_units([1u8; 16], vec![10, 20, 30]);
    let layer2 = ResolvedLayerScope::with_units([2u8; 16], vec![100, 200]);
    scope_set.push(layer1);
    scope_set.push(layer2);

    // Codec-Direkttest ohne async Vfs
    let valid_encoded = encode_scope(&scope_set).expect("Encode must succeed");
    let decoded = decode_scope(&valid_encoded).expect("Decode must succeed");
    assert_eq!(decoded.total_units(), 5);

    let mut tampered_codec = valid_encoded.clone();
    tampered_codec[20] ^= 0xEE; // Bitflip in Prüfsumme oder Scopes
    let codec_err = decode_scope(&tampered_codec);
    assert!(
        matches!(codec_err, Err(ErasureError::Corrupt)),
        "Tampered scope codec bytes must fail closed, got {:?}",
        codec_err
    );
}

#[test]
fn stale_lock_is_recovered_without_double_purge() {
    let vfs = Arc::new(MemoryVfs::default());
    let clock = Arc::new(MockClock::new(10_000_000));
    let lease_nanos = 2_000_000;

    let manager = ScopeLockManager::new(vfs.as_ref(), clock.as_ref(), lease_nanos);
    let scope_id = [0x55; 16];
    let owner_a = [0x01; 16];
    let owner_b = [0x02; 16];

    // Node A holt sich das Lock
    let guard_a = manager
        .acquire(scope_id, owner_a)
        .expect("Node A acquires lock");
    assert_eq!(guard_a.epoch(), 1);

    // Zeit vergeht, Node A hängt sich auf / stürzt ab (kein Heartbeat/Release)
    clock.advance(lease_nanos + 100);

    // Node B stellt fest, dass Lock stale ist und übernimmt es geordnet mit neuer Epoche 2
    let guard_b = manager
        .acquire(scope_id, owner_b)
        .expect("Node B recovers stale lock");
    assert_eq!(guard_b.epoch(), 2);

    // Falls der alte Prozess (Node A) jetzt erwacht und versucht, das Lock freizugeben,
    // scheitert dies wegen Epochen-Konflikt (EpochMismatch) -> keine Race Condition!
    let release_err = guard_a.release().unwrap_err();
    assert!(
        matches!(
            release_err,
            ScopeLockError::EpochMismatch {
                expected: 1,
                actual: 2
            }
        ),
        "Zombie process must fail to release with EpochMismatch, got {:?}",
        release_err
    );

    // Node B führt Purge sicher zu Ende
    guard_b.release().expect("Node B releases successfully");
    assert!(!manager.is_locked(&scope_id).expect("Scope is now unlocked"));
}

#[test]
fn test_purge_progress_tracker_resume_api() {
    let purge_id = [0x77; 16];
    let mut tracker = PurgeProgressTracker::new(purge_id);

    assert_eq!(tracker.current_phase(), PurgePhase::Admitted);
    assert_eq!(tracker.next_phase(), Some(PurgePhase::Fenced));
    assert!(!tracker.is_completed());

    // Schrittweises Avancieren
    assert_eq!(tracker.advance_to_next().unwrap(), PurgePhase::Fenced);
    assert_eq!(tracker.advance_to_next().unwrap(), PurgePhase::Flushed);
    assert_eq!(tracker.advance_to_next().unwrap(), PurgePhase::Rewriting);
    assert_eq!(tracker.advance_to_next().unwrap(), PurgePhase::Disposing);
    assert_eq!(tracker.advance_to_next().unwrap(), PurgePhase::WalRetiring);
    assert_eq!(tracker.advance_to_next().unwrap(), PurgePhase::Witnessing);
    assert_eq!(tracker.advance_to_next().unwrap(), PurgePhase::Finalizing);
    assert_eq!(tracker.advance_to_next().unwrap(), PurgePhase::Released);

    assert!(tracker.is_completed());
    assert_eq!(tracker.next_phase(), None);
    assert!(tracker.advance_to_next().is_err());

    // Recovery-Test nach Crash bei Phase Rewriting (4)
    let recovered = PurgeProgressTracker::recover(purge_id, 4, 1).expect("Recover must succeed");
    assert_eq!(recovered.current_phase(), PurgePhase::Rewriting);
    assert_eq!(recovered.next_phase(), Some(PurgePhase::Disposing));
    assert_eq!(recovered.attempts, 2);

    // Ungültiger Code bei Recovery scheitert fail-closed
    assert!(PurgeProgressTracker::recover(purge_id, 0, 1).is_err());
    assert!(PurgeProgressTracker::recover(purge_id, 10, 1).is_err());
}

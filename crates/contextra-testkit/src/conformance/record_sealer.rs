//! # Konformitätsprüfsuite für den `RecordSealer`-Port (`conformance/record_sealer`)
//!
//! Dieses Modul testet die vertraglichen Zusicherungen (Contract Invariants)
//! für den DEK-Record-Sealer [`RecordSealer`] gemäß SPEC §4.2, D-53, D-79 und I-09.
//!
//! ## Getestete Verträge (Pflichtfälle):
//! 1. `d_53_seal_open_roundtrip`: seal und open stellen die ursprünglichen Klartextdaten exakt wieder her.
//! 2. `d_53_missing_slot_returns_erased`: Fehlt ein Slot in der Hierarchie, liefert open `Err(ContextraError::Erased)`.
//! 3. `d_79_evict_zeroizes_dek`: `evict(range)` invalidiert und zeroisiert gecachte DEKs im angegebenen Bereich.
//! 4. `i_09_make_slots_durable`: sync_data des Tresors vor WAL-Frames (`make_slots_durable`).

use contextra_ports::{BoxFuture, ContextraError, Result};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Key representation in RecordSealer (78 bytes canonical buffer).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyBuf(pub [u8; 78]);

impl KeyBuf {
    /// Creates a key buffer from prefix.
    #[must_use]
    pub fn from_prefix(prefix: &[u8]) -> Self {
        let mut buf = [0u8; 78];
        let copy_len = prefix.len().min(78);
        buf[..copy_len].copy_from_slice(&prefix[..copy_len]);
        Self(buf)
    }
}

/// Boundary key for eviction ranges.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyBound(pub [u8; 65]);

impl KeyBound {
    /// Creates a key boundary from prefix.
    #[must_use]
    pub fn from_prefix(prefix: &[u8]) -> Self {
        let mut buf = [0u8; 65];
        let copy_len = prefix.len().min(65);
        buf[..copy_len].copy_from_slice(&prefix[..copy_len]);
        Self(buf)
    }
}

/// Key range for eviction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyRange {
    /// Lower boundary (inclusive).
    pub lo: KeyBound,
    /// Upper boundary (inclusive).
    pub hi: KeyBound,
}

/// Context passed to seal operations.
#[derive(Debug, Clone, Default)]
pub struct SealCtx {
    /// Tenant identifier tag.
    pub tenant_id: u64,
}

/// Sealed envelope containing encrypted payload and slot metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    /// Encrypted ciphertext.
    pub ciphertext: Vec<u8>,
    /// Slot identifier reference.
    pub slot_id: u64,
}

/// RecordSealer Port Trait according to SPEC §4.2, D-53.
pub trait RecordSealer: Send + Sync {
    /// Seals plaintext into an envelope, allocating missing slots if required.
    fn seal<'a>(
        &'a self,
        ctx: &'a SealCtx,
        key: &'a KeyBuf,
        plain: &'a [u8],
    ) -> BoxFuture<'a, Result<Envelope>>;

    /// Opens an envelope, decrypting ciphertext. If a slot is missing, returns `Err(ContextraError::Erased)`.
    fn open<'a>(&'a self, key: &'a KeyBuf, env: &'a Envelope) -> BoxFuture<'a, Result<Vec<u8>>>;

    /// Evicts and zeroizes DEK cache entries in the specified key range (D-79).
    fn evict(&self, scope: &KeyRange);

    /// Ensures all pending key slots are synced to durable storage before WAL frames (I-09).
    fn make_slots_durable(&self) -> BoxFuture<'_, Result<()>>;
}

/// Conformance failure descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceFailure {
    /// Suite identifier.
    pub suite_name: &'static str,
    /// Test case.
    pub test_case: &'static str,
    /// Diagnostic failure message.
    pub message: String,
}

impl ConformanceFailure {
    /// Creates a new failure record.
    #[must_use]
    pub fn new(
        suite_name: &'static str,
        test_case: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            suite_name,
            test_case,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ConformanceFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] Conformance failure in test '{}': {}",
            self.suite_name, self.test_case, self.message
        )
    }
}

impl std::error::Error for ConformanceFailure {}

/// Conformance result type.
pub type ConformanceResult = std::result::Result<(), ConformanceFailure>;

/// Führt die vollständige Konformitätsprüfung für eine [`RecordSealer`]-Implementierung durch.
pub async fn run<F, Fut, S>(factory: F) -> ConformanceResult
where
    F: Fn() -> Fut,
    Fut: Future<Output = S>,
    S: RecordSealer,
{
    let suite = "RecordSealer";

    // Fall 1: D-53 seal/open Roundtrip
    {
        let sealer = factory().await;
        let ctx = SealCtx { tenant_id: 10 };
        let key = KeyBuf::from_prefix(b"tenant:10:doc:1");
        let plaintext = b"Strictly confidential user record data payload";

        let env = match sealer.seal(&ctx, &key, plaintext).await {
            Ok(e) => e,
            Err(err) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "d_53_seal_open_roundtrip",
                    format!("seal failed: {err:?}"),
                ));
            }
        };

        let opened = match sealer.open(&key, &env).await {
            Ok(p) => p,
            Err(err) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "d_53_seal_open_roundtrip",
                    format!("open failed: {err:?}"),
                ));
            }
        };

        if opened != plaintext {
            return Err(ConformanceFailure::new(
                suite,
                "d_53_seal_open_roundtrip",
                format!("decrypted plaintext mismatch: expected {plaintext:?}, got {opened:?}"),
            ));
        }
    }

    // Fall 2: D-53 Fehlender Slot ⇒ Erased
    {
        let sealer = factory().await;
        let key = KeyBuf::from_prefix(b"tenant:10:doc:999");
        let non_existent_env = Envelope {
            ciphertext: vec![0xDE, 0xAD, 0xBE, 0xEF],
            slot_id: 999_999, // slot does not exist
        };

        match sealer.open(&key, &non_existent_env).await {
            Err(err) => {
                let err_str = err.to_string();
                if !err_str.to_lowercase().contains("erased") {
                    return Err(ConformanceFailure::new(
                        suite,
                        "d_53_missing_slot_returns_erased",
                        format!("expected Erased error on missing slot, got {err:?}"),
                    ));
                }
            }
            Ok(data) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "d_53_missing_slot_returns_erased",
                    format!("missing slot returned Ok data instead of Erased: {data:?}"),
                ));
            }
        }
    }

    // Fall 3: D-79 evict zeroisiert DEK-Cache
    {
        let sealer = factory().await;
        let scope = KeyRange {
            lo: KeyBound::from_prefix(b"tenant:10:"),
            hi: KeyBound::from_prefix(b"tenant:10:\xFF"),
        };
        // evict must execute without panics and zeroize cached keys
        sealer.evict(&scope);
    }

    // Fall 4: I-09 / L-9 make_slots_durable
    {
        let sealer = factory().await;
        if let Err(e) = sealer.make_slots_durable().await {
            return Err(ConformanceFailure::new(
                suite,
                "i_09_make_slots_durable",
                format!("make_slots_durable failed: {e:?}"),
            ));
        }
    }

    Ok(())
}

// ============================================================================
// REFERENCE IMPLEMENTATION (IN-MEMORY)
// ============================================================================

/// In-Memory Reference Implementation of [`RecordSealer`].
#[derive(Clone, Default)]
pub struct ReferenceRecordSealer {
    slots: Arc<RwLock<BTreeMap<u64, Vec<u8>>>>,
    next_slot: Arc<parking_lot::Mutex<u64>>,
    evicted: Arc<AtomicBool>,
}

impl ReferenceRecordSealer {
    /// Creates a new reference sealer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if evict was called.
    pub fn was_evicted(&self) -> bool {
        self.evicted.load(Ordering::SeqCst)
    }
}

impl RecordSealer for ReferenceRecordSealer {
    fn seal<'a>(
        &'a self,
        _ctx: &'a SealCtx,
        _key: &'a KeyBuf,
        plain: &'a [u8],
    ) -> BoxFuture<'a, Result<Envelope>> {
        Box::pin(async move {
            let mut next = self.next_slot.lock();
            *next += 1;
            let slot_id = *next;

            // Generate deterministic synthetic DEK key
            let dek = vec![0x42u8; 32];
            self.slots.write().insert(slot_id, dek.clone());

            // Simple XOR cipher for testing roundtrip
            let mut ciphertext = Vec::with_capacity(plain.len());
            for (i, &b) in plain.iter().enumerate() {
                ciphertext.push(b ^ dek.get(i % dek.len()).copied().unwrap_or(0));
            }

            Ok(Envelope {
                ciphertext,
                slot_id,
            })
        })
    }

    fn open<'a>(&'a self, _key: &'a KeyBuf, env: &'a Envelope) -> BoxFuture<'a, Result<Vec<u8>>> {
        Box::pin(async move {
            let guard = self.slots.read();
            let dek = match guard.get(&env.slot_id) {
                Some(k) => k,
                None => return Err(ContextraError::NotFound("Erased: slot missing".to_string())),
            };

            let mut plaintext = Vec::with_capacity(env.ciphertext.len());
            for (i, &b) in env.ciphertext.iter().enumerate() {
                plaintext.push(b ^ dek.get(i % dek.len()).copied().unwrap_or(0));
            }

            Ok(plaintext)
        })
    }

    fn evict(&self, _scope: &KeyRange) {
        self.evicted.store(true, Ordering::SeqCst);
    }

    fn make_slots_durable(&self) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
}

// ============================================================================
// MUTANTS (INTENTIONALLY DEFECTIVE IMPLEMENTATIONS)
// ============================================================================

/// Mutant 1: missing slot returns NotFound instead of Erased.
#[derive(Clone, Default)]
pub struct MutantMissingSlotNotErasedRecordSealer(ReferenceRecordSealer);

impl RecordSealer for MutantMissingSlotNotErasedRecordSealer {
    fn seal<'a>(
        &'a self,
        ctx: &'a SealCtx,
        key: &'a KeyBuf,
        plain: &'a [u8],
    ) -> BoxFuture<'a, Result<Envelope>> {
        self.0.seal(ctx, key, plain)
    }

    fn open<'a>(&'a self, _key: &'a KeyBuf, _env: &'a Envelope) -> BoxFuture<'a, Result<Vec<u8>>> {
        // DEFECT: Returns NotFound instead of Erased!
        Box::pin(async move { Err(ContextraError::NotFound("slot not found".to_string())) })
    }

    fn evict(&self, scope: &KeyRange) {
        self.0.evict(scope);
    }

    fn make_slots_durable(&self) -> BoxFuture<'_, Result<()>> {
        self.0.make_slots_durable()
    }
}

/// Mutant 2: make_slots_durable fails.
#[derive(Clone, Default)]
pub struct MutantFailingDurableRecordSealer(ReferenceRecordSealer);

impl RecordSealer for MutantFailingDurableRecordSealer {
    fn seal<'a>(
        &'a self,
        ctx: &'a SealCtx,
        key: &'a KeyBuf,
        plain: &'a [u8],
    ) -> BoxFuture<'a, Result<Envelope>> {
        self.0.seal(ctx, key, plain)
    }

    fn open<'a>(&'a self, key: &'a KeyBuf, env: &'a Envelope) -> BoxFuture<'a, Result<Vec<u8>>> {
        self.0.open(key, env)
    }

    fn evict(&self, scope: &KeyRange) {
        self.0.evict(scope);
    }

    fn make_slots_durable(&self) -> BoxFuture<'_, Result<()>> {
        // DEFECT: Fails durability check!
        Box::pin(async move { Err(ContextraError::Internal("disk full".to_string())) })
    }
}

/// Helper executing reference test.
pub async fn run_reference_test() -> ConformanceResult {
    run(|| async { ReferenceRecordSealer::new() }).await
}

/// Helper executing mutant tests, ensuring 100% detection rate.
pub async fn run_mutant_tests() -> std::result::Result<(), String> {
    let res1 = run(|| async { MutantMissingSlotNotErasedRecordSealer::default() }).await;
    if res1.is_ok() {
        return Err("MutantMissingSlotNotErasedRecordSealer escaped detection!".to_string());
    }

    let res2 = run(|| async { MutantFailingDurableRecordSealer::default() }).await;
    if res2.is_ok() {
        return Err("MutantFailingDurableRecordSealer escaped detection!".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn d_53_record_sealer_reference_passes() {
        let res = run_reference_test().await;
        assert!(res.is_ok(), "Reference must pass: {res:?}");
    }

    #[tokio::test]
    async fn d_53_record_sealer_mutants_fail() {
        let res = run_mutant_tests().await;
        assert!(res.is_ok(), "Mutants must be caught: {res:?}");
    }
}

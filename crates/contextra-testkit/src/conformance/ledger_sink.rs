//! # Konformitätsprüfsuite für den `LedgerSink`-Port (`conformance/ledger_sink`)
//!
//! Dieses Modul testet die vertraglichen Zusicherungen (Contract Invariants)
//! für die Lösch-Audit-Senke [`LedgerSink`] gemäß SPEC §4.2, §11.8 und F.9.
//!
//! ## Getestete Verträge (Pflichtfälle):
//! 1. `po_04_ledger_append_read_all_prune`: append, read_all und prune nach Timestamp.
//! 2. `po_04_chain_hash_tamper_evident`: Kettenhash-Integrität ist manipulationssicher (F.9).

use contextra_ports::{BoxFuture, ContextraError, Result};
use parking_lot::RwLock;
use std::future::Future;
use std::sync::Arc;

/// Audit log record written to the ledger outside the database directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerEntry {
    /// Cryptographic commitment of the erase target.
    pub commitment: [u8; 32],
    /// Sequence boundary at which the gate took effect.
    pub gated_seq: u64,
    /// Unix timestamp in nanoseconds when deletion completed.
    pub completed_at: u64,
    /// Previous record chain hash (BLAKE3).
    pub prev_chain_hash: [u8; 32],
    /// Current record chain hash: BLAKE3(prev_chain_hash || commitment || gated_seq || completed_at).
    pub chain_hash: [u8; 32],
}

impl LedgerEntry {
    /// Computes the cryptographic chain hash for this entry.
    #[must_use]
    pub fn compute_hash(
        prev_hash: &[u8; 32],
        commitment: &[u8; 32],
        gated_seq: u64,
        completed_at: u64,
    ) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(prev_hash);
        hasher.update(commitment);
        hasher.update(&gated_seq.to_le_bytes());
        hasher.update(&completed_at.to_le_bytes());
        *hasher.finalize().as_bytes()
    }
}

/// LedgerSink Port Trait according to SPEC §4.2 & §11.8.
pub trait LedgerSink: Send + Sync {
    /// Appends an audit entry, advancing the chain hash.
    fn append(&self, entry: &LedgerEntry) -> BoxFuture<'_, Result<()>>;
    /// Reads all committed ledger entries in sequential order.
    fn read_all(&self) -> BoxFuture<'_, Result<Vec<LedgerEntry>>>;
    /// Prunes entries older than the specified timestamp cutoff.
    fn prune(&self, older_than_timestamp: u64) -> BoxFuture<'_, Result<usize>>;
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

/// Führt die vollständige Konformitätsprüfung für eine [`LedgerSink`]-Implementierung durch.
#[allow(clippy::type_complexity, clippy::cast_possible_truncation)]
pub async fn run<F, Fut, L>(factory: F) -> ConformanceResult
where
    F: Fn() -> Fut,
    Fut: Future<Output = L>,
    L: LedgerSink,
{
    let suite = "LedgerSink";

    // Fall 1: PO-04 / SPEC §11.8 append, read_all & prune
    {
        let sink = factory().await;
        let mut prev = [0u8; 32];

        for i in 1..=5 {
            let comm = [i as u8; 32];
            let gated_seq = 100 + i;
            let completed_at = 1_000_000_000 + i * 1000;
            let chain_hash = LedgerEntry::compute_hash(&prev, &comm, gated_seq, completed_at);

            let entry = LedgerEntry {
                commitment: comm,
                gated_seq,
                completed_at,
                prev_chain_hash: prev,
                chain_hash,
            };

            if let Err(e) = sink.append(&entry).await {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_04_ledger_append_read_all_prune",
                    format!("append failed at item {i}: {e:?}"),
                ));
            }
            prev = chain_hash;
        }

        let entries = match sink.read_all().await {
            Ok(e) => e,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_04_ledger_append_read_all_prune",
                    format!("read_all failed: {e:?}"),
                ));
            }
        };

        if entries.len() != 5 {
            return Err(ConformanceFailure::new(
                suite,
                "po_04_ledger_append_read_all_prune",
                format!("expected 5 entries, got {}", entries.len()),
            ));
        }

        // Prune older than timestamp corresponding to entry 3
        let cutoff = 1_000_000_000 + 3 * 1000;
        let pruned_count = match sink.prune(cutoff).await {
            Ok(c) => c,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_04_ledger_append_read_all_prune",
                    format!("prune failed: {e:?}"),
                ));
            }
        };

        if pruned_count != 2 {
            return Err(ConformanceFailure::new(
                suite,
                "po_04_ledger_append_read_all_prune",
                format!("expected 2 pruned items, got {pruned_count}"),
            ));
        }

        let remaining = match sink.read_all().await {
            Ok(e) => e,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_04_ledger_append_read_all_prune",
                    format!("read_all after prune failed: {e:?}"),
                ));
            }
        };

        if remaining.len() != 3 {
            return Err(ConformanceFailure::new(
                suite,
                "po_04_ledger_append_read_all_prune",
                format!(
                    "expected 3 remaining entries after prune, got {}",
                    remaining.len()
                ),
            ));
        }
    }

    // Fall 2: PO-04 / F.9 Kettenhash manipulationssicher
    {
        let sink = factory().await;
        let prev = [0u8; 32];
        let comm = [0xAAu8; 32];
        let gated_seq = 500;
        let completed_at = 2_000_000_000;
        let correct_hash = LedgerEntry::compute_hash(&prev, &comm, gated_seq, completed_at);

        let mut corrupted_hash = correct_hash;
        corrupted_hash[0] ^= 0xFF; // corrupt chain hash

        let corrupted_entry = LedgerEntry {
            commitment: comm,
            gated_seq,
            completed_at,
            prev_chain_hash: prev,
            chain_hash: corrupted_hash,
        };

        // Sink MUST detect invalid chain hash and reject append
        match sink.append(&corrupted_entry).await {
            Err(err) => {
                let err_str = err.to_string();
                if !err_str.to_lowercase().contains("tamper")
                    && !err_str.to_lowercase().contains("corrupt")
                    && !err_str.to_lowercase().contains("mismatch")
                {
                    return Err(ConformanceFailure::new(
                        suite,
                        "po_04_chain_hash_tamper_evident",
                        format!("corrupted chain hash returned non-tamper error: {err:?}"),
                    ));
                }
            }
            Ok(()) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_04_chain_hash_tamper_evident",
                    "corrupted chain hash was accepted without error",
                ));
            }
        }
    }

    Ok(())
}

// ============================================================================
// REFERENCE IMPLEMENTATION (IN-MEMORY)
// ============================================================================

/// In-Memory Reference Implementation of [`LedgerSink`].
#[derive(Clone, Default)]
#[allow(clippy::type_complexity)]
pub struct ReferenceLedgerSink {
    entries: Arc<RwLock<Vec<LedgerEntry>>>,
}

impl ReferenceLedgerSink {
    /// Creates a new reference ledger sink.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl LedgerSink for ReferenceLedgerSink {
    fn append(&self, entry: &LedgerEntry) -> BoxFuture<'_, Result<()>> {
        let entry_clone = entry.clone();
        Box::pin(async move {
            let mut guard = self.entries.write();
            let expected_prev = guard.last().map(|e| e.chain_hash).unwrap_or([0u8; 32]);
            if entry_clone.prev_chain_hash != expected_prev {
                return Err(ContextraError::PolicyViolation(
                    "tamper: chain hash continuity mismatch".to_string(),
                ));
            }

            let computed = LedgerEntry::compute_hash(
                &entry_clone.prev_chain_hash,
                &entry_clone.commitment,
                entry_clone.gated_seq,
                entry_clone.completed_at,
            );

            if computed != entry_clone.chain_hash {
                return Err(ContextraError::PolicyViolation(
                    "tamper detected: entry chain_hash does not match payload digest".to_string(),
                ));
            }

            guard.push(entry_clone);
            Ok(())
        })
    }

    fn read_all(&self) -> BoxFuture<'_, Result<Vec<LedgerEntry>>> {
        Box::pin(async move {
            let guard = self.entries.read();
            Ok(guard.clone())
        })
    }

    fn prune(&self, older_than_timestamp: u64) -> BoxFuture<'_, Result<usize>> {
        Box::pin(async move {
            let mut guard = self.entries.write();
            let initial_len = guard.len();
            guard.retain(|e| e.completed_at >= older_than_timestamp);
            let pruned = initial_len - guard.len();
            Ok(pruned)
        })
    }
}

// ============================================================================
// MUTANTS (INTENTIONALLY DEFECTIVE IMPLEMENTATIONS)
// ============================================================================

/// Mutant 1: accepts corrupted chain hashes without validation.
#[derive(Clone, Default)]
pub struct MutantTamperBlindLedgerSink(ReferenceLedgerSink);

impl LedgerSink for MutantTamperBlindLedgerSink {
    fn append(&self, entry: &LedgerEntry) -> BoxFuture<'_, Result<()>> {
        let entry_clone = entry.clone();
        // DEFECT: Blindly appends without verifying hash validity!
        Box::pin(async move {
            self.0.entries.write().push(entry_clone);
            Ok(())
        })
    }
    fn read_all(&self) -> BoxFuture<'_, Result<Vec<LedgerEntry>>> {
        self.0.read_all()
    }
    fn prune(&self, older_than_timestamp: u64) -> BoxFuture<'_, Result<usize>> {
        self.0.prune(older_than_timestamp)
    }
}

/// Mutant 2: prune is a no-op.
#[derive(Clone, Default)]
pub struct MutantNoopPruneLedgerSink(ReferenceLedgerSink);

impl LedgerSink for MutantNoopPruneLedgerSink {
    fn append(&self, entry: &LedgerEntry) -> BoxFuture<'_, Result<()>> {
        self.0.append(entry)
    }
    fn read_all(&self) -> BoxFuture<'_, Result<Vec<LedgerEntry>>> {
        self.0.read_all()
    }
    fn prune(&self, _older_than_timestamp: u64) -> BoxFuture<'_, Result<usize>> {
        // DEFECT: Never prunes any entries!
        Box::pin(async move { Ok(0) })
    }
}

/// Helper executing reference test.
pub async fn run_reference_test() -> ConformanceResult {
    run(|| async { ReferenceLedgerSink::new() }).await
}

/// Helper executing mutant tests, ensuring 100% detection rate.
pub async fn run_mutant_tests() -> std::result::Result<(), String> {
    let res1 = run(|| async { MutantTamperBlindLedgerSink::default() }).await;
    if res1.is_ok() {
        return Err("MutantTamperBlindLedgerSink escaped detection!".to_string());
    }

    let res2 = run(|| async { MutantNoopPruneLedgerSink::default() }).await;
    if res2.is_ok() {
        return Err("MutantNoopPruneLedgerSink escaped detection!".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SPEC: PO-04
    #[tokio::test]
    async fn po_04_ledger_sink_reference_passes() {
        let res = run_reference_test().await;
        assert!(res.is_ok(), "Reference must pass: {res:?}");
    }

    /// SPEC: PO-04
    #[tokio::test]
    async fn po_04_ledger_sink_mutants_fail() {
        let res = run_mutant_tests().await;
        assert!(res.is_ok(), "Mutants must be caught: {res:?}");
    }
}

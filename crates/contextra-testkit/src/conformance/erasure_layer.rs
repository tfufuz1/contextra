//! # Konformitätsprüfsuite für den `ErasureLayer`-Port (`conformance/erasure_layer`)
//!
//! Dieses Modul testet die vertraglichen Zusicherungen (Contract Invariants)
//! für Subsysteme, die den [`ErasureLayer`]-Port implementieren (z. B. Store, TextIndex, VectorIndex),
//! gemäß SPEC §8.2, §11, P1, P12, D-69, PO-05 und HR-10.
//!
//! ## Getestete Verträge:
//! 1. `resolve` ist idempotent (mehrfache Ausführung mit identischem Ziel liefert gleichen Scope).
//! 2. `on_gate` ist wiederholbar (Vorbereitungsmaßnahmen führen zu keinem Fehler bei Wiederholung).
//! 3. `purge` ist idempotent und meldet echte Zähler (erster Lauf liefert Zähler > 0, zweiter Lauf 0).
//! 4. `witness` liefert `Err`, solange Reste existieren, und `Ok` erst nach dem Purge (PO-05, P1).
//! 5. `key_coverage` ist konstant über mehrere Aufrufe (D-69).
//! 6. Nicht-Ziel-Daten bleiben während des gesamten Löschvorgangs unverändert erhalten.

use contextra_ports::erasure::{
    Armed, EraseTarget, ErasureLayer, KeyCoverage, LayerId, LayerScope, LayerWitness, PurgeCx,
    PurgeOutcome, Scanner,
};
use contextra_ports::BoxFuture;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Conformance failure descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceFailure {
    /// Suite identifier.
    pub suite_name: &'static str,
    /// Test case name.
    pub test_case: &'static str,
    /// Detailed failure message.
    pub message: String,
}

impl ConformanceFailure {
    /// Creates a new failure description.
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

/// Führt die vollständige Konformitätsprüfung für eine [`ErasureLayer`]-Implementierung über eine Factory aus.
pub async fn run<F, L>(factory: F) -> ConformanceResult
where
    F: Fn() -> L,
    L: ErasureLayer,
{
    let layer = factory();
    run_on_layer(&layer).await
}

/// Führt die vollständige Konformitätsprüfung auf einer Referenz einer [`ErasureLayer`]-Implementierung aus.
pub async fn run_on_layer<L: ErasureLayer + ?Sized>(layer: &L) -> ConformanceResult {
    let suite = "ErasureLayer";

    // 1. Invariante: key_coverage ist konstant (D-69)
    let cov1 = layer.key_coverage();
    let cov2 = layer.key_coverage();
    if cov1 != cov2 {
        return Err(ConformanceFailure::new(
            suite,
            "key_coverage_is_constant",
            format!("key_coverage changed across calls: {cov1:?} != {cov2:?}"),
        ));
    }

    let target = EraseTarget::ExactKeys(vec![b"target_doc_1".to_vec(), b"target_doc_2".to_vec()]);

    // 2. Invariante: resolve ist idempotent
    let scope1 = match layer.resolve(&target).await {
        Ok(s) => s,
        Err(e) => {
            return Err(ConformanceFailure::new(
                suite,
                "resolve_idempotent",
                format!("Initial resolve failed: {e}"),
            ));
        }
    };
    let scope2 = match layer.resolve(&target).await {
        Ok(s) => s,
        Err(e) => {
            return Err(ConformanceFailure::new(
                suite,
                "resolve_idempotent",
                format!("Second resolve call failed: {e}"),
            ));
        }
    };

    if scope1.scope_id() != scope2.scope_id() || scope1.affected_units() != scope2.affected_units()
    {
        return Err(ConformanceFailure::new(
            suite,
            "resolve_idempotent",
            "resolve produced differing scopes for identical EraseTarget",
        ));
    }

    if scope1.affected_units().is_empty() {
        return Err(ConformanceFailure::new(
            suite,
            "resolve_non_empty_scope",
            "resolve produced empty affected_units for existing targets",
        ));
    }

    // 3. Invariante: on_gate ist wiederholbar
    if let Err(e) = layer.on_gate(&scope1).await {
        return Err(ConformanceFailure::new(
            suite,
            "on_gate_repeatable",
            format!("First on_gate call failed: {e}"),
        ));
    }
    if let Err(e) = layer.on_gate(&scope1).await {
        return Err(ConformanceFailure::new(
            suite,
            "on_gate_repeatable",
            format!("Second on_gate call failed (not repeatable): {e}"),
        ));
    }

    // 4. Invariante: witness liefert Err, solange Reste existieren (P1, PO-05)
    let scanner = Scanner::<Armed>::new();
    let premature_witness = layer.witness(&scope1, &scanner).await;
    if premature_witness.is_ok() {
        return Err(ConformanceFailure::new(
            suite,
            "witness_fails_before_purge",
            "witness returned Ok before purge was executed (residue existed, violating P1/PO-05)",
        ));
    }

    // 5. Invariante: purge ist idempotent und meldet echte Zähler
    let cx = PurgeCx::new(10_000);
    let outcome1 = match layer.purge(&scope1, &cx).await {
        Ok(o) => o,
        Err(e) => {
            return Err(ConformanceFailure::new(
                suite,
                "purge_idempotent",
                format!("Initial purge failed: {e}"),
            ));
        }
    };

    if outcome1.purged_count == 0 {
        return Err(ConformanceFailure::new(
            suite,
            "purge_real_counts",
            "purge reported purged_count == 0 for a non-empty target (stub behavior, violating HR-10)",
        ));
    }

    // Zweiter Aufruf muss idempotent gelingen und darf keine weiteren Daten fälschlich doppelt löschen
    let outcome2 = match layer.purge(&scope1, &cx).await {
        Ok(o) => o,
        Err(e) => {
            return Err(ConformanceFailure::new(
                suite,
                "purge_idempotent",
                format!("Second purge call failed (not idempotent): {e}"),
            ));
        }
    };

    if outcome2.purged_count != 0 {
        return Err(ConformanceFailure::new(
            suite,
            "purge_idempotent",
            format!(
                "Second purge call returned non-zero purged_count ({}), not idempotent",
                outcome2.purged_count
            ),
        ));
    }

    // 6. Invariante: witness liefert Ok erst nach dem Purge
    let post_purge_witness = match layer.witness(&scope1, &scanner).await {
        Ok(w) => w,
        Err(e) => {
            return Err(ConformanceFailure::new(
                suite,
                "witness_succeeds_after_purge",
                format!("witness failed after purge completed: {e}"),
            ));
        }
    };

    if post_purge_witness.witness_hash() == &[0u8; 32] {
        return Err(ConformanceFailure::new(
            suite,
            "witness_valid_digest",
            "witness returned all-zero hash after successful purge",
        ));
    }

    Ok(())
}

// ============================================================================
// REFERENCE IMPLEMENTATION (IN-MEMORY)
// ============================================================================

/// In-Memory Reference Implementation of [`ErasureLayer`].
#[derive(Clone)]
pub struct ReferenceErasureLayer {
    id: LayerId,
    coverage: KeyCoverage,
    /// Store items: doc_id -> (unit_id, payload)
    data: Arc<RwLock<BTreeMap<Vec<u8>, (u64, Vec<u8>)>>>,
    /// Units: unit_id -> count of active items
    units: Arc<RwLock<BTreeMap<u64, usize>>>,
    gated_scopes: Arc<RwLock<BTreeMap<[u8; 16], bool>>>,
}

impl Default for ReferenceErasureLayer {
    fn default() -> Self {
        Self::new(LayerId::new([0x01; 16]), KeyCoverage::Full)
    }
}

impl ReferenceErasureLayer {
    /// Creates a pre-populated reference erasure layer satisfying conformance contracts.
    #[must_use]
    pub fn new(id: LayerId, coverage: KeyCoverage) -> Self {
        let layer = Self {
            id,
            coverage,
            data: Arc::new(RwLock::new(BTreeMap::new())),
            units: Arc::new(RwLock::new(BTreeMap::new())),
            gated_scopes: Arc::new(RwLock::new(BTreeMap::new())),
        };

        // Populate with target and non-target items
        layer.insert(b"target_doc_1", 101, b"sensitive payload 1");
        layer.insert(b"target_doc_2", 102, b"sensitive payload 2");
        layer.insert(b"other_doc_survives", 201, b"innocuous non-target payload");

        layer
    }

    /// Inserts a test record.
    pub fn insert(&self, key: &[u8], unit_id: u64, payload: &[u8]) {
        self.data
            .write()
            .insert(key.to_vec(), (unit_id, payload.to_vec()));
        *self.units.write().entry(unit_id).or_insert(0) += 1;
    }

    /// Checks whether non-target data survived untouched.
    pub fn contains_key(&self, key: &[u8]) -> bool {
        self.data.read().contains_key(key)
    }
}

impl ErasureLayer for ReferenceErasureLayer {
    fn id(&self) -> LayerId {
        self.id
    }

    fn resolve(&self, t: &EraseTarget) -> BoxFuture<'_, Result<LayerScope, String>> {
        let data_guard = self.data.read();
        let mut affected = Vec::new();
        match t {
            EraseTarget::ExactKeys(keys) => {
                for k in keys {
                    if let Some((unit_id, _)) = data_guard.get(k) {
                        if !affected.contains(unit_id) {
                            affected.push(*unit_id);
                        }
                    }
                }
            }
            EraseTarget::KeyRange { start, end } => {
                for (k, (unit_id, _)) in data_guard.iter() {
                    if k >= start && k < end && !affected.contains(unit_id) {
                        affected.push(*unit_id);
                    }
                }
            }
        }
        affected.sort_unstable();

        // Deterministic scope ID based on affected units
        let mut scope_bytes = [0u8; 16];
        for (i, unit) in affected.iter().enumerate() {
            let b = unit.to_le_bytes();
            let idx = (i * 8) % 16;
            for j in 0..8 {
                if let (Some(dst), Some(src)) = (scope_bytes.get_mut((idx + j) % 16), b.get(j)) {
                    *dst ^= *src;
                }
            }
        }
        scope_bytes[0] |= 0x80; // Non-zero discriminator

        Box::pin(async move { Ok(LayerScope::new(scope_bytes, affected)) })
    }

    fn on_gate(&self, s: &LayerScope) -> BoxFuture<'_, Result<(), String>> {
        self.gated_scopes.write().insert(*s.scope_id(), true);
        Box::pin(async move { Ok(()) })
    }

    fn purge(&self, s: &LayerScope, _cx: &PurgeCx) -> BoxFuture<'_, Result<PurgeOutcome, String>> {
        let affected = s.affected_units().to_vec();
        let mut data_guard = self.data.write();
        let mut units_guard = self.units.write();

        let mut purged_count = 0u64;
        let mut bytes_reclaimed = 0u64;

        let to_remove: Vec<Vec<u8>> = data_guard
            .iter()
            .filter(|(_, (unit_id, _))| affected.contains(unit_id))
            .map(|(k, _)| k.clone())
            .collect();

        for k in to_remove {
            if let Some((unit_id, payload)) = data_guard.remove(&k) {
                purged_count = purged_count.saturating_add(1);
                bytes_reclaimed = bytes_reclaimed.saturating_add(payload.len() as u64);
                if let Some(cnt) = units_guard.get_mut(&unit_id) {
                    *cnt = cnt.saturating_sub(1);
                }
            }
        }

        Box::pin(async move { Ok(PurgeOutcome::new(purged_count, bytes_reclaimed)) })
    }

    fn witness(
        &self,
        s: &LayerScope,
        _sc: &Scanner<Armed>,
    ) -> BoxFuture<'_, Result<LayerWitness, String>> {
        let affected = s.affected_units().to_vec();
        let data_guard = self.data.read();

        // Check if any residue exists for affected units
        for (_k, (unit_id, _)) in data_guard.iter() {
            if affected.contains(unit_id) {
                return Box::pin(async move {
                    Err("Residue detected in affected units (PO-05)".to_string())
                });
            }
        }

        // Compute witness digest
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"ctx/witness/erasure_layer/v1");
        hasher.update(s.scope_id());
        for u in &affected {
            hasher.update(&u.to_le_bytes());
        }
        let hash = *hasher.finalize().as_bytes();

        Box::pin(async move { Ok(LayerWitness::from_hash(hash)) })
    }

    fn key_coverage(&self) -> KeyCoverage {
        self.coverage
    }
}

// ============================================================================
// MUTANTS (INTENTIONALLY DEFECTIVE IMPLEMENTATIONS)
// ============================================================================

/// Mutant 1: Premature witness before purge (violates P1, PO-05).
#[derive(Clone, Default)]
pub struct MutantPrematureWitnessLayer(ReferenceErasureLayer);

impl ErasureLayer for MutantPrematureWitnessLayer {
    fn id(&self) -> LayerId {
        self.0.id()
    }
    fn resolve(&self, t: &EraseTarget) -> BoxFuture<'_, Result<LayerScope, String>> {
        self.0.resolve(t)
    }
    fn on_gate(&self, s: &LayerScope) -> BoxFuture<'_, Result<(), String>> {
        self.0.on_gate(s)
    }
    fn purge(&self, s: &LayerScope, cx: &PurgeCx) -> BoxFuture<'_, Result<PurgeOutcome, String>> {
        self.0.purge(s, cx)
    }
    fn witness(
        &self,
        s: &LayerScope,
        _sc: &Scanner<Armed>,
    ) -> BoxFuture<'_, Result<LayerWitness, String>> {
        // DEFECT: Returns witness even though residue still exists!
        let hash = [0xAA; 32];
        let _ = s;
        Box::pin(async move { Ok(LayerWitness::from_hash(hash)) })
    }
    fn key_coverage(&self) -> KeyCoverage {
        self.0.key_coverage()
    }
}

/// Mutant 2: Stub purge reporting 0 purged count (violates HR-10, P1).
#[derive(Clone, Default)]
pub struct MutantStubPurgeLayer(ReferenceErasureLayer);

impl ErasureLayer for MutantStubPurgeLayer {
    fn id(&self) -> LayerId {
        self.0.id()
    }
    fn resolve(&self, t: &EraseTarget) -> BoxFuture<'_, Result<LayerScope, String>> {
        self.0.resolve(t)
    }
    fn on_gate(&self, s: &LayerScope) -> BoxFuture<'_, Result<(), String>> {
        self.0.on_gate(s)
    }
    fn purge(&self, _s: &LayerScope, _cx: &PurgeCx) -> BoxFuture<'_, Result<PurgeOutcome, String>> {
        // DEFECT: Fake zero counts (stub behavior, HR-10 defect)
        Box::pin(async move { Ok(PurgeOutcome::new(0, 0)) })
    }
    fn witness(
        &self,
        s: &LayerScope,
        sc: &Scanner<Armed>,
    ) -> BoxFuture<'_, Result<LayerWitness, String>> {
        self.0.witness(s, sc)
    }
    fn key_coverage(&self) -> KeyCoverage {
        self.0.key_coverage()
    }
}

/// Mutant 3: Non-idempotent purge reporting new deleted count on repeat (violates PO-05).
#[derive(Clone, Default)]
pub struct MutantNonIdempotentPurgeLayer(ReferenceErasureLayer);

impl ErasureLayer for MutantNonIdempotentPurgeLayer {
    fn id(&self) -> LayerId {
        self.0.id()
    }
    fn resolve(&self, t: &EraseTarget) -> BoxFuture<'_, Result<LayerScope, String>> {
        self.0.resolve(t)
    }
    fn on_gate(&self, s: &LayerScope) -> BoxFuture<'_, Result<(), String>> {
        self.0.on_gate(s)
    }
    fn purge(&self, s: &LayerScope, cx: &PurgeCx) -> BoxFuture<'_, Result<PurgeOutcome, String>> {
        let _ = self.0.purge(s, cx);
        // DEFECT: Always returns purged_count = 10, never stabilizes at 0 on repeat
        Box::pin(async move { Ok(PurgeOutcome::new(10, 100)) })
    }
    fn witness(
        &self,
        s: &LayerScope,
        sc: &Scanner<Armed>,
    ) -> BoxFuture<'_, Result<LayerWitness, String>> {
        self.0.witness(s, sc)
    }
    fn key_coverage(&self) -> KeyCoverage {
        self.0.key_coverage()
    }
}

/// Mutant 4: Inconsistent key coverage.
#[derive(Clone, Default)]
pub struct MutantInconsistentKeyCoverageLayer {
    inner: ReferenceErasureLayer,
    call_count: Arc<parking_lot::Mutex<usize>>,
}

impl ErasureLayer for MutantInconsistentKeyCoverageLayer {
    fn id(&self) -> LayerId {
        self.inner.id()
    }
    fn resolve(&self, t: &EraseTarget) -> BoxFuture<'_, Result<LayerScope, String>> {
        self.inner.resolve(t)
    }
    fn on_gate(&self, s: &LayerScope) -> BoxFuture<'_, Result<(), String>> {
        self.inner.on_gate(s)
    }
    fn purge(&self, s: &LayerScope, cx: &PurgeCx) -> BoxFuture<'_, Result<PurgeOutcome, String>> {
        self.inner.purge(s, cx)
    }
    fn witness(
        &self,
        s: &LayerScope,
        sc: &Scanner<Armed>,
    ) -> BoxFuture<'_, Result<LayerWitness, String>> {
        self.inner.witness(s, sc)
    }
    fn key_coverage(&self) -> KeyCoverage {
        let mut count = self.call_count.lock();
        *count += 1;
        if *count == 1 {
            KeyCoverage::Full
        } else {
            KeyCoverage::None
        }
    }
}

/// Executes reference test.
pub async fn run_reference_test() -> Result<(), ConformanceFailure> {
    let layer = ReferenceErasureLayer::default();
    run_on_layer(&layer).await?;

    // Verify non-target preservation invariant
    if !layer.contains_key(b"other_doc_survives") {
        return Err(ConformanceFailure::new(
            "ErasureLayer",
            "non_target_preservation",
            "non-target data was corrupted or removed during purge",
        ));
    }
    Ok(())
}

/// Executes mutant tests, ensuring 100% detection rate.
pub async fn run_mutant_tests() -> std::result::Result<(), String> {
    let res1 = run(|| MutantPrematureWitnessLayer(ReferenceErasureLayer::default())).await;
    if res1.is_ok() {
        return Err("MutantPrematureWitnessLayer escaped detection!".to_string());
    }

    let res2 = run(|| MutantStubPurgeLayer(ReferenceErasureLayer::default())).await;
    if res2.is_ok() {
        return Err("MutantStubPurgeLayer escaped detection!".to_string());
    }

    let res3 = run(|| MutantNonIdempotentPurgeLayer(ReferenceErasureLayer::default())).await;
    if res3.is_ok() {
        return Err("MutantNonIdempotentPurgeLayer escaped detection!".to_string());
    }

    let res4 = run(|| MutantInconsistentKeyCoverageLayer::default()).await;
    if res4.is_ok() {
        return Err("MutantInconsistentKeyCoverageLayer escaped detection!".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SPEC: PO-05
    #[tokio::test]
    async fn po_05_erasure_layer_reference_passes() {
        let res = run_reference_test().await;
        assert!(res.is_ok(), "Reference must pass: {res:?}");
    }

    /// SPEC: PO-05
    #[tokio::test]
    async fn po_05_erasure_layer_mutants_fail() {
        let res = run_mutant_tests().await;
        assert!(res.is_ok(), "Mutants must be caught: {res:?}");
    }
}

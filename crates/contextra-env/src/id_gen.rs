//! Unique ID generator port trait and atomic sequential implementation.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

// =========================================================================
// SECTION: IdGen Port Trait
// ANCHOR[id=idgen-root,epic=id-gen,seq=1]
// NOTE: Primäre Abstraktion zur Erzeugung prozessweiter oder lokaler eindeutiger IDs.
// LINK crates/contextra-env/SPEC.md#43-idgen
// =========================================================================

/// Port trait for unique ID generation.
pub trait IdGen: Send + Sync + 'static {
    /// Generates and returns the next unique 64-bit unsigned identifier.
    ///
    /// # Invarianten
    /// - `INV-ENV-ID-1`: Keine Duplikate bei aufeinanderfolgenden Aufrufen; threadsichere Monotonie.
    ///
    /// # Fehler
    /// Keine; garantiert panikfrei durch atomare Operationen.
    /// # Crash
    /// Reiner flüchtiger RAM-Zustand; bei Crash wird der Generator neu initialisiert.
    fn next_id(&self) -> u64;
}

// !SECTION: IdGen Port Trait

// =========================================================================
// SECTION: SequentialIdGen Implementation
// ANCHOR[id=idgen-seq,epic=id-gen,seq=2]
// NOTE: Threadsichere atomare Implementierung basierend auf `AtomicU64`.
// =========================================================================

/// A thread-safe ID generator that produces monotonically increasing 64-bit IDs.
///
/// Wraps around safely on integer overflow (`u64::MAX` -> `0`) via atomic standard wrapping.
#[derive(Debug)]
pub struct SequentialIdGen {
    /// Nächste zu vergebende ID im Prozessspeicher.
    next: AtomicU64,
}

impl SequentialIdGen {
    /// Creates a new [`SequentialIdGen`] starting at `start`.
    pub fn new(start: u64) -> Self {
        Self {
            next: AtomicU64::new(start),
        }
    }
}

impl Default for SequentialIdGen {
    /// Standardmäßig beginnt die Sequenz bei ID 1 (ID 0 bleibt oft für Sentinels/Null reserviert).
    fn default() -> Self {
        Self::new(1)
    }
}

impl IdGen for SequentialIdGen {
    /// Liefert die nächste atomare ID mittels sequentiell konsistenter Ordnung (`Ordering::SeqCst`).
    ///
    /// # Didaktischer Hinweis
    /// `fetch_add(1, Ordering::SeqCst)` liefert den Wert *vor* dem Inkrement zurück und
    /// garantiert über alle CPU-Kerne hinweg eine strikte Totalordnung ohne Duplikate.
    fn next_id(&self) -> u64 {
        self.next.fetch_add(1, Ordering::SeqCst)
    }
}

// !SECTION: SequentialIdGen Implementation

// =========================================================================
// SECTION: Blanket Implementation for Arc<T>
// ANCHOR[id=idgen-arc,epic=id-gen,seq=3]
// NOTE: Erlaubt direkte Nutzung von `Arc<dyn IdGen>` ohne zusätzliche Allokationen.
// =========================================================================

impl<T: IdGen + ?Sized> IdGen for Arc<T> {
    #[inline]
    fn next_id(&self) -> u64 {
        (**self).next_id()
    }
}

// !SECTION: Blanket Implementation for Arc<T>

// =========================================================================
// SECTION: Unit Tests
// ANCHOR[id=idgen-tests,epic=id-gen,seq=4]
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Statische Typprüfung auf Object Safety für `&dyn IdGen`.
    fn _assert_dyn_id_gen(_: Option<&dyn IdGen>) {}

    #[test]
    fn test_sequential_id_gen_monotonic() {
        // NOTE: Verifiziert lineare Monotonie im Single-Thread-Modus
        let gen = SequentialIdGen::new(10);
        assert_eq!(gen.next_id(), 10);
        assert_eq!(gen.next_id(), 11);
        assert_eq!(gen.next_id(), 12);
    }

    #[test]
    fn test_sequential_id_gen_concurrent_no_duplicates() {
        // NOTE: Verifiziert INV-ENV-ID-1 (Nebenläufige Eindeutigkeit über 8 Threads à 1000 IDs = 8000 eindeutige IDs)
        let gen = Arc::new(SequentialIdGen::new(1));
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let g = gen.clone();
                std::thread::spawn(move || {
                    let mut ids = Vec::with_capacity(1000);
                    for _ in 0..1000 {
                        ids.push(g.next_id());
                    }
                    ids
                })
            })
            .collect();

        let mut all_ids = BTreeSet::new();
        for handle in handles {
            let ids = handle.join().expect("Thread panicked");
            for id in ids {
                assert!(all_ids.insert(id), "Duplicate ID found: {id}");
            }
        }

        assert_eq!(all_ids.len(), 8000, "All 8000 generated IDs must be unique");
    }

    #[test]
    fn test_arc_dyn_id_gen() {
        // NOTE: Verifiziert Object Safety und Arc-Deref
        let gen: Arc<dyn IdGen> = Arc::new(SequentialIdGen::new(100));
        _assert_dyn_id_gen(Some(gen.as_ref()));
        assert_eq!(gen.next_id(), 100);
        assert_eq!(gen.next_id(), 101);
    }
}

// !SECTION: Unit Tests

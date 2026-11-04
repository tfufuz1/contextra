//! Pre-gate execution hook port trait (SPEC §3.1, §8.2, D-115, D-117, T-REPL-1..3, T-SUP-1..3).
//!
//! # Normative Referenz
//! - **D-115 (replace_erasing):** Vor dem Setzen des Gates für das alte Dokument
//!   committet die Engine das neue Dokument dauerhaft (R2 PreGate).
//! - **D-117 (update_with Purge):** Vor dem Gate committet die Engine die neue Version
//!   (Sequenz `s_n`). Gate matcht `PrefixSeq { prefix, seq_lt = s_n }`.
//! - **I-25:** Bei `replace_erasing` ist nie alter Inhalt gesperrt, während neuer Inhalt nicht dauerhaft ist.

use crate::BoxFuture;
use contextra_types::ids::ErasureId;
use std::fmt;
use std::sync::Arc;

/// Operation context or intent for pre-gate execution (SPEC D-115, D-117).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreGateIntent {
    /// Replacement of a multi-subject document (D-115, TargetKind 6).
    Replace {
        /// Associated erasure identifier.
        erasure_id: ErasureId,
        /// BLAKE3 tag of the new replacement document.
        new_doc_tag: [u8; 16],
    },
    /// Superseding an existing document version with purge of previous versions (D-117, TargetKind 5).
    Supersede {
        /// Associated erasure identifier.
        erasure_id: ErasureId,
        /// Monotonic commit sequence number before the new version commit.
        intent_seq: u64,
    },
}

/// Outcome of pre-gate execution indicating persisted version sequence (SPEC D-115, D-117).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreGateOutcome {
    /// Monotonic commit sequence number assigned to the newly committed document/version.
    pub committed_seq: u64,
}

/// Abstract port for coordinating atomic pre-gate state mutations (SPEC §8.2, D-115, D-117).
///
/// Implemented by the storage and collection engine to execute atomic pre-commit
/// operations before physical gates take effect.
pub trait PreGate: Send + Sync {
    /// Executes the pre-gate action, ensuring the new document or version is persisted.
    ///
    /// # Fehler
    /// Liefert einen Fehler-String bei Commit- oder I/O-Fehlern. Bei Scheitern bleibt
    /// das bestehende Ziel unberührt (I-25, Fail-Closed).
    ///
    /// # Crash
    /// Crash-konsistent; Recovery nimmt unterbrochene Operationen über das Journal wieder auf.
    fn run<'a>(
        &'a self,
        intent: &'a PreGateIntent,
    ) -> BoxFuture<'a, std::result::Result<PreGateOutcome, String>>;
}

impl<T: PreGate + ?Sized> PreGate for Arc<T> {
    fn run<'a>(
        &'a self,
        intent: &'a PreGateIntent,
    ) -> BoxFuture<'a, std::result::Result<PreGateOutcome, String>> {
        (**self).run(intent)
    }
}

impl fmt::Display for PreGateOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PreGateOutcome(committed_seq={})", self.committed_seq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::FutureExt;

    struct DummyPreGate;

    impl PreGate for DummyPreGate {
        fn run<'a>(
            &'a self,
            _intent: &'a PreGateIntent,
        ) -> BoxFuture<'a, std::result::Result<PreGateOutcome, String>> {
            Box::pin(async move { Ok(PreGateOutcome { committed_seq: 100 }) })
        }
    }

    /// SPEC: T-REPL-1
    #[test]
    fn test_dyn_pre_gate() {
        let pre_gate: Arc<dyn PreGate> = Arc::new(DummyPreGate);
        let intent = PreGateIntent::Replace {
            erasure_id: ErasureId::new([1u8; 16]),
            new_doc_tag: [0u8; 16],
        };
        let out = pre_gate.run(&intent).now_or_never().expect("now or never");
        assert_eq!(out.expect("ok").committed_seq, 100);
    }
}

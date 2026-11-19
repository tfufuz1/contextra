//! # Modul: read_gate.rs
//!
//! **Zweck:** Durchsetzung des synchronen ReadGate-Prüfpunkts an allen Lesepfaden
//! der Collection (SPEC §6.2, INV-GATE-COMPLETENESS).
//!
//! **Architektur:** Gehört zur Crate `contextra-db` (E2). Der Gate-Port ist kanonisch
//! in `contextra-erasure` definiert (SPEC §2.5, `ReadGate`/`GateDecision`).
//!
//! **Invarianten:**
//! - O(1), kein I/O, kein `await` innerhalb der Prüfung.
//! - Fail-Closed: jede Entscheidung ungleich `Open` verweigert die Ausgabe.
//! - `Restricted` (Art. 18) wird ohne Audit-Pfad wie `Erased` behandelt; ein Audit-Eintrag
//!   ist in diesem Modul nicht vorgesehen und darf nicht stillschweigend umgangen werden.

use super::Collection;
use contextra_erasure::{GateDecision, ReadGate, SagaGate};
use contextra_ports::{StorageEngine, VectorIndex};
use contextra_types::{ContextraError, Result};
use std::sync::Arc;



/// Standard-Gate: ein frisches [`SagaGate`] ohne registrierte Löschziele (alle Schlüssel `Open`).
pub(crate) fn default_read_gate() -> Arc<dyn ReadGate> {
    Arc::new(SagaGate::new())
}

impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    /// Ersetzt das ReadGate dieser Collection (z. B. durch das gemeinsame Gate der Saga).
    ///
    /// Klone der Collection teilen sich das Gate über den `Arc`.
    #[must_use]
    pub fn with_read_gate(mut self, gate: Arc<dyn ReadGate>) -> Self {
        self.read_gate = gate;
        self
    }

    /// Liefert die Gate-Entscheidung für einen rohen Storage-Schlüssel.
    #[inline]
    pub(crate) fn gate_decision(&self, raw_key: &[u8]) -> GateDecision {
        self.read_gate.check(raw_key)
    }

    /// `true`, wenn der Schlüssel ohne Einschränkung gelesen werden darf.
    #[inline]
    pub(crate) fn is_readable_key(&self, raw_key: &[u8]) -> bool {
        matches!(self.gate_decision(raw_key), GateDecision::Open)
    }

    /// Fail-Closed-Prüfung für Punktzugriffe: liefert einen Fehler statt Daten.
    #[inline]
    pub(crate) fn check_read_key(&self, raw_key: &[u8]) -> Result<()> {
        match self.gate_decision(raw_key) {
            GateDecision::Open => Ok(()),
            GateDecision::Erased => Err(ContextraError::Erased),
            GateDecision::Restricted => Err(ContextraError::RestrictedReadAuditRequired),
        }
    }
}

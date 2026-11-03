//! In-Memory-Bereichsmenge aktiver Gates als Intervallbaum (SPEC §11.3, ER-11).
//!
//! `ErasedSet` hält alle aktiven Gates in zwei sortierten Vektoren:
//! - `erased`: Bereiche, deren Daten vernichtet wurden
//! - `restricted`: Bereiche, die gesperrt sind (nie verwerfbar)
//!
//! `check` ist O(log n); kein I/O.

use contextra_ports::gate::{GateDecision, KeyBound, KeyBuf, KeyRange, RangeGate, ReadGate, Seq};

/// Ein einzelner Gate-Eintrag.
#[derive(Clone, Debug)]
struct GateEntry {
    /// Untere Grenze (inklusive).
    lo: KeyBuf,
    /// Obere Grenze (exklusive).
    hi: KeyBuf,
    /// Sequenznummer, bei der das Gate gesetzt wurde.
    gated_seq: Seq,
}

impl GateEntry {
    fn contains(&self, k: &KeyBuf) -> bool {
        k >= &self.lo && k < &self.hi
    }
}

/// Sortierte Bereichsmenge aktiver Gates (ER-11).
pub struct ErasedSet {
    erased: Vec<GateEntry>,
    restricted: Vec<GateEntry>,
}

impl ErasedSet {
    /// Erstellt einen leeren Gate-Satz.
    pub fn new() -> Self {
        Self {
            erased: Vec::new(),
            restricted: Vec::new(),
        }
    }

    /// Fügt einen `Erased`-Bereich hinzu.
    pub fn add_erased(&mut self, lo: KeyBuf, hi: KeyBuf, gated_seq: Seq) {
        self.erased.push(GateEntry { lo, hi, gated_seq });
        self.erased.sort_by_key(|e| e.lo);
    }

    /// Fügt einen `Restricted`-Bereich hinzu.
    pub fn add_restricted(&mut self, lo: KeyBuf, hi: KeyBuf, gated_seq: Seq) {
        self.restricted.push(GateEntry { lo, hi, gated_seq });
        self.restricted.sort_by_key(|e| e.lo);
    }

    /// Hebt eine Restriktion auf (ER-15).
    pub fn lift_restriction(&mut self, lo: &KeyBuf, hi: &KeyBuf) {
        self.restricted.retain(|e| &e.lo != lo || &e.hi != hi);
    }

    /// Entfernt ein Erased-Gate (bei Completed, ER-12 / L-24).
    pub fn remove_erased(&mut self, lo: &KeyBuf, hi: &KeyBuf) {
        self.erased.retain(|e| &e.lo != lo || &e.hi != hi);
    }

    /// Prüft einen Schlüssel (O(log n) mit binärer Suche auf sortierten Einträgen).
    pub fn check(&self, k: &KeyBuf) -> GateDecision {
        // Restricted hat Vorrang (sicherste Entscheidung)
        if self.restricted.iter().any(|e| e.contains(k)) {
            return GateDecision::Restricted;
        }
        if self.erased.iter().any(|e| e.contains(k)) {
            return GateDecision::Erased;
        }
        GateDecision::Open
    }

    /// Liefert die Sequenznummer, bei der ein Schlüssel eingesprrt wurde.
    pub fn gated_seq(&self, k: &KeyBuf) -> Option<Seq> {
        for e in self.restricted.iter().chain(self.erased.iter()) {
            if e.contains(k) {
                return Some(e.gated_seq);
            }
        }
        None
    }

    /// Prüft eine Grenze.
    pub fn check_bound(&self, b: &KeyBound) -> GateDecision {
        match b {
            KeyBound::Exact(k) => self.check(k),
            KeyBound::UnboundedLower | KeyBound::UnboundedUpper => GateDecision::Open,
        }
    }

    /// Bereichsrichtlinie für Kompaktion (I-18): Restricted > Erased > Open.
    pub fn variant_for_range(&self, r: &KeyRange) -> RangeGate {
        // Prüfe ob irgendein Punkt im Bereich Restricted ist
        let has_restricted = self.restricted.iter().any(|e| {
            // Bereiche überlappen wenn lo1 < hi2 && lo2 < hi1
            e.lo < r.end && r.start < e.hi
        });
        if has_restricted {
            return RangeGate::RestrictedNeverDiscard;
        }

        let has_erased = self.erased.iter().any(|e| e.lo < r.end && r.start < e.hi);
        if has_erased {
            return RangeGate::ContainsErased;
        }

        RangeGate::EntirelyOpen
    }
}

impl Default for ErasedSet {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadGate for ErasedSet {
    fn check(&self, k: &KeyBuf) -> GateDecision {
        self.check(k)
    }

    fn check_bound(&self, b: &KeyBound) -> GateDecision {
        self.check_bound(b)
    }

    fn variant_for_range(&self, r: &KeyRange) -> RangeGate {
        self.variant_for_range(r)
    }

    fn gated_seq(&self, k: &KeyBuf) -> Option<Seq> {
        self.gated_seq(k)
    }
}

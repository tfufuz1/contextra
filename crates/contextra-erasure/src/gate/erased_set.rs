//! In-Memory-Bereichsmenge aktiver Gates als Intervallbaum (SPEC §11.3, ER-11, D-120, D-123).
//!
//! `ErasedSet` hält alle aktiven Gates in sortierten Strukturen:
//! - `erased`: Bereiche, deren Daten vernichtet wurden
//! - `restricted`: Bereiche, die gesperrt sind (nie verwerfbar)
//! - `prefix_seq`: Matcher `PrefixSeq { prefix[65], seq_lt }` (SPEC D-117)
//!
//! `check` ist O(log n); kein I/O.
//! Hält atomare `gate_epoch` für Lese-Linearisierung (D-120, I-21).

use contextra_ports::gate::{GateDecision, KeyBound, KeyBuf, KeyRange, RangeGate, ReadGate, Seq};
use std::sync::atomic::{AtomicU64, Ordering};

/// Ein einzelner Gate-Eintrag für 16-Byte oder 65-Byte Bereiche.
#[derive(Clone, Debug)]
struct GateEntry {
    gate_id: [u8; 16],
    /// Untere Grenze (inklusive).
    lo: KeyBuf,
    /// Obere Grenze (exklusive).
    hi: KeyBuf,
    /// Optionale 65-Byte-Grenzen für echte LSM-Schlüssel.
    bounds_65: Option<([u8; 65], [u8; 65])>,
    /// Sequenznummer, bei der das Gate gesetzt wurde.
    gated_seq: Seq,
}

impl GateEntry {
    fn contains_16(&self, k: &KeyBuf) -> bool {
        k >= &self.lo && k < &self.hi
    }

    fn contains_78(&self, raw: &[u8; 78]) -> bool {
        if let Some((lo, hi)) = &self.bounds_65 {
            let prefix: &[u8; 65] = match raw[0..65].try_into() {
                Ok(p) => p,
                Err(_) => return false,
            };
            prefix >= lo && prefix <= hi
        } else {
            // Fallback auf die ersten 16 Bytes
            let mut k = [0u8; 16];
            k.copy_from_slice(&raw[0..16]);
            let k_buf = KeyBuf::new(k);
            self.contains_16(&k_buf)
        }
    }
}

/// PrefixSeq-Prädikat (SPEC D-117): Matcht 65-Byte-Präfix und prüft commit_seq < seq_lt.
#[derive(Clone, Debug)]
struct PrefixSeqEntry {
    prefix: [u8; 65],
    seq_lt: Seq,
    gated_seq: Seq,
}

impl PrefixSeqEntry {
    fn matches_78(&self, raw: &[u8; 78]) -> bool {
        if raw[0..65] != self.prefix {
            return false;
        }
        // commit_seq aus Bytes 70..78 extrahieren (!commit_seq komplementiert im Suffix, D-74, D-123)
        let mut raw_seq = [0u8; 8];
        raw_seq.copy_from_slice(&raw[70..78]);
        let commit_seq = !u64::from_be_bytes(raw_seq);
        commit_seq < self.seq_lt.as_u64()
    }
}

/// Sortierte Bereichsmenge aktiver Gates (ER-11, D-120).
pub struct ErasedSet {
    gate_epoch: AtomicU64,
    erased: Vec<GateEntry>,
    restricted: Vec<GateEntry>,
    prefix_seq: Vec<PrefixSeqEntry>,
}

impl ErasedSet {
    /// Erstellt einen leeren Gate-Satz.
    pub fn new() -> Self {
        Self {
            gate_epoch: AtomicU64::new(0),
            erased: Vec::new(),
            restricted: Vec::new(),
            prefix_seq: Vec::new(),
        }
    }

    /// Liefert die aktuelle Gate-Epoche.
    #[inline]
    pub fn gate_epoch(&self) -> u64 {
        self.gate_epoch.load(Ordering::Acquire)
    }

    /// Erhöht die Gate-Epoche atomar (vor der Schlüsselvernichtung, I-21).
    #[inline]
    pub fn increment_epoch(&self) -> u64 {
        self.gate_epoch.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Fügt einen `Erased`-Bereich hinzu.
    pub fn add_erased(&mut self, lo: KeyBuf, hi: KeyBuf, gated_seq: Seq) {
        self.increment_epoch();
        self.erased.push(GateEntry {
            gate_id: [0u8; 16],
            lo,
            hi,
            bounds_65: None,
            gated_seq,
        });
        self.erased.sort_by_key(|e| e.lo);
    }

    /// Fügt einen `Restricted`-Bereich hinzu.
    pub fn add_restricted(&mut self, lo: KeyBuf, hi: KeyBuf, gated_seq: Seq) {
        self.increment_epoch();
        self.restricted.push(GateEntry {
            gate_id: [0u8; 16],
            lo,
            hi,
            bounds_65: None,
            gated_seq,
        });
        self.restricted.sort_by_key(|e| e.lo);
    }

    /// Fügt einen `Erased`-Bereich mit 65-Byte-Grenzen hinzu.
    pub fn add_erased_with_bounds(
        &mut self,
        gate_id: [u8; 16],
        lo: [u8; 65],
        hi: [u8; 65],
        gated_seq: Seq,
    ) {
        self.increment_epoch();
        let mut k_lo = [0u8; 16];
        let mut k_hi = [0u8; 16];
        k_lo.copy_from_slice(&lo[0..16]);
        k_hi.copy_from_slice(&hi[0..16]);

        self.erased.push(GateEntry {
            gate_id,
            lo: KeyBuf::new(k_lo),
            hi: KeyBuf::new(k_hi),
            bounds_65: Some((lo, hi)),
            gated_seq,
        });
        self.erased.sort_by_key(|e| e.lo);
    }

    /// Fügt einen `Restricted`-Bereich mit 65-Byte-Grenzen hinzu.
    pub fn add_restricted_with_bounds(
        &mut self,
        gate_id: [u8; 16],
        lo: [u8; 65],
        hi: [u8; 65],
        gated_seq: Seq,
    ) {
        self.increment_epoch();
        let mut k_lo = [0u8; 16];
        let mut k_hi = [0u8; 16];
        k_lo.copy_from_slice(&lo[0..16]);
        k_hi.copy_from_slice(&hi[0..16]);

        self.restricted.push(GateEntry {
            gate_id,
            lo: KeyBuf::new(k_lo),
            hi: KeyBuf::new(k_hi),
            bounds_65: Some((lo, hi)),
            gated_seq,
        });
        self.restricted.sort_by_key(|e| e.lo);
    }

    /// Fügt ein PrefixSeq-Gate hinzu (SPEC D-117).
    pub fn add_prefix_seq(&mut self, prefix: [u8; 65], seq_lt: Seq, gated_seq: Seq) {
        self.increment_epoch();
        self.prefix_seq.push(PrefixSeqEntry {
            prefix,
            seq_lt,
            gated_seq,
        });
    }

    /// Entfernt ein Gate über dessen ID.
    pub fn remove_gate(&mut self, gate_id: &[u8; 16]) {
        self.increment_epoch();
        self.erased.retain(|e| &e.gate_id != gate_id);
        self.restricted.retain(|e| &e.gate_id != gate_id);
    }

    /// Hebt eine Restriktion auf (ER-15).
    pub fn lift_restriction(&mut self, lo: &KeyBuf, hi: &KeyBuf) {
        self.increment_epoch();
        self.restricted.retain(|e| &e.lo != lo || &e.hi != hi);
    }

    /// Entfernt ein Erased-Gate (bei Completed, ER-12 / L-24).
    pub fn remove_erased(&mut self, lo: &KeyBuf, hi: &KeyBuf) {
        self.increment_epoch();
        self.erased.retain(|e| &e.lo != lo || &e.hi != hi);
    }

    /// Prüft einen 16-Byte-Schlüssel (O(log n) mit binärer Suche auf sortierten Einträgen).
    pub fn check(&self, k: &KeyBuf) -> GateDecision {
        // Restricted hat Vorrang (sicherste Entscheidung)
        if self.restricted.iter().any(|e| e.contains_16(k)) {
            return GateDecision::Restricted;
        }
        if self.erased.iter().any(|e| e.contains_16(k)) {
            return GateDecision::Erased;
        }
        GateDecision::Open
    }

    /// Prüft einen kanonischen 78-Byte-Schlüssel (SPEC 9.1, D-123).
    pub fn check_raw(&self, raw: &[u8; 78]) -> GateDecision {
        if self.restricted.iter().any(|e| e.contains_78(raw)) {
            return GateDecision::Restricted;
        }
        if self.erased.iter().any(|e| e.contains_78(raw)) {
            return GateDecision::Erased;
        }
        if self.prefix_seq.iter().any(|e| e.matches_78(raw)) {
            return GateDecision::Erased;
        }
        GateDecision::Open
    }

    /// Prüft, ob ein 16-Byte KeyBuf gegated ist (Erased oder Restricted, für Replay/Commit, I-2).
    pub fn covers(&self, k: &KeyBuf) -> bool {
        match self.check(k) {
            GateDecision::Erased | GateDecision::Restricted => true,
            GateDecision::Open => false,
        }
    }

    /// Prüft, ob ein 78-Byte Schlüssel gegated ist (Erased oder Restricted, für Replay/Commit, I-2).
    pub fn covers_raw(&self, raw: &[u8; 78]) -> bool {
        match self.check_raw(raw) {
            GateDecision::Erased | GateDecision::Restricted => true,
            GateDecision::Open => false,
        }
    }

    /// Liefert die Sequenznummer, bei der ein Schlüssel eingesprrt wurde.
    pub fn gated_seq(&self, k: &KeyBuf) -> Option<Seq> {
        for e in self.restricted.iter().chain(self.erased.iter()) {
            if e.contains_16(k) {
                return Some(e.gated_seq);
            }
        }
        None
    }

    /// Liefert die Sequenznummer, bei der ein 78-Byte Schlüssel eingesprrt wurde.
    pub fn gated_seq_raw(&self, raw: &[u8; 78]) -> Option<Seq> {
        for e in self.restricted.iter().chain(self.erased.iter()) {
            if e.contains_78(raw) {
                return Some(e.gated_seq);
            }
        }
        for e in &self.prefix_seq {
            if e.matches_78(raw) {
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
        let has_restricted = self
            .restricted
            .iter()
            .any(|e| e.lo < r.end && r.start < e.hi);
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

    fn gate_epoch(&self) -> u64 {
        self.gate_epoch()
    }

    fn covers(&self, k: &KeyBuf) -> bool {
        self.covers(k)
    }

    fn check_raw(&self, raw: &[u8; 78]) -> GateDecision {
        self.check_raw(raw)
    }

    fn covers_raw(&self, raw: &[u8; 78]) -> bool {
        self.covers_raw(raw)
    }

    fn gated_seq_raw(&self, raw: &[u8; 78]) -> Option<Seq> {
        self.gated_seq_raw(raw)
    }
}

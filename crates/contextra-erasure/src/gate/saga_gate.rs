//! Layout-agnostischer Lese-Gate der Löschsaga (WP-E4, E-5, SPEC §5.2, §6.3).
//!
//! Schlüssel sind Text-Schlüssel (z. B. `t:{tenant}:{collection}:chunk:{doc}`); der Gate
//! parst sie nicht. Lookup-Kosten: Der Fast-Path ohne aktive Sperre ist ein einzelner
//! atomarer Lesezugriff ohne Lock. Mit aktiver Sperre nimmt `check` genau ein Read-Lock
//! und liest daraus einen konsistenten Snapshot. Der Präfix-Lookup iteriert über die
//! distinkten registrierten Präfixlängen `L` (Kosten `O(L)`, kein Heap-Allokation).
//! Kein I/O, kein `await`, kein globaler Zustand.
//!
//! Formatvermerk: keine persistierten Bytes; reiner Laufzeitzustand.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use parking_lot::RwLock;

use crate::error::ErasureError;

/// Richtwert für die erwartete Anzahl distinkter Präfixlängen (Schlüsselschema mit festen
/// Segmentlängen). Dient als Diagnosegröße; die Korrektheit hängt nicht davon ab.
pub const MAX_DISTINCT_PREFIX_LENGTHS: usize = 16;

/// Entscheidung des Lese-Gates für einen Schlüssel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateDecision {
    /// Schlüssel ist nicht betroffen.
    Open,
    /// Schlüssel ist gelöscht; Lesen liefert nichts.
    Erased,
    /// Eingeschränkt (Art. 18): Lesen nur mit Audit, nie verwerfen.
    Restricted,
}

/// Entscheidung des Gates für einen Schlüsselbereich `[start, end)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeGate {
    /// Bereich enthält keine betroffenen Schlüssel.
    AllOpen,
    /// Bereich enthält gelöschte Schlüssel (Kompaktion muss sie verwerfen).
    ContainsErased,
    /// Bereich enthält eingeschränkte Schlüssel (dürfen nie verworfen werden).
    RestrictedNeverDiscard,
}

/// Lese-Gate-Trait (SPEC §6.3, Schnittstellenvertrag V1).
pub trait ReadGate: Send + Sync {
    /// O(1), kein I/O, kein `await`.
    fn check(&self, key: &[u8]) -> GateDecision;
    /// Bereichsprüfung `[start, end)`; darf iterieren (Kompaktionszeit).
    fn check_range(&self, start: &[u8], end: &[u8]) -> RangeGate;
}

/// Gesamter Sperrzustand. Alle Felder werden unter einem gemeinsamen Lock gelesen bzw.
/// geschrieben, damit `check` einen konsistenten Snapshot sieht.
#[derive(Default)]
struct Shard {
    erased_exact: HashSet<Vec<u8>>,
    restricted_exact: HashSet<Vec<u8>>,
    erased_prefix: HashSet<Vec<u8>>,
    restricted_prefix: HashSet<Vec<u8>>,
    /// Anzahl der in den Präfixmengen (beide Kategorien) enthaltenen Präfixe je Länge.
    /// Ein Eintrag existiert nur, solange mindestens ein Präfix dieser Länge gesperrt ist.
    prefix_lengths: HashMap<usize, usize>,
}

impl Shard {
    fn prefix_set(&self, restricted: bool) -> &HashSet<Vec<u8>> {
        if restricted {
            &self.restricted_prefix
        } else {
            &self.erased_prefix
        }
    }

    fn prefix_set_mut(&mut self, restricted: bool) -> &mut HashSet<Vec<u8>> {
        if restricted {
            &mut self.restricted_prefix
        } else {
            &mut self.erased_prefix
        }
    }

    /// Fügt ein Präfix hinzu. Die Längen-Zählung wird nur bei echtem Neueintrag erhöht.
    fn insert_prefix(&mut self, prefix: &[u8], restricted: bool) {
        if self.prefix_set_mut(restricted).insert(prefix.to_vec()) {
            *self.prefix_lengths.entry(prefix.len()).or_insert(0) += 1;
        }
    }

    /// Entfernt ein Präfix. Gibt `true` zurück, wenn es vorhanden war.
    fn remove_prefix(&mut self, prefix: &[u8], restricted: bool) -> bool {
        if !self.prefix_set_mut(restricted).remove(prefix) {
            return false;
        }
        if let Some(count) = self.prefix_lengths.get_mut(&prefix.len()) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.prefix_lengths.remove(&prefix.len());
            }
        }
        true
    }

    /// Prüft, ob `key` mit einem gesperrten Präfix der jeweiligen Kategorie beginnt.
    fn prefix_hit(&self, key: &[u8], restricted: bool) -> bool {
        let set = self.prefix_set(restricted);
        self.prefix_lengths
            .keys()
            .any(|&len| key.get(..len).is_some_and(|cand| set.contains(cand)))
    }
}

/// Produktiver Löschgate der Saga.
pub struct SagaGate {
    /// Fast-Path: `false` solange nichts gesperrt ist; dann entfällt jeder Lock.
    active: AtomicBool,
    registrations: AtomicUsize,
    inner: RwLock<Shard>,
}

impl Default for SagaGate {
    fn default() -> Self {
        Self::new()
    }
}

fn is_delimiter_terminated(prefix: &[u8]) -> bool {
    prefix.last() == Some(&b':')
}

/// Obere Schranke eines Präfixes (lexikographisch): Präfix mit letztem Byte +1 (Carry).
fn prefix_upper(prefix: &[u8]) -> Option<Vec<u8>> {
    let mut upper = prefix.to_vec();
    while let Some(last) = upper.last_mut() {
        if *last < 0xFF {
            *last += 1;
            return Some(upper);
        }
        upper.pop();
    }
    None
}

impl SagaGate {
    /// Leerer Gate ohne Registrierungen.
    pub fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            registrations: AtomicUsize::new(0),
            inner: RwLock::new(Shard::default()),
        }
    }

    fn touch_active(&self) {
        self.registrations.fetch_add(1, Ordering::SeqCst);
        self.active.store(true, Ordering::SeqCst);
    }

    /// Aktiviert einen gelöschten Präfix (delimiter-terminiert, z. B. `t:3:7:`).
    pub fn activate_prefix(&self, prefix: &[u8]) -> Result<(), ErasureError> {
        self.register_prefix(prefix, false)
    }

    /// Aktiviert einen eingeschränkten Präfix (Art. 18).
    pub fn activate_restricted_prefix(&self, prefix: &[u8]) -> Result<(), ErasureError> {
        self.register_prefix(prefix, true)
    }

    fn register_prefix(&self, prefix: &[u8], restricted: bool) -> Result<(), ErasureError> {
        if !is_delimiter_terminated(prefix) {
            return Err(ErasureError::Corrupt);
        }
        self.inner.write().insert_prefix(prefix, restricted);
        self.touch_active();
        Ok(())
    }

    /// Aktiviert einen gelöschten exakten Schlüssel.
    pub fn activate_key(&self, key: &[u8]) -> Result<(), ErasureError> {
        self.inner.write().erased_exact.insert(key.to_vec());
        self.touch_active();
        Ok(())
    }

    /// Aktiviert einen eingeschränkten exakten Schlüssel.
    pub fn activate_restricted_key(&self, key: &[u8]) -> Result<(), ErasureError> {
        self.inner.write().restricted_exact.insert(key.to_vec());
        self.touch_active();
        Ok(())
    }

    /// Hebt eine Einschränkung auf (nur `Restricted→Lifted`, SPEC §5.2).
    pub fn lift_restricted_key(&self, key: &[u8]) {
        self.inner.write().restricted_exact.remove(key);
    }

    /// Hebt eine eingeschränkte Präfix-Registrierung auf (nur `Restricted→Lifted`).
    pub fn lift_restricted_prefix(&self, prefix: &[u8]) {
        self.inner.write().remove_prefix(prefix, true);
    }

    /// Anzahl aktiver Registrierungen (Diagnose; kein Klartext).
    pub fn registration_count(&self) -> usize {
        self.registrations.load(Ordering::SeqCst)
    }
}

impl ReadGate for SagaGate {
    fn check(&self, key: &[u8]) -> GateDecision {
        if !self.active.load(Ordering::SeqCst) {
            return GateDecision::Open;
        }
        // Ein einziges Read-Lock: alle vier Prüfungen sehen denselben Zustand.
        let sh = self.inner.read();
        if sh.restricted_exact.contains(key) || sh.prefix_hit(key, true) {
            return GateDecision::Restricted;
        }
        if sh.erased_exact.contains(key) || sh.prefix_hit(key, false) {
            return GateDecision::Erased;
        }
        GateDecision::Open
    }

    fn check_range(&self, start: &[u8], end: &[u8]) -> RangeGate {
        if !self.active.load(Ordering::SeqCst) {
            return RangeGate::AllOpen;
        }
        let mut erased = false;
        let mut restricted = false;
        {
            let sh = self.inner.read();
            for k in &sh.restricted_exact {
                if k.as_slice() >= start && k.as_slice() < end {
                    restricted = true;
                }
            }
            for k in &sh.erased_exact {
                if k.as_slice() >= start && k.as_slice() < end {
                    erased = true;
                }
            }
            for p in sh.restricted_prefix.iter().chain(sh.erased_prefix.iter()) {
                let overlaps = p.as_slice() < end
                    && prefix_upper(p).is_none_or(|u| u.as_slice() > start);
                if overlaps {
                    if sh.restricted_prefix.contains(p) {
                        restricted = true;
                    } else {
                        erased = true;
                    }
                }
            }
        }
        if restricted {
            RangeGate::RestrictedNeverDiscard
        } else if erased {
            RangeGate::ContainsErased
        } else {
            RangeGate::AllOpen
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leer_ist_offen() {
        let g = SagaGate::new();
        assert_eq!(g.check(b"t:1:2:chunk:3"), GateDecision::Open);
        assert_eq!(g.check_range(b"a", b"z"), RangeGate::AllOpen);
    }

    #[test]
    fn exakter_schluessel_wird_erased() {
        let g = SagaGate::new();
        g.activate_key(b"t:1:2:chunk:3").unwrap();
        assert_eq!(g.check(b"t:1:2:chunk:3"), GateDecision::Erased);
        assert_eq!(g.check(b"t:1:2:chunk:4"), GateDecision::Open);
    }

    #[test]
    fn delimiter_praefix_deckt_nur_eigenen_tenant() {
        let g = SagaGate::new();
        g.activate_prefix(b"t:3:7:").unwrap();
        assert_eq!(g.check(b"t:3:7:chunk:1"), GateDecision::Erased);
        assert_eq!(g.check(b"t:3:70:chunk:1"), GateDecision::Open);
        assert_eq!(g.check(b"t:3:8:chunk:1"), GateDecision::Open);
    }

    #[test]
    fn praefix_ohne_delimiter_wird_abgelehnt() {
        let g = SagaGate::new();
        assert!(g.activate_prefix(b"t:3:7").is_err());
    }

    #[test]
    fn restricted_hat_vorrang() {
        let g = SagaGate::new();
        g.activate_prefix(b"t:3:").unwrap();
        g.activate_restricted_prefix(b"t:3:7:").unwrap();
        assert_eq!(g.check(b"t:3:7:x"), GateDecision::Restricted);
        assert_eq!(g.check(b"t:3:8:x"), GateDecision::Erased);
    }

    #[test]
    fn lift_restricted_gibt_frei() {
        let g = SagaGate::new();
        g.activate_restricted_key(b"t:1:2:k").unwrap();
        assert_eq!(g.check(b"t:1:2:k"), GateDecision::Restricted);
        g.lift_restricted_key(b"t:1:2:k");
        assert_eq!(g.check(b"t:1:2:k"), GateDecision::Open);
    }

    #[test]
    fn lift_restricted_praefix_gibt_frei_und_bereinigt_laengen() {
        let g = SagaGate::new();
        g.activate_restricted_prefix(b"t:3:7:").unwrap();
        assert_eq!(g.check(b"t:3:7:x"), GateDecision::Restricted);
        g.lift_restricted_prefix(b"t:3:7:");
        assert_eq!(g.check(b"t:3:7:x"), GateDecision::Open);
        // Nach dem Lift darf die Präfixlänge nicht mehr im Lookup auftauchen.
        assert!(g.inner.read().prefix_lengths.is_empty());
    }

    #[test]
    fn doppelte_praefix_registrierung_zaehlt_laenge_einmal() {
        let g = SagaGate::new();
        g.activate_prefix(b"t:3:7:").unwrap();
        g.activate_prefix(b"t:3:7:").unwrap();
        assert_eq!(g.inner.read().prefix_lengths.get(&6), Some(&1));
        assert_eq!(g.registration_count(), 2);
    }

    #[test]
    fn range_erkennt_erased_und_restricted() {
        let g = SagaGate::new();
        g.activate_key(b"m").unwrap();
        assert_eq!(g.check_range(b"a", b"z"), RangeGate::ContainsErased);
        assert_eq!(g.check_range(b"n", b"z"), RangeGate::AllOpen);
        g.activate_restricted_key(b"k").unwrap();
        assert_eq!(g.check_range(b"a", b"z"), RangeGate::RestrictedNeverDiscard);
    }

    #[test]
    fn check_ist_nahezu_konstant_bei_vielen_schluesseln() {
        // Ratio-Test statt Absolutwert: 1k vs. 100k registrierte Schlüssel.
        fn build(n: usize) -> SagaGate {
            let g = SagaGate::new();
            for i in 0..n {
                g.activate_key(format!("t:1:2:chunk:{i}").as_bytes()).unwrap();
            }
            for p in 0..1000 {
                g.activate_prefix(format!("t:{p}:").as_bytes()).unwrap();
            }
            g
        }
        let small = build(1_000);
        let large = build(100_000);
        let probe = b"t:5000:9:chunk:x"; // kein Treffer: Worst Case für den Lookup
        let time = |g: &SagaGate| {
            let t = std::time::Instant::now();
            let mut acc = 0usize;
            for _ in 0..20_000 {
                acc += (g.check(probe) == GateDecision::Open) as usize;
            }
            assert!(acc > 0);
            t.elapsed().as_nanos() as f64
        };
        let r = time(&large) / time(&small).max(1.0);
        assert!(r < 3.0, "check-Laufzeit wächst zu stark: ratio {r}");
    }
}

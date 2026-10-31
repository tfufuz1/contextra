// ANCHOR[contextra-mvcc:SEQ-LOG-ROOT]
//! # Sequenz-Log (`SequenceLog`) für snapshot-isolierte Index-Suchen (`_at`-Familie)
//!
//! Dieses Modul implementiert ein leichtgewichtiges, speicherinternes Versionsprotokoll
//! für Dokumente. Es zeichnet auf, wann Dokumente eingefügt (`insert_seq`) und wann sie
//! gelöscht wurden (`delete_seq`). Anhand dieser Einträge können Abfragen zu einem
//! historischen Snapshot-Zeitpunkt (`as_of`) exakt ermitteln, ob ein Dokument zu jenem
//! Zeitpunkt sichtbar war oder nicht.
//!
//! ## Relevanz im System
//! - **Ring 0 (Kern)**: Grundlegender Baustein für MVCC (Multi-Version Concurrency Control).
//! - **Konsumenten**: Vektor- und Text-Indizes, die historische Suchen (`search_at`) unterstützen,
//!   sowie SSI-Validatoren (`SequenceLogSsiValidator`).
//! - **Port-Nutzung**: Verwendet den [`Clock`]-Port (`contextra-ports`) für deterministische
//!   Zeitbestimmung (wichtig für TTL-Prüfungen von Snapshot-Pins ohne direkte Systemuhr-Abhängigkeit).

// FILE-CONTEXT
// STAND: 2026-08-30T18:51:56Z (SESSION: e459bd5f)
// ZWECK: Shared Versioned Sequence Log für Snapshot-isolierte Index-Suchen (_at Familie).
// INVARIANTEN: Sichtbarkeitsprüfung: insert_seq <= as_of && (delete_seq.is_none() || delete_seq > as_of).
// HOTSPOTS: 20-75
// NICHT-OFFENSICHTLICH: Compaction prunt Einträge erst wenn delete_seq < min_active_seqno.
// SIEHE AUCH: rules/tag_taxonomy.md, DECISIONS.md (ADR-024)

use std::sync::Arc;
use std::time::Duration;

use contextra_ports::{Clock, SystemClock};

use crate::types::DocId;

// ANCHOR[contextra-mvcc:SEQ-LOG-CONST]
/// Standard-Lebensdauer für Snapshot-Pins (5 Minuten), bevor Diagnose-Warnungen für
/// blockierende oder verwaiste Pins ausgelöst werden.
pub const DEFAULT_MAX_PIN_DURATION: Duration = Duration::from_secs(300);

// ANCHOR[contextra-mvcc:SEQ-LOG-ENTRY]
/// Ein versionierter Protokolleintrag für die Snapshot-Isolation (`_at`-Familie).
///
/// Jeder Eintrag repräsentiert die Lebensspanne einer Dokumentversion:
/// von der Einfügesequenznummer `insert_seq` bis zur Löschsequenznummer `delete_seq`.
///
/// ### Speicherverbrauch
/// Jeder Eintrag belegt 24 Bytes:
/// - 8 Bytes für `DocId` (Dokument-Kennung)
/// - 8 Bytes für `insert_seq` (Sequenznummer der Erstellung)
/// - 8 Bytes für `delete_seq: Option<u64>` (Sequenznummer der Löschung)
///
/// Einträge, deren `delete_seq` unterhalb der minimal aktiven Sequenznummer
/// (`min_active_seqno`) aller aktiven Leser liegt, können via [`SequenceLog::compact`]
/// gefahrlos aus dem Speicher gelöscht (geprunt) werden.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeqLogEntry {
    /// Eindeutige Kennung des Dokuments.
    pub doc_id: DocId,
    /// Logische Sequenznummer, bei der dieser Dokumenteintrag eingefügt wurde.
    pub insert_seq: u64,
    /// Logische Sequenznummer, bei der dieser Eintrag gelöscht wurde (`None`, falls noch aktiv).
    pub delete_seq: Option<u64>,
}

impl SeqLogEntry {
    // ANCHOR[contextra-mvcc:SEQ-LOG-VISIBILITY]
    /// Prüft in O(1), ob dieser Protokolleintrag zu einem bestimmten Snapshot-Stand `as_of` sichtbar ist.
    ///
    /// ### Ablauf der Sichtbarkeitsprüfung:
    /// 1. Maskiere eventuell gesetzte Tombstone-Bits (höchstwertiges Bit) auf allen Sequenznummern aus.
    /// 2. Prüfe, ob der Eintrag vor oder genau zum Snapshot-Zeitpunkt eingefügt wurde (`insert_seq <= as_of`).
    /// 3. Prüfe, ob das Dokument ungelöscht ist (`delete_seq == None`) oder erst nach dem Snapshot
    ///    gelöscht wurde (`delete_seq > as_of`).
    ///
    /// Ist Bedingung 2 und 3 erfüllt, war das Dokument zum Zeitpunkt `as_of` aktiv und sichtbar.
    ///
    /// ### Komplexität
    /// $O(1)$ allokationsfrei und schleifenfrei; benötigt maximal zwei 64-Bit-Ganzzahlvergleiche.
    #[inline]
    pub fn is_visible(&self, as_of: u64) -> bool {
        // NOTE: Tombstone-Bit wird ausmaskiert, damit Berechnungen nicht durch interne Markierungsbits verfälscht werden.
        let as_of = as_of & !crate::types::TOMBSTONE_BIT;
        let insert_seq = self.insert_seq & !crate::types::TOMBSTONE_BIT;
        let delete_seq = self.delete_seq.map(|d| d & !crate::types::TOMBSTONE_BIT);
        insert_seq <= as_of && delete_seq.is_none_or(|del| del > as_of)
    }
}

// ANCHOR[contextra-mvcc:SEQ-LOG-CHANGE]
/// Repräsentiert eine historische Änderung (Einfügen oder Löschen) im Sequenz-Log für Delta-Replays.
///
/// Wird von Abfragen wie [`SequenceLog::changes_since`] erzeugt, um Indizes
/// inkrementell auf den neuesten Stand nachzuziehen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeqLogChange {
    /// Einfügen eines Dokuments zu einer bestimmten Sequenznummer.
    Insert {
        /// Eindeutige Kennung des Dokuments.
        doc_id: DocId,
        /// Sequenznummer des Einfügens.
        seq: u64,
    },
    /// Löschen eines Dokuments zu einer bestimmten Sequenznummer.
    Delete {
        /// Eindeutige Kennung des Dokuments.
        doc_id: DocId,
        /// Sequenznummer des Löschens.
        seq: u64,
    },
}

impl SeqLogChange {
    /// Liefert die logische Sequenznummer dieser Änderung.
    pub fn seq(&self) -> u64 {
        match self {
            Self::Insert { seq, .. } | Self::Delete { seq, .. } => *seq,
        }
    }

    /// Liefert die Kennung des betroffenen Dokuments.
    pub fn doc_id(&self) -> DocId {
        match self {
            Self::Insert { doc_id, .. } | Self::Delete { doc_id, .. } => *doc_id,
        }
    }
}

// ANCHOR[contextra-mvcc:SEQ-LOG-STRUCT]
/// Verwaltet das chronologische Sequenz-Protokoll und Sichtbarkeitsfilter für Indexstrukturen.
///
/// ### Funktionsweise:
/// `SequenceLog` speichert alle Einfüge- und Löschvorgänge chronologisch in einem Vektor (`entries`).
/// Zudem merkt es sich aktive Snapshot-Pins (`pinned_snapshots`), um zu verhindern, dass alte
/// Datenbereiche in Indizes vorzeitig freigegeben (geprunt) werden, solange noch Leser darauf zugreifen.
///
/// ### Port-Nutzung:
/// Verwendet [`Clock`] zur Erfassung von Zeitstempeln für gepinnte Snapshots, um verwaiste Pins
/// diagnostizieren und automatisch bereinigen zu können.
#[derive(Clone)]
pub struct SequenceLog {
    /// Liste aller historischen Dokumentversionen.
    entries: Vec<SeqLogEntry>,
    /// Zuordnung von gepinnter Sequenznummer zu Liste von Erstellungs-Zeitstempeln (in Nanosekunden).
    pinned_snapshots: ahash::AHashMap<u64, Vec<u64>>,
    /// Unterste Grenze, bis zu der bereits eine Verdichtung (Kompaktierung) stattgefunden hat.
    compacted_below: Option<u64>,
    /// Schneller Lookup für aktuell gelöschte Dokumente und ihre Lösch-Sequenznummer.
    deletions: ahash::AHashMap<DocId, u64>,
    /// Konfigurierte Maximaldauer für aktive Snapshot-Pins (Standard: 5 Minuten).
    max_pin_duration: Duration,
    /// Abstrakte Systemuhr zur Ermittlung monoton steigender Nanosekunden.
    clock: Arc<dyn Clock>,
}

impl std::fmt::Debug for SequenceLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SequenceLog")
            .field("entries", &self.entries)
            .field("pinned_snapshots", &self.pinned_snapshots)
            .field("compacted_below", &self.compacted_below)
            .field("deletions", &self.deletions)
            .field("max_pin_duration", &self.max_pin_duration)
            .finish()
    }
}

impl Default for SequenceLog {
    fn default() -> Self {
        Self::new()
    }
}

impl SequenceLog {
    // ANCHOR[contextra-mvcc:SEQ-LOG-NEW]
    /// Erstellt ein neues, leeres `SequenceLog` mit der Standard-Systemuhr [`SystemClock`]
    /// und einer maximalen Pin-Dauer von 5 Minuten.
    pub fn new() -> Self {
        Self::with_clock(Arc::new(SystemClock::new()))
    }

    // ANCHOR[contextra-mvcc:SEQ-LOG-WITH-CLOCK]
    /// Erstellt ein neues, leeres `SequenceLog` mit einer benutzerdefinierten Implementierung
    /// des [`Clock`]-Ports (z. B. `ManualClock` in deterministischen Tests).
    pub fn with_clock(clock: Arc<dyn Clock>) -> Self {
        Self {
            entries: Vec::new(),
            pinned_snapshots: ahash::AHashMap::default(),
            compacted_below: None,
            deletions: ahash::AHashMap::default(),
            max_pin_duration: DEFAULT_MAX_PIN_DURATION,
            clock,
        }
    }

    /// Gibt eine Referenz auf den aktiven [`Clock`]-Port zurück.
    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

    /// Ersetzt die aktive [`Clock`]-Instanz für dieses Log.
    pub fn set_clock(&mut self, clock: Arc<dyn Clock>) {
        self.clock = clock;
    }

    /// Konfiguriert die maximale Pin-Dauer als Fluent-Builder und gibt `self` zurück.
    pub fn with_max_pin_duration(mut self, duration: Duration) -> Self {
        self.max_pin_duration = duration;
        self
    }

    /// Setzt die maximale Pin-Dauer für Diagnosezwecke.
    pub fn set_max_pin_duration(&mut self, duration: Duration) {
        self.max_pin_duration = duration;
    }

    /// Liefert die aktuell konfigurierte Maximaldauer für Pins.
    pub fn max_pin_duration(&self) -> Duration {
        self.max_pin_duration
    }

    // ANCHOR[contextra-mvcc:SEQ-LOG-PIN]
    /// Pinnt eine historische Sequenznummer, um zu verhindern, dass die Hintergrund-Bereinigung
    /// (Rebuild/Compaction) gelöschte Knoten entfernt, die für diesen Stand noch sichtbar sein müssen.
    ///
    /// ### Ablauf:
    /// 1. Aktuellen Zeitstempel in Nanosekunden über die konfigurierte [`Clock`] abfragen.
    /// 2. Sequenznummer und Zeitstempel in `pinned_snapshots` registrieren.
    pub fn pin_snapshot(&mut self, seq_no: u64) {
        let now_nanos = self.clock.monotonic_nanos();
        self.pin_snapshot_at(seq_no, now_nanos);
    }

    /// Pinnt eine Sequenznummer zu einem expliziten Nanosekunden-Zeitstempel (für deterministische Tests).
    pub fn pin_snapshot_at(&mut self, seq_no: u64, at_nanos: u64) {
        let seq_no = seq_no & !crate::types::TOMBSTONE_BIT;
        self.pinned_snapshots
            .entry(seq_no)
            .or_default()
            .push(at_nanos);
    }

    // ANCHOR[contextra-mvcc:SEQ-LOG-UNPIN]
    /// Gibt einen gepinnten Snapshot wieder frei.
    ///
    /// ### Ablauf:
    /// 1. Tombstone-Bit der Sequenznummer ausmaskieren.
    /// 2. Den letzten Zeitstempel vom Stack dieses Eintrags entfernen (LIFO).
    /// 3. Ist die Liste der Zeitstempel für diese Sequenznummer leer, den Eintrag vollständig aus der Map löschen.
    pub fn unpin_snapshot(&mut self, seq_no: u64) {
        let seq_no = seq_no & !crate::types::TOMBSTONE_BIT;
        if let std::collections::hash_map::Entry::Occupied(mut entry) =
            self.pinned_snapshots.entry(seq_no)
        {
            let timestamps = entry.get_mut();
            timestamps.pop();
            if timestamps.is_empty() {
                entry.remove();
            }
        }
    }

    // ANCHOR[contextra-mvcc:SEQ-LOG-EXPIRED-PINS]
    /// Findet alle gepinnten Sequenznummern, deren Haltezeit `max_pin_duration` überschreitet,
    /// bezogen auf den übergebenen Monotonie-Zeitstempel `now_nanos`.
    ///
    /// Rückgabe ist aufsteigend sortiert.
    pub fn expired_pins_at(&self, now_nanos: u64) -> Vec<u64> {
        let max_nanos = u64::try_from(self.max_pin_duration.as_nanos()).unwrap_or(u64::MAX);
        let mut expired = Vec::new();
        for (&seq_no, timestamps) in &self.pinned_snapshots {
            for &ts in timestamps {
                if now_nanos.saturating_sub(ts) > max_nanos {
                    expired.push(seq_no);
                    break;
                }
            }
        }
        expired.sort_unstable();
        expired
    }

    /// Liefert alle abgelaufenen Snapshot-Pins bezogen auf die aktuelle Systemuhr.
    pub fn expired_pins(&self) -> Vec<u64> {
        self.expired_pins_at(self.clock.monotonic_nanos())
    }

    // ANCHOR[contextra-mvcc:SEQ-LOG-REAP-PINS]
    /// Bereinigt (entpinnt) alle Pins, die die Maximaldauer bezüglich `now_nanos` überschritten haben.
    ///
    /// ### Ablauf:
    /// 1. Abgelaufene Pins via [`SequenceLog::expired_pins_at`] ermitteln.
    /// 2. Für jeden abgelaufenen Pin [`SequenceLog::unpin_snapshot`] aufrufen.
    /// 3. Liste der entfernten Sequenznummern zurückgeben.
    pub fn reap_expired_pins_at(&mut self, now_nanos: u64) -> Vec<u64> {
        let expired = self.expired_pins_at(now_nanos);
        for &seq in &expired {
            self.unpin_snapshot(seq);
        }
        expired
    }

    /// Bereinigt alle abgelaufenen Snapshot-Pins bezogen auf die aktuelle Uhrzeit.
    pub fn reap_expired_pins(&mut self) -> Vec<u64> {
        let now_nanos = self.clock.monotonic_nanos();
        self.reap_expired_pins_at(now_nanos)
    }

    // ANCHOR[contextra-mvcc:SEQ-LOG-MIN-RETENTION]
    /// Berechnet die kleinste aktuell gepinnte Sequenznummer für Aufbewahrungs- und Pruning-Zwecke.
    ///
    /// Gibt `None` zurück, falls aktuell kein einziger Snapshot gepinnt ist.
    /// Falls ein aktiver Pin die konfigurierte Maximaldauer überschreitet, wird eine Tracing-Warnung ausgegeben.
    pub fn min_retention_seq_at(&self, now_nanos: u64) -> Option<u64> {
        let min_seq = self.pinned_snapshots.keys().copied().min();
        if let Some(seq) = min_seq {
            if let Some(timestamps) = self.pinned_snapshots.get(&seq) {
                let max_nanos = u64::try_from(self.max_pin_duration.as_nanos()).unwrap_or(u64::MAX);
                for &ts in timestamps {
                    let duration_nanos = now_nanos.saturating_sub(ts);
                    if duration_nanos > max_nanos {
                        tracing::warn!(
                            seq_no = seq,
                            max_pin_duration = ?self.max_pin_duration,
                            elapsed_nanos = duration_nanos,
                            "Snapshot pin active beyond max duration"
                        );
                        break;
                    }
                }
            }
        }
        min_seq
    }

    /// Berechnet die kleinste aktuell gepinnte Sequenznummer bezogen auf die aktuelle Systemuhr.
    pub fn min_retention_seq(&self) -> Option<u64> {
        self.min_retention_seq_at(self.clock.monotonic_nanos())
    }

    /// Liefert die Lösch-Sequenznummer des Dokuments, falls es aktuell als gelöscht verzeichnet ist.
    pub fn deletion_seq(&self, doc_id: DocId) -> Option<u64> {
        self.deletions.get(&doc_id).copied()
    }

    // ANCHOR[contextra-mvcc:SEQ-LOG-RECORD-INSERT]
    /// Zeichnet eine Einfügeoperation für ein Dokument zu einer Sequenznummer auf.
    ///
    /// ### Ablauf:
    /// 1. Tombstone-Bit der Sequenznummer ausmaskieren.
    /// 2. Falls für `doc_id` bereits eine ungelöschte Version existiert, diese mit `delete_seq = Some(seq)`
    ///    abschließen (Überschreiben = implizites Ersetzen).
    /// 3. Vorherige Löschmarkierung aus der Schnellsuch-Map `deletions` entfernen.
    /// 4. Neuen Eintrag [`SeqLogEntry`] mit `insert_seq = seq` und `delete_seq = None` anfügen.
    pub fn record_insert(&mut self, doc_id: DocId, seq: u64) {
        let seq = seq & !crate::types::TOMBSTONE_BIT;
        if let Some(entry) = self
            .entries
            .iter_mut()
            .rfind(|e| e.doc_id == doc_id && e.delete_seq.is_none())
        {
            entry.delete_seq = Some(seq);
        }
        self.deletions.remove(&doc_id);
        self.entries.push(SeqLogEntry {
            doc_id,
            insert_seq: seq,
            delete_seq: None,
        });
    }

    // ANCHOR[contextra-mvcc:SEQ-LOG-RECORD-DELETE]
    /// Zeichnet eine Löschoperation für ein Dokument zu einer Sequenznummer auf.
    ///
    /// ### Ablauf:
    /// 1. Tombstone-Bit der Sequenznummer ausmaskieren.
    /// 2. Den letzten noch aktiven Eintrag des Dokuments suchen und dessen `delete_seq` auf `seq` setzen.
    /// 3. Dokument und Lösch-Sequenznummer in `deletions` aufnehmen.
    pub fn record_delete(&mut self, doc_id: DocId, seq: u64) {
        let seq = seq & !crate::types::TOMBSTONE_BIT;
        if let Some(entry) = self
            .entries
            .iter_mut()
            .rfind(|e| e.doc_id == doc_id && e.delete_seq.is_none())
        {
            entry.delete_seq = Some(seq);
            self.deletions.insert(doc_id, seq);
        }
    }

    // ANCHOR[contextra-mvcc:SEQ-LOG-IS-VISIBLE]
    /// Prüft, ob ein Dokument zu einer bestimmten Sequenznummer `seq_no` im Index sichtbar ist.
    ///
    /// Ein Dokument ist sichtbar, wenn es vor oder bei `seq_no` eingefügt wurde und nicht
    /// vor oder bei `seq_no` gelöscht wurde.
    pub fn is_visible(&self, doc_id: DocId, seq_no: u64) -> bool {
        let seq_no = seq_no & !crate::types::TOMBSTONE_BIT;
        let mut inserted = false;
        let mut deleted = false;
        for entry in &self.entries {
            let insert_seq = entry.insert_seq & !crate::types::TOMBSTONE_BIT;
            if entry.doc_id == doc_id && insert_seq <= seq_no {
                inserted = true;
                if let Some(del) = entry.delete_seq {
                    let del_seq = del & !crate::types::TOMBSTONE_BIT;
                    deleted = del_seq <= seq_no;
                } else {
                    deleted = false;
                }
            }
        }
        inserted && !deleted
    }

    // ANCHOR[contextra-mvcc:SEQ-LOG-COMPACT]
    /// Verdichtet (kompaktiert) das Sequenz-Log und entfernt historische Versionen,
    /// deren Löschung strikt kleiner als `min_active_seqno` ist.
    ///
    /// ### Invariante:
    /// Versionen mit `delete_seq >= min_active_seqno` MÜSSEN im Speicher verbleiben, da
    /// aktive Leser mit einem älteren Snapshot diese Versionen noch sehen können müssen.
    ///
    /// ### Ablauf:
    /// 1. `min_active_seqno` maskieren und `compacted_below`-Wasserstand anpassen.
    /// 2. `entries` filtern: Nur Einträge behalten, die noch ungelöscht sind ODER deren
    ///    `delete_seq >= min_active_seqno` ist.
    /// 3. Map `deletions` bereinigen.
    pub fn compact(&mut self, min_active_seqno: u64) {
        let min_active_seqno = min_active_seqno & !crate::types::TOMBSTONE_BIT;
        self.compacted_below = Some(
            self.compacted_below
                .map_or(min_active_seqno, |c| c.max(min_active_seqno)),
        );
        self.entries.retain(|entry| {
            if let Some(del_seq) = entry.delete_seq {
                let del_seq = del_seq & !crate::types::TOMBSTONE_BIT;
                del_seq >= min_active_seqno
            } else {
                true
            }
        });
        self.deletions
            .retain(|_, &mut del_seq| (del_seq & !crate::types::TOMBSTONE_BIT) >= min_active_seqno);
    }

    /// Gibt die Anzahl der aktuell im Log vorgehaltenen Einträge zurück.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Gibt `true` zurück, wenn das Sequenz-Log keine Einträge enthält.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    // ANCHOR[contextra-mvcc:SEQ-LOG-CHANGES-SINCE]
    /// Liefert alle Änderungen (Einfügungen und Löschungen), die strikt nach `snapshot_seq`
    /// stattgefunden haben, aufsteigend sortiert nach Sequenznummer.
    ///
    /// Wird verwendet, um einen veralteten Index-Zustand per Delta-Replay schnell
    /// auf einen neueren Stand zu aktualisieren.
    pub fn changes_since(&self, snapshot_seq: u64) -> Vec<SeqLogChange> {
        let snapshot_seq = snapshot_seq & !crate::types::TOMBSTONE_BIT;
        let mut changes = Vec::new();
        for entry in &self.entries {
            let insert_seq = entry.insert_seq & !crate::types::TOMBSTONE_BIT;
            if insert_seq > snapshot_seq {
                changes.push(SeqLogChange::Insert {
                    doc_id: entry.doc_id,
                    seq: insert_seq,
                });
            }
            if let Some(del_seq) = entry.delete_seq {
                let del_seq = del_seq & !crate::types::TOMBSTONE_BIT;
                if del_seq > snapshot_seq {
                    changes.push(SeqLogChange::Delete {
                        doc_id: entry.doc_id,
                        seq: del_seq,
                    });
                }
            }
        }
        changes.sort_by_key(|c| c.seq());
        changes
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use contextra_testkit::ManualClock;

    // Prüft die O(1)-Sichtbarkeitsprüfung eines einzelnen SeqLogEntry-Eintrags an verschiedenen Grenzwerten.
    #[test]
    fn test_seq_log_entry_is_visible_o1() {
        let doc = DocId::from_key("doc1").expect("valid doc id"); // #[cfg(test)]

        // Active entry (no delete)
        let active_entry = SeqLogEntry {
            doc_id: doc,
            insert_seq: 10,
            delete_seq: None,
        };
        assert!(!active_entry.is_visible(9));
        assert!(active_entry.is_visible(10));
        assert!(active_entry.is_visible(100));

        // Deleted entry
        let deleted_entry = SeqLogEntry {
            doc_id: doc,
            insert_seq: 10,
            delete_seq: Some(20),
        };
        assert!(!deleted_entry.is_visible(9));
        assert!(deleted_entry.is_visible(10));
        assert!(deleted_entry.is_visible(19));
        // Boundary case: delete_seq == as_of -> NOT visible (delete_seq > as_of is strict)
        assert!(!deleted_entry.is_visible(20));
        assert!(!deleted_entry.is_visible(21));
    }

    // Prüft das Zusammenspiel von Einfügen, Löschen und der Sichtbarkeit von Dokumenten im Sequenzlog.
    #[test]
    fn test_sequence_log_insert_delete_visibility() {
        let mut log = SequenceLog::new();
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);

        let doc1 = DocId::from_key("doc1").expect("valid doc id"); // expect #[cfg(test)]
        let doc2 = DocId::from_key("doc2").expect("valid doc id"); // expect #[cfg(test)]

        // Insert doc1 at seq 10, doc2 at seq 20
        log.record_insert(doc1, 10);
        log.record_insert(doc2, 20);
        assert_eq!(log.len(), 2);
        assert!(!log.is_empty());

        // Visibility before insert
        assert!(!log.is_visible(doc1, 5));
        assert!(!log.is_visible(doc2, 15));

        // Visibility at and after insert
        assert!(log.is_visible(doc1, 10));
        assert!(log.is_visible(doc1, 15));
        assert!(log.is_visible(doc2, 20));

        // Record delete for doc1 at seq 30
        log.record_delete(doc1, 30);
        assert_eq!(log.len(), 2);

        // Visibility around delete seq
        assert!(log.is_visible(doc1, 29));
        assert!(!log.is_visible(doc1, 30));
        assert!(!log.is_visible(doc1, 35));

        // doc2 remains visible
        assert!(log.is_visible(doc2, 35));
    }

    // Prüft die korrekte Versionsbehandlung bei Wiedereinfügen eines zuvor gelöschten Dokuments.
    #[test]
    fn test_sequence_log_reinsert_after_delete() {
        let mut log = SequenceLog::new();
        let doc1 = DocId::from_key("doc1").expect("valid doc id"); // expect #[cfg(test)]

        log.record_insert(doc1, 10);
        log.record_delete(doc1, 20);
        assert!(!log.is_visible(doc1, 25));

        // Re-insert doc1 at seq 30
        log.record_insert(doc1, 30);
        assert_eq!(log.len(), 2);

        // Visibility timelines
        assert!(log.is_visible(doc1, 15));
        assert!(!log.is_visible(doc1, 25));
        assert!(log.is_visible(doc1, 30));
        assert!(log.is_visible(doc1, 40));
    }

    // Prüft, dass die Kompaktierung nur gelöschte Einträge unterhalb des Schwellenwerts entfernt und aktive behält.
    #[test]
    fn test_sequence_log_compact() {
        let mut log = SequenceLog::new();
        let doc1 = DocId::from_key("doc1").expect("valid doc id"); // expect #[cfg(test)]
        let doc2 = DocId::from_key("doc2").expect("valid doc id"); // expect #[cfg(test)]

        log.record_insert(doc1, 10);
        log.record_delete(doc1, 20); // deleted at seq 20

        log.record_insert(doc2, 15);
        log.record_delete(doc2, 40); // deleted at seq 40

        assert_eq!(log.len(), 2);

        // Compact with min_active_seqno = 30
        // doc1 (delete_seq = 20 < 30) should be purged.
        // doc2 (delete_seq = 40 >= 30) should be retained.
        log.compact(30);
        assert_eq!(log.len(), 1);

        // Compact with min_active_seqno = 50
        // doc2 (delete_seq = 40 < 50) should be purged.
        log.compact(50);
        assert_eq!(log.len(), 0);
        assert!(log.is_empty());
    }

    // Prüft das Ausmaskieren des Tombstone-Bits bei Einfüge- und Löschoperationen.
    #[test]
    fn test_sequence_log_edge_cases() {
        let mut log = SequenceLog::new();
        let doc1 = DocId::from_key("doc1").expect("valid doc id"); // expect #[cfg(test)]

        // Delete non-existent doc should not panic or add entry
        log.record_delete(doc1, 10);
        assert_eq!(log.len(), 0);

        // Multiple deletes on same active insert updates delete_seq or ignores second?
        log.record_insert(doc1, 5);
        log.record_delete(doc1, 15);
        log.record_delete(doc1, 25); // Should not overwrite existing delete_seq if already deleted
        assert!(log.is_visible(doc1, 10));
        assert!(!log.is_visible(doc1, 15));
    }

    // Prüft die chronologische Rückgabe aller Änderungen seit einem Referenz-Snapshot.
    #[test]
    fn test_sequence_log_changes_since() {
        let mut log = SequenceLog::new();
        let doc1 = DocId::from_key("doc1").expect("valid doc id"); // #[cfg(test)]
        let doc2 = DocId::from_key("doc2").expect("valid doc id"); // #[cfg(test)]

        log.record_insert(doc1, 10);
        log.record_insert(doc2, 20);
        log.record_delete(doc1, 30);
        log.record_insert(doc1, 40);

        let changes_0 = log.changes_since(0);
        assert_eq!(
            changes_0,
            vec![
                SeqLogChange::Insert {
                    doc_id: doc1,
                    seq: 10
                },
                SeqLogChange::Insert {
                    doc_id: doc2,
                    seq: 20
                },
                SeqLogChange::Delete {
                    doc_id: doc1,
                    seq: 30
                },
                SeqLogChange::Insert {
                    doc_id: doc1,
                    seq: 40
                },
            ]
        );

        let changes_15 = log.changes_since(15);
        assert_eq!(
            changes_15,
            vec![
                SeqLogChange::Insert {
                    doc_id: doc2,
                    seq: 20
                },
                SeqLogChange::Delete {
                    doc_id: doc1,
                    seq: 30
                },
                SeqLogChange::Insert {
                    doc_id: doc1,
                    seq: 40
                },
            ]
        );

        let changes_35 = log.changes_since(35);
        assert_eq!(
            changes_35,
            vec![SeqLogChange::Insert {
                doc_id: doc1,
                seq: 40
            }]
        );

        let changes_50 = log.changes_since(50);
        assert!(changes_50.is_empty());
    }

    // Prüft Referenzzählung und korrekte Minimalwert-Berechnung beim Pinnen und Entpinnen von Snapshots.
    #[test]
    fn test_sequence_log_pinning_and_retention() {
        let mut log = SequenceLog::new();
        let doc1 = DocId::from_key("doc1").expect("valid doc id"); // #[cfg(test)]

        log.record_insert(doc1, 10);
        log.record_delete(doc1, 20);

        assert_eq!(log.deletion_seq(doc1), Some(20));
        assert_eq!(log.min_retention_seq(), None);

        // Pin snapshot at seq 5 (older than insert_seq 10)
        log.pin_snapshot(5);
        assert_eq!(log.min_retention_seq(), Some(5));

        // Pin another snapshot at seq 15
        log.pin_snapshot(15);
        assert_eq!(log.min_retention_seq(), Some(5));

        // Unpin seq 5
        log.unpin_snapshot(5);
        assert_eq!(log.min_retention_seq(), Some(15));

        // Unpin seq 15
        log.unpin_snapshot(15);
        assert_eq!(log.min_retention_seq(), None);
    }

    // Prüft TTL-Ablauf und das automatische Bereinigen veralteter Snapshot-Pins über eine manuelle Uhr.
    #[test]
    fn test_sequence_log_pin_ttl_and_expiration() {
        let clock = Arc::new(ManualClock::new(100_000_000_000));
        let mut log =
            SequenceLog::with_clock(clock.clone()).with_max_pin_duration(Duration::from_secs(10));
        assert_eq!(log.max_pin_duration(), Duration::from_secs(10));

        let start_nanos = clock.now_nanos();
        let past_5s_nanos = start_nanos.saturating_sub(5_000_000_000);
        let past_15s_nanos = start_nanos.saturating_sub(15_000_000_000);

        // Pin seq 100 at past_5s (not expired)
        log.pin_snapshot_at(100, past_5s_nanos);
        // Pin seq 200 at past_15s (expired relative to current clock)
        log.pin_snapshot_at(200, past_15s_nanos);

        let expired = log.expired_pins();
        assert_eq!(expired, vec![200]);

        // Pin seq 200 a second time at past_5s (now has 2 pins: 1 expired, 1 active)
        log.pin_snapshot_at(200, past_5s_nanos);
        assert_eq!(log.min_retention_seq(), Some(100));

        // expired_pins should still report 200 because at least one pin on seq 200 is expired
        assert_eq!(log.expired_pins(), vec![200]);

        // Unpin newest pin on 200 (pops past_5s pin)
        log.unpin_snapshot(200);
        // past_15s pin remains for seq 200, so seq 200 is still reported as expired
        assert_eq!(log.expired_pins(), vec![200]);

        // Unpin second pin on 200 (pops past_15s pin) -> no pins left for seq 200
        log.unpin_snapshot(200);
        assert_eq!(log.expired_pins(), Vec::<u64>::new());
        assert_eq!(log.min_retention_seq(), Some(100));

        // Unpin seq 100 -> no pins left
        log.unpin_snapshot(100);
        assert_eq!(log.min_retention_seq(), None);
    }

    // Prüft die korrekte Handhabung einer benutzerdefinierten maximalen Pin-Dauer.
    #[test]
    fn test_sequence_log_pin_ttl_custom_duration() {
        let clock = Arc::new(ManualClock::new(1_000_000_000));
        let mut log = SequenceLog::with_clock(clock.clone());
        assert_eq!(log.max_pin_duration(), DEFAULT_MAX_PIN_DURATION);

        log.set_max_pin_duration(Duration::from_millis(50));
        assert_eq!(log.max_pin_duration(), Duration::from_millis(50));

        let now_nanos = clock.now_nanos();
        let past_100ms_nanos = now_nanos.saturating_sub(100_000_000);

        log.pin_snapshot_at(42, past_100ms_nanos);
        assert_eq!(log.expired_pins_at(now_nanos), vec![42]);

        // The pin is NOT automatically removed or forcibly unpinned by expired_pins
        assert_eq!(log.min_retention_seq(), Some(42));
        assert_eq!(log.expired_pins(), vec![42]);

        // Calling reap_expired_pins explicitly unpins expired sequence numbers
        let reaped = log.reap_expired_pins();
        assert_eq!(reaped, vec![42]);
        assert_eq!(log.min_retention_seq(), None);
    }

    // Prüft, dass rückwärtsspringende Zeitstempel dank sättigender Subtraktion keine Panics auslösen.
    #[test]
    fn test_sequence_log_zero_panic_time_arithmetic() {
        let clock = Arc::new(ManualClock::new(100_000_000));
        let mut log = SequenceLog::with_clock(clock.clone());
        let now_nanos = clock.now_nanos();
        let future_nanos = now_nanos + 100_000_000_000;

        // Pin in the future (clock skew edge case)
        log.pin_snapshot_at(50, future_nanos);

        // Checking expired pins with current clock should handle saturating duration without panic
        let expired = log.expired_pins();
        assert!(expired.is_empty());

        assert_eq!(log.min_retention_seq(), Some(50));
    }
}

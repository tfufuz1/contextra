//! # SequenceLog — Versionsbasiertes Sequenzlog für Snapshot-Sichten
//!
//! **Ring:** Ring 0 (Transaktionsisolation)
//! **Klasse:** Kern `[K]`
//!
//! ## Zweck & Architektur-Rolle
//!
//! Dieses Modul implementiert ein leichtgewichtiges, chronologisches Sequenzlog für Dokumentoperationen.
//! Es dient insbesondere Vektor- und Volltextindizes (`_at`-Suchmethoden) dazu, historische Sichtbarkeiten
//! von Dokumenten zu einem bestimmten Sequenzzeitpunkt (`as_of`) zu ermitteln, ohne den Hauptspeicher-Index
//! physisch duplizieren zu müssen.
//!
//! ### Sichtbarkeitsregel (Visibility Rule)
//! Ein Dokument ist zum Zeitpunkt `as_of` genau dann sichtbar, wenn:
//! `insert_seq <= as_of` UND (`delete_seq` ist `None` ODER `delete_seq > as_of`).
//!
//! ### Tombstone-Bit-Maskierung
//! Sequenznummern können systemweit mit dem höchstwertigen Bit (`TOMBSTONE_BIT`, Bit 63) als Löschung
//! gekennzeichnet sein. Sämtliche Methoden in diesem Modul maskieren dieses Bit strikt heraus (`& !TOMBSTONE_BIT`),
//! um Vergleiche rein auf den arithmetischen Sequenzzählern durchzuführen.

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

// ANCHOR[contextra-mvcc:SEQ-001]
/// Standard-Maximaldauer für Snapshot-Pins (5 Minuten), bevor Diagnose-Warnungen für abgelaufene Pins ausgelöst werden.
// NOTE: 5 Minuten verhindern, dass vergessene Transaktions-Pins unbemerkt die Speicherbereinigung blockieren.
pub const DEFAULT_MAX_PIN_DURATION: Duration = Duration::from_secs(300);

// ANCHOR[contextra-mvcc:SEQ-002]
/// Versionsbasierter Eintrag im Sequenzlog für Snapshot-Isolation (`_at`-Familie).
///
/// **Speicher-Overhead**: Jeder Eintrag belegt exakt 24 Bytes (8 Bytes `DocId` + 8 Bytes `insert_seq` + 8 Bytes `delete_seq: Option<u64>`).
/// Einträge, deren `delete_seq` kleiner als die minimal aktive Sequenznummer (`min_active_seqno`) ist, können über `compact()` sicher entfernt werden.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeqLogEntry {
    /// Eindeutige Dokument-ID (64-Bit-Bezeichner).
    pub doc_id: DocId,
    /// Sequenznummer, zu der dieser Dokumenteintrag eingefügt wurde.
    pub insert_seq: u64,
    /// Sequenznummer, zu der dieser Dokumenteintrag gelöscht wurde (`None`, falls aktiv).
    pub delete_seq: Option<u64>,
}

impl SeqLogEntry {
    // ANCHOR[contextra-mvcc:SEQ-003]
    /// Prüft, ob dieser Log-Eintrag zum Snapshot-Sequenzzeitpunkt `as_of` sichtbar ist.
    ///
    /// # Ablauf
    /// 1. Maskiere das `TOMBSTONE_BIT` aus `as_of`, `insert_seq` und optional `delete_seq` heraus.
    /// 2. Prüfe, ob die Einfügesequenz kleiner oder gleich dem Abfragezeitpunkt ist (`insert_seq <= as_of`).
    /// 3. Prüfe, ob keine Löschung vorliegt oder diese erst nach `as_of` stattfand (`delete_seq.is_none_or(|del| del > as_of)`).
    ///
    /// # Komplexität
    /// $O(1)$ allokationsfreie und schleifenfreie Prüfung über maximal zwei Integer-Vergleiche.
    ///
    /// # Parameter
    /// - `as_of`: Der historische Snapshot-Sequenzzeitpunkt.
    #[inline]
    pub fn is_visible(&self, as_of: u64) -> bool {
        // NOTE: Tombstone-Bit wird maskiert, um reine Versionsnummern zu vergleichen.
        let as_of = as_of & !crate::types::TOMBSTONE_BIT;
        let insert_seq = self.insert_seq & !crate::types::TOMBSTONE_BIT;
        let delete_seq = self.delete_seq.map(|d| d & !crate::types::TOMBSTONE_BIT);
        insert_seq <= as_of && delete_seq.is_none_or(|del| del > as_of)
    }
}

// ANCHOR[contextra-mvcc:SEQ-004]
/// Repräsentiert eine historische Änderung (Einfügung oder Löschung) zur Rekonstruktion von Deltas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeqLogChange {
    /// Einfügung eines Dokuments zu einer bestimmten Sequenznummer.
    Insert {
        /// Dokument-ID des neu eingefügten Dokuments.
        doc_id: DocId,
        /// Sequenznummer des Einfügevorgangs.
        seq: u64,
    },
    /// Löschung eines Dokuments zu einer bestimmten Sequenznummer.
    Delete {
        /// Dokument-ID des gelöschten Dokuments.
        doc_id: DocId,
        /// Sequenznummer des Löschvorgangs.
        seq: u64,
    },
}

impl SeqLogChange {
    // ANCHOR[contextra-mvcc:SEQ-005]
    /// Gibt die mit dieser Änderung verbundene Sequenznummer zurück.
    pub fn seq(&self) -> u64 {
        match self {
            Self::Insert { seq, .. } | Self::Delete { seq, .. } => *seq,
        }
    }

    // ANCHOR[contextra-mvcc:SEQ-006]
    /// Gibt die Dokument-ID zurück, auf die sich diese Änderung bezieht.
    pub fn doc_id(&self) -> DocId {
        match self {
            Self::Insert { doc_id, .. } | Self::Delete { doc_id, .. } => *doc_id,
        }
    }
}

// ANCHOR[contextra-mvcc:SEQ-007]
/// Verwaltungskomponente für das Sequenzlog und Sichtbarkeitsfilterung von Indizes.
///
/// Hält chronologische Versionseinträge vor, verwaltet temporäre Snapshot-Pins zur Verhinderung
/// voreiliger Kompaktierung und bietet Delta-Abfragen über `changes_since()`.
#[derive(Clone)]
pub struct SequenceLog {
    /// Chronologische Liste aller Versionseinträge.
    entries: Vec<SeqLogEntry>,
    /// Gepinnte Sequenznummern mit ihren Erstellungs-Zeitstempeln (Nanosekunden) für Lease-Tracking.
    pinned_snapshots: ahash::AHashMap<u64, Vec<u64>>,
    /// Untere Grenze bisheriger Kompaktierungen (`None`, wenn noch nie kompaktiert).
    compacted_below: Option<u64>,
    /// Aktive Soft-Deletes: Zuordnung von `DocId` zu ihrer Löschsequenznummer.
    deletions: ahash::AHashMap<DocId, u64>,
    /// Maximale Haltezeit für Pins vor Ausgabe von Warnungen oder Reaping.
    max_pin_duration: Duration,
    /// Zeit-Port für deterministische Zeitabfragen (Regel 3 aus CLAUDE.md).
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
    // ANCHOR[contextra-mvcc:SEQ-008]
    /// Erstellt ein neues, leeres Sequenzlog mit Standard-Pin-Dauer (5 Minuten) und [`SystemClock`].
    pub fn new() -> Self {
        Self::with_clock(Arc::new(SystemClock::new()))
    }

    // ANCHOR[contextra-mvcc:SEQ-009]
    /// Erstellt ein neues, leeres Sequenzlog unter Verwendung der übergebenen [`Clock`]-Instanz.
    ///
    /// Dies ermöglicht deterministische Zeitsteuerung in Tests via Mock-Uhren.
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

    // ANCHOR[contextra-mvcc:SEQ-010]
    /// Gibt eine Referenz auf den aktuell konfigurierten [`Clock`]-Port zurück.
    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

    // ANCHOR[contextra-mvcc:SEQ-011]
    /// Ersetzt den aktiven [`Clock`]-Port dieses Sequenzlogs.
    pub fn set_clock(&mut self, clock: Arc<dyn Clock>) {
        self.clock = clock;
    }

    // ANCHOR[contextra-mvcc:SEQ-012]
    /// Setzt eine benutzerdefinierte maximale Pin-Dauer für Diagnose-Warnungen (Builder-Muster).
    pub fn with_max_pin_duration(mut self, duration: Duration) -> Self {
        self.max_pin_duration = duration;
        self
    }

    // ANCHOR[contextra-mvcc:SEQ-013]
    /// Konfiguriert die maximale Pin-Dauer für Diagnose-Warnungen im laufenden Betrieb.
    pub fn set_max_pin_duration(&mut self, duration: Duration) {
        self.max_pin_duration = duration;
    }

    // ANCHOR[contextra-mvcc:SEQ-014]
    /// Liefert die aktuell konfigurierte maximale Pin-Dauer zurück.
    pub fn max_pin_duration(&self) -> Duration {
        self.max_pin_duration
    }

    // ANCHOR[contextra-mvcc:SEQ-015]
    /// Pinnt eine historische Sequenznummer, um zu verhindern, dass GC/Kompaktierung
    /// gelöschte Knoten entfernt, die in diesem Snapshot noch sichtbar sein müssen.
    ///
    /// Fragt den Zeitstempel über die interne Uhr ab.
    pub fn pin_snapshot(&mut self, seq_no: u64) {
        let now_nanos = self.clock.monotonic_nanos();
        self.pin_snapshot_at(seq_no, now_nanos);
    }

    // ANCHOR[contextra-mvcc:SEQ-016]
    /// Pinnt eine Sequenznummer zu einem expliziten monotonen Nanosekunden-Zeitstempel `at_nanos`.
    ///
    /// Ermöglicht deterministische Lease-Verwaltung und Zeittests.
    pub fn pin_snapshot_at(&mut self, seq_no: u64, at_nanos: u64) {
        let seq_no = seq_no & !crate::types::TOMBSTONE_BIT;
        self.pinned_snapshots
            .entry(seq_no)
            .or_default()
            .push(at_nanos);
    }

    // ANCHOR[contextra-mvcc:SEQ-017]
    /// Gibt einen zuvor gepinnten Snapshot wieder frei.
    ///
    /// Existierten mehrere Pins für dieselbe Sequenznummer (z. B. verschachtelte Transaktionen),
    /// wird der jüngste Pin entfernt. Erst wenn keine Pins mehr vorhanden sind, wird der Eintrag gelöscht.
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

    // ANCHOR[contextra-mvcc:SEQ-018]
    /// Liefert alle gepinnten Sequenznummern zurück, deren aktive Haltedauer bezogen auf
    /// `now_nanos` die konfigurierte Grenze `max_pin_duration` überschreitet.
    ///
    /// Die Liste wird sortiert zurückgegeben.
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

    // ANCHOR[contextra-mvcc:SEQ-019]
    /// Ermittelt alle gepinnten Sequenznummern, deren Haltedauer die Maximallaufzeit überschreitet,
    /// basierend auf der aktuellen Uhrzeit des `Clock`-Ports.
    pub fn expired_pins(&self) -> Vec<u64> {
        self.expired_pins_at(self.clock.monotonic_nanos())
    }

    // ANCHOR[contextra-mvcc:SEQ-020]
    /// Entfernt alle abgelaufenen Snapshot-Pins bezogen auf den Nanosekunden-Zeitstempel `now_nanos`
    /// und gibt deren Sequenznummern zurück.
    pub fn reap_expired_pins_at(&mut self, now_nanos: u64) -> Vec<u64> {
        let expired = self.expired_pins_at(now_nanos);
        for &seq in &expired {
            self.unpin_snapshot(seq);
        }
        expired
    }

    // ANCHOR[contextra-mvcc:SEQ-021]
    /// Entfernt alle abgelaufenen Snapshot-Pins und gibt die betroffenen Sequenznummern zurück.
    pub fn reap_expired_pins(&mut self) -> Vec<u64> {
        let now_nanos = self.clock.monotonic_nanos();
        self.reap_expired_pins_at(now_nanos)
    }

    // ANCHOR[contextra-mvcc:SEQ-022]
    /// Berechnet die kleinste aktuell gepinnte Sequenznummer bezogen auf `now_nanos`.
    ///
    /// Liefert `None`, wenn derzeit keine Snapshots gepinnt sind.
    /// Gibt eine Tracing-Warnung aus, falls ein Pin die Maximallaufzeit überschritten hat.
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

    // ANCHOR[contextra-mvcc:SEQ-023]
    /// Berechnet die minimale Retentions-Sequenznummer zur aktuellen Zeit des `Clock`-Ports.
    pub fn min_retention_seq(&self) -> Option<u64> {
        self.min_retention_seq_at(self.clock.monotonic_nanos())
    }

    // ANCHOR[contextra-mvcc:SEQ-024]
    /// Liefert die Löschsequenznummer eines Dokuments, falls es aktuell soft-deleted ist.
    pub fn deletion_seq(&self, doc_id: DocId) -> Option<u64> {
        self.deletions.get(&doc_id).copied()
    }

    // ANCHOR[contextra-mvcc:SEQ-025]
    /// Protokolliert das Einfügen eines Dokuments zur Sequenznummer `seq`.
    ///
    /// # Ablauf
    /// 1. Maskiere das Tombstone-Bit aus `seq`.
    /// 2. Falls für diese Dokument-ID ein aktiver Eintrag ohne Löschsequenz existierte,
    ///    wird dessen `delete_seq` auf `seq` gesetzt (Überschreiben/Update-Semantik).
    /// 3. Entferne das Dokument aus der Tabelle der aktiven Löschungen (`deletions`).
    /// 4. Hänge den neuen Versionseintrag mit `delete_seq = None` an das Log an.
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

    // ANCHOR[contextra-mvcc:SEQ-026]
    /// Protokolliert das Löschen eines Dokuments zur Sequenznummer `seq`.
    ///
    /// # Ablauf
    /// 1. Maskiere das Tombstone-Bit aus `seq`.
    /// 2. Suche rückwärts nach dem jüngsten offenen Eintrag dieses Dokuments (`delete_seq.is_none()`).
    /// 3. Setze dessen `delete_seq` auf `seq` und hinterlege die Löschung in `deletions`.
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

    // ANCHOR[contextra-mvcc:SEQ-027]
    /// Prüft, ob ein Dokument zu einer bestimmten Sequenznummer `seq_no` im Index sichtbar ist.
    ///
    /// Gibt `true` zurück, falls das Dokument zu oder vor `seq_no` eingefügt und nicht
    /// zu oder vor `seq_no` gelöscht wurde.
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

    // ANCHOR[contextra-mvcc:SEQ-028]
    /// Kompaktiert das Log: Entfernt Einträge, deren Löschsequenznummer strikt kleiner als `min_active_seqno` ist.
    ///
    /// Einträge, die vor der minimal aktiven Snapshot-Sequenznummer gelöscht wurden,
    /// können von keinem lebenden Snapshot mehr gesehen werden und werden freigegeben.
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

    // ANCHOR[contextra-mvcc:SEQ-029]
    /// Liefert die Gesamtzahl der gespeicherten Log-Einträge.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    // ANCHOR[contextra-mvcc:SEQ-030]
    /// Prüft, ob das Sequenzlog leer ist.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    // ANCHOR[contextra-mvcc:SEQ-031]
    /// Liefert alle Änderungen zurück, die strikt nach `snapshot_seq` stattfanden.
    ///
    /// Das Ergebnis ist chronologisch aufsteigend nach der Sequenznummer sortiert.
    /// Dient der inkrementellen Nachführung von Indizes (Delta Replay).
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

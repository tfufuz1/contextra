//! # SSI — Serializable Snapshot Isolation & Konflikt-Validierung
//!
//! **Ring:** Ring 0 (Transaktionsisolation & Speicherkern)
//! **Klasse:** Kern `[K]`
//!
//! ## Zweck & Architektur-Rolle
//!
//! Dieses Modul implementiert Serializable Snapshot Isolation (SSI) zur Verhinderung von
//! Anomalien wie *Write-Skew* und *Phantoms*, die unter reinem Snapshot Isolation (SI) auftreten können.
//!
//! ### Funktionsweise (Write-Skew-Erkennung)
//! 1. Während einer Transaktion werden alle gelesenen Schlüssel und Suchpräfixe im [`ReadSet`]
//!    zusammen mit der jeweiligen Snapshot-Sequenznummer registriert.
//! 2. Beim Commit-Versuch prüft der [`SsiValidator`] (bzw. [`SequenceLogSsiValidator`]), ob
//!    eine andere Transaktion seit dem Snapshot-Zeitpunkt Schreibzugriffe auf diese Schlüssel
//!    oder übereinstimmende Präfixe erfolgreich committet hat.
//! 3. Falls ja, wird der Commit atomar abgebrochen (`Err(ContextraError::Conflict)`).
//!
//! ### Coarsening & Fail-Closed Schutz (B-16)
//! Um bei hoher Last Speichererschöpfung zu verhindern, fasst der Validator ab 80 % Kapazität
//! ältere Schlüssel in gemeinsame Präfix-Buckets zusammen (*Coarsening*). Dies garantiert
//! **0 % False Negatives** (echte Konflikte werden garantiert erkannt), wobei konservativ
//! False Positives in Kauf genommen werden (Fail-Closed).

// FILE-CONTEXT
// STAND: 2026-09-28T00:00:00Z
// ZWECK: In-memory SSI Read-Set Tracking, Write-Skew-Konfliktvalidierung, Fail-Closed Pruning und atomare Commit-Registrierung.
// INVARIANTEN: INV-MVCC-SSI-1: Konfliktauflösung ist rein datengetrieben (Sequenznummernvergleich) und deterministisch.
// HOTSPOTS: 35-220
// NICHT-OFFENSICHTLICH: Pruning unter `committed_writes` Schreib-Lock sichert den atomaren Wasserstand. Fail-closed Protection verweigert Reads mit snapshot_seq < pruned_through.
// SIEHE AUCH: crates/contextra-mvcc/src/seq_log.rs, crates/contextra-mvcc/src/tx_buffer.rs

use crate::error::{ContextraError, Result};
use crate::seq_log::{SequenceLog, DEFAULT_MAX_PIN_DURATION};
use crate::snapshot::SnapshotRegistry;
use crate::types::{DocId, TxId};
use ahash::AHashMap;
use contextra_ports::{Clock, SystemClock};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::hash::Hasher;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

// ANCHOR[contextra-mvcc:SSI-001]
/// Standard-Obergrenze für nachverfolgte Commit-Schlüssel im [`SequenceLogSsiValidator`] (1.000.000 Schlüssel).
pub const DEFAULT_MAX_TRACKED_COMMIT_KEYS: usize = 1_000_000;

// ANCHOR[contextra-mvcc:SSI-002]
/// Erfasst Lesezugriffe (Einzelschlüssel und Scan-Präfixe) und deren Snapshot-Sequenznummern.
///
/// Dient als Nachweis der Lese-Historie einer Transaktion zur Write-Skew- und Phantomerkennung.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReadSet {
    /// Einzeln gelesene Schlüssel und ihre Snapshot-Sequenznummer.
    keys: AHashMap<Vec<u8>, u64>,
    /// Gelesene Präfixe (Bereichsabfragen) und ihre Snapshot-Sequenznummer.
    prefixes: AHashMap<Vec<u8>, u64>,
    /// Minimaler Snapshot-Sequenzwert über alle registrierten Leseoperationen.
    min_snapshot_seq: Option<u64>,
}

impl ReadSet {
    // ANCHOR[contextra-mvcc:SSI-003]
    /// Erstellt ein neues, leeres [`ReadSet`].
    pub fn new() -> Self {
        Self {
            keys: AHashMap::default(),
            prefixes: AHashMap::default(),
            min_snapshot_seq: None,
        }
    }

    // ANCHOR[contextra-mvcc:SSI-004]
    /// Registriert das Lesen eines Einzelschlüssels zur Snapshot-Sequenznummer `snapshot_seq`.
    ///
    /// Wurde der Schlüssel zuvor in derselben Transaktion gelesen, bleibt die frühere (kleinere)
    /// Sequenznummer erhalten, um strikte Isolationsgrenzen zu wahren.
    pub fn record_read(&mut self, key: impl Into<Vec<u8>>, snapshot_seq: u64) {
        let snapshot_seq = snapshot_seq & !crate::types::TOMBSTONE_BIT;
        let k = key.into();
        self.keys
            .entry(k)
            .and_modify(|existing_seq| {
                if snapshot_seq < *existing_seq {
                    *existing_seq = snapshot_seq;
                }
            })
            .or_insert(snapshot_seq);

        self.min_snapshot_seq = Some(
            self.min_snapshot_seq
                .map_or(snapshot_seq, |curr| curr.min(snapshot_seq)),
        );
    }

    // ANCHOR[contextra-mvcc:SSI-005]
    /// Registriert einen Präfix-Scan zur Snapshot-Sequenznummer `snapshot_seq`.
    ///
    /// Notwendig für Phantomschutz: Verhindert, dass eine parallele Transaktion
    /// neue Schlüssel einfügt, die in dieses Suchpräfix fallen würden.
    pub fn record_prefix(&mut self, prefix: impl Into<Vec<u8>>, snapshot_seq: u64) {
        let snapshot_seq = snapshot_seq & !crate::types::TOMBSTONE_BIT;
        let p = prefix.into();
        self.prefixes
            .entry(p)
            .and_modify(|existing_seq| {
                if snapshot_seq < *existing_seq {
                    *existing_seq = snapshot_seq;
                }
            })
            .or_insert(snapshot_seq);

        self.min_snapshot_seq = Some(
            self.min_snapshot_seq
                .map_or(snapshot_seq, |curr| curr.min(snapshot_seq)),
        );
    }

    // ANCHOR[contextra-mvcc:SSI-006]
    /// Alias für [`ReadSet::record_read`].
    pub fn register_read(&mut self, key: impl Into<Vec<u8>>, snapshot_seq: u64) {
        self.record_read(key, snapshot_seq);
    }

    // ANCHOR[contextra-mvcc:SSI-007]
    /// Liefert die Snapshot-Sequenznummer, zu der `key` gelesen wurde.
    pub fn get(&self, key: &[u8]) -> Option<u64> {
        self.keys.get(key).copied()
    }

    // ANCHOR[contextra-mvcc:SSI-008]
    /// Gibt `true` zurück, wenn weder Schlüssel noch Präfixe erfasst sind.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty() && self.prefixes.is_empty()
    }

    // ANCHOR[contextra-mvcc:SSI-009]
    /// Gibt die Gesamtanzahl der individuell erfassten Leseschlüssel und Bereichspräfixe zurück.
    pub fn len(&self) -> usize {
        self.keys.len() + self.prefixes.len()
    }

    // ANCHOR[contextra-mvcc:SSI-010]
    /// Leert alle erfassten Schlüssel und Präfixe und setzt das Sequenzminimum zurück.
    pub fn clear(&mut self) {
        self.keys.clear();
        self.prefixes.clear();
        self.min_snapshot_seq = None;
    }

    // ANCHOR[contextra-mvcc:SSI-011]
    /// Liefert die kleinste Snapshot-Sequenznummer über alle registrierten Leseoperationen.
    pub fn min_snapshot_seq(&self) -> Option<u64> {
        self.min_snapshot_seq
    }

    // ANCHOR[contextra-mvcc:SSI-012]
    /// Liefert einen Iterator über alle gelesenen Einzelschlüssel und deren Sequenznummer.
    pub fn iter(&self) -> impl Iterator<Item = (&Vec<u8>, &u64)> {
        self.keys.iter()
    }

    // ANCHOR[contextra-mvcc:SSI-013]
    /// Liefert einen Iterator über alle Bereichspräfixe und deren Sequenznummer.
    pub fn prefixes_iter(&self) -> impl Iterator<Item = (&Vec<u8>, &u64)> {
        self.prefixes.iter()
    }

    // ANCHOR[contextra-mvcc:SSI-014]
    /// Liefert eine Referenz auf die interne Schlüssel-Map.
    pub fn keys_map(&self) -> &AHashMap<Vec<u8>, u64> {
        &self.keys
    }

    // ANCHOR[contextra-mvcc:SSI-015]
    /// Liefert eine Referenz auf die interne Präfix-Map.
    pub fn prefixes_map(&self) -> &AHashMap<Vec<u8>, u64> {
        &self.prefixes
    }
}

// ANCHOR[contextra-mvcc:SSI-016]
/// Trait für Serializable Snapshot Isolation (SSI) Validierungs-Komponenten.
///
/// Prüft, ob konkurrierende Schreibtransaktionen Einträge aus dem [`ReadSet`]
/// verändert haben, nachdem der Snapshot der Transaktion erstellt wurde.
pub trait SsiValidator {
    /// Validiert das [`ReadSet`] einer Transaktion `tx_id`.
    ///
    /// # Fehlerfälle
    /// Liefert `Err(ContextraError::Conflict(...))`, falls ein Schlüssel modifiziert wurde
    /// oder der Snapshot älter als der Pruning-Wasserstand ist.
    fn validate(&self, tx_id: TxId, read_set: &ReadSet) -> Result<()>;
}

// ANCHOR[contextra-mvcc:SSI-017]
/// Sequenz-Bucket im Validator: Hält entweder exakte Schlüssel oder vergröberte Präfixe.
#[derive(Debug, Clone, PartialEq, Eq)]
enum SeqBucket {
    Exact(Vec<Vec<u8>>),
    Coarsened {
        prefixes: Vec<Vec<u8>>,
        original_key_count: usize,
    },
}

// ANCHOR[contextra-mvcc:SSI-018]
/// Diagnose-Informationen über blockierte Pruning-Grenzen im SSI-Validator.
///
/// Ermöglicht dem Systembetreiber festzustellen, welche langlebigen Transaktionen
/// oder Snapshots die Bereinigung alter Commit-Schlüssel im Speicher blockieren.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PruningBlockerInfo {
    /// Niedrigste unbereinigte Sequenznummer im Register.
    pub min_unpruned_seq: u64,
    /// Aktueller Pruning-Wasserstand (`pruned_through`).
    pub pruned_through_seq: u64,
    /// Anzahl aktuell gehaltener Commit-Schlüssel.
    pub tracked_commit_keys: usize,
    /// Gesamtanzahl der Sequenz-Buckets.
    pub total_seq_buckets: usize,
    /// Anzahl der vergröberten (coarsened) Buckets.
    pub coarsened_seq_buckets: usize,
    /// Konfigurierte Maximalkapazität an Schlüsseln.
    pub max_tracked_keys: usize,
    /// Sequenznummer des am längsten offenen Snapshots (falls vorhanden).
    pub longest_active_snapshot_seq: Option<u64>,
    /// Haltedauer des ältesten Snapshots (falls vorhanden).
    pub longest_pin_duration: Option<Duration>,
    /// Gibt an, ob der älteste Pin die Maximallaufzeit überschritten hat.
    pub is_pin_expired: bool,
}

// ANCHOR[contextra-mvcc:SSI-019]
/// Fasst exakte Commit-Schlüssellisten durch LCP-Extraktion (Longest Common Prefix) zusammen.
///
/// # Architektur-Entscheidung & Invariante (B-16 / INV-MVCC-SSI-1)
/// Um einen Totalausfall des Transaktionssystems bei Erreichen der Schlüsselobergrenze zu verhindern,
/// werden ältere Buckets in gemeinsame Präfixe vergröbert.
///
/// ## Soundness-Garantie (0 % False Negatives)
/// Vergröberung garantiert, dass kein echter Write-Skew-Konflikt übersehen wird:
/// - Jeder Originalschlüssel $k$ beginnt mit mindestens einem extrahierten Präfix $p$.
/// - Jeder Lesezugriff, der mit $k$ kollidieren würde, überlappt garantiert mit $p$.
/// - Konservative False Positives sind möglich, False Negatives sind mathematisch ausgeschlossen.
fn coarsen_keys_to_prefixes(keys: &[Vec<u8>], max_prefixes: usize) -> Vec<Vec<u8>> {
    if keys.is_empty() {
        return Vec::new();
    }
    if keys.len() <= max_prefixes {
        return keys.to_vec();
    }

    let mut sorted_keys = keys.to_vec();
    sorted_keys.sort_unstable();

    let chunk_size = sorted_keys.len().div_ceil(max_prefixes);
    let mut prefixes = Vec::with_capacity(max_prefixes);

    for chunk in sorted_keys.chunks(chunk_size) {
        if chunk.is_empty() {
            continue;
        }
        let mut lcp = if let Some(first) = chunk.first() {
            first.clone()
        } else {
            continue;
        };
        if let Some(rest) = chunk.get(1..) {
            for k in rest {
                let common_len = lcp.iter().zip(k.iter()).take_while(|(a, b)| a == b).count();
                lcp.truncate(common_len);
                if lcp.is_empty() {
                    break;
                }
            }
        }
        if !prefixes.contains(&lcp) {
            prefixes.push(lcp);
        }
    }

    prefixes
}

// ANCHOR[contextra-mvcc:SSI-020]
/// In-Memory-Commit-Verzeichnis zur Verwaltung committeter Schreibschlüssel und Sequenz-Buckets.
#[derive(Debug, Default)]
struct CommittedWrites {
    keys: AHashMap<Vec<u8>, u64>,
    seq_index: BTreeMap<u64, SeqBucket>,
}

impl CommittedWrites {
    // ANCHOR[contextra-mvcc:SSI-021]
    /// Liefert die Anzahl der exakt vorgehaltenen Einzelschlüssel plus vergröberte Präfixe.
    fn tracked_count(&self) -> usize {
        self.keys.len() + self.coarsened_prefix_count()
    }

    // ANCHOR[contextra-mvcc:SSI-022]
    /// Liefert die Gesamtanzahl der extrahierten Präfixe über alle vergröberten Buckets.
    fn coarsened_prefix_count(&self) -> usize {
        let mut count = 0;
        for bucket in self.seq_index.values() {
            if let SeqBucket::Coarsened { prefixes, .. } = bucket {
                count += prefixes.len();
            }
        }
        count
    }

    // ANCHOR[contextra-mvcc:SSI-023]
    /// Trägt einen committeten Schlüssel zur Sequenznummer `commit_seq` ein.
    fn insert(&mut self, key: Vec<u8>, commit_seq: u64) {
        let commit_seq = commit_seq & !crate::types::TOMBSTONE_BIT;
        self.keys.insert(key.clone(), commit_seq);

        match self.seq_index.entry(commit_seq) {
            std::collections::btree_map::Entry::Vacant(e) => {
                e.insert(SeqBucket::Exact(vec![key]));
            }
            std::collections::btree_map::Entry::Occupied(mut e) => match e.get_mut() {
                SeqBucket::Exact(keys) => {
                    if !keys.contains(&key) {
                        keys.push(key);
                    }
                }
                SeqBucket::Coarsened {
                    original_key_count, ..
                } => {
                    *original_key_count += 1;
                }
            },
        }
    }

    // ANCHOR[contextra-mvcc:SSI-024]
    /// Vergröbert exakte Buckets in Präfixe, um Speicher freizugeben.
    fn coarsen_oldest_buckets(&mut self) -> usize {
        let mut coarsened_count = 0;
        let seqs: Vec<u64> = self.seq_index.keys().copied().collect();

        for seq in seqs {
            if let Some(SeqBucket::Exact(keys)) = self.seq_index.get(&seq) {
                let original_keys = keys.clone();
                let prefixes = coarsen_keys_to_prefixes(&original_keys, 2);

                for k in &original_keys {
                    if self.keys.get(k) == Some(&seq) {
                        self.keys.remove(k);
                    }
                }

                self.seq_index.insert(
                    seq,
                    SeqBucket::Coarsened {
                        prefixes,
                        original_key_count: original_keys.len(),
                    },
                );
                coarsened_count += 1;
            }
        }

        coarsened_count
    }

    // ANCHOR[contextra-mvcc:SSI-025]
    /// Führt benachbarte vergröberte Buckets zusammen, um Bucket-Overhead zu reduzieren.
    fn merge_adjacent_buckets(&mut self) -> usize {
        if self.seq_index.len() < 2 {
            return 0;
        }

        let seqs: Vec<u64> = self.seq_index.keys().copied().collect();
        let mut merged_count = 0;

        for chunk in seqs.chunks(2) {
            if chunk.len() == 2 {
                if let (Some(&s1), Some(&s2)) = (chunk.first(), chunk.get(1)) {
                    if let (Some(b1), Some(b2)) =
                        (self.seq_index.remove(&s1), self.seq_index.remove(&s2))
                    {
                        let mut prefixes = Vec::new();
                        let mut total_keys = 0;

                        for (b, seq) in [(b1, s1), (b2, s2)] {
                            match b {
                                SeqBucket::Exact(keys) => {
                                    for k in &keys {
                                        if self.keys.get(k) == Some(&seq) {
                                            self.keys.remove(k);
                                        }
                                    }
                                    total_keys += keys.len();
                                    prefixes.extend(coarsen_keys_to_prefixes(&keys, 2));
                                }
                                SeqBucket::Coarsened {
                                    prefixes: p,
                                    original_key_count,
                                } => {
                                    total_keys += original_key_count;
                                    prefixes.extend(p);
                                }
                            }
                        }

                        let coarsened_prefixes = coarsen_keys_to_prefixes(&prefixes, 2);
                        self.seq_index.insert(
                            s2,
                            SeqBucket::Coarsened {
                                prefixes: coarsened_prefixes,
                                original_key_count: total_keys,
                            },
                        );
                        merged_count += 1;
                    }
                }
            }
        }

        merged_count
    }

    // ANCHOR[contextra-mvcc:SSI-026]
    /// Entfernt alle Schlüssel ab Sequenznummer `first_seq` (z. B. nach fehlgeschlagenen Batch-Commits).
    fn remove_from_seq(&mut self, first_seq: u64) -> usize {
        let first_seq = first_seq & !crate::types::TOMBSTONE_BIT;
        let mut removed = 0;
        let mut seqs_to_remove = Vec::new();

        for (&seq, bucket) in self.seq_index.range(first_seq..) {
            seqs_to_remove.push(seq);
            match bucket {
                SeqBucket::Exact(keys) => {
                    for key in keys {
                        if self.keys.get(key) == Some(&seq) {
                            let mut highest_old_seq = None;
                            for (&old_seq, old_bucket) in self.seq_index.range(..first_seq).rev() {
                                if let SeqBucket::Exact(old_keys) = old_bucket {
                                    if old_keys.contains(key) {
                                        highest_old_seq = Some(old_seq);
                                        break;
                                    }
                                }
                            }
                            if let Some(old_seq) = highest_old_seq {
                                self.keys.insert(key.clone(), old_seq);
                            } else {
                                self.keys.remove(key);
                            }
                            removed += 1;
                        }
                    }
                }
                SeqBucket::Coarsened {
                    original_key_count, ..
                } => {
                    removed += *original_key_count;
                }
            }
        }

        for seq in seqs_to_remove {
            self.seq_index.remove(&seq);
        }

        removed
    }

    // ANCHOR[contextra-mvcc:SSI-027]
    /// Bereinigt (prunt) Einträge mit `commit_seq <= bound_seq`.
    fn prune_through(&mut self, bound_seq: u64) -> usize {
        let bound_seq = bound_seq & !crate::types::TOMBSTONE_BIT;
        let mut removed = 0;
        let mut seqs_to_remove = Vec::new();

        for (&seq, bucket) in self.seq_index.range(..=bound_seq) {
            seqs_to_remove.push(seq);
            match bucket {
                SeqBucket::Exact(keys) => {
                    for key in keys {
                        if self.keys.get(key) == Some(&seq) && self.keys.remove(key).is_some() {
                            removed += 1;
                        }
                    }
                }
                SeqBucket::Coarsened {
                    original_key_count, ..
                } => {
                    removed += *original_key_count;
                }
            }
        }

        for seq in seqs_to_remove {
            self.seq_index.remove(&seq);
        }

        removed
    }

    // ANCHOR[contextra-mvcc:SSI-028]
    /// Berechnet die effektive Belegung (exakte Schlüssel + vergröberte Präfixe).
    fn len(&self) -> usize {
        self.tracked_count()
    }

    // ANCHOR[contextra-mvcc:SSI-029]
    /// Liefert die Commit-Sequenznummer für einen exakten Schlüssel.
    fn get(&self, key: &[u8]) -> Option<u64> {
        self.keys.get(key).copied()
    }
}

// ANCHOR[contextra-mvcc:SSI-030]
/// Referenz-Implementierung des [`SsiValidator`] basierend auf In-Memory-Commit-Tracking und optionalem [`SequenceLog`].
///
/// Arbeitet rein im Speicher ohne Festplatten-I/O oder externe Abhängigkeiten.
///
/// # Invariante (INV-MVCC-SSI-1)
/// Die Konflikterkennung ist rein datengetrieben und deterministisch über Sequenznummernvergleiche.
#[derive(Clone)]
pub struct SequenceLogSsiValidator {
    /// Durch `RwLock` geschützte Tabelle committeter Schreibzugriffe.
    committed_writes: Arc<RwLock<CommittedWrites>>,
    /// Optionales Sequenzlog für Dokumentänderungs-Prüfungen.
    sequence_log: Option<Arc<RwLock<SequenceLog>>>,
    /// Optionale Snapshot-Registry für Langlebigkeits-Diagnosen.
    snapshot_registry: Option<Arc<SnapshotRegistry>>,
    /// Atomarer Wasserstand für bereits bereinigte Sequenznummern (`pruned_through`).
    pruned_through: Arc<AtomicU64>,
    /// Obergrenze für nachverfolgte Schlüssel im Arbeitsspeicher.
    max_tracked_keys: usize,
    /// Maximale Haltezeit für Snapshots vor Alarmierung.
    max_pin_duration: Duration,
    /// Optionaler Metrik-Port für Observability (Monitoring).
    metrics_sink: Option<Arc<parking_lot::RwLock<Arc<dyn contextra_ports::MetricsSink>>>>,
    /// Deterministischer Zeit-Port.
    clock: Arc<dyn Clock>,
}

impl std::fmt::Debug for SequenceLogSsiValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SequenceLogSsiValidator")
            .field("committed_writes", &self.committed_writes)
            .field("sequence_log", &self.sequence_log)
            .field("snapshot_registry", &self.snapshot_registry)
            .field("pruned_through", &self.pruned_through)
            .field("max_tracked_keys", &self.max_tracked_keys)
            .field("max_pin_duration", &self.max_pin_duration)
            .field("has_metrics_sink", &self.metrics_sink.is_some())
            .finish()
    }
}

impl Default for SequenceLogSsiValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl SequenceLogSsiValidator {
    // ANCHOR[contextra-mvcc:SSI-031]
    /// Erstellt einen neuen [`SequenceLogSsiValidator`] mit Standard-Kapazität (1.000.000 Schlüssel) und [`SystemClock`].
    pub fn new() -> Self {
        Self::new_with_bounds(DEFAULT_MAX_TRACKED_COMMIT_KEYS)
    }

    // ANCHOR[contextra-mvcc:SSI-032]
    /// Erstellt einen neuen Validator mit benutzerdefinierter Kapazitätsobergrenze `max_tracked_keys`.
    pub fn new_with_bounds(max_tracked_keys: usize) -> Self {
        Self::new_with_clock_and_bounds(Arc::new(SystemClock::new()), max_tracked_keys)
    }

    // ANCHOR[contextra-mvcc:SSI-033]
    /// Erstellt einen neuen Validator mit benutzerdefiniertem [`Clock`]-Port und Kapazitätsgrenze.
    pub fn new_with_clock_and_bounds(clock: Arc<dyn Clock>, max_tracked_keys: usize) -> Self {
        Self {
            committed_writes: Arc::new(RwLock::new(CommittedWrites::default())),
            sequence_log: None,
            snapshot_registry: None,
            pruned_through: Arc::new(AtomicU64::new(0)),
            max_tracked_keys,
            max_pin_duration: DEFAULT_MAX_PIN_DURATION,
            metrics_sink: None,
            clock,
        }
    }

    // ANCHOR[contextra-mvcc:SSI-034]
    /// Verbindet eine benutzerdefinierte [`Clock`] mit diesem Validator (Builder-Muster).
    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    // ANCHOR[contextra-mvcc:SSI-035]
    /// Konfiguriert den aktiven [`Clock`]-Port für diesen Validator.
    pub fn set_clock(&mut self, clock: Arc<dyn Clock>) {
        self.clock = clock;
    }

    // ANCHOR[contextra-mvcc:SSI-036]
    /// Liefert den aktiven [`Clock`]-Port dieses Validators.
    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

    // ANCHOR[contextra-mvcc:SSI-037]
    /// Erstellt einen Validator, der mit einem [`SequenceLog`] für Dokumentprüfungen verknüpft ist.
    pub fn with_sequence_log(seq_log: Arc<RwLock<SequenceLog>>) -> Self {
        Self {
            committed_writes: Arc::new(RwLock::new(CommittedWrites::default())),
            sequence_log: Some(seq_log),
            snapshot_registry: None,
            pruned_through: Arc::new(AtomicU64::new(0)),
            max_tracked_keys: DEFAULT_MAX_TRACKED_COMMIT_KEYS,
            max_pin_duration: DEFAULT_MAX_PIN_DURATION,
            metrics_sink: None,
            clock: Arc::new(SystemClock::new()),
        }
    }

    // ANCHOR[contextra-mvcc:SSI-038]
    /// Verknüpft eine [`SnapshotRegistry`] zur Erkennung langlebiger Snapshot-Pins (Builder-Muster).
    pub fn with_snapshot_registry(mut self, registry: Arc<SnapshotRegistry>) -> Self {
        self.snapshot_registry = Some(registry);
        self
    }

    // ANCHOR[contextra-mvcc:SSI-039]
    /// Setzt die verknüpfte [`SnapshotRegistry`].
    pub fn set_snapshot_registry(&mut self, registry: Arc<SnapshotRegistry>) {
        self.snapshot_registry = Some(registry);
    }

    // ANCHOR[contextra-mvcc:SSI-040]
    /// Bindet einen Metrik-Port für Observability an (Builder-Muster).
    pub fn with_metrics_sink(
        mut self,
        sink: Arc<parking_lot::RwLock<Arc<dyn contextra_ports::MetricsSink>>>,
    ) -> Self {
        self.metrics_sink = Some(sink);
        self
    }

    // ANCHOR[contextra-mvcc:SSI-041]
    /// Konfiguriert den Metrik-Port im laufenden Betrieb.
    pub fn set_metrics_sink(
        &mut self,
        sink: Arc<parking_lot::RwLock<Arc<dyn contextra_ports::MetricsSink>>>,
    ) {
        self.metrics_sink = Some(sink);
    }

    // ANCHOR[contextra-mvcc:SSI-042]
    /// Konfiguriert die maximale Pin-Dauer vor Alarmauslösung (Builder-Muster).
    pub fn with_max_pin_duration(mut self, duration: Duration) -> Self {
        self.max_pin_duration = duration;
        self
    }

    // ANCHOR[contextra-mvcc:SSI-043]
    /// Bereinigt (prunt) Einträge mit `commit_seq <= bound_seq` und hebt den Wasserstand atomar an.
    pub fn prune_through(&self, bound_seq: u64) -> usize {
        let bound_seq = bound_seq & !crate::types::TOMBSTONE_BIT;
        let mut writes = self.committed_writes.write();
        let removed = writes.prune_through(bound_seq);
        self.pruned_through.fetch_max(bound_seq, Ordering::SeqCst);
        removed
    }

    // ANCHOR[contextra-mvcc:SSI-044]
    /// Entfernt alle Commit-Einträge ab Sequenznummer `first_seq` (Rollback-Support).
    pub fn forget_from(&self, first_seq: u64) -> usize {
        let first_seq = first_seq & !crate::types::TOMBSTONE_BIT;
        let mut writes = self.committed_writes.write();
        writes.remove_from_seq(first_seq)
    }

    // ANCHOR[contextra-mvcc:SSI-045]
    /// Liefert den aktuellen Pruning-Wasserstand.
    pub fn pruned_through(&self) -> u64 {
        self.pruned_through.load(Ordering::SeqCst)
    }

    // ANCHOR[contextra-mvcc:SSI-046]
    /// Liefert die Anzahl aktuell erfasster Commit-Schlüssel.
    pub fn tracked_commit_keys(&self) -> usize {
        self.committed_writes.read().len()
    }

    // ANCHOR[contextra-mvcc:SSI-047]
    /// Liefert die konfigurierte Obergrenze an Commit-Schlüsseln.
    pub fn max_tracked_keys(&self) -> usize {
        self.max_tracked_keys
    }

    // ANCHOR[contextra-mvcc:SSI-048]
    /// Prüft die Belegung des Commit-Registers und leitet ab 80 % Füllstand Vergröberung (Coarsening) ein.
    fn check_and_coarsen_if_needed(&self, writes: &mut CommittedWrites) {
        let threshold = (self.max_tracked_keys * 80) / 100;
        let initial_len = writes.len();
        let reached_threshold = initial_len >= threshold;

        let mut rounds = 0;
        while writes.len() >= threshold && rounds < 10 {
            let coarsened = writes.coarsen_oldest_buckets();
            let merged = writes.merge_adjacent_buckets();
            rounds += 1;
            if coarsened == 0 && merged == 0 {
                break;
            }
        }

        let now_nanos = self.clock.monotonic_nanos();

        if let Some(ref sink_lock) = self.metrics_sink {
            let sink = sink_lock.read().clone();
            let ratio = initial_len as f64 / self.max_tracked_keys as f64;
            sink.record_gauge("mvcc_ssi_commit_register_utilization_ratio", ratio, &[]);

            if reached_threshold {
                if let Some(ref blocker) = self.diagnose_pruning_blocker_internal(writes, now_nanos)
                {
                    sink.record_gauge(
                        "mvcc_ssi_coarsened_buckets_total",
                        blocker.coarsened_seq_buckets as f64,
                        &[],
                    );
                    if let Some(dur) = blocker.longest_pin_duration {
                        sink.record_gauge(
                            "mvcc_ssi_longest_pin_duration_seconds",
                            dur.as_secs_f64(),
                            &[],
                        );
                    }
                }
            }
        }

        if writes.len() >= threshold {
            if let Some(blocker) = self.diagnose_pruning_blocker_internal(writes, now_nanos) {
                tracing::warn!(
                    min_unpruned_seq = blocker.min_unpruned_seq,
                    pruned_through_seq = blocker.pruned_through_seq,
                    tracked_commit_keys = blocker.tracked_commit_keys,
                    total_seq_buckets = blocker.total_seq_buckets,
                    coarsened_seq_buckets = blocker.coarsened_seq_buckets,
                    max_tracked_keys = blocker.max_tracked_keys,
                    longest_active_snapshot_seq = ?blocker.longest_active_snapshot_seq,
                    longest_pin_duration = ?blocker.longest_pin_duration,
                    is_pin_expired = blocker.is_pin_expired,
                    "SequenceLogSsiValidator capacity utilization reached threshold (>= 80%). Coarsening active. Pruning may be blocked by active snapshot or unpruned sequence."
                );
            }
        }
    }

    // ANCHOR[contextra-mvcc:SSI-049]
    /// Liefert Diagnoseinformationen darüber, welcher Snapshot oder welche Sequenz das Pruning blockiert.
    pub fn diagnose_pruning_blocker(&self) -> Option<PruningBlockerInfo> {
        self.diagnose_pruning_blocker_at(self.clock.monotonic_nanos())
    }

    // ANCHOR[contextra-mvcc:SSI-050]
    /// Diagnostiziert Pruning-Blocker bezogen auf den Zeitstempel `now_nanos`.
    pub fn diagnose_pruning_blocker_at(&self, now_nanos: u64) -> Option<PruningBlockerInfo> {
        let writes = self.committed_writes.read();
        self.diagnose_pruning_blocker_internal(&writes, now_nanos)
    }

    // ANCHOR[contextra-mvcc:SSI-051]
    /// Interne Diagnose-Auswertung unter gehaltener Lesesperre auf `committed_writes`.
    fn diagnose_pruning_blocker_internal(
        &self,
        writes: &CommittedWrites,
        now_nanos: u64,
    ) -> Option<PruningBlockerInfo> {
        let min_unpruned_seq = writes.seq_index.keys().copied().next()?;
        let total_seq_buckets = writes.seq_index.len();
        let coarsened_seq_buckets = writes
            .seq_index
            .values()
            .filter(|b| matches!(b, SeqBucket::Coarsened { .. }))
            .count();

        let longest_pin = self
            .snapshot_registry
            .as_ref()
            .and_then(|reg| reg.longest_active_pin_at(now_nanos));

        let longest_active_snapshot_seq = longest_pin.map(|(seq, _)| seq);
        let longest_pin_duration = longest_pin.map(|(_, dur)| dur);
        let is_pin_expired = longest_pin.is_some_and(|(_, dur)| dur > self.max_pin_duration);

        Some(PruningBlockerInfo {
            min_unpruned_seq,
            pruned_through_seq: self.pruned_through(),
            tracked_commit_keys: writes.tracked_count(),
            total_seq_buckets,
            coarsened_seq_buckets,
            max_tracked_keys: self.max_tracked_keys,
            longest_active_snapshot_seq,
            longest_pin_duration,
            is_pin_expired,
        })
    }

    // ANCHOR[contextra-mvcc:SSI-052]
    /// Registriert einen committeten Schreibschlüssel zur Sequenznummer `commit_seq`.
    pub fn record_commit_key(&self, key: &[u8], commit_seq: u64) {
        let commit_seq = commit_seq & !crate::types::TOMBSTONE_BIT;
        let mut writes = self.committed_writes.write();
        writes.insert(key.to_vec(), commit_seq);
        self.check_and_coarsen_if_needed(&mut writes);
    }

    // ANCHOR[contextra-mvcc:SSI-053]
    /// Registriert mehrere committete Schreibschlüssel als atomare Batch-Operation.
    pub fn record_commit_keys<'a>(
        &self,
        keys: impl IntoIterator<Item = &'a [u8]>,
        commit_seq: u64,
    ) {
        let commit_seq = commit_seq & !crate::types::TOMBSTONE_BIT;
        let mut writes = self.committed_writes.write();
        for key in keys {
            writes.insert(key.to_vec(), commit_seq);
        }
        self.check_and_coarsen_if_needed(&mut writes);
    }

    // ANCHOR[contextra-mvcc:SSI-054]
    /// Validiert Punkt-Leseoperationen gegen committete Schreibschlüssel und vergröberte Buckets.
    ///
    /// # Ablauf
    /// 1. Fail-Closed Check: Ist `snapshot_seq < pruned_through`, wurde die Historie bereits bereinigt -> Konflikt.
    /// 2. Exakter Treffer: Wurde der Schlüssel committet und ist `commit_seq > snapshot_seq` -> Konflikt.
    /// 3. Vergröberter Treffer: Beginnt der Schlüssel mit einem vergröberten Präfix und `commit_seq > snapshot_seq` -> Konflikt.
    /// 4. Prüfung gegen optionales `SequenceLog`.
    fn validate_point_reads(
        &self,
        committed_writes: &CommittedWrites,
        tx_id: TxId,
        entries: &[(&Vec<u8>, &u64)],
        pruned_through: u64,
    ) -> Result<()> {
        for &(key, &snapshot_seq) in entries {
            if snapshot_seq < pruned_through {
                return Err(ContextraError::Conflict(format!(
                    "Serializable isolation violation for TxId({}): snapshot too old (snapshot_seq {} < pruned_through {}) for key '{:?}'",
                    tx_id.inner(),
                    snapshot_seq,
                    pruned_through,
                    String::from_utf8_lossy(key)
                )));
            }

            if let Some(commit_seq) = committed_writes.get(key) {
                if commit_seq > snapshot_seq {
                    return Err(ContextraError::Conflict(format!(
                        "Serializable isolation violation for TxId({}): key '{:?}' modified at commit_seq {} > snapshot_seq {}",
                        tx_id.inner(),
                        String::from_utf8_lossy(key),
                        commit_seq,
                        snapshot_seq
                    )));
                }
            }

            for (&commit_seq, bucket) in &committed_writes.seq_index {
                if commit_seq > snapshot_seq {
                    if let SeqBucket::Coarsened { prefixes, .. } = bucket {
                        for p in prefixes {
                            if key.starts_with(p) {
                                return Err(ContextraError::Conflict(format!(
                                    "Serializable isolation violation for TxId({}): key '{:?}' matches coarsened commit prefix '{:?}' at commit_seq {} > snapshot_seq {}",
                                    tx_id.inner(),
                                    String::from_utf8_lossy(key),
                                    String::from_utf8_lossy(p),
                                    commit_seq,
                                    snapshot_seq
                                )));
                            }
                        }
                    }
                }
            }

            self.validate_sequence_log_for_key(tx_id, key, snapshot_seq)?;
        }
        Ok(())
    }

    // ANCHOR[contextra-mvcc:SSI-055]
    /// Validiert Dokumentänderungen im optionalen `SequenceLog` für einen Einzelschlüssel.
    fn validate_sequence_log_for_key(
        &self,
        tx_id: TxId,
        key: &[u8],
        snapshot_seq: u64,
    ) -> Result<()> {
        if let Some(ref seq_log_lock) = self.sequence_log {
            let seq_log = seq_log_lock.read();
            let doc_id = derive_doc_id_for_key(key);

            let changes = seq_log.changes_since(snapshot_seq);
            for change in changes {
                if change.doc_id() == doc_id && change.seq() > snapshot_seq {
                    return Err(ContextraError::Conflict(format!(
                        "Serializable isolation violation for TxId({}): doc_id {} modified at seq {} > snapshot_seq {}",
                        tx_id.inner(),
                        doc_id,
                        change.seq(),
                        snapshot_seq
                    )));
                }
            }
        }
        Ok(())
    }

    // ANCHOR[contextra-mvcc:SSI-056]
    /// Validiert Bereichsscans (Präfixe) gegen committete Schlüssel zur Erkennung von Phantomen.
    ///
    /// # Ablauf
    /// 1. Fail-Closed Check gegen `pruned_through`.
    /// 2. Prüfung aller committeten Schlüssel, die mit dem Scan-Präfix beginnen.
    /// 3. Prüfung überlappender vergröberter Präfix-Buckets.
    fn validate_range_reads(
        &self,
        committed_writes: &CommittedWrites,
        tx_id: TxId,
        prefixes: &[(&Vec<u8>, &u64)],
        pruned_through: u64,
    ) -> Result<()> {
        for &(prefix, &snapshot_seq) in prefixes {
            if snapshot_seq < pruned_through {
                return Err(ContextraError::Conflict(format!(
                    "Serializable isolation violation for TxId({}): snapshot too old (snapshot_seq {} < pruned_through {}) for range prefix '{:?}'",
                    tx_id.inner(),
                    snapshot_seq,
                    pruned_through,
                    String::from_utf8_lossy(prefix)
                )));
            }

            for (committed_key, &commit_seq) in &committed_writes.keys {
                if committed_key.starts_with(prefix) && commit_seq > snapshot_seq {
                    return Err(ContextraError::Conflict(format!(
                        "Serializable isolation phantom violation for TxId({}): key '{:?}' matching range prefix '{:?}' committed at commit_seq {} > snapshot_seq {}",
                        tx_id.inner(),
                        String::from_utf8_lossy(committed_key),
                        String::from_utf8_lossy(prefix),
                        commit_seq,
                        snapshot_seq
                    )));
                }
            }

            for (&commit_seq, bucket) in &committed_writes.seq_index {
                if commit_seq > snapshot_seq {
                    if let SeqBucket::Coarsened {
                        prefixes: c_prefixes,
                        ..
                    } = bucket
                    {
                        for cp in c_prefixes {
                            if cp.starts_with(prefix) || prefix.starts_with(cp) {
                                return Err(ContextraError::Conflict(format!(
                                    "Serializable isolation phantom violation for TxId({}): coarsened prefix '{:?}' overlapping range prefix '{:?}' committed at commit_seq {} > snapshot_seq {}",
                                    tx_id.inner(),
                                    String::from_utf8_lossy(cp),
                                    String::from_utf8_lossy(prefix),
                                    commit_seq,
                                    snapshot_seq
                                )));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    // ANCHOR[contextra-mvcc:SSI-057]
    /// Interne SSI-Validierungslogik für Punkt- und Bereichslesevorgänge.
    ///
    /// Sortiert Schlüssel und Präfixe vor der Validierung deterministisch.
    fn validate_internal(
        &self,
        committed_writes: &CommittedWrites,
        tx_id: TxId,
        read_set: &ReadSet,
    ) -> Result<()> {
        if read_set.is_empty() {
            return Ok(());
        }

        if committed_writes.len() >= self.max_tracked_keys {
            return Err(ContextraError::Conflict(format!(
                "Serializable isolation validation failed for TxId({}): commit key capacity limit reached ({} >= {})",
                tx_id.inner(),
                committed_writes.len(),
                self.max_tracked_keys
            )));
        }

        let pruned_through = self.pruned_through.load(Ordering::SeqCst);

        let mut entries: Vec<(&Vec<u8>, &u64)> = read_set.iter().collect();
        entries.sort_unstable_by(|(k1, _), (k2, _)| k1.cmp(k2));

        self.validate_point_reads(committed_writes, tx_id, &entries, pruned_through)?;

        let mut prefixes: Vec<(&Vec<u8>, &u64)> = read_set.prefixes_iter().collect();
        prefixes.sort_unstable_by(|(p1, _), (p2, _)| p1.cmp(p2));

        self.validate_range_reads(committed_writes, tx_id, &prefixes, pruned_through)?;

        Ok(())
    }

    // ANCHOR[contextra-mvcc:SSI-058]
    /// Atomare Validierung und Registrierung: Validiert das `read_set` und trägt `write_keys`
    /// unter einer einzigen exklusiven Schreibsperre ein.
    ///
    /// Verhindert Lost Updates und Race Conditions zwischen Validierung und Commit-Registrierung.
    pub fn validate_and_record<'a>(
        &self,
        tx_id: TxId,
        read_set: &ReadSet,
        write_keys: impl IntoIterator<Item = &'a [u8]>,
        commit_seq: u64,
    ) -> Result<()> {
        let commit_seq = commit_seq & !crate::types::TOMBSTONE_BIT;
        let mut writes = self.committed_writes.write();
        self.validate_internal(&writes, tx_id, read_set)?;

        for key in write_keys {
            writes.insert(key.to_vec(), commit_seq);
        }

        Ok(())
    }
}

impl SsiValidator for SequenceLogSsiValidator {
    // ANCHOR[contextra-mvcc:SSI-059]
    /// Führt die Validierung unter gehaltener Lesesperre durch.
    fn validate(&self, tx_id: TxId, read_set: &ReadSet) -> Result<()> {
        let committed_writes = self.committed_writes.read();
        self.validate_internal(&committed_writes, tx_id, read_set)
    }
}

// ANCHOR[contextra-mvcc:SSI-060]
/// Leitet deterministisch eine `DocId` aus einem Byte-Schlüssel ab (UTF-8-Parsing oder Fallback-Hash).
fn derive_doc_id_for_key(key: &[u8]) -> DocId {
    if let Ok(s) = std::str::from_utf8(key) {
        if let Ok(doc_id) = DocId::from_key(s) {
            return doc_id;
        }
    }
    let mut hasher = ahash::AHasher::default();
    hasher.write(key);
    DocId::from(hasher.finish())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_read_set_basic() {
        let mut rs = ReadSet::new();
        assert!(rs.is_empty());
        assert_eq!(rs.len(), 0);
        assert_eq!(rs.min_snapshot_seq(), None);

        rs.record_read(b"key1".to_vec(), 10);
        rs.record_read(b"key2".to_vec(), 15);

        assert!(!rs.is_empty());
        assert_eq!(rs.len(), 2);
        assert_eq!(rs.get(b"key1"), Some(10));
        assert_eq!(rs.get(b"key2"), Some(15));
        assert_eq!(rs.get(b"key3"), None);
        assert_eq!(rs.min_snapshot_seq(), Some(10));

        // Lower snapshot sequence replaces higher
        rs.record_read(b"key1".to_vec(), 5);
        assert_eq!(rs.get(b"key1"), Some(5));
        assert_eq!(rs.min_snapshot_seq(), Some(5));

        // Higher snapshot sequence ignored for key1, min remains 5
        rs.record_read(b"key1".to_vec(), 20);
        assert_eq!(rs.get(b"key1"), Some(5));
        assert_eq!(rs.min_snapshot_seq(), Some(5));

        rs.clear();
        assert!(rs.is_empty());
        assert_eq!(rs.min_snapshot_seq(), None);
    }

    #[test]
    fn test_validator_no_conflict() {
        let validator = SequenceLogSsiValidator::new();
        let tx = TxId::new(1);

        let mut rs = ReadSet::new();
        rs.record_read(b"key1".to_vec(), 10);

        // Record a commit for key1 at seq 10 (equal to snapshot_seq)
        validator.record_commit_key(b"key1", 10);

        assert!(validator.validate(tx, &rs).is_ok());
    }

    #[test]
    fn test_validator_conflict_detected() {
        let validator = SequenceLogSsiValidator::new();
        let tx = TxId::new(2);

        let mut rs = ReadSet::new();
        rs.record_read(b"key1".to_vec(), 10);

        // Record a commit for key1 at seq 11 (> snapshot_seq 10)
        validator.record_commit_key(b"key1", 11);

        let res = validator.validate(tx, &rs);
        assert!(res.is_err());
        assert!(matches!(res, Err(ContextraError::Conflict(_))));
    }

    #[test]
    fn test_validator_batch_commit_keys() {
        let validator = SequenceLogSsiValidator::new();
        let tx = TxId::new(5);

        let mut rs = ReadSet::new();
        rs.record_read(b"key_a".to_vec(), 10);
        rs.record_read(b"key_b".to_vec(), 10);

        let keys = vec![b"key_a".as_slice(), b"key_b".as_slice()];
        validator.record_commit_keys(keys, 15);

        let res = validator.validate(tx, &rs);
        assert!(res.is_err());
        assert!(matches!(res, Err(ContextraError::Conflict(_))));
    }

    #[test]
    fn test_validator_with_sequence_log() {
        let seq_log = Arc::new(RwLock::new(SequenceLog::new()));
        let validator = SequenceLogSsiValidator::with_sequence_log(seq_log.clone());
        let tx = TxId::new(3);

        let doc_key = "doc_alpha";
        let doc_id = DocId::from_key(doc_key).expect("valid doc_id"); // #[cfg(test)]

        let mut rs = ReadSet::new();
        rs.record_read(doc_key.as_bytes(), 5);

        // No changes in sequence log -> OK
        assert!(validator.validate(tx, &rs).is_ok());

        // Record insert in sequence log at seq 8 (> snapshot_seq 5)
        seq_log.write().record_insert(doc_id, 8);

        let res = validator.validate(tx, &rs);
        assert!(res.is_err());
        assert!(matches!(res, Err(ContextraError::Conflict(_))));
    }
}

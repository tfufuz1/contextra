// ANCHOR[contextra-mvcc:SSI-ROOT]
//! # Serializable Snapshot Isolation (SSI) — Read-Set Tracking und Konfliktvalidierung
//!
//! Dieses Modul implementiert die serialisierbare Snapshot-Isolation (SSI) im Hauptspeicher.
//! Transaktionen unter Standard-Snapshot-Isolation sind anfällig für bestimmte Anomalien
//! wie *Write-Skew* (z. B. zwei parallele Transaktionen heben gleichzeitig Geld ab, weil beide
//! den gemeinsamen Mindestkontostand zu ihrem Snapshot-Beginn noch erfüllt sehen).
//!
//! SSI erkennt solche Konflikte, indem es während der Transaktion protokolliert, welche
//! Schlüssel gelesen wurden ([`ReadSet`]), und vor dem Commit prüft, ob eine nebenläufige
//! Transaktion diese Schlüssel in der Zwischenzeit verändert hat ([`SsiValidator`]).
//!
//! ## Relevanz im Gesamtsystem (Ring 0)
//! - **Ring 0 (Kern)**: Reine In-Memory-Konflikterkennung ohne Abhängigkeit von Festplattenstrukturen.
//! - **Phantom-Protection**: Schützt nicht nur Einzelpunkt-Lesezugriffe, sondern auch Bereichsleser
//!   (Range Scans) durch Präfix-Tracking vor Phantom-Einfügungen.
//! - **Fail-Closed Pruning**: Werden historische Commit-Informationen aus Speichermangel
//!   bereinigt (geprunt), schlägt die Validierung älterer Transaktionen deterministisch
//!   mit einem Konfliktfehler fehl (Fail-Closed-Sicherheitsarchitektur).
//! - **Vergröberung (Coarsening, B-16)**: Um Speichergrenzen einzuhalten, ohne ältere
//!   Transaktionen sofort abbrechen zu müssen, werden alte Schlüssel zu gemeinsamen
//!   Präfixen aggregiert (0 % False Negatives, konservative Konflikterkennung).

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

// ANCHOR[contextra-mvcc:SSI-CONST]
/// Standard-Höchstgrenze für im Speicher gehaltene bestätigte Schreib-Schlüssel (1.000.000 Schlüssel).
pub const DEFAULT_MAX_TRACKED_COMMIT_KEYS: usize = 1_000_000;

// ANCHOR[contextra-mvcc:SSI-READ-SET]
/// Protokolliert gelesene Einzelschlüssel, Präfixe und deren jeweilige Snapshot-Sequenznummer für SSI.
///
/// ### Speicher- und Risikoanalyse:
/// Der Speicherbedarf pro Transaktion wächst linear $O(R + P)$ mit der Anzahl der gelesenen Schlüssel $R$
/// und Präfixe $P$. Bei massiven Lese-Transaktionen muss die Obergrenze über
/// [`crate::tx_buffer::TxBuffer::try_register_read`] überwacht werden, um Speichererschöpfung zu verhindern.
///
/// ### Phantom-Schutz:
/// Normale Einzelschlüssel-Leser registrieren Keys via [`ReadSet::record_read`]. Bereichsscans (Range Scans)
/// müssen [`ReadSet::record_prefix`] aufrufen, um neu eingefügte Phantome zu erkennen.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReadSet {
    /// Zuordnung von gelesenen Schlüsseln (Bytefolgen) zur zugehörigen Snapshot-Sequenznummer.
    keys: AHashMap<Vec<u8>, u64>,
    /// Zuordnung von Scan-Präfixen zu deren Snapshot-Sequenznummer (Phantom-Schutz).
    prefixes: AHashMap<Vec<u8>, u64>,
    /// Kleinste Snapshot-Sequenznummer aller in diesem Set registrierten Schlüssel und Präfixe.
    min_snapshot_seq: Option<u64>,
}

impl ReadSet {
    // ANCHOR[contextra-mvcc:READ-SET-NEW]
    /// Erstellt ein neues, leeres [`ReadSet`].
    pub fn new() -> Self {
        Self {
            keys: AHashMap::default(),
            prefixes: AHashMap::default(),
            min_snapshot_seq: None,
        }
    }

    // ANCHOR[contextra-mvcc:READ-SET-RECORD-READ]
    /// Zeichnet einen Lesezugriff auf `key` zu einer bestimmten Snapshot-Sequenznummer `snapshot_seq` auf.
    ///
    /// ### Ablauf:
    /// 1. Maskiere das Tombstone-Bit der Sequenznummer aus.
    /// 2. Falls der Schlüssel in dieser Transaktion bereits gelesen wurde, wird die früheste
    ///    (kleinste) Sequenznummer beibehalten, um strikte Isolationsgrenzen zu garantieren.
    /// 3. Aktualisiere `min_snapshot_seq` mit dem Minimum.
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

    // ANCHOR[contextra-mvcc:READ-SET-RECORD-PREFIX]
    /// Zeichnet einen Bereichsscan-Präfix zu `snapshot_seq` auf, um Phantom-Reads zu erkennen.
    ///
    /// Wenn nach diesem Lesezugriff eine andere Transaktion einen neuen Schlüssel committet,
    /// der mit diesem Präfix beginnt, schlägt die spätere SSI-Validierung fehl.
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

    /// Alias-Funktion für [`ReadSet::record_read`].
    pub fn register_read(&mut self, key: impl Into<Vec<u8>>, snapshot_seq: u64) {
        self.record_read(key, snapshot_seq);
    }

    /// Liefert die registrierte Snapshot-Sequenznummer für einen Schlüssel, falls vorhanden.
    pub fn get(&self, key: &[u8]) -> Option<u64> {
        self.keys.get(key).copied()
    }

    /// Liefert `true`, falls weder Einzelschlüssel noch Präfixe im ReadSet enthalten sind.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty() && self.prefixes.is_empty()
    }

    /// Liefert die Gesamtzahl aller getrackten Schlüssel und Präfixe.
    pub fn len(&self) -> usize {
        self.keys.len() + self.prefixes.len()
    }

    /// Leert das ReadSet vollständig und setzt die kleinste Sequenznummer zurück.
    pub fn clear(&mut self) {
        self.keys.clear();
        self.prefixes.clear();
        self.min_snapshot_seq = None;
    }

    /// Liefert die kleinste Snapshot-Sequenznummer aller gelesenen Schlüssel und Präfixe.
    pub fn min_snapshot_seq(&self) -> Option<u64> {
        self.min_snapshot_seq
    }

    /// Liefert einen Iterator über alle gelesenen Schlüssel und deren Snapshot-Sequenzen.
    pub fn iter(&self) -> impl Iterator<Item = (&Vec<u8>, &u64)> {
        self.keys.iter()
    }

    /// Liefert einen Iterator über alle Bereichs-Präfixe und deren Snapshot-Sequenzen.
    pub fn prefixes_iter(&self) -> impl Iterator<Item = (&Vec<u8>, &u64)> {
        self.prefixes.iter()
    }

    /// Liefert die interne Schlüssel-Map als Referenz.
    pub fn keys_map(&self) -> &AHashMap<Vec<u8>, u64> {
        &self.keys
    }

    /// Liefert die interne Präfix-Map als Referenz.
    pub fn prefixes_map(&self) -> &AHashMap<Vec<u8>, u64> {
        &self.prefixes
    }
}

// ANCHOR[contextra-mvcc:SSI-VALIDATOR-TRAIT]
/// Schnittstelle (Trait) für die Validierung von Transaktionen unter Serializable Snapshot Isolation (SSI).
///
/// Prüft vor dem Festschreiben (Commit) einer Transaktion, ob nebenläufige Transaktionen
/// Schlüssel oder Präfixe verändert haben, die im [`ReadSet`] dieser Transaktion enthalten sind.
pub trait SsiValidator {
    /// Validiert das [`ReadSet`] der Transaktion `tx_id`.
    ///
    /// ### Fehlerfälle:
    /// Gibt `Err(ContextraError::Conflict(...))` zurück, falls:
    /// - Ein Schlüssel im ReadSet nach dem Lese-Snapshot von einer anderen Transaktion committet wurde (Write-Skew).
    /// - Ein neuer Schlüssel committet wurde, der auf einen gelesenen Bereichsscan-Präfix zutrifft (Phantom-Read).
    /// - Der Snapshot-Wasserstand der Transaktion älter ist als die bereits bereinigte Historie (`pruned_through`).
    ///
    /// ### Nebenläufigkeits-Invariante:
    /// Aufrufer koordinieren Validierung und Commit-Registrierung (z. B. via Commit-Gate-Lock),
    /// um atomare, konfliktfreie Zustandsübergänge sicherzustellen.
    fn validate(&self, tx_id: TxId, read_set: &ReadSet) -> Result<()>;
}

// Repräsentiert einen Sequenz-Bucket, der entweder exakte Schreib-Schlüssel oder vergröberte Präfixe enthält.
#[derive(Debug, Clone, PartialEq, Eq)]
enum SeqBucket {
    Exact(Vec<Vec<u8>>),
    Coarsened {
        prefixes: Vec<Vec<u8>>,
        original_key_count: usize,
    },
}

// ANCHOR[contextra-mvcc:SSI-PRUNING-BLOCKER]
/// Diagnoseinformationen über aktive Snapshots oder Sequenznummern, die das Pruning im Validator blockieren.
///
/// Wird erzeugt, wenn der Speicher für Commit-Schlüssel zu mehr als 80 % gefüllt ist,
/// um im Tracing und Monitoring exakt zu erkennen, welcher langlaufende Leser das Aufräumen blockiert.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PruningBlockerInfo {
    /// Kleinste Sequenznummer, die noch unbereinigt im Register liegt.
    pub min_unpruned_seq: u64,
    /// Aktueller `pruned_through`-Wasserstand.
    pub pruned_through_seq: u64,
    /// Gesamtanzahl der aktuell gehaltenen Commit-Schlüssel.
    pub tracked_commit_keys: usize,
    /// Gesamtanzahl der Sequenz-Buckets.
    pub total_seq_buckets: usize,
    /// Anzahl der bereits vergröberten (coarsened) Buckets.
    pub coarsened_seq_buckets: usize,
    /// Konfigurierte Kapazitätsobergrenze an Schlüsseln.
    pub max_tracked_keys: usize,
    /// Sequenznummer des am längsten aktiven Lese-Snapshots (falls vorhanden).
    pub longest_active_snapshot_seq: Option<u64>,
    /// Dauer, seit der dieser Snapshot aktiv ist.
    pub longest_pin_duration: Option<Duration>,
    /// Gibt an, ob der älteste Pin das konfigurierte Zeitlimit (z. B. 300 s) überschritten hat.
    pub is_pin_expired: bool,
}

/// Coarsens exact committed key lists into prefix summaries (e.g. LCP extraction).
///
/// # Architecture Decision & Invariant (B-16 / INV-MVCC-SSI-1)
/// To prevent total outage when `DEFAULT_MAX_TRACKED_COMMIT_KEYS` capacity is reached (finding B-16),
/// individual exact keys in older sequence buckets are coarsened into prefix summaries.
///
/// ## Soundness Invariant (0% False Negatives)
/// Coarsening guarantees that no real write-skew conflict is ever missed:
/// - Every original key $k$ in a coarsened bucket starts with at least one extracted prefix $p$.
/// - Any point read or range prefix read that would have conflicted with $k$ is guaranteed to overlap
///   with $p$.
/// - False positives may occur (a commit may be conservatively flagged as a conflict), but false
///   negatives are mathematically impossible.
/// - Fail-closed protection remains intact under `INV-MVCC-SSI-1`.
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

#[derive(Debug, Default)]
struct CommittedWrites {
    keys: AHashMap<Vec<u8>, u64>,
    seq_index: BTreeMap<u64, SeqBucket>,
}

impl CommittedWrites {
    fn tracked_count(&self) -> usize {
        self.keys.len() + self.coarsened_prefix_count()
    }

    fn coarsened_prefix_count(&self) -> usize {
        let mut count = 0;
        for bucket in self.seq_index.values() {
            if let SeqBucket::Coarsened { prefixes, .. } = bucket {
                count += prefixes.len();
            }
        }
        count
    }

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

    fn len(&self) -> usize {
        self.tracked_count()
    }

    fn get(&self, key: &[u8]) -> Option<u64> {
        self.keys.get(key).copied()
    }
}

// ANCHOR[contextra-mvcc:SSI-VALIDATOR-STRUCT]
/// Referenz-Implementierung von [`SsiValidator`] auf Basis von [`SequenceLog`] und In-Memory-Commit-Tracking.
///
/// Arbeitet vollständig ohne Festplatten-I/O im Arbeitsspeicher und bietet deterministische
/// Konflikterkennung für Transaktionen.
///
/// ### Deterministische Invariante (INV-MVCC-SSI-1):
/// Die Konfliktauflösung basiert rein auf den Sequenznummern der Lese- und Schreiboperationen.
/// Gleiche Eingabereihenfolgen führen stets zu exakt identischen Validierungsergebnissen.
#[derive(Clone)]
pub struct SequenceLogSsiValidator {
    committed_writes: Arc<RwLock<CommittedWrites>>,
    sequence_log: Option<Arc<RwLock<SequenceLog>>>,
    snapshot_registry: Option<Arc<SnapshotRegistry>>,
    pruned_through: Arc<AtomicU64>,
    max_tracked_keys: usize,
    max_pin_duration: Duration,
    metrics_sink: Option<Arc<parking_lot::RwLock<Arc<dyn contextra_ports::MetricsSink>>>>,
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
    // ANCHOR[contextra-mvcc:SSI-VALIDATOR-NEW]
    /// Erstellt einen neuen [`SequenceLogSsiValidator`] mit Standard-Obergrenze (1.000.000 Schlüssel)
    /// und Standard-[`SystemClock`].
    pub fn new() -> Self {
        Self::new_with_bounds(DEFAULT_MAX_TRACKED_COMMIT_KEYS)
    }

    /// Erstellt einen neuen Validator mit benutzerdefinierter Kapazitätsgrenze für Schlüssel.
    pub fn new_with_bounds(max_tracked_keys: usize) -> Self {
        Self::new_with_clock_and_bounds(Arc::new(SystemClock::new()), max_tracked_keys)
    }

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

    /// Bindet eine benutzerdefinierte Implementierung des [`Clock`]-Ports an den Validator.
    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    /// Ersetzt die aktive [`Clock`]-Instanz.
    pub fn set_clock(&mut self, clock: Arc<dyn Clock>) {
        self.clock = clock;
    }

    /// Liefert eine Referenz auf den aktiven [`Clock`]-Port.
    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

    // ANCHOR[contextra-mvcc:SSI-VALIDATOR-WITH-SEQLOG]
    /// Erstellt einen Validator, der an ein bestehendes [`SequenceLog`] angebunden ist.
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

    /// Verknüpft eine [`SnapshotRegistry`] zur Überwachung aktiver Snapshot-Pins.
    pub fn with_snapshot_registry(mut self, registry: Arc<SnapshotRegistry>) -> Self {
        self.snapshot_registry = Some(registry);
        self
    }

    /// Setzt die verknüpfte [`SnapshotRegistry`].
    pub fn set_snapshot_registry(&mut self, registry: Arc<SnapshotRegistry>) {
        self.snapshot_registry = Some(registry);
    }

    /// Bindet einen Metrik-Empfänger ([`contextra_ports::MetricsSink`]) für Monitoring an.
    pub fn with_metrics_sink(
        mut self,
        sink: Arc<parking_lot::RwLock<Arc<dyn contextra_ports::MetricsSink>>>,
    ) -> Self {
        self.metrics_sink = Some(sink);
        self
    }

    /// Aktualisiert den Metrik-Empfänger.
    pub fn set_metrics_sink(
        &mut self,
        sink: Arc<parking_lot::RwLock<Arc<dyn contextra_ports::MetricsSink>>>,
    ) {
        self.metrics_sink = Some(sink);
    }

    /// Konfiguriert die maximale Pin-Dauer für Diagnosealarme.
    pub fn with_max_pin_duration(mut self, duration: Duration) -> Self {
        self.max_pin_duration = duration;
        self
    }

    // ANCHOR[contextra-mvcc:SSI-PRUNE-THROUGH]
    /// Bereinigt (prunt) bestätigte Schreib-Einträge mit `commit_seq <= bound_seq`.
    ///
    /// Aktualisiert den atomaren `pruned_through`-Wasserstand.
    ///
    /// ### Fail-Closed-Sicherheit:
    /// Sobald ein Stand `bound_seq` bereinigt wurde, schlägt jede Transaktion, die einen Snapshot
    /// mit `snapshot_seq < bound_seq` nutzt, sofort fehl, um unerkannte Write-Skews zu verhindern.
    pub fn prune_through(&self, bound_seq: u64) -> usize {
        let bound_seq = bound_seq & !crate::types::TOMBSTONE_BIT;
        let mut writes = self.committed_writes.write();
        let removed = writes.prune_through(bound_seq);
        self.pruned_through.fetch_max(bound_seq, Ordering::SeqCst);
        removed
    }

    // ANCHOR[contextra-mvcc:SSI-FORGET-FROM]
    /// Entfernt alle Commit-Registrierungen mit `commit_seq >= first_seq` (z. B. nach Transaktionsabbruch).
    pub fn forget_from(&self, first_seq: u64) -> usize {
        let first_seq = first_seq & !crate::types::TOMBSTONE_BIT;
        let mut writes = self.committed_writes.write();
        writes.remove_from_seq(first_seq)
    }

    /// Liefert den aktuellen `pruned_through`-Wasserstand.
    pub fn pruned_through(&self) -> u64 {
        self.pruned_through.load(Ordering::SeqCst)
    }

    /// Liefert die Anzahl aktuell gehaltener Commit-Schlüssel.
    pub fn tracked_commit_keys(&self) -> usize {
        self.committed_writes.read().len()
    }

    /// Liefert die konfigurierte Obergrenze für Schlüssel.
    pub fn max_tracked_keys(&self) -> usize {
        self.max_tracked_keys
    }

    /// Checks register capacity utilization and triggers coarsening or diagnostic warnings if threshold is reached.
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

    // ANCHOR[contextra-mvcc:SSI-DIAGNOSE-PRUNING]
    /// Diagnostiziert, welcher unbereinigte Stand oder aktiver Snapshot das Pruning im Validator blockiert.
    pub fn diagnose_pruning_blocker(&self) -> Option<PruningBlockerInfo> {
        self.diagnose_pruning_blocker_at(self.clock.monotonic_nanos())
    }

    /// Diagnostiziert Blockaden bezüglich eines expliziten Zeitstempels `now_nanos`.
    pub fn diagnose_pruning_blocker_at(&self, now_nanos: u64) -> Option<PruningBlockerInfo> {
        let writes = self.committed_writes.read();
        self.diagnose_pruning_blocker_internal(&writes, now_nanos)
    }

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

    // ANCHOR[contextra-mvcc:SSI-RECORD-COMMIT-KEY]
    /// Zeichnet einen committeten Schlüssel zu einer Sequenznummer auf.
    ///
    /// ### Ablauf:
    /// 1. Tombstone-Bit der Commit-Sequenznummer ausmaskieren.
    /// 2. Schlüssel im Schreibregister unter Mutex eintragen.
    /// 3. Kapazitätsprüfung durchführen: Liegt der Füllstand über 80 %, werden ältere Buckets
    ///    automatisch zu Präfixen vergröbert (Coarsening).
    pub fn record_commit_key(&self, key: &[u8], commit_seq: u64) {
        let commit_seq = commit_seq & !crate::types::TOMBSTONE_BIT;
        let mut writes = self.committed_writes.write();
        writes.insert(key.to_vec(), commit_seq);
        self.check_and_coarsen_if_needed(&mut writes);
    }

    // ANCHOR[contextra-mvcc:SSI-RECORD-COMMIT-KEYS]
    /// Zeichnet eine Menge committeter Schlüssel stapelweise (Batch) zu einer Commit-Sequenznummer auf.
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

    /// Validates point reads against committed write keys and coarsened prefix buckets.
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

    /// Validates `SequenceLog` entries for a point read key.
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

    /// Validates range scan prefix reads against committed writes and coarsened prefix buckets for phantom protection.
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

    /// Validates `read_set` against `committed_writes` map and `sequence_log`.
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

    /// Atomically validates `read_set` and records `write_keys` at `commit_seq` under a single write lock.
    // ANCHOR[contextra-mvcc:SSI-VALIDATE-AND-RECORD]
    /// Führt die Validierung und das Eintragen der Commit-Schlüssel atomar unter demselben Schreib-Lock aus.
    ///
    /// Verhindert Race Conditions zwischen der Validierung eines ReadSets und dem Hinzufügen
    /// neuer Commits durch parallele Worker.
    ///
    /// ### Ablauf:
    /// 1. Schreib-Sperre auf `committed_writes` anfordern.
    /// 2. `pruned_through`-Wasserstand abfragen.
    /// 3. Validierung gegen committed writes und range reads durchführen.
    /// 4. Bei Erfolg: Alle Schlüssel mit `commit_seq` eintragen und Kapazität prüfen.
    /// 5. Bei Konflikt: Sofortigen Abbruch mit `Err(ContextraError::Conflict)` zurückgeben.
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
    fn validate(&self, tx_id: TxId, read_set: &ReadSet) -> Result<()> {
        let committed_writes = self.committed_writes.read();
        self.validate_internal(&committed_writes, tx_id, read_set)
    }
}

// Leitet eine DocId deterministisch aus einem Schlüssel-Byte-Array ab.
// Falls der Schlüssel valider UTF-8-Text ist, wird DocId::from_key verwendet, andernfalls AHasher.
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

    // Prüft Grundoperationen des ReadSets (Einfügen, Minimum-Tracking und Überschreiben älterer Snapshots).
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

    // Prüft, dass ein Commit zur gleichen Sequenznummer wie der Snapshot keinen Konflikt auslöst.
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

    // Prüft, dass eine Schreiboperation mit commit_seq > snapshot_seq zuverlässig als Konflikt erkannt wird.
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

    // Prüft die Erkennung von Konflikten bei Batch-Registrierung mehrerer Schlüssel.
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

    // Prüft das Zusammenspiel des Validators mit einem SequenceLog für Dokumentänderungen.
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

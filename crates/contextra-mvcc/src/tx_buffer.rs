//! # TxBuffer — Sharded Transaktionspuffer für zweiphasiges Index-Staging
//!
//! **Ring:** Ring 0 (Transaktionsisolation & Speicher-Kern)
//! **Klasse:** Kern `[K]`
//!
//! ## Zweck & Architektur-Rolle
//!
//! Der [`TxBuffer`] dient als temporärer Zwischenspeicher für noch nicht committete
//! Index-Operationen ([`IndexOp`]) während einer Transaktion.
//!
//! ### Sharding & Deadlock-Prävention
//! Zur Vermeidung von Lock-Contention ist der Puffer intern in mehrere unabhängige Shards unterteilt
//! (Standard: 64 Shards). Transaktionen werden über `tx_id.inner() % shard_count` deterministisch
//! einem Shard zugeordnet.
//! - **Deadlock-Freiheit:** Aufrufer dürfen niemals zwei Shard-Locks gleichzeitig halten.
//! - **Shard-Sweeps:** Methoden wie `reap_orphans()` oder `min_read_snapshot()` durchlaufen
//!   die Shards strikt sequenziell von 0 bis N-1 und sperren/entsperren jeden Shard einzeln.
//!
//! ### Schutz vor Speichererschöpfung (DoS-Schutz)
//! - **Operationen-Budget:** Begrenzung auf maximal 10.000 Operationen pro Transaktion.
//! - **Byte-Budgets:** Maximale Staging-Größe pro Transaktion (16 MiB) und global über alle Transaktionen (256 MiB).
//! - **Orphan Reaping:** Automatisches Bereinigen verwaister oder abgelaufener Transaktionen nach Timeout.

// FILE-CONTEXT
// STAND: 2026-09-28T00:00:00Z (SESSION: e459bd5f)
// ZWECK: Shard-basierter Transaktionsbuffer für das Staging von 2-Phase-Commit Index-Operationen, ReadSet-Limits & Watermark-Tracking.
// INVARIANTEN: Shard-Isolation per TxId; Niemals zwei Shards gleichzeitig sperren (Deadlock-Prävention); Bounded ReadSet via try_register_read.
// HOTSPOTS: 110-380
// NICHT-OFFENSICHTLICH: Orphan Reaper & min_read_snapshot führen getrennte Locks pro Shard 0..N-1 durch; keinesfalls verschachtelt.
// SIEHE AUCH: crates/contextra-mvcc/src/ssi.rs

// INVARIANT: Sharded Transaction Buffer für lock-freie Concurrency.

use crate::error::{ContextraError, Result};
use crate::ssi::ReadSet;
use crate::types::{DocId, TxId};
use ahash::AHashMap;
use contextra_ports::{Clock, SystemClock};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

// ANCHOR[contextra-mvcc:TXB-001]
/// Fester kalkulatorischer Overhead pro Staging-Eintrag (32 Bytes).
pub const STAGING_ENTRY_OVERHEAD_BYTES: usize = 32;

// ANCHOR[contextra-mvcc:TXB-002]
/// Standard-Anzahl an Shards zur Verteilung der Lock-Contention (64 Shards).
pub const DEFAULT_SHARD_COUNT: usize = 64;

// ANCHOR[contextra-mvcc:TXB-003]
/// Maximale Anzahl von Operationen pro Transaktion zum Schutz vor OOM-Angriffen (10.000 Operationen).
// NOTE: Bounded Staging schützt vor unkontrolliertem Speicherwachstum in Transaktionen.
pub const DEFAULT_MAX_OPS_PER_TX: usize = 10_000;

// ANCHOR[contextra-mvcc:TXB-004]
/// Standard-Obergrenze für nachverfolgte Lese-Schlüssel im [`ReadSet`] einer Transaktion (100.000 Schlüssel).
pub const DEFAULT_MAX_READ_SET_KEYS: usize = 100_000;

// ANCHOR[contextra-mvcc:TXB-005]
/// Standard-Obergrenze für zwischengespeicherte Bytes einer einzelnen Transaktion (16 MiB).
pub const DEFAULT_MAX_TX_STAGED_BYTES: usize = 16 * 1024 * 1024;

// ANCHOR[contextra-mvcc:TXB-006]
/// Standard-Gesamtobergrenze für zwischengespeicherte Bytes über alle Transaktionen hinweg (256 MiB).
pub const DEFAULT_MAX_TOTAL_STAGED_BYTES: usize = 256 * 1024 * 1024;

// ANCHOR[contextra-mvcc:TXB-007]
/// Konfigurationsoptionen für die Kapazitätsgrenzen und Timeouts des `TxBuffer`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxBufferConfig {
    /// Maximale Lebensdauer einer Transaktion vor Bereinigung als verwaist (Orphan).
    pub tx_timeout: Duration,
    /// Empfohlene maximale Anzahl aktiver Transaktionen.
    pub max_active_tx: usize,
    /// Maximale Anzahl an Staging-Operationen pro Transaktion (Standard: 10.000).
    pub max_ops_per_tx: usize,
    /// Maximales Byte-Budget einer einzelnen Transaktion (Standard: 16 MiB).
    pub max_tx_staged_bytes: usize,
    /// Maximales globales Byte-Budget aller Transaktionen (Standard: 256 MiB).
    pub max_total_staged_bytes: usize,
}

impl Default for TxBufferConfig {
    fn default() -> Self {
        Self {
            tx_timeout: Duration::from_secs(30),
            max_active_tx: 64,
            max_ops_per_tx: DEFAULT_MAX_OPS_PER_TX,
            max_tx_staged_bytes: DEFAULT_MAX_TX_STAGED_BYTES,
            max_total_staged_bytes: DEFAULT_MAX_TOTAL_STAGED_BYTES,
        }
    }
}

// ANCHOR[contextra-mvcc:TXB-008]
/// Repräsentiert eine im Index auszuführende Staging-Operation (Einfügen oder Löschen).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum IndexOp<T: Clone> {
    /// Dokumenteneinfügung mit Nutzdaten.
    Insert {
        /// Dokument-ID des einzufügenden Dokuments.
        doc_id: DocId,
        /// Assoziierte Nutzdaten (z. B. Vektor oder Text-Payload).
        data: T,
    },
    /// Dokumentenlöschung.
    Delete {
        /// Dokument-ID des zu löschenden Dokuments.
        doc_id: DocId,
        /// Optionale Nutzdaten (z. B. zur Validierung).
        data: Option<T>,
    },
}

impl<T: Clone> IndexOp<T> {
    // ANCHOR[contextra-mvcc:TXB-009]
    /// Gibt die Dokument-ID dieser Index-Operation zurück.
    pub fn doc_id(&self) -> DocId {
        match self {
            IndexOp::Insert { doc_id, .. } => *doc_id,
            IndexOp::Delete { doc_id, .. } => *doc_id,
        }
    }
}

// ANCHOR[contextra-mvcc:TXB-010]
/// Trait zur Ermittlung des Byte-Fußabdrucks von zwischengespeicherten Nutzdaten für Budgetprüfungen.
pub trait StagedOpSize {
    /// Estimates memory footprint in bytes for staging entry overhead accounting.
    fn staged_bytes(&self) -> usize;
}

impl<T> StagedOpSize for Vec<T> {
    fn staged_bytes(&self) -> usize {
        self.len() * std::mem::size_of::<T>()
    }
}

impl StagedOpSize for (Vec<u8>, Vec<u8>) {
    fn staged_bytes(&self) -> usize {
        self.0.len() + self.1.len()
    }
}

impl StagedOpSize for String {
    fn staged_bytes(&self) -> usize {
        self.len()
    }
}

impl StagedOpSize for usize {
    fn staged_bytes(&self) -> usize {
        std::mem::size_of::<usize>()
    }
}

impl StagedOpSize for u64 {
    fn staged_bytes(&self) -> usize {
        std::mem::size_of::<u64>()
    }
}

impl StagedOpSize for u8 {
    fn staged_bytes(&self) -> usize {
        std::mem::size_of::<u8>()
    }
}

impl<T: Clone + StagedOpSize> IndexOp<T> {
    // ANCHOR[contextra-mvcc:TXB-011]
    /// Schätzt den gesamten Speicherbedarf in Bytes (fester Overhead von 32 Bytes + Payload-Größe).
    pub fn estimated_bytes(&self) -> usize {
        STAGING_ENTRY_OVERHEAD_BYTES
            + match self {
                IndexOp::Insert { data, .. } => data.staged_bytes(),
                IndexOp::Delete { data: Some(d), .. } => d.staged_bytes(),
                IndexOp::Delete { data: None, .. } => 0,
            }
    }
}

// ANCHOR[contextra-mvcc:TXB-012]
/// Transaktions-Shard: Verwaltet Operationen, ReadSets und Staged-Bytes isoliert pro Transaktions-ID.
#[derive(Debug)]
struct TxShard<T: Clone> {
    ops: AHashMap<TxId, (Vec<IndexOp<T>>, u64)>,
    read_sets: AHashMap<TxId, ReadSet>,
    staged_bytes: AHashMap<TxId, usize>,
}

impl<T: Clone> TxShard<T> {
    // ANCHOR[contextra-mvcc:TXB-013]
    /// Erzeugt einen neuen leeren Transaktions-Shard.
    fn new() -> Self {
        Self {
            ops: AHashMap::new(),
            read_sets: AHashMap::new(),
            staged_bytes: AHashMap::new(),
        }
    }
}

// ANCHOR[contextra-mvcc:TXB-014]
/// Schlüssel-Shard: Hält Staging-Status von KV-Schlüsseln zur Vermeidung von Lese-Schreib-Anomalien.
#[derive(Debug)]
struct KeyShard {
    staged: AHashMap<Vec<u8>, AHashMap<TxId, bool>>,
}

impl KeyShard {
    // ANCHOR[contextra-mvcc:TXB-015]
    /// Erzeugt einen neuen leeren Schlüssel-Shard.
    fn new() -> Self {
        Self {
            staged: AHashMap::new(),
        }
    }
}

// ANCHOR[contextra-mvcc:TXB-016]
/// Zwischenspeicher für uncommittete Index-Operationen mit feingranularem Sharding.
///
/// Jeder Shard wird durch ein eigenes `parking_lot::RwLock` geschützt.
///
/// # Invariante (Deadlock-Freiheit)
/// Aufrufer dürfen niemals zwei Shard-Locks gleichzeitig akquirieren.
/// Sammeloperationen (wie `reap_orphans()` oder `min_read_snapshot()`) durchlaufen
/// die Shards sequenziell von Index 0 bis N-1 und geben jede Sperre vor der nächsten frei.
pub struct TxBuffer<T: Clone> {
    shards: Vec<RwLock<TxShard<T>>>,
    key_shards: Vec<RwLock<KeyShard>>,
    tx_timeout: Duration,
    config: TxBufferConfig,
    global_staged_bytes: AtomicUsize,
    clock: Arc<dyn Clock>,
}

impl<T: Clone> std::fmt::Debug for TxBuffer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TxBuffer")
            .field("tx_timeout", &self.tx_timeout)
            .field("config", &self.config)
            .field("global_staged_bytes", &self.global_staged_bytes)
            .field("shard_count", &self.shards.len())
            .finish()
    }
}

impl<T: Clone> TxBuffer<T> {
    // ANCHOR[contextra-mvcc:TXB-017]
    /// Erstellt einen neuen Transaktionspuffer mit 64 Shards und 30s Timeout unter [`SystemClock`].
    pub fn new() -> Self {
        Self::new_with_config(DEFAULT_SHARD_COUNT, Duration::from_secs(30))
    }

    // ANCHOR[contextra-mvcc:TXB-018]
    /// Erstellt einen neuen Transaktionspuffer mit benutzerdefinierter Shard-Anzahl und Timeout.
    pub fn new_with_config(shard_count: usize, tx_timeout: Duration) -> Self {
        let config = TxBufferConfig {
            tx_timeout,
            ..Default::default()
        };
        Self::new_with_config_ext(shard_count, tx_timeout, config)
    }

    // ANCHOR[contextra-mvcc:TXB-019]
    /// Erstellt einen Puffer mit expliziter `TxBufferConfig` unter Verwendung der Standard-Systemuhr.
    pub fn new_with_config_ext(
        shard_count: usize,
        _tx_timeout: Duration,
        config: TxBufferConfig,
    ) -> Self {
        Self::new_with_clock_and_config(Arc::new(SystemClock::new()), shard_count, config)
    }

    // ANCHOR[contextra-mvcc:TXB-020]
    /// Erstellt einen Transaktionspuffer mit injiziertem [`Clock`]-Port, Shard-Anzahl und Konfiguration.
    ///
    /// Garantiert deterministisches Verhalten in Tests durch Verwendung injizierbarer Uhren.
    pub fn new_with_clock_and_config(
        clock: Arc<dyn Clock>,
        shard_count: usize,
        config: TxBufferConfig,
    ) -> Self {
        let shard_count = if shard_count == 0 { 1 } else { shard_count };
        let mut shards = Vec::with_capacity(shard_count);
        let mut key_shards = Vec::with_capacity(shard_count);
        for _ in 0..shard_count {
            shards.push(RwLock::new(TxShard::new()));
            key_shards.push(RwLock::new(KeyShard::new()));
        }
        Self {
            shards,
            key_shards,
            tx_timeout: config.tx_timeout,
            config,
            global_staged_bytes: AtomicUsize::new(0),
            clock,
        }
    }

    // ANCHOR[contextra-mvcc:TXB-021]
    /// Verknüpft eine benutzerdefinierte [`Clock`] mit diesem Puffer (Builder-Muster).
    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    // ANCHOR[contextra-mvcc:TXB-022]
    /// Liefert den aktiven [`Clock`]-Port dieses Transaktionspuffers.
    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

    // ANCHOR[contextra-mvcc:TXB-023]
    /// Liefert die Summe aller aktuell gestageten Bytes über alle Transaktionen hinweg.
    pub fn staged_bytes(&self) -> usize {
        self.global_staged_bytes.load(Ordering::Relaxed)
    }

    // ANCHOR[contextra-mvcc:TXB-024]
    /// Berechnet deterministisch den Shard-Index für eine Transaktions-ID.
    #[inline]
    fn shard_idx(&self, tx: TxId) -> usize {
        let num_shards = self.shards.len();
        if num_shards == 0 {
            0
        } else {
            usize::try_from(tx.inner() % num_shards as u64).unwrap_or(0)
        }
    }

    // ANCHOR[contextra-mvcc:TXB-025]
    /// Berechnet deterministisch den Schlüssel-Shard für einen Byte-Schlüssel per AHash.
    #[inline]
    fn key_shard_idx(&self, key: &[u8]) -> usize {
        let num_shards = self.key_shards.len();
        if num_shards == 0 {
            0
        } else {
            use std::hash::Hasher;
            let mut hasher = ahash::AHasher::default();
            hasher.write(key);
            usize::try_from(hasher.finish() % num_shards as u64).unwrap_or(0)
        }
    }

    // ANCHOR[contextra-mvcc:TXB-026]
    /// Prüft, ob eine Transaktion im Puffer registriert ist.
    pub fn has_tx(&self, tx: TxId) -> bool {
        let idx = self.shard_idx(tx);
        if let Some(shard_lock) = self.shards.get(idx) {
            shard_lock.read().ops.contains_key(&tx)
        } else {
            false
        }
    }

    // ANCHOR[contextra-mvcc:TXB-027]
    /// Initialisiert eine Transaktion im Puffer mit explizitem Nanosekunden-Zeitstempel `at_nanos`.
    pub fn begin_at(&self, tx: TxId, at_nanos: u64) {
        let shard_idx = self.shard_idx(tx);
        if let Some(shard_lock) = self.shards.get(shard_idx) {
            let mut shard = shard_lock.write();
            shard
                .ops
                .entry(tx)
                .or_insert_with(|| (Vec::with_capacity(16), at_nanos));
            shard.read_sets.entry(tx).or_default();
            shard.staged_bytes.entry(tx).or_insert(0);
        }
    }

    // ANCHOR[contextra-mvcc:TXB-028]
    /// Initialisiert eine Transaktion unter Verwendung der aktuellen Zeit des `Clock`-Ports.
    pub fn begin(&self, tx: TxId) {
        let now_nanos = self.clock.monotonic_nanos();
        self.begin_at(tx, now_nanos);
    }

    // ANCHOR[contextra-mvcc:TXB-029]
    /// Registriert einen gelesenen Schlüssel und dessen Snapshot-Sequenznummer für Transaktion `tx`.
    pub fn register_read(&self, tx: TxId, key: impl Into<Vec<u8>>, snapshot_seq: u64) {
        let shard_idx = self.shard_idx(tx);
        if let Some(shard_lock) = self.shards.get(shard_idx) {
            let mut shard = shard_lock.write();
            shard
                .read_sets
                .entry(tx)
                .or_default()
                .record_read(key, snapshot_seq);
        }
    }

    // ANCHOR[contextra-mvcc:TXB-030]
    /// Registriert einen Leseschlüssel mit harter Kapazitätsprüfung (`max_keys`).
    ///
    /// Verhindert unbegrenztes ReadSet-Wachstum während großer Scans.
    pub fn try_register_read(
        &self,
        tx: TxId,
        key: impl Into<Vec<u8>>,
        snapshot_seq: u64,
        max_keys: usize,
    ) -> Result<()> {
        let shard_idx = self.shard_idx(tx);
        let shard_lock = self
            .shards
            .get(shard_idx)
            .ok_or_else(|| ContextraError::Storage("Shard index out of bounds".to_string()))?;
        let mut shard = shard_lock.write();
        let rs = shard.read_sets.entry(tx).or_default();

        let k = key.into();
        if rs.get(&k).is_none() && rs.len() >= max_keys {
            return Err(ContextraError::limit_exceeded(
                max_keys,
                format!("Read set size limit exceeded for transaction {tx}"),
            ));
        }

        rs.record_read(k, snapshot_seq);
        Ok(())
    }

    // ANCHOR[contextra-mvcc:TXB-031]
    /// Alias für [`Self::register_read`].
    pub fn record_read(&self, tx: TxId, key: impl Into<Vec<u8>>, snapshot_seq: u64) {
        self.register_read(tx, key, snapshot_seq);
    }

    // ANCHOR[contextra-mvcc:TXB-032]
    /// Ermittelt die kleinste gelesene Snapshot-Sequenznummer über alle aktiven Transaktionen und Shards.
    ///
    /// Durchläuft Shards sequenziell von 0 bis N-1 (keine geschachtelten Locks).
    pub fn min_read_snapshot(&self) -> Option<u64> {
        let mut overall_min: Option<u64> = None;
        for shard_lock in &self.shards {
            let shard = shard_lock.read();
            for rs in shard.read_sets.values() {
                if let Some(min_seq) = rs.min_snapshot_seq() {
                    overall_min = Some(overall_min.map_or(min_seq, |curr| curr.min(min_seq)));
                }
            }
        }
        overall_min
    }

    // ANCHOR[contextra-mvcc:TXB-033]
    /// Liefert einen Klon des erfassten [`ReadSet`] für Transaktion `tx`.
    pub fn get_read_set(&self, tx: TxId) -> Option<ReadSet> {
        let shard_idx = self.shard_idx(tx);
        let shard_lock = self.shards.get(shard_idx)?;
        let shard = shard_lock.read();
        shard.read_sets.get(&tx).cloned()
    }

    // ANCHOR[contextra-mvcc:TXB-034]
    /// Alias für [`Self::get_read_set`].
    pub fn read_set(&self, tx: TxId) -> Option<ReadSet> {
        self.get_read_set(tx)
    }

    // ANCHOR[contextra-mvcc:TXB-035]
    /// Reiht eine Operation in den Transaktionspuffer ein (prüft Kapazitäts- und Byte-Budgets).
    pub fn stage(&self, tx: TxId, op: IndexOp<T>) -> Result<()>
    where
        T: StagedOpSize,
    {
        self.stage_bounded(tx, op)
    }

    // ANCHOR[contextra-mvcc:TXB-036]
    /// Reiht eine Operation ein und erzwingt Budgets (`max_ops_per_tx`, Transaktions- und globales Byte-Limit).
    ///
    /// # Ablauf
    /// 1. Schätze den Speicherbedarf der Operation in Bytes ab.
    /// 2. Shard sperren (Write-Lock).
    /// 3. Prüfe Transaktions-Byte-Budget und globales Byte-Budget.
    /// 4. Prüfe maximale Operationsanzahl für `tx`.
    /// 5. Füge Operation in den Shard ein und aktualisiere Byte-Zähler.
    pub fn stage_bounded(&self, tx: TxId, op: IndexOp<T>) -> Result<()>
    where
        T: StagedOpSize,
    {
        let op_bytes = op.estimated_bytes();
        let shard_idx = self.shard_idx(tx);
        let shard_lock = self
            .shards
            .get(shard_idx)
            .ok_or_else(|| ContextraError::Storage("Shard index out of bounds".to_string()))?;
        let mut shard = shard_lock.write();

        let curr_tx_bytes = *shard.staged_bytes.entry(tx).or_insert(0);
        if curr_tx_bytes + op_bytes > self.config.max_tx_staged_bytes {
            return Err(ContextraError::Storage(format!(
                "Transaction {tx} staging budget exceeded: {} + {op_bytes} > {}",
                curr_tx_bytes, self.config.max_tx_staged_bytes
            )));
        }

        let curr_global = self.global_staged_bytes.load(Ordering::Relaxed);
        if curr_global + op_bytes > self.config.max_total_staged_bytes {
            return Err(ContextraError::Storage(format!(
                "Global staging budget exceeded: {curr_global} + {op_bytes} > {}",
                self.config.max_total_staged_bytes
            )));
        }

        let max_ops = self.config.max_ops_per_tx;
        let now_nanos = self.clock.monotonic_nanos();
        let entry = shard
            .ops
            .entry(tx)
            .or_insert_with(|| (Vec::with_capacity(16), now_nanos));

        if entry.0.len() >= max_ops {
            return Err(ContextraError::Transaction(format!(
                "Transaction {} exceeded max staging capacity ({} ops)",
                tx, max_ops
            )));
        }

        entry.0.push(op);
        shard.staged_bytes.entry(tx).and_modify(|b| *b += op_bytes);
        self.global_staged_bytes
            .fetch_add(op_bytes, Ordering::Relaxed);
        Ok(())
    }

    // ANCHOR[contextra-mvcc:TXB-037]
    /// Reiht mehrere Operationen atomar unter einem einzigen Shard-Write-Lock ein.
    pub fn stage_many(&self, tx: TxId, ops: impl IntoIterator<Item = IndexOp<T>>) -> Result<()>
    where
        T: StagedOpSize,
    {
        let ops_vec: Vec<_> = ops.into_iter().collect();
        let total_op_bytes: usize = ops_vec.iter().map(|op| op.estimated_bytes()).sum();

        let shard_idx = self.shard_idx(tx);
        let shard_lock = self
            .shards
            .get(shard_idx)
            .ok_or_else(|| ContextraError::Storage("Shard index out of bounds".to_string()))?;
        let mut shard = shard_lock.write();

        let curr_tx_bytes = *shard.staged_bytes.entry(tx).or_insert(0);
        if curr_tx_bytes + total_op_bytes > self.config.max_tx_staged_bytes {
            return Err(ContextraError::Storage(format!(
                "Transaction {tx} staging budget exceeded: {} + {total_op_bytes} > {}",
                curr_tx_bytes, self.config.max_tx_staged_bytes
            )));
        }

        let curr_global = self.global_staged_bytes.load(Ordering::Relaxed);
        if curr_global + total_op_bytes > self.config.max_total_staged_bytes {
            return Err(ContextraError::Storage(format!(
                "Global staging budget exceeded: {curr_global} + {total_op_bytes} > {}",
                self.config.max_total_staged_bytes
            )));
        }

        let max_ops = self.config.max_ops_per_tx;
        let now_nanos = self.clock.monotonic_nanos();
        let entry = shard
            .ops
            .entry(tx)
            .or_insert_with(|| (Vec::new(), now_nanos));

        if entry.0.len() + ops_vec.len() > max_ops {
            return Err(ContextraError::Transaction(format!(
                "Transaction {} exceeded max staging capacity ({} ops)",
                tx, max_ops
            )));
        }

        entry.0.extend(ops_vec);
        shard
            .staged_bytes
            .entry(tx)
            .and_modify(|b| *b += total_op_bytes);
        self.global_staged_bytes
            .fetch_add(total_op_bytes, Ordering::Relaxed);
        Ok(())
    }

    // ANCHOR[contextra-mvcc:TXB-038]
    /// Validiert, dass die Transaktion registriert ist und ausstehende Operationen enthält.
    pub fn validate_pending_ops(&self, tx: TxId) -> Result<()> {
        let shard_idx = self.shard_idx(tx);
        if let Some(shard_lock) = self.shards.get(shard_idx) {
            let shard = shard_lock.read();
            if let Some((ops, _)) = shard.ops.get(&tx) {
                if ops.is_empty() {
                    return Err(ContextraError::Transaction(format!(
                        "Transaction {} was registered but has no pending operations",
                        tx
                    )));
                }
            }
        }
        Ok(())
    }

    // ANCHOR[contextra-mvcc:TXB-039]
    /// Entnimmt alle gepufferten Operationen einer Transaktion beim Commit und gibt belegte Budgets frei.
    pub fn drain(&self, tx: TxId) -> Vec<IndexOp<T>> {
        let shard_idx = self.shard_idx(tx);
        if let Some(shard_lock) = self.shards.get(shard_idx) {
            let mut shard = shard_lock.write();
            shard.read_sets.remove(&tx);
            let tx_bytes = shard.staged_bytes.remove(&tx).unwrap_or(0);
            if tx_bytes > 0 {
                self.global_staged_bytes
                    .fetch_sub(tx_bytes, Ordering::Relaxed);
            }
            shard
                .ops
                .remove(&tx)
                .map(|(ops, _)| ops)
                .unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    // ANCHOR[contextra-mvcc:TXB-040]
    /// Verwirft alle gepufferten Operationen und das ReadSet einer Transaktion (Rollback) und gibt Budgets frei.
    pub fn discard(&self, tx: TxId) {
        let shard_idx = self.shard_idx(tx);
        if let Some(shard_lock) = self.shards.get(shard_idx) {
            let mut shard = shard_lock.write();
            shard.read_sets.remove(&tx);
            let tx_bytes = shard.staged_bytes.remove(&tx).unwrap_or(0);
            if tx_bytes > 0 {
                self.global_staged_bytes
                    .fetch_sub(tx_bytes, Ordering::Relaxed);
            }
            shard.ops.remove(&tx);
        }
    }

    // ANCHOR[contextra-mvcc:TXB-041]
    /// Liefert die Gesamtzahl aller aktuell gepufferten Transaktionen über alle Shards.
    pub fn len(&self) -> usize {
        self.shards.iter().map(|s| s.read().ops.len()).sum()
    }

    // ANCHOR[contextra-mvcc:TXB-042]
    /// Prüft, ob alle Shards des Puffers leer sind.
    pub fn is_empty(&self) -> bool {
        self.shards.iter().all(|s| s.read().ops.is_empty())
    }

    // ANCHOR[contextra-mvcc:TXB-043]
    /// Bereinigt abgelaufene Transaktionen bezogen auf den Zeitstempel `now_nanos`.
    pub fn reap_orphans_at(&self, now_nanos: u64) -> Vec<TxId> {
        self.reap_orphans_bounded_at(usize::MAX, now_nanos)
    }

    // ANCHOR[contextra-mvcc:TXB-044]
    /// Bereinigt abgelaufene Transaktionen unter Verwendung der aktuellen Uhrzeit.
    pub fn reap_orphans(&self) -> Vec<TxId> {
        let now_nanos = self.clock.monotonic_nanos();
        self.reap_orphans_at(now_nanos)
    }

    // ANCHOR[contextra-mvcc:TXB-045]
    /// Bereinigt maximal `max` abgelaufene Transaktionen und gibt deren Speicher frei.
    ///
    /// Verwendet `try_write()` auf jedem Shard und gibt die Kontrolle per `yield_now()` ab,
    /// um Blockaden aktiver Schreibthreads zu vermeiden.
    pub fn reap_orphans_bounded_at(&self, max: usize, now_nanos: u64) -> Vec<TxId> {
        let mut expired = Vec::new();
        let timeout_nanos = u64::try_from(self.tx_timeout.as_nanos()).unwrap_or(u64::MAX);

        for shard_lock in &self.shards {
            if expired.len() >= max {
                break;
            }
            if let Some(mut shard) = shard_lock.try_write() {
                let mut shard_expired = Vec::new();
                shard.ops.retain(|tx, (_, created_nanos)| {
                    if expired.len() + shard_expired.len() < max
                        && now_nanos.saturating_sub(*created_nanos) > timeout_nanos
                    {
                        shard_expired.push(*tx);
                        false
                    } else {
                        true
                    }
                });
                for tx in &shard_expired {
                    shard.read_sets.remove(tx);
                    let tx_bytes = shard.staged_bytes.remove(tx).unwrap_or(0);
                    if tx_bytes > 0 {
                        self.global_staged_bytes
                            .fetch_sub(tx_bytes, Ordering::Relaxed);
                    }
                }
                expired.extend(shard_expired);
            }
            std::thread::yield_now();
        }
        expired
    }

    // ANCHOR[contextra-mvcc:TXB-046]
    /// Bereinigt bis zu `max` verwaiste Transaktionen zur aktuellen Zeit.
    pub fn reap_orphans_bounded(&self, max: usize) -> Vec<TxId> {
        let now_nanos = self.clock.monotonic_nanos();
        self.reap_orphans_bounded_at(max, now_nanos)
    }

    // ANCHOR[contextra-mvcc:TXB-047]
    /// Liefert einen Klon der aktuell gepufferten Operationen einer Transaktion.
    pub fn get_ops(&self, tx: TxId) -> Option<Vec<IndexOp<T>>> {
        let shard_idx = self.shard_idx(tx);
        let shard_lock = self.shards.get(shard_idx)?;
        let shard = shard_lock.read();
        shard.ops.get(&tx).map(|(ops, _)| ops.clone())
    }

    // ANCHOR[contextra-mvcc:TXB-048]
    /// Liefert das konfigurierte Transaktions-Timeout.
    pub fn tx_timeout(&self) -> Duration {
        self.tx_timeout
    }
}

impl TxBuffer<(Vec<u8>, Vec<u8>)> {
    // ANCHOR[contextra-mvcc:TXB-049]
    /// Registriert einen Schlüssel im Key-Shard als gestaged.
    fn track_key_stage(&self, tx: TxId, key: &[u8], is_insert: bool) {
        let idx = self.key_shard_idx(key);
        if let Some(shard_lock) = self.key_shards.get(idx) {
            let mut shard = shard_lock.write();
            shard
                .staged
                .entry(key.to_vec())
                .or_default()
                .insert(tx, is_insert);
        }
    }

    // ANCHOR[contextra-mvcc:TXB-050]
    /// Entfernt Schlüssel aus dem Key-Shard nach Abschluss der Transaktion.
    fn untrack_key_ops_kv(&self, tx: TxId, ops: &[IndexOp<(Vec<u8>, Vec<u8>)>]) {
        for op in ops {
            let key = match op {
                IndexOp::Insert { data, .. } => &data.0,
                IndexOp::Delete { data: Some(d), .. } => &d.0,
                IndexOp::Delete { data: None, .. } => continue,
            };
            let idx = self.key_shard_idx(key);
            if let Some(shard_lock) = self.key_shards.get(idx) {
                let mut shard = shard_lock.write();
                if let std::collections::hash_map::Entry::Occupied(mut entry) =
                    shard.staged.entry(key.to_vec())
                {
                    entry.get_mut().remove(&tx);
                    if entry.get().is_empty() {
                        entry.remove();
                    }
                }
            }
        }
    }

    // ANCHOR[contextra-mvcc:TXB-051]
    /// Reiht eine Schlüssel-Wert-Operation ein und aktualisiert die atomare Key-Staging-Map.
    pub fn stage_kv(&self, tx: TxId, op: IndexOp<(Vec<u8>, Vec<u8>)>) -> Result<()> {
        let (key, is_insert) = match &op {
            IndexOp::Insert { data, .. } => (data.0.as_slice(), true),
            IndexOp::Delete { data: Some(d), .. } => (d.0.as_slice(), false),
            IndexOp::Delete { data: None, .. } => {
                return self.stage(tx, op);
            }
        };

        self.track_key_stage(tx, key, is_insert);
        let key_vec = key.to_vec();
        if let Err(e) = self.stage(tx, op) {
            let idx = self.key_shard_idx(&key_vec);
            if let Some(shard_lock) = self.key_shards.get(idx) {
                let mut shard = shard_lock.write();
                if let std::collections::hash_map::Entry::Occupied(mut entry) =
                    shard.staged.entry(key_vec)
                {
                    entry.get_mut().remove(&tx);
                    if entry.get().is_empty() {
                        entry.remove();
                    }
                }
            }
            return Err(e);
        }
        Ok(())
    }

    // ANCHOR[contextra-mvcc:TXB-052]
    /// Entnimmt alle gepufferten Operationen einer KV-Transaktion und bereinigt die Key-Staging-Map.
    pub fn drain_kv(&self, tx: TxId) -> Vec<IndexOp<(Vec<u8>, Vec<u8>)>> {
        let ops = self.drain(tx);
        if !ops.is_empty() {
            self.untrack_key_ops_kv(tx, &ops);
        }
        ops
    }

    // ANCHOR[contextra-mvcc:TXB-053]
    /// Verwirft alle Operationen einer KV-Transaktion und bereinigt das Key-Staging.
    pub fn discard_kv(&self, tx: TxId) {
        if let Some(ops) = self.get_ops(tx) {
            self.untrack_key_ops_kv(tx, &ops);
        }
        self.discard(tx);
    }

    // ANCHOR[contextra-mvcc:TXB-054]
    /// Prüft, ob ein Schlüssel derzeit durch die Transaktion `tx_id` gestaged ist.
    pub fn is_key_staged_for_tx(&self, tx_id: TxId, key: &[u8]) -> bool {
        let shard_idx = self.shard_idx(tx_id);
        if let Some(shard_lock) = self.shards.get(shard_idx) {
            let shard = shard_lock.read();
            if let Some((ops, _)) = shard.ops.get(&tx_id) {
                for op in ops {
                    if let IndexOp::Insert { data, .. } = op {
                        if data.0 == key {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    // ANCHOR[contextra-mvcc:TXB-055]
    /// Ermittelt atomar den Staging-Status eines Keys über alle uncommitteten Transaktionen.
    ///
    /// Liefert `Some(true)` für ein ausstehendes Insert der höchsten TxId, `Some(false)` für Delete, oder `None`.
    pub fn staged_status(&self, key: &[u8]) -> Option<bool> {
        let idx = self.key_shard_idx(key);
        if let Some(shard_lock) = self.key_shards.get(idx) {
            let shard = shard_lock.read();
            if let Some(map) = shard.staged.get(key) {
                let mut max_tx: Option<TxId> = None;
                let mut status: Option<bool> = None;
                for (&tx_id, &is_insert) in map {
                    match max_tx {
                        None => {
                            max_tx = Some(tx_id);
                            status = Some(is_insert);
                        }
                        Some(m) if tx_id > m => {
                            max_tx = Some(tx_id);
                            status = Some(is_insert);
                        }
                        _ => {}
                    }
                }
                return status;
            }
        }
        None
    }
}

impl<T: Clone> Default for TxBuffer<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use contextra_testkit::ManualClock;
    use proptest::{prop_assert, prop_assert_eq};

    #[test]
    fn test_tx_buffer_read_set_tracking() {
        let buffer = TxBuffer::<String>::new();
        let tx = TxId::new(10);
        buffer.begin(tx);

        let initial_rs = buffer.get_read_set(tx);
        assert!(initial_rs.as_ref().is_some_and(|rs| rs.is_empty()));

        buffer.register_read(tx, b"key_a".to_vec(), 100);
        buffer.record_read(tx, b"key_b".to_vec(), 105);

        let rs = buffer.read_set(tx).expect("read set present"); // #[cfg(test)]
        assert_eq!(rs.len(), 2);
        assert_eq!(rs.get(b"key_a"), Some(100));
        assert_eq!(rs.get(b"key_b"), Some(105));

        // Drain cleans up read_set
        buffer.drain(tx);
        assert!(buffer.get_read_set(tx).is_none());
    }

    #[test]
    fn test_txbuffer_respects_max_ops_limit() {
        let config = TxBufferConfig {
            max_ops_per_tx: 3,
            ..Default::default()
        };
        let buffer = TxBuffer::<String>::new_with_config_ext(64, Duration::from_secs(5), config);
        let tx = TxId::new(1);
        buffer.begin(tx);
        // 3 operations succeed
        for i in 0..3 {
            let res = buffer.stage_bounded(
                tx,
                IndexOp::Insert {
                    doc_id: DocId::from(u64::try_from(i).expect("valid u64")),
                    data: format!("data_{i}"),
                },
            );
            assert!(res.is_ok());
        }
        // 4th operation must fail
        let result = buffer.stage_bounded(
            tx,
            IndexOp::Insert {
                doc_id: DocId::from(99u64),
                data: "overflow".to_string(),
            },
        );
        assert!(matches!(result, Err(ContextraError::Transaction(_))));
    }

    #[test]
    fn test_txbuffer_default_config_allows_normal_workload() {
        let buffer = TxBuffer::<String>::new();
        let tx = TxId::new(42);
        buffer.begin(tx);
        for i in 0..500 {
            let res = buffer.stage(
                tx,
                IndexOp::Insert {
                    doc_id: DocId::from(u64::try_from(i).expect("valid u64")),
                    data: format!("data_{i}"),
                },
            );
            assert!(res.is_ok());
        }
        assert_eq!(buffer.get_ops(tx).map(|v| v.len()), Some(500));
    }

    #[test]
    fn test_tx_buffer_stage_many_and_max_ops_constant() {
        let buffer = TxBuffer::<String>::new();
        let tx = TxId::new(42);
        assert_eq!(DEFAULT_MAX_OPS_PER_TX, 10_000);

        let ops = vec![
            IndexOp::Insert {
                doc_id: DocId::from(1u64),
                data: "op1".to_string(),
            },
            IndexOp::Insert {
                doc_id: DocId::from(2u64),
                data: "op2".to_string(),
            },
        ];

        assert!(buffer.stage_many(tx, ops).is_ok());
        assert!(buffer.has_tx(tx));
        let drained = buffer.drain(tx);
        assert_eq!(drained.len(), 2);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_tx_buffer_stage_many_exceeds_capacity() {
        let config = TxBufferConfig {
            max_ops_per_tx: 2,
            ..Default::default()
        };
        let buffer = TxBuffer::<String>::new_with_config_ext(1, Duration::from_secs(5), config);
        let tx = TxId::new(10);

        let ops = vec![
            IndexOp::Insert {
                doc_id: DocId::from(1u64),
                data: "op1".to_string(),
            },
            IndexOp::Insert {
                doc_id: DocId::from(2u64),
                data: "op2".to_string(),
            },
            IndexOp::Insert {
                doc_id: DocId::from(3u64),
                data: "op3".to_string(),
            },
        ];

        let res = buffer.stage_many(tx, ops);
        assert!(matches!(res, Err(ContextraError::Transaction(_))));
    }

    #[test]
    fn test_tx_buffer_reap_orphans_bounded_capping() {
        let clock = Arc::new(ManualClock::new(1_000_000_000));
        let config = TxBufferConfig {
            tx_timeout: Duration::from_millis(5),
            ..Default::default()
        };
        let buffer = TxBuffer::<String>::new_with_clock_and_config(clock.clone(), 1, config);

        buffer.begin(TxId::new(1));
        buffer.begin(TxId::new(2));
        buffer.begin(TxId::new(3));

        // Advance clock by 20ms
        clock.advance(Duration::from_millis(20));

        let reaped = buffer.reap_orphans_bounded(2);
        assert_eq!(reaped.len(), 2);
        assert_eq!(buffer.len(), 1);
    }

    #[test]
    fn test_tx_buffer_discard() {
        let buffer = TxBuffer::<String>::new_with_config(64, Duration::from_secs(30));
        let tx = TxId::new(1);

        let _ = buffer.stage(
            tx,
            IndexOp::Insert {
                doc_id: DocId::from(1u64),
                data: "data1".to_string(),
            },
        );
        buffer.discard(tx);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_concurrent_stage_no_data_loss() {
        let buffer = Arc::new(TxBuffer::<usize>::new_with_config(
            64,
            Duration::from_secs(30),
        ));
        let num_tx = 100;
        let ops_per_tx = 100;

        let mut handles = Vec::new();
        for t in 0..num_tx {
            let buffer = buffer.clone();
            handles.push(std::thread::spawn(move || {
                let tx = TxId::new(t as u64);
                buffer.begin(tx);
                for i in 0..ops_per_tx {
                    let _ = buffer.stage(
                        tx,
                        IndexOp::Insert {
                            doc_id: DocId::from(i as u64),
                            data: i,
                        },
                    );
                }
            }));
        }

        for h in handles {
            h.join().expect("task panicked"); // #[cfg(test)]
        }

        assert_eq!(buffer.len(), num_tx);
        for t in 0..num_tx {
            let tx = TxId::new(t as u64);
            let ops = buffer.drain(tx);
            assert_eq!(ops.len(), ops_per_tx);
        }
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_tx_buffer_reap_orphans() {
        let clock = Arc::new(ManualClock::new(1_000_000_000));
        let config = TxBufferConfig {
            tx_timeout: Duration::from_millis(10),
            ..Default::default()
        };
        let buffer = TxBuffer::<String>::new_with_clock_and_config(clock.clone(), 1, config);
        let tx = TxId::new(1);
        buffer.begin(tx);

        clock.advance(Duration::from_millis(20));

        let expired = buffer.reap_orphans();
        assert_eq!(expired, vec![tx]);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_tx_buffer_reap_orphans_deterministic_time_injection() {
        let clock = Arc::new(ManualClock::new(20_000_000_000));
        let config = TxBufferConfig {
            tx_timeout: Duration::from_secs(10),
            ..Default::default()
        };
        let buffer = TxBuffer::<String>::new_with_clock_and_config(clock.clone(), 1, config);
        let tx = TxId::new(100);

        let past_nanos = clock.now_nanos().saturating_sub(15_000_000_000);

        // Begin transaction at 15 seconds in the past
        buffer.begin_at(tx, past_nanos);

        // Reaping at current clock (elapsed 15s > timeout 10s) must reap the orphan
        let expired = buffer.reap_orphans();
        assert_eq!(expired, vec![tx]);
        assert!(buffer.is_empty());
    }

    #[tokio::test]
    async fn test_has_tx_concurrent() {
        let buffer = Arc::new(TxBuffer::<String>::new());
        let num_tasks = 8;
        let mut handles = Vec::new();

        for i in 0..num_tasks {
            let buffer = buffer.clone();
            let tx = TxId::new(u64::try_from(i).expect("valid u64") + 100);
            handles.push(tokio::spawn(async move {
                buffer.begin(tx);
                let _ = buffer.stage(
                    tx,
                    IndexOp::Insert {
                        doc_id: DocId::from(u64::try_from(i).expect("valid u64")),
                        data: format!("data_{i}"),
                    },
                );
                tx
            }));
        }

        let mut txs = Vec::new();
        for h in handles {
            let tx = h.await.expect("task failed"); // #[cfg(test)]
            txs.push(tx);
        }

        for &tx in &txs {
            assert!(buffer.has_tx(tx));
        }

        for &tx in &txs {
            buffer.drain(tx);
            assert!(!buffer.has_tx(tx));
        }

        assert!(buffer.is_empty());
    }

    #[test]
    fn test_tx_buffer_validate_pending_ops() {
        let buffer = TxBuffer::<String>::new();
        let tx = TxId::new(1);

        // No tx yet
        assert!(buffer.validate_pending_ops(tx).is_ok());

        // Registered but empty
        buffer.begin(tx);
        assert!(buffer.validate_pending_ops(tx).is_err());

        // With ops
        let _ = buffer.stage(
            tx,
            IndexOp::Insert {
                doc_id: DocId::from(1u64),
                data: "s".to_string(),
            },
        );
        assert!(buffer.validate_pending_ops(tx).is_ok());
    }

    #[test]
    fn test_is_key_staged_for_tx_semantics() {
        let buffer = TxBuffer::<(Vec<u8>, Vec<u8>)>::new();
        let tx1 = TxId::new(1);
        let tx2 = TxId::new(2);

        buffer.begin(tx1);
        buffer.begin(tx2);

        let key_a = b"key_a".to_vec();
        let val_a = b"val_a".to_vec();

        buffer
            .stage_kv(
                tx1,
                IndexOp::Insert {
                    doc_id: DocId::from(100u64),
                    data: (key_a.clone(), val_a.clone()),
                },
            )
            .expect("stage tx1"); // #[cfg(test)]

        // tx1 has key_a staged
        assert!(buffer.is_key_staged_for_tx(tx1, &key_a));
        // tx2 does NOT have key_a staged in its transaction scope
        assert!(!buffer.is_key_staged_for_tx(tx2, &key_a));
    }

    #[test]
    fn test_index_op_helpers() {
        let op = IndexOp::Insert {
            doc_id: DocId::from(1u64),
            data: "d",
        };
        assert_eq!(op.doc_id(), DocId::from(1u64));

        let op2 = IndexOp::Delete::<String> {
            doc_id: DocId::from(2u64),
            data: None,
        };
        assert_eq!(op2.doc_id(), DocId::from(2u64));
    }

    #[test]
    fn test_tx_buffer_config() {
        let timeout = Duration::from_secs(100);
        let buffer = TxBuffer::<u8>::new_with_config(4, timeout);
        assert_eq!(buffer.tx_timeout(), timeout);
        for i in 0..10 {
            buffer.begin(TxId::new(i));
        }
    }

    #[test]
    fn test_tx_buffer_zero_shards_defaults_to_one() {
        let buffer = TxBuffer::<u8>::new_with_config(0, Duration::from_secs(1));
        let tx = TxId::new(42);
        buffer.begin(tx);
        let _ = buffer.stage(
            tx,
            IndexOp::Insert {
                doc_id: DocId::from(1u64),
                data: 0,
            },
        );
        let ops = buffer.drain(tx);
        assert_eq!(ops.len(), 1);
        assert!(buffer.is_empty());
    }

    proptest::proptest! {
        #[test]
        fn prop_tx_buffer_isolation(
            tx_ids in proptest::collection::vec(0..u64::MAX, 1..50),
            shard_count in 1..256usize
        ) {
            let buffer = TxBuffer::<u64>::new_with_config(shard_count, Duration::from_secs(60));

            for &id in &tx_ids {
                let tx = TxId::new(id);
                let _ = buffer.stage(tx, IndexOp::Insert { doc_id: DocId::from(id), data: id });
            }

            for &id in &tx_ids {
                let tx = TxId::new(id);
                let ops = buffer.get_ops(tx).unwrap(); // #[cfg(test)]
                for op in ops {
                    match op {
                        IndexOp::Insert { doc_id, data } => {
                            prop_assert_eq!(doc_id, DocId::from(id));
                            prop_assert_eq!(data, id);
                        },
                        _ => panic!("Unexpected op"),
                    }
                }
            }

            for &id in &tx_ids {
                let tx = TxId::new(id);
                buffer.drain(tx);
            }
            prop_assert!(buffer.is_empty());
        }

        #[test]
        fn prop_tx_buffer_reap_is_complete(
            tx_count in 1..100usize,
            timeout_ms in 1..100u64
        ) {
            let clock = Arc::new(ManualClock::new(1_000_000_000));
            let config = TxBufferConfig {
                tx_timeout: Duration::from_millis(timeout_ms),
                ..Default::default()
            };
            let buffer = TxBuffer::<u8>::new_with_clock_and_config(clock.clone(), 16, config);

            for i in 0..tx_count {
                buffer.begin(TxId::new(u64::try_from(i).expect("valid u64")));
            }

            clock.advance(Duration::from_millis(timeout_ms * 2));

            let reaped = buffer.reap_orphans();
            prop_assert_eq!(reaped.len(), tx_count);
            prop_assert!(buffer.is_empty());
        }

        #[test]
        fn prop_tx_buffer_stage_drain_stage_lifecycle(
            tx_id_raw in 0..u64::MAX,
            first_ops in proptest::collection::vec(0..100u64, 1..20),
            second_ops in proptest::collection::vec(0..100u64, 1..20)
        ) {
            let buffer = TxBuffer::<u64>::new();
            let tx = TxId::new(tx_id_raw);

            for &val in &first_ops {
                let _ = buffer.stage(tx, IndexOp::Insert { doc_id: DocId::from(val), data: val });
            }

            let drained1 = buffer.drain(tx);
            prop_assert_eq!(drained1.len(), first_ops.len());
            for (idx, op) in drained1.into_iter().enumerate() {
                match op {
                    IndexOp::Insert { doc_id, data } => {
                        let expected = first_ops.get(idx).copied().unwrap_or_default();
                        prop_assert_eq!(doc_id, DocId::from(expected));
                        prop_assert_eq!(data, expected);
                    }
                    _ => panic!("Expected Insert"),
                }
            }

            prop_assert!(buffer.get_ops(tx).is_none());
            prop_assert!(!buffer.has_tx(tx));

            for &val in &second_ops {
                let _ = buffer.stage(tx, IndexOp::Insert { doc_id: DocId::from(val), data: val });
            }

            let drained2 = buffer.drain(tx);
            prop_assert_eq!(drained2.len(), second_ops.len());
            for (idx, op) in drained2.into_iter().enumerate() {
                match op {
                    IndexOp::Insert { doc_id, data } => {
                        let expected = second_ops.get(idx).copied().unwrap_or_default();
                        prop_assert_eq!(doc_id, DocId::from(expected));
                        prop_assert_eq!(data, expected);
                    }
                    _ => panic!("Expected Insert"),
                }
            }
            prop_assert!(buffer.is_empty());
        }

        #[test]
        fn prop_tx_buffer_partial_discard_isolation(
            tx_ids in proptest::collection::vec(0..u64::MAX, 2..40),
            discard_indices in proptest::collection::vec(0..100usize, 1..20)
        ) {
            let buffer = TxBuffer::<u64>::new();

            let mut unique_txs = tx_ids;
            unique_txs.sort_unstable();
            unique_txs.dedup();
            if unique_txs.len() < 2 {
                return Ok(());
            }

            for &id in &unique_txs {
                let tx = TxId::new(id);
                let _ = buffer.stage(tx, IndexOp::Insert { doc_id: DocId::from(id), data: id });
            }

            let mut to_discard = ahash::AHashSet::default();
            for seed in discard_indices {
                let idx = seed % unique_txs.len();
                if let Some(&tx_id) = unique_txs.get(idx) {
                    to_discard.insert(tx_id);
                }
            }

            for &id in &to_discard {
                buffer.discard(TxId::new(id));
            }

            for &id in &unique_txs {
                let tx = TxId::new(id);
                if to_discard.contains(&id) {
                    prop_assert!(!buffer.has_tx(tx));
                    prop_assert!(buffer.get_ops(tx).is_none());
                } else {
                    prop_assert!(buffer.has_tx(tx));
                    let ops = buffer.get_ops(tx).unwrap(); // #[cfg(test)]
                    prop_assert_eq!(ops.len(), 1);
                    if let Some(op) = ops.first() {
                        match op {
                            IndexOp::Insert { doc_id, data } => {
                                prop_assert_eq!(*doc_id, DocId::from(id));
                                prop_assert_eq!(*data, id);
                            }
                            _ => panic!("Expected Insert"),
                        }
                    }
                }
            }
        }
    }
}

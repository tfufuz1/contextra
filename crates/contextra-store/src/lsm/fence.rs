// FILE-CONTEXT
// STAND: 2026-10-21
// ZWECK: Scope-weite und präfixbasierte Schreibsperre (ErasureGate & ErasureFence) zur Blockierung von Schreibvorgängen während der Purge-Compaction.
// INVARIANTEN: Lock-Hierarchie einhalten, keine Locks über .await, loom-kompatibel.

#![deny(clippy::wildcard_enum_match_arm)]

use contextra_mvcc::IndexOp;
use contextra_ports::erasure::{WriteFenced, WriteGate};
use std::collections::HashSet;
use std::time::Duration;

#[cfg(loom)]
use loom::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
#[cfg(loom)]
use loom::sync::{Arc, Mutex, RwLock};

#[cfg(not(loom))]
use parking_lot::Mutex;
#[cfg(not(loom))]
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
#[cfg(not(loom))]
use std::sync::Arc;

/// Key variant passed to `ErasureFence::enter_write`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FencedKey<'a> {
    Key(&'a [u8]),
    Unknown,
}

impl<'a> From<&'a [u8]> for FencedKey<'a> {
    fn from(s: &'a [u8]) -> Self {
        FencedKey::Key(s)
    }
}

impl<'a> From<&'a Vec<u8>> for FencedKey<'a> {
    fn from(v: &'a Vec<u8>) -> Self {
        FencedKey::Key(v.as_slice())
    }
}

/// Target representation for fence registration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FenceTarget {
    Exact(Vec<u8>),
    Prefix(Vec<u8>),
}

/// Scope-wide write block state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FenceScopeState {
    /// Active write access permitted.
    Active,
    /// Collection is currently being dropped / purged. Scope-wide write block active.
    CollectionDropping,
}

/// Errors returned by `ErasureFence` scope operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FenceError {
    #[error("Collection or scope write block active")]
    CollectionDropping,
    #[error("Fence scope conflict detected")]
    Conflict,
    #[error("Fence drain timeout")]
    DrainTimeout,
    #[error("Scope write fenced")]
    ScopeFenced,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefixFence {
    pub purge_id: u64,
    pub scope: PurgeScope,
}

impl PrefixFence {
    pub fn prefixes(&self) -> &[Vec<u8>] {
        match &self.scope {
            PurgeScope::Keys(keys) => keys.as_slice(),
            PurgeScope::Prefixes(prefixes) => prefixes.as_slice(),
        }
    }
}

/// Maps an `IndexOp` to `FencedKey` using exhaustive matching on known IndexOp variants.
pub fn fenced_keys<'a>(op: &'a IndexOp<(Vec<u8>, Vec<u8>)>) -> FencedKey<'a> {
    match op {
        IndexOp::Insert { data, .. } => FencedKey::Key(&data.0[..]),
        IndexOp::Delete {
            data: Some((k, _)), ..
        } => FencedKey::Key(&k[..]),
        IndexOp::Delete { data: None, .. } => FencedKey::Unknown,
        _ => FencedKey::Unknown,
    }
}

/// Low-level gate managing atomic active flag, RCU prefix fence table, and inflight write batch tracking.
#[derive(Debug)]
pub struct ErasureGate {
    active: AtomicBool,
    #[cfg(not(loom))]
    table: arc_swap::ArcSwap<Vec<PrefixFence>>,
    #[cfg(loom)]
    table: RwLock<Vec<PrefixFence>>,
    inflight: AtomicU32,
    drained: tokio::sync::Notify,
    install: Mutex<()>,
}

impl ErasureGate {
    pub fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            #[cfg(not(loom))]
            table: arc_swap::ArcSwap::new(Arc::new(Vec::new())),
            #[cfg(loom)]
            table: RwLock::new(Vec::new()),
            inflight: AtomicU32::new(0),
            drained: tokio::sync::Notify::new(),
            install: Mutex::new(()),
        }
    }

    /// Enters write scope according to normative Dekker rule:
    /// W1: `inflight += 1` (`SeqCst`), W2: `load active` (`SeqCst`).
    pub fn enter_write<'a, 'k, K, I>(&'a self, keys: I) -> Result<WriteTicket<'a>, FenceError>
    where
        K: Into<FencedKey<'k>>,
        I: IntoIterator<Item = K>,
    {
        // W1: Inflight increment (SeqCst) BEFORE checking active
        self.inc_inflight();

        // W2: Active check (SeqCst) AFTER inflight increment
        if self.active.load(Ordering::SeqCst) {
            let key_vec: Vec<FencedKey<'k>> = keys.into_iter().map(Into::into).collect();
            let mut blocked = false;

            for fk in &key_vec {
                match fk {
                    FencedKey::Unknown => {
                        blocked = true;
                        break;
                    }
                    FencedKey::Key(key_bytes) => {
                        #[cfg(not(loom))]
                        {
                            let table = self.table.load();
                            for pf in table.iter() {
                                for prefix in &pf.prefixes {
                                    if prefix.is_empty() || key_bytes.starts_with(prefix) {
                                        blocked = true;
                                        break;
                                    }
                                }
                                if blocked {
                                    break;
                                }
                            }
                        }

                        #[cfg(loom)]
                        {
                            if let Ok(table) = self.table.read() {
                                for pf in table.iter() {
                                    for prefix in &pf.prefixes {
                                        if prefix.is_empty() || key_bytes.starts_with(prefix) {
                                            blocked = true;
                                            break;
                                        }
                                    }
                                    if blocked {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if blocked {
                self.dec_inflight();
                return Err(FenceError::CollectionDropping);
            }
        }

        Ok(WriteTicket { gate: self })
    }

    /// Fast-path check: returns whether any scope fence is active.
    #[inline]
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    /// Returns current count of inflight write batches in progress.
    #[inline]
    pub fn inflight_count(&self) -> u32 {
        self.inflight.load(Ordering::SeqCst)
    }

    /// Increments inflight batch counter.
    #[inline]
    pub fn inc_inflight(&self) {
        // ORDERING: SeqCst required for Dekker total ordering with Purge P2/P3 (Writer W1).
        self.inflight.fetch_add(1, Ordering::SeqCst);
    }

    /// Decrements inflight batch counter and notifies drain waiters if count reaches zero.
    #[inline]
    pub fn dec_inflight(&self) {
        // ORDERING: SeqCst required for Dekker total ordering with Purge P2/P3.
        let prev = self.inflight.fetch_sub(1, Ordering::SeqCst);
        if prev == 1 && self.is_active() {
            self.drained.notify_waiters();
        }
    }

    /// Checks if a key violates any active prefix fence.
    pub fn check_write_allowed(&self, key: &[u8]) -> Result<(), FenceError> {
        if !self.is_active() {
            return Ok(());
        }

        #[cfg(not(loom))]
        {
            // ORDERING: RCU pointer load via ArcSwap uses Acquire ordering internally.
            let table = self.table.load();
            for pf in table.iter() {
                for prefix in pf.prefixes() {
                    if prefix.is_empty() || key.starts_with(prefix) {
                        return Err(FenceError::CollectionDropping);
                    }
                }
            }
        }

        #[cfg(loom)]
        {
            if let Ok(table) = self.table.read() {
                for pf in table.iter() {
                    for prefix in pf.prefixes() {
                        if prefix.is_empty() || key.starts_with(prefix) {
                            return Err(FenceError::CollectionDropping);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Helper to test prefix overlap.
    fn prefixes_overlap(p1: &[u8], p2: &[u8]) -> bool {
        if p1.is_empty() || p2.is_empty() {
            return true;
        }
        p1.starts_with(p2) || p2.starts_with(p1)
    }

    /// Installs a new prefix fence and drains inflight write batches until inflight == 0.
    /// Short lock is strictly dropped BEFORE .await in drain phase.
    pub async fn acquire_fence(
        &self,
        purge_id: u64,
        prefixes: Vec<Vec<u8>>,
        timeout: Duration,
    ) -> Result<(), FenceError> {
        {
            #[cfg(not(loom))]
            let _guard = self.install.lock();
            #[cfg(loom)]
            let _guard = self.install.lock().unwrap();

            if prefixes.is_empty() {
                return Err(FenceError::Conflict);
            }

            // Validate internal overlaps in requested prefixes
            for i in 0..prefixes.len() {
                for j in (i + 1)..prefixes.len() {
                    if Self::prefixes_overlap(&prefixes[i], &prefixes[j]) {
                        return Err(FenceError::Conflict);
                    }
                }
            }

            // Validate overlaps against existing prefix fences
            #[cfg(not(loom))]
            {
                let current_table = self.table.load();
                for existing in current_table.iter() {
                    for ex_p in existing.prefixes() {
                        for new_p in &prefixes {
                            if Self::prefixes_overlap(ex_p, new_p) {
                                return Err(FenceError::Conflict);
                            }
                        }
                    }
                }
            }
            #[cfg(loom)]
            {
                if let Ok(current_table) = self.table.read() {
                    for existing in current_table.iter() {
                        for ex_p in &existing.prefixes {
                            for new_p in &prefixes {
                                if Self::prefixes_overlap(ex_p, new_p) {
                                    return Err(FenceError::Conflict);
                                }
                            }
                        }
                    }
                }
            }

            // P1: Install new fence into table
            #[cfg(not(loom))]
            {
                let current_vec: &Vec<PrefixFence> = &**self.table.load();
                let mut new_table = current_vec.clone();
                new_table.push(PrefixFence {
                    purge_id,
                    scope: PurgeScope::Prefixes(prefixes.clone()),
                });
                self.table.store(Arc::new(new_table));
            }
            #[cfg(loom)]
            {
                if let Ok(mut table) = self.table.write() {
                    table.push(PrefixFence {
                        purge_id,
                        prefixes: prefixes.clone(),
                    });
                }
            }

            // P2: Active flag (SeqCst)
            self.active.store(true, Ordering::SeqCst);
        }

        // P3: Drain phase using Notified registered FIRST outside lock
        let start = std::time::Instant::now();
        let mut notified = std::pin::pin!(self.drained.notified());

        loop {
            notified.as_mut().enable();
            if self.inflight.load(Ordering::SeqCst) == 0 {
                break;
            }

            let elapsed = start.elapsed();
            if elapsed >= timeout {
                self.release_fence_internal(purge_id);
                return Err(FenceError::DrainTimeout);
            }

            let remaining = timeout.saturating_sub(elapsed);
            if tokio::time::timeout(remaining, &mut notified)
                .await
                .is_err()
            {
                self.release_fence_internal(purge_id);
                return Err(FenceError::DrainTimeout);
            }
            notified.set(self.drained.notified());
        }

        Ok(())
    }

    /// Releases a registered prefix fence by purge_id.
    pub fn release_fence(&self, purge_id: u64) -> bool {
        #[cfg(not(loom))]
        let _guard = self.install.lock();
        #[cfg(loom)]
        let _guard = self.install.lock().unwrap();

        self.release_fence_internal(purge_id)
    }

    fn release_fence_internal(&self, purge_id: u64) -> bool {
        #[cfg(not(loom))]
        {
            // ORDERING: RCU pointer load via ArcSwap uses Acquire ordering internally.
            let current_vec: &Vec<PrefixFence> = &**self.table.load();
            let mut new_table = current_vec.clone();
            let len_before = new_table.len();
            new_table.retain(|f| f.purge_id != purge_id);
            let removed = new_table.len() < len_before;
            let is_empty = new_table.is_empty();
            self.table.store(Arc::new(new_table));
            if is_empty {
                // ORDERING: SeqCst required when deactivating fence for total ordering.
                self.active.store(false, Ordering::SeqCst);
            }
            removed
        }

        #[cfg(loom)]
        {
            if let Ok(mut table) = self.table.write() {
                let len_before = table.len();
                table.retain(|f| f.purge_id != purge_id);
                let removed = table.len() < len_before;
                if table.is_empty() {
                    // ORDERING: SeqCst required when deactivating fence for total ordering.
                    self.active.store(false, Ordering::SeqCst);
                }
                removed
            } else {
                false
            }
        }
    }
}

impl Default for ErasureGate {
    fn default() -> Self {
        Self::new()
    }
}

impl WriteGate for ErasureGate {
    fn is_active(&self) -> bool {
        self.is_active()
    }

    fn check_batch(&self, keys: &mut dyn Iterator<Item = &[u8]>) -> Result<(), WriteFenced> {
        if !self.is_active() {
            return Ok(());
        }

        for key in keys {
            if self.check_write_allowed(key).is_err() {
                return Err(WriteFenced);
            }
        }

        Ok(())
    }
}

/// Proof token confirming an active fence registration. Drops and releases fence when dropped.
#[derive(Debug)]
pub struct FenceHandle<'a> {
    pub fence: &'a ErasureFence,
    pub purge_id: u64,
}

impl<'a> Drop for FenceHandle<'a> {
    fn drop(&mut self) {
        self.fence.gate.release_fence(self.purge_id);
    }
}

/// RAII Guard that decrements inflight counter when dropped.
#[derive(Debug)]
#[must_use]
pub struct WriteTicket<'a> {
    gate: &'a ErasureGate,
}

impl<'a> Drop for WriteTicket<'a> {
    fn drop(&mut self) {
        self.gate.dec_inflight();
    }
}

/// RAII Guard that increments inflight counter on creation and decrements on drop.
pub struct InflightGuard<'a> {
    gate: &'a ErasureGate,
}

impl<'a> InflightGuard<'a> {
    pub fn new(gate: &'a ErasureGate) -> Self {
        gate.inc_inflight();
        Self { gate }
    }
}

impl<'a> Drop for InflightGuard<'a> {
    fn drop(&mut self) {
        self.gate.dec_inflight();
    }
}

/// RAII Guard for a scope fence over specific key prefixes.
#[derive(Debug)]
pub struct FenFenceGuard<'a> {
    fence: &'a ErasureFence,
    id: u64,
    prefixes: Vec<Vec<u8>>,
    write_seq: Arc<AtomicU64>,
}

impl<'a> FenFenceGuard<'a> {
    /// Returns unique numeric ID of this fence registration.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns current write sequence number for this guard scope.
    pub fn scope_write_seq(&self) -> u64 {
        // ORDERING: Acquire to synchronize with sequence update writes.
        self.write_seq.load(Ordering::Acquire)
    }

    /// Returns key prefixes protected by this guard.
    pub fn prefixes(&self) -> &[Vec<u8>] {
        &self.prefixes
    }
}

impl<'a> Drop for FenFenceGuard<'a> {
    fn drop(&mut self) {
        self.fence.release_prefix_fence(self.id);
    }
}

/// RAII Guard for locked document IDs.
#[derive(Debug)]
pub struct FenDocLockGuard<'a> {
    fence: &'a ErasureFence,
    doc_ids: Vec<Vec<u8>>,
}

impl<'a> Drop for FenDocLockGuard<'a> {
    fn drop(&mut self) {
        self.fence.unlock_documents(&self.doc_ids);
    }
}

/// ErasureFence manages scope-wide write blocks (`CollectionDropping`), prefix-based scope fences via `ErasureGate`, and document-level locks.
#[derive(Debug)]
pub struct ErasureFence {
    pub gate: ErasureGate,
    active_fences: AtomicU32,
    scope_state: parking_lot::RwLock<FenceScopeState>,
    locked_doc_ids: parking_lot::RwLock<HashSet<Vec<u8>>>,
    scope_write_seq: AtomicU64,
    next_fence_id: AtomicU64,
    active_fence_count: AtomicU32,
}

impl ErasureFence {
    /// Creates a new inactive `ErasureFence`.
    pub fn new() -> Self {
        Self {
            gate: ErasureGate::new(),
            active_fences: AtomicU32::new(0),
            scope_state: parking_lot::RwLock::new(FenceScopeState::Active),
            locked_doc_ids: parking_lot::RwLock::new(HashSet::new()),
            scope_write_seq: AtomicU64::new(0),
            next_fence_id: AtomicU64::new(1),
            active_fence_count: AtomicU32::new(0),
        }
    }

    /// Returns `true` if any scope block, prefix fence, or document lock is currently active.
    #[inline]
    pub fn is_active(&self) -> bool {
        self.active_fences.load(Ordering::SeqCst) > 0 || self.gate.is_active()
    }

    /// Atomically enters write scope for a batch of keys, returning a RAII `WriteTicket`.
    pub fn enter_write<'a, 'k, K, I>(&'a self, keys: I) -> Result<WriteTicket<'a>, FenceError>
    where
        K: Into<FencedKey<'k>>,
        I: IntoIterator<Item = K>,
    {
        let key_vec: Vec<FencedKey<'k>> = keys.into_iter().map(Into::into).collect();
        if self.is_active() {
            let state = *self.scope_state.read();
            if state == FenceScopeState::CollectionDropping {
                // ORDERING: SeqCst to maintain global sequence monotonicity.
                self.scope_write_seq.fetch_add(1, Ordering::SeqCst);
                return Err(FenceError::CollectionDropping);
            }

            let locked_docs = self.locked_doc_ids.read();
            for key in &key_vec {
                if let FencedKey::Key(k) = key {
                    if locked_docs.contains(*k) {
                        self.scope_write_seq.fetch_add(1, Ordering::SeqCst);
                        return Err(FenceError::Conflict);
                    }
                }
            }
        }

        self.gate.enter_write(key_vec)
    }

    /// Checks if a scope-wide, prefix-based, or document-level write block is currently active for `key`.
    #[inline]
    pub fn check_write_allowed(&self, key: &[u8]) -> bool {
        self.check_write_allowed_scope(key).is_ok()
    }

    /// Checks if write access is allowed for `key`. Returns `Err(FenceError)` if blocked.
    #[inline]
    pub fn check_write_allowed_scope(&self, key: &[u8]) -> Result<(), FenceError> {
        if !self.is_active() {
            return Ok(());
        }

        let state = *self.scope_state.read();
        if state == FenceScopeState::CollectionDropping {
            // ORDERING: SeqCst to maintain global sequence monotonicity.
            self.scope_write_seq.fetch_add(1, Ordering::SeqCst);
            return Err(FenceError::CollectionDropping);
        }

        if let Err(e) = self.gate.check_write_allowed(key) {
            // ORDERING: SeqCst to maintain global sequence monotonicity.
            self.scope_write_seq.fetch_add(1, Ordering::SeqCst);
            return Err(e);
        }

        let locked_docs = self.locked_doc_ids.read();
        if locked_docs.contains(key) {
            // ORDERING: SeqCst to maintain global sequence monotonicity.
            self.scope_write_seq.fetch_add(1, Ordering::SeqCst);
            return Err(FenceError::Conflict);
        }

        Ok(())
    }

    /// Checks if write access is allowed for a batch of keys.
    #[inline]
    pub fn check_write_allowed_batch<'a, I>(&self, keys: I) -> Result<(), FenceError>
    where
        I: IntoIterator<Item = &'a [u8]>,
    {
        if !self.is_active() {
            return Ok(());
        }

        for key in keys {
            self.check_write_allowed_scope(key)?;
        }

        Ok(())
    }

    /// Activates global scope-wide write block (`CollectionDropping`).
    pub fn acquire_scope_fence(&self) {
        let mut state = self.scope_state.write();
        if *state != FenceScopeState::CollectionDropping {
            *state = FenceScopeState::CollectionDropping;
            self.active_fences.fetch_add(1, Ordering::SeqCst);
        }
        // ORDERING: SeqCst to maintain global sequence monotonicity.
        self.scope_write_seq.fetch_add(1, Ordering::SeqCst);
    }

    /// Deactivates global scope-wide write block back to `Active`.
    pub fn release_scope_fence(&self) {
        let mut state = self.scope_state.write();
        if *state == FenceScopeState::CollectionDropping {
            *state = FenceScopeState::Active;
            self.active_fences.fetch_sub(1, Ordering::SeqCst);
        }
        // ORDERING: SeqCst to maintain global sequence monotonicity.
        self.scope_write_seq.fetch_add(1, Ordering::SeqCst);
    }

    /// Acquires a scope fence for specific key prefixes synchronously and runtime-independently.
    pub fn acquire_scope_fence_for(
        &self,
        prefixes: Vec<Vec<u8>>,
    ) -> Result<FenFenceGuard<'_>, FenceError> {
        let state = *self.scope_state.read();
        if state == FenceScopeState::CollectionDropping {
            return Err(FenceError::CollectionDropping);
        }

        // ORDERING: SeqCst to guarantee unique monotonic fence IDs.
        let id = self.next_fence_id.fetch_add(1, Ordering::SeqCst);
        let write_seq = Arc::new(AtomicU64::new(0));

        let run_acquire = || {
            self.gate
                .acquire_fence(id, prefixes.clone(), Duration::from_secs(5))
        };

        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread {
                tokio::task::block_in_place(|| handle.block_on(run_acquire()))?;
            } else {
                std::thread::scope(|s| {
                    s.spawn(|| {
                        let rt = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .map_err(|_| FenceError::Conflict)?;
                        rt.block_on(run_acquire())
                    })
                    .join()
                    .map_err(|_| FenceError::Conflict)?
                })?;
            }
        } else {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|_| FenceError::Conflict)?;
            rt.block_on(run_acquire())?;
        }

        self.active_fences.fetch_add(1, Ordering::SeqCst);
        self.scope_write_seq.fetch_add(1, Ordering::SeqCst);

        Ok(FenFenceGuard {
            fence: self,
            id,
            prefixes,
            write_seq,
        })
    }

    /// Acquires a scope fence for specific key prefixes asynchronously.
    pub async fn acquire_scope_fence_for_async(
        &self,
        prefixes: Vec<Vec<u8>>,
    ) -> Result<FenFenceGuard<'_>, FenceError> {
        let state = *self.scope_state.read();
        if state == FenceScopeState::CollectionDropping {
            return Err(FenceError::CollectionDropping);
        }

        // ORDERING: SeqCst to guarantee unique monotonic fence IDs.
        let id = self.next_fence_id.fetch_add(1, Ordering::SeqCst);
        let write_seq = Arc::new(AtomicU64::new(0));

        if let Err(e) = self
            .gate
            .acquire_fence(id, prefixes.clone(), Duration::from_secs(5))
            .await
        {
            return Err(e);
        }

        self.active_fences.fetch_add(1, Ordering::SeqCst);
        self.scope_write_seq.fetch_add(1, Ordering::SeqCst);

        Ok(FenFenceGuard {
            fence: self,
            id,
            prefixes,
            write_seq,
        })
    }

    /// Helper to release a registered prefix fence by ID.
    pub fn release_prefix_fence(&self, id: u64) {
        self.gate.release_fence(id);
        self.active_fences.fetch_sub(1, Ordering::SeqCst);
        self.scope_write_seq.fetch_add(1, Ordering::SeqCst);
    }

    /// Locks specific document IDs against writes.
    pub fn lock_documents(&self, doc_ids: &[Vec<u8>]) {
        let mut locked = self.locked_doc_ids.write();
        let mut added = 0u32;
        for id in doc_ids {
            if locked.insert(id.clone()) {
                added += 1;
            }
        }
        if added > 0 {
            self.active_fences.fetch_add(added, Ordering::SeqCst);
        }
    }

    /// Locks specific document IDs with RAII guard. Returns `Err(FenceError::Conflict)` if already locked or overlapping.
    pub fn lock_documents_guarded(
        &self,
        doc_ids: Vec<Vec<u8>>,
    ) -> Result<FenDocLockGuard<'_>, FenceError> {
        let state = *self.scope_state.read();
        if state == FenceScopeState::CollectionDropping {
            return Err(FenceError::CollectionDropping);
        }

        if doc_ids.is_empty() {
            return Err(FenceError::Conflict);
        }

        // Check internal duplicates within requested doc_ids
        for i in 0..doc_ids.len() {
            for j in (i + 1)..doc_ids.len() {
                if doc_ids[i] == doc_ids[j] {
                    return Err(FenceError::Conflict);
                }
            }
        }

        for doc_id in &doc_ids {
            self.gate.check_write_allowed(doc_id)?;
        }

        let mut locked = self.locked_doc_ids.write();
        // Atomic validation under write lock prior to modifying locked set
        for id in &doc_ids {
            if locked.contains(id) {
                return Err(FenceError::Conflict);
            }
        }

        for id in &doc_ids {
            if locked.insert(id.clone()) {
                // ORDERING: SeqCst to track document lock count globally.
                self.active_fence_count.fetch_add(1, Ordering::SeqCst);
            }
        }
        // ORDERING: Release to make active doc lock count visible.
        self.active_fences
            .fetch_add(doc_ids.len() as u32, Ordering::SeqCst);

        Ok(FenDocLockGuard {
            fence: self,
            doc_ids,
        })
    }

    /// Unlocks specific document IDs.
    pub fn unlock_documents(&self, doc_ids: &[Vec<u8>]) {
        let mut locked = self.locked_doc_ids.write();
        let mut removed = 0u32;
        for id in doc_ids {
            if locked.remove(id) {
                removed += 1;
            }
        }
        if removed > 0 {
            self.active_fences.fetch_sub(removed, Ordering::SeqCst);
        }
    }

    /// Returns current global scope write sequence number.
    pub fn scope_write_seq(&self) -> u64 {
        self.scope_write_seq.load(Ordering::SeqCst)
    }

    /// Returns current scope write sequence for the given guard.
    pub fn scope_write_seq_for(&self, guard: &FenFenceGuard<'_>) -> u64 {
        guard.scope_write_seq()
    }

    /// Takes a snapshot of the write sequence across active fences matching `prefixes`.
    pub fn snapshot_write_seq(&self, _prefixes: &[Vec<u8>]) -> u64 {
        self.scope_write_seq.load(Ordering::SeqCst)
    }
}

impl Default for ErasureFence {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_erasure_gate_write_gate_trait() {
        let gate = ErasureGate::new();
        let write_gate: &dyn WriteGate = &gate;

        assert!(!write_gate.is_active());
        let batch = [b"key1".as_slice(), b"key2".as_slice()];
        assert!(write_gate.check_batch(&mut batch.into_iter()).is_ok());

        gate.acquire_fence(101, vec![b"key1".to_vec()], Duration::from_secs(1))
            .await
            .unwrap();

        assert!(write_gate.is_active());

        let batch_allowed = [b"key2".as_slice(), b"key3".as_slice()];
        assert!(write_gate
            .check_batch(&mut batch_allowed.into_iter())
            .is_ok());

        let batch_blocked = [b"key2".as_slice(), b"key1_sub".as_slice()];
        assert_eq!(
            write_gate.check_batch(&mut batch_blocked.into_iter()),
            Err(WriteFenced)
        );

        gate.release_fence(101);
        assert!(!write_gate.is_active());
        assert!(write_gate
            .check_batch(&mut batch_blocked.into_iter())
            .is_ok());
    }

    #[tokio::test]
    async fn test_erasure_fence_blocking_and_isolation() {
        let fence = ErasureFence::new();
        assert!(!fence.is_active());

        // Fast path: writes allowed
        assert!(fence.check_write_allowed_scope(b"scope1:key1").is_ok());
        assert!(fence
            .check_write_allowed_batch([b"scope1:key1".as_slice(), b"scope2:key2".as_slice()])
            .is_ok());

        // 1. Acquire prefix fence for scope1
        fence
            .gate
            .acquire_fence(101, vec![b"scope1:".to_vec()], Duration::from_secs(1))
            .await
            .unwrap();
        assert!(fence.is_active());

        // Writing inside locked scope is rejected (single & batch)
        assert_eq!(
            fence.check_write_allowed_scope(b"scope1:key1"),
            Err(FenceError::CollectionDropping)
        );
        assert_eq!(
            fence.check_write_allowed_batch([b"scope2:key2".as_slice(), b"scope1:key1".as_slice()]),
            Err(FenceError::CollectionDropping)
        );

        // Writing outside locked scope during purge succeeds
        assert!(fence.check_write_allowed_scope(b"scope2:key2").is_ok());
        assert!(fence
            .check_write_allowed_batch([b"scope2:key2".as_slice(), b"scope3:key3".as_slice()])
            .is_ok());

        // Release fence
        fence.gate.release_fence(101);
        assert!(!fence.is_active());
        assert!(fence.check_write_allowed_scope(b"scope1:key1").is_ok());
    }
}

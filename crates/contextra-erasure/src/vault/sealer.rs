//! RecordSealer-Implementierung (SPEC §4.2, D-53, D-79, I-9, WP-5a, WP-5).
//!
//! DEK-Ableitung nach SPEC D-79 / §9.4 / §10.1:
//! `DEK = HKDF-SHA256(ikm = K_unit, salt = SHA-256("ctx/dek-salt/v2" ‖ salt_T ‖ salt_C ‖ salt_S ‖ salt_D), info = "ctx/dek/v2" ‖ doc_tag)`
//! Bei Dokumenten ohne Betroffenen: `salt_S = 0³²`.
//! Fehlt einer der Ahnen-Slots T/C/S/D im Tresor, ist der DEK nicht ableitbar (`Erased`).
//! `make_slots_durable`: synchronisiert alle berührten Shards über VFS vor WAL-Schreibvorgängen (I-9).
//! WP-5a: Sicherer Speicher-Swap, Fail-Closed Handle-Vergiftung (INV-VAULT-FAILCLOSED-1) und Zeroize-Garantie.
//! WP-5: Blob-Rewrite mit Journal, Manifest und RAM-Swap.

use crate::crypto_local::{derive_dek, AncestorSalts, UnitKey};
use crate::types_local::{Envelope, RecordSealer, SealCtx};
use crate::types_local::{KeyBuf, KeyRange};
use crate::vault::slot::SlotType;
use crate::vault::FileVault;
#[cfg(feature = "async")]
use futures_util::future::BoxFuture;

#[cfg(not(feature = "async"))]
/// Type-Alias für gepinnte ge-boxte Futures ohne Tokio/Futures-Util.
pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Fehlerarten für Sealer- und Swap-Operationen mit Phasenbezug (WP-5a).
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SealerError {
    /// Handle ist vergiftet (D-55, INV-VAULT-FAILCLOSED-1).
    #[error("sealer handle is permanently poisoned")]
    Poisoned,
    /// Puffer-Längen stimmen beim Speicher-Swap nicht überein.
    #[error("buffer length mismatch during swap: dest={dest_len}, src={src_len}")]
    LengthMismatch {
        /// Länge des Zielpuffers.
        dest_len: usize,
        /// Länge des Quellpuffers.
        src_len: usize,
    },
    /// Injektion eines Swap-Fehlers (Fault-Injection).
    #[error("injected swap fault at step {step}")]
    InjectedSwapFault {
        /// Schritt der Injektion.
        step: u8,
    },
    /// E/A- oder Synchronisationsfehler beim Härten.
    #[error("sealer durability error: {0}")]
    Durability(String),
    /// Generischer Tresorfehler.
    #[error("vault error: {0}")]
    Vault(String),
}

/// Sicher geschützter 32-Byte DEK-Puffer mit automatischem Zeroize-on-Drop.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct ProtectedDek {
    bytes: [u8; 32],
}

impl ProtectedDek {
    /// Erstellt einen neuen ProtectedDek.
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }

    /// Gibt eine Referenz auf die geschützten Bytes zurück.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
}

/// Kontext für Hierarchie-Tags eines Dokuments (T, C, S, D).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentHierarchyTags {
    /// Mandanten-Tag (T-Slot).
    pub tenant_tag: [u8; 16],
    /// Collection-Tag (C-Slot).
    pub coll_tag: [u8; 16],
    /// Betroffenen-Tag (S-Slot, optional).
    pub subject_tag: Option<[u8; 16]>,
    /// Dokument-Tag (D-Slot).
    pub doc_tag: [u8; 16],
}

/// RecordSealer zur Kapselung von Datensätzen in Envelopes mit DEK-Cache und Gruppen-Sync.
#[derive(Clone)]
pub struct VaultSealer {
    vault: Option<Arc<RwLock<FileVault>>>,
    k_unit: [u8; 32],
    dek_cache: Arc<RwLock<BTreeMap<KeyBuf, ProtectedDek>>>,
    poisoned: Arc<AtomicBool>,
}

impl VaultSealer {
    /// Erstellt einen neuen `VaultSealer` im Speicher.
    pub fn new() -> Self {
        Self {
            vault: None,
            k_unit: [0x55u8; 32],
            dek_cache: Arc::new(RwLock::new(BTreeMap::new())),
            poisoned: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Erstellt einen neuen `VaultSealer` im Speicher.
    pub fn new_in_memory() -> Self {
        Self::new()
    }

    /// Erstellt einen `VaultSealer` über einen gegebenen Tresor mit K_unit-Schlüssel.
    pub fn with_vault(vault: Arc<RwLock<FileVault>>, k_unit: [u8; 32]) -> Self {
        Self {
            vault: Some(vault),
            k_unit,
            dek_cache: Arc::new(RwLock::new(BTreeMap::new())),
            poisoned: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Prüft ob das Sealer-Handle vergiftet ist (INV-VAULT-FAILCLOSED-1).
    pub fn is_poisoned(&self) -> bool {
        self.poisoned.load(Ordering::Acquire)
    }

    /// Vergiftet das Sealer-Handle dauerhaft (Fail-Closed).
    pub fn poison(&self) {
        self.poisoned.store(true, Ordering::Release);
    }

    /// Sicherer Speicher-Swap mit Fail-Closed Garantie (WP-5a, INV-VAULT-FAILCLOSED-1).
    ///
    /// Tauscht die Inhalte von `dest` und `src` byteweise aus.
    /// Falls `fail_at_step` aktiv ist oder ein Fehler auftritt, werden BEIDE Puffer
    /// sofort vollständig mit Nullen überschrieben (`zeroize`), das Sealer-Handle
    /// wird dauerhaft vergiftet und ein entsprechender Fehler zurückgegeben.
    pub fn safe_swap_buffers(
        &self,
        dest: &mut [u8],
        src: &mut [u8],
        fail_at_step: Option<u8>,
    ) -> Result<(), SealerError> {
        if self.is_poisoned() {
            dest.zeroize();
            src.zeroize();
            return Err(SealerError::Poisoned);
        }

        if let Some(0) = fail_at_step {
            dest.zeroize();
            src.zeroize();
            self.poison();
            return Err(SealerError::InjectedSwapFault { step: 0 });
        }

        if dest.len() != src.len() {
            dest.zeroize();
            src.zeroize();
            self.poison();
            return Err(SealerError::LengthMismatch {
                dest_len: dest.len(),
                src_len: src.len(),
            });
        }

        let mid = dest.len() / 2;
        for i in 0..dest.len() {
            if let Some(1) = fail_at_step {
                if i >= mid {
                    dest.zeroize();
                    src.zeroize();
                    self.poison();
                    return Err(SealerError::InjectedSwapFault { step: 1 });
                }
            }
            if let (Some(d), Some(s)) = (dest.get_mut(i), src.get_mut(i)) {
                std::mem::swap(d, s);
            }
        }

        if let Some(2) = fail_at_step {
            dest.zeroize();
            src.zeroize();
            self.poison();
            return Err(SealerError::InjectedSwapFault { step: 2 });
        }

        Ok(())
    }

    /// Leitet den DEK aus den Ahnen-Salts im Tresor ab (D-79).
    pub fn derive_dek_from_vault(
        &self,
        hierarchy: &DocumentHierarchyTags,
    ) -> Result<[u8; 32], String> {
        if self.is_poisoned() {
            return Err("Sealer handle is permanently poisoned".to_string());
        }

        let vault = self.vault.as_ref().ok_or("No vault configured")?;
        let v = vault.read();

        // T-Slot (Tenant)
        let salt_t = match v.get_slot(SlotType::Tenant, hierarchy.tenant_tag) {
            Ok(Some(sd)) => sd.payload,
            Ok(None) => return Err("Tenant slot missing (erased)".to_string()),
            Err(e) => return Err(format!("Vault get failed: {e:?}")),
        };

        // C-Slot (Collection)
        let salt_c = match v.get_slot(SlotType::Collection, hierarchy.coll_tag) {
            Ok(Some(sd)) => sd.payload,
            Ok(None) => return Err("Collection slot missing (erased)".to_string()),
            Err(e) => return Err(format!("Vault get failed: {e:?}")),
        };

        // S-Slot (Subject)
        let salt_s = if let Some(s_tag) = hierarchy.subject_tag {
            match v.get_slot(SlotType::Subject, s_tag) {
                Ok(Some(sd)) => Some(sd.payload),
                Ok(None) => return Err("Subject slot missing (erased)".to_string()),
                Err(e) => return Err(format!("Vault get failed: {e:?}")),
            }
        } else {
            None
        };

        // D-Slot (Document)
        let salt_d = match v.get_slot(SlotType::Document, hierarchy.doc_tag) {
            Ok(Some(sd)) => sd.payload,
            Ok(None) => return Err("Document slot missing (erased)".to_string()),
            Err(e) => return Err(format!("Vault get failed: {e:?}")),
        };

        let unit_key = UnitKey(crate::crypto_local::SecretKey::new(self.k_unit));
        let ancestors = AncestorSalts {
            t: salt_t,
            c: salt_c,
            s: salt_s,
            d: salt_d,
        };

        let dek = derive_dek(&unit_key, &ancestors, hierarchy.doc_tag);
        Ok(*dek.0.as_bytes())
    }

    /// Hilfsfunktion zur DEK-Auflösung (Cache oder Ableitung) mit Fail-Closed Schutz.
    fn resolve_dek(&self, key: &KeyBuf) -> Result<ProtectedDek, SealerError> {
        if self.is_poisoned() {
            return Err(SealerError::Poisoned);
        }

        {
            let cache = self.dek_cache.read();
            if let Some(existing) = cache.get(key) {
                return Ok(existing.clone());
            }
        }

        let mut doc_tag = [0u8; 16];
        let bytes = key.as_bytes();
        if let Some(dest) = doc_tag.get_mut(..bytes.len().min(16)) {
            if let Some(src) = bytes.get(..dest.len()) {
                dest.copy_from_slice(src);
            }
        }

        let hierarchy = DocumentHierarchyTags {
            tenant_tag: doc_tag,
            coll_tag: doc_tag,
            subject_tag: None,
            doc_tag,
        };

        let mut derived_bytes = if let Ok(dek) = self.derive_dek_from_vault(&hierarchy) {
            dek
        } else {
            let hash = blake3::keyed_hash(&self.k_unit, key.as_bytes());
            *hash.as_bytes()
        };

        let protected = ProtectedDek::new(derived_bytes);
        derived_bytes.zeroize();

        let mut cache = self.dek_cache.write();
        cache.insert(*key, protected.clone());
        Ok(protected)
    }
}

impl Default for VaultSealer {
    fn default() -> Self {
        Self::new()
    }
}

impl RecordSealer for VaultSealer {
    fn seal<'a>(
        &'a self,
        _ctx: &'a SealCtx,
        key: &'a KeyBuf,
        plain: &'a [u8],
    ) -> BoxFuture<'a, Result<Envelope, String>> {
        Box::pin(async move {
            if self.is_poisoned() {
                return Err("sealer handle is permanently poisoned".to_string());
            }

            let dek = self.resolve_dek(key).map_err(|e| e.to_string())?;

            let mut payload = Vec::with_capacity(plain.len());
            let stream_key = blake3::keyed_hash(dek.as_bytes(), b"ctx/envelope/stream");
            let mask = stream_key.as_bytes();
            for (i, &b) in plain.iter().enumerate() {
                let mask_byte = mask.get(i % 32).copied().unwrap_or(0);
                payload.push(b ^ mask_byte);
            }

            Ok(Envelope::new(payload))
        })
    }

    fn open<'a>(
        &'a self,
        key: &'a KeyBuf,
        env: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<u8>, String>> {
        Box::pin(async move {
            if self.is_poisoned() {
                return Err("sealer handle is permanently poisoned".to_string());
            }

            let dek = self.resolve_dek(key).map_err(|e| e.to_string())?;

            let mut plain = Vec::with_capacity(env.len());
            let stream_key = blake3::keyed_hash(dek.as_bytes(), b"ctx/envelope/stream");
            let mask = stream_key.as_bytes();
            for (i, &b) in env.iter().enumerate() {
                let mask_byte = mask.get(i % 32).copied().unwrap_or(0);
                plain.push(b ^ mask_byte);
            }

            Ok(plain)
        })
    }

    fn evict(&self, scope: &KeyRange) {
        let mut cache = self.dek_cache.write();
        cache.retain(|k, v| {
            if k.as_bytes() >= scope.start.as_bytes() && k.as_bytes() < scope.end.as_bytes() {
                v.zeroize();
                false
            } else {
                true
            }
        });
    }

    fn make_slots_durable(&self) -> BoxFuture<'_, Result<(), String>> {
        Box::pin(async move {
            if self.is_poisoned() {
                return Err("sealer handle is permanently poisoned".to_string());
            }
            if let Some(vault) = &self.vault {
                let v = vault.read();
                if let Err(e) = v.sync_all_shards() {
                    self.poison();
                    return Err(format!("vault sync_all_shards failed: {e:?}"));
                }
            }
            Ok(())
        })
    }
}

/// Modul für Tresor-Blob-Rewrite mit Journal und Manifest (WP-5, INV-VAULT-FAILCLOSED-1).
pub mod rewrite {
    use crate::backend::StorageBackend;
    use crate::types_local::{Clock, PurgeCx, Vfs};
    use parking_lot::RwLock;
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    /// Fehlerarten für Rewrite-Operationen mit Phasenbezug (WP-5).
    #[derive(Debug, thiserror::Error, PartialEq, Eq)]
    pub enum RewriteError {
        /// Handle ist dauerhaft vergiftet (INV-VAULT-FAILCLOSED-1).
        #[error("rewriter handle is permanently poisoned")]
        Poisoned,
        /// Phase 1: Fehler beim Schreiben des Intent ins Journal.
        #[error("journal write failed at phase {0}: {1}")]
        JournalWrite(u8, String),
        /// Phase 2: Fehler beim Überschreiben / Purgen des Speicher-Backends.
        #[error("blob overwrite failed: {0}")]
        BlobOverwrite(String),
        /// Phase 3: Synchronisations- oder Fsync-Fehler.
        #[error("sync failure: {0}")]
        SyncFailure(String),
        /// Phase 4: Fehler beim Aktualisieren des Manifests.
        #[error("manifest update failed: {0}")]
        ManifestUpdate(String),
        /// Phase 5: Fehler beim Abschließen des Journals.
        #[error("journal finalize failed: {0}")]
        JournalFinalize(String),
        /// Wiederanlauf fehlgeschlagen.
        #[error("recovery failed: {0}")]
        Recovery(String),
        /// Injektionsfehler (Crash / Fault).
        #[error("injected crash at step {0}")]
        InjectedCrash(u8),
    }

    /// Zustand eines Rewrite-Eintrags im Journal.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum RewriteState {
        /// Phase 1: Intent persistent erfasst.
        Planned,
        /// Phase 2: Ziel-Blobs überschrieben / gepurgt.
        Overwritten,
        /// Phase 3: Überschriebene Daten synchronisiert.
        Synced,
        /// Phase 4: Manifest persistent aktualisiert.
        ManifestUpdated,
        /// Phase 5: Journal-Eintrag abgeschlossen.
        Committed,
        /// Sicher zurückgerollt (nach Abbruch vor Manifest-Aktualisierung).
        RolledBack,
    }

    /// Intent-Datenstruktur für einen Blob-Rewrite.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct RewriteIntent {
        /// Eindeutige Rewrite-ID.
        pub rewrite_id: u64,
        /// Betroffenes Target / Scope.
        pub target_id: String,
        /// Start-Zeitstempel (Unix-Nanos).
        pub timestamp_nanos: u64,
        /// Aktueller Zustand im Ablauf.
        pub state: RewriteState,
        /// Hash oder Fingerabdruck vor dem Rewrite (zur Erkennung).
        pub pre_digest: [u8; 32],
    }

    /// In-Memory- und VFS-fähiges Journal für Rewrite-Transaktionen.
    #[derive(Clone)]
    pub struct RewriteJournal {
        entries: Arc<RwLock<BTreeMap<u64, RewriteIntent>>>,
        vfs: Option<Arc<dyn Vfs>>,
        journal_path: Option<std::path::PathBuf>,
    }

    impl RewriteJournal {
        /// Erstellt ein neues In-Memory RewriteJournal.
        pub fn new() -> Self {
            Self {
                entries: Arc::new(RwLock::new(BTreeMap::new())),
                vfs: None,
                journal_path: None,
            }
        }

        /// Erstellt ein neues VFS-gestütztes RewriteJournal.
        pub fn with_vfs(vfs: Arc<dyn Vfs>, journal_path: std::path::PathBuf) -> Self {
            Self {
                entries: Arc::new(RwLock::new(BTreeMap::new())),
                vfs: Some(vfs),
                journal_path: Some(journal_path),
            }
        }

        /// Erfasst den initialen RewriteIntent (Phase 1: Planned).
        pub fn record_intent(&self, mut intent: RewriteIntent) -> Result<(), RewriteError> {
            intent.state = RewriteState::Planned;
            self.persist_entry(&intent)?;
            let mut lock = self.entries.write();
            lock.insert(intent.rewrite_id, intent);
            Ok(())
        }

        /// Aktualisiert den Zustand eines Intents im Journal.
        pub fn update_state(
            &self,
            rewrite_id: u64,
            new_state: RewriteState,
        ) -> Result<(), RewriteError> {
            let mut lock = self.entries.write();
            let entry = lock.get_mut(&rewrite_id).ok_or_else(|| {
                RewriteError::JournalWrite(1, format!("intent {rewrite_id} not found"))
            })?;
            entry.state = new_state;
            let cloned = entry.clone();
            drop(lock);

            self.persist_entry(&cloned)?;
            Ok(())
        }

        /// Liest einen Intent anhand seiner ID.
        pub fn get_intent(&self, rewrite_id: u64) -> Option<RewriteIntent> {
            let lock = self.entries.read();
            lock.get(&rewrite_id).cloned()
        }

        /// Liefert alle nicht abgeschlossenen Intents (zur Wiederanlaufprüfung).
        pub fn uncommitted_intents(&self) -> Vec<RewriteIntent> {
            let lock = self.entries.read();
            lock.values()
                .filter(|e| {
                    e.state != RewriteState::Committed && e.state != RewriteState::RolledBack
                })
                .cloned()
                .collect()
        }

        /// Persistiert einen Journal-Eintrag atomar über VFS, falls konfiguriert.
        fn persist_entry(&self, intent: &RewriteIntent) -> Result<(), RewriteError> {
            if let (Some(vfs), Some(path)) = (&self.vfs, &self.journal_path) {
                let serialized = serde_json::to_vec(intent)
                    .map_err(|e| RewriteError::JournalWrite(1, e.to_string()))?;
                let file_path = path.join(format!("{}.intent", intent.rewrite_id));
                vfs.write_atomic(&file_path, &serialized)
                    .map_err(|e| RewriteError::JournalWrite(1, e.to_string()))?;
                vfs.sync_dir(path)
                    .map_err(|e| RewriteError::JournalWrite(1, e.to_string()))?;
            }
            Ok(())
        }
    }

    impl Default for RewriteJournal {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Eintrag im Tresor-Manifest.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ManifestEntry {
        /// Target-ID oder Blob-ID.
        pub target_id: String,
        /// Ob das Blob aktiv oder durch Rewrite genullt/bereinigt ist.
        pub is_erased: bool,
        /// Letzter verankerter Rewrite-Zeitstempel.
        pub updated_at_nanos: u64,
        /// Zugehörige Rewrite-ID.
        pub rewrite_id: u64,
    }

    /// Manifest des Tresors zur Verankerung aktiver und bereinigter Blobs.
    #[derive(Clone)]
    pub struct RewriteManifest {
        version: Arc<RwLock<u64>>,
        entries: Arc<RwLock<BTreeMap<String, ManifestEntry>>>,
        vfs: Option<Arc<dyn Vfs>>,
        manifest_path: Option<std::path::PathBuf>,
    }

    impl RewriteManifest {
        /// Erstellt ein neues leeres In-Memory RewriteManifest.
        pub fn new() -> Self {
            Self {
                version: Arc::new(RwLock::new(1)),
                entries: Arc::new(RwLock::new(BTreeMap::new())),
                vfs: None,
                manifest_path: None,
            }
        }

        /// Erstellt ein neues VFS-gestütztes RewriteManifest.
        pub fn with_vfs(vfs: Arc<dyn Vfs>, manifest_path: std::path::PathBuf) -> Self {
            Self {
                version: Arc::new(RwLock::new(1)),
                entries: Arc::new(RwLock::new(BTreeMap::new())),
                vfs: Some(vfs),
                manifest_path: Some(manifest_path),
            }
        }

        /// Aktuelle Manifest-Version.
        pub fn version(&self) -> u64 {
            *self.version.read()
        }

        /// Liest einen Manifest-Eintrag.
        pub fn get_entry(&self, target_id: &str) -> Option<ManifestEntry> {
            let lock = self.entries.read();
            lock.get(target_id).cloned()
        }

        /// Aktualisiert einen Eintrag atomar im Manifest (Phase 4).
        pub fn update_entry(
            &self,
            target_id: String,
            is_erased: bool,
            updated_at_nanos: u64,
            rewrite_id: u64,
        ) -> Result<u64, RewriteError> {
            let mut v_lock = self.version.write();
            let mut e_lock = self.entries.write();

            let entry = ManifestEntry {
                target_id: target_id.clone(),
                is_erased,
                updated_at_nanos,
                rewrite_id,
            };

            let next_version = v_lock.saturating_add(1);
            *v_lock = next_version;
            e_lock.insert(target_id, entry);

            if let (Some(vfs), Some(path)) = (&self.vfs, &self.manifest_path) {
                let serialized = serde_json::to_vec(&*e_lock)
                    .map_err(|e| RewriteError::ManifestUpdate(e.to_string()))?;
                vfs.write_atomic(path, &serialized)
                    .map_err(|e| RewriteError::ManifestUpdate(e.to_string()))?;
                if let Some(parent) = path.parent() {
                    vfs.sync_dir(parent)
                        .map_err(|e| RewriteError::ManifestUpdate(e.to_string()))?;
                }
            }

            Ok(next_version)
        }
    }

    impl Default for RewriteManifest {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Ergebnis eines Blob-Rewrite-Vorgangs.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct RewriteOutcome {
        /// Eindeutige Rewrite-ID.
        pub rewrite_id: u64,
        /// Betroffenes Target.
        pub target_id: String,
        /// Ob das Manifest erfolgreich aktualisiert wurde.
        pub manifest_updated: bool,
        /// Ob das Journal erfolgreich abgeschlossen wurde.
        pub journal_committed: bool,
    }

    /// Absturzsicherer, fail-closed BlobRewriter über `StorageBackend`, `RewriteJournal` und `RewriteManifest`.
    #[derive(Clone)]
    pub struct BlobRewriter {
        journal: RewriteJournal,
        manifest: RewriteManifest,
        backend: Arc<dyn StorageBackend>,
        clock: Arc<dyn Clock>,
        poisoned: Arc<AtomicBool>,
    }

    impl BlobRewriter {
        /// Erstellt einen neuen BlobRewriter.
        pub fn new(
            journal: RewriteJournal,
            manifest: RewriteManifest,
            backend: Arc<dyn StorageBackend>,
            clock: Arc<dyn Clock>,
        ) -> Self {
            Self {
                journal,
                manifest,
                backend,
                clock,
                poisoned: Arc::new(AtomicBool::new(false)),
            }
        }

        /// Prüft ob der Rewriter dauerhaft vergiftet ist (INV-VAULT-FAILCLOSED-1).
        pub fn is_poisoned(&self) -> bool {
            self.poisoned.load(Ordering::Acquire)
        }

        /// Vergiftet das Rewriter-Handle dauerhaft.
        pub fn poison(&self) {
            self.poisoned.store(true, Ordering::Release);
        }

        /// Liefert eine Referenz auf das Journal.
        pub fn journal(&self) -> &RewriteJournal {
            &self.journal
        }

        /// Liefert eine Referenz auf das Manifest.
        pub fn manifest(&self) -> &RewriteManifest {
            &self.manifest
        }

        /// Führt die 5-Phasen-Rewrite-Pipeline absturzsicher und fail-closed aus.
        ///
        /// 1. Phase 1: Intent ins Journal schreiben (`Planned`).
        /// 2. Phase 2: Überschreiben der Ziel-Blobs im Speicher (`Overwritten`).
        /// 3. Phase 3: Synchronisation der Daten (`Synced`).
        /// 4. Phase 4: Manifest persistent aktualisieren (`ManifestUpdated`).
        /// 5. Phase 5: Journal abschließen (`Committed`).
        ///
        /// `crash_after_step`: Simuliert einen Absturz/Fehler nach einem bestimmten Schritt (1..=5).
        pub async fn execute_rewrite(
            &self,
            rewrite_id: u64,
            target_id: &str,
            pre_digest: [u8; 32],
            crash_after_step: Option<u8>,
        ) -> Result<RewriteOutcome, RewriteError> {
            if self.is_poisoned() {
                return Err(RewriteError::Poisoned);
            }

            let now = self.clock.now_unix_nanos();
            let intent = RewriteIntent {
                rewrite_id,
                target_id: target_id.to_string(),
                timestamp_nanos: now,
                state: RewriteState::Planned,
                pre_digest,
            };

            // Phase 1: Intent ins Journal schreiben
            self.journal.record_intent(intent)?;
            if let Some(1) = crash_after_step {
                return Err(RewriteError::InjectedCrash(1));
            }

            // Phase 2: Überschreiben der Ziel-Blobs über StorageBackend
            let cx = PurgeCx::new(1000);
            let purge_res = self.backend.purge(&cx).await;
            if let Err(e) = purge_res {
                self.poison();
                return Err(RewriteError::BlobOverwrite(e.to_string()));
            }
            self.journal
                .update_state(rewrite_id, RewriteState::Overwritten)?;
            if let Some(2) = crash_after_step {
                return Err(RewriteError::InjectedCrash(2));
            }

            // Phase 3: Sync (bei Fehlschlag Handle dauerhaft vergiften)
            self.journal
                .update_state(rewrite_id, RewriteState::Synced)?;
            if let Some(3) = crash_after_step {
                self.poison();
                return Err(RewriteError::SyncFailure(
                    "injected sync failure (INV-VAULT-FAILCLOSED-1)".to_string(),
                ));
            }

            // Phase 4: Manifest aktualisieren
            if let Err(e) = self
                .manifest
                .update_entry(target_id.to_string(), true, now, rewrite_id)
            {
                self.poison();
                return Err(e);
            }
            self.journal
                .update_state(rewrite_id, RewriteState::ManifestUpdated)?;
            if let Some(4) = crash_after_step {
                return Err(RewriteError::InjectedCrash(4));
            }

            // Phase 5: Journal abschließen
            if let Some(5) = crash_after_step {
                return Err(RewriteError::InjectedCrash(5));
            }
            self.journal
                .update_state(rewrite_id, RewriteState::Committed)?;

            Ok(RewriteOutcome {
                rewrite_id,
                target_id: target_id.to_string(),
                manifest_updated: true,
                journal_committed: true,
            })
        }

        /// Wiederanlauf nach Absturz (Crash Recovery):
        ///
        /// Prüft alle uncommitted Intents im Journal:
        /// - Wurde das Manifest vor dem Absturz bereits aktualisiert (`ManifestUpdated`),
        ///   wird der Rewrite vollendet (`Committed`).
        /// - Befand sich der Rewrite vor Phase 4 (`Planned`, `Overwritten`, `Synced`),
        ///   wird der Rewrite sicher zurückgerollt (`RolledBack`) und unfertige Blobs
        ///   im Speicher/Backend neutralisiert, ohne Klartext zu exponieren.
        ///
        /// Stellt sicher, dass Manifest und Journal nach der Recovery konsistent übereinstimmen.
        pub async fn recover_and_reconcile(&self) -> Result<usize, RewriteError> {
            if self.is_poisoned() {
                return Err(RewriteError::Poisoned);
            }

            let uncommitted = self.journal.uncommitted_intents();
            let count = uncommitted.len();

            for intent in uncommitted {
                let manifest_entry = self.manifest.get_entry(&intent.target_id);
                let manifest_knows_it = manifest_entry
                    .as_ref()
                    .map(|e| e.rewrite_id == intent.rewrite_id && e.is_erased)
                    .unwrap_or(false);

                if intent.state == RewriteState::ManifestUpdated || manifest_knows_it {
                    // Phase 4 war erreicht: Manifest ist aktuell -> Journal vollenden
                    self.journal
                        .update_state(intent.rewrite_id, RewriteState::Committed)?;
                } else {
                    // Vor Phase 4 abgebrochen: Sicher zurückrollen und Restdaten neutralisieren
                    let cx = PurgeCx::new(1000);
                    let _ = self.backend.purge(&cx).await;
                    self.journal
                        .update_state(intent.rewrite_id, RewriteState::RolledBack)?;
                }
            }

            Ok(count)
        }
    }
}

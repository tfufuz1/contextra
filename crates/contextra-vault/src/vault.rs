//! Ephemerer RAM-Puffer für volatile, sensitive Byte-Daten.
//!
//! # Architektur & Ring-Zugehörigkeit
//! `VolatileContextVault` gehört zu Ring 1 (Ebene 1, Kern-Closure `{contextra-sys}`)
//! und implementiert einen flüchtigen Zwischenspeicher für sensible flüchtige In-Memory-Daten.
//!
//! # Funktionsweise & Garantien
//! Der Vault speichert Chunks ausschließlich im Arbeitsspeicher (Heap), niemals
//! auf SSD/Festplatte oder in persistenten WAL-Dateien (Write-Ahead-Log).
//! Beim Drop (oder explizitem Aufruf von [`VolatileContextVault::purge()`]) werden sämtliche Inhalte via
//! [`zeroize::Zeroize`] mit Nullen überschrieben.
//!
//! # Sicherheitshinweise
//! - **Memory-Locking:** Sensitive Chunks werden via [`contextra_sys::vault::LockedRegions`]
//!   (mittels `mlock` unter Linux/macOS bzw. `VirtualLock` unter Windows) im physischen RAM fixiert,
//!   um eine Auslagerung in den Betriebssystem-Swap zu verhindern (Best-Effort).
//! - **`std::mem::forget`:** Das Aufrufen von `std::mem::forget(vault)` ÜBERSPRINGT den
//!   `Drop`-Impl und damit die Zeroize-Garantie. Dies muss der Aufrufer selbst vermeiden.
//!   Nutze ausschließlich [`VolatileContextVault::purge()`] oder lass den Vault regulär per RAII droppen.
//! - **Swap-Risiko ohne mlock:** Falls `mlock()` fehlschlägt (z. B. fehlendes `RLIMIT_MEMLOCK`,
//!   fehlende Capabilities oder Kapazitätsgrenzen), bleibt der Vault nutzbar, aber ohne physische
//!   RAM-Fixierung. Eine Warnung wird via `tracing::warn!` ausgegeben.

use std::time::Instant;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::VaultError;
use contextra_sys::vault::LockedRegions;

/// Eindeutige ID eines Dokuments/Chunks im Vault (Kern-Typ).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VaultDocId(pub u64);

impl VaultDocId {
    /// Erzeugt eine neue VaultDocId.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Gibt den inneren numerischen Wert zurück.
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// Transaktions- und Sequenz-ID (Kern-Typ).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VaultTxId(pub u64);

impl VaultTxId {
    /// Erzeugt eine neue VaultTxId.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Gibt den inneren numerischen Wert zurück.
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// Mandanten-Kennung (Kern-Typ).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VaultTenantId(pub u64);

impl VaultTenantId {
    /// Erzeugt eine neue VaultTenantId.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Gibt den inneren numerischen Wert zurück.
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// Modalität eines Vault-Chunks — bestimmt den Verarbeitungskontext beim späteren Commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalModality {
    /// Audiospur oder Sprach-Transkript.
    AudioTranscript,
    /// Visuelle Erfassung, Screenshot oder Bilddaten.
    VisualCapture,
    /// Reguläre Texteingabe oder Prompt.
    TextInput,
    /// Strukturierte JSON- oder Key-Value-Eingabedaten.
    StructuredData,
}

/// Ein einzelner Datenblock (Chunk) im `VolatileContextVault`.
///
/// Sämtliche Inhalte werden beim Drop via [`ZeroizeOnDrop`] kryptographisch überschrieben.
#[derive(ZeroizeOnDrop)]
pub struct VaultChunk {
    /// Dokumenten-ID des Chunks.
    #[zeroize(skip)]
    pub id: VaultDocId,
    /// Sensitiver Inhalt im Speicher — wird bei Deallokation sofort gezeroized.
    pub content: Vec<u8>,
    /// Signal-Modalität des Chunks (Text, Audio, Bild etc.).
    #[zeroize(skip)]
    pub modality: SignalModality,
    /// Sequenznummer oder Transaktions-ID zum Aufnahmezeitpunkt.
    #[zeroize(skip)]
    pub captured_tx: u64,
    /// Optionale Kennzeichnung oder Kategorisierung des Chunks.
    #[zeroize(skip)]
    pub label: Option<String>,
}

impl VaultChunk {
    /// Erzeugt einen neuen sensitiven Chunk.
    ///
    /// # Parameter
    /// - `id`: Eindeutige Dokumenten-ID.
    /// - `content`: Rohe Nutzdaten (z. B. Text- oder Binärdaten).
    /// - `modality`: Art der Eingabedaten.
    /// - `captured_tx`: Transaktions-Sequenznummer zum Zeitpunkt der Aufnahme.
    pub fn new(
        id: VaultDocId,
        content: Vec<u8>,
        modality: SignalModality,
        captured_tx: VaultTxId,
    ) -> Self {
        Self {
            id,
            content,
            modality,
            captured_tx: captured_tx.0,
            label: None,
        }
    }

    /// Weist dem Chunk eine beschreibende Bezeichnung (Label) zu.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Gibt die Größe des Chunk-Inhalts in Bytes zurück.
    pub fn size_bytes(&self) -> usize {
        self.content.len()
    }
}

/// Konfiguration für den `VolatileContextVault`.
#[derive(Debug, Clone)]
pub struct VaultConfig {
    /// Maximale Speicherkapazität aller Chunks in Bytes (Standard: 512 MB).
    /// Bei Überschreitung schlägt `ingest()` mit [`VaultError::CapacityExceeded`] fehl (Fail-Closed).
    pub max_capacity_bytes: usize,
    /// Ob physisches Speicher-Locking (`mlock`) versucht werden soll (Best-Effort).
    pub attempt_mlock: bool,
}

impl Default for VaultConfig {
    /// Standardwerte: 512 MB Puffergröße, mlock aktiviert.
    fn default() -> Self {
        Self {
            max_capacity_bytes: 512 * 1024 * 1024, // 512 MB
            attempt_mlock: true,
        }
    }
}

/// Quittung nach erfolgreicher flüchtiger Speicherbereinigung via [`VolatileContextVault::purge_memory()`].
#[derive(Debug)]
pub struct VolatileDrainReceipt {
    /// Anzahl überschriebener und verworfener Chunks.
    pub chunks_purged: usize,
    /// Exakte Anzahl an Bytes, die im RAM mit Nullen überschrieben wurden.
    pub bytes_zeroed: usize,
    /// Zeitstempel, an dem die Bereinigung vollzogen wurde.
    pub purged_at: Instant,
}

include!("vault_compat.inc");

/// Quittung nach Übergabe der Chunks an die persistente Datenbank via [`VolatileContextVault::commit()`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitReceipt {
    /// Anzahl geschriebener Chunks.
    pub chunks_committed: usize,
    /// Gesamtgröße der übergebenen Nutzdaten in Bytes.
    pub bytes_committed: usize,
    /// Mandantenkennung, unter welcher der Commit verbucht wurde.
    pub tenant_id: VaultTenantId,
    /// Zugewiesene Transaktions- und Sequenznummer.
    pub tx_id: VaultTxId,
    /// Optionale generische Metadaten-Referenz.
    pub metadata_reference: Option<String>,
}

/// Ephemerer RAM-Puffer für sensitive Kontexte im Safe-Modus.
///
/// # INVARIANTEN
/// - **INV-VAULT-1:** `purge()` zeroized alle Chunk-Inhalte via `ZeroizeOnDrop` vor
///   der Deallokation. `std::mem::forget` auf den Vault hebelt diese Invariante aus.
/// - **INV-VAULT-2:** `ingest()` schreibt NIE in Storage, Log-Writer oder
///   irgendeinen Vektorindex. Einziger Schreibpfad auf dauerhafte Ablage:
///   expliziter Aufruf von `commit()`.
/// - **INV-VAULT-3:** Kapazitätsüberschreitung führt zu `VaultError::CapacityExceeded`.
///   Kein stiller Fallback auf Disk-Speicher.
pub struct VolatileContextVault {
    chunks: Vec<VaultChunk>,
    config: VaultConfig,
    current_bytes: usize,
    /// Gekapselte mlock'd Speicherbereiche via `contextra-sys`.
    mlock_regions: LockedRegions,
    /// Interne Konsumptions-Flag: verhindert doppeltes purge()/commit().
    consumed: bool,
    /// Tracking ob alle Chunk-Inhalte im RAM speicher-fixiert (locked) wurden.
    is_memory_locked: bool,
}

impl VolatileContextVault {
    /// Öffnet einen neuen leeren Vault im RAM.
    ///
    /// Führt keinerlei Datei-I/O oder Festplattenzugriffe aus.
    pub fn open(config: VaultConfig) -> Self {
        let initial_locked = config.attempt_mlock;
        if !config.attempt_mlock {
            tracing::info!(
                "VolatileContextVault: attempt_mlock is false. Memory-locking disabled (swap risk)."
            );
        }
        Self {
            chunks: Vec::new(),
            current_bytes: 0,
            config,
            mlock_regions: LockedRegions::new(),
            consumed: false,
            is_memory_locked: initial_locked,
        }
    }

    /// Nimmt einen sensitiven Chunk in den RAM-Puffer auf.
    ///
    /// # Ablauf von `ingest`:
    /// 1. Status prüfen: Wurde der Vault bereits konsumiert? Falls ja: `Err(AlreadyConsumed)`.
    /// 2. Kapazität prüfen: Überschreitet `current_bytes + new_bytes` das Limit? Falls ja: `Err(CapacityExceeded)`.
    /// 3. Memory-Locking: `mlock`-Aufruf versuchen, um Swap-Auslagerung auf SSD zu verhindern.
    /// 4. Chunk im flüchtigen RAM-Vektor speichern und Byte-Zähler inkrementieren.
    ///
    /// # Invariante INV-VAULT-2:
    /// Schreibt niemals auf Festplatte oder in ein persistentes Log.
    pub fn ingest(&mut self, chunk: VaultChunk) -> Result<(), VaultError> {
        if self.consumed {
            return Err(VaultError::AlreadyConsumed);
        }
        let new_bytes = chunk.size_bytes();
        if self.current_bytes + new_bytes > self.config.max_capacity_bytes {
            return Err(VaultError::CapacityExceeded {
                current_bytes: self.current_bytes,
                new_bytes,
                max_bytes: self.config.max_capacity_bytes,
            });
        }

        // Memory-Locking (best-effort): chunk.content im RAM fixieren via contextra-sys.
        let locked = self
            .mlock_regions
            .lock_slice(&chunk.content, self.config.attempt_mlock);

        if !locked && !chunk.content.is_empty() {
            self.is_memory_locked = false;
            if self.config.attempt_mlock {
                tracing::warn!(
                    "VolatileContextVault: Memory-locking failed for chunk id={:?}. \
                     Sensitive content is not locked in RAM (swap risk).",
                    chunk.id
                );
            }
        }

        self.current_bytes += new_bytes;
        self.chunks.push(chunk);
        Ok(())
    }

    /// Verwirft alle Chunks sofort und überschreibt deren Heap-Inhalte kryptographisch mit Nullen.
    ///
    /// # Ablauf von `purge`:
    /// 1. Alle Chunk-Inhalte im Heap explizit via [`Zeroize::zeroize()`] mit Nullen überschreiben.
    /// 2. Erst nach dem Nullsetzen: Betriebssystem-Locks via `unlock_all()` freigeben (`munlock`).
    /// 3. Chunk-Liste leeren, Vault als konsumiert markieren und [`VolatileDrainReceipt`] ausstellen.
    pub fn purge_memory(mut self) -> VolatileDrainReceipt {
        let chunks_purged = self.chunks.len();
        let bytes_zeroed = self.current_bytes;

        // 1. Explizit Chunks zeroizen:
        for chunk in &mut self.chunks {
            chunk.content.zeroize();
        }

        // 2. mlock-Regionen freigeben (nach Zeroize!).
        self.mlock_regions.unlock_all();

        // 3. Chunks droppen (ZeroizeOnDrop nochmals als Sicherheitsnetz).
        self.chunks.clear();
        self.consumed = true;

        #[allow(clippy::disallowed_methods)]
        let purged_at = Instant::now();

        VolatileDrainReceipt {
            chunks_purged,
            bytes_zeroed,
            purged_at,
        }
    }

    /// Neutralisierte Speicherleeren-Methode (Alias für [`Self::purge_memory`]).
    #[inline]
    pub fn purge(self) -> VolatileDrainReceipt {
        self.purge_memory()
    }

    /// Gibt die Summe der aktuell im flüchtigen RAM liegenden Nutzdaten in Bytes zurück.
    pub fn current_size_bytes(&self) -> usize {
        self.current_bytes
    }

    /// Gibt die Anzahl der aktuell gepufferten Chunks zurück.
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    /// Prüft, ob der Vault leer ist.
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    /// Gibt an, ob alle Chunk-Inhalte im RAM gegen Swap gesichert sind (`mlock`).
    ///
    /// Liefert `false`, falls `attempt_mlock` deaktiviert war oder das Betriebssystem
    /// das Sperren verweigerte.
    pub fn is_memory_locked(&self) -> bool {
        self.is_memory_locked
    }

    /// Transferiert alle Chunks zur dauerhaften Speicherung und stellt ein [`CommitReceipt`] aus.
    ///
    /// # Ablauf von `commit`:
    /// 1. Status prüfen: Falls bereits konsumiert, `Err(AlreadyConsumed)`.
    /// 2. Chunks aus dem Puffer entnehmen (`std::mem::take`).
    /// 3. Speicher-Locks lösen (`munlock`).
    /// 4. Quittung mit Mandanten-ID, Transaktions-ID und Prüfreferenz erzeugen.
    pub fn commit(
        mut self,
        tenant_id: VaultTenantId,
        tx_id: VaultTxId,
        metadata_reference: Option<impl Into<String>>,
    ) -> Result<(Vec<VaultChunk>, CommitReceipt), VaultError> {
        if self.consumed {
            return Err(VaultError::AlreadyConsumed);
        }

        let chunks_committed = self.chunks.len();
        let bytes_committed = self.current_bytes;
        let chunks = std::mem::take(&mut self.chunks);

        self.mlock_regions.unlock_all();
        self.consumed = true;

        let receipt = CommitReceipt {
            chunks_committed,
            bytes_committed,
            tenant_id,
            tx_id,
            metadata_reference: metadata_reference.map(|r| r.into()),
        };

        Ok((chunks, receipt))
    }

    /// Gibt eine Vorschau der Metadaten zurück, ohne sensible Inhalte zu exponieren.
    ///
    /// Verhindert versehentliches Ausleiten oder Protokollieren von Geheimnissen.
    pub fn preview_metadata(&self) -> Vec<VaultChunkMetadata> {
        self.chunks
            .iter()
            .map(|c| VaultChunkMetadata {
                id: c.id,
                modality: c.modality.clone(),
                size_bytes: c.content.len(),
                label: c.label.clone(),
            })
            .collect()
    }
}

/// Metadaten-Ansicht eines Chunks ohne sensiblen Nutzdateninhalt.
#[derive(Debug, Clone)]
pub struct VaultChunkMetadata {
    /// Eindeutige ID des Dokuments.
    pub id: VaultDocId,
    /// Modalität (Text, Audio etc.).
    pub modality: SignalModality,
    /// Größe der Nutzdaten in Bytes.
    pub size_bytes: usize,
    /// Optionale Beschreibung/Kategorie.
    pub label: Option<String>,
}

// RAII-Drop-Implementierung: Schützt vor Speicherlecks und ungelöschtem RAM,
// falls ein Vault den Gültigkeitsbereich verlässt, ohne explizit purge() aufzurufen.
impl Drop for VolatileContextVault {
    fn drop(&mut self) {
        if !self.consumed {
            tracing::debug!(
                "VolatileContextVault: impliziter Drop ohne purge(). \
                 {} Chunks ({} Bytes) werden gezeroized.",
                self.chunks.len(),
                self.current_bytes
            );

            // Explizites Zeroize der Inhalte.
            for chunk in &mut self.chunks {
                chunk.content.zeroize();
            }

            // munlock NACH Zeroize.
            self.mlock_regions.unlock_all();
        }
    }
}

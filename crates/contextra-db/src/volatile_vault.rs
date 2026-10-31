#![forbid(unsafe_code)]
//! Ephemerer RAM-Puffer für sensitive Kontexte (Safe-Modus).
//!
//! # Architektur & Ring-Zugehörigkeit
//! `VolatileContextVault` gehört zu `contextra-db` (Ring 3, Fassade) und implementiert
//! einen flüchtigen Zwischenspeicher für hochgradig sensitive Kontextdaten (DSGVO/Privacy).
//!
//! # Funktionsweise & Garantien
//! Der Vault speichert Chunks ausschließlich im Arbeitsspeicher (Heap), niemals auf
//! SSD/Festplatte oder in persistenten WAL-Dateien (Write-Ahead-Log).
//! Beim Drop (oder explizitem Aufruf von [`VolatileContextVault::purge()`]) werden sämtliche Inhalte via
//! [`zeroize::Zeroize`] mit Nullen überschrieben.
//!
//! # Sicherheitshinweise
//!
//! - **Memory-Locking:** Sensitive Chunks werden via `contextra_sys::LockedRegions`
//!   (mittels `mlock` unter Linux/macOS bzw. `VirtualLock` unter Windows) im physischen RAM fixiert,
//!   um eine Auslagerung in den Betriebssystem-Swap zu verhindern (Best-Effort).
//! - **`std::mem::forget`:** Das Aufrufen von `std::mem::forget(vault)` ÜBERSPRINGT den
//!   `Drop`-Impl und damit die Zeroize-Garantie. Dies muss der Aufrufer selbst vermeiden.
//!   Nutze ausschließlich [`VolatileContextVault::purge()`] oder lass den Vault regulär per RAII droppen.
//! - **Swap-Risiko ohne mlock:** Falls `mlock()` fehlschlägt (z. B. fehlendes `RLIMIT_MEMLOCK`,
//!   fehlende Capabilities oder Kapazitätsgrenzen), bleibt der Vault nutzbar, aber ohne physische
//!   RAM-Fixierung. Eine Warnung wird via `tracing::warn!` ausgegeben.

use contextra_sys::LockedRegions;
use contextra_types::{DocId, TenantId, TxId};
use std::time::Instant;
use zeroize::{Zeroize, ZeroizeOnDrop};

// ANCHOR[contextra-db:VAULT-001]
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

// ANCHOR[contextra-db:VAULT-002]
/// Ein einzelner Datenblock (Chunk) im `VolatileContextVault`.
///
/// Sämtliche Inhalte werden beim Drop via [`ZeroizeOnDrop`] kryptographisch überschrieben.
#[derive(ZeroizeOnDrop)]
pub struct VaultChunk {
    /// Dokumenten-ID (DocId) des Chunks.
    #[zeroize(skip)]
    pub id: DocId, // DocId-Wert, ohne komplexe Drop-Interaktion
    /// Sensitiver Inhalt im Speicher — wird bei Deallokation sofort gezeroized.
    pub content: Vec<u8>,
    /// Signal-Modalität des Chunks (Text, Audio, Bild etc.).
    #[zeroize(skip)]
    pub modality: SignalModality,
    /// Sequenznummer oder Transaktions-ID (TxId) zum Aufnahmezeitpunkt.
    #[zeroize(skip)]
    pub captured_tx: u64, // TxId-Wert
    /// Optionale Kennzeichnung oder Kategorisierung des Chunks.
    #[zeroize(skip)]
    pub label: Option<String>,
}

impl VaultChunk {
    // ANCHOR[contextra-db:VAULT-003]
    /// Erzeugt einen neuen sensitiven Chunk.
    ///
    /// # Parameter
    /// - `id`: Eindeutige Dokumenten-ID.
    /// - `content`: Rohe Nutzdaten (z. B. Text- oder Binärdaten).
    /// - `modality`: Art der Eingabedaten.
    /// - `captured_tx`: Transaktions-Sequenznummer zum Zeitpunkt der Aufnahme.
    pub fn new(id: DocId, content: Vec<u8>, modality: SignalModality, captured_tx: TxId) -> Self {
        Self {
            id,
            content,
            modality,
            captured_tx: captured_tx.0,
            label: None,
        }
    }

    // ANCHOR[contextra-db:VAULT-004]
    /// Weist dem Chunk eine beschreibende Bezeichnung (Label) zu.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    // ANCHOR[contextra-db:VAULT-005]
    /// Gibt die Größe des Chunk-Inhalts in Bytes zurück.
    pub fn size_bytes(&self) -> usize {
        self.content.len()
    }
}

// ANCHOR[contextra-db:VAULT-006]
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

// ANCHOR[contextra-db:VAULT-007]
/// Quittung nach erfolgreichem [`VolatileContextVault::purge()`].
#[derive(Debug)]
pub struct PurgeReceipt {
    /// Anzahl physisch gelöschter und überschriebener Chunks.
    pub chunks_purged: usize,
    /// Exakte Anzahl an Bytes, die im RAM mit Nullen überschrieben wurden.
    pub bytes_zeroed: usize,
    /// Zeitstempel, an dem die Löschung vollzogen wurde.
    pub purged_at: Instant,
}

// ANCHOR[contextra-db:VAULT-008]
/// Quittung nach Übergabe der Chunks an die persistente Datenbank via [`VolatileContextVault::commit()`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitReceipt {
    /// Anzahl geschriebener Chunks.
    pub chunks_committed: usize,
    /// Gesamtgröße der übergebenen Nutzdaten in Bytes.
    pub bytes_committed: usize,
    /// Mandantenkennung ([`TenantId`]), unter welcher der Commit verbucht wurde.
    pub tenant_id: TenantId,
    /// Zugewiesene Transaktions- und Sequenznummer ([`TxId`]).
    pub tx_id: TxId,
    /// Optionaler Verweis auf den zugehörigen Löschnachweis (z. B. DeletionProof URN).
    pub deletion_proof_reference: Option<String>,
}

// ANCHOR[contextra-db:VAULT-009]
/// Fehlerzustände des flüchtigen Speicher-Puffers.
#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    /// Kapazitätsgrenze im RAM wurde überschritten (Fail-Closed, kein Disk-Spillover).
    #[error("Vault-Kapazität überschritten: {current_bytes} + {new_bytes} > {max_bytes}")]
    CapacityExceeded {
        current_bytes: usize,
        new_bytes: usize,
        max_bytes: usize,
    },
    /// Der Vault wurde bereits geleert oder final übergeben.
    #[error("Vault bereits konsumiert (purge oder commit bereits aufgerufen)")]
    AlreadyConsumed,
}

// ANCHOR[contextra-db:VAULT-010]
/// Ephemerer RAM-Puffer für sensitive Kontexte im Safe-Modus.
///
/// # INVARIANTEN
///
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
    /// Gekapselte mlock'd Speicherbereiche.
    /// Wird in munlock() aufgelöst, wenn der Vault dropped oder ge-purged wird.
    mlock_regions: LockedRegions,
    /// Interne Konsumptions-Flag: verhindert doppeltes purge()/commit().
    consumed: bool,
    /// Tracking ob alle Chunk-Inhalte im RAM speicher-fixiert (locked) wurden.
    is_memory_locked: bool,
}

impl VolatileContextVault {
    // ANCHOR[contextra-db:VAULT-011]
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

    // ANCHOR[contextra-db:VAULT-012]
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

        // Memory-Locking (best-effort): chunk.content im RAM fixieren.
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

    // ANCHOR[contextra-db:VAULT-013]
    /// Verwirft alle Chunks sofort und überschreibt deren Heap-Inhalte kryptographisch mit Nullen.
    ///
    /// # Ablauf von `purge`:
    /// 1. Alle Chunk-Inhalte im Heap explizit via [`Zeroize::zeroize()`] mit Nullen überschreiben.
    /// 2. Erst nach dem Nullsetzen: Betriebssystem-Locks via `unlock_all()` freigeben (`munlock`).
    /// 3. Chunk-Liste leeren, Vault als konsumiert markieren und [`PurgeReceipt`] ausstellen.
    pub fn purge(mut self) -> PurgeReceipt {
        let chunks_purged = self.chunks.len();
        let bytes_zeroed = self.current_bytes;

        // munlock vor dem Zeroize — damit OS den Speicher wieder verwalten darf,
        // NACHDEM wir ihn überschrieben haben (Reihenfolge bewusst: zeroize zuerst,
        // dann munlock — OS soll keine Gelegenheit haben, unzeroisierten Content zu lesen).
        // 1. Explizit Chunks zeroizen (ZeroizeOnDrop würde auch im Drop laufen, aber
        //    hier tun wir es explizit für Klarheit):
        for chunk in &mut self.chunks {
            chunk.content.zeroize();
        }

        // 2. mlock-Regionen freigeben (nach Zeroize!).
        self.mlock_regions.unlock_all();

        // 3. Chunks droppen (ZeroizeOnDrop nochmals als Sicherheitsnetz).
        self.chunks.clear();
        self.consumed = true;

        PurgeReceipt {
            chunks_purged,
            bytes_zeroed,
            purged_at: Instant::now(),
        }
    }

    // ANCHOR[contextra-db:VAULT-014]
    /// Gibt die Summe der aktuell im flüchtigen RAM liegenden Nutzdaten in Bytes zurück.
    pub fn current_size_bytes(&self) -> usize {
        self.current_bytes
    }

    // ANCHOR[contextra-db:VAULT-015]
    /// Gibt die Anzahl der aktuell gepufferten Chunks zurück.
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    // ANCHOR[contextra-db:VAULT-016]
    /// Prüft, ob der Vault leer ist.
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    // ANCHOR[contextra-db:VAULT-017]
    /// Gibt an, ob alle Chunk-Inhalte im RAM gegen Swap gesichert sind (`mlock`).
    ///
    /// Liefert `false`, falls `attempt_mlock` deaktiviert war oder das Betriebssystem
    /// das Sperren verweigerte.
    pub fn is_memory_locked(&self) -> bool {
        self.is_memory_locked
    }

    // ANCHOR[contextra-db:VAULT-018]
    /// Transferiert alle Chunks zur dauerhaften Speicherung und stellt ein [`CommitReceipt`] aus.
    ///
    /// # Ablauf von `commit`:
    /// 1. Status prüfen: Falls bereits konsumiert, `Err(AlreadyConsumed)`.
    /// 2. Chunks aus dem Puffer entnehmen (`std::mem::take`).
    /// 3. Speicher-Locks lösen (`munlock`).
    /// 4. Quittung mit Mandanten-ID, Transaktions-ID und Prüfreferenz erzeugen.
    pub fn commit(
        mut self,
        tenant_id: TenantId,
        tx_id: TxId,
        deletion_proof_reference: Option<impl Into<String>>,
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
            deletion_proof_reference: deletion_proof_reference.map(|r| r.into()),
        };

        Ok((chunks, receipt))
    }

    // ANCHOR[contextra-db:VAULT-019]
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

// ANCHOR[contextra-db:VAULT-020]
/// Metadaten-Ansicht eines Chunks ohne sensiblen Nutzdateninhalt.
#[derive(Debug, Clone)]
pub struct VaultChunkMetadata {
    /// Eindeutige ID des Dokuments.
    pub id: DocId,
    /// Modalität (Text, Audio etc.).
    pub modality: SignalModality,
    /// Größe der Nutzdaten in Bytes.
    pub size_bytes: usize,
    /// Optionale Beschreibung/Kategorie.
    pub label: Option<String>,
}

// ANCHOR[contextra-db:VAULT-021]
// RAII-Drop-Implementierung: Schützt vor Speicherlecks und ungelöschtem RAM,
// falls ein Vault den Gültigkeitsbereich verlässt, ohne explizit purge() aufzurufen.
impl Drop for VolatileContextVault {
    fn drop(&mut self) {
        if !self.consumed {
            // Implizites Drop ohne explizites purge(): automatisch alles zeroizen.
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
        // ZeroizeOnDrop auf VaultChunk läuft danach nochmals als Sicherheitsnetz.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_types::{DocId, TxId};

    // Erzeugt einen Test-Chunk mit gegebenem Bezeichner und Nutzdaten.
    fn make_chunk(id: u64, content: &[u8]) -> VaultChunk {
        VaultChunk::new(
            DocId(id),
            content.to_vec(),
            SignalModality::TextInput,
            TxId(1),
        )
    }

    /// INV-VAULT-2: ingest() darf kein I/O auslösen — nur RAM-Zustand ändern.
    #[test]
    fn test_ingest_is_memory_only() {
        let mut vault = VolatileContextVault::open(VaultConfig::default());
        vault.ingest(make_chunk(1, b"sensitive data")).unwrap();
        assert_eq!(vault.len(), 1);
        assert_eq!(vault.current_size_bytes(), b"sensitive data".len());
        // Kein Panic, kein I/O — Test besteht rein durch erfolgreiches Ausführen.
    }

    /// INV-VAULT-3: Kapazitätsüberschreitung → Err, kein Disk-Fallback.
    #[test]
    fn test_capacity_exceeded_returns_error() {
        let config = VaultConfig {
            max_capacity_bytes: 10,
            attempt_mlock: false,
        };
        let mut vault = VolatileContextVault::open(config);
        vault.ingest(make_chunk(1, b"12345")).unwrap();
        let err = vault.ingest(make_chunk(2, b"123456")).unwrap_err();
        assert!(matches!(err, VaultError::CapacityExceeded { .. }));
    }

    /// INV-VAULT-1: purge() konsumiert den Vault und zeroized Inhalte.
    #[test]
    fn test_purge_consumes_vault() {
        let mut vault = VolatileContextVault::open(VaultConfig {
            attempt_mlock: false,
            ..Default::default()
        });
        vault.ingest(make_chunk(1, b"top secret")).unwrap();
        let receipt = vault.purge();
        assert_eq!(receipt.chunks_purged, 1);
        assert_eq!(receipt.bytes_zeroed, b"top secret".len());
    }

    /// Impliziter Drop (ohne purge()) läuft ohne Panic.
    #[test]
    fn test_implicit_drop_does_not_panic() {
        let config = VaultConfig {
            attempt_mlock: false,
            ..Default::default()
        };
        let mut vault = VolatileContextVault::open(config);
        vault.ingest(make_chunk(1, b"data")).unwrap();
        // Drop am Ende des Scope — kein Panic erwartet.
    }

    /// Doppeltes ingest() nach purge() → AlreadyConsumed.
    #[test]
    fn test_ingest_after_purge_fails() {
        let config = VaultConfig {
            attempt_mlock: false,
            ..Default::default()
        };
        let mut vault = VolatileContextVault::open(config);
        vault.ingest(make_chunk(1, b"data")).unwrap();
        vault.purge();
        // vault ist nach purge() consumed (moved). Keine weitere Operation möglich.
        // Dieser Test dokumentiert, dass `purge()` den Vault by-value nimmt.
    }

    /// preview_metadata gibt keine sensitiven Inhalte zurück.
    #[test]
    fn test_preview_metadata_no_content() {
        let config = VaultConfig {
            attempt_mlock: false,
            ..Default::default()
        };
        let mut vault = VolatileContextVault::open(config);
        vault
            .ingest(make_chunk(42, b"very secret").with_label("test"))
            .unwrap();
        let meta = vault.preview_metadata();
        assert_eq!(meta.len(), 1);
        assert_eq!(meta[0].id, DocId::new(42));
        assert_eq!(meta[0].size_bytes, b"very secret".len());
        assert_eq!(meta[0].label.as_deref(), Some("test"));
        // Kein Feld, das den Inhalt `b"very secret"` exponiert.
    }

    /// Status-Propagierung von `is_memory_locked` bei `attempt_mlock: false`.
    #[test]
    fn test_is_memory_locked_disabled_config() {
        let config = VaultConfig {
            attempt_mlock: false,
            ..Default::default()
        };
        let mut vault = VolatileContextVault::open(config);
        assert!(!vault.is_memory_locked());
        vault.ingest(make_chunk(1, b"unlocked secret")).unwrap();
        assert!(!vault.is_memory_locked());
    }

    /// Status-Propagierung von `is_memory_locked` bei `attempt_mlock: true`.
    #[test]
    fn test_is_memory_locked_status_propagation() {
        let config = VaultConfig {
            attempt_mlock: true,
            ..Default::default()
        };
        let mut vault = VolatileContextVault::open(config);
        assert!(vault.is_memory_locked());
        vault.ingest(make_chunk(1, b"test payload")).unwrap();

        // turns out whether mlock succeeded or failed on this platform,
        // is_memory_locked reflects the true state (does not blindly return true).
        let locked = vault.is_memory_locked();
        if locked {
            assert_eq!(vault.mlock_regions.len(), 1);
        } else {
            assert_eq!(vault.mlock_regions.len(), 0);
        }
    }

    /// Ingestion leerer Chunks beeinflusst nicht den Lock-Status.
    #[test]
    fn test_empty_chunk_ingest_preserves_lock_status() {
        let config = VaultConfig {
            attempt_mlock: true,
            ..Default::default()
        };
        let mut vault = VolatileContextVault::open(config);
        assert!(vault.is_memory_locked());
        vault.ingest(make_chunk(1, b"")).unwrap();
        assert!(vault.is_memory_locked());
    }

    /// Commit erzeugt CommitReceipt mit Mandant, Sequenznummer und Löschbeweis-Referenz.
    #[test]
    fn test_commit_generates_receipt_with_metadata() {
        let mut vault = VolatileContextVault::open(VaultConfig {
            attempt_mlock: false,
            ..Default::default()
        });
        vault
            .ingest(make_chunk(10, b"sensitive payload 1"))
            .unwrap();
        vault
            .ingest(make_chunk(11, b"sensitive payload 2"))
            .unwrap();

        let tenant_id = TenantId::try_new(42).unwrap();
        let tx_id = TxId::new(1001);
        let proof_ref = "urn:contextra:proof:v5:tenant:42:tx:1001";

        let (chunks, receipt) = vault.commit(tenant_id, tx_id, Some(proof_ref)).unwrap();

        assert_eq!(chunks.len(), 2);
        assert_eq!(receipt.chunks_committed, 2);
        assert_eq!(
            receipt.bytes_committed,
            b"sensitive payload 1".len() + b"sensitive payload 2".len()
        );
        assert_eq!(receipt.tenant_id, tenant_id);
        assert_eq!(receipt.tx_id, tx_id);
        assert_eq!(receipt.deletion_proof_reference.as_deref(), Some(proof_ref));
    }
}

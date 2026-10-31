// FILE-CONTEXT
// ZWECK: Contextra Primary Facade & Composition Root (Ring 4).
// INVARIANTEN: #![forbid(unsafe_code)]; <= 20 pub fn; zero direct dependencies on Ring 0/1 internal crates.

//! # Modul: Contextra Primary Facade & Composition Root (Ring 4)
//!
//! Dieses Modul bildet die oberste Einstiegsebene (Fassade bzw. Composition Root)
//! des Gesamtsystems **Contextra** innerhalb der Architektur (Ring 4).
//!
//! ## Zweck & Architektur
//! Die Fassade dient als zentrale Abstraktions- und Konfigurationsschicht für anwendende
//! Systeme (wie z. B. `contextra-mcp` oder externe SDKs/Applikationen). Sie verbirgt die
//! interne Komplexität der tieferen Schichten (Ring 0 bis 3) und stellt eine einheitliche,
//! ergonomische Programmierschnittstelle (API = Application Programming Interface) bereit.
//!
//! ## Ring-Zugehörigkeit & Abhängigkeiten
//! - **Ring 4 (Fassade)**: Greift als Composition Root auf Teilsysteme wie `contextra-db` (Ring 3),
//!   `contextra-types`, `contextra-ports`, `contextra-license`, `contextra-store`
//!   und `contextra-rank` zu.
//! - **Keine zyklischen Abhängigkeiten**: Höhere Ringe hängen von niederen Ringen ab, nie umgekehrt.
//! - **Eingebunden von**: `contextra-mcp` (Model Context Protocol Adapter für LLM-Agenten).
//!
//! ## Kernkonzepte & wichtigste Typen
//! - [`ContextraBuilder`]: Konfiguriert Speicherpfade, Vektordimensionen, Verschlüsselung, Lizenzprüfung
//!   und Profile vor dem Start der Datenbankinstanz.
//! - [`Contextra`]: Die eigentliche Datenbankinstanz zur Verwaltung von Collections (Dokumentensammlungen),
//!   Vektorindizes und Volltextindizes.
//! - [`AgentMemory`]: Eine agentenzentrierte High-Level-Abstraktion mit `remember()`, `recall()`, `forget()`
//!   und Wissensgraphen-Relationen (`relate()`).
//! - [`LlmQueryRewriter`]: LLM-gestützte Umformulierung von Suchanfragen für mehrstufiges Retrieval.
//!
//! ## Sicherheits- und Entwurfs-Invarianten
//! - **Kein Unsafe**: Striktes `#![forbid(unsafe_code)]` garantiert Speichersicherheit zur Compile-Zeit.
//! - **Kein Panik-Zustand**: Fehler werden strikt über den Ergebnistyp [`Result`] bzw. [`ContextraError`]
//!   nach oben gereicht (Fail-Closed-Prinzip).
//! - **Entkopplung**: Zeit-, Zufalls- und ID-Generierungsoperationen erfolgen deterministisch über Ports.

#![forbid(unsafe_code)]

pub mod agent_memory;
pub mod builder;
pub mod collection_profile;
pub mod performance_profile;
pub mod query_rewriter;

pub use agent_memory::{AgentMemory, Memory, MemoryId};
pub use builder::{ContextraBuilder, ContextraConfigExt, EncryptionMode};
pub use contextra_db::{
    chunker, execute_background_consolidation, memory_consolidation, Collection, CollectionConfig,
    Contextra, ContextraConfig, ContextraStats, DriftStatusProvider, EmbeddingBackend,
    SearchResult, TextEmbeddingEngine,
};
pub use contextra_types::error::ContextraError;
pub use contextra_types::types::domain::{DocId, ScoredDocument};
pub use contextra_types::DistanceMetric;
pub use query_rewriter::LlmQueryRewriter;

pub use contextra_rank as rank;

#[cfg(feature = "candle")]
pub use contextra_infer_candle as candle;

// ANCHOR[contextra:LIB-001]
/// Erzeugt einen neuen [`ContextraBuilder`] zur Konfiguration und Instanziierung der Datenbank [`Contextra`].
///
/// # Parameter
/// - `dimension`: Die Anzahl der Dimensionen (z. B. 384, 768, 1536) für die Vektoreinbettungen (Embeddings).
///
/// # Rückgabe
/// Gibt einen vorinitialisierten [`ContextraBuilder`] mit Standardeinstellungen zurück.
///
/// # Ablauf von `builder`:
/// 1. Übernahme der gewünschten Vektordimension (`dimension`).
/// 2. Initialisierung des Builders mit Default-Parametern (Speicherpfad `./contextra_data`, Fast-Lizenzgate).
/// 3. Rückgabe der Builder-Instanz zur methodenweisen Weiterkonfiguration.
///
/// # Vorbedingungen
/// - `dimension` muss größer als 0 sein (wird bei Vektorerzeugung tieferer Schichten validiert).
///
/// # Nebenwirkungen
/// - Keine I/O- oder Speicher-Nebenwirkungen; reine Allokation auf dem Stack.
///
/// # Beispiel
/// ```ignore
/// use contextra::builder;
///
/// let b = builder(768);
/// ```
pub fn builder(dimension: usize) -> ContextraBuilder {
    ContextraBuilder::new(dimension)
}

// ANCHOR[contextra:LIB-002]
/// Öffnet oder erstellt eine [`Contextra`]-Datenbankinstanz am angegebenen Dateisystempfad mit Standardkonfiguration.
///
/// Verwendet standardmäßig eine Vektordimension von 768 und das unbeschränkte Entwicklungs-Lizenzgate (`OpenFastGate`).
///
/// # Parameter
/// - `path`: Dateipfad zum Datenbankverzeichnis (wird bei Bedarf erstellt).
///
/// # Rückgabe
/// - `Ok(Contextra)`: Vollständig initialisierte Datenbankinstanz.
/// - `Err(ContextraError)`: Fehler bei Dateisystem-I/O, Berechtigungen oder Datenbankkorruption.
///
/// # Ablauf von `open`:
/// 1. Erzeugung eines Builders mit Standard-Vektordimension 768.
/// 2. Zuweisung des Zielpfads im Dateisystem (`with_storage_path`).
/// 3. Asynchroner Aufruf von `build().await` (Initialisierung des LSM-Speichers, WAL, Lockfiles).
///
/// # Vorbedingungen
/// - Das übergeordnete Verzeichnis muss im Dateisystem existieren und beschreibbar sein.
///
/// # Nachbedingungen
/// - Bei Erfolg existiert das Verzeichnis mit den initialisierten LSM- und WAL-Strukturen.
/// - Exklusiver Schreibzugriff wird über ein Sperrfile (`LOCK`) abgesichert.
///
/// # Nebenwirkungen
/// - Führt asynchrones Dateisystem-I/O aus (Verzeichniserstellung, Dateizugriff, Locks).
///
/// # Fehlerfälle
/// - [`ContextraError::Io`]: Dateisystemfehler (z. B. Pfad nicht schreibbar).
/// - [`ContextraError::LockConflict`]: Ein anderer Prozess hält bereits die Datenbanksperre.
///
/// # Beispiel
/// ```ignore
/// # async fn run() -> Result<(), contextra::ContextraError> {
/// let db = contextra::open("./meine_db").await?;
/// # Ok(())
/// # }
/// ```
pub async fn open(path: impl AsRef<std::path::Path>) -> Result<Contextra, ContextraError> {
    builder(768).with_storage_path(path.as_ref()).build().await
}

// ANCHOR[contextra:LIB-003]
/// Öffnet oder erstellt eine [`Contextra`]-Datenbankinstanz mit einer explizit übergebenen Konfiguration.
///
/// # Parameter
/// - `path`: Dateipfad zum Datenbankverzeichnis.
/// - `config`: Detaillierte Konfiguration ([`ContextraConfig`]) u. a. mit Vektordimension, Speichergrenzen und Tuning-Parametern.
///
/// # Rückgabe
/// - `Ok(Contextra)`: Bereitgestellte Datenbankinstanz.
/// - `Err(ContextraError)`: Konfigurationskonflikt, I/O-Fehler oder Verstoß gegen Lizenzrichtlinien.
///
/// # Ablauf von `open_with_config`:
/// 1. Entnahme der Dimension aus der übergebenen `config` (`config.dimension`).
/// 2. Konfiguration des Pfades (`with_storage_path`) und Übernahme der gesamten `config`.
/// 3. Validierung und asynchrones Hochfahren des Datenbanksystems (`build().await`).
///
/// # Vorbedingungen
/// - `path` muss ein gültiger Dateipfad sein.
/// - `config` muss in sich konsistente Parameter enthalten.
///
/// # Nachbedingungen
/// - Die Datenbank läuft mit den spezifizierten Parametern.
///
/// # Nebenwirkungen
/// - Asynchrones Dateisystem-I/O über die VFS-Abstraktionsschicht.
///
/// # Fehlerfälle
/// - I/O-Fehler, Lock-Kollisionen oder ungültige Konfigurationswerte.
pub async fn open_with_config(
    path: impl AsRef<std::path::Path>,
    config: ContextraConfig,
) -> Result<Contextra, ContextraError> {
    builder(config.dimension)
        .with_storage_path(path.as_ref())
        .with_config(config)
        .build()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR[contextra:TEST-LIB-001]
    // Prüft die drei primären Fassaden-Einstiegspunkte (open, open_with_config, builder) auf korrekte Initialisierung einer leeren Datenbank.
    #[tokio::test]
    async fn test_facade_open_and_builder_entrypoints() -> Result<(), Box<dyn std::error::Error>> {
        let base_tmp =
            std::env::temp_dir().join(format!("contextra_facade_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base_tmp);

        let db1_path = base_tmp.join("db1");
        let db1 = open(&db1_path).await?;
        assert_eq!(db1.len().await?, 0);

        let config = ContextraConfig {
            dimension: 16,
            ..Default::default()
        };
        let db2_path = base_tmp.join("db2");
        let db2 = open_with_config(&db2_path, config).await?;
        assert_eq!(db2.len().await?, 0);

        let db3_path = base_tmp.join("db3");
        let db3 = builder(32).with_storage_path(&db3_path).build().await?;
        assert_eq!(db3.len().await?, 0);

        let _ = std::fs::remove_dir_all(&base_tmp);
        Ok(())
    }
}

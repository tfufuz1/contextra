// FILE-CONTEXT
// ZWECK: Contextra Composition Root Builder (Ring 4).
// INVARIANTEN: Zero-panic in production code; orchestriert Subsystem-Konfiguration ohne Geschäftslogik.

//! # Modul: Contextra Composition Root Builder (Ring 4)
//!
//! Dieses Modul stellt den zentralen Builder (`ContextraBuilder`) zur Verfügung, welcher als
//! **Composition Root** dient: Hier fließen alle Subsysteme (Speicher, Lizenzprüfung, Verschlüsselung,
//! Embedding-Engine, Performance-Profile, CSPRNG-Zufallsquelle) zusammen und werden initial verdrahtet.
//!
//! ## Ring-Zugehörigkeit
//! - **Ring 4 (Fassade)**: Enthält selbst **keine Geschäftslogik**, sondern orchestriert
//!   ausschließlich die Konfiguration und Übergabe von Parametern an `contextra-engine` (Ring 3).
//!
//! ## Wichtigste Typen
//! - [`EncryptionMode`]: Steuert die Data-at-Rest-Verschlüsselung (Pflichtverschlüsselung gemäß D-05).
//! - [`ContextraConfigExt`]: Hilfstraiterweiterung für Konfigurationsabfragen bezüglich Verschlüsselung.
//! - [`ContextraBuilder`]: Die fluide Builder-Struktur zur Konstruktion einer lauffähigen Datenbank.
//!
//! ## Entwurfs- & Sicherheitsrichtlinien
//! - **Zero-Panic-Doktrin**: Keine unkontrollierten Panics im Produktionscode (`unwrap` / `expect` verboten).
//!   Fehler werden als typisierte [`ContextraError`]-Werte zurückgegeben.
//! - **Sichere Geheimnisverwaltung**: Passphrasen werden umgehend in [`Zeroizing<String>`] gekapselt,
//!   sodass ihr Speicherbereich beim Verlassen des Gültigkeitsbereichs mit Nullen überschrieben wird.
//!   Zudem wird in Debug-Ausgaben der Platzhalter `[REDACTED]` ausgegeben.
//! - **Pflichtverschlüsselung (D-05)**: Unverschlüsselter Betrieb ist im Soll-Zustand nicht zulässig.
//!   Fehlt eine Passphrase oder wird unverschlüsselter Modus angefordert, bricht `build()` mit einem
//!   typisierten Fehler ab.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::rng::OsRngAdapter;
use contextra_engine::{Contextra, ContextraConfig, EmbeddingBackend};
use contextra_ports::{Rng, TextEmbeddingEngine};
use contextra_types::error::ContextraError;
use contextra_types::DistanceMetric;
use zeroize::Zeroizing;

// ANCHOR[contextra:BLD-001]
/// Konfiguration der Speicherverschlüsselung (Encryption at Rest) für persistente Datenbankschichten.
///
/// Bestimmt, ob Schreib-Lese-Operationen auf Festplatte (LSM-Tree, WAL = Write-Ahead-Log)
/// über AES-GCM-SIV verschlüsselt werden.
/// Gemäß D-05 ist Verschlüsselung standardmäßig obligatorisch.
#[derive(Clone)]
pub enum EncryptionMode {
    /// Storage is encrypted using a key derived from a passphrase.
    Passphrase(Zeroizing<String>),
    /// Explicitly unencrypted mode is disallowed by policy (D-05) and rejected at build time.
    Unencrypted,
}

impl std::fmt::Debug for EncryptionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Passphrase(_) => f.debug_tuple("Passphrase").field(&"[REDACTED]").finish(),
            Self::Unencrypted => write!(f, "Unencrypted"),
        }
    }
}

// ANCHOR[contextra:BLD-002]
/// Erweiterungstraits für [`ContextraConfig`] zur Abfrage des Verschlüsselungsstatus.
pub trait ContextraConfigExt {
    /// Returns `true` if storage encryption is configured with a non-empty passphrase.
    fn is_encrypted(&self) -> bool;
}

impl ContextraConfigExt for ContextraConfig {
    fn is_encrypted(&self) -> bool {
        self.encryption_passphrase
            .as_ref()
            .is_some_and(|pass| !pass.is_empty())
    }
}

// ANCHOR[contextra:BLD-003]
/// Ein Builder zur flexiblen Konfiguration und Instanziierung der Datenbank [`Contextra`].
///
/// `ContextraBuilder` fungiert als primärer Einstiegspunkt (Composition Root) in Ring 4.
/// Er sammelt alle Einstellungen (Pfade, Vektordimensionen, Metriken, Passphrasen) und baut
/// bei [`ContextraBuilder::build`] die vollständige Engine zusammen.
pub struct ContextraBuilder {
    storage_path: PathBuf,
    config: ContextraConfig,
    encryption_mode: Option<EncryptionMode>,
    encryption_error: Option<String>,
    explicit_user_config: bool,
    embedder: Option<Arc<dyn TextEmbeddingEngine>>,
    performance_profile: Option<crate::performance_profile::PerformanceProfile>,
    rng: Option<Arc<dyn Rng>>,
}

impl std::fmt::Debug for ContextraBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextraBuilder")
            .field("storage_path", &self.storage_path)
            .field("config", &self.config)
            .field("encryption_mode", &self.encryption_mode)
            .field("explicit_user_config", &self.explicit_user_config)
            .field(
                "embedder",
                &self.embedder.as_ref().map(|_| "<dyn TextEmbeddingEngine>"),
            )
            .field("performance_profile", &self.performance_profile)
            .field("rng", &self.rng.as_ref().map(|_| "<dyn Rng>"))
            .finish()
    }
}

impl Clone for ContextraBuilder {
    fn clone(&self) -> Self {
        Self {
            storage_path: self.storage_path.clone(),
            config: self.config.clone(),
            encryption_mode: self.encryption_mode.clone(),
            encryption_error: self.encryption_error.clone(),
            explicit_user_config: self.explicit_user_config,
            embedder: self.embedder.clone(),
            performance_profile: self.performance_profile,
            rng: self.rng.clone(),
        }
    }
}

impl ContextraBuilder {
    // ANCHOR[contextra:BLD-004]
    /// Erzeugt einen neuen `ContextraBuilder` mit der angegebenen Vektordimension.
    ///
    /// # Parameter
    /// - `dimension`: Anzahl der Dimensionen für Vektoren (z. B. 768).
    ///
    /// # Standardwerte
    /// - Speicherpfad: `./contextra_data`
    /// - Verschlüsselung: Uninitialisiert (muss vor `build()` gesetzt werden, D-05)
    /// - RNG: Standardmäßig [`OsRngAdapter`] (CSPRNG aus OS-Entropie)
    pub fn new(dimension: usize) -> Self {
        Self {
            storage_path: PathBuf::from("./contextra_data"),
            config: ContextraConfig {
                dimension,
                ..Default::default()
            },
            encryption_mode: None,
            encryption_error: None,
            explicit_user_config: false,
            embedder: None,
            performance_profile: None,
            rng: Some(Arc::new(OsRngAdapter::new())),
        }
    }

    // ANCHOR[contextra:BLD-005]
    /// Setzt das Speicherverzeichnis für die Datenbankdateien (LSM-SSTables, WAL, Indizes).
    ///
    /// # Parameter
    /// - `path`: Pfad im Dateisystem (z. B. `/var/lib/contextra` oder `./data`).
    pub fn with_storage_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.storage_path = path.into();
        self
    }

    // ANCHOR[contextra:BLD-006]
    /// Begrenzt die maximale Anzahl an Vektorelementen im Index.
    ///
    /// # Parameter
    /// - `max_elements`: Maximalwert (z. B. 100_000).
    pub fn with_max_elements(mut self, max_elements: usize) -> Self {
        self.config.max_elements = max_elements;
        self
    }

    // ANCHOR[contextra:BLD-007]
    /// Legt die Distanzmetrik für die Ähnlichkeitssuche fest.
    ///
    /// # Parameter
    /// - `metric`: Metrik wie Kosinus-Ähnlichkeit (`Cosine`), Euklidischer Abstand (`Euclidean`) oder Skalarprodukt (`DotProduct`).
    pub fn with_distance_metric(mut self, metric: DistanceMetric) -> Self {
        self.config.distance_metric = metric;
        self
    }

    // ANCHOR[contextra:BLD-008]
    /// Konfiguriert den Verschlüsselungsmodus explizit.
    ///
    /// Validiert bei Passphrasen sofort, dass diese nicht leer sind oder nur aus Leerzeichen bestehen.
    /// Leere Passphrasen oder unverschlüsselter Modus merken einen Fehler für [`ContextraBuilder::build`] vor (Fail-Closed).
    pub fn with_encryption(mut self, mode: EncryptionMode) -> Self {
        match &mode {
            EncryptionMode::Passphrase(secret) => {
                if secret.is_empty() || secret.trim().is_empty() {
                    self.encryption_error =
                        Some("Encryption passphrase cannot be empty".to_string());
                    return self;
                }
            }
            EncryptionMode::Unencrypted => {
                self.encryption_error = Some(
                    "Unencrypted storage mode is not permitted by policy (D-05); an encryption passphrase is required".to_string(),
                );
                return self;
            }
        }
        self.encryption_mode = Some(mode);
        self.config.encryption_passphrase = None;
        self.encryption_error = None;
        self
    }

    // ANCHOR[contextra:BLD-009]
    /// Bequeme Methode zum Setzen einer Passphrase für die Speicherverschlüsselung.
    ///
    /// Kapselt die Passphrase direkt in einem [`Zeroizing<String>`], um Rückstände im Speicher
    /// zu minimieren.
    pub fn with_encryption_passphrase(self, passphrase: impl Into<String>) -> Self {
        let secret = Zeroizing::new(passphrase.into());
        self.with_encryption(EncryptionMode::Passphrase(secret))
    }

    // ANCHOR[contextra:BLD-010]
    /// Prüft, ob der Builder aktuell auf verschlüsselten Modus mit Passphrase eingestellt ist.
    pub fn is_encrypted(&self) -> bool {
        matches!(self.encryption_mode, Some(EncryptionMode::Passphrase(_)))
    }

    // ANCHOR[contextra:BLD-011]
    /// Wählt das Backend für Text-Embeddings (z. B. intern oder externer Dienst).
    pub fn with_embedding_backend(mut self, backend: EmbeddingBackend) -> Self {
        self.config.embedding_backend = backend;
        self
    }

    // ANCHOR[contextra:BLD-012]
    /// Konfiguriert die automatische Hintergrund-Konsolidierung (Memory Consolidation) von Erinnerungen.
    ///
    /// # Parameter
    /// - `enabled`: Schaltet den periodischen Konsolidierungs-Hintergrundtask an oder aus.
    /// - `interval`: Zeitintervall zwischen zwei Konsolidierungsläufen.
    pub fn with_consolidation(mut self, enabled: bool, interval: Duration) -> Self {
        self.config.consolidation_enabled = enabled;
        self.config.consolidation_interval = interval;
        self
    }

    // ANCHOR[contextra:BLD-013]
    /// Bindet eine benutzerdefinierte Implementierung des [`TextEmbeddingEngine`]-Ports ein.
    pub fn with_embedder(mut self, embedder: Arc<dyn TextEmbeddingEngine>) -> Self {
        self.embedder = Some(embedder);
        self
    }

    // ANCHOR[contextra:BLD-015]
    /// Injiziert eine benutzerdefinierte Zufallsquelle ([`contextra_ports::Rng`]), z. B. für deterministische Tests.
    pub fn with_rng(mut self, rng: Arc<dyn Rng>) -> Self {
        self.rng = Some(rng);
        self
    }

    // ANCHOR[contextra:BLD-016]
    /// Setzt ein vordefiniertes Performance-Profil ([`crate::performance_profile::PerformanceProfile`]).
    pub fn with_performance_profile(
        mut self,
        profile: crate::performance_profile::PerformanceProfile,
    ) -> Self {
        self.performance_profile = Some(profile);
        self
    }

    // ANCHOR[contextra:BLD-017]
    /// Überschreibt die Konfiguration vollständig mit einer bestehenden [`ContextraConfig`].
    ///
    /// Überträgt auch eventuell in `config` enthaltene Passphrasen in den abgesicherten `EncryptionMode`.
    pub fn with_config(mut self, mut config: ContextraConfig) -> Self {
        if let Some(passphrase) = config.encryption_passphrase.take() {
            if !passphrase.is_empty() && !passphrase.trim().is_empty() {
                let secret = Zeroizing::new(passphrase);
                self.encryption_mode = Some(EncryptionMode::Passphrase(secret));
            } else {
                self.encryption_error = Some("Encryption passphrase cannot be empty".to_string());
            }
        }
        self.config = config;
        self.config.encryption_passphrase = None;
        self.explicit_user_config = true;
        self
    }

    // ANCHOR[contextra:BLD-018]
    /// Baut und initialisiert die [`Contextra`]-Datenbankinstanz asynchron.
    ///
    /// # Ablauf von `build`:
    /// 1. **Vorabprüfung**: Prüfe, ob Konfigurationsfehler (z. B. leere Passphrase) vorliegen.
    /// 2. **Pflicht-Verschlüsselung (D-05)**: Erzwinge zwingend gesetzte, nicht-leere Passphrase.
    /// 3. **Performance-Profil auflösen**: Falls ein Profil gesetzt ist, Durability- und Löschmodi übernehmen.
    /// 4. **Instanziierung**: Aufruf von `Contextra::open_with_config` mit Pfad und Konfiguration.
    /// 5. **Embedder anhängen**: Falls ein Text-Embedder hinterlegt wurde, diesen registrieren.
    ///
    /// # Fehlerfälle
    /// - [`ContextraError::InvalidInput`]: Fehlende oder leere Passphrase oder unverschlüsselter Modus angefordert.
    /// - [`ContextraError::PolicyViolation`]: Konflikt zwischen Profil und Konfiguration.
    /// - [`ContextraError::Io`]: Dateisystem- oder Speicherinitialisierungsfehler.
    pub async fn build(self) -> Result<Contextra, ContextraError> {
        if let Some(ref err_msg) = self.encryption_error {
            return Err(ContextraError::invalid_input(err_msg.clone()));
        }

        let mut config_for_open = self.config.clone();

        match &self.encryption_mode {
            Some(EncryptionMode::Passphrase(secret)) => {
                if secret.is_empty() || secret.trim().is_empty() {
                    return Err(ContextraError::invalid_input(
                        "Encryption passphrase cannot be empty",
                    ));
                }
                config_for_open.encryption_passphrase = Some(secret.as_str().to_string());
            }
            Some(EncryptionMode::Unencrypted) => {
                return Err(ContextraError::invalid_input(
                    "Unencrypted storage mode is not permitted by policy (D-05); an encryption passphrase is required",
                ));
            }
            None => {
                return Err(ContextraError::invalid_input(
                    "Encryption passphrase is required by policy (D-05); unconfigured encryption mode is not allowed",
                ));
            }
        }

        if let Some(profile) = self.performance_profile {
            let resolved = profile.resolve();
            if self.explicit_user_config {
                if config_for_open.durability_mode != resolved.durability_mode
                    || config_for_open.deletion_proof_active != resolved.deletion_proof_active
                    || config_for_open.vector_delete_mode != resolved.vector_delete_mode
                {
                    return Err(ContextraError::PolicyViolation(
                        "Explicit ContextraConfig conflicts with PerformanceProfile settings"
                            .to_string(),
                    ));
                }
            } else {
                config_for_open.durability_mode = resolved.durability_mode;
                config_for_open.deletion_proof_active = resolved.deletion_proof_active;
                config_for_open.vector_delete_mode = resolved.vector_delete_mode;
            }
        }

        let instance = Contextra::open_with_config(&self.storage_path, config_for_open).await?;

        if let Some(embedder) = self.embedder {
            Ok(instance.with_embedder(embedder).await)
        } else {
            Ok(instance)
        }
    }
}

impl Default for ContextraBuilder {
    fn default() -> Self {
        Self::new(768)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    // ANCHOR[contextra:TEST-BLD-001]
    // Prüft, ob der Builder mit den erwarteten Default-Werten (Dimension 768, Standardpfad) startet.
    #[test]
    fn test_builder_defaults() {
        let builder = ContextraBuilder::default();
        assert_eq!(builder.config.dimension, 768);
        assert_eq!(builder.storage_path, PathBuf::from("./contextra_data"));
        assert!(builder.embedder.is_none());
        assert!(!builder.is_encrypted());
    }

    // ANCHOR[contextra:TEST-BLD-002]
    // Prüft das Setzen von benutzerdefinierten Parametern (Dimension, Pfad, Max-Elemente, Metrik, Passphrase, Konsolidierung).
    #[test]
    fn test_builder_custom_configuration() {
        let builder = ContextraBuilder::new(128)
            .with_storage_path("/tmp/test_contextra_store")
            .with_max_elements(50000)
            .with_distance_metric(DistanceMetric::Euclidean)
            .with_encryption_passphrase("secret_pass")
            .with_consolidation(false, Duration::from_secs(300));

        assert_eq!(builder.config.dimension, 128);
        assert_eq!(
            builder.storage_path,
            PathBuf::from("/tmp/test_contextra_store")
        );
        assert_eq!(builder.config.max_elements, 50000);
        assert_eq!(builder.config.distance_metric, DistanceMetric::Euclidean);
        assert!(builder.is_encrypted());
        assert!(!builder.config.consolidation_enabled);
        assert_eq!(
            builder.config.consolidation_interval,
            Duration::from_secs(300)
        );
    }

    // ANCHOR[contextra:TEST-BLD-003]
    // Verifiziert die Maskierung von Passphrasen ([REDACTED]) in Debug-Ausgaben von EncryptionMode und ContextraBuilder.
    #[test]
    fn test_encryption_passphrase_redaction() {
        let secret_str = "super_secret_master_key_12345";
        let mode = EncryptionMode::Passphrase(Zeroizing::new(secret_str.to_string()));
        let debug_mode_output = format!("{:?}", mode);
        assert!(!debug_mode_output.contains(secret_str));
        assert!(debug_mode_output.contains("[REDACTED]"));

        let builder = ContextraBuilder::new(256).with_encryption_passphrase(secret_str);
        let debug_builder_output = format!("{:?}", builder);
        assert!(!debug_builder_output.contains(secret_str));
        assert!(debug_builder_output.contains("[REDACTED]"));
    }

    // ANCHOR[contextra:TEST-BLD-004]
    // Stellt sicher, dass leere oder reine Whitespace-Passphrasen beim Bauen abgelehnt werden.
    #[tokio::test]
    async fn test_empty_passphrase_rejected() {
        let res = ContextraBuilder::new(4)
            .with_encryption_passphrase("")
            .build()
            .await;
        assert!(res.is_err());
        if let Err(ContextraError::InvalidInput(msg)) = res {
            assert!(msg.contains("cannot be empty"));
        } else {
            panic!("Expected InvalidInput error for empty passphrase");
        }

        let res_ws = ContextraBuilder::new(4)
            .with_encryption(EncryptionMode::Passphrase(Zeroizing::new(
                "   ".to_string(),
            )))
            .build()
            .await;
        assert!(res_ws.is_err());
    }

    // ANCHOR[contextra:TEST-BLD-005]
    // Prüft, dass der explizit unverschlüsselte Modus gemäß D-05 abgewiesen wird.
    #[tokio::test]
    async fn test_explicit_unencrypted_mode_rejected() {
        let tmp_path =
            std::env::temp_dir().join(format!("contextra_unencrypted_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_path);

        let builder = ContextraBuilder::new(4)
            .with_storage_path(&tmp_path)
            .with_encryption(EncryptionMode::Unencrypted);

        let res = builder.build().await;
        assert!(res.is_err());
        if let Err(ContextraError::InvalidInput(msg)) = res {
            assert!(msg.contains("Unencrypted storage mode is not permitted"));
        } else {
            panic!("Expected InvalidInput error for Unencrypted mode");
        }

        let _ = std::fs::remove_dir_all(&tmp_path);
    }

    // ANCHOR[contextra:TEST-BLD-006]
    // Prüft, dass das Weglassen jeglicher Passphrase gemäß D-05 mit einem klaren Fehler endet.
    #[tokio::test]
    async fn test_unconfigured_encryption_rejected() {
        let tmp_path =
            std::env::temp_dir().join(format!("contextra_no_enc_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_path);

        let res = ContextraBuilder::new(4)
            .with_storage_path(&tmp_path)
            .build()
            .await;

        assert!(res.is_err());
        if let Err(ContextraError::InvalidInput(msg)) = res {
            assert!(msg.contains("Encryption passphrase is required"));
        } else {
            panic!("Expected InvalidInput error for unconfigured encryption");
        }

        let _ = std::fs::remove_dir_all(&tmp_path);
    }

    // ANCHOR[contextra:TEST-BLD-007]
    // Testet einen vollständigen Durchlauf von Instanzerstellung, Initialisierung und Ressourcenfreigabe mit Passphrase.
    #[tokio::test]
    async fn test_builder_build_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let tmp_path =
            std::env::temp_dir().join(format!("contextra_builder_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_path);

        let db = ContextraBuilder::new(4)
            .with_storage_path(&tmp_path)
            .with_distance_metric(DistanceMetric::Cosine)
            .with_encryption_passphrase("valid-passphrase-roundtrip")
            .build()
            .await?;

        assert_eq!(db.len().await?, 0);
        let _ = std::fs::remove_dir_all(&tmp_path);
        Ok(())
    }

    /// T-ID: T-DEF-1
    #[tokio::test]
    async fn test_builder_builds_without_license_gate() -> Result<(), Box<dyn std::error::Error>> {
        let tmp_path = std::env::temp_dir().join(format!(
            "contextra_builder_no_license_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&tmp_path);

        let db = ContextraBuilder::new(8)
            .with_storage_path(&tmp_path)
            .with_encryption_passphrase("test-passphrase-without-license")
            .build()
            .await?;

        assert_eq!(db.config().dimension, 8);
        db.close().await?;
        let _ = std::fs::remove_dir_all(&tmp_path);
        Ok(())
    }
}

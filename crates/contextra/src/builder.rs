// FILE-CONTEXT
// ZWECK: Contextra Composition Root Builder (Ring 4).
// INVARIANTEN: Zero-panic in production code; orchestriert Subsystem-Konfiguration ohne Geschäftslogik.

//! # Modul: Contextra Composition Root Builder (Ring 4)
//!
//! Dieses Modul stellt den zentralen Builder (`ContextraBuilder`) zur Verfügung, welcher als
//! **Composition Root** dient: Hier fließen alle Subsysteme (Speicher, Lizenzprüfung, Verschlüsselung,
//! Embedding-Engine, Performance-Profile) zusammen und werden initial verdrahtet.
//!
//! ## Ring-Zugehörigkeit
//! - **Ring 4 (Fassade)**: Enthält selbst **keine Geschäftslogik**, sondern orchestriert
//!   ausschließlich die Konfiguration und Übergabe von Parametern an `contextra-db` (Ring 3).
//!
//! ## Wichtigste Typen
//! - [`EncryptionMode`]: Steuert, ob und wie Daten im Ruhezustand (Data-at-Rest) verschlüsselt werden.
//! - [`ContextraConfigExt`]: Hilfstraiterweiterung für Konfigurationsabfragen bezüglich Verschlüsselung.
//! - [`ContextraBuilder`]: Die fluide Builder-Struktur zur Konstruktion einer lauffähigen Datenbank.
//!
//! ## Entwurfs- & Sicherheitsrichtlinien
//! - **Zero-Panic-Doktrin**: Keine unkontrollierten Panics im Produktionscode (`unwrap` / `expect` verboten).
//!   Fehler werden als typisierte [`ContextraError`]-Werte zurückgegeben.
//! - **Sichere Geheimnisverwaltung**: Passphrasen werden umgehend in [`Zeroizing<String>`] gekapselt,
//!   sodass ihr Speicherbereich beim Verlassen des Gültigkeitsbereichs mit Nullen überschrieben wird.
//!   Zudem wird in Debug-Ausgaben der Platzhalter `[REDACTED]` ausgegeben.
//! - **Lizenzprüfung (Fail-Closed)**: Zugriffe auf erweiterte Ringe (z. B. Sovereign) werden über
//!   ein [`LicenseGate`] vor dem Hochfahren autorisiert. Ungültige Lizenzen führen zum Abbruch.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use contextra_db::{Contextra, ContextraConfig, EmbeddingBackend, TextEmbeddingEngine};
use contextra_license::SignedLicenseGate;
use contextra_ports::license::{FeatureRing, LicenseError, LicenseGate, OpenFastGate};
use contextra_types::error::ContextraError;
use contextra_types::DistanceMetric;
use zeroize::Zeroizing;

// ANCHOR[contextra:BLD-001]
/// Konfiguration der Speicherverschlüsselung (Encryption at Rest) für persistente Datenbankschichten.
///
/// Bestimmt, ob Schreib-Lese-Operationen auf Festplatte (LSM-Tree, WAL = Write-Ahead-Log)
/// über AES-GCM-SIV verschlüsselt werden.
#[derive(Clone)]
pub enum EncryptionMode {
    /// Storage is encrypted using a key derived from a passphrase.
    Passphrase(Zeroizing<String>),
    /// Storage is explicitly unencrypted.
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
/// Er sammelt alle Einstellungen (Pfade, Vektordimensionen, Metriken, Lizenzen) und baut
/// bei [`ContextraBuilder::build`] die vollständige Engine zusammen.
pub struct ContextraBuilder {
    storage_path: PathBuf,
    config: ContextraConfig,
    encryption_mode: Option<EncryptionMode>,
    encryption_error: Option<String>,
    explicit_user_config: bool,
    embedder: Option<Arc<dyn TextEmbeddingEngine>>,
    license_gate: Arc<dyn LicenseGate>,
    signed_license_error: Option<String>,
    performance_profile: Option<crate::performance_profile::PerformanceProfile>,
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
            .field("license_gate", &"<dyn LicenseGate>")
            .field("signed_license_error", &self.signed_license_error)
            .field("performance_profile", &self.performance_profile)
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
            license_gate: Arc::clone(&self.license_gate),
            signed_license_error: self.signed_license_error.clone(),
            performance_profile: self.performance_profile,
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
    /// - Lizenzgate: `OpenFastGate` (Entwicklungsmodus ohne Lizenzschlüssel)
    /// - Verschlüsselung: Uninitialisiert (wird bei `build` gewarnt oder auf unverschlüsselt gesetzt)
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
            license_gate: Arc::new(OpenFastGate),
            signed_license_error: None,
            performance_profile: None,
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
    /// Leere Passphrasen merken einen Fehler für [`ContextraBuilder::build`] vor (Fail-Closed).
    pub fn with_encryption(mut self, mode: EncryptionMode) -> Self {
        match &mode {
            EncryptionMode::Passphrase(secret) => {
                if secret.is_empty() || secret.trim().is_empty() {
                    self.encryption_error =
                        Some("Encryption passphrase cannot be empty".to_string());
                    return self;
                }
            }
            EncryptionMode::Unencrypted => {}
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
    /// Wählt das Backend für Text-Embeddings (z. B. intern via Candle oder externer Dienst).
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

    // ANCHOR[contextra:BLD-014]
    /// Injiziert ein beliebiges Lizenzgate ([`LicenseGate`]) zur Ring-Autorisierung.
    pub fn with_license_gate(mut self, gate: Arc<dyn LicenseGate>) -> Self {
        self.license_gate = gate;
        self.signed_license_error = None;
        self
    }

    // ANCHOR[contextra:BLD-015]
    /// Konfiguriert ein kryptografisch signiertes Lizenzgate (Ed25519-Signaturprüfung).
    ///
    /// Überprüft die Signatur des Lizenz-Payloads mit dem angegebenen öffentlichen Verifizierungsschlüssel.
    /// Schlägt die Verifikation fehl, wird ein Fehler gespeichert, der beim Aufruf von `build()` fehlschlägt.
    pub fn with_signed_license(
        mut self,
        payload_bytes: &[u8],
        signature: &[u8; 64],
        verifying_key_bytes: &[u8; 32],
    ) -> Self {
        match contextra_license::signed_gate::VerifyingKey::from_bytes(verifying_key_bytes) {
            Ok(key) => {
                match SignedLicenseGate::from_signed_payload(payload_bytes, signature, key) {
                    Ok(gate) => {
                        self.license_gate = Arc::new(gate);
                        self.signed_license_error = None;
                    }
                    Err(e) => {
                        self.signed_license_error = Some(e.to_string());
                    }
                }
            }
            Err(_) => {
                self.signed_license_error = Some(LicenseError::InvalidSignature.to_string());
            }
        }
        self
    }

    // ANCHOR[contextra:BLD-016]
    /// Setzt ein vordefiniertes Performance-Profil ([`PerformanceProfile`]).
    ///
    /// Profile (wie `Compliance`, `Balanced`, `BareMetal`) steuern standardisierte
    /// Kombinationen aus Durability (WAL-fsync), Vektor-Löschmodi und Lizenzringen.
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
    /// 1. **Vorabprüfung**: Prüfe, ob frühere Fehler (z. B. leere Passphrase oder ungültige Signatur) vorliegen.
    /// 2. **Verschlüsselung konfigurieren**: Passphrase aus dem `EncryptionMode` in die Laufzeitkonfiguration übertragen.
    /// 3. **Performance-Profil auflösen**: Falls ein Profil gesetzt ist, Durability- und Löschmodi übernehmen
    ///    oder bei Konflikten mit expliziten Parametern abbrechen.
    /// 4. **Lizenz-Autorisierung**: Den angeforderten Ring (z. B. `Fast` oder `Sovereign`) über `license_gate.authorize` abfragen.
    /// 5. **Instanziierung**: Aufruf von `Contextra::open_authorized` mit Pfad, Konfiguration und Token.
    /// 6. **Embedder anhängen**: Falls ein benutzerdefinierter Text-Embedder hinterlegt wurde, diesen registrieren.
    ///
    /// # Fehlerfälle
    /// - [`ContextraError::InvalidInput`]: Leere Passphrase.
    /// - [`ContextraError::PolicyViolation`]: Lizenz unzureichend oder Konflikt zwischen Profil und Konfiguration.
    /// - [`ContextraError::Io`]: Dateisystem- oder Speicherinitialisierungsfehler.
    pub async fn build(self) -> Result<Contextra, ContextraError> {
        if let Some(ref err_msg) = self.encryption_error {
            return Err(ContextraError::invalid_input(err_msg.clone()));
        }

        if let Some(ref err_msg) = self.signed_license_error {
            return Err(ContextraError::PolicyViolation(err_msg.clone()));
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
                config_for_open.encryption_passphrase = None;
            }
            None => {
                tracing::warn!(
                    "Contextra started in implicit unencrypted mode; explicit EncryptionMode selection will be required in a future version"
                );
                config_for_open.encryption_passphrase = None;
            }
        }

        let requested_ring = if let Some(profile) = self.performance_profile {
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
            resolved.feature_ring
        } else {
            FeatureRing::Fast
        };

        let authorized_token = self
            .license_gate
            .authorize(requested_ring)
            .map_err(|e| ContextraError::PolicyViolation(e.to_string()))?;

        let instance = Contextra::open_authorized(
            &self.storage_path,
            config_for_open,
            &authorized_token,
            self.license_gate,
        )
        .await?;

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
    // Prüft, ob der Builder mit den erwarteten Default-Werten (Dimension 768, Standardpfad, keine Verschlüsselung) startet.
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
            .with_storage_path("/tmp/test_contextra_db")
            .with_max_elements(50000)
            .with_distance_metric(DistanceMetric::Euclidean)
            .with_encryption_passphrase("secret_pass")
            .with_consolidation(false, Duration::from_secs(300));

        assert_eq!(builder.config.dimension, 128);
        assert_eq!(
            builder.storage_path,
            PathBuf::from("/tmp/test_contextra_db")
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
    // Prüft, dass der explizit unverschlüsselte Modus korrekt initialisiert werden kann.
    #[tokio::test]
    async fn test_explicit_unencrypted_mode() -> Result<(), Box<dyn std::error::Error>> {
        let tmp_path =
            std::env::temp_dir().join(format!("contextra_unencrypted_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_path);

        let builder = ContextraBuilder::new(4)
            .with_storage_path(&tmp_path)
            .with_encryption(EncryptionMode::Unencrypted);

        assert!(!builder.is_encrypted());

        let db = builder.build().await?;
        assert_eq!(db.len().await?, 0);

        let _ = std::fs::remove_dir_all(&tmp_path);
        Ok(())
    }

    // ANCHOR[contextra:TEST-BLD-006]
    // Testet einen vollständigen Durchlauf von Instanzerstellung, Initialisierung und Ressourcenfreigabe.
    #[tokio::test]
    async fn test_builder_build_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let tmp_path =
            std::env::temp_dir().join(format!("contextra_builder_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_path);

        let db = ContextraBuilder::new(4)
            .with_storage_path(&tmp_path)
            .with_distance_metric(DistanceMetric::Cosine)
            .build()
            .await?;

        assert_eq!(db.len().await?, 0);
        let _ = std::fs::remove_dir_all(&tmp_path);
        Ok(())
    }
}

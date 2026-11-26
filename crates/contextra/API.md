# API-Dokumentation: `contextra` (Fassade)

Dieses Dokument beschreibt die öffentlichen Schnittstellen der Fassaden-Crate `contextra`.

## Einstiegspunkte (`lib.rs`)

| Funktion | Signatur | Beschreibung |
|---|---|---|
| `builder` | `pub fn builder(dimension: usize) -> ContextraBuilder` | Erzeugt einen neuen `ContextraBuilder` mit angegebener Vektordimension. |
| `open` | `pub async fn open(path: impl AsRef<Path>) -> Result<Contextra, ContextraError>` | Öffnet oder erstellt eine Contextra-Instanz am Pfad mit Standardeinstellungen (Dimension 768). |
| `open_with_config` | `pub async fn open_with_config(path: impl AsRef<Path>, config: ContextraConfig) -> Result<Contextra, ContextraError>` | Öffnet eine Contextra-Instanz mit benutzerdefinierter Konfiguration. |

## Builder (`builder.rs`)

| Typ / Methode | Beschreibung |
|---|---|
| `ContextraBuilder::new(dimension: usize) -> Self` | Initialisiert den Builder mit Vektordimension. |
| `with_storage_path(path: impl Into<PathBuf>) -> Self` | Konfiguriert das Speicherverzeichnis. |
| `with_max_elements(max_elements: usize) -> Self` | Setzt das Elementlimit für Vektoren. |
| `with_distance_metric(metric: DistanceMetric) -> Self` | Konfiguriert Ähnlichkeitsmetrik (Cosine, Euclidean, DotProduct). |
| `with_encryption(mode: EncryptionMode) -> Self` | Setzt den Verschlüsselungsmodus explizit. |
| `with_encryption_passphrase(passphrase: impl Into<String>) -> Self` | Setzt Passphrase mit automatischer Zeroization. |
| `with_embedding_backend(backend: EmbeddingBackend) -> Self` | Wählt das Embedding-Backend. |
| `with_consolidation(enabled: bool, interval: Duration) -> Self` | Konfiguriert Speicher-Konsolidierung. |
| `with_embedder(embedder: Arc<dyn TextEmbeddingEngine>) -> Self` | Registriert externen Embedder. |
| `with_performance_profile(profile: PerformanceProfile) -> Self` | Setzt Leistungs- und Persistenzprofil. |
| `with_config(config: ContextraConfig) -> Self` | Überschreibt Gesamtkonfiguration. |
| `build(self) -> Result<Contextra, ContextraError>` | Baut und initialisiert die Contextra-Datenbankinstanz (lizenzfrei, direkter Durchlauf). |
| `#[deprecated] with_license_gate(self, _gate: Arc<dyn LicenseGate>) -> Self` | Veraltet: No-op; Lizenz-Gates wurden entfernt. |

## Profile (`performance_profile.rs`, `collection_profile.rs`)

| Typ / Methode | Beschreibung |
|---|---|
| `PerformanceProfile` | Enums: `Compliance`, `Balanced`, `BareMetal`. |
| `DurabilityMode` | Enums: `Full`, `WalNoHmac`, `MemoryOnly`. |
| `ResolvedProfileConfig` | Vollständig aufgelöste Profilkonfiguration. |
| `#[deprecated] ResolvedProfileConfig::enforce_license` | Veraltet: No-op; liefert immer `Ok(())`. |
| `CollectionProfile` | Konfiguration für Sammlungen (Performance, KV-Löschung, LSM-Tuning, Extraktion). |
| `DeploymentTier` | Vordefinierte Tiers: `EdgeMinimal`, `PowerUserLocal`, `EnterpriseShared`, `EnterpriseRegulated`. |
| `#[deprecated] CollectionProfile::validate_with_license` | Veraltet: No-op bzgl. Lizenzgate; validiert Profilkonsistenz. |

## Agent Memory (`agent_memory.rs`)

| Typ / Methode | Beschreibung |
|---|---|
| `AgentMemory::new(db: Contextra) -> Self` | Initialisiert die Agenten-Erinnerungsschicht über Contextra. |
| `remember(&self, content: impl Into<String>, tags: Option<Vec<String>>) -> Result<MemoryId, ContextraError>` | Speichert Inhalt als Erinnerung. |
| `recall(&self, query: &str, limit: usize) -> Result<Vec<Memory>, ContextraError>` | Ruft relevante Erinnerungen ab. |
| `forget(&self, id: &MemoryId) -> Result<bool, ContextraError>` | Löscht eine Erinnerung. |
| `relate(&self, from: &MemoryId, to: &MemoryId, relation: &str) -> Result<(), ContextraError>` | Verknüpft Erinnerungen im Wissensgraphen. |

use crate::error::{ContextraError, Result};
use crate::types::domain::{DocId, EntityId, TxId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

mod hex {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut hex_str = String::with_capacity(64);
        for &b in bytes {
            use std::fmt::Write;
            if write!(&mut hex_str, "{:02x}", b).is_err() {
                return Err(serde::ser::Error::custom("formatting hex failed"));
            }
        }
        serializer.serialize_str(&hex_str)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.len() != 64 {
            return Err(serde::de::Error::custom(format!(
                "invalid hex string length {}, expected 64",
                s.len()
            )));
        }
        let mut bytes = [0u8; 32];
        for i in 0..32 {
            bytes[i] =
                u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).map_err(serde::de::Error::custom)?;
        }
        Ok(bytes)
    }
}

/// Defines a frozen workflow state acting as a savepoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowState {
    /// Associated transaction.
    pub tx: TxId,
    /// Agent memory graph state footprint (BLAKE3 hash digest).
    #[serde(with = "hex")]
    pub graph_hash: [u8; 32],
}

// INVARIANT: Bit 63 der SeqNo markiert Tombstones.
/// Bit mask for identifying tombstones in sequence numbers.
pub const TOMBSTONE_BIT: u64 = 1 << 63;

/// Reserved metadata key for sequence-based document TTL expiration.
pub const EXPIRY_METADATA_KEY: &str = "__expires_at_seq";

/// Maximum number of search results that any hybrid/vector/text search may return.
///
/// Callers in contextra-mcp and contextra-db both enforce this limit before forwarding k to HNSW/BM25.
/// Duplicating the literal 1000 anywhere else in the workspace is prohibited — always import this constant.
///
/// This cap is applied at the orchestration layer (contextra-db) before forwarding `k`
/// to HNSW and BM25 sub-searches. All upstream layers (contextra-mcp, contextra-py,
/// contextra-py) MUST reference this constant — never duplicate the literal `1000`.
///
/// # DECISION-REF
/// AGT-DB-003 — Boundary defence at Layer 2 against unbounded `k` from untrusted JSON-RPC.
pub const MAX_SEARCH_K: usize = 1_000;

/// Relationship types between Zettelkasten memory chunks (A-MEM).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinkRelation {
    /// Target memory provides more detail or context.
    Elaborates,
    /// Target memory contradicts this memory.
    Contradicts,
    /// Target memory supersedes this memory, replacing its context.
    Supersedes,
    /// General reference without specific semantics.
    References,
}

/// A directional link to another memory chunk in the Zettelkasten.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryLink {
    /// The target document ID.
    pub target: DocId,
    /// The type of relationship.
    pub relation: LinkRelation,
    /// The transaction ID when this link was created.
    pub created_at_tx: TxId,
}

/// Represents a canonical node in the knowledge graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    /// Unique entity ID.
    pub id: EntityId,
    /// Canonical human-readable name.
    pub name: Arc<str>,
    /// Categorical entity type.
    pub entity_type: Arc<str>,
    /// Flexible key-value attribute metadata map.
    #[serde(default)]
    pub attributes: ahash::AHashMap<String, serde_json::Value>,
}

impl Entity {
    /// Creates a new `Entity` with id, name, and type.
    pub fn new(id: EntityId, name: impl Into<String>, entity_type: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into().into(),
            entity_type: entity_type.into().into(),
            attributes: Default::default(),
        }
    }

    /// Creates a new `Entity`, validating that `name` and `entity_type` are non-empty.
    ///
    /// # Errors
    /// Returns `ContextraError::InvalidInput` if `name` or `entity_type` is empty or whitespace-only.
    pub fn try_new(
        id: EntityId,
        name: impl Into<String>,
        entity_type: impl Into<String>,
    ) -> Result<Self> {
        let name_str = name.into();
        let type_str = entity_type.into();
        if name_str.trim().is_empty() {
            return Err(ContextraError::InvalidInput(
                "Entity name cannot be empty".to_string(),
            ));
        }
        if type_str.trim().is_empty() {
            return Err(ContextraError::InvalidInput(
                "Entity type cannot be empty".to_string(),
            ));
        }
        Ok(Self {
            id,
            name: name_str.into(),
            entity_type: type_str.into(),
            attributes: Default::default(),
        })
    }
}

/// Graph directed edge representation with explicit bitemporal axis separation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// Source entity identifier.
    pub from: EntityId,
    /// Destination entity identifier.
    pub to: EntityId,
    /// Relationship label.
    pub label: Arc<str>,
    /// Numeric relationship weight (default 1.0).
    pub weight: f32,
    /// Start of transaction validity (system time / MVCC); None = valid from beginning of transaction history.
    #[serde(default, alias = "valid_from")]
    pub tx_valid_from: Option<TxId>,
    /// End of transaction validity (system time / MVCC); None = currently valid transaction state.
    #[serde(default, alias = "valid_to")]
    pub tx_valid_to: Option<TxId>,
    /// Start of business validity (business time in Unix ms); None = valid from beginning of business time.
    #[serde(default)]
    pub business_valid_from: Option<i64>,
    /// End of business validity (business time in Unix ms); None = currently valid business state.
    #[serde(default)]
    pub business_valid_to: Option<i64>,
    /// Optional source document ID from which this edge was derived.
    #[serde(default)]
    pub source_doc_id: Option<DocId>,
}

impl Edge {
    /// Creates a new `Edge` between source and target entities with a label.
    pub fn new(from: EntityId, to: EntityId, label: impl Into<String>) -> Self {
        Self {
            from,
            to,
            label: label.into().into(),
            weight: 1.0,
            tx_valid_from: None,
            tx_valid_to: None,
            business_valid_from: None,
            business_valid_to: None,
            source_doc_id: None,
        }
    }

    /// Sets the source document ID from which this edge was derived.
    pub fn with_source_doc_id(mut self, doc_id: DocId) -> Self {
        self.source_doc_id = Some(doc_id);
        self
    }

    /// Creates a new `Edge`, validating non-empty label and finite non-negative weight.
    ///
    /// # Errors
    /// Returns `ContextraError::InvalidInput` if `label` is empty or `weight` is NaN/infinite/negative.
    pub fn try_new(
        from: EntityId,
        to: EntityId,
        label: impl Into<String>,
        weight: f32,
    ) -> Result<Self> {
        let label_str = label.into();
        if label_str.trim().is_empty() {
            return Err(ContextraError::InvalidInput(
                "Edge label cannot be empty".to_string(),
            ));
        }
        if !weight.is_finite() || weight < 0.0 {
            return Err(ContextraError::InvalidInput(
                "Edge weight must be finite and non-negative".to_string(),
            ));
        }
        Ok(Self {
            from,
            to,
            label: label_str.into(),
            weight,
            tx_valid_from: None,
            tx_valid_to: None,
            business_valid_from: None,
            business_valid_to: None,
            source_doc_id: None,
        })
    }

    /// Sets a custom weight on the edge.
    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight;
        self
    }

    /// Sets transaction validity window (system time / MVCC) on the edge.
    pub fn with_tx_validity(mut self, from: Option<TxId>, to: Option<TxId>) -> Self {
        self.tx_valid_from = from;
        self.tx_valid_to = to;
        self
    }

    /// Sets business validity window (business time in Unix ms) on the edge.
    pub fn with_business_validity(mut self, from: Option<i64>, to: Option<i64>) -> Self {
        self.business_valid_from = from;
        self.business_valid_to = to;
        self
    }

    /// Sets transaction validity window on the edge (alias for `with_tx_validity` for backward compatibility).
    pub fn with_validity(self, from: Option<TxId>, to: Option<TxId>) -> Self {
        self.with_tx_validity(from, to)
    }
}

/// Klassifiziert den kognitiven Gedächtnistyp einer gespeicherten Einheit.
///
/// # Persistence
/// Wird als Teil der Dokument-Metadaten serialisiert (JSON-Feld "memory_type").
/// Rückwärtskompatibel: Fehlendes Feld wird als MemoryType::Semantic deserialisiert
/// (bisherige Dokumente ohne Klassifikation = faktisches Wissen).
///
/// # Non-Exhaustive
/// `#[non_exhaustive]` erlaubt in zukünftigen Releases neue Varianten ohne
/// Breaking Change bei downstream match-Ausdrücken (KEIN wildcard-arm zwingend
/// für Library-Consumer bis zur nächsten Major-Version).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum MemoryType {
    /// Episodisches Gedächtnis: Erlebnisse, Unterhaltungen, zeitlich verankerte Ereignisse.
    /// Retrieval: Zeitliche Nähe und Relevanz zur aktuellen Session.
    /// Decay: Hohe Recency-Decay-Rate (Informationen veralten schnell).
    #[serde(alias = "Episodic")]
    Episodic,

    /// Semantisches Gedächtnis: Fakten, Konzepte, dauerhaftes Wissen.
    /// Retrieval: Inhaltliche Ähnlichkeit (Vektor + BM25).
    /// Decay: Keine automatische Decay (Fakten bleiben gültig bis Widerspruch).
    #[default]
    #[serde(alias = "Semantic")]
    Semantic,

    /// Prozedurales Gedächtnis: Abläufe, Tool-Nutzungsmuster, Workflows.
    /// Retrieval: Task-Matching (zukünftig: Instruktionskodierung).
    /// Decay: Aktivierungsbasiert (wird durch Nutzung gestärkt).
    #[serde(alias = "Procedural")]
    Procedural,

    /// Operatives Arbeitsgedächtnis: Kurzzeit-Kontext der aktuellen Session.
    /// Lebensdauer: Session-scoped, automatisch bei Session-Ende ablaufend.
    /// Decay: Sehr hohe Decay-Rate (Session-TTL, z. B. 30 Minuten Inaktivität).
    #[serde(alias = "Working")]
    Working,
}

impl MemoryType {
    /// Gibt den Standard-Decay-Typ für diesen Gedächtnistyp zurück.
    pub fn default_decay(&self) -> crate::types::importance::DecayFunction {
        match self {
            MemoryType::Episodic => crate::types::importance::DecayFunction::Exponential {
                half_life_tx: 10_000, // ca. 10.000 Transaktionen ≈ moderate Abnahme
            },
            MemoryType::Semantic => crate::types::importance::DecayFunction::None,
            MemoryType::Procedural => crate::types::importance::DecayFunction::StepFloor {
                access_count_floor: 50, // Verstärkt durch Nutzung
            },
            MemoryType::Working => crate::types::importance::DecayFunction::Exponential {
                half_life_tx: 500, // Sehr schnelle Abnahme
            },
        }
    }

    /// Gibt die empfohlene TTL (in Transaktionen) für Session-scoped Working Memory zurück.
    /// None bedeutet kein automatisches Ablaufen.
    pub fn default_ttl_tx(&self) -> Option<u64> {
        match self {
            MemoryType::Working => Some(50_000), // ~50.000 TX ≈ 30 Minuten bei normalem Tempo
            _ => None,
        }
    }

    /// Gibt den kanonischen Metadaten-Key zurück (für JSON-Serialisierung).
    pub fn as_metadata_key(&self) -> &'static str {
        match self {
            MemoryType::Episodic => "episodic",
            MemoryType::Semantic => "semantic",
            MemoryType::Procedural => "procedural",
            MemoryType::Working => "working",
        }
    }
}

/// Selection of algorithm strategy for Personalized PageRank (PPR).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PprAlgorithm {
    /// Auto heuristic dispatch based on seed count (ForwardPush for <= 100 seeds, DensePowerIteration otherwise).
    #[default]
    Auto,
    /// Dense power-iteration algorithm (matrix-vector multiplication over full graph vector).
    DensePowerIteration,
    /// Andersen-Chung-Lang Forward-Push local random walk algorithm.
    ForwardPush,
    /// Shadow mode: executes both algorithms, returns DensePowerIteration result, and logs discrepancies.
    ShadowMode,
}

/// Configuration parameters for Personalized PageRank (PPR).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PprConfig {
    /// Damping factor (probability of continuing random walk vs restarting). Default: 0.85.
    pub damping_factor: f32,
    /// Maximum power-iteration steps before terminating. Default: 100.
    pub max_iterations: u32,
    /// L1 norm threshold for early termination convergence check. Default: 1e-6.
    pub convergence_epsilon: f32,
    /// Algorithm strategy variant (Auto, DensePowerIteration, ForwardPush, ShadowMode). Default: Auto.
    #[serde(default)]
    pub algorithm: PprAlgorithm,
    /// Gibt eine nicht-konvergierte Warnung (tracing::warn!) aus, wenn
    /// max_iterations erreicht wird, bevor convergence_epsilon
    /// unterschritten wurde. Kein Fehler — die Berechnung liefert das
    /// beste bisher erreichte Ergebnis zurück. Default: true.
    #[serde(default = "default_warn_on_non_convergence")]
    pub warn_on_non_convergence: bool,
}

fn default_warn_on_non_convergence() -> bool {
    true
}

impl Default for PprConfig {
    fn default() -> Self {
        Self {
            damping_factor: 0.85,
            max_iterations: 100,
            convergence_epsilon: 1e-6,
            algorithm: PprAlgorithm::Auto,
            warn_on_non_convergence: true,
        }
    }
}

/// Konfigurations-Fingerabdruck für P8-Kalibrierungs-Integrität.
///
/// INVARIANTE INV-P8-1: Jede Änderung an einem der Felder MUSS
/// `IsotonicCalibrator::invalidate_on_config_change()` auslösen.
/// Kein Warmup-Fenster darf nach Fingerprint-Wechsel übersprungen werden.
///
/// BEGRÜNDUNG: arXiv:2608.01460 — Coverage-Kollaps unter Konfigurations-Drift.
/// Q4 und Q8 bei identischem model_id erzeugen unterschiedliche Score-Verteilungen.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConfigFingerprint {
    /// LLM-Modell-ID (z.B. "llama-3.2-3b-instruct")
    pub model_id: String,
    /// Quantisierungsgrad als String: "Q4_K_M", "Q8_0", "F16", "BF16".
    /// EXPLIZIT Teil des Fingerprints — Q4 ≠ Q8 bei gleichem model_id.
    pub quantization: String,
    /// SHA256 des Prompt-Templates (nicht der Inhalt — nur der Hash).
    /// Verhindert stille Kalibrierungs-Invalidierung bei Template-Drift.
    pub prompt_template_hash: [u8; 32],
    /// Temperatur als Bits für bit-exakten Vergleich (kein float-Gleichheitstest).
    /// `temperature_bits = temperature.to_bits()`
    pub temperature_bits: u32,
    /// Schwellenwert/Threshold als Bits für bit-exakten Vergleich.
    /// `threshold_bits = threshold.to_bits()`
    #[serde(default)]
    pub threshold_bits: u32,
}

impl ConfigFingerprint {
    /// Erstellt einen neuen `ConfigFingerprint`.
    pub fn new(
        model_id: impl Into<String>,
        quantization: impl Into<String>,
        prompt_template: &str,
        temperature: f32,
    ) -> Self {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(prompt_template.as_bytes());
        let hash: [u8; 32] = hasher.finalize().into();

        Self {
            model_id: model_id.into(),
            quantization: quantization.into(),
            prompt_template_hash: hash,
            temperature_bits: temperature.to_bits(),
            threshold_bits: 0,
        }
    }

    /// Setzt die Schwellenwert-Komponente (Threshold) für diesen Fingerabdruck (Builder Pattern).
    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.threshold_bits = threshold.to_bits();
        self
    }

    /// Extrahiert Schwellenwert/Threshold als f32 (verlustfrei da via to_bits gespeichert).
    #[inline]
    pub fn threshold(&self) -> f32 {
        f32::from_bits(self.threshold_bits)
    }

    /// Extrahiert Temperatur als f32 (verlustfrei da via to_bits gespeichert).
    #[inline]
    pub fn temperature(&self) -> f32 {
        f32::from_bits(self.temperature_bits)
    }
}

impl Default for ConfigFingerprint {
    fn default() -> Self {
        Self::new("default", "F16", "", 0.0)
    }
}

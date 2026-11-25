---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "21b"
---
### 21.3 FC-TS — Flow-Corrected Thompson Sampling (`contextra-adapt`) **[Phase 2]**

**Ziel:** Ersetzt nicht den Lyapunov-Drift-Wächter, sondern erweitert die bestehende
Drift-Bandit-Kopplung (`LyapunovDriftWatcher` + `DriftPolicyBridge`, bestätigt A4.2) um eine **richtungsbewusste**
Korrektur: statt pauschal die gesamte Präzisionsmatrix zu diskontieren (`drift_gamma = 0.95` bei erkannter
Drift, siehe `drift.rs`), werden historische Beobachtungen über einen expliziten Transport-Operator in die
Gegenwart korrigiert und selektiv gewichtet.

**Mathematische Spezifikation (unverändert gegenüber dem Bericht):** Bayesianisches Modell je Arm $a$:
$w_t^{(a)} \mid \mathcal{H}_t \sim \mathcal{N}(\mu_t^{(a)}, (\Lambda_t^{(a)})^{-1})$. Transport einer
vergangenen Belohnung: $\hat r_{s \to t} = r_s + (t-s)\langle \hat\delta_t^{(a)}, x_s \rangle$. Der
Drift-Vektor $\hat\delta_t^{(a)}$ wird periodisch per Online-Ridge-Regression über ein lokales Fenster
vergangener Beobachtungen geschätzt. Konfidenz-gewichtetes Präzisionsmatrix-Update:
$\Lambda_t^{(a)} = \lambda I_d + \sigma^{-2}\sum_{s<t}\omega_{s,t}x_s x_s^\top$, inkrementell in $O(d^2)$ ohne
Allokation über eine modifizierte Sherman-Morrison-Formel. Informationsvektor
$\eta_t^{(a)} = \lambda m_t^{(a)} + \sigma^{-2}\sum_{s<t}\omega_{s,t}\hat r_{s\to t}x_s$, Posterior-Mean
$\mu_t^{(a)} = (\Lambda_t^{(a)})^{-1}\eta_t^{(a)}$.

**Rust-Schnittstelle (normativ, korrigiert: als neue Variante von `BanditImplementation` statt als isolierter
neuer Typ, damit die bestehende Off-Policy-Evaluation aus §8.5 unverändert weiterverwendet werden kann):**

```rust
use contextra_types::ErrorClass;
use contextra_ports::Rng; // injizierter Determinismus, P28

/// Erweiterung des bestehenden `BanditImplementation`-Enums (`bandit.rs`) um eine dritte Variante,
/// NEBEN `ShermanMorrison` und `DiagonalApproximation` (bandit.rs:605) — kein Ersatz.
pub enum BanditImplementation {
    ShermanMorrison,
    DiagonalApproximation,
    FlowCorrectedThompson, // NEU (Fassung 4)
}

/// Feste Kapazität für das Rolling-Window der Drift-Schätzung — kein `Vec::push` mit Reallokation,
/// um §4(1) auch für den Ring-3-Hintergrund-Task einzuhalten (dort gilt Zero-Panic ebenso, nur nicht
/// die Latenzgarantie von Ring 0).
pub struct FlowCorrectedThompsonBandit {
    dim: usize,
    inv_a: Vec<f32>,           // A^-1, row-major, analog zu ShermanMorrisonBandit
    eta: Vec<f32>,
    mu: Vec<f32>,
    drift_rate: Vec<f32>,      // Transport-Vektor \hat{\delta}, NUR aus Ring 3 geschrieben (AK-18)
    drift_window: Box<[(Vec<f32>, f32, u64)]>, // fixe Kapazität, z. B. 200 Einträge (Kontext, Reward, Zeit)
    drift_window_len: usize,
    time_step: u64,
}

impl FlowCorrectedThompsonBandit {
    /// O(d²)-Update der Präzisionsmatrix mit konfidenz-gewichtetem Transport.
    /// Ring-0-Hot-Path: liest `drift_rate` nur (read-only), schreibt es NIE (AK-18).
    pub fn update_with_flow(
        &mut self,
        context: &[f32],
        reward: f32,
        observation_time: u64,
        confidence_weight: f32,
    ) -> Result<(), ErrorClass> {
        if context.len() != self.dim {
            return Err(ErrorClass::DimensionMismatch { expected: self.dim, actual: context.len() });
        }
        let delta_t = (self.time_step.saturating_sub(observation_time)) as f32;
        let drift_correction: f32 = context.iter().zip(&self.drift_rate).map(|(x, d)| x * d).sum();
        let transported_reward = reward + (delta_t * drift_correction);
        // Sherman-Morrison-Update für A^-1 mit confidence_weight, analog ShermanMorrisonBandit::discount_once
        unimplemented!()
    }

    /// Zieht ein Parameter-Sample zur Exploration; nutzt zwingend den injizierten `Rng`-Port (P28).
    pub fn select_arm(&self, context: &[f32], rng: &mut dyn Rng) -> Result<u32, ErrorClass> {
        unimplemented!()
    }

    /// NUR aus dem Ring-3-Hintergrund-Task aufrufbar (AK-18 prüft das als Architektur-Lint, nicht als
    /// Laufzeit-Panic — ein Aufruf aus Ring 0 wäre ein P26/P24-Verstoß, kein Safety-Verstoß).
    /// Aktualisiert `drift_rate` per Online-Ridge-Regression über `drift_window`.
    pub fn recompute_drift_rate_from_window(&mut self) {
        unimplemented!()
    }
}
```

**Invarianten-Nachweis:** Zero-Panic (§4(1)) und P24 (§4(6)) über die feste $O(d^2)$-Operationszahl je
Sherman-Morrison-Schritt; keine $O(d^3)$-Neuinvertierung. Harte Dimensionsprüfung mit `Result` (nicht
`debug_assert!`, konsistent mit der bereits in §17 Punkt 0.2 geforderten Korrektur für den bestehenden
Bandit). Determinismus (§4(3), P28) über injizierten `Rng`-Port — identische Seeds erzeugen bitgleiche
Routing-Entscheidungen.

**Restrisiken:** Das gleitende Fenster (z. B. 200 Beobachtungen je Arm) kostet zusätzlichen Speicher
gegenüber dem bestehenden Lyapunov-Watcher; `drift_window` MUSS als Ringpuffer fester Kapazität implementiert
werden (kein dynamisches Wachstum), sowohl aus Zero-Panic-Gründen als auch weil variable Kapazität die
Speicherbudget-Transparenz (§4(5)) unterläuft. Die Rekalkulation von $\hat\delta_t^{(a)}$ darf ausschließlich
im asynchronen Ring-3-Hintergrund-Task laufen (AK-18) — der Ring-0-Hot-Path wendet `drift_rate` nur lesend an.

---

### 21.4 LeanRAG Semantic Aggregation — dritte Pipeline-Stufe (`contextra-cognition`) **[Phase 2, hart gegatet auf H5]**

**Ziel:** Ergänzt die bereits bestehende, zweistufige Konsolidierungs-Pipeline
(`memory_consolidation.rs` — deterministischer, LLM-freier Structural Pass; `synthesis_phase.rs` — Generative
Synthesis Pass mit LLM-Kostenschutz) um eine **dritte** Stufe: Semantic Aggregation über Gaussian-Mixture-Clustering
mit Self-Supervised Type-Denoising, wie in LeanRAG (2508.10391, Tier A) und Type-Info-Denoising (2503.09916,
Tier A, AISTATS 2025 peer-reviewed) beschrieben. **Dies ist laut Teil A4.5 das am höchsten priorisierte der
vier Verfahren** — sowohl wegen der Beleglage als auch weil der größte Teil der benötigten Infrastruktur
bereits vorhanden ist (A4.2.1).

**Vorbedingung 0 (blockierend, AK-19, A4.4.1):** `child_edge_ids` im FlatBuffers-Schema (§12) und die
rekursive Erweiterung von `cascade_invalidate_hyperedges_for_superseded_doc` (§6.6 H5) MÜSSEN gemerged und
CI-grün sein, **bevor** irgendein Teil dieses Abschnitts produktiv (auch nicht hinter einem reinen
Opt-in-Feature-Flag mit realen Nutzerdaten) aktiviert wird. Ohne diese Erweiterung kann ein
Art.-17-DSGVO-Löschauftrag (§10) einen bereits konsolidierten Super-Knoten nicht erreichen.

**Vorbedingung 1 (Namensraum-Bereinigung, A4.2/A4.6 Punkt 3):** `contextra-cognition` enthält bereits
`memory_consolidation::ConsolidationConfig` (Zeile 28), `memory_consolidation::ConsolidationPhaseResult`,
`memory_consolidation::SynthesisConfig`, sowie je ein `SynthesisPhaseResult` in `memory_consolidation.rs`
**und** in `synthesis_phase.rs`. Die neue dritte Stufe **darf keinen dritten, kollidierenden
`ConsolidationConfig`-Typ einführen** (wie es der ursprüngliche Berichtsentwurf täte) — stattdessen werden die
bestehenden Typen um die neuen Felder erweitert bzw. ein klar unterscheidbar benannter dritter Typ nach
demselben Muster (`AggregationConfig`/`AggregationPhaseResult`) eingeführt, und die beiden gleichnamigen
`SynthesisPhaseResult`-Definitionen werden vor Beginn dieser Arbeit vereinheitlicht (ein Typ, re-exportiert).

**Mathematische Spezifikation (unverändert gegenüber dem Bericht, als kontrollierter Map-Reduce-Job in Ring 3):**

1. **Denoising:** Kanten einer Kohorte werden gegen ein Typen-Kompatibilitäts-Scoring evaluiert; logisch
   inkonsistente Kanten erzeugen einen asynchronen WAL-Tombstone-Eintrag.
2. **Gaussian Mixture Clustering:** hochdimensionale Entitäts-Embeddings der validen Knoten werden in $m$
   disjunkte Cluster $C_j$ unterteilt; EM-Konvergenz wird über einen festen RNG-Seed (`gmm_deterministic_seed`)
   deterministisch forciert.
3. **Abstraktion & Synthese:** je Cluster $C_j$ generiert der LLM-Synthetisierer (Port `TextGenerator`) einen
   abstrakten Meta-Knoten $\alpha_j$ — wiederverwendet denselben `max_llm_calls_per_cycle`-Kostenschutz
   (P12) und `CommunityStabilityTracker`/`compute_community_hash`-Mechanismus wie der bestehende Generative
   Synthesis Pass, statt einen eigenen Kostenschutz-Mechanismus neu zu bauen.
4. **Super-Hyperkante:** übersteigt die Konnektivität $\lambda_{j,k}$ zwischen Clustern $j,k$ einen
   Schwellenwert $\tau$, wird eine Super-Hyperkante mit `child_edge_ids` gesetzt auf die subsumierten
   Original-/Sub-Hyperkanten-IDs eingefügt.
5. **Atomic Replace:** deterministischer Transaktions-Commit über dasselbe Manifest-Batch-Fsync-Muster wie
   die bestehende Kompaktierung (§17 Punkt 2.4).

**Rust-Schnittstelle (normativ, korrigiert: erweitert bestehende Typen statt neue zu duplizieren, ruft die
bereits vorhandene Budget-Primitive auf statt sie neu zu spezifizieren):**

```rust
use contextra_types::{TxId, DocId, ErrorClass};
use contextra_ports::{Embedder, TextGenerator};
use contextra_engine::Collection;
// Wiederverwendung, nicht Neuerfindung:
use crate::memory_consolidation::{ConsolidationConfig, ConsolidationPhaseResult};

/// Konfiguration der dritten Pipeline-Stufe. Eigener Typ (nicht `ConsolidationConfig`, um die
/// Namenskollision aus A4.2 nicht zu wiederholen), aber im selben Namensmuster wie
/// `SynthesisConfig` für die zweite Stufe.
pub struct AggregationConfig {
    pub max_compaction_peak_memory_mb: usize,
    pub clustering_tau_threshold: f32,   // Schwellenwert lambda_{j,k} für Super-Kanten
    pub gmm_deterministic_seed: u64,
    pub max_llm_calls_per_cycle: usize,  // wiederverwendet dasselbe P12-Kostenschutz-Feld wie SynthesisConfig
}

/// Transparenter Report; folgt demselben Namensmuster wie `ConsolidationPhaseResult`/`SynthesisPhaseResult`.
pub struct AggregationPhaseResult {
    pub raw_edges_tombstoned: usize,
    pub abstract_hyperedges_created: usize,
    pub peak_memory_used_mb: usize,
    pub child_edge_ids_written: usize,   // NEU: Nachweis, dass H5-Traversierbarkeit hergestellt wurde
}

/// Gesamtergebnis der dreistufigen Pipeline (Struktur → Synthese → Aggregation), damit Aufrufer
/// nicht drei separate Report-Typen manuell zusammenführen müssen.
pub struct ConsolidationPipelineResult {
    pub structural: ConsolidationPhaseResult,
    pub synthesis: crate::synthesis_phase::SynthesisPhaseResult, // vereinheitlichter Typ, Vorbedingung 1
    pub aggregation: Option<AggregationPhaseResult>,             // None, falls Stufe 3 deaktiviert/nicht erreicht
}

impl Collection {
    /// Führt die semantische Abstraktion (dritte Pipeline-Stufe) als asynchronen Ring-3-Task aus.
    /// Darf den Ring-0-Lese-Pfad nicht blockieren (P26).
    pub async fn consolidate_semantic_hyperedges(
        &self,
        embedder: &dyn Embedder,
        llm: &dyn TextGenerator,
        config: &AggregationConfig,
    ) -> Result<AggregationPhaseResult, ErrorClass> {
        // 1. Budget-Vorprüfung — RUFT DIE BEREITS VORHANDENE RING-0-PRIMITIVE AUF (A4.2.1),
        //    baut sie NICHT neu:
        let estimated_peak = self.graph.estimate_compaction_peak_bytes() / (1024 * 1024);
        if estimated_peak > config.max_compaction_peak_memory_mb {
            return Err(ErrorClass::CompactionBudgetExceeded {
                budget_mb: config.max_compaction_peak_memory_mb,
                estimated_mb: estimated_peak,
            });
        }
        // 2. Self-Supervised Type-Denoising + deterministisches GMM-Clustering
        // 3. Batched TextGenerator-Aufruf zur Synthese der Alpha-Knoten (max_llm_calls_per_cycle,
        //    CommunityStabilityTracker wiederverwendet)
        // 4. Super-Hyperkanten mit `child_edge_ids` schreiben (NUR wenn Vorbedingung 0 erfüllt —
        //    Aufrufer MUSS vorab per CI-Gate sicherstellen, dass das Schema die Erweiterung trägt)
        // 5. Atomarer Transaktions-Commit über bestehendes Manifest-Batch-Fsync-Muster
        unimplemented!()
    }
}
```

**Invarianten-Nachweis:** Speicherbudget-Transparenz (§4(5)) ist durch Wiederverwendung der bereits
getesteten `estimate_compaction_peak_bytes()`-Primitive (A4.2.1) **stärker** abgesichert als im
ursprünglichen Berichtsentwurf, der diesen Aufruf als neu zu bauenden Baustein annahm. Determinismus (§4(3))
über festen `gmm_deterministic_seed` für die GMM-Initialisierung; die LLM-Synthese selbst bleibt inhärent
stochastisch, das nachfolgende Topologie-Update läuft jedoch WAL-first und deterministisch ein. P26
(Sync-Kern) bleibt gewahrt, da der Task als `async`-Operation in `contextra-cognition` (Ring 3) läuft,
während die aufgerufenen `contextra-graph`-Operationen (Ring 0) synchron und blockierungsfrei bleiben.

**Restrisiken (verschärft gegenüber dem Bericht, siehe A4.4.1):** Ohne Vorbedingung 0 (`child_edge_ids` +
H5-Rekursion) ist dieser Abschnitt **nicht produktionsreif**, unabhängig davon, wie vollständig die
Aggregations-Logik selbst implementiert ist — ein Löschauftrag für ein Quelldokument, dessen Fakten in einen
$\alpha_j$-Knoten abstrahiert wurden, würde sonst silently fehlschlagen und damit eine Datenschutzzusage
(§10) brechen. Diese Reihenfolge ist nicht verhandelbar und MUSS im CI-Gate erzwungen werden (AK-19).

---

*Ende §21. Zusammen mit §0–§20 und Teil A/A2/A3/A4 bildet dieser Abschnitt die vollständige, in sich
geschlossene Gesamtspezifikation der Fassung 4. Änderungsprotokoll und Prüfnachweise dieser Fassung: Anhang D.*

---

<a id="anhang-b"></a>

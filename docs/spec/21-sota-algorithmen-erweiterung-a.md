---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "21a"
---
## 21. Normative SOTA-Algorithmen-Erweiterung: TL-HFD, DiBud, FC-TS, LeanRAG

> **Rang und Herkunft:** Dieser Abschnitt ist normativ auf derselben Ebene wie §5–§10; er unterliegt jedoch
> Teil A4, soweit es um Priorität und Aktivierungsreihenfolge geht (Leitentscheidung (6), Kopf des Dokuments).
> Er führt die vier in Teil A4 geprüften SOTA-Verfahren aus dem Deep-Research-Bericht (§5.1–§5.4 dort) in die
> normative Form dieser Spezifikation über — **korrigiert** um die in Teil A4 dokumentierten Abweichungen
> gegenüber dem tatsächlichen Code (falsche Kanal-Enums, materialisierte statt Stream-Signaturen,
> Namenskollisionen, bereits vorhandene statt fehlende Bausteine). Wo ein Codeblock unten von einem Codeblock
> im Deep-Research-Bericht abweicht, **gilt dieser Abschnitt**, nicht der Bericht. Jeder Unterabschnitt trägt
> eine Phasen-Kennzeichnung nach §A.3; alle vier sind `[Phase 2]` (Hot-Path/Struktur-Stufe), da sie Stufe 1½
> der Gesamtroadmap (§18) zugeordnet sind — keines darf vor erfolgreichem Gate 0/1 für den jeweiligen Crate
> begonnen werden.

### 21.1 TL-HFD — Thresholded Local Hyper-Flow Diffusion (`contextra-graph`) **[Phase 2]**

**Ziel:** Ersetzt den heuristischen, ε-schwellenwertbasierten `forward_push_ppr` (Andersen-Chung-Lang) für
n-äre Hyperkanten durch einen formal exakten, projizierten Subgradientenabstieg auf der Lovász-Erweiterung
der Hyperkanten-Schnittkosten — mit einer durch Top-$k$-Randaktivierung **hart** (nicht nur statistisch)
begrenzten Lokalität, was P24 (§4(6)) strenger erfüllt als das bestehende ε-Konvergenzkriterium.

**Mathematische Spezifikation (unverändert gegenüber dem Bericht, hier normativ übernommen):**

Minimierung der stetigen konvexen Relaxation des Hyper-Flow-Diffusion-Dual-Objektivs über $x \in \mathbb{R}^{|V|}_+$:

$$\min_{x} F(x) := \frac{1}{2} \sum_{e \in E} \theta_e f_e(x)^2 + \frac{\sigma}{2} x^\top D x - \langle \Delta - d, x \rangle$$

mit Knotengrad $d_v = \sum_{e \ni v} \theta_e$, $D = \mathrm{diag}(d)$, Seed-Injektionsvektor $\Delta(v) = \delta d_v$
($\delta \ge 2$) für $v \in S$ und $0$ sonst, sowie der Lovász-Erweiterung $f_e(x) := \max_{\rho \in B_e} \langle \rho, x \rangle$
für Hyperkante $e$. Der Algorithmus hält eine aktive Region $A(t) = \mathrm{supp}(x^{(t)}) \cup S$ und eine
Ein-Hop-Grenze $\partial A(t)$; der Subgradient wird ausschließlich auf $A(t) \cup \partial A(t)$ ausgewertet
(kein $O(|V|)$-Schritt). Drei Phasen pro Iteration: (1) Update aktiver Knoten
$x^{(t+1)}_u \leftarrow \max\{0, x^{(t)}_u - \eta_{t+1} [g^{(t)}]_u / d_u\}$ mit $\eta_{t+1} = 1/\sigma(t+1)$;
(2) Boundary-Scoring $s^{(t)}(u) = \kappa^{(t)}(u) \cdot c^{(t)}(u)$ mit Motif-Conductance-Gewichtung
$c^{(t)}(u) = (d_\mathrm{in}^{(t)}(u)/d_u)^\gamma$; (3) Thresholded Top-$k$-Aktivierung der Grenzknoten mit
höchstem Score. Laufzeit pro Iteration: $O\!\left(\sum_{e: e \cap (A(t) \cup \partial A(t)) \neq \emptyset} |e|\right)$
— unabhängig von der Graph-Makrostruktur.

**Rust-Schnittstelle (normativ, korrigiert gegenüber dem Bericht: integriert in den bestehenden `PprAlgorithm`-Enum
statt als isolierte neue Methode):**

```rust
use crate::csr::{CsrGraph, GraphInner};
use crate::error::GraphError;
use contextra_types::{EntityId, HyperEdgeId};
use ahash::AHashMap;

/// Konfiguration für Thresholded Local Hyper-Flow Diffusion (TL-HFD).
/// Erzwingt deterministische Schranken gemäß P24 (§4(6)).
#[derive(Debug, Clone)]
pub struct TlHfdParams {
    pub sigma: f32,                 // Regularisierungsstärke
    pub delta: f32,                 // Seed-Masse-Injektionsrate, MUSS >= 2.0 sein (Result-Validierung, nicht debug_assert! — vgl. §17 Punkt 0.2)
    pub gamma: f32,                 // struktureller Gewichtungsfaktor (Motif Conductance)
    pub max_iterations: u32,        // harte Obergrenze
    pub max_top_k_expansion: usize, // Hard-Cap für Randaktivierung pro Schritt (P24)
    pub max_hyperedge_sort_size: usize, // NEU (Fassung 4, siehe Restrisiken unten): deterministischer Cutoff
                                         // für die Lovász-Subgradienten-Sortierung; Kanten mit |e| über diesem
                                         // Wert nutzen eine deterministische Top-Gewicht-Kürzung statt
                                         // Stochastik — kein RNG-Port nötig, kein P28-Bezug.
}

/// Zustand der aktiven Traversierung. Präallozierte Kapazität garantiert Zero-Panic (§4(1)) bei OOM.
pub struct ActiveRegion {
    pub x_values: AHashMap<EntityId, f32>,
    pub boundary_scores: Vec<(EntityId, f32)>,
}

/// Erweiterung des bestehenden `PprAlgorithm`-Enums aus `ppr.rs` (nicht neu einführen — anhängen).
/// `ShadowMode` vergleicht künftig wahlweise Forward-Push↔DensePowerIteration ODER
/// Forward-Push↔TlHfd, gesteuert über ein zweites Feld, um die bestehende Diskrepanz-Logging-Mechanik
/// unverändert weiterzunutzen (AK-16).
pub enum PprAlgorithm {
    ForwardPush,
    DensePowerIteration,
    TlHfd(TlHfdParams),             // NEU (Fassung 4)
    ShadowMode { compare_against: Box<PprAlgorithm> }, // Feld ergänzt, Rückwärtskompatibilität: Default bleibt DensePowerIteration
}

impl CsrGraph {
    /// Lokaler, subgradienten-basierter Submodularitäts-Clustering-Lauf.
    /// Läuft zunächst NUR über `PprAlgorithm::ShadowMode { compare_against: TlHfd(..) }` (AK-16);
    /// wird erst nach Auswertung der Diskrepanz-Logs (Teil A4.7) zum Default.
    pub fn thresholded_local_hfd(
        &self,
        seeds: &[EntityId],
        params: &TlHfdParams,
    ) -> Result<AHashMap<EntityId, f32>, GraphError> {
        let snapshot = self.inner.load();
        let max_capacity = seeds.len().saturating_add(
            (params.max_iterations as usize).saturating_mul(params.max_top_k_expansion)
        );
        let mut active = ActiveRegion {
            x_values: AHashMap::with_capacity(max_capacity),
            boundary_scores: Vec::with_capacity(params.max_top_k_expansion.saturating_mul(4)),
        };
        // Implementierung gemäß den drei Phasen oben; jeder Schritt prüft `f32::is_finite`
        // auf NaN-Kontamination vor der nächsten Iteration (§4(1)).
        unimplemented!()
    }

    /// Lovász-Erweiterung für eine Hyperkante, zero-copy über `HyperEdgeView`.
    /// Für `|e| > max_hyperedge_sort_size`: deterministische Top-Gewicht-Kürzung statt Vollsortierung
    /// (siehe Restrisiken) — KEIN Pseudo-RNG, um P28-Injektionspflicht zu vermeiden.
    #[inline(always)]
    fn compute_lovasz_extension<'a>(
        &self,
        edge_view: &'a crate::hyperedge::HyperEdgeView,
        x_values: &AHashMap<EntityId, f32>,
        max_sort_size: usize,
    ) -> f32 {
        unimplemented!()
    }
}
```

**Invarianten-Nachweis:** P24 wird strenger als durch das bestehende ε-Konvergenzkriterium erfüllt, da
`TOPK(s, k)` die Netzwerkexpansion pro Iteration exakt auf $k$ neue Knoten begrenzt; Gesamtlaufzeit
$O(\text{Iterations} \times k \times |\text{Seed}|)$, unabhängig von der Gesamtgraphgröße. Zero-Panic (§4(1))
über Präallokation nach bekanntem theoretischem Maximum sowie `f32::is_finite`-Prüfung. Determinismus (§4(3))
über einen strikten Tie-Breaker (totale `EntityId`-Ordnung bei Score-Gleichstand) — identisch zu dem bereits
in §6.6 H2/H6 geforderten Muster, kein neuer Mechanismus.

**Restrisiken (korrigiert gegenüber dem Bericht, siehe A4.5/S.2):** Die Lovász-Subgradienten-Berechnung
erfordert eine Sortierung der Komponentenwerte $x_u$ je aktiver Hyperkante ($O(|e| \log |e|)$). Der Bericht
schlägt für sehr dichte Kanten ($|e| > 1000$) eine **stochastische** Stichprobe vor — das würde einen
dedizierten `contextra_ports::Rng`-Port (P28) in einen bislang RNG-freien Algorithmus einführen und damit
Determinismus-Prüfpfad und Test-Oberfläche unnötig vergrößern. **Korrektur dieser Fassung:** stattdessen
`max_hyperedge_sort_size` als harten, deterministischen Cutoff verwenden (Top-Gewicht-Kürzung statt
Zufallsstichprobe) — liefert schwächere, aber reproduzierbare Approximationsgarantien ohne neue P28-Fläche.
Migrationspfad: `ShadowMode` (AK-16) → Log-Auswertung (Teil A4.7) → Default-Flip als eigene ⚖️-Entscheidung
analog K-16/K-17 (§16.1).

---

### 21.2 DiBud — Direct Budgeting für deterministische RRF-Präfixe (`contextra-rank`) **[Phase 2, zweistufig gegatet]**

**Ziel:** Ersetzt den blockierenden Vollabruf fester Top-$k$-Kandidatenmengen durch einen inkrementellen,
budget-gesteuerten Fusionsprozess mit beweisbar exaktem RRF-Präfix. **Bedingung laut Teil A4.4.2/A4.5:** Kein
Merge dieses Abschnitts, bevor die in Schritt 1 geforderte Streaming-Iterator-Vorarbeit abgeschlossen ist
(AK-17).

**Korrigiertes Kanalmodell (Fassung 4, siehe A4.2.2):** Das tatsächliche `SignalKind`-Enum
(`crates/contextra-rank/src/fusion.rs:316`) ist `{Vector, Text, Graph, EdgeReinforcement}` — **kein**
generischer Metadaten-/Filter-Kanal. `EdgeReinforcement` ist ein aus dem Bandit-Subsystem (§8)
rückgekoppeltes Signal ohne eigenen paginierbaren Index und wird **außerhalb** des DiBud-Budgets als stets
vollständig ausgewerteter additiver Term behandelt. Das Budget gilt ausschließlich für die drei echten
Retrieval-Kanäle Vector, Text, Graph.

**Mathematische Spezifikation (Kanalzahl auf 3 korrigiert):** Fusionsscore
$F(x) = \sum_{i \in \{V,T,G\}} g(r_i(x)) + w_{\mathrm{er}} \cdot \mathrm{er}(x)$, mit Dämpfung $g(r) = 1/(c+r)$
($c = 60$) und dem stets vollständig ausgewerteten `EdgeReinforcement`-Term $\mathrm{er}(x)$. Für jeden der
drei budgetierten Kanäle wird ein Lesetiefe-Zeiger $d_i$ geführt, $\sum_{i \in \{V,T,G\}} d_i \le B$. Für ein
beobachtetes, noch nicht zertifiziertes Dokument $x$: obere Schranke
$F^+(x) = \sum_{i \in \mathrm{obs}(x)} g(r_i(x)) + \sum_{j \notin \mathrm{obs}(x)} g(d_j+1) + w_{\mathrm{er}} \cdot \mathrm{er}(x)$,
untere Schranke $F^-(x) = \sum_{i \in \mathrm{obs}(x)} g(r_i(x)) + w_{\mathrm{er}} \cdot \mathrm{er}(x)$. Ein
Dokument ist zertifiziert, sobald $F^-(x)$ über $F^+(y)$ aller noch unzertifizierten $y$ liegt.

**Rust-Schnittstelle (normativ, korrigiert: 3-Kanal-Budget + Provenance-Anbindung, siehe unten):**

```rust
use contextra_types::{DocId, ErrorClass};

/// Nur drei budgetierte Retrieval-Kanäle (Fassung 4, A4.2.2) — EdgeReinforcement bewusst NICHT hier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetedChannel { Vector, Text, Graph }

#[derive(Debug, Clone)]
pub struct FusionBudget {
    pub max_total_accesses: usize,   // gilt nur für {Vector, Text, Graph}
    pub min_certified_results: usize,
    pub edge_reinforcement_weight: f32, // additiver Term, außerhalb des Budgets ausgewertet
}

/// Verwaltet Schranken und Zustand des inkrementellen RRF-Lösers.
/// Trägt — anders als im ursprünglichen Berichtsentwurf — von Anfang an eine Anbindung an die
/// bestehende `ProvenanceRecord`/`SignalContribution`-Infrastruktur (`fusion.rs`), damit die
/// Herkunfts-Nachvollziehbarkeit pro Score-Beitrag nicht nachträglich verheiratet werden muss
/// (der Bericht selbst benennt diese Lücke im eigenen Entwurf, Teil C §"Einordnung, die im Bericht fehlt").
pub struct DiBudFusionState {
    pub current_accesses: usize,
    partial_scores: ahash::AHashMap<DocId, PartialScoreBounds>,
    channel_depths: [usize; 3],                          // NUR die drei budgetierten Kanäle
    provenance: ahash::AHashMap<DocId, crate::fusion::ProvenanceRecord>, // wiederverwendet, nicht neu erfunden
}

#[derive(Debug, Clone, Default)]
struct PartialScoreBounds {
    pub known_score: f32,
    pub unobserved_channels_mask: u8, // Bitmaske über die 3 budgetierten Kanäle (nicht 4)
}

impl DiBudFusionState {
    pub fn new(capacity: usize) -> Self {
        Self {
            current_accesses: 0,
            partial_scores: ahash::AHashMap::with_capacity(capacity),
            channel_depths: [0; 3],
            provenance: ahash::AHashMap::with_capacity(capacity),
        }
    }

    /// Nimmt Iterator-Streams der DREI budgetierten Signalquellen entgegen; `edge_reinforcement`
    /// wird als vollständig materialisierte Lookup-Funktion übergeben (kein Iterator — es gibt
    /// strukturell keine "unerreichten" EdgeReinforcement-Kandidaten).
    ///
    /// VORAUSSETZUNG (AK-17, hart geprüft): `vector_stream`/`text_stream`/`graph_stream` MÜSSEN echte,
    /// on-demand nachliefernde Iteratoren sein (aus `contextra-vector`/`contextra-text`/`contextra-graph`),
    /// keine `.iter()`-Adapter über bereits vollständig materialisierte `Vec<DocId>` — sonst entfällt der
    /// P24-Gewinn vollständig (Teil A4.4, DiBud-Zeile).
    pub fn fuse_exact_prefix<I1, I2, I3>(
        &mut self,
        vector_stream: &mut I1,
        text_stream: &mut I2,
        graph_stream: &mut I3,
        edge_reinforcement: impl Fn(DocId) -> f32,
        budget: &FusionBudget,
    ) -> Result<Vec<DocId>, ErrorClass>
    where
        I1: Iterator<Item = DocId>,
        I2: Iterator<Item = DocId>,
        I3: Iterator<Item = DocId>,
    {
        // Polling-Schleife evaluiert F^-(x) > F^+(y) kontinuierlich; bricht garantiert bei
        // `current_accesses >= max_total_accesses` ab (P24). Kanal-Priorität bei gleicher oberer
        // Schranke: 1. Graph, 2. Text, 3. Vector (EdgeReinforcement ist kein Streaming-Kanal und
        // entfällt aus dem Tie-Breaker), in letzter Instanz totale `DocId`-Ordnung.
        unimplemented!()
    }
}
```

**Schritt 1 (Vorbedingung, AK-17):** Bevor `fuse_exact_prefix` implementiert wird, müssen
`contextra-vector` (DiskANN) und `contextra-text` (BM25/Block-Max-WAND) je einen
`impl Iterator<Item = DocId>` exponieren, der Kandidaten batchweise (Vorschlag: 16 pro Kanalzugriff, wie im
Bericht selbst zur Wahrung der Cache-Lokalität P25 vorgeschlagen) nachliefert, statt intern vollständige
Ergebnismengen zu berechnen. Dies ist unabhängig von DiBud selbst wertvoll (A4.4.2) und sollte als eigener
PR vor Schritt 2 gemerged werden.

**Invarianten-Nachweis:** P24 über die harte Abbruchbedingung `current_accesses >= max_total_accesses` in der
Polling-Schleife — deterministischer Stopp unabhängig von der Dichte der zugrundeliegenden Indexlisten. Zero-Panic
(§4(1)) über kapazitätsvorallozierte `AHashMap`s (Budget-Größe bekannt). Determinismus (§4(3)) über
präzise Fließkommaarithmetik plus hartkodierten, jetzt auf drei Kanäle korrigierten Tie-Breaker.

**Restrisiken (verschärft gegenüber dem Bericht, siehe A4.4.2):** Ohne Schritt 1 liefert eine Implementierung
gegen die heutige `fuse_signals`-Signatur **keinen** P24-Gewinn — das Budget würde nachträglich auf bereits
vollständig geladene Listen angewendet. Vor produktiver Default-Umstellung: eigene Kleinst-Implementierung +
Benchmark gegen den Paper-Aufbau (Teil C C.1) — Tier D, Einzelautor, sieben Tage alt, unrepliziert.

---

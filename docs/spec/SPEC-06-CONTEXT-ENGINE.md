# SPEC-06 — Context Engine und Retrieval

**Version 2.0 · November 2026 · Status: Final**
**Querverweise:** [Index](SPEC-00-INDEX.md) · [Datenmodell](SPEC-02-DATA-MODEL.md) · [Indizes](SPEC-05-INDEXES.md) · [API](SPEC-08-API.md)

---

## 1. Primitive Operationen

| Operation | Zweck | Latenz-Ziel |
|---|---|---|
| `recall(q, budget, policy)` | Beste Kontextmenge unter Token-Budget | p50 < 5 ms, p99 < 25 ms |
| `window(seq, anchor, budget)` | Zusammenhängendes Fenster | < 1 ms |
| `expand(handle, level)` | Feinere/gröbere Stufe nachladen | < 500 µs |
| `explain(recall_id)` | Warum wurde was gewählt | < 10 ms |
| `neighbors(handle, edge_types, fanout)` | Graph-Nachbarschaft | < 2 ms |
| `diff(a, b)` | Menschen-/LLM-lesbare Änderungszusammenfassung | < 50 ms |

---

## 2. Recall-Anfrage

### 2.1 API

```rust
pub struct Recall {
    pub query: Query,                   // Text, Embedding, oder beides
    pub namespaces: Vec<NsId>,          // Suchraum (leer = alle)
    pub budget: TokenBudget,            // Maximal-Budget + Tokenizer
    pub signals: Signals,               // Welche Retrieval-Pfade aktiv
    pub filters: Filters,               // Harte Filter (Trust, Kind, Zeit)
    pub diversity: DiversityConfig,     // Redundanzabzug
    pub valid_at: TemporalRef,          // Bitemporaler Sichtbarkeitszeitpunkt
    pub render: RenderFormat,           // Ausgabeformat
    pub order: OrderPolicy,             // Ergebnis-Sortierung
    pub explain: bool,                  // Explain-Daten mitliefern
}

pub struct TokenBudget {
    pub max_tokens: u32,
    pub tokenizer: TokenizerId,
    pub reserve_tokens: u32,            // Puffer für Formatierung (Default: 50)
}

pub struct Signals {
    pub vector: bool,                   // Default: true
    pub bm25: bool,                     // Default: true
    pub graph: bool,                    // Default: true (wenn Kanten vorhanden)
    pub recency: bool,                  // Default: true
    pub weights: SignalWeights,         // Gewichtung
    pub recency_half_life: Duration,    // Default: 30 Tage
}

pub struct Filters {
    pub min_trust: Option<TrustLabel>,
    pub kinds: Option<Vec<CellKind>>,
    pub time_range: Option<(Timestamp, Timestamp)>,
    pub tags: Option<Vec<String>>,
    pub attrs: Vec<AttrPredicate>,
}

pub enum DiversityConfig {
    None,
    Mmr { lambda: f32 },               // Default: 0.7 (0 = max Diversität, 1 = max Relevanz)
}

pub enum RenderFormat {
    Compact,                            // Handles + gewählte Zoom-Stufe (tokenoptimiert)
    Xml,                                // XML-Strukturiert (für System-Prompts)
    Json,                               // Maschinenlesbar
    Raw,                                // Nur CellIds + Metadaten, kein Text
}

pub enum OrderPolicy {
    Relevance,                          // Nach Score absteigend
    Chronological,                      // Zeitlich
    CacheFriendly,                      // Stabile Cells zuerst (für KV-Cache)
}
```

### 2.2 Beispiel

```rust
let ctx = branch
    .recall(
        Recall::new("Wie haben wir den Deployment-Fehler vom Oktober gelöst?")
            .namespaces(["agent/episodic", "agent/semantic"])
            .budget(Tokens(6_000), TokenizerId::CL100K)
            .signals(Signals::hybrid()
                .recency_half_life(days(30))
                .weights(SignalWeights { vector: 0.4, bm25: 0.3, graph: 0.2, recency: 0.1, trust: 1.0 }))
            .filters(Filters::new().min_trust(Trust::ToolVerified))
            .diversity(Mmr { lambda: 0.7 })
            .valid_at(Now)
            .render(Render::Compact)
            .order(Order::CacheFriendly),
    )
    .await?;
```

### 2.3 Recall-Antwort

```rust
pub struct RecallResult {
    pub recall_id: RecallId,            // Für explain() und Access-Trace
    pub items: Vec<RecallItem>,
    pub budget_used: u32,               // Verbrauchte Token
    pub budget_total: u32,              // Budget
    pub evidence_quality: f32,          // 0.0–1.0, Evidence-Sufficiency-Signal
    pub prefix_digest: [u8; 16],        // BLAKE3-128 der stabilen Präfix-Token
    pub truncated: bool,                // Mehr Treffer verfügbar
    pub explain: Option<ExplainSummary>,
}

pub struct RecallItem {
    pub handle: Handle,
    pub cell_id: CellId,
    pub zoom_level: u8,                 // Gewählte Stufe
    pub tok_count: u32,                 // Token dieser Stufe
    pub score: f32,                     // Fusionierter Score
    pub trust: TrustLabel,
    pub timestamp: Timestamp,
    pub text: String,                   // Gerenderter Text der gewählten Stufe
}
```

**Compact-Rendering-Beispiel:**

```
[c:k7qd] (L1, 142 tok, trust=verified, 2026-10-14)
Deployment schlug fehl wegen abgelaufenem Registry-Token; Lösung: …
[c:m2xa] (L0, 480 tok, trust=user, 2026-10-14)
<voller Text der Cell>
[c:p9hv] (L2, 18 tok) Rollback-Runbook v3 – Titel
… 11 weitere Handles verfügbar via expand(); Budget genutzt: 5 874 / 6 000
```

---

## 3. Recall-Pipeline

```
Query ──► Kandidaten ──► Score-Fusion ──► Zoom-Knapsack ──► Ordnung ──► Rendering
          (parallel)      (Merge)         (Optimierung)     (Sort)      (Format)
```

### 3.1 Stufe 1: Kandidatengenerierung (parallel)

Alle Retrieval-Pfade laufen **parallel** und liefern jeweils eine Score-Liste:

| Pfad | Quelle | Candidates | Komplexität |
|---|---|---|---|
| Vektor | emb.q (Hot Head) + Delta-Index | Top-200 (konfigurierbar) | O(log n · L_search) |
| BM25 | FTS-Segmente + Delta-Postings | Top-200 (Block-Max WAND) | O(query_terms · log n) |
| Graph | k-Hop-Expansion ab Query-Entities | bis fan_out_budget | O(fan_out × hops) |
| Recency | Temporal-Index | Top-100 (neueste) | O(log n) |

**Filter-Integration:** Harte Filter (Trust, Kind, Zeit) werden als Bitmaps **vor** der Kandidatengenerierung berechnet und in alle Pfade eingespeist.

### 3.2 Stufe 2: Score-Fusion

```rust
pub struct FusionConfig {
    pub method: FusionMethod,
    pub weights: SignalWeights,
    pub rrf_k: f32,                     // RRF-Parameter (Default: 60.0)
}

pub enum FusionMethod {
    /// Reciprocal Rank Fusion — robust, keine Kalibration nötig
    Rrf,
    /// Gewichtete Summe kalibrierter Scores
    Weighted,
    /// Gelernte Gewichte aus Access-Trace-Feedback
    Learned,
}
```

**RRF-Formel:**
```
score(d) = Σ_s  weight_s / (rrf_k + rank_s(d))
```

**Kalibration (für Weighted/Learned):**
- Isotonische Regression pro Retrieval-Pfad.
- Trainiert auf Expand-Events als implizites Relevanz-Feedback.
- Neutraining alle 10 000 Recalls oder bei Drift-Detection.

### 3.3 Trust- und Recency-Faktoren

Scores werden multiplikativ modifiziert:

```
final_score = fusion_score × trust_factor × recency_factor
```

| Trust-Label | trust_factor |
|---|---|
| System | 1.0 |
| Principal(User) | 1.0 |
| ToolVerified | 0.95 |
| ToolUnverified | 0.8 |
| ExternalUntrusted | 0.5 |

```
recency_factor = exp(-ln(2) / half_life_days × age_days)
```

### 3.4 Stufe 3: Zoom-Knapsack

**Problem:** Gegeben Kandidaten *i* mit Stufen *ℓ* (Kosten `c[i][ℓ]` in Token, Nutzen `u[i][ℓ]`), wähle genau eine Stufe pro Kandidat (oder „weglassen") so, dass Σ Kosten ≤ Budget und Gesamtnutzen maximal.

**Nutzen-Funktion:**
```
u[i][ℓ] = relevance_i × gain(ℓ) × trust_factor_i × recency_factor_i
```

| Stufe | gain(ℓ) |
|---|---|
| L0 (raw) | 1.0 |
| L1 (summary) | 0.7 |
| L2 (title) | 0.3 |
| L3 (handle) | 0.05 |

**Algorithmus:**

```rust
pub fn pack(candidates: &mut [Candidate], budget: Tokens, policy: &PackPolicy) -> PackPlan {
    // 1. Pro Kandidat: obere konvexe Hülle der (Kosten, Nutzen)-Punkte
    for c in candidates.iter_mut() {
        c.compute_convex_hull();
    }

    // 2. Start: alle Kandidaten auf gröbster Stufe (Handle/Titel)
    let mut plan = PackPlan::with_all_at_coarsest(candidates);

    // 3. Max-Heap über Upgrade-Schritte: Δ-Nutzen / Δ-Kosten
    let mut heap = BinaryHeap::new();
    for c in candidates {
        if let Some(upgrade) = c.next_upgrade() {
            heap.push(upgrade);
        }
    }

    // 4. Greedy: beste Upgrades bis Budget erschöpft
    while let Some(upgrade) = heap.pop() {
        if plan.remaining_budget() < upgrade.delta_cost {
            continue;
        }

        // MMR: Redundanzabzug wenn ähnliche Candidates bereits auf feinerer Stufe
        let adjusted_utility = upgrade.delta_utility
            * mmr_penalty(upgrade.candidate, &plan, policy.lambda);

        if adjusted_utility / upgrade.delta_cost as f32 > policy.min_efficiency {
            plan.apply_upgrade(upgrade);
            if let Some(next) = upgrade.candidate.next_upgrade() {
                heap.push(next);
            }
        }
    }

    plan
}
```

**Vorteil gegenüber Top-k:** Viele *grobe* Treffer + wenige *feine* statt wenige vollständige → Agent bekommt **Überblick und Tiefe** im selben Budget.

### 3.5 Stufe 4: Ordnung

**CacheFriendly-Modus:**
1. **Stabile Cells zuerst:** System-/Policy-/Skill-Cells mit unveränderlichen Hashes, sortiert nach ChunkId → identische Präfixe über Aufrufe hinweg.
2. **Semistabile Cells:** Cells, die in den letzten 10 Recalls ebenfalls enthalten waren.
3. **Volatile Cells:** Frische Episoden, query-spezifische Treffer.

**Prefix-Digest:** `BLAKE3-128(Token-IDs der stabilen Cells)` → Aufrufer kann für KV-Cache-Keys nutzen.

### 3.6 Stufe 5: Rendering

| Format | Token-Overhead | Verwendung |
|---|---|---|
| Compact | minimal (~5 Token/Cell Overhead) | Standard für Agenten |
| XML | ~15 Token/Cell | System-Prompts mit Struktur |
| JSON | ~20 Token/Cell | Maschine-zu-Maschine |
| Raw | 0 (kein Text) | Programmatischer Zugriff |

---

## 4. Progressive Disclosure

Standard-Pattern für Agenten:

1. `recall` liefert **kompakt mit Handles** (Budget-optimiert).
2. Agent liest Ergebnis, entscheidet welche Cells relevant sind.
3. `expand(handle, L0)` nur für wirklich relevante Cells.
4. Engine zählt Expand-Aufrufe als **Relevanzsignal**.

### 4.1 Expand-API

```rust
pub fn expand(
    handle: Handle,
    target_level: Option<u8>,       // None = L0 (Volltext)
    budget: Option<Tokens>,         // Optional: Budget für Expand-Ergebnis
) -> Result<ExpandResult>;

pub struct ExpandResult {
    pub handle: Handle,
    pub cell_id: CellId,
    pub level: u8,
    pub text: String,
    pub tok_count: u32,
    pub prov: Provenance,
    pub trust: TrustLabel,
    pub edges: Vec<EdgeSummary>,    // Kurzliste der Kanten
}
```

### 4.2 Expand als Relevanzsignal

Jeder Expand-Aufruf wird im Access-Trace gespeichert und beeinflusst:
- **Ranking:** Expand-Rate fließt in Salience-Score ein.
- **Layout:** Co-expanded Cells werden im Re-Layout gruppiert.
- **Zoom-Knapsack:** Candidates mit hoher historischer Expand-Rate werden bevorzugt auf feinerer Stufe angeboten.

---

## 5. Explainability

```rust
pub struct ExplainResult {
    pub recall_id: RecallId,
    pub candidates: Vec<ExplainCandidate>,
    pub rejected: Vec<RejectionReason>,
    pub pipeline_stats: PipelineStats,
}

pub struct ExplainCandidate {
    pub handle: Handle,
    pub signals: SignalBreakdown,
    pub chosen_level: u8,
    pub upgrade_efficiency: f32,        // Δ-Nutzen/Δ-Kosten des letzten Upgrades
}

pub struct SignalBreakdown {
    pub vector_score: Option<f32>,
    pub vector_rank: Option<u32>,
    pub bm25_score: Option<f32>,
    pub bm25_rank: Option<u32>,
    pub graph_score: Option<f32>,
    pub recency_score: f32,
    pub trust_factor: f32,
    pub fusion_score: f32,
    pub mmr_penalty: f32,
}

pub enum RejectionReason {
    BelowScoreThreshold { score: f32, threshold: f32 },
    FilteredByTrust { cell_trust: TrustLabel, min_trust: TrustLabel },
    FilteredByTime { cell_time: Timestamp, filter: (Timestamp, Timestamp) },
    BudgetExhausted { remaining: u32, cost: u32 },
    Redundant { similar_to: Handle, similarity: f32 },
    DeletedInVersion,
}
```

---

## 6. Evidence Sufficiency

Die Recall-Antwort enthält ein **Sufficiency-Signal** (`evidence_quality: f32`), geschätzt aus:

```rust
pub fn estimate_evidence_quality(result: &RecallResult, config: &SufficiencyConfig) -> f32 {
    let score_coverage = result.items.iter()
        .map(|i| i.score)
        .sum::<f32>() / result.items.len() as f32;

    let diversity = embedding_diversity(&result.items);  // Mittlere paarweise Distanz

    let budget_utilization = result.budget_used as f32 / result.budget_total as f32;

    let top_score_confidence = result.items.first()
        .map(|i| i.score)
        .unwrap_or(0.0);

    // Gewichtete Kombination
    0.3 * score_coverage
    + 0.25 * diversity
    + 0.2 * budget_utilization
    + 0.25 * top_score_confidence
}
```

Ein Agent kann damit entscheiden, ob die Evidenz ausreicht oder ob nachgeladen werden muss.

---

## 7. Abgeleitete Daten und Lineage

### 7.1 Derivation Cache

Abgeleitete Facetten unter Schlüssel `(source_chunk_hash, producer_id)` **memoisiert**:

- **Embedding-Cache:** Identischer Text → identischer Hash → Embedding wird nie zweimal berechnet.
- **Token-Zählungen** und **extraktive Zoom-Stufen** analog.
- **Tenant-Grenzen:** Cache ist **pro Repository** (kein Cross-Repo-Dedup) → kein Seitenkanal-Leak.

### 7.2 Staleness

Jede abgeleitete Facette kennt `source` (Hash). Ändert sich der Quellinhalt → Hash ändert sich → Facette **per Konstruktion erkennbar veraltet**. Keine Invalidierungsnachrichten nötig.

**Policy bei Staleness:**
1. **Strict:** Veraltete Facetten aus Recall ausschließen, Neuberechnung in Work Queue.
2. **Warn (Default):** Facetten mit Warnung liefern, Neuberechnung in Work Queue.
3. **Lazy:** Facetten ohne Warnung liefern, Neuberechnung nur bei Kapazität.

### 7.3 Lineage-DAG

`derived_from`-Kanten bilden einen DAG, der inkrementelle Neuberechnung ermöglicht:

```
L0 (Rohtext) ──► L1-Summary ──► Entity-Profil ──► Themen-Summary
       └──────► Embedding
       └──────► Token-Count
```

Bei Änderung von L0: alle Nachfahren im DAG werden als stale markiert und in die Work Queue eingestellt.

### 7.4 Work Queue

```rust
pub enum WorkTask {
    Embed       { cells: Vec<CellId>, producer: ProducerId },
    Summarize   { cluster: Vec<CellId>, target_level: u8, max_tokens: u32 },
    Consolidate { episode: SeqId, into_ns: NsId },
    Dedupe      { candidates: Vec<(CellId, CellId)> },
    Reconcile   { conflict: ConflictId },
    ReEmbed     { namespace: NsId, old_embedder: String, new_embedder: String },
    ZoomDown    { cells: Vec<CellId>, target_level: u8 },
}

pub struct WorkEntry {
    pub task_id: TaskId,
    pub task: WorkTask,
    pub priority: WorkPriority,         // Critical | High | Normal | Low | Background
    pub created_at: Hlc,
    pub lease: Option<Lease>,
    pub attempts: u8,
    pub max_attempts: u8,               // Default: 3
    pub idempotency_key: IdempotencyKey,
}
```

**Persistence:** Work Queue ist ein **System-Namespace** (`_system/work`), versioniert im RootSet. Tasks sind Cells mit `CellKind::System`.

**Leasing-Modell:**
- **At-least-once-Delivery** mit TTL-basiertem Lease.
- Lease-TTL: Default 5 Minuten, konfigurierbar pro Task-Typ.
- Bei Lease-Ablauf: Task wird für andere Worker freigegeben.
- Idempotenz-Keys stellen sicher, dass doppelte Completions harmlos sind.
- Ergebnisse sind normale Commits mit `derived_by = (model, prompt_hash)`.

### 7.5 Konsolidierung

Hintergrund-Prozess, der episodische Daten zu semantischen Fakten verdichtet:

| Schritt | Input | Output | Mechanismus |
|---|---|---|---|
| Episode → Fakten | Konversationsturns | Persistente Fakten | `Summarize`-Task in Work Queue |
| Duplikaterkennung | Semantisch ähnliche Cells | `Dedupe`-Tasks | Embedding-Ähnlichkeit > 0.95 |
| Widerspruchserkennung | Widersprüchliche Fakten | `Reconcile`-Tasks | Embedding-Distanz + Attribut-Überlappung |
| Profil-Aggregation | Einzelfakten zu Entity | Entity-Profile | `Consolidate`-Task in Work Queue |

### 7.6 Vergessen als „Zoom-Down"

Retention-Policies sind **gestuft statt binär:**

| Phase | Aktion | Auslöser |
|---|---|---|
| Hot → Warm | Cold-Tail-Lanes auf billigeres Tier | `retention.hot_days` überschritten |
| Zoom-Down | L0 entfernen, L1/L2 behalten | `retention.zoom_down_after` |
| Archive | L0 in Archiv-Tier (S3) | Manuell oder nach `warm_days` |
| Tombstone | Nur Handle + Provenance + Kanten-Spur | Salience dauerhaft < 0.01 + nicht pinned |

**Salience-Score** (siehe [SPEC-02](SPEC-02-DATA-MODEL.md), Abschnitt 9.2) bestimmt die Vergessens-Reihenfolge.

---

## 8. Adaptive Ranking

### 8.1 Drift-Detection (`contextra-adapt`)

| Signal | Methode | Reaktion |
|---|---|---|
| Embedding-Drift | Kolmogorov-Smirnov-Test auf Query-Embedding-Verteilung (Fenster: 1000 Queries) | Warnung + ggf. Re-Embedding-Trigger |
| Relevanz-Drift | Expand-Rate-Monitoring (gleitender Durchschnitt, 7 Tage) | Fusion-Gewichte anpassen |
| Modellwechsel | Neuer Embedder registriert | Re-Embedding-Work-Queue-Tasks |

### 8.2 Automatische Gewichtsanpassung

Bayessche Optimierung der Fusion-Gewichte basierend auf Expand-Events als implizites Feedback:

1. **Metrik:** Expand-Rate pro Recall (höher = Erstauswahl war unzureichend).
2. **Ziel:** Minimierung der Expand-Rate bei konstantem Budget.
3. **Update-Frequenz:** Alle 10 000 Recalls.
4. **Bounds:** Jedes Gewicht ∈ [0.05, 0.8] (kein Signal wird völlig abgeschaltet).
5. **Rollback:** Bei Verschlechterung → zurück zum vorherigen Gewichtssatz.

---

*Ende SPEC-06. Weiter: [SPEC-07 Durability](SPEC-07-DURABILITY.md)*

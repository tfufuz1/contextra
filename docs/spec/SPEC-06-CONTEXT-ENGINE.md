# SPEC-06 — Context Engine: Recall, Packer, Abstention, Verifikation, Lebenszyklus

**Version 3.0 · 26. November 2026 · Status: Final**
**Querverweise:** [Index](SPEC-00-INDEX.md) · [Datenmodell](SPEC-02-DATA-MODEL.md) · [Indizes](SPEC-05-INDEXES.md) · [API](SPEC-08-API.md)

---

## 1. Operationen

| Operation | Zweck |
|---|---|
| `recall(req)` | Hybride Suche unter Budget → `EvidencePack` |
| `window(seq, anchor, budget)` | zusammenhängendes Sequenzfenster |
| `expand(handle, level, budget?)` | andere Zoom-Stufe laden |
| `neighbors(handle, edge_kinds, limits)` | begrenzte Nachbarschaft |
| `verify(claims, recall_id?)` | Zitat- und Ankerprüfung |
| `explain(recall_id)` | Signale, Auswahl, Verwerfungen |
| `diff(a, b)` | lesbare, budgetierte Änderungsübersicht |

Jede Operation, die Text liefert, MUSS ein Token-Budget akzeptieren und einhalten. Latenzziele: [SPEC-09](SPEC-09-OPERATIONS.md) §3.

---

## 2. Anfrage und Antwort

```rust
pub struct Recall {
    pub query: Query,                    // Text | Vector{space} | Hybrid
    pub namespaces: Vec<NsId>,
    pub budget: TokenBudget,             // max_tokens, tokenizer, reserve_tokens, allow_estimate
    pub signals: Signals,                // vector, bm25, graph, recency + Gewichte
    pub filters: Filters,                // min_trust, min_grounding, taint, kinds, time, tags,
                                         // include_hypotheses, include_archived, include_history
    pub pinned: Vec<Handle>,             // stabiler Präfix
    pub diversity: Diversity,            // Coverage (Default) | None
    pub placement: OrderPolicy,
    pub valid_at: Option<TimestampMs>,   // Default: now
    pub as_of: AsOf,                     // Head | Commit | Seq | Timestamp
    pub now: TimestampMs,                // expliziter Abfragezeitpunkt
    pub render: RenderFormat,            // Compact | Xml | Json | Raw
    pub rerank: Option<RerankConfig>,    // Deadline Default 500 ms
    pub deadline: Duration,
    pub explain: bool,
}
```

Der Principal ist **kein** Anfragefeld; er stammt aus Session bzw. Capability.

```rust
pub struct RecallResult {
    pub recall_id: RecallId,             // H(snapshot ‖ request_digest ‖ principal_class)
    pub snapshot: SnapshotRef,           // branch, commit, seq, catalog, index_manifest, now
    pub items: Vec<RecallItem>,
    pub rendered: String,
    pub budget_used: u32, pub budget_total: u32,
    pub sufficiency: EvidenceSufficiency,
    pub evidence_quality: Option<f32>,   // nur bei aktiver Kalibration
    pub lane_coverage: LaneCoverage,
    pub warnings: Vec<PackWarning>,      // Conflict | Stale | Tainted | PartialIndex |
                                         // RerankSkipped | ThinEvidence | EstimatedTokens | Truncated
    pub prefix_digest: [u8; 16],
    pub truncated: bool,
}
```

---

## 3. Pipeline

```
1 Snapshot pinnen, Capability prüfen, Budget validieren
2 allow-Maske (Gid)                                   SPEC-05 §2
3 Kandidaten je Lane parallel: n_cand = max(100, 4·k)
4 Fusion → Priors
5 Validierung gegen Truth
6 optional Rerank (Deadline)
7 Sufficiency-Vorprüfung
8 Packer → Platzierung → Rendering → Budgetverifikation
```

Fällt ein Lane aus oder läuft er in die Deadline, wird ohne ihn fusioniert und `lane_coverage` entsprechend gemeldet. Fallen alle Such-Lanes aus ⇒ Fehler, **nicht** `NO_EVIDENCE_FOUND`: „nicht suchen können" ist keine Evidenzaussage.

### 3.1 Fusion

Default RRF: `s(d) = Σ_i w_i / (60 + rank_i(d))`, Rang ab 1. Relevanz `rel(d) = s(d) / Σ_i (w_i / 61)` ∈ (0, 1] über die aktiven Lanes.

Alternative `Calibrated`: isotonische Regression je Lane (an Space bzw. Analyzer gebunden), gewichtetes Mittel über die verfügbaren Lanes. Training aus Expand- und Reviewer-Signalen; Aktivierung nur nach Validierung auf Holdout, mit Rollback. Rohdaten verlassen den Store nicht.

### 3.2 Priors

```
score(d) = rel(d) · trust_factor · conf_factor · freshness
trust_factor:  System/Verified/Curated 1.00 · ToolVerified 0.92 · AgentInferred 0.60 · Untrusted 0.35
conf_factor  = 0.5 + 0.5 · lcb(d)
freshness    = max(floor, 2^(−max(0, now − t_ref) / half_life)),   floor ∈ [0.6, 1.0]
t_ref        = valid_from, sonst ts_tx
```

Die Konfigurationsprüfung lehnt `floor < 0.6` ab. Mit Default-Priors gilt: Bei gleicher Relevanz wird ein Treffer ab `ToolVerified` nie **allein durch sein Alter** hinter einen `AgentInferred`- oder `Untrusted`-Treffer gereiht (Property-Test). Gegen widersprechende Evidenz oder ersetzte Gültigkeit schützt nicht der Decay, sondern `valid_at` und die Konfidenz. Decay löscht nie; `ignore_decay` ist möglich.

### 3.3 Rerank

Cross-Encoder über `Reranker`, nur auf bereits autorisierten Texten; Cloud-Transport läuft durch den Egress-Guard. Deadline überschritten ⇒ Fusionsreihenfolge plus `RerankSkipped`. Rerank ersetzt `rel`, die Priors bleiben.

---

## 4. Packer

### 4.1 Zielfunktion

Kandidaten `D`, Renditionen `ℓ ∈ {L0, L1, L2, L3, parent}`. Je Dokument höchstens eine Rendition.

```
w_j        = score(j) · quality(j)                 quality ∈ [0.5, 1.0] (IDF-Dichte je Token)
gain(ℓ)    : L0 1.0 · L1 0.7 · L2 0.3 · L3 0.05
sim(d, j)  = 1 für d = j, sonst clamp((cos(d,j) − 0.80) / 0.20, 0, 1)
F(S)       = Σ_{j∈D} w_j · max_{(d,ℓ)∈S} gain(ℓ) · sim(d, j)
maximiere F(S)  unter  Σ_{(d,ℓ)∈S} tok(d, ℓ) ≤ B
```

`F` vereint Zoom-Wahl und Redundanzabzug: Ein zweites, nahezu gleiches Dokument bringt kaum Zugewinn, verschiedene Inhalte zählen voll. Ohne Embeddings ist `sim` für `d ≠ j` gleich 0 (bzw. lexikalische MinHash-Ähnlichkeit).

### 4.2 Verfahren

```
S ← ∅; Heap über Optionen (d, ℓ_neu) mit Dichte ΔF / max(Δtok, 1)
solange Heap nicht leer:
    o ← Maximum; Dichte neu berechnen (lazy)
    falls neue Dichte < nächstbeste: zurücklegen, weiter
    falls Δtok passt und ΔF > 0: anwenden (ersetzt ggf. Rendition von d); Folgeoptionen von d einfügen
vergleiche S mit der besten Einzeloption unter B; nimm das größere F
```

Je Dokument werden nur Renditionen auf der oberen konvexen Hülle `(tok, gain)` betrachtet. Gleichstände: höherer `score`, dann kleinere `Gid` (Determinismus).

**Keine formale Approximationsgarantie:** Die Kombination aus Budget, Renditionswahl je Dokument und Elternexpansion erfüllt die Voraussetzungen der bekannten Knapsack-Schranken nicht. Verbindlich ist stattdessen der Test in §10 gegen die erschöpfende Optimallösung kleiner Instanzen.

### 4.3 Vorverarbeitung und Small-to-Big

- **Harte Dubletten:** lexikalische MinHash-Jaccard-Schätzung ≥ 0,9 → ein kanonischer Kandidat (höchster Trust, dann Score), übrige als Alias gelistet. Widersprechende Claims werden nie dedupliziert.
- **Small-to-Big:** Sind ≥ 2 Kinder eines Elternabschnitts Kandidaten, wird `parent` als Option angeboten; seine Wahl ersetzt die gewählten Kinder und deckt deren Gewichte mit `gain(parent_zoom)` ab. Der Elternabschnitt unterliegt eigener ACL- und Trust-Prüfung; sein Text darf keine unsichtbaren Kinder enthalten, sonst entfällt die Option.

### 4.4 Budgetgarantie (INV-8)

Tokenzahlen sind unter Verkettung nicht streng additiv. Daher:

1. Planung mit vorberechneten Zählwerten plus Rahmenkosten.
2. Rendering.
3. **Exakte Zählung des gesamten Strings** mit dem angeforderten Tokenizer.
4. Bei Überschreitung: Option mit geringstem Zugewinnverlust je Token herabstufen oder entfernen; zurück zu 2. Terminiert, weil jede Iteration eine Rendition strikt verkleinert.

Nie wird mitten im Satz abgeschnitten. Ist der Tokenizer nicht registriert, wird die Anfrage abgelehnt, außer `allow_estimate = true`: dann 5 % Marge und Warnung `EstimatedTokens`; INV-8 gilt in diesem Modus nur als Schätzung. Passt nicht einmal die Minimalantwort (Statuszeile) ⇒ `budget_insufficient` mit Mindestwert.

### 4.5 Platzierung

| `OrderPolicy` | Anordnung |
|---|---|
| `CacheFriendly` (Default) | `pinned` in Anfragereihenfolge als stabiler Präfix, danach stabile Arten (`Config`, `Skill`) nach `content_hash`, dann volatile Treffer nach Score |
| `Relevance` | nach Score, je Quelle gruppiert |
| `Chronological` | nach `t_ref` |
| `ByDocument` | Dokumentreihenfolge |
| `EdgesFirst` | beste Treffer an Anfang und Ende |

`prefix_digest = trunc128(BLAKE3(gerenderte Präfix-Bytes))`. Nur ein byteidentischer Präfix verspricht KV-Cache-Wiederverwendung; deshalb sind Zeitstempel, `recall_id` und Zähler im Präfix verboten. `EdgesFirst` wird erst bei belegtem A/B-Gewinn Default.

---

## 5. Rendering und Injection-Härtung

```text
[c:k7qd-3f] (L1, 142 tok, trust=tool-verified, lcb=0.62, 2026-10-14)
Deployment schlug fehl wegen abgelaufenem Registry-Token; Lösung: Token-Rotation …
<<DATA 9f3a1c7e untrusted taint=external>>
[c:r4tn-7k] (L1, 88 tok, trust=untrusted)
… Text aus externer Quelle …
<<END 9f3a1c7e>>
-- 11 weitere Handles via expand | 5874/6000 tok | sufficiency=sufficient
```

Zahlen und Handles sind illustrativ.

- Der Rahmen-Marker ist `trunc32(HMAC(k_frame, recall_id ‖ block_no))` und damit für Inhalte nicht vorhersagbar.
- **Alle** Inhalte (unabhängig vom Trust) werden so maskiert, dass sie keine Engine-Struktur nachahmen können: Zeilen, die mit `[c:`, `<<` oder `-- ` beginnen, erhalten ein Escape-Zeichen. `source_uri` und Attribute werden längenbegrenzt und steuerzeichenbereinigt.
- Verifikation arbeitet auf Rohtext, nicht auf maskiertem Rendering.
- Datenrahmen **mindern** das Injection-Risiko, sie beseitigen es nicht. Autorisierung von Aktionen darf nie auf Prompt-Text beruhen.

---

## 6. Sufficiency und Abstention

Rangbasierte Fusionswerte sagen nichts über absolute Relevanz. Die Schwelle nutzt daher absolute Signale je Kandidat:

```
abs(d) = max( calib_vec(cos) , calib_lex(idf-gewichteter Anteil gematchter Query-Terme) , rerank(d) )
```

Ohne Kalibration gelten je Space bzw. Analyzer konfigurierte Untergrenzen (`min_cosine`, `min_term_coverage`); sie werden beim Registrieren eines Space auf Referenzfragen bestimmt. Fehlt die Konfiguration, ist höchstens `Thin` erreichbar.

Auswertung in dieser Reihenfolge:

| Stufe | Bedingung | Rendering |
|---|---|---|
| `None` | kein autorisierter Kandidat über der Schwelle | `NO_EVIDENCE_FOUND` mit durchsuchtem Bereich und Abdeckung |
| `Conflicting` | ausgewählter Treffer hat `Contradicts` zu einer gültigen, sichtbaren Cell (oder offene `ConflictCell` auf diesem Branch) | `CONFLICT`-Block mit **beiden** Seiten, Provenienz, `lcb` |
| `Sufficient` | ≥ 1 Treffer über Schwelle mit `GL0`–`GL2`, Trust ≥ `min_sufficient_trust` (Default `ToolVerified`), oder ≥ 2 unabhängige `source_root`s | regulär |
| `Thin` | sonst | Warnung `ThinEvidence` |

Bei `lane_coverage < 1` ergänzt `None` den Hinweis, dass die Suche unvollständig war. Reicht das Budget nicht für beide Konfliktseiten, werden beide auf kleinere Stufen gesetzt; nie wird nur eine Seite gezeigt.

---

## 7. Expand und Verify

### 7.1 `expand`

Syntax → MAC → `Gid` → Sichtbarkeit und ACL im Snapshot → Lane-Read. Nicht vorhanden und nicht berechtigt sind **ununterscheidbar** (`unknown_handle`). Das Ergebnis trägt denselben Trust-/Taint-Rahmen wie Recall. L0 größer als das Budget ⇒ satzgrenzengerechter Ausschnitt mit Fortsetzungs-Cursor.

### 7.2 `verify`

Deterministisch, optional ergänzt durch `Judge`. Geprüft werden Zahlen mit Einheiten, Datumsangaben, wörtliche Zitate und Eigennamen/Bezeichner gegen die zitierten Quellspannen nach Normalisierung.

| Verdikt (Vorrang von oben) | Bedingung |
|---|---|
| `InvalidHandle` | MAC falsch, unbekannt oder nicht berechtigt |
| `Expunged` | Inhalt entfernt |
| `Stale` | im Prüf-Head ersetzt oder retired |
| `Contradicted` | sichtbarer Widerspruch zur Quelle |
| `NotInContext` | Handle gültig, aber nicht im referenzierten Pack bzw. Expand der Session |
| `AnchorMismatch` | ein Anker der Aussage fehlt oder weicht ab |
| `Supported` | alle extrahierten Anker belegt, mindestens einer vorhanden |
| `UnanchoredParaphrase` | keine prüfbaren Anker |

`Supported` bedeutet **lexikalisch verankert**, nicht logisch erwiesen (z. B. werden Negationen nicht sicher erkannt). Das Ergebnis weist `method = lexical | judge` aus.

---

## 8. Explain

Je `recall_id` (Ringpuffer, Default 15 min bzw. 1 000 Einträge, an Principal gebunden): Snapshot, Lanes und Abdeckung, Segmentstrategien, je **sichtbarem** Kandidat Signale, Priors, Rendition, Grund (`BelowThreshold | BudgetExhausted | Redundant | Duplicate | Retired | StaleIndexEntry`). Unautorisierte Cells erscheinen **nicht** — auch nicht als Zähler oder Grund „gefiltert".

---

## 9. Derivationen, Konsolidierung, Lebenszyklus

### 9.1 Jobs

```rust
pub enum WorkTask {
    Embed { cells, space }, Summarize { cells, target_level, max_tokens },
    ExtractEntities { cells }, Consolidate { episode, into },
    CommunitySummary { community, members }, Dedupe { candidates },
    CheckContradiction { a, b }, Reconcile { conflict },
    BuildIndex { lane, range }, CompactPacks { packs },
}
```

Queue-Semantik: [SPEC-04](SPEC-04-VERSIONING.md) §8.2. Konsolidierung: Episode → Fakten (Session-Branch, `AgentInferred`, mit Quellspannen) → Dublettenprüfung → Widerspruch in drei Stufen (deterministisch → Kandidatensuche → `Judge`) → Profile → Community-Summaries. Worker erhalten nur Inhalte, die ihr Principal sehen darf; Cloud-Worker nur über den Egress-Guard.

Embeddings werden beim Ingest standardmäßig **vor** dem Commit aufgelöst (Batch 32). Ist der Embedder nicht verfügbar oder `Deferred` gewählt, wird ohne Vektor committed und `pending` gesetzt.

### 9.2 Lebenszyklus

| Stufe | Wirkung | Reversibel |
|---|---|---|
| Archivieren | Flag `ARCHIVED`, aus Default-Recall; `PINNED` schützt | ja |
| Zoom-Down | L0 nach Cold-Tier; L1/L2, Handle, Provenienz, Kanten bleiben; `expand(L0)` meldet Tier-Latenz | ja, solange archiviert |
| `retire` | Tombstone-Commit mit Begründung; historisch innerhalb Retention sichtbar | ja, durch neuen Commit |
| Retention-Ablauf | Compaction entfernt jenseits des Horizonts | nein |
| `purge` | administrativer Expunge ([SPEC-07](SPEC-07-DURABILITY.md) §6) | nein |

Zoom-Down in die Löschung von L0 (statt Auslagerung) ist eine Retention-Entscheidung und wird nur durch explizite Policy ausgelöst. `working`-TTL erzeugt Retire-Commits.

### 9.3 Drift

KS-Test über Query-Embedding-Projektionen und Expand-Rate-Trend erzeugen Warnungen und Vorschläge. Gewichte werden nie ohne validierte Kalibration automatisch geändert.

---

## 10. Qualitätsmetriken und Abnahme

| Metrik | Gate |
|---|---|
| Budget | 100 % `exact_count ≤ max − reserve` über Fuzz-Korpus mit allen Formaten |
| Packer vs. Optimum (n ≤ 14) | Median ≥ 0,95, Minimum ≥ 0,70 des optimalen `F` [Zielwert, in P3 zu kalibrieren] |
| Utility@Budget | signifikant über Top-k-Baseline (gepaarter Test, fester Korpus) |
| Abstention | Präzision und Recall auf unbeantwortbaren Fragen berichtet, Schwellen versioniert |
| Injection-Korpus | keine Rahmen- oder Header-Fälschung |
| ACL | keine unsichtbare Information in Pack, Explain, Verify |

Berichtet zusätzlich: Nutzen pro Token, Expand-Rate, Grounding-Rate, Konfliktsichtbarkeit, Task-Erfolg bei gleichem Budget.

---

*Weiter: [SPEC-07 Durability und Sicherheit](SPEC-07-DURABILITY.md)*

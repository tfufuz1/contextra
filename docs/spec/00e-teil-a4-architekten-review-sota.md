---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "00e"
---
## Teil A4 — Architekten-Review: Machbarkeit & Optimierungspotenzial der SOTA-Algorithmen (Fassung 4)

> **Rolle dieses Teils:** Ich (Principal Senior Rust Architect für Contextra) habe für diese Fassung das
> Repository `github.com/tfufuz1/contextra` live geklont (`git clone`, `HEAD 4b9387d6119be17180de21a0ea5b2be98121357a`,
> Commit-Message „refactor(rename): memfuse -> contextra (mechanical, script-driven) (#3494)") — **derselbe
> Commit**, gegen den bereits Teil C des Deep-Research-Berichts geprüft wurde. Das erlaubt einen direkten,
> reproduzierbaren Zweitabgleich statt einer bloßen Übernahme fremder Befunde. Methodik: `grep`/`wc -l` auf
> Datei-, Funktions- und Typnamen, `view` auf vollständige Modulinhalte, keine Ausführung von `cargo build`
> (Sandbox ohne Zugriff auf crates.io für alle workspace-internen Dependencies außer den in
> `<network_configuration>` freigegebenen Registries — ausreichend für Lesezugriff, nicht für einen vollen
> Build). Wo eine Aussage aus Teil C durch meine eigene Prüfung bestätigt wird, übernehme ich sie ohne
> erneuten Beleg; wo sie abweicht, ist das unten ausdrücklich als Korrektur markiert.

### A4.1 Ergebnis in einem Satz

Von den vier untersuchten Verfahren ist **keines** direkt copy-paste-fähig: Alle vier setzen auf Typen,
Traits oder Streaming-Fähigkeiten auf, die im jeweiligen Ziel-Crate entweder in anderer Form existieren
(→ Anpassung der normativen Schnittstellen nötig, siehe §21) oder schlicht fehlen (→ eigener Vorarbeits-Schritt
vor der eigentlichen Algorithmus-Implementierung); die Priorisierung des Deep-Research-Berichts (§6 dort)
gewichtet ausschließlich nach P24-Nutzen und ignoriert Beleglage und Implementierungsreife — Teil C hat das
bereits korrigiert (C.7), meine eigene Prüfung bestätigt C.7 im Kern und verschärft sie für DiBud um einen
zusätzlichen, bisher nicht benannten Blocker (A4.4.2).

### A4.2 Zweitverifikation der Teil-C-Befunde — Bestätigungen und Korrekturen

Alle in Teil C des Deep-Research-Berichts zitierten Datei-/Symbolbefunde wurden nachvollzogen:

| Teil-C-Befund | Meine Zweitprüfung | Ergebnis |
|---|---|---|
| `ppr.rs`: `forward_push_ppr`, `DensePowerIteration`, `PprAlgorithm::ShadowMode` | `grep -n` bestätigt alle drei Symbole exakt (`ppr.rs:124,132,134,137,177`) | **bestätigt** |
| `fusion.rs` (1332 Zeilen), `BoundedTopK<T>` (Z. 222), `rrf_k`-Parameter (Z. 419ff.) | `wc -l` = 1332, `grep` bestätigt Zeilenlage exakt | **bestätigt** |
| `lyapunov.rs` (552 Zeilen), `drift.rs`: `DriftPolicyBridge`, `bandit.rs`: `gamma_inv` (Korrektur der Discount-Richtung) | `wc -l` = 552 bestätigt; `DriftPolicyBridge` in `drift.rs:10`; `gamma_inv` in `bandit.rs:239,394` bestätigt | **bestätigt** |
| `memory_consolidation.rs` (972 Zeilen), Structural Pass, `group_turns_into_segments`, `detect_near_duplicates`, `ConsolidationConfig` (Z. 28) | `wc -l` = 972 bestätigt; alle vier Symbole und die Zeilenposition von `ConsolidationConfig` exakt bestätigt | **bestätigt** |
| `synthesis_phase.rs`: Generative Pass mit `max_llm_calls_per_cycle`, `CommunityStabilityTracker` | Datei existiert; zusätzlich `SynthesisConfig` und ein **zweites** `SynthesisPhaseResult` in `memory_consolidation.rs` selbst gefunden (Teil C nennt nur eines) | **bestätigt, ergänzt** — es gibt zwei mit dem Compiler eindeutig auflösbare, aber verwirrend gleichnamige `SynthesisPhaseResult`-Definitionen in unterschiedlichen Modulen; vor jeder Erweiterung um eine dritte Stufe muss dies vereinheitlicht werden (siehe §21.4) |
| `cascade.rs` (733 Zeilen), `MAX_HYPEREDGE_CASCADE_FANOUT`, `CASCADE_QUEUE_PREFIX`, `enqueue_cascade_deferred`, `cascade_invalidate_hyperedges_for_superseded_doc` | `wc -l` = 733 bestätigt, alle vier Symbole exakt an den genannten Stellen | **bestätigt** |
| `schemas/contextra.fbs`: `HyperEdge`-Tabelle mit `source_doc_id: ulong`, **kein** `child_edge_ids` | Volltext des Schemas gelesen — Tabelle heißt tatsächlich `HyperEdge` (nicht `HyperEdgeFb`, wie auch Teil C bereits anmerkt und wie **§12 dieser Spezifikation selbst** sie nennt, siehe A4.2.3 unten), `source_doc_id: ulong` vorhanden, kein `child_edge_ids` | **bestätigt** |
| „`grep` über den gesamten Workspace nach `estimate_compaction_peak_bytes` … liefert keine Treffer" | **Widerlegt.** `grep -rn "estimate_compaction_peak_bytes" --include="*.rs" .` liefert drei Treffer: `crates/contextra-graph/src/csr/graph_write.rs:205,207` (öffentliche Methode auf `CsrGraph`, delegiert an Snapshot) und `crates/contextra-graph/src/csr/inner.rs:387` (die eigentliche Implementierung), plus ein dedizierter Test `crates/contextra-graph/tests/hyperedge_memory_budget.rs:46`, der bereits gegen AK-2 (§16.2) prüft | **Korrektur, siehe A4.2.1** |
| „`grep` … nach … `consolidate_semantic_hyperedges` liefert keine Treffer" | Bestätigt — keine Treffer, die Ring-3-Orchestrierungsfunktion existiert tatsächlich nicht | **bestätigt** |

#### A4.2.1 Korrektur: Der Ring-0-Budget-Check für die Kompaktierung existiert bereits

Die im Bericht als fehlend angenommene Methode `estimate_compaction_peak_bytes()` ist **bereits produktiv
implementiert** — nicht als Stub, sondern mit eigenem Regressionstest (`hyperedge_memory_budget.rs`), der
laut AK-2 (§16.2) bereits prüft, dass die Schätzung den mit Zählallokator gemessenen Spitzenwert nie
unterschätzt und höchstens um Faktor 1,5 überschätzt. Das ändert die Aufwandsschätzung für LeanRAG
(§5.4 des Berichts, §21.4 dieser Spezifikation) spürbar: Der im Bericht als Kernbestandteil des neuen
`ConsolidationConfig`/`consolidate_semantic_hyperedges`-Entwurfs vorgesehene Budget-Vorprüfungs-Aufruf
(`self.graph.estimate_compaction_peak_bytes()`) ist kein neu zu bauender Baustein, sondern ein reiner
**Wiederverwendungs-Aufruf** einer bereits getesteten Ring-0-Primitive. Das senkt das in Teil C (C.6)
beschriebene Restrisiko für den Speicherbudget-Teil der LeanRAG-Integration von „zu bauen" auf „zu verdrahten".

#### A4.2.2 Korrektur: DiBuds Kanalmodell passt nicht auf das tatsächliche `SignalKind`

Der Bericht (§5.2) und Teil C (C.3) gehen von vier Fusionskanälen `{Vektor, Text, Graph, Metadaten}` mit
Tie-Breaker-Priorität „1. Graph, 2. Text, 3. Vektor, 4. Filter" aus. Das tatsächliche, laut §6.6 H3 dieser
Spezifikation **geschlossene** Enum in `crates/contextra-rank/src/fusion.rs:316` lautet:

```rust
pub enum SignalKind {
    Vector,
    Text,
    Graph,
    EdgeReinforcement, // Bandit-Rückkopplungssignal, kein "Filter"/"Metadaten"-Kanal
}
```

Der vierte Kanal ist `EdgeReinforcement` (Bandit-Recency-Gewicht), kein Metadaten-/Filter-Kanal. Das ist mehr
als eine Umbenennung: Ein Filter-Kanal ist per Definition ein zusätzliches, unabhängig auslesbares
Ranking-Signal; `EdgeReinforcement` ist dagegen ein **rückgekoppeltes** Signal aus dem Bandit-Subsystem
(§8) — sein Zugriffsmuster (Lesen aus `contextra-adapt`-Zustand, nicht aus einem eigenen Postings-Index) ist
grundlegend anders als das der drei Retrieval-Kanäle. Die normative `DiBudFusionState`-Schnittstelle aus dem
Bericht muss entsprechend korrigiert werden (§21.2); insbesondere ist unklar, ob ein Budget-limitierter
Iterator für `EdgeReinforcement` überhaupt sinnvoll ist, da dieses Signal nicht paginiert aus einem Index
gelesen, sondern pro Dokument direkt berechnet wird. **Empfehlung:** DiBud zunächst nur auf die drei echten
Retrieval-Kanäle (Vector, Text, Graph) anwenden; `EdgeReinforcement` bleibt ein vierter, stets vollständig
ausgewerteter additiver Term außerhalb des Budgets — das ist zugleich die konservativere, P24-konformere
Wahl, weil sie das Budget nicht künstlich auf ein Signal ausdehnt, das strukturell kein Zugriffsbudget hat.

#### A4.2.3 Korrektur/Ergänzung: `fuse_signals` ist bereits vollständig materialisiert, kein Iterator vorhanden

Eigene Prüfung von `crates/contextra-rank/src/fusion.rs`, Zeile 155–160:

```rust
pub fn fuse_signals(
    result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
) -> Vec<FusedScore> {
    weighted_reciprocal_rank_fusion(result_sets, max_results)
}
```

Das bestätigt Teil C (C.3) exakt: `fuse_signals` erhält bereits vollständig gefüllte `Vec<SearchResult>` pro
Kanal — es gibt keinerlei Iterator-, Generator- oder Lazy-Loading-Grenze an dieser Schnittstelle, durch die
ein `DiBudFusionState` streambasiert „hindurchgreifen" könnte. Zusätzlich zur bereits in Teil C (Restrisiken
§5.2) benannten Sorge, dass `contextra-vector` (DiskANN) und `contextra-text` (BM25/Block-Max-WAND) intern
Block-optimierte Heaps statt echter `Next()`-Iteratoren verwenden, habe ich geprüft, **ob** dort überhaupt
eine Iterator-Implementierung vorhanden ist, die sich wiederverwenden ließe: In
`crates/contextra-vector/src/diskann.rs` und `crates/contextra-text/src/wand.rs` finden sich keine
`impl Iterator`-Blöcke, die einzelne `DocId`s auf Anfrage nachliefern — beide Module sind auf vollständige
Ergebnismengen pro Aufruf ausgelegt. **Das ist eine dreifache, nicht nur einfache Bruchstelle** (fehlender
Iterator in DiskANN, fehlender Iterator in WAND, vollständig materialisierende Signatur von `fuse_signals`
selbst) — die Aufwandsschätzung „Gering-Mittel" aus §17 Punkt 1.8 dieser Spezifikation (die sich nur auf die
`build_provenance`-Parameterstruktur bezieht) darf nicht mit dem Aufwand für DiBud verwechselt werden; DiBud
ist strukturell näher an „Hoch" (vgl. §17-Einstufung von 1.9 „Text: Posting-Format umstellen") einzuordnen,
siehe A4.4.2.

### A4.3 Konsolidierte Vertrauens-/Beleglage

Ich übernehme die von Teil C (C.1) erstmals eingeführte, im Originalbericht fehlende Tier-Einordnung
unverändert — meine Prüfung der Repo-Fakten liefert keinen Anlass, eine der dort getroffenen
Autoren-/Reviewer-Aussagen zu revidieren (arXiv-Metadaten wurden nicht Teil dieser Zweitprüfung; das bleibt
laut Teil C C.8 ein offener Prüfpunkt für einen Menschen):

| Verfahren | arXiv-ID | Tier (Teil C) | Ziel-Crate |
|---|---|---|---|
| LeanRAG Path Extraction | 2508.10391 | **A — peer-adjacent, breit rezipiert** | `contextra-cognition` |
| Type-Info-Denoising | 2503.09916 | **A — peer-reviewed (AISTATS 2025)** | `contextra-cognition` |
| TL-HFD | 2606.09340 | **B — solide, nicht begutachtet** | `contextra-graph` |
| Flow-Corrected Thompson Sampling | 2606.23933 | **C — Workshop-Status** | `contextra-adapt` |
| DiBud | 2609.15143 | **D — frisch (7 Tage), unrepliziert, Einzelautor** | `contextra-rank` |
| Motif Conductance / Robust Rank Aggregation | 2507.10570 / 2609.19491 | **ungeprüft** | (Zusatzverfahren zu TL-HFD/DiBud) |

### A4.4 Invarianten-Konformität je Verfahren (Zero-Panic P4(1), Determinismus P4(3), Lokalität P24/P4(6), injizierter RNG P28)

| Verfahren | P4(1) Zero-Panic | P4(3) Determinismus | P24 Lokalität | P28 Injizierter RNG | Sync/Async-Grenze |
|---|---|---|---|---|---|
| TL-HFD | ✅ erreichbar — Top-k-Aktivierung und Präallokation nach bekanntem Maximum sind mit dem in `ppr.rs` bereits etablierten Muster kompatibel | ✅ erreichbar — Tie-Breaker über totale `EntityId`-Ordnung ist derselbe Mechanismus wie in H2/H6 dieser Spezifikation bereits gefordert | ✅ mathematisch strenger als der bestehende ε-Schwellen-Forward-Push, da `k` hart statt statistisch begrenzt ist | nicht benötigt (kein Zufallsschritt im Kernalgorithmus) | rein synchron, Ring 0 — passend |
| DiBud | ✅ erreichbar bei fester Kapazitäts-Vorabreservierung | ✅ erreichbar, aber **abhängig von A4.2.2**: der Tie-Breaker aus dem Bericht setzt eine Kanalpriorität voraus, die erst nach Korrektur des Kanalmodells eindeutig ist | ✅ das eigentliche Ziel des Verfahrens — aber nur wirksam, sobald die in A4.2.3 beschriebene Streaming-Lücke geschlossen ist; **bis dahin liefert eine Implementierung gegen die heutige `fuse_signals`-Signatur keinen echten P24-Gewinn**, weil das Budget den bereits vollständig geladenen Listen nachträglich aufgeprägt würde | nicht benötigt | rein synchron, Ring 0 — passend, aber siehe Sperre oben |
| FC-TS | ✅ erreichbar — Sherman-Morrison-Update ist bereits als Safe-Rust-Muster im Repo vorhanden (`bandit.rs`) | ✅ **nur** wenn der Transport-/Drift-Vektor $\hat\delta_t$ ausschließlich im Ring-3-Hintergrund aktualisiert und im Ring-0-Hot-Path nur gelesen wird (Bericht selbst fordert das in den Restrisiken) | ✅ wenn wie gefordert entkoppelt | ✅ Bericht sieht `contextra_ports::Rng`-Injektion explizit vor — konsistent mit P28 | Hot-Path synchron (Ring 0), Drift-Schätzung asynchron (Ring 3) — Trennung ist im Repo durch `DriftPolicyBridge` bereits vorgezeichnet |
| LeanRAG | ✅ solange `estimate_compaction_peak_bytes()` (bereits vorhanden, A4.2.1) **vor** jeder Allokation der GMM-Cluster-Strukturen aufgerufen wird | ✅ GMM-Seed und Community-Hashing sind bereits etablierte Muster (`gmm_deterministic_seed`, `CommunityStabilityTracker`) | ✅ per Definition — reiner Ring-3-Batch-Job, kein Hot-Path-Einfluss | nicht im Kernpfad benötigt (nur GMM-Initialisierung, per festem Seed bereits deterministisch) | rein asynchron, Ring 3 — passend, **aber siehe H5-Blocker in A4.4.1** |

#### A4.4.1 Bestätigter Hard-Blocker: Cascade-Invalidierung kann Super-Hyperkanten nicht durchqueren

Ich bestätige den in Teil C (C.6) beschriebenen Befund eigenständig: `cascade_invalidate_hyperedges_for_superseded_doc`
(`crates/contextra-graph/src/cascade.rs:161`) arbeitet über `source_doc_id`-Ketten mit hartem Fan-out-Limit
(`MAX_HYPEREDGE_CASCADE_FANOUT = 1_000`, `cascade.rs:11`) und einer persistenten Deferred-Queue
(`CASCADE_QUEUE_PREFIX`, `cascade.rs:67`). Ohne ein `child_edge_ids`-Feld (§12, §21.4) kann diese Funktion
einen von LeanRAG erzeugten abstrakten Super-Knoten nicht bis zu den Original-Tripeln durchqueren — ein
Löschauftrag (Art.-17-DSGVO-Pfad, §10) für ein Quelldokument, dessen Fakten in einen $\alpha_j$-Knoten
abstrahiert wurden, würde diesen Knoten **nicht** invalidieren. Ich übernehme die Einstufung aus Teil C
unverändert: **das ist ein Blocker, keine offene Frage** — LeanRAG darf nicht produktiv (auch nicht hinter
einem reinen Opt-in-Feature-Flag mit realen Nutzerdaten) aktiviert werden, bevor H5 (§6.6) um die
rekursive `child_edge_ids`-Traversierung erweitert ist. Diese Reihenfolge ist in §21.4 normativ festgeschrieben.

#### A4.4.2 Neu identifizierter Blocker: DiBud erfordert Vorarbeit in drei fremden Crates, bevor der Algorithmus selbst beginnen kann

Über C.1 (Tier D, unreplizierter Einzelautor) hinaus zeigt A4.2.3, dass DiBud — anders als TL-HFD, das
lediglich einen bestehenden Algorithmus innerhalb desselben Crates (`contextra-graph`) ersetzt — **drei**
Crate-Grenzen gleichzeitig berührt: `contextra-vector` (DiskANN müsste einen `Iterator<Item = DocId>`
exponieren), `contextra-text` (BM25/WAND ebenso) und `contextra-rank` selbst (`fuse_signals`-Signatur ändert
sich von `Vec<...>` auf generische Iteratoren, was laut Bericht selbst ein Batching-Intervall von z. B. 16
Kandidaten pro Kanalzugriff braucht, um die Cache-Lokalität P25 nicht zu verletzen). Das ist eine
Mehr-Crate-Schnittstellenänderung, keine lokale Algorithmus-Ersetzung — der Aufwand liegt strukturell näher
an §17 Punkt 1.9 (Text-Posting-Format, „Hoch") als an einem reinen RRF-Austausch. **Empfehlung:** DiBud in
zwei Schritten umsetzen — zuerst die Streaming-Iterator-Grenzen in `contextra-vector`/`contextra-text` als
eigenständige, algorithmusunabhängige Vorarbeit (§21.2, Schritt 1), erst danach den eigentlichen
Budget-Fusions-Algorithmus (§21.2, Schritt 2), mit einer eigenen Klein-Implementierung + Benchmark gegen den
im Paper beschriebenen Aufbau (Empfehlung aus Teil C C.1), bevor produktiv umgestellt wird.

### A4.5 Revidierte Priorisierung (erweitert C.7)

Ich übernehme die Rangfolge aus Teil C C.7 im Kern, verschärfe aber den DiBud-Rang um die in A4.4.2
identifizierte Mehr-Crate-Abhängigkeit und mache das ShadowMode-Wiederverwendungsmuster (bereits in
`ppr.rs` vorhanden, C.2) zur verbindlichen Einführungsstrategie für **alle vier** Verfahren, nicht nur für
TL-HFD:

| Rang | Verfahren | Δ ggü. Teil C | Begründung (Fassung 4) |
|---|---|---|---|
| 1 | **LeanRAG Semantic Aggregation** (§21.4, dritte Pipeline-Stufe) | unverändert Rang 1 | Höchstes Beleg-Tier, größter Teil der Infrastruktur bereits vorhanden — **und durch A4.2.1 sogar noch mehr als von Teil C angenommen** (Budget-Check bereits fertig). Einziger echter Blocker ist H5/`child_edge_ids` (A4.4.1) — technisch klar umrissen, kein Forschungsrisiko. |
| 2 | **TL-HFD** | unverändert Rang 2 | Solide Beweislage (Tier B), Shadow-Mode-Infrastruktur bereits vorhanden und **exakt** für einen Vergleichslauf gegen Forward-Push nutzbar (A4.2, bestätigt). Reine Ring-0-Algorithmus-Ersetzung innerhalb eines Crates — geringste Schnittstellen-Reichweite aller vier Verfahren. |
| 3 | **DiBud** | inhaltlich verschärft, Rang unverändert bei 3 | Tier D bleibt bestehen (C.1); zusätzlich durch A4.4.2 bestätigt: DiBud ist **keine** lokale Algorithmus-Ersetzung, sondern erfordert Streaming-Iterator-Vorarbeit in zwei fremden Crates, bevor der eigentliche Budget-Algorithmus überhaupt getestet werden kann. **Bedingung für Beginn:** eigene Kleinst-Implementierung + Benchmark (Teil C C.1) UND die Vorarbeit aus A4.4.2 Schritt 1 — beides vor jeder produktiven Aktivierung. |
| 4 | **Flow-Corrected Thompson Sampling** | unverändert Rang 4 | Ersetzt eine bereits funktionierende Komponente (Lyapunov-Watcher + `DriftPolicyBridge`, bestätigt A4.2) durch eine feinere Variante; Tier C (Workshop), zusätzlicher Speicherbedarf für das gleitende Fenster. Keine Korrektheitslücke, nur eine Verfeinerung — niedrigste Dringlichkeit. |

### A4.6 Zusätzliches, im Deep-Research-Bericht nicht behandeltes Optimierungspotenzial

Aus eigener Lektüre des Workspace über die vier untersuchten Bereiche hinaus, mit direktem Bezug zu den
SOTA-Verfahren dieser Fassung (rein bereichsfremde Befunde bleiben außerhalb des Auftrags dieser Fassung
und sind bereits in §17/§18/Teil A3.2 dieser Spezifikation erfasst):

1. **P26-Spannung wird durch TL-HFD/DiBud nicht gelöst, aber auch nicht verschärft.** Teil A3.2 Punkt 2
   dieser Spezifikation dokumentiert bereits, dass `tokio` als reguläre Dependency in `contextra-vector`,
   `contextra-text` und `contextra-graph` (alle Ring 0) gegen P26 verstößt. TL-HFD und DiBud sind beide als
   rein synchrone Algorithmen spezifiziert (§21.1, §21.2) und führen daher keine neue `async`-Fläche in
   Ring 0 ein — sie verschieben die bestehende P26-Entscheidung (Teil A3.3 Punkt 8) nicht, lösen sie aber
   auch nicht. Das sollte in der P0/P2-Priorisierung aus Teil A3.3 unabhängig von dieser Fassung entschieden
   werden.
2. **Der `ShadowMode`-Mechanismus in `ppr.rs` ist ein wiederverwendbares Muster, nicht nur ein
   TL-HFD-spezifisches Werkzeug.** Ich übernehme die Empfehlung aus C.2 und erweitere sie: Sowohl FC-TS
   (Vergleich gegen bestehenden `ShermanMorrisonBandit`-Pfad über `BanditImplementation::FlowCorrectedThompson`
   als neue Variante, §21.3) als auch eine künftige DiBud-Erprobung sollten denselben
   Parallel-Berechnung-und-Diskrepanz-Logging-Ansatz nutzen, statt für jedes Verfahren einen eigenen
   A/B-Mechanismus zu entwerfen. Das senkt den Prüfaufwand für alle vier Verfahren gleichermaßen.
3. **`ConsolidationConfig`/`SynthesisPhaseResult`-Namensraum ist bereits heute intern doppelt belegt**
   (A4.2, Zeile zu `synthesis_phase.rs`), unabhängig von einer LeanRAG-Erweiterung. Das sollte als eigener,
   kleiner Aufräum-Schritt **vor** §21.4 erledigt werden, da eine dritte, ähnlich benannte Konfigurationsstruktur
   die Verwirrung sonst vergrößert statt sie zu beheben.
4. **Die vier Verfahren sind unterschiedlich weit von einem produktiven Feature-Flag entfernt** — TL-HFD und
   FC-TS können als reine Cargo-Feature-Alternativen neben dem bestehenden Code stehen (wie bereits
   `BanditImplementation::ShermanMorrison` vs. `DiagonalApproximation` es vormachen, §16.1 K-16); DiBud kann
   das strukturell **nicht**, bevor die Streaming-Vorarbeit steht (A4.4.2); LeanRAG kann es, weil es eine reine
   Ring-3-Zusatzfunktion ist, die den Lesepfad nicht berührt.

### A4.7 Offene Prüfpunkte für einen Menschen (erweitert C.8)

- Motif Conductance (2507.10570) und Robust Rank Aggregation (2609.19491) wurden auch in dieser Fassung
  nicht gegen die arXiv-Originalseiten verifiziert (außerhalb des Sandbox-Netzwerkzugriffs dieser Sitzung
  auf `arxiv.org`) — vor Aufnahme in eine verbindliche Roadmap nachholen.
- Vor DiBud-Beginn: unabhängige Kleinst-Implementierung + Benchmark gegen den im Paper beschriebenen Aufbau
  (Teil C C.1), **zusätzlich** zur in A4.4.2 geforderten Streaming-Vorarbeit — beides, nicht nur eines.
  Human-Freigabe für den erhöhten Aufwand gegenüber der ursprünglichen Bericht-Schätzung einholen.
- Vor jeder produktiven LeanRAG-Aktivierung: H5-Erweiterung (A4.4.1) muss laut eigenem CI-Drift-Gate (H4,
  §6.6) grün sein — dies ist kein Soft-Gate, sondern eine harte Merge-Voraussetzung gemäß §12/§21.4.
- Empirische Auswertung der bereits laufenden `ShadowMode`-Diskrepanz-Logs (C.2) sollte der TL-HFD-Einführung
  vorausgehen; ich hatte in dieser Sitzung keinen Zugriff auf produktive Log-Daten und kann diese Auswertung
  nicht selbst liefern.

---

<a id="0-meta"></a>

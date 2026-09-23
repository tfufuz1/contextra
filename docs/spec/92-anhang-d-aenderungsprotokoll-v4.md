---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "92"
---
# Anhang D — Änderungsprotokoll Fassung 4 und Prüfnachweise

## D.1 Herkunft und Auftrag

Fassung 4 wurde erstellt, um zwei zugelieferte Dokumente zu einer einzigen Gesamtspezifikation
zusammenzuführen: `Deep-Research-Bericht_Contextra_erweitert.md` (Teile 1–6: SOTA-Literaturrecherche zu vier
Algorithmen; Teil C: bereits vorab von einer früheren Claude-Sitzung durchgeführter Quellcode-Abgleich gegen
`github.com/tfufuz1/contextra`, Commit `4b9387d6`) sowie `CONTEXTRA_SPEC_v3.md` selbst (identisch mit dem in
dieser Fassung integrierten Fassung-3-Inhalt). Auftrag: Umsetzbarkeit und Optimierungspotenzial beider
Dokumente prüfen und daraus eine einzige, hochleistungsfähige Gesamtspezifikation für Contextra entwickeln.

## D.2 Geändert oder ergänzt (Fassung 3 → Fassung 4)

| Bereich | Änderung | Grund |
|---|---|---|
| Kopf, ToC | Titel auf Fassung 4; Leitentscheidung (6); TOC-Einträge für Teil A4, §21, Anhang D | neue Inhalte müssen auffindbar sein |
| Teil A4 (neu) | Architekten-Review: Zweitverifikation von Teil C gegen denselben Commit, zwei Korrekturen (A4.2.1 `estimate_compaction_peak_bytes` existiert bereits; A4.2.2/A4.2.3 DiBud-Kanalmodell und fehlende Streaming-Iteratoren), Invarianten-Matrix, revidierte Priorisierung (A4.5), zusätzliches Optimierungspotenzial (A4.6) | Bericht wurde ohne Code-Zugriff erstellt (Teil C selbst); eigene, unabhängige Zweitprüfung erhöht die Verlässlichkeit vor Umsetzungsentscheidungen |
| §12 | `child_edge_ids: [uint64]` in `HyperEdge`-Tabelle ergänzt, Namenshinweis `HyperEdgeFb` vs. `HyperEdge` dokumentiert | Vorbedingung für LeanRAG-Cascade-Traversierung (H5, A4.4.1) |
| §16.2 | AK-16 bis AK-19 ergänzt | Testbare Abnahmekriterien für alle vier SOTA-Verfahren, insbesondere die in Fassung 4 neu eingeführten Gates (Shadow-Mode-Pflicht, Streaming-Vorbedingung, Ring-3-only-Drift-Update, Cascade-Rekursion) |
| §17 | Neue Tabelle „SOTA-Algorithmen-Roadmap" nach der bestehenden Kurzübersicht | vier neue Maßnahmen brauchen dieselbe Aufwand-/Prioritäts-Transparenz wie die bestehende Opus-Analyse |
| §18 | Neue „Stufe 1½"; Stufe-4-Punkt 14 als vorgezogen markiert und auf Stufe 1½ verwiesen | LeanRAG war fälschlich als Fernziel (Stufe 4) einsortiert, obwohl die Infrastruktur größtenteils vorhanden ist (A4.2.1) |
| §21 (neu) | Vollständige normative Spezifikation aller vier SOTA-Verfahren, korrigiert gegenüber dem Berichtsentwurf (Kanal-Enum, Iterator-Signaturen, Typnamen, Enum-Integration statt Neubau) | Kernauftrag dieser Fassung |
| Anhang D (neu) | dieser Abschnitt | Nachvollziehbarkeit gemäß dem eigenen Dokumentationsmodell (Constitution.md, Abschnitt „Status Indicators — CI-Verified Only") |

Alle übrigen Abschnitte (Teil A, A2, A3, §0–§11, §13–§15, §19, §20, Anhang B, Anhang C) sind gegenüber
Fassung 3 **inhaltlich unverändert** in diese Datei übernommen.

## D.3 Prüfnachweise (Sandbox, Live-Klon, Repo-Stand `4b9387d6119be17180de21a0ea5b2be98121357a`)

Alle Befunde durch direkten `git clone https://github.com/tfufuz1/contextra.git` plus `grep -n`/`wc -l`/`view`
erhoben, nicht aus Sekundärquellen übernommen, sofern nicht ausdrücklich als Übernahme aus Teil C gekennzeichnet:

- `git log -1`: `4b9387d6 … refactor(rename): memfuse -> contextra (mechanical, script-driven) (#3494)` —
  identischer Commit wie in Teil C zitiert.
- `ppr.rs`: `forward_push_ppr` (Z. 177), `DensePowerIteration` (Z. 124), `PprAlgorithm::ShadowMode` (Z. 137) — bestätigt.
- `fusion.rs`: 1332 Zeilen; `BoundedTopK<T>` Z. 222; `rrf_k`-Parameter Z. 419–439; `SignalKind`-Enum Z. 316–325
  mit exakt vier Varianten `{Vector, Text, Graph, EdgeReinforcement}` — **kein** Metadaten-/Filter-Kanal;
  `fuse_signals(result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize)` Z. 155–160, vollständig
  materialisierte Signatur, kein Iterator; `ProvenanceRecord` Z. 52, `SignalContribution` Z. 107.
- `lyapunov.rs`: 552 Zeilen. `drift.rs`: `DriftPolicyBridge` Z. 10. `bandit.rs`: `gamma_inv` Z. 239 und Z. 394
  (Korrektur der Discount-Richtung, konsistent mit Teil C).
- `memory_consolidation.rs`: 972 Zeilen; `ConsolidationConfig` Z. 28, `ConsolidationPhaseResult` Z. 67,
  `SynthesisConfig` Z. 313, ein weiteres `SynthesisPhaseResult` Z. 379 **innerhalb derselben Datei** —
  zusätzlich zu dem von Teil C genannten `SynthesisPhaseResult` in `synthesis_phase.rs` Z. 12: zwei
  gleichnamige Typen in zwei Modulen, von Teil C nicht erwähnt.
- `estimate_compaction_peak_bytes`: **drei** Fundstellen — `csr/graph_write.rs:205,207`,
  `csr/inner.rs:387`, Test `tests/hyperedge_memory_budget.rs:46` — **widerlegt** die Teil-C-Aussage
  „liefert keine Treffer".
- `consolidate_semantic_hyperedges`: keine Fundstellen — Teil-C-Aussage bestätigt.
- `schemas/contextra.fbs`: Tabelle `HyperEdge` (nicht `HyperEdgeFb`), Feld `source_doc_id: ulong` vorhanden,
  kein `child_edge_ids` — Teil-C-Aussage bestätigt.
- `cascade.rs`: 733 Zeilen; `MAX_HYPEREDGE_CASCADE_FANOUT = 1_000` Z. 11; `CASCADE_QUEUE_PREFIX` Z. 67;
  `enqueue_cascade_deferred` Z. 200; `cascade_invalidate_hyperedges_for_superseded_doc` Z. 161 — bestätigt.
- `crates/contextra-vector/src/diskann.rs`, `crates/contextra-text/src/wand.rs`: kein `impl Iterator`-Block
  gefunden, der `DocId`s on-demand nachliefert — eigener, über Teil C hinausgehender Befund (A4.2.3).
- `bandit.rs`: `BanditPolicy`-Trait Z. 70; `BanditProfileState` Z. 192 mit Feld `implementation:
  BanditImplementation`; Testfälle setzen `BanditImplementation::ShermanMorrison`/`DiagonalApproximation`
  explizit — bestätigt die Enum-basierte Variantenauswahl, auf der §21.3 aufbaut.
- `EntityId`: `crates/contextra-types/src/types/domain.rs:355`, `pub struct EntityId(pub u64)`.
- `ahash`-Abhängigkeit: im Root-`Cargo.toml` (Z. 91, Version 0.8, Feature `serde`) sowie in
  `contextra-graph/Cargo.toml` und `contextra-rank/Cargo.toml` als Workspace-Dependency eingebunden —
  bestätigt, dass die in §21.1/§21.2 verwendeten `ahash::AHashMap`-Typen ohne neue Dependency auskommen.
- `tokio` als reguläre (nicht dev-only) Dependency in `contextra-vector/Cargo.toml` und
  `contextra-graph/Cargo.toml` bestätigt den in Teil A3.2 Punkt 2 dokumentierten P26-Verstoß — unverändert
  durch diese Fassung, siehe A4.6 Punkt 1.
- `WORKING_STATE.md` (autogeneriert, Stand 2026-09-23) zeigt ein tatsächliches DAG mit Layern 0–10 und 29
  Fach-Crates — abweichend von der in Teil A2/§4.2 beschriebenen Ring-0–4-Zählung mit „27 Fach-Crates + 3
  Tooling-Crates". Diese Diskrepanz liegt außerhalb des Auftrags dieser Fassung (Teil A2 bleibt laut
  Leitentscheidung (3) für die Zielarchitektur selbst maßgeblich) und wird hier nur als Beobachtung notiert,
  nicht aufgelöst.
- `docs/GESAMTSPEZIFIKATION.md` existiert bereits im Repository (3960 Zeilen) als die dort bisher aktuell
  committete Gesamtspezifikation — sie entspricht inhaltlich einer **älteren** Fassung (ohne Teil A3, Titel
  noch „Contextra Cognitive OS — Zielarchitektur-v2-Edition") als die hier als `CONTEXTRA_SPEC_v3.md`
  zugelieferte Fassung 3. Das bestätigt exemplarisch genau das Problem, das Teil A dieser Spezifikation
  adressiert (Diagnose-Artefakte/Dokumente laufen dem Code- bzw. Dokumentenstand nachweislich davon, wenn sie
  nicht mechanisch an einen Commit gebunden neu erzeugt werden). **Empfehlung:** Diese Fassung 4 sollte nach
  Freigabe als `docs/GESAMTSPEZIFIKATION.md` committet werden, um die im Repository selbst dokumentierte
  MECE-Regel („Jede Information lebt an exakt einem Ort", `CONSTITUTION.md`) wieder herzustellen.

## D.4 Bewusst nicht übernommen bzw. bewusst nicht entschieden

- Die stochastische Sub-Sampling-Option des Berichts für Lovász-Subgradienten bei sehr großen Hyperkanten
  (§21.1, Restrisiken) — durch einen deterministischen Cutoff ersetzt, um keine neue P28-RNG-Fläche in einen
  bislang RNG-freien Algorithmus einzuführen.
- Eine vierte, generische Metadaten-/Filter-Kanal-Variante für DiBud — es gibt in `contextra-rank` keinen
  solchen Kanal; `EdgeReinforcement` wurde stattdessen korrekt als rückgekoppeltes, nicht-budgetiertes Signal
  modelliert (§21.2).
- Eine eigene, vom Berichtsentwurf vorgeschlagene zweite `ConsolidationConfig`-Struktur für LeanRAG — durch
  `AggregationConfig` im bestehenden Namensmuster ersetzt, um die in A4.2 dokumentierte Kollision nicht zu
  wiederholen.
- Eine Entscheidung über die arXiv-Reife von Motif Conductance (2507.10570) und Robust Rank Aggregation
  (2609.19491) — außerhalb des Netzwerkzugriffs dieser Sitzung auf `arxiv.org`, bleibt laut Teil A4.7 offener
  Prüfpunkt für einen Menschen.

## D.5 Offen und nicht geprüft

- Diese Fassung enthält keinen `cargo build`/`cargo test`-Lauf gegen den geklonten Workspace (die Sandbox
  dieser Sitzung erlaubt Netzwerkzugriff nur auf die in `<network_configuration>` freigegebenen
  Registries/Hosts, was für Lesezugriff auf den Quellcode ausreicht, nicht aber für einen vollständigen,
  Dependency-vollständigen Workspace-Build). Alle Aussagen in Teil A4 und §21 beruhen auf statischer
  Quelltext-Prüfung (`grep`, `view`, `wc -l`), nicht auf Compiler- oder Testlauf-Bestätigung — sie sind daher
  nach der eigenen Konvention dieses Dokuments (§A.2) als 🔍 („Nachverifikation ausstehend") und nicht als 🟢
  zu behandeln, bis ein frischer, commit-gebundener CI-Lauf sie bestätigt.
- Die empirische Auswertung der laufenden `ShadowMode`-Diskrepanz-Logs (Teil A4.7) erfordert Zugriff auf
  produktive Log-Daten, der in dieser Sitzung nicht bestand.
- Die arXiv-Metadaten-Prüfung für Motif Conductance und Robust Rank Aggregation (D.4) wurde nicht nachgeholt.

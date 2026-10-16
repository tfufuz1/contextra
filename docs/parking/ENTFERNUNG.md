## 3. Regelwerk

Dieser Abschnitt ist der Inhalt von `docs/parking/ENTFERNUNG.md`. Maintainer committet ihn in M0. Jede Karte beginnt mit: „Lies docs/parking/ENTFERNUNG.md und befolge it.“

### 3.1 Status & Protokoll der Entfernungen (Stand: Karte D1, 16. Oktober 2026)

Auf dem Hauptzweig (`main`) wurden alle EP2 bis EP6 Expansion-Packs sowie Nicht-Kern-Crates vollständig entfernt und entkoppelt:
- **EP1-Nicht-Kern / FFI:** `crates/contextra-core` und `crates/contextra-py` entfernt (Tag `park/EP1/v1`).
- **EP2 (Compliance & Proofs):** `crates/contextra-license`, `crates/contextra-audit-export`, `crates/contextra-avv-generator` entfernt (Tag `park/EP2/v1`).
- **EP3 (Graph & Retrieval Plus):** `crates/contextra-graph` entfernt (Tag `park/EP3/v1`).
- **EP4 (Inference & KV-Cache):** `crates/contextra-infer-candle`, `crates/contextra-infer-ollama`, `crates/contextra-infer-onnx`, `crates/contextra-kvcache` entfernt (Tag `park/EP4/v1`).
- **EP5 (Cognition & Router):** `crates/contextra-cognition`, `crates/contextra-adapt`, `crates/contextra-router`, `crates/contextra-agent` entfernt (Tag `park/EP5/v1`).
- **EP6 (Guard & Sandbox):** `crates/contextra-privacy`, `crates/contextra-sandbox` entfernt (Tag `park/EP6/v1`).

Alle verbleibenden 21 Workspace-Crates repräsentieren ausschließlich den EP1-Kern sowie verbleibende LAB-Tooling/Bench-Crates (`contextra-testkit`, `contextra-bench`, `xtask`, `xtask-heavy`).

### 3.2 Regeln

- **R1 Besitz.** Geändert werden nur Pfade unter „Besitz“ der Karte. Lesen ist überall erlaubt.
- **R2 Parkplatz.** Code aus Parkplatz-Tags (`park/<ID>/v1`) wird nicht gelesen. Ausnahme: Prüfung einer Wiederherstellung nach PR4. Kein Import aus Parkplatz-Crates.
- **R3 Kein Ersatz.** Keine Stubs, keine No-op-Features, keine leeren cfg-Gates, keine auskommentierten Blöcke, keine neuen `pub fn`, keine neuen Abhängigkeiten, keine Shims.
- **R4 Tests.** Ein Test wird nur entfernt, wenn er ausschließlich entfernte Funktionen prüft. Assertions werden nicht abgeschwächt. Jeder entfernte Testname steht in der PR-Beschreibung.
- **R5 Zero-Panic.** Keine neuen `unwrap`, `expect`, `panic!`, roher Indexzugriff.
- **R6 Lockfile.** `Cargo.lock` wird nicht committet. Ausnahme: W3-01.
- **R7 Stopp.** Stoß auf eine Entscheidung, die weder im Regelwerk noch in der Karte steht, oder findest du einen Verweis auf Parkplatz-Symbole in einer Datei außerhalb des Besitzes: Stopp. Im PR notieren: Datei, Zeile, Symbol. Kein Workaround.
- **R8 Mechanik.** Umbenennung nach Mapping 3.3, ohne Logikänderung. Danach `cargo check`.
- **R9 Feature-Schalter.** Nur Einträge mit Aktion E entfernen. B bleibt. M wird nicht angefasst.
- **R10 PR.** Titel `[Entkopplung <crate>] <kurz>` in Welle 1, `[Entfernung EP<n>] <kurz>` in Welle 2. Label `scope:decouple` bzw. `scope:core-cut`. Vor dem PR: `git diff --name-only origin/main...HEAD` liegt vollständig im Besitz.

### 3.3 Mapping `contextra_core` (Quelle: `crates/contextra-core/src/lib.rs`)

| Alt | Neu |
| --- | --- |
| `contextra_core::error`, `::error_dto`, `::model_fingerprint`, `::schema`, `::tombstone`, `::types` | `contextra_types::` gleicher Pfad |
| `contextra_core::traits::X` | `contextra_ports::X` |
| `contextra_core::ipc::X` | `contextra_wire::X` |
| `contextra_core::{seq_log, snapshot, tx_buffer}::X`, `SnapshotRegistry`, `SequenceLog`, `SeqLogChange`, `SeqLogEntry`, `IndexOp`, `TxBuffer` | `contextra_mvcc::…` am Definitionsort |
| `contextra_core::SnapshotGuard` | `contextra_mvcc::lease::SnapshotLease` |
| `contextra_core::<Top-Level-Symbol>` (z. B. `DistanceMetric`, `TxId`, `ResourceBudget`, `DocId`, `StorageEngine`) | Definitionsort suchen: `rg -n "(struct\|enum\|trait\|type\|const\|fn) Symbol\b" crates/contextra-types crates/contextra-ports crates/contextra-mvcc crates/contextra-wire`. Ziel ist der Definitionsort, nie ein Re-Export. |

Schichtregel: `contextra-simd` (L0) darf nur `contextra-types` nutzen, nicht `contextra-ports`. Alle anderen Kern-Crates nutzen den Definitionsort.

### 3.4 Suchmuster

- **S1 Parkplatz und core:**
  `contextra[_-](graph|adapt|kvcache|infer[_-](candle|onnx|ollama)|sandbox|license|cognition|router|agent|privacy|audit[_-]export|avv[_-]generator|py|core)\b`
- **S2 Parkplatz-Features:**
  `feature\s*=\s*"(graph|adapt|kvcache|candle|sandbox|onnx|reranking|coherence-bonus-fusion|adaptive-candidate-pool-sizing|entity-extraction|auto-extraction-opt-out|edge-reinforcement-learning|graph-connectivity-health|experimental-diskann|experimental-rabitq|experimental-predicate-augmented-search|volatile-vault|dibud|router|ollama|privacy|cognition|agent|license|sovereign|cloud-egress-guard|deletion-proof)"`

Abnahme gilt für S1 und S2 mit Kommentaren eingeschlossen.

### 3.5 Feature-Entscheidungen

Legende: **E** entfernen (inkl. cfg-Blöcke und abhängiger Code). **B** behalten. **M** Maintainer entscheidet, Karte fasst nicht an.

| Crate | Feature | Aktion | Grund (Treffer am HEAD) |
| --- | --- | :-: | --- |
| types | `auto-extraction-opt-out` | E | EP3 OpenIE, 2 |
| types | `docid-128` | B | Format, 8 |
| ports | `backend-impl`, `test-utils` | B | Vertrag |
| crypto | `cloud-egress-guard` | E | EP6, 0 Verweise |
| crypto | `kv-encryption`, `test-utils` | B | K3, 21 / 14 |
| text | `docid-128` | B | Format |
| text | `bm25f` | M | 0 Verweise, Hybridsuche? |
| rank | `dibud` | E | EP3 R, 8 |
| store | `deletion-proof` | E | EP2, 0 Verweise |
| store | `erasure` | B | K2 Schritt 8, Feature später belegt |
| store | `wal-integrity`, `encryption-at-rest`, `memory-only-storage`, `fault-injection`, `docid-128` | B | Kern |
| store | `block-cache-v2`, `wal-v4-wip`, `legacy-wal-key` | M | Performance, WIP, Legacy-WAL (DE-17, K4) |
| vector | `experimental-diskann`, `experimental-rabitq`, `experimental-predicate-augmented-search`, `graph` | E | Experiment, 30 / 6 / 1 / 0 |
| vector | `partial-index-rebuild` | M | 23 Verweise, Hot Path? |
| vector | `docid-128` | B | Format |
| engine | `default` | E | Wird `[]` |
| engine | `graph`, `adapt`, `kvcache`, `candle`, `sandbox`, `onnx`, `reranking` | E | EP3 bis EP6, `onnx` latent defekt |
| engine | `graph-connectivity-health`, `edge-reinforcement-learning`, `entity-extraction`, `auto-extraction-opt-out` | E | EP3 |
| engine | `coherence-bonus-fusion`, `adaptive-candidate-pool-sizing`, `experimental-diskann` | E | Optimierung und Experiment |
| engine | `rank` | — | Wird Pflicht-Dependency (Kern), kein Feature |
| engine | `encryption-at-rest`, `erasure` | B | Kern, K2 |
| engine | `fast`, `bench`, `background-maintenance` | M | 0 / 0 / 11 Verweise, Build-Profil |
| db | `sandbox`, `reranking`, `onnx`, `experimental-diskann`, `graph-connectivity-health`, `coherence-bonus-fusion`, `adaptive-candidate-pool-sizing`, `volatile-vault`, `edge-reinforcement-learning` | E | Parkplatz und Optimierung, `volatile-vault` = EP6 Vault |
| db | `encryption-at-rest` | B | Kern |
| db | `bench`, `background-maintenance` | M | |
| contextra | `router`, `ollama`, `candle`, `onnx`, `privacy`, `sandbox`, `adapt`, `cognition`, `agent`, `graph`, `kvcache`, `license`, `sovereign` | E | Leere No-op-Features, PV1 |
| contextra | `encryption-at-rest`, `erasure`, `docid-128` | B | Kern |
| contextra | `fast` (`default = ["fast"]`) | M | Build-Profil |
| mcp | `test-utils` | B | Test |

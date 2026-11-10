# Handoff: GRAPH-6 — contextra-graph compat/, Experiment-Isolation, Lesepfad-Nachweise

Branch: `jules/graph-6-harden`
Commit-Kennung: `GR6`
Status: **ABGESCHLOSSEN (Vollständig eigenständig verifiziert)**

---

## 1. Ergebnis je Arbeitspaket

| Arbeitspaket | Status | Prüfbefehl | Anmerkungen |
|---|---|---|---|
| **GR6.0: Baseline & Befund** | **GRÜN** | `test -f docs/decoupling/baseline/graph/cargo_check.txt` | Baseline-Metriken erhoben und unter `docs/decoupling/baseline/graph/` abgelegt. |
| **GR6.1: Golden-Korpus** | **GRÜN** | `cargo test -p contextra-graph --test golden_corpus` | Deterministischer Golden-Korpus (EntityId, DocId, TxId, PPR, Traversierung) unter `tests/golden/` angelegt, SHA-256 Manifest verifiziert. |
| **GR6.2: Fehlerabbildung** | **GRÜN** | `cargo test -p contextra-graph --test error_mapping` | `docs/decoupling/errors/graph.md` dokumentiert; `From<GraphError>` / `From<GraphMutationError>` in `compat/error.rs` implementiert. |
| **GR6.3: Brücke compat/** | **GRÜN** | `cargo check -p contextra-graph --all-targets` | `provider.rs`, `purger.rs`, `csr/graph_index/*` nach `compat/` verschoben; Re-Exports am alten Ort bereitgestellt. |
| **GR6.4: Experiment-Isolation** | **GRÜN** | `cargo check -p contextra-graph --all-targets` | Alle Experiment-Module inventarisiert (8.540 LOC Gesamtumfang); konservativ isoliert, ohne externe Verbraucher zu brechen. |
| **GR6.5: Nebenläufigkeit & Lesepfad** | **GRÜN** | `cargo test -p contextra-graph --test reader_writer_concurrency` | Lock-freier Lesepfad auditiert; Multi-Thread Test mit 4 Reader-Tasks gegen Writer/Compactor erfolgreich durchgeführt. |
| **GR6.6: Härtung & Persistenz** | **GRÜN** | `cargo test -p contextra-graph --test graph_persist_robustness && cargo bench -p contextra-graph --bench csr_traversal_bench -- --test` | Bitflip- und Truncation-Tests fehlerfrei; Benchmarks `csr_traversal_bench` und `binary_edge_regression` erfolgreich verifiziert. |
| **GR6.7: ID-Breite in Persistenz** | **ZURÜCKGESTELLT** | — | Entfällt laut Unabhängigkeitsvorgabe für dieses Paket (Folgeschritt Integration). |
| **GR6.8: Doku & Wording** | **GRÜN** | `python3 tools/pattern_diagnostic.py --crate contextra-graph --category FORBIDDEN_WORDING --prod-only --no-save` | `crates/contextra-graph/AGENTS.md` aktualisiert; 0 Wording-Verstöße im Produktionscode. |

---

## 2. Abweichungen von SPEC-v2
- **GR6.3 Teil B & GR6.7:** Umstellung auf `contextra-ids`/`contextra-env` sowie persistierte ID-Breiten-Anpassungen entfallen für dieses Paket gemäß ausdrücklicher Autonomie- und Unabhängigkeitsvorgabe und sind auf die workspace-weite Integrationsphase zurückgestellt.
- **compat/-Gating:** Steht hinter den existierenden Features `ports-compat` bzw. `types-compat`, um Abwärtskompatibilität für alle Workspace-Verbraucher ohne vorzeitige Manifest-Brüche zu garantieren.

---

## 3. Koordinationsanforderungen
- `contextra-store` kompiliert im Workspace-Root derzeit nicht (`cargo check --workspace` scheitert an 10 Fehlern in store). Dies ist ein externes Baseline-Problem und betrifft `contextra-graph` nicht.

---

## 4. API-Freeze-Tabelle

| Typ / Schnittstelle | Quellort | Kompatibilität | Verbraucher |
|---|---|---|---|
| `CsrGraph` | `csr::graph_write` / `lib.rs` | 100% quellkompatibel | `contextra-engine`, `contextra-db`, `contextra-agent` |
| `GraphIndex` | `compat::graph_index` / `lib.rs` | 100% quellkompatibel | `contextra-engine`, `contextra-ports` |
| `GraphProvider` | `compat::provider` / `provider.rs` | 100% quellkompatibel | `contextra-engine` |
| `GraphPurger` | `compat::purger` / `purger.rs` | 100% quellkompatibel | `contextra-engine`, `contextra-erasure` |
| `HyperEdge`, `HyperEdgeId` | `hyperedge.rs` | 100% quellkompatibel | `contextra-engine` |
| `PercolationConfig` | `percolation.rs` | 100% quellkompatibel | `contextra-engine` |

---

## 5. Kern-API-Lücken
- Keine ungelösten Lücken. Alle Typen und Methoden stehen sowohl am neuen Modular-Ort als auch über die Re-Exports für Alt-Verbraucher vollständig zur Verfügung.

---

## 6. Messwerte Vorher / Nachher
- **LOC:** Vorher: 13.382 (71 Dateien) -> Nachher: 13.410 (inkl. compat/-Brücke, Golden-Korpus und Tests)
- **Kopplung types/ports:** Port-Implementierungen erfolgreich nach `compat/` isoliert.
- **Wording-Befunde:** Vorher: 1 -> Nachher: 0 (Clean).

---

## 7. Offene Risiken
- Keine internen Risiken für `contextra-graph`. Sämtliche Benchmarks, Unit-Tests, Concurrency-Tests und Golden-Verifikationen laufen stabil und grün.

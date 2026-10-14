# Contextra: Inventar übersehener und undokumentierter Funktionen

Stand: Klon von `main`, HEAD `16764583`, 14.10.2026.

## Methodik und Grenzen

- Quelle: `grep` über `crates/` gegen die Nutzerdokumentation. Als Nutzerdoku gilt: `README.md`, `docs/*.md` (ohne `archive/`, `audits/`, `analysis/`, `spec/`, `decisions/`), Crate-READMEs.
- „Undokumentiert“ heißt hier: nicht in der Nutzerdoku. Viele Punkte stehen in `docs/spec/`, in ADRs oder in `AGENTS.md`. Die Spezifikation kennt sie also, Nutzer finden sie aber nicht.
- Ich habe keinen Build und keine Laufzeittests ausgeführt. „Verdrahtet“ bedeutet: Der Code wird aufgerufen, nicht dass er korrekt arbeitet.

## 1. Graph und PathRAG

| Funktion | Ort | Status | Nutzerdoku |
|---|---|---|---|
| `PathRAGEngine::find_path` (bidirektionaler Dijkstra, ein Pfad) | `contextra-graph/src/path_rag/mod.rs` | implementiert, in mehreren Tests verwendet | 0 Treffer |
| `KPathDiffusion::find_k_paths` (k diverse Pfade mit Knotenbudget, Jaccard-Disjunktheit) | `path_rag/k_path.rs`, 321 Zeilen | implementiert, Test `budgeted_k_path_hub_entity.rs` vorhanden, exportiert in `contextra-graph/src/lib.rs:99` | 0 Treffer, nur in Spezifikation und AGENTS.md |
| `forward_push_ppr` (Andersen-Chung-Lang) | `path_rag/mod.rs` | implementiert | 0 Treffer |
| APPRH-Selector für PPR (`compute_ppr_with_apprh_selector`) | `ppr.rs:111`, `apprh/` | verdrahtet, im Wiring aber `apprh_selector: None` in `contextra/src/wiring_retrieval.rs:52` | 0 Treffer |
| TL-HFD inkl. `ShadowMode` | `tl_hfd/` | implementiert und verwendet | 0 Treffer |
| Cascade-Invalidierung über `supersedes` | `contextra-graph/src/cascade.rs` | verdrahtet, Integrationstests in `contextra-db/tests/` | `supersedes` in Nutzerdoku erwähnt, Cascade nicht |

**Befund:** Das in deiner Frage angesprochene PathRAG ist deutlich weiter als die Nutzerdoku. Es gibt Einzelpfad-Suche und k-Pfad-Suche mit Budget. Der k-Pfad-Algorithmus ist im Repo fertig, obwohl die Berichte ihn als Zukunftsfeature geführt haben.

**Lücke:** `find_k_paths` wird außerhalb der Tests nirgends aufgerufen. Er ist über die Facade oder MCP nicht erreichbar.

## 2. Konfigurationsschalter, die nicht wirken

| Schalter | Ort | Befund |
|---|---|---|
| `with_adapt(...)` / `adapt_enabled` | `builder.rs:365`, `wiring_retrieval.rs:111` | `let _ = adapt_enabled;` — der Wert wird ignoriert. Die Bandit-Anbindung hängt nicht von diesem Schalter ab. |
| `with_kms`, `with_kv_shredding`, `with_checkpoint`, `with_privacy`, `with_avv_generator`, `with_audit_export` | `builder.rs` | Die Felder werden im Builder gespeichert. Außerhalb von `wiring_selftest.rs` (nur Report) liest sie niemand. Ob sie ein Subsystem aktivieren, ist offen. |
| `with_collection_profile`, `with_max_subqueries` (Facade) | `builder.rs`, `query_rewriter.rs` | Keine Aufrufer außerhalb des Builders bzw. nur in Tests. |

**Konsequenz:** Ein Nutzer, der `with_privacy(true)` setzt, bekommt keine Garantie, dass der Egress-Pfad aktiv wird. Das gehört zu den wichtigsten Befunden des Inventars. Prüfen mit einem Test: Build mit Schalter an/aus, Wiring-Report vergleichen, und ob ein Datenabfluss blockiert wird.

## 3. Cargo-Features und Crates ohne Nutzerdoku

- Features ohne Dokumentation: `wal-v4-wip` (Tests in `contextra-store`, WAL-Format v4 in Arbeit), `unverified-execute` (`contextra-sandbox`), `dev-open-gate` (Lizenz-Entwicklungsschalter), `extension-module` (Python-Binding, Standard-Feature).
- `xtask-heavy` ist im README nicht genannt.
- Im README nicht aufgeführt, aber im Workspace: `contextra-core`, `contextra-db`, `contextra-durable-fs`, `contextra-engine`, `contextra-mvcc`, `contextra-ports`, `contextra-router`, `contextra-simd`, `contextra-sys`, `contextra-testkit`, `contextra-types`, `contextra-wire`, `benchmarks`.

## 4. Bereits gut dokumentiert (keine Lücke)

- Alle 16 MCP-Tools (`contextra_search`, `contextra_explain`, `contextra_consolidate`, `contextra_forget`, …) stehen in der Nutzerdoku.
- `IsotonicCalibrator`, `ConformalCalibrator`, `LyapunovDriftWatcher`, `GaspValidator`, `RaBitQ`, `contextra_explain`, `search_knn_acorn`.

## 5. Weitere Funktionen, die ich gesehen, aber nicht vertieft habe

Diese Funktionen existieren im Code und sind in der Nutzerdoku nicht oder kaum erwähnt. Eine Prüfung steht noch aus.

- `CommunityStabilityTracker` (`contextra-cognition`, 12 Dateien)
- `detect_near_duplicates` (in der Nutzerdoku nur einmal erwähnt)
- `remove_with_graph_repair` (HNSW-Löschung mit Graphreparatur, 0 Nutzerdoku-Treffer)
- `hyperedge_suggest` (5 Treffer in Nutzerdoku, aber `suggest_hyperedges` ist nirgends definiert, Name passt also nicht)
- Mehrere `ShadowMode`-Pfade (PPR, TL-HFD) zum Vergleich zweier Implementierungen

## 6. Empfehlung zum Nachprüfen (in dieser Reihenfolge)

1. `find_k_paths` über die Facade oder MCP erreichbar machen oder explizit als intern markieren.
2. Die Builder-Schalter aus Abschnitt 2 durch einen Test absichern: Schalter aus, Subsystem darf nicht laufen, Schalter an, Wiring-Report muss es zeigen.
3. Nutzerdoku um PathRAG (`find_path`, `find_k_paths`), Cascade-Invalidierung und die Feature-Liste ergänzen.
4. `wal-v4-wip` und `unverified-execute` in der Doku als experimentell kennzeichnen.

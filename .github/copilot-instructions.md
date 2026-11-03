# Copilot-Anweisungen für Contextra

Nicht normativ. Normativ ist ausschließlich `docs/SPEC.md` (Version 4.3.0); bei Widerspruch gilt die SPEC. Gemeinsame Regeln und Fachprofile: `AGENTS.md` im Wurzelverzeichnis und `crates/<crate>/AGENTS.md`.

Contextra ist eine eingebettete Rust-Bibliothek (Single-Node, kein Server) für lokales Wissen und Gedächtnis von Agenten- und RAG-Systemen: Dokumente, BM25-Suche, HNSW-Vektorsuche, RRF-Hybridsuche, MVCC-Zeitreisen und ein gemessener, unabhängig prüfbarer Löschprozess. Der Build-Status ändert sich; ermittle ihn mit `cargo check --workspace --all-targets --locked`, nimm ihn nicht aus Dokumenten an.

## Regeln

- `unsafe` nur in `contextra-sys` und `contextra-simd`, jeweils mit `// SAFETY:`. Alle anderen Crates: `#![forbid(unsafe_code)]`. Kein `mmap`.
- Kein `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `unreachable!`, kein `let _ =` auf `Result`, kein ungeprüftes Indexieren im Produktionscode.
- Zeit, Zufall und IDs nur über `Clock`, `Rng`, `IdGen` aus `contextra-ports`.
- I/O nur über den Port `Vfs`. Ein `fsync`-Fehler wird propagiert und nie auf demselben Handle wiederholt.
- Der Mandant kommt immer aus dem `TenantContext`, nie als Literal oder Default.
- Verschlüsselung und WAL-Integrität sind nicht optional. Kein `cfg(feature)`-Schalter, kein Laufzeitparameter dafür.
- Dokumente werden mit `DocRef` adressiert; `DocId(u64)` gibt es nicht.
- Jeder Test trägt `#[ctx_test(id = "T-…")]` mit einer SPEC-ID.
- Keine Doppeldefinitionen öffentlicher Typen. Kanonische Definition vor dem Hinzufügen suchen (`docs/TYPE_REGISTRY.md`).
- Aussagen über Löschung nur im Wortlaut aus SPEC 6.4, auch in Kommentaren und Fehlertexten.

## Ringe

| Ring | Crates |
|---|---|
| 0 | `contextra-types`, `contextra-ports` |
| 1 | `contextra-sys`, `contextra-simd`, `contextra-durable-fs`, `contextra-crypto` |
| 2 | `contextra-mvcc`, `contextra-store`, `contextra-text`, `contextra-vector`, `contextra-rank` |
| 3 | `contextra-erasure`, `contextra-engine` |
| 4 | `contextra` |
| dev | `contextra-testkit`, `contextra-testmacros`, `contextra-lab`, `ctx-verify`, `tests/arch` |

Abhängigkeiten zeigen nur auf gleiche oder niedrigere Ringe; innerhalb von Ring 2 und für `erasure`, `engine`, `contextra` gilt die Matrix SPEC 5.3. `contextra-erasure` kennt keine Ring-2-Crates. `contextra-vector` hängt weder an `sys` noch an `mvcc`. `lab` und `ctx-verify` nutzen keine Produktions-Decoder.

## Nicht vorschlagen

Diese Crates sind entfernt und werden nicht neu angelegt oder referenziert: `contextra-graph`, `contextra-cognition`, `contextra-adapt`, `contextra-kvcache`, `contextra-router`, `contextra-agent`, `contextra-infer-*`, `contextra-mcp`, `contextra-py`, `contextra-privacy`, `contextra-ingest`, `contextra-sandbox`, `contextra-wire`, `contextra-checkpoint`, `contextra-license`, `contextra-audit-export`, `contextra-avv-generator`, `contextra-core`, `contextra-db`. Ebenso entfallen: Lizenz-Gate, Embedder, Reranker, Query-Rewriter, automatische Extraktion, Windows-Code, `WasmMergeOperator` (SPEC D-24, D-84).

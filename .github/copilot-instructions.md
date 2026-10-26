@AGENTS.md

# GitHub Copilot — Spezifische Regeln (Stand: 2026-10-26)

> Normative Quelle: `docs/SPEC.md` (Fassung 1.2). Diese Datei ergänzt nur Copilot-spezifisches.

## Completion-Richtlinien (SPEC §0.2)
- Kein `.unwrap()`, `.expect()`, `panic!()` außerhalb von `#[cfg(test)]` (Regel 2 / P7)
- `unsafe` NUR in den drei Inseln `contextra-simd`, `contextra-sys`, `contextra-wire` — stets mit `// SAFETY:` (Regel 6 / P2)
- Kein `Instant::now()`, `SystemTime::now()`, `thread_rng()` in Kernlogik — Port `Clock`/`Rng`/`IdGen` verwenden (Regel 3 / P28)
- Kein `std::env::var` in Kernlogik (A5)
- Mandant nur aus `RequestContext`, nie aus Literalen (Regel 7 / INV-TENANT-1)
- Fehler als `Result<_, ContextraError>` propagieren, kein `let _ =` auf I/O (Regel 2)
- DAG-Schichtgrenzen einhalten: Abhängigkeiten nur zu gleichem oder niedrigerem Ring (Regel 1 / P1/P5)

## Archivierte Crates (NICHT bearbeiten — werden aus Workspace entfernt, PLAN WP 2.1)
- `contextra-graph`, `contextra-cognition`, `contextra-adapt`, `contextra-kvcache`
- `contextra-router`, `contextra-agent`, `contextra-infer-onnx`, `contextra-infer-ollama`
- `contextra-core` (Migrationsfassade, wird durch `types`/`ports` ersetzt)

## Aktive Crates (Kern [K])
`types` · `ports` · `mvcc` · `crypto` · `vector` · `text` · `rank` (fusion/) · `store` · `checkpoint` · `engine` · `db` · `mcp` · `contextra`

## Sicherheitsports (INV-WIRING-2) — müssen immer real implementiert sein
`KmsProvider`, `Vfs`, `DurableLog`, `ErasureJournal`, `LayerScanner`, `Purger`

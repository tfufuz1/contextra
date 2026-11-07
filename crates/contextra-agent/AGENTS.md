# AGENTS.md — `contextra-agent`

> **Ring 3** · Klasse Rand `[R]` (SPEC-Katalog: Archiv, Root-§10 K5) · Reifegrad `beta` · Unsafe: nein · Rolle: Workflow-Engine für Agenten: Zustandsgraph, Event-Quellen, Tools, Audit, DLQ.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-agent --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `checkpoint`, `db`, `graph`, `ports`, `router`, `store`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `engine`, `ports`, `types` |
| Features | `test-utils` |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- **Fünf Kanten über das Soll:** `checkpoint`, `db`, `graph`, `router`, `store` (harte Bindung an `LsmStorage` in Signaturen, DEBT-04). Dev: `text`, `vector`, `testkit`.
- DEBT-02: Altlasten (SQLite-/Robotik-Prototyp) vollständig entfernen; Zustand über MVCC-/Storage-Ports.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `engine.rs`, `engine/` | `OrchestratorEngine`: Event-Loop, Replay |
| `graph.rs`, `context.rs`, `step.rs`, `goal_condition.rs` | `StateGraph`, `AgentContext`, `AgentTool` |
| `event_source.rs`, `dlq.rs`, `budget.rs`, `clm_scratchpad.rs` | Quellen, Dead-Letter, Budget |
| `audit.rs`, `tools/` | Audit-Log (Präfix `__agent:audit:`), Tool-Registry |

## 4. Verträge & Invarianten

1. **Zustandsautomat:** `Pending→Running→Suspended/Completed/Failed`; nur die Engine führt Übergänge aus.
2. **Identifier-Validierung:** `task_id`/`node_id` nicht leer, keine NUL-Bytes, ≤ 256 Byte, keine Pfadtrenner (`try_new`).
3. **Audit-Pflicht:** jeder Übergang, jede Tool-Ausführung, jeder Fehler wird protokolliert; Puffer sind begrenzt.
4. Tools nutzen `ContextraError`; kein Lock über Fremdaufrufe.

## 5. Ports

Soll: konsumiert `StorageEngine`/`CheckpointCoordinator` nur als Ports.

## 6. Nebenläufigkeit & Performance

- Engine hält keine globalen Locks; Event-Loop liefert `EventLoopExitReason` und endet kontrolliert.

## 7. Anti-Patterns

- Ungeprüfte Task-IDs.
- `Arc<LsmStorage>` in Signaturen.

## 8. Tests & Messung

```bash
cargo test -p contextra-agent --locked
cargo clippy -p contextra-agent --all-targets -- -D warnings
```

- 23 Testdateien.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 4/5 | DEBT-02/04: Kanten auf Ports umstellen |

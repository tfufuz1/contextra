# AGENTS.md — `contextra-agent`

> Ebene E2 · Gruppe `agentic` · Reifegrad `beta` · Unsafe: nein (`#![forbid(unsafe_code)]`) · Rolle: Workflow-Engine für Agenten: Zustandsgraph, Event-Quellen, Tools, Audit, DLQ.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-11.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo check -p contextra-agent --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `types`, `ports`, optional: `db`, `checkpoint`, `router` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types` |
| Features | `storage-bindings`, `workspace`, `test-utils` |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Architektur

- `contextra-graph` ist vollständig aus den Normal-Dependencies entfernt und in `[dev-dependencies]` isoliert.
- Optionale Speicherbindungen (`db`, `checkpoint`, `router`) sind über das Feature `storage-bindings` bzw. Brücken gekapselt.
- Kern-Abstraktionen: `ContextStore` und `ScratchpadStore` definieren lokale Port-Abstraktionen.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `engine.rs`, `engine/` | `OrchestratorEngine`: Event-Loop, Replay |
| `graph.rs`, `context.rs`, `step.rs`, `goal_condition.rs` | `StateGraph`, `AgentContext`, `AgentTool` |
| `event_source.rs`, `dlq.rs`, `budget.rs`, `clm_scratchpad.rs` | Quellen, Dead-Letter, Budget |
| `audit.rs`, `tools/` | Audit-Log (Präfix `__agent:audit:`), Tool-Registry |
| `ports.rs` | Lokale Port-Definitionen `ContextStore` und `ScratchpadStore` |
| `compat/` | Speicher- und Adapterbrücken für `contextra-db` |

## 4. Verträge & Invarianten

1. **Zustandsautomat:** `Pending→Running→Suspended/Completed/Failed`; nur die Engine führt Übergänge aus.
2. **Identifier-Validierung:** `task_id`/`node_id` nicht leer, keine NUL-Bytes, ≤ 256 Byte, keine Pfadtrenner (`try_new`).
3. **Audit-Pflicht:** jeder Übergang, jede Tool-Ausführung, jeder Fehler wird protokolliert; Puffer sind begrenzt.
4. Tools nutzen `ContextraError` / `AgentError`; kein Lock über Fremdaufrufe.

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

---

# Kommentar-Syntax (verbindlich)

| Form | Zweck |
|---|---|
| `//!` | Modul-/Crate-Doku am Dateianfang (vor jedem Item, nach `#![...]`-Attributen der Datei; nie mitten in der Datei) |
| `///` | Rustdoc für das **direkt folgende** Item (Struct, Enum, Variante, Feld, Trait, Methode, Funktion, Konstante). Nie vor `let`, Ausdrücken, Match-Armen oder Makroaufrufen; dort `//` verwenden. |
| `// NOTE:` | Warum etwas so gebaut ist (nicht was) |
| `// TODO:` / `// FIXME:` / `// STUB:` | Offene Arbeit / bekanntes Risiko / Platzhalter; immer mit konkreter Aussage |
| `// SAFETY:` | Pflicht direkt über jedem unsafe-Konstrukt (Invariante, Herstellungsort, Folge bei Verletzung) |
| `// ANCHOR[contextra-agent:<BEREICH>-<NNN>]` | Sprungmarke; vor dem Item/Abschnitt, auf den sie zeigt |
| `// FILE-CONTEXT` + `// ZWECK:` / `// INVARIANTEN:` / `// HOTSPOTS: <von>-<bis>` / `// STAND: <Datum>` | Repo-Konvention für den Dateikopf. Vorhandene Blöcke ergänzen; `HOTSPOTS`-Zeilenbereiche nach dem Kommentieren auf die neuen Zeilennummern aktualisieren. `STAND` auf das heutige Datum setzen, wenn du den Block änderst. |

Weitere Tags führst du nicht ein. Beim Kommentieren markierter Altlasten gilt: `// NOTE: Deprecated Debt DEBT-NN – <Beschreibung>`.

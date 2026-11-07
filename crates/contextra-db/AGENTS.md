# AGENTS.md — `contextra-db`

> **Ring 3** · Klasse `[K]` (Referenz-Komposition) · Reifegrad `beta` · Unsafe: nein · Rolle: Dünne Referenzkomposition für Single-Node: Prelude/Advanced-API, Regler (`homeostat`, PID), `MultiStepEngine`, `volatile_vault`.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-db --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `engine`, `ports`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `engine`, `ports`, `types` |
| Features | viele (u. a. `onnx`, `reranking`, `sandbox`, `volatile-vault`, `background-maintenance`) |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- ~1.500 Zeilen; die frühere Beschreibung als „Collection/RRF/Chunker-Monolith“ ist überholt (diese Logik liegt in `engine`).
- Überzählige Kante `sys` wurde entfernt (D9). Unbenutzte Dev-Kanten (`cognition`, `rank`, `sys`) und M3-Leck `contextra_checkpoint` wurden bereinigt.
- `pid_latency_controller.rs`/`homeostat.rs`/`decay_controller` duplizieren Inhalte aus `adapt`/`engine`: DRY prüfen.
- Alte Aufrufstellen: `crates/contextra-db/DEPRECATED_DEBT.md` (DEBT-05, 40 Stellen).

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `lib.rs` | `prelude`, `advanced`, Re-Exports (`Collection`, `CollectionConfig`) |
| `multistep.rs` | `MultiStepEngine`, `QueryRewriter` |
| `homeostat.rs`, `pid_latency_controller.rs` | Regelkreise |
| `volatile_vault.rs` | flüchtiger, verschlüsselter Ergebnisspeicher (nicht die Lösch-`FileVault`) |

## 4. Verträge & Invarianten

1. **Keine Logik** außer Komposition (SPEC §4.3). Neue Fachlogik gehört in `engine` oder einen Baustein.
2. Keine Inferenz-Backends in `db`; Embedder kommen als Port von der Fassade (SPEC §4.4 Regel 8).
3. Keine direkte Bindung an Löschketten-Interna (PLAN 3.6).
4. LLM-gestützte Operationen (Multi-Step) nie unter Key-Lock oder Lese-Guard.

## 5. Ports

Konsumiert `engine`-Ports; implementiert keine.

## 6. Nebenläufigkeit & Performance

- Lock-Reihenfolge der `Collection` gilt unverändert (siehe `engine`).

## 7. Anti-Patterns

- Fachlogik hier ablegen.
- Neue Dev-Kanten auf höhere Schichten.

## 8. Tests & Messung

```bash
cargo test -p contextra-db --locked
cargo clippy -p contextra-db --all-targets -- -D warnings
```

- Tests laufen über `tests/` (54 Dateien/Module). Charakterisierungstests vor jeder Entfernung veralteter Suchmethoden.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 3.6 | monolithische Löschkaskaden entfernen |
| 5.x/D9 | `sys`-Kante, Dev-Kanten bereinigt |
| DEBT-05 | Legacy-Such-Fassaden nach Frist entfernen |

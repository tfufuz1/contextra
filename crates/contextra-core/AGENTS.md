# AGENTS.md — `contextra-core`

> **– (deprecated)** · Klasse – (Strangler-Fassade) · Reifegrad `deprecated` · Unsafe: nein · Rolle: Re-Export-Hülle (256 Zeilen) für nach `types`/`ports`/`mvcc`/`wire` verlagerte Typen. **Wird entfernt** (DEBT-03, D11).
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-core --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `mvcc`, `ports`, `types`, `wire` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | entfällt |
| Features | `test-utils` |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- **Keine neuen Abhängigkeiten auf dieses Crate.** Noch in `capabilities.toml` erlaubt für `contextra` und `contextra-checkpoint`; in Cargo hängt es nur noch teilweise.
- Jede Nutzung migriert auf `contextra-types`/`contextra-ports`.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `lib.rs` | Re-Exports; 1 `thread_rng`-Treffer und 1 Zeitaufruf prüfen |

## 4. Verträge & Invarianten

1. Nur Re-Exports; keine neue Logik, keine neuen Typen.
2. Löschen, sobald `grep -rn "contextra_core" crates` keinen Konsumenten mehr findet (Gate: 0 Downstream-Konsumenten).

## 5. Ports

Keine.

## 6. Nebenläufigkeit & Performance

- –

## 7. Anti-Patterns

- Neue Funktionalität hier ablegen.
- Neuen Konsumenten hinzufügen.

## 8. Tests & Messung

```bash
cargo test -p contextra-core --locked
cargo clippy -p contextra-core --all-targets -- -D warnings
```

- Kein Testaufwand außer Konsumentenzählung.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 5.5 | vollständig dekommissionieren |

# governance – Gate- und Regeldateien

Nicht normativ. Normativ sind [`docs/SPEC.md`](../../docs/SPEC.md) Abschnitt 5.8 (Budgets), 15.1 (Tests), 15.3 (Gates) und D-93 (Lockfile).

Dieser Ordner enthält maschinenlesbare Regeln für Tests und Gates. Die Inhalte der TOML-Dateien sind in dieser Revision nicht geprüft.

| Datei | Zweck nach SPEC | Zustand |
|---|---|---|
| `constitution.toml` | Verfassungsregeln als Prüfdaten | nicht geprüft, Abgleich mit SPEC 2 und 6.3 offen |
| `gates.toml` | Gate-Definitionen | nicht geprüft, Abgleich mit SPEC 15.3 offen |
| `protected-paths.toml` | Pfade, die nicht ohne Freigabe geändert werden | nicht geprüft |
| `ratchet.toml` | Basiswerte für `unwrap`, `expect`, `panic!`, `let _ =` (sinkend, SPEC 5.8) | **fehlt**; wird mit WP-0.1 angelegt (F-20) |
| `exceptions.toml` | befristete Ausnahmen von T-ARCH- und T-TRACE-Prüfungen (SPEC 15.1) | **fehlt**; wird mit WP-0.1 angelegt |
| `unsafe-safety-baseline.toml` | Basis für `unsafe`-Blöcke in den Inseln | nicht geprüft |
| `determinism-allow.toml`, `determinism-baseline.toml` | Ausnahmen und Basis für Nicht-Determinismus (P7) | nicht geprüft |
| `verdict-required.toml` | Pflichtnachweise je Gate | nicht geprüft |

Regel für Änderungen: Basiswerte in Ratchet-Dateien dürfen nur sinken. Ausnahmen brauchen ein Ablaufdatum (SPEC 15.1, 17.3).

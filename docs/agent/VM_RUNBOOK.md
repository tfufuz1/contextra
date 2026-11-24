# VM-Runbook (gemessen, 2026-11-24)

Umgebung: Ubuntu 24.04, 1 vCPU, 3,9 GB RAM, kein Swap, Datenträger 252 GB (Stand Messung: 4,9 GB frei).
Hinweis: Eine frühere Annahme „nur 17 GB nutzbar“ war nicht belegt und ist nicht übernommen.

## Befunde

| # | Befund | Lösung | Status |
|---|---|---|---|
| R1 | `static.rust-lang.org` nicht erreichbar (DNS-Fehler) | Rust über conda-forge (micromamba). `tools/env/bootstrap.sh` | getestet, Rust 1.89.0 installiert |
| R2 | Rust nicht vorinstalliert, `rustup` nicht nutzbar | siehe R1 | getestet |
| R3 | `cargo fetch --locked` über crates.io | funktioniert, 6 s | getestet |
| R4 | `cargo metadata --locked` | funktioniert | getestet |
| R5 | Workspace-Check (`--workspace`) über 1 vCPU | Lauf im Hintergrund wurde beendet (Prozess verschwand ohne EXIT). Nicht abgeschlossen. Nur pro Crate testen | offen |
| R6 | Hintergrundprozesse enden mit dem Tool-Aufruf | Vordergrund mit Timeout, Cargo setzt fort (Fortschritt in `target/`) | beobachtet |
| R7 | `time` und `PIPESTATUS` fehlen in `/bin/sh` | Skripte mit `#!/usr/bin/env bash`, Dauer per `date +%s` | getestet |
| R8 | Ausgabe kann Kontext füllen | `CARGO_TERM_PROGRESS_WHEN=never`, `CARGO_TERM_COLOR=never`, Begrenzung in `ctx.sh` | getestet |
| R9 | Page-Cache belegt RAM | `echo 1 > /proc/sys/vm/drop_caches` (als root). Messung: verfügbar 3,5 → 3,7 GB | getestet |
| R10 | Platte knapp | `tools/ctx.sh space` löscht `incremental` und micromamba-`pkgs`. Rust-Installation bleibt | getestet: 4,9 → 5,3 GB frei (ca. 0,4 GB, nicht mehr) |
| R11 | `cargo xtask` schlägt fehl (Paket fehlt) | Alias in `.cargo/config.toml` auskommentiert | getestet |
| R12 | Keine Git-Zugangsdaten in der Shell, kein GitHub-Konnektor in der Werkzeugliste | Push nicht möglich. Ergebnis als ZIP/Patch liefern | geprüft |
| R13 | `cargo test -p contextra-ids` | 17 Tests grün, 9 s | getestet |

## Befehle, die in dieser Umgebung funktionieren

    bash tools/ctx.sh status
    bash tools/ctx.sh check contextra-ids
    bash tools/ctx.sh test contextra-ids
    bash tools/ctx.sh gates
    bash tools/ctx.sh find 'regex' crates
    bash tools/ctx.sh space

## Nicht getestet

- Hooks in `.claude/settings.json`: Die Datei enthält nur `permissions` (Deny-Regeln für `git push`). Ob Claude Code sie in dieser Umgebung anwendet, ist nicht geprüft. Im Chat-Modus gibt es keine Hooks.
- GitHub-Push und Pull-Request: nicht möglich ohne Konnektor oder Token.

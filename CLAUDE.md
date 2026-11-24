# CLAUDE.md — Contextra

Du arbeitest als Principal Rust Architect am Projekt Contextra. Ziel: ein produktionsreifes, gesetzeskonformes Produkt. Diese Datei ist der Einstieg. Details stehen in den genannten Dateien, nicht hier.

## 1. Produkt

- Contextra verbindet Unternehmenswissen mit LLMs und Agenten. Zielgruppe: Mittelstand und Großunternehmen in Deutschland und Europa, keine Behörden.
- Kernqualität ist die **Löschung** personenbezogener Daten aus dem RAG (DSGVO Art. 17). Sie soll nachweisbar, gesetzeskonform und für Kunden unbemerkt im Hintergrund laufen.
- Der Speicherkern wird neu entwickelt (`contextra-storage` ersetzt `contextra-store`). Finale Crate-Entscheidungen fallen in dieser Neuentwicklung. Nichts ist final entschieden.

## 2. Sitzungsstart

1. `bash tools/ctx.sh status` (Git, Toolchain, RAM, Platte).
2. Bei Projektfragen zuerst `docs/agent/CRATES.md` (Crate-Übersicht). Dann gezielt suchen mit `bash tools/ctx.sh find <regex> <pfad>`.
3. Vor einem Build: `bash tools/ctx.sh status` prüfen. Die Platte ist knapp, siehe `docs/agent/VM_RUNBOOK.md`.

## 3. Umgebung

- Rust **1.89.0** (Projekt-Toolchain, `rust-toolchain.toml`). Installation über conda-forge, nicht über rustup: `bash tools/env/bootstrap.sh`. Idempotent.
- `static.rust-lang.org` ist nicht erreichbar. crates.io und conda-forge sind erreichbar.
- Hardware: 1 vCPU, 3,9 GB RAM.
- Builds nur pro Crate: `bash tools/ctx.sh check <crate>`, `bash tools/ctx.sh test <crate> [filter]`. `--workspace` ist auf dieser VM nicht abgeschlossen worden.
- Immer `--locked`. Ein Build darf `Cargo.lock` nicht verändern.
- Speicher: Bei Platzmangel `bash tools/ctx.sh space`. Der Befehl löscht nur regenerierbare Artefakte.
- Pro Befehl Ausgabe begrenzen. `tools/ctx.sh` begrenzt sie bereits.

## 4. Architektur (belegt)

- Die Crates gliedern sich in Gruppen, Ebenen und Tiers. Quelle: `capabilities.toml`. Übersicht: `docs/agent/CRATES.md`.
- Ring-Matrix in `tests/layering.rs`. Abhängigkeiten nur nach unten. **Achtung:** Diese Datei liegt im Workspace-Root und wird von keinem Crate eingebunden. Sie läuft unter `cargo test` daher nicht.
- Unsafe-Inseln laut `tests/unsafe_islands.rs`: `contextra-simd`, `contextra-sys`, `contextra-wire`. Dieselbe Datei ist ebenfalls nicht eingebunden.
- Konsistenz prüfen: `bash tools/ctx.sh gates`. Stand: 7 Workspace-Mitglieder fehlen in `capabilities.toml`, und `contextra-vault` steht dort, ist aber nicht Mitglied.
- `contextra-erasure` hat Gruppe `compliance` und Tier `frozen`. Änderungen dort nur mit ausdrücklicher Rückfrage.

## 5. Bekannte Befunde (gemessen 2026-11-24)

- Der Cargo-Alias `xtask` zeigt auf ein nicht existierendes Paket. Deaktiviert in `.cargo/config.toml`.
- Die Architektur-Tests in `tests/` laufen nicht (siehe oben). Eine Einbindung ist offen.
- `contextra-ids` (17 Tests) ist grün.
- Ein vollständiger Workspace-Check ist nicht durchgelaufen. Die Zahl der Warnungen ist daher unbekannt.

## 6. Änderungsregeln

- Vor Änderungen am Erasure-Pfad, am Speicherformat oder an Verträgen die betroffene Spec lesen und Gates prüfen.
- Commits und Pull-Requests sind erlaubt. `git push` nur nach ausdrücklicher Freigabe in der aktuellen Sitzung. Deny-Regel in `.claude/settings.json`.
- Falls ein GitHub-Werkzeug fehlt oder Berechtigungen braucht, im Chat nachfragen.
- Ohne Push-Möglichkeit: Ergebnis als ZIP oder Patch ausliefern.

## 7. Agenten

- Keine Subagenten, wenn du Sonnet oder Opus bist. Skripte und Werkzeuge bevorzugen. Subagenten nur als Haiku (Vorgabe des Projekts).

## 8. Pflege

- Veränderliche Fakten gehören nach `docs/agent/` oder in Skripte, nicht hierher.

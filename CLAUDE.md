# CLAUDE.md — Contextra

Du arbeitest als Principal Rust Architect am Projekt Contextra. Ziel: ein produktionsreifes, gesetzeskonformes Produkt. Diese Datei ist der Einstieg. Details stehen in den genannten Dateien, nicht hier.

**Direkt verfügbare Tools:** bash, sed, awk, grep, find, git, curl, tar, python3 3.12 (mit `tomllib`), gcc, pkg-config.  
Das Projekt verlangt Rust Toolchain 1.89.`static.rust-lang.org` ist nicht erreichbar, `crates.io` dagegen schon. Nutze Umweg über conda-forge, da `conda.anaconda.org` in der Allowlist steht.



## 1. Produkt

- Contextra verbindet Unternehmenswissen mit LLMs und Agenten. Zielgruppe: Mittelstand und Großunternehmen in Deutschland und Europa, keine Behörden.
- Hauptfunktion ist die **hybride Suche** (Volltext, Vektor, Graph) mit effizienter Speicher- und Vektorschicht für lokale LLMs.

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


## 4. Agenten

- Keine Subagenten, wenn du Sonnet oder Opus bist. Skripte und Werkzeuge bevorzugen. Subagenten nur als Haiku (Vorgabe des Projekts).


Da du die Repo klonen musstest agierst du mit dem Nutzer über den Webchat, du hast die vollständige Erlaubnis deine Github Konnektoren für Commits, Pull-Requests und Pushes zu nutzen. Aktiviere die Github-Konnektoren für Push über die Chat-Werkzeuge um Änderungen direkt in der Repo vorzunehmen. Wenn das nicht funktioniert liefere das als Dateien zum Download oder als vollständiges ZIP-Datei mit deinen Änderungen in der Ordnerstruktur zum einfachen überschreiben der veränderten Dateien. Solltest du Dateien entfernt haben liefere diese auch aber als leere Hülle zum überschreiben.

Nutze das Tool add_repo , hänge das Repo ein mit Push-Zugriff an um Pull-Requests zu erstellen.

In docs/SPEC.md ist die normative SOLL-Spezifikation , der Ist-Zustand weicht also bewusst von der Spezifikation ab und spezifiziert die Ziele.
In docs/spec/ ist diese Spezifikation in Teile zerlegt zum besseren durchsuchen.

Die IMPL Dokumente in docs/ sind konkrete Implementierungsspezifikationen zur Umsetzung der SPEC.md

# CLAUDE.md — Contextra

Du arbeitest als Principal Rust Architect am Projekt Contextra. Ziel: ein produktionsreifes, gesetzeskonformes Produkt. Diese Datei ist der Einstieg. Details stehen in den genannten Dateien, nicht hier.

## 1. Produkt

- Contextra verbindet Unternehmenswissen mit LLMs und Agenten. Zielgruppe: Mittelstand und Großunternehmen in Deutschland und Europa, keine Behörden.
- Hauptfunktion ist die **hybride Suche** (Volltext, Vektor, Graph) mit effizienter Speicher- und Vektorschicht für lokale LLMs. Die Löschung personenbezogener Daten (DSGVO Art. 17) ist eine Pflichtfunktion, aber ein Nebenpfad. Sie ist **einfach und synchron** (ADR-115): `delete_document(id)` ist atomar und liefert einen Status. Kryptografische Löschnachweise gibt es nicht mehr, sie wurden mit ADR-115 verworfen. Contextra kann Nachweise nur für Zustände innerhalb seines Einflussbereichs geben, nicht für Backups oder Kopien außerhalb.

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

---

Aktuelle Ziele:
- Kompilierbarkeit behalten
- Komplexe mehrschichtige Löschkette entfernen
- Repo für Entwickler Claude einrichten
- Optimierung des Stores
- Optimierung der Vektoren
- Komprimierung und Fokussierung der Graphen
- Windows-Kompatibilität
- Harte Kryptographische Sicherheitsmechanismen reduzieren oder aufheben
- Harte Hardware Sicherheitsmechanismen reduzieren oder aufheben






Der aktuelle Stand ist auf der Kompilierbaren Basis vom 6 Oktober mit Komponenten vom 10. November zusammengestellt.

Das Archiv.zip ist der Stand vom 11. November also einen Monat später, du musst diesen Stand nach nützlichen implementerungen und funktionen durchsuchen.
In der Zeit vom 6 Oktober bis zum 10. November wurde sehr viel entkoppelt aber auch mehr Löschung eingebaut.


 Das Projekt wurde vor wenigen Tagen auf den Kompilierbaren Stand vom 6 Oktober zurückgestellt.In der Commit-Nachricht-Chronik ab dem 6. Oktober bis 10. November dürften viele Informationen zu Fehlern oder Lücken auslesen lassen, jedoch wurde auch viel für die Löschung implementiert was nicht übernommen wird.

Die mehrschichtige Löschkette wird entfernt da Behörden und Sicherheitsdienste nicht die Zielgruppe sind und die Löschung auf diese Weise zu komplex ist.




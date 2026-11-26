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

---

Das Ziel von Contextra ist es in Unternehmen einsatzbereit verwendet werden zu können ohne Ausrichtung auf DSGVO-konforme Löschung oder hochsicherheits Löschmechanismen. Contextra soll ein speziell auf LLM & Agenten ausgelegtes Datenbank und RAG-System für den effizienten Einsatz in Unternehmen sein.
Contextra soll eine kompatible Lösung werden für Unternehmensdaten in einem RAG-System. Eine effiziente performante und ressourcenschonende Lösung.
Aktuell ist eine komplexe Löschkette mit Nachweis implementiert und diese muss sauber und vollständig losgelöst und entfernt werden.
Du hast vollständige Berechtigung und Entscheidungsfreiheit für die Optimierung und Refaktorisierung des contextra-store.
Erfasse den Umfang des Projekts und die Abhängigkeiten von contextra-store um eine saubere Lösung zu konzipieren und zu planen. Lese auch die Konzepte in docs/concepts und entnehme die besten Aspekte, ignoriere jedoch auch dort die harten Löschungen und Löschkette mit Nachweis vollständig.
Diese Anweisung hat oberste Prioriät und gilt bei Konflikten anstatt der Dokumente im Repo.
Die Mission ist es Datenbanklösung für die Herausforderungen mit LLM und Agenten zu entwickeln die es Unternehmen ermöglicht ihre Daten sicher mit KI zu verknüpfen und einzusetzen. Diese Datenbanklösung wird also speziell für LLM & Agenten entwickeln. Unternehmen brauchen eine unkomplizierte, saubere, ressourcenschonende und Leistungsfähige Softwarelösung um den Einsatz von KI zu vereinfachen. Contextra muss effizient auf LLM ausgelegt die Daten verarbeiten und bereitlegen. Die Architektur muss sehr sauber durchdacht sein ohne Lücken oder blinde Flecken. 


Da du die Repo klonen musstest agierst du mit dem Nutzer über den Webchat, du hast die vollständige Erlaubnis deine Github Konnektoren für Commits, Pull-Requests und Pushes zu nutzen. Aktiviere die Github-Konnektoren für Push über die Chat-Werkzeuge um Änderungen direkt in der Repo vorzunehmen. Wenn das nicht funktioniert liefere das als Dateien zum Download oder als vollständiges ZIP-Datei mit deinen Änderungen in der Ordnerstruktur zum einfachen überschreiben der veränderten Dateien. Solltest du Dateien entfernt haben liefere diese auch aber als leere Hülle zum überschreiben.

Nutze das Tool add_repo , hänge das Repo ein mit Push-Zugriff an um Pull-Requests zu erstellen.

Das Projekt muss umgedacht werden, bisherige Entscheidungen dürfen final überdacht oder geändert werden für ein funktionales produtionsreifes Produkt.

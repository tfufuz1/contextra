# Contextra – Kontext für Jules-Sitzungen

Nicht normativ. Maßgeblich ist [`docs/SPEC.md`](../../docs/SPEC.md), Version 4.3.0.

## Ladeordnung

1. `.agents/jules/SESSION_BOOTSTRAP.md`: Ablauf einer Sitzung.
2. `.agents/jules/PREAMBLE.md`: Entscheidungsregeln, Stopp-Katalog, Prüfbefehle.
3. `.agents/jules/tasks/BACKLOG.md`: Arbeitsliste.
4. `AGENTS.md`: Ist-Zustand und harte Regeln.
5. `docs/SPEC.md`: Abschnitte 0, 2, 15.3 und 15.4 sowie der Abschnitt zum gewählten Paket.
6. `crates/<crate>/AGENTS.md`: Hinweise je Crate.

## Projektstand

- Contextra ist eine eingebettete Rust-Bibliothek für lokale Speicherung und nachweisbares Löschen in Agenten- und RAG-Systemen (SPEC 1.1).
- **Den Ist-Zustand ermitteln, nicht abschreiben.** SPEC 3.2 ist eine Momentaufnahme. Aktueller Stand: `cargo check --workspace --all-targets --locked`, `git ls-files crates`, `.agents/jules/tasks/BACKLOG.md`.
- Der Löschkern (`contextra-erasure`), `contextra-lab` und `ctx-verify` sind Workspace-Mitglieder und teilweise umgesetzt. Vollständigkeit je Schicht belegt nur ein Test mit Zeuge (P1); ein Ein-Zeilen-Stub gilt als Defekt-Backlog, nicht als Sollzustand.
- Das Testmakro `#[ctx_test]` existiert (`contextra-testmacros`, WP-0.1); siehe `PREAMBLE.md`, Abschnitt D.
- Crate-Namen aus älteren Dokumenten und Skripten (`contextra-wire`, `contextra-mcp`, `contextra-graph`, `contextra-db`, `contextra-core`) existieren nicht und werden nicht neu angelegt (D-24, D-45).

## Crate-Topologie

Die verbindliche Ringzuordnung und die erlaubten Abhängigkeiten stehen in SPEC 5.1 bis 5.3. Die Prüfung erfolgt durch `contextra-arch-tests` (`tests/arch`; T-ARCH-1 bis T-ARCH-8). Fachprofile je Crate stehen in `crates/<crate>/AGENTS.md`.

Vorhandene Crates: `crates/` enthält `contextra`, `contextra-crypto`, `contextra-durable-fs`, `contextra-engine`, `contextra-erasure`, `contextra-lab`, `contextra-mvcc`, `contextra-ports`, `contextra-rank`, `contextra-simd`, `contextra-store`, `contextra-sys`, `contextra-testkit`, `contextra-testmacros`, `contextra-text`, `contextra-types`, `contextra-vector`, `ctx-verify`; dazu `tests/arch` als Paket `contextra-arch-tests`. Prüfe die Mitgliedschaft im Workspace mit `cargo metadata --no-deps`.

## Feste Grenzen beim Schreiben

- `unsafe` nur in `contextra-sys` und `contextra-simd`. Zwölf Crates tragen `#![forbid(unsafe_code)]`. Keine neuen Ausnahmen.
- Kein `std::fs` außerhalb von `contextra-durable-fs` (Ziel laut WP-1.4; heute liegen Zugriffe in `store` und `vector` vor). Keine neuen Zugriffe einführen.
- Keine neuen Marken `ANCHOR[`, `FILE-CONTEXT`, `SESSION:`, `STAND:` im Quelltext.
- Keine Löschaussage außer den zulässigen Formulierungen in SPEC 6.4.

## Was Jules nicht tut

- Keine Änderung an geschützten Pfaden, auch nicht zum Lösen eines Fehlers. Ist eine solche Änderung nötig, siehe `PREAMBLE.md`, Stopp-Katalog S3.
- Keine Änderung an Verschlüsselung, WAL-Integrität, Löschlogik oder Mandantenprüfung ohne Arbeitspaket aus SPEC 15.4.
- Keine neuen Abhängigkeiten, `unsafe`-Blöcke oder Features ohne Begründung im PR (HR-12).
- Keine Löschaussagen außerhalb SPEC 6.4.

## Bekannte Stolperstellen

- `docs/COMPILATION.md` und die `justfile` nennen Befehle, die nicht überall funktionieren. Verlässlich sind die Befehle aus `PREAMBLE.md`, Abschnitt C, Punkt 7. `just gates` ruft `cargo test --test layering` auf; diese Testdatei existiert nicht. Der Arch-Test liegt in `tests/arch`.
- `cargo xtask …` existiert nicht. Befehle aus älteren Audit-Prompts (`docs/Jules-Prompts/`) sind nicht ausführbar, solange sie nicht ersetzt sind.
- Die PR-Vorlage und ältere Dokumente verweisen auf eine eigene Plandatei. Sie existiert nicht. Maßgeblich ist SPEC 15.4 (KF-9).
- Die Datei `.agents/jules/context/SESSION_START.md` ist generiert und nicht versioniert. Sie ist nur gültig, wenn `refresh-context.sh` in dieser Sitzung gelaufen ist.
- `GITHUB_WORKFLOW.md` beschreibt GitHub-Issues als Aufgaben. Maßgeblich für die Auswahl ist `tasks/BACKLOG.md`. Issues sind eine optionale Ergänzung für Kommentare und Befunde.

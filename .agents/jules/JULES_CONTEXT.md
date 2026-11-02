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

- Contextra ist eine eingebettete Rust-Bibliothek für lokale Speicherung und nachweisbares Löschen in Agenten- und RAG-Systemen.
- Der Workspace baut am HEAD nicht vollständig (SPEC F-32). Die Lockdatei ist veraltet.
- `crates/contextra-erasure` ist ein leeres Skelett (README und AGENTS.md, keine Quelldatei) und kein Workspace-Mitglied (SPEC F-33).
- Das Testmakro `#[ctx_test]` ist nicht definiert (siehe `PREAMBLE.md`, Abschnitt D).
- Die Crate-Namen `contextra-wire`, `contextra-mcp` und `contextra-lab` sind in älteren Dokumenten und Skripten genannt. Im Repository existieren sie nicht. `contextra-lab` und `ctx-verify` sind Ziel von WP-4.4.

## Crate-Topologie

Die verbindliche Ringzuordnung und die erlaubten Abhängigkeiten stehen in SPEC 5.1 bis 5.3. Die Prüfung erfolgt durch `contextra-arch-tests` (`tests/arch`, Tests T-ARCH-3, T-ARCH-5, T-ARCH-6).

Vorhandene Crates (`crates/`): `contextra`, `contextra-crypto`, `contextra-durable-fs`, `contextra-engine`, `contextra-erasure` (Skelett), `contextra-mvcc`, `contextra-ports`, `contextra-rank`, `contextra-simd`, `contextra-store`, `contextra-sys`, `contextra-testkit`, `contextra-text`, `contextra-types`, `contextra-vector`. Dazu `tests/arch` als Paket `contextra-arch-tests`.

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
- Die PR-Vorlage verweist auf `docs/PLAN.md`. Diese Datei gibt es nicht. Maßgeblich ist SPEC 15.4.
- Die Datei `.agents/jules/context/SESSION_START.md` ist generiert und nicht versioniert. Sie ist nur gültig, wenn `refresh-context.sh` in dieser Sitzung gelaufen ist.
- `GITHUB_WORKFLOW.md` beschreibt GitHub-Issues als Aufgaben. Maßgeblich für die Auswahl ist `tasks/BACKLOG.md`. Issues sind eine optionale Ergänzung für Kommentare und Befunde.

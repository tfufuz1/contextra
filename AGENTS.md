# AGENTS.md

Einzige verbindliche Quelle für dieses Repository ist `SPEC.md` im Wurzelverzeichnis (Version 1.1). Was dort nicht steht, ist nicht beauftragt. `SPEC.md` darf nur der Maintainer ändern.

## Arbeitsweise für Implementierer (Menschen und LLMs)

1. Lies `SPEC.md` vollständig, bevor du anfängst. Besonders: Abschnitt 4 (Regeln R-01 bis R-14), Abschnitt 14 (Arbeitspakete) und Abschnitt 16 (Arbeitsregeln).
2. Eine Aufgabe ist genau ein Arbeitspaket (WP) und genau ein Pull Request. Ändere nur die Dateien, die im Auftrag genannt sind (R-14).
3. Bei Unklarheit anhalten: unstrittigen Teil umsetzen, im PR-Text einen Abschnitt `## FRAGE` mit Default-Vorschlag anlegen, Aufgabe beenden (SPEC 0.3).
4. Fertig heißt: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` und `cargo test --workspace` sind grün und die Akzeptanzkriterien des WP sind durch Tests belegt (SPEC 16.2). Tests nie löschen, ignorieren oder abschwächen.
5. Kein `unwrap`, `expect`, `panic!`, `unsafe`, keine Platzhalter und keine Mocks im Produktionscode. Keine neue Abhängigkeit ohne Eintrag in `DEPENDENCIES.md`.
6. Texte (README, Doku, Fehlermeldungen) dürfen die Wörter aus SPEC 1.5 nicht enthalten.

## Altbestand

Der frühere Code ist als Tag `archive/legacy-v1` eingefroren (Commit 076a9882eb3e23209f9527d35c269ae09136934d). Lesen: `git show archive/legacy-v1:<pfad>`. Falls Tags nicht verfügbar sind: Branch `legacy/v1-src`. Übernahme nur Datei für Datei in einem eigenen PR mit Quellangabe (SPEC 13.2).

## Ehrlichkeit

Behaupte nie, ein Test sei gelaufen, wenn er nicht lief. Ein unvollständiger, ehrlicher PR ist erwünscht (SPEC 16.4).

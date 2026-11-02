# Routine: Flake-Jagd

- **Takt:** nächtlich
- **Ziel:** nicht deterministische Tests finden und beheben (Ursachen: Nebenläufigkeit, Zeitgrenzen, fehlende Ports für Zeit und Zufall, P7)
- **Eingaben:** `cargo nextest run --retries 0`. `cargo-nextest` muss installiert sein; die Vorfassung setzte es voraus, die Installation ist nicht geprüft.
- **Harte Grenzen:** höchstens 15 Dateien in einem Crate je PR. Scope: `crates/*/tests/**`.
- **Stopp:** Flakes durch Einflüsse außerhalb des Repositorys sind keine Codeänderung.
- **Ergebnis:** Flake-Issue oder PR mit Korrektur der Ursache. Keine Verlängerung von Zeitgrenzen als Lösung.
- **Eskalation:** siehe [`../HARNESS_QUICKSTART.md`](../HARNESS_QUICKSTART.md).

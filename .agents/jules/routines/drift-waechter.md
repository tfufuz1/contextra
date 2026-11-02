# Routine: Drift-Wächter

- **Takt:** täglich
- **Ziel:** Abweichungen früh erkennen zwischen den Zweck-Sätzen (SPEC 5.1a) in `Cargo.toml`, `lib.rs` und `README.md`, den Crate-Ringen (5.1, 5.2), den Veto-Einträgen und dem Wortlaut der Außenaussage (6.4). Das prüft `contextra-arch-tests` (T-ARCH-5, T-LINT-1).
- **Eingaben:** `cargo test -p contextra-arch-tests --locked`, `git diff --check`
- **Harte Grenzen:** Scope `docs/**`, `README.md`, `AGENTS.md`, `crates/*/README.md`, `crates/*/AGENTS.md`, `crates/*/Cargo.toml`.
- **Stopp:** Generierte Dateien, die von Hand geändert wurden, ohne dass die Quelle angepasst wurde. Unklare Abweichungen zwischen Code und SPEC werden nicht korrigiert, sondern gemeldet.
- **Ergebnis:** PR mit Korrektur der Dokumente oder Bericht zu Code-Abweichungen.
- **Eskalation:** siehe [`../HARNESS_QUICKSTART.md`](../HARNESS_QUICKSTART.md).

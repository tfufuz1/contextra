# Routine: Abbau von `unwrap` und `allow`

- **Takt:** wöchentlich
- **Ziel:** Verbleibende `unwrap()`, `expect()`, `panic!` und `#[allow(...)]` im Produktionscode abbauen (P6, SPEC 5.8). Die Zahlen dürfen nur sinken.
- **Eingaben:** `cargo clippy --workspace --all-targets --locked -- -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic`, Basis in `.github/unwrap_baseline.txt`. Der Ratchet in `governance/ratchet.toml` wird mit WP-0.1 angelegt.
- **Harte Grenzen:** höchstens 15 Dateien in einem Crate je PR. Scope: `crates/*/src/**`.
- **Stopp:** Wenn das Ersetzen eines `unwrap` eine Änderung der öffentlichen API erzwingt, wird die Änderung gemeldet, nicht umgesetzt.
- **Ergebnis:** PR mit ersetzten Stellen durch typisierte Fehlerbehandlung.
- **Eskalation:** siehe [`../HARNESS_QUICKSTART.md`](../HARNESS_QUICKSTART.md).

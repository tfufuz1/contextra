# Abhängigkeiten und Ringe

Abgeleitet aus SPEC 5.2, 5.3, AR-1 bis AR-14, D-54, D-88, D-93, HR-1, HR-12.

- **Ringe.** Ring 4 `contextra`; Ring 3 `contextra-erasure`, `contextra-engine`; Ring 2 `mvcc`, `store`, `text`, `vector`, `rank`; Ring 1 `sys`, `simd`, `durable-fs`, `crypto`; Ring 0 `types`, `ports`. Abhängigkeiten zeigen nur nach unten.
- **Matrix.** Erlaubte Kanten stehen in SPEC 5.3. Eine Kante ohne Matrixeintrag ist verboten. Dev-Abhängigkeiten folgen derselben Matrix (AR-14).
- **Netzwerk.** Keine HTTP-, Socket- oder DNS-Bibliothek im Produktionsbaum (HR-1, D-54). `cargo tree -e normal` und `deny.toml` prüfen das.
- **Neue Abhängigkeit.** Begründung im PR, Nachweis der Matrixkonformität, Nennung des belegenden Tests (HR-12).
- **Lockfile.** `Cargo.lock` ist eingecheckt und wird mit jeder Manifest-Änderung im selben Commit erneuert. Prüfungen mit `--locked` (D-93).
- **Glob-Re-Exporte** sind verboten, besonders in `ports` (AR-10).
- **Kein Selbst-Dev-Dependency** zum Aktivieren von Features (D-97).

# Grundregeln für Jules

Nicht normativ. Maßgeblich: [`docs/SPEC.md`](../../docs/SPEC.md) (Version 4.2.0) und [`AGENTS.md`](../../AGENTS.md).

1. Lies `AGENTS.md`, den Abschnitt der SPEC zur Aufgabe und die `AGENTS.md` des betroffenen Crates.
2. Verifiziere jedes Symbol, bevor du es verwendest. Nimm keine APIs aus dem Gedächtnis an.
3. Ändere nur Dateien, die zur Aufgabe gehören. Alles andere wird im PR unter „Gefundene Probleme außerhalb des Umfangs“ gemeldet.
4. Ändere niemals Gates, Lints, Baselines, Toolchain-Pins, `governance/`-Dateien oder geschützte Pfade, um Rot zu beseitigen. Behebe die Ursache.
5. Schreibe zuerst einen Test, der fehlschlägt, und dann die Korrektur. Der Test trägt die SPEC-ID (`#[ctx_test(id = "T-…")]`).
6. Tritt derselbe Fehler zweimal auf, reflektiere das Vorgehen und stoppe.
7. „Fertig“ heißt: `cargo check --workspace --all-targets --locked`, `cargo test --workspace --locked` und `cargo test -p contextra-arch-tests --locked` sind grün. Befehle: `docs/COMPILATION.md`. Der Task-Runner `just` ist optional.
8. Ändert sich `Cargo.toml`, wird `Cargo.lock` im selben Commit erneuert (D-93).

## Arbeitspaket

- WP-Nummer (aus `docs/SPEC.md` 15.4, z. B. `WP-1.1`):
- Gate (aus `docs/SPEC.md` 15.3, z. B. `B`):
- Karte: `.agents/jules/tasks/WP-x.y.toml` (oder „neu angelegt“)
- Status in `.agents/jules/tasks/BACKLOG.md` gesetzt auf: `done` / `blocked` / `in_progress`

## SPEC-IDs

<!-- Betroffene IDs aus docs/SPEC.md, z. B. P6, D-94, T-ISO-1, F-13. -->

## Änderung

<!-- Was wurde geändert und warum? Verweis auf SPEC-Abschnitt. -->

## Getroffene Entscheidungen

<!-- Jede Entscheidung, die Jules ohne Rückfrage getroffen hat: Entscheidung, Begründung, SPEC-Verweis. Leer lassen, wenn keine. -->

## Testliste

<!-- Neue und geänderte Tests mit SPEC-ID. Rot-Nachweis (fehlgeschlagene Ausgabe) und Grün-Nachweis nennen. -->

## Prüfung

Ausgabe der Befehle nennen, nicht nur ankreuzen.

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo check --workspace --all-targets --locked`
- [ ] `cargo test --workspace --locked` (oder betroffene Crates mit `-p`, Begründung falls nur Teilmenge)
- [ ] `cargo test -p contextra-arch-tests --locked`
- [ ] Keine neuen `unsafe`-Blöcke außerhalb `contextra-sys` und `contextra-simd`
- [ ] Keine neuen `unwrap`, `expect`, `panic!`, `let _ =` auf `Result` im Produktionscode
- [ ] Keine neuen `#[allow]`-Ausnahmen ohne Begründung im PR
- [ ] `Cargo.lock` im selben Commit, falls `Cargo.toml` geändert wurde
- [ ] Dokumentation angepasst (betroffene `AGENTS.md`, `docs/SPEC.md` bei Fakten)

## Risiko

<!-- Welche Teile könnten brechen? Welche Tests decken sie ab? -->

## Gefundene Probleme außerhalb des Umfangs

<!-- Befunde, die nicht zum Paket gehören. Leer lassen, wenn keine. -->

## Menschliche Freigabe nötig

<!-- Änderungen an geschützten Pfaden (siehe .agents/governance/protected-paths.toml), die dieser PR nicht umsetzen konnte. Genaue Datei und genaue Änderung nennen. Leer lassen, wenn keine. -->

# contextra-router — Umsetzungsplan

> Abgeleitet aus `docs/SPEC.md` v3 §6 (Umsetzungsreihenfolge) und Crate-lokaler `SPEC.md`.

## 1. Statusübersicht

| Schritt | Inhalt | Voraussetzung | Nachweis | Status |
|---|---|---|---|---|
| S-00R | Baseline mit `cargo`, `loc.py`, Inspector | — | Baseline-Messung verifiziert | ☑ erledigt |
| S-01R | Gate-Reparatur, Layering | S-00R | Keine Aufwärtskanten E2 -> E3 | ☑ erledigt |
| S-02R | Golden-Korpus-Verifikation | S-00R | `tests/golden_corpus_test.rs` grün | ☑ erledigt |
| S-62R | `router`: `wire` optional hinter `subprocess-dispatch`, Vfs-Injektion | S-01R | Feature-Prüfungen & Vfs-Laden grün | ☑ weitgehend erledigt (`ports_local` verbleibt für Rückwärtskompatibilität) |
| S-SYS | Systematisches Comment-Anchor & Sektions-System | — | Alle Quelldateien nach Schema dokumentiert | ☑ erledigt |

## 2. Definition of Done

- Alle Tests (`cargo test -p contextra-router --all-features`) grün.
- Clippy (`cargo clippy -p contextra-router --all-targets --all-features -- -D warnings`) ohne Befunde.
- `tools/zero_panic_analyzer.py` meldet 0 Prod-Verstöße.
- `tools/determinism_guard.py` meldet 0 Verstöße.
- Jede Rust-Datei besitzt vollständige `FILE-CONTEXT`-Header, `ANCHOR`-Tags und strukturierte `SECTION`-Blöcke.

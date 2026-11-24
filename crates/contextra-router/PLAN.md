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

S-00 (Baseline, exakte `from_key`-Zählung) und S-01 (Layering-Gate, funktionierendes `public-api`) blockieren jeden weiteren Schritt. Ohne S-01 darf nichts gemergt werden.

## 2. Schritte dieses Crates [S]

| Schritt | Inhalt | Voraussetzung | Nachweis | Rollback | Status |
|---|---|---|---|---|---|
| S-62 | `router`: `ports_local` weg (nach `mcp`-Umstellung), `wire` optional, `Vfs`-Stelle | S-01 | GL | Revert | ☑ abgeschlossen (WP-RT-1) |

## 3. Reihenfolge-Zwang [S]

Keine direkte Reihenfolgeregel im Zyklen-Absatz von §7. Gilt die globale Reihenfolge (S-00 → S-01 → S-02 → S-03).

## 4. Definition of Done [S]

- G-L2, G-L1, G-L3, G-L4 grün (§8.5, R6). Bei fehlender Brücke: G-L2 entfällt.
- Für jeden Schritt aus Abschnitt 2 der Nachweis in der Spalte „Nachweis“ erbracht.
- Kein Eintrag in einer Golden-Datei geändert (INV-1), außer bewusst über ID-Breite (§5.2).
- Keine neuen `unwrap`/`expect`/`panic!` in Kern-Pfaden (R8), geprüft mit `tools/zero_panic_analyzer.py`.
- `tools/determinism_guard.py` meldet keinen neuen Befund (INV-5).
- Dokumentation: nur Ist-Aussagen, keine Compliance-Zusicherungen (INV-7, G-DOC1).

## 5. Offene Entscheidungen [S]

Keine offenen Entscheidungen mit direktem Bezug. Die globalen Entscheidungen D-01 bis D-12 (§9.1) gelten.

## 6. Verweise

- Spezifikation: `SPEC.md` (dieses Verzeichnis)
- Normativ: `docs/SPEC.md` §7 (Reihenfolge), §8 (Gates), §9 (Entscheidungen, Risiken)
- Werkzeuge: `tools/gate_layering.py` (G-L1, nach S-01), `tools/isolation_proof.py` (G-L4), `tools/feature_audit.py` (S-12, nach S-12 vorhanden)

- Alle Tests (`cargo test -p contextra-router --all-features`) grün.
- Clippy (`cargo clippy -p contextra-router --all-targets --all-features -- -D warnings`) ohne Befunde.
- `tools/zero_panic_analyzer.py` meldet 0 Prod-Verstöße.
- `tools/determinism_guard.py` meldet 0 Verstöße.
- Jede Rust-Datei besitzt vollständige `FILE-CONTEXT`-Header, `ANCHOR`-Tags und strukturierte `SECTION`-Blöcke.

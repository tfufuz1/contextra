# contextra-types — Umsetzungsplan

> Abgeleitet aus docs/SPEC.md v2 §7 (Umsetzungsreihenfolge). Jeder Schritt ist ein **Commit-Paar** (INV-3) mit Nachweis (§8). Status-Stand: **alle Schritte offen**; in der Analyseumgebung wurde nichts ausgeführt.

## 1. Vorbedingungen (global) [S]

| Schritt | Inhalt | Voraussetzung | Nachweis | Rollback | Status |
|---|---|---|---|---|---|
| S-00 | Toolchain `1.89.0`; Baseline: `cargo metadata`-Graph, `cargo tree` je Crate (`--no-default-features` und Default), `cargo check` je Crate, `rg`-Zählungen (§8.2) nach `docs/decoupling/baseline/`; exakte Zählung der `from_key`-Aufrufer | — | Baseline-Dateien eingecheckt; Differenz zu `80f1c33` dokumentiert | n. a. | ☐ offen |
| S-01 | Gate-Reparatur (B05): `tools/gate_layering.py` (§8.1), `tests/layering.rs` scheitert bei Verstoß, `xtask`-Verweise entfernen, `tests/arch` Mitglied oder gelöscht, `public-api` lauffähig, CI: `cargo check --workspace --all-targets --locked`, TSan/Loom/Mutation-Pfade (B11), Matrix `docid-128`. Geschützte Pfade ⇒ ADR + Label `protected-path` | S-00 | Gate scheitert absichtlich an einem Test-Verstoß (Negativtest), grün am Baseline-Stand | Revert | ☐ offen |
| S-02 | Golden-Korpus + Differenz-Harness (§5.1, §5.4) | S-00 | GF grün am unveränderten HEAD | Revert | ☐ offen |
| S-03 | Doku/Metadaten: `AGENTS.md` (Autonomie-/Drohtext entfernen, Compliance-Absatz an K6 anpassen), `capabilities.toml` Schema v3 (§8.3) mit Ziel-`allowed_internal`, falsche Docstrings (`privacy`, `cognition`, `kvcache`, `avv-generator`, `types/lib.rs`, `sys/lib.rs:19`, `vector/types.rs:3–6`), Item-Diff `AvvError` | S-01 | `capabilities-consistency` grün; G-DOC1 grün | Revert | ☐ offen |
| S-70 | Aufräumen: deprecated Re-Exports (R9) entfernen, Fehlervarianten in `ContextraError` entfernen (nur wenn G-E1 grün), `archive/`-Verschiebungen laut B10 (nur bei erfülltem Nachweis) | alle | alle Gates | Revert | ☐ offen |

S-00 (Baseline, exakte `from_key`-Zählung) und S-01 (Layering-Gate, funktionierendes `public-api`) blockieren jeden weiteren Schritt. Ohne S-01 darf nichts gemergt werden.

## 2. Schritte dieses Crates [S]

| Schritt | Inhalt | Voraussetzung | Nachweis | Rollback | Status |
|---|---|---|---|---|---|
| S-10 | `ids`: (A) Crate anlegen, Quellcode **kopieren** (mit `IdError`), `types` unverändert; Differenztest alt↔neu; (B) `types` → Re-Export + `From<IdError>`; (C) Aufrufstellen von `from_key` & Co. anpassen | S-02 | GF, GD, GE, GL | Revert C, B, A | ☐ offen |
| S-24 | `erasure` Features + Typabgleich (die Typen-Übernahme aus `types`/`ports` folgt in S-50/S-51) | S-12 | `cargo check --no-default-features` + je Feature | Revert | ☐ offen |
| S-34 | `graph`: Brücke, `GraphError`, `default = ["std"]`, `PprConfig` & Co. aus `types` (zusammen mit S-51-Teil) | S-10, S-11, S-12 | GL, GF (bincode), GC (TSan), GE | Revert | ☐ offen |
| S-51 | `types` → `erasure` (Erasure-Typen), `graph` (Ppr/TlHfd); `lib.rs`-Verwenderliste | S-50, S-34 | GL, GE | Revert | ☐ offen |

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

| D-03 | `ModelFingerprint`/`RetrievalStrategy` bleiben in `types` (B16) |

## 6. Verweise

- Spezifikation: `SPEC.md` (dieses Verzeichnis)
- Normativ: `docs/SPEC.md` §7 (Reihenfolge), §8 (Gates), §9 (Entscheidungen, Risiken)
- Werkzeuge: `tools/gate_layering.py` (G-L1, nach S-01), `tools/isolation_proof.py` (G-L4), `tools/feature_audit.py` (S-12, nach S-12 vorhanden)


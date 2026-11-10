# contextra-crypto — Umsetzungsplan

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
| S-12 | Feature-Hygiene (B14): `tools/feature_audit.py`, Defaults gemäß §3.1 für E0-/Test-Crates (`adapt`, `license`, `audit-export`, `testkit`, `crypto`, `graph`, `kvcache`, `rank`, `text`, `vector`), leere Features entfernen, `erasure` Features; Gate `error-domain` aktivieren | S-01 | `feature_audit` grün; `cargo hack check --each-feature -p <crate>` je Crate; `--workspace` grün | je Crate Revert | ☐ offen |
| S-31 | `crypto` Kern: `TenantId`→`ids`; KV-Signaturen auf `[u8;32]` (eigener Commit); `default = []`; `MvccError`/`CryptoError`-Abbildung | S-10, S-11, S-12 | GL, GF (WAL-HMAC, MFKD, KV v2), GC, GE | Revert | ☐ offen |
| S-38 | **Paar:** `crypto`-KV-Teil → `kvcache` (`kv_cipher`, `kv_segment/*`, `kv_shredding`, `revocation_log`; Konstanten-Kopplung bis Vereinigung); `kvcache` Kern/Brücke/Features; `infer-candle/kv_bridge` auf `kvcache`; `mutants-guarantees.toml`, `loom-gate.yml`, `CODEOWNERS` im selben Commit | S-31, S-11 | GL, GF (KV-Segment), GC (Loom-Test `loom_kv_deferred_zeroize` am neuen Ort grün), Mutation-Score G2 ≥ Baseline | Revert als Paar | ☐ offen |

## 3. Reihenfolge-Zwang [S]

> Reihenfolge-Zwang (Zyklenfreiheit [A]): `ids`,`env` vor allem E1; `mvcc` vor `vector`/`store`; `crypto`-Kern vor `store`; `crypto`-KV-Umzug nach `crypto`-Kern und gemeinsam mit `kvcache`; Lizenz-/Erasure-Ports (S-50) vor dem Entfernen von `FeatureRing` aus `store`/`kvcache`; `cognition` nach `engine`-Lizenzumstellung nicht vorher.

## 4. Definition of Done [S]

- G-L2, G-L1, G-L3, G-L4 grün (§8.5, R6). Bei fehlender Brücke: G-L2 entfällt.
- Für jeden Schritt aus Abschnitt 2 der Nachweis in der Spalte „Nachweis“ erbracht.
- Kein Eintrag in einer Golden-Datei geändert (INV-1), außer bewusst über ID-Breite (§5.2).
- Keine neuen `unwrap`/`expect`/`panic!` in Kern-Pfaden (R8), geprüft mit `tools/zero_panic_analyzer.py`.
- `tools/determinism_guard.py` meldet keinen neuen Befund (INV-5).
- Dokumentation: nur Ist-Aussagen, keine Compliance-Zusicherungen (INV-7, G-DOC1).

## 5. Offene Entscheidungen [S]

| D-06 | `legacy-erasure`-Dateien in `crypto` bleiben bis Nachweis (B10) |

## 6. Verweise

- Spezifikation: `SPEC.md` (dieses Verzeichnis)
- Normativ: `docs/SPEC.md` §7 (Reihenfolge), §8 (Gates), §9 (Entscheidungen, Risiken)
- Werkzeuge: `tools/gate_layering.py` (G-L1, nach S-01), `tools/isolation_proof.py` (G-L4), `tools/feature_audit.py` (S-12, nach S-12 vorhanden)


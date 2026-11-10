# ADR-0118: Anhebung forbid_unsafe_crates auf 32

**Status:** Angenommen
**Datum:** 10. November 2026
**Autor:** Principal Rust Engineer (Auftrag F0)
**Bezug:** SPEC v2 §8 (G-U1, B15), Schritt S-03, ratchet.toml
**Label:** `protected-path`

---

## Kontext
`governance/ratchet.toml` führt den Ratchet `forbid_unsafe_crates`. Bisheriger Wert: 30.
Mit FND-1 wurden die neuen Mikro-Kerne `contextra-ids` und `contextra-env` hinzugefügt, die beide strikt `#![forbid(unsafe_code)]` erzwingen (Ring 0 / Ebene 0).
Gemäß SPEC §8.4 G-U1 und B15 muss der Ratchet-Wert auf 32 angehoben werden (später mit prompt-guard und vault weiter auf 34).

## Entscheidung
`forbid_unsafe_crates` in `governance/ratchet.toml` wird von 30 auf 32 angehoben.

## Konsequenzen
- CI und Governance-Gates sichern ab, dass mindestens 32 Crates im Workspace `#![forbid(unsafe_code)]` tragen.

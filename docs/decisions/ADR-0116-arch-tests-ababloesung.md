# ADR-0116: Entfernung von tests/arch zugunsten kanonischer Gate-Infrastruktur

**Status:** Angenommen
**Datum:** 10. November 2026
**Autor:** Principal Rust Engineer (Auftrag F0)
**Bezug:** SPEC v2 §8 (Gates), B05, Schritt S-01, JULES_RAHMEN.md R0/R6
**Label:** `protected-path`

---

## Kontext
`tests/arch` (`contextra-arch-tests`) war ein separates Crate außerhalb der Workspace-Mitglieder (`[workspace] members` in `Cargo.toml`).
Es implementierte veraltete Prüfungen gegen ein statisches `RING_TABLE` (`docs/REDUCTION_SPEC.md` v1/v3), welches:
1. Den neuen Spezifikationsstand SPEC v2/v4 mit flachem Layout (`crates/contextra-*`), Schema v3 (`capabilities.toml`) und den neuen Mikro-Kernen `contextra-ids`, `contextra-env`, `contextra-prompt-guard` und `contextra-vault` nicht widerspiegelte.
2. Nicht in `cargo check --workspace` oder CI standardmäßig eingebunden war.
3. Redundant zu den kanonischen Prüf-Gates `tools/gate_layering.py` (Gate G-L1 / SPEC §8.1), `tools/isolation_proof.py` (Gate G-L4) und `tests/layering.rs` war.

## Entscheidung
1. `tests/arch` wird gelöscht, da die Ring- und Schichtungsregeln nun vollständig, dynamisch und durchsetzungsstark durch `tools/gate_layering.py` und den Test `tests/layering.rs` auf Basis von `capabilities.toml` (Schema v3) geprüft werden.
2. `tests/layering.rs` dient als Workspace-Integrationstest im Wurzelverzeichnis und scheitert mit Exit-Code 1 bei echten Schichtungsverletzungen.

## Konsequenzen
- Keine veralteten hardcodierten `RING_TABLE`-Definitionen mehr im Baum.
- Das neue Gate G-L1 (`tools/gate_layering.py`) ist die alleinige Source of Truth für Layering- und Closure-Integrität.

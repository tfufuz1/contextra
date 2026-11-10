# Abschlussbericht AUFTRAG F0 — Gate-Infrastruktur, Workspace-Gerüst, Root-Metadaten

**Datum:** 10. November 2026
**Rolle:** Principal Rust Engineer (Workspace-Tooling, Cargo, CI, Governance)
**Repository:** `tfufuz1/contextra`
**Commit-Basis:** HEAD `9305c5d04` (Merge PR #5925 / FND-1: `9a7c9cb`)
**Status:** Vollständig abgeschlossen

---

## 1. Gate-Matrix

| Gate | Befehl | Ergebnis |
|---|---|---|
| **G-L1 Layering** | `python3 tools/gate_layering.py --crate contextra-ids` | GRÜN (Exit 0) |
| **G-L1 Negativtest** | `python3 tests/test_gate_layering_negative.py` | GRÜN (Exit 1 bei absichtlicher verbotener Kante bewiesen) |
| **G-L1 Baseline-Lauf** | `python3 tools/gate_layering.py --all` | 34 Verstöße dokumentiert in `docs/decoupling/baseline/layering_violations.md` |
| **G-L4 Isolation** | `python3 tools/isolation_proof.py -c contextra-ids --modes no-default,default` | GRÜN (Exit 0) |
| **G-U1 Unsafe-Ratchet** | `governance/ratchet.toml` | GRÜN (forbid_unsafe_crates = 32 per ADR-0118) |
| **Workspace-Gerüste** | `cargo check -p contextra-prompt-guard --offline`<br>`cargo check -p contextra-vault --offline` | GRÜN (beide kompilieren fehlerfrei) |
| **Bridge-Refs Scan** | `python3 tools/count_bridge_refs.py` | GRÜN (Zählung in Baseline erfasst) |
| **Feature-Audit** | `python3 tools/feature_audit.py` | 99 ungenutzte Features erfasst als Fahrplan für S-12 |

---

## 2. Abweichungen von Spec / Prompt mit Begründung
- `cargo-public-api`: Installiert in Version `0.52.0`. Bei Toolchain 1.89 / Nightly 2024-11-01 erfordert `cpufeatures` das Feature `edition2024`, während Nightly 2025-08 mindestens ein neueres Rustdoc-Format verlangt. Das Gate in `governance/gates.toml` wurde auf `nightly-2026-11-09` gepinnt und mit `cargo +nightly public-api -p contextra-ids` erfolgreich lauffähig belegt.
- `Cargo.lock`: Da der bestehende HEAD `9305c5d04` unaufgelöste Abhängigkeitskonflikte in Alttests enthielt, wurde `Cargo.lock` gezielt und minimal um die beiden neuen Workspace-Gerüste `contextra-prompt-guard` und `contextra-vault` ergänzt, ohne fremde externe Abhängigkeiten zu aktualisieren.

---

## 3. Getroffene Entscheidungen (ADRs)
1. **ADR-0116:** Entfernung von `tests/arch` zugunsten kanonischer Gate-Infrastruktur (`tools/gate_layering.py`, `tests/layering.rs`). Label: `protected-path`.
2. **ADR-0117:** Erweiterung der CI-Workflows (`rust-ci.yml`, `tsan.yml`, `loom-gate.yml`, `mutation-testing.yml`) um Matrix-Prüfungen für `docid-128`, Layering-Gate und erweiterte Pfadfilter für `mvcc`, `kvcache`, `checkpoint`, `ids`, `env`. Label: `protected-path`.
3. **ADR-0118:** Anhebung von `forbid_unsafe_crates` in `governance/ratchet.toml` von 30 auf 32 (für `contextra-ids` und `contextra-env`). Label: `protected-path`.

---

## 4. Offene Punkte / Risiken
- Die geerbten Workspace-Kompilierfehler in `contextra-store`, `contextra-crypto` und `contextra-vector` bleiben am aktuellen Stand bestehen (wie im Auftrag vorgegeben: nur dokumentieren, nicht reparieren).
- In Schritt S-12 werden die 99 ungenutzten Features bereinigt.

---

## 5. Übergaben an Z (Fremdnutzer-Inventar & Default-Flips)
- **`from_key` Aufrufer:** 87 Aufrufstellen in Produktion (`docs/decoupling/baseline/workspace/from_key_prod.txt`).
- **Bridge-Referenzen:** 496 in `engine`, 238 in `store`, 200 in `vector`, 164 in `graph` müssen in den jeweiligen Folgeschritten (S-20 bis S-64) auf `ids`/`env` umgestellt werden.

---

## 6. Messwerte
- Neue Workspace-Mitglieder: +2 (`contextra-prompt-guard`, `contextra-vault`).
- Neue Werkzeuge: `tools/gate_layering.py`, `tools/isolation_proof.py` (G-L4), `tools/feature_audit.py`, `tools/count_bridge_refs.py`.
- Neuer Negativtest: `tests/test_gate_layering_negative.py`.

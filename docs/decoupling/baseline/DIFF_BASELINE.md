# Baseline Differenzdokumentation (S-00)

**Datum:** 10. November 2026
**Commit-Basis:** HEAD `9305c5d04` (Merge PR #5925 / FND-1: `9a7c9cb`)
**Vergleichsstände:**
- Spec-Stand: `ef3364f` (Entkopplungs-Spezifikation)
- FND-1 Basis: `9a7c9cb`
- Aktueller HEAD: `9305c5d04`

---

## 1. Status `cargo check --workspace --all-targets`
Am aktuellen HEAD schlagen mehrere Fremd-Crates fehl (dokumentiert gemäß Auftrag, nicht repariert):
- `contextra-store`: 10 Fehler (`builder.rs` move out of SstableBuilder Drop; `scanner/mod.rs` API-Mismatch zu `contextra-ports::scanner`)
- `contextra-crypto`: 2 Fehler (`unexpected cfg condition value: serde` in `src/secret.rs` unter toolchain 1.89)
- `contextra-vector`: Fehler bezüglich `HnswDeletionError` und Kernel-Definitionen

Diese Fehler bestanden bereits am Baseline-Stand von FND-1 und sind als geerbte Fremdfehler isoliert.

---

## 2. Inventar `::from_key(` Aufrufer
- **Gesamt:** 339 Fundstellen (`from_key_all.txt`)
- **Produktionscode (ohne `/tests/`):** 87 Fundstellen (`from_key_prod.txt`)
  - Vorherige Baseline (FND-1): 83 Fundstellen (Zuwachs um 4 Stellen im Kontext von Re-Exports/Tests)
- **Testcode (`/tests/`):** 252 Fundstellen (`from_key_tests.txt`)

---

## 3. Bridge-Referenzen (`tools/count_bridge_refs.py`)
Zählung der `contextra_(types|ports|testkit)`-Referenzen je Crate außerhalb von `compat/` und ohne Kommentarzeilen:

| Crate | Bridge-Refs (types\|ports\|testkit) |
|---|---|
| `contextra` | 27 |
| `contextra-adapt` | 0 |
| `contextra-agent` | 82 |
| `contextra-audit-export` | 4 |
| `contextra-avv-generator` | 0 |
| `contextra-checkpoint` | 77 |
| `contextra-cognition` | 37 |
| `contextra-crypto` | 28 |
| `contextra-db` | 15 |
| `contextra-durable-fs` | 0 |
| `contextra-engine` | 496 |
| `contextra-env` | 0 |
| `contextra-erasure` | 0 |
| `contextra-graph` | 164 |
| `contextra-ids` | 0 |
| `contextra-infer-candle` | 46 |
| `contextra-infer-ollama` | 57 |
| `contextra-infer-onnx` | 0 |
| `contextra-ingest` | 0 |
| `contextra-kvcache` | 57 |
| `contextra-license` | 30 |
| `contextra-mcp` | 32 |
| `contextra-mvcc` | 11 |
| `contextra-ports` | 4 |
| `contextra-privacy` | 0 |
| `contextra-py` | 7 |
| `contextra-rank` | 40 |
| `contextra-router` | 49 |
| `contextra-sandbox` | 0 |
| `contextra-simd` | 13 |
| `contextra-store` | 238 |
| `contextra-sys` | 0 |
| `contextra-testkit` | 50 |
| `contextra-text` | 80 |
| `contextra-types` | 0 |
| `contextra-vector` | 200 |
| `contextra-wire` | 0 |

---

## 4. Differenzen zu ef3364f und 9a7c9cb
1. **Neue Crates:** `contextra-ids` und `contextra-env` sind seit `9a7c9cb` im Workspace integriert und haben 0 Bridge-Referenzen.
2. **Archivierung & Bereinigung in `contextra-crypto`:** Ed25519-Dalek und Proof-Module wurden archiviert.
3. **Workspace-Manifest:** Resolver 2, Toolchain `1.89.0` gepinnt.

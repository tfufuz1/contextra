# BASELINE — contextra-store (ST6.0)

## 1. Ausgangszustand vor Beginn
- Branch: `jules/store-6-harden`
- Datum: 2026-11-10
- Commit-Basis: `9305c5d048cc6a4143d4e7cbf114cd85a6fca34a` (origin/main)
- Vorbestehende Compilerfehler in contextra-store behoben in eigenem Commit `ST6.0: Baseline-Reparatur`:
  - `block_cache.rs`: `SieveCacheBackend` Implementierung von `current_bytes` und `capacity_bytes`; Bereinigung doppelter Definitionen in `LruBlockCacheBackend`.
  - `builder.rs`: `file: Option<Box<dyn VfsFile>>` zur Vermeidung von `cannot move out of type SstableBuilder which implements Drop`.
  - `scanner/mod.rs`: `inspect_key` an aktuelle Signaturen von `ports::scanner::{ScanTarget, ScanTechnique, LayerWitness}` angepasst.
  - `compaction/tests/basic.rs`: `SstableBuilder::create` und `await` auf `add()` verwendet.
  - `Cargo.toml`: `required-features` fuer Tests mit `test-hooks` und `fault-injection` deklariert.

## 2. Messwerte Vorher (Baseline)
- LOC: 18.015 LOC in 83 Dateien (`python3 tools/loc.py --crate contextra-store --json`)
- Modularitaets-Score (`decouple_inspector.py`): 33/100
- Direkte interne Abhaengigkeiten (5): `contextra-crypto`, `contextra-mvcc`, `contextra-ports`, `contextra-sys`, `contextra-types`
- Kopplung importierte Symbole:
  - `contextra-types`: 8 Symbole
  - `contextra-ports`: 21 Symbole
  - `contextra-mvcc`: 5 Symbole
  - `contextra-crypto`: 6 Symbole
- Nullpruefzahlen (`grep -rEn 'contextra_(types|ports|testkit)'`):
  - Produktionscode (`src/` ohne `tests`): 150 Treffer
  - Test-Module in `src/`: 90 Treffer
  - Integrationstests (`tests/`): 233 Treffer

## 3. Feature-Befund
| Feature | Im Manifest deklariert | Im Quelltext genutzt (cfg) | Status / Befund |
|---|---|---|---|
| `default` | Ja (leer) | Nein | Standard |
| `test-hooks` | Ja | Ja (14 Stellen) | Aktiv genutzt fuer Test-Introspektion |
| `inspect` | Ja | Nein | Fuer neutrale Inspektionssicht vorbereitet |
| `memory-only-storage` | Ja | Ja (1 Stelle) | `engine/memory_only.rs` |
| `fault-injection` | Ja | Ja (40 Stellen) | Durability- & Crash-Testing |
| `block-cache-v2` | Ja | Ja (5 Stellen) | Schneller Block-Cache (`quick_cache`) |
| `docid-128` | Ja | Ja (3 Stellen) | Breitere Dokument-IDs |
| `legacy-wal-key` | Ja | Ja (7 Stellen) | Legacy HMAC Migration |
| `encryption-at-rest` | **Nein** | Ja (10 Stellen in wal/tests) | Im Code per cfg(feature = "encryption-at-rest") referenziert, aber nicht im Manifest deklariert (unexpected_cfgs). Bleibt unveraendert dokumentiert. |

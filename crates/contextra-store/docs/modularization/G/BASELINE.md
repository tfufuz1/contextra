# Baseline Report für STORE-MOD-G (contextra-store)

## 1. Ausgangszustand vor Eingriff (HEAD fcd3870 / 922ee26)
- **Manifest-Defekt:** `Cargo.toml` enthielt `workspace = ["dep:contextra-types", "dep:contextra-ports"]`, während `contextra-types` und `contextra-ports` unter `[dependencies]` nicht mit `optional = true` deklariert waren. Dies verhinderte das Laden des Manifests durch Cargo.
- **Fehlende Moduldeklarationen (WP-G.0a):** In `src/lib.rs` fehlten `pub mod budget;` und `pub mod stats;`, wodurch 13 Compiler-Fehler in Compaction, LSM und Recovery ausgelöst wurden.
- **Dead Code Lints:** `#![deny(dead_code)]` in `src/lib.rs` löst 6 Fehler im reinen Non-Default-Features-Build aus (`sstables_arc`, `rollback_to_tx`, `inner`, etc.), welche aktiv unter `workspace`/`test-hooks` genutzt werden.
- **Golden Corpus Status:**
  - `tests/golden/MANIFEST.sha256` verzeichnet:
    - `manifest_v1.manifest`
    - `rollback_tx_42.intent`
    - `sstable_legacy.sst`
    - `sstable_multiblock_bloom.sst`
    - `sstable_v4.sst`
    - `wal_with_hmac.log` (durch `.gitignore` geschützt)
  - Deserialisierungstest schlug fehl wegen falscher Schlüsseldateilänge in `.wal_integrity_key` (59 Bytes statt 32 Bytes).

## 2. Feature-Matrix (Baseline)
- `cargo check -p contextra-store --features workspace`: GRÜN (mit Warnings bzgl. fehlender Doku).
- `cargo check -p contextra-store --features "test-hooks,fault-injection,legacy-wal-key,memory-only-storage,workspace"`: GRÜN.

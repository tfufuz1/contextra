# AGENTS.md — `contextra-vault`

> **Ebene 1** · Reifegrad `beta` · Unsafe: nein (`#![forbid(unsafe_code)]`) · Rolle: Flüchtiger In-Memory Chunk-Tresor und Zeroization.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo check -p contextra-vault` |
| Abhängigkeiten | `contextra-sys`, `contextra-types` |
| Features | `default = []`, `workspace` |
| Lints | `#[forbid(unsafe_code)]` |

## 2. Modul-Karte

| Pfad | Verantwortung |
|---|---|
| `lib.rs` | `VolatileContextVault`, `VaultChunk`, `VaultConfig`, `VaultError` |
| `vault_sys.inc` | RAII-Sicherheitsverwaltung via contextra_sys |
| `vault_compat.inc` | Kompatibilitätsaliase (`PurgeReceipt`) |

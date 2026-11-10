# Fehlerabbildung: contextra-checkpoint (CheckpointError)

## Inventur der heute genutzten ContextraError-Varianten in contextra-checkpoint

| Lokale Variante (`CheckpointError`) | `ContextraError`-Variante | Display-Format | Verwendungsort |
|---|---|---|---|
| `InvalidInput(String)` | `InvalidInput(String)` | `"Invalid input: {0}"` | `meta.rs`, `manifest.rs`, `store.rs`, `hardlink_cloner.rs` |
| `Serialization(String)` | `Serialization(String)` | `"Serialization error: {0}"` | `manifest.rs`, `store.rs` |
| `Transaction(String)` | `Transaction(String)` | `"Transaction error: {0}"` | `guard.rs`, `store.rs` |
| `CheckpointNotFound` | `CheckpointNotFound` | `"Checkpoint not found"` | `store.rs` |
| `CrossDeviceLink { source_path, target_path }` | `CrossDeviceLink { source_path, target_path }` | `"Cross-device link failed: cannot hard link '{source_path}' to '{target_path}' across distinct filesystems/mounts"` | `hardlink_cloner.rs` |
| `Io(#[from] std::io::Error)` | `Io(std::io::Error)` | `"IO error: {0}"` | `hardlink_cloner.rs`, `orphan.rs` |
| `Internal(String)` | `Internal(String)` | `"Internal error: {0}"` | `guard.rs`, `store.rs`, `orphan.rs` |

## Konvertierung & Bitgleichheit
In `compat/error.rs` wird `From<CheckpointError> for ContextraError` implementiert.
Alle Display-Texte und Varianten entsprechen exakt den `ContextraError`-Äquivalenten.

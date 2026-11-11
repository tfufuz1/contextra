# Fehlertyp-Mapping: contextra-router

## 1. Übersicht der lokalen Fehlertypen

`contextra-router` definiert zwei lokale Fehlertypen und verwendet an Systemgrenzen `contextra_types::ContextraError`.

### ArmRegistryError
Definiert in `crates/contextra-router/src/arm_registry.rs:34`:
```rust
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ArmRegistryError {
    #[error("Ungültiger Arm-Index {0}, muss im Bereich 0..=4 liegen")]
    InvalidArmIndex(u32),
}
```

### FcTsDispatchError
Definiert in `crates/contextra-router/src/fc_ts_dispatch.rs:21` (hinter Feature `flow-corrected-thompson`):
```rust
#[derive(Debug, thiserror::Error)]
pub enum FcTsDispatchError {
    #[error("FC-TS arm set is empty or misconfigured: {0}")]
    ArmSet(#[from] FcTsError),
    #[error("profile/arm count mismatch: {profiles} profiles vs {arms} arms")]
    ProfileArmMismatch { profiles: usize, arms: usize },
}
```

## 2. Nutzung von ContextraError

Innerhalb von `RouterEngineBuilder`, `RouterEngine::route`, `RouterEngine::try_new_with_vfs` und `dispatch_to_slm` werden Fehler als `contextra_types::ContextraError` repräsentiert:

| Aufrufstelle | ContextraError-Variante | Ursache |
|---|---|---|
| `builder.rs:build` | `ContextraError::InvalidInput` | Fehlender Pflicht-Port (`search_provider`, `community_resolver`, `context_preparer`) |
| `profile.rs:validate` | `ContextraError::InvalidInput` | Ungültige Profilgrenzen (NaN, negative Scores, leere Namen) |
| `dispatch.rs:split_endpoint` | `ContextraError::InvalidInput` | Leerer MCP-Endpunkt oder ungeschlossene Anführungszeichen |
| `dispatch.rs:dispatch_to_slm` | `ContextraError::Internal` | Prozess-Spawn fehlgeschlagen, I/O-Fehler, JSON-RPC-Protokollfehler |
| `router/dispatch_core.rs` | `ContextraError::InvalidInput` | NaN in Query-Embedding oder ungültige Chunks |
| `router/dispatch_core.rs` | `ContextraError::NotFound` | Kein SLM-Profil erfüllt die Mindestkriterien (Fallback) |

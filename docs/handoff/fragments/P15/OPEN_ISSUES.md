# Offene Punkte P15

## Fremdfehler (außerhalb BESITZ)
1. `contextra-crypto`: Build schlägt fehl mit `error: unexpected 'cfg' condition value: 'serde'` in `crates/contextra-crypto/src/secret.rs:94` und `:105`. Crate liegt nicht in BESITZ von P15.
2. `contextra-graph`: 151 Compiler-Fehler wegen nicht aufgelöster `contextra_types` / `contextra_ports` Crate-Namen. Crate liegt nicht in BESITZ von P15.
3. Fehlendes Crate `xtask`: 21 Workflows und diverse Scripts/Justfiles referenzierten das gelöschte/nicht existierende Paket `xtask`. In `.github/workflows/` wurden sämtliche Referenzen eliminiert.

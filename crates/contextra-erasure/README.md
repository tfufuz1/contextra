# `contextra-erasure`

Modularer, autarker Löschkern für Contextra: Saga, FileVault, ReadGate/GateLog, RestoreGuard, Messung/Witness, DeletionReport (Ed25519), Revocation-Log und kryptografisches Shredding.

Entspricht **Klasse A** gemäß `docs/SPEC.md` v4.0 (B1–B12).

## Eigenschaften (Klasse A)

- **Keine internen Abhängigkeiten:** Keine Abhängigkeiten zu `contextra-types`, `contextra-ports` oder anderen internen Crates.
- **Autarkie:** Sämtliche Löschtypen (`Purger`, `LayerScanner`, `KeyScope`, `WitnessLayer`, `ErasureId`, `PurgeOutcome`) sind hier kanonisch definiert.
- **Zero-Panic (B9):** Keine `unwrap`/`expect`/`panic!` im Produktionscode. Mutex-Poisoning und I/O-Fehler führen zu deterministischem Fail-Closed.
- **Unsafe-Freiheit (B10):** Vollständig unter `#![forbid(unsafe_code)]` implementiert.
- **Standalone Export (B5):** Vollständig exportierbar und kompilierbar über `tools/export_crate.py contextra-erasure`.

## Minimalbeispiel (Export & Build)

Export in ein isoliertes Verzeichnis außerhalb des Workspaces:

```bash
python3 tools/export_crate.py contextra-erasure --out /tmp/export-contextra-erasure
cd /tmp/export-contextra-erasure
cargo check --no-default-features
```

## Verwendung

```rust
use contextra_erasure::types_local::{ErasureId, KeyScope, TenantId, CollectionId, DocId};
use contextra_erasure::ed25519_proof::DeletionProofKeyPair;

let keypair = DeletionProofKeyPair::generate();
let scope = KeyScope::Document {
    tenant: TenantId::new(1),
    collection: CollectionId::new(2),
    doc_id: DocId::new(100),
};
```

## Nicht-Ziele

- Keine Speicherung von Nutzdaten (Storage Engine gehört in `contextra-store`).
- Keine Index-Strukturen (Vektor-, Text- oder Graph-Indizes gehören in die jeweiligen Crates).
- Keine rechtlichen Behauptungen oder Garantien (siehe `docs/SPEC.md` §6.4).

# contextra

High-Level Fassade und Composition Root des Baukastensystems.

Normativitätsklasse: **Kern `[K]`** (Ring 4, `stable`; §6.0, `capabilities.toml`).

## Rolle & Zweck
`contextra` ist die übergeordnete Endanwender-Bibliothek und zentrale Composition Root (Ring 4). Sie bindet die einzelnen Lego-Bausteine des Ökosystems über das Builder-Muster (`ContextraBuilder`) zu einer gebrauchsfertigen Gesamtlösung zusammen.

## Baukasten-Einordnung & Status
- **Ring:** Ring 4 (Endanwender-Fassade)
- **Status:** 🟡 In Migration (Fassade wird an die neuen modularen Bausteine angebunden)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Baukasten-Nutzung
```rust
use contextra::ContextraBuilder;

let client = ContextraBuilder::new()
    .with_data_dir("./contextra_data")
    .build()
    .await?;
```

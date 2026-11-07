# contextra

High-Level-Hauptfassade und Composition Root des Contextra-Baukastensystems.

Normativitätsklasse: **Kern `[K]`** (Ring 4, `stable`; SPEC §4.4, §4.5, `capabilities.toml`).

## Rolle & Zweck
`contextra` ist die übergeordnete Endanwender-Bibliothek und zentrale Composition Root (Ring 4). Sie orchestriert die Teilsysteme (`contextra-db`, `contextra-ports`, `contextra-types`) über das Builder-Muster (`ContextraBuilder`) zu einer gebrauchsfertigen Gesamtlösung.

## Baukasten-Profil & Entkopplung (Paket B27)
- **Ring:** Ring 4 (Hauptfassade)
- **Minimale Abhängigkeiten:** Keine unbedingten Bindungen an Inferenz-Backends oder Lizenzmodule.
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]` und Zero-Panic-Doktrin im Produktionscode.
- **Optionale Features:**
  - `embedded` (Standard): Minimaler lokaler Betrieb ohne Inferenz und ohne Lizenzprüfung.
  - `license`: Kryptografische Signaturprüfung und Lizenzautorisierung (`SignedLicenseGate`).
  - `candle`: Lokale Inferenz via `contextra-infer-candle`.

## Baukasten-Nutzung

```rust
use contextra::builder::ContextraBuilder;
use contextra_types::DistanceMetric;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = ContextraBuilder::new(384)
        .with_storage_path("./contextra_data")
        .with_distance_metric(DistanceMetric::Cosine)
        .build()
        .await?;

    println!("Datenbank initialisiert, Dokumente: {}", db.len().await?);
    Ok(())
}
```

## Agent Memory
Für KI-Agenten stellt die Fassade die Abstraktion `AgentMemory` bereit:

```rust
use contextra::{builder, AgentMemory};

let db = builder(384).build().await?;
let memory = AgentMemory::new(db);

let id = memory.remember("Wichtiger Kontext", None).await?;
let results = memory.recall("Suchbegriff", 5).await?;
```

## Beispiel ausführen
Ein eigenständiges Minimalbeispiel befindet sich unter `examples/embedded_facade.rs`:

```bash
cargo run -p contextra --example embedded_facade
```

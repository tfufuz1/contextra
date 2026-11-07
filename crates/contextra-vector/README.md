# contextra-vector

Vektorsuche und Indexierung (HNSW, SQ8 Quantisierung).

Normativitätsklasse: **Kern `[K]`** (Ring 1 laut SPEC; §6.1, §6.9, §9.2, §9.3, `capabilities.toml`).

## Rolle & Zweck
`contextra-vector` ist der autarke Vektorindex-Baustein des Contextra-Ökosystems. Er bietet speichereffiziente Vektorsuche und Indexierung (Hierarchical Navigable Small World, HNSW) mit skalarer Quantisierung (SQ8) und Cosine-/Dot-Produkt-Distanzen.

Der Baustein kann isoliert in beliebigen Rust-Systemen als eigenständige Vektor-Engine betrieben werden.

## Baukasten-Einordnung & Status
- **Ring:** Ring 1 (Vektorindexierung)
- **Status:** 🟢 Autarker Baustein (Kopplungen entkoppelt, keine Abhängigkeit zu `contextra-crypto`)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`
- **Abhängigkeiten (Produktion):** `contextra-types`, `contextra-ports`, `contextra-simd`, `contextra-mvcc`, `contextra-sys`.

## Features
- `default`: Minimale Standardkonfiguration für HNSW und SQ8.
- `experimental-diskann`: Out-of-Core Graphindex für große Vektordatenmengen.
- `experimental-rabitq`: Experimentelle RaBitQ-Quantisierung.
- `experimental-predicate-augmented-search`: Prädikat-augmentierte Vektorsuche.
- `docid-128`: Unterstützung für 128-Bit Dokument-IDs via `contextra-types`.
- `partial-index-rebuild`: Partieller Rebuild (Achtung: VETO-F02 beachten).

## Einbettungsbeispiel

```rust
use contextra_ports::VectorIndex;
use contextra_types::{DocId, TxId};
use contextra_vector::hnsw::{HnswConfigBuilder, HnswIndex};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dimension = 4;
    let config = HnswConfigBuilder::new(dimension)
        .m(16)
        .ef_construction(32)
        .ef_search(16)
        .build()?;

    let index = HnswIndex::try_new(config)?;
    let tx = TxId::new(1);

    // Vektoren stufen
    index.insert(tx, DocId::new(1), &[1.0, 0.0, 0.0, 0.0]).await?;
    index.commit(tx).await?;

    // Nächste Nachbarn suchen
    let results = index.search(&[0.9, 0.1, 0.0, 0.0], 1).await?;
    println!("Gefunden: {:?}", results[0].doc_id);
    Ok(())
}
```

## Deprecated Debt & Entkopplung
- **Löschketten-Entkopplung:** Gemäß Entscheidung D-E1 verbleiben Lösch- und Nachweispflichten exklusiv in `contextra-erasure`.
- **Wartungsprimitive:** `contextra-vector` beschränkt sich auf Mandantenisolation, Tombstone-Pruning und Rebuild-Mechanismen.

# contextra-vector

Vektorsuche und Indexierung (HNSW, DiskANN, SQ8 Quantisierung).

Normativitätsklasse: **Klasse A – Autarker Baustein** (`capabilities.toml`, SPEC §4.3, §11.4).

## Rolle & Zweck
`contextra-vector` ist ein vollständig isolierter, kopierbarer Vektorindex-Baustein des Contextra-Ökosystems. Er bietet speichereffiziente Vektorsuche und Indexierung (Hierarchical Navigable Small World, HNSW) mit skalarer Quantisierung (SQ8) und konfigurierbaren Distanzmetriken (Kosinus, Euklidisch, Skalarprodukt).

Der Baustein kann isoliert in beliebigen Rust-Systemen als eigenständige Vektor-Engine betrieben werden.

## Baukasten-Profil (Klasse A)
- **Ring:** Ring 0 (autarkes Triebwerk)
- **Status:** 🟢 Vollständig entkoppelter autarker Baustein
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`
- **Zero-Panic-Doktrin:** Kein `unwrap()`, `expect()`, `panic!` im Produktionscode.
- **Abhängigkeiten (Produktion):** Reine Standard- und Third-Party-Bibliotheken (`ahash`, `parking_lot`, `thiserror`, `blake3`, `zeroize`, `roaring`). Keine internen Pflichtabhängigkeiten.

## Features
- `default = ["std"]`: Minimale Standardkonfiguration.
- `ports-compat`: Optionale Kompatibilitätsschicht für bestehende Contextra-Workspace-Kerne (`contextra-ports`, `contextra-types`, `contextra-mvcc`).
- `simd`: Optionales Hardware-SIMD über `contextra-simd`.
- `experimental-diskann`: Out-of-Core Graphindex für große Vektordatenmengen.
- `experimental-rabitq`: Experimentelle RaBitQ-Quantisierung.
- `experimental-predicate-augmented-search`: Prädikat-augmentierte Vektorsuche.
- `docid-128`: Unterstützung für 128-Bit Dokument-IDs.
- `partial-index-rebuild`: Partieller Rebuild (Wartungspfad gemäß SPEC §11.4 & VETO-F02 unter Wahrung von INV-VEC-CONNECTIVITY-1).

## Copy-Paste-Anleitung & Schnelleinstieg
Kopiere das Verzeichnis `crates/contextra-vector` direkt in dein Fremdprojekt oder binde es als Cargo-Abhängigkeit ein:

```toml
[dependencies]
contextra-vector = { version = "0.1", default-features = false, features = ["std"] }
```

### Minimalbeispiel (Index bauen, suchen, löschen, Maintain)

```rust
use contextra_vector::contract::{ExternalId, Metric, ScoredPoint};
use contextra_vector::hnsw::{HnswConfigBuilder, HnswIndex};
use contextra_vector::types::{DocId, TxId};

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

    // 1. Vektoren einfügen
    index.insert(tx, DocId::new(1), &[1.0, 0.0, 0.0, 0.0]).await?;
    index.commit(tx).await?;

    // 2. Ähnlichste Nachbarn suchen
    let results = index.search(&[0.9, 0.1, 0.0, 0.0], 1).await?;
    if let Some(top) = results.first() {
        println!("Top-Hit: DocId({}) mit Score: {}", top.doc_id.inner(), top.score);
    }

    // 3. Vektor löschen
    let tx_del = TxId::new(2);
    index.delete(tx_del, DocId::new(1)).await?;
    index.commit(tx_del).await?;

    // 4. Wartung & Kompaktierung
    index.compact()?;

    Ok(())
}
```

### Einbindungsmuster für SIMD via `DistanceFn`-Adapter

```rust
use contextra_vector::contract::DistanceFn;

pub struct CustomSimdDistance;

impl DistanceFn for CustomSimdDistance {
    fn distance(&self, a: &[f32], b: &[f32]) -> f32 {
        // Eigene SIMD-Optimierung (z.B. AVX2, AVX-512 oder NEON)
        contextra_vector::distance::cosine_distance_scalar(a, b)
    }
}
```

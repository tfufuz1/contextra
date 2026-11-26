//! Eigenständiges Beispiel für `contextra-vector`.
//!
//! Demonstriert die autarke Verwendung von HNSW-Indexierung und Vektorsuche
//! über den standardisierten `VectorIndex`-Port.

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

    // Vektoren einfügen (staged via MVCC TxBuffer)
    index
        .insert(tx, DocId::new(1), &[1.0, 0.0, 0.0, 0.0])
        .await?;
    index
        .insert(tx, DocId::new(2), &[0.0, 1.0, 0.0, 0.0])
        .await?;
    index
        .insert(tx, DocId::new(3), &[0.707, 0.707, 0.0, 0.0])
        .await?;

    // Transaktion committen, damit Vektoren in den HNSW-Graphen übergehen
    index.commit(tx).await?;

    // Nächste Nachbarn suchen
    let query = [0.9, 0.1, 0.0, 0.0];
    let results = index.search(&query, 2).await?;

    println!("Gefundene Treffer:");
    for candidate in results {
        println!(
            "  DocId: {:?}, Score: {}",
            candidate.doc_id, candidate.score
        );
    }

    Ok(())
}

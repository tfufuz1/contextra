//! Eigenständiges Beispiel für `contextra-graph` (M10).
//!
//! Zeigt den autarken Einsatz des CSR-Graphen für Traversierung und Pfadsuche
//! ohne externe Server, Dateisysteme oder höhere Ringe.

use contextra_graph::path_rag::PathRAGEngine;
use contextra_graph::CsrGraph;
use contextra_ports::GraphIndex;
use contextra_types::{Edge, Entity, EntityId, TxId};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Contextra Graph: Autarker Baustein ===");

    // 1. In-Memory CSR Graph instanziieren
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    let e_rust = EntityId::from("lang_rust");
    let e_systems = EntityId::from("topic_systems");
    let e_rag = EntityId::from("topic_rag");

    // 2. Entitäten anlegen
    graph
        .add_entity(tx, Entity::new(e_rust, "Rust", "Language"))
        .await?;
    graph
        .add_entity(tx, Entity::new(e_systems, "Systems Programming", "Field"))
        .await?;
    graph
        .add_entity(
            tx,
            Entity::new(e_rag, "Retrieval-Augmented Generation", "AI"),
        )
        .await?;

    // 3. Gerichtete Relationen mit Gewichten einfügen
    graph
        .add_edge(
            tx,
            Edge::new(e_rust, e_systems, "enables").with_weight(0.95),
        )
        .await?;
    graph
        .add_edge(tx, Edge::new(e_systems, e_rag, "powers").with_weight(0.85))
        .await?;

    graph.commit(tx).await?;

    // 4. Traversierung ausführen
    let neighbors = graph.neighbors(e_rust).await?;
    println!("Direkte Nachbarn von Rust: {neighbors:?}");

    // 5. PathRAG Multi-Hop Pfadsuche
    let engine = PathRAGEngine::new(graph, 3, 0.01);
    if let Some(path) = engine.find_path(e_rust, e_rag) {
        println!("Pfad gefunden: {:?}", path.nodes);
        println!("Konfidenz: {:.4}", path.confidence);
    } else {
        println!("Kein Pfad gefunden.");
    }

    Ok(())
}

//! Autarkes Minimalbeispiel für die `contextra`-Fassade (Ring 4).
//!
//! Demonstriert die Instanziierung von `Contextra` mittels `ContextraBuilder`,
//! das Speichern und Abrufen von Erinnerungen über `AgentMemory` sowie
//! grundlegendes Retrieval ohne externe Inferenz-Backends.

use contextra::builder::ContextraBuilder;
use contextra::AgentMemory;
use contextra_types::DistanceMetric;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = std::env::temp_dir().join(format!("contextra_example_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp_dir);

    println!("Contextra-Fassade: Initialisiere Datenbank am Pfad {:?}", tmp_dir);

    // 1. Initialisierung über den Fluent Builder
    let db = ContextraBuilder::new(4)
        .with_storage_path(&tmp_dir)
        .with_distance_metric(DistanceMetric::Cosine)
        .build()
        .await?;

    println!("Datenbank initialisiert. Dokumentenanzahl: {}", db.len().await?);

    // 2. Verwendung der agentenzentrierten Fassade AgentMemory
    let memory = AgentMemory::new(db);

    let mem_id = memory
        .remember(
            "Rust ist eine typsichere Systemprogrammiersprache.",
            Some(json!({"topic": "systems_programming"})),
        )
        .await?;
    println!("Erinnerung gespeichert mit ID: {}", mem_id);

    // 3. Retrieval über Volltextsuche
    let recalled = memory.recall("Systemprogrammiersprache", 5).await?;
    println!("Gefundene Treffer: {}", recalled.len());
    for item in recalled {
        println!(" - ID: {}, Score: {}", item.id, item.score);
    }

    // 4. Bereinigen
    let _ = std::fs::remove_dir_all(&tmp_dir);
    println!("Beispiel erfolgreich beendet.");
    Ok(())
}

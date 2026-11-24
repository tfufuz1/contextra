//! # Collections — Multi-Namespace Workflow
//!
//! # Zweck des Beispiels
//! Demonstriert die Verwaltung logisch getrennter Namensräume (Kollektionen / Namespaces)
//! innerhalb einer gemeinsamen physischen Datenbankinstanz:
//! - Jede Kollektion besitzt einen eigenen isolierten HNSW-Vektorindex und BM25-Volltextindex.
//! - Die physische Persistenzschicht (LSM-Storage und WAL) wird geteilt, jedoch strikt nach Mandanten und Kollektionspräfixen getrennt.
//! - Zeigt Snapshot-Isolation (MVCC / Multi-Version Concurrency Control): Historische Lesevorgänge sehen keine Daten,
//!   die nach dem Snapshot-Zeitpunkt eingefügt wurden.
//! - Zeigt das sichere Löschen einer Kollektion (`drop_collection`) mit Berechtigungs- und Schlüsselprüfungen.
//!
//! Ausführen mit: `cargo run --example collections`

#![allow(deprecated)]

use contextra_db::{Contextra, ContextraConfig};

// ANCHOR[contextra-db:EXAMPLE-COLLECTIONS-001]
/// Einstiegspunkt für die Demonstration mandantenfähiger, isolierter Kollektionen.
#[tokio::main]
async fn main() -> contextra_types::Result<()> {
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config("./collections_data", config).await?;

    // --- Zwei getrennte Kollektionen erzeugen ---
    let agents = db.collection("agents").await?;
    let tools = db.collection("tools").await?;

    // Dokumente in die "agents"-Kollektion einfügen
    agents
        .insert(
            "planner",
            &[1.0, 0.0, 0.0, 0.0],
            Some(serde_json::json!({"role": "planning", "text": "Decomposes complex tasks into subtasks"})),
        )
        .await?;

    agents
        .insert(
            "coder",
            &[0.0, 1.0, 0.0, 0.0],
            Some(serde_json::json!({"role": "execution", "text": "Writes and debugs code"})),
        )
        .await?;

    // Dokumente in die "tools"-Kollektion einfügen
    tools
        .insert(
            "web-search",
            &[0.5, 0.5, 0.0, 0.0],
            Some(serde_json::json!({"type": "retrieval", "text": "Searches the web for information"})),
        )
        .await?;

    tools
        .insert(
            "code-exec",
            &[0.0, 0.9, 0.1, 0.0],
            Some(serde_json::json!({"type": "execution", "text": "Executes code in a sandbox"})),
        )
        .await?;

    // --- Suche innerhalb jeder Kollektion (strikt isoliert) ---
    let query = [0.1, 0.9, 0.0, 0.0]; // "coding-like" vector

    let agent_results = agents.query().embedding(query).k(2).execute().await?;
    println!("=== Agents Collection ===");
    for r in &agent_results {
        println!("  {} (score: {:.4})", r.id, r.score);
    }

    let tool_results = tools.query().embedding(query).k(2).execute().await?;
    println!("\n=== Tools Collection ===");
    for r in &tool_results {
        println!("  {} (score: {:.4})", r.id, r.score);
    }

    // --- Alle existierenden Kollektionen auflisten ---
    let all = db.list_collections().await?;
    println!("\n=== All Collections ===");
    for name in &all {
        println!("  • {}", name);
    }

    // --- Snapshot-Isolation: Konsistenter Lesevorgang zu einem bestimmten Zeitpunkt ---
    let snapshot = db.create_snapshot().await?;
    println!("\n=== Snapshot created at seq={} ===", snapshot);

    // Weiteres Dokument NACH Erstellung des Snapshots einfügen
    agents
        .insert(
            "reviewer",
            &[0.5, 0.0, 0.5, 0.0],
            Some(serde_json::json!({"role": "review", "text": "Reviews code for correctness"})),
        )
        .await?;

    // Lesevorgang auf dem alten Snapshot sieht das neue Dokument NICHT (Snapshot Isolation)
    let at_snapshot = agents.get_at_snapshot("reviewer", snapshot).await?;
    println!(
        "  'reviewer' visible at snapshot? {}",
        if at_snapshot.is_some() {
            "yes"
        } else {
            "no (correct — inserted after snapshot)"
        }
    );

    // Regulärer aktueller Lesevorgang sieht das Dokument sofort
    let current = agents.get("reviewer").await?;
    println!(
        "  'reviewer' visible at current?  {}",
        if current.is_some() { "yes" } else { "no" }
    );

    // --- Kollektion wieder verwerfen (Drop) ---
    let tenant_id = contextra_types::TenantId::new(1);
    db.drop_collection("tools", tenant_id, &[0u8; 32]).await?;
    let remaining = db.list_collections().await?;
    println!("\n=== After dropping 'tools' ===");
    for name in &remaining {
        println!("  • {}", name);
    }

    db.close().await?;
    let _ = tokio::fs::remove_dir_all("./collections_data").await;

    println!("\n✅ Collections demo complete!");
    Ok(())
}

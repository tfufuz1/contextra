//! Eigenständiges Beispiel zur Demonstration der modularen Kognitions- und Konsolidierungsschicht.
//!
//! Zeigt die Initialisierung von `ContextManager`, `InMemoryCognitiveChannel`,
//! Vorprüfung von Compaction-Budgets und Intake von vorverdichteten Segmenten.

use contextra_cognition::aggregation_phase::{check_compaction_budget, AggregationConfig};
use contextra_cognition::channel::{CognitiveInputChannel, InMemoryCognitiveChannel};
use contextra_cognition::context::ContextManager;
use contextra_cognition::memory_consolidation::PreCondensedInput;
use contextra_types::{Result, TenantId};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== contextra-cognition Baukasten-Demo ===");

    // 1. ContextManager konfigurieren
    let manager = ContextManager::with_defaults();
    let estimated = ContextManager::estimate_tokens("Beispiel-Kontext für autonome Konsolidierung");
    println!("Geschätzte Token-Anzahl: {estimated}");

    // 2. Speicherbudget für Aggregation validieren
    let config = AggregationConfig {
        max_compaction_peak_memory_mb: 128,
        clustering_tau_threshold: 0.35,
        gmm_deterministic_seed: 42,
        max_llm_calls_per_cycle: 5,
        ..Default::default()
    };
    config.validate()?;
    check_compaction_budget(64 * 1024 * 1024, &config)?;
    println!("Speicherbudget-Vorprüfung erfolgreich.");

    // 3. Entkoppelten Cognitive Channel nutzen
    let (channel, mut rx) = InMemoryCognitiveChannel::new(16);
    let tenant_id = TenantId::new(1001);
    let input = PreCondensedInput {
        text_digest: "digest_1001_demo".to_string(),
        abstract_label: "Projektstatus Q4".to_string(),
        source_provenance: vec!["doc_42".to_string()],
        estimated_importance: 0.95,
        embedding: vec![0.1, 0.2, 0.3, 0.4],
    };

    let segments = channel.send_pre_condensed(&tenant_id, &[input]).await?;
    println!("Erzeugte Segmente über Channel: {}", segments.len());

    if let Some((tenant, inputs)) = rx.recv().await {
        println!(
            "Empfangen auf Channel für Mandant {}: {} Eingaben",
            tenant,
            inputs.len()
        );
    }

    println!("Baukasten-Demo erfolgreich beendet.");
    Ok(())
}

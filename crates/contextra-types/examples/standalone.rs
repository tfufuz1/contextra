//! Eigenständiges Anwendungsbeispiel für `contextra-types` (M10).
//!
//! Demonstriert die Verwendung von fundamentalen Domänentypen, Newtypes,
//! Ressourcenbudgets und hybriden Abfragen ohne externe Abhängigkeiten.

use contextra_types::{
    AutoExtractionMode, DistanceMetric, DocId, Embedding, FilterExpr, FusionStrategy,
    FusionWeights, HybridQueryBuilder, RequestContext, TenantId, TokenBudget, TxId,
};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== contextra-types Standalone Example ===");

    // 1. Identifikatoren erzeugen (Design by Contract, VETO-F10)
    let tenant = TenantId::try_new(42)?;
    let tx = TxId::new(1001);
    let ctx = RequestContext::new(tenant, tx);
    println!("RequestContext: Tenant={}, Tx={}", ctx.tenant_id, ctx.tx_id);

    // 2. Dokument-ID deterministisch aus Key ableiten
    let doc_id = DocId::from_key("user_profile_alice")?;
    println!("Deterministische DocId: {}", doc_id);

    // 3. Vektor-Embedding & Distanzmetrik
    let embedding = Embedding::new(vec![0.1, 0.2, 0.3, 0.4]);
    let metric = DistanceMetric::Cosine;
    println!(
        "Embedding Dimension: {}, Metrik: {:?}",
        embedding.dim(),
        metric
    );

    // 4. Token- und Ressourcen-Budgetierung
    let budget = TokenBudget::new(8192, 512);
    budget.try_reserve(256)?;
    println!("Reservierte 256 Tokens, Verfügbar: {}", budget.available());

    // 5. Strukturierter Filterausdruck
    let filter = FilterExpr::And(
        Box::new(FilterExpr::Eq {
            field: "status".to_string(),
            value: json!("active"),
        }),
        Box::new(FilterExpr::Gt {
            field: "priority".to_string(),
            value: json!(5),
        }),
    );
    println!("Filterausdruck gebaut: {:?}", filter);

    // 6. Hybride Suchabfrage (4-Signal Fusion)
    let query = HybridQueryBuilder::new()
        .with_text_query("autonome Agenten-Architektur")
        .with_vector_query(vec![0.1, 0.2, 0.3, 0.4])
        .with_fusion_strategy(FusionStrategy::Rrf)
        .with_fusion_weights(FusionWeights::new(0.6, 0.4, 0.0)?)
        .with_filter(filter)
        .with_k(10)
        .build()?;

    println!("HybridQuery erfolgreich konstruiert für K={}", query.k);
    println!(
        "AutoExtractionMode Default: {:?}",
        AutoExtractionMode::default()
    );

    println!("=== Standalone Beispiel erfolgreich durchlaufen ===");
    Ok(())
}

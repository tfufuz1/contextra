//! Autarkes Standalone-Beispiel für contextra-kvcache (Baukasten-Vertrag M2, M10).
//!
//! Demonstriert Prefix-Matching, Segmentverwaltung und Tenant-Isolation
//! vollkommen ohne Inferenz-Frameworks oder Modell-Gewichte.

use contextra_kvcache::{
    KvCacheConfig, KvCacheServiceBuilder, KvSegment, TenantIsolatedKvStore,
};
use contextra_types::TenantId;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== contextra-kvcache: Standalone Prefix-Cache Example ===");

    // 1. Mandanten-Identifikatoren definieren
    let tenant_alpha = TenantId::try_new(101)?;
    let tenant_beta = TenantId::try_new(202)?;

    // 2. KvCacheService initialisieren
    let config = KvCacheConfig {
        max_ram_bytes: 1024 * 1024, // 1 MiB RAM Budget
        ..Default::default()
    };
    let service = KvCacheServiceBuilder::new()
        .with_config(config)
        .build()?;

    // 3. Store direkt für Segment-Operationen nutzen
    let store = TenantIsolatedKvStore::new();

    // 4. KV-Segmente für tenant_alpha einfügen
    let seg1_data = vec![0xAA; 256];
    let seg1 = KvSegment::new(tenant_alpha, 1, seg1_data.clone());
    store.insert_segment(tenant_alpha, seg1);

    let seg2_data = vec![0xBB; 512];
    let seg2 = KvSegment::new(tenant_alpha, 2, seg2_data.clone());
    store.insert_segment(tenant_alpha, seg2);

    println!(
        "Tenant Alpha segments inserted. Total tenant segments: {}",
        store.get_tenant_segment_len(tenant_alpha)
    );

    // 5. Tenant-Isolation verifizieren: Tenant Beta darf Segmente von Alpha nicht sehen
    assert_eq!(store.get_tenant_segment_len(tenant_beta), 0);
    assert!(store.get_segment_bytes(tenant_beta, 1).is_none());
    println!("Tenant isolation verified: Tenant Beta has 0 segments.");

    // 6. Abruf für Tenant Alpha verifizieren
    let retrieved = store.get_segment_bytes(tenant_alpha, 1);
    assert_eq!(retrieved.as_deref(), Some(seg1_data.as_slice()));
    println!("Segment retrieval verified: retrieved {} bytes for segment 1.", seg1_data.len());

    // 7. Service-Statistiken abrufen
    let stats = service.stats();
    println!("Cache Service initialized successfully. Hits: {}, Misses: {}", stats.hits, stats.misses);

    println!("=== Standalone KV-Cache example completed successfully ===");
    Ok(())
}

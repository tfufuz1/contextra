// STAND: 2026-10-17
// ZWECK: Keyspace-Konformitätstest & KEYFAMILIES.toml Sync-Test (INV-D1).

use contextra_engine::deletion::{
    reverse_sweep, ErasureFootprint, FprNeedles, KeyFamilyRegistry, SweepDepth, ALL_KEY_FAMILIES,
};
use contextra_engine::{Contextra, ContextraConfig};
use contextra_ports::StorageEngine;
use contextra_store::tenant_codec::TenantScopedStorage;
use contextra_types::{CollectionId, DocId, TenantId};
use serde::Deserialize;
use std::fs;
use tempfile::TempDir;

#[derive(Debug, Deserialize)]
struct KeyFamilyTomlEntry {
    name: String,
    prefix_pattern: String,
    owner_crate: String,
    erase_unit: String,
    scanner: String,
    contains_identifier: bool,
}

#[derive(Debug, Deserialize)]
struct KeyFamiliesToml {
    family: Vec<KeyFamilyTomlEntry>,
}

#[tokio::test]
async fn test_keyfamilies_toml_sync() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let toml_path = format!("{}/../../docs/deletion/KEYFAMILIES.toml", manifest_dir);
    let toml_content = fs::read_to_string(&toml_path)
        .unwrap_or_else(|_| panic!("Failed to read {}", toml_path));

    let parsed: KeyFamiliesToml =
        toml::from_str(&toml_content).expect("Failed to parse docs/deletion/KEYFAMILIES.toml");

    let registry_families = ALL_KEY_FAMILIES;

    assert_eq!(
        parsed.family.len(),
        registry_families.len(),
        "KEYFAMILIES.toml count ({}) does not match ALL_KEY_FAMILIES count ({})",
        parsed.family.len(),
        registry_families.len()
    );

    for (toml_entry, reg_entry) in parsed.family.iter().zip(registry_families.iter()) {
        assert_eq!(toml_entry.name, reg_entry.name);
        assert_eq!(toml_entry.prefix_pattern, reg_entry.prefix_pattern);
        assert_eq!(toml_entry.owner_crate, reg_entry.owner_crate);
        assert_eq!(
            toml_entry.erase_unit,
            format!("{:?}", reg_entry.erase_unit)
        );
        assert_eq!(toml_entry.scanner, reg_entry.scanner);
        assert_eq!(
            toml_entry.contains_identifier,
            reg_entry.contains_identifier
        );
    }
}

#[tokio::test]
async fn test_keyspace_conformance_all_collections_and_tenants() -> contextra_types::Result<()> {
    let tmp = TempDir::new().map_err(|e| contextra_types::ContextraError::InvalidInput(e.to_string()))?;
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;

    let default_col = db.collection("default").await?;
    let custom_col = db.collection("custom_col").await?;

    let doc_id_1 = "doc-default-001";
    let doc_id_2 = "doc-custom-001";

    // 1. Insert documents into default & custom collections with text metadata
    default_col
        .insert(
            doc_id_1,
            &[0.1, 0.2, 0.3, 0.4],
            Some(serde_json::json!({ "text": "conformance document default" })),
        )
        .await?;

    custom_col
        .insert(
            doc_id_2,
            &[0.5, 0.6, 0.7, 0.8],
            Some(serde_json::json!({ "text": "conformance document custom" })),
        )
        .await?;

    // 2. Relate documents
    default_col
        .insert("doc-default-002", &[0.1, 0.2, 0.3, 0.4], None)
        .await?;
    default_col
        .relate(doc_id_1, "doc-default-002", "references")
        .await?;

    custom_col
        .insert("doc-custom-002", &[0.5, 0.6, 0.7, 0.8], None)
        .await?;
    custom_col
        .relate(doc_id_2, "doc-custom-002", "references")
        .await?;

    let parsed_doc_1 = DocId::from_key(doc_id_1)?;
    let parsed_doc_2 = DocId::from_key(doc_id_2)?;

    let fp_1 = default_col.erasure_footprint(parsed_doc_1).await?;
    let fp_2 = custom_col.erasure_footprint(parsed_doc_2).await?;

    // 3. Scan all raw storage keys
    let raw_entries = default_col.storage().scan_prefix(b"").await?;

    let mut missing_keys_1 = Vec::new();
    let mut missing_keys_2 = Vec::new();

    for (raw_key, _) in &raw_entries {
        let key_str = String::from_utf8_lossy(raw_key);

        if key_str.contains(doc_id_1) || raw_key.windows(doc_id_1.len()).any(|w| w == doc_id_1.as_bytes()) {
            if !fp_1.matches(raw_key) {
                missing_keys_1.push(key_str.to_string());
            }
        }

        if key_str.contains(doc_id_2) || raw_key.windows(doc_id_2.len()).any(|w| w == doc_id_2.as_bytes()) {
            if !fp_2.matches(raw_key) {
                missing_keys_2.push(key_str.to_string());
            }
        }
    }

    assert!(
        missing_keys_1.is_empty(),
        "Footprint 1 missed storage keys for doc_id '{}': {:?}",
        doc_id_1,
        missing_keys_1
    );

    assert!(
        missing_keys_2.is_empty(),
        "Footprint 2 missed storage keys for doc_id '{}': {:?}",
        doc_id_2,
        missing_keys_2
    );

    // 4. Test TenantScopedStorage keyspace
    let tenant_id = TenantId::try_new(777)?;
    let _tenant_storage = TenantScopedStorage::new(default_col.storage().clone(), tenant_id);

    // Verify tenant footprint
    let tenant_fp = ErasureFootprint::fuer_tenant(tenant_id);
    let codec = contextra_store::tenant_codec::TenantKeyCodec::new(tenant_id);
    let tenant_key_prefix = codec.scan_prefix();
    assert!(
        tenant_key_prefix.starts_with(b"t:777:"),
        "Tenant key prefix must be 't:777:'"
    );

    assert!(
        tenant_fp.matches(b"__col:test:\x00doc1"),
        "Tenant footprint must cover __col:"
    );

    db.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_reverse_sweep_detects_unregistered_residue() -> contextra_types::Result<()> {
    let doc_id = DocId::new(9999);
    let doc_id_str = doc_id.to_string();

    let footprint = ErasureFootprint::fuer_document(TenantId::SYSTEM, "default", doc_id);
    let registry = KeyFamilyRegistry::new();

    // Intentionally forgotten/unregistered family prefix
    let unregistered_key = format!("__unregistered_family:{}", doc_id_str).into_bytes();
    let registered_key = format!("__rel:{}", doc_id_str).into_bytes();

    let sweep_source = vec![unregistered_key, registered_key];

    let needles = FprNeedles {
        doc_id_bytes: doc_id_str.as_bytes().to_vec(),
        collection_id: Some(CollectionId::new(1)),
        collection_name: "default".to_string(),
        tenant_prefix: Vec::new(),
    };

    let report = reverse_sweep(
        &sweep_source,
        &needles,
        &footprint,
        &registry,
        SweepDepth::Standard,
    )?;

    assert_eq!(
        report.unregistered_residues.len(),
        1,
        "Reverse sweep MUST detect exactly 1 unregistered residue"
    );

    let residue = &report.unregistered_residues[0];
    assert_eq!(
        residue.family_guess,
        Some("__unregistered_family".to_string()),
        "Family guess must match '__unregistered_family'"
    );

    Ok(())
}

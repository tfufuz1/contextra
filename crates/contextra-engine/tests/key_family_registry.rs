// Integration test for KeyFamilyRegistry and ErasureFootprint (WP-1.1).

#[path = "../src/deletion/mod.rs"]
mod deletion;

use contextra_types::{CollectionId, DocId, TenantId};
use deletion::*;

#[test]
fn test_keyfamilies_toml_matches_rust_registry() {
    let toml_content = include_str!("../../../docs/deletion/KEYFAMILIES.toml");
    let mut toml_family_names = Vec::new();
    for line in toml_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("name = ") {
            let name = trimmed
                .strip_prefix("name = \"")
                .and_then(|s| s.strip_suffix('"'))
                .unwrap_or("");
            assert!(!name.is_empty(), "Failed to parse name from line: {line}");
            toml_family_names.push(name.to_string());
        }
    }

    let registry = KeyFamilyRegistry::new();
    let rust_families = registry.all_families();
    let rust_names: Vec<String> = rust_families.iter().map(|f| f.name.to_string()).collect();

    assert_eq!(
        toml_family_names, rust_names,
        "KEYFAMILIES.toml entries must match ALL_KEY_FAMILIES in registry.rs"
    );
}

#[test]
fn test_all_families_reachable_via_enumerate() {
    let registry = KeyFamilyRegistry::new();

    let tenant_id = TenantId::try_new(1).unwrap();
    let collection_id = CollectionId::new(10);
    let doc_id = DocId::new(100);

    // Tenant scope returns ALL families
    let tenant_scope = DeletionScopeRef::Tenant { tenant_id };
    let tenant_families = registry.enumerate(&tenant_scope);
    assert_eq!(tenant_families.len(), registry.all_families().len());

    // Collection scope
    let col_scope = DeletionScopeRef::Collection {
        tenant_id,
        collection_id,
        collection_name: "test_col",
    };
    let col_families = registry.enumerate(&col_scope);
    for fam in &col_families {
        assert!(
            matches!(fam.erase_unit, EraseUnit::PerCollection | EraseUnit::Both),
            "Collection scope must only include PerCollection or Both families"
        );
    }
    assert!(col_families.iter().any(|f| f.name == "CollectionDocData"));
    assert!(col_families.iter().any(|f| f.name == "CollectionIndex"));
    assert!(col_families.iter().any(|f| f.name == "TextIndex"));

    // Document scope
    let doc_scope = DeletionScopeRef::Document {
        tenant_id,
        collection_id,
        collection_name: "test_col",
        doc_id,
    };
    let doc_families = registry.enumerate(&doc_scope);
    for fam in &doc_families {
        assert!(
            matches!(fam.erase_unit, EraseUnit::PerDocument | EraseUnit::Both),
            "Document scope must only include PerDocument or Both families"
        );
    }
    assert!(doc_families.iter().any(|f| f.name == "CollectionDocData"));
    assert!(doc_families.iter().any(|f| f.name == "TextIndex"));
    assert!(doc_families.iter().any(|f| f.name == "Relations"));
    assert!(doc_families.iter().any(|f| f.name == "DocIdMapping"));
}

#[test]
fn test_encode_prefix_property() {
    let registry = KeyFamilyRegistry::new();

    // encode_prefix for "__col:" or "CollectionDocData" with example name
    let sample_col = "my_sample_collection";
    let encoded_by_alias = registry.encode_prefix("__col:", &[sample_col]);
    let encoded_by_name = registry.encode_prefix("CollectionDocData", &[sample_col]);

    assert!(encoded_by_alias.is_some());
    assert!(encoded_by_name.is_some());

    let prefix_alias = encoded_by_alias.unwrap();
    let prefix_name = encoded_by_name.unwrap();

    assert_eq!(prefix_alias, prefix_name);
    assert!(
        prefix_alias.starts_with(b"__col:"),
        "Encoded prefix must start with b\"__col:\""
    );
    assert!(
        prefix_alias.starts_with(format!("__col:{}:", sample_col).as_bytes()),
        "Encoded prefix must include collection name"
    );

    // Test encode_prefix for all registered families
    for family in registry.all_families() {
        let encoded = registry.encode_prefix(family.name, &["test_param"]);
        assert!(
            encoded.is_some(),
            "encode_prefix must return Some for known family {}",
            family.name
        );
    }
}

#[test]
fn test_erasure_footprint_compute_hash() {
    let tenant_1 = TenantId::try_new(1).unwrap();
    let tenant_2 = TenantId::try_new(2).unwrap();

    let key_scope_1 = KeyScope {
        explicit: vec![b"__col_idx:\x00col_a".to_vec()],
        prefixes: vec![b"__col:col_a:\x00".to_vec()],
    };

    let footprint_1a = ErasureFootprint::new(StorageScope::Tenant(tenant_1), key_scope_1.clone());
    let footprint_1b = ErasureFootprint::new(StorageScope::Tenant(tenant_1), key_scope_1);

    // Deterministic: Identical footprints yield identical hash
    assert_eq!(footprint_1a.hash, footprint_1b.hash);
    assert_eq!(footprint_1a.compute_hash(), footprint_1b.compute_hash());

    // Difference in StorageScope
    let key_scope_2 = KeyScope {
        explicit: vec![b"__col_idx:\x00col_a".to_vec()],
        prefixes: vec![b"__col:col_a:\x00".to_vec()],
    };
    let footprint_tenant2 = ErasureFootprint::new(StorageScope::Tenant(tenant_2), key_scope_2);

    assert_ne!(
        footprint_1a.hash, footprint_tenant2.hash,
        "Footprints with different TenantIds must yield different hashes"
    );

    // Difference in KeyScope
    let key_scope_diff = KeyScope {
        explicit: vec![b"__col_idx:\x00col_b".to_vec()],
        prefixes: vec![b"__col:col_b:\x00".to_vec()],
    };
    let footprint_diff_keys = ErasureFootprint::new(StorageScope::Tenant(tenant_1), key_scope_diff);

    assert_ne!(
        footprint_1a.hash, footprint_diff_keys.hash,
        "Footprints with different KeyScopes must yield different hashes"
    );
}

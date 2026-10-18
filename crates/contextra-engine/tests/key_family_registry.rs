// Integration test for KeyFamilyRegistry and ErasureFootprint (WP-1.1).

use contextra_engine::deletion::*;
use contextra_types::{CollectionId, DocId, TenantId};

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

    // Scanner field check: No scanner may be "TODO" or empty
    for family in rust_families {
        assert!(
            !family.scanner.is_empty() && family.scanner != "TODO",
            "Scanner field for family '{}' must not be empty or 'TODO', got '{}'",
            family.name,
            family.scanner
        );
    }
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
fn test_erasure_footprint_compute_hash_and_constructors() {
    let tenant_1 = TenantId::try_new(1).unwrap();
    let tenant_2 = TenantId::try_new(2).unwrap();
    let doc_id = DocId::new(42);

    let doc_fpr = ErasureFootprint::for_document(tenant_1, "col_a", doc_id);
    let col_fpr = ErasureFootprint::for_collection(tenant_1, "col_a");
    let tenant_fpr = ErasureFootprint::for_tenant(tenant_1);

    assert_ne!(doc_fpr.hash, col_fpr.hash);
    assert_ne!(col_fpr.hash, tenant_fpr.hash);

    let doc_fpr_2 = ErasureFootprint::for_document(tenant_2, "col_a", doc_id);
    assert_ne!(doc_fpr.hash, doc_fpr_2.hash);

    // Zeroization test
    let mut doc_fpr_mut = doc_fpr.clone();
    doc_fpr_mut.zeroize_contents();
    assert!(doc_fpr_mut.keys.explicit.is_empty());
    assert!(doc_fpr_mut.keys.prefixes.is_empty());
    assert!(doc_fpr_mut.ram.doc_ids.is_empty());
}

#[test]
fn test_check_nonempty_or_observed_empty() {
    assert!(check_nonempty_or_observed_empty(0, 0).is_ok());
    assert!(check_nonempty_or_observed_empty(10, 5).is_ok());

    let err = check_nonempty_or_observed_empty(10, 0);
    assert!(
        err.is_err(),
        "Must return Err when handle contained entries but 0 were deleted"
    );
}

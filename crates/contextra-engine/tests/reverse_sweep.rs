// Integration tests for Reverse-Sweep subsystem (WP-1.1, C1.4).

use contextra_engine::deletion::*;
use contextra_types::{CollectionId, ContextraError, TenantId};

struct VecSweepSource {
    keys: Vec<Vec<u8>>,
}

impl FprSweepSource for VecSweepSource {
    fn for_each_key(&self, f: &mut dyn FnMut(&[u8]) -> bool) -> Result<(), ContextraError> {
        for k in &self.keys {
            if !f(k) {
                break;
            }
        }
        Ok(())
    }
}

struct Synthetic1MSource {
    count: usize,
}

impl FprSweepSource for Synthetic1MSource {
    fn for_each_key(&self, f: &mut dyn FnMut(&[u8]) -> bool) -> Result<(), ContextraError> {
        for i in 0..self.count {
            let key = format!("__col:test_col:\x00doc_{i}").into_bytes();
            if !f(&key) {
                break;
            }
        }
        Ok(())
    }
}

#[test]
fn test_injected_unregistered_key_found() {
    let registry = KeyFamilyRegistry::new();
    let tenant_id = TenantId::try_new(1).unwrap();
    let footprint = ErasureFootprint::for_collection(tenant_id, "test_col");

    let needles = FprNeedles {
        doc_id_bytes: b"doc_999".to_vec(),
        collection_id: Some(CollectionId::new(10)),
        collection_name: "test_col".to_string(),
        tenant_prefix: Vec::new(),
    };

    let source = VecSweepSource {
        keys: vec![
            b"__foo:doc_999".to_vec(), // Injected unregistered key
        ],
    };

    let report = reverse_sweep(
        &source,
        &needles,
        &footprint,
        &registry,
        SweepDepth::Standard,
    )
    .unwrap();

    assert_eq!(report.scanned_keys_count, 1);
    assert_eq!(report.unregistered_residues.len(), 1);
    assert_eq!(
        report.unregistered_residues[0].family_guess,
        Some("__foo".to_string())
    );
}

#[test]
fn test_registered_deleted_footprint_keys_produce_no_residue() {
    let registry = KeyFamilyRegistry::new();
    let tenant_id = TenantId::try_new(1).unwrap();
    let footprint = ErasureFootprint::for_collection(tenant_id, "test_col");

    let needles = FprNeedles {
        doc_id_bytes: Vec::new(),
        collection_id: Some(CollectionId::new(10)),
        collection_name: "test_col".to_string(),
        tenant_prefix: Vec::new(),
    };

    let source = VecSweepSource {
        keys: vec![
            b"__col:test_col:\x00doc_123".to_vec(),
            b"__txt:test_col:terms".to_vec(),
            b"__col_idx:\x00test_col".to_vec(),
        ],
    };

    let report = reverse_sweep(
        &source,
        &needles,
        &footprint,
        &registry,
        SweepDepth::Standard,
    )
    .unwrap();

    assert_eq!(report.scanned_keys_count, 3);
    assert_eq!(
        report.unregistered_residues.len(),
        0,
        "Keys covered by footprint must not produce unregistered residues"
    );
}

#[test]
fn test_short_needle_produces_ambiguous_match() {
    let registry = KeyFamilyRegistry::new();
    let tenant_id = TenantId::try_new(1).unwrap();

    // Empty footprint so key is outside
    let footprint = ErasureFootprint::new(StorageScope::Tenant(tenant_id), KeyScope::default());

    let short_doc_id = b"short_id".to_vec(); // 8 bytes < 16 bytes
    assert!(short_doc_id.len() < 16);

    let needles = FprNeedles {
        doc_id_bytes: short_doc_id,
        collection_id: None,
        collection_name: String::new(),
        tenant_prefix: Vec::new(),
    };

    let source = VecSweepSource {
        keys: vec![
            b"custom_prefix_short_id_suffix".to_vec(), // Matches needle as pure substring
        ],
    };

    let report = reverse_sweep(
        &source,
        &needles,
        &footprint,
        &registry,
        SweepDepth::Standard,
    )
    .unwrap();

    assert_eq!(report.scanned_keys_count, 1);
    assert_eq!(
        report.ambiguous_matches_count, 1,
        "Short needle match outside registered pattern must yield AmbiguousMatch"
    );
}

#[test]
fn test_streaming_1_million_synthetic_keys() {
    let registry = KeyFamilyRegistry::new();
    let tenant_id = TenantId::try_new(1).unwrap();
    let footprint = ErasureFootprint::for_collection(tenant_id, "test_col");

    let needles = FprNeedles {
        doc_id_bytes: b"doc_500000".to_vec(),
        collection_id: Some(CollectionId::new(10)),
        collection_name: "test_col".to_string(),
        tenant_prefix: Vec::new(),
    };

    let source = Synthetic1MSource { count: 1_000_000 };

    let report = reverse_sweep(
        &source,
        &needles,
        &footprint,
        &registry,
        SweepDepth::Standard,
    )
    .unwrap();

    assert_eq!(report.scanned_keys_count, 1_000_000);
    assert_eq!(
        report.unregistered_residues.len(),
        0,
        "All synthetic keys were covered by collection footprint"
    );
}

#[test]
fn test_registry_completeness() {
    let registry = KeyFamilyRegistry::new();
    let rust_families = registry.all_families();

    for family in rust_families {
        assert!(
            !family.scanner.is_empty() && family.scanner != "TODO",
            "Scanner must not be TODO or empty for {}",
            family.name
        );
    }
}

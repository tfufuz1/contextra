//! Differential test between acta-ids and frozen legacy implementations (SPEC-v2 §4.1 Abnahme 2)

use acta_ids::DocId;

#[path = "legacy_reference/types_legacy.rs"]
mod types_legacy;
#[path = "legacy_reference/vector_legacy.rs"]
mod vector_legacy;

#[test]
fn test_differential_over_10k_keys() {
    let long_s = "X".repeat(4096);
    let edge_cases = vec![
        "",
        "a",
        "Hello, World!",
        "Mandant_Übersicht_42_🚀",
        long_s.as_str(),
    ];

    for key in edge_cases {
        let actual = DocId::from_key(key);
        #[cfg(not(feature = "docid-128"))]
        {
            let expected_types = types_legacy::doc_id_from_key_64(key);
            let expected_vec = vector_legacy::vector_doc_id_from_key(key);
            match actual {
                Ok(id) => {
                    assert_eq!(id.inner(), expected_types.unwrap());
                    assert_eq!(id.inner(), expected_vec.unwrap());
                }
                Err(e) => {
                    assert_eq!(e.to_string(), expected_types.unwrap_err());
                    assert_eq!(e.to_string(), expected_vec.unwrap_err());
                }
            }
        }
        #[cfg(feature = "docid-128")]
        {
            let expected_types = types_legacy::doc_id_from_key_128(key);
            match actual {
                Ok(id) => {
                    assert_eq!(id.inner(), expected_types.unwrap());
                }
                Err(e) => {
                    assert_eq!(e.to_string(), expected_types.unwrap_err());
                }
            }
        }
    }

    // 10,000 deterministic generated keys
    for i in 0..10_000 {
        let key = format!("generated:test:key:number:{}:suffix_abc", i);
        let actual = DocId::from_key(&key).expect("non-empty key succeeds");
        #[cfg(not(feature = "docid-128"))]
        {
            let exp_types = types_legacy::doc_id_from_key_64(&key).unwrap();
            let exp_vec = vector_legacy::vector_doc_id_from_key(&key).unwrap();
            assert_eq!(actual.inner(), exp_types);
            assert_eq!(actual.inner(), exp_vec);
        }
        #[cfg(feature = "docid-128")]
        {
            let exp_types = types_legacy::doc_id_from_key_128(&key).unwrap();
            assert_eq!(actual.inner(), exp_types);
        }
    }
}

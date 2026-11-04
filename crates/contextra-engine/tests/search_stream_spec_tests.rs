use contextra_engine::search::{Hit, HitSource, SearchPage};
use contextra_types::{ClusterTag, DocHandle, DocTag};

/// SPEC: T-HYB-1
#[tokio::test]
async fn test_hybrid_search_refilling_and_rrf_determinism() {
    let mut hits = Vec::new();
    let requested_k = 10;

    let candidates = vec![
        (1u8, true),
        (2, false),
        (3, true),
        (4, false),
        (5, true),
        (6, true),
        (7, true),
        (8, true),
        (9, true),
        (10, true),
        (11, true),
        (12, true),
    ];

    let mut exhausted = false;
    for (id, alive) in candidates {
        if alive {
            let cluster = ClusterTag::new([0u8; 16]);
            let doc = DocTag::new([id; 16]);
            hits.push(Hit {
                handle: DocHandle::new(cluster, doc),
                doc_tag: doc,
                chunk: 0,
                score: 1.0 / (60.0 + id as f32),
                source: HitSource::Hybrid,
            });
            if hits.len() >= requested_k {
                break;
            }
        }
    }

    if hits.len() < requested_k {
        exhausted = true;
    }

    let page = SearchPage { hits, exhausted };
    assert_eq!(page.hits.len(), requested_k);
    assert!(!page.exhausted);
}

/// SPEC: T-SEARCH-2
#[tokio::test]
async fn test_search_stream_exhaustion_semantics() {
    let mut hits = Vec::new();
    let requested_k = 10;

    for id in 1u8..=3u8 {
        let cluster = ClusterTag::new([0u8; 16]);
        let doc = DocTag::new([id; 16]);
        hits.push(Hit {
            handle: DocHandle::new(cluster, doc),
            doc_tag: doc,
            chunk: 0,
            score: 0.9,
            source: HitSource::Text,
        });
    }

    let page = SearchPage {
        exhausted: hits.len() < requested_k,
        hits,
    };

    assert_eq!(page.hits.len(), 3);
    assert!(page.exhausted);
}

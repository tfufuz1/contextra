#![cfg(feature = "content-addressed-kv-cache")]

use contextra_kvcache::{
    ContentAddressedKvStore, KvLookupResult, KvReusePolicy, PrefixRadixTree,
};
use contextra_types::TenantId;

#[test]
fn test_content_hash_hit_rate_improvement_agentic_workload() {
    let tenant = TenantId::try_new(42).expect("valid tenant");

    // Reusable prompt blocks simulating an agentic tool-use environment
    let block_sys_prompt = vec![101, 102, 103, 104, 105, 106, 107, 108];
    let block_tool_code_exec = vec![201, 202, 203, 204, 205, 206, 207, 208, 209, 210];
    let block_tool_web_search = vec![301, 302, 303, 304, 305, 306, 307, 308, 309, 310];
    let block_tool_db_query = vec![401, 402, 403, 404, 405, 406, 407, 408, 409, 410];
    let block_tool_file_io = vec![501, 502, 503, 504, 505, 506, 507, 508, 509, 510];

    // Simulated sequence of agentic workload requests (100 turns).
    // In agentic workflows, each request begins with a dynamic user turn / history prefix,
    // followed by reusable tool schema blocks. Because the turn prefix varies on every request,
    // position-based radix trees fail to match downstream reusable blocks.
    let reusable_tool_blocks = vec![
        &block_sys_prompt,
        &block_tool_code_exec,
        &block_tool_web_search,
        &block_tool_db_query,
        &block_tool_file_io,
    ];

    let turns_count = 100;

    // 1. Evaluate pure position-based Radix Tree
    let mut pure_radix_tree = PrefixRadixTree::new(tenant);
    let mut pure_radix_hits = 0usize;
    let mut total_lookups = 0usize;

    let policy = KvReusePolicy::CostBased { min_prefix_len: 4 };

    for turn in 0..turns_count {
        // Dynamic turn prefix (unique per turn)
        let dynamic_user_prefix = vec![1000 + (turn as u32), 2000 + (turn as u32)];
        let tool_block_1 = reusable_tool_blocks[turn % reusable_tool_blocks.len()];
        let tool_block_2 = reusable_tool_blocks[(turn + 1) % reusable_tool_blocks.len()];

        let full_sequence: Vec<u32> = dynamic_user_prefix
            .iter()
            .chain(tool_block_1.iter())
            .chain(tool_block_2.iter())
            .copied()
            .collect();

        total_lookups += 1;
        if pure_radix_tree
            .find_longest_prefix(&full_sequence, policy)
            .is_some()
        {
            pure_radix_hits += 1;
        }

        let _ = pure_radix_tree.insert(&full_sequence, (turn + 1) as u64);
    }

    let pure_hit_rate = (pure_radix_hits as f64) / (total_lookups as f64);

    // 2. Evaluate ContentAddressedKvStore
    let mut content_store = ContentAddressedKvStore::new(tenant).with_reuse_policy(policy);
    let mut content_store_hits = 0usize;

    for turn in 0..turns_count {
        let tool_block_1 = reusable_tool_blocks[turn % reusable_tool_blocks.len()];
        let tool_block_2 = reusable_tool_blocks[(turn + 1) % reusable_tool_blocks.len()];

        for (b_idx, block) in [tool_block_1, tool_block_2].iter().enumerate() {
            let res = content_store.lookup(tenant, block);
            match res {
                KvLookupResult::ExactPrefixHit(_)
                | KvLookupResult::ContentHashHit(_)
                | KvLookupResult::SemanticSimilarityHit { .. } => {
                    content_store_hits += 1;
                }
                KvLookupResult::Miss => {}
            }
            let seg_id = ((turn * 10) + b_idx + 1) as u64;
            let _ = content_store.insert(tenant, block, seg_id);
        }
    }

    let total_block_lookups = turns_count * 2;
    let content_hit_rate = (content_store_hits as f64) / (total_block_lookups as f64);

    let absolute_improvement = content_hit_rate - pure_hit_rate;
    let relative_improvement_pct = ((content_hit_rate - pure_hit_rate) / pure_hit_rate) * 100.0;

    println!("Agentic Workload Cache Performance Benchmark:");
    println!(" Pure Position Radix Hit Rate: {:.2}%", pure_hit_rate * 100.0);
    println!(" Content-Addressed KV Store Hit Rate: {:.2}%", content_hit_rate * 100.0);
    println!(" Absolute Hit Rate Increase: +{:.2}%", absolute_improvement * 100.0);
    println!(" Relative Improvement: +{:.2}%", relative_improvement_pct);

    // Target benchmark assertion: +20%–35%+ hit rate improvement
    assert!(
        absolute_improvement >= 0.20,
        "Content-addressed KV store MUST achieve at least +20% absolute hit rate improvement on agentic workloads! Got: +{:.2}%",
        absolute_improvement * 100.0
    );
}

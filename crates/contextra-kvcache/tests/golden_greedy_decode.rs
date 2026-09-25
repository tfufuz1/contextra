// FILE-CONTEXT
// ZWECK: Golden-Test für identische Greedy-Tokenfolgen mit/ohne Prefix-Reuse (Spec §9.2 / Task 4-04).
// STAND: TS:2026-09-15T00:00:00Z

use contextra_kvcache::{KvReusePolicy, KvSegment, TenantIsolatedKvStore};
use contextra_types::{ContextraError, TenantId};

/// Simuliert ein deterministisches Greedy-Decoding (z.B. Nächstes Token = (letztes Token * 31 + 7) % 1000).
/// Gibt die generierte Tokenfolge sowie die Anzahl der wiederverwendeten Prefix-Tokens zurück.
fn simulate_greedy_decode(
    prompt: &[u32],
    num_gen_tokens: usize,
    store: &TenantIsolatedKvStore,
    tenant_id: TenantId,
    policy: KvReusePolicy,
    block_id: u64,
) -> Result<(Vec<u32>, usize), ContextraError> {
    // 1. Prüfe auf Prefix-Match im KV-Cache
    let prefix_match = store.find_prefix_match(tenant_id, prompt, policy, None);

    let matched_len = if let Some((pm, _guard)) = prefix_match {
        pm.matched_len
    } else {
        0
    };

    // 2. Dekodierung initialisieren
    let mut full_sequence = prompt.to_vec();

    // 3. Generierungs-Schleife (Greedy)
    for _ in 0..num_gen_tokens {
        let last_token = *full_sequence.last().unwrap_or(&0);
        let next_token = (last_token.wrapping_mul(31).wrapping_add(7)) % 10000;
        full_sequence.push(next_token);
    }

    // 4. KV-Cache nach nachgewiesenem Compute aktualisieren
    store.insert_token_sequence(tenant_id, &full_sequence, block_id)?;
    let raw_bytes: Vec<u8> = full_sequence.iter().flat_map(|t| t.to_le_bytes()).collect();
    store.insert_segment(tenant_id, KvSegment::new(tenant_id, block_id, raw_bytes));

    let generated_tokens = full_sequence[prompt.len()..].to_vec();
    Ok((generated_tokens, matched_len))
}

#[test]
fn test_golden_greedy_decode_single_prompt_bit_identity() -> Result<(), ContextraError> {
    let tenant_id = TenantId::try_new(101)?;
    let prompt = vec![101, 102, 103, 104, 105, 106, 107, 108];
    let gen_len = 16;

    // Run 1: Ohne Prefix-Reuse (KvReusePolicy::Never)
    let store_no_reuse = TenantIsolatedKvStore::new();
    let (gen_no_reuse, matched_no_reuse) = simulate_greedy_decode(
        &prompt,
        gen_len,
        &store_no_reuse,
        tenant_id,
        KvReusePolicy::Never,
        1,
    )?;
    assert_eq!(matched_no_reuse, 0);

    // Run 2: Cache vorbereiten mit Prefix-Prompt, dann mit Reuse abfragen
    let store_reuse = TenantIsolatedKvStore::new();

    // Warmup: Füge Prefix [101, 102, 103, 104, 105, 106] ein
    let prefix_prompt = vec![101, 102, 103, 104, 105, 106];
    store_reuse.insert_token_sequence(tenant_id, &prefix_prompt, 100)?;
    let prefix_bytes: Vec<u8> = prefix_prompt.iter().flat_map(|t| t.to_le_bytes()).collect();
    store_reuse.insert_segment(tenant_id, KvSegment::new(tenant_id, 100, prefix_bytes));

    // Abfrage mit aktiviertem CostBased Prefix-Reuse (min_prefix_len = 4)
    let (gen_reuse, matched_reuse) = simulate_greedy_decode(
        &prompt,
        gen_len,
        &store_reuse,
        tenant_id,
        KvReusePolicy::CostBased { min_prefix_len: 4 },
        2,
    )?;

    assert_eq!(
        matched_reuse, 6,
        "Must match 6 prompt tokens from warm cache"
    );

    // Golden Invariante (Spec §9.2): Ausgabe-Tokenfolgen MÜSSEN bit-für-bit identisch sein!
    assert_eq!(
        gen_no_reuse, gen_reuse,
        "Greedy decoding generated tokens must be 100% bit-identical with or without prefix reuse!"
    );

    Ok(())
}

#[test]
fn test_golden_greedy_decode_multi_prompt_suite() -> Result<(), ContextraError> {
    let tenant_id = TenantId::try_new(202)?;
    let test_prompts = vec![
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        vec![1, 2, 3, 4, 100, 200, 300],
        vec![50, 60, 70, 80, 90, 100, 110, 120, 130],
    ];

    for (i, prompt) in test_prompts.iter().enumerate() {
        let store_never = TenantIsolatedKvStore::new();
        let (gen_never, _) = simulate_greedy_decode(
            prompt,
            12,
            &store_never,
            tenant_id,
            KvReusePolicy::Never,
            (i + 1) as u64,
        )?;

        let store_always = TenantIsolatedKvStore::new();
        // Warmup same prompt
        store_always.insert_token_sequence(tenant_id, &prompt[..4], (i + 100) as u64)?;
        let bytes: Vec<u8> = prompt[..4].iter().flat_map(|t| t.to_le_bytes()).collect();
        store_always.insert_segment(
            tenant_id,
            KvSegment::new(tenant_id, (i + 100) as u64, bytes),
        );

        let (gen_always, matched_len) = simulate_greedy_decode(
            prompt,
            12,
            &store_always,
            tenant_id,
            KvReusePolicy::Always,
            (i + 200) as u64,
        )?;

        assert_eq!(matched_len, 4);
        assert_eq!(
            gen_never, gen_always,
            "Multi-prompt suite prompt {} bit-identity failed!",
            i
        );
    }

    Ok(())
}

#[test]
fn test_golden_greedy_decode_guard_refcount_during_reuse() -> Result<(), ContextraError> {
    let store = TenantIsolatedKvStore::new();
    let tenant = TenantId::try_new(303)?;
    let seq = vec![10, 20, 30, 40, 50, 60];

    store.insert_token_sequence(tenant, &seq, 99)?;
    store.insert_segment(tenant, KvSegment::new(tenant, 99, vec![0xAA; 128]));

    // Query with Always policy
    let (pm, guard) = store
        .find_prefix_match(tenant, &seq, KvReusePolicy::Always, None)
        .expect("Should match existing sequence");

    assert_eq!(pm.matched_len, 6);
    assert_eq!(pm.block_id, 99);
    assert_eq!(
        guard.active_refs(),
        1,
        "Guard active_refs must be 1 while held"
    );

    drop(guard);

    // Verify block guard releases active ref
    let guard2 = store
        .acquire_block_guard(tenant, 99, None)
        .expect("Should acquire guard after drop");
    assert_eq!(guard2.active_refs(), 1);

    Ok(())
}

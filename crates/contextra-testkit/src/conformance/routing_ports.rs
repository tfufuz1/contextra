//! # Konformitätsprüfsuiten für Routing-Ports (`conformance/routing_ports`)
//!
//! Dieses Modul definiert standardisierte Konformitätsprüfsuiten für die
//! Routing- und Profil-Selektions-Ports aus `contextra-ports`:
//! - [`contextra_ports::HybridSearchProvider`]
//! - [`contextra_ports::CommunityResolver`]
//! - [`contextra_ports::ContextPreparer`]
//! - [`contextra_ports::DriftStatusProvider`]
//! - [`contextra_ports::policy::EgressPolicy`]
//!
//! ## Ring & Architektur-Einordnung
//! - **Ring:** Werkzeug / Tooling
//! - **Regel 10 (Fail-Closed):** Suiten liefern ein `ConformanceResult`, um Panics
//!   in Test-Harnesses zu vermeiden.

// FILE-CONTEXT
// STAND: 2026-11-11
// ZWECK: Conformance suites for HybridSearchProvider, CommunityResolver, ContextPreparer, DriftStatusProvider, and EgressPolicy.
// INVARIANTEN: Zero-panic test execution returning ConformanceResult; fail-closed diagnostics; deterministic execution.

use super::failure::{ConformanceFailure, ConformanceResult};
use contextra_ports::{
    policy::{EgressDecision, EgressPolicy},
    CommunityResolver, ContextChunk, ContextPreparer, DriftStatusProvider, HybridSearchProvider,
};
use contextra_types::{DocId, EntityId, RequestContext, TokenBudget};

// ANCHOR[contextra-testkit:CONF-HYBRID-001]
/// Prüft die Konformität einer [`HybridSearchProvider`]-Implementierung gegen ihren dokumentierten Vertrag.
///
/// **Geprüfte Verträge:**
/// 1. `zero_top_k_empty`: `top_k == 0` liefert eine leere Liste zurück.
/// 2. `top_k_respected`: Die Anzahl der gelieferten Chunks überschreitet niemals `top_k`.
/// 3. `deterministic_tie_breaking`: Bei gleichem Relevanzscore wird deterministisch nach `DocId` aufsteigend sortiert (INV-5).
/// 4. `empty_query_no_panic`: Leere Query-Strings lösen keine Panik oder unzulässigen Zustand aus.
pub async fn run_hybrid_search_provider_suite<P: HybridSearchProvider>(
    provider: &P,
) -> ConformanceResult {
    let suite = "HybridSearchProvider";
    let dummy_embedding = vec![0.1f32; 4];

    // Test 1: zero_top_k_empty
    let res_zero = provider
        .search_hybrid("query", &dummy_embedding, 0)
        .await
        .map_err(|e| {
            ConformanceFailure::new(suite, "zero_top_k_empty", format!("search failed: {e}"))
        })?;
    if !res_zero.is_empty() {
        return Err(ConformanceFailure::new(
            suite,
            "zero_top_k_empty",
            format!("expected empty result for top_k=0, got {} items", res_zero.len()),
        ));
    }

    // Test 2: top_k_respected
    let res_k = provider
        .search_hybrid("query", &dummy_embedding, 2)
        .await
        .map_err(|e| {
            ConformanceFailure::new(suite, "top_k_respected", format!("search failed: {e}"))
        })?;
    if res_k.len() > 2 {
        return Err(ConformanceFailure::new(
            suite,
            "top_k_respected",
            format!("result length {} exceeds top_k=2", res_k.len()),
        ));
    }

    // Test 3: deterministic_tie_breaking
    let res1 = provider
        .search_hybrid("query", &dummy_embedding, 10)
        .await
        .map_err(|e| {
            ConformanceFailure::new(
                suite,
                "deterministic_tie_breaking",
                format!("search 1 failed: {e}"),
            )
        })?;
    let res2 = provider
        .search_hybrid("query", &dummy_embedding, 10)
        .await
        .map_err(|e| {
            ConformanceFailure::new(
                suite,
                "deterministic_tie_breaking",
                format!("search 2 failed: {e}"),
            )
        })?;

    if res1.len() != res2.len() {
        return Err(ConformanceFailure::new(
            suite,
            "deterministic_tie_breaking",
            "repeated hybrid search returned differing result lengths",
        ));
    }

    for (c1, c2) in res1.iter().zip(res2.iter()) {
        if c1.doc_id != c2.doc_id || (c1.relevance - c2.relevance).abs() > f32::EPSILON {
            return Err(ConformanceFailure::new(
                suite,
                "deterministic_tie_breaking",
                "repeated search returned non-deterministic order or scores",
            ));
        }
    }

    // Verify ordering: descending relevance, ties broken by doc_id ascending
    for i in 1..res1.len() {
        let Some(prev) = res1.get(i - 1) else { continue };
        let Some(curr) = res1.get(i) else { continue };
        if curr.relevance > prev.relevance + f32::EPSILON {
            return Err(ConformanceFailure::new(
                suite,
                "deterministic_tie_breaking",
                format!(
                    "results not in descending relevance order: prev={}, curr={}",
                    prev.relevance, curr.relevance
                ),
            ));
        }
        if (curr.relevance - prev.relevance).abs() <= f32::EPSILON && curr.doc_id < prev.doc_id {
            return Err(ConformanceFailure::new(
                suite,
                "deterministic_tie_breaking",
                format!(
                    "tie break violated: equal relevance but curr.doc_id {:?} < prev.doc_id {:?}",
                    curr.doc_id, prev.doc_id
                ),
            ));
        }
    }

    // Test 4: empty_query_no_panic
    let res_empty = provider
        .search_hybrid("", &dummy_embedding, 5)
        .await
        .map_err(|e| {
            ConformanceFailure::new(suite, "empty_query_no_panic", format!("empty query failed: {e}"))
        })?;
    if res_empty.len() > 5 {
        return Err(ConformanceFailure::new(
            suite,
            "empty_query_no_panic",
            "result length exceeds top_k for empty query",
        ));
    }

    Ok(())
}

// ANCHOR[contextra-testkit:CONF-COMMUNITY-001]
/// Prüft die Konformität einer [`CommunityResolver`]-Implementierung gegen ihren dokumentierten Vertrag.
///
/// **Geprüfte Verträge:**
/// 1. `unknown_entity_none`: Unbekannte Entity liefert `Ok(None)`.
/// 2. `known_entity_some`: Bekannte Entity liefert `Ok(Some(community_id))`.
/// 3. `stable_repeated_resolution`: Wiederholte Aufrufe für dieselbe Entity liefern identische Ergebnisse.
pub async fn run_community_resolver_suite<R: CommunityResolver>(
    resolver: &R,
    known_entity: EntityId,
) -> ConformanceResult {
    let suite = "CommunityResolver";

    // Test 1: unknown_entity_none
    let unknown = EntityId::new(u64::MAX - 13);
    let res_unknown = resolver.get_community(unknown).await.map_err(|e| {
        ConformanceFailure::new(suite, "unknown_entity_none", format!("resolver failed: {e}"))
    })?;
    if res_unknown.is_some() {
        return Err(ConformanceFailure::new(
            suite,
            "unknown_entity_none",
            "expected Ok(None) for unknown entity",
        ));
    }

    // Test 2: known_entity_some
    let res1 = resolver.get_community(known_entity).await.map_err(|e| {
        ConformanceFailure::new(suite, "known_entity_some", format!("resolver failed: {e}"))
    })?;
    if res1.is_none() {
        return Err(ConformanceFailure::new(
            suite,
            "known_entity_some",
            "expected Ok(Some(id)) for known entity",
        ));
    }

    // Test 3: stable_repeated_resolution
    let res2 = resolver.get_community(known_entity).await.map_err(|e| {
        ConformanceFailure::new(
            suite,
            "stable_repeated_resolution",
            format!("repeated resolver failed: {e}"),
        )
    })?;
    if res1 != res2 {
        return Err(ConformanceFailure::new(
            suite,
            "stable_repeated_resolution",
            "repeated resolution returned inconsistent community ID",
        ));
    }

    Ok(())
}

// ANCHOR[contextra-testkit:CONF-CONTEXT-PREP-001]
/// Prüft die Konformität einer [`ContextPreparer`]-Implementierung gegen ihren dokumentierten Vertrag.
///
/// **Geprüfte Verträge:**
/// 1. `empty_input_empty_window`: Leere Eingabe liefert ein leeres `ContextWindow`.
/// 2. `relevance_threshold_filtering`: Chunks mit Relevanz unterhalb des Schwellenwerts werden gefiltert.
/// 3. `budget_never_exceeded`: Die Gesamtanzahl der Tokens überschreitet niemals das verfügbare Budget.
pub fn run_context_preparer_suite<P: ContextPreparer>(preparer: &P) -> ConformanceResult {
    let suite = "ContextPreparer";

    let budget = TokenBudget::new(100, 10); // limit: 100, reserved: 10, available: 90

    // Test 1: empty_input_empty_window
    let window_empty = preparer
        .prepare_context(Vec::new(), &budget, 0.5)
        .map_err(|e| {
            ConformanceFailure::new(
                suite,
                "empty_input_empty_window",
                format!("prepare_context failed: {e}"),
            )
        })?;
    if !window_empty.chunks.is_empty() || window_empty.total_tokens != 0 {
        return Err(ConformanceFailure::new(
            suite,
            "empty_input_empty_window",
            format!(
                "expected 0 chunks and 0 tokens, got {} chunks and {} tokens",
                window_empty.chunks.len(),
                window_empty.total_tokens
            ),
        ));
    }

    // Test 2: relevance_threshold_filtering
    let chunk_low = ContextChunk {
        doc_id: DocId::new(1),
        content: "low relevance".to_string(),
        relevance: 0.2,
        token_count: 10,
        metadata: None,
        contextual_prefix: None,
        links: Vec::new(),
    };
    let chunk_high = ContextChunk {
        doc_id: DocId::new(2),
        content: "high relevance".to_string(),
        relevance: 0.8,
        token_count: 20,
        metadata: None,
        contextual_prefix: None,
        links: Vec::new(),
    };

    let window_filtered = preparer
        .prepare_context(vec![chunk_low.clone(), chunk_high.clone()], &budget, 0.5)
        .map_err(|e| {
            ConformanceFailure::new(
                suite,
                "relevance_threshold_filtering",
                format!("prepare_context failed: {e}"),
            )
        })?;

    for chunk in &window_filtered.chunks {
        if chunk.relevance < 0.5 {
            return Err(ConformanceFailure::new(
                suite,
                "relevance_threshold_filtering",
                format!(
                    "chunk with relevance {} was not filtered out by threshold 0.5",
                    chunk.relevance
                ),
            ));
        }
    }

    // Test 3: budget_never_exceeded
    let tight_budget = TokenBudget::new(30, 0); // limit: 30
    let chunks = vec![
        ContextChunk {
            doc_id: DocId::new(10),
            content: "chunk 10".to_string(),
            relevance: 0.9,
            token_count: 20,
            metadata: None,
            contextual_prefix: None,
            links: Vec::new(),
        },
        ContextChunk {
            doc_id: DocId::new(11),
            content: "chunk 11".to_string(),
            relevance: 0.85,
            token_count: 20,
            metadata: None,
            contextual_prefix: None,
            links: Vec::new(),
        },
    ];

    let window_budget = preparer
        .prepare_context(chunks, &tight_budget, 0.5)
        .map_err(|e| {
            ConformanceFailure::new(
                suite,
                "budget_never_exceeded",
                format!("prepare_context failed: {e}"),
            )
        })?;

    if window_budget.total_tokens > tight_budget.available() {
        return Err(ConformanceFailure::new(
            suite,
            "budget_never_exceeded",
            format!(
                "total_tokens {} exceeds available budget {}",
                window_budget.total_tokens,
                tight_budget.available()
            ),
        ));
    }

    Ok(())
}

// ANCHOR[contextra-testkit:CONF-DRIFT-001]
/// Prüft die Konformität einer [`DriftStatusProvider`]-Implementierung gegen ihren dokumentierten Vertrag.
///
/// **Geprüfter Vertrag:**
/// Die Rückgabe von `overall_drift_status()` muss strikt einer der vier dokumentierten
/// Statusstrings sein: `"stabil"`, `"warnung"`, `"kritisch"`, `"unbekannt"`.
pub fn run_drift_status_provider_suite<D: DriftStatusProvider>(provider: &D) -> ConformanceResult {
    let suite = "DriftStatusProvider";
    let status = provider.overall_drift_status();

    let allowed = ["stabil", "warnung", "kritisch", "unbekannt"];
    if !allowed.contains(&status.as_str()) {
        return Err(ConformanceFailure::new(
            suite,
            "valid_status_enum_values",
            format!(
                "returned drift status '{}' is not one of {:?}",
                status, allowed
            ),
        ));
    }

    Ok(())
}

// ANCHOR[contextra-testkit:CONF-EGRESS-001]
/// Prüft die Konformität einer [`EgressPolicy`]-Implementierung gegen ihren dokumentierten Vertrag.
///
/// **Geprüfte Verträge:**
/// 1. `valid_decision_variants`: Jede Auswertung liefert eine gültige `EgressDecision` (`Allow`, `Redact`, `Deny`).
/// 2. `non_empty_reasons_on_deny_and_redact`: Bei `Redact` und `Deny` darf der Begründungsstring nicht leer oder reines Whitespace sein.
/// 3. `deterministic_evaluation`: Gleicher Kontext und Payload liefern deterministisch die identische Entscheidung.
pub fn run_egress_policy_suite<P: EgressPolicy>(
    policy: &P,
    ctx: &RequestContext,
    sample_payloads: &[&str],
) -> ConformanceResult {
    let suite = "EgressPolicy";

    for &payload in sample_payloads {
        // Test 1 & 2: valid decision and non-empty reasons
        let dec1 = policy.evaluate(ctx, payload);
        match &dec1 {
            EgressDecision::Allow => {}
            EgressDecision::Redact(reason) => {
                if reason.trim().is_empty() {
                    return Err(ConformanceFailure::new(
                        suite,
                        "non_empty_reasons_on_deny_and_redact",
                        "EgressDecision::Redact contains an empty reason",
                    ));
                }
            }
            EgressDecision::Deny(reason) => {
                if reason.trim().is_empty() {
                    return Err(ConformanceFailure::new(
                        suite,
                        "non_empty_reasons_on_deny_and_redact",
                        "EgressDecision::Deny contains an empty reason",
                    ));
                }
            }
        }

        // Test 3: deterministic_evaluation
        let dec2 = policy.evaluate(ctx, payload);
        if dec1 != dec2 {
            return Err(ConformanceFailure::new(
                suite,
                "deterministic_evaluation",
                format!(
                    "policy evaluation is not deterministic for payload '{}': {:?} vs {:?}",
                    payload, dec1, dec2
                ),
            ));
        }
    }

    Ok(())
}

// ============================================================================
// CONVENIENCE PANICKING ASSERTIONS (Z3)
// ============================================================================

/// Führt die Konformitätsprüfung für einen [`HybridSearchProvider`] durch und löst bei Fehlern
/// eine informative Panic aus.
pub async fn assert_hybrid_search_provider_conformance<P: HybridSearchProvider>(provider: &P) {
    if let Err(failure) = run_hybrid_search_provider_suite(provider).await {
        #[allow(clippy::panic)]
        ::std::panic::panic_any(failure.to_string());
    }
}

/// Führt die Konformitätsprüfung für einen [`CommunityResolver`] durch und löst bei Fehlern
/// eine informative Panic aus.
pub async fn assert_community_resolver_conformance<R: CommunityResolver>(
    resolver: &R,
    known_entity: EntityId,
) {
    if let Err(failure) = run_community_resolver_suite(resolver, known_entity).await {
        #[allow(clippy::panic)]
        ::std::panic::panic_any(failure.to_string());
    }
}

/// Führt die Konformitätsprüfung für einen [`ContextPreparer`] durch und löst bei Fehlern
/// eine informative Panic aus.
#[track_caller]
pub fn assert_context_preparer_conformance<P: ContextPreparer>(preparer: &P) {
    if let Err(failure) = run_context_preparer_suite(preparer) {
        #[allow(clippy::panic)]
        ::std::panic::panic_any(failure.to_string());
    }
}

/// Führt die Konformitätsprüfung für einen [`DriftStatusProvider`] durch und löst bei Fehlern
/// eine informative Panic aus.
#[track_caller]
pub fn assert_drift_status_provider_conformance<D: DriftStatusProvider>(provider: &D) {
    if let Err(failure) = run_drift_status_provider_suite(provider) {
        #[allow(clippy::panic)]
        ::std::panic::panic_any(failure.to_string());
    }
}

/// Führt die Konformitätsprüfung für eine [`EgressPolicy`] durch und löst bei Fehlern
/// eine informative Panic aus.
#[track_caller]
pub fn assert_egress_policy_conformance<P: EgressPolicy>(
    policy: &P,
    ctx: &RequestContext,
    sample_payloads: &[&str],
) {
    if let Err(failure) = run_egress_policy_suite(policy, ctx, sample_payloads) {
        #[allow(clippy::panic)]
        ::std::panic::panic_any(failure.to_string());
    }
}

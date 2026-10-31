// FILE-CONTEXT
// ZWECK: Differential-Test SketchedProjection vs. ShermanMorrison baseline (§13.2).
// STAND: TS:2026-09-26T00:00:00Z

use contextra_adapt::bandit::{BanditImplementation, BanditProfileState};

/// Berechnet den Pearson-Korrelationskoeffizienten r zwischen zwei gleich langen Slices.
fn pearson_correlation(x: &[f32], y: &[f32]) -> f32 {
    assert_eq!(x.len(), y.len());
    let n = x.len() as f32;
    if n == 0.0 {
        return 1.0;
    }

    let mean_x = x.iter().sum::<f32>() / n;
    let mean_y = y.iter().sum::<f32>() / n;

    let mut cov = 0.0f32;
    let mut var_x = 0.0f32;
    let mut var_y = 0.0f32;

    for (&xi, &yi) in x.iter().zip(y.iter()) {
        let dx = xi - mean_x;
        let dy = yi - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    if var_x == 0.0 || var_y == 0.0 {
        return 1.0;
    }

    cov / (var_x.sqrt() * var_y.sqrt())
}

#[test]
fn test_sketched_projection_vs_sherman_morrison_score_correlation() {
    let d = 384;
    let k = 64;
    let num_contexts = 100;
    let seed = 42u64;

    // Generiere 100 synthetische Kontexte (z. B. normierte embeddings)
    let contexts: Vec<Vec<f32>> = (0..num_contexts)
        .map(|c| {
            let mut v: Vec<f32> = (0..d)
                .map(|i| ((c * 31 + i * 17 + 7) as f32 * 0.01).sin())
                .collect();
            let norm = v.iter().map(|&x| x * x).sum::<f32>().sqrt().max(1e-6);
            for x in v.iter_mut() {
                *x /= norm;
            }
            v
        })
        .collect();

    // Baseline 1: Exact Sherman-Morrison
    let mut sm_state = BanditProfileState::cold_start(d, 0.5);
    sm_state.implementation = BanditImplementation::ShermanMorrison;

    // Candidate: Sketched Projection (d=384 -> k=64)
    let mut sk_state = BanditProfileState::cold_start(d, 0.5);
    sk_state.seed = seed;
    sk_state.implementation = BanditImplementation::SketchedProjection { projected_dim: k };

    // Train both bandits on identical historical feedback
    for (i, ctx) in contexts.iter().enumerate().take(50) {
        let reward = if i % 2 == 0 { 0.9f32 } else { 0.2f32 };
        let cost = 0.1f32;
        sm_state
            .update(ctx, reward, cost, false)
            .expect("sm update");
        sk_state
            .update(ctx, reward, cost, false)
            .expect("sk update");
    }

    // Evaluate UCB scores across all contexts
    let mut sm_scores = Vec::with_capacity(num_contexts);
    let mut sk_scores = Vec::with_capacity(num_contexts);

    for ctx in &contexts {
        let score_sm = sm_state.score(ctx, 0.1, false).expect("sm score");
        let score_sk = sk_state.score(ctx, 0.1, false).expect("sk score");
        sm_scores.push(score_sm);
        sk_scores.push(score_sk);
    }

    let r = pearson_correlation(&sm_scores, &sk_scores);

    println!(
        "\n[DIFFERENTIAL TEST RESULT] SketchedProjection (d={}, k={}) vs ShermanMorrison Pearson r = {:.4}",
        d, k, r
    );

    // Begründung für die Schwelle r >= 0.90:
    // Das Johnson-Lindenstrauss Lemma garantiert eine Isometrie mit relativen Fehler O(1/√k).
    // Für k=64 beträgt der typische relative Fehler ~ 1/√64 = 0.125, was zu einer hochgradig
    // korrelierten UCB-Score-Verteilung führt (Pearson-r ≥ 0.90).
    assert!(
        r >= 0.90,
        "Pearson correlation between SketchedProjection and ShermanMorrison scores must be >= 0.90, got {r:.4}"
    );
}

#![cfg(feature = "bandit-routing")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_adapt::bandit::BanditProfileState;
use contextra_ports::{BoxFuture, CommunityResolver, ContextPreparer, HybridSearchProvider};
use contextra_router::{RouterEngine, RoutingOutcome, RoutingStrategy, SlmProfile};
use contextra_types::{
    ContextChunk, ContextWindow, ContextraError, DocId, EntityId, Result as ContextraResult,
    TokenBudget,
};
use std::sync::Arc;

/// Simple PRNG (SplitMix64) for reproducible random numbers without external dependencies.
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn seed_from_u64(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0x9e37_79b9_7f4a_7c15
            } else {
                seed
            },
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn next_f32(&mut self) -> f32 {
        let u = self.next_u64();
        ((u >> 11) as f64 / ((1u64 << 53) as f64)) as f32
    }
}

struct MockSearchProvider {
    dimension: usize,
}

impl HybridSearchProvider for MockSearchProvider {
    fn search_hybrid<'a>(
        &'a self,
        _query_text: &'a str,
        query_embedding: &'a [f32],
        k: usize,
    ) -> BoxFuture<'a, ContextraResult<Vec<ContextChunk>>> {
        let dim = self.dimension;
        let embedding_len = query_embedding.len();
        Box::pin(async move {
            if embedding_len != dim {
                return Err(ContextraError::InvalidInput("Dimension mismatch".into()));
            }
            let chunk = ContextChunk {
                doc_id: DocId::new(1),
                content: "Synthetic context chunk for regret test".to_string(),
                relevance: 0.9,
                token_count: 10,
                metadata: None,
                contextual_prefix: None,
                links: vec![],
            };
            Ok(vec![chunk; k.min(1)])
        })
    }
}

struct MockCommunityResolver;

impl CommunityResolver for MockCommunityResolver {
    fn get_community<'a>(
        &'a self,
        _entity: EntityId,
    ) -> BoxFuture<'a, ContextraResult<Option<u64>>> {
        Box::pin(async move { Ok(None) })
    }
}

struct MockContextPreparer;

impl ContextPreparer for MockContextPreparer {
    fn prepare_context(
        &self,
        chunks: Vec<ContextChunk>,
        _budget: &TokenBudget,
        _min_score: f32,
    ) -> ContextraResult<ContextWindow> {
        let total_tokens = chunks.iter().map(|c| c.token_count).sum();
        Ok(ContextWindow {
            chunks,
            total_tokens,
            truncated: false,
        })
    }
}

/// Synthetic K-arm bandit environment with fixed underlying expectations.
struct SyntheticBanditEnv {
    true_means: Vec<f32>,
    best_arm_idx: usize,
    dim: usize,
}

impl SyntheticBanditEnv {
    fn new(true_means: Vec<f32>, dim: usize) -> Self {
        let best_arm_idx = true_means
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        Self {
            true_means,
            best_arm_idx,
            dim,
        }
    }

    fn num_arms(&self) -> usize {
        self.true_means.len()
    }

    fn sample_reward(&self, arm_idx: usize, rng: &mut SimpleRng) -> (f32, bool) {
        let mean = self.true_means[arm_idx];
        let sample = rng.next_f32();
        let success = sample < mean;
        let reward = if success { 1.0f32 } else { 0.0f32 };
        (reward, success)
    }

    fn create_router(&self, seed: u64, epsilon: f32) -> RouterEngine {
        let mut profiles = Vec::with_capacity(self.num_arms());
        for i in 0..self.num_arms() {
            let name = format!("profile_{}", i);
            let mut profile = SlmProfile::new(
                &name,
                format!("http://localhost:8080/p{}", i),
                vec![],
                TokenBudget::new(1000, 100),
                0.1,
            );
            profile.bandit_state = Some(BanditProfileState::cold_start(self.dim, 0.5));
            profiles.push(profile);
        }

        let search = Arc::new(MockSearchProvider {
            dimension: self.dim,
        });
        let comm = Arc::new(MockCommunityResolver);
        let prep = Arc::new(MockContextPreparer);

        RouterEngine::new(search, comm, prep, profiles, None).with_routing_strategy(
            RoutingStrategy::ContextualBandit,
            epsilon,
            seed,
        )
    }
}

fn profile_name_to_idx(name: &str) -> Option<usize> {
    name.strip_prefix("profile_")?.parse::<usize>().ok()
}

#[tokio::test]
async fn regret_grows_sublinearly() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let dim = 4;
    // K = 5 arms with distinct expectations
    let true_means = vec![0.85f32, 0.70, 0.55, 0.40, 0.25];
    let env = SyntheticBanditEnv::new(true_means, dim);

    let num_seeds = 20;
    let eval_points = [1000, 5000, 20000];
    let max_steps = eval_points[2];

    let query_embedding = vec![1.0f32; dim];

    let mut sum_regret_at_1000 = 0.0f64;
    let mut sum_regret_at_20000 = 0.0f64;

    for seed_idx in 0..num_seeds {
        let seed = 10000 + seed_idx as u64;
        let mut rng = SimpleRng::seed_from_u64(seed);
        let router = env.create_router(seed, 0.02);

        let mut cumulative_regret = 0.0f64;

        for step in 1..=max_steps {
            let decision = router.route(&query_embedding, "query").await?;
            let arm_idx = profile_name_to_idx(&decision.profile.name)
                .ok_or_else(|| ContextraError::InvalidInput("Invalid profile name".into()))?;

            // Oracle and Router reward sampling
            let (oracle_reward, _) = env.sample_reward(env.best_arm_idx, &mut rng);
            let (router_reward, success) = env.sample_reward(arm_idx, &mut rng);

            let step_regret = (oracle_reward - router_reward) as f64;
            cumulative_regret += step_regret;

            let outcome = if success {
                RoutingOutcome::Success
            } else {
                RoutingOutcome::Rejected { reason: None }
            };
            router.record_outcome(decision.decision_id, outcome);

            if step == eval_points[0] {
                sum_regret_at_1000 += cumulative_regret;
            } else if step == eval_points[2] {
                sum_regret_at_20000 += cumulative_regret;
            }
        }
    }

    let avg_regret_1000 = sum_regret_at_1000 / num_seeds as f64;
    let avg_regret_20000 = sum_regret_at_20000 / num_seeds as f64;

    let rate_1000 = avg_regret_1000 / 1000.0;
    let rate_20000 = avg_regret_20000 / 20000.0;

    println!(
        "Average Regret ({} seeds): R(1000)={:.2} (rate={:.4}), R(20000)={:.2} (rate={:.4})",
        num_seeds, avg_regret_1000, rate_1000, avg_regret_20000, rate_20000
    );

    // Sublinearity assertion: Average regret rate per round decreases as T grows
    assert!(
        rate_20000 < rate_1000,
        "Regret must grow sublinearly: rate(20000)={:.4} should be less than rate(1000)={:.4}",
        rate_20000,
        rate_1000
    );

    Ok(())
}

#[tokio::test]
async fn router_eventually_prefers_best_arm() -> std::result::Result<(), Box<dyn std::error::Error>>
{
    let dim = 4;
    let true_means = vec![0.85f32, 0.70, 0.55, 0.40, 0.25];
    let env = SyntheticBanditEnv::new(true_means, dim);

    let seed = 42u64;
    let mut rng = SimpleRng::seed_from_u64(seed);
    let router = env.create_router(seed, 0.02);

    let total_rounds = 10000;
    let last_window = 2000;
    let query_embedding = vec![1.0f32; dim];

    let mut best_arm_chosen_in_window = 0;

    for step in 1..=total_rounds {
        let decision = router.route(&query_embedding, "query").await?;
        let arm_idx = profile_name_to_idx(&decision.profile.name)
            .ok_or_else(|| ContextraError::InvalidInput("Invalid profile name".into()))?;

        if step > (total_rounds - last_window) && arm_idx == env.best_arm_idx {
            best_arm_chosen_in_window += 1;
        }

        let (_, success) = env.sample_reward(arm_idx, &mut rng);
        let outcome = if success {
            RoutingOutcome::Success
        } else {
            RoutingOutcome::Rejected { reason: None }
        };
        router.record_outcome(decision.decision_id, outcome);
    }

    let best_arm_ratio = best_arm_chosen_in_window as f64 / last_window as f64;
    let random_level = 1.0 / env.num_arms() as f64; // 0.20 for K=5

    println!(
        "Best arm selection ratio in last {} rounds: {:.4} (random level: {:.2})",
        last_window, best_arm_ratio, random_level
    );

    // After 10,000 rounds, ratio of selecting best arm must be significantly above random (0.20), e.g. > 0.50
    assert!(
        best_arm_ratio > 0.50,
        "Router should prefer best arm: ratio {:.4} <= 0.50 (random level {:.2})",
        best_arm_ratio,
        random_level
    );

    Ok(())
}

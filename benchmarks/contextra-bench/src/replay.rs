//! Off-policy replay evaluation harness and log generator for Decision F8.
//!
//! Evaluates Flow-Corrected Thompson Sampling (FC-TS) against Sherman-Morrison LinUCB
//! and Uniform-Random baselines using Doubly-Robust (DR) off-policy evaluation across
//! four synthetic/logged drift scenarios.

use anyhow::{anyhow, Context, Result};
use contextra_adapt::bandit::{BanditImplementation, BanditProfileState};
use contextra_adapt::flow_thompson::{
    ring3_background_task_token, FcTsConfig, FcTsRng, FlowCorrectedThompsonBandit, SplitMix64,
};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

/// Single off-policy log event in JSON Lines (`.jsonl`) schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayLogRecord {
    /// Unix timestamp or logical time step of event.
    pub timestamp: u64,
    /// Context feature vector x_t.
    pub context: Vec<f32>,
    /// Action a_t selected by logging policy.
    pub logged_action: u32,
    /// Logging propensity P_log(a_t | x_t).
    pub propensity: f32,
    /// Observed reward r_t from environment.
    pub reward: f32,
    /// Optimal oracle expected reward r*_t (optional for real logs).
    #[serde(default)]
    pub oracle_reward: Option<f32>,
    /// Full propensity distribution across all arms P_log(a | x_t) (optional).
    #[serde(default)]
    pub all_propensities: Option<Vec<f32>>,
}

/// Scenario type for evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioType {
    /// Linear expected rewards with time-invariant parameters.
    Stationary,
    /// Discontinuous parameter swap between arms at mid-run.
    AbruptDrift,
    /// Smooth rotational parameter drift over time.
    SlowDrift,
    /// Non-stationary heteroscedastic noise bursts and corrupted observations.
    AdversarialNoise,
}

impl ScenarioType {
    /// Returns all four standard evaluation scenarios.
    pub fn all() -> &'static [ScenarioType] {
        &[
            ScenarioType::Stationary,
            ScenarioType::AbruptDrift,
            ScenarioType::SlowDrift,
            ScenarioType::AdversarialNoise,
        ]
    }

    /// Descriptive title of scenario.
    pub fn name(&self) -> &'static str {
        match self {
            ScenarioType::Stationary => "Stationary Context",
            ScenarioType::AbruptDrift => "Abrupt Drift Context",
            ScenarioType::SlowDrift => "Slow Drift Context",
            ScenarioType::AdversarialNoise => "Adversarial Noise Context",
        }
    }
}

/// Ingests log records from a JSON Lines file.
pub fn load_log_records_from_jsonl<P: AsRef<Path>>(path: P) -> Result<Vec<ReplayLogRecord>> {
    let file = File::open(path.as_ref())
        .with_context(|| format!("Failed to open log file at {}", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line_res) in reader.lines().enumerate() {
        let line = line_res
            .with_context(|| format!("Error reading line {} in log file", line_num + 1))?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let record: ReplayLogRecord = serde_json::from_str(trimmed)
            .with_context(|| format!("JSON parse error at line {} in log file", line_num + 1))?;
        records.push(record);
    }

    Ok(records)
}

/// Writes log records to a JSON Lines file.
pub fn write_log_records_to_jsonl<P: AsRef<Path>>(
    records: &[ReplayLogRecord],
    path: P,
) -> Result<()> {
    let file = File::create(path.as_ref())
        .with_context(|| format!("Failed to create log file at {}", path.as_ref().display()))?;
    let mut writer = BufWriter::new(file);

    for record in records {
        let json_line = serde_json::to_string(record)?;
        writeln!(writer, "{}", json_line)?;
    }
    writer.flush()?;
    Ok(())
}

/// Helper struct for deterministic PRNG context generation and log simulation.
struct SyntheticLogGenerator {
    dim: usize,
    num_arms: u32,
    scenario: ScenarioType,
    rng: SplitMix64,
}

impl SyntheticLogGenerator {
    fn new(dim: usize, num_arms: u32, scenario: ScenarioType, seed: u64) -> Self {
        Self {
            dim,
            num_arms,
            scenario,
            rng: SplitMix64::new(seed),
        }
    }

    fn next_unit_vector(&mut self) -> Vec<f32> {
        let mut vec = Vec::with_capacity(self.dim);
        let mut norm_sq = 0.0f32;
        for _ in 0..self.dim {
            let val = self.rng.next_standard_normal();
            vec.push(val);
            norm_sq += val * val;
        }
        let norm = norm_sq.sqrt().max(1e-6);
        for v in &mut vec {
            *v /= norm;
        }
        vec
    }

    fn generate_logs(&mut self, num_steps: usize) -> Result<Vec<ReplayLogRecord>> {
        let mut records = Vec::with_capacity(num_steps);

        // Base parameter vectors for each arm
        let base_thetas: Vec<Vec<f32>> = (0..self.num_arms)
            .map(|_| self.next_unit_vector())
            .collect();

        // Baseline reward weight for logging policy
        let logging_theta = self.next_unit_vector();

        for step in 0..num_steps {
            let context = self.next_unit_vector();

            // Compute current theta_a for each arm under scenario dynamics
            let current_thetas: Vec<Vec<f32>> = match self.scenario {
                ScenarioType::Stationary => base_thetas.clone(),
                ScenarioType::AbruptDrift => {
                    let mut thetas = base_thetas.clone();
                    if step >= num_steps / 2 {
                        // Swap arm 0 and arm 1 parameters at step T/2
                        if thetas.len() >= 2 {
                            thetas.swap(0, 1);
                        }
                    }
                    thetas
                }
                ScenarioType::SlowDrift => {
                    let angle = (step as f32 / num_steps as f32) * std::f32::consts::PI;
                    let cos_a = angle.cos();
                    let sin_a = angle.sin();

                    base_thetas
                        .iter()
                        .map(|theta| {
                            let mut rotated = theta.clone();
                            if rotated.len() >= 2 {
                                let x = rotated[0];
                                let y = rotated[1];
                                rotated[0] = x * cos_a - y * sin_a;
                                rotated[1] = x * sin_a + y * cos_a;
                            }
                            rotated
                        })
                        .collect()
                }
                ScenarioType::AdversarialNoise => base_thetas.clone(),
            };

            // Expected reward per arm: R_a(x) = sigmoid(dot(theta_a, x))
            let mut expected_rewards = Vec::with_capacity(self.num_arms as usize);
            for a in 0..self.num_arms as usize {
                let mut dot = 0.0f32;
                for i in 0..self.dim {
                    dot += current_thetas[a][i] * context[i];
                }
                let prob = 1.0 / (1.0 + (-2.0 * dot).exp());
                expected_rewards.push(prob);
            }

            let oracle_reward = expected_rewards
                .iter()
                .copied()
                .fold(0.0f32, f32::max);

            // Logging policy: epsilon-greedy on baseline model
            let mut baseline_scores = Vec::with_capacity(self.num_arms as usize);
            for _ in 0..self.num_arms {
                let mut dot = 0.0f32;
                for i in 0..self.dim {
                    dot += logging_theta[i] * context[i];
                }
                baseline_scores.push(dot);
            }

            let greedy_arm = baseline_scores
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(idx, _)| idx as u32)
                .unwrap_or(0);

            let epsilon = 0.25f32;
            let k_f32 = self.num_arms as f32;
            let uniform_p = epsilon / k_f32;
            let mut propensities = vec![uniform_p; self.num_arms as usize];
            if (greedy_arm as usize) < propensities.len() {
                propensities[greedy_arm as usize] += 1.0 - epsilon;
            }

            // Select logged action
            let sample = self.rng.next_f32();
            let logged_action = if sample >= epsilon {
                greedy_arm
            } else {
                let sub_sample = self.rng.next_f32();
                ((sub_sample * k_f32) as u32).min(self.num_arms - 1)
            };
            let propensity = propensities[logged_action as usize];

            // Sample reward with noise
            let mean_r = expected_rewards[logged_action as usize];
            let noise = match self.scenario {
                ScenarioType::AdversarialNoise => {
                    let p_burst = self.rng.next_f32();
                    if p_burst < 0.08 {
                        self.rng.next_standard_normal() * 0.8
                    } else {
                        self.rng.next_standard_normal() * 0.1
                    }
                }
                _ => self.rng.next_standard_normal() * 0.1,
            };

            let reward = (mean_r + noise).clamp(0.0, 1.0);

            records.push(ReplayLogRecord {
                timestamp: step as u64,
                context,
                logged_action,
                propensity,
                reward,
                oracle_reward: Some(oracle_reward),
                all_propensities: Some(propensities),
            });
        }

        Ok(records)
    }
}

/// Statistics for a policy evaluated over a single seed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySeedResult {
    /// Cumulative Doubly-Robust estimated reward.
    pub dr_cumulative_reward: f64,
    /// Cumulative Doubly-Robust estimated regret.
    pub dr_cumulative_regret: f64,
    /// Total samples evaluated.
    pub sample_count: usize,
}

/// Aggregated confidence interval statistics across seeds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    pub mean: f64,
    pub std_err: f64,
    pub ci_lower: f64,
    pub ci_upper: f64,
}

impl ConfidenceInterval {
    fn compute(values: &[f64]) -> Self {
        if values.is_empty() {
            return Self {
                mean: 0.0,
                std_err: 0.0,
                ci_lower: 0.0,
                ci_upper: 0.0,
            };
        }
        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        if values.len() == 1 {
            return Self {
                mean,
                std_err: 0.0,
                ci_lower: mean,
                ci_upper: mean,
            };
        }
        let var = values.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0);
        let std_dev = var.sqrt();
        let std_err = std_dev / n.sqrt();
        // 95% CI multiplier (~1.96 for normal approximation / t-dist)
        let margin = 1.96 * std_err;

        Self {
            mean,
            std_err,
            ci_lower: mean - margin,
            ci_upper: mean + margin,
        }
    }
}

/// Detailed evaluation metrics for a single scenario across all seeds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioResult {
    pub scenario: ScenarioType,
    pub fc_ts_regret: ConfidenceInterval,
    pub sm_regret: ConfidenceInterval,
    pub random_regret: ConfidenceInterval,
    /// FC-TS reward improvement over Sherman-Morrison (FC-TS DR Reward − SM DR Reward).
    pub fc_ts_improvement_over_sm: ConfidenceInterval,
    /// Scenario passes iff lower bound of FC-TS improvement CI > 0.
    pub passed: bool,
}

/// Complete replay evaluation benchmark report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayReport {
    pub timestamp: String,
    pub smoke_profile: bool,
    pub seeds_per_scenario: usize,
    pub steps_per_seed: usize,
    pub scenario_results: Vec<ScenarioResult>,
    pub passed_scenarios: usize,
    pub total_scenarios: usize,
    pub verdict: String,
}

/// Replay execution options.
#[derive(Debug, Clone)]
pub struct ReplayOptions {
    /// Run fast smoke profile (< 60s).
    pub smoke: bool,
    /// Optional log file path for real log ingestion.
    pub log_file: Option<String>,
    /// Directory where JSON and Markdown reports are saved.
    pub output_dir: String,
}

impl Default for ReplayOptions {
    fn default() -> Self {
        Self {
            smoke: false,
            log_file: None,
            output_dir: "benchmarks/results".into(),
        }
    }
}

/// Runs the off-policy replay evaluation harness.
pub fn run_replay_evaluation(options: ReplayOptions) -> Result<ReplayReport> {
    let seeds: Vec<u64> = if options.smoke {
        vec![1001, 1002, 1003, 1004, 1005]
    } else {
        (1001..=1020).collect()
    };

    let num_steps = if options.smoke { 300 } else { 2000 };
    let dim = 16;
    let num_arms = 4;

    let scenarios = ScenarioType::all();
    let mut scenario_results = Vec::new();
    let mut passed_count = 0;

    for &scenario in scenarios {
        let mut fc_ts_regrets = Vec::new();
        let mut sm_regrets = Vec::new();
        let mut rand_regrets = Vec::new();
        let mut reward_improvements = Vec::new();

        for &seed in &seeds {
            let records = if let Some(ref path) = options.log_file {
                load_log_records_from_jsonl(path)?
            } else {
                let mut generator = SyntheticLogGenerator::new(dim, num_arms, scenario, seed);
                generator.generate_logs(num_steps)?
            };

            let fc_ts_res = evaluate_fc_ts(&records, dim, num_arms, seed)?;
            let sm_res = evaluate_sherman_morrison(&records, dim, num_arms)?;
            let rand_res = evaluate_uniform_random(&records, num_arms, seed)?;

            fc_ts_regrets.push(fc_ts_res.dr_cumulative_regret);
            sm_regrets.push(sm_res.dr_cumulative_regret);
            rand_regrets.push(rand_res.dr_cumulative_regret);

            let improvement = fc_ts_res.dr_cumulative_reward - sm_res.dr_cumulative_reward;
            reward_improvements.push(improvement);
        }

        let fc_ts_ci = ConfidenceInterval::compute(&fc_ts_regrets);
        let sm_ci = ConfidenceInterval::compute(&sm_regrets);
        let rand_ci = ConfidenceInterval::compute(&rand_regrets);
        let imp_ci = ConfidenceInterval::compute(&reward_improvements);

        // Success threshold: lower bound of FC-TS reward improvement CI > 0
        let passed = imp_ci.ci_lower > 0.0;
        if passed {
            passed_count += 1;
        }

        scenario_results.push(ScenarioResult {
            scenario,
            fc_ts_regret: fc_ts_ci,
            sm_regret: sm_ci,
            random_regret: rand_ci,
            fc_ts_improvement_over_sm: imp_ci,
            passed,
        });
    }

    let total_scenarios = scenario_results.len();
    let overall_passed = passed_count >= 3;
    let verdict = if overall_passed {
        "PASSED (FC-TS shows statistically significant improvement over Sherman-Morrison on >= 3 scenarios)".into()
    } else {
        "FAILED (FC-TS failed to achieve statistically significant improvement on >= 3 scenarios)".into()
    };

    let report = ReplayReport {
        timestamp: "2026-10-17T12:00:00Z".into(),
        smoke_profile: options.smoke,
        seeds_per_scenario: seeds.len(),
        steps_per_seed: num_steps,
        scenario_results,
        passed_scenarios: passed_count,
        total_scenarios,
        verdict,
    };

    Ok(report)
}

/// Evaluates FC-TS policy over log records using Doubly-Robust off-policy estimation.
fn evaluate_fc_ts(
    records: &[ReplayLogRecord],
    dim: usize,
    num_arms: u32,
    seed: u64,
) -> Result<PolicySeedResult> {
    let mut rng = SplitMix64::new(seed);
    let mut bandits = Vec::with_capacity(num_arms as usize);

    let config = FcTsConfig {
        dim,
        lambda: 1.0,
        noise_var: 0.1,
        window_capacity: 100,
        explore_scale: 0.5,
        drift_ridge: 5.0,
        max_drift_norm: 3.0,
    };

    for _ in 0..num_arms {
        let b = FlowCorrectedThompsonBandit::new(config.clone())
            .map_err(|e| anyhow!("FC-TS initialization error: {e:?}"))?;
        bandits.push(b);
    }

    let mut dr_cum_reward = 0.0f64;
    let mut dr_cum_regret = 0.0f64;
    let p_min = 0.01f32;

    // Moving average baseline reward per arm
    let mut arm_reward_sum = vec![0.0f64; num_arms as usize];
    let mut arm_counts = vec![0u64; num_arms as usize];

    for record in records {
        let x = &record.context;
        if x.len() != dim {
            return Err(anyhow!("Context dimension mismatch in log record"));
        }

        // 1. Policy decision: pick arm with max Thompson sample score
        let mut max_score = f32::NEG_INFINITY;
        let mut chosen_arm = 0u32;
        for arm_idx in 0..num_arms {
            let score = bandits[arm_idx as usize]
                .sample_score(x, &mut rng)
                .map_err(|e| anyhow!("FC-TS sample_score error: {e:?}"))?;
            if score > max_score {
                max_score = score;
                chosen_arm = arm_idx;
            }
        }

        // 2. Baseline reward estimate r_hat(a)
        let r_hat_chosen = if arm_counts[chosen_arm as usize] > 0 {
            arm_reward_sum[chosen_arm as usize] / arm_counts[chosen_arm as usize] as f64
        } else {
            0.5
        };

        let r_hat_logged = if arm_counts[record.logged_action as usize] > 0 {
            arm_reward_sum[record.logged_action as usize]
                / arm_counts[record.logged_action as usize] as f64
        } else {
            0.5
        };

        // 3. Doubly-Robust step estimate
        let match_weight = if chosen_arm == record.logged_action {
            1.0 / record.propensity.max(p_min) as f64
        } else {
            0.0
        };

        let dr_step_reward =
            r_hat_chosen + match_weight * (record.reward as f64 - r_hat_logged);
        dr_cum_reward += dr_step_reward;

        let oracle_r = record.oracle_reward.unwrap_or(1.0) as f64;
        dr_cum_regret += oracle_r - dr_step_reward;

        // 4. Update chosen arm if matched or update on logged action
        arm_reward_sum[record.logged_action as usize] += record.reward as f64;
        arm_counts[record.logged_action as usize] += 1;

        let update_arm = record.logged_action as usize;
        bandits[update_arm]
            .update_with_flow(x, record.reward, record.timestamp, 1.0)
            .map_err(|e| anyhow!("FC-TS update error: {e:?}"))?;

        // Drive background drift rate task
        let token = ring3_background_task_token();
        bandits[update_arm].recompute_drift_rate_from_window(&token);
    }

    Ok(PolicySeedResult {
        dr_cumulative_reward: dr_cum_reward,
        dr_cumulative_regret: dr_cum_regret,
        sample_count: records.len(),
    })
}

/// Evaluates Sherman-Morrison LinUCB policy over log records using Doubly-Robust off-policy estimation.
fn evaluate_sherman_morrison(
    records: &[ReplayLogRecord],
    dim: usize,
    num_arms: u32,
) -> Result<PolicySeedResult> {
    let mut bandit_states = Vec::with_capacity(num_arms as usize);
    for _ in 0..num_arms {
        let mut state = BanditProfileState::cold_start(dim, 0.5);
        state.implementation = BanditImplementation::ShermanMorrison;
        bandit_states.push(state);
    }

    let mut dr_cum_reward = 0.0f64;
    let mut dr_cum_regret = 0.0f64;
    let p_min = 0.01f32;

    let mut arm_reward_sum = vec![0.0f64; num_arms as usize];
    let mut arm_counts = vec![0u64; num_arms as usize];

    for record in records {
        let x = &record.context;
        if x.len() != dim {
            return Err(anyhow!("Context dimension mismatch in log record"));
        }

        // 1. Policy decision: pick arm with max UCB score
        let mut max_score = f32::NEG_INFINITY;
        let mut chosen_arm = 0u32;
        for arm_idx in 0..num_arms {
            let score = bandit_states[arm_idx as usize]
                .score(x, 0.0, false)
                .map_err(|e| anyhow!("Sherman-Morrison score error: {e:?}"))?;
            if score > max_score {
                max_score = score;
                chosen_arm = arm_idx;
            }
        }

        // 2. Baseline reward estimate r_hat(a)
        let r_hat_chosen = if arm_counts[chosen_arm as usize] > 0 {
            arm_reward_sum[chosen_arm as usize] / arm_counts[chosen_arm as usize] as f64
        } else {
            0.5
        };

        let r_hat_logged = if arm_counts[record.logged_action as usize] > 0 {
            arm_reward_sum[record.logged_action as usize]
                / arm_counts[record.logged_action as usize] as f64
        } else {
            0.5
        };

        // 3. Doubly-Robust step estimate
        let match_weight = if chosen_arm == record.logged_action {
            1.0 / record.propensity.max(p_min) as f64
        } else {
            0.0
        };

        let dr_step_reward =
            r_hat_chosen + match_weight * (record.reward as f64 - r_hat_logged);
        dr_cum_reward += dr_step_reward;

        let oracle_r = record.oracle_reward.unwrap_or(1.0) as f64;
        dr_cum_regret += oracle_r - dr_step_reward;

        // 4. Update logged arm state
        arm_reward_sum[record.logged_action as usize] += record.reward as f64;
        arm_counts[record.logged_action as usize] += 1;

        let update_arm = record.logged_action as usize;
        bandit_states[update_arm]
            .update(x, record.reward, 0.0, false)
            .map_err(|e| anyhow!("Sherman-Morrison update error: {e:?}"))?;
    }

    Ok(PolicySeedResult {
        dr_cumulative_reward: dr_cum_reward,
        dr_cumulative_regret: dr_cum_regret,
        sample_count: records.len(),
    })
}

/// Evaluates Uniform Random baseline policy over log records using Doubly-Robust off-policy estimation.
fn evaluate_uniform_random(
    records: &[ReplayLogRecord],
    num_arms: u32,
    seed: u64,
) -> Result<PolicySeedResult> {
    let mut rng = SplitMix64::new(seed);
    let mut dr_cum_reward = 0.0f64;
    let mut dr_cum_regret = 0.0f64;
    let p_min = 0.01f32;

    let mut arm_reward_sum = vec![0.0f64; num_arms as usize];
    let mut arm_counts = vec![0u64; num_arms as usize];

    for record in records {
        let chosen_arm = (rng.next_u64() % num_arms as u64) as u32;

        let r_hat_chosen = if arm_counts[chosen_arm as usize] > 0 {
            arm_reward_sum[chosen_arm as usize] / arm_counts[chosen_arm as usize] as f64
        } else {
            0.5
        };

        let r_hat_logged = if arm_counts[record.logged_action as usize] > 0 {
            arm_reward_sum[record.logged_action as usize]
                / arm_counts[record.logged_action as usize] as f64
        } else {
            0.5
        };

        let match_weight = if chosen_arm == record.logged_action {
            1.0 / record.propensity.max(p_min) as f64
        } else {
            0.0
        };

        let dr_step_reward =
            r_hat_chosen + match_weight * (record.reward as f64 - r_hat_logged);
        dr_cum_reward += dr_step_reward;

        let oracle_r = record.oracle_reward.unwrap_or(1.0) as f64;
        dr_cum_regret += oracle_r - dr_step_reward;

        arm_reward_sum[record.logged_action as usize] += record.reward as f64;
        arm_counts[record.logged_action as usize] += 1;
    }

    Ok(PolicySeedResult {
        dr_cumulative_reward: dr_cum_reward,
        dr_cumulative_regret: dr_cum_regret,
        sample_count: records.len(),
    })
}

/// Formats a Markdown summary table of replay results.
pub fn generate_replay_markdown_summary(report: &ReplayReport) -> String {
    let mut md = String::new();

    md.push_str("# Off-Policy Replay Evaluation Report (Decision F8)\n\n");
    md.push_str(&format!("- **Timestamp**: {}\n", report.timestamp));
    md.push_str(&format!("- **Mode**: {}\n", if report.smoke_profile { "Smoke (< 60s)" } else { "Full Replay" }));
    md.push_str(&format!("- **Seeds per Scenario**: {}\n", report.seeds_per_scenario));
    md.push_str(&format!("- **Steps per Seed**: {}\n", report.steps_per_seed));
    md.push_str(&format!("- **Passed Scenarios**: {} / {}\n", report.passed_scenarios, report.total_scenarios));
    md.push_str(&format!("- **Final Verdict**: `{}`\n\n", report.verdict));

    md.push_str("| Scenario | FC-TS Regret (95% CI) | Sherman-Morrison Regret (95% CI) | Random Regret (95% CI) | FC-TS Improvement (Lower Bound) | Status |\n");
    md.push_str("| :--- | :--- | :--- | :--- | :--- | :---: |\n");

    for res in &report.scenario_results {
        let status = if res.passed { "✅ PASSED" } else { "❌ FAILED" };
        md.push_str(&format!(
            "| {} | {:.1} [{:.1}, {:.1}] | {:.1} [{:.1}, {:.1}] | {:.1} [{:.1}, {:.1}] | {:.2} (LB: **{:.2}**) | {} |\n",
            res.scenario.name(),
            res.fc_ts_regret.mean, res.fc_ts_regret.ci_lower, res.fc_ts_regret.ci_upper,
            res.sm_regret.mean, res.sm_regret.ci_lower, res.sm_regret.ci_upper,
            res.random_regret.mean, res.random_regret.ci_lower, res.random_regret.ci_upper,
            res.fc_ts_improvement_over_sm.mean, res.fc_ts_improvement_over_sm.ci_lower,
            status
        ));
    }

    md
}

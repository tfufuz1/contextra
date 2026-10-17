# Off-Policy Replay Protocol & Evaluation Criterion (Decision F8)

## 1. Context & Objective
Per Decision F8, Flow-Corrected Thompson Sampling (`FC-TS`) remains disabled by default until an offline replay experiment demonstrates a statistically significant performance advantage over the default Sherman–Morrison LinUCB bandit.

This protocol fixes the measurement criterion, metrics, success thresholds, scenario definitions, and data schemas prior to running evaluation experiments.

---

## 2. Evaluation Metric
The primary metric for policy comparison is **Cumulative Regret Estimated via Doubly-Robust (DR) Off-Policy Evaluation**:

$$\hat{R}_{\text{DR}}(\pi) = \sum_{i=1}^N \left[ \hat{r}_i(\pi(x_i)) + \frac{\mathbb{I}(\pi(x_i) = a_i)}{\max(p_i, p_{\text{min}})} (r_i - \hat{r}_i(a_i)) \right]$$

$$\text{Regret}_{\text{DR}}(\pi) = \sum_{i=1}^N r^*_i - \hat{R}_{\text{DR}}(\pi)$$

Where:
- $x_i$: Context vector at step $i$.
- $a_i$: Logged action taken by logging policy.
- $p_i$: Logging propensity $P_{\text{log}}(a_i | x_i)$, clamped to $p_{\text{min}} = 0.01$ to prevent variance explosion.
- $r_i$: Observed reward from log.
- $\hat{r}_i(a)$: Baseline model expected reward estimate for arm $a$ given $x_i$.
- $r^*_i$: Oracle optimal expected reward at step $i$.

The reward improvement of policy $\pi_1$ (FC-TS) over policy $\pi_0$ (Sherman-Morrison) on seed $s$ is:

$$\Delta_s = \hat{R}_{\text{DR}, s}(\text{FC-TS}) - \hat{R}_{\text{DR}, s}(\text{Sherman-Morrison})$$

---

## 3. Success Threshold & Decision Criteria
Across $S$ independent fixed-seed evaluation runs ($S \ge 20$ for standard evaluation, $S \ge 5$ for `--smoke` profile), we compute the sample mean $\bar{\Delta}$ and 95% Confidence Interval (CI):

$$\text{CI}_{95\%}(\Delta) = \left[ \bar{\Delta} - 1.96 \cdot \frac{s_{\Delta}}{\sqrt{S}}, \bar{\Delta} + 1.96 \cdot \frac{s_{\Delta}}{\sqrt{S}} \right]$$

- **Scenario Success Criterion**: A scenario passes iff the **lower bound** of the 95% CI of reward improvement is strictly greater than 0:

$$\text{LowerBound}_{95\%}(\Delta_{\text{scenario}}) > 0.0$$

- **Overall Verdict**:
  - `PASSED`: FC-TS satisfies the success criterion on **at least 3 out of 4 scenarios**.
  - `FAILED`: FC-TS fails to satisfy the success criterion on fewer than 3 scenarios.

---

## 4. Evaluation Scenarios
Evaluation is conducted across 4 distinct synthetic/replay environments:

1. **Stationary Context (`stationary`)**:
   - Linear expected rewards $R_a(x) = \theta_a^T x$ with fixed, time-invariant parameters $\theta_a$.
2. **Abrupt Drift Context (`abrupt_drift`)**:
   - Time-invariant parameters for $t < T/2$, followed by an instantaneous parameter swap/permutation between optimal and sub-optimal arms at $t = T/2$.
3. **Slow Drift Context (`slow_drift`)**:
   - Smooth rotational drift of context parameters over time $t$: $\theta_a(t) = R(\omega t) \theta_a(0)$.
4. **Adversarial-Noise Context (`adversarial_noise`)**:
   - Non-stationary heteroscedastic noise bursts, heavy-tailed noise spikes, and corrupted reward observations.

---

## 5. Experimental Conditions & Seeds
- **Seeds**: Fixed seeds (1001, 1002, ..., 1020 for full replay; 1001..1005 for `--smoke`).
- **Rounds per Seed**:
  - Standard run: $N = 2,000$ steps per scenario seed ($2,000 \times 20 = 40,000$ steps per scenario).
  - Smoke profile (`--smoke`): $N = 300$ steps per scenario seed ($300 \times 5 = 1,500$ steps per scenario), budget < 60 seconds.
- **Compared Policies**:
  1. `Sherman-Morrison`: LinUCB with Sherman-Morrison matrix updates ($O(d^2)$).
  2. `FC-TS`: Flow-Corrected Thompson Sampling with active drift rate task (`recompute_drift_rate_from_window` driven by harness).
  3. `Uniform-Random`: Baseline selecting arms uniformly at random.

---

## 6. Log File Schema (JSON Lines)
For real log ingestion and synthetic log export, events follow the JSON Lines (`.jsonl`) schema:

```json
{
  "timestamp": 1700000000,
  "context": [0.12, -0.45, 0.78, ...],
  "logged_action": 1,
  "propensity": 0.25,
  "reward": 0.85,
  "oracle_reward": 0.95,
  "all_propensities": [0.25, 0.25, 0.25, 0.25]
}
```

---

## 7. Reporting & Outputs
- **JSON Report**: Saved to `benchmarks/results/replay_report.json` with detailed per-scenario metrics, confidence intervals, and verdict.
- **Markdown Report**: Saved to `benchmarks/results/replay_report.md` (and printed to console) formatted with summary tables.

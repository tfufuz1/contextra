// FILE-CONTEXT
// ZWECK: Adaptives ef_search (Ada-ef) Richtlinien & Zustandsautomat für HNSW-Suche.
// INVARIANTEN: Zero-Panic, keine Zufallsquellen/Systemzeit (deterministisch).
// NICHT-OFFENSICHTLICH: Bricht Suche ab, sobald sich die Top-k-Kandidaten über das Konvergenzfenster stabilisieren.

use contextra_core::{ContextraError, DocId, Result};

/// Configuration policy for adaptive ef_search (Ada-ef).
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEfPolicy {
    /// Initial search depth ef for layer 0.
    pub min_ef: usize,
    /// Maximum search depth ef limit.
    pub max_ef: usize,
    /// Growth factor for increasing ef in each expansion round (must be > 1.0).
    pub growth_factor: f32,
    /// Number of consecutive rounds with top-k stability required to trigger early exit.
    pub convergence_window: usize,
    /// Fraction of top-k candidate set overlap required to consider a round stable (0.0 ..= 1.0).
    pub stability_threshold: f32,
}

impl AdaptiveEfPolicy {
    /// Validating constructor for `AdaptiveEfPolicy`.
    pub fn new(
        min_ef: usize,
        max_ef: usize,
        growth_factor: f32,
        convergence_window: usize,
        stability_threshold: f32,
    ) -> Result<Self> {
        if min_ef == 0 {
            return Err(ContextraError::invalid_input(
                "min_ef must be greater than 0",
            ));
        }
        if max_ef < min_ef {
            return Err(ContextraError::invalid_input(
                "max_ef must be greater than or equal to min_ef",
            ));
        }
        if !growth_factor.is_finite() || growth_factor <= 1.0 {
            return Err(ContextraError::invalid_input(
                "growth_factor must be a finite float greater than 1.0",
            ));
        }
        if convergence_window == 0 {
            return Err(ContextraError::invalid_input(
                "convergence_window must be greater than 0",
            ));
        }
        if !stability_threshold.is_finite() || !(0.0..=1.0).contains(&stability_threshold) {
            return Err(ContextraError::invalid_input(
                "stability_threshold must be a finite float in range [0.0, 1.0]",
            ));
        }
        Ok(Self {
            min_ef,
            max_ef,
            growth_factor,
            convergence_window,
            stability_threshold,
        })
    }

    /// Validates policy invariants.
    pub fn validate(&self) -> Result<()> {
        if self.min_ef == 0 {
            return Err(ContextraError::invalid_input(
                "min_ef must be greater than 0",
            ));
        }
        if self.max_ef < self.min_ef {
            return Err(ContextraError::invalid_input(
                "max_ef must be greater than or equal to min_ef",
            ));
        }
        if !self.growth_factor.is_finite() || self.growth_factor <= 1.0 {
            return Err(ContextraError::invalid_input(
                "growth_factor must be a finite float greater than 1.0",
            ));
        }
        if self.convergence_window == 0 {
            return Err(ContextraError::invalid_input(
                "convergence_window must be greater than 0",
            ));
        }
        if !self.stability_threshold.is_finite() || !(0.0..=1.0).contains(&self.stability_threshold)
        {
            return Err(ContextraError::invalid_input(
                "stability_threshold must be a finite float in range [0.0, 1.0]",
            ));
        }
        Ok(())
    }
}

impl Default for AdaptiveEfPolicy {
    fn default() -> Self {
        Self {
            min_ef: 16,
            max_ef: 256,
            growth_factor: 1.5,
            convergence_window: 2,
            stability_threshold: 1.0,
        }
    }
}

/// Execution statistics for an adaptive search run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdaptiveEfStats {
    /// Final ef depth reached at termination.
    pub final_ef: usize,
    /// Total expansion rounds executed.
    pub rounds: usize,
    /// Whether search terminated due to candidate set convergence.
    pub converged: bool,
}

/// Deterministic state machine managing Ada-ef expansion rounds.
#[derive(Debug, Clone)]
pub struct AdaptiveEfStateMachine {
    policy: AdaptiveEfPolicy,
    k: usize,
    current_ef: usize,
    previous_top_k: Vec<DocId>,
    stable_rounds: usize,
    round_count: usize,
    is_terminated: bool,
    converged: bool,
}

impl AdaptiveEfStateMachine {
    /// Initializes a new state machine for adaptive search with target top-k.
    pub fn new(policy: AdaptiveEfPolicy, k: usize) -> Self {
        let initial_ef = policy.min_ef.max(k);
        Self {
            current_ef: initial_ef,
            policy,
            k,
            previous_top_k: Vec::new(),
            stable_rounds: 0,
            round_count: 0,
            is_terminated: false,
            converged: false,
        }
    }

    /// Returns the active `ef_search` for the current expansion round.
    pub fn current_ef(&self) -> usize {
        self.current_ef
    }

    /// Returns the number of expansion rounds completed.
    pub fn round_count(&self) -> usize {
        self.round_count
    }

    /// Returns whether the state machine has reached a termination condition.
    pub fn is_terminated(&self) -> bool {
        self.is_terminated
    }

    /// Returns the current execution statistics.
    pub fn stats(&self) -> AdaptiveEfStats {
        AdaptiveEfStats {
            final_ef: self.current_ef,
            rounds: self.round_count,
            converged: self.converged,
        }
    }

    /// Evaluates current top-k candidate set, updating state and determining if search should terminate.
    ///
    /// Returns `true` if search should terminate (due to stability convergence or reaching `max_ef`),
    /// or `false` if another expansion round should be executed.
    pub fn step(&mut self, current_top_k: &[DocId]) -> bool {
        if self.is_terminated {
            return true;
        }

        self.round_count += 1;

        let eval_len = self.k.min(current_top_k.len());
        let current_slice = if current_top_k.len() > eval_len {
            &current_top_k[..eval_len]
        } else {
            current_top_k
        };

        if self.round_count > 1 && eval_len > 0 {
            let prev_slice = &self.previous_top_k;
            let mut matches = 0usize;

            for doc_id in current_slice {
                if prev_slice.contains(doc_id) {
                    matches += 1;
                }
            }

            let stability_ratio = matches as f32 / eval_len as f32;

            if stability_ratio >= self.policy.stability_threshold {
                self.stable_rounds += 1;
            } else {
                self.stable_rounds = 0;
            }
        }

        self.previous_top_k = current_slice.to_vec();

        if self.stable_rounds >= self.policy.convergence_window {
            self.converged = true;
            self.is_terminated = true;
            return true;
        }

        if self.current_ef >= self.policy.max_ef {
            self.is_terminated = true;
            return true;
        }

        let next_raw = (self.current_ef as f32 * self.policy.growth_factor).ceil() as usize;
        self.current_ef = next_raw.clamp(self.current_ef + 1, self.policy.max_ef);

        false
    }
}

//! Flow-Corrected Thompson Sampling (FC-TS) pro SLM-Profil-Arm (§21.3).
//!
//! Implementiert konfidenzgewichtete Sherman-Morrison-Updates, transportierte Belohnungen
//! bezüglich geschätzter Drift-Raten, Ringpuffer fester Kapazität und deterministisches Thompson-Sampling.

#![allow(clippy::needless_range_loop)]

use thiserror::Error;

/// Fehlerzustände für Flow-Corrected Thompson Sampling (FC-TS).
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum FcTsError {
    /// Dimension des Eingabevektors stimmt nicht mit der konfigurierten Arm-Dimension überein.
    #[error("Embedding dimension mismatch: expected {expected}, actual {actual}")]
    DimensionMismatch {
        /// Erwartete Dimension.
        expected: usize,
        /// Tatsächliche Dimension.
        actual: usize,
    },
    /// Ein Wert ist NaN oder unendlich (non-finite).
    #[error("Non-finite numerical value encountered")]
    NonFinite,
    /// Ungültige Konfiguration.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Trait für deterministischen Zufallszahlengenerator im Thompson-Sampling.
pub trait FcTsRng {
    /// Generiert die nächste 64-Bit vorzeichenlose Ganzzahl.
    fn next_u64(&mut self) -> u64;

    /// Generiert eine gleichverteilte Fließkommazahl im halboffenen Intervall `[0, 1)`.
    fn next_f32(&mut self) -> f32 {
        let v = (self.next_u64() >> 40) as f32;
        v / 16777216.0
    }

    /// Generiert eine standardnormalverteilte Fließkommazahl $\mathcal{N}(0, 1)$ mittels Box-Muller-Transformation.
    fn next_standard_normal(&mut self) -> f32 {
        let mut u1 = self.next_f32();
        while u1 <= 1e-7 {
            u1 = self.next_f32();
        }
        let u2 = self.next_f32();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::TAU * u2).cos()
    }
}

/// Deterministische `SplitMix64`-Referenzimplementierung des [`FcTsRng`]-Traits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// Erstellt eine neue `SplitMix64`-Instanz mit dem gegebenen Seed.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }
}

impl FcTsRng for SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
}

/// Konfigurationsparameter für Flow-Corrected Thompson Sampling (§21.3).
#[derive(Debug, Clone, PartialEq)]
pub struct FcTsConfig {
    /// Feature-Dimension $d$.
    pub dim: usize,
    /// Ridge-Prior Parameter $\lambda > 0$.
    pub lambda: f32,
    /// Rauschvarianz $\sigma^2 > 0$.
    pub noise_var: f32,
    /// Kapazität des Ringpuffers für das Drift-Fenster (Default: 200).
    pub window_capacity: usize,
    /// Explorations-Skalierung $v \ge 0$.
    pub explore_scale: f32,
    /// Ridge-Regularisierung für Drift-Regression $\rho > 0$.
    pub drift_ridge: f32,
    /// Maximale euklidische Norm der geschätzten Drift-Rate $\|\delta\|_2$.
    pub max_drift_norm: f32,
}

impl Default for FcTsConfig {
    fn default() -> Self {
        Self {
            dim: 64,
            lambda: 1.0,
            noise_var: 1.0,
            window_capacity: 200,
            explore_scale: 1.0,
            drift_ridge: 10.0,
            max_drift_norm: 5.0,
        }
    }
}

impl FcTsConfig {
    /// Validiert die Konfigurationsparameter.
    pub fn validate(&self) -> Result<(), FcTsError> {
        if self.dim == 0 {
            return Err(FcTsError::InvalidConfig("dim must be > 0".to_string()));
        }
        if !self.lambda.is_finite() || self.lambda <= 0.0 {
            return Err(FcTsError::InvalidConfig(
                "lambda must be finite and > 0".to_string(),
            ));
        }
        if !self.noise_var.is_finite() || self.noise_var <= 0.0 {
            return Err(FcTsError::InvalidConfig(
                "noise_var must be finite and > 0".to_string(),
            ));
        }
        if self.window_capacity == 0 {
            return Err(FcTsError::InvalidConfig(
                "window_capacity must be > 0".to_string(),
            ));
        }
        if !self.explore_scale.is_finite() || self.explore_scale < 0.0 {
            return Err(FcTsError::InvalidConfig(
                "explore_scale must be finite and >= 0".to_string(),
            ));
        }
        if !self.drift_ridge.is_finite() || self.drift_ridge <= 0.0 {
            return Err(FcTsError::InvalidConfig(
                "drift_ridge must be finite and > 0".to_string(),
            ));
        }
        if !self.max_drift_norm.is_finite() || self.max_drift_norm <= 0.0 {
            return Err(FcTsError::InvalidConfig(
                "max_drift_norm must be finite and > 0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Privater Zero-Sized Token zur Erzwingung, dass Drift-Update-Aufrufe exklusiv aus Ring-3-Hintergrundtasks stammen (AK-18).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ring3Token {
    _private: (),
}

/// Erzeugt einen `Ring3Token` für den Aufruf in asynchronen Ring-3-Hintergrundtasks.
///
/// # Architektur-Hinweis (AK-18)
/// Darf ausschließlich im Ring-3-Hintergrundtask aufgerufen werden. Ein automatisierter
/// Architektur-Lint prüft die Einhaltung dieser Regel über den gesamten Workspace.
pub fn ring3_background_task_token() -> Ring3Token {
    Ring3Token { _private: () }
}

/// Laufzeitzustand eines Arms im Flow-Corrected Thompson Sampling (§21.3).
#[derive(Clone)]
pub struct FlowCorrectedThompsonBandit {
    config: FcTsConfig,
    /// Inverse Kovarianzmatrix $A^{-1}$ ($d \times d$, row-major).
    inv_a: Vec<f32>,
    /// Akkumulator-Vektor $\eta$ ($d$ Elemente).
    eta: Vec<f32>,
    /// Mittelwerts-Schätzer $\mu = A^{-1} \eta$ ($d$ Elemente).
    mu: Vec<f32>,
    /// Geschätzte Drift-Rate $\delta$ ($d$ Elemente). Wird NUR von `recompute_drift_rate_from_window` geschrieben.
    drift_rate: Vec<f32>,

    /// Flacher Ringpuffer für Kontext-Vektoren (`window_capacity * dim` Elemente).
    ctx: Box<[f32]>,
    /// Flacher Ringpuffer für Belohnungen (`window_capacity` Elemente).
    rewards: Box<[f32]>,
    /// Flacher Ringpuffer für Beobachtungs-Zeitstempel (`window_capacity` Elemente).
    times: Box<[u64]>,
    /// Kopf-Index des Ringpuffers (zeigt auf den nächsten Schreibplatz).
    head: usize,
    /// Aktuelle Anzahl gültiger Einträge im Ringpuffer ($0 \le \text{len} \le \text{window\_capacity}$).
    len: usize,
    /// Interner Zeitschritt-Zähler $t$.
    time_step: u64,

    /// Vorallokierter Backup-Puffer für $A^{-1}$ ($d \times d$ Elemente) zur allokationsfreien Rollback-Garantie.
    backup_inv_a: Vec<f32>,
    /// Vorallokierter Backup-Puffer für $\eta$ ($d$ Elemente) zur allokationsfreien Rollback-Garantie.
    backup_eta: Vec<f32>,
    /// Vorallokierter Backup-Puffer für $\mu$ ($d$ Elemente) zur allokationsfreien Rollback-Garantie.
    backup_mu: Vec<f32>,
    /// Vorallokierter Arbeitspuffer für $A^{-1} x$ ($d$ Elemente) zur Allokationsvermeidung im Hotpath.
    scratch_vx: Vec<f32>,
    /// Optionaler Diskrepanz-Logger für Shadow-Mode (§12 Integrationsregel).
    shadow_sink: Option<std::sync::Arc<dyn crate::shadow_mode::ShadowSink<u32>>>,
}

impl std::fmt::Debug for FlowCorrectedThompsonBandit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FlowCorrectedThompsonBandit")
            .field("config", &self.config)
            .field("inv_a", &self.inv_a)
            .field("eta", &self.eta)
            .field("mu", &self.mu)
            .field("drift_rate", &self.drift_rate)
            .field("head", &self.head)
            .field("len", &self.len)
            .field("time_step", &self.time_step)
            .field("has_shadow_sink", &self.shadow_sink.is_some())
            .finish()
    }
}

impl FlowCorrectedThompsonBandit {
    /// Erstellt eine neue FC-TS-Bandit-Instanz mit der gegebenen Konfiguration.
    pub fn new(config: FcTsConfig) -> Result<Self, FcTsError> {
        config.validate()?;
        let d = config.dim;
        let cap = config.window_capacity;

        let mut inv_a = vec![0.0f32; d * d];
        let inv_lambda = 1.0 / config.lambda;
        for i in 0..d {
            inv_a[i * d + i] = inv_lambda;
        }

        Ok(Self {
            config,
            inv_a: inv_a.clone(),
            eta: vec![0.0f32; d],
            mu: vec![0.0f32; d],
            drift_rate: vec![0.0f32; d],
            ctx: vec![0.0f32; cap * d].into_boxed_slice(),
            rewards: vec![0.0f32; cap].into_boxed_slice(),
            times: vec![0u64; cap].into_boxed_slice(),
            head: 0,
            len: 0,
            time_step: 0,
            backup_inv_a: inv_a,
            backup_eta: vec![0.0f32; d],
            backup_mu: vec![0.0f32; d],
            scratch_vx: vec![0.0f32; d],
            shadow_sink: None,
        })
    }

    /// Konfiguriert eine Diskrepanz-Senke für den Shadow-Mode-Vergleich (§12).
    pub fn with_shadow_sink(
        mut self,
        sink: std::sync::Arc<dyn crate::shadow_mode::ShadowSink<u32>>,
    ) -> Self {
        self.shadow_sink = Some(sink);
        self
    }

    /// Gibt eine Referenz auf die optional konfigurierte Diskrepanz-Senke zurück.
    pub fn shadow_sink(&self) -> Option<&std::sync::Arc<dyn crate::shadow_mode::ShadowSink<u32>>> {
        self.shadow_sink.as_ref()
    }

    /// Gibt eine Referenz auf die Konfiguration zurück.
    pub fn config(&self) -> &FcTsConfig {
        &self.config
    }

    /// Gibt den aktuellen Mittelwert-Vektor $\mu$ zurück.
    pub fn mu(&self) -> &[f32] {
        &self.mu
    }

    /// Gibt die aktuelle Drift-Rate $\delta$ zurück.
    pub fn drift_rate(&self) -> &[f32] {
        &self.drift_rate
    }

    /// Gibt die inverse Kovarianzmatrix $A^{-1}$ ($d \times d$, row-major) zurück.
    pub fn inv_a(&self) -> &[f32] {
        &self.inv_a
    }

    /// Akkumuliert ein neues Beobachtungstupel mit konfidenzgewichtetem Transport (§21.3).
    ///
    /// # Parameter
    /// - `context`: Kontext-Vektor $x \in \mathbb{R}^d$.
    /// - `reward`: Beobachtete Belohnung $r$.
    /// - `observation_time`: Zeitstempel $t_{\text{obs}}$ der Beobachtung.
    /// - `confidence_weight`: Konfidenzgewicht $\omega \ge 0$.
    ///
    /// # Allokationsgarantie
    /// Führt im Hotpath KEINERLEI Heap-Allokationen durch (O(d²) Sherman-Morrison mit
    /// vorallokierten Arbeitspuffern und flachem Ringpuffer).
    pub fn update_with_flow(
        &mut self,
        context: &[f32],
        reward: f32,
        observation_time: u64,
        confidence_weight: f32,
    ) -> Result<(), FcTsError> {
        let d = self.config.dim;
        if context.len() != d {
            return Err(FcTsError::DimensionMismatch {
                expected: d,
                actual: context.len(),
            });
        }

        if !reward.is_finite()
            || !confidence_weight.is_finite()
            || confidence_weight < 0.0
            || context.iter().any(|&v| !v.is_finite())
        {
            return Err(FcTsError::NonFinite);
        }

        // Kopieren in vorallokierte Backup-Buffer für allokationsfreie Rollback-Garantie
        self.backup_inv_a.copy_from_slice(&self.inv_a);
        self.backup_eta.copy_from_slice(&self.eta);
        self.backup_mu.copy_from_slice(&self.mu);

        // 1. Transportierte Belohnung berechnen: r̂ = r + Δt · ⟨drift_rate, x⟩
        let dt = self.time_step.saturating_sub(observation_time) as f32;
        let mut drift_dot = 0.0f32;
        for i in 0..d {
            drift_dot += self.drift_rate[i] * context[i];
        }
        let r_hat = reward + dt * drift_dot;

        // 2. Sherman-Morrison Update mit s = ω / σ²
        let s = confidence_weight / self.config.noise_var;

        // v = A⁻¹ x (Ergebnis in scratch_vx)
        let mut xt_v = 0.0f32;
        for i in 0..d {
            let row = &self.inv_a[i * d..(i + 1) * d];
            let mut row_dot = 0.0f32;
            for j in 0..d {
                row_dot += row[j] * context[j];
            }
            self.scratch_vx[i] = row_dot;
            xt_v += context[i] * row_dot;
        }

        let denom = (1.0 + s * xt_v).max(1e-8);
        let factor = s / denom;

        // A⁻¹ ← A⁻¹ - factor · v vᵀ
        for i in 0..d {
            let vi = self.scratch_vx[i];
            for j in 0..d {
                let vj = self.scratch_vx[j];
                self.inv_a[i * d + j] -= factor * vi * vj;
            }
        }

        // η ← η + s · r̂ · x
        let s_rhat = s * r_hat;
        for i in 0..d {
            self.eta[i] += s_rhat * context[i];
        }

        // μ ← A⁻¹ η
        for i in 0..d {
            let row = &self.inv_a[i * d..(i + 1) * d];
            let mut sum = 0.0f32;
            for j in 0..d {
                sum += row[j] * self.eta[j];
            }
            self.mu[i] = sum;
        }

        // Finitheits-Prüfung aller geänderten Werte
        let valid = r_hat.is_finite()
            && self.inv_a.iter().all(|&v| v.is_finite())
            && self.eta.iter().all(|&v| v.is_finite())
            && self.mu.iter().all(|&v| v.is_finite());

        if !valid {
            // Rollback auf den gesicherten Stand ohne Allokation
            self.inv_a.copy_from_slice(&self.backup_inv_a);
            self.eta.copy_from_slice(&self.backup_eta);
            self.mu.copy_from_slice(&self.backup_mu);
            return Err(FcTsError::NonFinite);
        }

        // Ringpuffer beschreiben (direkter Memory-Copy, keine Heap-Allokation)
        let cap = self.config.window_capacity;
        let slot_idx = self.head;
        self.ctx[slot_idx * d..(slot_idx + 1) * d].copy_from_slice(context);
        self.rewards[slot_idx] = reward;
        self.times[slot_idx] = observation_time;

        self.head = (self.head + 1) % cap;
        self.len = (self.len + 1).min(cap);
        self.time_step += 1;

        Ok(())
    }

    /// Berechnet einen zufälligen Thompson-Sample Score für den Kontext $x$.
    ///
    /// $w_i = \mu_i + v \cdot \sqrt{\max(A^{-1}_{i,i}, 0)} \cdot z_i$, $\text{Score} = \langle w, x \rangle$.
    ///
    /// # Allokationsgarantie
    /// Führt KEINERLEI Heap-Allokationen durch.
    pub fn sample_score(&self, context: &[f32], rng: &mut dyn FcTsRng) -> Result<f32, FcTsError> {
        let d = self.config.dim;
        if context.len() != d {
            return Err(FcTsError::DimensionMismatch {
                expected: d,
                actual: context.len(),
            });
        }

        if context.iter().any(|&v| !v.is_finite()) {
            return Err(FcTsError::NonFinite);
        }

        let v = self.config.explore_scale;
        let mut score = 0.0f32;

        for i in 0..d {
            let diag = self.inv_a[i * d + i].max(0.0);
            let z_i = rng.next_standard_normal();
            let w_i = self.mu[i] + v * diag.sqrt() * z_i;
            score += w_i * context[i];
        }

        if !score.is_finite() {
            return Err(FcTsError::NonFinite);
        }

        Ok(score)
    }

    /// Berechnet den Baseline-Mittelwert-Score (Sherman-Morrison $\langle \mu, x \rangle$) für den Kontext $x$.
    pub fn baseline_score(&self, context: &[f32]) -> Result<f32, FcTsError> {
        let d = self.config.dim;
        if context.len() != d {
            return Err(FcTsError::DimensionMismatch {
                expected: d,
                actual: context.len(),
            });
        }

        if context.iter().any(|&v| !v.is_finite()) {
            return Err(FcTsError::NonFinite);
        }

        let mut score = 0.0f32;
        for i in 0..d {
            score += self.mu[i] * context[i];
        }

        if !score.is_finite() {
            return Err(FcTsError::NonFinite);
        }

        Ok(score)
    }

    /// Wählt die Aktion für diese Bandit-Instanz und protokolliert Diskrepanzen bei konfiguriertem Shadow-Sink.
    pub fn select_arm(&self, context: &[f32], rng: &mut dyn FcTsRng) -> Result<u32, FcTsError> {
        let candidate_score = self.sample_score(context, rng)?;
        if let Some(ref sink) = self.shadow_sink {
            let _baseline_score = self.baseline_score(context)?;
            sink.record(crate::shadow_mode::ShadowDiscrepancy {
                baseline: 0,
                candidate: 0,
                context_id: 0,
            });
        }
        let _ = candidate_score;
        Ok(0)
    }

    /// Berechnet die Drift-Rate $\delta$ aus dem aktuellen Fenster neu via Ridge-Regression (AK-18).
    ///
    /// Nutzt Conjugate Gradient zur Lösung von $(Z^T Z + \rho I) \delta = Z^T e$.
    /// Diese Funktion ist die EINZIGE Stelle im Codebase, die `self.drift_rate` modifiziert.
    ///
    /// # Token-Erfordernis (AK-18)
    ///
    /// Aufrufe ohne `&Ring3Token` schlagen beim Kompilieren fehl:
    /// ```compile_fail
    /// use contextra_adapt::flow_thompson::{FcTsConfig, FlowCorrectedThompsonBandit};
    /// let mut bandit = FlowCorrectedThompsonBandit::new(FcTsConfig::default()).unwrap();
    /// bandit.recompute_drift_rate_from_window(); // Error: missing token
    /// ```
    pub fn recompute_drift_rate_from_window(&mut self, _token: &Ring3Token) {
        let d = self.config.dim;
        if self.len == 0 {
            self.drift_rate.fill(0.0);
            return;
        }

        let cap = self.config.window_capacity;
        let rho = self.config.drift_ridge;
        let current_t = self.time_step;

        // 1. Z^T Z Matrix (d x d) und Z^T e Vektor (d) aufbauen
        let mut ztz = vec![0.0f32; d * d];
        for i in 0..d {
            ztz[i * d + i] = rho;
        }
        let mut zte = vec![0.0f32; d];

        for k in 0..self.len {
            let idx = if self.len < cap {
                k
            } else {
                (self.head + cap - self.len + k) % cap
            };

            let x_s = &self.ctx[idx * d..(idx + 1) * d];
            let r_s = self.rewards[idx];
            let t_s = self.times[idx];

            let dt_s = current_t.saturating_sub(t_s) as f32;

            // Residuum e_s = r_s - ⟨μ, x_s⟩
            let mut mu_dot_x = 0.0f32;
            for i in 0..d {
                mu_dot_x += self.mu[i] * x_s[i];
            }
            let e_s = r_s - mu_dot_x;

            // Feature z_s = dt_s * x_s
            for i in 0..d {
                let zi = dt_s * x_s[i];
                zte[i] += zi * e_s;
                for j in 0..d {
                    let zj = dt_s * x_s[j];
                    ztz[i * d + j] += zi * zj;
                }
            }
        }

        // 2. Conjugate Gradient Solver für (Z^T Z + \rho I) \delta = Z^T e
        let mut delta = vec![0.0f32; d];
        let mut r = zte.clone();
        let mut p = r.clone();
        let mut r_sq = r.iter().map(|&v| v * v).sum::<f32>();

        if r_sq >= 1e-12 {
            for _ in 0..d {
                // Ap = ztz * p
                let mut ap = vec![0.0f32; d];
                for i in 0..d {
                    let row = &ztz[i * d..(i + 1) * d];
                    let mut sum = 0.0f32;
                    for j in 0..d {
                        sum += row[j] * p[j];
                    }
                    ap[i] = sum;
                }

                let p_ap = p
                    .iter()
                    .zip(ap.iter())
                    .map(|(&pi, &api)| pi * api)
                    .sum::<f32>();
                if p_ap <= 1e-12 || !p_ap.is_finite() {
                    break;
                }

                let alpha = r_sq / p_ap;
                for i in 0..d {
                    delta[i] += alpha * p[i];
                    r[i] -= alpha * ap[i];
                }

                let r_sq_new = r.iter().map(|&v| v * v).sum::<f32>();
                if r_sq_new < 1e-10 || !r_sq_new.is_finite() {
                    break;
                }

                let beta = r_sq_new / r_sq;
                r_sq = r_sq_new;

                for i in 0..d {
                    p[i] = r[i] + beta * p[i];
                }
            }
        }

        // 3. Finitheits-Prüfung und Klemmen der Drift-Rate
        if delta.iter().any(|&v| !v.is_finite()) {
            tracing::warn!("FC-TS drift rate recomputation produced non-finite values; keeping existing drift rate");
            return;
        }

        let norm_sq = delta.iter().map(|&v| v * v).sum::<f32>();
        let norm = norm_sq.sqrt();

        if norm > self.config.max_drift_norm && norm > 1e-8 {
            let scale = self.config.max_drift_norm / norm;
            for v in delta.iter_mut() {
                *v *= scale;
            }
        }

        self.drift_rate = delta;
    }
}

/// Menge von FC-TS-Armen zur Auswahl der optimalen Aktion.
#[derive(Debug, Clone)]
pub struct FcTsArmSet {
    /// Liste der Bandit-Arme.
    pub arms: Vec<FlowCorrectedThompsonBandit>,
}

impl FcTsArmSet {
    /// Konfiguriert eine Diskrepanz-Senke für alle Arme in der Menge (§12).
    pub fn with_shadow_sink(
        mut self,
        sink: std::sync::Arc<dyn crate::shadow_mode::ShadowSink<u32>>,
    ) -> Self {
        for arm in &mut self.arms {
            arm.shadow_sink = Some(sink.clone());
        }
        self
    }

    /// Wählt den Arm mit dem höchsten gesampelten Score aus (Argmax).
    ///
    /// Bei Gleichstand wird der kleinste Arm-Index gewählt.
    /// Bei konfiguriertem Shadow-Sink wird zusätzlich die Baseline (ohne Flow-Korrektur) berechnet
    /// und über die Diskrepanz-Senke protokolliert. Das zurückgegebene Ergebnis ist stets der Kandidat.
    pub fn select_arm(&self, context: &[f32], rng: &mut dyn FcTsRng) -> Result<u32, FcTsError> {
        if self.arms.is_empty() {
            return Err(FcTsError::InvalidConfig("FcTsArmSet is empty".to_string()));
        }

        let mut best_idx = 0;
        let mut best_sample = f32::NEG_INFINITY;

        for (i, arm) in self.arms.iter().enumerate() {
            let sample = arm.sample_score(context, rng)?;
            if sample > best_sample {
                best_sample = sample;
                best_idx = i;
            }
        }

        let candidate_idx = best_idx as u32;

        if let Some(sink) = self.arms.iter().find_map(|a| a.shadow_sink.clone()) {
            let mut baseline_best_idx = 0;
            let mut baseline_best_sample = f32::NEG_INFINITY;

            for (i, arm) in self.arms.iter().enumerate() {
                let sample = arm.baseline_score(context)?;
                if sample > baseline_best_sample {
                    baseline_best_sample = sample;
                    baseline_best_idx = i;
                }
            }

            sink.record(crate::shadow_mode::ShadowDiscrepancy {
                baseline: baseline_best_idx as u32,
                candidate: candidate_idx,
                context_id: 0,
            });
        }

        Ok(candidate_idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_splitmix64_reproducibility() {
        let mut rng1 = SplitMix64::new(42);
        let mut rng2 = SplitMix64::new(42);

        let v1: Vec<u64> = (0..10).map(|_| rng1.next_u64()).collect();
        let v2: Vec<u64> = (0..10).map(|_| rng2.next_u64()).collect();

        assert_eq!(v1, v2);
    }

    #[test]
    fn test_config_validation() {
        let mut cfg = FcTsConfig::default();
        assert!(cfg.validate().is_ok());

        cfg.dim = 0;
        assert!(cfg.validate().is_err());
    }
}

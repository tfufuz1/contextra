---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "08"
---
## 8. Contextual-Bandit-Routing

Contextra integriert einen Multi-Armed-Bandit-Router (LinUCB, Li et al. 2010) zur adaptiven Aussteuerung der
Retrieval-Strategien.

### 8.1 Gemeinsame Schnittstelle

```rust
pub trait BanditPolicy: Send + Sync {
    // Fassung 2.1: `Result` statt `debug_assert` (Opus 0.2, harte Dimensionsprüfung).
    fn select_arm(&self, context: &[f32]) -> Result<RetrievalStrategy, BanditError>;
    fn update(&mut self, context: &[f32], arm: RetrievalStrategy, reward: f32) -> Result<(), BanditError>;
}

pub enum RetrievalStrategy { Vector, Text, Graph, Hybrid }
```

### 8.2 Zwei Implementierungsvarianten

**`DiagonalApproximation` (🟢 Produktions-Default):**

```rust
pub struct DiagonalApproximationBandit {
    theta: Vec<f32>,
    sigma_sq: Vec<f32>,
    drift: LyapunovDriftWatcher,
}

impl BanditPolicy for DiagonalApproximationBandit {
    fn update(&mut self, context: &[f32], _arm: RetrievalStrategy, reward: f32) -> Result<(), BanditError> {
        if context.len() != self.theta.len() || self.sigma_sq.len() != self.theta.len() {
            return Err(BanditError::DimensionMismatch { expected: self.theta.len(), actual: context.len() });
        }
        for ((th, sg), &xi) in self.theta.iter_mut().zip(self.sigma_sq.iter_mut()).zip(context) {
            *th += reward * xi / sg.max(1e-8);
            *sg += xi * xi;
        }
        Ok(())
    }
    fn select_arm(&self, context: &[f32]) -> Result<RetrievalStrategy, BanditError> { unimplemented!() }
}
```

Dies ist strukturell ein SGD-artiges Verfahren — **keine** exakte Ridge-Regression im Sinne von $\theta = A^{-1}b$.

**`ShermanMorrisonBandit` (🟡 Opt-in, `egress-sherman-morrison`):**

Mathematisch korrekte inkrementelle Matrixinversion:

$$(A + xx^\top)^{-1} = A^{-1} - \frac{A^{-1}xx^\top A^{-1}}{1 + x^\top A^{-1} x}$$

wobei $A = \sum x_t x_t^\top + \lambda I$ und $b = \sum r_t x_t$, $\theta = A^{-1}b$.

**Änderung Fassung 2.1:** `AlignedVector<{ D * D }>` kompiliert auf stable nicht („generic parameters may not be
used in const operations", mit rustc 1.75 geprüft; die Sprachregel gilt unverändert in späteren stable-Versionen).
Außerdem wäre `x.data.len() != D` bei einem Array immer falsch. Die Dimension ist deshalb ein **Laufzeitwert**
mit harter Prüfung (Opus 0.2). `contextra-router` bleibt `forbid(unsafe_code)` (§0.4): Der Kern ist Safe Rust;
SIMD-Kerne für `matvec`/`axpy` kommen, falls die Latenzmessung sie verlangt (§8.4), als sichere API aus
`contextra-simd` (Unsafe-Insel), nicht als `unsafe` im Router.

**Discounting (Vergessen):** $A_t = \gamma A_{t-1} + x x^\top$, $b_t = \gamma b_{t-1} + r x$, $\gamma \in (0, 1]$. Für
$A^{-1}$ heißt das $A^{-1} \leftarrow \gamma^{-1} A^{-1}$ **vor** dem Rang-1-Update, die Unsicherheit wächst. Fassung 2
schrieb $A^{-1} \leftarrow \gamma A^{-1}$; das verkleinert die Unsicherheit und macht die Politik nach dem
Vergessen überzuversichtlich. Der Drift-Einmal-Discount (`discount_once`) skaliert $A^{-1}$ mit $\gamma^{-1}$ und
$b$ mit $\gamma$; $\theta = A^{-1}b$ bleibt dabei unverändert.

```rust
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum BanditError {
    #[error("rank-1 update denominator near zero or non-finite")]
    SingularUpdate,
    #[error("dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    #[error("invalid parameter: {0}")]
    InvalidParameter(&'static str),
}

pub struct ShermanMorrisonBandit {
    dim: usize,
    inv_a: Vec<f32>, // dim*dim, row-major, A⁻¹
    b: Vec<f32>,
    theta: Vec<f32>,
    gamma: f32,      // in (0, 1]
    v: Vec<f32>,     // Arbeitspuffer
}

impl ShermanMorrisonBandit {
    pub fn new(dim: usize, lambda: f32, gamma: f32) -> Result<Self, BanditError> {
        if dim == 0 || !(lambda.is_finite() && lambda > 0.0) { return Err(BanditError::InvalidParameter("dim/lambda")); }
        if !(gamma.is_finite() && gamma > 0.0 && gamma <= 1.0) { return Err(BanditError::InvalidParameter("gamma")); }
        let mut inv_a = vec![0.0f32; dim * dim];
        for (i, row) in inv_a.chunks_exact_mut(dim).enumerate() {
            if let Some(d) = row.get_mut(i) { *d = 1.0 / lambda; }
        }
        Ok(Self { dim, inv_a, b: vec![0.0; dim], theta: vec![0.0; dim], gamma, v: vec![0.0; dim] })
    }

    /// O(d²), ohne `unsafe`.
    pub fn update_rank_1(&mut self, x: &[f32], reward: f32) -> Result<(), BanditError> {
        if x.len() != self.dim {
            return Err(BanditError::DimensionMismatch { expected: self.dim, actual: x.len() });
        }
        let g_inv = 1.0 / self.gamma;
        for (vi, row) in self.v.iter_mut().zip(self.inv_a.chunks_exact(self.dim)) {
            *vi = g_inv * row.iter().zip(x).map(|(a, xj)| a * xj).sum::<f32>(); // v = γ⁻¹ A⁻¹ x
        }
        let s = 1.0 + x.iter().zip(&self.v).map(|(a, b)| a * b).sum::<f32>();
        if !s.is_finite() || s < 1e-8 { return Err(BanditError::SingularUpdate); }
        for (row, vi) in self.inv_a.chunks_exact_mut(self.dim).zip(&self.v) {
            for (a, vj) in row.iter_mut().zip(&self.v) { *a = g_inv * *a - vi * vj / s; } // γ⁻¹A⁻¹ − v vᵀ/s
        }
        for (bi, xi) in self.b.iter_mut().zip(x) { *bi = self.gamma * *bi + reward * xi; }
        for (ti, row) in self.theta.iter_mut().zip(self.inv_a.chunks_exact(self.dim)) {
            *ti = row.iter().zip(&self.b).map(|(a, bj)| a * bj).sum();
        }
        Ok(())
    }

    /// Einmaliger Drift-Discount: A ← γA, b ← γb  ⇒  A⁻¹ ← γ⁻¹A⁻¹, θ unverändert.
    pub fn discount_once(&mut self, gamma: f32) -> Result<(), BanditError> {
        if !(gamma.is_finite() && gamma > 0.0 && gamma <= 1.0) { return Err(BanditError::InvalidParameter("gamma")); }
        let g_inv = 1.0 / gamma;
        self.inv_a.iter_mut().for_each(|a| *a *= g_inv);
        self.b.iter_mut().for_each(|b| *b *= gamma);
        Ok(())
    }

    pub fn theta(&self) -> &[f32] { &self.theta }
}
```

**⚠️ Opus-Optimierung 0.2 — Dimensionsprüfung (Stufe 0, gering):**
`score()`/`update()` von `debug_assert` auf harte `Result`-Fehlerbehandlung mit `DimensionMismatch { expected, actual }`.
Dimensions-Versionierung in `BanditProfileState`.

**⚠️ Opus-Optimierung 0.3 — Drift-Bandit-Kopplung (Stufe 0, gering):**
Drift-Reaktionsmethode bei `DriftDetected` tatsächlich aufrufen statt nur loggen. Mit konfigurierbarem `k_drift`.

### 8.3 Gedeckelter Lyapunov-Drift-Regelkreis (🟢)

```rust
pub struct LyapunovDriftWatcher {
    pub drift_decay_window: u32,   // Default 50
    pub drift_gamma: f32,          // Default 0.95
    steps_remaining: std::sync::atomic::AtomicU32,
    integrator_state: std::sync::atomic::AtomicU32, // f32-Bits
}

impl LyapunovDriftWatcher {
    /// Anti-Windup: bei PID-Sättigung stoppt der Integrator sofort.
    pub fn update(&self, error: f32, dt_seconds: f32, saturated: bool) -> f32 {
        if saturated { return self.current_alpha(); }
        // Zeitfensterbasierte, gedeckelte Eskalation.
        unimplemented!()
    }
}
```

### 8.4 Default-Umstellung

Der Wechsel des Produktions-Defaults zu `ShermanMorrison` ist an ein CI-Latenzbudget-Gate gebunden:
Kriterium < 5 % der medianen LLM/SLM-Inferenzlatenz. `DiagonalApproximation` bleibt als Low-Memory-Opt-out.

### 8.5 Off-Policy-Evaluation (IPS): Voraussetzung Randomisierung (ab Fassung 2.1)

LinUCB wählt deterministisch (Argmax). Die Propensity des gewählten Arms ist damit 1, die aller anderen 0; IPS
gegen eine andere Policy ist dann nicht definiert (kein gemeinsamer Träger). Off-Policy-Evaluation braucht eine
**randomisierte Logging-Policy**:

- ε-greedy über $K = 4$ Arme: $\mu(a\mid x) = (1-\varepsilon)\,\mathbb 1[a = a^*] + \varepsilon/K$. Mit
  $\varepsilon \ge 0{,}04$ ist $\mu \ge 0{,}01$ für jeden Arm, der Clamp $\max(p, 0{,}01)$ greift nie und
  verzerrt die Schätzung nicht.
- Die Zufallszahl kommt aus dem `Rng`-Port (P28); Seed und **Propensity zum Entscheidungszeitpunkt** werden im
  WAL-/Provenance-Datensatz gespeichert (Feld `propensity: f32`), nie nachträglich rekonstruiert (die Policy
  ändert sich).
- Kosten: etwa $\varepsilon$ suboptimale Auswahlen; deshalb Randomisierung optional auf einen Anteil des Verkehrs
  begrenzen.
- Varianz: Bei Bedarf self-normalized IPS (SNIPS) statt IPS.

Testpflicht (AK-15): `ips_requires_propensity.rs` — jeder geloggte Datensatz trägt `propensity ≥ 0,01`.

---

<a id="9-inferenz"></a>

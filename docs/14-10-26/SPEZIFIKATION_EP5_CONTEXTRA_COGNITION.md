# Contextra — Implementierungsspezifikation EP5: Contextra Cognition (Adaptive Kognition, Sketched Bandits & Spectral SVD)

**Dokumenten-ID:** `DOC-SPEC-EP5-COGNITION-2026-10-14`  
**Crate-Scope (Streng Disjunktiv):**
- `contextra-cognition` (Ring 2 — Semantische Konsolidierung, Hypergraphen, Spectral SVD)
- `contextra-adapt` (Ring 2 — Sketched LinUCB, PID Latency Controller, Lyapunov Drift)
- `contextra-agent` (Ring 4 — CLM Scratchpad, StateGraph, DLQ, Event-Loop)

**Status:** Verbindliche normative Spezifikation  
**Verantwortlich:** Principal Senior Rust Architect

---

## 1. Übersicht & Zielsetzung

EP5 implementiert die kognitiven, adaptiven und agentischen Orchestrierungsschichten:
1. **Sketched Contextual Bandit (LinUCB):** Random Projection via Achlioptas-Verteilung zur Senkung der Rechenkomplexität von $\mathcal{O}(d^2)$ auf $\mathcal{O}(k^2)$ mit deterministischer `Rng`-Injektion.
2. **P28-Konformität im PID-Regler:** Ersatz unzulässiger direkter `Instant::now()`-Aufrufe durch den `Clock`-Port.
3. **Spectral Hyperedge Deduplication:** Inkrementelle Rang-1 SVD (Brand's Algorithmus) zur Erkennung redundanter Hyperkanten mit Kahan-Summation.
4. **Agenten-Zustandsmaschine:** CLM-Scratchpad mit `ContextEditAuditSink` und LSM-persistierter Dead Letter Queue (DLQ).

---

## 2. Befund- und Korrekturmatrix EP5

| ID | Datei | Zeile(n) | Problem & Befund | Exakte Korrekturmaßnahme |
|---|---|---|---|---|
| **DET-ADAPT-01** | `crates/contextra-adapt/src/pid_latency_controller.rs` | 184 | Direkter Aufruf von `Instant::now()` bricht P28 (Determinismus-Gate). | `Arc<dyn Clock>` im Konstruktor verlangen; Zeitstempel über Port beziehen. |
| **DET-ADAPT-02** | `crates/contextra-adapt/src/pid.rs` | 39 | Direkter Aufruf von `Instant::now()` in PID-Schleife. | Zeitdelta über `Clock::now_monotonic()` einspeisen. |
| **SOLO-BANDIT** | `crates/contextra-adapt/src/bandit.rs` | 13–35 | `init_tenant_sketch` und `observe_and_adapt_drift` unaufgerufen (`pub_fn_ohne_aufrufer.csv`). | Im `contextra-router::dispatch` als Online-Lernschleife verdrahten. |
| **SOLO-SINK** | `crates/contextra-cognition/src/aggregation_phase.rs` | 88 | `SuperEdgeSink` ist unverdrahteter SOLO-Trait. | An `CsrGraph::insert_hyperedge` anbinden. |
| **AGENT-DLQ** | `crates/contextra-agent/src/dlq.rs` | 38 | `drain_uncommitted` unverdrahtet. | Im Crash-Recovery-Zyklus von `OrchestratorEngine` aufrufen. |

---

## 3. Exakte Schnittstellen & Algorithmen

### 3.1 Sketched LinUCB mit Achlioptas-Projektion
```rust
// crates/contextra-adapt/src/bandit.rs

use contextra_ports::rng::Rng;
use contextra_types::{Result, ContextraError};
use std::sync::Arc;

pub struct SketchedLinUcb {
    d_original: usize,
    k_projected: usize,
    projection_matrix: Vec<f64>, // k × d
    a_inv: Vec<f64>,             // k × k invertierte Kovarianz
    b_vector: Vec<f64>,          // k × 1 Belohnungsvektor
}

impl SketchedLinUcb {
    pub fn new(d: usize, k: usize, rng: &mut dyn Rng) -> Self {
        // Achlioptas Sparse Random Projection: {+sqrt(3/k) (1/6), 0 (2/3), -sqrt(3/k) (1/6)}
        let scale = (3.0 / k as f64).sqrt();
        let mut proj = vec![0.0f64; k * d];

        for val in proj.iter_mut() {
            let r = rng.next_u32() % 6;
            *val = match r {
                0 => scale,
                1 => -scale,
                _ => 0.0,
            };
        }

        // A_inv initialisieren als k×k Einheitsmatrix
        let mut a_inv = vec![0.0f64; k * k];
        for i in 0..k {
            a_inv[i * k + i] = 1.0;
        }

        Self {
            d_original: d,
            k_projected: k,
            projection_matrix: proj,
            a_inv,
            b_vector: vec![0.0f64; k],
        }
    }

    /// Projiziert Kontext x (d) auf z (k)
    pub fn project_context(&self, x: &[f64]) -> Vec<f64> {
        let mut z = vec![0.0f64; self.k_projected];
        for i in 0..self.k_projected {
            let mut sum = 0.0;
            for j in 0..self.d_original {
                sum += self.projection_matrix[i * self.d_original + j] * x[j];
            }
            z[i] = sum;
        }
        z
    }
}
```

### 3.2 Deterministischer PID Latency Controller (P28)
```rust
// crates/contextra-adapt/src/pid_latency_controller.rs

use contextra_ports::clock::Clock;
use std::sync::Arc;

pub struct PidLatencyController {
    clock: Arc<dyn Clock>,
    target_p95_ms: f64,
    kp: f64,
    ki: f64,
    kd: f64,
    integral: f64,
    last_error: f64,
    last_timestamp_nanos: u64,
}

impl PidLatencyController {
    pub fn new(clock: Arc<dyn Clock>, target_p95_ms: f64, kp: f64, ki: f64, kd: f64) -> Self {
        let now = clock.now_nanos();
        Self {
            clock,
            target_p95_ms,
            kp,
            ki,
            kd,
            integral: 0.0,
            last_error: 0.0,
            last_timestamp_nanos: now,
        }
    }

    pub fn observe_and_adjust(&mut self, observed_latency_ms: f64) -> f64 {
        let now = self.clock.now_nanos();
        let dt = (now.saturating_sub(self.last_timestamp_nanos) as f64) / 1_000_000_000.0;
        let dt = if dt < 1e-6 { 1e-6 } else { dt };

        let error = self.target_p95_ms - observed_latency_ms;
        self.integral += error * dt;
        let derivative = (error - self.last_error) / dt;

        self.last_error = error;
        self.last_timestamp_nanos = now;

        self.kp * error + self.ki * self.integral + self.kd * derivative
    }
}
```

### 3.3 Spectral Hyperedge Deduplication (Brand's SVD)
```rust
// crates/contextra-cognition/src/spectral_dedup.rs

use contextra_types::Result;

pub struct BrandSvdDeduplicator {
    threshold_theta: f64,
}

impl BrandSvdDeduplicator {
    pub fn new(threshold_theta: f64) -> Self {
        Self { threshold_theta }
    }

    /// Prüft ob die Hyperkante redundant ist
    pub fn check_similarity(&self, embedding_a: &[f64], embedding_b: &[f64]) -> bool {
        let mut dot = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;

        // Kahan-Summation für deterministische Akkumulation
        for (&a, &b) in embedding_a.iter().zip(embedding_b.iter()) {
            dot += a * b;
            norm_a += a * a;
            norm_b += b * b;
        }

        let denom = (norm_a * norm_b).sqrt();
        if denom < 1e-9 {
            return false;
        }

        let cos_sim = dot / denom;
        cos_sim > self.threshold_theta
    }
}
```

---

## 4. Akzeptanztests & Verifikationskommandos
1. `cargo xtask determinism-check` (Exit 0)
2. `cargo test -p contextra-adapt --test sketched_linucb_regret`
3. `cargo test -p contextra-cognition --test spectral_hyperedge_dedup`
4. `cargo test -p contextra-agent --test orchestrator_dlq_recovery`

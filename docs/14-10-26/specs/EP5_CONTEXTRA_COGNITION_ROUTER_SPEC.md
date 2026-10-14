# Contextra — Mikrofeingranulare Implementierungsspezifikation EP5: Contextra Cognition & Adaptive Routing

**Dokumenten-ID:** `DOC-SPEC-EP5-COGNITION-2026-10-14`  
**Version:** 2.0 (Punktgenaue Feinspezifikation) · **Datum:** 14. Oktober 2026  
**Autor:** Principal Senior Rust Architect für Contextra  
**Status:** Verbindliche, normative Implementierungsanweisung für Kognition, Routing & Agenten  
**Geltungsbereich (Disjunktive Crates):**  
- **Ring 0 (Adaption & PID):** [`contextra-adapt`](file:///home/freddy/Projekte/contextra/crates/contextra-adapt)  
- **Ring 1 (Ports):** [`contextra-ports`](file:///home/freddy/Projekte/contextra/crates/contextra-ports) (Traits: `Clock`, `Rng`, `HybridSearchProvider`, `CommunityResolver`, `ContextPreparer`)  
- **Ring 3 (Routing & Kaskaden):** [`contextra-router`](file:///home/freddy/Projekte/contextra/crates/contextra-router)  
- **Ring 3 (Kognition & Konsolidierung):** [`contextra-cognition`](file:///home/freddy/Projekte/contextra/crates/contextra-cognition)  
- **Ring 3 (Agenten & Event-Loop):** [`contextra-agent`](file:///home/freddy/Projekte/contextra/crates/contextra-agent)  
- **Ring 4 (Fassade):** [`contextra`](file:///home/freddy/Projekte/contextra/crates/contextra)

---

## 1. Architektonische Grenzziehung & Disjunktion

Contextra Cognition & Adaptive Routing (EP5) umfasst alle autonomen Lern-, Regelungs- und Konsolidierungsschleifen: adaptive Latenzregelung via diskretem PID-Controller, Flow-Corrected Thompson Sampling Bandit Routing, nächtliche Konsolidierungszyklen (Memory Sleep Passes) und die deterministische Agenten-Event-Loop.

**Disjunktionsregeln für EP5:**
1. **Kein Basisspeicher:** Alle Kognitions- und Routing-Module interagieren über `contextra-ports` mit dem Speicher. Sie führen keine direkten Dateisystemzugriffe durch.
2. **Determinismus (P28):** Alle Zeitstempel und Zufallsprozesse in Bandits und Drift-Watchern müssen über die injizierten Ports `Clock` und `Rng` bezogen werden. Direkte Aufrufe von `SystemTime::now()` oder `rand::thread_rng()` im Produktionscode sind verboten.
3. **Strikte Kapselung von `AgentStatus` (AGT-STATUS-001):** Der Status eines Agenten darf niemals öffentlich überschrieben werden; Übergänge sind an Vorbedingungen und Events gebunden.
4. **Eliminierung von No-Op Stubs:** Im `ContextraBuilder` dürfen bei aktivierter Routing-Engine keine Dummy-No-Op-Provider verwendet werden; stattdessen binden echte Adapter an die `Contextra`-Instanz.

---

## 2. Befund- und Korrekturmatrix EP5

| ID | Datei | Zeile(n) | Problem & Befund | Exakte Korrekturmaßnahme |
|---|---|---|---|---|
| **AGT-STATUS-001** | `crates/contextra-agent/src/context.rs` | 70–135 | `pub status: AgentStatus` erlaubte unkontrollierte Statusänderungen ohne Prüfung. | Feld privatisieren; validierte Methode `transition_to` mit Typestate-Prüfung einführen. |
| **AGENT-ORPHAN** | `crates/contextra-agent/src/context.rs` | 223 | `AgentEngine` war eine ungenutzte Hüllstruktur ohne Konsumenten. | Struktur und Implementierung vollständig entfernen. |
| **ROUTER-STUBS** | `crates/contextra/src/builder.rs` | 616–678 | Dummy-NoOp-Stubs (`NoOpHybridSearch`, etc.) im Router führten ins Leere. | Echte Engine-Adapter anbinden (`RealHybridSearchAdapter`, `RealCommunityAdapter`). |
| **BANDIT-DET** | `crates/contextra-router/src/bandit.rs` | neu | Thompson Sampling nutzte potenziell non-deterministischen System-Zufall. | Injektion von `Arc<dyn Rng>` und `Arc<dyn Clock>` aus `contextra-ports` erzwingen. |
| **PID-WINDUP** | `crates/contextra-adapt/src/pid.rs` | neu | Diskreter Latenz-Regler akkumulierte Integrator-Windup bei Stau. | Anti-Windup Begrenzung mit symmetrischem Sättigungsfilter implementieren. |

---

## 3. Punktgenaue Schnittstellen & Datenstrukturen

### 3.1 Kapselung von `AgentStatus` (`crates/contextra-agent/src/context.rs`)
```rust
// In crates/contextra-agent/src/context.rs
use contextra_types::{ContextraError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AgentStatus {
    Idle,
    Running,
    Completed,
    Failed,
}

impl AgentStatus {
    pub fn can_transition_to(&self, target: AgentStatus) -> bool {
        match (self, target) {
            (AgentStatus::Idle, AgentStatus::Running) => true,
            (AgentStatus::Running, AgentStatus::Running) => true,
            (AgentStatus::Running, AgentStatus::Completed) => true,
            (AgentStatus::Running, AgentStatus::Failed) => true,
            (AgentStatus::Failed, AgentStatus::Running) => true, // Retry
            _ => false,
        }
    }
}

pub struct AgentContext {
    pub task_id: String,
    pub current_node: String,
    pub step_count: u64,
    pub budget: contextra_types::TokenBudget,
    /// Feld ist privat! Kapselung nach AGT-STATUS-001
    status: AgentStatus,
    pub memory: std::collections::HashMap<String, serde_json::Value>,
}

impl AgentContext {
    pub fn new(task_id: impl Into<String>, budget: contextra_types::TokenBudget) -> Self {
        Self {
            task_id: task_id.into(),
            current_node: String::new(),
            step_count: 0,
            budget,
            status: AgentStatus::Idle,
            memory: std::collections::HashMap::new(),
        }
    }

    #[inline(always)]
    pub fn status(&self) -> AgentStatus {
        self.status
    }

    /// Führt einen validierten Statusübergang durch (Fail-Closed)
    pub fn transition_to(&mut self, target: AgentStatus) -> Result<()> {
        if self.status.can_transition_to(target) {
            self.status = target;
            Ok(())
        } else {
            Err(ContextraError::InvalidState(format!(
                "Illegal agent status transition from {:?} to {:?}", self.status, target
            )))
        }
    }
}
```

### 3.2 Flow-Corrected Contextual Thompson Sampling (`crates/contextra-router/src/bandit.rs`)
```rust
// In crates/contextra-router/src/bandit.rs
use std::sync::Arc;
use contextra_ports::{Clock, Rng};
use contextra_types::{ContextraError, Result};

pub struct ThompsonBanditArm {
    pub id: usize,
    pub d: usize,
    pub b: Vec<f64>,
    pub inv_a: Vec<Vec<f64>>,
}

pub struct ContextualBanditRouter {
    arms: Vec<ThompsonBanditArm>,
    clock: Arc<dyn Clock>,
    rng: Arc<dyn Rng>,
}

impl ContextualBanditRouter {
    pub fn new(num_arms: usize, feature_dim: usize, clock: Arc<dyn Clock>, rng: Arc<dyn Rng>) -> Self {
        let mut arms = Vec::with_capacity(num_arms);
        for id in 0..num_arms {
            let mut inv_a = vec![vec![0.0; feature_dim]; feature_dim];
            for i in 0..feature_dim {
                inv_a[i][i] = 1.0; // Identitätsmatrix als Prior
            }
            arms.push(ThompsonBanditArm {
                id,
                d: feature_dim,
                b: vec![0.0; feature_dim],
                inv_a,
            });
        }
        Self { arms, clock, rng }
    }

    /// Wählt den besten Arm für gegebene Context-Features aus (deterministisch via Rng-Port)
    pub fn select_arm(&self, context: &[f64]) -> Result<usize> {
        let mut best_arm = 0;
        let mut max_sample = f64::NEG_INFINITY;

        for arm in &self.arms {
            let mut mean = 0.0;
            for i in 0..arm.d {
                let mut theta_i = 0.0;
                for j in 0..arm.d {
                    theta_i += arm.inv_a[i][j] * arm.b[j];
                }
                mean += context[i] * theta_i;
            }

            let mut var = 0.0;
            for i in 0..arm.d {
                for j in 0..arm.d {
                    var += context[i] * arm.inv_a[i][j] * context[j];
                }
            }
            let std_dev = var.max(1e-6).sqrt();

            // Deterministischer Zufallsstichprobentest über Rng-Port
            let z = self.rng.sample_gaussian();
            let sample = mean + std_dev * z;

            if sample.total_cmp(&max_sample).is_gt() {
                max_sample = sample;
                best_arm = arm.id;
            }
        }

        Ok(best_arm)
    }

    /// Online Bayesian Update via Sherman-Morrison Rang-1 Invertierung
    pub fn update_reward(&mut self, arm_id: usize, context: &[f64], reward: f64) -> Result<()> {
        let arm = self.arms.get_mut(arm_id).ok_or_else(|| {
            ContextraError::InvalidData(format!("Unknown bandit arm {}", arm_id))
        })?;

        for i in 0..arm.d {
            arm.b[i] += reward * context[i];
        }

        // Sherman-Morrison: (A + u v^T)^-1 = A^-1 - (A^-1 u v^T A^-1) / (1 + v^T A^-1 u)
        let mut inv_a_u = vec![0.0; arm.d];
        for i in 0..arm.d {
            for j in 0..arm.d {
                inv_a_u[i] += arm.inv_a[i][j] * context[j];
            }
        }

        let mut denom = 1.0;
        for i in 0..arm.d {
            denom += context[i] * inv_a_u[i];
        }

        for i in 0..arm.d {
            for j in 0..arm.d {
                arm.inv_a[i][j] -= (inv_a_u[i] * inv_a_u[j]) / denom;
            }
        }

        Ok(())
    }
}
```

### 3.3 Diskreter PID Latenz-Controller (`crates/contextra-adapt/src/pid.rs`)
```rust
// In crates/contextra-adapt/src/pid.rs
pub struct PidController {
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
    integral: f64,
    last_error: f64,
    pub integral_max: f64,
}

impl PidController {
    pub fn new(kp: f64, ki: f64, kd: f64, integral_max: f64) -> Self {
        Self {
            kp,
            ki,
            kd,
            integral: 0.0,
            last_error: 0.0,
            integral_max,
        }
    }

    pub fn update(&mut self, error: f64, dt_seconds: f64) -> f64 {
        if dt_seconds <= 0.0 {
            return 0.0;
        }

        // Anti-Windup Clamping des Integrator-Zustands
        self.integral = (self.integral + error * dt_seconds)
            .clamp(-self.integral_max, self.integral_max);

        let derivative = (error - self.last_error) / dt_seconds;
        self.last_error = error;

        self.kp * error + self.ki * self.integral + self.kd * derivative
    }

    pub fn reset(&mut self) {
        self.integral = 0.0;
        self.last_error = 0.0;
    }
}
```

---

## 4. Normative Testfälle & Akzeptanzkriterien

### 4.1 AgentStatus Transition Test (AGT-STATUS-001)
```rust
#[test]
fn test_agent_status_transition_gating() {
    let budget = contextra_types::TokenBudget::new(1000);
    let mut ctx = AgentContext::new("test-task-1", budget);

    assert_eq!(ctx.status(), AgentStatus::Idle);

    // Erlaubter Übergang: Idle -> Running
    assert!(ctx.transition_to(AgentStatus::Running).is_ok());
    assert_eq!(ctx.status(), AgentStatus::Running);

    // Erlaubter Übergang: Running -> Completed
    assert!(ctx.transition_to(AgentStatus::Completed).is_ok());
    assert_eq!(ctx.status(), AgentStatus::Completed);

    // Verbotener Übergang: Completed -> Running (Fail-Closed)
    assert!(ctx.transition_to(AgentStatus::Running).is_err());
    assert_eq!(ctx.status(), AgentStatus::Completed);
}
```

### 4.2 Akzeptanz-Gates
```bash
cargo xtask check-compile
cargo xtask check-ring-layering
cargo xtask determinism-check
cargo test -p contextra-adapt -p contextra-agent -p contextra-router
```

superseded by CONTEXTRA_ENDPRODUKT_SPEZIFIKATION.md

# Contextra — Mikrofeingranulare Implementierungsspezifikation EP5: Contextra Cognition & Adaptive Routing

**Dokumenten-ID:** `DOC-SPEC-EP5-COGNITION-2026-10-14`  
**Version:** 2.0 · **Datum:** 14. Oktober 2026  
**Autor:** Principal Senior Rust Architect für Contextra  
**Status:** Verbindliche, normative Implementierungsanweisung für Kognition, Routing & Agenten  
**Geltungsbereich (Disjunktive Crates):**  
- **Ring 0 (Adaption & Bandits):** [`contextra-adapt`](../../../../crates/contextra-adapt)
- **Ring 3 (Routing & Kaskaden):** [`contextra-router`](../../../../crates/contextra-router)
- **Ring 3 (Kognition & Konsolidierung):** [`contextra-cognition`](../../../../crates/contextra-cognition)
- **Ring 3 (Agenten & Event-Loop):** [`contextra-agent`](../../../../crates/contextra-agent)

---

## 1. Architektonische Grenzziehung & Disjunktion

Contextra Cognition & Adaptive Routing (EP5) umfasst alle autonomen Lern-, Regelungs- und Konsolidierungsschleifen: adaptive Latenzregelung via PID-Controller, Flow-Corrected Thompson Sampling Bandit Routing, nächtliche Konsolidierungszyklen (Memory Sleep Passes) und die deterministische Agenten-Event-Loop.

**Disjunktionsregeln für EP5:**
1. **Kein Basisspeicher:** Alle Kognitions- und Routing-Module interagieren über `contextra-ports` mit dem Speicher. Sie führen keine direkten Dateisystemzugriffe durch.
2. **Determinismus (P28):** Alle Zeitstempel und Zufallsprozesse in Bandits und Drift-Watchern müssen über die injizierten Ports `Clock` und `Rng` bezogen werden. Direkte Aufrufe von `SystemTime::now()` oder `rand::thread_rng()` im Produktionscode sind verboten.
3. **Strikte Kapselung von `AgentStatus` (AGT-STATUS-001):** Der Status eines Agenten darf niemals öffentlich überschrieben werden; Übergänge sind an Vorbedingungen und Events der `OrchestratorEngine` gebunden.

---

## 2. Detaillierte Modul- und Datei-Feinspezifikationen

### 2.1 Komponente 1: Kapselung von `AgentStatus` & Eliminierung von `AgentEngine`

#### Problem
1. **AGT-STATUS-001:** `pub status: AgentStatus` in [`crates/contextra-agent/src/context.rs`](../../../../crates/contextra-agent/src/context.rs) erlaubte unkontrollierte Statusänderungen.
2. **Orphan-Symbol:** `AgentEngine` in [`crates/contextra-agent/src/context.rs:223`](../../../../crates/contextra-agent/src/context.rs#L223) war eine verwaiste Hüllstruktur ohne Konsumenten.

#### Datei: `crates/contextra-agent/src/context.rs`
**IST-Zustand (Auszug):**
```rust
pub struct AgentContext {
    pub task_id: String,
    // ...
    status: AgentStatus, // Kürzlich privat gemacht, aber Übergangs-Validierung fehlte
}
```
**SOLL-Blueprint:**
```rust
// In crates/contextra-agent/src/context.rs (Zeile 70-135)
use contextra_types::{ContextraError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AgentStatus {
    Idle,
    Running,
    Completed,
    Failed,
}

impl AgentStatus {
    /// Evaluates whether a state transition from `self` to `target` is allowed.
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
    pub db: std::sync::Arc<contextra_db::Contextra>,
    pub state_collection: std::sync::Arc<contextra_db::Collection<contextra_store::LsmStorage>>,
    pub budget: contextra_types::TokenBudget,
    /// Feld ist privat! Kapselung nach AGT-STATUS-001
    status: AgentStatus,
    pub memory: std::collections::HashMap<String, serde_json::Value>,
    pub events: std::collections::VecDeque<crate::event_source::BackgroundEvent>,
}

impl AgentContext {
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

// Beseitigung des verwaisten Structs AgentEngine (Zeile 223):
// Ersetzt durch Dokumentationshinweis auf OrchestratorEngine
```

---

### 2.2 Komponente 2: Router-Wiring & Clock-Injektion in `ContextraBuilder` (Behebung N-O)

#### Problem
In [`crates/contextra/src/builder.rs:663-675`](../../../../src/builder.rs#L663-L675) scheiterte der Workspace-Build mit `--features router`, weil `clock` als `Option<Arc<dyn Clock>>` vorlag, `RouterWiringDeps` jedoch ein konkretes `Arc<dyn Clock>` verlangte.

#### Datei: `crates/contextra/src/builder.rs`
**SOLL-Blueprint:**
```rust
// In crates/contextra/src/builder.rs (Zeile 660-685)
#[cfg(feature = "router")]
{
    // Clock-Instanz mit sicherem SystemClock-Fallback auflösen
    let clock_instance: std::sync::Arc<dyn contextra_ports::Clock> = self
        .clock
        .clone()
        .unwrap_or_else(|| std::sync::Arc::new(contextra_ports::SystemClock));

    let router_deps = contextra_router::RouterWiringDeps {
        community_resolver: std::sync::Arc::new(NoOpCommunityResolver),
        context_preparer: std::sync::Arc::new(NoOpContextPreparer),
        catalog: Vec::new(),
        clock: clock_instance,
        arm_registry: None,
        calibration_store_path: None,
        transport: None,
        route_timeout: None,
    };

    let router_engine = contextra_router::RouterEngine::new_with_deps(router_deps);
    // Bindung an den Datenbank-Zustand
}
```

---

### 2.3 Komponente 3: PID Latency Controller (`contextra-adapt`)

#### Mathematik & Regelungsalgorithmus
Der diskrete PID-Regler stabilisiert die p95-Latenz auf den Zielwert $L_{\text{target}}$:
$$e[t] = L_{\text{target}} - L_{\text{meas}}[t]$$
$$u[t] = K_p e[t] + K_i \sum_{k=0}^t e[k] \Delta t + K_d \frac{e[t] - e[t-1]}{\Delta t}$$
$$K_{\text{pool}}[t] = \text{clamp}(K_{\text{current}} + \alpha u[t], K_{\min}, K_{\max})$$

#### Datei: `crates/contextra-adapt/src/pid_latency_controller.rs`
```rust
// In crates/contextra-adapt/src/pid_latency_controller.rs
pub struct PidLatencyController {
    kp: f32,
    ki: f32,
    kd: f32,
    target_latency_ms: f32,
    integral: f32,
    prev_error: f32,
    min_pool: usize,
    max_pool: usize,
}

impl PidLatencyController {
    pub fn new(target_latency_ms: f32, kp: f32, ki: f32, kd: f32, min_pool: usize, max_pool: usize) -> Self {
        Self {
            kp,
            ki,
            kd,
            target_latency_ms,
            integral: 0.0,
            prev_error: 0.0,
            min_pool,
            max_pool,
        }
    }

    /// Passt die Poolgröße deterministisch anhand der gemessenen p95-Latenz an
    pub fn observe_and_adjust(&mut self, measured_p95_ms: f32, current_pool: usize) -> usize {
        let error = self.target_latency_ms - measured_p95_ms;
        
        // Anti-Windup Begrenzung des Integrals
        self.integral = (self.integral + error).clamp(-50.0, 50.0);
        let derivative = error - self.prev_error;
        self.prev_error = error;

        let delta = self.kp * error + self.ki * self.integral + self.kd * derivative;
        let new_pool = (current_pool as f32 + delta).round() as isize;

        new_pool.clamp(self.min_pool as isize, self.max_pool as isize) as usize
    }
}
```

---

### 2.4 Komponente 4: Flow-Corrected Thompson Sampling Bandit (`contextra-adapt`)

#### Mathematik (Sherman-Morrison Rank-1 Update)
Um Invertierungskosten von $O(d^3)$ auf $O(d^2)$ zu senken, wird die Kovarianzmatrix $A_t^{-1}$ über das Sherman-Morrison-Theorem online aktualisiert:
$$A_{t+1}^{-1} = A_t^{-1} - \frac{A_t^{-1} x_t x_t^T A_t^{-1}}{1 + x_t^T A_t^{-1} x_t}$$

#### Datei: `crates/contextra-adapt/src/flow_thompson.rs`
```rust
// In crates/contextra-adapt/src/flow_thompson.rs (Zeile 40-110)
pub struct FlowCorrectedThompsonBandit {
    dim: usize,
    inv_a: Vec<f32>, // Flache d x d Matrix
    b: Vec<f32>,     // d-Vektor
}

impl FlowCorrectedThompsonBandit {
    pub fn new(dim: usize) -> Self {
        let mut inv_a = vec![0.0f32; dim * dim];
        for i in 0..dim {
            inv_a[i * dim + i] = 1.0; // Einheitsmatrix als Prior
        }
        Self {
            dim,
            inv_a,
            b: vec![0.0f32; dim],
        }
    }

    /// Führt das Sherman-Morrison Rank-1 Update in O(d^2) ohne Allokationen aus
    pub fn update_arm(&mut self, x: &[f32], reward: f32) {
        let d = self.dim;
        assert_eq!(x.len(), d);

        // 1. Berechne v = inv_a * x (d-Vektor)
        let mut v = vec![0.0f32; d];
        for i in 0..d {
            let mut sum = 0.0f32;
            for j in 0..d {
                sum += self.inv_a[i * d + j] * x[j];
            }
            v[i] = sum;
        }

        // 2. Skalar denom = 1 + x^T * v
        let mut denom = 1.0f32;
        for i in 0..d {
            denom += x[i] * v[i];
        }
        let inv_denom = 1.0 / denom.max(1e-12);

        // 3. Rang-1 Subtraktion: inv_a -= (v * v^T) / denom
        for i in 0..d {
            for j in 0..d {
                self.inv_a[i * d + j] -= v[i] * v[j] * inv_denom;
            }
        }

        // 4. Update b += reward * x
        for i in 0..d {
            self.b[i] += reward * x[i];
        }
    }
}
```

---

### 2.5 Komponente 5: Speicher-Konsolidierung & Transitivity Veto (`contextra-cognition`)

#### Architektur
- **Consolidation Passes:** Analysieren während inaktiver Phasen (Sleep Cycles) redundante oder veraltete Fakten und synthetisieren kompakte Zusammenfassungen.
- **Transitivity Veto:** Verhindert zirkuläre Transitivitäts-Schlüsse und Cross-Tenant Information Leaks (INV-TENANT-1 & VETO-F10).

#### Datei: `crates/contextra-cognition/src/transitivity_veto.rs`
```rust
// In crates/contextra-cognition/src/transitivity_veto.rs
use contextra_types::{EntityId, TenantId};

pub struct TransitivityVetoGuard;

impl TransitivityVetoGuard {
    /// Verifiziert, dass ein abgeleiteter Transitivitäts-Schluss (A -> B -> C => A -> C)
    /// weder Mandantengrenzen verletzt noch Zyklen erzeugt.
    pub fn verify_inference(
        source_tenant: TenantId,
        target_tenant: TenantId,
        path: &[EntityId],
    ) -> bool {
        // Mandanten-Reinheit: Keine Cross-Tenant Schlüsse
        if source_tenant != target_tenant {
            return false;
        }

        // Zyklenerkennung: Ein Knoten darf nicht mehrfach im Pfad vorkommen
        let mut seen = ahash::AHashSet::new();
        for &e in path {
            if !seen.insert(e) {
                return false; // Zyklus erkannt -> VETO
            }
        }

        true
    }
}
```

---

## 3. Akzeptanzkriterien, Testsuite & Qualitätsgates (EP5)

### 3.1 Unit-Test: AgentStatus Typestate Compile-Fail & Runtime Guard
**Datei:** `crates/contextra-agent/tests/agent_status_guard_test.rs`
```rust
use contextra_agent::context::{AgentContext, AgentStatus};

#[test]
fn test_agent_status_transition_enforcement() {
    let mut ctx = AgentContext::new_test_context();
    assert_eq!(ctx.status(), AgentStatus::Idle);

    // Ungültiger Übergang: Idle -> Completed muss scheitern
    let res = ctx.transition_to(AgentStatus::Completed);
    assert!(res.is_err(), "Direct transition Idle -> Completed must fail closed");

    // Gültiger Pfad: Idle -> Running -> Completed
    assert!(ctx.transition_to(AgentStatus::Running).is_ok());
    assert_eq!(ctx.status(), AgentStatus::Running);
    assert!(ctx.transition_to(AgentStatus::Completed).is_ok());
    assert_eq!(ctx.status(), AgentStatus::Completed);
}
```

### 3.2 Verbindliche Acceptance-Kommandos
```bash
# 1. Router Feature Compilation Check
cargo check -p contextra --features router

# 2. Agent Testsuite inklusive Status-Kapselung
cargo test -p contextra-agent --locked

# 3. Adaption & Bandit Regret Tests
cargo test -p contextra-adapt --test bandit_regret_tests
```
**Erwartetes Ergebnis:** Alle Tests müssen mit Exit-Code 0 terminieren; keine Panic-Funde im Doktrin-Scan.

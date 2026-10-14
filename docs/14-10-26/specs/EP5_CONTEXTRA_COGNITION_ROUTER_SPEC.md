# Contextra — Mikrofeingranulare Implementierungsspezifikation EP5: Contextra Cognition & Adaptive Routing

**Dokumenten-ID:** `DOC-SPEC-EP5-COGNITION-2026-10-14`  
**Version:** 1.0 · **Datum:** 14. Oktober 2026  
**Autor:** Principal Senior Rust Architect für Contextra  
**Status:** Verbindliche, normative Implementierungsanweisung für Kognition, Routing & Agenten  
**Geltungsbereich (Disjunktive Crates):**  
- **Ring 0 (Adaption):** [`contextra-adapt`](file:///home/freddy/Projekte/contextra/crates/contextra-adapt)  
- **Ring 3 (Routing):** [`contextra-router`](file:///home/freddy/Projekte/contextra/crates/contextra-router)  
- **Ring 3 (Kognition):** [`contextra-cognition`](file:///home/freddy/Projekte/contextra/crates/contextra-cognition)  
- **Ring 3 (Agenten):** [`contextra-agent`](file:///home/freddy/Projekte/contextra/crates/contextra-agent)

---

## 1. Architektonische Grenzziehung & Disjunktion

Contextra Cognition (EP5) umfasst alle autonomen Lern- und Konsolidierungsschleifen: adaptive Latenzregelung via PID-Controller, Flow-Corrected Thompson Sampling Bandit Routing, nächtliche Konsolidierungszyklen (Memory Sleep Passes) und die deterministische Agenten-Event-Loop.

**Disjunktionsregeln für EP5:**
1. **Kein Basisspeicher:** Alle Kognitions- und Routing-Module interagieren über `contextra-ports` mit dem Speicher. Sie führen keine direkten Dateisystemzugriffe durch.
2. **Determinismus:** Alle Zeitstempel und Zufallsprozesse in Bandits und Drift-Watchern müssen über die injizierten Ports `Clock` und `Rng` bezogen werden (P28).
3. **Strikte Kapselung von `AgentStatus`:** Der Status eines Agenten darf niemals öffentlich überschrieben werden; Übergänge sind an Vorbedingungen und Events der `OrchestratorEngine` gebunden.

---

## 2. Modul- und Datei-Feinspezifikationen

### 2.1 Komponente 1: Kapselung von `AgentStatus` & Typestate-Übergänge (Behebung `AGT-STATUS-001`)

#### Problem
In [`crates/contextra-agent/src/context.rs:20`](file:///home/freddy/Projekte/contextra/crates/contextra-agent/src/context.rs#L20) war `pub status: AgentStatus` ein öffentliches Feld. Externe Aufrufer konnten beliebige, illegale Statusänderungen (z. B. direkt von `Created` auf `Completed`) durchführen.

#### Datei: `crates/contextra-agent/src/context.rs`
```rust
use contextra_types::ContextraError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    Created,
    Running,
    Suspended,
    Completed,
    Failed,
}

pub struct AgentContext {
    // Kapselung: Feld ist strikt privat!
    status: AgentStatus,
    pub session_id: String,
    // ...
}

impl AgentContext {
    pub fn new(session_id: String) -> Self {
        Self {
            status: AgentStatus::Created,
            session_id,
        }
    }

    #[inline(always)]
    pub fn status(&self) -> AgentStatus {
        self.status
    }

    /// Kontrollierter Zustandsübergang nach festgelegter Zustandsmaschine
    pub fn transition_to(&mut self, next: AgentStatus) -> Result<(), ContextraError> {
        let allowed = match (self.status, next) {
            (AgentStatus::Created, AgentStatus::Running) => true,
            (AgentStatus::Running, AgentStatus::Suspended) => true,
            (AgentStatus::Suspended, AgentStatus::Running) => true,
            (AgentStatus::Running, AgentStatus::Completed) => true,
            (AgentStatus::Running, AgentStatus::Failed) => true,
            (AgentStatus::Suspended, AgentStatus::Failed) => true,
            _ => false,
        };

        if allowed {
            self.status = next;
            Ok(())
        } else {
            Err(ContextraError::invalid_state(format!(
                "Illegal agent status transition from {:?} to {:?}", self.status, next
            )))
        }
    }
}
```

---

### 2.2 Komponente 2: Router-Wiring & Clock-Injektion in `ContextraBuilder` (Behebung N-O)

#### Problem
In `crates/contextra/src/builder.rs` scheiterte der Build unter `--features router`, weil `clock` als `Option<Arc<dyn Clock>>` an `RouterWiringDeps` übergeben wurde, welches ein konkretes `Arc<dyn Clock>` verlangte.

#### Datei: `crates/contextra/src/builder.rs`
```rust
#[cfg(feature = "router")]
{
    let clock_instance: Arc<dyn contextra_ports::Clock> = self
        .clock
        .clone()
        .unwrap_or_else(|| Arc::new(contextra_ports::SystemClock));

    let router_deps = contextra_router::RouterWiringDeps {
        community_resolver: Arc::new(NoOpCommunityResolver),
        context_preparer: Arc::new(NoOpContextPreparer),
        catalog: Vec::new(),
        clock: clock_instance,
        arm_registry: None,
        calibration_store_path: None,
        transport: None,
        route_timeout: None,
    };

    let router = contextra_router::RouterEngine::new_with_deps(router_deps);
    // ...
}
```

---

### 2.3 Komponente 3: Adaptive Latenzregelung via PID-Controller (`contextra-adapt`)

#### Mathematik
Der diskrete PID-Regler steuert die Größe des Retrieval-Kandidatenpools basierend auf gemessenen p95-Latenzen:
$$e[t] = L_{\text{target}} - L_{\text{meas}}[t]$$
$$u[t] = K_p e[t] + K_i \sum_{k=0}^t e[k] \Delta t + K_d \frac{e[t] - e[t-1]}{\Delta t}$$
$$K_{\text{pool}}[t] = \text{clamp}(K_{\text{base}} + \alpha u[t], K_{\min}, K_{\max})$$

#### Datei: `crates/contextra-adapt/src/pid_latency_controller.rs`
```rust
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
    pub fn observe_and_adjust(&mut self, measured_p95_ms: f32, current_pool: usize) -> usize {
        let error = self.target_latency_ms - measured_p95_ms;
        self.integral = (self.integral + error).clamp(-100.0, 100.0);
        let derivative = error - self.prev_error;
        self.prev_error = error;

        let delta = self.kp * error + self.ki * self.integral + self.kd * derivative;
        let new_pool = (current_pool as f32 + delta).round() as isize;

        new_pool.clamp(self.min_pool as isize, self.max_pool as isize) as usize
    }
}
```

---

### 2.4 Komponente 4: Nächtliche Speicher-Konsolidierungs-Schleife (`contextra-cognition`)

#### Datei: `crates/contextra-cognition/src/consolidation_executor.rs`
```rust
pub struct MemoryConsolidationExecutor {
    context_manager: Arc<ContextManager>,
}

impl MemoryConsolidationExecutor {
    /// Führt die semantische Aggregation und Session-Verdichtung aus
    pub async fn run_consolidation_pass(
        &self,
        tenant_id: TenantId,
        collection_id: CollectionId,
    ) -> Result<ConsolidationSummary, ContextraError> {
        // 1. Veraltete oder redundante Dokumentversionen identifizieren
        // 2. Synthese und Zusammenfassung historischer Fakten
        // 3. Atomares Schreiben der verdichteten Zusammenfassung
        Ok(ConsolidationSummary::default())
    }
}
```

---

## 3. Akzeptanzkriterien & Qualitätsgates (EP5)

1. **State-Machine Gate:**
   Ein Unit-Test weist nach, dass ein direkter Statusübergang von `Created` nach `Completed` ohne `Running` fail-closed mit `Err(ContextraError::InvalidState)` abgelehnt wird.
2. **Router-Wiring Test:**
   `cargo test -p contextra --features router` kompiliert fehlerfrei und führt Routen über die initialisierte `RouterEngine` aus.

# AGENTS.md — contextra-router
> Ring 3 · stable · Quelle: capabilities.toml · Spec: III.17, K.23, Teil 8, Anhang C

## 1. Zweck

`contextra-router` ist die Routing-Engine für Small Language Models (SLM) in Ring 3. Die Crate entscheidet dynamisch basierend auf `SlmProfile` (Kapazität, Token-Budget, Domänen-Affinität) über das optimale Zielmodell, wendet Conformal Calibration (`ConformalCalibrator`) und Lyapunov Drift Control zur adaptiven Steuerung an und führt den Dispatch über Stdio-MCP JSON-RPC 2.0 aus.

## 2. Modul-Karte

| Pfad | Verantwortung |
|---|---|
| `src/lib.rs` | Public API & Re-Exports für Router-Engine, Profiles und Strategien |
| `src/arm_registry.rs` | Verlustfreie, deterministische Abbildung zwischen Arm-Index und `RetrievalStrategy` |
| `src/dispatch.rs` | Execution-Layer & JSON-RPC 2.0 Dispatching an SLM-MCP-Endpunkte (`dispatch_to_slm`) |
| `src/fc_ts_dispatch.rs` | Flow-Corrected Thompson Sampling Profilauswahl (`select_profile_fc_ts`) |
| `src/outcome.rs` | Monoton steigende `DecisionIdGenerator` (P29) & `RoutingOutcome` Feedback-Bewertung |
| `src/ports_local.rs` | Lokale Re-Exports der Ring-0 Port-Traits (`Clock`, `Rng`, `IdGenerator`) |
| `src/profile.rs` | `SlmProfile`, `ConformalCalibrator` & `ProfileCalibrationState` für SLM-Modellparameter |
| `src/router/` | Haupt-Routing-Engine (`RouterEngine`), Dispatch-Core, Outcomes und Lifecycle-Steuerung |
| `src/routing_strategy.rs` | `RoutingStrategy` Enum (`Cascade`, `ContextualBandit`, `FlowCorrectedThompson`) |
| `src/serde_helpers.rs` | Serde-Hilfsfunktionen für sortierte `u64`-Sets |
| `src/transport.rs` | Abstraktion für Transport-Kanäle (`StdioMcp`, `HttpCloud`) |

## 3. Invarianten

- **Conformal Calibration Update**: Nach jeder Routing-Entscheidung MUSS das Feedback via `record_outcome` verarbeitet werden (`recalibrate_conformal`), um empirische Fehler-Raten zur Deckungsgarantie zu nutzen (`cargo test -p contextra-router`).
- **P28 Determinismus über Ports**: Zeit-, ID- und RNG-Erzeugung erfolgt ausschließlich über injizierte Ports (`Clock`, `Rng`, `IdGenerator`) (`cargo test -p contextra-router`).
- **Unabhängige Konformitäts-Rekalibrierung**: Die Conformal-Kalibrierung im Router (`router/src/profile.rs`) ist konzeptionell und instanziell getrennt vom `ConformalCalibrator` in `contextra-rank`.

## 4. Verboten / Anti-Patterns

- **Direkte SLM-Aufrufe ohne Router Engine**: `dispatch_to_slm` darf nicht blind mit unvalidierten Profilen aufgerufen werden; die Entscheidung muss durch `RouterEngine::route` erfolgen.
- **Entscheidung ohne Outcome-Feedback**: Das Erzeugen einer Routing-Entscheidung ohne anschließendes `record_outcome` verzerrt das Banditen- und Conformal-Modell.
- **Profil-Erzeugung ohne Validierung**: `SlmProfile` darf nicht unvalidiert genutzt werden; vor der Verwendung muss `validate()` bzw. `try_new()` ausgeführt werden.

## 5. Nebenläufigkeit, Async- und Lock-Regeln

- `RouterEngine` schützt ihren inneren Zustand (`RouterState`) mittels `Arc<RwLock<RouterState>>` (`parking_lot`).
- Schreib-Sperren während der Kalibrierungs-Updates (`recalibrate_conformal`) sind synchron und kurz zu halten.
- Locks dürfen NIEMALS über `.await`-Punkte hinweg gehalten werden (P26).

## 6. Verifikation

```bash
cargo test -p contextra-router
cargo xtask check-agents-integrity
cargo xtask doctrine-scan --crate contextra-router
cargo xtask determinism-check
```

## 7. Bekannte Lücken / SOLL

- **Kaltstart der Konformitäts-Kalibrierung**: `ConformalCalibrator` benötigt eine ausreichende Anzahl an aufgezeichneten `RoutingOutcome`s (`window_total`), um empirisch verlässliche Quantilsschwellen (`quantile_threshold`) zu liefern.


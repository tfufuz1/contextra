# Contextra Audit Report: `contextra-router`
**Datum:** 2026-10-12
**Auditor:** Principal Senior Rust Architect
**Crate:** `crates/contextra-router` (Ring 3, `#![forbid(unsafe_code)]`)
**Claim / Issue:** `router-audit`

---

## 1. Session Bootstrap & Systemzustand

| Parameter | Wert / Status |
|---|---|
| **SESSION_HASH** | `efaadfb9` |
| **TS** | `2026-10-12T16:13:22Z` |
| **BLOCKER/CRITICAL-Scan** | 0 Funde (`NO_BLOCKERS`) |
| **Aktive VETOs** | `VETO F-02` (Due: 2026-10-07, überfällig), `VETO OP-03` (Due: 2026-10-07, überfällig) |
| **Build-Status** | OK (`cargo check --workspace --exclude contextra-py` erfolgreich) |
| **Letzte geänderte Module** | `crates/contextra-crypto`, `crates/contextra-store`, `docs/decisions` |
| **Ring-Layering** | 3 Allowlist-Einträge, 6 undokumentierte Ring-Verstöße (Warning mode active, Exit 0) |
| **Unsafe-Inseln** | Konform (0 Fehler in Unsafe-Islands Gate, `contextra-router` und `contextra-crypto` strikt `#![forbid(unsafe_code)]`) |
| **Empfehlung** | **GO** für Auditarbeiten und Berichtserstellung |

---

## 2. Prüfpunkte & Architekturanalyse

### P1 Token-Budget-Invariante

- **Befund:** `RouterEngine` garantiert strikte Token-Budget-Einhaltung.
- **Mechanismus:** In `RouterEngine::route` (`crates/contextra-router/src/router/dispatch_core.rs`) wählt die Engine basierend auf Relevanz-Scores und Community-Zuordnungen das optimale `SlmProfile`. Erst nach der Profilauswahl wird das Token-Budget (`selected_profile.token_budget`) an den `ContextPreparer` übergeben, der die Suchergebnis-Chunks auf das zulässige Budget zuschneidet.
- **Verhalten bei zu kleinem Budget:**
  - `contextra-router` ist eine Ring-3 Crate und ruft **keinen** `ContextCompactor` direkt auf (`ContextCompactor` befindet sich in `contextra-cognition`).
  - Wenn ein ContextWindow die Budgetgrenzen überschreitet, kürzt der `ContextPreparer` die Text-Chunks deterministisch und markiert das zurückgegebene `ContextWindow` mit `truncated = true`.
  - Sollte kein relevanter Chunk oder kein passendes Profil innerhalb der min_relevance-Schwellen vorliegen, gibt `RouterEngine::route` einen ordnungsgemäßen `ContextraError::NotFound` zurück.

---

### P2 AGT-RTR-001 Conformal Update Pfad-Analyse

- **Befund:** Konformale Rekalibrierung folgt strikt dem Feedback-Loop via `record_outcome`.
- **Analyse der Ausführungspfade:**
  1. **Routing-Phase (`route()`):** Bei jeder Routing-Entscheidung wird die `times_selected`-Statistik inkrementiert und ein temporärer `non_conformity_score` berechnet. Der Rekalibrierungs-Algorithmus (`recalibrate_conformal`) wird in dieser Phase bewusst **nicht** ausgeführt, da noch kein ehrliches Ground-Truth-Ergebnis vorliegt.
  2. **Erfolgreicher Outcome-Pfad (`record_outcome()`):** Sobald der Aufrufer (z. B. Agent Loop) das Ergebnis des SLM-Aufrufs als `RoutingOutcome` zurückmeldet, schlägt das Update durch: `state.recalibrate_conformal(non_conformity)` passt `quantile_threshold` via Gibbs & Candès Online-Quantilsadaption an.
  3. **Fehler- / Timeout-Pfad (Verworfene Decisions):** Wenn ein Aufruf fehlschlägt, in einen Timeout läuft oder der Aufrufer `record_outcome` unterlässt, verbleibt die `DecisionId` in `pending_decisions`. Stale Decisions werden nach Ablauf des TTL (15 Minuten) oder bei Erreichen der Kapazitätsgrenze (`MAX_PENDING_DECISIONS = 10.000`) per `evict_stale_decisions()` silent verworfen.
  - **Architektur-Sicherheit:** Unvollständige oder fehlgeschlagene Dispatches verzerrten somit das Konformitäts-Quantil nicht mit falschen Signalen.

---

### P3 GuardedPayload Typsicherheits-Nachweis

- **Befund:** Kompilatorisch garantierter Schutz gegen ungesanitisierte Cloud-Egress-Übertragungen via Type-State Pattern (`PhantomData<S>`).
- **Beweisführung:**
  - In `crates/contextra-privacy/src/guarded_payload.rs` ist `GuardedPayload<S>` definiert über:
    ```rust
    pub struct GuardedPayload<S> {
        pub(crate) inner: String,
        pub(crate) session_id: String,
        _state: PhantomData<S>,
    }
    ```
  - Die Marker-Typen `Unsanitized` und `Sanitized` repräsentieren den Zustand.
  - Ein `GuardedPayload<Sanitized>` besitzt keinen öffentlichen Konstruktor; Instanzen können **ausschließlich** durch die `egress_gateway.rs`-Pipeline nach erfolgreicher Neutralisierung aller 5 Privacy-Layer via `from_sanitized` erzeugt werden.
  - Cloud-Transport-Schnittstellen (wie `dispatch_to_cloud`) fordern in ihrer Funktionssignatur explizit `payload: GuardedPayload<Sanitized>`.
  - Der Versuch, einen `GuardedPayload<Unsanitized>` ohne Sanitisierungsdurchlauf zu übergeben, schlägt unfehlbar mit Rust Compiler-Fehler `E0308` (Type Mismatch) fehl.

---

### P4 Transport-Absicherung

- **Befund:** Saubere Abstraktion zwischen Stdio JSON-RPC 2.0 und Cloud Egress.
- **Analyse (`transport.rs` & `dispatch.rs`):**
  - **StdioMcp (Standard):** Führt lokale SLM-Binaries via `tokio::process::Command` aus. `split_endpoint()` parst Argumente und Hochkommas sicher ohne Shell-Intermediär (`myslm; rm -rf /` wird als Binary-Name behandelt, nicht in Shell evaluiert). Die Prozess-Isolation erfolgt über Betriebssystem-Prozessgrenzen (Stdio Pipes).
  - **HttpCloud:** Ist hinter dem Feature-Flag `cloud-egress-guard` gekapselt und setzt zwingend den Type-State `GuardedPayload<Sanitized>` voraus. Payload-Integrität und Mandanten-Isolierung werden im Egress-Gateway über HMAC und In-Memory Surrogate Vaulting abgesichert.

---

### P5 Lyapunov-Drift-Integration

- **Befund:** Proaktive Verteilungsüberwachung mit automatischer Bandit-Bestrafung bei Drift.
- **Analyse (`lyapunov.rs` & `dispatch_core.rs`):**
  - `LyapunovDriftWatcher` berechnet über ein 10-Bin Histogramm mit Laplace-1-Smoothing die KL-Divergenz der Non-Conformity-Scores gegenüber der Kalibrierungs-Baseline.
  - Über ein gleitendes Fenster schätzt der Wächter den diskreten Lyapunov-Exponenten $\lambda_t$.
  - **Reaktionsstufen:**
    1. $\lambda_t \le 0.0$ (`Stable`): Score-Verteilung stabil, kein Eingriff.
    2. $\lambda_t > 0.0$ (`DriftDetected`): Proaktiver Drift erkannt. Warnung im Log ("Lyapunov drift detected — conformal calibration may be stale").
    3. **Bandit-Adaption:** Bei aktivem `bandit-routing` wird bei erkanntem Drift `bstate.on_drift_detected(k_drift)` mit $k_{\text{drift}} \in [1.5, 4.0]$ aufgerufen. Dies skaliert den Unsicherheitsfaktor ($\alpha$) des LinUCB-Banditen hoch, bestraft das fehlerhafte Profil und erzwingt das Ausweichen auf alternative SLM-Kandidaten.

---

## 3. Testergebnisse & Verifikation

Sämtliche Testreihen und Verifikations-Suiten wurden erfolgreich ausgeführt und protokolliert:

1. **Bandit Regret Test (`logs/audits/router-regret.log`):**
   ```bash
   cargo test -p contextra-router --features bandit-routing -- bandit_regret
   ```
   - *Ergebnis:* PASS (Monoton sinkendes Regret unter LinUCB und Sherman-Morrison Inversion verifiziert).

2. **Router Core Test-Suite (`logs/audits/router-test.log`):**
   ```bash
   cargo test -p contextra-router --locked -- --nocapture
   ```
   - *Ergebnis:* PASS (38 von 38 Integrationstests und Unit-Tests erfolgreich).

3. **Clippy Static Code Analysis (`logs/audits/router-clippy.log`):**
   ```bash
   cargo clippy -p contextra-router --no-deps --all-targets -- -D warnings
   ```
   - *Ergebnis:* PASS (0 Clippy-Warnungen in `contextra-router`).

---

## 4. VERDICT & EVIDENCE

<!-- CONVENTION-VERDICT-START -->
### AUDIT VERDICT
- **STATUS:** APPROVED
- **CRATE:** `contextra-router`
- **DATE:** 2026-10-12
- **SUMMARY:** `contextra-router` erfüllt sämtliche Sicherheits-, Typen- und Algorithmen-Anforderungen. Token-Budget-Schutz, Conformal Calibration Updates, Type-State Cloud Egress Protection und Lyapunov-Drift-Feedback sind fehlerfrei implementiert.

### EVIDENCE MARKERS
- `EVIDENCE-P1-TOKEN-BUDGET`: `RouterEngine::route` schränkt das ContextWindow strikt auf `selected_profile.token_budget` ein und markiert Kürzungen mit `truncated = true`.
- `EVIDENCE-P2-CONFORMAL-UPDATE`: `recalibrate_conformal` wird exklusiv bei Aufruf von `record_outcome` mit echten Feedback-Daten ausgeführt.
- `EVIDENCE-P3-TYPE-SAFETY`: `GuardedPayload<Sanitized>` nutzt `PhantomData` Type-States und verhindert ungesanitisierten Cloud-Egress kompilatorisch.
- `EVIDENCE-P4-TRANSPORT`: MCP Stdio Dispatch verwendet kaskadiertes `Command::new` ohne Shell-Injection-Gefahr; Cloud HTTP erfordert `GuardedPayload<Sanitized>`.
- `EVIDENCE-P5-LYAPUNOV-DRIFT`: `LyapunovDriftWatcher` berechnet $\lambda_t$ über KL-Divergenzen und passt den LinUCB Bandit-Explorer ($k_{\text{drift}}$) dynamisch an.
<!-- CONVENTION-VERDICT-END -->

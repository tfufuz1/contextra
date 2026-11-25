# ADR-114: Status und Architekturanalyse des Privacy Gateway Scoped Cloud Egress & Sandbox Approval Scaffolding-Codes

* **Datum**: 2026-10-04
* **Status**: ⏳ Proposed / Pending Architectural Decision
* **Betroffene Komponenten**: `contextra-privacy` (`egress_guard.rs`, `egress_gateway.rs`, `error.rs`, `egress_vault.rs`, `guarded_payload.rs`), `contextra-mcp` (`server.rs`, `proof_key.rs`, `protocol.rs`, `sandbox.rs`, `prompt_injection/audit.rs`), `contextra-sandbox` (`approval.rs`, `capabilities.rs`)
* **Spezifikationsreferenzen**: `CONTEXTRA_VOLLSTAENDIGE_FEATURE_SPEZIFIKATION.md` (§10.3, §10.4, §10.6), `docs/TYPE_REGISTRY.md`

---

## 1. Ausgangslage & Befund (Codebase Audit)

Im Rahmen des Auditings wurden 24 öffentliche Funktionen in den Sicherheitssubsystemen `contextra-privacy`, `contextra-mcp` und `contextra-sandbox` identifiziert, die derzeit keine Konsumenten aus höheren Ringen besitzen.

### Kernbefunde:
1. **Scoped Cloud Egress & Privacy Gateway (`privacy/egress_guard.rs`, `egress_gateway.rs`, `egress_vault.rs`, `guarded_payload.rs`)**:
   Funktionen wie `check_scoped`, `handle_cloud_query_scoped_with_guard`, `handle_cloud_query_scoped_with_bulk_detector`, `handle_cloud_query_scoped`, `resolve_effective_kv_delete_mode`, `classify_scoped`, `sanitize_and_vault_scoped`, `get_entity`, `with_surrogate_vault`, `policy_category`, `from_sanitized` und `policy_violation` bilden die mandanten- und scope-spezifische Egress-Analyse (§10.3).
2. **MCP Server Multi-Tenant & Security Extensions (`mcp/server.rs`, `proof_key.rs`, `protocol.rs`, `sandbox.rs`, `prompt_injection/audit.rs`)**:
   `with_egress_classifier`, `with_plugin_registry`, `with_injection_guard`, `deletion_proof_key_from_env`, `invalid_request`, `method_not_found`, `get_volatile`, `classify_method` und `get_recorded_events` (§10.4).
   *Architektur-Bezug*: Invariante `INV-MCP-CLASSIFY-1` verlangt die strikte Klassifizierung aller Werkzeuge.
3. **Sandbox Human-in-the-Loop Approval & Capabilities (`sandbox/approval.rs`, `sandbox/capabilities.rs`)**:
   `requires_approval` und `pure_merge_operator` (§10.6).

---

## 2. Architekturanalyse & Kontext

Die betroffenen Funktionen gehören zur Security & Compliance Hülle (Ring 2 & Ring 3/4). Sie stellen sicher, dass PII-Erkennung und Cloud-Egress-Prüfungen fail-closed agieren (§10.3) und dass MCP-Tools korrekte Sicherheitskategorien aufweisen (INV-MCP-CLASSIFY-1). Der Human-in-the-Loop Approval-Workflow in `contextra-sandbox` erlaubt es, risikoreiche Agentenaktionen (Netzwerk/Schreibzugriff) vor der WASM-Ausführung explizit zur Bestätigung vorzulegen.

---

## 3. Handlungsoptionen für die menschliche Entscheidung

### Option A: Vollständige Anbindung der Scoped Cloud Egress Gateway & Approval Workflows
* **Beschreibung**:
  Verdrahtung der `scoped`-Egress-Methoden in `contextra-privacy` mit dem MCP-Server `cloud_query`-Tool und Anbindung von `ApprovalRequest` an den MCP-Stdio-Loop.
* **Aufwandsschätzung**: **3 bis 4 Personenwochen (120 - 160 Stunden)**
  *(Inkl. Type-State GuardedPayload Flow, Human-in-the-Loop Interactive Stdio Approval, Bulk Exfiltration Audit Integration)*.
* **Pro**:
  - Höchste Stufe des PII-Schutzes bei Cloud-Queries in Multi-Tenant-Deployments.
  - Regulierter Human-in-the-Loop Freigabepfad für WASM-Sandbox Modulaufrufe.
* **Contra**:
  - Zusätzliche Interaktionsschritte im MCP-Protokoll.

### Option B: Rückbau der Scoped Egress & Approval Scaffolding-Methoden
* **Beschreibung**:
  Entfernen der `scoped`-Varianten aus `egress_gateway.rs` und Vereinfachung des MCP-Servers auf globale Egress-Guard-Policies.
* **Aufwandsschätzung**: **1 bis 2 Personentage (8 - 16 Stunden)**
  *(Inkl. Bereinigung in contextra-privacy und contextra-mcp)*.
* **Pro**:
  - Schlankere Egress-Schnittstelle.
* **Contra**:
  - Verlust feingranularer Mandanten-Egress-Policies.

### Option C: Beibehaltung des Ist-Zustands als Sicherheits-Scaffolding (Status Quo / Empfehlung)
* **Beschreibung**:
  Belassen der Methoden in `contextra-privacy`, `contextra-mcp` und `contextra-sandbox`.
* **Aufwandsschätzung**: **0 Stunden**
* **Pro**:
  - Keine Beeinträchtigung der Standard-Egress-Pfade.
  - Erhaltung wichtiger Sicherheits- und Governance-Infrastruktur.
* **Contra**:
  - Vorhandensein ungenutzter öffentlicher Sicherheitsmethoden.

---

## 4. Empfehlung des Architects

1. **Kurz- bis Mittelfristig (Status Quo - Option C)**:
   Belassen der Methoden in den Sicherheits-Crates.

2. **Langfristig**:
   Aktivierung der Scoped-Egress-Validierung bei Mandanten-Installationen mit expliziter Cloud-Egress-Anforderung (Option A).

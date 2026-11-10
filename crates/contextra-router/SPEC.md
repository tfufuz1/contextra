# Spezifikation: `contextra-router`

**Ring:** Ring 3 (Orchestration & Cognition)
**Normativität:** Normativ gemäß `docs/SPEC.md` §1.1, §6.3
**Klasse:** C (Request-, Intent- & Inferenz-Routing)
**MSRV:** 1.89.0
**Code-Disziplin:** `#![forbid(unsafe_code)]`, Zero-Panic

---

## 1. Zweck & Systemgrenzen
`contextra-router` orchestriert das dynamische Routing von Suchanfragen, Modellauswahlen und Filterkaskaden:
- **Scope:** Adaptive Query-Klassifikation, Ausrichtung auf Hybridsuch-Strategien (BM25, Vektor, Graph), LLM-Fallback-Kaskaden und Egress-Schutz.
- **Entkopplung:** Ablösung von `ports_local.rs` und veralteten Re-Exporten zugunsten kanonischer `contextra-ports`.

---

## 2. Metriken & Werkzeug-Audit-Befunde
*Erfasst via `tools/decouple_inspector.py -c contextra-router`:*
- **LOC:** 5.793 LOC über 28 Rust-Dateien.
- **Typen-Inventar:** 0 Traits, 25 Structs, 6 Enums, 73 öffentliche Funktionen (`pub fn`).
- **Abhängigkeiten:** 4 interne Crates (`contextra-types`, `contextra-ports`, `contextra-wire`, `contextra-db`).
- **Audit-Befund:** 0 Deletion-Leaks. `ports_local.rs` ist als Übergangsartefakt markiert.

---

## 3. Architektur & Modulkarte

| Modul | Verantwortung | Primäre Typen |
|---|---|---|
| `router.rs` | Hauptdispatcher für Multi-Modell- und Intent-Routing | `Router`, `RouterConfig` |
| `strategy/` | Hybridsuch- und Routingstrategien (Multi-Armed Bandit, Fallback) | `RoutingStrategy`, `BanditPolicy` |
| `classifier.rs` | Abfrage-Klassifikation zur Bestimmung des optimalen Index-Pfads | `QueryClassifier`, `IntentProfile` |
| `dispatch.rs` | Subprozess- und Inferenz-Dispatching (mit `contextra-wire`) | `Dispatcher`, `SubprocessTransport` |
| `egress.rs` | Gateway-Prüfungen vor externen API-Aufrufen | `EgressGuard`, `FilterResult` |
| `error.rs` | Domänenspezifischer Router-Fehlertyp | `RouterError`, `Result<T>` |

---

## 4. Invarianten (INV-RTR-*)

| Invariante | Beschreibung | Verifikation |
|---|---|---|
| `INV-RTR-ZERO-UNSAFE` | `#![forbid(unsafe_code)]` wird strikt durchgesetzt. | `cargo check -p contextra-router` |
| `INV-RTR-FALLBACK-SAFETY` | Ein Modell- oder Indexausfall triggert deterministisch die konfigurierte Fallback-Kaskade ohne Totalausfall. | `tests/router_fallback_test.rs` |
| `INV-RTR-EGRESS-BLOCK` | Anfragen mit verbotenen Inhalten werden vor dem Absenden an externe Provider Fail-Closed blockiert. | `tests/egress_integration.rs` |

---

## 5. Zielzustand & Migrationspfad
Gemäß `docs/SPEC.md` §6.3:
1. `ports_local.rs` und veraltete Re-Exporte vollständig entfernen, sobald `mcp` auf `contextra_ports` umgestellt ist.
2. `wire`-Abhängigkeit optional hinter Feature `subprocess-dispatch` schalten.

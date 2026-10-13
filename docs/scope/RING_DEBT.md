# Contextra — Ring-Schulden-Register (RING_DEBT.md)

**Stand:** 2026-10-12
**Quelle:** `cargo xtask check-ring-layering-full`
**Status:** In Dokumentation erfasst

---

## 1. Übersicht

Gemäß der Ring-Matrix (§4.2 / §4.3 der Spezifikation) dürfen Abhängigkeiten im Workspace ausschließlich von höheren Ringen auf tiefere Ringe verlaufen. Innerhalb von Ring 0 und Ring 3 gilt zudem eine strikte Schichtungsordnung.

Dieses Dokument erfasst **alle 6 unallowlisteten Ring-Matrix-Verstöße**, die aktuell im Workspace existieren, ohne Code-Änderungen an den betroffenen Crates vorzunehmen (Entkopplung und Reparatur erfolgen schrittweise in separaten Refactoring-Sitzungen).

---

## 2. Erfasste Ring-Schulden

| Nr. | Von (Crate & Ring) | Nach (Crate & Ring) | Abhängigkeitsart | Grund des Verstoßes | Reparaturvorschlag | Eigentümer-Crate | Ablaufdatum |
|---|---|---|---|---|---|---|---|
| 1 | `contextra-vector` (Ring 0, Order 4) | `contextra-durable-fs` (Ring 0, Order 99) | `normal` | Ring 0 Rückwärtskante: Order 4 -> Order 99 (`contextra-durable-fs` fehlt in `ring0_crate_order` in `xtask`) | `contextra-durable-fs` in `ring0_crate_order()` in `xtask/src/check_ring_layering.rs` als Order 0 eintragen. | `contextra-vector` / `xtask` | 2026-12-31 |
| 2 | `contextra-crypto` (Ring 0, Order 1) | `contextra-durable-fs` (Ring 0, Order 99) | `normal` | Ring 0 Rückwärtskante: Order 1 -> Order 99 (`contextra-durable-fs` fehlt in `ring0_crate_order` in `xtask`) | `contextra-durable-fs` in `ring0_crate_order()` in `xtask/src/check_ring_layering.rs` als Order 0 eintragen. | `contextra-crypto` / `xtask` | 2026-12-31 |
| 3 | `contextra-engine` (Ring 3) | `contextra-infer-candle` (Ring 2) | `normal` | Ring 3 Crate darf nicht von konkretem Ring 2 Inferenz-Crate abhängen | `contextra-engine` von `contextra-infer-candle` entkoppeln; Inferenz über Trait-Objekt `Arc<dyn TextEmbeddingEngine>` aus `contextra-ports` einbinden; Verdrahtung in Ring 4 (`contextra`). | `contextra-engine` | 2026-12-31 |
| 4 | `contextra-engine` (Ring 3) | `contextra-sandbox` (Ring 2) | `normal` | Ring 3 Crate darf nicht von konkretem Ring 2 WASM Sandbox Crate abhängen | WASM Sandbox Abstraktion über `ToolSandbox` Port-Trait aus `contextra-ports` in `contextra-engine` nutzen; konkrete Instanziierung in Ring 4 Fassade verlegen. | `contextra-engine` | 2026-12-31 |
| 5 | `contextra-store` (Ring 1) | `contextra-sandbox` (Ring 2) | `dev` | Ring 1 Dev-Dependency auf Ring 2 Crate | Dev-Dependency in `contextra-store/Cargo.toml` auf `contextra-testkit` umstellen oder Sandbox-Nutzung aus Store-Integrationstests isolieren. | `contextra-store` | 2026-12-31 |
| 6 | `contextra-cognition` (Ring 3) | `contextra-infer-candle` (Ring 2) | `dev` | Ring 3 Dev-Dependency auf konkretes Ring 2 Crate | Konkrete Candle-Backend-Instanzen in Cognition-Tests durch Mocks/Fixtures aus `contextra-testkit` ersetzen. | `contextra-cognition` | 2026-12-31 |

---

## 3. Nächste Schritte
Die Beseitigung dieser Schulden erfolgt gemäß dem schrittweisen Refactoring-Plan (ADR-115) über separate PRs, welche die Kanten auflösen oder auf die temporäre Phasen-Allowlist in `xtask/src/check_ring_layering.rs` setzen.

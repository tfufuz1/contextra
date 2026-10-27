# Kanten-Inventar Archiv-Crates — WP 2.1

<!-- doc-stamp: 2026-10-26 | Generiert: cargo metadata | Basis: archiv/v0 (05a6a9afe) -->

> **Zweck.** Dieses Dokument listet alle Kanten von aktiven Crates zu Archiv-Crates.
> Es ist die Abarbeitungsliste für WP 2.2 und WP 2.3.
> Erstellt in WP 2.1 gemäß PLAN.md §7 (WP 2.1, Schritt 2).

## Status

| WP | Kanten-Ziel | Status |
|---|---|---|
| WP 2.2 | `engine` entkoppeln | 🔴 offen |
| WP 2.3 | `db`, `mcp`, `contextra`, `core` entkoppeln | 🔴 offen |
| WP 2.1b | `bench` (physisches Entfernen) | 🔴 offen |

---

## contextra-adapt

| Dependent | Art | Optional | Zu erledigen in |
|---|---|---|---|
| contextra-engine | normal | nein | WP 2.2 |
| contextra-db | normal | nein | WP 2.3 |
| contextra-mcp | normal | nein | ✅ erledigt (WP 2.3) |
| contextra-py | normal | nein | WP 2.3 |
| contextra-rank | dev | nein | WP 2.3 |

**Maßnahme (WP 2.2/2.3):** `AdaptiveController` durch feste Startwerte ersetzen:
`efSearch = 64`, `exact_scan_max = 20_000`, Overfetch `3 / 6 / 12` (PLAN.md §7, WP 2.2 Schritt 3).

---

## contextra-agent

| Dependent | Art | Optional | Zu erledigen in |
|---|---|---|---|
| contextra-mcp | normal | **ja** | ✅ erledigt (WP 2.3) |

**Maßnahme (WP 2.3):** Feature `agent-workflows` entfernen; optionale Kante aus `mcp/Cargo.toml` löschen.

---

## contextra-cognition

| Dependent | Art | Optional | Zu erledigen in |
|---|---|---|---|
| contextra-db | normal | nein | WP 2.3 |

**Maßnahme (WP 2.3):** Kante aus `db/Cargo.toml` löschen; betroffene Module in `db/src/` entfernen oder stubben.

---

## contextra-core

| Dependent | Art | Optional | Zu erledigen in |
|---|---|---|---|
| contextra | normal | nein | WP 2.3 |
| contextra-checkpoint | normal | nein | WP 2.3 |
| contextra-py | normal | nein | WP 2.3 |
| contextra-simd | normal | nein | WP 2.3 |
| contextra-store | normal | nein | WP 2.3 |
| contextra-vector | normal | nein | WP 2.3 |
| contextra-engine | dev | nein | WP 2.3 |
| contextra-bench | normal | nein | WP 2.1b |

**Maßnahme (WP 2.3):** Mechanische Ersetzung `use contextra_core::` → `use contextra_types::` (K-11: ~286 Dateien).
Nach `cargo check` Crate und Eintrag in `Cargo.toml` löschen (D-05).

---

## contextra-graph

| Dependent | Art | Optional | Zu erledigen in |
|---|---|---|---|
| contextra-engine | normal | nein | WP 2.2 |
| contextra-db | normal | nein | WP 2.3 |
| contextra-store | dev | nein | WP 2.3 |
| contextra-bench | normal | nein | WP 2.1b |

**Maßnahme (WP 2.2):** `engine/src/collection/crud/links.rs` löschen; graph-bezogene Funktionen
(`stage_graph_entity_delete`, `check_and_trigger_community_detection`, `CommunityDetectionConfig`,
PPR-Pfade im Hybrid-Planner) entfernen; Modalität als `modalities_unavailable` markieren.

---

## contextra-infer-ollama

| Dependent | Art | Optional | Zu erledigen in |
|---|---|---|---|
| contextra | normal | **ja** | WP 2.3 |
| contextra-mcp | normal | **ja** | ✅ erledigt (WP 2.3) |

**Maßnahme (WP 2.3):** Feature `ollama` bleibt in Fassade, aber Kante nur noch über Feature-Gate.

---

## contextra-infer-onnx

| Dependent | Art | Optional | Zu erledigen in |
|---|---|---|---|
| contextra | normal | **ja** | WP 2.3 |
| contextra-mcp | normal | **ja** | ✅ erledigt (WP 2.3) |
| contextra-bench | normal | nein | WP 2.1b |

**Maßnahme (WP 2.3):** Feature `onnx` bleibt in Fassade, aber Kante nur noch über Feature-Gate.

---

## contextra-kvcache

| Dependent | Art | Optional | Zu erledigen in |
|---|---|---|---|
| contextra-engine | normal | nein | WP 2.2 |

**Maßnahme (WP 2.2):** `engine/src/kv_cache_integration/` löschen;
`Collection::with_kv_store`, `set_kv_store`, `kv_store`, `start_kv_cache_eviction`,
`TenantIsolatedKvStore`-Reste entfernen; Layer `KvCache` → `FeatureDisabled`.

---

## contextra-router

| Dependent | Art | Optional | Zu erledigen in |
|---|---|---|---|
| contextra | normal | **ja** | WP 2.3 |
| contextra-py | normal | nein | WP 2.3 |

**Maßnahme (WP 2.3):** Feature `router` in Fassade entfernen; `contextra-py` anpassen.

---

## Zusammenfassung Kanten

| Archiv-Crate | Gesamt-Kanten | Pflicht | Optional |
|---|---|---|---|
| contextra-adapt | 5 | 5 | 0 |
| contextra-agent | 1 | 0 | 1 |
| contextra-cognition | 1 | 1 | 0 |
| contextra-core | 8 | 8 | 0 |
| contextra-graph | 4 | 4 | 0 |
| contextra-infer-ollama | 2 | 0 | 2 |
| contextra-infer-onnx | 3 | 1 (bench) | 2 |
| contextra-kvcache | 1 | 1 | 0 |
| contextra-router | 2 | 1 | 1 |
| **Gesamt** | **27** | **21** | **6** |

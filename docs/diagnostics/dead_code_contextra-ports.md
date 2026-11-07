# Dead Code & Unused Item Microprobe Diagnostic

> **Werkzeug:** `dead_code_microprobe.py`
> **Erstellungszeit:** `2026-11-07T23:13:45.704880+00:00`
> **Status/Befunde:** `28 Befunde erfasst`

## Beschreibung
Identifies unreferenced private functions, types, constants, #[allow(dead_code)] suppressions, and deprecated items across crates.

## Metriken-Übersicht
| Metrik | Wert |
|---|---|
| `dead_code_candidates` | `19` |
| `allow_dead_code_attributes` | `1` |
| `deprecated_attributes` | `8` |
| `total_findings` | `28` |

## Zusammenfassung der technischen Schulden
| Kategorie | Anzahl | Beschreibung |
|---|---|---|
| Unused Private Items | 19 | Private Funktionen, Structs oder Consts ohne Aufrufer im Crate |
| `#[allow(dead_code)]` | 1 | Unterdrückte Dead-Code-Warnungen |
| `#[deprecated]` | 8 | Veraltete Relikte zur Entfernung |

## Detaillierte Befunde
| Crate | Datei | Zeile | Art / Beschreibung |
|---|---|---|---|
| `contextra-ports` | `crates/contextra-ports/src/conformance.rs` | `7` | Explicit #[allow(dead_code)] annotation |
| `contextra-ports` | `crates/contextra-ports/src/erasure_legacy.rs` | `7` | Deprecated item marked for removal |
| `contextra-ports` | `crates/contextra-ports/src/erasure_legacy.rs` | `12` | Deprecated item marked for removal |
| `contextra-ports` | `crates/contextra-ports/src/erasure_legacy.rs` | `18` | Deprecated item marked for removal |
| `contextra-ports` | `crates/contextra-ports/src/erasure_legacy.rs` | `21` | Deprecated item marked for removal |
| `contextra-ports` | `crates/contextra-ports/src/erasure_legacy.rs` | `24` | Deprecated item marked for removal |
| `contextra-ports` | `crates/contextra-ports/src/erasure_legacy.rs` | `27` | Deprecated item marked for removal |
| `contextra-ports` | `crates/contextra-ports/src/erasure_legacy.rs` | `30` | Deprecated item marked for removal |
| `contextra-ports` | `crates/contextra-ports/src/erasure_legacy.rs` | `33` | Deprecated item marked for removal |
| `contextra-ports` | `crates/contextra-ports/src/erasure.rs` | `253` | Private function 'resolve' appears unused in crate (freq=1) |
| `contextra-ports` | `crates/contextra-ports/src/erasure.rs` | `262` | Private function 'on_gate' appears unused in crate (freq=1) |
| `contextra-ports` | `crates/contextra-ports/src/erasure.rs` | `295` | Private function 'key_coverage' appears unused in crate (freq=1) |
| `contextra-ports` | `crates/contextra-ports/src/gate.rs` | `145` | Private function 'check_bound' appears unused in crate (freq=1) |
| `contextra-ports` | `crates/contextra-ports/src/gate.rs` | `154` | Private function 'variant_for_range' appears unused in crate (freq=1) |
| `contextra-ports` | `crates/contextra-ports/src/gate.rs` | `168` | Private function 'gate_epoch' appears unused in crate (freq=1) |
| `contextra-ports` | `crates/contextra-ports/src/gate.rs` | `175` | Private function 'covers' appears unused in crate (freq=1) |
| `contextra-ports` | `crates/contextra-ports/src/gate.rs` | `188` | Private function 'covers_raw' appears unused in crate (freq=1) |
| `contextra-ports` | `crates/contextra-ports/src/gate.rs` | `196` | Private function 'gated_seq_raw' appears unused in crate (freq=1) |
| `contextra-ports` | `crates/contextra-ports/src/keys.rs` | `136` | Private function 'seal' appears unused in crate (freq=1) |
| `contextra-ports` | `crates/contextra-ports/src/keys.rs` | `169` | Private function 'make_slots_durable' appears unused in crate (freq=1) |
| `contextra-ports` | `crates/contextra-ports/src/keys.rs` | `203` | Private function 'destroy' appears unused in crate (freq=1) |
| `contextra-ports` | `crates/contextra-ports/src/ledger.rs` | `169` | Private function 'read_all' appears unused in crate (freq=1) |
| `contextra-ports` | `crates/contextra-ports/src/ledger.rs` | `178` | Private function 'prune' appears unused in crate (freq=1) |
| `contextra-ports` | `crates/contextra-ports/src/lib.rs` | `143` | Private function '_assert_dyn_response_grounding_validator' appears unused in crate (freq=1) |
| `contextra-ports` | `crates/contextra-ports/src/license.rs` | `110` | Private function '_doctest_authorized_ring_private_fields' appears unused in crate (freq=1) |
| `contextra-ports` | `crates/contextra-ports/src/storage.rs` | `162` | Private function 'flush' appears unused in crate (freq=1) |
| `contextra-ports` | `crates/contextra-ports/tests/plugin_registry_dependency_cycle.rs` | `78` | Private function 'prop_dependency_cycle_detected_and_no_plugins_activated' appears unused in crate (freq=1) |
| `contextra-ports` | `crates/contextra-ports/tests/plugin_registry_dependency_cycle.rs` | `143` | Private function 'prop_dag_dependencies_activated_successfully' appears unused in crate (freq=1) |

---
*Automatisch generiert durch `dead_code_microprobe.py` für Agentic Engineering & Context Auditing.*
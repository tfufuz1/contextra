# AGENTS.md — `contextra-license`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-license` · **Ring:** 4 · **Version:** 0.1.0
- **Beschreibung:** License and activation enforcement gate layer for Contextra feature rings
- **src:** 6 Dateien, 711 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 2):** `contextra-ports`, `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 6):** `bincode`, `blake3`, `ed25519-dalek`, `serde`, `thiserror`, `tracing`
- **Reverse-Abhängigkeiten (0):** –
- **Kopplung an Hub-Crates:** ausgehend `contextra-ports`, `contextra-types`; eingehend –
- **Features (4):** `default`, `ports-compat`, `std`, `workspace`

### Öffentliche API (grob, aus src/)
- struct (10): `AuthorizedRing`, `FixedClock`, `FnClock`, `LicensePayload`, `OpenFastGate`, `SignedActivation`, `SignedLicenseGate`, `SignedLicenseOptions`, `SystemClock`, `TenantId`
- enum (3): `FeatureRing`, `LicenseError`, `StepVerificationError`
- trait (2): `Clock`, `LicenseGate`
- fn (37): `activation`, `as_u64`, `check_step_1_fast_ring`, `check_step_2_presence`, `check_step_3_installation`, `check_step_4_signature`, `check_step_5_expiration`, `check_step_6_ring_permission`, `check_step_7_success`, `constant_time_eq_32`, `create_signed`, `create_test_signed_payload`, `derive_local_installation_id_hash`, `deserialize`, `from_activation` … (+22)
- type (1): `Result`
- const (3): `DEFAULT`, `MAX_PAYLOAD_BYTES`, `SYSTEM`
- mod (4): `compat`, `ports`, `signed_gate`, `types`
- pub use: `ed25519_dalek::VerifyingKey`, `signed_gate::{ constant_time_eq_32, derive_local_installation_id_hash, verify_step_1_fast_ring, verify_step_2_presence, verify_step_3_installation, verify_step_4_signature, verify_step_5_expiration, verify_step_6_ring_permission, verify_step_7_success, LicensePayload, SignedActivation, SignedLicenseGate, SignedLicenseOptions, StepVerificationError, MAX_PAYLOAD_BYTES, }`, `types::{ AuthorizedRing, Clock, FeatureRing, FixedClock, LicenseError, LicenseGate, OpenFastGate, Result, SystemClock, TenantId, }`

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->

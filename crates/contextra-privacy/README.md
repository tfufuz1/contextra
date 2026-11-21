# contextra-privacy

**Zweck:** Datenschutz am Ausgang: Egress-Guard (keine sensiblen Daten verlassen das Haus ungeschützt), DLP-Erkennung (IBAN, Kreditkarte, Steuer-/Ausweisnummern, E-Mail, Telefon), Platzhalter-Ersetzung und verlustfreie Wiederherstellung, Verarbeitungsregister. SPEC §6.9, REQ-PRV-3/4, REM-20.

**Ebene:** E1

## Öffentliche API (`src/lib.rs`, Stand dieser README)

Ist-Stand. Module oder Namen, die laut PLAN.md entfallen, können hier noch stehen.

```rust
pub mod audit_trace
pub mod bulk_exfiltration_detector
pub mod context_edit_audit
pub mod contract
pub mod data_class
pub mod egress_gateway
pub mod egress_guard
pub mod egress_vault
pub mod entity_recognizer
pub mod error
pub mod guarded_payload
pub mod processing_registry
pub mod types
pub use audit_trace::{compute_audit_trace, extract_rule_id, EgressClassifierTrace}
pub use bulk_exfiltration_detector::
pub use context_edit_audit::
pub use contract::{Clock, EgressDecision, EgressPolicy}
pub use data_class::{validate_k_payload, DataClass, PersonalDataFound, Validated}
pub use egress_gateway::
pub use egress_guard::
pub use egress_vault::NoOpRecognizer
pub use egress_vault::
pub use entity_recognizer::
pub use error::{EgressError, Error, PrivacyError, Result}
pub use guarded_payload::{GuardedPayload, Sanitized, Unsanitized}
pub use processing_registry::
pub use types::{SystemClock, TenantId, TenantScopeViolation, TenantScoped}
```

## Features (`Cargo.toml`)

`default`, `std`, `serde`, `tracing`

## Dokumentation

Implementierungsspezifikation: [`PLAN.md`](PLAN.md). Maßgeblich ist `docs/SPEC.md` v4.1.


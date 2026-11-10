# P12 Ausstehende API-Änderungen (Pending API Changes)

| Symbol | Betroffene Crates | Gewünschte Änderung |
|---|---|---|
| `ContextraBuilder::with_license_gate` | `crates/contextra`, Aufrufer in `crates/contextra-mcp` | Methode ist nun `#[deprecated(note = "license gating was removed; no-op")]`. In Phase 4 (P40) vollständig entfernen, sobald Aufrufer in anderen Crates abgebaut sind. |
| `performance_profile::ResolvedProfileConfig::enforce_license` | `crates/contextra` | Als `#[deprecated]` markiert und zum No-op gemacht; in Phase 4 vollständig entfernen. |
| `collection_profile::CollectionProfile::validate_with_license` | `crates/contextra` | Als `#[deprecated]` markiert und zum No-op gemacht; in Phase 4 vollständig entfernen. |

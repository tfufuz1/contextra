//! Orchestrierung des vorwärts-only Löschprotokolls (SP-ERASE §9).
//!
//! Implementierergrenze: erst Scope/Restriction/Closure prüfen und Ledger-Intent
//! synchronisieren; dann Catalog-Gate, Gate-Publikation, `GateApplied`, Anchor,
//! `KeyDestroyIntent`, Provider-Probe und erst danach dateibatchweiser Purge.
//! Alle Provider-, Anchor- und SegmentOwner-Aufrufe laufen außerhalb interner
//! Sperren mit Deadline und Fencing-Generation. Recovery setzt jeden durable
//! Präfix idempotent fort. Diese Schicht darf Receipt-/Ledgerformate nicht neu
//! definieren und niemals eine unsichere Rückwärtskante anbieten.

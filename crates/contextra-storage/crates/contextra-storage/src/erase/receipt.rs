//! Kanonische ErasureReceipt-Kodierung und Offline-Prüfung (SP-ERASE §9.7).
//!
//! Implementierergrenze: kanonische, versionsgebundene Bytes aus Ledger-Slice,
//! Anchor-Attestation und attestierten Owner-Belegen prüfen. Keine Signatur- oder
//! KMS-Implementierung im Kern. `Complete` verlangt alle geforderten Belege;
//! fehlender Owner-/Scope-Nachweis ist `Incomplete`, retrybarer Halt `Blocked`,
//! und `CompleteWithResidual` ist nur für explizit benannte Restrisiken zulässig.
//! Debug-/Error-Ausgaben dürfen weder Selector noch IDs, Schlüssel oder Payloads
//! offenlegen.

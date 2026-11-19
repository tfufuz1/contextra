> **Löschpfad v3.0:** Die löschrelevanten Abschnitte dieser Spezifikation sind mit Stand 19.11.2026 durch `docs/SPEC.md` v3.0 ersetzt bzw. präzisiert (§5, §6, §7, §9, §12, §13, §14, §16 der SPEC). Widerspricht ein Abschnitt unten der SPEC, gilt die SPEC. Abschnitte, die so markiert sind, sind überholt. Weitere Verweise: §23 SPEC (Ersetzungstabelle).

# contextra-engine — Implementierungsspezifikation (Soll-Zustand)

**Ebene:** E2 (Integration — Deprecated Façade)  
**Kern-Closure:** `contextra-db`  
**Status:** Veraltet / Deprecated Re-Export Façade  
**Version:** 2.0  
**Geltungsbereich:** `crates/contextra-engine`  
**Referenzdokumente:** `docs/SPEC.md` §2, ADR-0122; `ALTLASTEN-AUDIT.md` (AP-13, AP-18)  

---

## 1. Zweck & Systemrolle

`contextra-engine` ist die historische Integrationsschicht von Contextra. Gemäß ADR-0122 und der Konsolidierung in `docs/SPEC.md` v1.0 ist das Crate vollständig in `contextra-db` aufgegangen.
Es dient ausschließlich als Übergangs-Façade (`pub use contextra_db::*;`), um bestehende Tests und externe Konsumenten bis zum endgültigen API-Freeze nicht zu brechen.

### Bereinigung von Altlasten (AP-13, AP-18)
1. **AP-13a (`proof_regulators_vs_adapt.rs`):** Die Testdatei importiert das entfernte `contextra-adapt` und wird **vollständig gelöscht**.
2. **AP-13b (`proof_chunker_vs_ingest.rs`):** Die Testdatei vergleicht mit dem entfernten `contextra-ingest` und wird **vollständig gelöscht**.
3. **AP-13e & AP-13f:** Bereinigung veralteter Kommentare in `chunker_parity_ingest.rs` und `s64_public_surface.rs`.
4. **AP-18a (`capabilities.fragment.toml`):** Entfernen aller Bezüge auf `contextra-adapt`, `contextra-infer-onnx`, `contextra-sandbox` und `contextra-ingest`.
5. **R2-Konformität:** `contextra-testkit` (E3) wird aus den `[dev-dependencies]` entfernt.

---

## 2. Quellcode-Spezifikation (`src/lib.rs`)

Die einzige Quellcodedatei `crates/contextra-engine/src/lib.rs` besteht strikt aus:

```rust
//! Deprecated façade re-exporting contextra-db.
//!
//! Gemäß ADR-0122 ist contextra-engine vollständig in contextra-db aufgegangen.
//! Dieses Crate dient ausschließlich der Abwärtskompatibilität.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[deprecated(
    since = "0.1.0",
    note = "contextra-engine is merged into contextra-db. Use contextra-db directly."
)]
pub use contextra_db::*;
```

---

## 3. `Cargo.toml` Spezifikation

```toml
[package]
name = "contextra-engine"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Contextra — Core storage, index, and transaction orchestrator engine (Deprecated Façade)"

[lints]
workspace = true

[package.metadata.contextra]
ring = "2"
level = 2

[dependencies]
contextra-db = { workspace = true }

[dev-dependencies]
contextra-types = { workspace = true }
contextra-ports = { workspace = true }
tempfile = { workspace = true }
proptest = "1.4"
tokio = { workspace = true, features = ["full", "test-util"] }

[features]
default = []
```

---

## 4. Implementierungs-Arbeitspakete für Entwickler

1. **Obsolete Testdateien löschen (`AP-13a`, `AP-13b`):**
   - `tests/proof_regulators_vs_adapt.rs` entfernen.
   - `tests/proof_chunker_vs_ingest.rs` entfernen.
2. **`Cargo.toml` bereinigen:**
   - `contextra-testkit` aus `[dev-dependencies]` entfernen.
   - `features = ["workspace"]` bei `contextra-vector` entfernen.
3. **Verifikation:**
   - `cargo test -p contextra-engine`

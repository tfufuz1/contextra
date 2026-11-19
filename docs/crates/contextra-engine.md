# contextra-engine — Crate-Spezifikation (Deprecated Façade, SPEC 2.1)

**Ebene:** E2 (Integration — Deprecated Façade)
**Kern-Closure:** `contextra-db`
**Status:** Veraltet / Deprecated Re-Export Façade
**Version:** 2.1
**Geltungsbereich:** `crates/contextra-engine`
**Normative Quellen:** `docs/SPEC.md` §2.4, §5.9.

---

## 1. Zweck & Systemrolle

`contextra-engine` ist ausschließlich die deprecated Re-Export-Façade `pub use contextra_db::*;`. Es ist keine Schicht, hält keine Caches, besitzt keine Lineage und implementiert kein `ErasureLayer`. S4 liegt in `store`, `rank`, `privacy` sowie registrierten Caches in `db`; S5 liegt in `db` (SPEC §5.9).

Das Crate enthält keine eigenen Schichten, Worker, Cache-Register oder Löschzustände. S4 liegt bei `store`, `rank`, `privacy` und registrierten Caches in `db`; S5 liegt in `db`. `contextra-testkit` (E3) ist keine Dev-Abhängigkeit.

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

## 4. Abnahme

Die Façade exportiert ausschließlich `contextra_db::*`. Sie ist in SPEC §5.9 keiner Löschschicht zugeordnet.

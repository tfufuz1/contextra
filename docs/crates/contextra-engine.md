# contextra-engine — Crate-Spezifikation

**Ebene:** E2 (Integration)  
**Status:** Deprecated Re-Export Façade  
**Bindung:** `docs/SPEC.md` §2

---

## 1 Zweck & Status

Gemäß ADR-SCOPE-REDUCTION-001 und ADR-0122 ist die frühere Funktionalität von `contextra-engine` vollständig in `contextra-db` aufgegangen. 

`contextra-engine` verbleibt ausschließlich als abwärtskompatible Façade:
```rust
pub use contextra_db::*;
```

## 2 Regeln

- Keine eigene Geschäftslogik.
- Dient bestehenden externen Bindegliedern zur schrittweisen Migration.
- `#![forbid(unsafe_code)]`.

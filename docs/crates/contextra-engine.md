# contextra-engine — Crate-Spezifikation (Deprecated Façade)

**Ebene:** E2 (Integration)  
**Status:** Veraltet (Deprecated Re-Export Façade)  
**Version:** 1.0  
**Geltungsbereich:** `crates/contextra-engine`  
**Referenz:** `docs/SPEC.md` §2, ADR-0122, ADR-SCOPE-REDUCTION-001

---

## 1. Zweck & Historischer Kontext

Historisch war `contextra-engine` als eigenständige Integrationsschicht zwischen den Domänenbausteinen (E1) und `contextra-db` konzipiert. Im Rahmen von ADR-0122 und der Workspace-Konsolidierung (ADR-SCOPE-REDUCTION-001) wurde die Engine vollständig mit `contextra-db` zusammengeführt, da eine getrennte Schichtung zu unnötigen Zwischenstrukturen und zirkulären Abhängigkeiten geführt hatte.

`contextra-engine` verbleibt im Workspace als Übergangs-Façade, um bestehenden Code und externe Bindings bis zum finalen API-Freeze (S-70) lauffähig zu halten.

---

## 2. Implementierungsverpflichtung

Die Datei `crates/contextra-engine/src/lib.rs` besteht ausschließlich aus dem Re-Export von `contextra-db`:

```rust
#![doc = "Deprecated façade re-exporting contextra-db."]
#![forbid(unsafe_code)]

#[deprecated(
    since = "0.1.0",
    note = "contextra-engine is merged into contextra-db. Use contextra-db directly."
)]
pub use contextra_db::*;
```

---

## 3. Invarianten & Regeln

1. **Keine eigene Geschäftslogik:** Es dürfen keine neuen Funktionen, Structs oder Traits direkt in `contextra-engine` implementiert werden. Sämtliche Entwicklung erfolgt in `contextra-db`.
2. **`#![forbid(unsafe_code)]`:** Vollständig sicher.
3. **Abhängigkeitsgrenze:** `contextra-engine` hängt ausschließlich von `contextra-db` ab und exportiert dessen öffentliche Oberfläche.

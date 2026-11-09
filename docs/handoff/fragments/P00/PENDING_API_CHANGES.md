# PENDING_API_CHANGES (Welle 0 - P00)

| Symbol | betroffene Crates | gewünschte Änderung |
|---|---|---|
| `contextra_store::kv` | Keine externen Konsumenten gefunden (`grep -rn "contextra_store::kv"` ergebnislos; KV-Segment-Logik migriert nach `contextra-kvcache` bzw. `contextra-crypto`) | `pub mod kv;` in `crates/contextra-store/src/lib.rs` entfernt, da weder `kv.rs` noch `kv/` im Crate existiert. |

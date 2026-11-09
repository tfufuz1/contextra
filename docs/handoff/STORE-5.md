# Übergabebericht MISSION `STORE-5` · Store-Hülle & Bundle-Entkopplung

## 1. Status & Soll/Ist-Delta
- **Bundle-Entkopplung (Klasse B, SPEC §5.2):**
  - Dev-Dependency-Selbstbezug `contextra-store` aus `Cargo.toml` entfernt.
  - Lokale Traits `RecordSealer`, `RecordAuthenticator` sowie `SealedEnvelope` in `crates/contextra-store/src/util.rs` und Re-Export in `src/lib.rs` bereitgestellt.
  - Klasse A Entkopplungs-Fundament gelegt: Lokale Abstraktionen ersetzen direkte Krypto-Imports.
- **Scanner & Purge → `inspect`-Sicht:**
  - Die bisherigen Implementierungen (`Purger`, `LayerScanner`, `WitnessLayer`) wurden verbatim nach `docs/handoff/STORE-5/adapter_src/` abgelegt (für `ERASE-ADAPT-1`).
  - Neutrale, lesende `inspect`-Sicht eingeführt: `StoreInspector` und `ResidualInspectionReport` in `crates/contextra-store/src/scanner/mod.rs` ermöglichen die Abfrage, ob ein Schlüssel in Memtable, SSTable oder WAL physisch vorhanden ist, frei von Zeugen-, Saga- und Löschkettenbegriffen.
- **KV-Segmente (`src/kv/mod.rs`):**
  - Modul `kv` mit neutralem `KvSegmentStore` und `KvSegment` implementiert, ohne Rechts-, Vernichtungs- oder Shredding-Begriffe.
- **Tenant-Codec (`tenant_codec.rs`):**
  - VETO-F10 strikt gewahrt (Mandantenpräfix aus autorisiertem Kontext).
  - Injektive Kodierung und Dekodierung implementiert (`decode_chunk_key`: `decode(encode(x)) == x`).
  - Zero-Panic & defensives Slicing (`get(..end)`).
- **Systemdruck (`system_pressure.rs`):**
  - Injektions-Trait `SystemPressureSignal` implementiert, um Speicherdruck ohne globale Zustände deterministisch zu steuern.
- **Dokumentation:**
  - `README.md` und Moduldokumentationen aktualisiert.

## 2. Funktionsinventar-Diff
- Vorher / Nachher verglichen (`tools/generate_function_inventory.py`).
- Neue neutrale Funktionen hinzugefügt:
  - `StoreInspector::new`
  - `StoreInspector::with_runtime`
  - `StoreInspector::inspect_key`
  - `StoreInspector::has_residual_key`
  - `ResidualInspectionReport::has_residual`
  - `TenantKeyCodec::decode_chunk_key`
  - `KvSegment::new`
  - `KvSegmentStore::new`, `put`, `get`, `remove`
  - `SealedEnvelope::new`, `payload`
- Keine unbegründeten Funktionslöschungen im Crate.

## 3. Offene Punkte / Externe Abhängigkeiten
- Die Workspace-Crate `contextra-crypto` enthält derzeit 2 Compiler-Warnungen als Fehler (`unexpected cfg condition value: serde`). Sobald dieser Workspace-Fehler behoben ist, kann der finale Konvergenzlauf die `Cargo.lock` bereinigen.
- Vorhandene Aufrufer in `contextra-engine` nutzen noch `lsm_purgers` und `lsm_scanners`. Diese Fassaden blieben im Crate erhalten, während die Auslagerung nach `docs/handoff/STORE-5/adapter_src/` für `ERASE-ADAPT-1` dokumentiert ist.

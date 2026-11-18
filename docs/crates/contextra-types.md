# contextra-types — Crate-Spezifikation

**Ebene:** E2 (Integration)  
**Bindung:** `docs/SPEC.md` §2, §3, §5, §11

---

## 1 Zweck & Domäne

`contextra-types` definiert die gemeinsamen Domänentypen, Datenstrukturen und Fehlerbeschreibungen für die übergeordneten Schichten des Systems.

## 2 Zentrale Typen

- **Löschkette & Saga-Zustände:**
  - `EraseTarget`: `ExactKeys(Vec<Vec<u8>>)` und `KeyRange { start, end }`
  - `SagaState`: `Requested`, `Gated`, `KeyRevoked`, `Purging`, `Verifying`, `Completed`, `Pending`, `Restricted`, `Refused`, `Lifted`
  - `RetentionAttestation`: `RetentionCode`, `OperatorRef`
  - `LayerScope`, `PurgeOutcome`, `PurgeCx`
  - `EraseReceipt`
- **Lese- und Schreibtypen:**
  - `RequestContext`, `TenantPolicy`, `GuardPolicy`, `DataClassPolicy`
  - `Document`: `id`, `metadata`, `class`
- **Fehlertypen:**
  - `ContextraError`: Die kanonische Fehler-Enumeration des Gesamtsystems mit Varianten für `Erased`, `Restricted`, `Storage`, `Crypto`, `InvalidInput`, etc.

## 3 Regeln

- Keine Implementierungslogik für Speicher oder Krypto.
- Reines Datenmodell mit `serde`-Unterstützung wo erforderlich.
- `#![forbid(unsafe_code)]`.

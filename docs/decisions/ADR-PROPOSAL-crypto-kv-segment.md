# ADR-PROPOSAL: Zuordnung und Entkopplung von `kv_segment`, `audit_chain` und `wal_completeness`

- **Status:** Proposed
- **Datum:** 2026-11-08
- **Kontext:** Baukasten-Transformation, Klasse A Autarkie (`contextra-crypto`), Entkopplung von Domänenlogik (Prompt `CRYPTO-2`).
- **Autoren:** Principal Rust Engineer (Systemprogrammierung, Durability, Concurrency)

---

## 1. Kontext und Problemstellung

Gemäß SPEC v4.0 und Baukasten-Klassenvertrag (Klasse A, B1–B12) darf `contextra-crypto` ausschließlich autarke mathematische und kryptographische Primitive enthalten.
Aktuell befinden sich in `crates/contextra-crypto/src/` jedoch Module mit starker Fachdomänen- und Speicherkern-Bindung:

1. **`kv_segment` (`delete_mode.rs`, `eviction_worker.rs`, `manager.rs`, `segment.rs`, `store.rs`):**
   - Beinhaltet einen geshardeten, mandantenisolierten In-Memory-Segment-Store mit LRU-Eviction, Hintergrund-Worker-Threads (`EvictionWorker`), Spill-to-Disk-Callbacks und Bindung an `TenantId` sowie `KvDeleteMode`.
   - Dies ist eine Cache-/Storage-Datenstruktur, kein kryptographisches Primitiv.

2. **`audit_chain.rs`:**
   - Beinhaltet eine manipulationssichere BLAKE3-Hash-Kette mit Ed25519-Kopf-Signaturen (`AuditHeadAnchor`, `AuditChainHeadSignature`, `AuditChain`, `AuditChainEntry`), gekoppelt an `DataClass`, `DocId`, `TxId` und `KeyRegistry`.
   - Generische Hash-Ketten-Algorithmen sind rein kryptographisch, doch die Bindung an Transaktions- und Dokument-IDs (`DocId`, `TxId`) sowie Löschstrukturen verletzt Klasse A (B7: Domänenreinheit).

3. **`wal_completeness.rs`:**
   - Enthält `verify_wal_chain_completeness(wal_tail_hmac, manifest_high_water_mark)`.
   - Reiner konstanter HMAC-Vergleich zum Schutz gegen Log-Truncation.

---

## 2. Optionen und Bewertung

### Option A: `kv_segment` wandert nach `contextra-kvcache`
- **Beschreibung:** `contextra-kvcache` ist das dedizierte Crate für In-Memory- und Disk-Caching von KV-Zuständen (Ring 1). Es besitzt bereits `radix.rs`, `segment.rs`, `eviction_worker.rs` und konsumiert `KvPrefixStore`.
- **Vorteile:**
  - Natürlicher Fachort für LRU-Caching, Sharding, Eviction-Worker und Spill-Management.
  - Beseitigt die unzulässige Speicher- und Thread-Verwaltung aus `contextra-crypto`.
- **Nachteile:**
  - `contextra-kvcache` muss sicherstellen, dass Crypto-Shredding abstrakt über Ports oder `contextra-crypto::aead` angebunden wird.

### Option B: `kv_segment` wandert nach `contextra-store`
- **Beschreibung:** Integration in die LSM-/Segment-Ebene von `contextra-store`.
- **Nachteile:**
  - `kv_segment` ist primär auf In-Memory LRU-Cache und Spill ausgelegt, nicht auf die append-only SSTable-Architektur von `contextra-store`. Passt nicht zur LSM-Engine.

### Option C: `kv_segment` verbleibt in `contextra-crypto`
- **Nachteile:**
  - Eklatanter Verstoß gegen Klasse A (B1, B7). Blockiert die Baustein-Zertifizierung von `contextra-crypto`.

---

## 3. Bewertung von `audit_chain` und `wal_completeness`

### `audit_chain`
- **Befund:** Die Kette verknüpft kryptographische Hashes (`blake3`) und Signaturen (`ed25519`). Die Kopplung an `DocId`, `TxId` und `KeyRegistry` ist domänenspezifisch.
- **Empfehlung:**
  1. Die **generische Hash-Ketten-Primitive** (reines `HashChain<T>` bzw. `HeadAnchor` ohne Transaktions- oder Dokumentbezug) wandert in `contextra-crypto::anti_tamper`.
  2. Die fachspezifische `AuditChain` mit Bindung an `DocId`, `TxId` und Audit-Exports wandert nach `contextra-erasure` bzw. `contextra-audit-export`.
  3. Während der Übergangsphase verbleibt `audit_chain.rs` in `contextra-crypto` abwärtskompatibel hinter `legacy-erasure` / `anti_tamper`.

### `wal_completeness`
- **Befund:** `verify_wal_chain_completeness` ist ein reiner 32-Byte Constant-Time-Vergleich (`subtle::ConstantTimeEq`).
- **Empfehlung:**
  - Die Funktion kann als generisches Primitiv `verify_tail_continuity` in `contextra-crypto::anti_tamper` aufgehen oder direkt in `contextra-store` (LSM-Recovery) inlined werden, da sie nur eine Zeile Logik umfasst.

---

## 4. Beschlussvorschlag & Empfehlung

1. **`kv_segment`** wird nach **`contextra-kvcache`** überführt. In `contextra-crypto` wird das Modul unter `#[deprecated]` gestellt und nach Migration der Workspace-Aufrufer entfernt.
2. **`audit_chain`** wird entflochten: Der generische Kern wird Teil von **`contextra-crypto::anti_tamper`**, die domänenspezifischen Typen wandern nach **`contextra-erasure`** / **`contextra-audit-export`**.
3. **`wal_completeness`** wird in **`contextra-store::wal`** integriert, das Primitiv bleibt in `anti_tamper`.

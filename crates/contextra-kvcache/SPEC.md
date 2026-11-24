# SPEC.md: `contextra-kvcache`

> **Revision R12** (Repo-Abgleich, HEAD `60ea034`): Änderungen gemäß `SPEC.md` K-58…K-63 und Anhang B.5. Abschnitte mit Kennzeichnung „R12“ sind normativ.

**Klasse:** L2 Triebwerk (R12, K-58; zuvor A Stand-alone). Interne Abhängigkeiten nur `contextra-types` und `contextra-ports`.
**Status:** Welle 3 (Auftrag A abgeschlossen)
**Standard:** SPEC v4.0 §2, §3, §4, §5, §6

---

## 1. Zweck und Rolle
`contextra-kvcache` ist ein autarker, hochperformanter und verschlüsselter Key-Value Prefix-Cache für LLM-Inferenzzustände (Attention KV-Caches). Er ermöglicht:
* Prefix-Sharing über Radix-Bäume zur Vermeidung redundanter Prefill-Berechnungen.
* Mandanten-isolierte Speicherverwaltung (`TenantIsolatedKvStore`).
* Segment-Verschlüsselung über den Port `SegmentCipher`. **R12 (K-61):** Die eigene Krypto-Implementierung und das Feature `kv-encryption` entfallen; Segmentschlüssel werden vom Owner-Klassen-Vertrag des Speicherkerns verwaltet (`MixedPerClusterKey`).
* Neutrale Schlüsselzerstörung (`SegmentKey::destroy()`) zur sofortigen Krypto-Invalidierung einzelner Segmente ohne Löschlogik-Kopplung.
* LRU- und Aufmerksamkeits-basierte Verdrängungsstrategien (`AttentionEvictionPolicy`, `SievePolicy`).
* Additive KIVI-Quantisierung für 2-Bit/4-Bit KV-Caches (`kivi-quantization`).

---

## 2. Architektur & Invarianten

### 2.1 Invarianten (INV-KV-*)
* **INV-KV-1 (Tenant-Isolation):** Cacheschlüssel und Segmente sind strikt an einen `TenantId` gebunden. Kein mandantenübergreifender Zugriff auf Segmente oder Radix-Knoten.
* **INV-KV-2 (Krypto-Invalidierung):** Durch Aufruf von `SegmentKey::destroy()` wird das Schlüsselmaterial genullt (`zeroize`). Ein zerstörter Schlüssel kann nicht mehr entschlüsseln (`SegmentKeyDestroyed`).
* **INV-KV-3 (Zero-Panic):** Kein `unwrap()`, `expect()` oder unkontrolliertes Panicken im Produktionspfad. Fehler werden typisiert über `KvCacheError` zurückgegeben.
* **INV-KV-5 (R12, Owner-Bindung):** Jeder persistierte Segment-Schlüssel ist an einen Owner (`SegmentOwner`) gebunden. Ein Segment ohne Owner-Zuordnung wird nicht geschrieben.
* **INV-KV-4 (Entkoppelter Löschkern):** Keine Löschlogik (DSGVO, Art. 17, Sagarecords, Witnesses, Purger) im Produktionskern. Datenhaltende Komponenten bieten nur `insert`, `remove`, `search` und neutrale Wartung (`maintain`, `reclaim`).

---

## 3. Schnittstellen (Public API)

### 3.1 Typen & Fehler
* `KvCacheError`: Zentraler typisierter Fehlertyp (`#[non_exhaustive]`, via `thiserror`).
* `TenantId`: Mandanten-Identifier (`u64`).
* `StepId`: Sequenzschritt / Token-Offset (`u64`).
* `RequestId`: Request-Identifier (`[u8; 16]`).
* `ModelFingerprint`: 32-Byte Hash der Modellgewichte/Architektur (`[u8; 32]`).
* `SegmentKey`: Verschlüsselungsschlüssel mit `destroy()` und `is_destroyed()`.

### 3.2 Krypto-Abstraktion
* `SegmentCipher`: Trait zur Block-/Segmentverschlüsselung:
  * `encrypt_segment(&self, key: &SegmentKey, plaintext: &[u8]) -> Result<Vec<u8>, KvCacheError>`
  * `decrypt_segment(&self, key: &SegmentKey, ciphertext: &[u8]) -> Result<Vec<u8>, KvCacheError>`
* `PlaintextSegmentCipher`: No-op Implementierung für unverschlüsselte Umgebungen oder Tests. **Nur** für Tests und Entwicklungsprofile; produktiver Build verlangt einen Owner-Schlüssel (R12, `INV-KV-5`).

### 3.3 Kernkomponenten
* `PrefixRadixTree`: Radix-Baum für Prefix-Caching mit Verzweigungs- und Zusammenführungslogik.
* `TenantIsolatedKvStore`: Speicherpool mit Kapazitätsgrenzen, Invalidation und neutraler Segmentzerstörung.
* `KvCacheService`: Asynchroner Orchestrator mit Pipeline für Lookup, Insert und Eviction.

---

## 4. Feature-Flags

* `default = ["std", "ports-compat"]`
* `kivi-quantization`: Aktiviert KIVI 2-Bit/4-Bit Quantisierungskerne. (Das im Code zusätzlich vorhandene Duplikat `kvcache-kivi-quant` wird entfernt, R12.)
* `kvcache-attention-eviction`: Aktiviert aufmerksamkeitsgewichtete Verdrängung.
* `ports-compat` (transitional, bis G-CUTOVER): Anbindung an `contextra-ports` und `contextra-types`.
* **R12 entfernt (K-61):** `content-addressed-kv-cache` (Inhaltsadressierung über Löschdomänen hinweg ist unzulässig, `INV-PRIV-1`), `kv-encryption`, `crypto-compat` (keine Kante zu `contextra-crypto`).

---

## 5. Concurrency & Durability

* Concurrency via `tokio::sync::RwLock` / `parking_lot::RwLock`.
* Keine Blockaden über Async-Grenzen hinweg.
* In-Memory Caching. **R12 (K-48, K-60):** Überlauf erfolgt nicht über eine eigene Datei-Schicht (`KvSpillStorage` entfällt), sondern über `SegmentOwner` der Index-Bridge. `KvSpillStorage` bleibt nur bis G-CUTOVER als Trait-Platzhalter.

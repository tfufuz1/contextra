# ADR-KV-CIPHER-SCOPE: Scope & Alignment Analysis of KV Encryption Mechanisms

* **Status:** Proposed
* **Datum:** 2026-10-02
* **Autoren:** Principal Rust Security Architect (Google-Jules)
* **Betroffene Komponenten:** `crates/contextra-crypto`, `crates/contextra-kvcache`, `crates/contextra-store`, `crates/contextra-infer-candle`

---

## 1. Kontext & Problemstellung

Im Contextra-Workspace existieren zwei scheinbar parallele Verschlüsselungs- und Schlüsselverwaltungspfade für Key-Value-Daten:

1. **`KvSegmentCipher` / `KvCipher` in `crates/contextra-crypto/src/kv_cipher.rs`:**
   Dient der AEAD-Verschlüsselung (AES-256-GCM-SIV) von In-Memory-KV-Cache-Aktivierungstensoren in `crates/contextra-kvcache` (und `crates/contextra-infer-candle`). Er erzwingt eine kryptographische Isolierung nach dem Tupel `(TenantId, ModelFingerprint)`.
2. **`KeyRegistry` & `KeyManager` in `crates/contextra-store/src/kv/segment.rs` bzw. `crates/contextra-store/src/sstable/builder.rs`:**
   Dienen der Verschlüsselung auf der persistenten Storage-Engine-Ebene (`contextra-store`). `KeyRegistry` verwaltet dynamische Subkeys für $O(1)$ Crypto-Shredding (DSGVO Art. 17 Löschbeweis) über `group_id`, während `KeyManager::derive_file_key` / `encrypt_auto_nonce` für die AES-GCM-SIV-Block-Verschlüsselung von SSTables und WALs verwendet wird.

Diese Architekturanalyse untersucht, ob die Trennung dieser beiden Mechanismen technisch begründet ist oder ob es sich um ungenutztes/redundantes Scaffolding handelt.

---

## 2. Detaillierter Vergleich der Mechanismen

| Kriterium | In-Memory KV-Cache Mechanism (`KvSegmentCipher`) | Persistent Storage Engine Mechanism (`KeyRegistry` / `KeyManager`) |
| :--- | :--- | :--- |
| **Primärer Zweck** | Mandanten- & Modellisolierung für ephemere LLM KV-Cache Tensoren im RAM. | Dauerhafte Encryption-at-Rest & DSGVO Art. 17 Crypto-Shredding für LSM SSTables / WAL. |
| **Zugrundeliegender KeyManager** | `contextra_crypto::crypto::KeyManager` (Master PRK via Argon2id / HKDF-SHA256). | Identischer `contextra_crypto::crypto::KeyManager`-Typ aus `contextra-crypto`. |
| **HKDF Key-Derivation Info** | `"contextra-kv-layer-v2:"` + `TenantId` + `ModelFingerprint` (hash, model_id, quantization) ODER `"contextra-kv-v1-segment-{tenant}-{segment}"`. | `"contextra-kv-shred-v1:"` + `group_id` (u64) via `KeyRegistry` ODER `"contextra-file-key-v1:"` + `file_id` via `KeyManager`. |
| **Nonce-Handling** | Generiert frische 12-Byte Nonces via `rand::rngs::OsRng` pro Encrypt-Aufruf (in `KeyRegistry::encrypt_with_group` & `KvSegmentCipher`). | `KeyManager::encrypt_auto_nonce` nutzt ein 4-Byte `OsRng` Prefix + atomaren 8-Byte `AtomicU64`-Zähler. |
| **Crypto-Shredding (Löschung)** | **Funktional / Ephemer:** Durch Verwerfen/Zeroizen des In-Memory-Segment-Keys. Kein O(1) Revocation-Index. | **Gezielt / O(1):** `KeyRegistry::revoke_subkey(group_id)` löscht den Subkey aus der In-Memory `HashMap` in $O(1)$, wodurch Persistenzdaten unlesbar werden. |
| **Performance & Overhead** | HKDF-Derivation pro Segment/Model-Wechsel + `OsRng`-Aufruf pro Nonce (höherer Overhead, optimiert für Tensor-Blöcke). | Subkey in `KeyRegistry` gecacht (RwLock-Lookup in $O(1)$); `encrypt_auto_nonce` nutzt schnellen atomaren Zähler ohne Syscall. |

### Detaillierte Befunde zu den Komponenten

1. **Gleicher `KeyManager`-Typ:**
   Sowohl `KvSegmentCipher` als auch `KeyRegistry` und `SstableBuilder` basieren auf derselben `KeyManager`-Struktur in `crates/contextra-crypto/src/crypto.rs`. Es gibt keinen duplizierten KeyManager-Typ.
2. **Unterschiedliche Domain Separation (HKDF Info Strings):**
   - `KvSegmentCipher` bindet Schlüssel an Modellparameter (`ModelFingerprint`, Quantisierung) und Mandanten-ID (`TenantId`). Dadurch wird garantiert, dass ein Modellwechsel oder ein Quantisierungswechsel bestehende KV-Cache-Segmente kryptographisch unlesbar macht.
   - `KeyRegistry` in `contextra-store` bindet Schlüssel an `group_id` (Anzahl von Records/Tombstones) für zielgerichtetes Crypto-Shredding unabhängig von Modellarchitekturen.
3. **Nonce-Generierungsstrategie:**
   - `KeyManager::encrypt_auto_nonce` verwendet ein 4-Byte `OsRng` Random Prefix kombiniert mit einem atomaren 8-Byte `AtomicU64`-Zähler. Dies vermeidet Betriebssystem-Syscalls pro Block und ist hochperformant für LSM-WAL/SSTable-I/O.
   - `KvSegmentCipher` nutzt frische `OsRng`-Nonces per Call für maximale Nonce-Misuse-Resistance bei In-Memory-Tensor-Aktivierungen.

---

## 3. Historischer Kontext & Design-Absicht

Eine Überprüfung der Code-Kommentare, der Git-Historie und der Dokumentation (`docs/decisions/ADR-072-kv-bridge-increment-2-kvsegment.md`, `ADR-N11-kvcache-doppelter-tenant-store.md` sowie Audits) zeigt Folgendes:

1. **Kein veraltetes Scaffolding:** `KvSegmentCipher` ist voll produktiv in `crates/contextra-infer-candle` (`kv_bridge.rs`) und `crates/contextra-kvcache` integriert. Es handelt sich **nicht** um ungenutzten Code.
2. **Bewusste Schichtentrennung:**
   - **LSM Storage Layer (`contextra-store`):** Benötigt langlebige Persistenz, SSTable/WAL-Verschlüsselung und $O(1)$ Crypto-Shredding über Gruppen-IDs zur Einhaltung von DSGVO Artikel 17 (Recht auf Vergessenwerden).
   - **Inference KV-Cache Layer (`contextra-kvcache` / `contextra-infer-candle`):** Benötigt ephemere RAM-Sicherheit, Zeroization-on-Drop (`ZeroizeOnDrop`) und kryptographische Schranken zwischen Mandanten und LLM-Quantisierungsstufen (z.B. Llama-3 Q4 vs Q8).

---

## 4. Architekturbewertung: Trennung vs. Konsolidierung

### Bewertung: **Die Trennung ist technisch vollumfänglich gerechtfertigt.**

Ausschlaggebende Gründe gegen eine erzwungene Vereinheitlichung:

1. **Orthogonale Sicherheitsdomänen:**
   - Ein KV-Cache-Segment repräsentiert In-Memory-Aktivierungstensoren eines spezifischen LLM-Inferenz-Laufs. Die Kryptographie muss hier `ModelFingerprint` und `TenantId` erzwingen.
   - Ein LSM-Storage-Segment speichert serialisierte Chunks/Dokumente auf Festplatte. Die Kryptographie muss hier $O(1)$ Schlüsselwiderruf (`KeyRegistry::revoke_subkey`) für rechtssichere Löschbeweise (`INV-KV-DELETE-1`) bieten.
2. **Performance-Anforderungen:**
   - Im Storage-Layer muss der Nonce-Generator extrem durchsatzstark sein (atomare Zähler in `encrypt_auto_nonce`).
   - Im KV-Cache-Layer steht die Vermeidung von Cross-Model-/Cross-Tenant-Data-Leaks im Vordergrund.

---

## 5. Empfehlungen für den menschlichen Entscheidungsträger

Falls in Zukunft eine Harmonisierung angestrebt wird, werden folgende Optionen zur Entscheidung vorgelegt:

### Option A: Status Quo beibehalten (Empfohlen)
* **Beschreibung:** Beibehaltung der klaren fachlichen Trennung zwischen `KvSegmentCipher` (In-Memory/Inferenz) und `KeyRegistry`/`KeyManager` (Persistenter Storage).
* **Vorteile:** Keine Regressionsrisiken, optimale Performanz und exakt abgestimmte Sicherheitsgarantien für beide Subsysteme.
* **Aufwand:** 0 Personentage.

### Option B: Vereinheitlichung der Trait-Schnittstelle (`KvCipher`)
* **Beschreibung:** Implementierung des Trait `KvCipher` für `KeyRegistry`, sodass In-Memory-KV-Cache-Segmente optional auch eine `KeyRegistry` als Verschlüsselungs-Backend akzeptieren können (z.B. für Auslagerung/Tiering auf Disk).
* **Vorteile:** Bessere Interoperabilität, wenn KV-Cache-Segmente über Tier-2-Spill in die LSM Storage Engine ausgelagert werden.
* **Nachteile:** Leichter Anstieg der Komplexität im Trait-Dispatching.
* **Geschätzter Aufwand:** ca. 2–3 Personentage.

---

## 6. Separater, risikoarmer Dokumentations-Fix (Gemäß Punkt 4)

Im Zuge der Recherche wurde eine Veraltung in den Modul-Kommentaren von `crates/contextra-crypto/src/kv_cipher.rs` identifiziert:
- **Problem:** Der Modulkommentar referenziert `crates/contextra-kv-bridge`, ein Crate, das in früheren Refactorings in `crates/contextra-crypto/src/kv_segment/` konsolidiert und in `contextra-kvcache` sowie `contextra-infer-candle` integriert wurde.
- **Korrektur:** Aktualisierung des Modulkommentars in `crates/contextra-crypto/src/kv_cipher.rs`, um auf `contextra-kvcache` und `contextra-infer-candle` anstelle der nicht mehr existierenden Pfade zu verweisen.

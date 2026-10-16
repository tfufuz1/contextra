# Verhaltensoffizielle Architekturspezifikation: KV-Bridge (`contextra-infer-candle`)

## 1. Zweck

Die KV-Bridge (`kv-bridge`) stellt die Laufzeitbrücke zwischen der nativer Candle-LLM-Inferenz (`CandleLlmClient` in `crates/contextra-infer-candle/src/inference/core.rs`) und dem mandantenisolierten, verschlüsselten Key-Value-Cache-Speicher (`TenantIsolatedKvStore` in `crates/contextra-crypto/src/kv_segment/store.rs`) dar.

Sie ermöglicht das Zwischenspeichern und Wiederverwenden von verarbeiteten Präfix-Kontext-Segmenten (Key-Value-Tensors) bei wiederholten Inferenzanfragen. Dadurch wird der kostenintensive Prefill-Schritt (Prompt Encoding) vermieden und die Antwortlatenz signifikant reduziert.

Im Sinne der **Fail-Safe-Doktrin** gilt: Bei jedem Speicher- oder Krypto-Fehler, Versionskonflikt, RoPE-Offset-Mismatch oder fehlenden Segmenten fällt das System transparent ohne Abbruch (Panic/Error) auf den vollständigen Prefill-Pfad zurück (`try_get_cached_segment` liefert `None`).

---

## 2. Datenfluss (ASCII-Skizze)

```
[CandleLlmClient]
       |
       | 1. generate_with_context(tenant, segments)
       v
[KvBridgeAdapter] ── 2. consult_segment(&segment) ──> Increment consultation counter
       |
       | 3. try_get_cached_segment(tenant, key)
       v
[TenantIsolatedKvStore (RAM Tier 1)]
       |
       |-- (Tier 1 Hit) ──> Decrypt via [KvSegmentCipher] ──> Payload validation
       |
       |-- (Tier 1 Miss & LSM Fallback) ──> [KvBridgeStorage (LSM Tier 2)]
                                                      |
                                                      v
                                        Decrypt via [KvSegmentCipher]
                                                      |
                                                      v
                                            Validate Payload
                                                      |
                                                      v
                                       (Hit: Return Bytes / Miss: None)
       |
       |-- (Cache Miss on all Tiers)
       v
[Candle Inferenz (Prefill Execution)]
       |
       | 4. store_segment(tenant, key, plaintext_kv_bytes)
       v
[KvBridgeAdapter] ── 5. bincode::serialize(CachedKvPayload)
       v
[KvSegmentCipher] ── 6. AES-256-GCM-SIV Encrypt (mit Tenant-ID & Chunk-ID AAD)
       v
[TenantIsolatedKvStore] (ggf. Tier 2 Spill via async SpillHandler bei Eviction)
```

---

## 3. Exportierter Zustand

Die KV-Bridge unterscheidet zwei Abstraktionsebenen bezüglich des transportierten Zustands:

### A. Segment-Level Bridge (`KvBridgeAdapter`)
Der Adapter transportiert opaque serialisierte Bytes eines gecachten KV-Segments (`CachedKvPayload` in `crates/contextra-infer-candle/src/kv_bridge.rs:21-25`). Dieser Zustand kapselt:
- `fingerprint`: `ModelFingerprint` (32-Byte SHA-256 Modell-Hash, Modell-ID, Quantisierungsstufe, z. B. `"llama-3.2-1b.gguf"`, `"Q4_K_M"`).
- `rope_offset`: `Option<usize>` (Position im RoPE-Einbettungsraum).
- `data`: `Vec<u8>` (Plaintext-Tensor-Byte-Payload).

### B. Tensor-State (`KvState` & `LayerKv`)
Unter dem Feature `kv-stage-b` (`crates/contextra-infer-candle/src/kv_state.rs:13-32`) exportiert und rekonstruiert `KvState` die eigentlichen Aufmerksamkeits-Tensoren:
- **Layer-Struktur**: Multi-Layer-Array `Vec<LayerKv>`, wobei jedes `LayerKv` aus einem Key-Tensor (`k`) und Value-Tensor (`v`) besteht.
- **Tensor-Shapes**:
  - Key Tensor `k`: Shape `(batch, seq_len, n_kv_head, head_dim)` oder `(batch, n_kv_head, seq_len, head_dim)`.
  - Value Tensor `v`: Shape `(batch, seq_len, n_kv_head, head_dim)` oder `(batch, n_kv_head, seq_len, head_dim)`.
- **Unterstützte Datentypen (dtype)**:
  - `f32`: 32-Bit Floating Point (Little-Endian Byte-Serialisierung).
  - `f16`: 16-Bit Half Precision (IEEE 754-2008 `f16` via `half::f16`, Little-Endian Byte-Serialisierung).
  - Andere Datentypen (z. B. `BF16`, `INT8`) lösen bei Export/Import einen typisierten `ContextraError::CapabilityUnsupported` aus (`crates/contextra-infer-candle/src/kv_state.rs:88, 134`).
- **Quantisierung**: Die KV-Cache-Tensoren werden derzeit in unquantisierter FP32/FP16-Repräsentation auf CPU-Device gehalten (`crates/contextra-infer-candle/src/kv_state.rs:58, 75`).

---

## 4. Serialisierungsformat und Versionierung

### Serialisierungsstruktur (`CachedKvPayload` & `KvStateBlockPayload`)
Die Daten-Payloads werden mit `bincode` (Little-Endian, Fixint Encoding) binarisiert:

1. **`CachedKvPayload`** (`crates/contextra-infer-candle/src/kv_bridge.rs:21-25`):
   ```rust
   struct CachedKvPayload {
       fingerprint: ModelFingerprint, // Hash (32 bytes) + Model ID + Quantization
       rope_offset: Option<usize>,    // Positional RoPE Offset
       data: Vec<u8>,                 // Serialisierte Tensor-Daten
   }
   ```

2. **`KvStateBlockPayload`** (`crates/contextra-infer-candle/src/kv_state.rs:47-51`):
   ```rust
   struct KvStateBlockPayload {
       layers: Vec<LayerPayload>,     // Vec<{ k: TensorPayload, v: TensorPayload }>
       seq_len: usize,                // Sequenzlänge des Blocks
   }
   ```
   Wobei `TensorPayload` die Form `shape: Vec<usize>`, `dtype: String` (`"f32"` / `"f16"`) und `data: Vec<u8>` speichert (`crates/contextra-infer-candle/src/kv_state.rs:34-39`).

### Versionierung
- **Krypto-Layer Versionierung**: `EncryptedKvLayer` speichert ein explizites Feld `format_version: u16` (`crates/contextra-crypto/src/kv_cipher.rs:118`). Aktuelle Standardversion ist `1`.
- **Modell-Fingerprint Validierung**: Bei `validate_payload` (`crates/contextra-infer-candle/src/kv_bridge.rs:133-165`) muss der in der Payload gespeicherte `ModelFingerprint` exakt mit dem `ModelFingerprint` des aktuellen Anfragestehlers übereinstimmen. Bei Abweichung wird ein Log-Eintrag erzeugt und `None` zurückgegeben.

---

## 5. Verschlüsselung

Die Verschlüsselung der KV-Cache-Segmente erfolgt über die Krypto-Komponenten in `contextra-crypto`:

- **Schlüsselherkunft & Hierarchie**:
  - Master Key Manager (`CryptoKey` in `crates/contextra-crypto/src/crypto.rs`).
  - `KvSegmentCipher` (`crates/contextra-crypto/src/kv_cipher.rs:99`) leitet mandantenspezifische Schlüssel über eine KDF unter Einbeziehung der `TenantId` ab.
- **Verschlüsselungsalgorithmus**:
  - **AES-256-GCM-SIV** (AEAD mit synthetischer Initialisierung / Misuse-Resistant AEAD).
- **Nonce-Strategie & Zufall**:
  - Die Nonce-Generierung erfolgt kryptografisch sicher innerhalb der Krypto-Schicht (`contextra-crypto/src/kv_cipher.rs`) über ein System-RNG.
- **Authenticated Additional Data (AAD)**:
  - Die Ver- und Entschlüsselung bindet den Kontext an die `TenantId` (als `u64`) und die `segment_id` (`chunk_id` als `u64`) als AAD ein (`crates/contextra-crypto/src/kv_cipher.rs:130`). Versuche, das Chiffretext-Segment eines Mandanten unter der Tenant-ID eines anderen Mandanten zu entschlüsseln, schlagen auf AEAD-Ebene fehl.

---

## 6. Isolationsgarantien und deren Grenzen

### Garantien
1. **Mandanten-Trennung (Tenant Isolation)**:
   - Die Speicherung im `TenantIsolatedKvStore` ist strikt nach `TenantId` partitioniert (`crates/contextra-crypto/src/kv_segment/store.rs:20-30`).
   - Ein Mandant B kann auf Segmente von Mandant A weder zugreifen noch diese durch Schlüsselablesung entschlüsseln.
2. **Krypto-Shredding & Eviction**:
   - Beim Löschen eines Mandanten-Schlüssels im `TenantIsolatedKvStore` werden alle zugehörigen Segmente unlesbar (Cryptographic Shredding).
3. **Fail-Closed Krypto-Schicht**:
   - Wenn der Entschlüsselungsversuch fehlschlägt (z. B. falscher Key, manipulierter Chiffretext, abgeschnittene Bytes), wird der Fehler von `try_get_cached_segment` gefangen und als Cache-Miss (`None`) gewertet.

### Grenzen
- **Speicherbegrenzung**: Der In-Memory-RAM-Store unterliegt Kapazitätsgrenzen (z. B. Segmentanzahl per Tenant). Bei Überschreitung greift der `EvictionWorker` bzw. der LSM-Spill-Handler (`crates/contextra-infer-candle/src/kv_bridge.rs:72-113`).
- **Prozessinterne Isolation**: Innerhalb des gleichen Rust-Prozessraums vertraut die KV-Bridge darauf, dass keine Speicherzugriffe außerhalb der sicheren Safe-Rust-Invarianten erfolgen (`#![forbid(unsafe_code)]` in `crates/contextra-infer-candle/src/lib.rs:18`).

---

## 7. Fehlerverhalten

Das Fehlerverhalten der KV-Bridge folgt ausnahmslos der **Zero-Panic-Doktrin** (`crates/contextra-infer-candle/src/kv_bridge.rs:10`):

| Fehlerszenario | Auslöser | Verhalten von `KvBridgeAdapter` | Auswirkung auf Inferenz |
| :--- | :--- | :--- | :--- |
| **Deserialisierungsfehler** | Manipulierte/Beschädigte Bytes in `data` | `validate_payload` fängt `bincode::Error`, loggt Warning, liefert `None` | Transparenter Fallback auf vollen Prefill |
| **Modell-Mismatch** | `ModelFingerprint` der Payload weicht ab | `validate_payload` erkennt Mismatch, loggt Warning, liefert `None` | Transparenter Fallback auf vollen Prefill |
| **RoPE-Offset-Mismatch** | `rope_offset` der Anfrage weicht ab | `validate_payload` erkennt Mismatch, loggt Warning, liefert `None` | Transparenter Fallback auf vollen Prefill |
| **Krypto-Entschlüsselungsfehler** | Falscher Key, abgeschnittener Chiffretext | `try_get_cached_segment` fängt `CryptoError`, loggt Warning, liefert `None` | Transparenter Fallback auf vollen Prefill |
| **LSM Spill Store I/O Error** | Fehler beim Lesen aus `lsm_store` | `try_get_cached_segment_async` fängt I/O-Fehler, liefert `None` | Transparenter Fallback auf vollen Prefill |

---

## 8. Feature-Kette

Die Abhängigkeiten des Features `kv-bridge` sind in `crates/contextra-infer-candle/Cargo.toml` wie folgt verdrahtet:

```toml
[features]
kv-bridge = ["dep:contextra-crypto", "contextra-crypto/kv-encryption", "dep:bincode"]
```

Weiterleitung in übergeordneten Crates (`crates/contextra-mcp/Cargo.toml`):
```toml
[features]
kv-bridge = ["contextra-infer-candle/kv-bridge", "dep:contextra-infer-candle"]
```

---

## 9. Offene Fragen (Perspektive: KV-Cache als viertes Fusionssignal)

### Frage / Perspektive: KV-Cache-Signal als 4. Signal in der RAG-Fusion
*Aktuell umfasst die RAG-Fusion in Contextra (z. B. in `contextra-rank` und `contextra-engine`) drei Primärsignale:*
1. **BM25 / Lexikalisches Signal** (Textsuche)
2. **Vektor / Semantisches Signal** (Dense Embeddings)
3. **Graph / Entitäten-Signal** (MemFuse Graph Anchors)

*Perspektivische Fragestellung (nur Architektur-Ausblick, nicht zu implementieren):*
Kann ein **KV-Cache-Hit- / Prefix-Reuse-Signal** (z. B. Wiederverwendungsgrad des Präfix-Caches, Attention-Dichte bereits im Cache vorhandener Segmente oder KV-Block-Warmness) als **viertes Fusionssignal** in `MultiStepEngine` / `mRRF` integriert werden?

- **Vorteil**: Dokumente / Chunks, die sich bereits im betriebswarmen KV-Cache des LLM befinden, könnten bei geringem Relevanzabstand leicht bevorzugt werden (Latency-Aware Re-Ranking / Cache-Conscious RAG).
- **Offene Forschungsfragen**:
  - Beeinträchtigt eine Bevorzugung gecontenter Segmente die semantische Präzision (Fairness vs. Latenz-Optimierung)?
  - Wie wirkt sich die Dynamik der Modulationsfunktion $w_s'$ im `mRRF` auf ein solches System-Latenz-Signal aus?

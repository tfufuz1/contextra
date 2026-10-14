superseded by CONTEXTRA_ENDPRODUKT_SPEZIFIKATION.md

# Contextra — Mikrofeingranulare Implementierungsspezifikation der disjunktiven Endprodukt-Komponenten (EP1–EP6)

**Dokumenten-ID:** `DOC-SPEC-EP-DISJUNCT-2026-10-14`  
**Datum:** 14. Oktober 2026  
**Status:** Verbindliche normative Spezifikation (Principal Senior Rust Architect)  
**Bezugsdokumente:**
- `docs/archive/14-10-26/ARCHITEKTUR_MASTERPLAN_v2.md`
- `docs/archive/14-10-26/SOTA_ALGORITHMIC_FEINSPEZIFIKATION_2026.md`
- `docs/archive/14-10-26/LOESCHNACHWEIS_SYSTEMSPEZIFIKATION_v2.md`
- `docs/archive/14-10-26/ADVANCED_ALGORITHMS_SPEC_v1.md`
- `docs/analysis/AUDIT_GESAMTSYNTHESE_2026-10-14.md`
- `capabilities.toml` (Single Source of Truth für Ring-Architektur)
- `docs/gaps/pub_fn_ohne_aufrufer.csv` & `docs/gaps/undokumentiert_streng.csv`

---

## Inhaltsverzeichnis

1. [Architektonische Gesamtdiagnose & Blinde Flecken der Vorplanung](#1-architektonische-gesamtdiagnose--blinde-flecken-der-vorplanung)
2. [Disjunktive Crate- und Komponenten-Matrix](#2-disjunktive-crate--und-komponenten-matrix)
3. [EP1: Contextra Memory — Lokaler Multi-Tenant-Speicher & Purge v2](#3-ep1-contextra-memory--lokaler-multi-tenant-speicher--purge-v2)
4. [EP2: Contextra Proof — Verifizierbare Löschnachweise & BSI TR-02102](#4-ep2-contextra-proof--verifizierbare-löschnachweise--bsi-tr-02102)
5. [EP3: Contextra Retrieval+ — Wissensgraph, OpenIE & Conformal PathRAG](#5-ep3-contextra-retrieval--wissensgraph-openie--conformal-pathrag)
6. [EP4: Contextra Inference — Lokale Embedder, Irminsul KV-Cache & 2-Bit-KIVI](#6-ep4-contextra-inference--lokale-embedder-irminsul-kv-cache--2-bit-kivi)
7. [EP5: Contextra Cognition — Adaptive Kognition, Sketched Bandits & Spectral SVD](#7-ep5-contextra-cognition--adaptive-kognition-sketched-bandits--spectral-svd)
8. [EP6: Contextra Guard — Egress Firewall, MCP-Sicherheit & Sandbox-Isolation](#8-ep6-contextra-guard--egress-firewall-mcp-sicherheit--sandbox-isolation)
9. [Workspace-weite Verdrahtung, Invarianten & Gate-Durchsetzung](#9-workspace-weite-verdrahtung-invarianten--gate-durchsetzung)

---

## 1. Architektonische Gesamtdiagnose & Blinde Flecken der Vorplanung

Eine gründliche Revision der Vorplanungen in `docs/archive/14-10-26/`, der Audits von gestern (`docs/analysis/`, `logs/`) sowie des aktuellen Workspace-Zustands (`find-unwired-gaps`: 736 Dateien, 5.502 pub-Symbole, 1.057 undokumentiert, 512 pub fn ohne Aufrufer, 43+ unverdrahtete Traits) deckt fundamentale Lücken auf:

### 1.1 Blinde Flecken in den Vorplanungsdokumenten

1. **Ring-Inkonsistenz von `contextra-kvcache`:**
   - In `docs/archive/14-10-26/SOTA_ALGORITHMIC_FEINSPEZIFIKATION_2026.md` (§1) wird `contextra-kvcache` fälschlich als `Ring 0` deklariert.
   - *Korrektur:* Laut `capabilities.toml` (§8 AGENTS.md) gehört `contextra-kvcache` zwingend in **Ring 2 (Domain Logic)**, da es komplexe Caching-, Quantisierungs- und Eviction-Algorithmen enthält. Ring 0 ist streng reserviert für fundamentale Typen (`contextra-types`) und Fehler (`contextra-errors`).
2. **Mathematische Determinismus-Lücke (P28):**
   - Brand's Incremental SVD und Conformal Calibration erfordern Operationen über reellen Zahlen. Bisherige Entwürfe spezifizieren keine deterministische Akkumulation. Ohne Kahan-Summation oder fixierte Assoziativität erzeugen verschiedene CPU-Architekturen unterschiedliche Rundungsergebnisse, was kryptografische Hashes invalidiert.
3. **Physische Scope-Diskrepanz im Store (N-K):**
   - `TenantScopedStorage` kodiert Schlüssel als `t:{tenant_id}:{user_key}`. Der Löschpfad suchte nach logischen Präfixen wie `__col:{id}:` und löschte im LSM-Baum schlichtweg null Datensätze.
4. **Fassaden-Bypass im MCP-Server (B9 & B4):**
   - `contextra-mcp` re-implementierte eigene Aufrufe, anstatt `ContextraDb` als einheitlichen Composition Root zu nutzen. `contextra_forget` rief zwar intern Löschlogik auf, verwarf das Ergebnis jedoch und lieferte hardcodiert `proof = null` zurück.
5. **Phantom-Optionen im Builder (S5):**
   - 7 Schalter im `ContextraBuilder` (`with_privacy`, `with_adapt`, `with_kms`, `with_kv_shredding`, `with_checkpoint`, `with_avv_generator`, `with_audit_export`) speicherten Bool-Werte, die in `build()` mit `let _ = ...;` ignoriert wurden.

---

## 2. Disjunktive Crate- und Komponenten-Matrix

Um zirkuläre Abhängigkeiten zu verhindern, Compiler-Zeiten zu minimieren und maximale Modularität zu gewährleisten, wird jedes der 6 Endprodukte strikt disjunktiven Crates und Modulen zugeordnet:

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                              DISJUNKTIVE PRODUKT-ARCHITEKTUR                           │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  EP1: CONTEXTRA MEMORY (Release 1.0)                                                   │
│  ├── Ring 0: contextra-types, contextra-errors                                         │
│  ├── Ring 1: contextra-ports (Basis-Traits)                                            │
│  ├── Ring 2: contextra-store, contextra-vector, contextra-text, contextra-mvcc         │
│  └── Ring 4: contextra-engine (Kern), contextra (Basis-Fassade)                         │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  EP2: CONTEXTRA PROOF (Compliance & Beweisführung)                                     │
│  ├── Ring 0: contextra-crypto-core, contextra-proof-verify (Standalone Verifier)       │
│  ├── Ring 2: contextra-crypto (Keys, Shredding, Nonces)                                │
│  └── Ring 3: contextra-erasure-evidence, contextra-audit-export, contextra-avv-generator│
├────────────────────────────────────────────────────────────────────────────────────────┤
│  EP3: CONTEXTRA RETRIEVAL+ (Graph, PathRAG & Conformal Fusion)                         │
│  ├── Ring 2: contextra-graph (CSR, OpenIE, PathRAG)                                    │
│  └── Ring 2: contextra-rank (RRF, DiBud, Conformal Calibration)                        │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  EP4: CONTEXTRA INFERENCE (Embedder & KV-Cache-Management)                             │
│  ├── Ring 2: contextra-kvcache (Irminsul, KIVI 2-Bit, SnapKV)                          │
│  ├── Ring 2: contextra-checkpoint (Hardlink Cloner, Copy Registry)                     │
│  └── Ring 3: contextra-embedder (Candle, Ollama, ONNX Adapter)                         │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  EP5: CONTEXTRA COGNITION (Adaptive Kognition & Hypergraph-Reasoning)                  │
│  ├── Ring 2: contextra-cognition (Hyperedge Engine, Spectral SVD)                      │
│  ├── Ring 2: contextra-adapt (Sketched LinUCB, PID Latency Controller, Thompson)       │
│  └── Ring 4: contextra-agent (CLM Scratchpad, Goal Graph, DLQ)                         │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  EP6: CONTEXTRA GUARD (Sicherheitsfassade & Isolation)                                 │
│  ├── Ring 2: contextra-privacy (Egress Vault, PII Masking, Hash-Chained Audit)         │
│  ├── Ring 3: contextra-sandbox (WASM Plugin Host)                                      │
│  ├── Ring 3: contextra-mcp (Model Context Protocol Gateway)                            │
│  └── Ring 4: contextra-py (Python CFFI Bindings)                                       │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. EP1: Contextra Memory — Lokaler Multi-Tenant-Speicher & Purge v2

### 3.1 Zweck & Geltungsbereich
Das fundamentale Fundament für hochperformanten, isolierten Vektor-, Volltext- und KV-Speicher auf einem einzelnen Knoten. Gewährleistet ACiD-Transaktionen (SSI), Zero-Panic, Tenant-Isolation und physikalisch wirksame Löschung (Purge v2) im Modus `ErasureMode::Off` und `ErasureMode::Assured`.

### 3.2 Exklusive Crate-Verantwortlichkeit
- `contextra-types` & `contextra-errors` (Ring 0)
- `contextra-ports` (Ring 1)
- `contextra-store`, `contextra-vector`, `contextra-text`, `contextra-mvcc` (Ring 2)
- `contextra-engine`, `contextra` (Ring 4)

### 3.3 Mikrofeingranulare Spezifikation der Behebungen & Verdrahtungen

#### 3.3.1 Behebung N-B & N-K: `ErasureBackend`-Trait & `PhysicalScope`
In `crates/contextra-ports/src/erasure.rs` und `storage.rs`:
```rust
// crates/contextra-ports/src/erasure.rs
use crate::BoxFuture;
use contextra_types::{Result, TxId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalScope {
    pub exact_keys: Vec<Vec<u8>>,
    pub prefixes: Vec<Vec<u8>>,
}

impl PhysicalScope {
    pub fn new(exact_keys: Vec<Vec<u8>>, prefixes: Vec<Vec<u8>>) -> Self {
        Self { exact_keys, prefixes }
    }
    
    pub fn matches(&self, key: &[u8]) -> bool {
        self.exact_keys.iter().any(|k| k == key)
            || self.prefixes.iter().any(|p| key.starts_with(p))
    }
}

pub trait ErasureBackend: Send + Sync + 'static {
    fn resolve_physical_scope(&self, scope: &LogicalScope) -> Result<PhysicalScope>;
    fn purge<'a>(
        &'a self,
        scope: &'a PhysicalScope,
        as_of: TxId,
    ) -> BoxFuture<'a, Result<PurgeOutput>>;
}
```

Implementierung in `crates/contextra-store/src/tenant_codec.rs`:
```rust
// TenantScopedStorage implementiert ErasureBackend und übersetzt logische Scopes
// unter strikter Berücksichtigung des Mandanten-Präfixes (Behebung N-K)
impl<S: StorageEngineHandle> ErasureBackend for TenantScopedStorage<S> {
    fn resolve_physical_scope(&self, scope: &LogicalScope) -> Result<PhysicalScope> {
        let prefix = self.codec.tenant_prefix();
        match scope {
            LogicalScope::Document { doc_id, .. } => {
                let encoded = self.codec.encode_key(doc_id.to_string().as_bytes());
                Ok(PhysicalScope::new(vec![encoded], Vec::new()))
            }
            LogicalScope::Collection { collection_id, .. } => {
                let col_str = format!("__col:{}:", collection_id.inner());
                let enc_prefix = self.codec.encode_key(col_str.as_bytes());
                Ok(PhysicalScope::new(Vec::new(), vec![enc_prefix]))
            }
            LogicalScope::Prefixes(raw_prefixes) => {
                let mapped = raw_prefixes.iter()
                    .map(|p| self.codec.encode_key(p))
                    .collect();
                Ok(PhysicalScope::new(Vec::new(), mapped))
            }
            LogicalScope::Keys(raw_keys) => {
                let mapped = raw_keys.iter()
                    .map(|k| self.codec.encode_key(k))
                    .collect();
                Ok(PhysicalScope::new(mapped, Vec::new()))
            }
        }
    }

    fn purge<'a>(
        &'a self,
        scope: &'a PhysicalScope,
        as_of: TxId,
    ) -> BoxFuture<'a, Result<PurgeOutput>> {
        Box::pin(async move {
            self.inner.purge(scope, as_of).await
        })
    }
}
```

In `crates/contextra-engine/src/collection/crud/delete.rs` (Behebung N-B):
```rust
// Ersetzen des illegalen Downcasts durch Trait-Aufruf
let erasure_backend = self.storage.as_erasure_backend();
let mode = self.erasure_mode();

if mode.requires_ledger() && erasure_backend.is_none() {
    return Err(ContextraError::capability_unsupported(
        "erasure_ledger",
        "Erasure mode Assured/Attested requires an active ErasureLedger",
    ));
}

let physical_scope = match erasure_backend {
    Some(backend) => backend.resolve_physical_scope(&logical_scope)?,
    None => PhysicalScope::new(Vec::new(), Vec::new()),
};
```

#### 3.3.2 Behebung L1–L6 & N-A: Streaming-Rewrite Purge v2 & Crash-Konsistenz
In `crates/contextra-store/src/purge_v2/streaming_rewrite.rs`:
- **L1 (Ledger-First):** Bevor ein Tombstone im MemTable gesetzt wird, MUSS der Eintrag im `ErasureLedger` mit Status `Pending` persistent serialisiert sein (`fsync`).
- **L4 (SSTable Deletion):** Obsolete Quell-SSTables werden in `PurgeOutput.obsolete_files` gesammelt und nach erfolgreichem Manifest-Commit mit Nullen überschrieben und entlinkt (`scrub_and_remove`).
- **L5 (3-Arm-Match):** Beim Lesen aus Quell-SSTables wird Korruption nicht verschluckt:
  ```rust
  match sstable_iter.next() {
      Some(Ok(entry)) => { /* Filter & Write */ }
      Some(Err(err)) => return Err(ContextraError::data_corruption(format!("SSTable read error: {err}"))),
      None => break,
  }
  ```
- **L6 (Metadata Preservation):** `min_seq`, `max_seq` und Rang-Grenzwerte werden in den neu geschriebenen SSTables exakt konserviert.
- **N-A (Intent Resume):** Beim Hochfahren ruft `LsmStorage::open()` zuerst `recover_pending_purges()` auf. Der `PhysicalScope` wird aus dem Ledger rehydriert. Erst nach erfolgreichem Abschluss des Purges wird der Store für reguläre Lese-/Schreibzugriffe freigegeben.

#### 3.3.3 HNSW Memory Zeroization (`GhostFreeVectorIndex`)
In `crates/contextra-vector/src/hnsw/deletion.rs`:
- Verdrahtung des ungenutzten Traits `GhostFreeVectorIndex` in `contextra-engine::collection::crud::delete`:
```rust
pub trait GhostFreeVectorIndex: Send + Sync {
    fn zeroize_and_prune_slots(&self, slot_ids: &[usize]) -> Result<()>;
}

// In HnswIndex implementation:
impl GhostFreeVectorIndex for HnswIndex {
    fn zeroize_and_prune_slots(&self, slot_ids: &[usize]) -> Result<()> {
        let mut arena = self.arena.write();
        for &slot in slot_ids {
            if let Some(entry) = arena.get_mut(slot) {
                entry.vector.zeroize();
                entry.deleted = true;
            }
        }
        self.repair_graph_edges(slot_ids)?;
        Ok(())
    }
}
```

#### 3.3.4 SSI Read-Set Tracking (INV-MVCC-SSI-1)
In `crates/contextra-mvcc/src/ssi.rs`:
```rust
pub struct ReadSnapshotToken {
    pub tx_id: TxId,
    pub read_set: Arc<Mutex<HashSet<Vec<u8>>>>,
}

impl ReadSnapshotToken {
    pub fn record_read(&self, key: &[u8]) {
        if let Ok(mut set) = self.read_set.lock() {
            set.insert(key.to_vec());
        }
    }
}
```
Lese-Iteratoren im Storage-Layer müssen den `ReadSnapshotToken` führen und bei jedem `iter.next()` aufrufen. Zur Commit-Zeit prüft `SequenceLogSsiValidator::validate_commit(tx_id, &read_set, &write_set)`, ob ein Konflikt vorliegt (Write-Skew-Verhinderung).

#### 3.3.5 Builder-Wiring-Validierung (Behebung S5)
In `crates/contextra/src/builder.rs`:
```rust
impl ContextraBuilder {
    pub fn verify_wiring(&self) -> Result<(), BuildError> {
        if self.privacy_enabled && self.egress_guard.is_none() {
            return Err(BuildError::incomplete("privacy", "EgressGuard not configured"));
        }
        if self.adapt_enabled && self.adaptive_controller.is_none() {
            return Err(BuildError::incomplete("adapt", "AdaptiveController not configured"));
        }
        if self.kms_enabled && self.kms_provider.is_none() {
            return Err(BuildError::incomplete("kms", "KmsProvider not configured"));
        }
        if self.checkpoint_enabled && self.checkpoint_coordinator.is_none() {
            return Err(BuildError::incomplete("checkpoint", "CheckpointCoordinator not configured"));
        }
        Ok(())
    }
}
```

---

## 4. EP2: Contextra Proof — Verifizierbare Löschnachweise & BSI TR-02102

### 4.1 Zweck & Geltungsbereich
Rechtsverbindliche, mathematisch und kryptografisch beweisbare Löschung (DSGVO Art. 17, ZPO § 371a). Unabhängige Verifikation ohne Datenbankzustand.

### 4.2 Exklusive Crate-Verantwortlichkeit
- `contextra-crypto` (Ring 2)
- `contextra-erasure-evidence` (Ring 3)
- `contextra-proof-verify` (Ring 0 / Standalone)
- `contextra-audit-export` & `contextra-avv-generator` (Ring 3)

### 4.3 Mikrofeingranulare Spezifikation

#### 4.3.1 Verdrahtung `KmsProvider` (FINDING-CRYPTO-01)
In `crates/contextra-crypto/src/kms.rs`:
```rust
pub trait KmsProvider: Send + Sync + 'static {
    fn derive_dek(&self, tenant_id: TenantId, key_purpose: KeyPurpose) -> Result<VolatileEncryptionKey>;
    fn destroy_dek(&self, tenant_id: TenantId, key_purpose: KeyPurpose) -> Result<()>;
}

pub struct LocalKmsProvider {
    master_key: Zeroizing<[u8; 32]>,
}
```
Verdrahtung im `ContextraBuilder`:
```rust
// In crates/contextra/src/builder.rs
if self.kms_provider.is_none() && self.erasure_mode >= ErasureMode::Assured {
    let local_kms = Arc::new(LocalKmsProvider::from_system_entropy()?);
    self.kms_provider = Some(local_kms);
}
```

#### 4.3.2 7-Schichten-Beweismodell nach BSI TR-02102
Die Erzeugung eines `DeletionProofV4` erfordert lückenlose Zeugen aus 7 Teilsystemen:
```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeletionProofV4 {
    pub format_version: u32, // = 4
    pub tenant_id: TenantId,
    pub target_scope: PhysicalScope,
    pub layer_witnesses: [LayerWitness; 7],
    pub aggregate_hash: [u8; 32],
    pub timestamp_utc: u64,
    pub ed25519_signature: [u8; 64],
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LayerWitness {
    pub layer_id: u8, // 1: Logical, 2: RAM/Zeroize, 3: Index, 4: LSM/SSTable, 5: WAL, 6: CryptoKey, 7: AuditChain
    pub witness_hash: [u8; 32],
    pub verified_at_tx: TxId,
}
```

#### 4.3.3 Behebung B4 (MCP `proof: null`)
In `crates/contextra-mcp/src/server.rs`:
```rust
// Anstatt das Ergebnis zu verwerfen:
let erasure_result = db.erase_document(tenant_id, doc_id).await?;
let response = match erasure_result.proof {
    Some(proof) => json!({
        "status": "erased",
        "proof_version": proof.format_version,
        "signature_base64": base64::encode(proof.ed25519_signature),
        "aggregate_hash": hex::encode(proof.aggregate_hash)
    }),
    None => json!({ "status": "tombstoned" }),
};
```

#### 4.3.4 Bereinigung toter Orphan-Symbole
In `crates/contextra-crypto/src/deletion_proof/proof.rs` und `exclusions.rs`:
- `pub fn default_signature_version()` wird auf `pub(crate)` reduziert.
- `pub fn mandatory_ids()` wird gelöscht; stattdessen wird die deterministische Konstante `REQUIRED_LAYER_BITS: u8 = 0b01111111` genutzt.

---

## 5. EP3: Contextra Retrieval+ — Wissensgraph, OpenIE & Conformal PathRAG

### 5.1 Zweck & Geltungsbereich
Graphenbasierte Wissensrepräsentation, automatische Entitäts- und Relationsextraktion sowie hybride Signal-Fusion (BM25 + Vektor + Graph-Diffusion) mit verteilungsfreier Konformitätsgarantie.

### 5.2 Exklusive Crate-Verantwortlichkeit
- `contextra-graph` (Ring 2)
- `contextra-rank` (Ring 2)

### 5.3 Mikrofeingranulare Spezifikation

#### 5.3.1 Verdrahtung von `KPathDiffusion` (321 LOC unverdrahtet)
In `crates/contextra-graph/src/path_rag/k_path.rs`:
```rust
pub trait KPathDiffusion: Send + Sync {
    fn find_k_paths(
        &self,
        start_nodes: &[NodeId],
        k: usize,
        node_budget: usize,
        diversity_weight: f64,
    ) -> Result<Vec<PathResult>>;
}
```
Verdrahtung in `crates/contextra-engine/src/search/hybrid_search.rs`:
```rust
// Bei Graph-Aktivierung wird KPathDiffusion für multi-hop Pfad-Retrieval aufgerufen
if let Some(graph) = &self.graph_engine {
    let seed_nodes = graph.resolve_entities(&query.text)?;
    let paths = graph.find_k_paths(&seed_nodes, query.top_k, 50, 0.3)?;
    graph_signals.extend(paths.into_iter().map(|p| p.into_signal()));
}
```

#### 5.3.2 Conformal Fusion Calibration (Ersatz von PAVA/Platt)
In `crates/contextra-rank/src/conformal.rs`:
Ersetzt ad-hoc Kalibrierung durch Split-Conformal Prediction:
$$\hat{q} = \text{Quantil}\left(\{s_i\}_{i=1}^n; \frac{\lceil (n+1)(1-\alpha) \rceil}{n}\right)$$
Unter Verwendung von `f64::total_cmp` für deterministische Sortierung:
```rust
pub struct ConformalCalibrator {
    calibration_scores: Vec<f64>,
    alpha: f64,
}

impl ConformalCalibrator {
    pub fn new(mut scores: Vec<f64>, alpha: f64) -> Result<Self> {
        if scores.is_empty() {
            return Err(ContextraError::invalid_input("Calibration set cannot be empty"));
        }
        scores.sort_by(|a, b| a.total_cmp(b));
        Ok(Self { calibration_scores: scores, alpha })
    }

    pub fn compute_threshold(&self) -> f64 {
        let n = self.calibration_scores.len();
        let rank = (((n + 1) as f64 * (1.0 - self.alpha)).ceil() as usize).min(n);
        self.calibration_scores[rank - 1]
    }
}
```

#### 5.3.3 Deterministische OpenIE Pipeline
In `crates/contextra-graph/src/entity_extraction.rs`:
1. Rule-based NER (Regex, Heuristiken, Dictionary).
2. LLM Validation via Port (Fixed Seed, Temperature = 0.0).
3. Kanonische Sortierung:
```rust
extracted_triples.sort_by(|a, b| {
    a.subject_id.cmp(&b.subject_id)
        .then_with(|| a.predicate.cmp(&b.predicate))
        .then_with(|| a.object_id.cmp(&b.object_id))
});
```

---

## 6. EP4: Contextra Inference — Lokale Embedder, Irminsul KV-Cache & 2-Bit-KIVI

### 6.1 Zweck & Geltungsbereich
LLM-Kontextfenster-Optimierung, 2-Bit KV-Cache-Quantisierung mit Prefix-Reuse und lokales Embedding.

### 6.2 Exklusive Crate-Verantwortlichkeit
- `contextra-kvcache` (Ring 2)
- `contextra-checkpoint` (Ring 2)
- `contextra-embedder` (Ring 3)

### 6.3 Mikrofeingranulare Spezifikation

#### 6.3.1 Irminsul-Eviction-Strategie
In `crates/contextra-kvcache/src/irminsul.rs`:
- **Phase 1 (Prefix-Schutz):** Radix-Trie-Knoten mit `ref_count.load(Ordering::Acquire) > 1` sind geschützt vor Eviction.
- **Phase 2 (SnapKV Attention-Pool):** Evaluierung des Beobachtungsfensters $W_{obs}$:
  $$S_i = \frac{1}{|W_{obs}|} \sum_{q \in W_{obs}} A_{q, i}$$
- **Phase 3 (Tier-2 Spillover):** Nicht geschützte Segmente unterhalb der Attention-Schwelle werden asynchron mit KIVI auf 2-Bit quantisiert und per `mmap` auf NVMe ausgelagert.

#### 6.3.2 KIVI 2-Bit Asymmetrische Quantisierung
In `crates/contextra-kvcache/src/quantize_kivi.rs`:
- **Key-Tensoren:** Quantisierung pro Kanal (Channel-Dimension), da Dimensionen stark unterschiedliche Wertebereiche aufweisen.
- **Value-Tensoren:** Quantisierung pro Token.
- **Unsafe-Insel:** `src/aligned_alloc.rs` (ADR-019) garantiert 64-Byte AVX-512/NEON Alignment mit vollständigen `// SAFETY:`-Invarianten.

---

## 7. EP5: Contextra Cognition — Adaptive Kognition, Sketched Bandits & Spectral SVD

### 7.1 Zweck & Geltungsbereich
Höhere kognitive Logik, kontinuierliche Systemanpassung unter Latenzgarantien und topologische Deduplizierung von Hypergraphen.

### 7.2 Exklusive Crate-Verantwortlichkeit
- `contextra-cognition` (Ring 2)
- `contextra-adapt` (Ring 2)
- `contextra-agent` (Ring 4)

### 7.3 Mikrofeingranulare Spezifikation

#### 7.3.1 Sketched Contextual Bandits (LinUCB mit Random Projection)
In `crates/contextra-adapt/src/bandit.rs`:
- Reduktion des Sherman-Morrison-Updateaufwands von $\mathcal{O}(d^2)$ auf $\mathcal{O}(k^2)$ mit $k = \mathcal{O}(\sqrt{d} \log d)$.
- Projektionsmatrix $S \in \mathbb{R}^{k \times d}$ generiert über deterministische Achlioptas-Verteilung:
  $$S_{i, j} = \sqrt{\frac{3}{k}} \times \begin{cases} +1 & p = 1/6 \\ 0 & p = 2/3 \\ -1 & p = 1/6 \end{cases}$$
- Seed der Projektionsmatrix wird zwingend über den `Rng`-Port (`contextra-ports::rng::Rng`) injiziert (P28-Konformität).

#### 7.3.2 PID Latency Controller Determinismus (Behebung Audit-Befund)
In `crates/contextra-adapt/src/pid_latency_controller.rs`:
- Ersetzen aller direkten `Instant::now()` Aufrufe durch den `Clock`-Port (`contextra-ports::clock::Clock`).

#### 7.3.3 Spectral Hyperedge Deduplication (Brand's Incremental SVD)
In `crates/contextra-cognition/src/spectral_dedup.rs`:
- Inkrementelle Rang-1-Updates der Hypergraphen-Inzidenzmatrix $H$.
- Kosinus-Ähnlichkeit im Spektralraum: Bei $\cos(\mathbf{v}_i, \mathbf{v}_j) > \theta$ wird die Hyperkante als Duplikat markiert und mit `SuperEdgeSink` aggregiert.

---

## 8. EP6: Contextra Guard — Egress Firewall, MCP-Sicherheit & Sandbox-Isolation

### 8.1 Zweck & Geltungsbereich
Schutz vor Prompt-Injection, Exfiltration sensibler Daten, sichere MCP-Toolausführung und WASM-Plugin-Sandboxing.

### 8.2 Exklusive Crate-Verantwortlichkeit
- `contextra-privacy` (Ring 2)
- `contextra-sandbox` (Ring 3)
- `contextra-mcp` (Ring 3)
- `contextra-py` (Ring 4)

### 8.3 Mikrofeingranulare Spezifikation

#### 8.3.1 Behebung B1: Egress Guard Pflichtprüfung
In `crates/contextra-mcp/src/server.rs`:
```rust
// Vor Ausführung von Cloud-Queries MUSS der EgressGuard aktiv sein
if tool_name == "contextra_cloud_query" {
    let guard = self.egress_guard.as_ref().ok_or_else(|| {
        ContextraError::security_violation("Cloud query rejected: EgressGuard is not active")
    })?;
    guard.validate_outbound_payload(&payload)?;
}
```

#### 8.3.2 Hash-Chained Egress Audit Trail (INV-EGRESS-AUDIT-1)
In `crates/contextra-privacy/src/audit_trace.rs`:
```rust
pub struct EgressTraceEntry {
    pub prev_hash: [u8; 32],
    pub tenant_id: TenantId,
    pub sanitized_payload_hash: [u8; 32],
    pub classification: SensitivityLevel,
    pub timestamp_utc: u64,
}

impl EgressTraceEntry {
    pub fn compute_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.prev_hash);
        hasher.update(&self.tenant_id.0.to_le_bytes());
        hasher.update(&self.sanitized_payload_hash);
        hasher.update(&[self.classification as u8]);
        hasher.update(&self.timestamp_utc.to_le_bytes());
        *hasher.finalize().as_bytes()
    }
}
```

---

## 9. Workspace-weite Verdrahtung, Invarianten & Gate-Durchsetzung

### 9.1 Verbindliche Invarianten-Tabelle

| Invariante | Regel | Verifizierendes Gate |
|---|---|---|
| **P2** | 0 `unwrap()`, 0 `expect()`, 0 unkontrollierte Panics in Produktion | `cargo xtask doctrine-scan` |
| **P5** | Strenger Ring-DAG: Ring $N$ darf nur Ring $< N$ referenzieren | `cargo xtask dependency-graph-audit` |
| **P12** | `capabilities.toml` ist die alleinige Quelle der Wahrheit | `cargo xtask check-ring-capabilities-consistency` |
| **P26** | Ring 0 & Ring 1 enthalten kein `tokio` in `[dependencies]` | `cargo xtask check-ring0-async-purity` |
| **P28** | Zeit, Zufall und IDs ausschließlich über Ports injiziert | `cargo xtask determinism-check` |
| **INV-TENANT-1** | `TenantId(0)` = SYSTEM, direkte Feld-Instanziierung verboten | `cargo test -p contextra-types` |
| **INV-DURABILITY-RING**| `MemoryOnly` ist inkompatibel mit `ErasureMode::Assured` | `cargo test -p contextra` |
| **INV-MVCC-SSI-1** | Write-Skew-Schutz durch lückenloses Read-Set-Tracking | `cargo test -p contextra-mvcc` |

### 9.2 Akzeptanzkriterien für die Implementierung
1. `cargo xtask harness-list` bestätigt alle Ausführbarkeiten.
2. `cargo xtask find-unwired-gaps` zeigt 0 unverdrahtete Produktions-Traits.
3. `cargo xtask doctrine-scan` liefert Exit-Code 0.
4. Alle Tests in `crates/contextra-types/tests/` (inkl. `p04_txid_origin.rs`) kompilieren fehlerfrei.
5. End-to-End-Löschtest über `TenantScopedStorage` bestätigt: Datensätze werden physisch aus SSTables getilgt und ein valider `DeletionProofV4` wird über MCP/Fassade exportiert.

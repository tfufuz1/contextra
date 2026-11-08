# Baustein-Dokumentation & Benchmark-Profile

**Geltungsbereich:** Zentrale, autarke Bausteine des Contextra-Baukastens
**Stand:** 2026-11

---

## 1. Baustein: `contextra-simd`

### Zweck & Einsatzbereich
Bereitstellung hardwarebeschleunigter Vektordistanz-Kernels für Vektorsuche, Clustering und Ähnlichkeitsvergleiche in LLM-Embeddings und Robotik-Sensorströmen.
Automatische Laufzeiterkennung von AVX-512 VNNI, AVX2 und ARM NEON mit sicherem Skalar-Fallback.

### Öffentliche API
```rust
use contextra_simd::{compute_distance, validate_vector, Metric};

let a = vec![1.0, 0.0, 0.0];
let b = vec![0.0, 1.0, 0.0];
validate_vector(&a)?;
let dist = compute_distance(&a, &b, Metric::Cosine)?;
```

### Feature-Flags & Abhängigkeiten
- **Abhängigkeiten:** `bytemuck` (Slice-Casting), `thiserror`, `tracing`.
- **Zero-Panic-Garantie:** Alle Eingaben werden auf `NaN` und `Infinity` geprüft; bei Ungültigkeit wird `Err(ContextraError::InvalidInput)` zurückgegeben.

### Benchmark & Performance
- **Throughput:** > 1.2 GB/s Vektordaten auf AVX2/AVX-512.
- **Latenz:** < 40 ns für 768-dimensionale Cosine-Distanz.
- **Speicherverbrauch:** 0 Bytes Heap-Allokation während der Distanzberechnung.

---

## 2. Baustein: `contextra-rank`

### Zweck & Einsatzbereich
Multi-Signal-Fusion (RRF / Reciprocal Rank Fusion, DiBud-Streaming) und statistische Konfidenz-Kalibrierung (Platt Scaling, Isotonische Regression / PAVA, Conformal Prediction).
Ideal zur Kombination von Vektor- und Volltext-Suchergebnissen in RAG-Pipelines.

### Öffentliche API
```rust
use contextra_rank::rrf_fuse;
// Fusioniert zwei oder mehr Ergebnislisten deterministisch mit Smoothing K = 60
let fused = rrf_fuse(&[&list_vector, &list_bm25], 10)?;
```

### Feature-Flags & Abhängigkeiten
- **Abhängigkeiten:** `serde`, `ahash`, `thiserror`.
- **Garantien:** Deterministische Tie-Breaking-Sortierung; NaN/Inf-Schutz.

### Benchmark & Performance
- **Throughput:** > 500.000 Dokumente/s RRF-Fusion.
- **Latenz:** < 15 µs für Top-100 Fusion zweier 1.000-Elemente Listen.

---

## 3. Baustein: `contextra-sandbox`

### Zweck & Einsatzbereich
Wasmtime-basierte isolierte Ausführungsumgebung für von LLMs generierten Code (WASM / WASI).
Schützt Host-Systeme vor unsicheren Datei- und Netzwerkzugriffen.

### Öffentliche API
```rust
use contextra_sandbox::{WasmSandbox, SandboxConfig};

let config = SandboxConfig::default().with_max_memory_mb(64);
let sandbox = WasmSandbox::new(config)?;
let output = sandbox.execute_module(&wasm_bytes, &args)?;
```

### Feature-Flags & Abhängigkeiten
- **Abhängigkeiten:** `wasmtime` (LTS 36.x).
- **Sicherheit:** Streng limitierte Table- und Speichergrößen; konfigurierbare Timeout-Schranken.

---

## 4. Baustein: `contextra-wire`

### Zweck & Einsatzbereich
Zero-Copy-Framing und gehärteter JSON-RPC 2.0 / FlatBuffers Serializer/Deserializer für Inter-Process-Communication (IPC) und Tool-Aufrufe.

### Öffentliche API
```rust
use contextra_wire::jsonrpc::{parse_request, JsonRpcRequest};

let req = parse_request(payload_bytes)?;
```

### Garantien & Schutzmaßnahmen
- Schutz vor JSON-Tiefenbomben (Rekursionslimit 32).
- Batch-Größenbegrenzung (max 100 Anfragen pro Batch).
- Schutz vor doppelten Schlüsseln (`duplicate_key_rejected`).

---

## 5. Baustein: `contextra-durable-fs`

### Zweck & Einsatzbereich
Crash-konsistentes, atomares Virtual File System (Vfs) mit Temp-Rename-Sync und Fsync-Poisoning-Erkennung (`INV-FSYNC-1`).
Ermöglicht verlässliche Datenspeicherung in instabilen Umgebungen (Edge, Robotik, mobile Knoten).

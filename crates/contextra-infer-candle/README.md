# contextra-infer-candle

Candle LLM Inference Client und KV-Bridge Adapter für Contextra (Ring 2).

Normativitätsklasse: **Rand `[R]`** (Ring 2, `stable`; §6.0, `capabilities.toml`).

`contextra-infer-candle` stellt ein vollständig autarkes, lokales Inferenz- und Embedding-Backend auf Basis von Candle (`candle-core`, `candle-transformers`) für native GGUF- und Safetensors-Modelle bereit.

## Baukasten-Profil & Architekturrolle

- **Ring:** Ring 2 (Native Inferenz & Embeddings)
- **Klasse:** Erweiterung / Rand `[R]`
- **Unsafe-Code:** `#![forbid(unsafe_code)]` (0 unsafe Blöcke)
- **Abhängigkeiten:** `contextra-types`, `contextra-ports`, `contextra-rank` (strikt abwärts gemäß `capabilities.toml`). Keine direkte Kopplung an Speichermonolithen oder `contextra-crypto`.
- **KV-Cache Anbindung:** Tier-2 Spill & Persistence vollständig entkoppelt über den Port `contextra_ports::KvBridgeStorage`.
- **Modell-I/O & Umwelt:** VFS-Injektion (`contextra_ports::Vfs`) für deterministische, entkoppelte Dateizugriffe.

## Features

- `candle`: Inferenz-Unterstützung via Candle.
- `kv-bridge`: Aktiviert die KV-Cache-Bridge (`KvBridgeAdapter`) mit Fail-Open-Semantik.
- `kv-stage-b`: Aktiviert Prefix-Reuse und Schicht-Zustands-Management.
- `cuda`: Optionale CUDA-Beschleunigung für Candle-Kernels.
- `test-utils`: Test-Hilfsfunktionen.

## Garantien & Invarianten

1. **Async Thread Safety:** Heavy CPU-Tensor-Berechnungen laufen strikt in isolierten Thread-Pools (`tokio::task::spawn_blocking`), um den Tokio-Async-Reactor nicht zu blockieren.
2. **Zero-Panic-Doktrin:** Im Produktivcode existieren keine `unwrap()` oder `expect()`-Aufrufe; alle Fehlerzustände werden sauber über `ContextraError` abgebildet.
3. **Fail-Open Fallback:** Cache-Misses oder Format-Inkompatibilitäten in der KV-Bridge lösen transparent einen Fallback auf den vollständigen Prefill aus, ohne Anfragen abzubrechen.
4. **Offline-First:** Keinerlei Netzwerk-Zugriffe oder Telemetrie.

## Minimalbeispiel (Eigenständiger Einsatz)

Ein vollständiges, autark lauffähiges Beispiel findet sich unter `examples/standalone_embedding.rs`:

```bash
cargo run --example standalone_embedding -p contextra-infer-candle
```

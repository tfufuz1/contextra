# contextra-kvcache

## Zweck
Prefix-Cache-Verwaltung für KV-Segmente; keine Anbindung an eine konkrete Inferenz-Runtime im Repo.
`contextra-kvcache` stellt deterministische Datenstrukturen (Radix-Baum, Segment-Store, LRU-/Attention-Eviction, KIVI-Quantisierung) zur Verwaltung und Wiederverwendung von Key-Value-Cache-Segmenten bei Prefill- und Decode-Workflows bereit.

## Reifegrad
`beta` (Eigenständiger Baustein für KV-Prefix-Management; als `frozen`-Kandidat für Workspace-Default-Members gemeldet, da die Entflechtung von `crypto-compat` upstream in Vorbereitung ist).

## Interne Abhängigkeiten
- `contextra-types` (optional über Feature `ports-compat`)
- `contextra-ports` (optional über Feature `ports-compat`)
- `contextra-crypto` (optional über Feature `crypto-compat` / `kv-encryption`)

Hinweis: Es bestehen keine Abhängigkeiten oder Importpfade zu Inferenz-Runtimes (`contextra-infer-*`) im Repository [V].

## Features
- `default = ["std", "ports-compat", "crypto-compat", "kv-encryption"]`
- `std`: Standardbibliotheksunterstützung.
- `ports-compat`: Anbindung an `contextra-ports` und `contextra-types`.
- `crypto-compat`: Optionale Anbindung an `contextra-crypto` für Segmentverschlüsselung.
- `kv-encryption`: Kryptografisches Sealing und Segmentverschlüsselung.
- `content-addressed-kv-cache`: Inhaltsadressierte KV-Blöcke über Hash-Präfixe.
- `kvcache-attention-eviction`: Attention-basierte Verdrängungsstrategien.
- `kivi-quantization` / `kvcache-kivi-quant`: KIVI-Quantisierung für KV-Tensoren.

## Minimalbeispiel

```rust
use contextra_kvcache::{KvSegment, PrefixRadixTree, TenantIsolatedKvStore};
use contextra_types::TenantId;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tenant = TenantId::try_new(42)?;
    let store = TenantIsolatedKvStore::new();

    // Segment isoliert einfügen
    let segment = KvSegment::new(tenant, 1, vec![0x11; 128]);
    store.insert_segment(tenant, segment);

    // Segment wieder abrufen
    let data = store.get_segment_bytes(tenant, 1);
    assert!(data.is_some());

    Ok(())
}
```

## Entnahme-Rezept
Zur Verwendung als eigenständiger Baustein in externen Projekten:
1. Verzeichnis `crates/contextra-kvcache` kopieren.
2. Crate-Abhängigkeiten `contextra-types`, `contextra-ports` und `contextra-crypto` durch eigene Typdefinitionen ersetzen oder über die Features konfigurieren.
3. `Cargo.toml` anpassen und Features wie gewünscht aktivieren.

## Bekannte Grenzen
- **Keine Inferenz-Runtime:** Enthält keine Anbindung an LLM-Laufzeitumgebungen wie Candle, ONNX Runtime oder Ollama.
- **Kopplung an Krypto-Schicht:** Die Segment-Verschlüsselung stützt sich in den Standard-Features auf `contextra-crypto`.
- **Async Runtime:** Benötigt für asynchrone Speicher-Adapter (`StorageEngineKvBridge`) eine Tokio-Runtime-Umgebung.

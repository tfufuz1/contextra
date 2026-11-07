# contextra-kvcache

> **Ring 1** · Klasse `[K]` (Kern) · Reifegrad `beta` · Unsafe: **nein** (`#![forbid(unsafe_code)]`) · Rolle: Verschlüsselter, mandantenisolierter KV-Prefix-Cache für LLM-Zustände (Radix-Baum, Attention-Eviction, KIVI-Quantisierung).

`contextra-kvcache` ist ein eigenständiger, modularer Baukasten für modernes Key-Value-Cache-Management bei LLM-Inferenz und RAG-Workloads. Er beschleunigt Prefill-Phasen durch deterministisches Teilen von Prompt-Präfixen und bietet Tenant-Isolierung sowie verschlüsseltes Tiering.

## 1. Baukasten-Profil & Entkopplung

- **Keine juristische Löschkettenlogik:** Bereinigung erfolgt als neutrale In-Memory-Freigabe, Verdrängung (Eviction) oder Zeroization.
- **Fail-Closed Verschlüsselung:** Segment-Verschlüsselung mit deterministischer AAD-Bindung (Segment-ID, Tenant, ModelFingerprint) und Schlüsselableitungs-Versionsprüfung.
- **Zero Shared Mutable State:** Saubere Sharding- und Lock-Hierarchie ohne globale Singletons.

## 2. Abhängigkeiten

Gemäß `capabilities.toml` und Ring-1-Vorgaben hängt `contextra-kvcache` ausschließlich an:
- `contextra-types` (Ring 0)
- `contextra-ports` (Ring 0)
- `contextra-crypto` (Ring 0)

Keine Abhängigkeiten zu höheren Ringen oder Inferenz-Frameworks (`candle`, `onnx`, `ollama`).

## 3. Features

- `default = []`: Minimaler Default-Footprint.
- `kv-encryption`: Aktiviert erweiterte kryptografische Bindung via `contextra-crypto`.
- `content-addressed-kv-cache`: Inhaltsadressierte KV-Blöcke.
- `kivi-quantization` / `kvcache-kivi-quant`: 2-Bit/4-Bit KIVI-Quantisierung für KV-Tensoren.
- `kvcache-attention-eviction`: Attention-basierte Verdrängungspolicies.

## 4. Garantien & Invarianten

1. **Mandantentrennung (VETO-F10):** Jede Operation wird strikt über `TenantId` isoliert. Cross-Tenant Cache-Sharing ist unmöglich.
2. **Speichersicherheit:** `#![forbid(unsafe_code)]` im gesamten Crate.
3. **Zero-Panic:** Keine unwrap/expect/panic! im Produktivcode.
4. **Fail-Closed:** Mismatches bei AAD, Schlüsselversionen oder fehlerhafte Entschlüsselung führen deterministisch zu Fehlern.

## 5. Einbettungsbeispiel

```rust
use contextra_kvcache::{KvCacheConfig, KvCacheServiceBuilder, KvSegment, TenantIsolatedKvStore};
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

Ein ausführliches Standalone-Beispiel findet sich unter `examples/standalone_prefix_cache.rs`.

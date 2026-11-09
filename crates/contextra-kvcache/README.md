# contextra-kvcache

> Klasse `A` (Stand-alone Baukasten) · Unsafe: **nein** (`#![forbid(unsafe_code)]`) · Rolle: Verschlüsselter, mandantenisolierter KV-Prefix-Cache für LLM-Inferenzzustände (Radix-Baum, Attention-Eviction, KIVI-Quantisierung).

`contextra-kvcache` ist ein eigenständiger, hochperformanter Baukasten für Key-Value-Cache-Management bei LLM-Inferenz und Agentic RAG Workloads. Er beschleunigt Prefill-Phasen durch deterministisches Teilen von Prompt-Präfixen und bietet vollständige Mandanten-Isolierung, neutrale Segment-Schlüsselzerstörung sowie optionales verschlüsseltes Tiering.

## 1. Zweck und Baukasten-Prinzip (Klasse A)

- **Vollständig autark:** Keine zwingenden internen Abhängigkeiten. Baut und testet mit `--no-default-features` komplett isoliert.
- **Copy-Paste-Readiness:** Kann via `python3 tools/export_crate.py contextra-kvcache` als eigenständige Library exportiert werden.
- **Keine juristische Löschkettenlogik (SPEC §6):** Bereinigung erfolgt ausschließlich als neutrale Invalidation, Verdrängung (Eviction) oder kryptografische Schlüsselzerstörung (`SegmentKey::destroy()`). Löschbeweise und DSGVO-Orchestrierung liegen in `contextra-erasure-adapters`.
- **Eigene Trait-Schnittstelle:** Segmentverschlüsselung ist über den lokalen Trait `SegmentCipher` abstrahiert (inklusive `PlaintextSegmentCipher`).

## 2. Abhängigkeiten

### Externe Crates
- `thiserror`, `tracing`, `parking_lot`, `tokio` (minimal), `serde`, `bitvec`, `half`, `criterion` (dev-dep).

### Übergangs-Features (Transitional / Compat)
- `ports-compat`: Optionale Konvertierung zu `contextra-types` und Implementierung von Ports aus `contextra-ports`.
- `crypto-compat`: Optionale Anbindung an `contextra-crypto` für `SegmentCipher`.

## 3. Features

- `default = ["std", "ports-compat", "crypto-compat"]`
- `kv-encryption`: Aktiviert kryptografische Bindung und Segmentverschlüsselung.
- `content-addressed-kv-cache`: Inhaltsadressierte KV-Blöcke über Hash-Präfixe.
- `kivi-quantization` / `kvcache-kivi-quant`: 2-Bit/4-Bit KIVI-Quantisierung für KV-Tensoren.
- `kvcache-attention-eviction`: Attention-basierte Verdrängungspolicies (Sieve, LRU).

## 4. Garantien & Invarianten

1. **Mandantentrennung (INV-KV-1):** Jede Operation wird strikt über `TenantId` isoliert. Kein mandantenübergreifendes Sharing.
2. **Krypto-Invalidierung (INV-KV-2):** `SegmentKey::destroy()` überschreibt das Schlüsselmaterial sicher (`zeroize`).
3. **Zero-Panic (INV-KV-3):** Kein `unwrap()` / `expect()` im Produktivcode; alle Fehler sind als `KvCacheError` typisiert.
4. **Speichersicherheit:** `#![forbid(unsafe_code)]` garantiert vollständige Speichersicherheit.

## 5. Minimalbeispiel

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

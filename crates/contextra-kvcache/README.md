# contextra-kvcache

> Prefix-Radix-Baum, KV-Block-Cache, mandantengetrennter Speicher und Tiering fuer Contextra.

## Zweck
`contextra-kvcache` ist ein autarker, release-faehiger Baustein (Klasse A) fuer das Caching und die Wiederverwendung von Key-Value-Zustaenden (Attention-Matrizen) in LLM-Inferenz-Pipelines.
Er implementiert:
- Prefix-Radix-Baum zur schnellen Identifikation geteilter Token-Sequenzen (Prompt Caching).
- Mandantengetrennte In-Memory-LRU-Speicher mit konfigurierbarer Kapazitaet und Sharding.
- Zeroize-on-Drop und ShreddableSegmentKey fuer kryptografisches Shredden von KV-Segmenten.
- Attention-gestuetzte Verdrängungsstrategien (SnapKV / H2O) mit deterministischem Fallback.
- Optionale KIVI 2-Bit Quantisierung fuer maximale Speichereffizienz.

## Architektur & Abgrenzung
- Reifegrad: **R3** (release-faehig, autark).
- Keine juristischen Loeschketten- oder Compliance-Logiken im Speicherbaustein; Bereinigung erfolgt neutral ueber Eviction / Invalidation.
- Reale interne Kernel-Abhaengigkeiten:
  - `contextra-ids`: Domain-Identitaeten (`TenantId`).
  - `contextra-env`: Kanonische Umwelt-Abstraktionen (`Clock`, `SystemClock`).
  - `contextra-crypto` (optional hinter Feature `crypto-compat` / `kv-encryption`).
  - `contextra-ports`, `contextra-types` (optional hinter Feature `workspace`).

## Quickstart
```rust
use contextra_kvcache::contract::{KvCacheEngine, MemoryKvCache};
use contextra_kvcache::types::TenantId;
use contextra_kvcache::radix::KvReusePolicy;
use contextra_kvcache::segment::KvSegment;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cache = MemoryKvCache::new();
    let tenant = TenantId::try_new(1)?;

    let segment = KvSegment::new(tenant, 100, vec![0xAA, 0xBB, 0xCC, 0xDD]);
    cache.insert(tenant, segment)?;

    let data = cache.get(tenant, 100).expect("segment 100 exists");
    assert_eq!(data, vec![0xAA, 0xBB, 0xCC, 0xDD]);

    let _prefix = cache.find_longest_prefix(tenant, &[101, 102], KvReusePolicy::Always);
    Ok(())
}
```

## Feature-Tabelle

| Feature | Beschreibung |
|---|---|
| `std` | Standardbibliothek-Integration (Default). |
| `workspace` | Bindet `contextra-ports` und `contextra-types` ueber `src/compat/` ein. |
| `ports-compat` | Rueckwaertskompatibler Alias fuer `workspace`. |
| `types-compat` | Rueckwaertskompatibler Alias fuer `workspace`. |
| `crypto-compat` | Bindet `contextra-crypto` ein. |
| `kv-encryption` | Aktiviert AES-256-GCM-SIV Segment-Verschluesselung. |
| `kivi-quantization` | Optionale KIVI 2-Bit Block-Quantisierung. |
| `content-addressed-kv-cache` | Inhaltsadressierter Cache und semantisches Lookup. |
| `docid-128` | 128-Bit Dokumenten-IDs via `contextra-ids`. |

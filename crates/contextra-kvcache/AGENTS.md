# AGENTS.md — contextra-kvcache
> Ring 1 · stable · Quelle: capabilities.toml · Spec: K.15 / III.10 / L.3 / L.4

## 1. Zweck
Speicher- und Tiering-Subsystem für den LLM-KV-Cache mit Mandantenisolierung, Prefix-Radix-Bäumen, Attention-basierter Eviction und 2-Bit-KIVI-Quantisierung.
Verwaltet In-Memory-Segmente, stuft ausgelagerte Blöcke auf Tier-2-Dateien ab und sichert KV-Segmente per Crypto-Shredding ab.
Erzwingt `#![forbid(unsafe_code)]` im gesamten Crate.

## 2. Modul-Karte

| Datei | Verantwortung |
|---|---|
| `lib.rs` | Crate-Einstiegspunkt mit `#![forbid(unsafe_code)]`, Modul-Deklarationen und Re-Exports von Eviction-Funktionen |
| `attention_score.rs` | `AttentionScoreSource` Trait, `NullAttentionScoreSource` und Eviction-Ranking (LRU-Alter + Attention-Gewichtung) |
| `eviction_worker.rs` | `EvictionWorker` (Hintergrund-Eviction auf dediziertem OS-Thread) und `emergency_wipe()` (synchroner Notfall-Sicherheits-Purge) |
| `prefix_store.rs` | `TenantPrefixKvStore` (Mandantenisolierter Prefix-Store implementiert `contextra_ports::KvPrefixStore`) |
| `quantize_kivi.rs` | 2-Bit KIVI Quantisierung, Dequantisierung, Unpacking und verlustfreie Byte-Kompression |
| `radix.rs` | `PrefixRadixTree` (komprimierter Radix-Baum), `KvReusePolicy` und optionaler `ContentAddressedKvStore` (Feature `content-addressed-kv-cache`) |
| `scratchpad_invalidation.rs` | Invalidation-Hilfsroutinen für Arbeitsfenster |
| `segment.rs` | `KvSegment` mit `ZeroizeOnDrop`, `ShreddableSegmentKey` und `Tier2EncryptedSegment` für AEAD/Crypto-Shredding |
| `store.rs` | `TenantIsolatedKvStore` (sharded In-Memory KV-Segment-Store mit Pinnings, Rollback-Handhabung und Tier-2 Spill-Handler) |

## 3. Invarianten

- **Absolute Mandantenisolierung (VETO-F10 / G.3 / INV-TENANT)**: Ein Mandant kann KV-Einträge anderer Mandanten weder einsehen, abfragen noch verdrängen (`TenantPrefixKvStore`, `TenantIsolatedKvStore`).
  *Test*: `cargo test -p contextra-kvcache --lib`
- **Zeroize-on-Drop (P9)**: Schlüssel- und KV-Segmentdaten implementieren `Zeroize` und `ZeroizeOnDrop` und werden beim Verwerfen im Speicher genullt.
  *Test*: `cargo test -p contextra-kvcache --lib`
- **Hot-Path Eviction Entkopplung**: `EvictionWorker` verarbeitet Löschbefehle asynchron über einen dedicated OS-Thread, damit synchrone `Zeroize`-Operationen niemals den Tokio-Scheduler blockieren.
  *Test*: `cargo test -p contextra-kvcache --lib`
- **Notfall-Purge Vollständigkeit**: `emergency_wipe()` führt Notfall-Löschungen synchron und vollständig aus, bevor der Aufruf zurückkehrt.
  *Test*: `cargo test -p contextra-kvcache --lib`

## 4. Verboten / Anti-Patterns

- **Mandantenübergreifende Lookup-Pfade**: Keys dürfen NIEMALS ohne `TenantId`-Scope verarbeitet oder gematcht werden.
- **Verhaltendes Zeroize im async-Kontext**: Intensive Speicherbereinigungen dürfen NICHT direkt auf Tokio-Worker-Threads ausgeführt werden; stattdessen `EvictionWorker` nutzen.
- **Direktes Verwerfen von unverschlüsselten Tier-2 Segmenten**: Tier-2-Disk-Segmente müssen stets verschlüsselt (`Tier2EncryptedSegment`) und shredderbar sein.

## 5. Nebenläufigkeit, Async- und Lock-Regeln

- In-Memory KV-Store nutzt Sharding mit per-Shard `parking_lot::RwLock` für hohen Lese-Durchsatz.
- Locks werden nie über `.await`-Punkte gehalten.
- Der `EvictionWorker` kommuniziert über synchrone `std::sync::mpsc`-Kanäle und ist vom Tokio-Executor isoliert.

## 6. Verifikation

```bash
cargo test -p contextra-kvcache --locked
```

## 7. Bekannte Lücken / SOLL

- **KIVI-Quantisierung als Feature-Flag**: 2-Bit-Quantisierung in `quantize_kivi.rs` erfordert die Features `kivi-quantization` oder `kvcache-kivi-quant`.
- **Content-Addressed Cache Opt-in**: Multi-Level-Match (Position, Content-Hash, Semantik) ist hinter Feature `content-addressed-kv-cache` gekapselt.

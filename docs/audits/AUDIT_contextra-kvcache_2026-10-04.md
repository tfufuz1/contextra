# Audit-Bericht: `contextra-kvcache`

**Datum**: 2026-10-04
**Auditor**: Principal Senior Rust Architect (`Contextra Engine & Storage Core`)
**Crate**: `crates/contextra-kvcache` (Ring 1, `#![forbid(unsafe_code)]`)
**Claim / Ticket**: `contextra-kvcache-claim.log` (Issue: `kvcache-audit`)
**Session-Hash**: `27971878` | **Timestamp**: `2026-10-04T06:52:17Z`

---

## Executive Summary & System-Einordnung

Der Crate `contextra-kvcache` bildet das zentrale In-Memory/Tiering-Substrat für KV-Cache-Blöcke und Prefix-Radix-Bäume im Ring 1 von Contextra. Er regelt die speichereffiziente Wiederverwendung von LLM-Attention-KV-Zuständen (`PrefixRadixTree`, `TenantPrefixKvStore`), die 2-Bit-Quantisierung (`quantize_kivi.rs`), die asynchrone / notfallmäßige Speicher-Eviction (`eviction_worker.rs`, `attention_score.rs`), die Zeroize-on-Drop-Sicherheit (`segment.rs`) sowie das sharded In-Memory-Segment-Management (`store.rs`).

Alle Quellcodedateien im Crate erzwingen ausnahmslos `#![forbid(unsafe_code)]`. Es wurden keine `unsafe`-Blöcke oder rohe Zeigeroperationen identifiziert.

---

## PRÜFPUNKTE (P1 - P6)

### P1 TENANT-ISOLATION ABSOLUT (VETO-F10)
- **Status**: **BESTANDEN (Strukturell erzwungen)**
- **Analyse**:
  1. In `TenantPrefixKvStore` (`prefix_store.rs`) speichert die Partitionen-Map Einträge explizit unter dem Verbundschlüssel `(TenantId, PrefixKey)` (`RwLock<AHashMap<(TenantId, PrefixKey), Partition>>`). Ein Lookup nimmt zwingend `tenant: TenantId` als Argument entgegen und führt `map.get(&(tenant, key))` aus. Es existiert kein Lookup-Pfad, der tenant-übergreifend Schlüssel auflöst oder Daten aggregiert.
  2. In `TenantIsolatedKvStore` (`store.rs`) greift jede Shard-Operation über `(tenant.inner() as usize) & (shard_count - 1)` auf eine per-Tenant-Instanz (`TenantState`) zu. Lookups (`get_segment_bytes`, `find_prefix_match`, `get_segments`) erfordern stets `TenantId`.
  3. In `ContentAddressedKvStore` (`radix.rs`, `feature = "content-addressed-kv-cache"`) ist der `content_index` als `AHashMap<(TenantId, blake3::Hash), KvSegmentRef>` strukturiert. Lookups erfordern `(tenant, hash)` und enthalten ein zusätzliches Defense-in-Depth-Check (`if seg_ref.tenant_id == tenant`).
  4. In `scratchpad_invalidation.rs` filtert `invalidate_prefix_scope_keys` strikt auf `*t == tenant`.
- **Fazit**: Die Mandantenisolierung gemäß VETO-F10 ist nicht nur konzeptionell dokumentiert, sondern über Typsystem und Datenstrukturen strukturell garantiert.

### P2 RADIX-TREE-KORREKTHEIT
- **Status**: **BESTANDEN (Sehr hohe Testabdeckung)**
- **Analyse**:
  1. `test_radix_tree_node_splitting` und `test_v1_radix_split_correctness_10k_keys` in `radix.rs` testen intensiv das Einfügen gemeinsamer Präfixe, Knoten-Splitting und die Wiederverwendbarkeit über 10.000 generierte Schlüssel mit 100 % Hit-Rate.
  2. `test_radix_tree_remove_and_clear` prüft das rückstandsfreie Entfernen von Knoten ohne Beeinträchtigung anderer Baum-Pfade.
  3. Integrationstests in `tests/prefix_store_port.rs` (`test_longest_prefix_wins`, `test_tenant_isolation`) verifizieren die Traversierung sowie die exakte Rückgabe gematchter Blöcke.

### P3 KIVI-QUANTISIERUNG (`quantize_kivi.rs`)
- **Status**: **BESTANDEN (Sicher mit Guard-Checks)**
- **Analyse**:
  1. **Overflow / Truncation**: Integer-Arithmetik für Packing (`byte_idx`, `bit_shift`) nutzt sichere Division/Modulo (`orig_idx / 4`, `(orig_idx % 4) * 2`). `div_ceil` verhindert Puffer-Unterdimensionierungen.
  2. **Range / Value Checking**: Sowohl `kivi_quantize` als auch `kivi_dequantize` prüfen explizit auf `is_finite()` für Scales, Zero-Points und Tensor-Floats. Bei `NaN` oder `f32::INFINITY` greift eine sichere Fallback-Behandlung (`min_v = 0.0`, `max_v = 0.0`) bzw. Abbruch mit `ContextraError::kv_quantization`.
  3. **End-to-End Test**: `test_v3_kivi_quantization_cosine_similarity_reconstruction` verifiziert eine Cosinus-Ahnlichkeit von > 0.99 bei der Rekonstruktion von 2-Bit-KIVI-Blöcken.

### P4 EVICTION-WORKER-KORREKTHEIT (`eviction_worker.rs`, `attention_score.rs`)
- **Status**: **BESTANDEN (Keine Async-Blockierung, RAII-geschützt)**
- **Analyse**:
  1. **Strategie**: Bietet kombinierte LRU- und Attention-Score-Eviction (`rank_for_eviction_weighted_for_tenant`). Bei `NullAttentionScoreSource` fällt das System nahtlos auf reines LRU zurück.
  2. **Thread-Isolation**: `EvictionWorker` verarbeitet Eviction-Commands auf einem dedizierten OS-Thread (`std::sync::mpsc`), um synchrone `Zeroize`-Drop-Kaskaden vom Tokio-Async-Executor fernzuhalten.
  3. **TOCTOU / Race Protection**: In `TenantState::pop_eviction_candidate` werden nur Kandidaten mit `active_refs == 0` und abgelaufenem PIN-TTL berücksichtigt. Der Test `test_store_guarded_segments_never_force_evicted_on_overflow` weist nach, dass ein aktiver `KvBlockGuard` das Segment garantiert vor Eviction schützt, selbst wenn die Kapazität überschritten wird.

### P5 AEAD-VERSCHLÜSSELUNG (`segment.rs`)
- **Status**: **BESTANDEN (Zeroize & Crypto-Shredding integriert)**
- **Analyse**:
  1. `KvSegment` erzwingt `#[derive(Zeroize, ZeroizeOnDrop)]` über den rohen Tensor-Speicher `data: Vec<u8>`. Sensitive Daten werden bei Freigabe unverzüglich gezeroized.
  2. Bei aktivem `feature = "kv-encryption"` ist die Verschlüsselung über `contextra-crypto` (`EncryptedKvLayer`, `KvSegmentCipher`) mit AES-256-GCM-SIV integriert.
  3. Für ausgelagerte Tier-2 Segmente stellt `ShreddableSegmentKey` und `Tier2EncryptedSegment` mathematisches Crypto-Shredding bereit: Ein Aufruf von `key.shred()` vernichtet den In-Memory-Key via `emergency_wipe()`, wodurch auf Disk befindliche Segmentdateien augenblicklich und unumkehrbar unlesbar werden.

### P6 TIERING-KORREKTHEIT & CONCURRENCY
- **Status**: **BESTANDEN (Locking schützt Konsistenz)**
- **Analyse**:
  1. Ein Auslagern von Hot-Tier zu Cold-Tier bzw. Spill-Handler erfolgt in `TenantIsolatedKvStore` atomar unter dem Schreib-Lock der Shard-Partition (`shard.lock.write()`).
  2. Bei einem parallelen Read sperrt `get_segment_bytes` bzw. `find_prefix_match` die entsprechende Shard. Erst wenn das Segment im Schreibpfad vollständig evictiert und dem Spill-Handler übergeben wurde, wird der Schreib-Lock freigegeben. Ein "stiller Not-Found"-Zustand durch ungeordnete Concurrent-Reads tritt nicht auf, da die Zustandstransition unter der Shard-Sperre geschützt ist.

---

## VERDICT & EVIDENCE

```markdown
<!-- VERDICT: APPROVED WITH CONDITIONS / GO FOR PRODUCTIVE AUDIT -->
```

### EVIDENCE MARKERS

- **EVIDENCE-F10-TENANT-ISOLATION**: `TenantPrefixKvStore` in `prefix_store.rs` (Zeilen 63-70) und `TenantIsolatedKvStore` in `store.rs` (Zeilen 267-270) erzwingen `TenantId` als Primärschlüssel-Komponente aller Zugriffe.
- **EVIDENCE-FORBID-UNSAFE**: `lib.rs`, `quantize_kivi.rs` und `attention_score.rs` deklarieren explizit `#![forbid(unsafe_code)]`.
- **EVIDENCE-ZEROIZE-ON-DROP**: `KvSegment` in `segment.rs` (Zeile 171) implementiert `#[derive(Zeroize, ZeroizeOnDrop)]`.
- **EVIDENCE-KIVI-COSINE**: `quantize_kivi.rs` (Zeilen 767-778) verifiziert Cosinus-Ähnlichkeit > 0.99 nach 2-Bit-Dequantisierung.
- **EVIDENCE-GUARD-EVICTION-PROTECTION**: `test_store_guarded_segments_never_force_evicted_on_overflow` in `store.rs` (Zeile 581) belegt den Schutz aktiver Guards gegen Speicher-Eviction.

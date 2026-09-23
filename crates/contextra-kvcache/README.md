# contextra-kvcache

`contextra-kvcache` stellt den segmentierten KV-Cache und Eviction-Worker bereit (Ring 1).

## Zweck

Verwaltet In-Memory LRU-KV-Caches, Tenant-Isolation, Verschlüsselung von KV-Segmenten und kontrolliertes Auslagern (Tiering) zur Vermeidung von Prefill-Rechenzeit.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 1 (Persistenz & Caching)
- **Status:** 🟢 Fertig
- **Sicherheits-Invariante:** Safe API

## Öffentliche API-Übersicht

- **KV Store:** `TenantIsolatedKvStore`, `SpillHandler`
- **Segments & Eviction:** `KvSegment`, `EvictionWorker`, `emergency_wipe`

## Architektur & Verweise

Details zur KV-Cache v2 Zielarchitektur finden sich in [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) sowie `README.md` §9.2.

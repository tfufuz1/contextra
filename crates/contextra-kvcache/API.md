# API von contextra-kvcache

Übersicht der öffentlichen Module, Typen und Schnittstellen von `contextra-kvcache`.

## Module

- `radix`: Deterministischer Prefix-Radix-Baum für KV-Cache-Wiederverwendung und Token-Präfix-Matching.
- `store`: Mandantenisolierter Speicher (`TenantIsolatedKvStore`) mit Sharding, Segment-Speicherung und LRU-Verdrängung.
- `segment`: Struktur und Typen für `KvSegment`, Segment-Handles und verschlüsselte Segment-Payloads.
- `service`: High-Level Service-Fassade (`KvCacheService`, `KvCacheServiceBuilder`) für mandantensichere Cache-Interaktionen.
- `attention_score`: Schnittstellen für Attention-basierte Score-Quellen und Verdrängungs-Sortierung.
- `quantize_kivi`: KIVI-Quantisierungs- und Dequantisierungs-Routinen für KV-Segmente.
- `adapters`: Port-Adapter für Persistenz (`StorageEngineKvBridge`), Embeddings (`EmbeddingProviderSemanticEmbedder`) und Plugin-Manifeste (`KvCachePluginManifest`).
- `error`: Fehlertypen (`KvCacheError`) und Typdefinitionen.
- `compat`: Typ-Konvertierungen und Kompatibilitätsdefinitionen für `contextra-types` und `contextra-ports`.

## Haupttypen

- `TenantIsolatedKvStore`: Kernkomponente zur segmentbasierten In-Memory-Speicherung von KV-Caches.
- `PrefixRadixTree`: Radix-Baum zur schnellen Identifikation geteilter Token-Präfixe.
- `KvCacheService`: Service-Schicht mit Lebenszyklus-Management, Speicherbudgetierung und Verdrängungssteuerung.
- `KvSegment`: Repräsentiert ein einzelnes KV-Cache-Segment eines Mandanten.
- `StorageEngineKvBridge`: Adapter zur Persistierung ausgelagerter Segmente über die generische Storage-Engine-Schnittstelle.
- `KvCachePruner`: Port-Implementierung zur Freigabe und Bereinigung von Segmentspeicher.
- `KvCacheScanner`: Port-Implementierung zur Restmengen-Messung im KV-Cache.

# AGENTS.md — `contextra-infer-candle`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-infer-candle` stellt ein lokales, vollständig offline-fähiges Inferenz- und Embedding-Backend auf Basis von Candle (`candle-core`, `candle-transformers`) für native GGUF-Modelle bereit (Ring 2).
Als autarker Baustein implementiert er `EmbeddingProvider` aus `contextra-ports` und ermöglicht vollständige Datenhoheit ohne externe Dienste oder Netzwerkanfragen.
Er integriert sich über standardisierte Ports und stellt optionale KV-Cache-Anbindungen (`contextra-kvcache`) über Entkopplungs-Adapter bereit.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 2 (Native Inferenz & Embeddings)
- **Klasse:** Erweiterung `[E]`
- **Reifegrad:** `beta`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports`, `contextra-rank`, `contextra-crypto` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **Async Thread Safety:** Candle-Tensor-Operationen sind CPU-blockierend. Alle Inferenz- und Embed-Aufrufe MÜSSEN strikt via `tokio::task::spawn_blocking` ausgeführt werden, um den async Reactor nicht zu blockieren.
- **Zero-Panic-Doktrin:** Vollständiger Ausschluss von `unwrap()` und `expect()` im Produktionscode. Alle Fehler werden als `ContextraError` abgebildet.
- **Deterministische Batch-Größen:** Einhaltung von `MAX_CANDLE_EMBED_BATCH_SIZE` zur Vermeidung von Out-of-Memory (OOM) Fehlern.
- **Offline-First:** Keinerlei Netzwerk-Egress; Modelle werden lokal aus GGUF-Dateien geladen.

## 4. Verbotene Muster
- Blockierende Tensor-Berechnungen direkt im asynchronen Tokio-Kontext.
- Verwenden von `contextra-core` (statdessen direkte Nutzung von `contextra-types` und `contextra-ports`).
- Abhängigkeiten zu höheren Schichten wie `contextra-mcp` oder `contextra-engine`.
- Unkontrollierte GPU/CPU-Allokationen ohne Backpressure-Semaphor.

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Implementiert:** `EmbeddingProvider`.
- **Konsumiert:** `contextra-types`, `contextra-ports`.
- **Optionale Integration:** KV-Bridge zu `contextra-kvcache` via Feature-Gate `kv-bridge`.

## 6. Tests, Sabotage & Qualitätssicherung
- **Model-Fingerprint-Validierung:** Integritätstests gegen fehlerhafte oder manipulierte GGUF-Modelldateien.
- **Concurrency & Backpressure Tests:** Semaphore-Tests zur Verhinderung von Überlastung bei parallelen Inferenz-Anfragen.
- **Sabotage-Testing:** Prüfung der Fehlerbehandlung bei abgeschnittenen oder inkompatiblen Tensor-Puffern.

## 7. Offene Punkte & Technische Schulden
- Keine.

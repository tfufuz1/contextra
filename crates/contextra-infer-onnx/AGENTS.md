# AGENTS.md — `contextra-infer-onnx`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-infer-onnx` stellt vollständig lokale, plattformübergreifende Vektor-Einbettungen und Cross-Encoder Reranking auf Basis der ONNX-Runtime (`ort`) bereit (Ring 2).
Als autarker Baustein implementiert er `EmbeddingProvider` aus `contextra-ports`.
Das Modul arbeitet strikt Offline-First (Modelldownloads standardmäßig deaktiviert), erzwingt kryptografische SHA-256-Prüfungen und entkoppelt maschinelles Lernen von externen Server-Infrastrukturen.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 2 (ONNX Inferenz & Reranking)
- **Klasse:** Erweiterung `[E]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports`, `contextra-rank` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **Offline-First & Checksummen-Pflicht:** Downloads standardmäßig blockiert (`allow_download = false`). Wenn aktiviert, nur über HTTPS mit Host-Allowlist, strikter SHA-256-Hash-Prüfung und Größenbeschränkung.
- **Nicht-blockierendes I/O:** Datei-I/O und Modell-Ladevorgänge laufen über `tokio::task::spawn_blocking` mit atomarem Tempfile-Staging und `fsync`.
- **Zero-Panic-Doktrin:** Vollständiger Ausschluss von `unwrap()` und `expect()`; robuste Validierung von Sequenzlängen, Batch-Größen und Thread-Limits.
- **Fail-Closed Eingabevalidierung:** Rechnen auf fehlerhaften Tensordimensionen wird vorab abgefangen.

## 4. Verbotene Muster
- Direkte blockierende ONNX-Runtime C-Aufrufe im asynchronen Tokio-Event-Loop.
- Verwendung von `contextra-core` (statdessen direkte Nutzung von `contextra-types` und `contextra-ports`).
- Ungeprüftes Laden beliebiger `.onnx`-Dateien ohne Validierung.
- Abhängigkeiten zu höheren Orchestrierungsschichten.

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Implementiert:** `EmbeddingProvider`, Cross-Encoder Reranker.
- **Konsumiert:** `contextra-types`, `contextra-ports`, `contextra-rank`.
- **Bereitgestellt für:** `contextra-engine`, `contextra-db`, `contextra-mcp`.

## 6. Tests, Sabotage & Qualitätssicherung
- **Model-Integrity Tests:** Verifikation der Hash-Validierung und Schutz gegen beschädigte Modell-Dateien.
- **Batching & Latenz-Benchmarks:** Empirische Messung von Throughput und Speicherauslastung.
- **Sabotage-Testing:** Prüfung der Fehlerfortpflanzung bei Injektion ungültiger Token-Arrays.

## 7. Offene Punkte & Technische Schulden
- Keine.

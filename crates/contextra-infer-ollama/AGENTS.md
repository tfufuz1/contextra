# AGENTS.md — `contextra-infer-ollama`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-infer-ollama` bildet die abgesicherte Schnittstelle zu lokalen oder kontrollierten Ollama-Instanzen für Text-Generierung und Vektor-Einbettungen (Ring 2).
Als autarker Baustein implementiert er `EmbeddingProvider` aus `contextra-ports`.
Er erzwingt strenge Sicherheits- und Egress-Richtlinien (Loopback-Only per Default, Circuit-Breaker, Jitter-Backoff) und ist vollständig von Ranking- oder Speicher-Engines entkoppelt.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 2 (Externe Inferenz & Provider-Adapter)
- **Klasse:** Erweiterung `[E]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **Network Egress Guard:** Standardmäßig striktes Loopback-Only (`127.0.0.0/8`, `::1`, `localhost`) und Unix-Domain-Sockets. Remote-Ziele erfordern explizite Allowlist-Freigabe und Egress-Audit-Hooks.
- **Fail-Closed Dimensionsprüfung:** Feststellen und Verriegeln der Vektor-Dimension beim ersten Aufruf; Dimension-Mismatches führen sofort zu kontrollierten Fehlern zur Vermeidung stiller Index-Korruption.
- **Circuit-Breaker & Jitter:** Absicherung vor Kaskaden-Ausfällen bei Hardware-Überlastung oder Server-Neustarts.
- **Zero-Panic-Doktrin:** Vollständiger Ausschluss von `unwrap()` und `expect()`; strukturierte Fehlerbehandlung über `ContextraError`.

## 4. Verbotene Muster
- Beliebige HTTP-Redirects (`reqwest::redirect::Policy::none()` ist Pflicht).
- Direkte Abhängigkeiten zu `contextra-rank` (Entkopplung erfolgt über lokale Abstraktionen wie `ImportanceCalibrator`).
- Verwendung von `contextra-core` (statdessen direkte Nutzung von `contextra-types` und `contextra-ports`).
- Ungeprüfte HTTP-Aufrufe ohne Timeout- und Response-Größenlimits (`max_response_bytes`).

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Implementiert:** `EmbeddingProvider`.
- **Konsumiert:** `Clock`-Port, `Rng`-Port, `contextra-types`, `contextra-ports`.
- **Bereitgestellt für:** `contextra-engine`, `contextra-db`, `contextra-mcp`.

## 6. Tests, Sabotage & Qualitätssicherung
- **Egress-Boundary Tests:** Verifikation der Blockierung nicht autorisierter externer IP-Adressen und manipulierte URLs.
- **Circuit-Breaker Tests:** Validierung von State-Transitions (Closed -> Open -> Half-Open) unter simulierten Fehlern.
- **Dimensions-Konsistenz-Tests:** Prüfung auf Fail-Closed bei variierenden Vektorlängen.

## 7. Offene Punkte & Technische Schulden
- Keine.

# Analyse der unverdrahteten Crates: `contextra-ingest`, `contextra-agent` und `contextra-kvcache`

**Datum:** 2026-10-29
**Autor:** Senior Rust Architect
**Status:** Abgeschlossen

---

## 1. Ausgangslage & Zielsetzung

Im Rahmen der Transformation von Contextra zu einer entkoppelten, produktionsreifen Fundamentbibliothek (gemäß SPEC §0/§1) wurde eine statische Abbildungsanalyse des Workspace-Abhängigkeitsgraphen durchgeführt.

Dabei identifizierte die Analyse drei substanzielle Crates ohne aktive Inbound-Abhängigkeiten (Reverse Dependencies / Fan-in = 0) durch andere Workspace-Crates:
1. `contextra-ingest` (Dokumenten-Format-Parsing, Chunking, IR; ca. 21.613 non-empty LOC)
2. `contextra-agent` (ReAct-Loops, Tool-Execution, Memory-Orchestrierung; ca. 9.181 non-empty LOC)
3. `contextra-kvcache` (Trie-basierter TenantPrefixKvStore für LLM-Prefix-Caching; ca. 6.233 non-empty LOC)

Zusammen umfassen diese drei Crates ca. 37.027 non-empty LOC (bzw. ca. 48.000 Brutto-Zeilen Code inkl. Kommentaren/Tests), was ca. 6,0 % des Gesamt-Workspace an non-empty LOC (bzw. ~15 % des produktiven Anwendungs-Codes außerhalb von Test-Fixtures) ausmacht.

---

## 2. Verifizierter Reverse-Dependency-Graph (Workspace Crates)

Die nachfolgende Tabelle basiert auf den exakten Ergebnissen von `cargo metadata --format-version 1` auf HEAD.

| Crate Name | Reverse Dependencies (Nutzer im Workspace) | In Cargo.toml `members` | In Cargo.toml `default-members` (CI-Abdeckung) |
|---|---|---|---|
| `contextra` | `contextra-mcp` | Ja | Ja |
| `contextra-adapt` | `contextra-db`, `contextra-graph`, `contextra-py`, `contextra-rank`, `contextra-router` | Ja | Ja |
| `contextra-agent` | **Keine (0)** | Ja | Ja |
| `contextra-audit-export` | `contextra` | Ja | Nein |
| `contextra-avv-generator` | `contextra`, `contextra-privacy` | Ja | Nein |
| `contextra-bench` | **Keine (0)** | Ja | Nein |
| `contextra-checkpoint` | `contextra-agent`, `contextra-db`, `contextra-engine` | Ja | Ja |
| `contextra-cognition` | `contextra-db` | Ja | Ja |
| `contextra-core` | `contextra`, `contextra-bench`, `contextra-engine` | Ja | Ja |
| `contextra-crypto` | `contextra`, `contextra-db`, `contextra-engine`, `contextra-infer-candle`, `contextra-kvcache`, `contextra-mcp`, `contextra-privacy`, `contextra-store`, `contextra-vector` | Ja | Ja |
| `contextra-db` | `contextra`, `contextra-agent`, `contextra-bench`, `contextra-py`, `contextra-router` | Ja | Ja |
| `contextra-engine` | `contextra-cognition`, `contextra-db`, `contextra-ingest` | Ja | Ja |
| `contextra-graph` | `contextra-agent`, `contextra-bench`, `contextra-cognition`, `contextra-db`, `contextra-engine`, `contextra-router`, `contextra-store` | Ja | Ja |
| `contextra-infer-candle` | `contextra`, `contextra-bench`, `contextra-cognition`, `contextra-mcp` | Ja | Nein |
| `contextra-infer-ollama` | `contextra-mcp` | Ja | Ja |
| `contextra-infer-onnx` | `contextra-bench`, `contextra-mcp` | Ja | Ja |
| `contextra-ingest` | **Keine (0)** | Ja | **Nein** |
| `contextra-kvcache` | **Keine (0)** | Ja | Ja |
| `contextra-license` | `contextra`, `contextra-mcp` | Ja | Nein |
| `contextra-mcp` | **Keine (0)** | Ja | Ja |
| `contextra-mvcc` | `contextra-checkpoint`, `contextra-core`, `contextra-engine`, `contextra-store`, `contextra-vector` | Ja | Ja |
| `contextra-ports` | `contextra`, `contextra-adapt`, `contextra-agent`, `contextra-audit-export`, `contextra-checkpoint`, `contextra-cognition`, `contextra-core`, `contextra-crypto`, `contextra-db`, `contextra-engine`, `contextra-graph`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-kvcache`, `contextra-license`, `contextra-mcp`, `contextra-mvcc`, `contextra-privacy`, `contextra-rank`, `contextra-router`, `contextra-store`, `contextra-testkit`, `contextra-text`, `contextra-vector` | Ja | Ja |
| `contextra-privacy` | `contextra`, `contextra-mcp`, `contextra-router` | Ja | Nein |
| `contextra-py` | **Keine (0)** | Ja | Nein |
| `contextra-rank` | `contextra`, `contextra-db`, `contextra-engine`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-mcp`, `contextra-py` | Ja | Ja |
| `contextra-router` | `contextra-agent`, `contextra-mcp` | Ja | Ja |
| `contextra-sandbox` | `contextra-engine`, `contextra-store` | Ja | Nein |
| `contextra-simd` | `contextra-vector` | Ja | Ja |
| `contextra-store` | `contextra`, `contextra-agent`, `contextra-bench`, `contextra-checkpoint`, `contextra-cognition`, `contextra-db`, `contextra-engine`, `contextra-graph`, `contextra-py` | Ja | Ja |
| `contextra-sys` | `contextra-bench`, `contextra-db`, `contextra-engine`, `contextra-store`, `contextra-vector` | Ja | Ja |
| `contextra-testkit` | `contextra`, `contextra-agent`, `contextra-checkpoint`, `contextra-cognition`, `contextra-crypto`, `contextra-db`, `contextra-engine`, `contextra-graph`, `contextra-mvcc`, `contextra-privacy`, `contextra-store` | Ja | Ja |
| `contextra-text` | `contextra-agent`, `contextra-bench`, `contextra-db`, `contextra-engine`, `contextra-router` | Ja | Ja |
| `contextra-types` | `contextra`, `contextra-adapt`, `contextra-agent`, `contextra-audit-export`, `contextra-avv-generator`, `contextra-checkpoint`, `contextra-cognition`, `contextra-core`, `contextra-crypto`, `contextra-db`, `contextra-engine`, `contextra-graph`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-ingest`, `contextra-kvcache`, `contextra-license`, `contextra-mcp`, `contextra-mvcc`, `contextra-ports`, `contextra-privacy`, `contextra-py`, `contextra-rank`, `contextra-router`, `contextra-simd`, `contextra-store`, `contextra-testkit`, `contextra-text`, `contextra-vector` | Ja | Ja |
| `contextra-vector` | `contextra-agent`, `contextra-bench`, `contextra-cognition`, `contextra-db`, `contextra-engine`, `contextra-router`, `contextra-store` | Ja | Ja |
| `contextra-wire` | `contextra-core`, `contextra-mcp`, `contextra-py`, `contextra-router` | Ja | Ja |
| `xtask` | **Keine (0)** | Ja | Nein |

---

## 3. Detail-Analyse der drei betroffenen Crates

### 3.1. `contextra-ingest`
- **Funktion:** Format-Parsing (PDF via `lopdf`, Word/Excel via `calamine`/`cfb`, HTML/XML via `html5ever`/`quick-xml`, E-Mail via `mail-parser`, ZIP, CSV) sowie chunking und kanonische IR-Transformation.
- **Sicherheitsrelevanz:** Höchste Angriffsfläche im Gesamtsystem für untrusted I/O Input (Zip-Bomben, XML-Entity Injections, beschädigte PDF-Streams, Speicherallokations-Angriffe).
- **Aktueller Verdrahtungsstatus:** Nicht in `default-members` von `Cargo.toml`. Wird bei Standard-`cargo build` oder `cargo test` in der CI nicht mitgebaut oder getestet. Inbound-Abonnenten fehlen vollständig (weder Facade noch DB binden ingest direkt ein).
- **Optionen & Bewertung:**
  - *(a) Anschließen:* Verdrahtung an `contextra-db` / Facade `contextra::Contextra` via Feature-Flag `ingest` (oder Ingestion-Extension-Port). Aufnahme in `default-members`.
  - *(b) Standalone-Deklaration:* Belassen in `members`, aber saubere API-Dokumentation + Aufnahme in `default-members` als optionales Zusatz-Crate für Ingestion-Pipelines.
  - *(c) Ausgliedern / Auslagern:* Parken unter `docs/restore/PARK-INGEST` (analog PARK-SANDBOX / PARK-ONNX).
- **Empfehlung:** (a) Verdrahtung an `contextra` Facade via Feature-Flag `ingest` + Aufnahme in `default-members`.

### 3.2. `contextra-agent`
- **Funktion:** Agentic ReAct-Loop, Tool Execution, Multi-Agent Planning & Memory-Orchestrierung.
- **Sicherheitsrelevanz:** Mittel bis hoch (Prompt Injection Risk, Tool Execution Boundaries).
- **Aktueller Verdrahtungsstatus:** In `default-members` enthalten, baut und testet in CI. Keines der Fassaden- oder Server-Crates (`contextra`, `contextra-mcp`) ruft `contextra-agent` derzeit auf.
- **Optionen & Bewertung:**
  - *(a) Anschließen:* Integration in `contextra-mcp` (für Agent-Tools) oder Option für Agentic Queries in `contextra`.
  - *(b) Standalone-Deklaration:* Deklaration als High-Level Ring-3 Agent-Framework mit stabilen Doctests.
  - *(c) Ausgliedern:* Parken / Archivieren.
- **Empfehlung:** (a) Anbindung an `contextra-mcp` (Tool Execution & Agent Workflows) als primärer Produktionsweg.

### 3.3. `contextra-kvcache`
- **Funktion:** Einzige produktive Implementierung des Trait-Ports `contextra_ports::kv::KvPrefixStore` (Radix-Trie mit TTL, Memory-Bounds und Eviction).
- **Sicherheitsrelevanz:** Gering bis mittel (DoS via Unbounded Cache Growth, Tenant Memory Contamination).
- **Aktueller Verdrahtungsstatus:** In `default-members` enthalten. `contextra-infer-candle` besitzt `with_prefix_store(Arc<dyn KvPrefixStore>)`, nutzt im eigenen Integrationstest bisher aber nur ein lokales `InMemoryKvPrefixStore`-Mock-Double.
- **Optionen & Bewertung:**
  - *(a) Anschließen:* Verdrahtung in `contextra-infer-candle` bzw. `contextra-engine` für Stage B Prefix Reuse.
  - *(b) Standalone-Deklaration:* Als Standard-Cache-Provider im Baukasten-Modell belassen.
  - *(c) Ausgliedern:* Nicht empfohlen, da dies die einzige produktive `KvPrefixStore`-Implementierung ist.
- **Empfehlung:** (a) Injektion des `TenantPrefixKvStore` in `contextra-infer-candle` / Engine-Inferenz-Pipeline.

---

## 4. Fazit & Handlungsbedarf

Die Detaillierung der Architekturentscheidungen sowie der konkreten Arbeitspakete erfolgt normativ in ADR-0009 (`docs/decisions/0009-architektur-entkoppelte-crates-integration-ausgliederung.md`).

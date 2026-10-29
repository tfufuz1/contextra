# ADR 0009: Integration, Modularisierung und Baukastenaspekte unverdrahteter Workspace-Crates (`contextra-ingest`, `contextra-agent`, `contextra-kvcache`)

## Status
Proposed — benötigt Maintainer-Freigabe

## Datum
2026-10-29

## Relevante Komponenten
- `contextra-ingest`
- `contextra-agent`
- `contextra-kvcache`
- `contextra` (Facade)
- `contextra-mcp`
- `contextra-infer-candle`
- `Cargo.toml` (Workspace Configuration)

---

## Kontext & Problemstellung
Im Rahmen der Weiterentwicklung von Contextra zu einer entkoppelten, produktionsreifen Fundamentbibliothek (gemäß SPEC §0/§1) ergab eine statische Analyse des Workspace-Abhängigkeitsgraphen via `cargo metadata`, dass drei substantielle Crates im Workspace ohne Inbound-Abonnenten (Reverse Dependencies / Fan-in = 0) vorliegen:
1. `contextra-ingest` (~21.613 non-empty LOC): Format-Parsing (PDF, DOCX, XLSX, HTML, E-Mail, ZIP) und Chunking-Pipelines. Steht aktuell nicht in `default-members` von `Cargo.toml` (keine automatische CI-Testabdeckung).
2. `contextra-agent` (~9.181 non-empty LOC): Agentic ReAct-Loops, Tool Execution und Memory-Orchestrierung. Steht in `default-members`, wird aber von keinem Fassaden-/Server-Crate aufgerufen.
3. `contextra-kvcache` (~6.233 non-empty LOC): Einzige produktive Implementierung des Trait-Ports `contextra_ports::kv::KvPrefixStore` (Tenant-isoliertes Prefix-Caching für LLM-Inferenz). Steht in `default-members`, ist jedoch nicht in die Inferenz-Pipeline von `contextra-infer-candle` oder `contextra-engine` verdrahtet.

Insgesamt betreffen diese drei Crates ca. 37.027 non-empty LOC (~15 % des produktiven Anwendungscodes). Gemäß SPEC §0/§1 bedeutet Entkopplung nicht Unverdrahtetheit, sondern das Bereitstellen bewusster, dokumentierter Schnittstellen in einem Baukasten-System.

Die Ingest-Komponente stellt zudem die mit Abstand größte Angriffsfläche für unvertrauenswürdige Benutzereingaben (Input Sanitization, Zip-Bomben, PDF/XML Parsing Defects) dar und erfordert strikte Sicherheitsisolierung sowie CI-Absicherung.

---

## Entscheidungsoptionen je Crate

### 1. `contextra-ingest`

- **Option 1a: Anschließen an `contextra` (Facade) via Feature-Flag `ingest` + Aufnahme in `default-members`**
  - *Beschreibung:* `contextra-ingest` wird über ein opt-in Feature `ingest` in `crates/contextra` als High-Level Ingestion Pipeline bereitgestellt. Aufnahme in `default-members` sichert kontinuierliche CI-Kompilierung und Testabdeckung.
  - *Aufwand:* Mittel (ca. 1–2 Personentage).
  - *Risiko:* Gering, da hinter Feature-Flag gekapselt.
  - *Sicherheitswirkung:* Hohe Sicherheitsrelevanz! `contextra-ingest` parst komplexe Binär-/XML-Formate. Die Ausführung muss defensive Parser-Limitierungen (ResourceBudget) und `catch_unwind` nutzen. Durch Aufnahme in `default-members` werden Security-Lints und Fuzzing in der CI garantiert.
  - *Betriebs-/Wartungskosten:* Moderate zusätzliche Abhängigkeiten (z.B. `calamine`, `lopdf`, `mail-parser`).
  - *Auswirkung auf CI-Zeit:* +8–12 Sekunden bei vollem `cargo test --workspace`.

- **Option 1b: als eigenständige Bibliothek mit stabiler API & Beispielen im Workspace belassen**
  - *Beschreibung:* Verbleibt in `members`, erhält Aufnahme in `default-members` und wird als Standalone-Parsing-Crate mit Doctests dokumentiert.
  - *Aufwand:* Gering (ca. 0,5 Personentage).
  - *Risiko:* Funktionsumfang im Contextra-Kern nicht direkt nutzbar.
  - *Sicherheitswirkung:* CI-Abdeckung gegeben, aber keine direkte Nutzung im Kern-System.

- **Option 1c: Ausgliedern / Parken unter `docs/restore/PARK-INGEST`**
  - *Beschreibung:* Entfernen aus `Cargo.toml` `members`, Archivierung des Codes analog PARK-SANDBOX/PARK-ONNX.
  - *Aufwand:* Gering.
  - *Risiko:* Verlust von 21,6k LOC getesteter Dokumenten-Extraktionslogik.
  - *Sicherheitswirkung:* Reduziert die Angriffsfläche im Haupt-Repo auf Null.
  - *Auswirkung auf CI-Zeit:* −10 Sekunden CI-Zeit.

---

### 2. `contextra-agent`

- **Option 2a: Anschließen an `contextra-mcp` (Tool-Server & Agent-Endpunkte)**
  - *Beschreibung:* ReAct-Execution-Loops und Agent-Memory-Pipelines werden in `contextra-mcp` als optionale Tool-/Agenten-Anbindungen verdrahtet.
  - *Aufwand:* Mittel (ca. 1 Personentag).
  - *Risiko:* Gering.
  - *Sicherheitswirkung:* ReAct-Loops benötigen strikte Execution-Limits (Max Steps, Tool Permission Guards) zur Verhinderung unendlicher Rekursionen oder Prompt-Injections.
  - *Betriebs-/Wartungskosten:* Gering (nutzt bestehende Async-Runtimes und Types).
  - *Auswirkung auf CI-Zeit:* Keine Änderung (bereits in `default-members`).

- **Option 2b: als eigenständige Bibliothek mit stabiler API & Beispielen deklarieren**
  - *Beschreibung:* Verbleibt in `default-members` als eigenständiges Agentic Framework für externe Nutzung.
  - *Aufwand:* Gering.
  - *Risiko:* Kein direkter Produktivweg im Kern.

- **Option 2c: Aus dem Workspace nehmen / Parken unter `docs/restore/PARK-AGENT`**
  - *Beschreibung:* Auslagern aus `members`.
  - *Aufwand:* Gering.
  - *Risiko:* Verlust der Agenten-Funktionalität.

---

### 3. `contextra-kvcache`

- **Option 3a: Anschließen an `contextra-infer-candle` (Stage B LLM Prefix-Reuse)**
  - *Beschreibung:* Injektion von `TenantPrefixKvStore` in `contextra-infer-candle::InferenceEngine` als Standard-Prefix-Cache via Trait `KvPrefixStore`.
  - *Aufwand:* Gering (ca. 0,5 Personentage).
  - *Risiko:* Vernachlässigbar.
  - *Sicherheitswirkung:* Tenant-Isolation auf Cache-Ebene wird durch Bounded Radix-Tries mit TTL strikt durchgesetzt.
  - *Betriebs-/Wartungskosten:* Vernachlässigbar (reine In-Memory Datenstruktur ohne externe Binär-Deps).
  - *Auswirkung auf CI-Zeit:* Keine Änderung (bereits in `default-members`).

- **Option 3b: als eigenständiges Crate mit stabiler API belassen**
  - *Beschreibung:* Verbleibt unverändert in `default-members`.
  - *Aufwand:* Null.

- **Option 3c: Aus dem Workspace nehmen**
  - *Beschreibung:* Nicht empfohlen, da dies die einzige produktive Implementierung des Ports `KvPrefixStore` im Gesamtsystem vernichten würde.

---

## Empfehlung & Begründung

Der Senior Rust Architect empfiehlt die Verfolgung von **Option 1a**, **Option 2a** und **Option 3a**:

1. **`contextra-ingest` (Option 1a):**
   - **Begründung:** Dokumenten-Ingestion ist für RAG-Anwendungsfälle essenziell. Die Aufnahme in `default-members` schließt eine gefährliche Blinde-Fleck-Lücke in der CI. Das Feature-Flag `ingest` in `contextra` wahrt das Entkopplungsprinzip (Zero-Cost if unused).
2. **`contextra-agent` (Option 2a):**
   - **Begründung:** `contextra-mcp` ist der logische Integrationspunkt für Agentic Execution und Tool-Calling.
3. **`contextra-kvcache` (Option 3a):**
   - **Begründung:** Beseitigt die Unverdrahtetheit der einzigen produktiven `KvPrefixStore`-Implementierung in `contextra-infer-candle` mit minimalem Aufwand und sofortigem Performancenutzen (Prefix-Reuse).

---

## Priorisierte Arbeitspakete für Folge-Sessions

Um parallele Bearbeitung ohne Merge-Konflikte zu ermöglichen, werden die Umsetzungsschritte in strikt disjunkte Arbeitspakete geschnitten:

### Arbeitspaket WP-INGEST-01: CI-Integration & Facade-Wiring für `contextra-ingest`
- **Titel:** `contextra-ingest` CI-Aufnahme & Facade Feature-Flag
- **Betroffene Crates:** `Cargo.toml`, `crates/contextra`
- **Disjunkte Dateimenge:** `Cargo.toml`, `crates/contextra/Cargo.toml`, `crates/contextra/src/ingest.rs` (neu)
- **Aufgaben:**
  1. Hinzufügen von `"crates/contextra-ingest"` zu `default-members` in `Cargo.toml`.
  2. Hinzufügen des optionalen Feature-Flags `ingest = ["dep:contextra-ingest"]` in `crates/contextra/Cargo.toml`.
  3. Re-Export der High-Level Ingestion Pipeline in `crates/contextra/src/lib.rs`.

### Arbeitspaket WP-KVCACHE-01: Verdrahtung von `contextra-kvcache` in Inferenz-Pipeline
- **Titel:** Integration von `TenantPrefixKvStore` in `contextra-infer-candle`
- **Betroffene Crates:** `crates/contextra-infer-candle`
- **Disjunkte Dateimenge:** `crates/contextra-infer-candle/src/inference/core.rs`, `crates/contextra-infer-candle/tests/kv_prefix_reuse_test.rs`
- **Aufgaben:**
  1. Hinzufügen von `contextra-kvcache` als Dev-Dependency/Optional-Dependency in `crates/contextra-infer-candle/Cargo.toml`.
  2. Ersetzen/Ergänzen des InMemory-Mocks in `kv_prefix_reuse_test.rs` durch `TenantPrefixKvStore`.

### Arbeitspaket WP-AGENT-01: Anbindung von `contextra-agent` an MCP-Server
- **Titel:** Anbindung von `contextra-agent` an `contextra-mcp` Agenten-Tools
- **Betroffene Crates:** `crates/contextra-mcp`
- **Disjunkte Dateimenge:** `crates/contextra-mcp/Cargo.toml`, `crates/contextra-mcp/src/agent_dispatch.rs` (neu)
- **Aufgaben:**
  1. Hinzufügen der optionalen Dependency `contextra-agent` unter Feature `agent` in `crates/contextra-mcp/Cargo.toml`.
  2. Integration des Agent-Runners in `server_dispatch.rs`.

---

## Konsequenzen & Risiken
- Alle Crates erhalten 100%ige CI-Abdeckung.
- Die Architektur entspricht dem Baukasten-Modell von SPEC §0/§1.
- Die Arbeitspakete können parallel ohne Dateikonflikte ausgeführt werden.

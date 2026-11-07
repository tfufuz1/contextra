# AGENTS.md — `contextra-mcp`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-mcp` implementiert den Model Context Protocol (MCP) Server für Contextra (Ring 4).
Er stellt standardisierte JSON-RPC 2.0 Schnittstellen (ausschließlich über `stdio`, HTTP-Schicht entfernt gem. ADR-010) bereit, über die externe Coding-Agenten und LLMs die Contextra-Bausteine sicher steuern können.
Als Gateway-Baustein kapselt er Zugriffsberechtigungen (`McpSandbox`), Prompt-Injection-Schutz und flüchtige Ergebnisablage, ohne interne Domänenlogik zu duplizieren.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 4 (Model Context Protocol Gateway & Server)
- **Klasse:** Erweiterung `[E]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports`, `contextra-wire`, `contextra-rank`, `contextra-crypto`, `contextra-privacy`, `contextra-router` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **Sandbox-Zwang (ADR-044):** Sämtliche Tool-Ausführungen laufen ausnahmslos über `McpSandbox` mit strikten Timeouts und Rechteprüfungen.
- **Single-Lock Disziplin:** `McpSandbox` schützt Ergebnisse über ein einziges, ungeschachteltes `parking_lot::Mutex`; geschachtelte Locks sind verboten.
- **UTF-8 sichere Token-Kappung:** `RecallTruncator` schneidet Text exakt an UTF-8 Zeichengrenzen (`is_char_boundary`), um Slicing-Panics bei Umlauten oder Emojis zu verhindern.
- **Zero-Panic-Doktrin:** Vollständiger Ausschluss von `unwrap()` und `expect()`; JSON-RPC Fehlercodes (-32700 bis -32603) für alle Fehlerzustände.

## 4. Verbotene Muster
- Direkte HTTP-, WebSocket- oder Netzwerklayer im MCP-Crate (ausschließlich `run_stdio`).
- Ungeprüfte Schreibzugriffe ohne explizite Write-Authorization.
- Verwendung von `contextra-core` (statdessen direkte Nutzung von `contextra-types` und `contextra-ports`).
- Geschachtelte Locks über Modulgrenzen hinweg (`detect_nested_locks.yml`).

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Implementiert:** MCP JSON-RPC Server über Stdio.
- **Konsumiert:** `contextra-wire`, `contextra-ports`, `contextra-types`.
- **Angebundene Inferenz-Provider:** Candle, Ollama und ONNX als optionale Feature-Gates.

## 6. Tests, Sabotage & Qualitätssicherung
- **Prompt Injection & Escaping Tests:** Verifikation der Sandbox gegen manipulative Tool-Argumente.
- **UTF-8 Slicing Fuzzing:** Fuzzing des Token-Truncators mit Multibyte-Sequenzen, CJK und Emojis.
- **JSON-RPC Conformance:** Sabotagetests mit fehlerhaften RPC-Nachrichten zur Prüfung der Spezifikationskonformität.

## 7. Offene Punkte & Technische Schulden
- DEBT-01: Vollständige Entkopplung der MCP-Erasure-Tools via `contextra-erasure`.

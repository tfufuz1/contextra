# WP-102 Vorbereitungsanalyse: Guards und Abhängigkeiten von contextra-mcp

Dieses Dokument analysiert im Vorfeld von Arbeitspaket WP-102 die Abhängigkeiten von `crates/contextra-mcp/` zu `contextra-privacy`, `contextra-router` und `contextra-crypto`.

## 1. Übersicht der Module und Abhängigkeiten

| Modul | Abhängigkeiten | Gehört zum Such-Kern? | Begründung / Empfehlung für Feature-Gating (`guards`) |
|---|---|---|---|
| `egress_guard.rs` | `contextra-privacy` (`egress_guard::*`, `egress_vault::BoxFuture`), `contextra::Collection` | **Nein** | Adapter für Egress-Prüfung gegen Exfiltration von privaten Daten vor Cloud-Weiterleitung. Gehört in Feature `guards`. |
| `egress_gateway.rs` | `contextra-privacy` (`egress_gateway::*`, `egress_vault::*`, `EgressError`, `InjectionDetector`) | **Nein** | Klassifiziert Cloud-MCP-Anfragen und schützt vor Datenabfluss. Gehört in Feature `guards`. |
| `bulk_exfiltration_detector.rs` | `contextra-privacy` (`bulk_exfiltration_detector::*`) | **Nein** | Erkennt massenhafte Datenabfragen/Scraping über MCP-Sessions. Gehört in Feature `guards`. |
| `prompt_injection/` | `rand`, Standardbibliothek (keine direkte Privacy/Crypto-Crate-Abhängigkeit; implementiert Trait `InjectionDetector` aus `contextra-privacy`) | **Nein** | Heuristischer und pattern-basierter Schutz vor Prompt-Injections. Gehört in Feature `guards`. |
| `routing.rs` | `contextra-router` (`DefaultRouterEngine`, `RouterEngine`, `SlmProfile`), `contextra_types` | **Nein** | Dynamisches Routing und Modell-Auswahl für SLMs/LLMs basierend auf Kontextgröße und Latenzbudgets. Kann hinter Feature `routing` oder `guards` isoliert werden. |
| `sandbox.rs` | `contextra-crypto` (`CryptoKey`) | **Nein** (Ausführungskern) | Die Sandbox kapselt Tool-Ausführungen mit Timeouts und temporärer Verschlüsselung des Volatile-Stores (`VolatileStore`). Soll später nach `contextra-sandbox` migriert werden (Auftrag SANDBOX-1). `contextra-crypto` wird hier nur für AES-GCM/SIV des flüchtigen Speichers verwendet. |
| `config.rs` | `contextra-router::SlmProfile` | **Nein** | Konfigurationsoptionen für SLM-Routing. |

## 2. Such-Kern vs. Begleitlogik

- **Reiner Such-Kern von `contextra-mcp`:**
  - `server.rs`, `server_dispatch.rs`, `server_tools.rs`, `tools_crud.rs`, `explain.rs`, `io.rs`, `protocol.rs`, `validation.rs`.
  - Stellt Standard-JSON-RPC-Werkzeuge bereit: `contextra_search`, `contextra_get`, `contextra_collections`, `contextra_create_collection`, `contextra_drop_collection`, `contextra_delete`, `contextra_explain`, `contextra_forget`, `contextra_insert`, `contextra_upsert`.
  - Benötigt nur `contextra` (Fassade) und `contextra-types`/`contextra-ports`/`contextra-wire`.

- **Begleitlogik für Sicherheit und Routing (Kandidaten für Entkopplung in WP-102):**
  - Alle Egress-, Bulk-Exfiltration- und Injection-Guards können über ein Cargo-Feature `guards` optional gemacht werden.
  - Das Routing über `contextra-router` kann optional über `routing` (oder ebenfalls unter `guards`) eingebunden werden.
  - Dadurch kann `contextra-mcp` im Minimalzustand ohne `privacy`, `router` und `crypto` gebaut werden.

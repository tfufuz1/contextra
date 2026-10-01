# ADR-PLUGIN-SYSTEM-STATUS: Status und Architekturanalyse des PluginManifest & PluginRegistry Scaffolding-Codes

* **Datum**: 2026-10-01
* **Status**: ⏳ Proposed / Pending Architectural Decision
* **Betroffene Komponenten**: `contextra-ports` (`src/plugin.rs`), `contextra-mcp` (`src/plugin_status.rs`), `contextra-types` (`error.rs`, `error_dto.rs`)
* **Spezifikationsreferenzen**: `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` (§INV-PLUGIN-DEPENDENCY, §10.1–10.5), `docs/TYPE_REGISTRY.md`

---

## 1. Ausgangslage & Befund (Codebase Audit)

Im Rahmen einer umfassenden Codebase-Untersuchung am HEAD wurde der Implementierungs- und Integrationsstatus des Plugin-Systems (`PluginManifest`, `PluginRegistry`, `PluginCapability`) evaluiert.

### Kernbefunde:
1. **Scaffolding-Charakter in `contextra-ports`**:
   In `crates/contextra-ports/src/plugin.rs` sind die zentralen Abstraktionen definiert:
   - Trait `PluginManifest`: Send + Sync Trait mit Methoden `name()`, `requires()`, `conflicts()`, `feature_ring_required()`, `capability()` und `activate()`.
   - Trait-Implementationen: Außerhalb von Testumgebungen (`SimplePlugin` in `plugin.rs` und `DummyPlugin` in `tests/plugin_registry_dependency_cycle.rs`) existiert **keine einzige produktive Implementierung** von `PluginManifest` im gesamten Repository.
   - Struct `PluginRegistry`: Implementiert vollständiges Topological Sorting via Kahns Algorithmus zur Abhängigkeitsauflösung (`requires()`), Konflikterkennung (`conflicts()`), Lizenz-Ring-Gating (`LicenseGate`) sowie All-or-Nothing Batched-Aktivierung (`activate_all()`).

2. **Fehlende Verdrahtung in Consumer-Crates**:
   Weder `contextra-engine`, `contextra-agent`, `contextra-db`, `contextra-core` noch `contextra-sandbox` besitzen Anbindungen, Aufrufe oder Registrierungs-Abläufe für `PluginManifest`. Es gibt keine dynamische Modulladelogik (z.B. via `libloading` oder WASM Plugin Host Extension) im Laufzeit-Engine-Kernel.

3. **Verwendung im MCP-Server (`contextra-mcp`)**:
   Der MCP-Server akzeptiert in `McpServer` ein optionales `Arc<PluginRegistry>`. Das MCP-Tool `contextra_plugin_status` (`plugin_status.rs`) ruft `registry.snapshot()` auf, um aktive Plugins als JSON-Response bereitzustellen. Wird dem MCP-Server kein Registry-Objekt übergeben, instanziiert er ein leeres `PluginRegistry` zur Beantwortung des Tool-Aufrufs.

4. **Keine offenen Entwicklungsaufträge**:
   Im gesamten Repository wurden weder offene `TODO`/`FIXME`-Marker noch ungeklärte PRs oder Branch-Notizen bezüglich einer anstehenden Plugin-System-Integration gefunden.

---

## 2. Architekturanalyse & Kontext

Das Plugin-System in `contextra-ports` erfüllt aktuell den Zweck eines **Spezifikationsnachweises** für die Invariante `INV-PLUGIN-DEPENDENCY` aus der Spezifikation (`CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` §INV-PLUGIN-DEPENDENCY). Es ist modular isoliert und in Ring 0 (`contextra-ports`) mit 100% Testabdeckung (einschließlich Property-basierten Zyklen-Tests) abgesichert.

Da Contextra als Sovereign AI Knowledge OS mit strikten Sicherheits- und Determinismus-Garantien (P28, Pure Rust / Sovereign Policy ADR-004) entworfen ist, wirft die Einbindung von dynamischen Third-Party Plugins architektonische Grundsatzfragen auf.

---

## 3. Handlungsoptionen für die menschliche Entscheidung

### Option A: Vollständige Ausgestaltung & Anbindung eines Plugin-Systems (Plugin Host)
* **Beschreibung**:
  Entwicklung eines vollwertigen Runtime Plugin Hosts. Plugins könnten entweder als isolierte WASM-Module in `contextra-sandbox` ausgeführt oder über definierte Extension Points in `contextra-engine` (z.B. Custom Reranker, Custom Guardrails, Custom Connectors) geladen und über `PluginRegistry::activate_all()` beim Systemstart verifiziert und aktiviert werden.
* **Aufwandsschätzung**: **3 bis 4 Personenwochen (120 - 160 Stunden)**
  *(Inkl. WASM-Host-Extension-Interfaces, FlatBuffers IPC-Adapter, Sandboxing-Limits, Manifest-Loader und Security-Audits)*.
* **Pro**:
  - Maximale Erweiterbarkeit für Enterprise-Deployments.
  - Dynamisches Nachladen von Custom Extensions ohne Re-Kompilierung von Contextra.
* **Contra**:
  - Signifikanter Anstieg der Komplexität und der Angriffsfläche.
  - Risiko der Verletzung von Determinismus-Invarianten (P28) bei nicht-deterministischen Third-Party-Plugins.

### Option B: Rückbau & Konsolidierung (Removal / Deprecation von PluginManifest)
* **Beschreibung**:
  Entfernen des ungenutzten Trait `PluginManifest` und der topologischen Sortierlogik aus `contextra-ports`. Das MCP-Tool `contextra_plugin_status` wird so angepasst, dass es direkt die aktivierten Workspace-Features und den `FeatureRing` aus dem `LicenseGate` abfragt, ohne ein Trait-basiertes Scaffolding vorzuhalten.
* **Aufwandsschätzung**: **1 bis 2 Personentage (8 - 16 Stunden)**
  *(Inkl. Bereinigung in `contextra-ports`, `contextra-mcp`, `contextra-types` DTOs, `TYPE_REGISTRY.md` und Spec-Update)*.
* **Pro**:
  - Beseitigung von ungenutztem Code (Dead Code / Scaffolding) in Ring 0/1.
  - Reduzierter Wartungs- und Testaufwand.
* **Contra**:
  - Erfordert Anpassung/Revision der Spezifikation (`CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` §INV-PLUGIN-DEPENDENCY).

### Option C: Beibehaltung des Ist-Zustands als Scaffolding (Status Quo / Empfehlung)
* **Beschreibung**:
  Der bestehende Code in `contextra-ports/src/plugin.rs` bleibt unverändert bestehen. Er dient weiterhin als formale Erfüllung von `INV-PLUGIN-DEPENDENCY` und bietet die Grundlage für das MCP-Tool `contextra_plugin_status`.
* **Aufwandsschätzung**: **0 Stunden** (Sofort einsatzbereit).
  - Keine Code-Änderungen im Produktsystem erforderlich.
  - ADR dokumentiert den Zustand transparent für das Entwicklerteam.

---

## 4. Empfehlung des Architects

1. **Kurz- bis Mittelfristig (Status Quo - Option C)**:
   Belassen des `PluginManifest`-Port-Codeblocks in `crates/contextra-ports/src/plugin.rs` in seinem aktuellen, vollständig getesteten Zustand. Er erzeugt keine Laufzeitkosten, keine Speicher-Allokationen bei Nicht-Nutzung und beeinträchtigt die Builds nicht.

2. **Langfristig (Strategische Entscheidung)**:
   Falls Contextra primär als fokussierte, eingebettete Library (Python/Rust SDK per ADR-018 / ADR-077) betrieben wird, sollte im Rahmen der nächsten Hauptversions-Bereinigung **Option B** gewählt werden. Sollte ein Enterprise-Ökosystem mit dynamischen Erweiterungen gefordert sein, bildet dieses Scaffolding das Fundament für **Option A**.

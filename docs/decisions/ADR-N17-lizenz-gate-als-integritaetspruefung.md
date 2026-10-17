# ADR-N17: F1 Lizenz-Gate als reine Integritätsprüfung

* **Status:** Accepted
* **Datum:** 2026-09-30
* **Kontext / Auslöser:**
  Contextra ist als eingebettete In-Process-Bibliothek (Pure Rust) und Open-Core-Architektur konzipiert. In Unternehmens- und Produktionsumgebungen erfordern Funktions-Freigaben (z. B. Enterprise-Erweiterungen oder Compliance-Module) eine Absicherung gegen unberechtigte Konfigurationsänderungen und Manipulationen.
  Eine klassische Lizenzprüfung über externe Phoning-Home-SaaS-Server widerspricht jedoch fundamental den Prinzipien der lokalen Datenhoheit, des Offline-Erst-Betriebs und des Sovereign Core (ADR-004, CONSTITUTION.md).

## Entscheidung

Das Lizenz-Gate (`LicenseGate` in `contextra-ports`, `contextra-license` und `ContextraBuilder`) fungiert ausschließlich als **lokale Integritäts- und Anti-Tamper-Prüfung** (`LicenseGateMode::Enforced` / `LicenseWiring`).
Es führt rein in-process kryptografische Prüfungen von Capability-Signaturen, Crate-Prüfsummen und Konfigurations-Hashes durch. Es finden keinerlei Netzwerkaufrufe, Telemetrie-Sendungen oder externe Lizenzschlüssel-Validierungen statt [V].

Für Entwicklungs- und Testumgebungen wird standardmäßig eine barrierefreie Entwickler-Gatter-Einstellung (`LicenseGateMode::DevelopmentOpen` / `OpenFastGate` / `with_development_open_gate()`) bereitgestellt [V].

## Alternativen

### Option A: Externe SaaS-Lizenzprüfung ("Phone Home")
* **Vorteile:** Zentrale administrative Entzugsmöglichkeit in Echtzeit; direkte Kontrolle über Kundennutzung.
* **Nachteile:** Bricht den Offline-Betrieb; erzeugt harte Netzwerk-Abhängigkeit; verletzt Datenschutz- und Souveränitäts-Garantien (§0.5).
* **Bewertung:** Verworfen [S].

### Option B: Striktes Lizenzmodell ohne Entwickler-Bypass
* **Vorteile:** Kein Risiko versehentlicher Nutzung im Entwickler-Modus in Produktion.
* **Nachteile:** Erschwert CI/CD-Pipelines, automatisierte Tests und Open-Source-Nutzung massiv.
* **Bewertung:** Verworfen [S].

## Konsequenzen

* **Integritätsschutz:** Lokale Integritätsprüfung verhindert versehentliche oder unautorisierte Modifikation von Feature-Flags in Binärpaketen [V].
* **Keine Latenz:** 100% in-process Ausführung ohne I/O- oder Netzwerk-Overhead [V].
* **Entwicklerfreundlichkeit:** Einfaches Umschalten auf `with_development_open_gate()` in Test- und Benchmarking-Szenarien [V].

## Reversibilitäts-Bedingung

Eine Umstellung erfordert einen grundlegenden Beschluss der Produktführung hin zu einem Geschäftsmodell, das eine strikte externe Erzwingung von Nutzungslizenzen zwingend vorschreibt [S].

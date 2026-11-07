# AGENTS.md — `contextra-audit-export`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-audit-export` stellt einen spezialisierten Export-Baustein für Audit-Logs und Verarbeitungsverzeichnisse (Ring 0 / Tooling) bereit.
Als isolierter Baustein formatiert er unveränderliche Protokolldaten in standardisierte Berichtsformate (JSON, CSV, PDF), ohne Einfluss auf die Speicher- oder Transaktionsleistung des Systems zu nehmen.
Er arbeitet rein deklarativ und konsumiert Daten über standardisierte Ports.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Audit-Export & Reporting)
- **Klasse:** Rand `[R]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **Unveränderbarkeit:** Export-Operationen sind rein lesend und verändern niemals den zugrundeliegenden Audit-Datenbestand.
- **Deterministische Formatierung:** Zeitstempel und Hash-Werte werden in standardisierten, kanonischen Formaten exportiert.
- **Zero-Panic-Doktrin:** Vollständiger Ausschluss von `unwrap()` und `expect()`; robuste Fehlerbehandlung bei ungültigen Eingabedaten.

## 4. Verbotene Muster
- Direkte I/O- oder Netzwerkoperationen ohne Port-Abstraktion.
- Veränderung von Transaktions- oder Audit-Zuständen.
- Verwendung von `contextra-core` (statdessen direkte Nutzung von `contextra-types` und `contextra-ports`).

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Konsumiert:** `contextra-types`, `contextra-ports`.
- **Bereitgestellt für:** CLI-Tools, Compliance-Auditoren und MCP-Server.

## 6. Tests, Sabotage & Qualitätssicherung
- **Format-Konformitätstests:** Prüfung exportierter Schemata gegen offizielle Validatoren.
- **Fuzzing:** Deserialisierungs-Fuzzing gegen unvollständige Audit-Log-Einträge.

## 7. Offene Punkte & Technische Schulden
- Keine.

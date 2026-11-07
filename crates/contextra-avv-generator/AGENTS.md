# AGENTS.md — `contextra-avv-generator`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-avv-generator` ist ein isolierter Dokumentengenerierungs-Baustein (Ring 0 / Tooling) zur deterministischen Erstellung von Auftragsverarbeitungsverträgen (AVV) und technischen Anlagen.
Er füllt vordefinierte Vorlagen anhand übergebener System-Konfigurationen und Sicherheits-Metadaten aus, ohne Abhängigkeiten zu Datenbank- oder Krypto-Kernen aufzuweisen.
Als autarker Baustein ist er ohne Seiteneffekte in administrativen Tools oder CI-Pipelines einsetzbar.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Template-Generierung & Dokumenten-Tooling)
- **Klasse:** Rand `[R]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **Deterministische Ausgabe:** Identische Konfigurationsparameter erzeugen bit-identische Ausgabedokumente.
- **Wording-Compliance:** Strikt neutrale und sachliche technische Beschreibungen ohne absolute Rechtsversprechen.
- **Zero-Panic-Doktrin:** Vollständiger Ausschluss von `unwrap()` und `expect()`.

## 4. Verbotene Muster
- Direkte I/O- oder Netzwerkoperationen im Generator.
- Vermischung mit Laufzeit-Speicherdaten oder Transaktionssystemen.
- Verwendung von `contextra-core`.

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Konsumiert:** `contextra-types`.
- **Bereitgestellt für:** CLI-Administration und Governance-Pipelines.

## 6. Tests, Sabotage & Qualitätssicherung
- **Template-Konsistenztests:** Validierung der Platzhalter-Ersetzung und Schema-Prüfung.
- **Sabotage-Testing:** Prüfung auf sauberes Fehlerverhalten bei fehlenden Pflichtfeldern.

## 7. Offene Punkte & Technische Schulden
- Keine.

# AGENTS.md — `contextra-privacy`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-privacy` ist ein autonomer Sicherheits- und Filtering-Baustein (Ring 3) für Cloud-Egress-Schutz, Data Loss Prevention (DLP), Anonymisierung/Surrogate-Vaulting und Bulk-Exfiltrations-Erkennung.
Er schützt sensible Daten vor unerlaubtem Abfluss in externe Netze, indem er ausgehende Payloads klassifiziert, maskiert und volumetrisch überwacht.
Als orthogonaler Sicherheits-Baustein implementiert er `EgressPolicy` aus `contextra-ports` und kann vor beliebige Netzwerk- oder Modell-Aufrufe geschaltet werden.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 3 (Egress Filtering, Anonymisierung & DLP)
- **Klasse:** Erweiterung `[E]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-ports`, `contextra-types`, `contextra-crypto` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **Strikte Speichersicherheit:** `#![forbid(unsafe_code)]` wird über das gesamte Crate hinweg erzwungen.
- **Fail-Closed Prinzip:** Bei Timeouts, Filter-Fehlern, unklaren Klassifikationen oder Regex-Fehlern blockiert das Gateway ausgehende Payloads strikt (`Block(...)`).
- **Surrogate Vaulting:** Anonymisierte Entitäten werden kryptografisch versiegelt und reversibel nur innerhalb autorisierter Session-Grenzen gehalten.
- **Zero-Panic-Doktrin:** Vollständiger Ausschluss von `unwrap()` und `expect()` in allen DLP- und Klassifikations-Pfaden.

## 4. Verbotene Muster
- Jegliche Verwendung von `unsafe`.
- Fail-Open Verhalten bei Sicherheitsprüfungen.
- Vermischung von DLP-Filterung mit Speicher- oder Transaktions-Logik.
- Verwendung von `contextra-core` (statdessen direkte Nutzung von `contextra-types` und `contextra-ports`).

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Implementiert:** `EgressPolicy`, `EgressGateway`.
- **Konsumiert:** `Clock`-Port, Krypto-Primitiven aus `contextra-crypto`.
- **Bereitgestellt für:** `contextra-mcp`, `contextra-router`, `contextra`.

## 6. Tests, Sabotage & Qualitätssicherung
- **DLP Regex Set Fuzzing:** Testen des Erkennungs-Sets gegen hochgradig permutierte sensible Muster (IBAN, Passwörter, API-Keys).
- **Exfiltrations-Simulation:** Sabotagetests mit volumetrischer Überschreitung zur Prüfung des Sliding-Window Rate-Limiters.
- **Fail-Closed Verifikation:** Gezielte Injektion von Timeouts zur Bestätigung der strikten Blockierung.

## 7. Offene Punkte & Technische Schulden
- Keine.

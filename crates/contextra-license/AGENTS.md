# AGENTS.md — `contextra-license`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-license` stellt eine vollständig offline-fähige, kryptografische Lizenz- und Aktivierungsprüfung für optionale Enterprise- und Feature-Ringe bereit (Ring 4).
Als autarker Baustein implementiert er den Port `LicenseGate` (`SignedLicenseGate`) aus `contextra-ports`.
Er arbeitet strikt ohne Netzwerk-Egress ("Phone-Home"-Logik ist verboten), erzwingt Ed25519-Signaturen und gewährleistet nicht-destruktive Degradation (Fast-Ring bleibt stets unberührt).

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 4 (Erweiterung & Feature-Gate)
- **Klasse:** Erweiterung `[E]`
- **Reifegrad:** `experimental`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-crypto`, `contextra-ports` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **Strikte Speichersicherheit:** `#![forbid(unsafe_code)]` wird über das gesamte Crate hinweg erzwungen.
- **Offline-Only:** Keine Netzwerkaufrufe oder externe Zertifikatsvalidierungen; Prüfung erfolgt rein über offline Signaturprüfungen.
- **Nicht-destruktive Degradation (INV-LICENSE-2):** Standard-Funktionen (`FeatureRing::Fast`) laufen stets ohne Lizenzprüfung.
- **Fail-Closed Verification Pipeline:** 7-stufige kryptografische Validierung; fehlerhafte Lizenzen verweigern erweiterte Ringe deterministisch.
- **Zero-Panic-Doktrin:** Vollständiger Ausschluss von `unwrap()` und `expect()`; Fehler werfen typisierte `LicenseError`.

## 4. Verbotene Muster
- Jegliche Verwendung von `unsafe`.
- Netzwerkaufrufe, DNS-Abfragen oder Phone-Home Telemetrie.
- Destruktives Blockieren von Kern-Speicherfunktionen bei abgelaufener Lizenz.
- Verwendung von `contextra-core` (statdessen direkte Nutzung von `contextra-types` und `contextra-ports`).

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Implementiert:** `LicenseGate`-Port.
- **Konsumiert:** `Clock`-Port, Ed25519-Signaturprüfer aus `contextra-crypto`.
- **Bereitgestellt für:** `contextra`, `contextra-mcp`.

## 6. Tests, Sabotage & Qualitätssicherung
- **Negative Matrix Tests:** Validierung gegen abgelaufene, manipulierte oder ungültig signierte Lizenzschlüssel.
- **Clock Expiration Boundary Tests:** Zeitstempel-Prüfungen an den exakten Gültigkeitsgrenzen über den `Clock`-Port.
- **Sabotage-Testing:** Prüfung auf fehlerfreie Nicht-Störung von Basisfunktionen bei ungültigen Schlüsseln.

## 7. Offene Punkte & Technische Schulden
- Keine.

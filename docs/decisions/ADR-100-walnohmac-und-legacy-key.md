# ADR-100: Semantik von WalNoHmac und Härtung des WAL Legacy-Schlüssels

* **Status:** Proposed
* **Datum:** 2026-09-29
* **Kontext / Auslöser:**
  Im Rahmen der K02-Härtung des WAL-Subsystems wurden Integritätslücken identifiziert:
  1. Der Legacy-Obfuscation-Schlüssel (`legacy_integrity_key`) ist im Quellcode öffentlich. Um Downgrade-Angriffe und unbeabsichtigte Nutzung zu verhindern, muss sichergestellt sein, dass der Legacy-Migration-Pfad atomar, crash-sicher und nach dem Schreiben des Migration-Markers irreversibel geschlossen ist.
  2. Der Performance-Profil-Modus `DurabilityMode::WalNoHmac` ist produktiv erreichbar. Replay- und Integritätsprüfungen müssen für unverkettete Zero-HMAC-Einträge definiert sein, wobei Einzelsummen (CRC32) weiterhin validiert werden und gemischte Segmente abgelehnt werden.
  3. Bei der Erzeugung von Sidecar-Dateien (`.wal_integrity_key`, `.uuid`) existierten Rename-Race-Bedingungen unter hoher Concurrency.

## Entscheidung
1. **Atomare Legacy-Migration:** `Wal::open_for_legacy_migration` führt eine einmalige Rekeying-Migration durch, schreibt eine neue V3-WAL signed mit einem frisch erzeugten, dateilokalen Schlüssel `.wal_integrity_key` und setzt atomar den Marker `.rekeyed`. Sobald der Marker existiert, öffnet `Wal::open_for_legacy_migration` das WAL über den regulären sicheren Pfad ohne Rückfalloption auf den Legacy-Schlüssel.
2. **`WalNoHmac`-Semantik:** In `DurabilityMode::WalNoHmac` wird keine HMAC-Hash-Kette fortgeführt (Zero-Key/Zero-Prev-HMAC), aber jeder WAL-Eintrag behält seine individuelle CRC32-Prüfsumme. Der Replay-Mechanismus verifiziert bei unverketteten Einträgen die CRC32-Einzelsummen und weist beschädigte oder gemischte Segmente ab.
3. **Erzeugung von Sidecar-Dateien:** Sidecar-Dateien (`.wal_integrity_key` und `.uuid`) werden race-frei mittels `hard_link` (oder `create_new(true)`) erzeugt. Parallele Erzeuger konkurrieren atomar; der Verlierer liest die von dem Gewinner erstellte Datei ohne Überschreiben.
4. **Dokumentation der Schutzgrenzen:** Der dateilokale `.wal_integrity_key` schützt gegen Bitrot und Speicherfehler, jedoch nicht gegen aktive Verzeichnis-Manipulationen. Vollständige kryptografische Authentizität erfordert die Schlüsselableitung aus der Passphrase via `KeyManager`.

## Begründung
Diese Maßnahmen schließen die verbliebenen Integritätslücken im WAL-Subsystem, stellen Crash-Sicherheit bei der Migration sicher und verhindern Rennbedingungen bei der Sidecar-Dateierstellung.

## Konsequenzen
- Integrations- und Crash-Tests (`tests/wal_legacy_migration_crash.rs`, `tests/wal_no_hmac_replay.rs`) verifizieren die Replay-Sicherheit und Atomarität.
- Die Dokumentation in `src/wal/hmac.rs` stellt die Sicherheitsgrenzen klar.

Eigenständiger Verifizierer für Löschberichte und Feldscans.

## Übersicht

`ctx-verify` ist eine eigenständige Binärdatei und Prüfbibliothek nach SPEC §8.17, §11.7, Anhang D, F.9 und F.10.
Sie verifiziert kryptografisch signierte Löschberichte und führt unabhängige Feldprüfungen (MK1, MK2, MK3) durch.

Die Prüfung läuft vollständig offline ohne Netzwerk, ohne Telemetrie und ohne Secrets in Ausgaben. Pfade im Datenverzeichnis werden nach HR-4 niemals im Klartext ausgegeben, sondern als kryptografische Hashes referenziert.

## Ausgabecodes

| Befehl | Ausgabe | Exit-Code | Bedeutung |
|---|---|---|---|
| `report` | `VALID` | 0 | Bericht, Signatur, Kette, Positivkontrollen und Restrisiken vollständig gültig |
| `report` | `INVALID(missing_key)` | 2 | Öffentlicher Ed25519-Schlüssel fehlt oder ist ungültig |
| `report` | `INVALID(io_error)` | 2 | Datei konnte nicht gelesen werden oder ist zu groß |
| `report` | `INVALID(non_canonical_json)` | 2 | JSON verletzt F.10 (kein NFC, falsche Sortierung, Whitespace, Floats) |
| `report` | `INVALID(bad_signature)` | 2 | Ed25519-Signatur ungültig oder passt nicht zum Schlüssel/Inhalt |
| `report` | `INVALID(broken_hash_chain)` | 2 | Journal-Hash-Kette nach F.9 gebrochen |
| `report` | `INVALID(missing_positive_control)` | 2 | Ein Zeuge (`LayerWitness`) besitzt keine gültige Positivkontrolle (P1) |
| `report` | `INVALID(witness_matches_nonzero)` | 2 | Ein Zeuge meldet Residuen (`matches != 0`) |
| `report` | `INVALID(incomplete_residual_risks)` | 2 | Restrisikoliste unvollständig gegen Katalog Anhang D |
| `report` | `INVALID(temporal_order_violation)` | 2 | Zeitstempel nicht monoton (`requested <= gated <= destroyed <= completed`) |
| `report` | `INVALID(ledger_mismatch)` | 2 | Ledger-Eintrag weicht vom Bericht ab |
| `scan` | `VALID` | 0 | Feldprüfung erfolgreich (MK1, MK2, MK3 bestätigt) |
| `scan` | `INVALID(passphrase_fd_missing)` | 2 | Passphrase-Dateideskriptor ungültig oder nicht lesbar |
| `scan` | `INVALID(mk1_structure_residue)` | 2 | Strukturscan MK1 findet Zielreste in Manifest/Index/Tabellen |
| `scan` | `INVALID(mk2_raw_residue)` | 2 | Rohscan MK2 findet Muster/Klartext im Speicherbereich |
| `scan` | `INVALID(mk3_key_present)` | 2 | Schlüsselprüfung MK3 stellt fest, dass Zielschlüssel noch vorhanden ist |
| `scan` | `INVALID(io_error)` | 2 | I/O-Fehler beim Verzeichnisscan |

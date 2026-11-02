# WAL, Schlüssel und Krypto

Abgeleitet aus SPEC 10.1 bis 10.5, D-05, D-42, D-52, D-60, D-73, D-79, D-80, D-85, Anhang F.

- **WAL-Kette.** Jeder Frame trägt `mac_n = HMAC-SHA256(K_wal, prev_mac ‖ len ‖ seq ‖ ct ‖ aead_tag)` (F.4).
- **Torn Write.** Ein ungültiger letzter Frame wird nur verworfen, wenn alle folgenden Bytes null sind. Sonst `Corrupt` und Start verweigert (T-WAL-1).
- **Nie umschreiben.** Das WAL wird nicht editiert. Gelöscht werden ganze Segmente bis zum Flush-Punkt. Die Kette setzt über `WalFloor` mit `tail_mac` fort (D-73, T-WAL-2).
- **Kein Downgrade.** Kein Schlüssel für ältere WAL-Formate, kein Integritäts-Opt-out (D-05).
- **Nonces.** Datendateien: `0x00000000 ‖ u64(Blockzähler)` unter zufälligem Dateischlüssel. Record-Envelope und Keyslot-Wrap: 96 Bit aus dem OS-CSPRNG. Höchstens 2³² Blöcke je Datei (D-60).
- **AAD.** Bindet Dateityp, `file_id`, Blockindex und bei Appendlogs den Vorgänger-MAC. Umordnen und Kreuzkopieren müssen scheitern (T-ENC-2).
- **Schlüsselhierarchie.** `K_master` ist zufällig und in Keyslots gewickelt (D-52). `K_sign` ist nur abgeleitet (D-80). Löschung über Slots `T`, `C`, `S`, `D` und die Dateischlüssel `F` (D-79, D-85).
- **Algorithmen.** Satz v1: AES-256-GCM-SIV, HMAC-SHA256, HKDF-SHA256, BLAKE3 keyed, Argon2id, Ed25519. `alg_id = 2` ist reserviert und nicht implementiert (D-80).
- **Header.** Jeder Header trägt `format_version` und `alg_ids`. Unbekannte Werte führen zu `Corrupt` (I-6, D-42).

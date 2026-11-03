# Referenz: Sicherheits-Checkliste

Nicht normativ. Maßgeblich: [`docs/SPEC.md`](../../../../../docs/SPEC.md), Kapitel 10 und 13.

## Schutzziele

1. **Vertraulichkeit im Ruhezustand.** Alle Daten sind mit AES-256-GCM-SIV verschlüsselt. Keine Ausnahme (D-05).
2. **Integrität und Bindung.** Jeder Chiffrat-Block trägt eine AAD mit Dateityp, `file_id` und Blockindex. Umordnen und Kreuzkopieren müssen scheitern (T-ENC-2).
3. **Mandantentrennung.** Kein Datenaustausch über Mandantengrenzen. Jede Funktion benötigt einen `TenantContext` (D-07).
4. **Schlüsselresilienz.** Schlüsselmaterial wird beim Verwerfen genullt (`zeroize`). Die Prozesshärtung verhindert Core-Dumps und Schlüsselseiten im Dump (D-63).
5. **Fail-closed.** Bei Konfigurationsfehlern oder Krypto-Fehlern wird die Operation abgelehnt.
6. **Blinde Tags.** Schlüssel auf Platte und in Indizes sind keyed Hashes mit BLAKE3 (D-09, D-21), keine Klartext-Kennungen.

## Parameter (SPEC 10, Anhang C, F.2)

| Parameter | Wert | Quelle |
|---|---|---|
| Argon2id `m` | 64 MiB bis 4 GiB | D-06 |
| Argon2id `t` | 3 bis 16 | D-06 |
| Argon2id `p` | 1 bis 16 | D-06 |
| Argon2id Standard | `m` 128 MiB, `t` 3, `p` 4 | D-68 |
| AEAD | AES-256-GCM-SIV (`alg_id = 1`) | D-80 |
| Nonce Records, Keyslots | 96 Bit aus dem OS-CSPRNG | D-60 |
| Nonce Blöcke | `0x00000000 ‖ u64(Blockzähler)` unter Dateischlüssel | D-60 |
| Salt | 32 Byte | 10.1, C |
| Tag (keyed BLAKE3) | 16 Byte | 9.3, C |
| Signatur Berichte | Ed25519 | D-13, D-80 |
| Hash-Ketten | HMAC-SHA256 (WAL), SHA-256 (Journal, Ledger) | F.4, F.9 |

## Prüfpunkte je Änderung

- Ist jede neue Funktion mit `TenantContext` aufrufbar?
- Verschlüsselt der neue Schreibpfad über den Record-Layer und den Block-Layer?
- Enthält die neue AAD Dateityp, `file_id` und Blockindex?
- Gibt es einen neuen Fehlertext mit Nutzerinhalt? (verboten, I-11)
- Wird ein Schlüssel als `Clone` oder mit `Debug` ausgegeben? (verboten)

# Contextra Security & Cryptography Checklist

> **Normatives Referenzdokument zur Durchsetzung der Sicherheitsarchitektur gemäß docs/SPEC.md §10.**

## 1. Schutzziele

1. **Vertraulichkeit**: Alle Daten im Ruhezustand (Data at Rest) sind mittels AEAD (AES-256-GCM-SIV) verschlüsselt. Pflicht ohne Opt-Out (`EncryptionMode::Required`, D-05).
2. **Authentizität**: Jedes Chiffrat besitzt eine kryptografische AAD-Bindung (`file_id ‖ block_index ‖ format_version` bzw. Envelope AAD).
3. **Mandantentrennung (VETO-F10)**: Kein Datenaustausch oder statistischer Leak über Mandantengrenzen. Pflicht ohne Opt-Out (`TenantPolicy::Required`, D-07).
4. **Schlüsselresilienz**: Schlüsselmaterial verbleibt im durch `mlock` gesicherten Speicher und wird beim Verwerfen genullt (`Zeroizing<T>`).
5. **Fail-Closed**: Bei Fehlkonfiguration oder Krypto-Fehlern wird die Operation abgelehnt, anstatt stillschweigend unverschlüsselt fortzufahren.
6. **Blinde Tags (D-31)**: Schlüssel auf Platte und in Indizes enthalten keine Klartext-Identifier, sondern blinde Hashes via `blind_tag` (Keyed-BLAKE3).

## 2. Parameter-Vorgaben (SPEC §10)

| Parameter | Normativer Wert | Begründung |
|---|---|---|
| Argon2id Memory Cost ($m$) | $\ge 64\,\text{MiB}$ ($65.536\,\text{KiB}$) | Schutz gegen GPU/ASIC-Brute-Force |
| Argon2id Time Cost ($t$) | $\ge 3$ Iterationen | Ausreichender Rechenaufwand |
| Argon2id Parallelism ($p$) | $\ge 1$ Lane | Vorhersagbarkeit im Threadpool |
| AEAD-Verfahren | AES-256-GCM-SIV | Nonce-Misuse-Resistant AEAD |
| AEAD Nonce-Länge | 96 Bit (12 Bytes) aus `OsRng` | CSPRNG-Zufall |
| Ed25519 Schlüssellänge | 256 Bit (32 Bytes) | Asymmetrische Signatur für Löschberichte |
| Salt Länge | 256 Bit (32 Bytes) | Schutz gegen Rainbow-Tables & Multi-Target-Angriffe |
| Blind-Tag Länge | 128 Bit (16 Bytes) | Keyed BLAKE3 mit Injektivitäts-Präfixen |

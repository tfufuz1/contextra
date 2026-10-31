# Contextra Security & Cryptography Checklist

Normatives Referenzdokument zur Durchsetzung der Sicherheitsarchitektur.

## 1. Schutzziele

1. **Vertraulichkeit**: Alle Daten im Ruhezustand (Data at Rest) sind mittels AEAD (AES-256-GCM-SIV oder XChaCha20-Poly1305) verschlüsselt.
2. **Authentizität**: Jedes Chiffrat besitzt eine kryptografische AAD-Bindung an Mandant, Collection, Dokument-ID, Schlüssel-Generation und Format-Version.
3. **Mandantentrennung (VETO-F10)**: Kein Datenaustausch oder statistischer Leak über Mandantengrenzen.
4. **Schlüsselresilienz**: Schlüsselmaterial verbleibt im durch `mlock` gesicherten Speicher (`contextra-sys::vault`) und wird beim Verwerfen genullt (`ZeroizeOnDrop`).
5. **Fail-Closed**: Bei Fehlkonfiguration oder Krypto-Fehlern wird die Operation abgelehnt, anstatt stillschweigend unverschlüsselt fortzufahren.

## 2. Parameter-Vorgaben

| Parameter | Mindestwert | Begründung |
|---|---|---|
| Argon2id Memory Cost | 64 MiB (65536 KiB) | BSI TR-02102 Empfehlung gegen GPU-Brute-Force |
| Argon2id Time Cost | 3 Iterationen | Ausreichender Rechenaufwand |
| Argon2id Parallelism | 1 Lane | Determinismus & Vorhersagbarkeit im Threadpool |
| DocDek Schlüssellänge | 256 Bit (32 Bytes) | Schutz gegen Kryptoanalyse |
| Ed25519 Schlüssellänge | 256 Bit | Asymmetrische Signatur für Beweisprüfung |
| Salt Länge | 128 Bit (16 Bytes) | Schutz gegen Rainbow-Tables |

# Externe Kryptoprüfung (Gate G-K) – Vorbereitungsdossier

Gemäß **SPEC §3.1, §8.6, §10, D-05, D-06, D-52, D-60, D-80, D-130, F.1–F.3, F.6, F.10**:

Dieses Dossier dient der Vorbereitung der externen kryptografischen Überprüfung für das Release 1.0.0 (Gate G-K) bezüglich `contextra-crypto` (Ring 1).

---

## 1. Algorithmensatz v1 (SPEC D-80)

Es werden ausschließlich standardisierte, moderne Kryptoprimitive eingesetzt. Es gibt keine Algorithmen-Agilität zur Laufzeit, keine benutzerdefinierten Schalter und keine Klartext-Fallback-Pfade (D-05, P5).

| Algorithmus | Standard / Referenz | Verwendung in Contextra | Algorithmus-ID / Kennung |
|---|---|---|---|
| **AES-256-GCM-SIV** | RFC 8452 | Record-Envelope (F.6), Block-AEAD (F.1), Keyslot-Wrap (F.2) | `alg_id = 1` |
| **Argon2id** | RFC 9106 | Password-Based Key Derivation (KEK-Ableitung) | `kdf_id = 1` |
| **HKDF-SHA256** | RFC 5869 | Schlüsselhierarchie-Ableitung (`K_master` → Subkeys) | Standard-Labels (Anhang C) |
| **HMAC-SHA256** | RFC 4231 | WAL-Frame-Integritätskette, Ledger-MAC, Header-MAC | Normativ |
| **BLAKE3 (keyed)** | BLAKE3 Spezifikation | Index-Tags, Mandanten- und Dokumentidentifikatoren | `K_idx` (16 Bytes Ausgabe) |
| **Ed25519** | RFC 8032 (`verify_strict`) | Löschberichtssignatur (F.10, D-13, D-130) | `K_sign` abgeleitet via HKDF |

*Hinweis zu `alg_id = 2`:* ChaCha20-Poly1305 ist für zukünftige Protokollversionen reserviert und in v1 **nicht implementiert**. Das Parsen oder Vorfinden von `alg_id = 2` führt deterministisch zum Fehler `CryptoError::Corrupt`.

---

## 1.1 Berichtssignatur-Port & Standardimplementierung (`DerivedReportSigner`, D-130, F.10, T-SIG-1)

Die Signierung von Löschberichten ist hexagonal über den Port `ReportSigner` (`contextra-ports`) abstrahiert:
- **Standardimplementierung (`DerivedReportSigner`):** Signiert mit `K_sign = HKDF-Expand(K_master, "ctx/sign/v1", 32)`.
- **Signer-ID (`signer_id`):** `0` (abgeleiteter interner Schlüssel `K_sign` per SPEC F.10). Signer-ID `1` ist für externe Signierer / HSMs reserviert.
- **Signatur-Domain:** `"ctx/report/v1" ‖ kanonische_Bytes_ohne_signature`.
- **Signatur-Format:** 64 Bytes Ed25519, unpadded Base64url-kodiert (`signature_base64`).
- **Verifizierungs-Fingerprint:** SHA-256 des 32-Byte Verifying-Keys, erste 16 Bytes / 32 Hex-Zeichen (`pub_fingerprint`).

---

## 2. KDF-Parameter & Speichergrenzen (Argon2id, D-06, D-68, FM-16, T-KDF-1/2)

Um Denial-of-Service-Angriffe durch manipulierte Header vor der Authentifizierung abzuwehren (CWE-770), werden alle KDF-Parameter **vor** jeder Speicherallokation oder Berechnung strikt validiert.

### Parametergrenzen
- **Speicher `m`:** $64\text{ MiB} \le m \le 4\text{ GiB}$ ($65\,536 \le m_{\text{kib}} \le 4\,194\,304$)
- **Iterationen `t`:** $3 \le t \le 16$
- **Parallelismus `p`:** $1 \le p \le 16$
- **Standard (`DEFAULT`):** $m = 128\text{ MiB}$ ($131\,072\text{ KiB}$), $t = 3$, $p = 4$

### Fehlerbehandlung & Herkunft
- **Header-Ursprung (`from_header`):** Bei Überschreitung oder Unterschreitung der Grenzen scheitert der Aufruf mit `CryptoError::Corrupt`.
- **Konfigurations-Ursprung (`from_config`):** Bei unzulässigen Werten im Builder scheitert der Aufruf mit `CryptoError::Config`.

---

## 3. Nonce-Schema & Eindeutigkeit (SPEC D-60, 10.3, T-NONCE-1)

Nonces werden strikt domänenspezifisch generiert und verhindern Nonce-Reuse:

1. **Block-AEAD (SSTables, WAL, Steuerdateien, F.1):**
   - Nonce-Format (12 Bytes / 96 Bits): `0x00000000 ‖ u64_be(block_index)`.
   - Zähler-Limit: Höchstens $2^{32}$ Blöcke je Datei. Bei Erreichen von $2^{32}$ verweigert `BlockSealer::seal` weitere Verschlüsselungen mit `CryptoError::Limit`, was eine Dateirotation erzwingt.
   - Header-Tag-Nonce (Kopfzeilen-Tag): `0xFFFFFFFF ‖ 0x0000000000000000`.

2. **Record-Envelope v2 (Dokumente, F.6):**
   - Nonce-Format (12 Bytes): Kryptografisch sichere 96-Bit-Zufalls-Nonce aus dem OS-CSPRNG (`getrandom` / `rand::rngs::OsRng`).

3. **Keyslot-Wrap (Master-Key-Wrapping, F.2):**
   - Nonce-Format (12 Bytes): 96-Bit-Zufalls-Nonce aus dem OS-CSPRNG je Slot-Erzeugung.

---

## 4. AAD-Bindung (Associated Data Binding, D-52, F.1, F.2, F.6, T-KEYSLOT-4, T-ENC-2)

Alle Chiffretexte sind kryptografisch an ihren Kontext, Ort und ihre Version gebunden:

1. **Keyslot AAD (F.2):**
   $$\text{AAD} = \text{"ctx/slot/v1"} \parallel \text{store\_id (16 B)} \parallel \text{slot\_id (1 B)} \parallel \text{anchor\_id (1 B)} \parallel \text{kdf\_id (1 B)} \parallel \text{u32\_be}(m) \parallel \text{u8}(t) \parallel \text{u8}(p) \parallel \text{slot\_salt (32 B)}$$
   - *Sicherheitsgarantie:* Ein kopierter Keyslot in einen anderen Store oder manipulierte KDF-Parameter (Herabsetzungsversuch) schlagen bei `unwrap_master` mit `CryptoError::Auth` fehl.

2. **Block-AEAD AAD (F.1):**
   $$\text{AAD} = \text{kind (1 B)} \parallel \text{file\_id (16 B)} \parallel \text{u64\_be}(block\_index) \ [ \parallel \text{prev\_hash (32 B)} \ ]$$
   - *Sicherheitsgarantie:* Vertauschen von Blöcken innerhalb einer Datei, Verschieben zwischen Dateien oder Appendlog-Reorderings werden sofort erkannt.

3. **Record Envelope AAD (F.6):**
   $$\text{AAD} = \text{"ctx/rec/v2"} \parallel \text{key\_bytes}$$
   - *Sicherheitsgarantie:* Schlüssel und Dokument-Identität sind kryptografisch im AEAD-Tag verankert.

---

## 5. Keyslot-Protokoll & Lebenszyklus (D-52, T-KEYSLOT-1..4)

- Ein Speicher enthält 1 bis 4 Keyslots im `HEADER` (Exemplare A/B mit Generationszähler, 1024 Bytes groß).
- **Passphrasenwechsel in $\mathcal{O}(1)$:** Ein neuer Slot wird hinzugefügt, indem derselbe `K_master` unter einem neuen KEK (neue Passphrase, neues Slot-Salt) gewickelt wird. Die Nutzdaten der Datenbank verbleiben unberührt.
- **Entfernung / Löschung:** Der alte Slot wird mit Nullen überschrieben (Status = 2) und die Header-Generationsnummer inkrementiert.

---

## 6. Schlüsselhierarchie & Vernichtbarkeit (SPEC 10.1, D-79, T-KEYCOV-1)

- `K_master` (32 Bytes, OS-CSPRNG)
  - $\to$ `K_enc = HKDF(K_master, "ctx/enc/v1")`
  - $\to$ `K_idx = HKDF(K_master, "ctx/idx/v1")`
  - $\to$ `K_unit = HKDF(K_master, "ctx/unit/v1")`
  - $\to$ `SignSeed = HKDF(K_master, "ctx/sign/v1")` (wird nie persistiert)
  - $\to$ `K_wal = HKDF(K_master, "ctx/wal/v1")`
  - $\to$ `K_ledger = HKDF(K_master, "ctx/ledger/v1")`
  - $\to$ `K_hdr = HKDF(K_master, "ctx/hdr/v1")`
  - $\to$ `K_commit(epoch) = HKDF(K_master, "ctx/commit/v1" ‖ epoch)`

### DEK-Ableitung (Dokumentenebene per D-79):
$$\text{salt} = \text{SHA-256}(\text{"ctx/dek-salt/v2"} \parallel \text{salt}_T \parallel \text{salt}_C \parallel \text{salt}_S \parallel \text{salt}_D)$$
$$\text{DEK} = \text{HKDF-Expand}(K_{\text{unit}}, \text{"ctx/dek/v2"} \parallel \text{salt} \parallel \text{doc\_tag})$$
- *Vernichtbarkeit:* Durch das Nullen eines der Salts $T$ (Mandant), $C$ (Sammlung), $S$ (Betroffener) oder $D$ (Dokument) im Tresor wird die DEK-Ableitung unwiderruflich unmöglich.

---

## 7. Speicherhygiene & Seitenkanalschutz

- Alle Schlüssel- und Secret-Typen (`MasterKey`, `SecretKey<N>`, `Secret`, `FileKey`, `Dek` etc.) implementieren `Zeroize` und `ZeroizeOnDrop`.
- Formatierung über `Debug` und `Display` gibt ausnahmslos `[REDACTED]` aus; kein Schlüsselbyte gelangt in Protokolle oder Fehlermeldungen (T-LOG-1, HR-4).
- Alle kryptografischen Vergleiche für MACs, Tags und Prüfsummen erfolgen über `subtle::ConstantTimeEq` in garantierter Konstantzeit.

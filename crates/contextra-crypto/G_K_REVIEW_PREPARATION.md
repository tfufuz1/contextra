# Vorbereitungsdokument zur externen Kryptoprüfung (Gate G-K)

Gemäß SPEC 4.7 (D-130), SPEC 8.6, SPEC Kapitel 10 (10.1 bis 10.5) und Anhang F fasst dieses Dokument die kryptografischen Entwurfsentscheidungen, Protokolle, Nonce-Schemata und AAD-Bindungen für das Audit unter Gate G-K zusammen.

---

## 1. Algorithmus-IDs und Standards (SPEC D-80, Anhang F.1)

Das Crate `contextra-crypto` implementiert ausschließlich anerkannte Standardprimitive und schließt selbstgebaute Kryptografie sowie dynamische Schalter zur Laufzeit aus.

| Algorithmus-Klasse | Algorithmus | Kennung / Alg-ID | Normativer Standard | Einsatzbereich |
|---|---|---|---|---|
| **AEAD** | AES-256-GCM-SIV | `alg_id = 1` | RFC 8452 | Records (F.6), Keyslots (F.2), Block-AEAD (F.1) |
| **AEAD (Reserviert)** | ChaCha20-Poly1305 | `alg_id = 2` | RFC 8439 | In v1 **nicht implementiert**, Header mit `alg_id = 2` werden als `Corrupt` verworfen |
| **Passwort-KDF** | Argon2id | – | RFC 9106 | Ableitung des KEK zur Entschlüsselung des Master-Keyslot |
| **Key Derivation** | HKDF-SHA256 | – | RFC 5869 | Subkey-Ableitung aus `K_master` und DEK-Ableitung |
| **MAC** | HMAC-SHA256 | `alg_id = 1` | RFC 4231 / FIPS 198-1 | WAL-Frames (F.4), Ledger-Ketten (F.9), Header-Integrität |
| **Tagging** | BLAKE3 (keyed) | `alg_id = 1` | BLAKE3 Spezifikation | Index-Tags (Dokumente, Terme, Mandanten, Kollektionen) |
| **Signatur** | Ed25519 (`verify_strict`) | – | RFC 8032 | Löschberichts-Signatur (F.10) über `ReportSigner` |

---

## 2. Nonce-Schema und Eindeutigkeitsgarantie (SPEC D-60, 10.3)

Um Nonce-Kollisionen bei AES-256-GCM-SIV absolut auszuschließen, kommen getrennte Nonce-Generierungsmodelle je Anwendungsbereich zum Einsatz:

1. **Record-Envelopes (F.6) & Keyslot-Wrap (F.2):**
   - **Länge:** 96 Bit (12 Bytes).
   - **Quelle:** Direkter Aufruf des Betriebssystem-CSPRNGs (`rand::rngs::OsRng` via `OsCsprng`).
   - **Garantie:** Kryptografisch unvorhersehbare 96-Bit-Zufallswerte je Verschlüsselungsvorgang.

2. **Block-AEAD (F.1 für SSTable, WAL, Segmente, Snapshots):**
   - **Länge:** 96 Bit (12 Bytes).
   - **Layout:** `0x00000000 (4 Bytes) || u64_be(Blockindex) (8 Bytes)`.
   - **Eindeutigkeit:** Monoton ansteigender Zähler je Dateischlüssel (`FileKey`).
   - **Harte Grenze:** Maximal 2^32 Blöcke je Datei. Beim Erreichen von `Blockindex >= 2^32` verweigert `BlockSealer::seal` weitere Verschlüsselungen und liefert fail-closed `CryptoError::Limit`.

3. **Dateikopf-AEAD-Tag (F.1 `header_tag`):**
   - **Länge:** 96 Bit (12 Bytes).
   - **Reservierter Nonce-Wert:** `0xFFFFFFFF (4 Bytes) || 0x0000000000000000 (8 Bytes)`.
   - **Zweck:** Authentisierung der Bytes `0..48` des Dateikopfs unter dem Dateischlüssel mit leerem Klartext. Durch das Präfix `0xFFFFFFFF` ist diese Nonce mathematisch disjunkt von allen regulären Datenblock-Nonces.

---

## 3. AAD-Bindungsmatrix (Context Binding, SPEC 10.3, Anhang C)

Jede Chiffretext-Operation bindet Metadaten an den Chiffretext, sodass Umordnen, Verkürzen, Substituieren oder Kreuzkopieren zwischen Partitionen, Dateien oder Mandanten zwingend scheitert:

| Kontext | Label (Anhang C) | AAD-Format / Komponenten | Zweck & Schutzwirkung |
|---|---|---|---|
| **Keyslot-Wrap** | `ctx/slot/v1` | `ctx/slot/v1 \|\| store_id(16) \|\| slot_id(1) \|\| anchor_id(1) \|\| kdf_id(1) \|\| m_kib(u32) \|\| t(u8) \|\| p(u8) \|\| slot_salt(32)` | Bindet den Keyslot fest an Store-ID, Slot-Nummer und exakte KDF-Kostenparameter. Verhindert Downgrade- und Copy-Paste-Angriffe. |
| **Record-Envelope v2** | `ctx/rec/v2` | `ctx/rec/v2 \|\| key_bytes` | Bindet die Entschlüsselung an den logischen Schlüsselpfad (Mandant, Kollektion, Doc-Ref). |
| **Block-AEAD** | – | `kind(u8) \|\| file_id(16) \|\| u64_be(Blockindex) [\|\| prev_mac(32)]` | Bindet Blöcke an Dateityp, Datei-ID und Blockposition; bei Append-Logs zusätzlich an den Vorgänger-MAC (Chaining). |
| **Dateikopf-Tag** | – | Dateikopf-Bytes `0..48` als AAD | Schützt Magic, Version, Kind, Alg-IDs, File-ID und Sequenznummer vor Manipulation. |
| **Textbindung** | `ctx/tbind/v1` | `ctx/tbind/v1 \|\| file_id(16) \|\| u64_be(doc_ord)` | Bindet Posting-Listen an den spezifischen Dokumenteintrag im Segment. |
| **Tresor-Slot-MAC** | `ctx/vault-mac/v1` | Schlüsselableitung via `HKDF(K_enc, ctx/vault-mac/v1)` | Schützt Shard-Slots gegen Fälschung. |

---

## 4. KDF-Parametergrenzen und Standardwerte (SPEC D-06, D-68, FM-16)

Zur Abwehr von Ressourcenerschöpfungsangriffen (CWE-770) bei manipulierten Header-Daten werden alle KDF-Parameter **hart vor Beginn der Ableitung** validiert:

- **Speicheraufwand (`m_kib`):** Min. 64 MiB (65.536 KiB), Max. 4 GiB (4.194.304 KiB).
- **Zeitaufwand / Iterationen (`t`):** Min. 3, Max. 16.
- **Parallelitätsgrad (`p`):** Min. 1, Max. 16.
- **Passphrasen-Mindestlänge:** Mindestens 12 Bytes (`MIN_PASSPHRASE_BYTES = 12`). Kürzere Passphrasen scheitern sofort mit `CryptoError::Config`.
- **Standard-Parameter (D-68):** `m = 128 MiB (131.072 KiB)`, `t = 3`, `p = 4`.
- **Fehlerbehandlung:** Verletzung aus Header-Parsing ergibt `CryptoError::Corrupt`, aus Konfiguration `CryptoError::Config`.

---

## 5. Keyslot-Protokoll und Lebenszyklus (SPEC D-52, F.2)

1. **A/B-Kopf-Architektur:**
   - Zwei Header-Exemplare `HEADER.a` und `HEADER.b` werden alternierend geschrieben.
   - Jedes Exemplar trägt einen monotonen Generationszähler `generation` (u64) und eine Prüfsumme `check16 = SHA-256(bytes[0..584))[0..16]`.
   - Ein Ausfall oder Crash während des Schreibens von Exemplar A lässt Exemplar B intakt (T-HDR-1).

2. **Keyslot-Aufbau (128 Bytes je Slot, max. 4 Slots):**
   - Enthält `slot_id`, `anchor_id`, `kdf_id`, `status` (0=leer, 1=aktiv, 2=entfernt), KDF-Parameter, `slot_salt` (32 B), `nonce` (12 B), `wrapped_master` (48 B).
   - `wrapped_master` schützt den 32-Byte MasterKey mit 16-Byte AEAD-Tag unter dem KEK.

3. **Passphrasenwechsel in O(1):**
   - Ein neuer Slot wird mit neuem KEK (abgeleitet aus neuer Passphrase) erzeugt und in einen freien Slot-Index geschrieben.
   - Der alte Slot wird genullt (`status = 2`).
   - Der Datenbestand (`K_master` und alle darunterliegenden DEKs) bleibt unverändert; es ist keine teure Neuverschlüsselung der Datenbank erforderlich (T-KEYSLOT-2).

---

## 6. Schlüsselhierarchie & Vernichtbarkeit (SPEC 10.1, D-79, D-80, D-85)

```
Passphrase ──Argon2id──▶ KEK ──AES-256-GCM-SIV unwrap──▶ K_master (32 B, zufällig)
K_master ─HKDF(ctx/enc/v1)──▶ K_enc ──HKDF(kind ‖ file_id)──▶ K_file
         │                           ──HKDF(ctx/fkmask/v1 ‖ file_id)──▶ Maske für F-Slots
         ─HKDF(ctx/idx/v1)──▶ K_idx (BLAKE3 Index-Tags)
         ─HKDF(ctx/unit/v1)─▶ K_unit ──HKDF(salt=SHA-256(T‖C‖S‖D), info=doc_tag)──▶ DEK
         ─HKDF(ctx/sign/v1)─▶ SignSeed (Ed25519-Signaturschlüssel, nie gespeichert)
         ─HKDF(ctx/wal/v1)──▶ K_wal (HMAC-Kette)
         ─HKDF(ctx/ledger/v1)─▶ K_ledger
```

- **Schlüsselvernichtung (Crypto-Erase):**
  Die Vernichtung der Salts `T`, `C`, `S` oder `D` im Tresor bewirkt, dass der DEK irreversibel unberechenbar wird. Da alle abgeleiteten Zwischenschlüssel unidirektional via HKDF-Expand erzeugt werden, kann aus einem Child-Key oder Chiffretext niemals auf übergeordnete Ebenen rückgeschlossen werden (T-KEYCOV-1, T-VAULT-2).
- **Dateischlüssel-Vernichtung (D-85):**
  Beim physischen Ausscheiden von Datendateien wird der zugehörige `F`-Slot im Tresor genullt. Dadurch werden eventuell verbleibende Dateiblöcke auf nicht-überschreibbaren Medien (Flash/COW) selbst für den Inhaber der Master-Passphrase unlesbar.
- **ZeroizeOnDrop:**
  Sämtliche geheimen Schlüsseltypen (`SecretKey`, `MasterKey`, `EncKey`, `IdxKey`, `UnitKey`, `SignSeed`, `WalKey`, `LedgerKey`, `HdrKey`, `CommitKey`, `VaultMacKey`, `FileKey`, `Dek`, `Secret`) implementieren `zeroize::ZeroizeOnDrop`.
- **Debug-Redaktion (T-LOG-1 / HR-4):**
  Alle `fmt::Debug`- und `fmt::Display`-Implementierungen maskieren Schlüsselmaterial ausnahmslos mit `[REDACTED]`. Keine geheimen Bytes können versehentlich in Logs oder Fehlertexte fließen.

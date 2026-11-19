# contextra-crypto — Crate-Spezifikation (Soll-Zustand, SPEC 2.1)

**Ebene:** E1 (Domäne)
**Kern-Closure:** `ids`, `env`, `ports`, `erasure` (E0, höchstens 5)
**Status:** Normativ (Soll-Zustand)
**Version:** 2.1
**Geltungsbereich:** `crates/contextra-crypto`
**Normative Quellen:** `docs/SPEC.md` §4, §5.1, §5.3 Schritte 5 und 7, §5.9 Schicht S7, §7 (`CryptoLayer`), §11, §13 (INV-11, INV-KEYPURPOSE-1, INV-PROOF-KEY-1, INV-EPOCH-SEAL-1, INV-ARTKEY-1).

---

## 1. Zweck

`contextra-crypto` ist der einzige Besitzer der kryptographischen Primitive und der Schlüsselhierarchie (R4, §4.10).

- AEAD `AES-256-GCM-SIV` für Records und für den Schlüssel-Wrap.
- HKDF-SHA-256 ausschließlich für strukturelle Schlüssel (`K_eng`, `K_sc/*`).
- Argon2id für die K1-Wurzel.
- Ed25519-Signatur der Löschberichte über den Port `Signer`, den `contextra-erasure` aufruft. Der Signaturschlüssel ist kein Datenschlüssel (INV-PROOF-KEY-1). Ed25519 ist Pflicht und kein Feature (R3).
- `KeyTable`, `KeyRegistry` und der Epoch Seal.
- Schicht **S7** (Sidecars) und `CryptoLayer` (Saga-Schritt 5).

`SubjectKey` und `SubjectDocDEK` gibt es in Version 2.1 nicht. Die Betroffenen-Einheit trägt einen zufälligen `DocDek` (§2.5, §3.2).

## 2. Architekturregeln

1. `#![forbid(unsafe_code)]`. Zero-Panic. `default = []`.
2. Keine Abhängigkeit auf E2 oder E3. Keine Abhängigkeit auf `store`, `vector`, `text`, `graph`, `db`.
3. Schlüsseltypen: `Zeroize` und `ZeroizeOnDrop`. Kein `Clone`, kein `Debug`, kein `Serialize` (INV-11).
4. Fehlender Schlüssel oder fehlgeschlagenes AEAD ergibt `Err`. Kein Klartext-Fallback, kein Null-Schlüssel, keine Notfall-Passphrase (§11.2).
5. Nonces entstehen über `OsRng::try_fill_bytes`. Ein RNG-Fehler erzeugt keinen Chiffretext.
6. Jeder Schlüssel hat genau einen Zweck und ein Label im Zweckregister (INV-KEYPURPOSE-1).
7. `aes-gcm-siv`, `sha2`, `hmac`, `argon2`, `ed25519` dürfen nur in diesem Crate als direkte Dependency stehen (§4.10).

## 3. Schlüsselhierarchie (SPEC §4.1)

```
Wurzel (Custody-Slot K1 | K2 | K3)
 └─ KEK[coll, e]        je Collection und Epoche, 32 Byte zufällig, gewrappt unter der Wurzel
     ├─ DocDek[einheit] je Betroffenen-Einheit, 32 Byte zufällig, gewrappt unter KEK
     └─ ArtKey[artefakt] je Index-Artefakt, 32 Byte zufällig, gewrappt unter KEK
K_eng                   Struktur und Integrität, HKDF aus der Wurzel, keine Nutzdaten
K_ref[tenant, e]        Referenz-Hashes, zufällig, gewrappt unter KEK
K_sc/*                  Sidecar-Zweckschlüssel, HKDF aus K_eng
K_saga[erasure_id]      ephemer, verschlüsselt den ScopeDescriptor
ErasureKey              Journal, Ledger, Berichte, getrennt von Datenschlüsseln
```

Zwischenschlüssel (`KEK`, `DocDek`, `ArtKey`, `K_ref`) werden niemals per HKDF aus einem übergeordneten Schlüssel abgeleitet. Nur die Vernichtung des konkreten Schlüssels macht die zugehörigen Chiffrate unlesbar. HKDF ist auf `K_eng` und `K_sc/*` beschränkt.

### 3.1 Custody (SPEC §4.2, §4.3)

| Stufe | Wurzel | Klasse nach Seal |
|---|---|---|
| K0 | keine Verschlüsselung, nur Entwicklung, `doctor` rot | nur L und P |
| K1 | Argon2id, Software-Keystore | C1 mit Annahme `A-FLASH` |
| K2 | TPM, HSM oder OS-Keyring, nicht exportierbar | C2 nach bestandener Konformitätssuite |
| K3 | externer KMS mit Vernichtungsattest | C3 mit Anbieter-Attest |

Das Crate implementiert den Port `EffaceableKeySlot` nur für K1. K2 und K3 sind in Version 2.1 nicht auswählbar und liefern `CapabilityUnsupported` (REQ-DEL-N14). Ein Stub, der Erfolg meldet, ist verboten (INV-WIRING-2).

`DestroyAttestation` enthält Custody, Verfahren (Zeroization), Zeitpunkt aus `Clock` und den Implementierungsbeleg. K1 liefert einen lokalen Beleg ohne Dritt-Attest.

### 3.2 Argon2id (K1)

Untergrenzen: Speicher `m ≥ 64 MiB`, Iterationen `t ≥ 3`, Parallelität `p = 1`, Laufzeit ≥ 250 ms. Die Ausgabe ist 32 Byte. Leere Passphrase ist `CryptoError::EmptyPassphrase`. Der Salt liegt in `<data_dir>/master.salt` und ist kein Geheimnis. Die Parameter sind Untergrenzen. Der Zielwert steht in `benchmarks/budgets.toml`.

### 3.3 Schlüsseltypen

`SecretKey([u8; 32])` ist der einzige Rohcontainer. Darauf sitzen getrennte Newtypes: `MasterKey`, `Kek`, `DocDek`, `ArtKey`, `ErasureKey`, `StructuralKey` (`K_eng`, `K_sc`). Ein Newtype ist nicht in ein anderes umwandelbar. `SubjectKey` wird nicht definiert.

## 4. Record- und Wrap-Format (SPEC §4.6)

- **Nutzdaten-AEAD:** `AES-256-GCM-SIV`, Nonce 96 Bit zufällig, Tag 128 Bit. AD = `scope_id ‖ doc_key_id ‖ record_version`.
- **Padding:** Padmé, Overhead höchstens 12 %. Mindestlänge `min_record_bytes` (Startwert 256).
- **Header:** `doc_key_id` und Sequenz stehen innerhalb des unter `K_eng` verschlüsselten Blocks (REQ-DEL-N15).
- **Schlüssel-Wrap:** `AES-256-GCM-SIV`, zufällige 96-Bit-Nonce, AD = `key_id ‖ kek_epoch`. Diese Nonce ist das Suchmuster des Nadelscans (§5.8).
- Jedes persistierte kryptographische Objekt trägt Algorithmus- und Formatkennung (INV-1). Ein Wechsel läuft über Re-Wrap oder Kompaktion. `doctor crypto-drill` übt das Notfallverfahren halbjährlich.

`encrypt_aead` gibt `Nonce ‖ Ciphertext ‖ Tag` zurück. `decrypt_aead` liefert bei Tag-Fehler einen einheitlichen `DecryptionFailed`. Der Lesepfad der Engine mappt einen fehlenden Schlüssel auf `Erased` (§11.1). Dieses Crate unterscheidet intern `KeyAbsent` und `DecryptionFailed`, damit die Entschlüsselungsprobe in Schritt 6 `KeyAbsent` prüfen kann.

## 5. KeyTable und KeyRegistry (SPEC §4.8)

- `KeyTable` hat Ersetzen-Semantik. Der gewrappte `DocDek` und der gewrappte `ArtKey` liegen hier, nicht in einem Append-Log.
- Nach jeder Vernichtung und nach jedem Seal wird die Segmentdatei ohne die vernichteten Einträge neu geschrieben: `tmp` → `fsync` → `rename` → Verzeichnis-`fsync` (REQ-DEL-N4). Die Primitive kommen aus `contextra-durable-fs`.
- `KeyRegistry` (`MFKR`) führt `created`, `destroyed`, `sealed` mit Referenz-Hash `BLAKE3-keyed(K_ref, scope)`. Es enthält kein Schlüsselmaterial und keinen Wrap-Blob (REQ-DEL-N5).

## 6. Rolle in der Löschsaga

Die Saga-Reihenfolge ist normativ (INV-DEL-ORDER-1). Dieses Crate handelt in Schritt 5 und Schritt 7. Ebene-A-Purge (Schritte 3 und 4) liegt davor und liegt bei den Index-Crates.

### 6.1 Schritt 5 — Schlüsselvernichtung (`CryptoLayer`)

`CryptoLayer` implementiert `ErasureLayer` aus `contextra-erasure`.

1. Nach `Frozen` gibt es keine Karenz. Die Vernichtung beginnt nach der erfolgreichen Ebene-A-Messung unmittelbar.
2. Journal-Intent J1, durabel, inklusive der 96-Bit-Wrap-Nonce des `DocDek`.
3. `DocDek` im `Zeroizing`-Speicher nullen. KeyTable-Eintrag mit Nullen überschreiben, `fsync`, Segment atomar ohne den Eintrag ersetzen.
4. `KeyRegistry` schreibt `destroyed` mit Referenz-Hash.
5. Ist der `DocDek` nach protokolliertem Intent nicht mehr in der `KeyTable`, gilt die Vernichtung als vollzogen (idempotent, T-DEL-14).
6. Der Key-Lock der Einheit wird vom Koordinator bis einschließlich dieses Schritts gehalten. Dieses Crate gibt ihn nicht früher frei.

Löschziele (SPEC §5.1):

| Ziel | Vernichtete Schlüssel |
|---|---|
| Betroffenen-Einheit | `DocDek` der Einheit |
| Betroffener | jeder `DocDek`, den der Subjekt-Index für das `SubjectRef` liefert |
| Collection | `KEK[coll, *]`, alle `DocDek` und `ArtKey` der Collection |
| Mandant | alle `KEK` des Mandanten |

`verify_key_absent` gelingt nur, wenn der Slot vollständig `0x00` ist (T-DEL-15). Ein Fehlschlag ist `ShredVerificationFailed`. Der Saga-Zustand bleibt `Frozen` (§11.1).

### 6.2 Schritt 6 — Beiträge zur Messung

Dieses Crate liefert der Messung:

- die Wrap-Nonce für den Nadelscan (0 Treffer erwartet),
- eine echte `unwrap`-Probe, die mit `KeyAbsent` endet,
- die Kanarien-Vernichtung über denselben Pfad. Die Kanarie belegt den Weg, nicht die Vernichtung des konkreten `DocDek`.

`C1` bis `C3` werden hier nicht vergeben. Vor dem Seal ist nur `C*` zulässig (INV-EPOCH-SEAL-1).

### 6.3 Schritt 7 — Epoch Seal (SPEC §4.4)

`EpochSeal` in `contextra-erasure` ruft dieses Crate auf:

1. `KEK[coll, e+1]` zufällig erzeugen und unter der Wurzel wrappen.
2. Alle lebenden `DocDek` und `ArtKey` unter `KEK[e+1]` neu wrappen. Kein Daten-Rewrite. Aufwand O(Anzahl Schlüssel).
3. `KeyTable` atomar ersetzen.
4. `KEK[e]` und seine Wrap-Kopie vernichten.
5. K1 mit `rotate_passphrase_on_seal`: neuer Argon2id-Lauf, alte Wurzel nicht mehr ableitbar.
6. K2 und K3: `rotate` und `destroy` am `EffaceableKeySlot`.

Ein Absturz mitten im Seal lässt `KEK[e]` und `KEK[e+1]` koexistieren. Recovery setzt den Re-Wrap fort (§5.5). `KeyRegistry` schreibt `sealed`.

Auslöser: `kek_epoch_len` (Start 30 Tage), `seal-now`, und `seal_after_deletion_max` (Start 7 Tage) nach einer Löschung. Löschungen derselben Collection und Epoche teilen sich einen Seal.

### 6.4 Schicht S7 — Sidecars

Sidecars enthalten Referenz-Hashes, keine Nutzinhalte und keine Schlüssel. `SidecarScanner` misst das. Ein Sidecar mit Wrap-Blob oder Payload ist ein Fehler.

### 6.5 ArtKey-Regel (INV-ARTKEY-1)

Wenn `text` oder `vector` ein Artefakt neu schreibt, vernichtet dieses Crate den alten `ArtKey` sofort, bevor der neue Artefakt-Commit als abgeschlossen gilt. Der neue `ArtKey` ist zufällig und wird unter der aktuellen `KEK` gewrappt.

## 7. Fehler (`CryptoError`)

`#[non_exhaustive]`. Mindestens:

| Variante | Bedeutung |
|---|---|
| `EmptyPassphrase` | K1-Ableitung verweigert |
| `Argon2Config`, `Argon2Derivation` | Parameter oder Ableitung |
| `HkdfExpandFailed` | nur für `K_eng` und `K_sc/*` |
| `RngFailure` | kein Chiffretext wurde erzeugt |
| `CipherInit`, `EncryptionFailed`, `DecryptionFailed` | AEAD, einheitlich nach außen |
| `CiphertextTooShort` | Blob ohne Nonce oder Tag |
| `KeyAbsent` | Unwrap eines vernichteten Schlüssels, erwartetes Ergebnis der Probe |
| `SlotNotZeroed` | `verify_key_absent` |
| `CapabilityUnsupported` | K2 oder K3 ohne Suite |
| `KeyPurposeViolation` | zweiter Zweck für einen Schlüssel |

## 8. `Cargo.toml`

```toml
[package]
name = "contextra-crypto"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "AEAD, key hierarchy, KeyTable, epoch seal, and Ed25519 report signing for Contextra"

[lints]
workspace = true

[dependencies]
contextra-ids = { workspace = true }
contextra-env = { workspace = true }
contextra-ports = { workspace = true }
contextra-erasure = { workspace = true }
contextra-durable-fs = { workspace = true }
thiserror = { workspace = true }
aes-gcm-siv = "0.11"
argon2 = { version = "0.5", default-features = false, features = ["alloc"] }
hkdf = "0.12"
sha2 = "0.10"
blake3 = { workspace = true }
ed25519-dalek = { version = "2", default-features = false, features = ["alloc", "zeroize"] }
subtle = "2.6"
zeroize = { version = "1.8", features = ["zeroize_derive"] }
rand = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
proptest = "1.4"

[features]
default = []
std = ["contextra-ids/std", "contextra-env/std"]
```

`ed25519` ist keine Feature-Weiche. Die Signatur ist immer gebaut.

## 9. Tests

| Kennung | Pflicht in diesem Crate |
|---|---|
| T-DEL-3 | Berichtsprüfung scheitert mit falschem Ed25519-Schlüssel |
| T-DEL-5 | Unter Custody K0 ist Klasse C nicht erreichbar |
| T-DEL-8 | Collection-Saga vernichtet `KEK[coll, *]` |
| T-DEL-15 | `verify_key_absent` nur nach vollständiger Nullung |
| T-DEL-21 | Nach Seal und Hygiene enthält keine Datei die Nonce eines vernichteten Schlüssels. Kein Schlüssel hat zwei Zwecke |
| T-DEL-22 | Bit-Flip und falscher Signaturschlüssel scheitern |

Zusätzlich: HKDF erzeugt keinen `DocDek`. Ein RNG-Fehler lässt `encrypt_aead` mit `RngFailure` enden und schreibt keine Bytes.

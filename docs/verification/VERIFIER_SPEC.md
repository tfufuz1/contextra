# Contextra DeletionProof Verifier — Normative Spezifikation (Version 4)

**Status:** Normativ
**Zielgruppe:** Unabhängige Dritte, externe Prüfer, Audit-Systeme
**Geltungsbereich:** Verifikation von Contextra DSGVO-Art.-17-Löschnachweisen ohne Contextra-Code oder Laufzeit-Abhängigkeiten

---

## 1. Überblick & Vertrauensmodell

Der Contextra DeletionProof-Verifier ist eine eigenständige, isolierte Referenzimplementierung zur unabhängigen mathematischen und strukturellen Prüfung von Löschnachweisen.

Er garantiert:
- **Zero Contextra Code**: Läuft vollständig ohne Rust-Toolchain, ohne Contextra-Crates und ohne externe Prozessaufrufe.
- **Fail-Closed Verifikation**: Jede Abweichung von der Spezifikation (Format, Reihenfolge, Signatur, Schlüssel) führt zur sofortigen Ablehnung.
- **Kanonische Determinismus-Garantie**: Prüft die exakte Byte-Repräsentation des signierten Bereichs nach ADR-13 / C7.

---

## 2. Format-Spezifikation (Version 4, TLV)

### 2.1 TLV Element-Struktur
Das kanonische Format basiert auf einer Stream-Sequenz von Type-Length-Value (TLV) Elementen:

```
+-------------------+-------------------+------------------------------+
| Tag (u16 BE)      | Length (u32 BE)   | Value (Length Bytes)         |
| 2 Bytes           | 4 Bytes           | Length Bytes                 |
+-------------------+-------------------+------------------------------+
```

1. **Tag**: 16-Bit Vorzeichenlose Ganzzahl in Network Byte Order (Big-Endian).
2. **Length**: 32-Bit Vorzeichenlose Ganzzahl in Network Byte Order (Big-Endian).
3. **Value**: Exakt `Length` Bytes.

### 2.2 Kritische Tags
- **Kritisches Bit**: Das oberste Bit (`0x8000`) des Tags signalisiert ein kritisches Feld:
  $$\text{is\_critical}(tag) \iff (tag \ \& \ 0x8000) \neq 0$$
- Ein Verifier **MUSS** das Verifizierungsverfahren abbrechen und den Nachweis als ungültig ablehnen (Exit Code 2), wenn ein kritisches Tag enthalten ist, das dem Verifier unbekannt ist.
- Ein unbekanntes **nicht-kritisches** Tag (oberstes Bit 0) darf ignoriert werden.

### 2.3 Kanonische Reihenfolge & Strikte Regeln
1. **Aufsteigende Tag-Sortierung**: Alle TLV-Felder im Haupt-Payload MÜSSEN in strikt aufsteigender Tag-Nummerierung angeordnet sein ($tag_1 < tag_2 < \dots < tag_n$).
2. **Keine Duplikate**: Jedes Tag darf im Haupt-Payload höchstens einmal vorkommen.
3. **Keine Trailing Bytes**: Nach dem letzten TLV-Element dürfen keine weiteren Bytes im Stream folgen.
4. **Keine abgeschnittenen Felder**: Längenangaben, die die verbleibende Eingabelänge überschreiten, werden strikt als Formatfehler abgelehnt.
5. **Größenbegrenzung**: Die Gesamteingabe darf 16 MiB (16.777.216 Bytes) nicht überschreiten.

---

## 3. Bekannte TLV-Tags für Version 4

| Tag (Hex) | Name | Typ / Struktur | Kritisch? | Beschreibung |
| :--- | :--- | :--- | :---: | :--- |
| `0x8001` | `version` | `u16 BE` (2 Bytes) | **Ja** | Format-Version. Muss exakt `4` (0x0004) sein. |
| `0x8002` | `proof_id` | `[u8; 32]` | **Ja** | Eindeutige Kennung des Löschnachweises (BLAKE3). |
| `0x8003` | `db_instance_id` | `String` (UTF-8) | **Ja** | Bezeichner der Contextra-Datenbankinstanz. |
| `0x8004` | `stage` | `u8` (1 Byte) | **Ja** | Nachweis-Stufe (`1` = Interim, `2` = Final). |
| `0x8005` | `supersedes` | `Option<[u8; 32]>` | **Ja** | Ersetzter Proof ID (0 Bytes falls `None`, 32 Bytes falls `Some`). |
| `0x8006` | `scope` | `Bytes` (bincode 1.x) | **Ja** | Bincode-serialisierter DeletionScope (Doc/Col/Tenant). |
| `0x8007` | `storage_scope` | `String` (UTF-8) | **Ja** | Lesbarer Speicherbereich (z. B. `"doc:42"`). |
| `0x8008` | `deleted_keys_hash` | `[u8; 32]` | **Ja** | Längenpräfixierter BLAKE3-Hash der gelöschten Keys. |
| `0x8009` | `footprint_hash` | `[u8; 32]` | **Ja** | Footprint-Hash der betroffenen Storage-Grenzen. |
| `0x800A` | `observed_empty` | `u8` (1 Byte) | **Ja** | `0x01` falls Bereich vor Löschung bereits leer war, sonst `0x00`. |
| `0x800B` | `deleted_after_tx` | `u64 BE` (8 Bytes) | **Ja** | Transaktions-ID der physischen Bereinigung. |
| `0x800C` | `claims` | `Bytes` (Claim-TLVs) | **Ja** | Sequenz von Schicht-Claims. Jedes Claim hat ein `u32 BE` Längenpräfix gefolgt von Sub-TLV. |
| `0x800D` | `excluded_scopes` | `Bytes` (`u16 BE` Seq) | **Ja** | Konkatenierte 2-Byte IDs explizit ausgenommener Scopes. |
| `0x800E` | `limitations_text_ids` | `Bytes` (`u16 BE` Seq) | **Ja** | Konkatenierte 2-Byte IDs von Einschränkungstexten. |
| `0x800F` | `environment_tlv` | `Bytes` | **Ja** | Umgebungs-Metadaten. |
| `0x8010` | `wal_chain_receipt` | `Option<[u8; 32]>` | **Ja** | Optionaler WAL-HMAC-Kettenbeleg (0 oder 32 Bytes). |
| `0x8011` | `chain` | `Bytes` (40 Bytes) | **Ja** | Proof-Chain Link: `seq` (`u64 BE`, 8B) + `prev_hash` (`[u8; 32]`). |
| `0x8012` | `issued_wall_ms` | `u64 BE` (8 Bytes) | **Ja** | Ausstellungs-Zeitstempel in Unix-ms. |
| `0x8013` | `issued_mono_ctr` | `u64 BE` (8 Bytes) | **Ja** | Monotoner Ausstellungs-Zähler. |
| `0x8014` | `signer_key_id` | `[u8; 32]` | **Ja** | Signer Key ID: `BLAKE3(pubkey_bytes)` (32 Bytes Ed25519 Pubkey). |

### 2.4 Sub-TLV: ProofClaimV4 (`0x800C`)
Innerhalb des `claims`-Feldes (`0x800C`) ist jedes Claim wie folgt aufgebaut:
`[claim_len: u32 BE]` + `[claim_tlv_bytes]`

Innerhalb von `claim_tlv_bytes` gelten folgende Sub-Tags:
- `0x0001` (`u16 BE`): `layer_id` (1..7, 20..26)
- `0x0002` (`u8`): `assurance` (0..4 = S0..S4)
- `0x0003` (`u16 BE`): `claim_text_id`
- `0x0004` (`[u8; 32]`): `witness_digest`
- `0x0005` (`u64 BE`): `scanned_at_tx`
- `0x0006` (`Bytes`): `evidence_tlv`

---

## 4. Rekonstruktion des signierten Bereichs & Signaturprüfung

### 4.1 Domänentrennung & Signatur-Eingabe
Die mathematische Signatur wird über die Konkatenation des Präfix-Domain-Tags und des kanonischen TLV-Payloads gebildet:

$$\text{signed\_bytes} = \text{PROOF\_V4\_DOMAIN\_TAG} \ \parallel \ \text{canonical\_tlv\_payload}$$

Wobei:
- `PROOF_V4_DOMAIN_TAG` = `b"contextra/deletion-proof/v4\0"` (28 Bytes UTF-8).
- `canonical_tlv_payload` = Die sortierten, binären TLV-Bytes aller Felder `0x8001` bis `0x8014`.

### 4.2 Kryptografische Verifikation
1. **Schlüssel-Identifikator**:
   Der Verifier berechnet den BLAKE3-Hash des übergebenen 32-Byte Ed25519 Public Keys:
   $$\text{computed\_key\_id} = \text{BLAKE3}(\text{pubkey\_bytes})$$
   Stimmt `computed_key_id` nicht mit `signer_key_id` (`0x8014`) überein, schlägt die Verifikation mit Exit Code 1 fehl.

2. **Ed25519 Signatur (RFC 8032)**:
   Die 64-Byte Ed25519 Signatur (`signature`) wird gegen `signed_bytes` und `pubkey_bytes` verifiziert:
   $$\text{Ed25519.Verify}(\text{pubkey\_bytes}, \text{signed\_bytes}, \text{signature}) \stackrel{?}{=} \text{True}$$

---

## 5. Feste Prüfreihenfolge (Prüf-Pipeline)

Der Verifier führt die Prüfung in folgender unveränderlicher Reihenfolge aus:

1. **Eingabegrößenprüfung**: Prüfen, ob Dateigröße $> 0$ und $\le 16 \text{ MiB}$ ist.
2. **Format-Erkennung & Dekodierung**:
   - Binärdatei: Direkt als TLV-Byte-Stream parsen.
   - JSON-Datei: Einlesen und Felder in kanonisches TLV bzw. Legacy-Struktur überführen.
3. **TLV-Strukturvalidierung**:
   - Vorhandensein aller Pflichtheader-Bytes.
   - Keine abgeschnittenen Längenangaben oder Payloads.
   - Keine doppelten Tags.
   - Strikte aufsteigende Tag-Sortierung.
   - Keine nachgestellten Trailing Bytes nach dem Stream.
4. **Versions- & Kritikalitäts-Check**:
   - Tag `0x8001` (`version`) muss `4` sein.
   - Prüfung aller im Stream vorkommenden Tags: Jedes Tag mit gesetztem Bit `0x8000`, das außerhalb der bekannten Spanne `0x8001`..`0x8014` liegt, führt zum Abbruch mit `FORMAT_ERROR` (Exit Code 2).
5. **Key-ID Match**:
   - Hash `BLAKE3(pubkey)` muss exakt mit `signer_key_id` (`0x8014`) übereinstimmen.
6. **Signaturverifikation**:
   - Rekonstruktion von `signed_bytes` (`PROOF_V4_DOMAIN_TAG` || `canonical_tlv`).
   - Ed25519-Verifikation der 64-Byte Signatur.

---

## 6. Fehlerkategorien & Exit-Codes

| Exit Code | Kategorie | Beschreibung | Beispiel-Ausgabe |
| :---: | :--- | :--- | :--- |
| `0` | **VALID** | Verifikation erfolgreich. | `VALID` |
| `1` | **INVALID** | Mathematische oder inhaltliche Ungültigkeit (falsche Signatur, falscher Schlüssel, manipuliertes Feld). | `INVALID: SIGNATURE_MISMATCH: Ed25519 signature verification failed` |
| `2` | **FORMAT_ERROR** | Unbekanntes kritisches Tag, unbekannte Version, falsche Tag-Sortierung, doppelte Tags, Längenfehler, nachgestellte Bytes. | `INVALID: FORMAT_ERROR: Unknown critical TLV tag encountered: 0x8999` |
| `3` | **USAGE_ERROR** | CLI-Bedienfehler (Datei nicht gefunden, ungültiger Hex-Schlüssel, fehlende Argumente). | `INVALID: USAGE_ERROR: Public key file not found` |

---

## 7. Offene Punkte & Mehrdeutigkeiten ("OFFEN")

Folgende Mehrdeutigkeiten in den Quelltexten wurden während der Spezifikation identifiziert und als `OFFEN` markiert:

1. **OFFEN (Quelltext-Inkonsistenz v4 Payload)**:
   - **Datei & Zeile**: `crates/contextra-crypto/src/deletion_proof/proof_v3.rs` Zeile 346 (`construct_v4_payload`) vs `crates/contextra-crypto/src/deletion_proof/proof_v4.rs` Zeile 125 (`ProofV4::construct_canonical_payload`).
   - **Beschreibung**: In `proof_v3.rs` ist eine Übergangsmethode `construct_v4_payload` auf der alten `DeletionProof`-Struktur definiert, welche die Felder mittels `bincode` konkateniert. In `proof_v4.rs` wird hingegen die kanonische TLV-Struktur `ProofV4` (C7, ADR-13) mit Tag-Sortierung implementiert.
   - **Festlegung für den Verifier**: Der v4-Verifier folgt ausschließlich der normativen TLV-Spezifikation nach ADR-13 aus `proof_v4.rs`.

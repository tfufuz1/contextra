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

## 7. Offene Punkte & Mehrdeutigkeiten ("GEKLÄRT")

1. **GEKLÄRT (Normative Zielkodierung TLV v4 vs. Legacy-Bincode)**:
   - **Festlegung für den Verifier**: Der v4-Verifier folgt ausschließlich der normativen TLV-Spezifikation nach ADR-13 / C7 aus `crates/contextra-crypto/src/deletion_proof/proof_v4.rs`. Bincode-basierte Hilfsmethoden in `proof_v3.rs` gelten als veralteter Übergangscode (Legacy) und sind für v4-Verifikationen unzulässig.

---

## 8. Testvektoren-Format & Normative Standards

### 8.1 Standardisiertes Testvektor-Format (JSON/Hex)
Um einen sprachunabhängigen Verifier in einer beliebigen Programmiersprache (Python, Go, C++, TypeScript) zu entwickeln und abzunehmen, werden Testvektoren im nachfolgenden JSON-Schema definiert:

```json
{
  "test_cases": [
    {
      "id": "TV-V4-VALID-01",
      "description": "Gültiger v4 DeletionProof mit strikter TLV-Sortierung und korrekter Ed25519-Signatur",
      "public_key_hex": "629f86221c54b38d3f6d711910d2382103350284f183751a02119d658252277d",
      "signature_hex": "10041d08d9292e9e8020584b422f986d4e868c2d58a82d73315a6b07c80526e8ad027df581d6f519d1dddc89e2e5058f4a3390c587fb8f3d8f8d6d634289a70f",
      "raw_tlv_hex": "80010002000480020020...",
      "expected_exit_code": 0,
      "expected_status": "VALID",
      "verification_level": 0
    },
    {
      "id": "TV-V4-INVALID-TAG-ORDER-02",
      "description": "Ungültiger Proof: TLV-Tags sind nicht strikt aufsteigend sortiert",
      "public_key_hex": "629f86221c54b38d3f6d711910d2382103350284f183751a02119d658252277d",
      "signature_hex": "...",
      "raw_tlv_hex": "80020020...800100020004",
      "expected_exit_code": 2,
      "expected_status": "FORMAT_ERROR: TAG_SORT_VIOLATION",
      "verification_level": 0
    }
  ]
}
```

### 8.2 Normative Standards & Verifikationsstufen

1. **RFC 8032 (Ed25519) inklusive Errata EID 5968**:
   - Die Verifikation der Ed25519-Signatur MUSS strikt nach RFC 8032 erfolgen. Gemäß Errata EID 5968 MÜSSEN malleable Signaturen (z. B. $S \ge L$ oder unkanonische Punktkodierungen) ohne Ausnahme als `INVALID` (Exit Code 1) abgelehnt werden.
2. **RFC 8949 §4.2 (Kanonische Binärrepräsentation)**:
   - Die Determinismus-Regeln für die serielle Bytedarstellung und sortierte Feldsequenzen richten sich nach den Grundsätzen kanonischer Binärkodierungen aus RFC 8949 §4.2.
3. **RFC 3161 (Time-Stamp Protocol - TSP)**:
   - Externe Zeitstempel-Anker für `issued_wall_ms` können optional über RFC 3161 Zeitstempel-Antworten (TimeStampResp) kryptografisch an den `proof_id`-Hash gebunden werden.
4. **RFC 9162 (Verifiable Data Structures / Certificate Transparency)**:
   - Die Verifikation von Löschnachweisen erfolgt in drei definierten Stufen:
     - **Stufe 0 (Standalone)**: Isolierte Validierung von TLV-Format, Tag-Sortierung, `signer_key_id == BLAKE3(public_key)` und Ed25519-Signatur über `contextra/deletion-proof/v4\0 || canonical_tlv`.
     - **Stufe 1 (Chain-Linked)**: Prüfung der fortlaufenden Hash-Ketten-Integrität (`chain.seq` und `chain.prev_hash` im Tag `0x8011`).
     - **Stufe 2 (Transparency Log)**: Kryptografischer Inklusionsbeweis (Merkle Audit Path nach RFC 9162) gegen ein fälschungssicheres, append-only Transparenzlog.

# ADR-N13: WAL-Frame-V4 Header-Integrität zur Absicherung gegen verdeckten Datenverlust

* **Status:** Proposed / Final-Spezifikation (Priorität H1)
* **Datum:** 2026-10-02
* **Kontext / Auslöser:**
  Bei der Sicherheits- und Integritätsanalyse der WAL-Frame-Verarbeitung wurde eine schwerwiegende Lücke im Längenpräfix-Handling der Formate V1, V2 und V3 identifiziert.
  Ein einziger Bitfehler im Längenfeld eines mittleren WAL-Eintrags führt derzeit zu verdecktem Datenverlust (Silent Data Loss), da Korruptionen fälschlicherweise als harmloser "Torn Tail" am Dateiende interpretiert werden.

---

## 1. Verifizierte Fakten & Ist-Analyse

1. **Unverifiziertes Längenpräfix (`len: u32`):**
   In den WAL-Formaten V1, V2 (`MFW2`) und V3 (`MFW3`) ist jedem Frame ein 4-Byte Little-Endian Längenpräfix `len` vorangestellt. Dieses Längenfeld liegt **weder unter der CRC32-Prüfsumme noch unter der HMAC-Integritätskette** des Eintrags.

2. **Fehlerhafte Klassifizierung als Tail-Truncation (`crates/contextra-store/src/wal/replay.rs:375-402` & `crates/contextra-store/src/wal/io.rs:107-135`):**
   Wird das Längenfeld `len` durch einen Bitfehler verfälscht (z.B. ein gekipptes Bit, das aus `len = 64` einen riesigen Wert `len = 0x80000030` macht), berechnet der Reader `pos + 4 + len > file_size`.
   Der Reader prüft folgendes:
   ```rust
   if pos + 4 + len as u64 > file_size || (pos + 4 + len as u64) as usize > slice.len() {
       if entries_count == 0 && file_size > 64 {
           return Err(ContextraError::wal_corruption(
               pos,
               format!("WAL entry length ({}) exceeds file size ({}) at start of file", len, file_size),
           ));
       }
       tracing::warn!("WAL tail corruption (partial entry) at offset {}", pos);
       break;
   }
   ```

3. **Stiller Datenverlust (Silent Commit Loss):**
   Sofern bereits mindestens ein valider Eintrag verarbeitet wurde (`entries_count > 0`), löst die Bedingung keinen Fehler (`Err`) aus, sondern gibt eine Warnung aus (`tracing::warn!`) und bricht die Replay-Schleife mit `break` ab.
   Das Speichersystem wertet dies als erfolgreiches Replay bis zum vorgeblichen "Dateiende". **Der beschädigte Eintrag sowie SäMTLICHE nachfolgenden validen Commits in dieser WAL-Datei gehen still und unbemerkt verloren.**

4. **Fehlende Bounds-Validierung vor Speicherallokation bei verschlüsselten Frames:**
   Bei verschlüsselten WAL-Einträgen wird das decrypted Byte-Array basierend auf dem unvalidierten `len`-Feld gebildet, bevor eine HMAC- oder CRC-Prüfung stattfinden kann. Ein manipulierter `len`-Wert kann zur Erschöpfung des Arbeitsspeichers (OOM Panic) führen.

---

## 2. Evaluierung der Lösungsoptionen

### Option A: WAL V4 Format mit Magic `MFW4` und geschütztem Header (*Spezifikations-Empfehlung*)
* **Funktionsweise:**
  Einführung von WAL V4 (Header `b"MFW4"`). Jeder Frame erhält einen explizit geschützten Header `[len: u32][header_crc: u32]`, wobei `header_crc = crc32c(len)`.
  Vor jedem Parse-Schritt wird erst `header_crc` validiert. Stimmt `header_crc` nicht mit `len` überein, liegt eine eindeutige Header-Korruption mitten in der Datei vor und das Replay wird **sofort mit `ContextraError::WalCorruption` abgebrochen**.
* **Vorteile:**
  - Mathematisch eindeutige Unterscheidung zwischen Header-Korruption mitten in der Datei und unvollständigem Torn Write am Dateiende.
  - Nur 4 Bytes Overhead pro Frame.
  - Schutz vor OOM-Allokationen vor der Entschlüsselung.
* **Nachteile:**
  - Benötigt WAL V4 Formatänderung.

### Option B: Frame-Trailer mit doppelter Längenangabe
* **Funktionsweise:**
  Die Frame-Länge wird sowohl am Anfang als auch am Ende des Frames geschrieben (`[len: u32][payload][len_copy: u32]`).
* **Vorteile:**
  - Erlaubt theoretisch auch Rückwärts-Scanning der WAL-Datei.
* **Nachteile:**
  - Bietet ohne Prüfsumme keine echte Korruptionserkennung (beide Längenfelder könnten unentdeckt bitflippen).
  - Höherer Platzbedarf.

---

## 3. Exaktes Byte-Layout (WAL V4)

### 3.1 Datei-Header
Jede WAL-V4-Datei beginnt mit einem 4-Byte Magic Header:
```text
+-----------------------+
| Magic: "MFW4" (4 B)   |
+-----------------------+
```

### 3.2 Frame-Layout (Unverschlüsselt & Verschlüsselt)
Jeder Frame folgt streng diesem binaeren Aufbau:
```text
+---------------------+---------------------------+-----------------------------------+
| len: u32 (4 Bytes)  | header_crc: u32 (4 Bytes) | payload: len Bytes                |
+---------------------+---------------------------+-----------------------------------+
```
* **`len`**: Länge des nachfolgenden `payload`-Puffers in Bytes (Little-Endian `u32`).
* **`header_crc`**: CRC32C-Prüfsumme berechnet exakt über die 4 Bytes von `len` (`crc32c(&len.to_le_bytes())`).
* **`payload`**:
  - Bei Klartext (Unencrypted): `[crc32: 4 B][hmac: 32 B][prev_hmac: 32 B][op_type: u8][tx_id: u64][seq_no: u64][key_len: u32][key][val_len: u32][value]`.
  - Bei Verschlüsselung (Encrypted Batch): `[nonce: 12 B][ciphertext: len - 12 B]`.

---

## 4. Leser-Zustandsmaschine & Policy-Matrix

Die Replay-Zustandsmaschine verarbeitet jeden Frame sequentiell nach folgender Entscheidungsmatrix:

```text
               +-----------------------------------+
               | Frame-Header lesen [len][crc_hdr] |
               +-----------------------------------+
                                 |
                     +-----------+-----------+
                     |                       |
            [crc_hdr valide]        [crc_hdr ungültig]
                     |                       |
                     v                       v
         +-----------------------+   +-------------------------------+
         | pos + 8 + len <= size |   | Prüfe: Ist pos am Dateiende?  |
         +-----------------------+   +-------------------------------+
           /                   \                 /               \
         Ja                     Nein           Ja                 Nein
         |                       |             |                   |
         v                       v             v                   v
 +---------------+      +----------------+  +----------------+  +-------------------+
 | Payload-Prüf. |      | Ungültiger len |  | Cleaner Tail / |  | HARD FAIL:        |
 | (CRC/HMAC)    |      | am Dateiende?  |  | Zero-Fill-Tail |  | WalCorruption     |
 +---------------+      +----------------+  | (Torn Write)   |  | (Middle Bitflip)  |
   /           \            /        \      +----------------+  +-------------------+
Valide      Ungültig       Ja        Nein
  |             |          |          |
  v             v          v          v
[OK]       [HARD FAIL]  [Clean Tail] [HARD FAIL]
```

### Policy-Matrix

| Nr. | Replay-Zustand | `header_crc` | `pos + 8 + len <= file_size` | `payload` Check | Resultat / Aktion |
| :-: | :--- | :---: | :---: | :---: | :--- |
| **1** | Valider Frame mitten in Datei | Valide | Ja | CRC & HMAC OK | **`OK`**: Commit wird replayed. |
| **2** | Bitflip im `len`-Header (Mitte) | **Ungültig** | Egal (oft riesiges `len`) | Nicht geprüft | **`HARD FAIL`**: `ContextraError::WalCorruption`. Kein Silent Drop! |
| **3** | Bitflip im `payload` (Mitte) | Valide | Ja | **CRC / HMAC Mismatch** | **`HARD FAIL`**: `ContextraError::WalCorruption`. |
| **4** | Echter Torn Write am Dateiende | **Ungültig / Truncated** | Nein | Non-existent | **`Clean Tail`**: Akzeptiert als unvollständiger Write beim Crash, wenn restliche Bytes Unwritten/Zero-Fill am Dateiende sind. |
| **5** | Zero-Fill-Tail (Page Prealloc) | `0x00000000` | Nein | Non-existent | **`Clean Tail`**: Sauberes Dateiende. |
| **6** | Verschlüsselter Frame | Valide | Ja | Decrypt & HMAC OK | **`OK`**: Vor-Allokation erst nach valide geparstem `header_crc`. |

---

## 5. Migration V3 zu V4 & Abwärtskompatibilität

1. **Abwärtskompatibles Lesen:**
   Der Parser liest den Dateianfang:
   - `b"MFW4"` -> V4 Parser (`scan_entries_v4`) mit Header-CRC-Erkennung.
   - `b"MFW3"` -> V3 Legacy Parser.
   - `b"MFW2"` -> V2 Legacy Parser.
   Standardmäßig bleiben bestehende V2/V3 WAL-Dateien nahtlos lesbar.
2. **Migration über den bestehenden Rewrite-Pfad:**
   Bei Store-Consolidation, WAL-Re-keying oder temporären Recovery-Backups (`.bak`, `recover_from_bak_if_present` in `crates/contextra-store/src/wal/replay.rs:698`) werden V2/V3 WALs automatisch in das sichere V4-Format umgeschrieben.

---

## 6. Test- & Fuzzing-Plan

1. **Spezifikationstests (`crates/contextra-store/tests/wal_spec_pending.rs`):**
   - `spec_n13_v4_frame_header_crc_bitflip_fails_hard_middle_entry`: Provoziert Bitflips in Längenfeldern mittlerer Eintrags-Header und erzwingt `ContextraError::WalCorruption`.
   - `spec_n13_v4_clean_torn_tail_distinguished_from_middle_corrupted_header`: Prüft die exakte Unterscheidung zwischen unsauberem Dateiende und Header-Korruption.
   - `spec_n13_v2_v3_to_v4_transparent_migration_and_compatibility`: Stellt Abwärtskompatibilität und automatische Konvertierung zu V4 sicher.
2. **Fuzzing Harness (`cargo-fuzz`):**
   - Ein Fuzz-Target injiziert Mutationen in zufälligen Byte-Positionen von V4-WAL-Streams, um sicherzustellen, dass niemals stille Abbrüche oder Panics (OOM) auftreten.

---

## 7. Empfehlung & Beschluss

Es wird **Option A (WAL V4 Format mit geschütztem Header `[len: u32][header_crc: u32]`)** beschlossen.
Sie beseitigt die Silent-Data-Loss-Lücke vollständig mit minimalem Overhead und garantiert strikte Data-Integrity-Compliance.

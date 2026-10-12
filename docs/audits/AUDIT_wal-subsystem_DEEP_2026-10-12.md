# Kryptografischer & Durability-Audit: Write-Ahead Log (WAL) Subsystem

> **Scope**: `crates/contextra-store/src/wal/` & `crates/contextra-store/src/lsm/recovery.rs`
> **Datum**: 2026-10-12
> **Auditor**: Principal Senior Rust Architect (Contextra)
> **Session-Hash**: `770cde6b`
> **Status**: PASS — System-Integrität und Durabilität sind kryptografisch & strukturell gewahrleistet.

---

## 1. HMAC-Chain-Verifikation (W1)

### Befund & Analyse
- **Verkettung (Chaining)**: Jeder `WalEntry` verkettet seinen HMAC (`checksum`) mit dem vorherigen Eintrag (`prev_hmac`). In `crates/contextra-store/src/wal/hmac.rs`:
  - `prepare_batch()` hält `self.last_hmac` (Zeile 156-180). Für jeden Eintrag in einer Batch wird `WalEntry::try_new(op, seq_no, &integrity_key, current_chain)` aufgerufen, wobei `current_chain` jeweils auf die `checksum` des gerade erzeugten Eintrags aktualisiert wird (Zeile 171-173).
- **HMAC-Berechnung (V3 Format)**: In `crates/contextra-store/src/wal/encode.rs`:
  - `compute_checksum_v3()` bindet `prev_hmac`, `seq_no`, `tx_id`, `op_type` sowie typisierte Längenpräfixe und Nutzlasten via `WalHmac` (Zeile 172-208) ein.
- **Initialer Chain-Seed / Rotation**:
  - Beim Erstellen oder Öffnen einer neuen WAL-Datei wird `last_hmac` auf `[0u8; 32]` bzw. den HMAC des letzten verifizierten Eintrags des bestehenden Segments initialisiert (`crates/contextra-store/src/wal/mod.rs` Zeile 622, 693-705).
  - Das Manifest in `crates/contextra-store/src/manifest/` speichert einen `WalCheckpoint` High-Water-Mark HMAC (`expected_hwm`).
  - Während `LsmStorage::new()` in `crates/contextra-store/src/lsm/recovery.rs` (Zeile 276-291) wird die Kontinuität über alle Segmente mittels `contextra_crypto::verify_wal_chain_completeness(&check_hmac, &expected_hwm)` erzwungen. Tritt eine Lücke oder unerlaubte Kürzung auf, schlägt die Recovery mit `ContextraError::WalTruncationDetected` fehl.

---

## 2. Bounds-Prüfung gegen OOM (W2)

### Befund & Analyse
- **Einlese-Grenzen (Length Header Guard)**:
  - In `crates/contextra-store/src/wal/io.rs` (`do_scan_entries_with_callback`) wird vor jeder Puffer-Allokation das Längenfeld `len` gelesen und geprüft:
    ```rust
    // crates/contextra-store/src/wal/io.rs:114-135
    if len > MAX_WAL_ENTRY_SIZE as usize {
        if pos + 4 + len as u64 > file_size {
            if entries_count == 0 && file_size > 64 {
                return Err(ContextraError::wal_corruption(...));
            }
            tracing::warn!("WAL tail corruption (huge len) at offset {}", pos);
            break;
        }
        return Err(ContextraError::wal_corruption(
            pos,
            format!("WAL entry too large ({} bytes)", len),
        ));
    }
    ```
  - `MAX_WAL_ENTRY_SIZE` ist absolut auf `64 MiB` limitiert (`crates/contextra-store/src/wal/mod.rs:252`).
  - Zusätzlich fordert `encode.rs` für Schlüssel `key_len <= 1 MiB` (`WalParseError::OversizedKey`) und für Werte `val_len <= MAX_WAL_ENTRY_SIZE` (`WalParseError::OversizedValue`), womit OOM-Allokationen durch korrumpierte Längen-Header ausgeschlossen sind.

---

## 3. CRC32-Validierung (W3)

### Befund & Analyse
- **Payload-CRC32-Verifikation**: In `crates/contextra-store/src/wal/encode.rs`:
  - Jeder serialisierte Frame besitzt einen 4-Byte CRC32 Header (`to_bytes()`, Zeile 212-259).
  - Beim Deserialisieren (`from_bytes_classified()`, Zeile 262-277) wird der gespeicherte CRC32 gegen `crc32fast::hash(payload)` abgeglichen.
  - Bei Diskrepanz wird `WalParseError::CrcMismatch` zurückgegeben.
- **Deterministische Fehlerbehandlung**:
  - In `crates/contextra-store/src/wal/replay.rs` (`handle_wal_entry_parse_error()`, Zeile 31-54):
    - Wenn ein CRC-Fehler mittendrin im Log auftritt (`pos < file_size`), wird dieser unmissverständlich als `ContextraError::WalCorruption` klassifiziert und stoppt die Ausführung (kein Undefined Behavior, kein Stillschweigen).
    - Wenn ein CRC-Fehler am physischen Ende auftritt (`pos >= file_size`), wird er als unvollständiger Write-Tail identifiziert.

---

## 4. Flusher-Actor-Fsync-Garantien (W4)

### Befund & Analyse
- **Fsync vor Caller-Bestätigung**: In `crates/contextra-store/src/wal/flusher.rs`:
  - Der `Wal`-Flusher-Actor verarbeitet Appends sequentiell.
  - Erst nach erfolgreichem `file.write_all()`, `file.flush()` UND `file.sync_all()` werden die Oneshot-Acks an die wartenden Caller abgesendet:
    ```rust
    // crates/contextra-store/src/wal/flusher.rs:271-294
    let sync_res: Result<()> = async {
        file.sync_all().await.map_err(|e| {
            ContextraError::Storage(format!("WAL fsync failed, outcome unknown: {}", e))
        })?;
        Ok(())
    }.await;
    ...
    for ack in acks {
        let _ = ack.send(Ok(()));
    }
    ```
- **Poisoning bei I/O- oder fsync-Fehler**:
  - Schlägt `sync_all()` oder `write_all()` fehl, setzt der Actor `poisoned.store(true, Ordering::SeqCst)` und schneidet die Datei per `file.set_len(size_before)` auf den Zustand vor dem Schreibversuch zurück. Nachfolgende Appends werden sofort mit einem Poisoning-Fehler abgelehnt, bis eine explizite Recovery (`recover_from_poison()`) erfolgt.

---

## 5. Partial-Write-Recovery (W5)

### Befund & Analyse
- **Crash-Konsistenz / Tail-Truncation**:
  - Wenn ein Absturz während eines Schreibvorgangs erfolgt (Torn Write am Dateiende), wird beim Neustart (`LsmStorage::new()`, `Wal::open_with_config()`) in `crates/contextra-store/src/wal/mod.rs` (Zeile 693-714) der verifizierte Endpunkt bestimmt.
  - Ist `verified_end < metadata.len()`, protokolliert der Store eine Warnung (`"Truncating torn WAL write tail on open"`) und ruft `wal.truncate(verified_end, last_hmac)` auf.
  - `truncate()` führt im Flusher-Actor ein physisches `file.set_len(offset)` und anschließendes `file.sync_all()` durch (`flusher.rs:327-362`), womit der unvollständige Tail dauerhaft entfernt wird.

---

## 6. WAL-Rotation-Atomizität (W6)

### Befund & Analyse
- **Sealing & Isolation**:
  - In `crates/contextra-store/src/wal/io.rs` und `flusher.rs` setzt `rotate_and_seal()` das Atomic-Flag `sealed = true` und sendet ein `WalCommand::Seal` an den Flusher-Actor.
  - Der Actor führt `file.sync_all()` aus, benennt die Datei atomar in `<filename>.sealed.<seq>` um, führt ein `fsync` auf das Parent-Directory aus (`fsync_parent_dir`) und setzt die Dateirechte auf Read-Only (`perms.set_readonly(true)`).
- **Ressourcen-Garantie & Recovery-Sicherheit**:
  - Bei der Compaction/Rotation im LSM-Storage (`crates/contextra-store/src/lsm/ops/compaction.rs`) wird das neue WAL-Segment erst vollständig geöffnet und als aktiver WAL-Handle im Store registriert, bevor das alte Segment als `sealed` markiert und zur späteren Löschung vorgemerkt wird.

---

## 7. Multi-Segment WAL-Recovery-Vollständigkeit (W7)

### Befund & Analyse
- **Vollständiges Replay aller Segmente**:
  - In `crates/contextra-store/src/lsm/recovery.rs` (`LsmStorage::new()`, Zeile 163-274) liest der Recovery-Prozess den Datenpfad aus.
  - Alle gefundenen WAL-Dateien (`wal-*.log` sowie `wal.log`) werden nach ihrer Sequenznummer aufsteigend sortiert in `wal_files` gesammelt.
  - Die Recovery iteriert über **ALLE** Elemente in `wal_files`, öffnet jedes Segment mittels `Wal::open_read_only()` und replayt alle Transaktionen in die `MemTable`.
  - Erst nach erfolgreichem Replay und einem erzwungenen Startup-Flush in eine SSTable werden obsolete alte WAL-Segmente gelöscht (`crates/contextra-store/src/lsm/recovery.rs:521-572`).

---

## 8. Verifikations- und Fuzz-Testergebnisse

### Testausführung Log-Auszug

1. **Unit & Module Tests**:
   - Befehl: `cargo test -p contextra-store --locked --lib wal`
   - Ergebnis: **37 passed, 0 failed** (0.52s)
   - Abgedeckte Szenarien: CRC-Roundtrip & -Fehlererkennung, Length-Header-Fuzzing, Partial-Write-Atomizität, Truncate-Durabilität über Crashs, Flusher-Coalescing & Queue-Backpressure, Read-Only-Garantien bei Rotation.

2. **Integration Test (Truncate Ordering)**:
   - Befehl: `cargo test -p contextra-store --test wal_truncate_ordering -- --nocapture`
   - Ergebnis: **5 passed, 0 failed** (0.31s)
   - Abgedeckte Szenarien: Flusher-Actor Exclusive Ownership, Size-Counter-Konsistenz nach Truncate, HMAC-Chain-Validität nach Truncate/Rewrite.

---

## 9. VERDICT & EVIDENCE

```text
================================================================================
VERDICT: APPROVED
EVIDENCE:
- W1 HMAC-CHAIN: crates/contextra-store/src/wal/hmac.rs:156-180 & encode.rs:172-208
- W2 BOUNDS-CHECK: crates/contextra-store/src/wal/io.rs:114-135 (MAX_WAL_ENTRY_SIZE = 64MB)
- W3 CRC32-VALIDATION: crates/contextra-store/src/wal/encode.rs:262-277 & replay.rs:31-54
- W4 FLUSHER-FSYNC: crates/contextra-store/src/wal/flusher.rs:271-294
- W5 PARTIAL-WRITE-RECOVERY: crates/contextra-store/src/wal/mod.rs:693-714 & flusher.rs:327-362
- W6 ROTATION-ATOMICE: crates/contextra-store/src/wal/flusher.rs:364-445
- W7 MULTI-SEGMENT-REPLAY: crates/contextra-store/src/lsm/recovery.rs:163-274
- TEST-SUITE: 42/42 WAL Tests PASSED (logs/audits/wal-lib-test.log, logs/audits/wal-bug7.log)
================================================================================
```

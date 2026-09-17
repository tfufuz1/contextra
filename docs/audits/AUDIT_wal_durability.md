# AUDIT Report: WAL Durability & Flusher Architecture (`memfuse-store`)

## Metadaten & Scope
* **Ziel-Crate:** `crates/memfuse-store` (Layer 1)
* **Auditierte Dateien:**
  * `crates/memfuse-store/src/wal/flusher.rs`
  * `crates/memfuse-store/src/wal/segment.rs` (Dateiexistenz geprüft — Datei existiert nicht im Repository)
* **Modus:** `audit-readonly`
* **Claim-Status:** Registriert via `cargo xtask claim --crate memfuse-store --mode audit-readonly`

---

## 1. Schreiben, `fsync`/`fdatasync` und Bestätigungs-Reihenfolge

* **Status:** **BELEGT**
* **Exakter Codepfad:** `crates/memfuse-store/src/wal/flusher.rs:182-231`

### Analyse & Ablauf
In `flusher.rs` verarbeitet der Flusher-Actor eingehende Schreibbefehle vom Typ `WalCommand::Append` in folgender sequenzieller Reihenfolge:

1. **Header-Schreiben:** Falls die WAL-Datei neu/leer ist, wird zuerst der V3-Magic-Header geschrieben (`WAL_V3_HEADER`, `flusher.rs:186-193`).
2. **Batch-Payload-Schreiben:** Das zusammengefasste Batch-Byte-Array wird via `file.write_all(&batch_payload).await` in den Betriebssystem-Puffer geschrieben (`flusher.rs:194-200`).
3. **Buffer-Flush:** `file.flush().await` leert den internen Tokio/I/O-Puffer (`flusher.rs:201-207`).
4. **Fsync auf Disk:** `file.sync_all().await` erzwingt den Syscall zum physischen Bündeln aller Blöcke auf das Speichermedium (`flusher.rs:208-214`).
5. **Größenaktualisierung:** Die Atombeschreibung `size.fetch_add(written_len as u64, ...)` wird erst nach erfolgreichem Fsync aktualisiert (`flusher.rs:220`).
6. **Bestätigung an Aufrufer (ACK):** Erst nachdem `sync_all().await` fehlerfrei abgeschlossen ist, wird das Ergebnis an die wartenden Aufrufer gesendet:
   ```rust
   for ack in acks {
       let send_res = match &res {
           Ok(()) => Ok(()),
           Err(e) => Err(MemFuseError::Storage(e.to_string())),
       };
       let _ = ack.send(send_res);
   }
   ```
   (`flusher.rs:226-231`)

### Durability-Garantie & Modi
* **Coalescing-Fenster:** `WalFlusherConfig::batch_window_micros` (Default: `100 µs`, `flusher.rs:60-70`) erlaubt dem Flusher, mehrere parallele Writes innerhalb des Microsecond-Fensters in einer einzigen I/O-Operation zusammenzufassen.
* **Kein geschwächter Modus:** Es existiert **kein Flag oder Modus** (wie z. B. `sync=false`), der `sync_all()` überspringt oder die Bestätigung an den Aufrufer vor Ausführung von `sync_all()` versendet. Die Durability-Garantie ist standardmäßig und ausnahmslos aktiv.

---

## 2. Fehlerbehandlung bei `write`/`fsync`-Syscall-Fehlern (`std::io` / `tokio::fs`)

* **Status:** **BELEGT**
* **Exakter Codepfad:** `crates/memfuse-store/src/wal/flusher.rs:182-231` sowie `crates/memfuse-store/src/wal/io.rs:504-508`

### Analyse
* **Fehlerumwandlung:** Jeder `std::io::Error` bei `write_all`, `flush` oder `sync_all` wird sofort über `.map_err(...)` abgefangen und in einen `MemFuseError::Storage(...)` konvertiert:
  ```rust
  file.sync_all().await.map_err(|e| {
      MemFuseError::Storage(format!("WAL flusher fsync failed for {}: {}", path.display(), e))
  })?;
  ```
  (`flusher.rs:208-214`)
* **Propagierung an Aufrufer:** Das Ergebnis `res: Result<()>` wird in der Schleife an alle Aufrufer (`acks`) der gepackten Batch versendet (`flusher.rs:226-231`).
* **Client-Aufruf-Sicht:** In `Wal::append_batch_locked` wartet der Aufrufer auf die ACK-Oneshot-Response:
  ```rust
  ack_rx.await.map_err(|_| MemFuseError::Storage("WAL flusher dropped".into()))??;
  ```
  (`io.rs:504-508`). Durch den doppelten `?`-Operator wird der `MemFuseError::Storage` direkt an den Caller zurückgegeben.
* **Kein Silent-Ignore / Kein Retry:** Es existiert weder ein stummes Verwerfen (Silent-Ignore) noch ein automatisches Retry im Flusher-Loop bei Syscall-Fehlern. Jeder Syscall-Fehler führt zum sofortigen Fehler-Return für alle Betroffenen der Batch.

---

## 3. Segment-Rotation & Atomizität

* **Status:** **BELEGT** (in `flusher.rs` und `io.rs`) / **NICHT VERIFIZIERBAR** (bezüglich `crates/memfuse-store/src/wal/segment.rs`)

### Dateiexistenz-Hinweis
Die Datei `crates/memfuse-store/src/wal/segment.rs` existiert **nicht** im Repository. Segment-Versiegelung und -Rotation sind direkt in `flusher.rs` (`WalCommand::Seal`) und `io.rs` (`Wal::rotate_and_seal`) verankert.

### Rotations-Ablauf & Atomizität (`WalCommand::Seal`)
1. **Versiegelungs-Lock:** Beim Aufruf von `Wal::rotate_and_seal` wird die Atombeschreibung `sealed` auf `true` gesetzt (`io.rs:624`). Neue Schreibzugriffe (`append_batch` / `truncate`) werden sofort mit `MemFuseError::Storage("Cannot append to sealed WAL segment...")` abgelehnt (`io.rs:430-434`, `io.rs:610-614`).
2. **Pre-Rename Fsync:** Bevor die Datei umbenannt wird, führt der Flusher zwingend `file.sync_all().await` auf der aktiven WAL-Datei aus (`flusher.rs:313-318`).
3. **Atomarer Rename:**
   ```rust
   crate::wal::fs::rename(&path, &sealed_path).await.map_err(|e| { ... })?;
   ```
   (`flusher.rs:328-337`). Der Dateiname wird atomar auf POSIX-/Dateisystem-Ebene von `wal` zu `wal.sealed.<micros>` geändert.
4. **Directory Fsync:** Das übergeordnete Verzeichnis wird synchronisiert:
   ```rust
   crate::util::fsync_parent_dir(&sealed_path).await?;
   ```
   (`flusher.rs:339`), um sicherzustellen, dass der Verzeichniseintrag auf dem Speichermedium persistent ist.
5. **Read-Only Rechte:** Die versiegelte Datei wird auf schreibgeschützt gesetzt (`flusher.rs:341-358`).

### Sichtbarkeit für Reader
Ein Reader kann zu keinem Zeitpunkt ein halb geschriebenes oder nicht per `fsync` gesichertes Segment sehen, da `sync_all()` strikt vor dem atomaren `rename` auf dem Dateisystem ausgeführt wird.

---

## 4. CRC/Checksummen-Validierung beim Replay (Recovery)

* **Status:** **BELEGT**
* **Exakter Codepfad:** `crates/memfuse-store/src/wal/encode.rs:201-209` und `crates/memfuse-store/src/wal/io.rs:183-200`

### Analyse
1. **CRC32-Berechnung & Prüfung:** Beim Deserialisieren jedes WAL-Eintrags via `WalEntry::from_bytes` (`encode.rs:193-210`) wird die abgelegte CRC32 mit der berechneten Hash-Summe verglichen:
   ```rust
   if stored_crc != computed_crc {
       return Err(MemFuseError::Serialization(format!(
           "CRC mismatch: stored={:#010x}, computed={:#010x}",
           stored_crc, computed_crc
       )));
   }
   ```
   (`encode.rs:201-209`).
2. **Behandlung beim Replay Scan:** In `do_scan_entries_with_callback` (`io.rs:183-200`) wird der von `from_bytes` zurückgegebene Fehler bewertet:
   ```rust
   let is_crc_error = err_msg.contains("CRC mismatch");
   if pos >= file_size && !is_crc_error {
       tracing::warn!("WAL truncation at tail (offset {}), partial entry: {}", chunk_start_pos, e);
       break;
   } else {
       let reason = if is_crc_error {
           format!("CRC validation failed: {e}")
       } else {
           format!("Deserialization failed: {e}")
       };
       return Err(MemFuseError::wal_corruption(chunk_start_pos, reason));
   }
   ```
   (`io.rs:191-200`).
3. **Kein Silent-Skip:** Bei einem CRC-Fehler an einer beliebigen Stelle in der Datei bricht das Replay **sofort mit `Err(MemFuseError::WalCorruption)`** ab. Es wird nicht nur eine Warnung geloggt oder der Rest übersprungen.
4. **Tail-Truncation-Unterscheidung:** Lediglich unvollständige Records am Dateiende (EOF / unvollständiger Length-Prefix am Tail) werden als saubere Crash-Truncation eingestuft und beenden den Replay-Loop mit einer Warnung (`tracing::warn!`).

---

## 5. Backpressure & Queue-Steuerung

* **Status:** **BELEGT**
* **Exakter Codepfad:** `crates/memfuse-store/src/wal/flusher.rs:105` und `crates/memfuse-store/src/wal/io.rs:498-508`

### Code-Zitat
```rust
// In Wal::enable_flusher_with_config (flusher.rs:105):
let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<WalCommand>();

// In Wal::append_batch_locked (io.rs:498-508):
tx.send(WalCommand::Append {
    payload: payload_bytes,
    last_hmac_val,
    ack: ack_tx,
})
.map_err(|_| MemFuseError::Storage("WAL flusher channel closed".into()))?;

ack_rx.await.map_err(|_| MemFuseError::Storage("WAL flusher dropped".into()))??;
```

### Analyse & Backpressure-Verhalten
* **Kanal-Typ:** Der Flusher nutzt einen **ungebundenen Kanal** (`unbounded_channel`).
* **Producer-Verhalten:** `tx.send(...)` blockiert nicht direkt beim Hinzufügen des `WalCommand::Append` in den Kanal.
* **Task-Level Backpressure:** Der aufrufende Producer-Task blockiert anschließend auf `ack_rx.await`. Dadurch wird der aufrufende Task anhaltend suspendiert, bis der Flusher-Actor die Batch geschrieben, ge-flusht und ge-fsynced hat.
* **Speicherrisiko bei extremer Last:** Da der Kanal selbst ungepuffert/ungebunden ist, können sich bei sehr vielen gleichzeitig aufrufenden Tokio-Tasks unbegrenzt viele `WalCommand`-Instanzen im Speicher aufstauen, falls der Flusher mit der I/O-Latenz des Datenträgers nicht schrittführen kann.

---

## 6. `.unwrap()` / `.expect()`-Scan

* **Status:** **BELEGT**
* **Scan-Scope:**
  * `crates/memfuse-store/src/wal/flusher.rs`
  * `crates/memfuse-store/src/wal/segment.rs`

### Befunde
* **`crates/memfuse-store/src/wal/segment.rs`**: **Datei existiert nicht** im Repository.
* **`crates/memfuse-store/src/wal/flusher.rs`**:
  * **0** Aufrufe von `.unwrap()`
  * **0** Aufrufe von `.expect()`
  * *Hinweis zu sicheren Alternativen:*
    * Zeile 101: `.unwrap_or_else(|e| e.into_inner())` auf `RwLockWriteGuard` (sichere Handhabung von Lock-Poisoning).
    * Zeile 321: `.unwrap_or_default()` für Epochen-Timestamp.
    * Zeile 325: `path.file_name().and_then(|n| n.to_str()).unwrap_or("wal")` für Fallback-Dateinamen.

Der Produktionscode des WAL-Flushers ist frei von unkontrollierten `.unwrap()`/`.expect()`-Panics.

---

## 7. Invarianten & DAG-Architektur-Check

* **Status:** **BELEGT**
* **DAG-Integrität:** In `crates/memfuse-store/Cargo.toml` sind ausschließlich Abhängigkeiten zu Layer-0-Crates (`memfuse-core`, `memfuse-crypto`) deklariert. Es existieren keinerlei Importe aus höheren Layern (`memfuse-db`, `memfuse-candle`, `memfuse-router`, `memfuse-index`). Die Schichtenarchitektur (DAG) ist eingehalten.

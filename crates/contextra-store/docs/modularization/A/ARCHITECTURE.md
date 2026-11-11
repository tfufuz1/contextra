# ARCHITECTURE — WAL (Komponente A)

## 1. Modulübersicht (Dateikarte)
- `src/wal/mod.rs`: Öffentliche Fassade und Re-Exporte (`KeyManager`, `Wal`, `WalConfig`, `PoisonCause` etc.).
- `src/wal/core.rs`: Hauptstruktur `Wal` mit Lebenszyklus, Konstruktoren und Statics.
- `src/wal/config.rs`: `WalConfig` und Builder-Methoden.
- `src/wal/poison.rs`: `PoisonCause`-Enum mit unveränderlichen Diskriminanten 1..5.
- `src/wal/fs.rs`: Dateisystem-Shim für Loom und Tokio-FS.
- `src/wal/encode.rs`: Serialisierung von `WalEntry`, Framings, CRC32C und AES-GCM-SIV-Payloads.
- `src/wal/hmac.rs`: HMAC-SHA256-Kettenvalidierung und Integritätsschlüsselverwaltung.
- `src/wal/flusher.rs`: Asynchroner Flusher-Actor mit Ack-Disziplin, Kanalsteuerung und Disk-Sync.
- `src/wal/replay.rs`: Sequentielles Replay, Validierung über Frame-Grenzen und Torn-Write-Erkennung.
- `src/wal/io/`: Low-Level-IO-Operationen (`append.rs`, `truncate.rs`, `scan.rs`, `acl.rs`).

## 2. Datenfluss: Append -> Flusher -> fsync -> Ack
1. **Prepare:** `Wal::prepare_batch` berechnet Integritäts-Hashes (CRC32C und HMAC) für die Mutationen und packt einen `PreparedBatch`.
2. **Enqueue:** Aufruf von `enqueue_append_batch_locked` oder `append_batch` übergibt den Batch an den Flusher-Kanal (`tokio::sync::mpsc`).
3. **Write & Sync:** Der Flusher-Actor schreibt die Bytes auf Platte (`fs::write`), erzwingt bei Bedarf `sync_all` (fsync).
4. **Ack:** Nach bestätigtem `sync_all` sendet der Flusher Erfolgs-Acks an wartende Aufrufer.

## 3. Poison-Zustandsdiagramm
```
           +----------------+
           |     Normal     |
           +----------------+
                   |
         +---------+---------+-----------------+-----------------+
         |                   |                 |                 |
    [fsync fehlschlägt]  [Schreibfehler]  [Truncate scheitert] [HMAC divergiert]
         |                   |                 |                 |
         v                   v                 v                 v
   FsyncFailed (1)     WriteFailed (2)   TruncateFailed (3) HmacChainDiverged (4)
   (Irreversibel)      (Irreversibel)    (Irreversibel)     (Irreversibel)
                                                                 |
                                                          [Torn Write]
                                                                 |
                                                                 v
                                                           TornWrite (5)
                                                        (Behebbar via Replay)
```

## 4. HMAC-Kettenbindung
Jeder WAL-Frame `n` bindet den Hash des Vorgängers `n-1` kryptographisch ein:
`HMAC_n = HMAC-SHA256(K_integrity, HMAC_{n-1} || seq_no || tx_id || op_payload)`
Dies verhindert unbemerkte Frame-Trunkierungen, Auslassungen oder Vertauschungen.

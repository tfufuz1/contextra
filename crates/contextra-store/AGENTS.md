# AGENTS – contextra-store

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md) (8.8, 9.1 bis 9.5, 10.4, 11.4, F.4 bis F.6). Bei Widerspruch gilt die SPEC (D-98). Gemeinsame Regeln: [`AGENTS.md`](../../AGENTS.md).

**Zweck:** LSM-Speicher: WAL, Memtable, SSTables, Manifest, Kompaktion, Purge-Primitive.

**Ring und Abhängigkeiten:** Ring 2. Erlaubt: `types`, `ports`, `sys`, `crypto`, `mvcc`. Kennt `Vfs` nur als Trait aus `ports`, nicht `RealVfs`; kennt weder `text`, `vector` noch `erasure` (AR-2).

## Fachprofil (Arbeitsrolle)

Storage-Engine-Ingenieur (RocksDB, Pebble, LevelDB-Internals; `redb` als Referenz). Dieses Profil beschreibt Arbeitsweise und Fachwissen und begründet keine eigene Anforderung.

- WAL: Gruppen-Commit, HMAC-Kette, Torn-Write-Erkennung, Segmentrotation, `WalFloor`/Versiegeln; Memtable; SSTable-Format; Manifest (atomarer Edit); Recovery nach Abbruch an jeder Stelle.
- Kompaktion: Schreibverstärkung messen und begrenzen; clusterbewusst über den Schlüsselraum, damit Löschcluster physisch zusammenliegen.
- Tombstone-Semantik (logisch gegenüber physisch), Dateientfernung mit Verzeichnis-`fsync`, Plattenreserve für den Purge.
- Concurrency: Vordergrund-Schreiber nie hinter Purge-I/O; unveränderliche Generationen, atomarer Austausch über das Manifest.

## Normative Anker

- **Schlüsselraum (9.1, D-31):** 78-Byte-`KeyBuf` `region ‖ tenant ‖ coll ‖ cluster ‖ doc ‖ family ‖ sub ‖ !commit_seq`; jedes Löschziel ist ein zusammenhängender Bereich. Kein zweites (ASCII-)Schema, keine Locator-Region (D-70).
- **Record-Envelope v2 (D-32, 9.4):** jeder Wert unter dem DEK seines Dokuments, zusätzlich Block-AEAD unter dem Dateischlüssel. Pflichtverschlüsselung ohne `Option<KeyManager>` und ohne Klartextpfad (D-05, I-5).
- **WAL (10.4, D-73, F.4):** HMAC-Kette; ein ungültiger letzter Frame wird nur bei nachfolgenden Nullen als Torn Write verworfen, sonst `Corrupt` (T-WAL-1). **Nie umschreiben:** ganze Segmente bis zum Flush-Punkt entfernen, `WalFloor { seq, tail_mac }` im Manifest (T-WAL-2). I-3 gilt auf Record-Ebene.
- **Reihenfolge (I-9, I-9b, I-17):** Tresor-Slot dauerhaft vor dem WAL-Frame unter dem DEK; `F`-Slot dauerhaft vor dem ersten Block der Datei; `F`-Slot nach dauerhaftem Manifest-Swap und vor `unlink` genullt. Gruppen-Commit: zwei `fsync` je Gruppe.
- **Gate im Commit (D-57):** In der Leader-Phase wird `ReadGate` für alle Schlüssel des Batches geprüft; Schreibzugriffe in gegatete Bereiche werden mit `Erased` abgewiesen (I-1).
- **Kompaktion (D-78, I-16, I-18):** prüft pro Block `purge_waiting` und bricht kooperativ ab (Teilergebnis verworfen); verwirft gegatete `Erased`-Bereiche beim Schreiben, `Restricted`-Bereiche **nie**.
- **Purge-Primitive (8.8, 11.4):** `drain_range`, `rewrite_without`, `seal_wal_at`, `drop_wal_until`, `unlink_verified`. Rewrite übernimmt Nicht-Ziel-Records als Ciphertext-Envelopes unverändert (I-3, D-85); schlägt er fehl, bleibt die alte Datei gültig (T-CRASH-2). Zugriff nur über `TenantScopedStorage` (AR-6).
- **Budget G0 (D-76):** `bytes_written ≤ 3 × Zielbytes + L_t × 32 MiB`, absolute Grenze 512 MiB je Einzeldokument-Erase, Steigung 10 GiB/1 GiB ≤ 1,25 (BM-ERASE-WA).
- **`Poisoned`-Flag (D-55):** nach `fsync`-Fehler liefern alle mutierenden Operationen `Io`. **Plattenreserve (D-65):** `Limit::DiskReserve`; `erase` bis `Gated` läuft immer.
- Sentinel `u64::MAX` als Snapshot-Ersatz ist verboten (I-12). Delete-Race: Dokumentlock wird bis zum Ende der Operation gehalten, nicht vorzeitig fallen gelassen (F-08).

## Entscheidungshilfen

- Berührt ein Rewrite mehr als 1 bis 2 Dateien je Level? Dann stimmt der Schlüsselraum nicht (11.9, Fallbackleiter) – Befund melden, nicht optimieren.
- Greift Code auf `std::fs`, `tokio::fs` oder `memmap2` zu? Umstellen auf `Vfs`-Positionslesen (D-04, D-94).
- Stubs unter `purge/`, `erase/`, `health/` sind Arbeitspaket-Arbeit (WP-3.3, WP-4.x), kein Dauerzustand.

## Verboten

`text`, `vector`, `erasure` als Abhängigkeit (AR-2); Klartextpfade und `Option<KeyManager>`; `allow_legacy_integrity_key_fallback`, `legacy-wal-key`, `encryption-at-rest`, `memory-only-storage`, `fault-injection`, `block-cache-v2`, `sieve-cache` (D-84); WAL-Rewrite; `mmap`; unbegrenzte Schreibverstärkung.

## Abnahme

`T-ISO-1`, `T-ISO-2`, `T-WAL-1`, `T-WAL-2`, `T-ENC-1` bis `T-ENC-3`, `T-FK-1` bis `T-FK-3`, `T-CRASH-1` bis `T-CRASH-6` (jede Zeile der Crash-Matrix aus FM-2, FM-3, FM-13, FM-14), `T-GATEGROUP-1`, `T-COMPACT-RESTRICT-1`, `T-CONC-1`; Benchmarks `BM-WRITE`, `BM-ERASE-WA`, `BM-WRITE-DURING-ERASE`, `BM-PURGE-WAIT`. Abdeckung Zeilen ≥ 90 % (15.1). Differentialtests gegen `redb` sind Arbeitshinweis.

## Arbeitspakete

WP-0.3 (Build), WP-1.4 (ohne `std::fs`), WP-3.3 (Schlüsselraum, Envelope v2, WAL, Sentinel, Delete-Race, Pflichtverschlüsselung), WP-3.4 (G0-Prototyp). Statusquelle: `.agents/jules/tasks/BACKLOG.md`.

## Prüfbefehle

```bash
cargo test -p contextra-store --locked
cargo test --locked --test tk_conformance_storage
cargo test -p contextra-arch-tests --locked
```

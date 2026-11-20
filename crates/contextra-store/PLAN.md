# contextra-store — Crate-Spezifikation (Soll-Zustand, SPEC 2.1)

**Ebene:** E1 (Domäne)
**Domänen-Closure:** `ids`, `env`, `sys`, `durable-fs` (höchstens 5, SPEC §15)
**Verträge:** `ports`, `erasure` (zählen nicht gegen die fünf)
**Status:** Normativ (Soll-Zustand)
**Version:** 2.1
**Geltungsbereich:** `crates/contextra-store`
**Normative Quellen:** `docs/SPEC.md` §4.5, §4.6, §5.3 Schritte 3 und 8, §5.9 Schichten S1 und S1b, §5.11, §6, §7 (`StoreLayer`), §9.2, §10.2, INV-6.

---

## 1. Zweck

`contextra-store` ist die LSM-Persistenz: WAL, MemTable, SSTables, Blob-Segmente, Block-Cache und Manifest.

- Records der Klassen K, P und D liegen verschlüsselt unter dem `DocDek` ihrer Einheit. Die AEAD-Versiegelung führt `contextra-db` mit `contextra-crypto` aus und übergibt Chiffretext. Store persistiert ihn. Damit bleibt die Kern-Closure in E0 (SPEC §15). Die äußere Blockhülle liegt unter `K_eng`.
- Ein Block darf Records mehrerer Einheiten enthalten. Jeder Record bleibt unter seinem eigenen `DocDek`. Die Vernichtung eines `DocDek` macht die anderen Records des Blocks nicht unlesbar (§4.5).
- `LsmStorage` ist nach außen `pub(crate)`. Jeder Lesezugriff der Engine läuft über `TenantScopedStorage` in `contextra-db`. Dort sitzt die einzige Gate-Prüfstelle (§6.1).

WAL-, Manifest-, Flush- und Recovery-Formate bleiben beim Umbau stabil (INV-6). Ein Formatwechsel trägt eine Formatkennung (INV-1).

## 2. Architekturregeln

1. `#![forbid(unsafe_code)]`. Zero-Panic. `default = []`.
2. Keine Abhängigkeit auf `contextra-crypto`, `contextra-mvcc` oder ein anderes E1-Crate. Keine Dev-Abhängigkeit auf `contextra-testkit` (R2).
3. `StoreLayer` implementiert `ErasureLayer` aus `contextra-erasure`.
4. Logs enthalten keinen Klartext des Löschziels (INV-12).
5. Dateinamen entstehen aus der Sequenz oder aus `OsRng::try_fill_bytes`. Kein ungeprüftes UUIDv4.

## 3. Komponenten

```
LsmStorage (pub(crate))
  WAL (append, group commit, Chiffretext)
  Active MemTable / Immutable MemTables
  SSTables L0..Ln und Blob-Segmente
  Block-Cache (Purgeable, Schicht S4 gemeinsam mit db)
  TableManifest
  PendingDelete-Journal
  StoreLayer (ErasureLayer)
```

SSTable-Aufbau, unverändert in der Rolle, mit Formatkennung im Magic:

```
Magic (Formatkennung) | Data Blocks (Chiffretext) | Filter | Index | Footer
```

Klartext-Canaries dürfen nach Insert und Flush in keiner Datei stehen (T-DEL-1).

## 4. Rolle in der Löschsaga

`StoreLayer` ist Schicht **S1** (LSM und WAL) und **S1b** (Blob-Segmente). Messung: `LsmScanner` (Klartext, Chiffrat, Nadel). Der Scanner teilt keinen Code mit dem Purger (INV-WITNESS-MINT-1). Der Orchestrator misst zusätzlich mit einer Black-Box-Probe (D-13).

`purge` ist idempotent (T-DEL-14). `witness` ist lesend.

### 4.1 Schritt 3 — Ebene A, MemTable und Cache

Vor der Bereinigung steht der Übergang `PlaintextPurged` im Journal. Die Arbeit ist asynchron und ruft kein `block_on`.

- Einträge des Scopes werden der aktiven MemTable entzogen oder mit dem nächsten Flush aus dem Klartext entfernt.
- Der Block-Cache des Scopes implementiert `Purgeable` aus `contextra-ports`, wird invalidiert und zeroized (S4, REQ-DEL-N7). Den dokumentbezogenen RRF-Cache `er_cache` hält `contextra-rank`. Den Surrogat-Vault hält `contextra-privacy`. Embedding- und Antwort-Cache registriert `contextra-db`, sobald sie existieren.
- Noch nicht versiegelte WAL-Puffer des Scopes werden verworfen. Bereits durabel geschriebene WAL-Chiffrate bleiben bis Schritt 8 bestehen. Sie sind unter dem `DocDek` und werden in Schritt 5 unlesbar.

### 4.2 Schritt 8 — Hygiene, Purge-Compaction und WAL

Die Hygiene ist durch `purge_rebuild_max_days` (Startwert 14 Tage) terminiert. Sie läuft unter `PurgeBarrier::purge` aus `contextra-ports`. `purge_waiting` fordert laufende Kompaktionen auf, den Slot freizugeben (§10.2).

1. **SSTables und Blobs.** `erase_keys` schreibt betroffene SSTables und Blob-Segmente streamend neu (`create_like`: gleicher Key-Manager, gleiche Formatversion, originale `seq` und `tx_id`). Alle MVCC-Versionen, Tombstones und Klartext-Indexschlüssel der Zielschlüssel entfallen. Austausch: `tmp` → `fsync` → Manifest-Commit → `fsync` → Verzeichnis-`fsync` → Reader-Swap.
2. **Kompaktionsfilter.** `ReadGate::check_range` liefert `ContainsErased`. Solche Ranges nimmt das Kompaktionsergebnis nicht auf (§6.3). `RestrictedNeverDiscard` behält die Range.
3. **PendingDelete.** Ersetzte Dateien werden erst gelöscht, wenn keine offene Zeitreise sie mehr pinnt (§9.2). Den `min_active_seqno` liefert `contextra-mvcc` über `contextra-db`. Store importiert `mvcc` nicht. Der Block-Cache ersetzter Dateien wird zeroized.
4. **WAL.** Aktive WALs rotieren, MemTables flushen, versiegelte WAL-Dateien werden gelöscht. Über `contextra-durable-fs` wird geprüft, dass die Dateien nicht mehr existieren. Bleibt eine Datei, bleibt die Saga `Pending` mit Retry.
5. **Abschlussmessung.** `LsmScanner` läuft mit Positivkontrolle `AC-1`. Klasse M setzt Siegel 4 voraus und wird vom Koordinator vergeben, nicht von Store.

Physisches Überschreiben ersetzt die Schlüsselvernichtung nicht. Die kryptographische Garantie liegt in Schritt 5 (`contextra-crypto`). Der Scanner sieht keine Flash-Remanenz (Annahme `A-FLASH`).

### 4.3 Lesepfad innerhalb des Store

Store führt den Gate-Check nicht als zweite, abweichende Stelle aus. Er reicht Schlüssel und Ranges an den Aufrufer. `get` entschlüsselt nicht. Kompaktionen rufen `check_range` auf, bevor eine Range in das Ergebnis kommt.

## 5. Fehler (`StoreError`)

`#[non_exhaustive]`: `WalCorrupted`, `SstableCorrupted`, `ManifestRecovery`, `CompactionFailed`, `DurableFs`, `Io`, `PendingDeleteBlocked`, `WalFileRemains`. Ein I/O-Fehler im Purge ist transient. Die Saga setzt `Pending(IoBackoff)` und wiederholt (§11.1). Store ändert den Saga-Zustand nicht selbst.

## 6. `Cargo.toml`

```toml
[package]
name = "contextra-store"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "LSM storage engine: WAL, memtable, SSTables, blob segments"

[lints]
workspace = true

[dependencies]
contextra-ids = { workspace = true }
contextra-env = { workspace = true }
contextra-sys = { workspace = true }
contextra-durable-fs = { workspace = true }
contextra-ports = { workspace = true }
contextra-erasure = { workspace = true }
bytes = { workspace = true }
blake3 = { workspace = true }
crc32fast = { workspace = true }
parking_lot = { workspace = true }
ahash = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
proptest = "1.4"

[features]
default = []
std = ["contextra-ids/std", "contextra-env/std"]
```

## 7. Tests

| Kennung | Pflicht |
|---|---|
| T-DEL-1 | Nach Insert und Flush kein Klartext-Canary in WAL, SSTable oder Blob |
| T-DEL-2 | Nach Purge-Compaction `residue_matches == 0` im `LsmScanner` |
| T-DEL-12 | Crash in Schritt 3: MemTable-Purge ist wiederholbar, Lesesperre bleibt Aufgabe des Koordinators |
| T-DEL-14 | Zweites `purge` ist ein No-op |
| T-DEL-16 | Abbruch während Manifest-Commit: Recovery sieht alte oder neue Generation, nie eine Mischung |
| T-DEL-17 | Rohsicht der Store-Dateien nach Hygiene |

`scrub` einer Datei ohne bestandene Positivkontrolle ist kein Nachweis.

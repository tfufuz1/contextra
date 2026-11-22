# contextra-storage: Refaktorisierungsplan (Fundament-Crate)

| | |
|---|---|
| Status | Entwurf zur Umsetzung. Entscheidungen final (`SD-xx`), Annahmen markiert (`A-xx`), Leistungsaussagen sind **Hypothesen** (`H-xx`). |
| Gegenstand | Crate `crates/contextra-store` (Repository `tfufuz1/acta`, Stand `47c9865`), künftig **`contextra-storage`** |
| Zielgruppe | Implementierer und Reviewer |
| Eigenständigkeit | Plan und Spezifikation in einem Dokument. Es verweist auf kein anderes Dokument. Die Anforderungen der aufsetzenden Schichten sind in Abschnitt 17 selbst formuliert. |
| Verifikation | Der Ist-Zustand wurde durch Lesen des Codes ermittelt (Abschnitt 1). Das Crate ist im Repository **nicht baubar** (siehe 1.1), daher konnte nichts kompiliert oder gemessen werden. |

---

## 0. Kurzfassung

Das heutige Crate ist ein funktionsreicher, aber historisch gewachsener LSM-Speicher (≈ 60 000 Zeilen inkl. Tests). Es ist nicht autark, mischt `async`-Locks mit blockierender Datei-E/A, trägt drei Generationen Legacy-Formate und besitzt keinen kontrollierten Löschpfad. Der Plan ist ein **Neubau des Kerns mit gezielter Wiederverwendung** (nicht ein Umbau im Bestand), weil die Probleme in den Formaten, im Parallelitätsmodell und in der Dateiverwaltung liegen und sich nicht lokal beheben lassen.

Die Entscheidungen in einem Blick:

1. **Zero-Dependency-Kern.** Keine Workspace-Crates, keine Laufzeit-Abhängigkeiten. Verschlüsselung ist ein optionales Feature mit geprüften externen Krypto-Crates (SD-01).
2. **Synchroner Kern, kein `tokio`.** Hintergrundarbeit läuft auf einer kleinen, teilbaren `Runtime`. Async-Einbettung ist Sache des Aufrufers (SD-02).
3. **Ein Port für alle E/A (`Fs`)**, dazu `Clock`, `Entropy`. Kein direkter `std::fs`-Aufruf im Kern, keine globalen Statics. Dadurch ist ein Stromausfall-Simulator (`FaultFs`) möglich, der das Hauptqualitätstor des Plans bildet (SD-03).
4. **Ein Format je Dateiart, kein Legacy im Kern.** `FORMAT`, Redo-Log, Run, Catalog haben je genau eine Version mit Feature-Bits (SD-04).
5. **Der Catalog ist die einzige Wahrheit.** Eine Datei ist genau dann „lebendig", wenn der Catalog sie nennt. Jede Zustandsänderung ist **ein** atomarer Edit-Record (SD-05).
6. **Atomarität eines Schreibvorgangs = ein Redo-Frame.** Keine `TxEnd`-Marker, kein Abbruch-Replay (SD-06).
7. **Spaces sind physisch getrennte LSM-Bäume** (eigene Memtable, eigene Runs, eigenes Verzeichnis), gemeinsames Redo-Log und gemeinsame Stamp-Folge. Dadurch ist das Löschen eines Spaces eine einzige Catalog-Änderung (SD-07).
8. **Löschung ist ein erstklassiger Mechanismus (`Erase`)** mit festem Protokoll, einem einzigen Durability-Punkt, Roll-forward nach Absturz, Quittung und Prüfwerkzeug (Abschnitt 12).
9. **Wenige orthogonale Primitive:** `Change` (mit `Guard`s), `View`, `Cursor`, `Hold`, `Erase`, `Listener`, `Checkpoint`. Alles Weitere bauen die oberen Schichten daraus (Abschnitt 5.3).
10. **Agenten-Workloads als Designgrundlage:** byte-budgetierte, fortsetzbare Seiten (`scan_page`), Größenschätzung aus dem Index (`estimate`), Versionsverlauf (`history`), append-only Spaces, handlungsleitende Fehlertexte, zusätzliche Index-Blöcke je Run (`Extension`) und Lokalitäts-Optionen (Abschnitte 5, 7, 10).

Geplante Größe: ≈ 10–12 k Zeilen Produktionscode (Hypothese, heute ≈ 32 k in `src/`, davon ≈ 11 k Testdateien).

---

## 1. Befund (Ist-Zustand)

Methode: Quelltext gelesen, Zählungen per `grep`/`wc`. Keine Kompilierung, keine Messung.

### 1.1 Kennzahlen

| Messgröße | Wert |
|---|---|
| Zeilen `src/` / `tests/` / `benches+examples+fuzz` | 32 015 / 25 563 / 2 210 (gesamt 59 788) |
| Testfunktionen (`#[test]`, `#[tokio::test]`) | 636, verteilt auf 160 Dateien in `tests/` plus Inline-Module |
| `async fn` in `src/` | 420, davon 31 Stellen mit `spawn_blocking` |
| Direkte Dateisystemaufrufe (`tokio::fs::`, `std::fs::`) außerhalb von Tests | 91 Vorkommen in 20+ Dateien, **am Vfs-Port vorbei** |
| Vorkommen von „legacy" | 290 |
| Zugriffe auf `TOMBSTONE_BIT` | 142 (Löschmarker als Bit 63 der Sequenznummer, überall maskiert) |
| Cargo-Features | 10, davon 5 ändern Logik (`legacy-wal-key`, `block-cache-v2`, `sieve-cache`, `docid-128`, `memory-only-storage`) |

### 1.2 Befunde mit Beleg

| # | Befund | Beleg | Folge |
|---|---|---|---|
| B-1 | **Nicht baubar, nicht autark.** Im Repository existiert nur `crates/contextra-store`. Es gibt kein Workspace-Root-`Cargo.toml`. Die Abhängigkeiten `contextra-ids`, `-env`, `-sys`, `-mvcc`, `-crypto` (und optional `-types`, `-ports`) fehlen. | `ls`, `Cargo.toml` (`workspace = true`) | Ein Differentialtest gegen den Altbestand ist unmöglich. Der Altcode dient als Spezifikationsquelle, nicht als Referenzimplementierung. |
| B-2 | **Domänenlogik im Kernel.** Transaktionspuffer und SSI-Validator (`contextra-mvcc`), Intent-Locks, Mandanten-Codec (`tenant_codec`), `docid-128`-Feature, Scanner/Purger für Ports (`compat/`, 2 650 Zeilen). | `lsm/engine.rs`, `compat/*`, `Cargo.toml` | Der Speicher kennt Dinge, die nur obere Schichten kennen dürfen. |
| B-3 | **Zwei parallele Dateisystem-Verträge** (`traits.rs`: `StoreFs`/`StoreFile`, ungenutzt außer in der Definition; `io_seam.rs`: `Vfs`/`VfsFile`) plus ein dritter Pfad `wal::fs` (Tokio-Wrapper), den `fsync.rs` benutzt. | `grep StoreFs`, `fsync.rs` | Fehlerinjektion erfasst nur einen Teil der E/A. Crash-Tests sind dadurch lückenhaft. |
| B-4 | **Async überall, obwohl die E/A blockierend ist.** Acht-stufige Sperrhierarchie (`lock_order.rs`, RANK 100…800) mit dem Dauerverbot „kein Guard über `.await`". Group-Commit als Leader/Follower-Übergabe von `commit_mutex`. | `lsm/mod.rs`, `ops/write/group.rs` | Hohe Komplexität, schwer beweisbar (`loom` nur für Teilstücke). |
| B-5 | **Drei Format-Generationen im Lesepfad.** WAL V1/V2/V3 (V1 ohne Integrität; statischer XOR-obfuszierter Legacy-Schlüssel `LEGACY_INTEGRITY_KEY_OBFUSCATED`), SSTable `MFSX` v0–v4 plus `MFST` v3, Manifest v0–v2 mit abgekündigtem `rank`-Feld. | `wal/core.rs`, `sstable/format.rs`, `manifest/entry.rs` | Jeder Decoder muss alle Generationen kennen. Angriffsfläche und Testlast steigen ohne Nutzen für ein Fundament. |
| B-6 | **Transaktion als Folge von Records** (`Put`/`Delete`/`TxEnd{committed}`), HMAC-Kette je Eintrag, Replay mit „pending tx"-Zustand. | `wal/encode.rs`, `lsm/recovery/startup.rs` | Atomarität ist Konvention statt Struktur. Recovery braucht Zustandsautomat. |
| B-7 | **`rollback_to_tx`** truncatiert das WAL in place, schneidet Memtables, baut „spanning" SSTables um (Intent-Datei `MFRLBK`, Sonderfall `MIN_ENTRIES_FOR_SSTABLE_REBUILD`). | `lsm/recovery/rollback.rs` | Komplexester Wiederherstellungspfad. Mit Versionierung in der oberen Schicht überflüssig. |
| B-8 | **Manifest als Pfad-String-Log** mit Rang-Rekonstruktion (`Add`/`Remove`/`Replace`/`RollbackComplete`/`WalCheckpoint`) und Rollover. Dateien werden über Dateinamen identifiziert. | `manifest/entry.rs`, `cleanup.rs` | Leseschlüssel (Rang, `max_seq`) stehen nicht im Manifest. Planung (Compaction, Löschung) braucht Dateiöffnung. |
| B-9 | **Löschung ist nicht kontrolliert.** (a) Der Purger liest ganze SSTables per `reader.iter().await.unwrap_or_default()` in den Speicher, Lesefehler werden verschluckt. (b) Er schreibt Nullen und ruft `tokio::fs::remove_file` auf, ohne `fsync` von Datei oder Verzeichnis und am Manifest vorbei. (c) Er erkennt tote Dateien über die Endung `.sst` und „nicht in `active_paths`". (d) Die Compaction löscht Eingabedateien nach dem Austausch mit nur einer Warnung bei Fehlern, ein Verzeichnis-`fsync` nach dem Unlink fehlt (`compaction/engine.rs`: einziger `sync_dir` bei Zeile 330, für die Ausgabedatei). | `compat/scanner/purge.rs` Z. 70–81, 145, 222; `compaction/engine.rs` Z. 330, 418 | Kein atomarer, nachweisbarer physischer Löschvorgang. Nach einem Absturz ist unklar, was noch existiert. |
| B-10 | **Globaler veränderlicher Zustand.** Drei `static AtomicU64` für Fehlerinjektion in `wal/core.rs` ohne Feature-Gate (`FAIL_APPEND_FOR_TX`, `DELAY_APPEND_FOR_TX`, `DELAY_APPEND_MS`). | `wal/core.rs` Z. 27–29 | Tests beeinflussen sich, Release-Binary enthält Testschalter. |
| B-11 | **Drei Block-Cache-Implementierungen** (`lru`, `quick_cache`, `sieve`) hinter Features. Bloom-Filter im Builder fest auf `BloomFilter::new(100_000, 0.01)`, unabhängig von der Eintragszahl. | `sstable/block_cache/*`, `builder/writer.rs` | Fehlerrate des Filters stimmt für große und kleine Runs nicht. Mehr Code, kein Nutzen. |
| B-12 | **Rauschen im Quelltext.** Kommentarbanner `FILE-CONTEXT`/`ANCHOR`/`STAND` mit veralteten Daten, gemischte Sprachen, `pub(super)`-Geflecht in `lsm/`. | diverse | Orientierung und Review erschwert. |

### 1.3 Was gut ist und übernommen wird

- Lint-Disziplin: `forbid(unsafe_code)`, `deny(unwrap_used, expect_used, panic, indexing_slicing)` in Produktionscode.
- Fail-stop bei `fsync`-Fehlern (INV-FSYNC-1): Handle vergiften, nie wiederholen. Bleibt als `INV-LOG-5`.
- Die Retention-Regel (alle Versionen über der Untergrenze behalten, die neueste darunter, Rest verwerfen) in `compaction/retention.rs`. Wird als reine Funktion mit Property-Tests übernommen.
- Atomares Schreiben über Temp-Datei, `fsync`, `rename`, Verzeichnis-`fsync` (`RealFs::write_atomic`).
- Der Testbestand als **Spezifikation**: Absturz-, Bitflip-, Platte-voll-, Abbruch-Szenarien (`chaos_*`, `crash_*`, `corrupted_files_fail_closed`) werden in die neue Harness überführt (Abschnitt 15.3).

---

## 2. Ziele, Nicht-Ziele, Leitprinzipien

### 2.1 Ziele

| ID | Ziel |
|---|---|
| Z-1 | Atomare Änderungen über mehrere Schlüssel **und Spaces** hinweg, mit Vorbedingungen (Compare-and-Swap), crash-sicher. |
| Z-2 | Konsistente Ansichten (`View`) und Zeitreise auf Stamp-Ebene für die oberen Schichten, mit Halteregeln (`Hold`) gegen Compaction. |
| Z-3 | **Kontrollierte, atomare, nachweisbare physische Löschung** von Schlüsselbereichen und ganzen Spaces. |
| Z-4 | Autark: keine internen Abhängigkeiten, keine externen im Standardbau. |
| Z-5 | Einfach: wenige Konzepte, kleine Module, ein Format je Dateiart, ein Compaction-Verfahren. |
| Z-6 | Beweisbar robust: deterministische Tests, Stromausfall-Simulation an **jedem** E/A-Schritt, Modelltests gegen ein Referenzmodell. |
| Z-7 | Eignung für Agenten- und LLM-Daten: viele kleine, atomare Schreibvorgänge, kurzlebige Ansichten, strukturierte Fehler mit Handlungsanweisung, Idempotenz auf Aufruferseite über `Guard::absent`. |
| Z-8 | Betreibbar: `verify`, konsistente Sicherung, Metriken, explizite Formatversion mit Pflicht-/optionalen Features. |
| Z-9 | Budgetierte Ergebnislieferung: Aufrufer können Leseumfang in Bytes begrenzen, vorab abschätzen und seitenweise stabil fortsetzen (Kontextfenster-Planung der oberen Schichten). |
| Z-10 | Erweiterbar ohne Formatbruch: zusätzliche, pro Run versionierte Index-Blöcke und ein kalter Speicherort sind im Format vorgesehen. |

### 2.2 Nicht-Ziele (bewusst)

Verteilter Konsens; Mehrprozess-Zugriff auf eine Instanz (eine Instanz, ein Prozess, `LOCK`-Datei); Transaktionspuffer, SSI, Intent-Locks, Mandanten- und Dokument-ID-Kodierung (obere Schichten); Merge-Operatoren (reserviert, siehe SD-12); Kompatibilität mit dem Altformat im Kern (SD-04); Windows mit voller Durability in Version 1 (A-02); `mmap`.

### 2.3 Leitprinzipien (Code-Regeln, im Review prüfbar)

1. Jede Datei auf der Platte hat **einen** Eigentümer-Mechanismus (Abschnitt 6) und wird nur über `Fs` berührt.
2. Zustandsänderung = **ein** Catalog-Edit. Wer „zwei Dinge atomar" braucht, packt sie in einen Edit.
3. Reihenfolge ist Vertrag: *Datei dauerhaft (Datei + Verzeichnis) → Edit dauerhaft → erst dann sichtbar machen → erst dann alte Dateien löschen.*
4. Dekodierung ist **total**: jede Byte-Folge ergibt `Ok` oder `Err(Corrupt)`, nie `panic`, nie unbegrenzte Allokation (Längen werden vor `Vec::with_capacity` gegen Limits geprüft).
5. Keine globalen Variablen. Keine `cfg(feature)` in Logik außer `encryption` und `testing`.
6. Dateien ≤ 400 Zeilen, Funktionen typisch ≤ 60 Zeilen, Modultiefe ≤ 3, keine Kommentarbanner. Invarianten stehen in `SPEC.md` und werden in Tests per ID zitiert.
7. Rustdoc auf Deutsch, Bezeichner auf Englisch (wie im Bestand).

---

## 3. Vokabular

Im Crate kommen weder Produkt- noch Fremdsystemnamen vor. Die Spalte „Bisher“ nennt die Bezeichnung im Altbestand.

| Neu | Bedeutung | Bisher |
| --- | --- | --- |
| **Storage** | Handle einer geöffneten Instanz | `LsmStorage` |
| **Space** | Benannter, physisch getrennter LSM-Baum (`SpaceId(u16)`) | Präfix-Reservierung in Schlüsseln |
| **Change** | Atomare Menge von Schreibvorgängen plus `Guard`s | Transaktion + `TxBuffer` |
| **Guard** | Vorbedingung innerhalb eines `Change` | `put_if_absent`, SSI-ReadSet |
| **Stamp** | Monotone Zählung angenommener Changes (u64, < 2^56) | `seq_no` + `TOMBSTONE_BIT` |
| **Receipt** | Quittung eines `Change` (`stamp`, `durable`) | – |
| **View** | Unveränderliche Sicht auf einen Stamp | Snapshot (`SnapshotRegistry`) |
| **Cursor** | Streaming-Iterator über einen Schlüsselbereich | `scan` (materialisiert) |
| **Hold** | Befristete Halteregel gegen das Verwerfen alter Versionen | `pin_checkpoint` |
| **Redo-Log** | Segmentiertes Wiederholungsprotokoll | WAL |
| **Run** | Unveränderliche sortierte Datei | SSTable |
| **Catalog** | Einzige Wahrheit über lebendige Dateien und Zustand | Manifest |
| **Erase** | Löschauftrag mit physischer Garantie | Purger (`compat/scanner`) |
| **Erasure Receipt** | Nachweis eines abgeschlossenen `Erase` | – |
| **Env** | Bündel der Ports (`Fs`, `Clock`, `Entropy`, `Runtime`, `Governor`, `Metrics`) | `contextra-env`, `Vfs` |
| **Graveyard** | Warteschlange für zu löschende, noch referenzierte Dateien | – |
| **Page / ResumeToken** | Byte-budgetierte Ergebnisseite und Fortsetzungsmarke (Schlüssel + Stamp) | `LIMIT/OFFSET`-artige Scans |
| **Estimate** | Schätzung von Eintrags- und Bytezahl eines Bereichs aus Indexdaten | – |
| **History** | Versionsverlauf eines Bereichs zwischen zwei Stamps | – |
| **Extension** | Zusätzlicher, pro Run gebauter Index-Block einer oberen Schicht | – |

---

## 4. Architektur

### 4.1 Modulschnitt

```
contextra-storage/
  src/lib.rs             Re-Exports der öffentlichen API
  src/api/               storage.rs  change.rs  view.rs  cursor.rs  page.rs  hold.rs
                         listener.rs config.rs  error.rs
  src/env/               fs.rs (Trait) real_fs.rs clock.rs entropy.rs
                         runtime.rs governor.rs metrics.rs
  src/codec/             varint.rs crc32c.rs bloom.rs tag.rs keyrange.rs bytes.rs
  src/redo/              frame.rs segment.rs writer.rs reader.rs recover.rs
  src/run/               builder.rs reader.rs block.rs footer.rs filter.rs
                         rangedel.rs extension.rs table_cache.rs block_cache.rs
  src/catalog/           edit.rs log.rs state.rs current.rs format.rs
  src/engine/            version.rs memtable.rs committer.rs flush.rs
                         compaction/{plan.rs,retain.rs,exec.rs}
                         maintenance.rs recovery.rs graveyard.rs views.rs
  src/erase/             plan.rs exec.rs registry.rs verify.rs
  src/crypto/            (Feature `encryption`) cipher.rs keys.rs
  src/testing/           (Feature `testing`) mem_fs.rs fault_fs.rs
                         manual_runtime.rs model.rs oracle.rs
```

### 4.2 Abhängigkeitsregeln (nur nach unten, per Test erzwungen)

```
 api ──► engine ──► { redo, run, catalog } ──► { codec, env }
 api ──► erase  ──► engine
 crypto (Feature) ──► { codec, env }; redo/run/catalog nutzen nur das Trait `Cipher`
 testing (Feature) ──► alles; nichts hängt von testing ab
```

`tests/layering.rs` liest alle `use crate::…`-Zeilen und schlägt fehl, wenn eine Kante gegen die Regel verstößt. Das ist billig und verhindert schleichende Kopplung.

### 4.3 Ports (öffentlich, semver-stabil)

```rust
pub trait Fs: Send + Sync + 'static {
    fn create_dir_all(&self, p: &Path) -> io::Result<()>;
    fn open(&self, p: &Path, mode: OpenMode) -> io::Result<Box<dyn File>>; // Read | ReadWrite | CreateNew
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
    fn remove_file(&self, p: &Path) -> io::Result<()>;
    fn remove_dir(&self, p: &Path) -> io::Result<()>;
    fn list_dir(&self, p: &Path) -> io::Result<Vec<DirEntry>>;           // Name + Länge
    fn sync_dir(&self, p: &Path) -> io::Result<()>;
    fn lock(&self, p: &Path) -> io::Result<Box<dyn LockGuard>>;          // exklusiv, nicht blockierend
    fn hard_link(&self, from: &Path, to: &Path) -> io::Result<()>;       // nur für Checkpoint (opt-in)
}
pub trait File: Send + Sync {
    fn read_at(&self, buf: &mut [u8], off: u64) -> io::Result<usize>;
    fn write_all_at(&self, buf: &[u8], off: u64) -> io::Result<()>;
    fn len(&self) -> io::Result<u64>;
    fn set_len(&self, len: u64) -> io::Result<()>;
    fn sync_data(&self) -> io::Result<()>;
    fn sync_all(&self) -> io::Result<()>;
}
pub trait Clock: Send + Sync { fn now_ms(&self) -> u64; fn monotonic_ns(&self) -> u64; }
pub trait Entropy: Send + Sync { fn fill(&self, dst: &mut [u8]) -> io::Result<()>; }
pub trait Metrics: Send + Sync { fn counter(&self, n: &'static str, v: u64); fn gauge(&self, n: &'static str, v: i64); fn observe(&self, n: &'static str, v: f64); }
```

`RealFs` nutzt ausschließlich `std` (`FileExt::read_at/write_at` unter Unix, `lock`/`try_lock` aus `std::fs::File`). Die Dateiverwaltung `write_atomic` (Temp → `sync_all` → `rename` → `sync_dir`) ist eine **Hilfsfunktion über `Fs`**, kein Trait-Mitglied, damit sie in `FaultFs` mitgetestet wird.

`Runtime` ist ein kleiner Thread-Pool (`spawn(Box<dyn FnOnce + Send>)`), der von vielen `Storage`-Instanzen geteilt werden kann. `Runtime::manual()` führt Jobs nur auf Aufruf aus (`run_pending()`): Tests steuern damit jede Verschränkung deterministisch. `Governor` bündelt Speicher- und E/A-Budgets (Memtable-Summe, Block-Cache, Compaction-Rate als Token-Bucket auf `Clock`).

---

## 5. Datenmodell und öffentliche API

### 5.1 Schlüssel, Tags, Reihenfolge

Interner Schlüssel = `user_key ‖ tag`, `tag = (stamp << 8) | kind`, sortiert **Schlüssel aufsteigend, Tag absteigend** (neueste Version zuerst). `kind ∈ {Put=1, Delete=2, RangeDelete=3, EraseRange=4}`. Der Löschmarker ist damit ein eigener Typ-Byte, nicht Bit 63; alle 142 Maskierungsstellen des Bestands entfallen. Stamps sind auf < 2^56 begrenzt (≈ 7·10^16, bei 10^9 Changes/s über zwei Jahre ausreichend; Überlauf wird als `Error::Exhausted` gemeldet, nie umgebrochen).

Der Space steckt **nicht** im Schlüssel, sondern in der Dateizuordnung (SD-07).

### 5.2 API (Skizze)

```rust
let st = Storage::open(Config::default(), Env::real("/var/lib/app"))?;
let facts = st.space("facts", SpaceOptions::default())?;   // legt an oder öffnet (Catalog-Edit)
let trace = st.space("trace", SpaceOptions::append_only().prefix_filter(8))?; // unveränderliches Protokoll

// Atomare Änderung über mehrere Spaces, mit Vorbedingungen
let r: Receipt = st.apply(
    Change::new()
        .guard(Guard::absent(idem, key_k))            // Idempotenz / Ref-CAS
        .guard(Guard::equals(refs, b"main", old))     // Compare-and-Swap
        .put(facts, k, v)
        .delete(facts, k2)
        .put(refs, b"main", new),
)?;                                                   // Err(Conflict{guard_index}) | Busy | Poisoned | …

// Lesen
let view = st.view();                                 // aktueller Stamp
let old  = st.view_at(stamp)?;                        // nur oberhalb der Hold-/View-Untergrenze
view.get(facts, k)?;                                  // Option<Bytes>
for kv in view.scan(facts, KeyRange::prefix(b"a/"), Dir::Fwd)? { let (k, v) = kv?; }
// Byte-budgetierte, fortsetzbare Seite (stabil innerhalb desselben Stamps)
let page = view.scan_page(facts, KeyRange::prefix(b"a/"), Dir::Fwd,
                          PageLimit::bytes(24_000).entries(200), resume)?;  // Page{ items, next: Option<ResumeToken> }
let est  = view.estimate(facts, KeyRange::prefix(b"a/"))?;               // Estimate{ entries, bytes }, nur aus Indexdaten
let hist = view.history(facts, KeyRange::prefix(b"a/"), StampRange::after(s0))?; // alle Versionen in (s0, view], inkl. Löschungen

// Halten, hören, löschen, sichern
let hold = st.hold(stamp, "analysis", Some(Duration::from_secs(600)))?;
st.subscribe(listener);                               // nach Durabilität, in Stamp-Reihenfolge
let q = st.erase(Erase::range(facts, KeyRange::prefix(b"subject/42/")).views(ViewPolicy::wait(5.secs())))?;
st.drop_space(facts)?;
st.checkpoint(dest_dir, CheckpointMode::Copy)?;
st.verify(VerifyScope::Online)?;
```

Eigenschaften der API:

- **Kein Teilzustand.** `apply` ist alles-oder-nichts. Doppelte Schlüssel innerhalb eines `Change` werden beim Aufbau zusammengelegt (letzter gewinnt). Ein `Change` erhält **einen** Stamp.
- **Guards werden unter der Schreibsperre gegen den neuesten Stand geprüft**, inklusive früherer Changes derselben Gruppe. `Guard::UnchangedSince(space, key, stamp)` genügt für optimistische Nebenläufigkeit der oberen Schichten.
- **Seiten statt Offsets.** `scan_page` begrenzt nach Bytes und/oder Einträgen und liefert eine `ResumeToken` (`space, letzter Schlüssel, Stamp, CRC`). Die Fortsetzung bindet an denselben Stamp; ist er nicht mehr haltbar, folgt `Err(Expired)`, nie eine stillschweigend andere Sicht.
- **`estimate`** summiert Einträge und Bytes aus den Index-Blöcken der betroffenen Runs (Genauigkeit: Blockgranularität an den Rändern; zählt überschattete Versionen mit, ist also eine obere Schranke) und schätzt den Memtable-Anteil durch Abtasten (≤ 1 024 Einträge). Kosten O(Index), keine Datenblöcke.
- **`history`** liefert alle Versionen (Puts, Deletes) eines Bereichs mit Stamp in `(von, bis]`, gefiltert über die Stamp-Bereiche der Runs. Voraussetzung: Untergrenze ≤ `von`, sonst `Err(HistoryPruned{oldest_available})`. Damit holen Abonnenten nach einem Neustart ab ihrer gespeicherten Stamp-Position auf.
- **Append-only Space** (`SpaceOptions::append_only()`): `Put` nur auf Schlüssel ohne lebende Version, `Delete`/`RangeDelete` verboten (`Err(Forbidden)`). Gedacht für Episoden-, Audit- und Ereignisprotokolle. Nur `Erase` mit ausdrücklichem `Erase::admin()` darf dort löschen.
- **Strukturierte Fehler** mit handlungsleitendem Text (Abschnitt 14).
- `Bytes` ist ein eigener, schlanker Typ (`Arc<[u8]>`-Ausschnitt mit `Deref<Target=[u8]>`), damit Block-Daten ohne Kopie geliefert werden können.

### 5.3 Operationsmatrix: was sich aus den Primitiven bauen lässt

| Fähigkeit der oberen Schicht | Gebaut aus |
|---|---|
| Referenz-Update mit Wiederholungssicherheit | `Guard::equals` auf Ref + `Guard::absent` auf Idempotenzschlüssel in **einem** `Change` |
| Mehrere Tabellen, Index, Ref atomar | ein `Change` über mehrere Spaces |
| Serialisierbare Sitzungen | `view_at(base)` + `Guard::UnchangedSince` oder Diff-Prüfung in der oberen Schicht |
| Zeitreise bis Stamp | `view_at` + `Hold` mit Ablauf |
| Konsistenter Lesezugriff über Index und Daten | eine `View` über alle Spaces |
| Abgeleitete Indizes folgen Änderungen | `subscribe` (Stamp-geordnet, nach Durabilität) |
| Datenschutz-Löschung eines Subjekts | `Erase::range` (Abschnitt 12) |
| Mandant/Repository löschen | `drop_space` oder Instanz-Verzeichnis löschen nach Schlüsselvernichtung |
| Backup, Test-Klon | `checkpoint` |
| Token-/Byte-budgetiertes Paketieren von Ergebnissen | `estimate` zur Planung, `scan_page` zur Auslieferung, `ResumeToken` zur Fortsetzung |
| Abgeleitete Indizes holen nach Neustart auf | `history` ab gespeicherter Stamp-Position, danach `subscribe` |
| Manipulationsarme Episoden-/Audit-Protokolle | append-only Space |
| Zusammen Gebrauchtes liegt nebeneinander | Schlüsselgestaltung mit Cluster-Präfix, Präfix-Filter, Run-Teilung an Präfixgrenzen (10.5) |
| Zusätzliche Run-Indizes (Volltext, Vektor, Graph) | `Extension`-Blöcke (10.4) |

---

## 6. Dateilayout

```
<root>/
  FORMAT              Magic, Formatversion, required/optional Feature-Bits, instance_id, Schlüsselmodus
  LOCK                exklusive Sperre (über Fs::lock)
  CURRENT             Zeiger auf aktive Catalog-Datei (atomar ersetzt)
  CATALOG-000007      Snapshot-Record + Edit-Records
  KEYS                (nur mit Verschlüsselung) umhüllte Daten-Schlüssel je Space, atomar ersetzt
  redo/
    redo-0000000000000012.log
  spaces/
    0001/
      0000000000000a3f.run
    0002/
      …
```

Regeln:

- **Run-Dateien und Redo-Segmente werden über numerische IDs benannt**, nie über Pfad-Strings im Catalog.
- Temporäre Dateien tragen die Endung `.building` und eine eindeutige ID aus dem Catalog-Zähler (nicht die Prozess-ID, die nach Neustart kollidieren kann). Beim Öffnen werden alle `.building` entfernt.
- **Verwaiste Dateien** (Run/Redo-Segment, das der Catalog nicht nennt) werden beim Öffnen und danach periodisch gelöscht. Das ist sicher, weil nur ein Catalog-Edit eine Datei lebendig macht (`INV-CAT-1`).
- Ein Space = ein Verzeichnis. Das Entfernen eines Spaces ist ein Edit plus `remove_dir` nach dem Löschen der Runs.

---

## 7. Formate

Alle Zahlen little-endian. Alle Prüfsummen CRC32C (in-Crate implementiert, ≈ 60 Zeilen, Tabellen-Variante). Jede Dateiart hat **genau eine** Version (`1`) und Feature-Bits.

### 7.1 `FORMAT`

```
"CTXS" | version u16 | required u64 | optional u64 | instance_id [16] | created_ms u64 | crc32c u32
```

Pflicht-Bits: `ENCRYPTED = 1<<0`. Unbekanntes Pflicht-Bit ⇒ `Error::Unsupported{feature}`, kein Öffnen. Unbekanntes optionales Bit wird ignoriert. Wird beim Anlegen einmal per `write_atomic` geschrieben.

### 7.2 Redo-Log

Segmentdatei:

```
Header (32 B): "CTXL" | version u16 | flags u16 | segment_id u64 | first_stamp u64 | chain_seed u32 | crc32c u32
Frame*       : len u32 | stamp u64 | op_count u32 | payload[len-12] | chk u32
chk          = crc32c(seed = vorheriges chk bzw. chain_seed, len ‖ stamp ‖ op_count ‖ payload)
Op           : kind u8 | space u16 | klen varint | key | vlen varint | value     (kind: Put, Delete, RangeDelete, EraseRange)
```

- **Ein Frame = ein `Change`.** Beim Replay ist ein Frame entweder vollständig gültig oder er endet den Log. Es gibt keinen Zwischenzustand (`INV-LOG-2`).
- Die Kettenprüfsumme erkennt vertauschte oder aus einem alten Segment stammende Frames. Gegen **böswillige** Veränderung schützt nur der Verschlüsselungsmodus (AEAD je Frame, Zusatzdaten = `segment_id ‖ frame_index`); das ist die ehrliche Bedrohungsgrenze (CRC ist keine Authentisierung).
- **Jede Prozessinstanz beginnt ein neues Segment** und hängt nie an ein altes an (`INV-LOG-4`). Das beseitigt die Frage „halb geschriebener Rest vor meinem ersten Frame".
- Rotation bei 64 MiB (`Config::redo_segment_bytes`).
- Ein Segment ist löschbar, wenn **alle** Spaces `flushed_stamp ≥ max_stamp(segment)` haben (Catalog-Zustand).

### 7.3 Run

```
Run    := DataBlock* ExtensionBlock* RangeDelBlock? FilterBlock PrefixFilterBlock? IndexBlock MetaBlock DirectoryBlock Footer
Block  := payload | codec u8 | crc32c u32          (verschlüsselt: AEAD-Tag statt crc32c)
Directory    : Liste (kind u16 | offset u64 | len u32) für index, filter, prefix_filter, rangedel, meta und jeden Extension-Block (kind = Extension-ID)
Footer (28 B): "CTXR" | version u16 | flags u16 | directory(offset u64, len u32) | crc32c u32 | "CTXR"
```

- **DataBlock** (Ziel 4 KiB): präfixkomprimierte Einträge `shared varint | unshared varint | vlen varint | tag u64 | key_suffix | value`, Neustartpunkt alle 16 Einträge, am Ende das Array der Neustartoffsets.
- **IndexBlock:** je Block der kürzeste Trenner ≥ letzter Schlüssel → `(offset, len, entries u32, bytes u32)`. Die Zähler tragen `estimate` (Abschnitt 5.2).
- **DirectoryBlock:** Ein Verzeichnis statt fest verdrahteter Handles im Footer. Neue Block-Arten (Extensions) brauchen dadurch **keine** Formatänderung; unbekannte optionale Blöcke werden ignoriert, unbekannte Pflichtblöcke (Flag im Eintrag) verweigern das Öffnen.
- **FilterBlock:** Bloom-Filter über Nutzerschlüssel, **Größe aus der tatsächlichen Eintragszahl** (`bits_per_key = 10`), nicht fest verdrahtet (behebt B-11).
- **PrefixFilterBlock** (optional, `SpaceOptions::prefix_filter(n)`): Bloom-Filter über die ersten `n` Schlüsselbytes. Er lässt Präfix-Scans (Cluster, Subjekt, Tabelle) Runs ohne Treffer überspringen, was der Punktfilter nicht kann.
- **ExtensionBlock** (optional): von einer oberen Schicht über das `Extension`-Trait pro Run erzeugter Block (z. B. Postings, Vektorsegment). Er entsteht beim Flush und wird bei der Compaction aus den Eingaben neu gebaut oder zusammengeführt (10.4). Weil Runs unveränderlich sind, ist jeder Extension-Block automatisch an den Datenstand seines Runs gebunden.
- **RangeDelBlock:** sortierte, fragmentierte Bereichs-Löschmarker `(start, end, tag)`; wird beim Öffnen vollständig in den Speicher gelesen (klein). Ein Flag im Meta vermerkt „enthält Bereichsmarker", damit Punktzugriffe Runs ohne Marker überspringen.
- **MetaBlock:** `space_id`, `run_id`, Eintragszahl, kleinster/größter Nutzerschlüssel, kleinster/größter Stamp, Anzahl Bereichs- und Erase-Marker, `created_ms` (aus `Clock`), Feature-Bits.
- **Kompression:** `codec` ist 0 (keine). Ein `BlockCodec`-Trait existiert, eine Implementierung kommt erst bei nachgewiesenem Bedarf (SD-13).
- **Leseweg:** `read_at` über `Fs`, kein `mmap`. Der Footer wird beim Öffnen geprüft, die Blöcke erst beim Lesen (Prüfsumme je Block).

### 7.4 Catalog

```
Datei  : "CTXC" | version u16 | Record*
Record : len u32 | crc32c u32 | tag u8 (Snapshot | Edit) | payload (TLV-Felder)
```

Ein **Edit** ist ein einzelner Record mit optionalen Feldern (TLV, jedes mit Pflicht-/optional-Bit):

| Feld | Bedeutung |
|---|---|
| `add_runs[RunMeta]` | `run_id, space, level, tier, min_key, max_key, min_stamp, max_stamp, bytes, entries, flags` |
| `remove_runs[run_id]` | |
| `space_flushed[(space, stamp)]` | Memtable bis `stamp` ist in Runs |
| `redo_floor` | kleinstes noch benötigtes Segment |
| `next_run_id`, `next_segment_id` | Zähler (Hochwassermarke) |
| `spaces_add[(id, name, options)]`, `spaces_drop[id]` | |
| `erase_pending_add[(id, space, range)]`, `erase_done[id]`, `erase_swept[id]` | Löschprotokoll (Abschnitt 12) |
| `erased_ranges_add / _remove` | Register gelöschter Bereiche (Abschnitt 12.6) |

- Die Run-Metadaten stehen **im Catalog**. Auswahl für Compaction und Löschung läuft ohne eine Datei zu öffnen (löst B-8). Die Reihenfolge von Level-0-Runs ergibt sich aus `max_stamp`, nicht aus einem Rang.
- **Rollover** (Größe > 8 MiB oder bei jedem Öffnen mit mehr als 1 000 Edits): neue Datei `CATALOG-(n+1)` mit einem Snapshot-Record schreiben, `sync_all`, `CURRENT` per `write_atomic` auf die neue Datei zeigen lassen, `sync_dir`, alte Datei löschen.
- **Lesen:** ein beschädigter oder halber **letzter** Record wird verworfen (Torn Write), jeder frühere Defekt ist `Error::Corrupt` (Fail-closed, kein „Wiederauferstehen" gelöschter Runs).

---

## 8. Schreibpfad

### 8.1 Zustandsmodell

```
Storage
 ├─ state: RwLock<Arc<Version>>            Lesezugriff klont das Arc, Haltezeit Nanosekunden
 ├─ committer: Mutex<CommitterState>       genau eine aktive Gruppe
 ├─ views: ViewRegistry                    schwache Referenzen, Untergrenze, Widerruf
 ├─ holds: HoldRegistry
 ├─ maintenance: Single-Flight-Job-Slot
 └─ env: Env

Version (unveränderlich)
 └─ spaces: Map<SpaceId, SpaceState{ active: Arc<Memtable>, frozen: Vec<Arc<Memtable>>,
                                      levels: Vec<Vec<Arc<RunHandle>>>, flushed_stamp, erased_ranges }>
```

Es gibt **drei** Sperren statt acht Rängen: `state` (kurz), `committer` (eine aktive Gruppe), und der Wartungs-Slot (Einzelablauf). Sie werden nie ineinander verschachtelt gehalten, außer `committer → state.write` beim Rotieren der Memtable (einzige Reihenfolge, dokumentiert und per `loom` getestet).

### 8.2 `apply` (Flat-Combining Group-Fsync)

```
apply(change):
  1. validate: Limits, Spaces vorhanden, Duplikate zusammenlegen            (ohne Sperre)
  2. Ticket in Warteschlange; wer den Committer-Mutex bekommt, wird Leiter, die anderen warten auf ihr Ticket
Leiter:
  3. Backpressure-Prüfung (8.4)
  4. Warteschlange leeren (≤ 1 000 Changes / 8 MiB)
  5. je Change in Reihenfolge: Guards gegen (neueste Version + frühere Changes der Gruppe) prüfen
       Fehlschlag ⇒ nur dieser Change erhält Err(Conflict), verbraucht keinen Stamp
       Erfolg    ⇒ Stamp = next_stamp++ ; Frame kodieren
  6. alle Frames mit EINEM write_all_at an das Redo-Segment anhängen
  7. Durability::Always ⇒ sync_data ; Fehler ⇒ Poison, alle Tickets der Gruppe Err(Poisoned), NICHTS in Memtables
  8. Memtables in Stamp-Reihenfolge füllen, dann visible_stamp (Release) veröffentlichen
  9. Tickets mit Receipt{stamp, durable} wecken
 10. Listener-Warteschlange füttern (Zustellung auf Runtime, in Stamp-Reihenfolge, nach Durabilität)
 11. Memtable über Grenze? ⇒ rotieren (neue active, alte frozen), Flush-Job anmelden
```

Folgen:

- **Memtables enthalten nur dauerhafte Daten** (bei `Durability::Always`), `INV-VIS-1`. Es gibt keinen „Rückbau" nach einem Fehler; der Leistungsverlust durch zweiphasiges Vorgehen ist ein Schreibvorgang auf Memtable nach dem Fsync (billig).
- Ein einziger `fsync` je Gruppe. Gruppengröße ergibt sich natürlich aus der Last (kein Zeitfenster-Parameter, entfernt `group_commit_window_micros`).
- `Durability::Interval(d)`: Quittung sofort mit `durable=false`, Wartung führt alle `d` ein `sync_data` aus. `Durability::Never`: nur für Tests und Wegwerf-Instanzen. `Storage::sync()` erzwingt.
- Stamps sind lückenlos über die **angenommenen** Changes.

### 8.3 Sichtbarkeit und Views

`View` = `(Arc<Version>, stamp)`. Lesen ignoriert Einträge mit größerem Stamp. Da `visible_stamp` erst nach dem Füllen der Memtables veröffentlicht wird, sieht keine View einen halben Change (`INV-VIS-2`). Views sind in der `ViewRegistry` (schwache Referenzen). Die **Untergrenze** `floor = min(Stamp aller lebenden Views, Stamp aller Holds)` steuert die Compaction (`INV-RET-1`). Holds haben einen Ablauf (`Clock`), abgelaufene werden von der Wartung entfernt (keine Zombie-Holds).

### 8.4 Gegendruck

Maßgeblich sind drei Größen: Anzahl eingefrorener Memtables, Level-0-Run-Anzahl, ausstehende Compaction-Bytes. Stufen (Konfiguration):

| Stufe | Auslöser | Wirkung |
|---|---|---|
| Normal | unter Verlangsamung | keine |
| Verlangsamt | L0 ≥ 8 oder frozen ≥ 2 | Leiter wartet bis zu `stall_slice` je Gruppe |
| Gestoppt | L0 ≥ 12 oder frozen ≥ 4 oder Governor voll | `apply` wartet höchstens `stall_timeout`, danach `Err(Busy{retry_after})` |

Strukturiertes `Busy` ist für Agenten wichtig: wiederholbar, mit Hinweis.

---

## 9. Lesepfad

- `get`: aktive Memtable → eingefrorene (neu nach alt) → Level 0 (neu nach alt, nur Runs deren Schlüsselbereich passt) → je tieferes Level genau ein Run (Binärsuche über Catalog-Metadaten). Pro Run: Bloom-Filter, Index, ein Block. Bereichs-Löschmarker aus Memtables und aus den berührten Runs werden gegen die Versionen geprüft (Marker sichtbar, wenn `marker.stamp ≤ view.stamp`; verdeckt, wenn `entry.stamp < marker.stamp`).
- `scan` liefert einen `Cursor` (Streaming, mischend). Die Memtable wird **in Blöcken zu 64 Einträgen mit erneutem Aufsetzen** (`seek_after(last_key)`) gelesen, damit keine Lesesperre über die Dauer eines Scans gehalten wird. Bereichs-Löschmarker für den Scan-Bereich werden einmal eingesammelt, fragmentiert und als Intervallliste mit maximalem Stamp gegen jeden Eintrag geprüft.
- `scan_page`, `estimate` und `history` (5.2) nutzen dieselbe Zusammenführung. `estimate` liest ausschließlich Index-Blöcke (im Cache gehalten), `history` verwirft Runs, deren `[min_stamp, max_stamp]` den Bereich `(von, bis]` nicht schneidet.
- **Präfix-Scans** fragen zuerst den `PrefixFilterBlock` der Runs ab (falls vorhanden).
- **`TableCache`** hält höchstens `max_open_runs` offene Dateien (löst Dateideskriptor-Grenzen). **`BlockCache`**: genau eine Implementierung (CLOCK-Verfahren, geteilt über `Governor`), Schlüssel `(run_id, block_no)`, mit `invalidate_run(run_id)`.
- Lesefehler (`Corrupt`, `Io`) werden **immer** als Fehler gemeldet, nie als „leer" (Gegenstück zu B-9a).

---

## 10. Wartung: Flush und Compaction

Es läuft **höchstens ein Wartungsjob je Storage-Instanz** (Single-Flight-Slot auf der `Runtime`). Reihenfolge der Priorität: `Erase` > `Flush` > `Compaction` (L0 vor tieferen Leveln) > `Cleanup` (Graveyard, verwaiste Dateien, abgelaufene Holds). Das macht das Verhalten nachvollziehbar und schließt Wechselwirkungen (Compaction während `Erase`) per Konstruktion aus.

### 10.1 Flush (je Space)

```
1. frozen Memtable → Run bauen (.building), alle Blöcke, Filter, Index, Meta, Footer
2. Run-Datei sync_all, Verzeichnis sync_dir
3. Edit { add_runs[L0], space_flushed(space, max_stamp), redo_floor, next_run_id } → Catalog append + sync_data   ◄── Durability-Punkt
4. neue Version veröffentlichen (frozen entfernen, Run hinzufügen)
5. Redo-Segmente unterhalb redo_floor ins Graveyard
```

`redo_floor` wird nur angehoben, wenn **alle** Spaces die Segmente abdecken. Hinkt ein selten beschriebener Space hinterher und das Redo-Volumen übersteigt `max_redo_bytes` (Standard 512 MiB), erzwingt die Wartung dessen Flush (auch mit kleinem Run). Das ist der Preis getrennter Spaces und wird in `stats()` als `redo_lag_bytes` sichtbar.

### 10.2 Compaction (ein Verfahren: Leveled)

- **Planer = reine Funktion** `plan(&Version, &Params, floor) -> Option<Plan>`. Keine E/A, vollständig property-testbar. Die adaptive, kostenbasierte Planung (`compaction/adaptive/*`) entfällt (SD-11).
- Auslöser: L0-Anzahl ≥ 4 ⇒ alle L0-Runs plus überlappende L1-Runs. Tiefere Level: Größe > `target(level)` (Faktor 10) ⇒ Run mit höchstem Score plus überlappende Runs des Zielniveaus. Ausgabe wird bei `target_run_bytes` (64 MiB) geteilt.
- **Aufbewahrung** (`retain.rs`, reine Funktion, aus dem Bestand übernommen): je Schlüssel alle Versionen mit Stamp > `floor` behalten, die neueste mit Stamp ≤ `floor` behalten, ältere verwerfen. Löschmarker werden nur verworfen, wenn unterhalb kein Run den Bereich überlappt (aus Catalog-Metadaten bestimmt) **und** kein Hold/View darunter liegt.
- **Installation:** Ausgabe-Runs `sync_all` + `sync_dir`, dann **ein** Edit `{add_runs, remove_runs}` (ersetzt `Replace`), dann neue Version, dann Eingabe-Runs per `Arc<RunHandle>`-Drop ins Graveyard.
- Rate-Begrenzung über `Governor` (Token-Bucket auf `Clock`).

### 10.3 Graveyard

`RunHandle` und Redo-Segment-Handles führen ein `obsolete`-Flag. Beim Drop der letzten Referenz mit gesetztem Flag wird der Pfad in das Graveyard gelegt, **ohne E/A im Drop**. Die Wartung löscht per `Fs::remove_file`, danach `sync_dir`. Fehlschläge werden wiederholt und als Metrik/Fehler gemeldet (nicht nur geloggt wie in B-9d). Nach Absturz räumt die Verwaisten-Suche beim Öffnen auf (`INV-CAT-4`).

### 10.4 Extensions (zusätzliche Index-Blöcke je Run)

```rust
pub trait Extension: Send + Sync {
    fn id(&self) -> u16;                                   // stabile Block-Art
    fn build(&self, entries: &mut dyn Iterator<Item = Entry>) -> io::Result<Vec<u8>>;
    fn merge(&self, inputs: &[&[u8]], survivors: &mut dyn Iterator<Item = Entry>) -> io::Result<Vec<u8>>;
}
```

Der Kern liefert **keine** Implementierung (kein Volltext, keine Vektoren). Er garantiert nur: der Block wird in `flush` und Compaction-Installation im selben Run-Bau erzeugt, steht im `DirectoryBlock`, wird mit dem Run atomar lebendig und mit dem Run gelöscht. Dadurch sind abgeleitete Indizes ohne eigene Versionierung stets konsistent mit ihren Daten. `merge` darf Eingabeblöcke wiederverwenden, muss aber Überlebende (nach `retain`) berücksichtigen, damit gelöschte Einträge nicht über den Index sichtbar bleiben. Die Schnittstelle wird erst in Phase F10 eingefroren (A-10); Format und Verzeichnis sind schon in F3 vorbereitet.

### 10.5 Lokalität

Die Reihenfolge ist die Schlüsselreihenfolge. **Wer zusammen gelesen wird, liegt zusammen, wenn die obere Schicht dem Schlüssel einen Cluster-Präfix voranstellt** (z. B. Cluster-ID, danach Zeit, danach Hash). Der Kern unterstützt das mit `SpaceOptions::locality_prefix(n)`: Compaction teilt Ausgabe-Runs bevorzugt an Präfixgrenzen (innerhalb ±25 % der Zielgröße), damit ein Cluster nicht über viele Runs streut. Ein Umordnen im Betrieb (Re-Clustering) ist eine Re-Key-Operation der oberen Schicht (neue Schlüssel schreiben, alte per `Erase`/`Delete` entfernen) und gehört wegen der Gefahr des Aufschaukelns bei wechselnder Last dorthin (Hysterese, Lastfenster). Der Kern selbst ordnet nie nach Zugriffsmustern um.

### 10.6 Kalter Speicher (vorgesehen, Phase F10)

`RunMeta.tier ∈ {Local, Cold}`. Ein `ColdTier`-Port (`upload(run)`, `read_range(run, off, len)`, `delete(run)`) wird als `File`-Implementierung hinter `TableCache` eingebunden; Runs sind unveränderlich und deshalb ohne Koordination hochladbar. **Folge für die Löschung:** E6 muss `delete` des Ports einschließen, und `Physical` kann nur zugesichert werden, soweit der Port Löschung garantiert; für kalte Runs ist `Cryptographic` die einzige belastbare Stufe (siehe 12.2).

---

## 11. Recovery

### 11.1 Ablauf beim Öffnen (deterministisch, 9 Schritte)

```
 1. Fs::lock(LOCK); scheitert ⇒ Err(Locked)
 2. FORMAT lesen; unbekanntes Pflicht-Feature ⇒ Err(Unsupported)
 3. CURRENT lesen ⇒ Catalog-Datei; Records lesen (letzter Torn-Record verworfen, sonst Corrupt)
    Catalog-Zustand aufbauen (Spaces, Run-Metadaten, flushed_stamps, redo_floor, pending erases, erased_ranges)
 4. *.building löschen; Run-Dateien und Redo-Segmente, die der Catalog nicht nennt, löschen (Verwaisten-Suche)
 5. Fehlende, vom Catalog genannte Run-Datei ⇒ Err(Corrupt{MissingRun}) (Fail-closed, kein Raten)
 6. Redo-Segmente ab redo_floor in Id-Reihenfolge lesen:
      gültige Frames anwenden, aber je Op nur wenn stamp > space.flushed_stamp
      ungültiges/halbes Frame im LETZTEN Segment ⇒ Rest abschneiden (Heilung), Warnung
      ungültiges Frame in früherem Segment ⇒ Err(Corrupt) (Mittendefekt, Fail-closed)
 7. next_stamp = max(letzter Redo-Stamp, max flushed_stamp, max Run-Stamp) + 1; visible_stamp = next_stamp − 1
 8. Neues Redo-Segment beginnen (nie anhängen); optional sofort Catalog-Rollover
 9. Pending Erases (Catalog ∪ Erase-Frames im Redo ohne erase_done) in den Wartungs-Slot legen (Roll-forward, Abschnitt 12.4)
```

Die Reihenfolge der Schritte 3→4 ist wichtig: **erst** den Catalog als Wahrheit lesen, **dann** aufräumen.

### 11.2 Absturzmatrix (Soll-Zustand je Unterbrechungspunkt)

| Unterbrechung | Zustand auf Platte | Ergebnis nach Neustart |
|---|---|---|
| vor Redo-`write` | nichts | Change nicht vorhanden, nie quittiert |
| mitten im `write` / vor `fsync` | halbes oder ungeflushtes Frame | Frame wird abgeschnitten oder fehlt, nie quittiert |
| nach `fsync`, vor Quittung | Frame dauerhaft | Change vorhanden (zulässig: nie quittiert, aber dauerhaft) |
| Flush: nach Run-`sync`, vor Edit | verwaister Run | Verwaisten-Suche löscht, Memtable aus Redo wiederhergestellt |
| Flush: nach Edit, vor Version | Run im Catalog | Run lebendig, Redo-Frames mit Stamp ≤ `flushed_stamp` werden übersprungen |
| Compaction: nach Ausgabe, vor Edit | verwaiste Ausgabe | gelöscht, Eingaben unverändert |
| Compaction: nach Edit, vor Unlink | Eingaben verwaist | gelöscht (Verwaisten-Suche) |
| Catalog-Rollover: nach neuer Datei, vor `CURRENT` | alte `CURRENT` gültig | neue Datei verwaist, gelöscht |
| Catalog-Rollover: nach `CURRENT`, vor Löschen alter Datei | beide vorhanden | alte Datei verwaist, gelöscht |
| `fsync`-Fehler zur Laufzeit | unbestimmt | Instanz vergiftet, kein Retry, Neustart führt Recovery aus |

Diese Tabelle wird zu einem **Test** (15.2): `FaultFs` bricht an jedem E/A-Index ab und prüft den Soll-Zustand.

---

## 12. Löschung (Erasure)

Dieser Abschnitt ist die Antwort auf „atomare physische Löschung durch sauberes Anlegen". Das saubere Anlegen besteht aus drei Maßnahmen in der Struktur: **(1)** Spaces sind physisch getrennt (Löschgrenze = Verzeichnis), **(2)** der Catalog kennt Schlüsselbereich und Stamp-Bereich jedes Runs (Löschplanung ohne Dateizugriff), **(3)** Dateien werden ausschließlich nach einem durablen Edit gelöscht (Reihenfolge ist Vertrag).

### 12.1 Was „atomar" und „physisch" hier heißt

- **Logisch atomar:** Ab Stamp `S` sieht keine View mit Stamp ≥ `S` irgendetwas aus dem Löschbereich, für alle Schlüssel zugleich (der Löschmarker ist ein Eintrag in einem Redo-Frame).
- **Physisch atomar:** Es gibt **genau einen** Durability-Punkt (einen Catalog-Edit), vor dem die alten Dateien lebendig und nach dem sie nicht mehr referenziert sind. Ein Absturz davor lässt den Vorgang fortsetzen, ein Absturz danach das Löschen der Dateien (Roll-forward, nie Rückabwicklung).
- **Nachweisbar:** Eine `ErasureReceipt` wird erst ausgestellt, nachdem die Dateien gelöscht und das Verzeichnis synchronisiert wurde.

### 12.2 Stufen und ehrliche Garantien

| Stufe | Verfahren | Schützt gegen | Schützt **nicht** gegen |
|---|---|---|---|
| `Logical` | Löschmarker | Lesen über die API | Dateiinhalt auf Platte |
| `Physical` (Standard) | Runs ohne den Bereich neu schreiben, Catalog-Edit, Unlink, Verzeichnis-`fsync`, Redo-Segmente entfernen | Lesen von Dateien im Verzeichnis, Wiederauftauchen nach Absturz | Reste im Dateisystem-Journal, Flash-Wear-Leveling, Snapshots/Backups außerhalb, Hardlinks |
| `Scrubbed` | wie `Physical`, vor dem Unlink Datei mit Nullen überschreiben und synchronisieren | zusätzlich einfache Wiederherstellung gelöschter Blöcke auf klassischen Datenträgern | Copy-on-Write-Dateisysteme und SSDs (Überschreiben ist dort **nicht** garantiert); wird als Best-Effort dokumentiert |
| `Cryptographic` | Daten-Schlüssel des Spaces vernichten (`KEYS` atomar ersetzen), **bevor** Dateien gelöscht werden | alle Reste, auch Backups ohne Schlüssel, Flash-Reste | Kopien des Schlüssels |

Konsequenz: **Echte Unwiederbringlichkeit auf modernen Datenträgern liefert nur `Cryptographic`.** Daher sieht das Design vor, dass sensible Daten verschlüsselt liegen (Abschnitt 13) und `Physical` die Rückstände im Dateisystem minimiert. Das wird in der Dokumentation nicht beschönigt.

Feinere Schlüsselvernichtung als „Space" (z. B. je Person) leistet die obere Schicht, indem sie Werte vor dem Speichern mit einem Subjektschlüssel verschlüsselt. Der Speicher stellt dafür `Erase::range` bereit, um Reste zu entfernen, und `Cryptographic` für ganze Spaces.

### 12.3 Protokoll `Erase::range(space, range)` mit Stamp `S`

```
E0  Vorprüfung: Space existiert; Bereich gültig; Views/Holds mit Stamp < S gemäß ViewPolicy / HoldPolicy
E1  Redo-Frame mit Op EraseRange(space, range) schreiben (Teil des normalen apply-Pfads), fsync   ◄── LOGISCHER Punkt, Stamp S
      Ab hier verdeckt der Marker alle Einträge mit Stamp < S im Bereich für Views ≥ S
E2  Wartungs-Slot belegen (keine Compaction/Flush läuft parallel)
    Flush-Barriere: alle Memtables aller Spaces bis S flushen, redo_floor anheben,
                    Redo-Segmente mit max_stamp ≤ S ins Graveyard   (Redo mischt Spaces, daher alle Spaces)
E3  Runs des Spaces wählen, deren [min_key, max_key] den Bereich überlappen (aus Catalog-Metadaten)
E4  Je Run: streamen, alle Einträge mit Stamp ≤ S im Bereich verwerfen (unabhängig von Holds, wenn Policy Override),
            Ausgabe-Run bauen (sync_all), Verzeichnis sync_dir; vollständig überdeckte Runs erzeugen keine Ausgabe
E5  EIN Edit { add_runs, remove_runs, erase_done(S), erased_ranges_add(range, S) }  → Catalog append + sync_data   ◄── PHYSISCHER Punkt
E6  Neue Version veröffentlichen; BlockCache.invalidate_run für entfernte Runs;
    entfernte Runs ins Graveyard; Graveyard wartet auf Rückgabe aller Referenzen (ViewPolicy),
    (Scrubbed: überschreiben + sync) → remove_file → sync_dir
E7  Edit { erase_swept(S) } (nicht kritisch), ErasureReceipt{ id=S, level, runs_rewritten, runs_removed,
    redo_segments_removed, bytes_freed, completed_ms } zurückgeben
```

- Beim `Erase` mit Stufe `Cryptographic` für einen ganzen Space entfällt E3–E4, E5 enthält zusätzlich `spaces_drop` und der Schlüssel in `KEYS` wird **vor** E5 vernichtet (`INV-ERA-3`).
- `drop_space` ist der Schnellpfad: Schlüssel vernichten (falls verschlüsselt), ein Edit `{spaces_drop, remove_runs[alle]}`, Graveyard, `remove_dir`.
- Schreibzugriffe nach `S` im selben Bereich bleiben erlaubt; E4 verwirft nur Stamp ≤ `S`.

### 12.4 Recovery der Löschung (Roll-forward)

Die Menge der offenen Löschungen ist `Catalog.erase_pending ∪ {EraseRange-Frames im Redo nach dem Anker} − erase_done`. Beim Öffnen werden sie in den Wartungs-Slot gelegt und ab E2 fortgesetzt. E3–E5 sind idempotent (Auswahl aus aktuellem Catalog; ein bereits durchgeführter E5 ist an `erase_done` erkennbar). E6/E7 sind idempotent über die Verwaisten-Suche. Ein Flush, der einen Erase-Marker in einen Run schreibt, trägt `erase_pending_add` im selben Edit.

### 12.5 Nebenläufigkeit mit Views und Holds

| Policy | Verhalten |
|---|---|
| `ViewPolicy::Wait(timeout)` (Standard) | Löschen der Dateien wartet, bis alle Views mit Verweis auf die alten Runs zurückgegeben sind. Nach Timeout: `Err(Blocked{views})`; der Vorgang bleibt in E6 offen und wird wiederholt (der Erfolg des logischen Teils ist bereits garantiert). |
| `ViewPolicy::Revoke` | Betroffene Views werden **widerrufen**: ihr nächster Zugriff liefert `Err(Revoked)`; sie geben ihre Version frei, danach werden die Dateien gelöscht. |
| `HoldPolicy::Fail` (Standard) | Ein Hold mit Stamp < `S` über dem Bereich ⇒ `Err(HoldBlocked)`. |
| `HoldPolicy::Override` | Hold wird für den Bereich ungültig (Datenschutz hat Vorrang). |

### 12.6 Kein stilles Fälschen der Historie

Der Catalog führt je Space ein Register `erased_ranges: [(range, S)]`. Liest eine View mit Stamp < `S` in einen dort verzeichneten Bereich, antwortet sie mit `Err(Erased{range, erased_at: S})`. Es gibt **keine Fallback-Antwort und keine leere Antwort**. Einträge werden entfernt, sobald die Untergrenze (`floor`) ≥ `S` ist.

### 12.7 Invarianten und Prüfung

| ID | Invariante |
|---|---|
| `INV-ERA-1` | Logische Atomarität: für jede View ≥ `S` ist der Bereich vollständig leer oder (View < `S`) vollständig `Erased`. Es gibt keine Mischsicht. |
| `INV-ERA-2` | Nach `erase_done(S)` enthält weder ein lebendiger Run noch ein lebendiges Redo-Segment eine Version im Bereich mit Stamp ≤ `S`. |
| `INV-ERA-3` | Schlüssel vor Datei: Der Daten-Schlüssel wird vernichtet und dauerhaft, bevor die zugehörigen Dateien gelöscht werden. |
| `INV-ERA-4` | Nur Roll-forward: Ein begonnener `Erase` wird nach jedem Absturz fortgesetzt oder ist nie begonnen. |
| `INV-ERA-5` | Reads unterhalb `S` in den Bereich liefern `Erased`, nie Daten und nie „leer". |

**Prüfwerkzeuge:**

1. `Storage::verify_erasure(id)`: prüft `INV-ERA-2` anhand Catalog-Metadaten und Stichproben-Scan der noch lebendigen Runs mit Überlappung, liefert Befund je Datei.
2. **`ErasureOracle` (Feature `testing`):** schreibt Schlüssel/Werte mit einem eindeutigen Muster, führt `Erase` aus und durchsucht danach **alle Bytes aller Dateien im Verzeichnis** (inkl. `MemFs`-Zustand, `.building`, Redo) nach dem Muster. Bei ausgeschalteter Verschlüsselung muss die Trefferzahl 0 sein.
3. **Crash-Matrix über E1–E7** (15.2): `FaultFs` bricht an jedem E/A-Index ab. Nach jedem Neustart gelten `INV-ERA-1…5`, und bei fortgesetztem Vorgang endet das Orakel mit 0 Treffern.
4. **Backup-Hinweis:** `checkpoint` ist standardmäßig `Copy`. `Hardlink` ist Opt-in mit Warnung, weil ein Hardlink die Datei nach dem Unlink am Leben hält und `INV-ERA-2` für die Kopie verletzt.

### 12.8 Grenzen (offen benannt)

Append-only Spaces (5.2) lassen `Erase` nur mit `Erase::admin()` zu; der Vorgang ist im Receipt vermerkt. Kalte Runs (10.6) unterliegen den Garantien des `ColdTier`-Ports.

Speicherinhalt, Auslagerungsdatei und Prozess-Core-Dateien werden nicht kontrolliert (Puffer werden best-effort überschrieben, aber ohne Garantie, `forbid(unsafe_code)` lässt keine volatile Garantie zu). Externe Snapshots/Backups, Dateisystem-Journal, Flash-Wear-Leveling siehe Tabelle 12.2.

---

## 13. Verschlüsselung und Schlüssel (Feature `encryption`)

- **Trait `Cipher`** in `env`/`codec`-Nähe: `seal(key, nonce, aad, plain) -> Vec<u8>`, `open(...)`. Standardbau ohne Feature kennt nur CRC32C.
- Mit Feature: ChaCha20-Poly1305 und BLAKE3 (Schlüsselableitung) aus geprüften Crates, `getrandom` für Zufall. **Eigene Kryptografie wird nicht geschrieben.**
- **Schlüsselhierarchie:** `KeyProvider`-Port liefert den 32-Byte-Hauptschlüssel (KEK). Pro Space wird ein zufälliger Daten-Schlüssel (DEK, `Entropy`) erzeugt, mit dem KEK umhüllt und in `KEYS` (atomar ersetzt) gespeichert. Run-Schlüssel = `KDF(DEK, "run" ‖ run_id)`, Redo-Schlüssel = `KDF(KEK, "redo" ‖ segment_id)`.
- **Nonce:** `run_id (8) ‖ block_no (4)` für Runs und `segment_id (8) ‖ frame_index (4)` für Redo. Eindeutig, weil IDs nie wiederverwendet werden (Zähler als Hochwassermarke, Abschnitt 6). Property-Test: keine doppelte `(Schlüssel, Nonce)`-Verwendung.
- **Passphrase-Ableitung** (bisher `KeyManager::try_new(passphrase, salt)` mit `SALT`-Datei) wandert aus dem Crate in die einbettende Anwendung; das Crate erwartet fertigen Schlüssel (A-07). Die `SALT`-Datei entfällt.
- **KEK-Rotation:** `KEYS` wird mit neuem KEK neu umhüllt und atomar ersetzt (Daten bleiben unberührt).
- **Dateien ohne Authentisierung gelten als nicht vertrauenswürdig:** im Modus `ENCRYPTED` ist jeder Block AEAD-geprüft, das Entfernen von Blöcken/Frames fällt durch Zusatzdaten und Catalog-Anker auf.

---

## 14. Fehlermodell und Grenzen

Ein einziges `Error`-Enum (kein `Storage(String)`), jeder Fehler hat `code()` (stabil), `retryable()`, strukturierte Felder und `hint()`: einen kurzen, handlungsleitenden Text, der für Menschen und Sprachmodelle gleichermaßen lesbar ist (z. B. „Space ‚trace‘ ist append-only; Schlüssel existiert bereits. Neuen Schlüssel wählen.“ oder „Stamp 880 nicht mehr verfügbar; älteste verfügbare Sicht: 1 012.“):

| Variante | Bedeutung | `retryable` |
|---|---|---|
| `Conflict{guard_index}` | Vorbedingung nicht erfüllt | nein (neu lesen) |
| `Busy{retry_after}` | Gegendruck | ja |
| `Poisoned{cause}` | `fsync`-/E/A-Fehler, Instanz vergiftet | nein (Neustart) |
| `Corrupt{file, offset, what}` | Prüfsumme/Struktur falsch | nein |
| `Erased{range, erased_at}` | Zugriff unterhalb eines Löschstamps | nein |
| `Revoked` | View widerrufen | nein |
| `HoldBlocked{holds}` / `Blocked{views}` | Löschung durch Halteregel blockiert | ja (nach Freigabe) |
| `Limit{what, max}` | Schlüssel/Wert/Change zu groß | nein |
| `Forbidden{space, op}` | Operation im Space nicht erlaubt (append-only) | nein |
| `Expired` | `ResumeToken` oder Stamp nicht mehr haltbar | nein (neu beginnen) |
| `HistoryPruned{oldest_available}` | Verlauf unterhalb der Untergrenze angefragt | nein |
| `Unsupported{feature}` | Pflicht-Feature unbekannt | nein |
| `Locked`, `Closed`, `NotFound`, `Exhausted`, `Io(kind)` | | je nach Fall |

**Limits (konfigurierbar, Fehlercode `Limit`):** Schlüssel ≤ 4 KiB, Wert ≤ 1 MiB (konfigurierbar bis 16 MiB; größere Daten gehören in eine Objektschicht, A-03), Change ≤ 32 MiB und ≤ 100 000 Ops, Redo-Frame ≤ 64 MiB.

---

## 15. Teststrategie und Qualitätstore

### 15.1 Netze

| Netz | Inhalt |
|---|---|
| **Modelltest** | Referenzmodell (`BTreeMap` mit Versionsliste je Schlüssel) gegen die Engine: zufällige Folgen aus `apply` (inkl. Guards), `view_at`, `scan`, Flush, Compaction, `Erase`, Neustart. Eigener Xorshift-PRNG für Reproduzierbarkeit, Seed wird bei Fehlschlag ausgegeben. |
| **Gesetze (proptest)** | Dekodieren ∘ Kodieren = Id für alle Formate; Run-Lesen = Eingabe; Compaction verändert keine Sicht ≥ `floor` (`INV-RET-1`); `retain` idempotent; Planer liefert nie überlappende Ausgaben auf Leveln ≥ 1. |
| **Absturzmatrix** | `FaultFs` bricht an **jedem** E/A-Index ab (15.2); danach `open`, Vergleich mit Modell-Präfix der quittierten Changes, `verify` grün. |
| **Bitflip** | Jedes einzelne Byte einer Run/Catalog/Redo-Datei wird gekippt: Ergebnis ist `Err(Corrupt)` oder identisches Ergebnis (nie falsche Daten). |
| **Fuzz** | Decoder für Redo-Frame, Run-Block/Footer, Catalog-Record, `FORMAT`, Schlüsselbereiche (bestehende 13 Fuzz-Ziele werden auf die neuen Decoder portiert). |
| **Nebenläufigkeit** | `loom` nur für die kleinen Synchronisationsstücke (Ticket-Warteschlange, Leiterwahl, Version-Austausch); Gesamtsystem über `Runtime::manual()` mit zufälliger Einplanung. |
| **Schichtregel** | `tests/layering.rs` (4.2). |
| **Lösch-Orakel** | siehe 12.7. |
| **Soak** | Langläufer mit Agenten-Lastprofil (viele kleine Changes, kurzlebige Views, seltene Erases), Platte-voll, Speicherdruck. |
| **Goldens** | Je Dateiart Referenzdateien; Byte-Stabilität des Formats über Plattformen. |

### 15.2 `FaultFs` (Stromausfall-Modell)

Kernstück der Qualität. `MemFs` hält je Datei zwei Zustände: *Seitencache* (alles Geschriebene) und *dauerhaft* (Stand der letzten `sync_data`/`sync_all`). Verzeichniseinträge (Anlegen, `rename`, `remove`) sind erst nach `sync_dir` dauerhaft. `FaultFs` kann:

- an E/A-Operation Nr. *n* einen **Absturz** auslösen: danach zählt nur der dauerhafte Zustand; nicht synchronisierte Schreibvorgänge werden **verloren, teilweise (zerrissen, in Sektorgrößen) oder in zufälliger Reihenfolge** übernommen,
- einen `fsync` **fehlschlagen** lassen (Poison-Pfad, `INV-LOG-5`),
- „Platte voll" (`write_all_at` → `StorageFull`) und Kurzschreibvorgänge simulieren,
- `rename` ohne `sync_dir` rückgängig machen (nach Absturz alter Name).

Die Matrix läuft für `apply`, Flush, Compaction, Catalog-Rollover, `Erase` (E1–E7), `drop_space`, Verschlüsselungsschlüsselwechsel und `checkpoint`.

### 15.3 Überführung des Bestandes

Die Datei `SPEC.md` (Stufe F0) katalogisiert die Invarianten des Bestands (INV-STORE-1, INV-FSYNC-1, INV-WAL-*, INV-LOCK-ORDER-1, …) und ordnet jedem ein Netz aus 15.1 zu. Aus den 160 Testdateien werden die **Szenarien** übernommen, nicht der Code. Verworfen werden Tests, die Legacy-Formate (`wal_legacy_*`, `hmac_legacy_*`, `format_version_compat`), `rollback_to_tx`, den adaptiven Planer oder `docid-128` prüfen.

### 15.4 Leistungsmodell (Hypothesen)

| ID | Hypothese | Messung |
|---|---|---|
| H-S1 | Punktlesen mit warmem Block-Cache `p99 < 50 µs`; kalt höchstens Anzahl Level × 1 Block-E/A | F3, F5 |
| H-S2 | Ein `apply` mit `Durability::Always` kostet ein `sync_data`; bei 64 parallelen Schreibern ≥ 20× Durchsatz gegenüber seriellem Fsync (Gruppenbildung) | F5 |
| H-S3 | Leveled mit Faktor 10: Schreibverstärkung ≤ 15, Platzverstärkung ≤ 1,3 (ohne Holds) | F6 |
| H-S4 | Wiederanlauf ≤ 1 s je 256 MiB Redo-Replay | F4/F5 |
| H-S5 | Öffnen ohne Redo und mit 1 000 Runs < 50 ms (Metadaten aus Catalog, Runs lazy) | F4 |
| H-S6 | `drop_space` mit 1 000 Runs < 100 ms; `Erase::range` kostet O(Bytes der berührten Runs) | F7 |
| H-S7 | Memtable mit `RwLock<BTreeMap>` und blockweisem Scan hält den Durchsatz des Bestandes (Schreiben, 8 Leser) bei ±20 % | F5 |
| H-S8 | `estimate` über einen Bereich mit 1 000 Index-Einträgen < 1 ms bei warmem Index-Cache; Fehler der oberen Schranke ≤ 2× auf Workloads ohne starke Überschattung | F5 |
| H-S9 | Präfix-Filter reduziert Block-Lesevorgänge bei Präfix-Scans ohne Treffer in ≥ 90 % der Runs um ≥ 80 % | F3 |

**Abbruchkriterium H-S7:** Verfehlt F5 die Hypothese um mehr als 2×, wird die Memtable durch eine Skip-Liste in einer **isolierten, gesondert geprüften** Einheit ersetzt (SD-10); das ist die einzige Stelle, an der `unsafe` überhaupt diskutiert wird, und nur nach Beschluss.

---

## 16. Phasenplan

Größen: S ≈ Tage, M ≈ 1–2 Wochen, L ≈ 3+ Wochen. Jede Phase endet mit grüner CI, aktualisierter `SPEC.md` und dem Fortschrittsprotokoll (Abschnitt 20).

| Phase | Inhalt | Abh. | Tor |
|---|---|---|---|
| **F0** (S) | Neues Crate-Skelett `contextra-storage` mit eigenem Workspace-Root; Alt-Crate nach `legacy/contextra-store/` verschieben (eingefroren, nicht gebaut); Lints und CI; `SPEC.md` mit Invariantenkatalog aus dem Bestand; `tests/layering.rs`. | – | CI grün auf leerem Skelett, `SPEC.md` reviewt |
| **F1** (M) | `env` (Ports, `RealFs`, `Clock`, `Entropy`, `Runtime` inkl. manuell, `Governor`), `testing` (`MemFs`, `FaultFs`, Orakel-Grundlage), `codec` (Varint, CRC32C, Bloom, Tag, KeyRange, `Bytes`), `Error`. | F0 | `FaultFs`-Selbsttests (nicht synchronisierte Daten gehen verloren, `rename` ohne `sync_dir` wird zurückgesetzt, zerrissene Schreibvorgänge); Decoder-Fuzz 10 min ohne Absturz |
| **F2** (M) | `redo`: Frame, Segment, Writer, Reader, Recover, Kette, Rotation. | F1 | Absturzmatrix nur für den Log: Ergebnis = Präfix der quittierten Frames; Torn-Tail geheilt; Mittendefekt erkannt |
| **F3** (M) | `run`: Builder, Reader, Blöcke, Filter (dynamische Größe), Bereichs-Löschmarker, `DirectoryBlock` (Extension-fähig), Index-Zähler, optionaler Präfix-Filter, `TableCache`, `BlockCache`. | F1 | Roundtrip-Property; Bitflip-Test; Lesen = Modell; H-S9 gemessen |
| **F4** (M) | `catalog`: `FORMAT`, `CURRENT`, Edit-Log, Snapshot, Rollover; `engine/recovery` (Schritte 1–9 ohne Erase); Verwaisten-Suche; `Version`-Aufbau. | F2, F3 | Absturzmatrix über jeden E/A-Schritt von Edit-Append und Rollover; Fuzz Catalog; H-S5 gemessen |
| **F5** (L) | `engine`: Memtable, Committer (Leiter/Gruppe), Guards, Views, Holds, Listener, Flush, Gegendruck, öffentliche API `apply/view/scan`, `scan_page`/`ResumeToken`, `estimate`, `history`, append-only Spaces. | F4 | Modelltest inkl. Neustart und Absturz (Seiten-Fortsetzung und Verlauf gegen Modell); `loom` für Ticket/Leiter/Version; H-S1, H-S2, H-S4, H-S7, H-S8 gemessen |
| **F6** (M) | Compaction (Planer, `retain`, Ausführung, Installation), Graveyard, Wartungs-Slot. | F5 | `INV-RET-1`-Property; Absturz in Compaction; H-S3 gemessen |
| **F7** (L) | `erase`: Protokoll E0–E7, Register `erased_ranges`, `drop_space`, Policies, Receipt, `verify_erasure`, Orakel. | F6 | Absturzmatrix über E1–E7; Orakel 0 Treffer; `INV-ERA-1…5`; H-S6 gemessen |
| **F8** (M) | Feature `encryption`: Cipher, `KEYS`, Runs/Redo AEAD, Schlüsselvernichtung, KEK-Rotation, kryptografisches Löschen. | F7 | Manipulationstests; Nonce-Eindeutigkeit (Property); nach Schlüsselvernichtung unlesbar trotz Kopie der Runs |
| **F9** (M) | Betrieb: `checkpoint` (Copy/Hardlink), `verify` (online/offline), `stats`/Metriken, Formatprüfung, Dokumentation, Benchmarks gegen Hypothesen, Entfernen von `legacy/` aus dem Arbeitsbaum, API-Freeze `0.1`. | F8 | Restore-CI (Sicherung → Zerstörung → Wiederherstellung → `verify` → Vergleich); Benchmark-Bericht; `cargo doc` ohne Warnungen |
| **F-I** (optional, M) | Importer für Alt-Verzeichnisse als **separates** Crate `contextra-storage-import` mit eingefrorenen Altdecodern. Nur falls produktive Altdaten existieren (A-01). | F5 | Importer-Äquivalenztest auf Fixture-Verzeichnis |
| **F10** (optional, M) | `Extension`-Trait einfrieren und mit einer Beispiel-Extension (Präfix-Zähler) testen; `locality_prefix`-Teilung; `ColdTier`-Port mit Datei-Referenzimplementierung. | F9 | Extension überlebt Flush, Compaction, Absturz und `Erase` konsistent (Orakel); Löschung kalter Runs im Absturztest |

Rückfalloptionen: Verfehlt F5 H-S7 → Memtable-Ersetzung (SD-10). Verfehlt F6 H-S3 deutlich → zweites Verfahren (Size-Tiered für Level 0) als eigene Entscheidung, nicht im Voraus.

---

## 17. Anforderungen der aufsetzenden Schichten

Das Crate ist bewusst generisch. Darüber liegen (nicht Teil dieses Plans) Schichten für inhaltsadressierte, versionierte Objekte mit Verzweigen und Zusammenführen, abgeleitete Daten (Zusammenfassungen, Vektoren, Volltext), Vertrauenskennzeichnung und budgetiertes Zusammenstellen von Kontext für Sprachmodelle. Aus deren Bedarf folgen diese Anforderungen an den Speicher, jeweils mit der erfüllenden Stelle:

| ID | Anforderung | Erfüllt durch |
|---|---|---|
| AR-1 | Atomare Mehrfach-Updates mit Compare-and-Swap (Zeiger umhängen, Idempotenzschlüssel setzen, Daten schreiben) | `Change` + `Guard` (8.2) |
| AR-2 | Eine konsistente Lesesicht über mehrere Bäume und Indizes | `View` über alle Spaces (8.3) |
| AR-3 | Große Scans ohne Materialisierung, mit Obergrenze nach Bytes | `Cursor`, `scan_page`, `ResumeToken` (5.2, 9) |
| AR-4 | Abgeleitete Indizes folgen Änderungen und holen nach Neustart auf | `subscribe`, `history` (5.2, 8.2) |
| AR-5 | Zeitreise und Aufbewahrung | `view_at`, `Hold` mit Ablauf (8.3) |
| AR-6 | Viele kleine Instanzen (eine je Mandant oder Agent) ohne Thread-Explosion | teilbare `Runtime`/`Governor`, `LOCK`, kein Thread je Instanz (4.3) |
| AR-7 | Löschpflicht trotz unveränderlicher Historie in den oberen Schichten, nachweisbar | `Erase`, `drop_space`, `Cryptographic`, Quittung, Orakel (12) |
| AR-8 | Formatentwicklung ohne Datenverlust | `FORMAT` mit Feature-Bits, Verzeichnisblock (7) |
| AR-9 | Planung von Kontextbudgets vor dem Lesen | `estimate` (5.2) |
| AR-10 | Unveränderliche Episoden- und Audit-Protokolle | append-only Space (5.2) |
| AR-11 | Zusätzliche Indizes je Datensegment, automatisch konsistent | `Extension` (10.4) |
| AR-12 | Kalter Speicher für alte, unveränderliche Segmente | `tier`, `ColdTier` (10.6) |
| AR-13 | Handlungsleitende Fehler für automatische Aufrufer | `Error::hint()`, `retryable()` (14) |

### 17.1 Bewusst nicht im Kern

| Thema | Begründung | Wo es hingehört |
|---|---|---|
| Inhaltsadressierte Objekte, Merkle-Bäume, Verzweigen/Zusammenführen | Der Kern liefert die nötigen Bausteine (atomarer CAS, Views, Erase); das Datenmodell ist Sache der oberen Schicht | obere Schicht |
| Vertrauens-/Taint-Labels | Semantik der Werte, nicht des Speichers; der Kern müsste Wertheader kennen | Wertkodierung der oberen Schicht |
| Token-native Speicherung, Tokenizer, Embeddings, ANN, Volltext | modellabhängig, Abhängigkeiten, wechselnde Algorithmen | obere Schicht über `Extension` |
| Umordnung nach Zugriffsmustern | Aufschaukelgefahr, Kenntnis der Semantik nötig | Re-Key in der oberen Schicht, Kern liefert Lokalitäts-Optionen (10.5) |
| Thread-per-Core, `io_uring`, `O_DIRECT` | zusätzliche Abhängigkeiten und `unsafe` | alternative `Fs`-Implementierung in einem separaten Crate |
| Bi-Temporalität | Stamp ist die Transaktionszeit; Gültigkeitszeit gehört in Schlüssel oder Wert | obere Schicht |
| Tensor-/KV-Cache-Blobs | übersteigen die Wertgrenze | Objektschicht |

---

## 18. Disposition des Bestandes

| Bereich (Zeilen inkl. Inline-Tests) | Schicksal | Neues Ziel |
|---|---|---|
| `wal/` (8 371) | **ersetzen** (Format, Replay, HMAC-Kette, Legacy, Flusher-Actor) | `redo/` (Idee: Kettenprüfsumme, Fail-stop, Open-Heal) |
| `lsm/` (7 638) | **ersetzen** (Engine, Commit-Pipeline, Group, Sperrhierarchie, Observer, Rollback, Recovery) | `engine/`, `api/` |
| `sstable/` (4 288) | **neu schreiben**, Blocklogik portieren (Präfixkompression, Binärsuche) | `run/` |
| `compaction/` (3 964) | **ersetzen**; `retention.rs` übernehmen; `adaptive/*`, `merge_operator`, `throttle` entfallen | `engine/compaction/` |
| `manifest/` (1 605) | **ersetzen** | `catalog/` |
| `memtable/` (1 004) | **ersetzen** (Sharding über Präfixe und reservierte Präfixe entfällt) | `engine/memtable.rs` |
| `compat/` (2 650) | **löschen** (Port-Adapter, Scanner, Purger, Lizenz). Der Purger wird durch `erase/` ersetzt | `erase/` |
| `tenant_codec` (246), `docid-128` | **löschen** (obere Schicht) | – |
| `system_pressure/` (454), `budget.rs` (91), `stats.rs` | **ersetzen** durch `Governor` und `stats()` | `env/governor.rs` |
| `io_seam/` (≈ 760), `traits.rs` (139), `fsync.rs` (128), `wal/fs.rs` (267), `util.rs` (155) | **zusammenführen** in einen Port `Fs` | `env/fs.rs`, `env/real_fs.rs` |
| `key_source.rs` (67) | **ersetzen** | `env/entropy.rs` |
| `RecordSealer`/`RecordAuthenticator` (`util.rs`) | **ersetzen** | `crypto/cipher.rs` |
| `lock_order.rs`, `fence.rs`, `group_commit.rs`, `guard.rs` | **entfallen** (Sperrenstruktur neu) | – |
| `tests/` (25 563) | **Szenarien übernehmen**, Code nicht | `tests/` neu, 15.3 |
| `fuzz/` (13 Ziele) | **portieren** auf neue Decoder | `fuzz/` |
| `benches/` | **neu**: Mikro-Benchmarks je Hypothese (kein Vergleichs-Benchmark gegen Fremdspeicher im Crate; `redb` fällt als Dev-Abhängigkeit weg) | `benches/` |
| Cargo-Features | 10 → 2 (`encryption`, `testing`) | – |

---

## 19. Entscheidungsregister

| ID | Entscheidung |
|---|---|
| SD-01 | Kein interner und kein externer Laufzeit-Abhängigkeitsbestand im Standardbau; `encryption` bringt geprüfte Krypto-Crates. |
| SD-02 | Synchroner Kern; Hintergrundarbeit auf teilbarer `Runtime`; `tokio` entfällt. |
| SD-03 | Ein Port `Fs` für alle E/A inkl. Verzeichnisse und Sperre; Stromausfall-Simulator ist Pflichttor. |
| SD-04 | Genau ein Format je Dateiart mit Feature-Bits; Altformate nur über separates Importer-Crate (falls nötig). |
| SD-05 | Der Catalog ist alleinige Wahrheit; Zustandsänderung = ein Edit; Dateien tragen numerische IDs, Run-Metadaten stehen im Catalog. |
| SD-06 | Ein Redo-Frame je `Change`; neues Segment je Prozessstart; Segmentrotation statt Truncate-in-place. |
| SD-07 | Spaces = getrennte LSM-Bäume mit gemeinsamem Redo-Log und gemeinsamer Stamp-Folge. |
| SD-08 | `Change` mit `Guard`s ersetzt Transaktionspuffer, SSI und Intent-Locks im Kern. |
| SD-09 | `rollback_to_tx` entfällt; Zeitreise über `view_at` und Holds. |
| SD-10 | Memtable = `RwLock<BTreeMap>` mit blockweisem Scan; Ersatz nur bei verfehlter H-S7. |
| SD-11 | Ein Compaction-Verfahren (Leveled), Planer als reine Funktion; keine adaptive Planung. |
| SD-12 | Kein Merge-Operator in Version 1 (Tag-`kind` reserviert); Read-Modify-Write über `Guard`. |
| SD-13 | Keine Kompression in Version 1 (`codec`-Byte und Trait reserviert). |
| SD-14 | `Erase` ist ein Kernprimitiv mit einem Durability-Punkt und Roll-forward; Löschung nie über Dateiendungen oder am Catalog vorbei. |
| SD-15 | Schlüsselvernichtung geschieht vor Dateilöschung; echte Unwiederbringlichkeit nur über `Cryptographic`. |
| SD-16 | Ein Wartungsjob je Instanz (Single-Flight), Priorität `Erase > Flush > Compaction > Cleanup`. |
| SD-17 | Fehler sind ein Enum mit `code`, `retryable` und strukturierten Feldern. |
| SD-18 | Eine Instanz, ein Prozess; Mehrprozess über einen Daemon der oberen Schicht. |
| SD-19 | Seitenweises Lesen mit `ResumeToken` statt Offsets; Token bindet an Stamp, Ablauf ist `Expired`. |
| SD-20 | `estimate` aus Index-Zählern (obere Schranke), ohne Datenblöcke. |
| SD-21 | `history` über Versionen mit Untergrenzenprüfung (`HistoryPruned`). |
| SD-22 | Append-only Spaces werden vom Kern erzwungen; `Erase` nur mit `admin()`. |
| SD-23 | `DirectoryBlock` statt fester Footer-Handles; optionaler Präfix-Filter; Extension-Blöcke im Format von Anfang an. |
| SD-24 | `tier` im Run-Metadatensatz reserviert, `ColdTier` als Port in F10. |
| SD-25 | Fehler tragen `hint()` als handlungsleitenden Text. |
| SD-26 | Lokalität nur über Schlüsselgestaltung und Run-Teilung an Präfixgrenzen; der Kern ordnet nie nach Zugriff um. |

---

## 20. Annahmen, offene Punkte, Fortschritt

| ID | Annahme / offener Punkt | Wirkung, falls falsch |
|---|---|---|
| A-01 | Es existieren **keine produktiven Altdaten** im Bestandsformat (das Repository enthält nur einen Initial-Stand). | Phase F-I wird Pflicht. |
| A-02 | Windows ist in Version 1 nur ohne Durability-Garantie (`Durability::Never`) unterstützt. Der Bestand liefert dort `Unsupported` für Verzeichnis-`fsync` (E20). | Eigener Windows-Port von `Fs::sync_dir` und `rename` nötig. |
| A-03 | Werte ≤ 1 MiB genügen; größere Daten liegen in einer Objektschicht. | Wertgrenze und Redo-Frame-Grenze anheben, Schlüssel-/Werttrennung prüfen. |
| A-04 | Obere Schichten nutzen `Guard` statt `TxBuffer`/SSI. Aufrufer des alten `contextra-mvcc`-Verfahrens (`checkpoint`, `agent`, `engine`) müssen migrieren. | Zusatzaufwand in den aufsetzenden Schichten; im Repository nicht sichtbar. |
| A-05 | Rust-Toolchain bietet `std::fs::File::try_lock` (der Bestand nutzt bereits `TryLockError`); MSRV wird in F0 festgelegt. Auf Apple-Plattformen verwendet `sync_all` laut Standardbibliothek vollständige Synchronisation (`F_FULLFSYNC`): **in F1 per Test zu belegen.** | Ersatz über eigenes `Fs`-Mitglied. |
| A-06 | Aufrufer von `rollback_to_tx` existieren nur in nicht sichtbaren Crates und sind durch Versionierung der oberen Schicht ersetzbar. | Übergangsfunktion nötig. |
| A-07 | Passphrase→Schlüssel (KDF) darf das Crate verlassen. | `KeyProvider` um Ableitung erweitern. |
| A-08 | Das Verzeichnis wird von `contextra-store` zu `contextra-storage` umbenannt; fremde Verweise sind im Repository nicht sichtbar und müssen im Gesamtworkspace nachgezogen werden. | Alias-Crate für eine Übergangszeit. |
| A-09 | Referenzhardware für die Hypothesen (NVMe, Kerne, RAM) wird in F1 festgelegt und dokumentiert. | Messwerte nicht vergleichbar. |
| A-10 | Die `Extension`-Schnittstelle ist erst nach einer realen Nutzung (z. B. Volltext oder Vektorsegment) stabil. Bis F10 gilt sie als unstabil. | Schnittstelle ändert sich nach Freigabe; Format bleibt unberührt. |
| A-11 | Ein kalter Speicher bietet zuverlässige Löschung oder wird nur mit Verschlüsselung betrieben. | `Physical` gilt für kalte Runs nicht. |

**Grenzen dieser Analyse:** Das Crate war nicht baubar, es wurde nichts ausgeführt. Aussagen zum Bestand beruhen auf Lesen der genannten Dateien; Stellen, die ich nicht gelesen habe (z. B. Details von `flusher.rs`, `replay.rs`, `reader/open.rs`), sind nicht bewertet. Alle Leistungsangaben sind Hypothesen.

**Fortschrittsprotokoll (Implementierer pflegt):**

| Phase | Status | PR/Commit | Abweichung | Datum |
|---|---|---|---|---|
| F0 | offen | | | |
| F1 | offen | | | |
| F2 | offen | | | |
| F3 | offen | | | |
| F4 | offen | | | |
| F5 | offen | | | |
| F6 | offen | | | |
| F7 | offen | | | |
| F8 | offen | | | |
| F9 | offen | | | |
| F-I | optional | | | |

---

## Beilage A: Beispielablauf Löschung eines Subjekts

```
st.erase(Erase::range(facts, prefix("subject/42/")).level(Physical).views(Wait(5s)).holds(Fail))
  E1  Frame EraseRange(facts, subject/42/) , Stamp S=1043, fsync          → ab jetzt für Views ≥ S leer
  E2  Flush aller Memtables bis 1043, Redo-Segmente ≤ 1043 → Graveyard
  E3  3 Runs von „facts" überlappen den Bereich (aus Catalog-Metadaten)
  E4  3 Runs ohne Einträge des Bereichs neu geschrieben (Stamp ≤ 1043), sync
  E5  Edit{ add 3, remove 3, erase_done(1043), erased_ranges_add } , sync     ← physischer Punkt
  E6  Graveyard wartet auf Views, remove_file ×3, sync_dir
  E7  ErasureReceipt{ id=1043, level=Physical, runs_rewritten=3, redo_segments_removed=2, … }
Absturz zwischen E1 und E5: Neustart findet EraseRange-Frame ohne erase_done → setzt bei E2 fort.
Absturz zwischen E5 und E6: Neustart löscht die verwaisten Runs (Verwaisten-Suche).
View mit Stamp 1000 liest in den Bereich: Err(Erased{ range, erased_at: 1043 }).
```

## Beilage B: Zustandsdiagramm einer Datei

```
(nicht vorhanden) ──create .building──► im Bau ──sync_all + sync_dir──► dauerhaft, UNREFERENZIERT
      ▲                                                                       │ Edit(add) dauerhaft
      │ Verwaisten-Suche                                                      ▼
      └──────────── Unlink + sync_dir ◄── Graveyard ◄── obsolet ◄──────── LEBENDIG (Catalog nennt sie)
                    (erst nach letzter Referenz)           ▲ Edit(remove) dauerhaft
```

Eine Datei wechselt nur in Pfeilrichtung, und jeder Wechsel zwischen „unreferenziert" und „lebendig" ist **ein** Catalog-Edit.

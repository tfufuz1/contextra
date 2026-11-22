# SPEC.md — `contextra-storage` (Fundament-Crate des Contextra-Speichers)

| | |
|---|---|
| Status | **Final, normativ, Revision R2** (Änderungsliste: §22.4).  Löst `REFACTORING_PLAN.md` (v1), `REFACTORING_PLAN_v2.md` (bricht in §16 ab) und `REVIEW_contextra-storage_vs_mnemos.md` als Spezifikation ab. Das *Wie und Wann* der Umsetzung steht in `IMPLEMENTATION_SPEC.md`. |
| Gegenstand | Neuer Kern `contextra-storage` samt `contextra-ports`. Ersetzt `crates/contextra-store` (Repository `tfufuz1/acta`, Stand `b001610`; Quelltext unverändert seit `47c9865`). |
| Sprache | Prosa und Rustdoc deutsch, Bezeichner englisch (P-7). |
| Geltung | Wer von dieser Datei abweicht, ändert zuerst diese Datei (§1.5). Jede Invariante (`INV-*`) wird in mindestens einem Test per ID zitiert (§18, §19). |
| Verifikation | Die Spezifikation wurde durch Lesen von Konzepten und Altcode erstellt. **Es wurde nichts kompiliert oder gemessen** (kein Rust-Toolchain in der Erstellungsumgebung). Alle Leistungsangaben sind Hypothesen (`H-S*`, §20). Auch Revision R2 wurde weder kompiliert noch gemessen; neu eingeführte Formatfelder sind vor **G-F** durch Goldens zu belegen (Anhang A.8). Die Referenzvektoren in Anhang A wurden mit einem unabhängigen Python-Referenzcodierer aus dieser Spezifikation berechnet (CRC32C und XXH64 gegen bekannte Prüfwerte validiert), nicht aus Rust-Code. |

---

## 1. Geltungsbereich und Konventionen

### 1.1 Zweck

`contextra-storage` ist ein **eingebetteter, synchroner, crash-sicherer Schlüssel-Wert-Kern** mit atomaren Mehr-Space-Änderungen, MVCC-Sichten, kontrollierter physischer Löschung und einer Wert-Trennung (Header im LSM, große Körper in Blob-Dateien). Er ist das Fundament für Speicher- und Retrieval-Schichten von Sprachmodell-Agenten (RAG-Speicherkerne), kennt aber **weder Tokenizer noch Embeddings, Hashes von Nutzdaten, Mandanten noch Branches**. Er liefert Primitive, aus denen diese Schichten korrekt gebaut werden (§7.6, Anhang C). Was er bewusst **nicht** baut und welche stabilen Flächen er aufsetzenden Schichten bereitstellt, regeln §2.3 und §7.8.

### 1.2 Normsprache

**MUSS / DARF NICHT** = zwingend, Verstoß ist ein Fehler. **SOLL / SOLL NICHT** = Regelfall, Abweichung braucht Begründung im Review. **KANN** = optional.

### 1.3 Dokumentenhierarchie

1. `SPEC.md` (diese Datei): normativ — Formate, Protokolle, Invarianten, API-Vertrag.
2. `IMPLEMENTATION_SPEC.md`: operativ — Phasen, Gates, Migration, Risiken. Widerspruch ⇒ SPEC gilt.
3. Alte Dokumente (`docs/archive/`): nur Herkunft. Sie sind **keine** Quelle für Anforderungen.

### 1.4 Kennungen

`Z-n` Ziel · `NZ-n` Nicht-Ziel · `P-n` Codierregel · `F-n` Fehlerklasse · `A-n` Angreifer (§5) und Annahme (§22; Kontext unterscheidet, Annahmen tragen immer `A-nn` zweistellig) · `SD-nn` Entscheidung (§21) · `INV-*` Invariante (§18) · `H-S*` Leistungshypothese (§20) · `O-n` offene Entscheidung (§22) · `G-*` Gate (Plan).

### 1.5 Änderungsprozess

Änderungen an Abschnitten 6–9, 13, 14 und 18 (Formate, Protokolle, Invarianten) erfordern zwei Reviewer, einen aktualisierten Referenzvektor (Anhang A) und, wenn ein Format betroffen ist, eine Prüfung gegen die Freeze-Liste (§9.8). Nach Gate **G-F** (Plan) ist jede Formatänderung ein Migrationswerkzeug (SD-04), nie ein Zweitpfad im Kern.

---

## 2. Ziele und Nicht-Ziele

### 2.1 Ziele

| ID | Ziel |
|---|---|
| Z-1 | Atomare Änderungen über mehrere Schlüssel **und Spaces** mit Vorbedingungen (Compare-and-Swap), crash-sicher. |
| Z-2 | Konsistente Sichten (`View`) und Zeitreise auf Stamp-Ebene mit Halteregeln (`Hold`) und persistenten Konsumenten-Cursorn (`Consumer`). |
| Z-3 | **Kontrollierte, atomare, nachweisbare physische Löschung** von Schlüsselbereichen und Spaces über **alle** Dateiarten (Runs, Blobs, Redo, Segmente). |
| Z-4 | Autark: keine internen Abhängigkeiten, **keine** externen Laufzeit-Abhängigkeiten (nur `std`). Kryptografie kommt über Ports (§4), nie als Kern-Abhängigkeit. |
| Z-5 | Einfach: wenige Konzepte, kleine Module, **ein** Format je Dateiart, **ein** Compaction-Verfahren. |
| Z-6 | Beweisbar robust: deterministische Tests, Fehlerinjektion an **jedem** E/A-Schritt, Modelltests, Linearisierbarkeitsprüfung. |
| Z-7 | Eignung für Agenten- und RAG-Daten: viele kleine atomare Schreibvorgänge, kurzlebige Sichten, strukturierte Fehler mit Handlungsanweisung, Idempotenz über `Guard::absent`, flüchtiges Arbeitsgedächtnis. |
| Z-8 | Betreibbar: `verify`, konsistente Sicherung, Metriken, explizite Formatversion mit Pflicht-/optionalen Feature-Bits. |
| Z-9 | **Budgetierte Ergebnislieferung** in Bytes und frei definierbaren Gewichten (typisch Token): vorab schätzen, seitenweise stabil fortsetzen, Filter auf dem Header ohne Körperzugriff. |
| Z-10 | Erweiterbar ohne Formatbruch: Index-Blöcke je Run, entkoppelte Segmente, Blob-Ebene, kalter Speicherort. |
| Z-11 | Robust unter dem Fehlermodell von §5, nicht nur unter Stromausfall (Bitfäule, verlorene/fehlgeleitete Schreibvorgänge, `EIO`, `ENOSPC`, Speicherdruck). |
| Z-12 | Ressourcenschonend: begrenzter, vorhersagbarer Speicher- und Deskriptorbedarf, kein Thread je Instanz, Leerlauf nahe null, faire Teilung zwischen Instanzen. |
| Z-13 | Feingranulare Sicherheit: drei Integritätsmodi, Verschlüsselung je Space, kryptografisches Löschen je Körper-Domäne, typisierte Zugriffsrechte, private Dateimodi, keine Datenlecks über Fehlertexte. |
| Z-14 | Beweisbare Nebenläufigkeit: wenige Sperren mit fester Rangordnung, alle Synchronisationsprotokolle klein genug für `loom`, schriftlicher Beweis der Kernaussagen (§16). |

### 2.2 Nicht-Ziele

| ID | Nicht-Ziel | Wohin |
|---|---|---|
| NZ-1 | Verteilter Konsens, Replikation | obere Schicht |
| NZ-2 | Mehrprozess-Zugriff auf eine Instanz (eine Instanz, ein Prozess, `LOCK`) | Daemon der oberen Schicht |
| NZ-3 | Transaktionspuffer, SSI, Intent-Locks, Mandanten- und Dokument-ID-Kodierung | obere Schicht (Aufbau aus `Guard`, §7.6) |
| NZ-4 | Merge-Operatoren (Tag-Kind reserviert, SD-12) | später, nur mit Formatentscheidung |
| NZ-5 | Kompatibilität mit dem Altformat im Kern (SD-04) | optional separates Importer-Crate (A-01) |
| NZ-6 | Windows mit voller Durability in Version 1 (A-02) | Port `Fs::sync_dir` für Windows |
| NZ-7 | `mmap` im Leseweg | Leseweg über `read_at` |
| NZ-8 | Inhaltsadressierung, Hashing von Nutzdaten, Dedup | obere Schicht |
| NZ-9 | Tokenizer, Embeddings, ANN, Volltext- und Graph-Algorithmen | obere Schicht, über `Extension`/`Segment` |
| NZ-10 | `O_DIRECT`, `io_uring`, Thread-per-Core im Kern | alternative `Fs`-Implementierung (SD-31) |
| NZ-11 | Kompression in Version 1 (`codec`-Byte reserviert, SD-13) | spätere Formatentscheidung |
| NZ-12 | Prolly-Tree/Merkle-DAG, Commit-DAG (Branches Stufe B) | obere Schicht; Stufe A (Overlay) siehe Anhang C |
| NZ-13 | Transaktionspuffer, Lesemengen-Verwaltung, SSI-Validierung, Sequenzprotokoll je Dokument, Snapshot-Register (Verschärfung von NZ-3) | obere Schicht, gebaut aus `Guard`/`View`/`Hold` (§7.8.1) |
| NZ-14 | Lösch-**Orchestrierung**: Saga über mehrere Schichten, anwendungsweite Lesesperre, Journal, Ledger, signierte Berichte, Restmessung mit Positivkontrolle | obere Schicht; der Kern liefert `Erase`, `ErasureReceipt`, `erase_status`, `scan_physical` (§7.8.4) |
| NZ-15 | Schlüsselverwaltung: Tresor, Slot-Tabellen, Passphrase-Ableitung, Beschaffung der KEK, gesperrter Speicher | `KeyProvider`, `SecretAlloc` (§4, §7.8.5) |
| NZ-16 | Kryptografische Primitive, SIMD, `unsafe`, `mlock`, `mmap`, Plattformhärtung (`F_FULLFSYNC`, ACL, Windows) | Ports und alternative `Fs`-Implementierungen mit `FsCaps` (§4) |
| NZ-17 | Benannte Checkpoints über mehrere Komponenten, Wiederherstellungs-Koordination, Checkpoint-Registry | obere Schicht; der Kern liefert dauerhafte `Hold`s und `checkpoint(dest)` (§7.8.2) |
| NZ-18 | Schlüsselcodierung für Mandanten, Kollektionen, Dokumente; Zugriffs-, Lizenz- und Richtlinienprüfung | obere Schicht (Anhang D) |
| NZ-19 | Vergessens-, Zerfalls- und Konsolidierungs**politik**, Bewertungsmodelle | obere Schicht; der Kern liefert den Mechanismus `CompactionFilter` (§12.2) |
| NZ-20 | `async`/`await`-Fassaden, Executor-Wahl | obere Schicht; der Kern liefert `submit` mit Fertigmeldung (§7.8.6) |

### 2.3 Verantwortungsgrenze: Kern, Ports, aufsetzende Schichten

Der Kern löst Speicherung, Dauerhaftigkeit, Nebenläufigkeit und Löschung **vollständig** und liefert für alles darüber **Mechanismus statt Politik**. Was außerhalb liegt, baut der Kern **nicht**, auch nicht vorsorglich: aufsetzende Schichten existieren teils bereits, teils entstehen sie später. Der Kern kennt sie nicht und richtet sich nicht nach ihnen; er stellt die stabilen Flächen aus §7.8 bereit (SD-51, SD-52).

| Bereich | Im Kern (MUSS) | Über Port hereingereicht | Außerhalb (Kern baut es nicht) | Fläche |
|---|---|---|---|---|
| Transaktionen | `Change`, `Guard`, `Stamp`, `View`, `Hold` | – | Puffer, Lesemengen, SSI, Sequenzprotokoll, Snapshot-Register | §7.6, §7.8.1 |
| Löschung | `Erase` (E0–E7), Marker, Blob-Sweep, `ErasureReceipt`, `verify_erasure`, `scan_physical` | `ColdTier::delete`, `KeyProvider` | Saga, Lesesperre der Anwendung, Journal, Ledger, Berichte, Restmessung | §14, §7.8.4 |
| Schlüssel | Hierarchie in `KEYS`, Nonce-Protokoll, Epochen, Domänen-Versiegelung | `Cipher`, `Mac`, `Kdf`, `KeyProvider`, `Entropy`, `SecretAlloc` | Tresor, Slots, Passphrase, KEK-Beschaffung | §15, §7.8.5 |
| Prüfsummen, Plattform | portables CRC32C und XXH64, `RealFs` (nur `std`) | beschleunigte `Checksum`, gehärtete `Fs` | SIMD, `unsafe`, `mlock`, `mmap` | §4 |
| Index-Strukturen | `Extension`-Blöcke, Segment-Lebenszyklus, -Rahmen und -Siegel, `Consumer`, `history` | Segment-Inhaltsformate | Tokenizer, Postings-Format, Vektor- und Graphstrukturen | §12.4, §12.5, §7.8.3 |
| Vergessen | `CompactionFilter`-Mechanismus | Entscheidungsfunktion | Zerfallsmodelle, Konsolidierung | §12.2 |
| Zeitreise | `View`, `Hold` (flüchtig, dauerhaft), `checkpoint(dest)` | – | benannte Checkpoints über Komponenten, Registry | §7.8.2 |
| Schlüsselcodierung | Bytes, Ordnung, Präfixfilter | – | Mandanten-/Kollektions-/Dokumentcodierung, Richtlinien | Anhang D |
| Asynchronität | synchrone API, `submit` | – | Executor, `async`-Fassade | §7.8.6 |

---

## 3. Architektur und Codierregeln

### 3.1 Workspace

```
acta/                              Repository-Wurzel
  Cargo.toml                       Workspace, resolver 3, MSRV in A-05 festgelegt
  rust-toolchain.toml  rustfmt.toml  clippy.toml  deny.toml
  SPEC.md  IMPLEMENTATION_SPEC.md
  crates/contextra-ports/          Traits und kleine Wertetypen, KEINE Logik, keine Abhängigkeiten
  crates/contextra-storage/        dieser Kern, nur std
  legacy/contextra-store/          eingefrorener Altstand (nicht im Workspace-Build, nur Referenz)
  docs/archive/                    alte Konzept- und Plandokumente
```

`contextra-ports` ist ein eigenes, winziges Crate, damit beliebige Implementierer (Prüfsummen, Kryptografie, Plattform-`Fs`, Indexbau, Telemetrie) `Checksum`, `Cipher`, `Mac`, `Kdf`, `Weigher`, `Filter`, `Extension` implementieren können, ohne vom Kern abzuhängen. Der Kern hängt an `ports`, nie umgekehrt.

### 3.2 Modulschnitt von `contextra-storage`

```
src/lib.rs            Re-Exports der öffentlichen API
src/sync.rs           Fassade über std::sync (unter cfg(loom): loom::sync) — einzige Stelle mit cfg(loom)
src/api/              storage.rs space.rs change.rs view.rs cursor.rs page.rs hold.rs consumer.rs
                      listener.rs config.rs error.rs submit.rs inspect.rs segment.rs
src/env/              env.rs real_fs.rs clock.rs entropy.rs runtime.rs governor.rs metrics.rs hooks.rs
src/codec/            varint.rs crc32c.rs xxh64.rs bloom.rs tag.rs keyrange.rs bytes.rs limits.rs
src/keys/             hierarchy.rs keyfile.rs domains.rs nonce.rs epoch.rs secret.rs      (Protokoll; Primitive über Ports)
src/redo/             frame.rs segment.rs seal.rs writer.rs reader.rs recover.rs prealloc.rs
src/run/              builder.rs reader.rs block.rs footer.rs filter.rs rangedel.rs extension.rs
                      table_cache.rs block_cache.rs
src/blob/             record.rs writer.rs reader.rs sweep.rs
src/catalog/          edit.rs log.rs state.rs current.rs format.rs reserve.rs
src/engine/           version.rs memtable.rs committer.rs flush.rs views.rs retention.rs registry.rs
                      compaction/{plan.rs,retain.rs,exec.rs,blobgc.rs}
                      maintenance.rs (Lanes, Claims) recovery.rs graveyard.rs
src/erase/            plan.rs exec.rs registry.rs verify.rs physical_scan.rs
src/testing/          (Feature `testing`) mem_fs.rs fault_fs.rs manual_runtime.rs hooks.rs
                      model.rs oracle.rs linearize.rs conformance.rs
```

### 3.3 Schichtregeln (per Test erzwungen)

```
 api ──► engine ──► { redo, run, blob, catalog } ──► { keys, codec, env }
 api ──► erase  ──► engine
 keys ──► { codec, env }                  redo/run/blob/catalog nutzen nur keys::*-Schnittstellen
 testing (Feature) ──► alles; nichts hängt von testing ab
```

`tests/layering.rs` liest alle `use crate::…`-Zeilen und schlägt fehl, wenn eine Kante gegen diese Regel verstößt. Zusätzlich prüft `cargo tree -e normal` in CI, dass `contextra-storage` ausschließlich `contextra-ports` als Abhängigkeit hat.

### 3.4 Features und Konfigurationsschalter

- Cargo-Feature **`testing`** (einziges): aktiviert `src/testing/`. Release-API enthält keine Testhaken (Test `no_test_hooks_in_release_api`).
- **`cfg(loom)`** und **`cfg(miri)`** sind zulässig, `cfg(loom)` nur in `src/sync.rs`.
- Es gibt **kein** Feature `encryption`: Verschlüsselung und Authentisierung laufen über Ports (§4, SD-36). Ein Space oder eine Instanz nutzt sie nur, wenn `FORMAT.integrity_mode` oder `domain_sealed` es verlangen und `Env` die Ports liefert.
- Alle Entwicklungs-Abhängigkeiten (`proptest`, `loom`, `arbitrary`) sind `dev-dependencies`; sie zählen nicht als Laufzeit-Abhängigkeiten.

### 3.5 Codierregeln

| ID | Regel |
|---|---|
| P-1 | Jede Datei auf der Platte hat **einen** Eigentümer-Mechanismus (§8) und wird nur über `Fs` berührt. |
| P-2 | Zustandsänderung = **ein** Catalog-Edit. Wer „zwei Dinge atomar“ braucht, packt sie in einen Edit. |
| P-3 | Reihenfolge ist Vertrag: *Datei dauerhaft (Datei + Verzeichnis) → Edit dauerhaft → erst dann sichtbar → erst dann alte Dateien löschen.* |
| P-4 | Dekodierung ist **total**: jede Byte-Folge ergibt `Ok` oder `Err(Corrupt)`, nie `panic`, nie unbegrenzte Allokation (Längen werden vor `Vec::with_capacity` gegen §6.7 geprüft). |
| P-5 | Keine globalen veränderlichen Variablen. Prüfpunkte für Tests laufen über `Env::hooks()` (Standard: leer). Keine `cfg(feature)` in Logik außer `testing`. |
| P-6 | Dateien ≤ 400 Zeilen, Funktionen typisch ≤ 60 Zeilen, Modultiefe ≤ 3, keine Kommentarbanner. Invarianten stehen in dieser Datei und werden in Tests per ID zitiert. |
| P-7 | Rustdoc deutsch, Bezeichner englisch. |
| P-8 | **Fehler sind Daten:** jede Fehlerklasse aus §5 hat einen definierten Zustand danach (heil, vergiftet, korrupt) und einen Test. |
| P-9 | **Kein Lock über E/A, außer dem Committer-Lock** (§16): Leser und Wartung warten nie auf Plattenzugriffe anderer. |
| P-10 | **Kein Lock über Aufrufer-Code:** `Weigher`, `Filter`, `CompactionFilter`, `Extension`, `Listener`, `KeyProvider`, `Cipher` laufen ohne gehaltene Sperre. Einzige Ausnahme: `Cipher`/`Mac`/`Checksum` im Committer beim Kodieren eines Frames (reine Funktionen der Ports, ohne Rückruf in den Kern). |
| P-11 | Jede Allokation im Heißpfad ist begründet: `apply` und `get` haben ein Allokationsbudget, das ein Test mit zählendem Allokator prüft (§17.5). |
| P-12 | **Geheimnisse verlassen den Prozess nie unverschlüsselt** und erscheinen weder in `Debug`, Logs, Metriken noch `hint()` (§15.7). |
| P-13 | `forbid(unsafe_code)`; `deny(unwrap_used, expect_used, panic, todo, unimplemented, unreachable, indexing_slicing)` in Produktionscode; `warn(missing_docs)` wird in CI zu Fehler. |
| P-14 | Zahlen und Offsets werden mit `checked_*`/`try_from` verarbeitet; `as`-Umwandlungen zwischen Breiten sind per Lint verboten. |

---

## 4. Ports (`contextra-ports`, semver-stabil)

```rust
pub enum Perm { Private /* 0600 Dateien, 0700 Verzeichnisse bzw. restriktive ACL */, Inherit }
pub enum OpenMode { Read, ReadWrite, CreateNew(Perm) }
pub enum LockMode { Exclusive, Shared }                      // nicht blockierend

pub trait Fs: Send + Sync + 'static {
    fn create_dir_all(&self, p: &Path, perm: Perm) -> io::Result<()>;
    fn open(&self, p: &Path, mode: OpenMode) -> io::Result<Box<dyn File>>;
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
    fn remove_file(&self, p: &Path) -> io::Result<()>;
    fn remove_dir(&self, p: &Path) -> io::Result<()>;
    fn list_dir(&self, p: &Path) -> io::Result<Vec<DirEntry>>;                // Name + Länge
    fn sync_dir(&self, p: &Path) -> io::Result<()>;
    fn lock(&self, p: &Path, mode: LockMode) -> io::Result<Box<dyn LockGuard>>;
    fn caps(&self) -> FsCaps;                                                  // zugesicherte Eigenschaften (SD-57)
    fn hard_link(&self, from: &Path, to: &Path) -> io::Result<()>;            // nur Checkpoint (Opt-in)
}
pub trait File: Send + Sync {
    fn read_at(&self, buf: &mut [u8], off: u64) -> io::Result<usize>;
    fn read_batch(&self, reqs: &mut [ReadReq]) -> io::Result<()> { /* Standard: Schleife über read_at */ }
    fn write_all_at(&self, buf: &[u8], off: u64) -> io::Result<()>;
    fn preallocate(&self, len: u64) -> io::Result<()> { /* Standard: Nullen in 1-MiB-Schritten */ }
    fn len(&self) -> io::Result<u64>;
    fn set_len(&self, len: u64) -> io::Result<()>;
    fn sync_data(&self) -> io::Result<()>;
    fn sync_all(&self) -> io::Result<()>;
}
pub struct FsCaps { pub durable_file_sync: bool, pub durable_dir_sync: bool, pub atomic_rename: bool,
                    pub private_perm: bool, pub no_follow_root: bool }
pub trait Clock:   Send + Sync { fn now_ms(&self) -> u64; fn monotonic_ns(&self) -> u64; }
pub trait Entropy: Send + Sync { fn fill(&self, dst: &mut [u8]) -> io::Result<()>; }
pub trait Checksum: Send + Sync { fn crc32c(&self, seed: u32, data: &[u8]) -> u32; }     // Standard: portabel, safe
pub trait Metrics: Send + Sync { fn counter(&self, n: &'static str, v: u64);
                                 fn gauge(&self, n: &'static str, v: i64);
                                 fn observe(&self, n: &'static str, v: f64); }
pub trait AccessSink: Send + Sync { fn sampled(&self, ev: AccessEvent); }   // (space, file, block_no, key_prefix, bytes); Standard: Null
pub trait Hooks: Send + Sync { fn point(&self, p: HookPoint) {} }           // Standard: leer
pub trait PressureSource: Send + Sync { fn level(&self) -> Pressure; }      // optional (cgroup/PSI in separater Impl.)

// Kryptografie: Primitive im Port, Protokoll im Kern (SD-36)
pub trait Cipher: Send + Sync {                                              // AEAD, 32-Byte-Schlüssel, 12-Byte-Nonce, 16-Byte-Tag
    fn seal(&self, key: &[u8; 32], nonce: &[u8; 12], aad: &[u8], plain: &[u8]) -> Vec<u8>;
    fn open(&self, key: &[u8; 32], nonce: &[u8; 12], aad: &[u8], sealed: &[u8]) -> io::Result<Vec<u8>>;
}
pub trait Mac: Send + Sync { fn mac16(&self, key: &[u8; 32], parts: &[&[u8]]) -> [u8; 16];
                             fn verify16(&self, key: &[u8; 32], parts: &[&[u8]], tag: &[u8; 16]) -> bool; } // konstante Zeit
pub trait Kdf: Send + Sync { fn derive(&self, key: &[u8; 32], context: &[&[u8]]) -> [u8; 32]; }
pub trait KeyProvider: Send + Sync {
    fn kek(&self) -> io::Result<Secret32>;                                   // Schlüsselverschlüsselungsschlüssel
    fn domain_key(&self, d: KeyDomain) -> io::Result<Option<Secret32>>;      // legt NIE an; None = dauerhaft vernichtet; nie angelegt ⇒ Err(NotFound)
    fn domain_key_create(&self, d: KeyDomain) -> io::Result<Option<Secret32>>; // legt bei Bedarf an; None = vernichtet (SD-55)
    fn make_durable(&self) -> io::Result<()>;                                // Barriere: alle bisher angelegten Schlüssel sind dauerhaft (INV-KEYP-1)
}
pub trait SecretAlloc: Send + Sync { fn alloc(&self, bytes: [u8; 32]) -> io::Result<Box<dyn SecretCell>>; }   // optional: geschützter Speicher (SD-56)
pub trait SecretCell: Send + Sync { fn expose(&self) -> &[u8; 32]; }                                        // Drop nullt
pub trait RollbackAnchor: Send + Sync {                                      // optional (F10, §15.5)
    fn record(&self, catalog_generation: u64, head_mac: [u8; 32]) -> io::Result<()>;
    fn latest(&self) -> io::Result<Option<(u64, [u8; 32])>>;
}
pub trait ColdTier: Send + Sync {                                            // Phase F10
    fn upload(&self, f: FileRef) -> io::Result<()>;
    fn read_range(&self, f: FileRef, off: u64, len: u32) -> io::Result<Vec<u8>>;
    fn delete(&self, f: FileRef) -> io::Result<()>;
}

// Aufrufer-Erweiterungen (Details §7.5, §12.2, §12.4)
pub trait Weigher: Send + Sync { fn version(&self) -> u32; fn weigh(&self, key: &[u8], header: &[u8], out: &mut [u32; 4]); }
pub trait Filter: Send + Sync { fn keep(&self, key: &[u8], header: &[u8]) -> bool; }
pub trait CompactionFilter: Send + Sync { fn decide(&self, key: &[u8], header: &[u8], stamp: Stamp, now_ms: u64) -> Decision; }
pub trait Extension: Send + Sync { fn id(&self) -> u16; fn build(..); fn merge(..); }
```

**Verträge:**

- `Fs`-Verzeichnisoperationen (Anlegen, `rename`, `remove`) sind erst nach `sync_dir` dauerhaft. `write_atomic(path, bytes)` (Temp `.building` → `sync_all` → `rename` → `sync_dir`) ist eine **Hilfsfunktion über `Fs`**, kein Trait-Mitglied, damit `FaultFs` sie mitprüft.
- `Perm::Private` MUSS unabhängig von der `umask` gelten (§15.6).
- `RealFs` nutzt ausschließlich `std` (`FileExt::read_at/write_at`, `File::lock/try_lock`).
- `Checksum::crc32c(seed, data)` setzt einen CRC-32C-Strom fort: Ergebnis = `!update(!seed, data)`; `seed = 0` liefert den üblichen CRC-32C (`"123456789"` → `0xE3069283`). Der Kern liefert eine **portable, sichere** Implementierung; beschleunigte Varianten (`unsafe`) kommen über denselben Port von außen und MÜSSEN die Goldens aus Anhang A.1 bestehen (SD-64).
- `Mac::mac16` und `Cipher` hängen **keine** Länge an; der Kern kodiert Längen selbst (§9.1).
- `Weigher::version()` ändert sich, wenn sich das Zählverfahren ändert (z. B. anderer Tokenizer); der Kern speichert sie je Run (§9.5).
- `Env` bündelt alle Ports: `fs, clock, entropy, runtime, governor, metrics, hooks, access_sink, checksum` immer (mit Standardimplementierungen); `cipher, mac, kdf, key_provider, secret_alloc, rollback_anchor, cold_tier, pressure` optional. `Env::real(root)` liefert die Standardbelegung.
- `Runtime` ist ein kleiner, **teilbarer** Thread-Pool (`spawn(Box<dyn FnOnce + Send>)`); `Runtime::manual()` führt Jobs nur auf `run_pending()` aus und macht jede Verschränkung in Tests deterministisch.
- **`FsCaps` und Fail-Closed (SD-57, `INV-FS-1`).** Jede `Fs`-Implementierung erklärt ihre Zusicherungen. `Config::fs_policy` ist `Strict` (Standard) oder `Relaxed`. Unter `Strict` öffnet der Kern nur, wenn `durable_file_sync`, `durable_dir_sync`, `atomic_rename` und `private_perm` wahr sind, sonst `Err(Unsupported{fs_caps})`. `Relaxed` ist für Tests und Wegwerf-Instanzen und setzt `Receipt.durable = false`. Eine schwächere Plattform wird nie stillschweigend akzeptiert. Neue `Fs`-Implementierungen bestehen `testing::conformance::fs_suite` (§19.1).
- **`KeyProvider`-Vertrag (SD-55, `INV-KEYP-1`).** (1) `domain_key_create` legt einen Schlüssel an, sobald er gebraucht wird. (2) `make_durable` kehrt erst zurück, wenn **alle** bis dahin angelegten Schlüssel dauerhaft sind; der Kern ruft es auf, **bevor** ein Frame mit einem so versiegelten Körper dauerhaft werden darf (§10.2 Schritt 1b). (3) `None` darf nur zurückkommen, wenn die Vernichtung dauerhaft und geprüft ist. (4) Aufrufe können blockieren und laufen ohne gehaltene Sperre (P-10). (5) Der Kern hält Domänenschlüssel nur im begrenzten Cache (§15.3).
- **`Secret32`/`SecretAlloc` (SD-56).** `Secret32` ist ein undurchsichtiges Handle (`expose() -> &[u8; 32]`), ohne `Debug`/`Clone`/`Serialize`. Standard: Heap, beim `Drop` `fill(0)` + `black_box` (best-effort). `Env.secret_alloc` ersetzt den Speicher durch geschützten Speicher der Anwendung, ohne dass der Kern `unsafe` braucht.
- `OpenMode::Direct` und `io_uring` sind **nicht** Teil des Ports in Version 1 (SD-31); der Port bleibt durch neue Enum-Varianten mit unverändertem Standardverhalten erweiterbar.

---

## 5. Fehler- und Bedrohungsmodell

### 5.1 Fehlerklassen

| ID | Fehlerklasse | Erwartetes Verhalten | Zustand danach | Prüfnetz (§19) |
|---|---|---|---|---|
| F-1 | Stromausfall / Prozess-Kill (`kill -9`) | Recovery (§13) | Präfix der quittierten Changes; jede Quittung mit `durable=true` bleibt erhalten | Absturzmatrix |
| F-2 | Zerrissener Schreibvorgang (Teil eines Sektors/Blocks) | Frame-/Block-/Record-Prüfsumme | Torn-Tail im **letzten ungesiegelten** Segment wird geheilt, sonst `Corrupt` | Absturzmatrix, Fuzz |
| F-3 | Nicht synchronisierte Schreibvorgänge in beliebiger Reihenfolge übernommen | Reihenfolge-Vertrag (P-3) | nie ein Edit vor seiner Datei | `FaultFs` (Umordnung) |
| F-4 | `fsync`-Fehler zur Laufzeit | **Poison**, kein Retry (`INV-LOG-5`) | Instanz tot; Neustart führt Recovery aus | `FaultFs` |
| F-5 | `ENOSPC` beim Schreiben | Abschneiden auf den letzten guten Stand (§10.5); Vorallokation verlagert den Fehler nach vorn | heil und wiederholbar (`Busy{StorageFull}`) oder Poison, wenn Abschneiden scheitert | `FaultFs` |
| F-6 | `EIO` beim **Lesen** | immer als Fehler melden, **nie als „leer“** | Operation scheitert; Instanz benutzbar; Datei als verdächtig markiert | `FaultFs` |
| F-7 | Bitfäule im Ruhezustand | Prüfsumme je Block/Frame/Record; in `Authenticated/Encrypted` kryptografisch | `Corrupt{file,offset}`; nie falsche Daten | Bitflip-Test |
| F-8 | Verlorener/fehlgeleiteter Schreibvorgang (Platte quittiert, schreibt nicht / falscher Ort) | Kettenprüfsumme im Redo, Block-AAD, **Redo-HWM** (`INV-LOG-8`), `verify` | erkannt, solange die Dateien intern inkonsistent werden; **nicht** erkennbar, wenn ein ganzer älterer, in sich stimmiger Stand zurückgespielt wird (→ F-10) | `FaultFs` (Lost/Misdirected), `verify` |
| F-9 | Speicherdruck / Deskriptor-Erschöpfung | `Governor`: verlangsamen, Flush erzwingen, Cache verkleinern; `TableCache` begrenzt offene Dateien | keine Allokationsabbrüche im Kern; `Busy{retry_after}` statt OOM | Ressourcentests (§17.5) |
| F-10 | Zurückspielen eines älteren Gesamtstands | nur mit **Rollback-Anker** (§15.5) erkennbar | ohne Anker nicht erkennbar, offen dokumentiert | Anker-Test |
| F-11 | Uhrsprung | `Clock` nur für Ablauf (Holds, TTL, Intervall-`fsync`); Ordnung läuft ausschließlich über Stamps | nie Ordnungsfehler | `Clock`-Tests |
| F-12 | Panic in Aufrufer-Code (`Weigher`, `Filter`, `CompactionFilter`, `Extension`, `Listener`) | `catch_unwind` an der Grenze in Wartung und Listener-Zustellung | Job schlägt fehl und wird gemeldet; Daten unverändert; Listener abgemeldet | Panic-Injektion |
| F-13 | `std`-Sperre vergiftet (Thread starb im kritischen Abschnitt) | `PoisonError` ⇒ `Error::Poisoned{cause: LockPoison}` | Instanz tot, Neustart | Panic-Injektion |
| F-14 | **Nonce-Wiederverwendung nach Absturz** (Datei mit ID *n* wurde angelegt, Zähler nicht dauerhaft) | ID-Reservierung im Catalog **vor** Nutzung (`INV-KEY-2`, §9.7) | kein `(Schlüssel, Nonce)`-Paar je doppelt | Property-Test + Absturzmatrix |
| F-15 | Downgrade des Integritätsmodus (Angreifer schreibt `FORMAT` auf `Plain` um) | `Config` nennt den erwarteten Modus; Abweichung ⇒ `Unsupported{integrity_mode}` (§9.2) | Öffnen verweigert | Manipulationstest |

### 5.2 Angreifermodell

| ID | Angreifer | Fähigkeit | Verteidigung | Grenze |
|---|---|---|---|---|
| A-1 | Anderer lokaler Benutzer / Mandant im selben Host | liest Dateien, wenn Rechte es zulassen | private Modi (`0600`/`0700`), eigenes Verzeichnis je Instanz | Root, Hypervisor |
| A-2 | Offline-Angreifer auf dem Datenträger (Diebstahl, Backup) | liest/verändert Dateien | `Encrypted`: Vertraulichkeit und Integrität | Rollback ganzer Stände (F-10) ohne Anker |
| A-3 | Offline-Angreifer ohne Schlüssel, der verändern will | Bits kippen, Frames umordnen, Blöcke vertauschen | `Authenticated`/`Encrypted`: MAC/AEAD mit AAD-Bindung; erwarteter Modus im `Config` | Löschen ganzer Dateien/Zurückspielen (F-10) |
| A-4 | Fehlerhafter oder bösartiger Aufrufer (obere Schicht, Agent) | ruft die API unzulässig auf, fälscht `ResumeToken` | typisierte Handles (§15.4), Limits, `Forbidden`, MAC-gesicherte Tokens | Vertrauen in die einbettende Anwendung |
| A-5 | Prozess mit Speicherzugriff (Core-Dump, Swap) | liest Schlüssel/Klartext | nicht kontrollierbar (kein `unsafe`, kein `mlock`); Bereinigung best-effort | §15.7 |

**Konsequenz:** Ohne `Authenticated`/`Encrypted` schützt CRC32C **nur** gegen Zufallsfehler, nicht gegen Angreifer. Das steht in der Dokumentation und im `FORMAT`-Feld `integrity_mode`.
---

## 6. Datenmodell

### 6.1 Vokabular

Im Crate kommen weder Produkt- noch Fremdsystemnamen vor.

| Begriff | Bedeutung | Bisher (Altstand) |
|---|---|---|
| **Storage** | Handle einer geöffneten Instanz | `LsmStorage` |
| **Space** | Benannter, physisch getrennter LSM-Baum (`SpaceId(u16)`) | Präfix-Reservierung |
| **Change** | Atomare Menge von Schreibvorgängen plus `Guard`s | Transaktion + `TxBuffer` |
| **Guard** | Vorbedingung innerhalb eines `Change` | `put_if_absent`, SSI-ReadSet |
| **Stamp** | Monotone Zählung angenommener Changes (`u64` < 2^56) | `seq_no` + `TOMBSTONE_BIT` |
| **Receipt** | Quittung eines `Change` (`stamp`, `durable`) | – |
| **View** | Unveränderliche Sicht auf einen Stamp | Snapshot |
| **Cursor** | Streaming-Iterator über einen Schlüsselbereich | `scan` (materialisiert) |
| **Hold** | Befristete Halteregel gegen das Verwerfen alter Versionen, mit Eigentümer | `pin_checkpoint` |
| **Consumer** | Benannter, persistenter Fortschrittszeiger eines abgeleiteten Index; zählt wie ein Hold, mit Maximalrückstand | – |
| **Redo-Log** | Segmentiertes, gesiegeltes Wiederholungsprotokoll | WAL |
| **Run** | Unveränderliche sortierte Datei mit Headern und Verweisen | SSTable |
| **Blob** | Unveränderliche Datei mit großen Wertkörpern, vom Catalog verwaltet | – |
| **BlobRef** | Verweis `(blob_no, offset, len, crc, domain)` im Run | – |
| **Header** | Die ersten `header_len` Bytes eines Wertes; ohne Körper lesbar | – |
| **Body** | Der Rest des Wertes | – |
| **Weight** | Frei definierbare Zählgröße je Eintrag (k ≤ 4, typisch Token), in Blocksummen geführt | – |
| **KeyDomain** | Verschlüsselungsdomäne eines Körpers (`u32`, z. B. ein Subjekt) | – |
| **Segment** | Entkoppelte, vom Catalog verwaltete Datei einer oberen Schicht (z. B. Vektorgraph) | – |
| **FileKind** | `Run`, `Blob`, `Segment(kind u16)`; Redo-Segmente sind **kein** `FileKind` (§8.2) | Pfadendung |
| **Tier** | `Local` oder `Cold`; Attribut jeder `FileMeta` | – |
| **Claim** | Reservierung von Dateien durch einen laufenden Wartungsjob | – |
| **Catalog** | Einzige Wahrheit über lebendige Dateien und Zustand | Manifest |
| **Erase / ErasureReceipt** | Löschauftrag mit physischer Garantie / Nachweis | Purger |
| **Env** | Bündel der Ports (§4) | `Vfs`, verstreute Umgebungstypen |
| **Graveyard** | Warteschlange für zu löschende, noch referenzierte Dateien | – |
| **Page / ResumeToken** | Budgetierte Ergebnisseite und Fortsetzungsmarke | `LIMIT/OFFSET` |

### 6.2 Schlüssel, Tags, Reihenfolge, Stamp

- **Nutzerschlüssel:** beliebige Bytes, 1 ≤ Länge ≤ `max_key_bytes`; Ordnung **bytelexikografisch aufsteigend**. Der Space steckt nicht im Schlüssel, sondern in der Dateizuordnung (SD-07).
- **Interner Schlüssel** = `user_key ‖ tag` mit `tag: u64 = (stamp << 8) | kind`. Sortierung: **Nutzerschlüssel aufsteigend, Tag absteigend** (neueste Version zuerst).
- `kind`: `Put = 1`, `Delete = 2`, `RangeDelete = 3`, `EraseRange = 4`, `PutRef = 5`, `Merge = 6` (**reserviert**, nicht erzeugbar in Version 1; ein Decoder, der `6` sieht, meldet `Unsupported`). Der Löschmarker ist ein eigenes Typ-Byte, nicht Bit 63; alle 142 Maskierungsstellen des Altstands entfallen.
- `PutRef` kommt **nur in Runs** vor (Körper ausgelagert, §9.5/§9.6). Redo und Memtable kennen nur `Put`.
- **Stamp:** `u64`, streng monoton über die **angenommenen** Changes, lückenlos (`INV-STAMP-1`), erster Stamp ist 1, Wert 0 bedeutet „leerer Zustand“. Stamps sind auf `< 2^56` begrenzt; Überlauf ⇒ `Error::Exhausted`, nie Umbruch (`INV-STAMP-2`).
- **Bereichsmarker** (`RangeDelete`, `EraseRange`): `(start, end, tag)` mit `end` exklusiv; fehlendes `end` bedeutet „bis zum Ende des Space“ (Flag `UNBOUNDED_END`). In Runs liegen sie im `RangeDelBlock`, nicht im DataBlock. Ein Marker mit Stamp `S` verdeckt für jede Sicht ≥ `S` alle Versionen mit Stamp < `S` im Bereich.

### 6.3 Werte: Header und Körper

Ein Wert besteht aus `header ‖ body`. `header_len` ist je Space fest (0…255). Der Kern liest den Header ohne Körper (§11), die Bedeutung des Headers kennt er nicht. Wer `Value::new(header, body)` übergibt, MUSS `header.len() == header_len` einhalten, sonst `Error::Limit{HeaderLen}`.

Hat der Space `separate_body(t)` und gilt `body.len() ≥ t` (bzw. immer bei `domain_sealed`, sofern `body` nicht leer ist), wird der Körper **erst beim Flush** in eine Blob-Datei ausgelagert; der Run enthält dann `kind = PutRef` mit `value = header ‖ BlobRef` (§9.5). Bis dahin liegt der Körper im Redo-Frame und in der Memtable (SD-29).

### 6.4 `SpaceOptions` (unveränderlich nach dem Anlegen, Teil von `spaces_add`)

| Option | Wirkung | Gültigkeitsregel |
|---|---|---|
| `header_len(n)` | Erste `n` Wertbytes sind der Header (≤ 255 B). | – |
| `weights(k)`, `k ≤ 4` | Index-Blöcke führen `k` `u32`-Summen; Gewicht 0 ist immer „Bytes“ (Header + Körper). Ein registrierter `Weigher` liefert die übrigen. | `k > 1` verlangt beim Öffnen einen registrierten `Weigher`, sonst `Error::Config{WeigherMissing}`. `k = 0` ist unzulässig (mindestens Bytes). |
| `separate_body(t)` | Körper ab `t` Bytes liegen in Blob-Dateien (Vorschlag `t = 4096`). | `512 ≤ t ≤ max_value_bytes`. |
| `domain_sealed()` | **Jeder** nichtleere Körper geht in Blob-Dateien und ist mit dem Schlüssel seiner `KeyDomain` versiegelt (§15.3). | verlangt `Env.cipher` und `Env.key_provider`, unabhängig vom Integritätsmodus; jedes `put` MUSS eine Domäne tragen (sonst `Forbidden`). |
| `prefix_filter(n)` | Bloom-Filter über die ersten `n` Schlüsselbytes je Run. | `1 ≤ n ≤ 64`. **Standard für Haupt-Spaces**, weil Präfix-Scans Punktlesen im Verhältnis ≈ 5 : 1 überwiegen (Anhang B.3). |
| `locality_prefix(n)` | Compaction teilt Ausgaben bevorzugt an Präfixgrenzen (±25 % der Zielgröße). | `1 ≤ n ≤ 64` |
| `append_only()` | `Put` nur auf Schlüssel ohne lebende Version (sonst `Forbidden{PutExisting}`); `Delete`/`RangeDelete` verboten; `Erase` nur mit `Erase::admin()`. | – |
| `forgetting()` | Erlaubt einen `CompactionFilter` (§12.2); Vergessen ist eine sichtbare Zustandsänderung (SD-48). | unvereinbar mit `append_only()` (Flag `16`, E-1). |
| `ephemeral()` | Nur Memtable: kein Redo, keine Runs, verfällt beim Schließen. Ein `Change` darf **nicht** dauerhafte und flüchtige Spaces mischen (`Forbidden{MixedDurability}`). | unvereinbar mit `separate_body`, `domain_sealed`, `prefix_filter`, `locality_prefix`, `weights(k>1)`; begrenzt durch `Config::ephemeral_max_bytes` (darüber `Busy`). |
| `merge()` (**reserviert**, SD-60) | In Version 1 **nicht erzeugbar**. Flag `32` in `spaces_add`, Pflicht-Bit `MERGE` und `kind = 6` sind reserviert, damit ein späterer Merge-Operator (assoziative, kommutative Deltas für Zähler und Kantengewichte) ohne neue Formatversion einführbar ist. | Ein Decoder, der Flag `32` sieht, meldet `Unsupported`. |

### 6.5 `Change`, `Guard`, `Receipt`

- Ein `Change` enthält Operationen (`put`, `delete`, `delete_range`) über beliebige Spaces und beliebig viele `Guard`s. Doppelte Schlüssel innerhalb eines `Change` werden beim Aufbau zusammengelegt (letzter gewinnt). Ein `Change` erhält **einen** Stamp.
- `Guard`-Varianten: `absent(space, key)`, `equals(space, key, expected)`, `unchanged_since(space, key, stamp)`, `range_unchanged_since(space, range, stamp)`. Guards werden unter der Schreibsperre gegen den **neuesten** Stand einschließlich früherer Changes derselben Gruppe geprüft (§10.2). `range_unchanged_since` arbeitet über `max_stamp` je Run/Block und ist **O(überlappende Blöcke)**, kein Vollscan.
- `Change::durability(Always | Interval(d) | Never)` überschreibt `Config::durability` für diesen Change.
- `Receipt { stamp: Stamp, durable: bool }`. `durable = true` bedeutet: Der Frame ist per `sync_data` gesichert.

### 6.6 Konfiguration (Standardwerte)

| Parameter | Standard | Anmerkung |
|---|---|---|
| `integrity` | **muss angegeben werden** (`Plain`, `Authenticated`, `Encrypted`) | bei bestehender Instanz muss er dem `FORMAT` entsprechen (F-15) |
| `durability` | `Always` | |
| `memtable_bytes` (je Space) | 16 MiB | zählt Schlüssel, Header **und** Körper |
| `max_frozen` | 4 | |
| `l0_compaction_trigger` / `l0_slowdown` / `l0_stop` | 4 / 8 / 12 | §10.4 |
| `level_multiplier` / `level1_target_bytes` / `max_levels` | 10 / 256 MiB / 7 | |
| `target_run_bytes` | 64 MiB | |
| `block_bytes` / `restart_interval` | 4 KiB / 16 | |
| `bloom_bits_per_key` | 10 | |
| `redo_segment_bytes` | 64 MiB | |
| `max_redo_bytes` | 512 MiB | erzwingt Flush hinterherhinkender Spaces |
| `blob_target_bytes` | 64 MiB | |
| `blob_gc_live_ratio` / `blob_gc_max_age` | 0,4 / 7 Tage | §12.3 |
| `blob_min_bytes` | 16 MiB (= `blob_target_bytes / 4`) | Konsolidierung kleiner Blobs (SD-43) |
| `id_reserve_block` | 256 | ID-Reservierung (§9.7, `INV-KEY-2`) |
| `max_open_files` | 512 | `TableCache` |
| `stall_slice` / `stall_timeout` | 1 ms / 100 ms | §10.4 |
| `ephemeral_max_bytes` | 64 MiB | |
| `listener_queue` | 10 000 Ereignisse | §7.4 |
| `consumer_check_interval` | 30 s | |
| `fs_policy` | `Strict` | §4, `INV-FS-1` |
| `listener_deadline` | 100 ms | §7.4 |
| `domain_key_cache` | 4 096 Einträge, TTL 5 min | §15.3 |
| `erase_receipt_keep` | 10 000 | §9.7 |

### 6.7 Limits (Fehlercode `Limit`, konfigurierbar)

Schlüssel ≤ 4 KiB (`max_key_bytes`), Header ≤ 255 B, Wert ≤ 1 MiB (`max_value_bytes`, konfigurierbar bis 16 MiB; größere Daten gehören in eine Objektschicht, A-03), Change ≤ 32 MiB (`= redo_segment_bytes / 2`) und ≤ 100 000 Ops, Redo-Frame ≤ 64 MiB, `weights` ≤ 4, Spaces ≤ 64 (Empfehlung ≤ 12, §7.6), Consumer ≤ 64, Holds je Instanz ≤ 10 000, Spacename ≤ 64 Byte aus `[a-z0-9._-]`, Segment-Dateien je `kind` ≤ 4 096, Segment-Block ≤ 16 MiB, `reference` genau 16 B, dauerhafte Holds ≤ 1 000. Alle Längenfelder in Dateien werden **vor** jeder Allokation gegen diese Grenzen und gegen die Restlänge der Datei geprüft (`INV-FMT-2`).

---

## 7. API-Vertrag

### 7.1 Überblick (Skizze, normativ in Semantik, nicht in Syntax)

```rust
let st = Storage::open(Config::new(Integrity::Plain), Env::real("/var/lib/app"))?;
let admin: AdminCap = st.admin_cap().ok_or(...)?;     // genau einmal je Storage abrufbar
// Typisierte Handles: Rd < Ap < Wr < Adm. Rechte lassen sich nur abschwächen.
let facts: Space<Wr> = st.space("facts", SpaceOptions::default()
                          .header_len(32).weights(2).separate_body(4096).prefix_filter(8))
                          .weigher(my_weigher).open()?;
let trace: Space<Ap> = st.space_append_only("trace", SpaceOptions::default().prefix_filter(8))?;
let facts_adm: Space<Adm> = st.space_adm(&admin, "facts")?;
let facts_ro: Space<Rd> = facts.read_only();

let r: Receipt = st.apply(
    Change::new()
        .durability(Durability::Always)
        .guard(Guard::absent(&idem, key_k))                          // Idempotenz
        .guard(Guard::equals(&refs, b"main", old))                   // Compare-and-Swap
        .guard(Guard::range_unchanged_since(&facts, KeyRange::prefix(b"a/"), s0))
        .put(&facts, k, Value::new(header, body).domain(KeyDomain(42)))
        .delete(&facts, k2)
        .put(&refs, b"main", new))?;   // Err(Conflict{guard_index}) | Busy | Poisoned | Forbidden | Limit …

let view = st.view();                                     // aktueller Stamp
let old  = st.view_at(stamp)?;                            // nur ≥ Untergrenze (Views, Holds, Consumer)
let e = view.get(&facts_ro, k)?;                          // Option<Entry>; e.header() ohne Körper-E/A, e.body()? lädt
let es = view.get_many(&facts_ro, &keys)?;                // gebündelt über read_batch
let bs = view.bodies(&entries)?;                          // Körper mehrerer Treffer, nach (blob_no, offset) sortiert und zusammengefasst
for kv in view.scan(&facts_ro, KeyRange::prefix(b"a/"), Dir::Fwd)? { let (k, e) = kv?; }
let page = view.scan_page(&facts_ro, KeyRange::prefix(b"a/"), Dir::Fwd,
              PageLimit::weight(1, 6_000).bytes(256_000).entries(200),
              Some(&my_filter), resume)?;                 // Page{ items, next: Option<ResumeToken>, oversized }
let est  = view.estimate(&facts_ro, KeyRange::prefix(b"a/"))?;   // Estimate{ entries, bytes, weights[k], exact_weights }
let hist = view.history(&facts_ro, KeyRange::prefix(b"a/"), StampRange::after(s0))?;

let hold = st.hold(stamp, HoldOwner::new("branch/a1"), Some(Duration::from_secs(600)))?;
let cons = st.consumer("vec-index", ConsumerOptions::max_lag_bytes(256 << 20))?;   // persistent
cons.advance(applied_stamp)?;
st.subscribe(listener);
let q = st.erase(Erase::range(&facts_adm, KeyRange::prefix(b"subject/42/")).level(Physical)
                      .views(ViewPolicy::wait(5.secs())).holds(HoldPolicy::Fail))?;
st.erase_batch(vec![e1, e2, e3])?;                        // eine Flush-Barriere für mehrere Aufträge
st.erase(Erase::domain(KeyDomain(42)))?;                  // nur nach Schlüsselvernichtung (§14.3)
st.drop_space(facts_adm)?;
st.checkpoint(dest_dir, CheckpointMode::Copy)?;
st.verify(VerifyScope::Online)?;
st.rotate_kek()?;                                         // O(1): schreibt nur KEYS neu (§15.2)
st.hibernate()?; st.sync()?; st.close()?;
```

### 7.2 Eigenschaften

- **Kein Teilzustand.** `apply` ist alles-oder-nichts.
- **Rechte als Typen (SD-34).** `Change::put` akzeptiert nur `Space<Wr>` (bzw. `Space<Ap>` für append-only Spaces), `Erase`/`drop_space` nur `Space<Adm>`; `View::get/scan` jedes Handle. Eine Verletzung ist ein **Übersetzungsfehler**. Dynamische, delegierbare Berechtigungen (Macaroons, TTL) sind Sache der oberen Schicht, die abgeschwächte Handles ausgibt. `open_readonly` liefert nur `Rd`-Handles.
- **Header-zuerst-Lesen.** `Entry::header()` kostet keinen Körperzugriff; `Entry::body()` lädt den Körper und prüft Prüfsumme/AEAD. Filter, Gewichte und Seitenbildung arbeiten ausschließlich auf Headern.
- **`estimate`** summiert Einträge, Bytes und Gewichte aus den Index-Blöcken der Runs (Blockgranularität an den Rändern; zählt überschattete Versionen mit, ist also eine **obere Schranke**) und schätzt den Memtable-Anteil durch Abtasten (≤ 1 024 Einträge, `Weigher` ohne Sperre). Kosten O(Index), keine Datenblöcke. Weicht die `Weigher`-Version eines Runs ab, meldet `estimate` `exact_weights = false` und rechnet betroffene Runs über die Header nach.
- **`history`** liefert alle Versionen (Puts, Deletes) mit Stamp in `(von, bis]`, gefiltert über `max_stamp` der Runs/Blöcke. Voraussetzung: Untergrenze ≤ `von`, sonst `Err(HistoryPruned{oldest_available})`.
- **Seiten statt Offsets.** `scan_page` begrenzt nach Gewicht, Bytes und/oder Einträgen. Die Seite endet **vor** dem ersten Eintrag, der ein Limit überschreiten würde. **Fortschrittsgarantie:** Ist schon der erste Eintrag größer als ein Limit, wird er allein geliefert und `Page.oversized = true` gesetzt (kein Endlosschleifen-Risiko für Aufrufer). Die `ResumeToken` enthält `(space, last_key, stamp, range_hash, dir, chk)`; `chk` ist im Modus `Plain` CRC32C, sonst `Mac16` über den Token (A-4: Token sind nicht fälschbar). Bindung an **denselben Stamp** und **denselben Bereich**: ist der Stamp nicht mehr haltbar ⇒ `Err(Expired)`; passt der Bereich nicht ⇒ `Err(InvalidToken)`; nie eine stillschweigend andere Sicht.
- **Konsumenten** sind persistent, zählen in die Untergrenze und werden bei Überschreiten von `max_lag_bytes` **verworfen** (nächster Zugriff `HistoryPruned`; die obere Schicht baut den Index neu). Der Rückstand wird als obere Schranke aus Run-Metadaten (Stamp-Bereiche und Bytes) und Redo-Länge geschätzt und von der Wartung alle `consumer_check_interval` geprüft; ein toter Indexbauer kann den Speicher damit nicht dauerhaft aufblähen.
- **Strukturierte Fehler** mit handlungsleitendem, **datenfreiem** Text (§15.7, §7.7).
- `Bytes` ist ein eigener, schlanker Typ (`Arc<[u8]>`-Ausschnitt mit `Deref<Target=[u8]>`), damit Blockdaten ohne Kopie geliefert werden.

### 7.3 Durability

`Durability::Always` quittiert nach `sync_data`; `Interval(d)` quittiert sofort mit `durable = false`, die `urgent`-Lane synchronisiert spätestens nach `d`; `Never` ist für Tests und Wegwerf-Instanzen. Redo ist ein gemeinsames Log: **ein `Always` in der Gruppe macht alle früheren Frames derselben Gruppe mit dauerhaft** (Präfix-Eigenschaft). `Storage::sync()` erzwingt. Die Quittung ist **nie** eine Aussage über andere Changes als den eigenen und seine Vorgänger im Log.

### 7.4 Listener

`subscribe(listener)` liefert `ChangeEvent { stamp, ops: [OpSummary{space, kind, key, header}] }` **nach** Durabilität und Sichtbarkeit, **in Stamp-Reihenfolge**, aus einem Dispatcher-Job ohne gehaltene Sperre (P-10). Körper werden nicht mitgeliefert (Abruf über `view_at(stamp)`). Die Warteschlange ist auf `listener_queue` Ereignisse begrenzt und jede Zustellung auf `listener_deadline`; läuft ein Listener über oder überschreitet er die Frist, wird er **abgemeldet** (nie blockiert der Dispatcher den Commit-Pfad; SD-62) und erhält beim nächsten Aufruf `ListenerLagged{missed_from}`. Er muss über `history` und seinen `Consumer` aufholen. Ereignisse sind höchstens-einmal nach einem Absturz; die Zustellgarantie **mindestens-einmal** ergibt sich nur über `Consumer` + `history`.

### 7.5 Aufrufer-Erweiterungen

- **`Weigher`:** rein, ohne Zustand außer `version()`; wird beim Run-Bau und bei `estimate`/`scan_page` aufgerufen, **nie** im Committer.
- **`Filter`:** wird in `scan_page` **vor** dem Laden eines Körpers ausgewertet (Trust-Stufe, Gültigkeitszeit, Typ). Gefilterte Einträge zählen nicht gegen das Limit.
- **`CompactionFilter`:** siehe §12.2. **`Extension`:** siehe §12.4.
- Alle laufen ohne gehaltene Sperre und hinter `catch_unwind` in Wartung/Listenern (F-12). In `scan_page` führt eine Panik zu `Err(CallerPanic)`.

### 7.6 Operationsmatrix: Was sich aus den Primitiven bauen lässt

| Fähigkeit der oberen Schicht | Gebaut aus |
|---|---|
| Referenz-Update mit Wiederholungssicherheit | `Guard::equals` auf Ref + `Guard::absent` auf Idempotenzschlüssel in **einem** `Change` |
| Mehrere Tabellen, Index, Ref atomar | ein `Change` über mehrere Spaces |
| Serialisierbare Sitzungen | `view_at(base)` + `Guard::unchanged_since` / `range_unchanged_since` |
| Zeitreise bis Stamp | `view_at` + `Hold` mit Ablauf und Eigentümer |
| Konsistenter Lesezugriff über Index und Daten | eine `View` über alle Spaces |
| Abgeleitete Indizes folgen Änderungen und holen auf | `subscribe` + `Consumer` + `history` |
| Datenschutz-Löschung eines Subjekts | `Erase::range` (§14) **oder** Vernichtung des Domänenschlüssels (§15.3) |
| Mandant/Repository löschen | `drop_space` oder Instanz-Verzeichnis nach Schlüsselvernichtung löschen |
| Backup, Test-Klon | `checkpoint` |
| Token-/byte-budgetiertes Paketieren | `estimate` zur Planung, `scan_page` zur Auslieferung, `bodies` zum Nachladen, `ResumeToken` zur Fortsetzung |
| Manipulationsarme Episoden-/Audit-Protokolle | append-only Space (+ Modus `Authenticated`) |
| Zusammen Gebrauchtes liegt nebeneinander | Cluster-Präfix im Schlüssel, `prefix_filter`, `locality_prefix` |
| Vertrauens-/Zeitfilter auf 10⁵ Kandidaten | `Filter` auf den Header (Trust, Gültigkeit, Typ) |
| Vergessen/Verkürzen (Ablauf, „Zoom-Down“) | `CompactionFilter` (§12.2) |
| Branches/Versuche (Stufe A, Overlay) | Overlay-Space + `Hold` (Eigentümer) + `range_unchanged_since` + `history` |
| Zusätzliche Run-Indizes (Volltext, Zähler) | `Extension`-Blöcke (§12.4) |
| Vektorgraphen, große Indizes | `Segment`-Dateien + `Consumer` (§12.5) |
| Arbeitsgedächtnis ohne Disk | `ephemeral`-Space |
| Dauerhafte Idempotenzschlüssel mit Ablauf | Space mit `CompactionFilter` (TTL im Header) |

**Space-Budget:** Jeder Space hat eigene Memtable, ein eigenes Run-Verzeichnis und hält über das gemeinsame Redo den langsamsten Space fest (`redo_lag_bytes`). **Faustregel ≤ 12 Spaces je Instanz.** Mandanten sind keine Spaces. Eine `remember`-Operation einer oberen Schicht ist ein `Change` über wenige Spaces (Kopf+Körper, Idempotenz, Kanten), atomar und wiederholungssicher.

### 7.7 Fehler

Ein einziges `Error`-Enum (kein `Storage(String)`). Jeder Fehler hat `code()` (stabil), `retryable()`, strukturierte Felder und `hint()`: kurzer, handlungsleitender, **datenfreier** Text, lesbar für Menschen und Sprachmodelle. Beispiele: „Space ‚trace‘ ist append-only; Schlüssel existiert bereits. Neuen Schlüssel wählen.“ — „Stamp 880 nicht mehr verfügbar; älteste verfügbare Sicht: 1 012.“

| Variante | Bedeutung | `retryable` |
|---|---|---|
| `Conflict{guard_index}` | Vorbedingung nicht erfüllt | nein (neu lesen) |
| `Busy{retry_after, reason}` | Gegendruck, `StorageFull`, Governor | ja |
| `Poisoned{cause}` | `fsync`-/E/A-/`LockPoison`-Fehler, Instanz vergiftet | nein (Neustart) |
| `Corrupt{file, offset, what}` | Prüfsumme/Struktur falsch | nein |
| `Erased{range, erased_at}` | Zugriff unterhalb eines Löschstamps | nein |
| `Shredded{domain}` | Körper einer vernichteten Schlüsseldomäne | nein |
| `Revoked` | View widerrufen | nein |
| `HoldBlocked{holds}` / `Blocked{views}` | Löschung durch Halteregel blockiert | ja (nach Freigabe) |
| `KeyStillPresent{domain}` | `Erase::domain` bei noch vorhandenem Schlüssel | nein (Schlüssel zuerst vernichten) |
| `Limit{what, max}` | Schlüssel/Wert/Change zu groß | nein |
| `Forbidden{space, op}` | Operation im Space nicht erlaubt | nein |
| `Config{what}` | Konfiguration widerspricht Space-Optionen/Format | nein |
| `Expired` / `InvalidToken` | `ResumeToken` oder Stamp nicht mehr haltbar / falsch gebunden | nein (neu beginnen) |
| `HistoryPruned{oldest_available}` | Verlauf unterhalb der Untergrenze, verworfener Consumer | nein |
| `ListenerLagged{missed_from}` | Listener-Warteschlange übergelaufen | nein (aufholen) |
| `CallerPanic{what}` | Panik in Aufrufer-Code | nein |
| `Rolledback{expected, found}` | Rollback-Anker verletzt (§15.5) | nein |
| `Unsupported{feature}` | Pflicht-Feature oder Modus unbekannt/abweichend | nein |
| `Locked`, `Closed`, `NotFound`, `Exhausted`, `Io(kind)` | | je nach Fall |

---

### 7.8 Stabile Flächen für aufsetzende Schichten (semver-stabil, SD-52)

Diese Flächen sind **Vertrag**. Der Kern implementiert sie vollständig; aufsetzende Schichten bauen darauf, ohne in den Kern greifen zu müssen. Jede Fläche hat einen Konformitätstest in `testing::conformance`.

**7.8.1 Transaktionsbausteine.** `view_at(stamp)` + `Guard::unchanged_since`/`range_unchanged_since` ersetzen Lesemengen und Validierung. Zusätzlich: `View::stamp()`, `Storage::visible_stamp()`, `Storage::durable_stamp()`, `Receipt{stamp, durable}`. Der Kern führt **keine** Lesemenge, keine Transaktionskennung und kein Rollback; ein verworfener Versuch ist ein nicht eingereichter `Change`. Verschachtelte Rücknahme (Teilziele) ist Sache der oberen Schicht (Overlay-Space, Anhang C).

**7.8.2 Holds und Checkpoints.** `hold(stamp, owner, ttl)` ist flüchtig (Registry). `hold_durable(stamp, owner, ttl) -> HoldId` schreibt `hold_set` per Edit und überlebt Neustarts; `hold_drop(id)`, `holds(owner_prefix) -> Vec<HoldInfo>`. Dauerhafte Holds zählen in die Untergrenze (§10.3), werden bei der Recovery wiederhergestellt und von der Wartung nach Ablauf entfernt. `checkpoint(dest, mode)` (§12.8) liefert den konsistenten Stand; **benannte** Checkpoints, Verzeichnis über mehrere Komponenten und Wiederherstellung sind außerhalb (NZ-17). Wiederherstellung = Öffnen des Zielverzeichnisses als eigene Instanz (§15.2, Epoche).

**7.8.3 Segmente.** `Segment` ist eine Datei der oberen Schicht mit **Kern-Rahmen** (SD-54, §12.5): `SegmentWriter` (über `Fs`, `.building`, Blöcke mit Prüfwert/AEAD je Modus, Siegel-Footer) und `SegmentReader` (`read_at`, Blockprüfung, optional `read_batch`). Der Kern kennt weder Inhalt noch Zerlegung der Blöcke; das 16-Byte-`reference` (Datenstand-Anker, z. B. Stamp-Tupel) ist opak. Besitzer können Segmentinhalte direkt aus dem Rahmen lesen, ohne `mmap` (NZ-7).

**7.8.4 Lösch-Beobachtung.** `Receipt` von `erase` ist serialisierbar und stabil (`ErasureReceipt`, §14.3 E7). Zusätzlich: `erase_status(id) -> Pending | InProgress{phase} | Done{receipt} | Unknown` (Quelle: `erase_pending`/`erase_done`; `erase_done` wird `erase_receipt_keep`-begrenzt aufbewahrt) und der **rein lesende** Prüfer `scan_physical(ScanSpec{space?, range?, patterns: &[&[u8]]}) -> ScanReport` (§14.7): durchsucht Runs, Blobs aller Generationen, Redo, `.building` und Segmente nach Mustern und meldet je Datei **nur Anzahl und Dateikennung**, nie Fundstellen-Inhalt (P-12). Der Kern entscheidet nicht, was ein „Nachweis“ ist; Positivkontrolle, Signatur und Journal gehören der oberen Schicht (NZ-14).

**7.8.5 Schlüssel.** Siehe §4 (`KeyProvider`, `SecretAlloc`). `Erase::domain` und `domain_retired` sind die einzigen schlüsselbezogenen Löschprimitive; ein Tresor, der Schlüssel vernichtet, steht außerhalb und ruft `Erase::domain` danach auf (`INV-ERA-3`).

**7.8.6 Fertigmeldung statt `async`.** `submit(change) -> Pending`; `Pending::poll() -> Option<Result<Receipt>>`, `Pending::wait()`, `Pending::on_ready(Box<dyn FnOnce(Result<Receipt>) + Send>)`. Der Callback läuft auf der `Runtime`, ohne gehaltene Sperre (P-10). Eine obere Schicht baut daraus beliebige `Future`-Fassaden; der Kern enthält keinen Executor (NZ-20). Dasselbe gilt für `Storage::flush_async`/`sync` (SD-59).

**7.8.7 Reservierungen.** `kind = 6`, Space-Flag `32`, Pflicht-Bit `MERGE`, `codec`-Byte, `BlockCodec`, optionale Feld-IDs `0x8000` (SD-12, SD-13, SD-60). Weitere Reservierungen brauchen einen Eintrag in §9.8.

## 8. Dateilayout und Dateilebenszyklus

### 8.1 Layout

```
<root>/
  FORMAT              Magic, Formatversion, required/optional Feature-Bits, integrity_mode, instance_id (§9.2)
  LOCK                exklusive (oder geteilte, read-only) Sperre über Fs::lock
  CURRENT             Zeiger auf aktive Catalog-Datei (atomar ersetzt)
  CATALOG-000007      Snapshot-Record + Edit-Records (§9.7)
  KEYS                (nur Authenticated/Encrypted) gesiegelte Schlüsselhierarchie (§9.3), atomar ersetzt
  redo/
    redo-0000000000000012.log        vorallokiert, am Ende gesiegelt (§9.4)
  spaces/
    0001/
      0000000000000a3f.run
      b0000000000000071.00000003.blob      Blob mit logischer Nummer 0x71, Generation 3 (§9.6)
    0002/ …
  segments/<kind:04x>/<id:016x>.seg        Dateien oberer Schichten (§12.5), vom Catalog verwaltet
```

### 8.2 Regeln

- **Numerische IDs** benennen alle Dateien, nie Pfad-Strings im Catalog. Blobs tragen eine **logische Nummer** `blob_no` und eine **Generation**; der Catalog bildet `blob_no → (generation, bytes, live_bytes)` ab. Referenzen in Runs nennen nur `blob_no` und bleiben bei einer Neufassung gültig (§14.3).
- **Redo-Segmente sind kein `FileKind`** (SD-44): Sie sind durch Namen (`segment_id`) und `redo_floor` bestimmt. Jedes Segment mit `segment_id ≥ redo_floor` ist lebendig (auch das vorallokierte Reservesegment, `INV-LOG-7`); jedes darunter ist verwaist. `FileKind` kennt nur `Run`, `Blob`, `Segment(kind)`. Kalter Speicher ist ein **Attribut** (`tier`), keine Dateiart.
- Alle Dateien und Verzeichnisse entstehen mit `Perm::Private` (§15.6).
- **Temporäre Dateien** tragen das Suffix `.building.<tmp_id>`; `tmp_id` stammt aus dem reservierten ID-Bereich (§9.7), nicht aus der Prozess-ID. Beim Öffnen werden alle `.building.*` entfernt.
- **Verwaiste Dateien** (Run, Blob, Segment-Datei, nicht aktuelle Blob-Generation, Redo unterhalb `redo_floor`), die der Catalog nicht nennt, werden beim Öffnen und danach periodisch gelöscht. Das ist sicher, weil nur ein Catalog-Edit eine Datei lebendig macht (`INV-CAT-1`).
- **Ein Space = ein Verzeichnis** inklusive seiner Blobs. Das Entfernen eines Spaces ist ein Edit plus `remove_dir` nach dem Löschen aller Dateien. Eine Blob-Datei gehört zu **genau einem** Space.

### 8.3 Zustandsdiagramm einer Datei

```
(nicht vorhanden) ──create .building──► im Bau ──sync_all + sync_dir──► dauerhaft, UNREFERENZIERT
      ▲                                                                       │ Edit(add) dauerhaft
      │ Verwaisten-Suche                                                      ▼
      └──────────── Unlink + sync_dir ◄── Graveyard ◄── obsolet ◄──────── LEBENDIG (Catalog nennt sie)
                    (erst nach letzter Referenz)           ▲ Edit(remove) dauerhaft
```

Eine Datei wechselt nur in Pfeilrichtung, und jeder Wechsel zwischen „unreferenziert“ und „lebendig“ ist **ein** Catalog-Edit (P-2). Blob-Dateien werden zusätzlich durch die Lebendbilanz (`live_bytes`) zu „obsolet“, wenn sie null erreicht.

---

## 9. Formate

### 9.1 Allgemeine Konventionen

- Alle Zahlen **little-endian**. `varint` = LEB128 für `u64` (7-Bit-Gruppen, niedrigwertigste zuerst, höchstens 10 Bytes). **Nur kanonische** Kodierungen sind gültig (keine überflüssigen `0x80 … 0x00`-Fortsetzungen, das zehnte Byte darf nur `0x01` sein); alles andere ist `Corrupt` (`INV-FMT-1`). Beispiele in Anhang A.
- Jede Dateiart hat **genau eine Version (`1`)** und Feature-Bits (SD-04). Unbekannte Version oder unbekanntes Pflicht-Bit ⇒ `Unsupported`.
- **Hash für Filter:** XXH64 (öffentliche Spezifikation, Implementierung im Crate `codec/xxh64.rs`, Prüfwert `XXH64("", 0) = 0xEF46DB3751D8E999`).
- **Prüfwert je Modus (`chk`):**

| Modus | Frames/Records/Blöcke | Header und Footer |
|---|---|---|
| `Plain` | CRC32C (4 B), im Redo über die Kette (§9.4) | CRC32C (4 B) |
| `Authenticated` | `Mac16` (16 B) über die Nutzbytes mit gebundenen Zusatzdaten (AAD) | CRC32C (4 B) |
| `Encrypted` | AEAD (`Cipher`): Nutzdaten verschlüsselt, der 16-Byte-Tag **ersetzt** den separaten Prüfwert | CRC32C (4 B) |

  Header und Footer tragen in allen Modi nur CRC32C (Schutz gegen Zufallsfehler). **Authentizität** entsteht, weil jeder Frame/Block/Record die Kennungen aus Header und Ort in seine AAD bindet (Dateiart, ID, Offset, Blockart, Space; im Redo `segment_id`, `frame_index`). Ein verändertes oder vertauschtes Header-/Footerfeld lässt die nachfolgenden Prüfungen scheitern.
- **Schlüsselableitung** (`Kdf`, §15.2) und **Nonces** (§15.2) sind zentral in `keys/`; kein anderes Modul bildet Nonces.
- Alle Längen werden vor Allokation gegen §6.7 und die Restlänge der Datei geprüft (`INV-FMT-2`).

### 9.2 `FORMAT`

```
"CTXS" | version u16 | required u64 | optional u64 | integrity_mode u8 | instance_id [16] | created_ms u64 | chk
chk = CRC32C über alle vorhergehenden Bytes (Plain, 4 B)  |  Mac16 mit KDF(root,"ctxs format v1") (Authenticated/Encrypted, 16 B)
```

Länge: **51 B** (`Plain`) bzw. 63 B. `integrity_mode ∈ {0 Plain, 1 Authenticated, 2 Encrypted}` steht an Offset 22, damit der Leser die Länge von `chk` vor der Prüfung kennt. Der Modus ist nach dem Anlegen **unveränderlich** (ein Wechsel ist ein Migrationswerkzeug, kein Betriebsschalter).

Pflicht-Bits: `ENCRYPTED = 1<<0`, `AUTHENTICATED = 1<<1`, `DOMAIN_SEALED = 1<<2`, `COLD_TIER = 1<<3`; weitere Bits < 1<<32 sind für Pflichtfeatures reserviert, Bits ≥ 1<<32 für optionale. Unbekanntes Pflicht-Bit ⇒ `Unsupported{feature}`; unbekanntes optionales Bit wird ignoriert. Pflicht-Bits werden nur **hinzugefügt**, und zwar per `write_atomic` **bevor** der Catalog-Edit erfolgt, der das Feature nutzt (ein gesetztes, ungenutztes Bit ist harmlos).

**Downgrade-Schutz (F-15):** `Config::integrity` ist beim Öffnen **verbindlich**. Weicht `FORMAT.integrity_mode` ab, ist das Ergebnis `Err(Unsupported{integrity_mode})`, bevor irgendeine andere Datei gelesen wird. Ein Angreifer ohne Schlüssel kann so keine Instanz „auf `Plain` umschreiben“.

### 9.3 `KEYS` (nur `Authenticated`/`Encrypted`)

```
"CTXK" | version u16 | keys_generation u64 | nonce [12] | sealed_len u32 | sealed[sealed_len]
sealed = Cipher.seal( key = Kdf(KEK, "ctxs keys v1"), nonce, aad = instance_id ‖ version ‖ keys_generation, plain )
plain  = root_key [32] | count u16 | ( space u16 | dek_gen u32 | dek [32] ) × count
```

- Die Datei wird per `write_atomic` ersetzt. `nonce` kommt aus `Entropy` und ist je Schreibvorgang neu (Schlüssel wechselt bei KEK-Rotation, Schreibvorgänge sind selten).
- **`root_key`** (zufällig, einmalig beim Anlegen) ist die Wurzel aller Datei- und Redo-Schlüssel (§15.2). Die KEK umhüllt nur `KEYS`; **KEK-Rotation ist deshalb O(1)** und fasst keine Daten an (SD-36).
- **Space-DEKs** (zufällig) sind je Space unabhängig; das Entfernen eines DEK ist der Schritt „Schlüssel vor Datei“ der kryptografischen Löschung eines Spaces (§14.3, `INV-ERA-3`).
- `keys_generation` zählt Schreibvorgänge hoch; `FORMAT` und `CURRENT` benennen die erwartete Generation nicht (sie stehen im Catalog: `keys_generation_min`, §9.7), sodass ein zurückgespielter alter `KEYS` auffällt.

### 9.4 Redo-Log

Segmentdatei (vorallokiert, §10.5), `redo/redo-<segment_id:016x>.log`.

```
Header (48 B): "CTXL" | version u16 | flags u16 | segment_id u64 | first_stamp u64 | chain_seed u32 (Entropy)
               | prealloc_len u64 | reserved u64 | crc32c u32
Frame        : len u32 | stamp u64 | op_count u32 | flags u8 | payload | chk
               len   = 13 + |payload|            (zählt stamp … payload, nicht len selbst, nicht chk)
               len == 0 beendet das Segment (vorallokierter Nullbereich)
Seal-Frame   : op_count = 0xFFFF_FFFF, flags.SEAL = 1, payload = frame_count u64 ‖ final_chain u32, stamp = Stamp des letzten Frames
Frame-flags  : Bits 0–1 Durability (0 Always, 1 Interval, 2 Never); Bit 7 SEAL; übrige Bits MÜSSEN 0 sein
```

**Prüfwert je Modus:**

| Modus | Frame |
|---|---|
| `Plain` | `chk = crc32c(seed = chk_vorher, len ‖ stamp ‖ op_count ‖ flags ‖ payload)`; für den ersten Frame ist `chk_vorher = chain_seed`. Die Kette erkennt vertauschte oder aus einer früheren Dateinutzung stammende Frames. |
| `Authenticated` | `chk = Mac16(len ‖ stamp ‖ op_count ‖ flags ‖ payload ‖ segment_id ‖ frame_index)` |
| `Encrypted` | `payload_gespeichert = Cipher.seal(redo_key, nonce(0x02 ‖ frame_index), aad = len ‖ stamp ‖ op_count ‖ flags ‖ segment_id ‖ frame_index, payload)`; `len` zählt die gespeicherte Länge; kein separates `chk` |

`frame_index` zählt Frames ab 0 je Segment, `redo_key = Kdf(root_key, "ctxs redo v1" ‖ segment_id)`. Im Modus `Encrypted` stehen `stamp`, `op_count` und `flags` im Klartext (sie sind keine Nutzdaten; `op_count` verrät die Operationszahl, das ist dokumentiert).

**Operation** (Inhalt von `payload`, `op_count` Stück hintereinander):

```
kind u8 | space u16 | opflags u8 | [domain u32 wenn opflags.DOMAIN] | klen varint | key | vlen varint | value
kind     : 1 Put, 2 Delete, 3 RangeDelete, 4 EraseRange
opflags  : Bit 0 DOMAIN, Bit 1 SEALED (Körper mit Domänenschlüssel versiegelt), Bit 2 UNBOUNDED_END; übrige Bits 0
value    : Put → header ‖ Körper (Körper bei SEALED: Cipher.seal(domain_key, nonce(0x03 ‖ segment_id ‖ frame_index ‖ op_index u24),
                                                                   aad = space ‖ domain ‖ key, body));
           Delete → leer;  RangeDelete/EraseRange → exklusives Ende des Bereichs (key = Anfang); leer bei UNBOUNDED_END
```

**Regeln:**

- **Ein Frame = ein `Change`** (`INV-LOG-1`). Beim Replay ist ein Frame entweder vollständig gültig oder er beendet den Log (`INV-LOG-2`). Ein Gruppen-Schreibvorgang (mehrere Frames, ein `write_all_at`) kann zerreißen; jedes vollständig gültige **Präfix** ist zulässig (unquittiert, aber dauerhaft ist erlaubt).
- **Stamps** in einem Log sind streng aufsteigend (`INV-LOG-3`), aber **nicht lückenlos**: rein flüchtige Changes verbrauchen Stamps ohne Frame.
- **Körper reisen im Frame** (SD-29). Der Redo ist der einzige dauerhafte Ort eines Körpers bis zum Flush; dort wandert er in die Blob-Datei. Dadurch bleibt es bei **einem** `fsync` je Gruppe.
- **Siegel statt Raten (`INV-LOG-6`).** Ein Segment ist *gesiegelt*, wenn sein letzter gültiger Frame ein Seal-Frame ist. **Jedes Segment außer dem neuesten Segment mit Frames ist gesiegelt.** Ein Defekt in einem gesiegelten Segment ist `Corrupt` (Mittendefekt, fail-closed). Ein ungesiegeltes Segment ist nur zulässig, solange alle späteren Segmente keine Frames enthalten. `frame_count` im Siegel MUSS zur Zahl der gelesenen Frames passen.
- **Jede Prozessinstanz beginnt ein neues Segment** (`INV-LOG-4`); es wird nie an ein bestehendes angehängt.
- **Reserve-Segment (`INV-LOG-7`):** Ein Segment ohne Frames, dessen Header gültig ist, ist die vorallokierte Reserve und **lebendig**. Die Recovery übernimmt es; die Verwaisten-Suche löscht nur Segmente mit `segment_id < redo_floor`.
- **Rotation** bei `redo_segment_bytes` (64 MiB). Ein Segment ist löschbar, wenn **alle** Spaces `flushed_stamp ≥ max_stamp(segment)` haben und `redo_floor` es überschritten hat. Ein Frame (≤ 32 MiB) MUSS in ein Segment passen; die Gruppe wird vor dem Schreiben auf die Restgröße geprüft und notfalls das Segment gesiegelt und gewechselt.
- **HWM-Verankerung (`INV-LOG-8`):** Jeder Catalog-Edit trägt `redo_hwm` = höchster Stamp, dessen Frame zum Zeitpunkt des Edits per `sync_data` gesichert war. Findet die Recovery danach Redo-Segmente, deren letzter gültiger Stamp **unter** `redo_hwm` liegt (und die Lücke nicht durch `redo_floor_stamp` erklärt ist), ist das Ergebnis `Corrupt{RedoTruncated}`: ein verlorener oder abgeschnittener Redo-Schwanz wird erkannt, statt still Daten zu verlieren (setzt die „High-Water-Mark-Verankerung“ der Altspezifikation um, siehe Anhang B).

### 9.5 Run

```
Run    := DataBlock* ExtensionBlock* RangeDelBlock? FilterBlock PrefixFilterBlock? IndexBlock MetaBlock DirectoryBlock Footer
Block  := stored_payload | codec u8 | chk                 (Encrypted: stored_payload = sealed, kein separates chk)
          AAD (Authenticated/Encrypted) = 0x01 (Run) ‖ run_id ‖ block_offset ‖ block_kind ‖ space
          block_kind: 0 Data, 1 Index, 2 Filter, 3 PrefixFilter, 4 RangeDel, 5 Meta, 6 Directory, 0x1000+id Extension
Footer (28 B): "CTXR" | version u16 | flags u16 | directory_offset u64 | directory_len u32 | crc32c u32 | "CTXR"
```

**Öffnen:** Footer → Directory → Meta (liefert `weights_k`, `header_len`, `space`) → Index. Filter und Datenblöcke werden **lazy** geladen. Vor jedem Zugriff werden Offset + Länge gegen die Dateilänge und gegen Überlappung geprüft.

- **DataBlock** (Ziel `block_bytes` = 4 KiB, ein einzelner größerer Eintrag erhält einen eigenen Block): präfixkomprimierte Einträge
  `shared varint | unshared varint | vlen varint | tag u64 | key_suffix[unshared] | value[vlen]`, Neustartpunkt alle 16 Einträge (dort `shared = 0`), am Ende `restart_offsets u32[n] | n u32`. `value = header ‖ (Körper | BlobRef)` je nach `kind` (`Put` bzw. `PutRef`); `Delete` hat `vlen = 0`.
- **BlobRef (28 B):** `blob_no u64 | offset u64 | len u32 | crc32c u32 | domain u32`. `len` ist die **logische** Körperlänge (vor Versiegelung), `crc32c` über die gespeicherten Bytes, `offset` der Anfang des Records in der Blob-Datei. Verweist auf die **logische** `blob_no`, nicht auf eine Datei. `blob_no = 0` bedeutet „Körper vernichtet“ (`Entry::body()` ⇒ `Err(Shredded{domain})`).
- **IndexBlock:** `count varint`, je Datenblock `sep_len varint | separator | offset u64 | len u32 | entries u32 | weights[k] u64 | max_stamp u64`. Der Trenner ist der kürzeste Schlüssel ≥ letzter Schlüssel des Blocks und < erster des nächsten. **`weights[0]` sind die logischen Bytes** (Schlüssel + Header + Körperlänge, bei `PutRef` aus `BlobRef.len`), `weights[1..k]` stammen vom `Weigher`. Die Summen tragen `estimate`; `max_stamp` trägt `Guard::range_unchanged_since` und das Überspringen von Blöcken in `history`. Gewichte sind `u64` (ein Block aus `PutRef`-Einträgen kann > 4 GiB logische Bytes beschreiben, SD-28).
- **DirectoryBlock:** `count varint`, je Eintrag `kind u16 | offset u64 | len u32 | required u8` für Index, Filter, PrefixFilter, RangeDel, Meta und jeden Extension-Block. Unbekannte optionale Blöcke werden ignoriert, unbekannte Pflichtblöcke (`required = 1`) verweigern das Öffnen.
- **FilterBlock:** Bloom-Filter über **Nutzerschlüssel** (distinkt), `bits_per_key = 10`, Größe aus der **tatsächlichen** Schlüsselzahl (behebt den festen 100 000-Elemente-Filter des Altstands). Layout: `k u8 | nbits u64 | bits[nbits/8]`, `nbits = max(64, ⌈n · bits_per_key⌉)` aufgerundet auf ein Vielfaches von 64, `k = 7`. Positionen: `h1 = XXH64(key, 0)`, `h2 = XXH64(key, 0x9E3779B97F4A7C15) | 1`, `pos_i = (h1 + i·h2) mod nbits` für `i = 0..k` (Doppel-Hashing, Berechnung in `u64` mit Umlauf).
- **PrefixFilterBlock** (`prefix_filter(n)`): gleiches Layout über die ersten `n` Bytes jedes Schlüssels (kürzere Schlüssel gehen vollständig ein). Ein Scan mit Präfix kürzer als `n` kann ihn nicht nutzen und überspringt ihn.
- **RangeDelBlock:** `count varint`, je Marker `start (len varint + bytes) | end_flag u8 (0 begrenzt, 1 unbegrenzt) | [end (len varint + bytes)] | tag u64`; sortiert und fragmentiert; beim Öffnen vollständig im Speicher (klein). `EraseRange`-Marker liegen hier mit `kind = 4`.
- **MetaBlock** (feste Reihenfolge, Version im Footer): `space_id u16 | run_id u64 | entries u64 | min_stamp u64 | max_stamp u64 | range_markers u32 | erase_markers u32 | created_ms u64 | header_len u16 | weights_k u8 | weigher_version u32 | feature_bits u64 | min_key | max_key | blob_count varint | (blob_no u64, bytes u64) × blob_count`. `min_key`/`max_key` (je `len varint + bytes`) umfassen **auch** die Bereichsmarker, damit Erase-Runs sicher überlappen.
- **ExtensionBlock** (optional): von einer oberen Schicht über `Extension` je Run erzeugt (§12.4).
- **Kompression:** `codec` ist 0 (keine). Ein `BlockCodec`-Trait ist reserviert (SD-13).
- **Leseweg:** `read_at` über `Fs`, kein `mmap`.

### 9.6 Blob-Datei

```
Header (48 B): "CTXB" | version u16 | flags u16 | blob_no u64 | generation u32 | space u16 | reserved [22] | crc32c u32
Record*      : rlen u32 | content | chk
               rlen = |content|;  content = kind u8 | domain u32 | stamp u64 | klen varint | key | body   (kind: 1 Body)
Hole-Record  : rlen u32 | 0x02 | zeros[rlen-1] | tag                   (tag: Plain crc32c; Authenticated/Encrypted Mac16(blob_no ‖ offset ‖ rlen ‖ 0x02))
Footer       : "CTXB" | record_count u32 | bytes u64 | crc32c u32       (20 B)
```

- **Selbstbeschreibend:** Jeder Record trägt `(domain, stamp, key)`. Dadurch kann `verify` Blobs ohne den Run prüfen, die GC Lebendigkeit prüfen und die Löschprüfung (§14.7) Reste finden.
- **Prüfwert je Modus:** `Plain` = CRC32C über `content`. `Authenticated` = `Mac16(content ‖ blob_no ‖ offset)`. `Encrypted`: `content` wird mit dem Blob-Schlüssel `Kdf(space_dek, "ctxs blob v1" ‖ blob_no)` und Nonce `0x04 ‖ blob_no u56 ‖ offset u32` versiegelt, `AAD = space ‖ blob_no ‖ offset`; der Tag ersetzt `chk`. Der Schlüssel steht **innerhalb** des versiegelten Inhalts, ein vertauschter oder verschobener Record öffnet nicht.
- **Domänenversiegelung** (`domain_sealed`): Der Körper (`body`) ist zusätzlich mit dem Domänenschlüssel versiegelt: `Cipher.seal(domain_key, nonce(0x04 ‖ blob_no u56 ‖ offset u32), aad = space ‖ domain ‖ key, body)`; bei `Encrypted` liegt das Ergebnis noch einmal unter dem Blob-Schlüssel. Wird der Domänenschlüssel vernichtet, ist der Körper für jeden unlesbar, auch mit dem Space-DEK.
- **Append-only während des Baus, danach unveränderlich.** Blobs entstehen ausschließlich im Flush und in der Blob-GC/Konsolidierung (§12.3) über `.building` → `sync_all` → `sync_dir` → Catalog-Edit. Footer wird beim Abschluss geschrieben.
- **Generation:** Eine Neufassung mit **gleichen Offsets** (Blob-Sweep bei `Erase`, §14.3) erzeugt Generation `g+1`: gelöschte Records werden zu **Hole-Records gleicher Länge mit Nullen**, die übrigen bleiben **byte-identisch**. Der Catalog tauscht `(blob_no, g) → (blob_no, g+1)` in einem Edit; Referenzen in Runs bleiben gültig.
- **Hole-Records werden nie verschlüsselt** (`INV-BLOB-3`): Eine Verschlüsselung der Nullen unter demselben `(Schlüssel, Nonce)` wie der ursprüngliche Record würde `C_alt ⊕ C_neu = Klartext_alt` verraten. Der Tag der Hole-Records ist nur ein MAC, keine Chiffre. Ein `BlobRef`, der in ein Hole zeigt, liefert `Corrupt{HoleRef}`.
- **Größe:** `blob_target_bytes` = 64 MiB; ein einzelner Record > Ziel erhält eine eigene Datei. Das begrenzt zugleich die Kosten eines Sweeps.
- **Kein Teilen:** Jeder `BlobRef` zeigt auf **einen eigenen** Record (`INV-BLOB-2`). Es gibt kein Dedup; dadurch ist die Lebendbilanz exakt.
- Die Lebendbytes je `blob_no` führt der Catalog (`blob_live_delta`), sodass die GC-Planung keine Datei öffnen muss.

### 9.7 Catalog

```
Datei  : Header (20 B) "CTXC" | version u16 | flags u16 | catalog_no u64 | crc32c u32, dann Record*
Record : len u32 | tag u8 (1 Snapshot, 2 Edit) | payload | chk            (len = 1 + |payload|; Encrypted: payload versiegelt, kein chk)
chk    : Plain  crc32c(seed = chk_vorher, tag ‖ payload)      (erster Record: seed = catalog_no als u32 gefaltet)
         Auth.  Mac16(chk_vorher ‖ len ‖ tag ‖ payload ‖ catalog_no ‖ record_index)
         Encr.  Cipher.seal(Kdf(root_key,"ctxs cat v1" ‖ catalog_no), nonce = record_index u64 ‖ 0u32, aad = chk_vorher ‖ len ‖ catalog_no ‖ record_index)
```

Der **Snapshot** ist ein Edit, der vom leeren Zustand ausgeht (alle Listen vollständig). Das Lesen kennt damit nur eine Anwendungsregel. Ein **Edit** ist **ein einzelner Record** (P-2) aus TLV-Feldern `field_id u16 | len varint | bytes`; gesetztes Bit `0x8000` in `field_id` markiert ein **optionales** Feld (unbekannt ⇒ überspringen), unbekannte Pflichtfelder ⇒ `Unsupported`.

| Feld-ID | Name | Inhalt |
|---|---|---|
| 0x0001 | `add_files` | `[FileMeta]`; `FileMeta = kind u8 (1 Run, 2 Blob, 3 Segment) \| id u64 \| space u16 \| level u8 \| tier u8 (0 Local, 1 Cold) \| flags u16 \| min_stamp u64 \| max_stamp u64 \| bytes u64 \| entries u64 \| generation u32 \| segment_kind u16 \| blob_no_min u64 \| blob_no_max u64 \| min_key \| max_key` (Schlüssel als `len varint + bytes`). Für Blobs ist `id = blob_no`. |
| 0x0002 | `remove_files` | `[(kind u8, id u64)]` |
| 0x0003 | `blob_live_delta` | `[(blob_no u64, delta i64)]`, im **selben** Edit wie die Compaction, die Versionen verwirft; `live_bytes == 0` ⇒ Blob im selben Edit entfernt |
| 0x0004 | `blob_replace` | `[(blob_no, gen_alt u32, gen_neu u32)]` (Sweep, gleiche Offsets) |
| 0x0005 | `space_flushed` | `[(space u16, stamp u64)]`: Memtable bis `stamp` ist in Runs |
| 0x0006 | `redo` | `redo_floor u64 \| redo_floor_stamp u64 \| redo_hwm u64` (§9.4, `INV-LOG-8`) |
| 0x0007 | `reserve` | `run_id_hwm u64 \| blob_no_hwm u64 \| segment_id_hwm u64 \| tmp_id_hwm u64 \| keys_generation_min u64`: **Hochwassermarken, nie zurück** (siehe unten, `INV-KEY-2`) |
| 0x0008 | `spaces_add` | `[(id u16, name, options)]`, `options = header_len u8 \| weights_k u8 \| flags u16 (1 separate_body, 2 domain_sealed, 4 append_only, 8 ephemeral, 16 forgetting, 32 merge [reserviert]) \| separate_threshold u32 \| prefix_filter u8 \| locality_prefix u8` |
| 0x0009 | `spaces_drop` | `[id u16]` |
| 0x000A | `consumer_set` | `[(name, consumed_stamp u64, max_lag_bytes u64)]` |
| 0x000B | `consumer_drop` | `[name]` |
| 0x000C | `domain_retired` | `[(domain u32, stamp u64)]`: Nachweis einer Schlüsselvernichtung (§15.3) |
| 0x000D | `erase_pending_add` | `[(id u64 (= S), space u16, range, level u8)]` |
| 0x000E | `erase_done` | `[id u64]` |
| 0x000F | `erase_swept` | `[id u64]` |
| 0x0010 | `erased_ranges_add` | `[(space u16, range, S u64)]` |
| 0x0011 | `erased_ranges_remove` | `[(space u16, range, S u64)]` |
| 0x0012 | `hold_set` | `[(hold_id u64, stamp u64, owner, expires_ms u64)]`: **dauerhafte** Holds (§7.8.2, SD-58) |
| 0x0013 | `hold_drop` | `[hold_id u64]` |
| 0x0014 | `key_epoch` | `[(space u16, epoch u32)]`: Schlüsselepoche je Space; steigt bei beschreibbaren Kopien (§15.2, SD-63) |

**ID-Reservierung (`INV-KEY-2`, schließt F-14).** `run_id`, `blob_no`, `segment_id` und `tmp_id` dürfen erst benutzt werden, wenn ein **dauerhafter** Edit eine Hochwassermarke über sie hinweg gesetzt hat. Die Wartung reserviert in Blöcken zu `id_reserve_block` (256), bevor der Vorrat erschöpft ist. Nach einem Absturz beginnt die Vergabe bei der Hochwassermarke; ungenutzte IDs verfallen. Damit wird **keine ID je nach einem Absturz mit anderem Inhalt wiederverwendet**; das ist Voraussetzung für Nonce-Eindeutigkeit (§15.2). `segment_id` und `blob_no` sind unabhängig von `run_id`; alle vier sind `< 2^56` (`segment_id < 2^32`) und überlaufen mit `Exhausted`.

- Die Metadaten stehen **im Catalog**: Auswahl für Compaction, Blob-GC und Löschung läuft, ohne eine Datei zu öffnen (B-8 des Altstands). Die Reihenfolge der Level-0-Runs ergibt sich aus `max_stamp`.
- **Rollover** (Größe > 8 MiB oder beim Öffnen mit > 1 000 Edits): neue Datei `CATALOG-(n+1)` mit einem Snapshot-Record schreiben, `sync_all`, `CURRENT` per `write_atomic` umhängen, `sync_dir`, alte Datei löschen. `CURRENT` enthält `catalog_no` als Text und `crc32c`.
- **Lesen:** Ein beschädigter oder halber **letzter** Record wird verworfen (Torn Write), jeder frühere Defekt ist `Error::Corrupt` (`INV-CAT-3`, fail-closed, kein „Wiederauferstehen“ gelöschter Dateien).
- Im Modus `Authenticated/Encrypted` ist `chk` des letzten Records der Kopf der Kette; `head_mac = Kdf(cat_key, [catalog_no, record_index, chk])` wird dem `RollbackAnchor` übergeben (§15.5).

### 9.8 Format-v1-Freeze-Liste (Gate **G-F**, vor Phase F3)

Jeder Punkt würde später eine Formatversion kosten, obwohl SD-04 genau eine je Dateiart verspricht. Sie werden in F0 als **Spezifikation durch Beispiel** (Goldens, Anhang A) festgeschrieben und von einem zweiten Reviewer abgenommen.

| # | Eintrag | Ort |
|---|---|---|
| 1 | `weights[k]` als `u64`, `weigher_version`, `max_stamp` je IndexBlock-Eintrag | Run |
| 2 | `header_len` je Space | Catalog, MetaBlock |
| 3 | Tag-Kind `PutRef`, `BlobRef` (28 B) inkl. `domain`, `blob_no = 0` als „vernichtet“ | Run |
| 4 | `FileKind`/`FileMeta` (Run/Blob/Segment) mit `tier`, `blob_no_min/max` | Catalog |
| 5 | `blob_no`-Indirektion, `generation`, `blob_live_delta`, `blob_replace`, Hole-Records ohne Verschlüsselung | Blob, Catalog |
| 6 | `consumer_set`, `max_lag_bytes` | Catalog |
| 7 | Durability-Bits je Frame, `ephemeral`-Flag je Space | Redo, Catalog |
| 8 | `integrity_mode`, Modus-Bits, Downgrade-Regel | FORMAT |
| 9 | Redo-Seal-Frame, `prealloc_len`, `chain_seed`, `redo_hwm`, `redo_floor_stamp` | Redo, Catalog |
| 10 | `domain` in Op und Blob-Record, `opflags.SEALED`, `domain_retired` | Redo, Blob, Catalog |
| 11 | `reserve` (ID-Hochwassermarken), `keys_generation_min` | Catalog |
| 12 | `KEYS`-Layout mit `root_key` und Space-DEKs | KEYS |
| 14 | Catalog `hold_set`/`hold_drop`/`key_epoch`, Segment-Rahmen (`CTXG`), Epoche in der Schlüsselableitung | Catalog, Segment, KEYS |
| 13 | Reservierte Werte: `kind = Merge (6)`, Codec-IDs, Bit-Bereiche für Pflicht-/optionale Features, optionale Feld-ID-Hälfte `0x8000` | alle |
---

## 10. Schreibpfad

### 10.1 Zustandsmodell

```
Storage
 ├─ committer: Mutex<CommitterState>        genau eine aktive Gruppe; als einziger Lock über E/A gehalten
 ├─ intake:    Mutex<VecDeque<Ticket>>      Eingangswarteschlange (Haltezeit Nanosekunden)
 ├─ state:     RwLock<Arc<Version>>         Haltezeit Nanosekunden (Arc klonen / austauschen)
 ├─ memtable (je Memtable): RwLock<BTreeMap<InternalKey, Slot>>
 ├─ registry:  Mutex<Registry>              Views, Holds, Consumer, Untergrenze, Claims (Blatt)
 ├─ maint:     Mutex+Condvar je Lane        urgent / background (Blatt)
 ├─ notify:    Mutex<ListenerQueue>         Listener-Warteschlange (Blatt)
 ├─ visible_stamp: AtomicU64                Release beim Veröffentlichen, Acquire beim Lesen
 ├─ durable_stamp: AtomicU64                höchster per sync_data gesicherter Stamp
 └─ env: Env

Version (unveränderlich)
 └─ spaces: Map<SpaceId, SpaceState{ active: Arc<Memtable>, frozen: Vec<Arc<Memtable>>,
                                      levels: Vec<Vec<Arc<FileHandle>>>, blobs: Map<blob_no, Arc<BlobHandle>>,
                                      flushed_stamp, erased_ranges }>
```

Es gibt **sieben** Sperrränge (§16.1) und genau einen Lock über E/A (`committer`). Das vollständige Protokoll mit Beweis steht in §16. `InternalKey` ordnet nach Nutzerschlüssel aufsteigend, `tag` absteigend (§6.2). Die Memtable zählt `Schlüssel + Header + Körper` gegen `memtable_bytes`.

### 10.2 `apply` (Flat-Combining Group-Fsync)

```
apply(change):
  1. validate: Limits, Spaces vorhanden, Rechte (Typ), Mischverbot ephemeral/dauerhaft, append-only-Regeln, Duplikate   (ohne Sperre)
     Guard-Vorwärmung (MUSS, SD-53; H-S2 prüft sie mit kaltem Cache und vielen Guards): Blöcke für Guards (Bloom, Index, Datenblock) vorab in den BlockCache laden (ohne Sperre)
  1b. Schlüssel: Für jeden Change mit `domain_sealed`-Körpern `KeyProvider::domain_key_create(d)` (Cache), danach EINMAL `make_durable()`,
      bevor das Ticket eingereiht wird (ohne Sperre, P-10). Fehler ⇒ `Err` an den Aufrufer, kein Frame (`INV-KEYP-1`).
  2. Ticket in `intake`; danach committer.lock(): wer den Committer bekommt, wird Leiter (Ticket schon erledigt ⇒ zurück)
Leiter:
  3. Gegendruck-Prüfung (§10.4)
  4. intake leeren (≤ 1 000 Changes / 8 MiB)
  5. je Change in Reihenfolge: Guards gegen (neueste Version + Gruppen-Overlay) prüfen;
       append-only: Existenzprüfung je Put;
       Fehlschlag ⇒ nur dieser Change erhält Err(Conflict|Forbidden), verbraucht keinen Stamp
       Erfolg    ⇒ Stamp = next_stamp++ ; Körper domänenversiegelter Spaces versiegeln ; Frame kodieren (MAC/AEAD je Modus)
       rein flüchtige Changes ⇒ Stamp, aber kein Frame
  6. Passt die Gruppe nicht in den Rest des Segments ⇒ siegeln und wechseln (§10.5)
  7. alle Frames mit EINEM write_all_at an das Redo-Segment anhängen                       Hook: AfterRedoWrite
  8. Sync nach der höchsten Anforderung der Gruppe: enthält sie ein Always ⇒ sync_data
       Fehler ⇒ Poison, alle Tickets Err(Poisoned), NICHTS in Memtables                      Hook: AfterRedoSync
       Erfolg ⇒ durable_stamp = höchster Stamp der Gruppe (bei Always)
  9. Memtables in Stamp-Reihenfolge füllen, in Unterbatches (≤ 64 Ops je Schreibsperre)      Hook: BeforeVisible
 10. visible_stamp.store(letzter Stamp, Release)                                             ← Sichtbarkeitspunkt
 11. Tickets mit Receipt{stamp, durable} wecken; Listener-Warteschlange füttern
 12. Memtable über Grenze ⇒ rotieren (active → frozen), Flush-Job anmelden
```

Folgen:

- **Memtables enthalten nur Daten aus erfolgreich geschriebenen Gruppen** (`INV-VIS-1`). Bei `Interval`/`Never` sind sie nicht zwingend dauerhaft; ein Fehler in Schritt 8 betrifft aber nur Gruppen, die nie in eine Memtable gelangten. Es gibt keinen Rückbau nach einem Fehler.
- **Teilweises Füllen ist unsichtbar** (`INV-VIS-2`): Solange `visible_stamp` nicht veröffentlicht ist, filtern Leser Einträge mit größerem Stamp aus. Deshalb darf der Leiter die Schreibsperre zwischen Unterbatches freigeben; die Leser-Latenz wird durch **64 Ops** statt durch die Gruppengröße begrenzt.
- Ein einziger `fsync` je Gruppe; die Gruppengröße ergibt sich aus der Last (kein Zeitfenster-Parameter).
- Stamps sind lückenlos über die **angenommenen** Changes (`INV-STAMP-1`).
- **Guards unter dem Committer-Lock** können Blockzugriffe auslösen (Cache-Fehlgriff): Das ist bewusst, weil Guards gegen den neuesten Stand geprüft werden müssen. Die Vorwärmung (Schritt 1) und Bloom-Filter halten das selten; H-S2 misst den Effekt.
- **Ephemerale Spaces** laufen durch dieselbe Gruppe ohne Redo-Anteil (kein Mischen mit dauerhaften Spaces).
- **Domänenversiegelung** (Schritt 5) ruft `Cipher` im Committer auf (P-10, Ausnahme). Fehlt der Domänenschlüssel (`None`), scheitert der `Change` mit `Shredded{domain}`; ein Schreibvorgang in eine vernichtete Domäne ist verboten.

### 10.3 Sichtbarkeit, Views und Untergrenze

`View` = `(Arc<Version>, stamp)`. Lesen ignoriert Einträge mit größerem Stamp. Die **Untergrenze** `retention_floor` ist **monoton nicht fallend** und wird unter dem Registry-Lock berechnet als `min(visible_stamp, Stamp aller lebenden Views, Stamp aller Holds, consumed_stamp aller Consumer)`. Registrierung einer View, die Prüfung `stamp ≥ retention_floor` bei `view_at` und die Berechnung der Untergrenze laufen unter **demselben** Registry-Lock; daraus folgt `INV-RET-1` (§16.4). Holds und Consumer haben Ablauf bzw. Maximalrückstand (`Clock`, `max_lag_bytes`); abgelaufene werden von der Wartung entfernt (keine Zombie-Holds). Ein Hold trägt einen **Eigentümer** (z. B. Branch-Name), damit ein verworfener Branch seine Halteregeln zuverlässig freigibt.

### 10.4 Gegendruck

Maßgeblich sind: eingefrorene Memtables, Level-0-Runs, ausstehende Compaction-Bytes, Redo-Rückstand (`redo_lag_bytes`) und der Speicherstand des `Governor`.

| Stufe | Auslöser | Wirkung |
|---|---|---|
| Normal | unter Verlangsamung | keine |
| Verlangsamt | L0 ≥ 8 oder frozen ≥ 2 oder Governor weich | Leiter wartet bis zu `stall_slice` je Gruppe; Governor verkleinert Caches |
| Gestoppt | L0 ≥ 12 oder frozen ≥ 4 oder Governor hart | `apply` wartet höchstens `stall_timeout`, danach `Err(Busy{retry_after})` |

`Busy` ist strukturiert und wiederholbar (wichtig für Agenten). Der Flush läuft auf der **urgent**-Lane (§12.1) und wird von Compaction nie verzögert; das verhindert den Zustand „gestoppt“ durch eine lange Compaction.

### 10.5 Redo-Segment-Lebenszyklus, `ENOSPC` und Poison

**Vorallokation.** Die `background`-Lane legt das *nächste* Segment im Voraus an: `.building` erzeugen, `preallocate(redo_segment_bytes)`, Header schreiben, `sync_all`, umbenennen, `sync_dir`. Eine Rotation im `apply` ist dann ein Wechsel auf eine fertige Datei: der Commit-Pfad enthält weder `create` noch `sync_dir`. Auf vorallokierten, beschriebenen Dateien muss `sync_data` keine Dateisystem-Metadaten schreiben (Hypothese H-S13). Nebeneffekt: `ENOSPC` tritt **bei der Vorallokation** auf, nicht mitten im Commit. Ist beim Rotieren keine Reserve vorhanden, legt der Leiter das Segment synchron an (seltener Pfad, `Busy` bei `ENOSPC`).

**Ende-Erkennung.** Der vorallokierte Rest besteht aus Nullen; ein Frame mit `len == 0` beendet das Segment. Das zufällige `chain_seed` verhindert, dass Frames einer früheren Dateinutzung als gültig erkannt werden.

**Fehlerregeln beim Anhängen:**

| Fehler | Aktion | Zustand danach |
|---|---|---|
| `write_all_at` scheitert mit `ENOSPC` oder Kurzschreiben | **Abschneiden auf den letzten guten Offset** (`set_len(good)` + `sync_data`), Tickets der Gruppe `Err(Busy{StorageFull})` | heil, wiederholbar; Segment bleibt ungesiegelt-und-gültig |
| Abschneiden scheitert **oder** anderer `write`-Fehler (`EIO` …) | **Poison** | tot; Neustart heilt das Segment in der Recovery |
| `sync_data` scheitert | **Poison** (`INV-LOG-5`), nie wiederholen, nichts in die Memtable | tot; nach Neustart gilt das Frame, falls es dauerhaft wurde, sonst nicht |

Ohne die erste Zeile würde `ENOSPC` nach Teilschreiben einen Torn-Tail mitten im Log hinterlassen, den die nächste Recovery wegen `INV-LOG-6` als `Corrupt` einstufen müsste.

**Poison-Semantik.** Ein vergiftetes Handle verweigert alle Schreibvorgänge und Wartungsjobs mit `Error::Poisoned{cause}`. **Lesen bleibt erlaubt** (degradierter Betrieb): Memtables enthalten nur Daten aus erfolgreich synchronisierten Gruppen, Runs sind unverändert und der Catalog zeigt nur dauerhaft gemachte Dateien. Ein fehlgeschlagener `fsync` einer Run-Datei geschah vor dem Catalog-Edit; die Datei war nie lebendig. `Storage::health()` meldet `Healthy | Poisoned{cause}`.

**Siegeln.** Beim Rotieren hängt der Leiter einen Seal-Frame an (`sync_data`), bevor das nächste Segment genutzt wird (`INV-LOG-6`).

---

## 11. Lesepfad

- **`get`:** aktive Memtable → eingefrorene (neu nach alt) → Level 0 (neu nach alt, nur Runs mit passendem Schlüsselbereich) → je tieferes Level genau ein Run (Binärsuche über Catalog-Metadaten). Pro Run: Bloom-Filter, Index, **ein** Block. Bereichsmarker aus Memtables und berührten Runs werden gegen die Versionen geprüft. Der Wertkörper (`PutRef`) wird **erst bei `Entry::body()`** gelesen.
- **`get_many`** bündelt die benötigten Blöcke je Run zu einem `read_batch`. **`bodies`** sortiert die Körperzugriffe nach `(blob_no, offset)` und fasst Lücken ≤ 16 KiB zu einem Lesevorgang zusammen (ein Recall mit 12 Gewinnern liest wenige, größere Bereiche statt zwölf Einzelzugriffe).
- **`scan`** liefert einen `Cursor` (Streaming, mischend). Die Memtable wird in Blöcken zu 64 Einträgen mit erneutem Aufsetzen (`seek_after(last_key)`) gelesen, sodass keine Lesesperre über die Dauer eines Scans gehalten wird. Bereichsmarker des Scan-Bereichs werden einmal eingesammelt und als Intervallliste mit maximalem Stamp geprüft. Folge-Blöcke eines sequentiellen Scans werden per `read_batch` **vorgelesen** (Tiefe über `Governor` begrenzt).
- **`scan_page`, `estimate`, `history`** nutzen dieselbe Zusammenführung. `estimate` liest nur Index-Blöcke (im Cache), `history` verwirft Runs und Blöcke, deren Stamp-Bereich `(von, bis]` nicht schneidet (`max_stamp`).
- **Filter-Pushdown:** `Filter::keep(key, header)` wird **vor** dem Laden eines Körpers ausgewertet, im Scan ohne gehaltene Sperre auf einem Block-Snapshot.
- **Präfix-Scans** fragen zuerst den `PrefixFilterBlock` der Runs ab.
- **Leseordnung (`INV-VIS-3`):** Ein Leser lädt **zuerst** `visible_stamp` (Acquire) und **danach** die Version. Die umgekehrte Reihenfolge ist verboten (Beweis §16.3).
- **Caches:** `TableCache` hält höchstens `max_open_files` offene Dateien (Runs und Blobs). `BlockCache`: genau **eine** Implementierung (CLOCK, geteilt über `Governor`, Quote je Instanz), Schlüssel `(file_id, block_no)`, `invalidate_file`. Körper werden **nicht** im `BlockCache` gehalten (Seitencache des Betriebssystems genügt; ein kleiner optionaler `BodyCache` ist Hypothese H-S15).
- **`AccessSink`** erhält gesampelt `(space, file, block_no, key_prefix, bytes)`; Standard ist die Null-Senke.
- **Fehler:** `Corrupt` und `Io` werden **immer** als Fehler gemeldet, nie als „leer“ (F-6, Gegenstück zu B-9a des Altstands). Liegt ein Lesepunkt in einem `erased_range` (§14.6), antwortet der Leser mit `Erased`.
- **Körper vernichteter Domänen:** `Entry::body()` prüft das Register `domain_retired` **zuerst** und liefert `Shredded{domain}`, auch wenn der Körper noch als Klartext in einer Memtable steht (`INV-ERA-7`).

---

## 12. Wartung: Lanes, Flush, Compaction, Blob-GC

### 12.0 Lanes, Claims, Prioritäten (SD-32, ändert SD-16)

Zwei **Lanes**, je höchstens ein Job je Instanz, beide auf der geteilten `Runtime`:

| Lane | Jobs (absteigende Priorität) | Eigenschaft |
|---|---|---|
| `urgent` | `Erase` (E2–E7) > `Flush` > Intervall-`fsync` | kurz, darf nie hinter Compaction warten |
| `background` | `Compaction` (L0 zuerst) > Blob-GC/Konsolidierung > Segment-Bau > Redo-Vorallokation > ID-Reservierung > Cleanup (Graveyard, Verwaiste, abgelaufene Holds/Consumer) | lang, ratenbegrenzt, abbrechbar |

**Claim-Tabelle (in der Registry).** Ein Job reserviert beim Planen **alle** Eingabedateien in **einem** kritischen Abschnitt (alles oder nichts). Der Planer ist eine reine Funktion `plan(&Version, &Params, floor, &Claims)` und überspringt beanspruchte Dateien. Ein Claim scheitert, wenn die Datei `obsolete` ist (§16.5, `INV-CLAIM-1`). Beanspruchen ein `urgent`- und ein `background`-Job dieselbe Datei (z. B. `Erase` und Compaction desselben Runs), setzt der `urgent`-Job ein `cancel`-Flag. Der `background`-Job prüft es an **Ausgabegrenzen** (spätestens alle 16 MiB Eingabe), verwirft seine `.building`-Ausgabe, gibt die Claims frei und beendet sich. Es gibt keinen Wartezyklus.

**Drosselung.** Die `background`-Lane läuft unter Token-Buckets des `Governor` (Lese- und Schreibbytes je Sekunde, `Clock`-basiert); `urgent` ist nur durch Speicher begrenzt.

### 12.1 Flush (je Space)

```
0. INV-FLUSH-1: Ist durable_stamp < max_stamp der Memtable, den Redo bis max_stamp per sync_data sichern
   (Aufruf von Storage::sync() über den Committer). Gilt für Always, Interval und Never.
1. frozen Memtable → Run bauen (.building): retain mit aktuellem floor, Einträge in Schlüsselreihenfolge;
   bei separate_body/domain_sealed: Körper in derselben Reihenfolge in eine Blob-Datei (.building) schreiben,
   Run enthält PutRef (Körper werden erstmals mit Blob-Nonce bzw. unter Domänenschlüssel mit Blob-Nonce versiegelt;
   ist der Domänenschlüssel inzwischen vernichtet ⇒ BlobRef{blob_no=0}, Körper nicht schreiben)
2. Run und Blob(s): sync_all ; Verzeichnis sync_dir
3. Edit { add_files[Run, Blob(s)], blob_live_delta (+bytes), space_flushed(space, max_stamp), redo{floor, floor_stamp, hwm=durable_stamp}, reserve }
   → Catalog append + sync_data                                                         ◄── Durability-Punkt
4. neue Version veröffentlichen (frozen entfernen, Dateien hinzufügen)
5. Redo-Segmente unterhalb redo_floor löschen (Graveyard)
```

**Warum Schritt 0:** Ein `Change` über Spaces A und B hat **einen** Frame. Würde A geflusht (Run dauerhaft), bevor der Frame dauerhaft ist, und ginge der Frame bei einem Absturz verloren (`Interval`/`Never`), wäre die Hälfte des `Change` dauerhaft und die andere weg: ein Verstoß gegen Z-1. `INV-FLUSH-1` verbietet das. Dasselbe gilt für die Flush-Barriere von `Erase` (§14.3) und für `checkpoint`.

Körper in Schlüsselreihenfolge im Blob erhalten **Leselokalität**: Ein Präfix-Scan über Volltexte liest zusammenhängende Bereiche. `redo_floor` wird nur angehoben, wenn **alle** Spaces die Segmente abdecken. Hinkt ein selten beschriebener Space hinterher und übersteigt das Redo-Volumen `max_redo_bytes`, erzwingt die Wartung dessen Flush (auch mit kleinem Run); `stats()` zeigt `redo_lag_bytes`.

### 12.2 Compaction (ein Verfahren: Leveled)

- **Planer = reine Funktion**, property-testbar. Die adaptive, kostenbasierte Planung des Altstands entfällt (SD-11).
- **Auslöser:** L0-Anzahl ≥ 4 ⇒ alle L0-Runs plus überlappende L1-Runs. Tiefere Level: Größe > `target(level)` (Faktor 10) ⇒ Run mit höchstem Score plus überlappende Runs des Zielniveaus. Ausgabe wird bei `target_run_bytes` (64 MiB) geteilt, bevorzugt an Präfixgrenzen (`locality_prefix`, §12.6). Level ≥ 1 sind je Space überlappungsfrei (Property-Test).
- **Aufbewahrung (`retain.rs`, rein):** je Schlüssel alle Versionen mit Stamp > `floor` behalten, die neueste mit Stamp ≤ `floor` behalten, ältere verwerfen. Löschmarker (`Delete`) werden nur verworfen, wenn unterhalb kein Run den Schlüsselbereich überlappt (aus Catalog-Metadaten) **und** nichts unter der Untergrenze liegt. `RangeDelete`-Marker analog. `EraseRange`-Marker bleiben bis `erase_done` (§14).
- **`CompactionFilter` (SD-41, SD-48):**

```rust
pub trait CompactionFilter: Send + Sync {
    /// Nur für Versionen mit stamp <= floor_used. Rein: Ergebnis hängt nur von den Argumenten ab.
    fn decide(&self, key: &[u8], header: &[u8], stamp: Stamp, now_ms: u64) -> Decision;
}
pub enum Decision { Keep, Drop, DropBody { new_header: Option<Vec<u8>> } }   // DropBody = „Zoom-Down“
```

  Der Filter ist nur in Spaces mit `SpaceOptions::forgetting()` zulässig (Flag `16` in `spaces_add`); sonst `Error::Config`. Er ermöglicht Ablauf von Idempotenzschlüsseln, Arbeitsgedächtnis und Vergessen (Volltext weg, Zusammenfassung bleibt) ohne eigene Maschinerie. **Ehrliche Semantik (Korrektur gegenüber v2):** Vergessen ist eine **sichtbare Zustandsänderung**. Bestehende `View`s halten ihre `Version` (unveränderliche Dateien) und bleiben **unverändert**. **Neue** Sichten ab dem Installationszeitpunkt sehen die Entscheidung, auch `view_at(s)` mit `s ≥ floor`, und ein `Hold` schützt in `forgetting`-Spaces vergessene Daten **nicht**. Versionen mit Stamp > `floor_used` bleiben unberührt, `history` über Consumer-Stände ist daher exakt. Der Filter wird je Job mit einem einmal gelesenen `now_ms` aufgerufen, nimmt Bereichs- und Erase-Marker aus, läuft ohne gehaltene Sperre (P-10) und wird im Modelltest gegen das Referenzmodell mitgeprüft. Er ist kein Merge-Operator (SD-12), weil er nur entfernt oder verkürzt.
- **`DropBody`:** Der Eintrag bleibt mit `new_header` (oder altem Header) als `Put` ohne Körper erhalten; ein vorhandener `BlobRef` wird entfernt und seine Bytes im selben Edit vom Lebendstand abgebucht.
- **Extensions:** `Extension::merge` erhält die Überlebenden nach `retain` und Filter (§12.4).
- **Installation:** Ausgabe-Runs `sync_all` + `sync_dir`, dann **ein** Edit `{add_files, remove_files, blob_live_delta, reserve}`, dann neue Version, dann Eingabedateien als `obsolete` markieren (§16.5) und per `Arc`-Drop ins Graveyard.
- Verworfene Versionen mit `BlobRef` buchen ihre Bytes im selben Edit vom Lebendstand ab; ein Blob mit `live_bytes == 0` wird im selben Edit entfernt.

### 12.3 Blob-GC und Konsolidierung (SD-43)

**Blob-GC ist compaction-getrieben.** Runs sind unveränderlich und tragen den *Ort* eines Körpers; ihn zu verschieben heißt, den Run neu zu schreiben. Es gilt:

1. **Berechtigte Menge `E`:** Blobs mit `live_bytes / bytes < blob_gc_live_ratio` (0,4), oder mit Alter > `blob_gc_max_age` und `live_bytes < bytes`, oder **kleine Blobs**: Datei < `blob_min_bytes` (16 MiB), sobald die Summe der kleinen Blobs eines Spaces ≥ `blob_min_bytes` erreicht (**Korrektur gegenüber v2:** Flushes erzeugen je Space eine Blob-Datei in Memtable-Größe, also typisch ≤ 16 MiB; ohne Konsolidierung wüchse die Dateizahl unbegrenzt, weil vollständig lebende Blobs nie unter 0,4 fallen und `TableCache` unter Druck gerät).
2. Eine Compaction, die einen Eintrag mit `BlobRef ∈ E` verarbeitet, kopiert den Körper in die **neue Blob-Datei der Ausgabe** und setzt den `BlobRef` im Ausgabe-Run neu (neue `blob_no`, neuer Nonce, Stamp unverändert). Verschlüsselte Körper werden neu versiegelt (Domänenschlüssel nötig).
3. Werden `E`-Blobs von der regulären Compaction nicht erreicht, plant der Planer eine **Umschreib-Compaction** (gleiches Level, gleiche Schlüssel) über die Runs, deren `[blob_no_min, blob_no_max]` `E` abdeckt (Kandidaten aus Catalog-Metadaten, Bestätigung über die Blob-Liste im MetaBlock).
4. Ist der Domänenschlüssel eines Körpers **vernichtet** (`KeyProvider::domain_key → None`), wird der Körper nicht kopiert; der Eintrag erhält `BlobRef{blob_no = 0, domain}` („Körper vernichtet“).

Das Verfahren verändert nie einen Stamp; `history`, `Consumer` und `ResumeToken` bleiben unberührt. Es nutzt dieselben Claims, Edits und Absturzregeln wie jede Compaction.

**Graveyard.** `FileHandle`/`BlobHandle` führen ein `obsolete`-Flag. Beim Drop der letzten Referenz mit gesetztem Flag wird der Pfad in das Graveyard gelegt, **ohne E/A im Drop**. Die Wartung löscht per `Fs::remove_file`, danach `sync_dir`. Fehlschläge werden wiederholt und als Metrik gemeldet (B-9d des Altstands). Nach einem Absturz räumt die Verwaisten-Suche auf (`INV-CAT-4`).

### 12.4 Extensions (zusätzliche Index-Blöcke je Run)

```rust
pub trait Extension: Send + Sync {
    fn id(&self) -> u16;                                   // stabile Block-Art (≥ 0x1000 im Run)
    fn build(&self, entries: &mut dyn Iterator<Item = Entry>) -> io::Result<Vec<u8>>;
    fn merge(&self, inputs: &[&[u8]], survivors: &mut dyn Iterator<Item = Entry>) -> io::Result<Vec<u8>>;
}
```

Der Kern liefert **keine** Implementierung. Er garantiert: Der Block entsteht im selben Run-Bau (Flush und Compaction), steht im `DirectoryBlock`, wird mit dem Run atomar lebendig und mit dem Run gelöscht. `merge` muss die Überlebenden (nach `retain` und `CompactionFilter`) berücksichtigen, damit gelöschte Einträge nicht über den Index sichtbar bleiben. Aufrufe laufen ohne gehaltene Sperre und hinter `catch_unwind` (F-12). **Eignung:** Strukturen, die mit den Daten zerlegt und zusammengeführt werden (Postings, Bloom, Zähler). Die Schnittstelle gilt bis Phase F10 als **instabil** (A-10).

### 12.5 Segmente (SD-30)

Strukturen, die sich nicht mit den Runs zerlegen lassen (Vektorgraphen, Graph-Adjazenz, große Volltext-Segmente), liegen als `Segment`-Dateien (`FileKind::Segment(kind)`) mit eigenem Format in der Hand der oberen Schicht. Der Kern liefert Lebenszyklus (bauen → dauerhaft → Edit → sichtbar → Graveyard), Absturzsicherheit und Einbezug in `Erase`. `FileMeta.min_stamp/max_stamp` beschreiben den **Datenstand** des Segments. Ein Index liest `Segmente ∪ history(stamp_to, view.stamp]`; der Aufholzeiger ist ein `Consumer`.

| Index | Mechanismus | Begründung |
|---|---|---|
| Postings, Bloom, Zähler | `Extension` je Run | billig, zerlegt mit den Daten |
| Volltext (BM25) | Extension **oder** Segment | beides tragfähig; Start mit Extension |
| Vektor (ANN) | **Segment** | Graphbau teuer, Lokalität folgt Cluster, nicht Schlüssel |
| Graph (Adjazenz, PPR) | Space (Kanten); Segment nur für Mehrschritt-Abfragen | Adjazenz-Scan genügt zunächst |

**`Erase`-Fähigkeit:** Ein Segment-Format muss `rewrite_without(range)` anbieten. Ohne diese Methode verwirft der Kern bei Überlappung das **ganze** Segment und verlangt den Neuaufbau durch den Besitzer (§14.3).

### 12.6 Lokalität

Die Reihenfolge ist die Schlüsselreihenfolge. Wer zusammen gelesen wird, liegt zusammen, wenn die obere Schicht dem Schlüssel einen Cluster-Präfix voranstellt. `locality_prefix(n)` lässt die Compaction Ausgaben bevorzugt an Präfixgrenzen teilen (±25 %). Ein Umordnen nach Zugriffsmustern ist eine **Re-Key-Operation der oberen Schicht** (Hysterese, Lastfenster) auf Basis der `AccessSink`-Spur; der Kern ordnet **nie** nach Zugriff um (SD-26).

### 12.7 Kalter Speicher (Phase F10)

`FileMeta.tier ∈ {Local, Cold}` gilt für **alle** Dateiarten (Run, Blob, Segment). Ein `ColdTier`-Port wird als `File`-Implementierung hinter `TableCache` eingebunden; Dateien sind unveränderlich und deshalb ohne Koordination hochladbar. **Folge für die Löschung:** E6 schließt `ColdTier::delete` ein; `Physical` ist für kalte Dateien nur so weit zusagbar, wie der Port Löschung garantiert; belastbar bleibt `Cryptographic` (A-11).

### 12.8 Betrieb

- **`verify(Online | Offline)`.** *Online:* Catalog ↔ Dateien (Existenz, Länge, Footer), Stichproben-Prüfsummen, `blob_live_delta`-Bilanz gegen die Runs, Consumer-/Hold-Stände. *Offline:* alle Blöcke und Records, jeder `BlobRef` löst auf einen Record gleichen Schlüssels/Stamps auf, jede Blob-Datei ist von mindestens einem Run referenziert oder Hole, Redo-Kette und Siegel, `redo_hwm`. Befund je Datei, nie nur Ja/Nein.
- **`checkpoint(dest, Copy | Hardlink)`.** Eine `View` plus `Hold` legt den Stand fest; Flush-Barriere mit `INV-FLUSH-1`; die Wartung beansprucht die Dateien (Claims); Dateien sind unveränderlich und werden kopiert, Redo bis zum Stand und ein Catalog-Snapshot werden geschrieben. Das Ergebnis ist ein normal öffenbares Verzeichnis. `Hardlink` ist Opt-in mit Warnung (ein Hardlink hält Dateien nach einem Unlink am Leben und verletzt `INV-ERA-2` für die Kopie).
- **`open_readonly`.** Geteilte Sperre, keine Heilung, keine Wartung; nur `Rd`-Handles; geeignet für `verify`, Sicherung, Inspektion.
- **`stats()` und Metriken** (über `Metrics`): `redo_lag_bytes`, `held_bytes`, `blob_live_ratio`, Schreib- und Platzverstärkung, Zustand der Lanes, Claims, `stall`-Zeiten, Cache-Trefferquoten, Gegendruckstufe, `health`. Metriken enthalten keine Schlüssel/Werte (P-12).
- **Lebenszyklus.** `hibernate()` flusht, gibt Memtable und Caches frei und hält nur Catalog-Zustand und `LOCK`. `close()` flusht optional, siegelt das Redo-Segment, gibt die Sperre frei.
- **Formatentwicklung.** Neue Fähigkeiten kommen über optionale Feature-Bits; ein Pflicht-Bit verweigert alten Lesern das Öffnen. Eine neue Dateiversion ist ein eigenes Migrationswerkzeug (SD-04).

---

## 13. Recovery

### 13.1 Ablauf beim Öffnen (deterministisch, 11 Schritte)

```
 1. Fs::lock(LOCK): Exclusive (lesend-schreibend) oder Shared (open_readonly: heilt nichts, startet keine Wartung); scheitert ⇒ Err(Locked)
 2. FORMAT lesen; integrity_mode gegen Config prüfen (F-15) ⇒ Err(Unsupported); unbekanntes Pflicht-Feature ⇒ Err(Unsupported)
    Authenticated/Encrypted: KEK holen, KEYS öffnen (root_key, Space-DEKs), FORMAT.chk prüfen
 3. CURRENT ⇒ Catalog-Datei; Records lesen (letzter Torn-Record verworfen, sonst Corrupt); Zustand aufbauen
    (Spaces, FileMeta, Blob-Tabelle, Consumer, flushed_stamps, redo{floor,floor_stamp,hwm}, reserve, pending erases, erased_ranges, domain_retired)
    keys_generation ≥ keys_generation_min prüfen ⇒ sonst Err(Rolledback); optional RollbackAnchor::latest vergleichen
 4. *.building.* löschen; Dateien, die der Catalog nicht nennt (Runs, Blobs inkl. nicht aktueller Generationen, Segmente,
    Redo-Segmente mit segment_id < redo_floor), löschen — Reserve-Segmente (≥ redo_floor) bleiben (INV-LOG-7)
 5. Vom Catalog genannte Datei fehlt ⇒ Err(Corrupt{MissingFile{kind,id}}) (fail-closed, kein Raten)
 6. Redo-Segmente ab redo_floor in Id-Reihenfolge lesen:
      gültige Frames anwenden, je Op nur wenn stamp > space.flushed_stamp
         (domänenversiegelte Körper entsiegeln; Domänenschlüssel vernichtet oder in domain_retired ⇒ Körper verwerfen, Eintrag „vernichtet“)
      EraseRange-Op ⇒ Marker in die Memtable und Auftrag in die pending-Menge, falls kein erase_done
      Defekt in gesiegeltem Segment ⇒ Err(Corrupt)                                   (INV-LOG-6)
      Ungesiegeltes Segment mit nachfolgendem Segment mit Frames ⇒ Err(Corrupt)
      letztes ungesiegeltes Segment: auf letztes gültiges Frame abschneiden, sync_data, SIEGELN, sync_data
      INV-LOG-8: letzter gültiger Stamp (bzw. redo_floor_stamp) < redo_hwm ⇒ Err(Corrupt{RedoTruncated})
 7. next_stamp = max(letzter Redo-Stamp, max flushed_stamp, max Run-Stamp) + 1; visible_stamp = next_stamp − 1; durable_stamp = visible_stamp
 8. ID-Vergabe beginnt bei den Hochwassermarken aus `reserve` (INV-KEY-2); vorallokiertes Reservesegment übernehmen oder neues Segment
    anlegen (nie anhängen, INV-LOG-4); optional Catalog-Rollover
 9. Consumer prüfen: Rückstand > max_lag_bytes ⇒ als verworfen markieren; Untergrenze berechnen
10. Pending Erases und ausstehende Blob-Sweeps (Catalog ∪ Erase-Frames im Redo ohne erase_done) in die urgent-Lane (Roll-forward, §14.4)
11. Wartung starten (Vorallokation, ID-Reservierung, Cleanup)
```

Die Reihenfolge 3→4 ist wichtig: **erst** den Catalog als Wahrheit lesen, **dann** aufräumen. Runs und Blobs werden **nicht** geöffnet (Metadaten stehen im Catalog, H-S5). Im Shared-Modus entfallen die Schritte 4, 6 (Heilen/Siegeln), 8, 10 und 11; ein Torn-Tail wird als Ende des Logs gelesen.

### 13.2 Absturzmatrix (Soll-Zustand je Unterbrechungspunkt)

| Unterbrechung | Zustand auf Platte | Ergebnis nach Neustart |
|---|---|---|
| vor Redo-`write` | nichts | Change nicht vorhanden, nie quittiert |
| mitten im `write` / vor `fsync` | halbes oder ungeflushtes Frame | Frame wird abgeschnitten oder fehlt, nie quittiert |
| nach `fsync`, vor Quittung | Frame dauerhaft | Change vorhanden (zulässig) |
| `ENOSPC` nach Teilschreiben, Abschneiden gelang | Segment endet beim letzten guten Frame | Neustart unauffällig |
| `ENOSPC`, Abschneiden gescheitert → Poison, Absturz | Torn-Tail im letzten Segment | Heilen + Siegeln (§13.1 Schritt 6) |
| Recovery-Heilen: nach Abschneiden, vor Siegel | Segment kurz, ungesiegelt | nächste Recovery wiederholt Heilen + Siegeln (idempotent) |
| Siegel geschrieben, nächstes Segment fehlt | Segment gesiegelt, kein Nachfolger | neues Segment wird angelegt |
| Reservesegment: Absturz beim Vorallokieren | `.building.*` | gelöscht, Reserve wird neu angelegt |
| Reservesegment fertig, noch unbenutzt | Segment ohne Frames, `id ≥ redo_floor` | wird übernommen (INV-LOG-7), nicht gelöscht |
| ID-Reservierung: Absturz vor Edit | keine Datei mit ungesicherter ID | IDs wurden nie benutzt |
| ID benutzt (Datei angelegt), Absturz vor Flush-Edit | verwaiste Datei mit ID < Hochwassermarke | Datei gelöscht; ID wird **nie** wieder vergeben (INV-KEY-2) |
| Flush (Interval/Never): vor Redo-Sync (Schritt 0) | Frame evtl. nicht dauerhaft, kein Run | nichts geflusht; Atomarität über Spaces bleibt erhalten (INV-FLUSH-1) |
| Flush: nach Run/Blob-`sync`, vor Edit | verwaiste Run- und Blob-Dateien | Verwaisten-Suche löscht, Memtable aus Redo wiederhergestellt |
| Flush: nach Edit, vor Version | Dateien im Catalog | lebendig, Redo-Frames mit Stamp ≤ `flushed_stamp` übersprungen |
| Compaction/Blob-GC: nach Ausgabe, vor Edit | verwaiste Ausgaben | gelöscht, Eingaben unverändert |
| Compaction: nach Edit, vor Unlink | Eingaben verwaist | gelöscht (Verwaisten-Suche) |
| Blob-Sweep: neue Generation gebaut, vor Edit | zusätzliche Generationsdatei | gelöscht (nicht die aktuelle Generation) |
| Blob-Sweep: nach Edit, vor Unlink | alte Generation | gelöscht |
| Catalog-Rollover: nach neuer Datei, vor `CURRENT` | alte `CURRENT` gültig | neue Datei verwaist, gelöscht |
| Catalog-Rollover: nach `CURRENT`, vor Löschen der alten Datei | beide vorhanden | alte Datei verwaist, gelöscht |
| `Consumer::advance`: Absturz | Edit atomar | alter oder neuer Stand, nie dazwischen |
| `KEYS`-Neuschreiben (KEK-Rotation, DEK-Entfernung): Absturz | `write_atomic` | alter oder neuer `KEYS`, nie halb |
| Redo-Schwanz nach Edit verloren (Lost Write) | Redo endet unter `redo_hwm` | `Corrupt{RedoTruncated}` statt stillem Verlust (INV-LOG-8) |
| `fsync`-Fehler zur Laufzeit | unbestimmt | Instanz vergiftet, kein Retry, Neustart führt Recovery aus |

Diese Tabelle wird zu einem **Test** (§19.2): `FaultFs` bricht an jedem E/A-Index ab und prüft den Soll-Zustand.

---

## 14. Löschung (`Erase`)

Antwort auf „atomare, nachweisbare physische Löschung“ (Z-3). Sie beruht auf vier strukturellen Maßnahmen: **(1)** Spaces sind physisch getrennt (Löschgrenze = Verzeichnis, Blobs eingeschlossen); **(2)** der Catalog kennt Schlüsselbereich, Stamp-Bereich und `blob_no`-Bereich jeder Datei, die Planung öffnet keine Datei; **(3)** Dateien werden ausschließlich **nach** einem dauerhaften Edit gelöscht (P-3); **(4)** sensible Körper sind je `KeyDomain` versiegelt, sodass Schlüsselvernichtung den Körper auch in Redo und Blob unlesbar macht.

### 14.1 Was „atomar“, „physisch“ und „nachweisbar“ heißt

- **Logisch atomar:** Ab Stamp `S` sieht keine View mit Stamp ≥ `S` etwas aus dem Löschbereich, für alle Schlüssel zugleich. Der Marker ist ein Op (`EraseRange`) in einem normalen Redo-Frame.
- **Physisch atomar:** Es gibt **genau einen** Durability-Punkt (E5, ein Catalog-Edit), vor dem die alten Dateien lebendig und nach dem sie unreferenziert sind. Ein Absturz davor setzt den Vorgang fort, ein Absturz danach löscht die Dateien (Roll-forward, nie Rückabwicklung, `INV-ERA-4`).
- **Nachweisbar:** Eine `ErasureReceipt` wird erst nach `remove_file` **und** `sync_dir` ausgestellt.

### 14.2 Stufen und ehrliche Garantien

|Stufe|Verfahren|Schützt gegen|Schützt **nicht** gegen|
|---|---|---|---|
|`Logical`|Marker|Lesen über die API|Dateiinhalt|
|`Physical` (Standard)|Runs ohne den Bereich neu schreiben, Blob-Sweep, Edit, Unlink, `sync_dir`, Redo-Segmente entfernen|Lesen der Dateien im Verzeichnis; Wiederauftauchen nach Absturz|Reste im Dateisystem-Journal, Flash-Wear-Leveling, externe Snapshots/Backups, Hardlinks|
|`Scrubbed`|wie `Physical`, vor dem Unlink Nullen überschreiben + `sync_data`|einfache Wiederherstellung auf klassischen Datenträgern|Copy-on-Write-Dateisysteme, SSDs (Best-Effort)|
|`Cryptographic`|Schlüssel der `KeyDomain` bzw. des Spaces vernichten, **bevor** Dateien gelöscht werden|alle Reste, auch Backups ohne Schlüssel, Flash-Reste und Redo-Frames (Körper sind domänenversiegelt)|Kopien des Schlüssels; Metadaten unter dem Space-DEK|

**Konsequenz:** Echte Unwiederbringlichkeit auf modernen Datenträgern liefert nur `Cryptographic`. Sensible Körper gehören deshalb in `domain_sealed`-Spaces (§15.3). `Physical` minimiert Rückstände, mehr wird nicht versprochen.

### 14.3 Protokoll `Erase::range(space, range)` mit Stamp `S`

```
E0  Vorprüfung: Space existiert, Recht Adm (append-only: Erase::admin()), Bereich gültig;
    Views/Holds/Consumer mit Stamp < S gemäß ViewPolicy / HoldPolicy (§14.5)
E1  Redo-Frame mit Op EraseRange(space, range) (normaler apply-Pfad, Durability::Always erzwungen)   ◄── LOGISCHER Punkt, Stamp S
    Gleichzeitig (selber Edit bei Flush, selbe Memtable bei Replay): Auftrag in die pending-Menge
E2  urgent-Lane belegen. Claims auf betroffene Dateien (laufende Compaction wird abgebrochen, §12.0).
    Flush-Barriere: alle Memtables aller Spaces bis S flushen (INV-FLUSH-1), redo_floor anheben,
    Redo-Segmente mit max_stamp ≤ S ins Graveyard. Erase::batch ⇒ EINE Barriere für alle Aufträge der Gruppe.
E3  Dateien wählen (nur Catalog-Metadaten): Runs des Spaces mit überlappendem [min_key, max_key];
    Blobs über BlobRef-Bereiche (blob_no_min/max, MetaBlock-Liste)
E4  Je Run streamen, alle Einträge im Bereich mit Stamp ≤ S verwerfen (Holds je HoldPolicy), Ausgabe-Run bauen
    (sync_all), sync_dir. Vollständig überdeckte Runs erzeugen keine Ausgabe. EraseRange-Marker wandern in den Ausgabe-Run.
E4b Blob-Sweep: je betroffenem Blob neue Generation g+1 mit GLEICHEN Offsets: gelöschte Records → Hole (Nullen, unverschlüsselt,
    INV-BLOB-3), übrige byte-identisch; sync_all, sync_dir
E4c Segmente: überlappende Segmente via rewrite_without(range) neu bauen, sonst verwerfen (Besitzer baut neu)
E5  EIN Edit { add_files, remove_files, blob_replace, blob_live_delta, erase_done(S), erased_ranges_add(space, range, S) }
    → Catalog append + sync_data                                                                   ◄── PHYSISCHER Punkt
E6  Neue Version veröffentlichen; BlockCache/TableCache invalidieren; alte Runs und Blob-Generationen ins Graveyard;
    nach Rückgabe aller Referenzen (ViewPolicy): [Scrubbed: überschreiben + sync] → remove_file → sync_dir;
    kalte Dateien: ColdTier::delete (§12.7)
E7  Edit { erase_swept(S) } (nicht kritisch). ErasureReceipt{ id=S, level, runs_rewritten, runs_removed, blobs_swept,
    hole_bytes, redo_segments_removed, segments_rebuilt, bytes_freed, completed_ms } zurückgeben
```

- **Kryptografisch je Space:** E3–E4c entfallen; E5 enthält `spaces_drop`; der Space-DEK wird **vor** E5 per `write_atomic(KEYS)` entfernt (`INV-ERA-3`).
- **Kryptografisch je Domäne (`Erase::domain(d)`):** Der Aufrufer vernichtet `d` im `KeyProvider` (außerhalb des Crates). Der Kern **verweigert** den Auftrag, solange `domain_key(d)` noch einen Schlüssel liefert (`KeyStillPresent`), leert den Domänenschlüssel-Cache, schreibt `domain_retired(d, S)` per Edit und überlässt die Platzfreigabe der Blob-GC (§12.3 Punkt 4). Keine Flush-Barriere, kein Run-Umbau (H-S14).
- **`drop_space`:** Schnellpfad: Schlüssel vernichten (falls verschlüsselt), ein Edit `{spaces_drop, remove_files[alle]}`, Graveyard, `remove_dir` (H-S6).
- Schreibvorgänge nach `S` im selben Bereich bleiben erlaubt; E4 verwirft nur Stamp ≤ `S`.
- **Kosten:** O(Bytes der berührten Runs) + O(Bytes der berührten Blobs, je ≤ 64 MiB). Der Sweep ist der teuerste Teil und der Grund, für große oder häufige Löschungen `Cryptographic` zu bevorzugen.

### 14.4 Recovery der Löschung (Roll-forward)

Offen = `Catalog.erase_pending ∪ {EraseRange-Frames im Redo ohne erase_done} − erase_done` (§13.1 Schritt 10). Sie werden in die `urgent`-Lane gelegt und ab E2 fortgesetzt. E3–E5 sind **idempotent** (Auswahl aus dem aktuellen Catalog; ein bereits ausgeführtes E5 ist an `erase_done` erkennbar). E6/E7 sind idempotent über die Verwaisten-Suche (alte Run-Dateien und nicht aktuelle Blob-Generationen sind verwaist). Ein Flush, der einen Erase-Marker in einen Run schreibt, trägt `erase_pending_add` im **selben** Edit.

### 14.5 Nebenläufigkeit mit Views, Holds, Consumern

|Policy|Verhalten|
|---|---|
|`ViewPolicy::Wait(timeout)` (Standard)|Das Löschen der Dateien wartet, bis alle Views mit Verweis auf die alten Dateien zurückgegeben sind. Nach Timeout `Err(Blocked{views})`; der Vorgang bleibt in E6 offen und wird wiederholt (der logische Teil gilt bereits).|
|`ViewPolicy::Revoke`|Betroffene Views werden widerrufen (`Err(Revoked)` beim nächsten Zugriff), geben ihre Version frei, danach Löschen.|
|`HoldPolicy::Fail` (Standard)|Hold oder Consumer mit Stamp < `S` über dem Bereich ⇒ `Err(HoldBlocked{holds})`, **vor** E1 (nichts geschrieben).|
|`HoldPolicy::Override`|Hold wird für den Bereich ungültig, Consumer verworfen (Datenschutz hat Vorrang).|

### 14.6 Kein stilles Fälschen der Historie

Der Catalog führt je Space `erased_ranges: [(range, S)]`. Liest eine View mit Stamp < `S` in einen dort verzeichneten Bereich, antwortet sie `Err(Erased{range, erased_at: S})`; es gibt weder Fallback noch „leer“ (`INV-ERA-5`). Einträge werden entfernt (`erased_ranges_remove`), sobald `retention_floor ≥ S`.

### 14.7 Invarianten und Prüfwerkzeuge

`INV-ERA-1…7` stehen in §18. Prüfwerkzeuge:

1. **`Storage::verify_erasure(id)`:** prüft `INV-ERA-2/6` anhand der Catalog-Metadaten und Stichprobenscan lebender, überlappender Dateien; Befund je Datei.
2. **`ErasureOracle` (Feature `testing`):** schreibt Schlüssel/Werte mit eindeutigem Muster (Header **und** Körper; mit/ohne Domäne; `Plain`/`Encrypted`), führt `Erase` aus und durchsucht **alle Bytes aller Dateien** (Runs, Blobs aller Generationen, Redo, `.building`, Segmente, `MemFs`-Zustand inkl. nicht synchronisierter Seiten) nach dem Muster. `Physical` ⇒ 0 Treffer. `Cryptographic` ⇒ Muster auch ohne Schlüsselzugriff nie im Klartext, mit vernichtetem Schlüssel nicht rekonstruierbar. Für Vektor-Segmente zusätzlich Rückrechnung (Embedding des gelöschten Textes darf nicht mehr als Nachbar auftauchen).
3. **Absturzmatrix über E1–E7 inkl. E4b/E4c:** `FaultFs` bricht an jedem E/A-Index ab; nach jedem Neustart gelten `INV-ERA-1…7`; fortgesetzte Vorgänge enden mit 0 Orakel-Treffern.
4. **Backup-Hinweis:** `checkpoint` ist standardmäßig `Copy`; `Hardlink` ist Opt-in mit Warnung (§12.8).

### 14.8 Grenzen (offen benannt)

Append-only Spaces lassen `Erase` nur mit `Erase::admin()` zu; das steht im Receipt. Kalte Dateien unterliegen den Garantien des `ColdTier`-Ports. Speicherinhalt, Auslagerungsdatei, Core-Dumps werden nicht kontrolliert (A-5, §15.7). Externe Snapshots, Dateisystem-Journal und Flash-Wear-Leveling siehe §14.2. Die orchestrierende Ebene (Saga, signierte Berichte) liegt **über** diesem Primitiv (NZ-14); `ErasureReceipt` ist dafür serialisierbar und stabil.

---

## 15. Sicherheit und Verschlüsselung

Kryptografische **Primitive** (AEAD, MAC, KDF, Zufall) kommen ausschließlich über Ports (§4); das **Protokoll** (Schlüsselhierarchie, Nonces, AAD, Reihenfolgen) lebt in `keys/` und ist hier normativ (SD-36). Der Kern hat keine Krypto-Abhängigkeit (Z-4). Empfohlene Port-Implementierungen (außerhalb des Kerns): BLAKE3 keyed für `Mac`/`Kdf`, ChaCha20-Poly1305 oder AES-256-GCM für `Cipher`. **Eigene Kryptografie wird nicht geschrieben.**

### 15.1 Modi (SD-33)

|Modus|Schutz|Verfahren|Gegen Angreifer|
|---|---|---|---|
|`Plain`|Zufallsfehler|CRC32C, Redo-Kette|A-1 nur über Dateirechte; **nicht** A-2/A-3|
|`Authenticated`|Integrität ohne Vertraulichkeit|`Mac16` mit AAD (§9.1)|A-3 (Manipulation)|
|`Encrypted`|Vertraulichkeit + Integrität|`Cipher` (AEAD), AAD-Bindung|A-2, A-3|

`Authenticated` deckt „lesbar, aber nicht unbemerkt veränderbar“ ab (Audit-Protokolle, append-only Spaces). Der Modus steht in `FORMAT`, ist unveränderlich und wird beim Öffnen gegen `Config::integrity` geprüft (F-15, §9.2).

### 15.2 Schlüsselhierarchie, Ableitung und Nonces

```
KEK (KeyProvider::kek, extern, 32 B)
 └─ KEYS-Schlüssel = Kdf(KEK, "ctxs keys v1")                       umhüllt NUR die Datei KEYS (§9.3)
KEYS enthält:
 ├─ root_key (zufällig, einmalig)
 │    ├─ Redo-Schlüssel    = Kdf(root_key, "ctxs redo v1"   ‖ segment_id u32)
 │    ├─ Catalog-Schlüssel = Kdf(root_key, "ctxs cat v1"    ‖ catalog_no u64)
 │    ├─ FORMAT-MAC-Schl.  = Kdf(root_key, "ctxs format v1")
 │    └─ Token-Schlüssel   = Kdf(root_key, "ctxs token v1")        (ResumeToken-MAC)
 └─ Space-DEK (zufällig, je Space, dek_gen)
      ├─ Run-Schlüssel    = Kdf(DEK, "ctxs run v1"  ‖ key_epoch u32 ‖ run_id u64)
      └─ Blob-Schlüssel   = Kdf(DEK, "ctxs blob v1" ‖ key_epoch u32 ‖ blob_no u64)
      └─ Segment-Schlüssel = Kdf(DEK, "ctxs seg v1" ‖ key_epoch u32 ‖ segment_kind u16 ‖ segment_id u64)
Domänenschlüssel: KeyProvider::domain_key(d) → Option<Secret32>   NICHT in KEYS; None = vernichtet
```

**KEK-Rotation ist O(1):** `KEYS` mit neuem KEK-Schlüssel und neuem `nonce` per `write_atomic` ersetzen, `keys_generation` hochzählen, `keys_generation_min` per Edit anheben. Keine Datendatei wird angefasst (SD-36).
**Schlüsselepoche (SD-63).** Ein beschreibbarer Klon (`checkpoint`, Test-Klon) setzt Dateiname-IDs ab derselben Hochwassermarke fort; ohne Gegenmaßnahme würden Original und Klon unter gleichem DEK, gleicher `run_id` und gleichem Nonce **verschiedene** Inhalte versiegeln. Deshalb vergibt `checkpoint` dem Ziel eine neue `instance_id`, einen **neuen** zufälligen DEK je Space und `key_epoch + 1`; ältere DEKs bleiben in `KEYS`, damit übernommene Dateien lesbar bleiben. Neue Dateien nutzen stets den DEK der höchsten Epoche. Dasselbe gilt für Domänen-Körper: Nonce `0x03`/`0x05` tragen `segment_id`/`blob_no`, die im Klon aus dessen eigener Hochwassermarke stammen; der Klon erhält zusätzlich einen disjunkten ID-Bereich (`id_hwm += 2^40`).

**Nonces (12 Byte, zentral in `keys/nonce.rs`; kein anderes Modul bildet Nonces):**

|Verwendung|Schlüssel|Nonce|Eindeutig weil|
|---|---|---|---|
|Run-Block|Run-Schlüssel|`0x01 ‖ block_offset u64 ‖ 0u24`|Schlüssel je `run_id`, `run_id` aus Hochwassermarke (`INV-KEY-2`); Offset je Run eindeutig|
|Redo-Frame|Redo-Schlüssel|`0x02 ‖ frame_index u32 ‖ 0u56`|Schlüssel je `segment_id`|
|Redo-Op-Körper|Domänenschlüssel|`0x03 ‖ segment_id u32 ‖ frame_index u32 ‖ op_index u24`|Position im Log eindeutig; `segment_id` nie wiederverwendet|
|Blob-Record|Blob-Schlüssel|`0x04 ‖ offset u64 ‖ 0u24`|Schlüssel je `blob_no`; `blob_no` nie wiederverwendet|
|Blob-Körper unter Domänenschlüssel|Domänenschlüssel|`0x05 ‖ blob_no u56 ‖ offset u32`|`blob_no` nie wiederverwendet; GC vergibt neue `blob_no`|
|Catalog-Record|Catalog-Schlüssel|`record_index u64 ‖ 0u32`|Schlüssel je `catalog_no`|
|`KEYS`|KEYS-Schlüssel|`nonce` aus `Entropy`|zufällig je Schreibvorgang, Schreibvorgänge selten|

> **Erratum E-6 (Nonce-Präfix):** §9.4 und §9.6 nennen für Blob-Körper unter dem Domänenschlüssel das Präfix `0x04`. Normativ ist `0x05` (oben), weil Blob-Record (Blob-Schlüssel) und Domänen-Körper in `domain_sealed` + `Encrypted` sonst dasselbe Nonce-Muster unter verschiedenen Schlüsseln nutzten. Das ist harmlos, aber unnötig verwechselbar; die Trennung macht die Eindeutigkeit **je Präfix** statt **je Schlüssel** beweisbar. Der Golden in Anhang A.6 folgt dieser Tabelle.

`run_id`, `blob_no`, `tmp_id` < 2^56, `segment_id` < 2^32; Überlauf ⇒ `Exhausted` (`INV-KEY-2`). Eine Blob-Neufassung gleicher Offsets (Sweep) übernimmt Records **byte-identisch**; ein GC-Umzug vergibt eine neue `blob_no`. Ein Property-Test erzeugt über Absturzmatrix und Wartung alle `(Schlüssel, Nonce)`-Paare und verlangt Eindeutigkeit (`INV-KEY-1`).

### 15.3 Domänenschlüssel: kryptografisches Löschen je Subjekt (SD-42)

Eine sortierte LSM mischt viele Subjekte in einem Block; Block-Verschlüsselung je Subjekt würde die Schlüsselordnung zerstören. Lösung auf **Körper-Ebene:**

- Schlüssel und Header bleiben unter dem Space-DEK. Ranking, Filter und Gewichte laufen ohne Domänenschlüssel; Metadaten (Provenienz) überleben eine Schlüsselvernichtung.
- Der Körper eines `domain_sealed`-Space wird bereits im Committer (Schritt 5) mit dem Domänenschlüssel versiegelt, **bevor** er in den Redo-Frame geht. Die Memtable hält Klartext (A-5); der Flush versiegelt mit Blob-Nonce unter dem Domänenschlüssel neu.
- Wird der Domänenschlüssel vernichtet, sind Körper in Redo **und** Blob ohne Flush-Barriere und ohne Run-Umbau unlesbar (`INV-ERA-7`); die Blob-GC gibt den Platz frei.
- Der Domänenschlüssel-Cache (TTL, begrenzt) wird bei `Erase::domain(d)` sofort geleert. Der Kern speichert **keine** Domänenschlüssel; ein Schlüsselbund mit Millionen Subjekten ist Sache des `KeyProvider`.
- **Offen (O-2):** Aus einem Körper abgeleitete Embeddings und Postings enthalten Rückstände. Sie liegen in Segmenten je Löschdomäne oder werden über `rewrite_without` neu gebaut.

### 15.4 Typisierte Zugriffsrechte (SD-34)

`Space<R>` mit Marken `Rd < Ap < Wr < Adm`, versiegelte Traits, nur Abschwächung (`read_only()`, `append_only()`), kein Rückweg. `AdminCap` ist je `Storage` genau einmal abrufbar. Dynamische, delegierbare Rechte (Macaroons, TTL) bleiben der oberen Schicht überlassen. `open_readonly` liefert nur `Rd`. **Grenze:** Das ist ein Übersetzungsfehler-Schutz gegen A-4 _innerhalb_ einer vertrauenswürdig kompilierten Anwendung, kein Prozess-Sandbox.

### 15.5 Rollback-Anker (F-10)

Authentisierte Blöcke schützen nicht vor dem Zurückspielen eines älteren, in sich stimmigen Gesamtstands. Im Modus `Authenticated/Encrypted` ist jeder Catalog-Record Glied einer Kette; `head_mac = Kdf(cat_key, [catalog_no, record_index, chk])` geht nach jedem dauerhaften Edit (ratenbegrenzt) an `RollbackAnchor::record`. Beim Öffnen vergleicht der Kern `(Generation, Kopf)` mit `latest()` ⇒ `Err(Rolledback{expected, found})`. Zusätzlich fällt ein alter `KEYS` über `keys_generation_min` auf. Das Fenster zwischen zwei `record`-Aufrufen bleibt Restrisiko. Ohne Anker ist F-10 nicht erkennbar (dokumentiert).

### 15.6 Dateisystem-Rechte

`Perm::Private` ⇒ `0700`/`0600`, **explizit gesetzt, unabhängig von der `umask`**, für `FORMAT`, `KEYS`, `CURRENT`, Catalog, Redo, Runs, Blobs, Segmente, `.building.*`, `LOCK`. Das Wurzelverzeichnis gilt als vertrauenswürdig (A-1). `RealFs` prüft vor dem Öffnen per `symlink_metadata`, dass das Wurzelverzeichnis kein Symlink ist; vollständige Härtung (`O_NOFOLLOW`, `openat`) gehört in eine alternative `Fs`-Implementierung. Windows: ACL-Härtung ist in Version 1 nicht zugesagt (NZ-6, A-02).

### 15.7 Geheimnisse und Informationslecks

- **`Secret32`:** kein `Debug`/`Display`/`Clone`/`Serialize`; `Drop` überschreibt per `fill(0)` + `black_box` (**best-effort**, ohne `unsafe` ist kein volatiler Schreibvorgang garantierbar; steht in der Doku).
- Vergleiche von Tags laufen in `Mac::verify16` (konstante Zeit, Port-Vertrag).
- **Keine Daten in Fehlern, Logs, Metriken:** `hint()` enthält nur Typen, Zahlen, Space-/Consumer-Namen und opake numerische Kennungen, **nie** Schlüssel-/Wertbytes oder Domänen-IDs im Klartext. Das **Lecks-Orakel** prüft alle in der Suite erzeugten Fehlertexte, Metriknamen und Logzeilen auf Muster der Testdaten (0 Treffer).
- **Kein Dedup, kein Inhaltshash** im Kern (NZ-8): keine Seitenkanal-Klasse „existiert dieser Text bei jemand anderem?“.
- Die in AAD gebundenen Schlüssel-Bindungen nutzen den Space-DEK als MAC-Schlüssel, nie unverschlüsselte Klartext-Hashes.

### 15.8 Restrisiken (offen)

|Risiko|Stand|
|---|---|
|Rollback ganzer Stände ohne Anker|nicht erkennbar (§15.5)|
|Klartext im Prozessspeicher, Swap, Core-Dumps|nicht kontrolliert (A-5)|
|Schlüsselmaterial im `KeyProvider`|Sache der einbettenden Anwendung|
|Seitenkanäle (Größe, Zeit, `op_count` im Klartext bei `Encrypted`)|keine Gegenmaßnahme; Padding wäre Sache der oberen Schicht|
|`Plain` gegen Manipulation|nicht geschützt; `FORMAT.integrity_mode` und Doku weisen darauf hin|
|Löschen ganzer Dateien durch Angreifer ohne Schlüssel|erkannt über `MissingFile`/Catalog (fail-closed), aber Verfügbarkeit nicht geschützt|

---

## 16. Nebenläufigkeit: Protokolle und Beweise (Z-14)

Alle Synchronisationsstücke sind so klein, dass `loom` sie vollständig erkundet (§19). Die Beweise sind schriftlich, damit Review und Modell dieselbe Aussage prüfen.

### 16.1 Sperrränge

Sperren werden **nur in aufsteigender Rangfolge** erworben (`INV-LOCK-1`), jede höchstens einmal je Thread. `src/sync.rs` kapselt `Mutex`/`RwLock` in `Ranked<T, const RANK: u8>`; unter `cfg(debug_assertions)` prüft ein thread-lokaler Rangstack jeden Erwerb.

|Rang|Sperre|Hält über E/A?|Typische Haltezeit|
|---|---|---|---|
|1|`committer`|**ja (einzige Ausnahme, P-9)**|ms (ein Gruppen-`fsync`)|
|2|`intake`|nein|ns|
|3|`state` (`RwLock<Arc<Version>>`)|nein|ns (Arc klonen/austauschen)|
|4|Memtable-`RwLock` (je Memtable)|nein|≤ 64 Ops je Schreibsperre|
|5|`registry` (Views, Holds, Consumer, Floor, Claims)|nein|µs|
|6|`maint` (je Lane, + Condvar)|nein|ns|
|7|`notify` (Listener-Warteschlange)|nein|ns|

**Blatt-Regel:** Ränge 5–7 sind Blätter: unter ihnen wird keine weitere Sperre erworben und kein Aufrufer-Code ausgeführt. Die geteilten Cache- und Governor-Sperren (sharded, kurz) sind ebenfalls Blätter und werden **nie über E/A gehalten**: Cache-Fehlgriffe laden ohne Sperre und fügen danach ein (zwei Lader desselben Blocks sind zulässig, einer gewinnt).

**Deadlock-Freiheit (Beweis).** Jeder Thread erwirbt Sperren nur in streng aufsteigender Rangfolge, Blätter erwerben nichts. Ein Wartezyklus `T1 hält a, wartet auf b; T2 hält b, wartet auf a` verlangt `rang(a) < rang(b)` und `rang(b) < rang(a)`, ein Widerspruch. Wartebeziehungen außerhalb von Sperren (Condvars, Ticket-Wartende, Claims) sind **kein** Sperrerwerb: Ein Condvar-Wartender hält beim Warten **keine** Sperre außer der zur Condvar gehörenden; Claims sind Tabellenzeilen (§16.5), kein Warten, sondern „überspringen oder abbrechen“ (`cancel`). ∎

### 16.2 Committer und Gegendruck ohne Wartezyklus

> **Erratum E-4 (korrigiert §10.2 Schritt 3 und §10.4):** Der Entwurf ließ den Leiter bis zu `stall_slice`/`stall_timeout` **unter dem Committer-Lock** warten. Das kann das Auflösen des Staus blockieren: Der Flush braucht für `INV-FLUSH-1` (Schritt 0) kurz den Committer (`Storage::sync()`), den der wartende Leiter hält. Normativ ist stattdessen:
> 
> 1. **`apply` wartet vor dem Ticket** (§10.2 Schritt 1½, ohne Sperre): bei Stufe `Verlangsamt` bis zu `stall_slice`, bei `Gestoppt` bis zu `stall_timeout` auf eine Condvar (`maint`-Rang 6), die Flush/Compaction/Governor bei Entspannung signalisieren; danach `Err(Busy{retry_after})`.
> 2. **Der Leiter wertet die Stufe nur aus** (Schritt 3) und wartet **nie**. Ist sie `Gestoppt`, erhält die Gruppe sofort `Err(Busy)`; sonst läuft sie.
> 
> Folge: Unter dem Committer wird nur noch auf bereits erteilte E/A gewartet, nie auf andere Wartungsjobs. Die Wartezeit je `apply` bleibt ≤ `stall_timeout`.

**Leiterwahl.** `apply` hängt ein `Ticket` in `intake` (Rang 2, kurz), danach `committer.lock()`. Wer ihn erhält, prüft zuerst, ob sein Ticket schon erledigt ist (dann zurück), sonst wird er Leiter und leert `intake`. **Lebendigkeit:** Jedes Ticket steht in `intake`, bevor sein Besitzer um den Committer konkurriert; der Leiter, der den Committer verlässt, hat `intake` zuletzt geleert, also wird jedes Ticket entweder vom aktuellen Leiter bedient oder sein Besitzer erwirbt den Committer als Nächster (Mutex ist endlich-fair genug: kein Ticket wartet auf unbegrenzt viele Gruppen, weil jede Gruppe `intake` komplett leert, höchstens 1 000 Changes/8 MiB; der Rest geht an den nächsten Leiter). `loom` prüft: kein verlorenes Ticket, kein doppelt bedientes Ticket.

### 16.3 Sichtbarkeit (`INV-VIS-1…3`)

Zustand: `visible_stamp: AtomicU64`, `state: RwLock<Arc<Version>>`.  
**Schreiber (Leiter):** (a) Memtable-Einträge der Gruppe einfügen (Rang 4, Unterbatches), (b) `visible_stamp.store(s, Release)`, (c) erst danach rotieren (Version tauschen, Rang 3).  
**Leser:** (a) `v = visible_stamp.load(Acquire)`, (b) danach `V = state.read().clone()` (Rang 3).

**Satz (`INV-VIS-3`).** Für jede View `(V, v)` enthält `V` (Memtables ∪ Runs) alle Einträge mit Stamp ≤ `v`. _Beweis._ Sei `e` ein Eintrag mit Stamp ≤ `v`. Weil `v` per Acquire geladen wurde, gilt für den Schreiber der Gruppe mit `v`: sein Einfügen von `e` (Schritt a) _happens-before_ das Laden (Release/Acquire). Zum Zeitpunkt des Einfügens stand `e` in der damaligen aktiven Memtable `M` der damaligen Version `V0`. Versionen wachsen nur monoton: Rotation macht `M` zu einer eingefrorenen Memtable derselben Daten, Flush ersetzt `M` durch einen Run mit denselben Einträgen (Schritt 4 des Flush erst **nach** dem Edit), Compaction erhält jede Version ≥ Untergrenze (`INV-RET-1`). `V` wurde **nach** dem Laden von `v` gelesen, ist also mindestens so neu wie jede Version, die zum Zeitpunkt des Releases galt; also enthält `V` `e` in `M`, einer eingefrorenen Memtable oder einem Run. ∎ _Warum die Reihenfolge nicht vertauschbar ist (Gegenbeispiel)._ Liest ein Leser **zuerst** `V0` (aktive Memtable `M`), rotiert der Leiter dann `M` und setzt `M2` als aktiv, fügt Einträge mit Stamp `s` in `M2` ein und veröffentlicht `s`, lädt der Leser danach `v = s`: `V0` kennt `M2` nicht, die View verliert Einträge ≤ `v`. Deshalb **zuerst `visible_stamp`, dann Version** (`INV-VIS-3`).

**`INV-VIS-2` (halbe Gruppen unsichtbar).** Leser ignorieren Einträge mit Stamp > `v`. Da `v` erst nach vollständigem Einfügen aller Einträge der Gruppe veröffentlicht wird, ist jeder Stamp ≤ `v` vollständig eingefügt; jeder Stamp > `v` wird gefiltert. Ein `Change` hat einen Stamp ⇒ er ist atomar sichtbar. **`INV-VIS-1`:** Einfügen (Schritt 9) folgt auf erfolgreiches Schreiben/Sync (Schritt 7/8). Scheitert `sync_data` (Poison), erreicht keine Op der Gruppe eine Memtable.

> **Erratum E-2 (Präzisierung §10.5 „Poison-Semantik“):** Der Satz „Memtables enthalten nur Daten aus erfolgreich synchronisierten Gruppen“ gilt nur für `Durability::Always`. Bei `Interval`/`Never` sind Daten mit Stamp > `durable_stamp` sichtbar, aber nicht dauerhaft; scheitert danach der Intervall-`fsync` (Poison), bleiben sie im degradierten Lesebetrieb sichtbar und können nach dem Neustart fehlen. Normativ: `health()` liefert `Poisoned{cause, durable_stamp}`; Views oberhalb `durable_stamp` tragen `maybe_lost = true`; Aufrufer, die `Interval`/`Never` wählen, haben diese Semantik gewählt. Das ist keine Verletzung von `INV-VIS-1` (das nur den Schreibpfad betrifft), sondern die dokumentierte Folge von `Interval`.

### 16.4 Untergrenze und Aufbewahrung (`INV-RET-1`)

**Registry (Rang 5)** hält: `views: Map<ViewId, Stamp>`, Holds, Consumer, `floor: u64`, Claims. **`register_view()`:** unter dem Registry-Lock `v = visible_stamp.load(Acquire)`; `floor = max(floor, compute_floor(v))`; View mit `v` eintragen. **`view_at(s)`:** unter **demselben** Lock `s ≥ floor` prüfen, eintragen, sonst `Err(HistoryPruned)`. `compute_floor(v) = min(v, min views, min holds, min consumed_stamp)`; `floor` ist durch `max` monoton. Die Wartung liest `floor_used` unter dem Registry-Lock, **bevor** sie plant, und bewahrt für den Job `retain(floor_used)`.

**Satz (`INV-RET-1`).** Eine Compaction/Flush mit `floor_used` verändert keine Sicht `(V, s)` mit `s ≥ floor_used`. _Beweis._ `retain` bewahrt je Schlüssel alle Versionen mit Stamp > `floor_used` und die neueste mit Stamp ≤ `floor_used`. Eine Sicht mit `s ≥ floor_used` liest je Schlüssel die neueste Version ≤ `s`: ist sie > `floor_used`, bleibt sie erhalten; sonst ist es die neueste ≤ `floor_used`, die ebenfalls erhalten bleibt. Löschmarker werden nur verworfen, wenn unter ihnen nichts mehr liegt (§12.2). Bleibt zu zeigen: Keine Sicht mit `s < floor_used` **entsteht später**. Jede neue Sicht wird unter dem Registry-Lock mit `s ≥ floor ≥ floor_used` registriert (monotones `floor`, `view_at`-Prüfung und Eintrag unter demselben Lock; `view()` nutzt `v ≥ floor`, weil `visible_stamp` monoton ist und `compute_floor` ≤ `v` ist). Bereits bestehende Sichten gingen in `compute_floor` ein: ihr Stamp ≥ `floor_used`. Holds/Consumer analog. ∎ Ausnahmen mit bewusst anderer Semantik: `CompactionFilter` in `forgetting`-Spaces (§12.2) und `Erase` (§14).

### 16.5 Claims, `obsolete`, Graveyard (`INV-CLAIM-1`)

Eine Datei trägt `obsolete: AtomicBool` im Handle. **Claim** = Zeile in der Claim-Tabelle (Registry, Rang 5) mit `(file, job, cancel_flag)`. `claim(files)` prüft unter dem Registry-Lock für **alle** Dateien: nicht beansprucht (oder vom `urgent`-Job abbrechbar) **und** `!obsolete`; dann alle oder keine.

**Satz (`INV-CLAIM-1`).** Kein Job beansprucht eine Datei, die ein anderer Job bereits ersetzt hat. _Beweis._ Installation eines Jobs (Edit → neue Version → `obsolete.store(true)` der Eingaben) geschieht, solange der Job seine Claims hält; Claims werden erst **nach** dem Setzen von `obsolete` freigegeben. Ein späterer `claim` sieht entweder die Zeile (Konflikt) oder, nach Freigabe, `obsolete = true` (Fehlschlag, Planer plant neu aus der aktuellen Version). Beides unter dem Registry-Lock, also nicht verschränkbar. ∎ **Graveyard:** Beim Drop der letzten `Arc<FileHandle>` mit `obsolete = true` wird der Pfad in eine Liste geschoben (kein E/A im `Drop`, P-9). Die `background`-Lane löscht per `remove_file` + `sync_dir`, wiederholt Fehlschläge und meldet sie als Metrik.

### 16.6 Poison und Panik

|Ereignis|Abbildung|
|---|---|
|`sync_data`/`sync_all`/`write_all_at` (nicht `ENOSPC`) im Committer scheitert|`Poisoned{cause: Io}`; alle Tickets der Gruppe `Err`; Flag `poisoned: AtomicBool` (Release); jeder Schreib-/Wartungseinstieg prüft es (Acquire)|
|`std`-Sperre vergiftet (`PoisonError`)|`Poisoned{cause: LockPoison}`; **kein** `into_inner()` zum Weitermachen|
|Panik in Aufrufer-Code in Wartung/Listener|`catch_unwind` an der Grenze; Job schlägt fehl, Listener abgemeldet; Instanz bleibt benutzbar (F-12)|
|Panik im Committer-Abschnitt|Sperrvergiftung ⇒ Poison (F-13)|
|Panik in `Weigher`/`Filter` in `scan_page`|`Err(CallerPanic)`; kein Poison|

Eine vergiftete Instanz verweigert alle Schreibvorgänge und Wartungsjobs, **liest weiter** (§10.5, E-2).

### 16.7 Was `loom` prüft

Genau diese Modelle (jeweils ≤ 3 Threads, begrenzte Schritte): (1) Ticket-Warteschlange und Leiterwahl; (2) Sichtbarkeitsprotokoll mit Rotation (§16.3, inklusive des Gegenbeispiels als erwartetem Fehlschlag mit vertauschter Reihenfolge); (3) Registry: `register_view` ‖ `compute_floor` ‖ Compaction-`retain` (`INV-RET-1`); (4) Claim-Tabelle mit `obsolete` (`INV-CLAIM-1`); (5) Graveyard-Übergabe beim letzten `Arc`-Drop. Gesamtsystemläufe nutzen `Runtime::manual()` mit zufälliger Einplanung und `Hooks`.

---

## 17. Ressourcen und Effizienz (Z-12)

Ziel: begrenzter, vorhersagbarer Verbrauch, kein Thread je Instanz, Leerlauf nahe null, faire Teilung zwischen vielen Instanzen im selben Prozess.

### 17.1 Governor

Ein geteiltes Objekt je Prozess (`Arc<Governor>` in `Env`), von vielen `Storage` genutzt:

|Budget|Verfahren|Wirkung bei Knappheit|
|---|---|---|
|Memtable-Gesamtsumme|gewichtete Quote je Instanz, harte Obergrenze|früherer Flush; Stufe `Verlangsamt`/`Gestoppt` (§10.4)|
|`BlockCache`|ein CLOCK-Pool, Mindestanteil je Instanz|fremde Einträge verdrängen, Mindestanteil bleibt|
|offene Dateien (`TableCache`)|globale LRU, Quote je Instanz|schließen und bei Bedarf neu öffnen; Footer/Index-Cache bleibt|
|E/A-Raten|Token-Buckets (Lesen/Schreiben, `Clock`-basiert) für `background`; `urgent` ungedrosselt|Compaction wird gedrosselt|
|Vorlesen|begrenzte Tiefe|weniger Vorlesen|
|Wartungs-Threads|`Runtime` mit `n = min(4, Kerne)`; je Instanz höchstens zwei Jobs (Lanes)|faire Reihenfolge nach Rückstand|
|`PressureSource` (optional)|cgroup/PSI-Stufe|verkleinert Caches, erzwingt Flush|

### 17.2 Speicherbudget je Instanz (Richtwerte, in F5 kalibriert)

`Σ Memtables ≤ (1 + max_frozen) · memtable_bytes · Spaces_aktiv` (Hard-Cap über Governor-Quote); Index-/Filter-Blöcke zählen zum `BlockCache`; Redo-Puffer ≤ Gruppengröße (8 MiB); Körper werden nie im `BlockCache` gehalten. Offene Datei-Deskriptoren ≤ `max_open_files` + Redo + LOCK + Catalog.

### 17.3 Leerlauf und Hibernation

Ohne Last: kein Thread, kein Timer außer dem optionalen Intervall-`fsync` und `consumer_check_interval`; beide laufen als Jobs der geteilten `Runtime`, nicht als Thread je Instanz. `hibernate()` gibt Memtable und Caches frei (H-S16).

### 17.4 Faire Teilung

Quoten sind Gewichte, keine festen Teile: eine ruhende Instanz gibt Cache und Dateien frei, eine aktive darf bis zur harten Grenze wachsen. Mindestanteile verhindern Verhungern.

### 17.5 Allokationsbudget im Heißpfad (P-11, per zählendem Allokator geprüft)

|Operation|Budget (Hypothese, in F5 festgelegt)|
|---|---|
|`View::get` mit Cache-Treffer|≤ 2 Allokationen (Arc-Klon des Blocks, Rückgabewert; Header/Wert als `Bytes`-Ausschnitt ohne Kopie)|
|`Cursor::next`|≤ 1 je Eintrag (nur wenn der Schlüssel rekonstruiert werden muss)|
|`apply` einer Gruppe|≤ `2 + Anzahl Changes` (Frame-Puffer wiederverwendet, Ticket-Slab)|
|Flush/Compaction|Puffer je Job einmal, nicht je Block|

Weitere Ressourcentests: RSS-Obergrenze je Konfiguration unter Last; `ulimit -n 256` mit 1 000 Runs; 1 000 hibernierte Instanzen (Fußabdruck und CPU); Gegendruck unter simuliertem Speicherdruck.

---

## 18. Invariantenkatalog

Jede Invariante wird in **mindestens einem** Test per ID zitiert (`// INV-LOG-6`); `tests/invariants.rs` liest diese Datei, extrahiert alle `INV-*`-IDs aus der Spalte 1 und schlägt fehl, wenn eine ID in keinem Test vorkommt.

|ID|Aussage|Netz (§19)|
|---|---|---|
|`INV-STAMP-1`|Stamps sind über **angenommene** Changes streng monoton und lückenlos; erster Stamp ist 1.|Modell, Property|
|`INV-STAMP-2`|Stamps < 2^56; Überlauf ⇒ `Exhausted`, nie Umbruch.|Property|
|`INV-FMT-1`|Nur kanonische Varints sind gültig.|Fuzz, Golden|
|`INV-FMT-2`|Jedes Längenfeld wird vor Allokation gegen §6.7 und die Restlänge der Datei geprüft; Dekodierung ist total (P-4).|Fuzz, Bitflip|
|`INV-CAT-1`|Nur ein dauerhafter Catalog-Edit macht eine Datei lebendig oder obsolet; nicht genannte Dateien sind verwaist und löschbar.|Absturzmatrix|
|`INV-CAT-2`|Ein Edit ist **ein** Record (P-2); er wird ganz oder gar nicht angewendet.|Absturzmatrix|
|`INV-CAT-3`|Nur der **letzte** Catalog-Record darf als Torn Write verworfen werden; früherer Defekt ⇒ `Corrupt` (kein Wiederauferstehen gelöschter Dateien).|Bitflip, Fuzz|
|`INV-CAT-4`|Nach einem Absturz räumt die Verwaisten-Suche alle `.building.*`, nicht genannten Dateien und nicht aktuellen Blob-Generationen auf; Reserve-Redo (`≥ redo_floor`) bleibt.|Absturzmatrix|
|`INV-LOG-1`|Ein Frame = ein `Change`.|Modell|
|`INV-LOG-2`|Beim Replay ist ein Frame vollständig gültig oder beendet den Log.|Fuzz, Absturzmatrix|
|`INV-LOG-3`|Stamps im Log streng aufsteigend (nicht lückenlos).|Property|
|`INV-LOG-4`|Jede Prozessinstanz beginnt ein neues Segment; nie anhängen.|Absturzmatrix|
|`INV-LOG-5`|`fsync`-Fehler ⇒ Poison, nie Retry, nichts in die Memtable.|`FaultFs`|
|`INV-LOG-6`|Jedes Segment außer dem neuesten mit Frames ist gesiegelt; Defekt in gesiegeltem Segment ⇒ `Corrupt`; `frame_count` passt.|Absturzmatrix, Bitflip|
|`INV-LOG-7`|Ein Segment ohne Frames mit gültigem Header und `id ≥ redo_floor` ist lebendig (Reserve).|Absturzmatrix|
|`INV-LOG-8`|Letzter gültiger Redo-Stamp < `redo_hwm` (ohne `redo_floor_stamp`-Erklärung) ⇒ `Corrupt{RedoTruncated}`.|`FaultFs` (Lost Write)|
|`INV-VIS-1`|Memtables enthalten nur Daten aus erfolgreich geschriebenen Gruppen.|`FaultFs`, Hooks|
|`INV-VIS-2`|Halb eingefügte Gruppen sind unsichtbar; ein `Change` ist atomar sichtbar.|Hooks, `loom`|
|`INV-VIS-3`|Leser laden erst `visible_stamp`, dann die Version (Beweis §16.3).|`loom`|
|`INV-RET-1`|Compaction/Flush verändert keine Sicht ≥ `floor_used`; neue Sichten entstehen nie unter `floor` (Beweis §16.4).|Property, `loom`|
|`INV-FLUSH-1`|Ein Run mit Stamp `s` wird erst dauerhaft, wenn der Redo bis `s` per `sync_data` gesichert ist (gilt auch für Erase-Barriere und `checkpoint`).|Absturzmatrix|
|`INV-CLAIM-1`|Kein Job beansprucht eine bereits ersetzte Datei (Beweis §16.5).|`loom`|
|`INV-LOCK-1`|Sperren nur in aufsteigendem Rang (§16.1).|Rangstack-Test, `loom`|
|`INV-KEY-1`|Kein `(Schlüssel, Nonce)`-Paar kommt je zweimal vor.|Property, Absturzmatrix|
|`INV-KEY-2`|IDs (`run_id`, `blob_no`, `segment_id`, `tmp_id`) werden erst nach dauerhaftem Hochwassermarken-Edit benutzt und nie wiederverwendet.|Absturzmatrix|
|`INV-BLOB-1`|Jeder `BlobRef` löst auf genau einen Record gleichen Schlüssels/Stamps (oder `blob_no = 0`).|`verify`, Property|
|`INV-BLOB-2`|Kein Teilen: ein `BlobRef` ↔ ein Record; `live_bytes` ist exakt.|Property|
|`INV-BLOB-3`|Hole-Records sind nie verschlüsselt; ein `BlobRef` auf ein Hole ⇒ `Corrupt{HoleRef}`.|Bitflip, Orakel|
|`INV-ERA-1`|Logische Atomarität: View ≥ `S` sieht den Bereich leer, View < `S` ⇒ `Erased`; keine Mischsicht.|Modell|
|`INV-ERA-2`|Nach `erase_done(S)` enthält kein lebender Run und kein lebendes Redo-Segment eine Version im Bereich mit Stamp ≤ `S`.|Orakel|
|`INV-ERA-3`|Schlüssel vor Datei; der Kern führt den Auftrag nie aus, solange der Schlüssel noch liefert.|Absturzmatrix|
|`INV-ERA-4`|Nur Roll-forward: ein begonnener Erase wird nach jedem Absturz fortgesetzt oder ist nie begonnen.|Absturzmatrix|
|`INV-ERA-5`|Lesen unterhalb `S` ⇒ `Erased`, nie Daten, nie „leer“.|Modell|
|`INV-ERA-6`|Nach `erase_done(S)` enthält keine lebende Blob-Generation einen Body-Record im Bereich mit Stamp ≤ `S`.|Orakel|
|`INV-ERA-7`|Körper einer vernichteten Domäne ist weder in Redo noch Blob ohne Schlüssel lesbar; `Entry::body()` ⇒ `Shredded`, auch aus der Memtable.|Orakel|
|`INV-POISON-1`|Ein vergiftetes Handle verweigert Schreiben und Wartung, liest weiter; `health()` nennt `durable_stamp` (E-2).|Fehlerklassen-Matrix|
|`INV-API-1`|Fehlertexte, Metriken und Logs enthalten keine Schlüssel-/Wertbytes (P-12).|Lecks-Orakel|
|`INV-FS-1`|Unter `FsPolicy::Strict` öffnet der Kern nur auf einer `Fs`, deren `caps()` Datei-/Verzeichnis-Sync, atomares `rename` und private Modi zusichert.|Manipulationstest, `fs_suite`|
|`INV-KEYP-1`|Ein Frame mit domänenversiegeltem Körper wird erst dauerhaft, nachdem `KeyProvider::make_durable` nach der Schlüsselanlage zurückkehrte; `None` aus `domain_key` bedeutet dauerhaft vernichtet.|Absturzmatrix, `FaultFs`|
|`INV-HOLD-1`|Dauerhafte Holds überleben Neustart und zählen bis Ablauf/`hold_drop` in die Untergrenze.|Absturzmatrix|
|`INV-SEG-1`|Ein Segment wird nur mit gültigem Siegel-Footer und per Edit lebendig; Rahmenprüfung vor jeder Block-Ausgabe.|Bitflip, Absturzmatrix|
|`INV-EPOCH-1`|Original und beschreibbarer Klon versiegeln nie unter gleichem (DEK, Nonce) verschiedene Inhalte.|Property (Klon-Divergenz)|
|`INV-SCAN-1`|`scan_physical` ändert nichts und gibt keine Inhalte preis.|Lecks-Orakel|

---

## 19. Teststrategie und Qualitätstore

### 19.1 Netze

|Netz|Inhalt|
|---|---|
|**Modelltest**|Referenzmodell (`BTreeMap` mit Versionsliste je Schlüssel, Holds, Consumer, `erased_ranges`) gegen die Engine: zufällige Folgen aus `apply` (Guards, Durability, ephemeral), `view_at`, `scan`, `scan_page`, `history`, Flush, Compaction, Blob-GC, `CompactionFilter`, `Erase`, Neustart. Eigener Xorshift-PRNG, Seed bei Fehlschlag ausgegeben.|
|**Gesetze (proptest)**|`decode ∘ encode = id` für alle Formate; Run-Lesen = Eingabe; `retain` idempotent; Planer liefert nie überlappende Ausgaben auf Level ≥ 1; Nonce-Eindeutigkeit; Blob-Lebendbilanz = Σ `PutRef` in lebenden Runs; Ordnung `(user_key ↑, tag ↓)`.|
|**Absturzmatrix**|`FaultFs` bricht an **jedem** E/A-Index ab (§19.2); danach `open`, Vergleich mit Modell-Präfix der quittierten Changes, `verify` grün. Jede Zeile aus §13.2 und §14.3 (E1–E7) wird zu einem Testfall.|
|**Fehlerklassen-Matrix**|Jede Klasse F-1…F-15 hat mindestens einen gezielten Test mit definiertem Zustand danach (§5.1, Spalte „Prüfnetz“).|
|**Linearisierbarkeit**|`testing::linearize`: nebenläufige Läufe (n Schreiber, m Leser, Wartung) werden aufgezeichnet; die Prüfung weist eine zur Stamp-Folge passende Ordnung nach, die alle gelesenen Sichten erklärt (wegen der Stamp-Ordnung linear in der Historie).|
|**Sichtbarkeit**|`Hooks` an `AfterRedoSync`/`BeforeVisible` pausiert: keine View sieht einen halben Change; nach Freigabe sieht jede neue View alle Ops oder keine.|
|**Bitflip**|Jedes Byte jeder Datei (Run, Blob, Catalog, Redo, `FORMAT`, `KEYS`, `CURRENT`) wird gekippt: Ergebnis `Err(Corrupt)` oder identisch, nie falsche Daten; in `Authenticated/Encrypted` ausnahmslos `Err` (außer in Hole-Nutzbereichen, dort `HoleRef`).|
|**Fuzz**|Decoder: Redo-Frame/Seal, Run-Block/Footer/Directory, Blob-Record/Hole, Catalog-Record, `FORMAT`, `KEYS`, `KeyRange`, `ResumeToken`; Stateful-Fuzz (`arbitrary`-Aktionsfolgen gegen das Modell).|
|**Nebenläufigkeit**|`loom` für die fünf Modelle aus §16.7; Gesamtsystem über `Runtime::manual()`.|
|**Panic-Injektion**|Panik in `Weigher`, `Filter`, `CompactionFilter`, `Extension`, `Listener`: Daten unverändert, Job scheitert, Instanz benutzbar (F-12); Panik im Committer ⇒ Poison (F-13).|
|**Schichtregel**|`tests/layering.rs` (§3.3); `cargo tree -e normal` zeigt nur `contextra-ports`.|
|**Lösch-Orakel**|§14.7.|
|**Lecks-Orakel**|§15.7, `INV-API-1`.|
|**Ressourcentests**|Allokationsbudget, Deskriptorgrenze, RSS, Leerlauf (§17.5).|
|**Soak**|Agenten-Lastprofil (viele kleine Changes, kurzlebige Views, seltene Erases, 4-KiB-/64-KiB-Körper), Platte-voll, Speicherdruck, Dauer ≥ 12 h nachts.|
|**Goldens**|Je Dateiart Referenzdateien (Anhang A, Freeze-Liste §9.8) in **allen drei** Integritätsmodi; Byte-Stabilität über Plattformen (x86-64, aarch64, big-endian-Host per `miri`/Cross-Test).|
|**Differentialtest (begrenzt)**|Schlanke Harness gegen die Bibliotheks-API des rekonstruierten Altstands (Anhang B.2) für `put/get/delete/scan/commit` ohne Stromausfall; Abweichungen sind Befunde, kein automatischer Fehler.|

**CI-Gates (jeder PR):** `cargo test` (alle Netze außer Soak/Fuzz-lang), `clippy -D warnings`, `cargo doc` ohne Warnungen, `cargo deny`, `cargo miri test` für `codec` und `keys`, Fuzz-Smoke (60 s je Ziel), Absturzmatrix, `loom`-Subset, `tests/invariants.rs`. **Nachts:** Fuzz lang (≥ 30 min je Ziel), Soak, Benchmarks gegen die Hypothesen (Regressionsgrenze ±10 %).

### 19.2 `MemFs` und `FaultFs`

`MemFs` hält je Datei zwei Zustände: _Seitencache_ (alles Geschriebene) und _dauerhaft_ (Stand der letzten `sync_data`/`sync_all`). Verzeichniseinträge (Anlegen, `rename`, `remove`) sind erst nach `sync_dir` dauerhaft. `FaultFs` kann:

- an E/A-Operation Nr. _n_ einen **Absturz** auslösen: danach zählt nur der dauerhafte Zustand; nicht synchronisierte Schreibvorgänge gehen **verloren, teilweise (zerrissen, 512 B/4 KiB) oder in zufälliger Reihenfolge** in den Zustand ein;
- `fsync`/`sync_dir` **fehlschlagen** lassen (Poison-Pfad);
- „Platte voll“ (`StorageFull`) und Kurzschreibvorgänge simulieren, auch **mitten im Gruppen-Schreibvorgang** (§10.5);
- `rename` ohne `sync_dir` rückgängig machen;
- **Lesefehler** (`EIO`) und **Bitfäule** einzelner Offsets beim Lesen einschleusen;
- **verlorene** (quittiert, nicht geschrieben) und **fehlgeleitete** Schreibvorgänge (falscher Offset derselben Datei) einschleusen;
- das **Zurückspielen eines älteren Gesamtstands** (F-10) für den Anker-Test;
- jede Operation mit einer **Hook-Marke** versehen (`AfterRedoWrite`, `AfterRedoSync`, `BeforeVisible`, `BeforeCatalogEdit`, …).

Selbsttests von `FaultFs` sind Gate von F1: Nicht synchronisierte Daten gehen verloren, `rename` ohne `sync_dir` wird zurückgesetzt, Zerreißen in Sektorgrößen funktioniert, Lost-/Misdirected-Write sind deterministisch aus dem Seed reproduzierbar.

### 19.3 Überführung des Bestandes

Aus den 160 Testdateien des Altstands werden **Szenarien** übernommen, nicht Code (der Testkorpus ist nicht übersetzbar, B-14). Verworfen werden Tests für Legacy-Formate (`wal_legacy_*`, `hmac_legacy_*`, `format_version_compat`), `rollback_to_tx`, den adaptiven Planer und `docid-128`. `MergeOperator`-Tests sind nach SD-12 zurückgestellt (B-15). Anhang B.3 ordnet die Szenarioklassen den Netzen zu.

---

## 20. Leistungshypothesen

**Es wurde nichts gemessen.** Jede Zeile ist eine Hypothese, die in der genannten Phase auf der Referenzhardware (A-09) belegt oder verworfen wird; Gate **G-P** verlangt den Bericht.

|ID|Hypothese|Phase|
|---|---|---|
|H-S1|Punktlesen mit warmem Block-Cache `p99 < 50 µs`; kalt höchstens Anzahl Level × 1 Block-E/A|F3, F5|
|H-S2|Ein `apply` mit `Always` kostet ein `sync_data`; bei 64 parallelen Schreibern ≥ 20× Durchsatz gegenüber seriellem Fsync (inkl. Guard-Vorwärmung)|F5|
|H-S3|Leveled, Faktor 10: Schreibverstärkung ≤ 15, Platzverstärkung ≤ 1,3 (ohne Holds) für Header-Daten; zusätzlich mit 4-KiB- und 64-KiB-Werten|F6|
|H-S4|Wiederanlauf ≤ 1 s je 256 MiB Redo-Replay|F4/F5|
|H-S5|Öffnen ohne Redo und mit 1 000 Runs < 50 ms (Metadaten aus Catalog, Runs lazy)|F4|
|H-S6|`drop_space` mit 1 000 Runs < 100 ms; `Erase::range` kostet O(Bytes der berührten Dateien)|F7|
|H-S7|Memtable `RwLock<BTreeMap>` mit blockweisem Scan und 64-Ops-Unterbatches hält den Durchsatz (Schreiben + 8 Leser) bei ±20 % der Referenz|F5|
|H-S8|`estimate` über 1 000 Index-Einträge < 1 ms; obere Schranke ≤ 2× ohne starke Überschattung|F5|
|H-S9|Präfix-Filter reduziert Block-Lesevorgänge bei Präfix-Scans ohne Treffer in ≥ 90 % der Runs um ≥ 80 %|F3|
|H-S10|`separate_body(4 KiB)` senkt die Schreibverstärkung bei 64-KiB-Werten um ≥ 3×|F6|
|H-S11|Scan mit Header-Filter (Körper 8 KiB, Header 32 B) liest ≥ 10× weniger Bytes als mit Inline-Werten|F5|
|H-S12|`read_batch` über 32 Blöcke ≥ 2× schneller als 32 serielle `read_at` auf NVMe|F3/F5|
|H-S13|Vorallokierte Redo-Segmente senken die `sync_data`-Latenz (Median, p99)|F5|
|H-S14|`Erase::domain` < 10 ms unabhängig vom Datenvolumen|F8|
|H-S15|Ein `BodyCache` lohnt nur, wenn die Trefferquote des Seitencaches < 70 % ist|F5|
|H-S16|1 000 hibernierte Instanzen: < 1 % CPU im Leerlauf, < 2 MiB RSS je Instanz|F9|
|H-S17|Blob-Konsolidierung hält die Dateizahl je Space unter `max_open_files / Spaces` bei 10 Mio. Flushes-Äquivalent|F6|
|H-S18|`Authenticated` kostet ≤ 15 %, `Encrypted` ≤ 40 % Durchsatz gegenüber `Plain` (mit AES-NI-/NEON-Port)|F8|

**Abbruchkriterien:** Verfehlt F5 H-S7 um > 2×, wird die Memtable durch eine Skip-Liste in einer **isolierten, gesondert geprüften** Einheit ersetzt (SD-10; einzige Stelle, an der `unsafe` überhaupt diskutiert wird, nur per Beschluss). H-S13 verfehlt ⇒ Vorallokation bleibt Option, nicht Pflicht. H-S3 deutlich verfehlt ⇒ zweites Verfahren (Size-Tiered für L0) als **eigene** Entscheidung, nicht im Voraus.

---

## 21. Entscheidungsregister

Die Nummern 01–26 stammen aus `REFACTORING_PLAN.md` (v1), 27–48 aus v2 und dieser Spezifikation. Nummern, die in den Altdokumenten nicht belegt waren (35, 37–40, 45–47), sind hier **erstmals** vergeben.

|ID|Entscheidung|
|---|---|
|SD-01|Keine internen und keine externen Laufzeit-Abhängigkeiten im Kern (nur `std` + `contextra-ports`); Kryptografie über Ports. _(ersetzt „encryption zieht Krypto-Crates“)_|
|SD-02|Synchroner Kern; Hintergrundarbeit auf teilbarer `Runtime`; `tokio` entfällt.|
|SD-03|Ein Port `Fs` für alle E/A inkl. Verzeichnisse und Sperre; Stromausfall-Simulator ist Pflichttor.|
|SD-04|Genau ein Format je Dateiart mit Feature-Bits; Altformate nur über ein separates Importer-Crate.|
|SD-05|Der Catalog ist alleinige Wahrheit; Zustandsänderung = ein Edit; numerische IDs; Metadaten im Catalog.|
|SD-06|Ein Redo-Frame je `Change`; neues Segment je Prozessstart; Rotation statt Truncate-in-place.|
|SD-07|Spaces = getrennte LSM-Bäume mit gemeinsamem Redo und gemeinsamer Stamp-Folge; Space steckt nicht im Schlüssel.|
|SD-08|`Change` mit `Guard`s ersetzt Transaktionspuffer, SSI und Intent-Locks.|
|SD-09|`rollback_to_tx` entfällt; Zeitreise über `view_at` und Holds (belegt durch B-7).|
|SD-10|Memtable = `RwLock<BTreeMap>` mit blockweisem Scan; Ersatz nur bei verfehlter H-S7.|
|SD-11|Ein Compaction-Verfahren (Leveled), Planer als reine Funktion.|
|SD-12|Kein Merge-Operator in Version 1 (`kind = 6` reserviert); Read-Modify-Write über `Guard`.|
|SD-13|Keine Kompression in Version 1 (`codec`-Byte und `BlockCodec`-Trait reserviert).|
|SD-14|`Erase` ist Kernprimitiv mit einem Durability-Punkt und Roll-forward; nie über Dateiendungen oder am Catalog vorbei.|
|SD-15|Schlüsselvernichtung vor Dateilöschung; echte Unwiederbringlichkeit nur über `Cryptographic`.|
|SD-16|**Geändert durch SD-32:** (v1: ein Wartungsjob je Instanz).|
|SD-17|Fehler sind ein Enum mit `code`, `retryable`, strukturierten Feldern.|
|SD-18|Eine Instanz, ein Prozess; Mehrprozess über einen Daemon der oberen Schicht.|
|SD-19|Seitenweises Lesen mit `ResumeToken`; Bindung an Stamp und Bereich; Ablauf ist `Expired`.|
|SD-20|`estimate` aus Index-Zählern (obere Schranke), ohne Datenblöcke.|
|SD-21|`history` mit Untergrenzenprüfung (`HistoryPruned`).|
|SD-22|Append-only Spaces vom Kern erzwungen; `Erase` nur mit `admin()`.|
|SD-23|`DirectoryBlock`, Präfix-Filter (Standard in Haupt-Spaces), Extension-Blöcke von Anfang an.|
|SD-24|`tier` für **alle** Dateiarten; `ColdTier` als Port in F10.|
|SD-25|Fehler tragen `hint()` (datenfrei).|
|SD-26|Lokalität nur über Schlüsselgestaltung und Run-Teilung; der Kern ordnet nie nach Zugriff um.|
|SD-27|Zwei Ebenen: Header im LSM, große Körper in Blob-Dateien unter derselben Catalog-Verwaltung.|
|SD-28|Gewichte als `u64`, mit `weigher_version`; `weights[0]` = logische Bytes.|
|SD-29|Körper reisen im Redo-Frame und wandern beim Flush ins Blob: ein `fsync` je Gruppe.|
|SD-30|Segmente (`FileKind::Segment`) für nicht zerlegbare Indizes; Lebenszyklus und Erase im Kern, Format beim Besitzer.|
|SD-31|Kein `O_DIRECT`, kein `io_uring`, kein Thread-per-Core im Kern; über alternative `Fs`-Implementierung.|
|SD-32|Zwei Lanes (`urgent`, `background`) mit Claim-Tabelle; Erase/Flush warten nie hinter Compaction.|
|SD-33|Drei Integritätsmodi (`Plain`, `Authenticated`, `Encrypted`), unveränderlich, Downgrade-Schutz über `Config`.|
|SD-34|Typisierte Capability-Handles `Rd < Ap < Wr < Adm`.|
|SD-35|Poison lässt Lesen zu (degradierter Betrieb); `health()` nennt `durable_stamp` (E-2).|
|SD-36|Kryptografische Primitive über Ports, Protokoll im Kern; `root_key` in `KEYS`, KEK umhüllt nur `KEYS` (Rotation O(1)).|
|SD-37|`RollbackAnchor` als optionaler Port; ohne Anker ist F-10 offen dokumentiert.|
|SD-38|Ein prozessweiter `Governor` mit Quoten je Instanz statt Budgets je Instanz.|
|SD-39|Prüfpunkte über `Env::hooks()`; keine globalen veränderlichen Variablen (löst B-10).|
|SD-40|`ResumeToken` mit `chk` (CRC32C bzw. `Mac16`), nicht fälschbar im Modus ≥ `Authenticated`.|
|SD-41|`CompactionFilter` nur in `forgetting`-Spaces; entfernt oder kürzt, kein Merge.|
|SD-42|Domänenschlüssel auf Körper-Ebene für kryptografisches Löschen je Subjekt.|
|SD-43|Blob-GC compaction-getrieben, mit Konsolidierung kleiner Blobs.|
|SD-44|Redo-Segmente sind kein `FileKind`; Lebendigkeit über `redo_floor`.|
|SD-45|Leser laden `visible_stamp` **vor** der Version (Beweis §16.3).|
|SD-46|ID-Hochwassermarken im Catalog **vor** Nutzung (`INV-KEY-2`).|
|SD-47|`redo_hwm` in jedem Edit; Lost-Write im Redo-Schwanz wird erkannt (`INV-LOG-8`).|
|SD-48|Ehrliche Semantik des Vergessens: sichtbare Zustandsänderung, kein Schutz durch `Hold` in `forgetting`-Spaces.|
|SD-49|**Neu:** Gegendruck wartet **vor** dem Ticket, nie unter dem Committer (E-4).|
|SD-50|**Neu:** Nonce-Präfixe sind je Verwendung disjunkt (`0x01…0x05`, E-6).|
|SD-51|Verantwortungsgrenze (§2.3) ist bindend: Mechanismus im Kern, Politik außerhalb; nichts wird „vorsorglich“ gebaut.|
|SD-52|Flächen aus §7.8 sind semver-stabil und konformitätsgetestet.|
|SD-53|Guard-Vorwärmung ist Pflicht im `apply`-Pfad (E-9).|
|SD-54|Segmente haben einen Kern-Rahmen (Block-Prüfwert/AEAD, Siegel-Footer) und ein opakes `reference`.|
|SD-55|`KeyProvider` mit `domain_key_create` und `make_durable`; `None` = dauerhaft vernichtet.|
|SD-56|`SecretAlloc` als optionaler Port für geschützten Speicher; Kern bleibt `forbid(unsafe_code)`.|
|SD-57|`FsCaps` + `FsPolicy::Strict` als Standard (fail-closed auf schwächeren Plattformen).|
|SD-58|Dauerhafte Holds im Catalog (`hold_set`/`hold_drop`).|
|SD-59|`submit`/`Pending` mit Fertigmeldung statt `async`.|
|SD-60|Merge-Space-Flag `32` und Pflicht-Bit `MERGE` reserviert, nicht erzeugbar.|
|SD-61|`erase_status` und rein lesender `scan_physical` als Beobachtungsfläche.|
|SD-62|Listener-Zustellung mit Frist; Überschreitung meldet ab, blockiert nie den Commit.|
|SD-63|Schlüsselepoche und neuer DEK für beschreibbare Klone (Nonce-Eindeutigkeit über Instanzen).|
|SD-64|Portables CRC32C im Kern; beschleunigte Varianten nur über Port und mit bestandenen Goldens.|

---

## 22. Annahmen, offene Punkte, Errata

### 22.1 Annahmen (`A-nn`)

|ID|Annahme|Wirkung, falls falsch|
|---|---|---|
|A-01|Es existieren **keine produktiven Altdaten** im Bestandsformat (das Repository enthält im HEAD nur das Crate; Historie siehe B-1).|Phase F-I (Importer) wird Pflicht.|
|A-02|Windows ist in Version 1 nur ohne Durability-Garantie unterstützt.|Eigener Windows-`Fs` für `sync_dir`/`rename`.|
|A-03|Werte ≤ 1 MiB genügen; größere Daten liegen in einer Objektschicht.|Wertgrenze und Frame-Grenze anheben.|
|A-04|Obere Schichten nutzen `Guard` statt `TxBuffer`/SSI.|Aufwand in aufsetzenden Schichten.|
|A-05|`File::try_lock` steht zur Verfügung; MSRV in F0 festgelegt. `sync_all` entspricht auf Apple-Plattformen `F_FULLFSYNC` — **in F1 per Test zu belegen**.|Ersatz über eigenes `Fs`-Mitglied.|
|A-06|Aufrufer von `rollback_to_tx` sind durch Versionierung der oberen Schicht ersetzbar.|Übergangsfunktion.|
|A-07|Passphrase→Schlüssel darf das Crate verlassen.|`KeyProvider` erweitern.|
|A-08|Umbenennung `contextra-store` → `contextra-storage`; fremde Verweise sind im Repository nicht sichtbar.|Alias-Crate in der Übergangszeit.|
|A-09|Referenzhardware (NVMe, Kerne, RAM) wird in F1 festgelegt.|Messwerte nicht vergleichbar.|
|A-10|`Extension` bleibt bis F10 instabil.|Schnittstelle ändert sich, Format nicht.|
|A-11|Kalter Speicher löscht zuverlässig oder wird nur verschlüsselt betrieben.|`Physical` gilt dort nicht.|
|A-12|Aufsetzende Schichten kommen in Version 1 ohne Merge-Operator aus (Read-Modify-Write über `Guard`, Pufferung, Anhang D.2).|Entscheidung SD-12 überprüfen.|

### 22.2 Offene Entscheidungen (`O-n`, mit Empfehlung)

|ID|Frage|Empfehlung|Entscheidung bis|
|---|---|---|---|
|O-1|Toolchain/MSRV|stabile Rust-Version mit `File::lock/try_lock` (nach meinem Kenntnisstand ≥ 1.89, in F0 gegen die Toolchain prüfen); in F0 pinnen|G-0|
|O-2|Rückstände abgeleiteter Daten (Embeddings, Postings) nach `Erase`|Segmente je Löschdomäne; `rewrite_without` Pflicht für löschfähige Formate|F10|
|O-3|`BodyCache`|nur, wenn H-S15 es belegt|F5|
|O-4|Windows-`Fs`|separates Crate nach V1|nach F9|
|O-5|Schwellen der Blob-Konsolidierung (`blob_min_bytes`)|16 MiB starten, in F6 messen|F6|
|O-6|Referenz-`Cipher`/`Mac` für die Goldens|BLAKE3 keyed + ChaCha20-Poly1305 im Dev-Crate `contextra-crypto-ref`|**G-F**|
|O-7|Block-Kompression (SD-13)|frühestens V2 nach Messung|nach F9|

### 22.3 Errata zu §1–§13

Beim Vervollständigen wurden folgende Stellen des ersten Teils als zu knapp oder inkonsistent erkannt. **Die Korrekturen gelten ab sofort normativ;** der erste Teil ist beim nächsten Edit entsprechend nachzuziehen (§1.5).

|ID|Stelle|Problem|Korrektur|
|---|---|---|---|
|E-1|§9.7 `spaces_add`, Flags|Das Flag `forgetting` (16, §12.2) fehlt in der Aufzählung `1, 2, 4, 8`.|Flags: `1 separate_body, 2 domain_sealed, 4 append_only, 8 ephemeral, 16 forgetting`. `forgetting` ist unvereinbar mit `append_only`.|
|E-2|§10.5 „Poison-Semantik“|„Memtables enthalten nur Daten aus erfolgreich synchronisierten Gruppen“ gilt nur für `Always`.|§16.3 (E-2): `health()` mit `durable_stamp`, `maybe_lost`-Views.|
|E-3|§9.7 Catalog-`chk` (Plain)|„`catalog_no` als `u32` gefaltet“ ist nicht definiert.|`fold(n) = (n as u32) ⊕ ((n >> 32) as u32)`, als `u32` little-endian Startwert des Seeds.|
|E-4|§10.2 Schritt 3, §10.4|Leiter wartet unter dem Committer-Lock auf Entspannung ⇒ möglicher Stillstand mit Flush (Schritt 0).|§16.2: Warten vor dem Ticket, Leiter wertet nur aus.|
|E-5|§9.4 `HWM-Verankerung`|`redo_hwm` unter `Interval`/`Never` ist `durable_stamp` des Edits, nicht `visible_stamp`; sonst falscher Alarm nach legitimem Verlust ungesicherter Frames.|`redo_hwm := durable_stamp` (so schon in §12.1 Schritt 3); hier ausdrücklich festgehalten.|
|E-6|§9.4/§9.6 Nonce-Präfixe|`0x04` doppelt belegt (Blob-Record und Domänen-Körper im Blob).|§15.2: Domänen-Körper im Blob `0x05`.|
|E-7|§7.2 `ResumeToken`|Layout nicht festgelegt.|Anhang A.7.|
|E-8|§9.5 Run-Block-Nonce|Nicht genannt.|§15.2: `0x01 ‖ block_offset u64 ‖ 0u24` unter Run-Schlüssel.|

---

### 22.4 Revision R2 (Änderungsliste)

|ID|Änderung|
|---|---|
|E-9|§10.2 Schritt 1: Guard-Vorwärmung von SOLL auf MUSS; neuer Schritt 1b (Schlüsselanlage + `make_durable`).|
|E-10|Neue Abschnitte §2.3 (Verantwortungsgrenze) und §7.8 (stabile Flächen); NZ-13…NZ-20.|
|E-11|§4: `FsCaps`, `KeyProvider` erweitert, `SecretAlloc`; `Config::fs_policy`.|
|E-12|§9.7: Catalog-Felder `0x0012…0x0014`; `spaces_add`-Flags `16` (E-1) und `32` (reserviert).|
|E-13|§15.2: Schlüsselepoche (`key_epoch`) in Run-/Blob-/Segment-Schlüsseln; Klon-Regel (SD-63).|
|E-14|§18/§21: `INV-FS-1`, `INV-KEYP-1`, `INV-HOLD-1`, `INV-SEG-1`, `INV-EPOCH-1`, `INV-SCAN-1`; SD-51…SD-64.|
|E-15|Namen aufsetzender Systeme aus der Spezifikation entfernt; sie bleiben außerhalb des Geltungsbereichs.|

Neue Formatfelder (`0x0012…0x0014`, Segment-Rahmen, Epoche in Schlüsselableitung) stehen auf der Freeze-Liste §9.8 (Punkt 14) und brauchen Goldens vor **G-F** (A.8).

## Anhang A: Referenzvektoren (Goldens)

Berechnet mit einem **unabhängigen Python-Referenzcodierer** (CRC32C und XXH64 vor der Nutzung gegen die Prüfwerte validiert). **Nicht** aus Rust-Code erzeugt; die Rust-Implementierung muss sie byte-genau reproduzieren (Test `golden_anhang_a`). Alle Hexwerte sind Bytefolgen in Dateireihenfolge.

### A.1 Primitive

|Eingabe|Ergebnis|
|---|---|
|`CRC32C("123456789")`, `seed = 0`|`0xE3069283`|
|`CRC32C("")`, `seed = 0`|`0x00000000`|
|`CRC32C("a")`, `seed = 0`|`0xC1D04330`|
|Fortsetzung: `crc32c(crc32c(0,"1234"), "56789")`|`0xE3069283` (= Einzelschritt)|
|`XXH64("", 0)`|`0xEF46DB3751D8E999`|
|`XXH64("a", 0)`|`0xD24EC4F1A98C6E5B`|
|`XXH64("abc", 0)`|`0x44BC2CF5AD770999`|

### A.2 Varint (LEB128, kanonisch)

|Wert|Bytes|
|---|---|
|0|`00`|
|1|`01`|
|127|`7f`|
|128|`80 01`|
|300|`ac 02`|
|16384|`80 80 01`|
|2^32|`80 80 80 80 10`|
|2^64 − 1|`ff ff ff ff ff ff ff ff ff 01`|

Ungültig (`Corrupt`): `80 00` (überflüssige Fortsetzung), `ff×9 02` (zehntes Byte ≠ `0x01`), mehr als 10 Bytes, abgebrochene Folge.

### A.3 Tag

`tag = (stamp << 8) | kind` (u64 LE):

|stamp|kind|Bytes|
|---|---|---|
|1|Put (1)|`01 01 00 00 00 00 00 00`|
|7|Delete (2)|`02 07 00 00 00 00 00 00`|
|2^56 − 1|PutRef (5)|`05 ff ff ff ff ff ff ff`|

Sortierung `(user_key ↑, tag ↓)`: `("k", tag(7,Delete))` steht **vor** `("k", tag(1,Put))`.

### A.4 `FORMAT` (`Plain`, 51 B)

Eingabe: `version = 1`, `required = 0`, `optional = 0`, `integrity_mode = 0`, `instance_id = 00 01 … 0f`, `created_ms = 1_700_000_000_000`.

```
43 54 58 53 | 01 00 | 00×8 | 00×8 | 00 | 00 01 02 03 04 05 06 07 08 09 0a 0b 0c 0d 0e 0f | 00 68 e5 cf 8b 01 00 00 | 56 63 b8 86
"CTXS"        ver     req    opt    mode instance_id (16)                                       created_ms (LE)          crc32c
```

Kontrolle: Offset 22 trägt `integrity_mode`; `crc32c = 0x86B86356` über die 47 vorhergehenden Bytes. Vollständig: `4354585301000000000000000000000000000000000000000102030405060708090a0b0c0d0e0f0068e5cf8b0100005663b886`.

### A.5 Redo-Frame (`Plain`)

Segment mit `chain_seed = 0xA5A5A5A5`; erster Frame: `stamp = 1`, `op_count = 1`, `flags = 0` (Always), ein Op `Put(space=1, key="k", value="v")` ohne Domäne, `header_len = 0`.

```
Op (8 B):       01 | 01 00 | 00 | 01 6b | 01 76
                kind space  opflags klen key vlen value
Frame (29 B):   15 00 00 00 | 01 00 00 00 00 00 00 00 | 01 00 00 00 | 00 | <Op> | c4 8d 63 6f
                len = 13+8 = 21   stamp                  op_count      flags payload  chk = 0x6F638DC4 (seed = chain_seed)
```

Vollständig: `150000000100000000000000010000000001010000016b0176c48d636f`.

**Seal-Frame** (nach diesem einen Frame, `frame_count = 1`, `final_chain = 0x6F638DC4`, `stamp = 1`, Kette setzt bei `chk = 0x6F638DC4` fort):

`190000000100000000000000ffffffff800100000000000000c48d636fdb7083de` (`chk = 0xDE8370DB`).

Ein Frame mit `len = 0` beendet das Segment (vorallokierter Nullbereich).

### A.6 Bloom, `BlobRef`, Nonces

**Bloom** (`nbits = 64`, `k = 7`, Schlüssel `"k"`): `h1 = 0xC3D31922C50B1B63`, `h2 = 0x6CBC77349206F859 | 1`, Bitpositionen `pos_i = (h1 + i·h2) mod 2^64 mod 64`, `i = 0…6`: **35, 60, 21, 46, 7, 32, 57**.

**BlobRef** (28 B): `blob_no = 0x71`, `offset = 4096`, `len = 1000`, `crc32c = 0xDEADBEEF`, `domain = 42`: `71 00 00 00 00 00 00 00 | 00 10 00 00 00 00 00 00 | e8 03 00 00 | ef be ad de | 2a 00 00 00`

**Nonces** (12 B, §15.2):

|Verwendung|Eingabe|Bytes|
|---|---|---|
|Run-Block|`block_offset = 4096`|`01 00 10 00 00 00 00 00 00 00 00 00`|
|Redo-Frame|`frame_index = 3`|`02 03 00 00 00 00 00 00 00 00 00 00`|
|Redo-Op-Körper|`segment_id = 0x12`, `frame = 3`, `op = 1`|`03 12 00 00 00 03 00 00 00 01 00 00`|
|Blob-Record|`offset = 4096`|`04 00 10 00 00 00 00 00 00 00 00 00`|
|Blob-Körper (Domäne)|`blob_no = 0x71`, `offset = 4096`|`05 71 00 00 00 00 00 00 00 10 00 00`|
|Catalog-Record|`record_index = 2`|`02 00 00 00 00 00 00 00 00 00 00 00`|

### A.7 `ResumeToken`

```
"CTXT" | version u8 (1) | space u16 | dir u8 (0 Fwd, 1 Bwd) | stamp u64 | range_hash u64 (XXH64 über kanonische Bereichskodierung, Seed 0)
       | klen varint | last_key | chk
chk = CRC32C (Plain, 4 B) | Mac16 mit Token-Schlüssel (Authenticated/Encrypted, 16 B), über alle vorhergehenden Bytes
```

Dekodierung ist total (P-4); `klen ≤ max_key_bytes`.

### A.8 Offene Goldens (Pflicht vor **G-F**)

Je ein Golden für `Authenticated` und `Encrypted` (Redo-Frame, Run-Block, Blob-Record, Catalog-Record, `KEYS`, `FORMAT` mit 16-Byte-Tag) mit der Referenz-Implementierung aus O-6; ein Run mit DataBlock + Index + Filter + Meta + Directory + Footer (`Plain`); eine Blob-Datei mit Body- und Hole-Record; ein Catalog mit Snapshot und Edit. Ohne diese Goldens ist **G-F** nicht erfüllt (§9.8).

---

## Anhang B: Herkunft und Auditspur

### B.1 Befunde im Altstand (`crates/contextra-store`, 59 788 Zeilen insgesamt, `src/` 32 015, `tests/` 25 563)

|ID|Befund|Antwort in dieser Spezifikation|
|---|---|---|
|B-1|HEAD enthält nur das Crate, kein Root-`Cargo.toml`; Abhängigkeiten nur in der Historie (`014319d`).|Anhang B.2 (Baseline), Plan F0|
|B-2|Domänenlogik im Kern (TxBuffer, SSI, Intent-Locks, Mandanten-Codec, `docid-128`, Scanner/Purger).|NZ-3, NZ-8; `Guard`, `Erase`|
|B-3|Drei Dateisystem-Verträge, 91 direkte FS-Aufrufe am Port vorbei.|P-1, SD-03|
|B-4|`async` überall bei blockierender E/A; achtstufige Sperrhierarchie.|SD-02, §16 (sieben Ränge, Beweise)|
|B-5|Drei Format-Generationen im Lesepfad (WAL V1–V3, SSTable v0–v4, Manifest v0–v2).|SD-04, `INV-FMT-*`|
|B-6|Transaktion als Record-Folge mit `TxEnd`.|`INV-LOG-1`|
|B-7|`rollback_to_tx` ohne produktiven Aufrufer.|SD-09|
|B-8|Manifest als Pfad-String-Log.|SD-05, §9.7|
|B-9|Unkontrollierte Löschung (Fehler verschluckt, kein `fsync`, am Manifest vorbei).|§14, `INV-ERA-*`, P-3|
|B-10|Sechs statische Testschalter (`wal/core.rs`) im Release.|P-5, SD-39|
|B-11|Drei Block-Caches, fester Bloom-Filter (100 000 Elemente).|§11 (ein Cache), §9.5 (dynamische Größe)|
|B-12|Kommentarbanner, gemischte Sprachen.|P-6, P-7|
|B-13|Dateirechte nur teilweise geregelt.|§15.6|
|B-14|Feature-Matrix bricht; Testkorpus nicht übersetzbar.|§19.3|
|B-15|Konsument nutzt `MergeOperator`.|SD-12, A-12|

### B.2 Legacy-Baseline

Rekonstruierbar aus Commit `014319d` mit einem auf 9 Crates reduzierten Workspace-Manifest (`ids, env, sys, mvcc, crypto, types, ports, testkit, store`): `cargo check -p contextra-store --lib` grün (234 Warnungen, überwiegend `missing_docs`); Tests, Benches und Konsumenten nicht baubar. Zweck: Differentialtest und Ausgangswerte. Die Rekonstruktion ist F0-Aufgabe mit Abbruchkriterium (Plan §6).

### B.3 Szenarioklassen des Altbestandes → Netze

|Altklasse|Neues Netz|
|---|---|
|`crash_*` (Kill, Durability)|Absturzmatrix|
|`chaos_bitflip_*`|Bitflip|
|`wal_*` (Replay, Torn Tail, Rotation)|Absturzmatrix (Redo), Fuzz|
|`compaction_*` (Retention, Konflikte)|Gesetze (`INV-RET-1`), Modell|
|`fsync_failure_*`, `disk_full_*`|Fehlerklassen-Matrix F-4/F-5|
|`manifest_*`|Absturzmatrix (Catalog), `INV-CAT-*`|
|`campaign_j21_hypotheses`|Benchmarks (H-S)|
|`compaction_merge_operator`|zurückgestellt (SD-12)|

### B.4 Übernommen aus den Konzeptdokumenten

Aus `MNEMOS_CONCEPT.md`, `Optimal-LLM-Datenbank.md` und `REVIEW_contextra-storage_vs_mnemos.md` (soweit in diesem Dokument genannt, siehe Anhang C): Header-zuerst-Lesen, budgetierte Seitenlieferung, Gewichte, Konsumenten-Cursor, Domänenschlüssel, `CompactionFilter` („Zoom-Down“), Cluster-Präfix und Lokalitätsteilung, Redo-HWM, Siegel-Frames.

---

## Anhang C: Nicht übernommen und Branches

|Konzept|Entscheidung|Begründung|
|---|---|---|
|Prolly-Tree / Merkle-DAG als Hauptstruktur|**zurückgestellt (NZ-12)**|Hash-Inhaltsadressierung widerspricht `Erase` (Z-3) und `forgetting`; obere Schicht|
|Feste 4-KiB-„Context Slabs“|verworfen|Wertgrößen variieren; `block_bytes` genügt|
|Commit ohne Log per CAS auf einen Superblock|verworfen|ein `fsync` je Gruppe ist einfacher und erprobt|
|Thread-per-Core, `io_uring`, `O_DIRECT` im Kern|verworfen (SD-31)|alternative `Fs`-Implementierung|
|2PC über Locality Groups|verworfen|Spaces teilen Redo und Stamp, ein `Change` ist ohnehin atomar|
|Merge-Operatoren|zurückgestellt (SD-12)|Formatentscheidung nötig|

**Branches, Stufe A (Overlay):** Ein Branch ist ein eigener **Overlay-Space** (Schreibvorgänge landen dort), gelesen wird über `View` auf Basis-Space + Overlay (obere Schicht mischt). Der Basisstand wird durch einen `Hold` mit Eigentümer (`HoldOwner::new("branch/a1")`) gehalten; Konflikte beim Zusammenführen prüft `Guard::range_unchanged_since(base_space, range, base_stamp)`; Änderungen am Basisstand seit dem Verzweigen liefert `history`. Verwerfen = `Hold` freigeben + `drop_space(overlay)`. Stufe B (Commit-DAG, Strukturteilung) ist NZ-12 und Sache der oberen Schicht.
---

## Anhang D: Leitplanken für aufsetzende Schichten (nicht Teil des Kerns)

Diese Hinweise sind **Empfehlungen** an Konsumenten des Kerns und erzeugen keine Kern-Anforderung.

1. **Schlüsselcodierung.** Mandant/Kollektion/Dokument werden vom Konsumenten als Bytepräfix codiert; feste, versionierte Präfixe verhindern Kollisionen. Der Kern vergleicht nur Bytes (`prefix_filter`, `locality_prefix`).
2. **Zähler und Gewichte.** Bis `merge()` aktiviert wird: Read-Modify-Write über `Guard::equals`; häufige Akkumulation (Kantenverstärkung, Zugriffszähler) wird in der oberen Schicht gepuffert und gebündelt in **einem** `Change` geschrieben, um `Conflict`-Wiederholungen zu vermeiden.
3. **Flüchtig vs. dauerhaft.** Ein `Change` mischt nie flüchtige und dauerhafte Spaces (`Forbidden{MixedDurability}`); Arbeitsgedächtnis und dauerhafter Zustand werden getrennt eingereicht.
4. **Abgeleitete Indizes** (Volltext, Vektor, Graph) tragen Löschbitmaps oder bauen über `rewrite_without` neu (O-2) und folgen dem Kern über `Consumer` + `history`; `domain_retired` ist ihr Auslöser.
5. **Löschnachweise.** Für Nachweise zählt `ErasureReceipt` + `scan_physical`; Signatur, Journal und Ledger liegen außerhalb.
6. **Fehlerbehandlung.** `Busy` und `Conflict` sind strukturiert; `retryable()` und `hint()` genügen für automatische Wiederholung.


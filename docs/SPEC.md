# SPEC.md — `contextra-storage` (Speicherkern), Revision R7

| | |
|---|---|
| Status | **Normativ (R7).** Einzige normative Spezifikation des Speicherkerns. Ersetzt vollständig `docs/archive/SPEC-R2.md` und `docs/archive/SPEC-R6-Cluster-Entwurf.md` (Herkunft siehe Anhang B). Gate **G-0** (§22) gilt mit dieser Revision als erfüllt. Gate **G-1** setzt O-1 und O-12 voraus. |
| Gegenstand | Eingebetteter, synchroner, crash-sicherer Schlüssel-Wert-Kern für Sprachmodell-Speicher: atomare Mehr-Space-Änderungen, MVCC-Sichten, löschlokale Datenanordnung (Cluster), kryptografische und physische Löschung mit Nachweis. |
| Crates | `contextra-storage` (Kern), `contextra-storage-ports` (Traits, Werttypen, keine Logik). `contextra-crypto-ref` nur als Dev-Abhängigkeit für Referenz-Primitive (O-6). Der bestehende Crate `contextra-ports` gehört nicht zum Kern und wird nicht verwendet (D-02). |
| Sprache | Prosa und Rustdoc deutsch, Bezeichner englisch. |
| Verifikation | **Nichts im Kern ist kompiliert oder gemessen.** Alle Leistungsangaben sind Hypothesen (§21). Die Primitiv-Goldens in Anhang A sind mit einem unabhängigen Python-Skript nachgerechnet (CRC32C, XXH64, Varint, Tag, Bloom). Byte-Goldens der Container-Formate sind vor **G-F** zu erzeugen. |
| Rechtlicher Vorbehalt | Der Kern liefert technische Eigenschaften. Er trifft keine Rechtsaussage zu Löschfristen oder Ausnahmen (O-2). Aussagen über Löschung sind immer an eine Stufe aus §15.2 und an die Prüfung nach §15.11 gebunden. |

---

## 1. Geltungsbereich und Konventionen

### 1.1 Zweck

`contextra-storage` speichert Daten für Sprachmodell-Anwendungen: viele kleine atomare Schreibvorgänge, kurzlebige Sichten, Lesen von Kopfdaten ohne Körperzugriff, budgetierte Ergebnisseiten, Verlauf mit Fortschrittszeigern, flüchtige Arbeitsbereiche und Löschung als Entwurfsprinzip. Der Kern kennt weder Tokenizer, Indexstrukturen, Mandanten, Rechte noch Rechtsgrundlagen. Er liefert Mechanismus, nie Politik (§2.3).

### 1.2 Normsprache

**MUSS / DARF NICHT**: zwingend, Verstoß ist ein Fehler. **SOLL / SOLL NICHT**: Regelfall, Abweichung braucht Begründung im Review. **KANN**: optional.

### 1.3 Dokumentenhierarchie

1. `docs/SPEC.md` (diese Datei): normativ.
2. `AGENTS.md` (Repository-Wurzel): Arbeitskontext. Darf der Spec nie widersprechen.
3. `CLAUDE.md`: Sitzungsumgebung und Arbeitsregeln.
4. `docs/archive/`: Herkunft. Keine Anforderungsquelle.

Eine Vorfassung gilt nie als Quelle. Widersprüche zwischen Dokumenten werden im Widerspruchsregister (§25) mit Entscheidung festgehalten.

### 1.4 Kennungen

`Z-n` Ziel · `NZ-n` Nicht-Ziel · `P-n` Codierregel · `F-n` Fehlerklasse · `AT-n` Angreifer · `AN-nn` Annahme · `SD-nn` Entscheidung (§23) · `D-nn` Entscheidung aus der Konsolidierung R7 (§23.2) · `C-nn` Widerspruch aus den Vorfassungen (§25) · `INV-*` Invariante (§18) · `H-n` Hypothese (§21) · `O-n` offene Entscheidung (§24) · `G-*` Gate (§22) · `ER0…ER7` Schritte der Löschung (§15.5) · `PD1…PD5` Schritte des Packgruppen-Abwurfs (§15.6).

### 1.5 Änderungsprozess

Änderungen an §4, §5, §9, §10, §11, §14, §15, §16 und §18 brauchen zwei Reviewer, aktualisierte Goldens (Anhang A) und, bei Formatbezug, eine Prüfung gegen die Freeze-Liste (§10.12). Reihenfolge immer: erst diese Datei, dann der Code. Nach **G-F** ist jede Formatänderung eine Migration, nie ein Zweitpfad im Kern.

---

## 2. Ziele, Nicht-Ziele, Verantwortungsgrenze

### 2.1 Ziele

| ID | Ziel |
|---|---|
| Z-1 | Atomarität über Spaces, auch nach Absturz: ein `Change` ist ganz oder gar nicht sichtbar. |
| Z-2 | Durabilität vor Sichtbarkeit (Ausnahme `Relaxed`, Opt-in je Aufruf). |
| Z-3 | Snapshot-Sichten ohne Lesesperren; Zeitreise auf Stamp-Ebene mit Halteregeln. |
| Z-4 | Löschung eines Clusters in Zeit unabhängig von der Gesamtgröße des Speichers (ER0–ER2 O(1), H-6). |
| Z-5 | Ablauf einer Packgruppe als O(Dateien)-Operation (H-7). |
| Z-6 | Nachweisbarkeit: `ErasureReceipt`, `verify`, Löschorakel, Ledger mit Kette und Anker. |
| Z-7 | Kopf ohne Körper lesen; budgetierte Seiten ohne Körperzugriff. |
| Z-8 | Race-Freiheit durch Konstruktion, belegt durch Modell, Simulation und `loom`. |
| Z-9 | Begrenzte Ressourcen: Speicher, Deskriptoren, Wartungs-E/A, kein Thread je Instanz. |
| Z-10 | Stabile Verträge nach oben (§15.9) ohne Formatbruch. |
| Z-11 | Flüchtige Arbeitsbereiche ohne Redo-Kosten. |
| Z-12 | Idempotente Wiederholung für Agenten (`IdemToken`, §11.5). |
| Z-13 | Manipulationserkennung auch gegen Angreifer mit Dateizugriff (AT-3), Rollback-Erkennung für Löschungen (AT-5). |
| Z-14 | Vertraulichkeit je Cluster und kryptografische Löschung ohne Datenumschreiben. |
| Z-15 | Fail-closed: schwächere Plattformen und Formatabweichungen werden abgelehnt, nicht stillschweigend akzeptiert. |

### 2.2 Nicht-Ziele

| ID | Nicht-Ziel | Wohin |
|---|---|---|
| NZ-1 | Verteilung, Replikation, Konsens | obere Schicht |
| NZ-2 | Eigene Kryptografie | Ports (§4) |
| NZ-3 | Schemas, Sekundärindizes, SSI, Lesemengen, Transaktionspuffer | obere Schicht, aus `Change`/`Guard`/`View` |
| NZ-4 | Merge-Operatoren (`kind = 6` reserviert, nicht erzeugbar) | spätere Formatentscheidung |
| NZ-5 | Schutz von Prozessspeicher, Swap, Core-Dumps | best-effort (§15.8) |
| NZ-6 | Umordnung über Clustergrenzen | nie |
| NZ-7 | Kompatibilität mit Altformaten im Kern | optionaler Importer |
| NZ-8 | Windows mit voller Durabilität in Version 1 | Port-Implementierung (O-4) |
| NZ-9 | `O_DIRECT`, `io_uring`, `mmap` im Leseweg | alternative `Fs`-Implementierung |
| NZ-10 | Kompression in Version 1 (Feld reserviert) | spätere Formatentscheidung |
| NZ-11 | Inhaltsadressierung und Dedup von Nutzdaten | obere Schicht (O-5) |
| NZ-12 | Tokenizer, Embeddings, ANN, Volltext, Graphalgorithmen | obere Schicht, über §15.9 |
| NZ-13 | Prolly-Tree, Merkle-DAG, Commit-DAG, Branches Stufe B | obere Schicht (Anhang D) |
| NZ-14 | Löschorchestrierung, Berichte, Signaturen, Journal | obere Schicht; der Kern liefert `Eraser`, `ErasureReceipt`, Ledger-Bytes |
| NZ-15 | Schlüsselverwaltung (Tresor, Passphrase, KEK-Beschaffung) | `KeyProvider` |
| NZ-16 | Mandanten-, Rechte-, Lizenzprüfung, Zweckbindung | obere Schicht |
| NZ-17 | `async`/`await`-Fassade, Executor | obere Schicht über `submit` (§8.8) |
| NZ-18 | Benannte Checkpoints über Komponenten, Checkpoint-Registry | obere Schicht |
| NZ-19 | Vergessens-, Zerfalls- und Konsolidierungspolitik | obere Schicht; der Kern liefert `CompactionFilter` (§13.2) |

### 2.3 Verantwortungsgrenze

Der Kern löst Speicherung, Dauerhaftigkeit, Nebenläufigkeit und Löschung vollständig und liefert für alles darüber **Mechanismus statt Politik**. Er verweigert Aufträge bei technischem Hindernis oder bei `Restriction`, entscheidet aber nie, ob gelöscht werden darf oder muss. Was außerhalb liegt, baut der Kern nicht vorsorglich.

---

## 3. Architektur

### 3.1 Workspace

```
Cargo.toml                          Workspace, resolver 3, MSRV und Toolchain-Pin gemäß O-1
crates/contextra-storage-ports/     Traits und Werttypen, keine Logik, keine Laufzeitabhängigkeiten
crates/contextra-storage/           Kern; #![forbid(unsafe_code)]
crates/contextra-crypto-ref/        Referenz-Primitive, nur Dev-Abhängigkeit (O-6)
docs/archive/                       Vorfassungen und Konzepte
```

Der Kern hängt an `contextra-storage-ports`, nie umgekehrt. Implementierer von Prüfsummen, Kryptografie, Plattform-`Fs`, Indexbau und Telemetrie hängen nur am Port-Crate.

### 3.2 Modulschnitt von `contextra-storage`

```
api        Storage, Space, View, Change, Cursor, Hold, Restriction, Consumer, Eraser, Inspector, Pending
engine     committer · memtable · version · views · registry · claims · retention · volatile
maint      flush · compaction · blob_gc · purge · pack_drop · scheduler (Lanes, Governor)
erase      gate · cluster_table · links · protocol · ledger · oracle (Feature testing)
recovery   open-Sequenz, Anker-Prüfung, Redo-Replay
catalog    snapshot · edits · log · rollover · current · reservations
store      run · blob · segment · redo
keys       hierarchy · keys_file · cluster_keys · key_cache · nonce · rotation
codec      varint · tag · checksum · comparator · limits
env        Fs · Clock · Entropy · Governor · Metrics · Anchor · MemFs · RealFs · FaultFs · SimClock
sync       Sperren mit Rang (Ranked<T, RANK>), Generationszähler
```

`std::fs` darf nur in `env/real_fs.rs` vorkommen.

### 3.3 Schichtregeln

```
api ──► engine ──► { store, catalog, erase } ──► { keys, codec, env }
api ──► erase  ──► engine
maint ──► { engine, store, catalog, keys }
keys ──► { codec, env }       store/catalog/redo/blob nutzen nur keys::*-Schnittstellen
testing (Feature) ──► alles; nichts hängt von testing ab
```

`tests/layering.rs` liest alle `use crate::…`-Zeilen und schlägt fehl, wenn eine Kante verletzt wird. CI prüft zusätzlich mit `cargo tree -e normal`, dass Laufzeitabhängigkeiten auf die Allow-Liste (§3.5) beschränkt sind.

### 3.4 Features

- Cargo-Feature **`testing`** (einziges): aktiviert `src/testing/`. Release-API enthält keine Testhaken (Test `no_test_hooks_in_release_api`).
- `cfg(loom)` nur in `src/sync.rs`. `cfg(miri)` ist zulässig.
- Es gibt **kein** Verschlüsselungs-Feature. Primitive kommen über Ports; der Modus steht in `FORMAT` (§10.2).
- Entwicklungsabhängigkeiten (`proptest`, `loom`, `shuttle`, `arbitrary`, `cargo-mutants`) zählen nicht zur Laufzeit.
- Die Testfunktion `testing` darf nicht Krypto-Crates in den Kern ziehen. Das Referenz-Primitiv liegt in `contextra-crypto-ref` (D-05).

### 3.5 Erlaubte Laufzeitabhängigkeiten des Kerns (D-05)

| Crate | Zweck | Bedingung |
|---|---|---|
| `std` | alles | – |
| `contextra-storage-ports` | Traits | Pflicht |
| `zeroize` | Löschen von Schlüsselmaterial | Pflicht für `Secret32` |
| `bytes` | Zero-Copy-Puffer | optional, Registereintrag nötig |
| `crossbeam-skiplist` | lock-freie Memtable (§17.3) | Registereintrag nötig |

Keine Krypto-Crates (`blake3`, `chacha20poly1305`, `aes-gcm-siv`) im Kern. Sie liegen in `contextra-crypto-ref` (Dev) oder in einem Produktiv-Port außerhalb des Kerns. Jede weitere Abhängigkeit braucht einen Eintrag in §23.3 und `cargo-deny` (`advisories`, `bans`, `sources`). Dev-Abhängigkeiten sind frei, aber nie in der Release-Konfiguration.

### 3.6 Codierregeln

| ID | Regel |
|---|---|
| P-1 | Jede Datei hat **einen** Eigentümer-Mechanismus (§9) und wird nur über `Fs` berührt. |
| P-2 | Zustandsänderung = **ein** Catalog-Edit. Wer „zwei Dinge atomar“ braucht, packt sie in einen Edit. |
| P-3 | Reihenfolge ist Vertrag: *Datei dauerhaft (Datei + Verzeichnis) → Edit dauerhaft → erst dann sichtbar → erst dann alte Dateien löschen.* |
| P-4 | Dekodierung ist **total**: jede Byte-Folge ergibt `Ok` oder `Err(Corrupt)`, nie `panic`, nie unbegrenzte Allokation. Längen werden vor Allokation gegen §20.1 und die Restlänge der Datei geprüft. |
| P-5 | Keine globalen veränderlichen Variablen. Prüfpunkte laufen über `Env::hooks()` (Standard: leer). |
| P-6 | Dateien ≤ 400 Zeilen, Funktionen typisch ≤ 60 Zeilen, Modultiefe ≤ 3, keine Kommentarbanner. |
| P-7 | Rustdoc deutsch, Bezeichner englisch. |
| P-8 | **Fehler sind Daten:** jede Fehlerklasse aus §5 hat einen definierten Zustand danach und einen Test. |
| P-9 | Kein Lock über E/A außer den in §17.1 genannten (Committer, Catalog-Edit). Leser und Wartung warten nie auf Plattenzugriffe anderer. |
| P-10 | Kein Lock über Aufrufer-Code: `Weigher`, `Filter`, `CompactionFilter`, `Extension`, `Listener`, `KeyProvider`, `Cipher` laufen ohne gehaltene Sperre. Ausnahme: `Cipher`/`Mac`/`Checksum` im Committer beim Kodieren (reine Funktionen ohne Rückruf). |
| P-11 | Heißpfad-Allokationen sind budgetiert und durch einen zählenden Allokator geprüft (§20.5). |
| P-12 | Geheimnisse verlassen den Prozess nie unverschlüsselt und erscheinen weder in `Debug`, Logs, Metriken noch in `hint()`. |
| P-13 | `#![forbid(unsafe_code)]`; `deny(unwrap_used, expect_used, panic, todo, unimplemented, unreachable, indexing_slicing)` im Produktionscode; `let _ =` auf `sync_*`, `rename`, `remove_file`, `truncate` ist verboten. |
| P-14 | Zahlen und Offsets werden mit `checked_*` und `try_from` verarbeitet. `as`-Umwandlungen zwischen Breiten sind per Lint verboten. |

---

## 4. Ports (`contextra-storage-ports`, semver-stabil)

```rust
pub enum Perm { Private /* 0600 Dateien, 0700 Verzeichnisse oder restriktive ACL */, Inherit }
pub enum OpenMode { Read, ReadWrite, CreateNew(Perm) }
pub enum LockMode { Exclusive, Shared }                           // nicht blockierend

pub trait Fs: Send + Sync + 'static {
    fn create_dir_all(&self, p: &Path, perm: Perm) -> io::Result<()>;
    fn open(&self, p: &Path, mode: OpenMode) -> io::Result<Box<dyn File>>;
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;   // atomar (AN-04), wirksam erst nach sync_dir
    fn remove_file(&self, p: &Path) -> io::Result<()>;
    fn remove_dir(&self, p: &Path) -> io::Result<()>;
    fn list_dir(&self, p: &Path) -> io::Result<Vec<DirEntry>>;     // Name + Länge
    fn sync_dir(&self, p: &Path) -> io::Result<()>;
    fn lock(&self, p: &Path, mode: LockMode) -> io::Result<Box<dyn LockGuard>>;
    fn caps(&self) -> FsCaps;                                      // zugesicherte Eigenschaften (INV-FS-1)
}
pub trait File: Send + Sync {
    fn read_at(&self, buf: &mut [u8], off: u64) -> io::Result<usize>;
    fn read_batch(&self, reqs: &mut [ReadReq]) -> io::Result<()> { /* Standard: Schleife über read_at */ }
    fn write_at(&self, buf: &[u8], off: u64) -> io::Result<()>;   // vollständiges Schreiben oder Fehler
    fn truncate(&self, len: u64) -> io::Result<()>;
    fn preallocate(&self, len: u64) -> io::Result<()> { /* Standard: Nullen in 1-MiB-Schritten */ }
    fn punch_hole(&self, off: u64, len: u64) -> io::Result<()> { Err(Unsupported) }   // optional, Standard: Nullschreiben
    fn len(&self) -> io::Result<u64>;
    fn sync_data(&self) -> io::Result<()>;
    fn sync_all(&self) -> io::Result<()>;
}
pub struct FsCaps { pub durable_file_sync: bool, pub durable_dir_sync: bool,
                    pub atomic_rename: bool, pub private_perm: bool, pub no_follow_root: bool }
pub trait Clock:   Send + Sync { fn now_ms(&self) -> u64; fn monotonic_ns(&self) -> u64; }
pub trait Entropy: Send + Sync { fn fill(&self, dst: &mut [u8]) -> io::Result<()>; }
pub trait Checksum: Send + Sync { fn crc32c(&self, seed: u32, data: &[u8]) -> u32; }   // Standard: portabel, safe
pub trait Hasher: Send + Sync { fn hash256(&self, parts: &[&[u8]]) -> [u8; 32]; }    // unkeyed, nur Ledger-Kette ohne Schlüssel (D-12)
pub trait Metrics: Send + Sync { fn counter(&self, n: &'static str, v: u64);
                                 fn gauge(&self, n: &'static str, v: i64);
                                 fn observe(&self, n: &'static str, v: f64); }
pub trait Observer: Send + Sync { fn event(&self, e: ObserverEvent); }   // nie unter Sperren aufrufen (§17.3)
pub trait Hooks: Send + Sync { fn point(&self, p: HookPoint) {} }        // Standard: leer, nur Tests
pub trait PressureSource: Send + Sync { fn level(&self) -> Pressure; }   // optional (cgroup/PSI)

// Kryptografie: Primitive im Port, Protokoll im Kern (SD-36)
pub trait Cipher: Send + Sync {                                          // AEAD, 32-Byte-Schlüssel, 24-Byte-Nonce, 16-Byte-Tag
    fn seal(&self, key: &Key32, nonce: &[u8; 24], aad: &[u8], plain: &[u8]) -> Vec<u8>;
    fn open(&self, key: &Key32, nonce: &[u8; 24], aad: &[u8], sealed: &[u8]) -> Result<Vec<u8>, CryptoError>;
}
pub trait Mac: Send + Sync { fn mac16(&self, key: &Key32, parts: &[&[u8]]) -> [u8; 16];
                             fn verify16(&self, key: &Key32, parts: &[&[u8]], tag: &[u8; 16]) -> bool; } // konstante Zeit
pub trait Kdf: Send + Sync { fn derive(&self, key: &Key32, context: &[&[u8]]) -> Key32; }

pub trait KeyProvider: Send + Sync {                                     // siehe §16
    fn kek(&self) -> Result<Secret32, KeyError>;                         // umhüllt nur KEYS
    fn cluster_key(&self, c: ClusterId, epoch: u32) -> Result<Option<Secret32>, KeyError>; // None = vernichtet
    fn create_cluster_key(&self, c: ClusterId, e: &dyn Entropy) -> Result<u32, KeyError>;  // liefert epoch, durabel bei Rückkehr
    fn destroy(&self, c: ClusterId) -> Result<(), KeyError>;             // alle Epochen; durabel bei Rückkehr; idempotent (D-07)
    fn probe(&self, c: ClusterId) -> Result<KeyState, KeyError>;         // Present | Absent
    fn make_durable(&self) -> Result<(), KeyError>;                      // Barriere für create_cluster_key (INV-KEYP-1)
}
pub trait SecretAlloc: Send + Sync { fn alloc(&self, bytes: [u8; 32]) -> Result<Box<dyn SecretCell>, KeyError>; } // optional (SD-56)
pub trait SecretCell: Send + Sync { fn expose(&self) -> &[u8; 32]; }                                                // Drop überschreibt

pub trait Anchor: Send + Sync {                                          // Rollback-Anker, §16.7
    fn read(&self) -> Result<Option<AnchorValue>, AnchorError>;
    fn advance(&self, to: AnchorValue) -> Result<(), AnchorError>;       // monoton; durabel bei Rückkehr
}
pub trait ColdTier: Send + Sync {                                        // Phase P6, optional
    fn upload(&self, f: FileRef) -> io::Result<()>;
    fn read_range(&self, f: FileRef, off: u64, len: u32) -> io::Result<Vec<u8>>;
    fn delete(&self, f: FileRef) -> io::Result<()>;
}

// Aufrufer-Erweiterungen (§8.5, §13.2, §10.9)
pub trait Weigher: Send + Sync { fn version(&self) -> u32; fn weigh(&self, key: &[u8], header: &[u8], out: &mut [u64; 4]); }
pub trait Filter: Send + Sync { fn keep(&self, key: &[u8], header: &[u8]) -> bool; }
pub trait CompactionFilter: Send + Sync { fn decide(&self, key: &[u8], header: &[u8], stamp: Stamp, now_ms: u64) -> Decision; }
pub trait ExtensionBuilder: Send + Sync { fn id(&self) -> u16; fn version(&self) -> u16;
    fn build(&self, entries: &mut dyn Iterator<Item = Entry>) -> Result<Vec<u8>, ExtError>;
    fn merge(&self, inputs: &[&[u8]], survivors: &mut dyn Iterator<Item = Entry>) -> Result<Vec<u8>, ExtError>;
    fn rewrite_without(&self, block: &[u8], drop: &ClusterSet) -> Result<Vec<u8>, ExtError>; }   // Pflicht (§15.9)
```

**Verträge.**

- `write_atomic(path, bytes)` (`.building.<tmp_id>` → `sync_all` → `rename` → `sync_dir`) ist Hilfsfunktion über `Fs`, kein Trait-Mitglied, damit `FaultFs` sie mitprüft.
- `Perm::Private` gilt unabhängig von der `umask` (§16.6).
- `RealFs` nutzt ausschließlich `std`.
- `Checksum::crc32c(seed, data)` setzt einen CRC-32C-Strom fort: Ergebnis = `!update(!seed, data)`; `seed = 0` liefert den üblichen CRC-32C (`"123456789"` → `0xE3069283`). Beschleunigte Varianten über denselben Port müssen die Goldens aus Anhang A bestehen (SD-64).
- **`FsCaps` und Fail-closed (INV-FS-1).** Unter `FsPolicy::Strict` (Standard) öffnet der Kern nur, wenn `durable_file_sync`, `durable_dir_sync`, `atomic_rename` und `private_perm` wahr sind; sonst `Err(Unsupported{fs_caps})`. `Relaxed` ist nur für Tests und Wegwerf-Instanzen und setzt `Receipt.durable = false`. Neue `Fs`-Implementierungen bestehen `testing::conformance::fs_suite`.
- **`KeyProvider`-Vertrag (INV-KEYP-1).** (1) `create_cluster_key` legt an, wenn ein Cluster erstmals beschrieben wird. (2) `make_durable` kehrt erst zurück, wenn alle bis dahin angelegten Schlüssel dauerhaft sind; der Kern ruft es vor dem Dauerhaftwerden eines Frames mit so versiegelten Körpern (§8.2, Schritt 1b). (3) `None` darf nur zurückkommen, wenn die Vernichtung dauerhaft und geprüft ist. (4) Aufrufe können blockieren und laufen ohne gehaltene Sperre (P-10). (5) Der Kern hält Clusterschlüssel nur im begrenzten Cache (§16.4).
- **`Secret32`:** undurchsichtiges Handle ohne `Debug`, `Clone`, `Serialize`. `Drop` überschreibt per `zeroize`. `SecretAlloc` ersetzt den Speicher durch geschützten Speicher der Anwendung, ohne dass der Kern `unsafe` braucht (SD-56).
- `Mac::mac16` und `Cipher` hängen **keine** Länge an; der Kern kodiert Längen selbst.
- `Weigher::version()` ändert sich, wenn sich das Zählverfahren ändert; der Kern speichert sie je Run.
- `Env` bündelt alle Ports. Immer belegt: `fs, clock, entropy, checksum, hasher, runtime, governor, metrics, observer, hooks`. Optional: `cipher, mac, kdf, key_provider, secret_alloc, anchor, cold_tier, pressure`. `Env::real(root)` liefert die Standardbelegung ohne Kryptoprimitive; ein `Integrity`-Modus, der sie braucht, ohne sie zu liefern, schlägt beim Öffnen mit `Config` fehl.
- `Runtime` ist ein teilbarer Thread-Pool (`spawn(Box<dyn FnOnce + Send>)`). `Runtime::manual()` führt Jobs nur auf `run_pending()` aus und macht jede Verschränkung in Tests deterministisch.

---

## 5. Fehler- und Bedrohungsmodell

### 5.1 Fehlerklassen

Die Klassen F-1 bis F-16 vereinen die Vorfassungen (C-20). Jede Klasse hat ein definiertes Verhalten, einen definierten Zustand danach und einen Test (§19.1).

| ID | Fehlerklasse | Verhalten | Zustand danach | Prüfnetz |
|---|---|---|---|---|
| F-1 | Stromausfall, Prozess-Kill | Recovery (§14) | Präfix der quittierten Changes; jede Quittung mit `durable = true` bleibt | Absturzmatrix |
| F-2 | Zerrissener Schreibvorgang | Gruppen-, Frame-, Block-, Recordprüfwert | Torn Tail im **letzten** Segment wird geheilt; sonst `Corrupt` | Absturzmatrix, Fuzz |
| F-3 | Umordnung unsynchronisierter Schreibvorgänge | Reihenfolge-Vertrag (P-3) | nie ein Edit vor seiner Datei | `FaultFs` |
| F-4 | `fsync`-Fehler zur Laufzeit | **Instanz-Poison**, kein Retry (§11.6) | Schreiben und Wartung tot; Lesen bleibt (§11.6) | `FaultFs` |
| F-5 | `ENOSPC` beim Anhängen | Abschneiden auf den letzten guten Offset, Gruppe `Busy{StorageFull}`; schlägt das Abschneiden fehl, Instanz-Poison | heil und wiederholbar oder Poison | `FaultFs` |
| F-6 | `EIO` beim **Lesen** | immer als Fehler melden, **nie als „leer“**; Datei wird als verdächtig markiert | Operation scheitert; Instanz benutzbar | `FaultFs` |
| F-7 | Bitfäule im Ruhezustand | Prüfwert je Block, Frame, Record; in `Authenticated`/`Encrypted` kryptografisch | `Corrupt{file, offset}` und **Datei-Quarantäne** (§5.3); nie falsche Daten | Bitflip |
| F-8 | Verlorener oder fehlgeleiteter Schreibvorgang | Kettenprüfwert im Redo, AAD mit Block-ID, Redo-HWM (INV-LOG-8), `verify` | erkannt, solange Dateien intern inkonsistent sind; ein ganzer älterer, in sich stimmiger Stand bleibt unerkannt (→ F-10) | `FaultFs` |
| F-9 | Speicherdruck, Deskriptorerschöpfung | `Governor`: verlangsamen, Flush erzwingen, Caches verkleinern; `Busy{retry_after}` | keine Allokationsabbrüche; kein OOM | Ressourcentests |
| F-10 | Zurückspielen eines älteren Gesamtstands | nur mit Rollback-Anker (§16.7) erkennbar | ohne Anker nicht erkennbar; offen dokumentiert | Anker-Test |
| F-11 | Uhrsprung | `Clock` nur für Ablauf (Holds, TTL, Intervall-`fsync`); Ordnung nur über Stamps | nie Ordnungsfehler | Clock-Tests |
| F-12 | Panik in Aufrufer-Code | `catch_unwind` an der Grenze in Wartung und Listenern | Job scheitert und wird gemeldet; Daten unverändert | Panik-Injektion |
| F-13 | Sperre vergiftet (`PoisonError`) | `Poisoned{cause: LockPoison}` | Instanz tot, Neustart | Panik-Injektion |
| F-14 | Nonce- oder ID-Wiederverwendung nach Absturz | ID-Reservierung im Catalog **vor** Nutzung (INV-KEY-2) | keine ID je mit anderem Inhalt wiederverwendet | Property, Absturzmatrix |
| F-15 | Downgrade des Integritätsmodus | `FORMAT` wird gegen `Config.integrity` geprüft (§10.2) | Öffnen verweigert, bevor andere Dateien gelesen werden | Manipulationstest |
| F-16 | Entfernen oder Umbenennen einer lebenden Datei durch Dritte | Catalog-Abgleich, fail-closed (`MissingFile`) | Öffnen verweigert, Verfügbarkeit nicht geschützt | Manipulationstest |

### 5.2 Angreifermodell

| ID | Angreifer | Fähigkeit | Verteidigung | Grenze |
|---|---|---|---|---|
| AT-1 | Anderer lokaler Benutzer im selben Host | liest Dateien, wenn Rechte es zulassen | `Perm::Private`, eigenes Verzeichnis je Instanz | Root, Hypervisor |
| AT-2 | Offline-Besitzer des Datenträgers (Diebstahl, Backup) | liest Dateien | `Encrypted`: Vertraulichkeit und Integrität | Rollback ganzer Stände (AT-5) |
| AT-3 | Offline-Manipulator ohne Schlüssel | Bits kippen, Frames und Blöcke einfügen, vertauschen, abschneiden | `Authenticated`/`Encrypted`: Prüfwerte mit AAD, Datei-MAC (§10.6) | Löschen ganzer Dateien (F-16) |
| AT-4 | Fehlerhafte oder bösartige obere Schicht | ruft die API unzulässig auf, fälscht `ResumeToken` | typisierte Handles (§16.5), Limits, MAC-gesicherte Tokens | Vertrauen in die einbettende Anwendung |
| AT-5 | Rollback des Verzeichnisses auf früheren, konsistenten Stand | macht gelöschte Daten wieder sichtbar | Rollback-Anker (§16.7) für Löschungen | Rollback innerhalb des Ankerintervalls (§16.7) |
| AT-6 | Prozess mit Speicherzugriff (Core-Dump, Swap) | liest Schlüssel und Klartext | nicht kontrollierbar; Bereinigung best-effort | §15.8 |

**Konsequenz.** Ohne `Authenticated`/`Encrypted` schützt CRC32C nur gegen Zufallsfehler, nicht gegen Angreifer. Das steht in der Dokumentation und im Feld `integrity_mode`.

### 5.3 Quarantäne

Eine Prüfwertverletzung in einer Datei **vergiftet nicht die Instanz**. Der Kern markiert die kleinste betroffene Einheit (Block, Abschnitt, Datei) als `Quarantined`, liefert `Corrupt{file, offset, what}` für Zugriffe darauf, und der Rest der Instanz bleibt benutzbar. Der Begriff „Poison“ ist ausschließlich dem Schreibpfad nach E/A-Fehlern vorbehalten (§11.6). Kein Fehler wird als „leer“ gemeldet.

---

## 6. Datenmodell

### 6.1 Vokabular

| Begriff | Bedeutung |
|---|---|
| **Storage** | Handle einer geöffneten Instanz |
| **Space** | Benannter, physisch getrennter LSM-Baum (`SpaceId(u16)`), dauerhaft oder `Volatile` |
| **Cluster** | Löschdomäne, Schlüsseldomäne und Platzierungseinheit (`ClusterId`, 16 Byte, opak). `ClusterId::NONE` = „ohne Cluster“, nur in nicht geclusterten Spaces. Nur über `Clusters::register` angelegt. |
| **Packgruppe** | Cluster gleichen Mandanten-Tags, Ablaufzeitraums und Shards: `PackKey = (tenant_tag, due_bucket, shard)` |
| **Change / Guard / Receipt** | Atomare Menge von Schreibvorgängen / Vorbedingung / Quittung `(stamp, durable)` |
| **Stamp** | Monoton steigende Nummer angenommener Changes (`u64`, `< 2^56`); Transaktionszeit |
| **View / Cursor / Page** | Unveränderliche Sicht / Streaming-Iterator / budgetierte Seite mit `ResumeToken` |
| **Hold** | Befristete technische Halteregel gegen Verwerfen alter Versionen, mit Eigentümer |
| **Restriction** | Rechtlicher Sperrvermerk mit Pflichtgrund (§15.4). Kein technisches Halten. |
| **Consumer** | Persistenter Fortschrittszeiger eines abgeleiteten Index; zählt wie ein Hold, mit Maximalrückstand |
| **Redo-Log** | Segmentiertes Wiederholungsprotokoll dauerhafter Spaces |
| **Run / Blob** | Unveränderliche sortierte Datei mit Köpfen und Verweisen / unveränderliche Datei mit Wertkörpern |
| **Header / Body / Weights** | Erste `header_len` Wertbytes / Rest / Zählgrößen im Klartext |
| **Pure / Packed** | Datei mit einem Cluster / mit mehreren Clustern einer Packgruppe (§7) |
| **Segment** | Vom Catalog verwaltete Datei einer oberen Schicht (§15.9) |
| **Claim / Graveyard** | Reservierung von Dateien durch einen Job / Warteschlange zu löschender Dateien |
| **Catalog** | Einzige Wahrheit über lebendige Dateien und Zustand |
| **Anchor** | Monotoner Rollback-Anker außerhalb des Datenverzeichnisses (§16.7) |
| **Quarantäne** | Markierung einer beschädigten Einheit ohne Instanz-Poison (§5.3) |
| **Env** | Bündel der Ports (§4) |

### 6.2 Cluster, Schlüssel, Reihenfolge, Stamp

- **Nutzerschlüssel:** Bytes, `1 ≤ Länge ≤ max_key_bytes`. In `Seal::Cluster`-Spaces geschützt (§10.10). Sonst MUSS er frei von personenbezogenen Daten sein (§15.9).
- **Vergleich** (`cmp`, kein `memcmp` über Konkatenation): Cluster aufsteigend, dann Nutzerschlüssel bytelexikografisch aufsteigend, dann **Tag absteigend** (neueste Version zuerst).
- **Tag** `u64 = (stamp << 8) | kind`. `kind`: `Put = 1`, `Delete = 2`, `RangeDelete = 3`, `EraseRange = 4`, `PutRef = 5`, `Merge = 6` (**reserviert**; Decoder meldet `Unsupported`).
- `PutRef` kommt nur in Runs vor. Redo und Memtable kennen `Put`, `Delete`, `RangeDelete`, `EraseRange`. Der Löschmarker ist ein eigenes Typ-Byte, kein Bit im Stamp.
- **Stamp:** erster Stamp ist 1, `0` bedeutet „leer“. Streng monoton über **angenommene** Changes (INV-STAMP-1). **Lücken sind zulässig:** Stamps, die ein Absturz vor der Quittung verbraucht hat, verfallen (INV-STAMP-3). Stamps werden nie wiederverwendet. Überlauf ⇒ `Exhausted` (INV-STAMP-2).
- **Bereichsmarker** (`RangeDelete`, `EraseRange`): `(cluster, start, end, tag)`, `end` exklusiv. Flags `UNBOUNDED_END`, `WHOLE_CLUSTER`. Ein Marker mit Stamp `S` verdeckt für jede Sicht mit Stamp ≥ `S` alle Versionen mit Stamp < `S` im Bereich.

### 6.3 Wert, Kopf, Körper, Gewichte

Ein Wert besteht aus `header ‖ body`, getrennt durch `header_len` (je Space fest, 0…255 Byte; Obergrenze `max_header_bytes`). Der Kern interpretiert den Kopf nie und liefert ihn ohne Körper-E/A.

- Jeder Eintrag trägt `weights: [u32; W]`, `W = weight_slots` des Space (0…4). Gewichtsbedeutung bestimmt die obere Schicht.
- **Summen sind `u64`** (D-08, C-13): Block- und Clusterverzeichnis führen `weight_sum: [u64; W]`. Eine Summe aus `u32`-Einträgen kann in einem Block überlaufen.
- Körper mit Länge ≥ `value_separation_min` (Standard 1 KiB) liegen in Blob-Dateien (`PutRef`). Kleinere Körper liegen inline im Run. Der Kopf liegt **immer** inline.
- `value_separation_min` ist der einzige Schwellwert für die Körpertrennung (C-06).

### 6.4 `SpaceOptions` (unveränderlich nach dem Anlegen)

```rust
pub struct SpaceOptions {
    pub durability_class: SpaceClass,       // Durable | Volatile (§10.8)
    pub clustered: bool,                    // true: Ordnung, Abschnitte und Löschung nach Cluster
    pub packing: PackPolicy,                // None | ByTenantAndExpiry { bucket: Month | Quarter | Year }; nur bei clustered
    pub pure_threshold: u64,                // Cluster ab dieser Größe wird bei Compaction Pure (Standard 8 MiB)
    pub seal: Seal,                         // None | Cluster; Cluster nur bei clustered + Durable + Integrity::Encrypted
    pub weight_slots: u8,                   // 0..=4
    pub history: HistoryPolicy,             // KeepLatest | KeepUntilFloor | AppendOnly
    pub forgetting: bool,                   // erlaubt CompactionFilter (§13.2); unvereinbar mit AppendOnly
    pub value_separation_min: u32,          // 1 KiB … max_body_bytes
    pub max_key_bytes: u32,
    pub prefix_filter: Option<u8>,          // Präfixlänge 1…64; Standard für Haupt-Spaces (Anhang D)
    pub locality_prefix: Option<u8>,        // Compaction teilt bevorzugt an Präfixgrenzen
    pub compression: Compression,           // v1: nur None (NZ-10)
    pub durability_default: Durability,     // Durable | Relaxed; bei Volatile ohne Wirkung
}
```

- `Seal::Cluster` setzt `clustered = true` voraus. `EraseLevel::Cryptographic` ist nur in Spaces mit `Seal::Cluster` zulässig, sonst `Err(Unsupported)`.
- `forgetting` und `AppendOnly` schließen sich aus (`Config`-Fehler).
- **Volatile** (§11.2) ist ein eigener Space-Typ. Er vermischt sich nie mit dauerhaften Zielen in einem Change: `Forbidden{MixedDurability}` (D-10, C-07). Ein Opt-in für Mischung gibt es nicht.

### 6.5 Change, Guard, Receipt

- Ein `Change` enthält Operationen (`put`, `delete`, `delete_range`) über beliebige Spaces und beliebig viele Guards. Doppelte Schlüssel innerhalb eines Change werden beim Aufbau zusammengelegt (letzter gewinnt). Ein Change erhält **einen** Stamp.
- Guard-Varianten: `Absent`, `Version(Stamp)` (Schlüssel unverändert seit Stamp), `Unchanged(Stamp)`, `RangeUnchangedSince(range, Stamp)`, `Equals(value)`. Sie werden unter der Schreibsperre gegen den neuesten Stand einschließlich früherer Changes derselben Gruppe geprüft (§11.2). `RangeUnchangedSince` arbeitet über `max_stamp` je Block und ist O(überlappende Blöcke).
- `Receipt { stamp, durable }`. `durable = true` heißt: der Frame ist per `sync_data` gesichert und seine Vorgänger im Log ebenfalls.
- `IdemToken` (16 Byte, vom Aufrufer gezogen) macht den Change wiederholungssicher (§11.5).

### 6.6 Konfiguration (Standardwerte, vollständig in §20)

Die Standardwerte stehen in §20.2. Ein Parameter ohne Standard (`purge_deadline`, `integrity`) MUSS beim Aufruf oder in `Config` gesetzt werden.

### 6.7 Cluster-Tabelle

Je Cluster führt der Catalog: `{ cluster_id, tenant_tag, pack_key, state, key_epoch, created_stamp, due_bucket, links, restrictions }`.

`state`: `Live → Retired(S) → [Shredded] → Purged` (nur vorwärts). `Shredded` entfällt, wenn kein Schlüssel vernichtet wird (`Seal::None` oder Stufe ohne Schlüsselvernichtung). `Restricted` ist keine Zustandsstufe, sondern eine Menge `restrictions` (§15.4). Ein `Purged`-Cluster wird nie wiederverwendet (INV-CLUSTER-1).

### 6.8 Limits

| Limit | Standard | Obergrenze | Bei Überschreitung |
|---|---|---|---|
| `max_key_bytes` | 1 KiB | 4 KiB | `Limit` |
| `max_header_bytes` | 4 KiB | 64 KiB | `Limit` |
| `max_body_bytes` | 16 MiB | 256 MiB | `Limit` (kein Stückeln) |
| `max_change_ops` | 4 096 | 65 536 | `ChangeTooLarge` (INV-LIM-1) |
| `max_change_bytes` | 8 MiB | 64 MiB | `ChangeTooLarge` (INV-LIM-1) |
| `max_page_bytes` | 1 MiB | 16 MiB | `Limit` |

Ein Change über den Limits wird **vor** jeder Sperre abgewiesen (§11.1).

---

## 7. Platzierung: Pure und Packed

### 7.1 Regeln

- **R-PL-1** Jeder Run, jeder Blob und jedes Segment ist **Pure** (ein Cluster) oder **Packed** (mehrere Cluster derselben Packgruppe), nie beides (INV-PL-1).
- **R-PL-2** In einer Packed-Datei bildet jeder Cluster **einen zusammenhängenden Abschnitt** (`ClusterSection`), sortiert nach `ClusterId`. Blöcke sind clusterrein (INV-PL-2).
- **R-PL-3** Überschreiten die lebenden Bytes eines Clusters `pure_threshold`, wird er bei der nächsten Compaction Pure. Der umgekehrte Weg findet nicht statt.
- **R-PL-4** Compaction mischt nur innerhalb **einer Wartungsgruppe**: einer Packgruppe oder eines Pure-Clusters (INV-PL-3).
- **R-PL-5** `PackKey = (tenant_tag, due_bucket, shard)`. Packgruppen je Space ≤ `max_pack_groups` (4 096).
- **R-PL-6** Der Wechsel in eine andere Packgruppe (`rebucket`) ist ein atomarer Umzug (§15.7, INV-PL-4).
- **R-PL-7** Blob-Dateien folgen denselben Regeln wie Runs (Pure oder Packed).

### 7.2 Wirkung auf die Löschung

| Fall | Pure | Packed |
|---|---|---|
| Cluster löschen | Dateien **entfernen**, ein Edit | Abschnitt **überspringen**, übrige Abschnitte **roh kopieren** (ohne Entschlüsseln), neues Verzeichnis und neuer Datei-MAC, ein Edit |
| Packgruppe läuft ab | alle Dateien der Gruppe entfernen, ein Edit | dieselbe Operation |
| Mandant verlässt | alle Pure-Cluster und Packgruppen des Tags nacheinander abwerfen | Packgruppen einzeln |

Rohkopieren ist gültig, weil die Block-AAD nicht an die Dateiposition gebunden ist (§10.10). Die Datei-Integrität deckt stattdessen der Datei-MAC über Verzeichnis und Abschnittsindex ab (§10.6).

---

## 8. Öffentliche API

Normativ in Form und Semantik. Lebensdauern sind Implementierungsfrage.

### 8.1 Öffnen und Struktur

```rust
pub struct Storage { /* Arc<Inner> */ }
impl Storage {
    /// LOCK, FORMAT-Prüfung, Anker-Prüfung, Recovery. Modus ReadWrite | ReadOnly (geteilte Sperre).
    pub fn open(cfg: Config, env: Env) -> Result<Storage, OpenError>;
    pub fn space(&self, name: &str) -> Result<Space, Error>;
    pub fn spaces_add(&self, name: &str, o: SpaceOptions) -> Result<Space, Error>;
    pub fn admin_cap(&self) -> Option<AdminCap>;          // genau einmal je Storage abrufbar, nie persistiert (§16.5)
    pub fn close(self) -> Result<CloseReport, Error>;
}
pub struct Config {
    pub dir: PathBuf,
    pub integrity: Integrity,               // PFLICHT: Plain | Authenticated | Encrypted (F-15)
    pub anchor: AnchorPolicy,               // Required | Optional { acknowledged_residual_risk } (§16.7)
    pub fs_policy: FsPolicy,                // Strict (Standard) | Relaxed (INV-FS-1)
    pub limits: Limits,
}
```

### 8.2 Schreiben

```rust
pub struct Change { /* Ops + Guards */ }
impl Change {
    pub fn new(idem: IdemToken) -> Change;
    pub fn put(&mut self, sp: &Space, c: ClusterId, key: &[u8], v: NewValue) -> &mut Self;
    pub fn delete(&mut self, sp: &Space, c: ClusterId, key: &[u8]) -> &mut Self;
    pub fn delete_range(&mut self, sp: &Space, c: ClusterId, r: KeyRange) -> &mut Self;
    pub fn guard(&mut self, sp: &Space, c: ClusterId, key: &[u8], g: Guard) -> &mut Self;
    pub fn durability(&mut self, d: Durability) -> &mut Self;     // Durable | Relaxed
}
pub struct NewValue { pub header: Bytes, pub body: Bytes, pub weights: [u32; 4] }
impl Storage {
    pub fn apply(&self, c: Change) -> Result<Receipt, ApplyError>;
    pub fn submit(&self, c: Change) -> Pending;                    // §8.8
    pub fn sync(&self) -> Result<(), Error>;                       // erzwingt Durabilität aller quittierten Changes
}
pub enum ApplyError {
    GuardFailed { guard_index: u32 }, AlreadyApplied { stamp: Stamp },
    ClusterRetired, ClusterShredded, Restricted { ids: Vec<u64>, reasons: Vec<u16> },
    ChangeTooLarge, Limit { what: LimitKind, max: u64 }, MixedDurability, Forbidden { space: SpaceId, op: OpKind },
    Busy { retry_after: Duration, reason: BusyReason },           // StorageFull, Governor, Stall
    Poisoned { cause: PoisonCause }, Indeterminate,               // §11.6
    Exhausted, Unsupported { feature: &'static str }, Config { what: &'static str },
}
```

### 8.3 Lesen

```rust
impl Storage {
    pub fn view(&self) -> View;                                     // neueste sichtbare Sicht
    pub fn view_durable(&self) -> View;                             // nur Stamps ≤ durable_stamp (INV-CONS-1)
    pub fn view_at(&self, s: Stamp) -> Result<View, Error>;         // nur ≥ retention_floor; sonst HistoryPruned
}
impl View {
    pub fn stamp(&self) -> Stamp;
    pub fn get_header(&self, sp: &Space, c: ClusterId, key: &[u8]) -> Result<Option<HeaderRef>, ReadError>;
    pub fn get(&self, sp: &Space, c: ClusterId, key: &[u8]) -> Result<Option<Entry>, ReadError>;  // Körper lazy
    pub fn get_many(&self, sp: &Space, c: ClusterId, keys: &[&[u8]]) -> Result<Vec<Option<Entry>>, ReadError>;
    pub fn cursor(&self, sp: &Space, c: ClusterId, r: KeyRange, o: ScanOpts) -> Cursor;
    pub fn page(&self, sp: &Space, c: ClusterId, r: KeyRange, b: Budget, resume: Option<ResumeToken>)
        -> Result<Page, ReadError>;                                  // summiert weight_sum, überspringt Blöcke
    pub fn estimate(&self, sp: &Space, c: ClusterId, r: KeyRange) -> Result<Estimate, ReadError>;  // §12.5
    pub fn history(&self, sp: &Space, c: ClusterId, key: &[u8], from: Stamp) -> Result<History, ReadError>;
}
impl Entry {
    pub fn header(&self) -> &[u8];                                  // ohne Körper-E/A
    pub fn weights(&self) -> &[u32];
    pub fn body(&self) -> Result<Bytes, BodyError>;                 // Err(Shredded | Erased | Quarantined | Corrupt)
}
```

- `Budget` begrenzt nach Gewicht (`weight(k, max)`), Bytes und Einträgen. `Page.oversized = true`, wenn der erste Eintrag allein das Limit überschreitet; er wird dann allein geliefert (Fortschrittsgarantie).
- `body()` unterscheidet `Shredded` (Schlüssel vernichtet) von `Erased` (Logische Löschung unter Stamp `S`). Beide liefern nie Daten und nie „leer“.

### 8.4 Cluster, Verknüpfung, Holds, Restrictions, Consumer

```rust
impl Storage {
    pub fn clusters(&self) -> Clusters;
    pub fn hold(&self, at: Stamp, owner: OwnerId, ttl: Duration) -> Result<Hold, Error>;   // ttl ≤ max_hold_ttl
    pub fn restrict(&self, c: ClusterId, r: RestrictionSpec) -> Result<RestrictionId, Error>;
    pub fn lift(&self, id: RestrictionId, by: OwnerId) -> Result<(), Error>;
    pub fn consumer(&self, id: ConsumerId, from: Stamp, max_lag: u64) -> Result<Consumer, Error>;
    pub fn subscribe(&self, l: Box<dyn Listener>) -> Result<ListenerId, Error>;               // §8.6
}
impl Clusters {
    pub fn register(&self, spec: ClusterSpec) -> Result<ClusterId, Error>;
    pub fn link(&self, derived: ClusterId, sources: &[ClusterId]) -> Result<(), Error>;       // ≤ max_links, azyklisch
    pub fn rebucket(&self, c: ClusterId, new_due: Due) -> Result<RebucketReport, Error>;
}
impl Consumer {
    pub fn next(&mut self, max: usize) -> Result<Vec<ChangeEvent>, Error>;
    pub fn ack(&mut self, upto: Stamp) -> Result<(), Error>;      // persistiert als Catalog-Edit consumer_ack
}
```

Restrictions: `RestrictionSpec { reason: u16 (≠ 0), owner, effects: BLOCK_ERASE | BLOCK_WRITE | BLOCK_READ, expires: Option<WallTime> }`. Ablauf lässt nur den Vermerk entfallen, nie automatisch löschen (O-3).

### 8.5 Löschung

```rust
impl Storage { pub fn eraser(&self) -> Eraser; }
impl Eraser {
    pub fn cluster(&self, c: ClusterId, req: EraseReq) -> Result<Ticket, EraseError>;        // ER0–ER1 synchron
    pub fn clusters_batch(&self, v: &[ClusterId], req: EraseReq) -> Result<Ticket, EraseError>; // eine Barriere (§15.5)
    pub fn pack_expired(&self, k: PackKey, req: EraseReq) -> Result<Ticket, EraseError>;
    pub fn range(&self, sp: &Space, c: ClusterId, r: KeyRange, req: EraseReq) -> Result<Ticket, EraseError>;
    pub fn status(&self, t: &Ticket) -> EraseStatus;      // Pending | InProgress{stage} | Done(ErasureReceipt) | Unknown
    pub fn wait(&self, t: &Ticket, until: EraseStage, timeout: Duration) -> Result<EraseStatus, Error>;
    pub fn verify(&self, t: &Ticket) -> VerifyReport;
    pub fn scan_physical(&self, spec: ScanSpec) -> ScanReport;   // rein lesend; nur Anzahl und Dateikennung (INV-SCAN-1)
}
pub struct EraseReq {
    pub level: EraseLevel,              // Logical | Physical | Scrubbed | Cryptographic
    pub views: ViewPolicy,              // Wait(timeout) | Revoke
    pub holds: HoldPolicy,              // Fail (Standard) — kein Override
    pub purge_deadline: Duration,       // PFLICHT, kein Standardwert (SD-06)
}
```

### 8.6 Listener, Betrieb, Prüfung

```rust
pub trait Listener: Send { fn on_change(&mut self, e: ChangeEvent); fn on_lagged(&mut self, missed_from: Stamp); }
impl Storage {
    pub fn checkpoint(&self, dst: &Path, m: CheckpointMode) -> Result<Manifest, Error>;   // Copy | Reflink | Hardlink (Opt-in)
    pub fn restore_check(&self, m: &Manifest, ledger: &LedgerRef) -> RestorePlan;         // §15.7
    pub fn verify(&self, scope: VerifyScope) -> VerifyReport;                             // Online | Offline
    pub fn stats(&self) -> Stats;                                                         // §19.3
    pub fn health(&self) -> Health;                                                       // Healthy | Poisoned{cause, durable_stamp}
    pub fn hibernate(&self) -> Result<(), Error>;
    pub fn inspect(&self) -> Inspector;
}
```

Listener-Zustellung: nach Durabilität und Sichtbarkeit, in Stamp-Reihenfolge, aus einem Dispatcher-Job ohne gehaltene Sperre (P-10). Körper werden nicht mitgeliefert (Abruf über `view_at`). Läuft ein Listener über `listener_queue` oder überschreitet er `listener_deadline`, wird er **abgemeldet** und erhält `ListenerLagged{missed_from}`. Die Zustellung ist höchstens einmal; mindestens-einmal entsteht nur über `Consumer` und `history` (C-18).

### 8.7 Fehlerkonvention

Ein einziges `Error`-Enum. Jeder Fehler hat `code()` (stabil), `retryable()`, strukturierte Felder und `hint()`: kurzer, handlungsleitender, **datenfreier** Text, lesbar für Menschen und Sprachmodelle. `hint()` enthält nur Typen, Zahlen, Space- und Consumer-Namen sowie opake numerische Kennungen (INV-KEY-3, §16.5).

Beispiel: „Space ‚trace‘ ist append-only; Schlüssel existiert bereits. Neuen Schlüssel wählen.“ — „Stamp 880 nicht mehr verfügbar; älteste verfügbare Sicht: 1 012.“

| Variante | Bedeutung | `retryable` |
|---|---|---|
| `GuardFailed` | Vorbedingung nicht erfüllt | nein (neu lesen) |
| `Busy` | Gegendruck, `StorageFull`, Governor | ja |
| `Poisoned` | E/A-Fehler im Schreibpfad oder Sperrenvergiftung | nein (Neustart) |
| `Indeterminate` | Fsync-Ergebnis unbekannt | ja, mit demselben `IdemToken` |
| `Corrupt{file, offset, what}` | Prüfwert oder Struktur falsch; Datei quarantänisiert | nein |
| `Erased{range, erased_at}` | Zugriff unterhalb eines Löschstamps | nein |
| `Shredded{domain}` | Schlüssel des Clusters vernichtet | nein |
| `Quarantined{file}` | Einheit wegen früherer Prüfwertverletzung gesperrt | nein |
| `Restricted{ids, reasons}` | Restriction blockiert Lesen, Schreiben oder Löschen | nein (nach `lift`) |
| `HoldBlocked{holds}` | Löschung durch Halteregel blockiert | ja (nach Freigabe) |
| `KeyStillPresent` | `Erase` mit noch vorhandenem Schlüssel | nein (Schlüssel zuerst vernichten) |
| `Limit{what, max}`, `ChangeTooLarge` | Größe überschritten | nein |
| `Forbidden{space, op}` | Operation im Space nicht erlaubt | nein |
| `MixedDurability` | Change über dauerhafte und flüchtige Ziele | nein |
| `Config{what}` | Konfiguration widerspricht Space-Optionen oder Format | nein |
| `Expired` / `InvalidToken` | `ResumeToken` oder Stamp nicht mehr haltbar, falsch gebunden | nein (neu beginnen) |
| `HistoryPruned{oldest_available}` | Verlauf unterhalb der Untergrenze | nein |
| `ListenerLagged{missed_from}` | Listener-Warteschlange übergelaufen | nein (aufholen) |
| `CallerPanic{what}` | Panik in Aufrufer-Code | nein |
| `Rollback{expected, found}` | Rollback-Anker verletzt (§16.7) | nein |
| `Unsupported{feature}` | Pflicht-Feature, Modus oder Dateiversion unbekannt | nein |
| `Locked`, `Closed`, `NotFound`, `Exhausted`, `Io(kind)` | Sonstiges | je nach Fall |

### 8.8 Fertigmeldung statt `async`

`submit(change) -> Pending`; `Pending::poll() -> Option<Result<Receipt, ApplyError>>`, `Pending::wait()`, `Pending::on_ready(Box<dyn FnOnce(Result<Receipt, ApplyError>) + Send>)`. Der Callback läuft auf der `Runtime`, ohne gehaltene Sperre. Eine obere Schicht baut daraus `Future`-Fassaden; der Kern enthält keinen Executor (NZ-17).

### 8.9 Stabile Flächen für aufsetzende Schichten (semver, D-16)

Die folgenden Flächen sind Vertrag und haben je einen Konformitätstest in `testing::conformance`:

- `view_at(stamp)` plus `Guard::Unchanged`/`RangeUnchangedSince` ersetzen Lesemengen und Validierung (kein SSI-Puffer im Kern).
- `View::stamp()`, `Storage::view_durable()`, `Receipt{stamp, durable}`.
- `hold`, `hold_durable`, `hold_drop`, `holds(owner_prefix)`; dauerhafte Holds überleben Neustarts (INV-HOLD-1).
- `checkpoint(dest, mode)` als konsistenter Stand (§15.7); benannte Checkpoints bleiben außerhalb (NZ-18).
- `SegmentWriter`/`SegmentReader` mit Kern-Rahmen (§10.10); opakes 16-Byte-`reference`.
- `ErasureReceipt` (serialisierbar, stabil), `erase_status`, `scan_physical`.
- `KeyProvider` (§4) und `SecretAlloc`.
- `submit`/`Pending`.
- `IdemToken`, `ResumeToken`, `Budget`, `Estimate`.

---

## 9. Dateilayout und Dateilebenszyklus

### 9.1 Layout

```
<root>/
  FORMAT              Magic, Formatversion, Feature-Bits, integrity_mode, instance_id, Prüfwert (§10.2)
  LOCK                exklusive (ReadWrite) oder geteilte (ReadOnly) Sperre über Fs::lock
  CURRENT             Zeiger auf die aktive Catalog-Datei (atomar ersetzt)
  CATALOG-0000007     Snapshot-Record und Edit-Records (§10.8)
  KEYS                (Authenticated/Encrypted) Schlüsselwurzel und Space-DEKs, mit KEK umhüllt (§16.2)
  ERASE-LEDGER/       append-only Löschprotokoll, kettenverknüpft (§15.8)
  redo/redo-<segment_id:016x>.log     vorallokiert, am Ende gesiegelt (§10.4)
  spaces/<space:04x>/<file_id:016x>.run
  spaces/<space:04x>/<blob_id:016x>.<inc:08x>.blob
  segments/<kind:04x>/<id:016x>.seg   Dateien oberer Schichten (§15.9)
```

Der Anker liegt **nicht** im Verzeichnis, sondern hinter dem Port `Anchor` (§16.7).

### 9.2 Regeln

- **Numerische IDs** benennen alle Dateien. Der Catalog speichert nie Pfadstrings.
- **Blob-Inkarnation (C-11):** Ein Blob wird nie verändert. Entfallen Bereiche eines Clusters, entsteht eine **neue Inkarnation** derselben `blob_id` (`<id>.<inc>.blob`) mit **gleichen Offsets**. Entfallene Bereiche werden mit Nullen überschrieben (`punch_hole`, wenn verfügbar). Der Catalog tauscht atomar (`blob_replace`). Ein Umzug in eine **neue** `blob_id` gehört zur Blob-GC (§13.3).
- **Blob-Lebendigkeit (INV-BLOB-1):** Ein Blob oder eine Inkarnation wird nur entfernt, wenn kein lebender Run und kein lebender Blob-Verweis mehr auf sie zeigt. Der Abgleich läuft über die lebenden Runs, nicht über einen Zähler. `blob_live_delta` dient nur der Auswahl von GC-Kandidaten (C-09).
- **Redo-Segmente sind kein `FileKind`:** lebendig ⇔ `segment_id ≥ redo_floor`. Das vorallokierte Reservesegment ist lebendig (INV-LOG-7). `FileKind` kennt `Run`, `Blob`, `Segment(kind)`.
- Alle Dateien und Verzeichnisse entstehen mit `Perm::Private`.
- **Temporäre Dateien** tragen das Suffix `.building.<tmp_id>`. `tmp_id` stammt aus dem im Catalog reservierten Bereich (§10.8), nie aus der Prozess-ID. Beim Öffnen werden alle `.building.*` entfernt.
- **Verwaiste Dateien** (nicht vom Catalog genannt) werden beim Öffnen und periodisch gelöscht. Das ist sicher, weil nur ein Catalog-Edit eine Datei lebendig macht (INV-CAT-1).
- **Ein Space = ein Verzeichnis.** Ein Space wird entfernt durch Edit, Löschen aller Dateien und `remove_dir`.
- Lebendige Dateien sind unveränderlich. Ausnahme: das Redo-Segment bis zu seinem Siegel.

### 9.3 Zustände einer Datei

```
(nicht vorhanden) ─create .building─► im Bau ─sync_all + rename + sync_dir─► dauerhaft, UNREFERENZIERT
      ▲                                                                           │ Edit(files_add) dauerhaft
      │ Verwaisten-Suche                                                          ▼
      └─ remove + sync_dir ◄── Graveyard ◄── obsolet ◄──────────────────── LEBENDIG
        (erst nach letzter Referenz)                       ▲ Edit(files_remove) dauerhaft
```

Jeder Wechsel zwischen „unreferenziert“ und „lebendig“ ist **ein** Catalog-Edit (P-2). Eine Datei wechselt nur in Pfeilrichtung.

---

## 10. Formate

### 10.1 Konventionen

- Alle Zahlen **little-endian**. `varint` = LEB128 für `u64`, höchstens 10 Byte, **nur kanonisch**: keine überflüssigen Fortsetzungsbytes, das zehnte Byte darf nur `0x01` sein. Alles andere ist `Corrupt` (INV-FMT-1).
- Jede Dateiart hat **genau eine Version (`1`)** und Feature-Bits (SD-04). Unbekannte Version oder unbekanntes Pflicht-Bit ⇒ `Unsupported`. Unbekannte optionale Bits werden ignoriert.
- **Prüfwert je Modus (`chk`):** Die Breite ist modusabhängig, das Feld heißt überall `chk`.

| Modus | Prüfwert pro Frame, Block, Record | Breite | Herkunft |
|---|---|---|---|
| `Plain` | CRC-32C, im Redo zusätzlich über die Kette | 4 B | `Checksum`-Port |
| `Authenticated` | `Mac16` über Nutzbytes mit gebundenen AAD-Feldern | 16 B | `Mac`-Port, Schlüssel aus `KEYS` |
| `Encrypted` | AEAD-Tag von `Cipher` (Tag ersetzt `chk`) | 16 B | `Cipher`-Port |

- **Header und Footer** tragen in allen Modi zusätzlich einen CRC-32C. In `Authenticated`/`Encrypted` tragen sie außerdem `Mac16` über dieselben Bytes (Datei-MAC, §10.6).
- **AAD (INV-FMT-3):** Jeder authentifizierte Frame, Block und Record bindet die Kennungen seiner Art, Ort und Zugehörigkeit (Dateiart, Space, Cluster, Block-UID, Frame-Index oder Record-Offset). Ein vertauschter Block öffnet deshalb nicht.
- **Nonces:** `Cipher` verwendet **24 Byte Zufallsnonce** (XChaCha-Familie). Jede Versiegelung zieht eine frische Nonce aus `Entropy`. Es gibt keinen Zählerzustand, der bei Absturz oder Kopie verloren gehen könnte (SD-14). Die Kollisionswahrscheinlichkeit je Schlüssel ist bei 192 Bit vernachlässigbar, solange je Schlüssel weniger als 2^64 Versiegelungen stattfinden. Die Anzahl je Schlüssel wird im Catalog gezählt und bei Erreichen einer Schranke rotiert (§16.3).
- **Längen** werden gegen §20.1 und die Restlänge der Datei geprüft, bevor allokiert wird (INV-FMT-2).
- **Reservierte Felder** werden mit 0 geschrieben und beim Lesen auf 0 geprüft.

### 10.2 `FORMAT`

```
magic "CTXS" [4] | format_ver u16 (=1) | feature_req u64 | feature_opt u64 | integrity_mode u8 |
instance_id [16] | algo_set u16 (1 = XChaCha20-Poly1305, BLAKE3-keyed, HKDF-artig über Kdf-Port) |
created_ms u64 | chk
```

- Länge vor `chk`: **49 B** (magic 4, format_ver 2, feature_req 8, feature_opt 8, integrity_mode 1, instance_id 16, algo_set 2, created_ms 8). `chk` = CRC-32C (`Plain`, 4 B) oder `Mac16` (`Authenticated`/`Encrypted`, 16 B). Gesamtlänge: **53 B** bzw. **65 B**.
- `integrity_mode ∈ {0 Plain, 1 Authenticated, 2 Encrypted}` steht an Offset 22, damit der Leser die Länge von `chk` kennt, bevor er prüft.
- Der Modus ist nach dem Anlegen **unveränderlich**. Ein Wechsel ist ein Migrationswerkzeug.
- **Downgrade-Schutz (F-15, INV-FMT-4):** `Config.integrity` ist beim Öffnen verbindlich. Weicht `integrity_mode` ab, ist das Ergebnis `Err(Unsupported{integrity_mode})`, bevor irgendeine andere Datei gelesen wird.
- Pflicht-Bits: `ENCRYPTED = 1<<0`, `AUTHENTICATED = 1<<1`, `SEAL_CLUSTER = 1<<2`, `COLD_TIER = 1<<3`. Bits `< 2^32` sind für Pflichtfeatures reserviert, Bits `≥ 2^32` für optionale. Pflicht-Bits werden nur **hinzugefügt**, und zwar per `write_atomic`, bevor der erste Catalog-Edit das Feature nutzt.

### 10.3 `KEYS` (nur `Authenticated`/`Encrypted`)

```
"CTXK" | version u16 | keys_generation u64 | nonce [24] | sealed_len u32 | sealed
sealed = Cipher.seal( key = Kdf(KEK, "ctxs keys v1" ‖ instance_id), nonce, aad = "CTXK" ‖ version ‖ instance_id ‖ keys_generation,
                      plain = root_gen_count u16 | ( root_gen u32 | root_key [32] ) × count
                              | space_count u16 | ( space u16 | dek_gen u32 | dek [32] ) × space_count )
```

- Die Datei wird per `write_atomic` ersetzt. `keys_generation` zählt hoch. `FORMAT` und `CURRENT` nennen die erwartete Generation (`keys_generation_min`, §10.8), sodass ein zurückgespielter alter `KEYS` auffällt.
- **Schlüsselwurzel (D-06, C-04):** `root_key` je Generation ist die Wurzel aller Nicht-Cluster-Schlüssel (§16.2). Die KEK umhüllt nur `KEYS`. **KEK-Rotation ist deshalb O(1)** und berührt keine Datendatei.
- **Space-DEK** je Space und `dek_gen`: Schlüssel für Blöcke, Blobs und Datei-MACs nicht versiegelter Spaces (§16.2).
- Alte `root_key`-Generationen und alte DEKs bleiben in `KEYS`, solange Dateien mit ihnen existieren. Jede Redo-Datei und jeder Run trägt seine Generation im Header.

### 10.4 Redo-Log

Datei `redo/redo-<segment_id:016x>.log`, vorallokiert (§12.5), am Ende gesiegelt.

**Segmentkopf** (`chain_seed` je Modus: 4 B bei `Plain`, 16 B sonst):

```
"CTXL" [4] | ver u16 | flags u16 | segment_id u64 | first_stamp u64 | root_gen u32 | chain_seed (4 oder 16 B) | chk
```

- Jede Prozessinstanz beginnt ein **neues** Segment (INV-LOG-4). Es wird nie an ein bestehendes angehängt (D-14).
- Ein Gate-Frame (§15.5) beginnt immer ein neues Segment; das vorige Segment wird vorher gesiegelt (INV-LOG-4).

**Gruppe.** Eine Gruppe beginnt an einer `redo_align`-Grenze (4 096 Byte). Sie besteht aus einem Gruppenkopf, einem oder mehreren Frames und einem Gruppenabschluss. Den Rest bis zur nächsten Grenze füllt Nullpadding.

```
Gruppenkopf: group_len u32 (0 = Ende des Segments) | n_frames u16 | group_flags u8 | group_chk
Frame:       frame_len u32 | stamp u64 | kind u8 | n_ops u16 | idem_token [16] | key_epoch u32 | flags u8 |
             ops_head[n_ops] | ops_payload | frame_chk
             ops_head[i] = space u16 | cluster [16] | op_kind u8 | op_uid [16] | payload_len u32
frame_len = Länge ab stamp bis einschließlich ops_payload (ohne frame_len und frame_chk)
kind: 1 Change · 2 ClusterGate (§15.5) · 3 Reserve · 4 Control (Restrict, Lift, Stamp-Reserve, Erase-Epoch)
```

- `frame_chk` (Plain) = CRC-32C, Seed = Kettenwert des Vorgängers. In `Authenticated`/`Encrypted` ist es `Mac16` über Kopf und Nutzbytes mit AAD `(segment_id, frame_index, space, cluster, op_uid)`. Die Kette bindet den Vorgänger und erkennt Abschneiden und Einfügen.
- **Ops-Nutzlast:**
  - `Seal::Cluster`: die **gesamte** Nutzlast (Schlüssel, Kopf, Körper, Gewichte) ist mit dem Clusterschlüssel versiegelt. AAD `(space, cluster, op_uid, op_index)`. **Kein Stamp** in der AAD, weil die Versiegelung vor der Stamp-Vergabe geschieht (C-16, D-15).
  - Sonst: Nutzlast unter dem Redo-Schlüssel (`Encrypted`) bzw. Klartext (`Plain`/`Authenticated`).
- **Siegel-Frame:** `kind = 1`, `flags` Bit 7 gesetzt, `n_ops = 0`, Nutzlast `frame_count u64 | final_chk`. Ein Segment ist gesiegelt, wenn sein letzter gültiger Frame ein Siegel ist. Das Feld heißt überall **Bit 7** (C-10).
- **Ende-Erkennung:** `group_len = 0` markiert das Ende des vorallokierten Nullbereichs.

**Regeln:**

- **Ein Frame = ein Change** (INV-LOG-1). Ein Frame ist beim Replay vollständig gültig oder beendet das Log (INV-LOG-2).
- **Torn-Tail (INV-LOG-8):** Der erste ungültige Gruppenkopf beendet das Log nur dann, wenn im Segmentrest danach keine prüfbar gültige Gruppe folgt. Sonst ist es `Corrupt{RedoMidLog}`. Alles bis zur letzten `redo_mark` MUSS gültig sein (INV-LOG-7).
- **Siegel (INV-LOG-6):** Jedes Segment außer dem neuesten mit Frames ist gesiegelt. Ein Defekt in einem gesiegelten Segment ist `Corrupt` (fail-closed).
- **Stamps** im Log sind streng aufsteigend (INV-LOG-3) und dürfen Lücken haben.
- **Körper reisen im Frame** (SD-29). Der Redo ist der einzige dauerhafte Ort eines Körpers bis zum Flush. So bleibt es bei **einem** `sync_data` je Gruppe.

### 10.5 Run

```
Run := Header
       ClusterSection*           (aufsteigend nach ClusterId; Pure: genau eine)
       ClusterDirectory          (je Cluster: cluster, section_off, section_len, n_blocks, min_stamp, max_stamp, weight_sum[W] u64)
       PackMeta                  (space, pack_key | pure_cluster, Eintragszahlen, Blob-Bereich)
       ExtensionBlock*           (§15.9)
       Footer                    (Offsets, Anzahl, Datei-MAC)

ClusterSection := DataBlock* ‖ SubIndexBlock ‖ FilterBlock ‖ RangeDelBlock
   DataBlock     Ziel 16 KiB; Präfixkompression mit Restart-Punkten; Seal oder Block-AEAD (Space-DEK)
   SubIndex      je Datenblock: letzter Schlüssel, Offset, Länge, entries, weight_sum[W] u64,
                 min_stamp, max_stamp, block_uid [16], block_chk
   FilterBlock   Schlüsselfilter des Clusters (Bloom, §10.5.1), keyed und clustergebunden
   RangeDelBlock Bereichsmarker des Clusters, sortiert und fragmentiert
```

- **`Seal::Cluster`:** SubIndex, Filter, RangeDel und alle DataBlocks sind versiegelt. Klartext bleiben `ClusterDirectory`, die Cluster-ID (opak), Zähler, Größen, Stamp-Bereiche und die Datei-MAC (§10.6).
- **Öffnen:** Footer → Datei-MAC prüfen → ClusterDirectory → SubIndex. Filter und Datenblöcke werden **lazy** geladen. Vor jedem Zugriff prüft der Kern Offset und Länge gegen die Dateilänge und gegen Überlappung.
- **Block-AAD** (`Authenticated`/`Encrypted`): `(space, cluster, block_uid)`. Die Dateiposition ist nicht Teil der AAD, damit Rohkopieren möglich ist (§7.2).
- Packed-Runs enthalten Körper nur inline bis `value_separation_min` (§6.3). Größere Körper liegen in Blobs.
- Kompression: `codec` ist `0` (keine). Der `BlockCodec`-Trait ist reserviert (NZ-10).

#### 10.5.1 Schlüsselfilter

Bloom-Filter über **Nutzerschlüssel** (distinkt), `bits_per_key = 10`, Größe aus der tatsächlichen Schlüsselzahl. Layout: `k u8 | nbits u64 | bits[nbits/8]`, `nbits = max(64, ⌈n · bits_per_key⌉)` aufgerundet auf ein Vielfaches von 64, `k = 7`. Positionen: `h1 = XXH64(key, 0)`, `h2 = XXH64(key, 0x9E3779B97F4A7C15) | 1`, `pos_i = (h1 + i·h2) mod nbits` für `i = 0..k`, Berechnung in `u64` mit Umlauf (Golden A.6).

- **PrefixFilter** (`prefix_filter(n)`): gleiches Layout über die ersten `n` Bytes jedes Schlüssels. Ein Präfix-Scan mit kürzerem Präfix als `n` kann ihn nicht nutzen und überspringt ihn.
- Ribbon-Filter sind als Alternative erlaubt (H-14), würden aber ein eigenes Format erfordern und sind deshalb vor G-F zu entscheiden (O-13).

### 10.6 Datei-MAC (Integrität der Struktur, C-08)

Nur in `Authenticated`/`Encrypted`:

```
Datei-MAC = Mac16( Kdf(DEK, "ctxs file v1" ‖ file_id), Header ‖ ClusterDirectory ‖ alle SubIndex-Einträge ‖ Footer-ohne-MAC )
```

- Der Datei-MAC bindet Verzeichnis, Abschnittsindex und Zähler. Ein Angreifer kann dadurch keinen Block aus einer anderen Datei desselben Clusters einsetzen, ohne dass Footer oder Verzeichnis nicht mehr stimmen.
- Der Datei-MAC ist der **einzige** Schutz gegen das Umsetzen von Blöcken zwischen Dateien derselben Gruppe. Die Block-AAD allein genügt dafür nicht.
- Rohkopieren (§7.2) berechnet den Datei-MAC mit dem DEK neu. Der DEK ist dafür vorhanden, weil der Cluster-Schlüssel allein das Verzeichnis nicht schützt.

### 10.7 Blob-Datei

```
Blob := Header ‖ Record* ‖ ClusterDirectory ‖ Footer
Header: "CTXB" | ver u16 | flags u16 | blob_id u64 | inc u32 | space u16 | reserved [22] | chk
Record: rlen u32 | nonce [24] (nur Encrypted) | content (Klartext oder Ciphertext mit Tag) | chk (Plain: CRC-32C; Authenticated: Mac16)
        content = kind u8 (1 Body) | domain u32 | stamp u64 | klen varint | key | body
Footer: "CTXB" | record_count u32 | bytes u64 | datei-mac | chk
```

- **Selbstbeschreibend:** Jeder Record trägt `(domain, stamp, key)`. Dadurch kann `verify` Blobs ohne den Run prüfen, die GC Lebendigkeit bestimmen und die Löschprüfung (§15.11) Reste finden.
- **Versiegelung:** `Encrypted` verschlüsselt `content` mit dem Blob-Schlüssel `Kdf(DEK, "ctxs blob v1" ‖ key_epoch u32 ‖ blob_id u64)`, Nonce zufällig, AAD `(space, cluster, blob_id, offset)`. `Seal::Cluster`-Körper werden mit dem Clusterschlüssel versiegelt, Nonce zufällig, AAD `(space, cluster, blob_id, offset)`.
- **Löcher (INV-BLOB-3):** Entfallende Records werden mit **Nullen gleicher Länge** überschrieben. Löcher werden nie verschlüsselt. Ein Verweis, der auf ein Loch zeigt, liefert `Corrupt{HoleRef}`.
- **Inkarnation (C-11):** Eine neue Inkarnation mit gleichen Offsets übernimmt alle überlebenden Records **byte-identisch**. Dadurch bleiben Verweise gültig, ohne Runs umzuschreiben.
- **Größe:** `blob_target_bytes` = 64 MiB. Ein einzelner Record über dem Ziel erhält eine eigene Datei.
- **Kein Teilen (INV-BLOB-2):** Jeder `BlobRef` zeigt auf **einen eigenen** Record. Kein Dedup. Dadurch ist die Lebendbilanz exakt.

### 10.8 Catalog

```
Datei  : Header (20 B) "CTXC" | ver u16 | flags u16 | catalog_no u64 | chk
Record : len u32 | tag u8 (1 Snapshot, 2 Edit) | payload | chk
chk    : Plain: CRC-32C über Tag ‖ payload, Seed = Vorgänger
         Auth.: Mac16(Schlüssel aus Wurzel, chk_vorher ‖ len ‖ tag ‖ payload ‖ catalog_no ‖ record_index)
         Encr.: wie Auth., Payload zusätzlich versiegelt (Nonce zufällig)
```

Der **Snapshot** ist ein Edit, der vom leeren Zustand ausgeht. Ein **Edit** ist genau ein Record (P-2) aus TLV-Feldern `field_id u16 | len varint | bytes`. Das Bit `0x8000` in `field_id` markiert ein **optionales** Feld; unbekannte optionale Felder werden übersprungen, unbekannte Pflichtfelder ⇒ `Unsupported`.

| Op | Inhalt |
|---|---|
| `space_add` / `space_drop` | Space mit Optionen und Namen; Entfernen nur nach Abwurf aller Dateien |
| `cluster_register` / `cluster_state(c, state, stamp)` | Clusterzustand und Gate-Stamp |
| `cluster_link` | Verknüpfung abgeleiteter Cluster |
| `restriction_set` / `restriction_lift` | Restrictions mit Grund und Eigentümer |
| `pack_assign` | Zuordnung Cluster → Packgruppe (nur zusammen mit Dateimengen, INV-PL-4) |
| `files_add(kind, id, group ∣ cluster, meta)` / `files_remove(kind, id)` | Dateimengen; `meta` enthält Schlüssel- und Stamp-Bereiche, Bytes, Blob-Bereich, Generation, `tier`, Datei-MAC-Stand |
| `blob_replace(blob_id, inc_alt, inc_neu)` | Inkarnationstausch |
| `blob_live_delta(blob_id, delta)` | Auswahlhilfe für GC; ist nicht Wahrheit (INV-BLOB-1) |
| `space_flushed(space, stamp)` | Memtable bis `stamp` ist in Runs |
| `redo_floor(floor, floor_stamp, hwm, mark)` | Untergrenze des Redo und Hochwassermarke (INV-LOG-8) |
| `reserve(run_id_hw, blob_id_hw, segment_id_hw, tmp_id_hw, stamp_hw, keys_gen_min, erase_epoch)` | Hochwassermarken, nur steigend (INV-KEY-2, INV-STAMP-3) |
| `consumer_set` / `consumer_ack` / `consumer_drop` | Consumer-Zustand und Rückstandsgrenze |
| `hold_set` / `hold_drop` | dauerhafte Holds (INV-HOLD-1) |
| `erase_pending_add(ticket)` / `erase_done(ticket)` / `erase_swept(ticket)` | Löschaufträge |
| `erased_ranges_add` / `erased_ranges_remove` | Bereiche mit Löschstamp (INV-ERA-5) |
| `domain_retired` | nur für Compat-Import; nicht erzeugt (§15.3) |
| `key_epoch(space, epoch)` | Schlüsselepoche je Space oder Cluster |
| `anchor_mark(value)` | Anker-Wert nach einem Löschschritt |
| `idem_trim(watermark)` | Entfernen abgelaufener Idempotenz-Tokens |

- Tags `0xF000…0xFFFF` sind reserviert. Unbekannte Tags ⇒ `Unsupported`.
- **Regel (INV-CAT-1):** Eine Datei ist lebendig ⇔ ein dauerhafter `files_add` ohne nachfolgendes `files_remove`.
- **ID-Reservierung (INV-KEY-2):** `run_id`, `blob_id`, `segment_id`, `tmp_id` und Stamps werden erst benutzt, nachdem ein **dauerhafter** `reserve`-Edit die Hochwassermarke über sie hinweg gesetzt hat. Reservierung erfolgt in Blöcken (`2^20` Stamps, `2^16` IDs). Nach Absturz beginnt die Vergabe oberhalb der Marke. Ungenutzte Werte verfallen. `segment_id` ist `< 2^32`, alle anderen `< 2^56`. Überlauf ⇒ `Exhausted`.
- **Rollover** bei 8 MiB oder 1 000 Edits: neue Datei `CATALOG-(n+1)` mit Snapshot, `sync_all`, `CURRENT` per `write_atomic`, `sync_dir`, alte Datei entfernen.
- **Lesen:** Ein beschädigter oder halber **letzter** Record wird verworfen (Torn Write). Jeder frühere Defekt ist `Corrupt` (INV-CAT-3, fail-closed).
- Im Modus `Authenticated`/`Encrypted` ist `chk` des letzten Records der Kopf der Kette. Er wird dem Anker übergeben (§16.7).

### 10.9 Segment (obere Schicht)

```
Segment := Header (kind u16 | id u64 | space u16 | cover | reference [16] | chk)
           Rahmenblöcke* (Block-chk oder AEAD je Modus, Block-AAD mit kind, id, block_index)
           Siegel-Footer (Blockanzahl, Datei-MAC, chk)
```

`SegmentWriter` schreibt `.building`, `sync_all`, `rename`, `sync_dir`; `SegmentReader` prüft jeden Block vor der Ausgabe (INV-SEG-1). Der Kern kennt Inhalt und Blockzerlegung nicht. `reference` ist opak (Datenstand-Anker der oberen Schicht).

### 10.10 Versiegelung und Seal-Rahmen

- `Seal::Cluster` versiegelt jeden Block, jede Redo-Op-Nutzlast und jeden Blob-Record mit dem Clusterschlüssel (`cluster_key(c, epoch)`).
- Siegelschlüssel: `Kdf(cluster_key, "ctxs seal v1" ‖ key_epoch u32)`. Nonce zufällig je Versiegelung.
- AAD-Regeln siehe §10.4 (Redo), §10.5 (Block) und §10.7 (Blob). Keine davon enthält die Dateiposition.

### 10.11 Dateizugriff und Prüfwert-Verletzungen

Jede Verletzung eines Prüfwerts oder einer Struktur wird als `Corrupt` gemeldet, nie als leere Antwort. Die betroffene Einheit wird quarantänisiert (§5.3). Der Kern repariert nur den Torn-Tail des letzten Redo-Segments und den letzten Catalog-Record.

### 10.12 Format-Freeze (Gate G-F)

Eingefroren werden: Dateikopf, `FORMAT`, `KEYS`, Segmentkopf, Gruppenkopf, Frame, Kette, `ClusterSection`, `ClusterDirectory`, SubIndex, Siegel-Rahmen (AAD- und Nonce-Regeln), Datei-MAC, Blob-Record und Inkarnationsregel, Catalog-Op-Tags (mit reserviertem Bereich), `anchor_mark`, Anker-Wert, Ledger-Format, `cmp`, Tag-Kodierung, Bloom-Layout.

Jeder Punkt braucht vor **G-F** einen Golden aus dem unabhängigen Referenzcodierer (`tools/reference/generator.py`) und einen Test gegen die Rust-Implementierung (Anhang A).

---

## 11. Schreibpfad

### 11.1 Stufen

```
submit(Change) ─► Intake (Rang 2) ─► Committer (Rang 1, einziger Schreiber des Redo) ─► Redo-Gruppe ─► sync_data ─► Memtable (lock-frei) ─► Receipt
```

1. **Intake** prüft Schema, Limits (§20.1), Idempotenz (`IdemToken`), Restriction und Clusterzustand gegen den Stand des Catalogs. Der Body wird versiegelt (§10.10). Danach ist die Änderung ein fertiger Frame ohne Stamp.
2. **Stamp-Vergabe** erfolgt erst im Committer. Der Committer nimmt Stamps aus dem reservierten Block (§10.8, INV-KEY-2). Reicht der Block nicht, reserviert er einen neuen per dauerhaftem `reserve`-Edit, bevor er den Frame schreibt.
3. **Committer** bündelt Frames zu einer Gruppe (§10.4), schreibt sie in das Redo und ruft **einmal** `sync_data` auf. Erst danach ist die Gruppe dauerhaft.
4. **Memtable** wird erst nach der Dauerhaftigkeit gefüllt. Ein Leser sieht also nie einen Wert, den ein Absturz zurücknehmen könnte (INV-COMMIT-1).
5. **Receipt** wird erst zurückgegeben, wenn Schritt 4 abgeschlossen ist.

### 11.2 Garantien

- **INV-COMMIT-1:** Ein bestätigter `submit` ist nach Absturz vorhanden. Ein nicht bestätigter ist entweder vollständig vorhanden oder vollständig nicht. Es gibt keinen Teilzustand.
- **INV-COMMIT-2:** Ein Change ist innerhalb eines Frames atomar. Frames einer Gruppe werden in Reihenfolge übernommen. Eine Gruppe ist eine Einheit des Fsync, keine Transaktionsgrenze für Nutzer.
- **INV-COMMIT-3 (Flüchtig):** Ein `Volatile`-Change (SD-19) wird nur in die Memtable geschrieben und nicht ins Redo. Er ist nach Absturz verloren. Volatile und dauerhafte Changes dürfen **nicht in derselben Gruppe** liegen, damit INV-COMMIT-2 gilt. Der Committer trennt sie. Ein Volatile-Change wird nie als dauerhaft bestätigt.
- **Gruppenbildung:** Der Committer sammelt Frames, bis `group_max_bytes` (1 MiB) oder `group_max_delay` (2 ms, §20.2) erreicht ist, oder bis die Warteschlange leer ist. Die Gruppe wird nie absichtlich künstlich verzögert, wenn kein weiterer Frame wartet.

### 11.3 Clustergate (ClusterGate-Frame, C-16)

Ein Cluster wechselt seinen Zustand (`Open → Restricted → Erasing → Erased`, §15.2) nur über ein **Gate**. Das Gate ist ein Frame `kind = 2` im Redo, mit Stamp `g`.

- Der Committer schreibt das Gate **vor** allen weiteren Frames des Clusters mit Stamp `> g`. Frames mit Stamp `< g` dürfen vor dem Gate stehen.
- Nach dem durable Gate lehnt der Committer alle Changes für diesen Cluster mit `ApplyError::ClusterGated` ab, die nach dem Gate ankommen. Die Prüfung erfolgt im Committer, nicht im Intake, damit keine Lücke zwischen Prüfung und Schreiben entsteht.
- Ein Gate-Frame beginnt ein neues Redo-Segment (§10.4). Das vorige Segment wird vorher gesiegelt. Dadurch ist „Stand vor dem Gate“ im Log eindeutig abgrenzbar.
- Die Gate-Stamp steht im Catalog als `cluster_state(c, state, stamp)`. Ein Cluster im Zustand `Erasing` ist durch Replay und Neustart wieder gegated, bevor er Änderungen annimmt (INV-ERA-2).

### 11.4 Idempotenz

- Jeder `Change` kann einen `IdemToken` (16 B) tragen. Das Redo speichert ihn im Frame.
- Der Kern hält einen Index `IdemToken → Stamp` für den Zeitraum bis zum `idem_trim`-Wasserzeichen (Konfiguration `idem_window`, Standard in §20.2).
- Ein erneuter `submit` mit bekanntem Token gibt das ursprüngliche `Receipt` zurück und schreibt nichts. Er wird verworfen, wenn sein Inhalt (Hash über Ops) vom gespeicherten abweicht: `ApplyError::IdemConflict`.
- Ein Token außerhalb des Fensters ist nicht prüfbar. Der Kern lehnt einen Change mit solch einem alten Token nicht ab, sondern meldet `ApplyError::IdemExpired`. Der Aufrufer muss selbst entscheiden.

### 11.5 Backpressure und Busy

- Die Memtable je Space hat eine Obergrenze (`memtable_limit`, §20.1). Erreicht sie die **Stopp-Grenze**, werden neue Changes für diesen Space mit `Busy` abgelehnt, bis der Flush sie senkt.
- Zwischen Warn- und Stopp-Grenze verzögert der Intake (`Governor`, §13.1). Die Verzögerung ist durch `Config` begrenzt; sie ist nie unbegrenzt.
- `Busy` ist ein Zustand, kein Fehler im Sinne von F-Klassen. Er ist wiederholbar, ohne dass etwas Halbfertiges entsteht.

### 11.6 Schreibfehler (fail-closed, F-Klassen §5)

- **Fehler vor `sync_data` einer Gruppe:** Die Gruppe ist nicht bestätigt. Der Committer schneidet das Redo auf das vorige Gruppenende zurück (`set_len` + `sync_data`), gibt allen Wartenden `Err(Io)` zurück und wechselt in den Zustand **`Degraded`**. In `Degraded` werden nur Lesezugriffe und Wartung erlaubt. Ein Neustart prüft das Log erneut.
- **ENOSPC (SD-08):** Wie oben. Zusätzlich verbietet der Kern danach jeden Schreibversuch, bis mindestens `enospc_cooldown` vergangen ist. Die Wiederaufnahme ist ein expliziter `Storage::resume_writes()`-Aufruf und kein stiller Retry.
- **Fehler beim Abschneiden:** Ist auch das Abschneiden nicht möglich, wechselt der Kern in **`Poisoned`**. `Poisoned` ist nur über Neustart verlassen. Der Kern bestätigt nichts mehr.
- **Fehler nach `sync_data`:** Die Gruppe ist dauerhaft. Der Fehler wird als `Io` an den Aufrufer gegeben, der Zustand ist aber `Degraded`. Das Receipt ist nicht vorhanden. Beim Neustart erscheint der Change als vorhanden, wenn er das Log erreicht hat. Der Aufrufer muss dann mit `IdemToken` nachfragen, nicht blind wiederholen.

### 11.7 Segmentschnitt

- Ein Redo-Segment endet bei `redo_segment_bytes` (Standard in §20.2), bei einem Gate-Frame (§11.3) oder beim Öffnen einer neuen Prozessinstanz.
- Das Ende ist ein Siegel-Frame (§10.4). Danach wird die Datei nicht mehr beschrieben. Das nächste Segment bekommt eine neue `segment_id` aus dem Catalog-Block.

---

## 12. Lesepfad

### 12.1 Ansicht (`View`)

- Eine `View` ist ein **Stamp-Schnitt**: sie sieht alle Änderungen mit Stamp `≤ s` und sonst nichts. `s` wird beim Erzeugen gesetzt und ist danach unveränderlich.
- Eine `View` hält eine **Referenz** auf die lebenden Runs und Memtables zu diesem Zeitpunkt. Sie verhindert, dass diese Dateien vor ihrem Ende entfernt werden (§13.3, INV-VIEW-1). Eine `View` ist ein Lease mit Zeitbudget (`view_max_age`). Abgelaufene Views werden gezwungen geschlossen und liefern danach `ViewExpired`.

### 12.2 Reihenfolge

Für einen Schlüssel `k` und Stamp-Schnitt `s`:

1. **Memtables** des Space, neueste zuerst, nur Einträge mit Stamp `≤ s`.
2. **Runs** in der Reihenfolge der Ebenen: zuerst L0 (neueste zuerst), dann tiefere Ebenen.
3. Je Run: **Cluster des Schlüssels bestimmen** (Schlüsselbereich der Cluster-Tabelle, §6.4), dann **Schlüsselfilter** (§10.5.1). Negativ ⇒ weiter, ohne Blockzugriff.
4. **SubIndex** binär suchen ⇒ genau ein Datenblock. Block laden, prüfen, Schlüssel suchen.
5. **RangeDel** des Clusters prüfen: Ein Löschbereich mit Stamp `> stamp(Eintrag)` und `≤ s` verdeckt den Eintrag.
6. Der **erste gefundene** Eintrag mit höchstem Stamp `≤ s` gewinnt. Ein Tombstone gewinnt ebenso und liefert „nicht vorhanden“.

### 12.3 Löschsperren im Lesepfad (INV-ERA-4)

- Ein Schlüssel in einem Bereich der Liste `erased_ranges` liefert nie einen Wert, auch wenn er in einem Run steht. Diese Prüfung erfolgt **vor** dem Blockzugriff, damit der Kern keinen gelöschten Körper entschlüsselt.
- Eine `Restriction` (§8.5) liefert `Restricted` für den Cluster, ohne Körper zu lesen. Ein Lesen ist nur mit `Holds`-Status erlaubt, den der Aufrufer ausdrücklich anfordert.

### 12.4 Iteratoren und Präfixe

- Ein Iterator mergt Memtables und Runs nach Schlüssel, mit demselben Stamp-Schnitt. Gleiche Schlüssel ergeben nur den neuesten sichtbaren Eintrag.
- **Präfix-Scan** nutzt den `PrefixFilter` (§10.5.1) nur bei Präfixlänge `≥ n`. Kürzere Präfixe lesen alle Runs ohne Filter.
- Ein Iterator ist an seine `View` gebunden und endet mit `ViewExpired`, wenn die Lease abläuft.
- Lesen aus Blobs geschieht erst beim Zugriff auf den Körper (`Entry::body()`). Ein Blob-Verweis auf ein Loch ergibt `Corrupt{HoleRef}` (§10.7).

### 12.5 Redo-Vorallokation

- Ein neues Redo-Segment wird auf `redo_segment_bytes` vorallokiert (`fallocate`, sonst Nullschreiben), bevor der erste Frame hineingeschrieben wird. Dadurch scheitert ein späterer Schreibvorgang nicht erst beim Platzbedarf. ENOSPC zeigt sich bereits bei der Allokation (SD-08).
- Der Nullbereich endet mit `group_len = 0` (§10.4). Beim Replay ist ein Null-Gruppenkopf das Ende, kein Fehler.
- **Reservesegment (INV-LOG-7):** Das jeweils nächste Segment ist vorallokiert und lebendig. Es wird beim Schnitt sofort verwendet, damit der Schnitt selbst keine Allokation braucht.

### 12.6 Blockcache

- Der Cache hält **entschlüsselte** Blöcke nur bis zu einem Budget (`block_cache_bytes`, §20.1). Jeder verdrängte oder freigegebene Block wird mit `zeroize` überschrieben (INV-KEY-4).
- Der Cache wird nie über Cluster- oder Löschgrenzen hinweg geteilt: ein Eintrag gehört zu `(space, cluster, block_uid)`. Ein gegateter oder gelöschter Cluster entfernt seine Einträge sofort (INV-ERA-3).
- Der Cache ist ein Zugriffsbeschleuniger und keine Wahrheit. Ein Cache-Fehler führt zu erneutem Lesen, nie zu einer geänderten Antwort.

---

## 13. Wartung

### 13.1 Lanes und Governor

| Lane | Zweck | Vorrang |
|---|---|---|
| `urgent` | Gate-Verarbeitung, Löschschritte, Recovery-Abschluss | höchster |
| `flush` | Memtable → Run | hoch, bei Stopp-Grenze höchster Nutzer-nahe Vorrang |
| `compact` | Runs zusammenführen | mittel |
| `blobgc` | Blob-Inkarnationen und Blobs räumen | niedrig |
| `idle` | Scrub, Verwaisten-Suche, Ledger-Prüfung | niedrigster |

- Jede Lane hat eine eigene Warteschlange und ein Budget an Bytes je Sekunde. Der **Governor** teilt I/O-Budget zwischen Lanes zu und senkt das Budget der Lanes unter `flush` und `compact`, wenn Nutzer-Schreibe-Latenz über `write_latency_target` liegt (H-11).
- `urgent` wird nie gedrosselt, weil sonst ein Löschauftrag unbegrenzt liegen bliebe (INV-ERA-6).
- Wartung läuft im Hintergrund auf dem Wartungsrang 8 (§17). Sie hält keine Sperre des Intake oder Committers über I/O hinweg.

### 13.2 Flush

1. Memtable eines Space wird eingefroren (neue Memtable übernimmt Schreibe).
2. Die eingefrorene Memtable wird in einen Run `spaces/<s>/<f>.run` geschrieben. Jeder Cluster kommt in eine eigene `ClusterSection` (§10.5). Der Cluster-Schlüssel wird aus `cluster_key` abgeleitet.
3. Run `sync_all`, `rename` aus `.building`, `sync_dir`.
4. Catalog-Edit `files_add` (L0) und `space_flushed(space, stamp)` als **ein** Edit (P-2).
5. Danach darf das Redo bis zum neuen `redo_floor` gekürzt werden (§13.4).

Ein Absturz vor Schritt 4 lässt den Run unreferenziert zurück. Er wird beim Öffnen als verwaist entfernt (§9.2). Die Daten sind dann noch im Redo, weil `redo_floor` nicht gestiegen ist.

### 13.3 Blob-GC (C-09, INV-BLOB-1)

- **Kandidatenauswahl:** `blob_live_delta` zeigt Blobs mit wenig Lebendanteil. Die Auswahl ist nur Hinweis.
- **Wahrheit:** Ein Blob bleibt, solange ein **lebender Run** oder eine **offene View** auf einen seiner Records zeigt. Der Kern prüft das durch Abgleich der Runs, nicht durch Zähler.
- **Umsetzung:** Records, die nicht mehr gebraucht werden, werden per Loch-Überschreiben entfernt und die Inkarnation getauscht (`blob_replace`, §10.7). Nur wenn **kein** Record mehr lebt, wird die Datei entfernt.
- **Umzug** in eine neue `blob_id` ist eine Compaction-Aufgabe der Lane `compact` und kein GC-Schritt.

### 13.4 Redo-Kürzung

- Das Redo darf bis `redo_floor` gekürzt werden, wenn alle Stamps darunter in Runs stehen **und** kein Gate und keine offene Löschung darin fehlt.
- Die Kürzung ist ein Catalog-Edit `redo_floor(floor, floor_stamp, hwm, mark)`. Erst nach dem dauerhaften Edit wird die Redo-Datei entfernt (`remove`, `sync_dir`).
- **INV-LOG-8:** Die Kürzung behält den letzten gültigen Stand, der für Recovery nötig ist. Gelöscht wird nur, was unterhalb des `mark` liegt und gesiegelt ist.

### 13.5 Compaction

- Runs einer Ebene werden zu einem Run der nächsten Ebene zusammengeführt. Die Ausgabe wird wie ein Flush geschrieben (§13.2).
- Ein Cluster in Zustand `Restricted` wird **nicht** entfernt. Er wird in der Ausgabe mit Restriction-Marker geführt. Ein Cluster in `Erasing` wird gar nicht zusammengeführt. Er wird über den Löschpfad behandelt (§15).
- Verdrängte Eingaben bleiben lebendig, bis die letzte `View` auf sie geschlossen ist. Danach `files_remove` (Edit), dann Entfernen.

### 13.6 Scrub

- Die Lane `idle` prüft periodisch jeden Run und jedes Blob: alle Prüfwerte, Datei-MAC (§10.6) und Stamp-Bereiche gegen das Catalog-`meta`.
- Ein Fehler führt zur Quarantäne der Einheit (§5.3). Der Scrub repariert nichts selbst.

### 13.7 Verwaisten-Suche

- Dateien im Verzeichnisbaum, die der Catalog nicht nennt (§9.2), werden erst nach einer **Schonfrist** (`orphan_grace`) entfernt. Die Frist schützt vor einem Lauf, der gerade eine Datei erzeugt, aber ihren Edit noch nicht geschrieben hat.
- Redo-Segmente mit `segment_id ≥ redo_floor` sind nie verwaist (§9.2).

---

## 14. Recovery

Beim Öffnen laufen die Schritte R1–R10 in dieser Reihenfolge. Scheitert einer, ist das Ergebnis `Err(...)` und der Kern startet nicht. Es gibt keinen „best effort“-Start (SD-07, fail-closed).

| Schritt | Aktion | Fehlerfall |
|---|---|---|
| **R1** | `LOCK` nehmen (`ReadWrite` exklusiv, `ReadOnly` geteilt) | `Locked` |
| **R2** | `FORMAT` lesen: Magic, Version, Pflicht-Bits, `integrity_mode` gegen `Config.integrity` (INV-FMT-4) | `Unsupported` / `Corrupt` |
| **R3** | `KEYS` entschlüsseln (nur `Authenticated`/`Encrypted`), `keys_generation` gegen `FORMAT` und `CURRENT` prüfen | `KeyUnavailable` / `Rollback` |
| **R4** | `CURRENT` lesen, Catalog-Dateien laden, Snapshot und Edits replayen. Nur der **letzte** Record darf halb sein und wird verworfen (INV-CAT-3) | `Corrupt` |
| **R5** | **Anker prüfen** (§16.7): gespeicherter `AnchorValue` gegen den Stand aus R4. Der Anker darf nie über dem Stand liegen. | `Rollback` |
| **R6** | `.building.*`-Dateien entfernen, Verwaiste nach Schonfrist markieren (§13.7) | — |
| **R7** | Redo ab `redo_floor` replayen: Segmente in Reihenfolge, Torn-Tail nach INV-LOG-8 | `Corrupt{RedoMidLog}` |
| **R8** | Gate-Frames anwenden: Cluster mit Zustand `Restricted`/`Erasing` aus Catalog und Redo gegated setzen (INV-ERA-2) | — |
| **R9** | Löschprotokoll prüfen: `ERASE-LEDGER` Kette und Kopf gegen Anker (§15.8). Offene `erase_pending` wieder einplanen | `Rollback` / `Corrupt` |
| **R10** | Zustand `Ready` setzen, Schreibe freigeben. Schreibfähig nur, wenn `ReadWrite` und Schritte R1–R9 erfolgreich. | — |

### 14.1 Replay-Regeln

- Ein Change aus dem Redo mit Stamp `≤ space_flushed` (§10.8) ist bereits in einem Run und wird nicht doppelt in die Memtable gesetzt (INV-RECOV-1).
- Ein Change mit Stamp oberhalb der Hochwassermarke `stamp_hw` wird nicht erwartet. Wird er gefunden, ist das `Corrupt` (INV-STAMP-3).
- Replay ist **idempotent**: zweimaliges Replay desselben Redo ergibt denselben Zustand (INV-RECOV-2).
- Ein Change für einen Cluster im Zustand `Erased` wird verworfen und als `Corrupt` gezählt, weil ein Schreiber vor dem Gate nicht hätte schreiben dürfen. Die Zählung geht in `Metrics`.

### 14.2 Rollback-Erkennung (C-03)

- Der Anker (§16.7) speichert `AnchorValue = (erase_epoch, ledger_head, catalog_epoch)`. Die Prüfung in R5 und R9 vergleicht diese Werte mit dem, was der Kern aus Catalog und Ledger rekonstruiert.
- Der Kern erkennt damit das Zurückspielen eines alten Verzeichnisses. Er erkennt **kein** Zurückspielen des Ankers selbst, sondern nur bei externem Anker-Port eine unabhängige Kopie.
- Mit dem eingebauten `FileAnchor` (Port-Standard, §16.7) ist die Erkennung auf den Schutz gegen Teilrückspielen beschränkt. Das wird in §23 und §24 offen gehalten.

### 14.3 Restore

- Ein **Restore** (Wiederherstellen einer Sicherung) ist kein Rollback. Er stellt ein Verzeichnis samt Anker aus derselben Sicherung her. Der Kern prüft dafür `instance_id` und `catalog_epoch` gegen den Anker. Stimmen sie nicht überein, lehnt der Kern ab.
- Ein Restore setzt `erase_epoch` nicht zurück. Löschungen, die seit der Sicherung stattfanden, bleiben wirksam, weil der Löschstand im Ledger steckt (C-02, §15.8).
- Die Wiederherstellung erfolgt nur über `Storage::restore(...)`. Eine einfache Kopie des Verzeichnisses ist kein unterstützter Restore.

---

## 15. Löschung

### 15.1 Ziel und Abgrenzung

Löschung ist die einzige Form, mit der ein Nutzer Daten aus dem Speicher entfernen kann. Der Kern verspricht **Stufen** (ER0–ER7, §15.2), keine pauschale Unwiederbringlichkeit. Jede Stufe ist an einen Prüfschritt gebunden. Aussagen wie „DSGVO-konform“ oder „unwiderruflich“ werden nicht verwendet, solange die Stufe nicht gemessen und in §18 als Invariante geprüft ist (CLAUDE.md-Regel, C-14).

### 15.2 Stufen ER0–ER7

| Stufe | Bedeutung | Nachweis |
|---|---|---|
| **ER0 Logisch** | `Erase` gilt für Leser: Schlüssel im Bereich liefern nichts mehr (§12.3) | Lesetest über alle Schnitte |
| **ER1 Gegated** | Cluster im Zustand `Erasing`; Änderungen werden abgelehnt (§11.3) | Gate-Stamp im Catalog |
| **ER2 Physisch im Log** | Redo-Frames mit betroffenen Körpern sind durch Kürzung oder Neuschreiben entfernt | Kürzung nach `redo_floor` mit `mark` |
| **ER3 Physisch in Runs** | Runs des Clusters sind durch neue Runs ohne ihn ersetzt (Compaction) | Compaction-Edit, alte Dateien entfernt |
| **ER4 Physisch in Blobs** | Records sind durch Nullen ersetzt, Inkarnation getauscht (§10.7) | Scrub über Blob-Records |
| **ER5 Gescrubbt** | Scrub über alle Dateien und Kopien im Verzeichnis bestätigt keine Reste (§15.11) | Löschorakel-Bericht |
| **ER6 Kryptografisch** | Der Clusterschlüssel ist aus Catalog und Speicher entfernt; ohne ihn sind verbliebene Bytes unlesbar (§16.4) | Schlüssel-Entfernung im Edit, Zeroize |
| **ER7 Quittiert** | Ledger-Eintrag mit Kettenkopf ist geschrieben; `ErasureReceipt` ist ausgestellt (§15.8) | Ledger-Kopf im Anker |

- Die Stufen sind **kumulativ**: ER7 setzt ER0–ER6 voraus.
- **Pure/Packed (PD1–PD5, §7):** Bei `Pure` (ein Cluster je Datei) wirkt ER3 mit dem Entfernen der Datei. Bei `Packed` (mehrere Cluster je Datei) entsteht ER3 erst mit der Neuschreibung der Packgruppe. Ein Löschauftrag wartet auf die Packgruppe, nicht auf den einzelnen Cluster.
- **Plain-Modus:** ER6 entfällt, weil es keine Schlüssel gibt. Der Kern meldet in `Plain` die höchste erreichbare Stufe als ER5 und sagt das im Receipt ausdrücklich (INV-ERA-7).

### 15.3 Löschaufträge

- `Eraser::request(EraseReq)` erzeugt einen Auftrag mit Pflicht-`purge_deadline` (§8.7). Der Auftrag wird im Catalog als `erase_pending_add(ticket)` festgehalten, bevor etwas anderes geschieht (INV-ERA-1).
- Der Gate-Schritt (ER1) läuft in Lane `urgent`. Danach arbeiten ER2–ER6 in der Lane `urgent` bzw. `compact`, je nach Stufe.
- Eine Frist, die verstreicht, bevor ER5 erreicht ist, setzt den Auftrag auf `Overdue` und erzeugt ein Ereignis über `Observer` und `Metrics`. Der Kern blockiert nicht und löscht nicht schneller als ER-Stufen zulassen.
- `domain_retired` (§10.8) wird nicht mehr erzeugt. Der Tag ist nur für Import älterer Kataloge reserviert (§15.3, C-06).

### 15.4 Gate und Zustände

Clusterzustand: `Open → Restricted → Erasing → Erased`.

- `Restricted`: Lesen nur mit Holds (§12.3), Schreiben abgelehnt.
- `Erasing`: wie `Restricted`, zusätzlich Löschauftrag aktiv. Zustand bleibt bei Neustart erhalten (INV-ERA-2).
- `Erased`: ER0–ER6 erreicht. Der Cluster-Eintrag bleibt im Catalog als Tombstone, bis ER7 quittiert ist. Danach wird er entfernt, außer die Restriction-Liste verlangt ihn.
- Die Übergänge laufen nur über Gate-Frames (§11.3) und Catalog-Edits. Kein anderer Pfad setzt den Zustand.

### 15.5 Löschdurchlauf (Ablauf)

1. **ER1:** Gate-Frame schreiben, Edit `cluster_state(c, Erasing, g)`.
2. **ER2:** Redo-Segmente, die Körper des Clusters enthalten, werden nach `redo_floor` gekürzt oder das Redo wird ohne den Cluster neu geschrieben (Edit `redo_floor`).
3. **ER3:** Packgruppen und Pure-Runs des Clusters werden neu geschrieben (Compaction ohne den Cluster), Edit `files_add`/`files_remove` atomar pro Gruppe.
4. **ER4:** Blob-Records des Clusters werden durch Nullen ersetzt, Inkarnation getauscht (`blob_replace`).
5. **ER5:** Scrub über alle Dateien, Kopien im Verzeichnis und Caches (INV-ERA-3). Ergebnis ist ein Löschorakel-Bericht (§15.11).
6. **ER6:** Clusterschlüssel-Eintrag aus dem Catalog entfernen (Edit `cluster_state(c, Erased, ...)` mit Löschung des umhüllten Schlüssels), Speicherkopie mit `zeroize` überschreiben.
7. **ER7:** Ledger-Eintrag schreiben, Anker auf den neuen `ledger_head` setzen (§16.7), `ErasureReceipt` ausgeben, `erase_done` und danach `erase_swept` als Edits.

Jeder Schritt ist idempotent. Ein Absturz zwischen zwei Schritten setzt bei Neustart mit dem nächsten Schritt fort, weil der Auftragszustand im Catalog steht (INV-ERA-6).

### 15.6 Zurückspielen eines Redo-Stands (Roll-forward)

- Recovery (§14, R8–R9) wendet Gate-Frames und Löschaufträge an, bevor Schreibe freigegeben wird. Ein Redo mit Körpern eines `Erasing`-Clusters wird beim Replay nicht in die Memtable übernommen, sondern nur in die Löschpipeline gestellt (INV-ERA-2).
- Damit ist Löschung **vorwärtsgerichtet**: Ein Absturz nimmt eine Löschung nie zurück.

### 15.7 Erase-Epoch

- `erase_epoch` (u64) zählt jeden abgeschlossenen ER7 im Ledger. Er steht im Catalog-`reserve` (§10.8) und im Anker (§16.7).
- Ein Catalog oder eine Sicherung mit kleinerem `erase_epoch` als im Anker wird abgelehnt (`Rollback`). Dadurch kann ein Angreifer einen Löschvorgang nicht durch Zurückspielen rückgängig machen (C-02).

### 15.8 Ledger

Datei-Ordner `ERASE-LEDGER/`, append-only, eine Datei je Segment.

```
Eintrag := seq u64 | prev_hash [32] | erase_epoch u64 | cluster [16] | space u16 | level u8 (ER0–ER7) |
           stamp u64 | scope_hash [32] | detail_len u16 | detail | hash [32]
hash    := Hasher(prev_hash ‖ alle Felder außer hash)
```

- `Hasher` ist ein eigener Port (§4, C-07). Der Kern verwendet dafür BLAKE3, Hashgröße 32 B.
- `scope_hash` ist der Hash über die Bereichsbeschreibung, nie der Klartext des Schlüssels (INV-ERA-8). Der Ledger enthält damit keine Nutzerdaten.
- `ErasureReceipt` = `(seq, hash, erase_epoch, level, ledger_head)`. Es beweist, dass ein Eintrag im Ledger steht. Es beweist **nicht**, dass die Daten fehlen. Dafür ist das Löschorakel (§15.11) zuständig.
- **Recovery R9:** Die Kette wird vollständig geprüft. Ihr Kopf muss zu `ledger_head` im Anker passen. Ein fehlender letzter Eintrag nach einem gemeldeten Schritt ist `Rollback`, ein Bruch in der Mitte ist `Corrupt`.

### 15.9 Oberer Schicht-Löschpfad

Segmente oberer Schichten (§10.9) haben die gleichen Stufen. Der Kern löscht sie nach Auftrag über `files_remove` und Entfernen der Datei. Der Inhalt ist für den Kern opak. Die Löschstufen gelten für Segmente wie für Runs, und der Löschorakel-Bericht (§15.11) erfasst sie.

### 15.10 Checkpoint und Rekey-Klon

- **Checkpoint:** `Storage::checkpoint(dir)` erzeugt eine konsistente Kopie. Der Kern schreibt dafür Redo-Frames bis zum aktuellen `stamp`, setzt einen Catalog-Snapshot und kopiert alle lebendigen Dateien. Die Kopie ist ein gültiges Verzeichnis mit eigenem Anker.
- **Rekey-Klon (`rekey_clone`):** Eine Kopie mit neuer KEK, neuen Wurzeln und neuen DEKs. Der Kern liest jeden Block, verschlüsselt ihn neu und schreibt ihn in ein neues Verzeichnis. Die Quelle bleibt unverändert. Der Vorgang ist die einzige Form der Schlüssel-Rotation, die Dateien neu schreibt.
- Eine Kopie vor einer Löschung ist eine Kopie mit dem alten Schlüssel. Ein Restore dieser Kopie nach einer Löschung wird durch `erase_epoch` abgelehnt (§14.3).

### 15.11 Löschorakel

- `Eraser::verify(scope)` durchsucht alle Dateien des Verzeichnisses nach Resten des Scopes. Es prüft Runs, Blobs, Redo, Catalog und Ledger-Details, nicht die Nutzerkopien.
- Der Bericht listet je Datei: Pfad, Treffer nach `scope_hash` oder nach Schlüssel-Präfix, Stufe. Bei `Encrypted` sind Reste nur über Schlüssel nachweisbar, daher prüft das Orakel den Schlüssel-Hash und nicht den Körper.
- Ein Treffer heißt: die Stufe ER5 ist nicht erreicht. Der Auftrag bleibt offen.
- Das Orakel verändert nichts. Es ist ein Nachweis, kein Löschvorgang.

---

## 16. Kryptografie

### 16.1 Modi

| Modus | Schutz | Schlüssel |
|---|---|---|
| `Plain` | CRC-32C gegen Zufallsfehler | keine |
| `Authenticated` | Integrität und Herkunft gegen Manipulation (`Mac16`) | DEK, Wurzel |
| `Encrypted` | Vertraulichkeit und Integrität (AEAD) | DEK, Wurzel, Clusterschlüssel |

- Der Modus ist in `FORMAT` festgelegt und unveränderlich (§10.2).
- Ohne Schlüssel gibt es keine Aussage über Vertraulichkeit. `Plain` schützt nicht vor einem Angreifer mit Dateizugriff.

### 16.2 Schlüsselhierarchie

```
KEK (extern, KeyProvider)          ─umhüllt─►  KEYS
KEYS ─► root_key (je Generation, 32 B, zufällig)
         │   └─ Kdf("ctxs redo v1" ‖ segment_id) ─► Redo-Schlüssel   (nur Encrypted)
         └─ Space-DEK (zufällig, je dek_gen, in KEYS geführt)        → Blöcke, Blobs, Datei-MAC
Cluster-Schlüssel ck(c, epoch): ZUFÄLLIG erzeugt, mit Space-DEK umhüllt, im Catalog (cluster_register)
  ├─ Kdf(ck, "ctxs seal v1" ‖ key_epoch)  → Versiegelung der Cluster-Inhalte (§10.10)
  └─ Kdf(ck, "ctxs blob v1" ...)          → Blob-Records mit Seal::Cluster
```

- **Clusterschlüssel sind zufällig, nicht abgeleitet.** Eine Ableitung aus der Wurzel würde einen einzelnen Cluster nicht kryptografisch löschbar machen (C-04). Das ist die zentrale Entscheidung dieser Spezifikation. Zufällige Clusterschlüssel machen ER6 möglich, ohne den ganzen Space neu zu schreiben.
- **KEK-Rotation** ist O(1): `KEYS` wird mit neuer KEK neu umhüllt (§10.3).
- **Wurzel-Rotation** erzeugt eine neue `root_gen`. Alte Generationen bleiben, bis alle Dateien mit ihnen neu geschrieben sind (§16.3).
- **Space-DEK-Rotation** geschieht mit `rekey_clone` (§15.10) oder durch Compaction, die alte Blöcke neu verschlüsselt.

### 16.3 Nonces und Rotationsgrenzen

- Jede Versiegelung benutzt eine **24-Byte-Zufallsnonce** aus `Entropy` (SD-14, §10.1).
- **Rechnung:** Bei `n` Versiegelungen je Schlüssel ist die Kollisionswahrscheinlichkeit ≈ `n² / 2^193`. Für `n = 2^32` ergibt das ≈ `2^-129`. Die Schranke `2^32` je Schlüssel ist deshalb konservativ. Der Zähler je Schlüssel steht im Catalog (`key_epoch`) und löst bei Erreichen die Rotation aus.
- Die Rotation greift bei `2^32` Versiegelungen je Clusterschlüssel und je Space-DEK. Eine höhere Schranke ist nur über eine Spezifikationsänderung zulässig.
- Nonces werden nie wiederverwendet, auch nicht nach einem Absturz. Ein Absturz verwirft die Nonce, ohne die Schlüsselzähler zu beeinflussen.

### 16.4 Kryptografische Löschung (ER6)

- Der Clusterschlüssel `ck(c, epoch)` wird aus dem Catalog entfernt (Edit), im Speicher mit `zeroize` überschrieben (INV-KEY-4) und aus dem Blockcache entfernt (§12.6).
- Danach sind alle Dateien mit diesem Schlüssel ohne den Schlüssel unlesbar. Das gilt für Runs, Blobs und Redo-Körper dieses Clusters.
- **Grenze:** Kopien des Catalogs, die den umhüllten Schlüssel noch enthalten, können ihn nicht offenbaren, solange der Space-DEK fehlt. Der Space-DEK bleibt aber bestehen. Deshalb verlangt ER6 zusätzlich, dass der alte Catalog gelöscht und der neue Snapshot durch Rollover geschrieben wird (§10.8). Ob eine alte Kopie in einem Backup liegt, kann der Kern nicht wissen. Die Restore-Regel (§14.3) und das Löschorakel (§15.11) decken diesen Fall ab, nicht die Kryptografie allein.

### 16.5 Schlüsselmaterial im Speicher (SecretAlloc)

- Alle Schlüssel sind vom Typ `Secret32`. Er hat keine `Debug`-, `Display`- oder `Clone`-Implementierung und wird beim Verwerfen mit `zeroize` gelöscht.
- Schlüssel liegen in Speicher, der über `SecretAlloc` (§4) angefordert wird. Der Port kann `mlock` oder gleichwertige Schutzmechanismen bereitstellen. Ist er nicht verfügbar, meldet der Kern dies im Start-Report (`SecretAllocDegraded`), ohne zu scheitern.
- Schlüssel verlassen den Kern nie über Logs, Metriken, Fehlermeldungen oder `ErasureReceipt` (INV-KEY-3).

### 16.6 KeyProvider

- Der Port `KeyProvider` liefert die KEK. Der Kern ruft ihn nur beim Öffnen und bei Rotation auf.
- **make_durable (R2-Stärke, übernommen):** `KeyProvider::make_durable()` muss zurückkehren, bevor eine neue KEK-Version in `KEYS` benutzt wird. Ohne diesen Aufruf wird `KEYS` nicht umgehängt (INV-KEYP-1).
- Der Kern prüft beim Öffnen, ob `KeyProvider` eine KEK liefert, die `KEYS` öffnet. Schlägt das fehl, ist das Ergebnis `KeyUnavailable`, nie ein neuer Start mit leeren Schlüsseln.

### 16.7 Anker

```
AnchorValue := (erase_epoch u64, ledger_head [32], catalog_epoch u64)
Anchor-Port := read() -> Option<AnchorValue> | write(value) (atomar, durable)
```

- Der Anker wird **nach** jedem ER7-Schritt und nach jedem Catalog-Rollover geschrieben.
- Die Recovery-Schritte R5 und R9 vergleichen den Anker mit dem Stand, den Catalog und Ledger ergeben (§14.2).
- **Standard-Anker `FileAnchor`:** eine Datei außerhalb des Speicherverzeichnisses, atomar ersetzt. Er schützt gegen das Zurückspielen des Speicherverzeichnisses, nicht gegen das Zurückspielen des Ankers selbst (C-03, offen in §24, O-7).
- Ein Anker mit leerem Wert bei einem Speicher, der bereits Löschungen oder Catalog-Epochen hat, ist `Rollback`.
- Die Anker-Schreibfolge ist: erst Ledger und Catalog dauerhaft, dann Anker. Ein Absturz dazwischen ergibt nur einen zu alten Anker, nie einen zu neuen.

---

## 17. Nebenläufigkeit

### 17.1 Sperrränge

Sperren werden nur in **aufsteigender** Rangfolge genommen. Eine Sperre niedrigeren Rangs darf nicht unter einer höheren angefordert werden. Rang 5 entfällt (SD-22), die Nummerierung bleibt zur Stabilität der Diagnose erhalten.

| Rang | Sperre | Schützt | Gehalten über I/O? |
|---|---|---|---|
| 1 | `committer` | einziger Schreiber des Redo, Gruppenbildung, Stamp-Vergabe | ja (Gruppen-`sync_data`) |
| 2 | `intake` | Warteschlange der Eingänge, Idempotenz-Index | nein |
| 3 | `catalog_edit` | Schreiben von Catalog-Edits und Rollover | ja (Edit-`sync`) |
| 4 | `state` | Zustand `Ready`/`Degraded`/`Poisoned`, Views | nein |
| — | — | Rang 5 entfällt | — |
| 6 | `registry` | Dateiregister lebendiger Runs und Blobs | nein |
| 7 | `cluster_table` | Clustertabelle und Gate-Zustände | nein |
| 8 | `maint` | Wartungsschritte (Flush, Compaction, GC, Löschung) | nein |
| 9 | `notify` | Ereignisse an `Observer` und `Listener` | nein |

- **Memtable:** lock-frei (`crossbeam-skiplist`). Sie hat keine Sperre im Rang und wird nur nach der Redo-Dauerhaftigkeit geschrieben (§11.1).
- **INV-LOCK-1:** Keine Sperre wird über einen Aufruf des Nutzers (`Hooks`, `Listener`, `Observer`) gehalten. Ereignisse werden nach Freigabe in Rang 9 zugestellt.
- **INV-LOCK-2:** Die Rangordnung ist durch Tests mit `loom` und `shuttle` geprüft (§19).

### 17.2 Claims und Graveyard

- Ein **Claim** ist die Zusicherung einer Ressource (Datei, Block, View). Er wird beim Lesen erworben und beim Verwerfen freigegeben.
- Ein Claim wird `obsolete`, wenn die Ressource durch Compaction ersetzt wurde. Neue Leser bekommen die Nachfolgerin, bestehende Claims behalten die alte Datei bis zur Freigabe.
- Der **Graveyard** hält obsolete Dateien, bis kein Claim mehr auf sie zeigt. Erst danach werden sie entfernt (INV-VIEW-1, INV-BLOB-1).

### 17.3 Ereignisse und Zustellung

- `Listener` erhält Ereignisse über einen begrenzten Kanal. Ein voller Kanal blockiert den Kern nicht; das Ereignis wird verworfen und als `Dropped` gezählt. Überlauf ist nie stille Datenänderung.
- `Observer` und `Metrics` werden in Rang 9 aufgerufen, nie unter einer Sperre der Ränge 1–8.

### 17.4 Modelltests

- Die Rangordnung und Claim-Übergänge werden als Modell mit `loom` (vollständige Verschränkung kleiner Szenarien) und `shuttle` (zufällige lange Läufe) geprüft.
- Jedes Modell ist an einen Invariantenkatalog-Eintrag (§18) gebunden.

---

## 18. Invariantenkatalog

Die Tabelle ist normativ. Die Spalte `ID` ist das Format, das `tests/invariants.rs` liest. Jede ID wird genau einmal definiert und in mindestens einem Test oder Modell geprüft.

| ID | Aussage | Prüfung |
|---|---|---|
| INV-FS-1 | Jeder Port-Aufruf, der Durabilität verspricht, ist ohne `FsCaps`-Bestätigung nicht zulässig (`FsPolicy::Strict`). | Fail-closed-Test mit `FaultFs` |
| INV-KEYP-1 | Eine neue KEK-Version wird erst nach `KeyProvider::make_durable()` in `KEYS` verwendet. | Absturzmatrix |
| INV-FMT-1 | Varint wird nur kanonisch gelesen; jede Abweichung ist `Corrupt`. | proptest, Goldens |
| INV-FMT-2 | Längen werden gegen §20.1 und die Restlänge geprüft, bevor allokiert wird. | Fuzz-Test, Längenmatrix |
| INV-FMT-3 | Authentifizierte Frames, Blöcke und Records binden Art, Ort und Zugehörigkeit in der AAD. | Vertauschungstest |
| INV-FMT-4 | `integrity_mode` in `FORMAT` muss `Config.integrity` entsprechen, sonst `Unsupported` vor jedem weiteren Lesen. | Downgrade-Test |
| INV-COMMIT-1 | Ein bestätigter `submit` ist nach Absturz vorhanden. | Absturzmatrix |
| INV-COMMIT-2 | Frames eines Changes sind atomar; eine Gruppe ist nur Fsync-Einheit. | Absturzmatrix, proptest |
| INV-COMMIT-3 | Volatile-Changes liegen nie in derselben Gruppe wie dauerhafte und werden nie als dauerhaft bestätigt. | Modelltest |
| INV-LOG-1 | Ein Frame enthält genau einen Change. | Format-Test |
| INV-LOG-2 | Ein Frame ist beim Replay ganz gültig oder beendet das Log. | Torn-Tail-Matrix |
| INV-LOG-3 | Stamps im Log sind streng aufsteigend und dürfen Lücken haben. | proptest |
| INV-LOG-4 | Jede Prozessinstanz und jedes Gate beginnt ein neues Segment. | Modelltest |
| INV-LOG-5 | Reserviert (nicht belegt). | — |
| INV-LOG-6 | Ein gesiegeltes Segment mit Defekt ist `Corrupt` (fail-closed). | Defekt-Injektion |
| INV-LOG-7 | Alles bis zur letzten `redo_mark` ist gültig; das Reservesegment ist lebendig. | Absturzmatrix |
| INV-LOG-8 | Torn-Tail endet das Log nur ohne prüfbar gültige Folgegruppe, sonst `Corrupt{RedoMidLog}`. | Torn-Tail-Matrix |
| INV-CAT-1 | Eine Datei ist lebendig genau dann, wenn ein dauerhafter `files_add` ohne nachfolgendes `files_remove` vorliegt. | Modelltest |
| INV-CAT-2 | Ein Edit ist genau ein Record (P-2); Rollover ist atomar über `CURRENT`. | Absturzmatrix |
| INV-CAT-3 | Nur der letzte Record darf halb sein; jeder frühere Defekt ist `Corrupt`. | Defekt-Injektion |
| INV-KEY-2 | IDs und Stamps werden erst benutzt, nachdem ein dauerhafter `reserve`-Edit sie überdeckt; Hochwassermarken steigen nur. | Absturzmatrix |
| INV-KEY-3 | Schlüssel erscheinen nie in Logs, Metriken, Fehlern oder Receipts. | Scan-Test |
| INV-KEY-4 | Entschlüsselte oder entfernte Schlüssel und Blöcke werden mit `zeroize` überschrieben. | Speichertest, Review |
| INV-STAMP-3 | Stamp-Hochwassermarke steigt nur; ein Change oberhalb der Marke bei Recovery ist `Corrupt`. | Recovery-Test |
| INV-PL-4 | `pack_assign` erscheint nur zusammen mit den Dateimengen der Packgruppe. | Modelltest |
| INV-BLOB-1 | Ein Blob oder eine Inkarnation wird nur entfernt, wenn kein lebender Run und keine offene View darauf zeigt. | Modelltest, Löschtest |
| INV-BLOB-2 | Kein Dedup: jeder `BlobRef` zeigt auf einen eigenen Record. | Format-Test |
| INV-BLOB-3 | Löcher sind Nullen gleicher Länge und werden nie entschlüsselt; ein Verweis auf ein Loch ist `Corrupt{HoleRef}`. | Löschtest |
| INV-SEG-1 | Jeder Block wird vor Ausgabe geprüft. | Defekt-Injektion |
| INV-ERA-1 | Ein Löschauftrag ist im Catalog festgehalten, bevor etwas anderes geschieht. | Absturzmatrix |
| INV-ERA-2 | Ein `Erasing`-Cluster ist nach Neustart wieder gegated und nimmt keine Änderungen an. | Recovery-Test |
| INV-ERA-3 | Gegatete oder gelöschte Cluster verlassen Caches und Views sofort. | Modelltest |
| INV-ERA-4 | Ein Schlüssel im Löschbereich liefert nie einen Wert, auch nicht aus Runs. | Lesetest über alle Schnitte |
| INV-ERA-5 | Erased-Bereiche tragen den Löschstamp und werden nur entfernt, wenn ER7 quittiert ist. | Modelltest |
| INV-ERA-6 | Löschschritte sind idempotent und werden nie gedrosselt, bis sie abgeschlossen sind. | Absturzmatrix, Lane-Test |
| INV-ERA-7 | In `Plain` meldet das Receipt höchstens ER5. | Receipt-Test |
| INV-ERA-8 | Der Ledger enthält keinen Klartext des Schlüssels, nur `scope_hash`. | Scan-Test |
| INV-RECOV-1 | Redo-Changes unterhalb `space_flushed` werden nicht doppelt übernommen. | Recovery-Test |
| INV-RECOV-2 | Replay ist idempotent. | Recovery-Test, proptest |
| INV-VIEW-1 | Eine View hält alle von ihr gelesenen Dateien bis zu ihrem Ende. | Modelltest |
| INV-HOLD-1 | Holds sind dauerhaft im Catalog; ein Hold verhindert ER6 bis zu seiner Freigabe. | Modelltest |
| INV-LOCK-1 | Keine Sperre wird über Nutzeraufrufe gehalten. | Review, Lock-Test |
| INV-LOCK-2 | Die Rangordnung 1–9 wird nie verletzt. | loom, shuttle |
| INV-CLUSTER-1 | Ein `Purged`-Cluster wird nie wiederverwendet. | Modelltest |
| INV-CONS-1 | `view_durable` sieht nur Stamps bis `durable_stamp`. | Modelltest |
| INV-LIM-1 | Changes über `max_change_ops` oder `max_change_bytes` werden vor jeder Allokation mit `ChangeTooLarge` abgewiesen. | Grenztest |
| INV-PL-1 | Ein Run, Blob oder Segment ist entweder Pure oder Packed, nie beides. | Format-Test |
| INV-PL-2 | In Packed-Dateien bildet jeder Cluster einen zusammenhängenden Abschnitt; Blöcke sind clusterrein. | Format-Test |
| INV-PL-3 | Compaction mischt nur innerhalb einer Wartungsgruppe. | Modelltest |
| INV-SCAN-1 | `scan_physical` ist rein lesend und liefert nur Anzahl und Dateikennung. | Test mit schreibgesperrtem `FaultFs` |
| INV-STAMP-1 | Stamps sind streng monoton über angenommene Changes. | proptest |
| INV-STAMP-2 | Ein Stamp-Überlauf ist `Exhausted` und nie ein Umlauf. | Grenztest |

Ein Eintrag mit „Reserviert“ hat keine Verpflichtung und darf nicht als Test verwendet werden; er bleibt aus Gründen der Stabilität der Nummerierung stehen.

---

## 19. Verifikation

### 19.1 Testwerkzeuge

| Werkzeug | Zweck |
|---|---|
| `FaultFs` | Fehlerinjektion an jedem Port-Aufruf (ENOSPC, EIO, kurzes Schreiben, Absturz vor und nach `sync`) |
| `MemFs` | deterministisches In-Memory-Dateisystem für Modelltests |
| Absturzmatrix | jede Kombination aus Absturzpunkt (vor und nach jedem `sync`) und Operation (§11, §13, §15) |
| `proptest` | Format-Roundtrips, Varints, Replay-Idempotenz |
| `loom` / `shuttle` | Sperrränge und Claim-Modelle (§17) |
| `cargo-mutants` | prüft, ob Tests Fehler in Vergleichen und Schreibreihenfolgen erkennen |
| `cargo-deny` | Lizenz- und Advisory-Prüfung der Laufzeitabhängigkeiten |
| Goldens (Anhang A) | Bytefolgen aus dem unabhängigen Referenzcodierer |

### 19.2 Pflichtläufe

- Jeder Invariantenkatalog-Eintrag hat mindestens einen Testnamen, der seine ID trägt.
- Die Absturzmatrix deckt jede Schreiboperation der Abschnitte 11, 13, 14 und 15 ab.
- Die Löschprüfung (§15.11) läuft nach jeder Löschung in Tests und im Scrub.
- `cargo-mutants` ist für `format`, `redo`, `catalog`, `erase` mit Schwelle `0` überlebender Mutanten bei Vergleichs- und Schreibreihenfolgen-Mutanten gefordert (§20.4). Die Mutationsschwellen stehen in §20.4.

### 19.3 Testdateien (normativ)

- `crates/contextra-storage/tests/invariants.rs` liest die Tabelle in §18 und prüft, dass jede ID genau einmal definiert ist und mindestens einen Testnamen mit ihr trägt.
- `crates/contextra-storage/tests/golden_anhang_a.rs` prüft alle Werte aus Anhang A gegen die Implementierung.
- `crates/contextra-storage/tests/format/golden_test.rs` prüft ein FORMAT-Golden im Layout des Altstands (51 Byte). Dieses Layout ist in R7 nicht mehr gültig (§10.2: 53 Byte bei `Plain`). Der Test wird in G-2 durch ein Golden nach A.1 ersetzt. Bis dahin gilt er nicht als Nachweis.

### 19.4 Referenzcodierer

- `tools/reference/generator.py` ist ein **unabhängiger** Python-Codierer. Er erzeugt die Goldens. Er teilt keinen Code mit dem Kern.
- Ein Golden wird nur übernommen, wenn der Referenzcodierer und der Kern übereinstimmen. Eine Abweichung ist ein Fehler in einem der beiden, nie ein Anlass, das Golden anzupassen.

---

## 20. Konfiguration

### 20.1 Limits

Die Grenzen gelten für Eingaben, Dateilängen und Speicher. Sie sind in `Config` einstellbar, aber nie größer als die Obergrenze der Version.

| Limit | Wirkung | Herkunft |
|---|---|---|
| Schlüssellänge, Wertlänge, Ops je Change, Frame-Länge | Schema-Grenzen; Überschreitung ⇒ `TooLarge` (vor Allokation) | §6.8 |
| `memtable_limit` | Stopp-Grenze je Space (Busy, §11.5) | §11.5 |
| `block_cache_bytes` | Budget für entschlüsselte Blöcke (§12.6) | §12.6 |
| `view_max_age` | Lease-Dauer einer View (§12.1) | §12.1 |
| `listener_queue`, `listener_deadline` | Puffer und Frist je Listener (§8.6) | §8.6 |
| `idem_window` | Dauer der Idempotenz-Indexierung (§11.4) | §11.4 |
| `enospc_cooldown` | Sperre nach ENOSPC (§11.6) | §11.6 |
| `orphan_grace` | Schonfrist für verwaiste Dateien (§13.7) | §13.7 |
| `write_latency_target` | Ziel für den Governor (§13.1) | §13.1 |

### 20.2 Standardwerte

| Parameter | Standard | Status |
|---|---|---|
| `group_max_bytes` | 1 MiB | festgelegt |
| `group_max_delay` | 2 ms | festgelegt, Messung in G-8 (H-1) |
| `redo_align` | 4 096 B | festgelegt (Sektorgröße als Mindestausrichtung) |
| `redo_segment_bytes` | Entwurf, in G-8 zu bestimmen | offen |
| `blob_target_bytes` | 64 MiB | festgelegt |
| `DataBlock`-Ziel | 16 KiB | festgelegt, Messung in G-8 |
| Bloom `bits_per_key`, `k` | 10, 7 | festgelegt (§10.5.1) |
| `memtable_limit`, `block_cache_bytes`, `view_max_age`, `listener_queue`, `listener_deadline`, `idem_window`, `enospc_cooldown`, `orphan_grace`, `write_latency_target` | kein Wert in dieser Revision | Entwurf, in G-8 zu bestimmen |
| `purge_deadline` | **kein Standard** (SD-06) | Pflicht |
| `integrity` | **kein Standard** | Pflicht |

Eine Zahl ohne Status „festgelegt“ ist kein Versprechen. Sie wird erst durch eine Messung nach §21 zu einer Zusage.

### 20.3 Konfigurationsfehler

- Ein fehlender Pflichtparameter ist `Err(Config{missing})` beim Öffnen, nie ein Standardwert.
- Ein Wert über der Obergrenze ist `Err(Config{range})`.
- Eine Kombination, die einen Invarianten verletzt (zum Beispiel `group_max_bytes` größer als `redo_segment_bytes`), ist `Err(Config{conflict})`.

### 20.4 Mutationsschwellen

- `cargo-mutants` läuft für `format`, `redo`, `catalog`, `erase`.
- **Schwelle 0** für überlebende Mutanten in Vergleichen und Schreibreihenfolgen.
- Für übrige Mutanten gibt es keine Schwelle. Der Bericht wird im Ordner `evidence/` abgelegt.

### 20.5 Allokationsbudget (P-11)

- Die Spitzen-Heapnutzung des Kerns ist beschränkt durch: `block_cache_bytes + Σ_space memtable_limit + Redo-Puffer + Konstante`.
- Der Heißpfad (`submit`, `get`) allokiert je Aufruf höchstens aus einem vorab angelegten Puffer. Ein zählender Allokator prüft das in Tests.
- Eine Verletzung ist ein Testfehler, kein Warnhinweis.

---

## 21. Hypothesen

Jede Hypothese hat ein Messkriterium. Eine Hypothese wird in G-8 bestätigt, verworfen oder durch eine Zahl ersetzt. Bis dahin gilt keine Leistungszusage.

| ID | Hypothese | Messkriterium |
|---|---|---|
| H-1 | Gruppen-Fsync dominiert die Commit-Latenz; die Gruppenbildung reduziert sie unter Last. | p99 eines bestätigten Einzel-Change bei Last L, mit und ohne Gruppenbildung |
| H-2 | Ein Lesezugriff in einem Run liest höchstens einen Datenblock, wenn der Filter negativ ist oder der Schlüssel existiert. | Zählung der Blockzugriffe je Lesezugriff |
| H-3 | Der Schlüsselfilter ist bei `bits_per_key = 10` unter der Falsch-Positiv-Rate, die die Blockzugriffe begrenzt. | gemessene Falsch-Positiv-Rate über Zufallsschlüssel |
| H-4 | Die Recovery-Zeit wächst linear mit dem Redo-Umfang seit dem letzten `redo_floor`. | Recovery-Zeit über Redo-Größen |
| H-5 | Die Compaction-Schreibverstärkung bleibt bei der gewählten Ebenenstruktur beschränkt. | Verhältnis geschriebener zu eingegebenen Bytes |
| H-6 | ER0–ER2 sind unabhängig von der Speichergröße (O(1) je Cluster). | Zeit für Löschung bei wachsendem Speicher |
| H-7 | Der Ablauf einer Packgruppe ist O(Dateien der Gruppe). | Zeit je Packgruppe über Dateizahlen |
| H-8 | Die Schlüsselrotation über `rekey_clone` ist linear in der Datenmenge. | Zeit über Datenmengen |
| H-9 | Die Lesetrennung durch `Seal::Cluster` kostet höchstens einen konstanten Faktor je Block. | Lesedurchsatz mit und ohne Seal |
| H-10 | Der Blockcache ist bei typischen Zugriffsmustern wirksam. | Trefferquote |
| H-11 | Der Governor hält die Schreib-Latenz unter `write_latency_target`, ohne `urgent` zu drosseln. | Latenzverlauf unter Lastwechsel |
| H-12 | `Volatile`-Schreibe hat keinen Fsync und ist deshalb deutlich schneller als dauerhafte. | Latenzvergleich |
| H-13 | Die Pufferwiederverwendung hält den Heißpfad ohne Allokation. | zählender Allokator (§20.5) |
| H-14 | Ein Ribbon-Filter ist bei gleicher Größe genauer als der Bloom-Filter. | Vergleichsmessung (O-13) |
| H-15 | Ein Ausfall des Speichers während der Löschung (ER1–ER6) führt nach Neustart zur selben Endstufe. | Fault-Injektion über alle Löschschritte |
| H-16 | Die Vorallokation des Redo verhindert ENOSPC nach Schreibbeginn. | Füllversuch bis zum Platzende |
| H-17 | Das Löschorakel (§15.11) findet alle Reste im Verzeichnis, sofern der Scope im Klartext oder über den Schlüssel-Hash erfassbar ist. | Vergleich mit gezielt eingebrachten Resten |
| H-18 | Der Rollback-Schutz über `FileAnchor` erkennt das Zurückspielen des Verzeichnisses ohne Anker. | Rollback-Fault-Injektion |

---

## 22. Phasen und Gates

| Gate | Inhalt | Voraussetzung |
|---|---|---|
| **G-0** | Spezifikation R7 als einzige normative Quelle, Register (§23–§25) vollständig | erfüllt mit dieser Revision |
| **G-1** | Ports-Crate `contextra-storage-ports` und `MemFs`, `FaultFs` | G-0, O-1, O-12 |
| **G-2** | Format-Codecs, Varint, Checksummen, Goldens aus `tools/reference/generator.py` | G-1 |
| **G-3** | Redo, Gruppen, Gate-Frames, Torn-Tail, Absturzmatrix | G-2 |
| **G-4** | Catalog, Hochwassermarken, Anker, Recovery R1–R10 | G-3 |
| **G-5** | Lesen und Schreiben über Memtable und Runs, Iteratoren, Views | G-4 |
| **G-6** | Löschung ER0–ER7, Ledger, Löschorakel, Checkpoint, Restore | G-5 |
| **G-7** | Krypto: Schlüsselhierarchie, Rotation, `rekey_clone`, Zeroize | G-6 |
| **G-8** | Nebenläufigkeit (loom/shuttle), Messung der Hypothesen H-1…H-18, Festlegung der Standardwerte (§20.2) | G-7 |
| **G-F** | Format-Freeze (§10.12) | G-2 bis G-8, alle Goldens verifiziert |

- Phasen laufen in dieser Reihenfolge. Ein Gate gilt erst, wenn seine Tests in der CI grün sind.
- Die Kern-API (§8) wird mit G-5 stabil. Die stabilen Flächen für aufsetzende Schichten (§8.9) gelten ab G-6.
- Die aufsetzenden Crates werden nach G-6 angepasst. Keine Anpassung vor G-6.

---

## 23. Entscheidungsregister

### 23.1 Entscheidungen SD

| ID | Entscheidung |
|---|---|
| SD-04 | Jede Dateiart hat Version 1 und Feature-Bits; unbekannte Pflicht-Bits ⇒ `Unsupported` (§10.1). |
| SD-06 | `purge_deadline` ist Pflichtparameter ohne Standardwert (§8.7). |
| SD-07 | Recovery ist fail-closed; es gibt keinen Best-effort-Start (§14). |
| SD-08 | Redo wird vorallokiert; ENOSPC führt zur Schreibsperre bis `resume_writes` (§11.6, §12.5). |
| SD-14 | Nonces sind 24-Byte-Zufallswerte; es gibt keinen Zählerzustand (§10.1, §16.3). |
| SD-19 | `Volatile` ist ein eigener Space-Typ, nur in der Memtable (§11.2). |
| SD-22 | Rang 5 entfällt; die Sperrordnung ist 1–9 (§17.1). |
| SD-29 | Körper reisen im Redo-Frame; ein `sync_data` je Gruppe (§10.4). |
| SD-36 | Kryptografische Primitive liegen im Port, das Protokoll im Kern (§4). |
| SD-56 | `SecretAlloc` ist optional; `Secret32` hat keine `Debug`-, `Clone`- oder `Serialize`-Implementierung (§4, §16.5). |
| SD-64 | Beschleunigte Checksum-Varianten müssen die Goldens aus Anhang A bestehen (§4). |

### 23.2 Entscheidungen D (Konsolidierung R7)

| ID | Entscheidung |
|---|---|
| D-02 | `contextra-ports` gehört nicht zum Kern und wird nicht verwendet. Der Kern nutzt `contextra-storage-ports`. |
| D-05 | Krypto-Crates sind im Kern verboten. Das Referenz-Primitiv liegt in `contextra-crypto-ref` (dev) oder in einem Port außerhalb des Kerns (§3.5). |
| D-06 | Die Schlüsselwurzel ist `root_key` je Generation; die KEK umhüllt nur `KEYS` (§10.3, §16.2). |
| D-07 | `destroy` ist idempotent und ist bei Rückkehr durabel (§4, KeyProvider). |
| D-08 | Gewichtssummen sind `u64` (C-13). |
| D-10 | Volatile und dauerhafte Ziele dürfen nicht in einem Change gemischt werden: `Forbidden{MixedDurability}` (§6.4, C-07). |
| D-12 | Der Hasher für die Ledger-Kette ist unkeyed (§4, §15.8). |
| D-14 | Jede Prozessinstanz beginnt ein neues Redo-Segment (§10.4). |
| D-15 | Die Cluster-Versiegelung bindet keinen Stamp in die AAD (§10.4, C-16). |
| D-16 | Stabile Flächen für aufsetzende Schichten sind über semver festgelegt (§8.9). |

### 23.3 Abhängigkeiten des Kerns

Erlaubt sind nur: `std`, `contextra-storage-ports`, `zeroize`, `crossbeam-skiplist`, `bytes` (optional). Jede weitere Abhängigkeit braucht einen Eintrag in dieser Liste und eine Prüfung mit `cargo-deny` (`advisories`, `bans`, `sources`). Krypto-Crates sind nicht erlaubt (D-05).

### 23.4 Annahmen

| ID | Annahme |
|---|---|
| AN-01 | Das Dateisystem gibt bei `sync_data` und `sync_all` die beschriebene Dauerhaftigkeit, sofern `FsCaps` das bestätigt. Bestätigt es das nicht, ist der Betrieb fail-closed (INV-FS-1). |
| AN-02 | Der Zufallsgenerator (`Entropy`) ist kryptografisch sicher. |
| AN-03 | Der Angreifer hat Dateizugriff, kennt aber nicht den Schlüssel (§5.2). |
| AN-04 | Der Anker ist ein eigener Speicher, den der Angreifer nicht unbemerkt zurücksetzt (O-7). |
| AN-05 | Zeitangaben aus `Clock` sind nur für Fristen, nie für Ordnung. Ordnung trägt der Stamp. |
| AN-06 | Ein Prozess mit Schreibe-Rechten ist der einzige Schreiber des Verzeichnisses (`LOCK`, R1). |

---

## 24. Offene Entscheidungen

| ID | Frage | Verantwortlich, Frist |
|---|---|---|
| O-1 | Toolchain-Pin und MSRV. `rust-toolchain.toml` pinnt 1.89.0, das in der Arbeitsumgebung nicht lädbar ist. Entscheidung über Pin und Ersatz (`stable`) vor G-1. | Maintainer, vor G-1 |
| O-2 | Rechtliche Löschfristen und Ausnahmen. Der Kern trifft keine Rechtsaussage (§1). | Rechtsberatung, außerhalb des Kerns |
| O-3 | Ablauf von Restrictions: nur Vermerk entfallen, nie automatisch löschen (§8.4). | Produkt, vor G-6 |
| O-4 | Windows mit voller Durabilität in Version 1 (NZ-8). | Port-Implementierung, nach G-8 |
| O-5 | Inhaltsadressierung und Dedup (NZ-11). | obere Schicht, nach G-6 |
| O-6 | Ob `contextra-crypto-ref` als eigener Dev-Crate bleibt oder im Workspace-Testcode aufgeht. | Maintainer, vor G-2 |
| O-7 | Externer Anker gegen Zurückspielen des Ankers selbst. `FileAnchor` schützt das nicht (§14.2, §16.7). | Betreiber, vor G-F |
| O-8 | Schnittstelle des Cold-Tier-Ports (`ColdTier`), Speicherung über Feature `COLD_TIER`. | Spezifikation, vor G-6 |
| O-9 | Kompressionscodec. Das Feld ist reserviert, der Codec ist nicht spezifiziert (NZ-10). | Spezifikation, nach G-F |
| O-10 | Anbindung eines Schlüsselverwaltungsdienstes über `KeyProvider`. | Betreiber, nach G-7 |
| O-11 | Verhalten bei überfälligen Löschaufträgen (`Overdue`): nur Ereignis, oder Betriebsstopp. | Produkt, vor G-6 |
| O-12 | Benannte Reviewer für G-0 und Änderungen an §4, §5, §9–§11, §14–§16, §18 (Anforderung aus §1.5). | Maintainer, vor G-1 |
| O-13 | Bloom- oder Ribbon-Filter. Die Entscheidung braucht eine Messung (H-3, H-14). | Maintainer, vor G-F |

---

## 25. Widerspruchsregister

Jede Zeile beschreibt einen Widerspruch aus den Vorfassungen (Anhang B) und die Entscheidung in R7.

| ID | Widerspruch in den Vorfassungen | Entscheidung in R7 |
|---|---|---|
| C-01 | `max_inline_bytes` und `value_separation_min` waren zwei Schwellwerte. | Ein einziger Schwellwert: `value_separation_min` (§6.3). |
| C-02 | `erase_epoch` war nicht definiert. | Definiert im Catalog und im Anker, ein Wert je ER7 (§15.7). |
| C-03 | Restore und Rollback waren vermischt. | Getrennt: Rollback wird erkannt, Restore ist ein eigener Pfad mit Prüfung gegen den Anker (§14.2, §14.3). |
| C-04 | Cluster-Schlüssel waren aus der Wurzel abgeleitet und damit nicht einzeln löschbar. | Clusterschlüssel sind zufällig und im Catalog umhüllt (§16.2). |
| C-05 | Der Stamp sollte lückenlos sein, im Log gab es Lücken. | Lücken sind erlaubt; streng aufsteigend (§6.2, INV-LOG-3). |
| C-06 | `domain_retired` war erzeugt, aber nie gelesen. | Nicht erzeugt; nur für Import reserviert (§15.3). |
| C-07 | Volatile-Mischung und INV-COMMIT-2 widersprachen sich. | Mischung verboten (D-10), Volatile in eigener Gruppe (§11.2). |
| C-08 | Der Datei-MAC fehlte. | Eingeführt in `Authenticated`/`Encrypted` (§10.6). |
| C-09 | `blob_live_delta` wurde als Wahrheit verwendet. | Nur Auswahlhilfe; Lebendigkeit über Runs (§13.3, INV-BLOB-1). |
| C-10 | Das Siegelbit hieß uneinheitlich. | Bit 7 (`0x80`) im Frame, überall gleich (§10.4). |
| C-11 | Blobs wurden in place geändert, Verweise konnten ins Leere zeigen. | Inkarnationen mit gleichen Offsets (§10.7). |
| C-12 | Der Hasher-Port fehlte. | Eingeführt in §4 (`Hasher`). |
| C-13 | Gewichtssummen waren `u32` und konnten überlaufen. | `u64` (D-08). |
| C-14 | Datenschutzbegriffe standen ohne Messbindung. | Nur mit Stufe ER0–ER7 und Prüfschritt (§15.1). |
| C-15 | Die Reihenfolge von Anker und Ledger war nicht festgelegt. | Erst dauerhaft Ledger und Catalog, dann Anker (§16.7). |
| C-16 | Der Stamp stand in der AAD, die Versiegelung geschah aber vor der Stamp-Vergabe. | Keine Stamp-AAD bei Cluster-Versiegelung (D-15, §10.4). |
| C-17 | Die Gate-Prüfung stand im Intake, das Schreiben im Committer; zwischen beiden konnte eine Lücke entstehen. | Gate-Prüfung im Committer (§11.3). |
| C-18 | Die Trait-Namen der Ports waren uneinheitlich. | Ein Kanon in §4. |
| C-19 | Das Redo-Segment war als `FileKind` geführt, obwohl es eigene Lebendigkeitsregeln hat. | Eigene Regel: lebendig ⇔ `segment_id ≥ redo_floor` (§9.2). |
| C-20 | Die Fehlerklassen unterschieden sich zwischen den Vorfassungen. | Vereinheitlicht zu F-1 bis F-16 (§5.1). |

---

## Anhang A: Goldens

### A.1 Verifiziert (unabhängiger Python-Referenzcodierer, Lauf in dieser Revision)

| Primitiv | Eingabe | Erwartet |
|---|---|---|
| CRC-32C | `"123456789"` | `0xE3069283` |
| CRC-32C | `"a"` | `0xC1D04330` |
| XXH64 | `""` | `0xEF46DB3751D8E999` |
| XXH64 | `"a"` | `0xD24EC4F1A98C6E5B` |
| XXH64 | `"abc"` | `0x44BC2CF5AD770999` |
| Bloom-Positionen | Schlüssel `"k"`, `nbits = 64`, `k = 7` | `[35, 60, 21, 46, 7, 32, 57]` |
| Bloom-Hash | `XXH64("k", 0)` | `0xC3D31922C50B1B63` |
| Bloom-Hash 2 | `XXH64("k", 0x9E3779B97F4A7C15) \| 1` | `0x6CBC77349206F859` |

### A.2 Nicht verifiziert

- **FORMAT-Kopf:** Der Referenzlauf ergab 47 B vor `chk`. Die Spezifikation in §10.2 ergibt 49 B. Ein Golden ist erst gültig, wenn das Layout in beiden Quellen übereinstimmt. Bis dahin gilt §10.2.
- **Redo-Frame und Seal:** Der Referenzlauf verwendet ein früheres Layout. Die Werte gelten nicht für R7. Neu zu erzeugen vor G-F.
- **Siegel-Flag:** Der Referenzlauf setzt das Flag als `0x80`. Die Spezifikation verlangt Bit 7 (C-10). Diese Abweichung ist im Lauf vermerkt und wird vor G-F bereinigt.

### A.3 Regeln

- Ein Golden wird nur übernommen, wenn der Referenzcodierer und der Kern übereinstimmen (§19.4).
- Die Tests lesen die Goldens aus `tests/golden_anhang_a.rs` (§19.3).

---

## Anhang B: Herkunft

| Quelle | Übernommen | Verworfen |
|---|---|---|
| `docs/archive/SPEC-R2.md` | Domänenschlüssel, Fail-closed-FsCaps, `KeyProvider.make_durable`, ENOSPC-Abschneiden, Guard-Vorwärmung, `SecretAlloc`, Consumer, Segmente, Goldens | 12-Byte-Nonces (ersetzt durch 24 B, SD-14) |
| `docs/archive/SPEC-R6-Cluster-Entwurf.md` | Cluster als Löschdomäne, Pure/Packed, Gate im Committer, Anker, Ledger, ER-Stufen, Packgruppen | Widersprüche aus §25 |
| Konsolidierung R7 | Entscheidungen D- und SD-Register, Widerspruchsregister, Hypothesen | — |

Die beiden Vorfassungen gelten nicht mehr als Quelle. Sie bleiben im Ordner `docs/archive/` zur Nachvollziehbarkeit erhalten.

---

## Anhang C: Nicht übernommen

| Thema | Grund |
|---|---|
| 12-Byte-Nonces | Zählerrisiko bei Absturz und Kopie; ersetzt durch 24-Byte-Zufallswerte (SD-14). |
| Schlüsselableitung von Clusterschlüsseln aus der Wurzel | Verhindert kryptografische Löschung eines einzelnen Clusters (C-04). |
| `domain_retired` als aktiver Tag | Nicht gelesen; nur für Import (C-06). |
| Mischung dauerhafter und flüchtiger Ziele | Verletzt INV-COMMIT-2 (C-07). |
| Blob-Dedup in Version 1 | Verhindert exakte Lebendbilanz (INV-BLOB-2); Inhaltsadressierung ist eine obere Schicht (O-5). |
| Kompression in Version 1 | Feld ist reserviert; Codec nicht spezifiziert (NZ-10, O-9). |

---

## Anhang D: Leitplanken

- Die Spezifikation ist vor dem Code zu ändern, nie danach.
- Jede Zahl ohne Messung ist eine Hypothese (§21).
- Keine Aussage über Löschung ohne Stufe aus §15.2 und ohne Prüfschritt aus §15.11.
- Kein Fehler wird verschluckt. Jeder Fehler hat eine Klasse aus §5.1.
- Jede Änderung am Format ist vor G-F frei. Danach ist sie eine Migration (§1.5).
- Ein Widerspruch wird nie durch Weglassen gelöst, sondern im Register (§25) mit Entscheidung festgehalten.

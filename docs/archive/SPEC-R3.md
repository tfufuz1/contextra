# SPEC.md — `contextra-storage` (Fundament-Crate), Revision R3

| | |
|---|---|
| Status | **Entwurf zur Freigabe (R3)**. Ersetzt SPEC R2 vollständig. Normativ wird R3 erst mit Gate **G-0** (§20): zwei Reviewer, offene Punkte aus §22 entschieden. |
| Gegenstand | Eingebetteter, synchroner, crash-sicherer, versionierter Schlüssel-Wert-Kern mit **löschgerechter physischer Anordnung** der Daten. |
| Sprache | Prosa und Rustdoc deutsch, Bezeichner englisch. |
| Normsprache | **MUSS / DARF NICHT** zwingend. **SOLL** Regelfall, Abweichung braucht Begründung. **KANN** optional. |
| Verifikation | Entwurf durch Lesen von `docs/`, `CLAUDE.md`, `ARCHITECTURE.md` und Teilen von SPEC R2 entstanden. **Nichts kompiliert, nichts gemessen.** Alle Zahlen sind Hypothesen (`H-*`, §18) bis zu einem Messbericht. |
| Rechtlicher Vorbehalt | Technische Eigenschaften werden benannt. Rechtsfolgen (Art. 17 DSGVO, Aufbewahrung, Beweislast) sind **keine** Aussage dieser Spezifikation und brauchen juristische Prüfung (§22). |

---

## 0. Was sich gegenüber R2 und dem Refaktorplan ändert

Soweit R2, Refaktorplan v2 und Review gelesen wurden, ist der Kern dort richtig: Catalog als einzige Wahrheit, Redo je Änderung, `Fs`-Port mit Stromausfall-Simulation, Roll-forward bei Löschung, Wertetrennung. Das bleibt. **Fehler im Fundament** liegt nach meiner Lesart an drei Stellen, die R3 ändert:

| # | Schwäche in R2/Plan | Folge | Änderung in R3 |
|---|---|---|---|
| C1 | Löschung ist ein **Umbauvorgang** über Runs und Blobs (E0–E7, Kosten O(Bytes der berührten Dateien)). Daten eines Betroffenen liegen verstreut. | Jede Löschung kostet eine Compaction-ähnliche Last und ist schwer zu beweisen. | **Löschgerechtes Layout:** Schlüsselordnung nach Löschscope, Dateien nach **Kohorten** getrennt (§3). Ablauf einer Aufbewahrung = Datei-Drop. Einzelfall = Schlüsselvernichtung + gebündelter, begrenzter Purge. |
| C2 | Nur **eine** Schlüsselebene (`KeyDomain`). Mehrbetroffenen-Einträge und abgeleitete Daten ungelöst (O-2). | Formatbruch nach Freeze nötig, sobald abgeleitete Daten oder Mehrfachbezug dazukommen. | **Scope-Mengen** je Eintrag, Hüllschlüssel je Scope, **Lineage-Bindung** und **Segment-Register mit Löschpflicht** sind Teil von Format v1 (§5, §7, §19). |
| C3 | Nebenläufigkeit durch Sperrränge und `loom` beschrieben, aber Protokolle für Löschung, Kohorten und Checkpoints nicht als Modell geprüft. | Races an den Nahtstellen (Erase × Compaction × Checkpoint × Schlüssel-Cache). | **Race-Freiheit durch Konstruktion** (§14): ein Schreiber, unveränderliche Versionen, Claim-Tabelle, Generationszähler für Schlüssel, formale Modelle als Gate. |

Ebenfalls geändert: Redo-Nutzlasten sensibler Spaces werden **je Scope versiegelt** (Redo wird damit durch Schlüsselvernichtung sofort unlesbar), Checkpoints bekommen ein **getrenntes Lösch-Register** (Backup-Problem), Header sind **ohne Körper lesbar** und tragen Gewicht und Klasse als generische Felder.

Nicht Teil des Kerns bleiben: Tokenizer, Indexstrukturen, Modellaufrufe, Mandantenverwaltung, Netzwerk, Rechtepolitik, Saga-Orchestrierung, Berichte. Der Kern bietet dafür **Verträge** (§19), keine Funktionen.

---

## 1. Zweck und Leitsätze

`contextra-storage` ist das Speicherfundament für Anwendungen mit **großen, überwiegend unveränderlichen Inhalten, vielen kleinen Metadaten, abgeleiteten Daten und strengen Löschpflichten**: Gedächtnis- und Abrufschichten für Sprachmodelle sind der Zielfall.

| Nr. | Leitsatz | Folge |
|---|---|---|
| L1 | **Löschbarkeit ist Layout, nicht Prozedur.** Was gemeinsam gelöscht wird, liegt gemeinsam. | §3 |
| L2 | **Ein Schreiber, viele Leser, unveränderliche Dateien.** Nebenläufigkeit entsteht nur an wenigen, modellierten Nahtstellen. | §14 |
| L3 | **Der Catalog ist die einzige Wahrheit.** Eine Datei existiert logisch erst nach ihrem Catalog-Edit und endet logisch mit ihm. | §7.6, §11 |
| L4 | **Schlüssel vor Datei, Gate vor Schlüssel.** Reihenfolge ist Vertrag und Invariante. | §12 |
| L5 | **Ehrliche Garantien.** Jede Löschstufe nennt, wogegen sie nicht schützt. | §12.2 |
| L6 | **Kein stilles Fälschen.** Gelöscht heißt `Erased`, nie „leer“ und nie Altdaten. | §12.6 |
| L7 | **Kein `unsafe` im Kern, keine Laufzeit-Abhängigkeiten ohne Entscheidung.** Plattformnahes liegt hinter Ports. | §4 |
| L8 | **Leistung wird gemessen oder behauptet nicht.** Jede Zahl ist eine Hypothese mit Messverfahren. | §18 |
| L9 | **Erweiterbar ohne Formatbruch.** Fremde Strukturen hängen sich über feste Verträge ein, der Kern kennt ihre Bedeutung nicht. | §19 |
| L10 | **Fehler sind Werte.** Kein `panic` im Produktionspfad, jede Fehlerklasse hat ein definiertes Verhalten. | §15 |

---

## 2. Ziele und Nicht-Ziele

**Ziele**

- **Z-1 Atomarität:** Ein `Batch` über beliebig viele Spaces wird ganz oder gar nicht sichtbar und durabel.
- **Z-2 Durability:** Nach Rückkehr von `apply(Durable)` überlebt der Batch Absturz und Stromausfall (mit den Grenzen des `Fs`-Ports, §15).
- **Z-3 Snapshot-Isolation:** Views sehen einen festen Stamp. Lesen blockiert nie Schreiben.
- **Z-4 Löschung nach Scope** in Zeit unabhängig von der Gesamtgröße des Stores (siehe §12.3 für Kosten).
- **Z-5 Ablauf von Aufbewahrung** als O(1)-Dateioperation je Kohorte.
- **Z-6 Nachweisbare Löschung:** `ErasureReceipt` plus unabhängiger Prüflauf (`verify_erasure`, Orakel).
- **Z-7 Header ohne Körper:** Rangfolge, Filter und Budgetscans lesen keine Körper.
- **Z-8 Race-Freiheit** nach den Methoden in §14 und §17.
- **Z-9 Ressourcenschonung:** begrenzter Speicher, begrenzte Dateihandles, begrenzte Wartungslast (Governor, §10.6).
- **Z-10 Stabile Verträge** für aufsetzende Schichten (§19).

**Nicht-Ziele**

- NZ-1 Verteilung, Replikation, Netzwerkprotokolle.
- NZ-2 Eigene Kryptografie. Es werden geprüfte Primitive verwendet.
- NZ-3 SQL, Schemas, Sekundärindizes im Kern.
- NZ-4 Rechtsbewertung (welcher Scope, welche Frist, welche Ausnahme).
- NZ-5 Garantien gegen Speicherabbilder, Swap und Core-Dateien (best-effort, §12.8).
- NZ-6 Nachträgliche Umordnung durch den Kern (Re-Layout läuft über Indirektion der oberen Schicht).

---

## 3. Das Löschkonzept in einem Bild

Dieses Kapitel ist die Grundlage der übrigen Kapitel.

### 3.1 Drei Begriffe

- **Scope (`ScopeId`)** = die **Einheit der Löschung** (zum Beispiel „Betroffener × Zweck“ oder „Vorgang“). 128 Bit, zufällig oder per Schlüssel-HMAC erzeugt, **nie** aus einer Kennung ableitbar (INV-SCOPE-1). Jeder Eintrag in einem `scoped`-Space gehört zu einer **Scope-Menge** von 1 bis 4 Scopes (Normalfall 1).
- **Kohorte (`CohortId`)** = die **Einheit des physischen Verwerfens**: Gruppe von Scopes mit gleichem Mandanten-Tag, gleicher Aufbewahrungsklasse und gleichem Ablaufzeitraum (Monat/Quartal/Jahr, konfigurierbar). Eine Kohorte hat **eigene Dateien** (Runs, Blobs, Segmente) in einem eigenen Verzeichnis.
- **Schlüsselordnung:** Im `scoped`-Space beginnt der interne Schlüssel mit `scope_id ‖ user_key`. Alle Einträge eines Scopes liegen in jeder Datei **zusammenhängend** (ein Bereich), Zugriff und Löschung sind Bereichsoperationen.

### 3.2 Zwei Löschwege

```
Ablauf der Aufbewahrung  → Kohorte gesamt verwerfen:  ein Catalog-Edit, dann unlink (O(1) je Datei)
Einzelner Scope (Art. 17)→ (1) Gate:  Lesesperre durabel                        sofort, µs–ms
                           (2) Schlüssel vernichten (Körper unlesbar, auch in Redo, Blob, Backups)  sofort
                           (3) Purge:  Bereich des Scopes aus den Dateien der Kohorte entfernen
                                       (gebündelt, mit Frist, nur Kohorten-Dateien, nie der ganze Space)
```

Wichtig: Nach Schritt (2) sind **Körper** unwiederbringlich unlesbar. Schlüssel (`user_key`) und Header liegen bis zum Purge noch unter dem Space-Schlüssel. Deshalb gilt die Regel **P-K: Schlüssel enthalten keine direkten Kennungen** (§5.2), und der Purge hat eine **harte Frist** (`purge_deadline`, Standard 72 h, §12.4). Der Purge ist Hygiene und Platzfreigabe, nicht die Vertraulichkeitsgrundlage.

### 3.3 Warum Kohorten und nicht „eine Datei je Scope“

Ein System mit Millionen Scopes kann keine Datei je Scope halten (Handles, Verzeichniseinträge, Schreibverstärkung). Kohorten begrenzen die Dateizahl (`max_cohorts_per_space`, Standard 4096) und machen **beide** Löschwege billig:

| Löschfall | Kosten | Begründung |
|---|---|---|
| Aufbewahrung endet (ganze Kohorte) | O(Dateien der Kohorte) `unlink` | Kohorte ist nach Ablaufzeitraum gebildet |
| Scope vorzeitig | Gate + Schlüsselvernichtung O(1); Purge O(Bytes der Kohorte-Dateien mit Treffer), **gebündelt** | Scopes sind zusammenhängend; Per-Run-Scope-Filter im Catalog sagen, welche Runs betroffen sind (§7.3) |
| Mandant verlassen | alle Kohorten des Mandanten-Tags: Drop | Mandanten-Tag Teil der Kohorte |
| Abgeleitete Daten eines Scopes | gleiche Kohorte oder Segmente mit Scope-Bindung | §19.2 |

### 3.4 Was **nicht** gemischt werden darf (Regeln)

- **R-COH-1** Compaction läuft **nur innerhalb einer Kohorte**. Ein Run enthält nie Scopes aus zwei Kohorten (`INV-COH-1`).
- **R-COH-2** Ein Scope gehört zu genau einer Kohorte. Wechsel (`rebucket`, etwa bei verlängerter Aufbewahrung) ist ein atomarer Umzug (§12.7) und selten.
- **R-COH-3** Alle Versionen eines Schlüssels liegen in derselben Kohorte (folgt aus R-COH-2 und der Schlüsselordnung).
- **R-COH-4** Redo-Segmente mischen Scopes (Schreibpfad). Sie sind **kurzlebig** und ihre sensible Nutzlast **je Scope versiegelt** (§13.4).

---

## 4. Architektur

### 4.1 Crates und Schichten

```
contextra-storage-ports   Traits und Werttypen, keine Logik, keine Laufzeit-Abhängigkeiten
        ▲
contextra-storage         Kern, nur std (+ ports + wenige bewilligte Abhängigkeiten, §4.5), forbid(unsafe_code)
        ▲ (nur dev-dependency)
contextra-crypto-ref      Referenzimplementierung der Krypto-Ports (Test und Erstbetrieb)
```

Der Kern hängt von `ports` ab, nie umgekehrt. Plattformnahes (Direkt-E/A, `mlock`, Hardware-CRC) kommt später als **Port-Implementierung in eigenen Crates** und berührt den Kern nicht.

### 4.2 Modulschnitt von `contextra-storage`

Abhängigkeiten nur nach unten (Test `layering`):

```
api        öffentliche Facade (Storage, Space, View, Batch, Hold, Consumer, Eraser, Checkpointer, Inspector)
engine     committer · memtable · version · views · registry · retention · claims
maint      flush · compaction · blob_gc · purge · cohort_drop · scheduler (Governor)
erase      gate · scope_table · protocol · ledger · oracle(testing)
recovery   open-Sequenz, Absturzmatrix
catalog    edits · log · rollover · current · erase_register
store      run · blob · segment · redo     (Dateiarten)
keys       scope_keys · key_cache · wrap · generations
codec      varint · tags · nonce · checksums
env        Fs · Clock · Entropy · Governor · Metrics   (Ports, Implementierung MemFs/RealFs/FaultFs)
sync       Sperren mit Rang, Epoche
```

`std::fs` nur in `env/real_fs.rs`.

### 4.3 Ports (stabil, semver, in `contextra-storage-ports`)

| Port | Aufgabe | Vertrag (Kern) |
|---|---|---|
| `Fs` | Dateien, Verzeichnisse, `sync_data`, `sync_all`, `sync_dir`, `preallocate`, `rename`, `link` (optional), `read_at`, `append`, Dateilisten | Fehlerklassen nach §15, `sync_*` nach POSIX-Semantik, keine globale Zustandsannahme |
| `Clock` | monotone Zeit, Wanduhr | injizierbar (Simulation) |
| `Entropy` | kryptografischer Zufall | injizierbar |
| `KeyProvider` | KEK, Scope-Schlüssel, `destroy`, `probe` | §13.2 |
| `Checksum` | CRC32C | Standard sicher in Rust (Slice-by-16), Hardware-Variante austauschbar |
| `Cipher` / `Mac` / `Kdf` | AEAD, Kettenprüfung, Ableitung | §13.1 |
| `Metrics` | Zähler, Histogramme, Gauges | nicht blockierend |
| `ColdTier` (Option) | Auslagerung kalter Dateien | Garantien des Ports gelten, §12.8 |
| `Observer` (Option) | Ereignisse (Flush, Purge, Poison) | nicht blockierend, nie unter Sperren aufgerufen |

### 4.4 Threading-Modell

- **Ein Committer** (Rang 1) serialisiert alle Schreibungen (Flat-Combining, Gruppen-Fsync). Kein Schreiber hält Sperren über Wartung.
- **Lese-Pfad:** Lock-frei bis auf kurze `Arc`-Klone: ein `View` hält `Arc<Version>`.
- **Wartung:** `lanes` (Flush, Compaction, Blob-GC, Purge, Kohorten-Drop) mit Prioritäten. Es gibt **zwei** unabhängige Wartungsjobs (Flush darf nie hinter Compaction warten), `urgent` für Löschung (§10.1).
- **Alle Threads gehören dem Aufrufer** oder einem vom Aufrufer übergebenen `Spawner` (kein globaler Pool im Kern).

### 4.5 Bewilligte Abhängigkeiten (Entscheidung SD-3)

| Zweck | Wahl | Begründung / Alternative |
|---|---|---|
| Speicherresidente Sortierstruktur | `crossbeam-skiplist` hinter dem Trait `Memtable` | lock-freie Lesevorgänge ohne eigenes `unsafe`; Alternative: eigener Baum hinter dem gleichen Trait |
| Zero-Copy-Puffer | `bytes` | gängig, auditiert |
| Hash/MAC | `blake3` | schnell, bereits in `crypto-ref` |
| AEAD | `chacha20poly1305` (XChaCha) | bereits in `crypto-ref` |
| Zeroize | `zeroize` | Schlüsselmaterial |
| Test | `loom`, `proptest`, `shuttle` (dev) | §17 |

Jede weitere Abhängigkeit braucht einen Eintrag im Entscheidungsregister und eine Prüfung (`cargo-deny` mit `[advisories]`, `[bans]`, `[sources]`).

---

## 5. Datenmodell

### 5.1 Begriffe

| Begriff | Bedeutung |
|---|---|
| `Space` | benannter, unabhängiger Schlüsselraum mit **unveränderlichen** `SpaceOptions` (nach dem Anlegen) |
| `Stamp` | `u64`, streng monoton, vergibt der Committer; Ordnung aller Änderungen |
| `Entry` | Schlüssel, Version (Stamp), Header, Körper |
| `Header` | ohne Körper lesbarer Teil (§5.3) |
| `Body` | Nutzlast, inline (klein) oder `BlobRef` (groß) |
| `View` | unveränderlicher Lesezustand bei Stamp `V` |
| `Hold` | Untergrenze: Versionen ≥ Hold-Stamp bleiben lesbar (Zeitreise, Index-Aufholen) |
| `Consumer` | durabler Cursor mit Watermark; zugleich Hold |
| `Guard` | Vorbedingung innerhalb eines `Batch` (Compare-and-Set auf Version oder Abwesenheit) |
| `Scope`, `Cohort` | §3.1 |
| `Segment` | vom Besitzer verwaltete Datei, vom Catalog registriert (§19.2) |

### 5.2 Schlüssel und Ordnung

- `user_key`: Bytes, 1…`max_key_len` (Standard 1024), lexikografisch geordnet.
- **Interner Schlüssel** im `scoped`-Space: `scope_id[16] ‖ user_key`. Im `plain`-Space nur `user_key`.
- **Regel P-K:** `user_key` DARF NICHT Kennungen natürlicher Personen enthalten (Name, E-Mail, Ausweis, Konto). Der Kern kann das nicht prüfen. `SpaceOptions::key_policy = Opaque` verlangt, dass Schlüssel Ausgaben einer vom Aufrufer gelieferten Funktion sind (nur dokumentierend); `Inspector::lint_keys` sucht in Tests Muster. Verstöße sind Aufrufer-Fehler und verlängern die Restexposition bis zum Purge (§12.2).
- Die **Version** ist `(Stamp, Op-Index)`; neuere zuerst bei gleichem Schlüssel.

### 5.3 Header (ohne Körper lesbar)

Fester Kopf, dann variabler Teil. Bytes in §7.4.

| Feld | Typ | Zweck |
|---|---|---|
| `kind` | u8 | Put · Tombstone · RangeTombstone · ErasedMarker · Merge (reserviert) |
| `flags` | u8 | `has_body`, `body_inline`, `scope_multi`, `sealed`, `derived` |
| `class` | u8 | Klassenbyte für die obere Schicht (zum Beispiel Vertrauensstufe). Der Kern interpretiert es nie. Er stellt nur `scan_class_mask` bereit. |
| `weight` | u32 | allgemeines **Kostengewicht** je Eintrag (obere Schicht legt fest: Token, Bytes, …). Der Kern summiert pro Block. |
| `body_len` | u32 | Klartextlänge |
| `body_ref` | Inline oder `BlobRef{blob_no, offset, len, gen}` | |
| `scopes` | 1–4 × `scope_id` (+ Schlüssel-ID je Scope) | Scope-Menge |
| `meta` | ≤ `max_meta` (Standard 64) Bytes | opake Metadaten der oberen Schicht (Herkunft, Verweise, Zeiten) |
| `lineage` | 0–8 × `EntryRef{space,key_hash,stamp}` | optional: Quellen eines abgeleiteten Eintrags (§19.3) |
| `body_crc` | u32 | Prüfsumme des (versiegelten) Körpers |

**Warum so:** `weight` und `class` im Header erlauben **Budgetscans** (`scan_budget`) und Klassenfilter ohne Körperzugriff. `meta` ersetzt viele kleine Zusatz-Spaces. Die Aggregate stehen im Block-Index (§7.3).

### 5.4 `SpaceOptions` (unveränderlich nach dem Anlegen)

```rust
pub struct SpaceOptions {
    pub placement: Placement,            // Plain | Scoped
    pub cohorts: CohortPolicy,           // Single | Bucketed { tag: TagMode, bucket: Bucket, shards: u8 }
    pub durability_default: Durability,
    pub integrity: IntegrityMode,        // Plain | Authenticated | Encrypted  (steht in FORMAT, §13.1)
    pub body_seal: BodySeal,             // None | PerScope   (PerScope nur bei Placement::Scoped)
    pub multi_scope_rule: MultiScopeRule,// AnyDestroyed (Standard) | AllDestroyed
    pub keep_header_after_erase: bool,   // false: Purge entfernt Header (Standard) | true: nur Marker + Provenienz
    pub history: HistoryPolicy,          // KeepLatest | KeepUntilFloor | Append-only (Erase nur mit Admin)
    pub value_separation_min: u32,       // Körper ≥ Schwelle → Blob (Standard 1 KiB)
    pub max_key_len: u32, pub max_meta: u8,
    pub compression: Compression,        // None | Lz4 | Zstd(level) – Port, Feature
}
```

### 5.5 Scope-Tabelle (Catalog)

Je Scope: `scope_id → { cohort_id, state, key_epoch, created_stamp, due_bucket, tenant_tag, retention_class }`.

`state`: `Live → Gated → Shredded → Purged` (nur vorwärts; `Restricted` ist Sache der oberen Schicht und erscheint im Kern als Gate-Grund, §12.5).

---

## 6. Öffentliche API (normative Signaturen, Nebensächlichkeiten wie Lifetimes offen)

```rust
// --- Öffnen und Struktur --------------------------------------------------
pub struct Storage { /* Arc<Inner> */ }
impl Storage {
    pub fn open(cfg: Config, env: Env) -> Result<Storage, OpenError>;     // Recovery inklusive
    pub fn space(&self, name: &str) -> Result<Space, Error>;
    pub fn spaces_add(&self, name: &str, opts: SpaceOptions) -> Result<Space, Error>; // atomar, Catalog
    pub fn close(self) -> Result<CloseReport, Error>;                      // drain, Flush optional
}

// --- Schreiben: ein Batch über viele Spaces, atomar -------------------------
pub struct Batch { /* Ops + Guards, kein Zustand außerhalb */ }
impl Batch {
    pub fn new() -> Batch;
    pub fn put(&mut self, sp: &Space, key: Key, e: NewEntry) -> &mut Self;
    pub fn delete(&mut self, sp: &Space, key: Key) -> &mut Self;
    pub fn delete_range(&mut self, sp: &Space, r: KeyRange) -> &mut Self;
    pub fn guard(&mut self, sp: &Space, key: Key, g: Precondition) -> &mut Self; // Absent | Version(Stamp) | Unchanged
    pub fn idempotency(&mut self, token: IdemToken) -> &mut Self;                // Wiederholung → AlreadyApplied
}
pub struct NewEntry { pub scopes: ScopeSet, pub class: u8, pub weight: u32,
                      pub meta: SmallBytes, pub lineage: Lineage, pub body: Body }
impl Storage {
    pub fn apply(&self, b: Batch, d: Durability) -> Result<Applied, ApplyError>;
    // Applied { stamp, bytes_redo }   ApplyError::{GuardFailed{..}, AlreadyApplied{stamp}, Scope(ScopeError), Backpressure(Retry), Poisoned, ..}
}

// --- Lesen -------------------------------------------------------------------
impl Storage { pub fn view(&self) -> View;                        // aktueller Stamp
               pub fn view_at(&self, s: Stamp) -> Result<View, Error>; } // ≥ Untergrenze, sonst Erased/TooOld
impl View {
    pub fn stamp(&self) -> Stamp;
    pub fn get_header(&self, sp: &Space, key: &[u8]) -> Result<Option<HeaderRef>, ReadError>;
    pub fn get(&self, sp: &Space, key: &[u8]) -> Result<Option<EntryRef>, ReadError>; // Körper lazy
    pub fn scan(&self, sp: &Space, r: KeyRange, o: ScanOpts) -> Scan;      // Iterator, Header zuerst
    pub fn scan_budget(&self, sp: &Space, r: KeyRange, max_weight: u64, o: ScanOpts) -> BudgetScan; // nutzt Block-Aggregate
    pub fn history(&self, sp: &Space, key: &[u8]) -> History;
}
impl EntryRef { pub fn header(&self) -> &Header;
                pub fn body(&self) -> Result<Body, BodyError>; }           // Err(Shredded{scope}) nie Daten, nie leer

// --- Holds und Consumer (Voraussetzung für abgeleitete Indizes) -------------
impl Storage { pub fn hold(&self, s: Stamp, why: HoldReason) -> Result<Hold, Error>;
               pub fn consumer(&self, id: ConsumerId, from: Stamp) -> Result<Consumer, Error>; }
impl Consumer { pub fn next_batch(&mut self, max: usize) -> Result<Changes, Error>; // Änderungen in Stamp-Reihenfolge
                pub fn ack(&mut self, upto: Stamp) -> Result<(), Error>; }          // durabel, hebt Watermark

// --- Scopes und Löschung -----------------------------------------------------
impl Storage { pub fn scopes(&self) -> Scopes; pub fn eraser(&self) -> Eraser; }
impl Scopes   { pub fn register(&self, spec: ScopeSpec) -> Result<ScopeId, Error>;   // Tenant-Tag, Klasse, Ablauf
                pub fn rebucket(&self, s: ScopeId, new_due: Due) -> Result<RebucketReport, Error>; }
impl Eraser {
    pub fn scope(&self, s: ScopeId, req: EraseReq) -> Result<ErasureTicket, EraseError>;   // Gate sofort (durabel)
    pub fn scopes_batch(&self, v: &[ScopeId], req: EraseReq) -> Result<ErasureTicket, EraseError>;
    pub fn cohort_expired(&self, c: CohortId) -> Result<ErasureTicket, EraseError>;
    pub fn range(&self, sp: &Space, r: KeyRange, req: EraseReq) -> Result<ErasureTicket, EraseError>; // Bereichslöschung
    pub fn status(&self, t: &ErasureTicket) -> EraseStatus;      // Gated · KeyDestroyed · Purging · Done(ErasureReceipt)
    pub fn wait(&self, t: &ErasureTicket, until: EraseStage, timeout: Duration) -> Result<EraseStatus, Error>;
    pub fn verify(&self, t: &ErasureTicket) -> VerifyReport;     // Catalog-Metadaten + Stichproben (INV-ERA-2/6)
}
pub struct EraseReq { pub level: EraseLevel,  // Logical | Physical | Scrubbed | Cryptographic
                      pub holds: HoldPolicy, pub views: ViewPolicy, pub purge_deadline: Option<Duration> }

// --- Checkpoints, Betrieb, Prüfung --------------------------------------------
impl Storage { pub fn checkpoint(&self, dst: &Path, m: CheckpointMode) -> Result<CheckpointManifest, Error>; // Copy | Reflink | Hardlink(Opt-in)
               pub fn restore_check(&self, m: &CheckpointManifest) -> RestorePlan;  // wendet Lösch-Register an (§12.9)
               pub fn maintenance(&self) -> Maintenance;   // Pause/Resume, Zustände, Governor
               pub fn inspect(&self) -> Inspector; }       // Dateien, Cohorts, Scope-Statistik, lint_keys, Kennzahlen
```

**Eigenschaften der API**

- Keine Methode blockiert **unbegrenzt**: jede Wartefunktion hat Timeout; Rückdruck ist `Backpressure(Retry)` mit Hinweis.
- Fehler sind aufzählbar (`Error::class()`), jeder trägt `hint()` mit Handlungstext (hilfreich für Aufrufer, die ein Modell ist).
- `apply` ist **idempotent über `IdemToken`** (Wiederholung nach Timeout ohne Doppelwirkung). Das Token wird im selben Redo-Frame abgelegt.
- Ein `View` ist `Send + Sync`, `Clone` billig, hält keine Sperre.
- `EntryRef::body()` liefert `Err(Shredded)` nach Schlüsselvernichtung und `Err(Erased)` nach Purge. Beide sind unterscheidbar.

---

## 7. Plattenformate (Format v1, Freeze nach Gate G-F)

**Konventionen:** Feste Felder **Little-Endian**, Sortierschlüssel **Big-Endian-orientiert** (lexikografisch). Alle Längenfelder werden vor Allokation gegen Plausibilitätsgrenzen geprüft (P-4). Prüfsumme CRC32C (Zufallsfehler), im Modus `Authenticated`/`Encrypted` zusätzlich Tag. Jede Datei beginnt mit `magic[8] ‖ file_kind u8 ‖ format_ver u16 ‖ file_id u64 ‖ header_crc`.

### 7.1 Verzeichnis-Layout (Löschgrenze = Verzeichnis)

```
<root>/
  FORMAT                         Format, Integritätsmodus, Parameter (unveränderlich), Kette
  KEYS                           umhüllte Space-DEKs (atomar ersetzt)
  CATALOG/ 000042.cat, CURRENT   Edit-Log + Verweis
  ERASE-LEDGER/ ledger-0007.led  append-only, kettenverknüpft (§12.9), bewusst getrennt vom Catalog
  redo/ 00000123.redo
  spaces/<space>/
     cohorts/<cohort>/{ runs/, blobs/, segments/ }       ← Kohorte = Löschgrenze
     plain/{ runs/, blobs/, segments/ }                  ← Spaces ohne Scope
  graveyard/                     wartet auf Rückgabe aller Referenzen
  tmp/ *.building                nie lebendig, wird beim Öffnen bereinigt
```

### 7.2 Redo-Segment und Frame

Segment: `header ‖ Frame*`. **Frame** je `apply`:

```
frame_head (Klartext, Ketten-MAC-geschützt):  len u32 | stamp u64 | n_ops u16 | idem_token[16] | flags
                                              ops_head[]: { space_id u16, kind u8, scope_id[16] (oder 0), len u32 }
ops_payload[] : Schlüssel, Header, Körper je Op
                - Plain-Space: unter Redo-Schlüssel (Modus Encrypted) bzw. Klartext (Plain)
                - Scoped + BodySeal::PerScope: GESAMTE Op-Nutzlast mit Scope-Schlüssel versiegelt (§13.4)
frame_tail:    crc32c | chain_mac (BLAKE3 keyed, Kette über die Frames: H_i = MAC(H_{i-1} ‖ frame_head ‖ payload_tags))
```

Folgen: Kettenprüfung entdeckt Abschneiden und Einfügen. Stamp und Scope-ID stehen im Klartext, damit **Recovery und Flush-Barriere ohne Schlüssel planen** können. Nach Vernichtung eines Scope-Schlüssels bleibt die Op-Nutzlast **unlesbar**, das Replay erzeugt für diese Op einen `ErasedMarker` im Speicher.

### 7.3 Run (sortierte unveränderliche Tabelle)

```
Run := header
       DataBlock*        (Zielgröße 16 KiB, Präfix-Kompression mit Restart-Punkten, optional Kompression, dann AEAD)
       IndexBlock        (je Datenblock: letzter Schlüssel, Offset, Länge, entries, weight_sum, class_mask, min/max_stamp)
       FilterBlock       (Schlüsselfilter; Ribbon oder Bloom, Parameter im Footer)
       ScopeBlock        (Scope-Verzeichnis des Runs: sortierte Liste der Scopes, wenn ≤ 8192, sonst Ribbon-Filter + Bereichsgrenzen)
       MetaBlock         (min/max Schlüssel und Stamp, BlobRef-Bereich, Eintragszahlen, cohort_id, space_id)
       ExtensionBlock*   (Tag, Version, Länge, Daten; opak für den Kern, §19.1)
       Footer            (Offsets, Prüfsumme, Format, Kette)
```

- `weight_sum` und `class_mask` im IndexBlock ermöglichen `scan_budget` mit **Block-Überspringen** ohne Dekomprimierung.
- **Der ScopeBlock** ist der Schlüssel zu billigem Purge: Der Catalog speichert je Run den Scope-Filter (oder dessen Hash), sodass der Purge-Planer ohne Dateizugriff die betroffenen Runs wählt.
- **Jeder Run gehört genau einer Kohorte** (`INV-COH-1`).

### 7.4 Entry-Kodierung im Block

```
entry := varint(shared_prefix) varint(unshared) varint(vlen) | unshared_key
         version: stamp u64 + op_idx u16
         header_fixed (kind, flags, class, weight, body_len)
         [scopes: n u8 | (scope_id[16], key_id u32)*]
         [meta: len u8 | bytes]  [lineage: n u8 | refs]
         body_ref: inline_len | bytes   oder   blob_no u56, offset u32, len u32, gen u16
         [wraps: n u8 | (scope_idx u8, wrapped_cek[48])*]   (nur flags.scope_multi)
         body_crc u32
```

Inline-Körper eines scoped-Spaces sind **bereits versiegelt** (§13.4). Header und Schlüssel liegen unter dem Block-AEAD (Space-DEK).

### 7.5 Blob-Datei (Körper, append-only je Generation)

```
BlobFile(blob_no, gen) := header | Record* | footer
Record := len u32 | scope_set_tag u8 | nonce[24] | sealed_body | crc32c
```

- Eine Datei liegt in der **Kohorte** ihres Scopes, enthält also nur Körper dieser Kohorte (`INV-COH-1` gilt auch für Blobs). Zielgröße ≤ 64 MiB.
- **Sweep-Neufassung** (Gen g+1) übernimmt überlebende Records **byte-identisch** an **gleichen Offsets**, gelöschte Records werden zu `Hole` (Nullen). Dadurch bleiben `BlobRef`s in älteren Runs gültig. Lebendanteil < `blob_gc_live_ratio` (Standard 50 %) löst GC mit Umzug (neue `blob_no`) aus.

### 7.6 Catalog (Metadaten-Log, einzige Wahrheit)

`Edit := { seq, ops[] }`, jeder Edit atomar durch einen Append + `sync_data`. Ops (Auswahl):

`spaces_add | spaces_drop | cohort_add | cohort_drop | scope_register | scope_state(scope, state) | files_add(kind, file_id, cohort, meta) | files_remove | blob_replace | blob_live_delta | redo_floor | segment_add | segment_remove | hold_set | consumer_ack | erase_pending_add(ticket) | erase_done(ticket) | erased_ranges_add | key_epoch(scope, epoch) | idem_trim`

- Rollover bei Größe, `CURRENT` per atomarem `rename`. Ein Rollover-Edit enthält den vollständigen Zustand (Snapshot) und die Kettenprüfsumme.
- **Regel (L3):** Eine Datei ist lebendig genau dann, wenn ein durabler `files_add` existiert und kein `files_remove`. Dateien ohne Eintrag sind verwaist und werden beim Öffnen entfernt.

### 7.7 Freeze-Liste (Gate G-F)

Eingefroren werden: Magic/Datei-Header, Frame-Kopf und Kette, Entry-Kodierung inkl. Scope-Menge, Wrap-Liste und `meta`/`lineage`-Feldern, IndexBlock-Felder (`weight_sum`, `class_mask`), ScopeBlock, ExtensionBlock-Rahmen, BlobRecord, Catalog-Op-Tags (mit **reserviertem Bereich** für künftige Tags, unbekannte Tags ⇒ `Err(FormatTooNew)` statt Ignorieren), Nonce-Regeln, Ledger-Format. Zu jedem Format gibt es einen **Referenzcodierer** (Python, unabhängig) und Goldene Vektoren. Nach G-F ist jede Änderung eine Migration (`format_ver`).

---

## 8. Schreibpfad

### 8.1 Ablauf von `apply`

```
1  Vorprüfung (ohne Sperre): Größen, Space/Scope bekannt, Rechtetyp, Idempotenz-Cache (Lesen)
2  Eintragen in den Intake (Rang 2) → Warten auf Gruppenführer
3  Gruppenführer = Committer (Rang 1), nimmt alle wartenden Batches:
     a) Guards prüfen gegen den aktuellen Zustand (Memtable + Version) → fehlgeschlagene Batches ausscheiden
     b) Scope-Gate prüfen (Gated/Shredded/Purged ⇒ ScopeError, kein Schreiben)
     c) Stamps vergeben (lückenlos je Gruppe)
     d) Körper versiegeln (PerScope) — außerhalb der Sperre vorbereitet, hier nur Nonce/Position
     e) Ein Redo-Frame je Batch, ein `write` + ein `sync_data` je Gruppe  ◄── DURABILITÄTSPUNKT
4  Memtable-Einträge einfügen (Rang 4, kurz), Version veröffentlichen (Rang 3, `Arc`-Swap), `visible_stamp` anheben
5  Rückkehr an alle Wartenden der Gruppe
```

- **Sichtbarkeit (§8.3):** Ein Eintrag ist ab `visible_stamp ≥ s` sichtbar, **nachdem** er durabel ist (Lese-nach-Durabilität). Ausnahme: `Durability::Relaxed` (Opt-in je Aufruf, sichtbar vor Fsync, bei Absturz verlierbar; im Receipt nicht zulässig).
- **Rückdruck (§8.4):** Memtable-Größe, Zahl unflushter Memtables, L0-Dateien und Redo-Rückstand wirken stufenweise: *verlangsamen* (kurzes Warten außerhalb jeder Sperre), dann `Backpressure(Retry)`. Warten **unter** dem Committer-Lock ist verboten (Erratum E-4 aus R2 gilt weiter).
- **Redo-Segmente** haben Lebenszyklus `Open → Sealed → Obsolete(≤ redo_floor) → Graveyard`. Maximalalter `redo_max_age` (Standard 1 h): danach zwingt der Scheduler einen Flush, damit Klartext-Schlüssel nicht unbegrenzt im Redo liegen (§3.2).
- **ENOSPC/EIO:** Der Committer geht in **Poison** (`Err(Poisoned)` für alle Schreiber), Lesen bleibt möglich. Wiederanlauf nur durch `Storage::recover_after_fault` oder Neustart (Fsync-Fehler werden **nie** wiederholt, siehe §15).

### 8.2 Memtable

- Je Space (nicht je Kohorte) eine aktive Memtable; Einträge liegen in interner Schlüsselordnung (also nach Scope zusammenhängend).
- Memtable hält **Klartext** (Annahme A-5: Prozessspeicher ist vertrauenswürdig). Beim Flush wird nach **Kohorte** aufgeteilt, jede Kohorte erhält eigene Runs.
- Größenziel `memtable_bytes` (Standard 64 MiB), max. `max_immutable` (Standard 3).

### 8.3 Flush

Je Space: Memtable einfrieren → Teilmengen je Kohorte → je ein Run (`.building` → `sync_all` → `rename` → `sync_dir`) → **ein** Catalog-Edit (`files_add` aller Runs + `redo_floor`) → neue Version veröffentlichen. Ein Absturz vor dem Edit lässt den Run verwaist (wird entfernt), Redo spielt erneut ein.

---

## 9. Lesepfad

1. `view()` klont `Arc<Version>` (Memtables, immutable Memtables, Dateimenge je Kohorte). Keine Sperre danach.
2. **Punktabfrage:** Scope-Tabelle (Cache) → Kohorte → Memtable → immutable Memtables → je Level Runs der Kohorte, gefiltert durch Schlüsselfilter → ein Datenblock. Ziel: höchstens **ein** Datenblock-Read je Level-Treffer.
3. **Scan:** k-Wege-Merge über Memtables und Runs **der betroffenen Kohorten**; Header kommen ohne Körper-E/A. `scan_budget` summiert `weight_sum` aus dem Index und überspringt Blöcke, deren Gewicht-Obergrenze das Budget sprengt oder deren `class_mask` den Filter ausschließt.
4. **Körper** werden erst bei `body()` geladen (Blob-Read, `read_at`), geprüft (CRC, Tag) und bei `PerScope` entsiegelt. Fehlt der Schlüssel: `Err(Shredded{scope})`.
5. **Block-Cache** (sharded, Klasse-aufwärtig, `Bytes`-zählend): Pin-Zähler; kein Block wird entfernt, solange eine `EntryRef` darauf zeigt. `invalidate_file` bei Dateiverlust.
6. **Gate-Prüfung zuerst:** `scope_state` wird vor jeder Entschlüsselung geprüft (O(1), Cache mit Generationszähler, §14.5). Zustand `Gated` oder schlimmer ⇒ `Erased`, keine Daten.
7. Lesen nach `Erased`-Register: Eine View mit Stamp unter dem Löschstamp `S` in den gelöschten Bereich liefert `Err(Erased{range|scope, at: S})` (INV-ERA-5).

---

## 10. Wartung

### 10.1 Lanes und Prioritäten

| Lane | Aufgabe | Priorität |
|---|---|---|
| `urgent` | Gate-Folgearbeit, Purge mit nahender Frist, Kohorten-Drop | höchste, darf Compaction abbrechen |
| `flush` | Memtable → Runs | hoch, **nie** hinter Compaction |
| `compact` | Leveled-Compaction je Kohorte | normal |
| `blobgc` | Blob-GC nach Lebendanteil | normal |
| `idle` | Verifikation (`scrub`), Verwaisten-Suche | niedrig |

**Claim-Tabelle:** Jede Wartungsarbeit beansprucht ihre Eingabedateien exklusiv (`claims: file_id → job`). Ein Claim-Konflikt führt nie zu Warten, sondern zu Abbruch der niederen Priorität (§14.4).

### 10.2 Compaction

Leveled je Kohorte, Level-Größen nach Verhältnis `T` (Standard 10), L0 mit Größenschwelle. **Cohort-Grenze ist hart (R-COH-1).** Eingabe wird nach Stamp-Untergrenze bereinigt: Versionen unter `floor` (kleinster Hold/View/Consumer-Stamp), die von einer neueren Version verdeckt sind, werden verworfen, Tombstones verschwinden, wenn keine tiefere Ebene den Schlüssel enthält. **Subcompactions** je Schlüsselbereich dürfen parallel laufen (Bereich = Vielfaches von Scope-Grenzen, nie innerhalb eines Scopes aufgeteilt, wenn `scope_aligned` gesetzt).

### 10.3 Blob-GC und Graveyard

Lebendanteil je Blob-Generation aus `blob_live_delta` im Catalog. GC: überlebende Records in neue Datei mit **neuer** `blob_no` (neuer Nonce-Raum), Runs werden über `blob_replace` und Rewrite der betroffenen Header-Einträge **in der nächsten Compaction** umgestellt. **Graveyard:** Eine Datei wird erst gelöscht, wenn (a) der entfernende Catalog-Edit durabel ist und (b) die letzte `Arc<Version>` mit Verweis darauf gefallen ist (Referenzzähler/Epoche, INV-GRAVE-1).

### 10.4 Cohort-Drop (Ablauf der Aufbewahrung)

```
CD1 Gate-Edit: cohort_state = Retiring (durabel), neue Schreibungen in Scopes der Kohorte → ScopeError
CD2 Wartet auf Views/Holds/Consumer mit Verweis (ViewPolicy), Compaction in der Kohorte abgebrochen
CD3 Ein Edit { cohort_drop, files_remove[alle], scope_state(alle Scopes der Kohorte → Purged), erased_ranges_add }  ◄── physischer Punkt
CD4 Graveyard → unlink → sync_dir (Scrubbed: überschreiben davor) → Receipt
```

Der Scope-Schlüssel dieser Scopes wird **ebenfalls vernichtet** (`KeyProvider::destroy`, §13.2) — der Drop allein genügt nicht, wenn Kopien der Dateien in Backups existieren.

### 10.5 Extensions und Segmente

Ihre Lebenszyklen (Anbindung ohne Formatänderung) stehen in §19.

### 10.6 Governor (Ressourcenschonung)

Token-Bucket je Klasse (Schreib-E/A der Wartung, CPU-Zeit, gleichzeitige Dateihandles). Standard: Wartung darf höchstens `maint_io_fraction` (50 %) der gemessenen E/A-Kapazität nutzen, `urgent` hat eine eigene Reserve. Dateihandles: LRU-Cache mit Obergrenze `max_open_files`, kalte Kohorten sind ohne offene Handles.

---

## 11. Recovery

### 11.1 Ablauf beim Öffnen (deterministisch)

```
R1  FORMAT lesen und prüfen (Kette, Version); `FormatTooNew` ⇒ Abbruch
R2  CURRENT → Catalog lesen, Edits bis zum letzten gültigen Rahmen anwenden, Rest verwerfen (Torn Tail)
R3  Verzeichnisse scannen: Dateien ohne Catalog-Eintrag ⇒ verwaist (Liste, noch nicht löschen)
R4  Schlüssel-Port prüfen: KEK verfügbar? Sonst: Öffnen im Modus `LockedKeys` (Lesen nur Klartext-Spaces)
R5  Redo ab `redo_floor` lesen: je Frame Kette prüfen; erster ungültiger Rahmen = Ende; 
    Scoped-Nutzlast: Scope-Schlüssel fehlt oder Scope Shredded ⇒ Op als ErasedMarker einspielen (keine Daten)
R6  Memtable aufbauen, `visible_stamp` = letzter gültiger Frame
R7  Offene Löschungen = erase_pending ∪ Gate-Zustände ≠ Purged ⇒ in `urgent` einplanen und AB DER ERSTEN OFFENEN STUFE fortsetzen
R8  Verwaiste Dateien entfernen (nach R7, nie davor)
R9  Wartung starten; `Storage` geben
R10 Bericht (`OpenReport`): Frames, verworfene Bytes, offene Löschungen, verwaiste Dateien
```

### 11.2 Absturzmatrix (Soll-Zustand je Unterbrechung)

| Unterbrechung | Soll nach Neustart |
|---|---|
| vor Redo-`sync` | Batch nicht sichtbar, `apply` hat nicht zurückgegeben |
| nach Redo-`sync`, vor Rückkehr | Batch vorhanden (Idempotenz-Token verhindert Doppelung) |
| Mitten im Run-Bau | `.building` entfernt, Redo spielt ein |
| nach Run `rename`, vor Catalog-Edit | Run verwaist, entfernt |
| nach Catalog-Edit, vor Unlink | alte Dateien verwaist, entfernt |
| Gate-Edit durabel, Schlüssel noch da | Löschung wird fortgesetzt (Roll-forward) |
| Schlüssel vernichtet, Edit `scope_state=Shredded` fehlt | Probe `KeyProvider::probe` = `Absent` ⇒ Edit nachholen |
| Purge-Edit durabel, Unlink offen | Verwaisten-Suche entfernt alte Dateien |
| `Erased`-Ledger-Rahmen halb | Kette erkennt Torn Tail, Eintrag wird aus dem Catalog-Zustand neu erzeugt |

---

## 12. Löschung

### 12.1 Bedeutung der Wörter

- **Logisch atomar:** Ab Stamp `S` sieht keine View ≥ `S` Inhalte des Scopes, für alle Schlüssel zugleich (Gate ist **ein** durabler Edit).
- **Physisch atomar:** genau **ein** Catalog-Edit trennt „Dateien lebendig“ von „Dateien nicht mehr referenziert“. Davor Roll-forward, danach Aufräumen.
- **Nachweisbar:** `ErasureReceipt` erst nach Unlink **und** Verzeichnis-Sync **und** durablem Ledger-Eintrag.

### 12.2 Stufen und ehrliche Garantien

| Stufe | Verfahren | Schützt gegen | Schützt **nicht** gegen |
|---|---|---|---|
| `Logical` | Gate + `ErasedMarker` | Lesen über die API | Inhalt auf Platte |
| `Physical` | Gate, Purge der Bereiche, Edit, Unlink, `sync_dir`, Redo-Segmente abgelaufen | Lesen aus Dateien im Verzeichnis, Wiederauftauchen | Dateisystem-Journal, Flash-Reste, Snapshots/Backups außerhalb, Hardlinks |
| `Scrubbed` | wie `Physical`, vor Unlink Datei mit Nullen überschreiben + Sync | einfache Wiederherstellung auf klassischen Datenträgern | CoW-Dateisysteme, SSDs (best-effort) |
| `Cryptographic` | Gate, **Scope-Schlüssel vernichten**, dann (wie `Physical`) Purge | **Körper** in Redo, Blob, Backups, Flash | Header/Schlüssel (`user_key`) bis zum Purge; Kopien des Scope-Schlüssels |

**Konsequenz:** Für Körper liefert nur `Cryptographic` echte Unwiederbringlichkeit auf modernen Datenträgern. Für Header und `user_key` liefert nur der **Purge** (und Regel P-K) die Entfernung. Beides zusammen ist der Standardpfad (`EraseLevel::Cryptographic` + Purge).

### 12.3 Protokoll `Eraser::scope` (Stufen G → K → P → D)

```
G  Gate:      Edit { scope_state(s, Gated), erase_pending_add(ticket) } durabel (Gruppen-Fsync). 
              Ab hier: alle Lesepfade antworten Erased; apply in den Scope → ScopeError.      ◄── LOGISCHER Punkt, Stamp S
              Vorprüfung: Holds/Consumer mit Stamp < S gemäß HoldPolicy (Fail oder Override)
K  Schlüssel: KeyProvider::destroy(scope) [muss durabel sein, idempotent]; probe(scope) == Absent
              Schlüssel-Cache leeren, Generationszähler erhöhen (§14.5)
              Edit { scope_state(s, Shredded), key_epoch(s, ∞) }                              ◄── KRYPTO-Punkt
P  Purge:     gebündelt je Kohorte (§12.4):
              P1 Redo: Flush-Barriere nur für Kohorten mit unflushten Einträgen dieses Scopes; Redo-Segmente bis S obsolet
              P2 Runs wählen: ScopeBlock-/Catalog-Filter der Kohorte, Treffer-Runs
              P3 Je Run streamen, Bereich [scope_id, scope_id+1) verwerfen (Holds je Policy), neuer Run (sync_all)
              P3b Blob-Sweep: Records des Scopes → Hole, Generation g+1 mit gleichen Offsets
              P4 EIN Edit { files_add, files_remove, blob_replace, blob_live_delta, scope_state(s, Purged),
                            erased_ranges_add(range, S), erase_done(ticket) }                 ◄── PHYSISCHER Punkt
D  Danach:    Version veröffentlichen, Block-Cache invalidieren, alte Dateien ins Graveyard,
              (Scrubbed: überschreiben + Sync) → unlink → sync_dir,
              Segmente mit Scope-Bindung: `SegmentOwner::rewrite_without` abgeschlossen (§19.2),
              Ledger-Eintrag (§12.9), `ErasureReceipt` zurückgeben
```

**Reihenfolge ist Vertrag (L4):** G vor K, K vor P4, P4 vor Unlink. `destroy` vor `Shredded`-Edit, damit eine Fehlerprobe die Wahrheit liefert.

**Kosten:** G und K sind O(1) (zwei durable Edits plus Provider-Aufruf). P ist O(Bytes der Treffer-Runs und -Blobs der **Kohorte**), unabhängig von der Gesamtgröße. Durch **Bündelung** (§12.4) teilen sich viele Scopes denselben Rewrite.

### 12.4 Purge-Bündelung und Frist

- Offene Purges je Kohorte werden gesammelt. Ein Lauf entfernt **alle** wartenden Scopes dieser Kohorte in einem Durchgang.
- Auslöser: (a) `purge_deadline` des ältesten wartenden Tickets zu 70 % verbraucht, (b) Menge wartender Scopes ≥ `purge_batch_min`, (c) Kohorte hat Compaction anstehen (Mitnahme: die Compaction verwirft die Scope-Bereiche gleich mit, `erase_swept` im selben Edit).
- Frist `purge_deadline` Standard **72 h**, je Request überschreibbar. Wird sie gerissen, steigt die Priorität auf `urgent` und es wird ein Alarm ausgelöst (`Observer`). Die Frist ist **keine Rechtsaussage**.
- **Vorgezogener Pfad** für besonders sensible Klassen: `EraseReq::purge_deadline = Some(0)` erzwingt sofortigen Purge nach K.

### 12.5 Nebenläufigkeit mit Views, Holds, Consumern

| Policy | Verhalten |
|---|---|
| `ViewPolicy::Wait(timeout)` (Standard) | Unlink wartet auf Rückgabe aller Views mit Verweis. Nach Timeout `Err(Blocked{views})`, Vorgang bleibt in D offen und wird wiederholt (logische Garantie gilt bereits). |
| `ViewPolicy::Revoke` | Betroffene Views werden widerrufen (`Err(Revoked)` beim nächsten Zugriff). |
| `HoldPolicy::Fail` (Standard) | Hold/Consumer mit Stamp < `S` über dem Bereich ⇒ `Err(HoldBlocked)`. |
| `HoldPolicy::Override` | Hold ungültig, Consumer verworfen. **Das ist eine rechtliche Abwägung der oberen Schicht** und der Kern erzwingt den Override nur mit `Admin`-Recht und Ledger-Eintrag (offene Frage K1, §22). |

Für Zurückhaltungen aus Rechtsgründen (Legal Hold, Einschränkung der Verarbeitung) stellt der Kern **keinen Zustand `Restricted`** bereit, sondern einen **Gate-Grund**: `Eraser::scope` wird von der oberen Schicht gar nicht erst aufgerufen, solange sie zurückhält. Der Kern bietet zusätzlich `Scopes::set_gate(s, GateReason)` (Lesesperre ohne Löschfolge, durabel, aufhebbar).

### 12.6 Kein stilles Fälschen

Der Catalog führt `erased_ranges: [(range|scope, S)]`. Eine View mit Stamp < `S` in einen dort verzeichneten Bereich antwortet `Err(Erased{..})`, nie leer und nie mit Altdaten. Einträge werden entfernt, sobald die Untergrenze `floor ≥ S` (und `keep_erased_marker` abgelaufen).

### 12.7 `rebucket` (Scope wechselt Kohorte)

Seltener, atomarer Umzug (zum Beispiel verlängerte Aufbewahrung): neue Kohorte wählen → Einträge des Scopes in einem Batch in die neue Kohorte schreiben (alte Stamps bleiben, Neuverpackung durch Run-Bau aus dem Scope-Bereich) → ein Edit `{files_add(neu), scope_cohort(s, neu), files_remove/Purge-Markierung(alt)}`. Zwischenzustand ist unsichtbar, weil die Zuordnung `scope → cohort` im selben Edit wechselt (INV-COH-2). Kosten O(Bytes des Scopes).

### 12.8 Grenzen (offen benannt)

Append-only-Spaces erlauben `Erase` nur mit `Admin` (im Receipt vermerkt). `ColdTier`-Dateien unterliegen den Garantien des Ports. Speicherinhalt, Auslagerungsdatei und Core-Dateien werden nicht kontrolliert (Puffer best-effort überschrieben, `forbid(unsafe_code)`; Speicherschutz liegt in einer Port-Implementierung). Externe Backups, Dateisystem-Journal, Flash-Wear-Leveling siehe §12.2. Eine Löschung **abgeleiteter Daten in Strukturen, die der Kern nicht kennt**, ist nur so sicher, wie ihre Besitzer den Vertrag §19.2 einhalten.

### 12.9 Lösch-Ledger und Backups

- **Ledger** (`ERASE-LEDGER/`): append-only, je Eintrag `{ticket, scope_id|cohort_id, level, stamp S, stage, ts, prev_hash}` mit BLAKE3-Kette. Enthält **keine Inhalte und keine Kennungen**. Signieren und Beweisberichte gehören zur oberen Schicht (der Kern liefert kanonische Bytes).
- **Checkpoint-Manifest** enthält `erase_epoch` (Ledger-Position). Das **Ledger liegt nie im Checkpoint**, sondern wird getrennt und länger aufbewahrt als jedes Backup.
- **`restore_check`/Wiederherstellung:** Vor dem Öffnen einer wiederhergestellten Kopie werden alle Ledger-Einträge nach `erase_epoch` erneut angewendet: Scopes werden `Gated/Shredded`, Purge läuft. Vernichtete Schlüssel kommen nicht aus dem Backup zurück (KEK-Trennung: Der Schlüsselbestand liegt nicht in den Checkpoint-Dateien, §13.2).
- **Hardlink-Checkpoints** sind Opt-in mit Warnung, weil ein Purge den alten Inode im Checkpoint nicht erreicht. Standard ist `Copy` oder `Reflink`.

---

## 13. Verschlüsselung und Schlüssel

### 13.1 Modi (stehen in FORMAT, unveränderlich)

| Modus | Schutz | Verfahren |
|---|---|---|
| `Plain` | Zufallsfehler | CRC32C, Ketten-Prüfsumme im Redo (nicht kryptografisch) |
| `Authenticated` | Integrität gegen Manipulation, nicht Vertraulichkeit | BLAKE3-keyed-MAC (Tag 16 B) mit Zusatzdaten (Dateiart, ID, Offset, Space), Kette im Redo, im Catalog, im Ledger |
| `Encrypted` | Vertraulichkeit und Integrität | AEAD (XChaCha20-Poly1305, 24-Byte-Nonce) für Blöcke, Redo, Catalog; Schlüssel per KDF |

Eigene Kryptografie wird nicht geschrieben (NZ-2). AEAD, MAC und KDF sind Ports. Austausch gegen AES-256-GCM-SIV ist ohne Formatbruch **nicht** vorgesehen (Algorithmus-ID im FORMAT, Wechsel = Migration). **Entscheidung SD-14:** XChaCha mit zufälligem 24-Byte-Nonce, weil bei langlebigen, umkopierten Dateien (Sweep, GC) Zählernonces fehleranfällig sind. Preis: 24 Byte je Körper-Record (Schwellenwert `value_separation_min` hält den Anteil klein).

### 13.2 Schlüsselhierarchie

```
KEK (KeyProvider, extern; nie in Dateien, nie in Checkpoints)
 ├─ Space-DEK        zufällig (Entropy), mit KEK umhüllt in KEYS, je Space
 │     ├─ Run-/Block-Schlüssel   = KDF(DEK, "ctxs run v1"  ‖ file_id)
 │     └─ Blob-Header/Index      = KDF(DEK, "ctxs blob v1" ‖ blob_no)
 ├─ Redo-Schlüssel   = KDF(KEK, "ctxs redo v1" ‖ segment_id)
 ├─ Catalog-/FORMAT-Schlüssel   = KDF(KEK, "ctxs cat v1")
 └─ Scope-Schlüssel  KeyProvider::scope_key(scope_id, epoch) -> Option<Key32>     (NICHT in KEYS)
         ├─ Körper-Schlüssel   = KDF(scope_key, "ctxs body v1" ‖ nonce)   Einzel-Scope-Pfad
         └─ CEK-Wrap           = AEAD(scope_key, cek)                      Mehr-Scope-Pfad
```

**`KeyProvider`-Vertrag:**

```rust
pub trait KeyProvider: Send + Sync {
    fn kek(&self) -> Result<Key32, KeyError>;
    fn scope_key(&self, s: ScopeId, epoch: u32) -> Result<Option<Key32>, KeyError>; // None = vernichtet
    fn create_scope_key(&self, s: ScopeId, entropy: &dyn Entropy) -> Result<u32 /*epoch*/, KeyError>;
    fn destroy(&self, s: ScopeId) -> Result<(), KeyError>;   // durabel vor Rückkehr, idempotent
    fn probe(&self, s: ScopeId) -> Result<KeyState, KeyError>; // Present | Absent
}
```

Der Kern speichert **keine** Scope-Schlüssel. Ein Schlüsselbund mit Millionen Scopes ist Sache des Providers (HSM, Tresor, eigener Store). Der Kern hält nur einen begrenzten **Schlüssel-Cache** (TTL, Obergrenze) mit **Generationszähler** (§14.5). Schlüsselmaterial liegt in `Zeroizing`-Hüllen.

### 13.3 Mehrere Scopes je Eintrag

- **Einzel-Scope (Normalfall):** Körper mit Schlüssel aus `scope_key` und zufälligem Nonce.
- **Mehr-Scope (2–4):** Zufälliger Körper-Schlüssel (CEK) pro Eintrag; CEK wird unter **jedem** Scope-Schlüssel umhüllt (Wrap-Liste im Header). Lesen entsiegelt über **irgendeinen** verfügbaren Scope-Schlüssel. Regel `multi_scope_rule`:
  - `AnyDestroyed` (Standard): Sobald **ein** Scope vernichtet wird, entfernt der Purge den Eintrag (Gate-Wirkung auf den Eintrag: `Erased`). Das ist die vorsichtige Wahl für Daten, die mehrere Betroffene enthalten.
  - `AllDestroyed`: Der Eintrag bleibt lesbar, bis alle Scopes vernichtet sind; die Wrap-Liste des vernichteten Scopes wird im Purge entfernt. Nur wählbar, wenn die obere Schicht rechtlich begründet hat, dass der Eintrag unabhängig von dem Betroffenen aussagekräftig ist (§22, K3).

### 13.4 Versiegelung und Redo

- Körper (und bei `PerScope` die **gesamte Op-Nutzlast** im Redo) werden in `apply` **vor** dem Redo-Frame versiegelt. Die Memtable hält Klartext, der Flush übernimmt den **bereits versiegelten** Körper (kein zweites Versiegeln, Nonce bleibt, dadurch ist der Sweep byte-identisch).
- Ohne Scope-Schlüssel (nach `K`) kann weder Redo-Replay noch Blob-Lesen Klartext erzeugen (INV-ERA-7).

### 13.5 Nonce- und Eindeutigkeitsregeln

Jeder AEAD-Aufruf erhält 24 Byte **aus `Entropy`** (nie gezählt). Property-Test: Kollisionsfreiheit der generierten Nonces über simulierte Läufe (statistisch) und Prüfung, dass **kein** Zähler-Nonce-Pfad im Code existiert (Layering-Test). Zusatzdaten (AAD) binden Ciphertext an `(file_kind, file_id, offset, space_id, scope_set_hash)`, sodass Umkopieren zwischen Orten fehlschlägt.

### 13.6 Schlüsselwechsel (Rotation)

Space-DEK-Rotation: neue DEK, alle Runs werden bei nächster Compaction neu geschrieben (Zeitplan), Blob-Header neu gewrappt. Scope-Schlüssel-Rotation (`epoch`): neue Schreibungen mit neuer Epoche, alte Körper bleiben unter alter Epoche lesbar, bis sie bei GC/Compaction umgezogen werden. **Rotation ersetzt keine Löschung.**

---

## 14. Nebenläufigkeit: Race-Freiheit durch Konstruktion

### 14.1 Prinzipien

1. **Ein Schreiber:** Alle Zustandsänderungen gehen durch den Committer (Rang 1) oder durch einen Catalog-Edit (serialisiert über den Catalog-Mutex Rang 2b, siehe §14.2).
2. **Unveränderlichkeit:** Dateien sind nach `rename` unveränderlich. Der veröffentlichte Zustand (`Version`) ist unveränderlich und wird durch `Arc`-Swap ersetzt.
3. **Besitz durch Claims:** Jede Datei hat höchstens einen Wartungs-Claim.
4. **Reihenfolge durch Edits:** Wahrheit entsteht durch durable Catalog-Edits, nicht durch Speicherzustand.
5. **Generationszähler statt Sperren** für Schlüssel-Cache und Scope-Zustand.
6. **Keine Sperre über E/A**, mit der **einzigen** Ausnahme des Committers während `sync_data` (P-9).

### 14.2 Sperrränge (nur aufsteigend, je Thread höchstens einmal; `debug_assertions` prüft per Rangstack)

| Rang | Sperre | Über E/A gehalten |
|---|---|---|
| 1 | `committer` | ja (nur Redo-`sync`) |
| 2 | `intake` | nein |
| 3 | `catalog_edit` (serialisiert Edits) | ja (`sync_data` des Catalog) — **einzige zweite Ausnahme**, kurz, nie zusammen mit `committer` über E/A gehalten, siehe unten |
| 4 | `state` (`RwLock<Arc<Version>>`) | nein |
| 5 | Memtable-`RwLock` | nein |
| 6 | `registry` (Holds, Consumer, Claims) | nein |
| 7 | `scope_table` | nein |
| 8 | `maint` (je Lane) | nein |
| 9 | `notify` | nein |

Ränge 6–9 sind Blätter. Cache- und Governor-Sperren sind Blätter. **Wichtig:** Committer-Rang 1 und Catalog-Rang 3 werden **nie gleichzeitig über E/A** gehalten: Der Committer gibt Rang 1 vor einem Catalog-Edit frei. Flush-Veröffentlichung und Redo-Anhängen sind dadurch entkoppelt, die Reihenfolge der Sichtbarkeit sichert `visible_stamp`.

### 14.3 Die Nahtstellen und ihre Lösungen

| Nahtstelle | Gefahr | Lösung | Nachweis |
|---|---|---|---|
| Lesen × Unlink | Datei weg, Leser hält Verweis | Graveyard + `Arc<Version>`-Referenz, Löschen erst nach letztem Drop (INV-GRAVE-1) | loom-Modell, Sim |
| Compaction × Erase | Compaction schreibt Daten zurück, die Erase verwirft | Claim-Konflikt ⇒ Compaction wird **abgebrochen**, ihre Ausgabe verworfen; Erase beansprucht Eingabedateien exklusiv; Erase-Edit prüft, dass **keine** Datei mit Überlappung im Edit fehlt (Validierung im Edit: `files_remove` nur für Claim-Inhaber) | TLA+-Modell, Sim |
| Flush × Erase | Memtable enthält noch Daten des Scopes | Gate prüft **vor** Memtable-Einfügung (§8.1 3b), Flush-Barriere in P1, Memtable-Bereich beim Gate als `ErasedMarker` überschrieben | Sim, Modell |
| Schlüssel-Cache × Vernichtung | Entsiegeln mit zwischengespeichertem Schlüssel nach `K` | Generationszähler: `K` erhöht `scope.gen` **vor** Rückkehr, jede Entsiegelung prüft `gen` vor und **nach** dem Entsiegeln (Optimistic Check) | loom |
| Checkpoint × Erase | Checkpoint kopiert Dateien, die Erase gerade ersetzt | Checkpoint nimmt `Arc<Version>` + schreibt `erase_epoch`; Erase-Edit P4 und Checkpoint-Start sind über `catalog_edit` serialisiert; Restore wendet Ledger an (§12.9) | Sim, Modell |
| Consumer × Erase/Compaction | Consumer verliert Versionen | Consumer ist Hold, `floor` = min(Holds); Erase mit `HoldPolicy` | Modell |
| Rebucket × Lesen | Leser sieht Eintrag zweimal oder nie | `scope→cohort` wechselt im **selben** Edit wie die Dateimengen; Version enthält die Zuordnung | Modell |
| Zwei Erase auf überlappenden Zielen | gegenseitige Blockade | Serialisierung nach Ticket-ID in der `urgent`-Lane, zweiter Plan wird nach dem ersten neu berechnet | Sim |
| Idempotenz-Token × Löschung | Token verrät Daten | Token-Tabelle ohne Inhalte, TTL, `idem_trim` im Catalog | Test |

### 14.4 Claim-Protokoll

```
claim(files, job) -> Ok(guard) | Err(Held{by})
   atomar unter `registry` (Rang 6)
   Priorität: urgent > flush > compact > blobgc > idle
   höhere Priorität darf niedrigere abbrechen: `job.cancel()` setzt ein Flag, der Job prüft es zwischen Blöcken,
   gibt Claims frei und verwirft Ausgabe (.building bleibt verwaist und wird bereinigt)
Ausgabe-Edit: Validierung `claims ⊇ files_remove`; sonst Edit abgelehnt (Defensive, INV-CLAIM-1)
```

### 14.5 Scope-Zustand und Schlüsselgenerationen

`ScopeState` im Speicher ist ein `AtomicU64` (`state:8 | gen:56`). Schreiber (Gate, K, Purge) erhöhen `gen`. Leser prüfen `state` beim Beginn und `gen` nach dem Entsiegeln. Dadurch kann kein Leser nach Rückkehr von `K` neuen Klartext **zurückgeben** (er kann höchstens zuvor entsiegelte Daten im eigenen Speicher halten, Grenze §12.8).

### 14.6 Verbotene Muster (per Lint und Review)

`unwrap/expect/panic/todo/unreachable/indexing_slicing` verboten (Workspace-Lint). Kein `std::sync::Mutex` außer über `sync::Ranked`. Keine Hintergrundthreads außerhalb des `Spawner`. Keine blockierende E/A unter Rang ≥ 4. Keine zweite Schreibquelle außer Committer und Catalog-Edit. Kein Rückruf (`Observer`, `SegmentOwner`) unter Sperren.

---

## 15. Fehlermodell

| Klasse | Beispiele | Verhalten |
|---|---|---|
| F-1 transient | `EINTR`, kurze Überlast | Wiederholung mit Backoff, begrenzt |
| F-2 Ressource | `ENOSPC`, Handle-Limit | Rückdruck, dann Poison des Schreibpfads; Lesen bleibt |
| F-3 E/A-hart | `EIO`, Fsync-Fehler | **nie wiederholen** (Seiten können als sauber gelten): Poison, Neustart über Recovery (Hintergrund: Fsync-Fehlersemantik auf Linux) |
| F-4 Integrität | CRC/MAC-Fehler | kleinste isolierbare Einheit „vergiftet“ (Block → Run → Kohorte → Space), `Err(Corrupt{scope})`, Lesen anderer Einheiten läuft |
| F-5 Format | `FormatTooNew`, unbekannter Catalog-Tag | Öffnen verweigert, nie ignorieren |
| F-6 Schlüssel | Provider offline, Schlüssel fehlt | `LockedKeys`-Modus bzw. `Shredded` |
| F-7 Aufrufer | Guard verletzt, Scope gesperrt, Größe | `ApplyError`-Varianten mit `hint()` |
| F-8 Logik | Invariante verletzt (Bug) | Debug: Abbruch im Test; Release: Poison + Observer-Ereignis. **Kein** `panic` im Produktionspfad |

**Annahmen über die Umgebung (A-Liste, zu bestätigen):** A-1 `fsync` auf Datei **und** Verzeichnis macht Änderungen durabel (POSIX). A-2 Ein einzelner `write` ≤ Seitengröße kann zerreißen (Torn Write), deshalb sind alle Rahmen prüfsummengeschützt. A-3 Fsync-Fehler sind nicht wiederholbar (F-3). A-4 Verzeichnis-Rename ist atomar. A-5 Prozessspeicher ist vertrauenswürdig. A-6 Der `KeyProvider` ist durabel, wenn `destroy` zurückkehrt.

---

## 16. Invarianten (Auswahl, vollständiger Katalog im Test-Index)

| ID | Aussage |
|---|---|
| INV-COMMIT-1 | Stamps sind lückenlos und streng monoton in Durabilitätsreihenfolge |
| INV-COMMIT-2 | Ein sichtbarer Eintrag ist durabel (außer `Relaxed`) |
| INV-ATOM-1 | Ein Batch ist ganz oder gar nicht sichtbar, auch nach Absturz |
| INV-VIEW-1 | Eine View bei Stamp `V` sieht dieselben Daten unabhängig von späteren Schreibungen |
| INV-LOCK-1 | Sperren nur aufsteigend, je Thread höchstens einmal |
| INV-CAT-1 | Datei lebendig ⇔ durabler `files_add` ohne `files_remove` |
| INV-GRAVE-1 | Keine Datei wird gelöscht, solange eine `Arc<Version>` auf sie zeigt |
| INV-CLAIM-1 | Ein Edit entfernt nur Dateien, deren Claim der Job hält |
| INV-COH-1 | Kein Run, Blob oder Segment enthält Scopes aus zwei Kohorten |
| INV-COH-2 | `scope → cohort` wechselt nur im selben Edit wie die Dateimengen |
| INV-SCOPE-1 | `ScopeId` ist nicht aus einer Kennung ableitbar (Erzeugung nur über `Scopes::register`) |
| INV-ERA-1 | Logische Atomarität: für Views ≥ `S` ist der Scope leer, für Views < `S` vollständig `Erased` |
| INV-ERA-2 | Nach `Purged` enthält weder ein lebender Run noch ein lebendes Redo-Segment eine Version des Scopes |
| INV-ERA-3 | Reihenfolge G < K < P4 < Unlink; der Kern führt P nie aus, solange `probe` den Schlüssel liefert (bei `Cryptographic`) |
| INV-ERA-4 | Nur Roll-forward: ein begonnener Erase wird nach jedem Absturz fortgesetzt oder ist nie begonnen |
| INV-ERA-5 | Reads unterhalb `S` liefern `Erased`, nie Daten, nie „leer“ |
| INV-ERA-6 | Nach `Purged` enthält keine lebende Blob-Generation einen Record des Scopes (nur `Hole`) |
| INV-ERA-7 | Ein Körper eines vernichteten Scopes ist weder in Redo noch in Blob lesbar; `body()` liefert `Shredded` |
| INV-ERA-8 | **(neu)** `ErasureReceipt` existiert nur, wenn INV-ERA-2/6 und der Ledger-Eintrag durabel sind und alle Segmente mit Scope-Bindung bestätigt haben |
| INV-ERA-9 | **(neu)** Nach `restore` ist jeder Scope aus dem Ledger wieder `≥ Shredded` |
| INV-KEY-1 | Schlüsselmaterial liegt nie in Dateien außer umhüllt (Space-DEK) und nie im Checkpoint |
| INV-KEY-2 | Keine Entsiegelung liefert Klartext, wenn `scope.gen` sich während des Aufrufs geändert hat |
| INV-FMT-1 | Unbekannte Catalog-Tags und zu neue Formate führen zu Fehlern, nie zu Ignorieren |

Jeder Test zitiert die ID (`// INV-ERA-4`).

---

## 17. Verifikation (Gate-relevant)

Ziel: **Fehlerfreiheit im Sinne belegter Eigenschaften**, nicht „hat keine Fehler gezeigt“.

### 17.1 Ebenen

| Ebene | Werkzeug | Inhalt |
|---|---|---|
| Formal | **TLA+/TLC** (oder Quint) | Modelle: (M1) Commit/Sichtbarkeit/Recovery, (M2) Erase-Protokoll G/K/P/D mit Absturz an jedem Schritt, (M3) Compaction × Erase × Claims, (M4) Checkpoint × Erase × Restore, (M5) Rebucket. Eigenschaften: INV-ATOM/ERA/COH/GRAVE. |
| Modellprüfung Code | **loom** | Sperren, `Arc`-Swap, Generationszähler, Claims, Graveyard, Committer-Gruppenführung |
| Deterministische Simulation | eigener Scheduler + `shuttle` | **Hauptwerkzeug gegen Races:** seeded Interleavings aller Threads, `FaultFs`, `SimClock`, `SimEntropy`; Millionen Läufe, reproduzierbar per Seed |
| Absturz-Matrix | `FaultFs` | Abbruch an **jedem** E/A-Index, inkl. Torn Writes, verlorene Fsyncs, Rename-Reihenfolgen; Prüfung aller INV nach Neustart |
| Eigenschaftstest | `proptest` | Codecs, Ordnung, Kompaktion äquivalent zum Referenzmodell (einfache BTreeMap-Orakel mit Versionen) |
| Linearisierbarkeit | Historienprüfer (Wing-Gong) | Aufzeichnung von `apply`/`view`-Historien aus Sim, Prüfung gegen sequenzielle Spezifikation |
| Löschorakel | `ErasureOracle` (Feature `testing`) | Eindeutiges Muster in Schlüssel, Header, Körper, mit/ohne Scope; nach Erase **alle Bytes aller Dateien** durchsuchen (Runs, Blobs aller Generationen, Redo, `.building`, Segmente, `MemFs`-Zustand, Checkpoints) |
| Fuzzing | `cargo-fuzz` | Dekoder aller Formate, Recovery auf beschädigten Verzeichnissen (Soll ≥ 8 Ziele) |
| Referenzvektoren | Python-Codierer | Goldene Bytes je Format (Anhang A) |
| Statisch | clippy, `cargo-deny`, Layering-Test, `make metrics` | Lint-Regeln, Lizenzen/Advisories, Schichtregeln |
| Dauerlauf | Soak (72 h), Crash-Loop, Platte-voll-Test | Stabilität, Speicherwachstum, Handle-Lecks |

### 17.2 Orakel-Vertrag

Bei `Plain`: Trefferzahl 0 für **alle** Muster nach Purge. Bei `Cryptographic` vor Purge: Körpermuster 0 Treffer (ohne Schlüssel unlesbar), Schlüsselmuster ≥ 0 (erwartet, bis Purge). Nach Purge: 0. Zusätzlich prüft `verify_erasure` die Catalog-Seite.

### 17.3 CI

Pflicht: `check`, `fmt`, `clippy -D warnings`, `test` (alle Targets), `layering`, `loom`, `doc -D warnings`, `deny`, Sim-Kurzlauf, Absturz-Matrix-Kurzlauf. Nachtlauf: lange Sim, Fuzz, Orakel-Vollsuche. **Ohne CI sind keine Gates gültig.**

---

## 18. Leistung (Hypothesen mit Messverfahren)

Alle Werte sind **Entwurfsziele**. Freigabe nur mit Messbericht (Hardware, Dateisystem, Last, Methode, Streuung).

| ID | Hypothese | Messung |
|---|---|---|
| H-1 | `apply` mit Gruppen-Fsync: Durchsatz ≥ 80 % der Fsync-Rate × mittlere Gruppengröße; Latenz p99 ≤ 2 × Fsync-Latenz bei mittlerer Last | Mikrobench `apply`, variierende Schreiberzahl |
| H-2 | Punktabfrage warm: ≤ 1 Datenblock-Read je Treffer-Level, p99 ≤ 50 µs (Block im Cache) | Zipf-Last |
| H-3 | Header-Scan ohne Körper: ≥ 10× Durchsatz gegenüber Scan mit Körper bei Körper ≥ 4 KiB | Scan-Bench |
| H-4 | `scan_budget` liest ≤ 1,2 × Blöcke des theoretischen Minimums | Gewichtsverteilungen |
| H-5 | Schreibverstärkung Metadaten ≤ 15, Körper ≤ 2 (Wertetrennung) | Compaction-Statistik |
| H-6 | **Gate + Schlüsselvernichtung** für einen Scope: ≤ 10 ms ohne Wartezeit auf Provider | Löschbench |
| H-7 | **Purge** eines Scopes: Dauer proportional zur Kohorten-Treffergröße, **nicht** zur Store-Größe (Faktor ≤ 1,5 bei 10× Store-Größe) | Skalierungstest |
| H-8 | **Cohort-Drop** ≤ 100 ms je 1000 Dateien (ohne Scrub) | Drop-Bench |
| H-9 | Bündelung: Purge von 1000 Scopes einer Kohorte ≤ 2 × Purge von 1 Scope | Bündelungsbench |
| H-10 | Speicher: ≤ `memtable_bytes × (1 + max_immutable)` + Cache-Obergrenze + O(offene Kohorten) Metadaten | RSS-Messung |
| H-11 | Wiederanlauf: Zeit ∝ Redo-Rückstand + Catalog-Größe; 1 GiB Redo ≤ 10 s | Recovery-Bench |
| H-12 | Ribbon-Filter ca. 30 % kleiner als Bloom bei gleicher Fehlrate | Filterbench (Literaturwert, nachmessen) |

Wenn eine Hypothese scheitert, wird **die Hypothese** korrigiert und im Bericht begründet, nicht das Messverfahren gelockert.

---

## 19. Erweiterungsverträge (für aufsetzende Schichten)

Der Kern kennt die Bedeutung fremder Strukturen nicht. Er bietet fünf feste Verträge. Dadurch wird die spätere Anbindung zusätzlicher Komponenten **ohne Formatbruch** möglich.

### 19.1 ExtensionBlock (an Runs gekoppelt)

Optionale, opake Blöcke im Run (Tag `u16`, Version `u16`). Ein `ExtensionBuilder`-Trait erhält beim Flush/Compaction den Strom der Einträge und schreibt Blöcke. Der Kern speichert, prüft Prüfsumme/Tag, kopiert bei Compaction nicht blind, sondern ruft `merge(inputs) -> output` auf. Fehlt der Besitzer (Tag unbekannt), bleibt der Block unverändert erhalten oder wird beim Rewrite verworfen (Option `drop_unknown_ext`, Standard: **Fehler**, nicht still verwerfen). **Löschvertrag:** `rewrite_without(scopes|range)` ist Pflichtmethode; ein Besitzer ohne diese Methode darf keinen Block in `scoped`-Spaces anlegen.

### 19.2 Segment-Register (entkoppelte Dateien)

Vom Besitzer geschriebene Dateien **neben** den Runs, vom Catalog verwaltet (`segment_add/remove`), liegen in `cohorts/<c>/segments/`.

```rust
pub trait SegmentOwner: Send + Sync {
    fn kind(&self) -> SegmentKind;                          // Tag
    fn covers(&self, seg: &SegmentMeta) -> ScopeCover;      // Exclusive(CohortId | ScopeId) | Mixed(exakte Liste / Filter)
    fn rewrite_without(&self, seg: &SegmentMeta, scopes: &ScopeSet, io: &mut SegmentIo) -> Result<RewriteOutcome, SegErr>; // Neubau oder Drop
    fn on_dropped(&self, seg: &SegmentMeta);                // nur Information
    fn watermark(&self) -> Stamp;                           // bis zu welchem Stamp ist das Segment aufgeholt
}
```

**Vertrag:** Ein Segment mit `Mixed`-Abdeckung wird bei Löschung über `rewrite_without` neu gebaut oder ganz verworfen und neu aufgebaut. **Der Erase gilt erst als `Done`, wenn alle überlappenden Segmente bestätigt haben** (INV-ERA-8). Ein Besitzer, der nicht rechtzeitig antwortet, hält den Vorgang in `Purging` (Alarm), er wird nie stillschweigend übersprungen. `ScopeCover::Exclusive` ist bevorzugt: Ein Segment, das nur Daten einer Kohorte oder eines Scopes enthält, wird beim Cohort-Drop einfach gelöscht, ohne Besitzer zu fragen.

### 19.3 Lineage und abgeleitete Einträge

`Header.lineage` (≤ 8 `EntryRef`) und `flags.derived` kennzeichnen Einträge, die aus anderen entstanden sind. Hilfsfunktion:

```rust
pub fn derive_scopes(view: &View, sources: &[(Space, Key)]) -> Result<ScopeSet, Error>; // Vereinigung der Quell-Scopes (≤ 4, sonst Err(TooManyScopes))
```

Regel: In einem `scoped`-Space mit `require_scope = true` muss jeder abgeleitete Eintrag eine **nichtleere Scope-Menge** tragen; der Kern prüft, dass sie die Scope-Mengen der angegebenen `lineage`-Quellen **enthält** (soweit die Quellen in der View lesbar sind). So läuft die Löschung abgeleiteter Daten **über dieselbe Scope-Mechanik** und nicht über Sondervorgänge.

### 19.4 Consumer und Change-Feed

`Consumer` (§6) ist die einzige Schnittstelle für Strukturen, die **aufholen** müssen (inkrementell abgeleitete Daten). Garantien: Reihenfolge nach Stamp, mindestens einmal (idempotente Verarbeitung ist Aufgabe des Konsumenten), Löschmarker erscheinen als `Change::Erased{scope|range, at}`, damit Konsumenten ihre eigenen Strukturen **innerhalb der Frist** bereinigen können. `ack` ist durabel. Ein Consumer ist ein Hold und kann Löschung blockieren (`HoldPolicy`).

### 19.5 Metadaten und Gewichte

`class`, `weight`, `meta` (§5.3) sind das Schnittstellenfeld für Rang- und Budgetlogik in oberen Schichten. Der Kern stellt `scan_budget`, `scan_class_mask` und blockweise Aggregate bereit und garantiert Header-Lesbarkeit ohne Körper. Mehr Kenntnis hat er nicht.

### 19.6 Stabilität

Ports, Werttypen und diese fünf Verträge folgen **semver** in `contextra-storage-ports`. Änderungen an Formaten laufen nur über §1.5-ähnliche Änderungsanträge mit zwei Reviewern und Migrationsplan.

---

## 20. Phasen und Gates

| Phase | Inhalt | Gate (Nachweis) |
|---|---|---|
| **P0 Fundament** | `ports`, `codec`, `env` (MemFs, RealFs, FaultFs, Sim), `sync`; Referenzcodierer; Lint- und CI-Gerüst | G-0: SPEC R3 normativ (Review, §22 entschieden); CI grün |
| **P1 Format** | Formate v1 (Redo, Run, Blob, Catalog, Ledger) inkl. Scope/Kohorte/ExtensionBlock; Goldene Vektoren; Fuzz-Ziele | **G-F Formatfreeze:** Golden, Property, Fuzz ≥ 8 Ziele, Review zu §7.7 |
| **P2 Schreibpfad** | Committer, Redo mit echter Nutzlast (S-1 beheben), Memtable, Recovery, Catalog | G-W: Absturz-Matrix vollständig grün, M1 modelliert und geprüft, Linearisierbarkeit in Sim |
| **P3 Lesepfad/Runs** | Flush, Runs, Cache, Views, Scan/Budget | G-R: Referenzmodell-Äquivalenz, H-2…H-4 gemessen |
| **P4 Wartung** | Compaction je Kohorte, Claims, Blob-GC, Governor | G-M: Modell M3, Sim-Läufe ohne Invariantenbruch, H-5 gemessen |
| **P5 Löschung** | Scopes, Gate, Schlüssel, Purge, Cohort-Drop, Ledger, Orakel, Restore | **G-E:** Modelle M2/M4/M5, Orakel 0 Treffer in allen Matrizen, H-6…H-9 gemessen |
| **P6 Härtung** | Soak, Fuzz-Dauerlauf, Platte-voll, Fehlerinjektion, Facade-Stabilisierung, Dokumentation | **G-P Produktionsreife:** alle INV getestet (M-09 = 100 %), Soak 72 h, Messbericht (§18) vollständig, Sicherheitsreview der Krypto-Nutzung |

**Regeln:** Keine Phase beginnt vor dem Gate der vorherigen. Keine Statusänderung ohne Testnachweis. Erst nach **G-P** werden aufsetzende Komponenten angebunden; bis dahin enthalten Repo und Dokumente **nur** den Kern und die Verträge aus §19.

**Sofortmaßnahmen (vor P1):** (1) Committer-Redo-Stub ersetzen (Blocker S-1), (2) `recovery`, `flush`, `compaction` in den Modulbaum, (3) `tests/engine` wieder kompilierbar, (4) `unwrap`/`panic` aus Produktionscode, (5) `default = ["testing"]` entfernen, (6) CI einrichten, (7) `deny.toml` vervollständigen, (8) Toolchain-Pin klären.

---

## 21. Entscheidungsregister (neue und geänderte Einträge)

| ID | Entscheidung | Begründung | Verworfene Alternative |
|---|---|---|---|
| SD-1 | Löschgerechtes Layout (Scope-Ordnung + Kohorten) | macht beide Löschwege billig (§3.3) | Löschung als reiner Umbauvorgang über verstreute Dateien (R2) |
| SD-2 | Kohorte = Mandanten-Tag × Aufbewahrungsklasse × Ablaufzeitraum (× Shard) | Ablauf = Drop, Mandantenwechsel = Drop | Datei je Scope (Skalierung), Kohorte nach Hash (kein Drop-Nutzen) |
| SD-3 | Bewilligte Abhängigkeiten (§4.5) | Sicherheit ohne eigenes `unsafe` | eigener Baum im Kern |
| SD-4 | Scope-Menge 1–4, Wrap-Liste je Scope | Mehrbetroffenen-Einträge ohne Formatbruch | ein Schlüssel je Eintrag (R2) |
| SD-5 | Redo-Nutzlast je Scope versiegelt, Frame-Kopf offen | Redo sofort unlesbar nach Schlüsselvernichtung, Recovery plant ohne Schlüssel | Redo nur unter Redo-Schlüssel (Klartext bis Flush-Barriere) |
| SD-6 | Purge gebündelt mit Frist (72 h Standard) | amortisiert Rewrite, Frist begrenzt Restexposition | sofortiger Purge je Antrag |
| SD-7 | Schlüssel- und Header-Hygiene über Regel P-K und Purge | Block-Verschlüsselung je Scope würde Schlüsselordnung zerstören | Schlüsselversiegelung je Eintrag (verhindert Bereichsscans) |
| SD-8 | Eigenes **Ledger** außerhalb von Catalog und Checkpoints | Backup-Problem: Löschungen werden bei Restore erneut angewendet | Ledger im Checkpoint |
| SD-9 | `Restricted` nicht im Kern, nur `set_gate` | Rechtsgründe sind Sache der oberen Schicht | Zustandsmodell mit Rechtsgründen im Kern |
| SD-10 | Zwei Wartungsjobs + `urgent` | Flush darf nie hinter Compaction warten | ein Job je Instanz (R2 SD-16) |
| SD-11 | Fünf Erweiterungsverträge (§19) | Anbindung ohne Formatbruch | Erweiterungspunkte nachrüsten |
| SD-12 | Kein 2PC, sondern **ein** Redo-Frame und **ein** Catalog-Edit | alles Atomare liegt im Kern; Mehr-Index-Atomarität durch Consumer und Segment-Watermark | verteilter Intent-Log mit Rückrollen |
| SD-13 | Hardlink-Checkpoints nur Opt-in | Purge erreicht Hardlink-Inodes nicht | Hardlink als Standard (schnell, aber löschfeindlich) |
| SD-14 | XChaCha20-Poly1305, Zufallsnonce | kein Zählerzustand, sicher bei Umkopieren | AES-GCM mit Zählernonce, GCM-SIV (später prüfbar) |
| SD-15 | Formale Modelle (TLA+) als Gate für Erase- und Compaction-Protokolle | Race-Freiheit ist sonst nur Hoffnung | nur Tests |
| SD-16 | `Fs`-Fehler F-3 nie wiederholen | Fsync-Semantik, Datenverlustrisiko | Retry-Schleifen |
| SD-17 | Gate-Zustand als atomarer Verbund `state|gen` | Leser ohne Sperre, Prüfung vor und nach Entsiegeln | Sperre je Scope |
| SD-18 | `weight`/`class`/`meta` im Header, Aggregate im Block-Index | Budgetscans ohne Körper, ohne Wissen über Bedeutung | Zusatz-Spaces der oberen Schicht |

---

## 22. Offene Punkte (Entscheidung des Auftraggebers oder juristische Prüfung nötig)

Diese Punkte **darf** ich nicht still entscheiden:

| # | Frage | Wirkung | Mein Vorschlag |
|---|---|---|---|
| O-1 (K1) | Soll `HoldPolicy::Override` (Datenschutz schlägt Hold) im Kern existieren? | Rechtsabwägung, hier nur als `Admin`-Operation mit Ledger | Standard `Fail`; Override nur mit Admin und Pflichtgrund |
| O-2 | Ist die **Frist von 72 h** für den Purge von Header und Schlüsseln tragfähig? Genügt Körper-Unlesbarkeit sofort? | Rechtsfrage (Datenschutzbeauftragter) | Konfigurierbar, Standard 72 h, Orakel-Nachweis |
| O-3 (K3) | `multi_scope_rule` Standard `AnyDestroyed` oder `AllDestroyed`? | Mehrbetroffenen-Daten | `AnyDestroyed` (vorsichtig) |
| O-4 | Sind Schlüssel (`user_key`) je nach Anwendung personenbezogen? Reicht Regel P-K? | Restexposition bis Purge | Regel P-K plus `lint_keys`; Entscheidung je Anwendung |
| O-5 | **Hash-Identität:** Inhalts-Hash von Klartext kann Rückschlüsse erlauben; soll der Kern Inhalts-Hashes über **versiegelte** Körper bilden? | Dedup entfällt für Scope-Daten | Ja für `scoped`; Dedup nur in `plain`-Spaces |
| O-6 | Schlüsselhaltung (HSM, eigener Tresor): Wer liefert den `KeyProvider` in Produktion? | Verfügbarkeit und Durabilität von `destroy` | Referenz in `crypto-ref`, Produktivprovider separat |
| O-7 | Maximale Kohortenzahl und Zeitraumgröße (Monat/Quartal/Jahr) | Dateizahl gegen Überretention | 4096 / Monat, aus Last ableiten |
| O-8 | Toolchain-Pin (BL-7), `GH_TOKEN`, CI-Plattform | Gates gültig erst mit CI | CI mit Pin auf ladbare Version |
| O-9 | Altbestand: `REPORT.md` (nicht reproduziert), `RunBuilder`, `spec/implementation-rev3`, `legacy/` | Hygiene | Kennzeichnen/entfernen nach Freigabe |
| O-10 | Kennungs-Kollisionen (`E-n`, `M-nn`, `A-n`) bereinigen | Lesbarkeit | Präfixe `ER-*` (Erase-Schritte jetzt G/K/P/D), `ME-*`, `AN-*` |

---

## Anhang A — Referenzvektoren (Pflicht vor G-F)

Je Format mindestens: leeres Objekt, ein Eintrag, Mehr-Scope-Eintrag, Tombstone, Blob-Record mit Hole, Catalog-Edit-Kette, Ledger-Kette, Redo-Frame mit versiegelter Nutzlast, abgeschnittener Frame, beschädigter Frame. Erzeugung durch `tools/reference/generator.py` (unabhängig von Rust), Vergleich im Test `golden`.

## Anhang B — Beispielablauf: Löschung eines Betroffenen

1. Obere Schicht ruft `eraser().scope(s, Cryptographic)`.
2. **G:** Gate-Edit durabel (ms). Ab jetzt `Erased` für alle Lesepfade.
3. **K:** `KeyProvider::destroy(s)`; Cache geleert; `Shredded`-Edit. Körper in Redo, Blob und Backups sind unlesbar.
4. Der Ticket-Status ist `KeyDestroyed`. Die obere Schicht kann dem Betroffenen **jetzt** antworten, dass Inhalte nicht mehr lesbar sind, und sieht in `status` den offenen Purge samt Frist.
5. **P:** Innerhalb der Frist wird gebündelt gereinigt (Header, Schlüssel, Löcher in Blobs), Segmente der Besitzer bestätigen.
6. **D:** Unlink, Verzeichnis-Sync, Ledger-Eintrag, `ErasureReceipt`.
7. `verify(ticket)` und (in Tests) das Orakel bestätigen 0 Treffer.

## Anhang C — Zustandsdiagramme

```
Scope:   Live ──G──► Gated ──K──► Shredded ──P4──► Purged        (nur vorwärts; Gate ohne Löschfolge: Live ⇄ Gated(GateReason))
Kohorte: Active ──CD1──► Retiring ──CD3──► Dropped
Datei:   Building ──rename+Edit──► Live ──Edit──► Obsolete ──letzte Referenz──► Unlinked
Erase:   Gated ─► KeyDestroyed ─► Purging ─► Done(Receipt)         (Absturz an jeder Stelle: Fortsetzen, nie Rückabwicklung)
```

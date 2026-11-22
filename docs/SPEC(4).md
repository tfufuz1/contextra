# SPEC.md — `contextra-storage` (Speicherkern)

|              |                                                                                                                                                                                                                                                                                                                  |
| ------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Status       | **Revision R5, Freigabekandidat.** Ersetzt R2, R3 und R4 vollständig. Normativ wird sie durch Gate **G-0** (§21): zwei Reviewer, offene Punkte aus §23 entschieden. Bis dahin ist jeder Abschnitt änderbar, danach gilt §1.5.                                                                                    |
| Gegenstand   | Eingebetteter, synchroner, crash-sicherer Schlüssel-Wert-Kern mit atomaren Mehr-Space-Änderungen, MVCC-Sichten, **löschlokaler Datenanordnung** (Cluster) und technisch nachweisbarer Löschung (kryptografisch und physisch).                                                                                     |
| Crates       | `contextra-storage` (Kern), `contextra-storage-ports` (Traits und Werttypen), `contextra-crypto-ref` (Referenz-Primitive, nur Tests und Erstbetrieb).                                                                                                                                                            |
| Sprache      | Prosa und Rustdoc deutsch, Bezeichner englisch (P-7).                                                                                                                                                                                                                                                            |
| Verifikation | Entstanden durch Lesen der Vorgängerfassungen, des Prüfberichts R3 und des Altcodes. **Nichts wurde kompiliert oder gemessen.** Alle Leistungsangaben sind Hypothesen (`H-*`, §19). Alle Formate brauchen vor **G-F** Referenzvektoren aus einem unabhängigen Referenzcodierer (Anhang A).                      |
| Rechtlich    | Die Spezifikation benennt **technische** Eigenschaften. Rechtsfolgen (Art. 17 DSGVO, Aufbewahrung, Beweislast, Fristen) sind keine Aussage dieses Dokuments und brauchen juristische Prüfung (§23).                                                                                                              |

---

## 1. Geltungsbereich und Konventionen

### 1.1 Zweck

`contextra-storage` speichert Daten für Sprachmodell-Anwendungen: viele kleine atomare Schreibvorgänge, große unveränderliche Inhalte, kurzlebige Sichten, Lesen von Kopfdaten ohne Körperzugriff, budgetierte Ergebnisseiten, Verlauf mit Fortschrittszeigern und **Löschung nach DSGVO als Entwurfsprinzip, nicht als Nachrüstung**. Der Kern kennt keine Fachlogik, keine Rechte, keine Mandanten und keine Rechtsgrundlagen. Er liefert **Mechanismus, nie Politik** (NZ-4).

### 1.2 Normsprache

**MUSS / DARF NICHT** zwingend, Verstoß ist ein Fehler. **SOLL / SOLL NICHT** Regelfall, Abweichung braucht Begründung im Review. **KANN** optional.

### 1.3 Dokumentenhierarchie

1. `SPEC.md` normativ (Formate, Protokolle, Invarianten, API-Vertrag).
2. `IMPLEMENTATION_SPEC.md` operativ (Phasen, Gates, Ist-Stand). Widerspruch ⇒ SPEC gilt.
3. Alles andere ist Herkunft und keine Anforderungsquelle.

### 1.4 Kennungen

`Z-n` Ziel · `NZ-n` Nicht-Ziel · `AN-n` Annahme · `P-n` Codierregel · `F-n` Fehlerklasse · `L-n` Leitsatz · `SD-nn` Entscheidung (§22) · `INV-*` Invariante (§17) · `H-n` Hypothese (§19) · `O-n` offene Entscheidung (§23) · `G-*` Gate (§21) · `M-n` formales Modell (§18) · `ER0…ER7` Schritte der Löschung (§13.3). Präfixe sind kollisionsfrei: kein `E-n`, kein `A-n`.

### 1.5 Änderungsprozess

Änderungen an §7, §9, §12, §13, §14, §15 und §17 (Formate, Protokolle, Invarianten) brauchen zwei Reviewer, aktualisierte Referenzvektoren (Anhang A) und eine Prüfung gegen die Freeze-Liste (§7.9). Nach **G-F** ist jede Formatänderung ein Migrationswerkzeug (SD-04), nie ein Zweitpfad im Kern.

---

## 2. Leitsätze

| Nr. | Leitsatz | Folge |
|---|---|---|
| L-1 | **Löschbarkeit ist Layout, nicht Prozedur.** Was gemeinsam gelöscht wird, liegt gemeinsam. Der **Cluster** ist zugleich Löschdomäne, Schlüsseldomäne und Platzierungseinheit. | §5.3, §11.3 |
| L-2 | **Ein Schreiber, viele Leser, unveränderliche Dateien.** Nebenläufigkeit entsteht nur an wenigen, modellierten Nahtstellen. | §15 |
| L-3 | **Der Catalog ist die einzige Wahrheit** über lebendige Dateien und Zustand. Ein Catalog-Edit ist die einzige Grenze zwischen „unreferenziert“ und „lebendig“. | §7.7, §8 |
| L-4 | **Gate vor Schlüssel, Schlüssel vor Datei.** Die Reihenfolge der Löschung ist Vertrag und Invariante. | §13.3 |
| L-5 | **Ehrliche Garantien.** Jede Löschstufe nennt, wogegen sie nicht schützt. | §13.2 |
| L-6 | **Kein stilles Fälschen.** Gelöscht heißt `Erased`, nie „leer“ und nie Altdaten. | §13.6 |
| L-7 | **Kein `unsafe` im Kern.** Plattformnahes liegt hinter Ports. Jede Abhängigkeit braucht eine Entscheidung. | §4 |
| L-8 | **Leistung wird gemessen oder nicht behauptet.** Jede Zahl ist eine Hypothese mit Messverfahren. | §19 |
| L-9 | **Erweiterbar ohne Formatbruch.** Fremde Strukturen hängen sich über feste Verträge ein; der Kern kennt ihre Bedeutung nicht. | §20 |
| L-10 | **Fehler sind Werte.** Kein `panic` im Produktionspfad; jede Fehlerklasse hat ein definiertes Verhalten, auch „Ausgang unbekannt“. | §16 |
| L-11 | **Nichts doppelt.** Jede Aufgabe hat genau einen Mechanismus (ein Schreibpfad, ein Erweiterungspfad, ein Löschpfad). | durchgängig |
| L-12 | **Mechanismus statt Politik.** Der Kern erzwingt Reihenfolgen und Sperren, nicht Fristen, Zwecke oder Rechtsgründe. | NZ-4 |

---

## 3. Ziele, Nicht-Ziele, Annahmen

**Ziele**

- **Z-1 Atomarität:** Eine `Change` über beliebig viele Spaces und Cluster wird ganz oder gar nicht sichtbar und durabel.
- **Z-2 Durability:** Nach `Receipt{durable:true}` überlebt die Änderung Absturz und Stromausfall (Grenzen: §16, AN-1…AN-4).
- **Z-3 Snapshot-Isolation:** Sichten sehen einen festen Stamp. Lesen blockiert nie Schreiben.
- **Z-4 Lokalität:** Alle Daten eines Clusters liegen je Space zusammenhängend in jeder Datei (§11.3).
- **Z-5 Löschung je Cluster** mit Kosten proportional zum Datenvolumen des Clusters und seiner Nachbarn **in derselben Datei**, nicht zur Gesamtgröße (§13.4).
- **Z-6 Nachweisbare Löschung:** Quittung, Ledger, Prüflauf, Orakel.
- **Z-7 Kopfdaten ohne Körper:** Rangfolge, Filter und Budgetseiten lesen keine Körper.
- **Z-8 Race-Freiheit** durch Konstruktion, belegt durch Modelle und Simulation (§15, §18).
- **Z-9 Ressourcenschonung:** begrenzter Speicher, begrenzte Handles, begrenzte Wartungslast (Governor).
- **Z-10 Stabile Verträge** für Aufrufer und Erweiterungen (§20).

**Nicht-Ziele**

- NZ-1 Verteilung, Replikation, Netzwerkprotokolle.
- NZ-2 Eigene Kryptografie; es werden geprüfte Primitive über Ports verwendet.
- NZ-3 SQL, Schemas, Sekundärindizes, serialisierbare Transaktionen mit Lese-Sets (Guards ersetzen sie).
- NZ-4 Rechtsbewertung: welcher Cluster, welche Frist, welcher Zweck, welche Ausnahme.
- NZ-5 Garantien gegen Speicherabbilder, Swap, Core-Dateien (best effort, §13.8).
- NZ-6 Nachträgliche Umordnung durch den Kern auf Wunsch der Anwendung: Platzierung folgt allein der `ClusterId`-Ordnung (§11.3).
- NZ-7 Inhaltsadressierung über Klartext-Hashes (INV-PRIV-1).

**Annahmen über die Umgebung** (zu bestätigen in P0, Test `env_assumptions`)

- AN-1 `fsync`/`fdatasync` auf Datei **und** `sync_dir` auf das Verzeichnis machen Änderungen durabel (POSIX-Semantik laut `Fs`-Vertrag).
- AN-2 Ein einzelner Schreibvorgang kann zerreißen (Torn Write, Sektorgranularität 512 B/4 KiB). Alle Strukturen sind prüfsummengeschützt.
- AN-3 Ein fehlgeschlagener `fsync` darf nicht wiederholt werden (F-3).
- AN-4 `rename` innerhalb eines Verzeichnisses ist atomar.
- AN-5 Prozessspeicher ist vertrauenswürdig (Klartext in Memtable und Cache).
- AN-6 `KeyProvider::destroy_clusters` ist durabel, wenn es zurückkehrt, und nicht umkehrbar.
- AN-7 Der Speicher liefert für nie beschriebene, vorallokierte Bereiche Nullbytes (`FsCaps::prealloc_zeroed`); sonst wird beim Anlegen explizit mit Nullen gefüllt (§7.4).

---

## 4. Architektur

### 4.1 Crates und Schichten

```
contextra-storage-ports   Traits und Werttypen, keine Logik, keine Laufzeitabhängigkeiten
        ▲
contextra-storage         Kern; forbid(unsafe_code); nur std + ports + bewilligte Abhängigkeiten (§4.5)
        ▲ (nur dev-dependency)
contextra-crypto-ref      Referenzimplementierung der Krypto-Ports (Tests, Erstbetrieb)
```

Der Kern hängt von `ports` ab, nie umgekehrt. Plattformnahes (Direkt-E/A, `mlock`, Hardware-CRC, `io_uring`) ist ausschließlich eine **Port-Implementierung** und berührt den Kern nicht.

### 4.2 Modulschnitt von `contextra-storage` (Abhängigkeiten nur nach unten, Test `layering`)

```
api        Facade: Storage, Space, View, Cursor, Page, Change, Guard, Hold, Consumer, Eraser, Inspector
engine     committer · memtable · version · views · registry · cluster_table · claims
maint      flush · compaction · placement · purge · scheduler (Governor)
erase      protocol (ER0…ER7) · ledger · restriction · oracle (nur Feature `testing`)
recovery   open-Sequenz
catalog    edits · snapshot · rollover · current · reserve
store      run · blob · segment · redo
keys       key_cache · derive · seal
codec      varint · tags · checksums
env        Fs · Clock · Entropy · Governor · Metrics   (Ports; MemFs, RealFs, FaultFs, Sim)
sync       Ranked-Sperren, Epochen
```

`std::fs` kommt nur in `env/real_fs.rs` vor. `std::sync::Mutex` nur über `sync::Ranked` (P-9).

### 4.3 Ports (stabil, semver, in `contextra-storage-ports`)

| Port | Aufgabe | Vertrag |
|---|---|---|
| `Fs` | Dateien, Verzeichnisse, `read_at`, `write_at`, `append`, `preallocate`, `sync_data`, `sync_all`, `sync_dir`, `rename`, `remove`, `list`, `lock`, optional `link`, `clone_file` (Reflink), `caps() -> FsCaps` | Fehlerklassen nach §16; `sync_*` nach POSIX; keine Annahme über globalen Zustand |
| `Clock` | monotone Zeit, Wanduhr | injizierbar (Simulation) |
| `Entropy` | kryptografischer Zufall | injizierbar |
| `KeyProvider` | KEK, Clusterschlüssel, Zerstörung, Probe | §14.2 |
| `Cipher` / `Mac` / `Kdf` / `Checksum` | AEAD, Kettenprüfung, Ableitung, CRC32C | §14.1 |
| `Compressor` | optional Block-Kompression (vor Versiegelung) | Feature-Bit; Standard aus (O-8) |
| `Metrics` | Zähler, Histogramme, Gauges | nicht blockierend |
| `Observer` | Ereignisse (Flush, Purge-Frist, Poison) | nicht blockierend, nie unter Sperren aufgerufen |
| `AccessSink` | optional: Lesezugriffe als `(space, cluster, Blockzahl)` ohne Schlüssel und ohne Inhalt | nicht blockierend; Grundlage für Platzierungsentscheidungen der Aufrufer |
| `Spawner` | Threads der Wartung | der Kern besitzt **keinen** globalen Pool |

### 4.4 Threading-Modell

- **Ein Committer** serialisiert alle Schreibvorgänge (Flat Combining, Gruppen-Fsync, §9.1).
- **Lesepfad:** lock-frei bis auf kurze `Arc`-Klone; eine `View` hält `Arc<Version>`.
- **Wartung:** Lanes `urgent` (Löschfolgearbeit), `flush`, `compact`, `idle`. Flush wartet nie hinter Compaction (SD-10).
- **Alle Threads gehören dem Aufrufer** oder dem übergebenen `Spawner`.

### 4.5 Bewilligte Abhängigkeiten (SD-03)

| Zweck | Wahl | Begründung |
|---|---|---|
| Memtable-Struktur | `crossbeam-skiplist` hinter Trait `Memtable` | lock-freies Lesen ohne eigenes `unsafe`; Austausch möglich |
| Puffer | `bytes` | Zero-Copy |
| Hash/MAC/KDF | `blake3` | schnell; in `crypto-ref` |
| AEAD | `chacha20poly1305` (XChaCha20) | Zufallsnonce ohne Zählerzustand |
| Schlüsselmaterial | `zeroize` | Löschen im Speicher |
| Test | `loom`, `shuttle`, `proptest` (dev) | §18 |

Jede weitere Abhängigkeit braucht einen Registereintrag und `cargo-deny` (`advisories`, `bans`, `sources`).

### 4.6 Codierregeln

- **P-1** Nur kanonische Kodierungen sind gültig (§7.1).
- **P-2** Jeder Wechsel einer Datei zwischen „unreferenziert“ und „lebendig“ ist **ein** Catalog-Edit.
- **P-3** Keine Allokation aus unvertrauten Längenfeldern ohne Plausibilitätsgrenze (P-4).
- **P-4** Alle Längen werden vor Allokation gegen Grenzen aus §5.5 geprüft.
- **P-5** Keine Panik: `unwrap`, `expect`, `panic!`, `todo!`, `unreachable!`, Indexzugriff sind per Workspace-Lint verboten.
- **P-6** Fsync-Fehler werden nie wiederholt.
- **P-7** Prosa und Rustdoc deutsch, Bezeichner englisch.
- **P-8** Keine Fremd- oder Produktnamen im Crate (Bezeichner, Doku, Tests, Metriknamen).
- **P-9** Sperren nur über `sync::Ranked` (§15.2). Kein Rückruf (`Observer`, `SegmentOwner`) unter Sperren.
- **P-10** Reservierte Felder werden mit 0 geschrieben und beim Lesen auf 0 geprüft, sonst `Unsupported`.

---

## 5. Datenmodell

### 5.1 Vokabular

| Begriff | Bedeutung |
|---|---|
| **Storage** | Handle einer geöffneten Instanz |
| **Space** | Benannter, physisch getrennter LSM-Baum (`SpaceId(u16)`) mit eigenem Verzeichnis; Optionen unveränderlich nach dem Anlegen |
| **Cluster** | Löschdomäne, Schlüsseldomäne **und** Platzierungseinheit (`ClusterId`, 16 Byte, opak). `ClusterId::NONE` (nur Nullbytes) heißt „kein Cluster“ |
| **Change / Guard / Receipt** | Atomare Menge von Schreibvorgängen / Vorbedingung / Quittung `(stamp, durable)` |
| **Stamp** | Monotone Zählung angenommener Änderungen (`u64` < 2^56) |
| **View / Cursor / Page** | Unveränderliche Sicht auf einen Stamp / Streaming-Iterator / budgetierte Ergebnisseite mit `ResumeToken` |
| **Hold** | Befristete **technische** Halteregel gegen das Verwerfen alter Versionen, mit Eigentümer |
| **Restriction** | **Rechtlicher** Sperrvermerk auf einem Cluster mit Pflichtgrund; verhindert Löschung durch den Kern (§13.5) |
| **Consumer** | Benannter persistenter Fortschrittszeiger; zählt wie ein Hold, mit Maximalrückstand |
| **Header / Body / Weights / Class** | Erste `header_len` Wertbytes / Rest / zwei `u32`-Zählgrößen je Eintrag / Klassenbyte 0…63 |
| **Redo-Log** | Segmentiertes Wiederholungsprotokoll aus sektorausgerichteten Gruppen |
| **Run / Blob** | Unveränderliche sortierte Datei / unveränderliche Datei mit Körpern |
| **Cluster-Segment** | Zusammenhängender, in sich versiegelter Bereich eines Runs, der genau einen Cluster enthält (§7.5) |
| **Pure / Packed** | Run mit genau einem Cluster / Run mit mehreren Clustern (§11.3) |
| **Retired / Purged** | Cluster, dessen Löschung logisch erfolgt ist (Stamp `S`) / dessen Dateien physisch bereinigt sind |
| **Segment** | Vom Catalog verwaltete Datei eines Aufrufers mit Kern-Rahmen (§20.2) |
| **Claim / Graveyard** | Exklusive Reservierung von Dateien durch einen Wartungsjob / Warteschlange zu löschender Dateien |
| **Catalog / Ledger** | Einzige Wahrheit über lebendige Dateien und Zustand / unabhängiges Lösch-Protokoll |
| **Env** | Bündel der Ports |

### 5.2 Schlüssel, Ordnung, Stamp

- **Nutzerschlüssel:** beliebige Bytes, 1 ≤ Länge ≤ `max_key_bytes`. Er MUSS frei von personenbezogenen Daten sein (Vertrag P-K, §13.8); Kennungen sind opak.
- **Interner Schlüssel** = `cluster ‖ user_key ‖ tag`, `tag: u64 = (stamp << 8) | kind`. Sortierung: **Cluster aufsteigend (bytelexikografisch), Nutzerschlüssel aufsteigend, Tag absteigend** (neueste Version zuerst). In Spaces ohne `clustered` ist der Cluster implizit `NONE` und wird nicht gespeichert.
- `kind`: `Put = 1`, `Delete = 2`, `RangeDelete = 3`, `EraseRange = 4`, `Merge = 6` (**reserviert**, Decoder meldet `Unsupported`). `5` ist frei und bleibt unbelegt.
- **Stamp:** erster Stamp ist 1, Wert 0 heißt „leerer Zustand“. Streng monoton über **angenommene** Changes (`INV-STAMP-1`). Überlauf ⇒ `Error::Exhausted`, nie Umbruch (`INV-STAMP-2`). **Kein Stamp wird je zwei verschiedenen Änderungen zugeordnet, auch nicht über einen Absturz** (`INV-STAMP-3`, §9.2). Lücken sind zulässig.
- **Bereichsmarker** (`RangeDelete`, `EraseRange`): `(cluster, start, end, tag)`, `end` exklusiv. Flag `UNBOUNDED_END` = bis zum Ende des Clusters, Flag `WHOLE_CLUSTER` = alle Schlüssel des Clusters. Ein Marker mit Stamp `S` verdeckt für jede Sicht ≥ `S` alle Versionen mit Stamp < `S` im Bereich. `EraseRange` verlangt zusätzlich physisches Entfernen (§13.7).

### 5.3 Cluster

- Ein `ClusterId` ist **opak und nie aus einer Person ableitbar** (`INV-PRIV-1`). Empfohlen: 128 Bit Zufall oder keyed Hash außerhalb des Kerns. Der Kern prüft das nicht.
- **Die Ordnung der `ClusterId` ist die einzige Platzierungsachse.** Aufrufer, die Cluster gemeinsam löschen oder verfallen lassen, vergeben Kennungen mit gemeinsamem Präfix (zum Beispiel `Gruppe[8] ‖ Untereinheit[8]`); sie liegen dann benachbart, teilen Packed-Dateien und werden mit **einer** `retire`-Operation atomar gelöscht (§13.3).
- **Zustände** (im Catalog nur, wenn nicht `Live`; ein `Live`-Cluster ist implizit): `Live` → `Restricted{reason, blocks_reads}` ⇄ `Live`; `Live|Restricted` ⇒ `Retired{S, level}` → `Purged{S}`. Nur vorwärts, kein Zurück aus `Retired`.
- Ein Cluster, der `Retired` ist, nimmt keine Schreibvorgänge mehr an (`Err(ClusterRetired)`), und jedes Lesen liefert `Err(Erased)`.
- **Mehrfachbezug** (ein Eintrag gehört mehreren Betroffenen) kennt der Kern nicht. Der Aufrufer legt dazu den Eintrag je Cluster getrennt ab oder trennt die Anteile (§20.3). Das ist eine bewusste Vereinfachung gegenüber R3 (SD-02).

### 5.4 Wert, Kopf, Körper, Gewichte

Ein Eintrag besteht aus: `header` (Bytes, ≤ `max_header_bytes`), `body` (Bytes), `weights: [u32; 2]` (Zählgrößen, Bedeutung legt der Aufrufer fest), `class: u8` (0…63, opak). `header_len` steht im Eintrag. Der Kern garantiert, dass `get_header` und Header-Scans **keinen Körper-E/A** auslösen (Z-7). Der Kern summiert `weights` und bildet `class`-Masken je Block (§7.5), sonst interpretiert er nichts.

### 5.5 Grenzen (Standardwerte, Konfiguration begrenzt durch harte Obergrenzen)

| Größe | Standard | Harte Grenze |
|---|---|---|
| `max_key_bytes` | 1 KiB | 64 KiB |
| `max_header_bytes` | 4 KiB | 64 KiB |
| `max_value_bytes` (Header+Körper) | 16 MiB | 16 MiB |
| Change gesamt | 32 MiB | 32 MiB |
| Operationen je Change | 4096 | 65 535 |
| Redo-Frame | ≤ 32 MiB | 32 MiB |
| Redo-Segment | 64 MiB | 1 GiB |
| Datenblock | 16 KiB Ziel | 1 MiB |
| Run-Zieldatei | 64 MiB | 1 GiB |
| `segment_id` | < 2^32 | 2^32 |
| Spaces je Instanz | 64 | 1024 |

---

## 6. Öffentliche API (normativ; Lifetimes und Nebensächlichkeiten offen)

```rust
// --- Öffnen und Struktur ------------------------------------------------------
impl Storage {
    pub fn open(cfg: Config, env: Env) -> Result<Storage, OpenError>;          // Recovery inklusive (§12)
    pub fn space(&self, name: &str) -> Result<Space, Error>;
    pub fn add_space(&self, name: &str, opts: SpaceOptions) -> Result<Space, Error>;   // atomar (Catalog)
    pub fn close(self) -> Result<CloseReport, Error>;
}
pub struct SpaceOptions {            // unveränderlich nach dem Anlegen
    pub clustered: bool,             // true: jede Op trägt eine ClusterId ≠ NONE
    pub value_separation_min: u32,   // Körper ≥ Schwelle → Blob (nur in Pure-Runs, §11.3); Standard 4 KiB
    pub history: HistoryPolicy,      // KeepLatest | KeepUntilFloor
    pub max_key_bytes: u32, pub max_header_bytes: u32,
    pub durability_default: Durability,
}

// --- Schreiben: eine Change über viele Spaces und Cluster, atomar -------------
pub struct Change { /* Ops + Guards, kein Zustand außerhalb */ }
impl Change {
    pub fn new() -> Change;
    pub fn put(&mut self, sp: &Space, c: ClusterId, key: &[u8], e: NewEntry) -> &mut Self;
    pub fn delete(&mut self, sp: &Space, c: ClusterId, key: &[u8]) -> &mut Self;
    pub fn delete_range(&mut self, sp: &Space, c: ClusterId, r: KeyRange) -> &mut Self;
    pub fn guard(&mut self, sp: &Space, c: ClusterId, key: &[u8], g: Precondition) -> &mut Self;
}                                    // Precondition: Absent | Present | Version(Stamp) | Unchanged(Stamp)
pub struct NewEntry { pub header: Bytes, pub body: Bytes, pub weights: [u32; 2], pub class: u8 }
pub enum Durability { Durable, Volatile }      // Volatile: sichtbar vor Fsync, bei Absturz verlierbar
impl Storage {
    pub fn commit(&self, c: Change, d: Durability) -> Result<Receipt, CommitError>;
}   // Receipt { stamp: Stamp, durable: bool }
    // CommitError::{ GuardFailed{index}, ClusterRetired{cluster}, Restricted{cluster}, TooLarge, Backpressure(RetryAfter),
    //               Poisoned, Indeterminate{..} /* Ausgang unbekannt, §16 */, Exhausted, Invalid(..) }

// --- Lesen --------------------------------------------------------------------
impl Storage {
    pub fn view(&self) -> View;                                    // aktueller sichtbarer Stamp
    pub fn view_durable(&self) -> View;                            // nur Stände ≤ durable_stamp
    pub fn view_at(&self, s: Stamp) -> Result<View, ViewError>;    // ≥ floor, sonst TooOld
}
impl View {
    pub fn stamp(&self) -> Stamp;
    pub fn get_header(&self, sp: &Space, c: ClusterId, key: &[u8]) -> Result<Option<HeaderRef>, ReadError>;
    pub fn get(&self, sp: &Space, c: ClusterId, key: &[u8]) -> Result<Option<EntryRef>, ReadError>;   // Körper lazy
    pub fn cursor(&self, sp: &Space, c: ClusterId, r: KeyRange, o: ScanOpts) -> Result<Cursor, ReadError>;
    pub fn page(&self, sp: &Space, c: ClusterId, r: KeyRange, spec: PageSpec,
                resume: Option<ResumeToken>) -> Result<Page, ReadError>;
    pub fn history(&self, sp: &Space, c: ClusterId, key: &[u8]) -> Result<History, ReadError>;
}
pub struct PageSpec { pub max_items: u32, pub max_bytes: u64, pub max_weight: [Option<u64>; 2],
                      pub class_mask: u64, pub header_only: bool }
pub struct Page { pub items: Vec<PageItem>, pub next: Option<ResumeToken>, pub oversize_first: bool }
impl EntryRef { pub fn header(&self) -> &[u8]; pub fn weights(&self) -> [u32; 2]; pub fn class(&self) -> u8;
                pub fn body(&self) -> Result<Bytes, BodyError>; }   // Err(Erased) nie Daten, nie leer
// Eine View kann ablaufen (view_max_age, Standard 15 min): danach Err(ViewExpired). Cursor und Page erben das.

// --- Holds, Consumer ------------------------------------------------------------
impl Storage {
    pub fn hold(&self, s: Stamp, owner: OwnerTag, ttl: Duration) -> Result<Hold, Error>;
    pub fn consumer(&self, id: ConsumerId, from: Stamp, max_lag: Lag) -> Result<Consumer, Error>;
}
impl Consumer { pub fn next_batch(&mut self, max: usize, f: &ClusterFilter) -> Result<Changes, ConsumerError>;
                pub fn ack(&mut self, upto: Stamp) -> Result<(), Error>; }   // ack durabel
// ConsumerError::Lagged{resume_hint}: Rückstand über max_lag, Hold aufgehoben, Neuaufbau durch den Aufrufer nötig.

// --- Löschung (§13) ---------------------------------------------------------------
impl Storage { pub fn eraser(&self) -> Eraser; pub fn restrict(&self, c: ClusterId, r: Restriction) -> Result<(), Error>;
               pub fn release(&self, c: ClusterId, reason: ReasonCode) -> Result<(), Error>; }
impl Eraser {
    pub fn retire(&self, clusters: &[ClusterId], req: EraseReq) -> Result<EraseTicket, EraseError>;   // alle oder keiner
    pub fn erase_range(&self, sp: &Space, c: ClusterId, r: KeyRange, req: EraseReq) -> Result<EraseTicket, EraseError>;
    pub fn status(&self, t: &EraseTicket) -> EraseStatus;      // Retired · KeyDestroyed · Purging{phase} · Done(Receipt)
    pub fn wait(&self, t: &EraseTicket, until: EraseStage, timeout: Duration) -> Result<EraseStatus, Error>;
    pub fn verify(&self, t: &EraseTicket) -> VerifyReport;
}
pub struct EraseReq { pub level: EraseLevel,   // Logical | Cryptographic | Physical | Scrubbed
                      pub views: ViewPolicy,   // Wait(Duration) | Revoke
                      pub purge_deadline: Option<Duration> }   // nur Cryptographic (§13.4)

// --- Checkpoint, Betrieb, Prüfung ---------------------------------------------------
impl Storage {
    pub fn checkpoint(&self, dst: &Path, m: CheckpointMode) -> Result<CheckpointManifest, Error>;  // Copy | Reflink | Hardlink(Opt-in)
    pub fn open_restored(cfg: Config, env: Env, ledger: LedgerSource) -> Result<Storage, OpenError>;
    pub fn maintenance(&self) -> Maintenance;   // Pause/Resume, Zustand, Governor
    pub fn inspect(&self) -> Inspector;         // Dateien, Cluster-Statistik, lint_keys, Kennzahlen
}
```

**Eigenschaften**

- Keine Methode blockiert **unbegrenzt**: jede Wartefunktion hat Timeout; Rückdruck ist `Backpressure(RetryAfter)`.
- Fehler sind aufzählbar (`Error::class()`); jeder trägt `hint()` mit Handlungstext. `hint()` enthält **nie** Schlüssel, Kennungen oder Bereichswerte; strukturierte Felder (zum Beispiel `Erased{cluster, at}`) dürfen opake Kennungen tragen.
- Der Kern prüft **keine Berechtigung**. `Erased` und „nicht vorhanden“ sind verschiedene Fehler; die Abbildung auf Antworten gegenüber Nutzern ist Sache des Aufrufers (L-12).
- `View`, `Cursor`, `Page` sind `Send`, `Clone` ist billig, keine Sperre wird gehalten.
- Es gibt **keine** Idempotenz-Token im Kern. Wiederholung nach `Indeterminate` erfolgt über `Guard` (`Absent`, `Version`); das hält die Quittungsstruktur frei von Zustand, der Löschung berühren würde (SD-05).
- **ResumeToken:** opake Bytes mit `(view_stamp, space, cluster, letzter interner Schlüssel, spec_hash)` und Prüfwert (CRC32C in `Plain`, Mac16 sonst). Ein Token ist nur gültig, solange `view_stamp ≥ floor`; sonst `Err(TooOld)`. Ein Token kann den Cluster nicht wechseln.

---

## 7. Plattenformate (Format v1; Freeze nach Gate G-F)

### 7.1 Konventionen

- Alle Zahlen **little-endian**. `varint` = LEB128 für `u64` (höchstens 10 Bytes), **nur kanonisch** (kein überflüssiges `0x80…0x00`, zehntes Byte nur `0x01`); alles andere ist `Corrupt` (`INV-FMT-1`).
- Jede Dateiart hat **genau eine Version (`1`)** und Feature-Bits (SD-04). Unbekannte Version oder unbekanntes Pflicht-Bit ⇒ `Unsupported`; unbekannte Catalog-Tags ⇒ `Unsupported`, nie Ignorieren.
- **Integritätsmodus** steht in `FORMAT` und ist unveränderlich: `Plain` (CRC32C), `Authenticated` (Mac16, BLAKE3-keyed, 16 Byte), `Encrypted` (AEAD, XChaCha20-Poly1305, Tag 16 Byte). Header und Footer tragen in allen Modi CRC32C; in `Authenticated|Encrypted` zusätzlich Mac16 über Header/Footer. `FORMAT` trägt in diesen Modi ein `Mac16` (§12.1 Reihenfolge).
- Jede Datei beginnt mit `magic[8] ‖ file_kind u8 ‖ format_ver u16 ‖ features u32 ‖ file_id u64 ‖ instance_id[16] ‖ header_crc u32`.

### 7.2 FORMAT und KEYS

`FORMAT` (einmal geschrieben): Magic, `format_ver`, Feature-Bits, `integrity_mode`, `algorithm_ids` (AEAD, MAC, KDF), `instance_id[16]`, `chk` (CRC32C; Mac16 in `Authenticated|Encrypted`).

`KEYS` (nur `Authenticated|Encrypted`, atomar ersetzt): `root_key[32]`, mit der KEK des `KeyProvider` umhüllt (AEAD), `key_gen u32`, `chk`. Der Kern speichert **keine** Clusterschlüssel (§14.2).

### 7.3 Catalog (einzige Wahrheit)

`Edit := { seq u64, ops[] , chk }`; jeder Edit atomar durch einen Append + `sync_data`. Rollover bei Größe (`catalog_roll_bytes`, 8 MiB): neue Datei beginnt mit einem **Snapshot-Record** des vollständigen Zustands; `CURRENT` zeigt per atomarem `rename` darauf. Edits tragen eine Kette (`prev_chk`); ein Torn Tail am Dateiende wird nach §12.1 R3 behandelt.

Ops: `space_add | space_drop | file_add{kind, id, space, level, min/max stamp, min/max key, pure_cluster?, footer_chk} | file_remove{id} | blob_ref_note | redo_floor{segment} | redo_mark{segment, offset, stamp, chain_hash} | stamp_limit{limit} | id_limit{limit} | cluster_restrict{cluster, reason, blocks_reads} | cluster_release{cluster} | cluster_retire{clusters[], S, level} | cluster_keys_destroyed{S} | cluster_purged{clusters[], S} | erase_pending_add{S, clusters[], level, deadline_wall?} | erase_phys{S} | erase_done{S} | erased_range_add{cluster?, range, S} | erased_range_drop{S} | hold_floor{stamp} | consumer_set{id, ack, max_lag} | segment_add{kind, id, cover} | segment_remove{id} | checkpoint_epoch{n}`.

Regeln:

- Eine Datei ist **lebendig genau dann**, wenn ein durabler `file_add` oder `segment_add` ohne späteres `file_remove`/`segment_remove` existiert (`INV-CAT-1`).
- **Hochwassermarken** (`stamp_limit`, `id_limit`) werden **vor** der Nutzung durabel angehoben (INV-KEY-3 für IDs, INV-STAMP-3 für Stamps). Blockgröße: `stamp_reserve_block` = 65 536, `id_reserve_block` = 4096.
- `tmp_id` für `.building.<tmp_id>`-Dateien stammt aus dem reservierten ID-Bereich, nie aus der Prozess-ID.

### 7.4 Redo-Log

**Segment** (`redo/redo-<segment_id:016x>.log`, `segment_id < 2^32`, **nie wiederverwendet**, `INV-LOG-2`): vorallokiert (`Fs::preallocate`) auf `redo_segment_bytes`, Inhalt Nullbytes. Wenn `FsCaps::prealloc_zeroed` falsch ist, schreibt der Kern beim Anlegen explizit Nullen und nutzt `sync_all` statt `sync_data`.

```
SegmentHeader (4096 B): magic ‖ kind ‖ ver ‖ features ‖ segment_id ‖ instance_id ‖ prev_chain[32] ‖ first_stamp_hint ‖ crc/mac
Group*   (jede Gruppe beginnt an einer 4096-B-Grenze, Rest bis zur Grenze Nullbytes)
SegmentFooter (beim Siegeln): magic ‖ last_stamp ‖ n_groups ‖ chain_hash ‖ crc/mac     (Siegel = Footer schreiben + sync)
```

**Gruppe** (eine je Fsync-Gruppe des Committers):

```
group_head: gmagic[8] ‖ segment_id u32 ‖ group_seq u64 ‖ stamp_first u64 ‖ stamp_last u64 ‖ n_frames u32 ‖ payload_len u32 ‖ flags u32
            ‖ chk[16]            Plain: CRC32C über head‖payload;  Auth/Enc: Mac16 über head‖payload‖prev_chk   (Kette)
payload:    Frame*
```

**Frame** (eine `Change` oder ein Erase):

```
frame_head (immer im Klartext, von chk der Gruppe gedeckt):
    kind u8 (Commit=1, Erase=2, Control=3) | stamp u64 | n_sections u16
    section*: { space u16 | cluster[16] | n_ops u16 | payload_len u32 }
section_payload: Op*   Op := kind u8 | key_len varint | key | tag_kind ... | header_len varint | weights[2] | class | value
    Plain/Authenticated: Klartext.   Encrypted: je Sektion ein AEAD-Block unter Einmalschlüssel aus dem Clusterschlüssel (§14.3).
Erase-Frame:   kind=2 | stamp S | n u16 | cluster[16]* | level u8 | flags | [range]   (keine Nutzlast)
Control-Frame: kind=3 | stamp | op u8 (Restrict=1, Release=2) | cluster[16] | reason u32 | blocks_reads u8
```

Folgen:

- Stamp, Space, Cluster und Längen stehen im Klartext: Recovery und Flush-Barriere **planen ohne Schlüssel** (§12). Nach Zerstörung eines Clusterschlüssels bleibt die Sektion unlesbar; das Replay erzeugt keinen Zustand für sie.
- **Segmentschnitt:** Ein Erase-Frame ist immer das **erste Frame** eines neuen Segments (`INV-LOG-4`); das vorige Segment wird vorher gesiegelt.
- **Gruppenanzahl und Fsync:** der Committer schreibt jede Gruppe mit einem `write_at`; ein `sync_data` je Gruppe (nur wenn eine `Durable`-Change enthalten ist).
- **redo_mark:** Der Kern schreibt periodisch (`redo_mark_bytes`, 8 MiB) und bei jedem Siegel einen Catalog-Edit `redo_mark`. Alles bis zur Marke MUSS prüfbar gültig sein (`INV-LOG-7`).
- **Reserve:** Es existiert stets ein vorallokiertes Reservesegment (`INV-LOG-6`); es gilt wie jedes Segment mit `segment_id ≥ redo_floor` als lebendig.

### 7.5 Run

```
Run := RunHeader
       ClusterSegment*            (aufsteigend nach Cluster; in Spaces ohne `clustered` genau eines für NONE)
       ClusterDirectory           (Cluster → Segmentoffset/-länge, Einträge, Bytes, min/max Stamp, key_epoch (=1 in v1))
       MetaBlock                  (space, level, min/max interner Schlüssel, min/max Stamp, Eintragszahl, pure_cluster?)
       Footer                     (Offsets, footer_chk; im Catalog als `footer_chk` hinterlegt)

ClusterSegment := Block*  SegIndex  [RangeMarkerBlock]
SegIndex       := je Block: letzter Schlüssel (kürzester Separator), Offset, Länge, Einträge, weights_sum[2], class_mask u64, min/max Stamp
                  [Filter: nur bei ≥ 256 Einträgen]     segment_uid u64
```

- **Cluster-Reinheit (`INV-PLACE-1`):** Jeder Block gehört genau einem Cluster. Kein Block enthält Einträge zweier Cluster.
- **Versiegelung je Cluster-Segment (Modus `Encrypted`):** Blöcke, `SegIndex`, Filter und Markerblock werden unter dem aus dem Clusterschlüssel abgeleiteten Schlüssel versiegelt (§14.3). **Schlüssel, Kopfdaten und Körper eines Clusters sind damit nach Schlüsselzerstörung unlesbar**, nicht nur die Körper. Im Klartext bleiben nur `ClusterDirectory`, `MetaBlock`, Footer: Cluster-IDs (opak), Größen, Stamps (Restexposition, §13.8).
- **Verschiebbar ohne Schlüssel (`INV-PLACE-4`):** Die Zusatzdaten (AAD) eines Blocks sind `(space, cluster, segment_uid, block_no)`, nicht Dateiname oder Offset. Ein Cluster-Segment kann daher **byte-identisch** in einen anderen Run kopiert werden, ohne zu entschlüsseln. Das ist die Grundlage des billigen Umschreibens (§13.4) und der Cluster-Trennung bei Compaction (§11.3).
- **Block:** Zielgröße 16 KiB; Präfixkompression mit Restart-Punkten (Abstand 16); optional `Compressor` vor Versiegelung. Entry-Kodierung:

```
entry := varint(shared) varint(unshared) | unshared_key | tag u64
         flags u8 (has_body, ref_body) | class u8 | weights[2] varint | header_len varint | header
         inline:  body_len varint | body            ref:  blob_id u64 | blob_off u64 | body_len u32 | body_tag[16]
```

- **PutRef-Körper** liegen **nur in Pure-Runs** (§11.3); `Packed`-Runs enthalten Körper inline (bis `max_inline_bytes`, Standard 1 MiB, größere erzwingen Pure).
- **Filter:** Ribbon oder Bloom über `hash_keyed(space_filter_key, user_key)`. Der Filterschlüssel ist je Segment aus dem Clusterschlüssel abgeleitet, damit ein Segment verschiebbar und beim Verwerfen restlos weg ist.

### 7.6 Blob

```
BlobFile := header | Record* | footer
Record   := len u32 | nonce[24] | sealed_body | tag[16]            (Plain: len | body | crc32c)
```

- Eine Blob-Datei gehört zu **genau einem Cluster** in genau einem Space (`INV-PLACE-2`). Zielgröße ≤ 64 MiB.
- AAD eines Records: `(space, cluster, key_digest[16], tag_of_entry)`. Sie nennt **keinen Ort**; ein Record ist deshalb byte-identisch in eine neue Blob-Datei kopierbar (Blob-GC, §11.5). `key_digest` ist ein keyed Hash des Nutzerschlüssels (Filterschlüssel-Familie) und ermöglicht die Bindung an die Version ohne Klartextschlüssel in der Datei.
- Blobs werden **nie verändert**; GC erzeugt eine neue Datei mit neuer `blob_id`. Die Verweise in den Runs desselben Clusters werden **im selben Edit** umgeschrieben (§11.5). Es gibt keine Generationen, keine Hole-Records, keine Indirektion.
- Blob-Records sind **nicht** zusätzlich blockversiegelt: Jede Darstellung trägt **genau eine** Versiegelungsschicht (§14.3).

### 7.7 Ledger

`ERASE-LEDGER/ledger-<n>.led`, append-only, je Eintrag `{seq, S, stage, level, cluster_count, clusters_digest, wall_ts, prev_hash}` mit BLAKE3-Kette. Stages: `Retired`, `KeyDestroyed`, `Physical`, `Done`, `Restricted`, `Released`. **Keine Inhalte und keine Nutzerschlüssel.** Cluster-IDs stehen als geordnete Liste in einer Anlage (`.clusters`-Datei je Eintrag, hash-gebunden), damit ein Restore sie anwenden kann. Signieren und Berichte sind Sache des Aufrufers; der Kern liefert **kanonische Bytes**. Das Ledger liegt nie im Checkpoint (§13.9).

### 7.8 Segment-Rahmen (Erweiterungsdateien)

`segments/<kind:04x>/<id:016x>.seg`: Kern-Rahmen `magic ‖ kind u16 ‖ owner_ver u16 ‖ cover (Exclusive cluster | Mixed liste) ‖ payload_len ‖ chk`, Nutzlast opak. Der Kern bietet dem Besitzer `SegmentIo::seal(cluster, bytes)`/`open`, damit auch Segmentinhalte unter dem Clusterschlüssel stehen (§20.2).

### 7.9 Freeze-Liste (Gate G-F)

Eingefroren werden: Datei-Kopf, `FORMAT`/`KEYS`, Gruppen- und Frame-Kopf samt Kette, Erase-Frame, Run (Directory, SegIndex, Block, Entry-Kodierung), Blob-Record, Catalog-Op-Tags (mit **reserviertem Bereich** `0xF000…0xFFFF`), Ledger, Segment-Rahmen, Nonce- und AAD-Regeln. Zu jedem Format gibt es Referenzvektoren aus dem unabhängigen Referenzcodierer (Anhang A).

---

## 8. Dateilayout und Dateilebenszyklus

### 8.1 Layout

```
<root>/
  FORMAT   LOCK   CURRENT   CATALOG-000007   KEYS
  redo/redo-<id>.log
  spaces/<space:04x>/
     runs/<id:016x>.run          Packed und Pure gemischt (Pure steht im Catalog)
     blobs/<cluster-prefix>/<id:016x>.blob     (Blobs sind clusterrein)
  segments/<kind:04x>/<id:016x>.seg
  ERASE-LEDGER/                  (Pfad konfigurierbar, §13.9)
```

`LOCK`: exklusive Sperre über `Fs::lock` (oder geteilte, read-only). Alle Dateien und Verzeichnisse entstehen mit `Perm::Private`.

### 8.2 Regeln

- **Numerische IDs** benennen Dateien, nie Pfad-Strings im Catalog.
- **Temporäre Dateien** tragen `.building.<tmp_id>`; beim Öffnen werden alle entfernt.
- **Verwaiste Dateien** (Run, Blob, Segment, Redo unter `redo_floor`), die der Catalog nicht nennt, werden beim Öffnen **nach** der Wiederaufnahme offener Löschungen (§12.1 R7→R8) und danach periodisch gelöscht. Das ist sicher, weil nur ein Catalog-Edit eine Datei lebendig macht.
- **Dateien sind unveränderlich**, sobald sie lebendig sind (Ausnahme: Redo-Segment bis zum Siegel). Kein Kern-Code schreibt in eine lebendige Run-, Blob- oder Segment-Datei.
- **Ein Space = ein Verzeichnis.** `drop_space` ist ein Edit plus Entfernen aller Dateien plus `remove_dir`.

### 8.3 Zustandsdiagramm einer Datei

```
(nicht vorhanden) ─create .building─► im Bau ─sync_all + rename + sync_dir─► dauerhaft, UNREFERENZIERT
      ▲                                                                           │ Edit(add) durabel
      │ Verwaisten-Suche                                                          ▼
      └──── Unlink + sync_dir ◄── Graveyard ◄── obsolet ◄─────────────────── LEBENDIG
            (nach letzter Version-Referenz)          ▲ Edit(remove) durabel
```

Eine Datei wechselt nur in Pfeilrichtung (P-2).

---

## 9. Schreibpfad

### 9.1 Ablauf von `commit` (Flat Combining, ein Committer)

```
1  Vorprüfung (ohne Sperre): Größen, Spaces, Clusterzustand (Retired ⇒ ClusterRetired, Restriction.blocks_reads berührt Schreiben nicht),
   Backpressure-Stufe (§9.6)
2  Vorbereiten auf dem Aufruferthread (parallel, ohne Sperre): Ops kodieren; im Modus Encrypted je (space, cluster) die Sektion
   mit Einmalschlüssel versiegeln (§14.3, Schicht R). Clusterschlüssel anlegen, falls neu (`ensure_cluster_key`, idempotent).
3  Stamp-Reserve prüfen: reicht `stamp_limit − next_stamp` nicht für die Gruppe, vorher `reserve_stamps()` (Catalog-Edit),
   **bevor** der Committer-Rang genommen wird (§9.2, §15.2).
4  Eintragen in den Intake; Warten auf den Gruppenführer.
5  Gruppenführer (hält `committer`): nimmt alle wartenden Changes in Ankunftsreihenfolge.
     a) Clusterzustand erneut prüfen (Retired seit Schritt 1 ⇒ diese Change scheidet mit ClusterRetired aus)
     b) Guards prüfen gegen: Memtable ∪ Version ∪ **Gruppen-Overlay** (Schreibvorgänge früherer Changes derselben Gruppe)
        → fehlgeschlagene Changes scheiden mit GuardFailed aus, ohne etwas zu schreiben
     c) Stamps vergeben (aufsteigend, aus dem reservierten Block)
     d) Gruppe bauen: ein Frame je Change, Sektionen je (space, cluster), Kette/MAC
     e) ein `write_at`; ein `sync_data`, wenn mindestens eine Change `Durable` ist   ◄── DURABILITÄTSPUNKT
6  Memtable-Einfügung in Stampreihenfolge; `durable_stamp` anheben (falls synchronisiert); danach `visible_stamp` anheben (Release).
7  Rückkehr an alle Wartenden; Quittung `{stamp, durable}`.
```

- **Sichtbarkeit:** `Durable`-Changes werden erst nach dem Sync sichtbar (Lesen-nach-Durabilität). `Volatile`-Changes werden sichtbar, sobald sie in der Memtable stehen, und sind im Redo nur geschrieben, nicht synchronisiert; jeder spätere Sync deckt sie mit ab.
- **Sync-Fehler:** `sync_data` schlägt fehl ⇒ **Poison** (§16, F-3). Alle Changes der Gruppe erhalten `Indeterminate` (die Gruppe kann nach Neustart vorhanden sein oder nicht), alle späteren `Poisoned`. Lesen bleibt möglich.
- **Memtable-Wechsel** führt **ausschließlich der Committer** an Gruppengrenzen aus (Anforderung durch ein Flag der Wartung). Dadurch gibt es genau eine Schreibquelle für aktive Memtables (L-2).
- **Erase-Frames** laufen durch denselben Committer (§13.3 ER1) und erzwingen den Segmentschnitt (§7.4).

### 9.2 Stamps und Hochwassermarke (`INV-STAMP-3`)

- Der Catalog führt `stamp_limit`: **jeder** Stamp, der je vergeben wurde oder werden darf, ist ≤ `stamp_limit`. Die Vergabe läuft nur im Bereich `(next_stamp, stamp_limit]`.
- Die Wartung hebt `stamp_limit` blockweise an (`stamp_reserve_block`), sobald weniger als die Hälfte des Blocks übrig ist; der Committer-Pfad hat den synchronen Fallback aus Schritt 3.
- **Beim Öffnen:** `next_stamp = max(stamp_limit, letzter gültiger Redo-Stamp, max Stamp aller Runs) + 1`. Nicht genutzte Stamps verfallen; das ist zulässig, weil `INV-STAMP-1` nur über **angenommene** Changes gilt.
- Folge: Ein `Volatile`-Stamp, der durch einen Absturz verloren geht, wird nie an eine andere Change vergeben. Ein Consumer kann deshalb keine neue Änderung mit einer alten Nummer überspringen.
- Consumer und `view_durable` sehen nur Stamps ≤ `durable_stamp` (`INV-CONS-1`).

### 9.3 Memtable

- Je Space eine aktive Memtable in interner Schlüsselordnung (also nach Cluster zusammenhängend), dazu höchstens `max_immutable` (Standard 2) eingefrorene.
- Größen: `memtable_bytes` je Space (Standard 16 MiB) und **`memtable_total_bytes`** über die Instanz (Standard 256 MiB, harte Obergrenze); beide wirken im Rückdruck (§9.6).
- Die Memtable hält **Klartext** (AN-5). Beim ER1 werden die Einträge eines retired Clusters aus aktiver und eingefrorenen Memtables entfernt und der Speicher überschrieben (best effort, §13.8); Flush überspringt retired Cluster.

### 9.4 Flush

Je Space: eingefrorene Memtable → Run(s) → ein Catalog-Edit. Ablauf je Run: `.building` schreiben → `sync_all` → `rename` → `sync_dir`. Danach **ein** Edit `{file_add[], redo_floor}` und Veröffentlichung einer neuen `Version`.

- **Platzierung beim Flush:** Ein Cluster, dessen Anteil in dieser Memtable ≥ `pure_min_bytes` (Standard 8 MiB) ist **oder** dessen Körper > `max_inline_bytes` enthalten, wird als eigener **Pure-Run** (mit Blob, wenn `value_separation_min` greift) geschrieben; alle anderen Cluster zusammen in **einen Packed-Run**, nach Cluster sortiert, **jeder Cluster vollständig in einem Cluster-Segment**.
- `redo_floor` rückt nur auf Segmente vor, deren sämtliche Frames in durablen Runs stehen **und** deren Erase-Frames im Catalog als `erase_pending` stehen (`INV-LOG-5`).
- Ein Absturz vor dem Edit lässt den Run verwaist (wird entfernt); Redo spielt erneut ein.

### 9.5 Redo-Lebenszyklus

`Open → Sealed → Obsolete (< redo_floor) → Graveyard → Unlink`. Der Scheduler zwingt einen Flush, wenn das älteste Segment `redo_max_age` (Standard 1 h) überschreitet; so liegen Klartext-Redo-Daten (Modi `Plain`, `Authenticated`) nicht unbegrenzt.

### 9.6 Rückdruck

| Stufe | Auslöser | Wirkung |
|---|---|---|
| 0 | normal | – |
| 1 | `memtable_total_bytes` ≥ 75 %, L0-Dateien ≥ `l0_slow` (8), Redo-Rückstand ≥ 75 % | `commit` wartet kurz **vor** dem Intake (nie unter Sperre) |
| 2 | `memtable_total_bytes` = 100 %, L0 ≥ `l0_stop` (24), Redo voll | `Backpressure(RetryAfter)` |
| Poison | `ENOSPC`, `EIO`, Fsync-Fehler | `Poisoned`; Lesen bleibt; Wiederanlauf nur über Neustart/Recovery |

Warten **unter** dem Committer-Rang ist verboten.

---

## 10. Lesepfad

1. `view()` lädt **zuerst** `visible_stamp` (Acquire), **danach** den `Arc<Version>` (`INV-VIEW-2`). Die Reihenfolge ist zwingend: jede Version, die nach dem Lesen von `visible_stamp` geladen wird, enthält (in Memtables oder Runs) alle Einträge mit Stamp ≤ `visible_stamp`. Danach hält die View keine Sperre. `view_durable` liest `durable_stamp`.
2. **Zugriffsprüfung zuerst:** Zustand des Clusters aus `cluster_table` (O(1), lock-freier Zähler `retire_gen`). `Retired|Purged` ⇒ `Err(Erased{cluster, at})` für **jede** View. `Restricted{blocks_reads:true}` ⇒ `Err(Restricted)`.
3. **Punktabfrage:** Memtable → eingefrorene Memtables → L0-Runs (nach Cluster-Bereich und Filter) → je Level der eine passende Run. Im Run: `ClusterDirectory` → Cluster-Segment → `SegIndex` → ein Block. Ziel: höchstens ein Block-Read je Level.
4. **Scan/Cursor:** k-Wege-Merge nur über Quellen, deren Cluster-Bereich den Cluster berührt. Header ohne Körper-E/A.
5. **Page:** läuft auf dem Cursor. Sie bricht **vor** dem Eintrag ab, der `max_items`, `max_bytes` oder ein `max_weight` überschreiten würde; der erste Eintrag wird immer geliefert (`oversize_first`), damit kein Fortschrittsstillstand entsteht. `class_mask` überspringt Blöcke, deren aggregierte `class_mask` den Filter vollständig ausschließt; `weights_sum` erlaubt Vorab-Abschätzung ohne Dekodierung.
6. **Körper:** `body()` lädt erst bei Bedarf (Blob-`read_at`), prüft Tag/CRC und entsiegelt.
7. **Nachprüfung (`INV-KEY-2`):** Jede Entsiegelung liest `retire_gen` vorher und nachher; hat sich der Wert geändert, wird der Clusterzustand neu geprüft und bei `Retired` der Klartext **verworfen** und `Err(Erased)` geliefert.
8. **Block-Cache:** sharded, pin-gezählt (kein Block wird entfernt, solange eine `EntryRef` darauf zeigt), hält entsiegelte Blöcke (AN-5). `invalidate_cluster` bei ER1, `invalidate_file` bei Dateiverlust.
9. **Ablauf:** Jede View trägt `created_at`; nach `view_max_age` liefert jeder Zugriff `Err(ViewExpired)`. Damit ist die Wartezeit von ER7 (§13.3) begrenzt.
10. **Floor:** `floor = min(älteste lebende View, Holds, Consumer-Acks)`; `hold_floor` im Catalog sichert Consumer über Neustarts. Versionen unterhalb `floor`, die von einer neueren verdeckt sind, darf Compaction verwerfen.

---

## 11. Wartung

### 11.1 Lanes und Prioritäten

| Lane | Aufgabe | Priorität |
|---|---|---|
| `urgent` | Löschfolgearbeit (ER3…ER7), Purge mit nahender Frist | höchste, darf Compaction abbrechen |
| `flush` | Memtable → Run | hoch, nie hinter Compaction |
| `compact` | Compaction, Platzierung, Blob-GC | normal |
| `idle` | `verify` (Scrub), Verwaisten-Suche, Reserve (Stamp, ID, Redo-Segment) | niedrig |

### 11.2 Claims

`claim(files, job) -> Ok(guard) | Err(Held{by})`, atomar unter `registry`. Höhere Priorität darf niedrigere abbrechen (`job.cancel()` setzt ein Flag; der Job prüft es zwischen Blöcken, gibt Claims frei, verwirft die Ausgabe). **Ausgabe-Edit:** Validierung `claims ⊇ file_remove` (`INV-CLAIM-1`).

### 11.3 Compaction und Platzierung

Leveled Compaction, Größenverhältnis `T` = 10, L0 nach Dateizahl. Ein **Level** ist eine nach `(cluster, user_key)` sortierte Folge nicht überlappender Runs; ein **Pure-Run** ist einfach ein Run, dessen `min.cluster == max.cluster`. Es gibt keine zweite Ebene für Pure.

**Ausgabeplanung** (`placement`), je Cluster im Merge-Strom:

1. Hat der Cluster in der Eingabe **≥ `pure_min_bytes`** (Standard 8 MiB) oder enthält er Körper > `max_inline_bytes`, wird er als **Pure-Run(s)** geschrieben (Aufteilung nach Schlüsselbereichen bei `run_target_bytes`), mit Blob für Körper ≥ `value_separation_min`.
2. Sonst kommt er in den aktuellen **Packed-Run**; **ein kleiner Cluster wird nie auf zwei Dateien eines Levels aufgeteilt** (`INV-PLACE-3`). Der Packed-Run wird bei `run_target_bytes` an einer **Clustergrenze** beendet.
3. Ein Cluster-Segment, das unverändert bleibt (keine Versionsbereinigung nötig, kein Marker), wird **byte-identisch kopiert** (`INV-PLACE-4`): keine Entschlüsselung, keine Neukompression.

Wirkung: Ein Cluster belegt je Level entweder eine eigene Datei oder einen zusammenhängenden Bereich einer einzigen Packed-Datei. Das begrenzt die Löschkosten (§13.4) und Handles.

- **Retired-Filter (Race-Schutz, `INV-ERA-6`):** Jeder Job notiert `retire_gen` beim Start. Beim Ausgabe-Edit (unter `catalog_edit`) vergleicht er: Hat sich `retire_gen` geändert und enthält der Eingabe-Clustersatz einen inzwischen `Retired`-Cluster, wird die Ausgabe **verworfen** und der Job neu geplant. Ausgaben enthalten nie Daten eines retired Clusters.
- **Verwerfen alter Versionen:** unterhalb `floor` verdeckte Versionen und Tombstones, wenn kein tieferes Level den Schlüssel enthält. **Einzelne Cluster sind die Parallelisierungseinheit**; Teil-Compactions teilen nie innerhalb eines Pure-Schlüsselbereichs mitten in einer Version-Kette.
- **Cluster-Promotion:** wächst ein Cluster über `pure_min_bytes`, wird er bei der nächsten Compaction, die ihn berührt, aus dem Packed-Run herausgelöst (Schritt 1).

### 11.4 Platzierungsregeln (Invarianten, Test `placement`)

- `INV-PLACE-1` Block clusterrein. `INV-PLACE-2` Blob clusterrein. `INV-PLACE-3` Kleiner Cluster höchstens in einer Datei je Level. `INV-PLACE-4` Cluster-Segmente sind ohne Schlüssel verschiebbar.
- **Packed-Anteil begrenzt Löschkosten:** `run_target_bytes` (64 MiB) ist die Obergrenze der Bytes, die ein Löschvorgang je Packed-Datei **kopieren** muss.

### 11.5 Blob-GC

Auslöser: Lebendanteil eines Blobs < `blob_gc_live_ratio` (50 %). Der Lebendanteil folgt aus `blob_ref_note`-Edits der Compaction. GC kopiert lebende Records **byte-identisch** (AAD ist ortsfrei) in eine neue Blob-Datei (neue `blob_id`), schreibt die betroffenen Runs **desselben Clusters** um (nur das `ref`-Feld im Eintrag, im Cluster-Segment neu versiegelt) und tauscht alles in **einem** Edit. Da Blobs clusterrein sind, kostet GC nie mehr als das Volumen eines Clusters.

### 11.6 Graveyard

Eine Datei wird erst gelöscht, wenn (a) der entfernende Edit durabel ist und (b) die letzte `Arc<Version>` mit Verweis gefallen ist (`INV-GRAVE-1`). `Drop` einer Version stellt Dateien in die Graveyard-Warteschlange; **das Unlink führt die Wartung aus, nie `Drop`**.

### 11.7 Governor

Token-Bucket je Klasse (Wartungs-E/A, CPU, offene Handles). Wartung nutzt höchstens `maint_io_fraction` (50 %) der gemessenen E/A-Kapazität; `urgent` hat eine eigene Reserve. Dateihandles: LRU-Cache mit `max_open_files`; kalte Cluster-Dateien halten kein Handle.

---

## 12. Recovery

### 12.1 Ablauf beim Öffnen (deterministisch)

```
R1  FORMAT lesen: Magic, Version, Feature-Bits, integrity_mode prüfen (CRC32C). `Unsupported` ⇒ Abbruch.
R2  KEYS lesen und root_key über KEK entsiegeln (Authenticated/Encrypted). Danach FORMAT.chk (Mac16) prüfen.
    KEK nicht verfügbar ⇒ Modus `LockedKeys` (nur Plain-Spaces lesbar, Schreiben verweigert).
R3  CURRENT → Catalog: Snapshot + Edits bis zum letzten gültigen Rahmen; Torn Tail verwerfen (nur am Dateiende).
R4  Redo ab `redo_floor` in **zwei Durchläufen**:
      Durchlauf 1: Gruppenkette prüfen, Erase-Frames und Stamps sammeln → retired_set = Catalog ∪ Erase-Frames
      Durchlauf 2: Frames anwenden; Sektionen retired Cluster überspringen (nie entsiegeln); sonst Sektion entsiegeln und einspielen.
      Fehlt der Clusterschlüssel eines **nicht** retired Clusters ⇒ `Corrupt{KeyMissing}` (Erase-Frame steht stets vor der Zerstörung).
      Torn-Tail-Regel `INV-LOG-8`: Der erste ungültige Gruppenkopf beendet das Log **nur**, wenn danach im Rest des Segments
      (Sektorraster, bis zum Segmentende) **keine** prüfbar gültige Gruppe folgt. Folgt eine, ist es `Corrupt{RedoMidLog}`.
      Alles bis zur letzten `redo_mark` MUSS gültig sein (`INV-LOG-7`), sonst `Corrupt`.
R5  next_stamp = max(stamp_limit, letzter Redo-Stamp, max Run-Stamp) + 1;  visible_stamp = durable_stamp = letzter gültiger Stamp.
R6  Verzeichnisse scannen: Dateien ohne Catalog-Eintrag ⇒ verwaist (Liste; noch nicht löschen).
R7  Offene Löschungen = erase_pending ∪ Erase-Frames ohne erase_done ⇒ in `urgent` einplanen und **ab der ersten offenen Stufe fortsetzen**
    (Roll-forward, `INV-ERA-4`). `destroy_clusters` ist idempotent und wird für offene ER2 wiederholt.
R8  Verwaiste Dateien und `.building.*` entfernen (nach R7, nie davor).
R9  Reserve (Stamp, ID, Redo-Segment) herstellen, Wartung starten.
R10 `OpenReport`: Gruppen, verworfene Bytes, offene Löschungen, verwaiste Dateien.
```

Grenze (ehrlich): Ein beschädigter **letzter** quittierter Gruppenabschnitt hinter der letzten `redo_mark`, hinter dem nichts Gültiges folgt, ist von einem Torn Write nicht unterscheidbar. Die Heilung kürzt dann; das ist die einzige Stelle, an der Bitfäule `durable=true` verletzen kann. `redo_mark` hält das Fenster klein (AN-2).

### 12.2 Absturzmatrix (Soll-Zustand je Unterbrechung, Gate G-W/G-E)

| Unterbrechung | Soll nach Neustart |
|---|---|
| vor Redo-Sync | Change nicht sichtbar, `commit` hat nicht (erfolgreich) zurückgegeben |
| nach Redo-Sync, vor Rückkehr | Change vorhanden (Aufrufer nutzt Guard bei Wiederholung) |
| `Volatile`-Change, Absturz vor Sync | verloren; Stamp wird nicht neu vergeben (`INV-STAMP-3`) |
| Run im Bau | `.building` entfernt, Redo spielt ein |
| Run `rename` ok, Edit fehlt | Run verwaist, entfernt |
| Edit durabel, Unlink offen | alte Dateien verwaist, entfernt |
| Erase-Frame durabel, `erase_pending` fehlt | Recovery R7 leitet `pending` aus dem Frame ab |
| ER1 durabel, Schlüssel noch vorhanden | Roll-forward ab ER2 |
| Schlüssel zerstört, Edit `cluster_keys_destroyed` fehlt | `probe` = `Absent` ⇒ Edit nachholen |
| ER6-Edit durabel, Unlink offen | Verwaisten-Suche entfernt, `erase_done` nachholen |
| Ledger-Eintrag halb | Kette erkennt Torn Tail; Eintrag wird aus Catalog-Zustand neu erzeugt |
| Checkpoint abgebrochen | `dst` unvollständig, Quelle unverändert; `dst` trägt keinen Manifest-Siegel |

---

## 13. Löschung

### 13.1 Bedeutung der Wörter

- **Logisch atomar:** Ab dem Stamp `S` des Erase-Frames liefert jeder Lesezugriff auf den Cluster `Err(Erased)`; das gilt für alle Spaces und alle Schlüssel zugleich, weil der Zustand `Retired` **ein** Eintrag je Cluster ist.
- **Kryptografisch:** Der Clusterschlüssel ist beim `KeyProvider` zerstört und per `probe` als `Absent` bestätigt.
- **Physisch atomar:** **ein** Catalog-Edit (ER6) trennt „Dateien lebendig“ von „nicht mehr referenziert“. Davor Roll-forward, danach Aufräumen.
- **Nachweisbar:** `ErasureReceipt` erst nach Unlink, `sync_dir` und durablem Ledger-Eintrag (`INV-ERA-8`).

### 13.2 Stufen und ehrliche Garantien

| Stufe | Ablauf | Schützt gegen | Schützt **nicht** gegen |
|---|---|---|---|
| `Logical` | ER0–ER1 | Lesen über die API | Inhalt auf dem Datenträger |
| `Cryptographic` (nur `Encrypted`) | ER0–ER2 sofort, ER3–ER7 gebündelt mit Frist (§13.4) | Schlüssel, Kopfdaten und Körper in Redo, Runs, Blobs, Backups, Flash (alles clusterversiegelt) | Klartext-Metadaten (Cluster-ID, Größen, Stamps) bis ER6; Kopien des Clusterschlüssels; Speicherabbilder |
| `Physical` | ER0–ER7 sofort (ER2 in `Encrypted`) | Wiederauftauchen aus lebenden Dateien des Verzeichnisses | Dateisystem-Journal, Flash-Reste, Snapshots/Backups außerhalb, Hardlinks |
| `Scrubbed` | wie `Physical`, vor Unlink Überschreiben mit Nullen + Sync | einfache Wiederherstellung auf klassischen Datenträgern | CoW-Dateisysteme, SSDs (best effort) |

In den Modi `Plain` und `Authenticated` gibt es keine Clusterschlüssel: `Cryptographic` liefert dort `Err(Unsupported)`, und Vertraulichkeit entsteht **nur** durch Physical/Scrubbed. Für personenbezogene Daten ist `Encrypted` der Pflichtmodus der Anwendung (O-1).

### 13.3 Protokoll `Eraser::retire` (ER0 → ER7)

```
ER0 Vorprüfung (ohne Zustandsänderung): Clusterliste normalisieren; Restriction ⇒ Err(Restricted) für die ganze Liste;
    Level vs. Modus (Cryptographic nur Encrypted); KeyProvider erreichbar (nur Crypto-Level); bereits Retired ⇒ vorhandenes Ticket.
ER1 LOGISCHER PUNKT (im Committer): Re-Check Restriction; Stamp S; **Segmentschnitt** (aktuelles Segment siegeln, Erase-Frame als
    erstes Frame des neuen Segments); Gruppen-Fsync. Danach: cluster_table ← Retired{S}; retire_gen++; Memtable-Einträge der Cluster
    entfernen; Block-Cache invalidieren. Danach Edit {cluster_retire, erase_pending_add}; Ledger `Retired`.
    → `retire()` kehrt hier mit `Ok(ticket)` zurück. Ab hier ist **nie** mehr ein Fehler Antwort des Aufrufs.
ER2 SCHLÜSSEL (nur Encrypted): KeyProvider::destroy_clusters(list) (durabel, idempotent); probe == Absent; Schlüssel-Cache leeren;
    Edit {cluster_keys_destroyed}; Ledger `KeyDestroyed`.                                                   ◄── KRYPTOGRAFISCHER PUNKT
ER3 REDO-HYGIENE: Flush-Barriere: alle Memtables mit Einträgen in Segmenten < Schnitt einfrieren und flushen (retired Cluster werden
    übersprungen); `redo_floor` ≥ Schnittsegment; ältere Segmente ⇒ Graveyard. (Kein Segment-Rewrite nötig, `INV-LOG-4/5`.)
ER4 PLAN: je Space betroffene Dateien: Pure-Runs/-Blobs des Clusters ⇒ Drop-Liste; Packed-Runs mit dem Cluster ⇒ Umschreib-Liste;
    Segmente der Besitzer ⇒ rewrite_without-Aufträge. Claims (urgent, bricht niedrigere ab).
ER5 UMSCHREIBEN: je Packed-Run neuen Run bauen: alle **anderen** Cluster-Segmente byte-identisch kopieren (`INV-PLACE-4`, kein Schlüssel,
    keine Entschlüsselung), neue ClusterDirectory/Footer; `.building` → sync_all → rename → sync_dir. Besitzer bauen Segmente neu.
ER6 PHYSISCHER PUNKT: **ein** Edit {file_add[neu], file_remove[alt + Drop-Liste], segment_*, cluster_purged{S}, erase_phys{S},
    erased_range_add}; danach Ledger `Physical`.
ER7 ABSCHLUSS: neue Version veröffentlichen; Dateien ins Graveyard; Wartezeit auf Views gemäß `ViewPolicy` (§13.5);
    (Scrubbed: überschreiben + sync); unlink; sync_dir; alle Besitzer bestätigt (`INV-ERA-8`); Ledger `Done`; Edit {erase_done};
    `ErasureReceipt`.
```

**Reihenfolge ist Vertrag (L-4):** ER1 vor ER2, ER2 vor ER6, ER6 vor Unlink (`INV-ERA-3`). Der Kern führt ER4–ER6 im Level `Cryptographic` erst aus, wenn `probe` den Schlüssel als `Absent` zeigt. `destroy` liegt vor dem Edit `cluster_keys_destroyed`, damit `probe` die Wahrheit liefert.

### 13.4 Kosten, Bündelung, Frist

- **ER1, ER2:** O(Clusterzahl); ER1 ein Gruppen-Fsync plus ein Edit, ER2 ein Provider-Aufruf plus ein Edit.
- **ER3:** normale Flush-Last der Instanz (begrenzt durch `memtable_total_bytes`).
- **ER4/ER5:** O(Bytes der betroffenen **Packed-Dateien**), je Datei höchstens `run_target_bytes`; **Pure-Cluster kosten nur Unlink**. Weil Clusterkennungen benachbart sein können (§5.3), teilen sich gemeinsam gelöschte Cluster dieselben Packed-Dateien: eine Datei wird einmal umgeschrieben, nicht je Cluster.
- **Bündelung:** Wartende Purges (ER3–ER7) je Space werden gesammelt; ein Lauf entfernt alle wartenden Cluster in einem Durchgang. Auslöser: (a) die Frist des ältesten Tickets zu 70 % verbraucht, (b) ≥ `purge_batch_min` (64) wartende Cluster, (c) eine anstehende Compaction in der betroffenen Datei **übernimmt** den Purge (der Edit enthält `cluster_purged` gleich mit).
- **Frist** `purge_deadline`: Standard 72 h, je Anfrage überschreibbar (nur `Cryptographic`). Wird sie gerissen, steigt die Priorität auf `urgent`, und `Observer` erhält ein Ereignis. Die Frist ist **keine Rechtsaussage** (O-1). `Physical`/`Scrubbed` laufen sofort (Bündelung nur mit gleichzeitigen Anfragen).

### 13.5 Restriction, Hold, Views, Consumer

- **Restriction ≠ Hold (SD-17).** Ein `Hold` ist **technisch**: ein Erase überstimmt ihn ohne Fehler (Views auf einen retired Cluster liefern `Erased`; Consumer sehen `Change::Erased{cluster, at}`). Eine `Restriction` ist ein **rechtlicher** Vermerk mit Pflichtgrund (`ReasonCode`, opak): Solange sie besteht, lehnt `retire` den Cluster mit `Err(Restricted)` ab; `blocks_reads` sperrt zusätzlich das Lesen. Der Kern **überschreibt** eine Restriction nie; es gibt kein `Override`. Ob und wann eine Restriction gesetzt oder gelöst wird, entscheidet der Aufrufer (NZ-4).
- **Atomarität:** `restrict`/`release` laufen als Control-Frame durch denselben Committer wie ER1; ein gleichzeitiges `retire` und `restrict` ist durch die Committer-Reihenfolge entschieden, nie beides.
- **`ViewPolicy`:** `Wait(timeout)` (Standard): ER7 wartet, bis alle `Arc<Version>`, die auf die Dateien zeigen, gefallen sind (spätestens nach `view_max_age`); nach dem Timeout bleibt der Vorgang in `Purging{AwaitingViews}`, ohne Fehler. `Revoke` lässt betroffene Views beim nächsten Zugriff `Err(Revoked)` liefern. Lesen **in** den retired Cluster ist ohnehin gesperrt (§10, Punkt 2), eine View kann also nur noch **andere** Cluster derselben Datei halten.

### 13.6 Kein stilles Fälschen

Der Catalog führt `erased_ranges` für `EraseRange` (§13.7); Cluster-Löschungen stehen im Zustand `Retired|Purged`. Beide antworten mit `Err(Erased{..})`, nie mit leerem Ergebnis und nie mit Altdaten (`INV-ERA-5`). Einträge werden entfernt, sobald `floor ≥ S` und kein Checkpoint mit `erase_epoch < Ledgerposition(S)` mehr vom Kern verwaltet wird.

### 13.7 `EraseRange` (Löschung unterhalb des Clusters)

Für Löschung einzelner Schlüsselbereiche (zum Beispiel ein Dokument in einem Cluster) gilt ein **rein physischer** Weg ohne Schlüssel: Marker `EraseRange` mit Stamp `S` im Redo (Erase-Frame mit Bereich), `erased_range_add`, dann `urgent`-Umschreiben aller Runs und Blobs, die den Bereich berühren (nur innerhalb des Clusters). Level: `Logical | Physical | Scrubbed` (`Cryptographic` ist nicht möglich: es gibt keinen Schlüssel je Bereich). Views **ab** `S` sehen den Bereich als gelöscht (nicht vorhanden); Views **unterhalb** `S` erhalten `Err(Erased{cluster, range, at})`. Aufrufer, die Löschung ohne physische Wartezeit brauchen, legen Löschbares in **eigene Cluster** (L-1).

### 13.8 Grenzen und Restexposition (offen benannt)

- **P-K (Vertrag):** Nutzerschlüssel enthalten keine Kennungen natürlicher Personen. Der Kern kann das nicht prüfen; `Inspector::lint_keys` sucht in Tests nach Mustern. Im Modus `Encrypted` stehen Nutzerschlüssel nur **innerhalb** clusterversiegelter Bereiche, ihre Zerstörung wirkt sofort; in `Plain|Authenticated` bis ER6.
- **Klartext-Metadaten** in `Encrypted`: `ClusterDirectory`, `MetaBlock`, Footer, Redo-Frame-Köpfe, Catalog, Ledger enthalten opake Cluster-IDs, Größen und Stamps. Nach ER6 sind sie entfernt (Directory) bzw. nach `floor ≥ S` (Catalog). Ob das datenschutzrechtlich genügt, ist **Rechtsfrage** (O-10).
- **Hash-Identität (SD-22):** Der Kern bildet **keine** Inhalts-Hashes über Klartext (`INV-PRIV-1`); Filter und `key_digest` sind keyed und clustergebunden.
- Speicherinhalt, Swap, Core-Dateien: Puffer werden best effort überschrieben (`forbid(unsafe_code)`); Speicherschutz ist Sache einer Port-Implementierung.
- Externe Backups, Dateisystem-Journal, Flash-Wear-Leveling, Hardlinks: siehe §13.2.
- **Abgeleitete Daten außerhalb des Kerns** sind nur so sicher, wie ihre Besitzer §20.2 einhalten; der Kern wartet auf deren Bestätigung (`INV-ERA-8`) und überspringt sie nie.
- Anhängige Aufrufer, die Klartext schon erhalten haben, sind außerhalb der Kontrolle des Kerns.

### 13.9 Ledger, Checkpoints, Restore

- **Ledger** (§7.7): append-only, außerhalb von Catalog und Checkpoints, mit eigener Aufbewahrung (länger als jedes Backup). Pfad konfigurierbar (`ledger_path`, Empfehlung: anderes Volume). Vor jedem ER-Übergang wird der Eintrag der **vorigen** Stufe durabel bestätigt gefunden; die Reihenfolge Catalog → Ledger → Receipt ist Vertrag.
- **Checkpoint:** (1) Flush-Barriere über alle Spaces; (2) unter `catalog_edit` die `Version` pinnen und `erase_epoch = Ledgerposition` lesen (Serialisierung mit ER6); (3) Dateien kopieren (`Copy` Standard, `Reflink`, `Hardlink` nur Opt-in mit Warnung: ein Purge erreicht den Inode im Checkpoint nicht); (4) Manifest (Stamp, Dateiliste mit `footer_chk`, `erase_epoch`, `instance_id`) schreiben und siegeln. **`KEYS` ist enthalten, die KEK und alle Clusterschlüssel nicht** (`INV-KEY-1`).
- **Restore:** `open_restored(cfg, env, ledger)` wendet vor der ersten Freigabe zum Lesen alle Ledger-Einträge **nach** `erase_epoch` erneut an: betroffene Cluster werden `Retired`, offene Stufen laufen weiter (`INV-ERA-9`). Zerstörte Clusterschlüssel kehren nicht zurück, weil sie nie Teil des Checkpoints waren. Läuft die Kopie parallel zum Original, ruft der Aufrufer `rekey_clone()` (neuer `instance_id`, neuer `root_key`, `KEYS` neu) **vor** dem ersten Schreiben (`INV-EPOCH-1`).

---

## 14. Verschlüsselung und Schlüssel

### 14.1 Modi (stehen in FORMAT, unveränderlich)

| Modus | Schutz | Verfahren |
|---|---|---|
| `Plain` | Zufallsfehler | CRC32C; Kettenprüfsumme im Redo (nicht kryptografisch) |
| `Authenticated` | Manipulation, keine Vertraulichkeit | BLAKE3-keyed-MAC (16 B), Kette in Redo, Catalog, Ledger |
| `Encrypted` | Vertraulichkeit + Integrität | AEAD (XChaCha20-Poly1305) für Redo-Sektionen, Blöcke, Blob-Records, Segmentinhalte, Catalog; Clusterschlüssel pro Cluster |

Eigene Kryptografie wird nicht geschrieben (NZ-2). Algorithmuswechsel ist nur durch Migration möglich (Algorithmus-IDs im FORMAT).

### 14.2 Schlüsselhierarchie

```
KEK (KeyProvider, extern; nie in Dateien, nie im Checkpoint)
 └─ root_key (zufällig, mit KEK umhüllt in KEYS)
      ├─ redo_key / catalog_key / mac_key  = Kdf(root_key, Zweck ‖ instance_id)
      └─ space_key(space) = Kdf(root_key, "space" ‖ space)         → Schlüssel für Spaces ohne `clustered`
Clusterschlüssel CK(cluster)   KeyProvider::cluster_key(cluster)   (NICHT in KEYS)  → Schlüssel für alles, was zum Cluster gehört
```

```rust
pub trait KeyProvider: Send + Sync {
    fn kek(&self) -> Result<Key32, KeyError>;
    fn ensure_cluster_key(&self, c: ClusterId, e: &dyn Entropy) -> Result<KeyHandle, KeyError>; // get-or-create; Err(Retired) nach Zerstörung
    fn cluster_key(&self, c: ClusterId, epoch: u32) -> Result<Option<Key32>, KeyError>;        // None = zerstört
    fn destroy_clusters(&self, c: &[ClusterId]) -> Result<(), KeyError>;                       // durabel vor Rückkehr, idempotent, endgültig
    fn probe(&self, c: ClusterId) -> Result<KeyState, KeyError>;                               // Present | Absent
}
```

Der Provider hält die Schlüssel (HSM, Tresor, eigener Store) und **merkt sich Zerstörungen**: `ensure_cluster_key` für eine zerstörte `ClusterId` liefert `Err(Retired)` (verhindert Wiederbelebung). Der Kern hält nur einen begrenzten **Schlüssel-Cache** (TTL, Obergrenze, `Zeroizing`). Ein Schlüsselbund für Millionen Cluster ist Sache des Providers; ein hierarchischer Provider (Elternschlüssel hüllt Kindschlüssel) ist möglich, der Kern sieht nur `cluster_key`.

### 14.3 Versiegelung: genau eine Schicht je Darstellung (SD-07)

**Einmalschlüssel:** Für jede Versiegelung zieht der Kern `salt[24]` aus `Entropy` und leitet `okm[56] = Kdf(CK, "ctxs seal v1" ‖ domain ‖ salt)`; `key = okm[0..32]`, `nonce = okm[32..56]`. Es gibt keinen Zähler; ein (Schlüssel, Nonce)-Paar entsteht nur einmal (Kollisionsraum 192 Bit). `salt` steht neben dem Chiffrat.

| Schicht | Domäne | Was | AAD |
|---|---|---|---|
| Redo | `0x01` | Sektion `(space, cluster)` eines Frames | `(instance_id, segment_id, stamp, space, cluster)` |
| Block | `0x02` | Block, `SegIndex`, Filter, Markerblock im Cluster-Segment | `(space, cluster, segment_uid, block_no)` |
| Blob | `0x03` | Blob-Record | `(space, cluster, key_digest, entry_tag)` |
| Segment | `0x04` | Besitzer-Nutzlast über `SegmentIo` | `(kind, cluster, owner_ver, seg_uid)` |

Jede **Darstellung** trägt genau eine Schicht; derselbe Klartext wird in Redo, Run und Blob unter **verschiedenen** Darstellungen mit verschiedenen Einmalschlüsseln versiegelt, nie dieselben Bytes zweimal. Die Alternative „Körper einmal versiegeln und überallhin kopieren“ (Prüfbericht R3, CR-R3-11) wurde geprüft und verworfen: sie zwingt die Memtable, Chiffrat zu halten (Entschlüsselung bei jedem Lesen), koppelt die AAD an die Version statt an den Ort und spart nur einen Hintergrund-AEAD-Durchlauf je Körper. Ortsfreie AAD für Blocks und Blobs (§7.5, §7.6) liefert die Verschiebbarkeit, die der Vorschlag erreichen wollte. Im Modus `Encrypted` für Spaces ohne `clustered` ist `CK := space_key`.

**Die Versiegelung des Redo geschieht auf dem Aufruferthread vor dem Intake** (parallel, ohne Sperre); der Committer vergibt nur Stamp und Position.

### 14.4 Rotation

`root_key`-Rotation: neuer `key_gen`, `KEYS` neu, abgeleitete Schlüssel wechseln bei Neuschreiben der betroffenen Dateien. Clusterschlüssel-Rotation (`key_epoch`) ist im Format **vorgesehen** (Feld in `ClusterDirectory`, in v1 immer 1) und hat in v1 **kein** Verfahren (O-9). **Rotation ersetzt keine Löschung.**

---

## 15. Nebenläufigkeit: Race-Freiheit durch Konstruktion

### 15.1 Prinzipien

1. **Ein Schreiber:** alle Zustandsänderungen laufen durch den Committer (Datenänderungen, Erase-Frames, Control-Frames) oder durch einen Catalog-Edit (serialisiert über `catalog_edit`).
2. **Unveränderlichkeit:** Dateien sind nach `rename` unveränderlich; die `Version` ist unveränderlich und wird atomar ersetzt.
3. **Besitz durch Claims:** jede Datei hat höchstens einen Wartungs-Claim.
4. **Wahrheit durch Edits:** der Catalog entscheidet, nicht der Speicherzustand.
5. **Zähler statt Sperren:** `visible_stamp`, `durable_stamp`, `retire_gen` sind atomare Zähler.
6. **Keine Sperre über E/A**, mit Ausnahme von `committer` (Redo-Sync), `catalog_edit` (Catalog-Sync) und `ledger` (Ledger-Sync); **nie zwei dieser drei gleichzeitig** (`INV-LOCK-2`).

### 15.2 Sperrränge (nur aufsteigend, je Thread höchstens einmal; `debug_assertions` prüft per Rangstack)

| Rang | Sperre | E/A erlaubt |
|---|---|---|
| 1 | `intake` | nein |
| 2 | `committer` | ja (Redo-Sync) |
| 3 | `catalog_edit` | ja (Catalog-Sync) |
| 4 | `ledger` | ja (Ledger-Sync) |
| 5 | `state` (`Arc<Version>`-Zeiger) | nein |
| 6 | Memtable-Sperre | nein |
| 7 | `registry` (Holds, Consumer, Claims) | nein |
| 8 | `cluster_table` | nein |
| 9 | Blätter: Cache, Schlüssel-Cache, Governor, `notify` | nein |

Stamp-Reserve und `retire`-Anfragen nehmen `catalog_edit` **nach** Freigabe von `committer`.

### 15.3 Die Nahtstellen und ihre Lösungen

| Nahtstelle | Gefahr | Lösung | Nachweis |
|---|---|---|---|
| Lesen × Unlink | Datei weg, Leser hält Verweis | Graveyard + `Arc<Version>`; Unlink nur durch die Wartung | loom, Sim |
| View-Aufbau × Veröffentlichung | View sieht Stamp ohne dessen Daten | `visible_stamp` zuerst, `Version` danach (`INV-VIEW-2`) | loom, M1 |
| Compaction × Erase | Compaction schreibt Daten zurück | Claims + `retire_gen`-Validierung im Ausgabe-Edit (§11.3), Retired-Filter | M3, Sim |
| Flush × Erase | Memtable hält Daten | Committer entfernt in ER1, Flush überspringt Retired, `redo_floor`-Regel (`INV-LOG-5`) | M2, Sim |
| Schlüssel-Cache × Zerstörung | Entsiegeln nach ER2 | ER1 vor ER2; Nachprüfung vor/nach Entsiegeln (`INV-KEY-2`); `invalidate_cluster` | loom |
| Restrict × Retire | beides gleichzeitig | beide als Frames im Committer (SD-17) | M2 |
| Checkpoint × Erase | Checkpoint kopiert Dateien, die ER6 ersetzt | Pin der `Version`, `erase_epoch`, Serialisierung über `catalog_edit`; Restore wendet Ledger an | M4, Sim |
| Consumer × Erase | Consumer sieht Gelöschtes | `Change::Erased`; Holds überstimmt, Restriction nicht | M2 |
| Zwei Erase auf überlappenden Clustern | Doppelarbeit, Blockade | Ticket-Reihenfolge in `urgent`; bereits Retired ⇒ vorhandenes Ticket | Sim |
| Blob-GC × Compaction | Verweise veralten | Blobs clusterrein; GC und Referenzumschreibung in **einem** Edit unter Claims | M3 |
| Stamp-Reserve × Committer | Warten unter Committer | Reserve **vor** dem Rang (§9.1 Schritt 3) | Sim |
| Segmentbesitzer × Erase | Besitzer antwortet nicht | Erase bleibt in `Purging`, Alarm, nie übersprungen | Test |

### 15.4 Verbotene Muster (Lint und Review)

`unwrap/expect/panic/todo/unreachable/indexing_slicing`; `std::sync::Mutex` außer über `sync::Ranked`; Hintergrundthreads außerhalb des `Spawner`; blockierende E/A unter Rang ≥ 5; eine zweite Schreibquelle neben Committer und Catalog-Edit; Rückrufe unter Sperren; Zählernonces; Vergabe eines Stamps ohne Reserve.

---

## 16. Fehlermodell

| Klasse | Beispiele | Verhalten |
|---|---|---|
| F-1 transient | `EINTR`, kurze Überlast | Wiederholung mit Backoff, begrenzt |
| F-2 Ressource | `ENOSPC`, Handle-Limit | Rückdruck, dann Poison des Schreibpfads; Lesen bleibt |
| F-3 E/A-hart | `EIO`, Fsync-Fehler | **nie wiederholen**; Poison; betroffene Changes `Indeterminate`; Neustart über Recovery |
| F-4 Integrität | CRC/MAC/AEAD-Fehler | kleinste isolierbare Einheit „vergiftet“ (Block → Cluster-Segment → Run → Space); `Err(Corrupt)`; übrige Einheiten laufen |
| F-5 Format | `Unsupported`, unbekannter Catalog-Tag | Öffnen verweigert, nie ignorieren |
| F-6 Schlüssel | Provider offline, Schlüssel fehlt | `LockedKeys` bzw. `Corrupt{KeyMissing}`; ER2 bleibt offen und wird wiederholt |
| F-7 Aufrufer | Guard verletzt, Cluster retired/restricted, Größen | `CommitError`/`ReadError`-Varianten mit `hint()` |
| F-8 Logik | Invariante verletzt (Bug) | Debug: Abbruch im Test; Release: Poison + Observer-Ereignis; **kein** `panic` im Produktionspfad |

`Indeterminate` heißt: Der Kern weiß nicht, ob die Änderung durabel ist. Der Aufrufer verifiziert nach Neustart per Guard oder Lesen.

---

## 17. Invarianten (vollständiger Katalog; jeder Test zitiert die ID, `// INV-…`)

| ID | Aussage |
|---|---|
| INV-STAMP-1 | Stamps sind streng monoton über angenommene Changes |
| INV-STAMP-2 | Überlauf ⇒ `Exhausted`, nie Umbruch |
| INV-STAMP-3 | Ein vergebener Stamp wird auch nach Absturz keiner anderen Änderung zugeordnet (`stamp_limit`) |
| INV-COMMIT-1 | Ein sichtbarer `Durable`-Eintrag ist durabel; `Volatile` ist sichtbar vor dem Sync |
| INV-ATOM-1 | Eine Change ist ganz oder gar nicht sichtbar, auch nach Absturz |
| INV-VIEW-1 | Eine View bei Stamp `V` sieht dieselben Daten unabhängig von späteren Schreibungen |
| INV-VIEW-2 | `visible_stamp` wird vor der `Version` geladen |
| INV-CONS-1 | Consumer und `view_durable` sehen nur Stamps ≤ `durable_stamp` |
| INV-FMT-1 | Nur kanonische Kodierungen; unbekannte Versionen/Tags/Pflichtbits ⇒ Fehler, nie Ignorieren |
| INV-LOG-1 | Die Gruppenkette ist lückenlos und prüfbar bis zum Ende des gültigen Präfixes |
| INV-LOG-2 | Ein Redo-Segment wird nie wiederverwendet |
| INV-LOG-3 | `durable=true` ⇒ die Gruppe wurde synchronisiert |
| INV-LOG-4 | Ein Erase-Frame ist das erste Frame seines Segments |
| INV-LOG-5 | `redo_floor` passiert keinen Erase-Frame, dessen `erase_pending` nicht im Catalog steht |
| INV-LOG-6 | Ein vorallokiertes Reservesegment existiert stets |
| INV-LOG-7 | Alles bis zur letzten `redo_mark` ist prüfbar gültig, sonst `Corrupt` |
| INV-LOG-8 | Torn-Tail-Heilung nur, wenn danach keine prüfbar gültige Gruppe folgt |
| INV-CAT-1 | Datei lebendig ⇔ durabler `file_add`/`segment_add` ohne `*_remove` |
| INV-GRAVE-1 | Keine Datei wird gelöscht, solange eine `Version` auf sie zeigt |
| INV-CLAIM-1 | Ein Edit entfernt nur Dateien, deren Claim der Job hält |
| INV-PLACE-1 | Jeder Block ist clusterrein |
| INV-PLACE-2 | Jede Blob-Datei gehört zu genau einem Cluster |
| INV-PLACE-3 | Ein kleiner (Packed-)Cluster liegt je Level in höchstens einer Datei |
| INV-PLACE-4 | Cluster-Segmente sind ohne Schlüssel byte-identisch verschiebbar |
| INV-ERA-1 | Logische Atomarität: ab `S` liefert jeder Lesezugriff auf den Cluster `Erased` |
| INV-ERA-2 | Nach `Purged` enthält weder ein lebender Run noch ein lebendes Redo-Segment noch ein Blob Daten des Clusters (Test: Orakel) |
| INV-ERA-3 | Reihenfolge ER1 < ER2 < ER6 < Unlink; kein Purge im Level `Cryptographic` vor `probe == Absent` |
| INV-ERA-4 | Nur Roll-forward: ein begonnener Erase wird nach jedem Absturz fortgesetzt oder ist nie begonnen |
| INV-ERA-5 | Lesen im gelöschten Bereich liefert `Erased`, nie Daten, nie „leer“ (Ausnahme §13.7: Views ab `S` einer `EraseRange`) |
| INV-ERA-6 | Ein Ausgabe-Edit enthält nie Daten eines Clusters, der seit Jobstart `Retired` wurde |
| INV-ERA-7 | Im Modus `Encrypted` ist nach ER2 nichts des Clusters (Schlüssel, Kopf, Körper) in Redo, Run, Blob, Segment lesbar |
| INV-ERA-8 | `ErasureReceipt` existiert nur, wenn INV-ERA-2 gilt, der Ledger-Eintrag durabel ist und alle Segmentbesitzer bestätigt haben |
| INV-ERA-9 | Nach `open_restored` ist jeder Cluster des Ledgers wieder `≥ Retired` |
| INV-RESTR-1 | Ein Cluster mit Restriction wird vom Kern nie `Retired`; beide Zustände schließen sich durch die Committer-Reihenfolge aus |
| INV-KEY-1 | Schlüsselmaterial liegt nie in Dateien außer als `root_key` umhüllt; nie im Checkpoint |
| INV-KEY-2 | Keine Entsiegelung liefert Klartext, wenn der Cluster während des Aufrufs `Retired` wurde |
| INV-KEY-3 | Hochwassermarken (`id_limit`) werden vor Nutzung durabel angehoben |
| INV-EPOCH-1 | Eine parallel betriebene Kopie hat eigenen `instance_id` und `root_key` |
| INV-PRIV-1 | Keine Inhalts-Hashes über Klartext; Cluster-IDs sind opak |
| INV-LOCK-1 | Sperren nur aufsteigend, je Thread höchstens einmal |
| INV-LOCK-2 | Nie zwei E/A-haltende Sperren (committer, catalog_edit, ledger) zugleich |

---

## 18. Verifikation (Gate-relevant)

Ziel: **belegte Eigenschaften**, nicht „bisher keine Fehler gesehen“.

### 18.1 Ebenen

| Ebene | Werkzeug | Inhalt |
|---|---|---|
| Formal | TLA+/TLC (oder Quint) | **M1** Commit, Sichtbarkeit, Recovery, Stamps · **M2** Erase ER0–ER7 mit Absturz an jedem Schritt, inkl. Restrict/Retire-Rennen · **M3** Compaction × Erase × Claims × Blob-GC · **M4** Checkpoint × Erase × Restore · **M5** Platzierung (Promotion, byte-identisches Kopieren). Eigenschaften: INV-ATOM-1, INV-ERA-*, -PLACE, -GRAVE, -LOG, -STAMP |
| Modellprüfung Code | `loom` | `Arc`-Wechsel, Zähler (`visible_stamp`, `retire_gen`), Claims, Graveyard, Gruppenführung, Schlüssel-Cache |
| Deterministische Simulation | eigener Scheduler + `shuttle` | **Hauptwerkzeug gegen Races:** seeded Interleavings aller Threads mit `FaultFs`, `SimClock`, `SimEntropy`; Millionen Läufe, per Seed reproduzierbar |
| Absturzmatrix | `FaultFs` | Abbruch an **jedem** E/A-Index; Torn Writes (sektorweise), verlorene Fsyncs, Rename-Reihenfolgen, Bitflips im Redo (Mitte/Ende); danach alle INV prüfen |
| Eigenschaftstest | `proptest` | Codecs (Rundlauf, Kanonizität), Ordnung, Compaction/Platzierung äquivalent zu einem `BTreeMap`-Referenzmodell mit Versionen |
| Linearisierbarkeit | Historienprüfer (Wing-Gong) | `commit`/`view`-Historien aus der Simulation gegen die sequenzielle Spezifikation |
| Löschorakel | `ErasureOracle` (Feature `testing`) | Eindeutige Muster in Schlüssel, Header, Körper; nach Erase **alle Bytes aller Dateien** durchsuchen (Runs, Blobs, Redo, `.building`, Segmente, `MemFs`-Zustand, Checkpoints) |
| Fuzzing | `cargo-fuzz` | Dekoder aller Formate, Recovery auf beschädigten Verzeichnissen (≥ 10 Ziele) |
| Referenzvektoren | unabhängiger Python-Codierer | Goldene Bytes je Format (Anhang A) |
| Statisch | clippy, `cargo-deny`, `layering`, Lint-Regeln | P-5, P-8, P-9, keine Zählernonces |
| Dauerlauf | Soak 72 h, Crash-Loop, Platte-voll | Stabilität, Speicherwachstum, Handle-Lecks |

### 18.2 Orakel-Vertrag

`Plain/Authenticated`: nach ER6 Trefferzahl 0 für **alle** Muster. `Encrypted`: nach ER2 und vor ER6 Treffer 0 für Körper-, Header- **und** Schlüsselmuster in allen Dateien außer den Klartext-Metadaten (dort nur Cluster-IDs); nach ER6 0 auch dort. Zusätzlich prüft `verify` die Catalog-Seite.

### 18.3 CI

Pflicht je Änderung: `check`, `fmt`, `clippy -D warnings`, `test` (alle Targets), `layering`, `loom`, `doc -D warnings`, `deny`, Simulations-Kurzlauf, Absturzmatrix-Kurzlauf. Nachtlauf: lange Simulation, Fuzz, Orakel-Vollsuche. **Ohne grüne CI ist kein Gate gültig.**

---

## 19. Leistung (Hypothesen mit Messverfahren)

Alle Werte sind **Entwurfsziele**. Freigabe nur mit Messbericht (Hardware, Dateisystem, Last, Methode, Streuung). Scheitert eine Hypothese, wird **die Hypothese** korrigiert und begründet, nie das Messverfahren gelockert.

| ID | Hypothese | Messung |
|---|---|---|
| H-1 | `commit` mit Gruppen-Fsync: Durchsatz ≥ 80 % der Fsync-Rate × mittlere Gruppengröße; p99 ≤ 2 × Fsync-Latenz bei mittlerer Last | Mikrobench, variierende Schreiberzahl |
| H-2 | Punktabfrage warm: ≤ 1 Block-Read je Level; p99 ≤ 50 µs mit Cache-Treffer | Zipf-Last |
| H-3 | Header-Scan ≥ 10× schneller als Scan mit Körper bei Körpern ≥ 4 KiB | Scan-Bench |
| H-4 | `page` liest ≤ 1,2 × Blöcke des theoretischen Minimums | Gewichtsverteilungen |
| H-5 | Schreibverstärkung: Metadaten ≤ 15, Körper in Pure-Clustern ≤ 2 | Compaction-Statistik |
| H-6 | ER1 + ER2 für einen Cluster ≤ 10 ms (ohne Providerlatenz); für 1000 Cluster in einer Anfrage ≤ 2 × ein Cluster + Providerzeit | Löschbench |
| H-7 | Löschdauer ER4–ER7 eines Clusters proportional zu den **Packed-Nachbardateien** (≤ `run_target_bytes` je Datei), unabhängig von der Store-Größe (Faktor ≤ 1,5 bei 10× Store) | Skalierungstest |
| H-8 | Pure-Cluster löschen: ER4–ER7 ≤ 100 ms je 1000 Dateien (ohne Scrub) | Drop-Bench |
| H-9 | Bündelung: 1000 benachbarte Cluster in Packed-Dateien ≤ 2 × der Aufwand für 1 Cluster | Bündelungsbench |
| H-10 | Speicher ≤ `memtable_total_bytes` + Cache-Obergrenze + O(offene Dateien + Retired-Cluster) | RSS-Messung |
| H-11 | Wiederanlauf ∝ Redo-Rückstand + Catalog; 1 GiB Redo ≤ 10 s | Recovery-Bench |
| H-12 | Byte-identisches Cluster-Segment-Kopieren erreicht ≥ 70 % der sequenziellen Plattenrate | Kopierbench |
| H-13 | `Consumer::next_batch` am Kopf (Rückstand ≤ 1 Memtable): ≤ 2 × Punktscan-Kosten; weit zurück: dokumentierte lineare Kosten je Run-Stamp-Fenster | Consumer-Bench (O-7) |
| H-14 | Zusatzkosten `Encrypted` ≤ 25 % Durchsatz gegenüber `Plain` bei Körpern ≤ 4 KiB | Vergleichsbench |

---

## 20. Erweiterungsverträge

Der Kern kennt die Bedeutung fremder Strukturen nicht. Er bietet vier feste Verträge (L-9, L-11).

### 20.1 Consumer und Änderungsstrom

`Consumer` ist die einzige Schnittstelle für Strukturen, die **aufholen** müssen. Garantien: Reihenfolge nach Stamp; mindestens einmal (Idempotenz ist Sache des Konsumenten); nur durable Stamps (`INV-CONS-1`); `Change::Erased{cluster, at}` für Löschungen, damit abgeleitete Strukturen **innerhalb der Frist** bereinigen; `ack` durabel; `Lagged` bei Überschreiten von `max_lag` (Hold wird aufgehoben, damit Compaction nicht unbegrenzt blockiert). Umsetzung: stampfensterweiser Scan mit Block-Stamp-Pruning (min/max Stamp je Block im `SegIndex`); Kosten sind Hypothese H-13 (O-7).

### 20.2 Segment-Register

Dateien neben den Runs, vom Catalog verwaltet (`segment_add/remove`), im Rahmen §7.8.

```rust
pub trait SegmentOwner: Send + Sync {
    fn kind(&self) -> SegmentKind;
    fn covers(&self, seg: &SegmentMeta) -> SegmentCover;     // Exclusive(ClusterId) | Mixed(Liste)
    fn rewrite_without(&self, seg: &SegmentMeta, clusters: &[ClusterId], io: &mut SegmentIo)
        -> Result<RewriteOutcome, SegError>;                  // Neubau oder Drop
    fn watermark(&self) -> Stamp;                             // bis wohin aufgeholt
    fn on_dropped(&self, seg: &SegmentMeta);                  // nur Information
}
```

Ein `Exclusive`-Segment wird beim Erase ohne Rückfrage gedroppt. Ein `Mixed`-Segment wird per `rewrite_without` neu gebaut. **Ein Erase gilt erst als `Done`, wenn alle überlappenden Besitzer bestätigt haben** (`INV-ERA-8`); ein Besitzer ohne Antwort hält den Vorgang in `Purging` (Alarm), er wird nie übersprungen. Ein Besitzer ohne `rewrite_without` darf kein `Mixed`-Segment in einem `clustered`-Space anlegen. `SegmentIo::seal/open` versiegelt unter dem Clusterschlüssel (§14.3).

### 20.3 Mehrfachbezug und abgeleitete Daten

Abgeleitete Einträge gehören **demselben Cluster** wie ihre Quelle und werden mit ihr gelöscht. Entstehen sie aus mehreren Clustern, legt der Aufrufer **je Cluster einen Anteil** ab (zum Beispiel getrennte Einträge) oder trennt personenbezogene von nicht personenbezogenen Anteilen; der Kern unterstützt **keine** Einträge mit mehreren Löschdomänen (SD-02), weil das Format sonst eine Wrap-Liste je Eintrag bräuchte und jede Lösch- und Bereichsoperation entlang mehrerer Schlüssel verzweigen würde. Verweise auf Quellen stehen in `header` (opak).

### 20.4 Kopfdaten, Gewichte, Klassen, Zugriffsereignisse

`header`, `weights[2]`, `class` (§5.4) sind das Schnittstellenfeld für Rang- und Budgetlogik. Der Kern bietet `page` mit Budget, Klassenmaske, Header-only und garantiert Körper-freie Header-Lesbarkeit. `AccessSink` (optional) meldet Lesezugriffe `(space, cluster, Blöcke)` ohne Schlüssel und ohne Inhalt, damit Aufrufer ihre `ClusterId`-Vergabe verbessern können; der Kern ordnet **nie** selbst um (NZ-6).

### 20.5 Stabilität

Ports, Werttypen und diese Verträge folgen **semver** in `contextra-storage-ports`. Formatänderungen laufen nur über §1.5 und einen Migrationsplan (SD-04).

---

## 21. Phasen und Gates

| Phase | Inhalt | Gate (Nachweis) |
|---|---|---|
| **P0 Fundament** | `ports`, `codec`, `env` (MemFs, RealFs, FaultFs, Sim), `sync`; Referenzcodierer; Lint-/CI-Gerüst; Test `env_assumptions` | **G-0:** SPEC normativ (zwei Reviewer, §23 entschieden); CI grün |
| **P1 Format** | Formate v1 (FORMAT, KEYS, Redo, Run mit Cluster-Segmenten, Blob, Catalog, Ledger, Segment-Rahmen); Goldene Vektoren; Fuzz-Ziele | **G-F Formatfreeze:** Golden, Property, Fuzz ≥ 10 Ziele, Review zu §7.9 |
| **P2 Schreibpfad** | Committer, Redo (echte Nutzlast), Memtable, Stamp-Reserve, Catalog, Recovery | **G-W:** Absturzmatrix vollständig grün, M1 geprüft, Linearisierbarkeit in der Simulation |
| **P3 Lesepfad/Runs** | Flush mit Platzierung, Runs, Cache, Views, Cursor, Page | **G-R:** Referenzmodell-Äquivalenz, H-2…H-4 gemessen |
| **P4 Wartung** | Compaction mit Platzierung, Claims, Blob-GC, Graveyard, Governor | **G-M:** M3, M5, Simulation ohne Invariantenbruch, H-5, H-12 gemessen |
| **P5 Löschung** | Cluster-Zustände, Control-Frames, ER0–ER7, Ledger, Orakel, Checkpoint/Restore, EraseRange, Consumer, Segmente | **G-E:** M2, M4, Orakel 0 Treffer in allen Matrizen, H-6…H-9, H-13 gemessen |
| **P6 Härtung** | Soak, Dauer-Fuzz, Platte-voll, Fehlerinjektion, Facade-Stabilisierung, Dokumentation | **G-P Produktionsreife:** alle INV getestet (100 %), Soak 72 h, vollständiger Messbericht (§19), Sicherheitsreview der Krypto-Nutzung |

**Regeln:** Keine Phase beginnt vor dem Gate der vorherigen. Keine Statusänderung ohne Testnachweis. Der Ist-Stand steht in `IMPLEMENTATION_SPEC.md`, nicht hier.

---

## 22. Entscheidungsregister

| ID | Entscheidung | Begründung | Verworfene Alternative |
|---|---|---|---|
| SD-01 | **Cluster** als einzige Domäne für Löschung, Schlüssel und Platzierung | eine Einheit statt Scope + Kohorte; Lokalität folgt aus der Schlüsselordnung `(cluster, key, tag↓)`; Löschen = Dateiabwurf oder Umschreiben | Scope-Mengen, Kohorten, Datei je Scope |
| SD-02 | Kein Eintrag mit mehreren Löschdomänen | Wrap-Listen und Mehrfach-Purge verkomplizieren Format und Protokoll; Aufrufer trennt Anteile | Scope-Menge 1–4 (R3) |
| SD-03 | Bewilligte Abhängigkeiten (§4.5) | Sicherheit ohne eigenes `unsafe` | eigener Baum |
| SD-04 | Eine Formatversion + Feature-Bits; Änderungen nach G-F nur als Migrationswerkzeug | kein Zweitpfad im Kern | mehrere Leseversionen |
| SD-05 | Keine Idempotenz-Token | Wiederholung über `Guard`; keine Quittungsstruktur, die Löschung berührt | Token-Tabelle mit TTL |
| SD-06 | Alles, was zu einem Cluster gehört, steht in clusterversiegelten, ortsfrei gebundenen Cluster-Segmenten | Schlüssel **und** Kopf **und** Körper werden mit einem Schlüssel unlesbar; Umschreiben ohne Schlüssel | Block-Verschlüsselung je Space (Schlüssel blieben bis Purge lesbar) |
| SD-07 | Genau eine Versiegelungsschicht je Darstellung; Einmalschlüssel aus dem Clusterschlüssel; Prüfbericht-Vorschlag CR-R3-11 verworfen (§14.3) | keine Chiffrat-Haltung in der Memtable, ortsfreie AAD erreicht dasselbe Ziel | Körper einmal versiegeln und kopieren |
| SD-08 | Redo: sektorausgerichtete Gruppen, Segmente nie wiederverwendet, `redo_mark`, Torn-Tail-Regel mit Vorausschau | unterscheidet Torn Write von Bitfäule (Prüfbericht CR-R3-03) | Abschneiden nach erstem Fehler |
| SD-09 | `stamp_limit` als Hochwassermarke im Catalog | keine Stamp-Wiederverwendung (CR-R3-01) | `next_stamp` aus wiederhergestelltem Redo |
| SD-10 | Zwei Wartungslanes plus `urgent` | Flush nie hinter Compaction | ein Job je Instanz |
| SD-11 | **Ein** Erweiterungspfad (Segment-Register), kein ExtensionBlock im Run | nichts doppelt (L-11); Löschvertrag an einer Stelle | ExtensionBlock + Segment |
| SD-12 | Kein 2PC: ein Redo-Frame, ein Catalog-Edit | Atomarität im Kern; Konsistenz mit abgeleiteten Strukturen über Consumer und Watermark | verteiltes Intent-Log |
| SD-13 | Hardlink-Checkpoints nur Opt-in | Purge erreicht Hardlink-Inodes nicht | Hardlink als Standard |
| SD-14 | XChaCha20-Poly1305, Einmalschlüssel mit Zufallssalt | kein Zählerzustand, sicher bei Umkopieren | AES-GCM mit Zähler (GCM-SIV später prüfbar, O-9) |
| SD-15 | Formale Modelle M1–M5 als Gate | Race-Freiheit sonst nur Hoffnung | nur Tests |
| SD-16 | Fsync-Fehler nie wiederholen | Fsync-Semantik | Retry-Schleifen |
| SD-17 | Restriction (rechtlich) ≠ Hold (technisch); beide Zustandswechsel als Committer-Frames; kein Override | löst den Konflikt „Datenschutz schlägt Hold“ gegen „Hold schlägt Löschung“ durch Trennung der Begriffe | `HoldPolicy::Override` |
| SD-18 | `weights`/`class`/Header im Eintrag, Aggregate im `SegIndex` | Budgetseiten ohne Körper, ohne Bedeutung zu kennen | Zusatz-Spaces der Aufrufer |
| SD-19 | Erase-Frame mit Segmentschnitt + Flush-Barriere (ER3) | Redo ohne Segment-Rewrite frei von Altdaten (CR-R3-02) | Redo-Rewrite über Stamp-Grenze |
| SD-20 | Blobs clusterrein, Packed-Runs inline, ortsfreie Blob-AAD, keine Generationen | Blob-GC kostet höchstens ein Cluster; kein Cluster-übergreifendes Umschreiben | Packed-Blobs mit Löchern |
| SD-21 | Ledger außerhalb von Catalog und Checkpoint | Restore wendet Löschungen erneut an | Ledger im Checkpoint |
| SD-22 | Kein Klartext-Hash im Kern | Rückschlüsse auf personenbezogene Daten ausgeschlossen | Inhaltsadressierung |
| SD-23 | Nach ER1 ist nie ein Fehler Antwort von `retire`; Wartezustände sind Status | Aufrufer wiederholt sonst und erzeugt zweiten Auftrag (CR-R3-04) | `Err(Blocked)` nach logischem Punkt |
| SD-24 | Views mit Maximalalter, Consumer mit Maximalrückstand | begrenzt Verlauf, Wartezeit ER7 und Speicher | unbegrenzte Pins |
| SD-25 | Consumer über stampfensterweisen Scan | kein zweites Änderungsprotokoll | persistentes Änderungslog (O-7) |
| SD-26 | Zustandsprüfung ohne Sperre über den Zähler `retire_gen` | Lesepfad ohne Sperre | Sperre je Cluster |

---

## 23. Offene Punkte (Entscheidung des Auftraggebers oder juristische Prüfung)

| # | Frage | Wirkung | Vorschlag |
|---|---|---|---|
| O-1 | Ist die Frist von 72 h für ER3–ER7 im Level `Cryptographic` tragfähig? Genügt sofortige Unlesbarkeit aller Inhalte (ER2)? | Rechtsfrage (Datenschutzbeauftragter) | konfigurierbar, Standard 72 h, Orakel-Nachweis |
| O-2 | Soll `Restriction` auch Lesen sperren dürfen (`blocks_reads`)? Welche `ReasonCode`s braucht der Aufrufer? | Rechtsabwägung des Aufrufers | Mechanismus vorhanden, Codes opak |
| O-3 | `pure_min_bytes` (8 MiB), `run_target_bytes` (64 MiB), `purge_batch_min` (64): richtig? | Dateizahl gegen Löschkosten | aus Messung H-7/H-9 ableiten |
| O-4 | Sind Nutzerschlüssel personenbezogen? Reicht P-K plus `lint_keys`? | Restexposition in `Plain|Authenticated` | `Encrypted` verpflichtend für personenbezogene Daten |
| O-5 | Wer liefert den `KeyProvider` in Produktion (HSM, Tresor)? Durabilität von `destroy_clusters` | Verfügbarkeit, AN-6 | Referenz in `crypto-ref`, Produktivprovider separat |
| O-6 | Toolchain-Pin, CI-Plattform | Gates gültig erst mit CI | CI mit gepinnter Toolchain vor P0 |
| O-7 | Reicht der stampfensterweise Consumer-Scan (H-13) oder braucht es ein Änderungslog? | Leistung und Formatumfang | messen vor G-F; Entscheidung dort |
| O-8 | Kompression (`Compressor`) in v1 an oder aus? | Platz gegen CPU; Feature-Bit vorhanden | aus, bis gemessen |
| O-9 | Clusterschlüssel-Rotation (`key_epoch`) und GCM-SIV | Formatfeld vorhanden, Verfahren fehlt | Verfahren in v2, Feld jetzt reserviert |
| O-10 | Größe der Klartext-Metadaten in `Encrypted` (Cluster-ID, Größen, Stamps) | Rechtsfrage und Dateiformat | Format bleibt; Prüfung durch Datenschutzbeauftragten vor G-F |
| O-11 | Sind opake Cluster-IDs im Ledger und Catalog nach Löschung personenbezogen? | Aufbewahrung von `Purged`-Spuren | Entfernung nach `floor ≥ S`; Ledger-Aufbewahrung rechtlich klären |

---

## Anhang A — Referenzvektoren (Pflicht vor G-F)

Je Format mindestens: leeres Objekt; ein Eintrag; Cluster mit mehreren Blöcken; zwei Cluster in einem Packed-Run; Tombstone; Bereichsmarker (`RangeDelete`, `EraseRange`); Blob-Record; Catalog-Edit-Kette mit Snapshot; Ledger-Kette; Redo-Gruppe mit Commit-, Erase- und Control-Frame; abgeschnittene Gruppe; Gruppe mit Bitflip in der Mitte; byte-identisch verschobenes Cluster-Segment (Entschlüsselung vor und nach dem Verschieben liefert gleichen Klartext). Erzeugung durch `tools/reference/generator.py` (unabhängig von Rust), Vergleich im Test `golden`.

## Anhang B — Beispielablauf: Löschung zweier Cluster einer Gruppe (`Encrypted`, `Cryptographic`)

1. Aufrufer: `eraser().retire(&[c1, c2], req{Cryptographic, Wait(60 s), 72 h})`.
2. **ER0:** keine Restriction, Provider erreichbar. **ER1:** Erase-Frame (Stamp `S`) als erstes Frame eines neuen Segments, Gruppen-Fsync. Ab jetzt `Erased` für alle Lesezugriffe auf c1, c2; `retire` kehrt mit Ticket zurück.
3. **ER2:** `destroy_clusters([c1, c2])`; `probe` = `Absent`. Status `KeyDestroyed`: Kopf, Schlüssel und Körper beider Cluster sind in Redo, Runs, Blobs und Backups unlesbar. Der Aufrufer darf das **jetzt** melden.
4. Innerhalb der Frist, gebündelt: **ER3** Flush-Barriere, ältere Redo-Segmente verworfen; **ER4/ER5** Pure-Dateien von c1/c2 entfallen; die Packed-Datei mit beiden Nachbarn wird einmal umgeschrieben (andere Cluster byte-identisch kopiert); **ER6** ein Edit; **ER7** Unlink, `sync_dir`, Ledger `Done`, `ErasureReceipt`.
5. `verify(ticket)` und (in Tests) das Orakel bestätigen 0 Treffer.

## Anhang C — Zustandsdiagramme

```
Cluster:  Live ⇄ Restricted(reason, blocks_reads)
          Live | Restricted(frei) ──ER1──► Retired(S) ──ER6──► Purged(S) ──(floor ≥ S)──► (Eintrag entfällt)
Erase:    Retired ─► KeyDestroyed ─► Purging{Redo|Plan|Rewrite|AwaitingViews|Owners} ─► Done(Receipt)   (Absturz: fortsetzen, nie zurück)
Datei:    Building ─rename+Edit─► Live ─Edit─► Obsolete ─letzte Version-Referenz─► Unlinked
Redo-Seg: Open ─Siegel─► Sealed ─redo_floor─► Obsolete ─► Graveyard ─► Unlinked        (nie wiederverwendet)
```

## Anhang D — Einarbeitung der R3-Befunde

| Befund | Behandlung in R5 |
|---|---|
| CR-R3-01 Stamp-Wiederverwendung | `stamp_limit`, `INV-STAMP-3` (§9.2) |
| CR-R3-02 Altdaten im Redo | Segmentschnitt, Flush-Barriere ER3, `INV-LOG-4/5` (SD-19) |
| CR-R3-03 Torn-Tail vs. Bitfäule | `INV-LOG-7/8`, sektorausgerichtete Gruppen, keine Segmentwiederverwendung (§7.4, §12.1) |
| CR-R3-04 `Wait` nach logischem Punkt | nach ER1 nie Fehler, Status (§13.3, SD-23) |
| CR-R3-05, -12 Blob-Schlüssel/-MAC uneinheitlich | Blob-Record einheitlich über §14.3, Domäne `0x03`, ortsfreie AAD |
| CR-R3-06 FORMAT-Authentisierung | §7.1/§7.2: CRC32C plus Mac16 für `FORMAT` |
| CR-R3-07 Redo-Frame-Grenze | 32 MiB (§5.5) |
| CR-R3-08 Prüfreihenfolge | §12.1 R1/R2 |
| CR-R3-09 Gesamtdeckel Memtable | `memtable_total_bytes` (§9.3) |
| CR-R3-10 Typbreiten | `segment_id < 2^32` (§5.5, §7.4) |
| CR-R3-11 Körper einmal versiegeln | geprüft, verworfen, Begründung in §14.3 (SD-07) |
| Namenskollisionen `E-n`, `A-n`, `M-nn` | Präfixe `ER`, `AN`, `M` (§1.4) |

# SPEC.md — `contextra-storage` (Speicherkern), Revision R5

| | |
|---|---|
| Status | **Abnahmekandidat R5** (Zusammenführung aus R4-Entwurf und R3-Entwurf, §25.2). Normativ erst mit Gate **G-0** (§24): zwei Reviewer, Punkte aus §26 entschieden. Bis dahin ist jeder Abschnitt änderbar, danach gilt §1.5. |
| Gegenstand | Eingebetteter, synchroner, crash-sicherer Schlüssel-Wert-Kern mit atomaren Mehr-Space-Änderungen, MVCC-Sichten, **löschlokaler Datenanordnung** und kryptografisch plus physisch nachweisbarer Löschung. Crates: `contextra-storage` (Kern), `contextra-storage-ports` (Ports), `contextra-crypto-ref` (Referenz-Primitive, nur Tests). |
| Sprache | Prosa und Rustdoc deutsch, Bezeichner englisch (P-7). |
| Verifikation | Erstellt durch Lesen von `docs/`, R2, dem R4-Entwurf (nur bis §11.1 vorliegend) und eigener Ausarbeitung. **Nichts kompiliert, nichts gemessen.** Alle Zahlen sind Hypothesen (`H-*`, §23). Formate brauchen vor **G-F** Referenzvektoren aus dem unabhängigen Referenzcodierer (Anhang A). |
| Rechtlicher Vorbehalt | Der Kern liefert technische Eigenschaften. Er trifft keine Rechtsaussage zu Art. 17 DSGVO, Fristen oder Ausnahmen (§26). |

---

## 1. Geltungsbereich und Konventionen

### 1.1 Zweck

`contextra-storage` speichert Daten und Informationen für Sprachmodell-Anwendungen: viele kleine atomare Schreibvorgänge, kurzlebige Sichten, Lesen von Kopfdaten ohne Körperzugriff, budgetierte Ergebnisseiten, Verlauf mit Fortschrittszeigern und **Löschung nach DSGVO als Entwurfsprinzip, nicht als Nachrüstung**. Der Kern kennt weder Tokenizer, Indexstrukturen, Mandanten, Rechte noch Rechtsgrundlagen. Er liefert Mechanismus, nie Politik (§3.3).

### 1.2 Normsprache

**MUSS / DARF NICHT** zwingend. **SOLL / SOLL NICHT** Regelfall, Abweichung braucht Begründung im Review. **KANN** optional.

### 1.3 Dokumentenhierarchie

1. `SPEC.md` normativ (Formate, Protokolle, Invarianten, API-Vertrag).
2. `IMPLEMENTATION_SPEC.md` operativ (Phasen, Gates, Ist-Stand). Widerspruch ⇒ SPEC gilt.
3. Alles andere ist Herkunft und keine Anforderungsquelle.

### 1.4 Kennungen

`Z-n` Ziel · `NZ-n` Nicht-Ziel · `P-n` Codierregel · `F-n` Fehlerklasse · `AT-n` Angreifer · `AN-nn` Annahme · `SD-nn` Entscheidung (§25) · `INV-*` Invariante (§22) · `H-n` Leistungshypothese (§23) · `O-n` offene Entscheidung (§26) · `G-*` Gate (§24) · `ER0…ER7` Schritte der Löschung (§17.5). Die Kollisionen des Altstands (`E-n`, `A-n`, `M-nn`) sind aufgehoben.

### 1.5 Änderungsprozess

Änderungen an §8, §11, §12, §15, §17, §18, §22 (Formate, Protokolle, Invarianten) brauchen zwei Reviewer, aktualisierte Referenzvektoren (Anhang A) und, wenn ein Format betroffen ist, eine Prüfung gegen die Freeze-Liste (§11.9). Nach **G-F** ist jede Formatänderung ein Migrationswerkzeug (SD-04), nie ein Zweitpfad im Kern.

---

## 2. Leitentscheidungen

| # | Entscheidung | Wirkung |
|---|---|---|
| L1 | **Cluster** (`ClusterId`, 16 Byte, opak) ist Löschdomäne, Schlüsseldomäne und Platzierungseinheit zugleich. | Eine Einheit für Löschung, Kryptografie und Lokalität. |
| L2 | Reihenfolge in geclusterten Spaces: `(cluster, user_key, tag↓)`. Datenblöcke sind **clusterrein**, die Daten eines Clusters liegen in jeder Datei als **ein zusammenhängender Abschnitt**. | Löschen ist Dateiabwurf (Pure) oder Abschnitt überspringen und Rest roh kopieren (Packed). |
| L3 | In verschlüsselten, geclusterten Spaces ist **alles** eines Clusters (Schlüssel, Kopf, Körper, Index, Filter, Bereichsmarker) mit dem Clusterschlüssel versiegelt (§17.3). | Schlüsselvernichtung macht den gesamten Cluster unlesbar, auch Nutzerschlüssel und Kopfdaten. |
| L4 | Dateien sind **Pure** (ein Cluster) oder **Packed** (mehrere Cluster, nach **Packgruppe** getrennt: Mandanten-Tag × Ablaufzeitraum). | Skaliert auf Millionen Cluster. Ablauf einer Aufbewahrung = Dateiabwurf der Packgruppe. |
| L5 | **Ein Schreiber** (Committer, Gruppen-Fsync), unveränderliche Dateien, unveränderliche `Version` per `Arc`. | Race-Freiheit durch Konstruktion (§19). |
| L6 | **Catalog ist die einzige Wahrheit.** Eine Datei ist lebendig genau dann, wenn ein durabler Edit sie nennt. | Crash-Sicherheit, Verwaisten-Suche. |
| L7 | **Reihenfolge ist Vertrag:** Gate vor Schlüsselvernichtung vor Dateiwechsel vor Unlink. Nur Roll-forward. | §17.5. |
| L8 | **Cluster-Links:** Ein abgeleiteter Cluster kann an Quell-Cluster gebunden werden. Löschung eines Quellclusters zieht den abgeleiteten Cluster **atomar** mit (nur auf ausdrücklichen Auftrag der oberen Schicht). | Abgeleitete Daten ohne Mehrfach-Scope-Einträge (§18.2). |
| L9 | **Restriction** (rechtlicher Sperrvermerk mit Pflichtgrund) verhindert Löschung durch den Kern. Der Kern kennt keinen Override. | Mechanismus statt Politik (§16.6). |
| L10 | **Stamps werden nie doppelt vergeben**, auch nicht nach Absturz (Reservierung im Catalog). | `INV-STAMP-3`. |
| L11 | Kopfdaten (`header`) sind ohne Körper lesbar. **Gewichte** (`weights`) stehen im Klartext des Eintrags und als Aggregat im Block-Index. | Budgetseiten ohne Körperzugriff. |
| L12 | **Fünf Erweiterungsverträge** für obere Schichten (§18), ohne Formatbruch. | Anbindung später ohne Umbau. |
| L13 | **Exklusive Instanzsperre** (`LOCK`) und `Perm::Private` für alle Dateien. | Verhindert Doppelöffnung und Fremdzugriff. |
| L14 | **Formale Modelle** (TLA+) und deterministische Simulation sind Gate-Bestandteil (§20, §24). | Race-Freiheit wird bewiesen oder gemessen, nicht angenommen. |

---

## 3. Architektur

### 3.1 Crates und Schichten

```
contextra-storage-ports   Traits und Werttypen, keine Logik, keine Laufzeit-Abhängigkeiten
        ▲
contextra-storage         Kern, nur std (+ ports + bewilligte Abhängigkeiten §3.5), #![forbid(unsafe_code)]
        ▲ (nur dev-dependency)
contextra-crypto-ref      Referenz für Cipher/Mac/Kdf
```

Plattformnahes (Direkt-E/A, Hardware-CRC, Speicherschutz) ist **Port-Implementierung in eigenen Crates** und berührt den Kern nie.

### 3.2 Modulschnitt

Abhängigkeiten nur nach unten, per Test `layering` erzwungen:

```
api        Facade: Storage, Space, View, Change, Cursor, Hold, Restriction, Consumer, Eraser, Checkpointer, Inspector
engine     committer · memtable · version · views · registry · claims · retention
maint      flush · compaction · blob_gc · purge · pack_drop · scheduler (Governor)
erase      gate · cluster_table · links · protocol · ledger · oracle (Feature testing)
recovery   open-Sequenz, Absturzmatrix
catalog    snapshot · edits · log · rollover · current · reservations
store      run · blob · segment · redo
keys       cluster_keys · key_cache · generations
codec      varint · tags · checksums · comparator
env        Fs · Clock · Entropy · Governor · Metrics (MemFs, RealFs, FaultFs, Sim)
sync       Sperren mit Rang, Epoche
```

`std::fs` nur in `env/real_fs.rs`.

### 3.3 Mechanismus, nie Politik

Der Kern entscheidet nie, **ob** gelöscht werden darf oder muss, **wer** ein Recht hat, **welche** Frist gilt oder **was** ein Cluster bedeutet. Er setzt Aufträge atomar, ordnungsgemäß und nachweisbar um und verweigert sie, wenn ein technischer oder (über `Restriction`) ein von der oberen Schicht gesetzter Hinderungsgrund vorliegt. Politik bleibt oben: Betroffenenanfragen, Fristen, Rechtsgründe, Saga, Berichte, Signaturen.

### 3.4 Threading

- **Ein Committer** (Rang 1): Flat-Combining, Gruppen-Fsync.
- **Lesen:** lock-frei bis auf `Arc`-Klone. Eine `View` hält `Arc<Version>`.
- **Wartung:** Lanes `urgent`, `flush`, `compact`, `blobgc`, `idle` mit Prioritäten (§14.1). Flush wartet nie hinter Compaction.
- **Threads gehören dem Aufrufer** oder einem übergebenen `Spawner`. Der Kern startet keinen globalen Pool.

### 3.5 Bewilligte Abhängigkeiten (SD-03)

| Zweck | Wahl | Begründung / Alternative |
|---|---|---|
| Memtable | `crossbeam-skiplist` hinter Trait `Memtable` | lock-freie Lesevorgänge ohne eigenes `unsafe`; Alternative: eigener Baum hinter demselben Trait |
| Puffer | `bytes` | Zero-Copy |
| Hash/MAC | `blake3` | schnell, in `crypto-ref` vorhanden |
| AEAD | `chacha20poly1305` (XChaCha) | in `crypto-ref` vorhanden |
| Zeroize | `zeroize` | Schlüsselmaterial |
| Test (dev) | `loom`, `shuttle`, `proptest` | §20 |

Jede weitere Abhängigkeit braucht einen Registereintrag und `cargo-deny` mit `[advisories]`, `[bans]`, `[sources]`.

---

## 4. Ziele, Nicht-Ziele, Bedrohungsmodell

**Ziele:** Z-1 Atomarität über Spaces. Z-2 Durability. Z-3 Snapshot-Isolation ohne Lesesperren. Z-4 Löschung eines Clusters in Zeit unabhängig von der Gesamtgröße. Z-5 Ablauf einer Packgruppe als O(Dateien)-Operation. Z-6 Nachweisbarkeit (Receipt, `verify`, Orakel). Z-7 Kopf ohne Körper lesen. Z-8 Race-Freiheit. Z-9 begrenzte Ressourcen. Z-10 stabile Verträge nach oben.

**Nicht-Ziele:** NZ-1 Verteilung, Replikation. NZ-2 eigene Kryptografie. NZ-3 Schemas, Sekundärindizes. NZ-4 Rechtsbewertung. NZ-5 Schutz von Prozessspeicher, Swap, Core-Dateien (best-effort, §17.9). NZ-6 Umordnung durch den Kern.

**Angreifer:** AT-1 Prozess mit Lesezugriff auf das Datenverzeichnis, ohne Schlüssel. AT-2 Besitzer des Datenträgers (Diebstahl, Backup-Kopie), ohne Schlüssel. AT-3 Manipulation von Dateien (Einfügen, Abschneiden, Vertauschen). AT-4 Fehlerhafte oder bösartige obere Schicht (falsche Cluster, Missbrauch der API).

**Annahmen:** AN-01 `sync_data`/`sync_all`/`sync_dir` machen Änderungen durabel (POSIX). AN-02 Ein Schreibvorgang kann zerreißen (Torn Write); alle Rahmen sind prüfsummengeschützt. AN-03 Ein Fsync-Fehler ist **nicht** wiederholbar. AN-04 Verzeichnis-`rename` ist atomar. AN-05 Prozessspeicher ist vertrauenswürdig. AN-06 `KeyProvider::destroy` ist durabel bei Rückkehr. AN-07 Zufall aus `Entropy` ist kryptografisch stark.

---

## 5. Ports (in `contextra-storage-ports`, semver-stabil)

| Port | Aufgabe | Vertrag |
|---|---|---|
| `Fs` | Dateien, Verzeichnisse, `read_at`, `append`, `preallocate`, `sync_data`, `sync_all`, `sync_dir`, `rename`, `lock`/`unlock`, `list`, optional `link`, `reflink`, `punch_hole` | Fehlerklassen §6, `lock` exklusiv oder geteilt, `Perm::Private` |
| `Clock` | monotone Zeit, Wanduhr | injizierbar |
| `Entropy` | kryptografischer Zufall | injizierbar |
| `KeyProvider` | KEK, Clusterschlüssel, `destroy`, `probe` | §17.2 |
| `Checksum` | CRC32C | Standard sicher (Slice-by-16), Hardware austauschbar |
| `Cipher`/`Mac`/`Kdf` | AEAD, Kettenprüfung, Ableitung | §17.1 |
| `Metrics` | Zähler, Histogramme | nicht blockierend |
| `Observer` | Ereignisse (Flush, Purge-Frist, Poison, Restricted-Read) | nie unter Sperren aufgerufen |
| `ColdTier` (Option) | Auslagerung kalter Dateien | Garantien des Ports gelten (§17.9) |

---

## 6. Fehlermodell

| Klasse | Beispiele | Verhalten |
|---|---|---|
| F-1 transient | `EINTR`, kurze Überlast | Wiederholung mit Backoff, begrenzt |
| F-2 Ressource | `ENOSPC`, Handle-Limit | Rückdruck, dann **Poison** des Schreibpfads; Lesen bleibt |
| F-3 E/A-hart | `EIO`, Fsync-Fehler | **nie wiederholen**, Poison, Wiederanlauf nur über Neustart/Recovery (SD-16) |
| F-4 Integrität | CRC/MAC-Fehler | kleinste isolierbare Einheit wird vergiftet (Block → Abschnitt → Datei → Space), `Err(Corrupt{..})`, Rest läuft weiter |
| F-5 Format | `Unsupported`, unbekannter Catalog-Tag | Öffnen verweigert, nie ignorieren (`INV-FMT-1`) |
| F-6 Schlüssel | Provider offline, Schlüssel fehlt | `LockedKeys`-Modus bzw. `Shredded` |
| F-7 Aufrufer | Guard verletzt, Cluster gesperrt, Größe, Restricted | typisierte Fehler mit `hint()` |
| F-8 Logik | Invariante verletzt (Bug) | Test: Abbruch. Produktion: Poison + Observer. **Kein `panic`** im Produktionspfad |

Jeder Fehler hat `class()` und `hint()` (Handlungstext, auch für Aufrufer, die ein Modell sind).

---

## 7. Datenmodell

### 7.1 Vokabular

Im Crate kommen weder Produkt- noch Fremdsystemnamen vor.

| Begriff | Bedeutung |
|---|---|
| **Storage** | Handle einer geöffneten Instanz |
| **Space** | Benannter, physisch getrennter LSM-Baum (`SpaceId(u16)`) mit eigenem Verzeichnis |
| **Cluster** | Löschdomäne, Schlüsseldomäne und Platzierungseinheit (`ClusterId`, 16 Byte, opak). `ClusterId::NONE` (nur Nullbytes) heißt „kein Cluster“. Cluster werden über `Clusters::register` angelegt, nie erfunden. |
| **Packgruppe** | Gruppe von Clustern gleichen Mandanten-Tags und gleichen Ablaufzeitraums (`PackKey`); Packed-Dateien enthalten nur Cluster **einer** Packgruppe |
| **Change / Guard / Receipt** | Atomare Menge von Schreibvorgängen / Vorbedingung / Quittung `(stamp, durable)` |
| **Stamp** | Monotone Zählung angenommener Changes (`u64` < 2^56) |
| **View / Cursor / Page** | Unveränderliche Sicht auf einen Stamp / Streaming-Iterator / budgetierte Ergebnisseite mit `ResumeToken` |
| **Hold** | Befristete **technische** Halteregel gegen das Verwerfen alter Versionen, mit Eigentümer |
| **Restriction** | **Rechtlicher** Sperrvermerk auf einem Cluster mit Pflichtgrund; verhindert Löschung (und optional Lesen) durch den Kern (§16.6) |
| **Consumer** | Benannter persistenter Fortschrittszeiger; zählt wie ein Hold, mit Maximalrückstand |
| **Redo-Log** | Segmentiertes Wiederholungsprotokoll aus ausgerichteten Gruppen |
| **Run / Blob** | Unveränderliche sortierte Datei mit Kopfdaten / unveränderliche Datei mit Wertkörpern |
| **Header / Body / Weights** | Erste `header_len` Wertbytes / Rest / Zählgrößen im Klartext |
| **Sealed** | Mit einem Einmalschlüssel aus dem Clusterschlüssel versiegelt (§17.3) |
| **Retired** | Cluster, dessen Löschung logisch erfolgt ist (Stamp `S`) |
| **Pure / Packed** | Datei mit einem Cluster / Datei mit mehreren Clustern einer Packgruppe (§8) |
| **Segment** | Vom Catalog verwaltete Datei einer oberen Schicht mit Kern-Rahmen (§18.1) |
| **Claim / Graveyard** | Reservierung von Dateien durch einen Wartungsjob / Warteschlange zu löschender Dateien |
| **Catalog** | Einzige Wahrheit über lebendige Dateien und Zustand |
| **Env** | Bündel der Ports |

### 7.2 Cluster, Schlüssel, Reihenfolge, Stamp

- **Nutzerschlüssel:** beliebige Bytes, 1 ≤ Länge ≤ `max_key_bytes`. In verschlüsselten, geclusterten Spaces ist er durch Versiegelung geschützt (L3). In allen anderen Spaces MUSS er frei von personenbezogenen Daten sein (Vertrag §17.9).
- **Vergleich** (Funktion `cmp`, **kein** `memcmp` über Konkatenation, wegen variabler Schlüssellängen): Cluster aufsteigend (bytelexikografisch), dann Nutzerschlüssel aufsteigend (bytelexikografisch), dann **Tag absteigend**. In Spaces ohne `clustered()` ist der Cluster implizit `NONE` und wird nicht gespeichert.
- **Tag** `u64 = (stamp << 8) | kind`. `kind`: `Put = 1`, `Delete = 2`, `RangeDelete = 3`, `EraseRange = 4`, `PutRef = 5`, `Merge = 6` (**reserviert**, ein Decoder, der `6` sieht, meldet `Unsupported`).
- `PutRef` kommt **nur in Runs** vor. Redo und Memtable kennen nur `Put`.
- **Stamp:** `u64` < 2^56, erster Stamp 1, `0` = leerer Zustand. Streng monoton über **angenommene** Changes (`INV-STAMP-1`). **Kein Stamp wird je zweimal vergeben, auch nicht nach Absturz** (`INV-STAMP-3`, §12.3). Lücken sind zulässig. Überlauf ⇒ `Error::Exhausted`, nie Umbruch (`INV-STAMP-2`).
- **Bereichsmarker** (`RangeDelete`, `EraseRange`): `(cluster, start, end, tag)`, `end` exklusiv. Flag `UNBOUNDED_END`: bis zum Ende des Clusters. Flag `WHOLE_CLUSTER`: alle Schlüssel des Clusters. Ein Marker mit Stamp `S` verdeckt für jede Sicht ≥ `S` alle Versionen mit Stamp < `S` im Bereich. Marker liegen in Runs im `RangeDelBlock` des Clusterabschnitts (§11.5).

### 7.3 Wert, Kopf, Körper, Gewichte

Ein Wert besteht aus `header` und `body`; `header_len` trennt sie. **Der Kern interpretiert den Kopf nie.** Er speichert ihn und liefert ihn ohne Körper-E/A. Zusätzlich trägt jeder Eintrag `weights: [u32; W]` (`W` = `weight_slots` des Spaces, 0…4, unveränderlich). Die obere Schicht legt fest, was ein Gewicht bedeutet (zum Beispiel Token). Im Block-Index stehen je Block `weight_sum[W]` und `entry_count`. `scan_budget` (§9) überspringt Blöcke damit ohne Dekomprimierung.

Körper ≥ `value_separation_min` (Standard 1 KiB) gehen in Blobs (`PutRef`); kleinere bleiben inline im Block. Kopf liegt **immer** inline.

### 7.4 `SpaceOptions` (unveränderlich nach dem Anlegen)

```rust
pub struct SpaceOptions {
    pub clustered: bool,                    // true: Schlüsselordnung und Abschnitte nach Cluster (L2)
    pub packing: PackPolicy,                // None | ByTenantAndExpiry { bucket: Month|Quarter|Year } ; nur bei clustered
    pub pure_threshold: u64,                // Cluster ≥ Schwelle wird bei Compaction Pure (Standard 8 MiB)
    pub seal: Seal,                         // None | Cluster   (Cluster nur bei clustered + integrity Encrypted)
    pub weight_slots: u8,                   // 0..=4
    pub history: HistoryPolicy,             // KeepLatest | KeepUntilFloor | AppendOnly (Erase nur mit Admin)
    pub value_separation_min: u32,
    pub max_key_bytes: u32,
    pub compression: Compression,           // None | Lz4 | Zstd(n) (Port/Feature); vor Versiegelung
    pub durability_default: Durability,
}
```

`integrity_mode` steht **global** in `FORMAT` (§11.2). `Seal::Cluster` ist nur in `Encrypted` zulässig. Kryptografische Löschung (`EraseLevel::Cryptographic`) ist nur für Spaces mit `Seal::Cluster` möglich (sonst `Err(Unsupported)`).

### 7.5 Cluster-Tabelle (Catalog)

Je Cluster: `{ cluster_id, tenant_tag, pack_key, state, key_epoch, created_stamp, due_bucket, links, restrictions[] }`.

`state`: `Live → Retired(S) → Shredded → Purged` (nur vorwärts). `Restricted` ist **kein** Zustand, sondern eine Menge `restrictions` (orthogonal, §16.6). Ein `Purged`-Cluster wird **nie** wiederverwendet (`INV-CLUSTER-1`).

---

## 8. Platzierung: Pure und Packed

### 8.1 Regeln

- **R-PL-1** Jeder Run, Blob und jedes Segment ist **Pure** (genau ein Cluster) oder **Packed** (mehrere Cluster derselben Packgruppe) und nie beides. `INV-PL-1`.
- **R-PL-2** Innerhalb einer Packed-Datei bildet jeder Cluster **einen zusammenhängenden Abschnitt** (`ClusterSection`), sortiert nach `ClusterId`. Blöcke sind clusterrein (`INV-PL-2`).
- **R-PL-3** Ein Cluster, dessen lebende Bytes in seiner Packgruppe `pure_threshold` überschreiten, wird bei der nächsten Compaction **Pure** (eigene Dateien, eigene Wartungsgruppe). Der umgekehrte Weg (Pure → Packed) findet nicht statt.
- **R-PL-4** Compaction mischt nur innerhalb **einer Wartungsgruppe**: einer Packgruppe (alle Packed-Dateien) oder eines Pure-Clusters. `INV-PL-3`.
- **R-PL-5** Eine Packgruppe wird über `PackKey = (tenant_tag, due_bucket, shard)` bestimmt. Die Anzahl Packgruppen je Space ist begrenzt (`max_pack_groups`, Standard 4096).
- **R-PL-6** Der Wechsel eines Clusters in eine andere Packgruppe (`rebucket`, zum Beispiel verlängerte Aufbewahrung) ist ein atomarer Umzug (§17.8).

### 8.2 Wirkung auf die Löschung

| Fall | Pure | Packed |
|---|---|---|
| Cluster löschen | Datei(en) **entfernen**, ein Edit | Abschnitt **überspringen**, übrige Abschnitte **roh kopieren** (kein Entschlüsseln, kein Neukomprimieren), Verzeichnis neu, ein Edit |
| Packgruppe läuft ab | alle Packed-Dateien der Gruppe entfernen, ein Edit | |
| Mandant verlässt | alle Pure-Cluster und Packgruppen des Tags entfernen | |

Weil Blöcke und Abschnitte clusterrein sind und ihre AAD **nicht** an Dateiposition gebunden ist (§17.3), ist das Rohkopieren gültig.

---

## 9. Öffentliche API

Normativ in Form und Semantik, Nebensächlichkeiten (Lebensdauern) offen.

```rust
// ---- Öffnen und Struktur -------------------------------------------------------
pub struct Storage { /* Arc<Inner> */ }
impl Storage {
    pub fn open(cfg: Config, env: Env) -> Result<Storage, OpenError>;     // LOCK, Recovery; Mode ReadWrite | ReadOnly(shared)
    pub fn space(&self, name: &str) -> Result<Space, Error>;
    pub fn spaces_add(&self, name: &str, o: SpaceOptions) -> Result<Space, Error>;
    pub fn close(self) -> Result<CloseReport, Error>;
}

// ---- Schreiben: ein Change über viele Spaces, atomar ---------------------------------
pub struct Change { /* Ops + Guards */ }
impl Change {
    pub fn new() -> Change;
    pub fn put(&mut self, sp: &Space, c: ClusterId, key: &[u8], v: NewValue) -> &mut Self;
    pub fn delete(&mut self, sp: &Space, c: ClusterId, key: &[u8]) -> &mut Self;
    pub fn delete_range(&mut self, sp: &Space, c: ClusterId, r: KeyRange) -> &mut Self;
    pub fn guard(&mut self, sp: &Space, c: ClusterId, key: &[u8], g: Precondition) -> &mut Self; // Absent | Version(Stamp) | Unchanged
    pub fn idempotency(&mut self, t: IdemToken) -> &mut Self;            // Wiederholung → AlreadyApplied { stamp }
}
pub struct NewValue { pub header: Bytes, pub body: Body, pub weights: [u32; 4] }
impl Storage {
    pub fn apply(&self, c: Change, d: Durability) -> Result<Receipt, ApplyError>;
    // ApplyError: GuardFailed, AlreadyApplied, ClusterRetired, Restricted, Backpressure(Retry), Poisoned, Exhausted, ...
}

// ---- Lesen -----------------------------------------------------------------------
impl Storage { pub fn view(&self) -> View;  pub fn view_at(&self, s: Stamp) -> Result<View, Error>; }
impl View {
    pub fn stamp(&self) -> Stamp;
    pub fn get_header(&self, sp: &Space, c: ClusterId, key: &[u8]) -> Result<Option<HeaderRef>, ReadError>;
    pub fn get(&self, sp: &Space, c: ClusterId, key: &[u8]) -> Result<Option<EntryRef>, ReadError>; // Körper lazy
    pub fn cursor(&self, sp: &Space, c: ClusterId, r: KeyRange, o: ScanOpts) -> Cursor;            // Streaming, Header zuerst
    pub fn page(&self, sp: &Space, c: ClusterId, r: KeyRange, budget: Budget, resume: Option<ResumeToken>)
        -> Result<Page, ReadError>;                                                                // nutzt weight_sum, überspringt Blöcke
    pub fn history(&self, sp: &Space, c: ClusterId, key: &[u8]) -> History;
}
impl EntryRef { pub fn header(&self) -> &[u8];  pub fn weights(&self) -> &[u32];
                pub fn body(&self) -> Result<Body, BodyError>; }     // Err(Shredded|Erased{..}), nie Daten, nie leer

// ---- Cluster und Verknüpfung -----------------------------------------------------------
impl Storage { pub fn clusters(&self) -> Clusters; }
impl Clusters {
    pub fn register(&self, spec: ClusterSpec) -> Result<ClusterId, Error>;           // Tenant-Tag, Ablauf (due), Klasse optional
    pub fn link(&self, derived: ClusterId, sources: &[ClusterId]) -> Result<(), Error>; // ≤ 8, azyklisch, nur auf Auftrag
    pub fn rebucket(&self, c: ClusterId, new_due: Due) -> Result<RebucketReport, Error>;
}

// ---- Holds, Restrictions, Consumer -------------------------------------------------------
impl Storage {
    pub fn hold(&self, at: Stamp, owner: OwnerId, ttl: Duration) -> Result<Hold, Error>;
    pub fn restrict(&self, c: ClusterId, r: RestrictionSpec) -> Result<RestrictionId, Error>;   // Pflichtgrund, Eigentümer, Effekte
    pub fn lift(&self, id: RestrictionId, by: OwnerId) -> Result<(), Error>;
    pub fn consumer(&self, id: ConsumerId, from: Stamp, max_lag: u64) -> Result<Consumer, Error>;
}
impl Consumer { pub fn next(&mut self, max: usize) -> Result<Changes, Error>;  pub fn ack(&mut self, upto: Stamp) -> Result<(), Error>; }

// ---- Löschung -------------------------------------------------------------------------------
impl Storage { pub fn eraser(&self) -> Eraser; }
impl Eraser {
    pub fn cluster(&self, c: ClusterId, req: EraseReq) -> Result<Ticket, EraseError>;             // ER0–ER1 synchron, Rest asynchron
    pub fn clusters_batch(&self, v: &[ClusterId], req: EraseReq) -> Result<Ticket, EraseError>;
    pub fn pack_expired(&self, k: PackKey, req: EraseReq) -> Result<Ticket, EraseError>;
    pub fn range(&self, sp: &Space, c: ClusterId, r: KeyRange, req: EraseReq) -> Result<Ticket, EraseError>;
    pub fn status(&self, t: &Ticket) -> EraseStatus;           // Retired · KeyDestroyed · Purging · Done(ErasureReceipt)
    pub fn wait(&self, t: &Ticket, until: EraseStage, timeout: Duration) -> Result<EraseStatus, Error>;
    pub fn verify(&self, t: &Ticket) -> VerifyReport;
}
pub struct EraseReq { pub level: EraseLevel,                   // Logical | Physical | Scrubbed | Cryptographic
                      pub views: ViewPolicy,                   // Wait(timeout) | Revoke
                      pub purge_deadline: Duration }           // PFLICHT, kein Standardwert (§21)

// ---- Checkpoint, Betrieb, Prüfung -------------------------------------------------------------
impl Storage {
    pub fn checkpoint(&self, dst: &Path, m: CheckpointMode) -> Result<Manifest, Error>;     // Copy | Reflink | Hardlink(Opt-in)
    pub fn restore_check(&self, m: &Manifest, ledger: &LedgerRef) -> RestorePlan;
    pub fn maintenance(&self) -> Maintenance;                  // Pause, Resume, Lanes, Governor
    pub fn inspect(&self) -> Inspector;                        // Dateien, Packgruppen, Clusterstatistik, Kennzahlen
}
```

**Eigenschaften:**

- Kein Aufruf blockiert unbegrenzt. Wartefunktionen haben Timeouts, Rückdruck ist `Backpressure(Retry)` mit Hinweis.
- `apply` ist über `IdemToken` idempotent. Das Token liegt im selben Redo-Frame.
- Eine `View` ist `Send + Sync`, billig klonbar, hält keine Sperre.
- `body()` unterscheidet `Shredded` (Schlüssel weg) von `Erased` (physisch entfernt). Beides liefert nie Daten.
- Ein Nutzer ohne Recht erhält **nicht** diesen Unterschied; das verbirgt die obere Schicht (Rechteprüfung liegt dort, §3.3).

---

## 10. Dateilayout und Dateilebenszyklus

### 10.1 Layout

```
<root>/
  FORMAT              Magic, Formatversion, Feature-Bits, integrity_mode, instance_id (§11.2)
  LOCK                exklusive (oder geteilte, read-only) Sperre über Fs::lock
  CURRENT             Zeiger auf die aktive Catalog-Datei (atomar ersetzt)
  CATALOG-000007      Snapshot-Record + Edit-Records (§11.8)
  KEYS                (Authenticated/Encrypted) gesiegelte Schlüsselhierarchie (§17.2), atomar ersetzt
  ERASE-LEDGER/       append-only Löschprotokoll, kettenverknüpft, getrennt vom Catalog (§17.8)
  redo/
    redo-0000000000000012.log        vorallokiert, am Ende gesiegelt (§11.4)
  spaces/
    0001/
      0000000000000a3f.run           Pure- oder Packed-Datei (Art steht im Datei-Kopf und im Catalog)
      0000000000000071.blob
    0002/ …
  segments/<kind:04x>/<id:016x>.seg  Dateien oberer Schichten (§18.1), vom Catalog verwaltet
  graveyard/                         (logisch; physisch dieselben Verzeichnisse, Zustand im Catalog)
```

### 10.2 Regeln

- **Numerische IDs** benennen alle Dateien, nie Pfad-Strings im Catalog.
- **Blob-Ersetzung ohne Referenz-Umschreibung:** ein `BlobRef = (blob_id, offset, len)`. Ein Blob wird nie verändert. Soll ein Cluster-Abschnitt entfallen, entsteht eine **neue Inkarnation** derselben `blob_id` (`<id>.<inc>.blob`) mit **gleichen Offsets**, entfallene Bereiche sind Löcher (Nullen, bei `punch_hole` sparse); der Catalog tauscht die Inkarnation atomar (`blob_replace`). Der Umzug in eine **neue** `blob_id` ist nur Blob-GC (§14.4).
- **Redo-Segmente sind kein `FileKind`.** Sie sind durch `segment_id` und `redo_floor` bestimmt: jedes Segment mit `segment_id ≥ redo_floor` ist lebendig (auch das vorallokierte Reservesegment, `INV-LOG-7`), darunter verwaist. `FileKind` kennt `Run`, `Blob`, `Segment(kind)`.
- Alle Dateien und Verzeichnisse entstehen mit `Perm::Private`.
- **Temporäre Dateien** tragen das Suffix `.building.<tmp_id>`; `tmp_id` stammt aus dem im Catalog reservierten Bereich (§11.8). Beim Öffnen werden alle `.building.*` entfernt.
- **Verwaiste Dateien** (nicht im Catalog) werden beim Öffnen und periodisch gelöscht. Das ist sicher, weil nur ein Catalog-Edit eine Datei lebendig macht (`INV-CAT-1`).
- **Ein Space = ein Verzeichnis**, samt seiner Blobs. Space entfernen = Edit + Dateien löschen + `remove_dir`.
- **Dateien sind unveränderlich**, sobald lebendig (Ausnahme: Redo-Segment bis zum Siegel).

### 10.3 Zustandsdiagramm einer Datei

```
(nicht vorhanden) ──create .building──► im Bau ──sync_all + sync_dir──► dauerhaft, UNREFERENZIERT
      ▲                                                                       │ Edit(add) dauerhaft
      │ Verwaisten-Suche                                                      ▼
      └──────────── Unlink + sync_dir ◄── Graveyard ◄── obsolet ◄──────── LEBENDIG (Catalog nennt sie)
                    (erst nach letzter Referenz)           ▲ Edit(remove) dauerhaft
```

Jeder Wechsel zwischen „unreferenziert“ und „lebendig“ ist **ein** Catalog-Edit (P-2).

---

## 11. Formate

### 11.1 Konventionen

- Alle Zahlen **little-endian**. `varint` = LEB128 für `u64` (höchstens 10 Bytes), **nur kanonische** Kodierungen gültig (kein überflüssiges `0x80 … 0x00`, zehntes Byte nur `0x01`), alles andere `Corrupt` (`INV-FMT-1`).
- Jede Dateiart hat **genau eine Version (`1`)** und Feature-Bits (SD-04). Unbekannte Version oder unbekanntes Pflicht-Bit ⇒ `Unsupported`.
- Jede Datei beginnt mit `magic[8] ‖ file_kind u8 ‖ format_ver u16 ‖ feature_bits u32 ‖ file_id u64 ‖ instance_id[16] ‖ header_crc u32`.
- Längenfelder werden gegen Plausibilitätsgrenzen geprüft, **bevor** allokiert wird (P-4).
- Prüfsumme CRC32C für Zufallsfehler. In `Authenticated`/`Encrypted` zusätzlich Tag (§17.1).

### 11.2 `FORMAT`

`magic ‖ format_ver ‖ feature_bits ‖ integrity_mode u8 ‖ algo_ids (aead, mac, kdf) ‖ instance_id ‖ limits (max_key_bytes, block_target, …) ‖ crc`. Unveränderlich nach Anlegen. Im Modus `Authenticated/Encrypted` mit KEK-abgeleitetem MAC geschützt (Schutz gegen Austausch).

### 11.3 `KEYS`

Umhüllte **Space-Schlüssel** (je Space ein DEK, mit KEK umhüllt, atomar ersetzt). **Clusterschlüssel stehen nicht hier**, sondern beim `KeyProvider` (§17.2). `KEYS` enthält höchstens `max_spaces` Einträge (≤ 4096).

### 11.4 Redo-Segment und Frame

Segment: vorallokiert (`preallocate`), `header ‖ Gruppe*`, am Ende **gesiegelt** (Siegel-Record mit Kettenstand). Eine **Gruppe** ist ein auf `align` (Standard 4096) ausgerichteter Block, der einen oder mehrere **Frames** enthält (Torn-Write-Eingrenzung).

```
Frame := head ‖ ops ‖ tail
head (Klartext, Ketten-MAC-geschützt):
      len u32 | stamp u64 | n_ops u16 | idem_token[16] | flags u8 |
      ops_head[]: { space u16, cluster[16], kind u8, payload_len u32 }
ops  : je Op die Nutzlast
       - Space mit Seal::Cluster:  GESAMTE Nutzlast (Schlüssel, Kopf, Körper, Gewichte) mit Clusterschlüssel versiegelt (§17.3)
       - sonst: unter Redo-Schlüssel (Modus Encrypted) bzw. Klartext (Plain/Authenticated)
tail : crc32c | chain_mac    (H_i = MAC(H_{i-1} ‖ head ‖ payload_tags))
```

Folgen: Die Kette erkennt Abschneiden und Einfügen (`wal_completeness`-Prüfung im Recovery). Stamp, Space und Cluster stehen im Klartext, damit Recovery und Löschplanung **ohne Schlüssel** arbeiten. Nach Vernichtung eines Clusterschlüssels bleibt die Nutzlast unlesbar, das Replay erzeugt für die Op einen `ErasedMarker` im Speicher.

### 11.5 Run

```
Run := header
       ClusterSection*          (aufsteigend nach ClusterId; Pure: genau eine)
       ClusterDirectory         (Klartext: je Cluster { cluster, section_off, section_len, n_blocks, min/max_stamp, weight_sum[W] })
       PackMeta                 (space_id, pack_key | pure_cluster, min/max Schlüssel entfällt (nur je Cluster), Blob-Bereich, Eintragszahlen)
       ExtensionBlock*          (§18.3)
       Footer                   (Offsets, Prüfsumme, Kette)

ClusterSection := DataBlock* ‖ SubIndexBlock ‖ FilterBlock ‖ RangeDelBlock
   DataBlock       Zielgröße 16 KiB, Präfix-Kompression mit Restart-Punkten, optional komprimiert, dann Seal (Clusterschlüssel) oder Block-AEAD (Space-Schlüssel)
   SubIndexBlock   je Datenblock: letzter Schlüssel, Offset, Länge, entries, weight_sum[W], min/max_stamp, block_uid[16]
   FilterBlock     Schlüsselfilter dieses Clusters (Ribbon oder Bloom)
   RangeDelBlock   Bereichsmarker des Clusters
```

- **Seal=Cluster:** `SubIndexBlock`, `FilterBlock`, `RangeDelBlock` und alle `DataBlock`s sind mit dem Clusterschlüssel versiegelt. Plaintext bleiben `ClusterDirectory`, Cluster-ID (opak/pseudonym), Zähler, Größen und Stamp-Bereiche.
- **Seal=None:** Blöcke unter Space-Schlüssel (Encrypted) bzw. nur Prüfsumme (Plain/Authenticated).
- Ein **Pure**-Run hat genau einen Abschnitt; das Verzeichnis hat einen Eintrag.
- Die Verzeichnisse aller Runs werden beim Öffnen einer Datei in den Speicher geladen (klein: ~48 Byte je Cluster und Datei) und über **Packgruppen-Filter im Catalog** (Ribbon über Cluster-IDs je Run) vorselektiert.

### 11.6 Blob

```
Blob := header ‖ ClusterSection* ‖ ClusterDirectory ‖ footer
ClusterSection := Record*
Record := len u32 | nonce[24] | sealed_body | crc32c
```

Bodies sind versiegelt wie in §17.3. **AAD** des Records ist `(space, cluster, blob_id, offset)`; deshalb bleiben Records bei einer Inkarnation mit gleichen Offsets gültig, ein GC-Umzug dagegen **versiegelt neu**. Zielgröße ≤ 64 MiB, ein Blob gehört zu einem Space und ist Pure oder Packed.

### 11.7 Segment (obere Schicht, Rahmen des Kerns)

`header (kind u16, id u64, space, pack_key|cluster, cover) ‖ opake Nutzlast ‖ footer (crc, Kette)`. Vertrag und Lebenszyklus §18.1.

### 11.8 Catalog

Datei = `Snapshot-Record` (vollständiger Zustand) gefolgt von `Edit-Records`. Jeder Edit ist atomar (ein Append + `sync_data`) und trägt `seq`, Kette und CRC. Rollover bei Größe: neue Datei mit Snapshot, `CURRENT` per atomarem `rename`.

Op-Auswahl: `space_add | space_drop | cluster_register | cluster_state(c, state, stamp) | cluster_link | restriction_set | restriction_lift | pack_assign | files_add(kind, id, group|cluster, meta) | files_remove | blob_replace(blob_id, inc) | blob_live_delta | redo_floor | segment_add | segment_remove | hold_set | consumer_ack | erase_pending_add(ticket) | erase_done(ticket) | erased_ranges_add | key_epoch | idem_trim | reserve_stamps(hw) | reserve_ids(kind, hw)`.

- **Reservierungen:** `reserve_stamps(hw)` und `reserve_ids` heben eine Hochwassermarke **vorab** an (Block von 2^20 Stamps, 2^16 Dateikennungen). Nach Absturz beginnt die Vergabe bei der Marke (`INV-STAMP-3`, `INV-ID-1`). Nachreservieren geschieht auf einem Wartungsthread bei < 50 % Rest. Fallback im Committer: ein synchroner Edit (Rang 1 → Rang 3, einzige zulässige Doppelhaltung über E/A, kurz).
- Unbekannte Tags ⇒ `Unsupported`, nie Ignorieren.
- **Regel (L6):** Eine Datei ist lebendig genau dann, wenn ein durabler `files_add` existiert und kein `files_remove`.

### 11.9 Freeze-Liste (Gate G-F)

Eingefroren: Datei-Kopf, `FORMAT`, `KEYS`, Redo-Gruppe/Frame/Kette, ClusterSection-Aufbau, ClusterDirectory, SubIndex-Felder (`weight_sum`, `block_uid`), Seal-Rahmen (Nonce, AAD-Regel), Blob-Record und Inkarnationsregel, Catalog-Op-Tags (mit reserviertem Bereich), Ledger-Format, Vergleichsfunktion `cmp`, Tag-Kodierung. Zu jedem Format gibt es Referenzcodierer (Python, unabhängig) und goldene Vektoren (Anhang A). Nach G-F ist jede Änderung eine Migration.

---

## 12. Schreibpfad

### 12.1 Ablauf von `apply`

```
1  Vorprüfung ohne Sperre: Größen, Space/Cluster bekannt, Idempotenz-Cache
2  Eintrag in den Intake (Rang 2), Warten auf Gruppenführer
3  Gruppenführer = Committer (Rang 1) nimmt alle Wartenden:
     a) Cluster-Zustand prüfen: Retired/Shredded/Purged ⇒ ClusterRetired; Restriction mit BlockWrite ⇒ Restricted
     b) Guards prüfen gegen aktuellen Zustand → fehlgeschlagene Changes ausscheiden
     c) Stamps vergeben (aus der reservierten Marke)
     d) Nutzlast versiegeln (vorbereitet außerhalb der Sperre: Nonce, Einmalschlüssel)
     e) Ein Redo-Frame je Change in ausgerichteter Gruppe, ein `write` + ein `sync_data` je Gruppe   ◄── DURABILITÄTSPUNKT
4  Memtable einfügen (Rang 5), Version veröffentlichen (Rang 4), `visible_stamp` anheben
5  Rückkehr an alle Wartenden
```

- **Sichtbarkeit:** nur nach Durabilität. Ausnahme `Durability::Relaxed` (Opt-in je Aufruf; vor Fsync sichtbar, bei Absturz verlierbar; für Löschnachweise unzulässig).
- **Rückdruck:** Memtable-Größe, Zahl ungeflushter Memtables, L0-Dateien, Redo-Rückstand wirken stufenweise: verlangsamen (Warten **außerhalb** jeder Sperre), dann `Backpressure(Retry)`.
- **Redo-Lebenszyklus:** `Open → Sealed → Obsolete (< redo_floor) → Graveyard`. Ein Reservesegment liegt immer vorallokiert bereit (`INV-LOG-7`), damit ein Rollover nie auf `ENOSPC` im kritischen Pfad trifft.
- **ENOSPC/EIO:** Poison (F-2/F-3), Lesen bleibt.

### 12.2 Memtable

Je Space eine aktive Memtable, geordnet nach `cmp`, daher je Cluster zusammenhängend. Sie hält **Klartext** (AN-05). Größenziel `memtable_bytes` (Standard 64 MiB), höchstens `max_immutable` (3). Flush teilt nach Cluster in Abschnitte.

### 12.3 Stamps und Reservierung

Der Committer vergibt Stamps lückenlos je Gruppe aus der reservierten Marke. Nach einem Absturz setzt Recovery den nächsten Stamp auf `max(letzter gültiger Redo-Stamp, Catalog-Marke) + 1` bzw. **über der Marke**, sodass nie ein Stamp doppelt entsteht, auch wenn nicht-durable Changes sichtbar waren (`INV-STAMP-3`).

### 12.4 Flush

Je Space: Memtable einfrieren → je Cluster ein Abschnitt → Packgruppe/Pure bestimmen (Pure, wenn Cluster ≥ `pure_threshold`) → Run schreiben (`.building` → `sync_all` → `rename` → `sync_dir`) → **ein** Edit (`files_add`, `redo_floor`) → neue Version veröffentlichen. Absturz vor dem Edit: Run verwaist, Redo spielt erneut ein.

### 12.5 Idempotenz

`IdemToken` liegt im Frame-Kopf und in einer begrenzten Token-Tabelle (TTL, `idem_trim` im Catalog). Die Tabelle enthält **keine Inhalte und keine Cluster-Kennungen**.

---

## 13. Lesepfad

1. `view()` klont `Arc<Version>` (Memtables, immutable Memtables, Dateimengen je Wartungsgruppe, Cluster-Tabelle). Danach keine Sperre.
2. **Gate zuerst:** Clusterzustand und Restrictions vor jeder Entschlüsselung (O(1), Generationszähler, §19.5). `Retired` oder weiter ⇒ `Erased`; `ReadBlocked` ⇒ `Restricted`.
3. **Punktabfrage:** Memtable → immutable Memtables → je Level die Dateien der Wartungsgruppe → `ClusterDirectory` (im Speicher) → Abschnitt → SubIndex (entsiegeln, gecacht) → Filter → ein Datenblock. Ziel: höchstens **ein** Datenblock-Read je Level-Treffer.
4. **Cursor/Page:** k-Wege-Merge über Quellen **eines Clusters**. Header ohne Körper-E/A. `page` summiert `weight_sum` aus dem SubIndex, überspringt Blöcke und liefert `ResumeToken` (Stamp, Cluster, Schlüssel, Rest-Budget).
5. **Körper** erst bei `body()`: Blob-Read, CRC, Entsiegeln. Schlüssel fehlt ⇒ `Shredded`.
6. **Block-Cache** (sharded, Pin-Zähler): Blöcke bleiben, solange eine `EntryRef` darauf zeigt. Bei Dateiverlust `invalidate_file`.
7. **Ältere Sicht:** eine View unter `S` in einen Bereich mit `erased_ranges`-Eintrag ⇒ `Err(Erased{..})`, nie leer, nie Altdaten (`INV-ERA-5`).
8. **Cluster-übergreifende Scans** gibt es nicht im Kern (ein Cursor gehört zu einem Cluster, in `NONE`-Spaces zu `NONE`). Wer alle Cluster braucht, iteriert über `Inspector::clusters()`.

---

## 14. Wartung

### 14.1 Lanes und Claims

| Lane | Aufgabe | Priorität |
|---|---|---|
| `urgent` | Löschfolgearbeit, Purge mit naher Frist, Packgruppen-Abwurf | höchste, darf Compaction abbrechen |
| `flush` | Memtable → Runs | hoch, nie hinter Compaction |
| `compact` | Leveled-Compaction je Wartungsgruppe | normal |
| `blobgc` | Blob-GC nach Lebendanteil | normal |
| `idle` | Scrub (Bit-Rot-Prüfung), Verwaisten-Suche | niedrig |

**Claims:** Jede Arbeit beansprucht ihre Eingabedateien exklusiv (`claims: file_id → job`, unter `registry`, Rang 6). Höhere Priorität bricht niedrigere ab (Flag, Prüfung zwischen Blöcken, Ausgabe verworfen). Der Ausgabe-Edit wird **abgelehnt**, wenn `files_remove ⊄ gehaltene Claims` (`INV-CLAIM-1`).

### 14.2 Flush

§12.4.

### 14.3 Compaction

Leveled je Wartungsgruppe, Verhältnis `T` (Standard 10), L0 nach Größe. Eingabe wird bereinigt: Versionen unter `floor` (kleinster Hold/View/Consumer-Stamp), verdeckt von neueren, entfallen; Tombstones entfallen, wenn keine tiefere Ebene den Schlüssel enthält. Pure-Promotion nach R-PL-3. Subcompactions je Cluster-Bereich dürfen parallel laufen, ein Cluster wird nie innerhalb eines Blocks geteilt (L2). Compaction braucht für versiegelte Cluster den Clusterschlüssel; ist der Cluster `Retired/Shredded`, wird sein Abschnitt **ohne Entschlüsseln verworfen**.

### 14.4 Blob-GC und Graveyard

Lebendanteil je Blob aus `blob_live_delta`. GC: überlebende Records in neuen Blob mit neuer `blob_id` (neu versiegelt), Runs werden in der nächsten Compaction auf die neuen Refs umgestellt. **Graveyard:** Eine Datei wird erst gelöscht, wenn (a) der entfernende Edit durabel ist **und** (b) die letzte `Arc<Version>` mit Verweis gefallen ist (`INV-GRAVE-1`).

### 14.5 Segmente

Siehe §18.1.

### 14.6 Packgruppen-Abwurf (Ablauf der Aufbewahrung)

```
PD1 Edit: alle Cluster der Gruppe → Retired(S)           ◄── logischer Punkt (ein Edit)
PD2 Restrictions in der Gruppe? ⇒ Abbruch (Err(Restricted)), nichts passiert
PD3 Clusterschlüssel aller Cluster vernichten (Cryptographic), Edit → Shredded
PD4 Claims auf Dateien der Gruppe, Compaction abgebrochen
PD5 Ein Edit { files_remove[alle Dateien der Gruppe], cluster_state → Purged, erased_ranges_add }  ◄── physischer Punkt
PD6 Graveyard → unlink → sync_dir → Ledger → Receipt
```

### 14.7 Governor

Token-Bucket je Klasse (Wartungs-E/A, CPU, offene Dateihandles). Wartung nutzt höchstens `maint_io_fraction` (50 %) der gemessenen Kapazität, `urgent` hat eine eigene Reserve. Dateihandles: LRU mit Obergrenze `max_open_files`, kalte Gruppen halten keine Handles.

---

## 15. Recovery

### 15.1 Ablauf beim Öffnen (deterministisch)

```
R1  LOCK nehmen (exklusiv; ReadOnly: geteilt). Misserfolg ⇒ Err(Locked)
R2  FORMAT lesen und prüfen (Kette, Version); Unsupported ⇒ Abbruch
R3  CURRENT → Catalog: Snapshot + Edits bis zum letzten gültigen Record, Rest verwerfen (Torn Tail)
R4  Verzeichnisse scannen: Dateien ohne Catalog-Eintrag ⇒ verwaist (Liste, noch nicht löschen)
R5  KeyProvider prüfen: KEK verfügbar? Sonst Modus LockedKeys (nur Spaces ohne Seal lesbar)
R6  Redo ab redo_floor: je Gruppe/Frame Kette prüfen; erster ungültiger Frame = Ende
    versiegelte Nutzlast ohne Schlüssel oder Cluster Shredded ⇒ ErasedMarker (keine Daten)
R7  Memtable aufbauen, visible_stamp = letzter gültiger Frame, nächster Stamp > Catalog-Marke
R8  Offene Löschungen = erase_pending ∪ Cluster in Retired/Shredded ≠ Purged ⇒ in `urgent` einplanen, ab erster offener Stufe fortsetzen
R9  Verwaiste Dateien und .building.* entfernen (nach R8, nie davor)
R10 Wartung starten, OpenReport (Frames, verworfene Bytes, offene Löschungen, verwaiste Dateien)
```

### 15.2 Absturzmatrix

| Unterbrechung | Soll nach Neustart |
|---|---|
| vor Redo-`sync` | Change nicht sichtbar, `apply` kehrte nicht zurück |
| nach Redo-`sync`, vor Rückkehr | Change vorhanden (Token verhindert Doppelung) |
| mitten im Run-Bau | `.building` entfernt, Redo spielt ein |
| nach `rename`, vor Edit | Run verwaist, entfernt |
| nach Edit, vor Unlink | alte Dateien verwaist, entfernt |
| Gate-Edit durabel, Schlüssel noch da | Löschung wird fortgesetzt (Roll-forward) |
| Schlüssel vernichtet, `Shredded`-Edit fehlt | `probe` = Absent ⇒ Edit nachholen |
| Purge-Edit durabel, Unlink offen | Verwaisten-Suche entfernt |
| Ledger-Rahmen halb | Kette erkennt Torn Tail, Eintrag wird aus Catalog neu erzeugt |
| Absturz nach nicht-durablem Relaxed-Change | Stamp wird nicht wiederverwendet (`INV-STAMP-3`) |

---

## 16. Sichten, Holds, Consumer, Restrictions

### 16.1 Sichten
`View` bei Stamp `V` ist unveränderlich (`INV-VIEW-1`). Jede View zählt in der `registry` (Untergrenze `floor`).

### 16.2 Holds
Befristet (`ttl`), mit Eigentümer, halten Versionen ≥ Hold-Stamp. Abgelaufene Holds werden verworfen. Technische Funktion (Zeitreise, Index-Aufholen), **keine** Rechtsfunktion.

### 16.3 Consumer
Persistent (`consumer_ack` im Catalog), zählt wie ein Hold, mit **Maximalrückstand** `max_lag` (Stamps). Überschreitet ein Consumer ihn, wird er `Lagged` (Hold fällt, `next` liefert `Err(Lagged)`; Aufholen per Vollscan Sache der oberen Schicht). Verhindert unbegrenztes Aufstauen alter Versionen.

### 16.4 Change-Feed
Reihenfolge nach Stamp, **mindestens einmal**. Löschungen erscheinen als `Change::Erased{cluster|range, at}`, damit Konsumenten ihre abgeleiteten Daten innerhalb der Frist bereinigen.

### 16.5 Cluster-Verknüpfung
Siehe §18.2.

### 16.6 Restriction

```rust
pub struct RestrictionSpec {
    pub reason: u16,              // opaker Code der oberen Schicht, PFLICHT (≠ 0)
    pub owner: OwnerId,
    pub effects: RestrictionEffects,   // BLOCK_ERASE (immer gesetzt) | BLOCK_WRITE | BLOCK_READ
    pub expires: Option<WallTime>,     // keine automatische Aufhebung ohne Ablauf; Ablauf lässt nur den Vermerk fallen
}
```

- Solange mindestens eine `Restriction` mit `BLOCK_ERASE` auf einem Cluster liegt (oder auf einem Cluster in der **Link-Hülle** der Löschung, §18.2), antwortet jede Löschung mit `Err(Restricted{ids, reasons})`. **Es gibt keinen Override im Kern** (SD-09). Die obere Schicht muss die Restriction selbst aufheben (`lift`, Eigentümer-Prüfung, Ledger-Eintrag), und diese Entscheidung ist ihre.
- `BLOCK_READ`: Normale Lesepfade liefern `Err(Restricted)`. Zugriff für zweckgebundene Lesung nur mit `RestrictedReadCap` (nicht klonbar, vom Storage-Eigentümer ausgestellt, `Observer`-Ereignis mit Zweck-Code, **nie** im Cache oder Consumer-Feed).
- `Restriction` verhindert **Ablauf-Abwürfe** (`pack_expired`) ebenso (die betroffenen Cluster werden ausgenommen und die Packgruppe wird für diese Cluster **neu gepackt**, bevor die übrigen verworfen werden; Fallback: Abwurf wartet).
- Restrictions sind durabel (Catalog) und überleben Restore (im Ledger-Anhang).

---

## 17. Verschlüsselung, Versiegelung und Löschung

### 17.1 Modi (in `FORMAT`, unveränderlich)

| Modus | Schutz | Verfahren |
|---|---|---|
| `Plain` | Zufallsfehler | CRC32C, Kettenprüfsumme im Redo (nicht kryptografisch) |
| `Authenticated` | Integrität gegen Manipulation (AT-3), keine Vertraulichkeit | BLAKE3-keyed-MAC (16-Byte-Tag) mit Zusatzdaten, Kette in Redo, Catalog, Ledger |
| `Encrypted` | Vertraulichkeit und Integrität (AT-1, AT-2, AT-3) | AEAD (XChaCha20-Poly1305, 24-Byte-Zufallsnonce), Schlüssel per KDF |

Eigene Kryptografie wird nicht geschrieben (NZ-2). AEAD/MAC/KDF sind Ports mit Algorithmus-IDs im `FORMAT`; Wechsel = Migration. Zufallsnonces (SD-14): kein Zählerzustand, sicher bei Umkopieren. Preis: 24 Byte je Block/Record.

### 17.2 Schlüsselhierarchie

```
KEK (KeyProvider, extern; nie in Dateien, nie in Checkpoints)
 ├─ Space-DEK        zufällig (Entropy), mit KEK umhüllt in KEYS
 │     └─ Block-Schlüssel (Spaces ohne Seal) = KDF(DEK, "ctxs blk v1" ‖ file_id)
 ├─ Redo-Schlüssel   = KDF(KEK, "ctxs redo v1" ‖ segment_id)
 ├─ Catalog-/FORMAT-/Ledger-Schlüssel = KDF(KEK, "ctxs cat v1" | "ctxs led v1")
 └─ Clusterschlüssel KeyProvider::cluster_key(cluster, epoch)   (NICHT in KEYS)
        └─ Siegelschlüssel = KDF(cluster_key, "ctxs seal v1" ‖ nonce)
```

```rust
pub trait KeyProvider: Send + Sync {
    fn kek(&self) -> Result<Key32, KeyError>;
    fn cluster_key(&self, c: ClusterId, epoch: u32) -> Result<Option<Key32>, KeyError>;  // None = vernichtet
    fn create_cluster_key(&self, c: ClusterId, e: &dyn Entropy) -> Result<u32, KeyError>; // liefert epoch
    fn destroy(&self, c: ClusterId) -> Result<(), KeyError>;   // durabel bei Rückkehr, idempotent
    fn probe(&self, c: ClusterId) -> Result<KeyState, KeyError>; // Present | Absent
}
```

Der Kern speichert **keine** Clusterschlüssel. Ein Schlüsselbund mit Millionen Clustern ist Sache des Providers. Der Kern hält einen begrenzten Schlüssel-Cache (TTL, Obergrenze) mit Generationszähler (§19.5). Schlüsselmaterial liegt in `Zeroizing`-Hüllen.

### 17.3 Versiegelung (Seal)

- Jeder Block, jede Op-Nutzlast im Redo und jeder Blob-Record eines `Seal::Cluster`-Spaces wird mit `AEAD(siegelschlüssel, nonce[24] zufällig, plaintext, aad)` versiegelt.
- **AAD:** Block `(space, cluster, block_uid)`, Redo-Op `(space, cluster, stamp, op_index)`, Blob-Record `(space, cluster, blob_id, offset)`. **Nicht** an Dateiname oder Dateiposition der Datei gebunden → Rohkopieren (Purge, Packgruppen-Umzug, Compaction-Übernahme) ist gültig.
- Versiegelt wird **vor** dem Redo-Frame (in `apply`). Die Memtable hält Klartext. Der Flush versiegelt den Block neu mit dem Block-Format (andere AAD).
- Folge (L3): Nach Vernichtung des Clusterschlüssels ist **alles** des Clusters (Nutzerschlüssel, Kopf, Körper, Index, Filter, Bereichsmarker) in Redo, Runs, Blobs, Checkpoints und Backups ohne Schlüssel unlesbar (`INV-ERA-7`). Offen bleiben nur Cluster-ID, Zähler, Größen, Stamp-Bereiche.

### 17.4 Stufen und ehrliche Garantien

| Stufe | Verfahren | Schützt gegen | Schützt **nicht** gegen |
|---|---|---|---|
| `Logical` | Gate (`Retired`) | Lesen über die API | Inhalt auf Platte |
| `Physical` | Gate, Purge (Datei abwerfen oder roh umkopieren), Edit, Unlink, `sync_dir` | Lesen aus Dateien im Verzeichnis, Wiederauftauchen nach Absturz | Journal, Flash-Reste, Backups außerhalb, Hardlinks |
| `Scrubbed` | wie `Physical`, vor Unlink Datei mit Nullen überschreiben + Sync | einfache Wiederherstellung auf klassischen Datenträgern | CoW-Dateisysteme, SSDs (best-effort) |
| `Cryptographic` | Gate, **Clusterschlüssel vernichten**, dann Purge | alle Reste inklusive Redo, Backups, Flash (Schlüsselfreiheit vorausgesetzt) | Kopien des Schlüssels; in Spaces ohne Seal nicht verfügbar |

Der Standardpfad ist `Cryptographic` + Purge in `Seal::Cluster`-Spaces. Für Spaces ohne Seal gilt höchstens `Physical`/`Scrubbed`; die Garantiegrenzen stehen im Receipt.

### 17.5 Protokoll `Eraser::cluster` (Schritte ER0–ER7)

```
ER0 Vorprüfung:   Cluster existiert, Link-Hülle H = {c} ∪ transitiv gebundene abgeleitete Cluster berechnen,
                  Restrictions auf H ⇒ Err(Restricted), Holds/Consumer mit Stamp < S gemäß Policy (Hold fällt nur mit Admin-Recht)
ER1 Gate:         EIN Edit { cluster_state(H → Retired, S), erase_pending_add(ticket) } durabel (Gruppen-Fsync)   ◄── LOGISCHER Punkt
                  Ab hier: Lesen ⇒ Erased, apply in H ⇒ ClusterRetired. Memtable-Bereiche von H werden als ErasedMarker überschrieben.
ER2 Schlüssel:    (Cryptographic) KeyProvider::destroy(c) für alle c ∈ H; probe == Absent; Cache leeren, Generationszähler ++;
                  Edit { cluster_state(H → Shredded) }                                                          ◄── KRYPTO-Punkt
ER3 Plan:         betroffene Dateien über Catalog-Packgruppen-Filter und ClusterDirectory bestimmen; Claims auf diese Dateien
                  (Compaction dort wird abgebrochen); Segmente mit Überlappung ermitteln (§18.1)
ER4 Barriere:     Flush aller Memtables mit Einträgen aus H; Redo-Segmente bis S werden obsolet (redo_floor anheben)
ER5 Umbau:        Pure-Dateien: nichts bauen, nur entfernen. Packed-Dateien: neue Datei = übrige Abschnitte roh kopiert + neues Verzeichnis
                  (sync_all, sync_dir). Blobs: neue Inkarnation (gleiche Offsets, Löcher) bzw. entfernen. Segmente: SegmentOwner::rewrite_without
ER6 Edit:         EIN Edit { files_add, files_remove, blob_replace, blob_live_delta, cluster_state(H → Purged),
                              erased_ranges_add(H, S), erase_done(ticket) }                                      ◄── PHYSISCHER Punkt
ER7 Danach:       Version veröffentlichen, Block-Cache invalidieren, alte Dateien ins Graveyard (ViewPolicy), Scrub optional,
                  unlink, sync_dir, Ledger-Eintrag (§17.8), ErasureReceipt zurückgeben
```

- **Reihenfolge ist Vertrag (L7):** ER1 < ER2 < ER6 < Unlink. Der Kern führt ER3–ER6 bei `Cryptographic` **nie** aus, solange `probe` den Schlüssel liefert (`INV-ERA-3`).
- **Zeit:** ER0–ER2 sind O(1) (zwei Edits, ein Provider-Aufruf). ER3–ER6 sind begrenzt durch die Größe der betroffenen Packgruppe und **nicht** durch den Store. Der Purge wird **gebündelt**: Alle wartenden Cluster derselben Packgruppe teilen sich einen Lauf. Auslöser: Frist zu 70 % verbraucht, Menge ≥ `purge_batch_min`, oder anstehende Compaction in der Gruppe (Mitnahme: `erase_swept` im selben Edit).
- **Frist:** `purge_deadline` ist **Pflichtparameter** des Aufrufs (SD-06), keine Standardannahme des Kerns. Wird sie gerissen, steigt die Priorität auf `urgent`, `Observer` meldet es.
- **Nebenläufigkeit mit Views:** `ViewPolicy::Wait(timeout)` (Unlink wartet, nach Timeout `Err(Blocked{..})`, der Vorgang bleibt in ER7 offen und wird wiederholt, die logische Garantie gilt bereits) oder `Revoke` (Views werden widerrufen).
- **`Receipt` (INV-ERA-8)** erst, wenn INV-ERA-2/6 gelten, der Ledger-Eintrag durabel ist und alle Segmentbesitzer bestätigt haben.

### 17.6 `Eraser::range` (Bereich innerhalb eines Clusters)

Marker `EraseRange` (Stamp `S`) per normalem `apply`-Pfad (**logischer Punkt**), Flush-Barriere für den Cluster, betroffene **Blöcke des Clusterabschnitts** (über SubIndex) werden neu geschrieben ohne die Versionen ≤ `S` im Bereich, Blobs per Inkarnation, **ein** Edit (physischer Punkt), danach wie ER7. Roll-forward, idempotent. Reads unter `S` in den Bereich liefern `Erased` (`INV-ERA-5`). Append-only-Spaces erlauben das nur mit Admin-Recht (im Receipt vermerkt).

### 17.7 Recovery der Löschung

Offene Menge = `erase_pending ∪ {EraseRange-Frames nach Anker} ∪ {Cluster Retired/Shredded} − erase_done`. R8 (§15.1) setzt ab der ersten offenen Stufe fort. ER3–ER6 sind idempotent (Auswahl aus dem aktuellen Catalog; ein vollzogener ER6 ist an `erase_done` erkennbar). ER7 ist idempotent über die Verwaisten-Suche.

### 17.8 Ledger, Backups, Rebucket

- **Ledger** (`ERASE-LEDGER/`): append-only, kettenverknüpft (BLAKE3), Einträge `{ticket, cluster_id|pack_key, level, S, stage, ts, prev_hash}`, **ohne Inhalte und Kennungen**. Signatur und Berichte sind Sache der oberen Schicht (der Kern liefert kanonische Bytes).
- **Checkpoint-Manifest** nennt `erase_epoch` (Ledger-Position). Das Ledger liegt **nie** im Checkpoint. Der KEK und die Clusterschlüssel liegen nicht in den Checkpoint-Dateien (`INV-KEY-1`).
- **Restore** (`restore_check`): Vor dem Öffnen wendet der Kern alle Ledger-Einträge nach `erase_epoch` erneut an: betroffene Cluster werden `Retired/Shredded`, der Purge läuft. Wiederhergestellte Restrictions werden aus dem Anhang des Ledgers übernommen. `INV-ERA-9`.
- **Hardlink-Checkpoints** sind Opt-in mit Warnung (ein Purge erreicht den alten Inode nicht). Standard `Copy` oder `Reflink`.
- **`rebucket(c, due')`:** Neue Packgruppe wählen, Abschnitt des Clusters in einen neuen Run der Zielgruppe (roh kopiert), **ein** Edit `{files_add, files_remove/Abschnitt-Purge in der Quellgruppe, pack_assign(c)}`. Der Zwischenzustand ist unsichtbar, weil die Zuordnung `cluster → Gruppe` im selben Edit wechselt (`INV-PL-4`).

### 17.9 Vertrag und Grenzen

- **Nutzerschlüssel:** In Spaces **ohne** `Seal::Cluster` MUSS er frei von personenbezogenen Daten sein (Vertrag, nicht prüfbar; `Inspector::lint_keys` sucht in Tests Muster). In `Seal::Cluster`-Spaces ist er versiegelt.
- **Cluster-IDs** sind opak und zufällig (oder per Schlüssel-HMAC der oberen Schicht erzeugt), nie aus Kennungen ableitbar (`INV-CLUSTER-2`). Sie bleiben als Pseudonym im Klartext (Verzeichnis, Redo-Kopf, Ledger).
- **Nicht kontrolliert:** Prozessspeicher, Auslagerungsdatei, Core-Dateien (best-effort, `forbid(unsafe_code)`; Speicherschutz liegt in einer Port-Implementierung). Dateisystem-Journal, Flash-Wear-Leveling, externe Backups (siehe §17.4). `ColdTier`-Dateien unterliegen den Garantien des Ports.
- **Abgeleitete Daten** in Strukturen, die der Kern nicht kennt, sind nur so sicher, wie ihre Besitzer §18 einhalten.

---

## 18. Abgeleitete Daten und Erweiterungsverträge

### 18.1 Segment-Register

Vom Besitzer geschriebene Dateien **neben** den Runs, vom Catalog verwaltet, in `segments/<kind>/`, **Pure oder Packed** wie Runs.

```rust
pub trait SegmentOwner: Send + Sync {
    fn kind(&self) -> SegmentKind;
    fn cover(&self, seg: &SegmentMeta) -> SegmentCover;   // Exclusive(ClusterId | PackKey) | Mixed(exakte Liste oder Filter)
    fn rewrite_without(&self, seg: &SegmentMeta, drop: &ClusterSet, io: &mut SegmentIo)
        -> Result<RewriteOutcome, SegErr>;                // Neubau oder Verwerfen; Pflichtmethode
    fn watermark(&self) -> Stamp;                         // bis wohin das Segment aufgeholt ist
}
```

- `Exclusive`: der Kern wirft das Segment bei Löschung/Abwurf selbst ab, ohne den Besitzer zu fragen.
- `Mixed`: der Kern ruft `rewrite_without` auf. **Ein Erase ist erst `Done`, wenn alle überlappenden Segmente bestätigt haben** (`INV-ERA-8`). Antwortet der Besitzer nicht innerhalb der Frist, bleibt der Vorgang in `Purging` mit Alarm. Er wird **nie** übersprungen.
- Der Besitzer arbeitet nur über `SegmentIo` (Port-gestützt, versiegelt wie §17.3, wenn der Space `Seal::Cluster` hat).

### 18.2 Cluster-Links (abgeleitete Cluster)

`Clusters::link(derived, sources)` bindet einen Cluster an ≤ 8 Quell-Cluster (azyklisch). Die obere Schicht legt abgeleitete Daten, die aus mehreren Quellen entstehen, in einen **eigenen abgeleiteten Cluster** und bindet ihn an alle Quellen. Löschung irgendeines Quellclusters zieht den abgeleiteten Cluster **im selben ER1-Edit** mit (Link-Hülle, §17.5). **Das gilt nur für ausdrücklich gebundene Cluster**; es gibt keine implizite Vererbung. So bleiben Einträge einclustrig (L2), und Mehrbetroffenen-Daten werden durch die Wahl des Clusters (zum Beispiel ein gemeinsamer Fallcluster) oder durch Links modelliert, nicht durch Mehrfach-Scopes im Eintrag (SD-24).

### 18.3 ExtensionBlock

Optionale opake Blöcke am Run (Tag `u16`, Version `u16`), je **Clusterabschnitt** oder je Datei. Ein `ExtensionBuilder` erhält beim Flush/Compaction den Strom der Einträge und schreibt Blöcke; `merge(inputs) → output` ist Pflicht. Unbekannter Tag beim Rewrite ⇒ **Fehler** (nicht still verwerfen). Pflichtmethode `rewrite_without(clusters|range)`; ein Besitzer ohne sie darf keinen Block in `clustered` Spaces anlegen.

### 18.4 Consumer und Change-Feed

§16.3 und §16.4. Einzige Schnittstelle für Strukturen, die inkrementell aufholen.

### 18.5 Kopf, Gewichte, Budget

`header` (opak), `weights`, `page` und `weight_sum`-Aggregate (§7.3) sind die Schnittstelle für Rang- und Budgetlogik oben. Der Kern kennt ihre Bedeutung nicht.

### 18.6 Stabilität

Ports, Werttypen und diese Verträge folgen semver in `contextra-storage-ports`. Formatänderungen laufen nach §1.5.

---

## 19. Nebenläufigkeit: Race-Freiheit durch Konstruktion

### 19.1 Prinzipien

1. Ein Schreiber (Committer) und serialisierte Catalog-Edits.
2. Unveränderlichkeit: Dateien nach `rename`, `Version` per `Arc`-Swap.
3. Besitz durch Claims.
4. Wahrheit durch durable Edits, nicht durch Speicherzustand.
5. Generationszähler statt Sperren für Schlüssel-Cache und Cluster-Zustand.
6. Keine Sperre über E/A, Ausnahmen nur Committer (`sync_data` des Redo) und `catalog_edit` (`sync_data` des Catalog).

### 19.2 Sperrränge (nur aufsteigend, je Thread höchstens einmal, `debug_assertions` prüft per Rangstack)

| Rang | Sperre | Über E/A gehalten |
|---|---|---|
| 1 | `committer` | ja (Redo-`sync`) |
| 2 | `intake` | nein |
| 3 | `catalog_edit` | ja (Catalog-`sync_data`) |
| 4 | `state` (`RwLock<Arc<Version>>`) | nein |
| 5 | Memtable-`RwLock` | nein |
| 6 | `registry` (Holds, Consumer, Claims, Restrictions) | nein |
| 7 | `cluster_table` | nein |
| 8 | `maint` (je Lane) | nein |
| 9 | `notify` | nein |

Ränge 6–9 sind Blätter. Cache- und Governor-Sperren sind Blätter. Rang 1 und Rang 3 werden **gleichzeitig nur** im Reservierungs-Fallback (§11.8) gehalten, in dieser Reihenfolge und kurz. Jede andere Reihenfolge ist verboten.

### 19.3 Nahtstellen

| Nahtstelle | Gefahr | Lösung | Nachweis |
|---|---|---|---|
| Lesen × Unlink | Datei weg, Leser hält Verweis | Graveyard + `Arc<Version>` (`INV-GRAVE-1`) | loom, Sim |
| Compaction × Erase | Compaction schreibt Gelöschtes zurück | Claim-Konflikt ⇒ Compaction wird abgebrochen, Ausgabe verworfen; Edit prüft `files_remove ⊆ Claims` | TLA+ M3, Sim |
| Flush × Erase | Memtable enthält Daten des Clusters | Gate vor Memtable-Einfügung (§12.1 3a), Flush-Barriere ER4, Memtable-Bereich als `ErasedMarker` überschrieben | Sim, M2 |
| Schlüssel-Cache × Vernichtung | Entsiegeln mit zwischengespeichertem Schlüssel nach ER2 | `gen`-Prüfung vor **und** nach dem Entsiegeln (§19.5) | loom |
| Checkpoint × Erase | Checkpoint kopiert Dateien, die Erase ersetzt | Checkpoint nimmt `Arc<Version>` und `erase_epoch`, serialisiert gegen ER6 über `catalog_edit`; Restore wendet Ledger an | M4, Sim |
| Consumer × Erase | Consumer verliert Versionen | Consumer ist Hold; Löschung zerstört Daten nur im gelöschten Cluster, Consumer erhält `Erased`-Change | M2 |
| Rebucket × Lesen | Eintrag doppelt oder nie | `cluster → Gruppe` wechselt im selben Edit (`INV-PL-4`) | M5 |
| Zwei Erase mit überlappender Hülle | gegenseitige Blockade | Serialisierung nach Ticket-ID in `urgent`; der zweite Plan wird nach dem ersten neu berechnet | Sim |
| Restriction × Erase | Löschung läuft während `restrict` | `restrict` und ER1 laufen im selben `catalog_edit`-Sequenzpunkt; ER0 und ER1 prüfen beide (ER1 prüft nochmals im Edit) | M2 |
| Link × Erase | Hülle ändert sich zwischen ER0 und ER1 | ER1 berechnet die Hülle **im** Edit neu und lehnt ab, wenn sie sich vergrößert hat (Retry von ER0) | M2 |
| Idempotenz × Löschung | Token verrät Daten | Tabelle ohne Inhalte, TTL | Test |

### 19.4 Claim-Protokoll

```
claim(files, job) -> Ok(guard) | Err(Held{by})            atomar unter `registry`
Priorität: urgent > flush > compact > blobgc > idle
höhere Priorität setzt `job.cancel()`, der Job prüft zwischen Blöcken, gibt Claims frei, verwirft die Ausgabe
Ausgabe-Edit: Validierung `files_remove ⊆ gehaltene Claims`, sonst Edit abgelehnt (INV-CLAIM-1)
```

### 19.5 Cluster-Zustand und Schlüsselgenerationen

Der Zustand im Speicher ist ein `AtomicU64` (`state:8 | gen:56`). Gate, ER2 und ER6 erhöhen `gen`. Ein Leser prüft `state` am Anfang und `gen` nach dem Entsiegeln. Dadurch gibt ein Leser nach Rückkehr von ER2 keinen **neu** entsiegelten Klartext zurück (bereits geliefertes Material im Speicher des Aufrufers ist Grenze, §17.9).

### 19.6 Verbotene Muster

`unwrap/expect/panic/todo/unimplemented/unreachable/indexing_slicing`. Kein `std::sync::Mutex` außer über `sync::Ranked`. Keine Threads außerhalb des `Spawner`. Keine blockierende E/A unter Rang ≥ 4. Keine zweite Schreibquelle außer Committer und `catalog_edit`. Kein Rückruf (`Observer`, `SegmentOwner`) unter Sperren.

---

## 20. Verifikation

| Ebene | Werkzeug | Inhalt |
|---|---|---|
| Formal | TLA+/TLC (oder Quint) | M1 Commit/Sichtbarkeit/Recovery/Stamps, M2 Erase (ER0–ER7 inkl. Restriction, Link), M3 Compaction × Erase × Claims, M4 Checkpoint × Erase × Restore, M5 Rebucket |
| Modellprüfung Code | `loom` | Sperren, `Arc`-Swap, Generationszähler, Claims, Graveyard, Committer-Gruppenführung |
| Simulation | eigener Scheduler + `shuttle` | **Hauptwerkzeug gegen Races:** seeded Interleavings, `FaultFs`, `SimClock`, `SimEntropy`, Millionen Läufe, Seed reproduzierbar |
| Absturz | `FaultFs` | Abbruch an **jedem** E/A-Index, Torn Writes, verlorene Fsyncs, Rename-Reihenfolgen, alle INV nach Neustart |
| Eigenschaft | `proptest` | Codecs, `cmp`, Kompaktion äquivalent zum Referenzmodell (`BTreeMap` mit Versionen) |
| Linearisierbarkeit | Historienprüfer (Wing-Gong) | Historien aus der Simulation gegen sequenzielle Spezifikation |
| Löschorakel | `ErasureOracle` (`testing`) | Eindeutiges Muster in Schlüssel, Kopf, Körper (mit/ohne Seal); nach Erase **alle Bytes aller Dateien** durchsuchen (Runs, Blobs aller Inkarnationen, Redo, `.building`, Segmente, `MemFs`, Checkpoints) |
| Fuzz | `cargo-fuzz` | Dekoder aller Formate, Recovery auf beschädigten Verzeichnissen (≥ 8 Ziele) |
| Referenz | Python-Codierer | Goldene Bytes je Format |
| Statisch | clippy, `cargo-deny`, `layering`, `make metrics` | Lints, Lizenzen/Advisories, Schichten |
| Dauerlauf | Soak 72 h, Crash-Loop, Platte-voll | Stabilität, Speicherwachstum, Handle-Lecks |

**Orakelvertrag:** Bei `Plain`/`Physical`: nach Purge **0** Treffer für alle Muster. Bei `Cryptographic`: nach ER2 Körper-, Kopf- **und** Schlüsselmuster ohne Schlüsselzugriff unlesbar (0 Treffer im Klartext, auch im Redo); nach Purge zusätzlich 0 Reste an Ciphertext der Cluster.

**CI-Pflicht:** `check`, `fmt`, `clippy -D warnings`, `test`, `layering`, `loom`, `doc -D warnings`, `deny`, Sim- und Absturz-Kurzlauf. Nachtlauf: lange Sim, Fuzz, Orakel-Vollsuche. **Ohne CI sind keine Gates gültig.**

---

## 21. Konfiguration und Limits

| Parameter | Standard | Bemerkung |
|---|---|---|
| `max_key_bytes` | 1024 | |
| `block_target` | 16 KiB | |
| `value_separation_min` | 1 KiB | |
| `memtable_bytes` / `max_immutable` | 64 MiB / 3 | |
| `redo_segment_bytes` | 64 MiB | |
| `redo_align` | 4096 | |
| `max_pack_groups` | 4096 | |
| `pure_threshold` | 8 MiB | |
| `max_open_files` | 1024 | |
| `maint_io_fraction` | 0,5 | |
| `purge_batch_min` | 16 | |
| `purge_deadline` | **kein Standard** | Pflichtparameter je Erase-Aufruf. `Config` mit `clustered`-Space ohne `default_purge_deadline` ist erlaubt, `Eraser` verlangt dann den Parameter. |
| Stamp-Reservierung | 2^20 | |
| ID-Reservierung | 2^16 | |
| `max_links` je Cluster | 8 | |
| `max_hold_ttl` | 24 h | |

---

## 22. Invarianten (Auswahl, vollständiger Katalog im Test-Index)

| ID | Aussage |
|---|---|
| INV-STAMP-1 | Stamps streng monoton über angenommene Changes |
| INV-STAMP-2 | Kein Umbruch, Überlauf ⇒ `Exhausted` |
| INV-STAMP-3 | Kein Stamp wird je zweimal vergeben, auch nicht nach Absturz |
| INV-ID-1 | Dateikennungen und `tmp_id` werden nie wiederverwendet |
| INV-ATOM-1 | Ein Change ist ganz oder gar nicht sichtbar, auch nach Absturz |
| INV-COMMIT-2 | Ein sichtbarer Eintrag ist durabel (außer `Relaxed`) |
| INV-VIEW-1 | Eine View bei `V` sieht stets dieselben Daten |
| INV-LOCK-1 | Sperren nur aufsteigend, je Thread höchstens einmal |
| INV-CAT-1 | Datei lebendig ⇔ durabler `files_add` ohne `files_remove` |
| INV-FMT-1 | Nur kanonische Kodierungen, unbekannte Version/Tags ⇒ Fehler |
| INV-LOG-7 | Reservesegment ist stets vorallokiert |
| INV-GRAVE-1 | Keine Datei wird gelöscht, solange eine `Arc<Version>` auf sie zeigt |
| INV-CLAIM-1 | Ein Edit entfernt nur Dateien unter eigenem Claim |
| INV-PL-1 | Jede Datei ist Pure oder Packed, nie beides |
| INV-PL-2 | Blöcke und Abschnitte sind clusterrein, ein Cluster ist je Datei ein zusammenhängender Abschnitt |
| INV-PL-3 | Compaction mischt nur innerhalb einer Wartungsgruppe |
| INV-PL-4 | `cluster → Gruppe` wechselt nur im selben Edit wie die Dateimengen |
| INV-CLUSTER-1 | Ein `Purged`-Cluster wird nie wiederverwendet |
| INV-CLUSTER-2 | `ClusterId` ist nicht aus einer Kennung ableitbar (nur über `register`) |
| INV-ERA-1 | Logische Atomarität: für Views ≥ `S` ist der Cluster leer, für Views < `S` vollständig `Erased`, keine Mischsicht |
| INV-ERA-2 | Nach `Purged` enthält weder ein lebender Run noch ein lebendes Redo-Segment eine Version des Clusters |
| INV-ERA-3 | Reihenfolge ER1 < ER2 < ER6 < Unlink; bei `Cryptographic` kein ER3–ER6, solange `probe` Present meldet |
| INV-ERA-4 | Nur Roll-forward: ein begonnener Erase wird fortgesetzt oder ist nie begonnen |
| INV-ERA-5 | Reads unterhalb `S` liefern `Erased`, nie Daten, nie „leer“ |
| INV-ERA-6 | Nach `Purged` enthält keine lebende Blob-Inkarnation einen Record des Clusters |
| INV-ERA-7 | In `Seal::Cluster`-Spaces ist nach ER2 alles des Clusters ohne Schlüssel unlesbar |
| INV-ERA-8 | `ErasureReceipt` nur bei INV-ERA-2/6, durablem Ledger-Eintrag und bestätigten Segmenten |
| INV-ERA-9 | Nach Restore ist jeder Cluster aus dem Ledger wieder ≥ `Shredded` |
| INV-ERA-10 | Eine `Restriction` mit `BLOCK_ERASE` in der Link-Hülle verhindert jede Löschung (kein Teil-Erase) |
| INV-KEY-1 | Schlüsselmaterial liegt nie unverhüllt in Dateien und nie in Checkpoints |
| INV-KEY-2 | Kein Entsiegeln liefert Klartext, wenn sich `gen` während des Aufrufs geändert hat |

Jeder Test zitiert seine ID.

---

## 23. Leistung (Hypothesen, Messverfahren)

Alles **Entwurfsziele**. Freigabe nur mit Messbericht (Hardware, Dateisystem, Last, Methode, Streuung). Scheitert eine Hypothese, wird **sie** korrigiert, nicht das Verfahren gelockert.

| ID | Hypothese | Messung |
|---|---|---|
| H-1 | `apply`: Durchsatz ≥ 80 % von Fsync-Rate × mittlere Gruppengröße; p99 ≤ 2 × Fsync-Latenz | Mikrobench, variierende Schreiberzahl |
| H-2 | Punktabfrage warm: ≤ 1 Datenblock-Read je Treffer-Level, p99 ≤ 50 µs | Zipf-Last |
| H-3 | Kopf-Scan ohne Körper ≥ 10× Durchsatz gegenüber Scan mit Körper (Körper ≥ 4 KiB) | Scan-Bench |
| H-4 | `page` liest ≤ 1,2 × Blöcke des Minimums | Gewichtsverteilungen |
| H-5 | Schreibverstärkung Metadaten ≤ 15, Körper ≤ 2 | Compaction-Statistik |
| H-6 | ER0–ER2 für einen Cluster ≤ 10 ms (ohne Providerlatenz) | Löschbench |
| H-7 | Purge eines Clusters ∝ Größe seiner Packgruppe, nicht des Stores (Faktor ≤ 1,5 bei 10× Store) | Skalierungstest |
| H-8 | Pure-Cluster löschen ≤ 100 ms je 1000 Dateien (ohne Scrub) | Drop-Bench |
| H-9 | Bündelung: 1000 Cluster einer Gruppe ≤ 2 × Aufwand eines Clusters | Bündelungsbench |
| H-10 | Rohkopie beim Purge ≥ 80 % sequentielle Plattenbandbreite | Purge-Bench |
| H-11 | Seal-Aufschlag ≤ 15 % CPU bei Lastmix, Overhead kleiner Cluster ≤ 5 % Bytes (Cluster ≥ 1 KiB) | A/B-Bench |
| H-12 | Speicher ≤ `memtable_bytes × (1 + max_immutable)` + Cache + O(offene Gruppen) | RSS-Messung |
| H-13 | Recovery ∝ Redo-Rückstand + Catalog; 1 GiB Redo ≤ 10 s | Recovery-Bench |
| H-14 | Ribbon-Filter ca. 30 % kleiner als Bloom bei gleicher Fehlrate | Filterbench (Literaturwert, nachmessen) |

---

## 24. Phasen und Gates

| Phase | Inhalt | Gate |
|---|---|---|
| **P0 Fundament** | `ports`, `codec`, `env` (MemFs, RealFs, FaultFs, Sim), `sync`, CI, Lint-Gerüst, Referenzcodierer | **G-0:** SPEC R5 normativ (zwei Reviewer, §26 entschieden), CI grün |
| **P1 Format** | alle Formate §11 inkl. ClusterSection/Seal/Ledger; goldene Vektoren; ≥ 8 Fuzz-Ziele | **G-F Formatfreeze** (§11.9): Golden, Property, Fuzz, Review |
| **P2 Schreibpfad** | Committer mit echtem Redo (Blocker S-1), Reservierungen, Memtable, Recovery, Catalog | G-W: Absturzmatrix vollständig grün, M1 geprüft, Linearisierbarkeit in Sim |
| **P3 Lesepfad** | Flush, Runs (Pure/Packed), Cache, Views, `page` | G-R: Modellgleichheit, H-2…H-4 gemessen |
| **P4 Wartung** | Compaction je Gruppe, Claims, Blob-GC, Governor | G-M: M3 geprüft, Sim ohne Invariantenbruch, H-5 |
| **P5 Löschung** | Cluster, Links, Restrictions, ER0–ER7, Packgruppen-Abwurf, Ledger, Restore, Orakel | **G-E:** M2/M4/M5 geprüft, Orakel 0 Treffer in allen Matrizen, H-6…H-11 gemessen |
| **P6 Härtung** | Soak, Fuzz-Dauerlauf, Platte-voll, Fehlerinjektion, Facade stabilisieren | **G-P Produktionsreife:** alle INV getestet (100 %), Soak 72 h, vollständiger Messbericht §23, Sicherheitsreview Krypto-Nutzung |

Keine Phase beginnt vor dem Gate der vorherigen. Keine Statusänderung in `IMPLEMENTATION_SPEC.md` ohne Testnachweis. Erst nach **G-P** werden aufsetzende Komponenten angebunden; bis dahin enthalten Repo und Dokumente nur den Kern und die Verträge aus §18.

**Sofortmaßnahmen vor P1:** (1) Redo-Stub im Committer ersetzen (S-1), (2) `recovery`, `flush`, `compaction` in den Modulbaum (S-2), (3) `tests/engine` wieder kompilierbar, (4) `unwrap`/`panic` aus Produktionscode (S-8), (5) `default = ["testing"]` entfernen (S-9), (6) CI einrichten (BL-9), (7) `deny.toml` vervollständigen, (8) Toolchain-Pin klären (BL-7).

---

## 25. Entscheidungen

### 25.1 Register

| ID | Entscheidung | Begründung | Verworfen |
|---|---|---|---|
| SD-01 | Catalog als einzige Wahrheit, ein Edit = ein Durabilitätspunkt | Crash-Sicherheit | Zustand in Dateinamen |
| SD-02 | Clusterreine Blöcke, ein Abschnitt je Cluster und Datei (L2) | Löschen ohne Entschlüsseln und Neukomprimieren | gemischte Blöcke |
| SD-03 | Bewilligte Abhängigkeiten (§3.5) | Sicherheit ohne eigenes `unsafe` | eigener Baum im Kern |
| SD-04 | Eine Formatversion je Dateiart, Migration als Werkzeug | kein Zweitpfad im Kern | Rückwärtskompatible Leser |
| SD-05 | Seal=Cluster umfasst Schlüssel, Kopf, Körper, Index, Filter, Marker (L3) | kryptografische Löschung auch von Nutzerschlüsseln | nur Körper versiegeln |
| SD-06 | `purge_deadline` ist Pflichtparameter ohne Standard | Fristen sind Rechts-/Betriebsentscheidung | Standard 72 h im Kern |
| SD-07 | Pure/Packed mit Packgruppe (Mandant × Ablaufzeitraum) | Skalierung und O(1)-Ablauf | Datei je Cluster; Kohorten-Verzeichnisse |
| SD-08 | Rohkopie beim Purge (AAD ohne Dateiposition) | Bandbreite statt CPU | Entschlüsseln/Neuverschlüsseln |
| SD-09 | `Restriction` im Kern, **kein Override** | Mechanismus ohne Politik, löst den Konflikt Datenschutz/Aufbewahrung zugunsten der oberen Schicht | `HoldPolicy::Override` |
| SD-10 | Zwei Wartungsjobs plus `urgent` | Flush nie hinter Compaction | ein Job je Instanz |
| SD-11 | Fünf Erweiterungsverträge (§18) | Anbindung ohne Formatbruch | Nachrüsten |
| SD-12 | Kein 2PC, ein Redo-Frame, ein Catalog-Edit | Atomarität im Kern | verteilter Intent-Log |
| SD-13 | Hardlink-Checkpoints nur Opt-in | Purge erreicht Hardlink-Inodes nicht | Hardlink als Standard |
| SD-14 | XChaCha20-Poly1305 mit Zufallsnonce | kein Zählerzustand | AES-GCM mit Zähler, GCM-SIV (später prüfbar) |
| SD-15 | TLA+-Modelle M1–M5 als Gate | Race-Freiheit beweisen | nur Tests |
| SD-16 | Fsync-Fehler nie wiederholen | Fsync-Semantik | Retry-Schleifen |
| SD-17 | Cluster-Zustand `state|gen` als Atomic | Leser ohne Sperre | Sperre je Cluster |
| SD-18 | `header` opak mit `header_len`, `weights` und `weight_sum` im Kern | Kopf ohne Körper, Budget ohne Bedeutungswissen | strukturierter Kopf im Kern |
| SD-19 | Stamp-Reservierung im Catalog | `INV-STAMP-3` | Stamp aus Redo-Ende ableiten |
| SD-20 | Redo in ausgerichteten Gruppen, vorallokiert, Reservesegment | Torn-Write-Eingrenzung, kein `ENOSPC` im Rollover | fortlaufender Append |
| SD-21 | Blob-Inkarnation ohne Referenz-Umschreibung | Purge ändert keine Runs fremder Cluster | Blob-Generationen im `BlobRef`; reine Neukopie mit neuer `blob_id` |
| SD-22 | `LOCK` über `Fs::lock`, `ReadOnly` geteilt | Doppelöffnung verhindern | keine Sperre |
| SD-23 | Ledger getrennt von Catalog und Checkpoints | Backup-Problem | Ledger im Checkpoint |
| SD-24 | **Einclustrige** Einträge, Mehrbetroffenen-Daten über gemeinsame Cluster oder Links | keine Querverweise zwischen Clustern beim Purge, einfachere Beweise | Mehrfach-Scope-Einträge mit Wrap-Liste |
| SD-25 | Vergleichsfunktion `cmp` statt `memcmp` über Konkatenation | variable Schlüssellängen | Escaping im Schlüssel |

### 25.2 Unterschiede R4-Entwurf ↔ R3-Entwurf und Entscheidung der Zusammenführung

Hinweis: Der R4-Entwurf lag nur bis §11.1 vor (§1, L1–L2, §7, §10, Anfang §11). Alles Weitere ist **meine** Ausarbeitung nach dessen Linie. Wo R4 später etwas anderes sagt, geht R4 vor und R5 ist anzupassen.

| Thema | R4-Entwurf | R3-Entwurf (meiner) | R5 |
|---|---|---|---|
| Löschdomäne | **Cluster** = Löschdomäne = Schlüsseldomäne = Platzierung | `Scope` (Löschung) + `Kohorte` (Dateigrenze) getrennt | **Cluster** (R4). Aus meiner Kohorte wird die schmalere **Packgruppe**. |
| Dateien | Pure/Packed | Kohortenverzeichnisse | **Pure/Packed** (R4), Packgruppe als Ablaufgruppe (R3-Idee) |
| Schlüsselordnung | `(cluster, user_key, tag↓)`, clusterreine Blöcke | `scope_id ‖ user_key` | R4 (mit klarer Vergleichsfunktion) |
| Nutzerschlüssel | Vertrag: frei von personenbezogenen Daten | Regel P-K, Restexposition bis Purge | R4-Vertrag **plus** Seal: in versiegelten Clustern ist der Schlüssel geschützt (Verbesserung gegenüber beiden) |
| Mehrfach-Bezug | einclustrig (impliziert) | Scope-Menge 1–4 mit Wrap-Liste | **einclustrig + Cluster-Links** (meine Mehr-Scope-Lösung entfällt, SD-24) |
| Rechtlicher Sperrvermerk | `Restriction` im Kern | nur `set_gate`, `HoldPolicy::Override` | **Restriction, kein Override** (R4, SD-09) |
| Stamps | nie doppelt, auch nicht nach Absturz, Tag `(stamp<<8)|kind` | `Version=(Stamp, op_idx)` | R4 (SD-19) |
| Blob | keine Generationen/Hole/Indirektion, nie verändert | Generationen mit gleichen Offsets und Löchern | **Kompromiss:** keine Generation im `BlobRef` (R4), Inkarnation mit gleichen Offsets im Catalog (R3), weil sonst jeder Purge fremde Runs umschreiben müsste (SD-21) |
| Redo | ausgerichtete Gruppen, vorallokiert, Siegel, Reservesegment | Frames, Segmentlebenszyklus | R4 + versiegelte Nutzlast je Cluster (R3-Idee, SD-05) |
| Dateinamen/IDs | numerisch, `tmp_id` aus reserviertem Bereich | Pfade mit Kohorten | R4 |
| Instanzsperre | `LOCK`, `Perm::Private` | fehlte | R4 |
| Kennungen | `AT-n`, `AN-nn`, `ER-n` bereinigt | später vorgeschlagen (O-10) | R4 |
| Wertaufbau | `header_len`, Weights im Klartext | strukturierter Kopf (`class`, `weight`, `meta`, `lineage`) | R4 (SD-18). `class_mask` und `lineage` im Format entfallen. |
| Cluster-Ableitung | nicht sichtbar | `lineage` im Kopf, Mehr-Scope | **Cluster-Links** (neu) |
| Löschprotokoll | ER0–ER7 | G/K/P/D | ER0–ER7 (neu belegt, §17.5) |
| Fristen | nicht sichtbar | Standard 72 h | **Pflichtparameter** (SD-06) |
| Ledger/Backup | nicht sichtbar | Ledger getrennt, Restore wendet an | R3 übernommen (SD-23) |
| Verifikation | nicht sichtbar | TLA+, loom, Sim, Orakel | R3 übernommen |

### 25.3 Überprüfung meiner R3-Entscheidungen

| R3-Entscheidung | Urteil | Begründung |
|---|---|---|
| Scope + Kohorte als zwei Begriffe | **geändert** | zwei Einheiten erzeugten Zuordnungstabellen und eine Querverweis-Quelle für Races; Cluster + Packgruppe genügt |
| Mehr-Scope-Einträge mit Wrap-Liste | **verworfen** | zwingt Purge zu clusterübergreifenden Verweisen, bricht Clusterreinheit, `AnyDestroyed/AllDestroyed` ist Rechtspolitik im Kern. Links lösen den Fall mit weniger Zustand |
| `class_mask` im Index | **verworfen** | nicht begründet gegenüber Filtern im opaken Kopf, vergrößert die Freeze-Liste |
| `lineage` im Eintragskopf | **verworfen** | Ableitung wird über Cluster-Links statt Eintrag modelliert |
| Strukturierter Kopf | **verworfen** | Kopf bleibt opak (R4); weniger Format zum Einfrieren |
| `HoldPolicy::Override` | **verworfen** | Kern trifft keine Rechtsabwägung (§3.3); `Restriction` schließt die Lücke |
| Schlüssel-Klartext bis Purge (P-K) | **verbessert** | Seal=Cluster schützt auch Nutzerschlüssel; P-K gilt nur noch für Spaces ohne Seal |
| 72 h Standardfrist | **verworfen** | Rechtsfrage, Pflichtparameter statt Standard |
| `redo_max_age` als Hygienegrenze | **verworfen** | in Seal-Spaces nicht nötig, Redo ist nach ER2 unlesbar |
| Blob-Sweep mit Generationen in `BlobRef` | **geändert** | Inkarnation im Catalog statt in der Referenz |
| Kohortenverzeichnisse im Dateisystem | **verworfen** | Catalog kennt Gruppen, Verzeichnisse brächten nur Pfadlogik (SD-07) |
| Rang 3 `catalog_edit` hält E/A neben Rang 1 | **präzisiert** | nur im Reservierungs-Fallback gleichzeitig, sonst nie (§19.2) |
| Fsync nie wiederholen, Poison | **bestätigt** | |
| XChaCha mit Zufallsnonce | **bestätigt**, mit AAD ohne Dateiposition | ermöglicht Rohkopie |
| Ledger getrennt, Restore wendet an | **bestätigt** | löst das Backup-Problem sauber |
| TLA+-Modelle als Gate | **bestätigt**, Aufwand realistisch planen | M1–M5 sind nötig, aber die Kosten sind ein Planungsrisiko |
| `scan_budget` mit Block-Aggregaten | **bestätigt**, als `page` mit `ResumeToken` | |
| Hardlink-Checkpoints nur Opt-in | **bestätigt** | |

**Weitere Funde bei der Prüfung:**

- In R3 war der Vergleich von Schlüsseln nur beschrieben, aber nicht definiert; mit variablen Längen ist `memcmp` über Konkatenation falsch. R5 definiert `cmp` (SD-25).
- In R3 hing der Löschpfad am Cluster-Ablauf an einer Verzeichnisgrenze. Mit `Restriction` muss ein Abwurf einzelne Cluster aussparen. R5 beschreibt das (§16.6).
- Die Link-Hülle kann zwischen Vorprüfung und Gate wachsen. R5 berechnet sie im Gate-Edit neu (§19.3).

---

## 26. Offene Entscheidungen

| # | Frage | Wirkung | Vorschlag |
|---|---|---|---|
| O-1 | Wer setzt `Restriction`s in der Praxis, und gilt ein Ablaufdatum automatisch? | Rechts-/Prozessfrage der oberen Schicht | Ablauf lässt nur den Vermerk fallen, Aufhebung bleibt aktiv (`lift`) |
| O-2 | Ist die Purge-Frist je Anfrage tragfähig, genügt Körper-Unlesbarkeit sofort, und was gilt für Spaces **ohne** Seal? | Rechtsfrage, Datenschutzbeauftragter | Seal=Cluster für alle personenbezogenen Spaces vorschreiben |
| O-3 | Gemeinsame Cluster vs. Links für Mehrbetroffenen-Daten: welche Konvention gilt? | Daten mehrerer Personen | Gemeinsamer Fallcluster + Links aus den Personenclustern, Löschung eines Betroffenen entfernt den Fallcluster nur, wenn die obere Schicht das beauftragt |
| O-4 | Schlüsselhaltung in Produktion (HSM, Tresor, eigener Store): Wer liefert den `KeyProvider`? | Verfügbarkeit und Durabilität von `destroy` | Referenz in `crypto-ref`, Produktivprovider getrennt |
| O-5 | Inhalts-Hashes über Klartext (falls später gewünscht) verraten Inhalte; Hashes nur über versiegelte Daten bilden? | Dedup entfällt für Cluster-Daten | Ja; Dedup nur in Spaces ohne Cluster |
| O-6 | Packgruppen-Zeitraum (Monat/Quartal/Jahr) und `max_pack_groups` | Dateizahl gegen Überretention | Monat / 4096, aus Last ableiten |
| O-7 | Toolchain-Pin (BL-7), `GH_TOKEN`, CI-Plattform | Gates gelten erst mit CI | CI mit Pin auf ladbare Version |
| O-8 | Altbestand: `REPORT.md` (nicht reproduziert), `RunBuilder`, `spec/implementation-rev3`, `legacy/` | Hygiene | nach Freigabe kennzeichnen oder entfernen |
| O-9 | Der R4-Entwurf bricht nach §11.1 ab: sind Abschnitte (L3–L12, §3–§6, §8, §9, §12–§26) dort anders gedacht? | Anpassung R5 | R4 vollständig nachliefern, dann Diff |

---

## Anhang A — Referenzvektoren (Pflicht vor G-F)

Je Format mindestens: leeres Objekt, ein Eintrag, Eintrag mit versiegeltem Körper, Cluster-Abschnitt mit zwei Blöcken, Packed-Run mit drei Clustern, Tombstone, Bereichsmarker, Blob mit Loch (Inkarnation), Catalog-Edit-Kette mit Reservierung, Ledger-Kette, Redo-Gruppe mit zwei Frames, abgeschnittener und beschädigter Frame, nicht-kanonische `varint`. Erzeugung durch `tools/reference/generator.py` (unabhängig von Rust), Vergleich im Test `golden`.

## Anhang B — Beispielablauf: Löschung eines Betroffenen

1. Obere Schicht prüft ihre Rechtslage und ruft `eraser().cluster(c, Cryptographic, purge_deadline)`.
2. **ER0/ER1:** Restrictions geprüft, Link-Hülle bestimmt, Gate-Edit durabel (ms). Ab jetzt `Erased` für alle Lesepfade.
3. **ER2:** `KeyProvider::destroy`, Cache geleert, `Shredded`. Schlüssel, Kopf und Körper des Clusters sind in Redo, Runs, Blobs und Backups unlesbar.
4. `status`: `KeyDestroyed`, Frist sichtbar. Die obere Schicht kann antworten, dass Inhalte unlesbar sind.
5. **ER3–ER6:** gebündelt in der Frist: Pure-Dateien entfernen oder Packed-Dateien roh umkopieren, Blobs per Inkarnation, Segmentbesitzer bestätigen, ein Edit.
6. **ER7:** Unlink, Verzeichnis-Sync, Ledger-Eintrag, `ErasureReceipt`.
7. `verify(ticket)` und (in Tests) das Orakel bestätigen 0 Treffer.

## Anhang C — Zustände

```
Cluster:  Live ──ER1──► Retired(S) ──ER2──► Shredded ──ER6──► Purged     (nur vorwärts; Restrictions orthogonal)
Packgruppe: Aktiv ──PD1──► Abwurf ──PD5──► Entfernt
Datei:    Building ──rename+Edit──► Lebendig ──Edit──► Obsolet ──letzte Referenz──► Unlink
Erase:    Retired ─► KeyDestroyed ─► Purging ─► Done(Receipt)             (Absturz an jeder Stelle: Fortsetzen, nie Rückabwicklung)
```

# Contextra Spezifikation v4.0 (RAG-Edition)

**Datum:** 20. November 2026
**Status:** Final zur Umsetzung. Einzige normative Quelle für Produktumfang, Architektur, Löschkette und Betrieb.
**Geltung:** Gesamter Workspace `contextra`.
**Ersetzt:** `docs/SPEC.md` v2.1, `Contextra Löschspezifikation v2.2.md`, `contextra_implementierungsspezifikation_v3.md`. Diese drei Dokumente sind Quellarchiv, nicht mehr normativ, und werden in Meilenstein M0 nach `docs/archive/` verschoben.
**Kennungen:** `ADR-n` Entscheidung, `REQ-<Bereich>-n` Anforderung, `INV-n` Invariante, `REM-n` Entfernung, `T-n` Test, `OPEN-n` offener Punkt. MUSS, DARF NICHT, SOLL nach RFC 2119.

---

## 0. Zusammenfassung der Neuausrichtung

Contextra war für Roboter ausgelegt: harte Zeitgrenzen, Offline-Pflicht, Nachweise gegenüber Prüfern, Schutz vor Hardwareausfall. Der Zieleinsatz ist jetzt eine **RAG-Engine für mittelständische bis große Unternehmen**. Dort zählen Suchlatenz, Durchsatz, Speicherbedarf, Zugriffsrechte und eine Löschung, die niemand im Betrieb bemerkt.

Fünf Grundentscheidungen tragen diese Spezifikation:

1. **Löschung und Versionierung sind getrennte Systeme** (Kapitel 4). Die Löschung kennt keine Versionen. Sie arbeitet auf *Seals*, das sind verschlüsselte Inhaltseinheiten. Die Versionierung kennt keine Schlüssel. Sie verweist nur auf Inhalte.
2. **Keine Eigennachweise.** Die Löschkette misst, signiert und bescheinigt sich nicht selbst. Sie führt Operationen aus und protokolliert sie. Prüfung geschieht in Tests, nicht im Produktbetrieb (ADR-02).
3. **Die Schlüssel werden sofort vernichtet, die Aufräumarbeit läuft gebündelt im Hintergrund** (Kapitel 7). Nach wenigen Millisekunden ist der Inhalt nicht mehr lesbar, nach Sekunden ist der Schlüssel vernichtet, der Rest wird in Stapeln mit Drosselung erledigt.
4. **Suche ist ANN-first.** HNSW mit SQ8 und exakter Nachberechnung ist Standard, exakte Suche bleibt für kleine Bestände und als Referenz (ADR-04).
5. **Zugriffsrechte, Filter und Lösch-Sichtbarkeit sind dieselbe Bitmap-Mechanik** (Abschnitt 6.5). Die Löschung kostet im Suchpfad praktisch nichts.

Was in v2.2 und v3.0 gut war, bleibt: Schlüssel je Inhaltseinheit, Gate vor Entschlüsselung, Idempotenz mit Journal, Wiederanlauf nach Absturz, Mehrbetroffenen-Prüfung, Trockenlauf, Aufbewahrung und Legal Hold, Backup-Spuren getrennt, Restore-Schutz.

---

## 1. Produkt

### 1.1 Ziel

Contextra ist eine **eingebettete Rust-Bibliothek für Retrieval in RAG-Anwendungen** mit MCP-Schnittstelle (stdio) und einem optionalen Server-Binary. Sie nimmt Dokumente auf, zerlegt sie, erzeugt Embeddings über ein austauschbares Backend, indiziert Vektoren und Text, beantwortet Hybridsuchen mit Zugriffsfilter und liefert Kontextpakete mit Quellenangabe innerhalb eines Token-Budgets. Textgenerierung liegt außerhalb.

Zusätzlich sind verbindlich:

- **Löschung personenbezogener Daten** nach Kapitel 7.
- **Mandantentrennung** mit eigener Schlüsselhierarchie je Mandant.
- **Zugriffsfilter** (Security Trimming) auf Dokument- und Chunk-Ebene.
- **Versionierung** mit optionaler Zeitreise (Kapitel 4.5).

### 1.2 Referenzlast (Auslegung)

| Stufe | Chunks je Mandant | Dimension | Anfragen | Betrieb |
|---|---|---|---|---|
| S | 1 Mio. | 384–768 | ≥ 200 QPS | 16 vCPU, 64 GB RAM, NVMe |
| L | 10 Mio. | 768 | ≥ 100 QPS | 16 vCPU, 128 GB RAM, NVMe |

Bis 100 Mandanten je Prozess, jeder mit mehreren Collections. Größere Bestände skalieren über mehrere Prozesse, aufgeteilt nach Mandant (Kapitel 8.7).

### 1.3 Nicht-Ziele

- Keine Textgenerierung, kein Chat, keine Agentenlogik, kein Agentengedächtnis im Kern.
- Keine Echtzeit-, Regelungs- oder Hardwarefunktionen. Keine Zeitgarantien auf Suchantworten.
- Keine interaktiven Mehrfachtransaktionen. Ein Schreibstapel ist atomar (REM-12).
- Keine Replikation und kein Cluster im Kern in v4.0 (OPEN-3).
- Keine Aussage über Kopien außerhalb des Systems: externe Sicherungen, Dateisystem-Snapshots, Flash-Remanenz, Speicherabbilder, bereits ausgeleitete Daten.
- Kein Compliance-Versprechen. Die Software führt Operationen aus. Ob diese die Pflichten des Verantwortlichen erfüllen, bewertet der Verantwortliche (REQ-DOC-2).

### 1.4 Entscheidungsfilter

Jede Funktion wurde gegen drei Fragen geprüft. Fällt eine Antwort negativ aus, wird die Funktion entfernt oder zum optionalen Feature.

1. Hilft sie der Qualität, Latenz oder dem Durchsatz einer Unternehmens-RAG?
2. Ist sie für Löschung, Mandantentrennung oder Zugriffsschutz erforderlich?
3. Kostet sie im Normalbetrieb weniger als ihr Nutzen (gemessen, nicht geschätzt)?

---

## 2. Entscheidungsregister

Alle Entscheidungen sind final. Änderungen erfordern eine neue Spezifikationsversion.

| ID | Entscheidung | Begründung | Verworfen |
|---|---|---|---|
| ADR-01 | Löschung wirkt auf **Seals** (Inhaltseinheiten mit eigenem Schlüssel), nicht auf Versionen. Version und Seal sind getrennte Systeme mit einer Einbahn-Schnittstelle. | Eine Löschung darf nicht von der Versionskette, deren Bereinigung oder Zeitreise abhängen. Version-GC und Löschung laufen unabhängig. | Gate-Mengen über Versionsordinale (`ErasedVer`, `SupersededVer`) wie in v2.2 |
| ADR-02 | **Keine Eigennachweise** im Produkt: kein Canary, kein Nadelscan, keine Messklassen L/P/C/M, keine signierten Berichte, keine Bescheinigung, kein Prüfwerkzeug. Stattdessen Journal, Ledger und Tests in CI. | Jede Selbstmessung prüft sich mit denselben Annahmen, kostet Laufzeit und erzeugt Aussagen, die der Betreiber nicht verwerten kann. | Messung A/B, `Scanner<Armed>`, JCS und Ed25519, `contextra-verify` |
| ADR-03 | **Frühe Schlüsselvernichtung ist der Standard.** Reihenfolge: Gate, Schlüssel, Arbeitsspeicher, danach gebündelte Bereinigung gemeinsamer Strukturen. | Ohne Messschritt entfällt der Grund für die Reihenfolge „erst bereinigen“. Aller Klartext, der nur über den Schlüssel erreichbar ist, ist sofort weg. | Reihenfolge S3, S4, S5 aus v3.0 |
| ADR-04 | **HNSW mit SQ8** ist der Standardindex. Exakte flache Suche bis 20.000 Vektoren je Shard und als Referenz im Test. DiskANN und RaBitQ entfallen. | 200.000 flache Vektoren genügen nicht für Unternehmensbestände. Flache Suche kostet ab 1 Mio. Vektoren zu viel Latenz. | Flach als Standard (v3.0), DiskANN |
| ADR-05 | **Graph-Crate wird reduziert**(Abschnitt 12.3). | reduzieren. | Verschlanken |
| ADR-06 | `contextra-mvcc` wird zu **`contextra-version`**: Versionskatalog, Snapshots, Aufbewahrung. Kein SSI, kein TxBuffer. | RAG braucht atomare Schreibstapel und Snapshot-Lesen, keine serialisierbaren Transaktionen. | SSI, interaktive Transaktionen |
| ADR-07 | **Ein Schreib-Task je Shard** (Mandant × Collection), viele Shards parallel. Lesen ist sperrfrei über unveränderliche Snapshots. | Der Knoten-weite Einzelschreiber aus v2.2 bremst Mehrmandantenbetrieb. | Ein Schreiber je Prozess |
| ADR-08 | **KeyTable als Slot-Datei mit Überschreiben an Ort** statt Shard-Neuschreiben. | Vernichtung kostet einen 80-Byte-Schreibvorgang statt eines Shards von Dutzenden MB. Gestapelte Vernichtungen teilen sich einen `fdatasync`. | 256 Shards mit `tmp → rename` (v2.2) |
| ADR-09 | **Gebündelte Bereinigung** (Scrub) gemeinsamer Strukturen mit Frist, Drosselung und Priorität unter dem Suchpfad. | Pro Auftrag ein Segment-Neuschreiben skaliert nicht. Ein Stapel schreibt jedes betroffene Segment einmal. | Sofortige Bereinigung je Auftrag |
| ADR-10 | **Mehrbetroffene: `Restricted` mit Review** als Standard. `DeleteAll` nur durch ausdrückliche Entscheidung (Vier-Augen optional). `RedactReingest` ist ein Entscheidungsweg. | Fremde Dokumente ohne Zustimmung zu vernichten ist nicht in jedem Fall verhältnismäßig. | `DeleteAll` als Standard (v2.2) |
| ADR-11 | **Kein Cluster-Segmentieren personenbezogener Daten** (ER-26/27 entfallen). Jeder Seal hat einen eigenen Schlüssel. Vektor- und Textnutzdaten sind je Seal verschlüsselt. | Schlüssel je Seal macht Dateitrennung überflüssig und beseitigt das Restrisiko Gruppierung. | Cluster-Segmente, `cluster_seg_min` |
| ADR-12 | **Ingest-Crate und Embedding-Backends werden aus der Git-Historie zurückgeholt** (Abschnitt 13.2). | ~21.000 Zeilen Format-Extraktoren (PDF, DOCX, E-Mail, XLSX) und ein ONNX-Embedder existieren bereits und sind getestet. | Neuschreiben |
| ADR-13 | **Embedding-Backends:** lokal (ONNX, Standard) und HTTP gegen interne Inferenzdienste (eigenes Crate, Feature). Der Kern bleibt netzfrei. | Unternehmen betreiben meist zentrale Inferenzdienste. Netzwerkcode gehört in Adapter. | Nur lokal (v3.0) |
| ADR-14 | **Zeitreise ist je Collection schaltbar, Standard aus.** Suchindizes enthalten nur die aktuelle Fassung. Historie liegt kalt im Store. | Historische Chunks im Index vergrößern Speicher, Latenz und Löschaufwand mit jeder Revision. | Historie immer im Index |
| ADR-15 | **Schlüsselwurzel:** K1 (Passphrase, Argon2id) und K1b (Schlüsseldatei, Rechte 0600) in v4.0. K3 (externer KMS) in M6. K2 entfällt als eigene Stufe. | Passphrase-Eingabe passt nicht zu unbeaufsichtigtem Serverbetrieb. KMS ist die übliche Unternehmensanbindung. | K1 allein (v3.0), TPM-Stufe |
| ADR-16 | **Eigene LSM behalten, unter Gate G-STORE** (Abschnitt 12.3). Fällt der Benchmark durch, ersetzt `redb` oder `fjall` sie hinter dem Port `KvStore`. | 35.000 Zeilen sind versunkene Kosten, die Benchmarks fehlen aber. Die Entscheidung gehört auf Messwerte. | Sofortiger Ersatz |
| ADR-17 | **Zugriffsfilter ist Kernfunktion.** ACL-Tags und Metadatenfilter laufen über Bitmaps und die ACORN-Traversierung. | Ohne Security Trimming ist eine Unternehmens-RAG nicht einsetzbar. | Filterung in der Anwendung |
| ADR-18 | **Bulk-Ingest darf mit gelockerter Dauerhaftigkeit laufen** (Synchronisierung am Jobende). Löschvorgänge sind nie gelockert. | Initiale Importe von Millionen Dokumenten sind mit `fsync` je Gruppe zu langsam. Ein Abbruch wiederholt den idempotenten Job. | Immer volle Dauerhaftigkeit (v3.0) |
| ADR-20 | **Standard ist Klasse P.** Jede Collection gilt als personenbezogen, bis der Betreiber sie ausdrücklich als K erklärt. Die Entity-Erkennung warnt, entscheidet aber nie über den Speicherweg. | Der Standardweg darf nicht von der Trefferquote einer Erkennung abhängen. | Klassifikation beim Ingest entscheidet den Weg |
| ADR-19 | **Selbstdeklarationen entfallen:** `contract.rs`, `ANCHOR`-, `LINK`-, `FILE-CONTEXT`-Kommentare, Auditor-Skripte. Verhalten prüft die Konformitätssuite in `contextra-testkit`. | Selbstbeschreibung wird nicht geprüft und veraltet. Tests schon. | Deklarationen pflegen |

---

## 3. Architektur

### 3.1 Zielbestand der Crates

Umfang alt = Zeilen laut Funktionsinventar (inkl. Tests). Umfang Ziel ist eine **Schätzung** (±25 %), kein Versprechen.

| Crate | Ebene | Entscheidung | Alt | Ziel |
|---|---|---|---|---|
| `contextra-ids` | E0 | behalten. Einziger Ort für Limits (`MAX_SEARCH_K` u. a.) und `SealOrd` | 1.484 | 1.500 |
| `contextra-env` | E0 | behalten (`Clock`, `SeededRng`, `MetricsSink`) | 2.658 | 2.000 |
| `contextra-sys` | E0 | behalten (unsafe-Insel: mlock, Direct-IO) | 2.987 | 2.500 |
| `contextra-simd` | E0 | behalten (unsafe-Insel). Abhängigkeit auf `types` entfernen | 6.461 | 3.500 |
| `contextra-durable-fs` | E0 | behalten. Fault-Injection raus (REM-13) | 3.491 | 2.500 |
| `contextra-types` | E0 | stark verschlanken (REM-24, REM-28) | 9.596 | 3.000 |
| `contextra-ports` | E0 | stark verschlanken, nur Traits (Abschnitt 4.4) | 7.208 | 2.500 |
| `contextra-crypto` | E1 | umbauen: AEAD, KDF, KeyTable, Wurzel-Port | 11.953 | 4.000 |
| `contextra-erasure` | E1 | **neu** nach Kapitel 7 (Altbestand: REM-10) | 6.708 | 5.000 |
| `contextra-version` | E1 | **neu aus `contextra-mvcc`** | 8.159 | 2.500 |
| `contextra-store` | E1 | verschlanken unter G-STORE | 35.383 | 14.000 |
| `contextra-vector` | E1 | HNSW, SQ8, exakt, Filter. DiskANN, RaBitQ, `compat/` raus | 34.675 | 11.000 |
| `contextra-text` | E1 | BM25, WAND, Tokenizer, Morphologie. `compat/` raus | 18.176 | 10.000 |
| `contextra-rank` | E1 | RRF, Top-k, Zeitgewichtung, Erklärung, Drift | 10.809 | 4.000 |
| `contextra-privacy` | E1 | Entity-Erkennung, Egress-Guard, Verarbeitungsregister | 9.278 | 3.500 |
| `contextra-checkpoint` | E1 | hartlink-basierter Snapshot und Restore | 10.901 | 3.000 |
| `contextra-ingest` | E2 | **zurückholen** aus Historie, beschneiden | – | 14.000 |
| `contextra-embed` | E2 | **neu aus Historie** (ONNX), Reranker-Port | – | 2.500 |
| `contextra-db` | E2 | Orchestrierung: Schreib- und Lesepfad, Scheduler, Wiederanlauf | 412 | 5.000 |
| `contextra` | E2 | Fassade, reparieren | 3.807 | 1.500 |
| `contextra-mcp` | E3 | MCP über stdio, verschlanken | 7.815 | 2.500 |
| `contextra-testkit` | E3 | behalten, nicht im Standard-Build | 7.389 | 7.000 |
| `contextra-bench` | E3 | Budgets, A/B gegen Basislinie | 5.199 | 3.000 |
| `contextra-embed-http` | E3 | **neu**, Feature, eigenes Crate (M5) | – | 1.200 |
| `contextra-server` | E3 | **neu**, optional, HTTP/gRPC mit Auth (M6) | – | offen |
| `contextra-kms` | E3 | **neu**, optional, KMS-Anbindung (M6) | – | 1.000 |

Entfernt: `contextra-engine`, `contextra-vault`, .
Gesamtschätzung: **rund 110.000 Zeilen gegenüber rund 249.000 heute** (ohne `server` und `kms`).

### 3.2 Schichten und Abhängigkeitsregeln

```
E3  mcp  server  embed-http  kms  testkit  bench
E2  contextra (Fassade) ── db ── ingest  embed
E1  store  vector  text  rank  version  crypto  erasure  privacy  checkpoint
E0  ports  types  ids  env  sys  simd  durable-fs
```

- **R1** Kanten zeigen abwärts. `ports` hängt nur von `ids` ab, nicht von `types`.
- **R2** **E1-Crates hängen nie von anderen E1-Crates ab.** Die Verbindung geschieht über Traits aus `ports` und die Verdrahtung in `db`. Das gilt insbesondere für `store ↛ crypto`, `store ↛ version`, `erasure ↛ crypto`, `erasure ↛ version`, `version ↛ erasure`.
- **R3** `default = []` in jedem Crate. Optionale Funktionen sind Features.
- **R4** Jedes Konzept genau einmal definiert. Kryptoprimitive nur in `contextra-crypto`. Limits nur in `contextra-ids`.
- **R5** `#![forbid(unsafe_code)]` in jedem Crate außer `sys` und `simd`. Workspace-Lint von `deny` auf `forbid`.
- **R6** Kein `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!` außerhalb von Tests.
- **R7 (neu)** Keine `compat/`-Module. Ein Crate implementiert die Ports direkt (REM-15).
- **R8 (neu)** Kein Netzwerk im Kern. Netzwerkcode lebt nur in `embed-http`, `server`, `kms`.

**REQ-NET-1.** Der Abhängigkeitsbaum der Kern-Crates (E0 bis E2, `mcp`) enthält keine Netzwerk-Crates (Sperrliste in G-05). Adapter-Crates dürfen Netzwerk nutzen, nur hinter Features und nie als Default. Jeder ausgehende Endpunkt steht im Verarbeitungsregister (REQ-PRV-4) und läuft durch den Egress-Guard (REQ-PRV-3).

### 3.3 Verzeichnisstruktur der Daten

```
<data_dir>/
  LOCK                                   Prozesssperre
  tenants/<tenant>/
    keys/keytable.cxkt                   Slot-Datei (Abschnitt 5.3)
    keys/anchor.cxan                     gewrappte KEK-Epochen, Wurzel-Metadaten
    erasure/erasure.cxel                 Löschlog (Abschnitt 7.8)
    erasure/gate.snap                    Gate-Snapshot, atomar ersetzt
    erasure/ledger.jsonl                 Ledger, nur anhängen
    registry/                            Seal-Registry (KV-Familie im Store)
    collections/<collection>/
      store/                             LSM: WAL, SSTables, Manifest
      vector/                            Graph, Deltas, Header
      text/                              Postings-Segmente
      versions/                          Versionskatalog
```

Ein Mandant ist ein Verzeichnis und ein Schlüsselsatz. Mandanten-Offboarding ist O(1) in der Schlüsselarbeit (Abschnitt 7.2).

---

## 4. Entkopplung von Löschung und Versionierung

Dieses Kapitel ist der Kern der Neukonzeption.

### 4.1 Problem im Altbestand

In v2.1, v2.2 und v3.0 hängen beide Systeme aneinander: Der Schlüssel gehört zur Version (`DocDek je Version`), das Gate führt Mengen über Versionsordinale (`ErasedVer`, `SupersededVer`), die Zeitreise braucht eine Sonderregel (ER-6), Lineage trägt `source_version`, und Löschen verlangt das Auflösen der Versionskette. Jede Änderung an einem System berührt das andere.

### 4.2 Das Seal-Modell

Ein **Seal** ist eine unveränderliche Inhaltseinheit. Er entsteht beim Schreiben einer Dokumentrevision und enthält deren Chunks, Texte und Vektoren. Er hat:

- eine dichte, nie wiederverwendete Ordinalzahl `SealOrd(u32)` je Mandant,
- einen eigenen zufälligen Schlüssel `DocKey` (32 Byte) in der KeyTable,
- einen Eintrag in der **Seal-Registry**: `SealOrd → (DocId, Collection, Slotbereich der Chunks, Betroffenen-Tags, Klasse)`.

Die Versionierung speichert pro Revision nur einen undurchsichtigen `ContentRef = SealOrd`. Sie öffnet ihn nie.

| Frage | Besitzer |
|---|---|
| Welche Fassung gilt zum Zeitpunkt T? Wie lange wird welche Fassung aufbewahrt? | `contextra-version` |
| Ist dieser Inhalt lesbar, eingeschränkt oder vernichtet? Wo liegt sein Schlüssel? | `contextra-erasure` und `contextra-crypto` |
| Welche Seals gehören zu einem Dokument oder Betroffenen? | Seal-Registry (Löschseite, nicht Versionsseite) |

### 4.3 Einbahn-Schnittstelle

```
Version ──(ContentRetired)──► Erasure         Aufbewahrung abgelaufen: Seal vernichten
Erasure ──────────────────✗──► Version        keine Rückwirkung, nie
```

- Wenn die Versions-GC eine Revision entfernt und kein anderer Eintrag ihren Seal nutzt, sendet sie `ContentRetired(seal, RetireReason::RetentionExpired)`. Die Löschseite vernichtet den Seal mit Grund `Retention` im Ledger.
- Eine Löschung nach Kapitel 7 ändert den **Versionskatalog nicht**. Revisionen, deren Seal vernichtet ist, bleiben als Metadaten (Zeitstempel, Revisionsnummer) stehen. Ihr Inhalt liefert `Erased`.
- Die Versionierung hat keine Kenntnis von Schlüsseln, Gate, Betroffenen und Saga.

### 4.4 Traits (in `contextra-ports`)

```rust
pub struct SealOrd(pub u32);
pub struct ChunkSlot(pub u32);

pub enum SealState { Open, Restricted, Erased }

/// Lesesperre. Besitzer: contextra-erasure.
pub trait ReadGate: Send + Sync {
    fn state(&self, s: SealOrd) -> SealState;       // O(1), keine E/A
    fn gate_seq(&self) -> u64;                       // wächst mit jeder Gate-Änderung
}

/// Sichtbarkeit nach Versionsstand. Besitzer: contextra-version.
pub trait Visibility: Send + Sync {
    fn is_current(&self, c: ChunkSlot) -> bool;
    fn visible_at(&self, c: ChunkSlot, seq: u64) -> bool;   // nur bei index_history
}

/// Verschlüsselung je Seal. Besitzer: contextra-crypto, Nutzer: store, db.
pub trait RecordSealer: Send + Sync {
    fn seal(&self, s: SealOrd, kind: RecordKind, aad: &[u8], pt: &[u8], out: &mut Vec<u8>) -> Result<()>;
    fn open(&self, s: SealOrd, kind: RecordKind, aad: &[u8], ct: &[u8], out: &mut Vec<u8>) -> Result<(), OpenError>;
}
pub enum OpenError { KeyAbsent, Integrity }   // KeyAbsent wird auf dem Lesepfad zu Erased

/// Einbahn-Ereignis von der Version zur Löschung.
pub trait ContentLifecycle: Send + Sync {
    fn content_retired(&self, s: SealOrd, why: RetireReason);
}

/// Jede Zwischenspeicherung, die Inhalte hält, trägt Seal-Tags.
pub trait SealScoped: Send + Sync {
    fn invalidate_seals(&self, seals: &RoaringBitmap);
}
```

### 4.5 Versionierung

**REQ-VER-1 (Katalog).** `contextra-version` führt je Dokument eine Kette `(rev, seq, valid_from, valid_to, content_ref, state)`. `state` ist `Current`, `Superseded` oder `Retired`.

**REQ-VER-2 (Snapshots).** Lesen läuft gegen einen Snapshot mit Sequenznummer. Das Anheften verhindert die GC der Fassungen, die der Snapshot sieht. Es gibt keine Schreibtransaktionen, nur den atomaren Schreibstapel (REQ-CON-6).

**REQ-VER-3 (Aufbewahrung je Collection).** `version_retention`: `LastN(n)` mit Standard `n = 3`, `Days(d)` oder `KeepAll`. Ausgelaufene Revisionen werden aus dem Store entfernt (normale Kompaktion). Danach `ContentRetired`.

**REQ-VER-4 (Löschen im Normalbetrieb).** `delete_document(doc)` setzt die aktuelle Revision auf `Retired`. Die Collection-Richtlinie `on_delete` ist `Retain` (Standard, Inhalt bleibt bis Aufbewahrungsende) oder `Shred` (sofort `ContentRetired`). Das ist **keine** Löschung nach Kapitel 7.

**REQ-VER-5 (Zeitreise, ADR-14).**
- `index_history = false` (Standard): Suchindizes enthalten nur `Current`. `as_of` bei `get` liest die Revision aus dem Store. `as_of` bei `search` wird abgewiesen mit `HistoryNotIndexed`.
- `index_history = true`: Chunks tragen `seq_from, seq_to`. `Visibility::visible_at` filtert. Der Aufwand in Speicher und Latenz steigt mit der Revisionszahl und wird im Benchmark ausgewiesen.
- `valid_at` ist ein Bereichsfilter auf `valid_from`/`valid_to` (Abschnitt 6.5) und von `as_of` unabhängig.

**REQ-VER-6 (Zeitreise und Löschung).** Die Zeitreise prüft das Gate nach Seal gegen den **aktuellen** Stand. Eine Sonderregel gibt es nicht, sie folgt aus der Trennung: Die Version wählt aus, das Gate gibt frei.

### 4.6 Zusammensetzung im Lesepfad

```
live(Anfrage) = visible(Version) ∩ open(Gate) ∩ acl(Principal) ∩ meta(Filter)
```

Alle vier sind Bitmaps über Chunk-Slots oder Seal-Ordinale. Die Löschung fügt keinen eigenen Pfad hinzu (REQ-SRC-3).

**INV-1.** Kein Crate außer `db` kennt gleichzeitig `ReadGate` und `Visibility`.

---

## 5. Datenmodell und Speicher

### 5.1 Datenklassen

| Klasse | Inhalt | Verschlüsselung | Löschmechanismus |
|---|---|---|---|
| **P** | Dokumente, Chunks, Vektoren einer Collection mit `personal_data = Possible` (Standard) | `DocKey` je Seal, Einzeldatensätze | Schlüsselvernichtung (Abschnitt 7.4) |
| **K** | Collection mit `personal_data = None`, ausdrücklich vom Betreiber erklärt | `ArtKey` je Segment, optional aus (`k_class_encryption`) | Gate, Scrub durch Segmentersatz, keine Schlüsselvernichtung |
| **D** | Abgeleitetes: Postings, Graphstruktur, Caches | wie Quelle bzw. im Arbeitsspeicher | Gate, Scrub, `SealScoped` |
| **E** | Löschlog, Ledger, KeyTable, Registry | eigene Schlüssel, siehe 5.3 | Aufbewahrungsfrist des Ledgers |

**ADR-20 (Standard ist P).** Der Standard verlässt sich **nicht** auf die Erkennung personenbezogener Daten. Jede Collection gilt als P, bis der Betreiber sie als K erklärt. Die regelbasierte Entity-Erkennung (REQ-PRV-1) warnt bei Treffern in K-Collections. Sie entscheidet nie über den Speicherweg.

### 5.2 Seal-Aufbau und Nutzdatenarten

Jeder Seal besteht aus Datensätzen der Arten:

| Art | Inhalt | Verwendung |
|---|---|---|
| `DocBlob` | Originaldokument | Abruf, Re-Embedding |
| `ChunkText` | Text und Provenienz je Chunk | Ausgabe, Kontextpaket |
| `VectorCode` | SQ8-Code je Chunk (Dimension Bytes) | Start: Aufbau des Suchindex |
| `VectorFull` | fp32-Vektor je Chunk | Nachberechnung der Top-Kandidaten, Neuaufbau |
| `SealedMeta` | `external_id`, Quellpfad, nicht filterbare Metadaten | Anzeige |

**REQ-FMT-1.** Datensatz: `[seal u32][kind u8][chunk u32][nonce 12][ct …][tag 16]`. AEAD ist AES-256-GCM mit zufälliger Nonce. AD = `tenant ‖ collection ‖ seal ‖ kind ‖ chunk`. Je Schlüssel liegen weit weniger als 2³² Datensätze vor.

**REQ-FMT-2 (kein Klartext in Metadaten).** Quellpfade, Dateinamen und `external_id` können Personennamen enthalten. Sie liegen nur in `SealedMeta`. Der Zugriff über `external_id` läuft über `BLAKE3-keyed(K_ref, external_id)`. Klartextmetadaten sind auf die vom Betreiber deklarierten Filterfelder begrenzt (Abschnitt 6.5) und dürfen laut Vertrag keine Betroffenen identifizieren.

**REQ-FMT-3.** Jedes persistierte Objekt trägt Magic, Formatversion und Algorithmuskennung. Ändert sich das Layout, steigt die Version.

**REQ-FMT-4 (handkodierte Formate).** Genau drei Plattenformate sind handkodiert mit geprüften Feldzugriffen: `CXKT` (KeyTable), `CXEL` (Löschlog), Datensatzumschlag aus REQ-FMT-1. Alles andere nutzt bestehende Kodierungen mit Versionsfeld. Ungültige Längen führen zu `Integrity`.

### 5.3 Schlüsselhierarchie und KeyTable

```
Wurzel (K1 Passphrase / K1b Schlüsseldatei / K3 KMS)
  └─ MandantenKEK[epoche]      zufällig, im Anker gewrappt
       ├─ DocKey je Seal        zufällig, in der KeyTable gewrappt
       ├─ ArtKey je Segment     K, D, Registry
       └─ strukturelle Schlüssel per HKDF: K_ref, K_registry
```

**REQ-KEY-1.** `DocKey` und `ArtKey` kommen aus `OsRng`. Sie werden nie per HKDF abgeleitet. HKDF ist nur für strukturelle Schlüssel (`K_ref`, `K_registry`) zulässig. Fehler der Zufallsquelle führen zu `Err`.

**REQ-KEY-2 (Wrap).** AES-256-GCM-SIV unter `MandantenKEK[epoche]`, Nonce 96 Bit aus `OsRng`, AD = `seal ‖ epoche ‖ algo_id`.

**REQ-KEY-3 (Slot-Datei).** `keytable.cxkt` ist ein Kopf plus ein Array fester Slots. Slot-Index = `SealOrd`. Ein Slot hat 80 Byte:

```
[state u8][pad 3][epoche u32][wrap_nonce 12][wrapped 48][crc32c u32][pad 8]
state: 0 Frei, 1 Live, 2 Destroyed
```

**REQ-KEY-4 (Vernichtung).** `Destroyed` heißt: `wrapped` und `wrap_nonce` mit Nullen überschrieben, `state = 2`, CRC neu berechnet, per `pwrite` an Ort und Stelle. Mehrere Vernichtungen eines Stapels werden nach Offset sortiert und teilen sich **einen** `fdatasync`. Anschließend `zeroize` aller Kopien im Speicher.

**REQ-KEY-5 (Epochenwechsel).** Alle `epoch_len` (Standard 30 Tage) entsteht eine neue `MandantenKEK`. Ein Hintergrundlauf liest alle `Live`-Slots sequentiell und wrappt sie um. Slots mit gemischten Epochen sind zulässig. Wenn kein Slot mehr die alte Epoche trägt, wird die alte KEK im Anker überschrieben. Aufwand: O(Anzahl Seals), sequentielle E/A, bei 10 Mio. Seals rund 800 MB je Lauf. Das ist die ausgleichende Maßnahme gegen Dateisysteme, die Überschreiben nicht physisch ausführen (Copy-on-Write, Flash). Sie begrenzt den Zeitraum, in dem eine verbliebene Kopie eines gewrappten Schlüssels noch nutzbar wäre.

**REQ-KEY-6 (Schlüsselcache).** Entwrappte `DocKey` liegen in einem begrenzten LRU-Cache (`key_cache_entries`, Standard 65.536) in gesperrtem Speicher (`mlock`). Kein `Clone`, `Debug`, `Serialize`. `Zeroizing` beim Verwerfen.

**REQ-KEY-7 (Wurzel).** Das Trait `RootKeyProvider` liefert den Schlüssel für die Anker-Entwrappung. Implementierungen: `Passphrase` (Argon2id, m ≥ 64 MiB, t ≥ 3, p ≥ 1, mindestens 250 ms auf der Zielhardware), `KeyFile` (Rechte 0600 geprüft, Inhalt 32 Byte, `mlock`), später `Kms` (M6). Der Start verweigert (`Refused`) bei leerer Passphrase, schwachen KDF-Parametern oder falschen Dateirechten.

### 5.4 Seal-Registry

KV-Familie im Shard-Store, geschützt mit `K_registry`. Schlüssel: `SealOrd`. Wert: `DocId`, Collection, `first_slot`, `chunk_count`, `class`, `subject_tags: Vec<[u8;16]>`, Seal-Zustand (`Active`). Zusätzlich zwei Indizes:

- `subject_tag → RoaringBitmap<SealOrd>` mit `subject_tag = BLAKE3-keyed(K_ref, SubjectRef)` (16 Byte). Die Betroffenen-ID im Klartext steht nirgends.
- `doc_id → [SealOrd]`.

Chunks eines Seals liegen in **zusammenhängenden Slots**. Das macht `LiveMask`-Änderungen beim Löschen zu Bereichsoperationen.

### 5.5 Ordinale, IDs

`DocId` ist ein zufälliger 128-Bit-Wert, nie aus Inhalt oder Pfad abgeleitet. `chunk_id = BLAKE3(doc_id ‖ rev ‖ chunk_index)`. `SealOrd` wird nur vom Schreib-Task vergeben, monoton, nie wiederverwendet.

---

## 6. RAG-Kernpipeline

### 6.1 Ingest

**REQ-ING-1 (Pipeline).** `Parse → Chunk → Embed (Stapel) → Seal und Store → Index`, jede Stufe mit begrenzter Warteschlange und eigener Parallelität. Rückstau pflanzt sich bis zum Aufrufer als `Backpressure` fort.

**REQ-ING-2 (Formate, Feature `ingest-*`).** Zurückgeholt aus der Historie (13.2): Text, Markdown, HTML, DOCX, PPTX, XLSX/CSV, PDF (Text, kein OCR), E-Mail (EML, MBOX), JSON. Standard-Features der Fassade: `ingest-text`, `ingest-pdf`, `ingest-docx`, `ingest-email`. Weitere nach Bedarf. Unbekannte Formate werden abgewiesen, nie geraten. Alte OLE-Formate (`.doc`, `.ppt`, `.msg`) und ODF sind in M3 nicht enthalten (OPEN-4).

**REQ-ING-3 (Chunking).** Standard: Ziel 512 Token, Mindestgröße 50, Trennung an Abschnitt, Absatz, Satz. Der Abschnittspfad (Breadcrumb) wird **vor** dem Embedding und der BM25-Indexierung dem Chunk-Text vorangestellt (`embed_text = breadcrumb + text`). Das verbessert die Trefferqualität ohne Latenzkosten. Die Tokenzählung ist eine lokale Schätzung ohne Modell. Chunk-IDs sind deterministisch (5.5).

**REQ-ING-4 (Provenienz).** Jeder Chunk trägt `doc_id`, `rev`, `chunk_index`, Zeichenbereich, Seite, Abschnittspfad. Quellpfad und Dateiname liegen in `SealedMeta` (REQ-FMT-2).

**REQ-ING-5 (Fehler).** Ein defektes Dokument bricht den Stapel nicht ab. Der `IngestReport` führt je Dokument `Ok`, `Partial` mit Hinweisen oder `Failed` mit Typ.

**REQ-ING-6 (Betroffene deklarieren).** `SourceDoc.subjects` nimmt die vom Quellsystem bekannten Betroffenen auf (z. B. Mitarbeiter-ID, Kunden-ID). Ohne Deklaration läuft das Dokument durch die Entity-Erkennung. Treffer erzeugen eine Warnung im `IngestReport` (`UndeclaredPersonalData`), nie eine automatische Zuordnung.

**REQ-ING-7 (Unterdrückungsliste).** Der Schreib-Task prüft beim Commit jedes Dokument gegen die Unterdrückungsliste (Abschnitt 7.2). Ein Treffer führt zu `Err(SubjectSuppressed)`.

**REQ-ING-8 (Idempotenz).** Gleiche `external_id` und gleicher Inhaltsdigest erzeugen keine neue Revision. Gleiche `external_id` mit neuem Inhalt erzeugt eine neue Revision mit neuem Seal.

**REQ-ING-9 (Bulk).** `IngestMode::Bulk { sync_every }` synchronisiert alle 64 MiB oder 5 s und am Jobende. Bestätigt wird erst nach dem Sync. Ein Abbruch wiederholt den Job (REQ-ING-8 macht ihn sicher). Löschaufträge sind nie `Bulk`.

### 6.2 Embedding

**REQ-EMB-1 (Port).** `EmbeddingProvider { dim(), model_id(), embed_batch(&[&str]) }`. `model_id` und `dim` gehen in den Index-Kopf.

**REQ-EMB-2 (Backends).** `contextra-embed`: lokales ONNX (`ort`), Modelldatei mit erwartetem SHA-256, Abweichung führt zu `Refused`. `contextra-embed-http`: Aufruf eines konfigurierten internen Dienstes (TLS, Allowlist, Zeitgrenzen). Ohne Backend läuft die Fassade mit vorberechneten Vektoren.

**REQ-EMB-3 (Datenabfluss).** Das HTTP-Backend sendet Chunk-Text nach außen. Das ist Datenabfluss im Sinne des Egress-Guards (REQ-PRV-3). Der Endpunkt muss im Verarbeitungsregister stehen. Für P-Collections ist `allow_personal = true` je Endpunkt Pflicht.

**REQ-EMB-4 (Modellwechsel).** Ein Index gehört zu genau einer `model_id` und `dim`. Anfragen mit anderem Modell werden abgewiesen. Ein Wechsel erfordert Neuberechnung aus `VectorFull` oder `DocBlob`. `drift` vergleicht Stichprobenstatistik und meldet Abweichung.

**REQ-EMB-5 (Reranker, optional).** `Reranker`-Port mit ONNX-Cross-Encoder. Standard aus. Bei Aktivierung bewertet er die besten `rerank_input` (Standard 50) Kandidaten, danach Auswahl der Top-k. Die Latenz wird im Benchmark ausgewiesen.

### 6.3 Vektorindex

**REQ-VEC-1 (HNSW).** Standardparameter `M = 16`, `ef_construction = 200`, `ef_search = 128` (adaptiv erweiterbar), SQ8 ab 20.000 Vektoren. Im Arbeitsspeicher liegen SQ8-Codes und Graph, **nicht** die fp32-Vektoren.

**REQ-VEC-2 (Exakt).** Bis `flat_below = 20.000` Vektoren je Shard sucht der Index exakt mit SIMD-Kosinus. Der exakte Pfad ist auch die Referenz für Rückrufmessungen.

**REQ-VEC-3 (Nachberechnung).** Die besten `rerank_k = 4 × k_cand` Kandidaten der SQ8-Suche werden aus `VectorFull` gelesen, entschlüsselt und in fp32 neu bewertet. Die Datensätze sind je Seal verschlüsselt, der Leseaufwand für rund 100 Kandidaten ist klein gegenüber der Traversierung.

**REQ-VEC-4 (Persistenz).** Auf Platte liegen:
- Graphstruktur (Nachbarlisten, Knoten→Slot) als Basis plus Delta-Log, ohne Vektorinhalt,
- SQ8-Kalibrierung (Wertebereich je Dimension) im Index-Kopf,
- Vektorinhalt nur als `VectorCode`/`VectorFull`-Datensätze je Seal (REQ-FMT-1).

Beim Start liest ein paralleler Lauf `VectorCode` aller `Open`-Seals und baut die Codes im Arbeitsspeicher auf. Zerstörte Seals liefern `KeyAbsent` und werden als tot markiert.

**REQ-VEC-5 (Löschen im Index).** Kein Neuaufbau je Löschung.
1. Sofort: `LiveMask`-Bits der Slots löschen (Abschnitt 6.5). Knoten bleiben für die Navigation, nie in Ergebnissen.
2. Sofort mit der Schlüsselvernichtung: **Routen-erhaltender Scrub.** Der SQ8-Code eines toten Knotens wird durch den Mittelwert der Codes seiner lebenden Nachbarn ersetzt. Der Personenbezug verschwindet aus dem Arbeitsspeicher, die Navigation der Nachbarn bleibt erhalten.
3. Im Scrub-Stapel (7.6): **Konsolidierung.** Tote Nachbarn in Nachbarlisten lebender Knoten werden durch deren lebende Nachbarn ersetzt (Auswahl nach Standardheuristik). Der Graph wird im nächsten Checkpoint ohne tote Knoten geschrieben.
4. Ein vollständiger Neuaufbau erfolgt nur, wenn der Anteil toter Knoten nach Konsolidierung 30 % übersteigt (`tombstone_rebuild_ratio`).

**REQ-VEC-6 (Kein DiskANN, kein RaBitQ).** Entfernt (REM-09). Beide kehren nur über das Gate G-INDEX (12.3) zurück.

### 6.4 Textindex

**REQ-TXT-1.** BM25 (`k1 = 1,2`, `b = 0,75`, konfigurierbar) mit Block-Max-WAND, Segmente höchstens `segment_max_bytes` (256 MiB), Speicherindex für frische Dokumente, Hintergrund-Merge. Deutsch und Englisch: Tokenizer, Stoppwörter, Komposita-Zerlegung. Domänenwörterbücher sind Features.

**REQ-TXT-2 (Löschen im Textindex).** Treffer toter Seals werden im Lesepfad über die `LiveMask` gefiltert (Dokument-Bits). Postings bleiben bis zum Scrub-Merge (7.6) physisch stehen. Statistik: `N` und die Längensumme werden beim Löschen exakt verringert (Längen liegen je Chunk vor). `df(t)` wird **nicht** je Auftrag korrigiert, sondern beim Merge neu berechnet. Begründung: Der Fehler betrifft nur Terme, die fast ausschließlich in gelöschten Dokumenten vorkommen, und diese Dokumente erscheinen ohnehin nie im Ergebnis. Die Abweichung ist auf die Scrub-Frist begrenzt.

**REQ-TXT-3 (Entdeckung).** Für den Trockenlauf (7.3) sucht der Textindex Bezeichner (Name, E-Mail, Personalnummer) als UND über Tokens. Kandidaten werden aus `ChunkText` entschlüsselt und exakt auf Teilzeichenfolge geprüft. Positionsdaten im Index sind nicht erforderlich.

### 6.5 Filter, Zugriffsrechte und LiveMask

**REQ-FLT-1 (Filterfelder).** Je Collection deklariert der Betreiber Filterfelder (`tags`, `dept`, `doc_type`, `valid_from`, `valid_to`, `acl`). Inverse Bitmaps (`Feld=Wert → RoaringBitmap<ChunkSlot>`) liegen im Arbeitsspeicher. Bereichsfelder liegen spaltenweise.

**REQ-FLT-2 (ACL).** Jeder Chunk trägt eine Menge von ACL-Tags (Gruppen, Rollen). Die Anfrage trägt `principals`. Die Filterbitmap ist die Vereinigung der Bitmaps der Principals.

**REQ-FLT-3 (Planer).** Selektivität `s = |Filter| / N`. Ist `|Filter| ≤ brute_force_max` (Standard 20.000): exakte SIMD-Suche über die Filtermenge. Sonst ACORN-artige Traversierung mit Prädikat. Deterministisch gewählt aus Zählwerten, nicht aus Laufzeit.

**REQ-FLT-4 (LiveMask).** Eine dichte Bitmap mit einem Bit je Chunk-Slot, atomar gelesen im Suchpfad. Sie ist **abgeleitet** aus Gate, Version und Registry und wird beim Start neu aufgebaut. Das Gate bleibt die Wahrheit.

**REQ-FLT-5 (Autoritative Endprüfung).** Jedes materialisierte Ergebnis wird einmal gegen `ReadGate::state(seal)` geprüft, bevor Schlüsselmaterial angefasst wird. Das kostet O(1) je Ergebnis. Fällt ein Kandidat dort heraus (die `LiveMask` hinkte hinterher), füllt die Suche in höchstens zwei Runden mit erhöhtem `k_cand` auf. Fehlt der Schlüssel (`KeyAbsent`), gilt das als `Erased`.

### 6.6 Suche und Fusion

**REQ-SRC-1 (Ablauf).**
1. Snapshot pinnen, Gate-Sequenz merken, Filterbitmap bilden (4.6).
2. Vektorsuche (HNSW oder exakt) und Textsuche, jeweils `k_cand = max(2k, 50)`, begrenzt durch `MAX_SEARCH_K`.
3. Endprüfung je Kandidat (REQ-FLT-5), Nachberechnung (REQ-VEC-3).
4. Fusion: gewichtete Reciprocal Rank Fusion, `k = 60`, Gewichte aus Konfiguration. Optional Zeitgewichtung.
5. Optionaler Reranker (REQ-EMB-5).
6. Auswahl der Top-k, Tie-Break aufsteigend nach `DocId`, dann `chunk_id`.
7. Ausgabe mit Provenienz, Signalrängen, optionaler Erklärung.

**REQ-SRC-2 (Determinismus).** Gleiche Anfrage auf gleichem Snapshot und gleichem Gate-Stand liefert identische Trefferlisten. Zeit, Last und Zufall wirken nicht auf das Ergebnis. HNSW-Suche ist je Anfrage deterministisch.

**REQ-SRC-3 (Löschung ohne Sonderpfad).** Das Gate wirkt über Filterbitmap und Endprüfung. Es gibt keinen zusätzlichen Suchdurchlauf.

**REQ-SRC-4 (Frist).** `SearchOptions.deadline` ist optional. Bei Überschreitung liefert die Suche `Err(DeadlineExceeded)`. Es gibt nie ein verkürztes oder anders sortiertes Teilergebnis. Die Prüfung der Frist ist kooperativ und erfolgt an Traversierungsschritten.

**REQ-SRC-5 (Trefferformat).** `chunk_id`, `doc_id`, `rev`, `score`, Provenienz (REQ-ING-4, ohne Quellpfad außer bei Berechtigung), `signals`, optional Erklärung.

### 6.7 Kontext

**REQ-CTX-1.** `assemble(response, TokenBudget) → ContextPack` wählt in Rangfolge bis zum Budget. Kürzung am Satzende. Quellen vollständig.

**REQ-CTX-2.** `ContextPack` nennt verwendete Chunks und ausgelassene mit Grund (`Budget`, `Erased`, `Duplicate`).

**REQ-CTX-3 (Gate beim Zusammenstellen).** `SearchResponse` trägt die Gate-Sequenz. `assemble` prüft jeden Chunk erneut, wenn `gate_seq` inzwischen gewachsen ist. Ein nach der Suche gelöschter Inhalt gelangt nicht in das Kontextpaket.

**REQ-CTX-4 (Duplikate).** Gleicher Textinhalt aus mehreren Revisionen: nur die neueste.

### 6.8 Fassade und Schnittstellen

| Operation | Zweck |
|---|---|
| `Contextra::open(config)` | Startsequenz (8.3) |
| `ingest(collection, SourceDoc, IngestOptions)`, `ingest_batch` | Aufnahme |
| `search(SearchQuery)` | Hybridsuche mit Filter |
| `assemble(SearchResponse, TokenBudget)` | Kontextpaket |
| `get(doc_id, as_of?)` | Abruf, mit Gate |
| `delete_document(doc_id)` | Normales Löschen mit Aufbewahrung (REQ-VER-4) |
| `plan_erasure(ErasureRequest)` | Trockenlauf ohne Nebenwirkung (7.3) |
| `erase(ErasureRequest)` | Löschauftrag (Kapitel 7) |
| `erase_status(TicketId)` | Status nach 7.10 |
| `resolve_review(TicketId, Decision)` | Entscheidung bei Mehrbetroffenen und Einschränkungen |
| `set_hold(scope, reason, until)` | Aufbewahrung, Legal Hold |
| `health()` | Zustand, Gründe, Zähler |

`remember`, `recall`, `forget` und der Query-Rewriter entfallen (REM-24, REM-25). Wer sie braucht, baut einen Adapter über `search` und `ingest`.

**REQ-MCP-1.** Der MCP-Server bietet dieselben Operationen über stdio. Die Tool-Schemas werden aus den Fassadentypen erzeugt. Es gibt keine Tier-Parameter. Fehlermeldungen für nicht eingebaute Backends nennen das Feature, das zu aktivieren ist.

**REQ-SRV-1 (optional, M6).** `contextra-server` bietet HTTP/gRPC mit Authentifizierung (OIDC-Token, Prüfung gegen konfigurierte Schlüssel), bildet Token-Claims auf `principals` und Mandanten ab und exportiert Metriken (Prometheus-Format). Der Server ist ein eigenes Binary-Crate. Der Kern bleibt netzfrei.

### 6.9 Datenschutzfilter am Ausgang

**REQ-PRV-1.** Entity-Erkennung ist regelbasiert (Muster, Prüfziffern, Listen). Sie dient der Warnung bei Ingest und der Ausgabeprüfung. Unsicherheit wird als `Unclassified` markiert und führt nie zur automatischen Freigabe.

**REQ-PRV-2.** Keine LLM-Klassifikation im Kern (REM-20).

**REQ-PRV-3 (Egress-Guard).** Jede Ausgabe über MCP, Server oder Export und jeder Aufruf des HTTP-Embedders läuft durch den Guard. Er arbeitet lokal, ohne E/A und fail-closed.

**REQ-PRV-4 (Verarbeitungsregister).** Das Register (Art. 30) führt Zweck, Kategorien, Empfänger und externe Endpunkte. Es ist Konfigurationsdaten und kein Laufzeitnachweis.

---

## 7. Löschkette v4

### 7.1 Ziele

1. **Sofort unlesbar** nach bestätigtem Auftrag, unabhängig von Bereinigung und Indexaufbau.
2. **Ein Auftrag endet** in `Completed`, `Released`, `Refused` oder in einem benannten, fristgebundenen Rest.
3. **Unauffällig im Betrieb:** Das Gate kostet im Suchpfad ein Bit und eine Endprüfung. Die Bereinigung läuft gebündelt, gedrosselt, unter der Priorität des Suchpfads.
4. **Entkoppelt von der Versionierung** (Kapitel 4).
5. **Keine Selbstbewertung.** Die Kette führt aus und protokolliert (ADR-02).

### 7.2 Umfang eines Auftrags

```rust
pub struct ErasureRequest {
    pub scope: Scope,                  // Subject | Document | Collection | Tenant
    pub reason: Reason,                // Art17 | Withdrawal | Retention | Offboarding | Other
    pub requester: PrincipalRef,
    pub urgency: Urgency,              // Normal | Immediate
    pub multi_subject: Option<MultiSubjectPolicy>,   // überschreibt Mandantenstandard
}
pub enum Scope {
    Subject  { tag_source: SubjectSelector },   // SubjectRef oder Bezeichnerliste (Entdeckung)
    Document { doc: DocId },                    // alle Seals des Dokuments
    Collection { id: CollectionId },
    Tenant,
}
```

| Scope | Wirkung |
|---|---|
| `Subject` | Alle Seals mit passendem Betroffenen-Tag (Registry). Bezeichner führen über die Entdeckung (7.3) zu Kandidaten. |
| `Document` | Alle Seals des `DocId`, aus der Registry, **ohne** den Versionskatalog zu lesen. |
| `Collection` | Alle Seals der Collection. Bei K-Collections Löschen des Verzeichnisses, bei P-Collections Vernichtung der Schlüssel. |
| `Tenant` | **O(1) in der Schlüsselarbeit:** `MandantenKEK` aller Epochen vernichten, danach Verzeichnis entfernen. Ohne die KEK ist jede `DocKey` unbrauchbar. |

Die Unterdrückungsliste (`ErasedSubj`) enthält die Betroffenen-Tags gelöschter Betroffener (16 Byte keyed Hash, keine Klartext-ID). Sie verhindert die Wiederaufnahme beim nächsten Quellsystem-Abgleich (REQ-ING-7) und wächst nur.

### 7.3 Trockenlauf und Entdeckung

**REQ-ER-1 (`plan_erasure`).** Ohne Nebenwirkung. Der Bericht nennt: Anzahl Seals je Klasse, Anzahl Chunks, Anzahl Dokumente mit weiteren Betroffenen (nur Anzahl), Aufbewahrungs- und Hold-Treffer (Anzahl und Grund), **undeklarierte Treffer** aus der Textentdeckung (Anzahl). Nie Kennungen, nie Inhalte. Laufzeit: Registry-Abfrage in Millisekunden, Textentdeckung begrenzt auf `discovery_max_candidates` (Standard 10.000).

**REQ-ER-2 (Zweistufigkeit).** `erase` führt den Trockenlauf intern aus und klassifiziert jeden Seal des Auftrags:

| Klasse | Bedingung | Weiterer Weg |
|---|---|---|
| **Erase** | einziger deklarierter Betroffener ist der Antragsteller, keine Sperre | Gate und Vernichtung |
| **Review** | weitere deklarierte Betroffene (ADR-10) | `Restricted(MultiSubject)` |
| **Hold** | Aufbewahrungspflicht oder Legal Hold aktiv | `Refused` für diesen Seal, Ledger-Eintrag |
| **Undeclared** | Treffer der Textentdeckung ohne Deklaration | Standard `Report` (nur melden). Mandantenoption `Restrict`. |

**REQ-ER-3 (Entscheidung).** `resolve_review(ticket, seals, Decision)` mit `DeleteAll`, `Release` (Einschränkung aufheben) oder `RedactReingest(spans)`. Bei `four_eyes = true` muss der Entscheider vom Antragsteller verschieden sein. Der Ablauf einer Frist erzeugt nur Erinnerungen (`review_remind_after`, 14 Tage), nie eine automatische Löschung.

**REQ-ER-4 (RedactReingest).** Aus den angegebenen Spannen wird eine bereinigte Fassung erzeugt und über den normalen Ingest als **neue Revision mit neuem Seal** geschrieben. Erst nach deren Commit wird der alte Seal vernichtet. Dazwischen ist der alte Seal `Restricted`. Die Versionierung sieht nur eine reguläre neue Revision.

### 7.4 Ablauf

Jeder Schritt ist idempotent. Das Löschlog ist die Wahrheit.

| Schritt | Inhalt | Dauerhaft ab | Zustand danach |
|---|---|---|---|
| **E0 Annahme** | Berechtigung, Ticket, `Requested` ins Löschlog (`fdatasync`). Gleicher Scope → bestehendes Ticket. | Löschlog | `Requested` |
| **E1 Gate** | Läuft im **Schreib-Task** des Mandanten (Prioritätsspur vor Ingest). Scope über die Registry auflösen, Klassifikation (7.3), `Gated`-Eintrag mit Seal-Bitmaps und Unterdrückungs-Tags, `fdatasync`, neuen Gate-Snapshot per `ArcSwap` veröffentlichen, `LiveMask`-Bereiche der Shards löschen lassen. Ticket zurückgeben. | Löschlog | `Gated` |
| **E2 Schlüssel** | Für Klasse **Erase**: `DocKey`-Slots überschreiben (REQ-KEY-4), ein `fdatasync` je Stapel, Schlüsselcache und Zwischenspeicher leeren (`SealScoped`), routen-erhaltender Scrub der SQ8-Codes (REQ-VEC-5), Registry-Eintrag auf `Erased`, Subject-Index bereinigen. | KeyTable und Löschlog | `KeysDestroyed` |
| **E3 Scrub** | Gemeinsame Strukturen gebündelt bereinigen (7.6). | Manifest je Teilschritt | `Scrubbing` |
| **E4 Abschluss** | Ledger-Eintrag, Status. Überschreitet E3 die Frist: Zustand bleibt `Scrubbing` mit Kennzeichen `Overdue`, Alarm, weiterer Versuch. Die Schlüssel sind zu diesem Zeitpunkt bereits vernichtet. | Ledger | `Completed` |

**INV-2 (Sichtbarkeit).** Ab `Gated` liefert kein Lesepfad Klartext des Seals. Ab `KeysDestroyed` gibt es keinen entschlüsselbaren Datensatz mehr (Voraussetzung: Wirkung des Überschreibens, siehe 7.9).

**INV-3 (Sperre bleibt).** Eine durable Sperre wird nie aufgehoben, außer durch `Restricted → Released`. Ein Löschauftrag kehrt nie in `Restricted` oder `Released` zurück.

**REQ-ER-5 (K-Klasse).** Bei K-Collections gibt es keinen Schlüssel. E2 entfällt. E3 ersetzt die betroffenen Segmente. Der Ledger vermerkt `no_key_destruction`.

**Begründung der Reihenfolge (ADR-03).** Alle Nutzdaten (Text, Vektoren, Originale) liegen je Seal verschlüsselt. Die einzigen Strukturen, die Inhaltsspuren ohne diesen Schlüssel tragen, sind abgeleitet (Postings, Graph). Für sie gilt Gate plus Scrub. Der Scrub benötigt keinen `DocKey`, nur die Menge toter Seals. Der frühe Schlüsselschritt verliert also nichts und schützt sofort.

### 7.5 Zustandsautomat

```
Requested → Gated → KeysDestroyed → Scrubbing → Completed
Requested → Restricted{grund, bis}          reversibel (Art. 18, Review)
Restricted → Gated                          Eskalation zur Löschung
Restricted → Released                       terminal
Requested → Refused                         Aufbewahrungspflicht, terminal
jeder nicht-terminale Zustand → Pending{grund} → Fortsetzung
```

Terminal: `Completed`, `Released`, `Refused`. Gründe für `Pending`: `DiskReserveLow`, `IoBackoff`, `ScrubRetry`, `AwaitReview`.

### 7.6 Gebündelter Scrub

**REQ-SCR-1 (Fenster).** Der erste Auftrag startet ein Fenster `scrub_window` (Standard 10 Minuten). Am Ende oder bei `scrub_batch_max_seals` (Standard 10.000) läuft **ein** Stapel über alle bis dahin angefallenen Seals. `Urgency::Immediate` startet den Stapel sofort.

**REQ-SCR-2 (Frist).** `scrub_deadline` (Standard 24 Stunden ab `Gated`) ist die Obergrenze für E3. Mindestwert 15 Minuten.

**REQ-SCR-3 (Inhalt eines Stapels).**
- **Textindex:** Segmente, die tote Seals enthalten, werden ohne deren Postings neu zusammengeführt. Jedes Segment einmal je Stapel.
- **Vektorindex:** Konsolidierung (REQ-VEC-5 Schritt 3), Checkpoint des Graphen ohne tote Knoten, Slots erst nach Checkpoint wiederverwendbar.
- **Registry:** Einträge toter Seals auf Minimalform kürzen (Ordinal, Zustand, Zeitpunkt). Subject-Index bereinigen.
- **Store:** Kompaktion der Bereiche mit toten Seals bei nächster planmäßiger Kompaktion. Keine Pflicht, weil die Datensätze ohne Schlüssel wertlos sind. Wird `compaction_after_erase = true` gesetzt, plant der Scheduler sie in den Stapel ein.
- **Zwischenspeicher:** Ergebnis- und Textcache über `SealScoped` bereits in E2 geleert, im Stapel nochmals geprüft.

**REQ-SCR-4 (Ressourcenregeln).** Der Scrub läuft mit `scrub_threads` (Standard 2) und `scrub_io_limit` (Standard 64 MiB/s, über `Clock` gedrosselt, REQ-TIM-2). Er weicht dem Suchpfad: Steigt die Suchlatenz p95 über das Budget (10.3), pausiert er. Dies ist die einzige lastabhängige Zeitsteuerung und beeinflusst nur den Zeitpunkt der Bereinigung, nie ein Suchergebnis. Die Frist hat Vorrang vor der Pause: Nach Ablauf von `scrub_deadline` arbeitet der Scrub unabhängig von der Latenz weiter.

**REQ-SCR-5 (Fehler).** Ein Fehler führt zu `Pending{ScrubRetry}` mit exponentiellem Backoff, Alarm über `MetricsSink` und Ledger-Eintrag. Ein Fehler im Scrub gefährdet nie bereits erreichte Zustände.

### 7.7 Wiederanlauf

**REQ-REC-1.** Der Start spielt das Löschlog vor dem Öffnen der Lesepfade eines Mandanten ab (C3, 8.3).

| Zustand beim Start | Aktion |
|---|---|
| `Requested` | E1 erneut auswerten |
| `Gated` | ab E2 fortsetzen |
| `KeysDestroyed` | ab E3 fortsetzen |
| `Scrubbing` | Stapel fortsetzen |
| `Pending{…}` | beim nächsten Takt aufnehmen |
| `Restricted` | Sperre erhalten, Review-Erinnerungen planen |

**REQ-REC-2.** E2 ist idempotent. Ein bereits überschriebener Slot gilt als vernichtet. Die Slot-CRC erlaubt, einen halb geschriebenen Slot zu erkennen: Er gilt als `Destroyed`, wenn `state = 2` oder die CRC nicht stimmt und der Journalzustand `KeysDestroyed` erreicht war, andernfalls als `Integrity`-Fehler mit Quarantäne.

**INV-4.** Eine durable Sperre geht nie verloren. Die Zustände `Gated` und `KeysDestroyed` überleben jeden Absturz.

### 7.8 Löschlog und Gate-Snapshot

**Löschlog `CXEL`.** Datensätze `[len u32][seq u64][ticket 16][op u8][payload][crc32c u32]`. `op`: `Requested`, `Gated`, `KeysDestroyed`, `ScrubQueued`, `Scrubbed`, `Restrict`, `Release`, `Refuse`, `Pending`, `Completed`, `Hold`, `Retire`. Das Log hat **keine** Schlüsselkette und keine Signatur (ADR-02). Es erkennt Abbrüche durch CRC und Sequenzlücken. Der Gate-Zustand ist daraus ableitbar.

**Gate-Snapshot.** `gate.snap` enthält `erased: RoaringBitmap<SealOrd>`, `restricted: Map<SealOrd, Grund>`, `suppression: Set<[u8;16]>` und die Sequenz. Er wird bei Schwellwert (Standard 10.000 Löscheinträge) oder wöchentlich atomar ersetzt (`tmp → fsync → rename → fsync(Verzeichnis)`). `erased` und `suppression` wachsen nur.

**REQ-GATE-1 (Veröffentlichung).** Der Schreib-Task schreibt den Gate-Eintrag, führt `fdatasync` aus und tauscht dann den Snapshot per `arc_swap::ArcSwap`. Leser greifen über `load()` zu. Es gibt keine Fence und keine Barriere. Schlägt `fdatasync` fehl, bleibt der Snapshot unverändert, die Antwort ist `Err`, und der Mandant geht in `Quarantine`.

**REQ-GATE-2 (Prüfreihenfolge).** `Erased` vor `Restricted`. `Restricted` liest nur mit Audit-Eintrag, im Standard gar nicht (`restricted_read = Deny`).

### 7.9 Ehrliche Grenzen

Diese Grenzen gelten unabhängig von der Implementierung und stehen im Betriebshandbuch. Sie werden nicht zur Laufzeit gemessen.

- **Überschreiben ist eine Annahme.** Auf Copy-on-Write-Dateisystemen und Flash-Speichern hat das Überschreiben eines Slots keine garantierte physische Wirkung. Die KEK-Rotation (REQ-KEY-5) begrenzt den Zeitraum. Mit K3 (KMS) ist die Zerstörung der KEK im KMS die tragende Maßnahme.
- **Arbeitsspeicher, Auslagerung, Speicherabbilder** können Schlüssel oder Klartext enthalten. `mlock` und `zeroize` mindern das. Auslagerung und Core-Dumps sind zu deaktivieren (Betriebshandbuch).
- **Externe Kopien:** Datensicherungen (7.11), Snapshots des Dateisystems, an externe Dienste ausgeleitete Daten und Ergebnisse, die ein Aufrufer bereits erhalten hat.
- **Nicht deklarierte Mitbetroffene** werden nur über die Textentdeckung gefunden und nie automatisch zugeordnet.
- **Restinformation im Graphen:** Bis zur Konsolidierung bleibt die Nachbarschaftsstruktur eines toten Knotens bestehen (kein Vektorinhalt). Frist: `scrub_deadline`.
- **Rechtliche Einordnung** der Schlüsselvernichtung als Löschung ist offen (OPEN-1).

### 7.10 Status und Ledger

**REQ-LED-1.** `ledger.jsonl` (nur anhängen) protokolliert je Auftrag Zeitstempel der Übergänge, Scope-Art, Zahlen je Klasse (Erase, Review, Hold, Undeclared), Anzahl vernichteter Schlüssel, Scrub-Stapel, Entscheider. Keine Kennungen von Betroffenen, keine Inhalte, keine Dateinamen. Betroffene erscheinen nur als `subject_ref_hash = BLAKE3-keyed(K_ref, SubjectRef)`.

**REQ-LED-2 (`erase_status`).** Liefert `state`, die Zeitstempel je Schritt, die Zahlen, offene Reste mit Frist (`scrub_pending: seals, segments, deadline`) und `overdue`. Der Status beschreibt **ausgeführte Operationen**, nie gemessene Eigenschaften. Er enthält keine Klassen wie „P“ oder „C“ und keine Signatur.

**REQ-DOC-2 (Formulierungen).** In Dokumentation, Oberflächen, Statustexten und Werbung gelten keine Aussagen wie „vollständig gelöscht“, „garantiert“, „unwiederbringlich“, „DSGVO-konform“, „zertifiziert“. Statt dessen: Der Status nennt die ausgeführten Schritte. `tools/doc_lint.py` prüft das in CI.

### 7.11 Datensicherung und Wiederherstellung

**REQ-BAK-1 (zwei Spuren).** Datensicherungen enthalten Chiffrat ohne `keytable.cxkt` und `anchor.cxan`. Schlüsselsicherungen sind eine eigene Spur mit `key_backup_max_age` (Standard 30 Tage) im Kopf. Eine abgelaufene Schlüsselsicherung wird beim Restore verweigert.

**REQ-BAK-2 (Senke, optional).** `erasure.sink` ist ein Port `ErasureSink` (Datei außerhalb von `data_dir`, Objektspeicher mit Aufbewahrungssperre, Syslog). Der Schreib-Task hängt `Gated`- und `KeysDestroyed`-Einträge an die Senke an, **nach** `fdatasync` des Löschlogs und asynchron mit begrenztem Rückstand (`sink_max_lag`, Standard 60 s, bei Überschreitung Alarm).

**REQ-BAK-3 (Restore-Schritt).** `contextra restore --reapply` liest die Senke und vernichtet in der wiederhergestellten KeyTable alle dort als zerstört geführten Seals erneut, schreibt `restore_reapplied` ins Ledger und stößt den Scrub an. Erst danach öffnen Lesepfade.

**REQ-BAK-4 (Erkennung).** Ist die Senke weiter als das lokale Löschlog (höhere Sequenz), erkennt der Start einen Restore und verlangt REQ-BAK-3. Ohne Senke ist ein Restore nicht erkennbar. Das ist eine dokumentierte Grenze, der Betriebsablauf führt `--reapply` aus. Optional `restore_without_sink = Quarantine`.

**REQ-BAK-5 (Aufbewahrungshorizont).** `key_backup_retention ≥ restore_horizon` und die Senke deckt mindestens `restore_horizon` ab. Die Prüfung erfolgt in C0.

### 7.12 Aufbewahrung und Einschränkung

**REQ-RET-1.** Einschränkung (Art. 18, `Restricted`) und Löschung (Art. 17) sind getrennte Pfade. `Restricted` wird vor `Gated` entschieden und ist keine Rücknahme eines Löschauftrags.

**REQ-RET-2.** `set_hold(scope, reason, until)` blockiert E1 für betroffene Seals (`Hold`, 7.3). Gesetzliche Aufbewahrungsfristen können je Collection als Regel hinterlegt sein. Ein Auftrag gegen gehaltene Seals wird für diese Seals `Refused` und im Ledger vermerkt.

---

## 8. Betrieb

### 8.1 Zustände je Shard

| Zustand | Lesen | Schreiben | Löschen | Ursache |
|---|---|---|---|---|
| `Ready` | ja | ja | ja | Normalbetrieb |
| `ReadOnly{grund}` | ja, letzter Snapshot | nein | ja, solange Löschlog und KeyTable schreibbar sind, sonst Annahme und Ausführung nach Rückkehr | `FsyncFailed`, `NoSpace`, `IoError` bei Datenschreibvorgängen |
| `Quarantine` | nein | nein | nein | Löschlog, Gate-Snapshot oder KeyTable nicht konsistent. Fehlende Senke nach erkanntem Restore. |
| `Refused` | nein | nein | nein | Start verweigert (Konfiguration, Format, Schlüssel) |

Der Zustand gilt je **Shard**, nicht je Prozess. Ein Fehler in einem Mandanten stoppt die anderen nicht. Ist das Löschlog eines Mandanten betroffen, gilt `Quarantine` für alle Collections dieses Mandanten.

**INV-5.** Ein Schreib- oder Löschfehler führt nie zu `Quarantine`, außer er betrifft Löschlog, Gate oder KeyTable. Ein fehlgeschlagener `fsync` bei Daten führt zu `ReadOnly`.

**REQ-OPS-1 (Rückkehr).** `ReadOnly → Ready` erfolgt ohne Neustart nach bestandener Strukturprüfung und einem erfolgreichen Probeschreibvorgang. Der Übergang steht im Ledger und im Health-Bericht. Der Zustand `Poisoned` entfällt (REM-13).

**INV-6 (Fail-closed).** Fehlt ein Schlüssel oder ist ein Gate-Zustand unbekannt, liefert der Lesepfad `Erased`, nie Klartext. AEAD-Fehler sind nach außen nicht unterscheidbar.

### 8.2 Fehlerklassen

| Klasse | Beispiel | Reaktion |
|---|---|---|
| `Transient` | I/O-Fehler beim Journalschreiben | Wiederholung mit Backoff, dann `ReadOnly` |
| `Persistent` | `fsync` schlägt wiederholt fehl | `ReadOnly`, Alarm |
| `Integrity` | Prüfsummenfehler in Löschlog, Gate, KeyTable | `Quarantine` des Mandanten |
| `Config` | Ungültige Konfiguration | `Refused` mit vollständiger Fehlerliste |
| `Crypto` | AEAD-Fehler, `KeyAbsent` | einheitlicher Fehler, auf dem Lesepfad `Erased` |
| `Backpressure` | Warteschlange voll | sofortige Abweisung, kein Blockieren |

Jeder Fehler wird mit Typ, Mandant, Zeitpunkt und Zustandsübergang protokolliert, bevor die Antwort erfolgt.

### 8.3 Startsequenz

Je Prozess `C0`, `C1` einmal. Je Mandant `C2`–`C5`. Je Collection `C6`, `C7`. Mandanten und Collections laden **parallel** und **lazy**: Die erste Anfrage öffnet einen Shard, `eager_open` kann das ändern.

| Phase | Inhalt |
|---|---|
| `C0` | Konfiguration vollständig prüfen (Kapitel 9) |
| `C1` | Datenverzeichnis, Sperre, Plattenreserve ≥ `disk_reserve_bytes` |
| `C2` | Wurzel und Anker öffnen, KDF-Parameter prüfen |
| `C3` | **Löschlog und Gate-Snapshot wiedergeben**, KeyTable prüfen. Erst danach sind Lesepfade dieses Mandanten zulässig (REQ-OPS-2) |
| `C4` | Offene Aufträge fortsetzen (7.7) |
| `C5` | Manifest, WAL, Prüfsummen der Store-Dateien |
| `C6` | Index laden: Graph, `VectorCode` je offenem Seal (parallel), Textsegmente. `LiveMask` aus Gate, Version und Registry aufbauen |
| `C7` | Zustand `Ready` oder `ReadOnly`, Lesepfade freigeben |

**REQ-OPS-2.** Vor `C3` liefert jede Anfrage `Erased` oder einen typisierten Fehler.

### 8.4 Nebenläufigkeit

**REQ-CON-1 (Schreib-Task).** Jeder Shard hat genau einen Schreib-Task. Er serialisiert Ingest-Stapel, Gate-Änderungen und Versionsänderungen. Löschaufträge laufen auf einer Prioritätsspur: Der Task prüft sie zwischen Ingest-Stapeln. Ein Ingest-Stapel ist auf 4 MiB oder 5 ms Arbeitszeit begrenzt. Damit wartet ein Löschauftrag höchstens einen Stapel.

**REQ-CON-2 (Snapshots).** Leser arbeiten auf unveränderlichen Snapshots (`ArcSwap`). Der Schreiber veröffentlicht erst nach der Dauerhaftigkeit (Gruppen-Commit).

**REQ-CON-3 (Gruppen-Commit).** Ingest-Schreibvorgänge teilen sich einen `fdatasync` innerhalb von `group_commit_window` (Standard 2 ms oder 4 MiB). Löschaufträge nutzen ein eigenes Fenster von 0–1 ms.

**REQ-CON-4 (Einzelausführung).** Pro Scope höchstens eine Saga. Ein Duplikat liefert das bestehende Ticket. Gleichzeitiger Ingest und Löschung desselben Dokuments: Die Serialisierung im Schreib-Task entscheidet, Ingest nach `Gated` erhält `SubjectSuppressed` oder legt einen neuen, unberührten Seal an (Dokument-Scope).

**REQ-CON-6 (Schreibstapel).** Ein Schreibstapel (ein oder mehrere Dokumente) wird atomar veröffentlicht: Store, Seal-Registry, Index und Versionskatalog erscheinen für Leser gemeinsam mit einem Snapshot-Tausch nach dem Gruppen-Commit. Ein Absturz vor dem Commit lässt keinen sichtbaren Teilzustand zurück.

**REQ-CON-5 (Rechenpool).** Eine gemeinsame Rechenpool-Instanz (`compute_threads`, Standard Anzahl Kerne − 2) bedient Embedding-Nachbearbeitung, Indexaufbau, Merge und Scrub. Suchen laufen auf Aufruferthreads oder einem eigenen Suchpool (`search_threads`).

### 8.5 Ressourcen

**REQ-RES-1 (Speicherbudget).** `memory_budget_bytes` begrenzt Memtable, Blockcache, Textspeicherindex, Zwischenspeicher. SQ8-Codes und Graph zählen zum Index und werden bei Start gegen das Budget geprüft (`Refused` mit Rechnung, wenn sie nicht passen). Überschreitung löst Flush oder Evict aus, nie Abbruch.

**REQ-RES-2 (Shard-Verdrängung).** Ungenutzte Shards werden nach `shard_idle_evict` (Standard 30 Minuten) aus dem Arbeitsspeicher entladen, sofern das Budget das erfordert, nie ein Shard mit laufendem Auftrag.

**REQ-RES-3 (Warteschlangen).** Alle Warteschlangen sind begrenzt. Bei Erreichen: `Backpressure`.

**REQ-RES-4 (Limits).** Trefferzahl, Dokumentgröße, Chunks je Dokument, Stapelgröße stehen einmal in `contextra-ids` und sind über Konfiguration veränderbar.

**REQ-RES-5 (Löschlast).** Siehe 7.6. Der Löschpfad darf die Suchlatenz im Normalfall nicht messbar verändern (Budget 10.3).

### 8.6 Zeit und Determinismus

**REQ-TIM-1.** Alle Zeitquellen laufen über `Clock`. Direkte Aufrufe von `Instant::now` und `SystemTime::now` sind außerhalb der `Clock`-Implementierungen verboten.

**REQ-TIM-2.** Drosselung nutzt `Clock` und ist mit `ManualClock` testbar.

**REQ-TIM-3.** Zeitgrenzen im Produkt: `deadline` der Suche (kooperativ, Fehler statt Teilergebnis), Scrub-Fenster und -Frist, HTTP-Zeitgrenzen in Adapter-Crates. Es gibt keine Zeitbudgets, die Ranking oder Kandidatenmenge verändern.

**INV-7 (Determinismus).** Siehe REQ-SRC-2.

### 8.7 Betrieb über Prozessgrenzen

- Ein Prozess hält die Sperre auf `data_dir`. Mehrere Prozesse teilen sich ein Verzeichnis nie.
- Skalierung geschieht über Mandanten-Aufteilung auf mehrere Prozesse durch die Anwendung. Hochverfügbarkeit im Kern ist ein Nicht-Ziel (OPEN-3). Der Weg dorthin ist die Replikation von Löschlog und KeyTable-Änderungen, bevor Datenreplikation sinnvoll ist.
- Metriken: `MetricsSink` schreibt lokal. Export (Prometheus, OpenTelemetry) liegt in `contextra-server`.

---

## 9. Konfiguration

Die Konfiguration wird vollständig in `C0` geprüft. Alle Fehler werden gesammelt gemeldet (`Refused`).

```rust
pub struct ContextraConfig {
    pub data_dir: PathBuf,
    pub root: RootConfig,                  // Passphrase | KeyFile(path) | Kms(...)
    pub embedding: Option<EmbeddingConfig>,// Backend, Modelldatei, SHA-256, dim
    pub index: IndexConfig,                // m, ef_construction, ef_search, flat_below, sq8, tombstone_rebuild_ratio
    pub text: TextConfig,                  // k1, b, Wörterbücher
    pub ingest: IngestConfig,              // chunk_target, min_tokens, Formate
    pub filter: FilterConfig,              // Felder je Collection, brute_force_max
    pub versions: VersionConfig,           // je Collection: retention, index_history, on_delete
    pub resources: ResourceConfig,         // memory_budget_bytes, compute_threads, search_threads,
                                           // group_commit_window, key_cache_entries, shard_idle_evict
    pub erasure: ErasureConfig,
}

pub struct ErasureConfig {
    pub scrub_window: Duration,            // 10 min
    pub scrub_batch_max_seals: u32,        // 10_000
    pub scrub_deadline: Duration,          // 24 h, Minimum 15 min
    pub scrub_threads: u16,                // 2
    pub scrub_io_limit: u64,               // 64 MiB/s
    pub compaction_after_erase: bool,      // false
    pub epoch_len: Duration,               // 30 Tage
    pub multi_subject: MultiSubjectPolicy, // Restrict (Standard) | DeleteAll | RedactReingest
    pub undeclared_hits: UndeclaredPolicy, // Report (Standard) | Restrict
    pub four_eyes: bool,                   // false
    pub restricted_read: RestrictedRead,   // Deny (Standard) | Audit
    pub review_remind_after: Duration,     // 14 Tage
    pub discovery_max_candidates: u32,     // 10_000
    pub sink: Option<SinkConfig>,          // Senke (7.11)
    pub sink_max_lag: Duration,            // 60 s
    pub key_backup_max_age: Duration,      // 30 Tage
    pub key_backup_retention: Duration,    // 400 Tage
    pub restore_horizon: Duration,         // 30 Tage
    pub disk_reserve_bytes: u64,           // ≥ 1 GiB
}
```

**Prüfungen in `C0`:**

- `data_dir` vorhanden, beschreibbar, nicht auf Netzwerkdateisystem ohne ausdrückliche Option.
- Passphrase nicht leer, KDF m ≥ 64 MiB, t ≥ 3. `KeyFile` mit Rechten 0600.
- `scrub_deadline ≥ 15 min`. `key_backup_retention ≥ restore_horizon`.
- `embedding.model_sha256` stimmt mit der Datei überein.
- Verschlüsselung für P-Collections ist nicht abschaltbar. Eine Konfiguration `encryption = none` wird abgewiesen.
- `memory_budget_bytes` ≤ verfügbarer Speicher minus Reserve.
- `index_history = true` nur mit ausdrücklicher Bestätigung der Kosten.

---

## 10. Leistungsziele

Startwerte. **INV-8:** Ein Wert gilt erst als erreicht, wenn der Benchmark ihn misst und `benchmarks/budgets.toml` ihn enthält. Messhardware laut 1.2.

### 10.1 Suche

| Größe | S (1 Mio.) | L (10 Mio.) | Bedingung |
|---|---|---|---|
| Hybridsuche p50 | ≤ 15 ms | ≤ 30 ms | k = 10, ohne Filter, ohne Reranker |
| Hybridsuche p95 | ≤ 40 ms | ≤ 80 ms | gleiche Bedingung, Last bei 50 % der Spitze |
| Mit Filter (Selektivität 10 %) p95 | ≤ 1,5 × ungefiltert | ≤ 1,5 × ungefiltert | ACORN-Pfad |
| Rückruf@10 gegen exakt | ≥ 0,95 | ≥ 0,95 | Referenzdatensatz |
| Durchsatz | ≥ 200 QPS | ≥ 100 QPS | 16 Kerne |
| RAM je Chunk | ≤ 1,5 KB | ≤ 1,5 KB | 768 Dimensionen, ohne Blockcache |
| Kaltstart bis `Ready` | ≤ 10 s | ≤ 60 s | lazy Shards, Index-Laden parallel |

### 10.2 Ingest

| Größe | Ziel | Bedingung |
|---|---|---|
| Store und Index | ≥ 2.000 Chunks/s je Shard | ohne Embedding |
| Chunker | ≥ 5 MB Text/s je Kern | |
| Bulk-Modus | ≥ 1,5 × normaler Ingest | ADR-18 |

### 10.3 Löschung und Kosten der Löschung

| Größe | Ziel |
|---|---|
| Auftrag bis Lesesperre (E1), bis 1.000 Seals | ≤ 5 ms p99 |
| Auftrag bis Schlüsselvernichtung (E2), bis 1.000 Seals | ≤ 1 s p99 |
| Mandanten-Offboarding (Schlüsselarbeit) | ≤ 100 ms |
| Scrub-Abschluss | ≤ `scrub_deadline` |
| **Suchlatenz p95 mit Seal- und Gate-Schicht** gegenüber Basislinie ohne Schicht | **≤ +3 %** |
| **Ingest-Durchsatz** mit Seal-Schicht gegenüber Basislinie | **≥ 95 %** |
| **Suchlatenz p95 während Scrub** gegenüber ruhig | **≤ +5 %** |
| RAM-Aufschlag der Schicht (`seal_of` 4 B + `LiveMask` 1 Bit je Chunk) | ≤ 1 % |

Die **Basislinie** ist derselbe Build mit der Konfiguration `bench_baseline = true` (nur im Bench-Crate, Seal-Schicht als Durchleitung). Sie existiert nur zur Messung der Mehrkosten und hat keinen Produktionspfad.

### 10.4 Messmethodik

Jeder Wert hat Datensatz (`benchmarks/data/de_mini` plus synthetisch für S und L), Hardware, Warmlauf, Wiederholungen und Konfidenzangabe. Die Messung ist ein CI-Lauf (nächtlich), kein Laufzeitmerkmal des Produkts.

---

## 11. Tests und Gates

Die Prüfung der Löschung liegt in Testcode. Ein Test darf das Datenverzeichnis nach bekannten Klartextmustern durchsuchen. Diese Fähigkeit existiert nur in `contextra-testkit` und nie im Produkt.

### 11.1 Löschtests

| ID | Prüft |
|---|---|
| T-ER-01 | Nach Ingest und Flush: kein Klartext-Canary der P-Collection in den Datenverzeichnisdateien (Testwerkzeug) |
| T-ER-02 | Nach E1: Suche, `get`, `assemble` liefern den Inhalt nicht. Auch vor E2 |
| T-ER-03 | Nach E2: `open` liefert `KeyAbsent`. SQ8-Code des toten Knotens enthält den ursprünglichen Code nicht |
| T-ER-04 | Neustart nach jedem Zustand: Gate bleibt, Aufträge setzen fort (Absturzmatrix an jedem `fdatasync`-Punkt, mit `FaultVfs`) |
| T-ER-05 | Halb geschriebener Slot: korrekte Behandlung nach REQ-REC-2 |
| T-ER-06 | Zwei gleichzeitige Aufträge, gleicher Scope: ein Ticket |
| T-ER-07 | Gleichzeitiger Ingest und Erase desselben Betroffenen: serialisiert, kein Seal bleibt unerfasst (Loom oder Schedule-Exploration) |
| T-ER-08 | Jeder Lesepfad prüft das Gate: Suche (Vektor, Text), `get`, Zeitreise, Ergebniscache, Export, `assemble`, Ingest-Duplikat |
| T-ER-09 | Jeder Cache implementiert `SealScoped` und wird bei E2 geleert (Registrierungstest über alle Cache-Typen) |
| T-ER-10 | Zeitreise auf gelöschten Seal: `NotFound` ohne Sonderregel (REQ-VER-6) |
| T-ER-11 | **Entkopplung:** Löschung ändert den Versionskatalog nicht. Versions-GC ohne `erasure`-Crate kompiliert und läuft. `erasure` ohne `version`-Crate kompiliert und läuft |
| T-ER-12 | `ContentRetired` vernichtet den Seal mit Grund `Retention` |
| T-ER-13 | Mehrbetroffene: Standard `Restrict`, keine automatische Löschung. `RedactReingest` legt neuen Seal an, alter wird erst danach vernichtet |
| T-ER-14 | Unterdrückungsliste verhindert Wiederaufnahme (`SubjectSuppressed`) |
| T-ER-15 | `Hold` blockiert E1 für betroffene Seals |
| T-ER-16 | Mandanten-Scope: nach Vernichtung der `MandantenKEK` ist keine `DocKey` mehr entwrappbar |
| T-ER-17 | Epochenwechsel: gemischte Epochen lesbar, alte KEK erst nach vollständigem Rewrap überschrieben, Absturz mitten im Rewrap |
| T-ER-18 | Scrub-Stapel: jedes betroffene Segment einmal neu geschrieben. Frist wird eingehalten (mit `ManualClock`) |
| T-ER-19 | Scrub pausiert bei Latenzüberschreitung, läuft nach Fristablauf trotzdem |
| T-ER-20 | Konsolidierung: Rückruf des Graphen nach Löschung von 10 % der Knoten ≥ Ziel |
| T-ER-21 | Restore mit alter KeyTable und Senke: Seals werden erneut vernichtet. Ohne Senke und mit `restore_without_sink = Quarantine`: `Quarantine` |
| T-ER-22 | K-Klasse: kein Schlüsselschritt, Segmentersatz, Ledger vermerkt `no_key_destruction` |
| T-ER-23 | Ledger enthält keine Kennungen, Namen oder Inhalte (Property-Test mit Canary-Namen) |
| T-ER-24 | `erase_status` enthält keine Wörter der verbotenen Liste (7.10, `doc_lint`) |

### 11.2 Betriebstests

| ID | Prüft |
|---|---|
| T-OPS-01 | `fsync`-Fehler bei Daten: `ReadOnly`, Suche liefert weiter. Rückkehr nach Probeschreibvorgang |
| T-OPS-02 | Fehler im Löschlog oder KeyTable: `Quarantine` nur für den Mandanten |
| T-OPS-03 | Ungültige Konfiguration: `Refused` mit vollständiger Fehlerliste |
| T-OPS-04 | Speicherbudget erreicht: Flush, kein Abbruch |
| T-OPS-05 | Warteschlange voll: `Backpressure`, kein Blockieren |
| T-OPS-06 | Gleiche Anfrage unter verschiedener Last: identische Trefferliste |
| T-OPS-07 | Netzfreier Lauf des Kernbaums (`unshare -n`) |
| T-OPS-08 | Löschauftrag wartet höchstens einen Ingest-Stapel (REQ-CON-1) |

### 11.3 RAG-Tests

| ID | Prüft |
|---|---|
| T-RAG-01 | Chunker: Offsets zeigen auf den Quelltext, Golden-Dateien |
| T-RAG-02 | Parser: Beispiele je Format, defekte Dateien brechen den Stapel nicht ab, Fuzz-Ziele (aus Historie) |
| T-RAG-03 | Hybridsuche: Rückruf gegen Referenzdatensatz (`de_mini`) |
| T-RAG-04 | HNSW gegen exakt: Rückruf@10 ≥ Ziel, mit und ohne Filter |
| T-RAG-05 | Filterplaner wählt Pfad nach Zählwerten, Ergebnis gleich bei beiden Pfaden |
| T-RAG-06 | ACL: Ergebnis enthält nie Chunks ohne passenden Tag (Property-Test) |
| T-RAG-07 | Kontextbudget: Tokensumme ≤ Budget, Quellen vollständig |
| T-RAG-08 | Modellwechsel wird abgewiesen |
| T-RAG-09 | Breadcrumb-Vorsatz verbessert Rückruf auf dem Kontextpräfix-Szenario aus `benchmarks` |
| T-RAG-10 | Idempotenter Ingest (REQ-ING-8), Wiederholung eines abgebrochenen Bulk-Jobs |

### 11.4 Leistungstests

`T-PERF-n` je Zeile aus 10.1 bis 10.3. Nächtlich, Ergebnis nach `benchmarks/results/`. Ein Lauf über dem Budget schlägt fehl, sobald das Budget in `budgets.toml` steht.

### 11.5 Build-Gates

| ID | Gate |
|---|---|
| G-01 | `cargo check --workspace` und `cargo test --workspace` grün |
| G-02 | `unsafe_code = "forbid"` im Workspace, Ausnahmen nur `sys` und `simd` (`tests/unsafe_islands.rs`) |
| G-03 | Schichtprüfung: R1, R2, R7 (`crates/contextra-testkit/tests/layering.rs`, liest `cargo metadata`) |
| G-04 | `MAX_SEARCH_K` und alle Limits genau einmal definiert |
| G-05 | `cargo tree -e normal` der Kern-Crates enthält keine Netzwerk-Crates (Sperrliste: `reqwest`, `hyper`, `ureq`, `tonic`, `h2`, `native-tls`, `openssl`, `rustls`) |
| G-06 | `default = []` in jedem Crate |
| G-07 | Keine Erwähnung entfernter Crates im Produktcode |
| G-08 | Dokumentationslint (REQ-DOC-2) |
| G-09 | Keine Dateien im Repository, die nur Selbstdeklarationen enthalten (REM-14) |
| G-10 | `benchmarks/budgets.toml` vorhanden, Verweise auf fehlende Dateien sind Fehler |

---

## 12. Entfernungen

Je Entfernung ein Commit. Nach jedem Commit `cargo check --workspace`. Keine Verhaltensänderung im selben Commit. Die Fundstellen sind bei Umsetzung per `grep` neu zu erheben, Zeilenzahlen sind Richtwerte.

### 12.1 Robotik, Profile, Lizenz

| ID | Entfernen | Ort | Abnahme |
|---|---|---|---|
| REM-01 | Crate `contextra-vault` | `crates/contextra-vault/`, Workspace-`exclude` | Verzeichnis fehlt |
| REM-02 | Profile `BareMetal`, `EdgeMinimal`, `PowerUserLocal`, Modi `MemoryOnly`, `WalNoHmac`, Löschmodus `TombstoneOnly` | `contextra/src/performance_profile.rs`, `collection_profile.rs`, `contextra-store/src/lsm/ops/write/`, `contextra-crypto/src/kv_segment/delete_mode.rs`, MCP-Schema | `grep -rn "BareMetal\|MemoryOnly\|WalNoHmac\|TombstoneOnly\|EdgeMinimal"` ohne Treffer |
| REM-03 | Lizenzreste: `LicenseGate`, `FeatureRing`, `VectorDeleteMode` | `contextra-ports/src/license.rs`, `builder.rs` | `grep -rn "License\|FeatureRing\|BackgroundRepair"` ohne Treffer |
| REM-04 | Hardware-Sensorik und Anti-Tamper | `contextra-crypto/src/anti_tamper.rs` | Datei fehlt |
| REM-05 | Observer-Subsystem mit Latenzfrist und Circuit-Breaker | `contextra-store/src/lsm/observer/`, `observer.rs` | Verzeichnis fehlt |
| REM-06 | Zeitbudget im Ranking | `contextra-rank/src/dibud/` | Verzeichnis fehlt |
| REM-07 | `thread::sleep`-Drosselung ohne `Clock` | `contextra-durable-fs/src/scrub.rs` | nur `Clock`-Drosselung |
| REM-08 | Kostenbasierte Kompaktionsplanung | `contextra-store/src/compaction/adaptive/` | einfache Größenschwelle |

### 12.2 Eigennachweise und Altlöschung

| ID | Entfernen | Ort | Abnahme |
|---|---|---|---|
| REM-09 | DiskANN, RaBitQ, `partial_rebuild` (ersetzt durch Konsolidierung), `purge.rs` | `contextra-vector/src/diskann/`, `quantize_rabitq.rs`, `partial_rebuild.rs`, `purge.rs` | Dateien fehlen. Feature `experimental` entfernt |
| REM-10 | Altes Erasure: `vault/`, `types_local.rs`, `crypto_local.rs`, Saga 2.0, `Armed`, `ControlEvidence`, `MeasureClass`, `KeyCoverage`, `LayerWitness`, `CommitBarrier`, `BarrierExclusive` | `contextra-erasure/src/` | `pub mod vault` und die Typen fehlen. Crate neu aufgebaut nach Kapitel 7 |
| REM-11 | Messen, Signieren, Berichten: `audit_chain.rs`, `wal_completeness.rs`, `wal_crypto.rs`, `kv_segment/`, `kv_cipher.rs` | `contextra-crypto/src/` | Dateien fehlen. Kein `Ed25519`, kein JCS im Workspace |
| REM-12 | SSI, TxBuffer, ReadSet | `contextra-mvcc/src/ssi.rs`, `tx_buffer.rs` | Dateien fehlen |
| REM-13 | `Poisoned`, Fault-Injection im Produktcode | `contextra-durable-fs/src/health.rs`, `fault_injector.rs`, `contextra-store/src/io_seam*` | Fault-Injection nur in `contextra-testkit` |
| REM-14 | Selbstdeklarationen: `contract.rs` je Crate, `ANCHOR`/`LINK`/`FILE-CONTEXT`/`STAND`-Kommentare, Auditor-Skripte (`port_contract_auditor`, `contract_diff_detector`, `type_registry_sync`, `isolation_proof`, `erasure_leak_prober`, `fsync_durability_tracer` u. ä.) | `crates/*/src/contract.rs`, `tools/` | `grep -rn "ANCHOR\[\|FILE-CONTEXT"` ohne Treffer. `tools/` enthält nur die Skripte aus 11.5 |
| REM-15 | Alle `compat/`-Module (rund 5.800 Zeilen) | `vector/compat`, `text/compat`, `rank/compat`, `checkpoint/compat`, `mvcc/compat`, `simd/compat` | keine `compat/`-Verzeichnisse |
| REM-16 | Rest des Altmodells: Berichte, Bescheinigung, Satzbibliothek, Annahmen- und Residuenkataloge | `contextra-erasure`, Dokumente | `grep -rn "Bescheinigung\|Satzbibliothek\|A-FLASH\|R-GROUP-LINK"` ohne Treffer außerhalb von 7.9 |

### 12.3 Funktionsumfang

| ID | Entfernen | Ort | Abnahme |
|---|---|---|---|
| REM-18 | Kalibrierung, Resonanz-, Ext-, Global-, Normalized-Fusion | `contextra-rank/src/calibration/`, `fusion/{resonance,ext,global,normalized}.rs` | Dateien fehlen |
| REM-19 | `contextra-engine` | `crates/contextra-engine/` | Verzeichnis fehlt |
| REM-20 | LLM-Klassifikation, Egress-Vault, Egress-Gateway, Bulk-Exfiltration, Kontext-Edit-Audit | `contextra-privacy/src/{egress_vault,egress_gateway,bulk_exfiltration_detector,context_edit_audit}.rs` | Dateien fehlen. Verbleibend: `entity_recognizer`, `data_class`, `egress_guard`, `processing_registry`, `guarded_payload` |
| REM-21 | MCP-Sandbox, Prompt-Injection-Stub, Reexport-Modul | `contextra-mcp/src/{sandbox,prompt_injection,bulk_exfiltration_detector}.rs` | Dateien fehlen |
| REM-22 | Ollama-/Candle-/ONNX-Konfiguration ohne Implementierung | `contextra-mcp/src/config.rs` | ersetzt durch 9 und REQ-EMB-2 |
| REM-23 | Duplizierte Distanzkerne in `vector/distance` (nur noch Dispatch auf `contextra-simd`), Abhängigkeit `simd → types` | `contextra-vector/src/distance/`, `contextra-simd/Cargo.toml` | ein Kernel-Satz |
| REM-24 | Agentengedächtnis: `agent_memory.rs`, Zettelkasten-Typen, `WorkflowState`, `MemoryType`, Zerfallskonstanten | `contextra/src/agent_memory.rs`, `contextra-types/src/types/domain/misc.rs` | Dateien und Typen fehlen |
| REM-25 | Query-Rewriter (LLM) | `contextra/src/query_rewriter.rs`, `ports/query_rewriter.rs` | Dateien fehlen |
| REM-26 | Tenant-Präfix-Codec im Store (ersetzt durch Verzeichnis je Mandant) | `contextra-store/src/tenant_codec*` | Dateien fehlen |
| REM-27 | Abhängigkeit `ports → types`, `store → crypto`, `store → mvcc`, Dev-Abhängigkeiten auf `testkit` in Produktcrates | `Cargo.toml` der Crates | G-03 grün |
| REM-28 | Duplizierte Kryptoabhängigkeiten (`sha2`, `hmac` in `erasure`, `engine`, `types`) | `Cargo.toml` | nur `contextra-crypto` hat Kryptoabhängigkeiten |
| REM-29 | Verweise auf entfernte Dokumente und Crates | `CLAUDE.md`, `AGENTS.md`, Crate-READMEs, `docs/crates/*.md` | G-07 grün |


**Gate G-INDEX (Rückkehr von DiskANN oder RaBitQ).** Nur mit einem Bestand, den HNSW im RAM-Budget nicht hält (> 50 Mio. Chunks je Prozess), und einem Benchmark mit Rückruf und Latenz.

**Gate G-STORE (Store).** Die eigene LSM bleibt, wenn sie in M1 auf Messhardware erreicht: Punktzugriff p99 ≤ 1 ms (warm), ≥ 50.000 Schreibvorgänge/s im Gruppen-Commit, Wiederanlauf nach Absturz in ≤ 10 s bei 10 GB. Sonst wird sie hinter dem Port `KvStore` durch `redb` oder `fjall` ersetzt. Die Entscheidung fällt anhand der Messung, nicht der Aufwandsschätzung.

---
### contextra-graph
Kein Enterprise-Kunde (CISO, Legal Counsel, Head of AI) kauft ein System wegen _TL-HFD mit Lovász-Erweiterung_. Was sich im Markt für regulierte KI-Systeme hingegen **extrem gut als Alleinstellungsmerkmal** verkaufen lässt, ist die Kombination aus:

  

1. **Auditierbares PathRAG + Provenance:** Nicht nur „Wir haben 5 Chunks gefunden“, sondern _„Hier ist der exakte, deterministisch reproduzierbare Begründungspfad über 3 Paragrafen hinweg“_.
    
2. **Spurlose Graph-Erasure (`ConsistencyEnforcer` + `contextra-erasure`):** Bei fast allen GraphRAG-Systemen bleiben nach dem Löschen eines Dokuments „Ghost-Nodes“ oder verwaiste Relationen im Graphen zurück. Garantieren zu können, dass ein gelöschter Mandant/Patient sowohl aus den Vektoren als auch aus der Graph-Topologie kryptografisch und strukturell verschwindet, ist ein echter Enterprise-USP.
    
3. **Time-Travel & Branching Agent Memory (`SessionBranchTree` + MVCC-Snapshots):** Nachweisen zu können, auf welchem exakten Wissens- und Graph-Zustand ein KI-Agent am Dienstag um 14:00 Uhr eine Entscheidung getroffen hat (und Sackgassen sauber verwerfen zu können), löst das größte Governance-Problem von KI-Agenten in Unternehmen.
    

## 3. Konkreter Schnittplan für `contextra-graph`

|**Komponente**|**Aktion**|**Begründung**|
|---|---|---|
|`tl_hfd` + `lovasz.rs`|**Komplett löschen**|Reines arXiv-Overengineering, hohe Komplexität, kein kundenseitiger Mehrwert.|
|`apprh` (`GateMonitor`, `Selector`, `ShadowComparison`)|**Shadow- & Gate-Logik löschen**|Entweder ihr nutzt _nur_ `ForwardPush` PPR auf einem normalen `CsrGraph` (Empfehlung) oder _nur_ Hypergraph-Push – aber ohne Shadow-A/B-Testing im Hot-Path.|
|`percolation` (`find_rebonding_candidates`)|**Komplett löschen**|Fachlich und datenschutzrechtlich riskant (konstruiert neue Zusammenhänge nach Löschungen).|
|`Dense PPR`|**Löschen**|Durch `ForwardPush` PPR vollständig abgedeckt und schneller.|
|`CsrGraph` + `ForwardPush` PPR|**Behalten**|Schnelles, deterministisches, speichereffizientes Fundament.|
|`PathRAG`|**Behalten**|Direkter Qualitäts- und Erklärbarkeitsgewinn für das LLM.|
|`ConsistencyEnforcer`|**Behalten**|Zwingend nötig für saubere DSGVO-Löschungen im Graphen.|
|`SessionBranchTree`|**Behalten**|Starker USP für auditable Agent-Workflows.|
|`EdgeReinforcementBuffer`|**Optional halten (Feature-Flag)**|Nur aktivieren, wenn der Mandant explizit adaptives Agent-Memory statt deterministischem Dokumenten-Retrieval nutzt.|


---

## 13. Umsetzung

### 13.1 Meilensteine

Aufwand grob als S (≤ 1 Woche), M (2–3 Wochen), L (4–6 Wochen), XL (über 6 Wochen), jeweils für ein Zweierteam, Schätzung ±50 %.

| M | Inhalt | Aufwand | Abnahme |
|---|---|---|---|
| **M0 Basis** | Alte Dokumente archivieren. Build reparieren (Fassade kompiliert). `MAX_SEARCH_K` vereinheitlichen. Workspace-Lint `forbid`. `budgets.toml` anlegen. Schichtprüfung G-03 | S | G-01, G-02, G-04, G-10 |
| **M1 Bereinigung** | REM-01 bis REM-29 in der Reihenfolge der Abschnitte 12.1 bis 12.3. Benchmark für G-STORE | L | Alle `grep`-Abnahmen. Umfang rund −50 %. G-STORE entschieden |
| **M2 Seal und Löschkette** | `contextra-crypto` (KeyTable, Wurzel), `contextra-erasure` (Löschlog, Gate, Saga, Scrub-Scheduler), `contextra-version`, Seal-Registry, Verdrahtung in `contextra-db`. Absturzmatrix ab dem ersten schreibenden Schritt | XL | T-ER-01 bis T-ER-24 grün |
| **M3 Ingest und Embedding** | `contextra-ingest` aus Historie (13.2), Chunker, Formate, `contextra-embed` (ONNX), Pipeline mit Backpressure, Bulk-Modus | L | T-RAG-01, 02, 10 grün |
| **M4 Suche** | HNSW-Standard mit SQ8 und Nachberechnung, Persistenz ohne Vektorinhalt, Filter und ACL, Planer, Fusion, Reranker-Port, Kontext, Fassade, MCP | L | T-RAG-03 bis 09, T-ER-08 grün |
| **M5 Betrieb und Leistung** | Zustände, Startsequenz mit Lazy-Laden, Ressourcenregeln, Drosselung, `embed-http`, Benchmarks gegen Basislinie, `budgets.toml` befüllt | M | T-OPS-01 bis 08, T-PERF grün |
| **M6 Enterprise** | `contextra-kms`, `contextra-server` (Auth, Metriken), Betriebshandbuch mit Grenzen (7.9), Restore-Ablauf | L | Ende-zu-Ende-Lauf: Ingest, Suche, Löschung, Restore. Offene Punkte aus Kapitel 14 entschieden |

Reihenfolge M2 vor M3 und M4: Der Seal-Schreibpfad ist Voraussetzung für beide. M3 und M4 sind danach parallel möglich.

### 13.2 Wiederverwendung aus der Git-Historie

Das Repository enthält die vollständige Historie (5.952 Commits). Die folgenden Bestandteile sind dort vorhanden und werden **zurückgeholt, dann beschnitten und an die neuen Ports angepasst**, nicht neu geschrieben:

| Bestandteil | Fundstelle | Umfang | Maßnahme |
|---|---|---|---|
| Ingest-Crate (Formate, Chunking, Erkennung, Fuzz) | `e5ec76c28^:crates/contextra-ingest/` | rund 21.000 Zeilen, 155 Quelldateien | Wiederherstellen als `contextra-ingest`. Entfernen: OLE/ODF/asciidoc (OPEN-4), `compat/`, `canonical.rs` prüfen. Features beibehalten. Fuzz-Ziele behalten |
| Markdown-Chunker | `ae524bedf^:crates/contextra-db/src/chunker/` | rund 1.100 Zeilen | Nur falls `ingest/chunking` ihn nicht abdeckt. Test `embedding_dimension_switch.rs` übernehmen |
| ONNX-Embedder und Reranker | `e5ec76c28^:crates/contextra-infer-onnx/` | rund 1.000 Zeilen | Als `contextra-embed`. `reqwest`-Pfad herauslösen nach `contextra-embed-http` |
| Candle-Embedder (GGUF) | `e5ec76c28^:crates/contextra-infer-candle/` | nur `embedding.rs`, `embedding_provider.rs`, `gguf_loader.rs` | Nicht in M3. Nur falls ONNX-Modelle nicht ausreichen. LLM-, Attention- und KV-Teile bleiben entfernt |
| Audit-Export | `dd895a253^:crates/contextra-audit-export/` | – | **Nicht** zurückholen (ADR-02) |

Vorgehen: `git checkout <commit>^ -- <pfad>` in einen Arbeitszweig, Compilerfehler beseitigen, auf neue Ports umstellen, Tests laufen lassen, erst dann in den Hauptzweig.

### 13.3 Migration

**Annahme:** Es gibt keine produktiven Daten im Format des heutigen Stands. Falls doch, ist vor M2 ein einmaliger Export über die heutige Fassade in ein neutrales Format (JSON Lines mit Quelldateien) und ein Import in die neue Struktur nötig. Dieser Export wird danach nicht weiter gepflegt (OPEN-5).

### 13.4 Dokumentation

**REQ-DOC-1.** Auf v4.0 gebracht oder entfernt: `docs/crates/*.md` (je Crate aus 3.1, die anderen gelöscht), `CLAUDE.md`, `AGENTS.md`, Crate-READMEs, `benchmarks/README.md` (bisher als „historisch“ gekennzeichnet). Veraltete Angaben (6- und 9-Schritt-Saga, `SubjectKey`, `m = 256 MB`, `Poisoned`, Profile, Erfüllungsklassen, Siegel) fallen weg.

---

## 14. Offene Punkte

| ID | Frage | Vorbelegung | Zeitpunkt |
|---|---|---|---|
| OPEN-1 | Rechtliche Einordnung der Schlüsselvernichtung als Löschung. Keine Aussage in dieser Spezifikation, keine Rechtsberatung. | Kombination aus Gate, Schlüsselvernichtung und gebündeltem Scrub, Einordnung durch den Verantwortlichen | vor Außenkommunikation |
| OPEN-2 | Auswahl der KMS-Anbieter für `contextra-kms` | Port stabil, Anbieter nach Zielumgebung | M6 |
| OPEN-3 | Hochverfügbarkeit und Replikation | Nicht-Ziel in v4.0. Voraussetzung wäre ein replizierbares Löschlog | nach M6 |
| OPEN-4 | Alte Formate (`.doc`, `.ppt`, `.msg`, ODF) im Ingest | nach Bedarf der ersten Kunden aus der Historie nachziehen | nach M3 |
| OPEN-5 | Migration bestehender Daten | Annahme: keine vorhanden (13.3) | vor M2 klären |

Alle übrigen früheren offenen Entscheidungen (D-1 bis D-10 aus v3.0) sind durch Kapitel 2 entschieden.

---

## Anhang A: Verbleib der Konzepte aus v2.2 und v3.0

| Altes Konzept | Neu |
|---|---|
| ER-1/2 `DocDek` je Version | Schlüssel je **Seal** (ADR-01), Version ohne Schlüsselbezug |
| ER-3 Wrap mit AES-GCM-SIV | beibehalten (REQ-KEY-2) |
| ER-4 Keine HKDF für Datenschlüssel | beibehalten (REQ-KEY-1) |
| ER-5 Gate vor Entschlüsselung | beibehalten (REQ-FLT-5) |
| ER-6 Zeitreise gegen aktuellen Stand | folgt aus der Trennung (REQ-VER-6) |
| ER-7/8 `fsync → Snapshot-Swap`, Quarantäne | beibehalten (REQ-GATE-1) |
| ER-9 Gate-Log-Verdichtung | Gate-Snapshot (7.8) |
| Gate-Log HMAC-Kette | entfällt, CRC und Sequenz (ADR-02) |
| ER-10/11 Generationen, Epochen-Rewrap | Epochen, Rewrap in Slot-Datei (REQ-KEY-5) |
| ER-12 KeyTable-Sharding | Slot-Datei mit Überschreiben an Ort (ADR-08) |
| ER-13 `DeleteAll` als Standard | `Restrict` als Standard (ADR-10) |
| ER-14 Trockenlauf | beibehalten, erweitert um Textentdeckung (7.3) |
| ER-15/16 Konfliktdokumente, Review | beibehalten (REQ-ER-2/3) |
| ER-17 `RedactSplit` | `RedactReingest` (REQ-ER-4) |
| ER-18 `Derived<T>` mit Quellmenge | ersetzt durch Seal-Bezug jedes Chunks und `SealScoped` |
| ER-20 Keine veralteten Ableitungen | beibehalten (Ableitungen verweisen auf Seals, werden gefiltert und im Scrub entfernt) |
| ER-21/22 Inkrementelle BM25-Statistik | `N` und Länge exakt, `df` beim Merge (REQ-TXT-2) |
| ER-23 Flache Suche, `zeroize` der Slots | HNSW mit routen-erhaltendem Scrub (REQ-VEC-5) |
| ER-24 Kein Umverdrahten | aufgegeben: Konsolidierung (REQ-VEC-5) |
| ER-25 Neuaufbau bei 20 % | Neuaufbau erst bei 30 % nach Konsolidierung |
| ER-26/27 Cluster-Segmente, Clusterkennung | entfallen (ADR-11) |
| ER-28 Handkodierte Formate | auf drei begrenzt (REQ-FMT-4) |
| ER-29 `segment_max_bytes` | beibehalten (REQ-TXT-1) |
| ER-30/31 Reihenfolge, frühe Vernichtung | frühe Vernichtung ist Standard (ADR-03) |
| ER-32 Bereinigungsfrist, fail-forward | `scrub_deadline`, Schlüssel schon vernichtet (7.4) |
| ER-33 Wiederanlauf | beibehalten (7.7) |
| ER-34 Einzelausführung | beibehalten (REQ-CON-4) |
| ER-35 bis 37 Bericht, JCS, Ed25519, Bescheinigung, Satzbibliothek | entfallen (ADR-02), ersetzt durch Ledger und Status (7.10) |
| ER-38 Inventarpflicht | entfällt mit der Messung |
| ER-39/40 Verifikation, Validierung, Prüfwerkzeug | entfallen. Vier-Augen-Prinzip als Option für Entscheidungen |
| ER-41 Sprach-Gate | als Dokumentationsregel (REQ-DOC-2) |
| ER-42 Zwei Spuren | beibehalten (REQ-BAK-1) |
| ER-43/44/45 Senke, Restore-Guard, Horizont | beibehalten, vereinfacht (7.11) |
| ER-46/47/48 Einschränkung, Automat | beibehalten, Automat auf fünf Zustände (7.5) |
| v3 Zustände `MeasuredA`, `MeasuredB`, `Reported`, `KeyDestroyBegun`, `Cleaned`, `Cleaning` | entfallen |
| v3 P-2 bis P-10 | durch dieses Dokument ersetzt |
| v3 Custody K1 bis K3 | K1, K1b, K3 (ADR-15) |
| v3 Modus `regulated`/`standard` | entfällt. Optionen `four_eyes`, `sink` |
| v3 REQ-IDX-1 Flach als Pflicht | exakt nur klein und als Referenz (ADR-04) |
| v3 REM-03 `WalNoHmac` entfernt | Datenpfad ohne HMAC. CRC32C, Datensätze je Seal per AEAD |
| v3 REQ-AUT-1 netzfreier Produktcode | netzfreier **Kern** (REQ-NET-1), Adapter-Crates dürfen Netzwerk |

## Anhang B: Feststellungen am Repository (Stand 20.11.2026)

Diese Punkte wurden bei der Erstellung geprüft:

- Der Workspace hat 23 Mitglieder, `contextra-vault` ist ausgeschlossen. Umfang laut Funktionsinventar: rund 244.000 Zeilen in `crates/` plus 5.200 in `benchmarks/`.
- `MAX_SEARCH_K` ist in sechs Crates definiert (`ids` 1.000, `types` 1.000, `graph` 1.000, `text` 10.000, `rank` 10.000, `vector` 10.000).
- `Cargo.lock` hat Version 4. Rust 1.89 liest sie. Die Maßnahme „Lockfile-Version 3 erzeugen“ aus v3.0 entfällt.
- Fehlende Dateien: `benchmarks/budgets.toml`, `docs/erasure.md`, die in v3.0 genannte `Finale Löscharchitektur.md`.
- Der Workspace-Lint steht auf `unsafe_code = "deny"`. `contextra-text` hat `default = ["std"]`. Weitere nicht-leere Standard-Features (laut v3.0 in `durable-fs`, `graph`, `rank`, `simd`, `contextra`, `testkit`) sind in M0 neu zu erheben.
- Das Repository enthält 5.952 Commits und sieben Tags. Ingest, Chunker, ONNX- und Candle-Embedder und Audit-Export wurden am 17. bis 19. November 2026 entfernt und sind in der Historie vorhanden (13.2).
- Der Build wurde **nicht** geprüft: In der Analyseumgebung war die Toolchain 1.89 nicht ladbar. Die Aussage aus v3.0, die Fassade kompiliere mit 20 Fehlern nicht, ist hier weder bestätigt noch widerlegt. M0 erhebt den Stand neu.
- Die Zeilenzahlen der Ziel-Tabelle (3.1) sind Schätzungen aus den Modulgrößen des Funktionsinventars, nicht gemessen.

## Anhang C: Glossar

| Begriff | Bedeutung |
|---|---|
| **Seal** | Unveränderliche Inhaltseinheit einer Dokumentrevision mit eigenem Schlüssel und dichter Ordinalzahl |
| **Gate** | Lesesperre über Seal-Ordinale (`Open`, `Restricted`, `Erased`) |
| **LiveMask** | Abgeleitete Bitmap lebender Chunk-Slots im Suchpfad |
| **Scrub** | Gebündelte Bereinigung gemeinsamer Strukturen (Postings, Graph, Registry) |
| **Konsolidierung** | Ersetzen toter Nachbarn in HNSW-Nachbarlisten durch deren lebende Nachbarn |
| **Routen-erhaltender Scrub** | Ersetzen des SQ8-Codes eines toten Knotens durch den Mittelwert lebender Nachbarn |
| **Ledger** | Nur-anhängen-Protokoll der Löschaufträge ohne Kennungen und Inhalte |
| **Senke** | Unabhängiger Speicher für Gate- und Vernichtungsereignisse, nicht Teil der Datensicherung |
| **ContentRef** | Undurchsichtiger Verweis der Versionierung auf einen Seal |

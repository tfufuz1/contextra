# Contextra — Gesamtspezifikation (normativ)

**Version:** 1.0  
**Datum:** 18. November 2026  
**Status:** FINAL — Einzige Quelle der Wahrheit  
**Geltungsbereich:** Gesamter Workspace `contextra`

---

## 0 Geltung und Rangfolge

Dieses Dokument und die zugehörigen Crate-Spezifikationen in `docs/crates/` sind die einzige normative Referenz für das Projekt. Alle vorherigen Spezifikationen (SPEC v6, SPEC v7, Implementierungsspezifikationen, Umsetzbarkeitsanalysen, Coupling-Reports) sind hiermit ersetzt und entfernt.

**Rangfolge:**
1. Diese Gesamtspezifikation (`docs/SPEC.md`)
2. Crate-Spezifikationen (`docs/crates/*.md`)
3. Crate-lokale `README.md`-Dateien

MUSS, DARF NICHT und SOLL gelten nach RFC 2119.

---

## 1 Produktdefinition

### 1.1 Was Contextra ist

Contextra ist eine **souveräne, offline betreibbare Such- und Wissens-Engine** für Unternehmen und Behörden. Sie bietet:

1. **Hybrid-Suche** (Vektor + Volltext + Graph) über lokale Datenbestände
2. **Versionierung und Zeitreise** über Daten und Regeln
3. **DSGVO-konforme Löschung** mit belegbarem Crypto-Shredding
4. **Souveränität**: keine Telemetrie, kein Fernzugriff, lokale Modelle, lokale Schlüssel

Contextra ist eine **eingebettete Rust-Bibliothek** (kein eigenständiger Server). Die primäre Schnittstelle ist die Rust-API, sekundär MCP (Model Context Protocol) über stdio.

### 1.2 Nicht-Ziele (dauerhaft)

- Keine automatisierte Entscheidung über natürliche Personen (Art. 22 DSGVO)
- Kein Training / Fine-Tuning mit personenbezogenen Daten
- Kein Dokumentenmanagementsystem
- Keine Compliance-Zusagen ohne gemessene Stufe — „DSGVO-konform" ist eine Bewertung des Verantwortlichen, keine Eigenschaft der Software (INV-7)

---

## 2 Architektur

### 2.1 Verbleibende Crates

Der aktive Workspace umfasst **23 Crates** in 4 Ebenen:

| Ebene | Crates | Regel |
|-------|--------|-------|
| **E0 — Fundament** | `ids`, `env`, `sys`, `durable-fs`, `simd`, `privacy`, `erasure` | Kern-Closure leer. Keine internen Abhängigkeiten (außer `ids`, `env` untereinander erlaubt). |
| **E1 — Domänenbausteine** | `mvcc`, `crypto`, `text`, `graph`, `rank`, `checkpoint`, `store`, `vector` | Kern-Closure ⊆ E0, max. 5 Mitglieder. |
| **E2 — Integration** | `types`, `ports`, `db`, `engine` (deprecated Façade), `contextra` | Darf E0, E1, E2 nutzen, nie E3. |
| **E3 — Endpunkte** | `mcp`, `testkit`, `bench` | Darf alles nutzen. |

**Excluded:** `vault` (dormant, Funktionalität in `erasure` aufgegangen)

### 2.2 Entfernte Crates (nicht reaktivieren ohne Gate G-REACTIVATE)

```
contextra-cognition    contextra-agent        contextra-adapt
contextra-wire         contextra-py           contextra-prompt-guard
contextra-ingest       contextra-infer-onnx   contextra-infer-ollama
contextra-infer-candle contextra-sandbox      contextra-router
contextra-kv-cache
```

### 2.3 Abhängigkeitsregeln

| Regel | Inhalt |
|-------|--------|
| **R1** | Kanten zeigen nur abwärts (E3→E2→E1→E0). |
| **R2** | Dev-Abhängigkeiten auf höhere Ebenen sind verboten. |
| **R3** | Default-Features MÜSSEN leer sein. |
| **R4** | Ein Konzept hat genau einen Besitzer. Keine Typ-Dubletten. |
| **R5** | `#![forbid(unsafe_code)]` in allen Crates außer `sys` und `simd`. |
| **R6** | Kein `unwrap`, `expect`, `panic!`, `todo!` in Nicht-Test-Code. |

### 2.4 Architekturdiagramm

```
┌─────────────────────── E3: Endpunkte ───────────────────────┐
│  mcp          testkit          bench                        │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────── E2: Integration ────────────────────┐
│  contextra ──► db ──► engine (deprecated, = pub use db::*)  │
│  types        ports                                         │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────── E1: Domäne ─────────────────────────┐
│  store    vector    text    graph    rank                    │
│  mvcc     crypto    checkpoint                              │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────── E0: Fundament ──────────────────────┐
│  ids   env   sys   durable-fs   simd   privacy   erasure    │
└─────────────────────────────────────────────────────────────┘
```

---

## 3 Datenklassifikation

### 3.1 Vier Datenklassen

Jedes Datum im System gehört zu genau einer Klasse:

| Klasse | Name | Beispiele | Verschlüsselung | Löschbar pro Betroffenen |
|--------|------|-----------|-----------------|-------------------------|
| **K** | Unternehmenswissen | Verträge, Richtlinien, Fachwissen | Ja (Collection-KEK) | Nein (nur Collection-Drop) |
| **P** | Personenbezogene Daten | Name, E-Mail, Interaktionen eines Betroffenen | Ja (Subject-DEK) | Ja (atomar, O(1) via Crypto-Shredding) |
| **D** | Abgeleitete Daten | Embeddings, Posting-Listen, Graph-Knoten, Caches | Verschlüsselt oder mit Lineage zum Quelldatum | Ja (kaskadiert über Lineage) |
| **E** | Löschmaterial | Journal, Ledger, Berichte, Attestierungen | Separater Schlüssel | Eigenständig, nach Aufbewahrungsfrist |

### 3.2 Kapselung personenbezogener Daten

**Kernentwurfsentscheidung:** Personenbezogene Daten (Klasse P) werden von allen anderen Daten physisch durch eigenes Schlüsselmaterial isoliert. Die atomare Löschbarkeit ergibt sich aus der Schlüsselhierarchie (§4), nicht aus physischem Löschen einzelner Bytes.

**Regeln:**

1. **P-Daten werden über `SubjectRef` adressiert.** Ein `SubjectRef` ist eine 128-Bit-Zufalls-ID, die den Betroffenen identifiziert, ohne dass die ID selbst personenbezogen ist. Die Zuordnung `SubjectRef ↔ natürliche Person` liegt außerhalb des Systems (beim Verantwortlichen).

2. **P-Daten bekommen einen eigenen DEK pro (Subject, Document, Version).** Dieser DEK wird aus dem `SubjectKey(subject_ref)` abgeleitet. Vernichtung des `SubjectKey` macht alle Daten dieses Betroffenen unlesbar — in O(1), unabhängig von der Datenmenge.

3. **K-Daten DÜRFEN NICHT `SubjectRef` enthalten.** Ein K-Dokument, das Personenbezug hat, MUSS als P klassifiziert werden oder der Personenbezug MUSS als P-Verweis (`PersonalRef`) eingefügt werden, nicht inline.

4. **D-Daten tragen `Lineage`.** Jede Ableitung (Embedding, Posting, Graph-Entität) verweist auf ihr Quelldatum. Bei Löschung des Quelldatums werden alle D-Daten über den Lineage-Index kaskadiert bereinigt.

---

## 4 Schlüsselhierarchie

### 4.1 Hierarchie (4 Stufen)

```
MasterKey (Passphrase → Argon2id → 256 Bit)
    │
    ├── HKDF ──► TenantKEK(tenant_id)
    │                │
    │                ├── HKDF ──► CollectionKEK(tenant_id, collection_id, epoch)
    │                │                │
    │                │                ├── HKDF ──► DocumentDEK(collection_id, doc_id, version)
    │                │                │                │
    │                │                │                └── AEAD ──► Payload (K-Klasse)
    │                │                │
    │                │                └── HKDF ──► SubjectKey(subject_ref)
    │                │                                 │
    │                │                                 └── HKDF ──► SubjectDocDEK(subject_ref, doc_id, version)
    │                │                                                  │
    │                │                                                  └── AEAD ──► P-Payload
    │                │
    │                └── (Epochenrotation bei Collection-Neuanlage nach Drop)
    │
    └── HKDF ──► ErasureKey (für Journal, Ledger, Berichte)
```

### 4.2 Algorithmen

| Funktion | Algorithmus | Crate |
|----------|------------|-------|
| AEAD | AES-256-GCM-SIV | `contextra-crypto` |
| Schlüsselableitung | HKDF-SHA-256 | `contextra-crypto` |
| Master-Key-Ableitung | Argon2id (Passphrase + Salt) | `contextra-crypto` |
| Nonces | 96 Bit, zufällig aus OS-RNG | `contextra-crypto` |
| Signatur (Berichte) | Ed25519 | `contextra-erasure` (Feature `signing`) |
| Hashing (IDs) | BLAKE3 | `contextra-ids` |

### 4.3 Bindungsregeln

1. **Eindeutigkeit:** Zwei Versionen eines Dokuments DÜRFEN NICHT denselben DEK teilen.
2. **AAD:** Jede AEAD-Operation bindet `(tenant_id, collection_id, doc_id, version)` als Associated Data. Chiffretext ist nicht in einen anderen Kontext verschiebbar.
3. **Fail-Closed:** Fehlt ein Schlüssel, bricht die Operation ab. KEIN Fallback auf Klartext, KEIN Fallback auf blake3-Ableitung, KEIN Null-Schlüssel als Default.
4. **Fehlbare Zufallszahlen:** `OsRng` wird über `try_fill_bytes` aufgerufen. Ein Fehler des Zufallsgenerators DARF NICHT zu einem Chiffretext führen.
5. **Zeroize:** Schlüsselmaterial implementiert `ZeroizeOnDrop`, nicht `Clone`, nicht `Debug`, nicht `Serialize`.

### 4.4 MasterKey-Persistenz

Der MasterKey wird aus einer Passphrase und einem persistierten Salt abgeleitet:

```
Salt: 32 Byte Zufall, gespeichert in <data_dir>/master.salt
MasterKey = Argon2id(passphrase, salt, m=256MB, t=4, p=1)
```

**Ohne Passphrase kein Betrieb mit Verschlüsselung.** `ContextraConfig::encryption` ist `Required` (Verschlüsselung obligatorisch, Passphrase muss gesetzt sein) oder `None` (kein Crypto-Shredding möglich, nur logische Löschung). Es gibt keinen Modus, der Verschlüsselung vortäuscht.

---

## 5 Löschkette

### 5.1 Löschziel-Typen

| Ziel | Schlüsselvernichtung | Auswirkung |
|------|---------------------|------------|
| **Einzeldokument (K)** | `DocumentDEK` widerrufen | Dieses eine Dokument + alle D-Ableitungen |
| **Betroffener (P)** | `SubjectKey` widerrufen | Alle Dokumente aller Collections dieses Betroffenen, O(1) |
| **Collection** | `CollectionKEK` widerrufen | Alle Dokumente der Collection, O(1) |
| **Mandant** | `TenantKEK` widerrufen | Alle Collections des Mandanten, O(1) |

### 5.2 Zustände eines Datums

```
Live ──► Gated ──► KeyRevoked ──► Purged ──► Measured ──► Completed
  │         │           │            │            │
  │         └───────────┴────────────┴────────────┴──► Pending(reason)
  ├──► Restricted (Art. 18 DSGVO)
  │         ├──► Lifted (Einschränkung aufgehoben, terminal)
  │         └──► Gated  (Einschränkung → Löschung)
  └──► Refused (Aufbewahrungspflicht aktiv, terminal)
```

| Zustand | Lesen | Schlüssel vorhanden | Reversibel |
|---------|-------|--------------------|-----------| 
| `Live` | Ja | Ja | — |
| `Gated` | Nein (ReadGate blockiert) | Ja | Nein (Gate wird nie zurückgenommen) |
| `KeyRevoked` | Nein | Nein | Nein |
| `Purged` | Nein | Nein | Nein |
| `Measured` | Nein | Nein | Nein |
| `Completed` | Nein | Nein | Nein (terminal) |
| `Pending` | Nein | Abhängig vom Phase | Recovery durch Hintergrundläufer |
| `Restricted` | Nur mit Audit-Eintrag | Ja | Ja (→ Lifted oder → Gated) |
| `Refused` | Ja | Ja | Nein (terminal) |

**Invariante:** Ab `Gated` ist das Datum für alle Lesepfade unsichtbar. Ab `KeyRevoked` ist es kryptographisch unlesbar. Beides ist irreversibel.

### 5.3 Die 6-Schritte-Lösch-Saga

Jeder Schritt ist idempotent und wiederholbar. Die Reihenfolge ist normativ.

#### Schritt 1 — Validierung und Intent-Journal (fsync #1)

1. **Aufbewahrungspflicht prüfen:** `RetentionAttestation` auswerten. Bei aktiver Pflicht → `Refused`.
2. **Duplikaterkennung:** HMAC über Target → bei aktiver Saga idempotent Receipt zurückgeben.
3. **Resolve:** Für jeden `ErasureLayer` ermitteln, welche Segmente/Dateien betroffen sind → `LayerScope`.
4. **Journal Intent schreiben** (fsync #1): `erasure_id`, `target`, `attestation`.

#### Schritt 2 — Gate setzen (fsync #2)

1. `CommitBarrier::exclusive()` — exklusiv gegen Writer.
2. `ReadGate` setzen: Ab jetzt liefern alle Lesepfade `Erased` für das Ziel.
3. Journal `Gated` schreiben (fsync #2).
4. Barrier freigeben.

**Kritische Reihenfolge:** Gate VOR Schlüsselvernichtung. Bei Absturz zwischen Schritt 2 und 3 ist das Datum über das Gate unlesbar, auch wenn der Schlüssel noch existiert.

#### Schritt 3 — Schlüsselvernichtung (fsync #3)

1. **Layer-Vorbereitung:** Jeder Layer erhält `on_gate` — RAM-Zustand vorbereiten (z.B. Vektor-Surrogate markieren).
2. **Schlüssel vernichten:** DEK/KEK/SubjectKey im Vault mit Zufallsbytes überschreiben, dann nullen, dann fsync.
3. **Verifikation:** `verify_key_absent` — Constant-Time-Vergleich, Slot muss Null sein.
4. Journal `KeyRevoked` schreiben (fsync #3).

**Ab hier ist das Datum kryptographisch unlesbar. Der Aufrufer erhält das Receipt zurück.**

#### Schritt 4 — Physische Bereinigung (Purge)

Läuft asynchron im Hintergrundläufer (`run_purge_window`). Exklusiv gegen Kompaktionen via `PurgeBarrier`.

**Reihenfolge der Sub-Schritte:**

| Prio | Sub-Schritt | Layer | Aktion |
|------|-------------|-------|--------|
| a | Snapshot-Widerruf | SnapshotRevoker | Snapshots bis `gated_seq` invalidieren |
| b | Text-Rewrite | TextLayer | Inverted-Index-Segmente ohne gelöschte DocIds neuschreiben |
| c | WAL-Drain | StoreLayer | WAL-Einträge flushen, Memtable leeren |
| d | SSTable-Rewrite | StoreLayer | Kompaktions-Rewrite ohne gelöschte Keys |
| e | Vektor-Reparatur | VectorLayer | HNSW-Graph reparieren, Surrogate entfernen |
| f | Graph-Bereinigung | GraphLayer | Entitäten und Kanten entfernen |
| g | Slot-Bereinigung | CryptoLayer | Residuale Datei-Schlüssel nullen |
| h | Cache-Invalidierung | Alle | In-Memory-Caches leeren |

#### Schritt 5 — Messung mit Positivkontrolle

1. **Canary einfügen:** Bekanntes Byte-Muster über den Schreibpfad einfügen, flushen, kompaktieren.
2. **Positivkontrolle:** Scanner MUSS den Canary finden. Schlägt dies fehl → Messung ungültig.
3. **Canary löschen und messen:** Nach Löschung des Canary DARF das Muster in keiner Datei des Datenverzeichnisses im Klartext vorkommen.
4. **Lesepfad-Test:** Eine Leseabfrage auf das gelöschte Datum MUSS `Erased` liefern.
5. **Schichtweise Witness-Sammlung:** Jeder Layer liefert `LayerWitness` mit `residue_matches`. Alle müssen 0 sein.

#### Schritt 6 — Bericht und Abschluss

1. **Bericht signieren:** Ed25519-signierter JSON-Bericht mit `erasure_id`, `target_kind`, `witnesses`, `attestation.code`, `completed_at`. KEIN PII im Bericht.
2. **Ledger schreiben:** Hash-verketteter Eintrag AUSSERHALB des Datenverzeichnisses.
3. Journal `Completed` schreiben (terminal).
4. Gate-Eintrag bereinigen.
5. Observer-Event `Completed`.

### 5.4 Erfüllungsklassen

| Klasse | Name | Erreicht, wenn |
|--------|------|----------------|
| **L** | Logisch entfernt | Gate gesetzt, Lesepfad liefert `Erased` |
| **K** | Schlüssel vernichtet | DEK/KEK/SubjectKey widerrufen, durabel, Positivkontrolle bestanden |
| **I** | Indizes bereinigt | Alle Layer-Witnesses mit `residue_matches == 0` |
| **M** | Vollständig gemessen | K + I + alle Schichten gemessen oder deklariert |

**Wortregel:** „gelöscht" steht nur mit Klasse K oder höher. Bei L heißt es „logisch entfernt".

### 5.5 Recovery nach Crash

```
Neustart
  ├─► journal.recover_open_sagas() → [Sagas mit letztem Zustand]
  ├─► Für jede nicht-terminale Saga: in active_sagas, beim nächsten run_purge_window aufgreifen
  └─► Für Restricted-Sagas: Gate ist gesetzt, ReadGate blockiert korrekt
```

Alle Schritte sind idempotent. Der Journal-Zustand ist der Wahrheitsspeicher.

### 5.6 P-Klasse: Atomare Löschung eines Betroffenen

Wenn ein Betroffener (Subject) gelöscht wird:

1. `SubjectKey(subject_ref)` wird vernichtet → O(1)
2. Alle Dokumente dieses Subjects über alle Collections sind sofort kryptographisch unlesbar
3. Der Lineage-Index liefert alle D-Ableitungen (Embeddings, Postings, Graph-Knoten)
4. Die Purge-Phase bereinigt alle D-Daten kaskadiert
5. `PersonalRef`-Verweise in K-Dokumenten werden zu toten Links (zeigen ins Leere)

**Das ist der Kern des Entwurfs:** Personenbezogene Daten werden durch eigenes Schlüsselmaterial physisch vom Rest getrennt. Die Löschung eines Betroffenen erfordert kein Scannen oder Umschreiben von K-Daten — nur die Vernichtung eines einzigen Schlüssels.

### 5.7 Restrict-Modus (Art. 18 DSGVO)

Separater Pfad: Gate wird mit `GateDecision::Restricted` statt `Erased` gesetzt. Lesezugriff nur mit vorherigem Audit-Eintrag. Kein physischer Purge. Übergang `Restricted → Gated` (Löschung) oder `Restricted → Lifted` (Aufhebung) möglich.

---

## 6 Lesesperre (ReadGate)

### 6.1 Anforderungen

- **O(1) ohne I/O, ohne Blockierung** — der Gate-Check ist ein synchroner In-Memory-Lookup
- **Fail-Closed** — bei unklarem Zustand: ablehnen, nie durchlassen
- **Gleiches Fehlerbild** — ein gelöschtes Dokument ist nicht von einem nie existierenden unterscheidbar
- **Vor Entschlüsselung** — der Gate-Check erfolgt vor jedem Zugriff auf Chiffretext

### 6.2 Prüfpunkte (vollständig)

Jeder einzelne Lesepfad MUSS vor der Rückgabe den ReadGate prüfen:

| Lesepfad | Gate-Check-Stelle |
|----------|-------------------|
| `get`, `get_at_snapshot` | Vor Entschlüsselung |
| `scan`, `scan_prefix` | Je Eintrag vor Ausgabe |
| Vektor-/Textsuche | Auf Kandidatenebene vor Fusion/Rerank |
| Hybrid-Suche | Vor Fusion, mit Auffüllen auf `k` |
| Graph-Traversal | Je Kante und Zielknoten |
| Export | Je Eintrag; gesperrte werden ausgelassen |
| MCP-Tools | Über die darunterliegenden Pfade |
| Kompaktionen | `variant_for_range` → `ContainsErased` → Range nicht in Ergebnis aufnehmen |

### 6.3 ReadGate-Trait

```rust
pub trait ReadGate: Send + Sync {
    fn check(&self, key: &[u8]) -> GateDecision;       // O(1)
    fn check_range(&self, range: &KeyRange) -> RangeGate;
}

pub enum GateDecision { Open, Erased, Restricted }
pub enum RangeGate { AllOpen, ContainsErased, RestrictedNeverDiscard }
```

---

## 7 ErasureLayer — Schnittstelle für Layer-Implementierer

Jede Speicher- oder Indexschicht, die an der Löschkette teilnimmt, implementiert:

```rust
pub trait ErasureLayer: Send + Sync {
    fn id(&self) -> LayerId;
    fn key_coverage(&self) -> KeyCoverage;  // Full | None

    fn resolve(&self, target: &EraseTarget)
        -> BoxFuture<'_, Result<LayerScope, ErasureError>>;

    fn on_gate(&self, scope: &LayerScope)
        -> BoxFuture<'_, Result<(), ErasureError>>;

    fn purge(&self, scope: &LayerScope, cx: &PurgeCx)
        -> BoxFuture<'_, Result<PurgeOutcome, ErasureError>>;

    fn witness(&self, scope: &LayerScope, scanner: &Scanner<Armed>)
        -> BoxFuture<'_, Result<LayerWitness, ErasureError>>;
}
```

**Invarianten für Implementierer:**
1. `purge` ist idempotent: zweiter Aufruf = no-op
2. `witness` ist rein lesend: ändert nie persistenten Zustand
3. `LayerWitness` wird nur in `contextra-ports::scanner` definiert
4. Logs enthalten nie Klartextinhalte des Ziels

**Layer-Zuordnung:**

| Layer | Crate | Priorität |
|-------|-------|-----------|
| StoreLayer | `contextra-store` | WAL-Drain (c), SSTable-Rewrite (d) |
| TextLayer | `contextra-text` | Segment-Rewrite (b) |
| VectorLayer | `contextra-vector` | HNSW-Reparatur (e) |
| GraphLayer | `contextra-graph` | Entitäten/Kanten (f) |
| CryptoLayer | `contextra-crypto` | Slot-Bereinigung (g) |

---

## 8 Lineage-System

### 8.1 Zweck

Jede Ableitung (D-Klasse) verweist auf ihr Quelldatum. Bei Löschung des Quelldatums liefert der Lineage-Index alle betroffenen Ableitungen für die kaskadierte Bereinigung.

### 8.2 Datenmodell

```rust
pub struct LineageRef {
    pub source_class: DataClass,     // K oder P
    pub source_id: SourceId,         // DocId oder SubjectRef
    pub source_version: u64,
}

pub struct DerivedRef {
    pub layer: LayerId,
    pub segment_id: u64,
    pub offset: u64,
}
```

### 8.3 Regeln

1. Jeder Schreibvorgang in D-Klasse MUSS eine `LineageRef` angeben.
2. `LineageIndex::derived_of(source, version)` liefert alle `DerivedRef`s.
3. Die Purge-Phase (Schritt 4) nutzt den Lineage-Index, um alle betroffenen Ableitungen zu finden.
4. Ohne Lineage keine Aussage „Bereinigt" (Klasse I nicht erreichbar).

---

## 9 Zeitreise

### 9.1 Zwei unabhängige Achsen

| Achse | Typ | Semantik |
|-------|-----|----------|
| **Transaktionszeit** (`as_of`) | `u64` (Sequenznummer oder Unix-ms) | „Was war zum Zeitpunkt X im System bekannt?" |
| **Gültigkeitszeit** (`valid_at`) | `i32` (Kalendertage seit 1970-01-01) | „Was galt zum fachlichen Zeitpunkt Y?" |

### 9.2 Regeln

1. **Unabhängig kombinierbar:** `as_of` wählt den MVCC-Stand, `valid_at` filtert die fachliche Gültigkeit.
2. **Fehlend = offen:** `None` bedeutet „offen" (kein Ende / kein Anfang), nicht „0".
3. **Zugriffsrichtlinie aus aktuellem Stand:** Der ReadGate wird immer mit dem aktuellen Stand geprüft, nicht mit historischen Rechten (Z-T5).
4. **Spätere Schreibvorgänge ändern frühere `as_of`-Antworten nicht** (Z-T2).

---

## 10 Concurrency-Barrieren

### 10.1 CommitBarrier

```
Writer 1: shared() → [schreibt]
Writer 2: shared() → [schreibt]
Saga:     exclusive() → [setzt Gate] → drop permit
```

Schützt den Gate-Gruppen-Commit gegen parallele Writer. `shared()` erlaubt mehrere Writer gleichzeitig. `exclusive()` wartet auf alle shared-Permits und blockiert neue.

### 10.2 PurgeBarrier

```
Kompaktions-Job: compaction() → [kompaktiert]
Saga-Purge:      purge() → [exklusiver Purge] → drop permit
```

Schützt den Purge gegen parallele Kompaktionen.

---

## 11 Fehlerverhalten

### 11.1 Fehler-Hierarchie

| Kategorie | Fehler | Verhalten |
|-----------|--------|-----------|
| **Transient** | I/O-Fehler im Index | Wiederholung mit Backoff, Zustand bleibt |
| **Nach Gate** | Fehler nach Schritt 2 | Datum bleibt `Gated`, Bereinigung wird nachgeholt |
| **Nach KeyRevoked** | Fehler nach Schritt 3 | Datum bleibt `KeyRevoked`, Bereinigung wird nachgeholt |
| **Beim Widerruf** | Fehler in Schritt 3 | Operation schlägt fehl, Datum bleibt `Gated`, Aufrufer erhält Fehler |
| **Im Lesepfad** | Fehlender Schlüssel | `Erased` — kein Unterschied zu „nie existiert" |
| **Crypto** | AEAD-Fehler, falscher Tag | Einheitlicher Fehler, nicht unterscheidbar |

### 11.2 Fail-Closed-Prinzip

- Fehlender Schlüssel → Fehler, nie Klartext-Fallback
- Fehlender ReadGate → Öffnungsfehler (bei `GuardPolicy::Required`)
- Unbekannter Gate-Zustand → Ablehnen
- Jeder Fehler im Löschpfad MUSS im Bericht erscheinen

---

## 12 Testkatalog (Löschkette)

| Kennung | Prüft | Kategorie |
|---------|-------|-----------|
| **T-DEL-1** | Nach Insert+Flush: kein Klartext-Canary in Dateien des Datenverzeichnisses | Integration |
| **T-DEL-2** | Nach Löschung+Flush+Compaction: kein Klartext-Canary; Lesepfad liefert `Erased` | Integration |
| **T-DEL-3** | Löschnachweis schlägt mit falschem Schlüssel fehl | Unit |
| **T-DEL-4** | Nach Löschung und Neustart bleibt das Datum `Erased` | Integration |
| **T-DEL-5** | `CryptoShred` führt Widerruf aus; ohne Verschlüsselung wird Klasse K nicht erreicht | Integration |
| **T-DEL-6** | Nach Löschung findet weder Volltext- noch Vektorsuche das Dokument | Integration |
| **T-DEL-7** | Positivkontrolle: Scanner findet eingebettetes Muster | Unit |
| **T-DEL-8** | Collection-Drop widerruft CollectionKEK; alle Dokumente `Erased` | Integration |
| **T-DEL-9** | Subject-Löschung: SubjectKey vernichten → alle P-Daten des Subjects `Erased` | Integration |
| **T-DEL-10** | Lineage-Kaskade: Quelldatum gelöscht → alle D-Ableitungen bereinigt | Integration |
| **T-DEL-11** | ReadGate blockiert alle Lesepfade (§6.2 vollständig) | Integration |
| **T-DEL-12** | Crash in Phase 3 → Recovery → Saga wird fortgesetzt | Integration |
| **T-DEL-13** | Restrict-Modus: Lesezugriff nur mit Audit-Eintrag | Integration |

---

## 13 Invarianten

| ID | Invariante |
|----|------------|
| **INV-1** | Kein persistiertes Byte ändert sich ohne Formatvermerk. |
| **INV-2** | Jeder öffentliche Pfad bleibt auflösbar. |
| **INV-3** | `forbid(unsafe_code)` außerhalb der Inseln (`sys`, `simd`). |
| **INV-4** | Kein `unwrap`/`expect`/`panic!`/`todo!` in Nicht-Test-Code. |
| **INV-5** | Gleiche Eingabe → gleiche Rangfolge, Tie-Break ID aufsteigend. |
| **INV-6** | WAL, Manifest, Flush, Recovery-Verhalten unverändert bei Umbau. |
| **INV-7** | Keine Compliance-Aussagen. „Gelöscht" nur mit gemessener Stufe. |
| **INV-8** | Abhängigkeitskanten nur abwärts. |
| **INV-9** | `Completed` und `Refused` sind terminale Saga-Zustände. |
| **INV-10** | Gate wird nie zurückgenommen. |
| **INV-11** | Schlüsselbytes werden mit `zeroize` genullt vor Drop. |
| **INV-12** | Logs/Traces enthalten nie Klartextinhalte des Löschziels. |

---

## 14 Konfiguration

```rust
pub struct ContextraConfig {
    pub data_dir: PathBuf,
    pub encryption: EncryptionConfig,
    pub tenant_policy: TenantPolicy,
    pub guard_policy: GuardPolicy,
    pub data_class_policy: DataClassPolicy,
    pub erasure: ErasureConfig,
}

pub enum EncryptionConfig {
    None,                              // Kein Crypto-Shredding möglich
    Required { passphrase: SecretString },  // Verschlüsselung obligatorisch
}

pub enum GuardPolicy {
    Optional,    // ReadGate kann, muss aber nicht gesetzt sein
    Required,    // Ohne ReadGate kein Öffnen — für regulierte Umgebungen
}

pub enum DataClassPolicy {
    Legacy,      // Alte Schreibmethoden ohne Klasse erlaubt
    Enforced,    // Jeder Schreibvorgang MUSS Klasse deklarieren
}

pub struct ErasureConfig {
    pub ledger_dir: PathBuf,           // AUSSERHALB des Datenverzeichnisses
    pub batch_window_secs: u64,        // Intervall für run_purge_window
    pub disk_reserve_bytes: u64,       // Mindest-Plattenreserve für Purge
}
```

---

## 15 Glossar

| Begriff | Definition |
|---------|------------|
| **Canary** | Bekanntes Byte-Muster, das über den Schreibpfad eingefügt wird, um den Scanner zu kalibrieren |
| **Crypto-Shredding** | Vernichtung des Schlüssels macht Chiffretext unlesbar, ohne physisches Löschen |
| **DEK** | Data Encryption Key — verschlüsselt ein einzelnes Payload |
| **Gate** | Synchrone Lesesperre, die Zugriff auf gelöschte Daten verhindert |
| **KEK** | Key Encryption Key — umhüllt DEKs einer Gruppe |
| **Lineage** | Herkunftsverweis von abgeleiteten Daten auf ihr Quelldatum |
| **PersonalRef** | 128-Bit-Zufalls-ID als Verweis auf P-Daten in K-Dokumenten |
| **Saga** | Die 6-Schritte-Löschsequenz mit Journal und Recovery |
| **SubjectRef** | 128-Bit-Zufalls-ID, die einen Betroffenen identifiziert |
| **SubjectKey** | Aus CollectionKEK abgeleiteter Schlüssel für einen Betroffenen |

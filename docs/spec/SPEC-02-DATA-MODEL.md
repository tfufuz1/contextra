# SPEC-02 — Datenmodell: Cells, Facets, Zoom

**Version 2.0 · November 2026 · Status: Final**
**Querverweise:** [Index](SPEC-00-INDEX.md) · [Architektur](SPEC-01-ARCHITECTURE.md) · [Storage](SPEC-03-STORAGE-ENGINE.md) · [Versionierung](SPEC-04-VERSIONING.md)

---

## 1. Namespace

Ein **Namespace** ist eine Sammlung mit gemeinsamer Policy (vergleichbar mit Collection/Table). Policies sind Teil des versionierten Zustands (im `RootSet`). Namespaces werden durch die Engine verwaltet und dürfen nicht direkt vom Dateisystem abgebildet werden.

### 1.1 Namespace-Identität

```rust
/// Namespace-Identifikator, sequentiell pro Repository vergeben.
/// u32 reicht für 4 Milliarden Namespaces — praktisch unbegrenzt.
pub struct NsId(u32);

/// Lesbare Namespace-Pfade, hierarchisch mit `/` getrennt.
/// Beispiele: "agent/episodic", "docs/api", "_system/work"
pub struct NsPath(Arc<str>);
```

**Reservierte Präfixe:**
- `_system/` — Engine-interne Namespaces (Work Queue, Idempotenz-Journal, Alias-Tabelle)
- `_gc/` — Tombstones und GC-Metadaten

### 1.2 Namespace-Arten (exhaustive)

| Kind | Semantik | Typischer Inhalt | Standard-Retention |
|---|---|---|---|
| `episodic` | Zeitlich geordnete Erlebnisse | Konversationsturns, Tool-Ergebnisse, Beobachtungen | hot 14d, warm 180d, zoom-down nach 180d |
| `semantic` | Fakten und Wissen | Extrahierte Fakten, Entity-Profile, Definitionen | unbegrenzt |
| `procedural` | Handlungswissen | Skills, Runbooks, Workflows, Rezepte | unbegrenzt |
| `working` | Kurzlebiges Arbeitsgedächtnis | Scratchpad, Hypothesen, Zwischenergebnisse | TTL 24h, dann verwerfen |
| `artifact` | Erzeugte Artefakte | Code, Dokumente, Reports | unbegrenzt |
| `reference` | Externe Referenzmaterialien | Importierte Dokumente, API-Docs, Manuals | unbegrenzt, zoom-down nach 365d |

### 1.3 Namespace-Policy (vollständiges Schema)

```toml
[namespace."agent/episodic"]
kind              = "episodic"
chunker           = "markdown-v2"
tokenizers        = ["cl100k", "o200k"]     # Token-Zählungen für diese Tokenizer-Familien
embedders         = ["emb-768-v3"]           # Embedding-Modelle
embed_dimensions  = [768]                    # Dimensionen je Embedder
zoom_ladder       = [
    "L0:raw",
    "L1:summary<=200tok",
    "L2:title<=24tok"
]
default_trust     = "tool-unverified"
max_cell_size     = "256KiB"                 # Maximale Rohgröße einer Cell
max_zoom_levels   = 5                        # Maximale Zoom-Stufen
retention         = { hot_days = 14, warm_days = 180, zoom_down_after = "180d" }
merge             = { text = "diff3", tags = "union", counters = "sum", default = "manual" }
durability        = "durable"                # volatile | buffered | durable | replicated
encryption        = "none"                   # none | aes256gcm | xchacha20poly1305
sequence_enabled  = true                     # ob Cells in diesem NS Sequenzen bilden
```

### 1.4 Namespace-Lifecycle

| Zustand | Beschreibung |
|---|---|
| `active` | Lese- und Schreibzugriff |
| `read_only` | Nur Lesezugriff, keine neuen Cells |
| `archived` | Nur über Zeitreise erreichbar, Cold-Tier |
| `deleted` | Tombstone im RootSet, GC räumt Daten auf |

Übergänge: `active → read_only → archived → deleted`. Jeder Übergang ist ein Commit. Rückwärts-Übergänge: `archived → read_only → active` (kein Datenverlust, da immutable).

---

## 2. Cell — die zentrale Primitive

### 2.1 Identität

```rust
/// Stabile, ortsunabhängige Identität eines Inhaltsblocks (BLAKE3-256).
/// Berechnet über den kanonischen Inhalt (rkyv-serialisiertes Cell ohne `derived`-Feld).
pub struct ChunkId([u8; 32]);

/// Logische Identität einer Cell, stabil über Versionen hinweg.
pub struct CellId {
    pub ns: NsId,
    pub key: CellKey,
}

/// Zeitlich sortierbare Zell-Schlüssel.
/// Format: 6 Bytes Millisekunden-Timestamp + 10 Bytes Zufall (ULID-kompatibel).
/// Bei anwendungsdefinierten Schlüsseln: beliebige 16 Bytes, nicht zeitlich sortierbar.
pub struct CellKey([u8; 16]);
```

**Invariante:** `CellId` ist über alle Versionen einer Cell stabil. `ChunkId` ändert sich bei jeder Inhaltsänderung.

### 2.2 Cell-Struktur

```rust
/// Kanonischer, content-addressed Teil einer Cell.
pub struct Cell {
    pub id: CellId,
    pub kind: CellKind,
    pub raw: ChunkRef,                  // kanonischer Inhalt (UTF-8 / JSON / Binary)
    pub attrs: Attrs,                   // typisierte Attribute
    pub edges: SmallVec<[Edge; 4]>,     // typisierte Kanten
    pub time: BiTemporal,               // Transaktionszeit + Gültigkeitszeit
    pub trust: TrustLabel,              // Vertrauensstufe
    pub prov: Provenance,               // Herkunft
    pub derived: DerivedRefs,           // ← NICHT Teil des ChunkId-Hash
}
```

**Hash-Berechnung:** `ChunkId = BLAKE3(rkyv::to_bytes(Cell { derived: DerivedRefs::EMPTY, ..self }))`. Das `derived`-Feld wird vor dem Hashen auf leer gesetzt, da es abgeleitete Daten referenziert, die sich unabhängig vom Inhalt ändern.

### 2.3 CellKind (exhaustive Enumeration)

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, rkyv::Archive)]
#[repr(u8)]
pub enum CellKind {
    /// Konversationsturn (User, Assistant, System, Tool-Aufruf)
    Episode      = 0,
    /// Persistenter Fakt / Erkenntnis
    Fact         = 1,
    /// Importiertes oder erzeugtes Dokument / Dokumentfragment
    Doc          = 2,
    /// Ergebnis eines Tool-Aufrufs
    ToolResult   = 3,
    /// Plan oder Planschritt
    Plan         = 4,
    /// Handlungswissen / Skill-Beschreibung
    Skill        = 5,
    /// Automatisch oder manuell erzeugte Zusammenfassung
    Summary      = 6,
    /// Entity-Profil (aggregiert aus Fakten)
    Profile      = 7,
    /// Code-Artefakt
    Code         = 8,
    /// Konfiguration / Policy
    Config       = 9,
    /// Annotation / Kommentar zu einer anderen Cell
    Annotation   = 10,
    /// Widerspruch / Konfliktdokumentation
    Contradiction = 11,
    /// Hypothese / spekulativer Inhalt
    Hypothesis   = 12,
    /// Beobachtung (Sensor, Log, Monitoring)
    Observation  = 13,
    /// Merge-Konflikt (automatisch erzeugt)
    Conflict     = 14,
    /// System-intern (Work-Queue-Task, GC-Metadaten)
    System       = 15,
}
```

Erweiterung über Feature-Flags möglich (Werte 16–255 reserviert). Unbekannte Kinds werden beim Lesen als `Unknown(u8)` behandelt und transparent durchgereicht.

### 2.4 Typisierte Attribute (Attrs)

```rust
/// Attribute sind ein geordnetes, dedupliziertes Set von typisierten Key-Value-Paaren.
/// Keys sind interned Strings (Namespace-Policy definiert erlaubte Keys).
pub struct Attrs {
    entries: SmallVec<[(AttrKey, AttrValue); 8]>,
}

#[derive(Clone, PartialEq, rkyv::Archive)]
pub enum AttrValue {
    /// UTF-8 String, max 1 KiB
    Str(CompactString),
    /// 64-bit signed Integer
    Int(i64),
    /// 64-bit IEEE 754
    Float(f64),
    /// Boolescher Wert
    Bool(bool),
    /// Zeitpunkt (Millisekunden seit Unix-Epoch)
    Timestamp(i64),
    /// Dauer in Millisekunden
    Duration(u64),
    /// Menge von Strings (Tags)
    Tags(BTreeSet<CompactString>),
    /// Enum-Wert (Index in Policy-definierter Wertemenge)
    Enum(u16),
    /// Binäre Daten, max 4 KiB (Signaturen, kleine Hashes)
    Bytes(SmallVec<[u8; 32]>),
}

/// Interned Attribute Key — u16-Index in die Policy-definierte Attributliste.
pub struct AttrKey(u16);
```

**Standardattribute** (immer verfügbar, unabhängig von Policy):

| Key | Typ | Beschreibung |
|---|---|---|
| `title` | Str | Kurztitel der Cell (extraktiv oder manuell) |
| `lang` | Str | ISO 639-1 Sprachcode |
| `mime` | Str | MIME-Type des Rohinhalts |
| `size_bytes` | Int | Größe des Rohinhalts in Bytes |
| `tags` | Tags | Freitext-Tags |
| `importance` | Float | Explizite Wichtigkeit (0.0–1.0, NaN = nicht gesetzt) |
| `pinned` | Bool | Vor Retention/Vergessen geschützt |

### 2.5 Typisierte Kanten (Edges)

```rust
pub struct Edge {
    pub kind: EdgeKind,
    pub target: CellId,
    pub weight: f32,        // 1.0 = Standard; 0.0–∞
    pub attrs: SmallAttrs,  // optionale Kantenattribute (max 4 Paare)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, rkyv::Archive)]
#[repr(u8)]
pub enum EdgeKind {
    // ── Strukturelle Kanten ──
    /// Gehört zu (Chunk → Dokument, Turn → Episode)
    PartOf       = 0,
    /// Folgt zeitlich/sequentiell auf
    Follows      = 1,
    /// Ist Vorgänger von (inverse Follows)
    Precedes     = 2,
    /// Enthält (Dokument → Chunks)
    Contains     = 3,

    // ── Semantische Kanten ──
    /// Zitiert / referenziert
    Cites        = 10,
    /// Abgeleitet aus (Summary aus Quelle, Fakt aus Episode)
    DerivedFrom  = 11,
    /// Ersetzt eine frühere Version
    Supersedes   = 12,
    /// Widerspricht
    Contradicts  = 13,
    /// Unterstützt / bestätigt
    Supports     = 14,
    /// Ist verwandt mit (schwache semantische Verbindung)
    RelatedTo    = 15,

    // ── Agenten-Kanten ──
    /// Erzeugt durch (Cell → Agent/Tool)
    CreatedBy    = 20,
    /// Verwendet in (Cell wurde in Recall-Ergebnis genutzt)
    UsedIn       = 21,
    /// Annotiert (Kommentar → Ziel)
    Annotates    = 22,

    // ── Entity-Kanten ──
    /// Betrifft Entity (Cell → Entity-Profil)
    About        = 30,
    /// Hat Eigenschaft (Entity → Fakt)
    HasProperty  = 31,
}
```

**Kardinalität:** Kanten sind gerichtet. Rückwärts-Traversal wird über den Graph-Index ermöglicht (siehe [SPEC-05](SPEC-05-INDEXES.md)). Eine Cell darf maximal **256 Kanten** haben (darüber hinaus: Verwendung von Adjazenzlisten im Graph-Index).

### 2.6 Provenance — Herkunft als First-Class-Citizen

```rust
pub struct Provenance {
    pub author: PrincipalId,       // Mensch, Agent, System
    pub source: SourceRef,         // URI, Tool-Call-ID, Dokument-Referenz
    pub model: Option<ModelRef>,   // bei LLM-generierten Inhalten
    pub confidence: Option<f32>,   // 0.0–1.0
    pub run_id: Option<RunId>,
    pub step_id: Option<StepId>,
}

/// Autor-Identifikation.
pub struct PrincipalId {
    pub kind: PrincipalKind,  // User, Agent, System, External
    pub id: CompactString,    // eindeutiger Bezeichner
}

/// Modellreferenz für LLM-erzeugte Inhalte.
pub struct ModelRef {
    pub model_id: CompactString,    // z.B. "gpt-4o-2024-11-20"
    pub prompt_hash: [u8; 16],      // BLAKE3-128 des verwendeten Prompts
    pub temperature: Option<f32>,
}
```

Provenance ist **unveränderlich** und Teil des ChunkId-Hash. Sie ermöglicht:
- Nachvollziehbarkeit jedes Datenpunkts bis zur Quelle
- Confidence-basiertes Ranking (höheres Vertrauen bei verifizierten Quellen)
- Modellwechsel-Tracking (welches Modell hat welche Ableitung erzeugt)

---

## 3. Facetten (Lanes)

Jede Cell wird physisch in **Lanes** zerlegt, die getrennt gespeichert werden (siehe [SPEC-03 Extent-Layout](SPEC-03-STORAGE-ENGINE.md)).

| Lane | Inhalt | Eigenschaft | Hot/Cold | Typische Größe |
|---|---|---|---|---|
| `meta` | CellKind, Attrs, Trust, Zeitstempel, Kanten-Kurzliste | klein, selektiv | **hot** | 64–512 B |
| `tokcount` | Token-Anzahl pro registriertem Tokenizer (u32 je Tokenizer) | Budget-Planung ohne Inhaltszugriff | **hot** | 4–16 B |
| `zoom.L2` | Titel / kürzeste Zusammenfassung | die meisten Recalls nutzen nur diese | **hot** | 20–100 B |
| `zoom.L1` | Zusammenfassung | häufig genutzt | **hot** | 100–800 B |
| `emb.q` | Quantisierter Embedding-Code (int8/binär, Matryoshka-Präfix) | Kandidatensuche | **hot** | 384–768 B |
| `raw` (= `zoom.L0`) | Voller Inhalt | groß | **cold** | 100 B – 256 KiB |
| `tok.ids` | Token-IDs (bitgepackt, optional) | Spart Re-Tokenisierung | **cold** | variabel |
| `emb.full` | Embedding in f16/f32 | Re-Ranking | **cold** | 1.5–3 KiB |

### 3.1 Lane-Präsenz

Nicht jede Cell hat jede Lane. Pflicht-Lanes:
- `meta`: immer
- `raw`: immer (auch wenn leer, z.B. bei Tombstones)
- `tokcount`: immer wenn `raw` nicht leer

Optionale Lanes werden über die Namespace-Policy gesteuert. Fehlende Lanes werden als `None` behandelt, nie als Fehler.

---

## 4. Zoom-Leiter

### 4.1 Zoom-Level-Definition

```rust
pub struct ZoomLevel {
    pub level: u8,              // 0 = voll, höher = gröber
    pub text: ChunkRef,         // Inhalt dieser Stufe
    pub tok_count: SmallVec<[(TokenizerId, u32); 2]>,  // Token-Kosten
    pub derived_from: ChunkId,  // Quelle (L0 oder nächstfeinere Stufe)
    pub producer: ProducerId,   // Wer hat diese Stufe erzeugt
    pub quality: ZoomQuality,   // Qualitätseinschätzung
}

#[derive(Clone, Copy)]
pub enum ZoomQuality {
    /// Extraktiv erzeugt (deterministisch, Engine-intern)
    Extractive,
    /// Abstraktiv erzeugt (LLM-Summary, über Work Queue)
    Abstractive { model: ModelRef },
    /// Manuell erstellt
    Manual,
}

/// Identifikation des Erzeugers einer abgeleiteten Facette.
pub enum ProducerId {
    /// Engine-interne, deterministische Erzeugung (z.B. Titel-Extraktion)
    Engine { version: u32 },
    /// Externes Modell
    Model { model_id: CompactString, prompt_hash: [u8; 16] },
    /// Menschlicher Autor
    Human { principal: PrincipalId },
}
```

### 4.2 Standard-Zoom-Stufen

| Stufe | Name | Ziel-Token | Erzeugung | Beschreibung |
|---|---|---|---|---|
| L0 | `raw` | unbegrenzt | Originaldaten | Vollständiger Inhalt |
| L1 | `summary` | ≤ 200 | abstraktiv (Work Queue) oder extraktiv | Zusammenfassung |
| L2 | `title` | ≤ 24 | extraktiv (Engine-intern) | Titel / erste Zeile / Schlüsselfelder |
| L3 | `handle` | 1 | Engine-intern | Nur Handle-ID, kein Text |

Namespace-Policies können zusätzliche Stufen definieren (max 5 Stufen gesamt, konfigurierbar via `max_zoom_levels`).

### 4.3 Extraktive Zoom-Erzeugung (deterministisch)

Die Engine erzeugt L2-Stufen **synchron beim Commit** über regelbasierte Extraktion:

| CellKind | Extraktionsregel für L2 |
|---|---|
| Episode | Erste 24 Token des Turns |
| Fact | Erster Satz |
| Doc | Erste Überschrift oder erste 24 Token |
| ToolResult | Tool-Name + Status |
| Code | Funktions-/Klassen-Signatur |
| Summary | Erster Satz |
| Profile | Entity-Name + Typ |
| * (Fallback) | Erste 24 Token |

L1-Stufen sind **standardmäßig abstraktiv** (Work Queue) mit Fallback auf extraktiv (erste 200 Token), falls kein Worker verfügbar.

### 4.4 Zoom-Level-Fehlverhalten

Fehlt eine angeforderte Stufe, gilt:
1. Nächstgröbere verfügbare Stufe verwenden
2. Wenn keine gröbere verfügbar: nächstfeinere verwenden
3. Wenn keine verfügbar: `raw` (L0) verwenden, ggf. auf Budget-Grenze trunkiert

---

## 5. Token-sparsame IDs (Handles)

### 5.1 Handle-Format

```rust
/// Kurzer Alias einer Cell (4–6 Zeichen, Crockford-Base32).
/// Global pro Repository eindeutig und stabil über Branches.
pub struct Handle {
    pub prefix: char,           // immer 'c' für Cell-Handles
    pub code: [u8; 4],          // 4 Bytes → 6–7 Crockford-Base32-Zeichen
}
// Darstellung: "c:k7qd", "c:m2xa"
```

### 5.2 Handle-Vergabe

- **Globaler Zähler** im Manifest (nicht im RootSet, da branchübergreifend). Atomares Inkrement pro neuer Cell.
- **Crockford-Base32-Encoding** des Zählers (keine verwechselbaren Zeichen: kein I/L/O/U).
- **Länge wächst natürlich:** 4 Zeichen für die ersten ~1M Cells, 5 für ~33M, 6 für ~1G.
- **Keine Wiederverwendung:** Gelöschte Cells behalten ihren Handle als Tombstone.

### 5.3 Handle-Tabelle

Die Alias-Tabelle (`aliases` im `RootSet`) ist ein Prolly-Tree mit Schlüssel `Handle → CellId`. Sie ist content-addressed und wird bei Forks strukturell geteilt.

**Merge-Strategie:** Union (beide Seiten behalten ihre neu vergebenen Handles). Kollisionsfreiheit ist durch den globalen Zähler garantiert.

### 5.4 Fuzzy-Handle-Matching

Bei unbekanntem Handle berechnet die Engine Edit-Distanz (Damerau-Levenshtein, max 2) gegen alle aktiven Handles und liefert bis zu 3 Vorschläge:

```json
{
  "error": "unknown_handle",
  "did_you_mean": ["c:k7qd", "c:k7qb"]
}
```

---

## 6. Bitemporalität

### 6.1 Zeitmodell

```rust
pub struct BiTemporal {
    pub ts_tx: Hlc,                      // Hybrid Logical Clock — Transaktionszeit
    pub valid_from: Option<Timestamp>,   // None = seit Erstellung
    pub valid_to: Option<Timestamp>,     // None = aktuell gültig
}

/// Hybrid Logical Clock: 64-bit.
/// Bits 63–16: physische Zeit (Millisekunden seit Unix-Epoch, 48 Bit → reicht bis Jahr 10889)
/// Bits 15–0:  logischer Zähler (16 Bit → 65535 Events pro Millisekunde)
pub struct Hlc(u64);

/// Millisekunden seit Unix-Epoch.
pub struct Timestamp(i64);
```

### 6.2 HLC-Regeln

1. **Send/Local:** `hlc = max(hlc, now()) + 1` (logischer Zähler inkrementiert)
2. **Receive:** `hlc = max(hlc, msg.hlc, now()) + 1`
3. **NTP-Drift-Toleranz:** 500 ms. Bei größerer Drift: Warnung loggen, physische Zeit verwenden, logischen Zähler auf 0 zurücksetzen.
4. **Overflow:** Bei logischem Zähler = 65535: auf nächste Millisekunde warten (busy-wait mit yield, max 2 ms).

### 6.3 Temporale Semantik

- Kante `supersedes` verknüpft ersetzte Fakten.
- **Default:** `recall(valid_at = now)` blendet überholte Fakten aus.
- **History:** `include_history = true` zeigt auch ersetzte Fakten (markiert).
- **Zeitreise:** `as_of = commit | timestamp` rekonstruiert den Zustand zu einem Zeitpunkt.

---

## 7. Strukturbewusstes Chunking

### 7.1 Chunker-Trait

```rust
pub trait Chunker: Send + Sync {
    /// Zerlegt ein Dokument in Cells mit Kanten und Zoom-Stufen.
    fn chunk(&self, input: &ChunkInput) -> Result<Vec<ChunkOutput>>;

    /// Eindeutiger Bezeichner dieses Chunkers (für Versionierung).
    fn id(&self) -> &str;
}

pub struct ChunkInput {
    pub content: Bytes,
    pub mime: MimeType,
    pub target_tokens: u32,         // Zielgröße pro Chunk in Token
    pub overlap_tokens: u32,        // Überlappung zwischen Chunks
    pub tokenizer: TokenizerId,     // Für Token-Zählung
}

pub struct ChunkOutput {
    pub content: Bytes,
    pub kind: CellKind,
    pub attrs: Attrs,
    pub edges: Vec<Edge>,           // PartOf, Follows, Contains
    pub zoom_l2: Option<String>,    // Extraktiver Titel
    pub boundary_safe: bool,        // Token-Grenzen stabil?
}
```

### 7.2 Standard-Chunker

| Chunker-ID | Ziel | Grenzen | boundary_safe |
|---|---|---|---|
| `markdown-v2` | Prosa/Markdown | Überschriften > Absätze > Sätze | ja (Satzgrenzen) |
| `code-treesitter` | Quellcode | Funktion/Klasse/Modul (tree-sitter AST) | ja (Statement-Grenzen) |
| `json-path` | JSON/JSONL | Top-Level-Objekte oder konfigurierbare Pfade | ja |
| `table-row` | CSV/TSV/XLSX | Zeilen mit Header-Kontext | ja |
| `html-semantic` | HTML | Semantische Blöcke (article, section, p) | ja |
| `binary-fastcdc` | Binärdaten | Content-Defined Chunking (FastCDC, 8 KiB Ziel) | nein |
| `passthrough` | Bereits atomare Einheiten | Keine Zerlegung | ja |

### 7.3 Token-Grenzen und boundary_safe

Persistierte Token-IDs sparen Re-Tokenisierung, **aber** Token-Grenzen sind kontextabhängig.

- Cells tragen `boundary_safe` (Chunker schneidet an stabilen Stellen).
- Nur bei `boundary_safe` darf die Context Engine Token-IDs direkt konkatenieren.
- Bei `!boundary_safe`: Text konkatenieren und neu tokenisieren.
- Token-Zählungen werden mit Sicherheitsmarge kalkuliert: `tok_count + ceil(tok_count * 0.02)` (2 % Toleranz pro Cell-Grenze).

---

## 8. Sequenzen

### 8.1 Sequenz-Modell

Sequenzen sind **geordnete Folgen von Cells** (Episode → Turns, Dokument → Abschnitte). Sie werden über den augmentierten Token-Summenbaum navigiert (siehe [SPEC-05](SPEC-05-INDEXES.md)).

```rust
/// Sequenz-Identifikator.
pub struct SeqId {
    pub ns: NsId,
    pub key: CellKey,       // Die Sequenz selbst ist eine Cell (CellKind::Doc oder Episode)
}

/// Sequenzmitgliedschaft als Prolly-Tree-Schlüssel.
/// Format: (SeqId, position: u32) → CellId
/// position ist monoton steigend, nicht notwendig lückenlos.
pub struct SeqEntry {
    pub seq: SeqId,
    pub position: u32,
    pub cell: CellId,
}
```

### 8.2 Sequenz-Operationen

- **Append:** `position = last_position + 1000` (Lücken für späteres Einfügen)
- **Insert:** `position = (prev + next) / 2` (ggf. Renumbering bei Erschöpfung)
- **Window:** O(log n) über Token-Summenbaum (siehe [SPEC-05](SPEC-05-INDEXES.md), Abschnitt 5)
- **Renumbering:** Hintergrund-Job, wenn Positionslücken < 1 werden

---

## 9. Veränderliche Sidecar-Daten

**Veränderliche Zähler sind NICHT Teil der Cell** (sonst würde jeder Lesezugriff eine neue Version erzeugen).

```rust
/// Sidecar-Tabelle für Zugriffsstatistiken.
/// Eventual-consistent, nicht im ChunkId-Hash, nicht im Commit.
pub struct AccessStats {
    pub cell: CellId,
    pub read_count: u32,            // Gesamtzugriffe
    pub expand_count: u32,          // Wie oft L0 angefordert
    pub last_read: Timestamp,       // Letzter Lesezugriff
    pub last_expand: Timestamp,     // Letzter Expand-Zugriff
    pub salience: f32,              // Berechnete Wichtigkeit (Cache)
}
```

### 9.1 Konsistenzmodell

**Entscheidung (C-19): Eventual Consistency.** Zugriffsstatistiken sind Soft-Signale für Ranking und Layout-Optimierung. Exakte Werte sind nicht nötig.

- **Sampling:** 10 % der Lesezugriffe werden aufgezeichnet (konfigurierbar).
- **Aggregation:** Zähler werden in-memory gesammelt und periodisch (alle 10 s) in die Sidecar-Tabelle geschrieben.
- **Merge:** Sum-CRDT-Semantik über Branches.
- **Persistence:** Sidecar-Tabelle wird im Manifest (nicht im WAL) gespeichert. Verlust bei Crash ist akzeptabel.

### 9.2 Salience-Score

```
salience = w_recency · recency(last_read)
         + w_access  · log(1 + read_count)
         + w_expand  · expand_rate
         + w_graph   · graph_centrality
         + w_pin     · is_pinned
```

Gewichte je Namespace-Kind konfigurierbar. Defaults:

| Gewicht | episodic | semantic | procedural | working |
|---|---|---|---|---|
| w_recency | 0.4 | 0.1 | 0.05 | 0.5 |
| w_access | 0.2 | 0.3 | 0.3 | 0.2 |
| w_expand | 0.2 | 0.3 | 0.3 | 0.1 |
| w_graph | 0.1 | 0.2 | 0.3 | 0.1 |
| w_pin | 0.1 | 0.1 | 0.05 | 0.1 |

---

*Ende SPEC-02. Weiter: [SPEC-03 Storage Engine](SPEC-03-STORAGE-ENGINE.md)*

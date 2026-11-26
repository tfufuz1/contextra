# SPEC-02 — Datenmodell: Entkoppelte Cells, Facetten, Zoom, Bitemporalität

**Version 2.1 · November 2026 · Status: Final**
**Querverweise:** [Index](SPEC-00-INDEX.md) · [Architektur](SPEC-01-ARCHITECTURE.md) · [Storage](SPEC-03-STORAGE-ENGINE.md) · [Versionierung](SPEC-04-VERSIONING.md)

---

## 1. Namespace und Katalog

Ein **Namespace** ist die organisatorische und Schema-Einheit (vergleichbar mit Collection/Schema-Bereich). Namespace-Policies, Attribut-Schemas, Embedding-Spaces, Tokenizer-Registry, Analyzer-Profile und ACL-Gruppen bilden zusammen den **Katalog**, der als Prolly-Tree (`policies` im `RootSet`) **versioniert** wird (Zeitreise gilt auch für Schemata).

### 1.1 Namespace-Identität

```rust
/// Sequentieller Namespace-Identifikator innerhalb eines Repositories.
pub struct NsId(pub u32);

/// Hierarchischer Pfad, z. B. "agent/episodic", "wissen/handbuecher".
pub struct NsPath(pub Arc<str>);
```

**Reservierte Präfixe:**
- `_system/` — schreibgeschützte oder Katalog-interne System-Metadaten (z. B. `_system/catalog`).
  > **Wichtig:** Ephemerer Laufzeitzustand (`Work Queue` inkl. Leases/Attempts und das `Idempotenz-Journal`) ist **KEIN** versionierter Namespace und liegt **nicht** im `RootSet`, sondern in dedizierten Kernel-Sidecar-Tabellen (`sidecar/work_queue.cxs`, `sidecar/idempotency.cxs`; siehe [SPEC-04](SPEC-04-VERSIONING.md) §8 und [SPEC-06](SPEC-06-CONTEXT-ENGINE.md) §7.2).

### 1.2 Namespace-Klassen (`MemoryClass`)

| Class | Semantik | Typischer Inhalt | Standard-Retention & Decay |
|---|---|---|---|
| `episodic` | Zeitlich geordnete Erlebnisse | Konversationsturns, Tool-Ergebnisse, Beobachtungen | `hot_days = 14`, `warm_days = 180`, `zoom_down_after = 180d`, `decay = 30d` |
| `semantic` | Extrahierte Fakten & Wissen | Claims `(Subjekt, Prädikat, Objekt)`, Entity-Profile, Definitionen | `history_days = 365`, `decay = 180d` |
| `procedural` | Handlungswissen | Skills, Runbooks, Workflows, Rezepte | unbegrenzt, `decay = none` |
| `working` | Kurzlebiges Arbeitsgedächtnis | Scratchpad, Hypothesen, Session-Pläne | `ttl = 24h` (Ablauf erzeugt expliziten Retire-Commit) |
| `artifact` | Erzeugte Artefakte | Code, Reports, Dokumente | unbegrenzt, `decay = 730d` |
| `reference` | Externe Referenzmaterialien | Handbücher, API-Docs, Richtlinien | unbegrenzt, `zoom_down_after = 365d`, `decay = 730d` |

### 1.3 Namespace-Policy (vollständiges Schema)

```toml
[namespace."agent/episodic"]
class                 = "episodic"
chunker               = "markdown-v2"
tokenizers            = ["cl100k", "o200k"]
embedders             = ["emb-768-v3"]         # Registrierte EmbeddingSpace-IDs
active_embedder       = "emb-768-v3"           # Primärer Space für neue Writes & Default-Queries
zoom_ladder           = ["L0:raw", "L1:summary<=200tok", "L2:title<=24tok"]
default_trust         = "tool-unverified"      # system | curated | tool-verified | tool-unverified | agent-inferred | untrusted
min_main_trust        = "tool-verified"        # Schwelle für Gate G2 bei Promotion nach main
acl_default           = "grp:default"
functional_predicates = ["current_status", "owner", "endpoint_url"]
max_cell_size         = "256KiB"
max_zoom_levels       = 5
decay                 = { kind = "exponential", half_life_days = 30, floor = 0.1 }
retention             = { history_days = 180, branches_days = 7, hot_days = 14, warm_days = 180, zoom_down_after = "180d" }
merge                 = { text = "diff3", attrs = "field", counters = "sum", edges = "add-wins", default = "manual" }
durability            = "durable"              # volatile | buffered | durable | strict | replicated
sequence_enabled      = true
```

### 1.4 Namespace-Lifecycle & Mandantengrenze

- **Mandantenisolation:** Ein Repository/Store (`<repo-root>/`) bildet die **harte** Isolationsgrenze (eigener `repo_seed`, eigene Schlüssel, eigener WAL, eigener Derivation Cache). Innerhalb eines Stores trennen Namespaces + ACL-Bitmaps Rollen und Teams.
- **Lifecycle:** `active → read_only → archived → deleted` (jeder Übergang ist ein Katalog-Commit; `archived → read_only → active` verlustfrei umkehrbar; `deleted` markiert als Tombstone für Retention-GC).

---

## 2. Cell — die entkoppelte Wissensprimitive

### 2.1 Identitäts- und Adressierungsmodell (löst Reibungspunkt 3 & Bitmap-Lücke)

Jede Cell besitzt vier klar getrennte Identitäten:

```rust
/// 1. Logische, branch- und versionsübergreifend stabile Identität.
pub struct CellId {
    pub ns: NsId,
    pub key: CellKey,
}

/// 16-Byte Schlüssel: Standard = 6B Timestamp-ms + 10B CSPRNG (ULID-kompatibel),
/// oder rein zufällig 16B im Confidential-Metadata-Modus, oder anwendungsdefiniert.
pub struct CellKey(pub [u8; 16]);

/// Optionaler externer Anwendungsschlüssel (max. 512 B pro Namespace eindeutig,
/// z. B. "sharepoint:/docs/arch.md#chunk-04"). Abbildung im Katalog.
pub struct ExternalKey(pub CompactString);

/// 2. Globaler dichter Index-Adressraum (u64, monoton pro Repo, NIE wiederverwendet).
/// Ermöglicht kompakte Roaring64-Bitmaps (ACL, Filter, Graph-Frontier) und
/// schnelle Range-Slice-Übersetzung in segmentlokale u32-DocIds.
pub struct Gid(pub u64);

/// 3. Content-addressed Hash (BLAKE3-256).
/// Bei verschlüsselten Repos: BLAKE3::keyed_hash(repo_hash_key, canonical_plaintext_bytes).
/// Wichtig: Gehasht wird IMMER der kanonische Klartext, NIE der Nonce-Ciphertext!
pub struct ChunkId(pub [u8; 32]);
pub type ContentHash = ChunkId;
```

**Kern-Invariante:**
- `CellId` und `Gid` bleiben über **alle** Versionen einer Cell stabil.
- `content_hash` (`ChunkId` des `CellContent`) ändert sich **ausschließlich**, wenn sich der unveränderliche Inhalt ändert.
- `meta_hash` (`ChunkId` der `CellMeta`) ändert sich bei Attribut-, Trust-, Gültigkeits- oder Belief-Updates, **ohne** `content_hash` oder davon abhängige Derivate zu invalidieren!

### 2.2 Entkoppelte Cell-Struktur

Um die Granularitäts-Lücke zwischen Content-Addressing und häufigen Metadaten-Mutationen (Tagging, `valid_to`-Schließung bei Fakt-Supersession, Trust-Promotion) zu schließen, wird jede Cell in **drei getrennte truth-layer Strukturen** zerlegt:

```rust
/// Eintrag im `primary`-Prolly-Tree (Schlüssel: CellId). Leichtgewichtig (~74 Bytes).
pub struct CellRecord {
    pub gid: Gid,
    pub content_hash: ChunkId,          // Hash des unveränderlichen CellContent
    pub meta_hash: ChunkId,             // Hash der versionierten CellMeta
    pub flags: CellFlags,               // TOMBSTONE | ARCHIVED | PINNED | STALE
}

/// Teil A: Unveränderlicher Inhaltsteil (content_hash = BLAKE3(canonical(CellContent))).
/// Grundlage für Derivation Cache, Embeddings, Token-Zählungen und Deduplizierung.
pub struct CellContent {
    pub raw: RawPayload,                // Inline (≤ 2 KiB) oder BlobRef (ChunkId)
    pub mime: CompactString,
    pub boundary_safe: bool,            // Token-Konkatenation an Chunk-Grenzen sicher?
    pub chunker_id: Option<CompactString>,
    pub origin_prov: OriginProvenance,  // Unveränderliche Erstellungs-Quelle (Ingest/Model/SourceSpan)
}

/// Teil B: Versionierter Metadatenteil (meta_hash = BLAKE3(canonical(CellMeta))).
/// Änderungen (Tags, valid_to, Trust, Confidence) erzeugen nur ein kleines neues CellMeta;
/// raw, Embeddings, Zoom-L1/L2 und Token-Counts bleiben unverändert und gültig!
pub struct CellMeta {
    pub id: CellId,
    pub gid: Gid,
    pub kind: CellKind,
    pub parent: Option<Gid>,            // Hierarchie: Document → Section → Chunk
    pub level: u8,                      // 0 = Rohchunk, 1 = Abschnitt, 2 = Dokument, 3 = Thema
    pub attrs: Attrs,                   // Typisierte Attribute (tags, title, custom...)
    pub time: BiTemporal,               // ts_tx (HLC) + valid_from / valid_to
    pub trust: TrustTier,               // Vertrauensstufe
    pub taint: TaintFlags,              // Orthogonale Kontaminations-Flags
    pub belief: BeliefBeta,             // Konfidenz als Beta(alpha, beta)
    pub grounding: GroundingLevel,      // G0..G3 Belegstufe
    pub acl: AclRef,                    // Gruppen-/Prinzipal-ACL
    pub prov_id: ProvId,                // Zeiger in append-only Provenance-Tree
}
```

**Teil C (Kanten):** Kanten (`Edge`) liegen **nicht** in `CellContent` oder `CellMeta`, sondern im eigenen **`graph`-Prolly-Tree** (`G:(src_gid, edge_kind, dst_gid, edge_id) → EdgePayload`, plus Reverse `R:`) — siehe [SPEC-05](SPEC-05-INDEXES.md) §4.1! Das Hinzufügen oder Entfernen einer Kante (z. B. `Supersedes`, `Supports`) verändert weder `content_hash` noch `meta_hash`!

**Claim-Modell für `CellKind::Fact`:** Ein Fakt kann strukturierte Claim-Felder in `attrs` tragen (`subject_gid`, `predicate`, `object_ref | literal`, `polarity: Positive | Negative`). Funktionale Prädikate (`NamespacePolicy.functional_predicates`) erlauben zu jedem `valid_at`-Zeitpunkt höchstens ein gültiges Objekt pro `(subject, predicate)`.

### 2.3 CellKind (exhaustive, `#[non_exhaustive]` für Erweiterung)

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, rkyv::Archive)]
#[repr(u8)]
pub enum CellKind {
    Episode       = 0,  // Konversationsturn (User, Assistant, Tool)
    Fact          = 1,  // Strukturierter Fakt / Claim
    Doc           = 2,  // Dokument oder Abschnitt (Hierarchie-Knoten)
    Chunk         = 3,  // Strukturiertes Dokument-Fragment
    ToolResult    = 4,  // Ergebnis eines Tool-Aufrufs (inkl. input/output hash)
    Plan          = 5,  // Agenten-Plan oder -Schritt
    Skill         = 6,  // Handlungswissen / Runbook
    Summary       = 7,  // Abgeleitete Zusammenfassung (mit source_hashes)
    Profile       = 8,  // Aggregiertes Entity-Profil
    Code          = 9,  // Code-Artefakt (AST-gechunkt)
    Config        = 10, // Konfiguration / Policy
    Annotation    = 11, // Kommentar / Review-Notiz
    Hypothesis    = 12, // Unbelegte Spekulation (G3, Quarantäne)
    Observation   = 13, // Sensor-/Log-/Systembeobachtung
    Conflict      = 14, // Repräsentation eines Merge-/Wissenskonflikts
    Unknown(u8)   = 255 // Vorwärtskompatibilitäts-Fallback (15..254 reserviert)
}
```

### 2.4 Typisierte Attribute (`Attrs`)

```rust
pub struct Attrs {
    entries: SmallVec<[(AttrKey, AttrValue); 8]>, // kanonisch sortiert nach AttrKey, dedupliziert
}

#[derive(Clone, PartialEq, rkyv::Archive)]
pub enum AttrValue {
    Str(CompactString),                  // UTF-8, max 1 KiB
    Int(i64),
    Float(f64),                          // NaN normalisiert beim Hashen auf kanonisches Bitmuster
    Bool(bool),
    Timestamp(i64),                      // ms seit Unix-Epoch
    Duration(u64),                       // ms
    Tags(BTreeSet<CompactString>),
    Enum(u16),                           // Index in Namespace-Schema-Wertemenge
    GidRef(Gid),                         // Verweis auf Entity/Subjekt (für Claims)
    Bytes(SmallVec<[u8; 32]>),           // max 4 KiB
}
```

**Standardattribute** (immer registriert): `title` (`Str`), `heading_path` (`Str`), `lang` (`Str`), `mime` (`Str`), `size_bytes` (`Int`), `tags` (`Tags`), `importance` (`Float`), `pinned` (`Bool`), `subject` (`GidRef`), `predicate` (`Str`), `expires_at` (`Timestamp` für `working`).

### 2.5 Typisierte Kanten (`Edge`)

```rust
pub struct Edge {
    pub id: u64,                         // eindeutige EdgeId
    pub kind: EdgeKind,
    pub src: Gid,
    pub dst: Gid,
    pub weight: f32,                     // Default 1.0
    pub valid: Interval<Hlc>,            // bitemporale Kanten-Gültigkeit
    pub trust: TrustTier,
    pub belief: BeliefBeta,
    pub prov_id: ProvId,
    pub attrs: SmallAttrs,               // max 4 Attribute
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, rkyv::Archive)]
#[repr(u8)]
pub enum EdgeKind {
    PartOf      = 0,  Follows    = 1,  Precedes    = 2,  Contains   = 3,
    Cites       = 10, DerivedFrom= 11, Supersedes  = 12, Contradicts= 13,
    Supports    = 14, RelatedTo  = 15, Summarizes  = 16, DuplicateOf= 17,
    CreatedBy   = 20, UsedIn     = 21, Annotates   = 22,
    About       = 30, HasProperty= 31, Role        = 32, // für n-äre reifizierte Facts
}
```

### 2.6 Provenienz, Grounding und Konfidenzmodell (`BeliefBeta`)

Provenienz ist **Pflichtfeld** beim Commit (Commit-Validierung lehnt Schreibvorgänge ohne gültige Provenienz ab, außer explizit als `CellKind::Hypothesis` im Quarantäne-Pfad markiert).

```rust
/// Append-only Provenance-Eintrag im `provenance`-Prolly-Tree.
pub struct ProvRecord {
    pub id: ProvId,                      // (subject_gid, ev_seq: u32)
    pub activity: ActivityKind,          // Ingest | Extract | Infer | Summarize | HumanEdit | Merge
    pub origin: Origin,                  // Ingest{parser} | LlmExtraction{model, prompt_hash}
                                         // | ToolCall{tool, input_hash, output_hash}
                                         // | Human{principal} | Derivation{producer, sources: Vec<Gid>}
    pub source_uri: Option<CompactString>,
    pub source_hash: Option<ChunkId>,    // Hash des unveränderten Quelldokuments
    pub source_span: Option<(u64, u64)>, // Byte-Spanne im Quelldokument
    pub quote_hash: Option<[u8; 16]>,    // BLAKE3-128 der normalisierten Zitat-Spanne
    pub source_root: [u8; 16],           // Wurzel-Hash der Ableitungskette (gegen Echokammern!)
    pub extractor_conf: f32,             // 0.0..1.0
    pub observed_at: Hlc,
    pub run_id: Option<RunId>,
}

/// Beta-Verteilung für Unsicherheit und Evidenz-Akkumulation.
pub struct BeliefBeta {
    pub alpha: f32,                      // bestätigende Evidenzgewichtssumme + Prior
    pub beta: f32,                       // widersprechende Evidenzgewichtssumme + Prior
    pub source_roots: SmallVec<[[u8; 16]; 4]>, // deduplizierte unabhängige Wurzeln
}
```

**Normative Konfidenz-Regeln:**
1. **Priors:** `System`/`Verified` = `Beta(4, 1)`, `Curated`/`ToolVerified` = `Beta(2, 1)`, `ToolUnverified`/`Unknown` = `Beta(1, 1)`, `AgentInferred`/`Hypothesis` = `Beta(1, 2)`.
2. **Unabhängigkeitsgebot (`source_root`):** Eine Bestätigung erhöht `alpha` (`+= w * c`) **nur dann**, wenn `prov.source_root ∉ belief.source_roots`. Zitiert ein Agent seine eigene frühere Ausgabe erneut, wächst `alpha` nicht (verhindert Echokammern).
3. **Ableitung (Noisy-AND):** Abgeleitete Fakten erben `lcb(derived) = min(lcb(parent_i)) * extractor_conf`.
4. **Planer-Score:** Der Context Planner rechnet mit der **unteren 10-%-Konfidenzgrenze (`lcb_10`)** der Beta-Verteilung, nicht mit dem Erwartungswert — wenig belegte Aussagen werden automatisch konservativ gewichtet.

**Grounding-Stufen (`GroundingLevel`):**
- **`G0` (`VerifiedSpan`):** Quelle vorhanden, `source_span` / `quote_hash` verifiziert.
- **`G1` (`UnverifiedSource`):** Quelle/URI vorhanden, Spanne nicht maschinell geprüft.
- **`G2` (`Inferred`):** Agenteninferenz mit Eltern-`Gid`s und Noisy-AND-Konfidenz.
- **`G3` (`Hypothesis`):** Keine Quelle; automatisch in **Quarantäne** (im Standard-Recall ausgeschlossen, nur mit `include_hypotheses = true` sichtbar).

---

## 3. Facetten (Lanes) und Hot/Cold-Aufteilung

Jede Cell wird physisch im Extent in spaltenorientierte **Lanes** zerlegt ([SPEC-03](SPEC-03-STORAGE-ENGINE.md) §2.2).

| Lane | Inhalt | Quelle (Truth / Cache) | Hot/Cold | Typische Größe |
|---|---|---|---|---|
| `meta` | `gid`, `CellId`, `kind`, `trust`, `taint`, `belief`, `valid_from/to`, `acl`, AttrBloom, Edge-Hint | `CellMeta` | **hot** | 64–512 B |
| `tokcount` | Exakte Token-Anzahl je registriertem `TokenizerId` (`u32`) | Derivation Cache `(content_hash, tok)` | **hot** | 4–16 B |
| `zoom.L2` | Extraktiver Titel / Gist (≤ 24 Tok) | Synchron bei Commit | **hot** | 20–100 B |
| `zoom.L1` | Abstraktive / extraktive Zusammenfassung (≤ 200 Tok) | Derivation Cache / Work Queue | **hot** | 100–800 B |
| `emb.q` | Quantisierter Embedding-Kopf (SQ8 / binär / Matryoshka-Präfix) | Derivation Cache `(content_hash, space)` | **hot** | 128–768 B |
| `raw` (= `L0`) | Kanonischer Rohinhalt (oder Inline-Slice für Cold-Tail) | `CellContent` | **cold** | 100 B – 256 KiB |
| `tok.ids` | Bitgepackte Token-IDs (optional) | Derivation Cache | **cold** | variabel |
| `emb.full` | Volles Embedding (`f16` / `f32`) für Re-Ranking | Derivation Cache | **cold** | 1.5–3 KiB |
| `attrs.ext` | Große/selten gefilterte `Attrs` & lange Strings | `CellMeta` | **cold** | variabel |

**Pflicht-Lanes:** `meta`, `raw` (oder Tombstone-Marker), `tokcount` (wenn `raw` nicht leer). Fehlende optionale Lanes werden als `None` behandelt, nie als Fehler.

---

## 4. Zoom-Leiter und Small-to-Big-Hierarchie

| Stufe | Name | Ziel-Token | Erzeugung |
|---|---|---|---|
| **L0** | `raw` | unbegrenzt (≤ `max_cell_size`) | Original-Chunk; Eltern-Knoten (`Doc`/`Section`) fassen Kinder per `Contains`/`Summarizes` zusammen (**Small-to-Big**) |
| **L1** | `summary` | ≤ 200 | Abstraktiv (Work Queue via `Llm`-Port) oder deterministisch extraktiv (Fallback: erste 200 Token an Satzgrenzen) |
| **L2** | `title` | ≤ 24 | **Synchron & deterministisch beim Commit** (Überschrift, AST-Signatur, erster Satz) |
| **L3** | `handle` | ~4 | Engine-intern (`c:<gid>-<mac2>`) |

**Lineage-Staleness (`derivation_hash`):** Jede `zoom.L1`/`L2` speichert `derived_from = content_hash` (bzw. bei `Summary`-Cells über Abschnitte `BLAKE3(sorted(child_content_hashes) || producer_id)`). Ändert sich ein Quell-`content_hash`, ist das Derivat ohne manuelle Invalidierung sofort als `stale` erkennbar.

---

## 5. MAC-geschützte, token-sparsame Handles (löst Reibungspunkt 4)

### 5.1 Handle-Format mit MAC-Schutz

Monoton hochgezählte Sequenz-Handles ohne Prüftag führen bei 1-Zeichen-Halluzinationen von LLMs oft auf *eine andere existierende Cell*, die dann fälschlich expandiert wird. Deshalb trägt jedes Handle ein kurzes kryptographisches MAC-Tag:

```rust
/// Externes Citation- und Expand-Handle.
/// Textform: "c:<seq_crockford>-<mac2>", z. B. "c:k7qd-3f"
pub struct Handle {
    pub gid: Gid,               // Monotoner globaler Cell-Index (Crockford-Base32, mind. 4 Zeichen)
    pub mac: [u8; 2],           // 2 Crockford-Base32-Zeichen (= 10 Bit MAC)
}
```

**Berechnung des 2-Zeichen-MAC-Tags:**
```rust
fn compute_handle_mac(repo_seed: &[u8; 32], gid: Gid) -> [char; 2] {
    let mac_key = hkdf_blake3(repo_seed, b"contextra-handle-mac-v1");
    let digest = blake3::keyed_hash(&mac_key, &gid.0.to_le_bytes());
    let bits10 = u16::from_le_bytes([digest.as_bytes()[0], digest.as_bytes()[1]]) & 0x03FF;
    // Kodiert 10 Bit als 2 Crockford-Base32-Zeichen (kleingeschrieben: 0-9, a-h, j, k, m, n, p-t, v-z)
    encode_crockford_10bit(bits10)
}
```

- **Kosten:** `"-3f"` verbraucht ca. 1–2 Subword-Token (~8 Bytes total).
- **Wirkung:** Ein zufällig geratenes oder um 1–2 Zeichen halluziniertes Handle (`c:k7qe-3f` oder `c:k7qd-9a`) schlägt in **99,90 % (1023/1024)** der Fälle bei der MAC-Prüfung fehl — die Engine löst niemals stillschweigend eine falsche Cell auf!

### 5.2 Vergabe und Alias-Tabelle (`RootSet.aliases`)

- **Globaler `Gid`-Zähler** im Manifest (atomar inkrementiert pro neuer Cell, nie wiederverwendet — auch nach `retire`/`purge` bleibt die `Gid` verbrannt).
- **Alias-Tree:** `aliases` im `RootSet` speichert die bidirektionale Zuordnung `CellId ↔ Gid` (Merge-Strategie: `Union` — da `Gid`s global eindeutig vergeben werden, sind Merges konfliktfrei).

### 5.3 Verifikation und Fuzzy-Handle-Matching

Beim Aufruf von `expand("c:k7qe-3f")` oder `VERIFY`:
1. Syntax parsen (`c:<seq>-<mac2>`).
2. `mac == compute_handle_mac(repo_seed, seq)` prüfen.
   - **MAC ungültig oder `Gid` existiert auf dem Snapshot nicht:**
     Suche via Damerau-Levenshtein (Edit-Distanz ≤ 2) über den Sequenzteil aller im Snapshot aktiven `Gid`s nach bis zu **3 Kandidaten** und gib Fehler `unknown_handle` (bzw. `invalid_handle_mac`) inkl. der korrekten MAC-Handles zurück:
     ```json
     {
       "error": "unknown_handle",
       "message": "Handle 'c:k7qe-3f' failed verification or does not exist on branch 'main'.",
       "retryable": false,
       "did_you_mean": ["c:k7qd-3f", "c:k7qb-8m"]
     }
     ```
     *(Hinweis INV-5: Nur Handles von Cells zurückgeben, für die der Aufrufer die ACL-Berechtigung besitzt!)*

---

## 6. Bitemporalität und HLC

### 6.1 Zeitmodell

```rust
pub struct BiTemporal {
    pub ts_tx: Hlc,                     // Transaktionszeit (Commit-Zeitpunkt, unveränderlich)
    pub valid_from: Option<Timestamp>,  // Realwelt-Beginn (None = seit Erstellung)
    pub valid_to: Option<Timestamp>,    // Realwelt-Ende (None = aktuell gültig)
}

/// 64-bit Hybrid Logical Clock: Bits 63..16 ms seit Epoch (48b), Bits 15..0 logisch (16b).
pub struct Hlc(pub u64);
```

### 6.2 HLC- & Bitemporal-Regeln

1. **HLC:** `Send/Local`: `hlc = max(hlc, clock.now_ms() << 16) + 1`; NTP-Drift > 500 ms loggt Warnung; Zähler-Überlauf (> 65535 Events/ms) wartet max. 2 ms. Maßgeblich für totale Commit-Ordnung ist die globale **`Seq(u64)`**; `Hlc` ordnet Zeitstempelabfragen (`AS OF TIMESTAMP`).
2. **Funktionale Prädikate & Supersession:** Wird ein neuer `Fact` für `(subject_gid, functional_predicate)` mit `new.valid_from > old.valid_from` und Policy `SupersedeOnNewer` geschrieben, schließt die Engine atomar `old.meta.time.valid_to = Some(new.valid_from)` (schreibt nur `CellMeta`, `CellContent`/`raw` bleibt unberührt!) und legt eine `Supersedes`-Kante an. Gleichrangige Widersprüche erzeugen ein `ConflictCell` (kein stilles Überschreiben).
3. **Abfrage-Defaults:** `VALID AT t` verwendet Standard `t = query_ctx.now` (**Fail-Open** falls `valid_from/to = None`, aber **Fail-Closed** für ACL/Trust!). `include_history = true` zeigt auch ersetzte Fakten (markiert). Kombination `AS OF TX t1 VALID AT t2` ist voll unterstützt.

---

## 7. Strukturbewusstes Chunking und Sequenzen

### 7.1 Chunker

Chunker schneiden anhand von Struktur und **Token-Zielgrößen** (Default 512 Token, min 50, max 1024) statt fester Byte-Grenzen, speichern `heading_path` sowie `chunker_id` und verknüpfen Chunks über `Contains` / `PartOf` / `Follows`-Kanten mit ihrem Eltern-`Doc`/`Section`.

| Chunker-ID | Ziel | Grenzen | `boundary_safe` |
|---|---|---|---|
| `markdown-v2` | Prosa/Markdown | Überschriften > Absätze > Sätze | ja |
| `code-treesitter` | Quellcode | Funktion/Klasse/Modul (AST), Signatur als L2 | ja |
| `json-path` / `table-row` | JSON/CSV/XLSX | Objekte / Zeilengruppen inkl. Header | ja |
| `html-semantic` / `pdf-layout` | HTML / PDF | Semantische Blöcke, Spaltenbereinigung | ja |
| `binary-fastcdc` | Binärdaten | Content-Defined Chunking (8 KiB Ziel) | nein |
| `passthrough` | Atomare Einheiten | Keine Zerlegung | ja |

Token-Kalkulation im Packer rechnet mit **2 % Sicherheitsmarge** (5 % falls Tokenizer im Katalog nicht registriert und geschätzt).

### 7.2 Sequenzen (Augmentierter Token-Summenbaum)

Geordnete Folgen (`Doc → Chunks`, `Episode → Turns`) werden als Prolly-Keyspace `(SeqId, position: u32) → (Gid, CellRecordRef)` mit `position += 1000`-Lücken gespeichert. Innere Knoten tragen die exakten Token-Summen pro `TokenizerId` — `window(seq, anchor, budget)` läuft in **O(log n)** ohne Inhaltszugriff ([SPEC-05](SPEC-05-INDEXES.md) §5.2).

---

## 8. Veränderliche Sidecar-Daten (`AccessStats`)

Zugriffsstatistiken sind **NICHT** Teil von `CellContent`, `CellMeta` oder `RootSet` (Lesezugriffe erzeugen niemals Commits):

```rust
/// Sidecar: sidecar/access_stats.cxs — eventual-consistent, nicht im WAL.
pub struct AccessStats {
    pub gid: Gid,
    pub read_count: u32,
    pub expand_count: u32,
    pub last_read: Timestamp,
    pub last_expand: Timestamp,
    pub salience: f32,
}
```

- **Sampling:** 10 % der Recalls, 100 % der Expands im RAM aggregiert, alle 10 s in Sidecar geflusht.
- **Salience-Formel:** `salience = w_recency·recency + w_access·log(1+read_count) + w_expand·expand_rate + w_graph·centrality + w_pin·is_pinned` (Salience steuert Prior-Boost und Archivierungsempfehlungen, löscht aber niemals selbst Daten).

---

*Ende SPEC-02. Weiter: [SPEC-03 Storage Engine](SPEC-03-STORAGE-ENGINE.md)*

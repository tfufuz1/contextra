# SPEC-02 — Datenmodell: Identität, kanonische Kodierung, Cells, Evidenz

**Version 3.0 · 26. November 2026 · Status: Final**
**Querverweise:** [Index](SPEC-00-INDEX.md) · [Architektur](SPEC-01-ARCHITECTURE.md) · [Storage](SPEC-03-STORAGE-ENGINE.md) · [Versionierung](SPEC-04-VERSIONING.md)

---

## 1. Identitäten

```rust
pub struct NsId(pub u32);        // globaler Allocator
pub struct BranchId(pub u32);    // globaler Allocator, nie wiederverwendet
pub struct Gid(pub u64);         // globaler Allocator, dichter Bitmap-Adressraum
pub struct EdgeId(pub u64);      // globaler Allocator
pub struct Seq(pub u64);         // globale Commit-Ordnung
pub struct Lsn(pub u64);         // WAL-Frame-Ordnung (≠ Seq)
pub struct Hlc(pub u64);         // 48 bit ms | 16 bit logisch
pub struct CellKey(pub [u8; 16]);
pub struct CellId { pub ns: NsId, pub key: CellKey }
pub struct ChunkId(pub [u8; 32]);
pub type  ContentHash = ChunkId;
pub struct ProvId(pub [u8; 16]); // inhaltsadressiert
pub struct AclRef(pub [u8; 16]); // inhaltsadressiert
pub struct ExternalKey(pub CompactString); // ≤ 512 B, eindeutig je Namespace
pub type  SeqId = Gid;           // Sequenz = Gid ihrer Eltern-Cell
```

### 1.1 Allocator-Regeln

- `Gid`, `EdgeId`, `NsId`, `BranchId` sind je Store monoton und werden **nie** wiederverwendet, auch nicht nach Abbruch, Rollback, `retire` oder `purge`. Lücken sind zulässig.
- Allocatoren vergeben Blöcke (Default 1 024). Jeder Commit-Frame trägt die höchsten verwendeten IDs; das Manifest persistiert die Hochwassermarke. Recovery setzt `next = max(Manifest-Marke, max(WAL)) + 1`. `volatile`-Branches reservieren ihren Block durabel im Voraus (Manifest), damit keine ID nach Neustart doppelt vergeben wird.
- IDs, die in zwei Branches **unabhängig** entstehen können, ohne zentralen Zähler, sind inhaltsadressiert (`ProvId`, `AclRef`). Damit ist Mengenvereinigung beim Merge kollisionsfrei.

### 1.2 `CellKey`

Default: 6 B Millisekunden + 10 B CSPRNG (ULID-kompatibel). Namespace-Option `confidential_ids = true`: 16 B Zufall. Anwendungsdefinierte Schlüssel sind zulässig.

---

## 2. Kanonische Binärkodierung (CBE) und Hashing

**Alles, was gehasht wird, MUSS in CBE kodiert werden.** Serialisierungsformate von Bibliotheken (`rkyv`, `serde`) sind für Hash-Eingaben unzulässig, weil ihr Layout an Bibliotheksversionen gebunden ist. `rkyv` ist nur für nicht gehashte, rekonstruierbare Strukturen erlaubt.

### 2.1 CBE-Regeln (`cbe_version = 1`)

| Typ | Kodierung |
|---|---|
| `u8…u64` in Werten | little-endian, feste Breite |
| Ganzzahlen in **Tree-Keys** | big-endian (Byte-Ordnung = numerische Ordnung); `i64` mit invertiertem Vorzeichenbit |
| Länge/Anzahl | minimales LEB128 (nicht-minimal ⇒ Dekodierfehler) |
| Bytes/String | Länge + Daten; Strings UTF-8, NFC |
| `Option<T>` | `0x00` oder `0x01 ‖ T` |
| Enum | `u16`-Tag + Felder; unbekannte Tags bleiben als `(tag, len, bytes)` erhalten |
| Struct | Felder in Deklarationsreihenfolge; Erweiterungen nur als sortierte TLV-Liste am Ende |
| Set/Map | nach kodiertem Schlüssel byteweise sortiert, duplikatfrei |
| `f32/f64` (nur `AttrValue`) | IEEE-754; jedes NaN ⇒ kanonisches Quiet-NaN; `-0.0` ⇒ `+0.0` |

Decoder MÜSSEN nicht-kanonische Eingaben ablehnen; `encode(decode(b)) == b` ist Fuzz-Invariante.

### 2.2 Objekt-Hashes mit Domänentrennung

```
preimage = "CXID" ‖ cbe_version:u16 ‖ kind:u16 ‖ len:u64 ‖ body
ChunkId  = BLAKE3(preimage)                         // unverschlüsselter Store
ChunkId  = BLAKE3_keyed(id_key, preimage)           // verschlüsselter Store
```

`id_key` ist ein eigener, beim Anlegen erzeugter Schlüssel, der **nicht** rotiert ([SPEC-03](SPEC-03-STORAGE-ENGINE.md) §8). Gehasht wird immer der unkomprimierte Klartext.

### 2.3 `content_hash`

```
content_hash = ChunkId(kind = CellContent, body = CBE(mime) ‖ CBE(raw_bytes))
```

Nur **MIME-Typ und kanonische Rohbytes** bestimmen die Identität. Chunker, Grenz-Eigenschaften und Herkunft gehören **nicht** dazu: Ein unveränderter Chunk einer neuen Dokumentrevision behält so seinen Hash und alle Derivate. Textinhalte werden beim Ingest auf NFC und `\n` normalisiert; Binärinhalte bleiben unverändert.

---

## 3. Cell

Eine Cell besteht aus vier getrennten Truth-Strukturen: **Record**, **Content**, **Meta** und **Beziehungen** (Kanten, Evidenz).

```rust
/// Baum `primary`, Key: CellId. 82 Byte Nutzdaten.
pub struct CellRecord {
    pub gid: Gid,                 // 8
    pub content_hash: ContentHash,// 32
    pub meta_hash: ChunkId,       // 32
    pub flags: CellFlags,         // 2: TOMBSTONE | ARCHIVED | PINNED | EXPUNGED
    pub born_seq: Seq,            // 8: Commit, der diese Version erzeugte
}

/// Unveränderlich.
pub struct CellContent {
    pub mime: CompactString,
    pub raw: RawPayload,          // ≤ 2 KiB inline, sonst LargeBlob
}

/// Versioniert. meta_hash = ChunkId(CellMeta).
pub struct CellMeta {
    pub id: CellId,
    pub gid: Gid,
    pub kind: CellKindCode,
    pub parent: Option<Gid>,      // Document → Section → Chunk
    pub level: u8,                // 0 Chunk, 1 Abschnitt, 2 Dokument, 3 Thema
    pub boundary_safe: bool,
    pub chunker_id: Option<CompactString>,
    pub attrs: Attrs,
    pub valid: ValidInterval,     // [valid_from, valid_to)
    pub trust: TrustTier,
    pub taint: TaintFlags,
    pub grounding: GroundingLevel,
    pub acl: AclRef,
    pub origin: ProvId,           // Erstellungsquelle
    pub claim: Option<Claim>,     // nur Fact
}
```

**Transaktionszeit** steht nicht in `CellMeta`. Sie ergibt sich als `ts_tx = commit(born_seq).hlc`. Damit sind `meta_hash` und `ChangeSet`-Digest unabhängig von Rebase und Commit-Zeitpunkt.

**Konfidenz** steht ebenfalls nicht in `CellMeta`; sie ist eine deterministische Funktion der Evidenzmenge (§6).

### 3.1 Änderungsmatrix

| Änderung | `content_hash` | `meta_hash` | `graph` | `provenance` | Re-Embedding |
|---|---|---|---|---|---|
| Rohinhalt | neu | ggf. | ggf. | neu | ja |
| Tags, Trust, ACL | gleich | neu | — | — | nein |
| `valid.to` schließen | gleich | neu | `Supersedes` | — | nein |
| Kante hinzufügen | gleich | gleich | neu | — | nein |
| Bestätigung/Widerspruch | gleich | gleich | ggf. | neu | nein |
| Kontextfeld der Einbettung (z. B. `heading_path`) | gleich | neu | — | — | ja, falls der Producer das Feld deklariert (§8) |

### 3.2 `CellKind`

```rust
pub struct CellKindCode(pub u8);
// 0 Episode, 1 Fact, 2 Doc, 3 Chunk, 4 ToolResult, 5 Plan, 6 Skill, 7 Summary,
// 8 Profile, 9 Code, 10 Config, 11 Annotation, 12 Hypothesis, 13 Observation,
// 14 Conflict, 15..=255 reserviert
```
Die öffentliche API bietet ein `#[non_exhaustive]`-Enum mit `Unknown(u8)`; auf Disk steht ausschließlich der Code, sodass unbekannte Werte verlustfrei erhalten bleiben.

### 3.3 Claims

```rust
pub struct Claim {
    pub subject: Gid,
    pub predicate: CompactString,
    pub object: ClaimObject,          // Ref(Gid) | Literal(AttrValue)
    pub polarity: Polarity,           // Positive | Negative
}
```

Für Prädikate in `functional_predicates` gilt je `(subject, predicate)` und Zeitpunkt höchstens ein gültiges positives Objekt (§7.3).

### 3.4 Attribute

`Attrs` ist eine nach `AttrKey` sortierte, duplikatfreie Liste; ≤ 128 Felder, ≤ 32 KiB. Werte: `Str` (≤ 1 KiB), `Int`, `Float`, `Bool`, `Timestamp`, `Duration`, `Tags` (Set), `Enum(u16)`, `GidRef`, `Bytes` (≤ 4 KiB), `Counter(i64)`. Standardattribute: `title`, `heading_path`, `lang`, `size_bytes`, `tags`, `importance`, `pinned`, `expires_at`.

---

## 4. Namespaces und Katalog

Der Katalog (Namespaces, Schemata, EmbeddingSpaces, Tokenizer, Analyzer, Producer, ACL-Ausdrücke, Gruppen) liegt im Baum `policies` und ist versioniert. Jeder Snapshot nutzt den Katalog seines `RootSet`.

> **Ausnahme Zugriffsentzug:** Gruppenmitgliedschaften und Capability-Widerrufe werden **immer gegen den aktuellen Stand** geprüft, nicht gegen den historischen Snapshot. Zeitreise darf entzogene Rechte nicht wiederherstellen.

### 4.1 Klassen und Defaults

| `MemoryClass` | Inhalt | Retention / Decay |
|---|---|---|
| `episodic` | Turns, Tool-Ergebnisse | `hot_days 14`, `warm_days 180`, `zoom_down 180d`, Halbwertszeit 30 d |
| `semantic` | Fakten, Profile | `history_days 365`, 180 d |
| `procedural` | Skills, Runbooks | unbegrenzt, kein Decay |
| `working` | Scratchpad, Hypothesen | `ttl 24h` ⇒ expliziter Retire-Commit |
| `artifact` | Code, Reports | unbegrenzt, 730 d |
| `reference` | Handbücher, Richtlinien | unbegrenzt, `zoom_down 365d`, 730 d |

Fehlt eine Angabe, gelten `history_days = 30`, `branches_days = 1`.

### 4.2 Policy-Schema

```toml
[namespace."agent/episodic"]
class                 = "episodic"
chunker               = "markdown-v2"
tokenizers            = ["cl100k", "o200k"]          # Aliase auf TokenizerId-Digests
embedders             = ["emb-768-v3"]
active_embedder       = "emb-768-v3"
zoom_ladder           = ["L0:raw", "L1:summary<=200tok", "L2:title<=24tok"]
default_trust         = "agent-inferred"
default_taint         = ["unverified-tool"]
min_main_trust        = "tool-verified"               # Gate G2
acl_default           = [["grp:default"]]             # CNF, siehe §9
functional_predicates = ["current_status", "owner", "endpoint_url"]
supersede_policy      = "supersede-on-newer"          # supersede-on-newer | conflict
max_cell_size         = "256KiB"
decay                 = { kind = "exponential", half_life_days = 30, floor = 0.6 }
retention             = { history_days = 180, branches_days = 7, hot_days = 14, warm_days = 180 }
merge                 = { text = "conflict", attrs = "field", counters = "sum", edges = "add-wins", default = "manual" }
durability            = "durable"
sequence_enabled      = true
idf_isolation         = "namespace"                   # namespace | acl-class
```

Lifecycle: `active ⇄ read_only ⇄ archived → deleted`; jeder Übergang ist ein Katalog-Commit. `_system/*` ist schreibgeschützt. Work Queue und Idempotenz-Journal sind **keine** Namespaces.

---

## 5. Kanten

```rust
pub struct Edge {
    pub id: EdgeId,
    pub kind: EdgeKind,
    pub src: Gid,
    pub dst: Gid,
    pub weight_q16: u32,              // Q16.16, Default 1.0
    pub valid: ValidInterval,         // Realweltzeit
    pub trust: TrustTier,
    pub taint: TaintFlags,
    pub origin: ProvId,
    pub attrs: SmallAttrs,            // ≤ 4
}
// EdgeKind: 0 PartOf, 1 Follows, 2 Precedes, 3 Contains, 10 Cites, 11 DerivedFrom,
// 12 Supersedes, 13 Contradicts, 14 Supports, 15 RelatedTo, 16 Summarizes,
// 17 DuplicateOf, 20 CreatedBy, 21 UsedIn, 22 Annotates, 30 About, 31 HasProperty, 32 Role
```

Tree-Keys im Baum `graph`: `G ‖ src ‖ kind ‖ dst ‖ id → Edge` und `R ‖ dst ‖ kind ‖ src ‖ id → ()`. Beide Einträge werden stets in derselben Transaktion geschrieben (Prüfung G1 und `ctx verify`). Warnschwelle 10⁴, hartes Limit 10⁶ aktive Kanten je Knoten.

---

## 6. Provenienz, Grounding, Konfidenz

### 6.1 Provenienz (Pflicht)

```rust
pub struct ProvRecord {                       // ProvId = trunc128(ChunkId(ProvRecord))
    pub subject: Gid,
    pub activity: ActivityKind,               // Ingest|Extract|Infer|Summarize|HumanEdit|Merge
    pub origin: Origin,                       // Ingest{parser} | LlmExtraction{model,prompt_hash}
                                              // | ToolCall{tool,input_hash,output_hash}
                                              // | Human{principal} | Derivation{producer,sources}
    pub source_uri: Option<CompactString>,
    pub source_hash: Option<ChunkId>,
    pub source_span: Option<(u64, u64)>,
    pub quote_hash: Option<[u8; 16]>,
    pub source_root: [u8; 16],
    pub extractor_conf_q16: u32,
    pub observed_at: Hlc,
    pub run_id: Option<RunId>,
}
```

Ein Schreibvorgang ohne gültige Provenienz wird mit `provenance_missing` abgelehnt, außer die Cell ist als `Hypothesis` (`GL3`) deklariert.

**`source_root` wird von der Engine berechnet, nie vom Client übernommen:**
- Primärquelle: `H("root" ‖ source_hash)`; ohne `source_hash`: `H("root" ‖ origin_identity)` mit `origin_identity` = Principal bzw. `(tool, output_hash)`.
- Ableitung mit genau einem unabhängigen Eltern-Root: Root wird **geerbt**.
- Ableitung aus mehreren Roots: `H("derived" ‖ sortierte Eltern-Roots)`; sie zählt **nicht** als zusätzliche Evidenz für eine Aussage, die bereits eine Teilmenge dieser Roots enthält.
- Reine Modellausgaben ohne Eltern: ein Root je `(principal, run_id)`.

Der Echokammer-Schutz ist heuristisch: Er erkennt Wiederverwendung innerhalb der aufgezeichneten Lineage, nicht extern kopierte Inhalte.

### 6.2 Evidenzmenge und `BeliefBeta`

Baum `provenance` enthält zwei Keyspaces:

- `P ‖ prov_id → ProvRecord`
- `E ‖ gid ‖ source_root ‖ polarity → Evidence { weight_q16, prov_id }`

Je Schlüssel gewinnt das größte Gewicht (Gleichstand: kleinere `prov_id`). Diese Regel ist kommutativ, assoziativ und idempotent; Merges sind damit konfliktfrei.

```
alpha = prior_a(trust_origin) + Σ weight(E[gid, *, Support])
beta  = prior_b(trust_origin) + Σ weight(E[gid, *, Contradict])
weight = w_activity · extractor_conf
lcb10  = Quantil_0.10( Beta(alpha, beta) )
```

| Ursprungs-Trust | Prior |
|---|---|
| `System`, `Verified` | Beta(4,1) |
| `Curated`, `ToolVerified` | Beta(2,1) |
| `AgentInferred`, `Hypothesis` | Beta(1,2) |
| `Untrusted` / unbekannt | Beta(1,1) |

Namespace-Override ist zulässig. Hinweis: Mit diesen Werten liegt der Prior für `Untrusted` im Mittel über dem für `AgentInferred`; die Rangordnung wird deshalb zusätzlich über `trust_factor` gesichert ([SPEC-06](SPEC-06-CONTEXT-ENGINE.md) §3.2).

**Ableitung (gedämpftes Minimum):** `lcb(derived) ≤ min(lcb(parent_i)) · extractor_conf`. Der angezeigte und gerankte Wert ist `min(lcb10, Ableitungsschranke)`.

`alpha`, `beta`, `lcb10` werden als Cache-Lane materialisiert und sind jederzeit neu berechenbar.

### 6.3 Grounding-Stufen

| Stufe | Name | Bedingung |
|---|---|---|
| `GL0` | `VerifiedSpan` | Quelle vorhanden, Spanne und `quote_hash` maschinell geprüft |
| `GL1` | `UnverifiedSource` | Quelle vorhanden, Spanne ungeprüft |
| `GL2` | `Inferred` | Inferenz mit Eltern-`Gid`s |
| `GL3` | `Hypothesis` | Keine Quelle; Quarantäne, nur mit `include_hypotheses` sichtbar |

Das Präfix `GL` ist von Merge-Gates (`G1–G7`) und Zeitreise-Granularitäten (`Z1–Z4`) getrennt.

---

## 7. Zeit, Sichtbarkeit, Supersession

### 7.1 HLC

`hlc' = max(hlc + 1, wall_ms << 16)`. Läuft der logische Anteil über, wandert der Übertrag in den Millisekundenanteil (Monotonie bleibt; keine Wartezeit auf dem WAL-Writer). Drift > 500 ms zur Wanduhr erzeugt eine Warnung. Totale Ordnung liefert ausschließlich `Seq`.

### 7.2 Gültigkeit

Halboffen: `[from, to)`; `None` = unbegrenzt. `VALID AT t` (Default `t = now` der Anfrage) schließt Cells mit fehlenden Grenzen ein. ACL und Trust sind dagegen fail-closed.

### 7.3 Funktionale Prädikate

Neuer Fakt `n`, überlappender gültiger Fakt `o` gleichen `(subject, predicate)` mit anderem Objekt, Policy `supersede-on-newer`:

| Fall | Wirkung |
|---|---|
| `n.from > o.from` und `n.trust ≥ o.trust` | `o.valid.to = n.from`; Kante `n Supersedes o` |
| `n.from < o.from` und `n.to` offen | `n.valid.to = o.from` (rückwirkende Einfügung) |
| gleiche `from`, oder `n.trust < o.trust`, oder Einschluss mit Aufteilungsbedarf | Strukturkonflikt: auf ungeschützten Branches `ConflictCell`; auf geschützten Branches Ablehnung mit `conflict` |

Ein Fakt niedrigeren Trusts schließt nie automatisch einen höher vertrauten Fakt.

---

## 8. Facetten, Zoom, Derivationen

### 8.1 Zoom-Leiter

| Stufe | Ziel | Erzeugung |
|---|---|---|
| L0 `raw` | ≤ `max_cell_size` | Original |
| L1 `summary` | ≤ 200 Tok | Abstraktiv per Work Queue; Fallback deterministisch extraktiv |
| L2 `title` | ≤ 24 Tok | Synchron und deterministisch beim Commit |
| L3 `handle` | wenige Tok | Aus `Gid` berechnet |

### 8.2 Derivation Cache

```
DerivationKey = (input_digest, producer_id)
input_digest  = H(content_hash ‖ CBE(deklarierte Kontextfelder))
producer_id   = H(Art ‖ Modell/Version ‖ Parameter ‖ Prompt-Hash)
```

Jeder Producer deklariert im Katalog, welche Metafelder in seine Eingabe eingehen (z. B. `title`, `heading_path`). Ohne Deklaration gilt `input_digest = content_hash`-basiert, und Metadatenänderungen invalidieren nichts. Veraltet ist ein Derivat, wenn der aktuelle `input_digest` der Cell von seinem abweicht.

Der Cache ist store-global, **nicht** Teil des `RootSet`, wird aber durabel geschrieben (WAL-Frame `DerivationPut`, später Pack). Nichtdeterministische Producer: Der erste durable Eintrag je Schlüssel gewinnt; Überschreiben erfordert eine neue `producer_id`. Ein Explain weist verwendete `producer_id`s aus.

Derivate erben Sicherheitsattribute immer von der **lesenden Cell**, nie vom Cache-Eintrag.

### 8.3 Summary-Cells

Zusammenfassungen über mehrere Quellen sind eigene Cells mit `DerivedFrom`-Kanten und `input_digest = H(sortierte child content_hashes)`. Für sie gilt INV-13:

```
trust(s) = min( min_i trust(src_i), producer_cap )      // LLM-Producer: AgentInferred
taint(s) = ⋃_i taint(src_i) ∪ producer_taint
acl(s)   = ⋃_i clauses(acl(src_i))                      // Konjunktion, siehe §9
```

Wird eine Quelle in ACL oder Trust verschärft, markiert der Fold alle Nachfahren `stale`; die Leseprüfung berechnet die effektive Schranke bis zur Neuerzeugung aus den **aktuellen** Quellen.

### 8.4 Lanes

| Lane | Inhalt | Hot/Cold |
|---|---|---|
| `meta.fixed` | `gid`, Kind, Level, Trust, Taint, Grounding, Valid, `AclRef`, Flags | hot |
| `meta.var` | restliche `CellMeta`-Felder inkl. kompakter Attribute | hot |
| `record` | `content_hash`, `meta_hash`, `born_seq` | hot |
| `attr.bloom` | 8 B je Cell | hot |
| `edge.hint` | 4 B Grad-/Typ-Maske (nur Beschleuniger) | hot |
| `belief.cache` | `alpha`, `beta`, `lcb10` | hot |
| `tokcount` | `u32` je Tokenizer | hot |
| `zoom.L2`, `zoom.L1` | Text | hot |
| `emb.q[space]` | quantisierter Kopf | hot |
| `raw`, `mime` | Inhalt | cold |
| `tok.ids` | bitgepackt | cold |
| `emb.full[space]` | `f16`/`f32` | cold |
| `attrs.ext` | große Attribute | cold |

Zusammen MÜSSEN `meta.fixed`, `meta.var`, `attrs.ext`, `raw` und `mime` die verlustfreie Rekonstruktion von `CellMeta` und `CellContent` samt Hashprüfung erlauben. Fehlende optionale Lanes sind `None`, nie ein Fehler.

---

## 9. Zugriffsausdrücke (`AclRef`)

```rust
pub struct AclExpr { pub clauses: BTreeSet<BTreeSet<GroupId>> }   // CNF
// sichtbar ⇔ ∀ c ∈ clauses: c ∩ groups(principal) ≠ ∅
```

`AclRef = trunc128(ChunkId(AclExpr))`. Eine leere Klauselmenge oder leere Klausel ist ungültig; „öffentlich" ist explizit `[["grp:public"]]`. Konjunktion (Derivate) ist die Vereinigung der Klauseln. Redundante Obermengen-Klauseln werden vor dem Hashen entfernt. Limits: ≤ 16 Klauseln, ≤ 64 Gruppen je Klausel; Überschreitung beim Ableiten ⇒ Derivat wird abgelehnt.

Vorberechnete Bitmaps existieren **je distinktem `AclRef`**; die Sichtmaske eines Principals ist die Vereinigung aller erfüllten.

---

## 10. Handles

Textform `c:<seq>-<mac2>`, kanonisch kleingeschrieben:

```
seq  = Crockford-Base32(gid), mindestens 4 Zeichen, sonst ohne führende Nullen
mac  = LE16(BLAKE3_keyed(k_handle, LE64(gid))[0..2]) & 0x03FF  → 2 Zeichen
k_handle = BLAKE3_derive_key("contextra 2026 handle-mac v1", repo_seed)
Regex: ^c:[0-9a-hjkmnp-tv-z]{4,13}-[0-9a-hjkmnp-tv-z]{2}$
```

Beispiele wie `c:k7qd-3f` sind illustrativ.

- Das MAC ist eine **Tippfehler- und Halluzinationsbremse, keine Berechtigung**. Ein zufällig geratenes Handle besteht die Prüfung mit Wahrscheinlichkeit 2⁻¹⁰; zusätzlich muss die `Gid` im Snapshot sichtbar sein. Jede Auflösung prüft ACL und Capability.
- Handles sind store-stabil, nicht sitzungsgebunden. Der strikte Modus `issued_only` akzeptiert nur in der Session ausgegebene Handles.
- **`did_you_mean`** ohne Bestandsscan: (1) in der Session ausgegebene Handles mit Damerau-Levenshtein ≤ 2; (2) falls leer, aufgezählte Distanz-1-Varianten der Eingabe, gefiltert nach MAC, Sichtbarkeit und ACL. Höchstens 3 Vorschläge, höchstens 1 024 Prüfungen.

---

## 11. Sequenzen

Baum `sequences`, Key `SeqId ‖ position:u64 ‖ gid`, Wert `{ token_counts: [(TokenizerId, u32)] }`. Positionen werden mit Abstand 2²⁰ vergeben. Das angehängte `gid` macht gleichzeitige Anhänge zweier Branches kollisionsfrei; ihre Reihenfolge ist deterministisch. Ist keine Lücke frei, werden betroffene Positionen in einer expliziten Transaktion neu verteilt.

Die Zählwerte stehen im Blattwert, damit Knotenaggregate reine Funktionen der Daten sind. Das Registrieren eines Tokenizers ist eine Katalogänderung, die betroffene Sequenzen in nachfolgenden Commits nachzieht; bis dahin schätzt `window` mit Marge und meldet das.

---

## 12. Veränderliche Sidecar-Daten

```rust
pub struct AccessStats { pub gid: Gid, pub read_count: u32, pub expand_count: u32,
                         pub last_read_ms: i64, pub last_expand_ms: i64 }
```

Eventual-consistent, nicht im WAL, Verlust zulässig. Sampling: 10 % der Recalls, 100 % der Expands, Flush alle 10 s. `salience = w_r·recency + w_a·ln(1+reads) + w_e·expand_rate + w_g·centrality + w_i·importance + w_p·pinned`; sie beeinflusst Ranking-Prior und Archivierungsempfehlungen und löscht nie.

---

## 13. Limits

| Objekt | Limit |
|---|---|
| Cell-Inhalt | ≤ 256 KiB (Namespace-konfigurierbar bis 4 MiB) |
| Attribute je Cell | ≤ 128 / ≤ 32 KiB |
| Ops je Transaktion | ≤ 100 000 und ≤ 48 MiB WAL-Payload |
| Namespaces | ≤ 4 096 |
| Branches je Store | ≤ 100 000 |
| Query-`k` | ≤ 1 000 |

---

## 14. Abnahme

- CBE-Golden-Vektoren für jeden gehashten Typ; Fuzz `encode∘decode = id`.
- Property-Tests: Änderungsmatrix §3.1; Evidenz-Merge ist kommutativ, assoziativ, idempotent; INV-13 über zufällige Lineage-DAGs.
- Handle-Test: gemessene MAC-Fehlakzeptanz über zufällige Strings ≈ 2⁻¹⁰.

---

*Weiter: [SPEC-03 Storage Engine](SPEC-03-STORAGE-ENGINE.md)*

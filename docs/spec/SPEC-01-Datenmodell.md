# SPEC-01 — Datenmodell

Teil der Serie `spec/SPEC.md`. Entscheidungs-IDs (K-xx) verweisen auf das Auflösungsregister in `SPEC.md` §5.

## Inhalt
1. Identitäten und Basistypen · 2. Namespaces, Katalog, Klassenprofile · 3. Cell: Content- und Attr-Zeilen · 4. Provenienz und Grounding · 5. BeliefBeta · 6. Trust und Taint · 7. Bitemporalität, Fakten, Supersession · 8. Kanten · 9. Hierarchie und Sequenzen · 10. Lanes und Zoom-Leiter · 11. Embedding-Spaces, Tokenizer, Analyzer · 12. Handles und Ledger · 13. Derivate · 14. Zugriffsstatistik

---

## 1. Identitäten und Basistypen

```rust
/// Logische, über alle Versionen und Branches stabile Identität einer Cell.
pub struct CellId { pub ns: NsId, pub key: CellKey }
pub struct CellKey(pub [u8; 16]);      // Default UUIDv7-artig (48 Bit ms + 80 Bit Zufall); id_mode=Random: 128 Bit Zufall
pub struct NsId(pub u32);

/// Dichter globaler Index-Adressraum: monoton je Store, NIE wiederverwendet, Lücken erlaubt.
pub struct Gid(pub u64);

/// Keyed BLAKE3-256 über kanonischen Klartext mit Typpräfix (K-13).
pub struct ContentHash(pub [u8; 32]);

pub struct ExternalKey(pub CompactString);  // ≤ 512 B, je Namespace eindeutig, optional
pub struct Seq(pub u64);        // globaler, monotoner Commit-Zähler (Transaktionszeit-Achse)
pub struct Lsn(pub u64);        // WAL-Record-Zähler (K-30)
pub struct Hlc(pub u64);        // 48 Bit ms Wanduhr + 16 Bit logischer Zähler; monoton je Store
pub struct ValidTime(pub i64);  // µs seit Unix-Epoche UTC; Gültigkeitszeit der Realwelt (K-34)
pub struct BranchId(pub u32);   // main = 0
pub struct SegmentId(pub u64);
pub struct RowAddr { pub seg: SegmentId, pub ord: u32 }  // nur innerhalb eines Snapshots gültig
```

**Regeln**
- `Gid` wird beim **Apply des Commits** deterministisch vergeben: `gid = next_gid + i` in der Reihenfolge der `PutCell`-Operationen des Commit-Records. Replay ist damit deterministisch. Transaktionsintern referenzieren neue Cells einander per `CellId`; der WAL-Record trägt `CellId`s, nie vorab vergebene `Gid`s.
- Abgebrochene oder verworfene Transaktionen und Branches hinterlassen Lücken im Gid-Raum. Lücken sind zulässig (Roaring64 ist dafür gebaut).
- `idmap`-Relation: `CellId → Gid` (eindeutig) und `ExternalKey → CellId` je Namespace. Beide sind im Truth-Digest (INV-1).
- `CellKey` im Modus `Uuid7` verrät den Anlagezeitpunkt (Metadatum). Namespaces mit Vertraulichkeitsanforderung setzen `id_mode = "random"`; die Range-Slicing-Optimierung der Gid-Übersetzung hängt **nur** an der Gid, nicht am `CellKey`.
- Der `ContentHash` deckt `type_tag ‖ canonical_bytes` ab; `type_tag` ∈ {`cell.raw`, `blob`, `l1`, `emb-src`}. Hashes sind pro Store (Schlüssel `hash_key`) und **nicht** zwischen Stores vergleichbar.

---

## 2. Namespaces, Katalog, Klassenprofile

Ein **Namespace** ist die Einheit für Schema, Richtlinien und Zugriff. Der **Katalog** ist eine versionierte Truth-Relation `catalog` (Zeitreise gilt auch für Schemata). Er enthält: Namespace-Policies, Prädikate, Edge-Typen, Embedding-Spaces, Tokenizer-Registry, Analyzer-Profile, Decay-Profile, Prinzipale/Gruppen/ACL-Sets, Gate-Konfigurationen, Merge-Funktionen.

### 2.1 MemoryClass und Standardprofile

| Klasse | Inhalt | Decay (Ranking) | `history_days` | `archive_after_days`¹ | `zoom_down_after_days`² |
|---|---|---|---|---|---|
| `episodic` | Konversationsturns, Tool-Ergebnisse | exp, T½ 30 d, floor 0,10 | 30 | 180 | 14 |
| `semantic` | Claims, Entity-Profile | exp, T½ 180 d, floor 0,20 | 365 | 730 | — |
| `procedural` | Skills, Runbooks | keiner | 365 | — | — |
| `working` | Scratchpad, Hypothesen, Pläne | exp, T½ 1 d, floor 0 | 1 (TTL 24 h über `expires_at`) | — | — |
| `artifact` | Code, Reports, Dokumente | exp, T½ 730 d, floor 0,30 | 90 | 1095 | 365 |
| `reference` | Handbücher, API-Docs, Richtlinien | exp, T½ 730 d, floor 0,30 | 30 | — | 365 |

¹ Schwelle, ab der ein Maintenance-Job Records mit niedriger Salience als `ARCHIVED` **vorschlägt/markiert** (Commit, sichtbar, rücknehmbar). ² Ab dann darf Compaction den L0-Rumpf in das **Archiv-Tier** (eigene Segmentfamilie, SPEC-02 §6.5) verlagern; L1/L2 bleiben hot. Weder Decay noch Archivierung löschen (INV-14).

**Drei getrennte Konzepte:** *Retention* (`history_days`, `branches_days`) = Horizont für Zeitreise/tote Versionen; *Archivierung* = Sichtbarkeits-/Tier-Markierung lebender Records; *Decay* = Ranking-Prior. Globale Fallbacks: `history_days = 30`, `branches_days = 1` (K-21).

### 2.2 Namespace-Policy (vollständig, Beispiel mit Defaults)

```toml
[namespace."agent/episodic"]
class                 = "episodic"
id_mode               = "uuid7"                 # | "random"
chunker               = "markdown-v2"           # SPEC-06 §2
tokenizers            = ["o200k", "cl100k"]     # tokenizers[0] = primärer Tokenizer (L2-Limit, Budget-Default)
analyzers             = ["de", "en"]
embedders             = ["emb-768-v3"]          # Embedding-Spaces
active_embedder       = "emb-768-v3"
zoom                  = ["L0:raw", "L1:summary<=200tok", "L2:gist<=24tok"]
default_trust         = "tool-unverified"       # entspricht TrustTier::AgentInferred + Taint UNVERIFIED_TOOL
min_main_trust        = "tool-verified"         # Gate G2
acl_default           = "grp:default"
max_cell_size         = "64KiB"                 # ≤ 256KiB; größere Payloads = BlobRef (≤ 1GiB)
decay                 = { kind = "exponential", half_life_days = 30, floor = 0.1 }
retention             = { history_days = 30, branches_days = 1 }
archive_after_days    = 180
zoom_down_after_days  = 14
merge                 = { text = "manual", props = "field", counters = "sum", edges = "add-wins", derived = "stale" }
durability            = "group"                 # strict | group | relaxed | volatile
signing               = "none"                  # none | commit-optional | commit-required
injection_policy      = "flag-only"             # quarantine | flag-only | escalate
dedupe                = { near_duplicate = "off" }   # off | minhash-0.9 (SPEC-06 §3.4)
gates                 = ["G1","G2","G3","G4","G6"]   # G5, G7 optional zuschaltbar
```

**Prädikate** (Katalog): `Predicate { id, name, cardinality: Functional | Multi, object_type, numeric_tolerance: Option<f64>, supersede_policy: SupersedeOnNewer | TrustWeighted | Manual }`. Namespaces listen ihre funktionalen Prädikate; ein Fakt mit unbekanntem Prädikat wird von G1 abgelehnt.

**Mandantenisolation:** ein Store ist die harte Grenze (`SPEC.md` §4.1). Der Derivat-Cache ist je Store; es gibt keinen Zugriff über Stores hinweg.

---

## 3. Cell: Content- und Attr-Zeilen

Eine **Cell** ist die logische Wissenseinheit. Physisch besteht sie aus **einer oder mehreren Content-Zeilen** (Revisionen des Inhalts), **einer oder mehreren Attr-Zeilen** (Revisionen der Attribute) und **Kanten** in der Relation `edge`. Alle Zeilen sind immutable; Ersetzung erzeugt eine neue Zeile und einen Death-Eintrag (SPEC-02 §7).

### 3.1 Content-Zeile (`row_kind = Content`)

```rust
pub struct ContentRow {
    // Schlüssel
    pub gid: Gid, pub born_seq: Seq, pub branch: BranchId,
    // Identitätsfacetten (ändern sich nie für eine Gid)
    pub cell_id: CellId,
    pub kind: CellKind,           // Chunk | Section | Document | Entity | Fact | Episode | Summary | Artifact | Contradiction
    pub level: u8,                // 0 Chunk/Episode/Fact/Entity, 1 Section, 2 Document, 3 Thema (Summary)
    pub class: MemoryClass,
    // Inhalt
    pub content_hash: ContentHash,
    pub payload: RawPayload,      // Inline (body.text, ≤ max_cell_size) | BlobRef(ContentHash)
    pub mime: MimeId,
    pub chunker: Option<(ChunkerId, u16)>,   // Id und Version, reproduzierbares Re-Chunking
    pub boundary_safe: bool,
    // Deterministisch abgeleitete Hot-Lanes (nicht im Digest)
    pub tok: SmallVec<[u32; 4]>,  // exakte Token-Zahl je registriertem Tokenizer (Reihenfolge = Katalog)
    pub l2: GistRef,              // extraktiver Titel/Gist ≤ 24 Token (primärer Tokenizer)
    pub emb_q: Option<QuantHead>, // Matryoshka-Präfix (Default 256 Dim, SQ8) des aktiven Space
    pub entry_hash: [u8; 32],     // BLAKE3 des logischen Digest-Eintrags (SPEC-03 §2.2)
}
```
- Eine **neue Content-Zeile für dieselbe `gid`** ist eine **Inhalts-Revision** (Revisions-Diff, SPEC-06 §3). Die vorherige Content-Zeile erhält einen Death-Eintrag. Embedding, L1, Index-Treffer der alten Zeile werden durch die Sichtbarkeitsregel (§3.3) ausgeblendet.
- Fakten-/Entity-Cells haben eine kurze Content-Zeile (Normalform des Claims als Text) damit sie in BM25/Vektor/Packer wie jede Cell funktionieren.

### 3.2 Attr-Zeile (`row_kind = Attr`)

```rust
pub struct AttrRow {
    pub gid: Gid, pub born_seq: Seq, pub branch: BranchId,
    pub parent: Option<Gid>,            // Hierarchie (§9)
    pub props: Props,                   // typisiert nach Namespace-Schema, ≤ 128 Felder, ≤ 32 KiB
    pub valid: ValidInterval,           // [from, to) in ValidTime; None = offen
    pub trust: TrustTier,
    pub taint: TaintFlags,
    pub grounding: GroundingLevel,      // G0..G3
    pub belief: BeliefBeta,             // Cache der Provenienz-Faltung (§5)
    pub conf_cap: f16,                  // Obergrenze abgeleiteter Konfidenz (§5.3); 1.0 = keine
    pub lcb: u16,                       // Hot-Lane: 10-%-LCB als Festkomma (v·65535), abgeleitet aus belief/conf_cap
    pub acl: AclId,                     // interniertes ACL-Set im Katalog
    pub flags: CellFlags,               // TOMBSTONE | ARCHIVED | PINNED | STALE | DERIVED
    pub support: u32,                   // Anzahl bestätigender Evidenzen (kommutativ, additiv beim Merge)
    pub expires_at: Option<ValidTime>,  // Working-Memory-TTL; Ablauf = Tombstone-Commit durch Maintenance
    pub prov_head: u32,                 // höchste ev_seq in `prov` für diese Gid
    pub entry_hash: [u8; 32],
}
```
- **`meta_hash`** = `BLAKE3(canonical(AttrRow ohne gid/born_seq/branch/entry_hash))`; reiner Gleichheitshash für Merge und `diff`. Er adressiert nichts.
- Eine Attr-Zeile ersetzt die vorherige Attr-Zeile (Death-Eintrag) und lässt die Content-Zeile **leben**. Attributänderungen (Tags, Trust, ACL, `valid_to`) lösen **keine** Neuberechnung von Embedding, Token-Zählung, Text-Index oder Zoom aus.
- **Kommutative Deltas** (`support += n`, Provenienz-/Belief-Evidenz anhängen) sind eigene Operationen, die beim Apply auf die aktuelle Attr-Zeile angewandt werden und deshalb **nicht** mit anderen Attr-Schreibern kollidieren (SPEC-03 §4.3).

### 3.3 Sichtbare Version einer Cell

Zu Snapshot `S` einer Branch-Lineage ist die sichtbare Cell das Paar aus **sichtbarer Content-Zeile** und **sichtbarer Attr-Zeile**, bestimmt nach der Lineage-Regel (SPEC-03 §6.2). Ist die Attr-Zeile `TOMBSTONE`, ist die Cell unsichtbar (in Suche, Lesen, Kanten-Expansion). In der Zeitreise innerhalb der Retention bleibt sie sichtbar.

Hat eine Cell eine Content-Zeile, aber keine eigene Attr-Zeile (Erstanlage setzt immer beide; Ausnahme: Merge-Rekonstruktion), gelten die Namespace-Defaults. Eine Attr-Zeile ohne sichtbare Content-Zeile ist ein Integritätsfehler (`verify`, `E_CORRUPT_TRUTH`).

### 3.4 CellKind und Hierarchie

| Kind | Level | Bedeutung |
|---|---|---|
| `Chunk` | 0 | Rohchunk eines Dokuments |
| `Section` | 1 | Abschnitt (Parent von Chunks); Rumpf = Überschrift oder leer, L1 = Zusammenfassung |
| `Document` | 2 | Dokument; Rumpf leer oder Metadaten, L1/L2 Summaries |
| `Episode` | 0 | Konversationsturn/Tool-Ergebnis in einer Sequenz |
| `Entity` | 0 | Kanonische Entität mit Alias-Liste in `props` |
| `Fact` | 0 | Strukturierte Aussage (§7.1) |
| `Summary` | ≥ 1 | Abgeleitete Zusammenfassung (Community/Thema), `DERIVED`, Edges `Summarizes` |
| `Artifact` | 0 | Code/Report/Binär-nahes Objekt; Payload evtl. `BlobRef` |
| `Contradiction` | 0 | Widerspruch zwischen zwei Aussagen (SPEC-03 §8.2) |

`level` und `kind` sind unveränderlich je Gid. Hierarchie `Document → Section → Chunk` ist über `parent` (Attr) und die Relation `seq` (§9) modelliert; **nicht** über Kanten (K-04-Nachtrag in §9).

---

## 4. Provenienz und Grounding

```rust
/// Append-only in Relation `prov`; Schlüssel (subject_gid, ev_seq). Pflicht für jede Content-Zeile (G2).
pub struct ProvRecord {
    pub id: ProvId,                       // (subject_gid, ev_seq); ev_seq startet bei 0
    pub activity: ActivityKind,           // Ingest | Extract | Infer | Summarize | HumanEdit | Merge | Reaffirm
    pub origin: Origin,
    pub polarity: Polarity,               // Supports | Refutes
    pub weight: f16,                      // Quellenzuverlässigkeit w ∈ [0,1] (Defaults §5.2)
    pub extractor_conf: f16,              // kalibrierte Extraktorkonfidenz c ∈ [0,1]
    pub source_uri: Option<CompactString>,
    pub source_hash: Option<ContentHash>,
    pub source_span: Option<(u64, u64)>,  // Byte-Bereich im Quelltext
    pub quote_hash: Option<[u8; 16]>,
    pub derived_from: SmallVec<[Gid; 4]>, // bei Infer/Summarize: genutzte Cells
    pub source_root: [u8; 16],            // Wurzel der Ableitungskette (§4.2)
    pub producer: Option<ProducerId>,     // Modell/Parser + Version + Parameter-Hash
    pub run: Option<RunId>,               // Ingest-Lauf / Session
    pub observed_at: Hlc,
}
pub enum Origin {
    Ingest { parser_version: u16 },
    LlmExtraction { model: ModelId, prompt_hash: [u8; 16] },
    ToolCall { tool: ToolId, input_hash: [u8; 16], output_hash: [u8; 16] },
    Human { principal: PrincipalId },
    Derivation { producer: ProducerId },
    Merge { from_branch: BranchId, mr: u32 },
}
```
Provenienz wird **nicht pro Record signiert**; Manipulationsschutz = ACL + Prüfsummen + Commit-Hash-Kette (K-27).

### 4.1 Reaffirmation
Dieselbe Aussage aus unabhängiger Quelle erzeugt **kein Duplikat**: ein weiterer `ProvRecord` (`ev_seq + 1`, `activity = Reaffirm`) wird angehängt; `support`, `belief` und `last_confirmed` (Prop `$last_confirmed`) aktualisieren sich über kommutative Deltas.

### 4.2 `source_root`
128-Bit-Hash (`BLAKE3-128`) der **unabhängigen Quellidentität**:

| Fall | `source_root` |
|---|---|
| Dokument-Ingest | `H("doc", source_uri)` (versionsunabhängig) |
| LLM-Extraktion aus Dokument | Root des Quelldokuments |
| Tool-Aufruf | `H("tool", tool_id, input_hash)` |
| Mensch | `H("human", principal)` |
| Ableitung/Inferenz | `H("derive", sortierte Roots der Eltern)` |
| Merge | Root der übernommenen Evidenz (unverändert) |

Folge: Ein Agent, der seine eigene frühere Ausgabe zitiert, erzeugt dieselbe abgeleitete Root und verstärkt nichts (Echokammer-Schutz).

### 4.3 Grounding-Stufen
| Stufe | Bedeutung | Standard-Recall |
|---|---|---|
| **G0** `VerifiedSpan` | Quelle vorhanden, Spanne gegen `source_hash` verifiziert | ja |
| **G1** `UnverifiedSource` | Quelle vorhanden, Spanne ungeprüft | ja (markiert) |
| **G2** `Inferred` | Agenteninferenz mit `derived_from` und `conf_cap` | nur mit `min_lcb` |
| **G3** `Hypothesis` | keine Quelle; Quarantäne; Trust = `Hypothesis` | nur mit `include_hypotheses` |

---

## 5. BeliefBeta

### 5.1 Struktur
```rust
pub struct BeliefBeta { pub alpha: f32, pub beta: f32, pub roots: SourceRoots }
/// Bis zu 8 exakte 16-Byte-Roots + 256-Bit-Bloom (k = 3) für weitere. Bloom-False-Positive = konservativ
/// (eine echte unabhängige Bestätigung wird fälschlich nicht gezählt, nie umgekehrt).
pub struct SourceRoots { pub exact: SmallVec<[[u8; 16]; 8]>, pub overflow: [u8; 32] }
```

### 5.2 Faltung (normativ, deterministisch)
`belief` ist ein **Cache**, berechnet aus allen `ProvRecord`s der Cell:

```
(α0, β0) = Prior(prior_tier)                       // Tier bei Erstanlage
α = α0 + Σ_{r ∈ Roots⁺}  max_{e: root=r, Supports}  (w_e · c_e)
β = β0 + Σ_{r ∈ Roots⁻}  max_{e: root=r, Refutes}   w_e
```
- **Priors:** `Verified`/System → Beta(4,1); `Curated`/`ToolVerified` → Beta(2,1); `Untrusted` → Beta(1,1); `AgentInferred`/`Hypothesis` → Beta(1,2).
- **Unabhängigkeitsgebot:** Je `source_root` zählt nur der stärkste Beitrag. Mehrere Bestätigungen derselben Root erhöhen `α` nicht.
- **Standardgewichte `w`** (Katalog `source_reliability`, überschreibbar): `Human` 1,0 · `Ingest` 0,8 · `ToolCall` 0,7 · `Derivation` 0,6 · `LlmExtraction` 0,5.
- **Merge:** `prov` wird vereinigt (nie Konflikt), `belief` danach **neu gefaltet** (kein Feldkonflikt möglich).

### 5.3 LCB und abgeleitete Konfidenz
Unterer 10-%-Wert (normative, plattformunabhängige Näherung; nur `+ − × ÷ √` in `f64`, daher bitgenau reproduzierbar):

```
n   = α + β
μ   = α / n
σ   = √( α·β / (n² · (n + 1)) )
lcb10(α,β) = clamp(μ − 1.2816·σ, 0, 1)
lcb_eff    = min(lcb10(α, β), conf_cap)
```
- **Ableitung (Noisy-AND):** `conf_cap(derived) = min_i lcb_eff(parent_i) · extractor_conf`. Eine Ableitung ist nie vertrauenswürdiger als ihr schwächster Elternteil.
- **Planner/Packer** verwenden ausschließlich `lcb_eff` (Hot-Lane `lcb`, 16-Bit-Festkomma), nie den Mittelwert.
- **Hochstufung aus Quarantäne** (Hypothesis → AgentInferred): `lcb_eff ≥ 0,70` bei ≥ 2 unabhängigen Roots **oder** menschliche Freigabe (G7).

---

## 6. Trust und Taint

```
TrustTier:  Verified(5) > Curated(4) > ToolVerified(3) > AgentInferred(2) > Untrusted(1) > Hypothesis(0)
TaintFlags: EXTERNAL | INJECTION_SUSPECT | UNVERIFIED_TOOL | PII        (Bitmaske, kombinierbar)
```
- **Vererbung:** abgeleitete Records erhalten das **Minimum** der Trust-Stufen und die **Vereinigung** der Taint-Flags ihrer Quellen.
- **Hochstufung** nur durch Gate G7 oder einen Prinzipal mit Capability `trust.promote` (Commit-Eintrag mit Begründung); `Verified` zusätzlich `signing ≠ none` (SPEC-07 §6).
- **Taint-Deckel (G6):** Records aus Session-Branches mit `EXTERNAL` werden auf `AgentInferred` gedeckelt, mit `INJECTION_SUSPECT` auf `Untrusted`, bis ein Mensch freigibt. Ingest über Connector-Prinzipale mit Capability `ingest.trust ≤ Curated` darf `Curated` + `EXTERNAL` setzen.
- `PII` ist ein reines **Ausgabe-Policy-Flag** (Ausschluss aus Recall/Export), kein Löschmechanismus.

---

## 7. Bitemporalität, Fakten, Supersession

```rust
pub struct ValidInterval { pub from: Option<ValidTime>, pub to: Option<ValidTime> }   // [from, to)
```
- **Transaktionszeit** `ts_tx` ist der Commit-`Hlc` des `born_seq` (aus dem Commit-Log; unveränderlich). **Gültigkeitszeit** `valid` ist Realwelt.
- Zeilen ohne Gültigkeitsangabe gelten als **immer gültig** (fail-open für Zeit; fail-closed für ACL/Trust).
- Zeitreise: `AS OF SEQ|TIMESTAMP|COMMIT` (Transaktionszeit), `VALID AT t` (Gültigkeitszeit), kombinierbar (SPEC-03 §9).

### 7.1 Fakten (`CellKind::Fact`)
Reservierte Props: `$subject: CellRef`, `$predicate: PredicateId`, `$object: CellRef | Literal`, `$polarity: +|-`. Die Content-Zeile enthält die Normalform `"<subject> <predicate> <object>"` (für Suche/Packer). n-äre Relationen = **reifizierte Facts** mit Rollen-Kanten (`RoleOf`), kein Hyperkanten-Speicher.

### 7.2 Supersession und Widerspruch beim Schreiben (deterministisch, synchron im Commit)
Für einen neuen Fakt `N` mit funktionalem Prädikat und offenem Fakt `O` zu `(subject, predicate)`, abweichendem Objekt (numerisch: außerhalb `numeric_tolerance`) und überlappender Gültigkeit:

1. `supersede_policy = SupersedeOnNewer` **und** `N.valid.from > O.valid.from` **und** `N.trust ≥ O.trust` ⇒ Commit schließt `O.valid.to = N.valid.from` (Attr-Zeile, kein Re-Embedding) und legt Kante `Supersedes(N → O)` an. Es entsteht **kein** Konflikt.
2. `TrustWeighted` ⇒ wie 1, aber nur wenn `lcb_eff(N) > lcb_eff(O) + 0,10`.
3. Alles andere (gleichrangig, `Manual`) ⇒ **`Contradiction`-Cell** (Kind `Direct` bzw. `Numeric`), beide Seiten bleiben unverändert (SPEC-03 §8.2).

---

## 8. Kanten

```rust
pub struct EdgeRow {
    pub edge_id: u64,            // = BLAKE3_keyed_64(src_gid ‖ kind ‖ dst_gid ‖ qualifier): deterministisch ⇒ Add/Add identisch = eine Kante
    pub src: Gid, pub kind: EdgeKind, pub dst: Gid, pub qualifier: u32,   // qualifier erlaubt Parallelkanten, Default 0
    pub born_seq: Seq, pub branch: BranchId,
    pub weight: f32,
    pub valid: ValidInterval,
    pub trust: TrustTier, pub belief: BeliefBeta, pub conf: f16,
    pub prov: ProvId, pub flags: EdgeFlags,   // TOMBSTONE
    pub props: SmallProps,
}
```
- **Kanten-Typen** (`EdgeKind: u16`; < 256 System, ≥ 256 Katalog-definiert): `Mentions, RelatedTo, Cites, DerivedFrom, Summarizes, Supersedes, Contradicts, Supports, DuplicateOf, SameAs, RoleOf, Custom`. Strukturkanten `PartOf/Contains/Follows` sind **synthetisch** (nur im CSR-Index aus `parent` + `seq` erzeugt, kein Truth-Speicher).
- Schlüssel im Truth-Segment: `(src, kind, dst, edge_id)`; Rückrichtung wird im CSR-Index aufgebaut, nicht doppelt im Truth gespeichert.
- **Sichtbarkeit einer Kante** folgt der Sichtbarkeit **beider** Endpunkte (Lesezeit-Join über Frontier-Bitmap). Retire eines Knotens retired **nicht** automatisch seine Kanten; G1 meldet „dangling edge" beim Merge.
- **Merge:** Mengensemantik, `AddWins` Default (Add/Add identisch = eine Kante; Add/Delete ⇒ Kante bleibt); je Kantentyp `AddWins | RemoveWins | Manual` (Katalog).
- **Hub-Schutz:** Warnung ab 10⁴ aktiven Kanten je Knoten, hartes Limit 10⁶ (`E_QUOTA_EXCEEDED`).

---

## 9. Hierarchie und Sequenzen

Relation `seq` (Truth, digestiert): `(parent_gid: Gid, order_key: u64) → child_gid: Gid` (+ `SEQ_TOMBSTONE`).

- **Geordnete Kinder** eines Parents: Dokument → Abschnitte → Chunks, Episode-Container → Turns. `parent` (Attr) und `seq` müssen konsistent sein (G1).
- **`order_key`:** Anhängen = `letzter + 2²⁰`; Einfügen = Mittelwert der Nachbarn; Lücke < 2 ⇒ Rebalancing-Transaktion über lokale Nachbarn (begrenzt, ≤ 1 024 Zeilen).
- **Fenster-Lesen** (`window`) nutzt `seq` zur Geschwisterfolge und die Hot-Lane `tok` der Content-Zeilen (kein Inhaltszugriff). Es gibt **keine** denormalisierten Token-Summen im Baum (Revisionen würden sie entwerten).
- Eine Cell hat höchstens einen Parent. Mehrfachzugehörigkeit = Kante `RelatedTo`.

---

## 10. Lanes und Zoom-Leiter

| Lane | Inhalt | Klasse | Quelle |
|---|---|---|---|
| `keys` | gid, born_seq, branch, row_kind, flags | hot | Truth |
| `meta` | kind, class, level, parent, trust, taint, grounding, `lcb`, `acl`, valid, expires, AttrBloom | hot | Truth |
| `tok` | `tok_count` je Tokenizer | hot | deterministisch (Port `Tokenizer`) |
| `l2` | Gist ≤ 24 Token | hot | deterministisch beim Commit |
| `emb.q` | Matryoshka-Präfix (SQ8) des aktiven Space | hot | Derivat (inline oder `.clane`) |
| `l1` | Zusammenfassung ≤ 200 Token | warm (eigener Block) | Derivat (Work Queue), Fallback extraktiv |
| `raw` (= L0) | kanonischer Rohinhalt (zstd) | cold | Truth |
| `emb.full` | F16/F32-Vollvektor je Space | cold | Derivat |
| `props.ext` | große/seltene Props | cold | Truth |

| Stufe | Name | Ziel | Erzeugung |
|---|---|---|---|
| L0 | `raw` | unbegrenzt | Original; Parents fassen Kinder zusammen (Small-to-Big) |
| L1 | `summary` | ≤ 200 Token | abstraktiv (Job); **Fallback deterministisch** `extractive-lead-v1` (führende Sätze bis 200 Token) |
| L2 | `gist` | ≤ 24 Token | **synchron und deterministisch** beim Commit (Überschriftenpfad + Schlüsselsatz; AST-Signatur bei Code; erster Satz sonst) |
| L3 | `handle` | ~5 Token | engine-intern (`c:<gid>-<mac>`) |

`derived_from = content_hash` je Zoom-Derivat (INV-13). Fehlende L1 ⇒ der Packer wählt die nächste verfügbare Stufe und weist `lane_coverage` aus.

---

## 11. Embedding-Spaces, Tokenizer, Analyzer

```rust
pub struct EmbeddingSpace {
    pub id: SpaceId, pub model: ModelId, pub version: u32, pub dim: u16,
    pub metric: Cosine | Dot | L2, pub normalize: bool,
    pub storage: F16 | F32,                    // Default F16
    pub matryoshka_prefixes: Vec<u16>,         // z. B. [256, 512, 1024]; emb.q nutzt den ersten
    pub index: IndexParams,                    // Flat | Hnsw{M,efC} | Vamana{R,L,alpha}
    pub quant: None | Sq8 | Pq{m,bits} | RaBitQ{bits},
    pub cos_floor: f32,                        // Abstention-Kalibrierung (SPEC-05 §5.2), per `calibrate_space` gesetzt
    pub tokenizer: Option<TokenizerId>,
}
```
- Embeddings sind **Derivate** `(ContentHash, SpaceId)`; identischer Inhalt wird pro Store **einmal** berechnet (über Namespaces und Branches). Clients dürfen Embeddings mitliefern (Space-ID Pflicht, NaN/∞/falsche Dimension ⇒ `E_VECTOR_INVALID`).
- **Modellwechsel:** neuer Space + Doppelbetrieb + Job `Reembed` (Priorität nach Salience) + `lane_coverage`; Umschalten = Katalog-Commit (`active_embedder`). Vektoren verschiedener Spaces werden nie verglichen. Alter Space wird nach `retention.history_days` verworfen.
- **Tokenizer-Registry:** `TokenizerSpec { id, family, vocab_hash, port }`. Für registrierte Tokenizer persistiert der Ingest exakte `tok`; für unregistrierte schätzt der Packer (≈ 4 Zeichen/Token, Code × 1,8) und markiert `estimated` (Budget mit 5 % Abschlag). Nachträglich registrierte Tokenizer füllen `tok` per `.clane`-Patch (Lazy-Job).
- **Analyzer-Profil:** `AnalyzerSpec { id, steps[], langs[], params_hash }`; `id + params_hash` fließen in den Index-Segmentschlüssel.

---

## 12. Handles und Ledger

```rust
/// Textform: "c:<gid-crockford>-<mac>", z. B. "c:k7qd-3f". Eingabe case-insensitiv (O→0, I/L→1); Ausgabe klein.
pub struct Handle { pub gid: Gid, pub mac: u64 /* mac_bits Bit */ }
```
- **Gid-Teil:** Crockford-Base32, auf mindestens 4 Zeichen aufgefüllt (keine Stripping-Mehrdeutigkeit; 6 Zeichen decken 10⁹).
- **MAC:** `mac = trunc_bits( BLAKE3::keyed_hash(handle_key, "h1" ‖ gid_le8) )`, `handle_key = HKDF(repo_seed, "handle-mac-v1")`. `mac_bits ∈ {10, 20, 40}` je Store (Default **10** = 2 Zeichen; `20` = 4, `40` = 8 Zeichen). Wechsel ist ein Katalog-Commit; ältere Längen bleiben bis zum Ablauf der Sessions lesbar.
- Handles sind **repo-weit stabil und branch-übergreifend** (Gid-basiert); es gibt keine Handle-Tabelle.
- **Ledger (INV-16):** Jede Session führt `HandleLedger: Gid → (content_hash, seq, level, issued_at)` für **alle in dieser Session ausgegebenen** Handles (Recall, Expand, Neighbors). Es wird beim Recall in `alog` protokolliert (kompakt: `(recall_id, [gid, content_hash])`) und nach Neustart lazy daraus rekonstruiert. Limit 100 000 Einträge je Session (LRU auf ältere Recalls; `E_QUOTA_EXCEEDED` für Citation-Pflicht-Modus erst danach).
- **Prüfreihenfolge (`resolve`)**: (1) Syntax → `E_HANDLE_UNKNOWN`; (2) MAC → `E_HANDLE_MAC_INVALID` mit `did_you_mean` (Damerau-Levenshtein ≤ 2 auf **ACL-berechtigten** Gids der Nachbarschaft, max. 3); (3) Sichtbarkeit/ACL → neutral `E_HANDLE_UNKNOWN`; (4) Ledger → `NotIssued` (für `verify`: `InvalidHandle`).
- **Zitatanker:** `c:k7qd-3f#s120-188` (Zeichenspanne im L0). Verifier prüft die Spanne gegen die Content-Zeile des Ledger-Eintrags.

---

## 13. Derivate

Ein **Derivat** ist eine Funktion des Inhalts: `(ContentHash, ProducerId) → Wert`.

```rust
pub struct ProducerId { pub kind: ProducerKind /* Embedder|Summarizer|Tokenizer|Extractor|LangDetect */,
                        pub model: ModelId, pub version: u32, pub params_hash: u64 }
```
- **Lane-Arten** (persistiert als Segment-Lanes oder `.clane`-Patch): `Emb(SpaceId)`, `Summary(L1, ProducerId)`, `Tok(TokenizerId)`. Entitäts-/Fakten-Extraktion ist **kein** Lane-Derivat, sondern erzeugt Cells/Kanten per **Commit** (Branch → Gates).
- Derivat-Cache **je Store**, branch-übergreifend sichtbar (reine Funktion; das Vorhandensein einer Lane verändert nie den Truth). Lane-Verfügbarkeit erscheint als `lane_coverage`/`lane_gen` im Explain (INV-15).
- **Staleness:** Ändert sich `content_hash`, existiert kein Derivat für den neuen Hash ⇒ Lane fehlt (kein „falsches" Derivat). Für **Eltern-Zusammenfassungen** (`level > 0`): `derivation_hash = H(Kind-content_hashes ‖ producer ‖ prompt_hash)`; Abweichung ⇒ `STALE`-Flag + Neuerzeugung in der Work Queue; bis dahin Warnung oder Fallback auf die Kinder.
- **Referenzzählung** über `content_hash`-Verwendung entscheidet bei Compaction über das physische Entfernen von Derivaten (SPEC-02 §11).

---

## 14. Zugriffsstatistik (Sidecar, nicht versioniert)

```rust
pub struct AccessStats { pub gid: Gid, pub reads: u32, pub expands: u32, pub last_read: ValidTime,
                         pub last_expand: ValidTime, pub salience: f32 }
```
`sidecar/access_stats.cxs`: periodischer Checkpoint, **nicht im WAL, nicht im Digest**; Verlust unkritisch. Gesampelt (Default 10 % der Reads, alle Expands). `salience`-Formel (Gewichte je Namespace konfigurierbar):

```
salience = 0.35·recency + 0.25·log1p(reads)/log1p(max_reads) + 0.20·expand_rate
         + 0.10·centrality + 0.10·importance
```
`recency = 2^(−Δt/T½)`, `centrality` = normalisierter PPR-Grad aus dem Graph-Batch, `importance` = Prop `$importance ∈ [0,1]` (Default 0,5). Die Salience steuert Ranking-Prior und Archivierungsvorschläge und **löscht nie**. Für Reproduzierbarkeit (INV-15) nennt das Explain den verwendeten Salience-Stand (`stats_gen`).

# `contextra-types` – Implementierungsspezifikation

|Feld|Wert|
|---|---|
|Zweck-Satz (D-97)|Reine Datentypen: IDs, Tags, Schlüssel, Fehler, Löschtypen. Kein I/O.|
|Ring|0 (SPEC 5.1)|
|Erlaubte Workspace-Abhängigkeiten|keine (SPEC 5.3)|
|Stufe / Gates|S0; G-B (Rückbau), G0 (Schlüsselraum-Prototyp braucht die Typen)|
|Arbeitspakete|WP-1.1, WP-1.2 (Rückbau), WP-3.1 (Neubau der Kerntypen)|
|SPEC-Anker|8.1, 9.1–9.3, 9.6, 7.2–7.5, 13.1, Anhang C, D, F|

## 1. Ist-Zustand (HEAD `1454bbec0`) [V]

|Punkt|Befund|
|---|---|
|Umfang|7.304 Zeilen `src`|
|Module|`erasure.rs` (44 Z.: nur `ScanMethod`, `RequestContext`, `SagaRecord`; nichts davon ist Soll, F-33), `error.rs`, `error_dto.rs`, `model_fingerprint.rs`, `retrieval_strategy.rs`, `schema.rs`, `tenant_scope.rs`, `tombstone.rs`, `types.rs` + `types/` (`domain/ids.rs`, `domain/document.rs`, `domain/misc.rs`, `filter.rs`, `budget.rs`, `importance.rs`, `saos.rs`)|
|Altlasten|`DocId(pub u64)` (Feature `docid-128` → `u128`, unkeyed, F-12); `types/saos.rs` 940 Z.; `AutoExtractionMode`; `pub use erasure as witness`; `SeqBitTombstone` und `TombstoneSemanticsCheck`; `DocIdWidth`|
|Features|`default`, `docid-128`, `auto-extraction-opt-out` (alle zu entfernen, D-84)|
|Abhängigkeiten|`thiserror`, `serde`, `serde_json`, `bincode`, `ahash`, `blake3`, `sha2`, `bytes`|
|Struktur|Crate, Modul und Pfad gleichnamig (`types::types::domain`, F-30); `misc.rs`; gemischte Modulform (`types.rs` + `types/`)|

## 2. Verantwortung und Abgrenzung

`types` hält **nur Daten und deren reine Invarianten**: Newtypes mit geprüften Konstruktoren, Schlüsselkodierung, Matcher, Fehlertaxonomie, Löschtypen, Konstanten (Grenzen, Labels). Es enthält weder I/O noch `async`, weder Zeit noch Zufall noch Kryptografie (kein Schlüsselmaterial, keine Hashfunktion zur Tag-Bildung). Die Tag-Bildung (keyed BLAKE3) liegt in `crypto`; `types` liefert nur die längenpräfigierte Vorform [E]. `types` kennt kein Crate des Workspace (T-ARCH-1).

## 3. Zielstruktur (Module) [E]

Namensregeln D-90: kein `types::types`, keine `misc`, eine Modulform.

|Modul|Inhalt|SPEC|
|---|---|---|
|`tags`|`TenantTag`, `CollTag`, `SubjectTag`, `ClusterTag`, `DocTag`, `TermTag`, `NameTag` (je `[u8; 16]`-Newtype); `TagPreimage` (längenpräfigierte Vorform)|D-21, 9.3|
|`labels`|Konstanten aller Labels aus Anhang C (`b"ctx/doc/v2"` …)|Anhang C|
|`ids`|`ErasureId` (16 B), `FileId` (16 B), `StoreId` (16 B), `Seq`, `Timestamp` (µs UTC), `GateId`|7.4, F.1, F.9|
|`refs`|`TenantRef`, `CollectionRef`, `SubjectRef`, `OperatorRef` (opak, begrenzt, nicht leer), `DocKey`, `DocRef`, `DocHandle`, `TenantContext`|D-70, 7.2, 7.3|
|`key`|`Region`, `Family`, `KeyBuf` (78 B), `KeyBound` (65 B), `KeyRange`, `Matcher`|9.1, D-10|
|`erasure`|`EraseRequest`, `EraseTarget`, `Timing`, `EraseMode`, `RetentionCode`, `RetentionAttestation`, `ErasureStatus`, `PendingReason`, `RefusalReason`, `GateVariant`, `LayerId`, `KeyCoverage`, `ResidualRisk`, `MeasureClass`, `Scanner<S>`, `LayerWitness`|8.1, 7.3, 7.4, 12.2, Anhang D|
|`file`|`FileKind` (1..10), `AlgIds`, `FormatVersion`|F.1|
|`limits`|Konstanten D-87 (`MAX_DOCS_PER_STORE`, `MAX_DOC_BYTES_DEFAULT`, `MAX_DOC_BYTES_HARD`, `MAX_SNAPSHOTS_PER_TENANT`, `MAX_DIMENSION`, `MAX_QUERY_TERMS`, `MAX_K`)|D-87, 14.4|
|`secret`|`Secret` (zeroize-on-drop, kein `Debug`/`Display`/`Clone`)|7.1|
|`error`|`Error` (geschlossene Enum mit festen Texten), Unterenums `Conflict`, `Limit`, `ConfigError`, `CorruptKind`|7.5|

## 4. Schnittstellen (Entwurf [E] auf der Basis fester SPEC-Aussagen)

### 4.1 Tags und Schlüssel

```rust
// tags.rs – 16 Byte, Eq/Ord/Hash, Copy. Debug zeigt höchstens 4 Byte Hex (HR-4).
pub struct DocTag([u8; 16]);           // analog TenantTag, CollTag, SubjectTag, TermTag, NameTag
pub struct ClusterTag([u8; 16]);
impl ClusterTag {
    pub fn from_subject(s: SubjectTag) -> Self;          // cluster_tag = subject_tag  (D-70)
    pub fn unassigned(d: DocTag) -> Self;                // 0x00¹² ‖ doc_tag[0..4]      (D-70)
}
/// Vorform: label ‖ (u32 len ‖ feld)*  – verhindert (a‖bc) == (ab‖c)  (9.3)
pub struct TagPreimage(Vec<u8>);
impl TagPreimage { pub fn new(label: &'static [u8]) -> Self; pub fn field(self, f: &[u8]) -> Self; pub fn as_bytes(&self) -> &[u8]; }

// key.rs
pub const KEY_BOUND_LEN: usize = 65;   // region(1) + 4 × 16
pub const KEY_LEN: usize = 78;         // + family(1) + sub(u32) + !commit_seq(u64)
#[repr(u8)] pub enum Region { Data = 0x01, Meta = 0x03 }                  // 0x02 reserviert (LOC entfällt)
#[repr(u8)] pub enum Family { Doc = 0x01, DocMap = 0x02, Manifest = 0x03, TextBuf = 0x05 } // 0x04 reserviert
pub struct KeyBuf([u8; KEY_LEN]);
impl KeyBuf { pub fn new(region, tenant, coll, cluster, doc, family, sub: u32, commit_seq: u64) -> Self;
              pub fn commit_seq(&self) -> u64;           // aus !commit_seq zurückgerechnet, ohne DEK lesbar (D-74)
              pub fn bound(&self) -> KeyBound; }
pub struct KeyBound([u8; KEY_BOUND_LEN]);
pub struct KeyRange { lo: KeyBound, hi: KeyBound }       // inklusive, gleiche Länge (D-10)
impl KeyRange {
    pub fn tenant(r: Region, t: TenantTag) -> Self;                       // Rest 0x00 / 0xFF auffüllen
    pub fn collection(r: Region, t: TenantTag, c: CollTag) -> Self;
    pub fn cluster(r: Region, t: TenantTag, c: CollTag, cl: ClusterTag) -> Self;
    pub fn doc(r: Region, t: TenantTag, c: CollTag, cl: ClusterTag, d: DocTag) -> Self;
}
pub enum MatcherItem { Range(KeyRange), Exact(KeyBuf) }
pub struct Matcher(Vec<MatcherItem>);   // normalisiert: sortiert, überlappende Bereiche verschmolzen
impl Matcher { pub fn matches(&self, k: &KeyBuf) -> bool; pub fn normalize(self) -> Self; }
```

Begründung der Bereichsform: `AddFile` im Manifest führt `lo(65)`/`hi(65)` (F.4), Ledger und Gate-Log ebenfalls (F.9). Variable Präfixe sind verboten (D-10, HR-7).

### 4.2 Dokumentidentität

```rust
pub struct DocRef { pub subject: Option<SubjectRef>, pub key: DocKey }   // 7.2, D-70
pub struct DocHandle { cluster: ClusterTag, doc: DocTag }                 // opak, Suchtreffer
```

`SubjectRef` und `DocKey` sind längenbegrenzte Byte-Newtypes (nicht leer, Obergrenze als Konstante, Verstoß ⇒ `ConfigError`/`Limit`); sie erscheinen nie in Fehlertexten (I-11).

### 4.3 Löschtypen (Auszug)

```rust
pub enum EraseTarget {
    Document   { tenant: TenantRef, collection: CollectionRef, doc: DocRef },
    Subject    { tenant: TenantRef, subject: SubjectRef },
    Collection { tenant: TenantRef, collection: CollectionRef },
    Tenant     { tenant: TenantRef },
}                                                      // 7.3, wörtlich
pub enum RetentionCode { NoObligation, ObligationExpired, ObligationActive,
                         LegalClaimsPending, DataSubjectRequestedRestriction, OperatorOverride } // E-L12, abschließend
pub enum KeyCoverage { Full, Unlinked, None }                                                   // D-69
pub enum GateVariant { Erased, Restricted }                                                     // 11.3
pub enum PendingReason { Io, SlaExceeded, LayerFailed(LayerId), WitnessMissing, DiskReserve }   // 11.2
pub enum MeasureClass { Mk1, Mk2, Mk3 }                                                         // 12.1
pub enum ResidualRisk { FlashRemanence, FsJournalOrCow, WrappedKeyRemanence, FreedBlockRemanence,
    OldKeyslotRemanence, ProcessMemory, IndexTopologyInference, TextIndexUntilPurge, ReplicaWindow,
    ExternalCopies, ExternalProcessors, PseudonymCommitment, PurgeScalesWithDb, PurgeDeferred,
    KeyCompromise }                                                                             // Anhang D, 15 Einträge
```

`ResidualRisk` trägt `fn id(&self) -> &'static str` mit **festem** Namen (Kennung aus Anhang D). Jeder Eintrag hat zusätzlich eine statische Bedingung als Datentabelle (`fn condition(&self) -> RiskCondition`), die `ctx-verify` unabhängig nachbaut (AR-4).

### 4.4 Typisierter Zeuge (D-72, I-19) – Auflösung L-5 [E]

```rust
pub struct Unarmed; pub struct Armed(());               // Armed hat privates Feld
#[must_use] pub struct Scanner<S> { layer: LayerId, class: MeasureClass, method_id: u16,
                                    files: u64, bytes: u64, _s: S }
impl Scanner<Unarmed> {
    pub fn new(layer: LayerId, class: MeasureClass, method_id: u16) -> Self;
    /// einzige Brücke nach Armed; verlangt Beleg, dass das Kontrollmuster gefunden wurde
    pub fn arm(self, ev: ControlEvidence) -> Scanner<Armed>;
}
impl Scanner<Armed> {
    pub fn record(&mut self, files: u64, bytes: u64);
    /// matches != 0  ⇒  Err(Residue)  ⇒  Pending(WitnessMissing); nie ein Zeuge mit Treffern
    pub fn finish(self, matches: u64, duration: Duration, hash: [u8; 32]) -> Result<LayerWitness, Residue>;
}
pub struct LayerWitness { /* Felder privat; Lesezugriff über Getter; matches() == 0 per Konstruktion */ }
```

`ControlEvidence::observed(..)` ist öffentlich. Die Sperre gegen Missbrauch ist organisatorisch-maschinell: `tests/arch` erlaubt Aufrufe von `Scanner::arm` und `ControlEvidence::observed` nur in `contextra-erasure/src/measure` (siehe Datei 18).

### 4.5 Fehlermodell (7.5)

`Error` ist eine **geschlossene** Enum (`#[non_exhaustive]` für Verbraucher): `Config`, `Io` (inkl. `Poisoned`), `Corrupt`, `Auth`, `Erased`, `Restricted`, `NotFound`, `Conflict` (`IdCollision`, `Exists`), `Limit` (`Capacity`, `DiskReserve`, `Snapshots`, `QueryTooLarge`), `SnapshotRevoked`, `Cancelled`, `Refused`, `Internal`. Jede Variante hat einen **festen** `&'static str`-Text über `Display`; kein Feld trägt Nutzerinhalt (I-11, T-LOG-1). `Corrupt` ist immer fatal für das Objekt (P2). `Conflict::SubjectImmutable` entfällt (D-40). Offen: wohin `Dimension` gehört (L-10).

## 5. Implementierungsvorgaben

|ID|Vorgabe|Quelle|Test|
|---|---|---|---|
|TY-01|Keine Abhängigkeit auf Workspace-Crates; keine `async fn`, kein `std::fs`, `std::net`, `std::time::SystemTime::now`, `rand`, `tokio`.|AR-5, T-ARCH-1|T-ARCH-1|
|TY-02|Zielabhängigkeiten nur `thiserror` und `zeroize`. `serde`, `serde_json`, `bincode`, `ahash`, `blake3`, `sha2`, `bytes` nur mit Test-Nennung wieder aufnehmen (HR-12). Serialisierung der Berichte (kanonisches JSON) liegt in `erasure`, nicht hier.|P8, HR-12|`cargo tree`, T-CFG-1|
|TY-03|Schlüssellängen fest: `KeyBuf` = 78 B, `KeyBound` = 65 B; Konstruktoren nehmen nur typisierte Tags.|9.1, D-10|T-KEY-1|
|TY-04|`!commit_seq` = bitweises Komplement, big-endian; jüngste Version sortiert zuerst; `commit_seq()` ist ohne DEK lesbar.|9.1, D-74|T-KEY-1|
|TY-05|`Matcher` besteht nur aus Bereichen fester Länge und exakten Schlüsseln; Normalisierung verschmilzt überlappende Bereiche, ist idempotent und ändert die Trefferrelation nicht.|D-10|T-ISO-2 (Property)|
|TY-06|`ClusterTag::unassigned(doc)` = `0x00¹² ‖ doc_tag[0..4]`; `from_subject` ist die Identität auf dem `subject_tag`.|D-70|T-DOCREF-1|
|TY-07|`TagPreimage` setzt vor jedes Feld die `u32`-Länge (big-endian). Zwei verschiedene Feldaufteilungen ergeben nie dieselbe Vorform.|9.3|T-KEY-1|
|TY-08|`Seq` hat keinen Sentinelwert und keinen `u64::MAX`-Pfad; `Option<Seq>` modelliert „kein Snapshot“.|I-12, 8.7|T-SNAP-1 (mvcc)|
|TY-09|`ErasureId` ist idempotent aus Ziel-Commitment und Attestierung ableitbar; `types` speichert nur den Wert, die Ableitung (keyed) liegt in `erasure`.|7.4|T-API-2|
|TY-10|`Scanner<Armed>` ist nur über `arm` erreichbar, `LayerWitness` nur über `finish` mit `matches == 0`.|I-19, D-72|T-RAW-2, T-MUT-1|
|TY-11|`Secret` implementiert weder `Debug` noch `Display` noch `Clone`; `Drop` zeroisiert.|7.1|T-API-1 (trybuild), T-DEF-1|
|TY-12|Alle Fehlertexte sind `&'static str`; ein Test iteriert alle Varianten und prüft, dass kein Text einen Platzhalter enthält.|I-11, HR-4|T-LOG-1|
|TY-13|`RetentionCode` ist ohne Freitext und abschließend; `RetentionAttestation.attested_by` ist ein opaker `OperatorRef` (nie im Bericht im Klartext).|E-L12, 7.3|T-API-2|
|TY-14|Konstanten aus Anhang C (Labels, Größen) stehen genau einmal in `labels`/`limits`; `crypto` verwendet sie, `lab` dupliziert sie bewusst (AR-4).|Anhang C|T-FMT-1|
|TY-15|`FileKind` kennt genau die Werte 1..10 aus F.1; unbekannte Werte ⇒ `Corrupt`.|F.1, I-6|T-FMT-1|

## 6. Rückbau und Migration (Ist → Soll)

|Position|Maßnahme|WP|Nachweis|
|---|---|---|---|
|`DocId` (Feature `docid-128`, `DocIdWidth`)|entfernen; Ersatz `DocRef`/`DocTag`/`DocHandle`. In der SPEC-Zählung 95 Dateien/791 Treffer (F-31); Migration crate-weise von unten nach oben (`types` → `ports` → `store` → …).|WP-3.1|Grep `\bDocId\b` = 0 in Produktionscode|
|`types/saos.rs` (940 Z.), `AutoExtractionMode`, `auto-extraction-opt-out`|entfernen|WP-1.1|Grep = 0|
|`pub use erasure as witness`; Modul `erasure.rs` (`ScanMethod`, `RequestContext`, `SagaRecord`)|durch Löschtypen nach Abschnitt 4 ersetzen|WP-3.1|T-ARCH-3|
|`tombstone.rs` (`SeqBitTombstone`, `TombstoneSemanticsCheck`)|Tombstones sind `vlen = 0` (F.5); Kern-Konsumenten prüfen, sonst entfernen. Kein Seq-Bit im neuen Schlüsselformat.|WP-3.1|Grep|
|`types::types`, `domain`, `misc.rs`, `types.rs` + `types/`|auf Modulplan Abschnitt 3 umstellen (D-90)|WP-1.5|T-ARCH-7|
|`error_dto.rs`, `model_fingerprint.rs`, `retrieval_strategy.rs`, `schema.rs`, `tenant_scope.rs`, `importance.rs`, `budget.rs`, `filter.rs`|**je prüfen** (P8): Behalten nur bei belegtem Kernkonsumenten (`cargo tree`/Grep). Embedding-, Reranking- und Retrieval-Strategie-Bezüge entfallen (D-24, D-26). `filter.rs` bleibt, soweit `filter` in `search_*` (7.2) gebraucht wird.|WP-1.1|Entscheidungsliste im PR|
|Features `default`, `docid-128`, `auto-extraction-opt-out`|entfernen|WP-1.2|T-CFG-1|

## 7. Arbeitspakete

1. **WP-1.1/1.2:** Rückbau (oben), Grep-Nachweis 0 Treffer.
2. **WP-3.1 (rot):** Tests T-KEY-1, T-ISO-2, T-DOCREF-1, T-ARCH-1 schreiben; sie schlagen fehl, weil die Typen fehlen.
3. **WP-3.1 (grün):** `tags`, `key`, `refs`, `error`, `erasure`, `limits`, `secret`, `file` implementieren. Reihenfolge: `tags` → `key` → `refs` → `erasure` → `error`.
4. Abhängige Crates (`ports`, `erasure`-Skelett, D-96) können ab Abschluss von `refs` und `erasure` parallel starten.

## 8. Tests und Benchmarks

|ID|Prüft|
|---|---|
|T-KEY-1|Schlüsselkodierung: feste Längen, Sortierung, `!commit_seq`, Längenpräfixe der Tag-Eingaben|
|T-ISO-2|Matcher: Bereichs- und Exakt-Matcher treffen genau das Ziel (Property-Test, keine variablen Präfixe)|
|T-DOCREF-1|`DocRef` mit gleichem `key`, anderem Betroffenen ⇒ verschiedene Identität; kein Locator nötig|
|T-ARCH-1|`types` ohne I/O und `async` (Quellscan plus Abhängigkeitsbaum)|
|T-FMT-1|`FileKind`/`AlgIds`-Parsing: unbekannte Werte ⇒ `Corrupt`; Golden-Konstanten aus Anhang F.1|
|T-LOG-1|Alle Fehlertexte statisch, ohne Inhalt|
|T-API-2|`RetentionCode`-Abdeckung und `ErasureId`-Idempotenz (Typebene)|
|T-UNSAFE-1 (miri)|`miri` läuft auf `types` (SPEC 15.1)|
|Property|`proptest`: keine Panics bei beliebigen Byte-Eingaben für alle `TryFrom<&[u8]>` (Panics ausgeschlossen, P6)|

## 9. Abnahme

Definition of Done aus Index Abschnitt 8, zusätzlich: `cargo tree -p contextra-types` zeigt **keine** Workspace-Kante; T-ARCH-1 grün; `miri` grün; Grep `DocId`, `saos`, `AutoExtractionMode` ohne Treffer.

## 10. Offene Punkte

- L-5 (typisierter Zeuge über Crate-Grenzen), L-8 (`META`-Schlüsselformat), L-10 (`Dimension`).
- Welche der in Abschnitt 6 „je prüfen“ genannten Module ein Kernkonsument noch braucht, entscheidet der Grep-Beleg im Rückbau-PR.

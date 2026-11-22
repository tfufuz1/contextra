# Review: `contextra-storage`-Refaktorplan ⇄ Mnemos-Konzept

**Ziel:** Den Refaktorplan (`REFACTORING_PLAN(2).md`) und die Entwürfe in `Optimal-LLM-Datenbank.md` gegen mein Mnemos-Konzept prüfen und festlegen, wie der Code zu einem für LLMs optimalen Storage wird, auf dem später `crypto`, `vector`, `text`, `graph`, `rank`, `simd`, `kvcache`, `routing` und `cognition` als getrennte Crates aufsetzen.
**Stand:** 21.11.2026 · Repo `tfufuz1/acta` (flacher Klon, Stand beim Klonen)

> **Grenzen dieser Prüfung:** Gelesen habe ich beide Markdown-Dokumente vollständig, dazu `Cargo.toml`, Dateiliste und einige Stichproben im Altcode. Das Crate ist laut Plan nicht baubar, und im Repo gibt es kein Workspace-Root-`Cargo.toml` (Stichprobe bestätigt). Ich habe nichts kompiliert oder gemessen. Alle Leistungsangaben bleiben Hypothesen.

---

## 0. Ergebnis in zehn Zeilen

1. **Der Plan ist als Kern richtig und gut begründet.** Das sind Catalog als einzige Wahrheit, ein Redo-Frame je `Change`, `Fs`-Port mit Stromausfall-Simulator, `Erase` mit einem Durability-Punkt und Roll-forward sowie Guards statt Transaktionspuffer. Das sollte man unverändert übernehmen. Mein ursprünglicher Vorschlag, Prolly-Tree, CAS, Extents und Re-Layout komplett neu zu bauen, war für einen MVP zu groß und **wird zurückgenommen**: Der Kern liefert die schwierigen Teile (Atomarität, Crash-Sicherheit, Löschung) bereits geplant und getestet.
2. **Der Plan ist ein Metadaten-Kern, kein LLM-Storage.** Er schiebt in 17.1 fast alles Entscheidende nach oben: Token-Budget, Lokalität, KV-Cache, Trust, Versionsbäume. Ein Teil davon kann oben bleiben. Ein anderer Teil **kann oben nicht effizient nachgerüstet werden**, weil er das Dateiformat oder die Wartungsstruktur betrifft.
3. **Hauptbefund: Es fehlt die zweite Ebene.** Ein LSM mit Leveled Compaction (Schreibverstärkung ≤ 15 laut H-S3) ist für kleine, veränderliche Metadaten richtig. Für große, unveränderliche Inhalte (Texte, Embeddings, Tool-Ausgaben, Tensoren) ist er die falsche Struktur. Nötig ist **Wert-Trennung** (Blob-Dateien, vom Catalog verwaltet) im Kern. Der Plan nennt dafür nur „eine Objektschicht“, ohne sie zu spezifizieren.
4. **Sieben Dinge müssen ins Format v1, bevor F3 einfriert**, weil sie später einen Formatbruch bedeuten (Abschnitt 5, Checkliste P0).
5. **Zwei Wartungsjobs statt einem.** „Ein Job je Instanz“ (SD-16) blockiert Flush hinter einer langen Compaction oder einem Index-Bau. Bei LLM-Lastspitzen führt das zu Schreibstopps.
6. **Abgeleitete Indizes brauchen mehr als `subscribe` + `history`.** Ohne **persistente Konsumenten-Cursor** (die zugleich Holds sind) kann der Kern die Versionen löschen, die ein Index zum Aufholen braucht.
7. **Token-Budget lässt sich schon im Kern abbilden, ohne Tokenizer zu kennen:** Blockaggregate (`weights`) und ein Wert-Header, den der Kern ohne Wert lesen kann. Das ersetzt mein „augmentiertes Token-Prolly“ und kostet fast nichts.
8. **Branches** laufen zuerst als **Overlay-Branches** auf `Guard`/`Hold`/`history` (Stufe A). Ein inhaltsadressierter Commit-DAG (Prolly, Stufe B) kommt nur, wenn Sync, Audit oder Replikation es verlangen.
9. **Löschung ist die gefährlichste Wechselwirkung mit den künftigen Crates.** Abgeleitete Segmente (Vektor, Text) enthalten Klartext-Rückstände. `Erase` und `ErasureOracle` müssen sie von Anfang an abdecken (Abschnitt 5, P1-4).
10. **Reihenfolge:** Kern nach Plan bis F5, dann sofort ein **vertikaler Schnitt** (Cell → Text → flacher Vektor → `assemble`), bevor die Kern-API in F9 eingefroren wird.

---

## 1. Einordnung der Entwürfe in `Optimal-LLM-Datenbank.md`

Die Datei enthält drei Entwürfe (Claude „Engram“, Gemini „AetherStore“, Grok „AetherStore“). Als Grundlage für den Code taugen sie nur als Ideenquelle:

| Idee | Quelle | Urteil | Begründung |
|---|---|---|---|
| `assemble` mit Budget-Packing, LOD, Handles, Fehler als Handlungstext | Claude | **übernehmen** (oben, Crate `rank`/`cognition`) | deckt sich mit Mnemos 8; Fehler-`hint()` ist im Plan schon in SD-25 |
| Taint-Labels, Crypto-Shredding, Lineage | Claude | **übernehmen** (Header im Wert, Kern bleibt blind) | siehe Abschnitt 4 |
| Affinitäts-Clustering per Co-Retrieval | Claude, Mnemos | **übernehmen, aber über Indirektion** | der Kern darf nach SD-26 nicht umordnen; Re-Layout über Locator-Space (Abschnitt 7) |
| Prolly-Tree als Hauptindex | Claude, Mnemos | **zurückstellen** (Stufe B) | siehe P2-1 |
| Context Slabs mit festen 4-KiB-Seiten (64 Token, 768-dim Int8 pro Seite, Inline-KV) | Gemini | **verwerfen** | feste Dimension und feste Tokenzahl pro Seite passen nicht zu mehreren Embedding-Modellen und variabler Textlänge; schlechte Raumnutzung |
| „Kein WAL nötig“, Commit per CAS auf `AtomicU64` im Superblock | Gemini | **verwerfen** | der Beispielcode hält den Zeiger nur im RAM. Dauerhaft wird ein Superblock-Eintrag erst nach `fsync` und mit Prüfsumme. Das ist dieselbe Garantie wie der Catalog-Edit im Plan, nur schlechter geprüft. Die Beispielfunktionen sind `unimplemented!`/Stubs. |
| GPUDirect-Storage, `O_DIRECT`, `io_uring` | Gemini | **später, über `Fs`-Erweiterung** | siehe P0-5; nicht in den Kern, nicht in F1 |
| Locality Groups, Hot-Working-Set, Hardware-Offload | Grok | **als Konzept ja**, Details verwerfen | kein Format, keine Konsistenzaussage; „2PC über Locality Groups“ ist durch den gemeinsamen Redo-Frame überflüssig |
| Branch-aware ANN mit `CommitMask` (Roaring) | Gemini | **übernehmen** (Crate `vector`) | entspricht meinen Deletion Vectors; passt zu Overlay-Branches |

---

## 2. Kernbefund: zwei Ebenen

```
┌────────────────────────────────────────────────────────────────────────────────┐
│ oben: Domäne            cell · rank · cognition · routing · kvcache · server   │
├────────────────────────────────────────────────────────────────────────────────┤
│ Indizes (getrennte Crates)   vector · text · graph      (nutzen simd)          │
│    ↑ Extension-Blöcke (an Runs gekoppelt)  ↑ Segmente (entkoppelt, Watermark) │
├────────────────────────────────────────────────────────────────────────────────┤
│ KERN  contextra-storage  (Plan, mit den Änderungen aus Abschnitt 5)            │
│                                                                                │
│   Meta-Ebene (LSM)                    Daten-Ebene (neu im Kern)                │
│   · kleine, veränderliche Werte       · Blob-/Pack-Dateien, append-only        │
│   · Header + Verweise (BlobRef)       · große, unveränderliche Inhalte         │
│   · refs, idem, alias, loc, work      · Texte, Embeddings, Tensor-Blöcke       │
│   · Guards, Views, Holds, Erase       · GC nach Lebendanteil, Erase-fähig      │
│                                                                                │
│   gemeinsam: Catalog (Dateiarten Run | Redo | Blob | Segment), Graveyard,      │
│   Verwaisten-Suche, Roll-forward, FaultFs-Absturzmatrix                        │
├────────────────────────────────────────────────────────────────────────────────┤
│ Ports (kein Code, nur Traits): Fs · Clock · Entropy · Cipher · KeyProvider ·   │
│                                 ColdTier · AccessSink · Checksum               │
└────────────────────────────────────────────────────────────────────────────────┘
```

**Warum die Daten-Ebene in den Kern gehört und nicht in eine „Objektschicht“ darüber:** Lebenszyklus einer Datei (bauen → dauerhaft → Catalog-Edit → sichtbar → Graveyard → Unlink), Absturz-Aufräumen und `Erase` sind genau das, was der Plan in F2 bis F7 mit großem Aufwand absichert. Eine zweite, parallele Dateiverwaltung in einem anderen Crate würde alle diese Arbeit doppeln und die Löschgarantie (E3 bis E6) für Blobs aushebeln. Inhaltsadressierung (BLAKE3) bleibt dagegen **außerhalb**: Der Kern speichert Blobs unter einer vom Aufrufer gelieferten Id und kennt keinen Hash.

---

## 3. Abgleich: Mnemos-Idee → Plan

| # | Mnemos-Idee | Stand im Plan | Urteil | Maßnahme |
|---|---|---|---|---|
| 1 | Cell mit Facetten (Inhalt, Tokens, Embeddings, Zoom, Kanten, Zeit, Trust) | nicht vorhanden (opake Werte) | richtig oben | Crate `cell`, Schlüsselschema Abschnitt 7 |
| 2 | Token-native Speicherung, Budget ohne Inhaltszugriff | `estimate` in Bytes/Einträgen | **Lücke im Format** | P0-1: `weights` + Header |
| 3 | Zoom-Leiter, Zoom-Knapsack | – | richtig oben (`rank`) | Kern liefert `scan_page`/`estimate` mit Gewichten |
| 4 | Hot-Head/Cold-Tail-Extents | Werte inline im DataBlock | **Lücke** | P0-1 + P0-2: Header im Run, Körper im Blob |
| 5 | Identität ≠ Ort, gelerntes Layout | SD-26: Kern ordnet nie um | **teilweise richtig** | Locator-Indirektion oben; Kern braucht P0-5 `AccessSink` |
| 6 | O(1)-Fork, Merge, Konflikte als Daten | Views/Holds/Guards, keine Branches | richtig oben, aber Kern-Bausteine fehlen teilweise | P1-3, P2-1 |
| 7 | Bitemporal | Stamp = Transaktionszeit, Rest oben | **richtig** | Gültigkeitszeit im Header/Schlüssel |
| 8 | Derivation Cache + Lineage | `Extension`, `subscribe`, `history` | **Lücke bei Konsum** | P0-3: Cursor + Segmente |
| 9 | Sleep-Cycle-Work-Queue | nicht vorhanden | richtig oben (`cognition`) | Space `work` + Lease über `Guard` |
| 10 | Trust/Taint | oben | richtig | im Header (1 Byte), Filter-Pushdown P1-2 |
| 11 | Crypto-Shredding je Subjekt | je **Space** (12.2), je Subjekt oben | **Risiko** bei abgeleiteten Daten | P1-4 |
| 12 | Capabilities (Macaroons) | nicht vorhanden | richtig oben (`crypto`) | – |
| 13 | Idempotenz | `Guard::absent` | **richtig**, es fehlt nur Ablauf | P1-5 TTL |
| 14 | Durability-Stufen je Transaktion, flüchtiges Arbeitsgedächtnis | nur instanzweit; `memory-only` entfällt | **Lücke** | P1-1 |
| 15 | Retention/Vergessen als Zoom-Down | nur Versions-Retention | **Lücke** | P1-5 `CompactionFilter` |
| 16 | Handles, handlungsleitende Fehler | `hint()`, `retryable()` | **richtig** | Handle-Space oben |
| 17 | Prefix-/KV-Cache-Store | Nicht-Ziel | richtig oben (`kvcache`), braucht Kern-Hilfen | P0-5 (`Fs`), P0-2 (Blob) |
| 18 | Tiering RAM/NVMe/S3 | `tier`, `ColdTier` (F10) | **richtig** | vorziehen, gilt auch für Blobs |
| 19 | Deterministische Simulation | `FaultFs`, Modelltest | **stärker als mein Konzept** | übernehmen, erweitern (P1-4) |
| 20 | Sync per Commit-Frontier (Git-artig) | Nicht-Ziel | richtig, nur mit Stufe B | P2-1 |

---

## 4. Was am Plan **nicht** geändert werden sollte

Diese Entscheidungen decken sich mit der Mnemos-Linie oder sind besser:

- **Catalog-Edit als einziger Wahrheitspunkt** und die Reihenfolge „Datei dauerhaft → Edit dauerhaft → sichtbar → löschen“ (Leitprinzip 3, Beilage B). Das ist die Grundlage für alles Weitere.
- **Ein Redo-Frame je `Change`**, neues Segment je Prozessstart, Heilen nur im letzten Segment.
- **`Fs`-Port mit `FaultFs`** als Qualitätstor. Absturz an *jedem* E/A-Index ist das, was mein Konzept mit „deterministischer Simulation“ nur beschrieben hat.
- **Spaces als getrennte LSM-Bäume** bei gemeinsamem Redo: genau die richtige Löschgrenze.
- **`Erase` mit Roll-forward**, `Erased` statt „leer“ (12.6), `Cryptographic` als einzige belastbare Stufe auf SSDs. Die ehrliche Garantietabelle (12.2) ist besser als meine Formulierung.
- **Seitenweises Lesen mit `ResumeToken`** statt Offsets, strukturierte Fehler, `Busy{retry_after}`.
- **Zero-Dependency-Kern, synchron.** Für die künftigen Crates ist das ein Vorteil: Sie hängen an Traits, nicht an einer Laufzeit.
- **Kein Merge-Operator, keine Kompression in v1** (SD-12/13). Reservierte Bytes genügen.

Zwei kleine Korrekturen am Befund (B-10): In `wal/core.rs` stehen mindestens **sechs** statische Testschalter, nicht drei (`FAIL_APPEND_FOR_TX`, `DELAY_APPEND_FOR_TX`, `DELAY_APPEND_MS`, `FAIL_TRUNCATE_ONCE`, `FAIL_APPEND_AFTER_PARTIAL_BYTES`, `FAIL_APPEND_PARTIAL_ONCE`). Und `memmap2` wird im Altcode tatsächlich im WAL-Replay und im SSTable-Reader benutzt, die Entscheidung „kein `mmap`“ ist also eine echte Änderung (sie ist vertretbar, Leseweg über `read_at`).

---

## 5. Änderungen am Plan (Patch-Liste)

Priorität: **P0 = muss vor F3 im Format stehen**, P1 = vor dem vertikalen Schnitt, P2 = Entscheidung nötig, Umsetzung später.

### P0-1 Gewichte und Wert-Header (Token-Budget ohne Tokenizer)

**Problem:** `estimate`/`scan_page` kennen nur Bytes und Einträge. Ein LLM-Budget ist in Token. Wer Token erst nach dem Lesen zählt, liest zu viel und kann nicht vorab planen.

**Änderung:**

- `SpaceOptions::weights(k)` mit k ≤ 4. Der IndexBlock führt je Block neben `entries` und `bytes` **k Summen** (u32). Gewicht 0 ist immer „Bytes“. Gewicht 1 und folgende sind frei, typisch Token je Tokenizer-Familie.
- Trait `Weigher` (zustandslos, je Space registriert): `fn weigh(&self, key: &[u8], header: &[u8]) -> [u32; K]`. Der Kern ruft ihn beim Run-Bau und beim Scan auf.
- `SpaceOptions::header_len(n)`: die ersten `n` Bytes jedes Wertes sind der **Header** (Typ, Trust, Flags, Zeitbereich, Token-Zahlen, Zoom-Level). Der Kern kann ihn lesen, ohne den Wert zu laden, sobald der Körper ausgelagert ist (P0-2).
- `PageLimit::weight(idx, n)` neben `bytes` und `entries`; `estimate` liefert `Estimate{entries, bytes, weights[k]}`.

**Kosten:** wenige Bytes je Block, ein Optional-Feld im `MetaBlock`. **Kein Tokenizer im Kern.**
**Gilt für:** `rank`, `cognition`, `kvcache` (Block-Token-Summen), `text` (Fenster um einen Treffer als Bereichs-Scan mit Gewicht).

### P0-2 Wert-Trennung (Blob-Dateien)

**Problem:** Werte bis 1 MiB inline in 4-KiB-Blöcken heißt: Ranking-Scans lesen Volltexte mit, und Leveled Compaction schreibt unveränderliche Inhalte bis zu 15-mal neu. Beides trifft LLM-Daten besonders.

**Änderung:**

- `SpaceOptions::separate_body(threshold)`: ab `threshold` Bytes (Vorschlag 4 KiB) kommt der Wertkörper in eine **Blob-Datei** (append-only, Extent-weise, 4-KiB-ausgerichtet), der Run enthält nur Header + `BlobRef{file_id, offset, len, crc}`. Neuer Tag-Kind `PutRef = 5`.
- Der Schreibpfad hängt Blobs **vor** dem Redo-Frame an und synchronisiert sie (Reihenfolge-Vertrag aus Leitprinzip 3); der Redo-Frame enthält bereits den `BlobRef`.
- Blob-Dateien sind **Catalog-Dateien** (P0-4) mit Lebendzähler je Datei. GC schreibt Blobs mit < 40 % Lebendanteil um (nur Kopieren, kein Sortieren).
- `Erase` (E3 bis E6) schließt Blob-Dateien ein. Das ist der Grund, warum diese Ebene im Kern liegen muss.
- Der Header-Leseweg liefert **Hot-Head**, der Blob-Zugriff **Cold-Tail**: Ranking (Header + Zusammenfassung < 1 KiB) bleibt in den Runs, Volltext kommt per Zeiger. Das ist mein Extent-Layout, nur mit den Mitteln des Plans.

### P0-3 Persistente Konsumenten-Cursor und entkoppelte Segmente

**Problem:** AR-4 sagt, abgeleitete Indizes holen per `history` ab ihrer Stamp-Position auf. Nichts verhindert aber, dass die Untergrenze `floor` darüber hinaus steigt und die Versionen weg sind. Zusätzlich passt der `Extension`-Block (an einen Run gekoppelt) für Vektorgraphen schlecht: Sie würden bei **jeder** Compaction neu gebaut.

**Änderung (zwei Mechanismen, klar getrennt):**

1. **Konsumenten-Cursor** im Catalog: `Consumer{name, consumed_stamp, max_lag_bytes}`. `floor = min(Views, Holds, Consumer.consumed_stamp)`. Überschreitet ein Konsument `max_lag_bytes`, wird er vom Kern **verworfen** (nächster Zugriff `HistoryPruned`), und die obere Schicht baut neu auf. So kann ein toter Indexbauer den Speicher nicht dauerhaft aufblähen.
2. **Segment-Register** im Catalog: `Segment{kind u16, space, stamp_from, stamp_to, file_id}`. Ein Segment ist eine eigene Datei (eigenes Format, vom Crate `vector`/`text`/`graph` geliefert), im Catalog lebendig wie ein Run, im Graveyard beim Ersetzen, in `Erase` berücksichtigt. Der **Lesepfad** eines Index fragt `Segmente ∪ history(stamp_to, view.stamp]`. Das ist mein „Delta-Index + Segmente“, ohne dass der Index hinter der Sicht zurückbleibt.

**Zuordnung:**

| Index | Mechanismus | Begründung |
|---|---|---|
| Postings, Bloom, Zähler, kleine Hilfsstrukturen | `Extension` je Run | billig, zerlegt mit den Daten, merge einfach |
| Volltext (BM25) | Extension je Run **oder** Segment | beides tragfähig; Start mit Extension, da Lucene-artig |
| Vektor (ANN) | **Segment** | Graphbau teuer, Lokalität folgt Cluster, nicht Schlüssel |
| Graph (CSR/PPR) | Space (Kanten) + optional Segment | Adjazenz-Scan genügt, Segment nur für schnelle Mehrschritt-Abfragen |

### P0-4 Catalog mit allgemeinen Dateiarten

`FileKind ∈ {Run, Redo, Blob, Segment(kind), Cold}` statt fest „Run“. `add_files/remove_files` im Edit, Verwaisten-Suche, Graveyard und Löschprotokoll arbeiten auf `FileKind`. **Zwei Wartungs-Slots:** `urgent` (Erase, Flush, Blob-Schreib-Abschluss) und `background` (Compaction, Blob-GC, Segment-Bau, Cleanup). Der Plan (SD-16) sieht nur einen Slot vor. Eine 64-MiB-Compaction würde sonst Flush blockieren, bis `frozen ≥ 4` zu `Err(Busy)` führt. Alternativ genügt kooperatives Abbrechen an Ausgabe-Grenzen, aber getrennte Slots sind einfacher zu beweisen.

### P0-5 Erweiterungen der Ports (klein, aber früh)

- `File::read_batch(&self, reqs: &mut [ReadReq]) -> io::Result<()>` mit Standardimplementierung als Schleife. Eine `io_uring`-Implementierung (separates Crate) kann sie überschreiben. Ein Recall mit 200 Kandidaten braucht Batch-Reads, sonst bleibt die Latenz an Einzelaufrufen hängen.
- `OpenMode::Direct` und ein ausgerichteter Puffer-Typ (`AlignedBuf`). Der Kern nutzt ihn nicht, `kvcache` und Blob-Dateien profitieren.
- `trait Checksum` im `Env` (Standard: portable CRC32C). Beschleunigte Varianten brauchen `unsafe` und liegen deshalb in `simd`, nicht im Kern (`forbid(unsafe_code)` bleibt).
- `trait AccessSink` im `Env`: gesampelter Lesetrace `(space, run_id, block_no, Schlüsselpräfix, bytes)`. Nur der Kern kennt Runs und Blöcke. Ohne diesen Strom gibt es kein Co-Retrieval-Clustering und keine Evaluation (Mnemos 5.3, 16.2). Standard: Null-Senke.
- `trait ColdTier` wie im Plan, aber **auch für Blob-Dateien**, und früher als F10, sobald Blobs existieren.

### P1-1 Durability je `Change` und flüchtige Spaces

`Change::durability(Always | Interval | Never)` überschreibt den Instanzwert. `SpaceOptions::ephemeral()`: nur Memtable, kein Redo, keine Runs, verfällt beim Schließen. Das ersetzt den gestrichenen Cargo-Schalter `memory-only-storage` und passt zum Arbeitsgedächtnis und zu Zugriffsstatistik (nicht versionierter Sidecar). Ein `Change` über dauerhafte und flüchtige Spaces ist verboten (`Err(Forbidden)`), sonst bricht die Atomaritätsaussage.

### P1-2 Filter-Pushdown im Cursor

`scan_page(…, Filter)` mit `trait Filter { fn keep(&self, key: &[u8], header: &[u8]) -> bool }`. Wirkung: Trust-Mindeststufe, Gültigkeitszeit, Typ und Löschkennzeichen werden gegen den Header geprüft, ohne den Körper zu laden. Das ist Voraussetzung für `min_trust` und `valid_at` bei 6-stelligen Kandidatenmengen.

### P1-3 Branch-Bausteine

Für Overlay-Branches (P2-1) fehlen zwei kleine Dinge: `Guard::RangeUnchangedSince(space, range, stamp)` (Konfliktprüfung beim Merge ohne Vollscan) und `view_at` + `Hold` mit **Eigentümer** (Branch-Name), damit ein verworfener Branch seinen Hold zuverlässig freigibt. Beides ist eine Variante bestehender Primitive.

### P1-4 `Erase` und abgeleitete Daten

- `ErasureOracle` (F7) durchsucht **alle Dateiarten** inklusive Blobs und Segmente nach dem Muster. Für Vektoren zusätzlich eine Rückrechnungsprüfung (Embedding des gelöschten Textes darf nicht mehr als Nachbar auftauchen).
- Das Segment-Format verlangt `fn rewrite_without(range)`; ohne diese Methode ist ein Segment nicht `Erase`-fähig und wird vom Kern bei Überlappung **komplett** verworfen und neu gebaut.
- **Löschdomänen:** Embeddings und Volltext-Postings leaken Inhalte. Wer je Subjekt crypto-shredden will, braucht Segmente **je Löschdomäne** (Tenant oder Subjektklasse), nicht je Namespace. Das ist eine Entwurfsentscheidung für `vector`/`text` (Abschnitt 8, O-2).
- **Erase-Bündelung:** Jeder `Erase` erzwingt laut Plan (E2) eine Flush-Barriere über **alle** Spaces. Bei häufigen DSGVO-Anfragen blockiert das. `Erase::batch` mit einem Zeitfenster fasst mehrere Bereiche zu einer Barriere zusammen.

### P1-5 `CompactionFilter` (Ablauf, Vergessen, Zoom-Down)

```rust
pub trait CompactionFilter: Send + Sync {
    /// Nur für Versionen mit Stamp <= floor (Sichten >= floor bleiben unberührt).
    fn decide(&self, key: &[u8], header: &[u8], stamp: Stamp, now_ms: u64) -> Decision;
}
pub enum Decision { Keep, Drop, ReplaceBody(Option<BlobRef>) } // None = Körper entfernen, Header behalten
```

Damit gibt es TTL für Idempotenz-Schlüssel, Arbeitsgedächtnis und spekulative Branches, und „Zoom-Down“ (Volltext weg, Zusammenfassung und Header bleiben) ohne eigene Maschinerie. Der Filter ist rein und wird im Modelltest gegen das Referenzmodell mitgeprüft. Er ist **kein** Merge-Operator (SD-12 bleibt gültig), weil er nur entfernt oder verkürzt.

### P2-1 Versionierung in zwei Stufen

| | Stufe A: Overlay-Branch | Stufe B: Commit-DAG (Prolly) |
|---|---|---|
| Bausteine | Overlay-Space, `Hold(base_stamp)`, `Guard::RangeUnchangedSince`, `history` | inhaltsadressierte Knoten im Blob-Space, `refs`-Space mit `Guard::equals` |
| Fork | O(1) (Space + Hold) | O(1) (Ref kopieren) |
| Merge | Replay des Overlays als ein `Change` mit Guards | strukturell über Hash-Vergleich |
| Diff | `history(base, head]` | O(Änderung) zwischen beliebigen Commits |
| Dedup/Sharing über Branches | nein | ja |
| Sync/Replikation, Audit-Kette | nein | ja |
| Schwäche | lange Branches halten Versionen (Platzverstärkung) | Schreibverstärkung, mehr Code |
| Wann | **Phase V1**, deckt Agenten-Versuche ab | erst bei nachgewiesenem Bedarf (Sync, Audit) |

Der Kern bleibt für beide unverändert. Wichtig ist nur, die beiden Zeitbegriffe sauber abzubilden: Commit → Stamp wird im Commit-Satz gespeichert, Branch-Refs sind GC-Wurzeln, und der Kern kennt weiterhin nur den Stamp-`floor`.

### Format-v1-Checkliste (alles, was vor F3 feststehen muss)

| # | Eintrag | Ort |
|---|---|---|
| 1 | `weights[k]` je IndexBlock-Eintrag, `k` im MetaBlock | Run |
| 2 | `header_len` je Space | Catalog `spaces_add`, MetaBlock |
| 3 | Tag-Kind `PutRef` und `BlobRef` im Wertformat | Run, Redo |
| 4 | `FileKind` und Blob-/Segment-Records | Catalog |
| 5 | `Consumer`-Records und `max_lag_bytes` | Catalog |
| 6 | Durability je Frame (`Always/Interval/Never`) und `ephemeral`-Flag | Redo-Frame, Catalog |
| 7 | `tier` für Blob und Segment (nicht nur Run) | Catalog |

Alle sieben sind klein. Nach F3 würde jede einzelne eine Formatversion kosten, obwohl SD-04 genau eine pro Dateiart verspricht.

---

## 6. Ziel-Workspace

Name-Präfix wie im Plan: `contextra-*`. (Die bisherigen Abhängigkeiten `contextra-ids`, `-env`, `-sys`, `-mvcc` fehlen im Repo und entfallen, `contextra-crypto` bleibt als Name erhalten.)

| Crate | Aufgabe | hängt ab von | Anmerkung |
|---|---|---|---|
| `contextra-ports` | Traits: `Fs`, `File`, `Clock`, `Entropy`, `Cipher`, `KeyProvider`, `ColdTier`, `AccessSink`, `Checksum`, `Extension`, `Weigher`, `Filter`, `CompactionFilter` | nichts | kein Code, nur Schnittstellen; Kern und Zusatzcrates sehen dieselben Typen |
| `contextra-storage` | Kern aus dem Plan: Redo, Run, Blob, Catalog, Engine, Erase, Testing (`FaultFs`) | `ports` | zero-dep |
| `contextra-simd` | Distanzen (f32/f16/i8/binär), Bit-Packing, Popcount, beschleunigtes CRC32C | `ports` (für `Checksum`) | einziges Crate mit `unsafe` für SIMD; portable Rückfallimplementierung, Miri |
| `contextra-crypto` | AEAD-`Cipher`, `KeyProvider`, Schlüsselhierarchie, Subjekt-Keyring, Capabilities | `ports` | Krypto-Crates nur hier; Nonce-Eindeutigkeitstests |
| `contextra-cell` | `CellId`, `ChunkId`, `Trust`, Header-Layout, Zoom, `Handle`, `Tokenizer`-Trait, Zeitmodell, Schlüsselschema | `ports` | die gemeinsame Sprache aller oberen Crates |
| `contextra-text` | Chunker, BM25-Postings (`Extension`/Segment), Tokenizer-Implementierungen, Token-Zählung | `cell`, `storage` | Tokenizer hinter Features |
| `contextra-vector` | Segmente (flach, IVF, Graph), Delta-Suche über `history`, Quantisierung, mehrere Embedding-Lanes | `cell`, `storage`, `simd` | Löschdomänen-Segmente |
| `contextra-graph` | Adjazenz-Space, k-Hop mit Fan-out-Budget, PPR | `cell`, `storage` | kommt zuerst ohne Segment aus |
| `contextra-rank` | Fusion (RRF/gewichtet), MMR, **Zoom-Knapsack**, Packer, `explain` | `cell`, `ports` | rein, kein Speicherzugriff; testbar mit Zufallskandidaten |
| `contextra-kvcache` | Präfix-Radix über Token-Blöcke, Blob-Verwaltung, kostenbasierte Eviction, Tiering | `cell`, `storage`, `simd` | Adapter getrennt: `-candle`, `-onnx`, `-ollama` |
| `contextra-routing` | Anfrage-Plan: welche Spaces/Indizes/Modelle/Tiers, Kostenmodell, Fallback | `cell`, `rank` und die Index-Crates | siehe Annahme unten |
| `contextra-cognition` | Speicherebenen, Konsolidierung (Work-Queue), Salience/Vergessen, Lineage-Pflege, Trust-Propagation, Bitemporalität | alles darunter | spricht über Aufgaben mit externen Modellen, ruft selbst kein LLM |
| `contextra-server` | `assemble`-API, MCP, gRPC, Multi-Tenant | alles | die einzige Stelle mit `tokio` |

**Abhängigkeitsregel** (per `tests/layering.rs` erzwingen, wie im Plan 4.2): `ports` ← `storage`, `simd`, `crypto`, `cell` ← Index-Crates, `rank` ← `routing`, `cognition` ← `server`. Index-Crates kennen einander nicht; Fusion geschieht ausschließlich in `rank`.

**Annahmen, die du bestätigen solltest:**

- **`routing`**: Ich nehme an, es meint *Anfrage- und Modell-Routing* (welche Indizes, Embedding-Modelle, Tiers, Summarizer pro Aufgabe). Falls Netz-/Shard-Routing gemeint ist, wird der Zuschnitt ein anderer.
- **`kvcache` mit Ollama:** Nach meinem Kenntnisstand gibt Ollama über seine HTTP-Schnittstelle keinen Zugriff auf Attention-Tensoren. Ein Adapter kann dort höchstens Präfix-Digests und eine Ordnung liefern (Trefferrate beim Prompt-Cache erhöhen). Echte KV-Persistenz ist über `candle` (Tensoren im eigenen Prozess) oder über ONNX-Decoder mit expliziten `past_key_values` plausibel. **Das ist vor dem Entwurf zu prüfen.**
- **Tokenizer-Kopplung:** KV-Blöcke sind nur mit exakt demselben Tokenizer und Modell wiederverwendbar. Der Schlüssel enthält deshalb `(model_id, tokenizer_id, Hash-Kette)`.

---

## 7. Datenmodell auf dem Kern (Spaces und Schlüssel)

Faustregel: **wenige** Spaces (≤ ~12 je Instanz). Jeder Space hat eigene Memtable und Run-Verzeichnis, und der gemeinsame Redo hält durch den langsamsten Space Segmente fest (`redo_lag_bytes`). Viele Mini-Spaces (z. B. einer je Lane und Mandant) wären ein Rückschritt.

| Space | Schlüssel | Wert | Optionen |
|---|---|---|---|
| `cell.hdr` | `ns │ cluster │ t_bucket │ cell_id` | Header (Typ, Trust, Zeit, Token-Zahlen) + Zoom L2/L1 | `header_len`, `weights(2)`, `prefix_filter`, `locality_prefix` |
| `cell.body` | `ns │ cell_id` | Volltext / Token-IDs / Roh-Embedding | `separate_body(4 KiB)` |
| `cell.loc` | `cell_id` | Platzierungsschlüssel in `cell.hdr` | klein, im Cache |
| `refs` | `branch` | `commit/stamp` | CAS über `Guard::equals` |
| `idem` | `run_id │ step_id │ key` | `commit_stamp` | `CompactionFilter` (TTL) |
| `alias` | `branch │ handle` | `cell_id` | Kurz-IDs je Branch |
| `edge.out` / `edge.in` | `src │ type │ dst` / umgekehrt | Gewicht, Zeit | `prefix_filter` |
| `trace` | `run │ step │ seq` | Episoden-Eintrag | **append-only** |
| `work` | `kind │ prio │ id` | Aufgabe | Lease über `Guard` |
| `stats` | `cell_id` | Zähler (nicht versioniert) | `ephemeral`, periodisch gesichert |
| `kv.prefix` | `model │ tokenizer │ hash` | `BlobRef` | `separate_body` |

**Eine `remember`-Operation** ist ein `Change`: `Guard::absent(idem, key)`, `put(cell.hdr)`, `put(cell.body)`, `put(cell.loc)`, `put(edge.out)`/`put(edge.in)`, `put(idem)`. Atomar über alle Spaces, mit Wiederholungssicherheit.

**Re-Layout (gelerntes Layout) ohne Kernänderung:** Der Optimierer berechnet neue Cluster aus `AccessSink`-Daten, schreibt ein `Change{delete(hdr alt), put(hdr neu), put(loc)}` mit `Guard::UnchangedSince`. Die Identität (`cell_id`) bleibt, nur der Platzierungsschlüssel ändert sich. Das ist langsamer als mein ursprüngliches „nur der Locator ändert sich“, aber es braucht keine Kernänderung und erfüllt SD-26. Wer später echtes Locator-Re-Layout will, bekommt es in der Blob-Ebene (Blob-Umschreiben ohne Sortierung).

**Warum ein Space je Lane nur für Header und Körper:** Die Trennung `cell.hdr`/`cell.body` ist das, was P0-2 intern über `separate_body` automatisch macht. Als eigene Spaces brauchst du sie nur, wenn Header und Körper unterschiedliche Lösch- oder Aufbewahrungsregeln haben. Sonst genügt ein Space mit Wert-Trennung.

---

## 8. Phasenplan (angepasst)

Die Phasen F0 bis F9 aus dem Plan bleiben; eingefügt oder verändert wird:

| Phase | Änderung |
|---|---|
| **F0** | Zusätzlich: `contextra-ports` als eigenes Crate, `FileKind`/`weights`/`header_len`/`Consumer` in `SPEC.md` festschreiben (Checkliste Abschnitt 5). Format-Review **vor** F3. |
| **F1** | `Fs` mit `read_batch`, `OpenMode::Direct`, `AccessSink`, `Checksum`. `FaultFs` kennt Blob-Dateien. |
| **F3** | Run mit `weights`, `header_len`, `PutRef`. Roundtrip- und Bitflip-Tests laufen mit und ohne Blob. |
| **F4/F5** | Catalog mit `FileKind`, Consumer, zwei Wartungs-Slots; `Durability` je `Change`, `ephemeral`; Filter-Pushdown. |
| **F6** | Compaction **plus** Blob-GC und `CompactionFilter`. |
| **F7** | `Erase` über Runs, Blobs, Segmente; Orakel erweitert; `Erase::batch`. |
| **V1 (neu, nach F5, parallel zu F6/F7)** | **Vertikaler Schnitt:** `cell` + `text` (BM25 als `Extension`) + `vector` (flach, SIMD) + `rank` (RRF + Zoom-Knapsack) + Overlay-Branches + `assemble` im Speicher-Server. Ziel: die Kern-API an echter Nutzung prüfen, **bevor** sie in F9 eingefroren wird (A-10 im Plan beschreibt genau dieses Risiko für `Extension`). |
| **V2** | `vector`-Segmente (Graph/IVF), `cognition` (Konsolidierung, Salience), `crypto`-Keyring je Subjekt, `AccessSink`-gestütztes Re-Layout. |
| **V3** | `kvcache` mit `candle`/ONNX-Adapter, `routing`, `ColdTier` für Blobs. |
| **V4 (optional)** | Stufe B (Commit-DAG, Sync). |

---

## 9. Risiken und offene Entscheidungen

| ID | Risiko / Frage | Einschätzung | Gegenmaßnahme |
|---|---|---|---|
| R-1 | **Plan zu groß für späten Nutzen:** F0 bis F9 sind rund 5 bis 6 Monate, bevor ein einziges LLM-Feature läuft | hoch | vertikaler Schnitt V1 direkt nach F5; F6/F7 parallel |
| R-2 | **Spaces-Explosion** durch zu feine Lanes oder je Mandant | mittel | Budget (~12), `redo_lag_bytes` überwachen, Mandanten nicht als Spaces |
| R-3 | **Erase = globale Flush-Barriere** und teuer bei vielen Löschanfragen | mittel | `Erase::batch`, Löschdomänen (O-2) |
| R-4 | **Leveled-Schreibverstärkung** für kleine Header ist unkritisch, für große Werte schädlich | mittel | gelöst durch P0-2; Messung H-S3 zusätzlich mit 4-KiB- und 64-KiB-Werten |
| R-5 | **Blob-GC und `Erase`** komplizieren das Löschprotokoll | hoch | Blob-Zustand in dasselbe Zustandsdiagramm (Beilage B), eigene Absturzmatrix |
| R-6 | **Overlay-Branches halten Versionen:** viele lange Branches blähen den Speicher | mittel | Branch-TTL, `Hold`-Eigentümer (P1-3), Metrik `held_bytes` |
| R-7 | **Header-Format** wird zu früh festgelegt | mittel | Header ist Aufgabe von `cell`; der Kern kennt nur `header_len` und `Weigher`/`Filter`, nicht den Inhalt |
| R-8 | **Index-Konsistenz zwischen Segment und Sicht** (Segment + `history`) | mittel | Modelltest: Suche über Segment+Delta = Suche über Vollscan |
| R-9 | **Embedding-Rückstände** nach Löschung (Nachbarschaft verrät Inhalt) | mittel | Segmente je Löschdomäne, Prüfung im Orakel |

**Offene Entscheidungen (bitte treffen):**

- **O-1** Gewichte: feste Anzahl k = 4 oder eine Nutzlast-Zeichenfolge beliebiger Länge? (Empfehlung: k = 4, u32.)
- **O-2** Löschdomäne: Tenant, Subjektklasse oder Einzelsubjekt? Das bestimmt, wie viele Vektor- und Text-Segmente existieren.
- **O-3** Blob-Dateien als Teil des Kerns (Empfehlung) oder als separates Crate mit Catalog-Erweiterungsschnittstelle?
- **O-4** Zwei Wartungs-Slots (Empfehlung) oder kooperative Preemption?
- **O-5** Stufe B (Commit-DAG) von Anfang an einplanen, oder erst bei konkretem Bedarf an Sync/Audit?
- **O-6** Bedeutung von `routing` (siehe Annahme in Abschnitt 6).

---

## 10. Nächste konkrete Schritte

1. **`SPEC.md` (F0) um die Format-v1-Checkliste erweitern** und mit dem Invariantenkatalog des Bestands zusammenführen. Diese Datei ist der Ort, an dem O-1 bis O-4 entschieden werden.
2. **Workspace-Skelett anlegen:** Root-`Cargo.toml`, `contextra-ports`, `contextra-storage` (leer, Lints), Alt-Crate nach `legacy/contextra-store/` verschieben, `tests/layering.rs`.
3. **`Fs`/`File`/`Env` in `ports` festlegen** (inklusive `read_batch`, `AccessSink`, `Checksum`) und die beiden Referenzimplementierungen `RealFs` und `MemFs/FaultFs` schreiben (F1).
4. **Schlüsselschema und Header** von `contextra-cell` auf Papier festlegen (Abschnitt 7), damit V1 ohne Formatänderung im Kern aufsetzen kann.
5. **Referenzhardware und Benchmark-Szenarien** (A-09) definieren, inklusive eines Agenten-Lastprofils mit 4-KiB- und 64-KiB-Werten, damit H-S3 und H-S7 aussagekräftig werden.

Auf Wunsch setze ich als Nächstes Schritt 1 und 2 im Repo um (Workspace, `ports`, `SPEC.md`-Entwurf). Das ist nur lokal möglich: Ich habe nur Lesezugriff auf `tfufuz1/acta` und kann nichts pushen.

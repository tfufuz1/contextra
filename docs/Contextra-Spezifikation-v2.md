# Contextra – Normative Systemspezifikation, Soll-Zustand v2.0

**Status:** verbindlich für die Neuplanung. Ersetzt die bisherigen Architekturdokumente, soweit sie auf der Löschkette beruhen. Schlüsselwörter MUSS, SOLL, KANN folgen RFC 2119.
**Grundlagen:** Analyse von acht Konzeptentwürfen (Claude, ChatGPT, Gemini, Gemini-2, Perplexity, Deepseek, Grok, Qwen) und statische Sichtung des Repositories (Stand `HEAD`, ohne Build, ohne Ausführung). Die bestehenden Spezifikationsdokumente und Kontextdateien des Repositories wurden bewusst nicht als Vorgabe verwendet.

---

## 0. Entscheidungsrahmen

1. **Löschkette entfällt vollständig.** Es gibt keine Löschnachweise, keine kryptografischen Löschbelege, keine mehrschichtige Tombstone-Kaskade.
2. **Provenienz-Integritätsbeweise bleiben.** Merkle-Inklusionsbeweise, die belegen, dass eine Aussage in einem bestimmten Commit stand, haben mit Löschung nichts zu tun. Sie sind Teil der Verifizierbarkeit des Wissensbestands und bleiben (D-18).
3. **Kein Kompatibilitätszwang.** Bestehender Code darf gelöscht, ersetzt oder neu geschnitten werden. Maßgeblich ist der Soll-Zustand, nicht der Ist-Stand.
4. **Keine Garantie ohne Messung.** Alle Leistungswerte in diesem Dokument sind Zielwerte. Sie werden erst nach Messung auf einem Referenzsystem bindend.

---

## 1. Produkt

### 1.1 Definition
Contextra ist ein transaktionaler Gedächtnis-Kern für KI-Agenten und RAG-Systeme. Er wird eingebettet oder als Dienst betrieben. Er speichert Wissen (Chunks, Entitäten, Kanten, Fakten, Embeddings, Provenienz) in einem gemeinsamen, versionierten Datenbestand. Er liefert hybride Abfragen (Vektor, BM25, Graph) als budgetierte, belegbare Kontextpakete.

### 1.2 Zielgruppe und Betrieb
Mittelstand und Großunternehmen in Deutschland und Europa, keine Behörden. Betrieb on-premises mit lokalem NVMe-Speicher und Datenhoheit beim Kunden.

### 1.3 Kernversprechen (prüfbar)
- **a)** Jede Antwort ist auf einen Commit-Snapshot zurückführbar.
- **b)** Jeder ausgegebene Beleg hat eine prüfbare Herkunft oder wird als unbelegt gekennzeichnet.
- **c)** Agenten schreiben nie direkt in den Hauptstand.
- **d)** Der Kontext überschreitet nie das Token-Budget.
- **e)** Widersprüche werden sichtbar gemacht und nicht still aufgelöst.

### 1.4 Nicht-Ziele (v1)
Verteilter Multi-Writer-Cluster. SQL-Vollständigkeit. OLAP-Scans über Milliarden Zeilen. LLM-Inferenz im Kern (nur über Hooks). Automatisches Sharding. Löschnachweise jeglicher Art.

---

## 2. Leitprinzipien (verbindlich)

- **P1 Wahrheit ≠ Index.** Der Commit-Hash deckt ausschließlich kanonische Daten ab. Indizes sind abgeleitet, rebuildbar und an einen Quell-Root gebunden.
- **P2 Unveränderlichkeit.** Chunks, Packs, Segmente und Commits werden nie in place geändert. Veränderlich sind nur Memtable, WAL-Tail und Ref-Dateien.
- **P3 Ein Zustand.** Jede Query und jeder Commit referenzieren genau einen konsistenten Zustand über alle Datenmodelle.
- **P4 Agenten schreiben nie nach `main`.** Schreibzugriffe erfolgen nur auf Session-Branches. Übernahme nur über Merge-Gates.
- **P5 Tokens statt Zeilen.** Das Ergebnis einer Kontextabfrage ist ein Auswahlproblem unter Budget, kein Top-k-Problem.
- **P6 Zeit ist Datenmodell.** Bitemporale Haltung. Supersession statt Überschreiben.
- **P7 Herkunft ist Pflicht.** Jede abgeleitete Aussage trägt Herkunft. Fehlende Herkunft wird gekennzeichnet, nie stillschweigend erzeugt.
- **P8 Ehrliche Garantien.** Deterministisch verifizierbar ist der Truth-Layer. Recall und Latenz der Indizes sind statistische Eigenschaften und werden gemessen.
- **P9 Ein Durability-Pfad.** Alle Persistenz läuft über ein Modul (WAL, Seal, Ref-Store). Verstreute `fsync`-Aufrufe außerhalb dieses Moduls sind unzulässig.
- **P10 Sicherheit by default.** Safe Rust. `unsafe` nur in benannten Inseln mit dokumentierten Invarianten. Eingaben gelten als untrusted, bis sie validiert sind.

---

## 3. Architektur

### 3.1 Schichten (Zielbild)

| Schicht | Verantwortung |
|---|---|
| L0 Primitive | Dateisystem-Abstraktion (VFS), I/O-Backend, Puffer-Pool, Prüfsummen, AEAD-Verschlüsselung |
| L1 Durabilität | WAL, Group Commit, Seal-Protokoll, Ref-Store mit Reflog, Recovery |
| L2 Truth-Layer | Content-addressed Chunk-Store (Packs), Prolly-Trees je Relation, Commit-Objekte, Manifeste |
| L3 Index-Layer | Vektor-Segmente (Flat, Vamana, Delta), Graph-CSR, Text-Postings, Bitmaps, Statistiken. Alles abgeleitet |
| L4 Versionierung | Commit-DAG, Branches und Sessions, Diff, 3-Wege-Merge, Time-Travel, GC |
| L5 Transaktionen | Snapshot-Manager, OCC, SI und optional SSI, Write-Koordinator |
| L6 Query | Parser/Binder, kostenbasierter Optimizer, Operatoren (VectorScan, Bm25Scan, GraphExpand, Rrf, PackContext), Explain |
| L7 Agenten-Dienste | Sessions, Idempotenz, Savepoints, Provenienz, Citation-Verifier, Merge-Gates, Konfliktmanager, Token-Planner |
| L8 Zugriff | Rust-API (typisiert), MCP-Server, gRPC/Arrow Flight, CLI |

### 3.2 Abhängigkeitsregeln
- Höhere Schichten dürfen tiefere nutzen, nie umgekehrt.
- `contextra-ports` ist die einzige Grenze zwischen L2–L5 und Erweiterungen.
- L0–L4 hängen **nicht** von der Fassade, von Inferenz-Crates oder von Sandbox-Crates ab. Dies wird in CI erzwungen (Abschnitt 14.3).

### 3.3 Write-Path (normativ)
1. Die Transaktion pinnt einen Snapshot: Branch-Head, Manifest, Sequenznummer.
2. Schreiboperationen landen im Transaktionspuffer. Schema- und Pflichtprüfung inklusive Provenienz.
3. Commit-Validierung per OCC auf Schlüsselebene gegen den Branch-Head. Optional SSI.
4. Group Commit: WAL-Records mit LSN und CRC32C, **ein** `fdatasync` pro Gruppe. Erst danach wird bestätigt.
5. Apply in die Branch-Memtable. Die Änderung ist ab diesem Moment für neue Snapshots sichtbar.
6. Seal (im Hintergrund oder explizit per `COMMIT`): Memtable wird zu sortiertem Batch, Prolly-Update, Packs, Index-Deltas, Commit-Objekt, Ref-CAS.
7. **Sofortige Suchbarkeit:** Neue Vektoren sind über den Delta-Puffer ab Bestätigung suchbar. Sie warten nicht auf Kompaktierung.

### 3.4 Read-Path (normativ)
Pin des Snapshots → Planung → alle Operatoren auf demselben Snapshot → Fusion (RRF) → Nachbearbeitung → PackContext → EvidencePack mit Herkunft, Konflikten und Abstention-Signal.

### 3.5 Invarianten (durch Tests erzwungen)
- **I1** Ein Snapshot entspricht genau einem Commit und einer Memtable-Sequenz.
- **I2** Index-Watermark ≤ Snapshot-Watermark, sonst exakter Fallback oder Scan. Ergebnisse tragen die verwendeten Watermarks.
- **I3** Keine sichtbare Teiltransaktion.
- **I4** Nach einer Recovery entspricht jeder Ref entweder dem alten oder dem neuen Zustand.
- **I5** Kein von einem Root erreichbares Objekt wird gelöscht.
- **I6** `tokens_used ≤ budget_tokens`, in allen Fällen.
- **I7** Kein Agenten-Schreibzugriff auf `main`.

---

## 4. Datenmodell

### 4.1 Identitäten
- **EntityId:** 128 Bit, zeitlich sortierbar (48 Bit Millisekunden, 80 Bit Zufall). Stabil über alle Versionen.
- **Versions-ID:** Content-Hash des kanonischen Inhalts. Ein logisches Objekt besitzt viele Versionen.
- **HLC:** 64 Bit (48 Bit physisch, 16 Bit logisch), ergänzt um Knoten-ID als Tie-Breaker.
- Der Zeitanteil der EntityId verrät den Ingest-Zeitpunkt. Mandanten mit strenger Metadaten-Vertraulichkeit erhalten einen rein zufälligen ID-Modus.

### 4.2 Objekte

- **Entity:** `{id, kind, labels, props (typisiert nach Schema), content_ref?, content_hash, valid: Interval<Hlc>, tier, belief: Beta(α,β), prov, level, flags}`. Level: 0 Rohchunk, 1 Abschnitt, 2 Dokument, 3 Thema.
- **Edge:** `{id, src, dst, etype, props, valid, tier, belief, prov}`.
- **Embedding:** Schlüssel `(content_hash, space_id)`. Ein Inhalt wird pro Embedding-Space nur einmal berechnet, auch über Branches und Entitäten hinweg.
- **EmbeddingSpace:** `{id, model_id, revision, dimension, metric, quantization, tokenizer_id}`. Vektoren verschiedener Spaces werden nie verglichen.
- **Claim:** typisierte Aussage `(Subjekt, Prädikat, Objekt, Polarität, valid, prov)`. Prädikate deklarieren Kardinalität (`functional` oder `multi`).
- **ProvRecord:** siehe 10.1.
- **Conflict:** siehe 10.18.
- **Summary:** Entität mit `level > 0` und `derivation_hash`.
- **Betriebsobjekte:** Commit, Manifest, Ref, Session, Savepoint, Quota.

### 4.3 Bitemporalität
Jedes Faktum besitzt `tx_time` (Commit-HLC, unveränderlich) und `valid = [valid_from, valid_to)` (Realwelt). Fakten werden nie überschrieben. Ein neuer Fakt zu `(Subjekt, Prädikat)` bei einem `functional`-Prädikat schließt das Gültigkeitsintervall des alten Fakts und legt eine `SUPERSEDES`-Kante an.

### 4.4 Schema-Katalog
Typen, Kardinalitäten, erlaubte Relationen, DecayProfile, Policies. Der Katalog ist versioniert. Schemaänderungen sind Commits und unterliegen den Merge-Regeln aus 8.4.

### 4.5 Serialisierung
- Interne Records: `rkyv`. Archivierte Daten werden beim Laden validiert, bevor sie vertraut werden.
- Vektorblöcke: alignierte `f16`- oder `f32`-Layouts, Arrow-kompatibel.
- Export, Batch-Schnittstellen und Flight-Streams: Arrow.
- Jeder Record trägt ein Format-Versionsfeld.

---

## 5. Speicherung

### 5.1 Truth-Layer

**Chunk-Store.** Adressierung per BLAKE3 mit Typ-Präfix. Die Adressen sind **mandantenspezifisch keyed** (BLAKE3-keyed mit einem aus dem Mandantenschlüssel abgeleiteten Schlüssel). Dadurch lässt sich aus einem Hash nicht ableiten, ob ein anderer Mandant denselben Inhalt besitzt (D-16). Mandantenübergreifende Deduplizierung ist unzulässig.

**Packs.** Dateiformat `[Header | Chunk* | Footer]`, zugehöriger Index mit Fanout-Tabelle (256 Einträge) und sortierter Hashliste. Kompression zstd mit einem Dictionary je Entitätsart. Blobs größer als 2 KiB liegen out-of-line.

**Prolly-Trees je Relation.** Ein Baum pro Relation:

| Baum | Schlüssel | Wert |
|---|---|---|
| `ent` | EntityId | EntityRecord |
| `edge` | (src, etype, dst, edge_id) | EdgeRecord |
| `emb` | (content_hash, space_id) | VectorRef |
| `prov` | ProvId | ProvRecord |
| `conf` | ConflictId | Conflict |
| `cat` | Name | Schema, Spaces, Decay-Profile, Policies |
| `idem` | Idempotency-Key | Ergebnis-Hash |
| `alog` | (session, seq) | Action-Log-Eintrag |

Eigenschaften: Zielgröße 8 KiB (Minimum 2, Maximum 32), history-independent, Path-Copying bei Änderung, strukturelles Sharing zwischen Branches, Diff in O(d·log N). Bulk-Loads werden bottom-up aufgebaut.

### 5.2 Ingest und WAL
- WAL-Segmente zu 64 MiB, vorallokiert.
- Record-Format: `[len u32 | crc32c u32 | lsn u64 | type u8 | txn_id | payload]`. Der Payload ist logisch und idempotent. Replay steuert über `applied_lsn` im Manifest.
- Ein Torn Write wird durch CRC, Länge und monotone LSN erkannt. Die Recovery stoppt am ersten ungültigen Record (Präfixkonsistenz).

### 5.3 Seal-Protokoll (Reihenfolge verbindlich)
1. Neue Packs schreiben, Footer und Index `fsync`.
2. Index-Segmente und deren Manifeste `fsync`.
3. **Reflog-Record** anhängen (Ref-CAS plus `checkpoint_lsn`) und `fdatasync`. **Dies ist der Commit-Punkt.**
4. Doppelt gepuffertes Superblock `CURRENT` aktualisieren.
5. WAL-Segmente unterhalb `checkpoint_lsn` recyceln.

Ein Seal, der vor Schritt 3 abbricht, hinterlässt nur verwaiste Packs. Diese werden bei der Recovery ignoriert und durch die GC entsorgt.

### 5.4 Recovery
Superblock laden → Reflog-Tail prüfen (CRC) → Refs rekonstruieren → verwaiste Packs ignorieren → WAL ab `checkpoint_lsn` je Branch in Memtables wiederholen → fehlende abgeleitete Indizes im Hintergrund neu bauen. Bis dahin fallen Queries auf exakte Suche zurück. Zielwert: unter 10 s bei einigen hundert MB WAL.

### 5.5 Durability-Klassen
| Klasse | Verhalten | Einsatz |
|---|---|---|
| `Strict` | `fdatasync` pro Commit, Verzeichnis-`fsync` bei Ref-Erzeugung | Compliance- und Auditdaten, Pflicht-Signaturen |
| `Group` (Standard) | Group Commit, ein Sync pro Gruppe, Wartezeit maximal 200 µs oder 1 MiB | `main`, kuratierte Branches |
| `Relaxed` | Ack nach `write`, Sync spätestens nach 100 ms | Session-Branches |
| `Volatile` | Nur RAM, kein WAL | Tests, Wegwerfexperimente |

Die Klasse ist je Branch konfigurierbar. `main` darf nicht `Relaxed` oder `Volatile` sein.

### 5.6 Index-Layer
Alle Indizes sind abgeleitet und liegen in unveränderlichen, hash-adressierten Segmenten. Das Index-Manifest hängt als Seitenobjekt am Commit und geht **nicht** in den Commit-Hash ein. Index-Builds dürfen nicht deterministisch sein. Ein Single-Thread-Modus ist deterministisch und dient den Tests.

### 5.7 Buffer Pool
- Reiner Lese-Cache mit Pin-Zählern. Kein Dirty-Page-Cache, da alles unveränderlich ist.
- Seitenausgerichtet: 4 KiB für Vektorsektoren und Metadaten, 16 KiB für Packs.
- Eviction nach W-TinyLFU oder CLOCK-Pro.
- Getrennte Budgets je Klasse: Truth-Innenknoten, Truth-Blätter, Postings, Graph-Adjazenz, Vektorsektoren, Metadaten. Ein großer Scan darf die heißen Strukturen nicht verdrängen.
- Innenknoten des Truth-Layers werden gepinnt.

### 5.8 I/O
- **Standard:** `pread`/`pwrite` über einen Blocking-Pool bzw. dedizierte Storage-Worker.
- **Optional:** `io_uring`-Backend hinter dem Trait `IoBackend`, nur unter Linux und hinter einem Feature-Flag. Aktivierung nur nach Benchmark-Nachweis (D-OPEN-3).
- **mmap** ist nur Entwicklungs- oder Read-only-Opt-in. Im Produktpfad ist es unzulässig, da unkontrollierte Eviction, TLB-Shootdowns und SIGBUS bei I/O-Fehlern entstehen.
- `O_DIRECT` ist optional und für das `io_uring`-Backend vorgesehen.

### 5.9 Verschlüsselung und Integrität
- AEAD (XChaCha20-Poly1305 oder AES-GCM) auf Pack- und Segmentebene, Schlüssel je Mandant.
- Schlüsselverwaltung außerhalb des Kerns über einen KMS-Hook.
- CRC bei jedem Read. Vollständige BLAKE3-Verifikation als Hintergrund-`scrub`, der den Merkle-DAG durchläuft.
- Schlüssel fließen nie in Hashes ein.

---

## 6. Indizes

### 6.1 Gemeinsamer Docid-Raum
Jedes Segment vergibt dichte lokale `u32`-Docids. Entity-Spalte, Content-Hash, Vektorzeile, Postings und Graph-Knoten teilen dieselbe Docid. Ein Graph-Operator liefert EntityIds als `Roaring64`. Sie werden pro Segment per sortiertem Merge in `Roaring32`-Docids übersetzt. Da EntityIds zeitlich sortiert sind, ist die Übersetzung meist ein Range-Slicing. Sie wird pro Query gecacht. Dieselbe Bitmap filtert anschließend ANN-Suche und BM25.

### 6.2 Vektor
- Normalisierung beim Ingest, sofern die Metrik Cosine ist. Die Metrik ist pro Space fix.
- **Segmenttypen:**
  - **Flat** (unter etwa 50.000 Vektoren): SIMD-Scan (AVX2, AVX-512, NEON). Exakt, ohne Graph.
  - **Vamana/DiskANN** (Standard für große Bestände): R = 64, L_build = 100, α = 1,2. Kompakte Codes (PQ oder RaBitQ) im RAM. Layer-0-Sektoren (4 KiB, Vektorcode und Nachbarlisten co-lokalisiert) auf NVMe. Beam-Search mit Rerank der etwa 2.000 besten Kandidaten aus dem Vollpräzisions-Vektorblock.
  - **Delta** (frische Daten): flacher Puffer im Memtable bis etwa 20.000 Vektoren, danach ein Mini-HNSW im RAM. Versiegelung zu einem Vamana-Segment im Hintergrund.
  - **HNSW** ist optional als RAM-Cache für aktive Namespaces. Es ist jederzeit aus dem Truth-Layer rebuildbar.
- **Kompaktierung:** Größenabgestuft mit Faktor 4 bis 8. Streaming-Merge nach dem FreshDiskANN-Prinzip. Gelöschte Knoten bleiben bis zur Kompaktierung als Routing-Knoten erhalten.
- **Liveness:** Eine Zeile ist sichtbar, wenn `truth[entity].content_hash == row.content_hash`. Das Ergebnis wird je Branch als Roaring-Tombstone-Bitmap gecacht.
- **Mehrere Spaces je Entität** sind zulässig. Spaces werden nie gemischt.
- **Migration:** Dual-Space. Der neue Space wird im Hintergrund befüllt, Abfragen vergleichen beide, der Wechsel ist ein Commit auf `cat`.

### 6.3 Graph
- **Truth:** sortierter Kantenbaum `(src, etype, dst, edge_id)`. Adjazenz ist ein Range-Scan.
- **Index:** CSR je Segment, vorwärts und rückwärts. Offsets Elias-Fano-kodiert, Ziele delta-kodiert und bitgepackt. Hub-Knoten (Grad über 10⁴) erhalten einen Skip-Index. Frische Kanten liegen im sortierten Delta-Overlay.
- **Operatoren:** Frontier als Roaring-Bitmap. BFS mit Richtungswechsel. k-Hop mit Budget. Kantentyp-Masken. Temporal-Pushdown. Approximatives Personalized PageRank per Forward Push (ε ≈ 10⁻⁴).
- **TraversalLimits sind Pflicht.** Standardwerte: Tiefe 3, 100.000 besuchte Knoten, 10.000 Pfade, 200 ms. Bei Überschreitung wird ein Teilergebnis mit `truncated = true` geliefert. Ungebundene Traversals sind unzulässig, da Agenten Abfragen ohne Rücksicht auf Kosten erzeugen.
- **Community-Detection** (Leiden) als Batch-Job. Das Ergebnis speist die Zusammenfassungshierarchie.

### 6.4 Text
- **Analyzer-Pipeline** ist versioniert: Unicode-Normalisierung, Lowercasing, Sprachdetektion, Tokenisierung, Stemming, Stopwords, deutsche Komposita-Zerlegung, N-Gramme für Code und Bezeichner. Analyzer-ID und Parameter gehen in den Segmentschlüssel ein. Eine Änderung erzwingt einen kontrollierten Rebuild.
- **Term-Dictionary** als FST. Postings in Blöcken zu 128 Dokumenten, delta-kodiert und SIMD-bitgepackt. Term-Frequenzen. Block-Max-Scores für Block-Max-WAND. Positionsströme optional für Phrasen.
- **BM25** mit globalen Statistiken (N, df, avgdl), aggregiert aus den Segmentstatistiken im Manifest. Parameter k1 = 1,2 und b = 0,75, je Feld konfigurierbar.
- Filter-Bitmaps überspringen ganze Postings-Blöcke.

### 6.5 Konsistenz der Indizes
Jedes Segment trägt eine Watermark. Abfragen nutzen einen Index nur, wenn seine Watermark ≤ dem Snapshot entspricht. Sonst erfolgt ein exakter Fallback. Jeder Index ist aus dem Truth-Layer rekonstruierbar. Der Test prüft die Gleichheit der Kandidatenmengen nach einem Rebuild.

---

## 7. Hybride Query

### 7.1 Sprache
Eine GQL/Cypher-nahe Syntax mit Erweiterungen `SEARCH` und `PACK CONTEXT`. Zusätzlich ein typisierter Rust-Builder und ein JSON-Plan für MCP. Die finale Syntax wird in M1 festgelegt.

Beispiel (Struktur verbindlich, Schlüsselwörter vorläufig):
```
MATCH (c:Chunk)-[:MENTIONS]->(e:Concept {name: 'Kubernetes'})
WHERE c.tier >= CURATED AND c.valid CONTAINS NOW
SEARCH c USING HYBRID(
    vector(embed($frage), space = 'e5-large', k = 100),
    bm25($frage, k = 100),
    fuse = RRF(k = 60))
EXPAND 1..2 HOPS VIA [:RELATED_TO, :PART_OF] SCORE = PPR
PACK CONTEXT budget = 3500 TOKENS tokenizer = 'cl100k' placement = EDGES_FIRST
ON BRANCH 'agent/planner/s-42' AS OF COMMIT 'a1b2c3'
RETURN EVIDENCE
```

### 7.2 Planner: Strategie für Vektorkandidaten
Eingabe ist die Selektivität σ = |S| / N, wobei S die Kandidatenmenge aus Graph- und Metadatenfiltern ist.

- **A, exakter Pre-Filter:** wenn |S| ≤ τ_A (Startwert 20.000, zu kalibrieren). SIMD-Distanzen über S.
- **B, gefilterte Graph-Traversierung:** Filter als Prädikat im Beam. Bei σ < 10 % zusätzlich Zwei-Hop-Expansion der Nachbarn (ACORN-Prinzip), damit der Graph nicht zerfällt.
- **C, Post-Filter mit adaptivem Over-Fetch:** bei σ > 50 %.
- **Laufzeit-Fallback:** liefert B nach `ef` Pops weniger als k gültige Treffer, wechselt der Operator auf A.

Alle Schwellen werden gemessen, nicht geschätzt.

### 7.3 Ausführung
Alle Operatoren arbeiten auf demselben Snapshot. Die Teilrankings (ANN, BM25, Graph-Hops) werden als Kandidatenlisten geliefert und per RRF fusioniert:

`score(d) = Σ_i w_i / (k + rank_i(d))`, mit k = 60 als Standard.

### 7.4 Nachfusion-Faktoren
Nach der Fusion wirken multiplikative Faktoren, konfigurierbar und kalibriert:
- **Freshness:** `f(d) = max(floor, 2^(−Δt / T½))`, siehe 10.17.
- **Trust:** Faktor aus der unteren Konfidenzgrenze (Abschnitt 10.3).
- **Gültigkeit:** Filter `valid_at`.

Ad-hoc-Gewichte ohne Evaluation sind unzulässig.

### 7.5 Alternative Fusion
Eine NORM-Fusion mit geklemmter Min-Max-Normalisierung ist optional. Standard bleibt RRF, da es skalenfrei ist und keine Score-Kalibrierung erfordert.

### 7.6 Re-Ranking
Ein Cross-Encoder ist als Plugin optional und deadline-gesteuert. Überschreitet er die Frist, wird das Ergebnis ohne Re-Ranking geliefert und mit einem Flag gekennzeichnet.

### 7.7 Explain
`EXPLAIN` liefert den Plan, die Kardinalitäten, die Strategie je Segment, die verwendeten Watermarks und die geschätzte Tokenzahl.

---

## 8. Versionierung

### 8.1 Objektmodell
- **Commit:** `{parents[1 oder 2], manifest_root, author, session?, hlc, message, stats, sig?}`. Der Commit-Hash ist BLAKE3 über die kanonische Kodierung, einschließlich des Provenienz-Roots. Er deckt alle Truth-Bäume ab.
- **Ref:** benannter CAS-Zeiger. Namensräume: `heads/main`, `sessions/<agent>/<id>`, `tags/<name>`, `merge-requests/<n>`.
- **Reflog:** bewahrt die Historie, auch verworfener Sessions. Standard-Retention 30 Tage, konfigurierbar.

### 8.2 Branching
- **Fork** ist ein neuer Ref auf einen bestehenden Commit, O(1), ohne Kopie.
- Session-Branches halten Schreibvorgänge zunächst im Memtable und versiegeln sie zu einem kleinen Segment.
- Die Zahl der Segmente pro Branch-View ist begrenzt (Obergrenze 16 bis 32). Darüber erzwingt der Planer eine Kompaktierung.

### 8.3 Diff
Prolly-Diff je Baum. Gleiche Teilbäume werden übersprungen, Komplexität O(d·log N). Ausgabe als Strom `(key, old, new)` mit den Klassen `Added`, `Removed`, `Modified`. Eine semantische Schicht gruppiert nach Entität und hängt die Provenienz an jede Änderung.

### 8.4 Merge (3-Wege)
1. **Merge-Base** per LCA über Generationsnummern. Bei Criss-Cross-Situationen wird eine rekursiv gemergte virtuelle Basis gebildet.
2. **Doppel-Diff:** `base → ours` und `base → theirs`, je Baum parallel.
3. **Merge-Walk** über die schlüsselsortierte Vereinigung der Diffs, nach dieser normativen Regeltabelle:

| Situation | Regel |
|---|---|
| Nur eine Seite geändert | übernehmen |
| Beide Seiten identisch geändert | übernehmen |
| `ent`: disjunkte Felder | feldweise zusammenführen |
| `ent`: gleiches Feld, verschiedene Werte, oder `content_hash` beidseitig geändert | **Konflikt** |
| `ent.belief` | neu berechnen aus dem deduplizierten Quell-Set (`source_root`), nicht durch Addition der Zähler |
| `ent.tier` | niedrigere Stufe; Hochstufung nur durch Gate G7 |
| `edge` | Mengensemantik. Add/Add mit gleicher Kante ergibt eine Kante. Delete gegen Modify nach Policy je Kantentyp: `AddWins`, `RemoveWins` oder `Manual` |
| `emb`, `prov` | nie Konflikt (content-addressed bzw. append-only, Vereinigung) |
| `conf` | Vereinigung, dedupliziert über das Paar |
| `cat` (Schema) | Kompatibilitätsprüfung, sonst manuell |
| Abgeleitete Summaries (`level > 0`) | nicht mergen, sondern `STALE` markieren und neu erzeugen |

**Vektoren werden nie gemittelt.** Embeddings folgen dem Inhalt. Eine automatische Auswahl per Cosine-Schwelle ist verboten (R-01).

4. **Validierung des Ergebnis-Manifests:** Kardinalitäten, referentielle Integrität, Widerspruchsscan der berührten Subjekte, Provenienz-Vollständigkeit. Verstöße werden als Violations am Merge-Request-Ref festgehalten.
5. **Abschluss:** Merge-Commit mit zwei Eltern auf einem temporären Ref, dann CAS auf das Ziel. Hat sich das Ziel bewegt, wird mit neuer Basis begrenzt wiederholt oder abgelehnt. **Konflikte leben im Merge-Request, nie in `main`.**

### 8.5 Strategien
`fast-forward`, `merge`, `squash`, `rebase`, `cherry-pick`. Cherry-pick übernimmt einen Fakt zusammen mit seinem Provenienz-Abschluss und den zugehörigen Kanten.

### 8.6 Time-Travel (bitemporal)
- `AS OF COMMIT <hash>` greift direkt auf das Manifest zu.
- `AS OF TIMESTAMP t` sucht im Commit-Index per binärer Suche.
- `VALID AT t` filtert auf die Realzeit der Fakten.
- Kombination `AS OF TX t1 VALID AT t2` beantwortet die Frage: Was glaubten wir am Stichtag t1 über den Zustand am Stichtag t2?
- **Reproduzierbarkeit:** `(Commit, Query-Hash, Parameter)` reproduziert jede Agentenantwort exakt.

### 8.7 Rollback
- **Revert** erzeugt einen neuen Commit, der die Änderungen eines früheren Commits zurücknimmt. Standard für geschützte Branches.
- **Reset** setzt einen Ref per CAS auf einen früheren Commit. Für `main` gesperrt, außer für Administratoren.
- **Savepoint** ist das Paar `(Commit, Memtable-Sequenz)`. `ROLLBACK TO` ist ein Ref-CAS. Nicht versiegelte Sequenzbereiche werden für Leser unsichtbar und beim Seal ignoriert.

### 8.8 Garbage Collection
- **Roots:** alle Refs, Reflog-Einträge innerhalb der Retention, Tags, gepinnte Commits, aktive Snapshots (Epochen-Register), aktive Sessions, Backup-Leases.
- **Mark:** Traversal Commit → Manifest → Bäume → Chunks. Ergebnis als Reachability-Bitmap, inkrementell über Generationsnummern.
- **Sweep:** Packs mit weniger als 50 % lebenden Bytes werden umgeschrieben. Vollständig tote Packs werden gelöscht. Abgeleitete Segmente werden freigegeben, sobald kein Manifest sie referenziert.
- **Write-Barrier:** Während des Mark entstandene Chunks gelten als jung und sind ausgenommen.

---

## 9. Nebenläufigkeit

### 9.1 Isolation
- **Snapshot Isolation** ist der Standard.
- **Read Committed** ist nur für administrative Lesepfade zulässig.
- **Serializable (SSI)** ist opt-in mit Read- und Predicate-Sets.
- Write-Write-Konflikte werden per OCC erkannt.

### 9.2 Schreibmodell
Genau ein serialisierender Committer je Branch. Parallel über Branches hinweg. Die Commit-Pipeline besteht aus MPSC-Kanal, Single-Writer je Branch, WAL-Writer mit Ringpuffer und einem Reflog-Thread für die CAS-Updates der Refs.

### 9.3 Lesemodell
- Manifest-Publikation über `ArcSwap<Manifest>` je Branch.
- Leser halten ihren `Arc` für die Dauer der Query.
- Speicherfreigabe über `crossbeam-epoch`.
- Leser blockieren nie Schreiber.

### 9.4 Präzise Garantien
Diese Formulierungen sind verbindlich, Überversprechen sind unzulässig:
- Der Lesepfad kommt ohne globale Locks aus.
- Der Commit-Pfad nutzt kurze kritische Abschnitte und Group Commit.
- Wait-free ist nur für einzeln analysierte Operationen zulässig, etwa den Head-Lookup.
- Memory-Ordering ist für jedes Atomic dokumentiert. Kritische Pfade werden mit `loom` getestet.

### 9.5 Threads und Runtimes
- `tokio` für Netzwerk, Scheduling und Timer.
- Storage-I/O über dedizierte Worker.
- `rayon` für CPU-Arbeit: Distanzen, BM25, Index-Builds, Hashing.
- CPU-intensive Arbeit läuft nie im async-Executor.
- Backpressure über begrenzte Kanäle und Semaphore (I/O-Slots, Speicherbudget).

### 9.6 Typsystem
- Newtypes: `Hash<T>`, `EntityId`, `SegmentId`, `Lsn`, `Seq`.
- Typestates: `Txn<ReadOnly>` und `Txn<ReadWrite>`.
- `Snapshot<'db>` ist lifetime-gebunden. Zero-Copy-Referenzen können den Pin nicht überleben.

### 9.7 Unsafe-Politik
`unsafe` ist nur in `contextra-store` (I/O, optionales mmap für Entwicklung), `contextra-vector` (SIMD) und `contextra-sys` (FFI) zulässig. Jeder Block dokumentiert seine Safety-Invarianten. Alle anderen Crates setzen `#![forbid(unsafe_code)]`.

---

## 10. Agenten- und LLM-Funktionen

### 10.1 Provenienz
```
ProvRecord {
  subject:        SubjectRef,            // EntityId oder EdgeId plus content_hash
  activity:       Ingest | Extract | Infer | Summarize | HumanEdit | Merge,
  tool:           Option<ToolRef>,       // Modell, Version, Parameter-Hash, Prompt-Hash
  sources:        Vec<SourceRef>,        // blob_hash, byte_span, quote_hash
  derived_from:   Vec<SubjectRef>,       // bei Inferenz: genutzte Fakten
  agent, session, tx_time: Hlc,
  extractor_conf: f16,                   // Rohkonfidenz
  source_root:    Hash,                  // Wurzel der Ableitungskette
  taint:          Trusted | Untrusted | Quarantined,
  sig:            Option<Ed25519Signature>,
}
```
Quelldokumente liegen unveränderlich im Blob-Store. Dadurch ist jeder Chunk bis auf Byte-Spannen im Original rückführbar.

### 10.2 Grounding-Stufen
| Stufe | Bedeutung | Sichtbarkeit im Standard-Read |
|---|---|---|
| G0 | Quelle vorhanden, Spanne verifiziert | ja |
| G1 | Quelle vorhanden, Spanne nicht verifiziert | ja, markiert |
| G2 | Agenteninferenz mit Eltern-Fakten | nur mit Konfidenzgrenze |
| G3 | Hypothese ohne Quelle | nur mit `include_hypotheses` |

Der Standard-Read liefert G0 und G1 mit einem Tier ab `AgentInferred`. Hypothesen und Quarantäne-Inhalte sind ausgeschlossen.

### 10.3 Konfidenzmodell
- Jede Entität und jede Kante trägt eine Beta-Verteilung `(α, β)`.
- Priors nach Quellenklasse: kuratiert `Beta(2,1)`, unbekannt `Beta(1,1)`, Agenteninferenz `Beta(1,2)`.
- **Unterstützung:** `α += w·c`, mit w als Quellenzuverlässigkeit und c als kalibrierter Extraktorkonfidenz. **Widerspruch:** `β += w`.
- **Unabhängigkeit:** Eine Bestätigung zählt nur über verschiedene `source_root`s. Ein Agent, der seine eigene frühere Ausgabe zitiert, verstärkt nichts. Das verhindert Echokammern.
- **Ableitung:** `conf(abgeleitet) = min(conf(Eltern)) · c_regel` (noisy-AND). Eine Ableitung ist nie höher als das Minimum ihrer Eltern.
- **Nutzung:** Der Planer rechnet mit der unteren Konfidenzgrenze, dem 10-%-Quantil, nicht mit dem Mittelwert. Wenig Evidenz wird damit automatisch vorsichtiger bewertet.

### 10.4 Session-gebundene Citation-Tokens
Jeder Chunk im Kontextpaket trägt ein kurzes Token, etwa `[[c:7QK4-3F]]`. Es besteht aus einem Handle-Index und einem MAC-Teil:

`MAC = trunc_40(BLAKE3-keyed(session_key, chunk_hash ‖ commit_hash ‖ span))`

Die Abbildung Token → `(chunk_hash, span, commit)` liegt serverseitig je Session. Ein vom Modell erfundenes Token ist mit Wahrscheinlichkeit 1 − 2⁻⁴⁰ nicht auflösbar.

### 10.5 Grounding-Verifier
`VERIFY(claims[])` arbeitet deterministisch. Ein optionaler `GroundingJudge` (NLI) ist nur für Fälle ohne lexikalische Anker zulässig.

| Verdikt | Bedingung |
|---|---|
| `Supported` | Handle gültig. Zitate (normalisiert, Edit-Distanz ≤ ε) sowie Zahlen, Daten und Eigennamen des Claims stehen in der zitierten Spanne. |
| `UnanchoredParaphrase` | Handle gültig, aber keine lexikalischen Anker. Entscheidung durch `GroundingJudge` oder Hinweis. |
| `InvalidHandle` | Token unbekannt oder aus anderer Session. |
| `Stale` | Der zitierte Chunk wurde im aktuellen Head ersetzt. |
| `Contradicted` | Zu den Quellen existiert ein offener Konflikt. |

### 10.6 Claim-Extraktion und Citation Binding
LLM-Ausgaben werden vor dem Commit in atomare, typisierte Claims zerlegt. Jede Assertion braucht mindestens einen Chunk-Bezug. Ohne Bezug wird sie als G3 markiert.

### 10.7 Abstention-Signal
Jedes Ergebnis trägt `evidence_sufficiency ∈ {Sufficient, Thin, Conflicting, None}`. Die Berechnung nutzt die Anzahl unabhängiger `source_root`s, die kalibrierte Top-Relevanz und den Konfliktstatus. Bei `None` rendert der Planer einen Block `NO_EVIDENCE_FOUND` mit Angabe des Suchbereichs. Das Modell soll dann abstinent antworten, statt zu raten.

### 10.8 Agenten-Sessions
Lebenszyklus: `Active → Validating → Merged | Aborted | Expired`.

```
Session {
  id, agent, base: Hash,           // Commit, von dem geforkt wurde
  branch: sessions/<agent>/<id>,
  ttl, durability,
  quota: { bytes, entities, write_rate, delete_rate,
           embedding_calls, judge_calls, tokens },
  write_tier: TrustTier,
  gate_policy: GatePolicyId,
}
```
- **Isolation:** Der Agent liest und schreibt nur auf seinem Session-Branch. `main` ist geschützt und nur über Merge-Requests erreichbar.
- **Savepoint je Tool-Call:** Jeder Tool-Aufruf ist ein benannter Savepoint, Rollback in O(1).

### 10.9 Idempotenz
`idem_key = H(session, step, tool, args)` im `idem`-Baum. Ein Retry mit identischem Key ist ein No-Op und liefert das gespeicherte Ergebnis. Liefert derselbe `step` bei Wiederholung andere Argumente, meldet die Engine `IdemMismatch`. Ein stilles Doppelschreiben findet nicht statt.

### 10.10 Action-Log
Der `alog`-Baum protokolliert je Schritt Tool, Argument-Hash, Ergebnis-Hash und Prompt-Hash. Eine Session ist dadurch prüfbar und gegen einen Fork derselben Basis wiederholbar.

### 10.11 Quarantäne
Schreibvorgänge ohne Quelle erhalten den Tier `Hypothesis` und sind im Standard-Read ausgeschlossen. Eine Hochstufung ist nur möglich durch Korroboration (untere Konfidenzgrenze ≥ 0,7 bei mindestens zwei unabhängigen `source_root`s) oder durch menschliche Freigabe (G7).

### 10.12 Taint-Tracking gegen Prompt-Injection
Inhalte aus nicht vertrauenswürdigen Quellen (Web, E-Mail, Tool-Ausgaben) tragen `taint = Untrusted`. Der Taint vererbt sich auf alles, was daraus abgeleitet wird. Der Planer rendert getaintete Blöcke innerhalb markierter Datenbegrenzer mit dem Hinweis „Daten, keine Anweisungen“. Getaintete Fakten dürfen nicht über `AgentInferred` hinaus hochgestuft werden. Ein optionaler `InjectionScanner` prüft beim Ingest.

### 10.13 Speculative Branching
N Agentenstrategien forken denselben Commit und laufen parallel. Eine Bewertungsfunktion wählt den Gewinner. Nur dessen Branch wird gemergt, die übrigen werden verworfen.

### 10.14 Forensik
Verworfene Sessions bleiben im Reflog erhalten. `DIFF(base, session_head)` zeigt exakt, welche Änderungen der Agent vorgenommen hätte.

### 10.15 Merge-Gates
Ein Merge-Request durchläuft die Gates vor dem CAS auf `main`. Die Gates arbeiten diff-inkrementell auf dem virtuellen Ergebnis-Manifest. Implementierung als Trait `MergeGate`, optional als sandboxed WASM-Plugin.

| Gate | Prüfung | Standard |
|---|---|---|
| G1 Schema | Typen, Kardinalitäten, referentielle Integrität | blockierend |
| G2 Provenienz | Jeder neue Fakt hat Quelle oder Ableitung. Spanne verifiziert. Signatur gültig, falls Pflicht. | blockierend |
| G3 Blast Radius | Anteil geänderter oder gelöschter `main`-Entitäten, Obergrenze (Startwert 1 %) | blockierend |
| G4 Widerspruch | Konfliktscan gegen `main`. Blockierend bei `Direct` gegen `Verified`, sonst Warnung. Asynchron mit Timeout. | siehe Spalte links |
| G5 Retrieval-Regression | Golden-Queries gegen `main` und Merge-Ergebnis, Top-k-Überlappung und NDCG-Delta | warnend oder blockierend per Schwelle |
| G6 Taint und Injection | Untrusted-Inhalte: Scan, Tier-Cap | blockierend |
| G7 Freigabe | Menschliche Bestätigung für Namespaces oder Tier `Verified` | konfigurierbar |

Ein nicht bestandenes Gate hält den Merge-Request offen. Der Agent erhält eine strukturierte Fehlerbeschreibung mit betroffenen Fakten und Regel und kann auf seinem Branch korrigieren.

### 10.16 Bitemporale Haltung und Veraltung
**DecayProfile** je Prädikat oder Entitätsart:
- `class`: `news` (14 Tage), `policy` (365 Tage), `api_docs` (180 Tage plus versionsbasierte Supersession), `constant` (kein Decay).
- `kind`: `Exponential{half_life}`, `Weibull{k, λ}`, `Step{valid_until}` oder `None`.
- `floor`: Mindestgewicht, damit alte Belege nie vollständig verschwinden.
- `refresh_on_reaffirm`.

`fresh(d) = max(floor, 2^(−Δt / T½))`, mit `Δt = as_of − max(valid_from, last_confirmed)`.

Regeln:
- Decay wirkt **nur auf das Ranking**. Er löscht oder beendet nie einen Fakt.
- `IGNORE DECAY` und `AS OF` sind jederzeit möglich.
- Adaptive Volatilitätsschätzung `λ̂ = Supersessions / Beobachtungszeit` wird als Vorschlag ausgegeben, nicht automatisch aktiviert.
- **Reaffirmation:** Derselbe Fakt aus einer unabhängigen Quelle erzeugt kein Duplikat. `last_confirmed` wird aktualisiert, `α` steigt, ein weiterer ProvRecord mit `ev_seq + 1` wird angehängt.

### 10.17 Konflikte
Ein Fakt wird bei Bedarf über den Hook `Extractor` als Claim normalisiert. Danach läuft eine gestufte Prüfung:

| Stufe | Prüfung | Ausführung |
|---|---|---|
| 1 Deterministisch | `functional`-Prädikat, überlappende Gültigkeit, abweichendes Objekt → `Direct`. Numerische Abweichung über Toleranz → `Numeric`. Neuere Gültigkeit mit Policy `SupersedeOnNewer` → automatische Supersession statt Konflikt. | synchron im Commit |
| 2 Kandidatensuche | ANN-Nachbarn des Claim-Embeddings innerhalb der 1–2-Hop-Nachbarschaft des Subjekts. BM25 mit Gegenbegriffen und Negationen. | asynchron |
| 3 Semantisch | `ContradictionJudge` (NLI oder LLM): `Entails`, `Contradicts`, `Neutral` mit Score | asynchron über die Jobtabelle |
| 4 Objekt | `Conflict` als Entität erster Klasse im `conf`-Baum | versioniert, mergebar |

```
Conflict {
  id, a: SubjectRef, b: SubjectRef,
  kind: Direct | Numeric | Temporal | Semantic,
  score, judge: Option<ToolRef>,
  status: Open | Resolved{by} | AcceptedBoth,
  policy: NewerWins | TrustWeighted | Corroboration | Contextual | Human,
}
```

**Regeln:**
- Auf `main` wird **nie automatisch gelöscht**. Beide Seiten bleiben erhalten, der Konflikt ist markiert.
- Der Read liefert Treffer mit offenem Konflikt gekennzeichnet. Der Planer legt beide Seiten mit Provenienz und Konfidenz in einen `CONFLICT`-Block, setzt `sufficiency = Conflicting` und senkt die Relevanz betroffener Items. Er wählt nicht still eine Seite.
- Ein LLM darf Konflikte vorschlagen oder begründen. Es darf einen Konflikt nicht allein auf Grundlage seiner eigenen Argumentation auflösen.
- Gate G4 wartet auf die asynchronen Stufen mit Timeout und degradiert bei Überschreitung zu einer Warnung.

---

## 11. Token-aware Planner (`PACK CONTEXT`)

### 11.1 Anfrage
```
ContextRequest {
  budget_tokens: u32,            // harte Obergrenze inklusive Rendering-Overhead
  tokenizer: TokenizerId,
  placement: EdgesFirst | Chronological | ByDocument | Sandwich | RankOrder,
  zoom: Raw | Summary | Adaptive,
  min_tier, min_grounding,
  dedup: DedupPolicy,
  include_conflicts: bool,
  explain: bool,
}
```

### 11.2 Pipeline
1. **Kandidaten:** 200 bis 500 Treffer aus der Hybridsuche, ergänzt um Graph-Expansion (PPR).
2. **Renditionen:** Jeder Kandidat hat mehrere Darstellungen: `raw`, `trimmed` (satzweise gekürzt), `summary` (Elternebene), `triples` (kompakte Faktennotation). Die Kosten `c(r)` sind exakte Tokenzählungen für den angeforderten Tokenizer. Sie werden im Index-Layer gecacht (`tok_count[tokenizer_id]`), damit der Truth-Layer deterministisch bleibt.
3. **Nutzen:** `u(d) = rel(d)^a · trust_lcb(d)^b · fresh(d)^c · quality(d)`.
   - `rel` ist kalibriert, isotonische Regression auf Feedback. Ohne Feedback der normalisierte RRF-Rang.
   - `quality` ist die Informationsdichte, berechnet beim Ingest: Summe der IDF-Gewichte distinkter Terme und Entitäten pro Token, abzüglich des Boilerplate-Anteils. Als billiger Proxy dient die zstd-Kompressionsrate.
4. **Selektion:** Maximiere die Facility-Location-Abdeckung `F(S) = Σ_j u(j) · max_{d∈S} sim(j, d)` unter der Nebenbedingung `Σ c ≤ B`.
   - Verfahren: lazy Greedy nach `ΔF / c`, verglichen mit dem besten Einzelelement. Die Garantie beträgt ½(1 − 1/e) ≈ 0,32, in der Praxis liegt das Ergebnis nahe am Optimum.
   - Pro Dokument wird genau eine Rendition gewählt (Partitionsbedingung).
   - Redundanz wird doppelt bekämpft: harte Near-Duplicate-Erkennung beim Ingest (SimHash oder MinHash, Schwelle ≥ 0,9 ergibt einen kanonischen Chunk plus Alias-Liste) und weiche Diversität über den Abdeckungsterm.
5. **Hierarchischer Zoom:** Zusammenfassungen sind abgeleitete Entitäten (`level` 1 Abschnitt, 2 Dokument, 3 Thema) mit `SUMMARIZES`-Kanten auf ihre Kinder. Ihr `derivation_hash = H(Kind-Hashes, Summarizer-ID, Prompt-Hash)`. Ändert sich ein Kind, stimmt der Hash nicht mehr. Die Zusammenfassung wird `STALE` markiert, ihre Neuerzeugung eingeplant. Bis dahin nutzt der Planer sie nur mit Warnung oder fällt auf die Kinder zurück. Im Modus `Adaptive` ersetzt der Planner mehrere Kinder durch die Elternzusammenfassung, wenn deren Nutzen pro Token höher ist. Jeder verdichtete Block erhält einen **Expand-Handle** für progressives Nachladen.
6. **Platzierung:** Die Policy ist konfigurierbar (`EdgesFirst`, `Chronological`, `ByDocument`, `Sandwich`, `RankOrder`). Die Wirkung von `EdgesFirst` gegen den Lost-in-the-Middle-Effekt ist modell- und aufgabenabhängig. Sie ist deshalb eine Hypothese. **Default ist `RankOrder` mit Gruppierung nach Quelle.** `EdgesFirst` wird Default erst, wenn ein A/B-Test im Rahmen von G5 einen messbaren Gewinn zeigt (D-OPEN-2).
7. **Rendering:** stabile Block-IDs, Citation-Tokens, Inhaltsverzeichnis mit Angabe enthaltener und ausgelassener Blöcke samt Expand-Handles, Konfliktblöcke, Staleness-Hinweise, Block `NO_EVIDENCE_FOUND` bei `None`.
8. **Budgetgarantie:** exakte Zählung mit 2 bis 3 % Sicherheitsmarge. Bei Überlauf wird der Block mit dem geringsten Grenznutzen entfernt. Ein Block wird nie mitten im Satz gekürzt.
9. **Explain:** `PlanTrace` mit gewählten und verworfenen Blöcken und deren Grenznutzen.

### 11.3 Verfügbares Budget
`Verfügbar = Kontextlimit − System-Prompt − User-Prompt − Tool-Schemas − Gesprächshistorie − reservierte Ausgabe − Sicherheitsmarge`

Der Tokenizer ist modellspezifisch. Wortzählungen sind unzulässig.

### 11.4 Budgetaufteilung (optional)
Beispiel: 70 % für Primärbelege, 20 % für Gegenbelege, 10 % für Rohquellen-Pointer. Die Aufteilung ist konfigurierbar.

---

## 12. Sicherheit, Mandanten und Datenschutz

### 12.1 Mandantentrennung
Jeder Schlüssel enthält den Mandanten bzw. Namespace. Lesezugriff über Mandantengrenzen erfordert eine explizite Policy. Deduplizierung findet nur innerhalb eines Mandanten statt (D-16).

### 12.2 Verschlüsselung und Hashes
AEAD auf Pack- und Segmentebene. Hashes werden über den Klartext gebildet, im mandantenspezifischen keyed-Modus (siehe 5.1). Schlüssel fließen nie in Hashes ein.

### 12.3 Entfernung personenbezogener Daten
Die Löschkette und Löschnachweise entfallen. Technisch bleibt eine einfache Operation nötig:

- `purge(entity | document)` läuft **synchron und atomar als Commit**. Der neue Zustand enthält die Daten nicht mehr: Records, Kanten, Embeddings und Provenienzreferenzen werden entfernt.
- Die physische Entfernung aus Packs und Indizes erfolgt bei Seal oder Kompaktierung. Bis dahin werden Index-Einträge über den Liveness-Check ausgeblendet.
- Für verschlüsselte Mandanten ist zusätzlich die Vernichtung des Mandantenschlüssels möglich.
- Es werden keine Nachweise über die Entfernung ausgestellt.
- **Historische Commits enthalten die Daten weiterhin**, solange sie durch die Retention gehalten werden. Diese Einschränkung wird in der Produktdokumentation ausgewiesen. Eine Redaktion der Historie (neuer Hash, dokumentierter Ausnahmeprozess) ist eine Admin-Operation.
- Die rechtliche Bewertung dieser Umsetzung ist offen (D-OPEN-1).

### 12.4 Taint und Injection
Siehe 10.12.

### 12.5 Egress-Schutz
`contextra-privacy` prüft alles, was den Kern verlässt: LLM-Aufrufe, Exporte, externe Judges. Die Prüfung umfasst DLP-Regeln und Exfiltrationsschutz.

### 12.6 Audit und Signaturen
`alog`, Reflog und Commits bilden die Auditspur. Commits werden mit Ed25519 signiert. Die Signatur ist im Modus `Strict` Pflicht. Signaturschlüssel sind je Agent und Dienst getrennt.

### 12.7 Quoten
Schreibrate, Tokenverbrauch, Embedding- und Judge-Aufrufe sind pro Session begrenzt. Quoten brechen Endlosschleifen fehlerhafter Agenten ab.

### 12.8 Authentisierung
Token-basiert auf API-Ebene mit Rollen (Leser, Agent, Kurator, Administrator). Details liegen außerhalb des Kernprofils.

---

## 13. Schnittstellen

### 13.1 Rust-API
Typisierte Transaktionen mit expliziten Grenzen:
```rust
let mut tx = db.begin_write(Branch::session("agent/planner/s-42"), Isolation::Snapshot)?;
let entity = tx.create_entity(props, prov)?;
let chunk  = tx.create_chunk(text, prov)?;
tx.attach_embedding(chunk.id(), space, vector)?;
tx.create_edge(entity.id(), chunk.id(), Relation::References, prov)?;
tx.commit("Fakten aus Dokument X")?;   // ein gemeinsamer Commit, atomar
```

### 13.2 MCP-Werkzeuge (verbindlich)

| Werkzeug | Zweck |
|---|---|
| `memory.search` | Hybridsuche mit `PACK CONTEXT`. Liefert `EvidencePack`. |
| `memory.expand` | Expand-Handle auflösen, also Zusammenfassung zu Rohdaten oder Nachbarschaft |
| `memory.write` | Fakten, Chunks, Kanten mit Pflichtprovenienz in den Session-Branch schreiben |
| `memory.savepoint`, `memory.rollback` | Savepoint setzen und zurückkehren |
| `memory.fork`, `memory.diff` | Branch erzeugen, Diff zwischen Commits oder Branches |
| `memory.merge_request` | Merge-Request mit Gate-Lauf erzeugen, Status und Verstöße abfragen |
| `memory.verify` | Claims gegen Citation-Tokens prüfen |
| `memory.history` | Verlauf einer Entität, bitemporal (`AS OF TX`, `VALID AT`) |
| `memory.conflicts` | Offene Konflikte listen und auflösen |
| `memory.purge` | Entfernung personenbezogener Daten. Nur Administratorrolle. |

### 13.3 gRPC und Arrow Flight
Batch-Ingest und gestreamte Ergebnisse als RecordBatches.

### 13.4 CLI (Administration)
`scrub`, `rebuild-index`, `gc`, `verify-chain`, `explain`.

### 13.5 Versionierung
API folgt semantischer Versionierung. Schema- und Format-Versionen stehen in jedem Record.

---

## 14. Zielarchitektur des Repositories

### 14.1 Ist-Befund (statisch, HEAD, ohne Build)

- **35 Crates** unter `crates/`, dazu `benchmarks/contextra-bench`, `xtask` und `xtask-heavy`.
- **Löschkette im Code:** `contextra-crypto/src/deletion_proof.rs` (1934 Zeilen), `contextra-graph/src/cascade.rs` (1059 Zeilen), `contextra-types/src/tombstone.rs` (131 Zeilen), `contextra-mcp/src/proof_key.rs` (166 Zeilen). Dazu Löschpfade in `contextra-store` (`kv/delete_mode.rs`, `compaction/retention.rs`) und in `contextra-engine` (`transaction/compensating_actions.rs`).
- **Schichtverletzungen:**
  - `contextra-store`, `contextra-vector` und `contextra-simd` hängen vom als deprecated markierten Facade-Crate `contextra-core` ab.
  - `contextra-engine` (Ring 3) hängt von `contextra-infer-candle` (Ring 2) und `contextra-sandbox` ab.
  - `contextra-engine` enthält `kv_cache_integration`. LLM-KV-Caching gehört nicht in den Gedächtniskern.
- **Doppelte Oberflächen:** Fassade `contextra` und `contextra-db` (Re-Exports) bilden zwei öffentliche Einstiege. `contextra-core` ist deprecated.
- **Experimentelle Module im Kernpfad** ohne dokumentiertes Produktziel: `apprh`, `tl_hfd`, `percolation`, `hyperedge`, Resonanz-Fusion, DIBUD. Sie haben keinen Platz im v1-Kern, sofern sie nicht per Benchmark begründet werden.
- **Durability verteilt:** Rund 175 Trefferstellen für `fsync`/`fdatasync`-Muster in mehreren Crates. Das verletzt P9.
- **Metadaten uneinheitlich:** Ring-Zuordnung in `package.metadata.contextra` ist nicht vollständig. Beschreibungen fehlen bei `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-mcp` und `contextra-router`.

### 14.2 Zielbild der Crates

**Kern (v1):**

| Crate | Zielrolle | Maßnahme |
|---|---|---|
| `contextra-types` | Typen, IDs, HLC, Budgets, Fehler | ausdünnen, Tombstone-Typen entfernen |
| `contextra-ports` | Traits als Grenze | behalten |
| `contextra-simd` | Distanzkerne | Abhängigkeit von `contextra-core` entfernen |
| `contextra-sys` | unsafe-Inseln, FFI | behalten |
| `contextra-durable-fs` | VFS, Fault-Injection, Journal | behalten, wird zum einzigen Durability-Pfad mit WAL |
| `contextra-wal` | WAL, Group Commit, Recovery | **neu**, aus `contextra-store/wal` herauslösen |
| `contextra-store` | Truth-Layer: Packs, Prolly-Trees, Memtable, Seal | umbauen, LSM-Compaction-Zoo reduzieren |
| `contextra-vcs` | Commit-DAG, Refs, Reflog, Merge, Diff, GC | **neu**, Konsolidierung aus `mvcc`, `checkpoint` und Transaktionslogik der Engine |
| `contextra-mvcc` | Snapshots, OCC, SSI | behalten |
| `contextra-vector` | Flat, Vamana, Delta, HNSW-Cache | umbauen, `contextra-core`-Abhängigkeit entfernen |
| `contextra-graph` | CSR, PPR, Traversal-Limits | behalten, `cascade.rs` entfernen, Experimente auslagern |
| `contextra-text` | Analyzer, FST, Postings, BM25/WAND | behalten |
| `contextra-rank` | RRF, Normierung, Kalibrierung | behalten, DIBUD und Resonanz nur nach Benchmark |
| `contextra-query` | Parser, Planner, Operatoren, Explain | **neu**, aus `contextra-engine` herauslösen |
| `contextra-context` | Token-Planner `PACK CONTEXT` | **neu**, aus `contextra-router` und `contextra-engine` |
| `contextra-agent` | Sessions, Idempotenz, Gates, Provenienz, Verifier, Konflikte | konsolidieren |
| `contextra-crypto` | AEAD, keyed Hashing, Signaturen | `deletion_proof.rs` entfernen |
| `contextra-privacy` | DLP und Egress | behalten |
| `contextra-ingest` | Extraktion und Chunking | behalten |
| `contextra-checkpoint` | Backup und Snapshot-Export | anpassen an `contextra-vcs` |
| `contextra-cognition` | Konsolidierung, Summaries | optional (Feature) |
| `contextra-mcp` | MCP-Server | behalten, `proof_key.rs` entfernen |
| `contextra` | **einzige** öffentliche Fassade | behalten, Feature-Flags |
| `contextra-testkit` | Deterministische Simulation, FaultVfs, ManualClock | erweitern |

**Optional (Feature oder eigene Edition):**

| Crate | Begründung |
|---|---|
| `contextra-infer-candle`, `-ollama`, `-onnx` | Embedder und Judges als Adapter über Ports. Nicht im Kern. |
| `contextra-sandbox` | WASM-Gates als Plugin |
| `contextra-py` | Python-Bindings, nicht im Kern |
| `contextra-adapt` | Bandits und PID-Regler. Erst nach Messbedarf. |
| `contextra-license` | Lizenz-Gate. Gehört in die Enterprise-Edition, nicht in den Engine-Kern. |
| `contextra-audit-export`, `contextra-avv-generator` | Compliance-Werkzeuge, außerhalb des Kerns |
| `contextra-wire` | FlatBuffers-IPC. Ersetzt durch Arrow Flight. Neubewertung in M5. |

**Entfernen:**

| Element | Begründung |
|---|---|
| `contextra-core` | deprecated Facade, verursacht Schichtverletzungen |
| `deletion_proof`, `cascade`, `tombstone`-Kette, `proof_key` | Löschkette, ausgeschlossen |
| `contextra-kvcache` | LLM-KV-Cache ist ein eigenes Produkt und nicht Teil des Gedächtniskerns |
| `apprh`, `tl_hfd`, `percolation`, `hyperedge` | keine Produktanforderung, keine Benchmark-Begründung |

### 14.3 Erzwungene Abhängigkeitsregeln (CI)
- Kern-Crates (`types`, `ports`, `simd`, `sys`, `durable-fs`, `wal`, `store`, `vcs`, `mvcc`, `vector`, `graph`, `text`, `rank`, `query`, `context`, `agent`) dürfen nicht von `infer-*`, `sandbox`, `py`, `license` oder `contextra-core` abhängen.
- `contextra-store` darf nicht von `contextra-core` abhängen.
- Die Fassade `contextra` ist die einzige Crate, die alles zusammensetzt.
- Keine Zyklen. Prüfung über `cargo metadata` in CI.
- Ring-Metadaten sind für jedes Crate Pflicht.

---

## 15. Qualitätssicherung und Abnahme

### 15.1 Korrektheit
- **Deterministische Simulation** mit Crash-Injection an jedem Schritt des Seal-Protokolls und jedem WAL-Write (Failpoints).
- **Invarianten I1 bis I7** als Property-Tests.
- **Prolly-Trees:** Property-Tests für History-Independence und Diff/Merge-Rundlauf.
- **loom** für Memtable, Buffer Pool und Commit-Pipeline.
- **Miri** für alle unsafe-Module.
- **cargo-fuzz** für Parser, WAL-Decoder, Pack-Decoder und rkyv-Validierung.

### 15.2 Suchqualität
- Recall@10 ≥ 0,95 bei definierter Latenz (ANN-Benchmark-Stil).
- BM25 und Hybridqualität auf BEIR-artigen Korpora, deutsch und englisch.
- Multi-Hop-Retrieval auf eigenen Agenten-Workloads.
- Graph-Operatoren gegen LDBC-SNB-artige Workloads.

### 15.3 Agenten-Szenarien
Fehlerinjektion mit erwarteten Gate-Ergebnissen als Abnahmetests: doppelte Tool-Calls, halluzinierte Handles, widersprüchliche Fakten, Injection-Inhalte.

### 15.4 Performance-Gates in CI
`criterion` oder `divan`. Regressionsgrenzen für Commit-Latenz p99, Seal-Durchsatz, p99-Hybridlatenz und Recovery-Zeit.

### 15.5 Zielwerte (Hypothesen, erst nach Messung bindend)
- Größe: 10⁵ bis 10⁸ Chunks je Knoten, 384 bis 3072 Dimensionen, bis 10⁹ Kanten.
- Hybrid-Query, k = 20: p99 unter 30 ms bei 10⁷ Chunks warm, unter 80 ms bei kaltem NVMe.
- Ingest: mindestens 50.000 Entitäten pro Sekunde je Branch, ohne Embedding-Berechnung.
- Commit: p99 unter 5 ms (Group Commit, NVMe).
- Branch-Fork: unter 100 µs. Diff und Merge: O(Δ · log N).
- Recovery: unter 10 s bei einigen hundert MB WAL.

### 15.6 Qualitätsmetriken für Agenten
Anteil grounded Antworten, Konfliktrate, Abstention-Rate, Budgetauslastung, Redundanzrate, Anteil veralteter Summaries. Regression wird über Golden-Queries überwacht.

---

## 16. Meilensteine und Abnahmekriterien

**M0 Fundament (Truth und Durability).** WAL, Memtable, Packs, Prolly-Trees, Commit-Grundform, Refs und Reflog, Seal-Protokoll, Recovery, Crash-Simulation.
- Abnahme: Kein bestätigter Commit geht verloren. Keine teilsichtbaren Transaktionen. Nach Crash ist jeder Ref alt oder neu. Die Simulation läuft fehlerfrei über 10⁵ Läufe (Ziel).

**M1 Suche.** FTS-Segmente, Flat- und Vamana-Segmente, Delta-Puffer, CSR, RRF, einfache Hybrid-Query, Explain.
- Abnahme: Index-Rebuild aus dem Truth-Layer liefert gleiche Kandidatenmengen. Watermark-Fallback ist getestet. Recall-Ziel auf Referenzkorpus erreicht.

**M2 Versionierung vollständig.** 3-Wege-Merge nach Regeltabelle, bitemporales Time-Travel, Diff, Revert und Reset, GC mit Roots.
- Abnahme: Zero-Copy-Branch ist gemessen. Konfliktfreie Merges sind korrekt. GC löscht nie erreichbare Objekte (Property-Test).

**M3 Agenten-Kern.** Sessions, Savepoints, Idempotenz, Action-Log, ProvRecord, Citation-Tokens, Verifier, Gates G1 bis G3, Taint-Grundform.
- Abnahme: Kein Schreibvorgang auf `main` ohne Gate. Erfundene Tokens sind nicht auflösbar. Ein Replay reproduziert die Session.

**M4 Planner und Zeit.** `PACK CONTEXT` mit Budgetgarantie, Abstraktionshierarchie, Decay, Konflikte (Stufen 1 bis 4), Gates G4 bis G6, MCP-Server.
- Abnahme: `tokens_used ≤ budget` in 100 % der Tests. Konflikte werden nie still überschrieben. Decay löscht nie Fakten.

**M5 Härtung.** AEAD, mandantenspezifisches CAS-Hashing, `purge`, `io_uring`-Backend nur bei Benchmark-Nachweis, Monitoring, Dokumentation, Beta.
- Abnahme: Mandantentrennungstests bestanden. Scrub läuft sauber. Lasttests innerhalb der Zielwerte.

**M6 Roadmap v2 (nicht v1).** Raft-Replikation über Reflog und WAL, Object-Storage-Tier für kalte Packs, Read-Replicas für Time-Travel.

---

## 17. Entscheidungsprotokoll

### 17.1 Übernommene Entscheidungen

| ID | Entscheidung | Herkunft / Begründung |
|---|---|---|
| D-01 | Ein Commit- und Snapshot-Modell für alle Datenmodelle | Konsens aller Entwürfe, ChatGPT P2 |
| D-02 | Truth-Layer und Index-Layer getrennt. Hash deckt nur Truth ab | Claude P1 und 5.2 |
| D-03 | Prolly-Trees für den Truth-Layer (Diff, Merge, Struktur-Sharing). WAL und Memtable für Ingest | Claude. Reiner LSM wäre für Diff und Merge teurer. |
| D-04 | Unveränderliche Index-Segmente mit Delta-Overlay | ChatGPT, Claude, Deepseek |
| D-05 | Gestufte ANN: Flat, Vamana, Delta/HNSW | Claude. Gemini-Schwelle gegen Claude-Schwelle geprüft, Schwelle wird gemessen. |
| D-06 | Filterstrategien nach Selektivität mit Laufzeit-Fallback | Claude 2.9, Gemini-2, ChatGPT 2.8 |
| D-07 | Docid-Raum je Segment | Claude 2.7 |
| D-08 | RRF als Standardfusion, k = 60 | alle Entwürfe |
| D-09 | Pflicht-TraversalLimits | ChatGPT 2.6. Wichtig für Agenten. |
| D-10 | Provenienz mit `source_root`-Unabhängigkeit, Beta mit unterem Quantil, noisy-AND bei Ableitung | Claude 4.1. Ersetzt Gemini-Produktformel (R-03). |
| D-11 | Session-gebundene MAC-Citation-Tokens, Verifier, Abstention | Claude 4.1 |
| D-12 | Token-Planner als Auswahl unter Budget (Facility Location) mit Renditionen | Claude 4.2. Gemini-MMR und U-Form als Optionen übernommen. |
| D-13 | Lost-in-the-Middle-Platzierung nur als evaluierte Option | ChatGPT 5.2 (Korrektur an Gemini, Perplexity, Grok) |
| D-14 | Konflikte als Objekte, nie automatisch löschen auf `main` | Claude 4.6, ChatGPT 5.4 |
| D-15 | Bitemporalität mit Supersession | alle Entwürfe |
| D-16 | Mandantenspezifisches keyed CAS-Hashing, keine mandantenübergreifende Deduplizierung | ChatGPT 7.5. Korrektur an Claude 5.5 (Klartext-Hash über Mandanten hinweg). |
| D-17 | Keine Löschkette, keine Löschnachweise | Auftrag |
| D-18 | Merkle-Inklusionsbeweise für Provenienz bleiben | Claude 4.1 (PROVE). Auslegung: Beweise zur Verifizierbarkeit, nicht zur Löschung. |
| D-19 | Seal-Protokoll mit Reflog-Record als Commit-Punkt | Claude 3.9 |
| D-20 | Durability-Klassen Strict, Group, Relaxed, Volatile | Claude 3.3 |
| D-21 | I/O: portabler Standard, `io_uring` hinter Trait, mmap nur Entwicklung | ChatGPT 4.5, Claude 2.2. Korrektur an Deepseek und Gemini (io_uring zentral). |
| D-22 | Ein Single-Writer je Branch, OCC, SI als Standard | Claude 3.8, ChatGPT 3.5 |
| D-23 | ArcSwap für Manifest, crossbeam-epoch für Reclamation | alle Entwürfe |
| D-24 | Merge-Regeltabelle, keine Vektormittelung, keine Cosine-Auto-Auswahl | Claude 3.5, ChatGPT 3.6. Ersetzt Gemini-2 (R-01). |
| D-25 | Belief nach Merge neu aus dem deduplizierten Evidenzset berechnen | Claude 5.3 (Korrektur an additiven Zählern) |
| D-26 | Merge-Gates G1 bis G7 | Claude 4.4 |
| D-27 | Decay wirkt nur auf Ranking, löscht nie | ChatGPT 5.4, Claude 4.5 |
| D-28 | Taint-Tracking gegen Injection | Claude 4.7, ChatGPT 5.5 |
| D-29 | Idempotenz und Action-Log | Claude 4.3, ChatGPT 5.5 |
| D-30 | Scope-Reduktion, Layering-Regeln in CI | Repo-Befund (14.1) |
| D-31 | Eine öffentliche Fassade, `contextra-core` entfällt | Repo-Befund |
| D-32 | Arrow für Export und Batch, rkyv nur für validierte interne Records | ChatGPT 4.6, Perplexity-Vorbehalt |
| D-33 | Deterministische Simulation und Failpoints sind Pflicht | Claude 5.4, ChatGPT 10 |
| D-34 | AEAD at-rest | Claude 3.9 |
| D-35 | Raft erst in v2 | Claude 5.5, Perplexity |

### 17.2 Verworfene Konzepte

| ID | Konzept (Quelle) | Grund der Ablehnung |
|---|---|---|
| R-01 | Chunk-Konflikt wird per Cosine ≥ 0,96 automatisch entschieden (Gemini-2) | Stille Entscheidung über Wahrheit. Konflikt muss sichtbar bleiben. |
| R-02 | Alle Segmente im Produktpfad per mmap (Deepseek, Gemini) | Unkontrollierte Eviction, SIGBUS-Risiko, keine Prefetch-Kontrolle |
| R-03 | Multiplikative Pfadkonfidenz `Π c · γ^(k−1)` (Gemini-2) | Unterschätzt Ketten willkürlich, ignoriert Korroboration. Ersetzt durch Minimum-Bound. |
| R-04 | Unbegrenzte Git-Branches ohne Gate-Pflicht (Grok, Qwen) | Verletzt P4 |
| R-05 | Vektoren als CRDT-Menge mit Historie in Tombstones (Qwen) | Embeddings folgen dem Inhalt und werden nicht zusammengeführt |
| R-06 | Feste U-Form-Platzierung als Standard (Gemini, Perplexity) | Wirkung ist nicht gesichert. Nur als evaluierte Option. |
| R-07 | Klartext-Hashes über Mandanten hinweg (Claude 5.5) | Information leakage. Ersetzt durch D-16. |
| R-08 | Globale Lock-free- und Wait-free-Versprechen (mehrere Entwürfe) | Nicht belegbar. Ersetzt durch präzise Garantien (9.4). |
| R-09 | Ablehnung jeder Antwort ohne Provenienz (Grok) | Verliert Nützlichkeit. Stattdessen Kennzeichnung und Ausschluss im Standard-Read. |
| R-10 | Globale Deduplizierung über Mandanten (ChatGPT 7.5 als Gegenposition) | Datenschutzrisiko |
| R-11 | Exakter 0/1-Knapsack mit DP (Deepseek) | Zu teuer bei 500 Kandidaten. Greedy mit Garantie stattdessen. |
| R-12 | Arrow als primäres Persistenzformat aller Daten (Perplexity, Teile von Gemini) | Key-Value- und Indexstrukturen brauchen optimierte Layouts. Arrow nur für Batches und Export. |

---

## 18. Offene Entscheidungen und Risiken

### 18.1 Offene Entscheidungen

- **D-OPEN-1 Rechtliche Umsetzung der Löschpflicht.** Die Vorgabe schließt Löschnachweise und die Löschkette aus. Die Pflicht zur Löschung personenbezogener Daten nach Art. 17 DSGVO bleibt davon unberührt. Die Spezifikation definiert deshalb nur die technische Operation `purge` (12.3). Die juristische Bewertung, insbesondere zum Umgang mit historischen Commits, muss vor M5 erfolgen.
- **D-OPEN-2 Platzierungs-Default.** Entscheidung auf Basis des A/B-Tests in M4.
- **D-OPEN-3 Aufnahme von `io_uring`.** Entscheidung auf Basis der Benchmarks in M5.
- **D-OPEN-4 Embedding-Ausführung.** Lokal (candle oder ort) gegen externe Provider. Abwägung nach Kosten, Latenz und Datenschutz.
- **D-OPEN-5 Mehrmandantenfähigkeit im Single-Node.** Umfang festlegen.
- **D-OPEN-6 Schwellen und Profile.** τ_A, Selektivitätsgrenzen, Decay-Profile, Fusionsgewichte. Kalibrierung auf einem Referenzkorpus.

### 18.2 Technische Risiken

- **Write-Amplification bei Prolly-Trees.** Im ungünstigsten Fall etwa 30 KB pro geändertem Schlüssel bei Höhe 4 und 8 KiB Knoten. Gegenmaßnahmen: Batching vor dem Seal, Bulk-Load-Pfad, große L0-Segmente.
- **Segment-Fan-out.** Jede Query fragt alle Segmente einer View ab. Gegenmaßnahmen: Obergrenze und Kompaktierung (8.2).
- **Komplexität.** Storage, MVCC, Versionierung und drei Suchmodelle zugleich sind ein großes Projekt. Gegenmaßnahme: M0 wird vor jeder Feature-Arbeit abgenommen (Feature-Freeze bis dahin).
- **Docid-Übersetzung.** Kosten steigen, wenn alte Entitäten häufig überschrieben werden und die Zeitlokalität verloren geht. Gegenmaßnahme: zeitlich ausgerichtete Kompaktierung.
- **Semantische Merge-Konflikte.** Textlich konfliktfreie Merges können semantisch falsch sein, etwa zwei `functional`-Kanten. Gegenmaßnahme: Kardinalitätsgates (G1) und Konfliktscan (G4).
- **Modelldrift.** Embeddings und Judges verändern sich mit Modellwechseln. Gegenmaßnahme: EmbeddingSpace-Versionierung und Dual-Space-Migration.
- **Überversprechen.** Jede nicht gemessene Eigenschaft ist ein Risiko für das Produktversprechen. Gegenmaßnahme: Zielwerte bleiben Hypothesen, bis sie gemessen sind (P8).

---

## 19. Umsetzungsreihenfolge für das bestehende Repository

- **R0 Einfrieren.** Keine neuen Features in experimentellen Modulen. Der Workspace-Baseline-Build muss grün sein. Vorbefund: `harness/claim_guard.rs` referenziert nicht vorhandene Module und bricht den Build von `xtask` auf HEAD.
- **R1 Löschkette entfernen.** `deletion_proof`, `cascade`, Tombstone-Kette und `proof_key` samt Aufrufern.
- **R2 Layering bereinigen.** `contextra-core` aus `store`, `vector` und `simd` entfernen. Inferenz und Sandbox aus `engine` über Ports entkoppeln.
- **R3 Durability bündeln.** `contextra-wal` einführen, verstreute `fsync`-Stellen durch den einzigen Pfad ersetzen.
- **R4 Fassade vereinheitlichen.** `contextra-core` entfernen, eine öffentliche Fassade.
- **R5 Experimente auslagern.** `apprh`, `tl_hfd`, `dibud`, Resonanz, `percolation`, `hyperedge`, `kvcache`.
- **R6 Milestones M0 bis M5** gemäß Abschnitt 16.

---

## Anhang A: Glossar

- **Commit:** unveränderlicher, hash-identifizierter Zustand des Truth-Layers mit Eltern-Verweisen.
- **Ref:** benannter, per CAS aktualisierbarer Zeiger auf einen Commit.
- **Seal:** Übergang einer Branch-Memtable in Packs, Index-Segmente und einen neuen Commit.
- **Pack:** unveränderliche Datei mit content-addressed Chunks.
- **Prolly-Tree:** probabilistischer B-Baum, dessen Struktur allein von der Schlüsselmenge abhängt.
- **Watermark:** Sequenzmarke, bis zu der ein abgeleiteter Index vollständig ist.
- **Session-Branch:** isolierter Branch eines Agenten. Wird nur über Gates in `main` übernommen.
- **source_root:** Wurzel der Ableitungskette eines Fakts. Grundlage der Unabhängigkeitsprüfung.
- **Citation-Token:** session-gebundenes, MAC-geschütztes Handle für einen Beleg.
- **Watermark-Fallback:** Ausweichen auf exakte Suche, wenn ein Index hinter dem Snapshot zurückliegt.
- **Facility Location:** Abdeckungsfunktion, die Auswahl nach Informationsabdeckung statt nach Einzelrang optimiert.

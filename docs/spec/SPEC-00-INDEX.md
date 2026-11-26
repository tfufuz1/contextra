# SPEC-00 — Contextra Spezifikationsindex

**Version 2.1 · November 2026 · Status: Final**

> Diese Spezifikationsreihe beschreibt den **normativen, final konsolidierten Soll-Zustand** von Contextra, einem LLM-nativen, versionierten Storage-System und hybriden RAG-Speicherkern in Rust. Alle Widersprüche und architektonischen Reibungspunkte sind aufgelöst; alle Performance-Zahlen sind als zu validierende Designziele markiert.

---

## Dokumentenstruktur

| Dokument | Inhalt | Fokus |
|---|---|---|
| **SPEC-00** (dieses Dokument) | Index, gelöste Reibungspunkte, Entscheidungsübersicht, Glossar | Navigation & Gesamtbild |
| [**SPEC-01** — Architektur](SPEC-01-ARCHITECTURE.md) | Vision, Profile, Prinzipien, Invarianten, Schichten L0–L6, Soll-Crate-Layout, Entscheidungsregister | Warum und wie |
| [**SPEC-02** — Datenmodell](SPEC-02-DATA-MODEL.md) | Namespaces, entkoppelte Cells (`Content` + `Meta` + `Edge`), `Gid`, MAC-Handles, `BeliefBeta`, Zoom, Bitemporalität | Was wird gespeichert |
| [**SPEC-03** — Storage Engine](SPEC-03-STORAGE-ENGINE.md) | Packs (`.cxp`), PAX-Extents (`Hot-Head`/`Cold-Tail`), `ObjectLocator` + `CellLocator`, Buffer Pool, AEAD (KEK/DEK) | Wie es auf Disk liegt |
| [**SPEC-04** — Versionierung](SPEC-04-VERSIONING.md) | Prolly-Trees, `MemDelta`, `RootSet` vs. `IndexManifestRef`, `Txn`-Typestate, Session-Branches, Merge-Gates `G1`–`G7`, Sidecar-Idempotenz | Wie Zustand verwaltet wird |
| [**SPEC-05** — Indizes](SPEC-05-INDEXES.md) | `Gid↔DocId`, Vektor (Flat/HNSW/Vamana), FTS (FST/BM25F global), Graph (CSR/Forward-Push-PPR), Bitmaps, Token-Summen | Wie gesucht wird |
| [**SPEC-06** — Context Engine](SPEC-06-CONTEXT-ENGINE.md) | Recall-Pipeline, Zoom-Knapsack/MMR, Small-to-Big, Abstention, `VERIFY`, Work-Queue-Sidecar, Retention/Purge | Wie Kontext optimiert wird |
| [**SPEC-07** — Durability & Sicherheit](SPEC-07-DURABILITY.md) | WAL, Manifest-Log, Recovery, SATB-GC, `MemoryBudget`, `TrustTier` + `TaintFlags`, Capabilities, Egress-Guard | Wie Daten geschützt werden |
| [**SPEC-08** — APIs & Protokolle](SPEC-08-API.md) | Rust-API, MCP-Tools (deklarativ klassifiziert), gRPC/Arrow Flight, SQLite-vtab, CLI, Fehlerkatalog | Wie man es benutzt |
| [**SPEC-09** — Betrieb & QS](SPEC-09-OPERATIONS.md) | Ingest-Revisions-Diff, `contextra.toml`, Performance-Hypothesen, `contextra-sim`-Testmatrix, Roadmap, Risiken | Wie man es betreibt & prüft |

---

## Lesepfade

- **Architektur-Review:** SPEC-01 → SPEC-02 → SPEC-03 → SPEC-04
- **RAG & Retrieval:** SPEC-02 (§2–5) → SPEC-05 → SPEC-06
- **Storage & Durability:** SPEC-03 → SPEC-04 → SPEC-07
- **API & Agent-Integration:** SPEC-08 → SPEC-02 (§5 Handles) → SPEC-04 (§4/§6 Gates)
- **Simulation, Testing & Betrieb:** SPEC-09 → SPEC-07 (§3–4)

---

## Aufgelöste architektonische Reibungspunkte (v2.1 Final)

Alle vier strukturellen Reibungspunkte sowie die Widersprüche zu den Begleitkonzepten sind final aufgelöst:

| # | Gelöstes Problem | Finale Lösung in dieser Spezifikation | Dokument |
|---|---|---|---|
| **1** | **Widerspruch `RootSet`-Determinismus vs. Index-Compaction** | Strikte Trennung: **`RootSet`** hasht nur deterministische Truth-Prolly-Bäume (`primary`, `sequences`, `graph`, `aliases`, `policies`, `conflicts`, `provenance`; ohne physische Hints!). Physische Vamana-/FTS-/CSR-Segmente hängen am separaten **`IndexManifestRef`** mit `source_truth_hash` + `watermark_seq` und exaktem Flat-Fallback (`INV-1`, `INV-4`). | SPEC-04 §2, SPEC-05 §1 |
| **2** | **Ephemerer Laufzeitzustand im versionierten `RootSet`** | **`Work Queue`** (inkl. `lease`, `attempts`) und **`Idempotenz-Journal`** sind in **nicht-versionierte Kernel-Sidecars** (`sidecar/work_queue.cxs`, `sidecar/idempotency.cxs`, WAL-gesichert!) ausgelagert. Worker-Leasing erzeugt keine Commits; erst Worker-Ergebnisse werden regulär committet. | SPEC-02 §1.1, SPEC-04 §8, SPEC-06 §7.2 |
| **3** | **Granularitäts-Lücke `Cell`-Hash, PAX-Extents & Re-Layout** | Cell in unveränderlichen **`CellContent`** (`content_hash`), versionierte **`CellMeta`** (`meta_hash`) und **`graph`-Prolly-Tree** entkoppelt → Tagging/`valid_to`-Updates invalidieren weder `raw` noch Embeddings. **`CellLocator`** adressiert pro Cell einmalig `(PackId, ExtentNo, RowIndex: u16)` statt Einzel-Lane-Offsets; Re-Layout biegt nur den Locator um. | SPEC-02 §2.2, SPEC-03 §2–4 |
| **4** | **Erratbare sequentielle Handles (`c:k7qd`)** | Jedes Handle trägt ein 2-Zeichen-BLAKE3-MAC-Tag über `(repo_seed, gid)`: **`c:k7qd-3f`** (10 Bit MAC → schließt Zufallstreffer durch LLM-Halluzinationen zu 99,90 % aus und triggert zuverlässig `did_you_mean`). | SPEC-02 §5, SPEC-06 §4, SPEC-08 §2 |
| **5** | **Bonus-Fix: Keyed ObjectIds über Ciphertext** | Hash-Berechnung bei verschlüsselten Repos nutzt `BLAKE3::keyed_hash(repo_hash_key, plaintext)` statt Ciphertext — schützt vor Cross-Repo-Orakeln, ohne Deduplizierung und Prolly-Sharing durch Nonces zu brechen. | SPEC-03 §8.3 |

---

## Entscheidungsregister (Gesamtübersicht)

| ID | Entscheidung | Dokument |
|---|---|---|
| C-01 | Prolly-Trees + `MemDelta` als deterministischer Truth-Layer | SPEC-01 §7, SPEC-04 §1 |
| C-02 | Trennung `RootSet` (Truth) vs. `IndexManifestRef` (Indizes + Watermarks) | SPEC-04 §2, SPEC-05 §1 |
| C-03 | Entkoppelte Cell (`CellContent` vs. `CellMeta`) & PAX-`CellLocator (PackId, ExtentNo, RowIndex)` | SPEC-02 §2.2, SPEC-03 §3 |
| C-04 | Ephemerer Zustand (`WorkQueue`, `Idempotency`) in WAL-gestützten Kernel-Sidecars | SPEC-04 §8, SPEC-06 §7.2 |
| C-05 | MAC-geschützte Handles `c:<base32>-<mac2>` (`c:k7qd-3f`) | SPEC-02 §5 |
| C-06 | Globales dichtes `Gid(u64)` + segmentlokales `DocId(u32)` für gemeinsame Bitmaps | SPEC-02 §2.1, SPEC-05 §1 |
| C-07 | Keyed CAS-Hash über kanonischen Klartext (`repo_hash_key`) | SPEC-03 §8.3 |
| C-08 | AEAD at Rest mit KEK/DEK-Hierarchie (O(1)-KEK-Rotation) | SPEC-03 §8.1 |
| C-09 | Agenten schreiben nie direkt nach `main`; Session-Branches + Merge-Gates `G1`–`G7` | SPEC-04 §4, §6.3 |
| C-10 | Pflichtprovenienz, `BeliefBeta(α,β)` & `source_root`-Unabhängigkeit gegen Echokammern | SPEC-02 §2.6 |
| C-11 | Orthogonale `TrustTier` (min) + `TaintFlags` (union) + Datenrahmen gegen Injection | SPEC-07 §7.1, SPEC-06 §2.2 |
| C-12 | Ein Commit-Punkt (WAL Group Commit); kein 4-Store-2PC | SPEC-04 §3.2, SPEC-07 §1 |
| C-13 | Zoom-Knapsack + Submodularitäts-Abzug + Small-to-Big + Abstention (`NO_EVIDENCE_FOUND`) | SPEC-06 §3.2, §5 |
| C-14 | Ein Standardalgorithmus je Modalität (HNSW/Vamana, BM25F+BMW global, CSR+Forward-Push-PPR, RRF) | SPEC-05 §2–4 |
| C-15 | Keine Löschkette / Löschnachweise; `retire` + Retention + Best-Effort-`purge` | SPEC-01 §1.4, SPEC-06 §7.3 |
| C-16 | Portables `pread`/`pwrite` + opt. `io_uring`; kein `mmap` im Produktpfad | SPEC-03 §5.2 |
| C-17 | Eigener FTS (FST + Block-Max-WAND + globale BM25F-Stats) statt Tantivy | SPEC-05 §3 |
| C-18 | Sandboxed WASM-Merge/Gates mit Fuel-Limits (`wasmtime`) | SPEC-04 §6.1 |
| C-19 | Eventual-consistent `AccessStats`; Affinity Re-Layout ab Phase 3/4 opt-in | SPEC-02 §8, SPEC-03 §4 |
| C-20 | Typestate `Txn<ReadOnly|ReadWrite>` & `Snapshot<'db>` (gegen unregistrierte SSI-Reads) | SPEC-04 §3.1 |
| C-21 | Embedded-first, Single-Process (`flock`), `contextra-server` als dünne Hülle | SPEC-01 §1.2 |

---

## Glossar

| Begriff | Bedeutung | Definiert in |
|---|---|---|
| **BeliefBeta** | Evidenzbasierte Konfidenz `Beta(α, β)` mit `source_root`-Deduplizierung und 10-%-LCB | SPEC-02 §2.6 |
| **Cell / CellContent / CellMeta** | Entkoppelte Wissenseinheit: unveränderlicher Inhalt (`content_hash`), versionierte Metadaten (`meta_hash`), Graph-Kanten getrennt | SPEC-02 §2.2 |
| **CellId / Gid / DocId** | Logische ID (`NsId+CellKey`), global dichter `u64`-Index (`Gid`) und segmentlokale `u32`-Position (`DocId`) | SPEC-02 §2.1, SPEC-05 §1 |
| **CellLocator / ObjectLocator** | Zweistufiger Locator: `(PackId, ExtentNo, RowIndex)` pro Cell vs. `(PackId, Offset, Len)` für Prolly/Blobs/Segmente | SPEC-03 §3 |
| **ChunkId (`ContentHash`)** | Keyed/plain BLAKE3-256 über kanonische Klartext-Bytes (niemals über Nonce-Ciphertext) | SPEC-02 §2.1, SPEC-03 §8.3 |
| **ConflictCell** | Versionierte Darstellung eines Konflikts auf `mr/*` (niemals ungelöst auf `main`) | SPEC-04 §6.2 |
| **Derivation Cache** | Pro `(content_hash, producer_id)` memoisierte Embeddings, Summaries & Token-Counts | SPEC-06 §7.1 |
| **EvidenceSufficiency** | Bewertung der Evidenzlage (`Sufficient | Thin | Conflicting | None` mit Abstention) | SPEC-06 §5 |
| **Extent (PAX)** | 256 KiB–1 MiB I/O-Einheit mit spaltenorientiertem `Hot-Head` (Ranking) und `Cold-Tail` (Volltext/f16) | SPEC-03 §2.2 |
| **GroundingLevel** | Belegstufe einer Aussage: `G0` (verifizierte Spanne) bis `G3` (`Hypothesis`, Quarantäne) | SPEC-02 §2.6 |
| **Handle (MAC)** | Token-sparsamer, 10-bit-MAC-geschützter Alias `c:<base32>-<mac2>`, z. B. `c:k7qd-3f` | SPEC-02 §5 |
| **IndexManifestRef** | Separater Seitenzeiger auf physische Vamana-/FTS-/CSR-Segmente mit `watermark_seq` (nicht im `RootSet`-Hash!) | SPEC-04 §2, SPEC-05 §1 |
| **MemDelta** | Branch-lokale In-Memory-Skip-List vor dem asynchronen Prolly-Fold | SPEC-04 §1.4 |
| **Merge-Gates (`G1`–`G7`)** | Deterministische Qualitäts-/Sicherheitsprüfungen vor Promotion in geschützte Branches (`main`) | SPEC-04 §6.3 |
| **Prolly-Tree** | History-unabhängiger, inhaltsadressierter B-Baum (ohne physische Hints im Hash) | SPEC-04 §1 |
| **Retire / Purge** | Versionierter Tombstone (`retire`) bzw. Best-Effort-Retention-Override (`purge`) ohne Löschnachweise | SPEC-06 §7.3 |
| **RootSet** | Rein logische, 100 % deterministische Prolly-Wurzeln (`primary`, `sequences`, `graph`, `aliases`, `policies`, `conflicts`, `provenance`) | SPEC-04 §2 |
| **SATB + Remark** | Snapshot-at-the-Beginning GC-Protokoll mit Write-Barrier | SPEC-07 §4 |
| **TrustTier & TaintFlags** | Vertrauensstufen (min-vererbt) kombiniert mit orthogonalen Kontaminations-Flags (union-vererbt) | SPEC-07 §7.1 |
| **Watermark-Fallback** | Exakter Scan über Zeilen `> watermark_seq` bei noch nicht gebauten Index-Segmenten | SPEC-05 §1.2 |
| **Work Queue (Sidecar)** | WAL-gesicherte, nicht-versionierte Job-Tabelle (`sidecar/work_queue.cxs`) für externe Worker | SPEC-06 §7.2 |
| **Zoom-Knapsack** | Multiple-Choice-Knapsack + Submodularitäts-Abzug + Small-to-Big unter hartem Token-Budget | SPEC-06 §3.2 |

---

*Letzte Aktualisierung: November 2026 (v2.1 Final). Alle Widersprüche und architektonischen Reibungspunkte aufgelöst.*

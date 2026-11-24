# SPEC.md — Contextra Gesamtsystem-Spezifikation (Speicherkern, Retrieval, Engine & Löscharchitektur), Revision R10

| | |
|---|---|
| Status | **Revision R10 (Gesamtsystem-Spezifikation), Entwurf zur Freigabe als Gate G-0.** Integriert die Speicherkern-Revision **R9.1+Ö** (Konsolidierung aus R7, Lethe-Architekturentwurf, Lethe-Engine-Spec, Mnemos, „Optimal-LLM-Datenbank“ und Octa-Abgleich) vollständig mit den übrigen Workspace-Crates (`contextra-engine`, `contextra-vector`, `contextra-text`, `contextra-graph`, `contextra-rank`, `contextra-checkpoint`, `contextra-mvcc`, `contextra-crypto`, `contextra-simd`, `contextra-sys`, `contextra-durable-fs`) sowie der in `capabilities.toml` definierten Ring-Topologie. |
| Gegenstand | Eingebettetes, crash-sicheres Gesamtsystem für europäische Unternehmens-LLM- und Agenten-Workloads: synchroner Speicherkern (`contextra-storage` als Ersatz für `contextra-store`) mit atomaren Mehr-Space-Änderungen, MVCC-Sichten, clusterlokalen Branches/Merges, mehrkanaligem Hybrid-Retrieval (Dense Vector HNSW/DiskANN + BM25/WAND + Wissensgraph CSR/Hyperkanten/PPR + 4-/5-Signal-Fusion & Kalibrierung), Agenten-Langzeitgedächtnis und **durchgängiger, nachweisbarer Löschung personenbezogener Daten** über Quell- und Indexschichten hinweg (logisch, kryptografisch, physisch). |
| Verifikation | **Storage-Kern:** Teilchecks (Storage-/Crypto-Compile, ausgewählte Config-, Catalog-/Redo-Crash-Sweeps und Strukturtests in `crates/contextra-storage/tests/`) existieren; vollständige Release-Gates (TLA+ M1–M6, Fuzzing, Referenzvektoren, Voll-Orakel) sind offen. **Gesamtsystem (Stand Commit `0dd5cfc41`):** Die im Workspace aktiven Crates (`contextra-engine`, `contextra-vector`, `contextra-text`, `contextra-graph`, `contextra-checkpoint`, `contextra-crypto`, `contextra-mvcc`) referenzieren teilweise noch Alt-Abhängigkeiten (`contextra-store`, `contextra-ports`, `contextra-types`) und sind noch nicht auf `contextra-storage` geschnitten (§14.1, §18.1). Keine Leistungszahl (H-1…H-35) gilt vor Gate-Abnahme als gemessen. |
| Rechtlicher Vorbehalt | Das System liefert technische Eigenschaften (atomare Sichtbarkeitsbarriere, Schlüsselvernichtung, physische Dateibereinigung, signierbare Löschprotokolle), keine Rechtsaussage zu Art. 17 DSGVO, gesetzlichen Fristen oder Ausnahmen. „Gelöscht“ ist immer an eine nachgewiesene Stufe aus §9.2 und ein konkretes `ErasureReceipt` gebunden. |
| Normative Quelle | **Diese Datei ist die einzige normative Gesamtsystem-Fassung.** Bestehende Modul-Specs unter `crates/contextra-storage/`, `capabilities.toml` und `CLAUDE.md` verweisen hierher oder sind vor G-0 an diese Fassung anzugleichen. Rangfolge bei Widerspruch: **§15 (Invarianten) vor §12.8 (Audit-Ergänzungen) vor §12 (Erweiterungs-/Retrieval-Integration) vor den übrigen Abschnitten.** |
| Sprache | Prosa deutsch, Bezeichner englisch. |
| R10-Gesamtsystem-Audit | Schließt die architektonische Lücke zwischen isolierter Storage-Planung (R9.1+Ö) und den Retrieval-/Engine-/Agenten-Crates: Ablösung der fehleranfälligen 4-Wege-2PC (`DbTransaction`/`CommitLedger`/`CompensatingAction`) zugunsten atomarer Single-Frame-Commits plus `ConsumerFeed`-Projektion; Beseitigung dualer MVCC- und Krypto-Pfade (`contextra-mvcc`/`aes-gcm-siv`/`blake3`); Schließung von `memmap2`/Raw-FS-Bypässen und harten Integritätsschlüsseln in `contextra-vector`; Schutz vor Term-Dictionary-Leaks in `contextra-text`; klare Trennung von Storage-`DerivedFrom`-Kaskade, Wissensgraph (`CsrGraph`), Session-DAG (`SessionBranchTree`), kognitivem Decay (`AdaptiveDecayController`) und DSGVO-Purge. |

---

## 0. Kurzfassung

**Contextra ist eine europäische Datenbank- und Gedächtnisplattform für produktive Unternehmens-LLM-Systeme und autonome KI-Agenten, in deren Speicher- und Indexfundament permanenter Datenschutz und belegbare Löschbarkeit (Art. 17 DSGVO) von Anfang an verankert sind.**

Während die Vorgänger-Architektur auf einem generischen LSM-Tree (`contextra-store`) plus lose gekoppelter Zwei-Phasen-Kompensation (`DbTransaction` + `CommitLedger`) über separate Vektor-, Volltext- und Graph-Indizes aufbaute, basiert Contextra ab R10 auf dem vollständig neu entwickelten Speicherkern **`contextra-storage`**:

1. **Ein Fundament für Dokumente, Vektoren, Text-Metadaten, Graph-Fakten und Agenten-Zustände:** Ein fachlicher Schreibvorgang (`Change`) schreibt kanonische Dokumentdaten, quantisierte/kompakte Vektorcodes, Vorwärts-Termdaten, Graph-Fakten und Bitemporalitäts-Kopfdaten **in einem einzigen Redo-Frame mit genau einem Gruppen-Fsync** atomar in die Spaces eines Clusters. Verteilte Kompensations-Transaktionen zwischen Storage und Indizes entfallen vollständig.
2. **Ein Cluster ist die universelle Lebens-, Versions-, Schlüssel- und Löschdomäne:** Alles zu einer Person, einem Mandanten-Fall oder einem Arbeitskontext liegt in einem `Cluster` (`ClusterId`, 128-Bit-Zufall). Der Cluster ist zugleich **Löschdomäne, Schlüsseldomäne, Versionsdomäne (`BranchId`), und physische Platzierungseinheit**. Branches für Agenten-Experimente (`SessionBranchTree`) sind clusterlokale O(1)-Forks und sterben automatisch mit ihrem Cluster.
3. **Mehrkanaliges Hybrid-Retrieval ohne Konsistenz- oder Löschlücken:** `contextra-vector` (HNSW, DiskANN, SQ8, RaBitQ, ACORN, Partial-Rebuild), `contextra-text` (BM25/BM25F, deutsche Morphologie & Komposita-Zerlegung, Block-Max WAND), `contextra-graph` (CSR, n-äre Hyperkanten, PPR/APPRH/TL-HFD, PathRAG) und `contextra-rank` (4-/5-Signal-Fusion, Isotonic/Platt/Conformal Kalibrierung, Drift-Erkennung, Streaming-Budget-Fusion `DiBud`) operieren als strikt vertragsgebundene obere Schichten (`SegmentOwner`, `ConsumerFeed`, `Index-Watermark`, `View::filter_visible`).
4. **Löschung in drei deterministischen, geordneten Schritten:**
   - **Gate (`ER1`, Millisekunden):** Cluster ab Stamp `S` für jede Sicht – einschließlich künftiger und historischer Sichten – atomar `Erased`; alle Retrieval-Owner maskieren Kandidaten synchron über `on_gate` und `View::filter_visible` vor Scoring und Rückgabe.
   - **Schlüsselvernichtung (`ER2`, `Cryptographic`):** Alle Epochen des Clusterschlüssels werden über `KeyProvider::destroy_many` vernichtet (`probe == Absent`). Sobald abgeschlossen, sind alle Kerndaten (Redo, Runs, Blobs) sowie cluster-sealed Index-Payloads ohne Schlüssel unlesbar (`INV-ERA-7`).
   - **Physischer Purge (`ER5–ER6`, fristgebunden & amortisiert):** Lebende Run-/Blob-Dateien und registrierte `Rebuildable`-Indexsegmente (`Hnsw`, `DiskANN`, `InvertedIndex`, `CsrGraph`) werden per Rohkopie bzw. segmentweisem `rewrite_without` bereinigt und mit `verify_absent` + `ErasureReceipt` bis zum `Anchor` und `Ledger` attestiert.
5. **Klare Open-Core- und Compliance-Grenzen:** Formate, Kernsemantik, Port-Traits, Simulations-/Test-Orakel und Basis-Löschung bleiben offen und verifizierbar (§2.6).

| Systemschicht | Verantwortung | Darf **niemals** |
|---|---|---|
| **Speicherkern** (`contextra-storage`) | Atomare Multi-Space-Changes, Sequencer+Syncer, Stamp-MVCC, clusterlokale Branches, Three-Step-Erase (`Gate < KeyDestroy < Purge`), Ledger, Anchor-Binding, Budget-Paging (`header`/`weights`/`page`) | Tokenizer, Embeddings, ANN, BM25, PPR, Reranking, UI-/Rechtspolitik oder eigene Krypto-/FS-Syscalls implementieren |
| **Retrieval-Triebwerke** (`vector`, `text`, `graph`, `rank`, `simd`) | HNSW/DiskANN, SQ8/RaBitQ, ACORN, BM25/WAND, deutsche Komposita, CSR/Hyperkanten, RRF/Kalibrierung/DiBud; **verbindliche `SegmentOwner`- & `ConsumerFeed`-Konformanz** | `memmap2`-Bypässe an Storage/Sealing vorbei auf Platte schreiben, globale Klartext-Wörterbücher ohne Löschbindung halten oder `filter_visible` umgehen |
| **Orchestrierung & Agenten** (`engine`, `checkpoint`, `agent`, `cognition`, `router`, `kvcache`) | `Collection` → `SpaceId`-Mapping, `TenantPolicy`, Hybrid-Query-Planung (`Index ∪ Delta`), Ingestion- & Privacy-Pipeline, Session-DAG ↔ Branch-Steuerung, kognitiver Decay | Eigenes Pseudo-2PC aufspannen, `AdaptiveDecay` als DSGVO-Löschung ausgeben oder unbegrenzte Snapshot-Pins leaken |
| **Sicherheit & Compliance** (`privacy`, `prompt-guard`, `sandbox`, `crypto-ref`, `keys-*`, `erasure`) | PII-Erkennung & -Maskierung, `XChaCha20-Poly1305`/`HMAC-SHA256`/`HKDF`/`CRC32C`, Argon2id/HSM/KMS-Keys, Ed25519-signierte `DeletionProof`s aus `ErasureReceipt` + Owner-Attesten | Klartext-ClusterIds ins Ledger schreiben, schwache Schlüssel still als `Complete` attestieren |

**Was weder Speicherkern noch Retrieval-Schicht allein garantieren können:** Prozessspeicher-Reste im Allokator, unverschlüsselter Swap, Core-Dumps, Dateisystem-Journale (ext4/xfs/APFS), SSD-Flash-Wear-Leveling, externe Backups mit Schlüsselkopien oder externe Cloud-LLM-Provider-Logs außerhalb der Contextra-Pipeline. Jedes `ErasureReceipt` weist diese Restexposition explizit nach Befund (§9.9, §13.2.1) aus.

---

## 1. Konfliktregister: Entscheidungen der Konsolidierung

Die Konsolidierung vereint acht Speicherkern-Quellen (`K-01`…`K-38`) sowie neun strukturelle Architekturkonflikte zwischen dem bisherigen Gesamtsystem (`engine`/`vector`/`text`/`graph`/`rank`/`checkpoint`/`mvcc`/`crypto`) und dem neuen Speicherkern (`K-39`…`K-47`). Jede Entscheidung ist final; die Spalte „Wirkung“ nennt den umsetzenden Abschnitt.

| ID | Konflikt | Entscheidung | Begründung | Wirkung |
|---|---|---|---|---|
| K-01 | Löscheinheit heißt „Cluster“ (R7, Lethe-Entwurf) bzw. „EU“ (Lethe-Spec) | **Cluster** | ein Begriff, R7 ist Referenz | §4 |
| K-02 | ClusterId: Aufrufer-ID / HMAC-`eu_tag` (Lethe-Spec, R6) vs. vom Kern erzeugt (R7) | **Kern erzeugt 128-Bit-Zufall**, kein `eu_tag`, kein `tenant_tag_key` | keine Wiederverwendung, nicht ableitbar, kein zusätzliches Schlüsselmaterial im Klartext-Pfad | §4.3 |
| K-03 | Ledger speichert ClusterId (R7) vs. pseudonymes `tomb_id` (Lethe-Entwurf) | **domain-separiertes `tomb_id`** | kein Klartext-Identifier im dauerhaften Ledger; HMAC bleibt linkbares Pseudonym und wird als personenbezogenes Metadatum behandelt | §9.7 |
| K-04 | Schlüssel: Cohort-KEK-Rewrap (Lethe-Spec) vs. Schlüssel je Cluster (R7) | **Port `KeyProvider` (je Cluster adressiert)**; Cohort-Rewrap ist *Strategie* in `contextra-keys-cohort` | Kern bleibt einfach; Skalierung auf Millionen Cluster bleibt möglich | §9.4 |
| K-05 | Ein globaler Commit-Log (Lethe) vs. Redo mit Committer und Catalog-Edits (R7) | **R7-Modell** | R7 ist bis zur Absturzmatrix durchgearbeitet | §6, §10 |
| K-06 | Segmentschnitt bei jedem Gate (R6) vs. keiner (R7) | **Kein Schnitt**; Barriere nur beim Purge von `Plain` | Commit-Pfad bleibt frei von Löschkosten | §9.5 |
| K-07 | Versionierung: Prolly-Tree/Merkle-DAG (Mnemos, Engram, Aether, Cairn) vs. R7 NZ-13 | **Clusterlokale Branches im Kern**, **ohne** Merkle/Inhaltsadressierung | Branches ohne Dedup sind löschlokal; ein globaler DAG über Cluster würde Löschung unmöglich machen | §8 |
| K-08 | Inhaltsadressierung/Dedup (Engram, Aether, Mnemos) | **Nein** (`INV-PRIV-1`) | Hash über Klartext = Seitenkanal, Dedup = Querverweis zwischen Löschdomänen | §3.3 |
| K-09 | „Hold“ als Rechtsfunktion mit Vier-Augen-Release vs. `Restriction` ohne Override (R7) | **`Restriction`** (R7) mit wählbarer `LiftPolicy` (`OwnerOrAdmin` oder `FourEyes` über Port `Authority`); `Hold` bleibt **technisch** | Mechanismus ohne Politik, aber Vier-Augen-Prinzip verfügbar | §9.6 |
| K-10 | Commit ohne WAL per CAS-Superblock (Aether) | **Verworfen** | Mehr-Space-Atomarität und Index-Deltas brauchen ein sequenziertes Redo-Log | §6 |
| K-11 | `mmap`, `io_uring`, `O_DIRECT`, GPUDirect (Aether, Engram) | **Nur als `Fs`-Implementierungen**, nie im Speicherkern oder unversiegelt in Indizes | Fehlerinjektion muss jeden Zugriff sehen; `mmap` umgeht AAD/Checksums/Gate-Fencing | §3.1, §12.4 |
| K-12 | Async-Runtime (Mnemos, Cairn) | **Synchroner Kern (`contextra-storage`)**, Async-Fassade in `contextra-storage-bridge` / `contextra-engine` | Race-Freiheit deterministisch per `loom`/Sim prüfbar | §3.4 |
| K-13 | Token-IDs, KV-Cache-Blöcke, feste `AlignedContextPage` im Kern | **Verworfen im Kern**; Token-Gewichte als `weights[W]`; KV-Verweise in `contextra-kvcache` / Kopf oben | Formatbruch bei Modellwechsel; Cloud-APIs erlauben keinen KV-Zugriff | §4.5, §12.5 |
| K-14 | Zoom-Leiter L0…Ln (Mnemos, Engram) | **Konvention oben**: Zoom-Stufen sind Schlüsselsuffixe im selben Cluster | Kern bleibt bedeutungsfrei; Löschung erfasst alle Stufen automatisch | §12.6 |
| K-15 | Trust-/Taint-Labels (Mnemos, Engram) | **Konvention oben** (erstes Kopfbyte), Vererbung = Minimum über `DerivedFrom` | Politik, nicht Mechanismus | §12.6 |
| K-16 | Indizes im Kern (Cairn, Engram, Aether) | **Oben als eigene Crates (`vector`, `text`, `graph`, `rank`)**, über Verträge (`SegmentOwner`, `ConsumerFeed`, **Watermarks**, `filter_visible`) | Löschbarkeit erzwingt Segmentverträge; Indexalgorithmen bleiben modular | §12 |
| K-17 | Gelerntes Affinitäts-Layout über Objekte hinweg (Mnemos, Engram, Grok) | **Verworfen**; Layoutfreiheit nur *innerhalb* eines Cluster-Abschnitts | kollidiert mit Clusterreinheit (`INV-PL-2`) | §5 |
| K-18 | Single-Sequencer als Engpass (Cairn-Kritik, Lethe-Spec B.5) | **Ein Committer je Instanz**; Skalierung über Tenant-Sharding oberhalb | ehrlich benannte Grenze | §11 |
| K-19 | Durability `Sync/GroupSync/Async` (Cairn) vs. `Durable/Relaxed` (R7) | **`Durable` (Gruppen-Fsync) / `Relaxed`** | gleiche Semantik, weniger Modi; `Relaxed` nie für Löschung | §6 |
| K-20 | Slab-Sub-Allokator mit Free-Lists (Grok) | **Verworfen** | veränderliche Free-Lists widersprechen unveränderlichen Dateien und Rohkopie | — |
| K-21 | Guard-Prüfung im Committer (kalte E/A unter Sperre) | **Zweistufig** (Stufe A beim Aufrufer exakt, Stufe B im Sequencer nur RAM-Delta) | keine Plattenlesezugriffe unter Rang 1 | §6.3 |
| K-22 | Ledger im Datenvolume (R7) vs. eigene Failure Domain (Lethe) | **`ledger_dir` konfigurierbar**, Produktivempfehlung & Prod-Gate: getrenntes Volume | vereint eingebetteten Default mit Compliance-Trennung | §9.7, §10.3 |
| K-23 | Stufen G0/G1/G2, `Logical/Physical/Scrubbed/Cryptographic` | **Zwei Achsen**: `EraseLevel` = gewünschter Endzustand; `Stage` = Fortschritt (`Gated`=G0, `KeysDestroyed`=G1, `Purged`/`Done`=G2) | keine Doppeldeutigkeit | §9.2 |
| K-24 | Zuordnung Person → Cluster | **Normatives Muster in §12.7**: Sealed-Wurzel je Person plus registrierter `IdentityDirectoryOwner` mit tenant-scoped HMAC + Entry-DEK | verhindert gemeinsame Shard-DEKs und Überlöschung | §12.7 |
| K-25 | Daten mehrerer Betroffener | **Gemeinsamer Fallcluster + Links** (R7); keine mehrclustrigen Einträge | Purge ohne Querverweise | §9.8 |
| K-26 | Bitemporalität (Mnemos, Cairn: vier Zeitachsen) | **Stamp = Transaktionszeit im Kern**; Gültigkeits-/Beobachtungs-/Quellzeit im Kopf der oberen Schicht | Kern ohne Anwendungs-Zeitlogik; ermöglicht präzises `BlockPruner`-Skipping | §8.6, §12.6 |
| K-27 | Siegelformen (Redo je Segment, Block mit `file_id`) | **Eine Siegelform ohne Datei-/Positionsbezug** | Rohkopie in jedem Modus gültig | §5.2 |
| K-28 | R7-Redaktionsfehler: doppelte `SD-38/39/40`, Rang 5 „entfällt“ | **Neu nummeriert** (§19), Ränge 1–8 durchgehend | Konsistenz | §11.2, §19 |
| K-29 | Zeitlanes `lane_min/max` je Block im SubIndex | **Nein im Kernformat.** Stattdessen Erweiterungsvertrag `BlockPruner` über `ExtensionBlock` | Blockskipping unter MVCC nur bei schattensicheren Quellen korrekt | §12.2, §8.6 |
| K-30 | Kumulative `cum_weight[W]`/`cum_count` im SubIndex | **Nein im Format.** Präfixsummen beim Laden des SubIndex im RAM berechnet | SubIndex ≤ ~4096 Blöcke je Abschnitt; vermeidet Format-Bloat | §7 |
| K-31 | `tomb_id` bleibt bis ans Ledger-Ende ein linkbares Pseudonym | **`tomb_epoch u32` als Format-Slot**; Tomb-Schlüssel im `KeyProvider` (nie in `KEYS`/Backups); optionaler Retire nach `backup_horizon` | nach Horizont anonym ohne Kettenverlust | §4.1, §9.4, §9.7, §10.4 |
| K-32 | Derivation Cache (Embedding-/Token-Memoisierung) vs. `INV-PRIV-1` | **Ja als Konvention oben**: Keyed-MAC unter zufälligem, clusterlokalem `cache_secret` im `Sealed`-Cluster | verhindert Re-Embedding ohne Cross-Domain-Seitenkanäle | §3.3, §12.6 |
| K-33 | Committer hielt Rang 1 über Redo-Sync, Guard-Langsampfad, Hüllen-Traversierung & Catalog-Fallback | **Pipeline Sequencer (Rang 1, nur RAM) + Syncer (ein Schreiber, kein Lock)**; Vorarbeit auf Aufruferthread; `policy_epoch` | konstante niedrige Tail-Latenz trotz Gate-/Guard-Verkehr | §6.1, §6.3, §9.5, §9.6, §11 |
| K-34 | Bloom ohne `branch` ⇒ Block-Reads je Branch-Ebene | **Branch-bewusster Filter `(branch, user_key)`, `BranchTableBlock`, Chain-Resolve mit Early-Exit, Default-Tiefe 4** | O(1) RAM-Probe statt unnötiger Plattenlesungen | §5.6, §7.1, §8.3 |
| K-35 | Metadaten-RAM ∝ Cluster × Dateien, Kleinstcluster mit 6 Siegelblöcken | **`ClusterIndex` ≤ 40 B, Sparse-Directory, `HotRecord`, `MiniSection`, Admission vor Allokation, „Session ≠ Cluster“** | 16 Mio. Cluster bleiben in ~610 MiB RAM | §4.3.1, §5.6, §12.6 |
| K-36 | Packgruppen-Write-Amplification & Einzel-Purge-Kosten | **Staging-Spill-Policy, Fenster-Amortisierung, Compaction-Huckepack, Hygiene-Lane für Sealed-Purge** | begrenzte Schreibverstärkung, keine Verdrängung | R-PL-7, §9.5 |
| K-37 | `ClusterLocal` ohne Admission ⇒ Inode-/FD-Explosion, „nie zurück“ | **Mindestgröße, Quoten, Pure-Demotion, FD-Cache statt FD-Limit, Verzeichnis-Sharding** | Ressourcenschutz ohne stille Herabstufung | §4.3.1, §5.4 |
| K-38 | Idempotenz-Prüfung als Sonderpfad im kritischen Pfad | **Implizite `Absent`-Guard auf `SYS_IDEM`-Schlüssel** | ein Zwei-Stufen-Mechanismus für Guards & Idempotenz | §6.5 |
| **K-39** | **Engine-Schreibpfad: `DbTransaction` + `CommitLedger` + 4-Wege-Kompensation (`LsmStorage`, HNSW, BM25, CSR) vs. Single-Source-Atomarität** | **Abschaffung des Pseudo-2PC:** Atomares Schreiben aller kanonischen Quell-, Vektor-, Vorwärts-Text- und Graph-Fakten einer Löschdomäne als **ein** `Change` in `contextra-storage`; abgeleitete globale/partitionierte Indizes aktualisieren asynchron über `ConsumerFeed` + Watermark + Delta-Merge | beseitigt Split-Brain zwischen 4 Stores, Kompensations-Races (`INV-DB-3`), JSON-Intent-Overhead und Lock-Kaskaden | §12.4.0, §12.4.5, §14.1 |
| **K-40** | **Duale MVCC-Systeme (`contextra-mvcc`: `TxId`/`SequenceLog`/`SsiValidator` vs. `contextra-storage`: `Stamp` + 2-Stufen-Guards)** | **`contextra-storage` liefert die einzige produktive MVCC-Wahrheit (`Stamp ≡ Epoch(u64)`)** und optimistische Punkt-/Range-Guards; Retrieval-Crates nutzen abstraktes `Epoch(u64)`; `contextra-mvcc` wird ab G-CUTOVER für Kernpfade abgelöst (nur optionaler Standalone-In-Memory-Modus verbleibt bis Entfall) | verhindert `TxId`-vs-`Stamp`-Drift, doppelte Snapshot-Pins (`PinGuard` vs `View-Lease`) und konfligierende Lock-Hierarchien | §12.4.0, §14.1 |
| **K-41** | **Duale Krypto & Erasure (`contextra-crypto`: `aes-gcm-siv`/`blake3`/`deletion_proof`/`kv_shredding` vs. `contextra-crypto-ref`/`XChaCha20-Poly1305` + `contextra-erasure`)** | **Krypto-Vereinheitlichung auf `XChaCha20-Poly1305` + `HMAC-SHA256` + `HKDF-SHA256` + `CRC32C`** für Persistenz & Indizes; `blake3` und `aes-gcm-siv` werden im Speicher-/Index-Pfad entfernt; `DeletionProof` wird durch `contextra-erasure` über `ed25519-dalek` (`verify_strict`) aus `ErasureReceipt` + `EraseAttestation`s gebaut | ein Audit-Pfad, keine inkonsistenten Nonce-/Key-Domänen, saubere Trennung zwischen Storage-Siegel und Compliance-Signatur | §3.7, §9.4, §12.8, §14.1 |
| **K-42** | **Vektor-Persistenz (`contextra-vector`: direktes `std::fs` + `memmap2` + statisches Literal `DISKANN_INTEGRITY_KEY` vs. versiegelter Speicher ohne `mmap`)** | **Verbot von direktem `std::fs`/`memmap2`/hartcodierten Keys in Produktivpfaden:** Modus A speichert Mikro-HNSW/Vektoren direkt als `ClusterSections`; Modus B betreibt HNSW/DiskANN als partitionierten `SegmentOwner` (`Rebuildable`) über `SegmentIo` + `BlockTransform` (`XChaCha20-Poly1305`); bestehendes `HnswPurger`/`GhostFreeVectorIndex`/`Zeroize`/`INV-HNSW-SLOT-1` bildet die RAM-/Arena-Bereinigung | macht `FaultFs`-Crash-Sweeps, Schlüssel-Rotation, mandantenreine Versiegelung und nachweisbaren Vektor-Purge erst möglich | §12.4.1, §14.1 |
| **K-43** | **Text-Index (`contextra-text`: globales `commit_lock` um `meta:stats`, `bincode`, globale `pl:`/`plb:`-Keys inkl. seltener Komposita)** | **Aufspaltung:** Forward-Daten (`fw:`/`dl:`) liegen clusterlokal in `ClusterSections`; Inverted/WAND-Postings (`plb`) laufen mandantenpartitioniert im `SegmentOwner` (`Rebuildable`) mit Refcount-Bereinigung verwaister seltener Komposita (`GermanCompoundSplitter`); `meta:stats` werden je Partition eventual-konsistent ohne globalen `commit_lock` fortgeschrieben; `bincode` entfällt zugunsten kanonischer Codecs | verhindert globale Lock-Engpässe bei Multi-Agent-Writes und eliminiert Metadaten-Leaks über seltene deutsche Komposita | §12.4.2, §14.1 |
| **K-44** | **Graph-Kaskaden: `DerivedFrom` (Storage) vs. `CsrGraph`/`HyperEdge`/`cascade.rs` (`contextra-graph`)** | **Strikte Dual-Graph-Trennung:** (1) `contextra-storage` hält ausschließlich den begrenzten `DerivedFrom`-Lösch-DAG (`max_links=8`, `max_gate_clusters=4096`); (2) `contextra-graph` hält semantische Fakten/Hyperkanten, trägt aber verpflichtend `(source_cluster, source_doc_id)` als Provenienz, trennt inhaltliches `Supersedes` vom DSGVO-Purge, und räumt per `topological_sort_hyperedge_closure` + `sweep_orphans` + Entity-`Zeroize` + Segment-Rewrite auf | schließt „Tombstone-only“-Datenreste in CSR-Arrays/Hyperkanten und verhindert unzulässige fachliche Löschlawinen | §9.6, §12.4.3 |
| **K-45** | **Agenten-Session-DAG (`SessionBranchTree`) vs. Storage-Branches (`SYS_BRANCH`)** | **Integration nach K-35 („Session ≠ Cluster“):** Knoten des Gesprächsbaums liegen präfix-strukturiert im Eigentümer-Cluster; spekulative Explorationspfade nutzen clusterlokale Storage-Branches (`fork(Head|At(Stamp))`, `ttl`, `branch_drop`, `merge`) statt separater `bincode`-Blobs in `contextra-graph` | Agenten-Verzweigung wird crash-sicher, O(1) ohne Kopien und atomar löschbar | §8, §12.6 |
| **K-46** | **Agenten-Checkpoints (`contextra-checkpoint`) & Kognitiver Decay (`AdaptiveDecayController`) vs. DB-Backup & DSGVO-Löschung** | **Dreiteilung der Konzepte:** (1) DB-Snapshot/Restore = `contextra-storage::checkpoint` (ohne Schlüssel/Ledger, mit `erase_epoch`); (2) `contextra-checkpoint` (`CheckpointGuard`/`StateCheckpoint`) speichert Agenten-Zustände clusterlokal auf `StorageView` + `View-Lease`/`Hold` (ohne unbegrenztes SSTable-Pinning und ohne `blake3`-Klartext-Manifeste); (3) `AdaptiveDecayController` steuert nur Relevanz-Decay/Konsolidierung, nie DSGVO-Löschung | verhindert GC-/Purge-Blockaden durch vergessene `PinGuard`s und schließt Verwechslung von Vergessen mit rechtlicher Löschung | §10.3, §11.4, §12.6 |
| **K-47** | **Workspace-/Ring-Inkonsistenz (`Cargo.toml` 12 Crates vs. `capabilities.toml` 30+ Crates, inaktive Root-Checks)** | **Verbindliche Topologie-Synchronisation:** `Cargo.toml`, `capabilities.toml`, `deny.toml` und eingebundene CI-Gates (`layering`, `unsafe_islands`) werden ab P0/P1 synchronisiert; nicht existierende Storage-Ports/Adapter-Crates werden sauber als Subcrates angelegt und verifiziert (§14.1, §18.1) | beendet „tote“ Architektur-Checks und Ghost-Referenzen in Crates | §3.1, §14.1, §18.1 |

### 1.1 Herkunft der übernommenen Elemente

| Quelle | Übernommen | Nicht übernommen / Abgelöst |
|---|---|---|
| **R7 (`contextra-storage` Basis)** | Gesamtgerüst: Cluster, Sealed/Plain, Pure/Packed/Staging, Sequencer+Syncer, Catalog-Edits, ER0–ER7, Anker, Notbetrieb, Absturzmatrix, Invarianten, TLA+-Gates | `rekey_clone`, Aufrufer-ClusterId |
| **Lethe-Entwurf & Lethe-Spec** | G0/G1/G2-Stufen, `Redacted`/`Erased` im Change-Feed, `tomb_id`, `DerivedSegment::on_gate/verify_absent`, Person→Wurzelcluster-Muster, Orakel O-Files/…/O-Segments, `ErasureReceipt` & `verify_receipt`, `KeyUnavailable`/`KeyLost`, Vier-Augen-Lift | eigene Redo-/WAL-Struktur, Cohort als Kernlogik, `eu_tag` |
| **Mnemos & Engram** | Zoom/Handles/Trust/Derivation als Konventionen oben, Fork-TTL, token-budgetiertes `page()`/`weights`, `Error::hint()`, Evaluationsmethodik `Utility@Budget` | Prolly-Tree, Content-Addressing/Merkle-DAG, KV-Tensor-Layouts im Kern |
| **CairnDB & AetherStore** | **Index-Watermarks** (`applied_position`) als Pflichtmuster, Guard-Katalog, 3 Benchmark-Gruppen, WiscKey-Wertetrennung, Mandanten-Vorfilterung vor ANN, Typ-Alignment optional | CAS ohne WAL, GPUDirect, `mmap` im Lesepfad, Slab-Allokator |
| **Octa-Spec & R9/R9.1+Ö-Audit** | Tomb-Epochen/Backup-Horizont (A3), `HostProbe`, `MeasureKind`, `View-Lease`, `Poisoned`-Ledger-Intent, Pipeline ohne E/A unter Rang 1, branch-bewusster Filter, `ClusterIndex ≤ 40B`, `MiniSection`, Spill-Policy, `deps-gate`/`deny.toml` | Zeitlanes/Kumulativsummen im SubIndex (A1/A2 → `BlockPruner` & RAM-Präfixsummen) |
| **Contextra Bestands-Crates (`0dd5cfc41`)** | `contextra-simd` (AVX-512/AVX2/NEON Runtime-Dispatch + NaN-Guard), `contextra-vector` (HNSW, DiskANN-Algorithmik, SQ8+Sq8Bias, RaBitQ, ACORN $\gamma$-Augmentierung, Partial-Rebuild `TraversalTracker`, `GhostFreeVectorIndex`), `contextra-text` (BM25/BM25F, `GermanMorphTokenizer`/`GermanCompoundSplitter`, Block-Max WAND), `contextra-graph` (CSR, n-äre Hyperkanten, PPR/APPRH/TL-HFD/`PathRAG`, Community/Percolation, `SessionBranchTree`), `contextra-rank` (4-/5-Signal RRF/mRRF/Resonance, PAVA/Platt/Conformal, Drift, `DiBud`, `explain`), `contextra-engine` (`TenantPolicy`, `Collection`, `HybridQueryBuilder`, `AdaptiveDecayController`), `contextra-ingest`/`privacy`/`prompt-guard`/`sandbox`/`router`/`kvcache` | `contextra-store` (Alt-LSM), `DbTransaction`/`CommitLedger` 4-Wege-2PC, `contextra-durable-fs` als `RealFs`, unversiegeltes DiskANN-`memmap2` mit Literal-Key, `blake3`/`aes-gcm-siv`/`bincode` in Persistenz, doppeltes MVCC in Produktionspfaden |

---

## 2. Leitentscheidungen, Ziele, Nicht-Ziele, Bedrohungsmodell

### 2.1 Leitentscheidungen

| # | Entscheidung |
|---|---|
| L1 | **Cluster** (`ClusterId`, 16 Byte, vom Kern zufällig erzeugt) = Löschdomäne = Schlüsseldomäne = Versionsdomäne = Platzierungseinheit. |
| L2 | Ordnung geclusterter Spaces: `(cluster, branch, user_key, tag↓)`. Blöcke clusterrein, je Datei ein zusammenhängender Abschnitt je Cluster mit **abschnittsrelativen** Offsets. |
| L3 | Cluster sind `Sealed` (eigener Schlüssel) oder `Plain`. **Alles** eines Sealed-Clusters ist unter seinem Schlüssel versiegelt (`INV-SEAL-1`). |
| L4 | Dateien sind Pure, Packed oder Staging (nur L0). |
| L5 | **Eine Sequenzierungsstelle** (Sequencer, Rang 1, ohne E/A) vergibt alle Stamps und linearisiert Gate, Control, Link und Branch-Ops; **ein Syncer** schreibt und synchronisiert das Redo in Stamp-Reihenfolge (§6.1). Dateien unveränderlich, `Version` per `Arc`. |
| L6 | **Catalog = einzige Wahrheit über Dateien.** Lebendig ⇔ durabler Edit nennt die Datei. |
| L7 | **Reihenfolge ist Vertrag:** Gate < Schlüsselvernichtung < Dateiwechsel < Unlink. Nur Roll-forward. |
| L8 | Cluster-Links (`DerivedFrom`) kaskadieren die Löschung atomar im selben Gate-Frame. |
| L9 | `Restriction` verhindert Löschung; kein Override im Kern. |
| L10 | Stamps werden nie doppelt vergeben (auch nicht nach Absturz). |
| L11 | Kopf ohne Körper-E/A lesbar; Gewichte/Aggregate sind Teil des cluster-sealed Index. Page entschlüsselt kleine Metadatenblöcke, aber keine Körper. |
| L12 | Fünf Erweiterungsverträge (§12). |
| L13 | Instanzsperre, `Perm::Private`, Rollback-Anker. |
| L14 | **Eine Siegelform** für alles (`XChaCha20-Poly1305` mit 24B Nonce), unabhängig von Datei und Position. |
| L15 | **Ein Purge-Algorithmus** für Pure, Packed, Staging, Packgruppen-Abwurf, Restriction-Ausnahme. |
| L16 | Redo-Barriere für `Plain`-Löschung beim Purge, nicht beim Gate. |
| L17 | **Branches sind clusterlokal und löschen mit dem Cluster.** Keine instanzweiten Commit-DAGs. |
| L18 | TLA+-Modelle und deterministische Simulation sind Gate-Bestandteil. |
| L19 | Der Kern ruft kein Modell auf und kennt keine Politik. |
| L20 | **Open-Core-Grenze:** Formate, Kernsemantik, Ports, Test-/Simulationsverträge und Referenzverhalten bleiben offen und vollständig genug, um ein interoperables System ohne proprietäre Erweiterung zu betreiben. Monetarisierbare Produkte implementieren dieselben offenen Ports und dürfen keine Kerninvariante abschwächen oder für Basis-Durability/Löschung erforderliche Funktion hinter eine Bezahlschranke legen. |
| **L21** | **Ein-Wahrheits-Commit statt 4-Wege-2PC:** Kanonische RAG-Daten (Dokumente, Vektordaten, Text-Vorwärtsdaten, Graph-Fakten einer Löschdomäne) committen atomar in **einem** Storage-`Change`; abgeleitete Indizes projizieren idempotent aus dem `ConsumerFeed` bis `durable_stamp` (`Index ∪ Delta` + `filter_visible`). |
| **L22** | **Löschkonforme Index-Klassen:** Abgeleitete Vektor-, Volltext- und Graph-Strukturen mit Personenbezug existieren ausschließlich als `ClusterSections`/`MixedPerClusterKey` oder als mandantenpartitionierte `Rebuildable`-`SegmentOwner` mit synchronem `on_gate`-Filter, dokumentierter Rewrite-Grenze und `verify_absent`-Attest; globale unversiegelte Klartext-Indizes oder `memmap2`-Bypässe sind unzulässig. |
| **L23** | **Eine Versions-/Zeitachse & ein Budget:** Alle Schichten (Storage, Vector, Text, Graph, Engine, Checkpoint) synchronisieren Sichten auf `(instance_id, Stamp)` (`Epoch(u64)`) und reservieren Arbeitsspeicher sowie Rewrite-Scratch aus dem gemeinsamen `MemoryBudget`/Governor. |
| **L24** | **Strikte Konzepttrennung oberer Mechanismen:** `DerivedFrom` (DSGVO-Cluster-Kaskade) ≠ `CsrGraph`/`HyperEdge` (Wissensgraph); `Eraser::cluster` ≠ `AdaptiveDecayController` (kognitive Relevanzabwertung); DB-`Checkpoint`/`Restore` ≠ Agenten-Workflow-`CheckpointGuard`; `SessionBranchTree` ↔ Storage-Branches (`SYS_BRANCH`). |

### 2.2 Ziele

- **Z-1** Atomarität über Spaces und Branches (inkl. Multi-Signal-Rohdaten eines Ingests)
- **Z-2** Durability (sichtbar nur nach Fsync, Ausnahme explizites `Relaxed`)
- **Z-3** Snapshot-Isolation ohne Lesesperren über Storage und alle abgeleiteten Indizes (`Index ∪ Delta`)
- **Z-4** Gate/KeyDestroy unabhängig von Storegröße; physischer Purge bounded durch betroffene Datei-/Extension-Partitionen
- **Z-5** Ablauf einer Packgruppe als O(Dateien)
- **Z-6** Nachweisbarkeit (Receipt, `verify_receipt`, Orakel, Ledger, Anker, signierte Compliance-Belege in `contextra-erasure`)
- **Z-7** Kopf ohne Körper-I/O, Budgetseiten (`page()` + `DiBud`) ohne Body-Block-Lesen
- **Z-8** Race-Freiheit durch Konstruktion, belegt durch Modell, Simulation, Loom
- **Z-9** Begrenzte Ressourcen über ein gemeinsames Gesamtsystem-`MemoryBudget`
- **Z-10** Stabile Verträge zwischen Speicherkern, Retrieval-Triebwerken und Engine
- **Z-11** Flüchtige Arbeitsbereiche (`Volatile`-Spaces)
- **Z-12** Idempotenz für Agenten-Retries (`IdemToken` + `SYS_IDEM`)
- **Z-13** Gate/KeyDestroy bei vollem Datenträger; physischer Purge nutzt vorgehaltene Scratch-Reserve
- **Z-14** Überwachbare Löschfristen mit Eskalations-Lanes
- **Z-15** O(1)-Fork, indexierter Diff und konfliktgeprüfter Merge je Cluster (inkl. Agenten-Session-DAG-Branches)
- **Z-16 (Datenklassifikation):** `DataClass` ist ein geschlossenes Enum; ungültige Kombinationen (`Personal` ohne Cluster/Encrypted, bei Durable ohne Cluster-Seal) werden vor dem ersten I/O abgewiesen
- **Z-17 (Observability):** Gate, KeyDestroy, Purge, Retrieval-Latenzen und Index-Lag haben getrennte Metriken; feste Label-Allowlist ohne IDs, Queries oder Inhalte
- **Z-18 (Entwicklerdiagnose):** Jeder öffentliche Fehler hat stabilen Typ und sichere `hint()`-Handlung (Fassade ergänzt typisierte `suggested_call`)
- **Z-19 (Tail-Latenz):** Unter Rang 1 keine E/A, kein Provider-, Graph- oder Plattenzugriff; kritischer Abschnitt O(Ops) im RAM (`H-24`)
- **Z-20 (Metadaten-Skalierung):** RAM, Dateien und Dateideskriptoren wachsen kontrolliert mit der Clusterzahl (`Admission` vor Allokation)
- **Z-21 (Deutsche Unternehmenssuche & Fusionsgüte):** Vollwertige Unterstützung deutscher Morphologie/Komposita-Zerlegung, BM25/BM25F + WAND, HNSW/DiskANN + SQ8/RaBitQ + ACORN sowie kalibrierter 4-/5-Signal-Fusion (`Isotonic`, `Platt`, `Conformal`, `DriftDetector`) bei voller Löschgarantie
- **Z-22 (Sichere Hardware-Ausnutzung):** Runtime-SIMD-Dispatch (`AVX-512 > AVX2 > NEON > Scalar`) isoliert in `contextra-simd` mit bitidentischen Prüfungen gegenüber Skalar-Referenz und NaN-Ablehnung vor Eintritt
- **Z-23 (PII- & Prompt-Sicherheit vor Egress):** Automatisierte PII-Maskierung/-Wiederherstellung (`contextra-privacy`), Prompt-Injection-Schutz (`contextra-prompt-guard`), und abgeschottetes Parsing untrusted Formate (`contextra-sandbox` + `contextra-ingest`)
- **Z-24 (Saubere Migration):** Verifizierte, datenverlustfreie Ablösung von `contextra-store`, `contextra-mvcc`, `contextra-crypto` (Erasure-Teile) und `bincode`/`memmap2` ohne Dual-Write-Split-Brain

### 2.3 Nicht-Ziele

NZ-1 Verteilung/Konsens in v1 · NZ-2 eigene Kryptografie · NZ-3 Schemas, Sekundärindizes, SSI im Speicherkern · NZ-4 benutzerdefinierte Merge-Operatoren im Speicherkern (Tag `6` reserviert) · NZ-5 Schutz von Prozessspeicher/Swap/Core-Dateien · NZ-6 physische Umordnung über Clustergrenzen · NZ-7 Altformat-Kompatibilität zu `contextra-store`-SSTables (Migration via Export/Forward-Ingest) · NZ-8 implizite Windows-Durability ohne bestandene Fs-Konformanz · NZ-9 `mmap`/`O_DIRECT`/`io_uring` im Speicherkern oder unversiegelt in Indizes · NZ-10 Kompression im Speicherkern in v1 (`COMPRESSION=0`) · NZ-11 Inhaltsadressierung/Dedup über Löschdomänen · NZ-12 Embeddings, ANN, BM25, CSR-Graphsuche innerhalb von `contextra-storage` selbst (leben als eigene Crates auf den Verträgen von §12) · NZ-13 clusterübergreifende Commit-DAGs / Merkle-Zustandshashes im Kern · NZ-14 gesetzliche Bewertung / Signatur im Speicherkern (Signatur in `contextra-erasure`) · NZ-15 Schlüsselverwaltung im Kern (`KeyProvider`) · NZ-16 Mandanten-/Rechteprüfung im Kern (`contextra-engine` / Auth-Schicht oben erzwingen `TenantPolicy` vor Kern-/Indexzugriff) · NZ-17 Async-Runtime im Speicherkern · NZ-18 KV-Cache-/Prefix-Store im Speicherkern (`contextra-kvcache` als eigenes Subsystem) · NZ-19 zwei lebende Klone einer Instanz · NZ-20 Multi-Node-Replikation/Failover bis zu einem eigenen Verteilungs-Gate.

**Verteilbarkeit vorbereiten, nicht vorwegnehmen:** IDs und Stamps sind instanzlokal; jedes externe Ereignis, Watermark (`ConsumerFeed`, Index-Generation) und `ErasureReceipt` muss `(instance_id, stamp)` führen und darf Stamps nie global sortieren. Restore erhält dieselbe `InstanceId` und darf schreibend erst nach extern belegtem Fencing des bisherigen Besitzers starten (`RestoreQuarantine`/ReadOnly).

### 2.4 Bedrohungsmodell

| ID | Angreifer/Fehler | Gegenmaßnahme im Gesamtsystem |
|---|---|---|
| AT-1 | Prozess mit Lesezugriff auf das Daten-/Indexverzeichnis, ohne Schlüssel | AEAD-Siegel auf Basis/Blob/Redo/Index-Blöcken (§5.2, §12.4) |
| AT-2 | Besitzer des Datenträgers, ohne Schlüssel | Siegel + Schlüssel ausschließlich extern über `KeyProvider`/`KMS` (§9.4) |
| AT-3 | Dateimanipulation (Einfügen, Abschneiden, Vertauschen, DiskANN-Manipulation) | Ketten-MAC, AEAD-AAD mit `op_uid`/`block_uid`/`record_uid`, Katalog-Kette, Schlüssel-gebundener Segment-MAC (Ablösung harter statischer Keys!) |
| AT-4 | Fehlerhafte/stale Index-Schicht oder bösartiger Index-Treffer | nur versiegelte `SegmentIo`-/`ExtStore`-Wege; Pflichtprüfung `View::filter_visible` vor Ausgabe; `ErasureOracle` (`O-Segments`, `O-Files`) |
| AT-5 | **Rollback** auf frühere Backups/Checkpoints, um Gelöschtes zurückzubringen | Ledger (`tomb_id`) + monotoner externer `Anchor` + `RestoreReconcile` (`OP8`, §10.3) + `BackupHorizon` (§10.4) |
| AT-6 | Betreiber mit Löschwunsch gegen aktive `Restriction` | Prüfung im Sequencer (`policy_epoch`) **und** Purge (`files_remove`/`cluster_state`); `lift` nur nach `LiftPolicy` |
| AT-7 | Seitenkanal über globale Index-Metadaten (seltene deutsche Komposita, Vorfilterungs-Latenzen) | Mandanten-Partitionierung vor ANN/BM25/Graph; Refcount-Löschung verwaister Terme; keine Klartext-Nutzdaten in Metriken |
| AT-8 | Prompt-Injection / PII-Abfluss an externe Cloud-LLMs | `contextra-prompt-guard`, `Trust-Label` im Kopf, `contextra-privacy` (Maskierung/Entmaskierung + Audit-Chain) vor Model-Egress |
| AT-9 | Bösartige Dokument-Uploads (PDF, ZIP, XLSX, HTML) | Ausführung von `contextra-ingest` innerhalb `contextra-sandbox` (Ressourcen-/Tiefenlimits + Parser-Fuzzing) |
| F-1…F-9 | Torn Write, Fsync-EIO, ENOSPC, Bit-Rot, Schlüsselverlust, Crash im Löschprotokoll, Retry-Stürme, Index-Lag | §3.5, §10, §12.4.0 |

**System-Annahmen (`AN-01`…`AN-10`):** `AN-01` `sync_*` macht durabel · `AN-02` Schreibvorgänge können zerreißen · `AN-03` Fsync-Fehler nicht wiederholbar (Handle vergiftet) · `AN-04` `rename` im selben Verzeichnis atomar · `AN-05` Prozessspeicher vertrauenswürdig, OS/Swap-Risiko wird deklariert · `AN-06` `destroy_many` macht Schlüssel bei `Absent` unbenutzbar · `AN-07` starker CSPRNG (`Entropy`) · `AN-08` `Anchor` außerhalb Datenverzeichnis · `AN-09` Stamps nie aus Wanduhr; TTL/Deadline nutzen monotone Laufzeit + UTC + durablen `clock_floor` (`max_clock_skew` ⇒ `ClockUncertain` fail-closed) · `AN-10` Append-Dateien schreiben nur in vorallokierte Bereiche.

### 2.5 Produktverträge (vom sichtbaren Ergebnis rückwärts spezifiziert)

| Anwender/Agent sieht | Gesamtsystem-Vertrag (Engine + Retrieval + Storage) | Freigabebeweis |
|---|---|---|
| Erfolgreiches `Durable`-Write überlebt Crash; Timeout ist retrybar | Ein atomarer Multi-Space-`Change` (Rohdaten, Metadaten, Codes); Receipt trägt `(instance_id, stamp)`; `IdemToken`/`SYS_IDEM` verhindert Doppelwirkung; kein 4-Wege-2PC | FaultFs-Crash-Sweep über Write-/Sync-Grenzen, Idempotenzmodell |
| Hybrid-Suche liefert nur sichtbare, mandantenkonforme Treffer innerhalb des Tokenbudgets | `view_durable()`, `Index(applied_stamp) ∪ Delta(applied_stamp, V]`, Mandanten-/Gate-Vorfilterung vor ANN/WAND/Graph, `View::filter_visible`, kalibrierte Fusion (`rank`), Budget-`page()` + `DiBud` | Differential Retrieval (`Index ∪ Delta ≡ Vollscan`); Recall@Budget; Race `Gate × Query`; `IndexLag` explizit |
| Löschauftrag wirkt sofort in allen Suchkanälen und ist belegbar entfernt | Durables Ledger-Intent → Gate (`ER1` + synchrones `on_gate` für Vector/Text/Graph) → Anchor → KeyDestroy (`ER2`, `Absent`) → Purge (`ER5` Rohkopie + Index-`rewrite_without`) → `verify_absent` + `ErasureReceipt` (+ Ed25519 in `contextra-erasure`) | Crash/Restore-Reconcile vor/nach jedem Schritt; `ErasureOracle` (`O-Files`, `O-Segments`, `O-Gate`, `O-Ledger`); `verify_receipt` |
| Agenten-Branching (`SessionBranchTree`) isoliert Experimente ohne Datenleaks nach Löschung | Clusterlokale Storage-Branches (`SYS_BRANCH`, `fork`, TTL, `merge`); Punkt-/Range-Guards; mit Cluster-Löschung sterben alle Branches | Referenzmodell, Race-Simulation (`RACE-15..17`), Branch-Orakel |
| Lastspitze oder Index-Rebuild verursacht kontrollierten Backpressure statt OOM/Disk-Full | Gemeinsames `MemoryBudget`/Governor (Storage + HNSW + DiskANN + BM25 + CSR + Rerank); Admission vor Allokation; `Backpressure(Retry)` | Combined RAG-Soak mit RSS-Spitzen, Scratch, Lag und p99 |
| Restore stellt exakt konsistenten Daten- & Index-Stand ohne gelöschte Daten her | `Ledger` + `Anchor` führen Restore in `RestoreQuarantine`; nach `RestoreReconciled` rekonstruieren Indizes ab `(replay_floor / durable_stamp)` ohne Gelöschtes | Restore-Matrix mit alten Backups, Intent-Präfixen, fehlendem KMS, altem Anker und Stale-Index-Rebuild |

### 2.6 Open-Core-Produktgrenze

Open-Core ist ein Packaging- und Betriebsmodell, keine technische Einschränkung der Datensicherheit oder Löschbarkeit. **Offen bleiben:** alle Speicher-, Index- und Receipt-Formate, Speicherkern, Ports, Retrieval-Triebwerke (`vector`, `text`, `graph`, `rank`), Engine, Referenz-Provider/-Anker (`crypto-ref`, `storage-fs`, `keys-file`, `FileAnchor`), Konformanzvektoren, Simulation, Orakel, Basis-Branching/-Erase sowie Export-/Restore-Werkzeuge.

Kommerzielle Editionen können optionale Adapter/Services bereitstellen (z. B. HSM-/Cloud-KMS-Provider, `Strong`-Anker wie TPM/Transparency-Log/Object-Lock, Multi-Node-Replikation nach eigenem Verteilungs-Gate, zentrale Mandanten-Orchestrierung, Audit-/AVV-/Compliance-Portale, spezielle Inference-/Kalt-Tier-Integrationen). Die zugrundeliegenden Schnittstellen, `ErasureReceipt`-Verifikation (`verify_receipt`) und Exportpfade bleiben offen; `LicenseGate`/`FeatureRing` (§18) dürfen keine Kerninvariante (`INV-*`) abschwächen und keine Basislöschung hinter einer Bezahlschranke verweigern.

---

## 3. Gesamtsystem-Architektur, Crate-Topologie & Abhängigkeits-Governance

### 3.1 Ring-/Ebenenmodell & Crate-DAG

Abhängigkeiten verlaufen **strikte gerichtet azyklisch nach unten** (`Level 3 → Level 2 → Level 1 → Level 0`). Jede Verletzung bricht das CI-Gate `layering` und `cargo tree`.

```text
Level 3 (Apps & Verifikation):
  contextra-mcp (MCP-Server)  |  contextra-py (PyO3)  |  contextra-testkit  |  contextra-bench
        │
        ▼
Level 2 (Fassade, Engine, Agentik & Speicher-Adapter):
  contextra (Produkt-Fassade) ──► contextra-db (DB-Fassade) ──► contextra-engine (Orchestrator)
  contextra-agent / contextra-cognition / contextra-router / contextra-erasure* (Compliance/Signatur)
  contextra-types & contextra-ports (System-Verträge, übergangsweise Ring-2-Brücke)
  Speicher-Adapter (Composition Root): contextra-storage-bridge | contextra-storage-fs |
                                       contextra-crypto-ref | contextra-keys-file | contextra-keys-cohort
        │
        ▼
Level 1 (Speicherkern & Domänen-Triebwerke):
  ┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
  │ Retrieval & Persistenz-Triebwerke:                                                              │
  │   contextra-vector  |  contextra-text  |  contextra-graph  |  contextra-rank  |  contextra-checkpoint
  │ LLM-Runtime-Bausteine: contextra-kvcache | contextra-infer-ollama | contextra-infer-candle    │
  │ Übergangs-Altspeicher / MVCC / Krypto (Ablösung bis G-CUTOVER, §14.1):                          │
  │   contextra-store (legacy)  |  contextra-mvcc (legacy)  |  contextra-crypto (legacy-erasure)    │
  └───────────────────────────────┬───────────────────────────────────┬─────────────────────────────┘
                                  │ (Ports & Verträge §12)            │
                                  ▼                                   ▼
  contextra-storage  (#![forbid(unsafe_code)], synchroner Kern) ──► contextra-storage-ports (Traits/Werttypen)
        │
        ▼
Level 0 (Fundament, Unsafe-Inseln & Sicherheit):
  Unsafe-Inseln (einzige erlaubte Crates mit `unsafe`):
    contextra-simd (AVX-512/AVX2/NEON/Scalar) | contextra-sys (OS-Primitives) | contextra-wire (IPC/Framing)
  Safe Foundation & Safety:
    contextra-ids | contextra-env | contextra-adapt | contextra-ingest | contextra-privacy
    contextra-sandbox | contextra-prompt-guard | contextra-infer-onnx
    (contextra-durable-fs: experimental; nicht für storage-fs RealFs genutzt, SD-70)
```

**Modulschnitt innerhalb von `contextra-storage`** (Abhängigkeiten nur nach unten, Test `layering`):

```text
lib.rs (Fassade) · engine (committer/sequencer+syncer, memtable, version, views, branches, registry,
                           extension, feed, segments, page, cursor, cache, idem, guard_window, graveyard, volatile)
maint (flush, compaction, blob_gc, purge, scheduler, scrub) · erase (gate, cluster_table, links, protocol, ledger)
recovery · checkpoint (create, restore) · catalog · store (run, blob, redo) · keys · codec (varint, tags, checksums, comparator, seal, format, header)
env (Fs, Clock, Entropy, Governor, Metrics, Anchor; MemFs, FaultFs, Sim) · sync (Ränge 1–8) · testing (ErasureOracle, cfg(feature="testing"))
```

**Ist-vs-Soll-Abgleich Workspace (`0dd5cfc41`):**
Die aktive Wurzel-`Cargo.toml` führt aktuell 12 Workspace-Crates (`checkpoint`, `crypto`, `durable-fs`, `engine`, `graph`, `mvcc`, `rank`, `simd`, `storage`, `sys`, `text`, `vector`), während `capabilities.toml` 30+ Crates beschreibt. `contextra-storage-ports`, `contextra-storage-fs`, `contextra-crypto-ref`, `contextra-keys-file`, `contextra-keys-cohort` und `contextra-storage-bridge` fehlen derzeit physisch im Verzeichnisbaum (`crates/contextra-storage/crates/*`), obwohl `crates/contextra-storage/src/*.rs` bereits `contextra_storage_ports` importiert. Gleichzeitig enthalten `tests/layering.rs` und `tests/unsafe_islands.rs` laut `CLAUDE.md` unverdrahtete Root-Prüfungen. Die Bereinigung ist Pflichtbestandteil von Phase **P0** (§14.1, §18.1).

`std::fs`, `std::time::SystemTime` und `rand`/`getrandom` sind im Speicherkern sowie in reinen Ring-0/1-Algorithmuskernen über `clippy.toml`/`disallowed_methods` verboten und ausschließlich über injizierbare Ports (`Fs`, `Clock`, `Entropy`, `Rng`) bzw. Adapter erlaubt.

### 3.2 Ports (Speicher-Ports & System-Ports)

**A) Speicher-Ports (`contextra-storage-ports`):**

| Port | Aufgabe & Vertrag |
|---|---|
| `Fs` | Dateien, `read_at`, geordnetes `read_ranges`, `append_prealloc`, `preallocate`, `sync_data/all/dir`, `rename`, `lock`, `list`, `free_space`, optionale `link/reflink/punch_hole`; kein `mmap` im Kernpfad |
| `Clock`, `Entropy` | Injizierbar; Uhr **nie** für Stamps. Liefert monotone Laufzeit, UTC, durablen High-Water-Mark (`clock_floor`) und Skew-Status; Fristablauf fail-closed bei unplausiblen Sprüngen (`ClockUncertain`, §3.5) |
| `Anchor` | `read()`, `advance(expect, next)` (CAS, monoton, durabel), `strength() → Strong\|Weak`; Wert `(erase_epoch, ledger_head, catalog_epoch)` |
| `KeyProvider` | §9.4 (`kek`, Batch-`create_cluster_keys`/`cluster_keys`/`destroy_many`, `probe`, `list`, Tomb-Epochen `create/tomb_key/retire_tomb_epoch`) |
| `Cipher` / `Mac` / `Kdf` / `Checksum` | AEAD (`XChaCha20-Poly1305`, 24-Byte-Zufallsnonce), Ketten-MAC (`HMAC-SHA256`, 16B/32B), `HKDF-SHA256`, `CRC32C` |
| `Authority` (optional) | Prüft Signaturen für `LiftPolicy::FourEyes` |
| `Metrics`, `Observer` | Nicht blockierend, **nie** unter Sperren; geschlossener Low-Cardinality-Metrikkatalog (§16) |
| `MemoryBudget` | Gemeinsamer RAM-/Scratch-Admission-Port für Storage **und** Extensions (`try_reserve(class, bytes) -> ResourcePermit`); RAII-Drop gibt exakt frei |
| `HostProbe` (optional) | `probe() -> HostReport { swap, core_dumps, memlock_limit, fs_kind, device_ids }`; reiner Adapter-Befund für Diagnose/Restexposition (§9.9); Werte `Unknown` gelten als `Declared` |
| `ColdTier` (optional) | Auslagerung kalter unveränderlicher Dateien unter Erhalt der Lösch-/Purge-Garantien |

**B) Erweiterungs-, Retrieval- & System-Ports (§12 & `contextra-ports`):**

| Port / Vertrag | Rolle im Gesamtsystem |
|---|---|
| `SegmentOwner` | Verbindlicher Lösch-, Gate-, Watermark-, Rewrite- und `verify_absent`-Vertrag für alle persistierten Indizes (`Vector`, `Text`, `Graph`, `IdentityDirectory`, `KvCache`) (§12.1) |
| `ExtensionBlock` & `BlockPruner` | Block-Granulare opake Indizes/Zeit-Aggregate am Run mit `PruneContract::ShadowSafe` auf `main` (§12.2) |
| `Consumer` / `ConsumerFeed` | Durabler Redo-Stream (`(instance_id, stamp) ≤ durable_stamp`) mit expliziter `Redacted`/`Erased`-Maskierung (§12.3) |
| `StorageBridge` (`contextra-storage-bridge`) | Asynchrone `spawn_blocking`-Fassade + typisierte Projektion zwischen `contextra-storage` (`Change`/`StorageView`/`ErasureReceipt`) und den Engines (`contextra-engine`, `contextra-erasure`) |
| `VectorIndex` / `Contract::Index` | `HnswIndex`, `DiskAnnIndex` (gebunden an `Epoch(u64) ≡ Stamp` und `BlockTransform`, §12.4.1) |
| `TextIndex` / `BM25` | `InvertedIndex`, `BM25`/`BM25F`, `GermanMorphTokenizer`, Block-Max WAND (gebunden an `Epoch(u64) ≡ Stamp`, §12.4.2) |
| `GraphIndex` / `GraphProvider` | `CsrGraph`, `HyperEdge`, PPR/APPRH/TL-HFD/`PathRAG` mit Quell-Provenienz (`source_cluster`, `source_doc_id`, §12.4.3) |
| `TextEmbeddingEngine` / `Reranker` | Lokale/remote Embedding- und Cross-Encoder-Modelle (`onnx`, `candle`, `ollama`) hinter `Engine`- und `Router`-Orchestrierung |
| `LicenseGate` | Schnittstelle für Edition/Feature-Checks (`OpenFastGate` u. a.), streng an Open-Core-Grenze (§2.6) gebunden |

### 3.3 Mechanismus, nie Politik (im Speicherkern)

`contextra-storage` entscheidet nie, **ob** gelöscht werden darf, **wer** Zugriff erhält oder **welche** Aufbewahrungsfrist gilt; das verantworten `contextra-engine` (`TenantPolicy`, `RestrictionSpec`), `contextra-privacy` (`DataClass`) und `contextra-erasure` (Compliance-Orchestrierung & Signatur). Der Speicherkern setzt Aufträge atomar, geordnet und nachweisbar um und verweigert nur bei technischem Hindernis oder aktiver `Restriction`.

Der Kern bildet **keine persistierten Inhalts-Hashes über Klartext (`INV-PRIV-1`)**. Der Derivation Cache der oberen Schicht (§12.6, K-32) ist nur als clusterlokaler Keyed-MAC unter einem zufälligen, im selben `Sealed`-Cluster gespeicherten `cache_secret` zulässig.

### 3.4 Threading, Async-Grenze & Compute-Pools

```text
Aufrufer / Async-Welt (tokio in mcp / engine / infer)
  │  (spawn_blocking via contextra-storage-bridge)
  ▼
Synchroner Speicherkern (contextra-storage, ohne Async-Runtime):
  ├─ Schreibpfad:  Aufruferthread (Stufe-A-Guards, Hülle, Krypto, Vorkodierung)
  │                    ──► Sequencer (Rang 1, rein RAM, Flat-Combining-Leader)
  │                    ──► Syncer (1 Thread, kein Lock, Redo-write + sync_data → Publish)
  ├─ Lesepfad:     Lock-frei über Arc<Version> + Atomics (Blatt-Locks max Rang 6)
  ├─ Wartung:      Lanes urgent > flush > compact > blobgc > idle
  └─ Ledger/Anker: Je genau 1 dedizierter Schreiber über Kanal (urgent, gebündelt)

Retrieval & CPU-Compute (contextra-vector / rank / text):
  └─ Eigener ComputePool für HNSW-Distanz-/Quantisierungs-/SVD-Arbeit;
     KEINE Speicherkern-Sperre darf jemals in CPU- oder .await-Wartezeiten gehalten werden.
```

Provider-, Cipher-, KDF-, Graph- und Guard-Arbeit findet ausschließlich **vor** Rang 1 auf Aufruferthreads oder auf Wartungs-Lanes statt, niemals im Sequencer. Async-Runtimes (`tokio`) existieren nur an den oberen Systemrändern (`contextra-engine`, `contextra-mcp`, `contextra-py`, Inferenz-Adapter).

### 3.5 Fehlermodell

| Klasse | Beispiele | Verhalten |
|---|---|---|
| F-1 transient | `EINTR` | Wiederholung mit Backoff |
| F-2 Ressource | `ENOSPC` beim Vorallokieren, `DiskFull`, `Limit{ClusterMetaBudget\|ClusterLocalAdmission}` | **kein Poison**; User-Commits abgewiesen, Löschung/Wartung laufen mit Reserve weiter (§6.6) |
| F-3 E/A-hart | `EIO`, Fsync-Fehler | **nie auf demselben Handle wiederholen**; Handle wird `Poisoned`; Löschaufträge bleiben über das Ledger aufnehmbar (§9.5.2). Gestartete Changes liefern `Indeterminate{first_stamp, last_stamp}`; Retry nur identisch mit `IdemToken` |
| F-4 Integrität | CRC/MAC/AEAD-Fehler | kleinste Einheit vergiften (Block → Abschnitt → Datei → Index-Segment), `Corrupt{..}`, gesunde Cluster laufen weiter |
| F-5 Format | `Unsupported` | Öffnen / Dekodieren strikt verweigert |
| F-6 Schlüssel | Provider offline / Schlüssel fehlt | `KeyUnavailable{retry_after}` bei Providerausfall; **`KeyLost`** bei fehlendem Schlüssel ohne Löschnachweis (Quarantäne, Alarm). Nach Gate liefern Sichten `Erased` (§7); `Shredded` ist Clusterzustand, nie Lesefehler |
| F-7 Aufrufer / Index | `Conflict`, `Restricted`, `UnknownCluster`, `BelowFloor`, `ReservedSpace`, `SnapshotTooOld` (View-Lease), `ErasePending`, `LedgerUnavailable`, `LedgerSameDevice`, `BackupBeyondHorizon`, `ProviderAssuranceInsufficient`, `IndexLag`, `IdemConflict`, `InvalidInput` (NaN/Inf/Dim) | typisiert mit stabilem `hint()` (`suggested_call` in Fassade) |
| F-8 Logik | Invariante verletzt | Test: Abbruch; Produktion: Poison + Observer; **kein `panic`** |
| F-9 Rollback | Anker höher als Catalog/Ledger | schreibend verweigert (`Err(Rollback)`), nur `ReadOnly` mit Diagnose |

Fehlertexte enthalten grundsätzlich **keine** Schlüssel, Nutzerschlüssel, Dokumentinhalte, Queries oder Embeddings.

### 3.6 Lint-Pflicht & Unsafe-Insel-Governance

Arbeitsbereichsweite Lints (`[workspace.lints]`): `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `unreachable`, `indexing_slicing`, `await_holding_lock`, `await_holding_refcell_ref`, `exit`, `dbg_macro` auf `"deny"`; `let _ =` auf `sync_*`/`rename`/`unlink` verboten. `Debug` für `ClusterId`, Schlüssel, Nutzdaten und Query-Texte ist redigiert (`Redacted<T>`).

`#![forbid(unsafe_code)]` gilt **ausnahmslos** in `contextra-storage`, `contextra-storage-ports`, `contextra-storage-fs`, `contextra-crypto-ref`, `contextra-keys-*`, `contextra-engine`, `contextra-vector`, `contextra-text`, `contextra-graph`, `contextra-rank`, `contextra-checkpoint`, `contextra-mvcc`, `contextra-crypto`. Die einzigen zulässigen Unsafe-Inseln sind **`contextra-simd`**, **`contextra-sys`** und **`contextra-wire`** (`unsafe_op_in_unsafe_fn = "deny"`, vollständige `SAFETY`-Kommentare).

### 3.7 Ökosystem: Finale Abhängigkeiten (Crates) und Bereinigungsplan

Normativ für alle `Cargo.toml`-Dateien im Workspace. Auswahlkriterien in Rangfolge: (1) `std` genügt ⇒ kein Fremdcrate; (2) `#![forbid(unsafe_code)]`-Kompatibilität (Fremd-`unsafe` nur in auditierten RustCrypto-/Crossbeam-Crates bzw. innerhalb der drei Unsafe-Inseln); (3) kein Async im Speicherkern und in Ring-0/1-Algorithmuskernen; (4) ein Crate je Aufgabe, **eine** Hauptversion je Crate im gesamten Workspace (`cargo tree -d`); (5) keine Abhängigkeit, die unkeyed Inhalts-Hashes über Klartext in persistierte Formate schreibt (`INV-PRIV-1`).

#### 3.7.1 Speicherkern `contextra-storage` und `contextra-storage-ports`

| Aufgabe | Entscheidung | Begründung |
|---|---|---|
| Fehler | `thiserror ^2` | einheitliche Workspace-Version (vorher 1 vs 2 gemischt); redigierte Meldungen |
| Byte-Puffer | `bytes ^1` | Zero-Copy-Slices für Körper/Blöcke |
| Speicherhygiene | `zeroize ^1` (`derive`) | Schlüsselmaterial, Klartextpuffer, `Redacted<T>` |
| Memtable-Index | `crossbeam-skiplist ^0.1`, **nur hinter Trait `MemIndex`** | lock-freie sortierte Map ohne eigenes `unsafe`; Fallback: sharded `BTreeMap` unter Rang-Sperre für `loom`-Modelltests; Skiplist selbst unter `shuttle`/`miri` |
| Guard-Fingerabdruck | `siphasher ^1` | ausschließlich flüchtige Prozess-Fingerabdrücke (`SipHash-1-3`), nie persistiert |
| Sperren & Threads | `std::sync::{Mutex, RwLock, Condvar}` + `RankedMutex` (§11.2) | futex-basiert, Rangstack + Poison-Abbildung auf F-8 (`parking_lot` bringt kein Poisoning und bleibt im Kern verboten) |
| Serialisierung | **keine** (handgeschriebener `codec`, §5) | byte-genaue, begrenzte Formate; `serde`/`bincode`/`flatbuffers` verboten |

**Im Speicherkern explizit verboten:** `tokio`/`async-std`/`futures`, `rayon`, `parking_lot`, `scc`, `dashmap`, `memmap2`, `serde*`, `bincode`, `blake3`, `rand`/`getrandom` (nur Port `Entropy`), `crc32fast`, `anyhow`, `tracing` (Observability ausschließlich über `Metrics`/`Observer`-Ports).

#### 3.7.2 Krypto-Adapter `contextra-crypto-ref` (Standard-Adapter & Composition Root)

Eingebunden durch das Composition Root / `contextra-storage-bridge` (Feature `default-crypto`), nicht vom Speicherkern:

| Aufgabe | Crate | Anmerkung |
|---|---|---|
| AEAD | `chacha20poly1305 ^0.10` (`XChaCha20Poly1305`, 24B Nonce) | Normatives Siegel (§5.2) |
| Ketten-/Auth-MAC | `hmac ^0.12` + `sha2 ^0.10` | HMAC-SHA256, 16B (Auth) / 32B (Kette/Tomb/Scope) |
| KDF | `hkdf ^0.12` | Einmalige Schlüsselrollen-Ableitung außerhalb von `SealFrame` |
| Prüfsumme | **`crc32c ^0.6`** | Zwingend CRC-32**C** (nicht austauschbar mit `crc32fast`) |
| CT-Vergleich & Zufall | `subtle ^2` · `getrandom ^0.3` (nur `Entropy`-Adapter) | Constant-time MAC-Tag-Vergleich; kein `rand` in neuen Crates |

**Bereinigung:** `blake3` wird aus `contextra-crypto-ref` und der `derive_subkey_fast`-Vorstufe restlos entfernt.

#### 3.7.3 Dateisystem-Adapter `contextra-storage-fs` (`RealFs`)

Implementiert `Fs` direkt über `std::fs` + `rustix ^1` (`features = ["fs", "std"]`, für `fallocate`/`punch_hole`/`statvfs`/macOS `F_FULLFSYNC`) sowie `std::fs::File::{lock, try_lock}` (stabil ab Rust 1.89) und sichere Win32-Wrapper aus `contextra-sys` unter Windows. **`contextra-durable-fs` ist als Basis für `RealFs` verworfen (SD-70)**; kein `libc`, kein `memmap2`, keine stille Simulation von `preallocate` (muss über `FsCaps` deklariert werden).

#### 3.7.4 Schlüssel-Provider & Compliance-Schicht

| Crate | Abhängigkeiten & Regeln |
|---|---|
| `contextra-keys-file` | `argon2 ^0.5` (Argon2id Pflicht), `zeroize`, `getrandom` |
| `contextra-keys-cohort` | wie `keys-file`; Cohort-KEK-Rewrap ausschließlich über Ports-AEAD |
| `contextra-keys-kms` (optional) | Anbieter-SDKs (HTTP/TLS) strikt hinter Features, isoliert vom Standard-Kern-Build |
| `contextra-erasure` (Tier `frozen`) | `ed25519-dalek ^2` (`verify_strict`, `SigningKey::from_bytes` aus 32B Seed ohne `rand`), `sha2`, `subtle`, `zeroize`, `serde`/`serde_json` (nur Anzeigeberichte), `thiserror`. Kanonische Receipt-Bytes werden **handcodiert**; **`aes-gcm-siv` entfällt**. `contextra-erase-verify` nutzt ausschließlich Ports + `sha2` |

#### 3.7.5 Retrieval-, Engine-, Safety-Crates & Alt-Abhängigkeits-Bereinigung

Die oberen Crates bleiben funktionsreich, müssen aber ihre Speicher-, Serialisierungs- und Abhängigkeits-Schnittstellen an den Gesamtkern anpassen:

| Crate-Gruppe | Erlaubte Spezifika | Normative Bereinigung bis `G-CUTOVER` (§14.1) |
|---|---|---|
| `contextra-vector` | `thiserror 2`, `ahash`, `roaring`, `subtle`, `zeroize`, `contextra-simd` (über `contract::DistanceFn` / Trait); optional `rayon`/eigener `ComputePool` | **Entfernen aus Persistenz:** `memmap2` (Ablösung durch `SegmentIo` + Block-Cache), direktes `std::fs` in `diskann`/`persistence`, statischer `DISKANN_INTEGRITY_KEY`, `blake3`-Klartext-DocID-Hashing (`hnsw/purge.rs`), `bincode` (Dev). `serde` nur für optionalen In-Memory-Debug-Export, produktive Indexblöcke über `SegmentIo` |
| `contextra-text` | `bytes`, `itoa`, `regex`, `unicode-segmentation`, `thiserror 2` | **Entfernen aus Persistenz:** `bincode`, unkeyed `blake3`, globales `futures::executor::block_on` in `TextPurger` (`purge.rs`), direktes `parking_lot`-Blocking im Hot-Path (Ersatz durch Storage-`ClusterSections` + `SegmentOwner` mit `Epoch`) |
| `contextra-graph` | `bytes`, `ahash`, `arc-swap`, `thiserror 2` | **Prüfen & Bereinigen:** `bincode` in `graph_persist.rs`/`hyperedge.rs`/`session_dag.rs` auf kanonische Codecs bzw. Storage-Segmente umstellen; `blake3` entfernen; `scc` (`RUSTSEC-2026-0205`) durch deterministische Strukturen oder gepatchte Map ablösen; `tokio` (`TRANSITION E29`) vollständig aus Graph-Core entfernen |
| `contextra-rank` | rein synchrones Rechnen (`serde`/`serde_json` nur für Config/Provenance-Reports, `ahash`, `thiserror 2`) | **Bereinigen:** `tokio` aus `TRANSITION E29` entfernen; Kalibrierungs-Persistenz ausschließlich über Engine/Storage (`NonPersonal` System-Space) |
| `contextra-checkpoint` | Workflow-Guards (`CheckpointGuard`, `StateCheckpoint`), `thiserror 2` | **Umbau:** `blake3`-Manifest-Prüfsumme + unbeschränkte `pin_checkpoint`-Aufrufe an `LsmStorage` ersetzen durch clusterlokale Storage-Records + `View-Lease`/`Hold` und Storage-eigene `checkpoint`-Pipeline (§10.3) |
| `contextra-crypto` | Übergangsweise für `kv_cipher`/`KvSegmentManager` & `audit_chain` | `deletion_proof`, `revocation_log`, `kv_shredding` (bereits `deprecated`) nach `contextra-erasure` überführen; `aes-gcm-siv` und `blake3` bis `G-CUTOVER` durch `XChaCha20-Poly1305` / `HMAC-SHA256` ablösen |
| `contextra-engine`, `contextra-db`, `contextra` | `tokio` + `tokio-util` (Async-Orchestrierung an Systemgrenze), `serde`/`serde_json` (API/Metadaten), `contextra-storage-bridge` | `contextra-store` und 2PC (`CommitLedger`) ersetzen; `tokio`-Nutzung streng auf Level ≥ 2 begrenzt (`TRANSITION E29` Frist 2027-04-30 für Level-1-Crates nicht verlängerbar); `parking_lot` über Hauptversion `0.12` vereinheitlichen (außerhalb des Speicherkerns) |
| `contextra-ingest` / `contextra-sandbox` | Parser: `lopdf`, `calamine`, `mail-parser`, `zip =0.6.6`, `cfb`, `html5ever`, `quick-xml`, `csv`, `encoding_rs`, `yaml-rust2`, `toml` | Jeder Parser ist Angriffsfläche für untrusted Dokumente: Pflicht-Fuzz-Ziele, harte Größen-/Rekursions-/Zip-Bomb-Limits, Ausführung isoliert in `contextra-sandbox` (`wasmtime`); Pin-Grund für `zip =0.6.6` dokumentieren & gegen Advisories prüfen |

#### 3.7.6 Test-, Prüf- und Werkzeug-Ökosystem & Durchsetzung

| Zweck | Werkzeug & Gates |
|---|---|
| Interleavings | `loom ^0.7` (`cfg(loom)`) + `shuttle ^0.8` |
| Property-/Fuzzing/UB | `proptest ^1`, `cargo-fuzz` + `libfuzzer-sys` + `arbitrary` (inkl. Ingest-Parser & Format-Decoder), `miri` (`codec`/Siegel/Skiplist) |
| Qualität & Formale | `cargo-mutants`, `cargo-llvm-cov`, `cargo-nextest`, TLA+/TLC (M1–M6), `criterion` (nur `contextra-bench`), `tempfile ^3` |
| Lieferkette & Schichten | **`cargo deny check`** (Positivliste in `deny.toml`; `ignore`-Einträge wie `RUSTSEC-2026-0205` `scc`, `pyo3`, `rustls` nur mit Begründung & Ablaufdatum), `cargo tree -d`, CI-Gate `layering` + `unsafe_islands` (als echte Cargo-Tests eingebunden, §18.1), eine `[workspace.dependencies]`-Version je Crate (`thiserror 2`, `parking_lot 0.12`, `rand` in neuen Crates verboten) |

---

## 4. Datenmodell (Speicherkern `contextra-storage`)

### 4.1 Vokabular

| Begriff | Bedeutung |
|---|---|
| **Space** | benannter, physisch getrennter LSM-Baum (`SpaceId u64`), monoton und dauerhaft reserviert, nie wiederverwendet; `Durable` oder `Volatile` |
| **Cluster** | Löschdomäne / Schlüsseldomäne / Versionsdomäne / Platzierungseinheit (`ClusterId`, 128-Bit); `Sealed` oder `Plain` |
| **Branch** | clusterlokaler Versionszweig (`BranchId u32`, `0 = main`) |
| **Change / Guard / Receipt** | atomare Menge von Ops / optimistische Vorbedingung / Quittung `(instance_id, stamp, durable)` |
| **Stamp** | monotone `u64` (< 2^56) = Transaktionszeit = Commit-Reihenfolge (entspricht `Epoch(u64)` der Retrieval-Verträge) |
| **View / Cursor / Page** | unveränderliche Sicht / Streaming-Iterator / token-/gewicht-budgetierte Seite |
| **Hold** | befristete **technische** Halteregel gegen Verwerfen alter Versionen (`ttl ≤ 24 h`, wird durch Erase überstimmt) |
| **Restriction** | **rechtlicher** Sperrvermerk mit Pflichtgrund; verhindert Löschung ohne Kern-Override |
| **Consumer** | persistenter Fortschrittszeiger (`(instance_id, applied_stamp)`) im Redo-Feed |
| **Segment** | vom Catalog verwaltete Datei einer oberen Schicht (`SegmentOwner`, §12.1) |
| **Packgruppe** | `(tenant_tag, due_bucket, shard)` |
| **tomb_id** | `HMAC(tomb_key(tomb_epoch), "ctxs cluster tomb v1" || instance_id[16] || ClusterId[16])`; instanzgebundenes, linkbares Pseudonym ohne Klartext-ID. Solange die Epoche nicht retired ist, bleibt es linkbar (K-31, §10.4) |
| **tomb_epoch** | `u32` je Tomb-tragendem Ledger-Event (K-31); Zustände `Open → Closed → Retired`; der zugehörige `tomb_key` liegt im `KeyProvider`, nie in `KEYS`, Checkpoints oder Backups (`INV-KEY-5`). Default: genau Epoche `0` |
| **View-Lease** | Frist, nach der eine `View`/ein `Cursor` ihre Version nicht mehr pinnen (`view_lease`, §11.4); danach `SnapshotTooOld` |

Space-IDs `0..=255` sind für den in §5.9 versionierten System-Space-Namensraum reserviert; der persistente Benutzer-ID-Allocator beginnt bei `256`. `reserve_space_ids` schreibt den nächsten High-Water-Mark vor Vergabe eines Blocks durabel; nach Crash werden ungenutzte IDs übersprungen. Exhaustion liefert `Exhausted`, niemals Wrap oder Wiederverwendung. Für `contextra-engine`-Collections entstehen Spaces paarweise bzw. rollenbasiert oberhalb `256` (§12.7).

### 4.2 Reihenfolge, Tag, Marker

`cmp`: Cluster aufsteigend, **Branch aufsteigend**, Nutzerschlüssel bytelexikografisch, Tag absteigend (kein `memcmp` über Konkatenation). Ohne `clustered` ist der Cluster implizit `NONE`.

Tag `u64 = (stamp << 8) | kind`; `Put=1, Delete=2, RangeDelete=3, EraseRange=4, PutRef=5, Merge=6 (reserviert, Decoder: Unsupported)`. `PutRef` nur in Runs. `RangeDelete` ist branch-lokal mit Marker `(cluster, branch, start, end, tag)`, `end` exklusiv. `EraseRange` ist dagegen ein clusterweiter Control-Marker ohne Branch und wirkt auf alle Branches und historischen Views; er ist nie ein gewöhnlicher Merge-Wert. Für Sealed-Cluster bleiben nur Markerart, Cluster-Routing und Stamp im Klartext; `start/end` sind cluster-key-versiegelt. Der Sequencer hält das Gate-Maskenintervall im RAM, und Recovery entschlüsselt es vor Freigabe des Clusters. Bei zerstörtem Clusterschlüssel wird der Cluster insgesamt gesperrt/retired, nicht selektiv rekonstruiert.

Stamps: erster = 1, streng monoton über angenommene Changes und Frames, nie doppelt vergeben (auch nicht nach Absturz), Überlauf ⇒ `Exhausted`. Lücken zulässig.

### 4.3 Cluster

- **Erzeugung:** `Clusters::register(ClusterSpec)` zieht 128 Bit aus `Entropy`, prüft Kollision, legt bei `Sealed` **zuerst** den Schlüssel an (`create_cluster_keys`, Batch, idempotent je `BatchId`, durabel), dann Catalog-Edit `cluster_register`. Ein Schlüssel ohne Cluster (Absturz dazwischen) wird von `idle` nach `orphan_key_grace` (24 h) über `KeyProvider::list` (liefert je Schlüssel `created_utc`) vernichtet.
- **Zustände:** `Live → Retired(S) → [Shredded] → Purged → (Trim)`. Zwischen `Live` und `Retired` existiert nur im Sequencer ein flüchtiges `Retiring(S)` (§6.1): es schließt die Schreibadmission, ist für Leser weder sichtbar noch persistiert. `Shredded` nur bei `Sealed`. Nur vorwärts.
- **Trim:** Ein `Purged`-Eintrag wird per `cluster_trim` entfernt, sobald `history_floor > S` (keine View/Hold/Consumer-Fenster darunter). Danach `UnknownCluster`. Wiederverwendung ausgeschlossen (128-Bit-Zufall). Der Dauer-Nachweis liegt als `tomb_id` im Ledger (§9.7).
- **Restrictions** sind eine Menge am Cluster, kein Zustand.
- `ClusterSpec` ist nach `register` unveränderlich und trägt die Felder `protection` (`Sealed` | `Plain`), `tenant_tag` (16 Byte, opak, vom Aufrufer — verknüpft den Cluster mit der Mandanten-Policy aus `contextra-engine` und bestimmt mit `due` die Packgruppe, §4.1), `due` (optionale Fälligkeit; `None` ⇒ kein Ablauf), `erase_locality` und `expected_bytes` (Größenklasse, ausschließlich Admission-Hinweis). v1-Default ist `PackedBounded{max_file_bytes: 64 MiB}`. `ClusterLocal` bedeutet: Run-/Blob-Dateien dieses Clusters enthalten **ausschließlich** seine Abschnitte (Pure); gepackte Sections erfüllen die Option nicht. Sie ist nur nach Admission (§4.3.1) wählbar (`expected_bytes ≥ cluster_local_min_bytes`, Cluster-Quote, Datei-Quote) und wird nie still herabgestuft; `ClusterLocal`-Cluster sind von der Pure-Demotion (R-PL-3) ausgenommen, die Quote ist daher eine harte Obergrenze der durch sie erzeugten Dateien. Beide Modi unterliegen den separaten Redo-/Pin-Kosten aus `INV-ERA-11`.

#### 4.3.1 Admission für Cluster, Dateien und Metadaten (K-35, K-37)

Ein Cluster kostet RAM, Dateien und Dateideskriptoren. Jede dieser Ressourcen unterliegt einer Admission mit eigener Fehlerklasse, geprüft **vor** der Allokation (`INV-ADM-1`).

**Residenz der Metadaten**

| Struktur | Residenz | Kosten | `MemoryBudget`-Klasse |
|---|---|---|---|
| `ClusterIndex` (offene Adressierung, Schlüssel = `ClusterId`) | immer resident | 36 B je Eintrag: id 16 · `state:8 \| gen:56` 8 · `pack_group` 4 · `hot_ref` 4 · `flags` 4; Rechenwert 40 B je Cluster inkl. Tabellen-Slack (Annahme: Füllung ≈ 0,9, in H-25 zu belegen) | `ClusterMeta` |
| `HotRecord` (Branch-Tabelle, Restrictions, Links, Pins, offene Views, Cursor-Zustand) | nur für Cluster mit solchen Merkmalen; kaltstellbar (Neuaufbau aus Redo/`SYS_BRANCH`), **nicht** kaltstellbar bei Restriction, Link oder Pin | ≤ 256 B + Inhalt | `ClusterHot` |
| `ClusterDirectory` je Datei | nur Sparse-Index (eine Zeile je 4-KiB-Seite, ≈ 0,3 B je Cluster und Datei) und `ClusterFilter` (≈ 1 B je Cluster); Seiten über den Block-Cache | ≈ 1,3 B je Cluster und Datei | `DirIndex`, `BlockCache` |

Rechenbeispiel: 16 Mio. Cluster × 40 B ≈ 610 MiB `ClusterIndex`, unabhängig von der Zahl der Dateien. Kapazität: `resident_clusters_max = cluster_meta_budget / 40 B` mit `cluster_meta_budget` = 15 % von `memory_budget_bytes` (§16). Der Gate-Check bleibt O(1) und ohne E/A.

**Admission-Regeln:**
1. **Cluster-Metadaten:** `register` reserviert `ClusterMeta` (40 B) beim `MemoryBudget`; Fehlschlag ⇒ `Err(Limit{ClusterMetaBudget})` ohne Teilzustand. `open` vergleicht die Clusterzahl des Catalogs mit dem Budget und öffnet andernfalls `ReadOnly` mit Diagnose statt in den OOM zu laufen.
2. **`ClusterLocal`:** nur wenn (a) `expected_bytes ≥ cluster_local_min_bytes` (Standard = `pure_threshold`, 8 MiB), (b) die Zahl lebender `ClusterLocal`-Cluster `< max_cluster_local_clusters` (Standard 1024, ≤ 65 536) und (c) `max_live_files` Reserve für mindestens zwei Dateien je Level lässt. Sonst `Err(Limit{ClusterLocalAdmission})`. Für Massen personenbezogener Cluster ist **`PackedBounded` + Crypto-Erase** der vorgesehene Weg: `KeysDestroyed` wirkt sofort, die Hygiene-Purge läuft amortisiert (§9.5).
3. **Pure-Demotion:** R-PL-3.
4. **`max_live_files`** (Standard 262 144): ab 80 % hebt der Governor den effektiven `pure_threshold` je Stufe um den Faktor 2 und alarmiert; bei 100 % entstehen keine neuen Pure-Dateien (alles Packed) und keine neuen `ClusterLocal`-Cluster. Gate, KeyDestroy und Purge bleiben ausführbar.
5. **FD-Cache:** `max_open_files` (Standard 1024) ist die Größe eines LRU-Caches offener Handles und **kein** Limit der Dateianzahl. Lesepfad, Wartung und Purge teilen ihn; `urgent`-Arbeit hat `fd_reserve` (256) exklusive Handles. `open` prüft `RLIMIT_NOFILE ≥ max_open_files + fd_reserve` über `HostProbe`/Fs-Adapter, sonst `Err(FdLimitTooLow{needed})`.
6. **Verzeichnis-Sharding:** §5.4; keine Millionen Einträge in einem Verzeichnis.
7. **Konvention „Session ≠ Cluster“:** §12.6. Agenten-Sessions leben als Präfix oder TTL-Branch im Eigentümer-Cluster, außer explizit eigenständige Löschdomäne.

### 4.4 Wert, Kopf, Körper, Gewichte

Wert = `header ‖ body`, getrennt durch `header_len`. **Der Kern interpretiert den Kopf nie.** `weights: [u32; W]` (`W = weight_slots`, 0…4); Block-Index hält `weight_sum[W]` und `entry_count`. Körper ≥ `value_separation_min` (1 KiB) gehen in Blobs (`PutRef`), der Kopf liegt immer inline. Retrieval-/Engine-Schichten nutzen `header` u. a. für `Trust-Label` (Byte 0), Bitemporalitäts-Daten und Dokument-Metadaten, und `weights[W]` für Token-Budgetierung (§12.5).

### 4.5 `SpaceOptions` (unveränderlich)

```rust
pub struct SpaceOptions {
    pub class: SpaceClass,          // Durable | Volatile
    pub clustered: bool,
    pub data_class: DataClass,      // NonPersonal | Personal; Personal erzwingt Cluster und Sealed
    pub packing: PackPolicy,        // None | ByTenantAndExpiry{ bucket: Month|Quarter|Year, shards: u8 }
    pub pure_threshold: u64,        // Standard 8 MiB
    pub seal: SealPolicy,           // ByMode | Cluster
    pub weight_slots: u8,           // 0..=4
    pub history: HistoryPolicy,     // KeepLatest | KeepUntilFloor | AppendOnly
    pub value_separation_min: u32,
    pub max_key_bytes: u32,
    pub compression: Compression,   // v1: None
    pub durability_default: Durability,
}
```

`DataClass::Personal` verlangt `integrity_mode = Encrypted`; bei Durable verlangt es `SealPolicy::Cluster` (`INV-SEAL-1`). **Volatile-Spaces:** kein Redo/Run/Blob, im Catalog nur der Name, sofort sichtbar, Inhalt verschwindet bei Schließen/Absturz, Erase = Freigabe mit Überschreiben (best-effort). Mischchanges mit Volatile- und Durable-Zielen nur mit `allow_mixed`, nur `durable`, wenn alle Ziele Durable sind (`INV-VOL-1`). `compression=None` ist die einzige v1-Option (`Unsupported` bei anderen Werten).

### 4.6 Limits

`max_key_bytes` 1 KiB (≤ 4 KiB) · `max_header_bytes` 4 KiB (≤ 64 KiB) · `max_body_bytes` **4 MiB (hart; E-35)** · `max_change_ops` 4096 · `max_change_bytes` 8 MiB · `max_page_bytes` 1 MiB · `max_clusters` 16 Mio. (weich, ≤ 64 Mio.; effektiv durch `cluster_meta_budget` begrenzt, §4.3.1) · `max_branches_per_cluster` 1024 · `max_branch_depth` **4 (Default), 8 (hart)**; Tiefen > 4 nur mit `BranchOptions::deep_ack`; H-21 misst jede Tiefe 1…8. Überschreitung ⇒ `Limit`/`ChangeTooLarge` **vor jeder Sperre** (`INV-LIM-1`).

### 4.7 Datenklassen (`DataClass`)

`DataClass` ist ein geschlossenes Enum mit `NonPersonal` und `Personal`; fehlende Klassifikation wird vor I/O abgewiesen. Die deklarierende Schicht (`contextra-privacy` + `contextra-engine`) ist für die Klassifikation verantwortlich.

| Klasse | Zulässige Ablage | Lösch-/Schutzsemantik |
|---|---|---|
| `NonPersonal` | geclusterte oder nicht geclusterte Spaces gemäß übrigen Optionen | Schutz folgt `SealPolicy`; z. B. globale Kalibrierungs-/Modellmetadaten |
| `Personal` | ausschließlich geclusterter Space; `IntegrityMode::Encrypted`; Durable zusätzlich `SealPolicy::Cluster` und ausschließlich `Sealed`-Cluster | Cluster ist kleinste Crypto-Erasure-Domäne; abgeleitete Retrieval-/Graph-Inhalte erben die strengere Klasse und benötigen `DerivedFrom`-Lineage bzw. registrierten `SegmentOwner` |

Ein Space-Wechsel oder Export darf Klassifikation nie stillschweigend herabstufen. Cross-Cluster-Referenzen zwischen unterschiedlichen Personen sind keine Erlaubnis zur gemeinsamen Schlüsseldomäne.

---

## 5. Platzierung, Formate, Siegel

### 5.1 Pure, Packed, Staging

- **R-PL-1** Jede Datei ist genau eine von Pure (ein Cluster), Packed (mehrere Cluster einer Packgruppe), Staging (nur L0, beliebige Cluster, nur Runs).
- **R-PL-2** Jeder Cluster bildet je Datei **einen zusammenhängenden Abschnitt**, sortiert nach ClusterId, Blöcke clusterrein, Offsets abschnittsrelativ.
- **R-PL-3** Cluster ≥ `pure_threshold` wird bei der nächsten Compaction Pure. Fällt ein **nicht** `ClusterLocal`-Cluster unter `pure_demote_ratio × pure_threshold` (Standard 0,25 ⇒ 2 MiB; Hysterese gegen Flattern), führt eine Compaction ihn per `rebucket` (ein Edit) in seine Packgruppe zurück. `ClusterLocal`-Cluster bleiben Pure (§4.3).
- **R-PL-4** Compaction mischt nur innerhalb einer Wartungsgruppe (Packgruppe oder Pure-Cluster); der Spill (R-PL-7) entmischt Staging.
- **R-PL-5** Ein Flush schreibt je Space **eine** Staging-Datei plus je großem Cluster eine Pure-Datei: L0-Dateizahl unabhängig von der Zahl der Packgruppen.
- **R-PL-6** `rebucket` ist ein atomarer Umzug in einem Edit.
- **R-PL-7 (Spill-Policy, K-36)** Staging-Runs werden size-tiered zusammengeführt (Schreibverstärkung ≈ log) und erst in Packgruppe `g` entmischt, wenn `staged_bytes(g) ≥ max(spill_min_bytes, size(L1(g)) / spill_ratio)` (Standard 8 MiB und 4), oder ihr ältestes gestagtes Byte älter als `spill_max_age` (Standard 6 h) ist, oder ein Purge/`rebucket` die Gruppe berührt. Damit gilt je Spill Schreibverstärkung ≤ 1 + `spill_ratio`. `max_staging_runs` (16) erzwingt Tier-Merge. Punktlesen prüft Staging-Runs nur bei `ClusterFilter`-Treffer.
- **R-PL-8 (MiniSection, K-35)** Ist die Nutzdatensumme eines Cluster-Abschnitts ≤ `mini_section_max_bytes` (Standard 4 KiB), besteht er aus **einem** SealFrame (Einträge, Range-Deletes, ChangeIndex- und BranchTable-Einträge als Teilstrukturen, lineare Suche, kein Filter). Verlässt der Abschnitt die Schwelle (50 % Hysterese), stellt die nächste Compaction auf `FullSection` um.

### 5.2 Siegelrahmen (eine Form für alles)

```text
SealFrame := key_epoch u32 | nonce[24] | ciphertext | tag[16]     (Authenticated: nur tag; Plain: CRC32C)
AEAD = XChaCha20-Poly1305(quelle, nonce[24], AAD)
quelle = Clusterschlüssel(cluster, epoch)   bei SealPolicy::Cluster
       = Space-DEK                          bei SealPolicy::ByMode/Encrypted (Plain-Cluster)
       = gemäß IntegrityMode                bei Plain-Cluster (CRC-only, authentifiziert oder Space-DEK)
       Sealed-Cluster-Nutzlasten in Redo verwenden immer SealPolicy::Cluster mit der im Op-Header gebundenen key_epoch.
AAD: Block `(instance_id, space_id, cluster_id, key_epoch, block_uid)` ·
     Redo-Op `(instance_id, space_id, cluster_id, key_epoch, op_uid, op_index)` ·
     Blob-Record `(instance_id, space_id, cluster_id, key_epoch, record_uid)`.
     `space_id` ist kanonisch unsigned BE-u64; AAD-Domain und Feldreihenfolge sind Teil der Referenzvektoren.
```

Pro `SealFrame` gibt es keine zusätzliche HKDF/HChaCha-Ableitung: `XChaCha20-Poly1305` erhält den bereits scope-spezifischen Quellschlüssel, 24-Byte-Zufallsnonce und AAD direkt. `Authenticated` nutzt die ersten 16 Bytes von `HMAC-SHA256` über kanonisch längencodierte Domain `ctxs-auth-v1`, Modus, Nonce, AAD und Klartext (Tagvergleich constant-time). `Plain` nutzt `CRC32C`. RootKey-abgeleiteter Redo-Schlüssel dient ausschließlich Ketten-MAC/Frame-Integrität. AAD und Siegel hängen nicht von Dateiname, Datei-ID, Blob-ID, Offset oder Dateiposition ab ⇒ **Rohkopie über Dateien und neue Blob-IDs ist ohne Entschlüsselung gültig**. Versiegelt wird vor Stamp-Vergabe.

`INV-SEAL-1`: Sealed-Cluster haben Nutzdaten nur in `SealPolicy::Cluster`- oder `Volatile`-Spaces; Plain-Cluster nie in `SealPolicy::Cluster`-Spaces; Spaces mit `DataClass::Personal` nehmen nur Sealed-Cluster. Klartext bleiben ausdrücklich nur zufällige `ClusterId`, Routing-Header/Op-Tags, Gate-/Control-Frames, Größen-/Stamp-Bereiche und Dateistruktur (Metadaten-Leckagebudget).

### 5.3 Konventionen aller Formate

Little-endian; `varint` = kanonisches LEB128; Dateikopf `magic[8] ‖ file_kind u8 ‖ format_ver u16 ‖ feature_bits u32 ‖ file_id u64 ‖ instance_id[16] ‖ header_crc u32` (CRC32C); Längen vor Allokation prüfen (`INV-LIM-1`); unbekannte Version/Pflicht-Bits ⇒ `Unsupported` (`INV-FMT-1`). Feature-Bit `BRANCHES` (§8) in `FORMAT`: solange ungesetzt, ist `branch ≠ 0` ein Format-Fehler. v1 setzt `COMPRESSION=0` und `filter_kind=Bloom(1)`.

### 5.4 Dateien

```text
<root>/FORMAT  LOCK  CURRENT  CATALOG-<n>  KEYS  RESERVE
       redo/redo-<n>.log
       spaces/<id>/<s>/<file_id>.run|.blob   (s = file_id mod file_shard_count, 3 Hex-Stellen)
       segments/<kind>/<id>.seg              (abgeleitete SegmentOwner-Dateien unter Kern-Io)
<ledger_dir>/ERASE-LEDGER/…                  (Prod-Empfehlung & Prod-Check: getrenntes Volume)
```

`FORMAT` unveränderlich (+ RootKey-MAC); `KEYS` = RootKey (KEK-umhüllt) + Space-DEKs (RootKey-umhüllt); **Clusterschlüssel stehen nie hier**. Alle Dateien `Perm::Private`. Temporäre Dateien `.building.<tmp_id>`. Verwaiste Dateien (nicht im Catalog) werden nach Recovery (`OP9`) entfernt (`INV-CAT-1`).

### 5.5 Redo

Vorallokierte Segmente (64 MiB, `redo_align` 4096), am Ende gesiegelt; Gruppe = ≥ 1 Frames, Ketten-MAC `H_i = MAC(H_{i-1} ‖ head ‖ payload_tags)`.

```text
Frame := head ‖ body ‖ tail
head (Klartext, kettengeschützt): len u32 | stamp u64 | kind u8 | n_ops u16 | idem_token[16] | epoch u64 | flags u8 |
      ops_head[]: { space_id u64, cluster[16], branch u32, op_kind u8, key_epoch u32, op_uid[16], payload_len u32 }
kind: Change=1 | Gate=2 | Control=3 | Reserve=4
Gate-Body (Klartext):    scope(Clusters|Space) | ticket[16] | level | policy_epoch u64 | n u16 | cluster[16]×n oder space_id u64
Control-Body (Klartext): op(Restrict|Lift|Link|BranchFork|BranchDrop|BranchMerge) | … Nutzdaten der Branch-Ops sind versiegelt;
                         Link-Body: derived[16] | n u8 | sources[16]×n | policy_epoch_after u64
tail: crc32c | chain_mac
```

Redo-Integritätsschlüssel = `KDF(RootKey, "ctxs redo-auth v1")`, ausschließlich für Ketten-MAC. Torn-Tail (`INV-LOG-8`): erster ungültiger Gruppenkopf beendet Log nur, wenn nichts Gültiges folgt; sonst `Corrupt{RedoMidLog}`; bis zur letzten `redo_mark` (alle 8 MiB) muss alles gültig sein. **Kein Segmentschnitt beim Gate.**

### 5.6 Run

```text
Run            := header · ClusterSection* · ClusterDirectory · PackMeta · ExtensionBlock* · Footer
ClusterSection := FullSection | MiniSection
FullSection    := DataBlock* · SubIndexBlock · BranchTableBlock · FilterBlock · RangeDelBlock · ChangeIndexBlock
MiniSection    := SealFrame{ entries · range_dels · change_idx · branch_tab }      (≤ mini_section_max_bytes, R-PL-8)
```

`ClusterDirectory` (Klartext): `{cluster, section_kind u8 (0 Full, 1 Mini), section_off, section_len, n_blocks, n_entries, min/max_stamp}`; keine Gewichte/Tokenprofile im Klartext. `DataBlock` Ziel 16 KiB, Präfixkompression, versiegelt; Eintrag trägt `branch`. `SubIndexBlock` je Block: letzter Schlüssel, abschnittsrelativer Offset, Länge, `entries`, `weight_sum[W]`, `min/max_stamp`, `block_uid`; bei Sealed unter Clusterschlüssel versiegelt. `FilterBlock` (v1: Bloom mit cluster-gesalztem Hash über `(branch, user_key)`). `BranchTableBlock` listet je Branch `{branch u32, first_block u32, last_block u32, n_entries, min/max_stamp, has_range_del}`. Im RAM liegen für das Directory nur der **Sparse-Index** (eine Zeile je 4-KiB-Seite, ≈ 20 B) und `ClusterFilter` (≈ 8 Bit/Cluster); Seiten laden über den Block-Cache.

### 5.7 Blob

`Blob := header ‖ ClusterSection* (Record*) ‖ ClusterDirectory ‖ footer`, `Record := record_uid[16] | len u32 | SealFrame | crc32c`. AAD bindet `record_uid`, nicht `blob_id` oder Offset ⇒ Rohkopie ohne Entschlüsselung. Ziel ≤ 64 MiB. **Inkarnation:** Kleine Löcher dürfen bei `punch_hole` dieselbe `blob_id` behalten; oberhalb Hole-/Fragment-Schwelle Umschreibung in neue `blob_id` per atomarem Catalog-Edit `blob_replace`. Blob-Entfernung nur, wenn kein lebender Run mehr per `PutRef` verweist (`INV-BLOB-1`).

### 5.8 Catalog

Snapshot-Record + Edit-Records (Kette, CRC32C) in vorallokierten 1-MiB-Blöcken. **Gruppen-Commit (`INV-CAT-2`):** ein `write` + ein `sync_data` für wartende Edits. Rollover bei `max(8 MiB, Größe des letzten Snapshots)`; `CURRENT` per atomarem `rename`.
Ops: `space_add(space_id,options) | space_state | space_remove | cluster_register | cluster_state | cluster_trim | cluster_link | restriction_set | restriction_lift | pack_assign | files_add(meta.floor) | files_remove | unlink_ack(file_id) | blob_replace | blob_live_delta | replay_floor | retain_floor | redo_mark | segment_add | segment_remove | hold_set | consumer_ack | erase_pending_add | erase_stage | erase_done | erased_ranges_add | erased_ranges_trim | key_epoch | kek_id | idem_trim | reserve_stamps | reserve_ids | reserve_space_ids | anchor_mark`. `files_remove` verschiebt im selben Edit in den persistenten Graveyard; `unlink_ack` räumt nach `unlink + sync_dir`.
Reservierungen: `reserve_stamps` (2^20), `reserve_ids` (2^16), `reserve_space_ids` (2^16); Notpool für `urgent`: `reserve_stamps_urgent` 2^10, `reserve_ids_urgent` 2^8 (schließt Doppelhaltung von Rang 1 + Rang 3 aus, §11.2).

### 5.9 Freeze-Liste (Gate G-F) & System-Spaces

Formate ab G-F eingefroren: Dateikopf, `FORMAT`, `KEYS`, Redo-Gruppe/Frame, Kette, ClusterSection (`Full`/`Mini`, `BranchTableBlock`, branch-bewusster Filter, `ExtensionBlock`), ClusterDirectory, PackMeta (`ClusterFilter`), SubIndex, Siegelrahmen, Pfad-Sharding, Blob-Record, Catalog-Op-Tags, `AnchorValue`, Ledger-Format (`tomb_epoch u32`, Domain-Separation), `cmp`, Tag-Kodierung, sowie die System-Space-Zuordnung:

| System-Space | `SpaceId` | Zweck |
|---|---:|---|
| `SYS_BRANCH` | 1 | clusterlokale Branch-Metadaten (inkl. spekulativer Session-Branches, §8.2) |
| `SYS_IDEM` | 2 | instanzweite Idempotenz-Ergebnisse (§6.5) |
| reservierte Systemzone | 0, 3…255 | zukünftige versionierte Kern-Spaces; User-`SpaceId`s beginnen zwingend bei `256` |

Öffentliche APIs weisen Zugriffe/Mutationen auf `SpaceId < 256` mit `ReservedSpace` ab.

---

## 6. Schreibpfad

### 6.1 `apply` (Commit-Pipeline)

Der Committer ist eine zweistufige Pipeline (K-33):

| Stufe | Sperre | Aufgabe | Verboten |
|---|---|---|---|
| **Sequencer** | Rang 1 (`sequencer`) | Zustandsprüfung, Guard-Delta, `policy_epoch`-Abgleich, Stamp-Vergabe, Übergabe an Syncer. **Rein RAM, O(Ops der Gruppe)** | jede E/A, Provider-/Cipher-/KDF-Aufruf, jedes Warten, Graph-Traversierung (Ausnahme: Selbstberechnung der Löschhülle nach ≥ 3 `PolicyRetry`, ≤ 4096 Knoten, rein RAM) |
| **Syncer** | keine (besitzt Redo-Handle) | CRC + Ketten-MAC, ein `write` + ein `sync_data`, Publikation strikt in Stamp-Reihenfolge | Stamps vergeben, Guards/Zustand validieren |

```text
1  Vorprüfung ohne Sperre: Limits (INV-LIM-1), Space/Cluster bekannt, INV-SEAL-1, Clusterzustand, DiskFull,
   Idempotenz-Lookup (Treffer ⇒ AlreadyApplied)
2  Vorbereiten auf Aufruferthread (ohne Kernsperre; I/O & Provider-Warten erlaubt):
   a) View V1 pinnen, Stufe-A-Guards exakt gegen V1 auswerten (§6.3), Punkt-Fingerabdrücke fp berechnen
   b) bei Gate/Link/Restrict/Lift/Space-Drop: Hülle H und Restrictions berechnen; policy_epoch P1 notieren
   c) Ops kodieren, op_uid ziehen, Nutzlast versiegeln, Frame vorkodieren (Stamp=0, Kette/CRC leer)
   d) implizite Absent-Guard für den SYS_IDEM-Schlüssel anfügen (§6.5)
3  Stamp-Reserve prüfen: erschöpft ⇒ Backpressure(Retry) (normale Changes); urgent nutzt Notpool (§5.8)
4  Pipeline-Credit holen (Semaphor, commit_pipeline_depth=2; urgent hat eigenen Credit). Einziges Warten außerhalb Locks
5  Einreihen: an Intake (Rang 2, kurz), Rang 2 frei. try_lock auf Rang 1:
   Erfolg ⇒ Leader (leert Intake aufsteigend Rang 1 → Rang 2); Misserfolg ⇒ auf Ergebnissignal warten.
   Leader prüft Intake nach Freigabe erneut (kein verlorener Wakeup, loom)
6  Sequencer (Rang 1), je Change in Intake-Reihenfolge:
   a) Zustand: Retiring/Retired/Shredded/Purged ⇒ ClusterRetired; BLOCK_WRITE ⇒ Restricted; Branch nicht Live ⇒ BranchClosed
   b) Guards Stufe B: Konfliktfenster-Delta (V1, head]; Treffer oder V1 < Fensterfuß ⇒ GuardRetry (ab 2a)
   c) Gate/Link/Restrict/Lift: policy_epoch == P1 (sonst PolicyRetry; ab 3 Retries Selbstberechnung)
   d) Stamps vergeben; Branch-Ops linearisieren; bei Gate: cluster_table Live → Retiring(S)
   e) Konfliktfenster fortschreiben (auch für noch nicht durable Gruppen)
   f) Gruppe in Syncer-Queue legen; Rang 1 frei
7  Syncer: Frames finalisieren (CRC, Ketten-MAC), ein write + ein sync_data       ◄── DURABILITÄTSPUNKT
8  Publikation (Syncer, in Stamp-Reihenfolge nach 7): Memtable einfügen, Fenstereinträge durabel setzen,
   Retiring → Retired + gen++, Version veröffentlichen (Rang 4), visible/durable_stamp anheben (Release),
   Aufrufer wecken, Credit freigeben
9  Receipt { instance_id, stamp, durable }
```

**Eigenschaften:** Sync-Fehler vergiftet Handle (`Indeterminate{first_stamp, last_stamp}` bzw. `Poisoned`), keine nachfolgende Gruppe wird veröffentlicht (`INV-PIPE-2`). Leser sehen `Retiring` als `Live` bis Schritt 8; Schreibadmission ist aber sofort zu. `Relaxed`-Changes reiten in der nächsten Gruppe ohne Sync mit (nie für Löschung). Vergleiche und Feed-Offsets sind stets instanzgebunden: `(instance_id, stamp)`.

### 6.2 Memtable und Stamps

Lock-freie Skiplist (`crossbeam-skiplist` hinter Trait `MemIndex`, Klartext gemäß `AN-05`), 64 MiB je Space, ≤ 3 immutable, 256 MiB gesamt. Nach Absturz: `next_stamp = max(stamp_limit, letzter gültiger Redo-Stamp, max Run-Stamp) + 1`. Consumer und `view_durable` sehen nur Stamps ≤ `durable_stamp` derselben `instance_id` (`INV-CONS-1`).

### 6.3 Guards: optimistische Validierung in zwei Stufen

- **Stufe A (Aufruferthread, ohne Sperre):** `V1` pinnen, Guards exakt gegen `V1` prüfen (`Absent | Version(Stamp) | Unchanged{since}` — inkl. Merge-Konfliktprüfung); Konflikt ⇒ `Err(Conflict)`. Sonst Beweise `(V1, fp)` (`SipHash-1-3` unter Prozessschlüssel) bzw. `(V1, ck, [start, end))`.
- **Stufe B (Sequencer, Rang 1, RAM-Konfliktfenster ≤ 1 Mio. Einträge):** Punkt-Guard `fp` mit `stamp > V1` ⇒ `GuardRetry`; Range-Guard gleiches `ck=(space,cluster,branch)` mit `stamp > V1` ⇒ `GuardRetry`; `V1 < Fensterfuß` ⇒ `GuardRetry`. Nach `guard_retry_max` (3) ⇒ `Err(Conflict{contended: true})`.

**Guard-Katalog für obere Schichten (Engine, Agenten, Graph):** Event anhängen → keine Guard; Agenten-/Working-State-Update → `Version`; Fakt/Dokument-Update → `Version` + `CheckDocId`; Tool-Ausführung → `IdemToken` + `Version`; Branch-Merge → `Unchanged{since: merge_base}` (`INV-GUARD-1`).

### 6.4 Flush

Je Space: Memtable einfrieren → je Cluster ein Abschnitt (**`Retired`-Cluster werden übersprungen**) → Staging- plus Pure-Runs (`.building` → `sync_all` → `rename` → `sync_dir`) → **ein** Catalog-Edit (`files_add`, `replay_floor`, `anchor_mark`) → neue Version. `replay_floor` rückt nur über Segmente, deren Frames in durablen Runs stehen **und** deren Gate-Frames als `erase_pending` im Catalog stehen (`INV-LOG-5`).

### 6.5 Idempotenz

`IdemToken` im Frame-Kopf plus interner `SYS_IDEM`-Record (`SpaceId 2`, dedizierter System-DEK, HMAC-Schlüssel & Request-Fingerprint) im **selben Redo-Frame** mit **impliziter `Absent`-Guard** (K-38). Gleichzeitige Retries nutzen denselben 2-Stufen-Pfad (`GuardRetry` → Stufe A findet `AlreadyApplied`). Überlebt `replay_floor` unabhängig bis TTL-Ablauf + durablem `idem_trim`. Abweichender Request bei gleichem Token ⇒ `IdemConflict`; Limitüberschreitung bei aktivem Pool ⇒ `IdemCapacity`; nach Cluster-Retire ⇒ `Erased`/`ClusterRetired`.

### 6.6 Notbetrieb bei vollem Datenträger

Redo, Catalog und Ledger nutzen vorallokierte Dateien (`AN-10`): `ENOSPC` beim Vorallokieren vergiftet nicht (`INV-DISK-1`), sondern liefert `DiskFull` für User-Commits; Gate/KeyDestroy nutzen Emergency-Reserven. Purge besitzt geschützten Scratch-Pool (`≥ 2 × max(run, blob, redo)` = 128 MiB) für einzelnen Datei-Rewrite plus Recovery-Slot (`PurgeBatchEdit` + Unlink + Scratch-Rückgabe). `Open` prüft tatsächliche CoW-/Quota-Reservierungsfähigkeit; fehlt sie, wird harter `DiskFull`-Purge verweigert bzw. deklariert.

---

## 7. Lesepfad

1. `view()` lädt `visible_stamp` (Acquire) vor `Arc<Version>` (`INV-VIEW-2`), `view_durable()` hält ausschließlich einen Root ≤ `durable_stamp`. Keine Sperren über E/A.
2. **Gate zuerst (O(1)):** Unbekannt ⇒ `UnknownCluster`; `Retired`+ ⇒ `Erased{at:S}` **für jede Sicht, auch `view_at ≥ S`**; `BLOCK_READ` ⇒ `Restricted`; widerrufene/abgelaufene Lease ⇒ `Revoked`/`SnapshotTooOld`.
3. **Punktabfrage:** Memtable → immutable → Staging-Runs (`ClusterFilter`) → Level-Dateien (Binärsuche + `ClusterFilter`) → `ClusterDirectory` (Sparse-Index, max. 1 Seite) → `MiniSection` / `FullSection` (`BranchTableBlock` → branch-bewusster Filter → `SubIndexBlock` → **ein** Datenblock je Treffer-Level).
4. **Cursor/Page:** k-Wege-Merge über Quellen eines Clusters und seiner Branch-Kette; **Kopf ohne Body-Block-E/A**; `page(budget)` entsiegelt `SubIndexBlock` (`weight_sum`, RAM-Präfixsummen, K-30), überspringt Blöcke ohne Body-Lesen, bricht bei Budgetüberschreitung ab (`oversize_single` garantiert Fortschritt bei leerer Seite) und liefert unter Prozessschlüssel AEAD-versiegeltes `ResumeToken`. Cursor prüfen Gate, Widerruf und View-Lease zwischen Blöcken.
5. **Körper** erst bei `body()`; `KeyUnavailable`/`KeyLost` fail-closed. `Shredded` ist Clusterzustand, nie Lesefehler.
6. **Block-Cache:** Schlüssel `(cluster, gen)`; **Check-after-decrypt** von `gen` vor **und** nach Entsiegeln (`INV-KEY-2`). Gate und ER2 erhöhen `gen` und überschreiben Puffer (`Zeroizing`).
7. `view_at(S)` mit `S < history_floor` ⇒ `BelowFloor`; Sicht unter `S_erase` in `erased_ranges` ⇒ `Erased` (`INV-ERA-5`).
8. **`View::filter_visible(&[RecordRef])`:** Prüft Clusterzustand/Restriction und Stamp für Treffer oberer Indizes (`vector`, `text`, `graph`, `rank`). **Jedes Ergebnis einer Extension muss diesen Filter passieren, bevor es den Aufrufer erreicht (`INV-VIS-1`).**
9. Keine clusterübergreifenden Kernscans (`Inspector::clusters()` nur administrativ ohne Nutzdatenentsiegelung).

### 7.1 Chain-Resolve und Branch-Kosten (K-34)

1. **Layout:** `cmp` sortiert `branch` vor `user_key`; Branch-Einträge sind je Abschnitt zusammenhängend (`BranchTableBlock`).
2. **Filter:** Branch-bewusster Bloom über `(branch, user_key)` mit cluster-gesalztem Hash.
3. **Punktlesen = Chain-Resolve:** Kette `[b0 = Blatt … bd = Wurzel]` mit Stamp-Kappen `cap_0 = V`, `cap_{i+1} = min(cap_i, fork_stamp(b_i))`. Je Ebene/Datei: `BranchTable` negativ ⇒ weiter; `has_range_del` ⇒ `RangeDelBlock`-Prüfung (`O(log r)`, auch bei negativem Punktfilter); Bloom negativ ⇒ weiter; Blocklesung im Branch-Bereich. Erster sichtbarer Treffer (`≤ cap_i`) bzw. überdeckendes `Delete`/`RangeDelete` liefert Early-Exit.
4. **Kosten & Grenze:** ≈ `1 (main) + d × FPR` Datenblöcke plus RAM-Proben; Default `max_branch_depth = 4`, hart `8` (`BranchOptions::deep_ack`). Spike H-21 vor G-F.

---

## 8. Versionierung (Branches, Diff, Merge)

### 8.1 Modell

| Git-Begriff | `contextra-storage` |
|---|---|
| Commit | durabler `Change` (Frame, Stamp = `Receipt.stamp`) |
| Zeitreise | `view_at(stamp)`, `View::on_branch(b)` |
| Branch | clusterlokale `BranchId u32` (`0 = main`); **O(1)-Fork** ohne Datenkopie |
| Diff | `diff(view_a, view_b, cluster, range)` |
| Merge | `merge(cluster, source, target, mode)`: Konfliktliste oder atomarer Change |
| Rollback | `branch_drop` (spekulative Äste) oder `reset`-Change |

Clusterlokalität verhindert Cross-Cluster-Hashes/DAG-Verflechtungen: **Löscht ein Cluster, sterben alle seine Branches und Session-Äste mit (`INV-BR-3`).**

### 8.2 Branch-Metadaten

Liegen **im Cluster**, nicht im Catalog: reservierter Space `SYS_BRANCH` (`SpaceId 1`, clustered, folgt `DataClass` des Clusters): `branch_id → { name (≤ 64 B, opak), parent, fork_point: Head|At(Stamp), state: Live|Merged|Dropped, ttl, created_by }`. Bei `Head` definiert der äußere MVCC-Stamp des Erstellungsrecords den `fork_stamp`.

### 8.3 Lesen auf einem Branch

Sicht `(V, b)` löst rekursiv entlang `parent(b)` mit Stamp-Obergrenze `min(V, fork_stamp(b))` auf. `Delete`/`RangeDelete` auf Kind-Branch verdeckt Elternversionen. Clusterweites `EraseRange` wird vor Versionsauflösung angewandt und durch keinen Merge aufgehoben.

### 8.4 Fork, Drop, TTL

`Storage::fork(cluster, parent, ForkPoint::{Head, At(Stamp)}, BranchOptions { name, ttl })` linearisiert als Control-Frame im Sequencer (BranchId zuvor via `reserve_ids` reserviert, Überlauf ⇒ `Exhausted`, `INV-BR-4`). `branch_drop` nur auf Leaf-Branches (`BranchHasChildren` sonst); TTL-Expiry bottom-up. Gemergte Sources sind read-only bis `branch_drop`. `At(S)` erfordert `history_floor ≤ S ≤ parent_head` (`BelowFloor` sonst). `gc_floor = min(aktive Views, Holds, Consumer-Stamps, lebende Fork-Stamps)`; `history_floor ≤ gc_floor`.

### 8.5 Diff und Merge

- **Diff:** Über `ChangeIndexBlock` (cluster-sealed, sortiert nach `(stamp, op_uid)`), Kosten ∝ Mutationen seit gemeinsamem Floor + Ausgabe.
- **Merge `source → target`:** v1 nur direkter Child → Parent (`parent(source) == target`, `merge_base = fork_stamp(source)`). Prüft via `ChangeIndex` Punkt- und Intervallüberlappungen (gleiche Endzustände kein Konflikt). Bei Konflikt `Conflicts(Vec<ConflictCell>)` ohne Write; bei Erfolg/Auflösung atomarer `Change` mit Guards seit `merge_base` und Head-Revalidierung (`source_head`, `target_head`, `merge_base`) plus `source → Merged` im selben Frame (`INV-BR-2`). Core-Modi: `FailOnConflict | PreferSource | PreferTarget`; semantische Auflösungen (LWW, Textmerge, LLM-Konsolidierung) liefert die Engine oben.

### 8.6 Zeitmodell

`Stamp` = Transaktionszeit im Kern. Gültigkeits-, Beobachtungs- und Quellzeit (`valid_from/to`, `observed_at`, `source_time`) liegen im `header` der oberen Schicht; `BlockPruner` (§12.2) erlaubt Blockskipping für schattensichere Zeitlanes auf `main` ohne Formatänderung am SubIndex (`K-29`).

---

## 9. Löschung

### 9.1 Prinzip

Löschen ist die primäre Architekturachse. Drei geordnete Stufen: **Gate (`ER1`)** < **Schlüssel (`ER2`)** < **Purge (`ER5–ER6`)**. Der Kern attestiert nie mehr als technisch belegt.

### 9.2 Stufen (zwei Achsen, K-23)

Jeder Löschauftrag trägt eine zufällige `EraseRequestId` (unabhängig vom 24h-`IdemToken`); identischer Retry liefert dasselbe Ticket, veränderter Scope liefert `IdemConflict`.

| `EraseLevel` (Ziel) | Für | Verfahren | Schützt gegen | Schützt nicht gegen |
|---|---|---|---|---|
| `Logical` | beide | Gate (`ER1`) + `on_gate` | Lesen über Kern- & Index-APIs | Inhalt auf Platte / RAM-Allokator |
| `Physical` | `Plain` | Gate, Barriere (`ER4`), Purge, Edit, Unlink, `sync_dir` + Index-Rewrite | Lesen aus lebenden Dateien, Rollback (mit Anker) | FS-Journal, Flash-Reste, externe Backups |
| `Scrubbed` | `Plain` | wie `Physical` + Überschreiben vor Unlink | klassische Datenträger-Forensik | CoW-Dateisysteme, SSD-Wear-Leveling (best-effort) |
| `Cryptographic` | `Sealed` | Gate, **Vernichtung aller Scope-/Extension-Schlüssel (`probe == Absent`)**, danach Hygiene-Purge & `verify_absent` | verschlüsselte Reste in Kern, Checkpoints & schlüsselgebundenen Backups | gemeinsam geschlüsselte unbereinigte Rebuildable-Indizes (§12.4), Schlüsselkopien |

| `Stage` (Fortschritt) | Bedeutung |
|---|---|
| `Gated` (G0) | Gate durabel im Ledger + Catalog/Sequencer, Cluster `Retired(S)` |
| `KeyDestroying{eta}` / `KeysDestroyed` (G1) | Schlüssel im Provider deaktiviert / für **alle** Epochen bestätigt `Absent` |
| `Purging` / `Purged` (G2) | alle betroffenen Bytes aus lebenden Kern-Dateien und `Rebuildable`-Indexsegmenten entfernt |
| `Done` | alle Owner attestiert, Ledger durabel, Anker fortgeschritten, `ErasureReceipt` ausgestellt |

Ein Cluster-Ticket endet nur auf passender Endstufe: **`Sealed` ⇒ `Cryptographic`**, **`Plain` ⇒ `Physical`/`Scrubbed`**. `Logical` als alleinige Cluster-Stufe endet als `CompleteWithResidual{LogicalOnly}` (oder wird per `escalate` fortgeführt).

### 9.3 Garantie-Kernsätze

1. Ab durablem Gate liefert kein Pfad (Kern oder Extension) Inhalt des Clusters; pro Cluster gilt voller Snapshot oder `Erased` (`INV-ERA-1`).
2. Ab `KeysDestroyed` sind alle Kerndaten (Redo, Runs, Blobs, Checkpoints) des Sealed-Clusters ohne Schlüssel unlesbar (`INV-ERA-7`). Registrierte Extensions erreichen G1 mit `ER2` nur bei identischem Scope-Key oder bleiben bis `Purge + verify_absent` im Zustand `Purging`.
3. Nach `Purged` trägt keine lebende Kern- oder Segmentdatei Bytes des Clusters (`INV-ERA-2`).
4. Gate/KeyDestroy skalieren mit der Zielmenge; physischer Purge bleibt auf betroffene Dateien/Packgruppen und deklarierte Rebuildable-Grenzen beschränkt (`INV-ERA-11`).

### 9.4 Schlüssel und `KeyProvider`

```text
KEK (KeyProvider, extern)  └─ RootKey (32 B, in KEYS mit KEK umhüllt)
       ├─ Space-DEK (je Space, mit RootKey umhüllt)
       ├─ Redo-/Catalog-/FORMAT-/Ledger-Schlüssel = KDF(RootKey, Label)
       └─ LedgerScopeKey = KDF(RootKey, "ctxs ledger scope v1") (AEAD für Selector-Ciphertexts)
Clusterschlüssel: KeyProvider::cluster_key(cluster, epoch)      (NIE in KEYS oder Kern-Backups)
Tomb-Schlüssel:   KeyProvider::tomb_key(tomb_epoch)             (NIE in KEYS oder Kern-Backups)
```

```rust
pub trait KeyProvider: Send + Sync {
    fn info(&self) -> KeyProviderInfo;
    fn kek(&self, id: KekId) -> Result<Key32, KeyError>;
    // Batch ≤ 256, idempotent je BatchId, jeder gemeldete Erfolg vor Rückgabe durabel
    fn create_cluster_keys(&self, request_id: BatchId, cs: &[ClusterId], e: &dyn Entropy) -> Result<Vec<KeyCreateResult>, KeyError>;
    fn cluster_keys(&self, req: &[(ClusterId, u32)]) -> Result<Vec<KeyLookup>, KeyError>;     // Missing ≠ Shredded
    fn destroy_many(&self, cs: &[ClusterId]) -> Result<DestroyReceipt, KeyError>;              // ALLE Epochen, ≤ 256/Aufruf
    fn probe(&self, c: ClusterId, epoch: u32) -> Result<KeyProbe, KeyError>;                   // Present | Disabled | Absent
    fn list(&self) -> Result<Vec<KeyListEntry>, KeyError>;                                     // { cluster, created_utc }
    fn create_tomb_epoch(&self, epoch: u32) -> Result<(), KeyError>;
    fn tomb_key(&self, epoch: u32) -> Result<Option<Key32>, KeyError>;                         // None ⇒ retired
    fn retire_tomb_epoch(&self, epoch: u32) -> Result<KeyProbe, KeyError>;
}
pub struct KeyProviderInfo { pub destroy: DestroyMode, pub keys_in_backups: KeysInBackups, pub assurance: KeyAssurance }
pub enum DestroyMode { Immediate, Scheduled { max_delay: Duration } }
pub enum KeysInBackups { No, Bounded { until: Duration }, Yes }
pub enum KeyAssurance { Hardware, Kms, FilePassphrase }
```

- **`Disabled`** erlaubt nur Planung/Scratch-Reserve (`ER3`), **niemals** Start von physischem Purge oder `Done` (`INV-ERA-3`).
- **`KeysInBackups`:** `No` ⇒ `Cryptographic` voll zulässig; `Bounded{until}` ⇒ Statuscap `CompleteWithResidual{BoundedBackupKeys{until}}` bis Ablauf; `Yes` ⇒ `Err(Unsupported)`.
- **`KeyAssurance`:** `FilePassphrase` cappt auf `CompleteWithResidual{WeakKeyAssurance}` (`OpenProfile::Production` weist ohne explizites opt-in mit `ProviderAssuranceInsufficient` ab).
- **Tomb-Epochen & Epochenabschluss:** `KeysDestroyed` verlangt `probe(c, e) == Absent` für **alle** Epochen `e ≤ key_epoch` (`INV-KEY-4`). `keys-cohort` führt intern Cohort-Rewrap (≤ 1024 je Cohort, `keys_in_backups = false`, Replikation in ≥ 2 Failure Domains), für den Kern unsichtbar (`K-04`). Jede Implementierung besteht `KeyProviderConformance`.

### 9.5 Protokoll `Eraser::cluster` (`ER0–ER7`)

```text
ER0 Vorprüfung:  H = {c} ∪ transitiv DerivedFrom-gebundene Cluster; BLOCK_ERASE in H ⇒ Err(Restricted);
                 Level passend zu Protection; Provider & Assurance ok
ER1 Gate:        1) Durables Ledger EraseIntent (≤ 4096 sortierte tomb_ids)
                 2) Sequencer-Op Gate-Frame { scope=Clusters(H), ticket, level, policy_epoch }       ◄── LOGISCH (G0)
                    (Mismatch policy_epoch ⇒ Retry ab ER0, max 3×, dann Sequencer-Selbstberechnung)
                 3) Nach durablem Sync: Retiring(S) → Retired(S), gen++, erase() kehrt Ok(ticket) zurück;
                    synchroner Aufruf von SegmentOwner::on_gate(H, S, gen) für alle registrierten Retrieval-Indizes!
                 4) Urgent-Lane: Memtable/ReadCache leeren/nullen, Catalog-Edit {cluster_state, erase_pending_add},
                    Ledger GateApplied → Anchor-CAS
ER2 Schlüssel:   (Cryptographic) KeyDestroyIntent durabel → destroy_many(H) in Chunks ≤ 256;
                 erst bei probe == Absent für alle Epochen KeysDestroyed, Cache-Gen++, Edit → Shredded ◄── KRYPTO (G1)
ER3 Plan:        Dateien aus Packgruppe & ClusterDirectory + alle betroffenen SegmentOwner (Vector, Text, Graph, IdentityDir)
ER4 Barriere:    NUR Plain/Space-Drop: Segmentschnitt, Flush, replay_floor/retain_floor vorziehen (bündelbar)
ER5 Umbau:       Je Kern-Datei purge() (PurgeBatchEdit + Unlink + unlink_ack) UND je SegmentOwner rewrite_without() + verify_absent()
ER6 Abschluss:   Nach Verify + allen SegmentOwner-Attesten: Edit {H→Purged, erase_done, anchor_mark}    ◄── PHYSISCH (G2)
ER7 Danach:      Ledger Done, Anker vorziehen, ErasureReceipt (Basis für signierten DeletionProof in contextra-erasure)
```

**Reihenfolge (`INV-ERA-3`):** `EraseIntent < Gate < GateApplied < Anchor < KeyDestroyIntent/KeyDestroy (Absent) < PurgeBatchEdit < Unlink+sync_dir < UnlinkAck < ER6/Done`.
**Frist & Amortisierung (K-36):** `purge_deadline` ist Pflichtparameter. Auslöser für Datei-Rewrite: Compaction-Huckepack, 70 % Frist verbraucht oder ≥ 16 Cluster je Datei; höchstens ein Rewrite pro Fenster `W = 0,3 × purge_deadline`. `Plain` läuft auf `urgent`; `Sealed` auf `compact` (Eskalation auf `flush` ab 70 %, `urgent` ab 90 %) — so verdrängt Crypto-Purge-Hygiene keine normale Wartungsbandbreite.

#### 9.5.1 `purge` (einziger Kern-Dateilöschalgorithmus)

Für jede betroffene Run-Datei oder Blob-Inkarnation `F` einzeln: ist `keep = Cluster(F) − D == ∅` ⇒ `PurgeBatchEdit{files_remove(F)}` (Graveyard); sonst Rohkopie der Überlebenden `keep` in vorallokierten Scratch-Slot (CRC-Prüfung ohne Entschlüsselung!), `sync_all`, `rename`, `sync_dir`, `PurgeBatchEdit{files_add(F'), files_remove(F)}`. Nach Reader-Drain (`Arc<Version>` + Cache): `unlink(F)`, `sync_dir`, `unlink_ack(F)`. Erst wenn alle Tickets einer Graveyard-Menge leer sind und **alle registrierten `SegmentOwner` (`vector`, `text`, `graph`, `identity`)** per `verify_absent` attestiert haben, setzt `PurgeCompleteEdit` `Purged/Done`.

#### 9.5.2 Löschauftrag auf vergiftetem Handle (E-75)

Ist das Speicher-Handle im Zustand `Poisoned` (F-3), nimmt der Kern die Löschabsicht atomar über den unabhängigen `Ledger`-Handle auf (`EraseIntent{origin: Poisoned}`, `Ok(Ticket{state: IntentRecorded})`), trägt die Cluster sofort in `pending_erase` im RAM ein (alle Reads/Feeds liefern `Err(ErasePending)`, Cache wird überschrieben) und führt beim nächsten Neustart (`OP8`) die Gate- und Folgestufen vor jeder Nutzersicht idempotent aus (`INV-ERA-13`).

### 9.6 Restriction, Hold, Links (`DerivedFrom`)

- **`Restriction`:** `RestrictionSpec { reason u16 ≠ 0, owner, effects (BLOCK_ERASE immer | BLOCK_WRITE | BLOCK_READ), expires }`. Kein Override im Kern; `lift` nach `LiftPolicy::{OwnerOrAdmin, FourEyes}`. `restrict`/`lift` laufen als Control-Frames durch den Sequencer (`INV-ERA-10`) und werden beim Gate **und** Purge geprüft.
- **`Hold`:** rein technisch, `ttl ≤ 24 h`; wird von jedem `Erase` überstimmt (wichtig für `contextra-checkpoint`, K-46!).
- **Links (`DerivedFrom`):** `Clusters::link(derived, sources[≤ 8])` läuft als Sequencer-Control-Frame `Link` (total geordnet mit Gate, §11.6 `RACE-22`). Admission vor Rang 1 prüft (a) azyklisch, (b) `max_link_depth = 8`, (c) für **jeden Vorfahren-oder-selbst** `a` jeder Quelle `|Hülle(a) ∪ Hülle(derived)| ≤ max_gate_clusters (4096)`, (d) keine Quelle `Retiring`/`Retired`. Löschung jeder Quelle zieht abgeleitete Cluster **im selben Gate-Frame** mit (`INV-ERA-12`). Cross-Subject-Prüfung obliegt der oberen Schicht (nie fremde Personen per `DerivedFrom` koppeln!).

### 9.7 Ledger, Anker, Receipt

- **Ledger:** append-only, vorallokiert, Ketten-MAC (`ledger_key`), **ein Schreiber** (bündelt Gruppen), konfigurierbares `ledger_dir` (`OpenProfile::Production`: eigenes Gerät gegenüber Datenverzeichnis per `HostProbe`, sonst `LedgerSameDevice` außer `_ack`, `LedgerUnavailable` bei Defekt). Trägt nur `tomb_id` (`tomb_epoch u32`), `space_tomb_id` und AEAD-versiegelte Selektoren (`LedgerScopeKey`) — **nie** Klartext-`ClusterId`, `SpaceId`, Personen-IDs oder Nutzdaten (`INV-LED-1`).
- **Anker:** `AnchorValue = (erase_epoch, ledger_head, catalog_epoch)`. CAS-Fortschreibung nach Gate-Catalog-Edit und zwingend **vor** `KeyDestroyIntent`/`KeyDestroy` oder Purge-Barriere. `Required` bei Encrypted/Authenticated.
- **`ErasureReceipt`:** `{instance_id, req/ticket, scope_digest, level, t_requested, t_logical, erase_epoch S, t_crypto?, t_physical?, t_verified?, stages, key_attestations, provider{destroy, keys_in_backups, assurance}, ext_acks, verifier_report_digest[MeasureKind], ledger_head, anchor{strength, value}, restrictions_encountered, status (Complete | CompleteWithResidual | Incomplete | Blocked | Failed), residual, canonical_bytes}`. Offline prüfbar über `verify_receipt(receipt, ledger_slice, anchor_attestation) → Verdict`. Statuscaps gemäß §9.4 und `INV-VER-1`; Ed25519-Signatur und Compliance-Export liefert `contextra-erasure`.
- **`Eraser::verify`:** strukturell ohne Schlüssel (lebende Dateien, Memtable, `probe == Absent`, Ledger bis Anker gültig) plus Auswertung aller `ext_acks` der registrierten `SegmentOwner`.

### 9.8 Weitere Löschpfade

- **`Eraser::range(space, cluster, range)`:** maskiert clusterweit über alle Branches & Views (`EraseRange`). `Plain`: `Logical|Physical|Scrubbed`; `Sealed`: ausschließlich `Logical` (selektive Bereichs-Kryptolöschung in einem gemeinsamen Cluster existiert in v1 nicht).
- **`Eraser::branch(cluster, branch)`:** nur terminale Leaf-Branches (`Plain`: `Logical|Physical|Scrubbed`; `Sealed`: `Logical`).
- **`Storage::space_drop(space, req, level, deadline)`:** prüft Restrictions vor Mutation; schreibt `SpaceDropIntent{space_tomb_id, level}`; maskiert gesamte `SpaceId` im Gate-Frame; ermittelt betroffene Cluster danach asynchron aus gepinnter Version (`SpaceScopeFrozen{count, scope_digest}`). `Cryptographic` auf Space-Ebene ⇒ `Unsupported` (Cluster können Spaces teilen; dafür explizite Cluster-Tickets nutzen).
- **`pack_expired(PackKey)`:** zerlegt nicht-gesperrte Cluster einer Packgruppe deterministisch in Tickets ≤ 4096.
- **Mehrbetroffene Falldaten (K-25):** gemeinsamer Fallcluster; `DerivedFrom`-Link nur dort, wo Löschung einer Person den Fallcluster mitziehen **soll**; andernfalls getrennte Referenz.

### 9.9 Restexposition (Pflichtbestandteil jedes Receipts)

| Bereich | Restrisiko | Zuständig & Diagnose |
|---|---|---|
| Prozessspeicher, Swap, Core-Dumps, Skiplist-Knoten | Klartext-Reste | `Zeroizing`, Cache-Drop beim Gate; `mlock`/`DONTDUMP` (`contextra-sys`); Betreiber-OS; `HostProbe` (`Declared` falls `Unknown`) |
| Dateisystem-Journal, Flash-Wear-Leveling, Snapshots, Backups | alte Blockversionen | nur Ciphertext bei `Sealed`; bei `Plain` ungeschützt; Betreiber-Retention |
| KMS-Karenzzeit / Backup-Schlüssel / `FilePassphrase` | Schlüssel temporär wiederherstellbar oder Medium nicht beweisbar gelöscht | `KeyDestroying`/`Blocked` bzw. `CompleteWithResidual` (`BoundedBackupKeys` / `WeakKeyAssurance`) |
| Metadaten & Ledger | pseudonyme `ClusterId` (bis Redo-Purge), `tomb_id` (bis `retire_tomb_epoch`), Größen, Packgruppe | epochengebundene HMACs, `tomb_epoch`-Retire (§10.4), optional Padding |
| Rollback im Ankerfenster / `LedgerSameDevice` | Rücksetzfenster bei schwachem Anker oder gleicher Failure Domain | Anker-Stärke & `HostProbe`-Geräteabgleich im Receipt |
| Obere Schichten (Prompt-Caches, unregistrierte Derivate, LLM-Provider-Logs) | **nicht** durch Kern oder SegmentOwner lösbar | nur über `SegmentOwner` (Vector, Text, Graph, KV-Cache, IdentityDir) angebundene Derivate sind abgedeckt |

---

## 10. Recovery, Checkpoint, Restore

### 10.1 Öffnen (deterministisch, `OP1–OP10`)

```text
OP1   LOCK (exklusiv; ReadOnly geteilt)
OP2   FORMAT lesen → KEYS entpacken (KEK ⇒ RootKey) → FORMAT-MAC prüfen → LedgerScopeKey bereitstellen
OP2a  Anker/Ledger: Catalog-Epoche < Strong-Anchor ⇒ Err(Rollback). Ledger muss ≥ E_a mit Hash H_a tragen.
      Ledger-Intent hinter Anker = Crash-Präfix → idempotent fortsetzen; Required-Anchor offline ⇒ AnchorUnavailable
OP3   CURRENT → Catalog bis letzter valider Record (Torn Tail verwerfen)
OP4   Verzeichnisse scannen (unreferenzierte Dateien vermerkt, erst OP9 räumt auf)
OP5   KeyProvider prüfen; KEK fehlt ⇒ LockedKeys
OP6   Redo ab replay_floor, zwei Durchläufe (+ versiegelte Ledger-Intents als Gate-Wahrheit):
        1: Kette prüfen; Full-Retires, Range-/Branch-Masken und Space-Gates einsammeln
        2: Full-retired Cluster nie entsiegeln; Selector-Masken anwenden;
           KeyUnavailable ⇒ LockedKeys; KeyLost ohne Erase-Intent ⇒ Quarantäne/Corrupt; Branch-/Merge-Frames einspielen
OP7   Memtable aufbauen; visible = durable = letzter gültiger Stamp; Branch-Tabellen lazy
OP8   Alle Ledger-Intents (inkl. origin=Poisoned aus §9.5.2) idempotent in Catalog-Gates überführen;
      alle offenen Tickets/Pending-Erases dringend fortsetzen bevor Views/Schreibzugriffe und Extensions freigegeben werden
OP9   Verwaiste Dateien und .building.* entfernen (strikt NACH OP8!)
OP10  Reserven (Stamp, Notpool, ID, Redo, RESERVE) herstellen; Admission (RLIMIT_NOFILE, max_live_files,
      cluster_meta_budget) prüfen; Wartung starten → OpenReport → SegmentOwner-Recovery & Watermark-Catchup (§12.4)
```

### 10.2 Absturzmatrix (Auszug, vollständig in `FaultFs`-Suite)

| Unterbrechung | Soll |
|---|---|
| vor Redo-`sync` / nach Redo-`sync` vor ACK | nicht sichtbar / vorhanden (`IdemToken` verhindert Doppelung) |
| mitten in Catalog-Gruppe / Run-Bau / vor Unlink | gültiger Präfix bzw. Verwaisten-Bereinigung nach OP8; Redo spielt nach |
| Ledger-Intent durable, Gate-Edit fehlt (`origin=Poisoned` o. normal) | `OP8` rekonstruiert Gate, zieht `GateApplied` und Anker nach |
| Anchor durable, KMS offen / KMS `Absent`, Event fehlt | `KeyDestroyIntent` + idempotentes `probe`/`destroy_many` zieht `KeysDestroyed` nach |
| `PurgeBatchEdit` durable, Unlink oder `unlink_ack` offen | Graveyard-Recovery entlinkt/bestätigt idempotent vor `PurgeCompleteEdit` |
| Branch-Fork / Merge / Checkpoint / `ENOSPC` bei Vorallokation | Replay aus Redo bzw. kein Teilzustand (`dst` ohne Manifest verworfen) |

### 10.3 Checkpoint, Restore, Rollback-Schutz

- **Speicherkern-Checkpoint (`contextra-storage::checkpoint`):** (1) Flush-Barriere; (2) unter `catalog_edit` Version, `erase_epoch`, `tomb_epoch` und Ankerwert pinnen; (3) unveränderliche `Copy`-Dateien (Reflink/Hardlink nur bei registriertem `CheckpointOwner`, der Extent-Bereinigung bei Purge attestiert); (4) `CheckpointManifest` siegeln. **`KEYS` ist enthalten, KEK, Clusterschlüssel und `Ledger` niemals (`INV-KEY-1`).** (Abgrenzung zu Agenten-`CheckpointGuard` in `contextra-checkpoint`: §12.6).
- **Restore (`RestoreQuarantine` → `Running`):** Ersetzt Instanz unter Beibehaltung der `InstanceId` (`NZ-19`, externes Writer-Fencing Pflicht vor Schreiböffnung!). Prüft `BackupMark` (`OP2a`/§10.4), vergleicht mit Ledger und externem `Anchor`, wendet **alle** Intents (`tomb_id`, `LedgerScopeKey`-Range/Branch, `space_tomb_id`) vor Datenzugriff scope-genau an (`INV-ERA-9`). Abgeleitete Indizes (`vector`, `text`, `graph`) bauen nach `RestoreReconciled` ausschließlich aus der rekonstruierten kanonischen Sicht ab `durable_stamp` neu bzw. wenden Delta ab `(applied_stamp, durable_stamp]` an — geladene Altsnapshots externer Indizes durchlaufen vorher zwingend Gate-Reconcile.

### 10.4 Backup-Horizont und Tomb-Retire (K-31, E-74)

`backup_horizon` ist die maximale Aufbewahrungsdauer aller Backups. Jeder Checkpoint/Backup trägt `BackupMark{instance_id, erase_epoch, created_at, tomb_epoch}`. Retire einer geschlossenen `tomb_epoch e` ist zulässig, sobald `now − closed_at ≥ backup_horizon` (ohne `ClockUncertain`), kein offenes Ticket/Restore darauf verweist, `TombEpochRetireIntent` (`restore_floor' = max(restore_floor, last_erase_epoch(e))`) + Anker durabel sind und `KeyProvider::retire_tomb_epoch(e) == Absent` ist (`INV-LED-2`, `INV-KEY-5`). Danach werden Backups `< restore_floor` mit `BackupBeyondHorizon` abgewiesen; Ledger-Einträge dieser Epoche sind anonym bei fortbestehender Ketten-/Receipt-Prüfung.

---

## 11. Nebenläufigkeit

### 11.1 Prinzipien

Ein `Sequencer` (Rang 1, nur RAM) und ein `Syncer` (Redo-Append/Sync ohne Lock); unveränderliche Dateien und `Arc<Version>`; `Claims` auf Dateimengen; Generationszähler für Caches und Gate-Zustand; genau ein Schreiber je Kanal für Ledger und Anker; Retrieval-Indizes synchronisieren intern nach derselben Stamp-Ordnung ohne Haltung von Speicherkern-Sperren über CPU- oder I/O-Arbeit.

### 11.2 Sperrränge (strikte totale Ordnung, `debug_assertions` Rangstack)

| Rang | Sperre | Über E/A | Bemerkung |
|---|---|---|---|
| 1 | `sequencer` | **nein** | Stamp-Vergabe, `Live → Retiring`, Link/Restrict/Lift, Branch-Ops; nur RAM, O(Ops) |
| 2 | `intake` | nein | Vom Leader unter Rang 1 aufsteigend geleert |
| 3 | `catalog_edit` | ja (Catalog-Sync) | Gruppen-Commit |
| 4 | `state` (`RwLock<Arc<Version>>`) | nein | Atomarer Snapshot-Wechsel |
| 5 | `registry` (`Holds`, `Consumer`, `Claims`, `Restrictions`, `ViewInner`) | nein | Blatt |
| 6 | `cluster_table` (sharded `RwLock`, inkl. Branch-Tabellen) | nein | Blatt; Lesen aus Rang 1 zulässig |
| 7 | `maint` (je Lane) | nein | Blatt |
| 8 | `notify` | nein | Blatt |

Rang 1 und Rang 3 schließen einander strikt aus (Notpool, §5.8). **Kein Rückruf (`Observer`, `SegmentOwner`, `KeyProvider`, Embedding-/Graph-Berechnung) unter irgendeiner Kernsperre (`INV-LOCK-1`).** Außerhalb `sync::Ranked` gibt es im Speicherkern kein `std::sync::Mutex`. In oberen Crates gilt zusätzlich: keine `tokio::sync::RwLock`/`parking_lot`-Sperre darf über `.await` gehalten werden (`await_holding_lock = "deny"`), und innerhalb von `SessionBranchTree` erzwingt das Typ-API-Design (`NodesGuard`/`NodesWriteGuard`) die Reihenfolge `nodes → edges/active_head`.

### 11.3 Lock-freie Pfade

Publish-Reihenfolge: neue `Arc<Version>` veröffentlichen, danach `visible_stamp`/`durable_stamp` (`Release`) anheben; Leser laden Stamp (`Acquire`), dann `Version`. Clusterzustand = `AtomicU64 (state:8 | gen:56)`; nur Sequencer setzt `Live → Retiring`, Publikation nach durablem Sync setzt `Retiring → Retired` (`INV-LOCK-2`).

### 11.4 Views, View-Lease und Graveyard

`ViewInner { version: RwLock<Option<Arc<Version>>> }`. `ViewPolicy::Revoke` entzieht die Referenz spätestens nach `revoke_grace` (5 s) (`INV-GRAVE-2`). Unlink erst nach durablem `files_remove` **und** Fallen der letzten `Arc<Version>` (`INV-GRAVE-1`); erst `unlink_ack` löscht Graveyard-Eintrag.
**View-Lease (E-76):** Jede `StorageView` / jeder `Cursor` trägt eine Lease (`view_lease` 60 s, verlängerbar via `renew()` bis `view_lease_max` 10 min). Ablauf löst die `Arc<Version>`-Pinung wie `Revoke`; jede Folgenutzung (auch in Retrieval-Deltas) liefert `Err(SnapshotTooOld{stamp, floor_hint})` (`INV-VIEW-3`). Dies schließt unbegrenztes Dateipinning aus, wie es früher durch vergessene `PinGuard`s entstand (`K-46`).

### 11.5 Claims

`claim(files, job)` atomar unter `registry`; Priorität `urgent > flush > compact > blobgc > idle`; höhere Priorität setzt `cancel`, Job prüft zwischen Blöcken (`INV-CLAIM-1`). `retire_gen` verhindert Einchecken veralteter Compaction-Resultate nach Gate.

### 11.6 Race-Nahtstellen (`RACE-1…RACE-22`)

| ID | Race | Lösung | Nachweis |
|---|---|---|---|
| RACE-1 | Lesen × Gate | Gate-Check vor Lesen, `gen`-Check vor und nach Entsiegeln | loom |
| RACE-2 | Flush × Erase | Gate vor Memtable-Einfügung, Flush überspringt `Retired`, ER4 | M2, Sim |
| RACE-3 | Compaction × Erase | Claims, `retire_gen`-Prüfung im Catalog-Edit | M3, Sim |
| RACE-4 | Cache × Schlüsselvernichtung | `gen` vor & nach Entsiegeln, sofortige Invalidierung + `Zeroizing` | loom |
| RACE-5 | Checkpoint × Erase | `Arc<Version>` + `erase_epoch`, serialisiert gegen ER6 | M4, Sim |
| RACE-6 | Consumer × Erase | `retain_floor`, `Lagged`, `Redacted`/`Erased`-Projektion | M2 |
| RACE-7 | Rebucket × Lesen | Packgruppe atomar im selben Edit | M5 |
| RACE-8 | Zwei Erase, überlappende Hülle | Serialisierung in `urgent` nach Ticket-ID | Sim |
| RACE-9 | Restriction × Erase | beides Sequencer-Frames, eine Stamp-Ordnung | M2 |
| RACE-10 | Link × Erase | beide Sequencer-Frames; `policy_epoch`-Abgleich, max. 3 Retries + Selbstberechnung | M2 |
| RACE-11 | Guard × Gate | Zustandsprüfung (6a) vor Guard (6b) | M2 |
| RACE-12 | Anker × Edit | Anker nie vor Edit, Nachzug in OP2a | M6 |
| RACE-13 | Staging × Erase | `purge` behandelt Staging identisch zu Packed | M3, Sim |
| RACE-14 | Space-Drop × Flush/Compaction | Claims, `Dropping` bricht laufende Jobs ab | M2, Sim |
| RACE-15 | Merge × Gate/Restriction | Merge ist Change; Prüfung 6a gilt für Quell-/Zielcluster | M2 |
| RACE-16 | Fork × Compaction | `gc_floor` berücksichtigt lebende `fork_stamp`s | M3 |
| RACE-17 | Branch-Drop × Merge | im Sequencer serialisiert; Merge auf gedropptem Ast ⇒ `BranchClosed` | M2 |
| RACE-18 | Sync-Fehler × Publikation | Publikation strikt in Stamp-Reihenfolge nach Sync; Fehler ⇒ `Indeterminate`/`Poisoned` (`INV-PIPE-2`) | M1, Sim |
| RACE-19 | Leader-Wechsel × Intake | Leader prüft Intake nach Freigabe von Rang 1 erneut | loom |
| RACE-20 | Guard Stufe A × Pipeline-Delta | Konfliktfenster hält sequenzierte, noch nicht durable Stamps; `V1 < Fensterfuß` ⇒ Retry | M1, Sim |
| RACE-21 | Space-Drop × Cluster-Zugang | Space-Gate schließt Writes im selben Frame; Mitgliedschaft ab Gate fest | M2 |
| RACE-22 | Link × Gate (gleiche Quelle) | Link ist Control-Frame; Quelle `Retiring`/`Retired` ⇒ `ClusterRetired` | M2 |

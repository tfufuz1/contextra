# SPEC.md — Contextra Gesamtsystem-Spezifikation (Speicherkern, Retrieval, Engine & Löscharchitektur), Revision R12

| | |
|---|---|
| Status | **Revision R12 (Gesamtsystem-Spezifikation, Repo-abgeglichen), Entwurf zur Freigabe als Gate G-0.** R12 gleicht R11 gegen den geklonten Code ab (HEAD `60ea034`) und löst die dabei gefundenen Widersprüche (K-58…K-63, Anhang B.5). Führt die Gesamtsystemsicht von R10 (§0–§3, K-39…K-47) mit dem vollständigen Speicherkern-Wortlaut von R9.1 (§4–§11) zusammen und schließt die in R10 nie geschriebenen Kapitel §12–§19 samt Anhängen, einschließlich des Vertrags zu `contextra-vector`, `-text`, `-graph`, `-rank`, `-engine`, `-checkpoint`. Konsolidiert R7, Lethe-Architekturentwurf, Lethe-Engine-Spec, Mnemos, „Optimal-LLM-Datenbank“ und Octa-Abgleich. |
| Gegenstand | Eingebettetes, crash-sicheres Gesamtsystem für europäische Unternehmens-LLM- und Agenten-Workloads: synchroner Speicherkern (`contextra-storage`, ersetzt `contextra-store`) mit atomaren Mehr-Space-Änderungen, MVCC-Sichten, clusterlokalen Branches/Merges; mehrkanaliges Hybrid-Retrieval (Vektor, BM25/WAND, Wissensgraph, kalibrierte Fusion) über löschbare Index-Partitionen; Agenten-Langzeitgedächtnis; **durchgängig nachweisbare Löschung personenbezogener Daten** über Quell- und Indexschichten (logisch, kryptografisch, physisch) mit Trennung personenbezogener und sonstiger Daten. |
| Verifikation | **Ist-Stand (Repository HEAD `60ea034`, 2026-11-24):** Der Kern hat Teilchecks und Strukturtests (`crates/contextra-storage/tests/`, 114 Quelldateien, ≈ 14 kLOC), **kompiliert aber nicht**, weil `contextra-storage-ports` weder als Crate existiert noch in `crates/contextra-storage/Cargo.toml` eingetragen ist (§18.1 Nr. 1). Die Retrieval-Crates (`vector`, `text`, `graph`, `rank`, `engine`, `checkpoint`, `crypto`, `mvcc`) enthalten Alt-Pfade (`memmap2`, `std::fs`, `bincode`, `blake3`, `aes-gcm-siv`, 4-Wege-2PC) und sind noch nicht auf `contextra-storage` geschnitten; gemessen (Dateien mit Treffer): `memmap2` 17 in `vector`, `bincode` in 5 Crates, `blake3` im Quelltext von 6 Crates, `block_on` in 6 Crates, `DISKANN_INTEGRITY_KEY` 4 Dateien (§14.1, §18.1). Vollständige Release-Gates (TLA+ M1–M6, Fuzzing, Referenzvektoren, Voll-Orakel) sind offen. **Keine Leistungszahl (H-1…H-35) gilt vor Gate-Abnahme als gemessen.** |
| Rechtlicher Vorbehalt | Das System liefert technische Eigenschaften (atomare Sichtbarkeitsbarriere, Schlüsselvernichtung, physische Dateibereinigung, Löschprotokolle, Trennung personenbezogener Daten), keine Rechtsaussage zu Art. 17 DSGVO, Fristen, Ausnahmen oder zur Einstufung pseudonymisierter Daten als anonym. „Gelöscht“ ist immer an eine nachgewiesene Stufe aus §9.2 und ein konkretes `ErasureReceipt` gebunden. |
| Normative Quelle | **Diese Datei ist die einzige normative Gesamtsystem-Fassung.** Modul-Specs unter `crates/*/SPEC.md`, `capabilities.toml` und `CLAUDE.md` verweisen hierher oder sind vor G-0 anzugleichen (§18.1). Rangfolge bei Widerspruch: **§15 (Invarianten) vor §12.8 (Audit-Ergänzungen) vor §12 (Erweiterungs-/Retrieval-Integration) vor den übrigen Abschnitten.** |
| Sprache | Prosa deutsch, Bezeichner englisch. |
| R11-Audit | Anhang B.4: Wiederherstellung der in R10 gekürzten Normtexte, Auflösung der Widersprüche zwischen Klasse-A-Autarkie, Ebenenregel und Index-Bindung, Ingest-Atomarität, löschbare Index-Partitionen, Trennungsprinzip, Query-Plan-Protokoll. Alle neuen Leistungsaussagen sind **Entwurfsziele** (H-29…H-35). |

---

## 0. Kurzfassung

**Contextra ist eine europäische Datenbank- und Gedächtnisplattform für produktive Unternehmens-LLM-Systeme und autonome KI-Agenten, in deren Speicher- und Indexfundament permanenter Datenschutz und belegbare Löschbarkeit (Art. 17 DSGVO) von Anfang an verankert sind.**

Während die Vorgänger-Architektur auf einem generischen LSM-Tree (`contextra-store`) plus lose gekoppelter Zwei-Phasen-Kompensation (`DbTransaction` + `CommitLedger`) über separate Vektor-, Volltext- und Graph-Indizes aufbaute, basiert Contextra ab R11 auf dem vollständig neu entwickelten Speicherkern **`contextra-storage`**:

1. **Ein Fundament für Dokumente, Vektoren, Text-Metadaten, Graph-Fakten und Agenten-Zustände:** Ein fachlicher Schreibvorgang (`Change`) schreibt kanonische Dokumentdaten, quantisierte/kompakte Vektorcodes, Vorwärts-Termdaten, Graph-Fakten und Bitemporalitäts-Kopfdaten **in einem einzigen Redo-Frame mit genau einem Gruppen-Fsync** atomar in die Spaces eines Clusters; Dokumente, die das Change-Limit überschreiten, werden über einen **Publish-Record** (ein abschließender Change setzt `head`) ebenso atomar sichtbar, ohne Zweiphasenprotokoll (§12.4.0). Verteilte Kompensations-Transaktionen zwischen Storage und Indizes entfallen vollständig.
2. **Ein Cluster ist die universelle Lebens-, Versions-, Schlüssel- und Löschdomäne:** Alles zu einer Person, einem Mandanten-Fall oder einem Arbeitskontext liegt in einem `Cluster` (`ClusterId`, 128-Bit-Zufall). Der Cluster ist zugleich **Löschdomäne, Schlüsseldomäne, Versionsdomäne (`BranchId`), und physische Platzierungseinheit**. Branches für Agenten-Experimente sind clusterlokale O(1)-Forks und sterben automatisch mit ihrem Cluster. **Geteilte Indizes sind in Partitionen entlang der Packgruppen zerlegt**, jede Generation mit eigenem Schlüssel: Löschen kostet höchstens den Neuaufbau betroffener Partitionen, danach macht die Schlüsselvernichtung alte Kopien unlesbar (§12.4.5). **Personenbezogene Inhalte liegen nur in Personen-/Fallclustern; der gemeinsame Korpus enthält pseudonymisierte Chunks, die Zuordnung liegt im Pseudonym-Tresor der Person** (§12.4.6).
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

---

## 1. Konfliktregister: Entscheidungen der Konsolidierung

Die Konsolidierung vereint die Speicherkern-Entscheidungen (`K-01`…`K-38`), die strukturellen Konflikte zwischen dem bisherigen Gesamtsystem (`engine`/`vector`/`text`/`graph`/`rank`/`checkpoint`/`mvcc`/`crypto`) und dem neuen Speicherkern (`K-39`…`K-47`) sowie die Entscheidungen der Gesamtsystem-Auditrunde R11 (`K-48`…`K-57`). Jede Entscheidung ist final; die Spalte „Wirkung“ nennt den umsetzenden Abschnitt.

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
| **K-48** | **Klasse-A-Autarkie der Retrieval-Crates (`vector`/`text`/`graph`/`rank`: „0 interne Abhängigkeiten“, B1) vs. verbindliche `SegmentOwner`-/`ConsumerFeed`-Bindung an den Speicherkern; `storage-ports` (L1) und `storage` (L1) auf derselben Ebene** | **Persistenzneutrale Schnittstellen statt Kernabhängigkeit:** Jedes Retrieval-Crate definiert selbst schmale Traits (`PartitionSink`/`PartitionSource`: blockweises Schreiben/Lesen opaker, bereits adressierter Byte-Blöcke) und kennt weder `contextra-storage` noch `SegmentIo`. Die Bindung (`SegmentOwner`, `ConsumerFeed`, Watermark, Attest) lebt in einem neuen Level-2-Crate **`contextra-index-bridge`**. `contextra-storage-ports` wird Vertragsebene (Ebenennummern: K-58) (nur Traits/Werttypen, keine Abhängigkeiten). Regel: Abhängigkeiten zeigen auf **strikt niedrigere** Ebenen, innerhalb einer Ebene sind keine internen Kanten erlaubt | Klasse-A-Autarkie (B1/B5) bleibt erhalten, der Kern bleibt ohne Retrieval-Wissen, die Ring-Matrix wird wieder prüfbar. Heute verletzt `vector → mvcc` (L1→L1) die Regel; `mvcc` entfällt als Abhängigkeit (K-40) | §3.1, §12.1, §18.1 |
| **K-49** | **Granularität abgeleiteter Indizes:** globaler Index je Mandant (schnell, aber Löschen = Vollrebuild) vs. Index je Cluster (löschbar, aber 10⁷ Mikroindizes, kein clusterübergreifendes Retrieval) | **Zwei Klassen, eine Regel:** (a) clusterlokale Mikroindizes für Cluster ≤ `micro_index_max_vectors` (flacher Scan/Mini-HNSW, aus den Records ableitbar); (b) **Index-Partition = Packgruppe** `(tenant_tag, due_bucket, shard)` mit hartem Größenlimit und **Generationsschlüssel je Partitionsgeneration**. Löschen = Tombstone sofort, Neuaufbau der betroffenen Partition fristgebunden und amortisiert, danach Vernichtung des Generationsschlüssels | Rebuild-Kosten sind nach oben beschränkt (Partitionsgröße), Ablauf einer Packgruppe löscht ihre Index-Partitionen in O(Dateien), Backups alter Generationen werden durch Schlüsselvernichtung unlesbar (Crypto-Erasure auch für geteilte Indexstrukturen) | §12.4.5 |
| **K-50** | **Trennung personenbezogener und sonstiger Daten:** „Klassifikation ist Sache der oberen Schicht“ ließ offen, wie ein Dokument mit Personenbezug in einem gemeinsamen Korpus löschbar bleibt | **Trennungsprinzip (`INV-SEP-1`):** Personenbezogene Inhalte liegen ausschließlich in `Sealed` Personen-/Fallclustern; im gemeinsamen Korpus stehen nur pseudonymisierte Chunks, die Zuordnung Pseudonym→Person liegt im Cluster der Person (**Pseudonym-Tresor**). Embeddings entstehen **nach** der Pseudonymisierung. Löschung der Person vernichtet den Tresor; ob der Rest danach anonym ist, bleibt eine Einzelfallbewertung (kein Kernversprechen) | Ein Cluster-Löschauftrag genügt für alle Namensbezüge; Vektoren kodieren keine Klarnamen; das Restrisiko (Re-Identifikation über Kontext) wird im Receipt ausgewiesen | §12.4.6 |
| **K-51** | **Atomarität großer Dokumente:** „ein Change = ein Dokument“ scheitert an `max_change_ops` (4096) / `max_change_bytes` (8 MiB); ein Rückfall auf Pseudo-2PC wäre ein Rückschritt (K-39) | **Publish-Record:** Chunks einer Dokumentversion werden in beliebig vielen Changes unter einem versionierten Präfix gestaged; **ein** abschließender Change setzt `doc/<id>/head` (Guard `Version`). Lesen und Indexprojektion lösen ausschließlich über `head` auf. Kleine Dokumente nutzen den Ein-Change-Pfad. Verwaiste Stagings räumt ein Worker | Dokumentweite Atomarität ohne Zweiphasenprotokoll, ohne Kompensation, crash-sicher, retrybar über `IdemToken` | §12.4.0 |
| **K-52** | **Read-your-writes für Agenten** bei asynchroner Indexprojektion | `Receipt.stamp` + `QueryOptions::min_stamp`: die Abfrage wartet höchstens `ryw_wait` auf `watermark ≥ min_stamp`, sonst deckt der **Delta-Pfad** (Brute-Force über gestagte Records bis `V`) die Lücke; nie ein stilles Auslassen | Agent sieht sein eigenes Write sofort, ohne synchrone Indexpflege im Commit-Pfad | §12.4.4 |
| **K-53** | **Reproduzierbarkeit/Erklärbarkeit** der Retrieval-Ergebnisse bei wechselnden Indexgenerationen | **Query-Plan-Protokoll mit gepinntem `SnapshotSet`:** `(instance_id, V)` + je Kanal `(partition, generation, applied_stamp)`; jedes Ergebnis trägt dieses Set. Gleiches Set ⇒ bitgleiches Ergebnis (`INV-RAG-4`) | Audit („warum wurde das geliefert?“), Differentialtests, Regressionsanalyse | §12.4.4 |
| **K-54** | **Lernsignale** (`edge_reinforcement`, Klick-/Feedback-Daten, Kalibratoren, `DriftDetector`) sind Verhaltensdaten und damit potenziell personenbezogen | Nutzerbezogene Signale liegen clusterlokal im Cluster des Nutzers (Löschung inklusive). Globale Kalibratoren und Gewichte werden nur aus **nicht personenbezogenen Score-Aggregaten** trainiert (Rohscore, Kanal, Label ohne Nutzer-/Query-Bezug); Trainingsdaten haben `NonPersonal`-Klasse und eine Mindestgruppengröße `k_min` | Vergessen entfernt auch gelernte Personalisierung; die Fusionsgüte bleibt global lernbar | §12.4.4, §12.9 |
| **K-55** | **Ist-Drift der Compliance-Crates:** `CLAUDE.md` nennt `contextra-erasure` (`frozen`) und `contextra-vault`; beide fehlen im Baum und in `capabilities.toml`, `vault` steht dort nicht | Die Abbildung `ErasureReceipt` + `EraseAttestation` → signierter `DeletionProof` ist Vertrag (§9.7, §12.8 Nr. 10) und wird im Bridge-Layer spezifiziert, **nicht** durch Änderung eines eingefrorenen Crates. Änderungen an `contextra-erasure` nur nach ausdrücklicher Freigabe; bis dahin ist die Signaturstufe optional und das Receipt als solches gültig | Spec und Repo-Wahrheit stimmen überein; kein stilles Anfassen eingefrorener Compliance-Software | §3.7.4, §18.1 |
| **K-56** | **Zu viele Konfigurationsschrauben** (Anker, Ledger-Gerät, Provider, Horizont, Fristen) gefährden Entwicklerfreundlichkeit und Betriebssicherheit | **Compliance-Profile als geprüfte Bündel** (`Development`, `Standard`, `Strict`) mit Zusatzprüfung beim Öffnen; Einzelschrauben bleiben, jede Abweichung vom Profil erscheint im Receipt | Sichere Voreinstellung in einem Wort; Auditoren sehen das Profil | §12.10, §16 |
| **K-57** | **Auskunft/Portabilität (Art. 15, 20 DSGVO)** waren nicht abgedeckt | Der Cluster als Löschdomäne ist zugleich die **Exportdomäne**: `export_cluster(view)` liefert einen konsistenten, inhaltsvollständigen Strom aller Records einer Sicht (ohne Schlüssel, ohne Indexstrukturen); die Aufbereitung (Format, Rechtsprüfung) liegt oben | Auskunft = ein Scan statt Suche über alle Systeme | §12.10 |

| **K-58** | **Ebenenmodell (ersetzt die Ebenennummern aus K-48):** K-48 setzte `storage-ports` auf Level 0 und ließ Retrieval-Crates „ohne interne Abhängigkeit“. Gemessen importieren `vector` (38×), `text`, `graph` direkt `contextra-types`/`contextra-ports`, und `capabilities.toml` führt `types`/`ports` auf Level 2, also über den Konsumenten | **Fünf Ebenen:** L0 Fundament (`ids`, `env`, `simd`, `sys`, `wire`, `durable-fs`, **`types`**), L1 Verträge (**`ports`**, `storage-ports`), L2 Triebwerke (`storage`, `vector`, `text`, `graph`, `rank`, `mvcc`*, `checkpoint`, `kvcache`, `crypto`), L3 Adapter und Orchestrierung (`engine`, `index-bridge`, `storage-bridge`, `storage-fs`, `crypto-ref`, `keys-*`, `erasure`, `agent`, `cognition`, `router`), L4 Fassade und Apps (`db`, `contextra`, `mcp`, `py`, `testkit`, `bench`). Regel unverändert: nur strikt abwärts. Retrieval-Crates dürfen **nur** `types` und `ports` kennen, nie `storage`, `storage-ports` oder einander. (*`mvcc` entfällt für Kernpfade, K-40) | Die Kanten existieren bereits real; sie zu verbieten hieße 3 Crates umschreiben, sie auf die richtige Ebene zu legen kostet eine Zeile in `capabilities.toml`. Isolation von Storage bleibt voll erhalten | §3.1, §18.1 |
| **K-59** | **Fünf Kopien von `TxId` und eine zweite `Epoch`:** `TxId(pub u64)` steht in `types`, `mvcc`, `vector`, `graph`, `checkpoint`; `text` führt `contract::Epoch` | **Ein** `Epoch(u64)` in `contextra-types`; `TxId` und alle Kopien entfallen, `text::contract::Epoch` wird Re-Export. `Stamp ↔ Epoch` wird nur in `index-bridge` konvertiert (Identität, zusätzlich `instance_id`) | Ein Versionsbegriff (L23); Kopien driften | §12.4, §14.1 |
| **K-60** | **Lücke Personenbezug × geteilter Index:** §12.4.6 legt Personenbezug in Einzel-Cluster, §12.4.5 teilt Partitionen über Cluster. Unklar blieb, ob Klartext-Embeddings von Personalakten in einer geteilten Partition liegen dürfen, und wie Suche über viele Personen löschbar bleibt | **Zwei Indexklassen, eine Regel:** `Corpus`-Partitionen und `Personal`-Partitionen. *Geteilte* Partitionen (beider Klassen) enthalten ausschließlich aus **pseudonymisiertem** Text abgeleitete Vektoren, Terme und Kanten (`INV-SEP-2`). Klartext-Embeddings entstehen nur im Cluster-Mikroindex (Modus A) des Personenclusters und sterben mit dem Clusterschlüssel. `Personal`-Partitionen haben kleinere Limits und `Rebuildable{max_age}` kürzer als `purge_deadline` | Suche über Personen bleibt möglich (pseudonymisiert), Löschung einer Person kostet höchstens den Neuaufbau ihrer Personal-Partitionen, kein Klartext im geteilten Index | §12.4.5, §12.4.6 |
| **K-61** | **`kvcache` bricht Löschvertrag:** Feature `content-addressed-kv-cache` (Inhaltsadressierung), `blake3`, `bincode`, `aes-gcm-siv` über `crypto/kv-encryption` | Feature entfällt vor G-R5 (NZ-11, `INV-PRIV-1`); `kvcache` wird `SegmentOwner` (`MixedPerClusterKey`) auf `storage-ports`-Krypto; `contextra-crypto/kv-encryption` entfällt | Prefix-Dedup über Löschdomänen ist ein Querverweis | §12.9, §14.1 |
| **K-62** | **Mmap und `block_on` im Bestand:** `memmap2` in `vector` (17 Dateien) und `contextra-sys`; `block_on` in `text`, `graph`, `engine`, `checkpoint`, `kvcache`, `crypto` | `memmap2` nur in `contextra-sys` hinter Feature `mmap`, nur als `Fs`-Implementierung ohne Index-/Sealed-Pfad; `block_on` nur im Engine-Rand (L3+), nie in Level-2-Crates | K-11, Z-19 | §14.1 |
| **K-63** | **`contextra-ports` trägt Alt-Verträge (`StorageEngine`, `StorageBackend`, `Vfs`, `Purger`, `LedgerSink`, `CommitBarrier`, `ErasureLayer`)** mit 2PC- und Async-Prägung | Bleiben bis G-CUTOVER als Kompatibilitätsschicht, werden durch `storage-bridge` bedient; neue Verträge (`SegmentOwner`, `PartitionSink`) kommen **nicht** in `ports`, sondern je Crate bzw. `storage-ports` | Ein Contract-Test-Satz (`conformance.rs`) existiert bereits und schützt die Migration | §3.2, §14.1 |


### 1.1 Herkunft der übernommenen Elemente

| Quelle | Übernommen | Nicht übernommen / Abgelöst |
|---|---|---|
| **R7 (`contextra-storage` Basis)** | Gesamtgerüst: Cluster, Sealed/Plain, Pure/Packed/Staging, Sequencer+Syncer, Catalog-Edits, ER0–ER7, Anker, Notbetrieb, Absturzmatrix, Invarianten, TLA+-Gates | `rekey_clone`, Aufrufer-ClusterId |
| **Lethe-Entwurf & Lethe-Spec** | G0/G1/G2-Stufen, `Redacted`/`Erased` im Change-Feed, `tomb_id`, `DerivedSegment::on_gate/verify_absent`, Person→Wurzelcluster-Muster, Orakel O-Files/…/O-Segments, `ErasureReceipt` & `verify_receipt`, `KeyUnavailable`/`KeyLost`, Vier-Augen-Lift | eigene Redo-/WAL-Struktur, Cohort als Kernlogik, `eu_tag` |
| **Mnemos & Engram** | Zoom/Handles/Trust/Derivation als Konventionen oben, Fork-TTL, token-budgetiertes `page()`/`weights`, `Error::hint()`, Evaluationsmethodik `Utility@Budget` | Prolly-Tree, Content-Addressing/Merkle-DAG, KV-Tensor-Layouts im Kern |
| **CairnDB & AetherStore** | **Index-Watermarks** (`applied_position`) als Pflichtmuster, Guard-Katalog, 3 Benchmark-Gruppen, WiscKey-Wertetrennung, Mandanten-Vorfilterung vor ANN, Typ-Alignment optional | CAS ohne WAL, GPUDirect, `mmap` im Lesepfad, Slab-Allokator |
| **Octa-Spec & R9/R9.1-Audit** | Tomb-Epochen/Backup-Horizont (A3), `HostProbe`, `MeasureKind`, `View-Lease`, `Poisoned`-Ledger-Intent, Pipeline ohne E/A unter Rang 1, branch-bewusster Filter, `ClusterIndex ≤ 40B`, `MiniSection`, Spill-Policy, `deps-gate`/`deny.toml` | Zeitlanes/Kumulativsummen im SubIndex (A1/A2 → `BlockPruner` & RAM-Präfixsummen) |
| **Contextra Bestands-Crates (HEAD `60ea034`)** | `contextra-simd` (AVX-512/AVX2/NEON Runtime-Dispatch + NaN-Guard), `contextra-vector` (HNSW, DiskANN-Algorithmik, SQ8+Sq8Bias, RaBitQ, ACORN $\gamma$-Augmentierung, Partial-Rebuild `TraversalTracker`, `GhostFreeVectorIndex`), `contextra-text` (BM25/BM25F, `GermanMorphTokenizer`/`GermanCompoundSplitter`, Block-Max WAND), `contextra-graph` (CSR, n-äre Hyperkanten, PPR/APPRH/TL-HFD/`PathRAG`, Community/Percolation, `SessionBranchTree`), `contextra-rank` (4-/5-Signal RRF/mRRF/Resonance, PAVA/Platt/Conformal, Drift, `DiBud`, `explain`), `contextra-engine` (`TenantPolicy`, `Collection`, `HybridQueryBuilder`, `AdaptiveDecayController`), `contextra-ingest`/`privacy`/`prompt-guard`/`sandbox`/`router`/`kvcache` | `contextra-store` (Alt-LSM), `DbTransaction`/`CommitLedger` 4-Wege-2PC, `contextra-durable-fs` als `RealFs`, unversiegeltes DiskANN-`memmap2` mit Literal-Key, `blake3`/`aes-gcm-siv`/`bincode` in Persistenz, doppeltes MVCC in Produktionspfaden |

---

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
| **L25** | **Persistenzneutrale Retrieval-Crates, Index-Partition = Packgruppe, Trennungsprinzip, Publish-Record** (K-48…K-51): Retrieval-Crates kennen den Kern nicht; geteilte Indizes sind löschbar partitioniert; personenbezogener Klartext liegt nur in `Sealed` Clustern; große Dokumente werden über einen abschließenden `head`-Change atomar sichtbar. |

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

---

## 3. Gesamtsystem-Architektur, Crate-Topologie & Abhängigkeits-Governance

### 3.1 Ring-/Ebenenmodell & Crate-DAG

Abhängigkeiten verlaufen **gerichtet azyklisch und nur auf strikt niedrigere Ebenen** (`Level 3 → Level 2 → Level 1 → Level 0`); innerhalb einer Ebene gibt es keine internen Kanten (K-48, `INV-LAYER-1`). Jede Verletzung bricht das CI-Gate `layering` und `cargo tree`.

```text
Level 4 (Fassade & Apps):   contextra (Produkt) | contextra-db | contextra-mcp | contextra-py | contextra-testkit | contextra-bench
        │
Level 3 (Orchestrierung & Adapter / Composition Root):
        contextra-engine | contextra-agent | contextra-cognition | contextra-router | contextra-erasure*
        contextra-index-bridge (SegmentOwner, ConsumerFeed, Watermark) | contextra-storage-bridge (Async-Fassade, Receipt→Proof)
        contextra-storage-fs | contextra-crypto-ref | contextra-keys-file | contextra-keys-cohort
        │
Level 2 (Triebwerke, untereinander ohne Kanten):
        contextra-storage (synchroner Kern, forbid(unsafe))
        contextra-vector | contextra-text | contextra-graph | contextra-rank | contextra-checkpoint | contextra-kvcache | contextra-crypto (Übergang)
        contextra-mvcc (Übergang, entfällt für Kernpfade) | contextra-store (Legacy, bis G-CUTOVER)
        │
Level 1 (Verträge, nur Traits/Werttypen):   contextra-ports | contextra-storage-ports
        │
Level 0 (Fundament):   contextra-types | contextra-ids | contextra-env
        Unsafe-Inseln: contextra-simd | contextra-sys | contextra-wire      (contextra-durable-fs: experimentell, kein RealFs, SD-70)
        Safety (stand-alone): contextra-privacy | contextra-sandbox | contextra-prompt-guard | contextra-ingest | contextra-adapt | contextra-infer-*
```
(* `contextra-erasure` ist `frozen`, Änderung nur nach Freigabe, K-55.)

**Ebenenregel (K-58).** `contextra-storage` kennt nur `storage-ports`. `vector`/`text`/`graph`/`rank` kennen nur `types` und `ports` (so wie heute im Code). Zusammenführung von Speicher und Retrieval geschieht ausschließlich in `index-bridge` und `engine`.

**Bindung statt Abhängigkeit (K-48).** `contextra-index-bridge` (Level 3) implementiert `SegmentOwner`, `ConsumerFeed`, Watermark und Attest für `vector`/`text`/`graph`/Identity-Directory über die je Crate definierten Traits `PartitionSink`/`PartitionSource`; die Retrieval-Crates kennen `SegmentIo` nicht.

**Modulschnitt innerhalb von `contextra-storage`** (Abhängigkeiten nur nach unten, Test `layering`):

```text
lib.rs (Fassade) · engine (committer/sequencer+syncer, memtable, version, views, branches, registry,
                           extension, feed, segments, page, cursor, cache, idem, guard_window, graveyard, volatile)
maint (flush, compaction, blob_gc, purge, scheduler, scrub) · erase (gate, cluster_table, links, protocol, ledger)
recovery · checkpoint (create, restore) · catalog · store (run, blob, redo) · keys · codec (varint, tags, checksums, comparator, seal, format, header)
env (Fs, Clock, Entropy, Governor, Metrics, Anchor; MemFs, FaultFs, Sim) · sync (Ränge 1–8) · testing (ErasureOracle, cfg(feature="testing"))
```

**Ist-vs-Soll-Abgleich Workspace (HEAD `60ea034`, per Datei geprüft).** `Cargo.toml` führt 14 Mitglieder; zusätzlich liegt `contextra-kvcache` im Verzeichnis `crates/`, ist aber **nicht** Mitglied. `capabilities.toml` beschreibt 42 Crates, davon existieren 14 (u. a. fehlen `ids`, `env`, `wire`, `store`, `storage-ports`, `storage-bridge`, `storage-fs`, `crypto-ref`, `keys-*`, `index-bridge`, `db`, `privacy`, `sandbox`, `ingest`, `router`, `agent`). Der Kern importiert `contextra_storage_ports` in `catalog/*.rs`, hat aber weder Crate noch Dependency-Eintrag. `contextra-types` und `contextra-ports` stehen in `capabilities.toml` auf Level 2, werden aber von `vector`, `text`, `graph`, `crypto`, `kvcache` genutzt (K-58). Es gibt kein Root-`tests/`-Verzeichnis (`CLAUDE.md` nennt `tests/layering.rs` und `tests/unsafe_islands.rs`); eine eigene `layering.rs` liegt nur in `crates/contextra-storage/tests/`. `docs/agent/` (von `CLAUDE.md` referenziert) fehlt. `scripts/check_forbidden_apis.sh` prüft bereits `storage` und `storage-ports`, aber nicht die Retrieval-Crates. Bereinigung ist Pflichtbestandteil von **P0** (§14.1, §18.1).

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
| `PartitionSink` / `PartitionSource` | Je Retrieval-Crate definierte, persistenzneutrale Traits (Block-Schreiben/-Lesen einer Partitionsgeneration); implementiert in `contextra-index-bridge` über `SegmentIo` (K-48) |
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

---

## 4. Datenmodell

### 4.1 Vokabular

| Begriff | Bedeutung |
|---|---|
| **Space** | benannter, physisch getrennter LSM-Baum (`SpaceId u64`), monoton und dauerhaft reserviert, nie wiederverwendet; `Durable` oder `Volatile` |
| **Cluster** | Löschdomäne/Schlüsseldomäne/Versionsdomäne; `Sealed` oder `Plain` |
| **Branch** | clusterlokaler Versionszweig (`BranchId u32`, `0 = main`) |
| **Change / Guard / Receipt** | atomare Menge von Ops / Vorbedingung / Quittung `(stamp, durable)` |
| **Stamp** | monotone `u64` (< 2^56) = Transaktionszeit = Commit-Reihenfolge |
| **View / Cursor / Page** | unveränderliche Sicht / Streaming-Iterator / budgetierte Seite |
| **Hold** | befristete **technische** Halteregel gegen Verwerfen alter Versionen |
| **Restriction** | **rechtlicher** Sperrvermerk mit Pflichtgrund |
| **Consumer** | persistenter Fortschrittszeiger im Redo-Feed |
| **Segment** | vom Catalog verwaltete Datei einer oberen Schicht |
| **Packgruppe** | `(tenant_tag, due_bucket, shard)` |
| **tomb_id** | `HMAC(tomb_key(tomb_epoch), "ctxs cluster tomb v1" || instance_id[16] || ClusterId[16])`; instanzgebundenes, linkbares Pseudonym ohne Klartext-ID. Solange die Epoche nicht retired ist, bleibt es linkbar (K-31, §10.4) |
| **tomb_epoch** | `u32` je Tomb-tragendem Ledger-Event (K-31); Zustände `Open → Closed → Retired`; der zugehörige `tomb_key` liegt im `KeyProvider`, nie in `KEYS`, Checkpoints oder Backups (`INV-KEY-5`). Default: genau Epoche `0` |
| **View-Lease** | Frist, nach der eine `View`/ein `Cursor` ihre Version nicht mehr pinnen (`view_lease`, §11.4); danach `SnapshotTooOld` |

Space-IDs `0..=255` sind für den in §5.9 versionierten System-Space-Namensraum reserviert; der persistente Benutzer-ID-Allocator beginnt bei `256`. `reserve_space_ids` schreibt den nächsten High-Water-Mark vor Vergabe eines Blocks durabel; nach Crash werden ungenutzte IDs übersprungen. Exhaustion liefert `Exhausted`, niemals Wrap oder Wiederverwendung.

### 4.2 Reihenfolge, Tag, Marker

`cmp`: Cluster aufsteigend, **Branch aufsteigend**, Nutzerschlüssel bytelexikografisch, Tag absteigend (kein `memcmp` über Konkatenation). Ohne `clustered` ist der Cluster implizit `NONE`.
Tag `u64 = (stamp << 8) | kind`; `Put=1, Delete=2, RangeDelete=3, EraseRange=4, PutRef=5, Merge=6 (reserviert, Decoder: Unsupported)`. `PutRef` nur in Runs. `RangeDelete` ist branch-lokal mit Marker `(cluster, branch, start, end, tag)`, `end` exklusiv. `EraseRange` ist dagegen ein clusterweiter Control-Marker ohne Branch und wirkt auf alle Branches und historischen Views; er ist nie ein gewöhnlicher Merge-Wert. Für Sealed-Cluster bleiben nur Markerart, Cluster-Routing und Stamp im Klartext; `start/end` sind cluster-key-versiegelt. Der Committer hält das Gate-Maskenintervall im RAM, und Recovery entschlüsselt es vor Freigabe des Clusters. Bei zerstörtem Clusterschlüssel wird der Cluster insgesamt gesperrt/retired, nicht versucht, das Intervall selektiv zu rekonstruieren.
Stamps: erster = 1, streng monoton über angenommene Changes und Frames, nie doppelt vergeben (auch nach Absturz), Überlauf ⇒ `Exhausted`. Lücken zulässig.

### 4.3 Cluster

- **Erzeugung:** `Clusters::register(ClusterSpec)` zieht 128 Bit aus `Entropy`, prüft Kollision, legt bei `Sealed` **zuerst** den Schlüssel an (`create_cluster_keys`, Batch, idempotent je `BatchId`, durabel), dann Catalog-Edit `cluster_register`. Ein Schlüssel ohne Cluster (Absturz dazwischen) wird von `idle` nach `orphan_key_grace` (24 h) über `KeyProvider::list` (liefert je Schlüssel `created_utc`) vernichtet.
- **Zustände:** `Live → Retired(S) → [Shredded] → Purged → (Trim)`. Zwischen `Live` und `Retired` existiert nur im Sequencer ein flüchtiges `Retiring(S)` (§6.1): es schließt die Schreibadmission, ist für Leser weder sichtbar noch persistiert. `Shredded` nur bei `Sealed`. Nur vorwärts.
- **Trim:** Ein `Purged`-Eintrag wird per `cluster_trim` entfernt, sobald `history_floor > S` (keine View/Hold/Consumer-Fenster darunter). Danach `UnknownCluster`. Wiederverwendung ausgeschlossen (128-Bit-Zufall). Der Dauer-Nachweis liegt als `tomb_id` im Ledger (§9.7).
- **Restrictions** sind eine Menge am Cluster, kein Zustand.
- `ClusterSpec` ist nach `register` unveränderlich und hat die Felder `protection` (`Sealed` | `Plain`), `tenant_tag` (16 Byte, opak, vom Aufrufer; bestimmt mit `due` die Packgruppe, §4.1), `due` (optionale Fälligkeit; `None` ⇒ kein Ablauf), `erase_locality` und `expected_bytes` (Größenklasse, ausschließlich Admission-Hinweis). v1-Default ist `PackedBounded{max_file_bytes: 64 MiB}`. `ClusterLocal` bedeutet: Run-/Blob-Dateien dieses Clusters enthalten **ausschließlich** seine Abschnitte (Pure); gepackte Sections erfüllen die Option nicht. Sie ist nur nach Admission (§4.3.1) wählbar (`expected_bytes ≥ cluster_local_min_bytes`, Cluster-Quote, Datei-Quote) und wird nie still herabgestuft; `ClusterLocal`-Cluster sind von der Pure-Demotion (R-PL-3) ausgenommen, die Quote ist daher eine harte Obergrenze der durch sie erzeugten Dateien. Beide Modi unterliegen den separaten Redo-/Pin-Kosten aus INV-ERA-11.

#### 4.3.1 Admission für Cluster, Dateien und Metadaten (K-35, K-37)

Ein Cluster kostet RAM, Dateien und Dateideskriptoren. Jede dieser Ressourcen unterliegt einer Admission mit eigener Fehlerklasse, geprüft **vor** der Allokation (`INV-ADM-1`). Es ist technischer Ressourcenschutz, keine Politik (§3.3).

**Residenz der Metadaten**

| Struktur | Residenz | Kosten | `MemoryBudget`-Klasse |
|---|---|---|---|
| `ClusterIndex` (offene Adressierung, Schlüssel = `ClusterId`) | immer resident | 36 B je Eintrag: id 16 · `state:8 \| gen:56` 8 · `pack_group` 4 · `hot_ref` 4 · `flags` 4; Rechenwert 40 B je Cluster inkl. Tabellen-Slack (Annahme: Füllung ≈ 0,9, in H-25 zu belegen) | `ClusterMeta` |
| `HotRecord` (Branch-Tabelle, Restrictions, Links, Pins, offene Views, Cursor-Zustand) | nur für Cluster mit solchen Merkmalen; kaltstellbar (Neuaufbau aus Redo/`SYS_BRANCH`), **nicht** kaltstellbar bei Restriction, Link oder Pin | ≤ 256 B + Inhalt | `ClusterHot` |
| `ClusterDirectory` je Datei | nur Sparse-Index (eine Zeile je 4-KiB-Seite, ≈ 0,3 B je Cluster und Datei) und `ClusterFilter` (≈ 1 B je Cluster); Seiten über den Block-Cache | ≈ 1,3 B je Cluster und Datei | `DirIndex`, `BlockCache` |

Rechenbeispiel: 16 Mio. Cluster × 40 B ≈ 610 MiB `ClusterIndex`, unabhängig von der Zahl der Dateien (ein Ansatz mit 48 B je Cluster **und Datei** plus 128 B je Cluster käme bei vier Dateien je Cluster auf ≈ 5 GiB). Kapazität: `resident_clusters_max = cluster_meta_budget / 40 B` mit `cluster_meta_budget` = 15 % von `memory_budget_bytes` (§16). Der Gate-Check bleibt O(1) und ohne E/A.

**Admission-Regeln**

1. **Cluster-Metadaten:** `register` reserviert `ClusterMeta` (40 B) beim `MemoryBudget`; Fehlschlag ⇒ `Err(Limit{ClusterMetaBudget})` ohne Teilzustand. `open` vergleicht die Clusterzahl des Catalogs mit dem Budget und öffnet andernfalls `ReadOnly` mit Diagnose statt in den OOM zu laufen.
2. **`ClusterLocal`:** nur wenn (a) `expected_bytes ≥ cluster_local_min_bytes` (Standard = `pure_threshold`, 8 MiB), (b) die Zahl lebender `ClusterLocal`-Cluster `< max_cluster_local_clusters` (Standard 1024, ≤ 65 536) und (c) `max_live_files` Reserve für mindestens zwei Dateien je Level lässt. Sonst `Err(Limit{ClusterLocalAdmission})`. Zweck der Option: `Plain`-Cluster und große `Sealed`-Cluster mit physischem Löschzeitziel. Für Massen personenbezogener Cluster ist **PackedBounded + Crypto-Erase** der vorgesehene Weg: `KeysDestroyed` wirkt sofort, die Hygiene-Purge läuft amortisiert (§9.5).
3. **Pure-Demotion:** R-PL-3. Pure ist keine Einbahnstraße mehr.
4. **`max_live_files`** (Standard 262 144): ab 80 % hebt der Governor den effektiven `pure_threshold` je Stufe um den Faktor 2 und alarmiert; bei 100 % entstehen keine neuen Pure-Dateien (alles Packed) und keine neuen `ClusterLocal`-Cluster. Gate, KeyDestroy und Purge bleiben ausführbar.
5. **FD-Cache:** `max_open_files` (Standard 1024) ist die Größe eines LRU-Caches offener Handles und **kein** Limit der Dateianzahl. Lesepfad, Wartung und Purge teilen ihn; `urgent`-Arbeit hat `fd_reserve` (256) exklusive Handles. `open` prüft `RLIMIT_NOFILE ≥ max_open_files + fd_reserve` über `HostProbe`/Fs-Adapter, sonst `Err(FdLimitTooLow{needed})`. Ein Cache-Miss kostet einen `open()`; H-27 begrenzt dessen Rate.
6. **Verzeichnis-Sharding:** §5.4; keine Millionen Einträge in einem Verzeichnis.
7. **Konvention „Session ≠ Cluster“:** §12.6. Der größte Hebel gegen Metadaten- und Dateiflut liegt oben: pro Sitzung einen Cluster anzulegen, wäre ein Modellfehler, kein Kernproblem.

### 4.4 Wert, Kopf, Körper, Gewichte

Wert = `header ‖ body`, getrennt durch `header_len`. **Der Kern interpretiert den Kopf nie.** `weights: [u32; W]` (`W = weight_slots`, 0…4); Block-Index hält `weight_sum[W]` und `entry_count`. Körper ≥ `value_separation_min` (1 KiB) gehen in Blobs (`PutRef`), der Kopf liegt immer inline.

### 4.5 `SpaceOptions` (unveränderlich)

```rust
pub struct SpaceOptions {
    pub class: SpaceClass,          // Durable | Volatile
    pub clustered: bool,
    pub data_class: DataClass,      // NonPersonal | Personal; Personal erzwingt Cluster und Sealed
    pub packing: PackPolicy,        // None | ByTenantAndExpiry{ bucket: Month|Quarter|Year, shards: u8 }
    pub pure_threshold: u64,        // Standard 8 MiB
    pub seal: Seal,                 // ByMode | Cluster
    pub weight_slots: u8,
    pub history: HistoryPolicy,     // KeepLatest | KeepUntilFloor | AppendOnly
    pub value_separation_min: u32, pub max_key_bytes: u32,
    pub compression: Compression,   // v1: None
    pub durability_default: Durability,
}
```

`DataClass::Personal` verlangt `integrity_mode = Encrypted`; bei Durable verlangt es `seal = Cluster`. Die Enum verhindert ungültige boolesche Zustände in der API, die Validierung der Kombinationen erfolgt vor I/O und ist kein automatischer PII-Detektor. **Volatile-Spaces:** kein Redo/Run/Blob, im Catalog nur der Name, sofort sichtbar, Inhalt verschwindet bei Schließen/Absturz, Erase = Freigabe mit Überschreiben (best-effort). Mischchanges mit Volatile- und Durable-Zielen nur mit `allow_mixed`, nur `durable`, wenn alle Ziele Durable sind (`INV-VOL-1`).
`compression=None` ist die einzige v1-Option; nicht unterstützte Kompression wird vor Mutation mit `Unsupported` abgewiesen. Kompression erfordert später ein versioniertes Format mit unkomprimierter Ziellänge und hartem Expansionslimit.

### 4.6 Limits

`max_key_bytes` 1 KiB (≤ 4 KiB) · `max_header_bytes` 4 KiB (≤ 64 KiB) · `max_body_bytes` **4 MiB (hart; E-35)** · `max_change_ops` 4096 · `max_change_bytes` 8 MiB · `max_page_bytes` 1 MiB · `max_clusters` 16 Mio. (weich, ≤ 64 Mio.; effektiv durch `cluster_meta_budget` begrenzt, §4.3.1) · `max_branches_per_cluster` 1024 · `max_branch_depth` **4 (Default), 8 (hart)**; Tiefen > 4 nur mit `BranchOptions::deep_ack`; H-21 misst jede Tiefe 1…8, verfehlt Tiefe 8 das Ziel, wird die harte Obergrenze gesenkt (§7.1). Überschreitung ⇒ `Limit`/`ChangeTooLarge` **vor jeder Sperre** (`INV-LIM-1`).

### 4.7 Datenklassen

`DataClass` ist ein geschlossenes Enum mit `NonPersonal` und `Personal`; fehlende oder unbekannte Klassifikation wird beim Erstellen eines Spaces abgewiesen (kein impliziter Default an externen API-Grenzen). Der Default des internen Builders darf `NonPersonal` nur für explizit nicht-personenbezogene System-/Anwendungsdaten wählen. Die deklarierende Contextra-Schicht ist für die inhaltliche Klassifikation verantwortlich; der Storage-Kern führt keine Erkennung personenbezogener Daten durch.

| Klasse | Zulässige Ablage | Lösch-/Schutzsemantik |
|---|---|---|
| `NonPersonal` | geclusterte oder nicht geclusterte Spaces gemäß übrigen Optionen | Schutz folgt `SealPolicy`; keine personenbezogene Löschzusage allein aus der Klassifikation |
| `Personal` | ausschließlich geclusterter Space; `IntegrityMode::Encrypted`; Durable zusätzlich `SealPolicy::Cluster` und ausschließlich `Sealed`-Cluster | Cluster ist kleinste Crypto-Erasure-Domäne; abgeleitete Inhalte erben die strengere Klasse und benötigen `DerivedFrom`-Lineage bzw. registrierten Owner |

Ein `Personal`-Space im `Volatile`-Modus bleibt nur im Prozessspeicher, hat keine Durability und erzeugt keinen dauerhaften Erasure-Nachweis; sein Schutz während der Laufzeit folgt dem konfigurierten verschlüsselten Modus. Ein Space-Wechsel oder Export darf Klassifikation nie stillschweigend herabstufen. Cross-Cluster-Referenzen zwischen unterschiedlichen Personen sind keine Erlaubnis zur gemeinsamen Schlüssel-/Löschdomäne; gemeinsam verantwortete Falldaten benötigen eine ausdrücklich modellierte gemeinsame Löschdomäne und dokumentierte Betroffenenbeziehungen.

---

## 5. Platzierung, Formate, Siegel

### 5.1 Pure, Packed, Staging

- **R-PL-1** Jede Datei ist genau eine von Pure (ein Cluster), Packed (mehrere Cluster einer Packgruppe), Staging (nur L0, beliebige Cluster, nur Runs).
- **R-PL-2** Jeder Cluster bildet je Datei **einen zusammenhängenden Abschnitt**, sortiert nach ClusterId, Blöcke clusterrein, Offsets abschnittsrelativ.
- **R-PL-3** Cluster ≥ `pure_threshold` wird bei der nächsten Compaction Pure. Fällt ein **nicht** `ClusterLocal`-Cluster unter `pure_demote_ratio × pure_threshold` (Standard 0,25 ⇒ 2 MiB; Hysterese gegen Flattern), führt eine Compaction ihn per `rebucket` (ein Edit) in seine Packgruppe zurück. Ohne Demotion hinterließen schrumpfende Cluster unbegrenzt viele Kleinstdateien (Inode-Flut, §4.3.1). `ClusterLocal`-Cluster bleiben Pure (§4.3).
- **R-PL-4** Compaction mischt nur innerhalb einer Wartungsgruppe (Packgruppe oder Pure-Cluster); der Spill (R-PL-7) entmischt Staging.
- **R-PL-5** Ein Flush schreibt je Space **eine** Staging-Datei plus je großem Cluster eine Pure-Datei: L0-Dateizahl unabhängig von der Zahl der Packgruppen.
- **R-PL-6** `rebucket` ist ein atomarer Umzug in einem Edit.
- **R-PL-7 (Spill-Policy, K-36)** Staging-Runs werden size-tiered zusammengeführt (gleichgroße Runs, Schreibverstärkung ≈ log); sie werden **nicht** bei jeder Gelegenheit in die Packgruppen entmischt. Eine Packgruppe `g` wird erst gespillt (in ihr L1 überführt), wenn `staged_bytes(g) ≥ max(spill_min_bytes, size(L1(g)) / spill_ratio)` (Standard 8 MiB und 4), oder ihr ältestes gestagtes Byte älter als `spill_max_age` (Standard 6 h) ist, oder ein Purge/`rebucket` die Gruppe berührt. Damit gilt je Spill Schreibverstärkung ≤ 1 + `spill_ratio`. Ohne diese Schwelle würde ein 64-MiB-Flush bei 4096 Gruppen je Gruppe ≈ 16 KiB in ein bis zu 64 MiB großes L1 mischen (Faktor ≈ 4000). Die Zähler `staged_bytes(g)` liegen im RAM (≤ `max_pack_groups` Einträge). Die Zahl der Staging-Runs ist durch `max_staging_runs` (16) begrenzt; Überschreitung erzwingt einen Tier-Merge. Punktlesen prüft Staging-Runs nur bei `ClusterFilter`-Treffer.
- **R-PL-8 (MiniSection, K-35)** Ist die Nutzdatensumme eines Cluster-Abschnitts ≤ `mini_section_max_bytes` (Standard 4 KiB), besteht er aus **einem** SealFrame (Einträge, Range-Deletes, ChangeIndex- und BranchTable-Einträge als Teilstrukturen, lineare Suche, kein Filter). Overhead = 40 B Siegel + Directory-Eintrag statt sechs versiegelter Blöcke. Verlässt der Abschnitt die Schwelle, stellt die nächste Compaction auf `FullSection` um (und umgekehrt, mit Hysterese 50 %).

### 5.2 Siegelrahmen (eine Form für alles)

```
SealFrame := key_epoch u32 | nonce[24] | ciphertext | tag[16]     (Authenticated: nur tag; Plain: CRC32C)
AEAD = XChaCha20-Poly1305(quelle, nonce[24], AAD)
quelle = Clusterschlüssel(cluster, epoch)   bei Seal::Cluster
       = Space-DEK                          bei Seal::ByMode/Encrypted (Plain-Cluster)
       = gemäß IntegrityMode                bei Plain-Cluster (CRC-only, authentifiziert oder Space-DEK)
       Sealed-Cluster-Nutzlasten in Redo verwenden immer Seal::Cluster mit der im Op-Header gebundenen key_epoch.
AAD: Block `(instance_id, space_id, cluster_id, key_epoch, block_uid)` · Redo-Op `(instance_id, space_id, cluster_id, key_epoch, op_uid, op_index)` · Blob-Record `(instance_id, space_id, cluster_id, key_epoch, record_uid)`. `space_id` ist kanonisch unsigned BE-u64; AAD-Domain und Feldreihenfolge sind Teil der Referenzvektoren.
```

Pro SealFrame gibt es keine zusätzliche HKDF/HChaCha-Ableitung: XChaCha20-Poly1305 erhält den bereits scope-spezifischen Quellschlüssel, 24-Byte-Nonce und AAD direkt und führt seine spezifizierte HChaCha-Subkey-Ableitung intern aus. `Authenticated` nutzt die ersten 16 Bytes von HMAC-SHA256 über kanonisch längencodierte Domain `ctxs-auth-v1`, Modus, Nonce, AAD und Klartext; Tagvergleich ist constant-time. `Plain` nutzt CRC32C und bietet keine Manipulationsauthentizität. RootKey-abgeleiteter Redo-Schlüssel dient ausschließlich Ketten-MAC/Frame-Integrität. Nutzlasten eines Sealed-Clusters im Redo sind zusätzlich mit dessen Cluster-Epoche versiegelt; ein unter dem RootKey allein verschlüsselter Nutzwert wäre mit `INV-ERA-7` unvereinbar. AAD und Siegel hängen nicht von Dateiname, Datei-ID, Blob-ID, Offset oder Dateiposition ab; `block_uid`/`record_uid` sind bei Rohkopie stabil ⇒ Rohkopie über Dateien und neue Blob-IDs ist ohne Entschlüsselung gültig. Versiegelt wird vor Stamp-Vergabe; Stamp ist über Frame-Ketten-MAC gebunden. `INV-SEAL-1`: Sealed-Cluster haben Nutzdaten nur in `Seal::Cluster`- oder Volatile-Spaces; Plain-Cluster nie in `Seal::Cluster`-Spaces; Spaces mit `DataClass::Personal` nehmen nur Sealed-Cluster. Folge: alle **Nutzdaten** eines Sealed-Clusters auf Platte (Redo-Op, Runs, Blobs, Kopf, Körper, Index, Filter, Branch-Nutzdaten) sind unter dessen Cluster-Epoche versiegelt. Klartext bleiben ausdrücklich: zufällige ClusterId, Routing-Header/Op-Tags, Gate-/Control-Frames, Größen/Stamp-Bereiche und Dateistruktur; dies ist das Metadaten-Leckagebudget und enthält keine Nutzwerte.

### 5.3 Konventionen aller Formate

Little-endian; `varint` = kanonisches LEB128; Dateikopf `magic[8] ‖ file_kind u8 ‖ format_ver u16 ‖ feature_bits u32 ‖ file_id u64 ‖ instance_id[16] ‖ header_crc u32`; Längen vor dem Allokieren prüfen (`INV-LIM-1`); reservierte Felder sind 0 und werden beim Lesen geprüft; eine Version je Dateiart, unbekannte Version/Pflicht-Bits ⇒ `Unsupported` (`INV-FMT-1`). Feature-Bit `BRANCHES` (§8) in `FORMAT`: solange ungesetzt, ist `branch ≠ 0` ein Format-Fehler. v1 setzt `COMPRESSION=0` und `filter_kind=Bloom(1)`; unbekannte Werte ⇒ `Unsupported`. Andere Kompressions-/Filterkodierungen benötigen ein neues Pflicht-Feature oder eine Formatmigration.

### 5.4 Dateien

```
<root>/FORMAT  LOCK  CURRENT  CATALOG-<n>  KEYS  RESERVE
       redo/redo-<n>.log  spaces/<id>/<s>/<file_id>.run|.blob   (s = file_id mod file_shard_count, 3 Hex-Stellen)  segments/<kind>/<id>.seg
<ledger_dir>/ERASE-LEDGER/…      (Standard: <root>/ERASE-LEDGER, Produktivempfehlung: getrenntes Volume)
```

`FORMAT` unveränderlich (+ RootKey-abgeleiteter MAC); `KEYS` = RootKey (mit KEK umhüllt) + Space-DEKs (mit RootKey umhüllt); **Clusterschlüssel stehen nie hier**. Alle Dateien `Perm::Private`. Temporäre Dateien `.building.<tmp_id>`. Verwaiste Dateien (nicht im Catalog) werden nach Recovery entfernt; sicher, weil nur ein Edit eine Datei lebendig macht (`INV-CAT-1`). `space_add` speichert die `SpaceId`; `reserve_space_ids` persistiert den High-Water-Mark vor Verwendung. Kein Recovery- oder Restore-Pfad darf IDs aus gelöschten oder alten Catalog-Ständen erneut vergeben.

### 5.5 Redo

Vorallokierte Segmente (64 MiB, `redo_align` 4096), am Ende gesiegelt; Gruppe = ≥ 1 Frames, Ketten-MAC `H_i = MAC(H_{i-1} ‖ head ‖ payload_tags)`.

```
Frame := head ‖ body ‖ tail
head (Klartext, kettengeschützt): len u32 | stamp u64 | kind u8 | n_ops u16 | idem_token[16] | epoch u64 | flags u8 |
      ops_head[]: { space_id u64, cluster[16], branch u32, op_kind u8, key_epoch u32, op_uid[16], payload_len u32 }
kind: Change=1 | Gate=2 | Control=3 | Reserve=4
Gate-Body (Klartext):    scope(Clusters|Space) | ticket[16] | level | policy_epoch u64 | n u16 | cluster[16]×n oder space_id u64
Control-Body (Klartext): op(Restrict|Lift|Link|BranchFork|BranchDrop|BranchMerge) | … Nutzdaten der Branch-Ops sind versiegelt; Link-Body: derived[16] | n u8 | sources[16]×n | policy_epoch_after u64
tail: crc32c | chain_mac
```

Redo-Integritätsschlüssel = `KDF(RootKey, "ctxs redo-auth v1")` (nicht je Segment), ausschließlich für Ketten-MAC. Nutzlasten von Sealed-Clustern sind mit deren `Clusterschlüssel(cluster,key_epoch)` verschlüsselt; Plain-Nutzlasten folgen dem `IntegrityMode` und ggf. Space-DEK. Beim Recovery: Gate-/Control-Scan bestimmt `retired_set` ohne Entsiegeln; aktive Sealed-Ops werden nur mit ihrer `key_epoch` geöffnet. Provider nicht erreichbar ⇒ `KeyUnavailable`, Recovery bleibt in `LockedKeys`; bestätigter Schlüsselverlust ohne Löschbeleg ⇒ `KeyLost/Corrupt` und Quarantäne der betroffenen Cluster-Domäne. Nie stillschweigend überspringen und diese Zustände nie als `Shredded` behandeln. Torn-Tail (`INV-LOG-8`): erster ungültiger Gruppenkopf beendet das Log nur, wenn danach nichts Gültiges folgt; sonst `Corrupt{RedoMidLog}`; alles bis zur letzten `redo_mark` (alle 8 MiB und bei Siegel) muss gültig sein. **Kein Segmentschnitt beim Gate.**

### 5.6 Run

```
Run := header · ClusterSection* · ClusterDirectory · PackMeta · ExtensionBlock* · Footer
ClusterSection := FullSection | MiniSection
FullSection    := DataBlock* · SubIndexBlock · BranchTableBlock · FilterBlock · RangeDelBlock · ChangeIndexBlock
MiniSection    := SealFrame{ entries · range_dels · change_idx · branch_tab }      (≤ mini_section_max_bytes, kein Filter, R-PL-8)
```

`ClusterDirectory` (Klartext): `{cluster, section_kind u8 (0 Full, 1 Mini), section_off, section_len, n_blocks, n_entries, min/max_stamp}`; keine Gewichte/Tokenprofile im Klartext. `DataBlock` Ziel 16 KiB, Präfixkompression, versiegelt; Eintrag trägt `branch`. `SubIndexBlock` je Block: letzter Schlüssel, abschnittsrelativer Offset, Länge, `entries`, `weight_sum[W]`, `min/max_stamp`, `block_uid`; bei Sealed vollständig unter Clusterschlüssel versiegelt. Page darf diesen kleinen Indexblock entsiegeln, ohne Body-Block/Blob zu lesen. v1-`FilterBlock` ist ausschließlich Bloom (`filter_kind=1`) über `(branch, user_key)` (branch-bewusst, K-34; je Kettenebene eine RAM-Probe), samt Salt cluster-sealed; Ribbon ist kein v1-Format. Der `BranchTableBlock` (cluster-sealed, mit dem SubIndex geladen) listet je Branch `{branch u32, first_block u32, last_block u32, n_entries, min/max_stamp, has_range_del}` sortiert nach `branch`; da `cmp` den Branch vor den Schlüssel sortiert, bilden die Einträge eines Branches einen zusammenhängenden Blockbereich (§7.1). Nur Directory, PackMeta und Footer mit nicht sensiblen Strukturmetadaten bleiben beim Umkopieren unverändert/werden neu erzeugt. Das `ClusterDirectory` ist nach `ClusterId` sortiert und in 4-KiB-Seiten gegliedert (≈ 78 Einträge je Seite); im RAM liegt nur der **Sparse-Index** (erste `ClusterId` und Seitenoffset je Seite, ≈ 20 B je Seite) sowie der `ClusterFilter` (Bloom über `ClusterId`, ≈ 8 Bit je Cluster, in `PackMeta`). Seiten laden über den Block-Cache (Klasse `DirPage`). Punktlesen kostet je Level-Datei höchstens einen zusätzlichen Seitenzugriff, bei Warm-Cache keinen; der `ClusterFilter` erspart ihn für Dateien, die den Cluster nicht enthalten. Resident-Kosten: §4.3.1.

### 5.7 Blob

`Blob := header ‖ ClusterSection* (Record*) ‖ ClusterDirectory ‖ footer`, `Record := record_uid[16] | len u32 | SealFrame | crc32c`. AAD bindet `record_uid`, nicht `blob_id` oder Offset; deshalb darf ein unverändertes SealFrame roh umkopiert werden. Ziel ≤ 64 MiB. **Inkarnation:** Kleine Löcher dürfen bei unterstütztem Hole-Punch dieselbe `blob_id`/Offsets behalten. Überschreitet Fragment-/Hole-Ratio den Grenzwert oder fehlt effizientes Hole-Punch, wird in eine neue `blob_id` mit gepackten Überlebenden umgeschrieben; Catalog-Edit `blob_replace` aktualisiert Referenzen atomar. Blob-GC darf zusätzlich in eine neue `blob_id` umziehen; bei unverändertem Record ist keine Neuversiegelung nötig. Eine Blob-Datei/Inkarnation wird nur entfernt, wenn **kein lebender Run** mehr einen `PutRef` darauf enthält (`INV-BLOB-1`).

### 5.8 Catalog

Snapshot-Record + Edit-Records (Kette, CRC), nur in vorallokierten 1-MiB-Blöcken. **Gruppen-Commit:** `catalog_edit` fasst wartende Edits mit einem `write` + einem `sync_data` zusammen, entlässt die Wartenden erst danach (`INV-CAT-2`); mehrere gekoppelte Zustandsänderungen sind **ein** Edit. Rollover bei `max(8 MiB, Größe des letzten Snapshots)`; `CURRENT` per atomarem `rename`.
Ops: `space_add(space_id,options) | space_state | space_remove | cluster_register | cluster_state | cluster_trim | cluster_link | restriction_set | restriction_lift | pack_assign | files_add(meta.floor) | files_remove | unlink_ack(file_id) | blob_replace | blob_live_delta | replay_floor | retain_floor | redo_mark | segment_add | segment_remove | hold_set | consumer_ack | erase_pending_add | erase_stage | erase_done | erased_ranges_add | erased_ranges_trim | key_epoch | kek_id | idem_trim | reserve_stamps | reserve_ids | reserve_space_ids | anchor_mark`. `space_id` ist ein unveränderliches Identitätsfeld; jedes Space-Gate bindet diese ID. `files_remove` verschiebt die Datei im selben Edit in den persistenten Graveyard; `unlink_ack` entfernt sie erst nach erfolgreichem `unlink + sync_dir`. Reservierter Bereich `0xF000…0xFFFF`; unbekannte Tags ⇒ `Unsupported`. **Der Catalog enthält keine Branch-Namen und keine Nutzdaten** (§8.2).
Reservierungen: `reserve_stamps` (2^20), `reserve_ids` (2^16) und `reserve_space_ids` (2^16) heben Hochwassermarken vorab an (Wartungsthread bei < 50 % Rest); Zuweisung ist crash-sicher und überspringt ungenutzte IDs. Erschöpft eine Reserve, weist der Sequencer normale Changes mit `Backpressure(Retry)` ab, **ohne** Rang 3 zu nehmen. Gate-, Control- und Link-Frames ziehen aus dem **Notpool** (`reserve_stamps_urgent` 2^10, `reserve_ids_urgent` 2^8), den nur die Lane `urgent` auffüllt. Es gibt **keine** Doppelhaltung von Rang 1 und Rang 3 (§11.2).

### 5.9 Freeze-Liste (Gate G-F)

Dateikopf, `FORMAT`, `KEYS`, Redo-Gruppe/Frame (inkl. `space_id`, `branch`, `kind`, `epoch`, `op_uid`, Gate-Body inkl. `policy_epoch`, Control-Body inkl. `Link`), Kette, ClusterSection (Full/Mini, `BranchTableBlock`, branch-bewusster Filter), ClusterDirectory (seitenweise, Sparse-Index, `section_kind`), PackMeta (inkl. `ClusterFilter`), SubIndex, Siegelrahmen, Dateipfad-Sharding, Blob-Record und Inkarnationsregel, Catalog-Op-Tags einschließlich `reserve_space_ids`, `AnchorValue`, Ledger-Format einschließlich Domain-Separation und Byteordnung der Tomb-HMACs, `tomb_epoch u32` je Tomb-tragendem Event und der Tomb-Epochen-Events (K-31), `cmp` (inkl. Branch), Tag-Kodierung. Je Format ein unabhängiger Referenzcodierer (Python) und goldene Vektoren (Anhang A). Nach G-F ist jede Änderung eine Migration.

| System-Space | SpaceId | Zweck |
|---|---:|---|
| `SYS_BRANCH` | 1 | clusterlokale Branch-Metadaten |
| `SYS_IDEM` | 2 | instanzweite Idempotenz-Ergebnisse |
| reservierte Systemzone | 0, 3…255 | für versionierte Kernspaces; normale `space_add`-Aufrufe dürfen diese IDs nie verwenden |

Die Belegung dieser Tabelle ist Teil des Format-Freeze. Neue Systemspaces erhalten nur bisher unbelegte IDs dieser Zone; die User-ID-Reservierung beginnt bei 256.
Öffentliche `spaces()`-/`apply()`-Pfade weisen jede ID `< 256` mit `ReservedSpace` ab; nur der jeweilige Systemspace-Owner darf über interne, typisierte Operationen zugreifen. Die numerische ID ist kein Autorisierungsnachweis.

---

## 6. Schreibpfad

### 6.1 `apply` (Commit-Pipeline)

Der Committer besteht aus zwei Stufen mit je genau einem aktiven Schreiber (K-33):

| Stufe | Sperre | Aufgabe | Verboten |
|---|---|---|---|
| **Sequencer** | Rang 1 (`sequencer`) | Zustandsprüfung, Guard-Delta, Epochenvergleich, Stamp-Vergabe, Übergabe an den Syncer. Rein RAM, O(Ops der Gruppe) | jede E/A, jeder Provider-/Cipher-/KDF-Aufruf, jedes Warten, Graph-Traversierung (einzige Ausnahme: Selbstberechnung nach Epochen-Retry, ≤ `max_gate_clusters` Knoten, nur RAM) |
| **Syncer** | keine (der Thread besitzt die Redo-Datei) | CRC und Ketten-MAC, ein `write` + ein `sync_data`, danach Publikation in Stamp-Reihenfolge | Stamps vergeben, Zustand validieren |

```
1  Vorprüfung ohne Sperre: Größen/Limits (INV-LIM-1), Space/Cluster bekannt, INV-SEAL-1, Clusterzustand, DiskFull,
   Idempotenz-Lookup (Treffer ⇒ gespeichertes Ergebnis, AlreadyApplied)
2  Vorbereiten auf dem Aufruferthread (keine Sperre; Plattenzugriffe und Provider-Wartezeit sind hier erlaubt):
   a) View V1 pinnen, Guards exakt gegen V1 auswerten (Stufe A, §6.3), Punkt-Fingerabdrücke fp berechnen
   b) bei Gate/Link/Restrict/Lift/Space-Drop: Hülle H, Restrictions und Space-Restriction-Treffer berechnen; `policy_epoch` P1 notieren
   c) Ops kodieren, op_uid ziehen, Nutzlast versiegeln, Frame vollständig vorkodieren (Stamp-Felder 0, Kette/CRC leer)
   d) implizite Absent-Guard für den SYS_IDEM-Schlüssel anfügen (§6.5)
3  Stamp-Reserve prüfen: erschöpft ⇒ Backpressure(Retry) (normale Changes); Gate/Control/Link ziehen aus dem Notpool (§5.8)
4  Pipeline-Credit holen (Semaphor, `commit_pipeline_depth`, Standard 2; `urgent` hat einen eigenen Credit).
   Warten findet nur hier statt, nie unter einer Sperre
5  Einreihen: an die Intake anhängen (Rang 2, kurz, ohne weitere gehaltene Sperre), Rang 2 wieder frei. Dann try_lock auf Rang 1:
   Erfolg ⇒ Leader (nimmt Rang 2 nur zum Leeren der Intake, also aufsteigend); Misserfolg ⇒ auf das Ergebnissignal warten.
   Der Leader prüft nach dem Freigeben von Rang 1 die Intake erneut und übernimmt bei Nichtleere erneut (kein verlorener Wakeup, loom)
6  Sequencer (Rang 1), je Change in Intake-Reihenfolge:
     a) Zustand: Retiring/Retired/Shredded/Purged ⇒ ClusterRetired; BLOCK_WRITE ⇒ Restricted; Branch nicht Live ⇒ BranchClosed
     b) Guards Stufe B: Delta (V1, head] im Konfliktfenster; Treffer oder V1 < Fensterfuß ⇒ GuardRetry (Aufrufer wiederholt ab 2a)
     c) Gate/Link/Restrict/Lift: policy_epoch == P1, sonst PolicyRetry; nach `policy_retry_max` (3) Fehlversuchen berechnet der
        Sequencer die Hülle einmalig selbst (eine Löschung wird nie ausgehungert)
     d) Stamps vergeben; vorreservierte Branch-IDs validieren, Branch-Ops ausführen; bei Gate: cluster_table Live → Retiring(S)
     e) Konfliktfenster um (stamp, fp, ck) der Gruppe fortschreiben (schon vor der Durabilität)
     f) Gruppe (vorkodierte Frames mit Stamps) in die Syncer-Warteschlange legen (nie blockierend, der Credit garantiert Platz); Rang 1 frei
7  Syncer: Frames finalisieren (CRC, Ketten-MAC), ein write + ein sync_data                  ◄── DURABILITÄTSPUNKT
8  Publikation (Syncer, strikt in Stamp-Reihenfolge, nur nach Schritt 7 derselben Gruppe): Memtable einfügen, Fenstereinträge als
   durabel markieren, Retiring → Retired + gen++, Version veröffentlichen (Rang 4), visible_stamp/durable_stamp anheben (Release),
   Aufrufer wecken, Credit freigeben
9  Receipt { instance_id, stamp, durable }
```

**Eigenschaften.**

1. *Durchsatz* = Gruppengröße / Sync-Latenz. Die Pipeline hebt die Fsync-Rate nicht an, aber der Sequencer baut die nächste Gruppe, während die vorige synchronisiert; alle in dieser Zeit eintreffenden Changes bilden eine Gruppe, die mit der Last wächst (bis `max_group_changes` 1024 / `max_group_bytes` 8 MiB). Kein Graph-, Guard-, Idempotenz- oder Kryptoaufwand verlängert den kritischen Pfad. Tiefen > 2 nur, wenn die Fs-Konformanzsuite überlappende `sync_data`-Aufrufe als beschleunigend belegt.
2. *Fehler:* Sync-Fehler ⇒ Poison. Die fehlgeschlagene und alle bereits sequenzierten späteren Gruppen sind `Indeterminate{first_stamp,last_stamp}` bzw. `Poisoned`; nichts davon wurde veröffentlicht (`INV-PIPE-2`). Ein Konflikt oder `ClusterRetired`, der durch eine noch nicht durable Gruppe entstand, ist harmlos: scheitert diese, ist das Handle ohnehin vergiftet, nach dem Neustart entscheidet allein das Redo.
3. *Sichtbarkeit:* Leser sehen `Retiring` als `Live`, bis das Gate durabel und publiziert ist. Es wird nie ein nicht durabler Zustand gelesen; die Schreibadmission ist dagegen sofort geschlossen, weil kein Change nach einem sequenzierten Gate in denselben Cluster geraten darf.
4. *Memtable-Einfügung* durch den Syncer ist Standard. Parallele Einfügung durch die wartenden Aufrufer mit Barriere vor der Publikation ist eine zulässige Optimierung, sofern der Syncer bei Ausfall eines Aufrufers nach `publish_timeout` selbst einfügt.
5. `Relaxed`-Changes reiten in der nächsten Gruppe mit und werden nach dem `write` (ohne Sync) veröffentlicht; nie für Löschung.

Sichtbarkeit nur nach Durabilität (Ausnahme `Relaxed`). Jeder Stamp und jede Receipt ist nur innerhalb ihrer `instance_id` eindeutig; Vergleiche, Feed-Offsets oder Audit-Referenzen über Instanzen müssen das Tupel `(instance_id, stamp)` führen. Rückdruck stufenweise **außerhalb** jeder Sperre (`Backpressure(Retry)`); Warten unter Rang 1 ist verboten. `Urgent`-Löschfolgen umgehen den Rückdruck (eigener Credit, Notpool).

### 6.2 Memtable und Stamps

Lock-freie Skiplist hinter dem Trait `Memtable`, Klartext (AN-05); 64 MiB je Space, ≤ 3 immutable, 256 MiB gesamt. Nach Absturz: `next_stamp = max(stamp_limit, letzter gültiger Redo-Stamp, max Run-Stamp) + 1`. Consumer und `view_durable` sehen nur Stamps ≤ `durable_stamp` derselben `instance_id` (`INV-CONS-1`).

### 6.3 Guards: optimistische Validierung in zwei Stufen

**Stufe A (Aufrufer, außerhalb jeder Sperre).** Der Aufrufer pinnt `V1` und wertet jede Guard **exakt** gegen `V1` aus (Plattenzugriffe, `ChangeIndex`-Scans, Merge-Konfliktprüfung; `Precondition`: `Absent | Version(Stamp) | Unchanged{since}`). Konflikt ⇒ `Err(Conflict)` an die Anwendung; der Sequencer wird nicht bemüht. Sonst entstehen Guard-Beweise: für Punkt-Guards `(V1, fp)` mit `fp = SipHash-1-3(prozessschlüssel, space‖cluster‖branch‖key)`, für `Guard::Range{start,end,since}` (halboffen) `(V1, ck, [start,end))` mit `ck = (space, cluster, branch)`.

**Stufe B (Sequencer, Rang 1, nur RAM, O(Guards)).** Der Sequencer prüft lediglich, ob seit `V1` etwas Relevantes sequenziert wurde. Datenstruktur: **Konfliktfenster** = stamp-sortierter Ring der letzten `guard_window_entries` (≤ 1 Mio.) Mutationen `(stamp, fp, ck)`, einschließlich sequenzierter, noch nicht durabler Gruppen; nur im Speicher.

- Punkt-Guard: ein Eintrag mit gleichem `fp` und `stamp > V1` ⇒ `GuardRetry`.
- Range-Guard: jeder Eintrag mit gleichem `ck` und `stamp > V1` (Punkt, `RangeDelete`, `EraseRange`, Merge) ⇒ `GuardRetry`. Es findet bewusst kein Schlüsselvergleich unter Rang 1 statt (konservativ, nie ein übersehener Konflikt); der Punktfilter bestätigt Range-Guards nie.
- `V1 < Fensterfuß` ⇒ `GuardRetry`.

Der Sequencer urteilt nie über Anwendungskonflikte; `GuardRetry` heißt nur „V1 ist veraltet“. Der Aufrufer wiederholt Stufe A mit frischem `V1`, höchstens `guard_retry_max` (3) Mal, danach `Err(Conflict{contended: true})`. Hash-Kollisionen kosten höchstens einen Retry. **Es gibt keinen Langsampfad im Sequencer**: Ein exakter Vergleich gegen Memtable ∪ Version bedeutete Plattenzugriffe unter Rang 1 (`K-21`).

`INV-GUARD-1`: Ein Change wird nur angenommen, wenn jede Guard zur Stamp-Vergabe gilt. Begründung: Stufe A ist exakt zu `V1`; Stufe B schließt jede Mutation in `(V1, Stamp-Vergabe]` aus oder liefert `GuardRetry`. M1 modelliert Fenster, Fensterfuß und Pipeline.

**Guard-Katalog je Operationsart (aus CairnDB, als Konvention für die obere Schicht):** Event anhängen → keine Guard (append-only); Working State → `Version`; Fakt ändern → `Version`; Tool-Zustand → `IdemToken` + `Version`; Branch-Merge → `Unchanged{since: merge_base}` je Schlüssel (§8.5).

### 6.4 Flush

Je Space: Memtable einfrieren → je Cluster ein Abschnitt (**Retired-Cluster werden übersprungen**) → Staging- plus Pure-Runs (`.building` → `sync_all` → `rename` → `sync_dir`) → **ein** Edit (`files_add`, `replay_floor`, `anchor_mark`) → neue Version. `replay_floor` rückt nur über Segmente, deren Frames in durablen Runs stehen **und** deren Gate-Frames als `erase_pending` im Catalog stehen (`INV-LOG-5`). Flush braucht die Clusterschlüssel der Memtable (gepinnt für die Jobdauer); fällt der Provider aus, pausiert er mit Backoff.

### 6.5 Idempotenz

`IdemToken` im Frame-Kopf; der Aufrufer legt zusätzlich einen internen `SYS_IDEM`-Record im **selben Redo-Frame/Commit** wie den Change an und setzt eine **implizite `Absent`-Guard** auf dessen HMAC-Schlüssel (K-38). Die Eindeutigkeit des Tokens wird damit durch denselben Zwei-Stufen-Mechanismus gesichert wie jede andere Guard (§6.3): kein zweiter, im Sequencer zu pflegender Token-Index und kein RAM-Wachstum ∝ aktive Tokens (bei 1000 Commits/s und 24 h TTL wären das ≈ 86 Mio. Einträge). Zwei gleichzeitige Requests mit demselben Token: der zweite erhält `GuardRetry`, wiederholt Stufe A, findet das gespeicherte Ergebnis und liefert `AlreadyApplied`. Der Vorprüfungs-Lookup (§6.1, Schritt 1) ist nur eine Abkürzung. `SYS_IDEM` ist ein reservierter, instanzweiter Systemspace unter dediziertem System-DEK; Record-Schlüssel und Request-Fingerprint sind keyed HMACs, kein Nutzwert und keine Klartext-ClusterId. Systemspace-Key/HMAC-Epoche bleibt bis Ablauf aller damit markierten Tokens verfügbar. Record-Payload wird vor Rang 1 versiegelt; Ergebnis-Stamp ist der äußere MVCC-Stamp des SYS_IDEM-Records, kein im Sequencer zu versiegelndes Feld. Aktive Records dürfen nicht durch `replay_floor` verworfen werden: sie werden geflusht/kompaktiert und bis TTL-Ende gepinnt. Gespeichert werden Token-Scope/Hash, kanonischer Request-Fingerprint, Status, Ablaufbucket und pseudonymes Cluster-Scope-Digest aus `tomb_id`s. Catalog-`idem_trim` ist nur eine durable Trim-Wasserstands-/Epoch-Marke, kein per-Commit zweiter Fsync. Aktive Tokenanzahl und SYS_IDEM-Bytes sind konfigurationsbegrenzt; bei ausgeschöpftem Limit wird ein tokenierter Commit vor Mutation mit `IdemCapacity` abgelehnt, aktive Tokens werden nie vorzeitig verdrängt. Bei nicht verfügbarem SYS_IDEM-Schlüssel werden tokenierte Reads/Writes fail-closed abgelehnt; kein Retry darf ohne Idem-Prüfung committen. Systemspace-Format, Schlüsselableitung, TTL-Index, Limits und Trim-Recovery sind G-F-Formatbestandteil. Wiederholung desselben Changes ⇒ gespeichertes Ergebnis; Token mit abweichendem Request ⇒ `IdemConflict`; nach Cluster-Retire ⇒ `Erased`/`ClusterRetired`, niemals neues Apply.

### 6.6 Notbetrieb bei vollem Datenträger

Redo, Catalog und Ledger schreiben nur in **vorallokierte** Bereiche (AN-10); `ENOSPC` tritt nur beim Vorallokieren auf und hinterlässt keine Teilschreibung. `DiskFull` verweigert User-Commits; Reads und Gate/KeyDestroy nutzen getrennte Emergency-Reserven. Purge hat eigenen geschützten Scratch-Pool von mindestens `2 × max(run_target_bytes, blob_max_bytes, redo_segment_bytes)` (Default 128 MiB) für genau einen aktiven Rewrite plus Crash-Recovery-Slot. Ordinary Flush/Compaction pausieren, sofern sie diesen Pool nicht verwenden dürfen. Pool wird bei Open geprüft und kann nicht von normalen Writes verbraucht werden. Purge rewritet Dateien einzeln: `PurgeBatchEdit` installiert je Ersatzdatei atomar; vollständig tote Dateien werden direkt aus dem Catalog genommen und danach unlinkt; nach Arc-/Reader-Reclamation wird freier Quellplatz dem Scratch-Pool zurückgegeben. Nur nach letzter Datei und Extension-Attest setzt ein finaler Edit `Purged/Done`. Filesystem-Adapter dürfen diese Garantie nur deklarieren, wenn Vorallokation auch unter CoW/Quota-Bedingungen tatsächlich Platz reserviert. Reicht Reserve/FS-Capability nicht, verweigert Open den harten DiskFull-Erase-Modus oder meldet das Restrisiko explizit; kein falsches `Done`. Gate-/Control-/Ledger-Frames passen ins Emergency-Reservesegment (`INV-LOG-7`). Ausstieg mit Hysterese. Nur `EIO`/Fsync-Fehler vergiften (`INV-DISK-1`).

---

## 7. Lesepfad

1. `view()` lädt zuerst `visible_stamp` (Acquire), dann `Arc<Version>` (`INV-VIEW-2`). Keine Sperre über E/A.
2. **Gate zuerst:** Clusterzustand und Restrictions vor jeder Entsiegelung (O(1)). Unbekannt ⇒ `UnknownCluster`; `Retired` oder weiter ⇒ `Erased{at:S}` **für jede Sicht, auch für Sichten ≥ S** (Erasure-Barriere überstimmt Snapshot-Isolation); `BLOCK_READ` ⇒ `Restricted`; widerrufene View ⇒ `Revoked`.
3. **Punktabfrage:** Memtable → immutable → Staging-Runs (nur `ClusterFilter`-Treffer) → je Level die Dateien der Wartungsgruppe (Binärsuche nach `[min_cluster,max_cluster]`, `ClusterFilter`) → ClusterDirectory (Sparse-Index, höchstens eine Seite) → Abschnitt (`MiniSection`: ein Frame) → `BranchTable` → branch-bewusster Filter → SubIndex → **ein** Datenblock je Treffer-Level. Auf einem Branch läuft dies als Chain-Resolve (§7.1).
4. **Cursor/Page:** k-Wege-Merge über Quellen **eines Clusters und seiner Branch-Kette**; Kopf ohne Body-Block-E/A; `page` entschlüsselt kleine SubIndex-Metadaten mit `weight_sum`, überspringt danach Body-Blöcke ohne deren Entsiegeln und liefert `ResumeToken`. Kumulative Gewichts-/Zählsummen über die Blöcke eines Abschnitts werden beim Laden des SubIndex im RAM berechnet und nicht persistiert (K-30); sie sind eine Obergrenze (Überlappung, Schatten) und nie Korrektheitsquelle. Ein Eintrag, der das Budget sprengen würde, beendet die Seite, **außer** die Seite wäre leer (Fortschrittsgarantie, `oversize_single`). Cursor prüfen Gate und Widerruf **zwischen Blöcken**. Der `ResumeToken` ist AEAD-versiegelt unter einem Prozessschlüssel und enthält Stamp, Cluster, Branch, letzten Schlüssel (kein Klartextschlüssel außen); nach Löschung ⇒ `Err(Erased)`.
5. **Körper** erst bei `body()`; Gate ⇒ `Erased`, Provider nicht erreichbar ⇒ `KeyUnavailable`, Schlüssel fehlt ohne Löschnachweis ⇒ `KeyLost`. `Shredded` ist nur ein Clusterzustand, nie ein Lesefehler. Nie Daten, nie „leer“.
6. **Block-Cache** hält entsiegelte Blöcke mit `(cluster, gen)`; Treffer nur gültig, wenn `gen` dem Clusterzustand entspricht; Gate und ER2 erhöhen `gen`; `urgent` überschreibt die Puffer (`Zeroizing`). **Check-after-decrypt:** `gen` wird vor **und** nach dem Entsiegeln geprüft (`INV-KEY-2`).
7. `view_at(S)` mit `S < history_floor` ⇒ `BelowFloor`; Sicht unter `S_erase` in einen Bereich mit `erased_ranges` ⇒ `Erased` (`INV-ERA-5`).
8. **`View::filter_visible(&[RecordRef])`:** prüft Clusterzustand/Restriction und Stamp für Treffer, die obere Strukturen (Indizes) liefern. **Jedes Ergebnis einer Extension muss diesen Filter passieren, bevor es den Aufrufer erreicht** (`INV-VIS-1`); damit ist die Sichtbarkeit unabhängig davon, wie schnell die Extension aufräumt.
9. Keine clusterübergreifenden Scans im Kern; `Inspector::clusters()` iteriert.

### 7.1 Chain-Resolve und Branch-Kosten (K-34)

Ein Filter ohne `branch` meldete einen Schlüssel, der auf `main` existiert, auf **jeder** Ebene der Kette als Treffer und löste dort einen Block-Read aus. Deshalb sind Filter und Layout branch-bewusst:

1. **Layout.** `cmp` (§4.2) sortiert `branch` vor `user_key`; die Einträge eines Branches bilden je Abschnitt einen zusammenhängenden Blockbereich. Der `BranchTableBlock` (§5.6) liefert diesen Bereich. Hat ein Branch keine Einträge im Abschnitt, kostet das eine RAM-Suche und nichts sonst.
2. **Filter.** Der Bloom hasht `(branch, user_key)` mit cluster-gesalzenem Hash (gleiche Bits je Eintrag wie zuvor). Eine Probe je Kettenebene ist reiner RAM-Zugriff.
3. **Punktlesen = Chain-Resolve.** Kette `[b0 = Blatt … bd = Wurzel]` mit Stamp-Kappen `cap_0 = V`, `cap_{i+1} = min(cap_i, fork_stamp(b_i))`. Je Ebene und Datei: (a) `BranchTable` negativ ⇒ weiter; (b) hat der Branch `has_range_del`, die beim Öffnen in eine fragmentierte Intervallliste geladene `RangeDelBlock`-Struktur prüfen (O(log r)); (c) Filter negativ ⇒ weiter; (d) sonst Blocklesung nur im Blockbereich des Branches. Der erste sichtbare Treffer mit Stamp ≤ `cap_i` beendet die Suche (Early-Exit). Ein `Delete` oder ein `RangeDelete` mit Stamp ≤ `cap_i`, der jede eigene Version des Schlüssels überdeckt, beendet sie mit „nicht vorhanden“, ohne die Eltern zu lesen: Elternversionen haben Stamps ≤ `fork_stamp`, also kleiner als der des `RangeDelete`. Der Range-Delete-Check läuft auch bei negativem Filter, weil ein Bereichs-Tombstone keinen Punkt-Eintrag hat.
4. **Kostenmodell.** Datenblock-Lesungen ≈ `1 (main)` + `d × FPR` für Zwischenebenen + Treffer im Branch. Bei 1 % geänderten Schlüsseln und FPR ≈ 1 % ist der erwartete Mehraufwand ≈ `d × 0,01` Blöcke plus `d` RAM-Proben (≈ 50 ns je Ebene); Treffer im Blatt sind sogar billiger als auf `main`. Das ist das Tragwerk von H-21; die Messung entscheidet.
5. **Cursor/Page.** k-Wege-Merge nur über Quellen, die im Schlüsselbereich laut `BranchTable`/SubIndex Einträge haben. `RangeDelete`-Fragmente und Stamp-Kappen werden im Iterator angewandt, nicht durch Nachfilterung. Merge-Fan-in = Σ je Kettenebene (Dateien mit Treffern im Bereich), nicht Σ aller Dateien.
6. **Begrenzung.** Default `max_branch_depth` = 4 (hart 8). Tiefen > 4 verlangen `BranchOptions::deep_ack` und erscheinen in der Metrik `branch_depth`. Verfehlt H-21 bei Tiefe 8 das Ziel, sinkt die harte Obergrenze (Spec-Änderung); das Messverfahren wird nicht gelockert.
7. **Korrektheit.** Filter und `BranchTable` dürfen nie ein falsches Negativ liefern (`INV-BR-5`). Property-Test: Chain-Resolve und Cursor ≡ Referenzmodell, einschließlich `RangeDelete` auf Zwischenebenen, `Fork At(S)` und Compaction mit Fork-Floor.
8. **Vor G-F.** Weil Filter-Hashing, `BranchTableBlock` und `MiniSection` Format sind, ist eine Spike-Messung H-21 (Tiefe 1…8) und H-11 (MiniSection) Voraussetzung des Formatfreezes (§14, P2).

---

## 8. Versionierung (Branches, Diff, Merge)

### 8.1 Modell

| Git-Begriff | contextra-storage |
|---|---|
| Commit | ein durabler `Change` (Frame, Stamp); `Receipt.stamp` ist die Commit-ID innerhalb der Instanz |
| Zeitreise | `view_at(stamp)`, `View::on_branch(b)` |
| Branch | `BranchId` je Cluster; **O(1)-Fork**: ein Control-Frame, keine Datenkopie |
| Diff | `diff(view_a, view_b, cluster, range)` |
| Merge | `merge(cluster, source, target, mode)`: Konfliktliste oder ein atomarer Change |
| Rollback | `branch_drop` oder `reset`-Change auf Branch |

**Warum clusterlokal (K-07):** Ein instanzweiter Commit-DAG mit Inhaltsadressierung würde Daten verschiedener Löschdomänen über Hashes und Strukturteilung verbinden. Löschung eines Clusters müsste Strukturen fremder Cluster berühren. Branches innerhalb eines Clusters teilen nichts mit anderen Clustern: **Löscht der Cluster, sterben alle seine Branches mit** (kein Sonderfall im Löschprotokoll). Tamper-Evidenz liefern Ketten-MAC, Ledger und Anker; Merkle-Zustandshashes sind Sache der oberen Schicht (NZ-13).

### 8.2 Branch-Metadaten

Branch-Metadaten liegen **im Cluster**, nicht im Catalog: reservierter Space `SYS_BRANCH` (SpaceId 1, clustered, `DataClass::Personal`-Regeln wie der Cluster), Eintrag `branch_id → { name (≤ 64 B, opak), parent, fork_point: Head|At(Stamp), state: Live|Merged|Dropped, ttl, created_by }`. Für `Head` ist der MVCC-Stamp des Branch-Erstellungsrecords der `fork_stamp`; der Stamp wird nicht zusätzlich im versiegelten Payload dupliziert. Der Catalog bleibt frei von Namen und Nutzdaten. Eine In-Memory-Tabelle je Cluster (Blatt in `cluster_table`) wird beim ersten Zugriff aufgebaut.

### 8.3 Lesen auf einem Branch

Sicht `(V, b)`: Für einen Schlüssel gilt die neueste Version in `b` mit Stamp ≤ `V`; sonst rekursiv in `parent(b)` mit Stamp-Obergrenze `min(V, fork_stamp(b))`. `Delete`/`RangeDelete` in `b` verdecken Elternversionen. Zusätzlich wird der clusterweite `EraseRange`-Maskensatz vor Versionsauflösung angewandt und kann durch keine Branch-Version oder Merge-Resolution aufgehoben werden. Umsetzung: k-Wege-Merge mit je Quelle einer **Stamp-Kappe**. Filter und `BranchTable` sind branch-bewusst (§7.1): Punktlesen läuft als Chain-Resolve (Blatt → Wurzel, Early-Exit), Bereichslesen als k-Wege-Merge nur über Quellen mit Treffern im Bereich.

### 8.4 Fork, Drop, TTL

```rust
impl Storage {
    pub fn fork(&self, c: ClusterId, parent: BranchId, at: ForkPoint, o: BranchOptions) -> Result<BranchId, Error>;
    pub fn branch_drop(&self, c: ClusterId, b: BranchId) -> Result<(), Error>;                 // logisch, nur leaf
}
impl Eraser {
    pub fn branch(&self, c: ClusterId, b: BranchId, req: EraseRequest) -> Result<Ticket, EraseError>;   // Logical | Physical | Scrubbed; EraseRequest { request_id, level, deadline } wie §9.2
}
pub enum ForkPoint { Head, At(Stamp) }
pub struct BranchOptions { pub name: Bytes, pub ttl: Option<Duration> /* spekulative Branches */ }
```

Fork linearisiert als Op im Sequencer; `BranchId` (u32) wird zuvor aus dem `reserve_ids`-Zähler (§5.8) durable reserviert, ein u32-Überlauf ergibt `Err(Exhausted)` und nie einen Wrap (`INV-BR-4`); SYS_BRANCH wird vor Rang 1 versiegelt. Bei `Head` setzt der äußere Frame-Stamp den `fork_stamp`. Kosten O(1), keine Nutzdatenkopie. `branch_drop` ist nur für leaf-Branches zulässig; ein Branch mit lebenden Kindern ergibt `BranchHasChildren`. TTL-Expiry folgt derselben Regel bottom-up. Erfolgreich gemergte Sources sind terminal/read-only, bleiben aber lesbar bis zu ihrem `branch_drop`. Branch-Drop macht nur die Sicht dieses leaf-Branches unsichtbar; physisch verschwinden Einträge bei Compaction. Für Sealed-Branches gibt es kein isoliertes Crypto-Erase; sofortiges Crypto-Erase löscht den ganzen Cluster. v1 erlaubt höchstens `max_branch_depth` Branch-Ebenen (Default 4, hart 8, §4.6); aktive Branch-Anzahl ist pro Cluster konfigurierbar und durch das gemeinsame MemoryBudget begrenzt. `gc_floor` ist das Minimum aller aktiven View-/Hold-/Consumer-Stamps und Fork-Stamps lebender Branches; ohne Pins gilt `durable_stamp + 1`. `history_floor` ist der publizierte untere Rand tatsächlich lesbarer Historie, wird erst nach erfolgreichem GC/Compaction fortgeschrieben und darf nie größer als `gc_floor` sein. `At(S)` verlangt `history_floor ≤ S ≤ parent_head`; `At(S < history_floor)` ergibt `BelowFloor` und wird nicht auf den Head umgedeutet.

### 8.5 Diff und Merge

- **Diff** `(view_a, view_b, cluster)`: Menge der Schlüssel, deren sichtbare Version sich unterscheidet. `ChangeIndexBlock` enthält cluster-sealed `(stamp, branch, op_uid, kind, key/range)`-Einträge einschließlich Punkt-, `RangeDelete`-, `EraseRange`- und Branch-State-Mutationen, sortiert nach `(stamp, op_uid)`. Vor dem Flush kommen dieselben Index-Records aus Memtable/Redo-Overlay; Recovery rekonstruiert sie aus Redo. Flush/Compaction übertragen sie verlustfrei in Runs. Diff vereinigt Kandidaten seit dem gemeinsamen Floor aus beiden Sichten, löst Sichtbarkeit auf und vergleicht Endzustände; Kosten ∝ Mutationen seit diesem Floor + Ausgabe, nicht pauschal ∝ Branch-Einträge. `branch_diff(branch)` relativ zum ForkPoint scannt nur Branch-Mutationen. Records bleiben gepinnt, solange Views/Branches/Merges sie benötigen, und werden erst unter `gc_floor` kompaktierbar. Das Layout ist G-F-Bestandteil.
- **Merge `source → target`** ist v1 nur für einen direkten Child-Branch erlaubt (`parent(source) == target`), wodurch `merge_base = fork_stamp(source)` eindeutig ist. Source mit lebenden Kindern, geschlossene/retired Cluster oder veraltete Heads ergeben einen Fehler ohne Mutation. In drei Schritten:
  1. **Berechnung:** `K` = Punkt-/Intervallmutationen aus `source` seit `fork_stamp`, ermittelt über den ChangeIndex. Für jeden Effekt prüft der Merge per Target-ChangeIndex, ob `target` seit `merge_base` (= `fork_stamp`) eine überlappende Punkt- oder Bereichsänderung trägt. Gleiche Endzustände (Kopf+Körper bytegleich sowie gleiche Löschsemantik) sind **kein** Konflikt.
  2. **Ergebnis:** `Merged(Receipt)` oder `Conflicts(Vec<ConflictCell>)`; **bei Konflikten wird nichts geschrieben.** `ConflictCell { key, base, ours, theirs }` sind Daten der Aufrufer-Schicht, nicht persistiert.
  3. **Anwendung:** Die Konfliktlösung wird als vollständige Ops-Menge (`Put | Delete | RangeDelete`; `EraseRange` bleibt unwiderruflicher clusterweiter Gate-Marker und kann nicht als Zielwert gewählt werden) in einem normalen `Change` auf `target` mit Punkt- und Range-Guards seit `merge_base` kodiert. Der Merge-Frame bindet `source`, `source_head_stamp`, `target_head_stamp` und `merge_base`; Commit revalidiert alle vier und setzt Source-Status `Merged(at stamp)` im selben durablem Frame. Alles atomar; bei Konflikt/Head-Drift wird nichts geschrieben.
- **Core-Modi:** `FailOnConflict`, `PreferSource`, `PreferTarget`. **Typisierte Strategien (LWW, Union, Summe, Textmerge, semantischer LLM-Merge) liegen oben** und liefern die Auflösung; Tag `Merge=6` bleibt reserviert (NZ-4).
- **Idempotenz:** Merge trägt einen `IdemToken`.

### 8.6 Zeitmodell

Stamp = Transaktionszeit. Gültigkeitszeit, Beobachtungszeit, Quellzeit leben im Kopf der oberen Schicht; `supersedes`-Kanten sind oben (§12.6). „Was wusste der Agent bei Stamp S?“ = `view_at(S)`; „Was galt am Datum t?“ = Filter oben. Zeitprädikate über Gültigkeits-/Beobachtungszeit dürfen Blöcke nur über den Erweiterungsvertrag `BlockPruner` (§12.2) überspringen, nie über Kernformat-Felder (K-29): das Überspringen ist nur für schattensichere Quellen korrekt, und diese Zusicherung kann allein die obere Schicht geben.

---

## 9. Löschung

### 9.1 Prinzip

Löschen ist die primäre Strukturierungsachse. Drei Schritte, unabhängig und geordnet: **Gate** (logisch, Millisekunden) < **Schlüssel** (kryptografisch) < **Purge** (physisch, bis zur Frist). Der Kern sagt nie mehr, als die erreichte Stufe trägt.

### 9.2 Stufen (zwei Achsen, K-23)

Jeder Löschauftrag (Cluster, Range, Branch oder Space) verlangt einen zufälligen `EraseRequestId`, der unabhängig vom 24-h-`IdemToken` ist. Das Ledger persistiert nur `request_id_hmac` und kanonischen `request_digest`; identischer Retry gibt dasselbe Ticket/Ergebnis zurück, abweichender Scope mit derselben ID ergibt `IdemConflict`. Die Zuordnung bleibt so lange erhalten wie Ticket-/Ledger-Evidence.

| `EraseLevel` (Ziel) | Für | Verfahren | Schützt gegen | Schützt nicht gegen |
|---|---|---|---|---|
| `Logical` | beide | Gate | Lesen über die API | Inhalt auf Platte |
| `Physical` | `Plain` | Gate, Barriere, Purge, Edit, Unlink, `sync_dir` | Lesen aus lebenden Dateien, Wiederkehr nach Absturz (mit Anker) | Journal, Flash-Reste, Backups, Hardlinks |
| `Scrubbed` | `Plain` | wie `Physical` + Überschreiben vor Unlink | einfache Wiederherstellung klassischer Datenträger | CoW-Dateisysteme, SSDs (best-effort) |
| `Cryptographic` | `Sealed` | Gate, **alle für den Scope erforderlichen Cluster-/Extension-Schlüssel vernichten**, Purge/Verify | verschlüsselte Reste in Kern und registrierten Owners/Backups, soweit jede Kopie unter der zerstörten Scope-Key-Domäne liegt | gemeinsam geschlüsselte Extension-/Backup-Kopien, Schlüsselkopien, unregistrierte Derivate |

| `Stage` (Fortschritt) | Bedeutung | Garantie (alt) |
|---|---|---|
| `Gated` | Gate durabel, Cluster `Retired(S)` | G0 |
| `KeyDestroying{eta}` / `KeysDestroyed` | Schlüssel unbenutzbar / `Absent` | G1 |
| `Purging` / `Purged` | Bytes aus lebenden Dateien entfernt | G2 |
| `Done` | Ledger durabel, Anker fortgeschritten, Receipt | — |

Ein Cluster-Ticket **endet** nur auf der zur Schutzklasse passenden Stufe: **`Sealed` ⇒ `Cryptographic`, `Plain` ⇒ `Physical`/`Scrubbed`**; jede andere Endstufe ⇒ `Err(Unsupported)`. `Logical` ist (a) Zwischenziel eines Cluster-Tickets, das per `escalate` auf die Endstufe gehoben wird, und (b) zulässiges Endziel für `range`, `branch` und `space_drop` (§9.8). Ein Cluster-Ticket, das nur `Logical` erreicht hat, ist `CompleteWithResidual{LogicalOnly}` (nie `Complete`). Das Receipt spricht nie von „unwiderruflich“ oder „konform“.

### 9.3 Garantie-Kernsätze

1. Ab durablem Gate liefert kein `view`/`page`/`poll` Inhalt des Clusters und kein Commit schreibt hinein; pro Cluster gilt voller Snapshot oder `Erased`, nie Mischsicht (`INV-ERA-1`).
2. Nach `KeysDestroyed` ist alles des Clusters innerhalb des Kernes in Redo, Runs, Blobs und gespeicherten Checkpoints ohne Cluster-Key unlesbar (`INV-ERA-7`). Eine Extension-/Backup-Kopie ist nur dann von dieser Aussage umfasst, wenn ihre registrierte Manifest-Key-Scope die betroffenen Kopien nachweislich an dieselbe Cluster-Key-Vernichtung bindet; andernfalls bleibt der Gesamtsystem-Receipt `Incomplete`.
3. Nach `Purged` enthält keine lebende Datei Bytes des Clusters (`INV-ERA-2`).
4. Gate/KeyDestroy-Arbeit skaliert mit der Zielmenge (zuzüglich Provider-Latenz). Physischer Purge bearbeitet nur Dateien/Extents mit Zielcluster; seine Byte-Kosten sind durch deren Zahl und Größe sowie Retention-Pins begrenzt, nicht generell store- oder clustergrößenproportional. `ClusterLocal` vermeidet fremde Survivor-Rewrites in Runs/Blobs, nicht zwingend im gemeinsam genutzten Redo. Externe globale Rebuildable-Indizes haben eigene Frist/Kosten und werden separat ausgewiesen.

### 9.4 Schlüssel und `KeyProvider`

```
KEK (KeyProvider, extern)  └─ RootKey (32 B, in KEYS mit KEK umhüllt)
       ├─ Space-DEK (je Space, mit RootKey umhüllt)
       ├─ Redo-/Catalog-/FORMAT-/Ledger-Schlüssel = KDF(RootKey, Label)
       └─ LedgerScopeKey = KDF(RootKey, "ctxs ledger scope v1") (nur AEAD für minimierte Selector-Ciphertexts)
Clusterschlüssel: KeyProvider::cluster_key(cluster, epoch)      (NICHT in KEYS)
```

```rust
pub trait KeyProvider: Send + Sync {
    fn info(&self) -> KeyProviderInfo;
    fn kek(&self, id: KekId) -> Result<Key32, KeyError>;
    // E-02/E-08: Batch ≤ 256, idempotent je BatchId, Teilresultate, jeder gemeldete Erfolg vor Rückgabe durabel
    fn create_cluster_keys(&self, request_id: BatchId, cs: &[ClusterId], e: &dyn Entropy) -> Result<Vec<KeyCreateResult>, KeyError>;
    fn cluster_keys(&self, req: &[(ClusterId, u32)]) -> Result<Vec<KeyLookup>, KeyError>;     // ≤ 256; Missing ≠ Shredded
    fn destroy_many(&self, cs: &[ClusterId]) -> Result<DestroyReceipt, KeyError>;              // ≤ 256 je Aufruf (der Eraser chunkt die Hülle bis 4096 und ruft parallel auf), ALLE Epochen, idempotent
    fn probe(&self, c: ClusterId, epoch: u32) -> Result<KeyProbe, KeyError>;                   // Present | Disabled | Absent
    fn list(&self) -> Result<Vec<KeyListEntry>, KeyError>;      // KeyListEntry { cluster: ClusterId, created_utc: u64 /* UTC-Sekunden */ }
    // Tomb-Epochenschlüssel (K-31) für Ledger-Pseudonyme; nicht in KEYS, nicht in Kern-Backups
    fn create_tomb_epoch(&self, epoch: u32) -> Result<(), KeyError>;                           // idempotent, durabel
    fn tomb_key(&self, epoch: u32) -> Result<Option<Key32>, KeyError>;                         // None ⇒ retired
    fn retire_tomb_epoch(&self, epoch: u32) -> Result<KeyProbe, KeyError>;                     // idempotent; Absent bestätigt
    // Partitionsgenerationsschlüssel für geteilte Indexstrukturen (§12.4.5); nicht in KEYS, nicht in Backups
    fn create_partition_key(&self, partition: PartitionTag, generation: u32) -> Result<(), KeyError>;   // idempotent, durabel
    fn partition_key(&self, partition: PartitionTag, generation: u32) -> Result<Option<Key32>, KeyError>; // None ⇒ vernichtet
    fn destroy_partition_key(&self, partition: PartitionTag, generation: u32) -> Result<KeyProbe, KeyError>; // idempotent; Absent bestätigt
}
pub struct KeyProviderInfo { pub destroy: DestroyMode, pub keys_in_backups: KeysInBackups, pub assurance: KeyAssurance }
pub enum DestroyMode { Immediate, Scheduled { max_delay: Duration } }
pub enum KeysInBackups { No, Bounded { until: Duration }, Yes }
pub enum KeyAssurance { Hardware, Kms, FilePassphrase }
```

- **`Disabled`** = unbenutzbar, nicht unumkehrbar (KMS-Karenz). ER2 gilt erst bei `Absent` als abgeschlossen; bis dahin `KeyDestroying{eta}` und Ticket/Receipt `Blocked` mit nächstem Prüfzeitpunkt und KMS-Restrisiko. Bei `Cryptographic` beginnt der Purge erst bei `Absent` (`INV-ERA-3`, §9.5); solange der Schlüssel nur `Disabled` ist, sind lediglich Planung und Scratch-Reservierung (ER3) zulässig. `Done` braucht ebenfalls `Absent`.
- **`KeysInBackups` (E-77):** `No` = kein Schlüsselexport außerhalb des Providers. `Bounded{until}` = Exporte existieren (KMS-Backup, Escrow), werden aber spätestens `until` nach der Vernichtung selbst vernichtet. `Yes` = unbegrenzt. `Cryptographic` ist bei `No` voll zulässig, bei `Bounded` zulässig mit Statuscap `CompleteWithResidual{BoundedBackupKeys{until}}`, bei `Yes` `Err(Unsupported)`. Das Receipt wird aus Ledger und Zeit berechnet: nach Ablauf von `until` entfällt der Cap bei einem neu berechneten Receipt, solange der Provider weiter `Absent` bestätigt. Das Receipt nennt `destroy`, `keys_in_backups` und `assurance`.
- **`KeyAssurance` (E-77):** `Hardware` (nicht exportierbarer HSM-/TPM-Schlüssel mit Vernichtungsnachweis des Moduls), `Kms` (Remote-KMS mit Destroy-API, Karenz möglich ⇒ `Disabled`), `FilePassphrase` (`keys-file`). `FilePassphrase` kann die Vernichtung auf Journal-, CoW- und Flash-Dateisystemen nicht beweisen: jedes Receipt mit diesem Provider trägt `CompleteWithResidual{WeakKeyAssurance}`, `Complete` ist ausgeschlossen. `open` mit `OpenProfile::Production` (§16) weist Spaces mit `DataClass::Personal` und `Seal::Cluster` bei einem Provider mit nur `FilePassphrase` mit `ProviderAssuranceInsufficient` ab, außer `acknowledged_weak_keys` ist gesetzt.
- **Tomb-Epochen (K-31, E-74):** `tomb_key(epoch)` leitet ausschließlich `tomb_id`s ab (§9.7). Tomb-Schlüssel liegen im Provider, nie in `KEYS`, Checkpoints oder Backups des Kerns (`INV-KEY-5`). Default: genau eine Epoche `0`, keine Rotation, kein Retire (`tomb_id` bleibt linkbar). Optional rotiert `rotate_tomb_epoch` auf eine neue Epoche für neue Intents (die alte wird `Closed`), und `retire_tomb_epoch` vernichtet den Schlüssel einer geschlossenen Epoche unter den Bedingungen von §10.4. `tomb_key` liefert `None` ausschließlich für retired Epochen; fehlt der Schlüssel einer nicht retired Epoche, ist das `KeyLost` der Tomb-Domäne: Restore bleibt quarantined, neue Erase-Intents liefern `KeyUnavailable`, nie wird ein fehlender Tomb-Schlüssel als Retire gedeutet. Für Tomb-Schlüssel gelten `keys_in_backups` und `assurance` des Providers sinngemäß.
- **Partitionsgenerationsschlüssel (K-49):** `PartitionTag` ist ein opaker, zufälliger 16-Byte-Bezeichner je Index-Partition (keine ClusterId, kein Klartext-Mandant). Die Schlüssel liegen im Provider, nie in `KEYS`, Checkpoints oder Backups (`INV-KEY-5` gilt sinngemäß); `keys_in_backups` und `assurance` des Providers gelten auch für sie. `Absent` für Generation `g` ist Voraussetzung für `Done` des Index-Owners (`INV-PART-1`); ein fehlender Schlüssel einer nicht vernichteten Generation ist `KeyLost` der Index-Domäne (Partition neu aufbauen, Ticket `Incomplete`), nie still `Absent`.
- **ER2-Abschluss je Epoche:** `KeysDestroyed` erst, wenn `probe(c, e) == Absent` für jede Epoche `e ≤ key_epoch` des Clusters gilt (`INV-KEY-4`).
- KEK-Rotation: nur RootKey neu umhüllen (`rotate_kek`), alle anderen Dateien bleiben. Rotation einer Cluster-Epoche: neue `key_epoch` für neue Writes; eine alte Epoche wird erst vernichtet, wenn nichts mehr mit ihr versiegelt ist (`INV-KEY-3`); `destroy_many` vernichtet **immer alle** Epochen (`INV-KEY-4`).
- **Cohort-Strategie (Provider-intern, K-04):** `keys-cohort` darf Clusterschlüssel in Cohorts (≤ 1024) unter einer Cohort-KEK-Generation führen; `destroy_many` = Rewrap ohne die gelöschten Einträge, dann KMS-Destroy der alten Generation. Voraussetzung: **keine Keyring-Kopie in Backups** (`keys_in_backups = false`), Pflicht-Replikation des aktuellen Keyrings in ≥ 2 Failure Domains. Der Kern sieht davon nichts.
- Schlüsselcache mit Generationszähler (§11.5), TTL, `Zeroizing`, Providerausfall: gecachte Schlüssel bis `key_grace` (10 min); Gate und ER2 invalidieren sofort durch `gen`.
- Konformanzsuite `KeyProviderConformance` (Idempotenz, alle Epochen, `probe`-Semantik je Epoche, Crashverhalten, Tomb-Epochen: `create`/`retire` idempotent, `tomb_key == None` nur nach Retire, Provider-Neustart verliert keine Epoche) ist Gate-Pflicht für jeden Provider.

### 9.5 Protokoll `Eraser::cluster` (ER0–ER7)

```
ER0 Vorprüfung:  Cluster existiert; Link-Hülle H = {c} ∪ transitiv DerivedFrom-gebundene Cluster;
                 BLOCK_ERASE-Restriction in H ⇒ Err(Restricted{ids,reasons}); level passt zur Protection; Provider erreichbar
                 und info() erlaubt Cryptographic; bereits Retired ⇒ vorhandenes Ticket (ggf. escalate)
ER1 Gate:        Sequencer-Op: Gate-Frame{scope=Clusters(H), ticket, level}, Gruppen-Fsync          ◄── LOGISCHER Punkt (G0)
                 Der Aufrufer hat Hülle und Restrictions gegen `policy_epoch` P1 berechnet (INV-ERA-10); der Sequencer vergleicht nur `policy_epoch == P1`, Abweichung ⇒ Retry ab ER0. Nach `policy_retry_max` (3) berechnet der Sequencer die Hülle einmalig selbst (nur RAM, ≤ `max_gate_clusters` Knoten): eine Löschung wird nie von Link-/Restrict-Verkehr ausgehungert.
                 Im Sequencer: cluster_table ← Retiring{S} (schließt nur die Schreibadmission); nach durablem Sync und Publikation Retired{S}, gen++. erase() kehrt mit Ok(ticket) zurück; ab hier nie mehr ein Fehler.
                 Asynchron (urgent): Memtable-Einträge freigeben/überschreiben, Cache-Puffer überschreiben,
                 Catalog-Edit{cluster_state(H→Retired,S), erase_pending_add}, Ledger `Retired`.
ER2 Schlüssel:   (Cryptographic) destroy_many(H); **nur** bei `probe == Absent` KeysDestroyed; Cache leeren, gen++;
                 Edit{cluster_state → Shredded} bei probe==Absent                                   ◄── KRYPTO-Punkt (G1)
ER3 Plan:        Dateien über Packgruppen-Index und ClusterDirectory (Staging eingeschlossen); Claims; Segmente mit Überlappung
ER4 Barriere:    NUR für Plain und Space-Drop: Segmentschnitt, Memtables einfrieren und flushen, replay_floor und retain_floor
                 hinter den Schnitt, ältere Segmente ⇒ Graveyard. Eine Barriere bedient alle fälligen Plain-Tickets.
ER5 Umbau:       PurgeBatchEdit je Datei/Blob-Inkarnation, Scratch-Slot wieder freigeben; stage bleibt Purging
ER6 Abschluss:   nach Verify + Extension-Acks ein Edit{cluster_state(H→Purged), erase_done, anchor_mark} ◄── PHYSISCHER Punkt (G2)
ER7 Danach:      Ledger `Done`, Anker fortschreiben, ErasureReceipt; allgemeine Graveyard-Dateien
                 ohne Zielnutzdaten werden separat nach ihren eigenen Reader-Pins entfernt
```

**Reihenfolge ist Vertrag:** ER1 < ER2 < jedes `PurgeBatchEdit` < zugehöriger Unlink < finales ER6/`Done`; bei `Cryptographic` beginnt physischer Purge nie vor `probe == Absent` (`INV-ERA-3`). Jede Batcherweiterung ist crash-sicher und roll-forward; nach finalem Verify gibt es keine lebende Datei/Inkarnation mit H. Das Gate liegt im Sequencer, weil ein vor dem Gate geprüfter Change nach dem durablen Gate nicht sichtbar werden darf. Es schneidet kein Segment: der Commit-Pfad bleibt frei von Löschkosten.
**Frist:** `purge_deadline` ist **Pflichtparameter** (kein Standard). Ticket speichert Deadline, monotone Dauer und `clock_floor`. Rücksprung wird geklemmt; ein UTC-Sprung außerhalb `max_clock_skew` friert Retention/Deadline-Berechnung ein und ergibt `ClockUncertain` mit Alarm, statt Holds vorzeitig zu beenden oder Compliance-Frist zu simulieren. Überfällige, zeitlich verifizierte Tickets erhalten `urgent`; echte Überschreitung meldet `erase_deadline_breaches_total` und `Blocked{reason=Deadline}`.
**Bündelung und Amortisierung (K-36):** Purge-Auslöser (das Erste, was eintritt): (a) **Compaction-Huckepack:** jede Compaction/Blob-GC, die eine Datei ohnehin umschreibt, lässt `Retired`-Cluster weg und erfüllt deren Purge ohne Zusatz-I/O (Standardfall); (b) Frist zu 70 % verbraucht; (c) ≥ `purge_batch_min` (16) Cluster **in derselben Datei**. Eine Datei wird je Fenster `W = purge_window_fraction × purge_deadline` (0,3) höchstens einmal umgeschrieben: alle in `W` anfallenden Löschungen der Datei teilen sich einen Rewrite. Obere Schranke der Purge-Bytes je `W` = Σ Größen der betroffenen Dateien ≤ 1 × Storegröße; die Kosten einer einzelnen Löschung sind damit amortisiert statt „eine 64-MiB-Datei je Kleinstcluster“. **Dringlichkeit:** `Plain` ⇒ Lane `urgent` (Klartext). `Sealed` ⇒ Lane `compact`, Eskalation auf `flush` bei 70 % und auf `urgent` bei 90 % der Frist (`sealed_purge_escalate`): nach `KeysDestroyed` liegt nur unlesbarer Ciphertext vor, die Hygiene darf die Wartungsbandbreite (`maint_io_fraction`) nicht verdrängen. Das Ticket bleibt bis zum Purge `Purging`; Receipt-Zeitpunkte `t_crypto` (G1) und `t_physical` (G2) werden getrennt ausgewiesen.
**Views:** `ViewPolicy::Wait(timeout)` (Unlink wartet; logische Garantie gilt bereits) oder `Revoke` (§11.4).
**Done-Bedingungen (`INV-ERA-8`):** (1) keine lebende Datei/Segment/Inkarnation trägt Bytes von H, auch keinen Ciphertext (bei Cryptographic ist Unlesbarkeit notwendig, aber nicht hinreichend), (2) Memtable-Einträge freigegeben, (3) Ledger durabel, (4) Anker fortgeschritten, (5) alle Segmentbesitzer bestätigt, (6) bei Cryptographic `probe == Absent`.

#### 9.5.1 `purge` (der einzige Dateilöschalgorithmus)

```
purge(plan: Dateien mit Claims, D: Dropmenge):
  für jede Datei/Blob-Inkarnation F einzeln:
     keep = Cluster(F) − D
     keep = ∅  → PurgeBatchEdit{files_remove(F)}  // files_remove legt Graveyard-Eintrag an
     sonst     → F' in vorallokiertem Scratch-Slot roh kopieren/repacken;
                 sync_all, rename, sync_dir;
                 PurgeBatchEdit{files_add(F'), files_remove(F)} // files_remove legt Graveyard-Eintrag an
  nach Reader-Drain: unlink(F); sync_dir; durables unlink_ack(F); erst dann Scratch-Slot auffüllen
  nach allen Dateibatches: SegmentOwner attestieren; finaler PurgeCompleteEdit setzt Purged/Done
```

`files_remove`/`graveyard_add` sind eine einzige Catalog-Edit-Operation, nicht zwei unabhängige Writes. Crash vor Unlink lässt den Graveyard-Eintrag wiederholbar; Crash nach Unlink, aber vor `unlink_ack`, wird beim Recovery als bereits fehlende Datei idempotent bestätigt. `PurgeCompleteEdit` ist unzulässig, solange ein Ticket noch Graveyard-Dateien oder offene `unlink_ack`s besitzt.

Rohkopie prüft Frame-CRCs, nie das AEAD-Tag (kein Schlüssel nötig). Blob-Records sind dank positionsunabhängigem AAD roh kopierbar; bei Repack wird nur Struktur/Directory neu gebaut. Pro Einzelschritt ist Scratch ≤ eine maximale Datei (Run/Blob/Redo, je ≤64 MiB); zwei Slots decken laufenden Rewrite plus Crash-Recovery. Vollständig tote Dateien benötigen keinen Scratch. Kosten bleiben proportional zu betroffenen begrenzten Dateien/Packgruppen, nicht generell zum Einzelcluster oder Store. Gleicher Algorithmus für Packgruppen-Abwurf (`D` = alle Cluster der Gruppe **ohne** Restricted-Cluster), Mandantenabgang (`D` = alle Cluster des `tenant_tag`), Staging-Dateien.

#### 9.5.2 Löschauftrag auf vergiftetem Handle (E-75)

Ein Handle im Zustand `Poisoned` (F-3) nimmt keinen Gate-Frame mehr an; die Löschpflicht endet dadurch nicht. Der Kern nimmt deshalb die **Löschabsicht** über das Ledger auf, das einen eigenen Handle, eine eigene Failure Domain und genau einen Schreiber (Lane `urgent`) hat:

1. **Aufnahme:** `Eraser::*` und `Storage::space_drop` prüfen Request und Restrictions gegen den RAM-Zustand (auf einem vergifteten Handle kann kein `restrict`/`lift` mehr durabel werden, der Zustand ist final). Danach hängt die Fassade einen kanonischen `EraseIntent{origin: Poisoned, …}` an (Format wie §9.7, `tomb_epoch` aus dem gecachten Tomb-Schlüssel; kein Providerzugriff nötig) und liefert `Ok(Ticket{state: IntentRecorded})`, **nicht** `Gated`. Schlägt der Append fehl (Ledger-Handle ebenfalls vergiftet, Tomb-Schlüssel nicht verfügbar), lautet das Ergebnis `Err(Poisoned{intent: NotRecorded})`; der Kern behauptet dann keine Aufnahme.
2. **Logische Wirkung sofort:** Die Fassade führt eine RAM-Menge `pending_erase`; Lesen, Views, `page`, `poll` und Feed-Zugriffe auf diese Cluster liefern `Err(ErasePending)`, nie Daten, nie „leer“. Cache-Puffer dieser Cluster werden überschrieben (`Zeroizing`). Das ist kein Gate: es gibt keinen `erase_epoch S` und kein `Erased{at}`.
3. **Neustart:** `open` löst zuerst `Indeterminate`-Changes auf (OP6/OP7), dann überführt OP8 jeden `EraseIntent` ohne `GateApplied` idempotent in ein Gate (Roll-forward, `INV-ERA-4`), vor jeder Nutzer-Sicht. `origin: Poisoned` ändert die Folgestufen nicht; ER2…ER7 laufen unverändert.
4. **Keine Abkürzung:** Auf dem vergifteten Handle gibt es kein `KeyDestroy`, keinen Purge und keine Anker-Fortschreibung: `INV-ERA-3` verlangt Gate und Anker vor der irreversiblen Stufe. Ein vernichteter Schlüssel ohne durables Gate würde bei Redo-Recovery als `KeyLost` erscheinen.
5. **Idempotenz:** `EraseRequestId` wie in §9.7; ein Retry nach Neustart liefert dasselbe Ticket.
6. **Ehrliche Grenze:** Zwischen Aufnahme und Neustart bleiben Klartext-Reste im Prozessspeicher (Memtable, Allocator) bis zum Prozessende. Der Observer meldet `poisoned_erase_pending` (Alarm, Neustart empfohlen); das Receipt weist `t_requested` aus dem Intent und die Wartezeit bis zum Gate aus. Der Kern stößt den Neustart nicht selbst an.

Tests: `FaultFs` mit `EIO` auf Redo/Catalog gefolgt von Erase-Request (Ledger gesund und Ledger ebenfalls vergiftet), Crash zwischen Intent und Neustart, Doppelaufnahme, Restriction-Zustand (`INV-ERA-13`).

### 9.6 Restriction, Hold, Links

- **Restriction:** `RestrictionSpec { reason u16 ≠ 0, owner, effects (BLOCK_ERASE immer | BLOCK_WRITE | BLOCK_READ), expires }`. Ablauf lässt nur den Vermerk fallen, löst nie Löschung aus. **Kein Override im Kern.** `lift` nach `LiftPolicy`: `OwnerOrAdmin` (Standard) oder `FourEyes` (Port `Authority`, Signatur einer Principal ≠ Requester). `restrict`/`lift` laufen als Control-Frame durch den Sequencer, in derselben Stamp-Reihenfolge wie das Gate (`INV-ERA-10`). Durchsetzung dreischichtig: Prüfung auf dem Aufruferthread mit Revalidierung im Sequencer (ER0/ER1), Purge verweigert `files_remove`/Umbau für Cluster mit BLOCK_ERASE, `cluster_state`-Edit verweigert den Übergang nach `Retired` für Restricted-Cluster. Ein Erase blockiert **sofort** mit `Err(Restricted)` (kein Zwischenzustand); will die obere Schicht den Cluster währenddessen verbergen, setzt sie selbst `BLOCK_READ`.
- **`BLOCK_READ`:** Lesen ⇒ `Err(Restricted)`; zweckgebundene Lesung nur mit nicht klonbarer `RestrictedReadCap` (Observer-Ereignis mit Zweck-Code, nie in Cache oder Consumer-Feed).
- **Hold:** technisch, befristet (`ttl ≤ 24 h`); ein Erase überstimmt einen Hold ohne Fehler.
- **Links:** `Clusters::link(derived, sources[≤ 8])` (azyklisch, Tiefenlimit). Die Storage-Schicht begrenzt die transitive `DerivedFrom`-Erase-Hülle auf `max_gate_clusters=4096`: Ein Link ist ein **Control-Frame `Link`** und damit im Sequencer total mit Gate, Restrict und Lift geordnet (ein Catalog-Edit außerhalb der Stamp-Ordnung ließe das Race „Quelle gelöscht, Link danach angelegt“ zu und verletzte `INV-ERA-12`). Admission außerhalb Rang 1 gegen `policy_epoch` P1: (a) azyklisch, (b) `max_link_depth` (8), (c) für **jeden Vorfahren-oder-selbst** `a` jeder Quelle gilt `|Hülle(a) ∪ Hülle(derived)| ≤ max_gate_clusters` (die Hülle eines Vorfahren wächst mit jedem neuen Nachfahren; eine Prüfung nur am neuen Kind genügt nicht), (d) keine Quelle ist `Retiring`/`Retired` (sonst `ClusterRetired`). Die Traversierung bricht bei `max_gate_clusters + 1` besuchten Knoten ab ⇒ `EraseFanoutExceeded` ohne Link-Mutation. Der Sequencer prüft nur `policy_epoch == P1` (sonst Retry) und „Quelle `Live`“, setzt die Kante und erhöht `policy_epoch`; keine Graph-Arbeit unter Rang 1. Dadurch passt jede zulässige Kaskade in einen Gate-Change. Löschung irgendeines Quellclusters zieht den abgeleiteten **im selben Gate-Frame** mit (`INV-ERA-12`): es gibt keinen Zustand, in dem die Quelle gelöscht ist und ein Abgeleiteter lebt. Ein Kind mit mehreren Eltern wird konservativ gelöscht, sobald **eines** seiner Eltern gelöscht wird. Die fachliche Cross-Subject-Prüfung gehört wegen NZ-16 in die Identity-/Policy-Schicht; sie muss vor Link-Commit sicherstellen, dass die betroffenen Löschdomänen übereinstimmen. Ein gemeinsam genutzter Cluster darf nicht als `DerivedFrom`-Kind einer einzelnen Person angelegt werden; pro Löschdomäne getrennte Ableitungen oder nicht-kaskadierende `Reference` verwenden.

### 9.7 Ledger, Anker, Receipt

- **Ledger:** append-only, vorallokiert, kettenverknüpft über `Mac`, **ein Schreiber**, der wartende Events zu Gruppen bündelt (ein `write` + ein `sync_data`). Erwartete Gate-Latenz ≈ `t_sync(Ledger) + t_sync(Redo-Pipeline)`; das ist die untere Schranke von H-6. Record ist ein kanonischer tagged union: `{seq, prev_hash, clock_sample, event, mac}`; keine Null-/Platzhalterfelder. Erase-Events enthalten nur `tomb_id = HMAC(tomb_key(tomb_epoch), "ctxs cluster tomb v1" || instance_id[16] || ClusterId[16])` samt `tomb_epoch u32`, Ticket, Level, Stamp/Stage und Evidence-Digest; niemals Klartext-ClusterId, SpaceId, Nutzdaten, Nutzerschlüssel oder Person-ID (`INV-LED-1`). Event-Kinds umfassen `EraseIntent`, `SpaceDropIntent`, `SpaceScopeFrozen`, `GateApplied`, `KeyDestroyIntent`, `KeysDestroyed`, `KeyLost`, `PurgeProgress`, `UnlinkProgress`, `ExtAck`, `PurgeVerified`, `Completed`, `RestrictionSet/Lifted`, `BranchEraseDone`, `SpaceDropped`, `RestoreReconciled`, `ReceiptIssued`, `TombEpochOpened`, `TombEpochClosed`, `TombEpochRetireIntent`, `TombEpochRetired`. Ein `EraseIntent` ist genau ein kanonischer, atomar appendbarer Datensatz mit höchstens 4096 sortierten `tomb_id`s (max. 256 KiB Record); selektive Range-/Branch-Selectoren sind als `scope_ciphertext` AEAD-versiegelt unter langlebigem `LedgerScopeKey`, AAD bindet Instance/Ticket/Digest. Ein `SpaceDropIntent` trägt nur `space_tomb_id` und Level; Count und Scope-Digest folgen nach dem Gate als `SpaceScopeFrozen`. `space_tomb_id = HMAC(ledger_key, "ctxs space tomb v1" || instance_id[16] || be_u64(SpaceId))`. Weder IDs noch Nutzdaten stehen im Ledger im Klartext; bei Restore berechnet der Matcher den Pseudonymwert über Catalog-Einträge und sperrt vor Zugriff jede passende SpaceId. Ledger-Scratch ist vorallokiert; ein Intent wird erst bei vollständigem validem Record plus Sync angenommen. Der Schlüssel bleibt außerhalb von Datenbackups und muss für Restore verfügbar bleiben. Klartext-ClusterId/SpaceId, Benutzerkeys und Nutzdaten stehen nie im Ledger. `request_id_hmac` und `request_digest` machen Retry exakt idempotent; gleicher Token mit anderem Digest ⇒ `IdemConflict`. Ein Gate ist erst zulässig, wenn sein versiegeltes Ledger-Intent durabel ist. Reihenfolge: Restriction-/Closure-Prüfung und Sequencer-Reservierung → einzelnes vollständiges Ledger-Intent durable → Gate-Catalog-Edit durable und In-Memory-Gate veröffentlichen → `GateApplied`-Ledger-Event → Anchor-CAS → erst dann KeyDestroy/Purge. Vor `destroy_many` steht ein durables `KeyDestroyIntent`; Recovery kann KMS-Ergebnis mit `probe` idempotent nachziehen. So kann weder ein altes Backup noch ein Crash ein angenommenes Gate ohne Tombstone wiederbeleben. Jeder Übergang ist crash-idempotent und vor irreversibler Folgestufe durabel. `ledger_dir` konfigurierbar (K-22).
- **Anker:** `AnchorValue = (erase_epoch, ledger_head, catalog_epoch)`. Der Anker wird nach durablem Gate-Catalog-Edit fortgeschrieben und **vor** `KeyDestroyIntent`/KeyDestroy oder Purge-Barriere muss die aktuelle Gate-/Ledger-Position verankert sein. Zusätzlich Fortschreibung bei Catalog-Rollover, `close` und spätestens `anchor_max_age` (1 s) bei ausstehendem Zustand; koalesziert. `Required` ist Standard bei Encrypted/Authenticated; wenn der erforderliche Anker vor einer irreversiblen Folgestufe nicht fortgeschrieben werden kann, bleibt das Ticket `Blocked` und diese Stufe beginnt nicht. `Weak`-Anker erfordert explizites `acknowledged_weak` und kann keinen starken Rollback-Schutz attestieren. Referenz: `MemAnchor` (Test), `FileAnchor` (`Weak`); `Strong`-Anker (TPM, Transparency-Log, Object-Lock) als getrennte Crates.
- **Scope-Digest:** Einzelcluster- und Batch-Erase-Digests sind HMACs über domain-separierte, sortierte Tomb-IDs plus kanonische Selector-Bytes. Beim Space-Drop wird die Mitgliedschaft **nicht** unter Rang 1 ermittelt (ein Einfrieren aller Cluster des Spaces wäre O(Cluster) im kritischen Pfad): Das Gate maskiert die gesamte SpaceId und schließt Writes in den Space, danach ermittelt ein Wartungsjob die Mitgliedschaft aus der zum Gate-Stamp gepinnten Version (O(Dateien), ohne Lock, nach dem Gate unveränderlich) und schreibt `SpaceScopeFrozen{count, scope_digest}` ins Ledger; `scope_digest = HMAC(ledger_key, "ctxs space scope v1" || instance_id[16] || be_u64(SpaceId) || level || count || sorted_tomb_ids)`. Die HMAC-Eingabe wird gestreamt; keine Klartext-ID-Liste kommt ins Ledger. `space_tomb_id` dient dem Restore-Gate, `scope_digest` dem Nachweis der eingefrorenen Mitgliedschaft.
- **Tomb-Epochen (K-31, E-74):** Jedes Event mit `tomb_id` trägt `tomb_epoch`; ein `tomb_id` wird nie ohne Epochenbezug interpretiert. Die Kette selbst (`prev_hash`, `mac`) nutzt `ledger_key` und bleibt von Tomb-Schlüsseln unabhängig; `verify_receipt` rechnet `tomb_id`s nur als Werte und braucht daher nie einen Tomb-Schlüssel. Nach `TombEpochRetired` ist kein `tomb_id` dieser Epoche mehr auf eine ClusterId abbildbar, auch nicht mit Kenntnis der ClusterId; der Ledger-Eintrag ist dann anonym (Ablauf und Bedingungen: §10.4). `space_tomb_id` bleibt unter `ledger_key`, weil SpaceIds nicht personenbezogen sind.
- **Ledger-Failure-Domain (K-22, E-81):** `OpenProfile::Production` verlangt, dass `ledger_dir` auf einem anderen Gerät liegt als das Datenverzeichnis (Vergleich der `device_ids` aus `HostProbe`, sonst Adapterangabe); andernfalls `Err(LedgerSameDevice)`, außer `ledger_same_device_ack` ist gesetzt (dann Restexposition im Receipt). Ist das Ledger beim schreibenden Öffnen nicht lesbar, lautet das Ergebnis `Err(LedgerUnavailable)`; `ReadOnly` ist nur mit Diagnose erlaubt, Gates aus dem Catalog gelten weiter.
- **Erase-Idempotenz:** `request_id_hmac = HMAC(ledger_key, "ctxs erase request id v1" || instance_id[16] || EraseRequestId[16])`. `request_digest = HMAC(ledger_key, "ctxs erase request digest v1" || canonical_erase_request)`, wobei `canonical_erase_request` ein versioniertes, tag- und längencodiertes Format mit sortierten eindeutigen Zielen, Erase-Level und semantischen Optionen ist. Deadlines und Retry-Metadaten sind nicht semantisch und bleiben außerhalb. Gleiche Request-ID und gleicher Digest liefern dasselbe Ticket/Resultat, gleiche ID mit anderem Digest ergibt `IdemConflict`. Die Ledger-Key-Epoche bleibt für die gesamte Ledger-Retention verfügbar.
- **`ErasureReceipt`:** `{instance_id, req/ticket, scope_digest (HMAC über kanonisch sortierte Tomb-IDs und Selector-Digest), level, t_requested, t_logical, erase_epoch S, t_crypto?, t_physical?, t_verified?, stages[{stage, ledger_seq, evidence_digest}], key_attestations, provider{destroy, keys_in_backups, assurance}, ext_acks, verifier_report_digest[MeasureKind je Messung, §13.2.1], ledger_head, anchor{strength, value}, restrictions_encountered, status (Complete | CompleteWithResidual | Incomplete | Blocked | Failed), residual[§9.9], canonical_bytes}`. `Complete` verlangt jede für den angeforderten Level erforderliche Kern- und Owner-Attestation. `CompleteWithResidual` bedeutet abgeschlossen mit explizit benannten, nicht vom Vertrag abgedeckten Restrisiken. `Incomplete` bedeutet, dass ein erforderlicher Owner-/Scope-Nachweis fehlt oder der Scope nur teilweise belegt ist; es ist niemals eine erfolgreiche Löschquittung. `Blocked` bedeutet ein sicher pausierter, fortsetzbarer Auftrag; `Failed` ist ein terminaler Integritäts-/Protokollfehler. **Statuscaps:** `BoundedBackupKeys{until}`, `WeakKeyAssurance`, `LogicalOnly` und deklarierte Host-/Gerät-Restexposition (§9.9) begrenzen `Complete` auf `CompleteWithResidual`; eine Pflichtmessung (§13.2.1) mit `NotMeasured` oder `NotImplemented` begrenzt auf `Incomplete` (`INV-VER-1`). **Die Signatur liegt oben (NZ-14);** der Kern liefert kanonische Bytes und die reine, offline lauffähige Funktion `verify_receipt(receipt, ledger_slice, anchor_attestation) → Verdict` (Kette, Anker, Stufenreihenfolge, Zeitmonotonie).
- **`Eraser::verify`:** strukturell, ohne Schlüssel: (a) keine lebende Datei/Segment/behaltenes Redo-Segment trägt ClusterDirectory-/Frame-Kopf-Eintrag für H, (b) Memtables ohne H, (c) bei Cryptographic `probe == Absent`, (d) Ledgerkette bis zum Anker gültig. Inhaltssuche ist Sache des Orakels.

### 9.8 Weitere Löschpfade

- **`Eraser::range(space, cluster, range)`:** keine Branch-Argumente; `EraseRange` ist ein clusterweiter Gate-Marker, der nach durablem Frame sofort alle Branches und auch ältere Views maskiert. Für `Plain` sind `Logical|Physical|Scrubbed` verfügbar; `Physical` rewritet alle Branch-/Stamp-Versionen des Bereichs aus lebenden Run-/Blob-Dateien. `Scrubbed` ist best-effort und kein SSD-/CoW-Beweis. Für `Sealed` ist ausschließlich `Logical` zulässig: selektive Range-Key-Vernichtung existiert in v1 nicht, und ein Rewrite allein macht Backup-Kopien mit demselben Cluster-Key nicht unlesbar. Für physische/kryptografische Entfernung eines Sealed-Bereichs ist der ganze Cluster zu löschen oder der Bereich vorab in eine eigene Cluster-Löschdomäne zu legen. `EraseRange` kann nie per Merge aufgehoben werden. `erased_ranges` dürfen erst getrimmt werden, wenn `history_floor > erase_stamp` und keine Branch-/View-/Consumer-Pins darunter liegen.
- **`Eraser::branch(cluster, branch)`:** nur terminale, leaf-Branches. Für `Plain` sind `Logical|Physical|Scrubbed` analog zu `range` möglich; physisch werden alle Versionen dieses Branches aus lebenden Run-/Blob-Dateien entfernt. Für `Sealed` nur `Logical`, da die Cluster-Keydomäne mit anderen Branches geteilt wird. Eine Crypto-Anforderung wird als Cluster-Erase ausgeführt; es gibt keine branchlokale Schlüsselvernichtung.
- **`Storage::space_drop(space, request_id, level, deadline)`:** Der Aufrufer prüft die RAM-Menge der `BLOCK_ERASE`-Restrictions (O(|R| × Dateien des jeweiligen Clusters), außerhalb Rang 1) darauf, ob ein eingeschränkter Cluster Daten im Space hat; jeder Treffer blockiert vor Mutation. Der Sequencer revalidiert `policy_epoch`, sodass keine zwischenzeitlich gesetzte Restriction übersehen wird. V1 unterstützt `Logical` für beide Seal-Modi und `Physical|Scrubbed` nur für Plain. `Cryptographic` auf Space-Ebene ist `Unsupported`: ein Space kann Cluster mit anderen Spaces teilen, und der Cluster-Key lässt keine sichere Teilvernichtung zu. Crypto-Erasure erfordert explizite Cluster-Tickets. Vor dem Gate wird `SpaceDropIntent{space_tomb_id,level}` durable ins Ledger geschrieben; `space_tomb_id=HMAC(ledger_key, "ctxs space tomb v1" || instance_id[16] || be_u64(SpaceId))`, ohne Klartext-ID oder Memberliste. Da SpaceId nie wiederverwendet wird, kann Restore jede alte Catalog-Kopie auf diese SpaceId matchen und ihre Sichtbarkeit vor Datenzugriff sperren; Scope-Digest/Count dienen dem Receipt-Abgleich, nicht als Ersatz für den Scope-Matcher. Nach Intent folgen `Gate{Space(id),intent_id}`, `GateApplied` und Anchor-CAS. Erst danach starten Physical-Folgestufen. `SpaceDropped` ist das Abschlussereignis nach `SpaceScopeFrozen` und allen erforderlichen Purge-/Extension-Attesten. Kein einzelner Catalog-Edit versucht beliebig viele Dateien umzuschreiben. Wiederanlage erfordert eine neue SpaceId.
- **System-Space-Lifecycle:** Öffentliche Drop- und Mutations-APIs akzeptieren keine `SpaceId < 256`; sie liefern `ReservedSpace`. Änderungen oder Entfernung von `SYS_BRANCH`/`SYS_IDEM` sind nur über versionierte interne Migrationen mit eigenem Recovery-Vertrag zulässig.
- **`pack_expired(PackKey)`:** `H` = nicht Restricted-Cluster der Packgruppe. Der Ablauf zerlegt H deterministisch in unabhängige Tickets mit höchstens `max_gate_clusters=4096`; jedes Ticket hat einen eigenen Gate-Frame und eigene Receipt-/Ledger-Kette. Es gibt keine behauptete atomare Gesamtlöschung der Packgruppe. Restricted-Cluster werden mit Grund im Aggregate ausgewiesen und separat erneut versucht, wenn die Restriction aufgehoben wird.
- **`escalate(ticket, level)`** (z. B. Logical → Cryptographic), **`open_tickets()`** (nach Absturz des Aufrufers), **`clusters_batch`**.
- **Mehrbetroffene Daten (K-25):** gemeinsamer Fallcluster; jede betroffene Person hat ihren Wurzelcluster; Verknüpfung per Link nur, wo die Löschung der Person den Fallcluster mitziehen **soll**; sonst Referenz. Keine mehrclustrigen Einträge.

### 9.9 Restexposition (Pflichtbestandteil jedes Receipts, vom Orakel geprüft)

| Bereich | Restrisiko | Zuständig |
|---|---|---|
| Prozessspeicher, Swap, Core-Dateien, noch nicht reklamierte Skiplist-Knoten | Klartext-Reste | `Zeroizing`, Cache-Drop beim Gate; `MemLock`/`DONTDUMP` im Adapter; Betreiber: Swap verschlüsseln, Core-Dumps aus. Der optionale Port `HostProbe` (§3.2) liefert den Ist-Zustand; ohne Probe oder bei `Unknown` erscheint die Zeile als `Declared` (§13.2.1) |
| Dateisystem-Journal, Flash-Wear-Leveling, Volume-Snapshots, externe Backups, Checkpoints mit bereits `Retired` Plain-Daten | Alte Blockversionen | nur Ciphertext bei Sealed; Plain: Schutz fehlt, Betreiber begrenzt Aufbewahrung |
| KMS-Karenzzeit | Schlüssel wiederherstellbar bis Fristende | Stage `KeyDestroying`, Status `Blocked`; kein Crypto-Erase-`Done` |
| Schlüsselkopien (Escrow, HSM-Backups) | G1 hinfällig | `KeysInBackups` (`Bounded{until}` ⇒ Statuscap bis Fristende, `Yes` ⇒ kein Cryptographic); Betreiber |
| Schlüsselzusicherung `FilePassphrase` | Vernichtung auf Journal-/CoW-/Flash-Medium nicht beweisbar | Statuscap `WeakKeyAssurance`; Produktion wählt `Hardware`/`Kms` (§9.4) |
| Metadaten | ClusterId (Pseudonym, bis Purge von Redo-Gate-Frames), Anzahl, Größen, Zeitpunkte, Packgruppe | zufällige IDs; Ledger enthält epochengebundene HMAC-`tomb_id` (linkbar bis `retire_tomb_epoch`, §10.4), scope-digests und verschlüsselte Selector-Ciphertexts, nie Klartextidentitäten/-keys; optional Padding auf Bucket-Größen |
| Rollback über den Anker hinaus / im Ankerfenster | Ticket rückgängig | Anker-Stärke im Receipt |
| Ledger auf demselben Gerät wie die Daten | gemeinsame Failure Domain (Verlust/Rollback beider) | `LedgerSameDevice` (§9.7); Produktion: getrenntes Volume |
| Obere Schichten (Prompt-Caches, Embedding-Kopien, Logs, Fine-Tuning-Daten, Modellgewichte, Provider-Logs) | **nicht** im Kern lösbar | nur Daten, die über Links und `SegmentOwner` angebunden sind, werden mitgelöscht |
| Verzeichnis Person → Wurzelcluster (§12.7) | nur physisch löschbar | Muster in §12.7 |

---

## 10. Recovery, Checkpoint, Restore

### 10.1 Öffnen (deterministisch)

```
OP1   LOCK (exklusiv; ReadOnly geteilt)
OP2   FORMAT-Kopf lesen → KEYS entpacken (KEK ⇒ RootKey) → FORMAT-MAC prüfen → LedgerScopeKey für Intent-Reconcile bereitstellen
OP2a  Anker/Ledger: Catalog-Epoche < Strong-Anchor ⇒ Err(Rollback). Ledger muss mindestens E_a Einträge haben und Eintrag E_a den Hash H_a tragen.
      Durables Ledger-Intent hinter Anker ist ein Crash-Präfix: vor Schreiböffnung idempotent fortsetzen; Required-Anchor unerreichbar ⇒ Locked/Err(AnchorUnavailable), nie schwächer öffnen.
      Ledger-Gerät nicht lesbar ⇒ schreibend `Err(LedgerUnavailable)`; `ReadOnly` nur mit Diagnose (E-81).
OP3   CURRENT → Catalog bis zum letzten gültigen Record (Torn Tail verwerfen)
OP4   Verzeichnisse scannen: Dateien ohne Catalog-Eintrag = verwaist (noch nicht löschen)
OP5   KeyProvider prüfen; KEK fehlt ⇒ Modus LockedKeys
OP6   Redo ab replay_floor, zwei Durchläufe; versiegelte Ledger-Intents sind zusätzliche Gate-Wahrheit:
        1: Kette prüfen; full-cluster Retires, Range-/Branch-Masken und Space-Gates getrennt aus Catalog/Redo/Ledger sammeln
        2: Full-retired Cluster überspringen (nie entsiegeln); Selector-Masken bei der Versionssicht anwenden;
           `KeyUnavailable` ⇒ LockedKeys/retry;
           bestätigtes `KeyLost` ohne passendes Erase-Intent ⇒ Quarantäne/Corrupt, nie still überspringen;
           Branch-/Merge-Frames anwenden
OP7   Memtable aufbauen; visible = durable = letzter gültiger Stamp; next_stamp nach §6.2; Branch-Tabellen lazy
OP8   Vor Freigabe von Views/Schreibzugriff: alle versiegelten Ledger-Erase-/SpaceDrop-Intents idempotent in Catalog-Gates überführen;
      danach offene Löschungen = Intents ohne GateApplied ∪ erase_pending ∪ Gate-Frames ohne erase_done ∪ Retired ≠ Purged ∪ Space Dropping
      ⇒ bei fehlendem LedgerScopeKey RestoreQuarantine/LockedKeys; sonst urgent fortsetzen; kein Body wird vor Gate-Auflösung entsiegelt, keine irreversible Stufe ohne Anchor
      (E-75: Intents mit `origin = Poisoned` laufen hier identisch; zuvor lösen OP6/OP7 `Indeterminate`-Changes auf)
OP9   Verwaiste Dateien und .building.* entfernen (nach OP8, nie davor)
OP10  Reserven (Stamp, Notpool, ID, Redo-Segment, RESERVE) herstellen; Admission-Prüfungen (FD-Limit `RLIMIT_NOFILE ≥ max_open_files + fd_reserve`, `max_live_files`, `cluster_meta_budget` gegen die Clusterzahl des Catalogs; Verstoß ⇒ `Err(AdmissionLimit{..})` bzw. `ReadOnly` mit Diagnose); Wartung starten, OpenReport
```

Ehrliche Grenze: Ein beschädigter letzter quittierter Gruppenabschnitt hinter der letzten `redo_mark`, dem nichts Gültiges folgt, ist von einem Torn Write nicht unterscheidbar; die Heilung kürzt dann. Das ist die einzige Stelle, an der Bitfäule `durable = true` verletzen kann; `redo_mark` hält das Fenster klein.

### 10.2 Absturzmatrix (Auszug, vollständig in der Testsuite)

| Unterbrechung | Soll |
|---|---|
| vor Redo-`sync` | Change nicht sichtbar, `apply` kehrte nicht zurück |
| nach Redo-`sync`, vor Rückkehr | Change vorhanden (Token verhindert Doppelung) |
| Run-Bau / vor Edit / vor Unlink | `.building` entfernt bzw. Run verwaist/entfernt, Redo spielt ein |
| mitten in einer Catalog-Gruppe | gültiger Präfix gilt |
| Ledger-Intent torn/Sync unbestimmt | Nur vollständiger valider Intent gilt; Retry derselben `EraseRequestId` findet Ticket oder kann sauber neu anlegen |
| Ledger-Intent durable, Gate-Catalog-Edit fehlt | vor Open Gate aus Intent rekonstruieren, `GateApplied`/Anchor nachziehen |
| Gate-Catalog-Edit durable, `GateApplied`/Anchor fehlt | vor Open idempotent nachziehen; keine irreversible Folge bis Anchor |
| Anchor durable, KMS-Aufruf noch offen | `KeyDestroyIntent` prüfen, idempotent `destroy_many`/`probe` fortsetzen |
| Schlüssel vernichtet, `KeysDestroyed`-Event fehlt | `probe == Absent` bei passendem `KeyDestroyIntent` ⇒ Event/Edit nachholen |
| Plain-Barriere unterbrochen | ER4 neu, idempotent |
| Purge-Edit durabel, Unlink offen | Verwaisten-Suche |
| Edit durabel, Anker alt | Nachzug in OP2a |
| Branch-Fork-Frame durabel, Memtable nicht | Replay stellt Branch-Tabelle wieder her |
| Merge-Frame durabel, `source`-Zustand nicht | `Merged` aus Frame abgeleitet |
| `DiskFull` beim Vorallokieren | sauberer Fehler, keine Teilschreibung |
| Checkpoint abgebrochen | `dst` unvollständig ohne Siegel, Quelle unverändert |

### 10.3 Checkpoint, Restore, Rollback-Schutz

- **Checkpoint:** (1) Flush-Barriere; (2) unter `catalog_edit` Version pinnen, `erase_epoch` und Ankerwert lesen; (3) Checkpoints innerhalb der physischen Purge-/DiskFull-Garantie verwenden unabhängige `Copy`-Dateien; (4) Manifest siegeln (enthält `erase_epoch`, `created_at` und die höchste offene `tomb_epoch`, §10.4). `Reflink`/`Hardlink` sind nur zulässig, wenn ein registrierter Checkpoint-Owner vor Scratch-Recycling Referenzen auf betroffene Extents nachweislich entfernt und attestiert. Andernfalls gelten sie als nicht verwaltete Retention-Copies und erscheinen als Restrisiko im Receipt. `KEYS` ist enthalten, **KEK und Clusterschlüssel nicht** (`INV-KEY-1`). Das Ledger liegt nie im Checkpoint. Das Manifest listet bereits `Retired`, aber noch nicht gepurgte Plain-Cluster.
- **Restore** ersetzt die bisherige Instanz und erhält ihre `InstanceId` (kein zweiter lebender Klon, NZ-19): Der Kern startet in `RestoreQuarantine` (keine Nutzer-Reads), verlangt vor schreibendem Open den externen Fencing-Nachweis für den bisherigen Besitzer, validiert Ledger-Kette und External Anchor und spielt jeden vollständigen Intent scope-genau ein (`INV-ERA-9`). Cluster-Intents matchen HMAC-Tombstones und retire'n Cluster; Range-/Branch-Intents werden mit `LedgerScopeKey` entschlüsselt und als Masken auf alle restaurierten Versionen angewandt; `SpaceDropIntent` matcht die nie wiederverwendete SpaceId und gate't den gesamten Space. Kein fehlender Schlüssel/Anchor darf als Löschung ausgelegt werden: bei `KeyUnavailable`, `KeyLost`, fehlendem Scope-Key, Fencing-Nachweis oder Ledger-Mismatch bleibt Restore quarantined. Nach Gate-Reconcile werden offene KeyDestroy-/Purge-/Extension-Stufen fortgesetzt. Liegt das Manifest hinter einem Strong Anchor, wird es als Rollback verweigert. Erst nach `RestoreReconciled` startet `Running`.
- **Ehrliche Grenze:** Wird zusätzlich der Anker-Dienst zurückgesetzt, ist Schutz nur noch kryptografisch (Schlüsselvernichtung), nicht mehr protokollarisch.

### 10.4 Backup-Horizont und Tomb-Retire (K-31, E-74, E-81)

**Problem:** Restore gleicht Backups über `tomb_id`s ab (§10.3). Bleiben die Pseudonyme ewig auflösbar, ist das Ledger ewig personenbezogen; werden sie zu früh anonymisiert, kann ein altes Backup gelöschte Cluster wieder beleben. Der Horizont verbindet beide Seiten.

- **Begriffe:** `backup_horizon` ist die maximale Aufbewahrungsdauer aller wiederherstellbaren Kopien (Checkpoints und externe Backups). `BackupMark{instance_id, erase_epoch, created_at, tomb_epoch}` liefert `Storage::backup_mark()`; `Storage::erase_epoch() -> u64` ist der monotone Zähler allein. Checkpoint-Manifeste enthalten die Marke automatisch; externe Backups (Volume-Snapshots, Dateikopien) müssen sie beim Erzeugen mitführen.
- **Default:** kein Retire. Dann gilt: Epoche `0`, `tomb_id` linkbar, Restore gleicht gegen alle Intents ab, es gibt keinen `restore_floor`.
- **Retire-Bedingungen** (alle, geprüft im Ledger-Writer): (1) Epoche `Closed` (eine neuere Epoche existiert; dadurch ist `last_erase_epoch(e)` final); (2) `now − closed_at ≥ backup_horizon` bei verifizierter Uhr (`ClockUncertain` ⇒ nicht); (3) kein offenes Ticket, kein `EraseIntent` ohne `Completed`, kein offener Restore mit `tomb_id`s dieser Epoche.
- **Ablauf:** `TombEpochRetireIntent{epoch, restore_floor' = max(restore_floor, last_erase_epoch(e))}` durabel, Anker fortgeschrieben, `KeyProvider::retire_tomb_epoch(e)` liefert `Absent`, dann `TombEpochRetired`. Absturz an jeder Stelle: Roll-forward (`INV-ERA-4`); ein Intent ohne Provider-Bestätigung bleibt `Blocked` und wird wiederholt.
- **Restore-Regel:** Eine Kopie mit `mark.erase_epoch < restore_floor` wird mit `Err(BackupBeyondHorizon{floor, mark})` abgelehnt; es gibt keinen Override im Kern (wie bei `Restriction`). Eine Kopie ohne Marke wird abgelehnt, sobald irgendeine Epoche retired ist. Gegen nicht retired Epochen gleicht Restore weiter ab; die Kosten sind `Cluster der Kopie × aktive Epochen` HMACs (`max_active_tomb_epochs`, Standard 4; H-23).
- **Wirkung:** Nach Retire sind alle `tomb_id`s dieser Epoche anonym; Kettenprüfung und `verify_receipt` bleiben gültig (§9.7).
- **Ehrliche Grenzen:** Kopien von Tomb-Schlüsseln außerhalb des Providers heben die Anonymität auf; für Tomb-Schlüssel gelten `keys_in_backups` und `assurance` des Providers (§9.4), ein `Bounded`-Provider macht Retire erst nach `until` wirksam. Ein Betreiber, der Kopien über `backup_horizon` hinaus aufbewahrt, bricht die Annahme; der Kern verweigert deren Restore, schützt aber keine Daten, die außerhalb des Kerns liegen.

---

## 11. Nebenläufigkeit

### 11.1 Prinzipien

Ein Sequencer (Rang 1, ohne E/A) für stamp-relevante Zustände und ein Syncer für das Redo; unveränderliche Dateien und `Arc<Version>`; Besitz durch Claims; Wahrheit durch durable Edits/Frames; Generationszähler statt Sperren für Schlüsselcache, Blockcache, Clusterzustand; keine Sperre über E/A außer `catalog_edit` (Catalog-Sync); der Redo-Sync läuft im Syncer-Thread ohne Sperre; Ledger/Anker haben je einen Schreiber über Kanäle.

### 11.2 Sperrränge (nur aufsteigend, je Thread höchstens einmal, `debug_assertions` prüft per Rangstack)

| Rang | Sperre | Über E/A | Bemerkung |
|---|---|---|---|
| 1 | `sequencer` | **nein** | Stamp-Vergabe, `Live → Retiring`, Link/Restrict/Lift, Branch-Ops; nur RAM, O(Ops) |
| 2 | `intake` | nein | |
| 3 | `catalog_edit` | ja (Catalog-Sync) | Gruppen-Commit |
| 4 | `state` (`RwLock<Arc<Version>>`) | nein | Veröffentlichung |
| 5 | `registry` (Holds, Consumer, Claims, Restrictions, `ViewInner`) | nein | Blatt |
| 6 | `cluster_table` (sharded `RwLock`, inkl. Branch-Tabellen) | nein | Blatt, Lesen aus Rang 1 zulässig |
| 7 | `maint` (je Lane) | nein | Blatt |
| 8 | `notify` | nein | Blatt |

Rang 1 und Rang 3 werden nie gleichzeitig gehalten (Notpool, §5.8). Rang 2 wird vom Leader nur unter Rang 1 genommen (aufsteigend); wartende Aufrufer halten beim Anhängen ausschließlich Rang 2 (§6.1, Schritt 5). Kein Rückruf (`Observer`, `SegmentOwner`, `KeyProvider`) unter Sperren, **insbesondere kein Provideraufruf unter Rang 1**. Memtable lock-frei. Kein `std::sync::Mutex` außerhalb `sync::Ranked`.

### 11.3 Lock-freie Pfade

`ArcSwap`/`RwLock<Arc<_>>` für `Version`; Reihenfolge Publish: Version, dann `visible_stamp` (Release); Leser laden Stamp (Acquire), dann Version. Cluster-Zustand = `AtomicU64 (state:8 | gen:56)`; nur der Sequencer setzt `Live → Retiring`, die Publikation nach durablem Sync setzt `Retiring → Retired` (`INV-LOCK-2`); spätere Übergänge sind monoton und laufen per `catalog_edit`.

### 11.4 Views und Graveyard

`View` hält `ViewInner { version: RwLock<Option<Arc<Version>>> }`. `ViewPolicy::Revoke` setzt `None`; spätestens nach `revoke_grace` (5 s) hält keine widerrufene View eine Referenz (`INV-GRAVE-2`). Eine Datei wird erst gelöscht, wenn der entfernende Edit durabel **und** die letzte `Arc<Version>` gefallen ist (`INV-GRAVE-1`). `files_remove` hält sie bis zum erfolgreichen `unlink + sync_dir` im persistenten Graveyard; erst `unlink_ack` räumt den Eintrag aus. Unlink führt Wartung aus, nie `Drop`.

**View-Lease (E-76):** Jede `View` und jeder `Cursor` trägt eine Lease (`view_lease`, Standard 60 s, gemessen mit der monotonen `Clock`), verlängerbar mit `renew()` bis höchstens `view_lease_max` (Standard 10 min ab Erzeugung); `renew` ist nur bei lebender, nicht widerrufener View möglich. Nach Ablauf verhält sich die View wie `ViewPolicy::Revoke`: ihre Referenz wird gelöst, und jede weitere Nutzung (`get`, `body`, `page`, `poll`, `filter_visible`) liefert `Err(SnapshotTooOld{stamp, floor_hint})`, nie Daten, nie „leer“. Cursor prüfen die Lease zwischen Blöcken wie Gate und Widerruf. Die Lease begrenzt, wie lange eine vergessene View Dateien (Graveyard) und `history_floor` pinnt. Lange Scans setzen über `ResumeToken` und `view_at(V)` neu auf; ist `V < history_floor`, folgt `BelowFloor`. `SnapshotTooOld` (Lease abgelaufen) und `BelowFloor` (Stamp nicht mehr vorhanden) sind getrennte Fehler. Erase widerruft unabhängig von der Lease; `ViewPolicy::Wait(timeout)` wartet höchstens bis `min(timeout, Lease-Ende)`. Der Hold (§9.6) ist davon unberührt.

### 11.5 Claims

`claim(files, job)` atomar unter `registry`; Priorität `urgent > flush > compact > blobgc > idle`; höhere Priorität setzt `cancel`, der Job prüft zwischen Blöcken; Ausgabe-Edit **abgelehnt**, wenn `files_remove ⊄ Claims` (`INV-CLAIM-1`). Compaction notiert `retire_gen`; Ausgabe wird verworfen, wenn ein Eingabecluster inzwischen `Retired` ist.

### 11.6 Race-Nahtstellen

| ID | Race | Lösung | Nachweis |
|---|---|---|---|
| RACE-1 | Lesen × Gate | Gate-Check am Anfang, `gen`-Check nach Entsiegeln | loom |
| RACE-2 | Flush × Erase | Gate vor Memtable-Einfügung, Flush überspringt Retired, ER4 | M2, Sim |
| RACE-3 | Compaction × Erase | Claims, `retire_gen`-Prüfung im Edit | M3, Sim |
| RACE-4 | Cache × Vernichtung | `gen` vor und nach Entsiegeln | loom |
| RACE-5 | Checkpoint × Erase | `Arc<Version>` + `erase_epoch`, serialisiert gegen ER6 | M4, Sim |
| RACE-6 | Consumer × Erase | `retain_floor`, `Lagged`, `Erased`-Change | M2 |
| RACE-7 | Rebucket × Lesen | Gruppe im selben Edit | M5 |
| RACE-8 | Zwei Erase, überlappende Hülle | Serialisierung nach Ticket-ID in `urgent` | Sim |
| RACE-9 | Restriction × Erase | beides im Sequencer, eine Stamp-Reihenfolge | M2 |
| RACE-10 | Link × Erase | beide als Frames im Sequencer total geordnet; `policy_epoch`-Vergleich, Retry, nach 3 Retries Selbstberechnung | M2 |
| RACE-11 | Guard × Gate | Zustandsprüfung (6a) vor Guard (6b) | M2 |
| RACE-12 | Anker × Edit | Anker nie vor Edit, Nachzug in OP2a | M6 |
| RACE-13 | Staging × Erase | `purge` behandelt Staging wie Packed | M3, Sim |
| RACE-14 | Space-Drop × Flush/Compaction | Claims, `Dropping` bricht Jobs ab | M2, Sim |
| **RACE-15** | **Merge × Gate/Restriction** | Merge ist ein Change; Zustandsprüfung 6a gilt für Quell- und Zielcluster (derselbe Cluster) | M2 |
| **RACE-16** | **Fork × Compaction** | `floor` berücksichtigt lebende `fork_stamp`s | M3 |
| **RACE-17** | **Branch-Drop × Merge** | beides im Sequencer, serialisiert; Merge auf gedroppten Branch ⇒ `BranchClosed` | M2 |
| **RACE-18** | **Sync-Fehler × Publikation** | Publikation nur nach Sync derselben Gruppe, in Stamp-Reihenfolge; Fehler ⇒ spätere Gruppen nie veröffentlicht (`INV-PIPE-2`) | M1, Sim |
| **RACE-19** | **Leader-Wechsel × Intake** | Leader prüft Intake nach Freigabe erneut, kein verlorener Wakeup | loom |
| **RACE-20** | **Guard Stufe A × Pipeline-Delta** | Konfliktfenster enthält sequenzierte, noch nicht durable Stamps; `V1 < Fensterfuß` ⇒ Retry | M1, Sim |
| **RACE-21** | **Space-Drop × neue Cluster-Zuordnung** | Space-Gate schließt Writes im selben Frame; Mitgliedschaft nach dem Gate unveränderlich | M2 |
| **RACE-22** | **Link × Gate (derselbe Quellcluster)** | Link ist Control-Frame; Quelle `Retiring`/`Retired` ⇒ `ClusterRetired`; sonst Hülle über `policy_epoch` | M2 |

---

---

## 12. Erweiterungsverträge, Retrieval-Integration und Muster für die obere Schicht

Dieses Kapitel ist der Vertrag zwischen Speicherkern und allem, was darauf aufsetzt (`vector`, `text`, `graph`, `rank`, `engine`, `kvcache`, Agenten). Es beantwortet vier Fragen: **Wo liegen die Wahrheit und ihre Ableitungen?** (§12.4.0), **wie bleiben abgeleitete Strukturen löschbar?** (§12.4.1–§12.4.6), **wie entsteht ein reproduzierbares Suchergebnis?** (§12.4.4) und **wie trennt das System personenbezogene von sonstigen Daten?** (§12.4.6).

**Schichtenregel (K-48).** Die Retrieval-Crates kennen den Speicherkern nicht. Sie sprechen mit einem eigenen, schmalen Persistenz-Trait (`PartitionSink`/`PartitionSource`: Blöcke schreiben/lesen, Generation sealen/abschließen). `contextra-index-bridge` (Level 2) implementiert diese Traits über `SegmentIo`, registriert die `SegmentOwner`, speist den `ConsumerFeed` und liefert die Attestationen. So gilt gleichzeitig: Klasse-A-Crates bleiben autark und einzeln testbar, und kein Index kann am Löschvertrag vorbei persistieren, weil ihm der Weg dorthin fehlt (`std::fs`, `memmap2` und feste Schlüssel sind in Produktivpfaden verboten, `check_forbidden_apis.sh`).

### 12.1 Segment-Register

```rust
pub trait SegmentOwner: Send + Sync {
    fn kind(&self) -> SegmentKind;
    fn manifest(&self, seg: &SegmentMeta) -> SegmentManifest;       // class, key scope, bounds, RAM/scratch, lineage
    fn on_gate(&self, erased: &[ClusterId], at: Stamp, gen: u64);   // Generation invalidieren/Worker wecken; nie Correctness-Quelle
    fn rewrite_without(&self, seg: &SegmentMeta, drop: &ClusterSet, io: &mut SegmentIo) -> Result<RewriteOutcome, SegErr>; // Pflicht
    fn verify_absent(&self, scope: &EraseScope, gen: u64) -> Result<EraseAttestation, SegErr>;
    fn watermark(&self) -> DurablePosition;                           // lückenlos verarbeiteter durable Feed
}
```

`SegmentClass` ist `ClusterSections | MixedPerClusterKey | MixedPerSubjectKey | Rebuildable{max_age,max_rewrite_bytes} | NoPersonalData`. `ClusterSections` liegen als cluster-sealed Blocks im Kern. `MixedPerClusterKey` nutzt je Cluster-Key-Epoche. `MixedPerSubjectKey` ist für IdentityDirectory-Einträge mit separatem EntryDEK. `Rebuildable` hält personenbezogene Nutzdaten nur verschlüsselt gemäß deklarierter Scope und bleibt bis Rebuild+Verify `Purging`. `NoPersonalData` benötigt versionierte Provenienz-/Schema-Evidence und keine bloße Selbstdeklaration.

`manifest` wird vor erster Nutzung registriert und versioniert. Jeder Owner-Aufruf läuft mit Deadline und Fencing-Generation außerhalb Kernlocks. `on_gate` ist eine beschleunigende Invalidierungs-/Wakeup-Notification nach durablem Gate, niemals die Korrektheitsbarriere: jede Query prüft unabhängig davon den Kern-Gate-Zustand, bevor Index-Payload ausgewertet und bevor ein Treffer zurückgegeben wird. Timeout/Owner-Ausfall ⇒ Ticketstatus `Blocked{owner, reason, since, retry_at}`, Ledger-Ereignis `ExtBlocked`, Alarm und Eskalation; verspätete Antworten alter Generation werden verworfen. `Blocked` zählt nie als `Done`; Replacement/Unregister ist ledger-protokolliert und muss mit neuer Generation `verify_absent` erfüllen. `RewriteOutcome` allein ist kein Erase-Beweis: nur `EraseAttestation{owner, generation, scope_digest, applied_position=(instance_id,stamp), keys_destroyed, segments_rewritten, evidence_digest}` mit erfolgreichem `verify_absent` kann ein Ack liefern. Zugriff nur über das versiegelte `SegmentIo`; Owner darf keine direkten Kernpfade öffnen. Jedes Extension-Job-RAM/Scratch-Byte muss vor Allokation per `MemoryBudget`/`SegmentIo` reserviert sein. Segmente speichern Daten bevorzugt **als Cluster-Sektionen im Kern** (eigener Space): dann ist `rewrite_without` trivial.

### 12.2 ExtensionBlock

Opake Blöcke am Run (Tag `u16`, Version `u16`) je Cluster-Abschnitt; `ExtensionBuilder` erhält den Eintragsstrom bei Flush/Compaction; `merge` und `rewrite_without` sind Pflicht; unbekannter Tag beim Rewrite ⇒ Fehler. Ein Besitzer ohne `rewrite_without` darf in `clustered`-Spaces keinen Block anlegen.

**Block-Aggregate und `BlockPruner` (K-29, E-79):** `ExtensionBuilder::block_boundary(block_uid, entries) -> Option<AggBytes>` wird an jeder Datenblockgrenze (Flush/Compaction) mit den Einträgen des Blocks aufgerufen und darf ein Aggregat (≤ 64 Byte, z. B. Min/Max eines Zeitfeldes im Kopf) in den ExtensionBlock des Abschnitts schreiben. `block_uid` ist der abschnittsrelative Blockindex (positionsunabhängig, `INV-PL-2`); bei Sealed-Clustern ist das Aggregat cluster-key-versiegelt. `BlockPruner::may_skip(block_uid, agg) -> bool` gibt der Besitzer einer Anfrage an `page`/`Cursor`. Regeln:
1. Der Kern persistiert **keine** Zeitlanes oder Aggregate im SubIndex (K-29) und ruft `may_skip` nur auf, wenn das Manifest `PruneContract::ShadowSafe{reason}` deklariert. Das heißt: für jeden Schlüssel eines übersprungenen Blocks gibt es keine neuere Version und keinen Tombstone in einem nicht übersprungenen Block (z. B. append-only mit eindeutigen Schlüsseln). Der Kern kann das nicht prüfen; die Zusicherung trägt der Besitzer.
2. Der Kern ruft `may_skip` nur für Lesevorgänge auf dem Branch `main` (`branch == 0`) auf; Elternketten würden die Schattenfreiheit je Branch verlangen.
3. Gate, Restriction und `filter_visible` laufen unabhängig vom Pruner; ein Pruner kann nur Arbeit sparen, nie Sichtbarkeit herstellen. `rewrite_without`/`merge` des Besitzers halten die Aggregate konsistent; ein unbekannter Aggregat-Tag ist ein Fehler.
4. Konformanz: Property-Test „Ergebnis mit Pruner ≡ Ergebnis ohne Pruner“ gegen das Referenzmodell, einschließlich Overwrites und Deletes; ein Besitzer, der ihn nicht besteht, darf `ShadowSafe` nicht deklarieren (`INV-PRUNE-1`).

### 12.3 Change-Feed (Consumer)

Gespeist aus dem **Redo-Log** (nicht aus Runs: diese sind nicht stamp-geordnet). Zwei Böden: `replay_floor` (Recovery-Start) und `retain_floor ≤ replay_floor` (älteste für Consumer behaltene Segmente), `retain_floor = min(replay_floor, kleinste Consumer-Position, Grenze aus max_lag_bytes)`. Consumer persistiert Fortschritt als `(instance_id, stamp)` und pinnt nur Redo-Segmente derselben Instanz; `max_lag_bytes` 1 GiB. Überschreitung oder Plain-Barriere ⇒ `Err(Lagged)` (Aufholen per Vollscan oben). Reihenfolge nach Stamp, mindestens einmal, nur Stamps ≤ `durable_stamp` derselben Instanz. Nutzlasten von Sealed-Clustern entsiegelt der Kern; **nach dem Gate erscheint stattdessen `Change::Redacted{cluster}` bzw. nach ER2/Purge `Change::Erased{cluster|range, at}`**; BLOCK_READ-Cluster erscheinen als `Restricted`-Platzhalter. Branch-Frames erscheinen mit `branch`-Feld. Volatile-Spaces erscheinen nicht. Ein persistierender Consumer, der RAG-/Graph-/Cache-Derivate erzeugt, muss als `SegmentOwner`/Extension registriert sein und Quell-Cluster-Lineage für Erase attestieren; nicht registrierte externe Persistenz wird als Receipt-Restrisiko ausgewiesen und verhindert systemweites `Complete`.

### 12.4 Retrieval-Integration

Index-Watermarks (Muster aus CairnDB) sind Pflicht für jeden abgeleiteten Index: `applied_position = (instance_id, applied_stamp) = watermark()`. Vergleiche gelten nur innerhalb derselben `instance_id`; ein Instanzwechsel erfordert Vollrebuild oder einen verifizierten Import-Cursor. Eine Abfrage unter View-Stamp `V` durchsucht **Basisindex ∪ Delta `(applied_stamp, V]`**, kennt Mandanten-/Rechtefilter **vor** der Indexexploration (keine Existenz-Seitenkanäle) und prüft jeden Kandidaten über Gate und `View::filter_visible` vor Scoring und vor Rückgabe (`INV-VIS-1`). Gate-Zustand und Generation werden zusätzlich vor jeder Payload-Auswertung geprüft; unbekannte oder veraltete Generationen ergeben `IndexLag`/Retry, nie einen Treffer.

#### 12.4.0 Ein-Wahrheits-Commit, Publish-Record und Projektion (K-39, K-51)

**Wahrheit und Ableitung.** Kanonisch (im Kern, atomar, durabel, cluster-sealed) sind je Chunk: Dokument-/Chunk-Metadaten im Kopf, Text im Körper, quantisierter Vektorcode (Space `emb/<model_id>`), Vorwärts-Termliste (`fw`) und Graph-Fakten mit Provenienz (`edge`). Abgeleitet (neu aufbaubar, mit Watermark) sind HNSW/DiskANN-Graphen, invertierte Listen, CSR-Segmente, Präfixsummen und Caches. Es gibt **keinen** Schreibpfad, der zuerst einen Index und dann den Kern ändert, und kein `CommitLedger`/keine Kompensation: Ein Index ist nie Teil der Commit-Entscheidung.

**Publish-Record (große Dokumente).** Ein Change ist durch `max_change_ops` (4096) und `max_change_bytes` (8 MiB) begrenzt; ein Dokument kann größer sein. Deshalb gilt für jeden Ingest das Protokoll **IG**:

| Schritt | Vorgang |
|---|---|
| IG-1 | Klassifikation und Routing (§12.4.6), Pseudonymisierung, Chunking, Embedding **außerhalb** jeder Storage-Sperre |
| IG-2 | Chunk-Records der neuen Version unter `doc/<id>/v<n>/…` in 1…m Changes (je ≤ Limit, jeder mit eigenem `IdemToken` abgeleitet aus `(ingest_id, j)`); sie sind für Leser und Indexprojektion **unsichtbar**, weil sie nur über `head` erreichbar sind |
| IG-3 | **Abschluss-Change:** `doc/<id>/head = {version n, chunk_range, chunk_count}` mit `Guard::Version(head_alt)`; derselbe Change tombstoned die Vorversion, soweit er in das Limit passt, sonst folgt ein Aufräum-Change (die Vorversion ist ab IG-3 unerreichbar) |
| IG-4 | Der `ConsumerFeed` löst `head` aus; die Indexprojektion liest die Chunks der referenzierten Version über eine View bei Stamp ≤ Head-Stamp und wendet sie als **eine** Einheit an (nie halb indexiert) |
| IG-5 | Crash vor IG-3: Staging bleibt unerreichbar, `staged_orphan_ttl` (24 h) und ein Worker räumen es ab; Crash nach IG-3: Recovery stellt `head` her, die Projektion setzt am Watermark fort |
| IG-6 | Passt das gesamte Dokument in ein Limit, entfällt Staging: Chunks und `head` sind **ein** Change (Normalfall) |

`head` enthält Zähler und Schlüsselbereiche, **keine** Inhalts-Hashes (`INV-PRIV-1`); die Integrität sichern AEAD und Ketten-MAC. `filter_visible` erhält für Treffer die Prüfart `DocHead`: ein Chunk ist sichtbar, wenn seine Version dem `head` der Sicht entspricht (ein RAM-Vergleich über die gecachte Head-Tabelle, ein Punktlesezugriff nur bei Cache-Miss).

**Atomarität in Zahlen.** 1000 Chunks × (Chunk, Vektor, `fw`, ≈ 2 Graphfakten) ≈ 5000 Ops: zwei bis drei Staging-Changes plus `head`. Ein 4096-Op-Change schreibt ≤ 8 MiB; der Syncer bündelt sie in Gruppen (§6.1), daher ist die Fsync-Zahl pro Dokument gering und unabhängig von der Indexlast.

#### 12.4.1 Vektorindex (`contextra-vector`, `contextra-simd`)

| Aspekt | Vertrag |
|---|---|
| Quantisierung | SQ8 (`Sq8Bias`) und RaBitQ als Kandidatengenerierung; Rerank mit dem kanonischen Vektor aus dem Cluster nur für die `R` besten Kandidaten (`rerank_depth`, Standard 4 × k), nie für alle |
| Modus A (Mikroindex) | Cluster ≤ `micro_index_max_vectors` (Standard 50 000): kein persistenter Index; flacher Scan oder RAM-Mini-HNSW, aufgebaut aus den `emb`-Records bei Bedarf und im Block-Cache gehalten. Löschung = Gate (nichts weiter nötig) |
| Modus B (Partition) | Größere Mengen: HNSW (`M`, `ef`) oder DiskANN je **Index-Partition** (§12.4.5), als `Rebuildable`-`SegmentOwner`; Blöcke über `PartitionSink` → `SegmentIo`, versiegelt unter dem **Generationsschlüssel** der Partition; kein `mmap`, kein `std::fs`, kein fester Integritätsschlüssel (`DISKANN_INTEGRITY_KEY` entfällt, Integrität = AEAD/MAC unter dem Generationsschlüssel) |
| Löschen | `on_gate` setzt synchron das generationsgefencete Tombstone-Bit (RAM) und blendet den Knoten vor jeder Distanzberechnung aus. Physisch gilt `INV-HNSW-SLOT-1`: Tombstone + Slot-Nullung + **Partitions-Neuaufbau** (kein Re-Wiring ohne Rebuild). Der Neuaufbau erzeugt Generation `g+1` nur aus lebenden Vektoren; nach Attest wird der Schlüssel von `g` vernichtet |
| Filter | Mandanten-/Gate-Maske **vor** Traversierung; ACORN-Augmentierung für selektive Prädikate (`filter_selectivity` < 5 %); NaN/Inf werden abgewiesen (`INV-SIMD-FINITE-1`) |
| Determinismus | Ebenen aus `hash(doc_id, seed)` bzw. injiziertem `Rng`; SIMD-Reduktionsordnung fest (`INV-SIMD-DETERMINISM-1`); gleicher `SnapshotSet` ⇒ gleiche Treffer |
| Isolation | keine Kanten und Caches über Mandanten (`INV-TENANT-IDX-1`); Partitionen sind mandantenrein |

#### 12.4.2 Volltextindex (`contextra-text`)

| Aspekt | Vertrag |
|---|---|
| Forward-Daten | `fw`/`dl` (Termliste, Dokumentlänge) clusterlokal als Records; sie sind die Quelle jedes Neuaufbaus |
| Invertierter Index | Posting-Listen (Delta-VByte) mit Block-Max-WAND je **Index-Partition**; **kein persistiertes globales Wörterbuch**: Term→ID-Zuordnung liegt je Partition und Generation, versiegelt wie die Vektoren |
| Statistik | `N`, `avgdl` je Partition und Generation; `df` wird zur **Abfragezeit** über die abgefragten Partitionen summiert (nur RAM). Es gibt keinen `commit_lock` über globale Statistik; Schreiben in verschiedene Partitionen ist unabhängig |
| Seltene Komposita | Ein Neuaufbau erzeugt das Wörterbuch ausschließlich aus lebenden Dokumenten; damit verschwindet ein seltener Term (und seine Zerlegung durch den `GermanCompoundSplitter`) mit dem letzten Dokument, das ihn trug. Zwischen Gate und Neuaufbau ist er nur über die Maske unerreichbar (`on_gate`); Score-Drift durch noch mitgezählte `df` ist beschränkt und wird als `stats_drift` gemessen |
| Löschen | Gate ⇒ Dokument-Bitmap (RAM) ⇒ Tombstone-Filter in WAND vor dem Scoring; physisch Partitions-Neuaufbau mit Generationswechsel |
| Persistenz | kanonischer Codec statt `bincode`; kein unkeyed `blake3`; kein `block_on` im Löschpfad (`TextPurger` wird synchroner Wartungsjob) |
| Determinismus | Score absteigend, `DocId` aufsteigend (`INV-DETERMINISM`); `INV-TEXT-WAND-1` (WAND ≡ Brute-Force) |

#### 12.4.3 Wissensgraph (`contextra-graph`)

**Zwei Graphen, strikt getrennt (K-44).** (1) Der **Lösch-DAG** (`DerivedFrom`, §9.6) gehört dem Kern: höchstens 8 Quellen je Cluster, Hülle ≤ 4096, Zweck ausschließlich Löschkaskade. (2) Der **Wissensgraph** (Entitäten, Relationen, n-äre Hyperkanten) gehört `contextra-graph`; seine Kanten sind Fakten, keine Löschabhängigkeiten.

| Aspekt | Vertrag |
|---|---|
| Provenienz | Jede Kante und Hyperkante trägt `(source_cluster, source_doc, version)`; ohne Provenienz ist sie unzulässig (`INV-GRAPH-PROV-1`) |
| Besitz | Eine Kante gehört dem Segment der **Packgruppe ihres Provenienz-Clusters**. Hat eine Hyperkante Belege aus mehreren Löschdomänen, existiert **je Domäne eine eigene Kopie** (kein geteilter Besitz, analog NZ-11); die Abfrage dedupliziert zur Laufzeit im RAM über den kanonischen Schlüssel |
| Personen-Entitäten | Knoten vom Typ `Person` sind Eigentum des Personen-Clusters (Pseudonym-Tresor, §12.4.6); das gemeinsame Graphsegment enthält nur `EntityRef(pseudonym)`. Kanten zwischen gemeinsamen Entitäten und Personen-Entitäten sind **nicht kaskadierende Referenzen** |
| Struktur | CSR-Basis (unveränderlich) + Delta-Puffer; Traversierung = Vereinigung der Adjazenzen über die Segmente des Mandanten (`max_graph_segments_per_tenant` 64, Knoten→Segment-Bloom in RAM); `MAX_TRAVERSAL_HOPS` 16 und Fan-out-Limit 10 000 bleiben |
| Löschen | Gate ⇒ Maske über `source_cluster` (RAM) vor PPR/Traversierung; physisch Segment-Neuaufbau ohne Kanten der gelöschten Provenienz, anschließend Entitäts-Sweep (Knoten ohne Kanten werden entfernt, Puffer `Zeroize`) |
| Inhaltliches Ersetzen | `Supersedes`/Decay (kognitiv) ist **kein** Löschen: es setzt Relevanz oder Gültigkeitszeit im Kopf, nie ein Gate (`INV-DECAY-1`, §12.9) |
| Algorithmen | PPR (Forward-Push), APPRH, TL-HFD, PathRAG, Community/Percolation laufen auf einem gepinnten Segment-Set; Ergebnisse sind deterministisch (`INV-GRAPH-004`) |

#### 12.4.4 Query-Plan-Protokoll, Fusion und Tokenbudget (`contextra-engine`, `contextra-rank`)

| Schritt | Vorgang |
|---|---|
| QP-1 | `TenantPolicy` und Zugriffskontext prüfen (NZ-16); Anfrage normalisieren (NaN/Inf, Dimensionen, `k` ≤ Limit) |
| QP-2 | `view_durable()` ⇒ `V`; **`SnapshotSet`** bilden: `(instance_id, V)` plus je Kanal und Partition `(partition, generation, applied_stamp)` (K-53) |
| QP-3 | Read-your-writes: bei `min_stamp` bis `ryw_wait` (Standard 50 ms) auf `watermark ≥ min_stamp` warten, sonst Delta-Pfad (K-52); unbedeckte Lücke ohne Delta ⇒ `IndexLag`, nie stilles Auslassen |
| QP-4 | Kanäle **parallel**, jeweils mit Gate-/Mandantenmaske vor der Exploration: Vektor (Basis ∪ Delta), BM25/WAND, Graph (PPR, Hyperkanten) |
| QP-5 | Kandidaten sammeln, `View::filter_visible` (Gate, Restriction, `DocHead`) |
| QP-6 | Fusion in `contextra-rank`: RRF/mRRF (`k = 60`), 4-/5-Signal-Fusion, Kalibrierung (Isotonic/Platt/Conformal); Gleichstände nach Score, dann stabiler ID-Ordnung (`INV-RRF-1`) |
| QP-7 | Budget: `page()`/`DiBud` wählen anhand `weights[W]`, ohne Körper-I/O für Auslassungen; Körper werden erst für die gewählten Treffer entsiegelt, mit Gate-Prüfung vor **und** nach dem Entsiegeln |
| QP-8 | **Letzte Prüfung vor Ausgabe:** Gate-Generation jedes Treffers erneut (`INV-VIS-1`); Trust-Label und Lineage in die Antwort |
| QP-9 | Antwort trägt `SnapshotSet`, Kanalbeiträge und `explain`-Daten; diese enthalten nur IDs/Scores, keine Inhalte gelöschter Records, und werden nicht länger gecacht als die Sicht lebt |

**Determinismus (`INV-RAG-4`).** Gleiche Anfrage und gleiches `SnapshotSet` liefern bitgleiche Ergebnisse, unabhängig von Threadzahl und Hash-Reihenfolge. Das macht Differentialtests (`Index ∪ Delta ≡ Vollscan`) und Audits („was hätte das System bei Stamp S geliefert?“, `view_at`) möglich. **Lernsignale (K-54):** Nutzerfeedback und Kantenverstärkung liegen im Cluster des Nutzers; globale Kalibratoren trainieren nur auf aggregierten Scores ohne Nutzer-/Query-Bezug und erst ab `k_min` Beobachtungen je Gruppe.

**Kosten pro Anfrage (Entwurfsziel, H-30/H-31).** Kanäle laufen parallel; die Latenz ist das Maximum der Kanäle plus Fusion. Die Zahl der durchsuchten Partitionen ist durch `tenant × aktive due_buckets × shards` begrenzt und wird über den Mandantenfilter und, wo vorhanden, über Zeit-/Quellenprädikate (`BlockPruner` nur für schattensichere Besitzer) reduziert.

#### 12.4.5 Index-Partitionen, Generationsschlüssel und Rebuild-Kosten (K-49)

**Regel.** Eine Index-Partition gehört genau einer Packgruppe `(tenant_tag, due_bucket, shard)` und hat ein hartes Größenlimit (`max_partition_vectors` 500 000, `max_partition_postings_bytes` 256 MiB, `max_partition_edges` 5 Mio.). Überschreitet sie das Limit, spaltet die Wartung nach `shard`. Der Index folgt damit derselben Platzierungsregel wie die Daten; Ablauf einer Packgruppe (`pack_expired`) verwirft ihre Partitionen in O(Dateien).

**Generationsschlüssel.** Jede Partitionsgeneration hat einen eigenen Schlüssel aus dem `KeyProvider` (Scope `PartitionGeneration`, Format wie Cluster-Schlüssel, nie in `KEYS`/Checkpoints/Backups). Ein Neuaufbau erzeugt Generation `g+1` mit neuem Schlüssel; nach `verify_absent` für die gelöschten Quellen und Schalten der Version wird der Schlüssel von `g` vernichtet (`probe == Absent`). Alte Kopien von `g` in Backups sind danach unlesbar. Das erfüllt die Forderung aus §12.4.7 („geteilter Segmentschlüssel reicht nicht“) ohne Pro-Cluster-Schlüssel im geteilten Index.

**Kostenmodell.** Sei `P` die Partitionsgröße in Bytes und `D(w)` die Menge der Partitionen mit Löschungen im Fenster `w = purge_window_fraction × purge_deadline`. Je Fenster wird jede Partition **höchstens einmal** neu aufgebaut; die Rebuild-Last ist `Σ_{p ∈ D(w)} P_p ≤ Σ aller Partitionen`, nie proportional zur Zahl der Löschaufträge. Der Neuaufbau läuft in der Lane `compact`, mit eigenem `MemoryBudget`-Anteil (`index_rebuild_bytes`) und `maint_io_fraction`; er verdrängt weder Gate/KeyDestroy noch Commits. Reicht die Kapazität nicht für die Frist, wird das Ticket `Blocked{owner=index, reason=RebuildCapacity}` und alarmiert; es gibt kein stilles `Done` (`INV-ERA-8`).

**Abwägung (Entscheidung K-49).**

| Variante | Löschkosten | Suchlatenz | Betrieb | Urteil |
|---|---|---|---|---|
| Ein globaler Index je Mandant | Vollrebuild je Löschfenster, wächst mit dem Korpus | beste | einfach | verworfen: verletzt Z-4 |
| Ein Index je Cluster | minimal | schlecht bei clusterübergreifender Suche, 10⁷ Segmente | Dateiflut | nur als Mikroindex (Modus A) |
| **Partition = Packgruppe** | beschränkt durch `P`, amortisiert | Fan-out ≤ Partitionszahl je Mandant | beherrschbar, folgt dem Datenlayout | **gewählt** |
| Gerichtete Löschung ohne Rebuild (Re-Wiring) | scheinbar gering | Recall-Verlust | verletzt `INV-HNSW-SLOT-1` | verworfen |

**Grundregeln (aus dem Löschvertrag).** Jedes Indexelement, jede Kante und jeder Posting-Eintrag ist einer Quell-`ClusterId`/Generation zuordenbar; nicht zuordenbare Strukturen sind für personenbezogene Daten unzulässig. `MixedPerClusterKey` ist nur zulässig, wenn Payload und alle Metadaten genau einem Cluster gehören. Clusterübergreifende HNSW-Kanten, Posting-Listen, Centroids und Routingdaten sind `Rebuildable{max_age, max_rewrite_bytes}`. `on_gate` aktiviert synchron einen generationsgefencten Tombstone-Zustand vor der Rückkehr der Query; das garantiert sofortige API-Unsichtbarkeit, aber keine physische Entfernung. Ein `Rebuildable`-Owner bleibt `Purging`, bis Payload, Kanten, Postings, Routing und gespiegelte Kopien entfernt und attestiert sind; über Obergrenze oder Deadline ⇒ `Blocked/Incomplete`.

**Indexklassen und Personenbezug (K-60).** Jede Partition trägt eine `IndexClass`:

| Klasse | Inhalt | Limits | Lebensdauer |
|---|---|---|---|
| `Corpus` | aus `NonPersonal`-Chunks und aus pseudonymisiertem Text abgeleitet | Standardlimits oben | bis Löschung/Neuaufbau |
| `Personal` | ausschließlich aus **pseudonymisiertem** Text von `Personal`-Chunks abgeleitet (Platzhalter, §12.4.6) | `max_partition_vectors_personal` 100 000, `max_partition_postings_bytes_personal` 64 MiB, `max_partition_edges_personal` 1 Mio. | `Rebuildable{max_age = purge_window_fraction × purge_deadline}`; Packgruppe nach Aufbewahrungsende der Person (`due_bucket`), Ablauf wirft die Partition ab |

`INV-SEP-2`: Eine geteilte Partition enthält keinen aus Personen-Klartext abgeleiteten Vektor, Term oder Graphknoten. Klartext-Embeddings existieren nur im Cluster-Mikroindex (Modus A) des Personenclusters, unter dessen Clusterschlüssel; ihre Löschung ist mit dem Gate und `KeysDestroyed` erledigt. Damit bleibt Suche **über** viele Personen (pseudonymisiert) möglich, und die Löschung einer Person kostet höchstens den Neuaufbau der kleinen `Personal`-Partitionen, in denen ihre Platzhalter vorkamen. Kleinere Limits halten die Rebuild-Kosten je Löschfenster klein (H-36); die Zuordnung Platzhalter → Partition führt der `index-bridge`-Owner als Refcount je Person (nur RAM-Cache, aus `fw`-Records ableitbar).

#### 12.4.6 Trennungsprinzip und Pseudonym-Tresor (K-50)

**Ziel.** Personenbezogene Daten sind von sonstigen Daten so getrennt, dass **ein** Cluster-Löschauftrag alle Namensbezüge entfernt, ohne den Wissenskorpus zu zerstören.

| Klasse | Ablage | Bemerkung |
|---|---|---|
| `NonPersonal` (Produktdokumentation, Richtlinien, Fachwissen) | Korpus-Cluster des Mandanten (`Plain` oder `Sealed`), Packgruppe `corpus` | normale Indizes |
| `Personal` (Akten, Mails, HR, Kundendaten mit Klarnamen) | Personen-/Fallcluster, `Sealed`, Space mit `DataClass::Personal` | Wurzelcluster + Links (§12.7) |
| `Mixed` (Dokument mit beidem) | **Routing durch die Ingest-Pipeline** (`contextra-privacy`, Richtlinie aus `TenantPolicy`): (a) Zerlegung in personenbezogene und sonstige Chunks, (b) Pseudonymisierung der Namensbezüge, (c) ganzer Chunk in den Personencluster, (d) Quarantäne bei Unsicherheit | Fehlklassifikation ist das Kernrisiko, siehe unten |

**Pseudonym-Tresor.** Bei (b) steht im Korpus-Chunk ein Platzhalter (`[PERSON:7f3a…]`), die Zuordnung Platzhalter → Klartext liegt als Records im `Sealed`-Cluster der Person. **Embeddings, BM25-Terme und Graph-Entitäten des Korpus entstehen aus dem pseudonymisierten Text**; sie enthalten keine Klarnamen. Wiederherstellung der Namen (z. B. für die Antwort an berechtigte Nutzer) passiert erst nach dem Retrieval beim Zusammenbau der Antwort, nur über den Tresor, nur unter Policy, und nur im Egress-Pfad (`contextra-privacy`). **Löschen der Person** gated und vernichtet ihren Cluster und damit den Tresor; der Korpus bleibt mit nicht mehr auflösbaren Platzhaltern bestehen.

**Ehrliche Grenzen.** (1) Pseudonymisierte Daten sind solange personenbezogen, wie die Zuordnung existiert; nach Vernichtung des Tresors hängt die Einstufung als anonym von der Re-Identifizierbarkeit über Kontext (Funktion, Datum, Ort, seltene Details) ab; das ist eine Einzelfallbewertung und kein Kernversprechen. (2) Die Qualität hängt an der PII-Erkennung (`contextra-privacy`); der Kern kann nur erzwingen, dass `Personal`-Spaces ausschließlich `Sealed`-Cluster aufnehmen (`INV-SEAL-1`). (3) Die Pipeline führt eine Messgröße `pii_residual_rate` (Stichprobe mit Prüf-Orakel, H-34); sie ist Teil der Berichterstattung, nicht Beleg für Fehlerfreiheit. (4) Der Pseudonym-Platzhalter ist je Person zufällig und nie aus dem Klartext abgeleitet (`INV-PRIV-1`).

**Sicht auf Backup und Restore.** Der Tresor liegt im Cluster; Restore wendet die Ledger-Intents zuerst an (§10.3), der Tresor wird dadurch nicht wiederbelebt. Index-Partitionen alter Generationen sind nach Vernichtung ihrer Generationsschlüssel unlesbar.

#### 12.4.7 Ende-zu-Ende-Vertrag für Contextra-RAG

(1) Validiere `DataClass`, Mandanten-/Zugriffskontext, Budget und Lineage in der oberen Schicht; (2) committe kanonische Quelldaten durabel (Publish-Record, §12.4.0); (3) Extensions konsumieren mindestens einmal und idempotent nur bis `durable_stamp`; (4) eine Indexgeneration wird erst nach vollständig angewandten Deltas veröffentlicht, samt `(instance_id, applied_stamp)`; (5) die Query bindet `view_durable()` und das `SnapshotSet`, wendet Mandanten-/Rechtefilter vor der Exploration an und prüft jeden Kandidaten über Gate und `filter_visible` vor Scoring und vor Ausgabe; (6) Lücken oder zu alte Generationen liefern `IndexLag`/Retry oder den Delta-Pfad, nie stillschweigend unvollständige Treffer.

Quelldaten und Indexgeneration sind nicht gemeinsam atomar; der Vertrag stellt Durability, Wiederaufbau, Idempotenz, Sichtbarkeit und Löschbeleg her, nicht Null-Lag. Ein Index darf erst als aktuell gelten, wenn sein Watermark die angefragte Position abdeckt oder der Delta-Pfad die Lücke schließt. Ein Owner darf nicht `NoPersonalData` deklarieren und gleichzeitig personenbezogene Lineage verarbeiten. Alle Komponenten reservieren RAM, Scratch, I/O und Lag aus dem gemeinsamen `MemoryBudget`/Governor; unter Druck wird der Indexaufbau gedrosselt oder abgewiesen, bevor unbeschränkt allokiert wird, Schreibbarkeit und Gate-Priorität bleiben erhalten. Tenant-/Cluster-IDs, Querytexte, Schlüssel, Inhalte und Embeddings sind nie Metrik-Label oder Standard-Observer-Payload.

### 12.5 Kopf, Gewichte, Budget

`header`, `weights`, `page`, `weight_sum` sind die Schnittstelle für Rang- und Budgetlogik. Die obere Schicht legt fest, was ein Gewicht bedeutet (Token je Tokenizer-Familie: je ein Slot, bis 4). Token-Budget-Packing (Knapsack, Redundanzabzug, präfix-stabile Ordnung) liegt oben und nutzt `page` für die Kandidaten.

### 12.6 Konventionen (keine Kernfunktionen)

| Thema | Konvention |
|---|---|
| **Zoom-Stufen** | Schlüsselsuffix je Stufe im selben Cluster (`doc/<id>/L0`, `/L1`, `/L2`); L1/L2 im Kopf, L0 im Körper |
| **Trust-Label** | erstes Kopfbyte `trust ∈ {system,user,tool_verified,tool_unverified,untrusted}`; Abgeleitetes erbt das Minimum seiner `DerivedFrom`-Quellen; Hochstufen nur privilegiert mit Audit |
| **Lineage/Derivation** | abgeleitete Cluster per `link(derived, sources)`; Kopf trägt `(producer_id, source_stamp)`; veraltet = `source_stamp` ≠ aktueller Quellstamp; Cache nur **je Tenant, nie über Löschdomänen** |
| **Derivation Cache** | Memoisierung von Embeddings/Token-IDs (K-32, E-82): Schlüssel `MAC(cache_secret, model_id || canonical_input)`; `cache_secret` ist ein zufälliges 32-Byte-Secret **je Cluster** aus `Entropy`, als Record im selben `Sealed`-Cluster abgelegt, nie aus Inhalt, Mandantenschlüssel oder Kern-Schlüsseln abgeleitet, nie exportiert. Einträge sind gewöhnliche Records desselben Clusters (löschlokal), nie in globalen Segmenten. Kein Treffer über Cluster- oder Mandantengrenzen, keine Dedup. `Plain`-Cluster erhalten keinen Cache. Die Gleichheit identischer Eingaben ist nur innerhalb des Clusters erkennbar, der Cache stirbt mit ihm; `INV-PRIV-1` bleibt gewahrt |
| **Handles** | kurze IDs (4–6 Zeichen) sind Schlüsselaliase innerhalb des Clusters (Alias-Space); „Meintest du …?“ per Präfixscan im Cluster |
| **Fehlertext für Agenten** | `Error::hint()` plus `suggested_call` in der Fassade oben |
| **Bitemporalität** | `valid_from/valid_to/observed_at/source_time` im Kopf; `supersedes`-Kante als Schlüssel `edge/supersedes/<id>` |
| **Embeddings** | Segmente (Index) oder Cluster-Space `emb/<model_id>`; Modellwechsel = neuer Space/Segment, alte Version bleibt lesbar bis Migration |
| **Work Queue / Sleep Cycles** | Consumer + Tasks als Daten im Cluster; Ergebnis = normaler `Change` mit Producer im Kopf |
| **Vergessen als Zoom-Down** | Löschauftrag der oberen Schicht; keine stille Entfernung ohne `Eraser`-Aufruf |
| **KV-/Prompt-Cache** | außerhalb; wer Prefix-Daten persistiert, legt sie als Segment mit `rewrite_without` ab |
| **Session ≠ Cluster (K-35)** | Der Cluster folgt der **Löschdomäne** (Person, Fall, Arbeitskontext), nicht der Sitzung. Agenten-Sessions sind Schlüsselpräfix (`sess/<id>/…`) oder ein TTL-Branch im Cluster der Person; rein flüchtiger Scratch liegt in `Volatile`-Spaces. Ein Cluster je Sitzung ist nur zulässig, wenn die Sitzung eine eigene Löschdomäne ist (z. B. anonyme Gastsitzung mit eigenem Aufbewahrungsende). Wer 10⁷ Mikro-Cluster anlegt, zahlt 40 B RAM je Cluster, ein Siegel je Abschnitt und eine Directory-Zeile; die Fassade meldet `ClusterMetaBudget`-Druck früh mit `hint()` |

### 12.7 Person → Cluster (normatives Muster)

1. **Wurzelcluster je Person** (`Sealed`, Space mit `DataClass::Personal`): enthält die Liste der Dokument-Cluster (Space `docs_index`). Dokument- und abgeleitete Cluster hängen per `link(derived, [wurzel])` an der Wurzel. **Eine Löschung der Wurzel zieht alle Cluster-Nachfahren im Gate mit.**
2. **IdentityDirectoryOwner oberhalb des Kerns:** Die Zuordnung `person_id → root ClusterId` liegt nicht in einem `Plain`-Space. Der Directory-Key ist `HMAC(K_dir_tenant, canonical_person_id)`; `K_dir_tenant` bleibt ausschließlich im KMS, nie in Backups oder Storage. Der Directory-Wert `ClusterId` ist zusätzlich mit einem eigenen, nicht exportierbaren `EntryDEK` je Person verschlüsselt. Für Löschung nach bekanntem Cluster hält der Owner einen Reverse-Lookup `HMAC(K_reverse, ClusterId) → DirectoryKey`; `K_reverse` bleibt ebenfalls im KMS. `EntryDEK`-Key-IDs werden deterministisch aus `HMAC(K_keyid, ClusterId)` gebildet und nie persistiert. `IdentityDirectoryOwner` ist ein registrierter `SegmentOwner` mit `SegmentClass::MixedPerSubjectKey`, Gate-Filter, durablem Watermark, `destroy_entry_key`, `rewrite_without` und `verify_absent`. Bei Personenlöschung werden im selben Orchestrierungsauftrag (a) der Root-Gate samt Cluster-Nachfahren, (b) Vernichtung des EntryDEK und Entfernen beider Lookup-Einträge sowie (c) die zugehörigen Atteste abgewartet. Kein systemweites `Complete`, solange ein Owner-Attest fehlt. Backups enthalten höchstens HMAC-Pseudonyme und nicht entschlüsselbaren EntryDEK-Ciphertext. Das stabile HMAC-Indexlabel ist weiterhin pseudonyme personenbezogene Information: solange ein Backup es enthält und `K_dir_tenant` für Zuordnungsversuche verfügbar bleibt, darf das Receipt nicht `Complete` behaupten. Striktes `Complete` verlangt physisches Entfernen des Labels aus allen retained Backups oder Rotation/Vernichtung der alten Directory-Key-Epoche samt Migration aller verbleibenden Einträge; letzteres ist O(Tenant) und ein expliziter oberer Workflow. Sonst lautet der Status `CompleteWithResidual` mit Backup-Frist/Linkability-Risiko.
3. **Aufträge nach Frist:** `Eraser::pack_expired` (Ablauf je Packgruppe `Month`) erzeugt unabhängige, deterministisch sortierte Tickets je höchstens 4096 Clustern; Restriction blockiert betroffene Cluster und wird im Aggregate ausgewiesen, der Rest läuft separat.
4. **Keine Zuordnungstabelle außerhalb des Löschvertrags:** Jeder persistente Kennung→Cluster-Pfad muss als `IdentityDirectoryOwner` mit per-Subject-Key/Lineage registriert sein oder selbst eine gleichwertige Gate-/Destroy-/Rewrite-/Verify-Attestierung erfüllen; unregistrierte Mappings verhindern systemweites `Complete` (`ext-conformance`).

---

### 12.8 Verbindliche Ergänzungen aus dem Audit (vor G-0/G-F)

Diese Regeln ergänzen die bisherigen Abschnitte und gehen bei Widerspruch den übrigen Abschnitten vor; die Invarianten in §15 gehen ihnen vor. `E-nn` bezeichnet den zugehörigen Befund des Audits. Detaillierte Arbeitspakete und Abnahmen stehen in `crates/contextra-storage/25-r8-audit-remediation.md`.

1. **Branch-Vorbereitung:** Branch-IDs werden per bestehendem `reserve_ids` außerhalb Rang 1 reserviert. SYS_BRANCH-Payload wird auf dem Aufruferthread versiegelt; kein `KeyProvider`, Cipher oder KDF-Aufruf unter Rang 1. Commit-Stamp wird ausschließlich im Sequencer vergeben und steht als äußerer MVCC-/Frame-Stamp bereit, nicht als stamp-abhängiges Feld im versiegelten Payload: `Head`-Fork leitet `fork_stamp` aus dem SYS_BRANCH-Record-Stamp ab, `Merged`-Status ebenfalls. Sequencer validiert Reservierung, Parent-Head, Cluster-Generation und Branch-Zustand erneut. Stamp-Vorreservierung pro Branch ist nicht erforderlich und darf keine Commit-Reihenfolge vortäuschen.
2. **Branch-Korrektheit:** `ForkPoint` ist `Head | At(Stamp)`. `Head` linearisiert im Sequencer. `At(S)` ist nur gültig, wenn `history_floor ≤ S ≤ parent_head` und Cluster/Branch nicht retired/closed sind; andernfalls `BelowFloor`/`InvalidForkPoint`, niemals stilles Rebasen. Merge ist v1 nur vom direkten Child zum Parent erlaubt; Source mit lebenden Kindern ist nicht mergbar. Merge-Konfliktmenge schließt Punkt- und Intervalländerungen ein: `Delete`, `RangeDelete` und jeder überlappende `EraseRange` kollidieren; ein bereits aktiver clusterweiter Erase-Marker kann durch Merge nicht überschrieben werden. Merge-Plan bindet `source_head`, `target_head` und `merge_base`; Commit prüft diese erneut atomar. Erfolgreich gemergte Source-Branches sind terminal (`Merged`); für weitere Änderungen wird ein neuer Branch geforkt.
3. **Idempotenz-Retention:** Idempotenz-Ergebnis (Token-Scope/Hash, Request-Fingerprint, Ergebnis-Stamp/Status und pseudonymes Cluster-Scope-Digest aus sortierten `tomb_id`s) überlebt Redo-`replay_floor` unabhängig, solange TTL aktiv ist. Es wird erst nach Ablauf plus durablem `idem_trim` entfernt. Scope enthält keine Nutzdaten/Klartext-ClusterIds. Retry auf inzwischen `Retired`-Cluster liefert `Erased`/`ClusterRetired`, niemals ein scheinbar neues Apply. Token-Wiederverwendung mit anderem Request-Inhalt wird als `IdemConflict` abgelehnt.
4. **Löschgranularität:** `Eraser::range` bietet für Plain `Logical|Physical|Scrubbed`, für Sealed ausschließlich `Logical`; es gibt **keine** kryptografische Einzelbereichslöschung. `key_epoch`-Rotation macht alte Cluster-Daten erst nach vollständigem Rewrap aller zu erhaltenden Daten und Vernichtung aller alten Epochen unlesbar; sie ist ein eigener, nicht atomarer Workflow und kein v1-Range-Versprechen. Für sofortige Crypto-Erasure müssen Daten in getrennte Cluster/Löschdomänen gelegt werden. Der Marker wirkt über alle Branches und historischen Views und kann nicht gemergt/überschrieben werden.
5. **Space-Drop:** `SpaceDropIntent{space_tomb_id,level}` wird vor dem Space-Gate dauerhaft geschrieben; HMAC-Pseudonym plus SpaceId im Backup reicht, um das Space-Gate beim Restore wieder zu gaten. `BLOCK_ERASE` blockiert vor Intent. V1 erlaubt auf Space-Ebene kein Crypto-Erase, weil Cluster mehrere Spaces teilen können; dafür sind Cluster-Tickets nötig. Nach Intent folgen Gate, `GateApplied`, Anchor und für Plain erst danach Purge. `SpaceDropped` ist nur finaler Abschluss nach erforderlichen Attesten. Keine Klartext-IDs oder Einzel-Edit-/Ein-Fsync-Behauptung für beliebig große Spaces.
6. **Klartext-Reclamation:** `Zeroizing` ist eine Maßnahme, kein Beweis für sämtliche Kopien. Memtable-/Cache-Interfaces verbieten unkontrollierte Clone-Pfade für Klartext; Ausgaben liegen in Zeroizing-Puffern. Gate erhöht Generation, entzieht neue Zugriffe und wartet auf Leser-Epochen bzw. weist noch aktive Leser als Exposition aus, bevor `Purged`/Receipt Abschluss behauptet. Allocator-/OS-Kopien sicher zu nullen ist keine Garantie und bleibt Restrisiko.
7. **Blob-Purge:** Purge darf nicht auf `punch_hole` angewiesen sein. Überschreitet der zu entfernende Anteil den konfigurierten Hole-Ratio- oder Max-Fragment-Grenzwert, wird eine neue `blob_id` mit gepackten Überlebenden geschrieben und per Catalog-Edit ersetzt. Reserveplatz und maximale Rewrite-Gruppe sind harte Admission-Grenzen. `Plain`-Barriere invalidiert Consumer explizit mit Epoch-Wechsel/`Lagged`; kein Consumer darf eine Lücke als verarbeitet bestätigen.
8. **Durable Extensions:** `watermark` darf nur bis zum höchsten angewandten `durable_stamp` reichen. Extension-Builder liest nur durable Views oder durable Feed-Frames. Query-Fassade erzwingt `filter_visible` nach Kandidatenfund und erneut vor Rückgabe (Generation-Check); ungefilterte Indexresultate sind kein unterstützter API-Pfad.
9. **Mixed-Segment-Erasure:** Jeder `SegmentOwner` deklariert `SegmentClass = ClusterSections | MixedPerClusterKey | MixedPerSubjectKey | Rebuildable{max_age} | NoPersonalData`, Schutzdomäne, Löschgranularität, RAM-Budget, Rewrite-/Rebuild-Obergrenze und Attestationsformat. `MixedPerSubjectKey` ist ausschließlich für Subjekt-Mappings wie das IdentityDirectory zulässig: jeder Record hat einen eigenen KMS-EntryDEK, die Key-ID wird deterministisch aus einem pseudonymen Cluster-Handle abgeleitet und ist nicht im Store abgelegt; der Owner attestiert `destroy_entry_key`, `rewrite_without` und `verify_absent`. Für `Sealed` muss jeder clusterbezogene Payload mit Cluster-Schlüssel/Epoche oder gleichwertig isoliertem Schlüssel geschützt sein; ein globaler Segment-Key reicht nicht für `INV-ERA-7`. `Rebuildable` bleibt bis zum bestätigten Rebuild `Purging`, nie `Done`. `NoPersonalData` ist deklarativ und erhält keine Crypto-Erasure-Zusage.
10. **RAG-/Audit-Grenze:** Ein `Change` kann Dokumentdaten, quantisierte Vektoren, BM25-Postings und Graph-Deltas atomar persistieren, sofern sie als Records derselben Cluster-Löschdomäne modelliert sind. Globale HNSW-Nachbarschaftsreparatur bleibt abgeleiteter Index-Job und wird nicht im selben Kern-Commit garantiert. Sie nutzt durable Watermarks, Gate-Filter und attestierten Rewrite/Rebuild. `ErasureReceipt` attestiert nur Kern und registrierte Extensions; signierte v15-`DeletionProof`s entstehen über eine versionierte Bridge aus Receipt plus Extension-Attesten. Ohne vollständige Atteste kein systemweites `Complete`.
11. **Gesamtsystem-Ressourcenbudget:** Storage und Extensions teilen einen `MemoryBudget` mit getrennten Limits für Memtables, Cluster-Metadaten, Guard-Fenster, Block-/Key-Cache, Index-Delta, In-flight-Rewrites und temporären Purge-Platz. Admission/Backpressure erfolgt vor Commit/Index-Job; keine Schicht darf unbeschränkt RAM oder Rewrite-Scratch allokieren. Budgets werden gemeinsam im RAG-Lastprofil gemessen.
12. **Archive-Konsolidierung:** Ältere Lethe-Entwürfe sind Designmaterial, keine zusätzliche Norm. Übernommen werden SYS_IDEM-Retention, Extension-Klassen/Atteste und gemeinsames Speicherbudget. Nicht übernommen werden globale EU-Tags/Content-Addressing, In-place-Scrub als Durability-Annahme sowie Zusagen kryptografischer Teillöschung.
13. **Schlüsselzustände (E-56):** `KeyUnavailable`, `KeyDestroyed/Absent`, `KeyDisabled` und `KeyLost` sind getrennte Zustände. Nur bestätigtes `Absent` erfüllt Crypto-Erasure; `Unavailable` blockiert mit Retry, `Lost` ergibt `Incomplete`/Quarantäne, `Disabled` bleibt `KeyDestroying`/`Blocked` mit Frist und Restrisiko. Provider-Nichterreichbarkeit darf nie als Vernichtung interpretiert werden.
14. **Redo-Crypto-Domäne (E-57):** RootKey-abgeleiteter Redo-Key ist nur Ketten-MAC-Schlüssel. Jeder Sealed-Redo-Op ist per eigener Cluster-Key-Epoche versiegelt; Op-Header bindet `key_epoch`. Redo-Rohreste werden nach `destroy_many` unlesbar. Gate-/Routing-Header bleiben als pseudonyme Metadaten sichtbar und sind ausdrücklich kein Nutzpayload.
15. **Blob-AAD (E-58):** Blob-AAD bindet stabile `record_uid`, `instance`, `space_id`, `cluster`, `key_epoch`; nie `blob_id`/Offset. Dadurch ist Survivor-Rohkopie in neue Blob-ID/Offsets gültig, ohne AEAD-Aufbrechen. Neue ID/Offsets bekommen eigene CRC/Directory.
16. **Diff-Kosten (E-59):** Branch-Diff- und Merge-Konfliktkandidaten kommen aus cluster-sealed, stamp-sortiertem `ChangeIndex` für Punkt-/Intervallmutationen beider Seiten. Allgemeiner Diff kostet Änderungen seit gemeinsamem Floor plus Ausgabe, nicht nur Branch-Einträge. ChangeIndex-Format, GC und Branch-Pins sind G-F-Bestandteil.
17. **Gewichtsvertraulichkeit (E-61):** `weight_sum[W]` und Tokenprofile sind bei Sealed-Clustern mit dem Cluster-Key verschlüsselt. `page` darf kleine SubIndex-Metadatenblöcke entschlüsseln, liest aber keine Body-Blöcke. Klartext-ClusterDirectory enthält keine Gewichte.
18. **Extension-Liveness (E-60):** Owner-Aufrufe besitzen Deadline und Fencing-Generation. Timeout erzeugt `Blocked{owner, reason, since, retry_at}` + Ledger-Ereignis/Alarm/Eskalation; verspätete alte Generation wird verworfen. Persistente Feed-Consumer, die Derivate schreiben, müssen SegmentOwner/Lineage registrieren. Nicht attestierte externe Kopien erscheinen als Residual und verhindern systemweites `Complete`.
19. **Purge-Reserve (E-63):** Physischer Rewrite nutzt getrennte, geschützte, vorallokierte Scratch-Slots; Standardpool ≥ zwei maximale Run/Blob/Redo-Dateien (128 MiB bei 64-MiB-Grenzen), ein Rewrite-Writer. Purge arbeitet pro Datei mit atomarem `PurgeBatchEdit`, direktem Unlink vollständig toter Dateien und finalem `PurgeCompleteEdit` erst nach allen Acks. FS-CoW/Quota muss vorallokierte Platzgarantie belegen; sonst wird harter DiskFull-Purge nicht zugesichert.
20. **Löschkostenvertrag (E-62):** `EraseLocality::ClusterLocal` isoliert Run-/Blob-Dateien und vermeidet dort fremde Survivor-Rewrites. Gemeinsam genutztes Redo, Retention-Pins und Checkpoints bleiben separat begrenzt/attestiert; daraus folgt keine allgemeine O(Clusterbytes)-Garantie. `PackedBounded` erlaubt Survivor-I/O bis zur Summe der betroffenen Dateien. Globale Rebuildable-Indizes berichten eigene Kosten/Fristen.
21. **Zeit- und Link-Semantik (E-64/E-65):** TTL/Deadline nutzt monotone Laufzeit + UTC + durablen `clock_floor`; Standard-Skew ist 5 min, Compliance-Profile dürfen ihn nur explizit verkürzen. Sprünge außerhalb Toleranz stoppen Retention-/Hold-Übergänge fail-closed und markieren Friststatus `ClockUncertain`. Cross-subject DerivedFrom-Kaskaden sind unzulässig; gemeinsame Ableitungen brauchen getrennte Löschdomänen oder nicht-kaskadierende Referenzen.
22. **Limit-Konsistenz (E-66):** `max_body_bytes` ist hart 4 MiB, `max_change_bytes` 8 MiB, `max_body ≤ max_change/2`; §4.6, Ports-Konstanten und Config müssen dieselben Werte verwenden. H-21 misst bis zur konfigurierten Branch-Maximaltiefe; Zielverfehlung senkt Default oder erzwingt Flattening.
23. **Purge-Edit-Zustandsautomat (E-67):** Jede Dateiersetzung/-entfernung ist ein eigenes durables `PurgeBatchEdit`; `files_remove` pinnt die Quelldatei im persistenten Graveyard. Unlink folgt nach Reader-Drain und `sync_dir`; `UnlinkAckEdit` räumt den Graveyard erst danach. Nur `PurgeCompleteEdit` darf bei leerem Ticket-Graveyard, allen Owner-Attesten und finalem Verify `Purged/Done` setzen. Recovery setzt Batches idempotent fort und kann nie aus einem Zwischenbatch `Done` ableiten.
24. **Krypto-Implementierungsabgleich (E-68):** Normativ ist XChaCha20-Poly1305 direkt mit dem bereits scope-/epoch-spezifischen Schlüssel; `Authenticated` ist domain-separated HMAC-SHA256 über Modus, Nonce, AAD und Payload. Die vorhandene `derive_subkey_fast`-Vorstufe im Referenzcode weicht davon ab und ist vor G-F zu entfernen oder auf einmalige Schlüsselrollen-Ableitung außerhalb jedes SealFrame zu begrenzen. Kein G-F ohne unabhängige Vektoren, die genau diese Byte-/Schlüsselableitung verifizieren.
25. **Begrenzte Löschkaskade (E-69):** Jede durch `DerivedFrom` mögliche transitive Löschhülle ist bei Link-Admission auf höchstens `max_gate_clusters=4096` zu begrenzen. Hülle außerhalb Rang 1 berechnen und Graph-Generation beim Commit revalidieren; bei Überschreitung `EraseFanoutExceeded` ohne Teilmutation. So bleibt die Zusage „eine Löschhülle, ein Gate-Frame“ innerhalb der Change-Limits.
26. **RAG-Index-Löschdomänen (E-70):** Clusterübergreifende HNSW-/CSR-Kanten, BM25-Postings, Centroids und Routing-Metadaten sind nicht durch Cluster-Key-Zerstörung erledigt. Sie müssen als bounded `Rebuildable` mit Quell-Lineage, Rewrite-Limit, Deadline und Verify attestiert werden. `on_gate`/Query-Owner sperrt Kandidaten vor Payload-Auswertung und vor Ausgabe; gemeinsame Segmente bleiben bis bestätigtem Rewrite `Purging`. Kein Storage-Receipt darf allein daraus eine systemweite Crypto-Erasure ableiten.

27. **Ledger-Tomb-Epochen (E-74, K-31):** `tomb_epoch u32` je Tomb-tragendem Ledger-Event (Format-Slot vor G-F); `tomb_key(epoch)` im `KeyProvider`, nie in `KEYS`/Backups; Default eine Epoche ohne Retire. Retire nur für geschlossene Epochen nach `backup_horizon` mit `restore_floor`, Anker und Provider-`Absent` (§10.4); danach sind die Ledger-Pseudonyme anonym, die Kettenprüfung bleibt (`INV-LED-2`, `INV-KEY-5`).
28. **Löschauftrag auf `Poisoned` (E-75):** durables `EraseIntent{origin: Poisoned}` im Ledger plus `pending_erase`/`ErasePending` auf dem vergifteten Handle; Gate beim Neustart vor jeder Nutzer-Sicht; kein KeyDestroy/Purge ohne durables Gate (§9.5.2, `INV-ERA-13`).
29. **View-Lease (E-76):** `view_lease` 60 s, verlängerbar bis `view_lease_max`; Ablauf wirkt wie `Revoke`, Folgenutzung `SnapshotTooOld`, getrennt von `BelowFloor` (§11.4, `INV-VIEW-3`).
30. **Schlüsselzusicherung (E-77):** `KeyProviderInfo{destroy, keys_in_backups: KeysInBackups, assurance: KeyAssurance}`; Statuscaps `BoundedBackupKeys{until}` und `WeakKeyAssurance`; `ProviderAssuranceInsufficient` beim Öffnen in `OpenProfile::Production` (§9.4). Der Code im Ports-Crate führt bisher nur die vier Batch-Methoden und ist vor G-0 anzugleichen (WP-01.5).
31. **Host-Befund (E-78):** optionaler Port `HostProbe` (Swap, Core-Dumps, `RLIMIT_MEMLOCK`, Dateisystemtyp, Geräte-IDs); nur Adapter und Diagnose, nie Korrektheitsquelle; Werte `Unknown` ⇒ `Declared` (§3.2, §9.9).
32. **Block-Pruning (E-79, K-29):** `ExtensionBuilder::block_boundary` und `BlockPruner::may_skip` nur für deklariert schattensichere Besitzer auf `main`; keine Lanes/Aggregate im Kernformat (§12.2, `INV-PRUNE-1`).
33. **Messarten (E-80):** jede Messung im Verifier-Bericht trägt `MeasureKind {Complete, Sample{n,N}, Declared, NotMeasured, NotImplemented}`; nur Pflichtmessungen mit `Complete` erlauben `Complete` (§13.2.1, `INV-VER-1`).
34. **Backup-Horizont und Gerätetrennung (E-81):** `Storage::erase_epoch()`, `BackupMark`, `BackupBeyondHorizon`, `LedgerUnavailable`, `LedgerSameDevice` (§9.7, §10.4).
35. **Derivation Cache (E-82, K-32):** Konvention der oberen Schicht mit clusterlokalem Zufallssecret, nur in `Sealed`-Clustern (§12.6); `INV-PRIV-1` entsprechend präzisiert.

### 12.9 Agenten, Sitzungen, Checkpoints, Decay, KV-Cache (K-45, K-46)

| Konzept | Zuordnung | Regel |
|---|---|---|
| Sitzungsbaum (`SessionBranchTree`) | Knoten als Records unter `sess/<id>/…` im Cluster des Eigentümers (Konvention „Session ≠ Cluster“, §12.6) | Der Baum ist Anwendungsdaten; Löschung des Eigentümers nimmt ihn mit |
| Spekulative Pfade | clusterlokale Storage-Branches (`fork(Head|At)`, `ttl`, `merge`, `branch_drop`, §8) | O(1), crash-sicher, atomar; Merge-Auflösung liefert die obere Schicht (LWW, Union, semantisch) |
| Zustands-Checkpoints (`CheckpointGuard`, `StateCheckpoint`) | clusterlokale Records plus `View-Lease`/`Hold` (≤ 24 h) | kein unbegrenztes Pinning, kein Klartext-Manifest mit `blake3`; ein vergessener Guard gibt nach der Lease frei |
| DB-Backup/Restore | `contextra-storage::checkpoint` (§10.3) | enthält weder Schlüssel noch Ledger; trägt `BackupMark` |
| Kognitiver Decay (`AdaptiveDecayController`) | verändert Relevanz/Gültigkeit im Kopf, löscht nie | **kein** Gate, kein Receipt; Compliance-Löschung ist ausschließlich `Eraser` (`INV-DECAY-1`) |
| Konsolidierung („Sleep Cycles“) | Work-Queue-Records plus Consumer; Ergebnis ist ein normaler `Change` mit Producer im Kopf | Abgeleitete Zusammenfassungen erben das Minimum des Trust-Labels und sind per `DerivedFrom` mit ihren Quellen verbunden, wenn eine Quelllöschung sie mitziehen soll |
| KV-/Prompt-Cache (`contextra-kvcache`) | `SegmentOwner` der Klasse `MixedPerClusterKey` (Schlüssel je Session-/Personen-Cluster) | Cloud-Provider-Caches liegen außerhalb und stehen als Restexposition im Receipt (§9.9) |

### 12.10 Produktfunktionen, Compliance-Profile, Auskunft und Export (K-56, K-57)

**Funktionen, die aus der Architektur folgen und für Kunden sichtbar sind** (jede ist an einen Beweis-Gate gebunden, §13):

| Funktion | Grundlage | Kundennutzen |
|---|---|---|
| Nachweisbare Löschung mit Receipt, Fristüberwachung und Restexposition | §9 | Rechenschaft (Art. 5 Abs. 2, Art. 17 DSGVO) ohne Handarbeit |
| Zeitreise-Retrieval („was wusste der Agent bei Stamp S?“) | `view_at`, `SnapshotSet` | Reproduzierbare Antworten, Audit, Fehleranalyse |
| Agenten-Experimente ohne Datenleck | clusterlokale Branches | Risikofreies Ausprobieren, vollständig löschbar |
| Tokenbudget-Retrieval mit geringem I/O | `page`/`weights`/`DiBud` | Weniger Kontextkosten und Latenz |
| Auskunft und Export je Person/Fall in einem Zug | `export_cluster` | Art. 15/20 ohne Suche über mehrere Systeme |
| Einschränkung der Verarbeitung | `Restriction` (`BLOCK_READ`/`BLOCK_WRITE`/`BLOCK_ERASE`) | Art. 18 und gesetzliche Aufbewahrung (`reason`-Codes) |
| Löschfristen je Packgruppe (Monat/Quartal/Jahr) | `pack_expired` | Aufbewahrungsende ohne Einzelaufträge |

**Compliance-Profile.** Ein Profil ist ein Bündel geprüfter Einzelwerte; `open` verweigert Kombinationen, die dem Profil widersprechen.

| Wert | `Development` | `Standard` | `Strict` |
|---|---|---|---|
| Schlüsselzusicherung | beliebig | `Kms` oder `Hardware` | `Hardware` |
| `KeysInBackups` | beliebig | `No` oder `Bounded` | `No` |
| Anker | `Weak` erlaubt | `Required`, `Weak` nur mit Bestätigung | `Strong` |
| Ledger-Gerät | gleich erlaubt | getrennt (Standard) | getrennt, anderes Failure Domain |
| `Personal` | Warnung | `Sealed` erzwungen | `Sealed` + Provider-Attest |
| Receipt-Status ohne Pflichtmessung | informativ | `Incomplete` | `Incomplete` + Alarm |
| `ryw_wait`/`IndexLag` | tolerant | Standard | strenger Alarm |

Abweichungen von einem Profil (z. B. `acknowledged_weak_keys`) erscheinen im Receipt als Restexposition; ein Profil ist kein Rechtsurteil.

**Auskunft/Export (K-57).** `Storage::export_cluster(view, cluster)` liefert einen Strom `(branch, key, header, body)` über eine konsistente Sicht, ohne Schlüsselmaterial und ohne Indexstrukturen, mit Gate-Prüfung vor jedem Block; für `Erased`-Cluster `Err(Erased)`. Der Export hält die View-Lease und ist rate-begrenzt (`export_io_fraction`). Formatierung, Prüfung auf Rechte Dritter und Übermittlung liegen oben.

---

## 13. Verifikation

### 13.1 Ebenen

| Ebene | Werkzeug | Inhalt |
|---|---|---|
| Formal | TLA+/TLC (oder Quint) | M1 Commit-Pipeline (Intake, Sequencer, Syncer, Publikation, `Retiring → Retired`)/Sichtbarkeit/Recovery/Stamps/Guards (Konfliktfenster, Fensterfuß) · M2 Erase (ER0–ER7, Gate, Restriction, Link, Consumer-Floors, Space-Drop, **Branch-Ops und Merge**) · M3 Compaction × Erase × Claims × Staging × Fork-Floor · M4 Checkpoint × Erase × Restore · M5 Rebucket · M6 Anker × Edit × Rollback |
| Code-Modell | `loom` | Sperren, `Arc`-Swap, Generationszähler, Claims, Graveyard, Leader-Wechsel (kein verlorener Wakeup), Pipeline-Credits, View-Widerruf |
| Simulation | eigener Scheduler + `shuttle` | **Hauptwerkzeug gegen Races:** seeded Interleavings, `FaultFs`, `SimClock`, `SimEntropy`, `SimKms`, reproduzierbarer Seed; Race-Nahtstellen RACE-1…RACE-22 als benannte Szenarien |
| Absturz | `FaultFs` | Crash an **jedem** E/A-Index, Torn Writes, verlorene Fsyncs, Rename-Reihenfolgen, Rollback-Versuche, `ENOSPC` beim Vorallokieren, **Geister-Journal** aller je geschriebenen Seiten |
| Eigenschaft | `proptest` | Codecs, `cmp`, Siegel-Roundtrip und Rohkopie, Compaction ≡ Referenzmodell (`BTreeMap` mit Versionen und Branches), Purge ≡ Modell ohne Cluster, Paging (Budget nie überschritten, lückenlos, vorwärts/rückwärts gleiche Menge, Block-Skipping ≡ Vollscan), Branch-Lesen ≡ Modell, Merge ≡ Modell |
| Lokalität | Byte-Zähler | `ClusterLocal` und `PackedBounded` separat bei Store ×1/×10/×100; Run/Blob, gemeinsam genutztes Redo, Checkpoints und Extensions getrennt zählen. Bounds für Redo-Retention, Pins, Packgruppen und Extensions explizit belegen; keine pauschale O(Cluster)-Behauptung |
| Mutation | `cargo-mutants` | Score auf `codec`, `recovery`, `erase`, `sequencer`, `syncer`, `branches` |
| Linearisierbarkeit | Wing-Gong | Historien aus der Simulation (inkl. Guards, Merge) |
| Löschorakel | `ErasureOracle` | siehe §13.2 |
| Provider-Konformanz | `KeyProviderConformance` | jeder `KeyProvider` |
| Fuzz | `cargo-fuzz` | Decoder aller Formate, Recovery auf beschädigten Verzeichnissen (≥ 8 Ziele); Bit-Flip-Differenztest: Corrupt/Auth-Fehler, nie stille Falschdaten; `miri` auf `codec`/Siegel |
| Statisch | clippy, `cargo-deny`, `layering`, §3.6, `cargo-geiger` | |
| Dauer | Soak 72 h, Crash-Loop, Platte voll | |
| Retrieval-Integration | Differentialtests, Sim, `ErasureOracle` | `Index ∪ Delta ≡ Vollscan` (Text/Graph bitgleich, Vektor-Recall ≥ Basis) · Race Gate × Query und Gate × Rebuild · Crash-Sweep über IG-1…IG-6 (nie halb sichtbar) · `SnapshotSet`-Determinismus (`INV-RAG-4`) · Canary-Personen mit Pseudonym-Tresor (`pii_residual_rate`, `INV-SEP-1`) · Partition-Generationswechsel mit Schlüsselvernichtung (`O-Segments`, `O-Files`, Backup-Kopie alter Generation unlesbar) · Verbotsprüfung `INV-LAYER-1` |

### 13.2 Löschorakel

Jeder Test-Cluster erhält eindeutige Canaries in Schlüssel, Kopf, Körper, `weights`-Kodierung, Idem-Token, Branch-Namen und Schlüsselmaterial (Test-Provider gibt seine Schlüssel preis); zusätzlich 16-Byte-Stichproben aus jedem Siegel.

| Orakel | Durchsucht | Erwartung |
|---|---|---|
| O-Files | Catalog-lebende Runs, Blob-Inkarnationen, Redo einschließlich `retain_floor`-Segmente, `.building` und registrierte purgepflichtige Segmente; getrennt davon Checkpoints/Cache-Dumps und externe Snapshots | nach `Done`: keine betroffenen Nutzdaten in lebenden Dateien; registrierte Retention-Copies sind attestiert entfernt, nicht verwaltete Kopien bleiben explizites Receipt-Restrisiko. Restore-Reconcile prüft externe Backups, zählt sie aber nicht als live purged |
| O-Device | Geister-Journal | keine Klartext-Canaries, keine Schlüssel; Ciphertext-Treffer erlaubt und als G1-Schutz ausgewiesen |
| O-Gate | alle `view/page/poll`-Aufrufe ab Gate, alte Snapshots, Cursor, Retries, Extension-Antworten | kein Inhalt |
| O-Ledger | Ledger-Bytes | nur `tomb_id`, keine ClusterId, keine Payload |
| O-Segments | `verify_absent` | für jedes registrierte Segment `true` |
| O-Restore | nach `restore` alter Checkpoints | `Erased`-Cluster tauchen nicht wieder als `Live` auf |
| O-Survivors | Überlebende im selben Packed-Run | bytegleich lesbar (Hash der Sektionen vor/nach) |
| O-Process | `gcore` nach Löschung | best-effort, **kein Beweis**, im Bericht so deklariert |

Das Orakel läuft **zwischen** den Phasen (Crash in Phase 2/3, Recovery, Fortsetzung).

#### 13.2.1 Messarten (E-80)

Jede Messung im Verifier-Bericht (`verifier_report_digest` bindet die geordnete Liste `{name, kind, scope_count}`) trägt eine `MeasureKind`:

| Art | Bedeutung |
|---|---|
| `Complete` | Vollprüfung über den gesamten Scope |
| `Sample{n,N}` | Stichprobe von `n` aus `N` Objekten |
| `Declared` | vom Betreiber/Adapter erklärt, vom Kern nicht prüfbar (z. B. O-Process, Swap verschlüsselt) |
| `NotMeasured` | nicht ausgeführt |
| `NotImplemented` | Prüfer fehlt (Platzhalter) |

Regeln: Pflichtmessungen sind die strukturellen Prüfungen (a)–(d) von `Eraser::verify` (§9.7) und müssen `Complete` sein, sonst Status `Incomplete`. Alle übrigen Messungen sind informativ: `Sample`/`Declared` begrenzen auf `CompleteWithResidual`, wo sie für den Level relevant sind; `NotMeasured`/`NotImplemented` werden ausgewiesen. Berichte und Receipts nennen nur `Complete`-Messungen „verifiziert“. Normalfall der Orakel: O-Files, O-Ledger, O-Segments, O-Gate und O-Restore im Testlauf `Complete`; O-Survivors `Sample`; O-Device `Sample`; O-Process und Host-Befund `Declared`; externe Backups `Declared`. CI prüft, dass kein Receipt mit `Complete` entsteht, wenn eine Pflichtmessung nicht `Complete` ist (`INV-VER-1`).

### 13.3 CI

Pflicht: `check`, `fmt`, `clippy -D warnings`, `test`, `layering`, `loom`, `doc`, `deny`, Sim- und Absturz-Kurzlauf, INV-Zitierprüfung (jede ID aus §15 in mindestens einem Test per `// INV-…`), Abdeckung. Nachtlauf: lange Sim, Fuzz, Orakel-Vollsuche, Mutation, Provider-Konformanz. **Ohne funktionierende CI ist kein Gate gültig.**

---

## 14. Phasen und Gates

Planungsannahme: 3–4 erfahrene Rust-Entwickler plus 1 Test-/Simulationsingenieur; Dauern sind grobe Schätzungen, keine Zusage. Die Phasen P0–P8 betreffen den Speicherkern; die **Retrieval-Spur R1–R5** läuft parallel auf den freigegebenen Verträgen und ist an dieselben Gates gebunden.

| Phase | Inhalt | Gate | ca. |
|---|---|---|---|
| **P0 Fundament & Topologie** | `contextra-storage-ports` als Level-0-Crate anlegen (heute fehlt es: `contextra-storage` importiert `contextra_storage_ports` ohne Abhängigkeit, der Kern kompiliert nicht), `codec`, `env` (MemFs, RealFs, FaultFs, Sim), `sync`, Anker, `crypto-ref` + Konformanz, Orakel-Skelett, CI, Lint-Gerüst, Referenzcodierer; `Cargo.toml` ⇄ `capabilities.toml` ⇄ `deny.toml` angleichen; `layering`/`unsafe_islands` als echte Cargo-Tests (§18.1) | **G-0:** Spec normativ (zwei Reviewer), CI grün, `cargo check -p contextra-storage` grün | 3 Wo. |
| **P1 Walking Skeleton** | ein Space, `apply`, Redo + Memtable, `view`, Cluster-Register, Gate, `destroy_many`, Ledger, Recovery aus Redo; Formate provisorisch | Crash-Sweep über alle Fs-Operationen grün; O-Files findet nach `Done` nichts | 4 Wo. |
| **P2 Runs und Catalog** | Flush, ClusterSection, Staging, Catalog + `CURRENT`, Purge, Plain-Barriere, vollständige Recovery, Absturzmatrix | O-Files für Daten in Runs grün; Modelltest; M1, M2 (ohne Branches); **Spike-Messungen als Freeze-Voraussetzung:** H-21, H-11, H-25, H-26 | 5 Wo. |
| **G-F Formatfreeze** | alle Formate (§5.9) inkl. reserviertem `branch`-Feld, goldene Vektoren, Rohkopie-Test, ≥ 8 Fuzz-Ziele; zusätzlich **Partitionsformat** der Indizes (§12.4.5) und `PartitionGeneration`-Schlüsselscope | Freeze-Review, Referenzcodierer vollständig | — |
| **P3 Schreibpfad** | Gruppen-Commit, Reservierungen, Guards zweistufig, Volatile, `DiskFull`, Idempotenz, Backpressure | **G-W:** Absturzmatrix, Linearisierbarkeit in Sim, H-1, H-18, H-24 | 4 Wo. |
| **P4 Lesepfad** | Blobs, Cache, Views (Revoke, Lease), `page` mit Gewichten, Block-Skipping, `ResumeToken`, `filter_visible` (inkl. `DocHead`) | **G-R:** Modellgleichheit, Paging-Properties, H-2…H-4, H-25 | 4 Wo. |
| **P5 Wartung** | Compaction je Wartungsgruppe, Claims, Blob-GC, Governor, Rotation, Consumer-Feed aus Redo | **G-M:** M3, Sim ohne Invariantenbruch, H-5, H-19, H-26, H-27, H-28 | 5 Wo. |
| **P6 Löschung komplett** | Links, Restrictions, Abwurf, Space-Drop, `range`, Ledger/Anker, Checkpoint/Restore, `keys-file`, `keys-cohort`, `SegmentOwner`/`ExtensionBlock`, Receipt + `verify_receipt`, Tomb-Epochen, Löschauftrag auf `Poisoned`, volle Orakel | **G-E:** M2/M4/M5/M6, Orakel 0 Treffer, INV-ERA-11, H-6…H-11, H-17, Konformanz | 6 Wo. |
| **P7 Versionierung** | `BRANCHES`-Bit, `SYS_BRANCH`, Fork/Drop/TTL, Branch-Lesen, Diff, Merge, `Eraser::branch`, Fork-Floor | **G-V:** Branch-Properties vs. Modell, M2/M3 mit Branches, H-20…H-22 | 5 Wo. |
| **P8 Härtung** | Soak 72 h, Dauer-Fuzz, Platte voll, Fehlerinjektion, Fassade, Abdeckung, Krypto-Review, Messbericht, Bedrohungsmodell-Review | **G-P:** alle INV getestet (100 %), H-15/H-16, Messbericht §17 | 5 Wo. |

**Retrieval-Spur (parallel, an Verträge gebunden).**

| Schritt | Inhalt | Gate |
|---|---|---|
| **R1 Persistenzneutralität** | `vector`/`text`/`graph`: `PartitionSink`/`PartitionSource` einführen, `memmap2`, `std::fs`, festen Integritätsschlüssel, `bincode`, unkeyed `blake3`, `tokio`/`scc` aus Produktivpfaden entfernen; Klasse-A-Build aus leerem Verzeichnis bleibt grün | **G-R1:** `check_forbidden_apis.sh` 0 Treffer, bestehende Invarianten-Tests grün |
| **R2 Index-Bridge** | `contextra-index-bridge`: `SegmentOwner`/Feed/Watermark/Attest für Vektor, Text, Graph, IdentityDirectory; Partitionen, Generationsschlüssel, `rewrite_without`, `verify_absent` | **G-R2:** Konformanzsuite je Owner, `ErasureOracle` `O-Segments` und `O-Files` ohne Treffer |
| **R3 Query-Plan** | `contextra-engine`: QP-1…QP-9, `SnapshotSet`, Delta-Pfad, `min_stamp`, Fusion | **G-R3:** Differential `Index ∪ Delta ≡ Vollscan`, Race Gate × Query, Determinismus (`INV-RAG-4`), H-29…H-31 |
| **R4 Ingest/Publish** | Publish-Record, Privacy-Routing, Pseudonym-Tresor, Idempotenz je Teil-Change, Orphan-Worker | **G-R4:** Crash-Sweep über IG-1…IG-6, `pii_residual_rate`-Messung (H-34), Partial-Ingest nie sichtbar |
| **R5 Agenten & Migration** | Sitzung/Branch-Mapping, Checkpoint-Umbau, Decay-Trennung, `export_cluster`, Forward-Export aus `contextra-store` (§14.1) | **G-R5:** Branch-/Checkpoint-Properties, Migrations-Rehearsal |

**G-CUTOVER (nach G-P, G-R1…G-R5).** Produktfreigabe: erst wenn sämtliche produktiven `contextra-store`-Verbraucher und Datenräume migriert sind, Restore-/Rollback-Rehearsal einschließlich externem Fencing des alten Instanzbesitzers und RAG-Differentialmessung bestanden und das gemeinsame Speicherbudget belegt ist, wird `contextra-storage` alleiniger Produktions-Schreibkern. Nach den ersten Writes in `contextra-storage` ist die Rückkehr nur per geprüftem Forward-Export oder Restore möglich; **kein Dual-Write.** Bis dahin gelten alle Leistungs- und Löschvorteile als unbestätigte Ziele.

Keine Phase beginnt vor dem Gate der vorherigen; Formatänderungen nach G-F nur per Migration. In jeder Phase wird der Crash-Sweep um die neuen Fs-Operationen **erweitert, nie nachgezogen**.

### 14.1 Ablösung der Altpfade (Bereinigungsplan bis G-CUTOVER)

| Altpfad | Ersatz | Wirksam ab |
|---|---|---|
| `contextra-store` (LSM + WAL), `LsmStorage` | `contextra-storage`; Daten per Forward-Export/-Ingest (NZ-7), kein Formatimport | G-CUTOVER |
| `DbTransaction` + `CommitLedger` + `CompensatingAction` (4-Wege-2PC) | Ein-Wahrheits-Commit + Publish-Record + Feed-Projektion (§12.4.0) | R3/R4 |
| `contextra-mvcc` (`TxId`, `SequenceLog`, SSI) | `Stamp`/`Epoch(u64)`, Guards (SSI-Verhalten bei Bedarf über Lese-Guards `Unchanged{since}`, §6.3) | R3; Standalone-In-Memory-Modus bis Entfall |
| `aes-gcm-siv`, `blake3` in `crypto`/`vector`/`text`/`graph`/`checkpoint` | `XChaCha20-Poly1305`/`HMAC-SHA256`; keine unkeyed Inhalts-Hashes | R1 |
| `memmap2`, `std::fs`, fester Integritätsschlüssel in `vector` | `PartitionSink` → `SegmentIo`; Integrität = AEAD unter Generationsschlüssel | R1/R2 |
| `commit_lock` + globale `meta:stats`/`pl:` in `text` | partitionsbezogene Statistik, Abfragezeit-`df` | R1/R2 |
| `bincode` in `graph`/`text`/`vector` | kanonische Codecs | R1 |
| `pin_checkpoint`/`blake3`-Manifest in `checkpoint` | clusterlokale Records + `View-Lease`/`Hold` | R5 |
| `TxId` ×5 (`types`, `mvcc`, `vector`, `graph`, `checkpoint`), `text::contract::Epoch` | ein `Epoch(u64)` in `types` (K-59) | R1 |
| `content-addressed-kv-cache`, `crypto/kv-encryption` (`aes-gcm-siv`) in `kvcache` | `SegmentOwner` `MixedPerClusterKey`, XChaCha20 (K-61) | R5 |
| `block_on` in `text`, `graph`, `engine`, `checkpoint`, `kvcache`, `crypto` | nur Engine-Rand (L3+), Level-2-Crates synchron (K-62) | R1 |
| `memmap2` in `contextra-sys` ohne Feature-Gate | Feature `mmap`, nie im Index-/Sealed-Pfad (K-62) | P0 |
| `StorageEngine`/`StorageBackend`/`CommitBarrier`/`Purger`/`LedgerSink` in `contextra-ports` | `storage-bridge` bedient sie bis G-CUTOVER (K-63) | R3 |
| `deletion_proof`, `revocation_log`, `kv_shredding` in `crypto` | Bridge-Abbildung Receipt → Proof (K-55) | nach Freigabe |

Jede Zeile hat einen Test, der den Altpfad **verbietet** (Lint, `check_forbidden_apis.sh`, `deny.toml`) und einen, der den Ersatz belegt; ein Altpfad darf erst nach bestandenem Ersatz-Gate entfernt werden.

---

## 15. Invarianten (normativ, jede wird in mindestens einem Test per ID zitiert)

| ID | Aussage |
|---|---|
| INV-STAMP-1/2/3 | Stamps streng monoton über angenommene Changes/Frames · kein Umbruch (`Exhausted`) · nie doppelt vergeben, auch nicht nach Absturz |
| INV-ID-1/2 | Datei-/`tmp_id`, `SpaceId` werden nie wiederverwendet; Space-Gates binden die vollständige SpaceId |
| INV-ATOM-1 | Ein Change ist ganz oder gar nicht sichtbar, auch nach Absturz, über alle Spaces und Branches (nicht für Volatile-Ziele) |
| INV-COMMIT-2 | Ein sichtbarer Eintrag ist durabel (außer `Relaxed`) |
| INV-GUARD-1 | Ein Change wird nur angenommen, wenn jede Guard zur Stamp-Vergabe gilt |
| INV-VIEW-1/2 | Eine View sieht stets dieselben Daten · `visible_stamp` vor `Arc<Version>` |
| INV-VIEW-3 | Nach Ablauf der Lease (plus `revoke_grace`) hält keine View eine Version; jede weitere Nutzung liefert `SnapshotTooOld`, nie Daten einer entfernten Datei |
| INV-VIS-1 | Jedes Ergebnis einer Extension passiert `filter_visible`, bevor es den Aufrufer erreicht |
| INV-PRUNE-1 | Ein Lesevorgang mit `BlockPruner` liefert dasselbe Ergebnis wie derselbe Lesevorgang ohne Pruner; der Kern ruft `may_skip` nur für deklariert schattensichere Besitzer auf `main` |
| INV-VER-1 | Kein Receipt mit Status `Complete`, wenn eine Pflichtmessung nicht `Complete` ist oder ein Statuscap (`BoundedBackupKeys`, `WeakKeyAssurance`, deklarierte Host-/Gerätereste) gilt |
| INV-CONS-1 | Consumer und `view_durable` sehen nur Stamps ≤ `durable_stamp` derselben `instance_id` |
| INV-LOCK-1/2 | Sperren nur aufsteigend · Stamp-Vergabe und `Live → Retiring` nur im Sequencer; `Retiring → Retired` nur durch die Publikation nach durablem Sync |
| INV-CAT-1/2 | Datei lebendig ⇔ durabler `files_add` ohne `files_remove`; entfernte Datei bleibt bis `unlink_ack` im Graveyard · Edit gilt erst nach `sync_data` der Gruppe |
| INV-FMT-1 | Nur kanonische Kodierungen; unbekannte Version/Tags ⇒ Fehler |
| INV-LOG-5/7/8/9 | `replay_floor`-Regel · Reservesegment vorallokiert, alles bis `redo_mark` gültig · Torn-Tail-Regel · nach ER4 kein lebendes Redo-Segment mit Plain-Nutzlast aus H |
| INV-GRAVE-1/2 | Kein Unlink solange eine `Arc<Version>` zeigt; Graveyard bleibt bis `unlink + sync_dir + unlink_ack` · nach `Revoke` hält nach `revoke_grace` keine View eine Version |
| INV-CLAIM-1 | Ein Edit entfernt nur Dateien unter eigenem Claim |
| INV-BLOB-1 | Blob/Inkarnation nur entfernt, wenn kein lebender Run verweist |
| INV-PL-1…5 | Pure/Packed/Staging eindeutig · clusterreine Abschnitte mit abschnittsrelativen Offsets · Compaction mischt nur in Wartungsgruppen · `cluster → Gruppe` nur im selben Edit wie die Dateimengen · kleiner Cluster höchstens eine Datei je Level |
| INV-CLUSTER-1/2 | `ClusterId` nie zweimal vergeben · nicht aus Kennung ableitbar |
| INV-SEAL-1 | Schutzklasse ↔ Space-Siegel (§5.2) |
| INV-ERA-1 | Logische Atomarität, `Erased{at:S}` für jede View, keine Mischsicht |
| INV-ERA-2 | Nach `Purged` enthält keine lebende Datei Bytes des Clusters, auch keinen Ciphertext (Physical wie Cryptographic); bei Cryptographic gilt zusätzlich INV-ERA-7 bereits ab `KeysDestroyed` |
| INV-ERA-3 | `EraseIntent` durable < Gate-Catalog-Edit < `GateApplied` < Anchor < irreversible KeyDestroy/Purge; vor KMS zusätzlich `KeyDestroyIntent`; danach jedes `PurgeBatchEdit` < Unlink+`sync_dir` < `UnlinkAck` < finales ER6/`PurgeCompleteEdit`; `Done` nur nach allen erforderlichen Attesten |
| INV-ERA-4 | Nur Roll-forward |
| INV-ERA-5 | Reads unter `S` in `erased_ranges` liefern `Erased`, nie Daten, nie „leer“ |
| INV-ERA-6 | Nach `Purged` keine lebende Blob-Inkarnation mit Record des Clusters |
| INV-ERA-7 | Nach ER2 ist alles des Sealed-Clusters im Kern ohne Cluster-Schlüssel unlesbar; registrierte Extensions/Backups nur bei nachgewiesen identischer oder gleichwertiger Scope-Key-Vernichtung |
| INV-ERA-8 | Receipt nur bei erfüllten Done-Bedingungen (§9.5) |
| INV-ERA-9 | Nach Restore werden Ledger-Intents vor Open scope-genau angewandt: Cluster ⇒ `Retired`, Range/Branch ⇒ Selector-Maske, Space ⇒ SpaceId-Gate; kein Backup wird vorher sichtbar |
| INV-ERA-10 | Eine BLOCK_ERASE-Restriction in der Link-Hülle verhindert jede Löschung; Prüfung auf dem Aufruferthread mit Revalidierung im Sequencer (`policy_epoch`), kein Override |
| INV-ERA-11 | Gate/KeyDestroy skaliert mit Zielclustern. Purge bearbeitet nur betroffene Dateien/Extents und gepinnte Redo-/Extension-Bereiche. `ClusterLocal` vermeidet fremde Run/Blob-Survivor, garantiert wegen gemeinsam genutztem Redo/Pins aber keine O(Clusterbytes)-Gesamtkosten. `PackedBounded` kostet höchstens die Summe betroffener, je `max_file_bytes` begrenzter Dateien. Kein Vollstore-Rewrite; keine pauschale Cluster-Schranke ohne Bounds für Redo-Retention, Pins und Extensions. |
| INV-ERA-12 | Die Erase-Menge ist der transitive `DerivedFrom`-Abschluss zum Gate-Zeitpunkt, höchstens `max_gate_clusters`; Link-Admission stellt die Schranke sicher; die ganze Menge wird in einem Frame gated |
| INV-ERA-13 | Eine auf einem `Poisoned` Handle aufgenommene Löschabsicht (durabler `EraseIntent`) wird beim nächsten schreibenden Open vor jeder Nutzer-Sicht gated; bis dahin liefern Zugriffe auf die Cluster `ErasePending`; vor dem durablen Gate gibt es weder KeyDestroy noch Purge |
| INV-LED-1 | Das Ledger enthält keine Klartext-ClusterId/SpaceId, Benutzerkeys oder Nutzdaten; selektive Scope-Selectoren sind ausschließlich AEAD-verschlüsselt unter `LedgerScopeKey` abgelegt |
| INV-LED-2 | Jedes `tomb_id` gehört zu genau einer `tomb_epoch`; nach `TombEpochRetired` ist es nicht mehr auf eine ClusterId abbildbar; Retire nur für geschlossene Epochen nach `backup_horizon`; Restore verweigert Kopien unter `restore_floor` (`BackupBeyondHorizon`); die Kettenprüfung ist vom Retire unabhängig |
| INV-ANCH-1 | Anker monoton; Catalog/Ledger hinter dem Anker wird nicht schreibend geöffnet; Anker nie vor dem durablen Edit |
| INV-KEY-1…4 | Schlüsselmaterial nie unverhüllt in Dateien/Checkpoints · `gen` vor und nach Entsiegeln · alte Epoche erst vernichtet, wenn nichts mehr mit ihr versiegelt · `destroy_many` vernichtet alle Epochen |
| INV-KEY-5 | Tomb-Schlüssel stehen nie in `KEYS`, Checkpoints, Backups oder Dateien des Kerns; `retire_tomb_epoch` ist erst mit `probe == Absent` abgeschlossen; ein fehlender Schlüssel einer nicht retired Epoche ist nie ein Retire |
| INV-PRIV-1 | Der Kern persistiert keine Inhalts-Hashes über Klartext (flüchtige Prozessschlüssel-Fingerabdrücke ausgenommen). Ausnahme nur als Konvention der oberen Schicht: Keyed-MAC unter einem zufälligen, im selben Sealed-Cluster abgelegten Secret (§12.6); er ist clusterlokal, stirbt mit dem Cluster und wird nie über Löschdomänen verglichen |
| INV-VOL-1 | Volatile schreibt kein Redo; Mischchange nicht durabel über Absturz |
| INV-LIM-1 | Changes über den Limits werden vor jeder Sperre abgewiesen |
| INV-DISK-1 | Datenträger voll vergiftet nicht; Gate, Schlüsselvernichtung, Purge bleiben ausführbar |
| **INV-BR-1** | Ein Branch-Lesevorgang liefert für jeden Schlüssel die Version, die das Referenzmodell (Eltern-Kette mit Stamp-Kappen) liefert |
| **INV-BR-2** | Ein Merge ist atomar: bei Konflikten ändert sich nichts; bei Erfolg sind alle Auflösungen und der `Merged`-Zustand in einem Frame |
| **INV-BR-3** | `history_floor` eines Clusters ≤ `fork_stamp` jedes lebenden Branches; Branch-Daten und -Metadaten liegen im Cluster und sterben mit ihm |
| **INV-BR-4** | Fork kopiert keine Nutzdaten (O(1)), Branch-IDs werden je Cluster nie wiederverwendet |
| **INV-BR-5** | Branch-bewusster Filter und `BranchTable` liefern nie ein falsches Negativ; Chain-Resolve und Cursor ≡ Referenzmodell, einschließlich `RangeDelete` auf Zwischenebenen |
| **INV-PIPE-1** | Unter Rang 1 findet weder E/A noch ein Provider-/Cipher-/KDF-Aufruf noch ein Warten statt; Publikation (Memtable, `visible_stamp`, `durable_stamp`) erfolgt strikt in Stamp-Reihenfolge und nur nach dem durablen Sync derselben Gruppe |
| **INV-PIPE-2** | Schlägt der Sync einer Gruppe fehl, wird keine Gruppe mit gleichem oder höherem Stamp veröffentlicht; alle sind `Indeterminate`/`Poisoned`; ein `Retiring` ohne durables Gate existiert nach dem Neustart nicht |
| **INV-POL-1** | Gate, Link, Restrict und Lift sind im Sequencer total geordnet; ein Gate übernimmt nur eine Hülle, die zu seiner `policy_epoch` berechnet wurde oder die der Sequencer selbst berechnet hat |
| **INV-ADM-1** | Cluster-Metadaten, `ClusterLocal`-Cluster, Live-Dateien und Dateideskriptor-Reserve werden nur nach erfolgreicher Admission (Budget/Quote) angelegt; keine stille Herabstufung der Löschlokalität |
| **INV-RAG-1** | Kein Ingest ist halb sichtbar: Chunks einer Dokumentversion sind ausschließlich über `doc/<id>/head` erreichbar; erst der Abschluss-Change macht sie für Leser und Indexprojektion sichtbar |
| **INV-RAG-2** | Eine Indexprojektion wendet eine Dokumentversion nur als Ganzes an; ihr Watermark reicht nie über `durable_stamp` derselben `instance_id` |
| **INV-RAG-3** | Eine Abfrage mit unbedecktem Bereich `(applied_stamp, V]` liefert Delta-Ergebnisse oder `IndexLag`, nie stillschweigend unvollständige Treffer |
| **INV-RAG-4** | Gleiche Anfrage und gleiches `SnapshotSet` ⇒ bitgleiches Ergebnis, unabhängig von Threadzahl und Hash-Reihenfolge |
| **INV-PART-1** | Jede Index-Partition gehört genau einer Packgruppe und überschreitet ihr Größenlimit nicht; der Generationsschlüssel `g` wird erst vernichtet, nachdem Generation `g+1` veröffentlicht und `verify_absent` für die gelöschten Quellen attestiert ist |
| **INV-SEP-1** | Personenbezogener Klartext liegt nur in `Sealed` Clustern; Korpus-Chunks, Embeddings, Terme und Graph-Entitäten des Korpus entstehen aus pseudonymisiertem Text; Platzhalter sind je Person zufällig und nie aus dem Klartext abgeleitet |
| **INV-GRAPH-PROV-1** | Jede Kante und Hyperkante des Wissensgraphen trägt `(source_cluster, source_doc, version)`; je Löschdomäne existiert eine eigene Kopie, kein geteilter Besitz |
| **INV-DECAY-1** | Kognitiver Decay und `Supersedes` löschen nie und erzeugen weder Gate noch Receipt; Compliance-Löschung geschieht ausschließlich über `Eraser` |
| **INV-SEP-2** | Geteilte Index-Partitionen enthalten nur aus pseudonymisiertem Text abgeleitete Daten; Klartext-Embeddings liegen nur im Cluster-Mikroindex unter dem Clusterschlüssel |
| **INV-EPOCH-1** | Es gibt genau einen `Epoch(u64)`-Typ (in `contextra-types`); `Stamp ↔ Epoch` wird nur in `index-bridge` konvertiert und trägt `instance_id` |
| **INV-LAYER-1** | Abhängigkeiten zeigen nur auf strikt niedrigere Ebenen; Retrieval-Crates (`vector`, `text`, `graph`, `rank`) kennen nur `contextra-types` und `contextra-ports`, nie `storage`, `storage-ports` oder einander (K-58); Produktivpfade enthalten kein `std::fs`, `memmap2`, `bincode`, keinen festen Schlüssel und keinen unkeyed Inhalts-Hash |

**Modul-Invarianten.** Invarianten, die ein Retrieval-/Basis-Crate besitzt (`INV-HNSW-SLOT-1`, `INV-VEC-*`, `INV-SIMD-*`, `INV-TEXT-*`, `INV-DETERMINISM`, `INV-GRAPH-00x`, `INV-RRF-*`, `INV-CAL-*`, `INV-FSYNC-1`, `INV-TENANT-IDX-1`), stehen in der jeweiligen `crates/*/SPEC.md` und sind hier nur zitiert; ihr Test-Zitat prüft das Crate-CI. Bei Widerspruch gilt diese Tabelle. `INV-DB-3` (Kompensation der Alt-Engine) entfällt mit dem 2PC (§14.1).

---

---

## 16. Konfiguration (Auszug)

`block_target` 16 KiB · `memtable_total_bytes` ≤ 256 MiB · `redo_segment_bytes` 64 MiB, `redo_mark_bytes` 8 MiB, `redo_max_age` 1 h · `run_target_bytes` 64 MiB · `blob_max_bytes` 64 MiB · `max_pack_groups` 4096 · `pure_threshold` 8 MiB · `max_gate_clusters` 4096 · `max_open_files` 1024 (Größe des FD-Caches, kein Dateilimit, §4.3.1) · `maint_io_fraction` 0,5 · `purge_batch_min` 16 · `purge_rewrite_concurrency` 1 · `purge_scratch_reserve_bytes` ≥128 MiB (zwei Slots, keine normale Allokation) · **`purge_deadline` kein Standard (Pflichtparameter)** · Reservierungen 2^20 / 2^16 · `max_links` 8 · `max_hold_ttl` / `idem_ttl` 24 h · `anchor` Required bei Encrypted/Authenticated, `anchor_max_age` 1 s · `guard_window_entries` ≤ 1 Mio. und durch MemoryBudget begrenzt · `max_lag_bytes` 1 GiB · `erase_reserve_bytes` ≥2 GiB (inklusive Scratch 128 MiB, Consumer-Retention 1 GiB, Emergency-Reserve und FS-Metadaten-Headroom) · `max_clock_skew` 5 min default, strenger nur explizit · `key_grace` 10 min · `orphan_key_grace` 24 h · `revoke_grace` 5 s · `max_branches_per_cluster` 1024, `max_branch_depth` 4 (Default, hart 8) · `LiftPolicy` OwnerOrAdmin.

**Gemeinsames RAM-Budget:** `memory_budget_bytes` defaultet auf `min(1 GiB, max(256 MiB, 20 % des effektiven cgroup-/Job-Limits))`; kann der Adapter kein verlässliches Limit liefern, ist der Wert beim Öffnen Pflicht. Unterbudgets sind Obergrenzen am selben Budget: 25 % Memtables, 15 % Cluster-/Branch-Metadaten, 2 % Guard-Fenster, 20 % Block-Cache, 1 % Key-Cache, 20 % Extension-/Indexdelta und 10 % In-flight-Changes/Rewrite-Buffer; 7 % bleiben als Allocator-/Runtime-Headroom unangetastet. Jede Allokation reserviert atomar beim `MemoryBudget` und gibt per RAII frei. `max_clusters`, Branch-Limits und Extension-Admission werden zusätzlich durch tatsächliche Metadatenbytes begrenzt; `max_clusters=16 Mio.` ist eine Obergrenze, keine zugesagte Resident-Kapazität: `resident_clusters_max = cluster_meta_budget / 40 B` (§4.3.1). Für 16 Mio. Cluster sind ≈ 610 MiB Metadatenbudget nötig (≈ 4,0 GiB `memory_budget_bytes` bei 15 % Anteil); `register` weist darüber hinaus vor jeder Allokation mit `Limit{ClusterMetaBudget}` ab. Ein Extension-Owner ohne Budget-Token darf keine Indexarbeit oder Cache-Allokation starten.
**View-, Ledger- und Profil-Schlüssel:** `view_lease` 60 s, `view_lease_max` 10 min (§11.4) · `tomb_epoch_rotation` (Default aus), `backup_horizon` (Pflicht, sobald Retire aktiv ist; sonst kein Retire), `max_active_tomb_epochs` 4 (§10.4) · `OpenProfile::{Development, Production}` (Default `Production` bei persistentem Open; `Development` schaltet die Prüfungen `ProviderAssuranceInsufficient` und `LedgerSameDevice` auf Warnung), `acknowledged_weak_keys`, `ledger_same_device_ack` (beide erscheinen im Receipt als Restexposition).
**Pipeline-, Platzierungs- und Admission-Schlüssel:** `commit_pipeline_depth` 2 · `max_group_changes` 1024 · `max_group_bytes` 8 MiB · `guard_retry_max` 3 · `policy_retry_max` 3 · `publish_timeout` 1 s · `reserve_stamps_urgent` 2^10 · `reserve_ids_urgent` 2^8 · `max_link_depth` 8 · `dir_page_bytes` 4096 · `mini_section_max_bytes` 4096 · `spill_min_bytes` 8 MiB · `spill_ratio` 4 · `spill_max_age` 6 h · `max_staging_runs` 16 · `cluster_local_min_bytes` = `pure_threshold` · `max_cluster_local_clusters` 1024 (≤ 65 536) · `max_live_files` 262 144 · `pure_demote_ratio` 0,25 · `fd_reserve` 256 · `file_shard_count` 1024 · `purge_window_fraction` 0,3 · `sealed_purge_escalate` 0,7/0,9 · `cluster_meta_budget` 15 % von `memory_budget_bytes`.
**Metrikvertrag (Pflicht, Z-14/Z-17):** Der feste Katalog in `metrics.rs` definiert Name, Typ, Einheit, begrenzte Labels und Bedeutung. Pflichtfamilien: Commits (Anzahl, Gruppengröße, Latenz nach `durability`/`result`), Erase-Stufen (Gate- und KeyDestroy-Latenz, getrennte Zähler nach Status), Durability (sichtbar-durable Stamp-Lag, Fsync-Latenz nach festem `file_role`), Ressourcen (Bytes/Limit nach festem `resource_class`, Memtable/Cache-Belegung und Trefferquote, Reserve frei/benutzt, Scratch belegt), Wartung (Flush/Compaction/Purge-Bytes und Laufzeit nach `phase`), Erase (offene Tickets, ältestes Alter, Fristverletzungen, Dauer und Ergebnis je `stage`, Gate-Latenz, KeyDestroy-Latenz, Purge-Bytes, Anchor-Lag), Extensions (Feed-/Index-Watermark-Lag und blockierte Owner nach festem `owner_kind`), Guards (Stufe-A-Konflikte, `GuardRetry`-Rate, Fensterfuß-Treffer), Branches (Anzahl, Tiefe, ältester Pin/History-Floor) sowie Fehler/Poison-Zustand nach geschlossenem `ErrorClass`; zusätzlich abgelaufene View-Leases, offene Poisoned-Löschabsichten, Anzahl/Alter aktiver Tomb-Epochen (nur Zähler/Gauges, keine IDs). Jede Zeitreihe hat definierte Einheit und monotone Counter-Semantik; Histogramm-Buckets sind pro Release versioniert.

Labelwerte stammen ausschließlich aus kleinen geschlossenen Enums (`durability`, `result`, `file_role`, `resource_class`, `phase`, `stage`, `owner_kind`, `ErrorClass`). Keine IDs, Namen, Tenant-/Cluster-/Space-Kennung, Schlüssel, Request-Token, Querytexte, Dokumentinhalte oder Embeddings als Label oder Standard-Event. `Metrics`/`Observer` sind best-effort, nicht blockierend und werden außerhalb aller Kernlocks aufgerufen; Telemetrie-Ausfall ändert keine Storage-Semantik. Der Inspector ist administrative Diagnose, standardmäßig nur aggregierte/opaque Werte, benötigt Capability und darf keine Inhalte entsiegeln. Metrik-Katalog-Review und Kardinalitäts-/Leakage-Test sind Release-Gates.

---

**Retrieval-, Partitions- und Profil-Schlüssel (R11):** `micro_index_max_vectors` 50 000 · `max_partition_vectors` 500 000 · `max_partition_postings_bytes` 256 MiB · `max_partition_edges` 5 Mio. · `max_graph_segments_per_tenant` 64 · `rerank_depth` 4 × k · `ryw_wait` 50 ms · `staged_orphan_ttl` 24 h · `index_rebuild_bytes` (Unterbudget am `MemoryBudget`, Standard 10 % des Extension-Anteils) · `k_min` (Mindestgruppengröße für globale Kalibratoren, Standard 50 = `DEFAULT_WARMUP_REQUIRED`) · `export_io_fraction` 0,2 · `ComplianceProfile::{Development, Standard, Strict}` (Standard bei persistentem Open: `Standard`, §12.10) · `filter_selectivity` (ACORN-Schwelle) 5 %.

## 17. Leistung: Hypothesen und Messverfahren

Alles **Entwurfsziele**. Freigabe nur mit Messbericht (Hardware, Dateisystem, Last, Methode, Streuung); scheitert eine Hypothese, wird **sie** korrigiert, nicht das Verfahren gelockert. Es gibt drei Benchmarkgruppen (aus CairnDB): **Storage** (Commit-Latenz, Mehr-Space-Transaktionen, Recovery, Schreib-/Leseverstärkung), **Agent-Workload** (Request → letzte 100 Events lesen → Zustand → 20 Erinnerungen → Update → 10 Events anhängen → Checkpoint → Commit, gemessen in physischen Reads/Writes, CPU, Speicher), **Retrieval-Qualität** der oberen Schicht (Recall@Budget, Nutzen-pro-Token, Task-Erfolg bei gleichem Budget; Mnemos-Utility@Budget).

| ID | Hypothese | Messung |
|---|---|---|
| H-1 | `apply`: Durchsatz ≥ 80 % von Fsync-Rate × mittlere Gruppengröße; p99 ≤ 2 × Fsync-Latenz bei ≤ 70 % Auslastung der Fsync-Rate | Mikrobench, variierende Schreiberzahl |
| H-2 | Punktabfrage warm: ≤ 1 Datenblock-Read je Treffer-Level, p99 ≤ 50 µs | Zipf-Last |
| H-3 | Kopf-Scan ohne Körper ≥ 10× Durchsatz gegenüber Scan mit Körper (≥ 4 KiB) | Scan-Bench |
| H-4 | `page` liest ≤ 1,2 × Blöcke des Minimums | Gewichtsverteilungen; zusätzlich Aufbauzeit der RAM-Präfixsummen je SubIndex (≤ ca. 4096 Blöcke) und Traversierungskosten getrennt berichten (K-30) |
| H-5 | Schreibverstärkung Metadaten ≤ 15, Körper ≤ 2 | Compaction-Statistik |
| H-6 | ER0–ER2 je Cluster ≤ 10 ms (ohne Providerlatenz), unabhängig von der Memtable-Größe | Löschbench |
| H-7 | Messhypothese: Purge-Bytes ≤ Summe der betroffenen begrenzten Datei-/Extent-Größen plus dokumentierte Metadaten. Verhältnis bei 10× Store separat je Layout, Redo-Retention und Pin-Last messen; kein O(Clusterbytes)-Ziel | je `EraseLocality` und Dateirolle skalieren |
| H-8 | Pure-Cluster löschen ≤ 100 ms je 1000 Dateien (ohne Scrub) | Drop-Bench |
| H-9 | 1000 Cluster einer Gruppe ≤ 2 × Aufwand eines Clusters | Bündelungsbench |
| H-10 | Rohkopie beim Purge ≥ 80 % sequentielle Plattenbandbreite | Purge-Bench |
| H-11 | Seal ≤ 15 % CPU im Lastmix; Overhead kleiner Cluster ≤ 5 % Bytes ab 4 KiB und ≤ 10 % für 1–4 KiB (`MiniSection`: ein AEAD-Rahmen je Abschnitt) | A/B-Bench |
| H-12 | Speicher ≤ `memtable_bytes × (1+max_immutable)` + Cache + O(offene Gruppen); `ClusterIndex` ≤ 40 B je Cluster, Directory-Sparse-Index + `ClusterFilter` ≤ 1,3 B je Cluster und Datei | RSS |
| H-13 | Recovery ∝ Redo-Rückstand + Catalog; 1 GiB Redo ≤ 10 s; Catalog-Snapshot mit 16 Mio. Clustern sequentiell bis `Running` (Cluster-Index-Aufbau) ≤ 20 s | Recovery-Bench |
| H-15 | Zeilenabdeckung ≥ 90 %, Zweigabdeckung ≥ 85 % | `cargo-llvm-cov` |
| H-16 | Mutationsscore ≥ 80 % auf `codec`, `recovery`, `erase`, `sequencer`, `syncer`, `branches` | `cargo-mutants` |
| H-17 | Gate + ER2 für 10 000 Cluster ≤ 1 s bei Providerlatenz ≤ 5 ms je `destroy_many` | Batchbench |
| H-18 | Guard-Stufe B ohne Retry ≥ 99 % bei Zipf-Last mit 1 % Konfliktrate; **0 E/A-Syscalls** im Sequencer (Fs-Zähler) | Guard-Bench |
| H-19 | L0-Dateizahl unabhängig von der Zahl aktiver Packgruppen | Flush-Bench mit 4096 Gruppen |
| H-23 | Restore-Matching: `Cluster der Kopie × aktive Tomb-Epochen` HMACs für 16 Mio. Cluster und 1/4 Epochen innerhalb des Restore-Budgets; Retire eines Epochenschlüssels ohne Wartungsstau | Restore-Bench, Ledger mit mehreren Epochen |
| **H-20** | Fork < 50 µs, Speicher O(1) | Fork-Bench |
| **H-21** | Punktlesen auf Branch bei 1 % geänderten Schlüsseln: Datenblock-Reads ≤ 1,1 × `main`; p99-Latenz ≤ 1,2 × `main` bis Tiefe 4 (Default) und ≤ 1,5 × bis Tiefe 8; Tiefen 1…8 einzeln, dazu Worst-Case (alle Kettenebenen Filter-Treffer) und Bereichs-Scan | Branch-Bench mit branch-bewusstem Filter (§7.1); als Spike vor G-F |
| **H-22** | Merge ∝ Source-Mutationen + Target-Mutationen seit `merge_base` + Konflikte, nicht Gesamt-Clustergröße; je 10⁴ Mutationen ≤ 200 ms ohne Körper-I/O/Konfliktauflösung | ChangeIndex/Merge-Bench |
| **H-24** | Sequencer-Abschnitt (Rang 1) p99 ≤ 20 µs für Gruppen ≤ 64 Changes ohne Gate; Gate mit `policy_epoch`-Treffer ≤ 10 µs; Selbstberechnungs-Fallback bei \|H\| = 4096 ≤ 250 µs; **0 E/A-Syscalls, 0 Provider-Aufrufe**; Commit-p99 bei parallelem 4096-Cluster-Gate ≤ 1,2 × Commit-p99 ohne Gate | Commit-Bench mit parallelem Erase-Strom, `FaultFs`-Zähler, Sim |
| **H-25** | 16 Mio. Cluster: `ClusterIndex` ≤ 700 MiB; Directory-Sparse-Index ≤ 1 B je Cluster und Datei; Gate-Check ohne E/A; kalter Punktlesezugriff ≤ 1 zusätzlicher Directory-Seitenzugriff je Level-Datei, bei Warm-Cache 0 | RSS-/Cache-Bench mit 1/4/16 Mio. Clustern, Zipf |
| **H-26** | Staging→L1-Schreibverstärkung je Packgruppe ≤ 1 + `spill_ratio` (≤ 5) bei 4096 Gruppen und ≈ 16 KiB Update je Gruppe und Flush; Gesamt-Metadaten-Schreibverstärkung ≤ 15 (H-5) für Millionen kleiner Cluster (90 % wenige KiB, Zipf-Updates) | Compaction-Statistik, Packgruppen-Bench |
| **H-27** | `ClusterLocal` bei 1 Mio. Clustern (1 % beantragt): Admission lehnt über der Quote ab; Live-Dateien ≤ `max_live_files`; FD-Cache-Trefferquote ≥ 99 % bei Zipf; `open()`-Syscalls ≤ 0,01 je warmem Punktlesezugriff; Dateierzeugung ohne Verzeichnis-Degradation (Sharding gegen flach) | FS-Bench (ext4/xfs), Inode-/FD-Zähler |
| **H-28** | je Fenster `W = 0,3 × purge_deadline` wird jede Datei höchstens einmal umgeschrieben; Purge-Bytes je `W` ≤ Σ Größen der betroffenen Dateien; bei 0,1 % Löschrate je Tag auf 16 Mio. Clustern Purge-Schreibverstärkung ≤ 1 je `W`; Huckepack-Anteil wird berichtet; Sealed-Hygiene verdrängt `maint_io_fraction` nicht | Purge-Bench mit verstreuten Löschungen, Langzeitlauf |
| **H-29** | Delta-Pfad: `Basis ∪ Delta` kostet ≤ 1,15 × die Latenz des Basisindexes bei Delta ≤ 1 % der Partition; Text und Graph bitgleich zu Vollscan, Vektor-Recall ≥ Basis | Differentialbench (`Index ∪ Delta ≡ Vollscan`) |
| **H-30** | Hybridanfrage (drei Kanäle, k = 20, 10 Mio. Chunks in ≤ 20 Partitionen, warm, ohne Rerank-Modell): p99 ≤ 150 ms; Storage-Anteil (Gate, `filter_visible`, Körperzugriff) ≤ 15 % | Retrieval-Bench, Zipf-Last |
| **H-31** | Fan-out: Latenz wächst von 1 auf 16 Partitionen um ≤ 1,3 × (parallele Kanäle/Partitionen) | Partitionsbench |
| **H-32** | Partitions-Neuaufbau: Rebuild-Bytes je Fenster `w` ≤ Σ betroffener Partitionen, nie proportional zur Zahl der Löschaufträge; Rebuild-Anteil an der Wartungsbandbreite ≤ 25 % bei 0,1 % Löschrate/Tag (Zielwert nach Spike R2 bestätigen oder korrigieren); Commit-p99 ≤ 1,2 × ohne Rebuild | Rebuild-Bench mit verstreuten Löschungen |
| **H-33** | Graph: Adjazenz über ≤ 64 Segmente ≤ 2 × Einzel-CSR; PPR-Latenz auf gepinntem Segment-Set reproduzierbar | Graphbench |
| **H-34** | `pii_residual_rate` je Release mit Stichprobe n ≥ 1000 und Konfidenzintervall gemessen und berichtet; kein Zielwert im Kern (Mandantenrichtlinie) | Prüf-Orakel mit Canary-Personen |
| **H-36** | Löschung einer Person mit 10⁴ Chunks in `Personal`-Partitionen: Rebuild-Bytes je Fenster ≤ Σ betroffener Personal-Partitionen (je ≤ 100 000 Vektoren); `Personal`-Rebuild-Anteil ≤ 25 % der Wartungsbandbreite; Receipt-Status `Incomplete` nur bis Generationswechsel und Schlüsselvernichtung der Altgeneration | Rebuild-Bench mit Canary-Personen (Spike vor G-R2) |
| **H-35** | Read-your-writes: p99 Zeit vom Receipt bis zur Sichtbarkeit im eigenen Query ≤ `ryw_wait` + Delta-Latenz (≈ 60 ms) | RYW-Bench |

---

---

## 18. Entschiedene Fragen, Risiken, externe Vorbedingungen

| Frage (Quelle) | Entscheidung |
|---|---|
| Wer setzt Restrictions; Ablauf automatisch? (R7 O-1) | Jeder Handle-Besitzer darf `restrict`; `lift` nach `LiftPolicy`; Ablauf lässt nur den Vermerk fallen |
| Zugriffsstatistik für Layout (Mnemos O-1) | Nicht im Kern (K-17); obere Schicht führt eigene Sidecar-Strukturen in eigenem Cluster/Space |
| Handle-Stabilität über Branches (Mnemos O-2) | Handles sind Aliase **im Cluster** (§12.6); Branches teilen den Alias-Space, Merge löst Alias-Konflikte als normale Schlüsselkonflikte |
| Granularität der Verschlüsselung vs. Extent-Layout (Mnemos O-3) | Gelöst durch Clusterreinheit |
| Embedding-Lanes bei Modellwechsel (Mnemos O-4) | Dual-Betrieb als Space/Segment je Modell, Speicherbudget obere Schicht |
| Merge-Funktionen in WASM (Mnemos O-5) | Nein im Kern (NZ-4); obere Schicht liefert Auflösungen |
| Eigener FTS vs. Tantivy (Mnemos O-6) | Obere Schicht; Tantivy nur als `SegmentOwner` mit `rewrite_without`/Rebuild-Intervall |
| Embedded vs. Server (Mnemos O-7) | Embedded-Kern; Server/MCP-Wrapper oben ab G-P |
| Keyring als Single Point of Loss (Lethe-Spec B.4) | Pflicht-Replikation im Provider, `KeyLost` unterscheidbar |
| KMS-Karenzzeit (Lethe-Spec B.6) | Ticket bleibt `Blocked`, bis der Provider `Absent` attestiert; strenge SLAs verlangen sofort zerstörenden Provider |
| Single-Sequencer-Durchsatz (Lethe-Spec B.5) | Ein Committer je Instanz; Tenant-Sharding über mehrere Instanzen oben |

**Risiken:** (1) Branch-Pinning treibt Speicher und Compaction-Kosten bei vielen Langzeit-Branches (Mitigation: `ttl`, Limits, `floor`-Metrik). (2) Branch-Reads über Elternketten (H-21) können bei tiefen Ketten teuer werden (Mitigation: branch-bewusster Filter und `BranchTable` §7.1, Default-Tiefe 4, Spike vor G-F). (3) Kosten der Plain-Barriere bei häufigen Plain-Löschungen (Mitigation: Bündelung; Personal-Daten sind per Konstruktion Sealed). (4) Verzeichnis Person → Wurzelcluster bleibt nur physisch löschbar (§12.7). (5) Formatfreeze nach P2 verlangt, dass P1/P2 echte Probeläufe sind, nicht Papier. (6) Scope-Creep in obere Schichten; Mitigation: Gates strikt. (7) Komplexität der Commit-Pipeline (Sequencer/Syncer/Publikation, `Retiring`); Mitigation: M1-Modell, loom, Simulation RACE-18…RACE-20. (8) RAM-Bedarf ab ≈ 4 Mio. Clustern (≈ 40 B je Cluster plus Hot-Records); Mitigation: Admission §4.3.1, Konvention „Session ≠ Cluster“. (10) Rebuild-Kosten geteilter Indizes bei hoher Löschrate (Mitigation: Partitionsgrößenlimit, Fenster-Amortisierung, ehrlicher `Blocked`-Status, H-32). (11) Fehlklassifikation personenbezogener Inhalte in der Ingest-Pipeline (Mitigation: Quarantäne bei Unsicherheit, `pii_residual_rate` als Messgröße, Receipt-Restrisiko, H-34). (12) Aufwand der Retrieval-Bereinigung (R1) bei gleichzeitig laufender Weiterentwicklung der Crates (Mitigation: Verbotslints, Klasse-A-Build als Gate). (9) Der Zeitplan in §14 ist für P2, P6 und P7 (Catalog/Purge/TLA+ bzw. Branches) wahrscheinlich um den Faktor 1,5–2 zu knapp; Mitigation: Gate-Kriterien bleiben, Termine werden nach P1/P2 neu geschätzt.

**Externe Vorbedingungen (keine Spec-Lücken):** rechtliche Prüfung der Fristen und der Aussage „kryptografisch gelöscht“ für den Einsatzfall; zweiter Reviewer für G-0; Beschaffung eines `Strong`-Ankers und eines HSM/KMS-Providers für den Produktivbetrieb (der Kern ist ohne sie vollständig, Restrisiken sind benannt).

---

### 18.1 Topologie-Synchronisation: Ist-Befunde und Pflichtmaßnahmen (K-47)

Befunde am Repository-Stand (HEAD `60ea034`, 2026-11-24), per Dateiprüfung belegt; die Maßnahmen sind Teil von P0:

| # | Befund | Maßnahme |
|---|---|---|
| 1 | `contextra-storage` importiert `contextra_storage_ports` (z. B. `catalog/reserve.rs`), das Crate existiert nicht, und `crates/contextra-storage/Cargo.toml` enthält keine Abhängigkeit darauf; der Kern kompiliert nicht | `crates/contextra-storage-ports` anlegen (Level 0), Workspace-Mitglied, Abhängigkeit eintragen; `cargo check -p contextra-storage --locked` als G-0-Kriterium |
| 2 | `Cargo.toml` führt 14 Mitglieder (`kvcache` liegt unter `crates/`, ist aber nicht Mitglied); `capabilities.toml` beschreibt 42 Crates, 28 davon existieren nicht (u. a. `store`, `ids`, `env`, `wire`, `router`, `agent`) | Zielzustand je Eintrag festlegen: anlegen, als `planned` markieren oder streichen; `bash tools/ctx.sh gates` muss ohne Abweichung laufen |
| 3 | `contextra-erasure` und `contextra-vault` (laut `CLAUDE.md`) fehlen in Baum und `capabilities.toml` | `CLAUDE.md` und `capabilities.toml` angleichen; Signaturstufe optional (K-55), Änderungen am eingefrorenen Crate nur nach Freigabe |
| 4 | Es gibt kein Root-`tests/`; `CLAUDE.md` nennt dennoch `tests/layering.rs` und `tests/unsafe_islands.rs`; nur `crates/contextra-storage/tests/layering.rs` existiert (verbietet `contextra_ports` im Kern, richtig); der Cargo-Alias `xtask` zeigt auf ein fehlendes Paket | als Cargo-Tests in einem Mitglied-Crate (`contextra-testkit` oder `storage/tests`) einbinden; `xtask` entfernen oder anlegen |
| 5 | `vector → mvcc` (optional, gleiche Ebene), `types`/`ports` auf Level 2 statt unterhalb ihrer Konsumenten; `engine`-`allowed_internal` führt das nicht vorhandene `store` | K-58: Ebenen neu zuordnen, `vector → mvcc` entfernen (K-40), `capabilities.toml` angleichen |
| 6 | `blake3`, `aes-gcm-siv`, `bincode`, `memmap2`, `std::fs` in Retrieval-/Crypto-/Checkpoint-Crates | Bereinigungsplan §14.1; Verbote als `deny.toml`-/Lint-Regeln, damit der Zustand nicht zurückkehrt |
| 8 | `contextra-kvcache` hat Feature `content-addressed-kv-cache` und hängt an `crypto/kv-encryption` | K-61 |
| 9 | `Cargo.toml [workspace.dependencies]` enthält `bincode`, `flatbuffers`, `memmap2`, `tokio full`, `parking_lot`, `rand` als Workspace-weite Defaults; jedes Crate kann sie ziehen | Verbote je Ebene in `deny.toml` (`wrappers`), Level-2-Crates ohne `tokio`/`bincode`/`blake3` |
| 10 | `docs/agent/` (von `CLAUDE.md` referenziert) fehlt | `docs/agent/CRATES.md` aus `capabilities.toml` generieren (P0) |
| 7 | Ein vollständiger Workspace-Check ist auf der Entwicklungs-VM (1 vCPU, 3,9 GB RAM) nicht abgeschlossen worden | Gates laufen in CI je Crate; das Gesamtergebnis ist erst nach P0 belastbar |

### 18.2 Offene Entscheidungen mit Freigabebedarf

| Entscheidung | Wer entscheidet | Standard bis dahin |
|---|---|---|
| Änderungen an `contextra-erasure` (Signaturformat, Abhängigkeiten) | Projektinhaber | unverändert (`frozen`); Bridge liefert Receipt, Signatur optional |
| Windows-Produktionsfreigabe (Fs-Konformanz) | Projektinhaber nach Konformanzlauf | nicht freigegeben (NZ-8) |
| `Strong`-Anker und HSM/KMS-Provider | Betrieb/Kunde | `Weak`-Anker nur mit Bestätigung; `Standard`-Profil verlangt KMS |
| Edition/`LicenseGate`-Grenzen | Produktentscheidung | Open-Core-Grenze §2.6, keine Kerninvariante hinter Bezahlschranke |
| Rechtliche Einstufung „anonym nach Tresorvernichtung“ | Rechtsberatung des Kunden | als Restrisiko im Receipt (K-50) |

---

## 19. Entscheidungsregister (neu nummeriert)

| ID | Entscheidung |
|---|---|
| SD-01 | Catalog als einzige Wahrheit, ein Edit = ein Durabilitätspunkt |
| SD-02 | Clusterreine Blöcke, ein Abschnitt je Cluster und Datei, abschnittsrelative Offsets |
| SD-03 | Bewilligte Abhängigkeiten. Kern: `crossbeam-skiplist` (hinter Trait), `bytes`, `zeroize`, `siphasher` (nur Guard-Fingerabdrücke). `contextra-crypto-ref`: `chacha20poly1305`, `hmac`, `sha2`, `hkdf`, `crc32c`. Dev: `loom`, `shuttle`, `proptest`, `cargo-mutants`. Krypto im Kern nur über Ports (`blake3` und `chacha20poly1305` sind keine Kernabhängigkeiten; sie widersprächen §5.2 und dem Portprinzip) |
| SD-04 | Eine Formatversion je Dateiart, Migration als Werkzeug |
| SD-05 | Sealed-Cluster umfasst Schlüssel, Kopf, Körper, Index, Filter, Marker, Branch-Metadaten |
| SD-06 | `purge_deadline` Pflichtparameter |
| SD-07 | Pure/Packed/Staging mit Packgruppe |
| SD-08 | Rohkopie beim Purge |
| SD-09 | Restriction ohne Override, `LiftPolicy` |
| SD-10 | Lanes inkl. `urgent` |
| SD-11 | Fünf Erweiterungsverträge |
| SD-12 | Kein 2PC; ein Redo-Frame je Change |
| SD-13 | Hardlink-Checkpoints nur Opt-in |
| SD-14 | XChaCha20-Poly1305, Zufallsnonce |
| SD-15 | TLA+-Modelle M1–M6 als Gate |
| SD-16 | Fsync-Fehler nie wiederholen |
| SD-17 | Cluster-Zustand `state|gen` als Atomic |
| SD-18 | Opaker Kopf, `weights`, `weight_sum` |
| SD-19 | Stamp-Reservierung im Catalog |
| SD-20 | Redo in ausgerichteten Gruppen, vorallokiert, Reservesegment |
| SD-21 | Blob-Inkarnation ohne Referenz-Umschreibung |
| SD-22 | `LOCK` über `Fs::lock` |
| SD-23 | Ledger getrennt von Catalog und Checkpoints, `ledger_dir` konfigurierbar |
| SD-24 | Einclustrige Einträge |
| SD-25 | `cmp` statt `memcmp` |
| SD-26 | Volatile-Spaces ohne Redo |
| SD-27 | Rollback-Anker als Port, Pflicht bei Encrypted/Authenticated |
| SD-28 | Gate ist ein Frame der Commit-Pipeline (im Sequencer sequenziert) |
| SD-29 | Blob-Entfernung nur nach Abgleich lebender Runs |
| SD-30 | Memtable lock-frei |
| SD-31 | Change-Limits |
| SD-32 | Bitemporalität als Konvention oben |
| SD-33 | Namensraum `contextra-storage(-ports)` |
| SD-34 | Eine normative Spec plus `AGENTS.md`; Vorfassungen in `docs/archive/` |
| SD-35 | Redo-AAD mit `op_uid` statt Stamp |
| SD-36 | `AdminCap`/`RestrictedReadCap` als nicht klonbare Fähigkeiten |
| SD-37 | Kern erzeugt `ClusterId`; Sealed/Plain; `DataClass::Personal` erzwingt Cluster-Seal |
| SD-38 | Eine Siegelform; RootKey-Indirektion; `Cryptographic` nur bei `KeysInBackups::No` oder `Bounded` mit Statuscap (E-77) |
| SD-39 | Gate ohne Segmentschnitt; Barriere beim Purge, nur Plain, gebündelt |
| SD-40 | Consumer-Feed aus Redo mit zwei Böden |
| SD-41 | Guards zweistufig: Stufe A Aufrufer exakt gegen `V1`, Stufe B Sequencer nur Delta (SD-62) |
| SD-42 | Staging-L0 und ein einheitlicher Purge-Algorithmus |
| SD-43 | Catalog-Gruppen-Commit; Anker-Koaleszenz bis vor Receipt |
| SD-44 | Append-Dateien nur in Vorallokation; ENOSPC vergiftet nicht |
| SD-45 | Referenzimplementierungen: `keys-file`, `keys-cohort`, `FileAnchor` (`Weak`) |
| SD-46 | Kein zweiter lebender Klon (NZ-19) |
| **SD-47** | **Ledger speichert `tomb_id`, nie ClusterId (K-03)** |
| **SD-48** | **Clusterlokale Branches mit Merge; kein globaler DAG, keine Inhaltsadressierung (K-07, K-08)** |
| **SD-49** | **Branch-Metadaten im Cluster (`SYS_BRANCH`), Format-Slot `branch` ab G-F, Semantik ab P7** |
| **SD-50** | **`View::filter_visible` und Index-Watermarks als Pflichtmuster (K-16)** |
| **SD-51** | **Phase P1 ist ein vertikaler Durchstich mit provisorischen Formaten; Formatfreeze G-F nach P2** |
| **SD-52** | **Person → Wurzelcluster + HMAC-Verzeichnis als normatives Muster (K-24)** |
| **SD-53** | **Receipt mit kanonischen Bytes und offline `verify_receipt`; Signatur oben** |
| **SD-54** | **Ränge 1–8 durchgehend, Registerneunummerierung (K-28)** |
| **SD-55** | **Keine Zeitlanes/Kumulativsummen im Kernformat; `BlockPruner` über `ExtensionBlock` für schattensichere Besitzer (K-29, K-30)** |
| **SD-56** | **`tomb_epoch` als Format-Slot; Tomb-Schlüssel im Provider; Retire nur nach Backup-Horizont (K-31)** |
| **SD-57** | **Löschabsicht auf `Poisoned` über Ledger; `ErasePending`; Gate beim Neustart (E-75)** |
| **SD-58** | **View-Lease mit `SnapshotTooOld` (E-76); `KeyAssurance`/`KeysInBackups`-Statuscaps (E-77); `MeasureKind` (E-80)** |
| **SD-59** | **Derivation Cache als clusterlokale Konvention (K-32)** |
| **SD-60** | **Commit-Pipeline: Sequencer (Rang 1, ohne E/A) + Syncer; Notpool statt Rang-1/3-Doppelhaltung (K-33)** |
| **SD-61** | **`policy_epoch`; Link, Restrict, Lift und Gate als Frames im Sequencer total geordnet; Selbstberechnung der Hülle nach drei Retries (K-33)** |
| **SD-62** | **Guards: Stufe A exakt auf dem Aufruferthread, Stufe B Delta im RAM-Konfliktfenster, nur `GuardRetry` (K-33)** |
| **SD-63** | **Branch-bewusster Filter `(branch, user_key)`, `BranchTableBlock`, Chain-Resolve, Default-Tiefe 4, Spike vor G-F (K-34)** |
| **SD-64** | **`ClusterIndex` 40 B, Sparse-Directory, `ClusterFilter`, `HotRecord`, `MiniSection`, Admission vor Allokation (K-35)** |
| **SD-65** | **Staging-Spill-Policy, Fenster-Amortisierung des Purge, Sealed-Hygiene-Lane (K-36)** |
| **SD-66** | **`ClusterLocal`-Admission, Pure-Demotion, FD-Cache, Verzeichnis-Sharding (K-37)** |
| **SD-67** | **Idempotenz als implizite `Absent`-Guard (K-38)** |
| **SD-68** | **Retrieval-Crates persistenzneutral; Bindung in `contextra-index-bridge` (K-48)** |
| **SD-69** | **Index-Partition = Packgruppe; Generationsschlüssel je Partitionsgeneration (K-49)** |
| **SD-70** | **`contextra-durable-fs` ist nicht die Basis von `RealFs`**: der `Fs`-Port verlangt `read_at`, `read_ranges`, `preallocate`, `lock`, `punch_hole`, `free_space`, die das Journal-orientierte Crate nicht abdeckt; seine Poisoning-Semantik (`INV-FSYNC-1`) wird als Anforderung übernommen, nicht der Code |
| **SD-71** | **Trennungsprinzip und Pseudonym-Tresor (K-50)** |
| **SD-72** | **Publish-Record statt Mega-Change und statt 2PC (K-51)** |
| **SD-73** | **Read-your-writes über `min_stamp` + Delta-Pfad (K-52)** |
| **SD-74** | **`SnapshotSet` je Antwort, Determinismus als Invariante (K-53)** |
| **SD-75** | **Lernsignale clusterlokal; globale Kalibratoren nur aus Aggregaten (K-54)** |
| **SD-76** | **Eingefrorene Compliance-Crates unangetastet; Receipt→Proof im Bridge-Layer (K-55)** |
| **SD-77** | **Compliance-Profile `Development`/`Standard`/`Strict` (K-56)** |
| **SD-78** | **`export_cluster` für Auskunft/Portabilität (K-57)** |
| **SD-80** | Fünf Ebenen L0–L4; `types` L0, `ports`/`storage-ports` L1 (K-58) |
| **SD-81** | Ein `Epoch(u64)`, `TxId`-Kopien entfallen (K-59) |
| **SD-82** | Indexklassen `Corpus`/`Personal`; geteilte Indizes nur aus pseudonymisiertem Text (K-60, `INV-SEP-2`) |
| **SD-83** | `kvcache`: keine Inhaltsadressierung, Owner `MixedPerClusterKey` (K-61) |
| **SD-84** | `memmap2` nur in `contextra-sys` hinter Feature; `block_on` nur im Engine-Rand (K-62) |
| **SD-85** | Alt-Verträge in `contextra-ports` bleiben bis G-CUTOVER, Bridge bedient sie (K-63) |
| **SD-79** | **Ebenenregel strikt: Abhängigkeit nur auf niedrigere Ebene; `storage-ports` ist Level 0** |

---

---

## Anhang A — Referenzvektoren (Pflicht vor G-F)

Je Format mindestens (zusätzlich Partitionsblock und `head`-Record, §12.4): leeres Objekt, ein Eintrag, Eintrag mit versiegeltem Körper (Cluster- und Space-Quelle), Cluster-Abschnitt mit zwei Blöcken, Packed-Run mit drei Clustern, Staging-Run, `MiniSection`, Abschnitt mit `BranchTable` (drei Branches, einer ohne Einträge), Tombstone, Bereichsmarker, Gate-Frame mit `policy_epoch`, Control-Frame `Link`, Blob mit Loch, Catalog-Edit-Kette mit Reservierung, Gruppen-Commit und `anchor_mark`, Ledger-Kette (`tomb_id`, zwei Tomb-Epochen, ein Retire), Redo-Gruppe mit Change-/Gate-/Control-/Branch-Frame, abgeschnittener und beschädigter Frame, nicht-kanonisches `varint`, **Rohkopie-Test** (Abschnitt aus Run A in Run B, Blöcke byte-identisch, Entsiegeln gelingt). Erzeugung durch `tools/reference/generator.py` (unabhängig von Rust), Vergleich im Test `golden`. **Stand: nicht vorhanden.**

## Anhang B — Änderungen gegenüber R7 (R8-01…R8-15), R8 (R8.1-01…R8.1-09) und R8.1 (R8.2-01…R8.2-09)

| ID | R7-Befund | R8-Lösung |
|---|---|---|
| R8-01 | Mission verlangt Versionskontrolle; R7 schloss Branches aus dem Kern aus (NZ-13) | clusterlokale Branches, Diff, Merge (§8), Format-Slot ab G-F |
| R8-02 | Ledger enthielt ClusterId bzw. Cluster-ID-Listen (überdauert Löschung und Backups) | `tomb_id` (§9.7), Restore per `tomb_id`-Abgleich (§10.3) |
| R8-03 | Keine Quelle für Extension-Sichtbarkeit trotz Pflicht zur Leckfreiheit | `filter_visible` und Watermark-Muster (§7, §12.4) |
| R8-04 | `SegmentOwner` ohne `on_gate`/`verify_absent` (Lethe-Entwurf hatte beides) | ergänzt (§12.1) |
| R8-05 | Lift nur Owner/AdminCap | `LiftPolicy::FourEyes` über `Authority` (Lethe-Spec) |
| R8-06 | Receipt ohne kanonische Form / Offline-Prüfung | `ErasureReceipt` + `verify_receipt` (§9.7) |
| R8-07 | Cohort-Rewrap fehlte als Skalierungsweg | Provider-intern (`keys-cohort`), Kern unverändert (§9.4) |
| R8-08 | Phasenplan begann mit Formatfreeze vor jedem Lauf | vertikaler Durchstich P1, Freeze nach P2 (§14) |
| R8-09 | Doppelte `SD-38/39/40`, Rang „5 entfällt“ | Neunummerierung (§19, §11.2) |
| R8-10 | `ResumeToken` ohne Sicherheitsvertrag | AEAD-versiegelt unter Prozessschlüssel (§7) |
| R8-11 | Person→Cluster-Zuordnung nur als Warnung | normatives Muster §12.7 |
| R8-12 | Keine Benchmark-Gruppen für Agenten/Retrieval | §17 (aus CairnDB/Mnemos), H-20…H-22 für Branches |
| R8-13 | Restriction-Zwischenzustand unklar (Lethe: Blocked mit Restricted-Hide) | sofortiger `Err(Restricted)`, Verbergen via `BLOCK_READ` durch die obere Schicht (§9.6) |
| R8-14 | Lokalitätstest (Lethe-Entwurf INV-22) fehlte in R7-Invarianten | `INV-ERA-11` mit Byte-Zählern |
| R8-15 | Change-Feed zeigte nur `Erased`; Lethe-Entwurf unterscheidet Gate von Purge | `Redacted` (ab Gate) und `Erased` (§12.3) |

**Änderungen R8 → R8.1:**

| ID | R8-Befund | R8.1-Lösung |
|---|---|---|
| R8.1-01 | Kurzfassung transportierte Europa-/Fusions-USP nicht; Risiko, ANN/BM25/Graph dem KV-Kern zuzuschreiben | §0 neu gefasst; Indizes bleiben obere Komponenten (NZ-12) |
| R8.1-02 | Open-Core-Grenze nicht festgeschrieben | L20 und §2.6: Basis-Durability, Löschung und Verifikation nie hinter Bezahlschranke |
| R8.1-03 | `personal: bool` erlaubte unklare Kombinationen | `DataClass` (geschlossenes Enum), Validierung vor I/O (Z-16, §4.5, §4.7) |
| R8.1-04 | Metrikliste ohne Katalog- und Kardinalitätsvertrag | geschlossener Metrikkatalog, Label-Allowlist, Leakage-Gate (Z-17, §16) |
| R8.1-05 | Fehler ohne Diagnosevertrag | stabile Fehlertypen, `hint()`, Seed-reproduzierbare Simulation (Z-18) |
| R8.1-06 | F-3 (`EIO`/Fsync) ließ Commit-Status offen | `Indeterminate{first_stamp,last_stamp}`; Idempotenz-Token statt blindem Retry |
| R8.1-07 | AAD-Definition mit doppeltem `space_id`; RootKey-Redo-Aussage kollidierte mit `INV-ERA-7` | AAD kanonisiert (`instance_id, space_id, cluster_id, key_epoch, …`); Klarstellung §5.2 |
| R8.1-08 | RAG-Integration (Ingest → Index → Query) nur verteilt beschrieben | §12.4.2 Ende-zu-Ende-Vertrag mit Watermark, `IndexLag`, Gate-Filter |
| R8.1-09 | Phasenplan blockierte parallele Entwicklung von Indizes/Cache | Entwicklung parallel; Integration P4–P8 unter gemeinsamen Gates (§14) |

**Änderungen R8.1 → R8.2 (Octa-Abgleich und Feinschliff):**

| ID | R8.1-Befund | R8.2-Lösung |
|---|---|---|
| R8.2-01 | `tomb_id` blieb bis ans Ledger-Ende linkbar (offenes Restrisiko aus K-03) | `tomb_epoch`, Provider-Tomb-Schlüssel, Backup-Horizont und Retire (K-31, §9.4, §9.7, §10.4) |
| R8.2-02 | `KeyProvider` kannte nur `keys_in_backups: bool`; keine Aussage zur Vernichtungsgüte; §9.4 und Ports-Code wichen ab | `KeysInBackups`, `KeyAssurance`, Statuscaps, Batch-Trait (E-77, §9.4) |
| R8.2-03 | Löschung auf vergiftetem Handle nicht spezifiziert | Ledger-Intent, `ErasePending`, Gate beim Neustart (E-75, §9.5.2) |
| R8.2-04 | Views konnten Dateien und History-Floor unbegrenzt pinnen | View-Lease, `SnapshotTooOld` (E-76, §11.4) |
| R8.2-05 | Receipt nannte nicht, wie jede Messung gewonnen wurde | `MeasureKind`, Pflichtmessungen (E-80, §13.2.1) |
| R8.2-06 | Host-Zustand (Swap, Core-Dumps) und Gerätetrennung des Ledgers nicht prüfbar | `HostProbe`, `LedgerSameDevice`, `LedgerUnavailable` (E-78, E-81) |
| R8.2-07 | Zeitlanes/Kumulativsummen im SubIndex vorgeschlagen (A1/A2) | verworfen im Kernformat; `BlockPruner`, RAM-Präfixsummen (K-29, K-30) |
| R8.2-08 | Embedding-/Token-Caches kollidierten mit `INV-PRIV-1` | clusterlokaler Keyed-MAC als Konvention (K-32, §12.6) |
| R8.2-09 | §4.1 zerriss die Vokabeltabelle mit einem Absatz; `create_cluster_key` im Text neben Batch-Trait; Konfliktzahl veraltet | bereinigt |

## Anhang B.2 — Änderungen R8.2 → R9.0 (R9-Audit)

| ID | R8.2-Befund | R9-Lösung |
|---|---|---|
| R9-01 | Rang 1 hielt die Sperre über den gesamten Redo-Sync; Tail-Latenz aller Agenten gekoppelt an Sync und Gate-Arbeit | Sequencer (Rang 1, nur RAM) + Syncer (ein Schreiber, kein Lock), Credits, Leader-Protokoll (§6.1, K-33, INV-PIPE-1/2) |
| R9-02 | Graph-Traversierung der `DerivedFrom`-Hülle (≤ 4096) im kritischen Pfad | Hülle auf dem Aufruferthread, Sequencer prüft nur `policy_epoch`; Selbstberechnung nach 3 Retries gegen Aushungern (§6.1, §9.5, INV-POL-1) |
| R9-03 | Guard-Langsampfad „exakt gegen Memtable ∪ Version“ unter Rang 1 widersprach K-21 und H-18 (Plattenzugriffe) | Stufe A exakt beim Aufrufer, Stufe B nur Delta im Konfliktfenster, nur `GuardRetry` (§6.3) |
| R9-04 | `SYS_IDEM`-Prüfung im Committer, Race zweier gleicher Tokens, RAM ∝ aktive Tokens | implizite `Absent`-Guard auf den SYS_IDEM-Schlüssel (§6.5, K-38) |
| R9-05 | Lock-Reihenfolge: Intake (Rang 2) vor Committer (Rang 1) verletzte `INV-LOCK-1`; Rang 1+3-Doppelhaltung legte den Catalog-Sync unter Rang 1 | Leader-Protokoll (Anhängen unter Rang 2, dann `try_lock` Rang 1, Leader leert Intake aufsteigend); Notpool statt Doppelhaltung (§6.1, §5.8, §11.2) |
| R9-06 | `Link` war ein Catalog-Edit außerhalb der Stamp-Ordnung: „Quelle gelöscht, Link danach“ verletzte `INV-ERA-12` | Control-Frame `Link` im Sequencer, Quelle `Retiring`/`Retired` ⇒ `ClusterRetired` (§9.6, RACE-22) |
| R9-07 | Link-Admission prüfte nur die Nachfahren des neuen Kindes; die Hülle jedes Vorfahrens wächst mit | Prüfung für jeden Vorfahren-oder-selbst, Abbruch bei `max_gate_clusters + 1` Knoten (§9.6) |
| R9-08 | Space-Drop fror die Mitgliedschaft unter Rang 1 ein (O(Cluster)); Restriction-Prüfung über unbekannte Mitgliedschaft unspezifiziert | Gate maskiert die SpaceId; `SpaceScopeFrozen` nachgelagert aus gepinnter Version; Restriction-Menge × Cluster-Dateien beim Aufrufer, `policy_epoch`-Revalidierung (§9.7, §9.8, RACE-21) |
| R9-09 | Bloom ohne `branch` ⇒ Block-Read je Branch-Ebene; H-21 unerreichbar | branch-bewusster Filter, `BranchTableBlock`, Chain-Resolve mit Early-Exit, Default-Tiefe 4, Spike vor G-F (§5.6, §7.1, §8.3, K-34, INV-BR-5) |
| R9-10 | Metadaten-RAM ≈ 5 GiB bei 16 Mio. Clustern (48 B je Cluster und Datei + 128 B je Cluster); „keine zugesagte Resident-Kapazität“ ohne Mechanismus | `ClusterIndex` ≤ 40 B (≈ 610 MiB), Sparse-Directory, `ClusterFilter`, `HotRecord`, Admission vor Allokation (§4.3.1, §5.6, H-25, INV-ADM-1) |
| R9-11 | Kleinstcluster: sechs versiegelte Blöcke je Abschnitt (≥ 240 B Overhead) widersprach H-11 (≤ 5 % ab 1 KiB) | `MiniSection` (ein AEAD-Rahmen), H-11 neu gefasst (R-PL-8) |
| R9-12 | Entmischen Staging→L1 schrieb für ≈ 16 KiB je Gruppe bis zu 64 MiB L1 um (Faktor ≈ 4000) | Spill-Policy mit Schwelle und Altersgrenze, size-tiered Staging (R-PL-7, H-26) |
| R9-13 | Purge-Kosten bei verstreuten Einzellöschungen; Sealed-Purge mit Klartext-Dringlichkeit | Compaction-Huckepack, Fenster-Amortisierung, Sealed-Hygiene-Lane, getrennte `t_crypto`/`t_physical` (§9.5, H-28) |
| R9-14 | `INV-ERA-8(1)` („bei Cryptographic unlesbar“) widersprach `INV-ERA-2` (Bytes entfernt) | präzisiert: Done verlangt Entfernung auch des Ciphertexts; Unlesbarkeit ab `KeysDestroyed` (§9.5, INV-ERA-2) |
| R9-15 | `ClusterLocal` ohne Admission ⇒ Inode- und FD-Explosion, `max_open_files` als Pseudo-Limit | Mindestgröße, Quoten, `max_live_files`, FD-Cache statt Dateilimit, `RLIMIT_NOFILE`-Prüfung (§4.3.1, §10.1) |
| R9-16 | R-PL-3 „nie zurück“: schrumpfende Cluster hinterlassen unbegrenzt viele Kleinstdateien | Pure-Demotion mit Hysterese per `rebucket` (R-PL-3) |
| R9-17 | §4.3 (`ClusterLocal` erzwingt eigene Dateien) widersprach R-PL-1…3 (Packed/Pure automatisch) | `ClusterLocal` = Pure-Zusage nach Admission; ausgenommen von Demotion (§4.3) |
| R9-18 | §9.2: „`Sealed` ⇒ `Cryptographic`“ widersprach `Logical`, `escalate`, logischer Range-/Space-Löschung | Endstufenregel, `Logical` als Zwischen- bzw. Endziel für range/branch/space, Statuscap `LogicalOnly` (§9.2, §9.7) |
| R9-19 | SD-03 (`blake3`, Kern-Abhängigkeit `chacha20poly1305`) widersprach §5.2 (HMAC-SHA256) und „Krypto nur über Ports“ | Abhängigkeiten nach Crate getrennt (SD-03) |
| R9-20 | `destroy_many` ohne Batchgrenze, Hülle bis 4096, Provider-Batches ≤ 256 | Chunking ≤ 256, parallel (§9.4) |
| R9-21 | Open-Zeit und Admission für Millionen Cluster nicht spezifiziert | OP10-Admission, H-13 um Cluster-Index-Aufbau erweitert (§10.1, §17) |
| R9-22 | Formatwirksame Branch-Entscheidungen wären nach G-F nur per Migration korrigierbar | Spike-Messungen als Freeze-Voraussetzung (§14, P2) |
| R9-23 | Zeitplan §14 für P2/P6/P7 optimistisch | als Risiko (9) geführt, Neuschätzung nach P1/P2 (§18) |
| R9-24 | Race-Nahtstellen deckten Pipeline, Leader-Wechsel, Guard-Fenster, Space-Drop und Link nicht ab | RACE-18…RACE-22 (§11.6), M1/M2 erweitert (§13.1) |

## Anhang B.3 — Änderungen R9.0 → R9.1 (Feinschliff)

R9.1 ändert kein Dateiformat, keine Gate-Reihenfolge und keinen Phasenplan. Alle Änderungen lösen Widersprüche, schließen Lücken oder bereinigen den Text.

| ID | R9.0-Befund | R9.1-Lösung |
|---|---|---|
| R9.1-01 | `Shredded` stand in §3.5 (F-6) und §7 (Punkt 5) als Lesefehler, ist aber nach der Gate-Prüfung (§7, Punkt 2) nie erreichbar | `Shredded` ist nur Clusterzustand; Lesen liefert `Erased`, `KeyUnavailable` oder `KeyLost` (§3.5, §7) |
| R9.1-02 | §9.4 erlaubte den Purge bei `Disabled`; `INV-ERA-3` und §9.5 verlangen `Absent` vor dem Purge | Invariante gewinnt; Purge beginnt bei `Cryptographic` erst bei `Absent`, `Disabled` erlaubt nur Planung (§9.4); allgemeine Rangfolge in der Kopfzeile „Normative Quelle“ und §12.8 |
| R9.1-03 | `ClusterSpec` war nur teilweise definiert (`protection`, `tenant_tag`, `due` fehlten); `KeyProvider::list` lieferte kein Alter, daher war `orphan_key_grace` nicht berechenbar | Felder ergänzt (§4.3); `list` liefert `KeyListEntry{cluster, created_utc}` (§4.3, §9.4). Port-Änderung, kein Format |
| R9.1-04 | `ClusterIndex`: 36 B je Eintrag bei Lastfaktor 0,8 ergeben 45 B, nicht ≤ 40 B; „16 Mio. ≈ 640 MiB“ mischte dezimal und binär; „≈ 4,3 GiB“ war ≈ 4,0 GiB | 40 B als Rechenwert mit Annahme „Füllung ≈ 0,9“ (in H-25 zu belegen); 16 Mio. × 40 B ≈ 610 MiB, Gesamtbudget ≈ 4,0 GiB (§4.3.1, §16). **Offen:** trägt H-25 die Füllung nicht, gelten 45 B (≈ 690 MiB, ≈ 4,5 GiB) |
| R9.1-05 | Die Bezeichner `R1…R10` (Recovery-Schritte) und `R1…R22` (Race-Nahtstellen) kollidierten untereinander und mit den Revisionsnamen R6…R9 | Recovery-Schritte `OP1…OP10` (§10.1), Race-Nahtstellen `RACE-1…RACE-22` (§11.6), alle Verweise angepasst; Revisionsnamen bleiben |
| R9.1-06 | §13.1 nannte „R1–R17“, obwohl R9-24 die Nahtstellen 18…22 ergänzt hatte; §16 führte einen „Slowpath-Anteil“ der Guards, obwohl es keinen Langsampfad gibt (§6.3) | §13.1: `RACE-1…RACE-22`; §16: Stufe-A-Konflikte, `GuardRetry`-Rate, Fensterfuß-Treffer |
| R9.1-07 | Rückblicke im Normtext („R8.2 verlangte …“) in §3.5, §4.3.1, §5.8, §6.3, §7.1, §9.2, §9.4, §9.6, §9.7, §10.4, §14, §16, §19 | durch zeitlose Aussagen ersetzt; die Geschichte steht in Anhang B |
| R9.1-08 | „Committer“ stand an Stellen, an denen der Sequencer entscheidet; §3.4 behalf sich mit einem Sonderhinweis | Sequencer benannt (§2.4, §3.1, §6.5, §8.4, §9.5, §9.6, §11.6, §12.8, §13.1, §15, §19); „Committer“ bleibt Oberbegriff für Sequencer + Syncer (§3.4) |
| R9.1-09 | `BranchId` (u32) kam aus dem `reserve_ids`-Zähler ohne Überlaufregel; `eraser().branch(…)` stand als unzulässige Signatur in `impl Storage`; §8.4 nannte „maximal 8“ Ebenen neben Default 4 | Überlauf ⇒ `Exhausted`; eigener `impl Eraser`; Default 4, hart 8 (§8.4) |
| R9.1-10 | Undefinierte Verweise `P-2`, `P-4`, `P-10`; `E-nn` ohne Erklärung | durch `INV-LIM-1` bzw. Klartext ersetzt; `E-nn` in §12.8 als Auditbefund erklärt |
| R9.1-11 | Keine Rangfolge bei Widerspruch zwischen Invarianten, §12.8 und Fließtext | §15 vor §12.8 vor übrigen Abschnitten (Kopfzeile, §12.8) |


## Anhang B.4 — Änderungen R10 → R11 (Gesamtsystem-Abschluss)

R11 ändert kein Dateiformat gegenüber R9.1, ergänzt aber Formatpunkte, die vor G-F festzulegen sind (Partitionsformat, `PartitionGeneration`-Schlüsselscope).

| ID | R10-Befund | R11-Lösung |
|---|---|---|
| R11-01 | R10 bricht in §11.6 ab; §12–§19 und alle Anhänge fehlen, Querverweise (§12.4.0…§12.4.5, §14.1, §18.1, SD-70, H-35) laufen ins Leere | Kapitel §12–§19 vollständig, alle Verweise aufgelöst |
| R11-02 | R10 hat die Kernkapitel gegenüber R9.1 stark gekürzt (z. B. §9.5 9,3 → 3,5 KB, §9.7 8,2 → 1,6 KB, §6.5, §8.4/8.5, §10.4); normative Details (Ledger-Events, Receipt-Felder, Merge-Ablauf, Tomb-Retire) gingen verloren | §4–§11 wieder auf dem vollständigen R9.1-Wortlaut; aus R10 bleiben §0–§3 (Gesamtsystem, Topologie, Abhängigkeiten) und K-39…K-47 |
| R11-03 | Ein Ingest-Change für Vektor, Text und Graph eines Dokuments passt nicht in `max_change_ops`/`max_change_bytes` | Publish-Record, IG-1…IG-6 (K-51, §12.4.0) |
| R11-04 | Klasse-A-Autarkie der Retrieval-Crates widerspricht der `SegmentOwner`-Pflicht; Ebenenregel verletzt (`storage-ports` L1, `vector → mvcc`) | persistenzneutrale Traits + `contextra-index-bridge`, `storage-ports` Level 0 (K-48) |
| R11-05 | Löschen in geteilten Indizes ohne beschränkte Kosten; „Segmentschlüssel reicht nicht“ ohne Lösung | Partition = Packgruppe, Generationsschlüssel, Kostenmodell (K-49, §12.4.5) |
| R11-06 | „Trennung personenbezogener Daten“ war Zielbild ohne Mechanismus | Trennungsprinzip, Pseudonym-Tresor, `INV-SEP-1` (K-50, §12.4.6) |
| R11-07 | Read-your-writes, Reproduzierbarkeit und Lernsignale ungeklärt | K-52, K-53, K-54, §12.4.4 |
| R11-08 | Graph: geteilte Hyperkanten verbanden Löschdomänen; Personen-Entitäten ohne Besitzer | Kopie je Löschdomäne, Personen-Entitäten im Personencluster (§12.4.3, `INV-GRAPH-PROV-1`) |
| R11-09 | Phasenplan ohne Retrieval-Spur und ohne Migration | R1–R5, §14.1 |
| R11-10 | Spec nannte Crates, die im Repo fehlen (`erasure`, `vault`, `storage-ports`); „Commit `0dd5cfc41`“ nicht prüfbar; Kern kompiliert nicht | Ist-Befunde §18.1, Stand `d8b45b8`, K-55 |
| R11-11 | Konfiguration ohne sichere Bündel; Auskunft/Portabilität fehlten | Profile (K-56), `export_cluster` (K-57), §12.10 |
| R11-12 | Unerklärtes „R9.1+Ö“ im Kopf | entfernt |



## Anhang B.5 — Änderungen R11 → R12 (Repo-Abgleich, HEAD `60ea034`)

R12 ändert kein Dateiformat. Alle Befunde wurden statisch am Quelltext geprüft (Cargo.toml, `use`-Zeilen, `capabilities.toml`, Verzeichnisbaum).

| ID | R11-Befund | R12-Lösung |
|---|---|---|
| R12-01 | R11 setzte `storage-ports` auf Level 0 und beschrieb Retrieval-Crates als „ohne interne Abhängigkeit“; real nutzen `vector`/`text`/`graph` `types` und `ports`, die in `capabilities.toml` auf Level 2 stehen | Fünf-Ebenen-Modell (K-58, §3.1) |
| R12-02 | Fünf `TxId`-Kopien und eine zweite `Epoch` im Bestand | ein `Epoch(u64)` (K-59, `INV-EPOCH-1`) |
| R12-03 | Personenbezogene Daten und geteilte Partitionen: Klartext-Embeddings in geteilten Indizes nicht ausgeschlossen | Indexklassen, `INV-SEP-2`, H-36 (K-60) |
| R12-04 | `kvcache` mit Inhaltsadressierung und `aes-gcm-siv` fehlte in Spec und Bereinigungsplan | K-61, §14.1 |
| R12-05 | `block_on` und `memmap2` nur teilweise benannt | gemessene Verteilung in §14.1 (K-62) |
| R12-06 | Alt-Verträge in `contextra-ports` ungeregelt | K-63 |
| R12-07 | Ist-Zahlen (Mitglieder, `capabilities.toml`, Root-`tests/`, `docs/agent`) falsch bzw. veraltet | §3.1 und §18.1 korrigiert |

## Anhang C — Verworfene Vorschläge

Zeitlanes `lane_min/max` und kumulative Gewichte im SubIndex (K-29, K-30) · mehrere parallele Redo-Logs/Striping zur Anhebung der Fsync-Rate (zerbräche Stamp-Kette und Gate-Ordnung; Skalierung bleibt Instanz-Sharding, K-18) · Branch-„Flatten“ durch Datenkopie (verletzte O(1)-Fork und Löschlokalität) · Retry-freie Langsampfad-Guards im Sequencer (Plattenzugriff unter Rang 1) · persistierter Token-Index im Sequencer (K-38) · Prolly-Tree/Merkle-DAG/Inhaltsadressierung im Kern (K-07, K-08) · CAS-Superblock ohne Log (K-10) · `mmap`/`io_uring`/GPUDirect im Kern (K-11) · Token-IDs, KV-Cache-Tensoren, feste Seitenstruktur (K-13) · gelernte Cross-Objekt-Affinität als Layout (K-17) · Slab-Sub-Allokator (K-20) · Segmentschnitt je Gate · Aufrufer-vergebene ClusterId · `eu_tag` im Klartext-Header · Poison bei ENOSPC · `rekey_clone` · Indizes (ANN, BM25, Graph) im Kern · MCP-Server, Tokenizer, Embedding-Modelle im Kern · benutzerdefinierte Merge-Operatoren im Kern. · persistente Vollindizes je Mandant ohne Partitionierung (Löschen = Vollrebuild) · ein Mikroindex je Cluster für alle Größen · gerichtetes Re-Wiring ohne Rebuild · Pseudo-2PC zwischen Storage und Indizes · geteilte Hyperkanten über Löschdomänen · globales persistiertes Termwörterbuch · Nutzer-Lernsignale in globalen Gewichten · synchrone Indexpflege im Commit-Pfad



## Anhang D — Zustände

```
Cluster:    Live ──Sequencer──► Retiring(S) ──durabler Sync──► Retired(S) ──ER2──► Shredded ──ER6──► Purged ──cluster_trim──► (UnknownCluster)
            (Shredded nur Sealed/Cryptographic; nur vorwärts; Restrictions orthogonal)
Branch:     Live ──merge──► Merged ;  Live ──drop/ttl──► Dropped ;  (alle: sterben mit dem Cluster)
Space:      Live ──SpaceGate(S)──► Dropping(S) ──SD3──► Dropped
Packgruppe: Aktiv ──PD1──► Abwurf ──PD4──► Entfernt   (Restricted-Cluster verbleiben)
Datei:      Building ──rename+Edit──► Lebendig ──Edit──► Obsolet ──letzte Referenz──► Unlink
Erase:      Gated ─► KeyDestroying ─► KeysDestroyed ─► Purging ─► Done   (Absturz an jeder Stelle: Fortsetzen)
Anker:      monoton; Catalog-Epoche ≥ Anker und Ledger enthält Ankerhash beim Öffnen, sonst Rollback (F-9)
Instanz:    Normal ─free_space<Reserve─► DiskFull (Löschung läuft) ─Hysterese─► Normal ; Normal ─EIO/Fsync─► Poisoned (Neustart)
            Poisoned ─Erase-Request─► Poisoned + ErasePending (Intent im Ledger) ─Neustart─► Gate (§9.5.2)
Tomb-Epoche: Open ──rotate──► Closed ──Backup-Horizont + Provider-Absent──► Retired   (nur vorwärts; Default: nur Epoche 0)
View:       Lebend ──Lease-Ablauf / Revoke──► Widerrufen (SnapshotTooOld)
Pipeline:   Intake ─Sequencer(Stamp, Retiring)─► Syncer(write+sync) ─► Publikation (Stamp-Reihenfolge) ─► Receipt ;
            Sync-Fehler ⇒ Gruppe und alle späteren Indeterminate/Poisoned, nichts davon veröffentlicht (INV-PIPE-2)
Index:      Partition(g) ──Löschung im Fenster──► Neuaufbau g+1 ──verify_absent + Publish──► Schlüssel g vernichtet ──► Partition(g) unlesbar
Ingest:     Staging(v n) ──Abschluss-Change (head)──► sichtbar + projiziert ;  Crash vor head ⇒ unerreichbar ⇒ Orphan-Worker
```

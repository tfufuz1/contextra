# SPEC.md — `contextra-storage` (Speicherkern), Revision R9.1 (+ Ökosystem §3.7)

| | |
|---|---|
| Status | **Revision R9.1 (Feinschliff von R9.0, keine Formatänderung), Entwurf zur Freigabe als Gate G-0.** Ersetzt SPEC R7 und konsolidiert sechs Quellen: R7 (`SPEC(18).md`), Lethe-Architekturentwurf, Lethe-Engine-Spec, Mnemos-Konzept, „Optimal-LLM-Datenbank“ (Engram, AetherStore ×2, CairnDB) und die Octa-Spec (Abgleich R8.1 → R8.2, Anhang B). |
| Gegenstand | Eingebetteter, synchroner, crash-sicherer Speicherkern für LLM-/Agenten-Workloads mit atomaren Mehr-Space-Änderungen, MVCC-Sichten, **versionskontrollartigen Branches und Merges (clusterlokal)** und **vollständiger, nachweisbarer Löschung personenbezogener Daten** (logisch, kryptografisch, physisch). |
| Verifikation | Der in `crates/contextra-storage/PHASE-STATUS.md` dokumentierte Teilcheck umfasst Storage-/Crypto-Compile sowie ausgewählte Config-, Catalog-/Redo-Crash-Sweeps und Strukturtests; vollständige Release-Gates sind offen. Keine Leistungszahl ist als gemessen anzusehen. R7-Messungen wurden nicht neu erhoben. Formate brauchen vor **G-F** Referenzvektoren aus einem unabhängigen Codierer. |
| Rechtlicher Vorbehalt | Der Kern liefert technische Eigenschaften, keine Rechtsaussage zu Art. 17 DSGVO, Fristen oder Ausnahmen. „Gelöscht“ ist immer an eine Stufe aus §9.2 gebunden. |
| Normative Quelle | **Diese Datei ist die einzige normative Fassung.** `docs/SPEC-R8.md` entfällt; Besitzer-SPs, `00-INDEX.md` und `PHASE-STATUS.md` verweisen hierher und dürfen sie nicht überschreiben. Rangfolge bei Widerspruch: §15 (Invarianten) vor §12.8 vor den übrigen Abschnitten. |
| Sprache | Prosa deutsch, Bezeichner englisch. |
| R9-Audit | R9.0 schließt die in Anhang B (R9-01…R9-24) gelisteten Engpässe, Lücken und Widersprüche von R8.2: Commit-Pipeline ohne E/A unter Rang 1, branch-bewusster Lesepfad, skalierende Cluster-Metadaten, amortisierte Löschung, Datei-/FD-Admission. Alle neuen Leistungsaussagen sind **Entwurfsziele** (H-24…H-28), keine Messungen. |
| R9.1-Feinschliff | R9.1 ändert kein Format. Es löst Widersprüche und Lücken (Anhang B.3, R9.1-01…R9.1-11), korrigiert Rechenwerte und ersetzt Rückblicke im Normtext durch zeitlose Aussagen. |

---

## 0. Kurzfassung

**Contextra ist als europäische Datenbank für Unternehmens-LLM-Systeme konzipiert: Dokumente, Vektoren, Text, Metadaten und Graphbeziehungen werden mit einer gemeinsamen Versions-, Sichtbarkeits- und Löschsemantik verbunden.** `contextra-storage` ersetzt den bisherigen `contextra-store` als crash-sicheres Fundament. Es speichert kanonische RAG-Daten und deren Lineage atomar; Retrieval-Indizes bleiben obere, über Watermarks und Löschatteste angebundene Komponenten. So entsteht eine integrierte Datenbankerfahrung ohne zu behaupten, ANN, BM25 oder Graphsuche seien Teil des KV-Kerns.

**Ein Cluster ist die Einheit, in der Daten leben, versioniert werden und sterben.** Alles zu einer Person, einem Fall oder einem Arbeitskontext liegt in einem Cluster: Nutzdaten, Verlauf, Branches und clustergebundene Ableitungen. Der Cluster ist zugleich **Löschdomäne, Schlüsseldomäne, Versionsdomäne und Platzierungseinheit**. Daraus folgt die Architektur:

| Eigenschaft | Wie sie entsteht |
|---|---|
| Atomare Änderung über Spaces | ein Redo-Frame je `Change`, ein Gruppen-Fsync, Sequencer + Syncer (§6.1) |
| Tail-Latenz und Skalierung | Commit-Pipeline: Sequencer ohne E/A unter Rang 1 plus Syncer (§6.1); branch-bewusster Lesepfad (§7.1); Metadaten- und Datei-Admission (§4.3.1); amortisierte Löschung (§9.5) |
| Git-artige Versionierung | Stamps = Commit-Reihenfolge; `view_at`; **Branches je Cluster** (O(1)-Fork, Diff, Merge mit Konfliktliste); kein Merkle-Baum, keine Inhaltsadressierung |
| Löschung in drei geordneten Schritten | **Gate** (Millisekunden, Cluster ist ab Stamp `S` für alle Sichten `Erased`) < **Schlüssel** (Clusterschlüssel vernichten, alles des Clusters unlesbar) < **Purge** (Bytes aus lebenden Dateien entfernen, Rohkopie ohne Entschlüsseln) |
| Gate/KeyDestroy ∝ Zielcluster; physischer Purge ∝ betroffene Dateien/Extents | clusterreine Abschnitte + `EraseLocality`; `ClusterLocal` vermeidet fremde Survivor-Rewrites, `PackedBounded` tauscht I/O gegen geringeren Platzverbrauch |
| Rollback-/Wiederkehr-Schutz | hashverkettetes Ledger (nur `tomb_id`, keine ClusterId) + monotoner Anker |
| RAG-Anbindung | fünf Verträge: `SegmentOwner`, `ExtensionBlock`, `Consumer`/Change-Feed, Kopf+Gewichte+`page`, `filter_visible` |

**Nicht im Kern:** Tokenizer, Embeddings, ANN, Volltext, Graph-Suche, Trust-Politik, Zoom-Stufen, Rechtsgrundlagen, Fristen, Mandanten- und Rechteprüfung, Replikation. Sie setzen auf den Verträgen in §12 auf.

**Was der Kern nicht löschen kann:** Prozessspeicher, Swap, Core-Dateien, Dateisystem-Journale, Flash-Reste, externe Backups mit Schlüsselkopien, Daten außerhalb des Kerns (Prompt-Caches, Provider-Logs). Jedes Receipt nennt diese Restexposition (§9.9).

---

## 1. Konfliktregister: Entscheidungen der Konsolidierung

Die Quellen widersprechen sich an 28 Stellen; K-29…K-32 halten die Entscheidungen zu den Vorschlägen des Octa-Abgleichs fest, K-33…K-38 die Entscheidungen des R9-Audits (Anhang B). Jede Entscheidung ist final; die Spalte „Wirkung“ nennt den Paragrafen, der sie umsetzt.

| ID | Konflikt | Entscheidung | Begründung | Wirkung |
|---|---|---|---|---|
| K-01 | Löscheinheit heißt „Cluster“ (R7, Lethe-Entwurf) bzw. „EU“ (Lethe-Spec) | **Cluster** | ein Begriff, R7 ist Referenz | §4 |
| K-02 | ClusterId: Aufrufer-ID / HMAC-`eu_tag` (Lethe-Spec, R6) vs. vom Kern erzeugt (R7) | **Kern erzeugt 128-Bit-Zufall**, kein `eu_tag`, kein `tenant_tag_key` | keine Wiederverwendung, nicht ableitbar, kein zusätzliches Schlüsselmaterial im Klartext-Pfad | §4.3 |
| K-03 | Ledger speichert ClusterId (R7) vs. pseudonymes `tomb_id` (Lethe-Entwurf) | **domain-separiertes `tomb_id`** | Kein Klartext-Identifier im dauerhaften Ledger; das HMAC bleibt linkbares Pseudonym und wird deshalb als personenbezogenes Metadatum behandelt, nicht als anonym | §9.7 |
| K-04 | Schlüssel: Cohort-KEK-Rewrap (Lethe-Spec) vs. Schlüssel je Cluster (R7) | **Port `KeyProvider` (je Cluster adressiert)**; Cohort-Rewrap ist eine *Strategie innerhalb* eines Providers (`keys-cohort`), nicht Kernlogik | Kern bleibt einfach; Skalierung auf Millionen Cluster bleibt möglich | §9.4 |
| K-05 | Ein globaler Commit-Log (Lethe) vs. Redo mit Committer und Catalog-Edits (R7) | **R7-Modell** | R7 ist bis zur Absturzmatrix durchgearbeitet | §6, §10 |
| K-06 | Segmentschnitt bei jedem Gate (R6) vs. keiner (R7) | **Kein Schnitt**; Barriere nur beim Purge von `Plain` | Commit-Pfad bleibt frei von Löschkosten | §9.5 |
| K-07 | Versionierung: Prolly-Tree/Merkle-DAG (Mnemos, Engram, Aether, Cairn) vs. R7 NZ-13 (kein Branching im Kern) | **Clusterlokale Branches im Kern** (Mission), **ohne** Merkle/Inhaltsadressierung | Branches ohne Dedup sind löschlokal; ein globaler DAG über Cluster würde Löschung unmöglich machen | §8 |
| K-08 | Inhaltsadressierung/Dedup (Engram, Aether, Mnemos) | **Nein** (`INV-PRIV-1`) | Hash über Klartext = Seitenkanal, Dedup = Querverweis zwischen Löschdomänen | §3.3 |
| K-09 | „Hold“ als Rechtsfunktion mit Vier-Augen-Release (Lethe-Spec) vs. `Restriction` ohne Override (R7) | **`Restriction`** (R7) mit wählbarer `LiftPolicy` (`OwnerOrAdmin` oder `FourEyes` über Port `Authority`); `Hold` bleibt **technisch** | Mechanismus ohne Politik, aber Vier-Augen-Prinzip verfügbar | §9.6 |
| K-10 | Commit ohne WAL per CAS-Superblock (Aether) | **Verworfen** | DiskANN-/Bitmap-Updates und Mehr-Space-Atomarität brauchen ein Log | §6 |
| K-11 | `mmap`, `io_uring`, `O_DIRECT`, GPUDirect (Aether, Engram) | **Nur als `Fs`-Implementierungen**, nie im Kern | Fehlerinjektion muss jeden Zugriff sehen | §3.1 |
| K-12 | Async-Runtime (Mnemos, Cairn) | **Synchroner Kern**, Async-Fassade oben | Race-Freiheit prüfbar, `loom`/Sim möglich | §3.4 |
| K-13 | Token-IDs, KV-Cache-Blöcke, feste `AlignedContextPage` (Aether) | **Verworfen im Kern**; Token-Gewichte als `weights[W]`; Token-IDs/KV-Verweise gehören in Kopf/Körper der oberen Schicht | Formatbruch bei Modellwechsel; Cloud-APIs erlauben keinen KV-Zugriff | §4.5 |
| K-14 | Zoom-Leiter L0…Ln (Mnemos, Engram) | **Konvention oben**: Zoom-Stufen sind Schlüsselsuffixe im selben Cluster | Kern bleibt bedeutungsfrei; Löschung erfasst Stufen automatisch | §12.6 |
| K-15 | Trust-/Taint-Labels (Mnemos, Engram) | **Konvention oben** (erstes Kopfbyte), Vererbung = Minimum über `DerivedFrom` | Politik, nicht Mechanismus | §12.6 |
| K-16 | Indizes im Kern (Cairn, Engram, Aether) | **Oben**, über Verträge; Index-**Watermarks** (Cairn) als Pflichtmuster | Löschbarkeit erzwingt Segmentverträge, nicht Kernindizes | §12 |
| K-17 | Gelerntes Affinitäts-Layout über Objekte hinweg (Mnemos, Engram, Grok) | **Verworfen**; Layoutfreiheit nur *innerhalb* eines Cluster-Abschnitts (Schlüsselordnung gestaltet der Aufrufer) | kollidiert mit Clusterreinheit (INV-PL-2) | §5 |
| K-18 | Single-Sequencer als Engpass (Cairn-Kritik, Lethe-Spec B.5) | **Ein Committer je Instanz**; Skalierung durch mehrere Instanzen (Tenant-Sharding) oben, keine instanzübergreifende Atomarität | ehrlich benannte Grenze | §11 |
| K-19 | Durability `Sync/GroupSync/Async` (Cairn) vs. `Durable/Relaxed` (R7) | **`Durable` (Gruppen-Fsync) / `Relaxed`** | gleiche Semantik, weniger Modi; `Relaxed` nie für Löschung | §6 |
| K-20 | Slab-Sub-Allokator mit Free-Lists (Grok) | **Verworfen** | veränderliche Free-Lists widersprechen unveränderlichen Dateien und Rohkopie | — |
| K-21 | Guard-Prüfung im Committer (kalte E/A unter Sperre) | **Zweistufig** (R7; R9: Stufe A auf dem Aufruferthread exakt, Stufe B im Sequencer nur Delta, §6.3) | keine Plattenlesezugriffe unter Rang 1 | §6.3 |
| K-22 | Ledger im Datenvolume (R7) vs. eigene Failure Domain (Lethe) | **`ledger_dir` konfigurierbar**, Produktivempfehlung: getrenntes Volume; Anker bindet den Kopf | beide Quellen vereint | §9.7, §10.3 |
| K-23 | Stufen G0/G1/G2 (Lethe-Entwurf), `Logical/Physical/Scrubbed/Cryptographic` (R7), Ticket-Stufen (Lethe-Spec) | **Zwei Achsen**: `EraseLevel` = gewünschter Endzustand; `Stage` = Fortschritt (`Gated`=G0, `KeysDestroyed`=G1, `Purged`/`Done`=G2) | keine Doppeldeutigkeit | §9.2 |
| K-24 | Zuordnung Person → Cluster | **Normatives Muster in §12.7**: Sealed-Wurzel je Person plus registrierter `IdentityDirectoryOwner` mit tenant-scoped HMAC-Indexschlüssel und per-Subject-Entry-DEK; Löschung widerruft Cluster und Directory-Eintrag | Verhindert gemeinsame Shard-DEKs und Überlöschung. HMAC bleibt pseudonymisierend, nicht anonymisierend; Schlüsselkompromittierung wird als Restrisiko geführt. Entry-DEK-Zerstörung und Owner-Attest sind Teil der Löschkette | §12.7 |
| K-25 | Daten mehrerer Betroffener | **Gemeinsamer Fallcluster + Links** (R7); keine mehrclustrigen Einträge | Purge ohne Querverweise | §9.8 |
| K-26 | Bitemporalität (Mnemos, Cairn: vier Zeitachsen) | **Stamp = Transaktionszeit im Kern**; Gültigkeits-/Beobachtungs-/Quellzeit im Kopf der oberen Schicht (Konvention) | Kern ohne Zeitlogik | §12.6 |
| K-27 | Siegelformen (Redo je Segment, Block mit `file_id`) | **Eine Siegelform ohne Datei-/Positionsbezug** (R7) | Rohkopie in jedem Modus gültig | §5.2 |
| K-28 | R7-Redaktionsfehler: doppelte `SD-38/39/40`, Rang 5 „entfällt“ | **Neu nummeriert** (§19), Ränge 1–8 durchgehend | Konsistenz | §11.2, §19 |
| K-29 | Zeitlanes `lane_min/max` je Block im SubIndex (Octa-Abgleich A1) | **Nein im Kernformat.** Stattdessen Erweiterungsvertrag `BlockPruner` über `ExtensionBlock` (§12.2) | Blockskipping nach einem Zeitprädikat ist unter MVCC nur bei schattensicheren Quellen korrekt: ein übersprungener Block kann die neuere Version enthalten, die eine ältere Version in einem nicht übersprungenen Block verdeckt, und die Anfrage lieferte dann die veraltete Version. Diese Zusicherung kann nur die obere Schicht geben (K-26). Kein Formatbruch vor G-F | §12.2, §8.6 |
| K-30 | Kumulative `cum_weight[W]`/`cum_count` im SubIndex (A2) | **Nein im Format.** Präfixsummen werden beim Laden des SubIndex im RAM berechnet | Ein Abschnitt hat ≤ 64 MiB, also ≤ ca. 4096 Blöcke; die Traversierung des RAM-Indexes ist vernachlässigbar (in H-4 zu messen). Persistierte Summen kosten Formatbytes und sind bei Überlappung/Schatten ohnehin nur eine Obergrenze | §7 |
| K-31 | `tomb_id` bleibt bis ans Ledger-Ende ein linkbares Pseudonym (offenes Restrisiko aus K-03; A3) | **Ja: `tomb_epoch` als Format-Slot jetzt.** Tomb-Schlüssel je Epoche liegen im `KeyProvider` (nicht in `KEYS`, nicht in Backups); Rotation und Retire sind optional (Default: aus) | Nach dem Backup-Horizont (§10.4) werden Ledger-Einträge anonym, ohne die Kettenprüfung zu verlieren. Der Slot ist jetzt billig, nach G-F nur per Migration nachrüstbar | §4.1, §9.4, §9.7, §10.4 |
| K-32 | Derivation Cache (Embedding-/Token-Memoisierung) vs. `INV-PRIV-1` (B4) | **Ja als Konvention** der oberen Schicht; `INV-PRIV-1` präzisiert; zufälliges Cluster-Secret statt Kern-API | Keyed MAC unter einem im Cluster versiegelten Zufallssecret ist clusterlokal, stirbt mit dem Cluster und bleibt über Löschdomänen nicht vergleichbar. Spart Re-Embedding bei Re-Ingest, Merge und Modellwechsel | §3.3, §12.6 |
| K-33 | Committer hielt Rang 1 über Redo-Sync, Guard-Langsampfad, Graph-Traversierung der Löschhülle (≤ 4096 Cluster) und Catalog-Fallback (R9-Audit 1) | **Pipeline aus Sequencer (Rang 1, nur RAM) und Syncer (ein Schreiber, kein Lock).** Hülle, Guards, Krypto und Idempotenz werden auf dem Aufruferthread vorbereitet; der Sequencer prüft nur `policy_epoch` und das Guard-Delta | p99 aller Agenten hängt nicht mehr an der langsamsten Gate-/Guard-Berechnung. Die Fsync-Rate bleibt die physische Obergrenze; Gruppengröße ist die Stellschraube | §6.1, §6.3, §9.5, §9.6, §11 |
| K-34 | Bloom ohne `branch` ⇒ Block-Reads je Branch-Ebene, H-21 unter R8.2 unerreichbar (R9-Audit 2) | **Branch-bewusster Filter `(branch, user_key)`, `BranchTableBlock`, Chain-Resolve mit Early-Exit, Default-Tiefe 4**, Spike-Messung vor G-F | `cmp` sortiert `branch` vor `user_key`: Branch-Bereiche sind zusammenhängend. Das Format ist noch nicht eingefroren, die Korrektur ist jetzt billig | §5.6, §7.1, §8.3 |
| K-35 | Metadaten-RAM ∝ Cluster × Dateien, Kleinstcluster mit sechs versiegelten Blöcken (R9-Audit 3) | **`ClusterIndex` ≤ 40 B, Sparse-Directory, `HotRecord` nur bei Bedarf, `MiniSection`, Admission vor Allokation; Konvention „Session ≠ Cluster“** | Der Cluster folgt der Löschdomäne, nicht der Sitzung; ein AEAD-Rahmen statt sechs für Kleinstabschnitte | §4.3.1, §5.6, §12.6 |
| K-36 | Packgruppen-Write-Amplification (Entmischen schrieb 64-MiB-L1 für ≈ 16 KiB), Einzel-Purge-Kosten (R9-Audit 3) | **Staging-Spill-Policy, Fenster-Amortisierung, Compaction-Huckepack, Hygiene-Lane für Sealed-Purge** | Entmischen erst bei genug Bytes je Gruppe; Kryptolöschung wirkt sofort, die Byte-Hygiene darf bis zur Frist warten | R-PL-7, §9.5 |
| K-37 | `ClusterLocal` ohne Admission ⇒ Inode-/FD-Explosion, „nie zurück“-Pure-Promotion (R9-Audit 4) | **Mindestgröße, Quoten, Pure-Demotion, FD-Cache statt FD-Limit, Verzeichnis-Sharding** | Ressourcenschutz ist Mechanismus, keine Politik; keine stille Herabstufung der Löschlokalität | §4.3.1, §5.4 |
| K-38 | Idempotenz-Prüfung als Sonderpfad im kritischen Pfad (Race zweier gleicher Tokens) | **Implizite `Absent`-Guard auf den SYS_IDEM-Schlüssel** | derselbe Zwei-Stufen-Mechanismus wie jede Guard; kein Token-Index im Sequencer, kein RAM ∝ aktive Tokens | §6.5 |

### 1.1 Herkunft der übernommenen Elemente

| Quelle | Übernommen | Nicht übernommen |
|---|---|---|
| **R7** | Gesamtgerüst: Cluster, Sealed/Plain, Pure/Packed/Staging, Committer, Catalog-Edits, ER0–ER7, Anker, Notbetrieb, Absturzmatrix, Invarianten, TLA+-Gates | `rekey_clone`, Aufrufer-ClusterId |
| **Lethe-Entwurf** | G0/G1/G2 als Fortschrittsstufen, `Redacted`/`Erased` im Change-Feed, `tomb_id`, `Volatile`-Semantik, `DerivedSegment::on_gate/verify_absent`, Person→Wurzelcluster-Muster, Löschorakel O-Files/O-Device/O-Gate/O-Ledger/O-Segments, Byte-Zähler-Lokalitätstest (INV-22) | eigene Redo-/WAL-Struktur |
| **Lethe-Spec** | `ErasureReceipt`-Inhalt und `verify_receipt`, Bedrohungsmodell, Restore-Reconcile, getrennte Failure Domain für Ledger, versiegelte Cursor, `KeyUnavailable/KeyLost`-Unterscheidung, Vier-Augen-Lift, Restrisikotabelle | Cohort als Kernlogik, `eu_tag` |
| **Mnemos** | Zoom/Handles/Trust/Derivation als Konventionen oben, Work Queue oben, Fork-TTL (speculative branches), typisierte Merge-Strategien (oben), Idempotenz, Crypto-Shredding-Gedanke, Evaluationsmethodik (Utility@Budget) | Prolly-Tree, Content-Addressing, Zugriffs-gelerntes Layout |
| **Engram** | token-budgetierter Seitenabruf, präfix-stabile Ordnung (oben), Fehler als handlungsleitender Text (oben), Lineage/Invalidierung (oben) | Merkle-DAG, MCP im Kern |
| **AetherStore (Gemini)** | Branch-Sichtbarkeitsmaske als Idee → hier als Branch-Kette im Merge-Iterator | CAS ohne WAL, GPUDirect, feste Seitenstruktur |
| **AetherStore (Grok)** | Locality-Group-Gedanke (= Cluster), Typ-abhängiges Alignment als Option der Block-Codecs | Slab-Allokator, `mmap` |
| **CairnDB** | **Index-Watermarks** (`applied_commit_id`) als Pflichtmuster für abgeleitete Indizes, Konfliktmatrix je Operationsart (als Guard-Katalog), Benchmark-Gruppen (Storage / Agent / Retrieval), WiscKey-artige Wertetrennung, Mandanten-Vorfilterung vor ANN | zentraler Sequencer als Sharding-Ersatz, Index-Typen im Kern |
| **Octa-Spec** | Abgleich R8.1 → R8.2: Tomb-Epochen/Backup-Horizont (A3), Provider-Zusicherung und Host-Befund, Messarten im Löschbeleg, View-Lease, Löschauftrag auf vergiftetem Handle | Zeitlanes und kumulative Gewichte im Kernformat (A1/A2 → K-29/K-30), eigene Crate-/Port-Topologie, Ebene-P/K-Modell (liegt oberhalb des Kerns) |

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
| L14 | **Eine Siegelform** für alles, unabhängig von Datei und Position. |
| L15 | **Ein Purge-Algorithmus** für Pure, Packed, Staging, Packgruppen-Abwurf, Restriction-Ausnahme. |
| L16 | Redo-Barriere für `Plain`-Löschung beim Purge, nicht beim Gate. |
| L17 | **Branches sind clusterlokal und löschen mit dem Cluster.** Keine instanzweiten Commit-DAGs. |
| L18 | TLA+-Modelle und deterministische Simulation sind Gate-Bestandteil. |
| L19 | Der Kern ruft kein Modell auf und kennt keine Politik. |
| L20 | **Open-Core-Grenze:** Formate, Kernsemantik, Ports, Test-/Simulationsverträge und Referenzverhalten bleiben offen und vollständig genug, um ein interoperables System ohne proprietäre Erweiterung zu betreiben. Monetarisierbare Produkte implementieren dieselben offenen Ports und dürfen keine Kerninvariante abschwächen oder für Basis-Durability/Löschung erforderliche Funktion hinter eine Bezahlschranke legen. |

### 2.2 Ziele

Z-1 Atomarität über Spaces und Branches · Z-2 Durability (sichtbar nur nach Fsync) · Z-3 Snapshot-Isolation ohne Lesesperren · Z-4 Gate/KeyDestroy unabhängig von Storegröße, Purge bounded durch betroffene Datei-/Extension-Partitionen · Z-5 Ablauf einer Packgruppe als O(Dateien) · Z-6 Nachweisbarkeit (Receipt, `verify`, Orakel, Ledger, Anker) · Z-7 Kopf ohne Körper-I/O, Budgetseiten ohne Body-Block-Lesen · Z-8 Race-Freiheit durch Konstruktion, belegt durch Modell, Simulation, Loom · Z-9 begrenzte Ressourcen · Z-10 stabile Verträge nach oben · Z-11 flüchtige Arbeitsbereiche · Z-12 Idempotenz für Agenten-Retries · Z-13 **Gate/KeyDestroy bei vollem Datenträger; physischer Purge nutzt vorgehaltene Scratch-Reserve** · Z-14 überwachbare Löschfristen · **Z-15** O(1)-Fork, indexierter Diff und konfliktgeprüfter Merge je Cluster. **Z-16 (Datenklassifikation):** `DataClass` ist ein geschlossenes Enum; ungültige Kombinationen (Personal ohne Cluster/Encrypted, bei Durable ohne Cluster-Seal) werden vor dem ersten I/O abgewiesen und durch API-/Property-Tests abgedeckt. **Z-17 (Observability):** Gate, KeyDestroy und Purge haben getrennte Dauer-/Ergebnis-/Byte-Metriken; feste Label-Allowlist, keine IDs/Inhalte; Katalogprüfung weist unzulässige Labels zurück. **Z-18 (Entwicklerdiagnose):** jeder öffentliche Fehler hat stabilen Typ und sichere `hint()`-Handlung; Fassade kann eine typisierte `suggested_call` ergänzen. Simulationsfehler enthalten reproduzierbaren Seed und Inspector-Ausgaben bleiben inhaltsfrei; Gates prüfen Reproduzierbarkeit und Leakage. **Z-19 (Tail-Latenz):** unter Rang 1 keine E/A, kein Provider-, Graph- oder Plattenzugriff; der kritische Abschnitt ist O(Ops der Gruppe) im RAM (H-24). **Z-20 (Metadaten-Skalierung):** RAM, Dateien und Dateideskriptoren wachsen kontrolliert mit der Clusterzahl; jede dieser Ressourcen hat Admission vor Allokation (§4.3.1, H-25, H-27).

### 2.3 Nicht-Ziele

NZ-1 Verteilung/Konsens in v1 · NZ-2 eigene Kryptografie · NZ-3 Schemas, Sekundärindizes, SSI · NZ-4 benutzerdefinierte Merge-Operatoren im Kern (Tag `6` reserviert) · NZ-5 Schutz von Prozessspeicher/Swap/Core-Dateien · NZ-6 Umordnung über Clustergrenzen · NZ-7 Altformat-Kompatibilität · NZ-8 Windows-Durability ist kein implizites Versprechen; Produktionsfreigabe erfordert bestandene Fs-Konformanz auf Windows · NZ-9 `mmap`/`O_DIRECT`/`io_uring` im Kern · NZ-10 Kompression in v1 (`COMPRESSION=0`, Feld/Featurewert reserviert) · NZ-11 Inhaltsadressierung, Dedup · NZ-12 Tokenizer, Embeddings, ANN, Volltext, Graph · NZ-13 clusterübergreifende Commit-DAGs, Merkle-Zustandshashes · NZ-14 Lösch-Orchestrierung, Berichte, Signaturen · NZ-15 Schlüsselverwaltung (→ `KeyProvider`) · NZ-16 Mandanten-/Rechteprüfung · NZ-17 Async-Fassade · NZ-18 KV-Cache-/Prefix-Store · NZ-19 zwei lebende Klone einer Instanz · NZ-20 Multi-Node-Replikation/Failover bis zu einem späteren, eigenen Protokoll-Gate.

**Verteilbarkeit vorbereiten, nicht vorwegnehmen:** IDs und Stamps sind instanzlokal; jedes externe Ereignis/Receipt muss daher `(instance_id, stamp)` tragen und darf Stamp allein nie als global sortierbar behandeln. Storage-Ports bleiben austauschbar und alle Zustandsänderungen laufen über versionierte Redo-/Catalog-Records. V1 verspricht weder Replikationslog, globale Reihenfolge, Split-Brain-Schutz noch automatisches Failover. Ein späteres verteiltes Profil braucht ein separates Protokoll für Leader-Epoch/Fencing, Replikationsquorum, Restore/Erase-Anker, Idempotenz über Knoten und Cluster-Platzierung; es darf die lokale Einzelknoten-Durability-Semantik nicht still ändern.

NZ-19 ist eine Betriebsinvariante, keine durch den lokalen `LOCK` allein beweisbare Eigenschaft: `LOCK` verhindert zwei Prozesse am selben Dateipfad, nicht zwei wiederhergestellte Kopien auf verschiedenen Hosts. Restore erhält dieselbe `InstanceId` und darf schreibend erst nach extern belegtem Fencing des bisherigen Besitzers starten. Rollback-Anchor und Besitz-Lease sind verschiedene Verträge; fehlt der externe Fencing-Nachweis, bleibt die Kopie `RestoreQuarantine`/ReadOnly. Diese Prüfung gehört in das Contextra-Deployment-/Restore-Runbook und G-CUTOVER.

### 2.4 Bedrohungsmodell

| ID | Angreifer/Fehler | Gegenmaßnahme |
|---|---|---|
| AT-1 | Prozess mit Lesezugriff auf das Verzeichnis, ohne Schlüssel | Siegel (§5.2) |
| AT-2 | Besitzer des Datenträgers, ohne Schlüssel | Siegel, Schlüssel außerhalb (`KeyProvider`) |
| AT-3 | Dateimanipulation (Einfügen, Abschneiden, Vertauschen) | Ketten-MAC, AEAD-AAD, Katalog-Kette |
| AT-4 | fehlerhafte/bösartige obere Schicht | nur versiegelte `ExtStore`-/`SegmentIo`-Wege; Orakel-Konformanz |
| AT-5 | **Rollback** auf frühere konsistente Stände, um Gelöschtes zurückzubringen | Ledger + Anker (§10.3), Restore-Reconcile |
| AT-6 | Betreiber mit Löschwunsch gegen Restriction | Prüfung im Sequencer (`policy_epoch`) **und** Purge; `lift` nur nach `LiftPolicy` |
| F-1…F-9 | Torn Write, Fsync-EIO, ENOSPC, Bit-Rot, Schlüsselverlust, Crash im Löschprotokoll, Retry-Stürme | §3.5, §10 |

Annahmen: AN-01 `sync_*` macht durabel · AN-02 Schreibvorgänge können zerreißen · AN-03 Fsync-Fehler nicht wiederholbar · AN-04 `rename` atomar · AN-05 Prozessspeicher vertrauenswürdig, nicht kontrolliert gelöscht · AN-06 `destroy_many` macht Schlüssel bei Rückkehr unbenutzbar · AN-07 starker Zufall · AN-08 Anker außerhalb des Datenverzeichnisses · AN-09 Stamps nie aus Uhr; Deadline-/TTL-Zeit nutzt monotone Laufzeit, UTC und durablen `clock_floor`. Rücksprung wird geklemmt; UTC-Sprung über `max_clock_skew` friert Hold-/Retention-Abläufe ein, alarmiert und markiert Erase-Frist als `ClockUncertain`, statt vorzeitig zu löschen oder Fristtreue vorzutäuschen · AN-10 Append-Dateien schreiben nur in vorallokierte Bereiche (ENOSPC nur beim Vorallokieren).

### 2.5 Produktverträge (vom sichtbaren Ergebnis rückwärts spezifiziert)

| Anwender sieht | Kern-/Systemvertrag | Freigabebeweis |
|---|---|---|
| Erfolgreiches `Durable`-Write überlebt Crash; Timeout ist als `Indeterminate` retrybar | Receipt enthält `(instance_id,stamp)`; `EraseRequestId`/`IdemToken` verhindern Doppelwirkung; kein ACK vor Sync | FaultFs-Crash-Sweep über alle Write-/Sync-Grenzen, Idempotenzmodell |
| Recall liefert nur sichtbare, erlaubte Records und hält Tokenbudget ein | `view_durable`, vollständiges Index-Delta bis View-Stamp, Gate-Prüfung vor Payload und Ausgabe, `page` überschreitet Budget außer `oversize_single` nicht | Differential Retrieval; Recall@Budget; Race Gate × Query; IndexLag explizit |
| Löschauftrag ist sofort unsichtbar und später nachvollziehbar | durables Ledger-Intent vor Gate, GateApplied+Anchor vor KeyDestroy/Purge; Status benennt `Blocked`, `Incomplete` oder `CompleteWithResidual` präzise | Crash/Restore-Reconcile vor/nach jedem Schritt; ErasureOracle; unabhängige Receipt-Verifikation |
| Branch-Experiment ist isoliert und Merge verliert keine parallele Änderung | Branches clusterlokal; direkte Child→Parent-Merges; Punkt-/Intervall-Guards und Head-Revalidation; Konflikt schreibt nichts | Referenzmodell, deterministische Race-Simulation, Branch-/Floor-Property-Tests |
| Lastspitze verursacht kontrollierte Verzögerung statt RSS-/Disk-Explosion | gemeinsame RAM-/Scratch-/FD-/Lag-Budgets; Admission vor Allokation; Backpressure mit sichtbarem Fehlerzustand | Combined RAG-Soak mit RSS-Spitzen, Scratch, Lag und tail-latency |
| Restore ist dieselbe Datenwahrheit wie vor dem Crash, ohne gelöschte Daten | Ledger/Anchor führen Restore; ungelöste Scope-Key-/Anchor-/Owner-Abhängigkeiten halten `RestoreQuarantine` | Restore-Matrix aus alten Backups, Intent-Präfixen, fehlendem KMS und altem Anker |

Ein Produktversprechen gilt erst, wenn der benannte Beweis-Gate bestanden und reproduzierbar archiviert ist. Eine API-Funktion ohne definierte Crash-/Retry-/Restore-Semantik wird nicht als stabil exportiert.

### 2.6 Open-Core-Produktgrenze

Open-Core ist ein Packaging- und Betriebsmodell, keine technische Abhängigkeit des Datenformats. Offen bleiben: Storage- und Receipt-Formate, Kern- und Port-Crates, Referenz-Provider/-Anker, Konformanzvektoren, lokale Simulation, grundlegende Sicht-/Branch-/Erase-Semantik sowie notwendige Werkzeuge für Export und Restore. Die offene Edition muss lokal eingebettet, getestet, gesichert und aus einem dokumentierten Format wiederhergestellt werden können.

Kommerzielle Pakete können optionale Implementierungen/Services anbieten: HSM-/Cloud-KMS- und starker Anchor-Adapter, Replikation/Failover nach eigenem Verteilungs-Gate, zentrale Tenant-Orchestrierung, verwaltete Betriebskonsole/SLA-Telemetrie, zusätzliche Audit-Export- und Signaturadapter, Cold-Tier-/Backup-Integrationen sowie Enterprise-Support. Die Schnittstellen, Wire-/On-Disk-Formate, Verifikation öffentlicher Belege und ein Exportpfad bleiben offen; ein kommerzieller Adapter ist austauschbar und darf keine geheimen Semantiken voraussetzen. Signierte gesetzliche Nachweise und die Erfüllung konkreter Compliance-Pflichten werden nicht allein durch ein Paket behauptet. Lizenzierung, Marken und Preisgestaltung sind Produktentscheidungen außerhalb dieser technischen Spec.

---

## 3. Architektur

### 3.1 Crates und Schichten

```
contextra-storage-ports   Traits und Werttypen, keine Logik
        ▲
contextra-storage         Kern, #![forbid(unsafe_code)], nur std + ports + bewilligte Abhängigkeiten
        ▲ (Kern: nur dev; produktiv über Composition Root, SD-71)
contextra-crypto-ref      Standard-/Referenz-Adapter Cipher/Mac/Kdf/Checksum, In-Memory-KeyProvider, KeyProviderConformance
contextra-keys-file       Referenz-KeyProvider Einzelhost (ab P6)
contextra-keys-cohort     optional: Cohort-Rewrap-Provider für Millionen Cluster (ab P6)
```

Modulschnitt (Abhängigkeiten nur nach unten, Test `layering`):

```
api · engine (sequencer, syncer, memtable, version, views, branches, registry, claims, volatile, guard_window)
maint (flush, compaction, blob_gc, purge, scheduler, scrub) · erase (gate, cluster_table, links, protocol, ledger, oracle[testing])
recovery · catalog · store (run, blob, segment, redo) · keys · codec (varint, tags, checksums, comparator, seal)
env (Fs, Clock, Entropy, Governor, Metrics, Anchor; MemFs, RealFs, FaultFs, Sim) · sync (Ränge, Epoche)
```

`std::fs`, `std::time::SystemTime`, `rand` nur in Adaptern (Lint `disallowed_methods`). Kein `mmap` im Lesepfad. Fremdabhängigkeiten: §3.7. Namensraum: `crates/contextra-storage(-ports)`; `contextra-store` (Altstand) bleibt unverändert bis zur Ablösung (§14).

### 3.2 Ports

| Port | Aufgabe |
|---|---|
| `Fs` | Dateien, `read_at`, geordnetes `read_ranges`, `append_prealloc`, `preallocate`, `sync_data/all/dir`, `rename`, `lock`, `list`, `free_space`, optionale `link/reflink/punch_hole` |
| `Clock`, `Entropy` | injizierbar; Uhr nie für Stamps. Liefert monotone Laufzeit, UTC, durablen High-Water und Skew-Status; Fristablauf fail-closed bei unplausiblen Sprüngen (§3.5) |
| `Anchor` | `read()`, `advance(expect,next)` (CAS, monoton, durabel), `strength() → Strong\|Weak`; Wert `(erase_epoch, ledger_head, catalog_epoch)` |
| `KeyProvider` | §9.4 |
| `Cipher`/`Mac`/`Kdf`/`Checksum` | AEAD (XChaCha20-Poly1305, 24-Byte-Zufallsnonce), Ketten-MAC, HKDF, CRC32C |
| `Authority` (optional) | prüft Signaturen für `LiftPolicy::FourEyes` |
| `Metrics`, `Observer` | nicht blockierend, nie unter Sperren; geschlossener, low-cardinality Metrikkatalog §16 |
| `MemoryBudget` | gemeinsamer RAM-Admission-Port; `try_reserve(class, bytes) -> ResourcePermit`, Drop gibt exakt frei |
| `HostProbe` (optional) | `probe() -> HostReport { swap, core_dumps, memlock_limit, fs_kind, device_ids }`; reiner Adapter-Befund, **nie** vom Kern aufgerufen, wo Korrektheit davon abhängt. `open` und `Receipt` übernehmen ihn als Diagnose/Restexposition (§9.9); Werte `Unknown` gelten als `Declared` (§13.2.1) |
| `ColdTier` (optional) | Auslagerung kalter Dateien, Garantien des Ports gelten |

### 3.3 Mechanismus, nie Politik

Der Kern entscheidet nie, **ob** gelöscht werden darf oder muss, **wer** ein Recht hat, **welche** Frist gilt. Er setzt Aufträge atomar, geordnet und nachweisbar um und verweigert sie nur bei technischem Hindernis oder `Restriction`. Der Kern bildet **keine persistierten Inhalts-Hashes** über Klartext (`INV-PRIV-1`). Ein Derivation Cache der oberen Schicht (§12.6, K-32) ist davon nur als clusterlokaler, im Cluster versiegelter Keyed-MAC unter einem Zufallssecret ausgenommen; der Kern bildet ihn nie selbst.

### 3.4 Threading

Der **Committer** ist eine Pipeline aus **Sequencer** (Rang 1, nur RAM, Flat-Combining-Leader) und **Syncer** (ein Thread, einziger Schreiber des Redo, kein Lock), siehe §6.1. Lesen lock-frei bis auf `Arc`-Klone und Blatt-Sperren. Wartung in Lanes `urgent > flush > compact > blobgc > idle`. Ledger und Anker haben je **genau einen Schreiber** (`urgent`); der Ledger-Writer bündelt wartende Events zu Gruppen. Threads gehören dem Aufrufer oder einem `Spawner`. Keine Async-Runtime im Kern. Provider-, Cipher-, KDF-, Graph- und Guard-Arbeit läuft ausschließlich auf Aufruferthreads und Wartungs-Lanes, nie im Sequencer.

### 3.5 Fehlermodell

| Klasse | Beispiele | Verhalten |
|---|---|---|
| F-1 transient | `EINTR` | Wiederholung mit Backoff |
| F-2 Ressource | `ENOSPC` beim Vorallokieren | **kein Poison**; `DiskFull`; Löschung/Wartung laufen weiter (§6.6) |
| F-3 E/A-hart | `EIO`, Fsync-Fehler | **nie auf demselben Handle wiederholen**; laufender Handle wird `Poisoned` (Löschaufträge bleiben über das Ledger aufnehmbar, §9.5.2). Für Changes, deren Write/Sync begonnen hat, ist der Rückgabestatus `Indeterminate{first_stamp,last_stamp}`; Neustart-Recovery bestimmt den Commitstatus. Idempotenz-Token löst denselben unveränderten Request auf, blindes Retry ohne Token ist verboten |
| F-4 Integrität | CRC/MAC/AEAD-Fehler | kleinste Einheit vergiften (Block → Abschnitt → Datei), `Corrupt{..}`, Rest läuft |
| F-5 Format | `Unsupported` | Öffnen verweigert |
| F-6 Schlüssel | Provider offline / Schlüssel fehlt | `KeyUnavailable{retry_after}` bei Providerausfall; **`KeyLost`** wenn der Schlüssel fehlt, obwohl kein Löschnachweis existiert (Alarm, keine automatische Löschung). Nach dem Gate liefern Lesezugriffe `Erased`; `Shredded` ist ein Clusterzustand und nie ein Lesefehler (§7) |
| F-7 Aufrufer | Guard, `Restricted`, `UnknownCluster`, `BelowFloor`, `Conflicts`, `ReservedSpace`, `SnapshotTooOld` (View-Lease abgelaufen, §11.4), `ErasePending` (Löschabsicht aufgenommen, Gate beim Neustart, §9.5.2), `LedgerUnavailable`, `LedgerSameDevice`, `BackupBeyondHorizon` (§10.4), `ProviderAssuranceInsufficient` (§9.4) | typisiert mit stabilem `hint()` |
| F-8 Logik | Invariante verletzt | Test: Abbruch; Produktion: Poison + Observer; **kein `panic`** |
| F-9 Rollback | Anker höher als Catalog/Ledger | schreibend verweigert (`Err(Rollback)`), nur `ReadOnly` mit Diagnose |

Fehlertexte enthalten keine Schlüssel, Nutzerschlüssel, Körper oder Klartextfragmente.

### 3.6 Lint-Pflicht

Fehler im Kern: `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `unreachable`, `indexing_slicing`, `let _ =` auf Rückgaben von `sync_*`, `rename`, `unlink`. `Debug` für ClusterId, Schlüssel, Körper ist redigiert (`Redacted<T>`).

### 3.7 Ökosystem: finale Abhängigkeiten (Crates) und Ausschlüsse

Dieser Abschnitt ist normativ für jede `Cargo.toml` im Bereich `contextra-storage*`, `contextra-crypto-ref`, `contextra-keys-*`. Stand der Prüfung: Snapshot `f68e2c3` von `tfufuz1/contextra`, Toolchain `1.89.0`. Nicht gebaut, nicht getestet: **Versionen sind Untergrenzen** (`^`), der exakte Pin steht in `Cargo.lock`; vor G-0 wird jede Zeile mit `cargo deny check` und `cargo tree -d` bestätigt. Was im Bestands-`Cargo.lock` steht, ist unten als *Lock* angegeben.

**Auswahlkriterien (in dieser Reihenfolge):** (1) `std` reicht ⇒ kein Crate; (2) Crate ist `#![forbid(unsafe_code)]`-kompatibel für den Kern, Fremd-`unsafe` nur in auditierten Crates der RustCrypto-/crossbeam-Familie; (3) kein Laufzeit-Async im Kern; (4) ein Crate je Aufgabe, **eine** Hauptversion im Workspace; (5) keine Abhängigkeit, die persistierte Inhalts-Hashes (`INV-PRIV-1`) oder Formatdetails vorgibt.

#### 3.7.1 Kern `contextra-storage` und `contextra-storage-ports`

| Aufgabe | Entscheidung | Begründung |
|---|---|---|
| Fehler | `thiserror ^2` (Workspace-Version) | Kern/Ports stehen auf `1`, Workspace auf `2` (E10-03); zwei Hauptversionen verboten. Fehlertexte bleiben redigiert (§3.5). |
| Byte-Puffer | `bytes ^1` | Teilen ohne Kopie für Körper/Blöcke. *Lock:* 1.12.1. |
| Speicherhygiene | `zeroize ^1` (`derive`) | Schlüsselmaterial, `Redacted<T>`. *Lock:* 1.9.0. |
| Memtable-Index | `crossbeam-skiplist ^0.1`, **nur hinter Trait `MemIndex`** | Lock-freie sortierte Map ohne eigenes `unsafe`. Fehlt im Lock und im Workspace (E10-03). Rückfall hinter demselben Trait: sharded `BTreeMap` unter Rang-Sperre. `loom` kann den Skiplist-Pfad nicht prüfen ⇒ Modelltests laufen gegen die Rückfall-Implementierung, der Skiplist-Pfad wird mit `shuttle`/Stress und `miri` gefahren. |
| Guard-Fingerabdruck | `siphasher ^1` | nur Guard-Fingerabdrücke (SD-03), nie persistiert. *Lock:* 1.0.4. |
| Sperren | `std::sync::{Mutex, RwLock, Condvar}` mit eigenem `RankedMutex` (§11.2) | futex-basiert, Rangstack und Poison-Abbildung auf F-8 (`Poisoned`) sind Teil des Vertrags; `parking_lot` bringt kein Poisoning und keinen Rangstack und wird nicht verwendet. |
| Atomics, Threads, Kanäle | `std` | Spawner-Port statt Runtime. |
| Serialisierung | **keine** (handgeschriebener `codec`, §5) | Formate sind eingefroren und byte-genau; `serde`/`bincode`/`flatbuffers` dürfen im Kern und in den Ports nicht auftauchen. |

**Im Kern verboten:** `tokio`/`async-std`/`futures`, `rayon`, `parking_lot`, `scc`, `dashmap`, `memmap2`/`mmap` (§7), `serde*`, `bincode`, `blake3` (SD-03), `rand`/`getrandom` (nur Port `Entropy`), jede Krypto-Implementierung, `anyhow`, `tracing` (Observability nur über `Metrics`/`Observer`-Ports).

#### 3.7.2 Krypto-Adapter `contextra-crypto-ref` (Standard-Adapter, nicht nur Referenz)

Der Kern bleibt krypto-frei (Ports). Es braucht aber ein produktives `Cipher`/`Mac`/`Kdf`/`Checksum`: **`contextra-crypto-ref` ist Referenz- und Standard-Adapter** und wird vom Composition Root (`contextra-storage-api`, Feature `default-crypto`) als normale Abhängigkeit eingebunden, nicht vom Kern.

| Aufgabe | Crate | Anmerkung |
|---|---|---|
| AEAD | `chacha20poly1305 ^0.10` (`XChaCha20Poly1305`, 24-Byte-Nonce) | §5.2; fehlt heute im Lock. |
| Ketten-/Auth-MAC | `hmac ^0.12` + `sha2 ^0.10` | HMAC-SHA256, Tag auf 16 B gekürzt (§5.2). *Lock:* 0.12.1 / 0.10.9. |
| KDF | `hkdf ^0.12` | Rollenableitung außerhalb jedes SealFrame (§12.8 Nr. 24). *Lock:* 0.12.4. |
| Prüfsumme | **`crc32c ^0.6`** | Die Spec verlangt CRC-32**C**; im Lock steht nur `crc32fast` (CRC-32/IEEE) — **nicht austauschbar**. `crc32fast` darf für Formate nicht verwendet werden. |
| Constant-time-Vergleich | `subtle ^2` | Tagvergleich (§5.2). *Lock:* 2.6.1. |
| Zufall | `getrandom ^0.3` (nur im `Entropy`-Adapter) | `rand` entfällt in neuen Crates (siehe 3.7.5). |

**Entfernen:** `blake3` aus `contextra-crypto-ref` (Abhängigkeit der verworfenen `derive_subkey_fast`-Vorstufe, E-68). Kein Selbstbau von AEAD/KDF/MAC (R10 E10-21: `crypto_local`).

#### 3.7.3 Dateisystem-Adapter `contextra-storage-fs` (`RealFs`)

Ziel: `RealFs` ohne `unsafe` im Adapter-Crate, ohne Umweg über `contextra-durable-fs` (R10 §4.1).

| Fs-Port-Operation | Mittel |
|---|---|
| `read_at`, `read_ranges`, Schreiben an Offset | `std::os::unix::fs::FileExt` / `std::os::windows::fs::FileExt` |
| `sync_data`/`sync_all`, `sync_dir` | `std::fs::File` (`sync_data`/`sync_all`; Verzeichnis auf Unix per `File::open` + `sync_all`) |
| `rename`, `list`, `remove_file`, Metadaten | `std::fs` |
| `lock` | `std::fs::File::{lock, try_lock}` (seit Rust 1.89 stabil; **per Compile verifizieren**), kein `fs2`/`fs4`/`fd-lock` |
| `preallocate`, `punch_hole`, `free_space`, macOS `F_FULLFSYNC` | **`rustix ^1`** (`default-features = false`, Features `fs`, `std`) — sichere Wrapper um `fallocate`/`fcntl`/`statvfs`. *Lock:* 1.1.4 (transitiv) |
| Windows-Sonderfälle | `windows-sys` ausschließlich in `contextra-sys` (bestehendes `unsafe`-Crate), `storage-fs` ruft dessen sichere Funktionen |

Wo `rustix` für eine Plattform keine Operation bietet (z. B. macOS-Vorallokation), ist der Rückfall „mit Nullen füllen und `sync_data`“ erlaubt, aber ausdrücklich als `FsCaps`-Merkmal zu melden, nie still. `libc` direkt: nein (`unsafe`). `memmap2`: nein.

#### 3.7.4 Schlüssel-Provider und obere Löschschicht

| Crate | Zusätzliche Abhängigkeiten | Regel |
|---|---|---|
| `contextra-keys-file` | `argon2 ^0.5` (Passphrase → Schlüssel, Pflicht, R10 E10-22), `zeroize`, `getrandom` | Parameter (m, t, p) im Provider-Header; `KeyManager::try_new` ohne Argon2id nicht für neue Daten. *Lock:* argon2 0.5.3. |
| `contextra-keys-cohort` | wie `keys-file` | Rewrap über Ports-AEAD, keine eigene Krypto. |
| `contextra-keys-kms` (optional) | je Anbieter-SDK **hinter Feature**, getrennt vom Kern | SDK-Abhängigkeiten (HTTP/TLS) dürfen nie in den Standard-Build des Kerns wandern. |
| `contextra-erasure` (Orchestrator) | `ed25519-dalek ^2` (`verify_strict` überall), `sha2`, `subtle`, `zeroize`, `serde`+`serde_json` (nur Berichte), `thiserror` | Signaturschlüssel aus 32-Byte-Seed (`SigningKey::from_bytes`), damit **kein `rand`** nötig ist. `aes-gcm-siv` entfällt (R10 §4.2). *Lock:* ed25519-dalek 2.2.0. |
| `contextra-erase-verify` | nur Ports + `sha2` | Orakel liest ausschließlich über Ports. |

Kanonische Receipt-Bytes (§9.7) werden **von Hand** codiert (wie `codec`), nicht über `serde`/`bincode`; JSON ist nur Anzeigeformat und nie signiert.

#### 3.7.5 Test-, Prüf- und Werkzeug-Ökosystem

| Zweck | Werkzeug | Pflicht ab |
|---|---|---|
| Interleavings (kleine Modelle) | `loom ^0.7` (*Lock:* 0.7.2) mit `cfg(loom)` | P1 |
| Interleavings (größere Modelle, Zufallsplanung) | `shuttle ^0.8` | P1 (fehlt im Lock) |
| Eigenschafts-/Modelltests | `proptest ^1` (*Lock:* 1.11.0) | P1 |
| Fuzzing | `cargo-fuzz` + `libfuzzer-sys` + `arbitrary`; echte `fuzz/Cargo.toml` und `fuzz_target!` (E10-06) | G-F |
| UB-Prüfung | `miri` auf `codec`/Siegel/Skiplist-Wrapper | G-F |
| Mutationen | `cargo-mutants` | G-F (H-16) |
| Abdeckung | `cargo-llvm-cov` | G-F (H-15) |
| Lieferkette | `cargo-deny` (Advisories, Lizenzen, Bans, Quellen) | **sofort** |
| Testläufer | `cargo-nextest` | P1 |
| Formale Modelle | TLA+/TLC für M1–M6 (E10-07) | G-F |
| Temp-Dateien (nur Dev) | `tempfile ^3` | P1 |
| Benchmarks | `criterion` nur in `contextra-bench` | P3 |

`cargo-geiger` ist **nicht** Pflichtgate (Pflege unsicher); `#![forbid(unsafe_code)]` plus `deny.toml` ersetzt es.

#### 3.7.6 Obere Schichten: Entscheidungen, die den Kern berühren

- **`tokio`:** nur an den Rändern (`contextra`, `contextra-mcp`, `contextra-py`, `contextra-infer-*`, Bench). `contextra-storage-api` ist synchron; Async-Verbraucher nutzen `spawn_blocking`. Die Übergangsausnahmen in `deny.toml` (`TRANSITION E29`, bis 2027-04-30) werden mit dem Ablauf entfernt, nicht verlängert.
- **`serde`/`bincode`:** `bincode 1.3.3` (Lock) ist nur Altstand; neue Persistenzformate nutzen es nicht. Migration der Konsumenten (`text`, `graph`, `types`, `erasure`) auf handcodierte Formate oder Kern-Segmente; bis dahin Eintrag in `deny.toml` mit Ablaufdatum.
- **`rand`:** im Lock drei Hauptversionen (0.8, 0.9, 0.10). Ziel: **keine** in neuen Crates (`getrandom` genügt), Altcrates auf eine Version.
- **`thiserror`:** Lock hat 1.0.69 und 2.0.20 ⇒ auf 2 vereinheitlichen. **`parking_lot`:** 0.11.2 und 0.12.5 ⇒ auf 0.12 (nur oberhalb des Kerns).
- **`scc`:** nicht im Kern; in `deny.toml` steht ein ignorierter Advisory-Eintrag (`RUSTSEC-2026-0205`). Alle `ignore`-Einträge (auch pyo3, rustls) erhalten Begründung und Ablaufdatum; Inhalt der Advisories ist hier **nicht geprüft**.
- **Parser-Crates** (`lopdf`, `calamine`, `mail-parser`, `zip =0.6.6`, `cfb`, `html5ever`): liegen außerhalb des Kerns, sind aber Angriffsfläche für gespeicherte Daten. Pflicht: Fuzz-Ziele je Parser, Größen-/Tiefenlimits, Ausführung in `contextra-sandbox` für untrusted Eingaben. Die Pin-Gründe von `zip =0.6.6` sind zu dokumentieren und gegen aktuelle Advisories zu prüfen.
- **`contextra-durable-fs`:** wird nicht Teil von `RealFs` (3.7.3).

#### 3.7.7 Durchsetzung

1. `deny.toml` bekommt für die Kern-Crates eine Positivliste (`[bans] deny` alles außer den in 3.7.1–3.7.4 genannten Crates, per `wrappers`); der `layering`-Test (§3.1) prüft zusätzlich `cargo metadata`.
2. `[workspace.dependencies]` führt **jedes** hier genannte Crate genau einmal; Kern-Manifeste verwenden `workspace = true` (behebt E10-03, inkl. `publish`).
3. CI-Job `deps-gate`: `cargo deny check`, `cargo tree -d` (Duplikate nur mit Eintrag), `cargo tree -p contextra-storage --edges normal` gegen die Positivliste.
4. Eine neue Abhängigkeit im Kern braucht einen Eintrag im Entscheidungsregister (§19) mit Kriterien 1–5 oben.

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

## 12. Erweiterungsverträge und Muster für die obere Schicht (RAG, Agenten)

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

### 12.4 Index-Watermarks (Pflichtmuster für Indexbesitzer, aus CairnDB)

Jeder abgeleitete Index führt `applied_position = (instance_id, applied_stamp) = watermark()`. Vergleiche sind nur mit derselben `instance_id` gültig; Instanzwechsel erfordert vollständigen Rebuild oder expliziten, verifizierten Import-Cursor. Eine Abfrage unter View-Stamp `V` derselben Instanz durchsucht **Basisindex ∪ Delta (Stamps `(applied_stamp, V]`)**, validiert jeden Kandidaten gegen die View (`filter_visible`) und kennt Mandanten-/Rechtefilter **vor** der ANN-Traversierung (keine Existenz-Seitenkanäle über Kandidatenlisten). Gate-Zustand/Generation wird zusätzlich vor jeder Payload-Auswertung und vor Rückgabe geprüft; unbekannte oder veraltete Generationen ergeben `IndexLag`/Retry, nie einen Treffer. So entsteht kein Suchloch, solange ein Hintergrundjob hinterherläuft, und kein Index kann Gelöschtes ausliefern (`INV-VIS-1`).

### 12.4.1 Löschdomänen für Vektor-, Text- und Graph-Indizes

Jedes Indexelement, jede Kante und jeder Posting-Eintrag muss einer Quell-`ClusterId`/Generation zugeordnet sein; nicht zuordenbare Strukturen sind für personenbezogene Daten unzulässig. `MixedPerClusterKey` ist nur zulässig, wenn Payload **und** alle zugehörigen Metadaten/Beziehungen ausschließlich zu genau einem Cluster gehören und unter dessen Schlüssel/Epoche liegen. Clusterübergreifende HNSW-Kanten, BM25-Postinglisten, Graph-Kanten/Communities oder geteilte Centroids erfüllen diese Bedingung nicht automatisch und sind `Rebuildable{max_age,max_rewrite_bytes}` zu deklarieren. `Rebuildable` mit gemeinsamem Segment-Key kann physisches Entfernen nach Rewrite attestieren, aber keine Crypto-Erasure für alte Backup-Kopien attestieren. Für Crypto-Erasure müssen auch alle retained Rebuildable-Kopien pro Cluster-/Subject-Key geschützt und diese Schlüssel vernichtet sein; unverschlüsselte oder global geschlüsselte Kopien erfordern Löschung ihrer gesamten Key-Domäne/Backups oder lassen das Systemreceipt `Incomplete`.

`on_gate` aktiviert synchron einen generationengefenceten Tombstone-/Filterzustand vor Query-Rückkehr. Index-Owner muss Gated-Records vor Vektor-Distanzberechnung, Textscore und Graph-Expansion ausschließen, Ergebnisse zusätzlich vor Rückgabe mit `View::filter_visible` prüfen und alle Cache-Kopien invalidieren. Das garantiert sofortige API-Unsichtbarkeit, aber keine physische oder kryptografische Entfernung der geteilten Topologie. `KeysDestroyed` allein schließt gemeinsame Index-Kopien nicht ab. Ein `Rebuildable`-Owner bleibt `Purging`, bis betroffene Payloads, Kanten, Posting-Einträge, Routing-Metadaten und persistierte/gespiegelte Kopien innerhalb seiner deklarierten Rewrite-Obergrenze entfernt und durch `verify_absent` attestiert sind. Über Obergrenze/Deadline ⇒ `Blocked/Incomplete`, niemals `Done`.

V1-Referenzarchitektur: Vektoren und kleine Ranking-Metadaten liegen clusterlokal versiegelt für subject-/fallgebundene Queries; tenantweite ANN/FTS/Graph-Indizes sind partitionierte, bounded `Rebuildable`-Derivate mit durablem Delta und Gate-Filter. Alternativ kann die obere Schicht vollständig clusterlokale Mikroindizes verwenden. Ein globaler HNSW-/CSR-Neubau ist kein O(Cluster)-Erase-Versprechen. Für token-budgetierte Suche bleiben `header`/`weights`/`page` die Kernprimitive; Modellwahl, Embedding, BM25, ANN und Ranking sind obere, messbare Services.

### 12.4.2 Ende-zu-Ende-Vertrag für Contextra-RAG

Der Storage-Kern ist der transaktionale Datensatz- und Lineage-Anker, nicht der ANN-/BM25-/Graph-Algorithmus. Ein fachlicher Ingest-`Change` darf Dokument-/Chunk-Records, quantisierte Vektoren, Text-/Term-Quelldaten, Metadaten und Graph-Deltas atomar schreiben, wenn sie derselben Cluster-Löschdomäne angehören und das Change-Limit einhalten. Der Kern berechnet weder Embeddings noch BM25-Postings, HNSW-Nachbarschaften, CSR-Kompression oder Communities. Solche Repräsentationen sind abgeleitete Indexgenerationen und werden über `SegmentOwner`/Change-Feed betrieben.

Der verpflichtende Ablauf lautet: (1) validiere DataClass, Tenant-/Zugriffskontext in der oberen Schicht, Budget und Lineage; (2) committe kanonische Quelldaten durable; (3) konsumieren Extensions mindestens-einmal und idempotent nur bis `durable_stamp`; (4) veröffentliche Indexgeneration samt `(instance_id, applied_stamp)` erst nach vollständig applizierten Deltas; (5) Query bindet eine `view_durable()`-Sicht, begrenzt sie durch Watermark/Delta, wendet Tenant-/Rechtefilter vor Indexexploration an und prüft jeden Kandidaten über Gate + `filter_visible` vor Scoring/Traversal und erneut vor Ausgabe; (6) Lücken oder zu alte Generationen liefern `IndexLag`/Retry bzw. explizite Fallback-Suche, nie stillschweigend unvollständige Treffer.

Quelldaten und externe Indexgeneration sind nicht gemeinsam atomar. Der Vertrag stellt Durability, Wiederaufbau, Idempotenz, Sichtbarkeit und Löschbeleg her, nicht Null-Lag. Ein Index darf erst als aktuell beworben werden, wenn sein Watermark die angefragte durable Position abdeckt oder die Delta-/Fallback-Abfrage die Lücke schließt. Gate invalidiert Treffer synchron auf dem Querypfad; physische Bereinigung und Krypto-Erasure benötigen weiterhin Owner-Atteste und Backup-Scope. Ein Owner darf nicht zugleich `NoPersonalData` deklarieren und personenbezogene Lineage verarbeiten.

Alle Komponenten beziehen RAM-, Scratch-, I/O- und Lag-Grenzen aus dem gemeinsamen `MemoryBudget`/Governor. Unter Druck wird Indexaufbau gedrosselt oder abgewiesen, bevor unbeschränkte Allokation stattfindet; die kanonische Storage-Schreibbarkeit und Gate-Priorität bleiben erhalten. Tenant-/Cluster-IDs, Querytext, Schlüssel, Dokumentinhalte und Embeddings sind niemals Metric-Labels oder Standard-Observer-Payloads.

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

Planungsannahme für 3–4 erfahrene Rust-Entwickler plus 1 Test-/Simulationsingenieur; Dauern sind grobe Schätzungen, keine Zusage.

| Phase | Inhalt | Gate | ca. |
|---|---|---|---|
| **P0 Fundament** | `ports`, `codec`, `env` (MemFs, RealFs, FaultFs, Sim), `sync`, Anker (Mem, Datei), `crypto-ref` mit In-Memory-Provider und Konformanzsuite, Orakel-Skelett, CI, Lint-Gerüst, Referenzcodierer, Workspace-Einbindung, neue Typen (`KeyProvider`-Angleichung, `HostProbe`, `BlockPruner`, `MeasureKind`, neue Fehlerklassen; WP-01.5) | **G-0:** diese Spec normativ (zwei Reviewer), CI grün | 3 Wo. |
| **P1 Walking Skeleton (vertikaler Durchstich)** | ein Space, `apply`, Redo + Memtable, `view`, Cluster-Register, Gate, `destroy_many`, Ledger (ohne Anker), Recovery aus Redo; **Formate provisorisch** | Crash-Sweep über alle Fs-Operationen grün; O-Files findet nach `Done` nichts, obwohl Daten nur in Redo/Memtable lagen | 4 Wo. |
| **P2 Runs und Catalog** | Flush, ClusterSection, Staging, Catalog + `CURRENT`, Purge, Plain-Barriere, vollständige Recovery, Absturzmatrix | O-Files für Daten in Runs grün; Modelltest grün; M1, M2 (ohne Branches) geprüft; **Spike-Messungen als Freeze-Voraussetzung:** H-21 (Tiefe 1…8 mit branch-bewusstem Filter), H-11 (MiniSection), H-25 (Directory-Sparse-Index) und H-26 (Spill-Policy) | 5 Wo. |
| **G-F Formatfreeze** | alle Formate (§5.9) inkl. reserviertem `branch`-Feld, goldene Vektoren, Rohkopie-Test, ≥ 8 Fuzz-Ziele | Freeze-Review, Referenzcodierer vollständig | — |
| **P3 Schreibpfad produktiv** | Gruppen-Commit, Reservierungen, Guards zweistufig, Volatile, Notbetrieb `DiskFull`, Idempotenz, Backpressure | **G-W:** Absturzmatrix, Linearisierbarkeit in Sim, H-1, H-18, H-24 | 4 Wo. |
| **P4 Lesepfad** | Blobs, Cache, Views (Revoke, Lease/`SnapshotTooOld`), `page` mit Gewichten, Block-Skipping, `ResumeToken`, `filter_visible` | **G-R:** Modellgleichheit, Paging-Properties, H-2…H-4, H-25 | 4 Wo. |
| **P5 Wartung** | Compaction je Wartungsgruppe, Claims, Blob-GC, Governor, Rotation, Consumer-Feed aus Redo | **G-M:** M3 geprüft, Sim ohne Invariantenbruch, H-5, H-19, H-26, H-27, H-28 | 5 Wo. |
| **P6 Löschung komplett** | Links, Restrictions (`LiftPolicy`), Abwurf, Space-Drop, `range`, Ledger/Anker, Checkpoint/Restore, `keys-file`, `keys-cohort`, `SegmentOwner`/`ExtensionBlock`, Receipt + `verify_receipt` (mit `MeasureKind` und Statuscaps), Tomb-Epochen/Backup-Horizont (§10.4), Löschauftrag auf `Poisoned` (§9.5.2), volle Orakel | **G-E:** M2/M4/M5/M6 geprüft, Orakel 0 Treffer, INV-ERA-11 (Lokalität), H-6…H-11, H-17, Konformanzsuite grün | 6 Wo. |
| **P7 Versionierung** | `BRANCHES`-Feature-Bit, `SYS_BRANCH`, Fork/Drop/TTL, Branch-Lesen, Diff, Merge, `Eraser::branch`, Fork-Floor | **G-V:** Branch-Properties vs. Modell, M2/M3 mit Branches, H-20…H-22, Orakel mit Branch-Canaries | 5 Wo. |
| **P8 Härtung** | Soak 72 h, Dauer-Fuzz, Platte voll, Fehlerinjektion, Fassade, Abdeckung, Krypto-Review, Messbericht, Bedrohungsmodell-Review | **G-P:** alle INV getestet (100 %), H-15/H-16, Messbericht §17, keine offenen Korruptionsbefunde | 5 Wo. |

**G-CUTOVER (P8+, SP-CUTOVER 24):** separates Produktfreigabe-Gate nach G-P. Erst wenn sämtliche produktiven `contextra-store`-Verbraucher, Datenräume und Erasure-Witnesses migriert, Restore-/Rollback-Rehearsal einschließlich externem Fencing des alten Instanzbesitzers und RAG-Differentialmessung bestanden sowie gemeinsames Speicherbudget belegt sind, wird `contextra-storage` alleiniger Produktions-Schreibkern. Nach den ersten Writes in `contextra-storage` ist Rückkehr nur per geprüftem Forward-Export oder Restore möglich; kein Dual-Write. Bis dahin gelten die Leistungs- und Löschvorteile von `contextra-storage` als unbestätigte Ziele.

Keine Phase beginnt vor dem Gate der vorherigen; Formatänderungen nach G-F nur per Migration. In jeder Phase wird der Crash-Sweep um die neuen Fs-Operationen **erweitert, nie nachgezogen**. Kontext-Engine, Vektor-/Volltext-/Graph-Indizes, Cache und MCP-Bridge dürfen parallel auf Basis freigegebener Verträge entwickelt und in P4–P8 integriert werden; sie müssen die gemeinsamen Durability-, Watermark-, MemoryBudget- und Erasure-Gates bestehen. G-P bewertet den vollständigen Storage-plus-RAG-Stack, und erst G-CUTOVER macht `contextra-storage` zum alleinigen Produktions-Schreibkern.

**Sofortmaßnahmen vor P1:** (1) `contextra-storage` in den Workspace aufnehmen, fehlende Crates `contextra-storage-ports`, `contextra-crypto-ref` anlegen; (2) Bestand (R2-Entwurf ohne Cluster) modulweise auf diese Spec umstellen (`redo`, `run`, `blob`, `catalog`, `committer` formatbetroffen); (3) `unwrap/expect/panic` aus Produktionscode entfernen (Altstand 1260 + 16 Treffer, R7-Anhang B); (4) `deny.toml`, CI, Toolchain-Pin (1.89.0); (5) fehlende Verweisdateien anlegen oder Verweise entfernen; (6) Altstand `contextra-store` per Tag `legacy-store-final` archivieren, **kein Code wird übernommen**.

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

---

## 16. Konfiguration (Auszug)

`block_target` 16 KiB · `memtable_total_bytes` ≤ 256 MiB · `redo_segment_bytes` 64 MiB, `redo_mark_bytes` 8 MiB, `redo_max_age` 1 h · `run_target_bytes` 64 MiB · `blob_max_bytes` 64 MiB · `max_pack_groups` 4096 · `pure_threshold` 8 MiB · `max_gate_clusters` 4096 · `max_open_files` 1024 (Größe des FD-Caches, kein Dateilimit, §4.3.1) · `maint_io_fraction` 0,5 · `purge_batch_min` 16 · `purge_rewrite_concurrency` 1 · `purge_scratch_reserve_bytes` ≥128 MiB (zwei Slots, keine normale Allokation) · **`purge_deadline` kein Standard (Pflichtparameter)** · Reservierungen 2^20 / 2^16 · `max_links` 8 · `max_hold_ttl` / `idem_ttl` 24 h · `anchor` Required bei Encrypted/Authenticated, `anchor_max_age` 1 s · `guard_window_entries` ≤ 1 Mio. und durch MemoryBudget begrenzt · `max_lag_bytes` 1 GiB · `erase_reserve_bytes` ≥2 GiB (inklusive Scratch 128 MiB, Consumer-Retention 1 GiB, Emergency-Reserve und FS-Metadaten-Headroom) · `max_clock_skew` 5 min default, strenger nur explizit · `key_grace` 10 min · `orphan_key_grace` 24 h · `revoke_grace` 5 s · `max_branches_per_cluster` 1024, `max_branch_depth` 4 (Default, hart 8) · `LiftPolicy` OwnerOrAdmin.

**Gemeinsames RAM-Budget:** `memory_budget_bytes` defaultet auf `min(1 GiB, max(256 MiB, 20 % des effektiven cgroup-/Job-Limits))`; kann der Adapter kein verlässliches Limit liefern, ist der Wert beim Öffnen Pflicht. Unterbudgets sind Obergrenzen am selben Budget: 25 % Memtables, 15 % Cluster-/Branch-Metadaten, 2 % Guard-Fenster, 20 % Block-Cache, 1 % Key-Cache, 20 % Extension-/Indexdelta und 10 % In-flight-Changes/Rewrite-Buffer; 7 % bleiben als Allocator-/Runtime-Headroom unangetastet. Jede Allokation reserviert atomar beim `MemoryBudget` und gibt per RAII frei. `max_clusters`, Branch-Limits und Extension-Admission werden zusätzlich durch tatsächliche Metadatenbytes begrenzt; `max_clusters=16 Mio.` ist eine Obergrenze, keine zugesagte Resident-Kapazität: `resident_clusters_max = cluster_meta_budget / 40 B` (§4.3.1). Für 16 Mio. Cluster sind ≈ 610 MiB Metadatenbudget nötig (≈ 4,0 GiB `memory_budget_bytes` bei 15 % Anteil); `register` weist darüber hinaus vor jeder Allokation mit `Limit{ClusterMetaBudget}` ab. Ein Extension-Owner ohne Budget-Token darf keine Indexarbeit oder Cache-Allokation starten.
**View-, Ledger- und Profil-Schlüssel:** `view_lease` 60 s, `view_lease_max` 10 min (§11.4) · `tomb_epoch_rotation` (Default aus), `backup_horizon` (Pflicht, sobald Retire aktiv ist; sonst kein Retire), `max_active_tomb_epochs` 4 (§10.4) · `OpenProfile::{Development, Production}` (Default `Production` bei persistentem Open; `Development` schaltet die Prüfungen `ProviderAssuranceInsufficient` und `LedgerSameDevice` auf Warnung), `acknowledged_weak_keys`, `ledger_same_device_ack` (beide erscheinen im Receipt als Restexposition).
**Pipeline-, Platzierungs- und Admission-Schlüssel:** `commit_pipeline_depth` 2 · `max_group_changes` 1024 · `max_group_bytes` 8 MiB · `guard_retry_max` 3 · `policy_retry_max` 3 · `publish_timeout` 1 s · `reserve_stamps_urgent` 2^10 · `reserve_ids_urgent` 2^8 · `max_link_depth` 8 · `dir_page_bytes` 4096 · `mini_section_max_bytes` 4096 · `spill_min_bytes` 8 MiB · `spill_ratio` 4 · `spill_max_age` 6 h · `max_staging_runs` 16 · `cluster_local_min_bytes` = `pure_threshold` · `max_cluster_local_clusters` 1024 (≤ 65 536) · `max_live_files` 262 144 · `pure_demote_ratio` 0,25 · `fd_reserve` 256 · `file_shard_count` 1024 · `purge_window_fraction` 0,3 · `sealed_purge_escalate` 0,7/0,9 · `cluster_meta_budget` 15 % von `memory_budget_bytes`.
**Metrikvertrag (Pflicht, Z-14/Z-17):** Der feste Katalog in `metrics.rs` definiert Name, Typ, Einheit, begrenzte Labels und Bedeutung. Pflichtfamilien: Commits (Anzahl, Gruppengröße, Latenz nach `durability`/`result`), Erase-Stufen (Gate- und KeyDestroy-Latenz, getrennte Zähler nach Status), Durability (sichtbar-durable Stamp-Lag, Fsync-Latenz nach festem `file_role`), Ressourcen (Bytes/Limit nach festem `resource_class`, Memtable/Cache-Belegung und Trefferquote, Reserve frei/benutzt, Scratch belegt), Wartung (Flush/Compaction/Purge-Bytes und Laufzeit nach `phase`), Erase (offene Tickets, ältestes Alter, Fristverletzungen, Dauer und Ergebnis je `stage`, Gate-Latenz, KeyDestroy-Latenz, Purge-Bytes, Anchor-Lag), Extensions (Feed-/Index-Watermark-Lag und blockierte Owner nach festem `owner_kind`), Guards (Stufe-A-Konflikte, `GuardRetry`-Rate, Fensterfuß-Treffer), Branches (Anzahl, Tiefe, ältester Pin/History-Floor) sowie Fehler/Poison-Zustand nach geschlossenem `ErrorClass`; zusätzlich abgelaufene View-Leases, offene Poisoned-Löschabsichten, Anzahl/Alter aktiver Tomb-Epochen (nur Zähler/Gauges, keine IDs). Jede Zeitreihe hat definierte Einheit und monotone Counter-Semantik; Histogramm-Buckets sind pro Release versioniert.

Labelwerte stammen ausschließlich aus kleinen geschlossenen Enums (`durability`, `result`, `file_role`, `resource_class`, `phase`, `stage`, `owner_kind`, `ErrorClass`). Keine IDs, Namen, Tenant-/Cluster-/Space-Kennung, Schlüssel, Request-Token, Querytexte, Dokumentinhalte oder Embeddings als Label oder Standard-Event. `Metrics`/`Observer` sind best-effort, nicht blockierend und werden außerhalb aller Kernlocks aufgerufen; Telemetrie-Ausfall ändert keine Storage-Semantik. Der Inspector ist administrative Diagnose, standardmäßig nur aggregierte/opaque Werte, benötigt Capability und darf keine Inhalte entsiegeln. Metrik-Katalog-Review und Kardinalitäts-/Leakage-Test sind Release-Gates.

---

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

**Risiken:** (1) Branch-Pinning treibt Speicher und Compaction-Kosten bei vielen Langzeit-Branches (Mitigation: `ttl`, Limits, `floor`-Metrik). (2) Branch-Reads über Elternketten (H-21) können bei tiefen Ketten teuer werden (Mitigation: branch-bewusster Filter und `BranchTable` §7.1, Default-Tiefe 4, Spike vor G-F). (3) Kosten der Plain-Barriere bei häufigen Plain-Löschungen (Mitigation: Bündelung; Personal-Daten sind per Konstruktion Sealed). (4) Verzeichnis Person → Wurzelcluster bleibt nur physisch löschbar (§12.7). (5) Formatfreeze nach P2 verlangt, dass P1/P2 echte Probeläufe sind, nicht Papier. (6) Scope-Creep in obere Schichten; Mitigation: Gates strikt. (7) Komplexität der Commit-Pipeline (Sequencer/Syncer/Publikation, `Retiring`); Mitigation: M1-Modell, loom, Simulation RACE-18…RACE-20. (8) RAM-Bedarf ab ≈ 4 Mio. Clustern (≈ 40 B je Cluster plus Hot-Records); Mitigation: Admission §4.3.1, Konvention „Session ≠ Cluster“. (9) Der Zeitplan in §14 ist für P2, P6 und P7 (Catalog/Purge/TLA+ bzw. Branches) wahrscheinlich um den Faktor 1,5–2 zu knapp; Mitigation: Gate-Kriterien bleiben, Termine werden nach P1/P2 neu geschätzt.

**Externe Vorbedingungen (keine Spec-Lücken):** rechtliche Prüfung der Fristen und der Aussage „kryptografisch gelöscht“ für den Einsatzfall; zweiter Reviewer für G-0; Beschaffung eines `Strong`-Ankers und eines HSM/KMS-Providers für den Produktivbetrieb (der Kern ist ohne sie vollständig, Restrisiken sind benannt).

---

## 19. Entscheidungsregister (neu nummeriert)

| ID | Entscheidung |
|---|---|
| SD-01 | Catalog als einzige Wahrheit, ein Edit = ein Durabilitätspunkt |
| SD-02 | Clusterreine Blöcke, ein Abschnitt je Cluster und Datei, abschnittsrelative Offsets |
| SD-03 | Bewilligte Abhängigkeiten: normativ **§3.7** (Positivliste je Crate, Ausschlüsse, Werkzeuge). Kern: `thiserror 2`, `bytes`, `zeroize`, `crossbeam-skiplist` (hinter `MemIndex`), `siphasher`; `std::sync` statt `parking_lot`. Krypto im Kern nur über Ports; Standard-Adapter `contextra-crypto-ref`: `chacha20poly1305`, `hmac`, `sha2`, `hkdf`, `crc32c`, `subtle`. `blake3` ist keine Kern- und keine Referenzabhängigkeit |
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
| SD-68 | Ökosystem-Positivliste §3.7; Kern ohne `tokio`, `parking_lot`, `scc`, `memmap2`, `serde*`, `bincode`, `rand`, `blake3` |
| SD-69 | Prüfsumme der Formate ist `crc32c` (CRC-32C); `crc32fast` (IEEE) ist für Formate verboten |
| SD-70 | `RealFs` = `std` + `rustix` (`fs`) hinter `contextra-storage-fs`; kein `contextra-durable-fs`, kein `libc`, kein `mmap` |
| SD-71 | `contextra-crypto-ref` ist Standard-Adapter (Composition Root, Feature `default-crypto`), nicht nur `dev-dependency` (Berichtigung zu SPEC-R10 §4.1) |
| SD-72 | Signaturen/Receipts: `ed25519-dalek` mit `verify_strict`, Schlüssel aus Seed; kanonische Receipt-Bytes handcodiert |
| SD-73 | Eine Hauptversion je Crate im Workspace (`thiserror 2`, `parking_lot 0.12`, höchstens eine `rand`-Linie, Ziel keine); `deny.toml`-Ausnahmen nur mit Begründung und Ablaufdatum |

---

## Anhang A — Referenzvektoren (Pflicht vor G-F)

Je Format mindestens: leeres Objekt, ein Eintrag, Eintrag mit versiegeltem Körper (Cluster- und Space-Quelle), Cluster-Abschnitt mit zwei Blöcken, Packed-Run mit drei Clustern, Staging-Run, `MiniSection`, Abschnitt mit `BranchTable` (drei Branches, einer ohne Einträge), Tombstone, Bereichsmarker, Gate-Frame mit `policy_epoch`, Control-Frame `Link`, Blob mit Loch, Catalog-Edit-Kette mit Reservierung, Gruppen-Commit und `anchor_mark`, Ledger-Kette (`tomb_id`, zwei Tomb-Epochen, ein Retire), Redo-Gruppe mit Change-/Gate-/Control-/Branch-Frame, abgeschnittener und beschädigter Frame, nicht-kanonisches `varint`, **Rohkopie-Test** (Abschnitt aus Run A in Run B, Blöcke byte-identisch, Entsiegeln gelingt). Erzeugung durch `tools/reference/generator.py` (unabhängig von Rust), Vergleich im Test `golden`. **Stand: nicht vorhanden.**

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

## Anhang B.4 — Ergänzung R9.1 → R9.1+Ö (Ökosystem, §3.7)

Kein Format-, Gate- oder Phasenänderung. Ergänzt wird die bisher fehlende Entscheidung über Fremdabhängigkeiten.

| ID | Befund | Lösung |
|---|---|---|
| Ö-01 | SD-03 nannte Crates ohne Auswahlkriterien; `crossbeam-skiplist`, `chacha20poly1305`, `crc32c`, `shuttle` fehlen im Bestands-Lock | §3.7 mit Kriterien, Positivliste, Ausschlüssen |
| Ö-02 | Spec verlangt CRC32C, Workspace führt nur `crc32fast` (IEEE) | `crc32c` verbindlich (SD-69) |
| Ö-03 | `contextra-crypto-ref` nur `dev`, produktiv aber nötig | Standard-Adapter (SD-71) |
| Ö-04 | `RealFs` ohne Plan für Prealloc/Hole-Punch/Dir-Sync/Lock | `std` + `rustix` (SD-70) |
| Ö-05 | `parking_lot`/`scc`/`rand`/`thiserror` in mehreren Hauptversionen oder mit Advisory-Ausnahmen | §3.7.6, SD-73 |
| Ö-06 | `blake3` in `crypto-ref` widerspricht E-68/SD-03 | entfernen |

## Anhang C — Verworfene Vorschläge

Zeitlanes `lane_min/max` und kumulative Gewichte im SubIndex (K-29, K-30) · mehrere parallele Redo-Logs/Striping zur Anhebung der Fsync-Rate (zerbräche Stamp-Kette und Gate-Ordnung; Skalierung bleibt Instanz-Sharding, K-18) · Branch-„Flatten“ durch Datenkopie (verletzte O(1)-Fork und Löschlokalität) · Retry-freie Langsampfad-Guards im Sequencer (Plattenzugriff unter Rang 1) · persistierter Token-Index im Sequencer (K-38) · Prolly-Tree/Merkle-DAG/Inhaltsadressierung im Kern (K-07, K-08) · CAS-Superblock ohne Log (K-10) · `mmap`/`io_uring`/GPUDirect im Kern (K-11) · Token-IDs, KV-Cache-Tensoren, feste Seitenstruktur (K-13) · gelernte Cross-Objekt-Affinität als Layout (K-17) · Slab-Sub-Allokator (K-20) · Segmentschnitt je Gate · Aufrufer-vergebene ClusterId · `eu_tag` im Klartext-Header · Poison bei ENOSPC · `rekey_clone` · Indizes (ANN, BM25, Graph) im Kern · MCP-Server, Tokenizer, Embedding-Modelle im Kern · benutzerdefinierte Merge-Operatoren im Kern.

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
```

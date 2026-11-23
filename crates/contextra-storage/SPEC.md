# SPEC.md — `contextra-storage` (Speicherkern), Revision R8 (Konsolidierung)

| | |
|---|---|
| Status | **Entwurf zur Freigabe als Gate G-0.** Ersetzt SPEC R7 und konsolidiert fünf Quellen: R7 (`SPEC(18).md`), Lethe-Architekturentwurf, Lethe-Engine-Spec, Mnemos-Konzept, „Optimal-LLM-Datenbank“ (Engram, AetherStore ×2, CairnDB). |
| Gegenstand | Eingebetteter, synchroner, crash-sicherer Speicherkern für LLM-/Agenten-Workloads mit atomaren Mehr-Space-Änderungen, MVCC-Sichten, **versionskontrollartigen Branches und Merges (clusterlokal)** und **vollständiger, nachweisbarer Löschung personenbezogener Daten** (logisch, kryptografisch, physisch). |
| Verifikation | Der in `crates/contextra-storage/PHASE-STATUS.md` dokumentierte Teilcheck umfasst Storage-/Crypto-Compile sowie ausgewählte Config-, Catalog-/Redo-Crash-Sweeps und Strukturtests; vollständige Release-Gates sind offen. Keine Leistungszahl ist als gemessen anzusehen. R7-Messungen wurden nicht neu erhoben. Formate brauchen vor **G-F** Referenzvektoren aus einem unabhängigen Codierer. |
| Rechtlicher Vorbehalt | Der Kern liefert technische Eigenschaften, keine Rechtsaussage zu Art. 17 DSGVO, Fristen oder Ausnahmen. „Gelöscht“ ist immer an eine Stufe aus §9.2 gebunden. |
| Sprache | Prosa deutsch, Bezeichner englisch. |

---

## 0. Kurzfassung

**Ein Cluster ist die Einheit, in der Daten leben, versioniert werden und sterben.** Alles zu einer Person, einem Fall oder einem Arbeitskontext liegt in einem Cluster: Nutzdaten, Verlauf, Branches, abgeleitete Strukturen. Der Cluster ist zugleich **Löschdomäne, Schlüsseldomäne, Versionsdomäne und Platzierungseinheit**. Daraus folgt die ganze Architektur:

| Eigenschaft | Wie sie entsteht |
|---|---|
| Atomare Änderung über Spaces | ein Redo-Frame je `Change`, ein Gruppen-Fsync, ein Schreiber (Committer) |
| Git-artige Versionierung | Stamps = Commit-Reihenfolge; `view_at`; **Branches je Cluster** (O(1)-Fork, Diff, Merge mit Konfliktliste); kein Merkle-Baum, keine Inhaltsadressierung |
| Löschung in drei geordneten Schritten | **Gate** (Millisekunden, Cluster ist ab Stamp `S` für alle Sichten `Erased`) < **Schlüssel** (Clusterschlüssel vernichten, alles des Clusters unlesbar) < **Purge** (Bytes aus lebenden Dateien entfernen, Rohkopie ohne Entschlüsseln) |
| Gate/KeyDestroy ∝ Zielcluster; physischer Purge ∝ betroffene Dateien/Extents | clusterreine Abschnitte + `EraseLocality`; `ClusterLocal` vermeidet fremde Survivor-Rewrites, `PackedBounded` tauscht I/O gegen geringeren Platzverbrauch |
| Rollback-/Wiederkehr-Schutz | hashverkettetes Ledger (nur `tomb_id`, keine ClusterId) + monotoner Anker |
| RAG-Anbindung | fünf Verträge: `SegmentOwner`, `ExtensionBlock`, `Consumer`/Change-Feed, Kopf+Gewichte+`page`, `filter_visible` |

**Nicht im Kern:** Tokenizer, Embeddings, ANN, Volltext, Graph-Suche, Trust-Politik, Zoom-Stufen, Rechtsgrundlagen, Fristen, Mandanten- und Rechteprüfung, Replikation. Sie setzen auf den Verträgen in §12 auf.

**Was der Kern nicht löschen kann:** Prozessspeicher, Swap, Core-Dateien, Dateisystem-Journale, Flash-Reste, externe Backups mit Schlüsselkopien, Daten außerhalb des Kerns (Prompt-Caches, Provider-Logs). Jedes Receipt nennt diese Restexposition (§9.9).

---

## 1. Konfliktregister: Entscheidungen der Konsolidierung

Die Quellen widersprechen sich an 28 Stellen. Jede Entscheidung ist final; die Spalte „Wirkung“ nennt den Paragrafen, der sie umsetzt.

| ID | Konflikt | Entscheidung | Begründung | Wirkung |
|---|---|---|---|---|
| K-01 | Löscheinheit heißt „Cluster“ (R7, Lethe-Entwurf) bzw. „EU“ (Lethe-Spec) | **Cluster** | ein Begriff, R7 ist Referenz | §4 |
| K-02 | ClusterId: Aufrufer-ID / HMAC-`eu_tag` (Lethe-Spec, R6) vs. vom Kern erzeugt (R7) | **Kern erzeugt 128-Bit-Zufall**, kein `eu_tag`, kein `tenant_tag_key` | keine Wiederverwendung, nicht ableitbar, kein zusätzliches Schlüsselmaterial im Klartext-Pfad | §4.3 |
| K-03 | Ledger speichert ClusterId (R7) vs. `tomb_id = HMAC(ledger_key, id)` (Lethe-Entwurf) | **`tomb_id`** | Ledger überlebt Löschung und Backups; er darf keine Pseudonyme enthalten, die auf Dateien zeigen | §9.7 |
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
| K-21 | Guard-Prüfung im Committer (kalte E/A unter Sperre) | **Zweistufig** (R7) | keine Plattenlesezugriffe unter Rang 1 | §6.3 |
| K-22 | Ledger im Datenvolume (R7) vs. eigene Failure Domain (Lethe) | **`ledger_dir` konfigurierbar**, Produktivempfehlung: getrenntes Volume; Anker bindet den Kopf | beide Quellen vereint | §9.7, §10.3 |
| K-23 | Stufen G0/G1/G2 (Lethe-Entwurf), `Logical/Physical/Scrubbed/Cryptographic` (R7), Ticket-Stufen (Lethe-Spec) | **Zwei Achsen**: `EraseLevel` = gewünschter Endzustand; `Stage` = Fortschritt (`Gated`=G0, `KeysDestroyed`=G1, `Purged`/`Done`=G2) | keine Doppeldeutigkeit | §9.2 |
| K-24 | Zuordnung Person → Cluster | **Normatives Muster in §12.7**: Sealed-Wurzel je Person plus registrierter `IdentityDirectoryOwner` mit tenant-scoped HMAC-Indexschlüssel und per-Subject-Entry-DEK; Löschung widerruft Cluster und Directory-Eintrag | Verhindert gemeinsame Shard-DEKs und Überlöschung. HMAC bleibt pseudonymisierend, nicht anonymisierend; Schlüsselkompromittierung wird als Restrisiko geführt. Entry-DEK-Zerstörung und Owner-Attest sind Teil der Löschkette | §12.7 |
| K-25 | Daten mehrerer Betroffener | **Gemeinsamer Fallcluster + Links** (R7); keine mehrclustrigen Einträge | Purge ohne Querverweise | §9.8 |
| K-26 | Bitemporalität (Mnemos, Cairn: vier Zeitachsen) | **Stamp = Transaktionszeit im Kern**; Gültigkeits-/Beobachtungs-/Quellzeit im Kopf der oberen Schicht (Konvention) | Kern ohne Zeitlogik | §12.6 |
| K-27 | Siegelformen (Redo je Segment, Block mit `file_id`) | **Eine Siegelform ohne Datei-/Positionsbezug** (R7) | Rohkopie in jedem Modus gültig | §5.2 |
| K-28 | R7-Redaktionsfehler: doppelte `SD-38/39/40`, Rang 5 „entfällt“ | **Neu nummeriert** (§19), Ränge 1–8 durchgehend | Konsistenz | §11.2, §19 |

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

---

## 2. Leitentscheidungen, Ziele, Nicht-Ziele, Bedrohungsmodell

### 2.1 Leitentscheidungen

| # | Entscheidung |
|---|---|
| L1 | **Cluster** (`ClusterId`, 16 Byte, vom Kern zufällig erzeugt) = Löschdomäne = Schlüsseldomäne = Versionsdomäne = Platzierungseinheit. |
| L2 | Ordnung geclusterter Spaces: `(cluster, branch, user_key, tag↓)`. Blöcke clusterrein, je Datei ein zusammenhängender Abschnitt je Cluster mit **abschnittsrelativen** Offsets. |
| L3 | Cluster sind `Sealed` (eigener Schlüssel) oder `Plain`. **Alles** eines Sealed-Clusters ist unter seinem Schlüssel versiegelt (`INV-SEAL-1`). |
| L4 | Dateien sind Pure, Packed oder Staging (nur L0). |
| L5 | **Ein Schreiber** (Committer) für alle stamp-relevanten Zustände einschließlich Gate, Control, Branch-Ops. Dateien unveränderlich, `Version` per `Arc`. |
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

### 2.2 Ziele

Z-1 Atomarität über Spaces und Branches · Z-2 Durability (sichtbar nur nach Fsync) · Z-3 Snapshot-Isolation ohne Lesesperren · Z-4 Gate/KeyDestroy unabhängig von Storegröße, Purge bounded durch betroffene Datei-/Extension-Partitionen · Z-5 Ablauf einer Packgruppe als O(Dateien) · Z-6 Nachweisbarkeit (Receipt, `verify`, Orakel, Ledger, Anker) · Z-7 Kopf ohne Körper-I/O, Budgetseiten ohne Body-Block-Lesen · Z-8 Race-Freiheit durch Konstruktion, belegt durch Modell, Simulation, Loom · Z-9 begrenzte Ressourcen · Z-10 stabile Verträge nach oben · Z-11 flüchtige Arbeitsbereiche · Z-12 Idempotenz für Agenten-Retries · Z-13 **Gate/KeyDestroy bei vollem Datenträger; physischer Purge nutzt vorgehaltene Scratch-Reserve** · Z-14 überwachbare Löschfristen · **Z-15 O(1)-Fork, indexierter Diff und konfliktgeprüfter Merge je Cluster.**

### 2.3 Nicht-Ziele

NZ-1 Verteilung/Konsens in v1 · NZ-2 eigene Kryptografie · NZ-3 Schemas, Sekundärindizes, SSI · NZ-4 benutzerdefinierte Merge-Operatoren im Kern (Tag `6` reserviert) · NZ-5 Schutz von Prozessspeicher/Swap/Core-Dateien · NZ-6 Umordnung über Clustergrenzen · NZ-7 Altformat-Kompatibilität · NZ-8 Windows mit voller Durability in v1 · NZ-9 `mmap`/`O_DIRECT`/`io_uring` im Kern · NZ-10 Kompression in v1 (`COMPRESSION=0`, Feld/Featurewert reserviert) · NZ-11 Inhaltsadressierung, Dedup · NZ-12 Tokenizer, Embeddings, ANN, Volltext, Graph · NZ-13 clusterübergreifende Commit-DAGs, Merkle-Zustandshashes · NZ-14 Lösch-Orchestrierung, Berichte, Signaturen · NZ-15 Schlüsselverwaltung (→ `KeyProvider`) · NZ-16 Mandanten-/Rechteprüfung · NZ-17 Async-Fassade · NZ-18 KV-Cache-/Prefix-Store · NZ-19 zwei lebende Klone einer Instanz · NZ-20 Multi-Node-Replikation/Failover bis zu einem späteren, eigenen Protokoll-Gate.

**Verteilbarkeit vorbereiten, nicht vorwegnehmen:** IDs und Stamps sind instanzlokal; jedes externe Ereignis/Receipt muss daher `(instance_id, stamp)` tragen und darf Stamp allein nie als global sortierbar behandeln. Storage-Ports bleiben austauschbar und alle Zustandsänderungen laufen über versionierte Redo-/Catalog-Records. V1 verspricht weder Replikationslog, globale Reihenfolge, Split-Brain-Schutz noch automatisches Failover. Ein späteres verteiltes Profil braucht ein separates Protokoll für Leader-Epoch/Fencing, Replikationsquorum, Restore/Erase-Anker, Idempotenz über Knoten und Cluster-Platzierung; es darf die lokale Einzelknoten-Durability-Semantik nicht still ändern.

### 2.4 Bedrohungsmodell

| ID | Angreifer/Fehler | Gegenmaßnahme |
|---|---|---|
| AT-1 | Prozess mit Lesezugriff auf das Verzeichnis, ohne Schlüssel | Siegel (§5.2) |
| AT-2 | Besitzer des Datenträgers, ohne Schlüssel | Siegel, Schlüssel außerhalb (`KeyProvider`) |
| AT-3 | Dateimanipulation (Einfügen, Abschneiden, Vertauschen) | Ketten-MAC, AEAD-AAD, Katalog-Kette |
| AT-4 | fehlerhafte/bösartige obere Schicht | nur versiegelte `ExtStore`-/`SegmentIo`-Wege; Orakel-Konformanz |
| AT-5 | **Rollback** auf frühere konsistente Stände, um Gelöschtes zurückzubringen | Ledger + Anker (§10.3), Restore-Reconcile |
| AT-6 | Betreiber mit Löschwunsch gegen Restriction | Prüfung in Committer **und** Purge; `lift` nur nach `LiftPolicy` |
| F-1…F-9 | Torn Write, Fsync-EIO, ENOSPC, Bit-Rot, Schlüsselverlust, Crash im Löschprotokoll, Retry-Stürme | §3.5, §10 |

Annahmen: AN-01 `sync_*` macht durabel · AN-02 Schreibvorgänge können zerreißen · AN-03 Fsync-Fehler nicht wiederholbar · AN-04 `rename` atomar · AN-05 Prozessspeicher vertrauenswürdig, nicht kontrolliert gelöscht · AN-06 `destroy_many` macht Schlüssel bei Rückkehr unbenutzbar · AN-07 starker Zufall · AN-08 Anker außerhalb des Datenverzeichnisses · AN-09 Stamps nie aus Uhr; Deadline-/TTL-Zeit nutzt monotone Laufzeit, UTC und durablen `clock_floor`. Rücksprung wird geklemmt; UTC-Sprung über `max_clock_skew` friert Hold-/Retention-Abläufe ein, alarmiert und markiert Erase-Frist als `ClockUncertain`, statt vorzeitig zu löschen oder Fristtreue vorzutäuschen · AN-10 Append-Dateien schreiben nur in vorallokierte Bereiche (ENOSPC nur beim Vorallokieren).

---

## 3. Architektur

### 3.1 Crates und Schichten

```
contextra-storage-ports   Traits und Werttypen, keine Logik
        ▲
contextra-storage         Kern, #![forbid(unsafe_code)], nur std + ports + bewilligte Abhängigkeiten
        ▲ (nur dev)
contextra-crypto-ref      Referenz Cipher/Mac/Kdf, In-Memory-KeyProvider, KeyProviderConformance
contextra-keys-file       Referenz-KeyProvider Einzelhost (ab P6)
contextra-keys-cohort     optional: Cohort-Rewrap-Provider für Millionen Cluster (ab P6)
```

Modulschnitt (Abhängigkeiten nur nach unten, Test `layering`):

```
api · engine (committer, memtable, version, views, branches, registry, claims, volatile, guard_window)
maint (flush, compaction, blob_gc, purge, scheduler, scrub) · erase (gate, cluster_table, links, protocol, ledger, oracle[testing])
recovery · catalog · store (run, blob, segment, redo) · keys · codec (varint, tags, checksums, comparator, seal)
env (Fs, Clock, Entropy, Governor, Metrics, Anchor; MemFs, RealFs, FaultFs, Sim) · sync (Ränge, Epoche)
```

`std::fs`, `std::time::SystemTime`, `rand` nur in Adaptern (Lint `disallowed_methods`). Kein `mmap` im Lesepfad. Namensraum: `crates/contextra-storage(-ports)`; `contextra-store` (Altstand) bleibt unverändert bis zur Ablösung (§14).

### 3.2 Ports

| Port | Aufgabe |
|---|---|
| `Fs` | Dateien, `read_at`, geordnetes `read_ranges`, `append_prealloc`, `preallocate`, `sync_data/all/dir`, `rename`, `lock`, `list`, `free_space`, optionale `link/reflink/punch_hole` |
| `Clock`, `Entropy` | injizierbar; Uhr nie für Stamps. Liefert monotone Laufzeit, UTC, durablen High-Water und Skew-Status; Fristablauf fail-closed bei unplausiblen Sprüngen (§3.5) |
| `Anchor` | `read()`, `advance(expect,next)` (CAS, monoton, durabel), `strength() → Strong\|Weak`; Wert `(erase_epoch, ledger_head, catalog_epoch)` |
| `KeyProvider` | §9.4 |
| `Cipher`/`Mac`/`Kdf`/`Checksum` | AEAD (XChaCha20-Poly1305, 24-Byte-Zufallsnonce), Ketten-MAC, HKDF, CRC32C |
| `Authority` (optional) | prüft Signaturen für `LiftPolicy::FourEyes` |
| `Metrics`, `Observer` | nicht blockierend, nie unter Sperren |
| `MemoryBudget` | gemeinsamer RAM-Admission-Port; `try_reserve(class, bytes) -> ResourcePermit`, Drop gibt exakt frei |
| `ColdTier` (optional) | Auslagerung kalter Dateien, Garantien des Ports gelten |

### 3.3 Mechanismus, nie Politik

Der Kern entscheidet nie, **ob** gelöscht werden darf oder muss, **wer** ein Recht hat, **welche** Frist gilt. Er setzt Aufträge atomar, geordnet und nachweisbar um und verweigert sie nur bei technischem Hindernis oder `Restriction`. Der Kern bildet **keine persistierten Inhalts-Hashes** über Klartext (`INV-PRIV-1`).

### 3.4 Threading

Ein Committer (Flat-Combining, Gruppen-Fsync). Lesen lock-frei bis auf `Arc`-Klone und Blatt-Sperren. Wartung in Lanes `urgent > flush > compact > blobgc > idle`. Ledger und Anker haben je **genau einen Schreiber** (`urgent`). Threads gehören dem Aufrufer oder einem `Spawner`. Keine Async-Runtime im Kern.

### 3.5 Fehlermodell

| Klasse | Beispiele | Verhalten |
|---|---|---|
| F-1 transient | `EINTR` | Wiederholung mit Backoff |
| F-2 Ressource | `ENOSPC` beim Vorallokieren | **kein Poison**; `DiskFull`; Löschung/Wartung laufen weiter (§6.6) |
| F-3 E/A-hart | `EIO`, Fsync-Fehler | **nie wiederholen**, `Poisoned`, Recovery per Neustart |
| F-4 Integrität | CRC/MAC/AEAD-Fehler | kleinste Einheit vergiften (Block → Abschnitt → Datei), `Corrupt{..}`, Rest läuft |
| F-5 Format | `Unsupported` | Öffnen verweigert |
| F-6 Schlüssel | Provider offline / Schlüssel fehlt | `KeyUnavailable{retry_after}` bzw. `Shredded`; **`KeyLost`** wenn der Schlüssel fehlt, obwohl kein Löschnachweis existiert (Alarm, keine automatische Löschung) |
| F-7 Aufrufer | Guard, `Restricted`, `UnknownCluster`, `BelowFloor`, `Conflicts` | typisiert mit `hint()` |
| F-8 Logik | Invariante verletzt | Test: Abbruch; Produktion: Poison + Observer; **kein `panic`** |
| F-9 Rollback | Anker höher als Catalog/Ledger | schreibend verweigert (`Err(Rollback)`), nur `ReadOnly` mit Diagnose |

Fehlertexte enthalten keine Schlüssel, Nutzerschlüssel, Körper oder Klartextfragmente.

### 3.6 Lint-Pflicht

Fehler im Kern: `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `unreachable`, `indexing_slicing`, `let _ =` auf Rückgaben von `sync_*`, `rename`, `unlink`. `Debug` für ClusterId, Schlüssel, Körper ist redigiert (`Redacted<T>`).

---

## 4. Datenmodell

### 4.1 Vokabular

| Begriff | Bedeutung |
|---|---|
| **Space** | benannter, physisch getrennter LSM-Baum (`SpaceId u16`, nie wiederverwendet), `Durable` oder `Volatile` |
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
| **tomb_id** | `HMAC(ledger_key, ClusterId)`; Löschnachweis ohne Pseudonym |

### 4.2 Reihenfolge, Tag, Marker

`cmp`: Cluster aufsteigend, **Branch aufsteigend**, Nutzerschlüssel bytelexikografisch, Tag absteigend (kein `memcmp` über Konkatenation). Ohne `clustered` ist der Cluster implizit `NONE`.
Tag `u64 = (stamp << 8) | kind`; `Put=1, Delete=2, RangeDelete=3, EraseRange=4, PutRef=5, Merge=6 (reserviert, Decoder: Unsupported)`. `PutRef` nur in Runs. `RangeDelete` ist branch-lokal mit Marker `(cluster, branch, start, end, tag)`, `end` exklusiv. `EraseRange` ist dagegen ein clusterweiter Control-Marker ohne Branch und wirkt auf alle Branches und historischen Views; er ist nie ein gewöhnlicher Merge-Wert. Für Sealed-Cluster bleiben nur Markerart, Cluster-Routing und Stamp im Klartext; `start/end` sind cluster-key-versiegelt. Der Committer hält das Gate-Maskenintervall im RAM, und Recovery entschlüsselt es vor Freigabe des Clusters. Bei zerstörtem Clusterschlüssel wird der Cluster insgesamt gesperrt/retired, nicht versucht, das Intervall selektiv zu rekonstruieren.
Stamps: erster = 1, streng monoton über angenommene Changes und Frames, nie doppelt vergeben (auch nach Absturz), Überlauf ⇒ `Exhausted`. Lücken zulässig.

### 4.3 Cluster

- **Erzeugung:** `Clusters::register(ClusterSpec)` zieht 128 Bit aus `Entropy`, prüft Kollision, legt bei `Sealed` **zuerst** den Schlüssel an (`create_cluster_key`, durabel), dann Catalog-Edit `cluster_register`. Ein Schlüssel ohne Cluster (Absturz dazwischen) wird von `idle` nach `orphan_key_grace` (24 h) über `KeyProvider::list` vernichtet.
- **Zustände:** `Live → Retired(S) → [Shredded] → Purged → (Trim)`. `Shredded` nur bei `Sealed`. Nur vorwärts.
- **Trim:** Ein `Purged`-Eintrag wird per `cluster_trim` entfernt, sobald `history_floor > S` (keine View/Hold/Consumer-Fenster darunter). Danach `UnknownCluster`. Wiederverwendung ausgeschlossen (128-Bit-Zufall). Der Dauer-Nachweis liegt als `tomb_id` im Ledger (§9.7).
- **Restrictions** sind eine Menge am Cluster, kein Zustand.
- `ClusterSpec` trägt unveränderlich `erase_locality`. v1-Default ist `PackedBounded{max_file_bytes: 64 MiB}`; `ClusterLocal` ist explizit zu wählen, wenn unlink-basierte Run-/Blob-Isolation die zusätzliche Datei-/FD-Last rechtfertigt. `ClusterLocal` erzwingt eine Cluster-ID je Run-/Blob-Datei; gepackte Sections allein erfüllen die Option nicht. Beide Modi unterliegen den separaten Redo-/Pin-Kosten aus INV-ERA-11.

### 4.4 Wert, Kopf, Körper, Gewichte

Wert = `header ‖ body`, getrennt durch `header_len`. **Der Kern interpretiert den Kopf nie.** `weights: [u32; W]` (`W = weight_slots`, 0…4); Block-Index hält `weight_sum[W]` und `entry_count`. Körper ≥ `value_separation_min` (1 KiB) gehen in Blobs (`PutRef`), der Kopf liegt immer inline.

### 4.5 `SpaceOptions` (unveränderlich)

```rust
pub struct SpaceOptions {
    pub class: SpaceClass,          // Durable | Volatile
    pub clustered: bool,
    pub personal: bool,             // erzwingt clustered, nur Sealed-Cluster, bei Durable seal = Cluster
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

`personal = true` verlangt `integrity_mode = Encrypted`. **Volatile-Spaces:** kein Redo/Run/Blob, im Catalog nur der Name, sofort sichtbar, Inhalt verschwindet bei Schließen/Absturz, Erase = Freigabe mit Überschreiben (best-effort). Mischchanges mit Volatile- und Durable-Zielen nur mit `allow_mixed`, nur `durable`, wenn alle Ziele Durable sind (`INV-VOL-1`).
`compression=None` ist die einzige v1-Option; nicht unterstützte Kompression wird vor Mutation mit `Unsupported` abgewiesen. Kompression erfordert später ein versioniertes Format mit unkomprimierter Ziellänge und hartem Expansionslimit.

### 4.6 Limits

`max_key_bytes` 1 KiB (≤ 4 KiB) · `max_header_bytes` 4 KiB (≤ 64 KiB) · `max_body_bytes` **4 MiB (hart; E-35)** · `max_change_ops` 4096 · `max_change_bytes` 8 MiB · `max_page_bytes` 1 MiB · `max_clusters` 16 Mio. (weich, ≤ 64 Mio.) · `max_branches_per_cluster` 1024 · `max_branch_depth` 8 (H-21 muss die konfigurierte Maximaltiefe messen; falls Ziel verfehlt, ist der Default zu senken oder Flattening zu erzwingen). Überschreitung ⇒ `Limit`/`ChangeTooLarge` **vor jeder Sperre** (`INV-LIM-1`).

---

## 5. Platzierung, Formate, Siegel

### 5.1 Pure, Packed, Staging

- **R-PL-1** Jede Datei ist genau eine von Pure (ein Cluster), Packed (mehrere Cluster einer Packgruppe), Staging (nur L0, beliebige Cluster, nur Runs).
- **R-PL-2** Jeder Cluster bildet je Datei **einen zusammenhängenden Abschnitt**, sortiert nach ClusterId, Blöcke clusterrein, Offsets abschnittsrelativ.
- **R-PL-3** Cluster ≥ `pure_threshold` wird bei der nächsten Compaction Pure (nie zurück).
- **R-PL-4** Compaction mischt nur innerhalb einer Wartungsgruppe (Packgruppe oder Pure-Cluster); L0→L1 entmischt Staging.
- **R-PL-5** Ein Flush schreibt je Space **eine** Staging-Datei plus je großem Cluster eine Pure-Datei: L0-Dateizahl unabhängig von der Zahl der Packgruppen.
- **R-PL-6** `rebucket` ist ein atomarer Umzug in einem Edit.

### 5.2 Siegelrahmen (eine Form für alles)

```
SealFrame := key_epoch u32 | nonce[24] | ciphertext | tag[16]     (Authenticated: nur tag; Plain: CRC32C)
AEAD = XChaCha20-Poly1305(quelle, nonce[24], AAD)
quelle = Clusterschlüssel(cluster, epoch)   bei Seal::Cluster
       = Space-DEK                          bei Seal::ByMode/Encrypted (Plain-Cluster)
       = gemäß IntegrityMode                bei Plain-Cluster (CRC-only, authentifiziert oder Space-DEK)
       Sealed-Cluster-Nutzlasten in Redo verwenden immer Seal::Cluster mit der im Op-Header gebundenen key_epoch.
AAD:  Block (instance, space, cluster, key_epoch, block_uid) · Redo-Op (instance, space, cluster, key_epoch, op_uid, op_index) · Blob-Record (instance, space, cluster, key_epoch, record_uid)
```

Pro SealFrame gibt es keine zusätzliche HKDF/HChaCha-Ableitung: XChaCha20-Poly1305 erhält den bereits scope-spezifischen Quellschlüssel, 24-Byte-Nonce und AAD direkt und führt seine spezifizierte HChaCha-Subkey-Ableitung intern aus. `Authenticated` nutzt HMAC-SHA256 über eindeutig codierte Domain `ctxs-auth-v1`, Modus, Nonce, AAD und Klartext; `Plain` nutzt CRC32C und bietet keine Manipulationsauthentizität. RootKey-abgeleiteter Redo-Schlüssel dient ausschließlich Ketten-MAC/Frame-Integrität und verschlüsselt **keine** Nutzlast eines Sealed-Clusters. AAD und Siegel hängen nicht von Dateiname, Datei-ID, Blob-ID, Offset oder Dateiposition ab; `block_uid`/`record_uid` sind bei Rohkopie stabil ⇒ Rohkopie über Dateien und neue Blob-IDs ist ohne Entschlüsselung gültig. Versiegelt wird vor Stamp-Vergabe; Stamp ist über Frame-Ketten-MAC gebunden. `INV-SEAL-1`: Sealed-Cluster haben Nutzdaten nur in `Seal::Cluster`- oder Volatile-Spaces; Plain-Cluster nie in `Seal::Cluster`-Spaces; `personal`-Spaces nehmen nur Sealed-Cluster. Folge: alle **Nutzdaten** eines Sealed-Clusters auf Platte (Redo-Op, Runs, Blobs, Kopf, Körper, Index, Filter, Branch-Nutzdaten) sind unter dessen Cluster-Epoche versiegelt. Klartext bleiben ausdrücklich: zufällige ClusterId, Routing-Header/Op-Tags, Gate-/Control-Frames, Größen/Stamp-Bereiche und Dateistruktur; dies ist das Metadaten-Leckagebudget und enthält keine Nutzwerte.

### 5.3 Konventionen aller Formate

Little-endian; `varint` = kanonisches LEB128; Dateikopf `magic[8] ‖ file_kind u8 ‖ format_ver u16 ‖ feature_bits u32 ‖ file_id u64 ‖ instance_id[16] ‖ header_crc u32`; Längen vor dem Allokieren prüfen (P-4); reservierte Felder 0 (P-10); eine Version je Dateiart, unbekannte Version/Pflicht-Bits ⇒ `Unsupported` (`INV-FMT-1`). Feature-Bit `BRANCHES` (§8) in `FORMAT`: solange ungesetzt, ist `branch ≠ 0` ein Format-Fehler. v1 setzt `COMPRESSION=0` und `filter_kind=Bloom(1)`; unbekannte Werte ⇒ `Unsupported`. Andere Kompressions-/Filterkodierungen benötigen ein neues Pflicht-Feature oder eine Formatmigration.

### 5.4 Dateien

```
<root>/FORMAT  LOCK  CURRENT  CATALOG-<n>  KEYS  RESERVE
       redo/redo-<n>.log  spaces/<id>/<id>.run|.blob  segments/<kind>/<id>.seg
<ledger_dir>/ERASE-LEDGER/…      (Standard: <root>/ERASE-LEDGER, Produktivempfehlung: getrenntes Volume)
```

`FORMAT` unveränderlich (+ RootKey-abgeleiteter MAC); `KEYS` = RootKey (mit KEK umhüllt) + Space-DEKs (mit RootKey umhüllt); **Clusterschlüssel stehen nie hier**. Alle Dateien `Perm::Private`. Temporäre Dateien `.building.<tmp_id>`. Verwaiste Dateien (nicht im Catalog) werden nach Recovery entfernt; sicher, weil nur ein Edit eine Datei lebendig macht (`INV-CAT-1`).

### 5.5 Redo

Vorallokierte Segmente (64 MiB, `redo_align` 4096), am Ende gesiegelt; Gruppe = ≥ 1 Frames, Ketten-MAC `H_i = MAC(H_{i-1} ‖ head ‖ payload_tags)`.

```
Frame := head ‖ body ‖ tail
head (Klartext, kettengeschützt): len u32 | stamp u64 | kind u8 | n_ops u16 | idem_token[16] | epoch u64 | flags u8 |
      ops_head[]: { space u16, cluster[16], branch u32, op_kind u8, key_epoch u32, op_uid[16], payload_len u32 }
kind: Change=1 | Gate=2 | Control=3 | Reserve=4
Gate-Body (Klartext):    scope(Clusters|Space) | ticket[16] | level | n u16 | cluster[16]×n  oder space u16
Control-Body (Klartext): op(Restrict|Lift|BranchFork|BranchDrop|BranchMerge) | … Nutzdaten der Branch-Ops sind versiegelt
tail: crc32c | chain_mac
```

Redo-Integritätsschlüssel = `KDF(RootKey, "ctxs redo-auth v1")` (nicht je Segment), ausschließlich für Ketten-MAC. Nutzlasten von Sealed-Clustern sind mit deren `Clusterschlüssel(cluster,key_epoch)` verschlüsselt; Plain-Nutzlasten folgen dem `IntegrityMode` und ggf. Space-DEK. Beim Recovery: Gate-/Control-Scan bestimmt `retired_set` ohne Entsiegeln; aktive Sealed-Ops werden nur mit ihrer `key_epoch` geöffnet. Provider nicht erreichbar ⇒ `KeyUnavailable`, Recovery bleibt in `LockedKeys`; bestätigter Schlüsselverlust ohne Löschbeleg ⇒ `KeyLost/Corrupt` und Quarantäne der betroffenen Cluster-Domäne. Nie stillschweigend überspringen und diese Zustände nie als `Shredded` behandeln. Torn-Tail (`INV-LOG-8`): erster ungültiger Gruppenkopf beendet das Log nur, wenn danach nichts Gültiges folgt; sonst `Corrupt{RedoMidLog}`; alles bis zur letzten `redo_mark` (alle 8 MiB und bei Siegel) muss gültig sein. **Kein Segmentschnitt beim Gate.**

### 5.6 Run

```
Run := header · ClusterSection* · ClusterDirectory · PackMeta · ExtensionBlock* · Footer
ClusterSection := DataBlock* · SubIndexBlock · FilterBlock · RangeDelBlock · ChangeIndexBlock
```

`ClusterDirectory` (Klartext): `{cluster, section_off, section_len, n_blocks, n_entries, min/max_stamp}`; keine Gewichte/Tokenprofile im Klartext. `DataBlock` Ziel 16 KiB, Präfixkompression, versiegelt; Eintrag trägt `branch`. `SubIndexBlock` je Block: letzter Schlüssel, abschnittsrelativer Offset, Länge, `entries`, `weight_sum[W]`, `min/max_stamp`, `block_uid`; bei Sealed vollständig unter Clusterschlüssel versiegelt. Page darf diesen kleinen Indexblock entsiegeln, ohne Body-Block/Blob zu lesen. v1-`FilterBlock` ist ausschließlich Bloom (`filter_kind=1`) über `user_key` ohne Branch (ein Filter für die Branch-Kette), samt Salt cluster-sealed; Ribbon ist kein v1-Format. Nur Directory, PackMeta und Footer mit nicht sensiblen Strukturmetadaten bleiben beim Umkopieren unverändert/werden neu erzeugt. Verzeichnisse liegen im Speicher (~48 Byte je Cluster und Datei).

### 5.7 Blob

`Blob := header ‖ ClusterSection* (Record*) ‖ ClusterDirectory ‖ footer`, `Record := record_uid[16] | len u32 | SealFrame | crc32c`. AAD bindet `record_uid`, nicht `blob_id` oder Offset; deshalb darf ein unverändertes SealFrame roh umkopiert werden. Ziel ≤ 64 MiB. **Inkarnation:** Kleine Löcher dürfen bei unterstütztem Hole-Punch dieselbe `blob_id`/Offsets behalten. Überschreitet Fragment-/Hole-Ratio den Grenzwert oder fehlt effizientes Hole-Punch, wird in eine neue `blob_id` mit gepackten Überlebenden umgeschrieben; Catalog-Edit `blob_replace` aktualisiert Referenzen atomar. Blob-GC darf zusätzlich in eine neue `blob_id` umziehen; bei unverändertem Record ist keine Neuversiegelung nötig. Eine Blob-Datei/Inkarnation wird nur entfernt, wenn **kein lebender Run** mehr einen `PutRef` darauf enthält (`INV-BLOB-1`).

### 5.8 Catalog

Snapshot-Record + Edit-Records (Kette, CRC), nur in vorallokierten 1-MiB-Blöcken. **Gruppen-Commit:** `catalog_edit` fasst wartende Edits mit einem `write` + einem `sync_data` zusammen, entlässt die Wartenden erst danach (`INV-CAT-2`); mehrere gekoppelte Zustandsänderungen sind **ein** Edit (P-2). Rollover bei `max(8 MiB, Größe des letzten Snapshots)`; `CURRENT` per atomarem `rename`.
Ops: `space_add | space_state | space_remove | cluster_register | cluster_state | cluster_trim | cluster_link | restriction_set | restriction_lift | pack_assign | files_add(meta.floor) | files_remove | unlink_ack(file_id) | blob_replace | blob_live_delta | replay_floor | retain_floor | redo_mark | segment_add | segment_remove | hold_set | consumer_ack | erase_pending_add | erase_stage | erase_done | erased_ranges_add | erased_ranges_trim | key_epoch | kek_id | idem_trim | reserve_stamps | reserve_ids | anchor_mark`. `files_remove` verschiebt die Datei im selben Edit in den persistenten Graveyard; `unlink_ack` entfernt sie erst nach erfolgreichem `unlink + sync_dir`. Reservierter Bereich `0xF000…0xFFFF`; unbekannte Tags ⇒ `Unsupported`. **Der Catalog enthält keine Branch-Namen und keine Nutzdaten** (§8.2).
Reservierungen: `reserve_stamps` (2^20) und `reserve_ids` (2^16) heben Hochwassermarken vorab an (Wartungsthread bei < 50 % Rest); Fallback im Committer ist die **einzige** Doppelhaltung von Rang 1 und Rang 3.

### 5.9 Freeze-Liste (Gate G-F)

Dateikopf, `FORMAT`, `KEYS`, Redo-Gruppe/Frame (inkl. `branch`, `kind`, `epoch`, `op_uid`, Gate-/Control-Body), Kette, ClusterSection, ClusterDirectory, PackMeta, SubIndex, Siegelrahmen, Blob-Record und Inkarnationsregel, Catalog-Op-Tags, `AnchorValue`, Ledger-Format, `cmp` (inkl. Branch), Tag-Kodierung. Je Format ein unabhängiger Referenzcodierer (Python) und goldene Vektoren (Anhang A). Nach G-F ist jede Änderung eine Migration.

---

## 6. Schreibpfad

### 6.1 `apply`

```
1  Vorprüfung ohne Sperre: Größen, Space/Cluster bekannt, INV-SEAL-1, Clusterzustand, IdemToken (AlreadyApplied), DiskFull
2  Vorbereiten auf dem Aufruferthread: View V0 pinnen, Guards gegen V0 (Fingerabdruck), Ops kodieren, op_uid ziehen,
   Nutzlast versiegeln (Provider-Wartezeit liegt hier, nie im Committer)
3  Stamp-Reserve prüfen (ggf. reserve_stamps vor Rang 1)
4  Intake (Rang 2), Warten auf Gruppenführer
5  Committer (Rang 1) nimmt alle Wartenden:
     a) Cluster-/Branch-Zustand: Retired/Shredded/Purged ⇒ ClusterRetired; BLOCK_WRITE ⇒ Restricted; Branch nicht Live ⇒ BranchClosed
     b) Guards bestätigen (Schnellpfad Konfliktfenster, sonst exakt gegen Memtable ∪ Version ∪ Gruppen-Overlay)
     c) Stamps vergeben, vorreservierte Branch-IDs validieren und Branch-Ops ausführen
     d) Gruppe bauen (Frames, Kette)
     e) ein write + ein sync_data                              ◄── DURABILITÄTSPUNKT
6  Memtable einfügen, Konfliktfenster fortschreiben, Cluster-/Branch-Zustand setzen, Version veröffentlichen (Rang 4),
   visible_stamp anheben (Release)
7  Receipt { stamp, durable }
```

Sichtbarkeit nur nach Durabilität (Ausnahme `Relaxed`, nie für Löschung). Sync-Fehler ⇒ Poison; alle Changes der Gruppe `Indeterminate`, spätere `Poisoned`; Lesen bleibt. Rückdruck stufenweise **außerhalb** jeder Sperre (`Backpressure(Retry)`); Warten unter Rang 1 ist verboten. `Urgent`-Löschfolgen umgehen den Rückdruck.

### 6.2 Memtable und Stamps

Lock-freie Skiplist hinter dem Trait `Memtable`, Klartext (AN-05); 64 MiB je Space, ≤ 3 immutable, 256 MiB gesamt. Nach Absturz: `next_stamp = max(stamp_limit, letzter gültiger Redo-Stamp, max Run-Stamp) + 1`. Consumer und `view_durable` sehen nur Stamps ≤ `durable_stamp` (`INV-CONS-1`).

### 6.3 Guards in zwei Stufen

Aufrufer wertet gegen `V0` aus und berechnet für Punkt-Guards `fp = SipHash-1-3(prozessschlüssel, space‖cluster‖branch‖key)`. Der Committer prüft das **Konfliktfenster** (Ring der letzten 1 Mio. `(fp, stamp)`, nur im Speicher): kein Stamp > `V0` ⇒ Punkt-Guard gilt (O(1), keine E/A). Sonst oder bei `V0` älter als Fensterfuß: exakter Langsampfad. Hashkollisionen dürfen nur den Langsampfad auslösen, nie Konflikte übersehen. `Guard::Range{start,end,since}` ist halboffen und wird über den stamp-sortierten ChangeIndex gegen Punkt- und Bereichsmutationen geprüft; der Punktfilter darf Range-Guards nicht vorzeitig bestätigen. `INV-GUARD-1`: Ein Change wird nur angenommen, wenn jede Guard zur Stamp-Vergabe gilt. `Precondition`: `Absent | Version(Stamp) | Unchanged{since}`.

**Guard-Katalog je Operationsart (aus CairnDB, als Konvention für die obere Schicht):** Event anhängen → keine Guard (append-only); Working State → `Version`; Fakt ändern → `Version`; Tool-Zustand → `IdemToken` + `Version`; Branch-Merge → `Unchanged{since: merge_base}` je Schlüssel (§8.5).

### 6.4 Flush

Je Space: Memtable einfrieren → je Cluster ein Abschnitt (**Retired-Cluster werden übersprungen**) → Staging- plus Pure-Runs (`.building` → `sync_all` → `rename` → `sync_dir`) → **ein** Edit (`files_add`, `replay_floor`, `anchor_mark`) → neue Version. `replay_floor` rückt nur über Segmente, deren Frames in durablen Runs stehen **und** deren Gate-Frames als `erase_pending` im Catalog stehen (`INV-LOG-5`). Flush braucht die Clusterschlüssel der Memtable (gepinnt für die Jobdauer); fällt der Provider aus, pausiert er mit Backoff.

### 6.5 Idempotenz

`IdemToken` im Frame-Kopf; Committer emittiert zusätzlich einen internen `SYS_IDEM`-Record im **selben Redo-Frame/Commit** wie den Change. `SYS_IDEM` ist ein reservierter, instanzweiter Systemspace unter dediziertem System-DEK; Record-Schlüssel und Request-Fingerprint sind keyed HMACs, kein Nutzwert und keine Klartext-ClusterId. Systemspace-Key/HMAC-Epoche bleibt bis Ablauf aller damit markierten Tokens verfügbar. Record-Payload wird vor Rang 1 versiegelt; Ergebnis-Stamp ist der äußere MVCC-Stamp des SYS_IDEM-Records, kein im Committer zu versiegelndes Feld. Aktive Records dürfen nicht durch `replay_floor` verworfen werden: sie werden geflusht/kompaktiert und bis TTL-Ende gepinnt. Gespeichert werden Token-Scope/Hash, kanonischer Request-Fingerprint, Status, Ablaufbucket und pseudonymes Cluster-Scope-Digest aus `tomb_id`s. Catalog-`idem_trim` ist nur eine durable Trim-Wasserstands-/Epoch-Marke, kein per-Commit zweiter Fsync. Aktive Tokenanzahl und SYS_IDEM-Bytes sind konfigurationsbegrenzt; bei ausgeschöpftem Limit wird ein tokenierter Commit vor Mutation mit `IdemCapacity` abgelehnt, aktive Tokens werden nie vorzeitig verdrängt. Bei nicht verfügbarem SYS_IDEM-Schlüssel werden tokenierte Reads/Writes fail-closed abgelehnt; kein Retry darf ohne Idem-Prüfung committen. Systemspace-Format, Schlüsselableitung, TTL-Index, Limits und Trim-Recovery sind G-F-Formatbestandteil. Wiederholung desselben Changes ⇒ gespeichertes Ergebnis; Token mit abweichendem Request ⇒ `IdemConflict`; nach Cluster-Retire ⇒ `Erased`/`ClusterRetired`, niemals neues Apply.

### 6.6 Notbetrieb bei vollem Datenträger

Redo, Catalog und Ledger schreiben nur in **vorallokierte** Bereiche (AN-10); `ENOSPC` tritt nur beim Vorallokieren auf und hinterlässt keine Teilschreibung. `DiskFull` verweigert User-Commits; Reads und Gate/KeyDestroy nutzen getrennte Emergency-Reserven. Purge hat eigenen geschützten Scratch-Pool von mindestens `2 × max(run_target_bytes, blob_max_bytes, redo_segment_bytes)` (Default 128 MiB) für genau einen aktiven Rewrite plus Crash-Recovery-Slot. Ordinary Flush/Compaction pausieren, sofern sie diesen Pool nicht verwenden dürfen. Pool wird bei Open geprüft und kann nicht von normalen Writes verbraucht werden. Purge rewritet Dateien einzeln: `PurgeBatchEdit` installiert je Ersatzdatei atomar; vollständig tote Dateien werden direkt aus dem Catalog genommen und danach unlinkt; nach Arc-/Reader-Reclamation wird freier Quellplatz dem Scratch-Pool zurückgegeben. Nur nach letzter Datei und Extension-Attest setzt ein finaler Edit `Purged/Done`. Filesystem-Adapter dürfen diese Garantie nur deklarieren, wenn Vorallokation auch unter CoW/Quota-Bedingungen tatsächlich Platz reserviert. Reicht Reserve/FS-Capability nicht, verweigert Open den harten DiskFull-Erase-Modus oder meldet das Restrisiko explizit; kein falsches `Done`. Gate-/Control-/Ledger-Frames passen ins Emergency-Reservesegment (`INV-LOG-7`). Ausstieg mit Hysterese. Nur `EIO`/Fsync-Fehler vergiften (`INV-DISK-1`).

---

## 7. Lesepfad

1. `view()` lädt zuerst `visible_stamp` (Acquire), dann `Arc<Version>` (`INV-VIEW-2`). Keine Sperre über E/A.
2. **Gate zuerst:** Clusterzustand und Restrictions vor jeder Entsiegelung (O(1)). Unbekannt ⇒ `UnknownCluster`; `Retired` oder weiter ⇒ `Erased{at:S}` **für jede Sicht, auch für Sichten ≥ S** (Erasure-Barriere überstimmt Snapshot-Isolation); `BLOCK_READ` ⇒ `Restricted`; widerrufene View ⇒ `Revoked`.
3. **Punktabfrage:** Memtable → immutable → je Level die Dateien der Wartungsgruppe (Binärsuche nach `[min_cluster,max_cluster]`) → ClusterDirectory → SubIndex → Filter → **ein** Datenblock je Treffer-Level.
4. **Cursor/Page:** k-Wege-Merge über Quellen **eines Clusters und seiner Branch-Kette**; Kopf ohne Body-Block-E/A; `page` entschlüsselt kleine SubIndex-Metadaten mit `weight_sum`, überspringt danach Body-Blöcke ohne deren Entsiegeln und liefert `ResumeToken`. Ein Eintrag, der das Budget sprengen würde, beendet die Seite, **außer** die Seite wäre leer (Fortschrittsgarantie, `oversize_single`). Cursor prüfen Gate und Widerruf **zwischen Blöcken**. Der `ResumeToken` ist AEAD-versiegelt unter einem Prozessschlüssel und enthält Stamp, Cluster, Branch, letzten Schlüssel (kein Klartextschlüssel außen); nach Löschung ⇒ `Err(Erased)`.
5. **Körper** erst bei `body()`; Schlüssel fehlt ⇒ `Shredded`, Gate ⇒ `Erased`. Nie Daten, nie „leer“.
6. **Block-Cache** hält entsiegelte Blöcke mit `(cluster, gen)`; Treffer nur gültig, wenn `gen` dem Clusterzustand entspricht; Gate und ER2 erhöhen `gen`; `urgent` überschreibt die Puffer (`Zeroizing`). **Check-after-decrypt:** `gen` wird vor **und** nach dem Entsiegeln geprüft (`INV-KEY-2`).
7. `view_at(S)` mit `S < history_floor` ⇒ `BelowFloor`; Sicht unter `S_erase` in einen Bereich mit `erased_ranges` ⇒ `Erased` (`INV-ERA-5`).
8. **`View::filter_visible(&[RecordRef])`:** prüft Clusterzustand/Restriction und Stamp für Treffer, die obere Strukturen (Indizes) liefern. **Jedes Ergebnis einer Extension muss diesen Filter passieren, bevor es den Aufrufer erreicht** (`INV-VIS-1`); damit ist die Sichtbarkeit unabhängig davon, wie schnell die Extension aufräumt.
9. Keine clusterübergreifenden Scans im Kern; `Inspector::clusters()` iteriert.

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

Branch-Metadaten liegen **im Cluster**, nicht im Catalog: reservierter Space `SYS_BRANCH` (SpaceId 1, clustered, `personal`-Regeln wie der Cluster), Eintrag `branch_id → { name (≤ 64 B, opak), parent, fork_point: Head|At(Stamp), state: Live|Merged|Dropped, ttl, created_by }`. Für `Head` ist der MVCC-Stamp des Branch-Erstellungsrecords der `fork_stamp`; der Stamp wird nicht zusätzlich im versiegelten Payload dupliziert. Der Catalog bleibt frei von Namen und Nutzdaten. Eine In-Memory-Tabelle je Cluster (Blatt in `cluster_table`) wird beim ersten Zugriff aufgebaut.

### 8.3 Lesen auf einem Branch

Sicht `(V, b)`: Für einen Schlüssel gilt die neueste Version in `b` mit Stamp ≤ `V`; sonst rekursiv in `parent(b)` mit Stamp-Obergrenze `min(V, fork_stamp(b))`. `Delete`/`RangeDelete` in `b` verdecken Elternversionen. Zusätzlich wird der clusterweite `EraseRange`-Maskensatz vor Versionsauflösung angewandt und kann durch keine Branch-Version oder Merge-Resolution aufgehoben werden. Umsetzung: k-Wege-Merge mit je Quelle einer **Stamp-Kappe**. Der Filter einer Datei bezieht sich nur auf `user_key` (ein Probe für die ganze Kette).

### 8.4 Fork, Drop, TTL

```rust
impl Storage {
    pub fn fork(&self, c: ClusterId, parent: BranchId, at: ForkPoint, o: BranchOptions) -> Result<BranchId, Error>;
    pub fn branch_drop(&self, c: ClusterId, b: BranchId) -> Result<(), Error>;                 // logisch, nur leaf
    pub fn eraser().branch(c, b, req) -> Result<Ticket, EraseError>;                           // Logical | Physical | Scrubbed
}
pub enum ForkPoint { Head, At(Stamp) }
pub struct BranchOptions { pub name: Bytes, pub ttl: Option<Duration> /* spekulative Branches */ }
```

Fork linearisiert als Op im Committer; `BranchId` wird zuvor mit `reserve_ids` durable reserviert und SYS_BRANCH vor Rang 1 versiegelt. Bei `Head` setzt der äußere Frame-Stamp den `fork_stamp`. Kosten O(1), keine Nutzdatenkopie. `branch_drop` ist nur für leaf-Branches zulässig; ein Branch mit lebenden Kindern ergibt `BranchHasChildren`. TTL-Expiry folgt derselben Regel bottom-up. Erfolgreich gemergte Sources sind terminal/read-only, bleiben aber lesbar bis zu ihrem `branch_drop`. Branch-Drop macht nur die Sicht dieses leaf-Branches unsichtbar; physisch verschwinden Einträge bei Compaction. Für Sealed-Branches gibt es kein isoliertes Crypto-Erase; sofortiges Crypto-Erase löscht den ganzen Cluster. v1 erlaubt maximal 8 Branch-Ebenen; aktive Branch-Anzahl ist pro Cluster konfigurierbar und durch das gemeinsame MemoryBudget begrenzt. `gc_floor` ist das Minimum aller aktiven View-/Hold-/Consumer-Stamps und Fork-Stamps lebender Branches; ohne Pins gilt `durable_stamp + 1`. `history_floor` ist der publizierte untere Rand tatsächlich lesbarer Historie, wird erst nach erfolgreichem GC/Compaction fortgeschrieben und darf nie größer als `gc_floor` sein. `At(S)` verlangt `history_floor ≤ S ≤ parent_head`; `At(S < history_floor)` ergibt `BelowFloor` und wird nicht auf den Head umgedeutet.

### 8.5 Diff und Merge

- **Diff** `(view_a, view_b, cluster)`: Menge der Schlüssel, deren sichtbare Version sich unterscheidet. `ChangeIndexBlock` enthält cluster-sealed `(stamp, branch, op_uid, kind, key/range)`-Einträge einschließlich Punkt-, `RangeDelete`-, `EraseRange`- und Branch-State-Mutationen, sortiert nach `(stamp, op_uid)`. Vor dem Flush kommen dieselben Index-Records aus Memtable/Redo-Overlay; Recovery rekonstruiert sie aus Redo. Flush/Compaction übertragen sie verlustfrei in Runs. Diff vereinigt Kandidaten seit dem gemeinsamen Floor aus beiden Sichten, löst Sichtbarkeit auf und vergleicht Endzustände; Kosten ∝ Mutationen seit diesem Floor + Ausgabe, nicht pauschal ∝ Branch-Einträge. `branch_diff(branch)` relativ zum ForkPoint scannt nur Branch-Mutationen. Records bleiben gepinnt, solange Views/Branches/Merges sie benötigen, und werden erst unter `gc_floor` kompaktierbar. Das Layout ist G-F-Bestandteil.
- **Merge `source → target`** ist v1 nur für einen direkten Child-Branch erlaubt (`parent(source) == target`), wodurch `merge_base = fork_stamp(source)` eindeutig ist. Source mit lebenden Kindern, geschlossene/retired Cluster oder veraltete Heads ergeben einen Fehler ohne Mutation. In drei Schritten:
  1. **Berechnung:** `K` = Punkt-/Intervallmutationen aus `source` seit `fork_stamp`, ermittelt über den ChangeIndex. Für jeden Effekt prüft der Merge per Target-ChangeIndex, ob `target` seit `merge_base` (= `fork_stamp`) eine überlappende Punkt- oder Bereichsänderung trägt. Gleiche Endzustände (Kopf+Körper bytegleich sowie gleiche Löschsemantik) sind **kein** Konflikt.
  2. **Ergebnis:** `Merged(Receipt)` oder `Conflicts(Vec<ConflictCell>)`; **bei Konflikten wird nichts geschrieben.** `ConflictCell { key, base, ours, theirs }` sind Daten der Aufrufer-Schicht, nicht persistiert.
  3. **Anwendung:** Die Konfliktlösung wird als vollständige Ops-Menge (`Put | Delete | RangeDelete`; `EraseRange` bleibt unwiderruflicher clusterweiter Gate-Marker und kann nicht als Zielwert gewählt werden) in einem normalen `Change` auf `target` mit Punkt- und Range-Guards seit `merge_base` kodiert. Der Merge-Frame bindet `source`, `source_head_stamp`, `target_head_stamp` und `merge_base`; Commit revalidiert alle vier und setzt Source-Status `Merged(at stamp)` im selben durablem Frame. Alles atomar; bei Konflikt/Head-Drift wird nichts geschrieben.
- **Core-Modi:** `FailOnConflict`, `PreferSource`, `PreferTarget`. **Typisierte Strategien (LWW, Union, Summe, Textmerge, semantischer LLM-Merge) liegen oben** und liefern die Auflösung; Tag `Merge=6` bleibt reserviert (NZ-4).
- **Idempotenz:** Merge trägt einen `IdemToken`.

### 8.6 Zeitmodell

Stamp = Transaktionszeit. Gültigkeitszeit, Beobachtungszeit, Quellzeit leben im Kopf der oberen Schicht; `supersedes`-Kanten sind oben (§12.6). „Was wusste der Agent bei Stamp S?“ = `view_at(S)`; „Was galt am Datum t?“ = Filter oben.

---

## 9. Löschung

### 9.1 Prinzip

Löschen ist die primäre Strukturierungsachse. Drei Schritte, unabhängig und geordnet: **Gate** (logisch, Millisekunden) < **Schlüssel** (kryptografisch) < **Purge** (physisch, bis zur Frist). Der Kern sagt nie mehr, als die erreichte Stufe trägt.

### 9.2 Stufen (zwei Achsen, K-23)

| `EraseLevel` (Ziel) | Für | Verfahren | Schützt gegen | Schützt nicht gegen |
|---|---|---|---|---|
| `Logical` | beide | Gate | Lesen über die API | Inhalt auf Platte |
| `Physical` | `Plain` | Gate, Barriere, Purge, Edit, Unlink, `sync_dir` | Lesen aus lebenden Dateien, Wiederkehr nach Absturz (mit Anker) | Journal, Flash-Reste, Backups, Hardlinks |
| `Scrubbed` | `Plain` | wie `Physical` + Überschreiben vor Unlink | einfache Wiederherstellung klassischer Datenträger | CoW-Dateisysteme, SSDs (best-effort) |
| `Cryptographic` | `Sealed` | Gate, **Clusterschlüssel vernichten**, Purge (ohne Redo-Barriere) | Reste in Redo, Backups, Flash, sofern der Schlüssel nirgends sonst liegt | Schlüsselkopien |

| `Stage` (Fortschritt) | Bedeutung | Garantie (alt) |
|---|---|---|
| `Gated` | Gate durabel, Cluster `Retired(S)` | G0 |
| `KeyDestroying{eta}` / `KeysDestroyed` | Schlüssel unbenutzbar / `Absent` | G1 |
| `Purging` / `Purged` | Bytes aus lebenden Dateien entfernt | G2 |
| `Done` | Ledger durabel, Anker fortgeschritten, Receipt | — |

Die Stufe muss zur Schutzklasse passen, sonst `Err(Unsupported)`: **`Sealed` ⇒ `Cryptographic`, `Plain` ⇒ `Physical`/`Scrubbed`.** Das Receipt spricht nie von „unwiderruflich“ oder „konform“.

### 9.3 Garantie-Kernsätze

1. Ab durablem Gate liefert kein `view`/`page`/`poll` Inhalt des Clusters und kein Commit schreibt hinein; pro Cluster gilt voller Snapshot oder `Erased`, nie Mischsicht (`INV-ERA-1`).
2. Nach `KeysDestroyed` ist alles des Clusters in Redo, Runs, Blobs, Checkpoints und Backups ohne Schlüssel unlesbar (`INV-ERA-7`).
3. Nach `Purged` enthält keine lebende Datei Bytes des Clusters (`INV-ERA-2`).
4. Gate/KeyDestroy-Arbeit skaliert mit der Zielmenge (zuzüglich Provider-Latenz). Physischer Purge bearbeitet nur Dateien/Extents mit Zielcluster; seine Byte-Kosten sind durch deren Zahl und Größe sowie Retention-Pins begrenzt, nicht generell store- oder clustergrößenproportional. `ClusterLocal` vermeidet fremde Survivor-Rewrites in Runs/Blobs, nicht zwingend im gemeinsam genutzten Redo. Externe globale Rebuildable-Indizes haben eigene Frist/Kosten und werden separat ausgewiesen.

### 9.4 Schlüssel und `KeyProvider`

```
KEK (KeyProvider, extern)  └─ RootKey (32 B, in KEYS mit KEK umhüllt)
       ├─ Space-DEK (je Space, mit RootKey umhüllt)
       ├─ Redo-/Catalog-/FORMAT-/Ledger-Schlüssel = KDF(RootKey, Label)
Clusterschlüssel: KeyProvider::cluster_key(cluster, epoch)      (NICHT in KEYS)
```

```rust
pub trait KeyProvider: Send + Sync {
    fn info(&self) -> KeyProviderInfo;     // { destroy: Immediate | Scheduled{max_delay}, keys_in_backups: bool }
    fn kek(&self, id: KekId) -> Result<Key32, KeyError>;
    fn create_cluster_key(&self, c: ClusterId, e: &dyn Entropy) -> Result<u32, KeyError>;   // epoch, durabel, idempotent
    fn cluster_key(&self, c: ClusterId, epoch: u32) -> Result<Option<Key32>, KeyError>;
    fn destroy_many(&self, cs: &[ClusterId]) -> Result<DestroyReceipt, KeyError>;           // ALLE Epochen, idempotent
    fn probe(&self, c: ClusterId) -> Result<KeyState, KeyError>;                            // Present | Disabled | Absent
    fn list(&self) -> Result<Vec<ClusterId>, KeyError>;
}
```

- **`Disabled`** = unbenutzbar, nicht unumkehrbar (KMS-Karenz). ER2 gilt erst bei `Absent` als abgeschlossen; bis dahin `KeyDestroying{eta}`, das Receipt trägt `CompleteWithResidual`. Der Purge darf bei `Disabled` laufen, `Done` braucht `Absent`.
- `Cryptographic` nur bei `info().keys_in_backups == false`, sonst `Err(Unsupported)`. Das Receipt nennt `destroy` und `keys_in_backups`.
- KEK-Rotation: nur RootKey neu umhüllen (`rotate_kek`), alle anderen Dateien bleiben. Rotation einer Cluster-Epoche: neue `key_epoch` für neue Writes; eine alte Epoche wird erst vernichtet, wenn nichts mehr mit ihr versiegelt ist (`INV-KEY-3`); `destroy_many` vernichtet **immer alle** Epochen (`INV-KEY-4`).
- **Cohort-Strategie (Provider-intern, K-04):** `keys-cohort` darf Clusterschlüssel in Cohorts (≤ 1024) unter einer Cohort-KEK-Generation führen; `destroy_many` = Rewrap ohne die gelöschten Einträge, dann KMS-Destroy der alten Generation. Voraussetzung: **keine Keyring-Kopie in Backups** (`keys_in_backups = false`), Pflicht-Replikation des aktuellen Keyrings in ≥ 2 Failure Domains. Der Kern sieht davon nichts.
- Schlüsselcache mit Generationszähler (§11.5), TTL, `Zeroizing`, Providerausfall: gecachte Schlüssel bis `key_grace` (10 min); Gate und ER2 invalidieren sofort durch `gen`.
- Konformanzsuite `KeyProviderConformance` (Idempotenz, alle Epochen, `probe`-Semantik, Crashverhalten) ist Gate-Pflicht für jeden Provider.

### 9.5 Protokoll `Eraser::cluster` (ER0–ER7)

```
ER0 Vorprüfung:  Cluster existiert; Link-Hülle H = {c} ∪ transitiv DerivedFrom-gebundene Cluster;
                 BLOCK_ERASE-Restriction in H ⇒ Err(Restricted{ids,reasons}); level passt zur Protection; Provider erreichbar
                 und info() erlaubt Cryptographic; bereits Retired ⇒ vorhandenes Ticket (ggf. escalate)
ER1 Gate:        Committer-Op: Gate-Frame{scope=Clusters(H), ticket, level}, Gruppen-Fsync          ◄── LOGISCHER Punkt (G0)
                 Committer berechnet Hülle und Restrictions NEU (INV-ERA-10); wuchs die Hülle ⇒ Retry ab ER0.
                 Danach: cluster_table ← Retired{S}, gen++. erase() kehrt mit Ok(ticket) zurück; ab hier nie mehr ein Fehler.
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

**Reihenfolge ist Vertrag:** ER1 < ER2 < jedes `PurgeBatchEdit` < zugehöriger Unlink < finales ER6/`Done`; bei `Cryptographic` beginnt physischer Purge nie vor `probe == Absent` (`INV-ERA-3`). Jede Batcherweiterung ist crash-sicher und roll-forward; nach finalem Verify gibt es keine lebende Datei/Inkarnation mit H. Das Gate liegt im Committer, weil ein vor dem Gate geprüfter Change nach dem durablen Gate nicht sichtbar werden darf. Es schneidet kein Segment: der Commit-Pfad bleibt frei von Löschkosten.
**Frist:** `purge_deadline` ist **Pflichtparameter** (kein Standard). Ticket speichert Deadline, monotone Dauer und `clock_floor`. Rücksprung wird geklemmt; ein UTC-Sprung außerhalb `max_clock_skew` friert Retention/Deadline-Berechnung ein und ergibt `ClockUncertain` mit Alarm, statt Holds vorzeitig zu beenden oder Compliance-Frist zu simulieren. Überfällige, zeitlich verifizierte Tickets erhalten `urgent`; echte Überschreitung meldet `erase_deadline_breaches_total` und `Blocked{reason=Deadline}`.
**Bündelung:** Purge-Auslöser: Frist zu 70 % verbraucht, ≥ `purge_batch_min` (16) Cluster oder anstehende Compaction in der Datei.
**Views:** `ViewPolicy::Wait(timeout)` (Unlink wartet; logische Garantie gilt bereits) oder `Revoke` (§11.4).
**Done-Bedingungen (`INV-ERA-8`):** (1) keine lebende Datei/Segment/Inkarnation trägt Daten von H (bei Cryptographic unlesbar), (2) Memtable-Einträge freigegeben, (3) Ledger durabel, (4) Anker fortgeschritten, (5) alle Segmentbesitzer bestätigt, (6) bei Cryptographic `probe == Absent`.

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

### 9.6 Restriction, Hold, Links

- **Restriction:** `RestrictionSpec { reason u16 ≠ 0, owner, effects (BLOCK_ERASE immer | BLOCK_WRITE | BLOCK_READ), expires }`. Ablauf lässt nur den Vermerk fallen, löst nie Löschung aus. **Kein Override im Kern.** `lift` nach `LiftPolicy`: `OwnerOrAdmin` (Standard) oder `FourEyes` (Port `Authority`, Signatur einer Principal ≠ Requester). `restrict`/`lift` laufen als Control-Frame durch den Committer, in derselben Stamp-Reihenfolge wie das Gate (`INV-ERA-10`). Durchsetzung dreischichtig: Committer-Prüfung (ER0/ER1), Purge verweigert `files_remove`/Umbau für Cluster mit BLOCK_ERASE, `cluster_state`-Edit verweigert den Übergang nach `Retired` für Restricted-Cluster. Ein Erase blockiert **sofort** mit `Err(Restricted)` (kein Zwischenzustand); will die obere Schicht den Cluster währenddessen verbergen, setzt sie selbst `BLOCK_READ`.
- **`BLOCK_READ`:** Lesen ⇒ `Err(Restricted)`; zweckgebundene Lesung nur mit nicht klonbarer `RestrictedReadCap` (Observer-Ereignis mit Zweck-Code, nie in Cache oder Consumer-Feed).
- **Hold:** technisch, befristet (`ttl ≤ 24 h`); ein Erase überstimmt einen Hold ohne Fehler.
- **Links:** `Clusters::link(derived, sources[≤ 8])` (azyklisch, Tiefenlimit). Löschung irgendeines Quellclusters zieht den abgeleiteten **im selben Gate-Frame** mit (`INV-ERA-12`): es gibt keinen Zustand, in dem die Quelle gelöscht ist und ein Abgeleiteter lebt. Ein Kind mit mehreren Eltern wird konservativ gelöscht, sobald **eines** seiner Eltern gelöscht wird. Darum darf ein gemeinsam von mehreren Personen/Löschdomänen genutzter Cluster nicht als `DerivedFrom`-Kind einer einzelnen Person angelegt werden; pro Löschdomäne getrennte Ableitungen oder nicht-kaskadierende `Reference` verwenden. Konformanzprüfung weist solche cross-subject Kaskaden zurück.

### 9.7 Ledger, Anker, Receipt

- **Ledger:** append-only, vorallokiert, kettenverknüpft über `Mac`, **ein Schreiber**. Record ist ein kanonischer tagged union: `{seq, prev_hash, clock_sample, event, mac}`; Felder sind pro Event-Variante festgelegt, keine Null-/Platzhalter-Clusterfelder. `EraseEvent` enthält nur pseudonymes `tomb_id = HMAC(ledger_key, ClusterId)`, Ticket, Level, Stamp/Stage und Evidence-Digest. Es enthält **nie ClusterId, SpaceId, Nutzdaten, Nutzerschlüssel oder Person-ID** (`INV-LED-1`). Event-Kinds: `Requested, Retired, KeysScheduled, KeysDestroyed, KeyLost, EraseBlocked, PurgeProgress, UnlinkProgress, ExtAck, PurgeVerified, Completed, RestrictionSet/Lifted, BranchEraseDone, SpaceDropped{space_tomb_id,scope_digest,count}, RestoreReconciled, ReceiptIssued`. `space_tomb_id=HMAC(ledger_key, SpaceId‖generation)`; `SpaceDropped` steht zusätzlich zu den einzelnen Cluster-Events und ersetzt sie nicht. Reihenfolge Catalog-Edit → Ledger-Append/Sync → Anchor-CAS → Receipt; jede Zustandsfortschreibung ist crash-idempotent und vor dem nächsten ER-Übergang durabel. `ledger_dir` konfigurierbar (K-22).
- **Anker:** `AnchorValue = (erase_epoch, ledger_head, catalog_epoch)`. Fortschreibung **nie vor dem durablen Catalog-Edit**, mindestens (a) vor jedem Receipt, (b) bei Catalog-Rollover, (c) bei `close`, (d) nach `anchor_max_age` (1 s) bei ausstehendem Zustand; koalesziert. Ein Rollback im Fenster (a)–(d) kann ein Ticket, nie ein Receipt rückgängig machen. `Required` ist Standard bei Encrypted/Authenticated; `Weak`-Anker verlangt `acknowledged_weak` und wird im Receipt vermerkt. Referenz: `MemAnchor` (Test), `FileAnchor` (`Weak`); `Strong`-Anker (TPM, Transparency-Log, Object-Lock) als getrennte Crates.
- **`ErasureReceipt`:** `{req/ticket, scope_digest (BLAKE3 über sortierte tomb_ids), level, t_requested, t_logical, erase_epoch S, t_crypto?, t_physical?, t_verified?, stages[{stage, ledger_seq, evidence_digest}], key_attestations, provider{destroy, keys_in_backups}, ext_acks, verifier_report_digest, ledger_head, anchor{strength, value}, restrictions_encountered, status (Complete | CompleteWithResidual | Blocked | Failed), residual[§9.9], canonical_bytes}`. **Die Signatur liegt oben (NZ-14);** der Kern liefert kanonische Bytes und die reine, offline lauffähige Funktion `verify_receipt(receipt, ledger_slice, anchor_attestation) → Verdict` (Kette, Anker, Stufenreihenfolge, Zeitmonotonie).
- **`Eraser::verify`:** strukturell, ohne Schlüssel: (a) keine lebende Datei/Segment/behaltenes Redo-Segment trägt ClusterDirectory-/Frame-Kopf-Eintrag für H, (b) Memtables ohne H, (c) bei Cryptographic `probe == Absent`, (d) Ledgerkette bis zum Anker gültig. Inhaltssuche ist Sache des Orakels.

### 9.8 Weitere Löschpfade

- **`Eraser::range(space, cluster, range)`:** keine Branch-Argumente; `EraseRange` ist ein clusterweiter Gate-Marker, der nach durablem Frame sofort alle Branches und auch ältere Views maskiert. Für `Plain` sind `Logical|Physical|Scrubbed` verfügbar; `Physical` rewritet alle Branch-/Stamp-Versionen des Bereichs aus lebenden Run-/Blob-Dateien. `Scrubbed` ist best-effort und kein SSD-/CoW-Beweis. Für `Sealed` ist ausschließlich `Logical` zulässig: selektive Range-Key-Vernichtung existiert in v1 nicht, und ein Rewrite allein macht Backup-Kopien mit demselben Cluster-Key nicht unlesbar. Für physische/kryptografische Entfernung eines Sealed-Bereichs ist der ganze Cluster zu löschen oder der Bereich vorab in eine eigene Cluster-Löschdomäne zu legen. `EraseRange` kann nie per Merge aufgehoben werden. `erased_ranges` dürfen erst getrimmt werden, wenn `history_floor > erase_stamp` und keine Branch-/View-/Consumer-Pins darunter liegen.
- **`Eraser::branch(cluster, branch)`:** nur terminale, leaf-Branches. Für `Plain` sind `Logical|Physical|Scrubbed` analog zu `range` möglich; physisch werden alle Versionen dieses Branches aus lebenden Run-/Blob-Dateien entfernt. Für `Sealed` nur `Logical`, da die Cluster-Keydomäne mit anderen Branches geteilt wird. Eine Crypto-Anforderung wird als Cluster-Erase ausgeführt; es gibt keine branchlokale Schlüsselvernichtung.
- **`Storage::space_drop`:** atomarer Admissionschritt unter Sequencer-Ordnung friert Cluster-Menge und Space-Generation ein; jeder neue Write/Register wird danach abgewiesen. Vor dem Gate werden alle aktiven Cluster auf `BLOCK_ERASE` geprüft; eine Restriction blockiert den gesamten Drop ohne Teilwirkung. Ein `Gate{Space(id)}` setzt `Dropping(S)` und macht sämtliche Inhalte logisch unsichtbar. Anschließend läuft das normale Erase-Protokoll pro Cluster (Schlüssel je Cluster vernichten, Plain-Barriere, Dateien/Extensions purgen); der Space-DEK wird nur vernichtet, wenn er ausschließlich zu diesem Space gehört und keine andere Datei ihn referenziert. Catalog führt `SpaceDropped` erst nach allen Cluster-Tickets und `UnlinkAck`s; Ledger enthält für Restore je Cluster einen `tomb_id` plus Aggregat `SpaceDropped{space_tomb_id,scope_digest,count}`, wobei `space_tomb_id=HMAC(ledger_key, SpaceId‖generation)` keine Klartext-SpaceId offenlegt. Kein einzelner Catalog-Edit versucht alle Dateien umzuschreiben. `SpaceId` wird nie wiederverwendet.
- **`pack_expired(PackKey)`:** `H` = nicht Restricted-Cluster der Packgruppe. Der Ablauf zerlegt H deterministisch in unabhängige Tickets mit höchstens `max_gate_clusters=4096`; jedes Ticket hat einen eigenen Gate-Frame und eigene Receipt-/Ledger-Kette. Es gibt keine behauptete atomare Gesamtlöschung der Packgruppe. Restricted-Cluster werden mit Grund im Aggregate ausgewiesen und separat erneut versucht, wenn die Restriction aufgehoben wird.
- **`escalate(ticket, level)`** (z. B. Logical → Cryptographic), **`open_tickets()`** (nach Absturz des Aufrufers), **`clusters_batch`**.
- **Mehrbetroffene Daten (K-25):** gemeinsamer Fallcluster; jede betroffene Person hat ihren Wurzelcluster; Verknüpfung per Link nur, wo die Löschung der Person den Fallcluster mitziehen **soll**; sonst Referenz. Keine mehrclustrigen Einträge.

### 9.9 Restexposition (Pflichtbestandteil jedes Receipts, vom Orakel geprüft)

| Bereich | Restrisiko | Zuständig |
|---|---|---|
| Prozessspeicher, Swap, Core-Dateien, noch nicht reklamierte Skiplist-Knoten | Klartext-Reste | `Zeroizing`, Cache-Drop beim Gate; `MemLock`/`DONTDUMP` im Adapter; Betreiber: Swap verschlüsseln, Core-Dumps aus |
| Dateisystem-Journal, Flash-Wear-Leveling, Volume-Snapshots, externe Backups, Checkpoints mit bereits `Retired` Plain-Daten | Alte Blockversionen | nur Ciphertext bei Sealed; Plain: Schutz fehlt, Betreiber begrenzt Aufbewahrung |
| KMS-Karenzzeit | Schlüssel wiederherstellbar bis Fristende | Stage `KeyDestroying`, Status `CompleteWithResidual` |
| Schlüsselkopien (Escrow, HSM-Backups) | G1 hinfällig | `keys_in_backups`; Betreiber |
| Metadaten | ClusterId (Pseudonym, bis Purge von Redo-Gate-Frames), Anzahl, Größen, Zeitpunkte, Packgruppe | zufällige IDs; Ledger nur `tomb_id`; optional Padding auf Bucket-Größen |
| Rollback über den Anker hinaus / im Ankerfenster | Ticket rückgängig | Anker-Stärke im Receipt |
| Obere Schichten (Prompt-Caches, Embedding-Kopien, Logs, Fine-Tuning-Daten, Modellgewichte, Provider-Logs) | **nicht** im Kern lösbar | nur Daten, die über Links und `SegmentOwner` angebunden sind, werden mitgelöscht |
| Verzeichnis Person → Wurzelcluster (§12.7) | nur physisch löschbar | Muster in §12.7 |

---

## 10. Recovery, Checkpoint, Restore

### 10.1 Öffnen (deterministisch)

```
R1  LOCK (exklusiv; ReadOnly geteilt)
R2  FORMAT-Kopf lesen → KEYS entpacken (KEK ⇒ RootKey) → FORMAT-MAC prüfen
R2a Anker: Catalog-Epoche < Anker ⇒ Err(Rollback). Ledger muss mindestens E_a Einträge haben und Eintrag E_a den Hash H_a tragen.
    Catalog/Ledger neuer als Anker ⇒ Anker nachziehen. AnchorPolicy::Required und Anker unerreichbar ⇒ Err(AnchorUnavailable).
R3  CURRENT → Catalog bis zum letzten gültigen Record (Torn Tail verwerfen)
R4  Verzeichnisse scannen: Dateien ohne Catalog-Eintrag = verwaist (noch nicht löschen)
R5  KeyProvider prüfen; KEK fehlt ⇒ Modus LockedKeys
R6  Redo ab replay_floor, zwei Durchläufe:
      1: Kette prüfen, Gate-/Control-Frames sammeln ⇒ retired_set = Catalog ∪ Gate-Frames
      2: Frames anwenden; Abschnitte retired Cluster überspringen (nie entsiegeln); Schlüssel eines NICHT retired Clusters fehlt
         ⇒ Corrupt{KeyMissing}; Branch-/Merge-Frames anwenden
R7  Memtable aufbauen; visible = durable = letzter gültiger Stamp; next_stamp nach §6.2; Branch-Tabellen lazy
R8  Offene Löschungen = erase_pending ∪ Gate-Frames ohne erase_done ∪ Retired/Shredded ≠ Purged ∪ Space Dropping
    ⇒ in urgent ab der ersten offenen Stufe fortsetzen (idempotent); ER4 neu
R9  Verwaiste Dateien und .building.* entfernen (nach R8, nie davor)
R10 Reserven (Stamp, ID, Redo-Segment, RESERVE) herstellen, Wartung starten, OpenReport
```

Ehrliche Grenze: Ein beschädigter letzter quittierter Gruppenabschnitt hinter der letzten `redo_mark`, dem nichts Gültiges folgt, ist von einem Torn Write nicht unterscheidbar; die Heilung kürzt dann. Das ist die einzige Stelle, an der Bitfäule `durable = true` verletzen kann; `redo_mark` hält das Fenster klein.

### 10.2 Absturzmatrix (Auszug, vollständig in der Testsuite)

| Unterbrechung | Soll |
|---|---|
| vor Redo-`sync` | Change nicht sichtbar, `apply` kehrte nicht zurück |
| nach Redo-`sync`, vor Rückkehr | Change vorhanden (Token verhindert Doppelung) |
| Run-Bau / vor Edit / vor Unlink | `.building` entfernt bzw. Run verwaist/entfernt, Redo spielt ein |
| mitten in einer Catalog-Gruppe | gültiger Präfix gilt |
| Gate durabel, Catalog-Edit fehlt | `Retired` aus Frame, Edit nachgeholt |
| Gate durabel, Schlüssel noch da | Roll-forward |
| Schlüssel vernichtet, `Shredded`-Edit fehlt | `probe` ≠ Present ⇒ Edit nachholen |
| Plain-Barriere unterbrochen | ER4 neu, idempotent |
| Purge-Edit durabel, Unlink offen | Verwaisten-Suche |
| Edit durabel, Anker alt | Nachzug in R2a |
| Branch-Fork-Frame durabel, Memtable nicht | Replay stellt Branch-Tabelle wieder her |
| Merge-Frame durabel, `source`-Zustand nicht | `Merged` aus Frame abgeleitet |
| `DiskFull` beim Vorallokieren | sauberer Fehler, keine Teilschreibung |
| Checkpoint abgebrochen | `dst` unvollständig ohne Siegel, Quelle unverändert |

### 10.3 Checkpoint, Restore, Rollback-Schutz

- **Checkpoint:** (1) Flush-Barriere; (2) unter `catalog_edit` Version pinnen, `erase_epoch` und Ankerwert lesen; (3) Checkpoints innerhalb der physischen Purge-/DiskFull-Garantie verwenden unabhängige `Copy`-Dateien; (4) Manifest siegeln. `Reflink`/`Hardlink` sind nur zulässig, wenn ein registrierter Checkpoint-Owner vor Scratch-Recycling Referenzen auf betroffene Extents nachweislich entfernt und attestiert. Andernfalls gelten sie als nicht verwaltete Retention-Copies und erscheinen als Restrisiko im Receipt. `KEYS` ist enthalten, **KEK und Clusterschlüssel nicht** (`INV-KEY-1`). Das Ledger liegt nie im Checkpoint. Das Manifest listet bereits `Retired`, aber noch nicht gepurgte Plain-Cluster.
- **Restore** ersetzt die Instanz (kein zweiter lebender Klon, NZ-19): Der Kern startet in `RestoreQuarantine` (keine Nutzer-Reads), wendet **alle Ledger-Einträge nach `erase_epoch` erneut an** (`INV-ERA-9`): Er berechnet `tomb_id` aller Clusterverzeichnisse der restaurierten Dateien und gleicht mit den Tomb-Einträgen ab; Treffer werden `Retired`, per Purge-Rewrite entfernt, ihre Schlüssel sind bereits vernichtet. Restrictions kommen aus dem Ledger-Anhang. Liegt das Manifest hinter dem Anker, wird es als Rollback verweigert. Erst nach `RestoreReconciled` startet `Running`.
- **Ehrliche Grenze:** Wird zusätzlich der Anker-Dienst zurückgesetzt, ist Schutz nur noch kryptografisch (Schlüsselvernichtung), nicht mehr protokollarisch.

---

## 11. Nebenläufigkeit

### 11.1 Prinzipien

Ein Schreiber (Committer) für stamp-relevante Zustände; unveränderliche Dateien und `Arc<Version>`; Besitz durch Claims; Wahrheit durch durable Edits/Frames; Generationszähler statt Sperren für Schlüsselcache, Blockcache, Clusterzustand; keine Sperre über E/A außer Committer (Redo-Sync) und `catalog_edit` (Catalog-Sync); Ledger/Anker haben je einen Schreiber über Kanäle.

### 11.2 Sperrränge (nur aufsteigend, je Thread höchstens einmal, `debug_assertions` prüft per Rangstack)

| Rang | Sperre | Über E/A | Bemerkung |
|---|---|---|---|
| 1 | `committer` | ja (Redo-Sync) | Stamp-Vergabe, `Live → Retired`, Branch-Ops |
| 2 | `intake` | nein | |
| 3 | `catalog_edit` | ja (Catalog-Sync) | Gruppen-Commit |
| 4 | `state` (`RwLock<Arc<Version>>`) | nein | Veröffentlichung |
| 5 | `registry` (Holds, Consumer, Claims, Restrictions, `ViewInner`) | nein | Blatt |
| 6 | `cluster_table` (sharded `RwLock`, inkl. Branch-Tabellen) | nein | Blatt, Lesen aus Rang 1 zulässig |
| 7 | `maint` (je Lane) | nein | Blatt |
| 8 | `notify` | nein | Blatt |

Rang 1 + 3 gleichzeitig nur im Reservierungs-Fallback. Kein Rückruf (`Observer`, `SegmentOwner`, `KeyProvider`) unter Sperren, **insbesondere kein Provideraufruf unter Rang 1**. Memtable lock-frei. Kein `std::sync::Mutex` außerhalb `sync::Ranked`.

### 11.3 Lock-freie Pfade

`ArcSwap`/`RwLock<Arc<_>>` für `Version`; Reihenfolge Publish: Version, dann `visible_stamp` (Release); Leser laden Stamp (Acquire), dann Version. Cluster-Zustand = `AtomicU64 (state:8 | gen:56)`; nur der Committer setzt `Live → Retired` (`INV-LOCK-2`); spätere Übergänge sind monoton und laufen per `catalog_edit`.

### 11.4 Views und Graveyard

`View` hält `ViewInner { version: RwLock<Option<Arc<Version>>> }`. `ViewPolicy::Revoke` setzt `None`; spätestens nach `revoke_grace` (5 s) hält keine widerrufene View eine Referenz (`INV-GRAVE-2`). Eine Datei wird erst gelöscht, wenn der entfernende Edit durabel **und** die letzte `Arc<Version>` gefallen ist (`INV-GRAVE-1`). `files_remove` hält sie bis zum erfolgreichen `unlink + sync_dir` im persistenten Graveyard; erst `unlink_ack` räumt den Eintrag aus. Unlink führt Wartung aus, nie `Drop`.

### 11.5 Claims

`claim(files, job)` atomar unter `registry`; Priorität `urgent > flush > compact > blobgc > idle`; höhere Priorität setzt `cancel`, der Job prüft zwischen Blöcken; Ausgabe-Edit **abgelehnt**, wenn `files_remove ⊄ Claims` (`INV-CLAIM-1`). Compaction notiert `retire_gen`; Ausgabe wird verworfen, wenn ein Eingabecluster inzwischen `Retired` ist.

### 11.6 Race-Nahtstellen

| ID | Race | Lösung | Nachweis |
|---|---|---|---|
| R1 | Lesen × Gate | Gate-Check am Anfang, `gen`-Check nach Entsiegeln | loom |
| R2 | Flush × Erase | Gate vor Memtable-Einfügung, Flush überspringt Retired, ER4 | M2, Sim |
| R3 | Compaction × Erase | Claims, `retire_gen`-Prüfung im Edit | M3, Sim |
| R4 | Cache × Vernichtung | `gen` vor und nach Entsiegeln | loom |
| R5 | Checkpoint × Erase | `Arc<Version>` + `erase_epoch`, serialisiert gegen ER6 | M4, Sim |
| R6 | Consumer × Erase | `retain_floor`, `Lagged`, `Erased`-Change | M2 |
| R7 | Rebucket × Lesen | Gruppe im selben Edit | M5 |
| R8 | Zwei Erase, überlappende Hülle | Serialisierung nach Ticket-ID in `urgent` | Sim |
| R9 | Restriction × Erase | beides im Committer, eine Stamp-Reihenfolge | M2 |
| R10 | Link × Erase | Hülle im Committer neu berechnet, Retry | M2 |
| R11 | Guard × Gate | Zustandsprüfung (5a) vor Guard (5b) | M2 |
| R12 | Anker × Edit | Anker nie vor Edit, Nachzug in R2a | M6 |
| R13 | Staging × Erase | `purge` behandelt Staging wie Packed | M3, Sim |
| R14 | Space-Drop × Flush/Compaction | Claims, `Dropping` bricht Jobs ab | M2, Sim |
| **R15** | **Merge × Gate/Restriction** | Merge ist ein Change; Zustandsprüfung 5a gilt für Quell- und Zielcluster (derselbe Cluster) | M2 |
| **R16** | **Fork × Compaction** | `floor` berücksichtigt lebende `fork_stamp`s | M3 |
| **R17** | **Branch-Drop × Merge** | beides im Committer, serialisiert; Merge auf gedroppten Branch ⇒ `BranchClosed` | M2 |

---

## 12. Erweiterungsverträge und Muster für die obere Schicht (RAG, Agenten)

### 12.1 Segment-Register

```rust
pub trait SegmentOwner: Send + Sync {
    fn kind(&self) -> SegmentKind;
    fn manifest(&self, seg: &SegmentMeta) -> SegmentManifest;       // class, key scope, bounds, RAM/scratch, lineage
    fn on_gate(&self, erased: &[ClusterId], at: Stamp, gen: u64);   // synchron, nicht-blockierend: Query-Filter aktivieren
    fn rewrite_without(&self, seg: &SegmentMeta, drop: &ClusterSet, io: &mut SegmentIo) -> Result<RewriteOutcome, SegErr>; // Pflicht
    fn verify_absent(&self, scope: &EraseScope, gen: u64) -> Result<EraseAttestation, SegErr>;
    fn watermark(&self) -> DurableStamp;                           // lückenlos verarbeiteter durable Feed
}
```

`SegmentClass` ist `ClusterSections | MixedPerClusterKey | MixedPerSubjectKey | Rebuildable{max_age,max_rewrite_bytes} | NoPersonalData`. `ClusterSections` liegen als cluster-sealed Blocks im Kern. `MixedPerClusterKey` nutzt je Cluster-Key-Epoche. `MixedPerSubjectKey` ist für IdentityDirectory-Einträge mit separatem EntryDEK. `Rebuildable` hält personenbezogene Nutzdaten nur verschlüsselt gemäß deklarierter Scope und bleibt bis Rebuild+Verify `Purging`. `NoPersonalData` benötigt versionierte Provenienz-/Schema-Evidence und keine bloße Selbstdeklaration.

`manifest` wird vor erster Nutzung registriert und versioniert. Jeder Owner-Aufruf läuft mit Deadline und Fencing-Generation außerhalb Kernlocks. Timeout/Owner-Ausfall ⇒ Ticketstatus `Blocked{owner, reason, since, retry_at}`, Ledger-Ereignis `ExtBlocked`, Alarm und Eskalation; verspätete Antworten alter Generation werden verworfen. `Blocked` zählt nie als `Done`; Replacement/Unregister ist ledger-protokolliert und muss mit neuer Generation `verify_absent` erfüllen. `RewriteOutcome` allein ist kein Erase-Beweis: nur `EraseAttestation{owner, generation, scope_digest, applied_stamp, keys_destroyed, segments_rewritten, evidence_digest}` mit erfolgreichem `verify_absent` kann ein Ack liefern. Zugriff nur über das versiegelte `SegmentIo`; Owner darf keine direkten Kernpfade öffnen. Jedes Extension-Job-RAM/Scratch-Byte muss vor Allokation per `MemoryBudget`/`SegmentIo` reserviert sein. Segmente speichern Daten bevorzugt **als Cluster-Sektionen im Kern** (eigener Space): dann ist `rewrite_without` trivial.

### 12.2 ExtensionBlock

Opake Blöcke am Run (Tag `u16`, Version `u16`) je Cluster-Abschnitt; `ExtensionBuilder` erhält den Eintragsstrom bei Flush/Compaction; `merge` und `rewrite_without` sind Pflicht; unbekannter Tag beim Rewrite ⇒ Fehler. Ein Besitzer ohne `rewrite_without` darf in `clustered`-Spaces keinen Block anlegen.

### 12.3 Change-Feed (Consumer)

Gespeist aus dem **Redo-Log** (nicht aus Runs: diese sind nicht stamp-geordnet). Zwei Böden: `replay_floor` (Recovery-Start) und `retain_floor ≤ replay_floor` (älteste für Consumer behaltene Segmente), `retain_floor = min(replay_floor, kleinste Consumer-Position, Grenze aus max_lag_bytes)`. Consumer persistiert Fortschritt, pinnt nur Redo-Segmente, `max_lag_bytes` 1 GiB; Überschreitung oder Plain-Barriere ⇒ `Err(Lagged)` (Aufholen per Vollscan oben). Reihenfolge nach Stamp, mindestens einmal, nur Stamps ≤ `durable_stamp`. Nutzlasten von Sealed-Clustern entsiegelt der Kern; **nach dem Gate erscheint stattdessen `Change::Redacted{cluster}` bzw. nach ER2/Purge `Change::Erased{cluster|range, at}`**; BLOCK_READ-Cluster erscheinen als `Restricted`-Platzhalter. Branch-Frames erscheinen mit `branch`-Feld. Volatile-Spaces erscheinen nicht. Ein persistierender Consumer, der RAG-/Graph-/Cache-Derivate erzeugt, muss als `SegmentOwner`/Extension registriert sein und Quell-Cluster-Lineage für Erase attestieren; nicht registrierte externe Persistenz wird als Receipt-Restrisiko ausgewiesen und verhindert systemweites `Complete`.

### 12.4 Index-Watermarks (Pflichtmuster für Indexbesitzer, aus CairnDB)

Jeder abgeleitete Index führt `applied_stamp = watermark()`. Eine Abfrage unter View-Stamp `V` durchsucht **Basisindex ∪ Delta (Stamps `(applied_stamp, V]`)**, validiert jeden Kandidaten gegen die View (`filter_visible`) und kennt Mandanten-/Rechtefilter **vor** der ANN-Traversierung (keine Existenz-Seitenkanäle über Kandidatenlisten). So entsteht kein Suchloch, solange ein Hintergrundjob hinterherläuft, und kein Index kann Gelöschtes ausliefern (`INV-VIS-1`).

### 12.5 Kopf, Gewichte, Budget

`header`, `weights`, `page`, `weight_sum` sind die Schnittstelle für Rang- und Budgetlogik. Die obere Schicht legt fest, was ein Gewicht bedeutet (Token je Tokenizer-Familie: je ein Slot, bis 4). Token-Budget-Packing (Knapsack, Redundanzabzug, präfix-stabile Ordnung) liegt oben und nutzt `page` für die Kandidaten.

### 12.6 Konventionen (keine Kernfunktionen)

| Thema | Konvention |
|---|---|
| **Zoom-Stufen** | Schlüsselsuffix je Stufe im selben Cluster (`doc/<id>/L0`, `/L1`, `/L2`); L1/L2 im Kopf, L0 im Körper |
| **Trust-Label** | erstes Kopfbyte `trust ∈ {system,user,tool_verified,tool_unverified,untrusted}`; Abgeleitetes erbt das Minimum seiner `DerivedFrom`-Quellen; Hochstufen nur privilegiert mit Audit |
| **Lineage/Derivation** | abgeleitete Cluster per `link(derived, sources)`; Kopf trägt `(producer_id, source_stamp)`; veraltet = `source_stamp` ≠ aktueller Quellstamp; Cache nur **je Tenant, nie über Löschdomänen** |
| **Handles** | kurze IDs (4–6 Zeichen) sind Schlüsselaliase innerhalb des Clusters (Alias-Space); „Meintest du …?“ per Präfixscan im Cluster |
| **Fehlertext für Agenten** | `Error::hint()` plus `suggested_call` in der Fassade oben |
| **Bitemporalität** | `valid_from/valid_to/observed_at/source_time` im Kopf; `supersedes`-Kante als Schlüssel `edge/supersedes/<id>` |
| **Embeddings** | Segmente (Index) oder Cluster-Space `emb/<model_id>`; Modellwechsel = neuer Space/Segment, alte Version bleibt lesbar bis Migration |
| **Work Queue / Sleep Cycles** | Consumer + Tasks als Daten im Cluster; Ergebnis = normaler `Change` mit Producer im Kopf |
| **Vergessen als Zoom-Down** | Löschauftrag der oberen Schicht; keine stille Entfernung ohne `Eraser`-Aufruf |
| **KV-/Prompt-Cache** | außerhalb; wer Prefix-Daten persistiert, legt sie als Segment mit `rewrite_without` ab |

### 12.7 Person → Cluster (normatives Muster)

1. **Wurzelcluster je Person** (`Sealed`, `personal`-Space): enthält die Liste der Dokument-Cluster (Space `docs_index`). Dokument- und abgeleitete Cluster hängen per `link(derived, [wurzel])` an der Wurzel. **Eine Löschung der Wurzel zieht alle Cluster-Nachfahren im Gate mit.**
2. **IdentityDirectoryOwner oberhalb des Kerns:** Die Zuordnung `person_id → root ClusterId` liegt nicht in einem `Plain`-Space. Der Directory-Key ist `HMAC(K_dir_tenant, canonical_person_id)`; `K_dir_tenant` bleibt ausschließlich im KMS, nie in Backups oder Storage. Der Directory-Wert `ClusterId` ist zusätzlich mit einem eigenen, nicht exportierbaren `EntryDEK` je Person verschlüsselt. Für Löschung nach bekanntem Cluster hält der Owner einen Reverse-Lookup `HMAC(K_reverse, ClusterId) → DirectoryKey`; `K_reverse` bleibt ebenfalls im KMS. `EntryDEK`-Key-IDs werden deterministisch aus `HMAC(K_keyid, ClusterId)` gebildet und nie persistiert. `IdentityDirectoryOwner` ist ein registrierter `SegmentOwner` mit `SegmentClass::MixedPerSubjectKey`, Gate-Filter, durablem Watermark, `destroy_entry_key`, `rewrite_without` und `verify_absent`. Bei Personenlöschung werden im selben Orchestrierungsauftrag (a) der Root-Gate samt Cluster-Nachfahren, (b) Vernichtung des EntryDEK und Entfernen beider Lookup-Einträge sowie (c) die zugehörigen Atteste abgewartet. Kein systemweites `Complete`, solange ein Owner-Attest fehlt. Backups enthalten höchstens HMAC-Pseudonyme und nicht entschlüsselbaren EntryDEK-Ciphertext; Kompromittierung eines Directory-KMS-Schlüssels bleibt als Schlüsselkompromiss im Receipt-Risiko benannt.
3. **Aufträge nach Frist:** `Eraser::pack_expired` (Ablauf je Packgruppe `Month`) erzeugt unabhängige, deterministisch sortierte Tickets je höchstens 4096 Clustern; Restriction blockiert betroffene Cluster und wird im Aggregate ausgewiesen, der Rest läuft separat.
4. **Keine Zuordnungstabelle außerhalb des Löschvertrags:** Jeder persistente Kennung→Cluster-Pfad muss als `IdentityDirectoryOwner` mit per-Subject-Key/Lineage registriert sein oder selbst eine gleichwertige Gate-/Destroy-/Rewrite-/Verify-Attestierung erfüllen; unregistrierte Mappings verhindern systemweites `Complete` (`ext-conformance`).

---

### 12.8 Verbindliche Korrekturen aus dem R8-Audit (vor G-0/G-F)

Diese Regeln ergänzen die bisherigen Abschnitte und gehen bei Widerspruch vor. Detaillierte Arbeitspakete und Abnahmen stehen in `crates/contextra-storage/25-r8-audit-remediation.md`.

1. **Branch-Vorbereitung:** Branch-IDs werden per bestehendem `reserve_ids` außerhalb Rang 1 reserviert. SYS_BRANCH-Payload wird auf dem Aufruferthread versiegelt; kein `KeyProvider`, Cipher oder KDF-Aufruf unter Rang 1. Commit-Stamp wird ausschließlich im Sequencer vergeben und steht als äußerer MVCC-/Frame-Stamp bereit, nicht als stamp-abhängiges Feld im versiegelten Payload: `Head`-Fork leitet `fork_stamp` aus dem SYS_BRANCH-Record-Stamp ab, `Merged`-Status ebenfalls. Sequencer validiert Reservierung, Parent-Head, Cluster-Generation und Branch-Zustand erneut. Stamp-Vorreservierung pro Branch ist nicht erforderlich und darf keine Commit-Reihenfolge vortäuschen.
2. **Branch-Korrektheit:** `ForkPoint` ist `Head | At(Stamp)`. `Head` linearisiert im Committer. `At(S)` ist nur gültig, wenn `history_floor ≤ S ≤ parent_head` und Cluster/Branch nicht retired/closed sind; andernfalls `BelowFloor`/`InvalidForkPoint`, niemals stilles Rebasen. Merge ist v1 nur vom direkten Child zum Parent erlaubt; Source mit lebenden Kindern ist nicht mergbar. Merge-Konfliktmenge schließt Punkt- und Intervalländerungen ein: `Delete`, `RangeDelete` und jeder überlappende `EraseRange` kollidieren; ein bereits aktiver clusterweiter Erase-Marker kann durch Merge nicht überschrieben werden. Merge-Plan bindet `source_head`, `target_head` und `merge_base`; Commit prüft diese erneut atomar. Erfolgreich gemergte Source-Branches sind terminal (`Merged`); für weitere Änderungen wird ein neuer Branch geforkt.
3. **Idempotenz-Retention:** Idempotenz-Ergebnis (Token-Scope/Hash, Request-Fingerprint, Ergebnis-Stamp/Status und pseudonymes Cluster-Scope-Digest aus sortierten `tomb_id`s) überlebt Redo-`replay_floor` unabhängig, solange TTL aktiv ist. Es wird erst nach Ablauf plus durablem `idem_trim` entfernt. Scope enthält keine Nutzdaten/Klartext-ClusterIds. Retry auf inzwischen `Retired`-Cluster liefert `Erased`/`ClusterRetired`, niemals ein scheinbar neues Apply. Token-Wiederverwendung mit anderem Request-Inhalt wird als `IdemConflict` abgelehnt.
4. **Löschgranularität:** `Eraser::range` bietet für Plain `Logical|Physical|Scrubbed`, für Sealed ausschließlich `Logical`; es gibt **keine** kryptografische Einzelbereichslöschung. `key_epoch`-Rotation macht alte Cluster-Daten erst nach vollständigem Rewrap aller zu erhaltenden Daten und Vernichtung aller alten Epochen unlesbar; sie ist ein eigener, nicht atomarer Workflow und kein v1-Range-Versprechen. Für sofortige Crypto-Erasure müssen Daten in getrennte Cluster/Löschdomänen gelegt werden. Der Marker wirkt über alle Branches und historischen Views und kann nicht gemergt/überschrieben werden.
5. **Space-Drop:** Admission friert Space-Generation und Cluster-Menge unter Sequencer-Ordnung ein; `BLOCK_ERASE` blockiert den ganzen Drop vor Gate, nach Gate sind alle Writes/Register gesperrt. Das Space-Gate macht alle Inhalte logisch unsichtbar, anschließend folgen normale Cluster-Erase-Tickets; es gibt keine Einzel-Edit-/Ein-Fsync-Behauptung für beliebig große Spaces. Ledger erhält je Cluster `tomb_id` plus ein `SpaceDropped`-Aggregate (Scope-Digest/Anzahl/Evidence), damit Restore einzelne Wiederkehrer erkennt. Keine Nutzdaten/Klartext-Clusterliste im Ledger; Receipt zeigt Restriction-Scope und Teilstatus nur für getrennte explizite Tickets.
6. **Klartext-Reclamation:** `Zeroizing` ist eine Maßnahme, kein Beweis für sämtliche Kopien. Memtable-/Cache-Interfaces verbieten unkontrollierte Clone-Pfade für Klartext; Ausgaben liegen in Zeroizing-Puffern. Gate erhöht Generation, entzieht neue Zugriffe und wartet auf Leser-Epochen bzw. weist noch aktive Leser als Exposition aus, bevor `Purged`/Receipt Abschluss behauptet. Allocator-/OS-Kopien sicher zu nullen ist keine Garantie und bleibt Restrisiko.
7. **Blob-Purge:** Purge darf nicht auf `punch_hole` angewiesen sein. Überschreitet der zu entfernende Anteil den konfigurierten Hole-Ratio- oder Max-Fragment-Grenzwert, wird eine neue `blob_id` mit gepackten Überlebenden geschrieben und per Catalog-Edit ersetzt. Reserveplatz und maximale Rewrite-Gruppe sind harte Admission-Grenzen. `Plain`-Barriere invalidiert Consumer explizit mit Epoch-Wechsel/`Lagged`; kein Consumer darf eine Lücke als verarbeitet bestätigen.
8. **Durable Extensions:** `watermark` darf nur bis zum höchsten angewandten `durable_stamp` reichen. Extension-Builder liest nur durable Views oder durable Feed-Frames. Query-Fassade erzwingt `filter_visible` nach Kandidatenfund und erneut vor Rückgabe (Generation-Check); ungefilterte Indexresultate sind kein unterstützter API-Pfad.
9. **Mixed-Segment-Erasure:** Jeder `SegmentOwner` deklariert `SegmentClass = ClusterSections | MixedPerClusterKey | MixedPerSubjectKey | Rebuildable{max_age} | NoPersonalData`, Schutzdomäne, Löschgranularität, RAM-Budget, Rewrite-/Rebuild-Obergrenze und Attestationsformat. `MixedPerSubjectKey` ist ausschließlich für Subjekt-Mappings wie das IdentityDirectory zulässig: jeder Record hat einen eigenen KMS-EntryDEK, die Key-ID wird deterministisch aus einem pseudonymen Cluster-Handle abgeleitet und ist nicht im Store abgelegt; der Owner attestiert `destroy_entry_key`, `rewrite_without` und `verify_absent`. Für `Sealed` muss jeder clusterbezogene Payload mit Cluster-Schlüssel/Epoche oder gleichwertig isoliertem Schlüssel geschützt sein; ein globaler Segment-Key reicht nicht für `INV-ERA-7`. `Rebuildable` bleibt bis zum bestätigten Rebuild `Purging`, nie `Done`. `NoPersonalData` ist deklarativ und erhält keine Crypto-Erasure-Zusage.
10. **RAG-/Audit-Grenze:** Ein `Change` kann Dokumentdaten, quantisierte Vektoren, BM25-Postings und Graph-Deltas atomar persistieren, sofern sie als Records derselben Cluster-Löschdomäne modelliert sind. Globale HNSW-Nachbarschaftsreparatur bleibt abgeleiteter Index-Job und wird nicht im selben Kern-Commit garantiert. Sie nutzt durable Watermarks, Gate-Filter und attestierten Rewrite/Rebuild. `ErasureReceipt` attestiert nur Kern und registrierte Extensions; signierte v15-`DeletionProof`s entstehen über eine versionierte Bridge aus Receipt plus Extension-Attesten. Ohne vollständige Atteste kein systemweites `Complete`.
11. **Gesamtsystem-Ressourcenbudget:** Storage und Extensions teilen einen `MemoryBudget` mit getrennten Limits für Memtables, Cluster-Metadaten, Guard-Fenster, Block-/Key-Cache, Index-Delta, In-flight-Rewrites und temporären Purge-Platz. Admission/Backpressure erfolgt vor Commit/Index-Job; keine Schicht darf unbeschränkt RAM oder Rewrite-Scratch allokieren. Budgets werden gemeinsam im RAG-Lastprofil gemessen.
12. **Archive-Konsolidierung:** Ältere Lethe-Entwürfe sind Designmaterial, keine zusätzliche Norm. Übernommen werden SYS_IDEM-Retention, Extension-Klassen/Atteste und gemeinsames Speicherbudget. Nicht übernommen werden globale EU-Tags/Content-Addressing, In-place-Scrub als Durability-Annahme sowie Zusagen kryptografischer Teillöschung.
13. **Schlüsselzustände (E-56):** `KeyUnavailable`, `KeyDestroyed/Absent`, `KeyDisabled` und `KeyLost` sind getrennte Zustände. Nur bestätigtes `Absent` erfüllt Crypto-Erasure; `Unavailable`/`Lost` blockieren oder markieren `Incomplete`, `Disabled` bleibt `KeyDestroying` mit Restrisiko. Provider-Nichterreichbarkeit darf nie als Vernichtung interpretiert werden.
14. **Redo-Crypto-Domäne (E-57):** RootKey-abgeleiteter Redo-Key ist nur Ketten-MAC-Schlüssel. Jeder Sealed-Redo-Op ist per eigener Cluster-Key-Epoche versiegelt; Op-Header bindet `key_epoch`. Redo-Rohreste werden nach `destroy_many` unlesbar. Gate-/Routing-Header bleiben als pseudonyme Metadaten sichtbar und sind ausdrücklich kein Nutzpayload.
15. **Blob-AAD (E-58):** Blob-AAD bindet stabile `record_uid`, `instance`, `space`, `cluster`, `key_epoch`; nie `blob_id`/Offset. Dadurch ist Survivor-Rohkopie in neue Blob-ID/Offsets gültig, ohne AEAD-Aufbrechen. Neue ID/Offsets bekommen eigene CRC/Directory.
16. **Diff-Kosten (E-59):** Branch-Diff- und Merge-Konfliktkandidaten kommen aus cluster-sealed, stamp-sortiertem `ChangeIndex` für Punkt-/Intervallmutationen beider Seiten. Allgemeiner Diff kostet Änderungen seit gemeinsamem Floor plus Ausgabe, nicht nur Branch-Einträge. ChangeIndex-Format, GC und Branch-Pins sind G-F-Bestandteil.
17. **Gewichtsvertraulichkeit (E-61):** `weight_sum[W]` und Tokenprofile sind bei Sealed-Clustern mit dem Cluster-Key verschlüsselt. `page` darf kleine SubIndex-Metadatenblöcke entschlüsseln, liest aber keine Body-Blöcke. Klartext-ClusterDirectory enthält keine Gewichte.
18. **Extension-Liveness (E-60):** Owner-Aufrufe besitzen Deadline und Fencing-Generation. Timeout erzeugt `Blocked{owner, reason, since, retry_at}` + Ledger-Ereignis/Alarm/Eskalation; verspätete alte Generation wird verworfen. Persistente Feed-Consumer, die Derivate schreiben, müssen SegmentOwner/Lineage registrieren. Nicht attestierte externe Kopien erscheinen als Residual und verhindern systemweites `Complete`.
19. **Purge-Reserve (E-63):** Physischer Rewrite nutzt getrennte, geschützte, vorallokierte Scratch-Slots; Standardpool ≥ zwei maximale Run/Blob/Redo-Dateien (128 MiB bei 64-MiB-Grenzen), ein Rewrite-Writer. Purge arbeitet pro Datei mit atomarem `PurgeBatchEdit`, direktem Unlink vollständig toter Dateien und finalem `PurgeCompleteEdit` erst nach allen Acks. FS-CoW/Quota muss vorallokierte Platzgarantie belegen; sonst wird harter DiskFull-Purge nicht zugesichert.
20. **Löschkostenvertrag (E-62):** `EraseLocality::ClusterLocal` isoliert Run-/Blob-Dateien und vermeidet dort fremde Survivor-Rewrites. Gemeinsam genutztes Redo, Retention-Pins und Checkpoints bleiben separat begrenzt/attestiert; daraus folgt keine allgemeine O(Clusterbytes)-Garantie. `PackedBounded` erlaubt Survivor-I/O bis zur Summe der betroffenen Dateien. Globale Rebuildable-Indizes berichten eigene Kosten/Fristen.
21. **Zeit- und Link-Semantik (E-64/E-65):** TTL/Deadline nutzt monotone Laufzeit + UTC + durablen `clock_floor`; Standard-Skew ist 5 min, Compliance-Profile dürfen ihn nur explizit verkürzen. Sprünge außerhalb Toleranz stoppen Retention-/Hold-Übergänge fail-closed und markieren Friststatus `ClockUncertain`. Cross-subject DerivedFrom-Kaskaden sind unzulässig; gemeinsame Ableitungen brauchen getrennte Löschdomänen oder nicht-kaskadierende Referenzen.
22. **Limit-Konsistenz (E-66):** `max_body_bytes` ist hart 4 MiB, `max_change_bytes` 8 MiB, `max_body ≤ max_change/2`; §4.6, Ports-Konstanten und Config müssen dieselben Werte verwenden. H-21 misst bis zur konfigurierten Branch-Maximaltiefe; Zielverfehlung senkt Default oder erzwingt Flattening.
23. **Purge-Edit-Zustandsautomat (E-67):** Jede Dateiersetzung/-entfernung ist ein eigenes durables `PurgeBatchEdit`; `files_remove` pinnt die Quelldatei im persistenten Graveyard. Unlink folgt nach Reader-Drain und `sync_dir`; `UnlinkAckEdit` räumt den Graveyard erst danach. Nur `PurgeCompleteEdit` darf bei leerem Ticket-Graveyard, allen Owner-Attesten und finalem Verify `Purged/Done` setzen. Recovery setzt Batches idempotent fort und kann nie aus einem Zwischenbatch `Done` ableiten.
24. **Krypto-Implementierungsabgleich (E-68):** Normativ ist XChaCha20-Poly1305 direkt mit dem bereits scope-/epoch-spezifischen Schlüssel; `Authenticated` ist domain-separated HMAC-SHA256 über Modus, Nonce, AAD und Payload. Die vorhandene `derive_subkey_fast`-Vorstufe im Referenzcode weicht davon ab und ist vor G-F zu entfernen oder auf einmalige Schlüsselrollen-Ableitung außerhalb jedes SealFrame zu begrenzen. Kein G-F ohne unabhängige Vektoren, die genau diese Byte-/Schlüsselableitung verifizieren.

## 13. Verifikation

### 13.1 Ebenen

| Ebene | Werkzeug | Inhalt |
|---|---|---|
| Formal | TLA+/TLC (oder Quint) | M1 Commit/Sichtbarkeit/Recovery/Stamps/Guards · M2 Erase (ER0–ER7, Gate, Restriction, Link, Consumer-Floors, Space-Drop, **Branch-Ops und Merge**) · M3 Compaction × Erase × Claims × Staging × Fork-Floor · M4 Checkpoint × Erase × Restore · M5 Rebucket · M6 Anker × Edit × Rollback |
| Code-Modell | `loom` | Sperren, `Arc`-Swap, Generationszähler, Claims, Graveyard, Committer-Führung, View-Widerruf |
| Simulation | eigener Scheduler + `shuttle` | **Hauptwerkzeug gegen Races:** seeded Interleavings, `FaultFs`, `SimClock`, `SimEntropy`, `SimKms`, reproduzierbarer Seed; Race-Nahtstellen R1–R17 als benannte Szenarien |
| Absturz | `FaultFs` | Crash an **jedem** E/A-Index, Torn Writes, verlorene Fsyncs, Rename-Reihenfolgen, Rollback-Versuche, `ENOSPC` beim Vorallokieren, **Geister-Journal** aller je geschriebenen Seiten |
| Eigenschaft | `proptest` | Codecs, `cmp`, Siegel-Roundtrip und Rohkopie, Compaction ≡ Referenzmodell (`BTreeMap` mit Versionen und Branches), Purge ≡ Modell ohne Cluster, Paging (Budget nie überschritten, lückenlos, vorwärts/rückwärts gleiche Menge, Block-Skipping ≡ Vollscan), Branch-Lesen ≡ Modell, Merge ≡ Modell |
| Lokalität | Byte-Zähler | `ClusterLocal` und `PackedBounded` separat bei Store ×1/×10/×100; Run/Blob, gemeinsam genutztes Redo, Checkpoints und Extensions getrennt zählen. Bounds für Redo-Retention, Pins, Packgruppen und Extensions explizit belegen; keine pauschale O(Cluster)-Behauptung |
| Mutation | `cargo-mutants` | Score auf `codec`, `recovery`, `erase`, `committer`, `branches` |
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

### 13.3 CI

Pflicht: `check`, `fmt`, `clippy -D warnings`, `test`, `layering`, `loom`, `doc`, `deny`, Sim- und Absturz-Kurzlauf, INV-Zitierprüfung (jede ID aus §15 in mindestens einem Test per `// INV-…`), Abdeckung. Nachtlauf: lange Sim, Fuzz, Orakel-Vollsuche, Mutation, Provider-Konformanz. **Ohne funktionierende CI ist kein Gate gültig.**

---

## 14. Phasen und Gates

Planungsannahme für 3–4 erfahrene Rust-Entwickler plus 1 Test-/Simulationsingenieur; Dauern sind grobe Schätzungen, keine Zusage.

| Phase | Inhalt | Gate | ca. |
|---|---|---|---|
| **P0 Fundament** | `ports`, `codec`, `env` (MemFs, RealFs, FaultFs, Sim), `sync`, Anker (Mem, Datei), `crypto-ref` mit In-Memory-Provider und Konformanzsuite, Orakel-Skelett, CI, Lint-Gerüst, Referenzcodierer, Workspace-Einbindung | **G-0:** diese Spec normativ (zwei Reviewer), CI grün | 3 Wo. |
| **P1 Walking Skeleton (vertikaler Durchstich)** | ein Space, `apply`, Redo + Memtable, `view`, Cluster-Register, Gate, `destroy_many`, Ledger (ohne Anker), Recovery aus Redo; **Formate provisorisch** | Crash-Sweep über alle Fs-Operationen grün; O-Files findet nach `Done` nichts, obwohl Daten nur in Redo/Memtable lagen | 4 Wo. |
| **P2 Runs und Catalog** | Flush, ClusterSection, Staging, Catalog + `CURRENT`, Purge, Plain-Barriere, vollständige Recovery, Absturzmatrix | O-Files für Daten in Runs grün; Modelltest grün; M1, M2 (ohne Branches) geprüft | 5 Wo. |
| **G-F Formatfreeze** | alle Formate (§5.9) inkl. reserviertem `branch`-Feld, goldene Vektoren, Rohkopie-Test, ≥ 8 Fuzz-Ziele | Freeze-Review, Referenzcodierer vollständig | — |
| **P3 Schreibpfad produktiv** | Gruppen-Commit, Reservierungen, Guards zweistufig, Volatile, Notbetrieb `DiskFull`, Idempotenz, Backpressure | **G-W:** Absturzmatrix, Linearisierbarkeit in Sim, H-1, H-18 | 4 Wo. |
| **P4 Lesepfad** | Blobs, Cache, Views (Revoke), `page` mit Gewichten, Block-Skipping, `ResumeToken`, `filter_visible` | **G-R:** Modellgleichheit, Paging-Properties, H-2…H-4 | 4 Wo. |
| **P5 Wartung** | Compaction je Wartungsgruppe, Claims, Blob-GC, Governor, Rotation, Consumer-Feed aus Redo | **G-M:** M3 geprüft, Sim ohne Invariantenbruch, H-5, H-19 | 5 Wo. |
| **P6 Löschung komplett** | Links, Restrictions (`LiftPolicy`), Abwurf, Space-Drop, `range`, Ledger/Anker, Checkpoint/Restore, `keys-file`, `keys-cohort`, `SegmentOwner`/`ExtensionBlock`, Receipt + `verify_receipt`, volle Orakel | **G-E:** M2/M4/M5/M6 geprüft, Orakel 0 Treffer, INV-ERA-11 (Lokalität), H-6…H-11, H-17, Konformanzsuite grün | 6 Wo. |
| **P7 Versionierung** | `BRANCHES`-Feature-Bit, `SYS_BRANCH`, Fork/Drop/TTL, Branch-Lesen, Diff, Merge, `Eraser::branch`, Fork-Floor | **G-V:** Branch-Properties vs. Modell, M2/M3 mit Branches, H-20…H-22, Orakel mit Branch-Canaries | 5 Wo. |
| **P8 Härtung** | Soak 72 h, Dauer-Fuzz, Platte voll, Fehlerinjektion, Fassade, Abdeckung, Krypto-Review, Messbericht, Bedrohungsmodell-Review | **G-P:** alle INV getestet (100 %), H-15/H-16, Messbericht §17, keine offenen Korruptionsbefunde | 5 Wo. |

**G-CUTOVER (P8+, SP-CUTOVER 24):** separates Produktfreigabe-Gate nach G-P. Erst wenn sämtliche produktiven `contextra-store`-Verbraucher, Datenräume und Erasure-Witnesses migriert, Restore-/Rollback-Rehearsal und RAG-Differentialmessung bestanden sowie gemeinsames Speicherbudget belegt sind, wird `contextra-storage` alleiniger Produktions-Schreibkern. Nach ersten R8-Writes ist Rückkehr nur per geprüftem Forward-Export oder Restore möglich; kein Dual-Write. Bis dahin gelten R8-Leistungs- und Löschvorteile als unbestätigte Ziele.

Keine Phase beginnt vor dem Gate der vorherigen; Formatänderungen nach G-F nur per Migration. In jeder Phase wird der Crash-Sweep um die neuen Fs-Operationen **erweitert, nie nachgezogen**. Erst nach G-P werden aufsetzende Komponenten (Kontext-Engine, Indizes, MCP-Server) angebunden.

**Sofortmaßnahmen vor P1:** (1) `contextra-storage` in den Workspace aufnehmen, fehlende Crates `contextra-storage-ports`, `contextra-crypto-ref` anlegen; (2) Bestand (R2-Entwurf ohne Cluster) modulweise auf R8 umstellen (`redo`, `run`, `blob`, `catalog`, `committer` formatbetroffen); (3) `unwrap/expect/panic` aus Produktionscode entfernen (Altstand 1260 + 16 Treffer, R7-Anhang B); (4) `deny.toml`, CI, Toolchain-Pin (1.89.0); (5) fehlende Verweisdateien anlegen oder Verweise entfernen; (6) Altstand `contextra-store` per Tag `legacy-store-final` archivieren, **kein Code wird übernommen**.

---

## 15. Invarianten (normativ, jede wird in mindestens einem Test per ID zitiert)

| ID | Aussage |
|---|---|
| INV-STAMP-1/2/3 | Stamps streng monoton über angenommene Changes/Frames · kein Umbruch (`Exhausted`) · nie doppelt vergeben, auch nicht nach Absturz |
| INV-ID-1/2 | Datei-/`tmp_id` und `SpaceId` werden nie wiederverwendet |
| INV-ATOM-1 | Ein Change ist ganz oder gar nicht sichtbar, auch nach Absturz, über alle Spaces und Branches (nicht für Volatile-Ziele) |
| INV-COMMIT-2 | Ein sichtbarer Eintrag ist durabel (außer `Relaxed`) |
| INV-GUARD-1 | Ein Change wird nur angenommen, wenn jede Guard zur Stamp-Vergabe gilt |
| INV-VIEW-1/2 | Eine View sieht stets dieselben Daten · `visible_stamp` vor `Arc<Version>` |
| INV-VIS-1 | Jedes Ergebnis einer Extension passiert `filter_visible`, bevor es den Aufrufer erreicht |
| INV-CONS-1 | Consumer und `view_durable` sehen nur Stamps ≤ `durable_stamp` |
| INV-LOCK-1/2 | Sperren nur aufsteigend · Stamp-Vergabe und `Live → Retired` nur im Committer |
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
| INV-ERA-2 | Nach `Purged` keine Version des Clusters in lebenden Runs/Redo/Memtable (Physical); bei Cryptographic zusätzlich INV-ERA-7 |
| INV-ERA-3 | ER1 < ER2 < jedes `PurgeBatchEdit` < zugehöriger Unlink < finales ER6/`PurgeCompleteEdit`; Crypto-Purge erst bei `probe == Absent`, `Done` nur nach allen erforderlichen Attesten |
| INV-ERA-4 | Nur Roll-forward |
| INV-ERA-5 | Reads unter `S` in `erased_ranges` liefern `Erased`, nie Daten, nie „leer“ |
| INV-ERA-6 | Nach `Purged` keine lebende Blob-Inkarnation mit Record des Clusters |
| INV-ERA-7 | Nach ER2 ist alles des Sealed-Clusters ohne Schlüssel unlesbar |
| INV-ERA-8 | Receipt nur bei erfüllten Done-Bedingungen (§9.5) |
| INV-ERA-9 | Nach Restore ist jeder Cluster aus dem Ledger wieder mindestens `Retired` |
| INV-ERA-10 | Eine BLOCK_ERASE-Restriction in der Link-Hülle verhindert jede Löschung; Prüfung im Committer, kein Override |
| INV-ERA-11 | Gate/KeyDestroy skaliert mit Zielclustern. Purge bearbeitet nur betroffene Dateien/Extents und gepinnte Redo-/Extension-Bereiche. `ClusterLocal` vermeidet fremde Run/Blob-Survivor, garantiert wegen gemeinsam genutztem Redo/Pins aber keine O(Clusterbytes)-Gesamtkosten. `PackedBounded` kostet höchstens die Summe betroffener, je `max_file_bytes` begrenzter Dateien. Kein Vollstore-Rewrite; keine pauschale Cluster-Schranke ohne Bounds für Redo-Retention, Pins und Extensions. |
| INV-ERA-12 | Die Erase-Menge ist der transitive `DerivedFrom`-Abschluss zum Gate-Zeitpunkt, in einem Frame |
| INV-LED-1 | Das Ledger enthält weder ClusterId noch Nutzdaten noch Nutzerschlüssel |
| INV-ANCH-1 | Anker monoton; Catalog/Ledger hinter dem Anker wird nicht schreibend geöffnet; Anker nie vor dem durablen Edit |
| INV-KEY-1…4 | Schlüsselmaterial nie unverhüllt in Dateien/Checkpoints · `gen` vor und nach Entsiegeln · alte Epoche erst vernichtet, wenn nichts mehr mit ihr versiegelt · `destroy_many` vernichtet alle Epochen |
| INV-PRIV-1 | Keine persistierten Inhalts-Hashes über Klartext (flüchtige Prozessschlüssel-Fingerabdrücke ausgenommen) |
| INV-VOL-1 | Volatile schreibt kein Redo; Mischchange nicht durabel über Absturz |
| INV-LIM-1 | Changes über den Limits werden vor jeder Sperre abgewiesen |
| INV-DISK-1 | Datenträger voll vergiftet nicht; Gate, Schlüsselvernichtung, Purge bleiben ausführbar |
| **INV-BR-1** | Ein Branch-Lesevorgang liefert für jeden Schlüssel die Version, die das Referenzmodell (Eltern-Kette mit Stamp-Kappen) liefert |
| **INV-BR-2** | Ein Merge ist atomar: bei Konflikten ändert sich nichts; bei Erfolg sind alle Auflösungen und der `Merged`-Zustand in einem Frame |
| **INV-BR-3** | `history_floor` eines Clusters ≤ `fork_stamp` jedes lebenden Branches; Branch-Daten und -Metadaten liegen im Cluster und sterben mit ihm |
| **INV-BR-4** | Fork kopiert keine Nutzdaten (O(1)), Branch-IDs werden je Cluster nie wiederverwendet |

---

## 16. Konfiguration (Auszug)

`block_target` 16 KiB · `memtable_total_bytes` ≤ 256 MiB · `redo_segment_bytes` 64 MiB, `redo_mark_bytes` 8 MiB, `redo_max_age` 1 h · `run_target_bytes` 64 MiB · `blob_max_bytes` 64 MiB · `max_pack_groups` 4096 · `pure_threshold` 8 MiB · `max_gate_clusters` 4096 · `max_open_files` 1024 · `maint_io_fraction` 0,5 · `purge_batch_min` 16 · `purge_rewrite_concurrency` 1 · `purge_scratch_reserve_bytes` ≥128 MiB (zwei Slots, keine normale Allokation) · **`purge_deadline` kein Standard (Pflichtparameter)** · Reservierungen 2^20 / 2^16 · `max_links` 8 · `max_hold_ttl` / `idem_ttl` 24 h · `anchor` Required bei Encrypted/Authenticated, `anchor_max_age` 1 s · `guard_window_entries` ≤ 1 Mio. und durch MemoryBudget begrenzt · `max_lag_bytes` 1 GiB · `erase_reserve_bytes` ≥2 GiB (inklusive Scratch 128 MiB, Consumer-Retention 1 GiB, Emergency-Reserve und FS-Metadaten-Headroom) · `max_clock_skew` 5 min default, strenger nur explizit · `key_grace` 10 min · `orphan_key_grace` 24 h · `revoke_grace` 5 s · `max_branches_per_cluster` 1024, `max_branch_depth` 8 · `LiftPolicy` OwnerOrAdmin.

**Gemeinsames RAM-Budget:** `memory_budget_bytes` defaultet auf `min(1 GiB, max(256 MiB, 20 % des effektiven cgroup-/Job-Limits))`; kann der Adapter kein verlässliches Limit liefern, ist der Wert beim Öffnen Pflicht. Unterbudgets sind Obergrenzen am selben Budget: 25 % Memtables, 15 % Cluster-/Branch-Metadaten, 2 % Guard-Fenster, 20 % Block-Cache, 1 % Key-Cache, 20 % Extension-/Indexdelta und 10 % In-flight-Changes/Rewrite-Buffer; 7 % bleiben als Allocator-/Runtime-Headroom unangetastet. Jede Allokation reserviert atomar beim `MemoryBudget` und gibt per RAII frei. `max_clusters`, Branch-Limits und Extension-Admission werden zusätzlich durch tatsächliche Metadatenbytes begrenzt; `max_clusters=16 Mio.` ist eine Obergrenze, keine zugesagte Resident-Kapazität. Ein Extension-Owner ohne Budget-Token darf keine Indexarbeit oder Cache-Allokation starten.
**Metriken (Pflicht, Z-14):** `erase_open_tickets`, `erase_oldest_pending_age`, `erase_deadline_breaches_total`, `gate_latency`, `key_destroy_latency`, `purge_bytes_copied`, `anchor_lag_ms`, `disk_free_vs_reserve`, `consumer_lag_bytes`, `guard_slowpath_ratio`, `key_cache_hit_ratio`, `diskfull_state`, `branch_count`, `branch_oldest_age`. Keine Metrik und kein Ereignis enthält Schlüssel, Nutzerschlüssel oder Inhalte.

---

## 17. Leistung: Hypothesen und Messverfahren

Alles **Entwurfsziele**. Freigabe nur mit Messbericht (Hardware, Dateisystem, Last, Methode, Streuung); scheitert eine Hypothese, wird **sie** korrigiert, nicht das Verfahren gelockert. Es gibt drei Benchmarkgruppen (aus CairnDB): **Storage** (Commit-Latenz, Mehr-Space-Transaktionen, Recovery, Schreib-/Leseverstärkung), **Agent-Workload** (Request → letzte 100 Events lesen → Zustand → 20 Erinnerungen → Update → 10 Events anhängen → Checkpoint → Commit, gemessen in physischen Reads/Writes, CPU, Speicher), **Retrieval-Qualität** der oberen Schicht (Recall@Budget, Nutzen-pro-Token, Task-Erfolg bei gleichem Budget; Mnemos-Utility@Budget).

| ID | Hypothese | Messung |
|---|---|---|
| H-1 | `apply`: Durchsatz ≥ 80 % von Fsync-Rate × mittlere Gruppengröße; p99 ≤ 2 × Fsync-Latenz | Mikrobench, variierende Schreiberzahl |
| H-2 | Punktabfrage warm: ≤ 1 Datenblock-Read je Treffer-Level, p99 ≤ 50 µs | Zipf-Last |
| H-3 | Kopf-Scan ohne Körper ≥ 10× Durchsatz gegenüber Scan mit Körper (≥ 4 KiB) | Scan-Bench |
| H-4 | `page` liest ≤ 1,2 × Blöcke des Minimums | Gewichtsverteilungen |
| H-5 | Schreibverstärkung Metadaten ≤ 15, Körper ≤ 2 | Compaction-Statistik |
| H-6 | ER0–ER2 je Cluster ≤ 10 ms (ohne Providerlatenz), unabhängig von der Memtable-Größe | Löschbench |
| H-7 | Messhypothese: Purge-Bytes ≤ Summe der betroffenen begrenzten Datei-/Extent-Größen plus dokumentierte Metadaten. Verhältnis bei 10× Store separat je Layout, Redo-Retention und Pin-Last messen; kein O(Clusterbytes)-Ziel | je `EraseLocality` und Dateirolle skalieren |
| H-8 | Pure-Cluster löschen ≤ 100 ms je 1000 Dateien (ohne Scrub) | Drop-Bench |
| H-9 | 1000 Cluster einer Gruppe ≤ 2 × Aufwand eines Clusters | Bündelungsbench |
| H-10 | Rohkopie beim Purge ≥ 80 % sequentielle Plattenbandbreite | Purge-Bench |
| H-11 | Seal ≤ 15 % CPU im Lastmix; Overhead kleiner Cluster ≤ 5 % Bytes (≥ 1 KiB) | A/B-Bench |
| H-12 | Speicher ≤ `memtable_bytes × (1+max_immutable)` + Cache + O(offene Gruppen); ~48 B je Cluster und Datei, ~128 B je Cluster | RSS |
| H-13 | Recovery ∝ Redo-Rückstand + Catalog; 1 GiB Redo ≤ 10 s | Recovery-Bench |
| H-15 | Zeilenabdeckung ≥ 90 %, Zweigabdeckung ≥ 85 % | `cargo-llvm-cov` |
| H-16 | Mutationsscore ≥ 80 % auf `codec`, `recovery`, `erase`, `committer`, `branches` | `cargo-mutants` |
| H-17 | Gate + ER2 für 10 000 Cluster ≤ 1 s bei Providerlatenz ≤ 5 ms je `destroy_many` | Batchbench |
| H-18 | Guard-Schnellpfad ≥ 99 % bei Zipf-Last mit 1 % Konfliktrate, keine E/A im Committer | Guard-Bench |
| H-19 | L0-Dateizahl unabhängig von der Zahl aktiver Packgruppen | Flush-Bench mit 4096 Gruppen |
| **H-20** | Fork < 50 µs, Speicher O(1) | Fork-Bench |
| **H-21** | Lesen auf Branch bis zur konfigurierten Maximal-Tiefe (Default 8): ≤ 1,5 × Punktlesekosten von `main` bei 1 % geänderten Schlüsseln; Tiefen 1…8 einzeln messen | Branch-Bench mit Vorfahren-Pruning und Worst-Case-Branch-Change-Rate |
| **H-22** | Merge ∝ Source-Mutationen + Target-Mutationen seit `merge_base` + Konflikte, nicht Gesamt-Clustergröße; je 10⁴ Mutationen ≤ 200 ms ohne Körper-I/O/Konfliktauflösung | ChangeIndex/Merge-Bench |

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
| KMS-Karenzzeit (Lethe-Spec B.6) | `CompleteWithResidual`; strenge SLAs verlangen sofort zerstörenden Provider |
| Single-Sequencer-Durchsatz (Lethe-Spec B.5) | Ein Committer je Instanz; Tenant-Sharding über mehrere Instanzen oben |

**Risiken:** (1) Branch-Pinning treibt Speicher und Compaction-Kosten bei vielen Langzeit-Branches (Mitigation: `ttl`, Limits, `floor`-Metrik). (2) Branch-Reads über Elternketten (H-21) können bei tiefen Ketten teuer werden (Mitigation: `max_branch_depth`, Merge/Flatten). (3) Kosten der Plain-Barriere bei häufigen Plain-Löschungen (Mitigation: Bündelung; Personal-Daten sind per Konstruktion Sealed). (4) Verzeichnis Person → Wurzelcluster bleibt nur physisch löschbar (§12.7). (5) Formatfreeze nach P2 verlangt, dass P1/P2 echte Probeläufe sind, nicht Papier. (6) Scope-Creep in obere Schichten; Mitigation: Gates strikt.

**Externe Vorbedingungen (keine Spec-Lücken):** rechtliche Prüfung der Fristen und der Aussage „kryptografisch gelöscht“ für den Einsatzfall; zweiter Reviewer für G-0; Beschaffung eines `Strong`-Ankers und eines HSM/KMS-Providers für den Produktivbetrieb (der Kern ist ohne sie vollständig, Restrisiken sind benannt).

---

## 19. Entscheidungsregister (neu nummeriert)

| ID | Entscheidung |
|---|---|
| SD-01 | Catalog als einzige Wahrheit, ein Edit = ein Durabilitätspunkt |
| SD-02 | Clusterreine Blöcke, ein Abschnitt je Cluster und Datei, abschnittsrelative Offsets |
| SD-03 | Bewilligte Abhängigkeiten: `crossbeam-skiplist` (hinter Trait), `bytes`, `blake3`, `chacha20poly1305`, `zeroize`; Dev: `loom`, `shuttle`, `proptest`, `cargo-mutants`; Krypto nur über Ports |
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
| SD-28 | Gate ist ein Committer-Frame |
| SD-29 | Blob-Entfernung nur nach Abgleich lebender Runs |
| SD-30 | Memtable lock-frei |
| SD-31 | Change-Limits |
| SD-32 | Bitemporalität als Konvention oben |
| SD-33 | Namensraum `contextra-storage(-ports)` |
| SD-34 | Eine normative Spec plus `AGENTS.md`; Vorfassungen in `docs/archive/` |
| SD-35 | Redo-AAD mit `op_uid` statt Stamp |
| SD-36 | `AdminCap`/`RestrictedReadCap` als nicht klonbare Fähigkeiten |
| SD-37 | Kern erzeugt `ClusterId`; Sealed/Plain; `personal` erzwingt Sealed |
| SD-38 | Eine Siegelform; RootKey-Indirektion; `Cryptographic` nur bei `keys_in_backups == false` |
| SD-39 | Gate ohne Segmentschnitt; Barriere beim Purge, nur Plain, gebündelt |
| SD-40 | Consumer-Feed aus Redo mit zwei Böden |
| SD-41 | Guards zweistufig |
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

---

## Anhang A — Referenzvektoren (Pflicht vor G-F)

Je Format mindestens: leeres Objekt, ein Eintrag, Eintrag mit versiegeltem Körper (Cluster- und Space-Quelle), Cluster-Abschnitt mit zwei Blöcken, Packed-Run mit drei Clustern, Staging-Run, Tombstone, Bereichsmarker, Blob mit Loch, Catalog-Edit-Kette mit Reservierung, Gruppen-Commit und `anchor_mark`, Ledger-Kette (`tomb_id`), Redo-Gruppe mit Change-/Gate-/Control-/Branch-Frame, abgeschnittener und beschädigter Frame, nicht-kanonisches `varint`, **Rohkopie-Test** (Abschnitt aus Run A in Run B, Blöcke byte-identisch, Entsiegeln gelingt). Erzeugung durch `tools/reference/generator.py` (unabhängig von Rust), Vergleich im Test `golden`. **Stand: nicht vorhanden.**

## Anhang B — Änderungen gegenüber R7

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

## Anhang C — Verworfene Vorschläge

Prolly-Tree/Merkle-DAG/Inhaltsadressierung im Kern (K-07, K-08) · CAS-Superblock ohne Log (K-10) · `mmap`/`io_uring`/GPUDirect im Kern (K-11) · Token-IDs, KV-Cache-Tensoren, feste Seitenstruktur (K-13) · gelernte Cross-Objekt-Affinität als Layout (K-17) · Slab-Sub-Allokator (K-20) · Segmentschnitt je Gate · Aufrufer-vergebene ClusterId · `eu_tag` im Klartext-Header · Poison bei ENOSPC · `rekey_clone` · Indizes (ANN, BM25, Graph) im Kern · MCP-Server, Tokenizer, Embedding-Modelle im Kern · benutzerdefinierte Merge-Operatoren im Kern.

## Anhang D — Zustände

```
Cluster:    Live ──Gate(S)──► Retired(S) ──ER2──► Shredded ──ER6──► Purged ──cluster_trim──► (UnknownCluster)
            (Shredded nur Sealed/Cryptographic; nur vorwärts; Restrictions orthogonal)
Branch:     Live ──merge──► Merged ;  Live ──drop/ttl──► Dropped ;  (alle: sterben mit dem Cluster)
Space:      Live ──SpaceGate(S)──► Dropping(S) ──SD3──► Dropped
Packgruppe: Aktiv ──PD1──► Abwurf ──PD4──► Entfernt   (Restricted-Cluster verbleiben)
Datei:      Building ──rename+Edit──► Lebendig ──Edit──► Obsolet ──letzte Referenz──► Unlink
Erase:      Gated ─► KeyDestroying ─► KeysDestroyed ─► Purging ─► Done   (Absturz an jeder Stelle: Fortsetzen)
Anker:      monoton; Catalog-Epoche ≥ Anker und Ledger enthält Ankerhash beim Öffnen, sonst Rollback (F-9)
Instanz:    Normal ─free_space<Reserve─► DiskFull (Löschung läuft) ─Hysterese─► Normal ; Normal ─EIO/Fsync─► Poisoned (Neustart)
```

# CONTEXTRA-SPEC R14 — Gesamtsystem-Spezifikation (normativ)

| | |
|---|---|
| Status | R14, Entwurf zur Freigabe als Gate G-0. Ersetzt SPEC R13 vollständig. Keine Ergänzungsabschnitte: jede Regel steht genau einmal. |
| Produkt | Eingebettete, crash-sichere Informations-Engine für europäische Unternehmens-LLM-Anwendungen: Speicherkern `contextra-storage` (ersetzt `contextra-store`), Hybrid-Retrieval (Vektor, BM25/WAND, Wissensgraph, kalibrierte Fusion), nachweisbare Löschung personenbezogener Daten, token-budgetierte Kontextlieferung. Contextra liefert Daten und Kontext an LLMs/Agenten; es betreibt weder Modelle noch Agenten. |
| Ist-Stand | Statisch geprüft (HEAD `78a1783`), nicht kompiliert. Workspace nicht baubar (§13.2). Alle Zahlen in §16 sind Entwurfsziele mit Messverfahren, keine Messwerte. |
| Rechtlicher Vorbehalt | Das System liefert technische Eigenschaften (Sichtbarkeitsbarriere, Schlüsselvernichtung, Dateibereinigung, Löschprotokoll, Datentrennung), keine Rechtsaussage zu Art. 17 DSGVO, Fristen, Ausnahmen oder zur Einstufung pseudonymisierter Daten als anonym. „Gelöscht“ ist immer an eine nachgewiesene Stufe (§8.2) und ein konkretes `ErasureReceipt` gebunden. |
| Sprache | Prosa deutsch, Bezeichner englisch. |

**Normative Konventionen.** MUSS / DARF NICHT / SOLL / KANN im Sinn von RFC 2119. „Garantie“ = durch Invariante (§14) und Test belegt. „Entwurfsziel“ = Hypothese `H-xx` (§16). Bei Widerspruch gilt: §14 Invarianten vor §8/§9 vor den übrigen Abschnitten. IDs (`INV-*`, `H-*`, `RACE-*`, `DEC-*`, `FIX-*`) sind stabil.

## 0. Navigation

| Frage | Abschnitt |
|---|---|
| Ziele, Mission, Nicht-Ziele, Bedrohungen | §1 |
| Crates, Ebenen, Ports, Threading, Fehler | §2 |
| Cluster, Spaces, Limits, Admission, Datenklassen | §3 |
| Byteformate, Siegel, Redo, Run, Blob, Catalog | §4 |
| `apply`, Guards, Idempotenz, DiskFull | §5 |
| Lesen, Paging, Branch-Lesen | §6 |
| Branches, Merge | §7 |
| Löschen: Stufen, Schlüssel, Protokoll, Ledger, Receipt | §8 |
| Open, Crash, Restore, Backup-Horizont | §9 |
| Sperren, Races | §10 |
| Indizes, Partitionen, Ingest, Query-Plan, Trennung PII | §11 |
| Tests, Orakel | §12 |
| Phasen, Gates, Altpfade | §13 |
| Invarianten | §14 |
| Konfiguration | §15 |
| Leistungshypothesen | §16 |
| Risiken, offene Entscheidungen | §17 |
| Entscheidungen | §18 |
| Zustandsautomaten, Korrekturprotokoll R13→R14 | Anhang A, B |

**Glossar (kanonische Begriffe).**

| Begriff | Bedeutung |
|---|---|
| Cluster | Löschdomäne = Schlüsseldomäne = Versionsdomäne = Platzierungseinheit; `ClusterId` 128 Bit, vom Kern zufällig erzeugt. `Sealed` (eigener Schlüssel) oder `Plain`. |
| Space | Benannter, physisch getrennter LSM-Baum (`SpaceId u64`, nie wiederverwendet). |
| Stamp / Epoch | Monotone `u64` (< 2^56) = Transaktionszeit = Commit-Reihenfolge. `Epoch(u64)` ist derselbe Wert; es gibt genau einen Typ. |
| Change / Guard / Receipt | Atomare Op-Menge / Vorbedingung / Quittung `(instance_id, stamp, durable)`. |
| Gate | Logisches Löschen: ab durablem Gate liefert der Cluster nichts mehr. |
| Packgruppe | `(tenant_tag, due_bucket, shard)`; `due=None ⇒ due_bucket=NONE`. |
| Partition | Einheit eines abgeleiteten Index (§11.5). |
| SnapshotSet | `(instance_id, V)` plus je Kanal/Partition `(partition, generation, applied_stamp)`. |
| Restriction | Rechtlicher Sperrvermerk (kein Override im Kern). Hold = technische Haltefrist (≤ 24 h). |
| Receipt | `ErasureReceipt`, Löschnachweis (§8.9). |

---

## 1. Scope, Ziele, Bedrohungsmodell

### 1.1 Mission und Ziele

Contextra beseitigt die Schwächen von LLM-Anwendungen im Unternehmen: fehlende Löschbarkeit, ungetrennte personenbezogene Daten, unzuverlässiger Kontext, Tokenverschwendung, fehlende Nachvollziehbarkeit.

| ID | Ziel | Umsetzung |
|---|---|---|
| M-1 | Europäischer Datenschutz von Anfang an | Löschung im Fundament (§8), Datentrennung (§11.9), Egress-Kontrolle (§11.9), Profile (§11.15) |
| M-2 | Performante, ressourcenschonende Gesamtlösung | gemeinsames `MemoryBudget`, Admission vor Allokation, partitionierte Indizes, SIMD-Dispatch |
| M-3 | Maximal stabiler Kern | synchroner Kern, Redo + Catalog, TLA+/Simulation/Fuzz als Gates (§12) |
| M-4 | Entwicklerfreundlichkeit | stabile Fehlertypen mit `hint()`, Profile statt Einzelschrauben, deterministische Tests |
| M-5 | Kundenfreundliche Funktionen | Receipt, Auskunft/Export, Restriction, Fristen, Zeitreise-Abfrage (§11.15) |
| M-6 | Hocheffiziente Suche | Vektor + BM25 + Graph + kalibrierte Fusion in einem Snapshot (§11.10) |
| M-7 | Token- und Kontextmanagement | Kopf/Gewichte/Seiten mit Budget, `DiBud` (§11.11) |
| M-8 | Transparenz | `SnapshotSet` je Antwort, `explain`, Receipt, Metrikvertrag |
| M-9 | Offener Kern, monetarisierbare Funktionen | §1.5 |

Technische Ziele:

- **Z-1** Atomarität über Spaces, Cluster und Branches (ein Redo-Frame, ein Gruppen-Fsync).
- **Z-2** Durability: sichtbar nur nach Fsync (Ausnahme `Relaxed`, nie für Löschung und nie mit `IdemToken`).
- **Z-3** Snapshot-Isolation ohne Lesesperren über Kern und Indizes (`Index ∪ Delta`).
- **Z-4** Gate und Schlüsselvernichtung unabhängig von der Storegröße; Purge begrenzt durch betroffene Dateien.
- **Z-5** Ablauf einer Packgruppe kostet O(Dateien) Purge plus O(Cluster/4096) Gate-Frames plus O(Cluster/256) Provideraufrufe (FIX-10).
- **Z-6** Nachweisbarkeit: Receipt, `verify_receipt` (offline), Orakel, Ledger, Anker.
- **Z-7** Kopf ohne Körper-I/O; Budgetseiten ohne Body-Lesen.
- **Z-8** Race-Freiheit durch Konstruktion, belegt durch Modell, Simulation, `loom`.
- **Z-9** Ein gemeinsames `MemoryBudget` für Kern und Indizes.
- **Z-10** Flüchtige Arbeitsbereiche (`Volatile`-Spaces).
- **Z-11** Idempotenz für Agenten-Retries (`IdemToken`).
- **Z-12** Löschung funktioniert bei vollem Datenträger (Reserven).
- **Z-13** Überwachbare Fristen mit Eskalation.
- **Z-14** O(1)-Fork, indexierter Diff, konfliktgeprüfter Merge je Cluster.
- **Z-15** Klassifikation ist ein geschlossenes Enum; ungültige Kombinationen werden vor dem ersten I/O abgewiesen.
- **Z-16** Observability ohne Inhalte: Label-Allowlist, keine IDs/Queries/Inhalte.
- **Z-17** Tail-Latenz: unter Rang 1 keine E/A, kein Provider-, Graph- oder Plattenzugriff.
- **Z-18** Metadaten-Skalierung: RAM, Dateien, FDs wachsen kontrolliert mit der Clusterzahl.
- **Z-19** Deutsche Unternehmenssuche: Morphologie, Komposita-Zerlegung, BM25/BM25F, WAND, HNSW/DiskANN + SQ8/RaBitQ + ACORN, kalibrierte Fusion.
- **Z-20** Sichere Hardwarenutzung: Runtime-SIMD-Dispatch isoliert in `contextra-simd`, bitidentisch zur Skalar-Referenz.
- **Z-21** PII- und Egress-Schutz über Ports `PiiClassifier`, `EgressPolicy`.
- **Z-22** Verifizierte, verlustfreie Ablösung von `contextra-store` ohne Dual-Write.

### 1.2 Nicht-Ziele

NZ-1 Verteilung/Konsens in v1 · NZ-2 eigene Kryptografie · NZ-3 Schemas, Sekundärindizes, SSI im Kern · NZ-4 Merge-Operatoren im Kern (Tag `6` reserviert) · NZ-5 Schutz von Prozessspeicher/Swap/Core-Dateien · NZ-6 Umordnung über Clustergrenzen · NZ-7 Kompatibilität zu `contextra-store`-SSTables (Migration per Export/Forward-Ingest) · NZ-8 Windows-Durability ohne bestandene Fs-Konformanz · NZ-9 `mmap`/`O_DIRECT`/`io_uring` im Kern oder unversiegelt in Indizes · NZ-10 Kompression in v1 (`COMPRESSION=0`) · NZ-11 Inhaltsadressierung/Dedup über Löschdomänen · NZ-12 ANN/BM25/Graph innerhalb von `contextra-storage` · NZ-13 clusterübergreifende Commit-DAGs/Merkle-Zustandshashes · NZ-14 Rechtsbewertung im Kern · NZ-15 Schlüsselverwaltung im Kern (`KeyProvider`-Port) · NZ-16 Mandanten-/Rechteprüfung im Kern (liegt in `engine`) · NZ-17 Async-Runtime im Kern · NZ-18 KV-/Prompt-Cache · NZ-19 zwei lebende Klone einer Instanz · NZ-20 Replikation/Failover vor eigenem Gate · NZ-21 Agenten-Orchestrierung und -Laufzeit · NZ-22 kognitiver Zustand (Decay, Session-DAG) · NZ-23 Modell-Hosting, Inferenz, Routing · NZ-24 Parser/Sandbox für untrusted Formate (Client/Plugin).

Verteilbarkeit vorbereiten: IDs und Stamps sind instanzlokal; jedes externe Ereignis, Watermark und Receipt führt `(instance_id, stamp)` und sortiert Stamps nie global. Restore erhält die `InstanceId` und startet schreibend erst nach extern belegtem Fencing des bisherigen Besitzers.

### 1.3 Bedrohungsmodell

| ID | Angreifer/Fehler | Gegenmaßnahme |
|---|---|---|
| AT-1 | Leser des Datenverzeichnisses ohne Schlüssel | AEAD-Siegel auf Redo/Runs/Blobs/Indexblöcken (§4.2) |
| AT-2 | Datenträgerbesitzer ohne Schlüssel | Schlüssel nur über `KeyProvider`/KMS (§8.4) |
| AT-3 | Dateimanipulation, Index-Manipulation | Ketten-MAC, AAD mit `op_uid`/`block_uid`/`record_uid`, Katalog-Kette, schlüsselgebundene Index-MACs; keine statischen Schlüssel |
| AT-4 | Stale/bösartiger Index | nur versiegelte `SegmentIo`-Wege; `filter_visible` vor Ausgabe; `O-Segments`/`O-Files` |
| AT-5 | Rollback auf Backups, um Gelöschtes zurückzubringen | Ledger (`tomb_id`) + monotoner Anker + Restore-Reconcile + Backup-Horizont (§9) |
| AT-6 | Löschwunsch gegen aktive Restriction | Prüfung im Sequencer (`policy_epoch`) und im Purge |
| AT-7 | Seitenkanal über globale Index-Metadaten (seltene Komposita, Filterlatenz) | mandantenreine Partitionen, Neuaufbau aus lebenden Dokumenten, Refcount-Löschung, keine Inhalte in Metriken |
| AT-8 | PII-Abfluss an Cloud-LLMs; Prompt-Injection über Inhalte | `EgressPolicy` (Standard `DenyExternal` für `Personal`), Pseudonymisierung vor Egress, Trust-Label im Kopf; Injection-Filter beim Client/Plugin |
| AT-9 | Bösartige Uploads (PDF, ZIP, Office) | Parsing außerhalb des Workspace; Contextra nimmt nur extrahierten Text mit Größen-/Zeichenprüfung an |
| AT-10 | Mandantenübergreifender Leak durch Engine-Fehler | Kern kennt keinen Mandanten (NZ-16); Gegenmittel: mandantenreine Partitionen, `tenant_tag` als zufälliger, geheimer Bezeichner (nie aus der Mandanten-ID ableitbar), Differentialtests |
| F-1…F-9 | Torn Write, Fsync-EIO, ENOSPC, Bit-Rot, Schlüsselverlust, Crash im Löschprotokoll, Retry-Sturm, Index-Lag | §2.5, §9, §11.4 |

### 1.4 System-Annahmen

AN-01 `sync_*` macht durabel · AN-02 Schreibvorgänge können zerreißen · AN-03 Fsync-Fehler sind nicht wiederholbar (Handle vergiftet) · AN-04 `rename` im selben Verzeichnis ist atomar · AN-05 Prozessspeicher ist vertrauenswürdig, OS/Swap-Risiko wird deklariert · AN-06 `destroy_many` macht Schlüssel bei `Absent` unbenutzbar · AN-07 starker CSPRNG (`Entropy`) · AN-08 `Anchor` liegt außerhalb des Datenverzeichnisses · AN-09 Stamps nie aus der Wanduhr; TTL/Deadline nutzen monotone Zeit + UTC + durablen `clock_floor` (`max_clock_skew` ⇒ `ClockUncertain`, fail-closed) · AN-10 Append-Dateien schreiben nur in vorallokierte Bereiche.

### 1.5 Open-Core-Grenze

Offen: alle Formate, Speicherkern, Ports, Retrieval-Triebwerke, Engine, Referenz-Provider/-Anker (`FilePassphrase`, `FileAnchor`), Konformanzvektoren, Simulation, Orakel, Branching, Basis-Löschung, Export/Restore-Werkzeuge, `verify_receipt`. Kommerziell (Plugins/Editionen): HSM-/Cloud-KMS-Provider, `Strong`-Anker, Replikation (nach eigenem Gate), zentrale Mandanten-Orchestrierung, Compliance-/AVV-Portale, NER-basierte `PiiClassifier`, Kalt-Tier. `LicenseGate` DARF KEINE Kerninvariante abschwächen und Basis-Durability/-Löschung nicht hinter eine Bezahlschranke legen.

---

## 2. Architektur

### 2.1 Crates und Ebenen (`INV-TOPO-1`, `INV-LAYER-1`)

Der Workspace besteht aus genau **14 Crates**; Neuanlage nur durch Änderung dieses Abschnitts plus Gate. Abhängigkeiten zeigen nur auf strikt niedrigere Ebenen; innerhalb einer Ebene gibt es keine Kanten.

```text
L3  contextra-engine      Composition Root, Async-Rand; Module: platform · kernel · projection · collection · query · ingest · compliance · workers
L2  contextra-storage     Speicherkern, sync                         → ports(kernel), types
    contextra-crypto      Cipher/Mac/Kdf/Checksum/KeyProvider, Beleg-Signatur → ports(kernel), types
    contextra-vector      HNSW/DiskANN/SQ8/RaBitQ/ACORN              → ports(retrieval), types
    contextra-text        BM25/WAND, deutsche Morphologie            → ports(retrieval), types
    contextra-graph       CSR, Hyperkanten, PPR/PathRAG              → ports(retrieval), types
    contextra-rank        Fusion, Kalibrierung, Drift, DiBud         → types
    contextra-mvcc        EpochPinRegistry                           → types
    contextra-checkpoint  Snapshot-Set-Register, Restore-Koordination → ports(kernel), types
L1  contextra-ports       Module: kernel · retrieval · policy · system · legacy (Feature, bis G-CUTOVER)
    contextra-durable-fs  sichere Dateischicht (Poisoning, Journal, Scrub, FaultInjector) → sys
L0  contextra-types · contextra-sys (Unsafe-Insel) · contextra-simd (Unsafe-Insel)
```

Regeln:

1. `storage` kennt nur `ports::kernel` und `types`. `vector`/`text`/`graph`/`rank` kennen weder `storage` noch einander. **Nur `engine` kennt Kern, Krypto, Dateischicht, SIMD und Retrieval-Crates.** Der Zusammenschluss geschieht ausschließlich in `engine::projection`.
2. **Persistenz-Verträge der Retrieval-Crates** (`PartitionSink`/`PartitionSource`: Blöcke schreiben/lesen, Generation sealen, `apply(batch, epoch)`, `rewrite_without`) werden **je Crate in dessen `contract`-Modul definiert** (FIX-06). `ports::retrieval` enthält nur die Lese-Verträge `VectorIndex`/`TextIndex`/`GraphIndex` (`search`, `search_at(epoch)`, `traverse*`, `stats`). `engine::projection` implementiert die Sinks über `SegmentIo`.
3. `SegmentOwner`, `ExtensionBlock`, `BlockPruner`, `Consumer`/`ConsumerFeed` und `View::filter_visible` sind **die fünf Erweiterungsverträge** (FIX-28); sie sind im öffentlichen API von `contextra-storage` definiert (§11). Implementierer ist `engine::projection`.
4. `checkpoint` kennt `storage` nicht: es definiert die Traits `SnapshotParticipant` (Pin, Manifest, Restore-Schritt) und `RestoreStep`; `engine` bindet Kern und Index-Generationen als Teilnehmer (FIX-07). `mvcc` hängt nur an `std` und `types`.
5. `rank` ist synchron und zustandslos; Persistenz von Kalibratoren liegt in `engine` (`NonPersonal`-System-Space).
6. Distanzfunktionen: `vector` nutzt `contract::DistanceFn`; `engine` injiziert `contextra-simd`.
7. `retrieval`/`policy`/`system`-Ports dürfen `futures`/`serde` nutzen; L2-Crates rufen nie `.await`/`block_on`. `block_on` existiert nur in `engine::kernel`.

**Modulplan `contextra-engine`:**

| Modul | Aufgabe |
|---|---|
| `platform` | Adapter `Fs` über `durable-fs`/`sys`, `Clock`, `Entropy`, `HostProbe`, `FileAnchor` |
| `kernel` | Kern öffnen (Ports verdrahten, `ComplianceProfile` prüfen), Blocking-Pool als Async-Fassade, Fehlerabbildung |
| `projection` | `SegmentOwner`/`ConsumerFeed`/Watermark/Attest je Index, Partitionen, Generationsschlüssel, `rewrite_without`, `verify_absent`, Delta-Pfad, `Stamp↔Epoch` |
| `collection` | Collection → `SpaceId` je (Collection, Schutzklasse), `TenantPolicy`, CRUD, Relationen |
| `query` | Query-Plan QP-1…QP-9, `SnapshotSet`, Fusion, Tokenbudget, `min_stamp` |
| `ingest` | Chunker, Pseudonymisierung (`PiiClassifier`), Publish-Record, Quarantäne, Orphan-Worker |
| `compliance` | Profile, Receipt→`DeletionProof`, Export, Retention |
| `workers` | Wartung, Projektion, Rebuild, Aufräumen |

**Modulschnitt `contextra-storage`** (nur abwärts, Test `layering`): `engine` (sequencer/syncer, memtable, version, views, branches, registry, extension, feed, page, cursor, cache, idem, guard_window, graveyard, volatile) · `maint` (flush, compaction, blob_gc, purge, scheduler, scrub) · `erase` (gate, cluster_table, links, protocol, ledger) · `recovery` · `checkpoint` · `catalog` · `store` (run, blob, redo) · `keys` · `codec` · `env` (Clock, Entropy, Governor, Metrics; MemFs, FaultFs, Sim) · `sync` (Ränge) · `testing` (Orakel, Feature `testing`).

### 2.2 Ports

**Kernel-Ports** (`contextra-ports::kernel`, `INV-PORT-1`): synchron, ohne `futures`, `serde`, `std::fs`; jeder Adapter besteht die Konformanzsuite (`ports::conformance`).

| Port | Vertrag | Standard-Adapter |
|---|---|---|
| `Fs` | Dateien, `read_at`, `read_ranges`, `append_prealloc`, `preallocate`, `sync_data/all/dir`, `rename`, `lock`, `list`, `free_space`, optional `link/reflink/punch_hole`; `FileRole`, `IoCtx`; kein `mmap` | `engine::platform` über `durable-fs`; Test: `MemFs`/`FaultFs` |
| `Clock`, `Entropy` | injizierbar; Uhr nie für Stamps; `clock_floor`; Skew-Status | `engine::platform` |
| `Anchor` | `read()`, `advance(expect,next)` (CAS, monoton, durabel), `strength() → Strong\|Weak`; Wert `(erase_epoch, ledger_head, catalog_epoch)` | `FileAnchor` (`Weak`); `Strong` per Plugin |
| `KeyProvider` | §8.4 | `crypto::keys` (`FilePassphrase`, Argon2id; `cohort` optional); KMS/HSM per Plugin |
| `Cipher`/`Mac`/`Kdf`/`Checksum` | XChaCha20-Poly1305 (24-B-Zufallsnonce), HMAC-SHA256 (16 B/32 B), HKDF-SHA256, CRC32C | `contextra-crypto` |
| `Authority` (opt.) | prüft Signaturen für `LiftPolicy::FourEyes` | Plugin |
| `Metrics`, `Observer` | nicht blockierend, nie unter Sperren, geschlossener Katalog (§15) | `engine::kernel` |
| `MemoryBudget` | `try_reserve(class, bytes) → ResourcePermit` für Kern und Extensions | `engine::kernel` |
| `HostProbe` (opt.) | Swap, Core-Dumps, memlock, FS-Typ, Geräte-IDs; nur Diagnose, nie Korrektheitsquelle; `Unknown` ⇒ `Declared` | `engine::platform` |
| `ColdTier` (opt.) | Auslagerung kalter Dateien unter Erhalt der Purge-Garantien | Plugin |

**Retrieval-/Policy-/System-Ports** (`ports::retrieval|policy|system`): `VectorIndex`/`TextIndex`/`GraphIndex` (nur lesend), `EmbeddingProvider`, `Reranker` (Feature), `TextGenerator` (schmal: Extraktion/Query-Umschreibung, kein Streaming), `EgressPolicy`, `PiiClassifier`, `IdentityResolver`, `ContextPreparer`, `GroundingValidator`, `ReportSigner`, `LedgerSink`, `ErasureObserver`, `LicenseGate`, `PluginManifest`.

### 2.3 Mechanismus, nie Politik

Der Kern entscheidet nie, ob gelöscht werden darf, wer zugreift oder welche Frist gilt; das verantworten `engine` (`TenantPolicy`, Restrictions), `ingest` (`PiiClassifier`, `DataClass`) und `compliance`. Der Kern setzt Aufträge atomar, geordnet und nachweisbar um und verweigert nur bei technischem Hindernis oder aktiver Restriction. Er persistiert keine Inhalts-Hashes über Klartext (`INV-PRIV-1`).

### 2.4 Threading

```text
Async-Rand (tokio, nur engine)  ──spawn_blocking (engine::kernel)──►
Kern (sync):  Aufruferthread: Stufe-A-Guards, Hülle, Krypto, Vorkodierung
              ► Sequencer (Rang 1, nur RAM, Flat-Combining-Leader) ► Syncer (1 Thread, kein Lock: write + sync_data ► Publikation)
              Lesen: lock-frei über Arc<Version> + Atomics
              Wartung: Lanes urgent > flush > compact > blobgc > idle
              Ledger/Anker: je 1 Schreiber über Kanal (urgent, gebündelt)
Retrieval: eigener ComputePool (Distanz, Quantisierung); keine Kernsperre über CPU- oder await-Wartezeit
```

Provider-, Cipher-, KDF-, Graph- und Guard-Arbeit läuft ausschließlich vor Rang 1 oder auf Wartungs-Lanes.

### 2.5 Fehlermodell

| Klasse | Beispiele | Verhalten |
|---|---|---|
| F-1 transient | `EINTR` | Wiederholung mit Backoff |
| F-2 Ressource | `ENOSPC` beim Vorallokieren, `DiskFull`, `Limit{..}` | kein Poison; User-Commits abgewiesen; Löschung/Wartung laufen mit Reserve (§5.6) |
| F-3 E/A-hart | `EIO`, Fsync-Fehler | nie auf demselben Handle wiederholen; Handle `Poisoned`; Löschaufträge bleiben über das Ledger aufnehmbar (§8.7); begonnene Changes `Indeterminate{first,last}`, Retry nur mit `IdemToken` |
| F-4 Integrität | CRC/MAC/AEAD | kleinste Einheit vergiften (Block → Abschnitt → Datei → Index-Segment), `Corrupt{..}`; gesunde Cluster laufen weiter |
| F-5 Format | `Unsupported` | Öffnen/Dekodieren verweigert |
| F-6 Schlüssel | Provider offline / Schlüssel fehlt | `KeyUnavailable{retry_after}`; `KeyLost` bei fehlendem Schlüssel ohne Löschnachweis (Quarantäne, Alarm). Nach Gate liefern Sichten `Erased`; `Shredded` ist Clusterzustand, nie Lesefehler |
| F-7 Aufrufer/Index | `Conflict`, `Restricted`, `UnknownCluster`, `BelowFloor`, `ReservedSpace`, `SnapshotTooOld`, `ErasePending`, `LedgerUnavailable`, `LedgerSameDevice`, `BackupBeyondHorizon`, `ProviderAssuranceInsufficient`, `IndexLag`, `IdemConflict`, `IdemCapacity`, `InvalidInput` | typisiert mit stabilem `hint()` |
| F-8 Logik | Invariante verletzt | Test: Abbruch; Produktion: Poison + Observer; kein `panic` |
| F-9 Rollback | Anker höher als Catalog/Ledger | schreibend verweigert (`Err(Rollback)`), nur `ReadOnly` mit Diagnose |

Fehlertexte enthalten nie Schlüssel, Nutzerschlüssel, Inhalte, Queries oder Embeddings.

### 2.6 Lints und Unsafe-Inseln

`[workspace.lints]` auf `deny`: `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `unreachable`, `indexing_slicing`, `await_holding_lock`, `await_holding_refcell_ref`, `exit`, `dbg_macro`; `let _ =` auf `sync_*`/`rename`/`unlink` verboten. `Debug` für `ClusterId`, Schlüssel, Nutzdaten, Query-Texte ist `Redacted<T>`.

`#![forbid(unsafe_code)]` gilt in **allen Crates außer `contextra-sys` und `contextra-simd`**. Nur diese beiden sind Unsafe-Inseln (`unsafe_op_in_unsafe_fn = deny`, vollständige `SAFETY`-Kommentare); Fremd-`unsafe` nur in auditierten RustCrypto-/Crossbeam-Crates. Test `unsafe_islands` erzwingt das (FIX-26).

### 2.7 Abhängigkeiten (normativ für alle `Cargo.toml`)

Auswahl in Rangfolge: (1) `std` genügt; (2) `forbid(unsafe)`-kompatibel; (3) kein Async im Kern und in L2-Algorithmuskernen; (4) ein Crate je Aufgabe, eine Workspace-Version je Abhängigkeit.

**`contextra-storage`:** `thiserror ^2`, `bytes ^1`, `zeroize ^1`, `crossbeam-skiplist ^0.1` (nur hinter Trait `MemIndex`; Fallback sharded `BTreeMap` für `loom`), `siphasher ^1` (nur flüchtige Guard-Fingerabdrücke), `std::sync` + `RankedMutex`. Handgeschriebener `codec`. **Verboten im Kern:** `tokio`/`async-std`/`futures`, `rayon`, `parking_lot`, `scc`, `dashmap`, `memmap2`, `serde*`, `bincode`, `blake3`, `sha2`, `rand`/`getrandom`, `crc32fast`, `anyhow`, `tracing`. `verify_receipt` nutzt nur den `Mac`-Port (FIX-21).

**`contextra-crypto`:** `chacha20poly1305 ^0.10`, `hmac ^0.12` + `sha2 ^0.10`, `hkdf ^0.12`, `argon2 ^0.5` (nur Passphrase-Provider), `crc32c ^0.6` (zwingend CRC-32**C**), `subtle ^2`, `getrandom ^0.3` (nur im `Entropy`-Adapter), `ed25519-dalek ^2` (`verify_strict`, Signaturbytes handcodiert). Entfernen: `blake3`, `aes-gcm-siv`, `bincode`, `tokio`, `kv_cipher`, `kv_shredding`, `kv_segment`. Behalten/anpassen: `aead`, `dek`, `kdf`, `secret`, `audit_chain`, `revocation_log`, `anti_tamper`, `deletion_proof`.

**Dateischicht:** `sys` = einziger Ort für OS-FFI (`F_FULLFSYNC`, `FlushFileBuffers`, `fdatasync`, `mlock`, ACL, optional `mmap` hinter Feature, Standard aus). `durable-fs` = sichere Politikschicht über `sys` (Poisoning `INV-FSYNC-1`, atomares Ersetzen `INV-VFS-ATOMIC-1`, Journal, Scrub, `FaultInjector`), erweitert um positionsbasierte Handles (`read_at`, `write_at`, `preallocate`, `lock`, `free_space`, `sync_dir`); kein `libc`/`memmap2`; Prüfsumme CRC32C über `crc32c` statt `crc32fast` (FIX-21). Der Kernel-`Fs` ist ein dünner Adapter (≤ ≈ 400 LOC) in `engine::platform`.

**Retrieval/Rest:** `types`: `thiserror 2`, `bytes`, `ahash`; `serde`/`bincode`/`blake3`/`sha2` nur hinter Feature `wire-compat`. `vector`: `ahash`, `roaring`, `subtle`, `zeroize`. `text`: `bytes`, `itoa`, `regex`, `unicode-segmentation`. `graph`: `bytes`, `ahash`, `arc-swap`. `rank`: `ahash`; `serde` nur für Config/Reports. `engine`: `tokio` + `tokio-util` (nur Rand), `serde`/`serde_json` (API). L2-Crates ohne `tokio`, `bincode`, `blake3`, `memmap2`, direktes `std::fs`, statische Schlüssel (`deny.toml` + `check_forbidden_apis.sh`).

**Werkzeuge:** `loom ^0.7`, `shuttle ^0.8`, `proptest ^1`, `cargo-fuzz`, `miri`, `cargo-mutants`, `cargo-llvm-cov`, `cargo-nextest`, TLA+/TLC, `criterion`, `tempfile`, `cargo deny` (Positivliste, `ignore` nur mit Begründung und Ablaufdatum).

---

## 3. Datenmodell

### 3.1 Leitentscheidungen des Kerns

| # | Entscheidung |
|---|---|
| L1 | Cluster = Löschdomäne = Schlüsseldomäne = Versionsdomäne = Platzierungseinheit. |
| L2 | Ordnung geclusterter Spaces: `(cluster, branch, user_key, tag↓)`; Blöcke clusterrein; je Datei ein zusammenhängender Abschnitt je Cluster mit abschnittsrelativen Offsets. |
| L3 | Cluster sind `Sealed` oder `Plain`. Alle Nutzdaten eines Sealed-Clusters sind unter dessen Schlüssel versiegelt (`INV-SEAL-1`). |
| L4 | Eine Sequenzierungsstelle (Rang 1, ohne E/A) vergibt alle Stamps und linearisiert Gate, Control, Link, Branch-Ops; ein Syncer schreibt und synchronisiert das Redo in Stamp-Reihenfolge. Dateien sind unveränderlich. |
| L5 | Catalog = einzige Wahrheit über Dateien. Lebendig ⇔ durabler Edit nennt die Datei. |
| L6 | Löschreihenfolge ist Vertrag (§8.5); nur Roll-forward. |
| L7 | `DerivedFrom`-Links kaskadieren die Löschung im selben Gate-Frame. |
| L8 | Restriction verhindert Löschung; kein Override im Kern. |
| L9 | Eine Siegelform für alles (XChaCha20-Poly1305, 24-B-Nonce), unabhängig von Datei und Position. |
| L10 | Ein Purge-Algorithmus für Pure, Packed, Staging, Packgruppen-Abwurf. |
| L11 | Branches sind clusterlokal und sterben mit dem Cluster. |
| L12 | **Ein-Wahrheits-Commit:** kanonische RAG-Daten committen in einem `Change`; abgeleitete Indizes projizieren idempotent aus dem `ConsumerFeed` (`Index ∪ Delta` + `filter_visible`). Kein 2PC, keine Kompensation. |
| L13 | Löschkonforme Indexklassen: abgeleitete Strukturen existieren nur als `ClusterSections` oder als mandantenpartitionierte `Rebuildable`-Owner mit synchronem `on_gate`-Filter, Rewrite-Grenze und `verify_absent`-Attest. |
| L14 | Eine Versions-/Zeitachse `(instance_id, Epoch)` und ein gemeinsames `MemoryBudget`. |
| L15 | Konzepttrennung: `DerivedFrom` (Lösch-DAG) ≠ Wissensgraph; `Eraser` ≠ Recency-Prior (löscht nie) ≠ Retention (löst `Eraser` aus); Kern-`Checkpoint` ≠ Snapshot-Set-Register. |
| L16 | Retrieval-Crates sind persistenzneutral; Index-Partition = Packgruppe(-Slice); personenbezogener Klartext nur in Sealed-Clustern; große Dokumente werden über Publish-Record atomar sichtbar. |

### 3.2 Reihenfolge, Tag, Marker

`cmp`: Cluster aufsteigend, Branch aufsteigend, Nutzerschlüssel bytelexikografisch, Tag absteigend (kein `memcmp` über Konkatenation). Ohne `clustered` ist der Cluster implizit `NONE`.

Tag `u64 = (stamp << 8) | kind`; `Put=1, Delete=2, RangeDelete=3, EraseRange=4, PutRef=5 (nur in Runs), Merge=6 (reserviert; Decoder: Unsupported)`. `RangeDelete` ist branch-lokal, `end` exklusiv. `EraseRange` ist ein clusterweiter Control-Marker ohne Branch, wirkt auf alle Branches und historischen Views und kann nie gemergt werden. Bei Sealed-Clustern bleiben Markerart, Routing und Stamp im Klartext; `start/end` sind clusterschlüssel-versiegelt.

Stamps: erster = 1, streng monoton, nie doppelt (auch nicht nach Absturz), Überlauf ⇒ `Exhausted`, Lücken zulässig.

### 3.3 Cluster

- **Erzeugung:** `Clusters::register(ClusterSpec)` zieht 128 Bit aus `Entropy`, prüft Kollision, legt bei `Sealed` **zuerst** den Schlüssel an (`create_cluster_keys`, Batch, idempotent je `BatchId`, durabel), dann Catalog-Edit `cluster_register`. Ein Schlüssel ohne Cluster wird nach `orphan_key_grace` (24 h) über `KeyProvider::list` vernichtet.
- **Zustände:** `Live → Retired(S) → [Shredded] → Purged → (Trim)`. Zwischen `Live` und `Retired` existiert nur im Sequencer ein flüchtiges `Retiring(S)`: es schließt die Schreibadmission, ist für Leser nicht sichtbar und nicht persistiert. `Shredded` nur bei `Sealed`. Nur vorwärts.
- **Trim:** `Purged` wird per `cluster_trim` entfernt, sobald `history_floor > S`; danach `UnknownCluster`. ClusterIds werden nie wiederverwendet. Der Dauernachweis liegt als `tomb_id` im Ledger.
- **`ClusterSpec`** (unveränderlich nach `register`): `protection` (`Sealed|Plain`), `tenant_tag` (16 B, opak, vom Aufrufer, zufällig und geheim, nie aus der Mandanten-ID abgeleitet), `due` (optional; `None ⇒ kein Ablauf`), `erase_locality`, `expected_bytes` (nur Admission-Hinweis). v1-Default `PackedBounded{max_file_bytes: 64 MiB}`. `ClusterLocal` = Run-/Blob-Dateien enthalten ausschließlich Abschnitte dieses Clusters; nur nach Admission (§3.4); nie still herabgestuft; von Pure-Demotion ausgenommen.
- **Retention-Verlängerung (FIX-22):** `due` ist unveränderlich. Eine Verlängerung der Aufbewahrung über `due` hinaus wird durch eine `Restriction{BLOCK_ERASE, reason=RetentionExtension}` ausgedrückt; `pack_expired` überspringt Restricted-Cluster und weist sie im Aggregat aus. Eine Verkürzung ist ein normaler `Eraser`-Auftrag.
- **Packgruppen-Limit:** `max_pack_groups` (Standard 4096) begrenzt die aktiven Gruppen je Space; eine Registrierung, die eine neue Gruppe über dem Limit anlegen würde, ergibt `Err(Limit{PackGroups})`. Deployments mit mehr Mandanten erhöhen das Limit (RAM O(Gruppen)) oder vergröbern `due_bucket`/`shards` (FIX-14).

### 3.4 Admission für Cluster, Dateien und Metadaten (`INV-ADM-1`)

Jede Ressource unterliegt einer Admission mit eigener Fehlerklasse, **vor** der Allokation. Technischer Ressourcenschutz, keine Politik.

| Struktur | Residenz | Kosten | Budgetklasse |
|---|---|---|---|
| `ClusterIndex` (offene Adressierung, Schlüssel `ClusterId`) | immer resident | 36 B je Eintrag (id 16, `state:8\|gen:56` 8, `pack_group` 4, `hot_ref` 4, flags 4); Rechenwert 40 B inkl. Slack | `ClusterMeta` |
| `HotRecord` (Branch-Tabelle, Restrictions, Links, Pins, Views, Cursor) | nur bei Bedarf; kaltstellbar, **nicht** bei Restriction/Link/Pin | ≤ 256 B + Inhalt | `ClusterHot` |
| `ClusterDirectory` je Datei | Sparse-Index (≈ 0,3 B/Cluster/Datei) + `ClusterFilter` (≈ 1 B) | ≈ 1,3 B | `DirIndex`, `BlockCache` |

16 Mio. Cluster × 40 B ≈ 610 MiB, unabhängig von der Dateizahl. `resident_clusters_max = cluster_meta_budget / 40 B`, `cluster_meta_budget` = 15 % von `memory_budget_bytes` (Standard 1 GiB ⇒ ≈ 3,9 Mio. Cluster).

Regeln:
1. `register` reserviert `ClusterMeta`; Fehlschlag ⇒ `Limit{ClusterMetaBudget}` ohne Teilzustand. `open` öffnet bei zu vielen Clustern `ReadOnly` mit Diagnose.
2. `ClusterLocal` nur wenn `expected_bytes ≥ cluster_local_min_bytes` (= `pure_threshold`, 8 MiB), lebende `ClusterLocal`-Cluster `< max_cluster_local_clusters` (1024, ≤ 65 536) und `max_live_files` Reserve für zwei Dateien je Level lässt; sonst `Limit{ClusterLocalAdmission}`. Für Massen personenbezogener Cluster gilt `PackedBounded` + Crypto-Erase.
3. Pure-Demotion (R-PL-3): nicht-`ClusterLocal`-Cluster unter `pure_demote_ratio × pure_threshold` (0,25) gehen per `rebucket` in ihre Packgruppe zurück.
4. `max_live_files` (262 144): ab 80 % hebt der Governor `pure_threshold` je Stufe ×2; bei 100 % entstehen keine neuen Pure-Dateien. Gate, KeyDestroy, Purge bleiben ausführbar.
5. FD-Cache: `max_open_files` (1024) ist LRU-Größe, kein Dateilimit; `urgent` hat `fd_reserve` (256). `open` prüft `RLIMIT_NOFILE ≥ max_open_files + fd_reserve`, sonst `FdLimitTooLow`.
6. Verzeichnis-Sharding (§4.4).
7. **Konvention „Session ≠ Cluster“:** Cluster folgt der Löschdomäne (§11.13), nie der Sitzung.

### 3.5 Wert, Kopf, Körper, Gewichte

Wert = `header ‖ body`, getrennt durch `header_len`. Der Kern interpretiert den Kopf nie. `weights: [u32; W]` (`W = weight_slots`, 0…4); Block-Index hält `weight_sum[W]` und `entry_count`. Körper ≥ `value_separation_min` (1 KiB) gehen in Blobs (`PutRef`); der Kopf liegt immer inline.

### 3.6 `SpaceOptions` (unveränderlich)

```rust
pub struct SpaceOptions {
    pub class: SpaceClass,            // Durable | Volatile
    pub clustered: bool,
    pub data_class: DataClass,        // NonPersonal | Personal (Personal erzwingt Cluster, Sealed)
    pub integrity: IntegrityMode,     // Crc | Authenticated | Encrypted   (FIX-18)
    pub packing: PackPolicy,          // None | ByTenantAndExpiry{ bucket: Month|Quarter|Year, shards: u8 }
    pub pure_threshold: u64,          // 8 MiB
    pub seal: Seal,                   // ByMode | Cluster
    pub weight_slots: u8,
    pub history: HistoryPolicy,       // KeepLatest | KeepUntilFloor | AppendOnly
    pub value_separation_min: u32, pub max_key_bytes: u32,
    pub compression: Compression,     // v1: None
    pub durability_default: Durability,
}
```

`DataClass::Personal` verlangt `integrity = Encrypted`; bei Durable zusätzlich `seal = Cluster` und ausschließlich `Sealed`-Cluster. Validierung vor I/O; kein PII-Detektor. Ein Space enthält nur Cluster einer Schutzklasse (`INV-SEAL-1`); die Engine legt je (Collection, Schutzklasse) einen eigenen Space an. **Volatile-Spaces:** kein Redo/Run/Blob, im Catalog nur der Name, sofort sichtbar, Inhalt verschwindet bei Schließen/Absturz, Erase = Freigabe mit Überschreiben (best-effort). Mischchanges mit Volatile- und Durable-Zielen nur mit `allow_mixed`, `durable` nur, wenn alle Ziele Durable sind (`INV-VOL-1`). `compression ≠ None` ⇒ `Unsupported` vor Mutation.

### 3.7 Limits

`max_key_bytes` 1 KiB (≤ 4 KiB) · `max_header_bytes` 4 KiB (≤ 64 KiB) · `max_body_bytes` 4 MiB hart · `max_change_ops` 4096 · `max_change_bytes` 8 MiB · `max_body ≤ max_change/2` · `max_page_bytes` 1 MiB · `max_clusters` 16 Mio. (weich, ≤ 64 Mio., effektiv durch `cluster_meta_budget`) · `max_branches_per_cluster` 1024 · `max_branch_depth` 4 (Default), 8 hart (Tiefe > 4 nur mit `BranchOptions::deep_ack`). Überschreitung ⇒ `Limit`/`ChangeTooLarge` vor jeder Sperre (`INV-LIM-1`). §3.7, Ports-Konstanten und Config verwenden dieselben Werte.

### 3.8 Datenklassen

`DataClass ∈ {NonPersonal, Personal}`; fehlende oder unbekannte Klassifikation wird an API-Grenzen abgewiesen. Der Builder darf `NonPersonal` nur für explizit nicht-personenbezogene System-/Anwendungsdaten wählen.

| Klasse | Ablage | Semantik |
|---|---|---|
| `NonPersonal` | geclustert oder nicht, gemäß Optionen | Schutz folgt `Seal`; keine Löschzusage allein aus der Klassifikation |
| `Personal` | nur geclusterter Space, `Encrypted`; Durable ⇒ `Seal::Cluster` und `Sealed`-Cluster | Cluster ist kleinste Crypto-Erasure-Domäne; Abgeleitetes erbt die strengere Klasse (`DerivedFrom` oder registrierter Owner) |

`Personal` im `Volatile`-Space hat keine Durability und erzeugt keinen Löschnachweis. Space-Wechsel/Export darf die Klasse nie herabstufen. Cross-Cluster-Referenzen zwischen Personen begründen keine gemeinsame Löschdomäne.

---

## 4. Formate

### 4.1 Platzierung

- **R-PL-1** Jede Datei ist Pure (ein Cluster), Packed (mehrere Cluster einer Packgruppe) oder Staging (nur L0, beliebige Cluster, nur Runs).
- **R-PL-2** Je Datei ein zusammenhängender Abschnitt je Cluster, nach ClusterId sortiert.
- **R-PL-3** Cluster ≥ `pure_threshold` wird bei der nächsten Compaction Pure; Demotion siehe §3.4.
- **R-PL-4** Compaction mischt nur innerhalb einer Wartungsgruppe (Packgruppe oder Pure-Cluster).
- **R-PL-5** Ein Flush schreibt je Space eine Staging-Datei plus je großem Cluster eine Pure-Datei.
- **R-PL-6** `rebucket` ist ein atomarer Umzug in einem Edit.
- **R-PL-7 Spill:** Staging-Runs werden size-tiered zusammengeführt. Gruppe `g` wird gespillt, wenn `staged_bytes(g) ≥ max(spill_min_bytes, size(L1(g))/spill_ratio)` (8 MiB, 4), oder das älteste gestagte Byte älter als `spill_max_age` (6 h) ist, oder ein Purge/`rebucket` die Gruppe berührt. Schreibverstärkung je Spill ≤ 1 + `spill_ratio`. `max_staging_runs` 16.
- **R-PL-8 MiniSection:** Abschnitte ≤ `mini_section_max_bytes` (4 KiB) bestehen aus **einem** SealFrame (Einträge, Range-Deletes, ChangeIndex, BranchTable); Overhead 40 B. Umstellung auf `FullSection` bei Überschreitung (Hysterese 50 %).

### 4.2 Siegelrahmen (eine Form für alles)

```
SealFrame := key_epoch u32 | nonce[24] | ciphertext | tag[16]       (Authenticated: nur tag; Crc: CRC32C)
AEAD  = XChaCha20-Poly1305(quelle, nonce[24], AAD)
quelle = Clusterschlüssel(cluster, epoch)  bei Seal::Cluster (Sealed-Cluster; im Redo immer mit der im Op-Header gebundenen key_epoch)
       | Space-DEK                          bei Seal::ByMode / Encrypted (Plain-Cluster)
AAD:   Block     (instance_id, space_id, cluster_id, key_epoch, block_uid)
       Redo-Op   (instance_id, space_id, cluster_id, key_epoch, op_uid, op_index)
       Blob      (instance_id, space_id, cluster_id, key_epoch, record_uid)
```

`space_id` ist unsigned BE-u64; AAD-Domain und Feldreihenfolge sind Teil der Referenzvektoren. Keine zusätzliche HKDF-Ableitung je SealFrame (die interne HChaCha-Ableitung genügt; Referenzcode-Vorstufe `derive_subkey_fast` ist vor G-F zu entfernen). `Authenticated` = erste 16 B von HMAC-SHA256 über `ctxs-auth-v1`, Modus, Nonce, AAD, Klartext; constant-time. `Crc` bietet keine Manipulationsauthentizität. Der RootKey-abgeleitete Redo-Schlüssel dient ausschließlich dem Ketten-MAC. AAD hängt nicht von Dateiname, Datei-ID, Blob-ID, Offset ab; `block_uid`/`record_uid` sind bei Rohkopie stabil ⇒ Rohkopie ist ohne Entsiegeln gültig. Versiegelt wird vor Stamp-Vergabe; der Stamp ist über die Frame-Kette gebunden.

`INV-SEAL-1`: Sealed-Cluster haben Nutzdaten nur in `Seal::Cluster`- oder Volatile-Spaces; Plain-Cluster nie in `Seal::Cluster`-Spaces; `Personal`-Spaces nehmen nur Sealed-Cluster. **Klartext bleiben ausdrücklich** (Metadaten-Leckagebudget, keine Nutzwerte): ClusterId, Op-Tags/Routing-Header, Gate-/Control-Frames, Größen/Stamp-Bereiche, Dateistruktur, `tenant_tag`/Packgruppe, `IdemToken`.

### 4.3 Konventionen

Little-endian; `varint` = kanonisches LEB128; Dateikopf `magic[8] ‖ file_kind u8 ‖ format_ver u16 ‖ feature_bits u32 ‖ file_id u64 ‖ instance_id[16] ‖ header_crc u32`; Längen vor dem Allokieren prüfen; reservierte Felder 0 und geprüft; eine Version je Dateiart; unbekannte Version/Pflicht-Bits ⇒ `Unsupported` (`INV-FMT-1`). Feature-Bit `BRANCHES`: ungesetzt ⇒ `branch ≠ 0` ist Formatfehler. v1: `COMPRESSION=0`, `filter_kind=Bloom(1)`.

### 4.4 Dateien

```
<root>/FORMAT  LOCK  CURRENT  CATALOG-<n>  KEYS  RESERVE
       redo/redo-<n>.log  spaces/<id>/<s>/<file_id>.run|.blob  (s = file_id mod file_shard_count, 3 Hex)  segments/<kind>/<id>.seg
<ledger_dir>/ERASE-LEDGER/…   (Standard <root>/ERASE-LEDGER; Produktion: getrenntes Volume)
```

`FORMAT` unveränderlich (+ RootKey-abgeleiteter MAC); `KEYS` = RootKey (KEK-umhüllt) + Space-DEKs (RootKey-umhüllt); **Clusterschlüssel nie hier**. Alle Dateien `Perm::Private`. Temporär `.building.<tmp_id>`. Verwaiste Dateien werden nach Recovery entfernt (`INV-CAT-1`). Kein Recovery-/Restore-Pfad vergibt IDs aus alten Catalog-Ständen erneut.

### 4.5 Redo

Vorallokierte Segmente (64 MiB, `redo_align` 4096), am Ende gesiegelt; Ketten-MAC `H_i = MAC(H_{i-1} ‖ head ‖ payload_tags)`.

```
Frame := head ‖ body ‖ tail
head (Klartext, kettengeschützt): len u32 | stamp u64 | kind u8 | n_ops u16 | idem_token[16] | epoch u64 | flags u8 |
      ops_head[]: { space_id u64, cluster[16], branch u32, op_kind u8, key_epoch u32, op_uid[16], payload_len u32 }
kind: Change=1 | Gate=2 | Control=3 | Reserve=4
Gate-Body (Klartext):    scope(Clusters|Space) | ticket[16] | level | policy_epoch u64 | n u16 | cluster[16]×n  oder space_id u64
Control-Body (Klartext): op(Restrict|Lift|Link|BranchFork|BranchDrop|BranchMerge) | … Branch-Nutzdaten versiegelt;
                         Link: derived[16] | n u8 | sources[16]×n | policy_epoch_after u64
tail: crc32c | chain_mac
```

Ein Change darf Ops mehrerer Cluster enthalten (`ops_head.cluster`); er ist atomar über alle. Redo-Integritätsschlüssel `KDF(RootKey, "ctxs redo-auth v1")`. Sealed-Nutzlasten sind mit `Clusterschlüssel(cluster, key_epoch)` versiegelt; Plain-Nutzlasten folgen `integrity`/Space-DEK. Recovery: Gate-/Control-Scan bestimmt `retired_set` ohne Entsiegeln; aktive Sealed-Ops öffnen nur mit ihrer `key_epoch`. Provider nicht erreichbar ⇒ `KeyUnavailable`, Recovery bleibt `LockedKeys`; bestätigter Schlüsselverlust ohne Löschbeleg ⇒ `KeyLost/Corrupt`, Quarantäne; nie still überspringen, nie als `Shredded` deuten. Torn-Tail (`INV-LOG-8`): erster ungültiger Gruppenkopf beendet das Log nur, wenn danach nichts Gültiges folgt, sonst `Corrupt{RedoMidLog}`; alles bis zur letzten `redo_mark` (alle 8 MiB und bei Siegel) muss gültig sein. Kein Segmentschnitt beim Gate; Redo-Segmente werden über die Redo-Barriere (§8.6) kontrolliert abgebaut.

### 4.6 Run

```
Run := header · ClusterSection* · ClusterDirectory · PackMeta · ExtensionBlock* · Footer
ClusterSection := FullSection | MiniSection
FullSection    := DataBlock* · SubIndexBlock · BranchTableBlock · FilterBlock · RangeDelBlock · ChangeIndexBlock
MiniSection    := SealFrame{ entries · range_dels · change_idx · branch_tab }
```

`ClusterDirectory` (Klartext, nach `ClusterId` sortiert, 4-KiB-Seiten, ≈ 78 Einträge je Seite): `{cluster, section_kind, section_off, section_len, n_blocks, n_entries, min/max_stamp}`; keine Gewichte oder Tokenprofile. Im RAM nur Sparse-Index (≈ 20 B je Seite) und `ClusterFilter` (Bloom über `ClusterId`, ≈ 8 Bit/Cluster, in `PackMeta`). `DataBlock` Ziel 16 KiB, Präfixkompression, versiegelt; Einträge tragen `branch`. `SubIndexBlock` je Block: letzter Schlüssel, Offset, Länge, `entries`, `weight_sum[W]`, `min/max_stamp`, `block_uid`; bei Sealed vollständig unter Clusterschlüssel (Page darf ihn entsiegeln, ohne Body zu lesen). Kumulative Präfixsummen werden beim Laden im RAM berechnet und nie persistiert. `FilterBlock` = Bloom über `(branch, user_key)`, cluster-gesalzen. `BranchTableBlock` (cluster-sealed): je Branch `{branch, first_block, last_block, n_entries, min/max_stamp, has_range_del}`; Einträge eines Branches bilden einen zusammenhängenden Blockbereich. Der Kern persistiert keine Zeitlanes oder Aggregate im SubIndex.

### 4.7 Blob

`Blob := header ‖ ClusterSection*(Record*) ‖ ClusterDirectory ‖ footer`, `Record := record_uid[16] | len u32 | SealFrame | crc32c`. AAD bindet `record_uid`, nicht `blob_id`/Offset ⇒ Rohkopie ohne Entsiegeln. Ziel ≤ 64 MiB. Kleine Löcher dürfen bei verfügbarem Hole-Punch dieselbe `blob_id` behalten; überschreitet Fragment-/Hole-Ratio den Grenzwert oder fehlt Hole-Punch, wird in eine neue `blob_id` mit gepackten Überlebenden geschrieben (`blob_replace`, atomar). Purge DARF NICHT auf `punch_hole` angewiesen sein. Eine Blob-Inkarnation wird nur entfernt, wenn kein lebender Run mehr einen `PutRef` darauf enthält (`INV-BLOB-1`).

### 4.8 Catalog

Snapshot-Record + Edit-Records (Kette, CRC) in vorallokierten 1-MiB-Blöcken. Gruppen-Commit: wartende Edits mit einem `write` + einem `sync_data`, Entlassung erst danach (`INV-CAT-2`); gekoppelte Zustandsänderungen sind **ein** Edit. Rollover bei `max(8 MiB, Größe des letzten Snapshots)`; `CURRENT` per atomarem `rename`.

Ops: `space_add | space_state | space_remove | cluster_register | cluster_state | cluster_trim | cluster_link | restriction_set | restriction_lift | pack_assign | files_add(meta.floor) | files_remove | unlink_ack | blob_replace | blob_live_delta | replay_floor | retain_floor | redo_mark | segment_add | segment_remove | hold_set | consumer_ack | erase_pending_add | erase_stage | erase_done | erased_ranges_add | erased_ranges_trim | key_epoch | kek_id | idem_trim | reserve_stamps | reserve_ids | reserve_space_ids | anchor_mark`. Reserviert `0xF000…0xFFFF`; unbekannte Tags ⇒ `Unsupported`. `files_remove` legt im selben Edit den Graveyard-Eintrag an; `unlink_ack` entfernt ihn nach `unlink + sync_dir`. Der Catalog enthält keine Branch-Namen und keine Nutzdaten.

Reservierungen: `reserve_stamps` 2^20, `reserve_ids` 2^16, `reserve_space_ids` 2^16 (Wartung füllt bei < 50 %). Erschöpft ⇒ normale Changes `Backpressure(Retry)` ohne Rang 3. Gate-/Control-/Link-Frames ziehen aus dem Notpool (`reserve_stamps_urgent` 2^10, `reserve_ids_urgent` 2^8), den nur die Lane `urgent` füllt. Rang 1 und 3 werden nie gleichzeitig gehalten.

### 4.9 Freeze-Liste (Gate G-F) und Systemspaces

Eingefroren: Dateikopf, `FORMAT`, `KEYS`, Redo-Frame (inkl. `space_id`, `branch`, `kind`, `epoch`, `op_uid`, Gate-Body inkl. `policy_epoch`, Control inkl. `Link`), Kette, ClusterSection (Full/Mini, BranchTable, Filter), ClusterDirectory, PackMeta, SubIndex, Siegelrahmen, Pfad-Sharding, Blob-Record, Catalog-Op-Tags, `AnchorValue`, Ledger-Format (Domain-Separation, Byteordnung der Tomb-HMACs, `tomb_epoch u32` und Tomb-Epochen-Events), `cmp`, Tag-Kodierung, **Partitionsformat der Indizes und `PartitionGeneration`-Schlüsselscope**, `head`-Record, `SYS_IDEM`-Format. Je Format ein unabhängiger Referenzcodierer (Python) und goldene Vektoren (Anhang C). Nach G-F ist jede Änderung eine Migration.

| System-Space | SpaceId | Zweck |
|---|---:|---|
| `SYS_BRANCH` | 1 | clusterlokale Branch-Metadaten |
| `SYS_IDEM` | 2 | instanzweite Idempotenz-Ergebnisse |
| reserviert | 0, 3…255 | versionierte Kernspaces; Benutzer-IDs ab 256 |

Öffentliche `spaces()`/`apply()`/Drop-Pfade weisen jede ID < 256 mit `ReservedSpace` ab; nur der Systemspace-Owner greift intern zu. Änderungen an Systemspaces nur per versionierter Migration mit Recovery-Vertrag.

---

## 5. Schreibpfad

### 5.1 `apply` (Commit-Pipeline)

| Stufe | Sperre | Aufgabe | Verboten |
|---|---|---|---|
| Sequencer | Rang 1 | Zustandsprüfung, Guard-Delta, Epochenvergleich, Stamp-Vergabe, Übergabe an den Syncer; rein RAM, O(Ops) | E/A, Provider-/Cipher-/KDF-Aufruf, Warten, Graph-Traversierung (Ausnahme: Selbstberechnung der Hülle nach Epochen-Retry, ≤ `max_gate_clusters` Knoten, nur RAM) |
| Syncer | keine | CRC + Ketten-MAC, ein `write` + ein `sync_data`, Publikation in Stamp-Reihenfolge | Stamps vergeben, validieren |

```
1  Vorprüfung ohne Sperre: Limits, Space/Cluster bekannt, INV-SEAL-1, Clusterzustand, DiskFull, Idempotenz-Lookup (Treffer ⇒ AlreadyApplied)
2  Vorbereiten auf dem Aufruferthread (Plattenzugriff und Providerwartezeit erlaubt):
   a) View V1 pinnen, Guards exakt gegen V1 auswerten (Stufe A), Punkt-Fingerabdrücke fp berechnen
   b) bei Gate/Link/Restrict/Lift/Space-Drop: Hülle H, Restrictions, Space-Restriction-Treffer berechnen; policy_epoch P1 notieren
   c) Ops kodieren, op_uid ziehen, Nutzlast versiegeln, Frame vorkodieren (Stamp 0, Kette/CRC leer)
   d) implizite Absent-Guard für den SYS_IDEM-Schlüssel anfügen (§5.5)
3  Stamp-Reserve prüfen (erschöpft ⇒ Backpressure(Retry); Gate/Control/Link: Notpool)
4  Pipeline-Credit holen (commit_pipeline_depth 2; urgent eigener Credit). Warten nur hier
5  Einreihen: Intake anhängen (Rang 2, kurz), Rang 2 frei; try_lock Rang 1: Erfolg ⇒ Leader (Rang 2 nur zum Leeren), sonst auf Ergebnis warten.
   Leader prüft nach Freigabe von Rang 1 die Intake erneut (kein verlorener Wakeup)
6  Sequencer (Rang 1), je Change in Intake-Reihenfolge:
     a) Zustand: Retiring/Retired/Shredded/Purged ⇒ ClusterRetired; BLOCK_WRITE ⇒ Restricted; Branch nicht Live ⇒ BranchClosed
     b) Guards Stufe B: Delta (V1, head] im Konfliktfenster; Treffer oder V1 < Fensterfuß ⇒ GuardRetry
     c) Gate/Link/Restrict/Lift: policy_epoch == P1, sonst PolicyRetry; nach policy_retry_max (3) berechnet der Sequencer die Hülle einmalig selbst
     d) Stamps vergeben; Branch-Ops ausführen; bei Gate: Live → Retiring(S)
     e) Konfliktfenster um (stamp, fp, ck) fortschreiben
     f) Gruppe in die Syncer-Warteschlange legen (nie blockierend); Rang 1 frei
7  Syncer: Frames finalisieren, ein write + ein sync_data                                  ◄── DURABILITÄTSPUNKT
8  Publikation (Syncer, strikt in Stamp-Reihenfolge, nur nach 7 derselben Gruppe): Memtable einfügen, Fenstereinträge durabel markieren,
   Retiring → Retired + gen++, Version veröffentlichen (Rang 4), visible_stamp/durable_stamp anheben (Release), Aufrufer wecken, Credit frei
9  Receipt { instance_id, stamp, durable }
```

Eigenschaften:
1. Durchsatz = Gruppengröße / Sync-Latenz; Gruppen wachsen mit der Last bis `max_group_changes` 1024 / `max_group_bytes` 8 MiB. Tiefe > 2 nur, wenn die Fs-Konformanz überlappende `sync_data`-Aufrufe als beschleunigend belegt.
2. Sync-Fehler ⇒ Poison; die fehlgeschlagene und alle sequenzierten späteren Gruppen sind `Indeterminate{first,last}`/`Poisoned`; nichts davon wurde veröffentlicht (`INV-PIPE-2`).
3. Leser sehen `Retiring` als `Live`, bis das Gate durabel und publiziert ist; die Schreibadmission ist dagegen sofort geschlossen.
4. Memtable-Einfügung durch den Syncer ist Standard; parallele Einfügung durch Aufrufer ist zulässig, wenn der Syncer nach `publish_timeout` selbst einfügt.
5. `Relaxed`-Changes reiten mit und werden nach dem `write` (ohne Sync) veröffentlicht; nie für Löschung, nie mit `IdemToken` (FIX-27).

Sichtbarkeit nur nach Durabilität. Vergleiche über Instanzen führen `(instance_id, stamp)`. Rückdruck stufenweise außerhalb jeder Sperre.

### 5.2 Memtable und Stamps

Lock-freie Skiplist hinter Trait `Memtable`, Klartext (AN-05); 64 MiB je Space, ≤ 3 immutable, 256 MiB gesamt. Nach Absturz: `next_stamp = max(stamp_limit, letzter gültiger Redo-Stamp, max Run-Stamp) + 1`. Consumer und `view_durable` sehen nur Stamps ≤ `durable_stamp` derselben Instanz (`INV-CONS-1`).

### 5.3 Guards (zweistufig, optimistisch)

**Stufe A (Aufrufer, ohne Sperre):** Pin `V1`, exakte Auswertung jeder Guard gegen `V1` (`Absent | Version(Stamp) | Unchanged{since}`; Plattenzugriff, ChangeIndex-Scan, Merge-Konfliktprüfung). Konflikt ⇒ `Err(Conflict)`. Beweise: Punkt `(V1, fp)` mit `fp = SipHash-1-3(prozessschlüssel, space‖cluster‖branch‖key)`; Range `(V1, ck, [start,end))` mit `ck = (space, cluster, branch)`.

**Stufe B (Sequencer, nur RAM, O(Guards)):** Konfliktfenster = stamp-sortierter Ring der letzten `guard_window_entries` (≤ 1 Mio.) Mutationen `(stamp, fp, ck)`, auch sequenzierter, noch nicht durabler Gruppen; nur im Speicher. Punkt-Guard: Eintrag mit gleichem `fp` und `stamp > V1` ⇒ `GuardRetry`. Range-Guard: jeder Eintrag mit gleichem `ck` und `stamp > V1` ⇒ `GuardRetry` (kein Schlüsselvergleich unter Rang 1). `V1 < Fensterfuß` ⇒ `GuardRetry`. Der Sequencer urteilt nie über Anwendungskonflikte; es gibt keinen Langsampfad. Der Aufrufer wiederholt Stufe A höchstens `guard_retry_max` (3) Mal, danach `Conflict{contended: true}`.

`INV-GUARD-1`: Ein Change wird nur angenommen, wenn jede Guard zur Stamp-Vergabe gilt.

Guard-Katalog (Konvention für obere Schichten): Event anhängen → keine Guard; Zustand/Fakt ändern → `Version`; Tool-Zustand → `IdemToken` + `Version`; Branch-Merge → `Unchanged{since: merge_base}` je Schlüssel.

### 5.4 Flush

Je Space: Memtable einfrieren → je Cluster ein Abschnitt (**Retired-Cluster übersprungen**) → Staging- und Pure-Runs (`.building` → `sync_all` → `rename` → `sync_dir`) → **ein** Edit (`files_add`, `replay_floor`, `anchor_mark`) → neue Version. `replay_floor` rückt nur über Segmente, deren Frames in durablen Runs stehen **und** deren Gate-Frames als `erase_pending` im Catalog stehen (`INV-LOG-5`). Flush braucht Clusterschlüssel (gepinnt für die Jobdauer); Providerausfall ⇒ Pause mit Backoff.

### 5.5 Idempotenz

`IdemToken` im Frame-Kopf; der Aufrufer legt im **selben Frame** einen `SYS_IDEM`-Record an und setzt eine implizite `Absent`-Guard auf dessen HMAC-Schlüssel. Zwei gleichzeitige Requests mit demselben Token: der zweite erhält `GuardRetry`, wiederholt Stufe A, findet das Ergebnis und liefert `AlreadyApplied`. Kein Token-Index im Sequencer, kein RAM-Wachstum ∝ aktive Tokens.

`SYS_IDEM` ist instanzweit unter dediziertem System-DEK; Record-Schlüssel und Request-Fingerprint sind keyed HMACs. Gespeichert: Token-Scope/Hash, kanonischer Request-Fingerprint, Status, Ablaufbucket, pseudonymes Cluster-Scope-Digest aus sortierten `tomb_id`s. Payload wird vor Rang 1 versiegelt; Ergebnis-Stamp ist der äußere MVCC-Stamp. Aktive Records überleben `replay_floor` (geflusht/kompaktiert, bis TTL gepinnt); `idem_trim` ist eine durable Trim-Wasserstandsmarke ohne Fsync je Commit. Limits für aktive Tokens und Bytes; Überschreitung ⇒ `IdemCapacity` vor Mutation; aktive Tokens werden nie verdrängt. SYS_IDEM-Schlüssel nicht verfügbar ⇒ tokenierte Operationen fail-closed. Gleicher Token mit anderem Request ⇒ `IdemConflict`; nach Cluster-Retire ⇒ `Erased`/`ClusterRetired`, nie neues Apply. Ein tokenierter Change MUSS `Durable` sein.

### 5.6 Notbetrieb bei vollem Datenträger

Redo, Catalog, Ledger schreiben nur in vorallokierte Bereiche (AN-10); `ENOSPC` tritt nur beim Vorallokieren auf, ohne Teilschreibung. `DiskFull` verweigert User-Commits; Reads, Gate und KeyDestroy nutzen getrennte Emergency-Reserven (`erase_reserve_bytes` ≥ 2 GiB). Purge hat einen geschützten Scratch-Pool von mindestens `2 × max(run_target_bytes, blob_max_bytes, redo_segment_bytes)` (128 MiB) für einen aktiven Rewrite plus Recovery-Slot; normale Flush/Compaction pausieren, sofern sie ihn nicht nutzen dürfen. Der Pool wird bei Open geprüft. Purge rewritet Dateien einzeln (`PurgeBatchEdit`); vollständig tote Dateien werden direkt entfernt; freier Quellplatz geht nach Reader-Reclamation in den Pool zurück. Fs-Adapter dürfen die Garantie nur deklarieren, wenn Vorallokation auch unter CoW/Quota Platz reserviert; sonst verweigert Open den harten DiskFull-Erase-Modus oder meldet das Restrisiko. Gate-/Control-/Ledger-Frames passen ins Emergency-Reservesegment (`INV-LOG-7`). Ausstieg mit Hysterese. Nur `EIO`/Fsync-Fehler vergiften (`INV-DISK-1`).

---

## 6. Lesepfad

1. `view()` lädt `visible_stamp` (Acquire), dann `Arc<Version>` (`INV-VIEW-2`). Keine Sperre über E/A.
2. **Gate zuerst:** Clusterzustand und Restrictions vor jeder Entsiegelung (O(1)). Unbekannt ⇒ `UnknownCluster`; `Retired` oder weiter ⇒ `Erased{at:S}` **für jede Sicht, auch ≥ S** (Erasure-Barriere überstimmt Snapshot-Isolation); `BLOCK_READ` ⇒ `Restricted`; widerrufene View ⇒ `Revoked`.
3. **Punktabfrage:** Memtable → immutable → Staging-Runs (nur `ClusterFilter`-Treffer) → je Level die Dateien der Wartungsgruppe (Binärsuche, `ClusterFilter`) → ClusterDirectory (Sparse-Index, höchstens eine Seite) → Abschnitt → BranchTable → Filter → SubIndex → **ein** Datenblock je Treffer-Level. Auf einem Branch als Chain-Resolve (§6.2).
4. **Cursor/Page:** k-Wege-Merge über Quellen eines Clusters und seiner Branch-Kette; Kopf ohne Body-Block-E/A; `page` entsiegelt SubIndex-Metadaten mit `weight_sum`, überspringt Body-Blöcke ohne Entsiegeln und liefert `ResumeToken`. RAM-Präfixsummen sind Obergrenzen, nie Korrektheitsquelle. Ein Eintrag, der das Budget sprengt, beendet die Seite, außer die Seite wäre leer (Fortschrittsgarantie). Cursor prüfen Gate, Widerruf und Lease zwischen Blöcken. `ResumeToken` ist AEAD-versiegelt unter einem Prozessschlüssel (Stamp, Cluster, Branch, letzter Schlüssel); nach Löschung `Err(Erased)`.
5. **Körper** erst bei `body()`; Gate ⇒ `Erased`; Provider offline ⇒ `KeyUnavailable`; Schlüssel fehlt ohne Löschnachweis ⇒ `KeyLost`. Nie Daten, nie „leer“.
6. **Block-Cache** hält entsiegelte Blöcke mit `(cluster, gen)`; Treffer nur bei passender `gen`; Gate und ER2 erhöhen `gen`; `urgent` überschreibt Puffer (`Zeroizing`). **Check-after-decrypt:** `gen` vor **und** nach dem Entsiegeln (`INV-KEY-2`).
7. `view_at(S)` mit `S < history_floor` ⇒ `BelowFloor`; Sicht unter `S_erase` in einem Bereich mit `erased_ranges` ⇒ `Erased` (`INV-ERA-5`).
8. **`View::filter_visible(&[RecordRef])`:** prüft Clusterzustand, Restriction und Stamp (Prüfart `DocHead` siehe §11.4). Jedes Extension-Ergebnis MUSS ihn passieren (`INV-VIS-1`).
9. Keine clusterübergreifenden Scans im Kern; `Inspector::clusters()` iteriert.

### 6.1 Chain-Resolve und Branch-Kosten

1. **Layout:** `cmp` sortiert `branch` vor `user_key`; ein Branch bildet je Abschnitt einen zusammenhängenden Blockbereich (`BranchTableBlock`).
2. **Filter:** Bloom über `(branch, user_key)`; je Kettenebene eine RAM-Probe.
3. **Punktlesen:** Kette `[b0=Blatt … bd=Wurzel]`, Stamp-Kappen `cap_0 = V`, `cap_{i+1} = min(cap_i, fork_stamp(b_i))`. Je Ebene und Datei: (a) BranchTable negativ ⇒ weiter; (b) bei `has_range_del` die RangeDel-Intervallliste prüfen (O(log r)), auch bei negativem Filter; (c) Filter negativ ⇒ weiter; (d) Blocklesung nur im Blockbereich des Branches. Erster sichtbarer Treffer ≤ `cap_i` beendet; ein überdeckendes `Delete`/`RangeDelete` beendet mit „nicht vorhanden“ ohne Elternlesung.
4. **Kosten:** Datenblock-Lesungen ≈ `1 + d × FPR` plus Treffer im Branch (H-21).
5. **Cursor:** Merge-Fan-in = Σ je Kettenebene der Dateien mit Treffern im Bereich; `RangeDelete`-Fragmente und Kappen im Iterator.
6. **Korrektheit:** Filter und BranchTable liefern nie ein falsches Negativ (`INV-BR-5`); Property-Test gegen Referenzmodell inkl. `RangeDelete` auf Zwischenebenen, `Fork At(S)` und Compaction mit Fork-Floor.
7. Verfehlt H-21 bei Tiefe 8 das Ziel, sinkt die harte Obergrenze; das Messverfahren wird nicht gelockert. Spike H-21/H-11 ist Voraussetzung des Formatfreezes.

---

## 7. Versionierung (Branches, Diff, Merge)

Branches sind clusterlokale O(1)-Forks (Control-Frame, keine Datenkopie) und sterben mit dem Cluster. Ein instanzweiter Commit-DAG mit Inhaltsadressierung ist ausgeschlossen: er würde Löschdomänen über Hashes verbinden. Tamper-Evidenz liefern Ketten-MAC, Ledger, Anker.

| Konzept | API |
|---|---|
| Commit | durabler `Change`; `Receipt.stamp` |
| Zeitreise | `view_at(stamp)`, `View::on_branch(b)` |
| Branch | `fork(cluster, parent, ForkPoint::{Head\|At(S)}, BranchOptions{name, ttl})` |
| Diff | `diff(view_a, view_b, cluster, range)`; `branch_diff(branch)` |
| Merge | `merge(cluster, source, target, mode)` |
| Rollback | `branch_drop` (nur leaf) |

**Metadaten:** `SYS_BRANCH` (SpaceId 1, clustered, Datenklasse des Clusters): `branch_id → { name ≤ 64 B, parent, fork_point, state: Live|Merged|Dropped, ttl, created_by }`. Bei `Head` ist der MVCC-Stamp des Erstellungsrecords der `fork_stamp`; kein stampabhängiges Feld im versiegelten Payload. Der Catalog bleibt frei von Namen.

**Lesen:** Sicht `(V, b)`: neueste Version in `b` mit Stamp ≤ `V`, sonst rekursiv im Parent mit Kappe `min(V, fork_stamp(b))`; `Delete`/`RangeDelete` in `b` verdecken Elternversionen; der clusterweite `EraseRange`-Maskensatz gilt vor der Versionsauflösung und ist durch nichts aufhebbar.

**Fork/Drop:** Fork linearisiert im Sequencer; `BranchId (u32)` wird zuvor aus `reserve_ids` reserviert (Überlauf ⇒ `Exhausted`, `INV-BR-4`); SYS_BRANCH-Payload wird vor Rang 1 versiegelt; Sequencer validiert Reservierung, Parent-Head, Cluster-Generation, Branch-Zustand erneut. `branch_drop` nur für leaf-Branches (`BranchHasChildren`); TTL-Expiry bottom-up. Gemergte Sources sind terminal/read-only bis `branch_drop`. Für Sealed-Branches gibt es kein isoliertes Crypto-Erase. `gc_floor` = Minimum aller View-/Hold-/Consumer-Stamps und `fork_stamp`s lebender Branches (ohne Pins `durable_stamp + 1`); `history_floor` ≤ `gc_floor` und wird erst nach GC fortgeschrieben. `At(S)` verlangt `history_floor ≤ S ≤ parent_head`, sonst `BelowFloor`/`InvalidForkPoint` (nie stilles Rebasen).

**Diff:** `ChangeIndexBlock` (cluster-sealed, nach `(stamp, op_uid)`: `(stamp, branch, op_uid, kind, key/range)` inkl. Punkt-, `RangeDelete`-, `EraseRange`-, Branch-State-Mutationen); vor dem Flush aus Memtable/Redo-Overlay; Recovery rekonstruiert aus Redo. Kosten ∝ Mutationen seit gemeinsamem Floor + Ausgabe. Records bleiben gepinnt, bis `gc_floor` sie freigibt.

**Merge** (v1 nur direkter Child → Parent, `merge_base = fork_stamp(source)`; Source mit lebenden Kindern, geschlossene Cluster, veraltete Heads ⇒ Fehler ohne Mutation):
1. Berechnung: Source-Mutationen seit `fork_stamp` über den ChangeIndex; Konflikt bei überlappender Target-Änderung seit `merge_base`; gleiche Endzustände sind kein Konflikt; `Delete`, `RangeDelete`, überlappendes `EraseRange` kollidieren.
2. Ergebnis `Merged(Receipt)` oder `Conflicts(Vec<ConflictCell{key, base, ours, theirs}>)`; bei Konflikten wird nichts geschrieben.
3. Anwendung als ein normaler `Change` auf `target` mit Guards seit `merge_base`; der Frame bindet `source`, `source_head`, `target_head`, `merge_base`, revalidiert alle und setzt `Merged(at stamp)` im selben Frame. `EraseRange` ist nie ein wählbarer Zielwert.
4. Core-Modi `FailOnConflict`, `PreferSource`, `PreferTarget`; typisierte Strategien (LWW, Union, Textmerge, semantisch) liefert der Client. Merge trägt einen `IdemToken`.

**Zeitmodell:** Stamp = Transaktionszeit. Gültigkeits-, Beobachtungs-, Quellzeit leben im Kopf der oberen Schicht. „Was wusste das System bei Stamp S?“ = `view_at(S)` (nach Erase `Erased`, nie ersatzweise Daten). Zeitprädikate dürfen Blöcke nur über `BlockPruner` überspringen.

---

## 8. Löschung

### 8.1 Prinzip

Drei geordnete, unabhängige Schritte: **Gate** (logisch, Millisekunden) < **Schlüsselvernichtung** (kryptografisch) < **Purge** (physisch, fristgebunden). Der Kern sagt nie mehr, als die erreichte Stufe trägt.

### 8.2 Stufen (zwei Achsen)

Jeder Löschauftrag (Cluster, Range, Branch, Space) trägt eine zufällige `EraseRequestId` (unabhängig vom 24-h-`IdemToken`). Das Ledger persistiert nur `request_id_hmac` und `request_digest`; identischer Retry liefert dasselbe Ticket, abweichender Scope mit derselben ID `IdemConflict`.

| `EraseLevel` (Ziel) | Für | Verfahren | Schützt gegen | Schützt nicht gegen |
|---|---|---|---|---|
| `Logical` | beide | Gate | Lesen über die API | Inhalt auf Platte |
| `Physical` | Plain | Gate, Redo-Barriere, Purge, Edit, Unlink, `sync_dir` | Lesen aus lebenden Dateien, Wiederkehr nach Absturz (mit Anker) | Journal, Flash, Backups, Hardlinks |
| `Scrubbed` | Plain | wie Physical + Überschreiben | einfache Wiederherstellung klassischer Datenträger | CoW, SSD (best-effort) |
| `Cryptographic` | Sealed | Gate, alle Cluster-/Extension-Schlüssel des Scope vernichten, Purge/Verify | verschlüsselte Reste in Kern und registrierten Owners/Backups, soweit jede Kopie unter der zerstörten Key-Domäne liegt | gemeinsam geschlüsselte Kopien, Schlüsselkopien, unregistrierte Derivate |

| `Stage` | Bedeutung |
|---|---|
| `IntentRecorded` | Intent durabel (auch auf Poisoned-Handle) |
| `Gated` | Gate durabel, Cluster `Retired(S)` |
| `KeyDestroying{eta}` / `KeysDestroyed` | Schlüssel unbenutzbar / `Absent` |
| `Purging` / `Purged` | Bytes aus lebenden Dateien entfernt |
| `Done` | Ledger durabel, Anker fortgeschritten, Receipt |

**Zulässige Ziele (FIX-04):** Cluster-Ticket: `Sealed` ⇒ `Logical` oder `Cryptographic`; `Plain` ⇒ `Logical`, `Physical`, `Scrubbed`; alles andere `Unsupported`. `Logical` ist als Endziel zulässig (z. B. unter Restriction-Fristen); das Receipt lautet dann `CompleteWithResidual{LogicalOnly}`, das Ticket bleibt per `escalate` eskalierbar. `range`, `branch`, `space_drop`: Plain `Logical|Physical|Scrubbed`, Sealed nur `Logical`; `Cryptographic` auf Space-Ebene ist `Unsupported` (Crypto-Erasure erfordert Cluster-Tickets). `Cryptographic` bei `KeysInBackups::Yes` ⇒ `Unsupported`; dann bleibt für Sealed-Cluster nur `Logical`, und die Profile `Standard`/`Strict` verbieten diese Provider-Konfiguration für `Personal`. Das Receipt spricht nie von „unwiderruflich“ oder „konform“.

### 8.3 Garantie-Kernsätze

1. Ab durablem Gate liefert kein `view`/`page`/`poll` Inhalt des Clusters und kein Commit schreibt hinein; voller Snapshot oder `Erased`, nie Mischsicht (`INV-ERA-1`).
2. Nach `KeysDestroyed` ist alles des Clusters im Kern (Redo, Runs, Blobs, gespeicherte Checkpoints) ohne Clusterschlüssel unlesbar (`INV-ERA-7`). Extension-/Backup-Kopien sind nur erfasst, wenn ihr registrierter Key-Scope an dieselbe Vernichtung gebunden ist; sonst bleibt das Gesamt-Receipt `Incomplete`.
3. **Definition „Bytes von H“ (FIX-02):** Nutzdaten, Ciphertext, Schlüsselmaterial und `ClusterDirectory`-/Block-/Record-Einträge von H in Runs, Blobs, Segmenten und Redo-Payload. **Nicht** dazu zählen Routing-Metadaten (ClusterId in Frame-Köpfen, Gate-/Control-Frames, Ledger-`tomb_id`); sie sind pseudonyme Metadaten (§8.11) und verschwinden mit dem Redo-Segment.
4. Nach `Purged` enthält keine lebende Datei Bytes von H (`INV-ERA-2`).
5. Gate/KeyDestroy skaliert mit der Zielmenge (zuzüglich Providerlatenz). Purge bearbeitet nur Dateien/Extents mit Zielcluster; Kosten ≤ Summe betroffener, je `max_file_bytes` begrenzter Dateien plus gepinnte Redo-/Extension-Bereiche (`INV-ERA-11`). `ClusterLocal` vermeidet fremde Survivor-Rewrites in Runs/Blobs, nicht im gemeinsamen Redo. Globale Rebuildable-Indizes haben eigene Fristen und Kosten.
6. **Korpus-Löschung ist kein Erase:** Das Löschen eines `NonPersonal`-Dokuments aus dem Korpus ist ein normales `Delete` plus Index-Tombstone (logische Unsichtbarkeit, Entfernung per Compaction); es erzeugt kein Receipt. Nachweispflichtige Löschung braucht eine eigene Löschdomäne (Cluster).

### 8.4 Schlüssel und `KeyProvider`

```
KEK (KeyProvider, extern) └─ RootKey (32 B, in KEYS KEK-umhüllt)
      ├─ Space-DEK (je Space, RootKey-umhüllt)
      └─ Redo-/Catalog-/FORMAT-/Ledger-Schlüssel, LedgerScopeKey = KDF(RootKey, Label)
Clusterschlüssel: KeyProvider::cluster_keys(cluster, epoch)         (NICHT in KEYS)
```

```rust
pub trait KeyProvider: Send + Sync {
    fn info(&self) -> KeyProviderInfo;
    fn kek(&self, id: KekId) -> Result<Key32, KeyError>;
    fn create_cluster_keys(&self, request_id: BatchId, cs: &[ClusterId], e: &dyn Entropy) -> Result<Vec<KeyCreateResult>, KeyError>; // ≤256, idempotent, durabel vor Rückgabe
    fn cluster_keys(&self, req: &[(ClusterId, u32)]) -> Result<Vec<KeyLookup>, KeyError>;       // ≤256; Missing ≠ Shredded
    fn destroy_many(&self, cs: &[ClusterId]) -> Result<DestroyReceipt, KeyError>;                // ≤256 je Aufruf, ALLE Epochen, idempotent
    fn probe(&self, c: ClusterId, epoch: u32) -> Result<KeyProbe, KeyError>;                     // Present | Disabled | Absent
    fn list(&self) -> Result<Vec<KeyListEntry>, KeyError>;                                       // { cluster, created_utc }
    fn create_tomb_epoch(&self, epoch: u32) -> Result<(), KeyError>;                             // idempotent, durabel
    fn tomb_key(&self, epoch: u32) -> Result<Option<Key32>, KeyError>;                           // None ⇒ retired
    fn retire_tomb_epoch(&self, epoch: u32) -> Result<KeyProbe, KeyError>;
    fn create_partition_key(&self, p: PartitionTag, generation: u32) -> Result<(), KeyError>;
    fn partition_key(&self, p: PartitionTag, generation: u32) -> Result<Option<Key32>, KeyError>; // None ⇒ vernichtet
    fn destroy_partition_key(&self, p: PartitionTag, generation: u32) -> Result<KeyProbe, KeyError>;
}
pub struct KeyProviderInfo { pub destroy: DestroyMode, pub keys_in_backups: KeysInBackups, pub assurance: KeyAssurance }
pub enum DestroyMode { Immediate, Scheduled { max_delay: Duration } }
pub enum KeysInBackups { No, Bounded { until: Duration }, Yes }
pub enum KeyAssurance { Hardware, Kms, FilePassphrase }
```

- **Schlüsselzustände** sind getrennt: `KeyUnavailable` (Retry, blockiert), `Disabled` (unbenutzbar, nicht unumkehrbar: `KeyDestroying`/`Blocked`), `Absent` (erfüllt Crypto-Erasure), `KeyLost` (`Incomplete`/Quarantäne). Nichterreichbarkeit ist nie Vernichtung. ER2 gilt erst bei `Absent` für jede Epoche `e ≤ key_epoch` (`INV-KEY-4`). Bei `Cryptographic` beginnt der Purge erst bei `Absent` (`INV-ERA-3`); bis dahin nur Planung und Scratch-Reservierung.
- **Frist-Konsistenz (FIX-17):** Bei `Scheduled{max_delay}` und `max_delay + purge_window_fraction × purge_deadline > purge_deadline` weist der Auftrag mit `Err(DeadlineBeforeKeyDestroy)` ab; Receipts weisen `t_crypto` und `t_physical` getrennt aus.
- **`KeysInBackups`:** `No` = kein Export außerhalb des Providers; `Bounded{until}` = Exporte werden spätestens `until` nach Vernichtung selbst vernichtet (Statuscap `BoundedBackupKeys{until}`, entfällt bei neu berechnetem Receipt nach Ablauf, solange der Provider `Absent` bestätigt); `Yes` ⇒ kein `Cryptographic`.
- **`KeyAssurance`:** `Hardware` (nicht exportierbar, Vernichtungsnachweis), `Kms` (Destroy-API, Karenz ⇒ `Disabled`), `FilePassphrase` (Argon2id; Vernichtung auf Journal-/CoW-/Flash-Medien nicht beweisbar ⇒ Statuscap `WeakKeyAssurance`, `Complete` ausgeschlossen). Nicht erlaubt in `Standard`/`Strict` für `Personal` (`ProviderAssuranceInsufficient`), außer `acknowledged_weak_keys`.
- **Tomb-Epochen:** `tomb_key(epoch)` leitet ausschließlich `tomb_id`s ab, liegt im Provider, nie in `KEYS`/Checkpoints/Backups (`INV-KEY-5`). Default: Epoche `0`, keine Rotation, kein Retire. Fehlt der Schlüssel einer nicht retired Epoche ⇒ `KeyLost` der Tomb-Domäne (Restore quarantined, neue Intents `KeyUnavailable`); nie als Retire deuten.
- **Partitionsgenerationsschlüssel** (§11.5): `PartitionTag` ist ein zufälliger 16-B-Bezeichner (keine ClusterId); nie in `KEYS`/Backups; `Absent` der Vorgängergeneration ist Voraussetzung für `Done` des Index-Owners (`INV-PART-1`); fehlender Schlüssel einer nicht vernichteten Generation ⇒ `KeyLost` der Index-Domäne (Partition neu aufbauen, Ticket `Incomplete`).
- **KEK-Rotation:** nur RootKey neu umhüllen. **Epoch-Rotation** eines Clusterschlüssels: neue `key_epoch` für neue Writes; alte Epoche wird erst vernichtet, wenn nichts mehr mit ihr versiegelt ist (`INV-KEY-3`); `destroy_many` vernichtet immer alle Epochen.
- **Cohort-Strategie** (Provider-intern): Clusterschlüssel in Cohorts ≤ 1024 unter einer Cohort-KEK; `destroy_many` = Rewrap ohne die Gelöschten, dann KMS-Destroy der alten Generation; Voraussetzung `keys_in_backups = No`, Replikation in ≥ 2 Failure Domains.
- Schlüsselcache mit `gen`, TTL, `Zeroizing`; bei Providerausfall gecachte Schlüssel bis `key_grace` (10 min); Gate und ER2 invalidieren sofort.
- `KeyProviderConformance` (Idempotenz, alle Epochen, `probe` je Epoche, Crash, Tomb-Epochen, Neustart ohne Epochenverlust) ist Gate-Pflicht für jeden Provider.

### 8.5 Protokoll `Eraser::cluster` (kanonische Sequenz, FIX-01)

Die Löschreihenfolge ist ein einziges, lineares Protokoll. Das **logische Gate** ist der durable Redo-Gate-Frame (G2); alle Catalog-/Ledger-/Anker-Schritte folgen in fester Ordnung.

```
ER0  Vorprüfung: Cluster existiert; Hülle H = {c} ∪ transitiv DerivedFrom-gebunden; BLOCK_ERASE in H ⇒ Err(Restricted);
     Level passt zur Protection; Provider erreichbar und info() erlaubt Cryptographic; Frist-Konsistenz (§8.4); bereits Retired ⇒ vorhandenes Ticket
G1   Ledger-`EraseIntent` durabel (≤ 4096 tomb_ids). Ab hier verpflichtet der Intent zur Ausführung, auch wenn der Aufrufer kein Ticket erhielt;
     Retry derselben EraseRequestId findet das Ticket (FIX-25)
G2   Gate-Frame im Redo (Sequencer, Gruppen-Fsync); Hülle gegen policy_epoch P1 berechnet, Sequencer vergleicht nur P1,
     nach policy_retry_max (3) Selbstberechnung. Live → Retiring(S) → (nach Sync + Publikation) Retired(S), gen++     ◄── LOGISCHER PUNKT (G0)
     erase() liefert Ok(ticket); ab hier nie mehr ein Fehler
G3   Catalog-Edit{cluster_state(H→Retired,S), erase_pending_add} durabel (Voraussetzung dafür, dass replay_floor den Gate-Frame überschreitet)
     Asynchron (urgent): Memtable-Einträge freigeben/überschreiben, Cache-Puffer überschreiben
G4   Ledger `GateApplied`
G5   Anker-CAS (Required-Anker unerreichbar ⇒ Ticket Blocked; keine irreversible Stufe)
ER2  (Cryptographic) Ledger `KeyDestroyIntent` durabel → destroy_many(H) → nur bei probe == Absent: KeysDestroyed, Cache leeren, gen++,
     Edit{cluster_state → Shredded}                                                                              ◄── KRYPTO-PUNKT (G1)
ER3  Plan: Dateien über Packgruppen-Index/ClusterDirectory (Staging eingeschlossen), Claims, Segmente mit Überlappung
ER4  Redo-Barriere (§8.6)
ER5  Umbau: PurgeBatchEdit je Datei/Blob-Inkarnation; Scratch-Slot wieder frei; Stage bleibt Purging
ER6  Abschluss: nach Verify + Extension-Attesten ein Edit{cluster_state(H→Purged), erase_done, anchor_mark}      ◄── PHYSISCHER PUNKT (G2)
ER7  Ledger `Done`, Anker fortschreiben, ErasureReceipt; Graveyard-Dateien ohne Zielnutzdaten werden separat nach ihren Reader-Pins entfernt
```

**Reihenfolge ist Vertrag:** G1 < G2 < G3 < G4 < G5 < KeyDestroyIntent < `destroy_many` < jedes `PurgeBatchEdit` < Unlink+`sync_dir` < `UnlinkAck` < ER6/`Done`. Bei `Cryptographic` beginnt der Purge nie vor `probe == Absent`. Das Gate liegt im Sequencer, weil ein vor dem Gate geprüfter Change nach dem durablen Gate nicht sichtbar werden darf. Jede Stufe ist crash-idempotent (Roll-forward).

**Frist:** `purge_deadline` ist Pflichtparameter. Das Ticket speichert Deadline, monotone Dauer und `clock_floor`; ein UTC-Sprung außerhalb `max_clock_skew` friert Deadline-Berechnung ein (`ClockUncertain`, Alarm). Überfällige, zeitlich verifizierte Tickets erhalten `urgent`; echte Überschreitung ⇒ `erase_deadline_breaches_total` und `Blocked{reason=Deadline}`.

**Bündelung und Amortisierung:** Purge-Auslöser (das Erste, was eintritt): (a) Compaction/Blob-GC schreibt die Datei ohnehin um und lässt Retired-Cluster weg; (b) 70 % der Frist verbraucht; (c) ≥ `purge_batch_min` (16) Cluster in derselben Datei. Je Fenster `W = purge_window_fraction × purge_deadline` (0,3) wird eine Datei höchstens einmal umgeschrieben; Purge-Bytes je `W` ≤ Σ Größen der betroffenen Dateien. Dringlichkeit: Plain ⇒ Lane `urgent`; Sealed ⇒ Lane `compact`, Eskalation auf `flush` bei 70 %, `urgent` bei 90 % der Frist.

**Views:** `ViewPolicy::Wait(timeout)` (Unlink wartet; die logische Garantie gilt bereits) oder `Revoke`.

**Done-Bedingungen (`INV-ERA-8`):** (1) keine lebende Datei/Segment/Inkarnation trägt Bytes von H (§8.3 Nr. 3), (2) Memtable-Einträge freigegeben, (3) Ledger durabel, (4) Anker fortgeschritten, (5) alle Segmentbesitzer attestiert, (6) bei Cryptographic `probe == Absent`, (7) Redo-Barriere abgeschlossen.

### 8.6 Redo-Barriere und `purge`

**Redo-Barriere (FIX-02).** Redo-Segmente mit Payload von H werden nicht durch Segmentschnitt unter Rang 1 abgebaut, sondern durch einen Wartungsjob: Segment rotieren, Memtables flushen, `replay_floor` und `retain_floor` hinter den Schnitt, ältere Segmente in den Graveyard; Consumer mit Position davor erhalten `Lagged`. Eine Barriere bedient alle fälligen Tickets. **Plain** und Space-Drop: sofort bei ER4. **Sealed:** zeitversetzt und gebündelt, spätestens bei 70 % der Frist; der Ciphertext im Redo ist ab `KeysDestroyed` unlesbar (`INV-ERA-7`), `Done` verlangt dennoch die Entfernung (`INV-LOG-9`). Obergrenze der Dauer ≈ `redo_max_age` (1 h) plus Flushzeit.

```
purge(plan: Dateien mit Claims, D: Dropmenge):
  für jede Datei/Blob-Inkarnation F einzeln:
     keep = Cluster(F) − D
     keep = ∅  → PurgeBatchEdit{files_remove(F)}                          // legt Graveyard-Eintrag an
     sonst     → F' im vorallokierten Scratch-Slot roh kopieren/repacken; sync_all, rename, sync_dir;
                 PurgeBatchEdit{files_add(F'), files_remove(F)}
  nach Reader-Drain: unlink(F); sync_dir; unlink_ack(F); erst dann Scratch-Slot auffüllen
  nach allen Dateibatches: SegmentOwner attestieren; PurgeCompleteEdit setzt Purged/Done
```

Rohkopie prüft Frame-CRCs, nie das AEAD-Tag. Pro Einzelschritt ist Scratch ≤ eine maximale Datei; zwei Slots decken Rewrite plus Crash-Slot. Vollständig tote Dateien brauchen keinen Scratch. `PurgeCompleteEdit` ist unzulässig, solange das Ticket Graveyard-Dateien oder offene `unlink_ack`s besitzt; Recovery leitet nie aus einem Zwischenbatch `Done` ab. Gleicher Algorithmus für Packgruppen-Abwurf (`D` = alle Cluster der Gruppe ohne Restricted), Mandantenabgang (`D` = alle Cluster des `tenant_tag`) und Staging-Dateien.

### 8.7 Löschauftrag auf vergiftetem Handle

Ein `Poisoned` Handle nimmt keinen Gate-Frame mehr an; die Löschpflicht endet nicht. Der Kern nimmt die Absicht über das Ledger auf (eigener Handle, eigene Failure Domain, ein Schreiber):

1. **Aufnahme:** Request und Restrictions werden gegen den RAM-Zustand geprüft; die Fassade hängt einen `EraseIntent{origin: Poisoned}` an (Tomb-Schlüssel aus dem Cache, kein Providerzugriff) und liefert `Ok(Ticket{IntentRecorded})`, nicht `Gated`. Schlägt der Append fehl ⇒ `Err(Poisoned{intent: NotRecorded})`.
2. **Logische Wirkung sofort:** RAM-Menge `pending_erase`; Lesen/Views/`page`/`poll`/Feed liefern `Err(ErasePending)`; Cache-Puffer werden überschrieben. Das ist kein Gate (kein `Erased{at}`).
3. **Neustart:** OP8 (§9.1) überführt jeden `EraseIntent` ohne `GateApplied` idempotent in ein Gate, vor jeder Nutzer-Sicht.
4. **Keine Abkürzung:** kein KeyDestroy, Purge oder Anker auf dem vergifteten Handle (`INV-ERA-13`).
5. **Ehrliche Grenze:** Klartextreste im Prozessspeicher bis Prozessende; Observer meldet `poisoned_erase_pending`; das Receipt weist `t_requested` und die Wartezeit aus.

### 8.8 Restriction, Hold, Links

- **Restriction:** `{reason u16 ≠ 0, owner, effects (BLOCK_ERASE immer | BLOCK_WRITE | BLOCK_READ), expires}`. Ablauf lässt nur den Vermerk fallen, löst nie Löschung aus. Kein Override. `lift` nach `LiftPolicy`: `OwnerOrAdmin` (Standard) oder `FourEyes` (Port `Authority`, Principal ≠ Requester). `restrict`/`lift` laufen als Control-Frame im Sequencer, stampgeordnet mit dem Gate (`INV-ERA-10`). Durchsetzung dreischichtig: Aufruferthread + Sequencer-Revalidierung, Purge verweigert Umbau, `cluster_state`-Edit verweigert `Retired`. Ein Erase blockiert sofort mit `Err(Restricted)`. `BLOCK_READ` ⇒ `Err(Restricted)`; zweckgebundene Lesung nur mit nicht klonbarer `RestrictedReadCap` (Observer-Ereignis, nie in Cache/Feed).
- **Hold:** technisch, `ttl ≤ 24 h`; ein Erase überstimmt jeden Hold ohne Fehler.
- **Links:** `Clusters::link(derived, sources[≤ 8])` ist ein Control-Frame `Link`. Admission außerhalb Rang 1 gegen `policy_epoch` P1: (a) azyklisch, (b) `max_link_depth` 8, (c) für jeden Vorfahren-oder-selbst `a` jeder Quelle `|Hülle(a) ∪ Hülle(derived)| ≤ max_gate_clusters` (4096), (d) keine Quelle `Retiring`/`Retired`. Traversierung bricht bei 4097 Knoten ab ⇒ `EraseFanoutExceeded` ohne Mutation. Der Sequencer prüft nur `policy_epoch == P1` und „Quelle Live“, setzt die Kante und erhöht `policy_epoch`. Löschung irgendeiner Quelle zieht den Abgeleiteten im selben Gate-Frame mit (`INV-ERA-12`); ein Kind mit mehreren Eltern wird gelöscht, sobald eines gelöscht wird. Cross-Subject-`DerivedFrom` ist unzulässig (die Identity-/Policy-Schicht prüft vor dem Link-Commit); gemeinsame Ableitungen brauchen getrennte Löschdomänen oder nicht-kaskadierende `Reference`.

### 8.9 Ledger, Anker, Receipt

**Ledger:** append-only, vorallokiert, über `Mac` kettenverknüpft, ein Schreiber mit Gruppenbündelung. Record = kanonischer tagged union `{seq, prev_hash, clock_sample, event, mac}`. Erase-Events enthalten nur `tomb_id = HMAC(tomb_key(tomb_epoch), "ctxs cluster tomb v1" ‖ instance_id[16] ‖ ClusterId[16])` samt `tomb_epoch u32`, Ticket, Level, Stamp/Stage, Evidence-Digest; nie Klartext-ClusterId, SpaceId, Nutzdaten, Schlüssel, Person-ID (`INV-LED-1`). Event-Kinds: `EraseIntent`, `SpaceDropIntent`, `SpaceScopeFrozen`, `GateApplied`, `KeyDestroyIntent`, `KeysDestroyed`, `KeyLost`, `PurgeProgress`, `UnlinkProgress`, `ExtAck`, `ExtBlocked`, `PurgeVerified`, `Completed`, `RestrictionSet/Lifted`, `BranchEraseDone`, `SpaceDropped`, `RestoreReconciled`, `ReceiptIssued`, `TombEpochOpened/Closed/RetireIntent/Retired`. Ein `EraseIntent` ist ein atomar appendbarer Datensatz mit ≤ 4096 sortierten `tomb_id`s (≤ 256 KiB); selektive Range-/Branch-Selektoren sind `scope_ciphertext` unter `LedgerScopeKey` (AAD: Instance/Ticket/Digest). `SpaceDropIntent` trägt `space_tomb_id = HMAC(ledger_key, "ctxs space tomb v1" ‖ instance_id ‖ be_u64(SpaceId))` und Level; Count und Scope-Digest folgen nach dem Gate als `SpaceScopeFrozen` (Mitgliedschaft wird nach dem Gate aus der gepinnten Version ermittelt, nie unter Rang 1). Ein Gate ist erst zulässig, wenn sein Intent durabel ist.

**Schlüsselableitung (FIX-09):** `ledger_key` und `LedgerScopeKey` werden aus dem RootKey abgeleitet und nicht separat gespeichert; ein Restore benötigt KEK + `KEYS` aus der Kopie. Das Ledger selbst liegt nie in Checkpoints.

**Anker:** `AnchorValue = (erase_epoch, ledger_head, catalog_epoch)`. Fortschreibung nach G3/G4 und **vor** KeyDestroyIntent oder Purge-Barriere (blockierend); zusätzlich bei Catalog-Rollover, `close`, spätestens `anchor_max_age` (1 s) bei ausstehendem Zustand (koalesziert). `AnchorPolicy=Required` ist Standard bei Encrypted/Authenticated; ist der Anker nicht fortschreibbar, bleibt das Ticket `Blocked`. `Weak` erfordert `acknowledged_weak` und attestiert keinen starken Rollback-Schutz. Referenz: `MemAnchor`, `FileAnchor` (`Weak`).

**Ledger-Failure-Domain:** `ledger_dir` auf anderem Gerät als das Datenverzeichnis (Vergleich der `device_ids`); sonst `Err(LedgerSameDevice)`, außer `ledger_same_device_ack` (Restexposition im Receipt). Ledger beim schreibenden Öffnen nicht lesbar ⇒ `Err(LedgerUnavailable)`; `ReadOnly` nur mit Diagnose.

**Idempotenz:** `request_id_hmac = HMAC(ledger_key, "ctxs erase request id v1" ‖ instance_id ‖ EraseRequestId)`; `request_digest = HMAC(ledger_key, "ctxs erase request digest v1" ‖ canonical_erase_request)` (versioniertes, tag-/längencodiertes Format; Deadlines nicht semantisch).

**`ErasureReceipt`:** `{instance_id, req/ticket, scope_digest, level, t_requested, t_logical, erase_epoch S, t_crypto?, t_physical?, t_verified?, stages[{stage, ledger_seq, evidence_digest}], key_attestations, provider{destroy, keys_in_backups, assurance}, ext_acks, verifier_report_digest[MeasureKind je Messung], ledger_head, anchor{strength, value}, restrictions_encountered, status, residual, canonical_bytes}`.

**Status (FIX-03):** `Complete` | `CompleteWithResidual` | `Incomplete` | `Blocked` | `Failed`.
- `Complete` verlangt jede für den Level erforderliche Kern- und Owner-Attestation, alle Pflichtmessungen `Complete` und keinen Statuscap.
- **Statuscaps** (begrenzen auf `CompleteWithResidual`): `LogicalOnly`, `WeakKeyAssurance`, `BoundedBackupKeys{until}`, `WeakAnchor` (bestätigt), `LedgerSameDeviceAck`, `HostKnownExposed` (HostProbe meldet bekannt unsicher: unverschlüsselter Swap, aktive Core-Dumps), unregistrierte externe Persistenz (Receipt-Restrisiko).
- **Disclosures** (immer im Receipt aufgeführt, **kein** Cap): Prozessspeicher-Reste, Dateisystem-Journal/Flash/Snapshots (bei Sealed nur Ciphertext), externe Backups außerhalb der Verwaltung, obere Schichten (§8.11), `HostProbe=Unknown`/`Declared`. Ein Disclosure beschreibt, was der Level laut §8.2 nicht abdeckt; er macht ein Receipt nicht unerreichbar.
- `Incomplete`: erforderlicher Owner-/Scope-Nachweis fehlt oder eine Pflichtmessung ist `NotMeasured`/`NotImplemented` (`INV-VER-1`); nie eine erfolgreiche Quittung. `Blocked`: sicher pausiert, fortsetzbar. `Failed`: terminaler Integritäts-/Protokollfehler.
- Die Signatur liegt oben (`engine::compliance`, `crypto::deletion_proof`, Ed25519, optional, nie Voraussetzung für `Done`); der Kern liefert kanonische Bytes und die offline lauffähige Funktion `verify_receipt(receipt, ledger_slice, anchor_attestation) → Verdict` (Kette, Anker, Stufenreihenfolge, Zeitmonotonie; braucht keinen Tomb-Schlüssel).

**`Eraser::verify`** (strukturell, ohne Schlüssel, Pflichtmessungen): (a) keine lebende Datei/Segment/behaltenes Redo-Segment trägt Nutzdaten-/Directory-Einträge von H (Routing-Header und Gate-Frames sind ausgenommen, §8.3 Nr. 3), (b) Memtables ohne H, (c) bei Cryptographic `probe == Absent`, (d) Ledgerkette bis zum Anker gültig.

### 8.10 Weitere Löschpfade

- **`Eraser::range(space, cluster, range)`:** `EraseRange` ist ein clusterweiter Gate-Marker, maskiert sofort alle Branches und älteren Views. Plain: `Logical|Physical|Scrubbed` (Physical rewritet alle Branch-/Stamp-Versionen des Bereichs); Sealed: nur `Logical` (keine selektive Schlüsselvernichtung in v1; für physische/kryptografische Entfernung den ganzen Cluster löschen oder den Bereich vorab in eine eigene Löschdomäne legen). `erased_ranges` werden erst getrimmt, wenn `history_floor > erase_stamp` und keine Pins darunter liegen.
- **`Eraser::branch(cluster, branch)`:** nur terminale leaf-Branches; Plain `Logical|Physical|Scrubbed`, Sealed `Logical`; eine Crypto-Anforderung wird als Cluster-Erase ausgeführt.
- **`Storage::space_drop(space, request_id, level, deadline)`:** Prüfung der `BLOCK_ERASE`-Restrictions außerhalb Rang 1, Revalidierung per `policy_epoch`. `SpaceDropIntent` durabel → `Gate{Space(id)}` → `GateApplied` → Anker → bei Plain Purge. `SpaceDropped` ist der Abschluss nach `SpaceScopeFrozen` und allen Attesten. Wiederanlage braucht eine neue SpaceId. Kein Einzel-Edit für beliebig viele Dateien.
- **`pack_expired(PackKey)`:** `H` = nicht-Restricted-Cluster der Gruppe, deterministisch zerlegt in unabhängige Tickets ≤ 4096 Cluster (eigener Gate-Frame, eigene Ledger-/Receipt-Kette; keine atomare Gesamtlöschung). Restricted-Cluster erscheinen mit Grund im Aggregat und werden nach Lift erneut versucht.
- **`escalate(ticket, level)`**, **`open_tickets()`** (nach Absturz des Aufrufers), **`clusters_batch`**.
- **Mehrbetroffene Daten:** gemeinsamer Fallcluster; jede Person hat ihren Wurzelcluster; Link nur, wo die Löschung der Person den Fallcluster mitziehen soll; sonst Referenz. Keine mehrclustrigen Einträge.

### 8.11 Restexposition (Disclosure-Katalog, vom Orakel geprüft)

| Bereich | Restrisiko | Zuständig |
|---|---|---|
| Prozessspeicher, Swap, Core-Dateien | Klartextreste | `Zeroizing`, Cache-Drop beim Gate; `mlock`/`DONTDUMP` im Adapter; Betreiber: Swap verschlüsseln, Core-Dumps aus; `HostProbe` liefert den Ist-Zustand |
| FS-Journal, Flash-Wear-Leveling, Volume-Snapshots, externe Backups, Checkpoints mit `Retired` Plain-Daten | alte Blockversionen | Sealed: nur Ciphertext; Plain: Betreiber begrenzt Aufbewahrung |
| KMS-Karenz | Schlüssel bis Fristende wiederherstellbar | `KeyDestroying`/`Blocked` |
| Schlüsselkopien | Stufe G1 hinfällig | `KeysInBackups`-Cap |
| `FilePassphrase` | Vernichtung nicht beweisbar | `WeakKeyAssurance` |
| Metadaten | ClusterId in Redo-Gate-Frames bis Segmentabbau, Anzahl, Größen, Zeitpunkte, Packgruppe/`tenant_tag` | zufällige IDs; Ledger nur epochengebundene HMAC-`tomb_id` |
| Rollback über den Anker hinaus | Ticket rückgängig | Ankerstärke im Receipt |
| Ledger auf demselben Gerät | gemeinsame Failure Domain | `LedgerSameDevice` |
| Obere Schichten (Prompt-Caches, Embedding-Kopien, Logs, Fine-Tuning, Modellgewichte, Provider-Logs) | nicht im Kern lösbar | nur über Links/`SegmentOwner` angebundene Daten werden mitgelöscht |
| Pseudonymisierter Restkontext im Korpus | Re-Identifizierbarkeit | Einzelfallbewertung (§11.9) |
| Verzeichnis Person → Wurzelcluster | nur physisch löschbar | §11.13 |

---

## 9. Recovery, Checkpoint, Restore

### 9.1 Öffnen (deterministisch)

```
OP1   LOCK (exklusiv; ReadOnly geteilt)
OP2   FORMAT-Kopf → KEYS entpacken (KEK ⇒ RootKey) → FORMAT-MAC prüfen → LedgerScopeKey ableiten
OP2a  Anker/Ledger: Catalog-Epoche < Strong-Anchor ⇒ Err(Rollback). Ledger hat ≥ E_a Einträge, Eintrag E_a trägt H_a.
      Durables Ledger-Intent hinter dem Anker ist ein Crash-Präfix: vor Schreiböffnung idempotent fortsetzen;
      Required-Anchor unerreichbar ⇒ Locked/Err(AnchorUnavailable), nie schwächer öffnen; Ledger nicht lesbar ⇒ LedgerUnavailable
OP3   CURRENT → Catalog bis zum letzten gültigen Record
OP4   Verzeichnisse scannen: Dateien ohne Catalog-Eintrag = verwaist (noch nicht löschen)
OP5   KeyProvider prüfen; KEK fehlt ⇒ LockedKeys
OP6   Redo ab replay_floor, zwei Durchläufe (versiegelte Ledger-Intents sind zusätzliche Gate-Wahrheit):
        1: Kette prüfen; Cluster-Retires, Range-/Branch-Masken, Space-Gates aus Catalog/Redo/Ledger sammeln
        2: Retired-Cluster überspringen (nie entsiegeln); Selektor-Masken anwenden; KeyUnavailable ⇒ LockedKeys;
           bestätigtes KeyLost ohne Erase-Intent ⇒ Quarantäne; Branch-/Merge-Frames anwenden
OP7   Memtable aufbauen; visible = durable = letzter gültiger Stamp; next_stamp (§5.2); Indeterminate-Changes auflösen
OP8   Vor Freigabe von Views/Schreibzugriff: alle Ledger-Erase-/SpaceDrop-Intents idempotent in Catalog-Gates überführen (auch Intents mit origin=Poisoned);
      offene Löschungen = Intents ohne GateApplied ∪ erase_pending ∪ Gate-Frames ohne erase_done ∪ Retired ≠ Purged ∪ Space Dropping ⇒
      fehlender LedgerScopeKey ⇒ RestoreQuarantine/LockedKeys; sonst urgent fortsetzen; kein Body wird vor Gate-Auflösung entsiegelt
OP9   Verwaiste Dateien und .building.* entfernen (nach OP8)
OP10  Reserven herstellen; Admission-Prüfungen (FD-Limit, max_live_files, cluster_meta_budget); Wartung starten; OpenReport
```

Ehrliche Grenze: Ein beschädigter letzter quittierter Gruppenabschnitt hinter der letzten `redo_mark`, dem nichts Gültiges folgt, ist von einem Torn Write nicht unterscheidbar; die Heilung kürzt dann. `redo_mark` hält das Fenster klein.

### 9.2 Absturzmatrix (Auszug; vollständig in der Testsuite)

| Unterbrechung | Soll |
|---|---|
| vor Redo-`sync` | Change nicht sichtbar |
| nach Redo-`sync`, vor Rückkehr | Change vorhanden (Token verhindert Doppelung) |
| Run-Bau / vor Edit / vor Unlink | `.building` entfernt bzw. Run verwaist/entfernt, Redo spielt ein |
| mitten in einer Catalog-Gruppe | gültiger Präfix gilt |
| Ledger-Intent torn | nur vollständiger valider Intent gilt; Retry findet Ticket oder legt neu an |
| G1 durabel, G2 fehlt | OP8 rekonstruiert das Gate aus dem Intent |
| G2 durabel, G3/G4/G5 fehlen | Nachzug vor Open; keine irreversible Folge vor Anker |
| Anker durabel, KMS-Aufruf offen | `KeyDestroyIntent` prüfen, `destroy_many`/`probe` idempotent fortsetzen |
| Schlüssel vernichtet, `KeysDestroyed` fehlt | `probe == Absent` bei passendem Intent ⇒ nachholen |
| Redo-Barriere unterbrochen | ER4 neu, idempotent |
| Purge-Edit durabel, Unlink offen | Verwaisten-Suche, `unlink_ack` idempotent |
| Edit durabel, Anker alt | Nachzug in OP2a |
| Branch-Fork-Frame durabel, Memtable nicht | Replay stellt Tabelle wieder her |
| Merge-Frame durabel, Source-Zustand nicht | `Merged` aus Frame abgeleitet |
| `DiskFull` beim Vorallokieren | sauberer Fehler, keine Teilschreibung |
| Checkpoint abgebrochen | `dst` unvollständig ohne Siegel, Quelle unverändert |
| Crash zwischen IG-2 und IG-3 (§11.4) | Staging unerreichbar, Orphan-Worker räumt auf |

### 9.3 Checkpoint, Restore, Rollback-Schutz

- **Kern-Checkpoint:** (1) Flush-Barriere; (2) unter `catalog_edit` Version pinnen, `erase_epoch` und Ankerwert lesen; (3) unabhängige `Copy`-Dateien (`Reflink`/`Hardlink` nur mit registriertem Checkpoint-Owner, der vor Scratch-Recycling Referenzen entfernt und attestiert; sonst Restrisiko); (4) Manifest siegeln (`erase_epoch`, `created_at`, höchste offene `tomb_epoch`). `KEYS` ist enthalten, **KEK und Clusterschlüssel nicht** (`INV-KEY-1`). Das Ledger liegt nie im Checkpoint. Das Manifest listet bereits `Retired`, noch nicht gepurgte Plain-Cluster.
- **Systemweiter Checkpoint (`contextra-checkpoint`):** Snapshot-Set-Register und Restore-Koordinator. Er koordiniert über `SnapshotParticipant` Kern **und** Index-Generationen (Manifest, beglaubigt per `Mac`-Port, in einem `NonPersonal`-System-Space). Benannte `SnapshotSet`s sind **lease-begrenzt** und halten im Kern einen `Hold` (≤ 24 h, verlängerbar); `EpochPinRegistry` hält die Index-Generationen. **Erase überstimmt jeden Pin (FIX-08):** wird ein Cluster gegated oder eine Partitionsgeneration für Vernichtung freigegeben, wird ein betroffenes `SnapshotSet` widerrufen (`SnapshotRevoked`); ein Pin darf nie einen Generationsschlüssel am Leben halten.
- **Restore** ersetzt die bisherige Instanz und erhält die `InstanceId` (NZ-19). Der Kern startet in `RestoreQuarantine` (keine Nutzer-Reads), verlangt vor schreibendem Open externen Fencing-Nachweis, validiert Ledger-Kette und Anker und spielt jeden Intent scope-genau ein (`INV-ERA-9`): Cluster-Intents matchen HMAC-Tombstones und retiren Cluster; Range-/Branch-Intents werden entschlüsselt und als Masken auf alle restaurierten Versionen angewandt; `SpaceDropIntent` matcht die SpaceId und gated den Space. Bei `KeyUnavailable`, `KeyLost`, fehlendem Scope-Key, Fencing-Nachweis oder Ledger-Mismatch bleibt Restore quarantined; ein fehlender Schlüssel/Anker ist nie eine Löschung. Danach werden offene KeyDestroy-/Purge-/Extension-Stufen fortgesetzt; Index-Generationen werden ab `(replay_floor / durable_stamp)` neu aufgebaut. Liegt das Manifest hinter einem Strong-Anker ⇒ Rollback verweigert. Erst nach `RestoreReconciled` startet `Running`.
- **Ehrliche Grenze:** Wird zusätzlich der Anker-Dienst zurückgesetzt, ist der Schutz nur noch kryptografisch.

### 9.4 Backup-Horizont und Tomb-Retire

Problem: Restore gleicht Backups über `tomb_id` ab. Ewig auflösbare Pseudonyme machen das Ledger ewig personenbezogen; zu früh anonymisierte lassen alte Backups Gelöschtes beleben.

- `backup_horizon` = maximale Aufbewahrungsdauer aller wiederherstellbaren Kopien. `BackupMark{instance_id, erase_epoch, created_at, tomb_epoch}` liefert `Storage::backup_mark()`; Checkpoint-Manifeste tragen sie automatisch, externe Backups müssen sie mitführen.
- **Default:** kein Retire (Epoche `0`, `tomb_id` linkbar, Restore gleicht gegen alle Intents ab, kein `restore_floor`).
- **Retire-Bedingungen** (alle): (1) Epoche `Closed` (neuere existiert; `last_erase_epoch(e)` final); (2) `now − closed_at ≥ backup_horizon` bei verifizierter Uhr; (3) kein offenes Ticket, kein `EraseIntent` ohne `Completed`, kein offener Restore mit `tomb_id`s dieser Epoche.
- **Ablauf:** `TombEpochRetireIntent{epoch, restore_floor' = max(restore_floor, last_erase_epoch(e))}` durabel → Anker → `retire_tomb_epoch(e)` liefert `Absent` → `TombEpochRetired`. Roll-forward; ein Intent ohne Providerbestätigung bleibt `Blocked`.
- **Restore-Regel:** Kopie mit `mark.erase_epoch < restore_floor` ⇒ `Err(BackupBeyondHorizon{floor, mark})`; kein Override. Kopie ohne Marke wird abgelehnt, sobald irgendeine Epoche retired ist. Gegen nicht retired Epochen gleicht Restore weiter ab (Kosten `Cluster × aktive Epochen` HMACs, `max_active_tomb_epochs` 4, H-23).
- **Wirkung:** Nach Retire sind alle `tomb_id`s der Epoche anonym; Kettenprüfung und `verify_receipt` bleiben gültig. Kopien von Tomb-Schlüsseln außerhalb des Providers heben die Anonymität auf; ein `Bounded`-Provider macht Retire erst nach `until` wirksam.

---

## 10. Nebenläufigkeit

### 10.1 Prinzipien und Sperrränge

Ein Sequencer (Rang 1, ohne E/A) für stamp-relevante Zustände und ein Syncer für das Redo; unveränderliche Dateien und `Arc<Version>`; Besitz durch Claims; Wahrheit durch durable Edits/Frames; Generationszähler statt Sperren für Schlüsselcache, Blockcache, Clusterzustand; keine Sperre über E/A außer `catalog_edit`; Ledger/Anker haben je einen Schreiber über Kanäle.

| Rang | Sperre | Über E/A | Bemerkung |
|---|---|---|---|
| 1 | `sequencer` | nein | Stamp-Vergabe, `Live → Retiring`, Link/Restrict/Lift, Branch-Ops; nur RAM |
| 2 | `intake` | nein | |
| 3 | `catalog_edit` | ja | Gruppen-Commit |
| 4 | `state` (`RwLock<Arc<Version>>`) | nein | Veröffentlichung |
| 5 | `registry` (Holds, Consumer, Claims, Restrictions, `ViewInner`) | nein | Blatt |
| 6 | `cluster_table` (sharded, inkl. Branch-Tabellen) | nein | Blatt, Lesen aus Rang 1 zulässig |
| 7 | `maint` (je Lane) | nein | Blatt |
| 8 | `notify` | nein | Blatt |

Nur aufsteigend, je Thread höchstens einmal (`debug_assertions` prüft per Rangstack). Rang 1 und 3 nie gleichzeitig. Rang 2 nimmt der Leader nur unter Rang 1. Kein Rückruf (`Observer`, `SegmentOwner`, `KeyProvider`) unter Sperren, insbesondere kein Provideraufruf unter Rang 1. Kein `std::sync::Mutex` außerhalb `sync::Ranked`.

### 10.2 Lock-freie Pfade, Views, Claims

- `ArcSwap`/`RwLock<Arc<_>>` für `Version`; Publikation: Version, dann `visible_stamp` (Release); Leser laden Stamp (Acquire), dann Version. Clusterzustand = `AtomicU64 (state:8 | gen:56)`; nur der Sequencer setzt `Live → Retiring`, die Publikation nach durablem Sync setzt `Retiring → Retired` (`INV-LOCK-2`).
- **Views:** `ViewInner { version: RwLock<Option<Arc<Version>>> }`; `Revoke` setzt `None`, spätestens nach `revoke_grace` (5 s) hält keine widerrufene View eine Referenz. Eine Datei wird erst gelöscht, wenn der entfernende Edit durabel **und** die letzte `Arc<Version>` gefallen ist; `files_remove` hält sie bis `unlink + sync_dir` im persistenten Graveyard.
- **View-Lease:** jede View/jeder Cursor trägt eine Lease (`view_lease` 60 s, monotone Uhr), verlängerbar per `renew()` bis `view_lease_max` (10 min ab Erzeugung). Nach Ablauf wirkt die View wie `Revoke`; jede Nutzung ⇒ `Err(SnapshotTooOld{stamp, floor_hint})`. `SnapshotTooOld` (Lease) und `BelowFloor` (Stamp nicht mehr vorhanden) sind getrennte Fehler. Lange Operationen (Export, Scans) setzen über `ResumeToken` + `view_at(V)` fort und halten dafür einen `Hold`. Erase widerruft unabhängig von der Lease; `Wait(timeout)` wartet höchstens bis `min(timeout, Lease-Ende)`.
- **Claims:** `claim(files, job)` atomar unter `registry`; Priorität `urgent > flush > compact > blobgc > idle`; höhere Priorität setzt `cancel`; Ausgabe-Edit abgelehnt, wenn `files_remove ⊄ Claims` (`INV-CLAIM-1`); Compaction verwirft Ausgabe, wenn ein Eingabecluster inzwischen `Retired` ist.

### 10.3 Race-Nahtstellen

| ID | Race | Lösung | Nachweis |
|---|---|---|---|
| RACE-1 | Lesen × Gate | Gate-Check am Anfang, `gen` nach Entsiegeln | loom |
| RACE-2 | Flush × Erase | Gate vor Memtable-Einfügung, Flush überspringt Retired, Barriere | M2, Sim |
| RACE-3 | Compaction × Erase | Claims, `retire_gen`-Prüfung im Edit | M3, Sim |
| RACE-4 | Cache × Vernichtung | `gen` vor und nach Entsiegeln | loom |
| RACE-5 | Checkpoint × Erase | `Arc<Version>` + `erase_epoch`, serialisiert gegen ER6 | M4, Sim |
| RACE-6 | Consumer × Erase | `retain_floor`, `Lagged`, `Erased`-Change | M2 |
| RACE-7 | Rebucket × Lesen | Gruppe im selben Edit | M5 |
| RACE-8 | Zwei Erase, überlappende Hülle | Serialisierung nach Ticket-ID in `urgent` | Sim |
| RACE-9 | Restriction × Erase | beides im Sequencer | M2 |
| RACE-10 | Link × Erase | Frames im Sequencer total geordnet; `policy_epoch`, Retry, Selbstberechnung | M2 |
| RACE-11 | Guard × Gate | Zustandsprüfung (6a) vor Guard (6b) | M2 |
| RACE-12 | Anker × Edit | Anker nie vor Edit, Nachzug in OP2a | M6 |
| RACE-13 | Staging × Erase | Purge behandelt Staging wie Packed | M3, Sim |
| RACE-14 | Space-Drop × Flush/Compaction | Claims, `Dropping` bricht Jobs ab | M2, Sim |
| RACE-15 | Merge × Gate/Restriction | Merge ist ein Change; Zustandsprüfung 6a | M2 |
| RACE-16 | Fork × Compaction | `floor` berücksichtigt `fork_stamp`s | M3 |
| RACE-17 | Branch-Drop × Merge | beides im Sequencer; `BranchClosed` | M2 |
| RACE-18 | Sync-Fehler × Publikation | nur nach Sync derselben Gruppe in Stamp-Ordnung | M1, Sim |
| RACE-19 | Leader-Wechsel × Intake | Intake nach Freigabe erneut prüfen | loom |
| RACE-20 | Guard Stufe A × Pipeline-Delta | Konfliktfenster enthält nicht durable Stamps; `V1 < Fensterfuß` ⇒ Retry | M1, Sim |
| RACE-21 | Space-Drop × neue Cluster-Zuordnung | Space-Gate schließt Writes im selben Frame | M2 |
| RACE-22 | Link × Gate (gleiche Quelle) | Link ist Control-Frame; Quelle `Retiring`/`Retired` ⇒ `ClusterRetired` | M2 |
| RACE-23 | Gate × Query × Index-Rebuild | Gate-Check vor Payload, Generation-Fencing, `IndexLag` statt Treffer | Sim |
| RACE-24 | Ingest-Head × Gate | Abschluss-Change scheitert mit `ClusterRetired`; Staging verwaist | M2, Sim |
| RACE-25 | SnapshotSet-Pin × Erase | Erase widerruft Pins und Holds | M4, Sim |

---

## 11. Erweiterungsverträge, Retrieval, Ingest, Query

Fragen dieses Kapitels: Wo liegen Wahrheit und Ableitung (§11.4)? Wie bleiben Ableitungen löschbar (§11.2, §11.5)? Wie entsteht ein reproduzierbares Ergebnis (§11.10)? Wie sind personenbezogene Daten getrennt (§11.9, §11.13)?

### 11.1 Grundregeln (EX-1…EX-9)

| ID | Regel |
|---|---|
| EX-1 | **Wahrheit** = kanonische Records im Kern (atomar, durabel, cluster-sealed). **Ableitung** = Indizes, Graphen, Präfixsummen, Caches; jederzeit aus der Wahrheit neu aufbaubar. |
| EX-2 | Kein Schreibpfad ändert zuerst einen Index und dann den Kern. Kein 2PC, kein `CommitLedger`, keine Kompensation. Ein Index ist nie Teil der Commit-Entscheidung (L12). |
| EX-3 | Retrieval-Crates (`vector`, `text`, `graph`, `rank`) sind persistenzneutral und kennen weder `storage` noch einander. Die Bindung geschieht ausschließlich in `engine::projection` (§2.1). |
| EX-4 | Jede abgeleitete Struktur ist entweder `ClusterSections` (Blöcke im Kern, unter dem Clusterschlüssel) oder ein mandantenreiner `Rebuildable`-Owner mit Generationsschlüssel (§11.5). Anderes ist für Strukturen, die aus Nutzdaten abgeleitet sind, unzulässig. |
| EX-5 | Jeder Kandidat passiert den Kern-Gate-Zustand und `View::filter_visible` vor Scoring **und** vor Ausgabe (`INV-VIS-1`). Owner-eigene Masken (`on_gate`) sind Beschleuniger, nie Korrektheitsbarriere (FIX-11). |
| EX-6 | Ein Index-Watermark ist `(instance_id, applied_stamp)` und reicht nie über `durable_stamp` derselben Instanz (`INV-RAG-2`). Vergleiche über Instanzen sind verboten. |
| EX-7 | Quelldaten und Indexgeneration sind nicht gemeinsam atomar. Der Vertrag liefert Durability, Wiederaufbau, Idempotenz, Sichtbarkeit und Löschbeleg, nicht Null-Lag. Lücken ⇒ Delta-Pfad oder `IndexLag`, nie stilles Auslassen (`INV-RAG-3`). |
| EX-8 | Jede Allokation von Index-RAM, Scratch und I/O reserviert vor der Allokation am gemeinsamen `MemoryBudget`. Unter Druck wird Indexaufbau gedrosselt oder abgewiesen; Commit-Fähigkeit und Gate-Priorität bleiben erhalten. |
| EX-9 | Tenant-/Cluster-IDs, Querytexte, Schlüssel, Inhalte und Embeddings sind nie Metrik-Label oder Standard-Observer-Payload (§15). |

### 11.2 `SegmentOwner`

```rust
pub trait SegmentOwner: Send + Sync {
    fn kind(&self) -> SegmentKind;
    fn manifest(&self, seg: &SegmentMeta) -> SegmentManifest;      // class, key scope, bounds, RAM/scratch, lineage; vor Erstnutzung, versioniert
    fn on_gate(&self, erased: &[ClusterId], at: Stamp, gen: u64);  // synchron, nur RAM, O(|erased|): Tombstone-Maske setzen, Worker wecken
    fn rewrite_without(&self, seg: &SegmentMeta, drop: &ClusterSet, io: &mut SegmentIo) -> Result<RewriteOutcome, SegErr>; // Pflicht
    fn verify_absent(&self, scope: &EraseScope, gen: u64) -> Result<EraseAttestation, SegErr>;
    fn watermark(&self) -> DurablePosition;                         // lückenlos verarbeiteter durabler Feed
}
```

`SegmentClass`: `ClusterSections | MixedPerClusterKey | MixedPerSubjectKey | Rebuildable{max_age, max_rewrite_bytes} | NoPersonalData`.

| Klasse | Zulässig für | Löschwirkung |
|---|---|---|
| `ClusterSections` | Blöcke als Records/Abschnitte des Kerns | Gate + `KeysDestroyed` genügen; `rewrite_without` trivial |
| `MixedPerClusterKey` | nur wenn Payload **und** alle Metadaten genau einem Cluster gehören; je Key-Epoche | Schlüsselvernichtung |
| `MixedPerSubjectKey` | ausschließlich Subjekt-Mappings (`IdentityDirectoryOwner`, §11.13) mit je eigenem Entry-Schlüssel | `destroy_entry_key` + `rewrite_without` + `verify_absent` |
| `Rebuildable` | clusterübergreifende Strukturen (HNSW-Kanten, Postings, Centroids, CSR, Routing) | `Purging` bis Rebuild + Generationswechsel + Schlüsselvernichtung + `verify_absent`; über Obergrenze/Deadline ⇒ `Blocked`/`Incomplete` |
| `NoPersonalData` | Strukturen ohne Nutzdatenbezug mit versionierter Provenienz-/Schema-Evidence (keine Selbstdeklaration) | keine Crypto-Erasure-Zusage |

Regeln:

1. **Beweis (FIX-05):** `RewriteOutcome` ist kein Löschbeweis. Nur `EraseAttestation{owner, generation, scope_digest, applied_position=(instance_id, stamp), keys_destroyed, segments_rewritten, evidence_digest}` nach erfolgreichem `verify_absent` ist ein Ack (Ledger `ExtAck`).
2. **`on_gate` (FIX-11):** Es setzt synchron, vor der Rückkehr der Gate-Publikation, eine generationsgefencte Tombstone-Maske, damit für gegatete Einträge weder Distanz- noch Score-Arbeit anfällt. Es ist **nie** die Korrektheitsbarriere: jede Query prüft den Kern-Gate-Zustand unabhängig (EX-5). Fehlt die Maske, sind Ergebnisse gleich, nur langsamer; Scoring-Statistiken (`df`, PPR-Masse) können bis zum Rebuild driften und werden als `stats_drift` gemessen.
3. **Liveness:** Jeder Owner-Aufruf läuft mit Deadline und Fencing-Generation, außerhalb aller Kernsperren. Timeout ⇒ Ticket `Blocked{owner, reason, since, retry_at}`, Ledger `ExtBlocked`, Alarm, Eskalation; verspätete Antworten alter Generation werden verworfen. `Blocked` zählt nie als `Done`. Ersatz/Unregister eines Owners ist ledger-protokolliert und verlangt mit neuer Generation `verify_absent`.
4. **I/O:** Zugriff nur über das versiegelte `SegmentIo` (kein `std::fs`, kein `mmap`, kein fester Schlüssel). Jedes Job-RAM-/Scratch-Byte ist vor der Allokation per `MemoryBudget` reserviert; ohne Budget-Token startet keine Indexarbeit.
5. **Deklaration:** Ein Owner darf nicht `NoPersonalData` deklarieren und gleichzeitig personenbezogene Lineage verarbeiten. Ein persistierender Feed-Consumer, der Derivate erzeugt, ist als Owner/Extension mit Quell-Cluster-Lineage zu registrieren; nicht registrierte externe Persistenz verhindert systemweites `Complete` und steht als Restrisiko im Receipt.
6. **Sealed:** Jeder clusterbezogene Payload ist mit Clusterschlüssel/Epoche oder gleichwertig isoliertem Schlüssel geschützt; ein globaler Segmentschlüssel erfüllt `INV-ERA-7` nicht.

### 11.3 `ExtensionBlock` und `BlockPruner`

Opake Blöcke (Tag `u16`, Version `u16`) je Cluster-Abschnitt eines Runs. `ExtensionBuilder` erhält den Eintragsstrom bei Flush/Compaction; `merge` und `rewrite_without` sind Pflicht; unbekannter Tag beim Rewrite ⇒ Fehler. Ein Besitzer ohne `rewrite_without` darf in `clustered`-Spaces keinen Block anlegen.

**Block-Aggregate:** `ExtensionBuilder::block_boundary(block_uid, entries) -> Option<AggBytes>` (≤ 64 B, z. B. Min/Max eines Zeitfelds aus dem Kopf) wird an jeder Datenblockgrenze aufgerufen. `block_uid` ist der abschnittsrelative Blockindex (positionsunabhängig); bei Sealed-Clustern ist das Aggregat clusterschlüssel-versiegelt. `BlockPruner::may_skip(block_uid, agg) -> bool` liegt beim Besitzer und wird `page`/`Cursor` übergeben.

1. Der Kern persistiert keine Zeitlanes oder Aggregate im SubIndex und ruft `may_skip` nur, wenn das Manifest `PruneContract::ShadowSafe{reason}` deklariert: für jeden Schlüssel eines übersprungenen Blocks gibt es keine neuere Version und keinen Tombstone in einem nicht übersprungenen Block (z. B. append-only mit eindeutigen Schlüsseln). Der Kern kann das nicht prüfen; die Zusicherung trägt der Besitzer.
2. `may_skip` wird nur für Lesevorgänge auf `main` (`branch == 0`) aufgerufen.
3. Gate, Restriction und `filter_visible` laufen unabhängig; ein Pruner spart Arbeit, stellt nie Sichtbarkeit her.
4. Konformanz: Property-Test „mit Pruner ≡ ohne Pruner“ gegen das Referenzmodell inkl. Overwrites und Deletes; wer ihn nicht besteht, darf `ShadowSafe` nicht deklarieren (`INV-PRUNE-1`).

### 11.4 Ein-Wahrheits-Commit, Publish-Record, Feed, Projektion

**Kanonisch (Kern):** je Chunk Metadaten im Kopf, Text im Körper, kanonischer Vektor (Space `emb/<model_id>`, §11.6), Vorwärts-Termliste `fw` samt Dokumentlänge `dl`, Graphfakten mit Provenienz (`edge`). **Abgeleitet:** HNSW/DiskANN, invertierte Listen, CSR-Segmente, Präfixsummen, Caches.

**Publish-Record (Protokoll IG).** Ein Change ist durch 4096 Ops/8 MiB begrenzt, ein Dokument nicht.

| Schritt | Vorgang |
|---|---|
| IG-1 | Klassifikation und Routing (§11.9), Pseudonymisierung, Chunking, Embedding **außerhalb** jeder Storage-Sperre |
| IG-2 | Chunk-Records der Version `n` unter `doc/<id>/v<n>/…` in 1…m Changes; jeder Change trägt einen aus `(ingest_id, j)` abgeleiteten `IdemToken`. Sie sind unsichtbar, weil nur über `head` erreichbar |
| IG-3 | **Abschluss-Change:** `doc/<id>/head = {version n, chunk_range, chunk_count}` mit `Guard::Version(head_alt)`; derselbe Change tombstoned die Vorversion, soweit sie ins Limit passt, sonst folgt ein Aufräum-Change (die Vorversion ist ab IG-3 unerreichbar) |
| IG-4 | Der `ConsumerFeed` löst `head` aus; die Projektion liest die Chunks der referenzierten Version über eine View bei Stamp ≤ Head-Stamp und wendet sie als **eine** Einheit an (nie halb indexiert) |
| IG-5 | Crash vor IG-3: Staging bleibt unerreichbar; `staged_orphan_ttl` (24 h) und ein Worker räumen auf. Crash nach IG-3: Recovery stellt `head` her, die Projektion setzt am Watermark fort |
| IG-6 | Passt das Dokument in ein Limit, entfällt Staging: Chunks und `head` sind **ein** Change (Normalfall) |

`head` enthält Zähler und Schlüsselbereiche, **keine** Inhalts-Hashes (`INV-PRIV-1`); Integrität sichern AEAD und Ketten-MAC. `filter_visible` mit Prüfart `DocHead`: ein Chunk ist sichtbar, wenn seine Version dem `head` der Sicht entspricht (RAM-Vergleich über gecachte Head-Tabelle, Punktlesen nur bei Cache-Miss). Scheitert IG-3 mit `ClusterRetired`, bleibt das Staging verwaist (RACE-24).

**Atomarität in Zahlen.** Je Chunk ≈ 5 Ops (Chunk, `emb`, `fw`, ≈ 2 `edge`); je Change entfällt eine Op auf `SYS_IDEM`. 1000 Chunks ≈ 5000 Ops ⇒ 2 Staging-Changes + `head`. Der Syncer bündelt (§5.1); die Fsync-Zahl je Dokument ist unabhängig von der Indexlast.

**Feed (`Consumer`/`ConsumerFeed`).** Gespeist aus dem Redo (Runs sind nicht stamp-geordnet). Böden: `replay_floor` (Recovery) und `retain_floor ≤ replay_floor` = `min(replay_floor, kleinste Consumer-Position, Grenze aus max_lag_bytes)`. Der Consumer persistiert `(instance_id, stamp)` (`consumer_ack`) und pinnt nur Redo-Segmente derselben Instanz.

1. Reihenfolge nach Stamp, mindestens einmal, nur Stamps ≤ `durable_stamp` (`INV-CONS-1`); `max_lag_bytes` 1 GiB; Überschreitung oder Plain-Barriere ⇒ `Err(Lagged)` (Aufholen per Vollscan über eine View).
2. Sealed-Nutzlasten entsiegelt der Kern. **Nach dem Gate** erscheint `Change::Redacted{cluster}`, nach `KeysDestroyed`/Purge `Change::Erased{cluster|range, at}`; `BLOCK_READ`-Cluster erscheinen als `Restricted`-Platzhalter. Branch-Frames tragen `branch`; Volatile-Spaces erscheinen nicht.
3. Consumer-Positionen gehen nur bis `max_lag_bytes` in `gc_floor` ein; darüber `Lagged` statt unbegrenztem Versions-Pinning.
4. **DiskFull (FIX-38):** Der Governor setzt `retain_floor := replay_floor`; Consumer hinter dem Boden erhalten `Lagged`. Retention-Platz wird nie gegen Gate/KeyDestroy gehalten.
5. Projektion ist idempotent über `(instance_id, stamp, op_index)`; Wiederholung ändert den Zustand nicht.

### 11.5 Index-Partitionen, Generationsschlüssel, Rebuild-Kosten

**Regel.** Eine Partition gehört genau einer Packgruppe `(tenant_tag, due_bucket, shard)` (`INV-PART-1`) und hat harte Größenlimits; Überschreitung ⇒ Wartung spaltet nach `shard`. Der Index folgt der Platzierungsregel der Daten: `pack_expired` verwirft die Partitionen einer Gruppe in O(Dateien). Jede Partition trägt einen zufälligen `PartitionTag` (16 B, keine ClusterId).

| `IndexClass` (FIX-13) | Inhalt | Limits | Lebensdauer |
|---|---|---|---|
| `Corpus` | aus `NonPersonal`-Chunks abgeleitet | `max_partition_vectors` 500 000 · `max_partition_postings_bytes` 256 MiB · `max_partition_edges` 5 Mio. | bis Löschung/Neuaufbau |
| `Pseudo` | ausschließlich aus **pseudonymisiertem** Text von `Personal`-Chunks abgeleitet (§11.9) | `…_personal`: 100 000 Vektoren · 64 MiB Postings · 1 Mio. Kanten | `Rebuildable{max_age = purge_window_fraction × purge_deadline}`; Gruppe nach Aufbewahrungsende (`due_bucket`) |

(R13 nannte die zweite Klasse `Personal`; das kollidierte mit `DataClass::Personal`. `Pseudo` bleibt personenbezogen im Sinn pseudonymisierter Daten und erhält deshalb Löschpflicht über Refcounts.)

`INV-SEP-2`: Eine geteilte Partition enthält keinen aus Personen-Klartext abgeleiteten Vektor, Term oder Graphknoten. Klartext-Embeddings liegen nur in Cluster-Abschnitten des Personenclusters (Modus A/C, §11.6) unter dessen Clusterschlüssel.

**Blockformat.** `PartBlock := SealFrame`, AAD `(instance_id, partition_tag, generation, block_uid)`, Schlüssel = `partition_key(tag, generation)`; keine Datei-/Offset-Bindung (Rohkopie gültig). Integrität = AEAD/MAC unter dem Generationsschlüssel; kein fester Schlüssel (`DISKANN_INTEGRITY_KEY` entfällt).

**Generationslebenszyklus.**

```
Building(g+1) ──alle Deltas bis applied_stamp angewandt, Attest──► Published(g+1)
Published(g) ──Publish g+1──► Draining(g) ──revoke_grace, Pins gelöst/widerrufen──► KeyDestroyed(g) (probe == Absent)
```

1. Rebuild(g+1) liest ausschließlich lebende Quellen (kanonische Records über eine gepinnte View, Gate-gefiltert) und schreibt unter neuem Schlüssel (`create_partition_key`).
2. **Pins (FIX-08):** Ein Gate auf einen Quellcluster widerruft jedes `SnapshotSet` und jede Query-Sicht, die Generation `g` dieser Partition hält (`SnapshotRevoked{reason=Erase}`); ein Pin hält nie einen Generationsschlüssel am Leben. Betroffene Halter wiederholen auf `g+1`; Reproduzierbarkeit dieses Satzes ist damit bewusst beendet.
3. `Done` des Index-Owners verlangt `Absent` der Vorgängergeneration (`INV-PART-1`). Fehlender Schlüssel einer nicht vernichteten Generation ⇒ `KeyLost` der Index-Domäne (Partition neu aufbauen, Ticket `Incomplete`).
4. Backup-Kopien von `g` sind nach `Absent` unlesbar; das ersetzt Pro-Cluster-Schlüssel im geteilten Index.

**Kostenmodell.** `P` = Partitionsgröße; `D(w)` = Partitionen mit Löschungen im Fenster `w = purge_window_fraction × purge_deadline`. Je Fenster wird jede Partition höchstens einmal neu aufgebaut: Rebuild-Last `Σ_{p∈D(w)} P_p`, nie proportional zur Zahl der Aufträge (H-32). Auslöser wie Purge (§8.5): natürlicher Neuaufbau, 60 % der Frist, oder Bündelung. Lane `compact`, eigener Budgetanteil `index_rebuild_bytes`, `maint_io_fraction`; verdrängt weder Gate/KeyDestroy noch Commits. Reicht die Kapazität nicht, wird das Ticket `Blocked{owner=index, reason=RebuildCapacity}` und alarmiert; kein stilles `Done` (`INV-ERA-8`).

**Refcounts (`Pseudo`).** Der Projektions-Owner führt je Person einen Refcount Platzhalter → Partition (nur RAM-Cache, aus `fw` ableitbar; nach Neustart lazy rekonstruiert). Personenlöschung ⇒ Rebuild genau der Partitionen mit Refcount > 0.

| Variante | Löschkosten | Suchlatenz | Betrieb | Urteil |
|---|---|---|---|---|
| Ein globaler Index je Mandant | Vollrebuild je Löschfenster, wächst mit dem Korpus | beste | einfach | verworfen (Z-4) |
| Ein Index je Cluster | minimal | schlecht clusterübergreifend, 10⁷ Segmente | Dateiflut | nur als Mikro-/Cluster-Index (Modus A/C) |
| **Partition = Packgruppe** | beschränkt durch `P`, amortisiert | Fan-out ≤ Partitionszahl je Mandant | folgt dem Datenlayout | **gewählt** |
| Gerichtetes Re-Wiring ohne Rebuild | scheinbar gering | Recall-Verlust | verletzt `INV-HNSW-SLOT-1` | verworfen |

### 11.6 Vektorindex (`contextra-vector`, `contextra-simd`)

**Kanonischer Vektor (FIX-12).** Wahrheit ist der **Vollpräzisionsvektor** als Körper eines Records in `emb/<model_id>` (`emb_precision ∈ F16 | F32`, Standard `F16`), nicht der quantisierte Code. Nur so sind Rerank und Neuaufbau ohne erneutes (kostenpflichtiges, modellabhängiges) Embedding möglich. SQ8-/RaBitQ-Codes sind abgeleitet und liegen in Partitionen. Profil `Lean` (`emb_precision = Sq8`) spart Platz, schaltet Rerank ab und weist die Recall-Einbuße im `explain` aus.

| Modus | Bereich | Persistenz | Löschung |
|---|---|---|---|
| **A Mikroindex** | Cluster ≤ `micro_index_max_vectors` (50 000) | keine; flacher Scan oder RAM-Mini-HNSW aus `emb`-Records, im Block-Cache (`gen`-gefencet) | Gate (Cache `gen++`) und `KeysDestroyed` |
| **B Partition** | geteilte Mengen aus `Corpus`/`Pseudo` | HNSW (`M`, `ef`) oder DiskANN je Partition, `Rebuildable`, Generationsschlüssel | `on_gate`-Tombstone, `INV-HNSW-SLOT-1`: Tombstone + Slot-Nullung + Partitions-Neuaufbau |
| **C Cluster-Index (FIX-19)** | Sealed-Cluster mit `micro_index_max_vectors` < n ≤ `cluster_index_max_vectors` (2 Mio.) | HNSW-Blöcke als abgeleitete Records im Cluster-Space `ix/<model_id>` (cluster-sealed, `producer`-Kopf, `derived`-Flag), aufgebaut von der Projektion als normale Changes | Gate + `KeysDestroyed` (`ClusterSections`, kein Rebuild nötig) |

Modus C schließt die Lücke zwischen Mikroindex und dem Verbot von Klartext-Embeddings in geteilten Partitionen (`INV-SEP-2`). Darüber hinaus ⇒ `Err(Limit{ClusterIndexVectors})` mit `hint()`: Akte in Teilcluster nach Löschdomäne aufteilen (§11.13).

| Aspekt | Vertrag |
|---|---|
| Kandidatengenerierung | SQ8 (`Sq8Bias`) und RaBitQ (Rotation pro Generation, im Manifest); Rerank mit dem kanonischen Vektor nur für die besten `R` Kandidaten (`rerank_depth` = 4 × k) |
| Filter | Mandanten-/Gate-Maske **vor** Traversierung; ACORN-Augmentierung bei `filter_selectivity` < 5 %; NaN/Inf werden abgewiesen (`INV-SIMD-FINITE-1`) |
| Determinismus | Ebenen aus `hash(partition_seed, doc_id)` bzw. injiziertem `Rng`; SIMD-Reduktionsordnung fest und bitidentisch zur Skalar-Referenz (`INV-SIMD-DETERMINISM-1`); gleicher `SnapshotSet` ⇒ gleiche Treffer |
| Isolation | keine Kanten und Caches über Mandanten (`INV-TENANT-IDX-1`) |
| Modellwechsel | neuer Space `emb/<new_model>`; Dual-Betrieb bis Migrationsjob fertig; Löschung wirkt auf alle Modell-Spaces, da sie im selben Cluster liegen |
| Delta | `Index ∪ Delta (applied_stamp, V]`: Delta ist flacher Scan über neue `emb`-Records; Recall ≥ Basis (H-29) |

### 11.7 Volltextindex (`contextra-text`)

| Aspekt | Vertrag |
|---|---|
| Vorwärtsdaten | `fw`/`dl` clusterlokal als Records; Quelle jedes Neuaufbaus |
| Analyse (Deutsch) | Normalisierung (NFKC, Kleinschreibung, ß/ss, Umlautvariante als Zusatzterm), regelbasierte Lemmatisierung/Stemming, Komposita-Zerlegung (`GermanCompoundSplitter`, statische mitgelieferte Ressourcen); Zerlegungsterme mit reduziertem Gewicht; Felder (Titel, Überschrift, Text) für BM25F |
| Invertierter Index | Posting-Listen (Delta-VByte) mit Block-Max-WAND je Partition; **kein persistiertes globales Wörterbuch**: Term→ID je Partition und Generation, versiegelt wie Vektoren |
| Statistik | `N`, `avgdl` je Partition/Generation; `df` zur **Abfragezeit** über die abgefragten Partitionen summiert (ganzzahlig, feste Reihenfolge ⇒ bitgleich); kein `commit_lock` |
| Seltene Komposita (AT-7) | Neuaufbau erzeugt das Wörterbuch ausschließlich aus lebenden Dokumenten; ein seltener Term verschwindet mit dem letzten Träger. Zwischen Gate und Rebuild ist er nur per Maske unerreichbar; `stats_drift` wird gemessen |
| Löschen | Gate ⇒ Dokument-Bitmap (RAM) ⇒ Tombstone-Filter vor Scoring; physisch Partitions-Neuaufbau |
| Persistenz | kanonischer Codec statt `bincode`, kein unkeyed `blake3`, kein `block_on`; `TextPurger` = synchroner Wartungsjob |
| Determinismus | Score absteigend, `DocId` aufsteigend (`INV-DETERMINISM`); `INV-TEXT-WAND-1` (WAND ≡ Brute-Force) |

### 11.8 Wissensgraph (`contextra-graph`)

**Zwei Graphen, strikt getrennt (L15).** (1) Lösch-DAG (`DerivedFrom`, §8.8) gehört dem Kern: ≤ 8 Quellen je Cluster, Hülle ≤ 4096, ausschließlich Löschkaskade. (2) Wissensgraph (Entitäten, Relationen, n-äre Hyperkanten) gehört `contextra-graph`; Kanten sind Fakten, keine Löschabhängigkeiten.

| Aspekt | Vertrag |
|---|---|
| Provenienz | Jede Kante/Hyperkante trägt `(source_cluster, source_doc, version)`; ohne Provenienz unzulässig (`INV-GRAPH-PROV-1`) |
| Besitz | Eine Kante gehört dem Segment der Packgruppe ihres Provenienz-Clusters. Hyperkanten mit Belegen aus mehreren Löschdomänen existieren **je Domäne als eigene Kopie** (kein geteilter Besitz, NZ-11); die Abfrage dedupliziert zur Laufzeit über den kanonischen Schlüssel |
| Personen-Entitäten | Knoten vom Typ `Person` gehören dem Personencluster; das gemeinsame Segment enthält nur `EntityRef(pseudonym)`. Kanten zwischen gemeinsamen und Personen-Entitäten sind nicht kaskadierende Referenzen |
| Struktur | CSR-Basis (unveränderlich) + Delta-Puffer; Traversierung = Vereinigung der Adjazenzen über die Segmente des Mandanten (`max_graph_segments_per_tenant` 64, Knoten→Segment-Bloom im RAM); `MAX_TRAVERSAL_HOPS` 16, Fan-out-Limit 10 000 |
| Löschen | Gate ⇒ Maske über `source_cluster` (RAM) vor PPR/Traversierung; physisch Segment-Neuaufbau ohne Kanten der gelöschten Provenienz, danach Entitäts-Sweep (verwaiste Knoten entfernen, Puffer `Zeroize`) |
| Ersetzen | `Supersedes`/Recency-Prior ist kein Löschen (`INV-DECAY-1`) |
| Algorithmen | PPR (Forward-Push), APPRH, TL-HFD, PathRAG, Community/Percolation auf gepinntem Segment-Set; deterministisch (`INV-GRAPH-004`) |

### 11.9 Trennung personenbezogener Daten, Pseudonym-Tresor, Egress

**Ziel.** Ein Cluster-Löschauftrag entfernt alle Namensbezüge einer Person, ohne den Wissenskorpus zu zerstören.

| Klasse | Ablage | Bemerkung |
|---|---|---|
| `NonPersonal` (Produktdoku, Richtlinien, Fachwissen) | Korpus-Cluster des Mandanten (`Plain` oder `Sealed`), Packgruppe `corpus` | normale Indizes (`Corpus`) |
| `Personal` (Akten, Mails, HR, Kundendaten mit Klarnamen) | Personen-/Dokumentcluster, `Sealed`, `DataClass::Personal`-Space | Wurzelcluster + Links (§11.13) |
| `Mixed` | Routing durch die Ingest-Pipeline (`PiiClassifier`, Richtlinie aus `TenantPolicy`) | Fehlklassifikation ist das Kernrisiko |

**Mixed-Routing (FIX-20).** Je Chunk: (a) `NonPersonal` ⇒ Korpus; (b) `Personal` ⇒ ganzer Chunk in den Personen-/Dokumentcluster, zusätzlich ein pseudonymisierter Text für `Pseudo`-Partitionen; (c) gemischter Chunk ⇒ Namensbezüge durch Platzhalter ersetzen, pseudonymisierte Fassung in den Korpus, Original in den Personencluster; (d) **Unsicherheit ⇒ Quarantäne**: Chunk liegt in einem Sealed-Quarantänecluster des Mandanten (Klasse `Personal`, nur Modus A/C, in keiner geteilten Partition) bis zur Freigabe; Freigabe ist ein Routing-Change, kein Kernmechanismus.

**Pseudonym-Tresor.** Der Platzhalter (`[PERSON:7f3a…]`) ist je Person zufällig, nie aus dem Klartext abgeleitet (`INV-SEP-1`). Zuordnung Platzhalter → Klartext liegt als Records im Sealed-Cluster der Person. Embeddings, BM25-Terme und Graph-Entitäten des Korpus entstehen aus dem pseudonymisierten Text. Rückübersetzung `Vault::resolve(principal, placeholders)` erfolgt erst beim Zusammenbau der Antwort, nur unter Policy, nur lokal. Löschen der Person gated und vernichtet ihren Cluster und damit den Tresor.

**Platzhalter-Politik (FIX-39).** Zwei Parameter in `TenantPolicy`, weil Verknüpfbarkeit und Entitätskohärenz gegeneinander stehen:

| Parameter | Werte | Abwägung |
|---|---|---|
| `pseudonym_scope` | `PerPerson` (Standard) · `PerDocument` | `PerPerson` erhält Entitäts-/Graphkohärenz über Dokumente, macht Chunks über den Platzhalter verknüpfbar; `PerDocument` senkt Verknüpfbarkeit, zerstört Personenbezug im Graphen |
| `placeholder_retention` | `Keep` (Standard, Profil `Standard`) · `Scrub` (Pflicht in `Strict`) | `Keep`: Platzhalter bleiben nach Tresorvernichtung im Korpus, Receipt führt `PseudonymousPlaceholdersInCorpus{count}` als Restrisiko. `Scrub`: Folgejob ersetzt den Platzhalter in allen Korpus-Chunks durch `[PERSON]` (normale Changes, Kosten ∝ Chunks mit Refcount) und baut die betroffenen Partitionen neu |

**Egress (AT-8, Z-21).** `EgressPolicy::check(items, target) -> Verdict{Allow | AllowPseudonymized | Deny}` ist Pflicht auf jedem Pfad zu externen Modellen (`INV-EGRESS-1`). Standard `DenyExternal` für `Personal`; `NonPersonal` mit Trust-Label ≥ `tool_verified` darf extern, sofern die Policy es erlaubt. Der Kontext zu externen Zielen trägt ausschließlich Platzhalter; die Rückübersetzung findet lokal nach der Modellantwort statt. Prompt-Injection-Filter liegen beim Client/Plugin (AT-8); Contextra liefert Trust-Labels (§11.12).

**Ehrliche Grenzen.** (1) Pseudonymisierte Daten bleiben personenbezogen, solange die Zuordnung existiert; nach Tresorvernichtung entscheidet die Re-Identifizierbarkeit über Kontext (Funktion, Datum, Ort, seltene Details) — Einzelfallbewertung, kein Kernversprechen. (2) Die Qualität hängt an `PiiClassifier`; der Kern erzwingt nur, dass `Personal`-Spaces ausschließlich `Sealed`-Cluster aufnehmen (`INV-SEAL-1`). (3) `pii_residual_rate` wird je Release gemessen (H-34) und berichtet; sie ist kein Beleg für Fehlerfreiheit. (4) `Pseudo`-Partitionen sind pseudonymisiert-personenbezogen; ihre Löschung kostet Rebuild (H-36), der Receipt-Status bleibt bis Generationswechsel und Schlüsselvernichtung `Incomplete`.

**Backup/Restore.** Der Tresor liegt im Cluster; Restore wendet Ledger-Intents zuerst an (§9.3), der Tresor wird nicht wiederbelebt. Alte Partitionsgenerationen sind nach Schlüsselvernichtung unlesbar.

### 11.10 Query-Plan, Fusion, `SnapshotSet`

| Schritt | Vorgang |
|---|---|
| QP-1 | `TenantPolicy` und Zugriffskontext prüfen (NZ-16); Anfrage normalisieren (NaN/Inf, Dimensionen, `k` ≤ Limit) |
| QP-2 | `view_durable()` ⇒ `V`; **`SnapshotSet`** bilden: `(instance_id, V)` plus je Kanal/Partition `(partition, generation, applied_stamp)` |
| QP-3 | Read-your-writes: bei `min_stamp` bis `ryw_wait` (50 ms) auf `watermark ≥ min_stamp` warten, sonst Delta-Pfad; unbedeckte Lücke ohne Delta ⇒ `IndexLag` |
| QP-4 | Kanäle **parallel**, je mit Gate-/Mandantenmaske vor der Exploration: Vektor (Basis ∪ Delta), BM25/WAND, Graph |
| QP-5 | Kandidaten sammeln, `View::filter_visible` (Gate, Restriction, `DocHead`) |
| QP-6 | Fusion in `contextra-rank`: RRF/mRRF (`k = 60`), Mehrsignal-Fusion, Kalibrierung (Isotonic/Platt/Conformal); Gleichstand: Score, dann stabile ID-Ordnung (`INV-RRF-1`) |
| QP-7 | Budget (§11.11): `page()`/`DiBud` wählen anhand `weights[W]` ohne Körper-I/O für Auslassungen; Körper erst für gewählte Treffer, Gate-Prüfung vor **und** nach dem Entsiegeln |
| QP-8 | **Letzte Prüfung vor Ausgabe:** Gate-Generation jedes Treffers (`INV-VIS-1`); Trust-Label und Lineage anfügen; ggf. `EgressPolicy` |
| QP-9 | Antwort trägt `SnapshotSet`, Kanalbeiträge, `explain`-Daten (nur IDs/Scores, nie Inhalte gelöschter Records; nicht länger gecacht als die Sicht lebt) |

**Determinismus (`INV-RAG-4`).** Gleiche Anfrage + gleiches `SnapshotSet` ⇒ bitgleiches Ergebnis, unabhängig von Threadzahl und Hash-Reihenfolge. Grundlage für Differentialtests (`Index ∪ Delta ≡ Vollscan`) und Audit.

**Zeitreise (FIX-15).** `Query::at(S)` ist nur möglich, wenn (a) ein `SnapshotSet` benannt ist (Generationen gepinnt, lease-begrenzt ≤ 24 h) oder (b) für jede beteiligte Partition eine retained Generation mit `applied_stamp ≤ S` existiert; sonst Kern-Vollscan-Pfad nur für `Corpus`-Partitionen unter `scan_fallback_max_vectors`, sonst `Err(IndexHistoryUnavailable)`. Ein Ergebnis enthält nie Daten gegateter Cluster (Erase überstimmt Zeitreise, `INV-ERA-1`); nach Generationswechsel ist die Zeit vor einer Löschung für die gelöschten Quellen unwiederbringlich. Das Ergebnis weist `time_travel_degraded` aus, wenn Quellen fehlen.

**Lernsignale.** Nutzerfeedback und Kantenverstärkung liegen im Cluster des Nutzers; globale Kalibratoren trainieren nur auf aggregierten Scores ohne Nutzer-/Query-Bezug und erst ab `k_min` (50) Beobachtungen je Gruppe. Kalibratoren persistiert `engine` in einem `NonPersonal`-System-Space.

**Kosten (H-30, H-31).** Latenz = Maximum der Kanäle + Fusion. Partitionszahl ≤ `tenant × aktive due_buckets × shards`, reduziert durch Mandanten-, Zeit-, Quellenprädikate (`BlockPruner` nur für schattensichere Besitzer).

### 11.11 Kopf, Gewichte, Tokenbudget (`DiBud`)

`header`, `weights[W]`, `page`, `weight_sum` bilden die Schnittstelle der Budgetlogik (§3.5). Die obere Schicht legt fest, was ein Gewicht bedeutet: je Tokenizer-Familie ein Slot (≤ 4). Der Kern liest nie Körper für Budgetentscheidungen.

**`DiBud`-Auswahl (`contextra-rank`, deterministisch, zustandslos):**

1. Kandidaten aus QP-6 mit Zoomstufen (L0 Körper, L1/L2 Kopf, §11.12) und Gewichten je Stufe.
2. Je Kandidat die höchste Stufe, deren Gewicht in das Restbudget passt (Absenken bei Knappheit).
3. Greedy nach Nutzendichte `score / weight` mit Redundanzabzug (Ähnlichkeit zu bereits gewählten Items, z. B. MMR); Gleichstand: stabile ID-Ordnung.
4. Ausgabereihenfolge `OrderPolicy::{ByScore (Standard) | PrefixStable}`; `PrefixStable` ordnet nach `(Klasse, DocId)` für prefix-cache-freundliche Prompts (Cache selbst liegt außerhalb, NZ-18).
5. Ergebnis `ContextPack{items, tokens_used[W], omitted, snapshot_set, trust_labels, lineage}`.

**Pseudonym-Auflösung (FIX-16).** Gewichte beschreiben den **pseudonymisierten** Text. Die Rückübersetzung (§11.9) ändert die Tokenzahl. `DiBud` reserviert `restore_slack_fraction` (5 %) des Budgets; nach der Auflösung wird die Endlänge geprüft, bei Überschreitung werden die Items niedrigster Nutzendichte entfernt (deterministisch) und das Ergebnis trägt `trimmed_after_restore`. Für externe Ziele (nur Platzhalter, §11.9) entfällt die Reserve.

**Paging.** `page` überspringt Body-Blöcke ohne Entsiegeln über `weight_sum`; RAM-Präfixsummen sind Obergrenzen, nie Korrektheitsquelle (§6). Fortschrittsgarantie: ein Item, das das Budget sprengt, beendet die Seite, außer sie wäre leer.

### 11.12 Konventionen der oberen Schicht (keine Kernfunktionen)

| Thema | Konvention |
|---|---|
| Zoom-Stufen | Schlüsselsuffix je Stufe im selben Cluster (`doc/<id>/L0`, `/L1`, `/L2`); L1/L2 im Kopf, L0 im Körper |
| Trust-Label | erstes Kopfbyte `trust ∈ {system, user, tool_verified, tool_unverified, untrusted}`; Abgeleitetes erbt das Minimum seiner `DerivedFrom`-Quellen; Hochstufen nur privilegiert mit Audit |
| Lineage | abgeleitete Cluster per `link(derived, sources)`; Kopf trägt `(producer_id, source_stamp)`; veraltet = `source_stamp` ≠ aktueller Quellstamp |
| Derivation Cache | Memoisierung von Embeddings/Token-IDs: Schlüssel `MAC(cache_secret, model_id ‖ canonical_input)`; `cache_secret` = zufälliges 32-B-Secret **je Cluster** aus `Entropy`, als Record im selben `Sealed`-Cluster, nie aus Inhalt/Mandantenschlüssel abgeleitet, nie exportiert. Einträge sind Records desselben Clusters; kein Treffer über Cluster-/Mandantengrenzen, keine Dedup; `Plain`-Cluster erhalten keinen Cache (`INV-PRIV-1`) |
| Handles | kurze IDs (4–6 Zeichen) sind Schlüsselaliase innerhalb des Clusters (Alias-Space); Branches teilen den Alias-Space, Merge löst Alias-Konflikte als Schlüsselkonflikte |
| Bitemporalität | `valid_from/valid_to/observed_at/source_time` im Kopf; `supersedes` als Schlüssel `edge/supersedes/<id>` |
| Embeddings | Space `emb/<model_id>` (§11.6) |
| Work Queue / Sleep Cycles | Consumer + Tasks als Daten im Cluster; Ergebnis = normaler `Change` mit Producer im Kopf |
| Vergessen | Löschauftrag der oberen Schicht; keine stille Entfernung ohne `Eraser` |
| **Session ≠ Cluster** | Cluster folgt der **Löschdomäne** (Person, Fall, Arbeitskontext). Sessions sind Schlüsselpräfix (`sess/<id>/…`) oder TTL-Branch im Personencluster; Scratch liegt in `Volatile`-Spaces. Cluster je Sitzung nur bei eigener Löschdomäne (z. B. Gastsitzung mit eigenem Aufbewahrungsende). 10⁷ Mikro-Cluster kosten 40 B RAM, ein Siegel je Abschnitt und eine Directory-Zeile; die Fassade meldet `ClusterMetaBudget`-Druck früh mit `hint()` |

### 11.13 Person → Cluster (normatives Muster)

1. **Wurzelcluster je Person** (`Sealed`, `Personal`-Space): enthält die Liste der Dokumentcluster (`docs_index`). **Dokumentcluster sind die Löschgranularität** (ein Dokument/eine Mail/ein Fall-Teil = ein Cluster, Eraser `Cryptographic`), hängen per `link(derived, [wurzel])` an der Wurzel; Löschung der Wurzel zieht alle Nachfahren im selben Gate-Frame mit (`INV-ERA-12`). Teilbereichs-Löschung innerhalb eines Clusters ist für `Sealed` nur `Logical` (§8.10); wer physische Entfernung eines Teils braucht, legt diesen Teil von Anfang an in einen eigenen Cluster.
2. **`IdentityDirectoryOwner`** (registrierter `SegmentOwner`, Klasse `MixedPerSubjectKey`). Die Zuordnung `person_id → Wurzel-ClusterId` liegt nie in einem `Plain`-Space. Ablauf (FIX-23; R13 hatte einen Zirkelschluss, weil die Entry-Key-ID aus der ClusterId abgeleitet werden sollte, die erst durch die Entschlüsselung bekannt wird):

   | Element | Inhalt |
   |---|---|
   | Vorwärtszeile | Schlüssel `dir_key = keyed_mac(DirectoryKey, tenant_slot, canonical_person_id)`; Wert `{entry_tag[16] (zufällig, Klartext), AEAD_{entry_key(entry_tag)}(ClusterId)}` |
   | Rückwärtszeile | Schlüssel `rev_key = keyed_mac(ReverseKey, tenant_slot, ClusterId)`; Wert `{dir_key, entry_tag}` unter System-DEK |
   | Schlüssel | `K_dir`/`K_rev` verlassen den Provider nie (`keyed_mac`); `entry_key(entry_tag)` je Person über `create_entry_key`/`destroy_entry_key` (§8.4) |

   Personenlöschung (ein Orchestrierungsauftrag): (a) Root-Gate samt Nachfahren, (b) `destroy_entry_key(entry_tag)` + Entfernen beider Zeilen (`rewrite_without`), (c) alle Atteste. Kein systemweites `Complete`, solange ein Owner-Attest fehlt.
3. **Restrisiko Backup:** Backups enthalten höchstens `dir_key`/`rev_key`-Pseudonyme und nicht entschlüsselbaren Entry-Ciphertext. Das stabile `dir_key`-Label ist weiterhin pseudonyme Information: solange ein Backup es enthält und `K_dir` für Zuordnungsversuche verfügbar bleibt, ist der Status höchstens `CompleteWithResidual` (Backup-Frist/Linkability). Striktes `Complete` verlangt Entfernen aus allen retained Backups oder Rotation/Vernichtung der Directory-Key-Epoche samt Migration aller Einträge (O(Tenant), expliziter oberer Workflow).
4. **Fristaufträge:** §11.14 Nr. 3.
5. **Keine Zuordnungstabelle außerhalb des Löschvertrags (`INV-DIR-1`):** Jeder persistente Kennung→Cluster-Pfad ist als `IdentityDirectoryOwner` mit Per-Subject-Key/Lineage registriert oder erfüllt gleichwertig Gate/Destroy/Rewrite/Verify; unregistrierte Mappings verhindern systemweites `Complete` (`ext-conformance`).
6. **Mehrbetroffene Daten:** gemeinsamer Fallcluster; jede Person hat ihren Wurzelcluster; Link nur, wo die Löschung der Person den Fallcluster mitziehen soll; sonst nicht kaskadierende Referenz. Cross-Subject-`DerivedFrom` ist unzulässig (die Identity-/Policy-Schicht prüft vor dem Link-Commit).

### 11.14 Zeit, Retention, Branches, Backup, Abgrenzung zu Agenten

Contextra liefert Daten und Kontext, keine Agentenlogik.

| Nr. | Konzept | Zuordnung | Regel |
|---|---|---|---|
| 1 | Gedächtnisdaten von Clients | Records im Cluster des Eigentümers (§11.12) | Löschung des Eigentümers nimmt alles mit; Inhalt wird nicht interpretiert |
| 2 | Experimente, Was-wäre-wenn, Re-Indexierung | clusterlokale Branches (§7) | O(1), atomar, löschbar; Merge-Auflösung liefert der Client; Merge ≤ `max_merge_ops` |
| 3 | **Aufbewahrung (Retention)** | `engine::compliance`: Richtlinie je Tenant/Datenklasse | siehe unten |
| 4 | Reproduzierbare Abfrage | benanntes `SnapshotSet` + `EpochPinRegistry` | lease-begrenzt (≤ 24 h); Erase widerruft (FIX-08) |
| 5 | DB-Backup/Restore | `storage::checkpoint` (§9.3), koordiniert von `contextra-checkpoint` über Kern **und** Index-Generationen | enthält weder Schlüssel noch Ledger; trägt `BackupMark`; Restore belebt Gelöschtes nie wieder |
| 6 | Relevanz über die Zeit | Recency-Prior: zustandslose Funktion der Gültigkeitszeit im Kopf; `Supersedes` im Kopf | verändert nie Daten, löscht nie (`INV-DECAY-1`) |
| 7 | Konsolidierung/Zusammenfassung | Aufgabe des Clients; Ergebnis ist ein `Change` mit Producer im Kopf | erbt das Minimum des Trust-Labels; `DerivedFrom`, wenn Quelllöschung mitziehen soll |
| 8 | **Nicht gebaut** | KV-/Prompt-Cache, Agenten-Orchestrierung, Session-DAG, kognitiver Decay, Modell-Routing, Modell-Hosting | NZ-18, NZ-21…23; Provider-Caches stehen als Restexposition im Receipt (§8.11) |

**Retention (Nr. 3, FIX-34).** Retention ist die einzige zeitgesteuerte Löschung (`INV-DECAY-1`). Klärung der Semantik von `due` gegenüber `due_bucket`:

- `due` ist der Zeitpunkt, zu dem der Löschauftrag entsteht. `engine::compliance` löst zu `due` (oder spätestens im nächsten Worker-Lauf) `Eraser::clusters_batch` je ≤ 4096 Cluster aus; das **Gate liegt damit am `due`**, die Frist `purge_deadline` läuft ab `due`.
- `due_bucket` (Monat/Quartal/Jahr) steuert nur die **physische Ko-Lokation**: Cluster ähnlicher Ablaufzeit liegen in denselben Dateien, sodass nach Ablauf der Gruppe ganze Dateien ohne Survivor-Rewrite entfallen. Kein Cluster wird später gegated, weil sein Bucket noch offen ist.
- `pack_expired(PackKey)` ist die Sammelform nach Bucket-Ende; sie ist idempotent zu bereits ausgelösten Einzelaufträgen (§8.10).
- Verlängerung der Aufbewahrung nur über `Restriction{BLOCK_ERASE, RetentionExtension}` (FIX-22). Bei `ClockUncertain` stoppt Retention fail-closed.
- Standardskew 5 min; strengere Werte nur explizit.

**Historie (FIX-31).** `history_floor` folgt `HistoryPolicy` (§3.6). Zeitreise-Lesen (`view_at`) ist nur oberhalb `history_floor` möglich; ohne Pins entspricht der Boden `KeepLatest`. Wer Zeitreise-Audit für Korpusdaten wünscht, wählt `KeepUntilFloor` mit `history_retention`; für `Personal` ist die Historie auf Pins/Branches und `history_retention` begrenzt, ein Löschen/Korrigieren einzelner Records lässt die Vorversion bis zum Boden plus Compaction physisch bestehen (Disclosure im Receipt von `Logical`-Range-Aufträgen).

### 11.15 Produktfunktionen, Compliance-Profile, Auskunft/Export, Fehlerhinweise

| Funktion | Grundlage | Kundennutzen |
|---|---|---|
| Nachweisbare Löschung mit Receipt, Frist, Restexposition | §8 | Rechenschaft (Art. 5 Abs. 2, Art. 17 DSGVO) ohne Handarbeit |
| Zeitreise-Retrieval | `view_at`, `SnapshotSet` (§11.10) | reproduzierbare Antworten, Audit |
| Agenten-Experimente ohne Datenleck | Branches | risikofrei, vollständig löschbar |
| Tokenbudget-Retrieval | `page`/`weights`/`DiBud` | weniger Kontextkosten und Latenz |
| Auskunft/Export je Person/Fall | `export_cluster` | Art. 15/20 in einem Zug |
| Einschränkung der Verarbeitung | `Restriction` (`BLOCK_READ/WRITE/ERASE`) | Art. 18, gesetzliche Aufbewahrung |
| Löschfristen je Packgruppe | `pack_expired`, Retention | Aufbewahrungsende ohne Einzelaufträge |
| Datentrennung + Egress | §11.9 | Cloud-LLMs ohne Klarnamen |

**Compliance-Profile (FIX-24).** Es gibt **ein** Profil `ComplianceProfile::{Development, Standard, Strict}` (Standard bei persistentem Open: `Standard`); das R13-Paar `OpenProfile`/`ComplianceProfile` entfällt. `open` verweigert widersprechende Kombinationen (`ProfileViolation`).

| Wert | `Development` | `Standard` | `Strict` |
|---|---|---|---|
| Schlüsselzusicherung | beliebig | `Kms` oder `Hardware` | `Hardware` |
| `KeysInBackups` | beliebig | `No` oder `Bounded` | `No` |
| Anker | `Weak` erlaubt | `Required`; `Weak` nur mit `acknowledged_weak` | `Strong` |
| Ledger-Gerät | gleich erlaubt | getrennt; `ledger_same_device_ack` erlaubt | getrennt, andere Failure Domain; kein Ack |
| `Personal` | Warnung | `Sealed` erzwungen | `Sealed` + Provider-Attest |
| Acknowledgments (`acknowledged_weak_keys`, `…_weak`, `…_same_device_ack`) | alle | erlaubt, erscheinen im Receipt als Statuscap/Restexposition | **verboten** |
| `placeholder_retention` | beliebig | `Keep` oder `Scrub` | `Scrub` |
| Pflichtmessung nicht `Complete` | informativ | `Incomplete` | `Incomplete` + Alarm |
| `ryw_wait`/`IndexLag` | tolerant | Standard | strenger Alarm |

Ein Profil ist kein Rechtsurteil.

**Auskunft/Export.** `Storage::export_cluster(view, cluster)` liefert einen Strom `(branch, key, header, body)` über eine konsistente Sicht, ohne Schlüsselmaterial und Indexstrukturen, mit Gate-Prüfung vor jedem Block; `Erased` ⇒ `Err(Erased)`. Der Export hält die View-Lease (Fortsetzung über `ResumeToken` + `view_at` + `Hold`), ist rate-begrenzt (`export_io_fraction` 0,2). Formatierung, Prüfung Rechte Dritter und Übermittlung liegen oben.

**Fehlerhinweise.** Jeder Fehlertyp liefert `Error::hint()` (stabil, ohne Schlüssel/Inhalte/Queries) und in der Fassade `suggested_call`.

---

## 12. Verifikation

### 12.1 Ebenen

| Ebene | Werkzeug | Inhalt |
|---|---|---|
| Formal | TLA+/TLC (oder Quint) | **M1** Commit-Pipeline (Intake, Sequencer, Syncer, Publikation, `Retiring → Retired`), Sichtbarkeit, Recovery, Stamps, Guards (Konfliktfenster, Fensterfuß) · **M2** Erase (ER0–ER7, Gate, Intent-Erweiterung/`link_fence`, Restriction, Link, Consumer-Floors, Space-Drop, Branch-Ops, Merge) · **M3** Compaction × Erase × Claims × Staging × Fork-Floor · **M4** Checkpoint × Erase × Restore · **M5** Rebucket · **M6** Anker × Edit × Rollback |
| Code-Modell | `loom` | Sperren, Version-Swap, Generationszähler, Claims, Graveyard, Leader-Wechsel, Pipeline-Credits, View-Widerruf |
| Simulation | eigener Scheduler + `shuttle` | **Hauptwerkzeug gegen Races**: seeded Interleavings, `FaultFs`, `SimClock`, `SimEntropy`, `SimKms`, reproduzierbarer Seed; RACE-1…RACE-25 als benannte Szenarien |
| Absturz | `FaultFs` | Crash an **jedem** E/A-Index, Torn Writes, verlorene Fsyncs, Rename-Reihenfolgen, Rollback-Versuche, `ENOSPC` beim Vorallokieren, Geister-Journal aller geschriebenen Seiten |
| Eigenschaft | `proptest` | Codecs, `cmp`, Siegel-Roundtrip und Rohkopie, Compaction ≡ Referenzmodell (`BTreeMap` mit Versionen/Branches), Purge ≡ Modell ohne Cluster, Paging (Budget nie überschritten, lückenlos, vorwärts/rückwärts gleiche Menge, Block-Skipping ≡ Vollscan), Branch-Lesen ≡ Modell, Merge ≡ Modell |
| Lokalität | Byte-Zähler | `ClusterLocal` und `PackedBounded` separat bei Store ×1/×10/×100; Run/Blob, gemeinsames Redo, Checkpoints, Extensions getrennt zählen; keine pauschale O(Cluster)-Behauptung |
| Mutation | `cargo-mutants` | `codec`, `recovery`, `erase`, `sequencer`, `syncer`, `branches` |
| Linearisierbarkeit | Wing-Gong | Historien aus der Simulation (inkl. Guards, Merge) |
| Löschorakel | `ErasureOracle` | §12.2 |
| Provider-Konformanz | `KeyProviderConformance` | jeder `KeyProvider` (inkl. Entry-Schlüssel, `keyed_mac`) |
| Fuzz | `cargo-fuzz`, `miri` | Decoder aller Formate, Recovery auf beschädigten Verzeichnissen (≥ 8 Ziele); Bit-Flip-Differenztest: `Corrupt`/Auth-Fehler, nie stille Falschdaten; `miri` auf `codec`/Siegel |
| Statisch | clippy, `cargo-deny`, `layering`, `unsafe_islands`, `cargo-geiger`, `check_forbidden_apis.sh` | §2.6, §2.7 |
| Dauer | Soak 72 h, Crash-Loop, Platte voll | |
| Retrieval | Differentialtests, Sim, Orakel | `Index ∪ Delta ≡ Vollscan` (Text/Graph bitgleich, Vektor-Recall ≥ Basis) · Race Gate × Query und Gate × Rebuild · Crash-Sweep über IG-1…IG-6 · `SnapshotSet`-Determinismus (`INV-RAG-4`) · Canary-Personen mit Pseudonym-Tresor (`pii_residual_rate`, `INV-SEP-1`) · Partition-Generationswechsel mit Schlüsselvernichtung · Modus-C-Löschung · Directory-Löschung (`INV-DIR-1`) · Zeitreise nach Löschung (FIX-15) |

### 12.2 Löschorakel

Jeder Test-Cluster erhält eindeutige Canaries in Schlüssel, Kopf, Körper, `weights`-Kodierung, Idem-Token, Branch-Namen, Platzhalter und Schlüsselmaterial (Test-Provider legt seine Schlüssel offen); zusätzlich 16-Byte-Stichproben aus jedem Siegel.

| Orakel | Durchsucht | Erwartung |
|---|---|---|
| O-Files | Catalog-lebende Runs, Blob-Inkarnationen, Redo einschließlich `retain_floor`-Segmente, `.building`, registrierte purgepflichtige Segmente; getrennt Checkpoints/Cache-Dumps/externe Snapshots | nach `Done` keine Nutzdaten von H in lebenden Dateien; registrierte Kopien attestiert entfernt; nicht verwaltete Kopien bleiben Receipt-Restrisiko |
| O-Device | Geister-Journal | keine Klartext-Canaries, keine Schlüssel; Ciphertext-Treffer erlaubt und als G1-Schutz ausgewiesen |
| O-Gate | alle `view/page/poll`, alte Snapshots, Cursor, Retries, Extension-Antworten ab Gate | kein Inhalt |
| O-Ledger | Ledger-Bytes | nur `tomb_id`, keine ClusterId/Payload |
| O-Segments | `verify_absent` | für jedes registrierte Segment `true` |
| O-Restore | nach `restore` alter Checkpoints | `Erased`-Cluster nie wieder `Live` |
| O-Survivors | Überlebende im selben Packed-Run | bytegleich lesbar (Hash der Sektionen vor/nach) |
| O-Process | `gcore` nach Löschung | best-effort, kein Beweis, so deklariert |

Das Orakel läuft **zwischen** den Phasen (Crash in Phase 2/3, Recovery, Fortsetzung).

### 12.3 Messarten (`MeasureKind`)

`verifier_report_digest` bindet die geordnete Liste `{name, kind, scope_count}`.

| Art | Bedeutung |
|---|---|
| `Complete` | Vollprüfung über den gesamten Scope |
| `Sample{n,N}` | Stichprobe |
| `Declared` | vom Betreiber/Adapter erklärt, nicht prüfbar (O-Process, Swap verschlüsselt) |
| `NotMeasured` | nicht ausgeführt |
| `NotImplemented` | Prüfer fehlt |

Pflichtmessungen sind (a)–(d) von `Eraser::verify` (§8.9) und müssen `Complete` sein, sonst `Incomplete`. Übrige Messungen sind informativ; `Sample`/`Declared` begrenzen auf `CompleteWithResidual`, wo für den Level relevant. Nur `Complete`-Messungen heißen „verifiziert“. Normalfall im Testlauf: O-Files, O-Ledger, O-Segments, O-Gate, O-Restore `Complete`; O-Survivors, O-Device `Sample`; O-Process, Host-Befund, externe Backups `Declared`. CI prüft, dass kein `Complete`-Receipt bei unvollständiger Pflichtmessung entsteht (`INV-VER-1`).

### 12.4 CI

Pflicht: `check`, `fmt`, `clippy -D warnings`, `test`, `layering`, `loom`, `doc`, `deny`, Sim-/Absturz-Kurzlauf, INV-Zitierprüfung (jede ID aus §14 in mindestens einem Test per `// INV-…`), Abdeckung. Nachtlauf: lange Sim, Fuzz, Orakel-Vollsuche, Mutation, Provider-Konformanz. **Ohne funktionierende CI ist kein Gate gültig.**

---

## 13. Phasen, Gates, Altpfade

### 13.1 Phasen

Planungsannahme: 3–4 erfahrene Rust-Entwickler plus 1 Test-/Simulationsingenieur; Dauern sind grobe Schätzungen (für P2, P6, P7 Faktor 1,5–2 Risiko, §17), keine Zusage. Keine Phase beginnt vor dem Gate der vorherigen; Formatänderungen nach G-F nur per Migration; der Crash-Sweep wird mit jeder Phase um neue `Fs`-Operationen **erweitert, nie nachgezogen**.

| Phase | Inhalt | Gate | ca. |
|---|---|---|---|
| **P0-A Baubarkeit** | (1) `contextra-kvcache` aus `[workspace].members` streichen; (2) `contextra-storage/Cargo.toml`: Abhängigkeiten `ports`, `types`; (3) `contextra_storage_ports::…` → `contextra_ports::kernel::…` (134 Zeilen, mechanisch); (4) `engine`: `contextra_store`-Importe hinter Feature `legacy-store`; (5) `cargo check -p <crate> --locked` je Crate | **G-0a:** jedes der 14 Crates baut einzeln; `cargo metadata` fehlerfrei | 1 Wo. |
| **P0-B Fundament** | `ports::kernel` (+ Konformanz), `ports::legacy`-Abtrennung; `durable-fs` → `sys` + positionsbasierte Handles; `engine::platform`; `crypto` auf XChaCha20/HMAC/HKDF/CRC32C, Löschung `kv_*`; `types`: `Epoch`, `Stamp`, `ClusterId`, `SpaceId`, `Error`; `capabilities.toml`, `CLAUDE.md`, `deny.toml`; `layering`/`unsafe_islands`; Orakel-Gerüst; Spikes H-21, H-11, H-25, H-26, H-37 | **G-0:** Konformanz `Fs`/`Cipher`/`KeyProvider`/`Anchor` grün, `INV-TOPO-1`, `INV-LAYER-1` als Test, Spec-Review durch zweiten Reviewer | 5 Wo. |
| **P1 Walking Skeleton** | ein Space, `apply`, Redo + Memtable, `view`, Cluster-Register, Gate, `destroy_many`, Ledger, Recovery; Formate provisorisch | Crash-Sweep über alle `Fs`-Operationen; `O-Files` findet nach `Done` nichts | 4 Wo. |
| **P2 Runs und Catalog** | Flush, ClusterSection, Staging, Catalog + `CURRENT`, Purge, Plain-Barriere, Recovery, Absturzmatrix | `O-Files` für Runs grün; Modelltest; M1, M2 (ohne Branches) | 5 Wo. |
| **G-F Formatfreeze** | alle Formate (§4.9) inkl. `branch`-Feld, Partitionsformat und `PartitionGeneration`-Schlüsselscope, goldene Vektoren (Anhang C), Rohkopie-Test, ≥ 8 Fuzz-Ziele | Freeze-Review, Referenzcodierer vollständig | — |
| **P3 Schreibpfad** | Gruppen-Commit, Reservierungen, Guards zweistufig, Volatile, `DiskFull`, Idempotenz, Backpressure | **G-W:** Absturzmatrix, Linearisierbarkeit in Sim, H-1, H-18, H-24 | 4 Wo. |
| **P4 Lesepfad** | Blobs, Cache, Views (Revoke, Lease), `page` mit Gewichten, Block-Skipping, `ResumeToken`, `filter_visible` (`DocHead`) | **G-R:** Modellgleichheit, Paging-Properties, H-2…H-4, H-25 | 4 Wo. |
| **P5 Wartung** | Compaction je Wartungsgruppe, Claims, Blob-GC, Governor, Rotation, Consumer-Feed | **G-M:** M3, Sim ohne Invariantenbruch, H-5, H-19, H-26…H-28 | 5 Wo. |
| **P6 Löschung komplett** | Links, Restrictions, Abwurf, Space-Drop, `range`, Ledger/Anker, Checkpoint/Restore, `FilePassphrase`, `SegmentOwner`/`ExtensionBlock`, Receipt + `verify_receipt`, Tomb-Epochen, Löschauftrag auf `Poisoned`, Intent-Erweiterung/`link_fence`, volle Orakel | **G-E:** M2/M4/M5/M6, Orakel 0 Treffer, Krypto-Review | 6 Wo. |
| **P7 Versionierung** | `BRANCHES`-Bit, `SYS_BRANCH_S/P`, Fork/Drop/TTL, Branch-Lesen, Diff, Merge (`max_merge_ops`), `Eraser::branch`, Fork-Floor | **G-V:** Branch-Properties vs. Modell, M2/M3 mit Branches, H-20…H-22 | 5 Wo. |
| **P8 Härtung** | Soak 72 h, Dauer-Fuzz, Platte voll, Fehlerinjektion, Fassade, Abdeckung, Messbericht, Bedrohungsmodell-Review | **G-P:** alle INV getestet (100 %), H-15/H-16, Messbericht §16 | 5 Wo. |

**Retrieval-Spur** (parallel, an freigegebene Verträge gebunden):

| Schritt | Inhalt | Gate |
|---|---|---|
| **R1 Persistenzneutralität** | `vector`/`text`/`graph`: `PartitionSink`/`PartitionSource` in `contract`-Modulen; `memmap2`, `std::fs`, festen Schlüssel, `bincode`, unkeyed `blake3`, `tokio`, `scc`, `block_on` aus Produktivpfaden entfernen; `TxId` → `Epoch`; Index-Traits auf Lesevertrag; `graph::session_dag`/`edge_reinforcement` löschen | **G-R1:** `check_forbidden_apis.sh` ohne Treffer in L2; jedes Crate baut/testet einzeln |
| **R2 Projektion** | `engine::projection`: `SegmentOwner`/Feed/Watermark/Attest für Vektor, Text, Graph, IdentityDirectory; Partitionen, Generationsschlüssel, `rewrite_without`, `verify_absent`; Modus C; `engine::kernel` (Blocking-Pool) | **G-R2:** Konformanzsuite je Owner; `O-Segments`, `O-Files` ohne Treffer; Spike H-36 |
| **R3 Query-Plan** | `engine::query`: QP-1…QP-9, `SnapshotSet`, Delta, `min_stamp`, Fusion, `DiBud`; `mvcc` → `EpochPinRegistry` | **G-R3:** Differential `Index ∪ Delta ≡ Vollscan`, Race Gate × Query, `INV-RAG-4`, H-29…H-31 |
| **R4 Ingest/Publish** | Publish-Record, `PiiClassifier`-Routing, Pseudonym-Tresor, Quarantäne, Idempotenz je Teil-Change, Orphan-Worker, `EgressPolicy` | **G-R4:** Crash-Sweep IG-1…IG-6, `pii_residual_rate` (H-34), Partial-Ingest nie sichtbar |
| **R5 Checkpoint, Export, Migration** | `contextra-checkpoint` Neuzuschnitt, Retention, `export_cluster`, Forward-Export aus `contextra-store` | **G-R5:** Restore-/Rollback-Properties, Migrations-Rehearsal |

**G-CUTOVER** (nach G-P und G-R1…G-R5): erst wenn alle produktiven `contextra-store`-Verbraucher und Datenräume migriert sind, Restore-/Rollback-Rehearsal einschließlich externem Fencing des alten Instanzbesitzers und RAG-Differentialmessung bestanden und das gemeinsame RSS-Budget (H-35) eingehalten ist. Danach wird `ports::legacy` gelöscht. Kein Dual-Write (Z-22).

### 13.2 Ist-Stand Repository (HEAD `78a1783`, statisch geprüft, nicht kompiliert)

Entwicklungsrechner 1 vCPU/3,9 GB RAM. Umfang je Crate (Dateien/Zeilen): `types` 38/9 852 · `ports` 42/7 641 · `sys` 11/2 649 · `simd` 17/5 827 · `durable-fs` 21/3 244 · `storage` 164/16 824 · `crypto` 74/18 780 · `mvcc` 26/7 964 · `checkpoint` 29/9 930 · `vector` 127/29 113 · `text` 62/16 906 · `graph` 133/32 725 · `rank` 42/8 455 · `engine` 126/29 467.

| # | Befund | Maßnahme |
|---|---|---|
| 1 | `Cargo.toml` listet `crates/contextra-kvcache` (Verzeichnis fehlt); jeder Cargo-Aufruf scheitert | Mitglied streichen (P0-A) |
| 2 | `storage` importiert `contextra_storage_ports` in 134 Zeilen und deklariert `pub use … as ports`; Crate/Abhängigkeit fehlen | Importe auf `contextra_ports::kernel`, Abhängigkeiten eintragen (P0-A) |
| 3 | `engine` importiert `contextra_store` (`LsmStorage`, `DurabilityMode`) in 18 Dateien, ohne Abhängigkeit | hinter `legacy-store`, dann `kernel`/`projection` (P0-A, R2/R3) |
| 4 | `mvcc` und `checkpoint` enthalten Stub-Module `contextra_ports.rs`/`contextra_types.rs` und je eine `TxId`; `checkpoint` führt `WorkflowState` | ein `Epoch(u64)` in `types`; `checkpoint` neu zuschneiden |
| 5 | `ports` enthält Alt-Verträge (`StorageEngine`, `Vfs`, `CommitBarrier`, `Purger`, …) und KV-/Attention-/Inferenz-Ports; Index-Traits tragen `TxId`-Schreibpfade | `legacy`, KV/Inferenz löschen, Index-Traits lesend |
| 6 | `crypto` führt `kv_segment/`, `kv_cipher.rs`, `kv_shredding.rs`, nutzt `aes-gcm-siv`/`blake3` | löschen; XChaCha20/HMAC/CRC32C |
| 7 | `engine` enthält `orchestration/`, `decay_controller.rs`, `collection/kv_lock.rs`, `adapters/ollama.rs` | löschen (NZ-21…23) |
| 8 | `capabilities.toml` beschreibt nicht existente Crates; `CLAUDE.md` verweist auf fehlende Tests/Docs; Cargo-Alias `xtask` ohne Paket | aus §2.1 neu erzeugen (14 Einträge), Tests in Mitglied-Crates, Doku generieren |
| 9 | `graph/purger.rs` verweist auf nicht existentes `contextra-erasure-spi` | Schnittstelle ist `PartitionSink::rewrite_without` |
| 10 | `[workspace.dependencies]` enthält `bincode`, `flatbuffers`, `memmap2`, `tokio full`, `parking_lot`, `crossbeam-skiplist` als Defaults | Verbote je Ebene in `deny.toml` (`wrappers`) |
| 11 | `simd`/`sys` ohne `forbid(unsafe_code)` (korrekt: Unsafe-Inseln); `durable-fs`, `types`, `text` führen es | Test `unsafe_islands` (§2.6) |
| 12 | `durable-fs::Vfs` dateiweise; positionsbasierte Zugriffe, `preallocate`, `lock`, `free_space` fehlen | P0-B |
| 13 | Alt-Muster: `memmap2`, `bincode`, `blake3`, `block_on`, `tokio`, `DISKANN_INTEGRITY_KEY`, `std::fs`, `scc` | §13.3 |
| 14 | Kein vollständiger Workspace-Check gelaufen; Warnungszahl unbekannt | CI je Crate; belastbar erst nach G-0a |

### 13.3 Ablösung der Altpfade (bis G-CUTOVER)

Jede Zeile hat einen Test, der den Altpfad **verbietet** (Lint, `check_forbidden_apis.sh`, `deny.toml`) und einen, der den Ersatz belegt; ein Altpfad wird erst nach bestandenem Ersatz-Gate entfernt.

| Altpfad | Ersatz | ab |
|---|---|---|
| Mitglied `contextra-kvcache` | gestrichen | P0-A |
| Phantom-Import `contextra_storage_ports` | `contextra_ports::kernel` | P0-A |
| `contextra-store` (LSM + WAL), `LsmStorage`, Importe in 18 `engine`-Dateien | `contextra-storage`; Forward-Export/-Ingest (NZ-7) | G-CUTOVER |
| `DbTransaction` + `CommitLedger` + `CompensatingAction` (4-Wege-2PC) | Ein-Wahrheits-Commit + Publish-Record + Feed (§11.4) | R3/R4 |
| `mvcc`: `TxId`, `SequenceLog`, SSI, `TxBuffer` | `Epoch(u64)`, Guards (§5.3); `EpochPinRegistry` bleibt | R3 |
| `aes-gcm-siv`, `blake3` in `crypto`/`vector`/`text`/`graph`/`checkpoint`/`types`/`ports` | XChaCha20-Poly1305/HMAC-SHA256; keine unkeyed Inhalts-Hashes | R1 |
| `memmap2`, `std::fs`, fester Integritätsschlüssel in `vector` | `PartitionSink`; AEAD unter Generationsschlüssel | R1/R2 |
| `commit_lock` + globale `meta:stats`/`pl:` in `text` | partitionsbezogene Statistik, Abfragezeit-`df` | R1/R2 |
| `bincode` in `graph`/`text`/`vector`/`crypto`/`types` | kanonische Codecs; `types` nur hinter `wire-compat` | R1 |
| `TxId` ×5, `text::contract::Epoch`, Stub-Module | ein `Epoch(u64)` | R1 |
| `crypto::{kv_segment, kv_cipher, kv_shredding}`, `ports::{kv, kv_bridge_port, attention}` | gelöscht | P0-B |
| `engine::{orchestration, decay_controller, adapters/ollama}`, `collection/kv_lock.rs` | gelöscht; Retention in `compliance` | R3 |
| `graph::{session_dag, edge_reinforcement*}`, `GraphPurger` | gelöscht bzw. `PartitionSink::rewrite_without` | R1 |
| `checkpoint::{StateCheckpoint, WorkflowState, hardlink_cloner, pin_checkpoint, blake3-Manifest}` | Snapshot-Set-Register + Restore-Koordinator | R5 |
| `block_on` in `text`, `graph`, `engine`, `checkpoint`, `crypto` | nur `engine::kernel` | R1 |
| `memmap2` in `sys` ohne Feature | Feature `mmap`, Standard aus | P0-B |
| `StorageEngine`/`StorageBackend`/`CommitBarrier`/`Purger`/`LedgerSink`/`ErasureLayer`/`Vfs` in `ports` | `ports::legacy`, bedient von `engine::kernel` bis G-CUTOVER | R3 |
| `deletion_proof`, `revocation_log` in `crypto` | Umbau auf `ErasureReceipt`-Eingang | R5 |

---

## 14. Invarianten

Normativ; jede ID wird in mindestens einem Test per `// INV-…` zitiert (§12.4). Invarianten gehen allen anderen Abschnitten vor.

| ID | Aussage |
|---|---|
| INV-STAMP-1/2/3 | Stamps streng monoton über angenommene Changes/Frames · kein Umbruch (`Exhausted`) · nie doppelt vergeben, auch nicht nach Absturz |
| INV-ID-1/2 | Datei-/`tmp_id`, `SpaceId` nie wiederverwendet; Space-Gates binden die vollständige SpaceId |
| INV-ATOM-1 | Ein Change ist ganz oder gar nicht sichtbar, auch nach Absturz, über alle Spaces und Branches (nicht für Volatile-Ziele) |
| INV-COMMIT-2 | Ein sichtbarer Eintrag ist durabel (außer `Relaxed`) |
| INV-GUARD-1 | Ein Change wird nur angenommen, wenn jede Guard zur Stamp-Vergabe gilt |
| INV-VIEW-1/2 | Eine View sieht stets dieselben Daten · `visible_stamp` vor `Arc<Version>` |
| INV-VIEW-3 | Nach Lease-Ablauf (plus `revoke_grace`) hält keine View eine Version; jede Nutzung ⇒ `SnapshotTooOld`, nie Daten einer entfernten Datei |
| INV-VIS-1 | Jedes Extension-Ergebnis passiert Kern-Gate-Zustand und `filter_visible`, bevor es den Aufrufer erreicht |
| INV-PRUNE-1 | Lesen mit `BlockPruner` ≡ Lesen ohne; `may_skip` nur für schattensichere Besitzer auf `main` |
| INV-VER-1 | Kein `Complete`-Receipt, wenn eine Pflichtmessung nicht `Complete` ist oder ein Statuscap gilt |
| INV-CONS-1 | Consumer und `view_durable` sehen nur Stamps ≤ `durable_stamp` derselben `instance_id` |
| INV-LOCK-1/2 | Sperren nur aufsteigend · Stamp-Vergabe und `Live → Retiring` nur im Sequencer; `Retiring → Retired` nur durch Publikation nach durablem Sync |
| INV-CAT-1/2 | Datei lebendig ⇔ durabler `files_add` ohne `files_remove`; entfernte Datei bleibt bis `unlink_ack` im Graveyard · Edit gilt erst nach `sync_data` der Gruppe |
| INV-FMT-1 | Nur kanonische Kodierungen; unbekannte Version/Tags ⇒ `Unsupported` |
| INV-LOG-5/7/8/9 | `replay_floor`-Regel · Reservesegment vorallokiert, alles bis `redo_mark` gültig · Torn-Tail-Regel · nach ER4 kein lebendes Redo-Segment mit Plain-Nutzlast aus H (Sealed: spätestens bei `Done`) |
| INV-GRAVE-1/2 | Kein Unlink solange eine `Arc<Version>` zeigt; Graveyard bis `unlink + sync_dir + unlink_ack` · nach `Revoke` hält nach `revoke_grace` keine View eine Version |
| INV-CLAIM-1 | Ein Edit entfernt nur Dateien unter eigenem Claim |
| INV-BLOB-1 | Blob/Inkarnation nur entfernt, wenn kein lebender Run darauf verweist |
| INV-PL-1…5 | Pure/Packed/Staging eindeutig · clusterreine Abschnitte mit abschnittsrelativen Offsets · Compaction mischt nur in Wartungsgruppen · `cluster → Gruppe` nur im selben Edit wie die Dateimengen · kleiner Cluster höchstens eine Datei je Level |
| INV-CLUSTER-1/2 | `ClusterId` nie zweimal vergeben · nicht aus Kennung ableitbar |
| INV-SEAL-1 | Schutzklasse ↔ Space-Siegel (§4.2); gilt auch für Systemspaces (`SYS_BRANCH_S/P`) |
| INV-ERA-1 | Logische Atomarität, `Erased{at:S}` für jede View, keine Mischsicht |
| INV-ERA-2 | Nach `Purged` enthält keine lebende Datei Bytes von H (§8.3 Nr. 3), auch keinen Ciphertext; bei `Cryptographic` zusätzlich INV-ERA-7 ab `KeysDestroyed` |
| INV-ERA-3 | `EraseIntent` durabel < Gate < Catalog-Gate-Edit < `GateApplied` < Anker < `KeyDestroyIntent` < `destroy_many` < jedes `PurgeBatchEdit` < Unlink+`sync_dir` < `UnlinkAck` < ER6/`Done`; `Done` nur nach allen Attesten |
| INV-ERA-4 | Nur Roll-forward |
| INV-ERA-5 | Reads unter `S` in `erased_ranges` liefern `Erased`, nie Daten, nie „leer“ |
| INV-ERA-6 | Nach `Purged` keine lebende Blob-Inkarnation mit Record des Clusters |
| INV-ERA-7 | Nach ER2 ist alles des Sealed-Clusters im Kern ohne Clusterschlüssel unlesbar; Extensions/Backups nur bei nachgewiesen gleichwertiger Scope-Key-Vernichtung |
| INV-ERA-8 | Receipt nur bei erfüllten Done-Bedingungen (§8.5) |
| INV-ERA-9 | Nach Restore werden Ledger-Intents vor Open scope-genau angewandt; kein Backup wird vorher sichtbar |
| INV-ERA-10 | Eine `BLOCK_ERASE`-Restriction in der Hülle verhindert jede Löschung; Prüfung auf dem Aufruferthread, Revalidierung im Sequencer (`policy_epoch`), kein Override |
| INV-ERA-11 | Gate/KeyDestroy skaliert mit Zielclustern; Purge bearbeitet nur betroffene Dateien/Extents und gepinnte Redo-/Extension-Bereiche; `PackedBounded` ≤ Summe betroffener, je `max_file_bytes` begrenzter Dateien; kein Vollstore-Rewrite |
| INV-ERA-12 | Die Erase-Menge ist der transitive `DerivedFrom`-Abschluss zum Gate-Zeitpunkt, ≤ `max_gate_clusters`, in einem Frame gated |
| INV-ERA-13 | Eine auf `Poisoned` aufgenommene Löschabsicht wird beim nächsten schreibenden Open vor jeder Nutzer-Sicht gated; bis dahin `ErasePending`; kein KeyDestroy/Purge vor durablem Gate |
| **INV-LED-3** | Jede Cluster-ID in einem Gate-Frame ist durch einen durablen `EraseIntent`/`EraseIntentExtend` desselben Tickets gedeckt; der Sequencer gatet nie eine ungedeckte Hülle (FIX-29) |
| INV-LED-1 | Das Ledger enthält keine Klartext-ClusterId/SpaceId, Benutzerkeys oder Nutzdaten; Selektoren nur AEAD unter `LedgerScopeKey` |
| INV-LED-2 | Jedes `tomb_id` gehört zu genau einer `tomb_epoch`; nach `TombEpochRetired` nicht mehr auf eine ClusterId abbildbar; Retire nur für geschlossene Epochen nach `backup_horizon`; Restore verweigert Kopien unter `restore_floor` |
| INV-ANCH-1 | Anker monoton; Catalog/Ledger hinter dem Anker wird nicht schreibend geöffnet; Anker nie vor dem durablen Edit |
| INV-KEY-1…4 | Schlüsselmaterial nie unverhüllt in Dateien/Checkpoints · `gen` vor und nach Entsiegeln · alte Epoche erst vernichtet, wenn nichts mehr mit ihr versiegelt · `destroy_many` vernichtet alle Epochen |
| INV-KEY-5 | Tomb-Schlüssel stehen nie in `KEYS`, Checkpoints, Backups oder Kerndateien; Retire erst mit `probe == Absent`; ein fehlender Schlüssel einer nicht retired Epoche ist nie ein Retire |
| **INV-DL-1** | Ein angenommener Löschauftrag erfüllt `purge_deadline ≥ purge_deadline_min` und `max_delay + W + Reserve ≤ purge_deadline` (FIX-30) |
| INV-PRIV-1 | Der Kern persistiert keine Inhalts-Hashes über Klartext (flüchtige Prozessschlüssel-Fingerabdrücke ausgenommen); Ausnahme nur als clusterlokaler Keyed-MAC unter einem im selben Sealed-Cluster liegenden Zufallssecret (§11.12) |
| INV-VOL-1 | Volatile schreibt kein Redo; Mischchange nicht durabel über Absturz |
| INV-LIM-1 | Changes über den Limits werden vor jeder Sperre abgewiesen |
| INV-DISK-1 | Datenträger voll vergiftet nicht; Gate, Schlüsselvernichtung, Purge bleiben ausführbar |
| INV-BR-1 | Branch-Lesen ≡ Referenzmodell (Eltern-Kette mit Stamp-Kappen) |
| INV-BR-2 | Merge ist atomar: bei Konflikten ändert sich nichts; bei Erfolg sind alle Auflösungen und `Merged` in einem Frame |
| INV-BR-3 | `history_floor` ≤ `fork_stamp` jedes lebenden Branches; Branch-Daten und -Metadaten sterben mit dem Cluster |
| INV-BR-4 | Fork kopiert keine Nutzdaten (O(1)); Branch-IDs je Cluster nie wiederverwendet |
| INV-BR-5 | Branch-bewusster Filter und `BranchTable` liefern nie ein falsches Negativ; Chain-Resolve und Cursor ≡ Referenzmodell inkl. `RangeDelete` auf Zwischenebenen |
| **INV-MERGE-1** | Ein Merge mit mehr als `max_merge_ops` Mutationen oder `max_change_bytes` wird vor jeder Mutation mit `MergeTooLarge` abgewiesen (FIX-33) |
| **INV-HIST-1** | `history_floor ≤ gc_floor`; `AppendOnly` ist für `DataClass::Personal` unzulässig; Versionen unter dem Boden sind nach Compaction physisch entfernt (FIX-31) |
| INV-PIPE-1 | Unter Rang 1 weder E/A noch Provider-/Cipher-/KDF-Aufruf noch Warten; Publikation strikt in Stamp-Reihenfolge nur nach durablem Sync derselben Gruppe |
| INV-PIPE-2 | Sync-Fehler ⇒ keine Gruppe mit gleichem/höherem Stamp wird veröffentlicht; alle `Indeterminate`/`Poisoned`; ein `Retiring` ohne durables Gate existiert nach Neustart nicht |
| INV-POL-1 | Gate, Link, Restrict, Lift sind im Sequencer total geordnet; ein Gate übernimmt nur eine zu seiner `policy_epoch` berechnete Hülle (oder die unter `link_fence` selbst berechnete, die der Intent deckt) |
| INV-ADM-1 | Cluster-Metadaten, `ClusterLocal`-Cluster, Live-Dateien, FD-Reserve nur nach erfolgreicher Admission; keine stille Herabstufung |
| INV-RAG-1 | Kein Ingest ist halb sichtbar: Chunks einer Version sind nur über `doc/<id>/head` erreichbar |
| INV-RAG-2 | Eine Projektion wendet eine Dokumentversion nur als Ganzes an; ihr Watermark reicht nie über `durable_stamp` derselben Instanz |
| INV-RAG-3 | Unbedeckter Bereich `(applied_stamp, V]` ⇒ Delta-Ergebnisse oder `IndexLag`, nie stilles Auslassen |
| INV-RAG-4 | Gleiche Anfrage + gleiches `SnapshotSet` ⇒ bitgleiches Ergebnis |
| INV-PART-1 | Jede Partition gehört genau einer Packgruppe, überschreitet ihr Limit nicht; Generationsschlüssel `g` wird erst nach Publish von `g+1` und `verify_absent` vernichtet |
| INV-SEP-1 | Personenbezogener Klartext nur in `Sealed`-Clustern; Korpus-Chunks, Embeddings, Terme, Graph-Entitäten des Korpus aus pseudonymisiertem Text; Platzhalter je Person zufällig |
| INV-SEP-2 | Geteilte Partitionen enthalten nur aus pseudonymisiertem Text Abgeleitetes; Klartext-Embeddings nur in Cluster-Abschnitten (Modus A/C) |
| **INV-DIR-1** | Jeder persistente Kennung→Cluster-Pfad ist registrierter `MixedPerSubjectKey`-Owner oder gleichwertig attestiert; die Entry-Key-ID wird nie aus der ClusterId abgeleitet (FIX-23) |
| **INV-EGRESS-1** | Kein `Personal`-Kontext erreicht ein externes Ziel ohne `EgressPolicy`-Verdikt `Allow`/`AllowPseudonymized`; externe Ziele erhalten nur Platzhalter |
| INV-GRAPH-PROV-1 | Jede Kante/Hyperkante trägt `(source_cluster, source_doc, version)`; je Löschdomäne eigene Kopie |
| INV-DECAY-1 | Recency-Prior, `Supersedes` und Relevanzgewichtung löschen nie und erzeugen weder Gate noch Receipt; zeitgesteuerte Löschung ist ausschließlich Retention über `Eraser` |
| INV-EPOCH-1 | Genau ein `Epoch(u64)`-Typ (`types`); `Stamp ↔ Epoch` nur in `engine::projection`, mit `instance_id` |
| INV-LAYER-1 | Abhängigkeiten nur auf strikt niedrigere Ebenen; `storage` kennt nur `ports::kernel` und `types`; Retrieval-Crates kennen weder `storage` noch einander; Produktivpfade ohne `std::fs`, `memmap2`, `bincode`, festen Schlüssel, unkeyed Inhalts-Hash |
| INV-TOPO-1 | Der Workspace besteht aus genau den 14 Crates aus §2.1; `[workspace].members`, `capabilities.toml`, `crates/` stimmen überein (Test) |
| INV-SCOPE-1 | Kein Code für KV-/Prompt-Cache, Agenten-Orchestrierung, Session-DAG, kognitiven Decay, Modell-Routing (`check_forbidden_apis.sh`: `kv_cache`, `KvCache`, `kvcache`, `AdaptiveDecay`, `SessionBranchTree`, `contextra_store`, `contextra_storage_ports`) |
| INV-PORT-1 | Kernel-Ports nur in `contextra-ports::kernel`, synchron, frei von `futures`/`serde`/`std::fs`; jeder Adapter besteht die Konformanzsuite vor der Verdrahtung |

**Modul-Invarianten** (`INV-HNSW-SLOT-1`, `INV-VEC-*`, `INV-SIMD-*`, `INV-TEXT-*`, `INV-DETERMINISM`, `INV-GRAPH-00x`, `INV-RRF-*`, `INV-CAL-*`, `INV-FSYNC-1`, `INV-VFS-ATOMIC-1`, `INV-TENANT-IDX-1`) stehen in `crates/*/SPEC.md` und werden hier nur zitiert; bei Widerspruch gilt diese Tabelle. `INV-DB-3` (Kompensation der Alt-Engine) entfällt mit dem 2PC.

---

## 15. Konfiguration

Alle Werte sind Standardwerte; „Pflicht“ = kein Standard. Dieselben Zahlen gelten in §3.7, Ports-Konstanten und Config (`INV-LIM-1`).

**Speicher.** `memory_budget_bytes` = `min(1 GiB, max(256 MiB, 20 % des effektiven cgroup-/Job-Limits))`; liefert der Adapter kein verlässliches Limit, ist der Wert beim Öffnen Pflicht. Unterbudgets sind Obergrenzen am selben Budget: 25 % Memtables · 15 % Cluster-/Branch-Metadaten (`cluster_meta_budget`) · 2 % Guard-Fenster · 20 % Block-Cache · 1 % Key-Cache · 20 % Extension-/Indexdelta (davon `index_rebuild_bytes` 10 %) · 10 % In-flight-Changes/Rewrite-Puffer · 7 % Allocator-/Runtime-Headroom. Jede Allokation reserviert atomar (`try_reserve`) und gibt per RAII frei. 16 Mio. Cluster brauchen ≈ 610 MiB Metadatenbudget (≈ 4,0 GiB Gesamtbudget); `max_clusters` ist Obergrenze, nicht zugesagte Residenz.

**Formate/Platzierung.** `block_target` 16 KiB · `run_target_bytes`, `blob_max_bytes`, `redo_segment_bytes` je 64 MiB · `redo_mark_bytes` 8 MiB · `redo_max_age` 1 h · `redo_align` 4096 · `dir_page_bytes` 4096 · `mini_section_max_bytes` 4096 · `file_shard_count` 1024 · `pure_threshold` 8 MiB · `pure_demote_ratio` 0,25 · `max_pack_groups` 4096 · `spill_min_bytes` 8 MiB · `spill_ratio` 4 · `spill_max_age` 6 h · `max_staging_runs` 16 · `memtable` 64 MiB je Space, ≤ 3 immutable, 256 MiB gesamt · `value_separation_min` 1 KiB.

**Limits.** `max_key_bytes` 1 KiB (≤ 4 KiB) · `max_header_bytes` 4 KiB (≤ 64 KiB) · `max_body_bytes` 4 MiB · `max_change_ops` 4096 · `max_change_bytes` 8 MiB · `max_page_bytes` 1 MiB · `max_clusters` 16 Mio. (≤ 64 Mio.) · `max_branches_per_cluster` 1024 · `max_branch_depth` 4 (hart 8, >4 nur mit `deep_ack`) · **`max_merge_ops` 4096** (= `max_change_ops`).

**Admission.** `max_live_files` 262 144 · `max_open_files` 1024 · `fd_reserve` 256 · `max_cluster_local_clusters` 1024 (≤ 65 536) · `cluster_local_min_bytes` = `pure_threshold`.

**Pipeline/Wartung.** `commit_pipeline_depth` 2 · `max_group_changes` 1024 · `max_group_bytes` 8 MiB · `guard_window_entries` ≤ 1 Mio. (durch Budget begrenzt) · `guard_retry_max` 3 · `policy_retry_max` 3 · `publish_timeout` 1 s · Reservierungen `reserve_stamps` 2^20, `reserve_ids` 2^16, `reserve_space_ids` 2^16; Notpool `reserve_stamps_urgent` 2^10, `reserve_ids_urgent` 2^8 · `maint_io_fraction` 0,5 · `max_lag_bytes` 1 GiB.

**Löschung.** `purge_deadline` **Pflicht** · `purge_deadline_min` 6 h · `purge_window_fraction` 0,3 · `deadline_reserve_fraction` 0,1 · `sealed_purge_escalate` 0,6 (`flush`) / 0,8 (`urgent`) · `purge_batch_min` 16 · `purge_rewrite_concurrency` 1 · `purge_scratch_reserve_bytes` ≥ 128 MiB (zwei Slots) · `erase_reserve_bytes` ≥ 2 GiB (**enthält** Scratch, Emergency-Segmente für Redo/Catalog/Ledger und FS-Metadaten-Headroom) · `max_gate_clusters` 4096 · `max_links` 8 · `max_link_depth` 8 · `link_fence_ttl` 5 s · `logical_only_alarm_age` 30 d · `anchor_max_age` 1 s · `orphan_key_grace` 24 h · `key_grace` 10 min · `revoke_grace` 5 s.

**Zeit/Sichten.** `view_lease` 60 s · `view_lease_max` 10 min · `max_hold_ttl` = `idem_ttl` 24 h · `max_clock_skew` 5 min · `history_retention` 7 d (nur `KeepUntilFloor`) · `LiftPolicy` `OwnerOrAdmin`.

**Tomb/Backup.** `tomb_epoch_rotation` aus · `backup_horizon` Pflicht, sobald Retire aktiv · `max_active_tomb_epochs` 4.

**Retrieval/Profil.** `micro_index_max_vectors` 50 000 · `cluster_index_max_vectors` 2 Mio. · `emb_precision` F16 · `max_partition_vectors` 500 000 · `max_partition_postings_bytes` 256 MiB · `max_partition_edges` 5 Mio. · `…_personal` 100 000 / 64 MiB / 1 Mio. · `max_graph_segments_per_tenant` 64 · `rerank_depth` 4 × k · `ryw_wait` 50 ms · `staged_orphan_ttl` 24 h · `k_min` 50 · `restore_slack_fraction` 5 % · `scan_fallback_max_vectors` 1 Mio. · `export_io_fraction` 0,2 · `filter_selectivity` 5 % · `ComplianceProfile` `Standard` · `pseudonym_scope` `PerPerson` · `placeholder_retention` `Keep` (`Strict`: `Scrub`).

**Metrikvertrag (Z-16).** Ein fester Katalog (`metrics.rs`) definiert Name, Typ, Einheit, begrenzte Labels, Bedeutung. Pflichtfamilien: Commits (Anzahl, Gruppengröße, Latenz nach `durability`/`result`), Durability (Stamp-Lag, Fsync-Latenz nach `file_role`), Ressourcen (Bytes/Limit nach `resource_class`, Cache-Belegung/-Trefferquote, Reserve, Scratch), Wartung (Bytes/Laufzeit nach `phase`), Erase (offene Tickets, ältestes Alter, Fristverletzungen `erase_deadline_breaches_total`, Dauer je `stage`, Gate-/KeyDestroy-Latenz, Purge-Bytes, Anchor-Lag, `logical_only_clusters_aged`, `poisoned_erase_pending`), Extensions (Watermark-Lag, blockierte Owner nach `owner_kind`), Guards (Stufe-A-Konflikte, `GuardRetry`-Rate, Fensterfuß-Treffer), Branches (Anzahl, Tiefe, ältester Pin), Fehler/Poison nach `ErrorClass`, abgelaufene Leases, Tomb-Epochen (nur Zähler/Gauges). Labelwerte stammen nur aus kleinen geschlossenen Enums (`durability`, `result`, `file_role`, `resource_class`, `phase`, `stage`, `owner_kind`, `ErrorClass`). Keine IDs, Namen, Tenant-/Cluster-/Space-Kennung, Schlüssel, Token, Querytexte, Inhalte, Embeddings. `Metrics`/`Observer` sind best-effort, nicht blockierend, außerhalb aller Kernlocks; Ausfall ändert keine Semantik. Der Inspector liefert standardmäßig nur aggregierte/opake Werte, braucht Capability, entsiegelt nie. Katalog-Review und Kardinalitäts-/Leakage-Test sind Release-Gates.

---

## 16. Leistungshypothesen

Alle Werte sind **Entwurfsziele**. Freigabe nur mit Messbericht (Hardware, Dateisystem, Last, Methode, Streuung); scheitert eine Hypothese, wird **sie** korrigiert, nicht das Verfahren gelockert. Benchmarkgruppen: **Storage**, **Agent-Workload** (Request → letzte 100 Events → Zustand → 20 Erinnerungen → Update → 10 Events anhängen → Checkpoint → Commit; gemessen in physischen Reads/Writes, CPU, RAM), **Retrieval-Qualität** (Recall@Budget, Nutzen pro Token, Task-Erfolg bei gleichem Budget).

| ID | Hypothese | Messung |
|---|---|---|
| H-1 | `apply`: Durchsatz ≥ 80 % von Fsync-Rate × mittlere Gruppengröße; p99 ≤ 2 × Fsync-Latenz bei ≤ 70 % Auslastung | Mikrobench |
| H-2 | Punktabfrage warm: ≤ 1 Datenblock-Read je Treffer-Level, p99 ≤ 50 µs | Zipf-Last |
| H-3 | Kopf-Scan ohne Körper ≥ 10× Durchsatz gegenüber Scan mit Körper (≥ 4 KiB) | Scan-Bench |
| H-4 | `page` liest ≤ 1,2 × Blöcke des Minimums; Aufbauzeit der Präfixsummen (≤ ≈ 4096 Blöcke) und Traversierung getrennt berichten | Gewichtsverteilungen |
| H-5 | Schreibverstärkung Metadaten ≤ 15, Körper ≤ 2 | Compaction-Statistik |
| H-6 | ER0–ER2 je Cluster ≤ 10 ms (ohne Providerlatenz), unabhängig von Memtable-Größe | Löschbench |
| H-7 | Purge-Bytes ≤ Summe betroffener begrenzter Datei-/Extent-Größen plus dokumentierte Metadaten; kein O(Clusterbytes)-Ziel | je `EraseLocality`/Dateirolle |
| H-8 | Pure-Cluster löschen ≤ 100 ms je 1000 Dateien (ohne Scrub) | Drop-Bench |
| H-9 | 1000 Cluster einer Gruppe ≤ 2 × Aufwand eines Clusters | Bündelungsbench |
| H-10 | Rohkopie beim Purge ≥ 80 % sequentielle Bandbreite | Purge-Bench |
| H-11 | Seal ≤ 15 % CPU im Lastmix; Overhead kleiner Cluster ≤ 5 % Bytes ab 4 KiB, ≤ 10 % für 1–4 KiB (`MiniSection`) | A/B-Bench |
| H-12 | RAM ≤ `memtable × (1+max_immutable)` + Cache + O(offene Gruppen); `ClusterIndex` ≤ 40 B je Cluster; Directory-Sparse-Index + `ClusterFilter` ≤ 1,3 B je Cluster und Datei | RSS |
| H-13 | Recovery ∝ Redo-Rückstand + Catalog; 1 GiB Redo ≤ 10 s; Catalog mit 16 Mio. Clustern bis `Running` ≤ 20 s | Recovery-Bench |
| **H-14** | Redo-Barriere: Dauer ≤ `redo_max_age` + Flushzeit; eine Barriere bedient alle fälligen Tickets | Barriere-Bench |
| H-15 | Zeilenabdeckung ≥ 90 %, Zweigabdeckung ≥ 85 % | `cargo-llvm-cov` |
| H-16 | Mutationsscore ≥ 80 % auf `codec`, `recovery`, `erase`, `sequencer`, `syncer`, `branches` | `cargo-mutants` |
| H-17 | Gate + ER2 für 10 000 Cluster ≤ 1 s bei Providerlatenz ≤ 5 ms je `destroy_many` | Batchbench |
| H-18 | Guard-Stufe B ohne Retry ≥ 99 % bei Zipf mit 1 % Konflikt; **0 E/A-Syscalls** im Sequencer | Guard-Bench |
| H-19 | L0-Dateizahl unabhängig von der Zahl aktiver Packgruppen | Flush-Bench, 4096 Gruppen |
| H-20 | Fork < 50 µs, Speicher O(1) | Fork-Bench |
| H-21 | Branch-Punktlesen bei 1 % Änderungen: Datenblock-Reads ≤ 1,1 × `main`; p99 ≤ 1,2 × bis Tiefe 4, ≤ 1,5 × bis Tiefe 8; Worst-Case und Bereichs-Scan; Spike vor G-F | Branch-Bench |
| H-22 | Merge ∝ Source- + Target-Mutationen seit `merge_base` + Konflikte; je 10⁴ Mutationen ≤ 200 ms (ohne Körper-I/O); Abweisung `MergeTooLarge` in O(1) ab dem ChangeIndex-Zähler | Merge-Bench |
| H-23 | Restore-Matching: Cluster × aktive Tomb-Epochen HMACs für 16 Mio. Cluster und 1/4 Epochen im Restore-Budget; Retire ohne Wartungsstau | Restore-Bench |
| H-24 | Sequencer (Rang 1) p99 ≤ 20 µs für Gruppen ≤ 64 Changes ohne Gate; Gate mit `policy_epoch`-Treffer ≤ 10 µs; Hüllenberechnung bei \|H\| = 4096 ≤ 250 µs; **0 E/A, 0 Provider-Aufrufe**; Commit-p99 bei parallelem 4096-Cluster-Gate ≤ 1,2 × ohne Gate | Commit-Bench, `FaultFs`, Sim |
| H-25 | 16 Mio. Cluster: `ClusterIndex` ≤ 700 MiB; Sparse-Index ≤ 1 B je Cluster/Datei; Gate-Check ohne E/A; kalter Punktlesezugriff ≤ 1 Directory-Seite je Level-Datei, warm 0 | RSS-/Cache-Bench |
| H-26 | Staging→L1-Schreibverstärkung je Gruppe ≤ 1 + `spill_ratio` (≤ 5) bei 4096 Gruppen; Gesamt ≤ 15 für Millionen kleiner Cluster | Packgruppen-Bench |
| H-27 | `ClusterLocal` bei 1 Mio. Clustern (1 % beantragt): Admission lehnt über Quote ab; FD-Cache-Trefferquote ≥ 99 %; `open()`-Syscalls ≤ 0,01 je warmem Punktlesen; Sharding ohne Verzeichnis-Degradation | FS-Bench (ext4/xfs) |
| H-28 | je `W = 0,3 × purge_deadline` wird jede Datei höchstens einmal umgeschrieben; Purge-Bytes je `W` ≤ Σ betroffener Dateien; bei 0,1 % Löschrate/Tag auf 16 Mio. Clustern Purge-Schreibverstärkung ≤ 1 je `W` | Purge-Bench, Langzeitlauf |
| H-29 | `Basis ∪ Delta` ≤ 1,15 × Basislatenz bei Delta ≤ 1 %; Text/Graph bitgleich zu Vollscan, Vektor-Recall ≥ Basis | Differentialbench |
| H-30 | Hybridanfrage (3 Kanäle, k = 20, 10 Mio. Chunks in ≤ 20 Partitionen, warm, ohne Rerank-Modell): p99 ≤ 150 ms; Storage-Anteil ≤ 15 % | Retrieval-Bench |
| H-31 | Fan-out 1 → 16 Partitionen: Latenz ≤ 1,3 × | Partitionsbench |
| H-32 | Rebuild-Bytes je Fenster ≤ Σ betroffener Partitionen, nie ∝ Aufträge; Anteil an Wartungsbandbreite ≤ 25 % bei 0,1 % Löschrate/Tag; Commit-p99 ≤ 1,2 × | Rebuild-Bench |
| H-33 | Graph: Adjazenz über ≤ 64 Segmente ≤ 2 × Einzel-CSR; PPR reproduzierbar | Graphbench |
| H-34 | `pii_residual_rate` je Release (n ≥ 1000, Konfidenzintervall); kein Zielwert im Kern | Prüf-Orakel mit Canary-Personen |
| H-35 | Read-your-writes: p99 Receipt → Sichtbarkeit ≤ `ryw_wait` + Delta-Latenz (≈ 60 ms); gemeinsames RSS-Budget eingehalten | RYW-Bench |
| H-36 | Person mit 10⁴ Chunks in `Pseudo`-Partitionen: Rebuild-Bytes je Fenster ≤ Σ betroffener Partitionen (je ≤ 100 000 Vektoren); Anteil ≤ 25 %; `Incomplete` nur bis Generationswechsel + Schlüsselvernichtung | Rebuild-Bench (Spike vor G-R2) |
| **H-37** | Modus C: Aufbau für 1 Mio. Vektoren ≤ 10 min im Hintergrund ohne Commit-p99-Anstieg > 1,2 ×; Query-p99 ≤ 1,5 × einer Partition gleicher Größe; Löschung = Gate + Schlüsselvernichtung ohne Rebuild | Cluster-Index-Bench (Spike P0-B) |
| **H-38** | `emb_precision = F16`: Rerank-Recall@10 ≥ 99 % des F32-Reranks; Speicherbedarf ≤ 55 % | Recall-Bench |
| **H-39** | Pseudonym-Auflösung + Budgetprüfung für k = 20 ≤ 5 ms; `trimmed_after_restore` ≤ 1 % der Anfragen bei 5 % Reserve | Query-Bench |

---

## 17. Risiken, offene Entscheidungen, externe Vorbedingungen

### 17.1 Entschiedene Fragen

| Frage | Entscheidung |
|---|---|
| Wer setzt Restrictions, Ablauf automatisch? | Jeder Handle-Besitzer darf `restrict`; `lift` nach `LiftPolicy`; Ablauf lässt nur den Vermerk fallen |
| Zugriffsstatistik für Layout | nicht im Kern; obere Schicht in eigenem Cluster/Space |
| Verschlüsselungsgranularität vs. Extent-Layout | gelöst durch Clusterreinheit |
| Merge-Funktionen in WASM | nein (NZ-4); Client liefert Auflösungen |
| Eigener FTS vs. Tantivy | eigener Index; Tantivy nur als `SegmentOwner` mit `rewrite_without` |
| Embedded vs. Server | Embedded-Kern; Server/MCP-Wrapper oben ab G-P |
| Keyring als Single Point of Loss | Pflicht-Replikation im Provider; `KeyLost` unterscheidbar |
| KMS-Karenzzeit | Ticket `Blocked`, bis Provider `Absent` attestiert; strenge SLAs verlangen sofort zerstörenden Provider |
| Single-Sequencer-Durchsatz | ein Committer je Instanz; Mandanten-Sharding über mehrere Instanzen oben |
| Merge größer als ein Change | abgewiesen (`MergeTooLarge`), nicht gesplittet (INV-BR-2 bleibt atomar) |

### 17.2 Risiken

| Nr. | Risiko | Mitigation |
|---|---|---|
| 1 | Branch-Pinning treibt Speicher/Compaction bei vielen Langzeit-Branches | `ttl`, Limits, `floor`-Metrik |
| 2 | Branch-Reads über tiefe Ketten (H-21) | branch-bewusster Filter, Default-Tiefe 4, Spike vor G-F |
| 3 | Plain-Barriere bei häufigen Plain-Löschungen | Bündelung; `Personal` ist per Konstruktion `Sealed` |
| 4 | Verzeichnis Person → Wurzelcluster nur physisch löschbar | §11.13, `dir_key`-Restrisiko im Receipt |
| 5 | Formatfreeze nach P2 verlangt echte Probeläufe | P1/P2 als vertikale Durchstiche, nicht Papier |
| 6 | Scope-Creep in obere Schichten | Gates strikt, `INV-SCOPE-1` |
| 7 | Komplexität der Commit-Pipeline | M1, `loom`, Simulation RACE-18…20 |
| 8 | RAM ab ≈ 4 Mio. Clustern | Admission (§3.4), „Session ≠ Cluster“ |
| 9 | Zeitplan P2/P6/P7 um Faktor 1,5–2 zu knapp | Gates bleiben, Termine nach P1/P2 neu schätzen |
| 10 | Rebuild-Kosten geteilter Indizes bei hoher Löschrate | Größenlimit, Fenster-Amortisierung, `Blocked`, H-32 |
| 11 | Fehlklassifikation personenbezogener Inhalte | Quarantäne, `pii_residual_rate`, Receipt-Restrisiko, H-34 |
| 12 | Retrieval-Bereinigung (R1) bei laufender Weiterentwicklung | Verbotslints, Klasse-A-Build als Gate |
| 13 | Platzhalter-Verknüpfbarkeit im Korpus (`Keep`) | `Scrub`-Option, Profil `Strict` (§11.9) |
| 14 | Personenbezogene Akten > 2 Mio. Vektoren ohne Teilcluster | `Limit{ClusterIndexVectors}` mit `hint()`; Aufteilung nach Löschdomäne |
| 15 | Zeitreise-Audit nach Löschung nicht reproduzierbar | bewusst (FIX-15); `time_travel_degraded` ausgewiesen |

### 17.3 Offene Entscheidungen mit Freigabebedarf

| Entscheidung | Wer | Standard bis dahin |
|---|---|---|
| Signaturformat `DeletionProof` (Ed25519, Schlüsselverwaltung) | Projektinhaber | `crypto::deletion_proof` unverändert; Signatur optional |
| Windows-Produktionsfreigabe | Projektinhaber nach Fs-Konformanzlauf | nicht freigegeben (NZ-8) |
| `Strong`-Anker und HSM/KMS-Provider | Betrieb/Kunde | `Weak` nur mit Bestätigung; `Standard` verlangt KMS |
| Edition/`LicenseGate`-Grenzen | Produkt | Open-Core-Grenze §1.5; keine Kerninvariante hinter Bezahlschranke |
| Rechtliche Einstufung „anonym nach Tresorvernichtung“ | Rechtsberatung des Kunden | Restrisiko im Receipt |
| Aufspaltung von `contextra-engine` | Architektur-Review nach G-R3 | nein in v1; Trigger > 40 kLOC oder zyklische Modulabhängigkeit (`INV-TOPO-1` bleibt 14) |
| `edge-reinforcement-learning` | Projektinhaber | nicht freigegeben |
| `max_merge_ops` > 4096 (zweiphasiger Merge) | nach G-V-Messung | 4096 |

**Externe Vorbedingungen (keine Spec-Lücken):** rechtliche Prüfung der Fristen und der Aussage „kryptografisch gelöscht“ im Einsatzfall; zweiter Reviewer für G-0; `Strong`-Anker und HSM/KMS-Provider für den Produktivbetrieb (der Kern ist ohne sie vollständig, Restrisiken sind benannt).

---

## 18. Entscheidungen

Kompromisse und verworfene Alternativen. IDs sind stabil.

| ID | Entscheidung | Verworfene Alternative / Abwägung |
|---|---|---|
| DEC-01 | Eingebetteter synchroner Kern; Async nur am Rand (`engine`) | Async-Kern: Lock-über-await-Risiken, nicht deterministisch simulierbar |
| DEC-02 | Ein Sequencer (Rang 1, RAM) + ein Syncer; Skalierung per Instanz-Sharding | mehrere Redo-Logs: zerbricht Stamp-Kette und Gate-Ordnung |
| DEC-03 | Cluster = Löschdomäne = Schlüsseldomäne = Versionsdomäne | Löschen auf Zeilenebene: nur Logical, keine physische/krypto Garantie |
| DEC-04 | Eine Siegelform (XChaCha20-Poly1305, 24-B-Zufallsnonce), Rohkopie ohne Entsiegeln | AES-GCM-SIV/Nonce-Zähler: Zustand/Abhängigkeit ohne Nutzen |
| DEC-05 | Clusterschlüssel im `KeyProvider`, nie in `KEYS` | Schlüssel im Datenverzeichnis: Crypto-Erase nicht beweisbar |
| DEC-06 | Löschung in drei geordneten Schritten Gate < KeyDestroy < Purge; nur Roll-forward | atomare Gesamtlöschung: Gate-Latenz ∝ Datenmenge |
| DEC-07 | Packgruppen `(tenant_tag, due_bucket, shard)`; Purge amortisiert je Fenster `W` | Cluster-je-Datei: Dateiflut; globale Dateien: Survivor-Rewrites ∝ Store |
| DEC-08 | `due` löst Löschauftrag exakt aus; `due_bucket` nur physische Ko-Lokation (FIX-34) | Löschung erst bei Bucket-Ende: Überaufbewahrung bis Bucket-Länge |
| DEC-09 | Gate-Frame im Sequencer; Intent vor Gate; Hülle ⊆ Intent; `link_fence` statt Sequencer-I/O (FIX-29) | Sequencer schreibt Ledger: E/A unter Rang 1; Selbstberechnung ohne Intent: Backup belebt Gelöschtes |
| DEC-10 | Ein `Mac`-gekettetes Ledger mit epochengebundenen `tomb_id`s; Retire nach `backup_horizon` | ewig linkbare Pseudonyme: Ledger ewig personenbezogen; sofortige Anonymisierung: Backups beleben Gelöschtes |
| DEC-11 | Receipt mit kanonischen Bytes, offline `verify_receipt`; Signatur oben | Signatur im Kern: NZ-2/NZ-15 |
| DEC-12 | Restriction ohne Override | Override-Flag: untergräbt gesetzliche Aufbewahrung |
| DEC-13 | Retention verlängert über Restriction, nicht über `due`-Änderung (FIX-22) | veränderliches `due`: Packgruppe wäre falsch platziert |
| DEC-14 | Guards zweistufig: Stufe A exakt beim Aufrufer, Stufe B Delta im RAM | exakte Prüfung im Sequencer: Plattenzugriff unter Rang 1 |
| DEC-15 | Idempotenz als implizite `Absent`-Guard auf `SYS_IDEM` | Token-Index im Sequencer: RAM ∝ aktive Tokens |
| DEC-16 | Branches clusterlokal, O(1), Merge ≤ `max_merge_ops` (FIX-33) | globaler Commit-DAG/Inhaltsadressierung: verbindet Löschdomänen; unbegrenzter Merge: bricht Change-Limit |
| DEC-17 | Branch-Metadaten in `SYS_BRANCH_S`/`SYS_BRANCH_P` (FIX-32) | ein gemeinsamer Space: verletzt `INV-SEAL-1` |
| DEC-18 | `history_floor` aus `HistoryPolicy`; `AppendOnly` nie für `Personal` (FIX-31) | implizite unbegrenzte Historie: Personenbezug bleibt physisch |
| DEC-19 | Ein-Wahrheits-Commit; Indizes projizieren idempotent aus dem Feed | 2PC/Kompensation: Komplexität, Teilzustände |
| DEC-20 | Publish-Record (`head`) statt Mega-Change | Mega-Change: sprengt Limits; Sichtbarkeit pro Chunk: Halbzustände |
| DEC-21 | Index-Partition = Packgruppe; Generationsschlüssel je Partitionsgeneration | globaler Index: Vollrebuild; Index je Cluster: Dateiflut |
| DEC-22 | `on_gate` nur Beschleuniger, Kern-Gate ist Barriere (FIX-11) | `on_gate` als Barriere: Owner-Ausfall ⇒ Datenleck |
| DEC-23 | Kanonischer Vektor F16 (F32 optional); Codes abgeleitet (FIX-12) | nur quantisiert: kein sinnvoller Rerank, Neuaufbau braucht Re-Embedding |
| DEC-24 | Modi A/B/C je nach Größe und Personenbezug (FIX-19) | Klartext-Embeddings in geteilten Partitionen: bricht `INV-SEP-2`; kein Index für große Akten: unbrauchbar |
| DEC-25 | Pseudonymisierung vor Korpus-Indexierung; Tresor im Personencluster | Klartext im Korpus: Löschung = Korpus-Rebuild |
| DEC-26 | `pseudonym_scope`/`placeholder_retention` als Policy (FIX-39) | feste Wahl: entweder Kohärenz oder Verknüpfbarkeit verloren |
| DEC-27 | Identity-Directory mit zufälligem `entry_tag` und Provider-`keyed_mac` (FIX-23) | Key-ID aus ClusterId: Zirkelschluss |
| DEC-28 | Zeitreise nur über gepinnte Generationen; Erase überstimmt (FIX-15) | Indexhistorie unbegrenzt: Schlüssel alter Generationen lebten fort |
| DEC-29 | Gewichte beschreiben pseudonymisierten Text; Reserve für Auflösung (FIX-16) | Budget nach Auflösung allein: Items erst nach Entsiegelung verwerfbar |
| DEC-30 | `Strict` verbietet Acknowledgments; ein Profil statt zwei (FIX-24) | Acknowledgments in `Strict`: Restrisiko ohne Prüfbarkeit |
| DEC-31 | Ein Anker-Port; `FileAnchor` `Weak`, `Strong` per Plugin | Anker im Kern: externe Abhängigkeit |
| DEC-32 | Frist-Arithmetik mit Reserve: `max_delay ≤ 0,6 × D` (FIX-30) | 0 Reserve: jede Providerverzögerung bricht die Frist |
| DEC-33 | 14 Crates in vier Ebenen, Retrieval-Crates persistenzneutral | Crate je Adapter: Phantom-Topologie |
| DEC-34 | `contextra-mvcc` = `EpochPinRegistry`; `contextra-checkpoint` = Snapshot-Set-Register | eigene Transaktionslogik: duplizierte Zeitachsen |
| DEC-35 | Parsing untrusted Formate außerhalb des Workspace | Parser im Kern: Angriffsfläche |
| DEC-36 | `PiiClassifier` regelbasiert offen, NER als Plugin | NER im Kern: Modellabhängigkeit |
| DEC-37 | `EgressPolicy` Pflicht, Standard `DenyExternal` für `Personal` | Opt-in: Datenabfluss als Standard |
| DEC-38 | Kein KV-/Prompt-Cache, keine Agentenorchestrierung, kein Router | Scope-Creep; Restexposition im Receipt |
| DEC-39 | `durable-fs` über `sys` als Dateischicht (positionsbasierte Handles) | eigener VFS: doppelte Poisoning-/Atomic-Logik |
| DEC-40 | Open Core: Formate, Kern, Basis-Löschung offen; KMS/HSM, Strong-Anker, NER, Kalt-Tier kommerziell | Basis-Löschung hinter Bezahlschranke: widerspricht Mission |

---

## Anhang A — Zustandsautomaten

```
Cluster:     Live ──Sequencer──► Retiring(S) ──durabler Sync──► Retired(S) ──ER2──► Shredded ──ER6──► Purged ──cluster_trim──► (UnknownCluster)
             (Shredded nur Sealed/Cryptographic; nur vorwärts; Restrictions orthogonal; Logical-only bleibt Retired bis escalate)
Ticket:      IntentRecorded ─► [Intent erweitert] ─► Gated ─► KeyDestroying{eta} ─► KeysDestroyed ─► Purging ─► Done
             (Blocked/Incomplete orthogonal und fortsetzbar; Absturz an jeder Stelle: Roll-forward)
Gate-Hülle:  H(P1) ──PolicyRetry (neuer Link)──► Intent-Erweiterung ──► erneut ; nach 3 Retries: link_fence(H_s) ──► Intent ⊇ H_s ──► Gate
Branch:      Live ──merge──► Merged ; Live ──drop/ttl──► Dropped ; (alle sterben mit dem Cluster)
Space:       Live ──SpaceGate(S)──► Dropping(S) ──SpaceScopeFrozen + Atteste──► Dropped
Packgruppe:  Aktiv ──pack_expired──► Abwurf ──alle Tickets Done──► Entfernt   (Restricted-Cluster verbleiben)
Datei:       Building ──rename + Edit──► Lebendig ──Edit──► Obsolet ──letzte Referenz──► Unlink ──unlink_ack──► weg
Anker:       monoton; Catalog-Epoche ≥ Anker und Ledger enthält Ankerhash beim Öffnen, sonst Rollback (F-9)
Instanz:     Normal ─free_space<Reserve─► DiskFull (Löschung läuft) ─Hysterese─► Normal ; Normal ─EIO/Fsync─► Poisoned (Neustart)
             Poisoned ─Erase-Request─► Poisoned + ErasePending (Intent im Ledger) ─Neustart─► Gate
Tomb-Epoche: Open ──rotate──► Closed ──Backup-Horizont + Provider-Absent──► Retired   (Default: nur Epoche 0)
View:        Lebend ──Lease-Ablauf / Revoke──► Widerrufen (SnapshotTooOld)
Pipeline:    Intake ─Sequencer(Stamp, Retiring)─► Syncer(write+sync) ─► Publikation (Stamp-Reihenfolge) ─► Receipt ;
             Sync-Fehler ⇒ Gruppe und alle späteren Indeterminate/Poisoned, nichts veröffentlicht (INV-PIPE-2)
Partition:   Published(g) ──Löschung im Fenster──► Building(g+1) ──Attest + Publish──► Draining(g) ──Pins gelöst──► KeyDestroyed(g)
Ingest:      Staging(v n) ──Abschluss-Change (head)──► sichtbar + projiziert ; Crash vor head ⇒ unerreichbar ⇒ Orphan-Worker
Restore:     RestoreQuarantine ──Fencing + Ledger-Kette + Anker + Intents angewandt──► RestoreReconciled ──► Running
```

## Anhang B — Korrekturprotokoll R13 → R14

IDs sind stabil. R14 zitiert in §1–§10 nur einen Teil der FIX-IDs; die übrigen betreffen §11 ff. und sind hier zugeordnet.

| ID | Korrektur | Abschnitt |
|---|---|---|
| FIX-01 | Löschreihenfolge als eine kanonische Sequenz G1…ER7 | §8.5 |
| FIX-02 | „Bytes von H“ definiert; Redo-Barriere statt Segmentschnitt unter Rang 1 | §8.3, §8.6 |
| FIX-03 | Receipt-Status; Statuscaps getrennt von Disclosures | §8.9 |
| FIX-04 | Zulässige Löschziele je Protection | §8.2 |
| FIX-05 | `RewriteOutcome` ist kein Löschbeweis; nur `EraseAttestation` | §11.2 |
| FIX-06 | Persistenzverträge in `contract`-Modulen der Retrieval-Crates | §2.1 |
| FIX-07 | `checkpoint` kennt `storage` nicht (`SnapshotParticipant`) | §2.1 |
| FIX-08 | Erase überstimmt Pins, Holds, `SnapshotSet`s | §9.3, §11.5 |
| FIX-09 | Ledger-/Scope-Schlüssel aus RootKey abgeleitet | §8.9 |
| FIX-10 | Ablaufkosten einer Packgruppe (Z-5) | §1.1 |
| FIX-11 | `on_gate` ist Beschleuniger, Kern-Gate ist Barriere (R13 widersprach sich) | §11.2 |
| FIX-12 | Kanonischer Vektor Vollpräzision; Rerank gegen quantisierte Codes war sinnlos | §11.6 |
| FIX-13 | `IndexClass::Personal` → `Pseudo` (Namenskollision mit `DataClass::Personal`) | §11.5 |
| FIX-14 | `max_pack_groups`-Limit | §3.3 |
| FIX-15 | Zeitreise vs. Index-Generationen und Löschung | §11.10 |
| FIX-16 | Tokenbudget nach Pseudonym-Auflösung | §11.11 |
| FIX-17 | Frist-Konsistenz Schlüsselvernichtung/Purge | §8.4 |
| FIX-18 | `IntegrityMode` | §3.6 |
| FIX-19 | Modus C (Cluster-Index) schließt die Lücke zwischen Mikroindex und `INV-SEP-2` | §11.6 |
| FIX-20 | Mixed-Routing mit Quarantänecluster | §11.9 |
| FIX-21 | CRC32C, `verify_receipt` nur mit `Mac`-Port | §2.7 |
| FIX-22 | Retention-Verlängerung über Restriction | §3.3 |
| FIX-23 | Identity-Directory: Zirkelschluss der Entry-Key-ID beseitigt; `KeyProvider` um Entry-Schlüssel und `keyed_mac` erweitert | §8.4, §11.13 |
| FIX-24 | Ein Profil statt `OpenProfile`+`ComplianceProfile`; `Strict` ohne Acknowledgments | §11.15 |
| FIX-25 | Retry derselben `EraseRequestId` findet das Ticket | §8.5 |
| FIX-26 | Unsafe-Inseln nur `sys`, `simd` | §2.6 |
| FIX-27 | `Relaxed` nie mit `IdemToken` | §5.1 |
| FIX-28 | Fünf Erweiterungsverträge | §2.1 |
| FIX-29 | Hüllenwachstum zwischen Intent und Gate: Intent-Erweiterung, `link_fence`, `INV-LED-3` | §8.5, §8.8, §8.9 |
| FIX-30 | Fristarithmetik ohne Reserve (0,7 + 0,3): Reserve 0,1, Auslöser 60 %/80 %, `purge_deadline_min` | §8.4, §8.5, §8.6, §15 |
| FIX-31 | `history_floor`-Treiber definiert; `AppendOnly` ⊄ `Personal` | §3.6, §11.14 |
| FIX-32 | `SYS_BRANCH` verletzte `INV-SEAL-1` ⇒ `SYS_BRANCH_S`/`_P` | §4.9, §7 |
| FIX-33 | Merge als ein Change vs. unbegrenzte Größe ⇒ `max_merge_ops`, `MergeTooLarge` | §7 |
| FIX-34 | `due` vs. `due_bucket`: Gate zu `due`, Bucket nur Ko-Lokation | §3.3, §11.14 |
| FIX-35 | Logical-only-Cluster: Alarm statt unbegrenzter stiller Retired-Bestand | §8.2 |
| FIX-36 | `ArcSwap` im Kern entfernt (`RwLock<Arc<Version>>`) | §10.2 |
| FIX-37 | Zahlenfehler 3,9 → 4,0 Mio. Cluster bei 15 % von 1 GiB | §3.4 |
| FIX-38 | Consumer-Retention unter DiskFull | §11.4 |
| FIX-39 | Platzhalter-Politik (`pseudonym_scope`, `placeholder_retention`) | §11.9 |

## Anhang C — Referenzvektoren (Pflicht vor G-F)

Je Format mindestens: leeres Objekt, ein Eintrag, Eintrag mit versiegeltem Körper (Cluster- und Space-Quelle), Cluster-Abschnitt mit zwei Blöcken, Packed-Run mit drei Clustern, Staging-Run, `MiniSection`, Abschnitt mit `BranchTable` (drei Branches, einer ohne Einträge), Tombstone, Bereichsmarker, Gate-Frame mit `policy_epoch`, Control-Frame `Link`, Blob mit Loch, Catalog-Edit-Kette mit Reservierung, Gruppen-Commit und `anchor_mark`, Ledger-Kette (`tomb_id`, zwei Tomb-Epochen, ein Retire, `EraseIntent` + `EraseIntentExtend`), Redo-Gruppe mit Change-/Gate-/Control-/Branch-Frame, abgeschnittener und beschädigter Frame, nicht-kanonisches `varint`, `PartBlock` (AAD mit `partition_tag`, `generation`), `head`-Record, Directory-Vorwärts-/Rückwärtszeile, `SYS_IDEM`-Record, `SYS_BRANCH_S`/`_P`-Record, **Rohkopie-Test** (Abschnitt aus Run A in Run B, Blöcke byte-identisch, Entsiegeln gelingt). Erzeugung durch `tools/reference/generator.py` (unabhängig von Rust), Vergleich im Test `golden`. **Stand: nicht vorhanden.**

## Anhang D — Verworfene Vorschläge

Zeitlanes/kumulative Gewichte im SubIndex · mehrere parallele Redo-Logs/Striping · Branch-„Flatten“ durch Datenkopie · Langsampfad-Guards im Sequencer · persistierter Token-Index im Sequencer · Prolly-Tree/Merkle-DAG/Inhaltsadressierung im Kern · CAS-Superblock ohne Log · `mmap`/`io_uring`/GPUDirect im Kern · Token-IDs, KV-Cache-Tensoren, feste Seitenstruktur · gelernte Cross-Objekt-Affinität als Layout · Slab-Sub-Allokator · Segmentschnitt je Gate · Aufrufer-vergebene ClusterId · `eu_tag` im Klartext-Header · Poison bei ENOSPC · `rekey_clone` · Indizes (ANN, BM25, Graph) im Kern · MCP-Server, Tokenizer, Embedding-Modelle im Kern · benutzerdefinierte Merge-Operatoren im Kern · persistente Vollindizes je Mandant ohne Partitionierung · ein Mikroindex je Cluster für alle Größen · gerichtetes Re-Wiring ohne Rebuild · Pseudo-2PC zwischen Storage und Indizes · geteilte Hyperkanten über Löschdomänen · globales persistiertes Termwörterbuch · Nutzer-Lernsignale in globalen Gewichten · synchrone Indexpflege im Commit-Pfad · Sequencer-Selbstgate einer nicht vom Intent gedeckten Hülle · Entry-Key-ID aus ClusterId · zwei Profilsysteme (`OpenProfile` + `ComplianceProfile`) · Löschung erst bei Bucket-Ende · quantisierter Vektor als alleinige Wahrheit · mehrteiliger atomarer Merge über Change-Limits.

# Schnittstellenspezifikation: `contextra-storage` als Speicherkern

**Status:** Architekturvertrag für die Entwicklung nach Storage-Gate G-0.  
**Normative Basis:** [`crates/contextra-storage/SPEC.md`](../crates/contextra-storage/SPEC.md), insbesondere §3, §6–§12 und §15. Bei Konflikt hat die Storage-Spezifikation Vorrang.

## 1. Entscheidung und Zielbild

`contextra-storage` ist der einzige persistente Wahrheitskern. Es ersetzt den
bisherigen LSM-/WAL-Kern `contextra-store`; der Altkern bleibt bis G-CUTOVER
eingefroren und wird anschließend archiviert. Die permanente Async-Grenze
gehört zu `contextra-storage-bridge` (`AsyncStorage`); ein Legacy-KV-Adapter
ist ausdrücklich befristet. Der Kern bleibt synchron und eingebettet.

```text
 API / MCP / Python / Agenten
             │ async boundary, policy, auth, DTOs
             ▼
 contextra-engine ── contextra-records ── AsyncStorage
             │              │
             │              ├── canonical Change/View/Erase/Branch API
             ▼              └── feed, SegmentOwner, Receipt
  vector / text / graph / checkpoint / privacy / audit
             │  durable feed + watermark + filter_visible
             ▼
       contextra-storage (synchroner Speicherkern)
             │
 storage-ports / fs / keys / crypto-ref / anchor
```

### 1.1 Namen, Abhängigkeiten und verbotene Pfade

* **Kern-API:** `contextra-storage` und `contextra-storage-ports` bleiben die
  fachlich maßgeblichen Crates. Ihr `Stamp` ist nur zusammen mit `InstanceId`
  gültig.
* **Produkt-Fassade:** `contextra-storage-bridge::AsyncStorage` ist die
  dauerhafte, begrenzte Async-Fassade. Das bisherige `contextra-store` bleibt
  bis G-CUTOVER unverändert/eingefroren und wird danach archiviert. Ein
  `LegacyKvAdapter` ist feature-gated und befristet; er exportiert keine
  LSM-/WAL-/SSTable-/`TxId`-Semantik als Ziel-API.
* **Keine direkten Kernimporte:** Nur `contextra-storage-bridge` und das neue
  `contextra-records` importieren den Kern. Fach-, API- und
  Algorithmus-Crates hängen von ihren engsten Records-/Domain-Ports bzw. der
  Engine ab.
* **Kein zweiter Persistenzpfad:** `contextra-ports::StorageEngine`,
  `contextra-mvcc`, `contextra-store`-LSM und `contextra-durable-fs` sind nach
  Cutover nicht Autorität für kanonische Daten, Sichtbarkeit oder Löschung.
  Ihre APIs werden nicht 1:1 nachgebaut.
* **Keine globale Transaktion über Derivate:** Der atomare `Change` umfasst
  kanonische Records derselben Cluster-Löschdomäne. ANN, BM25, CSR und
  Checkpoint-Manifest-Kopien folgen über Feed, Watermark, Delta und
  `filter_visible`; sie sind nie stillschweigend atomar mit dem Kern.

### 1.2 Gemeinsames Integrationsprofil

Die Fassade definiert vier Ports. Die Persistenztypen gehören in
`contextra-storage-ports`; `AsyncStorage` und die Record-Abbildung liegen in
`contextra-storage-bridge` beziehungsweise `contextra-records`, nicht in
`contextra-types`:

| Port | Verantwortlichkeit | Unverhandelbare Semantik |
|---|---|---|
| `CanonicalStore` | `open`, Spaces, Cluster, `apply`, `view*`, Branches, Links, Erase | `Change` mit `IdemToken`; `Durable` für fachliche Daten und Löschung; Ergebnis `Receipt { instance_id, stamp, durable }` |
| `CanonicalView` | `get`, range/cursor, `page`, `filter_visible` | View-Lease; `view_durable` als Query-Ausgangspunkt; Gate vor Payload und nochmals vor Ausgabe |
| `DurableConsumer` | Redo-Feed, Cursor und Watermark | mindestens einmal, Stamp-Reihenfolge, nur bis durable position; `Lagged` erzwingt Vollaufbau statt Lückenschluss |
| `ErasureCoordinator` | Ticket, Receipt, registrierte Owner, Export | Kernreceipt plus Owner-Atteste; `Blocked`/`Incomplete` nie zu `Complete` glätten |

Ein fachlicher Schlüssel ist immer `(space, cluster, branch, user_key)`;
`SpaceId` ist kein Mandanten- oder Personenersatz. `ClusterId` wird vom Kern
erzeugt, ist Lösch-, Schlüssel-, Versions- und Platzierungsdomäne. Die obere
Schicht entscheidet Tenant, Rechte, Retention und Zuordnung Person→Rootcluster
vor jedem Aufruf; der Kern erzwingt die technische Klassifikation
`DataClass::{Personal,NonPersonal}` und die Schutzkombination.

### 1.3 Fehler-, Retry- und Abfragevertrag

* `Conflict`, `BelowFloor`, `SnapshotTooOld`, `Lagged`, `IndexLag`,
  `Restricted`, `Erased`, `ClusterRetired`, `KeyUnavailable` und `KeyLost`
  bleiben unterscheidbar bis zur äußeren API. Keine Schicht darf sie als
  „nicht gefunden“ normalisieren.
* Nach begonnenem Write/Sync ist nur `Indeterminate` erlaubt. Der Aufrufer
  öffnet neu und löst mit **demselben** `IdemToken` auf; blindes Retry ist
  verboten. `Relaxed` ist nur für explizit flüchtige/nicht-rechtserhebliche
  Daten zulässig, nie für Erase, Audit oder Kanonika.
* Query-Fassaden öffnen `view_durable()`; historische Reads benutzen
  `view_at` und akzeptieren `BelowFloor`. Jede View/ jeder Cursor behandelt
  Lease-Ablauf als Fehler, nicht als leeres Ergebnis.
* Jede persistente Ableitung registriert einen `SegmentOwner`, deklariert
  Segmentklasse, Schlüssel-/Löschscope, Ressourcenobergrenzen und Lineage.
  Nicht registrierte Kopien erscheinen als Receipt-Residual und verhindern
  systemweites `Complete`.

## 2. Crate-spezifische Verträge

„Direkt“ bedeutet eine erlaubte Compile-Abhängigkeit auf den neuen Kern; alle
anderen Crates integrieren über die Engine/Fassade.

| Crate | Zielrolle und zulässliche Schnittstelle | Verpflichtungen bei Cutover |
|---|---|---|
| `contextra-storage-ports` | Direkte, synchrone Verträge und redigierte Werttypen des Kerns. | Fassade aus §1.2 vollständig und semver-stabil machen: `InstanceId` in allen Positionen/Receipts, View, Feed, Branch, Erase, Owner und Checkpoint. Keine Produktpolitik oder Async-Traits. |
| `contextra-storage` | Einziger kanonischer Speicherkern. | SPEC R9.1 vor jeder Ökosystemmigration fertigstellen; nur `Change`/View/Branch/Erase/Owner-Register als öffentliche Speichersemantik freigeben. |
| `contextra-storage-fs` | Produktionsadapter für `Fs`, Locking und geordnete Syncs. | Ersetzt direkte Nutzung von `contextra-durable-fs`/`contextra-sys` im Kernpfad; `FaultFs`-Konformanz und Platform-Durability nachweisen. |
| `contextra-crypto-ref` | Referenz-`Cipher`/`Mac`/`Kdf`, Test-KeyProvider. | Nur Referenz und Konformanz, keine Anwendungsschlüssel oder DeletionProof-Politik. |
| `contextra-keys-file` | Einzelhost-`KeyProvider`. | Cluster-Key-Lebenszyklus, `destroy_many`, Tomb-Epochen und `KeyUnavailable`/`KeyLost` exakt nach Kernvertrag. |
| `contextra-keys-cohort` | Optionaler skalierender Provider. | Cohort-Rewrap ist Providerstrategie, keine sichtbare Kern- oder RAG-Semantik; gleiche Erase-Garantien wie `keys-file`. |
| `contextra-storage-bridge` | Zeitlich begrenzte Import-/Migrationsbrücke. | Read-only Legacy-Import bzw. expliziter, geprüfter Export; nie Dual-Write und nie Legacy-Fallback auf einer neuen Query. Nach Cutover entfernen. |
| `contextra-store` | Eingefrorener LSM-Altbestand. | Keine neuen Verbraucher oder Funktionen; einmaliger verifizierter Forward-Export, danach Archivierung. Der befristete `legacy-kv`-Adapter liegt in `contextra-storage-bridge`. |
| `contextra-ports` | Produktweite, algorithmische Ports. | `StorageEngine` nicht erweitern, um Kernsemantik zu simulieren. In P2 als deprecated markieren und stattdessen schmale `Canonical*`-Adapter exportieren; `Scanner`/`Purger` auf Owner-Atteste umstellen. |
| `contextra-types` | Fach-IDs, DTOs, Budgets. | `TxId`/Seq als Legacy markieren. Neue öffentliche DTOs führen `(instance_id, stamp)`, `ClusterRef`, Erase-Stage und nicht nur einen u64-Stand; keine Storage-Geheimnisse/Keys. |
| `contextra-core` | Reexport-/Kompatibilitätsschicht. | Keine Reexports von `contextra-mvcc`-Snapshot oder Legacy-Storage als Kernvertrag. Reexportiert ausschließlich stabile Fassade-DTOs, nicht innere Run-/Redo-Typen. |
| `contextra-mvcc` | Übergangsmodul für rein lokale Algorithmik. | Nicht mehr Quelle von globaler Versionsordnung, SSI oder historischen Snapshots. Seine Nutzer werden auf View+Guards migriert; danach aus Produktionsgraph entfernen oder als eigenständige RAM-Hilfe begrenzen. |
| `contextra-durable-fs` | Altadapter/Kompatibilitätscode. | Nicht im neuen Kernpfad. Seine Fault-Tests werden nach `contextra-storage-fs` portiert, danach Crate entfernen oder ausschließlich externes Hilfspaket machen. |
| `contextra-sys` | OS-/Unsafe-Insel außerhalb des Kernformats. | Kein `mmap`, `O_DIRECT` oder direkter Durability-Aufruf im Kern. Darf für HostProbe und nichtkanonische Indexbeschleuniger dienen; Capability-Ausfall muss korrekt bleiben. |
| `contextra-engine` | Einziger fachlicher Orchestrator für CanonicalStore. **Direkt** auf die neue Fassade. | Mappt Collection-CRUD in clustergebündelte `Change`s mit Guards/Idempotenz. Erzeugt Root-/Dokument-/Derivationscluster und `link`; baut Query über `view_durable`, Watermark+Delta und doppeltes `filter_visible`. Entfernt `DurabilityMode`, direkte LSM-Pressure und globale Tx-Rollbacks. |
| `contextra-db` | Produktdatenbank-/Query-Fassade über Engine. | Keine eigene Persistenz, kein `inner_storage`. Leitet `ClusterRef`, View-Position, `IndexLag` und Erase-Status bis zur Public API; Volatile Vault bleibt ausdrücklich `Volatile` und nicht Receipt-relevant. |
| `contextra` | Öffentliche Rust-API und Builder. | Konfiguration nimmt Storage-Ports/KeyProvider/Anchor sowie klare Durability-/Erase-Optionen an. `open` darf erst nach Restore-/Fencing-Prüfung Write-Zugriff geben; Legacy-LSM-Tuning verschwindet aus Collection-Profilen. |
| `contextra-ingest` | Reine Extraktion und Chunk-Drafts. | Bleibt I/O-frei gegenüber dem Kern. Übergibt an Engine einen validierten Ingest-Plan mit DataClass, Tenant-Kontext, Lineage, Header/Weights und einem Cluster-Scope; kein eigener Dokument-Write. |
| `contextra-vector` | Abgeleiteter ANN-/Vektorindex. | Registrierter `SegmentOwner`: entweder clusterlokale `ClusterSections` oder bounded `Rebuildable`. Feed idempotent bis Watermark; Tenant-/ACL-Filter vor Traversierung; Gate-Generation vor Distanz und Ausgabe; Kandidaten stets `filter_visible`. Kein globaler HNSW als Crypto-Erase-Versprechen. |
| `contextra-text` | Abgeleiteter BM25-/Inverted-Index. | Gleiches Owner-/Feed-/Watermark-Modell. Jeder Posting-Eintrag trägt Cluster/Generation; Gated Einträge vor Score ausschließen, Kandidaten nachvalidieren. Persistente Postingsegmente attestieren `rewrite_without` und `verify_absent`. |
| `contextra-graph` | Abgeleiteter Graph/CSR/Community-Index. | Kanten, Hyperkanten und Communities erhalten Quellen-Cluster/Generation. Feed-Consumer ist Owner; Gate vor Expansion und Ausgabe. Geteilte CSR/Community-Strukturen sind `Rebuildable`, nicht fälschlich clusterlokal. |
| `contextra-rank` | Reine Ranking-/Fusion-Algorithmen. | Keine Persistenzabhängigkeit. Nimmt nur sichtbare, mit View-Position gebundene Kandidaten entgegen; Header/Weights und `page` für Budget-Packing nutzen. |
| `contextra-adapt` | Reine Controller/Bandits. | Keine kanonischen Entscheidungen/Events direkt speichern. Die Engine persistiert ihre auditierbaren Zustände als normale clustergebundene Changes; Metriken enthalten keine IDs oder Inhalte. |
| `contextra-cognition` | Konsolidierung, Summaries, Ableitungen. | Ergebnisse in neue/abgeleitete Cluster schreiben und per `link` an Quellen hängen. Optimistische Konflikte über Kern-Guards statt `TxId`/SSI; Konsolidierungsjobs sind idempotente Feed-Consumer und respektieren Gate/Branch/Floor. |
| `contextra-checkpoint` | Checkpoint-/Restore-Orchestrierung. **Direkt** auf Kern-Checkpoint/Views. | Ersetzt Datei-/Hardlink- und Tx-Rollback-Authority durch durablen View-Checkpoint mit `(instance_id, stamp)`, Pins/Leases und Kernel-Restore-Reconcile. Eigene Manifeste nur als Owner-registrierte Derivate; Restore darf Gate/Ledger/Anchor nie zurückdrehen. |
| `contextra-crypto` | Anwendungscrypto, Audit-/Proof-Bridge, Egress-Keymaterial. | Kein zweites WAL-/Revocation-Log für kanonische Löschung. Erstellt versionierte v15-DeletionProofs ausschließlich aus `ErasureReceipt` plus allen Owner-Attesten; fehlende Atteste = `Blocked`/`Incomplete`. KeyProvider verbleibt in den Storage-Key-Crates. |
| `contextra-privacy` | Politik für Klassifikation, Directory und Egress. | Prüft DataClass/Tenant/Rechte vor Change. Implementiert `IdentityDirectoryOwner` als `MixedPerSubjectKey` mit EntryDEK, Reverse Lookup, Feed-Watermark, Gate, rewrite und Attestation; unregistriertes Personenmapping ist verboten. |
| `contextra-kvcache` | Nichtkanonischer Prompt-/KV-Cache. | Standard: flüchtig, außerhalb Receipt. Bei Persistenz verpflichtend registrierter SegmentOwner mit clusterlokalem Scope und `rewrite_without`; keine globale Prefix-Deduplication, keine Löschbehauptung ohne Attest. |
| `contextra-agent` | Workflow, Audit und Task-Zustand. | Sitzung ist Schlüsselpräfix/TTL-Branch im Personen-/Fallcluster, nicht automatisch Cluster. Step-Commit, Audit und Ergebnis sind ein durable, idempotenter Change; Checkpoint nutzt View/Branch. DLQ und Tool-Output entweder volatile oder registrierter Owner. |
| `contextra-router` | Modellwahl, Anfrage-Steuerung. | Persistiert nur über Engine und nur wenn fachlich nötig. Query erhält View-Position und darf `IndexLag` nicht durch stillen Teiltreffer verdecken; Routing-Cache ist flüchtig oder Owner-registriert. |
| `contextra-infer-candle`, `contextra-infer-onnx`, `contextra-infer-ollama` | Modelladapter. | Keine direkte Storage-Abhängigkeit. Modell-/Embedding-Caches folgen der `contextra-kvcache`-Regel; Modellfingerprints gehen als Header/Provenienz in Engine-Changes. |
| `contextra-mcp` | Async-Protokoll- und Toolgrenze. | Mappt typed Store-Ergebnisse auf stabile RPC-Fehler inklusive Retry-Hint/Receipt-Status. Delete gibt Ticket/Stage/Residual aus, nie pauschal „gelöscht“. Search wartet nicht auf Indexaufbau, sondern liefert `IndexLag` oder dokumentierten Fallback. |
| `contextra-wire` | Wire-/IPC-DTOs. | Versioniert DTOs für `InstanceId+Stamp`, View-/Resume-Cursor, Erase-Ticket/Stage, `IndexLag` und redigierte Fehler. Keine `ClusterId`, Schlüssel oder Payloads in Diagnosefeldern. |
| `contextra-py` | Python-Fassade. | Blocking-Bridge auf die async Produkt-API, nicht direkt in den synchronen Kern. Exponiert View-/Receipt-/Ticket-Datentypen und unterscheidet Erased, Lagged, Conflict und Indeterminate; entfernt LSM-spezifische Stats/Tuning. |
| `contextra-audit-export` | Audit-/Compliance-Export. | Liest auditierbare kanonische Records über eine durable View und exportiert Receipt/Owner-Atteste unverändert. Kein Export kann `Complete` hinzufügen oder personenbezogene Ledger-/Keydaten offenlegen. |
| `contextra-avv-generator` | Dokumentgenerator. | Nutzt nur konfigurierte, redigierte Capability-/Receipt-DTOs; keine direkte Storage-Abhängigkeit und keine Rechtsbehauptung aus technischem Stage allein. |
| `contextra-license` | Lizenz-/Feature-Gate. | Vor Storage-Aufruf in der oberen Fassade prüfen; darf Erase, Recovery, Gate oder Owner-Attestation niemals verhindern oder verändern. |
| `contextra-sandbox` | Tool-Sandbox. | Ergebnisse standardmäßig flüchtig. Persistenz ausschließlich über Agent/Engine mit DataClass/Cluster/Lineage; Sandbox-Dateien sind keine versteckten Indexsegmente. |
| `contextra-simd` | Reine numerische Primitive. | Keine Storage-Integration; Vector nutzt es nur nach der Sichtbarkeitsbarriere. |
| `contextra-testkit` | Referenzmodelle und Konformanztests. | Ersetzt Legacy-`InMemoryStorageEngine`-Autorität durch deterministische CanonicalStore-/Feed-/Owner-Fakes sowie Crash-, Gate- und Watermark-Konformanzsuiten. |

## 3. Verbindliche Daten- und Ablaufgrenzen

### 3.1 Canonical Record Envelope

Die Engine allein kodiert die fachliche Schlüsselnamensgebung. Jeder persistente
kanonische Record hat mindestens: `schema_version`, `record_kind`, `tenant_scope`
(nur oberhalb des Kerns geprüft), `DataClass`, `producer_id`, `source_stamp` für
Ableitungen, Trust-Byte im Header sowie optional gültige Fachzeit. Körper bleiben
opaque. Tokengewichte belegen einen der vier Kern-Slots und werden über
`header`/`weights`/`page` gelesen. Persönliche Daten verlangen einen Sealed
Cluster; Plain ist dafür vor dem ersten I/O abzuweisen.

`DerivedFrom` ist eine Kern-Clusterverknüpfung, kein bloßes Feld im JSON. Der
Write eines abgeleiteten Clusters und seine Links erfolgen in einem durable
Change. Dadurch zieht ein Gate des Rootclusters Quellen, Dokumente, Summaries,
Embeddings und registrierte Derivate nachvollziehbar ein.

### 3.2 Ingest, Indizierung und Query

1. Ingest klassifiziert, autorisiert und plant Cluster/Lineage oben.
2. Engine schreibt Quelle, Chunks, Metadaten, Vektor-/Text-Quelldaten und
   Graph-Deltas als einen `Durable Change` derselben Löschdomäne.
3. Vector, Text, Graph, Cognition und persistente Caches konsumieren den
   durable Feed mindestens einmal und idempotent. Ihre Watermark wird erst
   nach vollständiger Anwendung veröffentlicht.
4. Query bindet `view_durable`. Sie nutzt Basisindex plus Delta bis zur View
   oder einen expliziten Fallback. ACL/Tenant-Filter laufen vor jeder
   Indexexploration, Kern-Gate/Generation vor Payloadarbeit und
   `filter_visible` nochmals vor Rückgabe.
5. Ist dies nicht möglich, lautet das Ergebnis `IndexLag`/Retry oder ein
   ausgewiesener Fallback — nie ein unvollständiges Erfolgsergebnis.

### 3.3 Löschung, Export und Backups

Der einzige fachliche Löschaufruf ist `ErasureCoordinator`. Er nimmt eine
`EraseRequestId`, startet Kern-Gate/KeyDestroy/Purge und beobachtet die
registrierten Owner. `on_gate` beschleunigt, ist aber nie Sichtbarkeitsbeweis.
Erst `verify_absent`-Atteste mit passender Generation und Scope erlauben den
Receipt-Fortschritt. Außenflächen geben die gewünschte `EraseLevel`, den
tatsächlichen Stage (`Gated`, `KeysDestroyed`, `Purged`, `Done`), Blocker und
Residuen aus. Ein Proof-/Audit-Export signiert diese Daten nur nach, er ersetzt
sie nicht.

Backup, Checkpoint und Restore unterliegen Ledger/Anchor/Fencing. Ein Restore
ist kein Zeitreise-API und darf gelöschte Daten nicht wieder sichtbar machen.
Ein wechselnder `InstanceId` verlangt bei Indizes Vollaufbau oder einen
verifizierten Importcursor.

## 4. Reihenfolge, Abnahme und Stilllegung

| Phase | Voraussetzung | Ergebnis / harte Abnahme |
|---|---|---|
| P0 – Kern-Gates | Keine Ökosystemumstellung | G-0 bis G-F der Storage-SPEC: Formats, Recovery, Erase, Extension- und Referenzvektoren. |
| P1 – Verträge | P0-API stabil | `storage-ports`, `contextra-storage-bridge::AsyncStorage` und `contextra-records`; Compile-Layer-Test: keine unzulässigen Kernimporte. |
| P2 – Kanonika | P1 | Engine/DB/Ingest/Privacy schreiben und lesen nur CanonicalStore; Differential Import aus Legacy, kein Dual-Write. |
| P3 – Derivate | P2 | Vector/Text/Graph/Cognition/KV-Cache als Owner mit Feed, Watermark, Gate-Filter, Delta/Fallback und Erase-Attest. |
| P4 – Betriebsflächen | P3 | Checkpoint, Agent, Crypto-Proof, MCP, Python und Exporte führen Position/Ticket/Fehler vollständig weiter. |
| P5 – Cutover | P4 | Neue Instanzen erzeugen keine Legacy-WAL/SSTables. Migrationsreport, Restore-/Erase-Matrix und Last-/Fault-Nachweise bestehen. |
| P6 – Entfernung | P5 stabil | `contextra-store`-LSM, `StorageEngine`-Produktpfade, Legacy-`TxId`-Snapshots und den `LegacyKvAdapter` entfernen; `AsyncStorage` bleibt die dauerhafte Fassade, nur das Offline-Migrationswerkzeug wird archiviert. |

Jede P3-Abnahme enthält mindestens: Feed-Duplikat/Retry, `Lagged`-Vollaufbau,
Gate während Query, Owner-Timeout/Fencing, View-Lease/Floor, Crash zwischen
durable Change und Watermark, sowie Erase-Receipt mit fehlendem Attest. Jede
P2-/P5-Abnahme enthält einen `Indeterminate`-Retry mit identischem Token und
Restore ohne Datenwiederkehr. Leistungs- oder Löschversprechen sind erst nach
den in der Storage-SPEC benannten Gates zulässig.

## 5. Explizit verworfene Migrationen

Nicht zulässig sind: ein Adapter, der `StorageEngine` auf Prefix-Scans des
Kerns emuliert und dadurch Cluster/Gate verliert; Dual-Write nach LSM und neuem
Kern; eine globale DocId-/Tenant-Key-Partition als Ersatz für Cluster; direkte
Index-Persistenz ohne Owner; Indexresultate ohne View-Filter; Umdeutung von
`Lagged`, `Erased`, `KeyLost` oder `Indeterminate` zu leeren/erfolgreichen
Ergebnissen; und eine „Rollback“-Funktion, die Ledger, Anchor oder Erase
zurücksetzt.

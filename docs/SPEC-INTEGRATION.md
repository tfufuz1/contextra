# SPEC-INTEGRATION — Gesamtsystem-Schnittstellenspezifikation

| | |
|---|---|
| Revision | I1.1 — Integrationsentwurf, Gate G-INT-0 offen |
| Gegenstand | Verträge zwischen `contextra-storage` und dem übrigen Workspace |
| Normativer Kern | [`crates/contextra-storage/SPEC.md`](../crates/contextra-storage/SPEC.md), R9.1 |
| Rangfolge | Storage-SPEC §15 → §12.8 → übrige Storage-SPEC → dieses Dokument → Crate-Spezifikationen |

## 1. Verbindliche Entscheidungen

1. `contextra-storage` ist der synchrone, eingebettete alleinige
   Wahrheitskern. `contextra-store` bleibt bis G-CUTOVER der **eingefrorene**
   LSM-Altbestand; es wird nicht zur neuen Fassade und erhält keine neuen
   Verbraucher.
2. `contextra-storage-ports` ist die einzige Persistenz-Portwelt. Die
   parallelen Persistence-/Krypto-/FS-Ports in `contextra-ports`,
   `contextra-crypto` und `contextra-durable-fs` werden ausschließlich über
   Adapter zurückgebaut. Insbesondere ist deren domänenorientierter
   `KeyProvider` nicht kompatibel mit dem cluster-/epoch-orientierten
   Storage-`KeyProvider`.
3. `contextra-storage-bridge` besitzt dauerhaft `AsyncStorage` mit begrenztem
   Blocking-Pool, Backpressure und urgenter Erase-Lane. Sein
   `LegacyKvAdapter` ist optional, `legacy-kv`-gefeaturet und wird bei
   G-CUTOVER entfernt.
4. Das neue Ring-1-Crate `contextra-records` besitzt Space-Katalog,
   Schlüsselcodecs, Kopfversionen, `ScopeResolver`, nicht konstruierbares
   `ScopeBinding`, `RecordStore` und `TrackedView`. Domain-Crates sprechen
   diese Schicht, nicht `Storage` direkt.
5. ANN, Text und Graph werden `SegmentOwner` plus idempotente
   Feed-Konsumenten mit Watermark. Das alte 4-Index-2PC, Intent-Records und
   kompensierende Aktionen entfallen.
6. Jede Query bindet eine `view_durable()`-Sicht. Tenant-/Scope-Filter laufen
   vor Indexexploration; Gate/Generation vor Payloadarbeit; jeder Treffer
   passiert `filter_visible` vor Rückgabe. Lücken sind `IndexLag` oder ein
   expliziter Fallback, nie ein stiller Teiltreffer.
7. Kein Dual-Write. Migration ist ein nachweisbarer Forward-Export; der neue
   Pfad schreibt niemals zurück in den Altkern.

## 2. Fassadenvertrag

Der Kern bleibt synchron. `AsyncStorage` verschiebt blockierende Aufrufe in
einen begrenzten Pool. Cancellation nach Übergabe eines Changes erzeugt keinen
zweiten Write: der Aufrufer löst ausschließlich mit demselben zufälligen
`IdemToken` auf. Queue-Überlauf ist `Backpressure`, nicht unbeschränkte
Speicherung; Löschaufträge nutzen eine reservierte Lane.

Die öffentliche Persistenzoberfläche enthält mindestens:

| Bereich | Vertrag |
|---|---|
| Write | `apply(Change) -> Applied(Receipt) | AlreadyApplied(Receipt)`; `Receipt` enthält immer `InstanceId`, `Stamp`, `durable` |
| Read | `view`, `view_durable`, `view_at`; View-Lease, Cursor/`page`, `body`, `filter_visible` |
| Scope | `Spaces::ensure`, `Clusters::register/info/link`; Space-Optionen unveränderlich und Namen versioniert |
| Erase | Ticket, Stage, Receipt und Verify; Gate → KeyDestroy → Purge, Owner-Atteste zwingend |
| Feed | mindestens einmal, Stamp-Reihenfolge bis durable position; `Lagged` verlangt Vollaufbau |
| Restore | Checkpoint/Restore nur mit Ledger-/Anchor-Reconcile und Fencing; kein In-place-Rollback |

`Stamp` wird außerhalb des Kerns ausschließlich zusammen mit `InstanceId` als
`DurablePosition` gespeichert. `Indeterminate`, `Lagged`, `Erased`,
`BelowFloor`, `SnapshotTooOld`, `KeyUnavailable`, `KeyLost`, `Restricted` und
`IndexLag` sind eigenständige Außenfehler und dürfen nicht zu „leer“ oder
Erfolg normalisiert werden.

## 3. Record-, Scope- und Löschmodell

`contextra-records` registriert versionierte Spaces nach
`ctx.<family>.<personal|sealed|plain>.v<n>`. `Personal` verlangt `Sealed` und
cluster-sealed; Space-IDs werden nie fest codiert. Schlüssel sind binär,
ordnungserhaltend und enthalten weder Tenant noch Cluster: Routing erfolgt über
den Scope.

`ScopeResolver` ist die einzige Konstruktion eines `ScopeBinding`. Er prüft
den im Cluster gespeicherten zufälligen `tenant_tag` gegen den Request-Kontext;
rohe `ClusterId` ist für Domain-Code nicht ausreichend. Personen verwenden das
Storage-SPEC-Muster Wurzelcluster plus `IdentityDirectoryOwner` mit per-subject
EntryDEK. Jeder weitere Kennung→Cluster-Pfad muss dieselbe Owner-/Attestierung
haben.

Die gewählte Clustergranularität bestimmt die Löschgranularität: Bei Sealed
Clustern ist `range` nur logisch. Dokument-Crypto-Erasure erfordert einen
Dokumentcluster und bleibt innerhalb der Gate-Fanout-Grenze. Mehr als 4096
Cluster benötigen orchestrierte Teil-Tickets; bis zu deren Abschluss ist der
Gesamtstatus nicht `Complete`.

## 4. Index- und Engine-Vertrag

Ein `DerivedIndex` verarbeitet nur durable Feed-Batches und veröffentlicht
seine Watermark erst nach vollständiger, idempotenter Anwendung. Der
Indexbesitzer deklariert `SegmentClass`, Scope, Schlüsselstrategie,
RAM-/Scratch-/Rewrite-Grenze und stellt `on_gate`, `rewrite_without` und
`verify_absent` bereit. Geteilte HNSW-, Posting-, CSR- und Community-Strukturen
sind `Rebuildable`; sie dürfen keine Crypto-Erasure pro Cluster behaupten.

`contextra-engine` ersetzt `DbTransaction` durch einen `IngestUnit`:
Eingabe/Scope/DataClass prüfen, außerhalb des Commits vorbereiten,
`ChangeBuilder` mit expliziten Key-/Range-Guards füllen, einmal durable
committen. `TrackedView` ersetzt SSI-Lesesets; jede Lesung, die einen Write
beeinflusst, braucht einen Guard. Spekulative Konsolidierung verwendet
TTL-Branches und direkten Child→Parent-Merge.

## 5. Migration

| Phase | Ergebnis |
|---|---|
| G-INT-0 | Storage-Ports gegen R9.1 schließen; Fassaden-/Provider-Signaturen eingefroren |
| I1 | `AsyncStorage`, `contextra-records` und Layer-Tests; kein Dual-Write |
| I2 | Engine/DB/Ingest/Privacy auf RecordStore; kanonischer Forward-Import |
| I3 | Vector/Text/Graph/KV/Checkpoint als Owner/Consumer mit Watermark und Attest |
| I4 | Agent, MCP, Python, Crypto-Proof und Exporte tragen Position/Ticket/Fehler vollständig |
| G-CUTOVER | Neue Instanzen ohne LSM-WAL/SSTables; Fault-/Restore-/Erase-Matrix bestanden |
| I5 | LegacyKvAdapter und `contextra-store`-Produktpfad entfernen/archivieren |

## 6. Offene Punkte für G-INT-0

| ID | Lücke / Entscheidung nötig |
|---|---|
| INT-01 | `storage-ports` weicht noch von R9.1 ab: `Receipt` ohne InstanceId, fehlende Fehler, `Shredded` als Error, unvollständiger KeyProvider und Platzhalter-Fassade. Vor jeder Verbraucherfreigabe schließen. |
| INT-02 | Eine Kern-API für mandantenweiten Erase ist nicht normiert. Bis zur Entscheidung orchestriert `ScopeResolver::enumerate` deterministische Cluster-Batches; es gibt keinen stillen `tenant()`-Aufruf. |
| INT-03 | View-Lease und Hold decken keine beliebigen Langzeit-Checkpoints ab. Langzeitbackups werden als Owner-registrierte Backups mit Residual im Receipt modelliert; eine stärkere Pin-API braucht Storage-Review. |
| INT-04 | `contextra-vector::persistence::MmapIndex` muss vor I3 detailliert geprüft und hinter `SegmentIo` verlegt oder für Personal deaktiviert werden. |
| INT-05 | Format-/Limitwerte werden aus der Storage-SPEC übernommen; widersprüchliche bestehende Konstanten werden nicht durch Adapter kaschiert. |

## 7. Crate-Verträge

Die verbindlichen, einzelnen Crate-Spezifikationen stehen in
[`docs/storage-interfaces/`](storage-interfaces/README.md). Fuzz-Packages
testen jeweils das Contract ihres Mutter-Crates und führen keine eigene
Produktionsschnittstelle.

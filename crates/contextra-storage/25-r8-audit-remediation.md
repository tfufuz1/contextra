# SP-AUDIT — Verbindliche R8-Audit-Remediation

| | |
|---|---|
| Besitzer | Querschnitt; Implementierung verbleibt im jeweiligen Besitzer-SP |
| Normative Quelle | `docs/SPEC-R8.md` §12.8 (hat Vorrang bei Widerspruch) |
| Ziel | Keine Audit-Regel als unzugeordnete Prosa verlieren; jede Regel erhält Besitzer, Phase und prüfbare Abnahme. |
| Phase | Vor G-0/G-F planen; Umsetzung in P0–P8 gemäß Tabelle |

Dieses Dokument führt keine Parallelimplementierung ein. Ein Patch wird in der Quelldatei des zuständigen SP umgesetzt; dieses Register dient als Traceability-Matrix. AUDIT besitzt weder neue Wire-Formate noch Laufzeitcode.

## Verbindliche Zuordnung

| ID | Korrektur | Besitzer-SP | Phase | Abnahmebedingung |
|---|---|---|---|---|
| A-01 | Branch-ID reservieren; SYS_BRANCH vor Rang 1 versiegeln; Commit-Stamp als äußeren Frame-/Record-Stamp führen statt im Ciphertext; unter Sequencer Reservation, Parent-Head, Generation und Zustand erneut prüfen; keine Crypto-/Provider-Rückrufe unter Rang 1 | PORTS, CODEC, CATALOG, COMMIT, BRANCH | P0/P1/P3/P7 | Race-/Fault-Model beweist keine doppelte ID/Stamp; Sperr-Audit findet keinen Kryptoaufruf unter Rang 1 |
| A-02 | `ForkPoint::{Head,At(Stamp)}`; gültiger Floor-/Head-Bereich; Punkt-/Intervallkonflikte inkl. Delete/RangeDelete/EraseRange; Merge-Plan bindet und revalidiert Heads/Base | BRANCH, COMMIT, MAINT | P3/P7 | Referenzmodell für Fork-Leseansicht und Merge-Konflikte; Konflikt lässt Zustand unverändert |
| A-03 | Interner, vor Rang 1 versiegelter `SYS_IDEM`-Record im selben Redo-Commit; Stamp als äußerer MVCC-Stamp; aktive Ergebnisse überleben Redo-Floor; HMAC-Epoche/Scope/Status; TTL plus durable Trim; bounded capacity; Token-Mismatch ⇒ IdemConflict; Retry nach Retire ⇒ Erased | COMMIT, RUN, CATALOG, RECOVERY | P1/P3 | Restart-/Retry-Modell deckt Flush, Compaction, Redo-Rotation, Retired, KeyUnavailable, Kapazitätsgrenze und Token-Reuse ohne zweiten Fsync ab |
| A-04 | Range-Erase nur Logical/Physical/Scrubbed; kein Crypto-Range-Versprechen; Cluster-Key-Rotation erst nach vollständigem Rewrap und Vernichtung aller alten Epochen | ERASE, KEYS, FACADE | P6 | API kann Crypto-Range nicht ausdrücken; Rotation meldet unvollständige Rewraps und zählt alle Epochen |
| A-05 | Space-Drop friert Cluster-Menge unter Sequencer ein; BLOCK_ERASE/Restriction blockiert atomar; Ledger führt pro Cluster `tomb_id` plus `SpaceDropped`-Aggregate mit Scope-Digest, Count/Evidence | CLUSTER, COMMIT, ERASE, LEDGER | P6 | Crash-/Race-Modell: kein partieller Drop; Restore kann einzelne Tombstones matchen; Ledger-Canary enthält keine Klartext-Clusterliste |
| A-06 | Klartext-Clone-Pfade begrenzen; Zeroizing-Ausgaben; Gate widerruft neue Zugriffe, wartet auf Leser-Epochen oder meldet Exposition vor Purged/Receipt | MEMVER, READ, KEYS, ERASE | P1/P4/P6 | Generation-/Epoch-Race-Modell und Receipt-Abnahme berücksichtigen aktive Leser sowie benannte OS-Restexposition |
| A-07 | Blob-Purge ohne Hole-Punch-Annahme; Hole-Ratio/Fragmentlimit erzwingt neue Blob-ID; Rewrite-/Reservegrenzen vor Admission; Plain-Barriere invalidiert Consumer per Epoch/Lagged | BLOB, PURGE, COMMIT, EXT, FACADE | P2/P4/P6 | Purge funktioniert ohne punch_hole; keine Lücke wird als Consumer-Ack bestätigt |
| A-08 | Extension-Watermark höchstens durable_stamp; Builder nutzt durable Quelle; Query-Fassade filtert nach Fund und erneut vor Rückgabe mit Generation-Check | EXT, READ, COMMIT | P4/P5/P6 | Contracttest mit verspätetem Watermark und Gate zwischen Kandidatenfund/Rückgabe liefert keinen Treffer |
| A-09 | Segmentklassen, Schutzdomäne, Löschgranularität, RAM-/Rewritebudget, Attestationsformat; Sealed-Daten clusterlokal geschützt; Rebuildable bleibt bis bestätigtem Rebuild Purging | PORTS, EXT, ERASE, PURGE | P0/P6 | Registrierungsvalidierung verweigert inkompatible Klasse/Schutzdomäne; Receipt wartet auf Attest/Rebuild |
| A-10 | Records im selben Cluster können Dokument/Vektor/BM25/Graph-Delta atomar persistieren; globale HNSW-Reparatur bleibt abgeleiteter Job; Receipt umfasst Kern + registrierte Extensions, nicht ungeattestierte Systembehauptung | COMMIT, EXT, ERASE, BRIDGE | P3/P5/P6/P8 | Atomare Record-Tests; Receipt-Versionierung + Extension-Atteste; ohne Vollständigkeit kein systemweites Complete |
| A-11 | Gemeinsames `MemoryBudget` für Memtables, Cluster-Metadaten, Guards, Caches, Indexdelta, Rewrite/Purge; Admission vor Commit/Indexjob | PORTS, FACADE, MEMVER, READ, MAINT, EXT, PURGE | P0/P1/P4/P5/P6 | jeder Inflight-Pfad reserviert und gibt Budget frei; Lastprofil misst Storage + Extensions gemeinsam |
| A-12 | Archive sind Designmaterial; nur explizit übernommene Korrekturen gelten; kein globales EU-Tag/Content-Addressing, kein In-place-Scrub als Durability-Annahme, keine Crypto-Teillöschung | AUDIT, VERIFY | alle Gates | Architektur-/Formatreview findet keine verworfenen Muster in Produktionspfaden |
| A-13 | KeyProvider unterscheidet `Unavailable`, `Destroyed/Absent`, `Disabled`, `Lost`; nur verifiziertes `Absent` darf Crypto-Erasure abschließen | KEYS, ERASE, RECOVERY, LEDGER | P0/P6 | Conformance- und Fault-Tests zeigen: timeout/KeyLost erzeugt nie `KeysDestroyed` oder Complete |

## Gate-Auswertung

- **G-0:** A-01/A-09/A-11 benötigen stabile Ports-Entscheidungen und klare Rückruf-/Budgetverträge; A-12 ist Teil des Architekturchecks.
- **G-F:** A-01/A-02/A-03/A-04/A-07/A-09 sind in Format, Tag- und Receipt-Freeze zu berücksichtigen.
- **G-W bis G-P:** restliche Abnahmen folgen dem Besitzer-SP und den Test-/Soak-Gates aus SPEC §13–14.
- Eine Zeile gilt erst als erledigt, wenn Code, Testreferenz mit A-ID und Gate-Evidenz vorhanden sind.

## Architekturabgleich mit `docs/archive/`

Archive sind keine Norm, enthalten aber wichtige frühere Lösungen, deren Wegfall in R8 erklärt werden muss:

| Quelle | Starker Aspekt | R8-Entscheidung |
|---|---|---|
| `lethe-final-spec.md` | `SYS_IDEM`, Retention-Pins für Branches, Fixpunkt-/Restore-Races, Extension-Lifecycle, aggregierte Speicherbudgets | Idempotenz wird unabhängig vom Redo-Floor dauerhaft; Floor schließt Branches/Views/Consumers ein; Extension-Registration/Unregister und Restore werden auditiert. |
| `lethe-storage-engine-spec.md` | `SegmentClass`-Unterscheidung (EU-pure, mixed, rebuildable, no-personal-data), explizite Rebuild-Frist, `MemoryBudget`, Trennung `KeyUnavailable/KeyDestroyed/KeyLost` | Klassen und Atteste sind Pflicht; `Mixed` mit globalem Key reicht nicht für Crypto-Erasure. Unbekannter/verlorener Key wird nicht als zerstörter Key ausgegeben. |
| `Erasure-Native Storage Core (Rust)` | Idempotenzsystemspace, Indexe bevorzugt als clusterreine Sektionen, klare Race-Tabelle | Übernommen mit R8-Stamps/Frames. `Zeroizing` wird nicht als alleiniger Beweis für lock-free Skiplist-Reclamation akzeptiert. |
| `MNEMOS_CONCEPT.md` | Utility@Budget und agentische Retrieval-Traces | End-to-End-Benchmark für Recall@Budget, Latenz und RSS; Modell-/Promptsemantik bleibt oberhalb des Kernes. |
| `Optimal-LLM-Datenbank.md` | HNSW/FTS/Graph sind wichtige Vergleichs- und Benchmarkkandidaten | Keine Übernahme globaler Content-Addressing- oder tenantübergreifender Graph-Layouts, die Löschdomänen vermischen. |
| `OCTA-SPEC-FINAL.md` | Bounded allocation, sichere Puffer, signierte Evidence | Als angrenzende Prüfanforderung; keine ungeprüfte Übernahme fremder Cipher-/Receipt-On-Disk-Formate. |

Ältere Lethe-Varianten nutzen teils EU-Tags, In-place-Scrub oder globale Extension-Keys. Diese sind mit der finalen R8-Cluster-/Purge- und Crypto-Erasure-Semantik nicht automatisch kompatibel. Insbesondere impliziert „Mixed-Frame hat Cluster-Tag“ nur Auffindbarkeit/Filterbarkeit, keine kryptografische Unlesbarkeit nach Schlüsselvernichtung. Für Restore-Abgleich müssen Space-Drops deshalb pro Cluster pseudonyme Tombstones persistieren; ein bloßer aggregierter Digest reicht nicht, um einzelne wiederkehrende Cluster aus älteren Backups wiederzufinden.

## Optimierungsvorschläge: verbindliche Gates statt ungeprüfter Zielzahlen

| Vorschlag | Entscheidung für R8 | Abnahmekriterien |
|---|---|---|
| AES-256-GCM statt XChaCha20-Poly1305 | Kein Default-Wechsel ohne Cipher-Review. Cipher-Suite und Version im Format bleiben explizit; Algorithmus austauschbar über Port. | Nonce-/Key-Reuse-Modell, bekannte Antwort auf CPU ohne AES-NI und ARM, Krypto-Review, Formatvektoren und Migration. Keine pauschale Compliance-Aussage aus Algorithmuswahl. |
| Zstd Level 1–2 | Kompressionsfeld vor G-F entscheiden; zunächst `None` als kompatibler Default. | Corpus-Mix und CPU/p99 messen; pro Cluster/Record vor AEAD komprimieren; unkomprimierte Länge und Expansion-Limit speichern; keine geteilten Dictionaries über Cluster. |
| Linux `io_uring`, SIEVE-Cache | Optionaler Adapter-/Cache-Spike nach portablem Basispfad. | Vergleich zu `pread`/bestehendem Cache auf Zielkernel: fsync-Korrektheit, Purge-Bandbreite, p99, CPU und RSS. Kein Sicherheits-/Durability-Vertrag hängt davon ab. |
| `SYS_CLUSTER_META`-Dictionary | Nur clusterlokal und nach Layout-/Corpusmessung. | Versionierung, Bounds, AEAD-Grenze und Rewrite bei Purge; keine cross-cluster-Kompressionsdictionarys. |
| Sharded Sealed `person_dir` | Nicht ungeprüft übernehmen. | Löschgranularität des Shards, Überlöschung anderer Personen, Backup-Rollback und Reconciliation müssen lösbar sein; HMAC-Key-Drehung allein löscht keine Backup-Kopien. |
| Vektor-Mikrobuckets + Tenant-HNSW | Als Indexarchitektur der oberen Schicht zulässig, kein Kernversprechen. | Recall@k, Filterselektivität, Update-/Erase-Rate, Indexlag, RAM und Rebuild-Zeit gegen HNSW/DiskANN messen. `weights[4]` nicht als untypisiertes Ablagefeld für Quantisierungsskalare missbrauchen. |
| Dokument + Vektor + BM25 + Graph in einem Change | Ja für die kanonischen Records/Deltas derselben Cluster-Löschdomäne. | Alle Records tragen denselben Cluster-/Branch-/Stamp-/Durability-Kontext. Abgeleitete HNSW-Nachbarschaftsreparatur bleibt eventual und watermark-/delta-konsistent. |
| ReceiptSigner / v15 `DeletionProof` | Versionierte Bridge, keine Kopplung des Kerns an Ed25519/PKCS#11. | Signiert kanonisches Receipt plus verifizierte Extension- und Anchor-Atteste. Fehlender Witness ergibt `Incomplete`, nie `Complete`. Offline-Referenzvektoren. |

Zahlen wie „12 GB/s pro Kern“, „2,5–4× Zstd“ oder „70 % Speicherersparnis“ werden nicht als R8-Planwerte übernommen. Sie sind Hypothesen bis zu reproduzierbaren Messungen auf festgelegten CPU-, Corpus-, Blockgrößen- und Lastprofilen.

## Geschärfte Systemgrenzen und Kostenmodelle

- **Atomarität:** Ein synchroner `Change` ersetzt 2PC nur für Records, die im Kern liegen. Externe Indexdateien bleiben abgeleitete Teilnehmer mit durablem Feed, Watermark, Delta-/Lag-Verhalten und Erase-Attest.
- **Purge:** Kosten sind nicht generell O(Cluster). Sie sind O(betroffene begrenzte Packgruppe/Blob-Extents) im Kern und können für globale Index-Rebuilds O(Indexpartition) sein. Ein lokales Kostenversprechen braucht harte Maxima für Packgruppe, Extents und temporären Scratch.
- **ENOSPC:** Reserve kann Gate-/Control-Frames und Schlüsselvernichtung ermöglichen, aber keinen Rewrite überlebender Bytes ohne Platz garantieren. Bei zu wenig Scratch bleibt Ticket `Purging/Blocked(DiskSpace)`; es gibt kein falsches physisches `Done`.
- **Klartext:** `Zeroizing` deckt nur den konkreten Speicherbereich ab. Shared `Bytes`, allocator copies, OS-Pagecache, Swap und Dumps bleiben eigene Grenzen. Memtable muss Ownership/Reader-Epochen definieren; Receipt benennt Restexposition.
- **Indexsichtbarkeit:** Nur `durable_stamp` darf als angewandt bestätigt werden. Query ist Basisindex + vollständiges Delta bis View-Stamp oder explizites `IndexLag`; Gatefilter wird nach Kandidatenfund und vor Rückgabe erzwungen.
- **Crypto-Erasure:** Nur dann, wenn alle persistenten Kopien pro Cluster-Schlüssel/Epoche geschützt sind und KeyProvider die Vernichtung attestiert. Globaler Index-Key plus Cluster-Tag ermöglicht Purge, aber keine Crypto-Erasure.

## Zusätzliche Gate-Details

**Vor G-0:** API- und Modellreview für A-01–A-13; klare Zusageebenen `Core`, `Registered Extension`, `Gesamtsystem`; Resource-Budget-Ports; Key-Statusmodell inklusive `Unavailable`, `Destroyed`, `Disabled`, `Lost`.

**Vor G-F:** Golden Vectors/Feature-Bits für `SYS_IDEM`, `ForkPoint`/Merge-Basis, Blob-Repack mit neuer Blob-ID/Offsets, Extension-Klassen/Atteste und Receipt-Bridge. Hole-Punch, AES-Hardware und Zstd dürfen nicht als implizite Plattformannahmen im Format stecken.

**Vor G-E:** Fault-Sweep Gate → KeyDestroy → Repack/Rewrite → Catalog-Install → Unlink → Extension-Ack → Ledger → Anchor → Signatur; Restore aus Backup vor und nach Gate; Extension-Ausfall; kein Scratch; aktiver Cursor/Reader; alle Ergebnisse prüfen Statusmonotonie und dass kein Ticket vor erfüllten Attesten `Done` wird.

**Vor G-P:** Gemeinsamer RAG-Lastmix mit Storage + HNSW + BM25 + CSR + Cache. Messen RSS-Spitzen und Dauer, Flush-/Compaction-/Purge-Bytes, Indexlag, Recall@Budget, Gate-to-filter latency, Rebuild-Fenster und Kosten bei 1×/10×/100× Storegröße.

## 11. Ablösung von `contextra-store` und Produkt-Cutover

R8 ist der künftige Persistenzkern des gesamten Contextra-Systems. Der Cutover ist daher ein eigenes Produktions-Gate und nicht bloß ein Adapter-Compile. Bis dieses Gate besteht, ist `contextra-store` der produktive Rückfallpfad; R8 darf keine implizite Teilmigration einzelner Crates erzwingen.

### 11.1 Vollständiges Persistenz-Inventar

Vor API-Freeze wird jeder direkte und indirekte `contextra-store`-Nutzer inventarisiert: Engine-Transaktionen, Collection-/Dokumentzustand, Vektorindex und WAL, BM25, Graph/CSR/Hyperkanten, KV-Cache, Audit-/DeletionProof, Checkpoints/Restore, Background Worker, CLI/MCP sowie externe SDK-/Python-Schnittstellen. Pro Nutzer wird festgehalten: Persistenzmodell, Durability-Anforderung, Transaktionsgrenze, Recovery-Verhalten, Delete-/Proof-Pfad, Datenformat, Migrationsowner und akzeptierte Latenz/RSS.

Keine Schicht darf nach Cutover heimlich weiter in `contextra-store` schreiben. CI erhält eine Architekturregel/Dependency-Allowlist, die Legacy-Abhängigkeiten nach Freigabe nur noch im `contextra-storage-bridge` und im Migrationswerkzeug zulässt.

### 11.2 API- und Semantik-Kompatibilität

Die Bridge stellt die konkret benötigten Alt-APIs bereit und mappt Semantik sichtbar: Commit-/Rollback, Read-your-writes, Snapshot, Range-Delete, Idempotenz, Cancel, Error/Indeterminate, Locking und Batchverhalten. `Unsupported`-Fälle sind versioniert und vorab mit allen Aufrufern geschlossen. Das synchron arbeitende R8 darf keine blockierende IO auf Tokio-Workerthreads ausführen; Blocking-Pool und Queue erhalten Grenzen/Backpressure.

Altstand-2PC/Saga wird nicht als stiller Adapter nachgebaut. Jede bisherige Multi-Index-Operation erhält eine explizite neue Grenze: kanonische Records in einem R8-Change; derived-index Jobs per durable feed/outbox, Watermark und Recovery. Vor jeder Mutation muss klar sein, ob ein Crash zu Retry, `Indeterminate`, Rebuild oder user-visible `IndexLag` führt.

### 11.3 Migrations- und Umschaltprotokoll

1. **Baseline:** versionierter Snapshot/Export des Altstores plus Indexmanifeste und Counts; Input wird während Export entweder eingefroren oder durch ein beweisbares Change-Capture-Protokoll fortgeschrieben.
2. **Import:** resumierbare, chunkweise, IdemToken-gesicherte Writes nach R8. Mapping-Bericht für Keys/Cluster/Spaces/Branches, Protection-Klasse und bewusst nicht migrierte Daten. Jeder Chunk bekommt Checksumme/Count.
3. **Rebuild:** HNSW/BM25/CSR/Cache-Segmente aus R8-konsistentem Quellzustand neu aufbauen oder kontrolliert migrieren; Watermarks nie über importierte durable Stamps setzen.
4. **Differential Read:** Shadow-Queries vergleichen KV-Werte, Retrieval-Kandidaten/Recall, Graph-Ergebnisse, Löschsichtbarkeit und Fehlerklassen. Nichtdeterministische Rankings erhalten definierte Toleranzen, aber keine Toleranz für fehlende/gated Records.
5. **Cutover:** Schreibstopp oder geordnete finale Change-Capture-Barriere; finaler Count-/Digest-Abgleich; alle R8-Indizes durable bis Cutover-Watermark; danach genau ein Writer pro Datenbestand. Kein unkoordiniertes Dual-Write auf Legacy und R8.
6. **Rollback:** Vor dem ersten R8-Write kann auf unveränderten Legacy-Stand zurückgeschaltet werden. Nach R8-Write nur über getesteten Forward-Export/Replay zurück; „Rollback“ darf keine bestätigten R8-Schreibvorgänge verlieren. Runbook benennt diesen irreversiblen Punkt und dessen Freigabekriterium.
7. **Retirement:** Erst nach Soak, Restore-Probe, Erasure-Probe und belegter Rebuild-/Rollback-Probe werden Legacy-Writer, CommitLedger/2PC-Codepfade und alte Datenformate deaktiviert. Archiv-/Exportzugriff bleibt entsprechend Aufbewahrungsrichtlinie lesbar, ohne neue Writes.

### 11.4 Freigabe-Gate für das Gesamtprodukt (G-CUTOVER)

G-CUTOVER verlangt: vollständige Caller-Matrix ohne ungeklärte Semantik; reale Export/Import/Resume-Rehearsal; differential audit ohne unerklärte Abweichungen; Crash- und Restore-Soak; konkurrierende Write/Erase/Index-Rebuild-Simulation; ErasureReceipt→DeletionProof v3 Bridge für alle derzeitigen Witness-Layer; End-to-End-RAG-Benchmarks gegen den heutigen Stack; gemessene RSS-/Disk-/CPU-Budgets; operatives Monitoring, Alarmierung und Runbooks für DiskFull, Poison, KeyUnavailable/Lost, IndexLag, Purging und RestoreQuarantine.

Bis G-CUTOVER bestanden ist, dürfen R8-Vorteile gegenüber dem bisherigen Stack nur als Designziele beschrieben werden. Danach bleibt `contextra-store` aus dem normalen Produktions-Schreibpfad entfernt; ein paralleler Legacy-Write-Pfad würde erneut uneinheitliche Commit- und Löschwahrheit schaffen.

# SP-AUDIT — Verbindliche R8.1/R8.2-Audit-Remediation

| | |
|---|---|
| Besitzer | Querschnitt; Implementierung verbleibt im jeweiligen Besitzer-SP |
| Normative Quelle | `crates/contextra-storage/SPEC.md` §12.8 (hat Vorrang bei Widerspruch) |
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
| A-05 | Space-Drop schreibt vor Gate ein pseudonymes `SpaceDropIntent`, blockiert bei Restriction, Restore matcht die nie wiederverwendete SpaceId; Crypto-Erase auf Space-Ebene ist v1 verboten, Cluster-Erase läuft über Cluster-Tickets | CLUSTER, COMMIT, ERASE, LEDGER, RECOVERY | P6 | Crash-/Restore-Modell gated jede alte Kopie des Space vor Zugriff; kein Intent ohne Gate-Recovery; Ledger-Canary enthält keine Klartext-IDs |
| A-06 | Klartext-Clone-Pfade begrenzen; Zeroizing-Ausgaben; Gate widerruft neue Zugriffe, wartet auf Leser-Epochen oder meldet Exposition vor Purged/Receipt | MEMVER, READ, KEYS, ERASE | P1/P4/P6 | Generation-/Epoch-Race-Modell und Receipt-Abnahme berücksichtigen aktive Leser sowie benannte OS-Restexposition |
| A-07 | Blob-Purge ohne Hole-Punch-Annahme; Hole-Ratio/Fragmentlimit erzwingt neue Blob-ID; Rewrite-/Reservegrenzen vor Admission; Plain-Barriere invalidiert Consumer per Epoch/Lagged | BLOB, PURGE, COMMIT, EXT, FACADE | P2/P4/P6 | Purge funktioniert ohne punch_hole; keine Lücke wird als Consumer-Ack bestätigt |
| A-08 | Extension-Watermark höchstens durable_stamp; Builder nutzt durable Quelle; Query-Fassade filtert nach Fund und erneut vor Rückgabe mit Generation-Check | EXT, READ, COMMIT | P4/P5/P6 | Contracttest mit verspätetem Watermark und Gate zwischen Kandidatenfund/Rückgabe liefert keinen Treffer |
| A-09 | Segmentklassen, Schutzdomäne, Löschgranularität, RAM-/Rewritebudget, Attestationsformat; Sealed-Daten clusterlokal geschützt; Rebuildable bleibt bis bestätigtem Rebuild Purging | PORTS, EXT, ERASE, PURGE | P0/P6 | Registrierungsvalidierung verweigert inkompatible Klasse/Schutzdomäne; Receipt wartet auf Attest/Rebuild |
| A-10 | Records im selben Cluster können Dokument/Vektor/BM25/Graph-Delta atomar persistieren; globale HNSW-Reparatur bleibt abgeleiteter Job; Receipt umfasst Kern + registrierte Extensions, nicht ungeattestierte Systembehauptung | COMMIT, EXT, ERASE, BRIDGE | P3/P5/P6/P8 | Atomare Record-Tests; Receipt-Versionierung + Extension-Atteste; ohne Vollständigkeit kein systemweites Complete |
| A-11 | Gemeinsames `MemoryBudget` für Memtables, Cluster-Metadaten, Guards, Caches, Indexdelta, Rewrite/Purge; Admission vor Commit/Indexjob | PORTS, FACADE, MEMVER, READ, MAINT, EXT, PURGE | P0/P1/P4/P5/P6 | jeder Inflight-Pfad reserviert und gibt Budget frei; Lastprofil misst Storage + Extensions gemeinsam |
| A-12 | Archive sind Designmaterial; nur explizit übernommene Korrekturen gelten; kein globales EU-Tag/Content-Addressing, kein In-place-Scrub als Durability-Annahme, keine Crypto-Teillöschung | AUDIT, VERIFY | alle Gates | Architektur-/Formatreview findet keine verworfenen Muster in Produktionspfaden |
| A-13 | KeyProvider unterscheidet `Unavailable`, `Destroyed/Absent`, `Disabled`, `Lost`; nur verifiziertes `Absent` darf Crypto-Erasure abschließen | KEYS, ERASE, RECOVERY, LEDGER | P0/P6 | Conformance- und Fault-Tests zeigen: timeout/KeyLost erzeugt nie `KeysDestroyed` oder Complete |
| A-14 | Redo-Recovery unterscheidet `KeyUnavailable` (LockedKeys, retrybar) von bestätigtem `KeyLost` (Quarantäne/Corrupt); kein aktiver Sealed-Op wird ohne Cluster-Epoch-Key übersprungen | REDO, RECOVERY, KEYS | P1/P3 | Fault-Injektion für Provider-Timeout, Lost und Shredded; nur Shredded nach Ledger-Beleg erlaubt Retire-Skip |
| A-15 | Blob-AAD nutzt stabile logische Record-Identität statt Blob-ID/Offset; unabhängige Identität verhindert Cross-Cluster-Replay und ermöglicht Rohkopie | CODEC, BLOB, PURGE | P1/P4/P6 | Golden-/Property-Tests für File-/Offset-Repack und negative AAD-Replay-Fälle |
| A-16 | INV-ERA-11 trennt isolierte Run/Blob-Kosten von gemeinsamem Redo, Pins, Retention-Copies und Extensions; keine unbedingte O(Cluster)-Behauptung | ERASE, PURGE, CKPT, EXT, VERIFY | P0/P6 | Byte-Zähler unter Store-Skalierung und künstlich gesetzten Pins bestätigen nur explizit konfigurierte Obergrenzen |
| A-17 | DiskFull-Scratch-Fortschritt ist nur zugesagt, wenn Purge-Quelle nicht durch Reflink-/Hardlink-Checkpoints gepinnt bleibt oder deren Owner Extent-Freigabe attestiert | CKPT, PURGE, LEDGER | P0/P6 | FaultFs/FS-Matrix mit Copy, Reflink, Hardlink, CoW/Quota; kein Pool-Recycling vor belegter Freigabe |
| A-18 | Katalog-Purge ist mehrstufig (E-67): Batch-Edit pinnt entfernte Datei im persistenten Graveyard; Reader-Drain, `unlink + sync_dir`, `UnlinkAckEdit`, finaler Complete-Edit nach allen Owners | CATALOG, PURGE, ERASE | P1/P6 | Crash vor/nach Unlink setzt idempotent fort; Scratch wird erst nach Ack recycelt; `Done` kann keinem offenen Graveyard vorausgehen |
| A-19 | Implementierung darf die Siegelspezifikation nicht still abändern: v1 ruft XChaCha direkt mit dem bereits scope-spezifischen Schlüssel auf; vorhandene `derive_subkey_fast`-Vorstufe weicht davon ab und muss vor G-F entfernt bzw. auf stabile Schlüsselrollen begrenzt werden | CODEC, KEYS, VERIFY | P0/G-F | unabhängige Vektoren prüfen Cipher-Key/Nonce/AAD exakt; kein `derive_subkey_fast` im SealFrame-Pfad |
| A-20 | Transitive `DerivedFrom`-Erase-Hülle muss in einen begrenzten Gate-Change passen; Link-Admission berechnet sie außerhalb Rang 1 und revalidiert Graph-Generation bei Commit | CLUSTER, COMMIT, ERASE | P1/P6 | Grenztests bei 4096/4097 Nachfahren; parallele Link-Races führen zu Retry oder `EraseFanoutExceeded`, nie zu Teil-Gate |
| A-21 | Clusterübergreifende ANN-/BM25-/Graph-Strukturen sind keine per-Cluster crypto-erasebaren Sektionen; sie brauchen sofortigen Gate-Filter und bounded Rewrite/Verify als `Rebuildable` | EXT, ERASE, BRIDGE, VERIFY | P0/P5/P6/P8 | Nach Gate liefern Queries null Treffer; nach Crypto-Erase ist kein betroffener Payload/Edge/Posting in Segment, Cache oder Backup lesbar; über Limit bleibt Receipt `Incomplete` |
| A-22 | Rollback-/Restore-sichere Löschung braucht Ledger-Intent vor Catalog-Gate; selektive Range-/Branch-Scope wird nur verschlüsselt im Ledger abgelegt, nie als Klartext | LEDGER, ERASE, RECOVERY, ANCHOR | P0/P1/P6 | Crash an jeder Intent/Gate/Anchor/KMS-Grenze konvergiert vor Open zur Gate-Wahrheit; ältere Backups bleiben gefiltert; Ankerfehler stoppt irreversible Folgestufe |
| A-23 | `SpaceId` ist für eine langlebige Embedded-Instanz ein monoton reservierter `u64`; die bisherige `u16`-Grenze und Wiederverwendung nach Catalog-Trim sind ausgeschlossen | PORTS, CATALOG, CODEC, ERASE, RECOVERY | P0/G-F | `reserve_space_ids` ist vor Vergabe durabel; alle Space-Frames, Catalog-Tags, Gates und Tomb-HMACs nutzen eine kanonische 64-Bit-ID; Crash-/Restore-Vektoren beweisen keine Wiedervergabe und `Exhausted` statt Wrap |
| A-24 | Lokaler `LOCK` verhindert keine zwei aktiven Kopien einer restaurierten `InstanceId`; Rollback-Anchor ist kein Eigentümer-Lease | RECOVERY, CKPT, CUTOVER | P0/P8+ | Schreibender Restore erhält externen Fencing-Nachweis; Zwei-Host-Rehearsal weist eine aktive Kopie nach, die zweite bleibt `RestoreQuarantine` |
| A-25 | `tomb_epoch`-Format-Slot, Provider-Tomb-Schlüssel, Retire nur nach Backup-Horizont mit `restore_floor`; Restore verweigert `BackupBeyondHorizon` (E-74, K-31) | LEDGER, KEYS, CKPT, RECOVERY | Slot vor G-F, Ablauf P6 | `INV-LED-2`, `INV-KEY-5`; Crash-Matrix Retire; Vektoren mit zwei Epochen |
| A-26 | Löschauftrag auf `Poisoned`: durabler Intent im Ledger, `ErasePending`, Gate beim Neustart, kein KeyDestroy/Purge ohne Gate (E-75) | ERASE, LEDGER, FACADE, RECOVERY, READ | P6 | `INV-ERA-13`; Fehlerinjektion `EIO` + Erase, Ledger gesund/vergiftet |
| A-27 | View-Lease mit `SnapshotTooOld`, getrennt von `BelowFloor`; Ablauf wirkt wie Revoke (E-76) | READ, MEMVER, FACADE | P4 | `INV-VIEW-3`; Lease-Test mit Sim-Clock, Graveyard-Freigabe |
| A-28 | `KeysInBackups`/`KeyAssurance`, Statuscaps, `ProviderAssuranceInsufficient` (E-77); `KeyProvider`-Trait an SPEC §9.4 angleichen | PORTS, KEYS, ERASE, FACADE | P0/P6 | Konformanzsuite, Matrix Provider × Receipt-Status |
| A-29 | Optionaler `HostProbe` (Swap, Core-Dumps, memlock, Dateisystem, Geräte-IDs), nie Korrektheitsquelle (E-78) | PORTS, ENV, ERASE | P5 | `Unknown` bleibt `Unknown`; Receipt-Restexposition `Declared` |
| A-30 | `BlockPruner`/`block_boundary` nur für deklariert schattensichere Besitzer auf `main`; keine Zeitlanes im Kernformat (E-79, K-29) | EXT, READ, RUN, PORTS | P6 | `INV-PRUNE-1`; Property „mit ≡ ohne Pruner“ |
| A-31 | `MeasureKind` je Messung; Pflichtmessungen (a)–(d) müssen `Complete` sein (E-80) | ERASE, VERIFY, BRIDGE | P6 | `INV-VER-1`; CI-Prüfung der Receipt-Cap-Regeln |
| A-32 | Backup-Marke (`erase_epoch`, `created_at`), `LedgerSameDevice`/`LedgerUnavailable`, `Storage::erase_epoch()` (E-81) | FACADE, CKPT, LEDGER, RECOVERY | P6 | Restore-/Open-Tests, Production-Profil-Test |
| A-33 | Derivation Cache als clusterlokale Konvention; `INV-PRIV-1` präzisiert (E-82, K-32) | EXT, BRIDGE, CUTOVER | P6/P8 | Canary nach Cluster-Erase, kein Cross-Cluster-Treffer |
| A-34 | Kumulative SubIndex-Summen im RAM statt im Format (K-30) | RUN, READ, VERIFY | P4 | H-4-Bericht trennt Aufbau/Traversierung |

## R8.1-Querschnittsverträge

| Ziel | Besitzer-SP | Abnahme |
|---|---|---|
| `DataClass`-Enum und Vor-I/O-Validierung; keine implizite PII-Erkennung oder Herabstufung | PORTS, FACADE, COMMIT | Personal-Kombinationen und Export-/Space-Übergänge sind typisiert und vor Mutation abgewiesen |
| Erase-Metriken für Gate/KeyDestroy/Purge, endliche Label-Allowlist, leakagefreier Inspector und sichere Fehlerhinweise | PORTS, FACADE, ERASE, VERIFY | unbekannte Namen/Labels verworfen; keine Kennungen/Inhalte; Observer-Ausfall verändert keine Commit-/Erase-Semantik |
| Open-Core-Technikgrenze und optionale kommerzielle Adapter ohne semantische Abhängigkeit | PORTS, BRIDGE, CUTOVER | offenes Format, lokaler Betrieb, Export/Restore und Konformanz ohne kommerzielle Adapter möglich |
| RAG Ende-zu-Ende: atomare kanonische Records, durable Watermarks, prefilter, Gate-Filter, Delta/Fallback und gemeinsames Budget | COMMIT, EXT, READ, FACADE | Index hinter Watermark meldet Lag oder schließt Lücke; keine gated Treffer, kein unbeschränktes Budget |

## Gate-Auswertung

- **G-0:** A-01/A-09/A-11 benötigen stabile Ports-Entscheidungen und klare Rückruf-/Budgetverträge; A-12 ist Teil des Architekturchecks.
- **G-F:** A-01/A-02/A-03/A-04/A-07/A-09/A-15/A-18/A-19/A-23 sind in Format-, Tag-, AAD- und Receipt-Freeze zu berücksichtigen. Insbesondere werden Systemspace-Belegung, `SpaceId u64` samt Allocator, Domain-Separation/Byteordnung aller Ledger-HMACs und Request-Digests, Receipt-Status/Canonicalisierung sowie Purge-Batch-/Graveyard-Tags und Extension-Manifest-/Attestationsformat eingefroren. Es gibt kein G-F bei `SpaceId u16` in Ports oder einem Wire-Codec.
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
| `Optimal-LLM-Datenbank.md` | Recall unter Tokenbudget, Zoom-/Handle-Modell, getrennte Vektor-/Text-/Graph-Zugriffspfade und End-to-End-RAG-Benchmarks | Übernommen als obere Context-/Retrieval-Schicht und Benchmarkvertrag. Im Kern bleiben kanonische Records und ihre Metadaten atomar; globale ANN-/BM25-/Graph-Strukturen sind abgeleitet, watermark-konsistent und müssen Gate-Filter sowie Löschatteste erfüllen. Kein globales Content-Addressing oder Layout über Löschdomänen. |
| `MNEMOS_CONCEPT.md` | Utility@Budget, Agenten-Branches, Ableitungs-Lineage und „raw vs. derived“ | Utility@Budget wird Release-Benchmark; Branches und Lineage werden durch Cluster-Branches/Links begrenzt. Der Commit-DAG, tenantweites Dedupe und globale Access-Trace-Platzierung werden wegen Speicher-/Löschkopplung nicht übernommen. |
| `OCTA-SPEC-FINAL.md` | Begrenzte Allokationen, sichere Puffer und signierte Evidence | Gemeinsamer MemoryBudget, Ownership-/Epoch-Reclamation und kanonische Receipt-Bridge werden übernommen. Signatur bleibt außerhalb des Kerns; kein fremdes On-Disk-/Cipher-Format wird ungeprüft übernommen. |

Ältere Lethe-Varianten nutzen teils EU-Tags, In-place-Scrub oder globale Extension-Keys. Diese sind mit der finalen R8-Cluster-/Purge- und Crypto-Erasure-Semantik nicht automatisch kompatibel. Insbesondere impliziert „Mixed-Frame hat Cluster-Tag“ nur Auffindbarkeit/Filterbarkeit, keine kryptografische Unlesbarkeit nach Schlüsselvernichtung. Für Restore-Abgleich müssen Space-Drops deshalb pro Cluster pseudonyme Tombstones persistieren; ein bloßer aggregierter Digest reicht nicht, um einzelne wiederkehrende Cluster aus älteren Backups wiederzufinden.

## Optimierungsvorschläge: verbindliche Gates statt ungeprüfter Zielzahlen

| Vorschlag | Entscheidung für R8 | Abnahmekriterien |
|---|---|---|
| AES-256-GCM statt XChaCha20-Poly1305 | Kein Default-Wechsel ohne Cipher-Review. Cipher-Suite und Version im Format bleiben explizit; Algorithmus austauschbar über Port. | Nonce-/Key-Reuse-Modell, bekannte Antwort auf CPU ohne AES-NI und ARM, Krypto-Review, Formatvektoren und Migration. Keine pauschale Compliance-Aussage aus Algorithmuswahl. |
| Kompression | **v1 verbindlich `None`**; `COMPRESSION=0`. Kein Kompressionsdecoder und keine stillschweigende Kompression im Format-Freeze. | Spätere Version nur mit unabhängiger Corpus-/CPU-/p99-Messung, bounded Expansion, gespeicherter Klartextlänge und Cluster-isolierten Dictionaries; Migration/Feature-Bit vor Implementierung spezifizieren. |
| Linux `io_uring`, SIEVE-Cache | Optionaler Adapter-/Cache-Spike nach portablem Basispfad. | Vergleich zu `pread`/bestehendem Cache auf Zielkernel: fsync-Korrektheit, Purge-Bandbreite, p99, CPU und RSS. Kein Sicherheits-/Durability-Vertrag hängt davon ab. |
| `SYS_CLUSTER_META`-Dictionary | Nur clusterlokal und nach Layout-/Corpusmessung. | Versionierung, Bounds, AEAD-Grenze und Rewrite bei Purge; keine cross-cluster-Kompressionsdictionarys. |
| `person_dir` | Kein gemeinsam versiegelter Shard: ein gemeinsamer Shard-DEK macht Einzellöschung unmöglich bzw. löscht Nachbarn mit. Gewählt ist `IdentityDirectoryOwner` mit per-Subject EntryDEK, tenant-scoped HMAC-Lookup und registriertem Reverse-Lookup. | Backup-/Restore-Reconcile muss nach EntryDEK-Vernichtung den Eintrag als Tombstone behandeln; HMAC-Pseudonyme sind weiterhin personenbezogene/pseudonyme Daten. Der Owner muss Watermark, `on_gate`, Key-Destroy, Rewrite und Verify attestieren. |
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

**Vor G-F:** Golden Vectors/Feature-Bits für Systemspace-Belegung samt `SpaceId u64`/Reservierung, `SYS_IDEM`, `ForkPoint`/Merge-Basis/ChangeIndex, Blob-Repack mit stabiler Record-AAD, Extension-Klassen/Atteste, Ledger- und Request-HMAC-Domänen/Byteordnung, Receipt-Status und -Canonicalisierung sowie Purge-Batch-/Graveyard-Ablauf. `COMPRESSION=0` und Bloom `filter_kind=1` sind die einzigen v1-Werte. Hole-Punch, AES-Hardware und spätere Kompression dürfen nicht als implizite Plattformannahmen im Format stecken.

**Vor G-E:** Fault-Sweep Ledger-Intent append/sync → Catalog-Gate → `GateApplied` → Anchor-CAS → `KeyDestroyIntent` → KMS destroy/probe → PurgeBatchEdit/Repack → Reader-Drain → Unlink+`sync_dir` → UnlinkAck → Extension-Attest → finaler Complete-Edit → Receipt-Signatur. Restore aus Backups vor/nach jedem durablem Präfix; zusätzlich Extension-Ausfall, fehlender Scratch, aktiver Cursor/Reader und Anker-Ausfall. Jede Injektion prüft Vorwärtskonvergenz vor Open, Statusmonotonie und dass kein Ticket vor erfüllten Attesten `Done` wird.

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

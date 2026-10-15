# Contextra – Gesamtspezifikation v4 (final)

Oct 15, 2026 · @Freddy

**Stand:** Donnerstag, 15. Oktober 2026 (Änderungen gegenüber v4 nur in den Abschnitten 4.5, 10.1, 10.4, 11, 12, 14 und I.3) · **Basis:** Repository tfufuz1/contextra, HEAD 5d349c1 (geklont und am Quelltext geprüft) · **Status:** v4.1, Entfernung statt Einfrieren

v4 ist die einzige normative Spezifikation von Contextra. Sie schließt alle in v3 offenen Fragen mit einer Entscheidung, korrigiert v3 dort, wo der Quelltext v3 widerlegt, und ist ohne v3 lesbar. Der ausführliche Befundbestand aus Teil II von v3 ist vollständig in den Anhängen D bis J übernommen und gilt dort als Archiv, soweit Abschnitt 2 ihn nicht ändert.

## 1. Status, Lesehilfe und Entscheidungsübersicht

**Messgrenze:** Die Prüfumgebung hat `cargo`, kann die gepinnte Toolchain 1.89.0 aber nicht laden (kein Zugriff auf static.rust-lang.org). Es gibt daher weiterhin keinen Build- und Testlauf. Alle Aussagen sind Datei- und Textprüfungen. K0.9 bleibt die erste Arbeitshandlung.

| Marke | Bedeutung |
| --- | --- |
| \[G\] | am HEAD 5d349c1 in Datei oder Git gelesen |
| \[H\] | Heuristik, Textsuche |
| \[E\] | Emulation ohne cargo-Lauf (Zahlen aus v3, nicht neu gemessen) |
| \[T\] | Hypothese, braucht einen Test |
| \[D\] | Dokumentbehauptung aus den Quellplänen, nicht reproduziert |
| \[U\] | unverifiziert |
| **DE-nn** | Entscheidung dieses Dokuments (Register in Abschnitt 3) |

**Zehn Kernentscheidungen von v4:**

1. **Reihenfolge bleibt:** Messinstrumente, Reparatur, Entkopplung, Schnitt, Nachweis, Erweiterungen. Ebenen R bis G starten erst nach K4.
2. **Löschmodi:** Off (Standard), Assured, Attested. v0.1.0 liefert Off und Assured. Attested ist ein Plugin außerhalb der Default-Hülle (EP2).
3. **Fence:** Option B in angepasster Form. Der Check liegt in der ersten Haltephase des `commit_mutex`, vor Sequenzvergabe und WAL-Append. Der Purge wartet nach dem Erwerb auf eine Drain-Wasserstandsmarke, weil der Mutex vor dem WAL-Append freigegeben wird (DE-05).
4. **Schlüssel:** zufällig und gewickelt, nie abgeleitet. Kryptoverfahren für v0.1.0 ist AES-256-GCM-SIV (DE-09). Ein Krypto-Review kann nur mit einem Befund widersprechen.
5. **HNSW (Veto-Auflösung):** v0.1.0 nutzt nur Tombstone-Pruning plus Neuaufbau des Collection-Snapshots aus dem gefilterten LSM. ADR-097 (Nachbarschaftsreparatur) wird nicht verwendet, VETO-F02 bleibt unangetastet (DE-11).
6. **Ports:** Der vorhandene Port `contextra-ports::erasure` wird weiterentwickelt, nicht neu geschrieben. Er erhält Provenienz im Scope, `StorageEngine::erasure_backend()` statt `StorageErasureExt` und ein öffentliches Statusvokabular (DE-03, DE-04).
7. **Fehlersemantik:** Nach dem Tombstone-Commit gibt es für die Löschung kein `Err`. Fehler sind abfragbare Zustände.
8. **Parkplatz-Crates:** Tag `park/EPn/v1`, dann aus dem Workspace entfernen. Kein eigenes Repository, bis eine Ebene startet (DE-14).
9. **Lizenz:** Die Lizenz steuert nur das Ausstellen neuer Beweise, nie `open` oder die Löschpflicht. EP1 bleibt vollständig offen.
10. **Nur zwei Punkte bleiben beim Maintainer:** der Wortlaut der Honesty-Klausel (Entwurf in Abschnitt 13) und die Lizenz des EP2-Ausstellers (Abschnitt 13.4).

## 2. Re-Verifikation am Klon (HEAD 5d349c1)

Der Klon bestätigt den größten Teil der v3-Befunde und widerlegt fünf Annahmen. Diese fünf ändern Entwürfe und stehen oben in der Tabelle.

| Aussage | Beleg am Klon | Stand | Folge in v4 |
| --- | --- | --- | --- |
| `contextra-ports` enthält kein Erasure-Modul (v3 II.5.6) | `crates/contextra-ports/src/erasure.rs`, 514 Zeilen: ErasureMode, LogicalScope, PhysicalScope, ErasureBackend, StorageErasureExt, ErasureHooks, WriteGate, FreeSpaceProbe, RestoreGuard, LayerScan, DeleteOutcome, PurgeState, EraseError \[G\] | **v3 widerlegt** | Abschnitt 4.3 und 17 entwickeln den Bestand weiter, DE-03 |
| Vorhandene Port-Abweichungen gegen v3 | LogicalScope trägt `tenant_id`, PhysicalScope hat keine Provenienz (`try_new` nur hinter Feature `backend-impl`), Methode heißt `purge_physical`, Erkennung über `StorageErasureExt::as_erasure_backend`, DeleteOutcome kennt nur Logical und Assured, EraseError kennt NotStarted und Indeterminate \[G\] | Abweichung | DE-03, DE-04 |
| `commit_mutex` ist der Serialisierungspunkt, Check dort synchron (v3 6.3, 8.5) | `lsm/ops/write.rs:355` `commit_mutex.lock().await` ist ein `tokio::sync::Mutex`. Laut `lib.rs:59` wird er vor `wal.append_batch` freigegeben und für Memtable und Sichtbarkeit neu erworben \[G\] | **v3 unvollständig** | Check in der ersten Haltephase plus Drain-Marke, DE-05 |
| KvDeleteMode-Default ist CryptoShred (v3 5.5, K1.15) | `store/src/kv/delete_mode.rs:18` Default, Test `kv_delete_mode_wiring_test.rs` erwartet TombstoneOnly \[G\] | **v3 überholt, erledigt** | K1.15 entfällt |
| Physische Schlüssel tragen Collection-Namen (v3 17.2) | `store/src/tenant_codec.rs`: `t:{tenant}:{collection_id}:chunk:{doc}`, `collection_prefix` endet auf `:` \[G\]. Die Engine nutzt daneben `__col:{name}:\x00` (`collection/mod.rs:399`) und den Katalog `__col_idx:` \[G\] | **zwei Layouts** | DE-06 |
| DurabilityMode-Namen sind unklar (v3 22.5) | `store/src/lsm/config.rs:8`: Full, WalNoHmac, MemoryOnly \[G\] | geklärt | Matrix 5.3 gilt, Zeile Buffered entfällt |
| check-dag verwirft Fehler | `xtask/src/lib.rs` um Zeile 730: `let _ = compute_crate_layers(&mut crates)` \[G\] | besteht | K0.1 |
| engine hängt an license, Default zieht graph, rank, adapt, kvcache, candle | `engine/Cargo.toml:18` und `:64` \[G\] | besteht | K1.7, K2 Schritt 1 |
| mcp zieht router, encryption-at-rest und license | `mcp/Cargo.toml:22` und `:25`, `default = []` \[G\] | besteht | K2 Schritt 3 |
| Fünf Downcasts auf LsmStorage | `collection/mod.rs:405`, `:528`, `:532`, `crud/delete.rs:91`, `transaction/db_transaction_commit.rs:35` \[G\] | bestätigt, alle fünf | K2.9 |
| WAL-Retire fail-open | `store/src/wal/retire.rs:39` `while let Ok(Some(entry))` \[G\] | besteht | K1.10 |
| Fehlerabbildung im Purge | `lsm/ops/purge.rs`: 11 Treffer für `CompactionBusy` \[G\]. Der Port-Fehlertyp hat dafür kein Busy, aber `Io`, `Manifest`, `ReadError` \[G\] | besteht im Store | K1.10 |
| Resume und Ledger ohne Produktionsaufrufer | `resume_pending` nur in `purge/resume.rs`, `wiring.rs`, `mod.rs`; `ledger_append` nur `lsm/engine.rs:113` \[H\] | besteht | K1.12, K3 |
| Kryptoverfahren | `crypto/Cargo.toml:20`: `aes-gcm-siv = 0.11` \[G\] | bestätigt | DE-09 |
| Zwei Unwrap-Baselines | `governance/ratchet.toml`: `unwrap_exact = 502` \[G\] | besteht | K0.5 |
| Toolchain gepinnt | `rust-toolchain.toml`: 1.89.0 \[G\] | bestätigt | S1 |
| Gate-Katalog und Governance-Dateien | `governance/` enthält gates.toml, ratchet.toml, protected-paths.toml und weitere; `security_ports.toml` fehlt, `products/` fehlt \[G\] | besteht | K0.6, K2 |
| EP-Spezifikationen und Verweise | `docs/EP1` bis `EP6_*_SPEC.md`, `GESAMTSPEZIFIKATION.md`, `SOURCE_OF_TRUTH.md` vorhanden \[G\] | vorhanden | Anhänge je Produkt, siehe 3 und 10 |

**Regel:** Zahlen zu Hülle (25 und 22), Ring-Verletzungen (6) und unverdrahteten Funktionen (96) bleiben \[E\] oder \[H\], bis K0.9 sie mit `cargo` misst.

## 3. Finales Entscheidungsregister

Alle in v3 als offen oder zu bestätigen geführten Punkte sind entschieden. Jede Entscheidung nennt, was verworfen wurde. Eine Entscheidung ändert sich nur per ADR.

| Nr. | Frage | Entscheidung | Begründung und verworfene Alternative |
| --- | --- | --- | --- |
| DE-01 | Umfang v0.1.0 | Off und Assured im Kern. Attested ist ein Plugin außerhalb der Default-Hülle und ab EP2 nutzbar. v1.0 ist ein Gate, kein Termin | Ein belegter Assured-Modus ist die Voraussetzung jedes Nachweises. Verworfen: Attested in v0.1.0 |
| DE-02 | Standardmodus | Off. Ledger, Fence und Resume gehören zu Assured, in Off gibt es keine Datei, kein Feld, keinen Task | Prinzip P6. Der fehlende Restore-Schutz in Off ist dokumentierte Nicht-Garantie. Verworfen: Ledger immer an |
| DE-03 | Port-Modul | `contextra-ports::erasure` (514 Zeilen am HEAD) wird weiterentwickelt: Provenienz im PhysicalScope, DeleteOutcome um Attested, Statusvokabular, LayerEraser. Kein Neuschreiben | Bestand und Tests sind vorhanden. Verworfen: Neubau nach v3-Entwurf |
| DE-04 | Backend-Erkennung | `StorageEngine::erasure_backend() -> Option<&dyn ErasureBackend>`. `StorageErasureExt` entfällt. Der Methodenname `purge_physical` des HEAD bleibt, die Signatur erhält `&FenceGuard` | Die Engine hält `Arc<dyn StorageEngine>`, ein zweiter Trait bräuchte einen weiteren Cast. Alle fünf Downcasts verschwinden |
| DE-05 | Fence | Option B in der Form aus Abschnitt 6.3: Check in der ersten Haltephase des `commit_mutex` vor Sequenzvergabe, Drain-Marke beim Erwerb. Rückfall Option A, wenn Loom oder Benchmark scheitern | Der Mutex wird vor dem WAL-Append freigegeben \[G\]. Ein Check nach dem Neuerwerb käme zu spät, die Daten wären schon im WAL |
| DE-06 | Schlüssellayout | Physische Schlüssel tragen nur die `CollectionId`. Das Layout `t:{tenant}:{collection_id}:` mit abschließendem Doppelpunkt ist kanonisch. Das Namenslayout `__col:{name}:\x00` wird per Upgrade-on-open (Formatversion) migriert und entfernt. Der Name lebt nur im Katalog | Der Store-Codec ist bereits ID-basiert \[G\]. Verworfen: Name im Schlüssel, weil Wiederanlage unter gleichem Namen den Purge der alten Collection träfe |
| DE-07 | Assured ohne Verschlüsselung | erlaubt. Ansprüche leiten sich aus der Fähigkeit ab (Abschnitt 13.1) | Kleine Installationen ohne Schlüsselverwaltung bleiben möglich. Verworfen: Verschlüsselungszwang |
| DE-08 | Crypto-Erase | primär für Collection und Mandant, zweistufig (kryptografisch sofort, Bezeichnerreste per Purge). Einzeldokumente per gebündeltem Rewrite | Rewrite je Collection wäre O(Gesamtdaten). Verworfen: Rewrite für alles |
| DE-09 | Kryptoverfahren | AES-256-GCM-SIV für v0.1.0. DEK-Rotation nach 2^32 Verschlüsselungen je Schlüssel. Kein Text behauptet BSI-Konformität, nur ein Algorithmen-Inventar | Nonce-Fehlgebrauch ist nach Restore und Rollback tolerierbar (RFC 8452), das Verfahren ist im Code \[G\]. Ein Krypto-Review kann nur mit einem Befund widersprechen. Verworfen: AES-GCM wegen Nonce-Katastrophe, XChaCha20 wegen Softwarepfad ohne Bedarf |
| DE-10 | Schlüsselhierarchie | Master-Schlüssel (KMS), KEK je Mandant, DEK je Collection, alle zufällig und gewickelt. Widerruf sofort, Vernichtung nach Wartefrist (Standard 72 Stunden) | Ein abgeleiteter Schlüssel ist nicht vernichtbar. Verworfen: HKDF für KEK und DEK |
| DE-11 | HNSW und Veto | VETO-F02 bleibt gültig. Löschschicht 26 arbeitet in v0.1.0 mit Tombstone-Pruning, Zeroisierung des Arena-Slots und Neuaufbau des Collection-Snapshots aus dem gefilterten LSM im Koaleszenzfenster. ADR-097 (Nachbarschaftsreparatur) wird nicht eingesetzt | Das Veto verbietet Reparatur, der Neuaufbau ist keine. Der Scan prüft Arena und Snapshot auf das Vektormuster. Verworfen: Veto aufheben |
| DE-12 | Modus-Downgrade | nur über die Offline-Aktion `erasure_downgrade` mit Bestätigungs-Token. Der Ledger wird archiviert, ein signierter Eintrag hält das Ende der Garantie fest. Nie über MCP oder Anfrage | Verhindert stilles Absenken. Verworfen: nur Neuaufbau (Downtime) |
| DE-13 | Stall außerhalb des Scopes | Schreibvorgänge anderer Scopes blockieren nur während der Mutex-Haltephase des Fence-Erwerbs (Obergrenze p99 5 ms), nie während des Rewrites. Abnahmetest `chaos_fence_contention` | Heute blockiert ein Purge den ganzen Store. Das ist für Produktion nicht tragbar |
| DE-14 | Parkplatz-Crates | Tag `park/EPn/v1`, danach ein PR mit Label `scope:core-cut` entfernt sie aus dem Workspace. Kein eigenes Repository, bis eine Ebene startet | Git-Historie und Tag bewahren den Code, ein Repository ohne Pflege wäre Ballast |
| DE-15 | EP2 | eigenes Repository nach K4. Format-Spezifikation und Verifier sind offen lizenziert | Erzeuger und Prüfer dürfen keinen Code teilen |
| DE-16 | Ring-Gate | Pflicht-Gate nach Ordnungs-Fix und ohne Kante engine zu license. Schicht L0 bis L4 steht nur in `capabilities.toml`. Gleiche Schicht verboten, außer in L0 nach deklarierter Ordnung | Eine Quelle statt drei Kopien. CI-Status ist aus dem Klon nicht ablesbar (K0.4) |
| DE-17 | Legacy-WAL | wird vor v0.1.0 entfernt, neuer ADR löst ADR-048, 055, 100 ab | Keine externen Konsumenten, weniger Formate |
| DE-18 | Lizenz | `LicenseGate` ist ein Port. Die Lizenz steuert nur das Ausstellen neuer Beweise. `open` und Löschpflicht hängen nie am Lizenzstatus. EP1 offen (MIT oder Apache-2.0) | Eine Löschpflicht darf nicht kommerziell abschaltbar sein |
| DE-19 | Rangordnung der Quellen | 1 Gates und Code für den Ist-Zustand, 2 Governance-Dateien, 3 akzeptierte ADRs, 4 diese Spezifikation für Soll und Reihenfolge, 5 Archiv. Soll-Ist-Widerspruch ist ein Befund | Ersetzt ADR-E36 und löst die drei widersprüchlichen Rangordnungen |
| DE-20 | Snapshot-Lebensdauer | In Assured Pflichtwert `max_snapshot_age`, Standard 15 Minuten, zulässig 1 Minute bis 24 Stunden. Abgelaufene Snapshots liefern `SnapshotPruned`. In Off kein Limit | Ein vergessener Lesevorgang darf den Purge nicht unbegrenzt blockieren |
| DE-21 | Beweis im Ergebnis | Nie. `forget` liefert nach dem Commit sofort einen Status, Beweis und Folgezustände über `erasure_status` | Ein Beweis entsteht erst nach Purge und Scans |
| DE-22 | Zeugen | Pull-Modell: Das Plugin ruft die LayerScans selbst auf. Scan, Bescheinigung und Ausschluss sind getrennt | Zeugen aus echten Scans, Mutationstest Scanner gleich No-op |
| DE-23 | SSI und SSN | SSI-Validator bleibt Standard. SSN frühestens nach K4 unter den Bedingungen aus 8.1 | Kein Parallelumbau am Kern |
| DE-24 | Zeitachsen | getrennte Parameter `as_of_tx` und `as_of_business`. TxId und TenantId mit geprüften Konstruktoren | ENG-FILTER-001 vermischt beides |
| DE-25 | Zweite Verifier-Implementierung | Pflicht für den EP2-Release, nicht für v0.1.0. Referenz in Python nur aus der Verifier-Spezifikation, differenzieller Korpus-Test | Unabhängigkeit der Abnahme |
| DE-26 | Zeitanker | Die TSA-Adresse ist Konfiguration ohne Voreinstellung. Ohne Anker trägt der Beweis den Vermerk „unverankert“ und die Ausstellungszeit gilt als Behauptung des Ausstellers | Kein stilles Vertrauen in einen Dienst |

**Aufgelöste Widersprüche zwischen den Quellplänen (Auszug der verbindlichen Wahl):**

| Nr. | Thema | Wahl |
| --- | --- | --- |
| 1 | Crate-Namen | `erasure-evidence` und `proof-verify` |
| 2 | Layer-Modell | Scan, Bescheinigung, Ausschluss getrennt (7.1) |
| 3, 15, 16 | Beweisformat | Tag-TLV ohne feste Offsets, Zeitstempel als Zahl, kein CBOR im Beweis (7.2) |
| 4, 25 | LayerScan und Backend | synchron, Ausführung auf dediziertem Thread oder Blocking-Pool |
| 5 | Backend-Erkennung | `StorageEngine::erasure_backend()` (DE-04) |
| 6 | Fence | Option B angepasst, Rückfall A (DE-05) |
| 7 | Ledger | gehört zu Assured |
| 8, 9 | Zeiträume und Phasen | v2-Zeiträume, Expertenwerte als Untergrenze, K-Meilensteine |
| 10 | AES-GCM gegen GCM-SIV | GCM-SIV (DE-09) |
| 11 | KvDeleteMode | TombstoneOnly, am HEAD bereits so \[G\] |
| 12 | RestoreGuard | Bloom nur zur Vorauswahl, exakte Prüfung danach |
| 13 | SSN gegen SSI | SSI (DE-23) |
| 14 | Irminsul und BSI-Mapping als Phase 1 | Freeze bis K4, KV-Cache ist EP4 |
| 17 | Layer-IDs | ein Register (7.1) |
| 18 | DurabilityMode | Full, WalNoHmac, MemoryOnly \[G\], Matrix 5.3 |
| 20 | KEK und DEK per HKDF | verboten (DE-10) |
| 23 | `find_k_paths` als MCP-Werkzeug | `pub(crate)` bis EP3 |
| 24 | Resume | Hintergrund-Task nach Storage-Aufbau (6.4) |
| 27 | Betriebsprofile | in v0.1.0 nur Balanced und Assured |
| 28 | Hüllen-Gate | je Binary, `-p contextra-mcp` |

## 4. Zielarchitektur des Fundaments

Contextra ist ein lokaler, deterministischer Speicher für KI-Agenten auf einem Knoten: Dokumente, Hybridsuche (Vektor, BM25, Fusion), Mandantentrennung, MVCC-Transaktionen und eine Löschgarantie, die in dem Maß belegbar ist, in dem man sie einschaltet. Es gibt keine Cluster-, Sharding- oder Raft-Logik. Das Versprechen hat zwei getrennte Teile: die **Löschgarantie** (Daten sind weg, auch nach Absturz, Restore und Replay; Kern, ab Assured) und den **Löschnachweis** (signiertes, unabhängig prüfbares Audit mit ausgewiesenen Ausschlüssen; Plugin, Attested, kein mathematischer Beweis).

### 4.1 Prinzipien (verbindlich)

| Nr. | Prinzip | Prüfung |
| --- | --- | --- |
| P1 | Zero-Panic: kein `unwrap`, `expect`, `panic!`, kein roher Indexzugriff im Produktionspfad | ratchet, doctrine-scan |
| P2 | Schichten-DAG: Abhängigkeiten nur abwärts | check-ring-layering |
| P3 | Determinismus: Uhr, Zufall, IDs nur über die Ports Clock, Rng, IdGenerator | determinism-check |
| P4 | `unsafe` nur in simd, sys, wire, mit SAFETY-Kommentar. Überall sonst `forbid(unsafe_code)` | gate-unsafe-audit |
| P5 | Single-Node | scope-guard |
| P6 | Kosten sind opt-in: Was nicht aktiv ist, kostet weder Code noch CPU | Abschaltprobe, Hot-Path-Benchmark |
| PE1 | Kern-Crates sprechen nur mit `contextra-ports`, nie mit Crypto-Plugins | dependency-graph-audit |
| PE2 | Schreibpfad: höchstens ein Atomic-Load je Batch | Benchmark |
| PE3 | Fail-closed: Zeugen nur aus echten Scans, Fehler sind sichtbare Zustände | Mutationstest Scanner gleich No-op |
| PE4 | ErasureMode ist eine monotone Hochwassermarke im Manifest | Downgrade-Test |
| PE5 | Gemergt ist nicht wirksam: Symbol braucht Produktionsaufrufer und End-to-End-Test über die Fassade | deletion-guarantee-gate |
| PE6 | Jeder Löschtest läuft über `LsmStorage` und `TenantScopedStorage` | Tenant-Matrix |
| PV1 | Builder-Ehrlichkeit: kein Schalter ohne Wirkung und Prüfung | check-builder-wiring |
| PV2 | Jedes Feature kompiliert allein | check-feature-isolation |
| PV3 | Snapshot-Konsistenz: alle Signale einer Anfrage lesen denselben Stand | check-snapshot-consistency |
| PV4 | Sperrenhierarchie, keine Locks über `await` (8.5) | Loom, gate-lock-audit |
| PV5 | Jedes MCP-Werkzeug ist klassifiziert und feature-gated | MCP-Registry-Test |

### 4.2 Schichten und Crates

Die Architektur-Ringe heißen **Schicht L0 bis L4**. Der Begriff FeatureRing (Fast, Sovereign, Compliance) bleibt Lizenz-API. Die Schicht steht nur in `capabilities.toml`, Kopien im Gate und in den Cargo-Metadaten werden daraus abgeleitet (DE-16).

| Schicht | Crates (EP1) | Rolle |
| --- | --- | --- |
| L0 | types, sys, wire, simd, durable-fs, crypto-core | Grundtypen, die drei Unsafe-Inseln, dauerhafte Dateiarbeit, I/O-freie Kryptografie |
| L1 | ports | alle Verträge |
| L2 | mvcc, text, rank, store, vector, checkpoint | Datenhaltung und Suche |
| L3 | engine, db | Koordination, Löschkoordinator, Collection-Logik |
| L4 | contextra (Fassade, Composition Root), mcp | Builder, Werkzeuge, Server |

Gleiche Schicht ist verboten, außer innerhalb von L0 nach der in `capabilities.toml` deklarierten Ordnung und für Dev-Abhängigkeiten. `durable-fs` steht in dieser Ordnung. **Plugin-Crates** außerhalb der Default-Hülle: `crypto-kv`, `erasure`, `erasure-evidence`, `proof-verify`, `audit-export`, `avv-generator`. Nur die Fassade kennt Kern und Plugins. Pfeile von store, vector, engine, db auf `erasure-evidence` sind verboten, auch optional. Plugins sind eigene Crates und keine Features, weil Cargo Features je Build-Graph vereinigt.

### 4.3 Ports (`contextra-ports::erasure`, Weiterentwicklung des Bestands)

Alle Löschports sind **synchron** und haben keine Panikpfade. Der Koordinator führt sie auf einem dedizierten Purge-Thread oder per `spawn_blocking` aus, nie auf dem Async-Executor. Zeit, Zufall und IDs kommen aus den Ports (P3). `PurgeId` stammt aus dem `IdGenerator`.

Änderungen gegenüber dem Bestand am HEAD:

| Element | Bestand \[G\] | Soll |
| --- | --- | --- |
| LogicalScope | Varianten Document, Collection, Tenant, jede mit `tenant_id` | unverändert. Der Storage-Wrapper prüft `tenant_id` gegen den eigenen Mandanten, sonst `ForeignScope` |
| PhysicalScope | private Felder `keys`, `prefixes`, Konstruktor `try_new` hinter Feature `backend-impl`, prüft das Präfix `t:<tenant>:` | zusätzlich privates Feld `origin: ScopeOrigin` (`backend_id` als zufälliger, im Manifest persistierter `u64`, plus Digest des Mandanten). Jede Backend-Methode prüft `origin` und liefert sonst `ForeignScope`. Ein Feature-Gate ist keine Sicherheitsgrenze, weil Cargo es crate-übergreifend vereinigt |
| Präfix-Invariante | Präfix beginnt mit `t:<tenant>:` | zusätzlich: jedes Präfix endet auf `:`, sonst `MalformedScope`. Pflichttest mit den IDs 4 und 42 |
| Backend-Erkennung | `StorageErasureExt::as_erasure_backend` | `StorageEngine::erasure_backend()` (DE-04), Default `None`. `TenantScopedStorage` gibt das innere Backend zurück und ergänzt den Mandanten in `physical_scope` |
| ErasureBackend | `physical_scope`, `acquire_fence`, `purge_physical(scope, as_of, probe)`, kein `dispose` | `purge_physical` erhält `&FenceGuard`. Neu: `dispose(files)` |
| FenceGuard | `pub purge_id`, nicht `Clone` | Drop gibt die Fence frei. Der Guard wird in den Hintergrund-Task verschoben, nie in der aufrufenden Funktion gehalten |
| PurgeError | InsufficientDiskSpace, FenceConflict, ReadError, Io, Manifest, ProbeFailed, SnapshotPinned | zusätzlich `WriteError{file}` und `CompactionBusy` (nur für echten Busy-Zustand). `Io` darf nie auf Busy abgebildet werden |
| DeleteOutcome | Logical, Assured mit `purge_id` | zusätzlich Attested. Der Beweis steht nie im Ergebnis (DE-21) |
| EraseError | NotStarted, Indeterminate | `delete` gibt nur `NotStarted` zurück. Fehler nach dem Commit-Punkt sind `PurgeStatus::Pending` mit Grund |
| PurgeState | Admitted, Flush, PurgeCompaction, Dispose, WalRetire, Witness, Finalize, Release | bleibt die interne Phasenmaschine. Das öffentliche `PurgeStatus` wird daraus abgeleitet (5.2) |
| LayerScan | `layer()`, `scan(&PhysicalScope)` | bleibt. Neu daneben `LayerEraser` (idempotent) |

Neue und geänderte Signaturen (Illustration der Form, nicht kopierbar):

```rust
pub trait StorageEngine: Send + Sync + 'static {
    fn erasure_backend(&self) -> Option<&dyn ErasureBackend> { None }
}

pub trait ErasureBackend: Send + Sync {
    fn physical_scope(&self, scope: &LogicalScope) -> Result<PhysicalScope, ErasureError>;
    fn acquire_fence(&self, scope: &PhysicalScope, id: PurgeId) -> Result<FenceGuard, ErasureError>;
    fn purge_physical(&self, scope: &PhysicalScope, as_of: TxId,
                      probe: &dyn FreeSpaceProbe, fence: &FenceGuard)
                      -> Result<PurgeOutput, PurgeError>;
    fn dispose(&self, files: &[ObsoleteFile]) -> Result<DisposeReport, DisposeError>;
}

pub trait LayerEraser: Send + Sync {
    fn layer(&self) -> LayerId;
    fn erase(&self, scope: &PhysicalScope, as_of: TxId) -> Result<EraseReport, ErasureError>;
}
```

Jeder Layer-Eigentümer (store, vector, text, graph, kvcache) implementiert `LayerScan` und `LayerEraser` hinter den Features `erasure-scan` und `erasure` (Standard aus). `FreeSpaceProbe::available_bytes() -> Option<u64>`: `None` heißt unbekannt, `Some(0)` heißt kein Platz. Weitere Ports: `WriteGate`, `RestoreGuard` (streamend), `LicenseGate` (nur das Token `AuthorizedRing`), `KmsProvider`, `EgressGuard`, `PluginManifest`, `Clock`, `Rng`, `IdGenerator`, `ReadSnapshot`, `MergeOperatorFactory`, `ConsolidationTask`, `EmbeddingProvider`, `LlmTextGenerator`.

Fehlertypen (abschließend je Typ im Code):

| Typ | Varianten |
| --- | --- |
| ErasureError | NotAvailable, PolicyViolation{current, requested}, NotConfigured, ForeignScope, MalformedScope, LedgerCorrupt, Internal |
| DisposeError | ScrubRefused{file, reason}, RemoveFailed{file}, SyncFailed |
| ScanError | Io, CorruptedIndex, Incomplete |
| ResumeError | ScopeNotInLedger{purge\_id}, IntentCorrupt, LedgerCorrupt, StoreNotReady |
| RestoreError | GuardRequired{mode}, LedgerCorrupt, Io |

### 4.4 Produkt-Manifest und Hülle

Ein Produkt wird durch `products/EPn.toml` definiert (Kennung, Status, erlaubte Crates, Build-Features). Das Gate **check-product-hull** führt `cargo tree -p contextra-mcp -e normal --no-default-features` mit den Build-Features aus und scheitert, wenn ein Crate außerhalb der Liste auftaucht. Der Lauf mit allen Features ist nur ein Bericht. Ein Modul ist abschaltbar, wenn es hinter einem Feature liegt, das Manifest es nicht listet und der Build ohne das Feature besteht. Die Abwesenheit der Plugins wird je Binary belegt (contextra, contextra-mcp, contextra-py), nie mit `--workspace`.

| Metrik \[E\] | Start | nach K2 | Release v0.1.0 |
| --- | --: | --: | --: |
| Hülle mcp, Default-Features | 25 | höchstens 18 | 18 |
| Hülle contextra, Default | 22 | höchstens 17 | 17 |
| Hülle engine ohne Default-Features | 15 | 15 | 15 |
| Parkplatz-Crates in der mcp-Hülle | 7 | 0 | 0 |
| Kanten Kern zu Parkplatz (regulär) | 9 | 0 | 0 |
| Ring-Verletzungen | 6 | 0 | 0 |

Produkt EP1 (Memory) umfasst: types, ports, sys, wire, simd, durable-fs, crypto-core, mvcc, text, rank, store, vector, checkpoint, engine, db, contextra, mcp. `rank` gehört zum Kern (nur Fusion und Kalibrator). `contextra-core` wird ersatzlos entfernt (405 Dateien referenzieren es, 3 bis 4 mechanische PRs).

### 4.5 Entfernung von Funktionen aus dem Kern (verbindlich)

PR1 Entfernen statt Einfrieren: Ein Feature, das den Kern destabilisiert oder dessen Umfang nicht durch S1 bis S12 gedeckt ist, wird aus main entfernt. Es wird nicht durch Feature-Flags, cfg-Blöcke oder auskommentierten Code im Build gehalten.
PR2 Wiederherstellbarkeit: Jede Entfernung erzeugt vorher einen annotierten Tag park/<Feature-ID>/<Version> auf dem letzten Stand mit dem Feature. Wiederherstellung über git checkout park/<ID>/<Version> -- <Pfade> in einem eigenen Branch, nie durch Zurückschreiben in main.
PR3 Entfernung ist ein eigener Schritt: eigene Karte, eigener Tag, eigener PR. Keine Vermischung mit Reparatur oder Umbau.
PR4 Wiedereinführung nur per Gate: Rückkehr nur, wenn (a) die Voraussetzungen der Ebene nach 11.1 erfüllt sind, (b) S1 bis S12 grün sind, (c) ein Produkt-Manifest und ein Hüllen-Gate existieren. Der PR nennt den Tag, aus dem er übernimmt.
PR5 Keine Halluzinationsfläche: Im Hauptzweig existiert nur Code mit Produktionsaufrufer und Test über den echten Pfad (PE5).

## 5. Löschmodi, Ablauf, API und Schlüssel

### 5.1 Die drei Modi

| Eigenschaft | Off (Standard) | Assured | Attested |
| --- | --- | --- | --- |
| Löschmethode | Tombstone, reguläre Compaction | Footprint, Ledger, Tombstone, Purge, Crypto-Erase | wie Assured |
| Ledger, Präfix-Fence, Resume | keine | ja | ja |
| Restore-Schutz | keiner (dokumentierte Nicht-Garantie) | RestoreGuard Pflicht, fehlt er, ist es ein Startfehler | plus Eintrag im Beweis |
| Zeugen, Beweis | keine | keine | echte Scans, Beweis v4, Ed25519 |
| Zeitanker, Transparenzlog | keine | keine | RFC 3161, RFC 9162 |
| Compile-Feature | Kern | `erasure` | `erasure` plus `erasure-evidence` |
| Lizenzring | keiner | keiner | Sovereign |
| Normalbetrieb | null: kein Feld, keine Datei, kein Task | ein Atomic-Load je Batch, ein Ledger-Append je Löschung | wie Assured |
| Löschung | Tombstone | Collection und Mandant O(1) per Crypto-Erase (mit Verschlüsselung), Dokument gebündelter Rewrite im Hintergrund | zusätzlich Scans und Signatur auf Anforderung |

Budgets (Obergrenzen, vor Festlegung zu messen): `erasure` in Off 0 Prozent; Assured im Leerlauf höchstens 1 Prozent p99 beim Schreiben, Start höchstens 50 ms; Assured während eines Purge höchstens 5 Prozent, Schreibvorgänge außerhalb des Scopes blockieren nie (DE-13); `encryption-at-rest` höchstens 3 Prozent p99; `erasure-evidence` Start höchstens 100 ms.

### 5.2 Ablauf in Assured (schrittgenau)

In Off ruft `delete` direkt `storage.delete`: kein Footprint, keine Zeugen, keine Allokation. In Assured und Attested führt der `DeletionCoordinator` der Engine diese Schritte aus. **Der Commit-Punkt ist der Ledger-Append, danach der Tombstone.**

| Schritt | Aktion | Dauerhaft vermerkt | Fehlerverhalten |
| --- | --- | --- | --- |
| 1 | `hooks.begin`, Footprint einmal über die KeyFamilyRegistry (alle abgeleiteten Speicher) | nein | Err an den Aufrufer, noch nichts gelöscht |
| 2 | Ledger-Append `Intent` mit fsync (Gruppen-fsync erlaubt) | Ledger | Err an den Aufrufer |
| 3 | Tombstone-Commit (Dokument) oder Katalog-Tombstone (Collection, Mandant) | WAL | wiederholbar aus dem Ledger (CRASH-01) |
| 4 | `hooks.tombstoned`, `LayerEraser` für RAM- und persistierte Indizes, KV-Schlüssel widerrufen | RevocationLog | ab hier kein Err mehr, Fehler werden `PendingReason` |
| 5 | Intent-Datei `purge/<purge_id>.intent` mit Datei- und Verzeichnis-fsync | Intent | wie Schritt 4 |
| 6 | Hintergrund: Startbedingung `snapshot_floor > erased_at_tx` | Status | `Pending{OpenSnapshot}` mit Wiederholzeitpunkt |
| 7 | `acquire_fence`, Drain-Marke abwarten (6.3) | Intent | `Pending{Fenced}` |
| 8 | WAL-Barriere: WAL rotieren, Memtables flushen, danach Segmente bis zum Flush-Stand stilllegen und scrubben | Manifest | `Pending{ScrubRefused}` |
| 9 | `purge_physical` (6.2) | Manifest | typisierter `PurgeError`, Dateien unverändert |
| 10 | `dispose` mit Scrub und Verzeichnis-fsync | Dispose-Report | `Pending{ScrubRefused}`, kein Rückfall auf normales Löschen |
| 11 | Fence freigeben, Ledger-Append `Purged` (nur Digests), Intent entfernen | Ledger | wiederholbar |
| 12 | Nur Attested: Scans, Beweis, Zeitanker (7) | Proof-Store | `ScanFailed`, nie ein Beweis |

**Fehlersemantik:** Nach dem Tombstone-Commit gibt es für die Löschung kein `Err`. Ein Fehler der Prüfung `ConcurrentModification` nach dem Commit darf den Aufrufer nicht mit `Err` zurücklassen, obwohl gelöscht wurde.

**Öffentliches Statusvokabular** (`PurgeStatus`, abgeleitet aus der internen `PurgeState`-Phasenmaschine des Bestands):

| PurgeStatus | Bedeutung | Abgeleitet aus PurgeState |
| --- | --- | --- |
| `Pending{reason, since_ms, retry_at_ms}` | wartet. Gründe: OpenSnapshot, DiskSpaceUnknownOrLow, ScrubRefused, CompactionLease, Fenced, ReadError, WriteError, ManifestError, KeyWaitPeriod | Admitted, oder Fehler in jeder Phase |
| `InProgress{sstables_done, sstables_total}` | Rewrite läuft | Flush, PurgeCompaction, Dispose, WalRetire |
| `Purged` | physisch bereinigt | nach Dispose und WalRetire |
| `Completed` | Assured abgeschlossen | Finalize, Release |
| `ScanFailed{layer}` | Seitenzustand, nie ein Beweis | Witness mit Fehler |
| `Attested{proof_ref}` | Beweis liegt vor | Witness, Finalize |

Die Antwort `assured_verified` setzt Stufe C2 voraus (13.1).

### 5.3 Hochwassermarke und Durability-Matrix

Der höchste je aktive Modus steht im Manifest (HMAC-geschützt). `open` mit niedrigerem Modus liefert `PolicyViolation`. Der Modus bleibt je Öffnen unveränderlich. Ein Downgrade geschieht nur über die Offline-Aktion `erasure_downgrade` (DE-12). Der Wechsel von Off zu Assured deckt zunächst nur Löschungen ab dem Wechsel ab. `erasure_adopt()` schreibt einen `AdoptMarker` mit `tx_adopt`, hebt die Marke an und startet einen unterbrechbaren Lauf, der alle vorhandenen Tombstones koalesziert purgt. Bis dahin gilt `assured_verified` nur ab `tx_adopt`, ältere Löschungen sind `assured_pending`.

| DurabilityMode am HEAD \[G\] | Off | Assured | Attested |
| --- | --- | --- | --- |
| `Full` | erlaubt | erlaubt | erlaubt |
| `WalNoHmac` | erlaubt | erlaubt mit Warnung | verboten (HMAC-Kette nötig) |
| `MemoryOnly` | erlaubt | verboten | verboten |

Die Matrix wird im Builder und beim Öffnen des Manifests durchgesetzt. Standardprofil ist **Balanced** (Off, keine Verschlüsselung, Full). Compliance nur explizit.

### 5.4 API-Vertrag und MCP

Der Kern-`delete` kennt kein Assurance-Feld. `ForgetOptions` lebt nur im Plugin, dadurch ist der Capability-Fehler bei Mandanten-Storage strukturell unmöglich. Antwortvokabular von `forget` und `drop_collection`: `logically_deleted`, `assured_pending`, `assured_verified`, `assured_unverified`, `attested`. Kein Beweisfeld ohne Beleg. `forget` kehrt nach dem Tombstone-Commit sofort zurück. Optional blockiert `wait_ms` bis `Purged`. Fordert eine Anfrage einen höheren Modus als die Instanz, antwortet der Server mit `PolicyViolation` und senkt nie still ab. Die Mandanten-ID kommt aus der Sitzung, nie aus einer Konstante. Zustände und Beweis werden über `Contextra::erasure_status(purge_id)` und das MCP-Werkzeug `contextra_erasure_status` (ReadOnly, nur mit Feature `erasure`) abgerufen.

### 5.5 Schlüsselhierarchie (DE-10)

| Stufe | Erzeugung | Speicherung | Vernichtung |
| --- | --- | --- | --- |
| Master-Schlüssel | KMS, HSM oder OS-Keystore über `KmsProvider` | nie im Klartext im Datenverzeichnis | gehört dem Betreiber |
| KEK je Mandant | zufällig (CSPRNG über `Rng`), 256 Bit | gewickelt unter dem Master-Schlüssel | gewickelten Blob überschreiben, fsync, scrubben |
| DEK je Collection | zufällig, 256 Bit, mit Generationszähler | gewickelt unter dem KEK | wie KEK |
| KV-Segment-Schlüssel (Tier 2) | zufällig je Segment und Version | gewickelt unter dem KEK | wie KEK |
| Zweckgebundene Unterschlüssel | HKDF-SHA-256 mit festem Label | nicht gespeichert | nicht shreddbar, tragen keine Nutzdaten |

Erlaubte HKDF-Labels: `wal-hmac/v1`, `audit-chain/v1`, `ledger/v1`, `manifest-hmac/v1`. **Regel:** Ein Schlüssel, der geshreddet werden soll, darf nicht aus einem überlebenden Schlüssel ableitbar sein. Ein Test prüft, dass kein Shred-Schlüssel ein HKDF-Label trägt. Schlüsseltypen sind `Zeroize` und `ZeroizeOnDrop` und nicht `Clone`. Der Nachweis ist ein `compile_fail`-Doctest (`impl !Clone` ist auf stabilem Rust 1.89 nicht zulässig). `KvDeleteMode::default()` ist `TombstoneOnly` \[G\]. `CryptoShred` gibt es nur mit dem Feature `encryption-at-rest`. `KmsProvider` und `LocalKmsProvider` werden im Builder als Standard-KMS-Port registriert.

### 5.6 Dreistufige Vernichtung

| Phase | Zeitpunkt | Wirkung | Zustand |
| --- | --- | --- | --- |
| Widerruf | T0 | RevocationLog-Eintrag mit fsync (fail-closed), RAM-Kopien zeroisiert, Schlüssel nicht mehr verwendbar | `Revoked{destroy_at}` |
| Wartefrist | T0 bis T0 plus Δ | gewickeltes Material existiert noch, ein Operator mit Schlüsselzugriff kann es wiederherstellen | `Revoked` |
| Vernichtung | T0 plus Δ | gewickelter Blob überschrieben, fsync, scrub, Vernichtungsprotokoll mit Zeit und Schlüssel-ID | `Destroyed` |

Δ ist standardmäßig 72 Stunden und nur mit dem Flag `immediate_destruction` null. Während der Wartefrist antwortet das System `assured_pending` mit `key_state: revoked`. Erst nach der Vernichtung darf `crypto_erased` ausgewiesen werden. Sicherungen des Schlüsselspeichers vor dem Löschzeitpunkt enthalten den vernichteten Schlüssel und fallen unter Ausschluss 3 (7.5). Schlüsselverlust ist Datenverlust, deshalb braucht jede Installation mit Crypto-Erase eine Schlüssel-Sicherung des Master-Schlüssels beim Betreiber.

### 5.7 Abdeckung des Crypto-Erase je Layer

Der Store hat heute einen Schlüssel je SSTable. Ein Schlüssel je SSTable kann keine Collection shreddern, weil eine SSTable mehrere Collections enthält. Deshalb gilt:

| Layer | Schutz |
| --- | --- |
| LSM-Werte | Verschlüsselung auf Record-Ebene vor dem WAL-Append mit dem DEK der Collection. Der Record-Kopf trägt Schlüssel-ID, Generation und Nonce. Der SSTable-Schlüssel bleibt als äußere At-rest-Schicht (beide koexistieren) |
| WAL-Frames, Memtable | tragen Chiffretext unter dem Collection-DEK. Lesen entschlüsselt (Budget 3 Prozent) |
| HNSW-Snapshot, Text-Segmente, Relationen | je Collection eine Datei, verschlüsselt unter dem Collection-DEK. Die RAM-Arena bleibt Klartext, `zeroize` beim Drop |
| KV-Cache Tier 2 | Segment-Schlüssel (EP4) |
| Checkpoints | enthalten Chiffretext, nach Vernichtung des DEK unlesbar. Status je Kopie |
| Schlüssel und Dokument-IDs im LSM | **nicht verschlüsselt** (Metadatenrest) |

Crypto-Erase macht Nutzlast, Vektoren und Textindex unlesbar, nicht die Bezeichner. Ein Collection-Drop ist deshalb zweistufig: sofort O(1) logisch und kryptografisch, danach Hintergrund-Purge der Schlüsselreste. Pro-Collection-Dateien werden direkt gescrubbt und gelöscht. `assured_verified` für einen Collection-Drop verlangt beide Stufen (Rohträger-Test mit Bezeichner-Canary). Ohne Feature `encryption-at-rest` arbeitet Assured ausschließlich über Rewrite-Purge.

### 5.8 Kryptoverfahren und RevocationLog

Verfahren: AES-256-GCM-SIV, HKDF-SHA-256, Ed25519, BLAKE3 (DE-09). Pflichtpunkte des Krypto-Reviews bleiben Nonce-Länge, Schlüsselaufbau (5.5), das `unsafe` im Test und der fail-closed Widerruf. Kein Text behauptet „BSI TR-02102-konform“, zulässig ist ein Algorithmen-Inventar. Das RevocationLog ist fail-closed (ein gescheiterter Widerruf ist ein Fehler), trägt eine Generation je Eintrag, die im Manifest gespiegelt wird (Rollback-Erkennung), prüft im Lesepfad per atomarer Bitmaske (`AtomicU64`-Slices) statt Sperren und liegt in `crypto-kv`. Die Compaction überspringt Einträge widerrufener Schlüssel. Crash-Test CRASH-13.

## 6. Purge v2, Fence, Resume, Ledger, Restore und Crash-Matrix

Der Purge-Kern muss vor jedem Crate-Split korrekt arbeiten. Ohne ihn ist jede Aussage über Garantie oder Leistung wertlos.

### 6.1 Korrekturmatrix

| Nr. | Befund | Korrektur | Stand am HEAD |
| --- | --- | --- | --- |
| L4 | Obsolete SSTables bleiben liegen | `PurgeOutput.obsolete_files` wird an `dispose` gereicht und in die Intent-Datei geschrieben | Feld im Port vorhanden \[G\], Wirkung laut v3 gesetzt, per Rohsweep unbelegt |
| L5 | Lesefehler laufen wie Dateiende | Schleife mit `Ok(Some)`, `Ok(None)`, `Err`. Err liefert `ReadError`, Dateien unverändert | purge.rs laut v3 behoben, `wal/retire.rs:39` offen \[G\] |
| L6 | Umgeschriebener Reader am falschen Platz | alter Reader wird an derselben Position ersetzt, rank und Sequenzbereich bleiben | unbelegt, Differenztest I2 |
| L8 | Probe `None` gleich null Byte frei | `None` heißt unbekannt, konservativ eine SSTable gleichzeitig. `Some(0)` heißt kein Platz. Produktiv-Probe über `sys` | offen |
| L12 | `CompactionBusy` für beliebige I/O-Fehler | eigene Varianten, Busy nur für Busy | offen, 11 Treffer in purge.rs \[G\] |
| N-K | Logische Präfixe treffen nicht | `purge_physical` nimmt nur `PhysicalScope`, ID-basierte, mit `:` terminierte Präfixe (DE-06) | offen |
| N-E | Eine Intent-Datei je Store | Datei je `purge_id`, Fence mit Referenzzähler je Präfix | offen |
| N-F | Vec und doppeltes Lesen | Streaming-Rewrite, Speicher-Peak unabhängig von der SSTable-Größe | offen |
| N-G | Orphan-Aufräumen ohne Scrub, `manifest_generation` wird auf 1 gesetzt | `dispose` mit Scrub, Generation nie zurücksetzen | offen |
| N-J | Failpoints über Umgebungsvariablen | nur unter Feature `fault-injection`, `grep` auf Umgebungszugriff gleich null | offen |
| N-I | Beweise v3 werden noch erzeugt | `DeletionProof::create` (v3) entfernen, v3 nur lesend | offen |

### 6.2 Purge v2 (Algorithmus)

1. **Budget:** `probe.available_bytes()`. `None` heißt konservativ genau eine SSTable gleichzeitig. `Some(n)` kleiner als die größte Kandidaten-SSTable plus Reserve ergibt `InsufficientDiskSpace`.
2. **Kandidaten:** nur SSTables, deren Schlüsselbereich oder Bloom-Filter einen Treffer erlauben. Die Auswahl darf Treffer nie auslassen (Bloom ohne falsche Negative).
3. **Streaming-Rewrite je Kandidat:** ein Durchlauf, Ausgabe nach `<name>.purge_tmp`. Einträge im Scope mit `seq <= as_of_tx` entfallen, alle anderen werden übernommen. **Jeder Lesefehler** bricht mit `ReadError` ab, Quelldatei und Manifest bleiben unverändert (Test: Dateihash vor und nach der Fehlerinjektion gleich).
4. **Ersetzung:** Der neue Reader erhält rank, Ebene und Sequenzbereich des alten und ersetzt ihn an derselben Position. Sein Schlüsselbereich ist eine Teilmenge des alten, die Nicht-Überlappung ab Ebene 1 bleibt. Bloom-Filter und Metadaten werden neu berechnet. Eine leere neue SSTable entfällt, die alte ist dann obsolet.
5. **Dauerhaftigkeit:** neue Datei fsync, Verzeichnis-fsync, dann ein einziger atomarer Manifest-Eintrag (Replace oder Remove), danach `obsolete_files`, danach `dispose`. Ein Absturz an jeder Stelle ist durch CRASH-02 bis 04 abgedeckt.
6. **Tombstones:** Der Purge entfernt auch Tombstones im Scope mit `seq <= as_of_tx`. Danach trägt der Ledger den Schutz vor Wiederauferstehung. Einträge mit `seq > as_of_tx` (Neuanlage nach der Löschung) bleiben.
7. **Koaleszenz:** Ein Sammelfenster (Standard 5 Sekunden) fasst mehrere ausstehende Scopes zu einem Rewrite zusammen. Die Kosten je Löschung sinken von O(SSTable) auf O(SSTable geteilt durch Löschungen im Fenster).
8. **Compaction-Ausschluss:** Purge und Hintergrund-Compaction schließen sich über eine `CompactionGate`-Lease aus. Sie wird nach jeder SSTable freigegeben, damit keine Schreibdrosselung entsteht. Zustand `Pending{CompactionLease}`, kein Fehler.
9. **Fehlerabbildung:** Jede I/O-Fehlerart hat ihre eigene Variante.

Startbedingung und WAL-Barriere stehen in 5.2 (Schritte 6 und 8). Die Barriere ist nötig, weil der WAL nach dem Tombstone weiter die Nutzdaten trägt und ein Segment erst stillgelegt werden darf, wenn alle seine Einträge in SSTables stehen. Ohne sie ist der Rohträger-Test nicht erfüllbar.

### 6.3 Fence-Protokoll (DE-05)

**Befund am Klon:** Der `commit_mutex` ist ein `tokio::sync::Mutex`. Der Leader hält ihn während der Batch-Vorbereitung und der Sequenzvergabe, gibt ihn vor dem WAL-Append frei und erwirbt ihn für Memtable und Sichtbarkeit neu \[G\]. Ein Fence-Check nach dem Neuerwerb käme zu spät, weil die Daten dann schon im WAL stehen. Ein Fence-Erwerb zwischen den beiden Haltephasen würde Batches übersehen, die bereits zugelassen, aber noch nicht angewendet sind.

**Normatives Protokoll:**

1. **Struktur:** `Option<ErasureGate>` im LsmStorage (kein `dyn`, Feld nicht öffentlich, ohne Feature `erasure` ein ZST). Inhalt: `active: AtomicBool` und `fences: ArcSwap<Vec<PrefixFence>>`, wobei `PrefixFence{prefix, ref_count, purge_id}`.
2. **Check:** In der ersten Haltephase, nach `drain_kv` und **vor** der Sequenzvergabe. Der Check prüft **alle** Operationen des Batches einschließlich der eigenen des Leaders mit einem Acquire-Load von `active`. Nur bei `active` wird gegen die Präfixliste geprüft. Abgewiesene Follower erhalten `WriteFenced{prefix, purge_id}` über ihren Rückkanal, die übrigen Follower laufen weiter (CRASH-07). Einzel-Schreibpfade außerhalb des Commit-Pfads (`ops/write.rs` Zeilen 30, 88, 153, 189, 224 \[G\]) rufen denselben Check mit einem Load je Aufruf auf. Die heutigen drei Read-Locks je Schlüssel entfallen.
3. **Erwerb:** auf einem dedizierten Purge-Thread über `Handle::block_on(commit_mutex.lock())` (kein `blocking_lock`, das im Async-Kontext panikt). Unter dem Mutex: Präfix eintragen (Referenzzähler erhöhen, verschachtelte Präfixe gelten als Vereinigung), `active` auf wahr setzen (Release), `fence_seq` gleich zuletzt vergebene Sequenznummer lesen, Mutex freigeben.
4. **Drain-Marke:** Der Purge wartet, bis die Sichtbarkeits-Sequenz mindestens `fence_seq` erreicht hat. Dann sind alle vor dem Erwerb zugelassenen Batches angewendet. Danach kann kein Schreibvorgang im Scope mehr entstehen. `as_of_tx` des Rewrites ist `fence_seq`. Zeitgrenze `fence_drain_timeout` (Standard 30 Sekunden), danach Fence freigeben und `Pending{Fenced}`.
5. **Freigabe:** unter dem Mutex, Referenzzähler senken, bei null Präfix entfernen, bei leerer Liste `active` auf falsch.
6. **Modus Off:** Der Gate existiert nicht. Der Stall anderer Scopes ist auf die Mutex-Haltephase des Erwerbs begrenzt (DE-13), nie während des Rewrites.
7. **Nachweis ohne Zähler:** `write_seq` entfällt. Der Scan prüft per MVCC, dass die jüngste sichtbare Version jedes Footprint-Schlüssels der eigene Tombstone ist und keine Version mit `tx` größer `erased_at_tx` existiert.
8. **Pflicht vor Freigabe:** Loom-Modell mit mindestens drei Threads und drei Szenarien: (a) Erwerb vor dem Leader-Erwerb, (b) Erwerb zwischen erster und zweiter Haltephase, (c) Freigabe während eines laufenden Batches. Dazu Prüfung der Sequenzvergabe. Benchmark höchstens 1 Prozent Abweichung. **Scheitert eine Bedingung, gilt Option A** (Drain mit In-flight-Zähler).

### 6.4 Resume

Resume läuft **nach** dem Aufbau von Storage, Schlüsselverwaltung und Ledger, nie in der Recovery. Der Bestand ist nicht verdrahtet (`resume_pending` ohne Produktionsaufrufer \[H\]).

1. Alle `purge/*.intent` lesen und gegen den Ledger prüfen. Ein Intent ohne `Intent`-Record im Ledger ist `ScopeNotInLedger`, und **das Öffnen schlägt fehl** (fail-closed).
2. Ledger-`Intent` ohne Tombstone (CRASH-01): Per MVCC prüfen, ob die jüngste sichtbare Version jedes Footprint-Schlüssels der Tombstone ist. Fehlt er, wird er aus dem Ledger neu geschrieben (bei Collection und Mandant der Katalog-Tombstone).
3. `LayerEraser` für alle Layer erneut ausführen (Idempotenz, CRASH-09).
4. **Fence erwerben, bevor `open` Ok meldet.** Der `FenceGuard` wird in den Hintergrund-Task verschoben.
5. Purge als Hintergrund-Task starten, `open` blockiert nicht auf den Rewrite. Ein offener Snapshot ergibt `Pending{OpenSnapshot}` mit Backoff.
6. Die Intent-Datei wird erst nach dem Ledger-Eintrag `Purged` entfernt. Zweimaliges Resume führt zum selben Endzustand (CRASH-14).
7. In Off ist Resume ein No-op ohne Dateizugriff.

### 6.5 Anti-Resurrection-Ledger (Format `CTRALEDG` v1)

| Element | Festlegung |
| --- | --- |
| Segment-Kopf | Magic `CTRALEDG` (8 Byte), Version `u32`, Segmentnummer `u64`, BLAKE3-Hash des Vorgängersegments (32 Byte) |
| Record | Länge `u32` (Torn-Write-Erkennung), Typ `u8`, `tx u64`, Zeit `u64` (Unix-Millisekunden aus `Clock`), `purge_id [u8;16]`, `scope_digest [u8;32]`, Familien-Digests, Nutzlast, `mac [u8;32]`, `crc32c u32` |
| Typen | `0x01 Intent` (mit Klartextpräfixen, nötig für Resume), `0x02 Purged` (nur Digests), `0x03 EpochMarker` (signierter Downgrade-Eintrag), `0x04 AdoptMarker` |
| Bezeichner | `scope_digest` und Familien-Digests sind `HMAC-BLAKE3(ledger_key, kanonische Bytes)`. Der RestoreGuard zerlegt Schlüssel über den Key-Codec und schlägt Digests nach, er braucht keine Klartextpräfixe |
| Verkettung | `mac = HMAC-BLAKE3(ledger_key, vorheriger_mac, Record ohne mac und crc)`. `ledger_key` ist ein HKDF-Unterschlüssel mit Label `ledger/v1` |
| Dauerhaftigkeit | fsync je Append. Gleichzeitige Löschungen teilen sich einen fsync (Gruppen-Append) |
| Torn Write am Ende | wird beim Öffnen am letzten gültigen Record abgeschnitten (CRASH-12). Beschädigung in der Mitte ist in Assured ein Öffnungsfehler `LedgerCorrupt` |
| Rollback-Erkennung | Das Manifest trägt `ledger_head_seq` und `ledger_head_mac`. Ein zurückgespielter Ledger stimmt nicht überein, das Öffnen scheitert (CRASH-17) |
| Rotation und Verdichtung | Rotation bei 64 MiB. Ein Hintergrundlauf ersetzt `Intent`-Records abgeschlossener Purges durch ihre `Purged`-Digests (atomar, danach Scrub des alten Segments). Klartextpräfixe verschwinden so aus dem Ledger |

Der Ledger ist für Assured Pflicht, im Modus Off gibt es keinen. `ledger_append` hat am HEAD keinen Produktionsaufrufer \[H\].

### 6.6 RestoreGuard und WAL-Replay

Je Schlüssel: (0) `cp_tx >= max_erased_tx` heißt Admit ohne weitere Prüfung. (1) Zerlegung über den Key-Codec. (2) Bloom-Filter nur als Vorauswahl. (3) Bei Treffer exakte Prüfung im Ledger-Index (im Speicher, beim Öffnen aus den `Intent`- und `Purged`-Records aufgebaut, `ArcSwap`). (4) Prüfung auf widerrufenen Schlüssel. Der Restore meldet `RestoreReport{admitted, rejected_ledger, rejected_revoked}`, abgewiesene Schlüssel erscheinen nie still als Erfolg. **Der WAL-Replay nutzt denselben Guard**, Frames mit `tx <= erased_at_tx` im Scope werden abgewiesen. Abgeleitete Indizes werden nach einem Restore aus dem gefilterten LSM neu aufgebaut. In Assured ist ein fehlender Guard `GuardRequired`. `restore_checkpoint` ruft heute nur `rollback_to_tx` und umgeht den Guard (laut v3 \[U\], K1.12).

### 6.7 Dispose, Scrub und Kopien

`dispose(files)` überschreibt je Datei (Nullen, ein Durchlauf, 64-KiB-Blöcke), fsync, `unlink`, Verzeichnis-fsync. **Verweigerungsfälle:** Hardlink-Zähler größer 1, Copy-on-Write- oder reflink-Dateisystem, Immutable-Flag, Überschreiben schlägt fehl. Dann `ScrubRefused`, Zustand `Pending{ScrubRefused}`, die Datei erscheint als Ausschluss im Beweis, **es gibt keinen Rückfall auf normales Löschen**. Auf SSDs ist Überschreiben Best-Effort (Ausschluss 1), belastbar ist nur Crypto-Erase.

Eine `CopyRegistry` führt jede bekannte Kopie mit Pfad, Art (Checkpoint, Hardlink, Backup), Transaktionsbereich und Status (`Contains`, `Purged`, `NotApplicable`, `Unknown`). In Assured und Attested gilt `checkpoint_policy`: `Retain` (Standard, Status `Contains`, geschützt durch den RestoreGuard), `Rewrite` (derselbe Streaming-Rewrite) oder `Expire` (Checkpoints mit dem Scope werden verworfen). Externe Backups außerhalb der Registry bleiben Ausschluss 3.

### 6.8 Crash-Matrix

| Nr. | Absturz | Erwartung nach Recovery | Test |
| --- | --- | --- | --- |
| CRASH-01 | nach Ledger, vor Tombstone | Re-Tombstone aus dem Ledger | `chaos_ledger_orphan` |
| CRASH-02 | im SSTable-Rewrite | altes Manifest aktiv, `purge_tmp` entfernt | `chaos_sst_rewrite_crash` |
| CRASH-03 | nach SSTable, vor Manifest | Aufräumen, Intent-Resume | `chaos_pre_manifest_crash` |
| CRASH-04 | nach Manifest, vor Scrub | Resume führt `dispose` aus | `chaos_post_manifest_scrub` |
| CRASH-05 | Platte voll im Purge | `InsufficientDiskSpace`, Pending | `chaos_disk_full` |
| CRASH-06 | Lesefehler (Bit-Rot) | `ReadError`, fail-closed, kein Zeuge | `chaos_bitrot_fail_closed` |
| CRASH-07 | Schreibkonflikt im Rewrite | `WriteFenced` nur im Scope | `chaos_fence_contention` |
| CRASH-08 | Restore aus altem Backup | gelöschte Schlüssel abgewiesen | `chaos_anti_resurrection` |
| CRASH-09 | zwischen Tombstone und RAM-Bereinigung | Resume bereinigt Indizes | `chaos_index_cleanup` |
| CRASH-10 | MANIFEST-Rollover und Checkpoint-Restore | konsistenter Stand, Guard greift | `chaos_manifest_rollover` |
| CRASH-11 | während Compaction | Lease verhindert Überschneidung | `chaos_compaction_lease` |
| CRASH-12 | Ledger: Torn Write am Ende | Abschneiden am letzten gültigen Record | `chaos_ledger_torn_tail` |
| CRASH-13 | zwischen Widerruf und Vernichtung | `Revoked` bleibt, Vernichtung nach der Frist | `chaos_key_revoke_destroy` |
| CRASH-14 | Resume zweimal hintereinander | gleicher Endzustand | `chaos_resume_idempotent` |
| CRASH-15 | WAL-Barriere unterbrochen | Barriere wiederholt, kein Segment verloren | `chaos_wal_barrier` |
| CRASH-16 | offener Snapshot blockiert Purge | `Pending{OpenSnapshot}`, nach Ablauf `SnapshotPruned` | `chaos_snapshot_starvation` |
| CRASH-17 | Ledger-Rollback | Öffnen scheitert | `chaos_ledger_rollback` |

## 7. Löschnachweis (Stufe Attested)

Ein Nachweis ist ein **Audit mit definierten Zeugen und Toleranzen**, kein mathematischer Beweis. Er ist so stark wie seine schwächste Annahme. Attested ist ein Plugin und startet erst nach dem Austritt aus K3. Für v0.1.0 ist es nicht Teil des Releases (DE-01).

### 7.1 Layer-Modell und stabile Layer-IDs

Scans (Abwesenheit, gemessen) und Bescheinigungen (Ereignis, protokolliert) sind getrennt, Ausschlüsse sind ausdrücklich nicht belegt. Layer-IDs sind stabil und stehen im Beweis. Die IDs 1 bis 7 sind die Berichtssicht, ab 20 stehen Unterlayer und Zusatzscans.

| ID | Layer | Art | Persistente Artefakte | Löscht (`LayerEraser`) | Prüft |
| --- | --- | --- | --- | --- | --- |
| 1 | Tombstone und Memtable | Scan | Memtable (RAM) | store | `LsmMemtableScan` |
| 2 | Physisches Überschreiben | Bescheinigung | Dispose-Protokoll | store (`dispose`) | Digest des Dispose-Reports |
| 3 | SSTables aller Ebenen | Scan | `*.sst`, `.purge_tmp` | store (Purge v2) | Rohsweep über alle aktiven SSTables |
| 4 | WAL-Segmente | Scan | `wal/*.log` | store (WAL-Barriere) | Segmentscan |
| 5 | Checkpoints und Kopien | Bescheinigung | Checkpoints, Hardlinks | checkpoint (CopyRegistry) | Status je Kopie |
| 6 | Kryptografisches Shredding | Bescheinigung | Schlüsselspeicher, RevocationLog | crypto-kv | Vernichtungsprotokoll |
| 7 | Audit-Kettenglied | Bescheinigung | Audit-Kette | erasure-evidence | Kettenhash |
| 20 | Text-Index | Scan | Text-Segmente je Collection | text | `TextIndexScan` |
| 21 | Relationen und Graph | Scan | Relationen, CSR | graph, db | `RelationsScan` |
| 26 | HNSW | Scan | RAM-Arena, Snapshot-Dateien | vector (DE-11) | `HnswVectorScan` |
| 30 | Ledger | Scan | `erasure_ledger/` | erasure | `LedgerScan` (nur Digests nach Purge) |
| 35 | KV-Cache | Scan | Tier-2-Segmente | kvcache (EP4) | `KvCacheScan` |
| 40 | Egress-Audit | Scan | Audit-Einträge | privacy (EP6) | `EgressAuditScan` |

Die Policy verlangt eine **Pflicht-Layer-Menge**. Für EP1 Attested ist sie {1, 3, 4, 20, 26, 30} plus die Bescheinigungen {2, 5, 7} und {6}, falls Verschlüsselung aktiv ist. Layer 21 kommt hinzu, sobald Relationen in EP1 aktiv sind. Ein Test stellt sicher, dass die Pflichtmenge eine Teilmenge der real erzeugbaren Layer ist. Ein fehlender Zeuge ist ein Fehler, nie `Ok(None)`. Layer 40 (Egress-Audit) prüft, dass kein Audit-Eintrag gelöschte Daten im Klartext referenziert.

Die Berichtssicht gruppiert nach „Contextra-Schichtensicht“: L1 (ID 1), L2 (2), L3 (3, 20, 21, 26), L4 (4), L5 (5, 30), L6 (6), L7 (7). Das ist eine Gruppierung, kein Drahtformat. Mehrere Quellen sprechen von einem „7-Schichtenmodell nach BSI TR-02102“. TR-02102 beschreibt Verfahren und Schlüssellängen, nicht ein Löschverfahren \[U\]. Der Bericht verwendet den Namen „Contextra-Schichtensicht“ und verweist auf TR-02102 höchstens im Algorithmen-Inventar.

### 7.2 Beweisformat v4 (eine verbindliche Fassung)

```
Kopf        32 Byte ASCII "contextra/deletion-proof/v4" mit NUL aufgefüllt
Record      tag u16 LE | länge u32 LE | wert
Reihenfolge Tags streng aufsteigend; 0x0008 und 0x0010 wiederholt, dann nach Schlüssel aufsteigend
Signiert    Kopf plus alle Records mit Tag <= 0x00FE   (Ed25519, PureEdDSA, verify_strict)
Unsigniert  Records mit Tag >= 0x0100 nach der Signatur (Zeitanker, Transparenz-Nachweis)
```

| Tag | Inhalt | Länge |
| --- | --- | --- |
| 0x0001 | Scope-Digest (BLAKE3) | 32 |
| 0x0002 | `erased_at_tx` | 8 (u64 LE) |
| 0x0003 | Ausstellungszeit, Unix-Millisekunden aus `Clock` | 8 |
| 0x0004 | Modus (1 Assured, 2 Attested) und Policy-ID | 4 |
| 0x0005 | Ausschlussmaske (Bits 0 bis 4, **alle gesetzt**) und Textversions-ID | 4 |
| 0x0006 | Witness-Merkle-Wurzel (BLAKE3) | 32 |
| 0x0008 | Kopie: Art `u8`, Status `u8`, Digest | 34 (wiederholt) |
| 0x0010 | Layer: `layer_id u32`, Art `u8` (1 Scan, 2 Bescheinigung), Status `u8` (1 sauber, 2 ausstehend, 3 nicht anwendbar), Digest | 38 (wiederholt, nach `layer_id` aufsteigend, eindeutig) |
| 0x0020, 0x0021 | Hash des Vorgängerbeweises, Kettenfolge | 32, 8 |
| 0x00FE | Ed25519-Public-Key | 32 |
| 0x00FF | Ed25519-Signatur | 64 |

**Verifier lehnt ab:** unbekannter oder doppelter Tag, falsche Reihenfolge, falsche feste Länge, Gesamtlänge über 1 MiB, mehr als 64 Layer, unvollständige Ausschlussmaske, Layer mit Status „nicht sauber“, fehlender Pflichtlayer der Policy, nicht kanonische oder schwache Signatur (`verify_strict`). Ein Beweis mit Layer-Status „ausstehend“ wird **nicht ausgestellt**. Die Policy bestimmt, welche Layer „nicht anwendbar“ sein dürfen (etwa Layer 6 ohne Verschlüsselung), und jeder solche Fall steht zusätzlich als Text im Beweis. Der Beweis ist ausschließlich TLV, CBOR ist nur die Hülle des Audit-Exports in EP2. Getrennte Domain-Tags je Formatversion. v3 wird nie erzeugt und nur lesend verstanden. `bincode` gibt es nur zum Lesen von v2 und v3, nie im Signatur- oder Commitment-Pfad (`audit_chain.rs:85` ist umzustellen \[G\]). `verify_strict` gilt in allen Prüfpfaden, kein `unwrap_or_default` im Signaturpfad. Der Signierschlüssel stammt aus CSPRNG oder KMS, je Datenbank, mit Rotation, Dateirechten und `zeroize`. Es gibt keinen Nutzerschlüssel in `~/.contextra` für alle Datenbanken.

### 7.3 Unfälschbarkeit und Abnahme

- **Pull-Modell (DE-22):** Das Plugin ruft die LayerScans selbst auf und baut die Zeugen. Konstruktoren sind `pub(crate)` im Plugin. Wer ein eigenes LayerScan registriert, kann lügen. Das fangen Policy und Mutationstest, keine Typprüfung.
- **Mutationstest:** Scanner gleich No-op muss fehlschlagen.
- **Tampering-Matrix:** (1) Jedes Feld aus 7.2 einzeln verändert. (2) Bytegenau: jedes Byte eines Korpus gültiger Beweise umgekehrt, jede Kürzung und Verlängerung. Der Verifier lehnt ab, außer in den unsignierten Anhängen, die separat geprüft werden. (3) Strukturell: eingefügtes Fremd-TLV, doppelte oder vertauschte Tags, Layer-Anzahl, Überlängen. (4) Kryptografisch: Public Key kleiner Ordnung, nicht kanonisches S, Signatur eines anderen Schlüssels, gültige Signatur über anderen Domain-Tag. Dazu Ed25519-Grenzfall-Vektoren (RFC 8032, Wycheproof).
- **Orakel:** unabhängiger Rohsweep über alle Dateien nach Marker-Bytes und je ein Canary je Layer.

### 7.4 Verifier, Zeitanker und Transparenzlog

**`contextra-proof-verify`:** `no_std` mit `alloc`, kein `unwrap`, `expect` oder Indexzugriff (nur `get` und `try_into` mit Fehlerwert), Abhängigkeiten `ed25519-dalek`, `blake3`, `sha2`. Ohne Engine, Store, Netz und Lizenz-Gate. Eigene Spezifikation, Fuzz-Ziel, Testvektoren. Erzeuger und Prüfer teilen keinen Code. Die zweite Implementierung (DE-25) und die Rust-Fassung müssen auf einem Korpus aus gültigen und mutierten Beweisen übereinstimmen.

**Verifikationsprotokoll:**

1. Strikter Parser gemäß 7.2, danach `verify_strict` über den signierten Bereich.
2. Kettenprüfung: Hash des Vorgängers gleich Tag 0x0020, Folge streng aufsteigend.
3. Layer-Vollständigkeit gegen die Policy-ID (die Policy-Tabelle ist Teil der Verifier-Spezifikation).
4. Layer 6: Schlüssel-ID verweist auf einen vernichteten Schlüssel, Vernichtungszeit liegt vor der Ausstellungszeit.
5. Zeitliche Konsistenz: Ausstellungszeit nicht in der Zukunft relativ zum Anker.
6. Anker und Inklusionsnachweis (falls vorhanden) gegen die öffentlichen Schlüssel der TSA und des Logs.
7. Ausgabe zusammen mit den fünf Ausschlüssen und dem Wortlaut „beobachtet, nicht garantiert“.

**Zeitanker (RFC 3161, DE-26):** Das Message-Imprint ist SHA-256 über die vollständigen signierten Beweisbytes (BLAKE3 ist in RFC 3161 nicht standardisiert). Das Token steht im unsignierten Anhang (Tag 0x0100). Ein RFC-3161-Zeitstempel im Hash eines Layers des signierten Beweises wäre zirkulär und ist verboten. **Transparenzlog (RFC 9162):** Beweis-Hashes sind Blätter eines lokalen Merkle-Logs, der Inklusionsnachweis steht im Anhang (Tag 0x0101). Ein lokales Log allein verhindert das Zurückspulen nicht, deshalb wird der signierte Baumkopf periodisch per RFC 3161 verankert. Beides gehört zu Attested und EP2. Ohne Anker ist die Ausstellungszeit eine Behauptung des Ausstellers und der Beweis sagt das aus.

### 7.5 Pflicht-Ausschlüsse

Im signierten Beweis und im Prüfbericht stehen immer fünf Ausschlüsse: (1) SSD-Remanenz und Wear-Leveling, (2) Prozessspeicher vor dem Überschreiben, (3) externe Backups vor dem Löschzeitpunkt, (4) Auftragsverarbeiter und Netzwerkspeicher, (5) Swap und Core-Dumps. Die Texte unterscheiden strikt **beobachtet** von **garantiert**. Crypto-Erase ist auf SSDs der einzige robuste Weg. `harden_runtime` und `mlock` melden nur, was sie belegen. Kopien erscheinen als Status je Kopie, nicht als Zähler. Die HNSW-Zeroisierung nutzt `zeroize` samt Kopien und Mmap-Seiten (ein `fill` mit null kann der Compiler entfernen).

### 7.6 Recht

Ob Crypto-Erase als Löschung nach Art. 17 DSGVO gilt, ist offen. Eine Ed25519-Signatur ist keine qualifizierte Signatur, § 371a ZPO greift nicht automatisch. Beides steht als Hinweis in Rechtsmatrix und Betriebsvorgabe, nicht als Zusicherung im Beweis. Das ist Recherche, keine Rechtsberatung. Quellen (EDPB, DIN, EuGH) sind nicht geprüft. Aussagen in README und Marketing sind zu belegen oder zu streichen.

## 8. Querschnittsspezifikation

### 8.1 Funktionsweise von EP1 (Soll)

**Schreibpfad (`remember`).** Der Embedder läuft vor der Transaktion außerhalb aller Sperren. Eine MVCC-Transaktion schreibt Dokument, Metadaten und Indexeinträge in einem Commit (WAL-Frame mit HMAC-Kette, Group-Commit). RAM-Indizes (HNSW, BM25) werden nach dem Commit aktualisiert. In Assured prüft der Leader je Batch das Fence-Gate (6.3), sonst nichts.

**Lesepfad (`search`).** (1) Die Anfrage erhält **einen** `ReadSnapshot` (PV3). (2) Vektorsuche (HNSW, SIMD-Kernel nach `CpuFeatures::detect`) und BM25 (invertierter Index mit Roaring-Bitmaps) liefern Kandidaten. (3) Der Index liefert nur Kandidaten, die MVCC-Sichtbarkeit unter dem Snapshot entscheidet. Ein logisch gelöschtes Dokument erscheint nie, auch wenn der Index noch nicht bereinigt ist. (4) Fusion über RRF (Summe von 1 durch 60 plus Rang), danach Kalibrierung (Platt, isotonisch, adaptiv). (5) Deterministische Ordnung: Score absteigend mit `total_cmp`, dann Dokument-ID aufsteigend. Der Mandant kommt aus der Sitzung, jede Storage-Operation läuft über `TenantScopedStorage`.

**Persistenz.** WAL (v3, Frame-Header v4) plus SSTables mit Bloom-Filtern, MANIFEST mit Hochwassermarke, Checkpoints (Manifest mit BLAKE3). Beim Öffnen: Manifest, Hochwassermarke, WAL-Replay (mit RestoreGuard in Assured), Ledger, Resume.

### 8.2 MVCC, Snapshots, Serialisierbarkeit

- Kein `u64::MAX` als Lesezeitpunkt (`collection/crud/read.rs`, `links.rs`, weitere Stellen). Alle Pfade lesen unter einem echten Snapshot.
- `ReadSnapshot` (max\_seq als opakes Token) wird durch den gesamten Lesepfad über store, vector, text und graph gereicht. Gate: check-snapshot-consistency.
- `SnapshotFloor` hat genau eine Quelle (`floor.rs`). `register` und `register_at` in mvcc entfallen, alle Aufrufer nutzen `acquire`. `recover_pending_intents` wird in `repair_on_open` verdrahtet.
- Der `SequenceLogSsiValidator` bleibt Standard (rw-Antiabhängigkeiten, fail-closed mit `SnapshotPruned`). Read-Set-Tracking und atomares `put_if_absent` gehören zu K4.
- **SSN nicht vor K4 (DE-23).** Voraussetzungen: ein Property-Test zeigt mehr als 5 Prozent Fehlabbrüche des SSI-Validators bei lesestarken Lasten, und das Modell ist im Single-Thread-Replay deterministisch nachgewiesen. SSN läuft nur seriell unter einer Sperre mit monotoner `tx_seq`.
- **Snapshot-Lebensdauer (DE-20):** In Assured ist `max_snapshot_age` Pflicht (Standard 15 Minuten, zulässig 1 Minute bis 24 Stunden).

### 8.3 Zeitachsen (DE-24)

Systemzeit (`TxId`) und Geschäftszeit (Unix) werden nie vermischt. `apply_temporal_validity_filter_at` castet heute einen Unix-Zeitstempel in `TxId` (ENG-FILTER-001 \[U\]). Es gelten getrennte Parameter `as_of_tx: TxId` und `as_of_business: Option<u64>`. `TxId` und `TenantId` erhalten geprüfte Konstruktoren und private Felder, die Prüfung gilt auch im Release-Build. Am HEAD ist `CollectionId::try_new` mit Nullprüfung vorhanden \[G\], `CollectionId(pub u64)` bleibt öffentlich konstruierbar und wird privat.

### 8.4 Determinismus und Zero-Panic

Uhr, Zufall und IDs nur über `Clock`, `Rng`, `IdGenerator`, auch in checkpoint, adapt und der Compaction-Ratenbegrenzung (heute direkte `Instant::now`-Aufrufe \[U\]). Gleitkomma: `total_cmp`, Kahan-Summen, Festkomma (i128) wo Teleskopschranken gelten. Zero-Panic-Muster: `get` statt Index, `checked_mul`, Prüfung von Nenner und `is_finite` vor Division und Float-Cast. **Eine** Unwrap-Baseline: `governance/ratchet.toml` (`unwrap_exact = 502` \[G\]), `.github/unwrap_baseline.txt` wird generiert, die Ratsche fällt nur.

### 8.5 Persistente Formate

| Format | Versionierung | Pflicht |
| --- | --- | --- |
| WAL | v3, Frame-Header v4. Legacy-WAL wird vor v0.1.0 entfernt (DE-17) | Upgrade-Test, partieller AEAD-Frame am Ende wird beim Replay abgeschnitten (Test) |
| SSTable | Format-Version | Upgrade-Test |
| MANIFEST | mit Hochwassermarke, `ledger_head_seq`, `ledger_head_mac`, `backend_id` | Upgrade-on-open, Downgrade-Test |
| Schlüssellayout | Formatversion für die Migration des Namenslayouts `__col:{name}:\x00` auf `t:{tenant}:{collection_id}:` (DE-06) | Migrationstest mit Wiederanlage unter gleichem Namen |
| Ledger | `CTRALEDG` v1 (6.5) | CRASH-12, CRASH-17 |
| Beweis | v4 TLV (7.2), v3 nur lesend | Tampering-Matrix |
| FlatBuffers-Schema | check-fbs-drift als Pflicht-Gate. Der Exit-1-Lauf aus den Audit-Logs ist zu klären | Gate grün |
| Checkpoint-Manifest | BLAKE3 | Upgrade-Test |

### 8.6 Sperrenhierarchie und Nebenläufigkeit

Store-Ordnung laut `ops/write.rs:5` \[G\]: `commit_mutex` (tokio) vor `pending_commit_queue` vor `truncate_lock` vor `state`. Engine-Ordnung: collections, kv\_locks, embedder, erasure\_ledger. Der Fence-Check ist lock-frei (Atomic-Load). Der Fence-Erwerb nimmt nur den `commit_mutex` auf dem Purge-Thread (6.3). Keine Sperre über `await`, außer dem `commit_mutex` des Store, dessen Haltedauer in `write.rs` dokumentiert ist. Loom prüft Fence, Group-Commit und Sperrenordnung. Der JSON-RPC-Dispatcher des MCP-Servers wird rein asynchron (`block_in_place` mit `block_on` entfällt, FINDING-MCP-BLOCKING).

### 8.7 Lebenszyklus der Hintergrundprozesse

Vier Reaper (Ablauf, Orphan, Thermostat, KV-Cache-Eviction) werden im `open`-Pfad bisher nie gestartet \[U\]. Ein `EngineLifecycle`-Manager startet sie. **Start:** Manifest und Hochwassermarke, WAL-Recovery, Ledger, Schlüsselverwaltung, Resume (Fences aktiv), dann Reaper und Purge-Worker. **Ende:** Shutdown-Signal, der Purge-Worker hält an der SSTable-Grenze an (der Intent bleibt), Reaper stoppen, Join mit Zeitgrenze, Fences gehen mit dem Prozess unter und werden beim nächsten Öffnen aus den Intents wiederhergestellt. Ein Prozess, der nie startet, ist im Sinne von PE5 nicht verdrahtet.

### 8.8 Builder, Features und Profile

Sieben Builder-Optionen (`with_privacy`, `with_adapt`, `with_kms`, `with_kv_shredding`, `with_checkpoint`, `with_avv_generator`, `with_audit_export`) speichern laut Audit Werte, die nie gelesen werden \[D\]. `build()` prüft in dieser Reihenfolge und scheitert beim ersten Verstoß mit einem benannten Fehler:

1. Durability-Matrix (5.3) mit `DurabilityConflict`.
2. Modus, Feature und Hooks: Assured ohne Feature oder Hooks mit `FeatureRequired` oder `NotConfigured`. Attested ohne Evidence-Plugin oder ohne `FeatureRing::Sovereign` mit `FeatureRequired` oder `LicenseRequired`. **Kein stilles Herabsetzen** (das Muster in `lifecycle.rs` entfällt).
3. Store öffnen, Hochwassermarke prüfen mit `PolicyViolation`.
4. Ports verdrahten, `verify_wiring` gegen `governance/security_ports.toml` mit `IncompleteWiring{flag, missing}`. Der Selbsttest läuft im Startpfad, nicht nur in Tests.
5. Resume (6.4), danach Lebenszyklus (8.7).

Hohle `*Wiring`-Typen werden gelöscht, der Rest wird ein einheitliches Modul-Registrierungsmuster. Gate check-builder-wiring.

Feature-Graph der Fassade:

```toml
[features]
default = ["fast"]                       # hüllenneutral
fast = ["contextra-simd/default"]
erasure = ["dep:contextra-erasure", "contextra-store/erasure", "contextra-engine/erasure"]
erasure-evidence = ["erasure", "dep:contextra-erasure-evidence"]
encryption-at-rest = ["dep:contextra-crypto-kv", "contextra-store/encryption-at-rest"]
license = ["dep:contextra-license"]       # nur Fassade, nie Kern
# ab EP3, nicht Teil von EP1: graph, adapt, kvcache, cognition, router, agent,
# candle, ollama, onnx, privacy, sandbox, guard
sovereign  = ["privacy", "erasure-evidence", "encryption-at-rest"]   # erst mit EP2 und EP6
compliance = ["sovereign", "audit-export", "avv-generator"]
```

CI-Matrix je Paket, `--locked`, nie `--workspace`: `--no-default-features`, `default`, `erasure`, `erasure,encryption-at-rest`, `erasure,erasure-evidence`.

| Profil | Modus | Verschlüsselung | Durability | Verfügbar |
| --- | --- | --- | --- | --- |
| **Balanced** (Standard) | Off | aus | Full | v0.1.0 |
| **Assured** | Assured | optional, empfohlen | Full | v0.1.0 |
| Attested | Attested | an | Full mit HMAC | EP2 |
| Compliance | Attested plus Export | an | Full mit HMAC | EP2 und EP6 |

### 8.9 MCP-Werkzeuge und Sicherheitsgrenze

| Werkzeug | Klasse | Bedingung |
| --- | --- | --- |
| `contextra_search`, `contextra_remember` | ReadOnly, WriteLocal | EP1 |
| `contextra_forget` | WriteLocal | EP1, reicht Request-Optionen durch (am HEAD `DocEraseOptions::default()` \[G\]) |
| `contextra_erasure_status` | ReadOnly | Feature `erasure` |
| `contextra_explain` (nutzt `find_path`) | ReadOnly | Feature `graph`, EP3 |
| `contextra_relate`, `contextra_consolidate` | WriteLocal | EP3 und EP5, aus dem EP1-Build entfernt |
| `contextra_cloud_query` | EgressExternal | nur Feature `guard`, EP6 (am HEAD ohne Guard registriert) |

Jedes Werkzeug trägt eine Klassifikation, ein Werkzeug ohne Klassifikation lässt den Registry-Test scheitern (INV-MCP-CLASSIFY-1). `find_k_paths` bleibt `pub(crate)` bis EP3. Es gibt nur einen Weg für `remember` und `recall`: MCP ruft die Fassade. Die Mandanten-ID kommt aus der Sitzung.

### 8.10 Unsafe, Mandanten, Sandbox-Kante

`unsafe` nur in simd, sys, wire. Vor jedem AVX2- oder AVX-512-Kernel ist `CpuFeatures::detect` Pflicht. Alle anderen Crates haben `forbid(unsafe_code)`. Python fällt nicht auf `TenantId::SYSTEM` zurück, der Mandant kommt immer aus dem Kontext. Die Kanten store und engine auf sandbox entfallen über `MergeOperatorFactory`. WASM-Merge-Operatoren brauchen CRDT-Property-Tests, ein Freischaltungs-Gate und Modul-Caching, bevor sie aktiv werden (EP6).

## 9. Stabilität, Gates, Tests und Budgets

### 9.1 Definition: das Fundament ist stabil, wenn S1 bis S12 gelten

| Nr. | Kriterium | Beleg |
| --- | --- | --- |
| S1 | Build und Test mit gepinnter Toolchain 1.89.0, `--locked`: Exit 0 | Log im Repo mit Befehl, Commit, Exit-Code |
| S2 | Gates können rot werden (Negativtests für check-dag und Ring-Gate) | Negativtest-Suite in xtask |
| S3 | 0 unverdrahtete öffentliche Funktionen in EP1, 0 hohle Wiring-Symbole | Gate orphan-symbols |
| S4 | Ring-Gate strict grün, Hülle von mcp mit Default-Features höchstens 18 | check-ring-layering, check-product-hull |
| S5 | Purge: Differenztest gegen Referenzmodell und Rohsweep grün | `purge_correctness` |
| S6 | Crash-Matrix CRASH-01 bis 17 grün | je Zeile ein benannter Test |
| S7 | Rohträger-Test grün (kein Marker in WAL, SSTables, Checkpoints, HNSW, KV-Segmenten) | aus der Spezifikation, nicht vom Implementierer |
| S8 | Fence: Loom-Modell grün, Hot-Path-Abweichung höchstens 1 Prozent | Criterion p50 und p99 |
| S9 | Snapshot-Isolation überall, kein `u64::MAX` als Lesezeitpunkt | check-snapshot-consistency |
| S10 | Zero-Panic: eine Unwrap-Baseline, nur fallend | ratchet |
| S11 | Alle persistenten Formate versioniert, Upgrade- und Downgrade-Test | je Format ein Test |
| S12 | Dokumentation gleich Code, Anspruchsstufe mindestens C2 für Löschung | doc-truth, claims-in-pr |

Erst wenn S1 bis S12 erfüllt sind, ist v0.1.0 freigabefähig. Ebenen R bis G beginnen vorher nicht.

### 9.2 Gates

| Gate | Zweck | Stand |
| --- | --- | --- |
| check-dag | Zyklen als Verstoß (Negativtest) | kaputt, K0.1 |
| check-ring-layering strict | Schichtenregel aus `capabilities.toml` | K0.2 bis K0.4 |
| check-ring-capabilities-consistency, orphan-symbols, check-fbs-drift, unwired-ports | in `gates.toml` aufnehmen, JSON nur auf stdout | K0.12 |
| deletion-guarantee-gate | Rohsweep, Marker, Produktionsaufrufer plus Fassadentest, Doku gleich Code | K0.16 |
| check-product-hull | Hülle gegen Manifest | K2 |
| check-builder-wiring, check-feature-isolation, check-snapshot-consistency | neu | K1 bis K3 |
| check-loom-mocks, doctrine-scan | Loom ohne unzulässige Mocks, Zero-Panic und Unsafe-Doktrin | CI |
| scope-guard, task-card, diff-budget, protected-paths, ratchet, gate-weakening, test-integrity, check-vetoes, doc-truth, claims-in-pr | bestehend, erweitern statt neu bauen | vorhanden |

### 9.3 Testpyramide

Unit, **Integration über den echten Pfad** (Fassade, MCP; je Verdrahtung ein Test, rot ohne die Änderung), **Tenant-Matrix** (jeder Löschtest über `LsmStorage` und `TenantScopedStorage`), **Differenztest**, **Rohträger-Test**, Crash-Matrix, Fault-Injection (`fault_vfs`), Loom, Mutationstest je Garantiegruppe (`mutants-guarantees.toml`), Fuzz (Smoke in CI, Langlauf nächtlich), Abschaltprobe je Modul. Der Mangel des Projekts ist nicht die Zahl der Tests, sondern die Unabhängigkeit der Abnahme: **Der implementierende Agent schreibt den Abnahmetest nicht.**

**Differenztest (S5):** Ein Proptest mit festem Seed erzeugt Folgen aus `put`, `delete`, Snapshot-Lesen, `purge(scope)` und Fehlerinjektion und vergleicht den LSM mit einem `BTreeMap<Key, Vec<(Seq, Value)>>`-Modell. Invarianten:

- **I1:** Für jeden Schlüssel außerhalb des Scopes liefert jeder Snapshot mit `seq >= as_of_tx` denselben Wert wie das Modell.
- **I2:** Die Versionsreihenfolge bleibt erhalten (kein älterer Wert verdeckt einen neueren). Der Test erzwingt überlappende Bereiche in Ebene 0.
- **I3:** Nach dem Purge sind im Scope alle Versionen mit `seq <= as_of_tx` im Rohsweep nicht auffindbar.
- **I4:** Ein injizierter Lesefehler an jedem Offset führt zu `Err` und byteidentischen Dateien.
- **I5:** Zweifacher Purge desselben Scopes führt zum selben Zustand.

**Rohträger-Test (S7):** Canary je Layer und je Dokument (Nutzlast, Dokument-ID, Textterm, Einbettungsmuster mit eindeutigem Bitmuster, Relationenattribut). Träger sind alle Dateien rekursiv unter dem Datenverzeichnis (WAL, SSTables, `.purge_tmp`, Checkpoints, Index-Snapshots, KV-Segmente, Ledger, Proof-Store, Logdateien, Intent-Dateien). Prozessspeicher, Swap und Core-Dumps sind ausgeschlossen (Ausschlüsse 2 und 5) und stehen so in der Testdokumentation. **Positivkontrolle:** Vor der Löschung muss der Canary gefunden werden, sonst ist der Test leer. Im verschlüsselten Betrieb liegt der Canary nur als Chiffretext vor, dort gilt ein Lauf mit bekanntem Schlüssel als Positivkontrolle, und der negative Lauf prüft, dass nach der Schlüsselvernichtung kein verbliebener Schlüssel den Klartext rekonstruiert. Der Test läuft ohne Verschlüsselung (harter Fall, Rewrite-Purge) und mit. Bezeichner-Canary für den Metadatenrest (5.7).

**Fehlerinjektion:** `fault_vfs` injiziert Abbruch vor fsync, Kurzschreiben, `ENOSPC`, `EIO` beim Lesen, Rename-Fehler und Fehler beim Verzeichnis-fsync. Failpoints existieren nur hinter dem Feature `fault-injection`.

### 9.4 Leistungsbudgets (Obergrenzen, Messung vor Festlegung)

| Feature | Schreibpfad | Lesepfad | Start |
| --- | --- | --- | --- |
| `erasure` (Off) | 0 | 0 | 0 |
| `erasure` (Assured, Leerlauf) | höchstens 1 % p99 | 0 | höchstens 50 ms |
| `erasure` (Assured, Purge) | höchstens 5 % p99, außerhalb des Scopes nie blockiert | 0 | – |
| `erasure-evidence` | höchstens 1 % p99 | 0 | höchstens 100 ms |
| `encryption-at-rest` | höchstens 3 % p99 | höchstens 3 % p99 | höchstens 50 ms |
| `graph` (EP3) | 0 | höchstens 5 % p99 | höchstens 200 ms |
| `adapt` (EP5) | 0 | höchstens 1 % p99 | höchstens 10 ms |
| `privacy` (EP6) | 0 | 0 | höchstens 20 ms |

Criterion-Benchmarks für `put` und `commit` (p50, p99, geschlossene und offene Last gegen Coordinated Omission) in den Varianten Off, Assured im Leerlauf und Assured während Purge. Baseline: `benches/results/baseline_20260830.txt` und `bench-regression.yml`. Vor K4 werden drei Budgets aus den Baselines festgelegt (Schreibpfad in Off, Hybridsuche, Löschpfad). Benchmarks bauen je Paket.

### 9.5 CI-Hygiene

Merge-Queue mit Check je Paket, `--locked` in jedem Lauf, Feature-Matrix aus 8.8, **kein Commit auf einem roten `main`**, ein Agentenauftrag führt zu genau einem Merge. Die Blocker der älteren Analyse (Engine und Fassade kompilieren nicht, Lockfile veraltet, N-N, N-O, N-P) sind nicht bestätigt und nicht widerlegt, weil kein Build vorliegt. K0.19 klärt das mit `cargo check --locked` je Paket.

## 10. Roadmap 2026 bis 2029 und Arbeitspakete

Zeitangaben sind Größenordnungen für einen Maintainer mit Agentenunterstützung ab dem 15. Oktober 2026. Die Tagesschätzungen der Quellpläne (rund 34 Arbeitstage, rund 13 Wochen) gelten als Untergrenze des reinen Implementierungsaufwands, nicht als Terminplan.

### 10.1 Meilensteine

| Meilenstein | Zeitraum | Inhalt | Austritt |
| --- | --- | --- | --- |
| **K0 Wahrheit** | 15.10. bis etwa 23.10.2026 | Messung mit `cargo`, Gates reparieren, eine Unwrap-Baseline, AGENTS.md ersetzen, ADR-Hygiene, Scheinverdrahtung prüfen, deletion-guarantee-gate, Rohsweep und Differenztest | S1 und S2, Zahlen reproduzierbar |
| **K1 Fail-closed und Reparatur** | ab Tag 3 parallel, bis Ende November 2026 | cloud\_query nur mit guard, verify\_wiring im Start, Antwortvertrag forget, drop\_collection, engine ohne license, kein Downcast, Purge v2, Resume und Ledger im Öffnungspfad, Snapshot-Isolation | Differenztest, Rohsweep und Tenant-Matrix grün |
| **K2 Entkoppeln, dann schneiden** | Dezember 2026 bis Mitte Januar 2027 | engine ohne Default-Features, contextra-core entfernen, Produkt-Manifest, Ports erweitern, Feature `erasure` im Store, danach Parkplatz aus dem Workspace | S4: Hülle mcp höchstens 18 |
| **K3 Beweisbar vergessen** | Mitte Januar bis Ende April 2027 | Krypto-Befunde, Rohträger-Test (rot), Assured aufbauen, Zeugen aus Scans, Lineage-Kaskade, Crypto-Split | S5 bis S7, forget liefert `assured_verified` |
| **K4 Crash-sicher und vollständig** | Mai bis Juli 2027 | Crash-Matrix, Loom, Fault-Injection, Mutationstest, Formate, SSI-Read-Set, 0 unverdrahtete Funktionen, Budgets | S8 bis S12 |
| **Release v0.1.0** | frühestens August bis September 2027 | EP1 mit Off und Assured, Honesty-Klausel, Quickstart | Kriterien 13.3 |
| **EP2 Proof** | Q4 2027 bis Q2 2028 | eigenes Repository, Verifier, Zeitanker, Transparenzlog, Audit-Export, Lizenz-Aktivierung, dann Python-Bindings | unabhängiger Verifier bestätigt Beweise |
| **v1.0 (Gate)** | frühestens zweites Halbjahr 2028 | externer Krypto-Review, ein Pilotbetrieb, Formatstabilität, Budgets | 13.3 |
| EP3 | 2028 | Retrieval+ | Experimentgate je Feature |
| EP4 | 2029 | Inferenz und KV-Cache | Break-even-Gate |
| EP5, EP6 | experimentell, 2029 oder nach Bedarf | Kognition, Guard | keine Zusage |

Bis K4 abgeschlossen ist, sind entfernt (Tag park/<ID>/v1, Regel PR1 bis PR5): EP2 bis EP6, py, Optimierungen sowie alle Features außer Hybridsuche, Mandanten und Löschung. Konkret entfernt: Graph und PathRAG (EP3), Inferenz und KV-Cache (EP4), Kognition, Adapt, Router, Agent (EP5), Privacy, Sandbox, Guard (EP6), Lizenz-Aktivierung und Audit-Export (EP2). SSN, ACI, DiBud, KIVI, SnapKV, Sketched LinUCB und Spektral-SVD werden ebenfalls entfernt und nicht als Feature-Flag gehalten. Irminsul und BSI-Mapping werden nicht vorgezogen (Widerspruch 14).

### 10.2 K0: Wahrheit

Geschützte Pfade (xtask, governance, .github, `capabilities.toml`, AGENTS.md) ändert nur der Maintainer mit ADR, das ist nicht an Agenten delegierbar.

| Nr. | Aufgabe | Akzeptanz | Stand am Klon |
| --- | --- | --- | --- |
| K0.1 | `compute_crate_layers`: Fehler propagieren, Zyklen als Verstoß melden | Negativtest mit Zyklus wird rot | offen \[G\] |
| K0.2 | check-dag um Ring-Prüfung nach `capabilities.toml` ergänzen oder durch check-ring-layering ersetzen | Negativtest mit verletzender Testcrate wird rot | offen |
| K0.3 | durable-fs in die Ring-0-Ordnung, stale Allowlist-Eintrag graph zu store entfernen | Gate-Lauf ohne diese Befunde | offen |
| K0.4 | CI-Status des Jobs Ring Layering Check und der Branch-Protection klären | Ergebnis im Dokument | offen, aus dem Klon nicht ablesbar |
| K0.5 | Eine Unwrap-Baseline, `.github/unwrap_baseline.txt` wird generiert | eine Quelle | offen |
| K0.6 | `governance/security_ports.toml` anlegen, Gate prüft reale Implementierungen | Gate in CI | offen \[G\] |
| K0.7 | prune-branches reparieren oder abschalten | Workflow läuft oder ist aus | \[U\] |
| K0.8 | Toolchain 1.89.0 in Agentenumgebung und CI | `cargo xtask harness-list` läuft | Pin vorhanden \[G\], Umgebung \[U\] |
| K0.9 | Hülle und Gate-Zahlen mit `cargo` messen (Anhang A) | Zahlen reproduzierbar | offen, in dieser Prüfumgebung nicht möglich |
| K0.10 | AGENTS.md durch die Regeln dieses Dokuments ersetzen (Rangordnung, keine Roadmap Phasen 3 bis 6, keine toten Pfade) | doc-truth grün | offen |
| K0.11 | ADR-Hygiene (Dubletten, Status, Herkunft der E-Beschlüsse), Rangordnung als ADR (DE-19) | ADR akzeptiert | offen |
| K0.12 | orphan-symbols, check-fbs-drift, check-ring-capabilities-consistency in `gates.toml`, `--json` nur JSON auf stdout | Katalog gleich Gates | offen |
| K0.13 | je Wiring-Symbol einen echten Verbraucher im Composition Root oder löschen | 0 Wiring-Symbole ohne Verbraucher | offen |
| K0.14 | VERIFIED-Etiketten nur mit Reproduktionslog und Prüfer ungleich Implementierer | Regel in AGENTS.md | offen |
| K0.15 | publish-pypi, build-wheels, release-mcp-binary bis K4 nur manuell | Trigger geprüft | \[U\] |
| K0.16 | deletion-guarantee-gate, Rohsweep-Orakel, Differenztest | ohne diese Tests sagt gemergt nichts | offen |
| K0.17 | Ring-Regel festlegen (DE-16), Ordnung aus einer Quelle | Gate und Manifest stimmen überein | offen |
| K0.18 | CI: Merge-Queue, Check je Paket, `--locked`, Feature-Matrix, kein Merge auf rotes main | Workflow vorhanden, Branch-Protection dokumentiert | offen |
| K0.19 | `cargo check --locked` je Paket als erste Messung, N-N, N-O, N-P bestätigen oder schließen | Log im Repository | offen |
| K0.20 | Codeblöcke in Plänen gelten als Illustration, doctrine-scan läuft über neue Codepfade | Regel in AGENTS.md | offen |

### 10.3 K1: Fail-closed und Reparatur

| Nr. | Aufgabe | Akzeptanz | Stand am Klon |
| --- | --- | --- | --- |
| K1.1 | `contextra_cloud_query` nur mit Feature `guard` | Aufruf liefert Werkzeug nicht verfügbar | offen |
| K1.2 | verify\_wiring mit echten Ports im Startpfad, Pflichtliste aus security\_ports.toml | Start ohne Sicherheits-Port scheitert | offen |
| K1.3 | Antwortvertrag forget und drop\_collection mit einheitlichem Statusvokabular (5.4) | MCP-Test über beide Pfade | teilweise |
| K1.4 | MCP reicht Request-Optionen an erase\_document durch | Assurance-Option ändert das Ergebnis | offen \[G\] |
| K1.5 | Krypto-Review: Nonce-Länge, Schlüsselaufbau, unsafe im Test (DE-09) | Befundliste im Dokument | offen |
| K1.6 | drop\_collection: Beweisordner aus Konfiguration, Instanz-ID aus dem Manifest, Schlüssel je Datenbank, Err nicht verwerfen, Purge-Ergebnis in die Antwort | Purge-Fehler führt zu Fehlerantwort, keine Artefakte in /tmp | offen |
| K1.7 | engine zu license entfernen, LicenseGate nur als Port | Ring-Gate ohne diese Kante | offen \[G\] |
| K1.8 | erase\_document und alle fünf Downcasts durch `erasure_backend()` ersetzen, Test mit TenantScopedStorage | `grep downcast_ref` gleich null für LsmStorage | offen \[G\] |
| K1.9 | Docstring von erase\_document an den Code anpassen, bis Ledger-Append umgesetzt ist | doc-truth grün | offen |
| K1.10 | Purge v2 nach 6.1 bis 6.3 einschließlich `wal/retire.rs` | Differenztest, Rohsweep, Schreibvorgänge außerhalb des Scopes laufen | teilweise |
| K1.11 | MVCC-Rescan statt write\_seq | Paralleler Schreibvorgang wird erkannt | offen |
| K1.12 | Resume und Ledger im Öffnungspfad, RestoreGuard im Checkpoint-Restore | Absturz im Purge wird fortgesetzt, Restore bringt Gelöschtes nicht zurück | offen |
| K1.13 | `u64::MAX` als Lesezeitpunkt ersetzen | Lesen immer mit Snapshot | offen |
| K1.14 | Standard-delete unter TenantScopedStorage ohne CapabilityUnsupported | `tenant_isolation_facade` grün | offen |
| K1.15 | KvDeleteMode-Default | entfällt: am HEAD TombstoneOnly mit Test \[G\] | **erledigt** |
| K1.16 | Builder: verify\_wiring für alle sieben No-op-Optionen | check-builder-wiring grün | offen |
| K1.17 | Failpoints nur hinter `fault-injection` | `grep` gleich null | offen |
| K1.18 | Orphan-GC mit Scrub, `manifest_generation` nie zurücksetzen | Orphan-Test | offen |
| K1.19 | Schlüssellayout vereinheitlichen (DE-06), Migration im Upgrade-on-open | Test mit IDs 4 und 42, Wiederanlage unter gleichem Namen | offen |

### 10.4 K2: Entkoppeln, dann schneiden

**K2a** (PRs unter etwa 400 geänderten Zeilen, unten zuerst, mcp zuletzt):

| Schritt | Änderung | Hülle mcp danach \[E\] |
| --- | --- | --- |
| 1 | engine: `default = []`. Wo ein Rand-Crate ohne Feature nicht kompiliert: Port oder `cfg`. engine zu license raus | 25 auf ca. 21 |
| 2 | db: graph und adapt regulär entfernen | ca. 19 bis 20 |
| 3 | mcp: privacy und adapt hinter Feature oder Port, router aus den contextra-Features nehmen, Werkzeugliste aus der Registrierung | ca. 18 bis 19 |
| 3a | Parkplatz-Crates (EP2 bis EP6) aus der Workspace-Liste der Wurzel-Cargo.toml nehmen, nachdem Tag park/<ID>/v1 gesetzt ist. Der Dateibaum bleibt in diesem Schritt bestehen. Das macht die Entfernung für den Abhängigkeitsbaum sofort wirksam und für Git reversibel. K2b bleibt als Löschung des Dateibaums nach Messung. |  |
| 4 | contextra (Fassade): Default-Kette prüfen | Fassade 17 |
| 5 | contextra-core ersatzlos entfernen: 405 Dateien, 3 bis 4 mechanische PRs nach Modul | core verschwindet |
| 6 | `default-members` auf EP1 und LAB |  |
| 7 | `ports::erasure` erweitern (DE-03, DE-04), nur Typen, rein additiv | kompiliert, keine Verhaltensänderung |
| 8 | Feature `erasure` im Store, Fence als `Option<ErasureGate>` mit Check in der ersten Haltephase, Engine-Löschpfad ohne Footprint und Zeugen in Off | Messung gegen Baseline, Zähler-Mocks zeigen keine Scans |
| 9 | `erasure_backend()` auf StorageEngine, alle fünf Downcasts ersetzt, `StorageErasureExt` entfernt | `grep downcast_ref` gleich null |

**K2b:** Erst wenn die Hülle das Ziel erreicht und check-ring-layering strict grün ist, entfernt ein einziger PR mit Label `scope:core-cut` die Parkplatz-Crates aus Workspace und Dateibaum (DE-14). Vorher Tag `park/EPn/v1`. **Austritt K2:** Default-Hülle mcp höchstens 18, Kern zu Parkplatz 0, Produkt-Manifest EP1 existiert, `capabilities.toml` führt je Crate Produkt und Status.

### 10.5 K3: Beweisbar vergessen

Die Reihenfolge ist verbindlich:

1. **Krypto-Befunde schließen** (Nonce-Länge, Schlüsselaufbau, unsafe im Test, KMS-Widerruf fail-closed, WAL-Scan-Härtung, partielle AEAD-Frames).
2. **Veto klären:** gelöst durch DE-11, VETO-F02 bleibt, ADR-097 wird in v0.1.0 nicht eingesetzt. Ein ADR dokumentiert das.
3. **Rohträger-Test schreiben**, bevor die Nachweis-Kette verdrahtet wird. Er stammt aus der Spezifikation und ist zunächst rot.
4. **Assured aufbauen:** Koordinator mit Zustandsmaschine, Hochwassermarke, Ledger-Append vor Tombstone, Resume, RestoreGuard, Crypto-Erase.
5. **Zeugen aus echten Scans** (Store, HNSW, Text, Relationen, Ledger). Keine Fabrikation.
6. **Purge verdrahten:** `purge_physical` nach erase\_document, `dispose` im Purge-Pfad. Löschumfang: alle abgeleiteten Speicher.
7. **KV-Nachweis:** `generate_kv_deletion_proof`, KV-Segment-Lesen und -Schreiben, `KeyRegistry::open_durable` im Startpfad.
8. **Nachweis-Ausstellung verdrahten:** `DpiDeletionRuntime::open` im Startpfad, Signer-Schlüssel aus Konfiguration.
9. **Lineage-Kaskade:** Quelllöschung entfernt abgeleitete Daten. Ohne sie darf kein Aggregat-Feature aktiv werden.
10. **Betriebsumgebung ehrlich machen:** HNSW-Zeroisierung, mlock, Scrub-Verweigerung, Kopienstatus.
11. **Crypto-Split** ohne Shim (crypto-core, crypto-kv, erasure-evidence).
12. **Dokumentation** gleich Code (Rechtsmatrix, Betriebsvorgabe, Python-Doku, Stufenbeschreibung).
13. **proof wird erst `assured_verified`, wenn der Rohträger-Test grün ist.**

Zusätzliche Aufgaben: K3.14 `LayerEraser` für store, vector, text, graph (Rohsweep entdeckt keine Indexdateien). K3.15 WAL-Barriere (CRASH-15). K3.16 Snapshot-Lebensdauer und Startbedingung (CRASH-16). K3.17 Record-Verschlüsselung mit Collection-DEK vor dem WAL-Append und Dateien je Collection für Indizes (5.7). K3.18 Schlüsselhierarchie und dreistufige Vernichtung (CRASH-13, Test „kein Shred-Schlüssel mit HKDF-Label“). K3.19 Ledger-Format mit geschlüsselten Digests (CRASH-12, CRASH-17). K3.20 CopyRegistry und `checkpoint_policy`.

**Austritt K3:** Verdrahtungen laufen über Integrationstests auf dem echten Pfad, Rohträger-Test grün, forget liefert für Einzeldokumente einen belegten Assured-Befund, drop\_collection ohne die Fehler des Drop-Pfads.

### 10.6 K4 und K5

**K4:** Nachweisformat nach 7.2 mit `verify_strict`, TLV, getrennten Domain-Tags und `pub(crate)`-Zeugen-Konstruktoren. Verifier-Crate mit Ed25519-Grenzfällen. Zeitanker und Transparenzlog (EP2-Anteil). Fassade mit Features, Builder-API, einem Default. Crash-Sicherheit: Compaction und MANIFEST, HMAC-Zustand, Group-Commit-Liveness, Checkpoint-Indizes, atomares `put_if_absent`, SSI-Read-Set, Hochwassermarke gegen WAL-Kürzung, Legacy-WAL-Entfernung, HNSW-Arena-Zeroisierung. `erasure_adopt()` (K4.10) und `contextra_erasure_status` mit `wait_ms` (K4.11). Pflicht-Gates: Fault-Injection, Loom, Mutationstest für Crash/WAL, Löschung/Krypto und Audit. Budgets laufen in xtask und CI. **Austritt K4:** Zähler öffentlicher Funktionen ohne Aufrufer gleich 0 für EP1, Crash-Matrix grün, Release v0.1.0. **K5:** EP2 zuerst, dann EP3 bis EP6 nach Bedarf, je Produkt Manifest, Hüllen-Gate, Akzeptanzliste und Abschaltprobe.

## 11. Ebenen über dem Fundament und Erhaltungsregister

Kein geplantes Feature geht verloren. Jedes hat Ebene, Voraussetzung und Zeitfenster. Die Parkplatz-Crates behalten ihren Code über Tag `park/EPn/v1` (DE-14).

| Ebene | Produkt | Inhalt | Start | Zustand im Hauptzweig bis K4 |
| --- | --- | --- | --- | --- |
| F Fundament | EP1 Contextra Memory | Speicher, MVCC, Hybridsuche, Mandanten, Off und Assured, MCP und Fassade | jetzt | aktiv |
| N Nachweis | EP2 Contextra Proof | Attested, Verifier, Zeitanker, Transparenzlog, Audit-Export, Lizenz-Aktivierung | nach K4, eigenes Repository | entfernt, park/EP2/v1 |
| R Retrieval+ | EP3 | Graph, PathRAG, OpenIE, Conformal, DiBud | 2028 | entfernt, park/EP3/v1 |
| I Inference | EP4 | Embedder, Reranker, Generatoren, KV-Cache | 2029, Break-even-Gate | entfernt, park/EP4/v1 |
| C Cognition | EP5 | Konsolidierung, Adapt, Router, Agent, Bandits, Spectral Dedup | experimentell | entfernt, park/EP5/v1 |
| G Guard | EP6 | Privacy, Surrogat-Vault, Sandbox, WASM-Merge-Operatoren | 2029 oder nach Bedarf | entfernt, park/EP6/v1 |
| LAB | intern | Testkit, Benchmarks, xtask, xtask-heavy | laufend | aktiv, nur Werkzeuge, nicht ausgeliefert |

### 11.1 Aktivierungsregel für jede Ebene

Eine Ebene startet erst, wenn (0) Die Ebene ist im Hauptzweig nicht vorhanden oder wird über git checkout park/<ID>/<Version> in einem Wiederherstellungs-Branch geholt. Die Wiederherstellung unterliegt PR4, (1) die darunterliegende ihr Austrittskriterium erfüllt, (2) ein Produkt-Manifest, ein Hüllen-Gate, eine Akzeptanzliste und eine Abschaltprobe existieren, (3) jedes Feature ein **Experimentgate** hat (Messgröße, Baseline, Abbruchkriterium; Zahlen aus Forschungsberichten sind Anlass für einen Versuch, nie Evidenz, INV-EVIDENCE-1), (4) die Veto-Liste geprüft ist (VETO-F02 HNSW, VETO-F10 keine mandantenübergreifenden Flüsse, VETO-OP03 keine Audio- und Sprachfunktionen, VETO-ADR) und (5) das Feature im Schreibpfad nichts kostet (PE2). Vier Querregeln gelten für alle Ebenen:

1. **Lineage:** Jede abgeleitete Struktur (Graph, Hyperkanten, Aggregate, KV-Cache-Segmente, Spektralfaktoren) kennt ihre Quelldokumente und wird über die Lineage-Kaskade bereinigt. Ohne Kaskade darf das Feature nicht aktiv werden.
2. **Mandantentrennung (VETO-F10):** Keine abgeleitete Struktur und kein Cache wird über Mandanten geteilt.
3. **Determinismus:** gleiche Eingaben, gleiche Ausgaben. Zufall nur über `Rng`, Seeds werden persistiert.
4. **Zero-Panic und Gleitkomma-Regeln** (8.4).

### 11.2 Erhaltungsregister

| Funktion | Ebene | Voraussetzung und Hinweis | Zeitfenster |
| --- | --- | --- | --- |
| Hybridsuche Vektor, BM25, RRF, Kalibrierung (Platt, isotonisch, adaptiv) | F (rank im Kern) | S3 | jetzt |
| Split-Conformal-Fusion mit ACI | R | Coverage-Messung auf dem Projektkorpus | 2028 |
| DiBud (budgetierte RRF-Präfixe) | R | Zertifizierung gegen Ground Truth | 2028 |
| Graph, find\_path, PathRAG, APPRH, Kaskaden-Invalidierung | R (EP3) | `find_k_paths` bleibt `pub(crate)`, MCP-Werkzeug explain nutzt find\_path | 2028 |
| OpenIE und Hyperkanten | R | Aufzeichnung der Extraktionen, Lineage-Kaskade aus K3 | 2028 |
| Domänenvokabulare, Cross-Encoder-Reranker | R und I | echter Aufrufer im Suchpfad | 2028 bis 2029 |
| Inferenz-Backends (candle, ollama, onnx) | I (EP4) | onnx bleibt Experiment, kein Pfeil von onnx auf candle | 2029 |
| KV-Cache Irminsul, KIVI, Tier 2, Prefix-Radix-Baum | I | `aligned_alloc` (unsafe, ADR-019) wandert nach `sys` (P4) | 2029 |
| Sketched LinUCB | C | Regret-Messung gegen Voll-LinUCB | experimentell |
| Spektrale Hyperkanten-Dedup | C | nur nach Lineage-Kaskade | experimentell |
| Konsolidierung, Vergessensdynamik, PID-Regler, Router, Agent | C | `cognition` hängt am Port `ConsolidationTask` | experimentell |
| Privacy, DLP-Surrogat-Vault, Egress-Gateway, WASM-Merge-Operatoren | G (EP6) | CRDT-Property-Tests, Freischaltungs-Gate | 2029 oder nach Bedarf |
| Lizenz-Aktivierung, Audit-Export (Art. 30), AVV-Generator (Art. 28) | N (EP2) | Attested belegt | Q4 2027 bis 2028 |
| Python-Bindings | F, nach K4 | dünn, nur über die Fassade | Q4 2027 |
| SSN statt SSI | F (mvcc), nach K4 | Bedingungen aus 8.2 | offen |
| **BMSSP und FindPivots** | **ausgeschlossen** | Theorieschranke, Konstanten unbekannt | nie im Hot Path |

Arxiv-Kennungen in den Quellen sind \[U\]. Eine Kennung begründet keinen Einsatz, das Experimentgate tut es.

### 11.3 Ebene R: Retrieval+

**Split-Conformal-Fusion.** Der fusionierte Score `f(d)` liegt in \[0,1\]. Die Kalibrierung nutzt **nur relevante** Beispiele mit `s_i = 1 − f(d_i)`. Der Schwellwert `q̂` ist der `⌈(n+1)(1−α)⌉`-kleinste Score, die Menge ist `{d : 1 − f(d) ≤ q̂}`. Die Garantie lautet: Ein relevantes Dokument liegt mit Wahrscheinlichkeit mindestens `1 − α` in der Menge (klassenbedingter Recall) unter Austauschbarkeit von Kalibrier- und Testanfragen je Collection. Ein Nonconformity-Maß `s = f(d)` für irrelevante Dokumente ist durch die Garantie nicht gedeckt und dient nur der Precision-Messung. Randwerte: leere Kalibriermenge oder `rank > n` ergibt ALL, `α` außerhalb von (0,1) ist ein Fehler.

**ACI.** `α_{t+1} = α_t + γ (α − err_t)` in Festkomma (i128), `γ` dyadisch. `α_t <= 0` ergibt ALL, `α_t >= 1` ergibt EMPTY. Die Garantie ist eine Langzeit-Mittelgarantie bei Drift, keine Endlich-Stichproben-Garantie, und der Text darf beides nicht vermischen.

**DiBud.** Untere Schranke `L(d)` ist die Summe von `1/(60 + r_ch(d))` über gelesene Kanäle. Restwert eines unvollständigen Kanals `U_ch = 1/(60 + n_ch + 1)`. Obere Schranke `U(d) = L(d)` plus die Summe der `U_ch` ungelesener Kanäle. Ein Dokument ist als Rang `m` zertifiziert, wenn `L(d) > max_{e≠d} U(e)` und `L(d)` mindestens die Summe aller `U_ch` ist. Rechnung rational oder in Festkomma (`u128`, Q64.64), Ausreißertest gegen die vollständige RRF als Ground Truth ist Pflicht.

**PathRAG.** Knotenbudget `B` über alle `k` Pfade, Relevanz `Rel(P)` als Mittel des Personalized-PageRank-Werts (Forward Push, ε = 10^-5), Diversität als minimale Jaccard-Distanz zu bereits gewählten Pfaden, Score `Rel·w_rel + Div·w_div`, Komplexität O(B log B). Bindungen werden deterministisch gelöst (Score, dann Knoten-ID aufsteigend).

**OpenIE und Hyperkanten.** Chunk, Satzsplitting, regelbasierte NER, LLM-Validierung über `LlmExtractionPort`, Kanonisierung, CSR-Einfügung. „Temperatur 0 und fester Seed“ macht LLM-Ausgaben nicht bitgleich über Hardware und Backends. Deshalb werden Ausgaben **aufgezeichnet** (Cache mit Schlüssel `Hash(model_id, prompt, params)`), die Aufzeichnung ist die deterministische Quelle, `extractor_version` steht an jedem Ergebnis. n-äre Relationen werden Hyperkanten. **Kaskaden-Invalidierung** läuft iterativ mit `BTreeSet` (Zyklenschutz, deterministische Reihenfolge) und festem Knotenlimit. Überschreitung ist ein Fehler, keine stille Kürzung.

### 11.4 Ebene I: Inferenz und KV-Cache

Die Engine kennt nur `EmbeddingProvider` und `LlmTextGenerator`. **Irminsul** (pro Mandant): Segmente mit `prefix_refcount > 1` sind nie evictbar. Für die übrigen wird der Attention-Score über ein Fenster `W ≈ 32` gemittelt. Pro Head bleiben die Top-`k` Positionen plus die jüngsten `L_recent` Token. Sortierung mit `total_cmp` (NaN ans Ende), Eviction aufsteigend nach Priorität bis zum Mandantenbudget, danach Tier-2-Spill. Budgetüberlauf ist `budget_exhausted: true`, kein Panic. **KIVI (2 Bit):** Keys pro Kanal, Values pro Token, Gruppengröße `g` zwischen 32 und 128 (Standard 64), Restfenster 128 bis 512 Token in voller Genauigkeit, `scale = (x_max − x_min) / 3`, `zero_point = round(−x_min / scale)`, `X_q = clamp(round(X / scale) + zero_point, 0, 3)`. Pflichtschutz: `x_max == x_min`, NaN und Inf werden vor der Quantisierung geprüft. **Tier 2:** Normalisierung, KIVI, zstd Level 1, AEAD mit dem Segment-Schlüssel, mmap. **Löschkopplung:** Wird ein Quelldokument gelöscht, werden abgeleitete KV-Segmente evictet und ihr Segment-Schlüssel vernichtet (Layer 35).

### 11.5 Ebene C: Kognition

**Sketched LinUCB.** Kontext `x` wird mit einer dünnen JL-Matrix `S` (Achlioptas: Einträge ±√(3/k) mit Wahrscheinlichkeit je 1/6, sonst 0, `k` etwa 50 bis 100) auf `z = S x` projiziert. Sherman-Morrison-Update in `k×k`, UCB `z^T θ̂_a + β √(z^T A_a^{-1} z)`. Der Seed der Matrix stammt aus `Rng` und wird **persistiert**. Re-Sketching bei Drift setzt alle Arme zurück. **Spektrale Dedup:** Brand-Update der inkrementellen SVD, Duplikat bei Kosinus-Ähnlichkeit über `θ` (Startwert 0,92, auf dem Projektkorpus zu kalibrieren, nicht übernommen). Re-Orthogonalisierung nach festem Takt, deterministische Vorzeichenkonvention. Beim Zusammenführen bleibt die **Provenienzmenge** erhalten, die Hyperkante entfällt erst bei leerer Provenienz, die Spektralfaktoren sind abgeleitete Daten und werden nach einer Löschung neu aufgebaut. Konsolidierung läuft in Produktion heute nie \[U\], sie braucht einen Aufrufer im Lebenszyklus oder wird entfernt.

### 11.6 Ebene G, verworfene Verfahren und Experimentgates

**G:** Privacy mit DLP-Surrogat-Vault, Egress-Gateway (fail-closed, Feature `guard`, Audit-Spur mit Layer 40), Sandbox mit WASM-Merge-Operatoren (Fuel und Timeout, Modul-Cache statt Neukompilierung je `merge()`). Egress liegt außerhalb des Löschumfangs. **Verworfen:** BMSSP und FindPivots, HyperQuant (für LLM-Gewichte, nicht für dynamische KV-Caches), gemischte Gleitkommagenauigkeit (verletzt Bitgleichheit zwischen AVX2 und NEON), ML-basierte Compaction (CPU-Varianz, Latenzspitzen).

| Feature | Messgröße | Baseline | Abbruch |
| --- | --- | --- | --- |
| Conformal-Fusion | Recall der Menge gegen `1 − α`, Mengengröße | RRF mit Platt oder isotonisch | Coverage unter `1 − α − 0,02` oder Mengengröße über Faktor 3 |
| DiBud | Zahl gelesener Einträge bei gleicher Top-k-Menge | volle RRF | Abweichung der Top-k-Menge ungleich null |
| PathRAG | nDCG und Latenz p99 | reine Hybridsuche | kein nDCG-Gewinn oder p99 über 5 % |
| Irminsul und KIVI | Perplexität, Speicher, Latenz | volle Genauigkeit, LRU | Perplexitätsverlust über Schwelle (je Modell festlegen) |
| Sketched LinUCB | kumulativer Regret | Voll-LinUCB | Regret über Faktor 1,2 |
| Spektrale Dedup | Duplikat-Precision und -Recall auf Stichprobe | Hash-Gleichheit | Precision unter 0,95 |

## 12. Betriebsmodell für Agenten

Der Engpass ist die Prüfkapazität des Maintainers (rund 3.740 Commits unter einem Konto, 460 durch den Jules-Bot, 124 Karten, 32 Gates), nicht die Sitzungszahl.

- **Höchstens drei offene PRs**, jeder betrifft genau ein Crate oder eine Gate-Datei. Zwei Karten ändern nicht dasselbe Crate gleichzeitig. Geschützte Pfade gehen nur an den Maintainer.
- **Kartenformat** (Erweiterung von `.jules/tasks/_TEMPLATE.toml`): Pflichtfelder `product` (EP1 oder LAB), `messgroesse` (Start und Ziel), `acceptance_by` (spezifikation, referenzmodell, zweite\_karte oder compiler, der Implementierer schreibt den Abnahmetest nicht), `verbot_parkplatz`, `milestone` (K0 bis K5), `adr_ref`.
- **Regeln:** keine neue `pub fn` ohne Produktionsaufrufer (neue Funktionen sind `pub(crate)`), kein Wiring-Symbol ohne Verbraucher, keine neuen Crates oder Abhängigkeiten ohne ADR, Parkplatz-Code wird nicht gelesen, jede Verdrahtung hat einen Integrationstest über den echten Pfad, VERIFIED-Etiketten nur mit Reproduktionslog und Prüfer ungleich Implementierer, Dokumente mit HEAD-Hash gelten 48 Stunden.
- **Dokumentation:** Alle öffentlich bleibenden Symbole aus der Liste der undokumentierten Schnittstellen erhalten Doc-Kommentare mit Fehler-, Panik- und Sicherheitsgarantien. Die 512 öffentlichen Funktionen ohne Aufrufer \[D\] werden verdrahtet, auf `pub(crate)` beschränkt oder entfernt.
- **Definition of Done je Karte:** Test über den echten Pfad, rot ohne die Änderung; Messgröße verbessert und im PR belegt; keine neue ungenutzte `pub fn`; Gates scope-guard, ratchet, test-integrity, gate-weakening grün; ADR bei geschützten Pfaden.
- **Dokumentenpolitik:** Dieses Dokument ist die einzige normative Spezifikation. Alle anderen Planungsdokumente wandern nach `docs/archive`. Die EP-Spezifikationen in `docs/` werden Anhänge je Produkt und dürfen diesem Dokument nicht widersprechen. AGENTS.md verweist weiter auf das nicht vorhandene `docs/14-10-26/` \[D\] und wird ersetzt (K0.10).
- **Erste Karten:** (Maintainer) Zyklen in check-dag, Ring-0-Ordnung. (EP1) forget reicht Request-Optionen durch, cloud\_query nur mit guard, verify\_wiring mit echten Ports, `wal/retire.rs` Fehlerbehandlung, Rohträger-Test (zunächst rot), Symbole aus core nach types.

Entfernung und Welle: Eine Welle entfernt Features in disjunkten Crates. Jede Karte nennt den Tag, den sie erzeugt, und den Wiederherstellungsbefehl. Das Gate scope-guard prüft, dass eine Entfernungskarte nur Pfade des entfernten Produkts ändert.

## 13. Release, Anspruchsstufen, Honesty-Klausel und Lizenz

### 13.1 Anspruchsstufen

| Stufe | Bedingung | Erlaubte Formulierung |
| --- | --- | --- |
| C0 | Funktion im Code, kein Test über den echten Pfad | keine Aussage im Produkttext |
| C1 | Integrationstest über Fassade oder MCP grün, Gate grün | unterstützt, löscht logisch (Tombstones, Index-Bereinigung) |
| C2 | Rohträger-Test und Crash-Matrix grün | löscht physisch nach Purge, Befund `assured_verified` |
| C3 | unabhängiger Verifier hat Nachweise nachvollzogen, externer Review liegt vor | kryptografisch beweisbar, extern prüfbar |

Am HEAD gilt für Löschung C1 (nur Einzeldokument, logisch). Für Collection-Drop-Beweise gilt keine Stufe. README, Positionierungssatz und ARCHITECTURE.md werden auf die tatsächliche Stufe gesenkt. Ohne Verschlüsselung leiten sich die Ansprüche aus der Fähigkeit ab (DE-07): „physisch gelöscht, SSD-Remanenz ausgenommen“ statt „kryptografisch gelöscht“.

### 13.2 Honesty-Klausel (Entwurf, Wortlaut bestätigt der Maintainer)

Sie gehört in die Release-Notes und die Nutzerdokumentation:

> Contextra löscht Daten logisch sofort und physisch nach einem Purge. Der Löschnachweis ist ein Audit mit definierten Zeugen, kein mathematischer Beweis. Nicht erfasst sind: (1) SSD-Remanenz und Wear-Leveling, (2) Prozessspeicher vor dem Überschreiben, (3) externe Backups vor dem Löschzeitpunkt, (4) Auftragsverarbeiter und Netzwerkspeicher, (5) Swap und Core-Dumps. Daten, die jemand außerhalb von Contextra kopiert hat, bleiben bestehen. Cloud-Egress liegt außerhalb des Löschumfangs. Im Modus Off gibt es keinen Restore-Schutz: Ein Restore kann Gelöschtes zurückbringen.

### 13.3 Release-Kriterien

| Version | Kriterium |
| --- | --- |
| v0.1.0 | S1 bis S12, Hülle höchstens 18, Legacy-WAL entfernt, Formate versioniert, ErasureMode Off und Assured, Nutzerdokumentation (Quickstart, MCP-Einrichtung, Löschvertrag, Grenzen), Sicherheitsmeldeprozess |
| v1.0 (Gate) | zusätzlich externer Krypto-Review geschlossen, mindestens ein Pilotbetrieb, Upgrade-Test, Leistungsbudgets |
| EP2 Attested | eigenes Repository, Verifier ohne gemeinsamen Code mit dem Erzeuger, zweite Verifier-Implementierung (DE-25) |

### 13.4 Lizenz, Lieferkette, Distribution

Die Workspace-Lizenz ist MIT oder Apache-2.0. **Entscheidung (DE-18):** Die Lizenz steuert nur das Ausstellen neuer Beweise (Attested), nie `open` und nie die Löschpflicht. EP1 bleibt vollständig offen. Format-Spezifikation und Verifier sind offen lizenziert, damit Prüfer unabhängig bleiben. **Maintainer-Vorbehalt:** Die Lizenz des EP2-Ausstellers (offen oder abweichend) ist eine geschäftliche Entscheidung und fällt vor dem EP2-Zuschnitt. Der Vorbehalt ändert keine technische Entscheidung dieses Dokuments. Vor v0.1.0 fehlen: Offenlegungsprozess für Schwachstellen (Kontakt, Fristen), SBOM, Signatur der Release-Binaries, reproduzierbare Builds. Vorhanden: `deny.toml`, `supply-chain`, Unsafe-Inventar \[G\]. Auslieferung: Git-Abhängigkeit und MCP-Binary, crates.io nach lösbaren Pfadabhängigkeiten. Die Workflows build-wheels, publish-pypi und release-mcp-binary werden bis K4 nur manuell ausgelöst.

## 14. Risiken

| Risiko | Gegenmaßnahme |
| --- | --- |
| Kein Build in der Prüfumgebung, Zahlen sind \[E\] oder \[H\] | K0 beginnt mit der Messung (Anhang A) |
| AGENTS.md bindet Agenten an Phasen 3 bis 6 | AGENTS.md vor allen anderen Karten ersetzen (K0.10) |
| Löschversprechen nicht belegt | Anspruchsstufen, Rohträger-Test, bis C2 kein Wort beweisbar |
| Purge-Umbau ist ein Eingriff in den Speicherkern, Fehler verlieren Daten | Differenztest, Fehlerinjektion, Loom sind Pflicht |
| Fence-Annahme (Serialisierungspunkt) nicht belegt, `commit_mutex` ist async und wird zweimal gehalten | Loom, Benchmark, Rückfall Option A (6.3) |
| Schlüsselverlust ist Datenverlust | Schlüssel-Sicherung, absturzsicheres RevocationLog, Wartefrist 72 Stunden |
| Umschalten der Defaults deckt Kompilierfehler auf (engine ohne graph: 19 Dateien, 90 Zeilen; core: 405 Dateien) | PRs unter etwa 400 Zeilen, Abschaltprobe |
| Migration des Schlüssellayouts (DE-06) bricht Format | Formatversion, Upgrade-Test, Wiederanlage-Test |
| Scheinverdrahtung durch Wiring-Dateien | Gate, Verbraucher im Composition Root |
| Falsche Verifikation durch Etiketten | Reproduktionslog, Prüfer ungleich Implementierer |
| Single Maintainer, Review-Last | drei PRs, Karten, geschützte Pfade |
| ADR-Wildwuchs und Dokumentenlast (7,0 MB) | ADR-Hygiene, ein normatives Dokument, Archiv |
| Ring-Gate eventuell rot auf main | CI-Status zuerst klären (K0.4) |
| Krypto-Review widerspricht GCM-SIV | nur mit Befund. Der Schlüsselaufbau (5.5) ist verfahrensunabhängig |
| Entfernte Features driften bei Wiederherstellung gegen den Kern | Wiederherstellung nur per PR4, Kompilierprüfung gegen den aktuellen Kern vor Merge, Tag als Referenz |

**Was dieses Dokument nicht leistet:** Es ersetzt keinen Build. Kryptografie wurde nicht auditiert. Nicht gelesen wurden die Compaction-Engine im Detail, der WAL-Schreibpfad außerhalb des Commit-Pfads, Checkpoint-Restore, die HNSW-Zeroisierung und der KV-Cache. Aussagen der Quellpläne zu Dateinamen, Zeilennummern und Audit-Kennungen sind \[D\], soweit nicht ausdrücklich am Klon gelesen. Rechts- und Literaturaussagen (DSGVO, EDPB, RFC 3161, RFC 9162, BSI TR-02102, arXiv) sind nicht geprüft. Der Plan ist keine Rechtsberatung.

## 15. Anhänge

### Anhang A: Messprotokoll (gepinnte Toolchain 1.89.0)

```
cargo build --locked -p contextra-mcp
cargo test --workspace --locked
cargo xtask check-ring-layering --strict
cargo xtask check-ring-capabilities-consistency
cargo xtask orphan-symbols --json > orphan.json
cargo tree -p contextra-mcp -e normal --prefix none | grep ^contextra | sort -u | wc -l
cargo tree -p contextra-mcp -e normal --no-default-features --prefix none | grep ^contextra | sort -u | wc -l
cargo tree -p contextra-mcp -e normal --all-features --prefix none | grep ^contextra | sort -u | wc -l
cargo tree -p contextra-engine -e normal --no-default-features --prefix none | grep ^contextra | sort -u | wc -l
```

Je Lauf werden Befehl, Commit-Hash, Exit-Code und Ausgabe in `logs/audits` mit Datum und Namen abgelegt. Emulationswerte werden bei Abweichung korrigiert.

### Anhang B: Befundbestand aus v3 (verdichtet)

Der vollständige Befundbestand aus v3 (Teil II) bleibt gültig, soweit Abschnitt 2 ihn nicht ändert. Die wichtigsten offenen Befunde stehen hier mit der Karte, die sie schließt:

| Befund | Schwere | Karte |
| --- | --- | --- |
| check-dag kann nicht rot werden | H | K0.1 |
| Collection-Drop: Beweisartefakte in `temp_dir()`, konstante `db_instance_id`, `if let Ok` und `let _ = run_purge` | K | K1.6 |
| MCP forget nutzt `DocEraseOptions::default()` | H | K1.4 |
| `contextra_cloud_query` ohne Guard registriert | H | K1.1 |
| Purge fail-open in `wal/retire.rs`, globale Fence, Reihenfolge der Reader | K bis H | K1.10 |
| Resume, Ledger, RestoreGuard ohne Produktionsaufrufer | H | K1.12 |
| Fünf Downcasts auf LsmStorage | H | K1.8, K2.9 |
| `u64::MAX` als Lesezeitpunkt | H | K1.13 |
| `bincode` im Signaturpfad, `verify` statt `verify_strict` in Restpfaden | H | K4 (Nachweisformat) |
| Zwei Unwrap-Baselines (502 gegen 786) | M | K0.5 |
| 96 von 111 EP1-Verdrahtungsfunktionen ohne strikten Produktionsaufruf \[H\] | H | K0.13, S3 |
| engine zu license, Default-Features ziehen Parkplatz in die Hülle | H | K1.7, K2 |

### Anhang C: Quellenzuordnung

Basis ist Gesamtspezifikation v3 mit ihren zehn Quelldokumenten (Gesamtplan v2, Implementierungsspezifikation, Architektur-Masterplan v2, Architekturplan v1, Architektur-Optimierung v1, Systemdesign-Spezifikation v1, Advanced Algorithms Spec, SOTA-Feinspezifikation). v4 ergänzt die Prüfung am Klon (Abschnitt 2) und die Entscheidungen DE-01 bis DE-26 (Abschnitt 3).

## 16. Archivanhänge D bis J: vollständiger Bestand aus v3

Die Anhänge D bis J übernehmen den gesamten Bestand von v3 (Teil II und die Prüfberichte), damit v4 ein Superset von v3 ist. **Vorrang:** Wo Abschnitt 2 (Re-Verifikation am Klon) oder das Entscheidungsregister in Abschnitt 3 abweicht, gilt der normative Teil oben. Zahlen mit \[D\] stammen aus den Quellplänen und sind am Klon nicht neu geprüft.

### Anhang D: Lage, Kennzahlen und Quellenlage

**D.1 Zusammenfassung der Lage (v3 II.1)**

1. **Das Projekt ist nicht stabil, und das ist messbar.** 4.224 Commits seit 05.05.2026, davon 724 seit 11.10. (bis zu 254 pro Tag). Seit 11.10.: 149 feat, 118 docs, 39 fix, 9 test \[G\]. Der Orphan-Gate steht in allen lesbaren Läufen auf fail (36 bis 42 Orphans). Ein FlatBuffers-Drift-Lauf endet mit Exit 1. Ein belastbarer Build- und Testlauf liegt in keinem Dokument vor.
2. **Der Kern ist das Problem, nicht die Breite.** 96 der 111 als verdrahten markierten Kernfunktionen haben keinen strikten Produktionsaufruf \[H\], nur 3 haben einen Aufrufer außerhalb des eigenen Crates.
3. **Drei Rangordnungen der Quellen widersprechen sich.** AGENTS.md (geschützter Pfad) verweist auf ein nicht existierendes Masterplan-Dokument und bindet Agenten an Phasen 3 bis 6 (EP3 bis EP6). Das widerspricht dem Freeze und ist die wahrscheinlichste Ursache der Änderungsrate außerhalb des Kerns \[G\].
4. **Das Löschversprechen ist nur teilweise eingelöst.** Ein Hotfix hat die fabrizierten Löschzeugen entfernt: erase\_document liefert `witnesses = Vec::new()` und führt keinen Ledger-Append aus. forget(id) liefert `logically_deleted`. Das ist ehrlich, aber schwach \[G\].
5. **Der Collection-Drop-Pfad ist fehlerhaft \[G\]:** Beweisartefakte in `temp_dir()/contextra_drop_proofs`, `db_instance_id` konstant `[0x33; 16]`, Err per `if let Ok` verworfen, `run_purge` per `let _ =`.
6. **Der physische Purge funktionierte nach der Analyse vom 14.10. nicht** (alte SSTables bleiben liegen, Lesefehler laufen fail-open bis zu stillem Datenverlust, globale Schreibsperre). Am HEAD sind Teile behoben (Abschnitt 2), der Rest ist offen \[T/U\].
7. **Garantie und Nachweis müssen getrennt werden.** Beide hängen heute unbedingt im Kern und kosten jede Installation Leistung.
8. **Reihenfolge: erst Messinstrumente, dann reparieren, dann entkoppeln.**
9. **Die Gates sind weniger belastbar als angenommen.** check-dag kann strukturell nicht fehlschlagen. Der Ring-Gate zeigt in der Emulation 6 Verletzungen, davon eine neu seit 13.10. (engine zu license). orphan-symbols fehlt im Gate-Katalog \[G\]\[E\].
10. **Hülle am HEAD \[E\]:** mcp Default 25 Crates, davon 7 Parkplatz (adapt, core, graph, infer-candle, kvcache, privacy, router). contextra Default 22, engine Default 20 (15 ohne Default-Features). Alle Features: mcp 28, contextra 29.
11. **Der größte Hebel ist das Umschalten der Defaults:** `engine.default = graph, rank, adapt, kvcache, candle` und die router-Anbindung von mcp ziehen den Parkplatz in die Hülle.
12. **Das Produktversprechen ist stärker als der Beleg.** Die Anspruchsstufen C0 bis C3 legen fest, welche Formulierung wann zulässig ist.
13. **Ein Ökosystem aus abschaltbaren Modulen braucht ein Produkt-Manifest.**
14. **Entscheidungen kollidieren mit bestehenden ADRs** (Release-Name, license, core, Crypto-Split, Quellenrang, HNSW-Veto). Die Auflösung steht in Abschnitt 3 und D.5.

**D.2 Kennzahlen am HEAD 5d349c18 (v3 II.2.1)**

| Messgröße | Wert | Quelle |
| --- | --- | --- |
| Crate-Verzeichnisse unter crates/ | 34 (eine frühere Angabe 36 war falsch) | \[G\] |
| Workspace-Mitglieder inkl. benchmarks, xtask, xtask-heavy | 37 | \[G\] |
| Einträge in capabilities.toml | 35 (inkl. xtask-heavy, ohne contextra-bench und xtask) | \[G\] |
| Rust-Zeilen in crates/ (inkl. Tests) | 382.890 gesamt, rund 269.500 in den 18 Kern-Crates, 71.918 in store | \[G\] |
| Rust-Zeilen xtask, xtask-heavy, benchmarks | 74.392 | \[G\] |
| Testattribute / Dateien in tests/ | 5.016 / 979 | \[G\] |
| Commits gesamt / seit 11.10. | 4.224 / 724 | \[G\] |
| Autoren | tfufuz1 3.740 (inkl. Merges von Agenten-PRs), google-labs-jules\[bot\] 460, github-actions\[bot\] 23 | \[G\] |
| ADR-Dateien / Markdown-Dateien unter docs/ | 170 (am Klon 171) / 651 (7,0 MB) | \[G\] |
| Task-Karten / CI-Workflows | 123 (am Klon 124) / 36 | \[G\] |
| Gate-Katalog (`governance/gates.toml`) | 32 Gates, 27 mit required\_ci | \[G\] |
| wiring-Dateien (`wiring*.rs`) in crates/ | 49 | \[G\] |
| Öffentliche Funktionen der Funktionszuordnung | 333, davon 332 am HEAD definiert (`cosine_similarity_parts_f32_u8` nicht gefunden) | \[H\] |
| Kern-Crates / Parkplatz-Crates | 18 / 15 | \[G\] |

Der Doku-Umfang (7,0 MB in 651 Dateien) liegt in der Größenordnung des Codes. Das ist ein Wartungsrisiko, kein Reifezeichen.

**D.3 Begriffe (v3 II.2.2)**

| Begriff | Bedeutung in Contextra |
| --- | --- |
| Tombstone | Löschmarke im LSM-Store. Das Dokument verschwindet logisch sofort, die Bytes erst durch Compaction oder Purge |
| Footprint | Alle Schlüssel und Präfixe eines Dokuments oder einer Collection über alle Layer (Key-Value, Text-Index, Graph, Relationen, Vektor-Index) |
| Layer | Speicherebene: LSM (Memtable, SSTables, WAL), Text-Index, Graph, Vektor-Index (HNSW), KV-Cache, Checkpoints |
| Purge | Physische Bereinigung: SSTables ohne die Zielschlüssel neu schreiben, alte Dateien entsorgen, WAL-Segmente stilllegen |
| Fence | Schreibsperre auf einem Scope, solange ein Purge läuft |
| Erasure Ledger | Dauerhaftes Löschprotokoll gegen Wiederauferstehung (Anti-Resurrection) |
| Zeuge (Witness) | Beobachtung eines Layers mit Scope, Zeitpunkt und Scanner-Kennung |
| Löschnachweis | Signiertes Dokument aus Zeugen, Scope, Ausschlüssen und Zeitpunkt, ein Audit mit definierten Toleranzen |
| Crypto-Erase | Löschen durch Vernichten des Schlüssels |
| Hülle | Anzahl Workspace-Crates im Abhängigkeitsbaum eines Produkts (Wurzel eingeschlossen, ohne Dev- und Build-Kanten) |

**D.4 Drei widersprüchliche Rangordnungen (v3 II.3.1) \[G\]**

| Quelle | Rangfolge |
| --- | --- |
| AGENTS.md | Gates und Code, dann AGENTS.md, dann Spezifikation und Endprodukt-Spezifikationen, dann Masterplan, dann Übriges. Punkt 3 und 5 nennen dasselbe Dokument |
| ADR-E36 | Governance-Dateien, dann die Spezifikation, dann Code (Code zuletzt) |
| Spezifikation v15 | Bei Widerspruch gilt der Code, innerhalb des Dokuments schlägt Teil 0 |
| CONSTITUTION.md | `capabilities.toml` ist alleinige Quelle für Ringe und Reife, WORKING\_STATE.md ist reine Projektion, ADRs nur in `docs/decisions/` |

Agenten können je nach gelesener Quelle Gegensätzliches umsetzen. Die Spezifikation §13 E01 bis E37, aus der alle E-ADRs laut Kopfzeile stammen, ist im Repo nicht auffindbar. Die Auflösung ist DE-19.

**Tote Verweise in geschützten Pfaden \[G\]:** AGENTS.md verweist auf `docs/14-10-26/ARCHITEKTUR_MASTERPLAN_v2.md`, `docs/14-10-26/specs/` und `docs/SPEZIFIKATION_EP*.md`, mit lokalem Pfad `/home/freddy/Projekte/contextra/`. Das Verzeichnis `docs/14-10-26/` fehlt. Die README verweist auf Dokumente, die im docs-Listing fehlen. AGENTS.md liegt in `governance/protected-paths.toml`, nur der Maintainer darf es mit ADR ändern, und es gibt Agenten die Phasen 3 bis 6 als Roadmap vor. Die Audit-Gesamtsynthese vom 14.10. verweist auf einen anderen, ebenfalls nicht vorhandenen Masterplan-Pfad.

**D.5 Abgleich der Entscheidungen mit bestehenden ADRs (v3 II.3.3)**

| Entscheidung | Bestehender ADR (Status) | Befund | Empfehlung |
| --- | --- | --- | --- |
| Legacy-WAL entfernen | ADR-048, 055, 100 (Feature-Gating, Fallback-Schutz) | Richtung gleich, ADR beschreibt Gating, nicht Entfernung | neuer ADR vor v0.1.0 (DE-17) |
| rank im Kern | ADR-E14: rank Pflicht (akzeptiert) | konsistent | übernehmen |
| license im Kern | ADR-E32 (nicht in EP1, LicenseGate ist ein Port, Engine zu Ring 4 verboten), ADR-E29 | **Konflikt** | LicenseGate als Port, Crate nur als Feature der Fassade (DE-18) |
| graph für Hybridsuche nicht nötig | ADR-E14 (engine.default leer) | konsistent | übernehmen |
| EP2 als eigenes Repository | ADR-E01, ADR-E16 | konsistent | Crypto-Split vor v0.1.0 |
| core entfernen | ADR-E15 (akzeptiert) gegen ADR-SCOPE-FREEZE (Vorschlag) | zwei ADRs widersprechen sich | SCOPE-FREEZE-Zeile zu core durch ADR-E15 ersetzen |
| Release | ADR-E01: v0.1.0, 1.0.0 ist ein Gate | konsistent | v0.1.0 |
| Hülle mit Default-Features | ADR-E14 (höchstens 18) | konsistent | übernehmen |
| Unwrap-Baseline | ADR-E34: ratchet.toml ist einzige Quelle | nicht umgesetzt (502 gegen 786) | umsetzen |
| Löschkette | ADR-E09 (Ledger-Append als Commit-Punkt), E21, E35, E37, T03 (Entwurf) | E09 am HEAD nicht umgesetzt | K3 |
| HNSW-Löschung | VETO-F02 (nur Tombstone-Pruning, Review fällig 2027-10-12) gegen ADR-097 (Nachbarschaftsreparatur) | **Widerspruch zu einem Veto**, check-vetoes ist critical | **gelöst durch DE-11** |
| Fence-Protokoll | ADR-E12 (Barriere im Serialisierungspunkt) | konsistent mit Option B | angepasst durch DE-05, per Loom und Benchmark verifizieren |
| Crypto-Erase | ADR-E08 | konsistent | übernehmen |
| Hochwassermarke, Downgrade-Schutz | ADR-N12, ADR-E20 | konsistent | übernehmen |
| Beweisformat v4, v3 verboten | ADR-E05, ADR-E06 | konsistent | übernehmen |

**D.6 ADR-Hygiene \[G\]:** Die Planungsdokumente verwenden ADR-01 bis ADR-18 (Crypto-Split, Crypto-Erase, v3-Verbot, Hochwassermarke), im Repo sind ADR-001 bis 018 andere Entscheidungen (ADR-001 LSM-Tree, ADR-007 Produktstrategie). Zuordnung: ADR-01 Erasure als Zero-Cost-Plugin entspricht E01 und E13. ADR-04 Crypto-Split entspricht E16. ADR-07 Crypto-Erase entspricht E08. ADR-13 v3-Verbot entspricht E06. ADR-15 Hochwassermarke entspricht N12 und E20. Fence Option B entspricht E12. contextra-core entfernen entspricht E15. KvDeleteMode-Default entspricht E11. Ring-Modell entspricht E02 und N01 (Vorschlag, Inhalt nicht Zeile für Zeile verglichen). **Dubletten:** ADR-056 und ADR-059 (Python-FFI-Panic-Isolation), ADR-E03 und ADR-N03 (drei Unsafe-Inseln). **Lücken:** 066 bis 069, 085 bis 095. Platzhalter `ADR-0XX-contextra-sandbox.md`. **Status:** E01 bis E37 sind Akzeptiert (aus Spezifikation §13) ohne auffindbare Quelle, T01 bis T06 Entwurf, SCOPE-FREEZE Vorschlag.

**D.7 Dokumentenlast und Wahrheitsverlust (v3 II.3.5) \[G\]**

| Aussage im Repo | Befund |
| --- | --- |
| Audit-Gesamtsynthese (14.10.): check-dag wurde korrigiert, Exit 2 bei Zyklen \[D\] | `xtask/src/lib.rs`: `let _ = compute_crate_layers` besteht |
| dieselbe Synthese: `check_fbs_drift.exit` ist 0 \[D\] | `logs/audits/unsafe-islands_2026-10-13/check_fbs_drift.exit` ist 1 (ein anderer Lauf zeigt 0) |
| dieselbe Synthese: Unsafe-Inseln vector, store, kvcache \[D\] | capabilities.toml: simd, sys, wire, `unsafe` im Quelltext nur in diesen drei Crates \[H\] |
| dieselbe Synthese: 6 Ring-Verletzungen als Auditergebnis \[D\] | Die Zahl stammt aus einer Emulation (zirkulär) |
| README: 0 unwrap() im Produktionspfad | `ratchet.toml`: unwrap\_exact 502, Baseline-Datei 786 |
| WORKING\_STATE.md: alle Crates stable (Stand 11.10.) | Orphan-Gate fail, Beschreibungsfelder von Hand, nicht aus Gates |
| WORKING\_STATE.md: 14 CRITICAL-Tags mit Text RESOLVED und Status OPEN | Auflösungsvermerke werden als offene Schuld gezählt |

Die Audits tragen Etiketten wie VERIFIED: PASSED von Agenten-Spezialisten. Ein Etikett ohne Reproduktionslog ist keine Verifikation.

### Anhang E: Korrekturen an früheren Annahmen und Widersprüche im Ist-Zustand

**E.1 Korrekturen (v3 II.4)**

| Frühere Aussage | Stand am HEAD laut v3 | Bewertung |
| --- | --- | --- |
| 36 Crates unter crates/ | 34 | falsch \[G\] |
| Löschkette: Footprint, Ledger, Layer-Zeugen laufen | erase\_document: Tombstones und RAM-Index-Bereinigung, witnesses leer, kein ledger\_append | überholt \[G\] |
| Ledger-Zeuge mit entry\_confirmed true und Null-MAC wird fabriziert | durch Hotfix entfernt | behoben \[G\] |
| forget liefert proof null | Einzeldokument logically\_deleted, Tests `contextra_delete_no_unverified_proof.rs`, `delete_proof_contract.rs` | teils erledigt \[G\] |
| `_opts` in erase\_document wird nicht gelesen | `opts.options.validate()`, Assurance größer S0 und FullSweep liefern CapabilityUnsupported. MCP übergibt aber `DocEraseOptions::default()` | teils erledigt \[G\] |
| `DpiDeletionRuntime::open` ohne Produktionsaufrufer | Aufrufer in `engine/deletion/wiring.rs:62`, gebaut in `collections.rs:550` (im fehlerhaften Drop-Pfad) | angeschlossen, aber fehlerhaft \[G\] |
| generate\_kv\_deletion\_proof, open\_durable, scrub\_and\_remove ohne Aufrufer | weiterhin 0 Produktionsaufrufer | besteht \[H\] |
| db zu cognition regulär | nicht mehr vorhanden | erledigt \[G\] |
| Kern zu Parkplatz: 10 regulär, 16 optional | 9 regulär, 19 optional, 3 dev (31 gesamt) | aktualisiert \[E\] |
| contextra: candle Default-aktiv | Fassade default gleich fast, die Hülle zieht candle über den engine-Default | Default-Kette entscheidend \[E\] |
| Hülle mcp rund 26, contextra rund 23 | 25 / 22 (Default), 28 / 29 (alle Features) | ersetzt \[E\] |
| core-Ablösung betrifft 110 Dateien | 405 Dateien referenzieren contextra\_core (ADR-E15 nennt 404), die 110 betrafen vier Crates | unterschätzt \[G\] |
| store zu sandbox (dev) ist Ring-Verletzung | am HEAD nicht mehr vorhanden | behoben \[E\] |
| engine zu license nicht erfasst | regulär seit Commit 9408b9c7 (13.10.), Ring 3 zu Ring 4, entgegen ADR-E32 | neu \[G\]\[E\] |
| Zwei Unwrap-Baselines (502 und 786) | besteht, ADR-E34 sagt, ratchet.toml ist einzige Quelle | nicht umgesetzt \[G\] |
| Neues Gate für Produktgrenzen und neues Kartenformat nötig | scope-guard (required CI) und `.jules/tasks/_TEMPLATE.toml` mit Gate task-card existieren | erweitern statt neu bauen |
| Spezifikation v4 maßgeblich | im Repo gilt Spezifikation v15 (393 KB) | Quelle war falsch benannt |
| Release 1.0 als einziges Ziel | ADR-E01: v0.1.0, 1.0 ist ein Gate | behoben |
| Ring kommt aus capabilities.toml | Ring steht an drei Stellen: capabilities.toml, Cargo.toml-Metadaten, hartkodiert im Gate | unvollständig \[G\] |
| Zwei Kommentar-Versprechen in erase\_document | Docstring beschreibt Ledger-Append und Zeugen, der Code führt beide nicht aus | Doku lügt \[G\] |

Unverändert am HEAD \[G\]: `contextra_cloud_query` bleibt ohne Guard registriert (`server_dispatch.rs:297`, `with_egress_guard` nur definiert). `verify_wiring` hat nur Test-Aufrufer. `governance/security_ports.toml` und `products/` fehlen. engine hat `default = graph, rank, adapt, kvcache, candle`.

**E.2 Widersprüche im Ist-Zustand und ihre Auflösung (v3 II.6)**

| Widerspruch | Auflösung |
| --- | --- |
| Das Flag `deletion_proof_active` wird beim Öffnen auf false gesetzt, von der Engine nie gelesen und von erase\_document ignoriert. Nur die Profil-Validierung der Fassade liest es | Flag entfällt. Ersatz: ErasureMode, der wirklich steuert |
| `PerformanceProfile::default()` ist Compliance, der Builder ohne Profil nimmt FeatureRing::Fast | Ein Default: Balanced. Compliance nur explizit |
| Das Cargo-Feature `deletion-proof` im Store hat keinen cfg(feature)-Treffer | entfernen, echte Features laut Zielarchitektur |
| Ring hat zwei Bedeutungen: FeatureRing (Lizenz) und Architektur-Ring 0 bis 4. contextra-crypto steht in Ring 0, enthält aber Nachweis-Code | Architektur-Ringe in Schicht L0 bis L4 umbenennen, FeatureRing bleibt öffentliche API |
| Kommentare versprechen Ledger-Append vor Tombstone und fail-closed ohne Ledger, der Code tut es nicht. Der Drop-Pfad verspricht nur gescannte Layer, verwirft aber das Purge-Ergebnis | Gate gegen Drift zwischen Doku und Code zuerst (deletion-guarantee-gate), dann Umbau |
| Der Löschnachweis ist das Versprechen des Projekts gegen Löschnachweis nur für Enterprise | Begriffe trennen: Löschgarantie (Versprechen, verfügbar) und Löschnachweis (Evidenz, Plugin) |
| `contextra-crypto/src/lib.rs` nennt sich lock-free und I/O-frei, schreibt aber Dateien und hängt an durable-fs | Die Aussage gilt nur für einen I/O-freien Kern (crypto-core) |
| capabilities.toml führt contextra-crypto mit `unsafe_island` false, die Regeldatei zu Unsafe-Inseln nennt es eine Unsafe-Insel | Beim Split beide gemeinsam anpassen |
| Löschen ohne Nachweis gilt als Nicht-Feature, aber der Schutz vor Resurrection fehlt auch dort, wo er versprochen wird | Garantie-Stufen mit klarer Aussage je Stufe |
| Das Gate verbietet Ring 1 zu Ring 1 und Ring 2 zu Ring 2 (außer dev), die Schichtregel nur abwärts oder auf denselben Ring würde gleichen Ring erlauben | Regel einmal festlegen (DE-16) |
| Workspace-Lizenz MIT oder Apache-2.0 gegen Lizenz-Aktivierung, Installation-Bindung und bezahlten Compliance-Export | DE-18 und Maintainer-Vorbehalt 13.4 |

### Anhang F: Befunde am HEAD (v3 II.5)

Schwere: K kritisch (Integrität des Versprechens), H hoch, M mittel.

**F.1 Stabilität und Prozess**

| Befund | Beleg |
| --- | --- |
| Änderungsrate | 254 Commits am 12.10., 198 am 13.10., 176 am 14.10. \[G\] |
| Commit-Mix seit 11.10. | feat 149, docs 118, fix 39, test 9, Shell-Commit 24 \[G\] |
| Orphan-Gate | fail: 40 auf 36 (Engine-Lauf), 40 auf 37 (Store-Lauf) \[G\] |
| Log-Qualität | 5 von 11 `orphan_symbols.json` beginnen mit Cargo-Ausgabe (Finished) und sind kein JSON \[G\] |
| Test-Log | privacy-license-compliance: cargo test grün \[G\], weitere Testläufe liegen nicht vor |
| Scheinverdrahtung | 49 wiring-Dateien, 194 öffentliche Symbole in wiring.rs-Dateien, 31 ohne Verweis aus einer anderen Produktionsdatei (u. a. CheckpointWiring, CompactionEngineWiring, issue\_proof, execute\_cascade) \[H\] |
| Evidenz-Logs | 57 von 58 referenzierten Evidenz-Logs früherer Audit-Läufe fehlen im Repository \[D\] |

**F.2 Sicherheits- und Korrektheitsbefunde**

| Befund | Stand | Quelle |
| --- | --- | --- |
| Cloud-Egress ohne Guard: `contextra_cloud_query` registriert, `with_egress_guard` nur definiert | besteht | \[G\] |
| Selbsttest `verify_wiring(bool, bool, bool)` ohne Produktionsaufrufer | besteht | \[G\] |
| `governance/security_ports.toml` fehlt | besteht | \[G\] |
| MCP-Werkzeuge aus dem Parkplatz: cloud\_query, consolidate, relate | besteht | \[G\] |
| Zwei Wege für remember und recall | nicht neu geprüft | \[U\] |
| drop\_collection: Beweisverzeichnis `temp_dir()/contextra_drop_proofs`, `db_instance_id` konstant `[0x33; 16]`, `signer_cfg_path` None, `if let Ok(deletion_wiring)` ohne Else, `let _ = driver.run_purge`. Mit None greift `proof_issue.rs` auf `~/.contextra/proof_ed25519.key` zurück: ein Signaturschlüssel je Unix-Nutzer für alle Datenbanken | Purge-Fehler verschwinden still, Instanz-ID nicht eindeutig, Artefakte in /tmp (`collections.rs:536` bis `566`) | \[G\] |
| engine zu license regulär (Ring 3 zu Ring 4) | verletzt ADR-E32 und den Ring-Gate | \[G\]\[E\] |
| erase\_document: `downcast_ref` auf LsmStorage (`delete.rs:91`), bei anderem Storage wird `scope_write_seq_before` gleich 0 gesetzt, ohne Fehler | Fence-Prüfung kann bei Mandanten-Storage wirkungslos sein, Wirkung braucht Test mit TenantScopedStorage | Code \[G\], Wirkung \[U\] |
| MCP forget nutzt `DocEraseOptions::default()` | Request-Optionen wirken nicht (`server_tools.rs:430`) | \[G\] |
| Sieben Builder-Optionen gespeichert, aber ignoriert | `let _ = adapt_enabled` im HEAD-builder.rs nicht mehr gefunden, Wirksamkeit je Option ungeprüft | \[D\]\[H\] |
| KmsProvider und LocalKmsProvider in keinem Composition Root verdrahtet | nicht erneut geprüft | \[D\] |
| MCP-Server bindet `_llm_generator` und `_deletion_key` und verwirft beide | nicht erneut geprüft | \[U\] |

**F.3 Die Löschkette am HEAD (laut v3)**

```
contextra_forget (MCP, server_tools.rs)
  id gesetzt: col.erase_document(id, DocEraseOptions::default())
    opts.validate(); Assurance größer S0: CapabilityUnsupported           [G]
    FullSweep: CapabilityUnsupported                                      [G]
    Footprint berechnen                                                   [G]
    Tombstones in einer Transaktion                                       [G]
    RAM-Indizes (HNSW, Text, Relationen, KV-Hooks) bereinigen             [G]
    Ledger-Append: NICHT vorhanden (ledger_append ohne Aufrufer)          [G]
    Zeugen: Vec::new() (keine Fabrikation)                                [G]
    Antwort: status logically_deleted, tx                                 [G]
  id nicht gesetzt: drop_collection(proof_key)
    DeletionWiring::build (Beweisordner in /tmp, Instanz-ID konstant)     [G]
      if let Ok(..) { let _ = run_purge(..) }  Fehler verworfen           [G]
```

Ohne Produktionsaufrufer \[H\]: `generate_kv_deletion_proof`, read/write/delete\_kv\_segment, `KeyRegistry::open_durable`, `scrub_and_remove`, `ledger_append`, `DpiPurgeDriver::run_physical`, `resume_pending`, der Restore-Filter der Engine. Der Einzeldokument-Pfad ist ehrlich, aber schwach (logische Löschung). Der Collection-Pfad erzeugt Beweisartefakte, deren Entstehung nicht belastbar ist. Bis die Löschphase abgeschlossen ist, ist „beweisbar“ keine zulässige Aussage.

**F.4 Integrität des Löschpfads und des Purge (Analyse vom 14.10., Stand 4bfc29c)**

Beim Gegenlesen am Klon sind einige Befunde teilweise behoben (Abschnitt 2 und 6.1). Alles andere ist als offen zu führen.

| Sev | Befund | Fundstelle | Status |
| --- | --- | --- | --- |
| K | **Fabrizierter Ledger-Zeuge** (entry\_confirmed true, MAC aus Nullen, kein Ledger-Eintrag) | `engine/collection/crud/delete.rs` | **behoben** durch Hotfix |
| K | **Zeugen-Fehler verschluckt** (`if let Ok` bei LSM-, Text-, Relations-Zeuge) | delete.rs | gegenstandslos, solange keine Zeugen erzeugt werden, bei Wiedereinführung zwingend fail-closed |
| H | **Fehler nach wirksamer Löschung:** Die ConcurrentModification-Prüfung läuft nach `db_tx.commit()`, der Aufrufer bekommt Err, obwohl die Löschung durchgeführt ist | delete.rs | nicht neu geprüft, Regel in 5.2 |
| K | **Keine physische Entfernung.** purge\_keys schreibt SSTables ohne die Zielschlüssel neu und hängt einen Replace-Eintrag ans Manifest, entfernt die alte Datei aber nicht und liefert keine Liste obsoleter Dateien. Der Orchestrator ruft `dispose_obsolete_files` mit leerer Liste auf. Ein Produktions-Orphan-Remover existiert nicht | `lsm/ops/purge.rs`, `lsm/purge/orchestrator.rs` | am HEAD teilweise behoben, Wirkung unbelegt |
| K | **Fail-open im Scan.** `while let Ok(Some(..)) = stream.next_entry().await` behandelt einen Lesefehler wie das Dateiende. Treffersuche: die SSTable gilt als ohne Treffer. Rewrite: die restlichen Einträge fehlen, die SSTable wird durch eine gekürzte ersetzt (stiller Datenverlust). Dasselbe Muster in `wal/retire.rs` | purge.rs, retire.rs | purge.rs behoben, **retire.rs:39 offen** |
| H | **Reihenfolge nach dem Rewrite.** Der Lesepfad iteriert `sstables.iter().rev()`, der erste Treffer gewinnt. purge\_keys hängt neue Reader am Ende an und schreibt rank 0. Eine umgeschriebene ältere SSTable wird zur neuesten und kann neuere Werte jüngerer SSTables verdecken | `lsm/ops/read.rs`, purge.rs | \[T\], Differenztest I2 |
| H | **Globale Fence und O(Gesamtdaten).** Der Orchestrator setzt CollectionDropping für den ganzen LSM: Alle Schreibvorgänge aller Mandanten liefern Err. Präfix-Fence und `lock_documents_guarded` haben keinen Produktionsaufrufer. purge\_keys liest pro Aufruf jede SSTable. Das batch\_window im Driver schläft nur | `purge/orchestrator.rs`, `lsm/fence.rs`, `deletion/purge_driver.rs` | nicht neu geprüft |
| H | **Purge im Drop-Pfad immer blockiert, Ergebnis verschwindet.** run\_purge mit space\_probe None: `free_space = unwrap_or(0)` gegen mindestens 1 MiB, Ergebnis InsufficientDiskSpace, Aufruf endet in `let _ =`. FreeSpaceProbe hat nur Test-Implementierungen | collections.rs, purge.rs | Verwerfen am HEAD bestätigt |
| H | **Fence-Zähler belegen nichts.** `check_write_allowed_scope` erhöht write\_seq nur, wenn ein Schreibvorgang abgewiesen wird. erase\_document hält weder Präfix-Fence noch Doc-Sperre. Einziger Schutz ist `kv_locks.lock_for(id)` je Schlüssel | fence.rs, delete.rs | nicht neu geprüft, ersetzt durch 6.3 |
| M | **Purge gegen Compaction.** Ein Ausschluss wurde nicht gefunden | purge.rs | \[T\], ersetzt durch CompactionGate-Lease |
| M | **Irreführende Fehlercodes:** `map_err` auf CompactionBusy steht für beliebige I/O-Fehler | purge.rs | 11 Treffer am HEAD \[G\] |
| H | **Zwei Drop-Pfade mit verschiedener Garantie.** `Collection::drop_collection` löscht per Präfix, ignoriert Fehler des Vektor-Deletes und purgt nicht. `Contextra::drop_collection` mit Variante arbeitet mit Nachweis | `collection/maintenance.rs`, `contextra_impl/collections.rs` | \[U\] |
| H | **Beweis ohne Scan.** Im Drop-Pfad wurde die Ebene SsTableAllLevels nach erfolgreichem purge\_keys ohne Scan belegt und ein Beweis v3 mit leerem Zeugen-Vektor erzeugt. `drop_collection_with_report` liefert ohne Feature encryption-at-rest CapabilityUnsupported. Die Doku behauptet das Gegenteil | collections.rs | \[U\] |
| H | **Wiederaufnahme und Anti-Resurrection nicht wirksam.** resume\_pending hat keinen belegten Aufrufer im Öffnungspfad, `lsm/recovery.rs` kennt nur Rollback-Intents. Ledger und Restore-Filter haben keinen Produktionsaufrufer | `lsm/purge/resume.rs`, `erasure_ledger/mod.rs`, `deletion/restore_filter.rs` | für ledger\_append und resume\_pending bestätigt |

Sobald der Tombstone committed ist, darf es für die Löschung kein Err mehr geben. Ein fehlgeschlagener Zeuge oder Scan ist ein dauerhafter, abfragbarer Zustand.

**F.5 Qualität des Nachweises**

| Sev | Befund | Status |
| --- | --- | --- |
| H | **Nicht-strikte Signaturprüfung.** `verify` statt `verify_strict` in `audit_chain.rs`, `deletion_proof/verify.rs`, `deletion_proof_typestate.rs` | am Klon überholt: verify\_strict an sieben Stellen, Vollständigkeit prüfen |
| H | **bincode im Signatur- und Commitment-Pfad** (`audit_chain.rs`, `kv_segment/segment.rs`) | bestätigt (`audit_chain.rs:85`), auf TLV umstellen |
| H | **Domain-Tags vermischt.** Beweisformat v3 benutzt den Domain-Tag von v4 | \[U\] |
| H | **Signierschlüssel.** Die Ableitung aus der öffentlichen db\_instance\_id ist entfernt (legacy\_test\_only). Der Schlüssel aus der Umgebung wird nur auf Länge mindestens 32 geprüft. Der Drop-Pfad nutzt einen Nutzerschlüssel für alle Datenbanken | teilweise behoben |
| H | **Policy.** Die Default-Policy akzeptiert fast alles, keine Pflicht-Layer-Menge | \[U\], siehe 7.1 |
| H | **Zeugen sind fälschbar.** `LayerWitness::issue` und `WitIssueArgs` sind pub, die Engine setzt Zähler selbst. Der Verifier prüft weder Digest noch Layer-Claim | entschärft, solange keine Zeugen erzeugt werden, Pull-Modell DE-22 |
| H | **LSM-Zeuge mit festen Werten** (manifest\_generation 1, in\_flight\_commits 0, unpersisted\_edits 0) | \[U\] |
| H | **KMS-Widerruf ohne Fehlerbehandlung:** `let _ = self.registry.revoke_group(..)` | \[U\] |
| H | **WAL-Scan mit Lücken.** Lesefehler beenden die Schleife stumm, Frames nach dem Stichtag zählen nicht | \[U\], gleiches Muster im Purge bestätigt |
| K | **Checkpoint-Restore bringt Gelöschtes zurück.** restore\_checkpoint ruft nur rollback\_to\_tx, Löschprotokoll und Restore-Filter greifen nicht | \[U\], siehe 6.6 |
| H | **Dispose fällt auf normales Löschen zurück.** Verweigert das Überschreiben (Hardlinks, Copy-on-Write), wird die Datei trotzdem entfernt und als gelöscht gezählt | \[U\], siehe 6.7 |
| M | **Residue-Zeuge liefert Ok(None)**, wenn der Zeuge fehlt | \[U\] |
| M | **HNSW-Zeroisierung nur mit fill(0).** Der Compiler darf solche Schreibvorgänge entfernen, Kopien und Mmap-Seiten bleiben unberührt | \[U\] |
| M | **Speicherschutz meldet mehr, als er garantiert** (mlock, harden\_runtime) | \[U\] |
| M | **Kein externer Zeitanker, kein Transparenzlog.** Beweise lassen sich zurückdatieren, die Audit-Kette zurückspulen | Design-Lücke, siehe 7.4 |
| M | **Kopien ohne Statusmodell.** Checkpoints, Hardlinks und Backups erscheinen als Zähler | Design-Lücke, siehe 6.7 |
| M | **Teilweiser AEAD-Frame am WAL-Ende** wurde beim Replay nicht abgeschnitten | offen |
| M | **Python-Doku weicht vom Verhalten ab** (Löschsemantik der Bindings) | \[U\] |

**F.6 Kopplung und Leistung**

| Sev | Befund | Status |
| --- | --- | --- |
| H | store, vector, engine, kvcache und db hängen zwingend an contextra-crypto. vector nutzt fast nur WalHmac, trägt aber den gesamten Nachweis-Code mit. Rund 46 Prozent der Zeilen von contextra-crypto (6.142 von 13.492) sind Nachweis-Code | bestätigt (14.10.) |
| H | Der Store kennt das Beweisformat: rund elf Importe aus deletion\_proof | \[U\] |
| H | Es gab keinen Port für Löschung, Zeugen oder Ledger. Die Engine importiert rund 30 konkrete Crypto-Typen | **überholt für die Ports**: `ports::erasure` existiert (Abschnitt 2), die Engine-Importe bleiben |
| H | Die Engine castet per dyn Any auf LsmStorage und bricht den StorageEngine-Port. Andere Backends scheitern bei Assurance ab S1 fail-closed | am HEAD bestätigt, fünf Stellen |
| H | **Hot Path.** `ErasureFence::check_write_allowed_scope` läuft in jedem put() und für jeden Schlüssel im Commit (sechs Stellen in `ops/write.rs`). Drei parking\_lot-Read-Locks plus HashSet-Lookup, auch wenn nie eine Löschung läuft. Die Fence ist ein pub-Feld von LsmStorage. Kosten abgeleitet, nicht gemessen | bestätigt (acht Aufrufstellen am Klon) |
| M | erase\_document berechnet den Footprint immer, auch ohne Nachweiswunsch | bestätigt |
| M | engine hat contextra-license als Pflicht-Dependency (nur zwei Zeilen im Öffnungspfad) und candle als Default-Feature. cognition hängt an engine. mcp hat crypto und privacy als Pflicht. py umgeht die Fassade | teils \[U\] |
| M | Ungenutzte Abhängigkeiten: db zu crypto (nur Tests), mcp zu license, mcp zu agent, py zu router, rank, adapt, store, infer-onnx zu infer-candle | \[U\] |
| M | contextra-crypto enthält Datei-I/O (Signer, Proof-Store, Revocation-Log), obwohl der Kopfkommentar I/O-frei behauptet. Die Fassaden-Features sovereign, compliance, audit-export, avv-generator schalten keinen Code | \[U\] |
| M | mcp zieht router über contextra mit Features router und encryption-at-rest zwangsweise ein, router zieht graph, text, adapt | \[E\] |

**F.7 Verdrahtungsstand der 333 Funktionen**

Methode: Suche nach `name(`-Mustern in Nicht-Testcode. Die strikte Variante schließt zusätzlich Dateien mit Test im Namen, Testkit und alles nach dem ersten `cfg(test)` je Datei aus. Das Ergebnis ist eine Heuristik \[H\].

| Endprodukt | Funktionen | früherer Stand (Treffer) | Methode alt am HEAD | strikt am HEAD |
| --- | --: | --: | --: | --: |
| EP1 Memory | 165 | 13 | 26 | 19 |
| EP2 Proof | 16 | 0 | 3 | 3 |
| EP3 Retrieval+ | 42 | 4 | 9 | 8 |
| EP4 Inference | 21 | 2 | 2 | 2 |
| EP5 Cognition | 59 | 3 | 3 | 3 |
| EP6 Guard | 15 | 1 | 3 | 3 |
| LAB | 15 | 1 | 3 | 2 |
| **Summe** | **333** | **24** | **49** | **40** |

17 Funktionen sind strikt neu verdrahtet, 9 neue Treffer sind testnah, 1 Treffer ging verloren, 1 Definition fehlt.

**EP1, Aktion verdrahten (111 Funktionen):** 96 ohne strikten Produktionsaufruf (früher 101). Nur 3 haben einen Aufrufer außerhalb des eigenen Crates. Neun haben Treffer ausschließlich in wiring-Dateien (Scheinverdrahtung prüfen).

| Crate | EP1-V | ohne strikten Treffer |
| --- | --: | --: |
| contextra-engine | 29 | 27 |
| contextra-crypto | 15 | 12 |
| contextra-store | 15 | 9 |
| contextra-vector | 14 | 14 |
| contextra-mvcc | 12 | 8 |
| contextra-types | 8 | 8 |
| contextra-checkpoint | 5 | 5 |
| contextra-text | 4 | 4 |
| contextra-wire, contextra | je 3 | je 3 |
| contextra-mcp | 2 | 2 |
| contextra-durable-fs | 1 | 1 |

Zuordnung der Aktionen: V verdrahten, A anpassen, P parken, T Test oder intern, L Lizenz, G Gate, U unklar. EP1: V 111, A 10, P 9, T 21, L 8, G 6. EP2: P 13, U 3. EP3: P 40, L 2. EP4: P 21. EP5: P 59. EP6: P 15. LAB: T 15. Die 9 rank-Funktionen gehören zum Kern, bleiben aber geparkt. Die Lizenzfunktionen gehören zur Aktivierung (EP2). Mögliche Namenskollisionen: `digest` (Treffer im Signer, Funktion aus sandbox), `set_permissions` (unklar). `root_as_search_response_with_opts` hat nur einen Treffer im Fuzz-Target und zählt faktisch nicht. Die regelbasierte Meilenstein-Zuordnung der Funktionen (K3: 13, K1 und K2a: 10, K4: 88 für EP1-V) ist ein Vorschlag nach Pfad und Namen und muss je Karte bestätigt werden.

Ohne Produktionsaufrufer laut früherer Prüfung \[U\]: `shred_segment_key`, `guard_and_sanitize_payload`, Cross-Encoder-Reranker, Fachvokabulare für Recht und Medizin, `remove_orphans`, `collect_witnesses`, `select_purge_candidates`, `retire_segments_up_to`. Der Suchpfad rerankt nicht. Compliance-Exporter haben keinen Produktionsaufrufer. Die Konsolidierung läuft in Produktion nie (`consolidation_launcher` wird gelesen, aber nirgends gesetzt). `LsmStorage::open_with_merge_operator` und der WASM-Merge-Operator haben keinen Aufrufer. 15 offene `TODO(wiring)`-Marker.

**F.8 Qualität, Governance und Tests**

Seit früheren Prüfungen am Code bestätigt behoben: Der Standard-Builder setzt kein OpenFastGate mehr. Im Store gibt es kein thread\_rng mehr. Das Journal verschluckt den Flush-Fehler nicht mehr. Der Commit-Pfad ruft die SSI-Validierung auf. `secure_buf` ist in lib.rs deklariert. MockEmbedder hängt hinter `cfg(any(test, feature test-utils))`. xtask-heavy steht in capabilities.toml. Die Python-Skripte im Repo-Root sind entfernt. `AgentContext.status` ist nicht mehr pub. `wiring_selftest` ist in lib.rs deklariert. Ein fest kodierter Mandant im Commit-Pfad ist nicht mehr zu finden.

| Sev | Befund | Status |
| --- | --- | --- |
| H | Lesen mit `u64::MAX` als Zeitpunkt umgeht die Snapshot-Isolation (`collection/crud/read.rs`, `crud/links.rs`). Weitere Stellen in Schreibpfaden nicht nachgeprüft | bestätigt (14.10.) |
| H | Der Zeitfilter vermischt Unix-Zeit und TxId (`temporal_filter.rs`) | \[U\], DE-24 |
| H | TxId und TenantId sind mit öffentlichem Feld konstruierbar, TxId hat keinen geprüften Konstruktor. debug\_assert prüft nur im Debug-Build | \[U\] |
| M | contextra-core hängt von Crates ab, die capabilities.toml nicht erlaubt. contextra-adapt und die Compaction-Ratenbegrenzung rufen `Instant::now()` und `SystemTime::now()` direkt auf | \[U\] |
| M | WASM-Merge-Operatoren haben keine CRDT-Property-Tests und kein Freischaltungs-Gate, Module werden bei jedem merge() neu kompiliert | \[U\] |
| M | check\_fbs\_drift endet mit Exit 1 in einem Lauf, Ursache undokumentiert | \[G\] |
| M | Zwei widersprüchliche Unwrap-Baselines (ratchet.toml 502, `.github/unwrap_baseline.txt` 786, am Klon eine Zeile), weitere Ratchet-Baselines überschritten | \[G\] |
| M | Testtargets, die auf nicht eingebundene Module zeigen, können den cargo-test-Lauf abbrechen. Der aktuelle Stand ist nicht gebaut | nicht geprüft |
| M | BM25-Parameter nicht auf dem Projektkorpus gemessen. `validate_candidates_with_llm` liefert feste Mock-Kandidaten | \[U\] |
| M | Python fällt auf TenantId::SYSTEM zurück, Stand nach dem letzten Merge nicht nachgeprüft | offen |

### Anhang G: Produkte, Ringe und Gate-Ist (v3 II.7)

**G.1 Endprodukte mit Crates**

| ID | Endprodukt | Versprechen | Zeitpunkt | Crates |
| --- | --- | --- | --- | --- |
| **EP1** | **Contextra Memory** | Lokaler Speicher für Agenten: Dokumente, Hybridsuche (Vektor, BM25, RRF), Mandanten, logische Löschung, optional Assured-Löschung. Bibliothek und MCP-Server | v0.1.0 nach K4; 1.0 ist ein Gate | types, ports, sys, wire, simd, durable-fs, crypto (Kern, ohne Beweis- und Verifier-Teil), mvcc, text, rank, store, vector, checkpoint, engine, db, contextra, mcp; license nur als Port in ports |
| **EP2** | **Contextra Proof** | Beweisausstellung (Attested), unabhängiger Verifier, Audit- und DSGVO-Export. Eigener Verifier, möglichst in einer zweiten Sprache | nach K4, eigenes Repository | audit-export, avv-generator, Beweis-Teile aus crypto, license-Aktivierung |
| EP3 | Contextra Retrieval+ | Graph, PathRAG, Domänenvokabulare | Jahr 2 | graph |
| EP4 | Contextra Inference | Embedder, Reranker, Generatoren, KV-Cache | Jahr 3, mit Break-even-Gate | infer-candle, infer-ollama, infer-onnx (Experiment), kvcache |
| EP5 | Contextra Cognition | Konsolidierung, Vergessensdynamik, adaptive Parameter, Routing, Agentenlauf | experimentell, keine Zusage | cognition, adapt, router, agent |
| EP6 | Contextra Guard | Kontrollierter Datenaustritt, Surrogat-Vault, WASM-Plugins mit Fuel-Begrenzung | Jahr 3 oder nach Bedarf | privacy, sandbox |
| LAB | Interne Werkzeuge | Testkit, Benchmarks, Gates, Fault-Injection. Wird nie ausgeliefert | laufend | testkit, benchmarks, xtask, xtask-heavy |

Löschstufen in EP1: Off und Assured im Kern-Release (Attested als optionales Plugin-Crate außerhalb der Default-Hülle, Verifier und Compliance-Export in EP2).

**Crypto-Split:** Vor v0.1.0 ohne Shim, weil keine externen Konsumenten existieren. Der Split folgt der Beweisgrenze: Schlüsselableitung, KV-Verschlüsselung und WAL-HMAC bleiben im EP1-Crate; Beweis-Typen, Signer und Verifier gehen nach EP2. Die Hüllenzahl steigt nicht, solange der Beweisteil hinter einem Feature im Kern-Crate bleibt, bis EP2 existiert. Der Split folgt nach der Löschphase. (Hinweis v4: Schlüsselableitung meint hier nur Ableitung für Zwecke außerhalb der KEK/DEK-Hierarchie; KEK/DEK sind zufällig und gewickelt, siehe Abschnitt Schlüssel.)

**Python-Bindings** gehören zu EP1, werden aber nach K4 gebaut, dünn, ohne eigene Logik und nur über die Fassade. Die Workflows build-wheels.yml und publish-pypi.yml existieren; ihr Aktivierungsstatus ist unbekannt. Veröffentlichung bis K4 abschalten.

**G.2 Produkt-Manifest (Entwurf für EP1)**

```toml
[product]
id = "EP1"
name = "Contextra Memory"
status = "v0.1.0"

[crates]
allowed = [
  "contextra-types", "contextra-ports", "contextra-sys", "contextra-wire",
  "contextra-simd", "contextra-durable-fs", "contextra-crypto", "contextra-mvcc",
  "contextra-text", "contextra-rank", "contextra-store", "contextra-vector",
  "contextra-checkpoint", "contextra-engine", "contextra-db",
  "contextra", "contextra-mcp",
]

[features]
build = ["fast"]
```

Das Gate check-product-hull --product EP1 führt `cargo tree -p contextra-mcp -e normal --no-default-features --features <Build-Features>` aus und schlägt fehl, wenn ein Crate außerhalb von allowed auftaucht. Der Lauf mit allen Features ist nur ein Bericht. Ein Modul ist abschaltbar, wenn es hinter einem Feature liegt, das Produkt-Manifest es nicht listet und der Build ohne das Feature besteht (Abschaltprobe).

Der kleinste wirksame Schnitt ist das Entfernen von router aus den contextra-Features von mcp zusammen mit engine.default gleich leer. Die Hülle wird standardmäßig mit Default-Features gemessen (Annahme). Die Emulation löst dep:, crate/feature, crate?/feature, implizite Optional-Features, default-features und Workspace-Vererbung auf. Sie bildet nicht ab: \[patch\], die cfg-Auswertung von Zielabhängigkeiten (alle werden gezählt), Resolver-Sonderfälle. Der Wert ist mit cargo tree zu bestätigen.

**G.3 Ring-Angabe an drei Stellen**

Die Ring-Angabe steht in capabilities.toml, in Cargo.toml (package.metadata.contextra.ring, vom Gate gelesen; beide stimmen für 34 von 34 Crates überein) und hartkodiert im Gate (ring0\_crate\_order, ring3\_crate\_order, Allowlist). Der dritte Ort ist eine stille Kopie und muss zusammengeführt werden.

**G.4 Ring-Regeln des Gates (Ist-Logik)**

- Ring 0 darf nur auf Ring 0 nach einer hartkodierten Ordnung (ring0\_crate\_order) zeigen. Crates ohne Eintrag bekommen Ordnung 99. Jede Abhängigkeit eines gelisteten Crates auf ein ungelistetes Crate ist deshalb eine Verletzung.
- Ring 1 darf auf Ring 0 zeigen, nicht auf Ring 1.
- Ring 2 darf auf Ring 0 nur types, ports, crypto, core, simd, rank zeigen.
- Ring 3 darf auf Ring 0 und Ring 1 zeigen, nicht auf konkrete Ring-2-Crates und nicht auf Ring 4. Innerhalb Ring 3 gilt ring3\_crate\_order.
- Ring 4 darf auf alles zeigen.
- Dev-Abhängigkeiten auf Tooling sind immer erlaubt, ebenso Dev-Abhängigkeiten innerhalb Ring 0 und Ring 3. Alle anderen werden geprüft.
- Stale Allowlist-Einträge wertet der Gate-Code (und --strict) als Verstoß.

Das Gate wird Pflicht, sobald die Ring-0-Ordnung ergänzt und die Kante engine zu license entfernt ist. Die Ring-Ordnung wird nur noch aus capabilities.toml abgeleitet. Die direkte Ring-0-Kante crypto zu durable-fs wird akzeptiert und durable-fs in die Ordnung aufgenommen.

**G.5 Was check-dag tatsächlich prüft**

Layer gleich max(Abhängigkeiten) plus 1, dann Vergleich mit sich selbst. Bei Zyklen liefert die Berechnung einen Fehler, der mit `let _ =` verworfen wird, alle Layer bleiben 0. Das Gate kann strukturell nicht rot werden \[G\].

**G.6 Emulation des Ring-Gates am HEAD \[E\]**

| Von | Nach | Art | Regel | 13.10. | 15.10. |
| --- | --- | --- | --- | --- | --- |
| vector | durable-fs | normal | Ring-0-Ordnung | verletzt | verletzt |
| crypto | durable-fs | normal | Ring-0-Ordnung | verletzt | verletzt |
| engine | sandbox | normal, optional | Ring 3 zu Ring 2 | verletzt | verletzt |
| engine | infer-candle | normal, optional | Ring 3 zu Ring 2 | verletzt | verletzt |
| cognition | infer-candle | dev | Ring 3 zu Ring 2 | verletzt | verletzt |
| engine | license | normal | Ring 3 zu Ring 4 | nicht erfasst | **neu verletzt** |
| store | sandbox | dev | Ring 1 zu Ring 2 | verletzt | **weg** |
| graph | store | dev | Ring 0 zu Ring 1 | allowlistet | weg; Allowlist **stale** |
| infer-onnx | infer-candle | normal, optional | Ring 2 zu Ring 2 | allowlistet | allowlistet |
| infer-ollama | infer-onnx | dev | Ring 2 zu Ring 2 | allowlistet | allowlistet |

Die Allowlist-Einträge laufen am 31.12.2026 ab. merge-gate.yml führt den Job Ring Layering Check mit --strict aus (Zeile 280/281). Liefe die CI so, wie die Emulation sagt, wäre der Job rot. Dass Pull Requests trotzdem gemergt werden, hat drei mögliche Gründe: Job nicht required, Emulation ungenau, oder das Gate läuft ohne alle Kanten. **Zu prüfen: Actions-Tab des letzten main-Laufs und Branch-Protection.** Wenn der Gate tatsächlich rot ist, hat main ein Problem, das vor allen anderen Arbeiten steht.

**G.7 Kanten vom Kern in den Parkplatz \[E\]**

Regulär (9): contextra zu core, checkpoint zu core, db zu adapt, db zu graph, mcp zu adapt, mcp zu privacy, simd zu core, store zu core, vector zu core. Dazu 19 optionale und 3 Dev-Kanten (u. a. engine zu graph, adapt, kvcache, candle, sandbox; contextra zu agent, cognition, router; mcp zu agent, candle, ollama, onnx). Die fünf Kanten auf contextra-core verschwinden mit dessen Entfernung. Die übrigen vier regulären Kanten (db auf adapt und graph, mcp auf adapt und privacy) brauchen Ports oder Feature-Gates.

**G.8 Umfang der Nutzungsstellen \[G, Stand 13.10.\]**

| Kante | Dateien | Zeilen | Mit cfg(feature) |
| --- | --: | --: | --: |
| engine zu graph | 19 | 90 | 9 |
| engine zu adapt | 6 | 8 | 3 |
| engine zu kvcache | 3 | 4 | 2 |
| engine zu infer-candle | 3 | 4 | 2 |
| engine zu sandbox | 1 | 2 | 0 |
| db zu cognition | 1 | 12 | 1 (inzwischen entfernt) |
| db zu graph | 2 | 2 | 1 |
| db zu adapt | 3 | 6 | 1 |
| mcp zu privacy | 3 | 6 | 0 |
| store, vector, simd, checkpoint zu core | 110 | rund 300 | 22 |
| gesamter Workspace zu contextra\_core | 405 | nicht erhoben | nicht erhoben |

engine zu graph ist mit 90 Zeilen der aufwendigste Teil beim Umschalten der Defaults. Die 10 Dateien ohne cfg-Bezug müssen einzeln geprüft werden. Die Spalte zählt Dateien mit irgendeinem cfg(feature), nicht die betroffene Stelle.

**G.9 Produktgrenzen-Gate**

Der vorhandene scope-guard wird erweitert: Ein PR darf nur Pfade ändern, die zum Produkt aus der Karte gehören. Das Gate vergleicht die geänderten Dateien mit dem Produkt-Manifest. Es ist die technische Form der Regel Parkplatz wird nicht angefasst.

### Anhang H: Zielarchitektur der Löschung, Alternativen und Kosten (v3 II.8, II.9)

Dieser Anhang bewahrt die Herleitung. Wo er von der normativen Fassung (Abschnitte 4 bis 7) abweicht, gilt die normative Fassung; die Abweichungen sind als DE-Entscheidungen im Register geführt.

**H.1 Crate-Schnitt im Detail**

| Crate | Rolle | Änderung |
| --- | --- | --- |
| contextra-ports | Verträge | **Neu:** Modul erasure mit ErasureMode, ErasureHooks, LayerScan, LayerObservation, ErasureOutcome, WriteGate. StorageEngine::erasure\_backend() ersetzt den dyn-Any-Downcast. Dazu der Port LicenseGate |
| contextra-crypto-core (aus contextra-crypto) | I/O-freier Kern | crypto (KeyManager), kdf, keystore, kms, wal\_crypto, secure\_buf, anti\_tamper, wal\_completeness. Ohne durable-fs. Einziger Teil, den store, vector, engine, db und kvcache brauchen |
| contextra-crypto-kv (Feature kv-encryption) | Crypto-Erase | kv\_cipher, kv\_segment, kv\_shredding, revocation\_log. Braucht durable-fs |
| **contextra-erasure** (neu) | Plugin Assured | Engine-Teile aus deletion/: purge\_driver, registry, footprint, restore\_filter, reverse\_sweep, copy\_observer, Koordinator. Die Store-Teile (lsm/purge, erasure\_ledger, Fence) bleiben im Store hinter dem Feature erasure (Standard aus), weil sie LsmStorage intern brauchen |
| **contextra-erasure-evidence** (neu) | Plugin Attested | deletion\_proof, deletion\_proof\_typestate, deletion\_witness, ed25519\_proof, audit\_chain, Signer, Proof-Store, Policy, Zeitanker, Transparenzlog |
| **contextra-proof-verify** (neu, klein) | Externer Verifier | Nur Verifikation (verify\_strict, TLV, Testvektoren), ohne Engine, Store und Netz, ohne Lizenz-Gate |
| store, vector, text, graph | Layer-Eigentümer | Implementieren LayerScan, liefern neutrale LayerObservation. Scan-Code hinter dem Feature erasure-scan (Standard aus) |
| contextra-engine | Koordinator | Hält Option von Arc dyn ErasureHooks. Im Modus Off ruft delete direkt storage.delete, ohne Footprint und Zeugen. Kein contextra-license mehr |
| contextra-audit-export, contextra-avv-generator | Compliance | Hängen am Evidence-Plugin statt am Kern-Crypto |
| contextra (Fassade) | Composition Root | Features und Builder |

**Schlüsselverwahrung:** Crypto-Erase ist ein Garantie-Mechanismus und bleibt außerhalb des Nachweis-Plugins. Das RevocationLog bleibt in crypto-kv, weil seine Rollback-Erkennung für die Korrektheit gebraucht wird. Das Plugin liest es nur.

**H.2 Konfiguration und Aktivierung**

- **Compile-Zeit (Fassade):** erasure aktiviert contextra-erasure, store/erasure und engine/erasure; erasure-evidence setzt erasure voraus und aktiviert contextra-erasure-evidence; sovereign setzt privacy und erasure-evidence voraus; compliance setzt sovereign, audit-export und avv-generator voraus. Ohne diese Features ist nichts davon im Binary.
- **Laufzeit:** ErasureMode (Off, Assured, Attested) in ContextraConfig, Standard Off. Nur bei Assured und Attested legt open das Subsystem an (Ledger, Fence, Purge-Resume). Bei Off entsteht keine Datei und kein Task.
- **Kein stilles Ignorieren:** Findet open im Modus Off eine Hochwassermarke Assured, schlägt es mit PolicyViolation fehl. Attested ohne registriertes Plugin oder ohne FeatureRing::Sovereign schlägt ebenfalls fehl. Es gibt kein stilles Herabsetzen mehr wie heute in lifecycle.rs.
- **Builder:** with\_erasure(..) und with\_erasure\_evidence(..), registriert über den vorhandenen PluginManifest-Port.
- **Lizenz:** Assured braucht keinen Lizenzring (Vorschlag), Attested braucht FeatureRing::Sovereign. Das Lizenz-Gate in MIT- und Apache-Code ist ein kommerzielles Gate, keine Sicherheitsgrenze. Eine Löschpflicht darf nicht am Lizenzstatus hängen: Die Lizenz steuert nur das Ausstellen neuer Beweise, kein open bei Ablauf. Der Verifier bleibt unlizenziert.
- **Ein Default:** Balanced. Compliance nur explizit.

**H.3 Weitere Entkopplungen**

| Maßnahme | Zweck |
| --- | --- |
| engine ohne contextra-license als Pflicht-Dependency: nur der Token-Typ (AuthorizedRing) aus ports, SignedLicenseGate::unactivated\_local wandert in die Fassade | Lizenz aus dem Kern |
| candle aus den Default-Features der Engine. Die Engine kennt nur EmbeddingProvider und LlmTextGenerator aus den Ports | Inferenz aus dem Kern |
| cognition hängt an einem Port (ConsolidationTask), nicht an engine. db bleibt Aggregator | Kognition aus dem Kern |
| mcp: crypto, privacy, license optional hinter Features. py: nur über die Fassade, ungenutzte Abhängigkeiten entfernen | schlankere Binaries |
| Hohle Wiring-Typen löschen, die übrigen auf ein einheitliches Modul-Registrierungsmuster im Builder bringen statt 32 Einzeltypen | weniger Scheinverdrahtung |
| StorageEngine::erasure\_backend() ersetzt den Downcast auf LsmStorage | austauschbare Backends |
| db zu crypto nur als dev-dependency, mit crypto-core meist hinfällig | weniger Kanten |
| Konsolidierung, WASM-Merge-Operator und Reranker an echte Aufrufer im Builder binden oder entfernen | tote Pfade beseitigen |
| Architektur-Ringe in Schicht L0 bis L4 umbenennen, FeatureRing bleibt | Begriffsklarheit |

**H.4 Fence-Varianten im Vergleich**

`if active.load() == 0 { return Ok }` ist als Fast Path richtig, aber nicht ausreichend: Zwischen dem Load und dem Schreiben kann ein Purge die Fence erwerben.

| Option | Mechanik | Kosten im Hot Path |
| --- | --- | --- |
| A: Drain | Der Erwerber setzt active und wartet, bis ein In-flight-Zähler 0 ist. Schreiber erhöhen und senken den Zähler | Zwei atomare Read-Modify-Writes je Schreibvorgang auf einer gemeinsamen Cacheline. Nicht besser als die drei Read-Locks heute |
| B: Barriere im Serialisierungspunkt | Der Fence-Erwerb ist ein Eintrag in der Schreibreihenfolge. Der Schreibpfad prüft einmal je Batch unter dem vorhandenen Lock oder Leader | Praktisch null: ein Load je Batch, kein zusätzlicher Read-Modify-Write |

Die heutigen parking\_lot-Read-Locks sind selbst atomare Read-Modify-Writes (drei Acquire und drei Release je put()) und die Kostenbasis, an der die Alternative zu messen ist. Im WAL-Code wurde kein Leader- oder Group-Commit-Begriff gefunden, append\_batch\_locked deutet auf einen Lock; die Fachdokumente beschreiben commit\_internal unter commit\_mutex mit Leader-Follower \[D\]. Der Check wandert von sechs Aufrufstellen in ops/write.rs an eine Stelle auf Batch-Ebene. (Auflösung v4: Die Prüfung am HEAD ergab, dass commit\_mutex ein tokio-async-Mutex ist, der vor dem WAL-Append freigegeben wird; Option B wird deshalb mit Drain-Wasserstand angepasst umgesetzt, siehe Abschnitt Fence.)

**H.5 Crypto-Erase: Hintergrund**

Der Rewrite-Purge ist O(Gesamtdaten) je Aufruf und auf SSDs wegen Wear-Leveling ohnehin schwach. Der Store hat bereits einen key\_manager je SSTable, und contextra-crypto hat ein KeyRegistry mit Gruppen (DEFAULT\_SHRED\_KEY\_GROUP\_SIZE). Eine Schlüsselhierarchie je Collection (und je Mandant) macht drop\_collection zu einem O(1)-Vorgang: Schlüssel widerrufen, RevocationLog fortschreiben, Dateien lazy bei der nächsten Compaction entfernen. Schlüsselkopien (Caches, Subschlüssel, KV-Segmente) gehören in den Widerrufspfad. Das RevocationLog braucht Crash-Tests und Rollback-Erkennung (Generationen).

**H.6 Offene Entscheidungen zur Löschung und ihre Auswirkungen (v3 II.9)**

Kostenangaben sind aus dem Code abgeleitet, nicht gemessen. Alle drei Fragen sind in v4 entschieden (DE-Register); die Optionen bleiben zur Nachvollziehbarkeit.

*H.6.1 Ist Verschlüsselung im Ruhezustand Voraussetzung für Assured?*

| Option | Leistung | Stabilität und Risiko | Aussage an den Nutzer |
| --- | --- | --- | --- |
| **Ja:** Crypto-Erase ist die primäre Löschung für Collection und Mandant | drop\_collection und Mandantenlöschung werden O(1). Kein Rewrite über den Store, kein Purge-Stall. Kosten: AES-256 auf Lese- und Schreibpfad, mit AES-NI meist verkraftbar, bei Cache-Misses zu benchmarken | **Schlüsselverlust ist Datenverlust.** Backups brauchen ein Schlüssel-Backup, das RevocationLog muss absturzsicher sein. Einzeldokumente bleiben beim Rewrite, nur seltener und gebündelt | Kryptografisch gelöscht, auf SSDs die einzige robuste Methode |
| **Nein:** Assured auch unverschlüsselt | Jede Löschung ist ein Rewrite-Purge, O(Gesamtdaten) je Aufruf, bis Metadatenauswahl und Koaleszenz gebaut sind | Keine Schlüsselverwaltung, einfacher für kleine Installationen. Der physische Anspruch ist schwächer (SSD-Remanenz) | Logisch gelöscht plus Best-Effort-Physik |

Empfehlung v3: Assured ohne Verschlüsselung erlauben, aber die Claims aus der Fähigkeit ableiten. Der Modus Off bleibt unberührt.

*H.6.2 Darf es einen Modus-Downgrade geben?*

| Option | Leistung | Stabilität und Risiko |
| --- | --- | --- |
| Nur Neuaufbau (Export, Import) | Keine Laufzeitkosten, aber Downtime proportional zur Datenmenge. Widerspricht dem Ziel, den Nachweis für Performance zu deaktivieren | Maximal integer, die Zustandsmaschine bleibt klein |
| **Explizite Aktion** (empfohlen) | Laufzeitkosten null, betrifft nur das Öffnen | Das System schreibt einen signierten Eintrag Garantie endet bei Tx X und archiviert den Ledger. Löschungen vor X bleiben belegbar. Danach bewusst kein Schutz mehr gegen Resurrection |

*H.6.3 Ist ein Schreib-Stall außerhalb des Scopes während eines Purge tragbar?*

Diese Entscheidung ist für ein stabiles, schnelles System die wichtigste. Heute stoppt ein Purge alle Schreibvorgänge im ganzen Store, seine Dauer ist O(Gesamtdaten): für ein produktives System nicht tragbar. Mit Präfix-Fence werden nur Schreibvorgänge in den Scope abgewiesen. Zielbild: Ein Stall entsteht höchstens beim Erwerb der Fence (Mikro- bis Millisekunden), nie während des Rewrites. Das gilt für Fence-Option B; lässt sie sich nicht umsetzen, bleibt Option A mit zwei Read-Modify-Writes je Schreibvorgang. Indirekter Stall: Hält das CompactionGate die Compaction während eines langen Purge an, wachsen SSTable-Zahl und Leseaufwand, und eine Schreibdrosselung kann greifen (ob der Store eine solche Drosselung hat, ist nicht geprüft). Gegenmaßnahme: Purge SSTable für SSTable ausführen und das Gate dazwischen freigeben. **Empfehlung:** Außerhalb des Scopes darf kein Schreibvorgang für die Dauer des Rewrites blockieren, nur kurz beim Fence-Erwerb. Das ist ein Abnahmetest.

*H.6.4 Kosten je Modus*

| Modus | Kosten im Normalbetrieb | Kosten bei Löschung |
| --- | --- | --- |
| Off | Null: kein Feld, keine Datei, kein Task | Tombstone und reguläre Compaction |
| Assured | Ein Check je Batch, ein Ledger-Append je Löschung | Collection und Mandant: O(1) mit Verschlüsselung. Dokument: gebündelter Rewrite im Hintergrund |
| Attested | wie Assured | zusätzlich Scans und Signatur, nur auf Anforderung |

Diese Entscheidungen helfen nur, wenn zuerst der Purge-Kern korrekt arbeitet. Solange alte SSTables liegen bleiben und Lesefehler stumm Daten verkürzen, ist jede Aussage über Leistung oder Garantie wertlos.

### Anhang I: Gates, Meilensteine und Betrieb im Detail (v3 II.10 bis II.16)

**I.1 Gate-Katalog am HEAD \[G\]**

governance/gates.toml führt 32 Gates, required\_ci gleich true bei 27.

| Gate | Phase | CI | Bedeutung |
| --- | --- | --- | --- |
| check-dag, check-ring-layering | verify | ja | Zyklen- und Ring-Prüfung reparieren |
| scope-guard, protected-paths (critical), task-card, diff-budget | check, start, submit | ja | Produktgrenzen und Kartenformat: erweitern, nicht neu bauen |
| check-vetoes (critical) | verify | ja | HNSW-Veto klären; EP3- und EP5-Features gegen VETO-F10 (keine mandantenübergreifenden Flüsse) |
| ratchet, gate-weakening (critical), test-integrity | verify, submit | ja | schützt Baselines |
| doc-truth, claims-in-pr, registry-check | verify, submit | ja | Anspruchsregeln, Rückbau toter Verweise |
| gate-unsafe-audit, gate-lock-audit, gate-wal-replay-verify, gate-determinism-check | verify | ja | Crash-Sicherheit |
| fuzz-smoke, mutants-diff | verify | nein (nightly) | Mutationstest für die Garantiegruppen |
| **nicht im Katalog:** orphan-symbols, check-fbs-drift, check-ring-capabilities-consistency, unwired-ports |  |  | aufnehmen |

Der Gate check-claims existiert in xtask/src/check\_claims.rs; seine Claims-Liste muss die Anspruchsstufen abbilden.

**I.2 Prüfbares Budget für Entkopplung und Leistung**

- **Plugin-Abwesenheit pro Binary:** `cargo tree -e normal -p contextra --no-default-features` enthält kein contextra-erasure-evidence und kein contextra-crypto-kv. Zusätzlich `cargo tree -e features` für contextra-mcp und contextra-py. Für store, vector, engine und db auch mit --all-features kein Pfeil auf das Evidence-Plugin.
- **Hot Path:** Criterion-Benchmarks für put und commit (p50, p99, mit geschlossener und offener Last gegen Coordinated Omission) in den Varianten Off, Assured ohne aktive Löschung und Assured während Purge. Zulässige Abweichung gegen eine Baseline ohne Fence: höchstens 1 Prozent. Für während Purge gilt: Schreibvorgänge außerhalb des Scopes dürfen nicht blockieren.
- **Differenztest Purge:** Referenzmodell gegen LSM vor und nach Purge, mit Fehlerinjektion im Stream.
- **Physische Entfernung:** Nach Purged enthält keine Datei im Store-Verzeichnis mehr die Marker-Bytes (Rohsweep des Löschorakels). Das ist der Test, der die Lücke im heutigen Purge gefunden hätte.
- **Löschen ohne Nachweis:** Zähler-Mocks belegen, dass kein LayerScan und keine Zeugen-Allokation vorkommen.
- **Korrektheit:** Loom für das Fence-Protokoll, Crash-Matrix für Ledger, Purge und Recovery (deterministische Simulation), Mutationstest Scanner gleich No-op muss fehlschlagen, verify\_strict-Testvektoren für Ed25519-Grenzfälle.
- **Gate gegen Scheinverdrahtung (deletion-guarantee-gate):** Jedes Symbol der Löschkette braucht einen Produktionsaufrufer und einen End-to-End-Test über die Fassade. Das Gate prüft außerdem die Übereinstimmung von Doku und Code und kommt zuerst, damit gemergt nicht mehr mit wirksam verwechselt wird.
- **Benchmarks bauen pro Paket**, nie mit --workspace.

**I.3 Meilensteine im Detail (v3 II.11)**

Die Messinstrumente werden zuerst verlässlich gemacht, dann wird das Löschsubsystem repariert, erst danach wird entkoppelt und in Crates geschnitten. Status: erledigt, teilweise, offen, \[U\] nicht prüfbar (Stand v3). Geschützte Pfade (xtask, governance, .github, capabilities.toml, AGENTS.md): nur der Maintainer mit ADR, **nicht an Agenten delegierbar**.

*K0: Wahrheit (ca. 1 Woche)*

| Nr. | Aufgabe | Akzeptanz | Status |
| --- | --- | --- | --- |
| K0.1 | compute\_crate\_layers: Fehler propagieren, Zyklen in check-dag als Verstoß melden | Negativtest mit Zyklus wird rot | offen (`let _ =` besteht) |
| K0.2 | check-dag um Ring-Prüfung nach capabilities.toml ergänzen, oder durch check-ring-layering ersetzen | Negativtest mit verletzender Testcrate wird rot | offen |
| K0.3 | durable-fs in ring0\_crate\_order; stale Allowlist-Eintrag graph zu store entfernen | Gate-Lauf ohne diese Befunde | offen |
| K0.4 | CI-Status des Jobs Ring Layering Check und der Branch-Protection klären und dokumentieren | Ergebnis steht im Dokument | offen |
| K0.5 | Eine Unwrap-Baseline (ratchet.toml; .github/unwrap\_baseline.txt wird generiert). Ratsche nur nach unten | eine Quelle | offen (502 gegen 786, am Klon eine Zeile) |
| K0.6 | governance/security\_ports.toml anlegen; Gate prüft, dass jeder Eintrag eine reale Implementierung hat | Gate in CI | offen |
| K0.7 | prune-branches reparieren oder abschalten | Workflow läuft oder ist aus | \[U\] |
| K0.8 | Toolchain 1.89.0 in Agenten-Umgebung und CI | cargo xtask harness-list läuft | in rust-toolchain.toml erledigt; Agentenumgebung \[U\] |
| K0.9 | Hülle und Gate-Zahlen mit cargo messen (Messprotokoll im Anhang) | Zahlen reproduzierbar | offen (bisher Emulation) |
| K0.10 | AGENTS.md (Rangordnung, Roadmap, tote Verweise, lokaler Pfad) durch diese Spezifikation ersetzen | doc-truth grün, kein Verweis auf nicht existierende Dokumente | offen |
| K0.11 | ADR-Hygiene (Dubletten, Status, Herkunft der E-Beschlüsse) und Rangordnung als ADR | ADR akzeptiert | offen |
| K0.12 | orphan-symbols, check-fbs-drift, check-ring-capabilities-consistency in gates.toml aufnehmen; --json gibt nur JSON auf stdout | Katalog gleich Gates; Logs parsebar | offen |
| K0.13 | Scheinverdrahtung prüfen: je wiring-Symbol einen echten Verbraucher im Composition Root oder löschen | 0 Wiring-Symbole ohne Verbraucher | offen (31 von 194) |
| K0.14 | Audit-Etiketten (VERIFIED) nur mit Reproduktionslog und Prüfer ungleich Implementierer | Regel in AGENTS.md | offen |
| K0.15 | publish-pypi.yml, build-wheels.yml, release-mcp-binary.yml bis K4 nur manuell auslösbar | Trigger geprüft | \[U\] |
| K0.16 | Gate deletion-guarantee-gate, Rohsweep-Orakel und Differenztest (Referenzmodell gegen LSM) bauen | Ohne diese Tests sagt gemergt nichts | offen |
| K0.17 | Ring-Regel festlegen (gleicher Ring erlaubt oder nicht) und Ring-Ordnung aus einer Quelle speisen | Gate und Manifest stimmen überein | offen |

*K1: Fail-closed und Reparatur des Löschpfads (ca. 3 bis 5 Wochen; die ersten Aufgaben parallel zu K0 ab Tag 3)*

| Nr. | Aufgabe | Akzeptanz | Status |
| --- | --- | --- | --- |
| K1.1 | contextra\_cloud\_query nur mit Feature guard registrieren. Im EP1-Build nicht erreichbar | Aufruf liefert Werkzeug nicht verfügbar | offen |
| K1.2 | verify\_wiring mit echten Port-Instanzen im Startpfad, Pflichtliste aus security\_ports.toml (include\_str) | Start ohne Sicherheits-Port scheitert | offen |
| K1.3 | **Antwortvertrag forget:** Einzeldokument logically\_deleted (heute); assured\_verified und assured\_unverified erst mit der Löschphase; Collection-Drop mit gleichem Statusvokabular; nie ein Beweisfeld ohne Beleg | MCP-Test über beide Pfade | teilweise (Einzeldokument erledigt) |
| K1.4 | MCP reicht Request-Optionen an erase\_document durch | Assurance-Option ändert das Ergebnis | teilweise (opts gelesen, MCP nutzt Default) |
| K1.5 | Krypto-Review: Schlüsselaufbau, Nonce-Länge, unsafe im Test | Befundliste im Dokument | offen |
| K1.6 | drop\_collection: Beweisordner aus Konfiguration, Instanz-ID aus Manifest, Signaturschlüssel je Datenbank, Err nicht verwerfen, run\_purge-Ergebnis in die Antwort | Purge-Fehler führt zu Fehlerantwort, kein Beweis; keine Artefakte in /tmp | offen |
| K1.7 | engine zu license entfernen; LicenseGate nur als Port | Ring-Gate ohne diese Kante | offen |
| K1.8 | erase\_document ohne Downcast: Fence über Trait oder Port, Test mit TenantScopedStorage | Test grün für beide Storage-Arten | offen |
| K1.9 | Docstring von erase\_document an den Code anpassen, bis Ledger-Append umgesetzt ist | doc-truth grün | offen |
| K1.10 | Purge v2 (typisierte Ausgabe, Fehlerbehandlung, Positionserhalt, Präfix-Fence, CompactionGate, Koaleszenz, produktive FreeSpaceProbe, PurgePending, Dispose verweigert) | Fehlerinjektion liefert Err bei unveränderten Dateien; Rohsweep findet nach Purge keine Marker-Bytes; Referenzmodell-Gleichheit; Schreibvorgänge außerhalb des Scopes laufen während des Purge | teilweise (obsolete\_files, Err-Zweig im Code; Rest offen) |
| K1.11 | MVCC-Rescan statt write\_seq | Paralleler Schreibvorgang wird erkannt | offen |
| K1.12 | Resume und Ledger im Öffnungspfad, Restore-Filter im Checkpoint-Restore | Absturz im Purge wird fortgesetzt, Restore bringt Gelöschtes nicht zurück | offen |
| K1.13 | Snapshot-Isolation: u64::MAX als Lesezeitpunkt ersetzen (read.rs, links.rs, weitere Stellen prüfen) | Lesen immer mit Snapshot | offen |

*K2: Entkoppeln, dann schneiden (ca. 3 bis 5 Wochen)*

**K2a: Kanten schneiden, Crates bleiben im Baum.** PRs unter etwa 400 geänderten Zeilen; unten zuerst, mcp zuletzt.

| Schritt | Änderung | Hülle mcp (Default) danach \[E, erwartet\] |
| --- | --- | --- |
| 1 | engine: default leer. Wo ein Rand-Crate ohne Feature nicht kompiliert: Port oder cfg. engine zu license raus | engine-Hülle 15; mcp 25 auf ca. 21 |
| 2 | db: graph und adapt regulär entfernen (cognition ist bereits weg) | ca. 19 bis 20 |
| 3 | mcp: privacy und adapt hinter Feature oder Port; router aus den contextra-Features nehmen; Werkzeugliste aus der Registrierung statt Konstante | ca. 18 bis 19 |
| 4 | contextra (Fassade): Default-Kette prüfen (Fassade default gleich fast; entscheidend ist der engine-Default) | Fassade 17 |
| 5 | contextra-core ersatzlos entfernen: 405 Dateien im Workspace. **3 bis 4 mechanische PRs nach Modul**, Skript-PRs über Kartenbudget | core verschwindet |
| 6 | default-members auf EP1 und LAB |  |
| 7 | ports::erasure mit ErasureMode (Standard Off), ErasureHooks, LayerScan, WriteGate, erasure\_backend(). Nur Typen, rein additiv | kompiliert, keine Verhaltensänderung |
| 8 | Feature erasure im Store. Fence als Option mit Batch-Check. Engine-Löschpfad ohne Footprint und Zeugen im Modus Off | Messung gegen Baseline, Zähler-Mocks zeigen keine Scans |

**K2b: Mechanische Löschung.** Erst wenn die Hülle das Ziel erreicht und check-ring-layering --strict grün ist, entfernt ein einziger PR (Label scope:core-cut) die Parkplatz-Crates aus Workspace und Dateibaum. Vorher Tags park/EP/v1. Mehrere Ring-Verletzungen (engine zu sandbox, engine zu candle, cognition zu candle, die Allowlist-Einträge) verschwinden dabei von selbst.

**Austritt K2:** Default-Hülle mcp höchstens 18, Kern zu Parkplatz regulär 0, Produkt-Manifest EP1 existiert, capabilities.toml führt je Crate product und status.

*K3: Beweisbar vergessen (ca. 2 bis 3 Monate)*

Die Reihenfolge ist verbindlich:

1. **Krypto-Befunde schließen** (Schlüsselaufbau, Nonce-Länge, unsafe im Test, KMS-Widerruf fail-closed, WAL-Scan-Härtung, partielle AEAD-Frames). Vorrang vor allem anderen.
2. **Veto klären:** HNSW-Reparatur (ADR-097) gegen VETO-F02 formell lösen, sonst darf der HNSW-Zeuge nicht gebaut werden.
3. **Rohträger-Test schreiben, bevor die Nachweis-Kette verdrahtet wird.** Er stammt aus der Spezifikation, nicht vom implementierenden Agenten: Ein rohes Medienabbild (WAL, SSTables, Checkpoints, HNSW-Dateien, KV-Segmente) enthält nach der Löschung die Nutzdaten nicht mehr. Der Test ist zunächst rot.
4. **Assured aufbauen:** Koordinator mit Zustandsmaschine, persistenter Modus (Hochwassermarke), **Ledger-Append vor Tombstone**, Resume im Öffnungspfad, Restore-Filter, Crypto-Erase als primäre Löschung für Collection und Mandant. Der Ledger speichert nach dem Purge nur Digests.
5. **Zeugen aus echten Scans** (Store, HNSW, Text, Relationen, Ledger). Keine Fabrikation (der Hotfix bleibt).
6. **Purge verdrahten:** DpiPurgeDriver::run\_physical nach erase\_document, scrub\_and\_remove im Purge-Pfad. Löschumfang: alle abgeleiteten Speicher.
7. **KV-Nachweis:** generate\_kv\_deletion\_proof, read/write/delete\_kv\_segment, KeyRegistry::open\_durable im Startpfad.
8. **Nachweis-Ausstellung verdrahten:** DpiDeletionRuntime::open im Startpfad, finalize\_logical im Löschpfad, Signer-Schlüssel aus Konfiguration.
9. **Lineage-Kaskade:** Quelllöschung entfernt abgeleitete Daten. Ohne sie darf kein Aggregat-Feature aktiv werden.
10. **Betriebsumgebung ehrlich machen:** HNSW-Zeroisierung, mlock, Scrub-Verweigerung, Kopienstatus statt Zähler.
11. **Crypto-Split** ohne Shim.
12. **Dokumentation:** Rechtsmatrix, Betriebsvorgabe, Python-Doku und Stufenbeschreibung stimmen mit dem Code überein (Gate).
13. **proof wird erst assured\_verified, wenn der Rohträger-Test grün ist.**

**Austritt K3:** Die kritischen Verdrahtungen laufen über Integrationstests auf dem echten Pfad. Rohträger-Test grün. forget liefert für Einzeldokumente einen belegten Assured-Befund. drop\_collection ohne die Fehler des Drop-Pfads.

*K4: Crash-sicher und vollständig verdrahtet (ca. 2 bis 3 Monate)*

- **Nachweisformat:** verify\_strict, TLV, getrennte Domain-Tags, Policy-Defaults, Zeugen-Konstruktoren pub(crate). Abnahme: Testvektoren und Fälschungstests bestehen.
- **Verifier-Crate** mit Ed25519-Grenzfall-Vektoren und Verifier-Spezifikation. Abnahme: unabhängiger Build ohne Engine und Store.
- **Zeitanker (RFC 3161) und Transparenzlog (RFC 9162).** Abnahme: Rollback der Audit-Kette wird erkannt.
- **Fassade:** Features und Builder-API, totes Feature und Flag entfernen, ein Default (Balanced).
- **Crash-Sicherheit:** Compaction und MANIFEST, HMAC-Zustand, Group-Commit-Liveness, Checkpoint-Indizes, atomares put\_if\_absent, SSI-Read-Set-Tracking, Hochwassermarke gegen WAL-Kürzung, Legacy-WAL-Entfernung, HNSW-Arena-Zeroisierung bei Purge. Jeder Prüfauftrag liefert einen Test, der den Befund belegt, oder wird als gegenstandslos geschlossen.
- **Pflicht-Gates:** Fault-Injection, Loom, Mutationstest für die Garantiegruppen Crash/WAL, Löschung/Krypto und Audit (mutants-guarantees.toml, Schwellen festlegen).
- **Budget** aus dem Gate-Teil läuft in xtask und CI.

**Austritt K4:** Zähler öffentliche Funktionen ohne Aufrufer und ohne Aktion gleich 0 für EP1. Crash-Matrix grün. Release v0.1.0. Danach beginnt EP2 als eigenes Repository.

**Entfernt bis K4 abgeschlossen (Regel PR1 bis PR5, Abschnitt 4.5):** Optimierungen, py, EP2 bis EP6 und alle Features außer Hybridsuche, Mandanten und Löschung; ACI, SSN, DiBud, KIVI, SnapKV, Sketched LinUCB, Spektral-SVD.

*K5: Erweiterungen (nach K4, je Produkt eigenes Gate)*

EP2 zuerst (Trennung von Erzeuger und Prüfer), EP3 bis EP6 nach Bedarf. Jedes Produkt braucht Manifest, Hüllen-Gate, Akzeptanzliste und eine Abschaltprobe.

**I.4 Betriebsmodell: Ergänzungen (v3 II.12)**

**Veto-Register:** VETOES.md und der Gate check-vetoes gelten vor jedem Plan. Aktuell: VETO-F02 (HNSW nur Tombstone-Pruning, Review bis 2027-10-12), VETO-F10 (permanent: keine mandantenübergreifenden Flüsse), VETO-OP03 (keine Audio- oder Sprachfunktionen), VETO-ADR (keine Umgehung ohne ADR). Jede Karte für EP3 bis EP6 prüft gegen diese Liste.

**Erste Karten** (die ersten beiden sind wegen des geschützten Pfads Maintainer-Aufgaben):

| Karte | product | Pfad (scope) | Akzeptanz | acceptance\_by |
| --- | --- | --- | --- | --- |
| Zyklen in check-dag | LAB | xtask/src/lib.rs | Negativtest mit Zyklus wird rot | Maintainer |
| Ring-0-Ordnung | LAB | xtask/src/check\_ring\_layering.rs | Gate-Lauf ohne die genannten Verletzungen | Maintainer |
| forget: Request-Optionen durchreichen | EP1 | crates/contextra-mcp/src/server\_tools.rs, crates/contextra-engine/src/collection/crud/delete.rs | MCP-Test: Assurance-Option ändert das Ergebnis | Spezifikation |
| cloud\_query nur mit guard | EP1 | crates/contextra-mcp/src/server\_dispatch.rs, server\_tools.rs, Cargo.toml | Werkzeug fehlt im EP1-Build | zweite Karte |
| verify\_wiring mit echten Ports | EP1 | crates/contextra/src/wiring\_selftest.rs, Startpfad der Fassade | Start ohne Sicherheits-Port scheitert | zweite Karte |
| Purge-Fehlerbehandlung | EP1 | crates/contextra-store/src/lsm/ops/purge.rs, wal/retire.rs | Fehlerinjektion liefert Err, Dateien unverändert | Referenzmodell |
| Symbole aus core nach types (Modul store, Teil 1) | EP1 | crates/contextra-store/src/\*\* (max. 25 Dateien) | cargo check -p contextra-store ohne core-Abhängigkeit | Compiler |
| Rohträger-Test | EP1 | crates/contextra-engine/tests/\*\* | Test ist zunächst rot und schlägt ohne Implementierung fehl | Spezifikation |

**I.5 Format-Inventar und Lieferkette (v3 II.13)**

Formatstabilität gilt ab v0.1.0. Inventar \[D, Detail ungeprüft\]:

| Format | Quelle | Gate |
| --- | --- | --- |
| WAL (v3, Frame-Header v4) | ADR-029, N13, 100 | gate-wal-replay-verify |
| SST-Format-Version | ADR-105 | \[U\] |
| MANIFEST, Upgrade-on-open | ADR-E17 | \[U\] |
| Beweisformat v4 (TLV, deterministisch), v3 entfernt | ADR-E05, E06 | proof-correctness.yml |
| FlatBuffers-Schema | ADR-084 | check-fbs-drift (nicht im Katalog; ein Lauf Exit 1) |
| Checkpoint-Manifest (Blake3) | ADR-E31 | \[U\] |

Pflicht vor v0.1.0: Upgrade- und Downgrade-Test je Format und Hochwassermarke für ErasureMode. DSGVO und Compliance: Der Compliance-Nutzen entsteht erst mit C2 und C3. audit-export (Art. 30) und avv-generator (Art. 28) gehören zu EP2, sind klein (473 und 315 Zeilen laut WORKING\_STATE.md) und experimentell. SECURITY.md beschreibt das Bedrohungsmodell (Prompt-Injection, Fremdcode, MCP-Missbrauch).

**I.6 Architekturentscheidungen in Kurzform (v3 II.14)**

| Entscheidung | Verworfene Alternative und Grund |
| --- | --- |
| Ports in contextra-ports, Plugins implementieren sie | Traits im Plugin: Der Kern müsste vom Plugin abhängen |
| dyn im Löschpfad. Im Schreibpfad Option oder cfg-Feld mit Batch-Check | Generischer Engine-Parameter: infiziert die gesamte öffentliche API und die Python-Bindings für einen nicht heißen Pfad |
| Ledger gehört zu Assured, nicht in den immer aktiven Kern | Ledger im Kern: ein weiteres Pflichtartefakt in jedem Prozess. Preis: im Modus Off kein Restore-Schutz |
| Crypto-Erase-Mechanik in crypto-kv, Beleg im Plugin Attested | Alles ins Plugin: Verschlüsselung ohne Widerruf wäre nicht löschfähig |
| Eigener Verifier-Crate | Verifier im Plugin: Auditoren müssten Engine-Code bauen |
| Compile-Feature und Laufzeitmodus, Standard Off | Nur Laufzeit: Der Code bliebe im Binary und der Fence-Check im Schreibpfad |
| Modus als Hochwassermarke im Manifest, Downgrade nur per expliziter Aktion | Nur Ledger-Existenz prüfen: Downgrade bleibt unbemerkt |
| Crypto-Erase primär für Collection und Mandant, Rewrite für Einzeldokumente | Rewrite für alles: O(Gesamtdaten) je Aufruf, SSD-Remanenz |
| Plugins als eigene Crates, Features nur für Store-Interna | Nur Features: Unification zieht Code in jedes Binary |
| Fence-Check je Batch am Serialisierungspunkt (zu verifizieren, sonst Drain-Variante) | Check je Schlüssel: sechs Aufrufstellen, mehrere Read-Modify-Writes |
| MVCC-Rescan als Prüfung, Fence als Prävention | write\_seq als Beweis: zählt nur Abweisungen |
| Purge v2 vor dem Crate-Split | Split zuerst: Ein nicht funktionierendes Subsystem würde verschoben, Fehler blieben verdeckt |
| Crypto-Split ohne Shim vor v0.1.0 | Shims: unnötig ohne externe Konsumenten |

**I.7 Weitere Risiken und Grenzen (v3 II.15)**

Ergänzend zu Abschnitt 15: Die Verdrahtungs-Heuristik zählt falsch in beide Richtungen (Funktionszeiger und Makros werden übersehen, Namenskollisionen und Wiring-Dateien blähen auf; Gegenmaßnahme: strikte Variante, Kollisionsindikator). EP1 bleibt groß (rund 270.000 Zeilen in den 18 Kern-Crates, davon 71.918 in store): Der Schnitt macht den Kern fokussiert, nicht klein. Geschützte Pfade machen einen großen Teil von K0 nicht delegierbar und begrenzen den Durchsatz stärker als die Sitzungszahl. Nicht verifiziert (Stand v3): die Spezifikation v15 im Einzelnen (nur Gliederung und Positionierung gelesen), die Mehrzahl der 170 ADRs (nur Titel; acht inhaltlich gelesen), docs/audits, der Inhalt der 123 Task-Karten, der Jules-Betrieb, CI-Läufe, Branch-Protection, die Aktivierung der Release-Workflows. Nicht gelesen: Compaction-Engine, WAL-Schreibpfad im Detail, Checkpoint-Restore, HNSW-Zeroisierung, KV-Cache, Gateway-Logik, target-Abhängigkeiten. Die 867 undokumentierten Elemente aus einem früheren Plan sind nicht erfasst. Die Einzelzeilen der drei Emulations- und Zuordnungstabellen liegen nur als Aggregate und Beispiele vor.

**I.8 Emulationsmethoden (Python, ohne cargo) (v3 II.16)**

1. **Hülle:** Parser über alle 37 Cargo.toml mit Workspace-Vererbung; Feature-Auflösung als Fixpunkt über default, dep:x, x/feat, x?/feat und implizite Optional-Features; nur Normal-Kanten; Zielabhängigkeiten zählen vollständig.
2. **Ring-Gate:** Regeln aus xtask/src/check\_ring\_layering.rs nachgebildet (Ring aus package.metadata.contextra.ring, ring0\_crate\_order, ring3\_crate\_order, Allowlist); alle deklarierten Kanten inklusive optional, dev und Zielabschnitte.
3. **Verdrahtung:** Textsuche name( in crates/\*\*/\*.rs; Variante alt schließt Tests, benches, examples, fuzz aus; Variante strikt zusätzlich Dateien mit Test im Namen, Testkit und alles ab dem ersten cfg(test) je Datei; Definitions- und Kommentarzeilen zählen nicht.
4. **Scheinverdrahtung:** Öffentliche Symbole in wiring-Dateien (pub fn, struct, enum, trait), Verweis als Wortgrenzentreffer in einer anderen Produktionsdatei.

### Anhang J: Kryptovergleich, Audit-Export, Phasenabbildung und Prüfbericht (v3 20.4, 21.5, 25, 26)

**J.1 Entscheidungskriterien für das Kryptoverfahren (v3 20.4, K1.5)**

| Kriterium | AES-256-GCM | AES-256-GCM-SIV | XChaCha20-Poly1305 |
| --- | --- | --- | --- |
| Nonce-Fehlgebrauch | katastrophal bei Wiederverwendung | tolerant (RFC 8452) | 192-Bit-Zufallsnonce, praktisch kollisionsfrei |
| Rollback und Restore | Zähler-Nonce nach Restore gefährdet | unkritisch | unkritisch |
| BSI TR-02102-1 | genannt \[D\] | nicht als Standardmodus genannt \[U\] | nicht genannt \[U\] |
| Beschleunigung | AES-NI | AES-NI | Software, schnell |
| Grenze je Schlüssel | NIST: höchstens 2^32 Verschlüsselungen mit Zufallsnonce | höher, praktisch unkritisch | sehr hoch |

**Entscheidungsregel (v3):** Der Code verwendet heute AES-256-GCM-SIV (aes-gcm-siv). Das Verfahren bleibt, bis der Krypto-Review entscheidet (in v4 als final festgelegt, siehe Register). **Unabhängig davon** gilt: Kein Text, auch kein Beweis oder Export, behauptet "BSI TR-02102-konform" für ein Verfahren, das der Maintainer nicht gegen die jeweils aktuelle Fassung der Richtlinie geprüft hat. Zulässig ist ein **Algorithmen-Inventar** ("verwendet: AES-256-GCM-SIV, HKDF-SHA-256, Ed25519, BLAKE3"). Bei GCM gilt zusätzlich: DEK-Rotation vor 2^30 Verschlüsselungen je Schlüssel (Generationszähler). Pflichtpunkte des Reviews bleiben: Nonce-Länge, Schlüsselaufbau, das `unsafe` im Test, fail-closed Widerruf.

**J.2 Verifier-Crate und Audit-Export (v3 21.5)**

`contextra-proof-verify`: `no_std` mit `alloc`, kein `unwrap`, `expect` oder Indexzugriff (der Quell-Blueprint verstößt dagegen, J.5 Nr. 8), nur `get` und `try_into` mit Fehlerwert. Abhängigkeiten: `ed25519-dalek`, `blake3`, `sha2`. Eigene Spezifikation, Fuzz-Ziel und die Testvektoren aus RFC 8032 und Wycheproof (kleine Ordnung, nicht kanonisches S, Randwerte). **Differenzielle Verifikation:** Eine zweite Implementierung (andere Sprache, nur aus der Spezifikation geschrieben) und die Rust-Fassung müssen auf einem Korpus aus gültigen und mutierten Beweisen übereinstimmen.

`AuditExportDocument` (EP2): CBOR-Hülle um die Beweisbytes mit `ComplianceAttestation` (Algorithmen-Inventar, Schlüssellängen, Schlüssel-Lebensdauer-Richtlinie) und maschinenlesbaren `VerificationInstructions` (die Schritte des Verifikationsprotokolls für externe Prüfer). Der Export enthält nie Klartext der Anwendungsdaten.

**J.3 Phasenabbildung (v3 25.1)**

| Quelle | Phase | Entspricht |
| --- | --- | --- |
| Impl-Spezifikation | 0a (Build, CI) | K0.9, K0.18, K0.19 |
|  | 0b (N-B Hotfix) | K1.8 und K1.14 |
|  | 0c (Gate, Orakel) | K0.16 |
|  | 0d (Purge-Korrektheit) | K1.10 |
|  | 0e (Löschpfad ehrlich) | K1.3 bis K1.6, K1.9 |
|  | 1 (Ports) | K2 Schritt 7 |
|  | 2 (Off-Nullpfad, Fence B) | K2 Schritt 8 |
|  | 3 (Assured) | K3 Punkt 4 |
|  | 4 (Crate-Split) | K3 Punkt 11 |
|  | 5 (Attested-Härtung) | K4 (Nachweisformat) und EP2 |
|  | 6 (Fassade) | K4 (Fassade) |
| Masterplan v2 und Optimierung | Phase 0 bis 5 | K0 bis K4, Zeiträume gelten als Untergrenze (Widerspruch 8) |

**J.4 Zusätzliche Aufgaben (v3 25.2, ergänzen die Tabellen in Anhang I)**

| Nr. | Aufgabe | Akzeptanz |
| --- | --- | --- |
| K0.18 | CI: Merge-Queue, Check je Paket, `--locked`, Feature-Matrix, kein Merge auf rotes `main` | Workflow vorhanden, Branch-Protection dokumentiert |
| K0.19 | `cargo check --locked` je Paket als erste Messung; Befund N-N, N-O, N-P bestätigen oder schließen | Log im Repository |
| K0.20 | Spezifikationscode-Regel: Codeblöcke in Plänen gelten als Illustration; `doctrine-scan` läuft über neue Codepfade | Regel in AGENTS.md |
| K1.14 | N-B: Standard-`delete` unter `TenantScopedStorage` ohne `CapabilityUnsupported` (Tenant-Matrix) | `tenant_isolation_facade` grün |
| K1.15 | `KvDeleteMode::default()` auf `TombstoneOnly` (am HEAD bereits der Fall, siehe Befunde; Unit-Test bleibt Pflicht) | Unit-Test, kein Feature nötig |
| K1.16 | Builder: `verify_wiring` für alle sieben No-op-Optionen, `BuildError::IncompleteWiring` | `check-builder-wiring` grün |
| K1.17 | Failpoints nur hinter `fault-injection` (N-J) | `grep` gleich null |
| K1.18 | Orphan-GC mit Scrub, `manifest_generation` nie zurücksetzen (N-G) | Orphan-Test |
| K1.19 | ID-basierte Präfixe statt Namenspräfixe | Test mit Collection-IDs 4 und 42, Wiederanlage unter gleichem Namen |
| K2.9 | `erasure_backend()` auf `StorageEngine`, alle fünf Downcasts ersetzt | `grep downcast_ref` gleich null |
| K3.14 | `LayerEraser` für store, vector, text, graph | Löschung entfernt persistierte Indexdateien (Rohsweep) |
| K3.15 | WAL-Barriere | CRASH-15 grün |
| K3.16 | Snapshot-Lebensdauer in Assured, Startbedingung des Purge | CRASH-16 grün |
| K3.17 | Record-Verschlüsselung mit Collection-DEK vor dem WAL-Append, Dateien je Collection für Indizes | Crypto-Erase-Abdeckungstest |
| K3.18 | Schlüsselhierarchie zufällig und gewickelt, dreistufige Vernichtung | CRASH-13 grün, Test "kein Shred-Schlüssel mit HKDF-Label" |
| K3.19 | Ledger-Format mit geschlüsselten Digests, Rollback-Erkennung, Verdichtung | CRASH-12 und CRASH-17 grün |
| K3.20 | `CopyRegistry` und `checkpoint_policy` | Status je Kopie im Statusbericht |
| K4.9 | Beweisformat gemäß Beweisformat v4, Verifier `no_std`, differenzielle Verifikation | Tampering-Matrix bytegenau grün |
| K4.10 | `erasure_adopt()` (Backfill Off nach Assured) | Test Off nach Assured mit vorhandenen Tombstones |
| K4.11 | `contextra_erasure_status` und `forget` mit `wait_ms` | MCP-Test über alle Zustände |

**J.5 Prüfbericht zu den Quelldokumenten: Fehler in den Blueprints (v3 26.1, nicht übernommen)**

Verweise auf Abschnitte der v3 sind auf die normativen Abschnitte von v4 abzubilden (Port-, Löschpfad-, Ledger-, Schlüssel- und Beweisformat-Abschnitt).

| Nr. | Quelle | Befund | Korrektur |
| --- | --- | --- | --- |
| 1 | Resume-Blueprints (Impl, Löschnachweis) | `let _fence = acquire_fence(..)` wird am Funktionsende fallen gelassen, die Fence ist beim Rewrite nicht mehr aktiv | Guard wird in den Task verschoben |
| 2 | Resume-Blueprint (Code-Blueprints) | führt den Purge synchron beim Öffnen aus, ohne Fence, ohne `dispose`, mit Verzeichnisname `purge_intents` | Hintergrund-Task, Fence, `dispose`, Verzeichnis `purge/` (Widerspruch 24) |
| 3 | Port-Entwurf | `PhysicalScope::new` ist `pub`, obwohl "nur vom Backend erzeugbar" | Provenienzprüfung |
| 4 | Fence-Entwurf | Präfixvergleich ohne Trennzeichen: Präfix von Collection 4 trifft Collection 42 | trennzeichenterminierte Präfixe |
| 5 | Schlüsseltyp-Entwurf | `impl !Clone` ist auf stabilem Rust nicht zulässig | kein `Clone`, `compile_fail`-Doctest |
| 6 | Advanced Algorithms | KEK und DEK per HKDF aus dem Master-Schlüssel abgeleitet: nicht shreddbar | zufällig und gewickelt |
| 7 | Verifier-Blueprint | feste Offsets (`proof_bytes[36..68]`), fixe sieben Layer, Länge 164 | TLV-Parser |
| 8 | Verifier, Hüllengate, Dispatcher | `unwrap`, `expect("checked above")`, Indexzugriffe in Code, der Zero-Panic verlangt | `get` und `try_into` mit Fehlerwert, `Option`-Match |
| 9 | Löschnachweis-Spezifikation | RFC-3161-Zeitstempel im Hash des Layers 7, der im signierten Beweis steht: zirkulär | unsignierter Anhang |
| 10 | Masterplan, Löschnachweis | "Forward-Secure Hashkette" ohne Schlüsselentwicklung: eine einfache Hashkette ist nicht vorwärts-sicher | Bezeichnung "Hashkette", Vorwärtssicherheit nur mit Schlüsselrotation und eigenem Entscheid |
| 11 | Ledger-Layout | Klartextpräfixe dauerhaft im Ledger, im Widerspruch zu "nur Digests nach Purge" | geschlüsselte Digests, Verdichtung |
| 12 | Layer-6-Beschreibung | "Löschung des Schlüssels aus dem RevocationLog": das Log hält Widerrufe, nicht Schlüsselmaterial | Widerruf und Vernichtung getrennt |
| 13 | MCP-Blueprint | Beweis und `verified: true` direkt in der Antwort | Beweis über Status-Abruf |
| 14 | Crypto-Erase-Beschreibung | Schlüssel je SSTable trägt Collection-Shredding nicht | Record-Verschlüsselung |
| 15 | N-K-Fix | Präfix über Collection-Namen | `CollectionId` |
| 16 | Purge-Beschreibungen | WAL-Segmente mit Nutzdaten bleiben bis zur Stilllegung bestehen, keine Barriere | WAL-Barriere |
| 17 | Purge-Beschreibungen | Purge zerstört Versionen für offene Snapshots | Startbedingung, Snapshot-Lebensdauer |
| 18 | SOTA-Feinspezifikation | Ring-Zuordnung weicht ab (kvcache, rank, graph in Ring 0; engine in Ring 1) und widerspricht `capabilities.toml` | `capabilities.toml` gilt |
| 19 | Hüllengate-Blueprint | prüft `-p contextra` statt `-p contextra-mcp`, mit `unwrap` | je Binary (Widerspruch 28) |
| 20 | Conformal-Beschreibung | Nonconformity-Maß für irrelevante Dokumente fällt aus der Garantie | nur relevante Beispiele |
| 21 | OpenIE-Beschreibung | "Temperatur 0 und fester Seed" als Bitdeterminismus | Aufzeichnung der Ausgaben |
| 22 | Masterplan, Advanced, SOTA | "7-Schichtenmodell nach BSI TR-02102" | nicht belegt \[U\], Berichtssicht |
| 23 | SOTA-Verifikation | Signatur über zusammengeklebte Felder ohne Domain-Tag und Längenpräfix (`document_id` als Zeichenkette) | kanonisches TLV mit Domain-Tag |
| 24 | Port-Blueprints | `LogicalScope` mit den Varianten `Keys` und `Prefixes` neben `Document` und `Collection` | drei Varianten |
| 25 | Code-Blueprints | `ErasureBackend::purge` asynchron | synchron (Widerspruch 25) |
| 26 | Systemdesign | Typ-Zuordnung widersprüchlich: `contextra-vector` Ring 0 und `store` Ring 1 | Schichten aus dem Architekturabschnitt |
| 27 | Löschnachweis-Spezifikation | `proof: "layer_witnesses: [[u8;32];7]"` und Layerstatus `Pending` in einem ausgestellten Beweis | Beweis nur bei abgeschlossenen Layern |

**J.6 Lücken, die kein Quelldokument schließt (v3 26.2, dort entschieden)**

`LayerEraser` für persistierte Indizes, WAL-Barriere, Snapshot-Lebensdauer, `CopyRegistry`, Record-Verschlüsselung, Metadatenrest der Bezeichner, zweistufiger Collection-Drop, Katalog-Tombstone, `erasure_status`, Backfill, Ledger-Rollback-Erkennung, Verifier-Differenztest, Positivkontrolle im Rohträger-Test, Lineage für abgeleitete Strukturen. Alle sind in den normativen Abschnitten von v4 spezifiziert.

**J.7 Offene Punkte der v3-Revision und ihr Stand in v4 (v3 26.3)**

| Nr. | Offener Punkt in v3 | Stand in v4 |
| --- | --- | --- |
| 1 | Ob die ID-basierte Präfixierung mit dem bestehenden Key-Codec ohne Formatbruch möglich ist (sonst Formatversion und Upgrade-Test, S11) | Am HEAD geprüft: der Store-Key-Codec ist bereits ID-basiert; die Namens-Layouts liegen in der Engine. Damit am HEAD geklärt; die Namens-Layouts der Engine sind zu migrieren (K1.19) |
| 2 | Ob Record-Verschlüsselung vor dem WAL mit dem Schlüssel je SSTable koexistieren kann | Entschieden: Record-Ebene vor dem WAL, Metadatenrest getrennt (DE-08, DE-10) |
| 3 | Ob `max_snapshot_age` Standardwert oder Pflichteinstellung in Assured ist | Entschieden: Pflichtwert in Assured, Standard 15 Minuten (DE-20) |
| 4 | Wortlaute der Honesty-Klausel und der Claim-Texte (13.2) | **Dem Maintainer vorbehalten** (Wortlaut ist eine Rechtsentscheidung) |
| 5 | Ob die zweite Verifier-Implementierung in EP2 oder früher entsteht | Entschieden: Pflicht für den EP2-Release, nicht für v0.1.0 (DE-25); die Lizenz des EP2-Ausstellers bleibt dem Maintainer vorbehalten |

**J.8 Quellenherkunft und Hinweis zu v3 14.2**

v3 14.2 (Aufgelöste Widersprüche zwischen den Expertenplänen) listet Widersprüche 1 bis 14 ausdrücklich; weitere Widersprüche (u. a. 24, 25, 28) werden in v3 nur an den Stellen genannt, an denen sie entschieden wurden, und sind oben in J.5 zitiert. Die Quellenzuordnung der v3 (Anhang A) und das Messprotokoll (Anhang B) bleiben in den Anhängen A und B von v4 erhalten.

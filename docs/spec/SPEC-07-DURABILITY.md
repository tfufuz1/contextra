# SPEC-07 — Durability, Recovery, GC, Ressourcen und Sicherheit

**Version 3.0 · 26. November 2026 · Status: Final**
**Querverweise:** [Index](SPEC-00-INDEX.md) · [Storage](SPEC-03-STORAGE-ENGINE.md) · [Versionierung](SPEC-04-VERSIONING.md) · [API](SPEC-08-API.md)

---

## 1. Durability-Vertrag

### 1.1 Ordnungsregeln

| ID | Regel |
|---|---|
| **D-1** | Kein Zeiger (Manifest, Ref, Locator) wird durabel, bevor sein Ziel durabel ist (Datei-Sync **und** Verzeichnis-Sync). |
| **D-2** | Packs sind nach dem Checkpoint Wahrheitsquelle; Locators und Indizes sind rekonstruierbar. |
| **D-3** | Ein Sync-Fehler führt zu Fail-Stop. |
| **D-4** | Commits sind unveränderlich. |
| **D-5** | Nichts wird gelöscht, bevor der ersetzende Zustand durabel registriert ist. |

### 1.2 Stufen

| Stufe | WAL | Sichtbar / bestätigt nach | Verlustfenster bei Stromausfall |
|---|---|---|---|
| `volatile` | nein | RAM-Apply | alles |
| `buffered` | ja | `write` | bis Sync-Intervall (Default 10 ms, max. 100 ms) |
| `durable` (Default) | ja | Group-`fdatasync` | keines |
| `strict` | ja | eigener sofortiger Sync, bei neuen Dateien inkl. Verzeichnis | keines |
| `replicated` | reserviert (v2) | Quorum | — ; v1 lehnt ab |

Die Zusagen setzen voraus, dass Dateisystem und Gerät Flushes korrekt umsetzen. Plattformspezifisch: Linux `fdatasync`; macOS `F_FULLFSYNC`; Windows `FlushFileBuffers`. Netzwerkdateisysteme sind nicht unterstützt.

---

## 2. WAL

**Ein gemeinsamer Log** je Store. Segmente 64 MiB, vorallokiert, Rotation bei Größe oder 60 s Aktivität.

### 2.1 Segment-Header (4 KiB)

`magic "CXWS" · version u16 · compat u16 · incompat u32 · segment_no u64 · segment_epoch u64 · store_uuid[16] · segment_salt[16] · dek_id u32 · first_lsn u64 · crc32c u32`, Rest 0.

### 2.2 Frame (8-Byte-aligned)

| Off | Len | Feld |
|---|---|---|
| 0 | 4 | `magic = "CXWF"` |
| 4 | 4 | `frame_len` (gesamt inkl. Padding) |
| 8 | 8 | `lsn` |
| 16 | 8 | `commit_seq` (0 = kein Commit) |
| 24 | 8 | `hlc` |
| 32 | 8 | `segment_epoch` |
| 40 | 4 | `branch_id` |
| 44 | 1 | `frame_kind` |
| 45 | 1 | `durability` |
| 46 | 2 | `flags` (`FIRST`, `LAST`, `ENCRYPTED`) |
| 48 | 4 | `payload_len` |
| 52 | 2 | `idem_len` |
| 54 | 2 | `intent_len` |
| 56 | 4 | reserviert |
| 60 | 4 | `header_crc32c` (Bytes 0–59) |
| 64 | var | Idempotenzblock ‖ Intent ‖ Payload (CBE) ‖ ggf. AEAD-Tag |
| Ende | 4 + Pad | `frame_crc32c` über alles davor, dann Nullpadding auf 8 |

Header 64 Byte. Der Idempotenzblock enthält Scope, Key und den 32-B-`request_digest`.

| `frame_kind` | Inhalt |
|---|---|
| 1 `CommitTx` | CommitHeader, ChangeSet, Work-Enqueues, Allocator-Marken |
| 2 `BranchOp` | Create, Savepoint, Rollback, Discard |
| 3 `WorkOp` | Attempt, Complete, Fail, Cancel |
| 4 `DerivationPut` | Schlüssel, Daten (große Daten als Verweis auf bereits durables Pack) |
| 5 `IdemOp` | Widerruf, Ablauf |
| 6 `Expunge` | Purge-Absicht und betroffene Hashes |

**Mehrframe-Transaktionen:** Überschreitet eine Transaktion die Framegrenze (16 MiB) oder den Segmentrest, wird sie in `FIRST … LAST` geteilt. Sie gilt nur, wenn alle Frames lückenlos und gültig vorliegen. Fortsetzungen dürfen ins nächste Segment reichen; der WAL-Writer verschachtelt keine fremden Frames dazwischen.

**Complete + Ergebnis:** Ein `complete`, das Wahrheit ändert, ist **ein** `CommitTx`-Frame mit eingebettetem Queue-Übergang.

### 2.3 Group Commit

Ein WAL-Writer-Thread. Läuft kein Sync, wird sofort geschrieben und synchronisiert; währenddessen eintreffende Commits bilden die nächste Gruppe. Optional zusätzliche Sammelzeit `group_commit_max_wait_ms` (Default 0, max. 1), Grenzen 64 Commits bzw. 1 MiB je Gruppe (eine größere Einzeltransaktion bildet eine eigene Gruppe). Ein `fdatasync` deckt alle zuvor geschriebenen Frames, auch `buffered`.

### 2.4 Fail-Stop („fsyncgate")

Scheitert Schreiben oder Sync an WAL, Manifest, Superblock oder Pack-Registrierung:

1. Store ⇒ `repo_poisoned`; keine Publikation ausstehender Commits; der Sync wird **nie** auf demselben Deskriptor wiederholt.
2. Lesen bereits publizierter Zustände bleibt erlaubt.
3. Recovery nach Neustart traut dem Page Cache nicht: Der letzte nicht checkpointete WAL-Abschnitt wird mit `O_DIRECT` (bzw. nach Cache-Verwerfen) gelesen, der als gültig akzeptierte Präfix in ein **neues** Segment übernommen und synchronisiert, bevor er sichtbar wird. Ist direktes Lesen nicht verfügbar, wird dies im Recovery-Bericht ausgewiesen und ein Neustart des Hosts empfohlen.

### 2.5 Recycling

Ein Segment darf entfallen, wenn alle Frames abgedeckt sind durch **(a)** durable `TruthBinding` bzw. ChangeSets, **(b)** Idempotenz-Checkpoint, **(c)** Work-Queue-Checkpoint, **(d)** Derivations-Flush, alle im Manifest registriert:

```
checkpoint_lsn = min(truth_lsn, idem_lsn, work_lsn, derivation_lsn)
```

Ein untätiger Branch mit ungefaltetem Delta wird durch den Zeit-Trigger gefaltet und hält das WAL damit nicht dauerhaft fest.

---

## 3. Manifest und Superblock

### 3.1 Records

`len u32 · type u16 · flags u16 · gen u64 · prev_record_blake3[32] · payload · crc32c u32`

`AddPack`, `Condemn`, `RemovePack`, `UpdateRefs` (CAS + Reflog), `AddLocatorRuns`, `DropLocatorRuns`, `TruthBinding`, `Signature`, `IndexManifest`, `Allocators`, `Checkpoint`, `SidecarCheckpoint`, `Expunged`, `Snapshot` (Vollzustand).

Eine Manifest-**Gruppe** wird mit einem Abschlussrecord atomar; unvollständige Gruppen werden verworfen. Die Hashkette erkennt Beschädigung und Vertauschung; sie ist **kein** manipulationssicherer Nachweis gegen Angreifer mit Schreibzugriff (dafür Signaturen bzw. AEAD). Rotation ab 16 MiB oder 10 000 Records.

### 3.2 `CURRENT`

Datei mit zwei 4-KiB-Slots: `magic · version · generation u64 · manifest_no u64 · tail_hint u64 · store_uuid · repo_seed (nur unverschlüsselt) · feature_flags · crc32c`. Aktualisiert wird **nur der ältere** Slot, dann `sync_data`. Gültig ist der Slot mit höchster Generation und korrekter CRC. Beim Anlegen entsteht die Datei per Temp-Datei, Sync, Rename, Verzeichnis-Sync. `tail_hint` ist nur ein Hinweis; Recovery liest bis zum letzten gültigen Record.

---

## 4. Recovery

```
1 LOCK erwerben; CURRENT und ggf. KEYS laden; Format-Flags prüfen
2 Manifest bis zur letzten vollständigen Gruppe lesen
3 Packs, Locators, Refs, Bindungen, Allocatoren, Sidecar-Checkpoints laden
  (fehlende Locators aus Pack-Footern rekonstruieren)
4 WAL ab checkpoint_lsn lesen; Stopp am ersten Frame mit falscher Länge, CRC,
  Epoche, nicht fortlaufender LSN oder unvollständiger FIRST…LAST-Kette
5 Frames in LSN-Reihenfolge idempotent anwenden: Deltas, Idempotenz, Queue,
  Derivationen, Expunge-Absichten, Allocatoren
6 Leases verwerfen (Leased → Pending); offene Expunges fortsetzen
7 neue WAL- und Manifest-Datei öffnen (neues Salt, neue Epoche)
8 nicht registrierte Packs und tmp/ als Waisen vormerken
9 bereit; fehlende Indizes über Flat-Fallback, Builds einreihen
```

Ein ungültiger Frame, **hinter dem** noch gültige Frames derselben Epoche folgen, ist kein normaler Torn Tail: Recovery stoppt mit `wal_corrupted` und verlangt eine explizite Operatorentscheidung, außer die Folgeframes gehören nachweislich ausschließlich zur nie synchronisierten letzten Gruppe (durch Gruppenmarken im Frame-Flag erkennbar). Recovery ist wiederholbar: Absturz während der Recovery ändert das Ergebnis nicht.

---

## 5. Garbage Collection (SATB + Remark)

### 5.1 Wurzeln

Branch-Heads, Tags, Checkpoints, offene `mr/*`; Reflog-Einträge (jünger als 30 Tage, mindestens die letzten 1 000); Savepoints aktiver Sessions; Snapshot-Leases (300 s, erneuerbar, harte Obergrenze 24 h); laufende Transaktionen, Folds, Builds, Backups; nicht gecheckpointete WAL-Inhalte.

### 5.2 Retention-Horizont

Die Markierung folgt Eltern-Commits **nur bis zum Horizont** des jeweiligen Namespace und Branches (`history_days`, `KeepLast`, `KeepMinMax`, `undo_window` 24 h). Ohne diese Grenze wäre jede historische Version dauerhaft erreichbar und Retention wirkungslos. Am Horizont entsteht eine **Shallow-Grenze**: Der älteste behaltene Commit wird materialisiert; seine Eltern-IDs bleiben im Header stehen, ihre Auflösung liefert `beyond_retention`. Ein Horizont wird nie so gewählt, dass eine für offene Branches nötige Merge-Basis entfällt.

Da ein Store Branch-weit gemeinsame Commits hat, gilt für die Commit-Historie das Maximum der betroffenen Namespace-Horizonte; Zeilen kürzer aufbewahrter Namespaces werden über Expunge-by-Retention entfernt.

Zu markieren sind außerdem: ChangeSets, Prolly-Knoten, `cell_ver_key`- und `content_hash`-Zeilen, Blobs, Derivationen lebender Eingaben, Wörterbücher referenzierter Extents, IndexManifeste und Segmente.

### 5.3 Protokoll

```
1 ROOTS      Snapshot S0, Markierungsepoche beginnt
2 MARK       gegen S0
3 REMARK     seit S0 neu entstandene oder wiederverwendete Objekte nachmarkieren
4 CONDEMN    Kandidatenmenge im Manifest durabel veröffentlichen.
             Ab jetzt behandelt die Deduplizierung condemned Objekte als nicht vorhanden
             und schreibt neu. Die Veröffentlichung erfolgt unter derselben Barriere,
             die Wiederverwendung registriert; Wiederverwendungen seit REMARK werden entfernt.
5 COPY       Packs mit Live-Anteil < 50 %: lebende Daten kopieren, Locators umbiegen
6 GRACE      ≥ 1 h und Ablauf aller Epochen/Leases, die vor CONDEMN begannen
7 SWEEP      RemovePack registrieren, dann Unlink, dann Verzeichnis-Sync
```

Schritt 4 schließt das Rennen zwischen Wiederverwendung und Löschung. GC pausiert bei ungeklärten Integritätsfehlern. Eine abgelaufene Lease führt beim nächsten Zugriff zu `snapshot_expired`, nie zu Fehlreads.

---

## 6. Purge (Expunge)

Best-Effort, administrativ, **ohne Löschnachweis**.

```
1 Auswahl auflösen (Cells, alle Versionen, alle Branches, Derivate über Lineage)
2 Blocker melden (Tags, Checkpoints, Backups); ohne force Abbruch
3 Expunge-Frame durabel; ab sofort liefern alle Lesepfade `expunged`
4 Tombstones auf allen betroffenen Branches; abhängige Summaries stale und entfernt
5 Packs ohne die Inhalte neu schreiben (Extents, Blobs, Derivationen, Indizes, Wörterbücher,
  die aus betroffenen Daten trainiert wurden); Idempotenz-/Queue-Zusammenfassungen bereinigen
6 WAL checkpointen, rotieren, alte Segmente entfernen
7 Caches verwerfen; Bericht mit Restorten, die außerhalb der Engine liegen
```

Ein Inhalt wird nur entfernt, wenn keine nicht betroffene Cell denselben `content_hash` nutzt. Prolly-Knoten und Hashes bleiben bestehen; expungte Zeilen sind markiert (INV-9-Ausnahme). **Nicht erfasst:** Backups, Exporte, Replikate, Dateisystem-Snapshots, SSD-Reste, bereits an LLMs gesendete Daten. Der Bericht MUSS das sagen. `purge` über geschützte Branches braucht `Admin`.

---

## 7. Ressourcenmodell (INV-10)

| Klasse | Inhalt | Default |
|---|---|---|
| `fixed` | Locator-Sparse-Index, FST, ANN-Codes, CSR-Offsets, Bitmaps, Katalog | 25 % |
| `memtable` | Deltas und RAM-Indizes aller Branches | 15 % |
| `block_cache` | Buffer Pool | 40 % |
| `query` | Abfragen, Packer, Rerank | 10 % |
| `jobs` | Fold, Compaction, Builds, Ingest | 10 % |

Jede nennenswerte Allokation läuft über eine `BudgetReservation` (RAII). Reicht `fixed` nicht, werden Strukturen seitenweise über den Block Cache geladen oder stärker quantisiert; sonst schlägt der Segmentbau fehl. Query-Reservierungen werden vor Ausführung angefordert.

| Druck | Schwelle | Wirkung |
|---|---|---|
| `Normal` | < 80 % | — |
| `Elevated` | 80–95 % | Ingest drosseln, früher falten, niedrige Jobs pausieren |
| `Critical` | > 95 % oder WAL-/Disk-Reserve unterschritten | neue Writes `backpressure{retry_after_ms}`; Lesen, Fold, Compaction laufen |

Freier Plattenplatz: reservierte Marge (Default 5 % bzw. ≥ 2 GiB) für WAL und GC; Unterschreitung ⇒ `Critical`.

---

## 8. Sicherheit

### 8.1 Bedrohungsmodell

| Angreifer | Abgedeckt | Nicht abgedeckt |
|---|---|---|
| Unberechtigter Client/Agent | ACL, Capabilities, neutrale Fehler | Timing-Seitenkanäle |
| Bösartige Inhalte (Injection) | Trust/Taint, Rahmen, G6, Egress-Guard | vollständige Verhinderung von LLM-Fehlverhalten |
| Diebstahl der Datenträger | AEAD (falls aktiv) | Klartextheader, Dateigrößen |
| Kompromittierter Host-Prozess | — | vollständig außerhalb |
| Manipulation auf Disk | AEAD bzw. Hashprüfung, Signaturen | Rollback auf älteren, in sich gültigen Gesamtzustand |

Im eingebetteten Modus ist der Host-Prozess vertrauenswürdig; Capabilities sind dort eine Disziplin-, keine Sicherheitsgrenze.

### 8.2 Trust und Taint

```rust
#[repr(u8)] pub enum TrustTier {
    Untrusted = 0, AgentInferred = 1, ToolVerified = 2, Curated = 3, Verified = 4, System = 5,
}
bitflags! { pub struct TaintFlags: u16 {
    const EXTERNAL = 1; const INJECTION_SUSPECT = 2; const UNVERIFIED_TOOL = 4; const PII_SUSPECT = 8;
}}
```

- Ein Client kann Trust höchstens bis zur `MaxTrust`-Caveat seiner Capability setzen; `System` nur die Engine.
- Ableitung nach INV-13. Hochstufung nur über G7 oder `Admin`, als Commit mit Attest.
- `Verified` bedeutet menschlich oder redaktionell bestätigt; die Engine prüft keine Wahrheit.
- **Injection-Scan** beim Ingest: Phrasen (de/en), Struktur-Heuristiken, Steuer- und Bidi-Zeichen, Base64 bis Tiefe 2. Policy `quarantine | flag | escalate`. Treffer sind Heuristik; Nicht-Treffer sind keine Entwarnung.

### 8.3 Capabilities

```rust
pub struct Capability {
    pub id: [u8; 16], pub key_id: u32,
    pub caveats: Vec<Caveat>,  // Namespaces, BranchPattern, Ops, Groups, MaxTrust,
                               // NotAfter, TokenBudget, MaxBudgetPerRecall, Audience
    pub mac: [u8; 32],         // m0 = MAC(root, id); m_i = MAC(m_{i−1}, CBE(caveat_i))
}
```

`Ops`: `Read | Write | Fork | Merge | ProtectedBranchWrite | Admin`. Caveats wirken ausschließlich konjunktiv; Attenuierung ist offline möglich, Erweiterung nicht. Unbekannte Caveats ⇒ Ablehnung. Vergleich in konstanter Zeit. Widerruf über `id`-Sperrliste und Root-Key-Rotation. Bearer-Token gehören nie in Prompts, Logs oder gespeicherte Cells.

Branch-Muster werden auf kanonische Namen angewandt; `sess/a1/*` umfasst nicht `sess/a10/…`.

### 8.4 ACL

Auswertung nach [SPEC-02](SPEC-02-DATA-MODEL.md) §9 mit **aktuellen** Gruppenmitgliedschaften. Wirkt vor Ranking, Traversal, Zählung und Rendering. Namespace-Rechte und ACL müssen beide erfüllt sein.

### 8.5 Egress-Guard (`contextra-security`)

Typestate `GuardedPayload<Unsanitized> → GuardedPayload<Sanitized>`; Cloud-Adapter akzeptieren nur `Sanitized`. Prüfungen: Policy `LocalOnly | CloudAllowed{namespaces, min_trust, allow_tainted, destinations}`; Ausschluss `PII_SUSPECT`; Secret-Muster (Schlüssel, Tokens, IBAN) mit Schwärzung oder Block; Raten- und Volumenlimits je Session; Timeout ⇒ `egress_blocked`. `transport()` wird bei Adapterregistrierung vom Betreiber bestätigt, nicht allein vom Adapter behauptet. Query-Texte an Cloud-Embedder unterliegen derselben Prüfung. Mustererkennung ist unvollständig und kein DLP-Ersatz.

### 8.6 Server

TLS Pflicht außerhalb Loopback/Unix-Socket; Authentisierung per mTLS oder Capability; Default `LocalOnly`; Eingabelimits vor Allokation; Rate-Limits je Principal.

### 8.7 Protokolle und Integrität

Strukturierte JSON-Logs für Capability-Entscheidungen, Commits, Gates, Promotions, Egress, Purge — ohne Inhalte und Tokens. Action-Log je Agentenschritt (Tool, Argument-, Ergebnis-, Prompt-Hash) als Cells im Session-Branch. Keine Löschketten- oder Zertifikatsfunktionen.

`ctx verify`: Wurzeln, Hashes, Locator, Alias-Bijektion, Kantenpaare, Bindungen, Watermarks. `ctx scrub`: alle CRCs, AEAD-Tags, BLAKE3 im Hintergrund mit Bandbreitenbudget. Reaktionen: [SPEC-03](SPEC-03-STORAGE-ENGINE.md) §9.

---

## 9. Tiering, Backup

`T0` RAM · `T1` NVMe · `T2` Objektspeicher (**v2**; in v1 nur Port-Trait). Backup: `ctx backup` pinnt einen Manifeststand, kopiert unveränderliche Dateien, WAL-Präfix und Superblock; Restore wird durch `ctx verify` und Öffnen geprüft. Schlüssel werden getrennt gesichert.

---

## 10. Abnahme

- Crash-Injektion an jedem I/O-Schritt von Commit, Fold, Manifest-Rotation, Compaction, GC, Purge, Recovery.
- Sync-Fehler, verlorene Syncs, Torn/Short Writes, `ENOSPC`, Bit-Flips.
- Nonce-Prüfer: kein (Schlüssel, Nonce)-Paar mit zwei Klartexten über alle Simulationsläufe.
- GC-Rennen: Schreiber + Deduplizierung + GC + Abstürze.
- Capability-Fuzzing; ACL- und Egress-Suite.

---

*Weiter: [SPEC-08 APIs und Fehlerkatalog](SPEC-08-API.md)*

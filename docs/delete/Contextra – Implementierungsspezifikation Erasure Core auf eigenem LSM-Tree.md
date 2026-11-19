# Contextra – Implementierungsspezifikation „Erasure Core“ auf eigenem LSM-Tree

**Basis:** `tfufuz1/contextra` @ `9da0eca` (Shell-Commit, 01.11.2026), statisch gelesen. Kein Build, kein Testlauf. Dazu die fünf Anhänge (Fehlerkatalog, Rückbau-Dokument, Löschpipeline-Konzept, LSM-Audit, Rechtssynthese). **Kennzeichnung:** **\[V\]** von mir heute am Code gelesen. **\[D\]** nur aus den Dokumenten übernommen, nicht selbst geprüft. **\[E\]** mein Entwurf oder meine Schlussfolgerung. **Hinweis:** Keine Rechtsberatung. Die Dokumente nennen als Repo-Stand `f653f0f73`; der geklonte Stand ist ein einzelner Squash-Commit. Zeilennummern können abweichen, die zitierten Stellen habe ich selbst gelesen.

---

## 0. Ergebnis in Kürze

1. **Den LSM-Tree behalten, aber mit anderer Begründung als in den Dokumenten.** „redb kann kein Crypto-Shredding“ trägt nicht: Verschlüsselung pro Dokument funktioniert auf jeder Engine. Tragfähig ist etwas anderes: Nur mit eigenem LSM kontrollierst du den **physischen Lebenszyklus** von Schlüsseln, Metadaten, WAL und allen MVCC-Versionen (Zeitreisen) und kannst ihn **atomar über das Manifest** austauschen. Das ist ein Vorteil, sobald es gebaut und mit einem Forensik-Scan belegt ist (Gate G0, Abschnitt 2).
2. **Das heutige Löschprimitiv im Store ist nicht reparierbar, sondern zu ersetzen.** Es ignoriert das Ziel, verschluckt Fehler und erzeugt bei verschlüsseltem Store unverschlüsselte SSTables (Befund N1, neu).
3. **Es gibt Befunde, die in keinem der Dokumente stehen** (N1 bis N10, Abschnitt 1). Zwei davon verändern das Design: Der **Schlüssel-Anker-Effekt** (N10) und die **64-Bit-DocId** (N4).
4. **Reihenfolge:** erst LSM-Härtung (Phase 0), dann das Messinstrument „Erasure Lab“ (Phase 1), **erst danach** die Löschkette (Phase 2). Wer die Löschkette vor dem Messinstrument baut, hat blinde Flecken per Konstruktion.
5. **Neue Löschkette in vier Säulen:** (a) DocDek-Hülle pro Dokument an der Collection/Store-Grenze, (b) **Purge-Compaction** als neues LSM-Primitiv (alle Versionen, exakt nach Schlüsselmenge, streamend, atomar), (c) durable Saga mit ErasedSet und ReadGate, (d) Ebene-A-Adapter je Index und Cache. Ziel: unter 2.500 Zeilen Produktionscode (heute rund 6.700 \[D\]).
6. **Haftungsrahmen:** Alles, was Contextra selbst hält, inklusive Zeitreisen, Checkpoints und Forks. Externe Backups sind außerhalb und werden im Nachweis nur deklariert.

---

## 1. Verifizierter Ist-Zustand

### 1.1 Löschprimitiv und Store (selbst gelesen)

| ID | Befund | Beleg |
| --- | --- | --- |
| F01 \[V\] | `LsmPurger::purge(&self, ctx, _target)` verwirft das Ziel. Filter ist `codec.scan_prefix()` = `t:{tenant}:`. Ein Dokument-Auftrag trifft alle Daten des Mandanten. | `store/src/scanner/purge.rs:36,44` |
| F02 \[V\] | `let _ = builder.add(..)`, `unwrap_or_default()` auf Lesefehlern, `reader.iter()` doppelt voll in den RAM, neuer Reader nur bei Erfolg, danach löscht ein Verzeichnis-Sweep **jede** nicht aktive `.sst`. Fehlschlag bedeutet Datenverlust. | `purge.rs` Block „SSTables filtern“ |
| **N1 \[V\]** | Der Rewrite nutzt `SstableBuilder::create(path)`, das ist `create_with_key_manager(path, None)`; der neue Reader wird mit `None` geöffnet. Bei verschlüsseltem Store entsteht eine Datei ohne Verschlüsselung, die der reguläre Reader nicht lesen kann, oder (bei Klartext-Store) eine zusätzliche Klartextkopie. Außerdem ist `tx_id` fest auf `1` gesetzt. | `builder.rs:229`, `purge.rs` (`add(.., rseq, 1)`) |
| F06 \[V\] | „Nullen“ per `tokio::fs::write(zeros)`: Fehler verworfen, kein `fsync`, kein Vfs. Auf SSD/CoW ohne Wirkung. WAL-Rewrite im selben Muster inklusive `rename` ohne Verzeichnis-`fsync`. | `purge.rs` |
| F16 \[V/D\] | Canary wird in den produktiven WAL geschrieben. | `scanner/wal.rs` |
| **N2 \[V\]** | Bei aktivierter Verschlüsselung werden nur die **Datenblöcke** verschlüsselt (`flush_block`). Der **Index** (letzter Schlüssel je Block, im Klartext) wird direkt geschrieben. Nutzer-IDs bleiben also in Klartext in jeder SSTable, bis die Datei ersetzt wird. Bloom-Filter-Inhalt: nicht geprüft. | `sstable/builder.rs` `flush_block`, `finish` |
| **N3 \[V\]** | Es gibt **einen** `KeyManager` (AES-256-GCM-SIV, 4-Byte-Zufallspräfix + Zähler) für WAL und SSTables des ganzen Stores. Im Hauptpfad existiert kein Schlüssel pro Dokument. `KeyRegistry` (Gruppen à 64) gehört zum KV-Segmentspeicher und wird laut \[D\] vom Engine-Löschpfad nicht aufgerufen. | `crypto/src/crypto.rs:65`, `kv_shredding.rs` |
| **N4 \[V\]** | `DocId = BLAKE3(user_key)[..8]`, ungekeyt. `docid-128` ist ein Feature, nicht Default. Folgen: (a) Bei 10⁹ Dokumenten liegt die Wahrscheinlichkeit für mindestens eine Kollision bei etwa 2,7 % (n²/2⁶⁵) \[E\]. Eine Löschung nach DocId allein kann ein Fremddokument treffen. (b) Die DocId ist ohne Geheimnis aus der Nutzer-ID berechenbar, also **Pseudonym, nicht anonym**. | `types/src/types/domain/ids.rs:225,372`, `types/Cargo.toml:17` |
| **N5 \[V/E\]** | Schlüsselfamilien einer Collection: `0` Nutzerschlüssel, `1` `__docid:`-Mapping, `2` `__rel:`, `3` `__tx_intent:`, dazu `__txt:{collection}:` (Textindex) und `graph:`-Schlüssel. Das sind magische `u8`-Werte. Der Textindex liegt im Namespace desselben Stores. Wie die Postings dort gespeichert sind (pro Term über viele Dokumente), habe ich nicht abschließend gelesen \[E\]. Wenn ja, kann ein DocDek sie nicht abdecken. | `engine/src/collection/mod.rs:507`, `maintenance.rs:857`, `text/src/inverted/purge.rs` |
| **N6 \[V\]** | Die Logik „AEAD-Tail-Toleranz“ steht doppelt in `wal/io.rs` und `wal/replay.rs`. In `replay.rs:35` wird ein CRC-Mismatch immer als Korruption gemeldet, auch am Dateiende. Der Audit-Befund F1 (verschlüsseltes Replay scheitert) ist damit nicht abschließend lokalisiert \[D\]. | `wal/io.rs`, `wal/replay.rs` |
| **N7 \[V\]** | `Collection::delete`: `let _guard = self.kv_locks.lock_for(id).await; drop(_guard);`. Das Lock wird sofort wieder freigegeben. Löschung ist nicht gegen parallele Writes desselben Dokuments serialisiert. | `engine/src/collection/crud/delete.rs` |
| **N8 \[V\]** | Es gibt zwei `Vfs`-Traits (`ports/src/vfs.rs`, `durable-fs/src/vfs.rs`). Das Port-Trait kennt nur `read`, `write_atomic`, `create_dir_all`, `remove_file`, `exists`, `len`, `list_dir`, `sync_dir`. Es fehlen `rename`, Datei-`fsync`, `append`, Bereichslesen. Der Store nutzt kein Vfs, sondern `tokio::fs` direkt. Für die Löschkette reicht das Port-Trait nicht aus. | `ports/src/vfs.rs` |
| **N9 \[V\]** | Nach dem Flush werden alte WAL-Dateien per `remove_file` entfernt; ein Fehler wird nur auf `debug` geloggt. Ein WAL mit Resten kann unbemerkt stehen bleiben. | `store/src/lsm/ops/compaction.rs` (Block „Delete ALL sealed WAL files“) |
| D-C02 \[V\] | `prune_bound = min_read_snapshot().unwrap_or(u64::MAX).min(min_active_seqno())`. Ohne Leser ist das `u64::MAX`. Der Rest der Kette (`ssi.rs`) ist laut \[D\] bestätigt. Blockiert Commits nach dem ersten Flush. | `lsm/ops/compaction.rs` |

### 1.2 Die gelöschten Annahmen der Altplanung, die nicht tragen

| ID | Befund | Konsequenz |
| --- | --- | --- |
| **N10 \[E\]** | **Schlüssel-Anker-Problem.** Ein „KeyTable-Replace“ vernichtet einen DocDek nur dann wirksam, wenn alte Tabellenkopien nicht wiederherstellbar sind. Auf SSD/CoW bleiben alte Sektoren. Ist `KEK_root` (Passphrase) bekannt, entschlüsselt sich jeder alte gewrappte DocDek weiter. Auch ein „rotierender Tabellenschlüssel“ (Konzept 4.5) hilft nur, wenn dieser Schlüssel selbst irgendwo löschbar liegt. Eine Kette von Schlüsseln braucht einen **löschbaren Anker** außerhalb des Flash-Dateisystems (TPM, HSM, OS-Schlüsselbund mit Löschung). | Klasse C ist ohne Anker nur eingeschränkt erreichbar. Spezifikation führt ein `KeyAnchor`-Port ein; ohne Anker wird der Nachweis auf „C-soft“ gedeckelt und `KeyStoreRemanence` als Restrisiko ausgewiesen (4.3). |
| N11 \[E\] | Die Dokumente nehmen an, dass DocDek-Vernichtung die SSTable-Reste „sofort“ unlesbar macht. Das gilt nur für Werte unter dem DocDek. **Schlüssel, Indexeinträge, Termschlüssel und `__docid`-Mappings bleiben Klartext** (N2, N5). Ohne Purge-Compaction bleibt Metadaten-Leakage (Synthese 3.5 Nr. 4). | Purge-Compaction ist Pflicht, nicht optional (4.4). |
| N12 \[D\] | Der Gemini-Teil des LSM-Audits ist durch den Claude-Teil widerlegt (Retention „Off-by-one“, „keine expect“, 8 statt 10 Fehlschläge, „95 % produktionsreif“). Ich folge dem Claude-Teil: **LSM im Kern funktionsfähig, nicht produktionsreif.** | – |

### 1.3 LSM-Stand laut Audit \[D\]

Lib baut mit 1.89; 187 von 197 Unit-Tests grün; Differential-Test gegen redb 3/3 grün (nach Fix des Test-Restarts); F1 verschlüsseltes WAL-Replay scheitert (CRC bei Offset 4 schon am ersten Batch); F2 zwei Sequenznummern pro Commit (2, 4, 6) und nicht spezifiziert; `expect` in `scanner/sstable.rs:35,47`; `HashMap` in `manifest/core.rs`; Clippy, Benchmarks, Forensik-Test nicht gelaufen; #5, #6, #8 ungeklärt.

---

## 2. Entscheidung: LSM behalten, mit Gates

| Argument | Tragfähig? |
| --- | --- |
| „redb hat keine Schlüsselvernichtung“ | **Nein.** Werte pro Dokument zu verschlüsseln geht auf jeder Engine. |
| „redb lässt freie Seiten liegen“ | **Plausibel, nicht belegt** \[D\]. Test unten. Gilt aber auch für den eigenen LSM, solange kein Purge-Compaction existiert. |
| Kontrolle über Schlüssel- und Metadaten-Reste in Dateien (alle MVCC-Versionen, Index, Tombstones) | **Ja**, wenn Purge-Compaction gebaut wird. |
| Kontrolle über WAL-Lebenszyklus und atomaren Manifest-Austausch | **Ja.** |
| Eigene MVCC/SSI-Semantik (Zeitreisen) mit definierter Erase-Wirkung | **Ja.** Hier liegt der eigentliche Alleinstellungswert. |
| Reife und Wartungsaufwand | **Gegen** den Eigenbau. Deshalb Härtung (Phase 0) vor allem anderen. |

**Gate G0 (Forensik-Vergleich, 3 bis 5 Tage):** Gleicher Workload (Insert, Update, Erase, Restart) auf (a) LSM nach Phase 2 und (b) redb mit identischer DocDek-Hülle. Danach Rohscan des Datenverzeichnisses mit dem Forensic Scanner (7.2) nach Schlüssel-Needles, Wert-Needles und Chiffretext. Ergebnis wird als Tabelle `residual_by_layer` festgehalten. Entscheidungsregel: Bleibt redb mit Vacuum auf demselben Restbestand, ist das LSM-Argument Kontrolle und Zeitreisen, nicht Löschwirkung. Dann bewusst weiterentscheiden, mit Zahlen.

**Ausstiegskriterium:** Die Speicherschicht liegt hinter dem Port `StorageEngine` \[V `ports/src/storage.rs`\]. Zeigt die Crash-Matrix aus WP-L2 nach der Härtung weiterhin Recovery-Fehler, ist ein redb-Adapter der Fallback. Die Erase-Logik (DocDek, Saga, Gate) ist dann unverändert, nur Purge-Compaction entfällt.

---

## 3. Haftungsrahmen (was Contextra verantwortet)

| Bereich | Einfluss | Behandlung |
| --- | --- | --- |
| Memtable, WAL, SSTable, Manifest, Blockcache | voll | Ebene A und B, Purge-Compaction |
| **Zeitreisen:** `get_at_seq`, `scan_prefix_at`, `SnapshotRegistry`, `pin_checkpoint`, alte MVCC-Versionen | voll | Erase gewinnt gegen Snapshots (ReadGate), alle Versionen werden entfernt |
| **Checkpoints und Forks** (Hardlink-Klone im Datenverzeichnis, `contextra-checkpoint`) | voll | Sweep je Klon (WP-E8); Hardlinks teilen die Datei, Rewrite im Quellverzeichnis ändert den Klon **nicht** |
| Text-, Vektor-, Graph-Index, Caches (Embedding, KV, Query) | voll | Ebene-A-Adapter mit Messung |
| Audit-Kette, Logs, Proofs, Journal von Contextra | voll | nur Commitments, keine Klartext-IDs |
| Schlüsselmaterial, Prozess (mlock, Coredump, Swap) | teilweise | Hardening, Rest deklariert |
| Exportdateien, die Contextra erzeugt (`engine/src/export.rs`) | voll | in Datenflussinventar, Export trägt Erase-Prüfung |
| **Externe Backups** (Volume-Snapshots, Cloud, Dateikopien) | keiner | im Nachweis als `ExternalCopies` deklariert |
| Externe LLM-Dienste | keiner | deklariert; Egress-Guard bleibt |

---

## 4. Zielarchitektur

### 4.1 Datenflussinventar (jeder Ort hat Mechanismus und Messung)

| # | Ort | Inhalt mit Personenbezug | Mechanismus | Messung (unabhängig vom Mechanismus) |
| --- | --- | --- | --- | --- |
| S1 | Memtable (alle Versionen) | Schlüssel, Werte | exakter Schlüssel-Entzug, dann Flush | Strukturscan + Needle |
| S2 | WAL aktiv und versiegelt | Schlüssel, Werte (bei Store-Verschlüsselung als Batch-Chiffrat) | Rotate, Flush, Entfernen, **Prüfung, dass die Datei fehlt** (N9) | Dateiliste + Rohscan |
| S3 | SSTable-Datenblöcke | Wert-Chiffrat unter DocDek | DocDek-Vernichtung + Purge-Compaction | Rohscan, Entschlüsselungsprobe |
| S4 | SSTable-Index, Bloom, Footer | Schlüssel im Klartext (N2) | **nur** Purge-Compaction | Rohscan nach Schlüssel-Needle |
| S5 | Manifest, Dateinamen | evtl. Schlüsselbereiche (nicht geprüft) | Manifest-Austausch | Rohscan |
| S6 | MVCC-Versionen, SSI-Lesemengen | Schlüssel in S1/S3, RAM | Purge-Compaction über **alle** Versionen | Versionsscan je Seq |
| S7 | Blockcache | entschlüsselte Blöcke im RAM \[E, nicht geprüft\] | Eviction + `zeroize` je ersetzter Datei | Heap-Scan (7.2) |
| S8 | Textindex (`__txt:`) | Terme in Schlüsseln, DocIds in Postings | Rewrite der Postings + Purge-Compaction der Termschlüssel | Posting-Scan + Needle |
| S9 | HNSW (fp32, SQ8, Arena, Snapshotdateien) | Vektor | Slot-Zeroing (bleibt), Rebuild-Schwelle | Slot-Scan + Heap-Scan |
| S10 | Graph, `__rel:` | Kanten, Entitäts-IDs | Eingangskanten bei Erase des Ziels entfernen (Pflicht) | Kantenscan |
| S11 | Caches (Embedding, KV, Query) | Klartext | Register `Purgeable`, Key enthält Scope | Cache-Introspektion |
| S12 | Checkpoints/Forks | Hardlinks auf S3 | Sweep je Klon | Rohscan je Klon |
| S13 | Logs, Audit, Metriken | IDs in `tracing`-Feldern | Typ ohne Freitext, Lint | Log-Scan nach Needle |
| S14 | KeyTable, Schlüssel im RAM, Journal | gewrappte DocDeks | `KeyAnchor`, Replace, `mlock` | Abwesenheitsprüfung |
| S15 | Exporte | Klartext | Pflichtregister | Dateiscan |

Blob-/Originalablage: Ob Contextra Originaldateien selbst hält, habe ich nicht geprüft. Falls ja, ist es Zeile S16 mit derselben Pflicht.

### 4.2 DocDek-Hülle (Collection/Store-Grenze, nicht im LSM)

`KEK_root = Argon2id(Passphrase, Salt; m ≥ 64 MiB, t ≥ 3)` → `KEK_coll = HKDF(KEK_root, "ctx/coll/v1" ‖ tenant ‖ collection)` → `DocDek` je Dokument (32 Byte, `OsRng`).

Wert auf Platte: `magic ‖ fmt_ver ‖ suite_id ‖ key_gen ‖ nonce(12) ‖ ct ‖ tag`. AAD = `tenant ‖ collection ‖ KeyKind ‖ scope_id ‖ fmt_ver`. Damit scheitert jede Vertauschung zwischen Dokumenten oder Mandanten.

- **Warum an der Grenze und nicht im LSM:** Der LSM kennt nur Bytes und einen Store-Schlüssel (N3). Er bleibt dadurch einfach und separat testbar. Die Hülle sitzt im neuen `DocEnvelope`-Adapter zwischen `Collection` und `TenantScopedStorage`.
- **Ersetzung der magischen `u8`** durch `enum KeyKind { User, DocMap, Rel, Intent, Text, Graph, ErasureManifest }`. Jeder `match` ist exhaustiv. Neue Schlüsselfamilie ohne Eintrag im `LayerCatalog` kompiliert nicht (7.4).
- `DocDek` ist `!Clone`, `!Copy`, `Zeroizing`, Zugriff nur über `with_key(|k| ..)`. Die vorhandene `SubKey: Clone`-Implementierung in `kv_shredding.rs` ist als Gegenbeispiel zu entfernen \[V\].
- **Kollisionsschutz (N4):** `scope_id` = 128 Bit aus `BLAKE3-keyed(K_scope, tenant ‖ collection ‖ user_key)`. Das Feature `docid-128` wird im Erase-Profil Pflicht. Beim Insert wird ein vorhandenes `__docid`-Mapping auf gleiche DocId mit anderem Nutzerschlüssel **abgelehnt** (Fail-Closed).
- **Chiffre-Padding** auf Bucket-Größen (Potenzen von 2 ab 256 Byte), Default an, konfigurierbar.

### 4.3 Schlüssel-Anker (löst N10)

```
trait KeyAnchor {
    fn put_secret(&self, id: AnchorId, bytes: &Zeroizing<[u8;32]>) -> Result<()>;
    fn get_secret(&self, id: AnchorId) -> Result<Option<Zeroizing<[u8;32]>>>;
    fn destroy_secret(&self, id: AnchorId) -> Result<DestroyEvidence>; // idempotent
    fn profile(&self) -> AnchorProfile; // Hardware | OsKeyring | FileOnly
}
```

- **KeyTable** (DocDeks gewrappt) liegt als Datei mit festen Slots. Mutation per `write tmp → fsync → rename → fsync Verzeichnis` über das erweiterte Vfs (N8). Tabelle ist mit `KEK_table[g]` verschlüsselt. `KEK_table[g]` liegt **im Anker**, nicht auf dem Dateisystem. Nach jedem Replace wird der Anker-Eintrag der Vorgängergeneration zerstört.
- Profil `Hardware` oder `OsKeyring`: Klasse **C**. Profil `FileOnly` (Default für Soloentwicklung, ohne Zusatzhardware): Klasse **C-soft**, Proof enthält `ResidualRisk::KeyStoreRemanence`. Das ist eine ehrliche Aussage und entspricht der Synthese (5.11).
- Kein Escrow im Kern; ein Recovery-Pfad wird als `EscrowPresent` ausgewiesen (Synthese 8.1).

### 4.4 Purge-Compaction (das neue LSM-Primitiv)

`LsmStorage::erase_keys(&self, keys: &KeyMatcherSet, ctx: &EraseCtx) -> Result<EraseReceipt>` in `store/src/erase/rewrite.rs`.

1. **Barrier:** exklusiv gegen reguläre Compaction (nicht gegen Reads/Writes). Fehlende Barrier = Fehler, nie Warten ohne Frist.
2. **WAL-Drain:** `rotate_and_seal`, Memtables flushen, versiegelte WALs entfernen. **Danach prüfen**, dass die Dateien fehlen. Bleibt eine bestehen, Zustand `Pending`, Wiederholung (N9).
3. **Kandidaten:** SSTables, deren Schlüsselbereich und Block-Index das Ziel schneiden. Bloom nur als Vorfilter. Exakte Bestätigung per Stream.
4. **Streamender Rewrite**: Builder wird aus dem Quell-Reader abgeleitet (`create_like`): gleicher `KeyManager`, gleiche Formatversion, **Originalwerte für `seq`, `tx_id`, Tombstone-Flag** (behebt N1). Es werden **alle Versionen** der Zielschlüssel und deren Tombstones verworfen. Kein `Vec` über die ganze Datei. Jeder Schreibfehler bricht ab, alter Reader bleibt aktiv.
5. **Atomarer Austausch:** neue Datei `tmp` → `fsync` → Manifest-Eintrag „ersetze {alt} durch {neu}“ → `fsync` Manifest → `fsync` Verzeichnis → Reader tauschen. Erst danach alte Datei in das **PendingDelete-Journal** (`durable-fs`), Unlink verzögert bis keine offene Zeitreise sie mehr hält. Alarm bei Überschreitung von `erase_sla`.
6. **Matcher:** `Exact(bytes)` für Nutzerschlüssel (`alice` darf `alice2` nicht treffen). `Prefix{prefix, fixed_suffix_len}` nur für Familien mit fester Länge (`__docid:` + 8 oder 16 Byte). Freie `starts_with` auf Strings sind verboten (behebt F01, C-L26).
7. **Blockcache:** Einträge der ersetzten Dateien entfernen und mit `zeroize` überschreiben (S7).
8. **Messung** übernimmt nicht der Rewriter, sondern der unabhängige Rohscanner (7.2).

**Welche Schlüssel ein Dokument besitzt:** Beim Schreiben legt die Collection pro Dokument einen `ErasureManifest`-Eintrag (`KeyKind::ErasureManifest`, unter DocDek) in derselben Transaktion ab. Er listet alle Hilfsschlüssel (Rel, Intent, Termschlüssel, Graph-Entität, Vektor-Slot). Der Erase liest ihn **vor** der Schlüsselvernichtung. Fehlt er (Altbestand), verweigert Erase mit `NoManifest`, außer der Aufrufer fordert ausdrücklich `legacy_scan` (langsamer Vollscan). Das macht das Datenflussinventar ausführbar und testbar (7.4).

### 4.5 Saga, Journal, ErasedSet, ReadGate

**Zustände** (rein, total, ohne I/O): `Requested → Frozen → LogicallyDeleted → PlaintextPurged → PlaintextMeasured → KeyDestroyed → Measured → Completed`, Nebenzweig `Frozen → Restricted → (Release) → Requested`. Übergangsfunktion `ErasureState::step` wie im Konzept 4.3, aber mit **Fehlern als Wiederholung** (`attempt`, `next_retry_at`) im Journal, nie als `Ok(verified:false)`.

**Journal:** eigene Datei über Vfs, nicht im LSM-WAL (keine Rekursion, eigener Lebenszyklus). Eintrag: `erasure_id (128 Bit zufällig) ‖ state ‖ attempt ‖ Zeit (Clock-Port) ‖ payload`. Der **Scope**, der zum Fortsetzen nötig ist, liegt als `AEAD(K_saga[erasure_id], ScopeDescriptor)` darin. `K_saga` liegt im Anker und wird bei `Completed` zerstört. Danach enthält das Journal keinen Personenbezug mehr (Lösch-Paradoxon, Synthese 6.4).

**ErasedSet:** durabel, Einträge `SubjectCommitment = BLAKE3-keyed(K_commit[tenant, epoch], canonical(scope))`. Die Synthese nennt HMAC-SHA256; beides sind PRFs, ich wähle BLAKE3-keyed wegen der Latenz im Lesepfad (Gate: ReadGate-Overhead ≤ 3 % p50 auf `get`). `K_commit`-Lebensdauer ≥ `erased_set_retention`, validiert beim Start.

**ReadGate (eine Engstelle):** `TenantScopedStorage` ist der einzige Zugang der Engine zum Store \[V `collections.rs:58,271`\]. Dort wird **vor jedem Lesen** geprüft: `get`, `get_at_seq`, `scan_prefix`, `scan_prefix_at`, getrackte Varianten. Treffer liefert `Erased`, auch bei Zeitreisen („Erase gewinnt“). Der rohe `LsmStorage` ist für Reads von außen `pub(crate)`; ein Architekturtest verbietet andere Zugänge. Damit **blockieren offene Snapshots die Löschung nicht mehr** (behebt F09, C-L17).

**Recovery:** `recover()` läuft vor Freigabe des Lesepfads: Journal lesen, ErasedSet füllen, offene Sagas fortsetzen. Unklarer Zustand heißt Lesen verweigern.

**Nebenläufigkeit:** Erase serialisiert pro Dokument mit demselben Lock wie Writes. Der Lock-Bug N7 ist Vorbedingung.

### 4.6 Ports (async, abweichend vom Konzept)

Das Konzept (D2) empfiehlt synchrone Ports. **Ich empfehle async mit `BoxFuture`**, wie alle bestehenden Ports (`ports/src/storage.rs`). Synchrone Ports über asynchroner I/O erzwingen `block_on` (F12, C-L18, C-L19) und waren die Ursache der Deadlock-Risiken. Neue Ports (ADR nötig, `AGENTS.md` §7): `ErasureJournal`, `KeyAnchor`, `LayerPurger`, `LayerScanner`, `ErasedSetView`; Vfs wird um `rename`, `sync_file`, `open_append`, `read_at` erweitert und **ein** Vfs-Trait bleibt (das zweite in `durable-fs` wird entfernt oder re-exportiert).

### 4.7 Ebene A: Adapter je Schicht

Jede Schicht bekommt einen `LayerPurger` **und** einen `LayerScanner` mit typisiertem `ErasureScope`. Beide sind getrennt implementiert; der Purger kennt den Scanner nicht.

| Schicht | Bestand im Repo \[D, nicht neu gelesen\] | Aktion |
| --- | --- | --- |
| Textindex | `text/src/inverted/purge.rs`, `scanner.rs` mit `block_on` je Dokument und Term | Logik als Vorlage, `block_on` entfernen (async), Postings im Fenster-Rewrite, Termschlüssel über Purge-Compaction, Termliste aus dem ErasureManifest |
| HNSW | `hnsw/purge.rs` mit Slot-Zeroing fp32 und SQ8 | **behalten**; `hnsw/deletion.rs` (Kantenreparatur, Entry-Point-Neuwahl) **entfernen** (VETO-F02: kein Rewiring); Rebuild bei `tombstone_rebuild_ratio` 0,2 im Hintergrund; Arena-Reuse erst nach Rebuild; Snapshotdateien über Vfs |
| Graph | Entitäts- und `__rel:`-Schlüssel | Eingangskanten bei Erase des Ziels entfernen; ohne diesen Schritt bleibt der Bezug im Nachbardokument |
| Caches | verteilt (kvcache, Embedding, Query) | gemeinsames Register `Purgeable`; Registrierung Pflicht; Test iteriert Register gegen `LayerCatalog` |
| Blockcache | `store/src/sstable/block_cache.rs` | Eviction mit `zeroize` bei Dateiersetzung |
| Checkpoints | `contextra-checkpoint` | WP-E8 |

### 4.8 Nachweis (DeletionProof v4)

- **Ein** Format, nur Ed25519 (asymmetrisch, extern prüfbar). V1 bis V3 und `ed25519_proof.rs` entfallen. Es gibt keine Legacy-Verifikation.
- Inhalt: `SubjectCommitment`, `ErasureClass` als Bitset (L, P, C oder C-soft, E), je Schicht `LayerWitness`-Digest, KDF-/Suite-IDs, `ResidualRisk`-Liste (`KeyStoreRemanence`, `ExternalCopies`, `EscrowPresent`, `SwapOrDumpPossible`, `CryptoObsolescence`, `UnmeasuredLayer(id)`), Zeit über `Clock`. **Kein** Feld mit Klartext-ID, Inhalt, Chunk-Hash oder DocId (N4).
- Der Proof wird aus `Evidence` durch die reine Funktion `classify` berechnet. `Evidence` entsteht ausschließlich aus `LayerWitness`en, deren Positivkontrolle bestätigt ist. Eine Schicht ohne Witness senkt die Klasse automatisch (N-09 des Rückbau-Dokuments).
- **Jedes exportierte Feld ist signiert**, der Verifier lehnt unbekannte Felder ab. Merkle-Pfade binden Einzelwitnesses und Schichtliste an die Wurzel (behebt R-13, C-L02).
- Veröffentlichter Verifier (`examples/verify_deletion_proof_external.rs`) mit Testvektoren.

### 4.9 Aufbewahrung (v1 minimal)

`erase(scope, on_retention: Reject | Restrict)`. `Restrict` setzt den Zustand `Restricted`: aus dem Retrieval-Index isoliert, **kein** Purge, **keine** Schlüsselvernichtung, API und Proof sagen „eingeschränkt“, nie „gelöscht“ (ein `Display`-Typ erzwingt das Wort). Einen Fristen-Scheduler gibt es in v1 nicht; ohne gesetzte Policy meldet das System `PolicyMissing`, die Verantwortung bleibt beim Anwender (Synthese 8.3).

---

## 5. Umbau am Quellcode

| Pfad | Aktion | Grund |
| --- | --- | --- |
| `store/src/scanner/purge.rs`, `scanner/wal.rs` | **löschen** | F01, F02, N1, F06, F16; WAL-Rewrite entfällt (Drain und Entfernen genügen) |
| `store/src/scanner/{sstable,memtable}.rs`, `scanner/tests.rs` | **ersetzen** durch `store/src/erase/scan_raw.rs` | Mandantenpräfix-Scan, `block_on_future` mit `expect` (F4) |
| `store/src/kv/*`, `crypto/src/{kv_segment,kv_cipher,kv_shredding,revocation_log}.rs` | **Entscheidung E6**; Empfehlung: entfernen, falls `cargo tree` keinen Konsumenten zeigt | dritte KV-Variante, Gruppen à 64, `SubKey: Clone` |
| `crypto/src/deletion_proof/`, `ed25519_proof.rs` | **ersetzen** durch `proof_v4.rs` | V1/V2 symmetrisch, Feld `integrity_warning` unsigniert \[D\] |
| `crypto/src/audit_chain.rs` | **behalten** | Hash-Kette (Positivbefund), Abhängigkeit von `KeyRegistry` prüfen \[D\] |
| `engine/src/contextra_impl/erasure.rs` | **ersetzen** durch `engine/src/erasure/{saga,gate,recovery}.rs` | 616 Zeilen, `verified: bool`, keine Saga \[D\] |
| `engine/src/collection/crud/delete.rs` | **umbauen** | Lock-Bug N7; logische Löschung bleibt als Baustein, Name `logical_delete` |
| `store/src/tenant_codec.rs` | **umbauen** | `TenantScopedStorage` wird ReadGate-Träger |
| `engine/src/collection/mod.rs:507` | **umbauen** | `key_type: u8` wird `KeyKind` |
| `store/src/sstable/{builder,reader}.rs` | **erweitern** | `create_like`, Streaming-Iterator mit Seq und Flags |
| `store/src/manifest/*` | **erweitern** | Eintrag „ersetze {alt} durch {neu}“ atomar |
| `store/src/lsm/ops/compaction.rs` | **umbauen** | WAL-Entfernung mit Prüfung (N9), Sentinel-Fix (D-C02) |
| `ports/src/scanner/*`, `types/src/witness.rs` | **ersetzen** | `ScanTarget` mit String-ID, Purger synchron |
| `ports/src/vfs.rs`, `durable-fs/src/vfs.rs` | **zusammenführen** | N8; `scrub.rs` und `PendingDeleteJournal` nach Test als Baustein |
| `hnsw/deletion.rs` | **entfernen** | Rewiring, C-L22 |
| `mcp`: `contextra_delete`, `handle_drop_collection`, `proof_key*.rs` | **sofort deaktivieren**, später an neue Saga hängen | `_proof_key` und `_tenant_id` werden verworfen (C-L04 \[D\]) |
| `docs/DELETION-DSGVO.md`, `SPEC.md`, README | **Status „Konzept“**, Aussagen bis Release-Gate zurückhalten | Wording-Lint (T-LINT-1) |

**Neue Dateien** (keine neuen Crates, `AGENTS.md` §5.2): `types/src/erasure/{scope,state,class,evidence,catalog}.rs`; `ports/src/erasure.rs`; `crypto/src/{docdek,envelope,keytable,commit_key,proof_v4}.rs`; `store/src/erase/{barrier,rewrite,scan_raw,journal,keytable_store}.rs`; `engine/src/erasure/*`; `testkit/src/erasure_lab/*`. **Budget:** unter 2.500 Zeilen Produktionscode; die Zahl wird als CI-Gate gemessen, damit der Umfang nicht wieder wächst.

**Sofortmaßnahme vor allem anderen (WP-L0b):** die MCP-Löschwerkzeuge und `Contextra::erase_*` auf `Unsupported` setzen, `drop_collection` und `delete` als „nur logisch“ kennzeichnen. Das stoppt den Datenverlustpfad F02 sofort und ist die Übergangsregel aus Abschnitt 9 des Rückbau-Dokuments.

---

## 6. Phase 0: LSM-Härtung (Voraussetzung für alles Weitere)

| WP | Inhalt | Abnahme (messbar) |
| --- | --- | --- |
| L0 | Basislinie: `cargo check --workspace --all-targets --locked`, `cargo test -p contextra-store`, Toolchain 1.89 in CI. Die Datei `kv_crypto_shred_instant_unrecoverable.rs:32` (`assert!(revoked)` auf `Result`) reparieren oder entfernen \[D\]. | CI grün, Zahl der übersprungenen und `#[ignore]`-Tests als Zahl im Report (20 \[D\]) |
| L1 | **D-C02** Sentinel: `prune_bound` auf höchste committete Seq begrenzen; `prune_through` ohne stilles Maskieren. | Regressionstest: Flush ohne Snapshot, danach Commit mit frischem Snapshot gelingt |
| L2 | **WAL F1 und N6**: eine Replay-Implementierung statt zwei, CRC-Mismatch am Tail wie Torn-Write behandeln, Ursache des CRC-Fehlers bei Offset 4 im verschlüsselten Fall lokalisieren (Schlüssel-/Salt-Ableitung Schreiber vs. Leser prüfen). | T-WAL-1: Abschneiden an **jedem** Byte-Offset, plain und verschlüsselt, V2 und V3: Open gelingt, Präfix wird wiederhergestellt. T-WAL-2: Paritaet der Replay-Pfade |
| L3 | **F2 Sequenzsemantik** entscheiden. Empfehlung: Sichtbarkeit einer Transaktion ist die Seq ihres `TxEnd`; die Seq der Einzeloperationen ist intern. In `SPEC` festschreiben, Tests #2 und #7 anpassen. | Property-Test: Snapshot-Sichtbarkeit gegen Referenzmodell |
| L4 | **D-C01** Follower-Commit cancel-safe (Guard, Timeout auf `rx.await`). | Chaos-Test mit Abbruch an jeder `await`-Stelle |
| L5 | **B-S01/B-S02/B-S04** Argon2id im Produktionspfad, Verschlüsselung Standard **an** (explizites Opt-out), `MIN_M_COST_KIB` ≥ 64 MiB. Ohne das liegt der WAL im Klartext und die Erase-Aussage ist wertlos. | Test: Default-Builder erzeugt keine Klartextdatei (Rohscan) |
| L6 | Vfs vereinheitlichen und erweitern (N8), zunächst nur für neue Erase-Module und die WAL-Entfernung nach Flush. Lint verbietet `tokio::fs` und `std::fs` in `store/src/erase/`. | ADR, Lint-Test |
| L7 | Testhygiene laut Audit: #1, #3, #4, #10 (Nicht-root), #5/#6/#8 untersuchen; `expect` in `scanner/sstable.rs` entfällt mit Ersetzung; `HashMap`→`AHashMap` in `manifest/core.rs`; Clippy `unwrap_used`, `expect_used`, `indexing_slicing` im Store erzwingen. | Store-Suite grün, Clippy `-D warnings` |

---

## 7. Phase 1: Erasure Lab (Test, Messung, Verifikation)

### 7.1 Prinzipien

1. **Unabhängige Messung:** Der Scanner benutzt keinen Code des geprüften Mechanismus (kein SSTable-Reader, kein WAL-Parser). Er sucht in Rohbytes.
2. **Positivkontrolle immer:** Jeder Messlauf enthält einen bekannten Treffer. Findet der Scanner ihn nicht, ist der Lauf ungültig.
3. **Erst erzeugen, dann löschen:** Jedes Needle entsteht reproduzierbar (Seed) und wird protokolliert, wo es geschrieben wurde.
4. **Deterministisch und fehlerinjizierbar:** Zeit, Zufall, Dateisystem laufen über Ports (Regel 3 und 4 der `AGENTS.md`).
5. **Jeder Lauf erzeugt Evidenz** (`erasure-evidence.json`: Klasse, Witness je Schicht, Restliste). Das ist derselbe Datensatz, aus dem der Proof entsteht.

### 7.2 Komponenten (in `contextra-testkit`)

| Komponente | Zweck | Fehlerklasse, die sie sichtbar macht |
| --- | --- | --- |
| `FaultVfs` | Absturz nach Operation *n*, abgerissene Writes an Byte *b*, `ENOSPC`, `EIO` bei `fsync`, Verlust ungesyncter Daten, nicht atomares `rename` | Absturz- und Teilfehler (F02, C-L15, N9) |
| `ForensicScanner` | Rohscan des Datenverzeichnisses nach Needles: Schlüsselbytes, Wertbytes, DocId (LE-Bytes), UTF-8 und UTF-16LE, Base64, Chiffretext-Präfixe; je Datei und Offset; berücksichtigt Hardlink-Klone | Reste in S1 bis S5, S12 |
| `HeapScanner` (Linux) | liest `/proc/self/mem` nach Needles inklusive fp32-Vektormuster | RAM-Reste S7, S9, S11 |
| `CanaryFactory` | eindeutige, hochentropische Needles je Dokument und Feld | Zuordnung von Rest zu Quelle |
| `ReferenceModel` | In-Memory-Orakel (BTreeMap, ErasedSet), gleiche Operationsfolge wie das System | Abweichungen im sichtbaren Verhalten |
| `RecordingStorage` | Dekorator, der jeden geschriebenen Schlüssel je öffentlicher Operation protokolliert | **blinde Flecken**: Schlüssel ohne Katalogeintrag (7.4) |
| `NeighborDigest` | Hash über alle `(key, seq, flags, value)` der Nicht-Ziel-Paare über alle Dateien und Versionen | Kollateralschaden (F01) |
| `LeakyPurger` (Mutant) | absichtlich fehlerhafter Purger | **Selbsttest** des Labs (T-RAW-2) |
| Mutation | `cargo-mutants`-Einträge in `mutants-guarantees.toml` für `step`, `classify`, Matcher, Barrier, Anker | Testqualität, Gate ≥ 95 % |
| `loom` | ErasedSet, ReadGate, Barrier | Nebenläufigkeit |
| Fuzz | Journal-Replay, KeyTable-Parser, SSTable-Index, WAL-Frame | Parser-Robustheit |

### 7.3 Testmatrix (Auswahl der Pflichttests)

| ID | Prüft | Typ |
| --- | --- | --- |
| T-ISO-1 | Erase von A lässt `NeighborDigest` bitgleich (gleiche und andere Collection, anderer Mandant) | Property, tausende Seeds |
| T-ISO-2 | IDs, die Präfixe voneinander sind (`alice`/`alice2`, Doc `1`/`10`) | gezielt |
| T-ISO-3 | Erzwungene DocId-Kollision: Insert wird abgelehnt; Erase trifft nur den Scope (N4) | gezielt mit Test-Hook |
| T-VER-1 | Nach Erase liefert **jeder** Lesepfad `Erased`: `get`, `scan`, `get_at_seq` über alle Seqs, `scan_prefix_at`, Checkpoint, Fork | exhaustiv |
| T-RAW-1 | Needles in Datenverzeichnis nach jedem Saga-Zustand; erlaubte Restliste je Klasse | Rohscan |
| T-RAW-2 | `LeakyPurger` wird entdeckt, sauberes System besteht | Lab-Selbsttest |
| T-RAW-3 | Heap-Scan nach Ebene A (HNSW, Blockcache, Text) | Rohscan RAM |
| T-ENC-1 | Neu geschriebene SSTable ist mit regulärem Reader lesbar und hat dieselbe Verschlüsselung (N1) | Regression |
| T-ENC-2 | Chunk von A unter DocDek von B entschlüsseln scheitert (AAD) | Sicherheit |
| T-CRASH-1 | Absturz nach **jeder** Vfs-Operation der Saga (enumeriert, nicht zufällig); nach `recover()` Zustand terminal erreichbar, **nie** Lesezugriff auf `Frozen`-Scope | exhaustiv |
| T-CRASH-2 | Abgerissener Write an jedem Byte von Journal, KeyTable, Manifest | exhaustiv |
| T-CRASH-3 | `ENOSPC` und `EIO` bei `fsync` im Rewrite: Zustand `Pending`, Nachbarn unverändert (F02) | gezielt |
| T-KEY-1 | Nach Anker-Vernichtung scheitert Entschlüsselung der Canary-Probe; `destroy` idempotent | gezielt |
| T-SAGA-1 | `step` für alle 81 Kombinationen; `classify` monoton; E impliziert L, P, C | Property |
| T-CONC-1 | Erase parallel zu Put desselben Dokuments, Erase mit offenem Snapshot, Barrier gegen Compaction | loom + Stress |
| T-DIFF-1 | Zufallsfolgen (Insert, Update, Flush, Compaction, Restart, Erase) gegen `ReferenceModel` | Differential |
| T-WAL-1/2 | siehe L2 | exhaustiv |
| T-COV-1 | Alle Schlüssel jeder öffentlichen Collection-Operation liegen im `LayerCatalog` und im `ErasureManifest` | RecordingStorage |
| T-LINT-1 | kein `tokio::fs`/`std::fs`, kein `unwrap`/`expect`/`let _ =` in Erase-Modulen; Wording-Lint für „garantiert“, „DSGVO-konform“, „informationstheoretisch“ | Lint-Test |
| T-PROOF-1 | Änderung **jedes einzelnen Bytes** des exportierten Proofs wird abgelehnt; Testvektoren für den externen Verifier | Mutation |

### 7.4 Wie blinde Flecken ausgeschlossen werden

- **Kompilierzeit-Abdeckung:** `KeyKind` (4.2) und `LayerCatalog` sind exhaustiv. Neue Schlüsselfamilie ohne Eintrag kompiliert nicht.
- **Laufzeit-Abdeckung:** `RecordingStorage` meldet jeden Schlüssel, den eine Operation schreibt und der nicht im Katalog oder im ErasureManifest steht (T-COV-1).
- **Scanner-Abdeckung:** `LeakyPurger` und ein absichtlich lecker gebautes Fixture pro Schicht prüfen, dass der Scanner Reste auch wirklich meldet.
- **Zeitreisen:** T-VER-1 iteriert alle Sequenznummern, nicht nur die letzte.
- **Deklarierte Grenze statt Schweigen:** Was nicht messbar ist (Swap, Kernel-Pagecache, SSD-Controller), steht als `ResidualRisk` im Proof.

### 7.5 Benchmarks und Gates

| Messgröße | Wie | Gate |
| --- | --- | --- |
| Erase-Latenz p50/p99 nach Klasse L, P, C | bei 10⁴, 10⁶, 10⁷ Schlüsseln | Zeit bis L ≤ 10 ms; P und C ohne Anstieg proportional zur Gesamtmenge (Rewrite nur betroffene Dateien) |
| Schreibverstärkung der Purge-Compaction | Bytes geschrieben je entferntem Byte | Budget festlegen, Regression ≤ 5 % |
| RSS-Spitze beim Rewrite | kein Vollladen (`iter()`) | ≤ Konstante × Blockgröße |
| ReadGate-Overhead | `get` und `scan` mit und ohne Gate | ≤ 3 % p50 |
| `fsync` je Erase | Zähler im `FaultVfs` | obere Schranke je Zustand |
| Batch-Erase | 1.000 Dokumente | Durchsatz und Anzahl Dateirewrites |
| Regression Bestand | `lsm_concurrency_bench`, `wal_bench`, `sstable_bench`, `redb_comparative_bench` \[V vorhanden, nie gelaufen\] | ≤ 5 % |

Der Bench in `benchmarks/contextra-bench` misst Retrieval-Qualität, nicht den Store \[D\]; die Store-Benchmarks bleiben getrennt. Der Bench-Crate ist laut \[D\] nicht kompilierbar (A-B06) und wird erst nach Reparatur Gate.

---

## 8. Arbeitspakete und Reihenfolge

Aufwand \[E, grobe Schätzung für Soloentwicklung mit KI, Teilzeit\]: S = 1 bis 3 Tage, M = 1 bis 2 Wochen, L = 3 bis 5 Wochen.

| Phase | WP | Inhalt | Abhängig von | Aufwand |
| --- | --- | --- | --- | --- |
| 0 | L0b | Löschwerkzeuge deaktivieren, „nur logisch“ kennzeichnen | – | S |
| 0 | L0 bis L7 | siehe Abschnitt 6 | L0b | L bis L+M |
| 1 | X0 | `FaultVfs` und Vfs-Erweiterung (ADR) | L6 | M |
| 1 | X1 | `ForensicScanner`, `HeapScanner`, `CanaryFactory`, Selbsttest (T-RAW-2) | X0 | M |
| 1 | X2 | `ReferenceModel`, `RecordingStorage`, `KeyKind`-Refactor, `NeighborDigest` | L3 | M |
| 1 | X3 | Evidenz-JSON, Bench-Basislinien, CI-Verdrahtung | X1, X2 | S bis M |
| 2 | E1 | Typen und Ports (Scope, State, Class, Evidence, Catalog, Ports) | X3 | M |
| 2 | E2 | `DocDek`, Envelope, `KeyAnchor`, KeyTable | E1 | M bis L |
| 2 | E3 | Collection-Integration: `DocEnvelope`, ErasureManifest, `docid-128`, Kollisionsschutz, Lock-Fix N7 | E2 | M |
| 2 | E4 | Store: Barrier, `create_like`, Streaming-Rewrite, Manifest-Austausch, PendingDelete | L2, X1 | L |
| 2 | E5 | ReadGate (inklusive Zeitreisen) | E3 | S bis M |
| 2 | E6 | Saga, Journal, ErasedSet, `recover()` | E4, E5 | M bis L |
| 2 | E7 | Ebene-A-Adapter (Text, HNSW, Graph, Caches, Blockcache) | E6 | L |
| 2 | E8 | Checkpoints und Forks (Sweep je Klon) | E4 | M |
| 2 | E9 | Proof v4, Verifier, Testvektoren | E6 | M |
| 2 | E10 | Prozess-Hardening, Retention minimal, MCP/API an Saga | E6 | M |
| 2 | E11 | Altcode entfernen, Docs und Wording-Lint | E9 | S bis M |
| 2 | E12 | **Gate G0** (Forensik-Vergleich), Release-Gate, externes Review beauftragen | E11 | M |

**Gesamt \[E\]:** etwa 5 bis 8 Monate Teilzeit. Die Schwankung hängt an L2 (WAL-Ursache offen) und E7 (Text-Postings, nicht abschließend gelesen). Jedes WP ist ein eigener PR mit höchstens einem Crate-Schnitt und einem **zuerst roten** Test (Regel 12 der `AGENTS.md`).

---

## 9. Scope-Reduktion für ein auslieferbares Produkt

Der Workspace hat 35 Crates \[V\] (Doku spricht von 15). Für einen Soloentwickler ist das nicht pflegbar. Empfehlung:

1. **Erstprodukt definieren:** eingebettete Rust-Bibliothek „Insert, Search, Erase mit Nachweis“. Alles ohne Konsumenten in diesem Produkt wird **eingefroren** (aus `members` genommen, Code bleibt im Repo).
2. **Kern (muss bleiben, vorbehaltlich Abhängigkeitsprüfung):** `types`, `ports`, `sys`, `crypto`, `store`, `mvcc`, `engine`, `vector`, `text`, `checkpoint` (Zeitreisen und Forks), `testkit`, Fassade `contextra`, `durable-fs` (in `ports` aufgehen).
3. **Kandidaten zum Einfrieren:** `agent`, `cognition`, `adapt`, `router`, `rank`, `license`, `sandbox`, `infer-candle`, `infer-ollama`, `infer-onnx`, `kvcache`, `audit-export`, `avv-generator`, `py`, `mcp`, `bench`. Ob `graph`, `ingest`, `privacy`, `db`, `core`, `wire` bleiben, hängt von E-Entscheidungen ab (Abschnitt 11).
4. **Verfahren (ich konnte es nicht ausführen):** `cargo tree -p contextra -e normal --prefix none | sort -u` liefert den echten Kern. Jedes Crate außerhalb dieser Liste braucht eine ausdrückliche Begründung, sonst wird es eingefroren.
5. **Produktaussage erst nach Release-Gate E12:** Bis dahin nur „logische Löschung“ und „Löschung in Entwicklung“.

---

## 10. Rechtsabbildung (Anforderung → Mechanismus → Test → Restrisiko)

Quelle ist die beiliegende Synthese; sie ist laut eigener Angabe nur teils am Primärtext verifiziert. Keine Rechtsberatung.

| Anforderung (Synthese) | Mechanismus | Test | Deklariertes Restrisiko |
| --- | --- | --- | --- |
| Unverzüglich, Retry statt Abbruch (Art. 12, 17) | Saga mit Retry, `erase_sla`, Alarm | T-CRASH-\*, T-CONC-1 | Offene Zeitreise verzögert nur den Unlink, nicht die Wirkung |
| Unkenntlichmachen (SDM), Ebene A und B (0.2 Nr. 4) | Ebene-A-Adapter, DocDek, Purge-Compaction | T-RAW-1, T-RAW-3 | RAM/Swap, SSD-Controller |
| Granularität je Dokument (3.5 Nr. 1) | DocDek je Dokument; `scope_id` 128 Bit | T-ISO-1/2/3 | Betroffener mit mehreren Dokumenten braucht `SubjectIndex` (E-Entscheidung) |
| Kein Parallelbestand (3.5 Nr. 3) | Verschlüsselung Standard an, Cache-Register, Index-Purge | T-RAW-1, T-LINT-1 | Externe LLM-Anbieter |
| Metadaten ohne Re-Identifizierung (3.5 Nr. 4) | Padding, gekeyte Commitments, Schlüssel-Entfernung durch Purge-Compaction | T-RAW-1 (Schlüssel-Needle) | Bucket-Länge, Zeitstempel |
| Alle Schlüsselkopien vernichten (3.5 Nr. 2, 7) | `KeyAnchor`, Replace, kein append-only Schlüssellog | T-KEY-1 | **`KeyStoreRemanence` im Profil `FileOnly`** (N10) |
| Nachweis ohne Klartext-ID (5.7, 6.4) | `SubjectCommitment`, `K_saga`, `K_commit`-Lebensdauer | T-PROOF-1, Journal-Scan | Commitment bleibt pseudonym, solange `K_commit` existiert |
| Zeitreisen und Checkpoints | ReadGate, Versionen und Klone bereinigt | T-VER-1 | – |
| Backups | **außerhalb**; Proof nennt `ExternalCopies` | – | Rechtlich bei Anwender und Betreiber des Backups |
| Aufbewahrung (5.9) | `Restricted`, kein Purge | T-SAGA-1 | Fristenverwaltung beim Anwender |
| Kein Absolutversprechen (0.2 Nr. 11, 12) | Klassen L/P/C/C-soft/E, Wording-Lint | T-LINT-1 | – |

**Rechtsfragen, die ein Fachanwalt oder die DSB klären muss, bevor eine Produktaussage entsteht:** Gilt Schlüsselvernichtung als Löschung (offen laut Synthese 0.2 Nr. 2)? Sind Embeddings personenbezogen (E-4)? Welche Rolle hat der Anbieter bei lokal installierter Software (E-8)? Ist ein Commitment mit `K_commit` als Pseudonym zulässig und wie lange? Freigabe der Vorlagen (Art. 30, AVV).

---

## 11. Offene Entscheidungen und Grenzen

### 11.1 Entscheidungen für dich

| # | Frage | Empfehlung |
| --- | --- | --- |
| E1 | Sollen Nutzer-IDs im Schlüssel blind (HMAC) gespeichert werden? Verlust: Präfix- und Bereichsscans nach Nutzer-ID. Gewinn: kein Klartext-Rest in S4 | v1: nein, Purge-Compaction reicht; als Collection-Option (`opaque_ids`) vormerken |
| E2 | `docid-128` als Default im Erase-Profil | **ja** |
| E3 | `KeyAnchor`-Profil v1 | `FileOnly` plus Schnittstelle für OS-Schlüsselbund; Hardware später |
| E4 | Ports async oder sync | **async** (4.6) |
| E5 | Zeitreise-Semantik beim Erase | **Erase gewinnt** (ReadGate), Dateien werden verzögert entfernt |
| E6 | Schicksal von `kv_segment`, `kv_cipher`, `kvcache`, `KeyRegistry` | entfernen, wenn kein Konsument |
| E7 | Textindex: Postings im LSM belassen oder eigenes Segmentformat | erst nach Lesen von `text/src/inverted` entscheiden (WP-E7) |
| E8 | `graph` im Erstprodukt | nur wenn `relate` im Erstprodukt gebraucht wird; sonst einfrieren |
| E9 | Gibt es ausgelieferte V1 bis V3 Beweise (J-08)? | falls ja, Anwender informieren |
| E10 | Externes Krypto-Review (J-02) | **nach** E9 und vor jeder öffentlichen Löschaussage |

### 11.2 Was ich nicht geprüft habe

- **Kein Compiler, kein Testlauf.** Alle Test- und Benchmarkaussagen stammen aus dem LSM-Audit \[D\] oder sind Spezifikation.
- Nicht gelesen: `ssi.rs`, Text-Postings-Format, `hnsw/purge.rs` und `deletion.rs`, Inhalt von Bloom und Manifest (speichert es Schlüsselbereiche?), Blockcache (entschlüsselt?), ob eine Originaldateiablage existiert, `kvcache`, `graph`, `checkpoint/orphan`.
- Zahlen aus den Dokumenten (6.700 Zeilen, 187/197, 20 `#[ignore]`) habe ich nicht nachgezählt.
- Die Kollisionswahrscheinlichkeit 2,7 % ist eine Rechnung (n²/2⁶⁵ bei n = 10⁹), keine Messung.
- N10 ist eine Schlussfolgerung aus der Dateisystem- und SSD-Eigenschaft, kein Test.

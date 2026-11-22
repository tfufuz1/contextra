# contextra-storage: Refaktorisierungsplan v2 (Fundament-Crate)

| | |
|---|---|
| Status | Entwurf zur Umsetzung, **Überarbeitung und Vertiefung von v1**. Entscheidungen final (`SD-xx`), Annahmen markiert (`A-xx`), Leistungsaussagen sind **Hypothesen** (`H-xx`). Offene Entscheidungen sind als `O-xx` mit Empfehlung geführt. |
| Gegenstand | Crate `crates/contextra-store` (Repository `tfufuz1/acta`, Stand `5e9d44a`; Code identisch zum Stand `47c9865` von v1), künftig **`contextra-storage`** |
| Grundlage | v1 des Plans, `REVIEW_contextra-storage_vs_mnemos.md`, `MNEMOS_CONCEPT.md`, `Optimal-LLM-Datenbank.md` (drei Entwürfe), Quelltext, Git-Historie |
| Zielgruppe | Implementierer und Reviewer |
| Eigenständigkeit | Plan und Spezifikation in einem Dokument. Verweise auf andere Dokumente dienen nur der Herkunft (Anhang D). |
| Verifikation | Ist-Zustand durch Lesen und Zählen am Code ermittelt. **Neu gegenüber v1:** Der Altstand wurde in einer Scratch-Kopie aus der Git-Historie rekonstruiert und gebaut (Abschnitt 1.4). Es wurde **nichts gemessen**; alle Leistungszahlen bleiben Hypothesen. |

---

## 0. Kurzfassung

Die Grundstruktur von v1 trägt und bleibt: Catalog als einzige Wahrheit, ein Redo-Frame je `Change`, ein `Fs`-Port mit Stromausfall-Simulator, getrennte Spaces, `Erase` mit einem Durability-Punkt. v2 ändert, was die Prüfung gegen Code, Historie und die Konzeptdokumente als falsch, zu knapp oder zu spät festgelegt erwiesen hat.

**Korrekturen an v1 (belegt, Abschnitt 1):**

1. **B-1 war falsch.** Der Altstand ist *nicht* unbaubar. In Commit `014319d` liegt ein Workspace mit 20 Crates. Mit den 9 Crates des Abhängigkeitsabschlusses baut `contextra-store` (`cargo check --lib` grün). Es gibt also eine lauffähige Vergleichsbasis und reale Konsumenten (`contextra-db`), deren Aufrufmuster in den Plan eingehen.
2. B-10 zählte drei statische Testschalter, es sind **sechs**. „Kein `mmap`“ ist eine echte Änderung (der Replay nutzt `memmap2`).
3. `MergeOperator` wird vom Konsumenten `contextra-db` benutzt (WASM-Brücke). SD-12 bleibt, aber die Migrationslast wird ausgewiesen.

**Neue Leitentscheidungen:**

4. **Zwei Ebenen im Kern.** Kleine, veränderliche Header liegen im LSM, große, unveränderliche Körper in **Blob-Dateien** unter derselben Catalog-Verwaltung (SD-27). Das senkt die Schreibverstärkung für Texte und Embeddings und hält `Erase` über alle Dateiarten beweisbar.
5. **Ein Durability-Punkt auch mit Blobs.** Körper reisen im Redo-Frame und wandern erst beim Flush in die Blob-Datei. Der Schreibpfad behält genau **einen** `fsync` je Gruppe.
6. **Format-v1-Freeze-Liste** (Abschnitt 8.7): Gewichte, Wert-Header, `PutRef`, `FileKind`, Konsumenten-Cursor, Durability je Frame, `tier` für alle Dateiarten. Alles, was später einen Formatbruch kosten würde, wird vor F3 eingefroren.
7. **Nebenläufigkeit als Protokoll, nicht als Absicht** (Beilage C): Sperrrangordnung, Sichtbarkeits- und Speicherordnungsprotokoll, Beweis von `INV-RET-1`, Poison-Abbildung bei `std`-Sperren.
8. **Durability-Details**, die v1 offenließ: Vorallokation der Redo-Segmente, Siegel statt Raten (`INV-LOG-6`), Regel für `ENOSPC` nach Teilschreibvorgang.
9. **Sicherheit feingranular:** drei Integritätsmodi (`Plain`, `Authenticated`, `Encrypted`), **Domänenschlüssel auf Körperebene** (kryptografisches Löschen je Subjekt, ohne die Schlüsselordnung zu stören), typisierte Capability-Handles, Bindung der AEAD-Zusatzdaten an Schlüssel und Ort, optionaler Rollback-Anker (Abschnitt 14).
10. **Ressourcenbudget als Entwurfsgröße:** Governor mit Quoten je Instanz, geteilte Caches, Leerlauf-Fußabdruck, kein Thread je Instanz (Abschnitt 16).
11. **Zwei Wartungslanes mit Claim-Tabelle** statt eines Einzelslots (SD-16 geändert): Flush und `Erase` werden nie hinter einer langen Compaction blockiert.
12. **Testnetz erweitert:** Fehlerklassen jenseits Stromausfall (Bitfäule, verlorene und fehlgeleitete Schreibvorgänge, `EIO` beim Lesen), Linearisierbarkeitsprüfung über die Stamp-Ordnung, Differentialtest gegen den rekonstruierten Altstand (soweit lauffähig).

**Nicht übernommen** aus den Konzeptdokumenten (Begründung in Anhang C): Prolly-Tree und Merkle-DAG als Hauptstruktur (zurückgestellt auf Stufe B), feste 4-KiB-„Context Slabs“, Commit ohne Log per CAS auf einen Superblock, Thread-per-Core mit `io_uring` im Kern, `O_DIRECT` im Standardbau, 2PC über Locality Groups.

Geplante Größe: ≈ 13–16 k Zeilen Produktionscode (Hypothese; v1: 10–12 k; Blob-Ebene, Sicherheitsmodi und Governor sind die Differenz).

---

## 1. Befund (Ist-Zustand)

Methode: Quelltext gelesen, Zählungen per `grep`/`wc`, Git-Historie ausgewertet, Altstand in einer Scratch-Kopie gebaut. Stellen, die nicht gelesen wurden (z. B. Details von `flusher.rs`, `replay.rs`, `reader/open.rs`), sind nicht bewertet.

### 1.1 Kennzahlen

| Messgröße | Wert | Stand |
|---|---|---|
| Zeilen `src/` / `tests/` | 32 015 / 25 563 | in v2 nachgezählt, identisch zu v1 |
| Zeilen `benches+examples+fuzz` (gesamt 59 788) | 2 210 | aus v1 übernommen |
| Testfunktionen | 636 in 160 Dateien plus Inline-Module | aus v1 übernommen |
| `async fn` in `src/` | 420, davon 31 Stellen mit `spawn_blocking` | aus v1 übernommen |
| Direkte Dateisystemaufrufe außerhalb von Tests | 91 Vorkommen in 20+ Dateien, am Vfs-Port vorbei | aus v1 übernommen |
| Zugriffe auf `TOMBSTONE_BIT` | 142 | aus v1 übernommen |
| Cargo-Features | 10, davon 5 ändern Logik | aus v1 übernommen |
| Statische Testschalter in `wal/core.rs` | **6** (`FAIL_APPEND_FOR_TX`, `DELAY_APPEND_FOR_TX`, `DELAY_APPEND_MS`, `FAIL_TRUNCATE_ONCE`, `FAIL_APPEND_AFTER_PARTIAL_BYTES`, `FAIL_APPEND_PARTIAL_ONCE`) | in v2 nachgezählt (v1: 3) |
| Git-Historie | 55 Commits; Workspace mit ≈ 39 Mitgliedern bis Commit `014319d` (12.11.2026), entfernt in `813305c` (13.11.2026, 2 931 Dateien) | in v2 ermittelt |

### 1.2 Befunde mit Beleg

| # | Befund | Beleg | Folge |
|---|---|---|---|
| B-1 | **(korrigiert)** Der HEAD enthält nur `crates/contextra-store` und kein Root-`Cargo.toml`. Die Abhängigkeiten sind aber **in der Historie vorhanden** (`014319d`). Das dortige Workspace-Manifest nennt 20 Mitglieder, die nicht existieren (z. B. `contextra-wire`, `contextra-adapt`); `contextra-db` hängt von dem fehlenden `contextra-adapt` ab. Mit einem auf 9 Crates reduzierten Manifest (`ids, env, sys, mvcc, crypto, types, ports, testkit, store`) baut `cargo check -p contextra-store --lib` (234 Warnungen, überwiegend `missing_docs`). | `git log -- crates/contextra-mvcc`, `git ls-tree 014319d^…`, Scratch-Bau (1.4) | Der Altcode ist als **Bibliotheks-Baseline** nutzbar. Tests, Benches und Beispiele sind es nicht (B-14). Plan: F0-Aufgabe „Baseline wiederherstellen“. |
| B-2 | **Domänenlogik im Kernel.** Transaktionspuffer und SSI-Validator (`contextra-mvcc`), Intent-Locks, Mandanten-Codec (`tenant_codec`), `docid-128`, Scanner/Purger für Ports (`compat/`, 2 650 Zeilen). | `lsm/engine.rs`, `compat/*`, `Cargo.toml` | Der Speicher kennt Dinge, die nur obere Schichten kennen dürfen. |
| B-3 | **Drei Dateisystem-Verträge**: `traits.rs` (`StoreFs`/`StoreFile`, außer in der Definition ungenutzt), `io_seam.rs` (`Vfs`/`VfsFile`), `wal::fs` (Tokio-Wrapper, von `fsync.rs` benutzt). | `grep StoreFs`, `fsync.rs` | Fehlerinjektion erfasst nur einen Teil der E/A. |
| B-4 | **Async überall, obwohl die E/A blockiert.** Achtstufige Sperrhierarchie (`lock_order.rs`, RANK 100…800), Dauerverbot „kein Guard über `.await`“, Group-Commit als Leader/Follower-Übergabe von `commit_mutex`. | `lsm/mod.rs`, `ops/write/group.rs` | Hohe Komplexität, schwer beweisbar (`loom` nur für Teilstücke). |
| B-5 | **Drei Format-Generationen im Lesepfad.** WAL V1/V2/V3 (V1 ohne Integrität; statischer, XOR-obfuszierter Legacy-Schlüssel), SSTable `MFSX` v0–v4 plus `MFST` v3, Manifest v0–v2 mit abgekündigtem `rank`-Feld. | `wal/core.rs`, `sstable/format.rs`, `manifest/entry.rs` | Jeder Decoder kennt alle Generationen; Angriffsfläche und Testlast ohne Nutzen. |
| B-6 | **Transaktion als Folge von Records** (`Put`/`Delete`/`TxEnd{committed}`), HMAC-Kette je Eintrag, Replay mit „pending tx“-Zustand. | `wal/encode.rs`, `lsm/recovery/startup.rs` | Atomarität ist Konvention statt Struktur. |
| B-7 | **`rollback_to_tx`** truncatiert das WAL in place, schneidet Memtables, baut „spanning“ SSTables um (Intent-Datei `MFRLBK`). Außerhalb des Stores gibt es **nur Test-Doubles** (`contextra-testkit`, `contextra-text`-Mocks) als Implementierer; ein produktiver Aufrufer wurde im Altstand nicht gefunden (Suche über `crates/`). | `lsm/recovery/rollback.rs`; `grep rollback_to_tx` im Altstand | Komplexester Wiederherstellungspfad ohne nachgewiesenen Nutzer. SD-09 (entfällt) ist damit **belegt**, nicht nur angenommen (A-06). |
| B-8 | **Manifest als Pfad-String-Log** mit Rang-Rekonstruktion; Leseschlüssel (Rang, `max_seq`) fehlen im Manifest. | `manifest/entry.rs`, `cleanup.rs` | Planung braucht Dateiöffnung. |
| B-9 | **Löschung ist nicht kontrolliert.** (a) Der Purger liest ganze SSTables per `reader.iter().await.unwrap_or_default()`, Lesefehler werden verschluckt. (b) Er schreibt Nullen und ruft `remove_file` ohne `fsync` von Datei oder Verzeichnis und am Manifest vorbei. (c) Tote Dateien werden über die Endung `.sst` erkannt. (d) Compaction löscht Eingabedateien nur mit Warnung bei Fehlern; ein Verzeichnis-`fsync` nach dem Unlink fehlt. | `compat/scanner/purge.rs` Z. 70–81, 145, 222; `compaction/engine.rs` Z. 330, 418 | Kein atomarer, nachweisbarer physischer Löschvorgang. |
| B-10 | **Globaler veränderlicher Zustand:** sechs `static Atomic*` für Fehlerinjektion in `wal/core.rs` ohne Feature-Gate. | `wal/core.rs` Z. 27–40 | Tests beeinflussen sich; Release-Binary enthält Testschalter. |
| B-11 | **Drei Block-Cache-Implementierungen** hinter Features; Bloom-Filter im Builder fest auf `BloomFilter::new(100_000, 0.01)`. | `sstable/block_cache/*`, `builder/writer.rs` | Fehlerrate stimmt weder für große noch für kleine Runs. |
| B-12 | **Rauschen im Quelltext:** Kommentarbanner mit veralteten Daten, gemischte Sprachen, `pub(super)`-Geflecht. | diverse | Review erschwert. |
| B-13 | **Dateirechte nur teilweise geregelt.** Ein expliziter Modus `0o600` findet sich nur für die HMAC-Schlüsseldatei (`wal/hmac.rs:292`); `set_restrictive_file_acl` existiert nur für Windows (`wal/io/acl.rs`, `#[cfg(windows)]`). Für `create_new` der WAL-Datei (`wal/core.rs:311`) fand die Suche keine expliziten Modusbits. (Die `0o644` in `wal/fs.rs` stammen aus einem `loom`-Shim und sind **kein** Befund.) | `grep 'mode(0o'`, `core.rs:311` | WAL-/Run-Dateien mit Klartext erben unter Unix die `umask`. Plan: `Fs`-Port erzwingt private Modi (Abschnitt 14.6). |
| B-14 | **Feature-Matrix bricht im Altstand.** Mit `--features workspace` (`compat/`, Benches, Beispiele) und in allen Testkonfigurationen scheitert die Übersetzung (u. a. `Bytes`, `Arc`, `BTreeMap`, `TxId`, `StorageEngine`, `ManualClock` unaufgelöst; 72 Fehler `E0433`). Nur der feature-lose Bibliotheksbau ist grün. | `cargo check --benches --features workspace`, `--tests …` im Scratch-Bau | Der Bestandstestkorpus ist **kein** automatisches Orakel. Er bleibt Spezifikationsquelle (15.3). |
| B-15 | **Konsument nutzt `MergeOperator`.** `contextra-db` implementiert `contextra_store::MergeOperator` als Brücke zu einer WASM-Merge-Funktion (`merge_operator_adapter.rs`). | `grep MergeOperator` im Altstand | SD-12 (kein Merge-Operator in v1) hat Migrationskosten in der oberen Schicht (A-12). Tag-Kind `Merge` bleibt reserviert. |

### 1.3 Was gut ist und übernommen wird

- Lint-Disziplin: `forbid(unsafe_code)`, `deny(unwrap_used, expect_used, panic, indexing_slicing)` in Produktionscode.
- Fail-stop bei `fsync`-Fehlern (INV-FSYNC-1): Handle vergiften, nie wiederholen. Bleibt als `INV-LOG-5`.
- Die Retention-Regel (`compaction/retention.rs`): alle Versionen über der Untergrenze behalten, die neueste darunter, Rest verwerfen. Wird als reine Funktion mit Property-Tests übernommen.
- Atomares Schreiben über Temp-Datei, `fsync`, `rename`, Verzeichnis-`fsync`.
- Der Testbestand als **Spezifikation**: Absturz-, Bitflip-, Platte-voll-, Abbruch-Szenarien (`chaos_*`, `crash_*`).
- **Vorarbeit im Altstand (nur als Ideenquelle, nicht als Code):** `contextra-durable-fs` (≈ 2 000 Zeilen) enthält einen Fehlerinjektor (`FaultInjectingVfs`: Platte voll, `fsync`-Fehler, `rename`-Fehler, Verzeichnis-`fsync`-Fehler, `simulate_crash`) und eine Gesundheits-Zustandsmaschine (`HealthState`/`HealthEvent`, Poison). Beides stützt das Design von `FaultFs` und `Poison` (Abschnitt 9.5, 15.2).

### 1.4 Legacy-Baseline (neu in v2)

**Ziel:** Ein lauffähiger Altstand erlaubt (a) Differentialtests gegen den neuen Kern, (b) echte Ausgangswerte für die Hypothesen, (c) eine Auswertung realer Aufrufmuster.

**Rekonstruktion (verifiziert, Scratch-Kopie, nicht im Repository):**

```
git archive 014319d | tar -x -C legacy-baseline/
# Root-Cargo.toml: members und default-members auf die 9 vorhandenen Crates reduzieren,
# exclude entfernen
cargo check -p contextra-store --lib        # grün, 234 Warnungen
```

**Ergebnis und Grenzen:**

| Prüfung | Ergebnis |
|---|---|
| `cargo check -p contextra-store --lib` | **grün** |
| `cargo test --lib` (auch mit `workspace,test-hooks,fault-injection`) | **nicht übersetzbar** (B-14) |
| Benches, Integrationstests | **nicht übersetzbar** (B-14) |
| `contextra-db` und weitere Konsumenten | nicht baubar (fehlende Crates, u. a. `contextra-adapt`) |

Der Differentialtest läuft daher über eine **eigene schlanke Harness**, die gegen die Bibliotheks-API des Altstands linkt (kein Rückgriff auf dessen Tests). Der Reparaturaufwand ist als Aufgabe F0 eingeplant (Größe S bis M; Abbruchkriterium in Abschnitt 18).

**Reale Aufrufmuster** (textuelle Zählung über `crates/contextra-db/src` im Altstand; grob, enthält Inline-Tests, keine semantische Analyse):

| Aufruf | Anzahl | Folge für den Plan |
|---|---|---|
| `.get(` | 161 | Punktlesen dominiert: Bloom, Index-Cache, Block-Cache zuerst (F3, H-S1) |
| `.commit(` | 54 | Batch-Schreiben ist der Normalfall → `Change` |
| `.put(` / `.delete(` | 38 / 23 | |
| `.scan_prefix(` / `.scan(` | 28 / 6 | Präfix-Scans überwiegen **ca. 5:1** → `PrefixFilterBlock` ist Pflicht, nicht optional (SD-23) |
| `.put_if_absent(` | 2 | → `Guard::absent` |
| `.pin_checkpoint(` | 2 | → `Hold` |
| `.flush(` | 2 | |
| Typen: `LsmStorage` 51+, `LsmConfig` 45+, `TenantScopedStorage` 3, `SystemPressure` 3, `MergeOperator` 2, `DurabilityMode` 1 | | `TenantScopedStorage` → obere Schicht; `SystemPressure` → `Governor`; `MergeOperator` → B-15 |

---

## 2. Ziele, Nicht-Ziele, Leitprinzipien

### 2.1 Ziele

| ID | Ziel |
|---|---|
| Z-1 | Atomare Änderungen über mehrere Schlüssel **und Spaces** hinweg, mit Vorbedingungen (Compare-and-Swap), crash-sicher. |
| Z-2 | Konsistente Ansichten (`View`) und Zeitreise auf Stamp-Ebene, mit Halteregeln (`Hold`) und Konsumenten-Cursorn gegen Compaction. |
| Z-3 | **Kontrollierte, atomare, nachweisbare physische Löschung** von Schlüsselbereichen und Spaces, über **alle** Dateiarten (Runs, Blobs, Redo, Segmente). |
| Z-4 | Autark: keine internen Abhängigkeiten, keine externen im Standardbau. |
| Z-5 | Einfach: wenige Konzepte, kleine Module, ein Format je Dateiart, ein Compaction-Verfahren. |
| Z-6 | Beweisbar robust: deterministische Tests, Fehlerinjektion an **jedem** E/A-Schritt, Modelltests, Linearisierbarkeitsprüfung. |
| Z-7 | Eignung für Agenten- und LLM-Daten: viele kleine atomare Schreibvorgänge, kurzlebige Ansichten, strukturierte Fehler mit Handlungsanweisung, Idempotenz über `Guard::absent`. |
| Z-8 | Betreibbar: `verify`, konsistente Sicherung, Metriken, explizite Formatversion mit Pflicht-/optionalen Features. |
| Z-9 | Budgetierte Ergebnislieferung in **Bytes und frei definierbaren Gewichten** (typisch Token): vorab abschätzen, seitenweise stabil fortsetzen. |
| Z-10 | Erweiterbar ohne Formatbruch: Index-Blöcke je Run, entkoppelte Segmente, kalter Speicherort, Blob-Ebene. |
| Z-11 | **Robustheit unter dem Fehlermodell von Abschnitt 3**, nicht nur unter Stromausfall: Bitfäule, verlorene/fehlgeleitete Schreibvorgänge, `EIO`, `ENOSPC`, Speicherdruck. |
| Z-12 | **Ressourcenschonend:** begrenzter und vorhersagbarer Speicher- und Dateideskriptorbedarf, kein Thread je Instanz, Leerlauf-Fußabdruck nahe null, faire Teilung von Caches und E/A zwischen Instanzen. |
| Z-13 | **Feingranulare Sicherheit:** Integrität konfigurierbar, Verschlüsselung je Space und je Körper-Domäne, typisierte Zugriffsrechte, private Dateimodi, keine Datenlecks über Fehlertexte. |
| Z-14 | **Beweisbare Nebenläufigkeit:** wenige Sperren mit fester Rangordnung, alle Synchronisationsprotokolle klein genug für `loom`, Sichtbarkeit und Untergrenze mit schriftlichem Beweis (Beilage C). |

### 2.2 Nicht-Ziele (bewusst)

Verteilter Konsens; Mehrprozess-Zugriff auf eine Instanz (eine Instanz, ein Prozess, `LOCK`-Datei); Transaktionspuffer, SSI, Intent-Locks, Mandanten- und Dokument-ID-Kodierung (obere Schichten); Merge-Operatoren (reserviert, SD-12); Kompatibilität mit dem Altformat im Kern (SD-04); Windows mit voller Durability in Version 1 (A-02); `mmap`; Inhaltsadressierung und Hashing von Nutzdaten im Kern; Tokenizer und Embeddings im Kern.

### 2.3 Leitprinzipien (Code-Regeln, im Review prüfbar)

1. Jede Datei auf der Platte hat **einen** Eigentümer-Mechanismus (Abschnitt 7) und wird nur über `Fs` berührt.
2. Zustandsänderung = **ein** Catalog-Edit. Wer „zwei Dinge atomar“ braucht, packt sie in einen Edit.
3. Reihenfolge ist Vertrag: *Datei dauerhaft (Datei + Verzeichnis) → Edit dauerhaft → erst dann sichtbar → erst dann alte Dateien löschen.*
4. Dekodierung ist **total**: jede Byte-Folge ergibt `Ok` oder `Err(Corrupt)`, nie `panic`, nie unbegrenzte Allokation (Längen werden vor `Vec::with_capacity` gegen Limits geprüft).
5. Keine globalen Variablen. Keine `cfg(feature)` in Logik außer `encryption` und `testing`. Prüfpunkte für Tests laufen über `Env::hooks()` (Standard: leer), nicht über Statics.
6. Dateien ≤ 400 Zeilen, Funktionen typisch ≤ 60 Zeilen, Modultiefe ≤ 3, keine Kommentarbanner. Invarianten stehen in `SPEC.md` und werden in Tests per ID zitiert.
7. Rustdoc auf Deutsch, Bezeichner auf Englisch.
8. **Fehler sind Daten:** jede Fehlerklasse aus Abschnitt 3 hat einen definierten Zustand danach (heil, vergiftet, korrupt) und einen Test.
9. **Kein Lock über E/A, außer dem Committer-Lock** (Beilage C): Leser und Wartung warten nie auf Plattenzugriffe anderer.
10. **Kein Lock über Aufrufer-Code:** `Weigher`, `Filter`, `CompactionFilter`, `Extension`, `Listener` laufen ohne gehaltene Sperre.
11. **Jede Allokation im Heißpfad ist begründet:** `apply` und `get` haben ein Allokationsbudget, das ein Test mit zählendem Allokator prüft (Abschnitt 16.5).
12. **Geheimnisse verlassen den Prozess nie unverschlüsselt** und erscheinen weder in `Debug`, Logs noch `hint()` (Abschnitt 14.7).

---

## 3. Fehler- und Bedrohungsmodell (neu)

v1 behandelte Stromausfall gründlich, andere Fehlerklassen nur am Rand. Dieser Abschnitt legt fest, wogegen der Kern sich verteidigt und welche Garantie er danach gibt.

### 3.1 Fehlerklassen

| ID | Fehlerklasse | Erwartetes Verhalten | Garantie / Zustand danach | Netz (15) |
|---|---|---|---|---|
| F1 | Stromausfall / Prozess-Kill (`kill -9`) | Recovery (Abschnitt 11) | Präfix der quittierten Changes, jede Quittung mit `durable=true` bleibt erhalten | Absturzmatrix |
| F2 | Zerrissener Schreibvorgang (Teil eines Sektors/Blocks persistiert) | Frame-/Block-/Record-Prüfsumme | Torn-Tail im **letzten ungesiegelten** Segment wird geheilt, sonst `Corrupt` | Absturzmatrix, Fuzz |
| F3 | Nicht synchronisierte Schreibvorgänge in beliebiger Reihenfolge übernommen | Reihenfolge-Vertrag (Prinzip 3) | nie ein Edit vor seiner Datei | `FaultFs` (Umordnung) |
| F4 | `fsync`-Fehler zur Laufzeit | **Poison**, kein Retry (`INV-LOG-5`) | Instanz tot; Neustart führt Recovery aus | `FaultFs` |
| F5 | `ENOSPC` beim Schreiben | Abschneiden auf den letzten guten Stand (9.5); Segment-Vorallokation verlagert den Fehler nach vorn | heil und wiederholbar (`Busy{StorageFull}`) oder Poison, wenn Abschneiden scheitert | `FaultFs` |
| F6 | `EIO` / Fehler beim **Lesen** | immer als Fehler melden, **nie als „leer“** | Operation scheitert; Instanz bleibt benutzbar; wiederholte Fehler → Metrik + Datei als verdächtig markiert | `FaultFs` |
| F7 | Bitfäule im Ruhezustand | Prüfsumme je Block/Frame/Record; im Modus `Authenticated/Encrypted` kryptografisch | `Corrupt{file,offset}`; nie falsche Daten | Bitflip-Test |
| F8 | Verlorener Schreibvorgang (Platte quittiert, schreibt nicht) / fehlgeleiteter Schreibvorgang (falscher Ort) | Kettenprüfsumme im Redo, Block-AAD (Run-ID, Offset), Catalog-Anker; `verify` prüft Gesamtkonsistenz | erkannt, wenn die Datei intern inkonsistent wird; **nicht erkennbar**, wenn ein ganzer älterer, in sich stimmiger Dateistand zurückgespielt wird (→ F10) | `FaultFs` (Lost/Misdirected), `verify` |
| F9 | Speicherdruck / Dateideskriptor-Erschöpfung | `Governor`: Verlangsamen, Flush erzwingen, Cache verkleinern; `TableCache` begrenzt offene Dateien | keine Allokationsabbrüche im Kern; `Busy{retry_after}` statt OOM | Ressourcentests (16.5) |
| F10 | Zurückspielen eines älteren Gesamtstands (Rollback-Angriff, versehentliches Restore) | nur mit **Rollback-Anker** (14.5) erkennbar | ohne Anker: **nicht erkennbar**, in der Doku offen genannt | Anker-Test |
| F11 | Uhrsprung / Uhr rückwärts | `Clock` nur für Ablauf (Holds, TTL, Intervall-`fsync`); Ordnung läuft ausschließlich über Stamps | nie Ordnungsfehler; Abläufe können früher/später greifen | `Clock`-Tests |
| F12 | Panic in Aufrufer-Code (`Weigher`, `Filter`, `Listener`, `Extension`) | `catch_unwind` an der Grenze zu Aufrufer-Code in Wartung und Listener-Zustellung; im Committer **nicht** (dort keine Aufrufer-Callbacks) | Job schlägt fehl und wird gemeldet; Daten unverändert; Listener wird abgemeldet | Panic-Injektion |
| F13 | `std`-Sperre vergiftet (ein Thread starb im kritischen Abschnitt) | `PoisonError` wird auf `Error::Poisoned{cause: LockPoison}` abgebildet | Instanz tot, Neustart | Panic-Injektion |

### 3.2 Angreifermodell (Sicherheit)

| ID | Angreifer | Fähigkeit | Verteidigung | Grenze |
|---|---|---|---|---|
| A1 | Anderer lokaler Benutzer / anderer Mandant im selben Host | liest Dateien, wenn Rechte es zulassen | private Modi (`0600`/`0700`), eigene Verzeichnisse je Instanz | Root, Hypervisor |
| A2 | Offline-Angreifer auf dem Datenträger (Diebstahl, Backup-Zugriff) | liest/verändert Dateien | Modus `Encrypted`: Vertraulichkeit und Integrität | Rollback ganzer Stände (F10) ohne Anker |
| A3 | Offline-Angreifer ohne Schlüssel, der Dateien verändern will | Bits kippen, Frames umordnen, Blöcke vertauschen | Modus `Authenticated` oder `Encrypted`: Authentisierung mit AAD-Bindung | Löschen ganzer Dateien/Zurückspielen (F10) |
| A4 | Fehlerhafter oder bösartiger Aufrufer (obere Schicht, Agent) | ruft die API unzulässig auf | typisierte Handles (14.4), Limits, `Forbidden`, append-only Spaces | Vertrauen in die einbettende Anwendung |
| A5 | Prozess mit Speicherzugriff (Core-Dump, Swap) | liest Schlüssel/Klartext | nicht kontrollierbar (kein `unsafe`, kein `mlock`); Schlüsselbereinigung best-effort | siehe 14.7 |

**Konsequenz:** Ohne `Authenticated`/`Encrypted` schützt CRC32C **nur** gegen Zufallsfehler, nicht gegen Angreifer. Das steht so in der Dokumentation und in `FORMAT` (Feld `integrity_mode`).

---

## 4. Vokabular

Im Crate kommen weder Produkt- noch Fremdsystemnamen vor.

| Neu | Bedeutung | Bisher |
|---|---|---|
| **Storage** | Handle einer geöffneten Instanz | `LsmStorage` |
| **Space** | Benannter, physisch getrennter LSM-Baum (`SpaceId(u16)`) | Präfix-Reservierung |
| **Change** | Atomare Menge von Schreibvorgängen plus `Guard`s | Transaktion + `TxBuffer` |
| **Guard** | Vorbedingung innerhalb eines `Change` | `put_if_absent`, SSI-ReadSet |
| **Stamp** | Monotone Zählung angenommener Changes (u64, < 2^56) | `seq_no` + `TOMBSTONE_BIT` |
| **Receipt** | Quittung eines `Change` (`stamp`, `durable`) | – |
| **View** | Unveränderliche Sicht auf einen Stamp | Snapshot |
| **Cursor** | Streaming-Iterator über einen Schlüsselbereich | `scan` (materialisiert) |
| **Hold** | Befristete Halteregel gegen das Verwerfen alter Versionen, mit Eigentümer | `pin_checkpoint` |
| **Consumer** | Benannter, persistenter Fortschrittszeiger eines abgeleiteten Index (zählt wie ein Hold, mit Maximalrückstand) | – |
| **Redo-Log** | Segmentiertes, gesiegeltes Wiederholungsprotokoll | WAL |
| **Run** | Unveränderliche sortierte Datei mit Headern und Verweisen | SSTable |
| **Blob** | Unveränderliche Datei mit großen Wertkörpern, vom Catalog verwaltet | – |
| **BlobRef** | Verweis `(blob_no, offset, len, crc, domain)` im Run | – |
| **Header** | Die ersten `header_len` Bytes eines Wertes; vom Kern ohne Körper lesbar | – |
| **Weight** | Frei definierbare Zählgröße je Eintrag (k ≤ 4, typisch Token), in Block-Summen geführt | – |
| **KeyDomain** | Verschlüsselungsdomäne eines Körpers (`u32`, z. B. ein Subjekt) | – |
| **Segment** | Entkoppelte, vom Catalog verwaltete Datei einer oberen Schicht (z. B. Vektorgraph) | – |
| **FileKind** | Dateiart im Catalog: `Run`, `Redo`, `Blob`, `Segment(kind)`, `Cold` | Pfadendung |
| **Claim** | Reservierung von Dateien durch einen laufenden Wartungsjob | – |
| **Catalog** | Einzige Wahrheit über lebendige Dateien und Zustand | Manifest |
| **Erase / ErasureReceipt** | Löschauftrag mit physischer Garantie / Nachweis | Purger |
| **Env** | Bündel der Ports (`Fs`, `Clock`, `Entropy`, `Runtime`, `Governor`, `Metrics`, `Hooks`, `AccessSink`, `Checksum`) | `contextra-env`, `Vfs` |
| **Graveyard** | Warteschlange für zu löschende, noch referenzierte Dateien | – |
| **Page / ResumeToken** | Budgetierte Ergebnisseite und Fortsetzungsmarke | `LIMIT/OFFSET` |
| **Estimate / History / Extension** | Schätzung aus Indexdaten / Versionsverlauf / Index-Block je Run | – |

---

## 5. Architektur

### 5.1 Crate-Schnitt

```
contextra/                       Workspace-Root (neu, F0)
  Cargo.toml                     resolver 2, MSRV in F0 festgelegt (A-05)
  crates/contextra-ports/        Traits und kleine Wertetypen, KEINE Logik (SD-31)
  crates/contextra-storage/      der Kern dieses Plans, zero-dep
  legacy/contextra-store/        eingefrorener Altstand, nicht Teil des Workspace-Builds
```

`contextra-ports` ist ein eigenes, winziges Crate, damit künftige Crates (`simd`, `crypto`, `cell`, `text`, `vector`, …) `Checksum`, `Cipher`, `Weigher`, `Filter`, `Extension` implementieren können, ohne vom Kern abzuhängen (Abschnitt 20.3). Der Kern hängt an `ports`, nicht umgekehrt. `ports` hat keine Abhängigkeiten.

### 5.2 Modulschnitt von `contextra-storage`

```
src/lib.rs             Re-Exports der öffentlichen API
src/api/               storage.rs space.rs (typisierte Handles) change.rs view.rs cursor.rs page.rs
                       hold.rs consumer.rs listener.rs config.rs error.rs
src/env/               real_fs.rs clock.rs entropy.rs runtime.rs governor.rs metrics.rs hooks.rs
src/codec/             varint.rs crc32c.rs bloom.rs tag.rs keyrange.rs bytes.rs
src/redo/              frame.rs segment.rs seal.rs writer.rs reader.rs recover.rs
src/run/               builder.rs reader.rs block.rs footer.rs filter.rs rangedel.rs extension.rs
                       table_cache.rs block_cache.rs
src/blob/              record.rs writer.rs reader.rs sweep.rs
src/catalog/           edit.rs log.rs state.rs current.rs format.rs
src/engine/            version.rs memtable.rs committer.rs flush.rs views.rs retention.rs
                       compaction/{plan.rs,retain.rs,exec.rs,blobgc.rs}
                       maintenance.rs (Lanes, Claims) recovery.rs graveyard.rs
src/erase/             plan.rs exec.rs registry.rs verify.rs
src/crypto/            (Feature `encryption`) cipher.rs keys.rs domains.rs
src/testing/           (Feature `testing`) mem_fs.rs fault_fs.rs manual_runtime.rs hooks.rs
                       model.rs oracle.rs linearize.rs
```

### 5.3 Abhängigkeitsregeln (nur nach unten, per Test erzwungen)

```
 api ──► engine ──► { redo, run, blob, catalog } ──► { codec, env }
 api ──► erase  ──► engine
 crypto (Feature) ──► { codec, env };  redo/run/blob/catalog nutzen nur das Trait `Cipher`
 testing (Feature) ──► alles; nichts hängt von testing ab
```

`tests/layering.rs` liest alle `use crate::…`-Zeilen und schlägt fehl, wenn eine Kante gegen die Regel verstößt.

### 5.4 Ports (öffentlich, semver-stabil, in `contextra-ports`)

```rust
pub enum Perm { Private /* 0600 Dateien, 0700 Verzeichnisse bzw. restriktive ACL */, Inherit }
pub enum OpenMode { Read, ReadWrite, CreateNew(Perm) }

pub trait Fs: Send + Sync + 'static {
    fn create_dir_all(&self, p: &Path, perm: Perm) -> io::Result<()>;
    fn open(&self, p: &Path, mode: OpenMode) -> io::Result<Box<dyn File>>;
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
    fn remove_file(&self, p: &Path) -> io::Result<()>;
    fn remove_dir(&self, p: &Path) -> io::Result<()>;
    fn list_dir(&self, p: &Path) -> io::Result<Vec<DirEntry>>;          // Name + Länge
    fn sync_dir(&self, p: &Path) -> io::Result<()>;
    fn lock(&self, p: &Path, mode: LockMode) -> io::Result<Box<dyn LockGuard>>; // Exclusive | Shared, nicht blockierend
    fn hard_link(&self, from: &Path, to: &Path) -> io::Result<()>;      // nur Checkpoint (opt-in)
}
pub trait File: Send + Sync {
    fn read_at(&self, buf: &mut [u8], off: u64) -> io::Result<usize>;
    fn read_batch(&self, reqs: &mut [ReadReq]) -> io::Result<()> { /* Standard: Schleife über read_at */ }
    fn write_all_at(&self, buf: &[u8], off: u64) -> io::Result<()>;
    fn preallocate(&self, len: u64) -> io::Result<()> { /* Standard: Nullen in 1-MiB-Schritten */ }
    fn len(&self) -> io::Result<u64>;
    fn set_len(&self, len: u64) -> io::Result<()>;
    fn sync_data(&self) -> io::Result<()>;
    fn sync_all(&self) -> io::Result<()>;
}
pub trait Clock: Send + Sync { fn now_ms(&self) -> u64; fn monotonic_ns(&self) -> u64; }
pub trait Entropy: Send + Sync { fn fill(&self, dst: &mut [u8]) -> io::Result<()>; }
pub trait Checksum: Send + Sync { fn crc32c(&self, seed: u32, data: &[u8]) -> u32; }   // Standard: portabel, slicing-by-8, safe
pub trait Metrics: Send + Sync { fn counter(&self, n: &'static str, v: u64); fn gauge(&self, n: &'static str, v: i64); fn observe(&self, n: &'static str, v: f64); }
pub trait AccessSink: Send + Sync { fn sampled(&self, ev: AccessEvent); }   // (space, file, block_no, key_prefix, bytes); Standard: Null
pub trait Hooks: Send + Sync { fn point(&self, p: HookPoint) {} }          // Standard: leer; Tests blockieren/vertauschen hier
pub trait PressureSource: Send + Sync { fn level(&self) -> Pressure; }      // optional (cgroup/PSI in separater Impl.)
pub trait Cipher: Send + Sync { fn seal(&self, key: &[u8;32], nonce: &[u8;12], aad: &[u8], plain: &[u8]) -> Vec<u8>;
                                fn open(&self, key: &[u8;32], nonce: &[u8;12], aad: &[u8], sealed: &[u8]) -> io::Result<Vec<u8>>; }
pub trait KeyProvider: Send + Sync { fn kek(&self) -> io::Result<Secret32>; fn domain_key(&self, d: KeyDomain) -> io::Result<Option<Secret32>>; }
pub trait ColdTier: Send + Sync { fn upload(&self, f: FileRef) -> io::Result<()>; fn read_range(&self, f: FileRef, off: u64, len: u32) -> io::Result<Vec<u8>>; fn delete(&self, f: FileRef) -> io::Result<()>; }
// Aufrufer-Erweiterungen (Abschnitte 6, 10):  Weigher, Filter, CompactionFilter, Extension
```

Änderungen gegenüber v1: `Perm` (private Modi sind Teil der `Fs`-Verträge, B-13), `LockMode::Shared` (für `open_readonly`, `verify`, `checkpoint`), `read_batch` (ein Recall mit 200 Kandidaten braucht Sammel-Lesevorgänge; eine `io_uring`-Implementierung in einem separaten Crate kann sie überschreiben), `preallocate` (9.5), `Checksum` (beschleunigtes CRC32C braucht `unsafe` und liegt deshalb außerhalb des Kerns), `AccessSink` (ohne Lesespur kein lernendes Layout und keine Evaluation), `Hooks` (ersetzt die sechs Statics aus B-10), `PressureSource`, `KeyProvider::domain_key` (14.3).

`RealFs` nutzt ausschließlich `std` (`FileExt::read_at/write_at` unter Unix, `File::lock/try_lock`). `write_atomic` (Temp → `sync_all` → `rename` → `sync_dir`) ist eine **Hilfsfunktion über `Fs`**, kein Trait-Mitglied, damit sie in `FaultFs` mitgetestet wird. `OpenMode::Direct` (`O_DIRECT`) ist **nicht** Teil von v1 des Ports (SD-31): Ausgerichtete Puffer und Plattformunterschiede gehören in eine alternative `Fs`-Implementierung, sobald ein Verbraucher (z. B. KV-Cache-Blöcke) sie braucht; der Port bleibt dafür erweiterbar (neue Variante, Standardverhalten unverändert).

`Runtime` ist ein kleiner, **teilbarer** Thread-Pool (`spawn(Box<dyn FnOnce + Send>)`); `Runtime::manual()` führt Jobs nur auf Aufruf aus (`run_pending()`), sodass Tests jede Verschränkung deterministisch steuern. `Governor` bündelt Speicher- und E/A-Budgets (Abschnitt 16).

---

## 6. Datenmodell und öffentliche API

### 6.1 Schlüssel, Tags, Reihenfolge

Interner Schlüssel = `user_key ‖ tag`, `tag = (stamp << 8) | kind`, sortiert **Schlüssel aufsteigend, Tag absteigend** (neueste Version zuerst). `kind ∈ {Put=1, Delete=2, RangeDelete=3, EraseRange=4, PutRef=5, Merge=6 (reserviert, nicht erzeugbar in v1)}`. Der Löschmarker ist ein eigenes Typ-Byte, nicht Bit 63; alle 142 Maskierungsstellen des Bestands entfallen. Stamps sind auf < 2^56 begrenzt; Überlauf wird als `Error::Exhausted` gemeldet, nie umgebrochen. Der Space steckt nicht im Schlüssel, sondern in der Dateizuordnung (SD-07).

`PutRef` ist `Put` mit ausgelagertem Körper: Der Wert im Run besteht aus `header ‖ BlobRef` (8.3, 8.4).

### 6.2 `SpaceOptions` (Teil von `spaces_add` im Catalog, unveränderlich nach dem Anlegen)

| Option | Wirkung | Begründung |
|---|---|---|
| `header_len(n)` | Die ersten `n` Wertbytes sind der **Header** (≤ 255 B); der Kern kann ihn ohne Körper lesen. | Ranking, Filter und Gewichte ohne Körperzugriff (Mnemos „Hot Head“) |
| `weights(k)`, `k ≤ 4` | Index-Blöcke führen k `u32`-Summen; Gewicht 0 ist immer „Bytes“. Ein `Weigher` (rein, mit `version()`) liefert die übrigen. | Token-Budget ohne Tokenizer im Kern (SD-28) |
| `separate_body(threshold)` | Körper ab `threshold` Bytes (Vorschlag 4 KiB) liegen in Blob-Dateien. | Schreibverstärkung und Leselokalität (SD-27) |
| `domain_sealed()` | **Jeder** Körper (auch kleine) geht in Blob-Dateien und wird mit dem Schlüssel seiner `KeyDomain` versiegelt. | Kryptografisches Löschen je Subjekt (14.3) |
| `prefix_filter(n)` | Bloom-Filter über die ersten `n` Schlüsselbytes je Run. | Präfix-Scans überwiegen 5:1 (1.4) |
| `locality_prefix(n)` | Compaction teilt Ausgaben bevorzugt an Präfixgrenzen (±25 % der Zielgröße). | Zusammen Gelesenes liegt zusammen (10.5) |
| `append_only()` | `Put` nur auf Schlüssel ohne lebende Version; `Delete`/`RangeDelete` verboten; `Erase` nur mit `Erase::admin()`. | Episoden-/Audit-Protokolle |
| `ephemeral()` | Nur Memtable: kein Redo, keine Runs, verfällt beim Schließen. Ein `Change` darf **nicht** dauerhafte und flüchtige Spaces mischen (`Err(Forbidden)`). | Arbeitsgedächtnis, Zugriffsstatistik; ersetzt `memory-only-storage` |

### 6.3 API (Skizze)

```rust
let st = Storage::open(Config::default(), Env::real("/var/lib/app"))?;
// Typisierte Handles: Rd (lesen) < Ap (anhängen) < Wr (schreiben) < Adm (Erase/drop). Rechte lassen sich nur abschwächen.
let facts: Space<Wr> = st.space("facts", SpaceOptions::default().header_len(32).weights(2).separate_body(4096))?;
let trace: Space<Ap> = st.space_append_only("trace", SpaceOptions::default().prefix_filter(8))?;
let facts_ro: Space<Rd> = facts.read_only();

// Atomare Änderung über mehrere Spaces, mit Vorbedingungen und eigener Durability
let r: Receipt = st.apply(
    Change::new()
        .durability(Durability::Always)                        // überschreibt den Instanzwert für diesen Change
        .guard(Guard::absent(&idem, key_k))                    // Idempotenz
        .guard(Guard::equals(&refs, b"main", old))             // Compare-and-Swap
        .guard(Guard::range_unchanged_since(&facts, KeyRange::prefix(b"a/"), s0)) // Merge-Prüfung ohne Vollscan
        .put(&facts, k, Value::new(header, body).domain(KeyDomain(42)))
        .delete(&facts, k2)
        .put(&refs, b"main", new),
)?;                                                            // Err(Conflict{guard_index}) | Busy | Poisoned | Forbidden | …

// Lesen
let view = st.view();                                          // aktueller Stamp
let old  = st.view_at(stamp)?;                                 // nur oberhalb der Untergrenze (Views, Holds, Consumer)
let e = view.get(&facts_ro, k)?;                               // Option<Entry>
//   e.header()  → kein E/A-Zugriff über Run hinaus;  e.body()? → lädt den Körper (Blob) erst jetzt
for kv in view.scan(&facts_ro, KeyRange::prefix(b"a/"), Dir::Fwd)? { let (k, e) = kv?; }
let page = view.scan_page(&facts_ro, KeyRange::prefix(b"a/"), Dir::Fwd,
              PageLimit::weight(1, 6_000).bytes(256_000).entries(200),   // Gewicht 1 = z. B. Token
              Some(&my_filter),                                          // Filter-Pushdown auf den Header
              resume)?;                                                  // Page{ items, next: Option<ResumeToken> }
let est  = view.estimate(&facts_ro, KeyRange::prefix(b"a/"))?; // Estimate{ entries, bytes, weights[k] } nur aus Index
let hist = view.history(&facts_ro, KeyRange::prefix(b"a/"), StampRange::after(s0))?;

// Halten, hören, aufholen, löschen, sichern
let hold = st.hold(stamp, HoldOwner::new("branch/a1"), Some(Duration::from_secs(600)))?;
let cons = st.consumer("vec-index", ConsumerOptions::max_lag_bytes(256 << 20))?;   // persistent, Teil der Untergrenze
cons.advance(applied_stamp)?;                                  // nach Aufbau eines abgeleiteten Index
st.subscribe(listener);                                        // nach Durabilität, in Stamp-Reihenfolge
let q = st.erase(Erase::range(&facts_adm, KeyRange::prefix(b"subject/42/")).views(ViewPolicy::wait(5.secs())))?;
st.erase_batch(vec![e1, e2, e3])?;                             // eine Flush-Barriere für mehrere Aufträge
st.drop_space(facts_adm)?;
st.checkpoint(dest_dir, CheckpointMode::Copy)?;
st.verify(VerifyScope::Online)?;
```

### 6.4 Eigenschaften der API

- **Kein Teilzustand.** `apply` ist alles-oder-nichts. Doppelte Schlüssel innerhalb eines `Change` werden beim Aufbau zusammengelegt (letzter gewinnt). Ein `Change` erhält **einen** Stamp.
- **Guards** werden unter der Schreibsperre gegen den neuesten Stand geprüft, einschließlich früherer Changes derselben Gruppe. Varianten: `absent`, `equals`, `unchanged_since(space,key,stamp)`, `range_unchanged_since(space,range,stamp)`. Letztere prüft über Run-Metadaten (`max_stamp` je Run/Block) und ist **O(überlappende Blöcke)**, kein Vollscan.
- **Rechte als Typen (SD-34).** `Change::put` akzeptiert nur `Space<Wr>` (oder `Space<Ap>` für append-only Spaces); `Erase` nur `Space<Adm>`. Eine Verletzung ist ein **Übersetzungsfehler**, kein Laufzeitfehler. Die obere Schicht reicht an Subagenten abgeschwächte Handles weiter. Die dynamische Rechteprüfung (Macaroons, TTL) bleibt Sache der oberen Schicht.
- **Header-zuerst-Lesen.** `Entry::header()` kostet keinen Körperzugriff; `Entry::body()` lädt ihn und prüft Prüfsumme/AEAD. Filter, Gewichte und Seitenbildung arbeiten ausschließlich auf Headern.
- **Seiten statt Offsets.** `scan_page` begrenzt nach Gewicht, Bytes und/oder Einträgen und liefert eine `ResumeToken` (`space, letzter Schlüssel, Stamp, CRC`). Die Fortsetzung bindet an denselben Stamp; ist er nicht mehr haltbar, folgt `Err(Expired)`, nie eine stillschweigend andere Sicht.
- **`estimate`** summiert Einträge, Bytes und Gewichte aus den Index-Blöcken der Runs (Genauigkeit: Blockgranularität an den Rändern; zählt überschattete Versionen mit, ist also eine **obere Schranke**) und schätzt den Memtable-Anteil durch Abtasten (≤ 1 024 Einträge). Kosten O(Index), keine Datenblöcke. Hat sich die `Weigher`-Version zwischen Runs geändert, meldet `estimate` `exact_weights=false` und rechnet betroffene Runs über die Header nach.
- **`history`** liefert alle Versionen (Puts, Deletes) mit Stamp in `(von, bis]`, gefiltert über die Stamp-Bereiche der Runs. Voraussetzung: Untergrenze ≤ `von`, sonst `Err(HistoryPruned{oldest_available})`.
- **Konsumenten** (`Consumer`) sind persistent, zählen in die Untergrenze und werden bei Überschreiten von `max_lag_bytes` **verworfen** (nächster Zugriff `HistoryPruned`; die obere Schicht baut den Index neu). Ein toter Indexbauer kann den Speicher damit nicht dauerhaft aufblähen.
- **Strukturierte Fehler** mit handlungsleitendem, **datenfreiem** Text (Abschnitt 15).
- `Bytes` ist ein eigener, schlanker Typ (`Arc<[u8]>`-Ausschnitt mit `Deref<Target=[u8]>`), damit Block-Daten ohne Kopie geliefert werden können.

### 6.5 Operationsmatrix: was sich aus den Primitiven bauen lässt

| Fähigkeit der oberen Schicht | Gebaut aus |
|---|---|
| Referenz-Update mit Wiederholungssicherheit | `Guard::equals` auf Ref + `Guard::absent` auf Idempotenzschlüssel in **einem** `Change` |
| Mehrere Tabellen, Index, Ref atomar | ein `Change` über mehrere Spaces |
| Serialisierbare Sitzungen | `view_at(base)` + `Guard::unchanged_since` / `range_unchanged_since` |
| Zeitreise bis Stamp | `view_at` + `Hold` mit Ablauf und Eigentümer |
| Konsistenter Lesezugriff über Index und Daten | eine `View` über alle Spaces |
| Abgeleitete Indizes folgen Änderungen und holen auf | `subscribe` + `Consumer` + `history` |
| Datenschutz-Löschung eines Subjekts | `Erase::range` (Abschnitt 13) **oder** Vernichtung des Domänenschlüssels (14.3) |
| Mandant/Repository löschen | `drop_space` oder Instanz-Verzeichnis löschen nach Schlüsselvernichtung |
| Backup, Test-Klon | `checkpoint` |
| Token-/Byte-budgetiertes Paketieren | `estimate` zur Planung, `scan_page` zur Auslieferung, `ResumeToken` zur Fortsetzung |
| Manipulationsarme Episoden-/Audit-Protokolle | append-only Space (+ Modus `Authenticated`) |
| Zusammen Gebrauchtes liegt nebeneinander | Cluster-Präfix im Schlüssel, `prefix_filter`, `locality_prefix` |
| Vertrauens-/Zeitfilter auf 10⁵ Kandidaten | `Filter` auf den Header (Trust, Gültigkeit, Typ) |
| Vergessen/Verkürzen (Ablauf, „Zoom-Down“) | `CompactionFilter` (10.2) |
| Branches/Versuche (Stufe A) | Overlay-Space + `Hold` (Eigentümer) + `range_unchanged_since` + `history` |
| Zusätzliche Run-Indizes (Volltext, Vektor, Graph) | `Extension`-Blöcke (10.4) oder `Segment`-Dateien (10.5) |
| Arbeitsgedächtnis ohne Disk | `ephemeral`-Space |

### 6.6 Space-Budget und Schlüsselgestaltung (Hinweis für obere Schichten)

Jeder Space hat eigene Memtable, eigenes Run-Verzeichnis und hält über den gemeinsamen Redo den langsamsten Space fest (`redo_lag_bytes`). **Faustregel: ≤ 12 Spaces je Instanz.** Mandanten sind keine Spaces. Eine `remember`-Operation einer oberen Schicht ist ein `Change` über wenige Spaces (z. B. Kopf+Körper, Idempotenz, Kanten), atomar und wiederholungssicher.

---

## 7. Dateilayout

```
<root>/
  FORMAT              Magic, Formatversion, required/optional Feature-Bits, instance_id, integrity_mode, Schlüsselmodus
  LOCK                exklusive (oder geteilte, read-only) Sperre über Fs::lock
  CURRENT             Zeiger auf aktive Catalog-Datei (atomar ersetzt)
  CATALOG-000007      Snapshot-Record + Edit-Records
  KEYS                (nur mit Verschlüsselung) umhüllte Schlüssel je Space und je Domäne, atomar ersetzt
  redo/
    redo-0000000000000012.log        vorallokiert, am Ende gesiegelt (8.2)
  spaces/
    0001/
      0000000000000a3f.run
      b0000000000000071.0003.blob    Blob mit logischer Nummer 0x71, Generation 3 (8.4)
    0002/ …
  segments/<kind>/<id>.seg           Dateien oberer Schichten (10.5), vom Catalog verwaltet
```

Regeln:

- **Alle Dateien werden über numerische IDs benannt**, nie über Pfad-Strings im Catalog. Blobs tragen eine **logische Nummer** (`blob_no`) und eine **Generation**; der Catalog bildet `blob_no → (generation, bytes, live_bytes)` ab. Referenzen in Runs nennen nur `blob_no` und bleiben bei einer Neufassung der Datei gültig (13.3, Blob-Sweep).
- Alle Dateien und Verzeichnisse entstehen mit `Perm::Private` (B-13, 14.6).
- Temporäre Dateien tragen `.building` und eine eindeutige ID aus dem Catalog-Zähler (nicht die Prozess-ID). Beim Öffnen werden alle `.building` entfernt.
- **Verwaiste Dateien** (Run, Blob, Redo-Segment, Segment-Datei, die der Catalog nicht nennt) werden beim Öffnen und danach periodisch gelöscht. Das ist sicher, weil nur ein Catalog-Edit eine Datei lebendig macht (`INV-CAT-1`).
- **Ein Space = ein Verzeichnis** inklusive seiner Blobs. Das Entfernen eines Spaces ist ein Edit plus `remove_dir` nach dem Löschen aller Dateien.
- Eine Blob-Datei gehört zu **genau einem** Space (Löschgrenze bleibt das Verzeichnis).

---

## 8. Formate

Alle Zahlen little-endian. Prüfsumme je Modus: `Plain` = CRC32C (im Crate, tabellenbasiert, safe), `Authenticated` = geschlüsselter MAC (16-Byte-Tag), `Encrypted` = AEAD-Tag (14.1). Jede Dateiart hat **genau eine** Version (`1`) und Feature-Bits. Alle Längen werden vor jeder Allokation gegen Limits geprüft (Prinzip 4).

### 8.1 `FORMAT`

```
"CTXS" | version u16 | required u64 | optional u64 | integrity_mode u8 | instance_id [16] | created_ms u64 | crc32c u32
```

`integrity_mode ∈ {0 Plain, 1 Authenticated, 2 Encrypted}`, nach dem Anlegen **unveränderlich** (ein Wechsel ist ein Migrationswerkzeug, kein Betriebsschalter). Pflicht-Bits: `ENCRYPTED = 1<<0`, `AUTHENTICATED = 1<<1`, `DOMAIN_SEALED = 1<<2`, `COLD_TIER = 1<<3`. Unbekanntes Pflicht-Bit ⇒ `Error::Unsupported{feature}`, kein Öffnen; unbekanntes optionales Bit wird ignoriert. `FORMAT` wird einmal per `write_atomic` geschrieben. Im Modus `Authenticated/Encrypted` ist `FORMAT` selbst durch den Tag aus `KEYS` verankert (14.5).

### 8.2 Redo-Log

Segmentdatei (vorallokiert, 9.5):

```
Header (48 B): "CTXL" | version u16 | flags u16 | segment_id u64 | first_stamp u64 | chain_seed u32 (Entropy)
               | prealloc_len u64 | reserved u64 | crc32c u32
Frame*       : len u32 | stamp u64 | op_count u32 | flags u8 | payload[len-13] | chk
Seal-Frame   : op_count = 0xFFFF_FFFF, flags = SEAL, payload = frame_count u64 ‖ final_chain u32
chk (Plain)  = crc32c(seed = vorheriges chk bzw. chain_seed, len ‖ stamp ‖ op_count ‖ flags ‖ payload)
chk (Auth.)  = MAC_16(len ‖ stamp ‖ op_count ‖ flags ‖ payload ‖ segment_id ‖ frame_index)
chk (Encr.)  = AEAD-Tag; payload verschlüsselt; Zusatzdaten = Header-Felder ‖ segment_id ‖ frame_index
Op           : kind u8 | space u16 | opflags u8 | [domain u32 wenn opflags.DOMAIN] | klen varint | key | vlen varint | value
               (kind: Put, Delete, RangeDelete, EraseRange; value = header ‖ Körper inline)
flags(Frame) : Durability (0 Always, 1 Interval, 2 Never) in Bits 0–1
```

- **Ein Frame = ein `Change`.** Beim Replay ist ein Frame entweder vollständig gültig oder er beendet den Log (`INV-LOG-2`). Ein Gruppen-Schreibvorgang (mehrere Frames, ein `write_all_at`) kann zerreißen; jedes vollständig gültige Präfix ist zulässig (unquittiert, aber dauerhaft ist erlaubt).
- **Körper reisen im Frame.** Der Redo ist der einzige dauerhafte Ort eines Körpers bis zum Flush; erst dann wandert er in die Blob-Datei (SD-27, 10.1). Dadurch bleibt es bei **einem** `fsync` je Gruppe und es gibt keine Reihenfolge-Abhängigkeit zwischen Blob und Redo.
- **Siegel statt Raten (`INV-LOG-6`).** Ein Segment ist *gesiegelt*, wenn sein letzter gültiger Frame ein Seal-Frame ist. **Jedes Segment außer dem neuesten Segment mit Frames ist gesiegelt.** Defekt in einem gesiegelten Segment ⇒ `Corrupt` (Mittendefekt, Fail-closed). Ein ungesiegeltes Segment ist nur zulässig, solange alle späteren Segmente keine Frames enthalten (vorallokierte Reserve). Recovery heilt das letzte ungesiegelte Segment (Abschneiden auf das letzte gültige Frame, `sync_data`), **siegelt** es und legt erst danach ein neues Segment an.
- Die Kettenprüfsumme erkennt vertauschte oder aus einem alten Segment stammende Frames (zufälliges `chain_seed`, deshalb auch bei Wiederverwendung vorallokierter Dateien sicher). Gegen **böswillige** Veränderung schützen nur `Authenticated`/`Encrypted`.
- **Jede Prozessinstanz beginnt ein neues Segment** und hängt nie an ein altes an (`INV-LOG-4`).
- Rotation bei 64 MiB (`Config::redo_segment_bytes`). Ein Segment ist löschbar, wenn **alle** Spaces `flushed_stamp ≥ max_stamp(segment)` haben.

### 8.3 Run

```
Run    := DataBlock* ExtensionBlock* RangeDelBlock? FilterBlock PrefixFilterBlock? IndexBlock MetaBlock DirectoryBlock Footer
Block  := payload | codec u8 | chk                       (chk je Modus wie oben; AAD = Dateiart ‖ id ‖ Blockoffset ‖ space)
Directory : Liste (kind u16 | offset u64 | len u32 | required u8) für index, filter, prefix_filter, rangedel, meta und jeden Extension-Block
Footer (28 B): "CTXR" | version u16 | flags u16 | directory(offset u64, len u32) | crc32c u32 | "CTXR"
```

- **DataBlock** (Ziel 4 KiB): präfixkomprimierte Einträge `shared varint | unshared varint | vlen varint | tag u64 | key_suffix | value`, Neustartpunkt alle 16 Einträge, am Ende das Array der Neustartoffsets. `value = header ‖ (Körper inline | BlobRef)` je nach `kind` (`Put` bzw. `PutRef`).
- **BlobRef (28 B):** `blob_no u64 | offset u64 | len u32 | crc32c u32 | domain u32`. Verweist auf die **logische** Blob-Nummer (8.4), nicht auf eine Datei.
- **IndexBlock:** je Block der kürzeste Trenner ≥ letzter Schlüssel → `(offset u64, len u32, entries u32, bytes u32, weights[k] u32, max_stamp u64)`. Zähler tragen `estimate`; `max_stamp` trägt `Guard::range_unchanged_since` und das Überspringen von Blöcken in `history`.
- **DirectoryBlock:** ein Verzeichnis statt fest verdrahteter Handles. Unbekannte optionale Blöcke werden ignoriert, unbekannte Pflichtblöcke (`required`) verweigern das Öffnen.
- **FilterBlock:** Bloom-Filter über Nutzerschlüssel, **Größe aus der tatsächlichen Eintragszahl** (`bits_per_key = 10`; behebt B-11).
- **PrefixFilterBlock** (`prefix_filter(n)`): Bloom-Filter über die ersten `n` Schlüsselbytes. Er lässt Präfix-Scans Runs ohne Treffer überspringen, was der Punktfilter nicht kann. Angesichts des Verhältnisses 28 : 6 in den realen Aufrufen (1.4) ist er für Haupt-Spaces Standard, nicht Option.
- **ExtensionBlock** (optional): von einer oberen Schicht über `Extension` je Run erzeugt (10.4).
- **RangeDelBlock:** sortierte, fragmentierte Bereichs-Löschmarker `(start, end, tag)`; beim Öffnen vollständig im Speicher (klein). Ein Meta-Flag vermerkt „enthält Bereichsmarker“.
- **MetaBlock:** `space_id`, `run_id`, Eintragszahl, kleinster/größter Nutzerschlüssel, kleinster/größter Stamp, Anzahl Bereichs- und Erase-Marker, `created_ms`, `header_len`, `weights_k`, `weigher_version`, Liste der referenzierten `blob_no` mit Bytesumme, Feature-Bits.
- **Kompression:** `codec` ist 0 (keine). `BlockCodec`-Trait reserviert (SD-13).
- **Leseweg:** `read_at` über `Fs`, kein `mmap`. Der Footer wird beim Öffnen geprüft, Blöcke erst beim Lesen. Eine Plausibilitätsprüfung der Verzeichnisgrenzen (Offset + Länge ≤ Dateilänge, keine Überlappung) erfolgt vor jedem Zugriff.

### 8.4 Blob-Datei

```
Header (48 B): "CTXB" | version u16 | flags u16 | blob_no u64 | generation u32 | space u16 | record_count u32
               | reserved u64 | crc32c u32
Record*      : rlen u32 | kind u8 (Body | Hole) | domain u32 | stamp u64 | klen varint | key | body | chk
Footer       : "CTXB" | record_count u32 | bytes u64 | crc32c u32
```

- **Selbstbeschreibend:** Jeder Record trägt `(domain, stamp, key)`. Dadurch kann `verify` Blobs ohne den Run prüfen, die GC Lebendigkeit prüfen und die Löschprüfung (13.7) Reste finden. Der Chk bindet **Schlüssel und Ort** an den Körper: `Plain` = CRC32C über `domain ‖ stamp ‖ key ‖ body`; `Encrypted` = AEAD mit Zusatzdaten `space ‖ blob_no ‖ offset ‖ BLAKE3_16(key)`, sodass ein vertauschter oder verschobener Körper nicht öffnet.
- **Nonce (Encrypted):** `blob_no u64 ‖ offset u32` (Blob-Dateien ≤ 64 MiB, `blob_no` aus einer nie zurückgesetzten Hochwassermarke), eindeutig je Domänenschlüssel.
- **Append-only während des Baus, danach unveränderlich.** Blobs entstehen ausschließlich im Flush und in der Blob-GC (10.3), beide über `.building` → `sync_all` → `sync_dir` → Catalog-Edit.
- **Generation:** Eine Neufassung mit **gleichen Offsets** (Blob-Sweep bei `Erase`, 13.3) erzeugt Generation `g+1`; gelöschte Records werden zu `Hole` gleicher Länge mit Nullen, übrige Records bleiben **byte-identisch** (nicht neu verschlüsselt, damit kein Nonce doppelt genutzt wird). Der Catalog tauscht `(blob_no, g) → (blob_no, g+1)` in einem Edit; Referenzen in Runs bleiben gültig.
- **Größe:** `blob_target_bytes` = 64 MiB. Das begrenzt zugleich die Kosten eines Sweeps.
- Die Lebendbytes je `blob_no` führt der Catalog (`blob_live_delta`), sodass die GC-Planung keine Datei öffnen muss.

### 8.5 Catalog

```
Datei  : "CTXC" | version u16 | Record*
Record : len u32 | chk | tag u8 (Snapshot | Edit) | payload (TLV-Felder, je Pflicht-/optional-Bit)
```

Ein **Edit** ist ein einzelner Record:

| Feld | Bedeutung |
|---|---|
| `add_files[FileMeta]` | `kind (Run/Blob/Segment/Cold), id, space, level, tier, min_key, max_key, min_stamp, max_stamp, bytes, entries, flags, blob_no_min, blob_no_max, segment_kind, generation` |
| `remove_files[(kind,id)]` | |
| `blob_live_delta[(blob_no, Δbytes i64)]` | Lebendbilanz je Blob, im **selben** Edit wie die Compaction, die Versionen verwirft |
| `blob_replace[(blob_no, gen_alt, gen_neu)]` | Neufassung gleicher Offsets (Sweep) |
| `space_flushed[(space, stamp)]` | Memtable bis `stamp` ist in Runs |
| `redo_floor` | kleinstes noch benötigtes Segment |
| `next_run_id`, `next_blob_no`, `next_segment_id` | Zähler (Hochwassermarke, nie zurück) |
| `spaces_add[(id, name, options)]`, `spaces_drop[id]` | Optionen inkl. `header_len`, `weights`, `separate_body`, `domain_sealed`, `ephemeral`, `append_only` |
| `consumer_set[(name, stamp, max_lag_bytes)]`, `consumer_drop[name]` | persistente Konsumenten-Cursor (6.4) |
| `domain_retired[(domain, stamp)]` | Nachweis einer Schlüsselvernichtung (14.3) |
| `erase_pending_add[(id, space, range, level)]`, `erase_done[id]`, `erase_swept[id]` | Löschprotokoll (13) |
| `erased_ranges_add / _remove` | Register gelöschter Bereiche (13.6) |

- Die Metadaten stehen **im Catalog**: Auswahl für Compaction, Blob-GC und Löschung läuft ohne eine Datei zu öffnen (B-8). Die Reihenfolge der Level-0-Runs ergibt sich aus `max_stamp`.
- **Rollover** (Größe > 8 MiB oder beim Öffnen mit > 1 000 Edits): neue Datei `CATALOG-(n+1)` mit einem Snapshot-Record schreiben, `sync_all`, `CURRENT` per `write_atomic` umhängen, `sync_dir`, alte Datei löschen.
- **Lesen:** ein beschädigter oder halber **letzter** Record wird verworfen (Torn Write), jeder frühere Defekt ist `Error::Corrupt` (Fail-closed, kein „Wiederauferstehen“ gelöschter Dateien).

### 8.6 Segment-Register (Format reserviert, Semantik in 10.5)

`FileKind::Segment(kind u16)` mit `stamp_from`/`stamp_to` (Datenstand, bis zu dem das Segment gilt). Der Kern kennt das innere Format nicht. Er garantiert Lebenszyklus (bauen → dauerhaft → Edit → sichtbar → Graveyard), Aufräumen nach Absturz und Einbezug in `Erase`.

### 8.7 Format-v1-Freeze-Liste (Gate **G-F**, vor F3)

Jeder dieser Punkte würde später eine Formatversion kosten, obwohl SD-04 genau eine je Dateiart verspricht. Sie werden in F0 als **Spezifikation durch Beispiel** (Goldens in `SPEC.md`) festgeschrieben und von einem zweiten Reviewer abgenommen.

| # | Eintrag | Ort |
|---|---|---|
| 1 | `weights[k]`, `weigher_version`, `max_stamp` je IndexBlock-Eintrag | Run |
| 2 | `header_len` je Space | Catalog, MetaBlock |
| 3 | Tag-Kind `PutRef`, `BlobRef` (28 B) inkl. `domain` | Run, Redo |
| 4 | `FileKind` und `FileMeta` (Run/Blob/Segment/Cold), `blob_no_min/max` | Catalog |
| 5 | `blob_no`-Indirektion, `generation`, `blob_live_delta`, `blob_replace`, Hole-Records | Blob, Catalog |
| 6 | `consumer_set`, `max_lag_bytes` | Catalog |
| 7 | Durability-Bits je Frame, `ephemeral`-Flag je Space | Redo, Catalog |
| 8 | `tier` für Blob und Segment (nicht nur Run) | Catalog |
| 9 | `integrity_mode` und Modus-Bits | FORMAT |
| 10 | Redo-Seal-Frame, `prealloc_len`, `chain_seed` | Redo |
| 11 | `domain` in Op und Blob-Record, `domain_retired` | Redo, Blob, Catalog |
| 12 | Reservierte Werte: `kind=Merge(6)`, Codec-IDs, Bit-Bereiche für Pflicht-/optionale Features | alle |

---

## 9. Schreib- und Lesepfad

### 9.1 Zustandsmodell und Sperren

```
Storage
 ├─ committer: Mutex<CommitterState>       Rang 1; genau eine aktive Gruppe; als einziger Lock über E/A gehalten
 ├─ state: RwLock<Arc<Version>>            Rang 2; Haltezeit Nanosekunden (Arc klonen / austauschen)
 ├─ memtable (je Memtable): RwLock<BTreeMap>   Rang 3; Schreiber ist ausschließlich der Leiter
 ├─ registry: Mutex<Registry>              Rang 4 (Blatt); Views, Holds, Consumer, Untergrenze, Claims
 ├─ maint: Mutex+Condvar je Lane           Rang 5 (Blatt); urgent / background
 ├─ visible_stamp: AtomicU64               Release beim Veröffentlichen, Acquire beim Lesen
 └─ env: Env

Version (unveränderlich)
 └─ spaces: Map<SpaceId, SpaceState{ active: Arc<Memtable>, frozen: Vec<Arc<Memtable>>,
                                      levels: Vec<Vec<Arc<FileHandle>>>, blobs: Map<blob_no, Arc<BlobHandle>>,
                                      flushed_stamp, erased_ranges }>
```

Es gibt **fünf** Sperrränge statt acht (B-4) und genau einen Lock über E/A (`committer`). Sperren werden nur in Rangfolge genommen; Verschachtelungen sind auf `committer → state.write` (Rotation) und `committer → memtable.write` (Füllen) beschränkt. Leser berühren `committer` **nie**. Alle Sperren sind `std::sync`; ein vergifteter Mutex/RwLock wird auf `Error::Poisoned{LockPoison}` abgebildet (F13). Das vollständige Protokoll mit Beweis steht in Beilage C.

### 9.2 `apply` (Flat-Combining Group-Fsync)

```
apply(change):
  1. validate: Limits, Spaces vorhanden, Rechte (Typ), Mischverbot ephemeral/dauerhaft, Duplikate     (ohne Sperre)
  2. Ticket in Warteschlange; wer den Committer-Mutex bekommt, wird Leiter, die anderen warten auf ihr Ticket (Condvar je Ticket)
Leiter:
  3. Gegendruck-Prüfung (9.4)
  4. Warteschlange leeren (≤ 1 000 Changes / 8 MiB)
  5. je Change in Reihenfolge: Guards gegen (neueste Version + Gruppen-Overlay) prüfen
       Fehlschlag ⇒ nur dieser Change erhält Err(Conflict), verbraucht keinen Stamp
       Erfolg    ⇒ Stamp = next_stamp++ ; Frame kodieren (siegeln im Modus Authenticated/Encrypted)
  6. alle Frames mit EINEM write_all_at an das Redo-Segment anhängen      Hook: AfterRedoWrite
  7. Sync nach der höchsten Anforderung der Gruppe: enthält sie ein Always ⇒ sync_data
       Fehler ⇒ Poison, alle Tickets Err(Poisoned), NICHTS in Memtables  Hook: AfterRedoSync
  8. Memtables in Stamp-Reihenfolge füllen, in Unterbatches (≤ 64 Ops je Schreibsperre)   Hook: BeforeVisible
  9. visible_stamp.store(letzter Stamp, Release)                          ← Sichtbarkeitspunkt
 10. Tickets mit Receipt{stamp, durable} wecken; Listener-Warteschlange füttern
 11. Memtable über Grenze? ⇒ rotieren (active → frozen), Flush-Job anmelden; Segment voll? ⇒ siegeln, wechseln
```

Folgen:

- **Memtables enthalten nur dauerhafte Daten** (bei `Always`), `INV-VIS-1`. Es gibt keinen Rückbau nach einem Fehler.
- **Teilweises Füllen ist unsichtbar** (`INV-VIS-2`): Solange `visible_stamp` nicht veröffentlicht ist, filtern Leser Einträge mit größerem Stamp aus. Deshalb darf der Leiter die Schreibsperre der Memtable zwischen Unterbatches freigeben, und die Leser-Latenz wird durch **64 Ops** statt durch die Gruppengröße begrenzt (löst das Latenzrisiko von `RwLock<BTreeMap>` bei großen Gruppen).
- Ein einziger `fsync` je Gruppe; Gruppengröße ergibt sich aus der Last (kein Zeitfenster-Parameter).
- **Durability je Change** (`Always | Interval(d) | Never`, überschreibt `Config`): `Always` quittiert nach `sync_data`; `Interval` quittiert sofort mit `durable=false`, die Wartung synchronisiert alle `d`; `Never` nur für Tests und Wegwerf-Instanzen. Redo ist ein gemeinsames Log: **Ein `Always` in der Gruppe macht alle früheren Frames derselben Gruppe mit dauerhaft** (Präfix-Eigenschaft). `Storage::sync()` erzwingt.
- Stamps sind lückenlos über die **angenommenen** Changes.
- **Ephemerale Spaces** schreiben keinen Frame; ihre Changes laufen durch dieselbe Gruppe, aber ohne Redo-Anteil (kein Mischen mit dauerhaften Spaces).

### 9.3 Sichtbarkeit, Views und Untergrenze

`View` = `(Arc<Version>, stamp)`. Lesen ignoriert Einträge mit größerem Stamp. Die **Untergrenze** `retention_floor` ist **monoton nicht fallend** und wird unter dem Registry-Lock berechnet als `min(visible_stamp, Stamp aller lebenden Views, Stamp aller Holds, consumed_stamp aller Consumer)`. Registrierung einer View, Prüfung `stamp ≥ retention_floor` bei `view_at` und Berechnung der Untergrenze laufen unter **demselben** Registry-Lock; daraus folgt `INV-RET-1` (Compaction verändert keine Sicht ≥ Untergrenze), siehe Beweis in Beilage C. Holds und Consumer haben Ablauf bzw. Maximalrückstand (`Clock`, `max_lag_bytes`); abgelaufene werden von der Wartung entfernt (keine Zombie-Holds). Ein Hold trägt einen **Eigentümer** (z. B. Branch-Name), damit ein verworfener Branch seine Halteregeln zuverlässig freigibt.

### 9.4 Gegendruck

Maßgeblich sind: eingefrorene Memtables, Level-0-Runs, ausstehende Compaction-Bytes, Redo-Rückstand (`redo_lag_bytes`) und der Speicherstand des `Governor`.

| Stufe | Auslöser | Wirkung |
|---|---|---|
| Normal | unter Verlangsamung | keine |
| Verlangsamt | L0 ≥ 8 oder frozen ≥ 2 oder Governor weich | Leiter wartet bis zu `stall_slice` je Gruppe; Governor verkleinert Caches |
| Gestoppt | L0 ≥ 12 oder frozen ≥ 4 oder Governor hart | `apply` wartet höchstens `stall_timeout`, danach `Err(Busy{retry_after})` |

`Busy` ist strukturiert und wiederholbar (wichtig für Agenten). Der Flush läuft auf der **urgent**-Lane (10) und wird von Compaction nie verzögert; das verhindert, dass der Zustand „gestoppt“ durch eine lange Compaction entsteht.

### 9.5 Redo-Segment-Lebenszyklus, `ENOSPC` und Poison

**Vorallokation (neu).** Die `background`-Lane legt das *nächste* Segment im Voraus an: `.building` erzeugen, `preallocate(redo_segment_bytes)` (Standard: Nullen schreiben, `Fs`-Implementierungen dürfen `fallocate` nutzen), Header schreiben, `sync_all`, umbenennen, `sync_dir`. Eine Rotation im `apply` ist dann ein Wechsel auf eine fertige Datei: Der Commit-Pfad enthält weder `create` noch `sync_dir`. Begründung: Auf vorallokierten, bereits beschriebenen Dateien muss `sync_data` keine Dateisystem-Metadaten (Blockzuordnung) schreiben; das ist die bekannte Absenkung der Latenz von `fdatasync` (Hypothese H-S13, Messung nötig). Ein Nebeneffekt: `ENOSPC` tritt **bei der Vorallokation** auf, nicht mitten im Commit.

**Ende-Erkennung.** Der vorallokierte Rest besteht aus Nullen; ein Frame mit `len == 0` beendet das Segment. Das `chain_seed` (zufällig je Segment) verhindert, dass Frames einer früheren Nutzung der Datei als gültig erkannt werden.

**Fehlerregeln beim Anhängen (neu, schließt eine Lücke von v1).**

| Fehler | Aktion | Zustand danach |
|---|---|---|
| `write_all_at` scheitert mit `ENOSPC` oder Kurzschreiben | **Abschneiden auf den letzten guten Offset** (`set_len(good)` + `sync_data`), Tickets der Gruppe `Err(Busy{StorageFull})` | heil, wiederholbar; Segment bleibt ungesiegelt-und-gültig |
| Abschneiden scheitert **oder** anderer `write`-Fehler (`EIO` …) | **Poison** | tot; Neustart heilt das Segment in der Recovery |
| `sync_data` scheitert | **Poison** (`INV-LOG-5`), nie wiederholen, nichts in die Memtable | tot; nach Neustart gilt das Frame, falls es dauerhaft wurde, sonst nicht |

Ohne die erste Zeile würde ein `ENOSPC` nach Teilschreiben einen Torn-Tail mitten im Log hinterlassen, den die nächste Recovery wegen `INV-LOG-6` als `Corrupt` einstufen müsste.

**Poison-Semantik.** Ein vergiftetes Handle verweigert alle Schreibvorgänge und Wartungsjobs mit `Error::Poisoned{cause}`. **Lesen bleibt erlaubt** (degradierter Betrieb): Memtables enthalten nur Daten aus erfolgreich synchronisierten Gruppen, Runs sind unverändert und der Catalog zeigt nur dauerhaft gemachte Dateien (`INV-VIS-1`, Prinzip 3). Ein fehlgeschlagener `fsync` einer Run-Datei geschah vor dem Catalog-Edit; die Datei war nie lebendig. `Storage::health()` meldet `Healthy | Poisoned{cause}`. Kern und `contextra-durable-fs` des Altstands kennen eine ähnliche Zustandsmaschine (`HealthState`); sie dient als Vorlage.

**Siegeln.** Beim Rotieren hängt der Leiter ein Seal-Frame an (`sync_data`), bevor das nächste Segment genutzt wird (`INV-LOG-6`).

### 9.6 Lesepfad

- **`get`:** aktive Memtable → eingefrorene (neu nach alt) → Level 0 (neu nach alt, nur Runs mit passendem Schlüsselbereich) → je tieferes Level genau ein Run (Binärsuche über Catalog-Metadaten). Pro Run: Bloom-Filter, Index, **ein** Block. Bereichs-Löschmarker aus Memtables und berührten Runs werden gegen die Versionen geprüft. Der Wertkörper (`PutRef`) wird **erst bei `Entry::body()`** gelesen.
- **`scan`** liefert einen `Cursor` (Streaming, mischend). Die Memtable wird in Blöcken zu 64 Einträgen mit erneutem Aufsetzen (`seek_after(last_key)`) gelesen, sodass keine Lesesperre über die Dauer eines Scans gehalten wird. Bereichs-Löschmarker des Scan-Bereichs werden einmal eingesammelt und als Intervallliste mit maximalem Stamp geprüft. Folge-Blöcke eines sequentiellen Scans werden per `read_batch` **vorgelesen** (Tiefe über `Governor` begrenzt).
- **`scan_page`, `estimate`, `history`** nutzen dieselbe Zusammenführung. `estimate` liest nur Index-Blöcke (im Cache), `history` verwirft Runs und Blöcke, deren Stamp-Bereich `(von, bis]` nicht schneidet (`max_stamp`).
- **Filter-Pushdown:** `Filter::keep(key, header)` wird **vor** dem Laden eines Körpers ausgewertet (Trust-Stufe, Gültigkeitszeit, Typ). Mehrfach-`get` (`get_many`) bündelt die benötigten Blöcke zu einem `read_batch`.
- **Präfix-Scans** fragen zuerst den `PrefixFilterBlock` der Runs ab.
- **Caches:** `TableCache` hält höchstens `max_open_files` offene Dateien (Runs und Blobs; löst Dateideskriptor-Grenzen). `BlockCache`: genau eine Implementierung (CLOCK, geteilt über `Governor`, Quote je Instanz), Schlüssel `(file_id, block_no)`, `invalidate_file`. Körper werden **nicht** im `BlockCache` gehalten (Seitencache des Betriebssystems genügt; ein kleiner optionaler `BodyCache` ist Hypothese H-S15).
- **`AccessSink`** erhält gesampelt `(space, file, block_no, key_prefix, bytes)`; Standard ist die Null-Senke.
- Lesefehler (`Corrupt`, `Io`) werden **immer** als Fehler gemeldet, nie als „leer“ (F6, Gegenstück zu B-9a).

---

## 10. Wartung: Lanes, Flush, Compaction, Blob-GC

### Lanes, Claims, Prioritäten (SD-16 geändert)

v1 sah **einen** Wartungsjob je Instanz vor. Das blockiert den Flush hinter einer langen Compaction und führt bei Lastspitzen in den Zustand „gestoppt“ (9.4). v2 führt **zwei Lanes** ein, je höchstens ein Job, beide auf der geteilten `Runtime`:

| Lane | Jobs (absteigende Priorität) | Eigenschaft |
|---|---|---|
| `urgent` | `Erase` (E2–E7) > `Flush` > Intervall-`fsync` | kurz, darf nie hinter Compaction warten |
| `background` | `Compaction` (L0 zuerst) > Blob-GC > Segment-Bau > Redo-Vorallokation > Cleanup (Graveyard, Verwaiste, abgelaufene Holds/Consumer) | lang, ratenbegrenzt, abbrechbar |

**Claim-Tabelle (Rang 4, in der Registry).** Ein Job reserviert beim Planen **alle** Eingabedateien in einem einzigen kritischen Abschnitt (alles oder nichts). Der Planer ist eine reine Funktion `plan(&Version, &Params, floor, &Claims)` und überspringt beanspruchte Dateien. Beanspruchen ein `urgent`- und ein `background`-Job dieselbe Datei (z. B. `Erase` und Compaction desselben Runs), setzt der `urgent`-Job ein `cancel`-Flag. Der `background`-Job prüft es an **Ausgabegrenzen** (spätestens alle 16 MiB Eingabe), verwirft seine `.building`-Ausgabe, gibt die Claims frei und beendet sich. Es gibt keinen Wartezyklus: Claims werden atomar genommen und von keinem Job gehalten, der auf einen anderen wartet. Damit bleibt „Compaction während `Erase`“ per Konstruktion ausgeschlossen, ohne dass `Erase` hinter Compaction steht.

**Drosselung.** Die `background`-Lane läuft unter Token-Buckets des `Governor` (Lese- und Schreibbytes je Sekunde, `Clock`-basiert); `urgent` ist nur durch Speicher begrenzt.

### 10.1 Flush (je Space)

```
1. frozen Memtable → Run bauen (.building): Einträge in Schlüsselreihenfolge;
   bei separate_body/domain_sealed: Körper in derselben Reihenfolge in eine Blob-Datei (.building) schreiben,
   Run enthält PutRef (Körper werden beim Flush erstmals mit Blob-Nonce versiegelt)
2. Run und Blob(s): sync_all ; Verzeichnis sync_dir
3. Edit { add_files[Run, Blob(s)], blob_live_delta, space_flushed(space, max_stamp), redo_floor, Zähler }
   → Catalog append + sync_data                                                             ◄── Durability-Punkt
4. neue Version veröffentlichen (frozen entfernen, Dateien hinzufügen)
5. Redo-Segmente unterhalb redo_floor ins Graveyard
```

Körper in Schlüsselreihenfolge im Blob erhalten **Leselokalität**: ein Präfix-Scan über Volltexte liest zusammenhängende Bereiche. `redo_floor` wird nur angehoben, wenn **alle** Spaces die Segmente abdecken. Hinkt ein selten beschriebener Space hinterher und übersteigt das Redo-Volumen `max_redo_bytes` (Standard 512 MiB), erzwingt die Wartung dessen Flush (auch mit kleinem Run); `stats()` zeigt `redo_lag_bytes`.

### 10.2 Compaction (ein Verfahren: Leveled)

- **Planer = reine Funktion**, property-testbar. Die adaptive, kostenbasierte Planung entfällt (SD-11).
- Auslöser: L0-Anzahl ≥ 4 ⇒ alle L0-Runs plus überlappende L1-Runs. Tiefere Level: Größe > `target(level)` (Faktor 10) ⇒ Run mit höchstem Score plus überlappende Runs des Zielniveaus. Ausgabe wird bei `target_run_bytes` (64 MiB) geteilt, bevorzugt an Präfixgrenzen (`locality_prefix`, 10.5).
- **Aufbewahrung (`retain.rs`, rein):** je Schlüssel alle Versionen mit Stamp > `floor` behalten, die neueste mit Stamp ≤ `floor` behalten, ältere verwerfen. Löschmarker werden nur verworfen, wenn unterhalb kein Run den Bereich überlappt (aus Catalog-Metadaten) **und** nichts unter der Untergrenze liegt.
- **`CompactionFilter` (neu, SD-41):**

```rust
pub trait CompactionFilter: Send + Sync {
    /// Nur für Versionen mit stamp <= floor. Rein: Ergebnis hängt nur von den Argumenten ab.
    fn decide(&self, key: &[u8], header: &[u8], stamp: Stamp, now_ms: u64) -> Decision;
}
pub enum Decision { Keep, Drop, DropBody { new_header: Option<Vec<u8>> } }   // DropBody = „Zoom-Down“
```

  Ablauf von Idempotenz-Schlüsseln, Arbeitsgedächtnis und Vergessen (Volltext weg, Zusammenfassung bleibt) brauchen keine eigene Maschinerie. Der Filter wirkt **nur unterhalb der Untergrenze** (Views, Holds und Consumer sehen unveränderte Daten), `now_ms` wird je Job einmal aus `Clock` gelesen, Bereichs- und Erase-Marker sind ausgenommen. Er ist kein Merge-Operator (SD-12), weil er nur entfernt oder verkürzt. Er läuft ohne gehaltene Sperre (Prinzip 10) und wird im Modelltest gegen das Referenzmodell mitgeprüft.
- **Installation:** Ausgabe-Runs `sync_all` + `sync_dir`, dann **ein** Edit `{add_files, remove_files, blob_live_delta}`, dann neue Version, dann Eingabedateien per `Arc`-Drop ins Graveyard.
- Verworfene Versionen mit `BlobRef` buchen ihre Bytes im selben Edit vom Lebendstand ab (`blob_live_delta`); ein Blob mit `live_bytes == 0` wird im selben Edit entfernt.

### 10.3 Blob-GC und Graveyard

**Blob-GC ist compaction-getrieben.** Runs sind unveränderlich und tragen den *Ort* eines Körpers; ihn zu verschieben heißt, den Run neu zu schreiben. Deshalb gilt:

1. Die Menge `E` der GC-berechtigten Blobs: `live_bytes / bytes < 0,4` oder Alter über `blob_gc_max_age`.
2. Eine Compaction, die einen Eintrag mit `BlobRef ∈ E` verarbeitet, kopiert den Körper in die **neue Blob-Datei der Ausgabe** und setzt den `BlobRef` im Ausgabe-Run neu (neue `blob_no`, neuer Nonce, Stamp unverändert). Verschlüsselte Körper werden neu versiegelt (Domänenschlüssel nötig).
3. Werden `E`-Blobs von der regulären Compaction nicht erreicht, plant der Planer eine **Umschreib-Compaction** (gleiches Level, gleiche Schlüssel) über die Runs, deren `[blob_no_min, blob_no_max]` `E` abdeckt (Kandidaten aus Catalog-Metadaten, Bestätigung über den MetaBlock).
4. Ist der Domänenschlüssel eines Körpers **vernichtet** (`KeyProvider::domain_key → None`), wird der Körper nicht kopiert; der Eintrag erhält `BlobRef{blob_no = 0, domain}` („Körper vernichtet“, `Entry::body()` ⇒ `Err(Shredded{domain})`).

Das Verfahren verändert nie einen Stamp; `history`, `Consumer` und `ResumeToken` bleiben unberührt. Es nutzt dieselben Claims, Edits und Absturzregeln wie jede Compaction.

**Graveyard.** `FileHandle`/`BlobHandle` führen ein `obsolete`-Flag. Beim Drop der letzten Referenz mit gesetztem Flag wird der Pfad in das Graveyard gelegt, **ohne E/A im Drop**. Die Wartung löscht per `Fs::remove_file`, danach `sync_dir`. Fehlschläge werden wiederholt und als Metrik/Fehler gemeldet (B-9d). Nach Absturz räumt die Verwaisten-Suche auf (`INV-CAT-4`).

### 10.4 Extensions (zusätzliche Index-Blöcke je Run)

```rust
pub trait Extension: Send + Sync {
    fn id(&self) -> u16;                                   // stabile Block-Art
    fn build(&self, entries: &mut dyn Iterator<Item = Entry>) -> io::Result<Vec<u8>>;
    fn merge(&self, inputs: &[&[u8]], survivors: &mut dyn Iterator<Item = Entry>) -> io::Result<Vec<u8>>;
}
```

Der Kern liefert **keine** Implementierung. Er garantiert: Der Block entsteht im selben Run-Bau (Flush und Compaction), steht im `DirectoryBlock`, wird mit dem Run atomar lebendig und mit dem Run gelöscht. `merge` muss Überlebende (nach `retain` und `CompactionFilter`) berücksichtigen, damit gelöschte Einträge nicht über den Index sichtbar bleiben. Aufrufe laufen ohne gehaltene Sperre und hinter `catch_unwind` (F12). **Eignung:** Extensions passen für Strukturen, die mit den Daten zerlegt und zusammengeführt werden (Postings, Bloom, Zähler). Die Schnittstelle gilt bis F10 als instabil (A-10).

### 10.5 Segmente und Lokalität

**Segmente (SD-30).** Strukturen, die sich nicht mit den Runs zerlegen lassen (Vektorgraphen, Graph-Adjazenz, große Volltext-Segmente), liegen als `Segment`-Dateien mit eigenem Format in der Hand der oberen Schicht. Der Kern liefert Lebenszyklus, Absturzsicherheit und Löschung. Ein Index liest `Segmente ∪ history(stamp_to, view.stamp]`; der Aufholzeiger ist ein `Consumer` (er hält die Versionen, die der Index braucht, und wird bei Überschreiten von `max_lag_bytes` verworfen).

| Index | Mechanismus | Begründung |
|---|---|---|
| Postings, Bloom, Zähler | `Extension` je Run | billig, zerlegt mit den Daten |
| Volltext (BM25) | Extension **oder** Segment | beides tragfähig; Start mit Extension |
| Vektor (ANN) | **Segment** | Graphbau teuer, Lokalität folgt Cluster, nicht Schlüssel |
| Graph (Adjazenz, PPR) | Space (Kanten); Segment nur für Mehrschritt-Abfragen | Adjazenz-Scan genügt zunächst |

**`Erase`-Fähigkeit:** Ein Segment-Format muss `rewrite_without(range)` anbieten. Ohne diese Methode verwirft der Kern bei Überlappung das **ganze** Segment und verlangt den Neuaufbau (13.3).

**Lokalität.** Die Reihenfolge ist die Schlüsselreihenfolge. Wer zusammen gelesen wird, liegt zusammen, wenn die obere Schicht dem Schlüssel einen Cluster-Präfix voranstellt. `locality_prefix(n)` lässt die Compaction Ausgaben bevorzugt an Präfixgrenzen teilen (±25 %). Ein Umordnen nach Zugriffsmustern ist eine Re-Key-Operation der oberen Schicht (Hysterese, Lastfenster) auf Basis der `AccessSink`-Spur; der Kern ordnet **nie** nach Zugriff um (SD-26).

### 10.6 Kalter Speicher (Phase F10)

`FileMeta.tier ∈ {Local, Cold}` gilt für **alle** Dateiarten (Run, Blob, Segment). Ein `ColdTier`-Port wird als `File`-Implementierung hinter `TableCache` eingebunden; Dateien sind unveränderlich und deshalb ohne Koordination hochladbar. **Folge für die Löschung:** E6 schließt `delete` des Ports ein; `Physical` ist für kalte Dateien nur so weit zusagbar, wie der Port Löschung garantiert; belastbar bleibt `Cryptographic` (A-11).

---

## 11. Recovery

### 11.1 Ablauf beim Öffnen (deterministisch, 10 Schritte)

```
 1. Fs::lock(LOCK): Exclusive (lesend-schreibend) oder Shared (open_readonly: heilt nichts, startet keine Wartung); scheitert ⇒ Err(Locked)
 2. FORMAT lesen; unbekanntes Pflicht-Feature ⇒ Err(Unsupported); Modi Authenticated/Encrypted: KEK holen, KEYS-Tag prüfen
 3. CURRENT ⇒ Catalog-Datei; Records lesen (letzter Torn-Record verworfen, sonst Corrupt); Zustand aufbauen
    (Spaces, FileMeta, Blob-Tabelle, Consumer, flushed_stamps, redo_floor, pending erases, erased_ranges, domain_retired)
 4. *.building löschen; Dateien, die der Catalog nicht nennt (Runs, Blobs inkl. nicht aktueller Generationen, Segmente, Redo unterhalb des floor), löschen
 5. Vom Catalog genannte Datei fehlt ⇒ Err(Corrupt{MissingFile{kind,id}}) (Fail-closed, kein Raten)
 6. Redo-Segmente ab redo_floor in Id-Reihenfolge lesen:
      gültige Frames anwenden, je Op nur wenn stamp > space.flushed_stamp (domänenversiegelte Körper entsiegeln;
         Domänenschlüssel vernichtet ⇒ Körper verwerfen, Eintrag „vernichtet“)
      Defekt in gesiegeltem Segment ⇒ Err(Corrupt)                                   (INV-LOG-6)
      Ungesiegeltes Segment mit nachfolgendem Segment mit Frames ⇒ Err(Corrupt)
      letztes ungesiegeltes Segment: auf letztes gültiges Frame abschneiden, sync_data, SIEGELN, sync_data
 7. next_stamp = max(letzter Redo-Stamp, max flushed_stamp, max Run-Stamp) + 1; visible_stamp = next_stamp − 1
 8. Vorallokiertes Reservesegment übernehmen oder neues Segment anlegen (nie anhängen, INV-LOG-4); optional Catalog-Rollover
 9. Consumer prüfen: Rückstand > max_lag_bytes ⇒ als verworfen markieren; Untergrenze berechnen
10. Pending Erases und ausstehende Blob-Sweeps (Catalog ∪ Erase-Frames im Redo ohne erase_done) in die urgent-Lane (Roll-forward, 13.4)
```

Reihenfolge 3→4 ist wichtig: **erst** den Catalog als Wahrheit lesen, **dann** aufräumen. Runs und Blobs werden **nicht** geöffnet (Metadaten stehen im Catalog, H-S5). Im Shared-Modus entfallen die Schritte 4, 6 (Heilen/Siegeln), 8 und 10; ein Torn-Tail wird als Ende des Logs gelesen.

### 11.2 Absturzmatrix (Soll-Zustand je Unterbrechungspunkt)

| Unterbrechung | Zustand auf Platte | Ergebnis nach Neustart |
|---|---|---|
| vor Redo-`write` | nichts | Change nicht vorhanden, nie quittiert |
| mitten im `write` / vor `fsync` | halbes oder ungeflushtes Frame | Frame wird abgeschnitten oder fehlt, nie quittiert |
| nach `fsync`, vor Quittung | Frame dauerhaft | Change vorhanden (zulässig) |
| `ENOSPC` nach Teilschreiben, Abschneiden gelang | Segment endet beim letzten guten Frame | Neustart unauffällig |
| `ENOSPC`, Abschneiden gescheitert → Poison, Absturz | Torn-Tail im letzten Segment | Heilen + Siegeln (11.1 Schritt 6) |
| Recovery-Heilen: nach Abschneiden, vor Siegel | Segment kurz, ungesiegelt | nächste Recovery wiederholt Heilen + Siegeln (idempotent) |
| Siegel geschrieben, nächstes Segment fehlt | Segment gesiegelt, kein Nachfolger | neues Segment wird angelegt |
| Reservesegment: Absturz beim Vorallokieren | `.building` | gelöscht, Reserve wird neu angelegt |
| Flush: nach Run/Blob-`sync`, vor Edit | verwaiste Run- und Blob-Dateien | Verwaisten-Suche löscht, Memtable aus Redo wiederhergestellt |
| Flush: nach Edit, vor Version | Dateien im Catalog | lebendig, Redo-Frames mit Stamp ≤ `flushed_stamp` übersprungen |
| Compaction/Blob-GC: nach Ausgabe, vor Edit | verwaiste Ausgaben | gelöscht, Eingaben unverändert |
| Compaction: nach Edit, vor Unlink | Eingaben verwaist | gelöscht (Verwaisten-Suche) |
| Blob-Sweep: neue Generation gebaut, vor Edit | zusätzliche Generationsdatei | gelöscht (nicht die aktuelle Generation) |
| Blob-Sweep: nach Edit, vor Unlink | alte Generation | gelöscht |
| Catalog-Rollover: nach neuer Datei, vor `CURRENT` | alte `CURRENT` gültig | neue Datei verwaist, gelöscht |
| Catalog-Rollover: nach `CURRENT`, vor Löschen der alten Datei | beide vorhanden | alte Datei verwaist, gelöscht |
| `Consumer::advance`: Absturz | Edit atomar | alter oder neuer Stand, nie dazwischen |
| `fsync`-Fehler zur Laufzeit | unbestimmt | Instanz vergiftet, kein Retry, Neustart führt Recovery aus |

Diese Tabelle wird zu einem **Test** (15.2): `FaultFs` bricht an jedem E/A-Index ab und prüft den Soll-Zustand.

---

## 12. Betrieb

- **`verify(Online | Offline)`.** *Online:* Catalog ↔ Dateien (Existenz, Länge, Footer), Stichproben-Prüfsummen, `blob_live_delta`-Bilanz gegen die Runs, Consumer/Hold-Stände. *Offline:* alle Blöcke und Records, jeder `BlobRef` löst auf einen Record gleichen Schlüssels/Stamps auf, jede Blob-Datei ist von mindestens einem Run referenziert oder Hole, Redo-Kette und Siegel. Befund je Datei, nie nur ein Ja/Nein.
- **`checkpoint(dest, Copy | Hardlink)`.** Eine `View` plus `Hold` legt den Stand fest; die Wartung beansprucht die Dateien (Claims); Dateien sind unveränderlich und werden kopiert, das Redo bis zum Stand und ein Catalog-Snapshot werden geschrieben. Das Ergebnis ist ein normal öffenbares Verzeichnis. `Hardlink` ist Opt-in mit Warnung (ein Hardlink hält Dateien nach einem Unlink am Leben und verletzt `INV-ERA-2` für die Kopie).
- **`open_readonly`.** Geteilte Sperre, keine Heilung, keine Wartung; geeignet für `verify`, Sicherung, Inspektion.
- **`stats()` und Metriken** (über `Metrics`): `redo_lag_bytes`, `held_bytes` (durch Holds/Consumer festgehaltene Bytes), `blob_live_ratio`, Schreib- und Platzverstärkung, Zustand der Lanes, Claims, `stall`-Zeiten, Cache-Trefferquoten, Gegendruckstufe, `health`. Metriken enthalten keine Schlüssel/Werte (Prinzip 12).
- **Lebenszyklus.** `Storage::hibernate()` flusht, gibt Memtable und Caches der Instanz frei und hält nur Catalog-Zustand (für ruhende Mandanten, Abschnitt 16.3). `close()` flusht optional, siegelt das Redo-Segment, gibt die Sperre frei.
- **Formatentwicklung.** Neue Fähigkeiten kommen über optionale Feature-Bits; ein Pflicht-Bit verweigert alten Lesern das Öffnen. Eine neue Dateiversion ist ein eigenes Migrationswerkzeug, nie ein Zweitpfad im Kern (SD-04). Der Plan friert in 8.7 ab, was ohne Formatbruch nicht nachrüstbar wäre.

---

## 13. Löschung (Erasure)

Dieser Abschnitt ist die Antwort auf „atomare physische Löschung durch sauberes Anlegen“. Das saubere Anlegen besteht aus vier Maßnahmen in der Struktur: **(1)** Spaces sind physisch getrennt (Löschgrenze = Verzeichnis, Blobs eingeschlossen), **(2)** der Catalog kennt Schlüsselbereich, Stamp-Bereich und Blob-Referenzbereich jeder Datei (Planung ohne Dateizugriff), **(3)** Dateien werden ausschließlich nach einem durablen Edit gelöscht (Reihenfolge ist Vertrag), **(4)** sensible Körper sind je Domäne versiegelt, sodass Schlüsselvernichtung den Körper auch in Redo und Blob unlesbar macht.

### 13.1 Was „atomar“ und „physisch“ hier heißt

- **Logisch atomar:** Ab Stamp `S` sieht keine View mit Stamp ≥ `S` irgendetwas aus dem Löschbereich, für alle Schlüssel zugleich (der Löschmarker ist ein Eintrag in einem Redo-Frame).
- **Physisch atomar:** Es gibt **genau einen** Durability-Punkt (einen Catalog-Edit), vor dem die alten Dateien lebendig und nach dem sie nicht mehr referenziert sind. Ein Absturz davor lässt den Vorgang fortsetzen, ein Absturz danach das Löschen der Dateien (Roll-forward, nie Rückabwicklung).
- **Nachweisbar:** Eine `ErasureReceipt` wird erst ausgestellt, nachdem die Dateien gelöscht und das Verzeichnis synchronisiert wurde.

### 13.2 Stufen und ehrliche Garantien

| Stufe | Verfahren | Schützt gegen | Schützt **nicht** gegen |
|---|---|---|---|
| `Logical` | Löschmarker | Lesen über die API | Dateiinhalt auf Platte |
| `Physical` (Standard) | Runs ohne den Bereich neu schreiben, Blob-Sweep, Catalog-Edit, Unlink, Verzeichnis-`fsync`, Redo-Segmente entfernen | Lesen von Dateien im Verzeichnis, Wiederauftauchen nach Absturz | Reste im Dateisystem-Journal, Flash-Wear-Leveling, Snapshots/Backups außerhalb, Hardlinks |
| `Scrubbed` | wie `Physical`, vor dem Unlink Datei mit Nullen überschreiben und synchronisieren | zusätzlich einfache Wiederherstellung gelöschter Blöcke auf klassischen Datenträgern | Copy-on-Write-Dateisysteme und SSDs (Best-Effort, dokumentiert) |
| `Cryptographic` | Schlüssel der `KeyDomain` bzw. des Spaces vernichten, **bevor** Dateien gelöscht werden | alle Reste, auch Backups ohne Schlüssel, Flash-Reste, **auch Redo-Frames** (Körper sind domänenversiegelt, 14.3) | Kopien des Schlüssels |

Konsequenz: **Echte Unwiederbringlichkeit auf modernen Datenträgern liefert nur `Cryptographic`.** Das Design legt sensible Körper deshalb in `domain_sealed`-Spaces (14.3); `Physical` minimiert die Rückstände im Dateisystem. Das wird nicht beschönigt.

### 13.3 Protokoll `Erase::range(space, range)` mit Stamp `S`

```
E0  Vorprüfung: Space existiert, Recht Adm, Bereich gültig; Views/Holds/Consumer mit Stamp < S gemäß ViewPolicy / HoldPolicy
E1  Redo-Frame mit Op EraseRange(space, range) (normaler apply-Pfad), fsync                   ◄── LOGISCHER Punkt, Stamp S
      Ab hier verdeckt der Marker alle Einträge mit Stamp < S im Bereich für Views ≥ S
E2  urgent-Lane belegen (Claims auf betroffene Dateien; Compaction wird abgebrochen, 10)
    Flush-Barriere: alle Memtables aller Spaces bis S flushen, redo_floor anheben,
                    Redo-Segmente mit max_stamp ≤ S ins Graveyard   (Redo mischt Spaces)
    — bei Erase::batch gilt EINE Barriere für alle Aufträge der Gruppe —
E3  Dateien wählen: Runs des Spaces mit überlappendem [min_key, max_key]; Blobs über die in den Runs
    (MetaBlock) und aus E4 ermittelten BlobRefs                                                  (aus Catalog-Metadaten)
E4  Je Run: streamen, alle Einträge mit Stamp ≤ S im Bereich verwerfen (Holds je HoldPolicy),
    Ausgabe-Run bauen (sync_all), Verzeichnis sync_dir; vollständig überdeckte Runs erzeugen keine Ausgabe
E4b Blob-Sweep: je Blob mit gelöschten Körpern neue Generation g+1 mit GLEICHEN Offsets bauen:
    gelöschte Records → Hole (Nullen), übrige Records byte-identisch; sync_all, sync_dir
E5  EIN Edit { add_files(Runs), remove_files(Runs), blob_replace[…], blob_live_delta,
               erase_done(S), erased_ranges_add(range, S) }  → Catalog append + sync_data   ◄── PHYSISCHER Punkt
E6  Neue Version veröffentlichen; BlockCache.invalidate_file; alte Runs und alte Blob-Generationen ins Graveyard;
    Graveyard wartet auf Rückgabe aller Referenzen (ViewPolicy), (Scrubbed: überschreiben + sync) → remove_file → sync_dir
E7  Edit { erase_swept(S) } (nicht kritisch), ErasureReceipt{ id=S, level, runs_rewritten, runs_removed,
    blobs_swept, holes_bytes, redo_segments_removed, segments_rebuilt, bytes_freed, completed_ms } zurückgeben
```

- **Kryptografisch je Space:** E3–E4b entfallen; E5 enthält zusätzlich `spaces_drop`; der Space-Schlüssel wird **vor** E5 vernichtet (`INV-ERA-3`).
- **Kryptografisch je Domäne (`Erase::domain(d)`):** Der Aufrufer vernichtet `d` im `KeyProvider` (außerhalb des Crates, mit dessen eigener Dauerhaftigkeit). Der Kern **verweigert** den Auftrag, solange `domain_key(d)` noch einen Schlüssel liefert (`Err(KeyStillPresent)`), leert den Schlüssel-Cache der Domäne, schreibt `domain_retired` und überlässt die Platzfreigabe der Blob-GC (10.3, Punkt 4). Redo-Frames bleiben ohne Flush-Barriere unlesbar. Es gibt **keine** Flush-Barriere und keinen Run-Umbau.
- **Segmente (10.5):** Überlappende Segmente werden über `rewrite_without(range)` neu gebaut oder ganz verworfen und vom Besitzer neu aufgebaut; der Edit nimmt das Segment in `add_files/remove_files` auf.
- `drop_space` ist der Schnellpfad: Schlüssel vernichten (falls verschlüsselt), ein Edit `{spaces_drop, remove_files[alle]}`, Graveyard, `remove_dir`.
- Schreibzugriffe nach `S` im selben Bereich bleiben erlaubt; E4 verwirft nur Stamp ≤ `S`.
- **Kosten:** `Erase::range` kostet O(Bytes der berührten Runs) **plus** O(Bytes der berührten Blob-Dateien, je ≤ 64 MiB). Der Sweep ist der teuerste Teil und der Grund, `Cryptographic` für große oder häufige Löschungen zu bevorzugen.

### 13.4 Recovery der Löschung (Roll-forward)

Die Menge der offenen Löschungen ist `Catalog.erase_pending ∪ {EraseRange-Frames im Redo nach dem Anker} − erase_done`. Beim Öffnen werden sie in die urgent-Lane gelegt und ab E2 fortgesetzt. E3–E5 sind idempotent (Auswahl aus dem aktuellen Catalog; ein bereits durchgeführter E5 ist an `erase_done` erkennbar). E6/E7 sind idempotent über die Verwaisten-Suche (alte Run-Dateien und nicht aktuelle Blob-Generationen sind verwaist). Ein Flush, der einen Erase-Marker in einen Run schreibt, trägt `erase_pending_add` im selben Edit.

### 13.5 Nebenläufigkeit mit Views, Holds und Consumern

| Policy | Verhalten |
|---|---|
| `ViewPolicy::Wait(timeout)` (Standard) | Löschen der Dateien wartet, bis alle Views mit Verweis auf die alten Dateien zurückgegeben sind. Nach Timeout: `Err(Blocked{views})`; der Vorgang bleibt in E6 offen und wird wiederholt (der logische Teil ist bereits garantiert). |
| `ViewPolicy::Revoke` | Betroffene Views werden **widerrufen** (`Err(Revoked)` beim nächsten Zugriff), geben ihre Version frei, danach werden die Dateien gelöscht. |
| `HoldPolicy::Fail` (Standard) | Ein Hold oder Consumer mit Stamp < `S` über dem Bereich ⇒ `Err(HoldBlocked{holds})`. |
| `HoldPolicy::Override` | Hold wird für den Bereich ungültig, Consumer wird verworfen (Datenschutz hat Vorrang). |

### 13.6 Kein stilles Fälschen der Historie

Der Catalog führt je Space ein Register `erased_ranges: [(range, S)]`. Liest eine View mit Stamp < `S` in einen dort verzeichneten Bereich, antwortet sie mit `Err(Erased{range, erased_at: S})`. Es gibt **keine Fallback-Antwort und keine leere Antwort**. Einträge werden entfernt, sobald die Untergrenze ≥ `S` ist.

### 13.7 Invarianten und Prüfung

| ID | Invariante |
|---|---|
| `INV-ERA-1` | Logische Atomarität: für jede View ≥ `S` ist der Bereich vollständig leer; für jede View < `S` vollständig `Erased`. Keine Mischsicht. |
| `INV-ERA-2` | Nach `erase_done(S)` enthält weder ein lebendiger Run noch ein lebendiges Redo-Segment eine Version im Bereich mit Stamp ≤ `S`. |
| `INV-ERA-3` | Schlüssel vor Datei: Ein vernichteter Schlüssel ist dauerhaft vernichtet, bevor zugehörige Dateien gelöscht werden; der Kern führt den Auftrag nie aus, solange der Schlüssel noch liefert. |
| `INV-ERA-4` | Nur Roll-forward: Ein begonnener `Erase` wird nach jedem Absturz fortgesetzt oder ist nie begonnen. |
| `INV-ERA-5` | Reads unterhalb `S` in den Bereich liefern `Erased`, nie Daten und nie „leer“. |
| `INV-ERA-6` | **(neu)** Nach `erase_done(S)` enthält keine lebende Blob-Generation einen Body-Record im Bereich mit Stamp ≤ `S`; die Stellen sind `Hole` mit Nullen. |
| `INV-ERA-7` | **(neu)** Ein Körper einer vernichteten Domäne ist weder in Redo noch in Blob ohne den Schlüssel lesbar; `Entry::body()` liefert `Shredded{domain}`, nie Daten und nie „leer“. |

**Prüfwerkzeuge:**

1. `Storage::verify_erasure(id)`: prüft `INV-ERA-2/6` anhand Catalog-Metadaten und Stichproben-Scan der noch lebendigen Dateien mit Überlappung; Befund je Datei.
2. **`ErasureOracle` (Feature `testing`):** schreibt Schlüssel/Werte mit eindeutigem Muster (Header **und** Körper, mit und ohne Domäne), führt `Erase` aus und durchsucht danach **alle Bytes aller Dateien im Verzeichnis** (Runs, Blobs aller Generationen, Redo, `.building`, Segmente, `MemFs`-Zustand) nach dem Muster. Bei ausgeschalteter Verschlüsselung muss die Trefferzahl 0 sein; bei `Cryptographic` darf das Muster auch ohne Schlüsselzugriff nicht im Klartext auftauchen und **mit** vernichtetem Schlüssel nicht rekonstruierbar sein. Für Vektor-Segmente zusätzlich eine Rückrechnungsprüfung (Embedding des gelöschten Textes darf nicht mehr als Nachbar auftauchen).
3. **Crash-Matrix über E1–E7 inkl. E4b** (15.2): `FaultFs` bricht an jedem E/A-Index ab. Nach jedem Neustart gelten `INV-ERA-1…7`, und bei fortgesetztem Vorgang endet das Orakel mit 0 Treffern.
4. **Backup-Hinweis:** `checkpoint` ist standardmäßig `Copy`; `Hardlink` ist Opt-in mit Warnung.

### 13.8 Grenzen (offen benannt)

Append-only Spaces lassen `Erase` nur mit `Erase::admin()` zu; der Vorgang ist im Receipt vermerkt. Kalte Dateien unterliegen den Garantien des `ColdTier`-Ports (10.6). Speicherinhalt, Auslagerungsdatei und Prozess-Core-Dateien werden nicht kontrolliert (Puffer werden best-effort überschrieben, ohne Garantie, `forbid(unsafe_code)`). Externe Snapshots/Backups, Dateisystem-Journal, Flash-Wear-Leveling siehe Tabelle 13.2. Die orchestrierende Ebene (Saga, Quittungen, signierte Berichte; im Altstand das Crate `contextra-erasure`) liegt **über** diesem Primitiv; die `ErasureReceipt` ist dafür serialisierbar und stabil.

---

## 14. Sicherheit und Verschlüsselung (Feature `encryption`)

### 14.1 Integritäts- und Vertraulichkeitsmodi (SD-33)

| Modus | Schutz | Verfahren | Feature | Gegen Angreifer |
|---|---|---|---|---|
| `Plain` | Zufallsfehler | CRC32C (Blöcke, Frames, Records), Kettenprüfsumme im Redo | – | A1 nur über Dateirechte; **nicht** A2/A3 |
| `Authenticated` | Integrität gegen Manipulation, ohne Vertraulichkeit | geschlüsselter MAC (BLAKE3, 16-Byte-Tag) mit Zusatzdaten (Dateiart, ID, Offset, Space) | `encryption` | A3 (Manipulation) |
| `Encrypted` | Vertraulichkeit + Integrität | AEAD (ChaCha20-Poly1305), Schlüssel per KDF | `encryption` | A2, A3 |

`Authenticated` ist neu: Es deckt den häufigen Fall „Daten dürfen lesbar sein, dürfen aber nicht unbemerkt verändert werden“ (Audit-Protokolle, append-only Spaces) ohne Verschlüsselungskosten ab. Der Modus steht in `FORMAT` und ist unveränderlich. **Eigene Kryptografie wird nicht geschrieben**; verwendet werden geprüfte Crates (SD-01), `getrandom` für Zufall über `Entropy`.

### 14.2 Schlüsselhierarchie und Nonces

```
KEK (KeyProvider, extern)                      32 Byte
 ├─ Space-DEK (zufällig, Entropy)              mit KEK umhüllt, in KEYS (atomar ersetzt), je Space
 │    ├─ Run-Schlüssel   = KDF(DEK, "ctxs run v1"  ‖ run_id)
 │    └─ Blob-Schlüssel  = KDF(DEK, "ctxs blob v1" ‖ blob_no)   (für nicht domänenversiegelte Körper)
 ├─ Redo-Schlüssel       = KDF(KEK, "ctxs redo v1" ‖ segment_id)
 └─ Katalog-/FORMAT-Schlüssel = KDF(KEK, "ctxs cat v1")
Domänenschlüssel (je Subjekt, KeyProvider::domain_key(d), NICHT in KEYS)   Option<32 Byte>; None = vernichtet
```

**Nonce-Konstruktion (zentral, 12 Byte, Property-Test: nie doppelt je Schlüssel):**

| Verwendung | Nonce | Eindeutig weil |
|---|---|---|
| Run-Block | `0x01 ‖ run_id u56 ‖ block_no u32` | `run_id` aus Hochwassermarke, nie wiederverwendet |
| Redo-Frame (Redo-Schlüssel) | `0x02 ‖ segment_id u32 ‖ frame_index u32 ‖ 0 u24` | Schlüssel je Segment |
| Redo-Op-Körper (Domänenschlüssel) | `0x03 ‖ segment_id u32 ‖ frame_index u32 ‖ op_index u24` | Position im Log eindeutig |
| Blob-Record (Space-/Domänenschlüssel) | `0x04 ‖ blob_no u56 ‖ offset u32` | `blob_no` nie zurückgesetzt, Datei ≤ 64 MiB |

`run_id`, `blob_no` < 2^56 und `segment_id` < 2^32 werden erzwungen (`Exhausted`). Eine Blob-Neufassung gleicher Offsets (Sweep) übernimmt Records **byte-identisch**; GC-Umzug vergibt eine neue `blob_no` (neuer Nonce).

### 14.3 Domänenschlüssel: kryptografisches Löschen je Subjekt (neu, SD-42)

Problem: Eine sortierte LSM mischt Einträge vieler Subjekte in einem Block. Verschlüsselung je Subjekt auf Blockebene ist deshalb unmöglich, ohne die Schlüsselordnung (Lokalität) zu opfern. Lösung: **Körper-Ebene.**

- Header und Schlüssel bleiben unter dem Space-DEK, sodass Ranking, Filter und Gewichte weiter ohne Domänenschlüssel laufen und Metadaten nach einer Schlüsselvernichtung erhalten bleiben (Provenienz, `supersedes`-Spur).
- Der **Körper** eines Eintrags aus einem `domain_sealed`-Space wird bereits in `apply` mit dem Schlüssel seiner `KeyDomain` versiegelt, **bevor** er in den Redo-Frame geht; die Memtable hält Klartext (im Speicher, A5), der Flush versiegelt neu mit Blob-Nonce.
- Wird der Domänenschlüssel vernichtet (außerhalb des Crates im `KeyProvider`), sind die Körper in Redo **und** Blob ohne Flush-Barriere und ohne Run-Umbau unlesbar (`INV-ERA-7`). Die Blob-GC gibt den Platz später frei (10.3).
- Der Domänenschlüssel-Cache (TTL, begrenzte Größe) wird bei `Erase::domain(d)` sofort geleert.
- Skalierung: Der Kern speichert **keine** Domänenschlüssel (`KEYS` enthält nur Space-DEKs, ≤ 12). Ein Schlüsselbund mit Millionen Subjekten ist Sache des `KeyProvider` der oberen Schicht.
- **Nicht gelöst (offen, O-2):** Embeddings und Postings, die aus einem Körper abgeleitet wurden, enthalten Rückstände. Sie müssen in **Segmenten je Löschdomäne** (Tenant oder Subjektklasse) liegen oder über `rewrite_without` neu gebaut werden (10.5, 13.3).

### 14.4 Typisierte Zugriffsrechte (SD-34)

`Space<R>` mit Rechtemarken `Rd < Ap < Wr < Adm`, versiegelte Traits, Abschwächung (`read_only()`, `append_only()`) ohne Rückweg. `Change::put`/`delete` akzeptieren nur `Wr` (bzw. `Ap` für append-only Spaces), `Erase`/`drop_space` nur `Adm`, `View::get/scan` jedes Handle. Die Prüfung kostet zur Laufzeit nichts (Übersetzungsfehler). Der Kern beantwortet damit „wer darf was im Prozess?“ für den Fall A4; dynamische, delegierbare Berechtigungen mit Ablauf (Macaroons) bleiben Sache der oberen Schicht, die abgeschwächte Handles ausgibt. `open_readonly` (12) liefert nur `Rd`-Handles.

### 14.5 Rollback-Anker und Verankerung (F10)

Authentisierte Blöcke schützen nicht davor, dass ein **älterer, in sich stimmiger Gesamtstand** zurückgespielt wird. v2 benennt das offen und bietet einen optionalen Port:

```rust
pub trait RollbackAnchor: Send + Sync {
    fn record(&self, catalog_generation: u64, head_mac: [u8; 32]) -> io::Result<()>;   // nach dauerhaftem Edit (ratenbegrenzt)
    fn latest(&self) -> io::Result<Option<(u64, [u8; 32])>>;                           // beim Öffnen
}
```

Im Modus `Authenticated/Encrypted` trägt jeder Catalog-Record einen Ketten-MAC; beim Öffnen vergleicht der Kern `(Generation, Kopf)` mit `latest()` und meldet `Err(Rolledback{expected, found})`. Der Anker (TPM-Zähler, entfernte Protokollierung, separates Medium) liegt außerhalb des Crates; das Fenster zwischen zwei `record`-Aufrufen bleibt als Restrisiko dokumentiert. `FORMAT` und `KEYS` tragen den Tag aus dem Katalog-Schlüssel, sodass ein Austausch einzelner Dateien auffällt.

### 14.6 Dateisystem-Rechte (B-13)

- `Fs::create_dir_all(…, Perm::Private)` ⇒ `0700`, `OpenMode::CreateNew(Perm::Private)` ⇒ `0600`, **unabhängig von der `umask`** (explizit gesetzt). Gilt für `FORMAT`, `KEYS`, `CURRENT`, Catalog, Redo, Runs, Blobs, Segmente, `.building`, `LOCK`.
- Das Wurzelverzeichnis gilt als vertrauenswürdig (A1: kein anderer Benutzer darf dort schreiben). `RealFs` prüft vor dem Öffnen mit `symlink_metadata`, dass kein Symlink auf das Wurzelverzeichnis zeigt; eine vollständige Härtung (`O_NOFOLLOW`, `openat`) braucht Plattformaufrufe und gehört in eine alternative `Fs`-Implementierung.
- Windows: restriktive ACL und `Perm::Private` sind in v1 nicht zugesagt (A-02).

### 14.7 Umgang mit Geheimnissen und Informationslecks

- **`Secret32`:** kein `Debug`/`Display`/`Clone`/`Serialize`; beim `Drop` wird der Puffer mit `fill(0)` und `black_box` überschrieben. Das ist **best-effort**; ohne `unsafe` ist kein volatiler Schreibvorgang garantierbar, und das steht in der Doku.
- **Vergleiche** von Tags und Schlüsseln laufen in der Krypto-Bibliothek (konstante Zeit).
- **Keine Daten in Fehlern, Logs und Metriken:** `Error::hint()` enthält nur Typen, Zahlen und Namen von Spaces/Konsumenten, **nie** Schlüssel- oder Wertbytes, Subjekt-IDs oder Domänen-IDs im Klartext (nur numerische, opake Kennungen, die der Aufrufer vergeben hat). Ein **Lecks-Orakel** im Testnetz prüft alle in der Suite erzeugten Fehlertexte und Metriknamen auf die Muster der Testdaten.
- **Kein Dedup über Mandanten:** Der Kern adressiert nichts über Inhaltshashes; damit entfällt die Seitenkanal-Klasse „existiert dieser Text bei jemand anderem?“ im Kern.
- **Gesalzene Hashes** für `BLAKE3_16(key)` in AAD (Schlüssel-Bindung) nutzen den Space-DEK als Schlüssel (`BLAKE3 keyed`), nie unverschlüsselte Klartext-Hashes.

### 14.8 Restrisiken (offen)

| Risiko | Stand |
|---|---|
| Rollback ganzer Stände ohne Anker | nicht erkennbar (14.5) |
| Klartext im Prozessspeicher, Swap, Core-Dumps | nicht kontrolliert (A5) |
| Schlüsselmaterial im `KeyProvider` | Sache der einbettenden Anwendung |
| Seitenkanäle durch Zugriffsmuster (Größe, Zeit) | nicht gegenmaßnahmenbewehrt; Padding wäre Sache der oberen Schicht |
| `Plain` gegen Manipulation | nicht geschützt; Dokumentation und `FORMAT`-Feld weisen darauf hin |

---

## 15. Fehlermodell und Teststrategie

### 15.1 Netze

| Netz | Inhalt |
|---|---|
| **Modelltest** | Referenzmodell (`BTreeMap` mit Versionsliste je Schlüssel, Holds, Consumer) gegen die Engine: zufällige Folgen aus `apply` (inkl. Guards, Durability, ephemeral), `view_at`, `scan`, `scan_page`, `history`, Flush, Compaction, Blob-GC, `CompactionFilter`, `Erase`, Neustart. Eigener Xorshift-PRNG, Seed wird bei Fehlschlag ausgegeben. |
| **Gesetze (proptest)** | Dekodieren ∘ Kodieren = Id für alle Formate; Run-Lesen = Eingabe; Compaction verändert keine Sicht ≥ Untergrenze (`INV-RET-1`); `retain` idempotent; Planer liefert nie überlappende Ausgaben auf Leveln ≥ 1; Nonce-Eindeutigkeit (kein doppeltes `(Schlüssel, Nonce)`); Blob-Lebendbilanz = Summe der `PutRef` in lebenden Runs. |
| **Absturzmatrix** | `FaultFs` bricht an **jedem** E/A-Index ab (15.2); danach `open`, Vergleich mit Modell-Präfix der quittierten Changes, `verify` grün. |
| **Fehlerklassen-Matrix (neu)** | Jede Klasse F1–F13 aus 3.1 hat mindestens einen gezielten Test mit definiertem Zustand danach (Tabelle 3.1, Spalte „Netz“). |
| **Linearisierbarkeit (neu)** | `testing::linearize`: nebenläufige Läufe (n Schreiber, m Leser, Wartung) werden aufgezeichnet; die Prüfung weist eine Ordnung nach, die der Stamp-Folge entspricht und alle gelesenen Sichten erklärt (Wing-Gong-artig, wegen der Stamp-Ordnung linear in der Historie). |
| **Gesetz „Sichtbarkeit“ (neu)** | Mit `Hooks` an `AfterRedoSync`/`BeforeVisible` pausiert: keine `View` sieht einen halben Change; nach der Freigabe sieht jede neue View alle Ops oder keine. |
| **Bitflip** | Jedes einzelne Byte jeder Datei (Run, Blob, Catalog, Redo, `FORMAT`, `KEYS`) wird gekippt: Ergebnis ist `Err(Corrupt)` oder identisch (nie falsche Daten); im Modus `Authenticated/Encrypted` ausnahmslos `Err`. |
| **Fuzz** | Decoder für Redo-Frame/Seal, Run-Block/Footer/Directory, Blob-Record, Catalog-Record, `FORMAT`, `KEYS`, Schlüsselbereiche, `ResumeToken` (13 bestehende Ziele werden portiert, 3 neue: Blob, Seal, ResumeToken). **Stateful-Fuzz:** `arbitrary`-erzeugte Aktionsfolgen gegen das Modell (ersetzt `fuzz_compaction_interleave`/`fuzz_recovery_arbitrary_state`). |
| **Nebenläufigkeit** | `loom` für die Synchronisationsstücke (Ticket-Warteschlange/Leiterwahl, Version-Austausch, Registry/Untergrenze, Claim-Tabelle); Gesamtsystem über `Runtime::manual()` mit zufälliger Einplanung und `Hooks`. |
| **Panic-Injektion (neu)** | Panik in `Weigher`, `Filter`, `CompactionFilter`, `Extension`, `Listener`: Daten unverändert, Job schlägt fehl, Instanz bleibt benutzbar (F12). Panik im Committer-Abschnitt ⇒ Poison (F13). |
| **Schichtregel** | `tests/layering.rs` (5.3). |
| **Lösch-Orakel** | siehe 13.7. |
| **Lecks-Orakel (neu)** | Alle Fehlertexte, Metrik- und Logausgaben der Suite werden auf Muster der Testdaten (Schlüssel, Werte, Domänen) durchsucht: 0 Treffer. |
| **Ressourcentests (neu)** | Allokationsbudget im Heißpfad, Dateideskriptor- und Speichergrenzen, Leerlauf-Fußabdruck (16.5). |
| **Differentialtest (neu, begrenzt)** | Schlanke Harness gegen die Bibliotheks-API des rekonstruierten Altstands (1.4) für `put/get/delete/scan/commit` mit Stromausfall-freien Läufen; Abweichungen sind Befunde, keine automatischen Fehler (der Altstand hat Fehler, B-9). Nur soweit die Baseline in F0 lauffähig wird. |
| **Soak** | Langläufer mit Agenten-Lastprofil (viele kleine Changes, kurzlebige Views, seltene Erases, 4-KiB- und 64-KiB-Körper), Platte-voll, Speicherdruck. |
| **Goldens** | Je Dateiart Referenzdateien inkl. Freeze-Liste (8.7); Byte-Stabilität über Plattformen. |

**CI-Gates:** `cargo test` (alle Netze außer Soak/Fuzz-lang), `clippy -D warnings`, `cargo doc` ohne Warnungen, `cargo deny` (Abhängigkeiten, Lizenzen), `cargo miri test` für `codec` und `crypto`, Fuzz-Smoke (je Ziel 60 s), Absturzmatrix, `loom`-Subset. Nachts: Fuzz lang (je Ziel ≥ 30 min), Soak, Benchmarks gegen die Hypothesen (Regressionsgrenze ±10 %).

### 15.2 `FaultFs` (Stromausfall- und Fehlermodell)

Kernstück der Qualität. `MemFs` hält je Datei zwei Zustände: *Seitencache* (alles Geschriebene) und *dauerhaft* (Stand der letzten `sync_data`/`sync_all`). Verzeichniseinträge (Anlegen, `rename`, `remove`) sind erst nach `sync_dir` dauerhaft. `FaultFs` kann:

- an E/A-Operation Nr. *n* einen **Absturz** auslösen: danach zählt nur der dauerhafte Zustand; nicht synchronisierte Schreibvorgänge werden **verloren, teilweise (zerrissen, in Sektorgrößen 512 B/4 KiB) oder in zufälliger Reihenfolge** übernommen,
- einen `fsync`/`sync_dir` **fehlschlagen** lassen (Poison-Pfad, `INV-LOG-5`),
- „Platte voll“ (`write_all_at` → `StorageFull`) und Kurzschreibvorgänge simulieren, auch **mitten in einem Gruppen-Schreibvorgang** (Regel 9.5),
- `rename` ohne `sync_dir` rückgängig machen,
- **(neu)** **Lesefehler** (`EIO`) und **Bitfäule** beim Lesen einzelner Offsets,
- **(neu)** **verlorene Schreibvorgänge** (Platte quittiert, schreibt nicht) und **fehlgeleitete Schreibvorgänge** (Daten landen an falschem Offset derselben Datei) einschleusen,
- **(neu)** das Zurückspielen eines älteren Gesamtstands (F10) für den Test des Rollback-Ankers,
- jede Operation mit einer **Hook-Marke** versehen, sodass Tests die Reihenfolge `AfterRedoWrite`, `AfterRedoSync`, `BeforeVisible` usw. kontrollieren.

Die Matrix läuft für `apply` (inkl. Gruppen), Flush mit und ohne Blob, Compaction, Blob-GC, Blob-Sweep, Catalog-Rollover, Redo-Vorallokation und -Siegel, `Erase` (E1–E7 inkl. E4b), `drop_space`, Verschlüsselungsschlüsselwechsel, `Consumer::advance`, `checkpoint`.

### 15.3 Überführung des Bestandes

Die Datei `SPEC.md` (F0) katalogisiert die Invarianten des Bestands (INV-STORE-1, INV-FSYNC-1, INV-WAL-*, INV-LOCK-ORDER-1, …) und ordnet jedem ein Netz aus 15.1 zu. Aus den 160 Testdateien werden die **Szenarien** übernommen, nicht der Code (der Bestandstestkorpus ist nicht übersetzbar, B-14). Verworfen werden Tests, die Legacy-Formate (`wal_legacy_*`, `hmac_legacy_*`, `format_version_compat`), `rollback_to_tx`, den adaptiven Planer oder `docid-128` prüfen. `MergeOperator`-Tests werden nach SD-12 zurückgestellt (B-15).

### 15.4 Fehlermodell und Limits

Ein einziges `Error`-Enum (kein `Storage(String)`), jeder Fehler hat `code()` (stabil), `retryable()`, strukturierte Felder und `hint()`: einen kurzen, handlungsleitenden, **datenfreien** Text (14.7), lesbar für Menschen und Sprachmodelle. Beispiele: „Space ‚trace‘ ist append-only; Schlüssel existiert bereits. Neuen Schlüssel wählen.“ oder „Stamp 880 nicht mehr verfügbar; älteste verfügbare Sicht: 1 012.“

| Variante | Bedeutung | `retryable` |
|---|---|---|
| `Conflict{guard_index}` | Vorbedingung nicht erfüllt | nein (neu lesen) |
| `Busy{retry_after, reason}` | Gegendruck, `StorageFull`, Governor | ja |
| `Poisoned{cause}` | `fsync`-/E/A-/`LockPoison`-Fehler, Instanz vergiftet | nein (Neustart) |
| `Corrupt{file, offset, what}` | Prüfsumme/Struktur falsch | nein |
| `Erased{range, erased_at}` | Zugriff unterhalb eines Löschstamps | nein |
| `Shredded{domain}` | Körper einer vernichteten Schlüsseldomäne | nein |
| `Revoked` | View widerrufen | nein |
| `HoldBlocked{holds}` / `Blocked{views}` | Löschung durch Halteregel blockiert | ja (nach Freigabe) |
| `KeyStillPresent{domain}` | `Erase::domain` bei noch vorhandenem Schlüssel | nein (Schlüssel zuerst vernichten) |
| `Limit{what, max}` | Schlüssel/Wert/Change zu groß | nein |
| `Forbidden{space, op}` | Operation im Space nicht erlaubt (append-only, Mischen ephemeral/dauerhaft) | nein |
| `Expired` | `ResumeToken` oder Stamp nicht mehr haltbar | nein (neu beginnen) |
| `HistoryPruned{oldest_available}` | Verlauf unterhalb der Untergrenze, verworfener Consumer | nein |
| `Rolledback{expected, found}` | Rollback-Anker verletzt (14.5) | nein |
| `Unsupported{feature}` | Pflicht-Feature unbekannt | nein |
| `Locked`, `Closed`, `NotFound`, `Exhausted`, `Io(kind)` | | je nach Fall |

**Limits (konfigurierbar, Fehlercode `Limit`):** Schlüssel ≤ 4 KiB, Header ≤ 255 B, Wert ≤ 1 MiB (konfigurierbar bis 16 MiB; größere Daten gehören in eine Objektschicht, A-03), Change ≤ 32 MiB und ≤ 100 000 Ops, Redo-Frame ≤ 64 MiB, `weights` ≤ 4, Spaces ≤ 64 (Empfehlung ≤ 12, 6.6), Consumer ≤ 64, Holds je Instanz ≤ 10 000.

---

## 16. Ressourcen und Effizienz (neu)

Ziel Z-12: begrenzter, vorhersagbarer Verbrauch, kein Thread je Instanz, Leerlauf nahe null, faire Teilung zwischen vielen Instanzen (AR-6).

### 16.1 Governor

Ein geteilter `Governor` (ein Objekt je Prozess, von vielen `Storage` genutzt) führt:

| Budget | Verfahren | Wirkung bei Knappheit |
|---|---|---|
| **Memtable-Gesamtsumme** | Quote je Instanz (Gewichtung), harte Obergrenze | früherer Flush; Gegendruck `Verlangsamt/Gestoppt` |
| **BlockCache** | CLOCK, ein globaler Pool, Mindestanteil je Instanz | Einträge fremder Instanzen verdrängen, Mindestanteil bleibt |
| **Offene Dateien** | globaler `TableCache` mit LRU-Ausschluss, Quote je Instanz | Dateien schließen, bei Bedarf neu öffnen (Footer-Cache bleibt) |
| **E/A-Raten** | Token-Buckets für Lesen/Schreiben der `background`-Lane, gewichtet je Instanz | Compaction wird gedrosselt, `urgent` nicht |
| **Vorlesen (Prefetch)** | Tiefe begrenzt | weniger Vorlesen |
| **Wartungs-Threads** | `Runtime` mit `n` Threads (Standard: min(4, Kerne)), Jobs je Instanz höchstens zwei (Lanes) | Warteschlange, faire Reihenfolge nach Rückstand |

`PressureSource` (optional, z. B. aus cgroup-Speichergrenzen) hebt die Stufe an: *weich* ⇒ Caches schrumpfen, Flush früher; *hart* ⇒ `Busy` statt Allokation. Der Kern fängt keine Allokationsfehler ab (Rust bricht bei OOM ab); er vermeidet sie durch Budgets, **gemessen** in Ressourcentests (16.5).

### 16.2 Speicherbudget (Struktur)

| Posten | Größenordnung | Steuerung |
|---|---|---|
| Memtable (aktiv + eingefroren) | `memtable_bytes` je Space (Standard 16 MiB) × (1 + frozen ≤ 3) | Governor-Quote |
| Pro offener Run: Footer, Index, Filter | ≈ 1–2 % der Datenbytes (Bloom 10 Bit/Schlüssel, Index je 4 KiB-Block) | `TableCache`, bei ruhenden Instanzen freigegeben |
| Pro Run: Bereichs-Löschmarker | klein, vollständig im Speicher | – |
| Block-Cache | globaler Pool | Governor |
| Körper | **kein** eigener Cache im Standard (Seitencache des Betriebssystems) | H-S15 |
| Offene Ansichten | `Arc<Version>` hält Dateien am Leben | `held_bytes` in `stats()`, `View`-Ablaufwarnung |

Index und Filter eines Runs werden **lazy** beim ersten Zugriff geladen und sind verdrängbar; der Catalog hält nur `FileMeta`.

### 16.3 Viele Instanzen (ein Mandant, ein Agent)

- Kein Thread je Instanz; alle Hintergrundarbeit über die geteilte `Runtime`.
- `Storage::hibernate()` (12) für ruhende Instanzen: Memtable flushen, Caches und Handles freigeben, Wartung einstellen. Wiedererwachen beim nächsten Zugriff **transparent und ≤ H-S5** (Öffnen < 50 ms ohne Redo).
- Leerlauf: kein Timer-Thread je Instanz; Intervall-`fsync`, Hold-Ablauf und Aufräumen laufen über **einen** gemeinsamen Timer-Job der `Runtime` (Hypothese H-S16: Leerlauf-CPU < 0,1 %, bei 1 000 ruhenden Instanzen < 1 %).
- Dateideskriptor-Bedarf je ruhender Instanz: 0 (außer `LOCK`; bei `hibernate` wird die Sperre gehalten, andere Handles nicht).

### 16.4 E/A-Effizienz

| Maßnahme | Wirkung |
|---|---|
| Header-zuerst-Lesen, Körper separat | Rankings lesen keine Volltexte (H-S11) |
| `read_batch` und Vorlesen bei Scans | weniger Systemaufrufe, höhere Warteschlangentiefe auf NVMe (H-S12) |
| Gruppen-`fsync`, Vorallokation der Redo-Segmente | ein `fsync`, kein Metadatenschreiben im Commit-Pfad (H-S2, H-S13) |
| Wert-Trennung | Schreibverstärkung für große Körper ≈ 2 (Redo + Blob) statt bis 15 (Leveled) (H-S10) |
| `PrefixFilterBlock`, Blockbereich-`max_stamp` | überspringen Runs/Blöcke bei Präfix-Scans und `history` |
| Compaction-Planung ohne Dateizugriff | keine Wartungs-Lesevorgänge für Entscheidungen |
| Aufräumen im Graveyard, nicht im `Drop` | keine E/A auf dem Leserpfad |
| Block-Cache-Schlüssel `(file_id, block_no)` mit `invalidate_file` | keine veralteten Blöcke nach Erase/Compaction |

### 16.5 Allokationsbudget und Ressourcentests

Ein `CountingAllocator` im Testnetz (`#[global_allocator]` nur in der Testbinärdatei, nie im Crate) misst:

| Pfad | Budget (Hypothese, in F5 kalibriert) |
|---|---|
| `view.get` (Treffer im Block-Cache, Körper nicht angefordert) | **0** Heap-Allokationen (Rückgabe nutzt `Bytes`-Ausschnitt) |
| `view.scan` je Eintrag | ≤ 1 (nur wenn Schlüssel rekonstruiert werden muss) |
| `apply` einer Gruppe | ≤ `2 + Anzahl Changes` (Ticket, Frame-Puffer wiederverwendet) |
| Flush/Compaction | Puffer je Job einmal, nicht je Block |

Weitere Ressourcentests: begrenzter Speicher unter Last (RSS-Obergrenze je Konfiguration), Dateideskriptor-Obergrenze (`ulimit -n 256` mit 1 000 Runs), 1 000 ruhende Instanzen (Fußabdruck und CPU), Gegendruck unter simuliertem Speicherdruck.

### 16.6 Leistungsmodell (Hypothesen)

| ID | Hypothese | Messung |
|---|---|---|
| H-S1 | Punktlesen mit warmem Block-Cache `p99 < 50 µs`; kalt höchstens Anzahl Level × 1 Block-E/A | F3, F5 |
| H-S2 | Ein `apply` mit `Durability::Always` kostet ein `sync_data`; bei 64 parallelen Schreibern ≥ 20× Durchsatz gegenüber seriellem Fsync | F5 |
| H-S3 | Leveled mit Faktor 10: Schreibverstärkung ≤ 15, Platzverstärkung ≤ 1,3 (ohne Holds) **für Header-Daten**; zusätzlich mit 4-KiB- und 64-KiB-Werten gemessen | F6 |
| H-S4 | Wiederanlauf ≤ 1 s je 256 MiB Redo-Replay | F4/F5 |
| H-S5 | Öffnen ohne Redo und mit 1 000 Runs < 50 ms (Metadaten aus Catalog, Runs lazy) | F4 |
| H-S6 | `drop_space` mit 1 000 Runs < 100 ms; `Erase::range` kostet O(Bytes der berührten Runs und Blobs) | F7 |
| H-S7 | Memtable mit `RwLock<BTreeMap>`, blockweisem Scan und 64-Ops-Unterbatches hält den Durchsatz des Bestands (Schreiben, 8 Leser) bei ±20 % | F5 |
| H-S8 | `estimate` über 1 000 Index-Einträge < 1 ms bei warmem Index-Cache; Fehler der oberen Schranke ≤ 2× ohne starke Überschattung | F5 |
| H-S9 | Präfix-Filter reduziert Block-Lesevorgänge bei Präfix-Scans ohne Treffer in ≥ 90 % der Runs um ≥ 80 % | F3 |
| H-S10 | Mit `separate_body(4 KiB)` sinkt die gemessene Schreibverstärkung bei 64-KiB-Werten um ≥ 3× gegenüber Inline-Werten | F6 |
| H-S11 | Ein Scan mit Filter auf Header (Körper 8 KiB, Header 32 B) liest ≥ 10× weniger Bytes als ein Scan mit Inline-Werten | F5 |
| H-S12 | `read_batch` über 32 Blöcke ist ≥ 2× schneller als 32 serielle `read_at` auf NVMe (Referenzhardware) | F3/F5 |
| H-S13 | Mit vorallokierten, beschriebenen Segmenten sinkt die `sync_data`-Latenz gegenüber wachsenden Dateien (Median, p99) merklich | F5 |
| H-S14 | `Erase::domain` (Schlüsselvernichtung) dauert < 10 ms unabhängig vom Datenvolumen | F8 |
| H-S15 | Ein kleiner `BodyCache` verbessert wiederholte Körper-Lesevorgänge unter Agenten-Last nur messbar, wenn die Trefferquote des Seitencaches < 70 % ist; sonst entfällt er | F5 |
| H-S16 | 1 000 ruhende (hibernierte) Instanzen: < 1 % CPU im Leerlauf, < 2 MiB RSS je Instanz | F9 |

**Abbruchkriterium H-S7:** Verfehlt F5 die Hypothese um mehr als 2×, wird die Memtable durch eine Skip-Liste in einer **isolierten, gesondert geprüften** Einheit ersetzt (SD-10); das ist die einzige Stelle, an der `unsafe` überhaupt diskutiert wird, und nur nach Beschluss. **H-S13** wird in F5 gemessen; verfehlt sie, bleibt die Vorallokation als Option, nicht als Pflicht. **A-09** legt die Referenzhardware in F1 fest; ohne sie ist keine Hypothese beurteilbar.

<!-- TEIL6 -->

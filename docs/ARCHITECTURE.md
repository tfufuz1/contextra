# Architektur

Abgeleitet aus [`SPEC.md`](SPEC.md) Kapitel 5 (Ringe, Matrix, Laufzeitmodell, Datenflüsse, Verzeichnislayout) und Kapitel 9 bis 11. Bei Widerspruch gilt die SPEC.

## 1. Überblick

Contextra ist eine eingebettete Bibliothek mit einem LSM-Speicher, einem segmentierten Textindex, einem HNSW-Vektorindex und einem eigenen Löschkern. Alle Schichten sind über Traits aus `contextra-ports` entkoppelt (P12). Jede Löschung ist ein Vorgang mit Journal, Gate, Schlüsselvernichtung, Purge und signiertem Bericht (SPEC 11.2).

## 2. Ringe

```
Ring 4  contextra                         Fassade, keine Logik
Ring 3  contextra-erasure   contextra-engine
Ring 2  contextra-mvcc  contextra-store  contextra-text  contextra-vector  contextra-rank
Ring 1  contextra-sys  contextra-simd  contextra-durable-fs  contextra-crypto
Ring 0  contextra-types  contextra-ports
dev     contextra-testkit  contextra-lab  ctx-verify  contextra-arch-tests
```

Abhängigkeiten zeigen nur auf gleiche oder niedrigere Ringe (AR-1). Innerhalb von Ring 2 gilt die Matrix unten.

## 3. Crates

| Crate | Ring | Zweck | Status (Ist) |
|---|---|---|---|
| `contextra-types` | 0 | Reine Datentypen: IDs, Tags, Schlüssel, Fehler, Löschtypen. Kein I/O. | vorhanden, enthält Altlasten |
| `contextra-ports` | 0 | Schnittstellen (Traits) zwischen den Schichten | vorhanden, Löschports fehlen |
| `contextra-sys` | 1 | Einzige `unsafe`-Stelle für Betriebssystemaufrufe | vorhanden, Build-Fehler durch `mmap` |
| `contextra-simd` | 1 | Vektor-Distanzfunktionen mit SIMD-Kernen und Skalar-Referenz | vorhanden, u8-VNNI zu entfernen |
| `contextra-durable-fs` | 1 | `RealVfs` mit dauerhaften Schreib- und Austauschoperationen | vorhanden, Startprobe nicht nachgewiesen |
| `contextra-crypto` | 1 | Kryptoprimitive: AEAD, Argon2id, HKDF, HMAC, Ed25519, Tags | vorhanden, Feature-Altlasten |
| `contextra-mvcc` | 2 | Sequenzen, Snapshots und SSI-Validierung | vorhanden, Sentinel `u64::MAX` zu entfernen |
| `contextra-store` | 2 | LSM-Speicher: WAL, Memtable, SSTables, Manifest, Kompaktion, Purge-Primitive | vorhanden, Build-Fehler |
| `contextra-text` | 2 | BM25-Volltextindex als verschlüsselte Segmente mit deutschem Analyzer | vorhanden, Umbau nötig |
| `contextra-vector` | 2 | HNSW-Vektorindex mit Arena, löschbaren Slots und verschlüsseltem Snapshot | vorhanden, Build-Fehler, großer Anteil außerhalb des Soll |
| `contextra-rank` | 2 | Reciprocal Rank Fusion für die Hybridsuche | vorhanden, deutlich größer als nötig |
| `contextra-erasure` | 3 | Löschkern: Saga, Gate, Journal, Tresor, Messung, signierter Bericht | **leer** |
| `contextra-engine` | 3 | Orchestrierung: Mandanten, Collections, Transaktionen, Suche, Hintergrundtasks | vorhanden, Build-Status nicht erreicht |
| `contextra` | 4 | Öffentliche API und Verdrahtung | vorhanden, widerspricht §7 |
| `contextra-testkit` | dev | Testhilfen: `FaultVfs`, Referenzmodell, deterministische Uhr | vorhanden |
| `contextra-lab` | dev | Unabhängige Prüfinstanz für die Löschmessung | **fehlt** |
| `ctx-verify` | dev/bin | Eigenständiger Verifizierer für Berichte und Feldscans | **fehlt** |
| `contextra-arch-tests` | dev | Architektur-, Ring-, Namens- und Lint-Tests | vorhanden |

Die Zweck-Sätze sind in SPEC 5.1a festgelegt und stehen wortgleich in `Cargo.toml`, im `//!`-Kopf von `lib.rs` und in der ersten Zeile jeder `README.md`.

## 4. Abhängigkeitsmatrix (SPEC 5.3)

Zeilen sind die Quellcrates, Spalten die erlaubten Ziele.

| von \ nach | types | ports | sys | simd | dfs | crypto | mvcc | store | text | vector | rank | erasure |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| ports | ✓ | | | | | | | | | | | |
| sys | ✓ | | | | | | | | | | | |
| simd | ✓ | | | | | | | | | | | |
| durable-fs | ✓ | ✓ | ✓ | | | | | | | | | |
| crypto | ✓ | ✓ | | | | | | | | | | |
| mvcc | ✓ | ✓ | | | | | | | | | | |
| store | ✓ | ✓ | ✓ | | | ✓ | ✓ | | | | | |
| text | ✓ | ✓ | | | | ✓ | | | | | | |
| vector | ✓ | ✓ | | ✓ | | ✓ | | | | | | |
| rank | ✓ | ✓ | | | | | | | | | | |
| erasure | ✓ | ✓ | | | | ✓ | | | | | | |
| contextra | ✓ | | | | ✓ | | | | | | | |
| engine | ✓ | ✓ | ✓ | | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

Zusätzliche Regeln (SPEC 5.3, D-88, D-95):

- `engine` darf `sys` nur für die Prozesshärtung nutzen.
- `vector` hängt weder an `sys` noch an `mvcc`. Die Arena-Sperren laufen über einen Port aus `ports`, implementiert in `engine`.
- `text` darf `crypto` nutzen.
- Die Fassade übergibt nur `RealVfs`, Uhr und OS-CSPRNG. `engine` erzeugt den Tresor.
- `store`, `text` und `vector` beziehen Dateischlüssel ausschließlich über den Port `FileKeyring`.

## 5. Laufzeitmodell

- **Runtime.** Tokio multi-thread. Die Bibliothek erzeugt keine eigene Runtime. `EngineBuilder::runtime(Handle)` ist Pflicht.
- **Cancel-Safety (I-14).** Jede mutierende Operation läuft in einem vom Aufrufer unabhängigen Task. Der Aufrufer wartet auf ein `oneshot`.
- **Hintergrundtasks.** WAL-Flusher, Memtable-Flush, Kompaktion, Purge-Scheduler, Vektor-Reparatur und Rebuild, Textsegment-Merge, Ledger-Rotation. Jede Task hat ein Stop-Flag und ein `JoinHandle`.
- **Sperrreihenfolge.** Dokumentlock, dann Commit-Barriere, dann Purge-Barriere, dann Journal. Keine Sperre über ein `.await` fremder Tasks.
- **Drop** blockiert nicht. Ein geordneter Shutdown läuft über `close().await`.

## 6. Datenflüsse

**Schreiben.** `Collection::insert` prüft den Mandantenkontext und das ReadGate, nimmt den Dokumentlock und leitet `doc_tag` und `cluster_tag` ab. Der Tresor legt fehlende Slots der Ahnenkette an und leitet den DEK ab. Danach folgt eine Store-Transaktion: Der Gruppen-Commit-Leader prüft das ReadGate erneut, schreibt die neuen Tresor-Slots mit `sync_data`, anschließend die WAL-Frames mit `sync_data` und zuletzt Memtable und Sichtbarkeit.

**Lesen.** `get` leitet die Tags ab, prüft das ReadGate, liest den Record, öffnet den Envelope mit dem DEK. Fehlt ein Slot der Ahnenkette, ist das Ergebnis `Erased`.

**Suchen.** Text und Vektor liefern `DocHandle`-Treffer. Die Bindung wird nur für Kandidaten entschlüsselt. Ein ReadGate-Filter lädt bis `k` gültige Treffer nach (D-62). Danach RRF und Laden der Dokumente.

**Löschen.** Validierung und Aufbewahrungsattestation, Journal `Requested`, Gate-Gruppen-Commit, Schlüsselvernichtung der höchsten Ebene und Vektor-Surrogat, Journal `KeyDestroyed`, Rückgabe `Gated`. Im Purge-Fenster folgen Drain, Flush, Entfernen der WAL-Segmente, SSTable-Rewrite, Dateischlüssel-Vernichtung, Textsegment-Rewrite, Vektor-Reparatur, Entfernen der `D`-Slots, Cache-Eviction, Messung, Bericht, Ledger und `Completed` (SPEC 11.2).

## 7. Löschkern im Überblick

- **Gate (`ReadGate`).** Varianten `Erased` und `Restricted`. Prüfung in jedem Lesepfad, im Suchstrom, in Zeitreisen, in der Commit-Phase und in der Kompaktion. Persistiert in `gates.log`.
- **Schlüsselhierarchie (D-79).** Slots `T` (Mandant), `C` (Collection), `S` (Betroffener), `D` (Dokument). Die Vernichtung eines Slots der höchsten Ebene macht alle darunter liegenden DEKs unableitbar. Die Zeit bis `Gated` ist unabhängig von der Zielgröße (P16).
- **Tresor.** Plattenresidente Hash-Tabelle mit 80-Byte-Slots (SPEC F.3). Keine vollständigen RAM-Indizes.
- **Purge.** Die WAL wird nie umgeschrieben (D-73). Entfernt werden ganze WAL-Segmente bis zum Flush-Punkt. SSTables werden ohne Zielrecords neu geschrieben.
- **Messung.** `LayerWitness` nur aus `Scanner<Armed>`, also nur nach erfolgreicher Positivkontrolle (D-72). Produktion misst MK1, MK2 und die Schlüsselabwesenheit (MK3-Feld). Die unabhängige Prüfung erfolgt im Lab und mit `ctx-verify scan`.

## 8. Datenmodell

**Löschcluster-Schlüsselraum (SPEC 9.1, D-31).** Schlüsselaufbau:

```
region(1) ‖ tenant_tag(16) ‖ coll_tag(16) ‖ cluster_tag(16) ‖ doc_tag(16) ‖ family(1) ‖ sub(u32) ‖ !commit_seq(u64)
```

Regionen: `DATA` (0x01) und `META` (0x03). Die Region `LOC` entfällt (D-70). Ein Subject-Ziel ist ein Bereich `DATA ‖ tenant ‖ coll ‖ cluster`.

**Familien (SPEC 9.2).** `Doc`, `DocMap`, `Manifest`, `TextBuf`.

**Tags (SPEC 9.3, Anhang C).** Alle Tags sind BLAKE3-keyed über `K_idx` mit festen Labels und längenpräfigierten Eingaben. Die Tag-Länge beträgt 16 Byte.

**Envelope v2 (SPEC 9.4, F.6).** Version, `alg_id`, `doc_tag`, Nonce (96 Bit), Ciphertext, AEAD-Tag. Der Schlüssel ist der DEK des Dokuments; die AAD umfasst den vollständigen Schlüssel einschließlich `!commit_seq`.

**Textindex (SPEC 9.7).** Segmente sind unveränderlich, verschlüsselt und enthalten nur `term_tag` statt Klartextterme. Postings tragen segmentlokale `doc_ord`. Die Bindung `doc_ord → (cluster_tag, doc_tag)` liegt als Envelope unter dem DEK. Nach Schlüsselvernichtung sind die Postings `unlinked`.

**Vektorindex (SPEC 9.8).** HNSW mit Arena. Je Slot liegen alle Repräsentationen zusammenhängend (D-86). Löschung: Surrogat vor `Gated`, Batch-Kantenreparatur im Purge-Fenster, deterministischer Rebuild nach Quote (D-77).

## 9. Verzeichnislayout (SPEC 5.6)

```
<data>/
  LOCK                     exklusiver Prozess-Lock (D-50)
  HEADER.a  HEADER.b       Format-Version, Store-ID, Keyslots; A/B mit Generationszähler
  report.pub               öffentlicher Schlüssel für Berichte (vom Betreiber extern hinterlegt)
  wal/      <file_id>.wal  HMAC-Kette, verschlüsselt
  sst/      <file_id>.sst  SSTables, Block-AEAD
  MANIFEST/ <file_id>.log  Manifest-Log
  text/     <tenant_tag>/<coll_tag>/<file_id>.seg
  vector/   <tenant_tag>/<coll_tag>/<file_id>.vsn
  vault/    vault.dat      Tresor (Slots T, C, S, D, F)
  erasure/  journal.log, gates.log, scope/, report/
  tmp/                     Waisen werden beim Start entfernt
```

Außerhalb des Datenverzeichnisses liegt das Ledger (`LedgerSink`, SPEC 11.8). Kein Datei- oder Verzeichnisname enthält Klartext (D-75, T-RAW-6).

## 10. Startsequenz (SPEC 14.1)

1. Verzeichnis-Lock. 2. Startprobe des Dateisystems. 3. HEADER lesen, KDF-Grenzen prüfen, Passphrase prüfen. 4. Prozesshärtung. 5. Orphans in `tmp/` entfernen. 6. Gate-Log laden. 7. Journal lesen, offene Löschungen aufnehmen. 8. Manifest und WAL-Recovery. 9. Tresor öffnen, Waisen bereinigen. 10. Ledger prüfen. 11. Lesepfade freigeben, Tasks starten. Fehler in den Schritten 1 bis 10 verhindern den Start.

## 11. Ist-Zustand

Der Workspace baut derzeit nicht vollständig (SPEC F-32). Der Löschkern fehlt (F-33). Die Kapazitätsgrenzen, Kryptografie-Parameter und Formate gelten als Soll; ihre Umsetzung wird mit den Arbeitspaketen aus SPEC 15.4 geprüft.

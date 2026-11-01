# Contextra v1 – Spezifikation (SPEC)

|Feld|Wert|
|---|---|
|Spezifikationsversion|**3.0.0** (SOLL-Zustand für Release 1.0.0; konsolidiert)|
|Stand|2026-11-01|
|Status|**Final.** Alle Architektur- und Produktentscheidungen sind getroffen und begründet (Abschnitt 4). Offen bleiben ausschließlich externe Freigaben (Gate G-LEGAL, Abschnitt 6.7), keine technischen Fragen.|
|Basis der Analyse|`github.com/tfufuz1/contextra`, `main` @ `e5e29108da11f86f54d51094a3f67a4cdabb7ae6` (2026-11-01 21:25 +01:00)|
|Prüfmodus|Quelltext gelesen, Manifeste ausgewertet, Muster gezählt **und** `cargo metadata`, `cargo fmt --check`, `cargo check --workspace --all-targets --keep-going` sowie Teile der Test-Suite mit `rustc 1.89.0` ausgeführt (Marke **[M]**). Rechtsstände mit Webrecherche am 2026-11-01 gegengeprüft (Marke **[W]**).|
|Ersetzt|`docs/SPEC.md` 2.0.0 und die darin eingebettete Fassung 1.0.0-spec.1 (beide waren in **einer** Datei hintereinander abgelegt und widersprachen sich in D-01, D-15, D-20, D-30)|
|Geltung|Einzige normative Beschreibung des Zielzustands. Jede Abweichung des Quellcodes oder eines Folgedokuments von diesem Dokument ist ein Defekt (Anhang E listet die Folgedokumente, die angepasst werden müssen).|
|Begleitdokument|`PLAN.md` 3.0.0 (Arbeitspakete, Gates, Reihenfolge, Aufwand). Beide Dokumente sind in sich vollständig.|

> **Keine Rechtsberatung.** Kein Softwareprodukt kann „100 % Rechtskonformität“ zusagen, denn Rechtskonformität ist ein Merkmal des Betriebs durch den Verantwortlichen, nicht des Codes. Dieses Dokument legt deshalb fest, was Contextra technisch maximal leistet, wie jede Leistung gemessen wird und was Contextra wahrheitsgemäß behaupten darf (Abschnitt 6.4 und 16). Wo eine Rechtsfrage die Technik berührt, ist sie in Abschnitt 6.6 **konservativ entschieden**: Es gilt immer die strengste Auslegung. Eine spätere juristische Prüfung kann dadurch Aussagen lockern, nie technische Maßnahmen nachfordern.

---

## 0. Konventionen

- **MUSS / DARF NICHT / SOLL** gelten im Sinne von RFC 2119.
- **IDs:** `P` Prinzip, `D` Entscheidung, `HR` Harte Regel, `R` Rechtsanforderung, `E-L` Rechtsentscheidung, `I` Invariante, `T` Test, `B` Benchmark, `FI` Funktionsinventar, `FM` Fehlermodus, `B-nn`/`H-nn` Befund.
- **Evidenzmarken:** **[M]** am HEAD `e5e29108d` gemessen (Befehl in PLAN.md WP-0.1). **[V]** am HEAD statisch geprüft (Datei, Zeile, Zählung). **[W]** am 2026-11-01 im Web gegen Primär- oder Fachquellen geprüft. **[D]** aus den Repo-Dokumenten übernommen, nicht eigenständig geprüft (insbesondere Rechtssynthese in `docs/DELETION-DSGVO.md`). **[E]** Entscheidung, Rechnung oder Zielwert dieser Spezifikation. Zielwerte sind bis zum Benchmarklauf (WP-6.1) Gate-Schwellen, keine Messungen.
- `grep`-Zählungen enthalten Inline-Testmodule und sind **Obergrenzen** für Produktionscode.
- Alle Byte-Layouts sind big-endian, sofern nicht anders angegeben. Alle Zeitangaben sind UTC aus dem Port `Clock`.
- Evidenzstufen für Aussagen im Repo (INV-EVIDENCE-1): E0 Behauptung, E1 Quelltextbeleg, E2 lokaler Gate-Lauf mit Protokoll, E3 reproduzierbar in CI auf unabhängigem Runner, E4 zusätzlich unabhängiges Orakel, Mutationsprüfung und Nachweis im Erasure Lab. Eine Produktaussage über Löschung ist erst ab **E4** zulässig.

---

## 1. Produkt

### 1.1 Mission

Contextra ist eine **eingebettete Rust-Bibliothek** (Single-Node, kein Server) für lokale Speicherung von Wissen und Gedächtnis von Agenten- und RAG-Systemen: Dokumente (Textchunks, Metadaten, Zuordnung zu einer betroffenen Person), Volltextsuche (BM25), Vektorsuche (HNSW), Hybridsuche (RRF) und Zeitreisen über MVCC-Versionen. Kurzformel: **„SQLite für Agenten, die gezielt und nachweisbar löschen können.“**

**Alleinstellungsmerkmal:** Löschen ist ein **eigener, entkoppelter, gemessener Prozess**. Ein Betreiber ruft `erase(Ziel)` auf und erhält einen signierten, von einem unabhängigen Programm prüfbaren Bericht, der je Schicht ausweist, was entfernt, was gemessen und was nicht erfasst wurde. Der Aufwand der Löschung skaliert mit der Größe des **Ziels**, nicht mit der Größe der Datenbank (P11). Alles andere ist Dienst am Löschen: Keine Funktion DARF Daten an einem Ort ablegen, den der Löschkern nicht erreicht.

### 1.2 Zielnutzer

Entwickler und Betreiber von Agenten- und RAG-Systemen im EU-Raum (Schwerpunkt Deutschland), die Betroffenenrechte (Art. 17 und 18 DSGVO) technisch umsetzen müssen, einen prüfbaren Nachweis (Art. 5 Abs. 2, Art. 24 DSGVO) brauchen und Enterprise-Anforderungen an Betrieb, Lieferkette und Support stellen.

### 1.3 Produktstufen (jede einzeln lieferbar)

|Stufe|Inhalt|Lieferbares Ergebnis|
|---|---|---|
|**S0**|Gehärteter LSM-Speicher mit Löschcluster-Schlüsselraum, Pflichtverschlüsselung samt Keyslots, Tresor, Gate, Saga, Purge, Journal, Bericht, Lab, Verifier. Dokumente, Zeitreisen, Mandanten.|Verschlüsselter Dokumentenspeicher mit belegbarer Löschung. Vollwertiges Produkt (Agentengedächtnis ohne Suche).|
|S1|Textsuche als abgeleiteter, segmentierter, verschlüsselter Index|Volltextsuche mit denselben Nachweisen|
|S2|Vektorsuche (HNSW) mit Slot-Zeroing, verschlüsseltem Snapshot, Batch-Rebuild|RAG-Kern|
|S3|Hybridsuche (RRF), API-Politur, Dokumentation, Release-Candidate|v1.0.0|

Reihenfolge-Begründung: S0 beweist das Alleinstellungsmerkmal auf kleinstem Umfang. Jede weitere Stufe fügt eine Löschschicht und neue Fehlerquellen hinzu. Scheitert eine spätere Stufe, bleibt S0 ein vollwertiges Produkt. **Release-Regel (D-59):** v1.0.0 erscheint erst, wenn S0 bis S3 das Gate G-H bestehen. Gibt es bis G-G keine tragfähige Vektor- oder Hybridschicht, gilt die dokumentierte **Cut-Linie** (PLAN 6): v1.0.0 = S0 + S1, S2/S3 folgen als v1.1. Es gibt keine halb fertige Löschschicht in einem Release.

### 1.4 Nicht-Ziele v1

Netzwerkdienst, Server-Socket, Sharding, Replikation, P2P; Erzeugung von Embeddings und LLM-Aufrufe (Vektoren und Text kommen vom Aufrufer); Dateiformat-Parsing; Graph und Relationen; Checkpoints, Forks, Hardlink-Klone; persistente Caches; automatische Entitäts- oder Metadatenextraktion; Lizenz- und DRM-Schicht; MCP-Server und Python-Bindings; Mehr-Betroffenen-Dokumente (ein Dokument gehört zu höchstens einem Betroffenen); Rotation des Master-Schlüssels (Passphrasenwechsel ist **kein** Nicht-Ziel, D-52); Telemetrie; Schlüssel-Escrow durch den Hersteller; Löschung außerhalb von Contextra (Backups, Kopien beim Betreiber, Hardware-Ebene); Windows als Produktionsplattform (D-56).

### 1.5 Prüfbare Zusagen (Leistungsbeschreibung, Anhang B)

1. Ein Löschauftrag wirkt ausschließlich auf sein typisiertes Ziel; alle Nicht-Ziel-Daten bleiben bitgleich erhalten (T-ISO-1).
2. Ab persistierter Sperre liefert jeder Lesepfad `Erased`; Zeitreisen eingeschlossen (T-VER-1). Nach der Sperre wird nichts mehr in das Ziel geschrieben (T-GATE-1).
3. Für jedes Ziel wird der dokumentspezifische Schlüssel aus dem Tresor entfernt und das Fehlen geprüft (T-VAULT-1..4).
4. Nach abgeschlossener Löschung enthält kein Datenverzeichnis-File, kein Index und kein von Contextra kontrollierter Speicher einen gemessenen Rest des Ziels (T-RAW-1..5, Messklassen E1/E2/E3).
5. Unterbrechungen (Absturz, I/O-Fehler, Abbruch des Aufrufers) führen nach Neustart zur Vollendung oder zu sichtbarem Status `Pending`. Stille Teilzustände existieren nicht (T-CRASH-1..4, T-CANCEL-1, T-FSYNC-1).
6. Der Bericht nennt, was gemessen wurde, und listet alle zutreffenden Restrisiken (T-REPORT-1, T-PROOF-1).

### 1.6 Zusagen, die Contextra nicht macht

„DSGVO-konform“, „rechtskonform“, „garantiert gelöscht“, „unwiederbringlich“, Löschung in Backups, auf Flash-Ebene, in Dateisystem-Journalen, in Swap oder bei externen Verarbeitern (Abschnitt 6.4, 16). **Die Formulierung „maximal rechtlich garantieren“ ist als Produktaussage verboten**: Der Code garantiert einen messbaren technischen Zustand, die rechtliche Wertung trifft der Betreiber.

---

## 2. Leitprinzipien (normativ)

|ID|Prinzip|Folge|
|---|---|---|
|P1|**Wirkung vor Behauptung**|Ein Status oder Zähler „gelöscht“ entsteht nur aus einem Zeugen (`LayerWitness`) eines Scanners.|
|P2|**Fail-closed**|Unbekannter Zustand, korrupte Daten, abgelaufene Fristen: Verweigerung oder `Pending`, nie Weiterlaufen.|
|P3|**Ein Weg**|Ein Löschpfad, ein Format, eine Fehlerbehandlung je Konzept. Keine Legacy-Lesepfade.|
|P4|**Ziel-Isolation**|Jede mutierende Operation hat ein typisiertes Ziel. Nachbardaten sind bitgleich unverändert (messbar).|
|P5|**Keine Schalter für Sicherheit**|Kein `cfg(feature)` auf Verschlüsselung, Integrität, Löschmessung, Mandantenprüfung.|
|P6|**Keine Panics, keine verschluckten Fehler**|Im Produktionscode kein `unwrap`, `expect`, `panic!`, `unreachable!`, `let _ =` auf `Result`, kein `unwrap_or_default` auf I/O.|
|P7|**Determinismus über Ports**|Zeit (`Clock`), Zufall (`Rng`), Dateisystem (`Vfs`), IDs (`IdGen`). Ausnahme: Schlüssel-, Salt- und Nonce-Material aus dem OS-CSPRNG.|
|P8|**Kleinster Kern**|Ein Crate oder Modul ist nur im Kern, wenn ein Kernkonsument es braucht (`cargo tree`-belegt).|
|P9|**Messen vor Optimieren**|Jedes Leistungsgate hat eine Basislinie auf dokumentierter Hardware. Ohne Basislinie kein Gate.|
|P10|**Verifizierbarkeit vor Eleganz**|Jede Entscheidung nennt den Test, der sie belegt. Ohne Test keine Entscheidung.|
|P11|**Löschbarkeit als Strukturprinzip**|Schlüsselraum, Dateiformat und Indexe sind so gebaut, dass jedes Löschziel einen zusammenhängenden, kleinen Bereich belegt. Löschaufwand skaliert mit dem Ziel.|
|P12|**Entkopplung**|Der Löschkern kennt nur Traits aus `ports`. Speicher, Textindex und Vektorindex implementieren `ErasureLayer`. Ein Austausch einer Schicht ändert den Löschkern nicht.|
|P13|**Datensparsamkeit im Nachweis**|Journal, Bericht und Ledger enthalten nur Commitments, Zähler und Zeiten, nie Inhalte, Schlüssel oder Klartext-Kennungen.|
|P14|**Dauerhaftigkeit ist ein Vertrag mit dem Betriebssystem**|Contextra verlässt sich nur auf geprüfte Dateisystem-Eigenschaften (Startprobe), behandelt jeden `fsync`-Fehler als **fatal für die betroffene Datei** und wiederholt nie blind (D-55).|
|P15|**Betreibbarkeit ist Teil des Produkts**|Gesundheitsstatus, feste Fehlertexte, Kapazitätsregeln, Runbooks und Support-Zeiträume sind testbare Lieferbestandteile (Abschnitt 14, 17).|

---

## 3. Befund und Konsolidierung (Review-Ergebnis)

### 3.1 Gesamturteil zur Produktionsreife

**Das Projekt ist am HEAD `e5e29108d` nicht produktionsreif und nicht einmal lauffähig [M].** Die Spezifikationen sind weitgehend tragfähig, der Code folgt ihnen nur teilweise. Die Mission (nachweisbare, isolierte Löschung) ist **im Code derzeit nicht vorhanden**: Das Crate `contextra-erasure` enthält nur `README.md` und `AGENTS.md`, keine einzige Quelldatei [M]. Die Entscheidung lautet **weiterbauen auf dem vorhandenen Fundament** (Store, MVCC, Krypto, SIMD, Testkit), **neu bauen** den Löschkern, **umbauen** Textindex, Vektorpersistenz, Schlüsselraum und Engine-Schreibpfad. Ein Neuanfang wäre teurer, weil 345 Testdateien und 102.000 Zeilen `src` [M] als Orakel und Regressionsschutz erhalten bleiben (PLAN 1).

### 3.2 Gemessener Ist-Zustand am HEAD [M]

|ID|Befund|Messung|
|---|---|---|
|B-01|**`contextra-engine` (lib) kompiliert nicht**: 33 Fehler (18× E0609 fehlendes Feld, 6× undefinierter `cfg`-Wert, 4× E0599, 3× E0433). Betroffen: `collection/search/hybrid`, `transaction/*`, `query_builder`, `background_workers`. Damit baut auch die Fassade `contextra` nicht.|`cargo check --workspace --all-targets --locked`|
|B-02|`--all-targets` scheitert in 24 Einheiten: `crypto` (9 Test-/Bench-Ziele), `engine` (lib test: 171 Fehler), `ports` (lib test), `rank` (3), `store` (4), `text` (2), `vector` (2).|`--keep-going`|
|B-03|`cargo fmt --check` schlägt fehl (67 Diff-Blöcke).|`cargo fmt --all -- --check`|
|B-04|`crates/contextra-erasure/`: 0 Zeilen Quelltext. `ports` und `types` enthalten die Löschtypen (`ErasureLayer`, `ReadGate`, `RecordSealer`, `LedgerSink`, `Matcher`, `KeyBuf`) **nicht**.|Verzeichnislisten|
|B-05|Rückbau halb fertig: Workspace listet nur noch 15 Mitglieder (gut), aber 86 Verweise auf `contextra_graph`/`contextra-graph` bleiben in Quelltext und Manifesten; `ports` deklariert weiter `graph`, `graph_index`, `checkpoint`, `license`, `attention`, `plugin`, `reranker`, `embedding`, `query_rewriter`; `AutoExtractionMode` ist in 10+ Dateien.|`grep`|
|B-06|**WAL-Integrität und Verschlüsselung per undeklariertem Feature:** 4 Treffer `wal-integrity`, 13 Treffer `encryption-at-rest` (kein Feature deklariert); `#![allow(unexpected_cfgs)]` in Tests und `kv_lock.rs`.|`grep`|
|B-07|**Snapshot-Sentinel unverändert:** `min_read_snapshot().unwrap_or(u64::MAX).min(…)` in `store/src/lsm/ops/compaction.rs:225–231`.|Quelltext|
|B-08|**Delete-Race unverändert:** `let _guard = …lock_for(id).await; drop(_guard);` in `engine/src/collection/crud/delete.rs`.|Quelltext|
|B-09|**Verschlüsselung optional:** `encryption_mode: Option<EncryptionMode>` Default `None` im Builder; `Option<Arc<KeyManager>>` im WAL.|Quelltext|
|B-10|**Mandantenpolitik:** `TenantPolicy` Default `Optional`.|`engine/src/lib.rs:192–198`|
|B-11|**KDF:** Untergrenze m = 19.456 KiB, t = 2; `try_new(passphrase, salt)` leitet per HKDF aus der Passphrase ab (`crypto/src/crypto.rs:88`).|Quelltext|
|B-12|**`DocId(pub u64)`** (Feature `docid-128` → `u128`), unkeyed.|`types/src/types/domain/ids.rs:173,180`|
|B-13|**Zwei `Vfs`-Traits** (`ports/src/vfs.rs:29`, `durable-fs/src/vfs.rs:54`); im Store 0 Verwendungen von `Vfs`, aber 157 direkte Dateizugriffe (`std::fs`, `tokio::fs`, `File::*`); im Vektor-Crate 53.|`grep`|
|B-14|`block_on` an 7 Stellen im Bibliothekscode.|`grep`|
|B-15|Produktionsmuster (Obergrenzen): `.unwrap()` 1.120 · `.expect(` 1.043 · `panic!` 72 · `let _ =` 237 · `#[allow` 134 · `HashMap/HashSet` 211 · `Instant::now/SystemTime::now` 24 · `thread_rng/from_entropy` 12.|`grep` über `crates/*/src`|
|B-16|**`tokio` mit Feature `full`** im Workspace; `Cargo.lock` enthält `mio` und `socket2`. Das widerspricht HR-1 (kein Netzwerk-Stack im Produktionsbaum), solange nicht `net` entfernt ist. Quelltext selbst nutzt `tokio::net`/`std::net` nicht (0 Treffer).|`Cargo.toml:55`, `Cargo.lock`|
|B-17|CI: 19 Workflows, **15× `continue-on-error`**. `deny.toml` ignoriert 6 Advisories ohne Ablaufdatum und enthält veraltete `wrappers`-Einträge auf entfernte Crates.|`.github/workflows`, `deny.toml`|
|B-18|`unsafe`: nur in `contextra-simd` (269 Treffer) und `contextra-sys` (21). 12 Crates tragen `#![forbid(unsafe_code)]`. Positiv, entspricht D-29. Offen: zwei `safe`-Funktionen mit unsafe-Inhalt (`mmap_readonly`, `reopen_and_dup2`) laut Audit.|`grep`, Audit|
|B-19|Teilweise positiv: Workspace-Lints (`unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, …) sind als `deny` gesetzt; `mvcc` 47/47, `simd` 13/13, `sys` 30/30 Lib-Tests bestanden.|`cargo test --lib`|
|B-20|Dokumente widersprechen sich: `VETOES.md` (VETO-D01), `CONSTITUTION.md`, `ARCHITECTURE.md`, `SECURITY.md` und `SOURCE_OF_TRUTH.md` behaupten „keine Ebene B in v1“ und zugleich „SPEC 2.0.0“; `SPEC.md` 2.0.0 verlangt in D-01 den Tresor. `docs/DELETION-DSGVO.md` empfiehlt die Zwei-Ebenen-Löschung mit Schlüsselvernichtung. `PLAN.md` bezieht sich auf die ältere Fassung 1.0.0-spec.1.|Dokumentvergleich|

### 3.3 Auflösung der Dokumentkonflikte (verbindlich)

|#|Konflikt|Entscheidung|Begründung|
|---|---|---|---|
|K-1|Schlüsselvernichtung in v1? (VETO-D01/ADR-001 „nein“ vs. SPEC 2.0.0 D-01 „ja, Säule 2“)|**Ja.** Drei Säulen (Gate + Purge, Schlüsselvernichtung, Messung). **ADR-002 ersetzt ADR-001 in D-01; VETO-D01 wird aufgehoben und durch VETO-D01b ersetzt** (Anhang E).|Das Datenformat (Record-Envelope, Tresor-Salt) lässt sich nach Auslieferung nicht rückwirkend einführen. Physisches Überschreiben ist auf Flash/COW unzuverlässig; die Schlüsselvernichtung ist die einzige Maßnahme, die den Aufwand auf ein kleines Objekt je Dokument reduziert und technisch anerkannt ist (NIST SP 800-88 Rev. 2, Cryptographic Erase [D]). Die rechtliche Wertung bleibt ausdrücklich beim Betreiber (E-L2).|
|K-2|Restriktion (Art. 18) in v1?|**Ja** als Gate-Variante `Restricted` (D-15).|Das Gate existiert ohnehin; Art. 17 Abs. 3 und Art. 18 sind reale Rechtslagen; Nachrüsten kostet mehr als Mitliefern.|
|K-3|Textindex: Postings als Einzelschlüssel `(term_tag, doc)` (1.0.0-spec.1 D-20) oder Segmentindex?|**Segmentindex** (D-20 in 3.0.0).|Einzelschlüssel streuen jedes Dokument über den Schlüsselraum und zerstören P11.|
|K-4|Erase-API: Rückgabe nach `Gated` (alle Fassungen)|bestätigt, p50-Ziel plattformrelativ (D-30).|–|
|K-5|Schlüsselableitung: `K_master = Argon2id(passphrase)` (2.0.0)|**Geändert:** zufälliger `K_master`, durch passphrase-abgeleiteten KEK gewickelt (Keyslots, D-52).|Ohne Keyslots ist ein Passphrasenwechsel nur durch Neuverschlüsselung der gesamten Datenbank möglich – für Enterprise-Betrieb (ISO 27001 A.8.24, Personalwechsel) inakzeptabel.|
|K-6|`Collection::delete`|**Umbenannt** in `soft_delete` (D-51).|Das Verb „delete“ suggeriert Löschung; die Operation ist rein logisch. Ein Rechtsprodukt darf an dieser Stelle nicht irreführen (H1).|
|K-7|Dokumentzahl der Crates „14 + dev“|Workspace-Mitglieder: 14 Produktions-Crates + `testkit` + `tests/arch` + `lab` + `ctx-verify`. Am HEAD fehlen `lab`, `ctx-verify`, `contextra-erasure`-Quelltext.|Abschnitt 5.1.|
|K-8|Rechtsfristen in 2.0.0 („ProdHaftG ab 9.12.2026“, CRA „SBOM, Meldeweg“)|**Präzisiert [W]:** CRA-Meldepflichten (Art. 14) gelten **seit 11.09.2026**, die übrigen CRA-Pflichten ab 11.12.2027; das neue ProdHaftG (Software als Produkt) gilt für Produkte, die ab **9.12.2026** in Verkehr gebracht werden, der Bundestag hat es am 08.10.2026 beschlossen; EDPB Guidelines 02/2026 (Anonymisierung) liegen als **Entwurf** vor (Konsultation bis 30.10.2026).|Abschnitt 6.2, 6.6.|

### 3.4 Nicht geprüft (bewusst nicht behauptet)

`clippy` mit `-D warnings` über den Workspace (Engine kompiliert nicht; Audit vom 2026-11-01 meldete bis zu 303 Fehler je Crate [D]); vollständige Test-Suite (nur Lib-Tests der Leaf-Crates); `miri`, `loom`, Fuzz, Mutation; Leistungsbasislinie; Inhalt der Teile von `contextra-engine`, die nicht kompilieren; HNSW-Snapshot-Format im Detail; ob der Blockcache entschlüsselte Blöcke hält; Kompaktions-Retention-Semantik. Jede Lücke hat ein Arbeitspaket mit Messung (PLAN WP-0.1).

---

## 4. Entscheidungsregister (final)

Jede Entscheidung nennt Begründung und belegenden Test. **Fett** gesetzte IDs wurden gegenüber SPEC 2.0.0 geändert oder sind neu.

### 4.1 Löschen, Speicher, Schlüssel

|ID|Entscheidung|Begründung|Test / Beleg|
|---|---|---|---|
|**D-01**|**Löschen in drei Säulen, alle in v1:** (1) Gate und physischer Purge, (2) Schlüsselvernichtung je Dokument über den Tresor, (3) unabhängige Messung mit signiertem Bericht. ADR-002 ersetzt ADR-001 (D-01). Die Säulen sind **Verteidigung in der Tiefe, keine rechtliche Wertung** (E-L2).|Überschreiben garantiert auf Flash/COW nichts; Schlüsselvernichtung reduziert die Remanenzfläche auf ein 80-Byte-Objekt je Dokument. Nachrüsten bricht das Datenformat. Der Tresor liegt selbst im Dateisystem (`WrappedKeyRemanence`); das ist ein dokumentiertes Restrisiko, kein Argument gegen die Säule, und wird mit austauschbarem Anker (D-53) adressierbar.|T-VAULT-_, T-RAW-_, T-PROOF-1|
|D-02|**Eigener LSM-Tree bleibt, nur mit Löschcluster-Schlüsselraum (D-31).** Gate G0 (11.9) entscheidet verbindlich mit Zahlen. Rückfallebene: redb-Adapter hinter `StorageEngine`.|Der LSM-Vorteil ist die Dateigranularität. Mit verstreuten Hash-Schlüsseln läge ein Dokument in ca. 26 % aller Dateien (Rechnung 11.9), ein Purge schriebe fast die ganze Datenbank um. Ein Vollersatz durch SQLite/redb machte den Löschaufwand proportional zur Datenbankgröße und zerstört die Mission.|B-ERASE-WA, T-DIFF-1|
|D-03|Ports sind `async` (`BoxFuture`). Kein `block_on` im Bibliothekscode.|Synchrone Ports über asynchroner I/O erzwingen `block_on` und Deadlocks (B-14).|T-LINT-2|
|D-04|**Ein** `Vfs`-Trait in `ports`, Implementierung in `durable-fs`. Store, Vektor, Text und Erasure nutzen ausschließlich `Vfs`.|B-13. Fehler- und Crashinjektion setzt `Vfs` voraus.|T-LINT-2, T-CRASH-*|
|D-05|Verschlüsselung und WAL-Integrität sind nicht abschaltbar. Kein `Option<KeyManager>`, kein Klartextmodus, kein Feature `wal-integrity`/`encryption-at-rest`.|B-06, B-09. Ein Opt-out erzeugt Parallelbestände.|T-DEF-1, T-CFG-1|
|D-06|Passphrase nur über Argon2id, Minimum m = 64 MiB, t = 3, p = 1 (hart). Unterschreiten ist Fehler. HKDF nur für Subkeys.|B-11. Untergrenze folgt RFC 9106 (zweite empfohlene Option, 64 MiB, t = 3) [D].|T-KDF-1|
|**D-68**|KDF-**Standard** m = 128 MiB, t = 3, **p = 4**; die Parameter stehen im Keyslot und sind je Slot änderbar.|Parallelität erhöht die Kosten für Angreifer bei gleicher Latenz des Betreibers; Defaults sind Konfiguration, kein Format.|T-KDF-1|
|D-07|`TenantContext` ist Pflicht. Keine mandantenlose Zugriffsfunktion; `TenantPolicy` entfällt.|B-10.|T-API-1 (trybuild)|
|D-08|Dokumentidentität: `doc_tag = BLAKE3-keyed(K_idx, "ctx/doc/v1" ‖ tenant ‖ coll ‖ doc_key)[0..16]`. Kollision mit anderem `doc_key` → `Conflict::IdCollision`.|B-12. 128 Bit keyed, nicht aus Nutzerkennung berechenbar.|T-ISO-3|
|D-09|Alle Schlüsselkomponenten sind **blinde Tags** (keyed BLAKE3, `K_idx`). Nutzerkennungen stehen nur verschlüsselt im Wert.|Der Indexbereich einer SSTable darf keine Nutzerdaten enthalten.|T-RAW-4|
|D-10|Matcher sind **Schlüsselbereiche fester Tag-Länge** plus **exakte Schlüssel**. Keine String-Präfixe, keine variablen Präfixe.|Variable Präfixe waren Ursache des Datenvernichtungsfehlers im Altbestand (HR-7).|T-ISO-2, T-KEY-1|
|D-11|Zeitreisen bleiben. Purge entfernt alle Versionen. Offene Snapshots **desselben Mandanten** (D-61), die ein Ziel berühren, werden widerrufen (`SnapshotRevoked`). `history_horizon` (Standard 24 h) begrenzt die Versionsmenge.|Warten auf Snapshots macht Löschfristen unbegrenzt.|T-VER-1, T-SNAP-1|
|D-12|**ReadGate** vor Purge, auch für Zeitreisen.|Sofortwirkung für Nutzer.|T-VER-1|
|D-13|Bericht: Ed25519-signiert, kanonisches JSON, ein Format. Die Signatur beweist Integrität und Herkunft aus der Installation, nicht die Wahrheit des Inhalts.|Symmetrische Beweise sind durch den Prüfer fälschbar.|T-PROOF-1|
|D-14|Betroffenen-Löschung über `subject_tag` = Cluster (D-31).|Ohne Zuordnung findet Contextra eine Person nicht.|T-SUBJ-1|
|D-15|`RetentionAttestation` ist Pflicht. **`Restricted` (Art. 18) ist Gate-Variante in v1**; eine Fristenautomatik ist kein v1-Umfang.|K-2.|T-API-2, T-RESTR-1|
|D-16|Keine Checkpoints, Forks, Hardlink-Klone.|Jede Kopie liegt außerhalb der Löschgrenze.|T-ARCH-3|
|D-17|Graph und Relationen entfernt, nicht per Feature abgeschaltet.|P5; Kantenreparatur pro Löschung ist fehleranfällig (VETO-F02).|T-ARCH-3|
|D-18|Vektorlöschung: Slot-Zeroing sofort, Kanten bleiben bis **Batch-Rebuild** bei Tombstone-Quote ≥ 0,2 oder spätestens zum Ende des Purge-Fensters nach `erase_sla`.|Kanten enthalten nur Slot-Nummern. Rewiring pro Löschung ist teuer und riskant.|T-VEC-*, T-RAW-3|
|D-19|Vektor-Snapshots: Payload je Slot unter dem DEK des Dokuments, Graphkanten unter dem Store-Schlüssel, ausschließlich über `Vfs`.|B-13. Embeddings gelten vorsorglich als personenbezogen (E-L3).|T-RAW-3|
|**D-20**|**Textindex: abgeleiteter, segmentierter, verschlüsselter Index** (9.7). Der LSM hält keine Postings.|K-3.|T-TEXT-*, B-ERASE-WA|
|D-21|`tenant_tag`, `coll_tag`, `cluster_tag`, `doc_tag`, `term_tag`, `name_tag` sind keyed BLAKE3 mit festen Labels (Anhang C).|Index enthält keine Klartexte.|T-RAW-4|
|D-22|Entfernte Crates liegen im Branch `archive/v0-full`.|Historie auffindbar.|PLAN WP-1.2|
|D-23|Lizenz-/DRM-Schicht entfällt.|Geschäftsmodell liegt außerhalb des Speicherkerns.|T-ARCH-3|
|D-24|Ingest, Privacy, Agent, Cognition, Router, Adapt, KV-Cache, Inference, Sandbox, Audit-Export, AVV-Generator, `contextra-db` entfallen.|Nicht im Kern.|T-ARCH-3|
|D-25|MCP-Server und Python-Bindings sind kein v1-Umfang. Sie entstehen nach API-Freeze als Adapter.|Löschwerkzeuge dürfen erst nach der Saga exponiert werden.|–|
|D-26|`rank` enthält nur RRF-Fusion.|Hybridsuche ist Kern, der Rest nicht.|T-RANK-1|
|D-27|Keine automatische Entitäts- oder Metadatenextraktion. `AutoExtractionMode` entfällt vollständig.|B-05. Erzeugt Parallelbestände und PII-Abfluss.|T-ARCH-3|
|D-28|SIMD: u8-VNNI-Kernel entfällt; f32-Kernel plus Skalar-Referenz. Jeder Kernel hat Cross-Check.|Falsche Operandentypen bei `_mm512_dpbusd_epi32` (u8×u8 statt u8×i8).|T-SIMD-1|
|D-29|`unsafe` nur in `contextra-sys` und `contextra-simd`. Safe-Funktionen mit offenen Vorbedingungen werden `unsafe fn` oder entfernt.|Kleinste Auditfläche (B-18).|T-UNSAFE-1|
|**D-30**|`erase` kehrt nach persistiertem Gate und vernichtetem Schlüssel zurück (Status `Gated`). Ziel: **p50 ≤ 10 ms auf der NVMe-Referenzplattform** (14.8) und allgemein **≤ 6 × `fsync_p50` + 2 ms** der Startprobe; p99 ≤ 5 × p50. `erase_sla` ist Pflichtparameter ohne Standardwert.|Die Latenz wird durch die Zahl der `fsync` bestimmt (4 im Normalpfad, D-58), nicht durch CPU. Eine absolute Grenze wäre auf Netzlaufwerken oder SATA-SSD nicht erfüllbar und deshalb kein ehrliches Gate.|T-SLA-1, B-ERASE-LAT|
|D-31|**Löschcluster-Schlüsselraum:** `region ‖ tenant_tag ‖ coll_tag ‖ cluster_tag ‖ doc_tag ‖ family ‖ sub` (9.1). Jedes Löschziel ist ein zusammenhängender Bereich.|P11, Rechnung 11.9.|B-ERASE-WA|
|D-32|**Record-Envelope:** Jeder Wert liegt unter dem DEK seines Dokuments; zusätzlich Block-AEAD.|Entkoppelt Schlüsselvernichtung von der Blockstruktur.|T-ENC-_, T-VAULT-_|
|D-33|**Tresor:** Slot je Dokument mit 32-Byte-Salt; `DEK = HKDF-SHA256(K_unit, salt, "ctx/dek/v1" ‖ doc_tag)`. Löschen = Slot nullen, `fsync`, Fehlen prüfen.|Kleinster löschbarer Anker.|T-VAULT-1..4|
|D-34|`contextra-erasure` hängt nur an `types`, `ports`, `crypto`. Schichten implementieren `ErasureLayer`.|P12.|T-ARCH-3|
|D-35|**Messung E1/E2/E3** (12.1) mit unabhängigem Lab. Ein Rohscan auf Klartext allein ist keine Messung.|In einem verschlüsselten Store ist er für Dateischichten fast immer leer.|T-RAW-2|
|D-36|**Klassenverband G, K, P** statt linearer Skala (12.5).|Die Aussagen sind unabhängig.|T-CLASS-1|
|D-37|**Restore-Guard** über Ledger (11.8).|Restore aus Backup kann gelöschte Daten zurückbringen.|T-RESTORE-1|
|D-38|`EraseTarget::Tenant` (Offboarding).|Häufigster realer Löschfall beim Betreiber.|T-TENANT-1|
|D-39|**Purge-Fenster:** Gate und Schlüsselvernichtung sofort, physischer Purge gebündelt bis höchstens `batch_window` (Standard 60 s) bzw. spätestens `erase_sla`; `Timing::Now` erzwingt Sofortlauf.|Bündelung senkt die Zahl berührter Dateien (11.9).|B-ERASE-BATCH|
|D-40|Ein Dokument gehört zu höchstens einem Betroffenen; der Betroffene ist **unveränderlich** (`Conflict::SubjectImmutable`).|Sonst ist der Cluster nicht eindeutig.|T-SUBJ-2|
|D-41|`scan` iteriert in `doc_tag`-Reihenfolge (Locator-Region), nicht in Nutzerschlüssel-Reihenfolge.|Folge von D-09 und D-31.|T-SCAN-1|
|D-42|Jeder Header (WAL, SSTable, Manifest, Tresor, Journal, Ledger, Gate-Log, Snapshot, Textsegment, Bericht, HEADER) trägt `format_version` und Algorithmus-IDs.|Krypto-Obsoleszenz planbar (EDPB 02/2025 Rn. 85, 96 [D]).|T-FMT-1|
|D-43|Kein Netzwerk, keine Telemetrie, kein Escrow im Produktionsbuild.|HR-1..3.|T-ARCH-4|
|D-44|Rot-vor-Grün; Evidenzbündel je Arbeitspaket.|Verifizierbar.|PLAN §1|
|D-45|`contextra-db` entfällt; `homeostat`, `multistep`, `volatile_vault` sind nicht im Kern.|Features leiten an entfernte Crates.|T-ARCH-3|
|D-46|Tests dürfen den Master-Schlüssel direkt injizieren (`EngineBuilder::master_key_for_testing`) unter Feature `testing`, das **nur** den Argon2-Schritt ersetzt, nie die Verschlüsselung. CI prüft, dass kein Release-Build `testing` aktiviert.|Schnelle Tests ohne Sicherheitsschalter.|T-CFG-2|
|D-47|**Locator-Region:** Je Dokument ein Locator-Eintrag `LOC ‖ tenant ‖ coll ‖ doc_tag → sealed(cluster_tag)`.|`get(doc_key)` kennt den Cluster nicht.|T-KEY-2|
|D-48|Commitment-Epoche und Ledger-Lebensdauer: 365 Tage; danach Rotation und Entfernen der Ledger-Einträge.|Pseudonymrest begrenzen (E-L4).|T-JRN-3|
|D-49|Textsegmente: Zielgröße 32 MiB, ein Dokument liegt zu jedem Zeitpunkt in genau einem Segment oder im Textpuffer.|Rewrite je Erase = ein Segment.|B-ERASE-WA|
|D-50|Ein Datenverzeichnis hat genau einen schreibenden Prozess (exklusiver Verzeichnis-Lock über `Vfs`).|Journal- und Manifest-Konsistenz.|T-LOCK-1|

### 4.2 Neu in 3.0.0 (Produktionsreife, Enterprise, Lücken der Vorfassungen)

|ID|Entscheidung|Begründung|Test / Beleg|
|---|---|---|---|
|**D-51**|`Collection::soft_delete` (logisch, Tombstone). Es gibt **kein** `delete`. Rustdoc und Dokumentation sagen an erster Stelle: „Für Löschpflichten `erase` verwenden.“ Nach `history_horizon` entfernt die Kompaktion Versionen physisch; das ist keine Löschaussage.|K-6.|T-API-3 (Doc-Test), T-LINT-1|
|**D-52**|**Keyslots:** `K_master` ist ein zufälliger 32-Byte-Schlüssel (OS-CSPRNG). Er liegt im `HEADER` je Slot gewickelt (AES-256-GCM-SIV) unter `KEK = Argon2id(passphrase, slot_salt, params)`. 1..4 Slots. `change_passphrase` fügt einen Slot hinzu, prüft ihn, entfernt den alten (Nullen, `fsync`) in O(1). `HEADER` wird als A/B-Paar mit Generationszähler geschrieben. Master-Rotation (Neuverschlüsselung) bleibt v1.1.|K-5. Betrieb braucht Passphrasenwechsel ohne Datenbankumschreibung; A/B verhindert Totalverlust bei Headerkorruption. Restrisiko `OldKeyslotRemanence` (alter Slot auf Flash) wird dokumentiert und im Bericht geführt.|T-KEYSLOT-1..3, T-HDR-1|
|**D-53**|Der Schlüsselanker ist **austauschbar**: `RecordSealer` (Port) hat in v1 die Implementierung `FileVault`; jeder Slot trägt eine `anchor_id`. Hardware-Anker (TPM 2.0, PKCS#11/HSM, KMS-Adapter außerhalb des Kerns) sind v1.1 ohne Formatbruch.|Löst `WrappedKeyRemanence` bei Bedarf, ohne v1 an Hardware zu binden.|Conformance-Suite `RecordSealer`|
|**D-54**|`tokio`-Features im Workspace: `rt-multi-thread`, `sync`, `time`, `macros`. **Kein** `full`, `net`, `fs`, `process`, `signal`, `io-std`. `deny.toml` verbietet `mio`, `socket2`, `reqwest`, `hyper`, `ureq`, `curl`, `isahc`, `surf`, `attohttpc`, `tonic`, `h2` im Produktionsbaum (`cargo tree -e normal`). Dev-Abhängigkeiten (`criterion`) sind ausgenommen.|B-16. HR-1 ist sonst nicht nachweisbar.|T-ARCH-4|
|**D-55**|**`fsync`-Fehler sind für die betroffene Datei fatal.** Nach einem Fehler von `sync_data`/`sync_all`/`sync_dir` wird die Datei nie auf demselben Handle wiederholt; der Store wechselt in `Poisoned`, alle mutierenden Operationen liefern `Io`, `health()` meldet es, Recovery beim nächsten Öffnen. Löschpfad: `Pending` mit Neuöffnen der Datei und Prüfung.|Nach einem fehlgeschlagenen `fsync` ist der Zustand der Seiten im Page-Cache undefiniert (bekanntes PostgreSQL-„fsyncgate“-Verhalten [D]). Blindes Wiederholen meldet fälschlich Dauerhaftigkeit – für ein Löschprodukt ein Haftungsfall.|T-FSYNC-1|
|**D-56**|**Plattformstufen:** Tier 1 = Linux (x86_64, aarch64) auf ext4 oder XFS. Tier 2 = macOS (APFS, `F_FULLFSYNC`) und Linux auf btrfs/ZFS (COW → Restrisiko `FsJournalOrCow` immer). **Nicht unterstützt:** Windows, NFS, SMB, FUSE, overlayfs und unbekannte Dateisysteme: die Startprobe (`statfs`-Typ plus atomares Rename plus Verzeichnis-`fsync`) verweigert den Start (`Config::UnsupportedFilesystem`).|Verzeichnis-`fsync` und atomares Rename sind die Basis der Crash-Konsistenz. Netz- und Overlay-Dateisysteme geben diese Zusagen nicht.|T-PLAT-1, FI-F03|
|**D-57**|**Gate-Commit-Protokoll:** `ErasedSet::add` nimmt die **exklusive Commit-Barriere** des Stores, persistiert das Gate und gibt frei. Jeder Commit prüft in der Gruppen-Commit-Leader-Phase `ReadGate` für alle Schlüssel seines Batches und weist Schreibzugriffe in gegatete Bereiche mit `Erased` ab. Dokumentlocks gelten nur für `EraseTarget::Document`; für Subject/Collection/Tenant gibt es **kein** Lock-Set.|2.0.0 verlangte Dokumentlocks vor dem Gate. Bei einem Betroffenen mit 10⁶ Dokumenten sind Locks nicht haltbar. Die Barriere ist O(1) und beweisbar: alles vor dem Gate ist im Purge-Umfang (`gated_seq`), alles danach wird abgewiesen.|T-GATE-1, T-CONC-1 (loom)|
|**D-58**|Gate-Menge als **append-only Log** (`erasure/gates.log`, AEAD, Hash-Kette, `fsync` je Eintrag), Kompaktion beim `Completed` einer Löschung.|Atomarer Dateiaustausch je Gate kostet 3+ `fsync` (Datei, Rename, Verzeichnis); ein Append kostet einen. Normalpfad `Gated`: Journal `Requested+Intent` (1), Gate-Log (1), Tresor-Nullen (1), Journal `KeyDestroyed` (1) = **4 `fsync`**.|B-ERASE-LAT|
|**D-59**|Release-Regel und Cut-Linie (1.3).|Kein Release mit halber Löschschicht.|G-H|
|**D-60**|**Schlüssel je Datei:** `K_file = HKDF-SHA256(K_enc, "ctx/file/v1" ‖ kind ‖ file_id)` für WAL, SSTable, Manifest, Textsegment, Snapshot-Chunks. Nonces (96 Bit, OS-CSPRNG) sind je Datei zu begrenzen (≤ 2³² Blöcke, sonst neue Datei).|Begrenzt das Nonce-Budget je Schlüssel unabhängig von der Datenbankgröße (AES-GCM-SIV ist nonce-misuse-resistent, die Grenze bleibt Defense in Depth).|T-NONCE-1, T-ENC-3|
|**D-61**|**Snapshot-Widerruf ist mandantenbezogen:** `revoke_covering` widerruft nur Snapshots des Mandanten des Ziels (Registry ist nach `tenant_tag` geführt), die `seq ≤ gated_seq` halten. Snapshots anderer Mandanten bleiben unberührt.|Isolation gilt auch für Verfügbarkeit (VETO-F10); ein Offboarding darf fremde Leser nicht stören.|T-SNAP-1|
|**D-62**|Suche: `ReadGate`-Filter im Kandidatenstrom mit Nachladen bis `k` Treffer oder Erschöpfung (kein stilles Verkürzen der Ergebnisliste); Metadatenfilter bleiben **nachgelagert** (bekannte v1-Grenze, dokumentiert).|Sonst verdrängen gegatete Treffer gültige aus dem Top-k.|T-HYB-1, T-SEARCH-2|
|**D-63**|**Prozesshärtung** (`contextra-sys`): `RLIMIT_CORE = 0`, `PR_SET_DUMPABLE = 0`, `MADV_DONTDUMP` für Schlüsselseiten und Vektorarena, `mlock` best effort. Builder-Option `process_hardening` (Standard an). Scheitert `mlock`, bleibt der Betrieb möglich, der Bericht führt `ProcessMemory`.|Coredumps und Swap sind die realistischen Wege für Klartext-Remanenz im Prozess.|T-DUMP-1, T-SEC-2|
|**D-64**|**Beobachtbarkeit:** `Metrics`-Port (statische Namen, keine ID-Labels) und `Contextra::health() -> HealthReport` (Zustand `Ok / Degraded / Poisoned / Recovering`, `fsync_p50`, offene Löschungen, Alter der ältesten, SLA-Verletzungen, Tombstone-Quote, freier Platz). Kein Exporter, kein Server im Kern.|Betreibbarkeit (P15).|T-OPS-1|
|**D-65**|**Plattenreserve:** Schreibzugriffe werden bei freiem Platz unter `max(1 GiB, 2 × größte Kandidatendatei)` mit `Limit::DiskReserve` abgewiesen; `erase` bis `Gated` braucht keinen nennenswerten Platz und läuft immer. Der Purge-Rewrite reserviert vorab.|Eine volle Platte darf weder Löschung noch Gate verhindern.|T-CRASH-3, T-OPS-2|
|**D-66**|**API-Stabilität:** SemVer, MSRV = Toolchain-Pin (1.89.0; Anhebung nur in Minor-Releases mit Changelog), alle öffentlichen Typen `#[non_exhaustive]` (außer Konfigurationsbausteinen), einzige öffentliche Oberfläche ist `contextra`. Format-Version und Crate-Version sind getrennt.|Enterprise-Verlässlichkeit.|T-API-1, `cargo-semver-checks`|
|**D-67**|**Support und Schwachstellenmanagement:** Unterstützungszeitraum je Hauptversion ≥ 5 Jahre ab Veröffentlichung; Meldeprozess nach CRA Art. 14 (Frühwarnung 24 h, Meldung 72 h, Abschlussbericht gemäß Verordnung); intern HR-8 (kritisch 7 Tage, hoch 30 Tage). Contextra behandelt sich als Hersteller im Sinne des CRA und des ProdHaftG (E-L9).|Strengste Annahme (6.6). [W] CRA-Meldepflichten gelten seit 11.09.2026.|T-SUPPLY-1, `SECURITY.md`|

---

## 5. Gesamtarchitektur

### 5.1 Workspace (Soll)

14 Produktions-Crates, dazu vier Hilfs-Mitglieder. Die Spalte „Ist“ zeigt den gemessenen Zustand am HEAD [M].

|Crate|Ring|Rolle|Ist am HEAD|
|---|---|---|---|
|`contextra-types`|0|Reine Typen (IDs, Tags, Schlüssel, Fehler, Löschtypen). Kein I/O, kein `async`.|vorhanden; enthält Altlasten (`DocId`, `saos`, `tombstone`, `AutoExtractionMode`); `erasure.rs` ist nicht das Soll|
|`contextra-ports`|0|Traits: `StorageEngine`, `TextIndex`, `VectorIndex`, `Vfs`, `Clock`, `Rng`, `IdGen`, `ErasureLayer`, `ReadGate`, `RecordSealer`, `LedgerSink`, `Metrics`.|vorhanden; Altports (`graph`, `checkpoint`, `license`, …) und zweites `Vfs` sind zu entfernen; Löschports fehlen|
|`contextra-sys`|1|`unsafe`-Insel: `fsync`, `flush_dir`, `rename_durable`, `mlock`, Prozesshärtung, Dateirechte.|vorhanden, 30/30 Lib-Tests grün|
|`contextra-simd`|1|`unsafe`-Insel: f32-Distanzkernel mit Skalar-Referenz.|vorhanden, 13/13 Lib-Tests grün; u8-VNNI-Pfad noch zu entfernen|
|`contextra-durable-fs`|1|`Vfs`-Implementierung (`RealVfs`), atomarer Austausch, `scrub` best effort, Startprobe.|vorhanden (1.593 Zeilen), eigenes `Vfs`-Trait|
|`contextra-crypto`|1|AEAD, KDF, HKDF, Keyslots, WAL-HMAC, Hash-Ketten, Ed25519, Tags.|vorhanden; 9 Test-/Bench-Ziele kompilieren nicht; HKDF-Passphrase, Untergrenze zu niedrig|
|`contextra-mvcc`|2|Snapshots, SSI, Sequenzen.|vorhanden, 47/47 Lib-Tests grün; Sentinel-Fehler (B-07)|
|`contextra-store`|2|LSM-Tree (WAL, Memtable, SSTable, Manifest, Kompaktion, Purge-Primitive).|vorhanden (26.996 Zeilen, 124 Testdateien); B-06, B-07, B-09, B-13|
|`contextra-text`|2|BM25-Segmentindex.|vorhanden als termgeordneter LSM-Index; Umbau nötig (D-20)|
|`contextra-vector`|2|HNSW, Arena, Snapshot.|vorhanden (13.624 Zeilen); `std::fs`/`memmap2`, unverschlüsselt|
|`contextra-rank`|2|RRF-Fusion.|vorhanden (3.440 Zeilen, Ziel deutlich kleiner)|
|`contextra-erasure`|3|Löschkern: Saga, Gate, Journal, Tresor, Messung, Bericht, Ledger.|**leer** (nur README/AGENTS)|
|`contextra-engine`|3|Orchestrierung: Collections, Suche, Transaktionen, Verdrahtung.|vorhanden (20.357 Zeilen), **kompiliert nicht**|
|`contextra`|4|Fassade, einzige öffentliche Oberfläche.|vorhanden, hängt an Engine|
|`contextra-testkit`|dev|Fixtures, `FaultVfs`, `ReferenceModel`, `ManualClock`, In-Memory-Ports.|vorhanden|
|`contextra-lab`|dev|Unabhängige Prüfinstanz (12.3).|**fehlt**|
|`ctx-verify`|dev/bin|Eigenständiger Berichtsprüfer (8.17).|**fehlt**|
|`tests/arch`|dev|Architektur- und Lint-Tests.|vorhanden|

`members` = alle obigen Einträge. `default-members` = die 14 Produktions-Crates (ohne `testkit`, `lab`, `ctx-verify`). Pfadabhängigkeiten und `[workspace.dependencies]` auf entfernte Crates entfallen (am HEAD bereits weitgehend umgesetzt, Reste laut B-05).

### 5.2 Ringe und Abhängigkeitsregeln

```
Ring 4  contextra                         (Fassade)
Ring 3  contextra-erasure   contextra-engine
Ring 2  contextra-mvcc  contextra-store  contextra-text  contextra-vector  contextra-rank
Ring 1  contextra-sys  contextra-simd  contextra-durable-fs  contextra-crypto
Ring 0  contextra-types  contextra-ports
dev     testkit  lab  ctx-verify  tests/arch
```

|Regel|Inhalt|Test|
|---|---|---|
|AR-1|Abhängigkeiten zeigen nur auf gleichen oder niedrigeren Ring; innerhalb Ring 2 nur laut Matrix 5.3.|T-ARCH-3|
|AR-2|`store` kennt weder `text`, `vector` noch `erasure`.|T-ARCH-3|
|AR-3|`erasure` kennt **keine** Ring-2-Crates, nur `ports`-Traits, `types` und `crypto`.|T-ARCH-3|
|AR-4|`lab` und `ctx-verify` hängen nicht an `store`, `engine`, `erasure` und nicht an den Produktions-Decodern von `crypto`. Sie haben eigene Implementierungen der Formate und Primitive (unabhängiger Prüfer).|T-ARCH-3|
|AR-5|`types` ohne `async` und ohne I/O.|T-ARCH-1|
|AR-6|Nur `TenantScopedStorage` ruft den rohen Store.|T-ARCH-2|
|AR-7|Kein Netzwerk-Crate im Abhängigkeitsbaum der Produktionscrates (D-54).|T-ARCH-4|
|AR-8|`unsafe` nur in `sys` und `simd`; alle anderen `#![forbid(unsafe_code)]`.|T-UNSAFE-1|
|AR-9|Jedes `feature = "…"` im Quelltext ist in einem `[features]`-Abschnitt deklariert; kein `#[allow(unexpected_cfgs)]`.|T-CFG-1|

### 5.3 Abhängigkeitsmatrix (erlaubte Kanten)

|von \ nach|types|ports|sys|simd|dfs|crypto|mvcc|store|text|vector|rank|erasure|
|---|---|---|---|---|---|---|---|---|---|---|---|---|
|ports|✓||||||||||||
|sys|✓||||||||||||
|simd|✓||||||||||||
|durable-fs|✓|✓|✓||||||||||
|crypto|✓|✓|||||||||||
|mvcc|✓|✓|||||||||||
|store|✓|✓|✓|||✓|✓||||||
|text|✓|✓||||✓|||||||
|vector|✓|✓||✓||✓|||||||
|rank|✓|✓|||||||||||
|erasure|✓|✓||||✓|||||||
|engine|✓|✓|||✓|✓|✓|✓|✓|✓|✓|✓|

`store` darf `Vfs` nur als Trait aus `ports` nutzen; die konkrete `RealVfs` wird in `engine`/`contextra` verdrahtet. `contextra-sys` ist für `durable-fs`, `store` und `vector` (Prozesshärtung, `mlock`) zulässig; `vector` benötigt `sys` nur über den Port-Wrapper in `engine`.

### 5.4 Laufzeitmodell

- **Runtime:** Tokio multi-thread. Die Bibliothek erzeugt keine eigene Runtime; `EngineBuilder::runtime(Handle)` ist Pflicht, alle Hintergrundtasks laufen darauf. Es werden nur die Tokio-Features nach D-54 verwendet.
- **Cancel-Safety (I-14):** Jede mutierende Operation läuft in einer vom Aufrufer unabhängigen Task (`spawn`); der Aufrufer wartet auf ein `oneshot`. Abbruch des Aufruferfutures ändert den Ausgang nicht.
- **Hintergrundtasks:** WAL-Flusher, Memtable-Flush, Kompaktion, Purge-Scheduler, Vektor-Rebuild, Textsegment-Merge, Ledger-Rotation. Alle besitzen Stop-Flag und `JoinHandle`; `close()` beendet sie geordnet.
- **Sperren und Barrieren:** (a) Pro Dokument ein asynchroner Lock für Insert, Update, `soft_delete` und `erase(Document)`. (b) **Commit-Barriere** (`RwLock`): jede Gruppen-Commit-Phase hält sie shared, `ErasedSet::add` hält sie exklusiv (D-57). (c) **Purge-Barrier** (`RwLock`) zwischen Kompaktion (shared) und Purge (exklusiv). Lockreihenfolge: Dokumentlock → Commit-Barriere → Purge-Barrier → Journal. Keine Sperre über `.await` einer fremden Task (`await_holding_lock = deny`).
- **Keine Blockierung in `Drop`** (14.3).
- **Poisoned-Zustand (D-55):** atomares Flag im Store; setzt jede mutierende Operation auf `Io`.

### 5.5 Datenflüsse

**Schreiben (Insert/Update):** `Collection::insert` → `TenantContext` prüfen → `ReadGate` prüfen (Ziel nicht `Restricted`/`Gated`) → Dokumentlock → `doc_tag`, `cluster_tag` ableiten → Tresor: Slot anlegen (neu) oder DEK holen → Envelope sealen (`Doc`, `DocMap`, `Manifest`, `Locator`) → Text- und Vektorpuffer vormerken → **eine** Store-Transaktion: Gruppen-Commit-Leader prüft `ReadGate` erneut (D-57), schreibt **vorher** alle neuen Tresor-Slots der Gruppe und führt **einen** `sync_data` des Tresors aus (I-9), danach WAL-Frames, HMAC, `sync_data` WAL (zwei `fsync` je Gruppe, nicht je Dokument) → Memtable → Sichtbarkeit.

**Lesen:** `get(doc_key)` → `doc_tag` → `ReadGate` → Locator lesen → Cluster → Record lesen → DEK (Tresor/LRU) → Envelope öffnen. Fehlt der Tresor-Slot: `Erased`.

**Suchen:** Text/Vektor liefern `doc_tag`-Treffer im Strom → `ReadGate`-Filter mit Nachladen (D-62) → RRF → Dokumente laden.

**Löschen:** `erase` → Validierung, Retention → Journal `Requested` (mit Absicht der Schlüsselvernichtung) → Gate persistieren (Commit-Barriere exklusiv, Gate-Log) → Tresor-Slots nullen → Journal `KeyDestroyed` → Rückgabe `Gated` → Purge-Fenster → Drain → Rewrite → Unlink → Indexbereinigung → Messung → Bericht → `Completed` (11.2).

### 5.6 Verzeichnislayout

```
<data>/                       0700
  LOCK                        exklusiver Prozess-Lock (D-50)
  HEADER.a  HEADER.b          Format-Version, Keyslots (D-52), Store-ID, Algorithmus-IDs; A/B mit Generationszähler
  wal/        *.wal           HMAC-Kette, verschlüsselt
  sst/        *.sst           SSTables (Block-AEAD), Level-Struktur
  MANIFEST/   *.log           Manifest-Log (Datei-Mengen, Schlüsselbereiche, Seq)
  text/       <coll>/*.seg    Textsegmente (verschlüsselt)
  vector/     <coll>/         Vektorsnapshots (verschlüsselt)
  vault/      vault.dat       Tresor (Slots)
  erasure/    journal.log     Löschjournal (Hash-Kette, verschlüsselt)
              gates.log       aktive Gates (Append-Log, D-58)
              ledger.log      abgeschlossene Löschungen (Restore-Guard)
              scope/          Auflösung je laufender Löschung (verschlüsselt)
              report/         signierte Berichte
  keys/       0700, Dateien 0600 (Signaturschlüssel-Seed verschlüsselt)
  tmp/        Waisen werden beim Start entfernt
```

Außerhalb des Datenverzeichnisses: das Ledger-Exemplar des Betreibers (`LedgerSink`, 11.8). Die Öffentliche Signaturschlüssel-Datei `erasure/report.pub` (Klartext, 32 Byte Ed25519) wird beim Anlegen des Stores ausgegeben, damit `ctx-verify` ohne Zugriff auf Geheimnisse prüfen kann.

### 5.7 Konfiguration (`EngineBuilder`)

|Parameter|Pflicht|Standard|Prüfung|
|---|---|---|---|
|`data_dir`|ja|–|existiert oder wird mit 0700 angelegt; Dateisystem laut D-56|
|`passphrase`|ja|–|Länge ≥ 12 Bytes|
|`erase_sla`|ja|–|1 min ≤ sla ≤ 30 d (Empfehlung für Dokumentation: Abschluss ≤ 24 h, Alarm bei `Pending` > 1 h)|
|`runtime` (Handle)|ja|–|–|
|`vfs`, `clock`, `rng`|ja (Fassade: `system_defaults()`)|–|Traits|
|`kdf`|nein|m = 128 MiB, t = 3, p = 4|≥ Minimum (D-06)|
|`erase_return_timeout`|nein|60 s|1 s … 10 min|
|`batch_window`|nein|60 s|0 s … `erase_sla`|
|`history_horizon`|nein|24 h|0 … 365 d|
|`max_doc_bytes`|nein|4 MiB|≤ 64 MiB|
|`vector_rebuild_ratio`|nein|0,2|0,05 … 0,5|
|`commit_epoch`|nein|365 d|30 d … 3 Jahre|
|`ledger_sink`|nein|Datei außerhalb des Datenverzeichnisses (`<data>/../.contextra-ledger/<store_id>`)|Pfad ≠ Unterpfad von `data_dir`|
|`process_hardening`|nein|an|D-63|
|`min_free_bytes`|nein|`max(1 GiB, 2 × größte Kandidatendatei)`|≥ 256 MiB|

Fehlende Pflichtparameter oder Werte außerhalb der Grenzen sind `ConfigError` beim `open`. Es gibt keine stillen Korrekturen.

### 5.8 Budgets (CI-blockierend)

|Budget|Wert|Messung|
|---|---|---|
|Erase-Produktcode (`contextra-erasure`, `store/src/erase`, `types/src/erasure`, `crypto/src/vault_cipher.rs`)|≤ 7.000 Zeilen (6.000 in 2.0.0; +1.000 für Gate-Log, Keyslot-Wiederherstellung und Poisoned-Pfade)|`tokei`|
|`unwrap`/`expect`/`panic!`/`let _ =`/`#[allow]` im Produktionscode|nur sinkend (Ratchet); Ziel 0 in allen Kern-Crates ab Gate G-C|`governance/ratchet.toml`|
|Clippy|0 Meldungen mit `-D warnings`|CI|
|Produktcode Kern gesamt|Reduktion ≥ 30 % gegenüber der Basislinie von **101.943 Zeilen** [M] (ohne Testcode)|`tokei`|
|`unsafe`|nur `sys`, `simd`|`forbid(unsafe_code)` sonst|

---

## 6. Rechts- und Haftungsrahmen als technische Anforderung

### 6.1 Rollen

Verantwortlicher im Sinne der DSGVO ist der **Betreiber**, der Contextra in seine Anwendung einbaut. Ihn treffen die Pflichten aus Art. 5 Abs. 2, 17, 18, 24, 25, 32. Contextra haftet nicht für diese Pflichten, sondern ausschließlich dafür, (a) **was es zusagt**, (b) **ob es als Produkt fehlerhaft ist**, und (c) falls es selbst personenbezogene Daten erhielte. Punkt (c) wird **konstruktiv ausgeschlossen**: Contextra sieht nie Daten des Betreibers, weil es lokal läuft und nichts nach außen sendet (HR-1..4). Dadurch ist Contextra bei bestimmungsgemäßem Betrieb weder Verantwortlicher noch Auftragsverarbeiter (E-L6; im Schrifttum überwiegend vertreten, einzelfallabhängig [D]).

### 6.2 Haftungsköpfe und Designwirkung

|ID|Haftungskopf|Auslöser|Designmaßnahme|Test|
|---|---|---|---|---|
|H1|Irreführende Werbung (§ 5, § 5a UWG)|Aussagen wie „DSGVO-konform“, „garantiert gelöscht“ in README, Doku, Berichten|Wortlaut-Lint blockierend; Außenaussagen nur aus 6.4 und Berichtsklassen|T-LINT-1|
|H2|Beschaffenheitsvereinbarung (§ 434, §§ 280 ff. BGB)|Werbung und Dokumentation werden Vertragsinhalt|Leistungsbeschreibung (Anhang B) schmal; jede Zeile hat Test und Evidenzlauf|T-LEIST-1..12|
|H3|**Produkthaftung** (ProdHaftG-Neufassung nach RL (EU) 2024/2853; Software ist Produkt) **[W]**: vom Bundestag am 08.10.2026 beschlossen, anwendbar auf Produkte, die ab **09.12.2026** in Verkehr gebracht werden; Beweiserleichterungen für Geschädigte|Falsches Löschziel, Datenverlust nach Absturz, unbemerkte Korruption|T-ISO, Crash-Matrix, WAL-HMAC, Block-AEAD sind haftungskritisch; Release-Blocker HR-7; Update- und Sicherheitsupdate-Prozess. **Folge für die Release-Planung:** Ein 1.0.0, das ab 09.12.2026 bereitgestellt wird, unterliegt dem neuen Recht; Aufzeichnungen der Gates (Evidenz) sind Beweismittel.|T-ISO-_, T-CRASH-_, T-WAL-1|
|H4|**Cyber Resilience Act** (VO (EU) 2024/2847) **[W]**: Meldepflichten für aktiv ausgenutzte Schwachstellen und schwere Vorfälle (Art. 14) gelten **seit 11.09.2026** über die ENISA Single Reporting Platform; übrige Pflichten (u. a. CE-Kennzeichnung, SBOM, Support-Zeitraum) ab **11.12.2027**|Bereitstellung eines Produkts mit digitalen Elementen|Meldeprozess (D-67), SBOM (`cargo cyclonedx`) je Release, `SECURITY.md` mit Meldeweg und Fristen, benannter Support-Zeitraum, signierte Releases|T-SUPPLY-1|
|H5|DSGVO direkt|Contextra-Seite erhält Daten (Telemetrie, Crash-Upload, Lizenzserver, Escrow)|HR-1..HR-4|T-ARCH-4, T-LOG-1|
|H6|Berufsgeheimnis (§ 203 StGB)|Einsatz bei Kanzlei, Arzt, Steuerberater|Kein Zugriff durch Contextra; Support nur mit Berichten ohne Inhalt|HR-4|
|H7|Verlässlichkeit der Nachweise|Betreiber verlässt sich auf einen falschen Bericht|Fail-closed, Positivkontrolle, Mutationstests auf `classify`, unabhängiger Verifier|T-RAW-2, T-PROOF-1, T-MUT-1|
|H8|Krypto-Obsoleszenz (Art. 32)|Algorithmus gebrochen oder Parameter veraltet|Algorithmus-ID und Format-Version in jedem Header; jährliche Neubewertung (Release-Checkliste); Keyslots erlauben KDF-Wechsel|T-FMT-1, T-KEYSLOT-2|

### 6.3 Harte Regeln

|ID|Regel|Durchsetzung|
|---|---|---|
|HR-1|Kein Netzwerkzugriff im Produktionsbuild. Keine HTTP-, Socket- oder DNS-Bibliothek im Abhängigkeitsbaum der Produktionscrates.|`deny.toml [bans]` (D-54), `cargo tree -e normal`-Test, Quellscan nach `std::net`, `tokio::net`|
|HR-2|Keine Telemetrie, kein Crash-Upload, kein Phone-Home.|wie HR-1|
|HR-3|**Kein Schlüssel-Escrow, keine Wiederherstellung durch den Hersteller.** Passphrase verloren heißt Daten verloren; dies steht an erster Stelle der Dokumentation. Betreiberseitige zusätzliche Keyslots (z. B. Notfallpassphrase im Tresor des Betreibers) sind Betreiberentscheidung, nicht Hersteller-Escrow.|Doku-Test, Review|
|HR-4|Fehler- und Lognachrichten enthalten nie Inhalte, Nutzerkennungen, Schlüssel oder Pfade mit Personenbezug.|T-LOG-1|
|HR-5|Außenaussagen ausschließlich aus 6.4.|T-LINT-1|
|HR-6|Jede Zeile der Leistungsbeschreibung hat mindestens einen automatisierten Test und einen Evidenzlauf.|CI-Job `leistung`|
|HR-7|Ein Test, der zeigt, dass ein Erase Daten außerhalb des Ziels verändert, ist **Release-Blocker ohne Ausnahme**.|G-D, G-H|
|HR-8|Sicherheitslücken werden innerhalb dokumentierter Fristen bearbeitet (kritisch 7 Tage, hoch 30 Tage); aktiv ausgenutzte Schwachstellen werden nach CRA Art. 14 gemeldet; unterstützte Versionen sind benannt.|`SECURITY.md`, Release-Checkliste|
|HR-9|Haftungsausschlüsse der Lizenz (MIT/Apache-2.0) werden **nicht** als Schutz verbucht. Für Vorsatz, grobe Fahrlässigkeit und Produkthaftung gelten zwingende Grenzen.|Prozess|
|HR-10|Löschzusagen im Bericht enthalten nie einen Zeugen ohne Positivkontrolle.|T-RAW-2|
|HR-11|Kein Release ohne bestandenes Gate G-H und signierte Artefakte samt SBOM.|PLAN Phase 7|
|HR-12|Kein neuer `unsafe`-Block, keine neue Abhängigkeit, kein neues Feature ohne Nennung des Tests, der es belegt (PR-Vorlage).|PR-Template, `cargo-deny`|

### 6.4 Zulässige Außenaussage (einzige)

> **DE:** Contextra entfernt die Daten eines Löschziels aus dem Datenverzeichnis, aus den Indexen und aus den gespeicherten Zeitreise-Versionen und weist den Zustand je Schicht messtechnisch nach. Für jedes Ziel wird zusätzlich der dokumentspezifische Schlüssel aus dem Schlüsseltresor entfernt; ob und wie das rechtlich zu werten ist, entscheidet der Betreiber. Nicht erfasst sind externe Sicherungen, Flash-Remanenz, Dateisystem-Journale, Prozessspeicher und externe Verarbeiter. Der Bericht listet diese Restrisiken.
> 
> **EN:** Contextra removes the data of an erasure target from the data directory, from its indexes and from stored time-travel versions, and provides per-layer, measured evidence of the resulting state. For each target the document-specific key is additionally removed from the key vault; whether and how this is to be assessed legally is for the operator to decide. Not covered are external backups, flash remanence, file-system journals, process memory and external processors. The report lists these residual risks.

**Verbotene Begriffe** (Wortlaut-Lint über Quelle, Doku, Berichtsvorlagen, Marketing, deutsch und englisch): „garantiert“/„guaranteed“, „100 %“ (im Zusammenhang mit Löschung), „DSGVO-konform“/„GDPR-compliant“, „rechtskonform“, „unwiederbringlich“/„irrecoverable“, „unwiderruflich“, „informationstheoretisch“, „kryptografisch gelöscht“/„cryptographically erased“ (ohne Verweis auf Bericht und Voraussetzungen), „sicher gelöscht“ ohne Berichtsbezug, „vollständig gelöscht“ ohne Klassenangabe, „zertifiziert“/„certified“, „von einer Aufsichtsbehörde geprüft“, „anonymisiert“ als Beschreibung des Löschergebnisses.

### 6.5 Rechtsanforderungsmatrix

|#|Anforderung (Orientierung)|Mechanismus|Test|Restrisiko im Bericht|
|---|---|---|---|---|
|R1|Unverzüglich, Monatsfrist (Art. 17 Abs. 1, Art. 12 Abs. 3)|Saga mit Wiederholung, `erase_sla`, Status, Alarm|T-CRASH-1, T-SLA-1|`ReplicaWindow`|
|R2|Unkenntlichmachen, nicht reproduzierbar (SDM-Baustein 60, ErwGr 26)|Klassen K und P|T-RAW-1..5|`FlashRemanence`, `WrappedKeyRemanence`|
|R3|Einfache OS-Löschbefehle genügen nicht (SDM-Baustein 60)|Rewrite, Unlink, Fehlen geprüft, Tresor-Slot genullt|T-CRASH-1, T-RAW-1|`FsJournalOrCow`|
|R4|Alle Versionen und Kopien (SDM M60.P14)|Purge aller Versionen, Snapshot-Widerruf, Index-Segmente|T-VER-1|`ExternalCopies`|
|R5|Rechenschaft (Art. 5 Abs. 2, Art. 24)|signierter Bericht, Audit-Kette|T-PROOF-1, T-REPORT-1|`PseudonymCommitment`|
|R6|Datenschutz durch Technikgestaltung und Voreinstellung (Art. 25)|Verschlüsselung und Mandantenpflicht ohne Opt-out|T-DEF-1, T-API-1|–|
|R7|Sicherheit der Verarbeitung (Art. 32)|Argon2id, WAL-HMAC, AEAD mit AAD, Keyslots|T-KDF-1, T-WAL-1, T-ENC-2|Schlüsselkompromittierung|
|R8|Information von Empfängern (Art. 17 Abs. 2, Art. 19)|Bericht führt `ExternalProcessors`; Checkliste (16); Hook `on_erased` in v1.1|–|Verantwortung Betreiber|
|R9|Aufbewahrungspflichten (Art. 17 Abs. 3), Einschränkung (Art. 18)|`RetentionAttestation`, `Restricted`|T-API-2, T-RESTR-1|Prüfung beim Betreiber|
|R10|Embeddings vorsorglich personenbezogen|Vektoren im Umfang, Snapshot verschlüsselt, Slot-Zeroing|T-RAW-3, T-RAW-5|`ProcessMemory`, `GraphRebuildPending`|
|R11|Nach Restore erneut löschen (SDM M60.S10, M60.P08)|Restore-Guard (Ledger)|T-RESTORE-1|Backup-Inhalte außerhalb|
|R12|Löschbarkeit auf Datensatz- und Gruppenebene mit beherrschbarem Aufwand (SDM M60.D01)|Löschcluster-Schlüsselraum, Segmentindex|B-ERASE-WA|–|
|R13|Datensparsames Löschprotokoll (SDM M60.S05/S06)|Journal-Reduktion, Commitments mit Epochenrotation|T-REPORT-1, T-JRN-3|`PseudonymCommitment`|
|R14|Regeln zu Schlüsseln zu löschender Daten (SDM M60.P09/P10)|Tresor-Protokoll (11.5)|T-VAULT-1..4|`WrappedKeyRemanence`|
|R15|Algorithmenwechsel planbar|Algorithmus-ID in Headern, Keyslots|T-FMT-1, T-KEYSLOT-2|–|
|R16|Werbeaussagen belegbar (§ 5 UWG)|Wortlaut-Lint|T-LINT-1|–|
|R17|Datenschutz-Folgenabschätzung des Betreibers (Art. 35)|Dokumentationspaket `docs/DSFA-BAUSTEIN.md` (technische Maßnahmen, Restrisiken, Datenflüsse); keine Rechtsbewertung|Review|–|
|R18|Meldung von Schwachstellen (CRA Art. 14)|`SECURITY.md`, PSIRT-Prozess (D-67)|T-SUPPLY-1|–|

### 6.6 Rechtsentscheidungen (technisch konservativ, endgültig)

Jede Frage, die die Technik berührt, ist hier entschieden. Maßstab ist immer die **strengste** Auslegung.

|ID|Frage|Entscheidung|Folge|
|---|---|---|---|
|E-L1|Genügt physische Umschreibung samt Entfernen der Altdateien?|Sie ist Pflichtmaßnahme (Klasse P), wird aber **nie allein** als ausreichend dargestellt. Der Bericht führt `FlashRemanence` und `FsJournalOrCow` immer, wenn zutreffend.|Säule 1 plus 2|
|E-L2|Ist die Vernichtung des Dokumentschlüssels (Klasse K) Löschung, Anonymisierung oder Einschränkung?|Contextra **behauptet keine** dieser Wertungen. Der Mechanismus läuft immer. Der Bericht weist `key_destroyed` als Tatsache aus und nennt `WrappedKeyRemanence`. Stand der Rechtslage [D, [W] für EDPB]: keine Behörden- oder Gerichtsaussage erkennt Schlüsselvernichtung als Löschung an; EDPB-Leitlinien 02/2025 (Blockchain, Endfassung 07/2026) nennen Bedingungen (Algorithmus, Schlüsselkopien); EDPB Guidelines 02/2026 (Anonymisierung) sind Entwurf und verlangen dokumentierte, fortlaufend geprüfte Wirksamkeit. Die Wertung trifft der Betreiber.|Bericht, 6.4|
|E-L3|Sind Embeddings, HNSW-Slotnummern und blinde Tags personenbezogen?|**Ja, vorsorglich.** Embeddings werden zeroisiert, Snapshots verschlüsselt, Tags im Purge entfernt. Der Graph-Rest (Slot-Nummern ohne Inhalt) wird bis zum Rebuild als `GraphRebuildPending` ausgewiesen und ist spätestens nach `erase_sla` beseitigt.|D-18, D-19|
|E-L4|Sind Bericht, reduziertes Journal und Ledger personenbezogen?|**Pseudonym ja, daher begrenzt:** nur Commitments, Bereichs-Tags und Zeiten, nie Kennungen; Lebensdauer = Commitment-Epoche (365 d), danach Entfernen.|D-48|
|E-L5|Sind aggregierte Textstatistiken personenbezogen?|Sie enthalten keine Dokumentbezüge. Sie werden dennoch **nach jedem Segment-Rewrite exakt neu berechnet**; kein Zähler überlebt die Löschung seines Beitrags.|9.7|
|E-L6|Rolle von Contextra?|Weder Verantwortlicher noch Auftragsverarbeiter, solange HR-1..4 gelten. Dokumentation weist darauf hin; Betreiber prüft selbst.|HR|
|E-L7|Aufbewahrungspflichten?|Contextra führt kein Fristenwerk. Der Betreiber erklärt per `RetentionAttestation`. Bei Pflicht: `Refused(RetentionObligation)`, oder `Restricted` als Gate-Variante ohne Vernichtung.|D-15|
|E-L8|Löschcheckliste und Berichtstext?|Wortlaut ist in Abschnitt 16 und Anhang D **endgültig festgelegt**; sie sind kein Rechtsrat und enthalten diesen Hinweis.|Lint|
|E-L9|ProdHaftG/CRA bei Open-Source-Bibliothek?|Contextra behandelt sich als **dem CRA und dem ProdHaftG unterliegend** (strengste Annahme, auch wenn eine Ausnahme für nicht-kommerzielle freie Software bestehen kann): SBOM, Meldeweg, Support-Zeitraum, Sicherheitsupdates.|H3, H4, D-67|
|E-L10|Wirkung von Haftungsausschlüssen?|Werden nicht als Schutz verbucht (HR-9). Die Leistungsbeschreibung ist schmal und testbar gehalten.|Anhang B|
|E-L11|Reichweite der Signatur?|Die Signatur beweist Integrität und Herkunft aus der Installation, nicht die Wahrheit des Inhalts. Dieser Satz steht im Bericht. Wer den Passphrasen-Schlüssel hält, kann Berichte erzeugen; eine unabhängige Zeitstempelung (RFC 3161) ist ein optionaler Betreiber-Hook ab v1.1.|D-13|
|E-L12|Liste der `RetentionCode`s?|Festgelegt in 7.2 (Enum ohne Freitext).|T-API-2|
|E-L13|Beanspruchen wir „Anonymisierung“?|**Nein.** Weder API, Bericht noch Dokumentation verwenden den Begriff für das Löschergebnis (Wortlaut-Lint).|T-LINT-1|
|E-L14|Wie wird der Zeitbezug „unverzüglich“ technisch bestimmt?|Contextra misst Zeiten (`requested_at`, `gated_at`, …) aus dem `Clock`-Port und weist `erase_sla`-Verletzungen aus. Die Frist selbst (Monatsfrist, ggf. Verlängerung nach Art. 12 Abs. 3) bestimmt der Betreiber.|T-SLA-1|

### 6.7 Gate G-LEGAL (externe Freigabe, nicht technisch)

Ohne schriftliche Antworten eines Fachanwalts für Datenschutzrecht bzw. der Datenschutzbeauftragten darf das Produkt keine Löschaussage außer der Zulässigen (6.4) treffen. Prüffragen: (1) Qualifikation der Schlüsselvernichtung (Löschung, Anonymisierung, Unzugänglichmachung, Einschränkung) und Haltung der zuständigen Aufsichtsbehörde; (2) relativer Personenbezug nach EuGH C-413/23 P; (3) Wirksamkeitsnachweis, Umfang und Remanenzrisiko des dateibasierten Tresors; (4) Embeddings und Slot-Nummern; (5) Backups und Restore-Guard-Horizont; (6) Zulässigkeit, Inhalt und Dauer von Commitments in Bericht und Ledger; (7) Rolle des Herstellers bei lokal installierter Software; (8) Aufbewahrungspflichten je Branche; (9) Freigabe der Löschcheckliste (16) und der Berichtstexte; (10) Prüfung jeder Werbeaussage auf UWG/BGB. **Der Zeitplan hängt nicht an diesem Gate**: Es blockiert nur die Veröffentlichung von Löschaussagen über 6.4 hinaus, nicht die Entwicklung (PLAN Phase 7).

---

## 7. Öffentliche API (Fassade `contextra`)

Einzige öffentliche Oberfläche (D-66). Alle Typen `#[non_exhaustive]`, alle Methoden `async` (D-03), alle mutierenden Methoden cancel-safe (I-14). Hier steht die normative Form; Signaturdetails folgen im Rustdoc, Abweichung ist ein Defekt.

### 7.1 Einstieg und Handles

```rust
pub struct Contextra;                       // geöffnete Instanz, Clone (Arc)
impl Contextra {
    pub fn builder() -> EngineBuilder;       // 5.7
    pub async fn tenant(&self, ctx: TenantContext) -> Result<Tenant, Error>;
    pub async fn erase(&self, req: EraseRequest) -> Result<EraseReceipt, Error>;
    pub async fn erase_status(&self, id: ErasureId) -> Result<ErasureStatus, Error>;
    pub async fn report(&self, id: ErasureId) -> Result<SignedReport, Error>;
    pub async fn change_passphrase(&self, old: &Secret, new: &Secret) -> Result<(), Error>;
    pub fn health(&self) -> HealthReport;    // D-64, nicht async, kein I/O
    pub async fn close(self) -> Result<(), Error>;
}
pub struct Tenant;        // an TenantContext gebunden (D-07)
impl Tenant {
    pub async fn collection(&self, name: &str) -> Result<Collection, Error>;
}
```

Es gibt keine Funktion ohne `TenantContext` (T-API-1, trybuild). `Secret` ist `zeroize`-on-drop, hat kein `Debug`/`Display`/`Clone`.

### 7.2 Dokument-API (`Collection`)

|Methode|Wirkung|Fehler (Auszug)|
|---|---|---|
|`insert(doc_key, Document, subject: Option<SubjectRef>)`|legt an; setzt Betroffenen (D-40)|`Conflict::IdCollision`, `Erased`, `Restricted`, `Limit::*`|
|`update(doc_key, Document)`|neue MVCC-Version; Betroffener unveränderlich|`Conflict::SubjectImmutable`, `NotFound`|
|`get(doc_key, at: Option<SnapshotId>)`|Lesen, optional Zeitreise|`Erased`, `NotFound`, `SnapshotRevoked`|
|`soft_delete(doc_key)`|logisch (Tombstone), **keine Löschaussage** (D-51)|`NotFound`|
|`scan(range)`|Iteration in `doc_tag`-Reihenfolge (D-41)|`Erased` (Elemente werden still übersprungen, nie sichtbar)|
|`search_text(query, k, filter)`|BM25, ReadGate-Filter mit Nachladen (D-62)|`Limit::QueryTooLarge`|
|`search_vector(vec, k, filter)`|HNSW|`Dimension`, `Limit::*`|
|`search_hybrid(query, vec, k, filter)`|RRF aus Text und Vektor|–|
|`snapshot()`|MVCC-Snapshot (mandantengebunden, D-61)|`Limit::Snapshots`|

`Document` = Textchunks (max `max_doc_bytes`), Metadaten (typisierte Schlüssel/Werte, keine Freitext-Nutzerkennung im Schlüsselraum), optionale Vektoren je Chunk. Rustdoc von `soft_delete` beginnt mit: „Für Löschpflichten `erase` verwenden.“ (T-API-3).

### 7.3 Lösch-API

```rust
pub struct EraseRequest {
    pub target: EraseTarget,
    pub timing: Timing,                       // Now | Batched
    pub retention: RetentionAttestation,      // Pflicht (D-15)
    pub mode: EraseMode,                      // Erase | Restrict
}
pub enum EraseTarget {
    Document   { tenant, collection, doc_key },
    Subject    { tenant, subject },           // Art.-17-Standardfall (D-14)
    Collection { tenant, collection },
    Tenant     { tenant },                    // Offboarding (D-38)
}
pub struct RetentionAttestation {
    pub code: RetentionCode,
    pub attested_by: OperatorRef,             // opake Betreiberkennung, nicht im Bericht im Klartext
    pub attested_at: Timestamp,
}
```

**`RetentionCode` (E-L12, abschließend, kein Freitext):**

|Code|Bedeutung (Erklärung des Betreibers)|Folge|
|---|---|---|
|`NoObligation`|Es besteht keine Aufbewahrungspflicht|Löschung läuft|
|`ObligationExpired`|Pflicht bestand und ist abgelaufen|Löschung läuft|
|`ObligationActive`|Pflicht besteht (Art. 17 Abs. 3)|`Refused(RetentionObligation)`; mit `EraseMode::Restrict` Gate-Variante `Restricted`, keine Vernichtung|
|`LegalClaimsPending`|Rechtsansprüche (Art. 17 Abs. 3 lit. e)|wie `ObligationActive`|
|`DataSubjectRequestedRestriction`|Art. 18 verlangt|nur `EraseMode::Restrict`|
|`OperatorOverride`|Betreiber trägt Verantwortung, z. B. gerichtliche Anordnung|Löschung läuft, im Bericht vermerkt|

Contextra prüft die Erklärung nicht inhaltlich (E-L7); sie wird signiert im Bericht geführt.

### 7.4 Rückgabe und Status

```rust
pub struct EraseReceipt { pub id: ErasureId, pub status: ErasureStatus, pub gated_at: Timestamp }
pub enum ErasureStatus {
    Requested, Gated, KeyDestroyed, Purging, Verifying, Completed,
    Pending { reason: PendingReason },      // sichtbar, nie still
    Refused(RefusalReason),
}
```

`erase` kehrt zurück, sobald Gate persistiert **und** Schlüssel vernichtet (`Gated`), oder nach `erase_return_timeout` mit `Pending`. `Completed` entsteht nur aus Zeugen aller Schichten (P1). Idempotenz: gleicher `EraseRequest` liefert dieselbe `ErasureId`.

### 7.5 Fehlermodell

`Error` ist eine geschlossene Enum mit festen Fehlertexten (HR-4, T-LOG-1): `Config`, `Io` (inkl. Poisoned), `Corrupt`, `Auth` (falsche Passphrase), `Erased`, `Restricted`, `NotFound`, `Conflict`, `Limit`, `SnapshotRevoked`, `Cancelled`, `Refused`, `Internal`. Kein Fehler trägt Nutzerinhalt, Schlüssel oder Klartext-Kennung. `Corrupt` ist immer fatal für das betroffene Objekt (P2), nie „best effort“.

---

## 8. Komponenten (je Crate)

Jede Komponente nennt Verantwortung, öffentliche Verträge und Abgrenzung. Ringe und Kanten: 5.1–5.3.

**8.1 `types`** – IDs, Tags (`[u8;16]`-Newtypes), `KeyBuf` (feste Länge, kein String), Fehler, Löschtypen (`ErasureId`, `EraseTarget`, `LayerWitness`, `ResidualRisk`). Kein I/O, kein `async`, `DocId(u64)` entfällt (D-08).

**8.2 `ports`** – Traits: `StorageEngine`, `TextIndex`, `VectorIndex`, `Vfs` (**ein** Trait, D-04), `Clock`, `Rng`, `IdGen`, `ErasureLayer`, `ReadGate`, `RecordSealer`, `LedgerSink`, `Metrics`. Altports (`graph`, `checkpoint`, `license`, `attention`, `plugin`, `reranker`, `embedding`, `query_rewriter`) entfallen. `ErasureLayer`:

```rust
trait ErasureLayer: Send + Sync {
    fn id(&self) -> LayerId;
    fn resolve(&self, t: &EraseTarget) -> BoxFuture<Result<LayerScope>>;   // Umfang bestimmen
    fn purge(&self, s: &LayerScope, cx: &PurgeCx) -> BoxFuture<Result<PurgeOutcome>>;
    fn witness(&self, s: &LayerScope) -> BoxFuture<Result<LayerWitness>>;  // Scanner, nur lesend
}
```

**8.3 `sys`** – einzige `unsafe`-Stelle für OS: `fsync`/`fdatasync`/`F_FULLFSYNC`, `flush_dir`, `rename_durable`, `mlock`, `RLIMIT_CORE`, `PR_SET_DUMPABLE`, Dateirechte. Jede Funktion mit offener Vorbedingung ist `unsafe fn` (D-29). Panics ausgeschlossen.

**8.4 `simd`** – f32-Distanzkerne (Dot, L2, Cosine) mit Skalar-Referenz; Laufzeit-Dispatch; jeder Kernel hat Cross-Check gegen die Referenz (T-SIMD-1); u8-VNNI entfällt (D-28).

**8.5 `durable-fs`** – `RealVfs`: Datei öffnen, positionelles Lesen/Schreiben, `sync_data`/`sync_all`/`sync_dir`, atomarer Austausch (tmp, fsync, rename, Verzeichnis-fsync), exklusiver Lock (D-50), `scrub` (best effort Überschreiben vor Unlink, **keine** Wirkungsbehauptung), Startprobe (D-56). Meldet `fsync`-Fehler unveränderlich nach oben (D-55).

**8.6 `crypto`** – AES-256-GCM-SIV (Records, Keyslots), AES-256-GCM oder ChaCha20-Poly1305 für Block-AEAD (Algorithmus-ID im Header), Argon2id, HKDF-SHA256, BLAKE3 keyed, HMAC-SHA256 (WAL-Kette), Ed25519. Nonces aus OS-CSPRNG. Alle Schlüssel `ZeroizeOnDrop`. Keine Feature-Schalter (D-05).

**8.7 `mvcc`** – Sequenzen, Snapshots, SSI-Validierung. `min_read_snapshot` liefert `Option`; der Sentinel `u64::MAX` (B-07) entfällt: ohne offene Snapshots gilt `current_seq`, nie „alles sichtbar“.

**8.8 `store`** – LSM: WAL (HMAC-Kette, verschlüsselt), Memtable, SSTable mit Block-AEAD, Manifest-Log, Kompaktion, **Purge-Primitive** (`drain_range`, `rewrite_without`, `unlink_verified`). Nur über `TenantScopedStorage` erreichbar (AR-6). Kennt `Vfs`-Trait, nicht `RealVfs` (5.3). `Poisoned`-Flag (D-55). Gruppen-Commit mit ReadGate-Prüfung (D-57).

**8.9 `text`** – Segmentierter BM25-Index (9.7): Textpuffer, Segmente, Merge, Rewrite je Zielsegment. Implementiert `TextIndex` und `ErasureLayer`.

**8.10 `vector`** – HNSW mit Arena, Slot-Zeroing, Batch-Rebuild (D-18), verschlüsselter Snapshot (D-19) nur über `Vfs`. Kein `std::fs`, kein `memmap2` auf Klartext (B-13). Implementiert `VectorIndex` und `ErasureLayer`.

**8.11 `rank`** – nur RRF (D-26), `k_rrf = 60`, deterministische Tie-Breaks nach `doc_tag`.

**8.12 `erasure`** – Löschkern (11): Saga-Treiber, `ErasedSet`/Gate-Log, Journal, Tresor-Protokoll (`FileVault`), Messung, Berichtsbau, Ledger, Restore-Guard. Hängt nur an `types`, `ports`, `crypto` (AR-3).

**8.13 `engine`** – Verdrahtung: `TenantScopedStorage`, Collections, Transaktionen, Suche, Hintergrundtasks, Startsequenz (14.1). Enthält keine Löschlogik.

**8.14 `contextra`** – Fassade, `system_defaults()`, Re-Exports, Doc-Tests.

**8.15 `testkit`** – `FaultVfs` (Fehler-/Crashinjektion je `fsync`, Write, Rename), `ReferenceModel` (naives In-Memory-Orakel), `ManualClock`, deterministischer `Rng`, In-Memory-Ports.

**8.16 `lab`** – unabhängiges Erasure Lab (12.3): eigener Format-Parser, eigene Krypto-Aufrufe, keine Abhängigkeit zu `store`, `engine`, `erasure` (AR-4).

**8.17 `ctx-verify`** – Eigenständige Binärdatei zur Berichtsprüfung. Eingaben: Bericht (kanonisches JSON), `report.pub`, optional Ledger-Exemplar. Prüft: kanonische Serialisierung, Ed25519-Signatur, Hash-Kette der Journal-Reduktion, Konsistenz der Zeugen (jeder Zeuge hat Positivkontrolle), Vollständigkeit der Restrisikoliste gegen Katalog (Anhang D), Zeitordnung. Ausgabe: `VALID` / `INVALID(code)`; Exit-Code 0/2. Kein Netzwerk, keine Secrets.

---

## 9. Datenmodell

### 9.1 Löschcluster-Schlüsselraum (D-31)

Schlüssel (feste Länge, alles blinde Tags, D-09):

```
region(1) ‖ tenant_tag(16) ‖ coll_tag(16) ‖ cluster_tag(16) ‖ doc_tag(16) ‖ family(1) ‖ sub(var, nur numerisch)
```

|Region|Code|Inhalt|
|---|---|---|
|`DATA`|0x01|Dokumentfamilien unter Cluster|
|`LOC`|0x02|Locator `tenant ‖ coll ‖ doc_tag → sealed(cluster_tag)` (D-47)|
|`META`|0x03|Collection-Katalog (`name_tag`), Zähler|

`cluster_tag` = `subject_tag` bei zugeordneten Dokumenten, sonst `doc_tag`-abgeleiteter Einzelcluster (`unassigned ‖ doc_tag[0..4]`-Bucket). Ein Subject-Ziel ist damit **ein** Bereich `region‖tenant‖coll‖cluster`; ein Dokument-Ziel ein Bereich bis `doc_tag`. Matcher: `Range{lo,hi}` fester Länge oder `Exact` (D-10).

### 9.2 Familien

|Family|Code|Inhalt|
|---|---|---|
|`Doc`|0x01|Chunks und Metadaten (Envelope)|
|`DocMap`|0x02|Chunk-Zuordnung, Vektor-Slot-Nummern|
|`Manifest`|0x03|Versionen, Subject-Verweis, Größen|
|`Locator`|0x04|Rückverweis (nur in `LOC` verwendet)|
|`TextBuf`|0x05|noch nicht segmentierte Textdeltas (9.7)|

### 9.3 Tags

`tag = BLAKE3-keyed(K_idx, label ‖ eingaben)[0..16]` mit festen Labels (Anhang C). `K_idx` wird aus `K_master` abgeleitet; ohne `K_master` sind Tags nicht berechenbar. Kollisionen: bei 128 Bit praktisch ausgeschlossen, dennoch geprüft (`Conflict::IdCollision`, D-08).

### 9.4 Record-Envelope (D-32)

```
Envelope := version(1) ‖ alg_id(1) ‖ doc_tag(16) ‖ nonce(12) ‖ ciphertext ‖ tag(16)
AAD := "ctx/rec/v1" ‖ key_bytes ‖ commit_seq(8)?   // key_bytes immer; seq nur bei MVCC-Versionen
DEK := HKDF-SHA256(K_unit, salt_doc, "ctx/dek/v1" ‖ doc_tag)
```

Der Wert enthält Nutzerkennungen (`doc_key`, Metadaten) nur im Ciphertext. Block-AEAD der SSTable ist die zweite Hülle. Wird der Tresor-Slot genullt, ist der Envelope nicht mehr öffenbar (`Erased`).

### 9.5 Lebenszyklus und MVCC

Insert/Update schreibt neue Version; `soft_delete` schreibt Tombstone. `history_horizon` (D-11) begrenzt Versionen; Kompaktion entfernt Versionen oberhalb `min_read_snapshot` **und** älter als Horizont. Das ist keine Löschaussage (D-51). `erase` entfernt **alle** Versionen (Purge drainiert Memtable, WAL-Segmente werden nach Flush zum Ziel-Rewrite herangezogen, 11.4).

### 9.6 Subject-Zuordnung

`subject_tag = tag("ctx/subj/v1", tenant ‖ subject_ref)`. Unveränderlich nach Insert (D-40, T-SUBJ-2). Dokumente ohne Betroffenen sind nur per `Document`, `Collection`, `Tenant` löschbar.

### 9.7 Textindex (D-20, D-49)

- **Textpuffer:** neue und geänderte Dokumente liegen zunächst als `TextBuf`-Records im Cluster des Dokuments (löschbar wie Dokumente).
- **Segmente** (`text/<coll>/*.seg`): unveränderlich, verschlüsselt (`K_file`, D-60), Zielgröße 32 MiB. Inhalt: Termwörterbuch (Terme **nur** als `term_tag`, nie Klartext), Postings je Term als Delta-codierte `doc_ord`-Listen, Dokumenttabelle (`doc_ord → doc_tag`, Länge), Segmentstatistik.
- Ein Dokument liegt zu jedem Zeitpunkt in genau einem Segment oder im Puffer (D-49). Update = Eintrag im alten Segment als gelöscht markieren (Bitmap) und neu in den Puffer; Merge baut Segmente neu.
- **Erase:** betroffenes Segment wird ohne die Zieldokumente **neu geschrieben** (alle Statistiken, df, Längensummen exakt neu berechnet, E-L5), altes Segment `scrub` + Unlink; Messung scannt Segmentverzeichnis (E1) und entschlüsselt Segmente zur Positivkontrolle (E3).
- Suche: Query-Terme → `term_tag` (keyed), Postings pro Segment, BM25 mit globalen Statistiken aus Segmentsummen (`k1 = 1.2`, `b = 0.75`), Kandidatenstrom → ReadGate-Filter (D-62).

### 9.8 Vektorindex (D-18, D-19)

HNSW (`M = 16`, `efConstruction = 200`, Standard `efSearch = 64`), Arena mit festen Slots. Slot-Tabelle `slot → doc_tag, chunk`. `erase`: (1) Gate: Kandidaten mit gegatetem `doc_tag` werden im Suchstrom verworfen; (2) Slot-Zeroing in Arena sofort; (3) Kanten bleiben (enthalten nur Slot-Nummern); (4) Rebuild bei Tombstone-Quote ≥ `vector_rebuild_ratio` oder spätestens zum Ablauf der `erase_sla`. Snapshot: Chunks mit Payload je Slot unter DEK des Dokuments (nach Schlüsselvernichtung unlesbar), Kanten unter `K_file`. Arena-Seiten: `MADV_DONTDUMP`, `mlock` best effort (D-63).

### 9.9 Dateiformate (Rahmen, D-42)

Jede Datei beginnt mit: `magic(8) ‖ format_version(u16) ‖ kind(u8) ‖ alg_ids(u8[4]) ‖ file_id(16) ‖ header_hmac/AEAD`. `file_id` speist `K_file` (D-60). Unbekannte `format_version` → `Corrupt` (fail-closed, P3: keine Legacy-Pfade). Die byte-genauen Layouts liegen in `docs/FORMATS.md` und sind normativ; `lab` und `ctx-verify` implementieren sie unabhängig.

---

## 10. Kryptografie und Schlüsselverwaltung

### 10.1 Hierarchie

```
passphrase ──Argon2id(slot_salt, params)──▶ KEK ──unwrap(AES-256-GCM-SIV)──▶ K_master (32 B, zufällig)
K_master ─HKDF─▶ K_enc   ("ctx/enc/v1")   ─HKDF(kind‖file_id)─▶ K_file
         ─HKDF─▶ K_idx   ("ctx/idx/v1")   (Tags)
         ─HKDF─▶ K_unit  ("ctx/unit/v1")  ─HKDF(salt_doc, doc_tag)─▶ DEK
         ─HKDF─▶ K_sign  ("ctx/sign/v1")  (Ed25519-Seed, verschlüsselt in keys/)
         ─HKDF─▶ K_wal   ("ctx/wal/v1")   (HMAC-Kette)
```

Domänentrennung durch eindeutige Labels; jedes Label steht in Anhang C.

### 10.2 Keyslots und HEADER (D-52)

- `HEADER.a` / `HEADER.b`: abwechselnd geschrieben, je mit Generationszähler und AEAD/CRC-Schutz (`T-HDR-1`: Korruption eines Exemplars führt nie zu Totalverlust).
- 1..4 Slots: `slot_id, anchor_id, kdf_params, slot_salt, wrapped_master(48 B), status`.
- `change_passphrase`: neuen Slot erzeugen → entpacken und prüfen → HEADER schreiben (A/B, fsync, Verzeichnis-fsync) → alten Slot nullen (O(1)). Restrisiko `OldKeyslotRemanence`.
- Falsche Passphrase: `Auth` ohne Hinweis, welcher Teil falsch war; konstante Zeitgrenze durch Argon2-Kosten.

### 10.3 Nonces und AAD

96-Bit-Nonces aus OS-CSPRNG, ≤ 2³² Blöcke je `K_file`, dann neue Datei (D-60). AAD bindet Dateityp, `file_id`, Blockindex und Schlüssel, sodass Umordnen und Kreuz-Kopieren scheitern (T-ENC-2).

### 10.4 WAL-Integrität

HMAC-Kette über Frames (`frame_n.mac = HMAC(K_wal, frame_{n-1}.mac ‖ frame_n)`). Recovery verwirft ab erstem fehlerhaften Frame **nur dann** automatisch, wenn der Rest der Datei Nullen/unbeschrieben ist (Torn Write); sonst `Corrupt` und Start verweigert (T-WAL-1).

### 10.5 Algorithmus-Wechsel (R15)

`alg_id` in jedem Header; neue Dateien verwenden den aktuellen Satz, alte bleiben lesbar bis zur Kompaktion; Neubewertung jährlich (Release-Checkliste, H8).

---

## 11. Löschkern

### 11.1 Ziele und Umfangsauflösung

Je `EraseTarget` ruft jede `ErasureLayer` `resolve` auf; das Ergebnis (Bereiche, `doc_tag`-Mengen, Segmentlisten, Slot-Mengen) wird verschlüsselt in `erasure/scope/<id>` persistiert **vor** dem Gate (damit Recovery denselben Umfang kennt). Umfang wird zum Zeitpunkt `gated_seq` fixiert: alles ≤ `gated_seq` ist im Umfang; Schreibzugriffe danach werden abgewiesen (D-57).

### 11.2 Saga (Zustandsmaschine)

```
Requested ─▶ Gated ─▶ KeyDestroyed ─▶ Purging ─▶ Verifying ─▶ Completed
    │           │            │            │            │
    └───────────┴────────────┴─── Pending(reason) ◀────┘      (jeder Schritt idempotent)
Refused (terminal, vor Gate)
```

Schritte und Dauerhaftigkeit (Normalpfad bis `Gated`: 4 `fsync`, D-58):

1. Validierung, `RetentionAttestation`, Journal `Requested+Intent` (fsync #1).
2. Umfang auflösen und persistieren (`scope/`, nicht im 4-fsync-Pfad für `Document` mit kleinem Umfang: Umfang wird in den Journal-Eintrag eingebettet).
3. Gate: `ErasedSet::add` unter exklusiver Commit-Barriere → Gate-Log-Append (fsync #2) (D-57/58).
4. Tresor-Slots nullen, `sync_data` (fsync #3), Fehlen lesend prüfen.
5. Journal `KeyDestroyed` (fsync #4) → Rückgabe `Gated`.
6. Purge-Fenster (`batch_window`, D-39): Snapshots widerrufen (D-61), Memtable-Drain, Rewrite betroffener SSTables, WAL-Segmente mit Zielframes rotieren und neu schreiben, Unlink, Textsegment-Rewrite, Vektor-Zeroing/Rebuild, Locator-Einträge entfernen.
7. Messung (12), Bericht signieren, Ledger-Eintrag (11.8), Journal `Completed`, Gate-Kompaktion.

**Recovery:** Beim Start liest der Löschkern das Journal; jede nicht `Completed`-Löschung wird beim nächsten Schritt fortgesetzt; Gate-Log wird **vor** dem Öffnen von Lesepfaden geladen (fail-closed). Fehlt der Journal-Eintrag zum Gate, gilt das Gate trotzdem (konservativ) und die Löschung wird vollendet. `Pending` trägt `PendingReason` (`Io`, `SlaExceeded`, `LayerFailed(id)`, `WitnessMissing`).

### 11.3 Gate (`ReadGate`)

Gate-Varianten: `Erased` und `Restricted`. Prüfung in: jedem Lesepfad, jedem Suchstrom, `scan`, Zeitreisen, Commit-Phase (Schreibabweisung), Kompaktion (darf gegatete Bereiche nicht „wiederbeleben“). Datenstruktur: sortierte Bereichsmenge im Speicher (Intervallbaum), O(log n) je Prüfung; Persistenz: `gates.log` (AEAD, Hash-Kette). `Restricted` verhindert Lesen/Suchen/Schreiben, vernichtet nichts; Aufhebung nur durch Betreiber-API `lift_restriction` mit Journal-Eintrag.

### 11.4 Purge

- **Drain:** Memtable- und Puffer-Einträge im Umfang entfernen.
- **Rewrite:** je betroffener SSTable eine neue Datei ohne Zieleinträge (Kandidaten über Dateibereich/Bloom-Index; durch Cluster-Schlüsselraum meist 1–2 Dateien je Level). Neue Datei: schreiben, `sync_data`, Manifest-Log-Eintrag (fsync), erst dann alte Datei `scrub` + Unlink + Verzeichnis-fsync.
- **Verifikation vor Unlink:** neue Datei wird gelesen und gegen Matcher geprüft (Zeuge E1).
- **Plattenreserve:** Rewrite reserviert vorab (D-65); fehlt Platz, `Pending(Io)`, Gate bleibt aktiv.
- **Barrieren:** Purge exklusiv gegen Kompaktion (5.4). Fehler beim Rewrite lässt die alte Datei unverändert (kein Teilzustand, T-CRASH-2).

### 11.5 Tresor-Protokoll (D-33)

`vault/vault.dat`: feste Slotgröße 80 Byte: `doc_tag(16) ‖ salt(32) ‖ status(1) ‖ gen(7) ‖ mac(24)` (Layout normativ in FORMATS.md). Anlegen: Slot schreiben und gruppenweise `sync_data` **vor** den WAL-Frames (I-9), sodass nie ein Record ohne Schlüssel existiert. Löschen: Slot mit Nullen überschreiben, `sync_data`, Slot lesen und Nullen **verifizieren** (T-VAULT-1); bei Fehler `Pending` mit Neuöffnen der Datei (D-55). Slot-Indexierung per Hash-Tabelle im Speicher (aus Datei beim Start). Kompaktion des Tresors (freie Slots wiederverwenden) überschreibt nie einen belegten Slot. Restrisiko `WrappedKeyRemanence` (Flash/COW) wird im Bericht geführt.

### 11.6 Journal

Hash-Kette, AEAD, `fsync` je Eintrag. Einträge enthalten nur `ErasureId`, Zustand, Zeit, Commitments (`BLAKE3-keyed(K_commit_epoch, subject ‖ tenant)`), Zähler, Zeugen-Hashes, Retention-Code. Nach `Completed` wird das Journal reduziert (Zwischenzustände entfernt, R13). Epochen-Rotation (D-48): nach `commit_epoch` neuer Epochenschlüssel, alte Commitments und Ledger-Einträge werden entfernt.

### 11.7 Bericht

Kanonisches JSON (sortierte Schlüssel, keine Gleitkommazahlen, UTF-8 NFC), Ed25519 (D-13). Felder: `report_version`, `store_id`, `erasure_id`, Ziel-Typ (nicht Kennung), Commitment, Zeiten (`requested_at`, `gated_at`, `key_destroyed_at`, `completed_at`), `sla` und Verletzung, Retention-Code, je Schicht `LayerWitness` (Messklasse, Methode, Positivkontrolle, Zähler), Klassenverband G/K/P (12.5), Restrisiken (Katalog Anhang D), Pflichtsatz zur Signatur (E-L11), Hinweis „keine Rechtsberatung“. **Keine** Inhalte, Schlüssel oder Klartext-Kennungen (P13).

### 11.8 Ledger und Restore-Guard (D-37)

Ledger: abgeschlossene Löschungen als `(commitment, scope_range_hash, completed_at)`, Datei **außerhalb** des Datenverzeichnisses über `LedgerSink`. Beim Öffnen: Ledger-Einträge gegen den Store abgleichen; findet sich ein gelöschter Umfang wieder (Restore aus Backup), wird der Store **nicht freigegeben**: Status `Recovering`, automatische Wiederholung der Löschungen (`RestoreReplay`), Bericht mit Vermerk. Fehlt das Ledger (neuer Rechner), meldet `health()` `Degraded: LedgerMissing`; Betreiber bestätigt per `acknowledge_ledger_reset` (Journal-Eintrag). Horizont = `commit_epoch`.

### 11.9 Rechnung: Warum Löschcluster-Schlüsselraum (Grundlage D-02, D-31) **[E]**

Annahmen: Datenbank mit N = 150 SSTables, ein Dokument mit m = 45 Schlüsseln (Familien × Chunks × Versionen), Schlüssel hash-verteilt. `P(Datei betroffen) = 1 − (1 − 1/N)^m ≈ 1 − e^(−m/N) = 1 − e^(−0,3) ≈ 26 %`. Ein Purge schriebe ≈ 39 von 150 Dateien um (Write Amplification ∝ DB-Größe). Mit Cluster-Schlüsselraum liegt das Ziel pro Level in höchstens 1–2 Dateien (7 Level ⇒ ≤ 14 Dateien, praktisch 3–6); Subject-Ziele skalieren mit der Zielgröße. Batch (D-39): _B_ Löschungen im Fenster berühren die Vereinigung der Dateimengen, nicht die Summe. **Gate G0 (verbindlich):** B-ERASE-WA misst Dateien/Bytes je Erase auf 1 GiB und 10 GiB Testbestand. Bestehensschwelle: umgeschriebene Bytes ≤ 3 × Zielbytes + 2 SSTable-Zielgrößen und **nicht** wachsend mit der DB-Größe (Steigung 10 GiB/1 GiB ≤ 1,5). Scheitert G0, tritt die Rückfallebene (redb-Adapter) in Kraft (D-02), die Spezifikation bleibt gültig, Annahmen zu Dateigranularität entfallen und der Bericht weist `PurgeScalesWithDb` aus.

### 11.10 Mandanten-Offboarding (D-38)

`EraseTarget::Tenant`: Bereich `region‖tenant` über alle Regionen; zusätzlich Löschung aller Collection-Katalog-Einträge, Tresor-Slots (Bereichsscan über Tag-Index), Textsegmente und Vektor-Snapshots des Mandanten; Dateien, die ausschließlich Mandantendaten enthalten, werden direkt `scrub`+unlinked, ohne Rewrite. Fremde Mandanten: bitgleich (T-ISO-1, T-TENANT-1).

---

## 12. Messung und Nachweis

### 12.1 Messklassen (D-35)

|Klasse|Methode|Aussage|
|---|---|---|
|**E1**|Strukturscan der Dateien: Matcher (Bereiche, Tags, `doc_tag`) gegen entschlüsselte Struktur und Rohbytes|Kein Eintrag des Ziels im Datenverzeichnis außerhalb des Tresors|
|**E2**|Rohscan aller Dateien (inkl. `tmp/`, alte Segmente, WAL, Snapshots) auf **bekannte Testmuster** (Tag-Bytes, Envelope-Header, Schlüssel-Salt)|Kein Fragment der Tag-/Envelope-Struktur|
|**E3**|Positivkontrolle: Mit dem **gelöschten** DEK (aus Testfixtur vor Löschung gesichert) versucht das Lab, übrig gebliebene Envelopes zu öffnen|Kein Ciphertext des Ziels ist mit dem alten Schlüssel öffenbar|

Ein Zeuge ohne **Positivkontrolle** (Scanner findet bekanntes eingestreutes Muster in einer Kontrolldatei) ist ungültig (HR-10, T-RAW-2). Ein Rohscan auf Klartext allein zählt nicht.

### 12.2 `LayerWitness`

`{layer, class (E1|E2|E3), method_id, scanned_files, scanned_bytes, matches: 0, control_ok: true, duration, hash}`. `matches ≠ 0` ⇒ Status `Pending(WitnessMissing)`, nie `Completed`.

### 12.3 Erasure Lab

`contextra-lab` ist ein separates Programm/Crate (AR-4): eigener Parser aller Formate, eigene Krypto-Aufrufe, keine Produktions-Decoder. Funktionen: Fixturen erzeugen (Dokumente mit bekannten Mustern), Löschung auslösen über die öffentliche API, danach E1–E3 unabhängig ausführen, Differenzprüfung (T-DIFF-1: Nicht-Ziel-Daten bitgleich bis auf erwartete Manifest-/Seq-Änderungen). Mutationstests: absichtlich gebrochene Purge-Implementierungen müssen vom Lab erkannt werden (T-MUT-1).

### 12.4 Erhebung des Messstands

Das Lab läuft in CI (E3-Evidenz: reproduzierbar auf unabhängigem Runner) und als Teil des Gates G-D/G-H. Für Produktionsbetrieb führt die Bibliothek denselben Scanner aus (Eigenmessung, Klasse E1/E2); die Evidenzstufe E4 (Produktaussage) bezieht sich auf das Lab, nicht auf die Eigenmessung.

### 12.5 Klassenverband G, K, P (D-36)

|Klasse|Aussage im Bericht|Voraussetzung|
|---|---|---|
|**G** (Gate)|Ab Zeitpunkt `gated_at` liefern alle Lesepfade `Erased`|T-GATE-1, T-VER-1|
|**K** (Key)|Dokumentschlüssel vernichtet, Fehlen geprüft|T-VAULT-1..4|
|**P** (Purge)|Daten im Datenverzeichnis, in Indexen und Versionen entfernt, E1/E2/E3-Zeugen je Schicht|T-RAW-1..5|

Jede Klasse wird einzeln als `reached/not_reached` ausgewiesen; es gibt keine zusammenfassende Note und keine Rangfolge (die Aussagen sind unabhängig).

---

## 13. Invarianten und Fehlermodi

### 13.1 Invarianten

|ID|Invariante|
|---|---|
|I-1|Kein Schreibzugriff in einen gegateten Bereich nach persistiertem Gate.|
|I-2|Ein gegateter Bereich ist in jedem Lesepfad `Erased`, auch nach Neustart (Gate-Log wird vor Lesepfaden geladen).|
|I-3|Nicht-Ziel-Daten bleiben bitgleich (außer Manifest/Seq).|
|I-4|`Completed` ⇒ alle Schichten haben gültigen Zeugen.|
|I-5|Kein Klartext von Nutzerdaten auf Platte (außerhalb Prozessspeicher).|
|I-6|Jeder Header enthält `format_version` und Algorithmus-IDs.|
|I-7|Hash-Ketten (WAL, Journal, Gate-Log) sind lückenlos; Bruch ⇒ `Corrupt`.|
|I-8|Journal-Eintrag `Requested` ist vor dem Gate dauerhaft.|
|I-9|Tresor-Slot ist dauerhaft **bevor** ein Record unter dem DEK im WAL dauerhaft wird.|
|I-10|Ein `fsync`-Fehler wird nie auf demselben Handle wiederholt (D-55).|
|I-11|Kein Fehlertext enthält Nutzerinhalt (HR-4).|
|I-12|`min_read_snapshot` hat keinen „alles sichtbar“-Sentinel.|
|I-13|Ein Dokument gehört zu höchstens einem Betroffenen, unveränderlich.|
|I-14|Mutierende Operationen sind cancel-safe (Spawn + oneshot).|
|I-15|Jede Löschung ist idempotent und nach Neustart fortsetzbar.|

### 13.2 Fehlermodi (FM)

|ID|Fehlermodus|Reaktion|Test|
|---|---|---|---|
|FM-1|Absturz zwischen Gate und Key-Nullen|Recovery nullt Slot, setzt fort|T-CRASH-1|
|FM-2|Absturz im Rewrite|Alte Datei bleibt gültig; Rewrite wird wiederholt; Waisen in `tmp/` entfernt|T-CRASH-2|
|FM-3|Platte voll beim Purge|Gate bleibt, `Pending(Io)`, Schreibzugriffe `Limit::DiskReserve`|T-CRASH-3|
|FM-4|Abbruch des Aufruferfutures|Task läuft zu Ende|T-CANCEL-1|
|FM-5|`fsync`-Fehler|Datei fatal, `Poisoned`, Neuöffnen|T-FSYNC-1|
|FM-6|Beschädigter HEADER-Slot|zweites Exemplar übernimmt|T-HDR-1|
|FM-7|Korrupter WAL-Frame mitten in der Datei|Start verweigert (`Corrupt`)|T-WAL-1|
|FM-8|Restore aus Backup|Restore-Guard wiederholt Löschungen|T-RESTORE-1|
|FM-9|Journal vorhanden, Gate-Log fehlt|Gate aus Journal rekonstruiert|T-CRASH-4|
|FM-10|Passphrase falsch|`Auth`, keine Datei wird verändert|T-KDF-1|
|FM-11|Unbekanntes Dateisystem|Start verweigert (`UnsupportedFilesystem`)|T-PLAT-1|
|FM-12|SLA überschritten|Bericht weist Verletzung aus, `health()` meldet|T-SLA-1|

---

## 14. Betrieb, Nebenläufigkeit, Plattform

### 14.1 Startsequenz

1. Verzeichnis-Lock (D-50). 2. Startprobe Dateisystem (D-56). 3. HEADER lesen (A/B), Keyslots, Passphrase prüfen. 4. Prozesshärtung (D-63). 5. Orphans in `tmp/` entfernen. 6. Gate-Log laden. 7. Journal lesen, offene Löschungen aufnehmen. 8. Manifest, WAL-Recovery. 9. Tresor laden. 10. Ledger prüfen (Restore-Guard). 11. Lesepfade freigeben, Hintergrundtasks starten, offene Löschungen fortsetzen. Fehler in 1–10 verhindern den Start.

### 14.2 Nebenläufigkeit

Sperren und Reihenfolge laut 5.4. Gruppen-Commit: Leader sammelt Batches, prüft ReadGate, schreibt Tresor-Slots + `sync_data`, dann WAL + `sync_data` (2 fsync je Gruppe). `loom`-Tests für Commit-Barriere, Gate, Purge-Barrier (T-CONC-1).

### 14.3 Drop und Herunterfahren

`Drop` blockiert nie; es setzt Stop-Flags. Geordneter Shutdown nur über `close().await`. Nicht abgeschlossene Löschungen bleiben im Journal.

### 14.4 Kapazität und Grenzen

`max_doc_bytes` 4 MiB (≤ 64 MiB), Snapshots je Mandant ≤ 1024, Dimension ≤ 4096, Query-Terme ≤ 256, `k` ≤ 1000. Überschreitung: `Limit::*`, nie stilles Kürzen. Plattenreserve D-65.

### 14.5 Gesundheitsstatus (D-64)

`HealthReport { state: Ok|Degraded|Poisoned|Recovering, fsync_p50, open_erasures, oldest_open_age, sla_violations, tombstone_ratio, free_bytes, reasons: Vec<DegradedReason> }`. `Metrics`-Namen statisch (`ctx_erase_gated_seconds`, `ctx_fsync_seconds`, …), keine IDs als Labels.

### 14.6 Logging

`tracing`-Fassade mit festen Ereigniscodes, keine Inhalte, keine Kennungen, keine Pfade mit Personenbezug (HR-4, T-LOG-1).

### 14.7 Plattformen (D-56)

Tier 1: Linux x86_64/aarch64, ext4/XFS. Tier 2: macOS APFS (`F_FULLFSYNC`), Linux btrfs/ZFS (Restrisiko `FsJournalOrCow` immer). Nicht unterstützt: Windows, NFS, SMB, FUSE, overlayfs.

### 14.8 Referenzplattform für Benchmarks

Bare-Metal-Linux (x86_64, ≥ 8 Kerne, 32 GiB RAM), ext4 auf NVMe (fsync_p50 ≤ 1,2 ms gemessen), Kernel ≥ 6.1, `rustc 1.89.0`, `--release`, `lto = "thin"`. Die Hardware wird im Benchmark-Protokoll dokumentiert (P9); Gate-Schwellen sind relativ zu `fsync_p50` der Startprobe (D-30).

---

## 15. Teststrategie, Benchmarks, Gates

### 15.1 Testkatalog (Auszug der referenzierten IDs)

|Gruppe|Tests|
|---|---|
|Isolation|T-ISO-1 (Nicht-Ziel bitgleich), T-ISO-2 (Matcher), T-ISO-3 (IdCollision), T-DIFF-1|
|Gate/Lesen|T-GATE-1, T-VER-1, T-SNAP-1, T-SCAN-1, T-RESTR-1|
|Schlüssel|T-VAULT-1..4, T-KEYSLOT-1..3, T-HDR-1, T-KDF-1, T-NONCE-1, T-ENC-1..3, T-FMT-1|
|Messung|T-RAW-1..5, T-CLASS-1, T-PROOF-1, T-REPORT-1, T-MUT-1|
|Crash|T-CRASH-1..4, T-CANCEL-1, T-FSYNC-1, T-WAL-1, T-LOCK-1, T-RESTORE-1|
|Suche|T-TEXT-_, T-VEC-_, T-HYB-1, T-SEARCH-2, T-RANK-1|
|Architektur|T-ARCH-1..4, T-CFG-1/2, T-DEF-1, T-UNSAFE-1, T-LINT-1/2, T-API-1..3, T-PLAT-1|
|Betrieb|T-SLA-1, T-OPS-1/2, T-LOG-1, T-DUMP-1, T-SEC-2, T-SUPPLY-1, T-SUBJ-1/2, T-TENANT-1, T-CONC-1, T-SIMD-1, T-JRN-3|
|Leistung/Beleg|T-LEIST-1..12 (je Zeile Anhang B)|

Methoden: Property-Tests gegen `ReferenceModel`; Crash-Matrix mit `FaultVfs` (Crash nach jedem Write/fsync/Rename); `loom` für Barrieren; Fuzz (Parser aller Formate, Journal-Replay); Mutationstests (`cargo-mutants` auf `erasure`, `classify`, Matcher); `miri` auf `types`/`mvcc`/`simd`-Referenz; `trybuild` für API.

### 15.2 Benchmarks

|ID|Messung|Ziel (Gate-Schwelle, [E])|
|---|---|---|
|B-ERASE-LAT|`erase` bis `Gated` (Dokument)|p50 ≤ 10 ms (NVMe), allgemein ≤ 6 × fsync_p50 + 2 ms, p99 ≤ 5 × p50|
|B-ERASE-WA|Bytes/Dateien je Purge bei 1/10 GiB|Gate G0 (11.9)|
|B-ERASE-BATCH|1000 Erases im Fenster|Dateien berührt ≪ Summe|
|B-WRITE|Insert-Durchsatz Gruppen-Commit|≥ 20 k Dokumente/s (1 KiB) auf Referenz, Regression ≤ 10 %|
|B-GET|`get` p99 (warm)|≤ 200 µs|
|B-TEXT|BM25 top-10 auf 1 M Chunks|p99 ≤ 30 ms|
|B-VEC|HNSW top-10, Recall@10 ≥ 0,95 (1 M × 768)|p99 ≤ 15 ms|
|B-OPEN|`open` ohne Argon2|≤ 2 s bei 10 GiB|
|B-KDF|Argon2id-Latenz Standardparameter|0,3 … 1,5 s|

Zielwerte sind bis zum Lauf WP-6.1 Schwellen, keine Messungen (0). Regressionen blockieren.

### 15.3 Gates

|Gate|Bedingung|
|---|---|
|G-A|`cargo check --workspace --all-targets --locked` grün, `cargo fmt --check` grün (B-01..03)|
|G-B|Rückbau vollständig (0 Verweise auf `graph` etc.), Features/Tokio laut D-54, `deny` ohne offene Ausnahmen (B-05, B-16, B-17)|
|G0|Speicherentscheid (11.9)|
|G-C|Kern-Crates: 0 `unwrap/expect/panic/let _ =/allow`; Clippy `-D warnings`; Sentinel- und Delete-Race-Fixes (B-07, B-08)|
|G-D|Löschkern S0: T-ISO, T-GATE, T-VAULT, T-CRASH, T-RAW, T-PROOF grün; Lab E3; HR-7 erfüllt|
|G-E|S1 Textindex (T-TEXT-*, B-TEXT)|
|G-F|Keyslots/HEADER/Plattformprobe (T-KEYSLOT, T-HDR, T-PLAT)|
|G-G|S2 Vektor (T-VEC-*, B-VEC)|
|G-H|Release: S0–S3 (oder Cut-Linie), alle T-* und B-*, SBOM, signierte Artefakte, Doku-Lint, Support-Dokumente (HR-11)|
|G-LEGAL|extern (6.7), blockiert nur Aussagen jenseits 6.4|

Reihenfolge: A → B → G0 → C → D → F → E → G → H.

---

## 16. Löschcheckliste für Betreiber (Wortlaut endgültig, E-L8)

> **Keine Rechtsberatung.** Diese Liste benennt technische Schritte, die außerhalb von Contextra liegen.

1. Rechtsgrundlage und Aufbewahrungspflichten prüfen; `RetentionAttestation` wahrheitsgemäß setzen.
2. `erase` mit passendem Ziel (`Subject` für Betroffene) auslösen; Status bis `Completed` beobachten (`Pending` > 1 h alarmieren).
3. Bericht mit `ctx-verify` prüfen und gemäß Art. 5 Abs. 2 DSGVO aufbewahren (Commitments laufen nach 365 Tagen ab).
4. **Externe Sicherungen** (Backups, Snapshots der Plattform) separat bereinigen oder nach Restore die Löschung erneut ausführen (Restore-Guard; Ledger **außerhalb** des Backups halten).
5. Kopien beim Betreiber (Logs, Exporte, Caches, Suchindizes anderer Systeme, Embeddings-Dienste) identifizieren und löschen.
6. Empfänger nach Art. 17 Abs. 2 und Art. 19 informieren (`ExternalProcessors` im Bericht).
7. Flash-/Dateisystem-Remanenz bewerten: bei hohem Schutzbedarf Volume-Verschlüsselung mit Schlüsselvernichtung und zertifizierter Medienaussonderung ergänzen.
8. Swap verschlüsseln oder deaktivieren; Coredumps abschalten (Contextra setzt `RLIMIT_CORE=0`).
9. Passphrase-Verwaltung: kein Escrow durch den Hersteller (HR-3); Notfall-Keyslot im eigenen Tresor.
10. DSFA-Baustein (`docs/DSFA-BAUSTEIN.md`) in die eigene Dokumentation übernehmen; Wirksamkeit regelmäßig prüfen.

---

## 17. Support, Lieferkette, Betriebshandbuch

### 17.1 Lieferung je Release

Signierte Quell- und Binärartefakte (Sigstore oder minisign), SBOM (CycloneDX), `cargo-audit`/`cargo-deny`-Bericht, Changelog mit Format-Version, Evidenzbündel (Gate-Protokolle E2/E3), reproduzierbarer Build (Toolchain-Pin 1.89.0, `Cargo.lock` eingecheckt, `--locked`).

### 17.2 Versionierung und Support

SemVer; Format-Version getrennt (D-66). Je Hauptversion ≥ 5 Jahre Sicherheitsupdates (D-67). Formatänderungen: Migrationspfad nur vorwärts, explizit (`migrate`), nie stilles Upgrade (P3). `cargo-semver-checks` in CI.

### 17.3 Schwachstellenmanagement

`SECURITY.md`: Meldeweg (verschlüsselt), Frühwarnung 24 h und Meldung 72 h nach CRA Art. 14 bei aktiv ausgenutzten Schwachstellen, interne Fristen kritisch 7 d / hoch 30 d (HR-8), Advisory-Ausnahmen in `deny.toml` nur mit Ablaufdatum (B-17). CI ohne `continue-on-error` auf Sicherheitsgates.

### 17.4 Runbooks (Pflichtlieferung)

|Runbook|Inhalt|
|---|---|
|RB-1 Passphrase wechseln|`change_passphrase`, Verifikation, alter Slot|
|RB-2 Löschung bleibt `Pending`|Ursachenliste je `PendingReason`, Platz prüfen, Neustart|
|RB-3 `Poisoned`|Ursache (`fsync`-Fehler) auswerten, Neustart, Dateisystem prüfen|
|RB-4 Restore|Ledger bereitstellen, Replay abwarten, Bericht|
|RB-5 Plattenvoll|Reserve, Löschung weiter möglich, Platz schaffen|
|RB-6 Offboarding|`Tenant`-Löschung, Nachweis|
|RB-7 Passphrase verloren|Keine Wiederherstellung (HR-3)|

### 17.5 Dokumentationspflichten

README beginnt mit HR-3-Hinweis und Abschnitt 6.4; Rustdoc je Lösch-API mit Restrisiken; `docs/FORMATS.md`, `docs/DSFA-BAUSTEIN.md`, `docs/OPERATIONS.md`; Wortlaut-Lint über alles (T-LINT-1).

---

## Anhang A – Funktionsinventar (Auszug)

|ID|Funktion|Stufe|
|---|---|---|
|FI-F01|Dokument-CRUD mit Mandanten|S0|
|FI-F02|Zeitreise und Snapshot|S0|
|FI-F03|Startprobe Dateisystem, Plattformstufen|S0|
|FI-F04|`erase` mit Gate, Tresor, Purge, Bericht|S0|
|FI-F05|Keyslots, Passphrasenwechsel|S0|
|FI-F06|BM25-Suche|S1|
|FI-F07|HNSW-Suche|S2|
|FI-F08|Hybridsuche (RRF)|S3|
|FI-F09|Restore-Guard, Ledger|S0|
|FI-F10|Gesundheitsstatus, Metriken|S0|

Fehlermodi: 13.2. Invarianten: 13.1.

## Anhang B – Leistungsbeschreibung (HR-6)

Die Zeilen entsprechen 1.5 und sind einzeln testbar:

|Zeile|Zusage|Tests|
|---|---|---|
|L1|Ziel-Isolation|T-ISO-1, T-DIFF-1|
|L2|ReadGate ab Sperre inkl. Zeitreisen|T-GATE-1, T-VER-1|
|L3|Schlüsselvernichtung je Ziel|T-VAULT-1..4|
|L4|Messklassen E1/E2/E3 nach Abschluss|T-RAW-1..5|
|L5|Crash-/Abbruchsicherheit|T-CRASH-1..4, T-CANCEL-1, T-FSYNC-1|
|L6|Bericht mit Restrisiken|T-REPORT-1, T-PROOF-1|
|L7|Pflichtverschlüsselung|T-DEF-1, T-ENC-1..3|
|L8|Mandantenpflicht|T-API-1|
|L9|Keine Netzwerkfunktion/Telemetrie|T-ARCH-4, T-LOG-1|
|L10|Restore-Guard|T-RESTORE-1|
|L11|Passphrasenwechsel ohne Neuverschlüsselung|T-KEYSLOT-1..3|
|L12|Plattformprobe, Fail-closed|T-PLAT-1, T-FSYNC-1|

Nichtleistungen: 1.6.

## Anhang C – Tag-Labels und Konstanten

|Zweck|Label|
|---|---|
|`tenant_tag`|`ctx/tenant/v1`|
|`coll_tag`|`ctx/coll/v1`|
|`cluster_tag` (Subject)|`ctx/subj/v1`|
|`doc_tag`|`ctx/doc/v1`|
|`term_tag`|`ctx/term/v1`|
|`name_tag`|`ctx/name/v1`|
|Commitment|`ctx/commit/v1` (+ Epochenschlüssel)|
|Schlüsselableitung|`ctx/enc/v1`, `ctx/idx/v1`, `ctx/unit/v1`, `ctx/sign/v1`, `ctx/wal/v1`, `ctx/dek/v1`, `ctx/file/v1`|
|Record-AAD|`ctx/rec/v1`|

Konstanten: Tag 16 B, Salt 32 B, Nonce 12 B, Tresor-Slot 80 B, Segmentziel 32 MiB, `k_rrf = 60`, `commit_epoch = 365 d`.

## Anhang D – Berichtsformat und Restrisikokatalog

**Restrisiken (`ResidualRisk`, geschlossen):** `FlashRemanence`, `FsJournalOrCow`, `WrappedKeyRemanence`, `OldKeyslotRemanence`, `ProcessMemory`, `GraphRebuildPending`, `ReplicaWindow`, `ExternalCopies`, `ExternalProcessors`, `PseudonymCommitment`, `PurgeScalesWithDb`, `KeyCompromise`. Jedes Risiko hat Kennung, Kurztext, Bedingung (wann es zutrifft) und Betreiberhinweis; `ctx-verify` prüft die Vollständigkeit gegen die Bedingungen.

**Pflichttexte im Bericht:** (1) Signaturumfang (E-L11): „Die Signatur beweist Integrität und Herkunft aus dieser Installation, nicht die Wahrheit des Inhalts.“ (2) Hinweis „Keine Rechtsberatung; die rechtliche Wertung obliegt dem Betreiber.“ (3) Außenaussage aus 6.4. (4) Klassen G/K/P je mit `reached`.

## Anhang E – Anzupassende Folgedokumente

|Dokument|Änderung|
|---|---|
|`docs/SPEC.md`|ersetzt durch diese Fassung 3.0.0; alte Fassungen 2.0.0 und 1.0.0-spec.1 entfernen|
|`PLAN.md`|auf 3.0.0 heben (Arbeitspakete, Gates G-A..G-H, Cut-Linie)|
|`VETOES.md`|VETO-D01 aufheben, VETO-D01b einführen (Schlüsselvernichtung in v1)|
|`CONSTITUTION.md`, `ARCHITECTURE.md`, `SOURCE_OF_TRUTH.md`|„keine Ebene B in v1“ streichen; Verweis auf SPEC 3.0.0|
|`SECURITY.md`|CRA-Meldeweg, Fristen, Support-Zeitraum (D-67)|
|`docs/DELETION-DSGVO.md`|an E-L1..E-L14 angleichen; Wortlaut-Lint|
|`ADR-001`/`ADR-002`|ADR-002 ersetzt ADR-001 (D-01)|
|`deny.toml`, Workflows|D-54-Bans, keine `continue-on-error` auf Sicherheitsgates, Advisory-Ablaufdaten|
|`README.md`|HR-3-Hinweis, Außenaussage 6.4|

**Ende der Spezifikation 3.0.0.**

# Contextra v1 – Spezifikation (SPEC)

|Feld|Wert|
|---|---|
|Spezifikationsversion|**4.1.0** (SOLL-Zustand für Release 1.0.0; Revision von 4.0.0; Änderungen: Anhang H, „Änderungen 4.1.0“)|
|Stand|2026-11-02 (4.1.0)|
|Status|**Final bis auf zwei benannte Vorbehalte:** (1) Gate **G0** (Speicherentscheid, 11.9): scheitert es, werden 4.1, 9.1, 11.4 und 11.9 nach der Fallbackleiter (11.9) neu gefasst; (2) Gate **G-LEGAL** (extern, 6.7): blockiert nur Löschaussagen jenseits 6.4. Alles Übrige ist entschieden und begründet (Abschnitt 4).|
|Basis der Analyse|`github.com/tfufuz1/contextra`, `main` @ `25c8f17568025b120ef820f3cb4e45e6107c4f8e` (2026-11-02 00:38 +01:00), frisch geklont. Gegenüber `33c3e78e4` (Basis der Vorprüfung) haben sich nur `contextra-ports` (Altports entfernt, `Vfs` nach `ports` verschoben, `rng`/`vfs` gehärtet) und `contextra-testkit` geändert (24 Dateien, +687/−2.480 Zeilen) **[V]**. Gegenüber der Basis von 3.0.0 (`e5e29108d`) kommen `contextra-sys` (Härtung, `9f1e77633`) hinzu. Alle Zählungen, die nicht ausdrücklich als „neu geprüft“ gekennzeichnet sind, stammen von `33c3e78e4` und sind für die unveränderten Crates weiter gültig. **Ergänzung 4.1.0:** Architektur- und Strukturprüfung am HEAD `f8a3bbb24` (11 Commits nach `25c8f17`: `contextra`, `mvcc`, `durable-fs`, `tests/arch`), statisch **[V]**, ohne Build; neue Befunde F-26 bis F-31, Korrekturen F-07/F-13 (F-29).|
|Prüfmodus|Quelltext gelesen, Manifeste ausgewertet, Muster gezählt (**[V]**). **Nicht gebaut und nicht ausgeführt:** Die Prüfumgebung hatte keine Rust-Toolchain. Alle Build- und Testwerte (**[M]**) stammen aus 3.0.0 (HEAD `e5e29108d`) und wurden nicht wiederholt. Rechtsstände wurden am 2026-11-02 gegen Primär- und Fachquellen geprüft (**[W]**).|
|Quelle der Wahrheit|**Ausschließlich diese Datei.** `AGENTS.md`, `PLAN.md`, `VETOES.md`, `CONSTITUTION.md`, `ARCHITECTURE.md`, `SOURCE_OF_TRUTH.md`, `SECURITY.md` und alle Dateien unter `docs/` außer dieser sind **veraltet und nicht normativ**. Widerspricht eine davon dieser Spezifikation, gilt die Spezifikation; die Datei wird erneuert oder gelöscht (Anhang E). Diese Spezifikation enthält deshalb auch die Byte-Formate (Anhang F) und den Umsetzungsplan (15.4); es gibt kein Begleitdokument.|
|Ersetzt|`docs/SPEC.md` 4.0.0 (und 3.0.0). Änderungen und ihre Begründung: Anhang H.|
|Geltung|Jede Abweichung des Quellcodes oder eines Folgedokuments von diesem Dokument ist ein Defekt.|

> **Keine Rechtsberatung.** Kein Softwareprodukt kann „100 % Rechtskonformität“ zusagen: Rechtskonformität ist ein Merkmal des Betriebs durch den Verantwortlichen, nicht des Codes. Dieses Dokument legt deshalb fest, was Contextra technisch maximal leistet, wie jede Leistung gemessen wird und was Contextra wahrheitsgemäß behaupten darf (6.4, 16). Wo eine Rechtsfrage die Technik berührt, ist sie in 6.6 **konservativ entschieden**: Es gilt die strengste Auslegung. Eine spätere juristische Prüfung kann dadurch Aussagen lockern, nie technische Maßnahmen nachfordern.

---

## 0. Konventionen

- **MUSS / DARF NICHT / SOLL** gelten im Sinne von RFC 2119.
- **IDs:** `P` Prinzip · `D` Entscheidung · `HR` Harte Regel · `R` Rechtsanforderung · `E-L` Rechtsentscheidung · `H` Haftungskopf · `I` Invariante · `FM` Fehlermodus · `FI` Funktionsinventar · `AR` Architekturregel · `RB` Runbook · `F-nn` Befund am Ist-Stand · `KF-n` aufgelöster Dokumentkonflikt · `T-…` Test · `BM-…` Benchmark · `WP-n.m` Arbeitspaket. (Frühere Fassungen nutzten `B-nn` sowohl für Befunde als auch für Benchmarks; das ist getrennt.)
- **Evidenzmarken** (Herkunft einer Aussage): **[M]** gemessen mit Toolchain, Wert aus 3.0.0 übernommen, nicht wiederholt · **[V]** am Repository statisch geprüft (Datei, Zeile, Zählung) · **[W]** am 2026-11-02 im Web geprüft · **[D]** aus Repo-Dokumenten oder Fachwissen übernommen, nicht eigenständig geprüft · **[A]** Argumentation oder Rechnung dieser Spezifikation · **[E]** Entscheidung oder Zielwert. Zielwerte sind bis zum Benchmarklauf (WP-6.1) Gate-Schwellen, keine Messungen.
- **Evidenzstufen** für Aussagen im Repo: **EV0** Behauptung · **EV1** Quelltextbeleg · **EV2** lokaler Gate-Lauf mit Protokoll · **EV3** reproduzierbar in CI auf unabhängigem Runner · **EV4** zusätzlich unabhängiges Orakel, Mutationsprüfung und Nachweis im Erasure Lab. Eine Produktaussage über Löschung ist erst ab **EV4** zulässig.
- **Messklassen** der Löschmessung (12.1): **MK1** Struktur · **MK2** Rohscan · **MK3** Schlüsselprobe. Sie sind von den Evidenzstufen und den Evidenzmarken getrennt benannt.
- `grep`-Zählungen enthalten Inline-Testmodule und sind **Obergrenzen** für Produktionscode. Jede Zählung nennt ihren Befehl oder Geltungsbereich.
- Alle Byte-Layouts sind big-endian, sofern nicht anders angegeben (Anhang F). Alle Zeitangaben sind UTC aus dem Port `Clock`.
- Tags: `tag(label, eingaben)` bezeichnet immer `BLAKE3-keyed(K_idx, label ‖ eingaben)[0..16]` (9.3, Anhang C).

---

## 1. Produkt

### 1.1 Mission

Contextra ist eine **eingebettete Rust-Bibliothek** (Single-Node, kein Server) für lokale Speicherung von Wissen und Gedächtnis von Agenten- und RAG-Systemen: Dokumente (Textchunks, Metadaten, Zuordnung zu einer betroffenen Person), Volltextsuche (BM25), Vektorsuche (HNSW), Hybridsuche (RRF) und Zeitreisen über MVCC-Versionen. Kurzformel: **„SQLite für Agenten, die gezielt und nachweisbar löschen können.“**

**Alleinstellungsmerkmal:** Löschen ist ein **eigener, entkoppelter, gemessener Prozess**. Ein Betreiber ruft `erase(Ziel)` auf und erhält einen signierten, von einem unabhängigen Programm prüfbaren Bericht, der je Schicht ausweist, was entfernt, was gemessen und was nicht erfasst wurde. Zwei Eigenschaften tragen das Merkmal und sind messbar:

1. **Sofortwirkung unabhängig von der Zielgröße (P16):** Die Zeit bis `Gated` (Lesesperre persistiert, Schlüssel des Ziels vernichtet) hängt nicht von der Zahl der Dokumente des Ziels ab. Ein Mandant mit 10⁶ Dokumenten ist so schnell gesperrt und kryptografisch unlesbar wie ein einzelnes Dokument (D-79, BM-ERASE-O1).
2. **Physischer Aufwand skaliert mit dem Ziel, nicht mit der Datenbank (P11):** Der Purge schreibt nur Dateien um, die das Ziel enthalten (D-31, D-71, BM-ERASE-WA). Wo diese Eigenschaft strukturell nur abgeschwächt gilt (Textsegmente eines Betroffenen, 9.7), ist die Grenze ausgewiesen und gemessen, nicht behauptet.

Alles andere ist Dienst am Löschen: Keine Funktion DARF Daten an einem Ort ablegen, den der Löschkern nicht erreicht (P12).

### 1.2 Zielnutzer

Entwickler und Betreiber von Agenten- und RAG-Systemen im EU-Raum (Schwerpunkt Deutschland), die Betroffenenrechte (Art. 17 und 18 DSGVO) technisch umsetzen müssen, einen prüfbaren Nachweis (Art. 5 Abs. 2, Art. 24 DSGVO) brauchen und Enterprise-Anforderungen an Betrieb, Lieferkette und Support stellen.

### 1.3 Produktstufen (jede einzeln lieferbar)

|Stufe|Inhalt|Lieferbares Ergebnis|
|---|---|---|
|**S0**|Gehärteter LSM-Speicher mit Löschcluster-Schlüsselraum, Pflichtverschlüsselung samt Keyslots, Tresor mit Schlüsselhierarchie, Gate, Saga, Purge, Journal, Bericht, Lab, Verifier. Dokumente, Zeitreisen, Mandanten.|Verschlüsselter Dokumentenspeicher mit belegbarer Löschung. Vollwertiges Produkt (Agentengedächtnis ohne Suche).|
|S1|Textsuche als abgeleiteter, segmentierter, verschlüsselter Index|Volltextsuche mit denselben Nachweisen|
|S2|Vektorsuche (HNSW) mit Surrogat-Zeroing, Kantenreparatur, verschlüsseltem Snapshot, deterministischem Rebuild|RAG-Kern|
|S3|Hybridsuche (RRF), API-Politur, Dokumentation, Release-Candidate|v1.0.0|

Reihenfolge-Begründung: S0 beweist das Alleinstellungsmerkmal auf kleinstem Umfang. Jede weitere Stufe fügt eine Löschschicht und neue Fehlerquellen hinzu. Scheitert eine spätere Stufe, bleibt S0 ein vollwertiges Produkt. **Release-Regel (D-59):** v1.0.0 erscheint erst, wenn S0 bis S3 das Gate G-H bestehen. Gibt es bis G-G keine tragfähige Vektor- oder Hybridschicht, gilt die **Cut-Linie**: v1.0.0 = S0 + S1, S2/S3 folgen als v1.1. Es gibt keine halb fertige Löschschicht in einem Release.

### 1.4 Nicht-Ziele v1

Netzwerkdienst, Server-Socket, Sharding, Replikation, P2P; Erzeugung von Embeddings und LLM-Aufrufe (Vektoren und Text kommen vom Aufrufer); Dateiformat-Parsing; Graph und Relationen; Checkpoints, Forks, Hardlink-Klone; persistente Caches; automatische Entitäts- oder Metadatenextraktion; Lizenz- und DRM-Schicht; MCP-Server und Python-Bindings; Mehr-Betroffenen-Dokumente (ein Dokument gehört zu höchstens einem Betroffenen); Rotation des Master-Schlüssels (Passphrasenwechsel ist **kein** Nicht-Ziel, D-52); Telemetrie; Schlüssel-Escrow durch den Hersteller; Löschung außerhalb von Contextra (Backups, Kopien beim Betreiber, Hardware-Ebene); Windows als Plattform (D-56); Präfix-, Wildcard- und Regex-Suche im Textindex (Terme liegen nur als `term_tag` vor, 9.7); Vorfilter für Metadaten (v1.1, Format-Haken in 9.7/9.8); Wasm-Plugins (inkl. `WasmMergeOperator`).

### 1.5 Prüfbare Zusagen (Leistungsbeschreibung, Anhang B)

1. Ein Löschauftrag wirkt ausschließlich auf sein typisiertes Ziel; alle Nicht-Ziel-Daten bleiben auf Record-Ebene bitgleich erhalten (T-ISO-1).
2. Ab persistierter Sperre liefert jeder Lesepfad `Erased`; Zeitreisen eingeschlossen (T-VER-1). Nach der Sperre wird nichts mehr in das Ziel geschrieben (T-GATE-1).
3. Für jedes Ziel wird der Schlüssel seiner höchsten Ebene (Dokument, Betroffener, Collection oder Mandant) aus dem Tresor entfernt und das Fehlen geprüft (T-VAULT-1..4). Die Zeit bis `Gated` ist von der Zielgröße unabhängig (BM-ERASE-O1). Der Bericht weist je Schicht aus, wie weit diese Schlüsselvernichtung reicht (`key_coverage`, 12.5).
4. Nach abgeschlossener Löschung weist der Bericht für jede Schicht eine **Feldmessung** (MK1, MK2; Schlüsselabwesenheit MK3) des Produktionscodes aus. Dieselbe Messung ist mit `ctx-verify scan` unabhängig wiederholbar (D-72). Die unabhängige Belegung aller Messklassen MK1–MK3 erfolgt im CI-Lab am Release (Evidenzbündel).
5. Unterbrechungen (Absturz, I/O-Fehler, Abbruch des Aufrufers) führen nach Neustart zur Vollendung oder zu sichtbarem Status `Pending`. Stille Teilzustände existieren nicht (T-CRASH-1..5, T-CANCEL-1, T-FSYNC-1).
6. Der Bericht nennt, was gemessen wurde, und listet alle zutreffenden Restrisiken (T-REPORT-1, T-PROOF-1).
7. Ein Restore aus einem Backup bringt gelöschte Daten nicht in den Betrieb zurück, solange das Ledger verfügbar ist und die Aufbewahrungsdauer `backup_retention` nicht überschritten ist (T-RESTORE-1..3).

### 1.6 Zusagen, die Contextra nicht macht

„DSGVO-konform“, „rechtskonform“, „garantiert gelöscht“, „unwiederbringlich“, Löschung in Backups, auf Flash-Ebene, in Dateisystem-Journalen, in Swap oder bei externen Verarbeitern (6.4, 16). **Die Formulierung „100 % löschbar, garantiert DSGVO-konform“ ist als Produktaussage verboten** (§ 5 UWG; technisch falsch für Flash, Backups und Dateisystem-Journale): Der Code garantiert einen messbaren technischen Zustand, die rechtliche Wertung trifft der Betreiber.

**Tragfähige Mission-Formulierung (zulässig):** *„Contextra macht Löschen zu einem messbaren, unabhängig prüfbaren Vorgang: Wirkung sofort, Entfernung physisch, Nachweis signiert, Restrisiken benannt.“* Sie ist stärker als ein Garantiesatz, weil sie falsifizierbar ist: Jede Teilaussage hat einen Test (Anhang B), und ein einziger fehlschlagender Test widerlegt sie.

### 1.7 Messlatte für „maximal“

„Maximal rechtssicher“ ist als Technikziel nur operationalisierbar über Eigenschaften, die ein Dritter nachprüfen kann. Contextra misst sich an folgenden, jeweils mit Test und Benchmark hinterlegten Eigenschaften:

|#|Eigenschaft|Beleg|
|---|---|---|
|M1|Sofortwirkung (Gate) in O(1) bezüglich Zielgröße|BM-ERASE-O1, T-GATE-1|
|M2|Kryptografische Unlesbarkeit der Inhalte in O(1) (Schlüsselhierarchie)|T-VAULT-*, T-KEYCOV-1|
|M3|Physische Entfernung mit Aufwand ∝ Ziel|BM-ERASE-WA, BM-ERASE-WA-TEXT|
|M4|Auch freigegebene Dateiblöcke sind ohne Schlüssel wertlos (Dateischlüssel-Vernichtung, D-85)|T-FK-1..3|
|M5|Nachweis durch unabhängigen Prüfer, mit Positivkontrolle|T-RAW-*, T-MUT-1, T-PROOF-1|
|M6|Crash- und Restore-Festigkeit|T-CRASH-*, T-RESTORE-*|
|M7|Ehrliche Restrisikoliste, maschinenprüfbar|Anhang D, `ctx-verify`|

---

## 2. Leitprinzipien (normativ)

|ID|Prinzip|Folge|
|---|---|---|
|P1|**Wirkung vor Behauptung**|Ein Status oder Zähler „gelöscht“ entsteht nur aus einem Zeugen (`LayerWitness`) eines Scanners mit erfolgreicher Positivkontrolle.|
|P2|**Fail-closed**|Unbekannter Zustand, korrupte Daten, abgelaufene Fristen: Verweigerung oder `Pending`, nie Weiterlaufen.|
|P3|**Ein Weg**|Ein Löschpfad, ein Format, eine Fehlerbehandlung je Konzept. Keine Legacy-Lesepfade, keine Downgrade-Pfade.|
|P4|**Ziel-Isolation**|Jede mutierende Operation hat ein typisiertes Ziel. Nachbardaten bleiben auf Record-Ebene bitgleich (messbar).|
|P5|**Keine Schalter für Sicherheit**|Kein `cfg(feature)` und kein Laufzeitparameter, der Verschlüsselung, Integrität, Löschmessung oder Mandantenprüfung abschaltet. Laufzeitparameter, die Schutzgrade verändern (`process_hardening`), erscheinen im Bericht. **Abschließende Ausnahmeliste (zwei Einträge):** (1) `process_hardening` (D-63, Bericht `ProcessMemory`), (2) Feature `testing` (D-46, ersetzt nur den Argon2-Schritt, CI-geprüft T-CFG-2). Jede weitere Ausnahme ist ein Defekt.|
|P6|**Keine Panics, keine verschluckten Fehler**|Im Produktionscode kein `unwrap`, `expect`, `panic!`, `unreachable!`, `let _ =` auf `Result`, kein `unwrap_or_default` auf I/O.|
|P7|**Determinismus über Ports**|Zeit (`Clock`), Zufall (`Rng`), Dateisystem (`Vfs`), IDs (`IdGen`). Ausnahme: Schlüssel-, Salt- und Nonce-Material aus dem OS-CSPRNG.|
|P8|**Kleinster Kern**|Ein Crate oder Modul ist nur im Kern, wenn ein Kernkonsument es braucht (`cargo tree`-belegt).|
|P9|**Messen vor Optimieren**|Jedes Leistungsgate hat eine Basislinie auf dokumentierter Hardware. Ohne Basislinie kein Gate.|
|P10|**Verifizierbarkeit vor Eleganz**|Jede Entscheidung nennt den Test, der sie belegt. Ohne Test keine Entscheidung.|
|P11|**Löschbarkeit als Strukturprinzip**|Schlüsselraum, Dateiformat und Indexe sind so gebaut, dass jedes Löschziel einen zusammenhängenden, kleinen Bereich belegt. Physischer Löschaufwand skaliert mit dem Ziel. Wo ein Index das nicht leistet, steht die Grenze im Bericht und im Benchmark.|
|P12|**Entkopplung**|Der Löschkern kennt nur Traits aus `ports`. Speicher, Textindex und Vektorindex implementieren `ErasureLayer`. Ein Austausch einer Schicht ändert den Löschkern nicht.|
|P13|**Datensparsamkeit im Nachweis**|Journal, Bericht und Ledger enthalten nur Commitments, Bereichs-Tags, Zähler und Zeiten, nie Inhalte, Schlüssel oder Klartext-Kennungen. Dateinamen und Verzeichnisnamen tragen nur Tags (D-75).|
|P14|**Dauerhaftigkeit ist ein Vertrag mit dem Betriebssystem**|Contextra verlässt sich nur auf geprüfte Dateisystem-Eigenschaften (Startprobe), behandelt jeden `fsync`-Fehler als **fatal für die betroffene Datei** und wiederholt nie blind (D-55).|
|P15|**Betreibbarkeit ist Teil des Produkts**|Gesundheitsstatus, feste Fehlertexte, Kapazitätsregeln, Runbooks und Support-Zeiträume sind testbare Lieferbestandteile (14, 17).|
|P16|**Sofortwirkung ist zielgrößenunabhängig**|Alles auf dem Pfad bis `Gated` ist O(1) bezüglich der Zahl der Dokumente des Ziels (Schlüsselhierarchie, Bereichs-Gate). Zielgrößenabhängige Arbeit gehört in den Purge.|
|P17|**Ehrliche Schicht-Reichweite**|Jede Schutzmaßnahme weist je Schicht aus, was sie abdeckt (`key_coverage`, Restrisiken). Eine Aussage über „das System“ aus der Wirkung auf einen Teil ist verboten.|

---

## 3. Befund und Konsolidierung (Review-Ergebnis)

### 3.1 Gesamturteil zur Produktionsreife

**Das Projekt ist am HEAD nicht produktionsreif und nach den Messungen von 3.0.0 nicht einmal lauffähig [M].** Die Spezifikationen sind weitgehend tragfähig, der Code folgt ihnen nur teilweise. Die Mission (nachweisbare, isolierte Löschung) ist **im Code nicht vorhanden**: Das Crate `contextra-erasure` enthält nur `README.md` und `AGENTS.md`, keine Quelldatei [V, am HEAD `25c8f17` erneut bestätigt]. Entscheidung: **weiterbauen auf dem Fundament** (Store, MVCC, Krypto, SIMD, Testkit), **neu bauen** den Löschkern, **umbauen** Textindex, Vektorpersistenz, Schlüsselraum und Engine-Schreibpfad. Ein Neuanfang wäre teurer, weil 345 Testdateien und 102.000 Zeilen `src` [M] als Orakel und Regressionsschutz erhalten bleiben.

Die Revision 4.0.0 behebt **vier strukturelle Lücken in der Spezifikation selbst**, die das Alleinstellungsmerkmal berührten (Details Anhang H):

1. P11 galt nur für den LSM-Kern; Locator-Region und Textsegmente verstreuten Löschziele (D-70, D-71).
2. Die Schlüsselvernichtung deckte nur Dokumentspeicher und Vektor-Payload; Textpostings und Dateistrukturen blieben unter einem nie vernichteten Schlüssel (D-69, D-85).
3. Das Messversprechen war in Produktion nicht einlösbar (Messklasse „E3“ braucht den gelöschten Schlüssel; Namenskollision der Evidenzbegriffe) (D-72).
4. Subject- und Mandanten-Löschung waren in der Schlüsselklasse O(m) statt O(1) (D-79).

### 3.2 Ist-Zustand am HEAD

Marken: **[V]** geprüft am Repository (HEAD `25c8f17` bzw. unverändert gegenüber `33c3e78e4`); **[M]** aus 3.0.0 übernommen (nicht wiederholt).

|ID|Befund|Beleg|
|---|---|---|
|F-01|**`contextra-engine` (lib) kompiliert nicht**: 33 Fehler (18× E0609, 6× undefinierter `cfg`-Wert, 4× E0599, 3× E0433) in `collection/search/hybrid`, `transaction/*`, `query_builder`, `background_workers`. Die Fassade `contextra` baut damit nicht. **[M]**|`cargo check --workspace --all-targets --locked` (3.0.0)|
|F-02|`--all-targets` scheitert in 24 Einheiten: `crypto` (9), `engine` (lib test: 171 Fehler), `ports`, `rank` (3), `store` (4), `text` (2), `vector` (2). **[M]**|`--keep-going` (3.0.0)|
|F-03|`cargo fmt --check` schlägt fehl (67 Diff-Blöcke). **[M]**|3.0.0|
|F-04|`crates/contextra-erasure/`: nur `README.md`, `AGENTS.md`; `crates/contextra-lab`, `ctx-verify` fehlen; `ports` und `types` enthalten die Löschtypen nicht. **[V]**|`ls`, `find`|
|F-05|Rückbau halb fertig: Workspace listet 15 Mitglieder (ohne `contextra-erasure`). **100 Zeilen** verweisen auf `contextra_graph`/`contextra-graph` (33c3e78e4: 101). Der Commit `c2a864d` hat die Ports `graph`, `graph_index`, `checkpoint`, `license`, `attention`, `plugin`, `lifecycle` entfernt; `ports` führt weiter `embedding`, `reranker`, `query_rewriter` (D-24: zu entfernen). `AutoExtractionMode` in 10+ Dateien. **[V]**|`grep -rIn "contextra_graph\|contextra-graph" crates tests tools .github Cargo.toml Cargo.lock`|
|F-06|**Undeklarierte `cfg(feature)`** (kein `[features]`-Eintrag): `encryption-at-rest` (11 Treffer, `store`), `adaptive-candidate-pool-sizing` (7), `onnx` (2), `graph-connectivity-health` (1), `sandbox` (1) (alle `engine`). 11 Treffer `#[allow(unexpected_cfgs)]`. **[V]**|Skript über alle `cfg(feature = …)` gegen `[features]` je Crate|
|F-07|*(Stand `f8a3bbb24`: siehe F-29)* **Snapshot-Sentinel unverändert:** `.unwrap_or(u64::MAX)` in `store/src/lsm/ops/compaction.rs:228`. **[V]**|Quelltext|
|F-08|**Delete-Race unverändert:** `let _guard = …lock_for(id).await; drop(_guard);` in `engine/src/collection/crud/delete.rs`. **[V]**|Quelltext|
|F-09|**Verschlüsselung optional:** `encryption_mode: Option<EncryptionMode>` im Builder (`contextra/src/builder.rs:86`); `Option<Arc<KeyManager>>` im WAL (`store/src/wal/mod.rs:323,353,401,445`). **[V]**|Quelltext|
|F-10|**Mandantenpolitik:** `TenantPolicy` Default `Optional` (`engine/src/lib.rs:333`). **[V]**|Quelltext|
|F-11|**KDF:** Untergrenze `MIN_M_COST_KIB = 19456`, `t ≥ 2` (`crypto/src/kdf.rs:36`); keine Obergrenze. `try_new(passphrase, salt)` leitet per HKDF aus der Passphrase ab. **[V]**|Quelltext|
|F-12|**`DocId(pub u64)`** (Feature `docid-128` → `u128`), unkeyed (`types/src/types/domain/ids.rs:173,180`). **[V]**|Quelltext|
|F-13|*(Stand `f8a3bbb24`: Teil 1 behoben, siehe F-29)* **Zwei `Vfs`-Traits:** `ports/src/vfs.rs:115` (seit `c2a864d` gehärtet) und `durable-fs/src/vfs.rs:54` (noch vorhanden); `store` greift 155-mal direkt auf das Dateisystem zu, `vector` 53-mal (Zählung 33c3e78e4). **[V]**|`grep "std::fs\|tokio::fs\|File::open\|File::create"`|
|F-14|`block_on` an 7 Stellen in `crates/*/src`, davon **1 produktiv** (`engine/src/merge_operator_adapter.rs:114`, `WasmMergeOperator`-Worker). Die übrigen 6 liegen in Testmodulen. **[V]**|`grep`|
|F-15|Produktionsmuster (Obergrenzen): `.unwrap()` **1.120** · `.expect(` **1.043** · `panic!` **72** (exakt wie 3.0.0). `let _ =` 237 · `#[allow` 134 · `HashMap/HashSet` 211 · `Instant::now/SystemTime::now` 24 · `thread_rng/from_entropy` 12 **[M]**|`grep` über `crates/*/src`|
|F-16|**`tokio` mit Feature `full`** im Workspace (`Cargo.toml`); `Cargo.lock` enthält `mio` und `socket2`. Quelltext nutzt `tokio::net`/`std::net` nicht (0 Treffer). Das widerspricht HR-1, solange `net` im Baum ist. **[V]**|`Cargo.toml`, `Cargo.lock`, `grep`|
|F-17|CI: 19 Workflows, **15× `continue-on-error`**. `deny.toml`: 6 ignorierte Advisories **ohne Ablaufdatum**; `clarify` und `wrappers` verweisen auf entfernte Crates (`contextra-mcp`, `-db`, `-graph`, …). **[V]**|`.github/workflows`, `deny.toml`|
|F-18|`unsafe`: `contextra-simd` 278 Treffer, `contextra-sys` 57 Treffer (nach dem Härtungs-Commit `9f1e77633`; 3.0.0: 269/21). 12 Crates tragen `#![forbid(unsafe_code)]`. Zwei `safe`-Funktionen mit `unsafe`-Inhalt (`mmap_readonly`, `reopen_and_dup2`) laut Audit [D]. **[V]** für die Zählung|`grep unsafe`|
|F-19|Positiv: Workspace-Lints (`unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, …) sind `deny`; `mvcc` 47/47, `simd` 13/13, `sys` 30/30 Lib-Tests bestanden (3.0.0). **[M]**|`cargo test --lib`|
|F-20|`governance/ratchet.toml` (in 3.0.0 als Budget-Quelle referenziert) existiert nicht. **[V]**|`ls`|
|F-21|**Downgrade-Pfad im WAL-HMAC:** Laufzeitparameter `allow_legacy_integrity_key_fallback` (`store/src/wal/io.rs:27,274,422`, `flusher.rs:182`) und Feature `legacy-wal-key`. Widerspricht P3 und D-05. **[V]**|Quelltext|
|F-22|**Windows-Code in der `unsafe`-Insel:** `contextra-sys/src/acl_win32.rs` und 24 `cfg(windows)`-Stellen, obwohl Windows nicht unterstützt wird (D-56). Vergrößert die Auditfläche ohne Nutzen. **[V]**|`grep`|
|F-23|**Produktfremde Features in Produktionscrates:** `experimental-diskann` (`engine`, `vector`), `experimental-rabitq`, `experimental-predicate-augmented-search`, `partial-index-rebuild`, `graph` (`vector`); `memory-only-storage`, `fault-injection`, `legacy-wal-key`, `block-cache-v2`, `sieve-cache` (`store`); `cloud-egress-guard`, `kv-encryption` (`crypto`); `auto-extraction-opt-out`, `entity-extraction`, `edge-reinforcement-learning`, `reranking` (`engine`). **[V]**|`[features]`-Abschnitte|
|F-24|**Kryptoalgorithmen:** Im Code existiert nur AES-256-GCM-SIV (`aes-gcm-siv`); AES-256-GCM und ChaCha20-Poly1305 (so in 3.0.0 8.6 für Block-AEAD genannt) sind nicht vorhanden. Nonce-Schema im Code: 4 Byte Zufallspräfix je `KeyManager` plus 8-Byte-Zähler (`crypto/src/crypto.rs`), nicht „96 Bit OS-CSPRNG“ wie in 3.0.0 D-60. **[V]**|Quelltext|
|F-25|Dokumente widersprechen sich (`VETOES.md`, `CONSTITUTION.md`, `ARCHITECTURE.md`, `SECURITY.md`, `SOURCE_OF_TRUTH.md`, `PLAN.md`, `docs/DELETION-DSGVO.md` zu Ebene B, Fassungen, Gates). Aufgelöst durch die Festlegung „nur SPEC.md ist Quelle der Wahrheit“ (Kopf, Anhang E). **[D]**|Dokumentvergleich|
|F-26|**Fassade `contextra` widerspricht §7 und D-05/D-07/D-23/D-24:** `builder(dimension)` mit fester Dimension, `EncryptionMode::Unencrypted`, `LicenseGate`/`with_license_gate`, `TextEmbeddingEngine`/`with_embedder`, `LlmQueryRewriter`, `collection_profile.rs`, `performance_profile.rs`, Re-Export von `DocId` und `chunker`; kein `tenant()`, `erase()`, `report()`, kein `DocRef`. `lib.rs` behauptet „zero direct dependencies on Ring 0/1“, die `Cargo.toml` hängt an `types`, `ports`, `crypto`, `store`, `rank`. `README` nennt `Collection::delete` (D-51). `AGENTS.md` enthält einen absoluten Entwicklerpfad. **[V]**|`crates/contextra/src`, `Cargo.toml`, `README.md`, `AGENTS.md`|
|F-27|**Abhängigkeitskanten gegen Matrix 5.3** (aus den `Cargo.toml`): `engine→sys`, `vector→sys`, `vector→mvcc` nicht in der Matrix; `engine→durable-fs` und `engine→erasure` (vorgeschrieben) fehlen; `text→crypto` (erlaubt, nötig für D-85) fehlt; für `contextra` gab es keine Matrixzeile (behoben durch D-88). `tokio` mit `features = ["full"]` wird von 10 Crates geerbt (F-16). **[V]**|`Cargo.toml` aller Crates|
|F-28|**Ring-Angaben in `lib.rs`-Köpfen widersprechen 5.1:** `store` „Ring 1“, `text` und `rank` „Ring 0“, `vector` „Layer-1“, `engine` „Ring 2“ und „Layer 3“ im selben Kopf; `types` nennt 30 Konsumenten, davon rund 20 gelöschte Crates. Ursache: handgeschriebene Angaben ohne Test (D-91, T-ARCH-5). **[V]**|`crates/*/src/lib.rs`|
|F-29|**Korrektur zu F-07 und F-13 am HEAD `f8a3bbb24`:** (a) F-13: Das zweite `Vfs`-Trait in `durable-fs` ist entfernt (`RealVfs` implementiert `ports::Vfs`); offen bleiben **169** direkte Dateizugriffe in `store` und **51** in `vector` (`grep "std::fs\|tokio::fs\|File::open\|File::create\|OpenOptions"`). (b) F-07: Der Treffer `compaction.rs:228` ist nicht mehr vorhanden; der Sentinel steht aber weiter in `mvcc/src/snapshot.rs:314` (`update_min`: `.unwrap_or(u64::MAX)`, Kommentar „ist der korrekte Standardwert“) und widerspricht 8.7. **[V]**|`grep`, Quelltext|
|F-30|**Benennung und Struktur:** Modul `engine/src/contextra_impl` (die eigentliche Instanz, Name ohne Aussage); `Cargo.toml` führt den Alias `contextra-index` auf `contextra-vector`; `types/src/types.rs` + `types/src/types/` und `types::types::domain` (Crate, Modul und Pfad gleichnamig); Dateien `misc.rs` (`types`) und `util.rs` (`store`); gemischte Modulform (`foo.rs` + `foo/` neben `mod.rs`) in 5 Fällen (`store/compaction`, `store/sstable`, `text/inverted`, `text/morphology`, `types/types`); Dateiköpfe mit `FILE-CONTEXT`, `ANCHOR[...]`, `STAND: TS:…`, `SESSION:`. **[V]**|`find`, `grep`|
|F-31|**Umfang gegenüber D-24/D-26/D-84 (Zeilen `src`, inkl. Inline-Tests, Obergrenzen):** `vector`: `diskann` 4.122, `quantize` 1.116, `quantize_rabitq` 506, `partial_rebuild` 363, `acorn` 213; `rank` gesamt 5.553 (`fusion` 2.252, `dibud` 524, `explain` 469, `drift` 139) bei Soll „nur RRF“; `types/types/saos.rs` 940; `engine`: `chunker` 1.533, `extraction` 642, `export`+`import` 528, `decay_controller` 115, `temporal_filter` 359, `merge_operator_adapter` 201. Restmuster (`crates/*/src`, Dateien/Treffer): `DocId` 95/791, `contextra_graph` 18/78, `LicenseGate` 5/25, `AutoExtractionMode` 4/24, `TextEmbeddingEngine` 10/28, `Reranker` 9/76, `TenantPolicy` 3/10, `saos` 4/20, `allow_legacy_integrity_key_fallback` 28 Treffer, `Option<Arc<KeyManager>>`/`Option<EncryptionMode>` 15 Treffer. Undeklarierte `cfg(feature)`: `encryption-at-rest`, `adaptive-candidate-pool-sizing`, `partial-index-rebuild`, `onnx`, `graph-connectivity-health`. **[V]**|`wc -l`, `grep`|

### 3.3 Auflösung der Dokumentkonflikte (verbindlich)

|#|Konflikt|Entscheidung|Begründung|
|---|---|---|---|
|KF-1|Schlüsselvernichtung in v1? (ADR-001/VETO-D01 „nein“ vs. SPEC D-01 „ja“)|**Ja.** Drei Säulen (Gate und Purge, Schlüsselvernichtung, Messung). ADR-002 ersetzt ADR-001; VETO-D01 entfällt (Anhang E).|Das Datenformat (Record-Envelope, Tresor-Salt) lässt sich nach Auslieferung nicht rückwirkend einführen. Physisches Überschreiben ist auf Flash/COW unzuverlässig; Schlüsselvernichtung reduziert den Aufwand auf kleine Objekte und ist technisch anerkannt (NIST SP 800-88 Rev. 2, Cryptographic Erase [D]). Die rechtliche Wertung bleibt beim Betreiber (E-L2).|
|KF-2|Restriktion (Art. 18) in v1?|**Ja** als Gate-Variante `Restricted` (D-15).|Das Gate existiert ohnehin; Nachrüsten kostet mehr als Mitliefern.|
|KF-3|Textindex: Postings als Einzelschlüssel oder Segmentindex?|**Segmentindex** (D-20).|Einzelschlüssel streuen jedes Dokument über den Schlüsselraum und zerstören P11.|
|KF-4|Erase-Rückgabe nach `Gated`|bestätigt, p50 plattformrelativ (D-30).|–|
|KF-5|`K_master = Argon2id(passphrase)` oder zufällig mit Keyslots?|**Keyslots** (D-52).|Passphrasenwechsel ohne Neuverschlüsselung (ISO 27001 A.8.24, Personalwechsel).|
|KF-6|`Collection::delete`|**Umbenannt** in `soft_delete` (D-51).|Das Verb „delete“ suggeriert Löschung (H1).|
|KF-7|Crate-Zahl|14 Produktions-Crates + `testkit` + `tests/arch` + `lab` + `ctx-verify`.|5.1|
|KF-8|Rechtsfristen|Siehe 6.2/6.6 (Stand 2026-11-02 [W]): CRA-Meldepflichten (Art. 14) **seit 11.09.2026**, übrige CRA-Pflichten ab 11.12.2027; neues ProdHaftG (Software als Produkt) vom Bundestag am **08.10.2026** beschlossen, gilt für Produkte, die ab **09.12.2026** in Verkehr gebracht werden; EDPB Guidelines 02/2026 (Anonymisierung) als Entwurf, **Konsultation am 30.10.2026 (23:59 CET) beendet**, Endfassung offen.|6.2, 6.6|
|KF-9|Wo stehen Byte-Formate und Umsetzungsplan?|**In dieser Datei** (Anhang F, 15.4). `docs/FORMATS.md` und `PLAN.md` sind nicht normativ.|Einzige Quelle der Wahrheit; `lab` und `ctx-verify` implementieren die Formate aus Anhang F unabhängig.|

### 3.4 Nicht geprüft (bewusst nicht behauptet)

Kompilier-, `fmt`-, `clippy`-, Test- und Benchmarkstand am neuen HEAD (keine Toolchain; Werte aus 3.0.0); `miri`, `loom`, Fuzz, Mutation; Inhalt der nicht kompilierenden Teile von `contextra-engine`; HNSW-Snapshot-Format im Detail; ob der Blockcache entschlüsselte Blöcke hält; Kompaktions-Retention-Semantik; konkrete Argon2-Parameterprüfung über `kdf.rs:36` hinaus. Rechtsquellen jenseits ProdHaftG, EDPB 02/2026 und Rust-Release-Stand (6.2) sind **[D]** und gehören in Gate G-LEGAL. Jede Lücke hat ein Arbeitspaket mit Messung (15.4).

---

## 4. Entscheidungsregister (final)

Jede Entscheidung nennt Begründung und belegenden Test. **Fett** gesetzte IDs wurden gegenüber 3.0.0 geändert oder sind neu. `entfällt` heißt: in v1 nicht vorhanden; die ID bleibt reserviert.

### 4.1 Löschen, Speicher, Schlüssel

|ID|Entscheidung|Begründung|Test / Beleg|
|---|---|---|---|
|**D-01**|**Löschen in drei Säulen, alle in v1:** (1) Gate und physischer Purge, (2) Schlüsselvernichtung über den Tresor (höchste Ebene des Ziels, D-79), (3) unabhängige Messung mit signiertem Bericht. ADR-002 ersetzt ADR-001. Die Säulen sind **Verteidigung in der Tiefe, keine rechtliche Wertung** (E-L2). **Die Reichweite der Schlüsselvernichtung ist je Schicht ausgewiesen (`key_coverage`, D-69):** sie macht Dokumentspeicher und Vektor-Payload unlesbar und löst die Bindung der Textpostings; Kanten und Dateistrukturen werden durch Purge und Dateischlüssel-Vernichtung (D-85) beseitigt, bis dahin schützt das Gate.|Überschreiben garantiert auf Flash/COW nichts. Der Satz „Remanenzfläche = 80-Byte-Objekt je Dokument“ aus 3.0.0 galt nur für Dokumentspeicher und Vektor-Payload und entfällt (P17).|T-VAULT-*, T-KEYCOV-1, T-RAW-*, T-PROOF-1|
|**D-02**|**Eigener LSM-Tree bleibt, nur mit Löschcluster-Schlüsselraum (D-31).** Gate **G0** (11.9) entscheidet verbindlich mit Zahlen und läuft als Prototyp **mit** den Varianten D-70 und D-71. Rückfall nur über die Fallbackleiter 11.9; jede Stufe bedeutet Neufassung der betroffenen Abschnitte, nicht stilles Weiterlaufen.|Der LSM-Vorteil ist die Dateigranularität. Mit verstreuten Hash-Schlüsseln läge ein Dokument in ca. 26 % aller Dateien (Rechnung 11.9).|BM-ERASE-WA, T-DIFF-1|
|**D-03**|Ports mit Heißpfad (`StorageEngine`, `TextIndex`, `VectorIndex`) sind **statisch dispatcht**: `async fn` in Traits bzw. `-> impl Future + Send`; die Engine ist generisch darüber. Kalte Ports (`ErasureLayer`, `RecordSealer`, `FileKeyring`, `LedgerSink`, `Metrics`, `Clock`, `Rng`, `IdGen`) und `Vfs` sind dyn-fähig (`BoxFuture`). Kein `block_on` im Bibliothekscode.|`BoxFuture` allokiert je Aufruf und verhindert Inlining; auf `get`, `scan` und Postings-Abfragen ist das messbar, bei Datei-I/O nicht. Synchrone Ports über asynchroner I/O erzwingen `block_on` und Deadlocks (F-14).|T-LINT-2, BM-GET|
|D-04|**Ein** `Vfs`-Trait in `ports`, Implementierung in `durable-fs`. Store, Vektor, Text und Erasure nutzen ausschließlich `Vfs`.|F-13. Fehler- und Crashinjektion setzt `Vfs` voraus.|T-LINT-2, T-CRASH-*|
|D-05|Verschlüsselung und WAL-Integrität sind nicht abschaltbar. Kein `Option<KeyManager>`, kein Klartextmodus, kein Feature `wal-integrity`/`encryption-at-rest`/`legacy-wal-key`, **kein Laufzeitparameter für Integritäts-Downgrade** (`allow_legacy_integrity_key_fallback` entfällt).|F-06, F-09, F-21. Ein Opt-out erzeugt Parallelbestände und einen Angriffspfad.|T-DEF-1, T-CFG-1|
|**D-06**|Passphrase nur über Argon2id. **Grenzen (hart, vor dem Start der Ableitung geprüft):** `m` ∈ [64 MiB, 4 GiB], `t` ∈ [3, 16], `p` ∈ [1, 16]. Verstoß: `Corrupt` (Header) bzw. `Config` (Builder). HKDF nur für Subkeys.|F-11. Untergrenze folgt RFC 9106 (64 MiB, t = 3) [D]. Die Obergrenze verhindert Ressourcenerschöpfung durch einen manipulierten Header (CWE-770); der Header wird vor der Passphrasenprüfung gelesen.|T-KDF-1, T-KDF-2 (Fuzz)|
|D-68|KDF-**Standard** m = 128 MiB, t = 3, p = 4; die Parameter stehen im Keyslot und sind je Slot änderbar.|Parallelität erhöht die Kosten für Angreifer bei gleicher Latenz des Betreibers.|T-KDF-1|
|D-07|`TenantContext` ist Pflicht. Keine mandantenlose Zugriffsfunktion; `TenantPolicy` entfällt.|F-10.|T-API-1 (trybuild)|
|**D-08**|Dokumentidentität: `doc_tag = tag("ctx/doc/v2", tenant_tag ‖ coll_tag ‖ subject_tag_oder_0¹⁶ ‖ doc_key)`. **Der Betroffene ist Teil der Identität** (D-70). Kollision zweier verschiedener `(subject, doc_key)` mit gleichem `doc_tag` → `Conflict::IdCollision` (Vergleich der entschlüsselten Identität im vorhandenen Record).|F-12. 128 Bit keyed, nicht aus Nutzerkennung berechenbar.|T-ISO-3, T-DOCREF-1|
|D-09|Alle Schlüsselkomponenten **und alle Datei- und Verzeichnisnamen** sind blinde Tags oder zufällige IDs (D-75). Nutzerkennungen stehen nur verschlüsselt im Wert.|Der Indexbereich einer SSTable und der Verzeichnisbaum dürfen keine Nutzerdaten enthalten.|T-RAW-4, T-RAW-6|
|D-10|Matcher sind **Schlüsselbereiche fester Tag-Länge** plus **exakte Schlüssel**. Keine String-Präfixe, keine variablen Präfixe.|Variable Präfixe waren Ursache des Datenvernichtungsfehlers im Altbestand (HR-7).|T-ISO-2, T-KEY-1|
|D-11|Zeitreisen bleiben. Purge entfernt alle Versionen. Offene Snapshots **desselben Mandanten** (D-61), die ein Ziel berühren, werden widerrufen (`SnapshotRevoked`). `history_horizon` (Standard 24 h) begrenzt die Versionsmenge.|Warten auf Snapshots macht Löschfristen unbegrenzt.|T-VER-1, T-SNAP-1|
|D-12|**ReadGate** vor Purge, auch für Zeitreisen.|Sofortwirkung für Nutzer.|T-VER-1|
|D-13|Bericht: Ed25519-signiert, kanonisches JSON, ein Format. Die Signatur beweist Integrität und Herkunft aus der Installation, nicht die Wahrheit des Inhalts. Der Betreiber hinterlegt `report.pub` bei Anlage des Stores an einem Ort außerhalb des Datenverzeichnisses; `ctx-verify` verlangt den öffentlichen Schlüssel als expliziten Parameter.|Symmetrische Beweise sind durch den Prüfer fälschbar; eine im Datenverzeichnis liegende Schlüsseldatei beweist nichts.|T-PROOF-1|
|**D-14**|Betroffenen-Löschung über `subject_tag` = Cluster (D-31). Schlüsselvernichtung über den `S`-Slot (D-79).|Ohne Zuordnung findet Contextra eine Person nicht.|T-SUBJ-1|
|D-15|`RetentionAttestation` ist Pflicht. **`Restricted` (Art. 18) ist Gate-Variante in v1**; eine Fristenautomatik ist kein v1-Umfang.|KF-2.|T-API-2, T-RESTR-1|
|D-16|Keine Checkpoints, Forks, Hardlink-Klone.|Jede Kopie liegt außerhalb der Löschgrenze.|T-ARCH-3|
|D-17|Graph und Relationen entfernt, nicht per Feature abgeschaltet.|P5; Kantenreparatur pro Löschung im Graph ist fehleranfällig. (Nicht zu verwechseln mit den HNSW-Kanten, D-77.)|T-ARCH-3|
|**D-18**|Vektorlöschung: siehe **D-77** (Surrogat sofort, Batch-Kantenreparatur im Purge-Fenster, deterministischer Rebuild). Die Fassung „Nullen und Kanten bis Rebuild“ aus 3.0.0 entfällt.|Nullvektoren zerstören die Navigation (D-77).|T-VEC-DEL-*|
|**D-19**|Vektor-Snapshots: je Slot ein Envelope unter dem DEK des Dokuments mit **Vektor und Slot-Bindung** `(cluster_tag, doc_tag, chunk)`; Graphkanten unter dem Dateischlüssel (D-85); ausschließlich über `Vfs`.|Embeddings gelten vorsorglich als personenbezogen (E-L3). Mit der Bindung unter dem DEK sind nach Schlüsselvernichtung weder Vektor noch Zuordnung lesbar.|T-RAW-3, T-KEYCOV-1|
|D-20|**Textindex: abgeleiteter, segmentierter, verschlüsselter Index** (9.7). Der LSM hält keine Postings.|KF-3.|T-TEXT-*, BM-ERASE-WA-TEXT|
|D-21|`tenant_tag`, `coll_tag`, `cluster_tag`, `doc_tag`, `term_tag`, `name_tag` sind keyed BLAKE3 mit festen Labels (Anhang C).|Index enthält keine Klartexte.|T-RAW-4|
|D-22|Entfernte Crates liegen im Branch `archive/v0-full`.|Historie auffindbar.|15.4 WP-1.2|
|D-23|Lizenz-/DRM-Schicht entfällt.|Geschäftsmodell liegt außerhalb des Speicherkerns.|T-ARCH-3|
|D-24|Ingest, Privacy, Agent, Cognition, Router, Adapt, KV-Cache, Inference, Sandbox, Audit-Export, AVV-Generator, `contextra-db`, Wasm-Merge-Operator sowie die Ports `embedding`, `reranker`, `query_rewriter` entfallen.|Nicht im Kern.|T-ARCH-3|
|D-25|MCP-Server und Python-Bindings sind kein v1-Umfang. Sie entstehen nach API-Freeze als Adapter.|Löschwerkzeuge dürfen erst nach der Saga exponiert werden.|–|
|D-26|`rank` enthält nur RRF-Fusion.|Hybridsuche ist Kern, der Rest nicht.|T-RANK-1|
|D-27|Keine automatische Entitäts- oder Metadatenextraktion. `AutoExtractionMode` entfällt vollständig.|F-05. Erzeugt Parallelbestände und PII-Abfluss.|T-ARCH-3|
|D-28|SIMD: u8-VNNI-Kernel entfällt; f32-Kernel plus Skalar-Referenz. Jeder Kernel hat Cross-Check.|Falsche Operandentypen bei `_mm512_dpbusd_epi32` (u8×u8 statt u8×i8).|T-SIMD-1|
|D-29|`unsafe` nur in `contextra-sys` und `contextra-simd`. Safe-Funktionen mit offenen Vorbedingungen werden `unsafe fn` oder entfernt. **Windows-Code (`acl_win32.rs`, `cfg(windows)`) wird entfernt (D-84).**|Kleinste Auditfläche (F-18, F-22).|T-UNSAFE-1|
|**D-30**|`erase` kehrt nach persistiertem Gate und vernichtetem Schlüssel zurück (Status `Gated`). Ziel: **p50 ≤ 10 ms auf der NVMe-Referenzplattform** (14.8) und allgemein **≤ 6 × `fsync_p50` + 2 ms** der Startprobe; p99 ≤ 5 × p50. **Gilt unabhängig von der Zielgröße** (P16, BM-ERASE-O1). `erase_sla` ist Pflichtparameter ohne Standardwert.|Die Latenz wird durch die Zahl der `fsync` bestimmt (4 im Normalpfad, D-58), nicht durch CPU und nicht durch die Zahl der Dokumente (Schlüsselhierarchie, D-79).|T-SLA-1, BM-ERASE-LAT, BM-ERASE-O1|
|**D-31**|**Löschcluster-Schlüsselraum:** `region ‖ tenant_tag ‖ coll_tag ‖ cluster_tag ‖ doc_tag ‖ family ‖ sub` (9.1). Jedes Löschziel ist ein zusammenhängender Bereich. **Die Region `LOC` entfällt** (D-70): jedes Dokument ist aus `DocRef` direkt adressierbar.|P11, Rechnung 11.9.|BM-ERASE-WA|
|**D-32**|**Record-Envelope** (Version 2, Anhang F.6): Jeder Wert liegt unter dem DEK seines Dokuments; zusätzlich Block-AEAD unter dem Dateischlüssel. Der Record-Layer ist für die Löschbarkeit nötig; der Block-Layer schützt Struktur und Tags gegen Angreifer ohne `K_master` und macht Dateiblöcke nach D-85 wertlos.|Entkoppelt Schlüsselvernichtung von der Blockstruktur. Der Block-Layer bleibt AEAD statt reiner Authentisierung, weil Tags und Clustergrößen sonst im Klartext auf dem Medium liegen; Kosten werden in BM-WRITE gemessen.|T-ENC-*, T-VAULT-*|
|**D-33**|**Tresor mit Schlüsselhierarchie** (D-79, 11.5): Slots der Typen `T`, `C`, `S`, `D`, `F`, je 32-Byte-Zufallssalt bzw. maskierter Dateischlüssel; `DEK` wird aus allen Salts der Ahnenkette abgeleitet. Löschen = Slot nullen, `sync_data`, Fehlen prüfen.|Kleinster löschbarer Anker, O(1) je Ziel.|T-VAULT-1..4|
|D-34|`contextra-erasure` hängt nur an `types`, `ports`, `crypto`. Schichten implementieren `ErasureLayer`.|P12.|T-ARCH-3|
|**D-35**|**Messung MK1/MK2/MK3** (12.1) mit unabhängigem Lab; Produktion misst MK1/MK2 und die Schlüsselabwesenheit (MK3-Feld) selbst. Ein Rohscan auf Klartext allein ist keine Messung.|In einem verschlüsselten Store ist ein Rohscan auf Tag-Bytes für Dateischichten systematisch leer; er schützt dort nur gegen Klartext-Lecks (12.1).|T-RAW-2|
|**D-36**|**Klassenverband G, K, P** statt linearer Skala (12.5); Klasse K mit `key_coverage` je Schicht.|Die Aussagen sind unabhängig, K ist schichtabhängig.|T-CLASS-1, T-KEYCOV-1|
|**D-37**|**Restore-Guard** über Ledger mit Bereichen und `gated_seq` (11.8, D-74).|Restore aus Backup kann gelöschte Daten zurückbringen.|T-RESTORE-1..3|
|D-38|`EraseTarget::Tenant` (Offboarding); Schlüsselklasse O(1) über den `T`-Slot.|Häufigster realer Löschfall beim Betreiber.|T-TENANT-1|
|**D-39**|**Purge-Fenster:** Gate und Schlüsselvernichtung sofort, physischer Purge gebündelt bis höchstens `batch_window` (Standard 60 s, **≤ `erase_sla / 2`**) bzw. spätestens `erase_sla`; `Timing::Now` erzwingt Sofortlauf.|Bündelung senkt die Zahl berührter Dateien (11.9); die Randbedingung lässt dem Purge Zeit.|BM-ERASE-BATCH|
|**D-40**|Der Betroffene ist **Teil der Dokumentidentität** und damit unveränderlich (D-70). Ein Wechsel ist ein neues Dokument (`insert` mit anderem `DocRef`, danach `erase` des alten). `Conflict::SubjectImmutable` entfällt als Fehlerfall.|Sonst ist der Cluster nicht eindeutig.|T-SUBJ-2|
|**D-41**|`scan` iteriert in `(cluster_tag, doc_tag)`-Reihenfolge (Region `DATA`); `ScanRange::Subject` ist ein einzelner Bereich.|Folge von D-09, D-31 und D-70.|T-SCAN-1, T-SCAN-2|
|D-42|Jeder Header (WAL, SSTable, Manifest, Tresor, Journal, Ledger, Gate-Log, Snapshot, Textsegment, Bericht, HEADER) trägt `format_version` und Algorithmus-IDs (Anhang F.1).|Krypto-Obsoleszenz planbar (EDPB 02/2025 Rn. 85, 96 [D]).|T-FMT-1|
|D-43|Kein Netzwerk, keine Telemetrie, kein Escrow im Produktionsbuild.|HR-1..3.|T-ARCH-4|
|D-44|Rot-vor-Grün; Evidenzbündel je Arbeitspaket.|Verifizierbar.|15.4|
|D-45|`contextra-db` entfällt; `homeostat`, `multistep`, `volatile_vault` sind nicht im Kern.|Features leiten an entfernte Crates.|T-ARCH-3|
|D-46|Tests dürfen den Master-Schlüssel direkt injizieren (`EngineBuilder::master_key_for_testing`) unter Feature `testing`, das **nur** den Argon2-Schritt ersetzt, nie die Verschlüsselung. CI prüft, dass kein Release-Build `testing` aktiviert.|Schnelle Tests ohne Sicherheitsschalter.|T-CFG-2|
|D-47|**entfällt** (Locator-Region; ersetzt durch D-70).|Anhang H.|–|
|**D-48**|**Commitment-Epoche:** `commit_epoch` (Standard 365 d) rotiert den Schlüssel der **Commitments in Journal und Bericht**. Die **Ledger-Lebensdauer ist davon entkoppelt und gleich `backup_retention`** (D-74).|Die Wirksamkeit des Restore-Guards hängt an der Backup-Aufbewahrung, die Datensparsamkeit der Commitments an der Epoche; beide Fristen sind getrennt begründbar (E-L4).|T-JRN-3, T-RESTORE-3|
|**D-49**|Textsegmente: **ein Dokument liegt zu jedem Zeitpunkt in genau einem Segment oder im Textpuffer.** Ein Betroffener kann in vielen Segmenten liegen; der Rewrite-Aufwand skaliert mit der Zahl dieser Segmente (D-71).|Ehrliche Fassung von P11 für den Textindex.|BM-ERASE-WA-TEXT|
|D-50|Ein Datenverzeichnis hat genau einen schreibenden Prozess (exklusiver Verzeichnis-Lock über `Vfs`).|Journal- und Manifest-Konsistenz.|T-LOCK-1|

### 4.2 Produktionsreife, Enterprise, Lücken der Vorfassungen

|ID|Entscheidung|Begründung|Test / Beleg|
|---|---|---|---|
|**D-51**|`Collection::soft_delete(DocRef)` (logisch, Tombstone). Es gibt **kein** `delete`. Rustdoc und Dokumentation sagen an erster Stelle: „Für Löschpflichten `erase` verwenden.“ Nach `history_horizon` entfernt die Kompaktion Versionen physisch; das ist keine Löschaussage.|KF-6.|T-API-3 (Doc-Test), T-LINT-1|
|**D-52**|**Keyslots:** `K_master` ist ein zufälliger 32-Byte-Schlüssel (OS-CSPRNG), im `HEADER` je Slot gewickelt (AES-256-GCM-SIV) unter `KEK = Argon2id(passphrase, slot_salt, params)`. 1..4 Slots. **Die AAD des Wraps bindet `store_id`, `slot_id`, `anchor_id`, `kdf_id`, `m`, `t`, `p` und `slot_salt`** (Anhang F.2). `change_passphrase` fügt einen Slot hinzu, prüft ihn, entfernt den alten (Nullen, `fsync`) in O(1). `HEADER` als A/B-Paar mit Generationszähler. Master-Rotation bleibt v1.1.|KF-5. Ohne AAD-Bindung lässt sich ein Slot zwischen Stores kopieren oder seine KDF-Parameter herabsetzen. Restrisiko `OldKeyslotRemanence`.|T-KEYSLOT-1..4, T-HDR-1|
|**D-53**|Der Schlüsselanker ist **austauschbar**: Port `RecordSealer` (DEK-Operationen) und Port `FileKeyring` (Dateischlüssel, D-85) haben in v1 die gemeinsame Implementierung `FileVault`; jeder Keyslot trägt eine `anchor_id`. Hardware-Anker (TPM 2.0, PKCS#11/HSM, KMS-Adapter außerhalb des Kerns) sind v1.1 ohne Formatbruch.|Löst `WrappedKeyRemanence` bei Bedarf, ohne v1 an Hardware zu binden.|Conformance-Suite `RecordSealer`/`FileKeyring`|
|D-54|`tokio`-Features im Workspace: `rt-multi-thread`, `sync`, `time`, `macros`. **Kein** `full`, `net`, `fs`, `process`, `signal`, `io-std`. `deny.toml` verbietet `mio`, `socket2`, `reqwest`, `hyper`, `ureq`, `curl`, `isahc`, `surf`, `attohttpc`, `tonic`, `h2` im Produktionsbaum (`cargo tree -e normal`). Dev-Abhängigkeiten (`criterion`) sind ausgenommen.|F-16. HR-1 ist sonst nicht nachweisbar.|T-ARCH-4|
|D-55|**`fsync`-Fehler sind für die betroffene Datei fatal.** Nach einem Fehler von `sync_data`/`sync_all`/`sync_dir` wird die Datei nie auf demselben Handle wiederholt; der Store wechselt in `Poisoned`, alle mutierenden Operationen liefern `Io`, `health()` meldet es, Recovery beim nächsten Öffnen. Löschpfad: `Pending` mit Neuöffnen der Datei und Prüfung.|Nach einem fehlgeschlagenen `fsync` ist der Zustand der Seiten im Page-Cache undefiniert („fsyncgate“ [D]). Blindes Wiederholen meldet fälschlich Dauerhaftigkeit – für ein Löschprodukt ein Haftungsfall.|T-FSYNC-1|
|**D-56**|**Plattformstufen:** Tier 1 = Linux (x86_64, aarch64) auf ext4 oder XFS. Tier 2 = macOS (APFS, `F_FULLFSYNC`) und Linux auf btrfs/ZFS (COW → Restrisiko `FsJournalOrCow` immer). **Nicht unterstützt:** Windows (Code entfernt, D-84), NFS, SMB, FUSE, overlayfs und unbekannte Dateisysteme: die Startprobe (`statfs`-Typ plus atomares Rename plus Verzeichnis-`fsync` plus Atomaritätsprobe für 64-Byte-Schreibvorgänge an ausgerichteten Offsets) verweigert den Start (`Config::UnsupportedFilesystem`).|Verzeichnis-`fsync` und atomares Rename sind die Basis der Crash-Konsistenz; Netz- und Overlay-Dateisysteme geben diese Zusagen nicht.|T-PLAT-1, FI-F03|
|**D-57**|**Gate-Commit-Protokoll:** `ErasedSet::add` nimmt die **exklusive Commit-Barriere** des Stores, persistiert das Gate und gibt frei. Jeder Commit prüft in der Gruppen-Commit-Leader-Phase `ReadGate` für alle Schlüssel seines Batches und weist Schreibzugriffe in gegatete Bereiche mit `Erased` ab. Dokumentlocks gelten nur für `EraseTarget::Document`; für Subject/Collection/Tenant gibt es **kein** Lock-Set. **Gleichzeitige `erase`-Anfragen teilen sich eine Barriere-Phase und einen Gate-Log-`fsync` (Gate-Gruppen-Commit, D-78).**|Locks sind bei 10⁶ Dokumenten nicht haltbar. Die Barriere ist O(1) und beweisbar: alles vor dem Gate ist im Purge-Umfang (`gated_seq`), alles danach wird abgewiesen.|T-GATE-1, T-CONC-1 (loom), T-GATEGROUP-1|
|D-58|Gate-Menge als **append-only Log** (`erasure/gates.log`, AEAD, Hash-Kette, `fsync` je Gruppe), Kompaktion beim `Completed` der Löschung. Normalpfad `Gated`: Journal `Requested+Intent` (1), Gate-Log (1), Tresor-Nullen (1), Journal `KeyDestroyed` (1) = **4 `fsync`** je Anfrage, bei Last weniger je Anfrage (Gruppen).|Atomarer Dateiaustausch je Gate kostet 3+ `fsync`; ein Append kostet einen.|BM-ERASE-LAT|
|D-59|Release-Regel und Cut-Linie (1.3).|Kein Release mit halber Löschschicht.|G-H|
|**D-60**|**Dateischlüssel und Nonces.** Datendateien (WAL, SSTable, Textsegment, Vektor-Snapshot, `scope/`) haben einen **zufälligen** Dateischlüssel (D-85); Steuerdateien (Manifest-Log, Journal, Gate-Log, Ledger) den abgeleiteten `K_file = HKDF-SHA256(K_enc, "ctx/file/v1" ‖ kind ‖ file_id)`. **Nonce:** in schreibeinmaligen oder append-only Dateien `0x00000000 ‖ u64(Blockzähler)` (deterministisch, unter dem Dateischlüssel eindeutig); je Record-Envelope und je Keyslot-Wrap 96 Bit aus dem OS-CSPRNG. Höchstens 2³² Blöcke je Datei, dann neue Datei.|F-24: Der Code nutzt ein Zufallspräfix je `KeyManager` plus Zähler; das ist mit Dateischlüsseln überflüssig. Ein Zähler je Datei schließt Kollisionen aus, statt sie nur unwahrscheinlich zu machen. AES-GCM-SIV bleibt als Defense in Depth.|T-NONCE-1, T-ENC-3|
|D-61|**Snapshot-Widerruf ist mandantenbezogen:** `revoke_covering` widerruft nur Snapshots des Mandanten des Ziels (Registry nach `tenant_tag`), die `seq ≤ gated_seq` halten. Snapshots anderer Mandanten bleiben unberührt.|Isolation gilt auch für Verfügbarkeit; ein Offboarding darf fremde Leser nicht stören.|T-SNAP-1|
|**D-62**|Suche: `ReadGate`-Filter im Kandidatenstrom. **Nachladen bis `k` gültige Treffer oder Erschöpfung:** Text setzt den Postings-Strom fort; Vektor verdoppelt `ef` ab `max(ef, k)` bis `ef_max = 4096`. Das Ergebnis `SearchPage { hits, exhausted }` trägt `exhausted = true`, wenn weniger als `k` Treffer lieferbar waren. Nie stilles Verkürzen. Metadatenfilter bleiben in v1 **nachgelagert** (dokumentierte Grenze; Vorfilter-Bitmap v1.1, `DocMap` bleibt dafür erweiterbar).|Sonst verdrängen gegatete Treffer gültige aus dem Top-k; ohne Algorithmus ist „kein stilles Verkürzen“ nicht testbar.|T-HYB-1, T-SEARCH-2|
|**D-63**|**Prozesshärtung** (`contextra-sys`) als Builder-Option `process_hardening ∈ {Required, BestEffort, Off}` (Standard `BestEffort`): `RLIMIT_CORE = 0`, `PR_SET_DUMPABLE = 0` (Linux), `MADV_DONTDUMP` für Schlüsselseiten und Vektorarena, `mlock` best effort. `Required` verweigert den Start, wenn eine Maßnahme scheitert. Der Status steht in `health()`; bei `Off` oder Teilerfolg führt der Bericht `ProcessMemory`. Die Doku nennt die Nebenwirkung: `RLIMIT_CORE` und `PR_SET_DUMPABLE` sind prozessglobal (Debugger, `perf`, Crash-Reporter).|Eine eingebettete Bibliothek darf Prozesszustand nicht still ändern; Coredumps und Swap sind die realistischen Wege für Klartext-Remanenz im Prozess.|T-DUMP-1, T-SEC-2|
|**D-64**|**Beobachtbarkeit:** `Metrics`-Port (statische Namen, keine ID-Labels) und `Contextra::health() -> HealthReport` (Zustand `Ok/Degraded/Poisoned/Recovering`, `fsync_p50`, offene Löschungen, Alter der ältesten, SLA-Verletzungen, Tombstone-Quote, freier Platz, maximale Wartezeit auf die Purge-Barrier, Prozesshärtungsstatus). Kein Exporter, kein Server im Kern.|P15.|T-OPS-1|
|D-65|**Plattenreserve:** Schreibzugriffe werden bei freiem Platz unter `max(1 GiB, 2 × größte Kandidatendatei)` mit `Limit::DiskReserve` abgewiesen; `erase` bis `Gated` braucht keinen nennenswerten Platz und läuft immer. Der Purge-Rewrite reserviert vorab.|Eine volle Platte darf weder Löschung noch Gate verhindern.|T-CRASH-3, T-OPS-2|
|D-66|**API-Stabilität:** SemVer, alle öffentlichen Typen `#[non_exhaustive]` (außer Konfigurationsbausteinen), einzige öffentliche Oberfläche ist `contextra`. **MSRV (Kompatibilitätsversprechen) und Build-Toolchain sind getrennt (D-82).** Format-Version und Crate-Version sind getrennt.|Enterprise-Verlässlichkeit.|T-API-1, `cargo-semver-checks`|
|D-67|**Support und Schwachstellenmanagement:** Unterstützungszeitraum je Hauptversion ≥ 5 Jahre ab Veröffentlichung; Meldeprozess nach CRA Art. 14 (Frühwarnung 24 h, Meldung 72 h, Abschlussbericht gemäß Verordnung); intern HR-8 (kritisch 7 Tage, hoch 30 Tage). Contextra behandelt sich als Hersteller im Sinne des CRA und des ProdHaftG (E-L9).|Strengste Annahme (6.6). [W] CRA-Meldepflichten gelten seit 11.09.2026.|T-SUPPLY-1|

### 4.3 Neu in 4.0.0 (Schließen der Lücken aus der Prüfung von 3.0.0)

|ID|Entscheidung|Begründung|Test / Beleg|
|---|---|---|---|
|**D-69**|**Klasse K je Schicht.** `key_coverage ∈ {full, unlinked, none}` je Schicht (`doc_store`, `vector_payload`, `text_postings`, `vector_edges`, `file_structure`): **`full`** = Inhalt unlesbar; **`unlinked`** = Struktur vorhanden, aber ohne Bindung an Dokument und Betroffenen; **`none`** = nicht durch Schlüsselvernichtung geschützt. **Textindex:** Postings tragen nur segmentlokale `doc_ord`; die Bindung `doc_ord → (cluster_tag, doc_tag)` liegt als Envelope unter dem DEK (9.7). Nach `KeyDestroyed` sind Textpostings daher `unlinked`, HNSW-Kanten `none`. Der **Textsegment-Rewrite läuft im ersten Purge-Fenster**, nicht erst bis `erase_sla`. Restrisiko `TextIndexUntilPurge` gilt bis dahin.|3.0.0 legte Postings unter einen nie vernichteten Schlüssel: Wer `K_idx` hat, rekonstruiert per Wörterbuchangriff den Wortschatz eines gelöschten Dokuments (inhaltlicher Fingerabdruck). Die entkoppelte Bindung nimmt dem Rest die Zuordenbarkeit, ohne die Suchlatenz zu belasten (Bindungen werden nur für Top-k-Kandidaten entschlüsselt).|T-KEYCOV-1, T-TEXT-4|
|**D-70**|**`DocRef`:** Dokumente werden mit `DocRef { subject: Option<SubjectRef>, key: DocKey }` adressiert; der Betroffene ist Teil der Identität (D-08, D-40). `cluster_tag = subject_tag`, ohne Betroffenen `0x00¹² ‖ doc_tag[0..4]`. **Die Locator-Region und die Familie `Locator` entfallen.** Suchtreffer tragen `DocHandle { cluster_tag, doc_tag }`.|3.0.0 D-47 legte je Dokument einen Locator unter `LOC ‖ … ‖ doc_tag` ab. `doc_tag` ist ein keyed Hash, die Locator-Schlüssel eines Betroffenen sind also **gleichverteilt**: Ein Subject-Erase schriebe min(*m*, #LOC-Dateien) SSTables um (das „26 %-Problem“ aus 11.9) und verletzte P11 an der zentralen Stelle.|T-DOCREF-1, T-SUBJ-1, BM-ERASE-WA|
|**D-71**|**Textsegmente (ehrliches P11):** (a) Segment-Header trägt `bucket` und `bucket_count` (v1: `bucket_count = 1`; Formatbruch später ausgeschlossen); Bucket-Funktion `cluster_tag[0] mod bucket_count`. (b) Segmentzielgröße adaptiv: 8 MiB bis 1 Mio. Chunks je Collection, danach 32 MiB. (c) Rewrite-Aufwand eines Subject-Erase = Σ Größe der Segmente, die Zieldokumente enthalten (im Bericht: `segments_rewritten`). (d) Die Entscheidung über `bucket_count > 1` und die Segmentgrößen trifft Gate G-E anhand **BM-ERASE-WA-TEXT** und BM-TEXT, nicht Annahme.|3.0.0 D-49 sagt „Rewrite je Erase = ein Segment“; das gilt nur für ein Dokument. Die Dokumente eines Betroffenen entstanden über Jahre und liegen in vielen Segmenten.|BM-ERASE-WA-TEXT, T-TEXT-*|
|**D-72**|**Messbegriffe und Feldprüfung.** Evidenzstufen heißen **EV0–EV4**, Messklassen **MK1–MK3** (0). Der Bericht trennt `lab_evidence` (Verweis auf den Hash des Release-Evidenzbündels; MK1–MK3 im CI-Lab, EV4) und `field_measurement` (Zeugen dieses Laufs: MK1, MK2 und Schlüsselabwesenheit MK3-Feld, durch Produktionscode). Anhang B L4 bezieht sich auf `field_measurement`. **`ctx-verify scan <datadir> --passphrase-fd N --target …`** liefert die unabhängige Feldprüfung: eigene Parser und Krypto-Aufrufe (AR-4), Tag-Berechnung aus `K_master`, MK1 über alle Strukturen, MK2 mit Klartext-Needles des Betreibers, Positivkontrolle. **Die Positivkontrolle ist typisiert:** `LayerWitness` kann nur aus einem `Scanner<Armed>` entstehen, den allein eine erfolgreiche Kontrolle erzeugt.|3.0.0 verwendete „E1–E3“ (Messklassen) und „E0–E4“ (Evidenzstufen) parallel und versprach in Produktion MK3 mit dem gelöschten DEK, den es dort nicht mehr gibt. Die Produktions-Eigenmessung ist Selbstattestierung (gleiche Decoder wie der Purge); unabhängig ist sie nur im Lab oder mit `ctx-verify scan`.|T-LINT-3, T-RAW-1..6, T-CLASS-1, T-MUT-1|
|**D-73**|**Der WAL wird nie umgeschrieben.** Purge-Schritt: Memtable-Einträge des Ziels verwerfen, **Memtable flushen**, WAL-Segment an einer Sequenzgrenze versiegeln, **alle WAL-Segmente bis zum Flush-Punkt als Ganzes entfernen** (Dateischlüssel vernichten, unlink). Der Manifest-Eintrag `WalFloor { seq, tail_mac }` hält die Kettenfortsetzung (Anhang F.4). I-3 gilt auf **Record-Ebene** (Ciphertext-Records anderer Ziele bleiben bytegleich), nicht auf Dateiebene.|3.0.0 11.2 Schritt 6 verlangte, Zielframes aus WAL-Segmenten zu entfernen. Die HMAC-Kette verkettet jeden Frame mit dem Vorgänger; Entfernen ändert die MACs aller Folgeframes, verletzt „bitgleich“ und macht Torn-Write-Erkennung im Crash unscharf.|T-ISO-1, T-WAL-2, BM-ERASE-BATCH|
|**D-74**|**Restore-Guard mit Bereichen.** Pflichtparameter `backup_retention` (Dauer, ohne Standard; Zweckbindung „Restore-Schutz“, Begründung gehört in die DSFA). Ledger-Eintrag: `(commitment, range_set, gated_seq, completed_at)` mit Bereichs-**Tags**. Erkennung: (a) **Zeitregression** `store.current_seq < max(gated_seq im Ledger)` ⇒ Restore, ohne Scan; (b) **Wiederauftauchen** (Dateirestore): Datei im Zielbereich mit `min_seq ≤ gated_seq` und Record mit `seq ≤ gated_seq`. Daten mit `seq > gated_seq` sind legitime Neuanlage (`doc_key` und Betroffener dürfen nach `Completed` neu verwendet werden). Die Sequenz bleibt über Restores monoton: `next_seq = max(current_seq, ledger_floor) + 1`. `commit_seq` steht als Schlüsselsuffix (lesbar ohne DEK).|3.0.0 hatte als Horizont nur `commit_epoch` (365 d) und speicherte `scope_range_hash`: Backups in Steuer-/Finanz-/Gesundheitsbranchen werden länger aufbewahrt (Scheinschutz), und aus einem Hash lässt sich kein Bereich prüfen (Vollscan). Ohne `gated_seq`-Regel wäre Wiederverwendung ein Fehlalarm.|T-RESTORE-1..3, BM-RESTORE-OPEN|
|**D-75**|**Pfade nur mit Tags:** `text/<tenant_tag_hex>/<coll_tag_hex>/<file_id_hex>.seg`, `vector/<tenant_tag_hex>/<coll_tag_hex>/…`; Dateinamen sonst nur `file_id` (hex). Mandanten-Offboarding löscht die Verzeichnisse `…/<tenant_tag>/` plus Tresor-/LSM-Bereiche.|3.0.0 5.6 nutzte `<coll>` (Klartextname, mögliche Personenbezüge, D-09/HR-4) und hatte keine Mandantendimension (T-ISO-1/T-TENANT-1 unmöglich).|T-RAW-6, T-TENANT-1|
|**D-76**|**G0 messbar und mit absoluter Grenze.** SSTable-Zielgröße Level ≥ 1: 32 MiB. Schwelle je Erase: `bytes_written ≤ 3 × Zielbytes + L_t × 32 MiB` mit `L_t` = Zahl der Level mit überlappender Datei (≤ 7), **absolute Obergrenze 512 MiB je isoliertem Einzeldokument-Erase**, Steigung 10 GiB/1 GiB ≤ 1,25. Fallbackleiter in 11.9.|3.0.0 verlangte „≤ 3 × Zielbytes + 2 SSTable-Zielgrößen“ bei gleichzeitig „3–6 Dateien praktisch“ (widersprüchlich) und hatte keine absolute Grenze: bei 64 MiB × 7 Level × 2 Dateien wären ≈ 0,9 GiB je Erase möglich (SSD-Verschleiß).|BM-ERASE-WA, G0|
|**D-77**|**HNSW-Löschung.** (1) **Vor `Gated`:** Arena-Vektoren der Zielslots werden in allen Repräsentationen durch ein **Navigationssurrogat** ersetzt (Mittel der Vektoren der nicht gegateten Nachbarn; ohne solche: Vektor des Einstiegsknotens). Kein Zielvektor-Byte bleibt in der Arena. (2) **Im Purge-Fenster:** **Batch-Kantenreparatur**: ein linearer Durchlauf über alle Adjazenzlisten entfernt Kanten auf Zielslots und wählt Ersatzkanten mit der Standard-Auswahlheuristik aus den Nachbarn der Zielknoten; danach existiert keine Referenz auf Zielslots, die Slots gehen in die Freiliste. (3) **Deterministischer Rebuild** (Einfügung in `doc_tag`-Reihenfolge, deterministische Konfliktauflösung) bei Tombstone-Quote ≥ `vector_rebuild_ratio` (0,2) oder ≥ 10 % reparaturgelöschter Slots seit dem letzten Rebuild; mit `vector_strict_rebuild` (Standard aus) im selben Purge-Fenster, spätestens zum Ende von `erase_sla`. **Eigenschaft:** Der Graph nach Rebuild hängt nur von der Menge der lebenden Slots ab (T-VEC-REBUILD-INDEP). Bis zum nächsten Rebuild nach einer Löschung weist der Bericht `IndexTopologyInference` aus (Reparaturkanten sind eine Funktion der Zielnachbarschaft).|Nullvektoren zerstören die Navigation: Distanzen zu einem Nullvektor sind bedeutungslos, hnswlib-artige Implementierungen halten gelöschte Knoten **mit** Vektor im Graphen. Bei Tombstone-Quoten bis 0,2 fiele die Erreichbarkeit ganzer Regionen (Recall@10 ≥ 0,95 ungesichert). Ein Vollrebuild je `erase_sla`-Fenster wäre bei kontinuierlichem Erase-Strom Dauerlast; die Reparatur ist ein einziger speichergebundener Durchlauf.|T-VEC-DEL-1/2, T-VEC-REBUILD-INDEP, BM-VEC-DEL, BM-VEC-REBUILD|
|**D-78**|**Gate-Gruppen-Commit und kooperative Kompaktion.** Gleichzeitige `erase`-Anfragen teilen sich *eine* exklusive Barriere-Phase und *einen* Gate-Log-`fsync`. Kompaktion prüft pro Block `purge_waiting` und **bricht kooperativ ab** (Teilergebnis wird verworfen, nicht committet); keine Hintergrundoperation blockiert eine wartende Löschung länger als 1 s (I-16). Kompaktion **verwirft gegatete Bereiche der Variante `Erased`** beim Schreiben der Ausgabe (kostenloser Teilpurge; Zeuge MK1 bleibt Pflicht im Purge). **Bereiche der Variante `Restricted` verwirft sie nie** (I-18).|Die exklusive Barriere hält sonst bei einem Erase-Strom jeden Schreiber an; eine mehrminütige Kompaktion blockiert den Purge, während die `erase_sla`-Uhr läuft (Prioritätsinversion). Restriktion (Art. 18) ist keine Löschung: Verwerfen wäre Datenverlust.|T-GATEGROUP-1, T-COMPACT-RESTRICT-1, BM-WRITE-DURING-ERASE, BM-PURGE-WAIT|
|**D-79**|**Schlüsselhierarchie und Tresor.** `DEK = HKDF-SHA256(ikm = K_unit, salt = SHA-256("ctx/dek-salt/v2" ‖ salt_T ‖ salt_C ‖ salt_S ‖ salt_D), info = "ctx/dek/v2" ‖ doc_tag)`. Slot-Typen `T` (tag = tenant_tag), `C` (coll_tag), `S` (subject_tag), `D` (doc_tag), je 32-Byte-Zufallssalt; Dokumente ohne Betroffenen nutzen `salt_S = 0³²`. Ziel → vernichteter Slot: `Document → D`, `Subject → S`, `Collection → C`, `Tenant → T`. **Die Vernichtung eines Slots macht alle darunter liegenden DEKs unableitbar; die Schlüsselklasse ist für jedes Ziel O(1)** (4 `fsync`, P16). Der DEK-Cache ist nach `(tenant, coll, cluster)` indiziert; der Gate-Schritt evictet und zeroisiert betroffene Einträge **vor** der Rückgabe `Gated`. Die `D`-Slots der Zielmenge entfernt der Purge (zielgrößenabhängig, nicht auf dem Pfad bis `Gated`). Der Tresor ist eine **plattenresidente Open-Addressing-Hash-Tabelle** (11.5, F.3) ohne vollständigen RAM-Index; Obergrenze `max_docs_per_store = 5·10⁷` (`Limit::Capacity`).|3.0.0 hatte nur Dokument-Slots: Ein Betroffener mit 10⁶ Dokumenten bedeutete 10⁶ Slots nullen, Mandanten-Offboarding ebenso, während das Gate selbst O(1) ist. Der RAM-Index von 80 B × N hätte BM-OPEN ≤ 2 s bei großen Beständen gesprengt.|T-VAULT-1..5, BM-ERASE-O1, BM-VAULT-OPEN|
|**D-80**|**Kryptografie-Details.** (1) Argon2-Grenzen: D-06. (2) Keyslot-AAD: D-52. (3) **`K_sign` nur abgeleitet** (`HKDF(K_master, "ctx/sign/v1")`); das Verzeichnis `keys/` und der „verschlüsselte Seed“ entfallen. (4) **Algorithmensatz v1:** AES-256-GCM-SIV (Records, Keyslots, Block-AEAD; `alg_id = 1`), HMAC-SHA256, HKDF-SHA256, BLAKE3 (keyed), Argon2id, Ed25519. `alg_id = 2` (ChaCha20-Poly1305) ist reserviert und in v1 nicht implementiert. (5) Block-AEAD bleibt (D-32).|F-24: nur GCM-SIV existiert im Code; 3.0.0 8.6 nannte nicht vorhandene Algorithmen. 3.0.0 leitete `K_sign` ab (10.1) **und** legte einen Seed ab (5.6): eine doppelte Quelle erzeugt Abweichungsfälle.|T-KDF-2, T-KEYSLOT-4, T-ENC-*|
|**D-81**|**Textanalyse.** Segment-Header trägt `analyzer_id`, `analyzer_version`, `lang`. Analyzer v1: `de/1` (UAX-#29-Tokenisierung, NFKC, Case-Folding, `ß→ss`, Snowball-German, Stoppwortliste, wörterbuchbasierter Dekompounder) und `und/1` (neutral). Der Analyzer ist je Collection unveränderlich; ein Wechsel ist Neuaufbau in einer neuen Collection. `term_tag = tag("ctx/term/v1", analyzer_id ‖ lang ‖ normalisierter_term)`. Postings in 128er-Blöcken mit Blockmetadaten für Block-Max-Pruning (BMW), optional **Positionen** (Collection-Option, Standard an) für Phrasensuche.|3.0.0 legte `term_tag` und damit den Recall für deutschsprachige Korpora ohne Analyzer fest; ein Analyzerwechsel ist ein vollständiger Reindex und ohne `analyzer_id` im Format ein Formatbruch. Ohne Pruning ist BM-TEXT (p99 ≤ 30 ms bei 1 Mio. Chunks) für häufige Terme nicht erreichbar.|T-TEXT-ANALYZER-1 (Goldenset Komposita/Umlaute), T-TEXT-PHRASE-1, BM-TEXT|
|**D-82**|**Toolchain.** *MSRV* (`rust-version`) = älteste in der CI-Matrix geprüfte Version, nur in Minor-Releases anhebbar. *Build-Toolchain* (`rust-toolchain.toml`) = zum Release-Schnitt aktuelle Stable, **höchstens 2 Minor-Versionen hinter der aktuellen Stable**; Release-Builds mit `--locked`, Toolchain-Version und Build-Hash im Evidenzbündel.|Das Rust-Projekt pflegt nur die jeweils aktuelle Stable mit Sicherheitskorrekturen [W]. Der Pin `1.89.0` (August 2025) ist seit dem Erscheinen von 1.90 ohne Pflege; zum 2026-06-30 war 1.96.1 aktuell [W]. Ein eingefrorener Compiler samt Standardbibliothek ist für ein CRA-pflichtiges Produkt (E-L9) ein Lieferkettenrisiko.|T-SUPPLY-2|
|**D-83**|**Betriebsdetails.** (1) `scrub` (Überschreiben vor `unlink`) nur, wenn die Startprobe `rotational = true` und nicht-COW meldet; sonst entfällt es (nicht tragend nach D-85); Kosten getrennt in BM-ERASE-WA. (2) **Verwaiste Tresor-Slots** (`D`/`F` ohne Record bzw. Datei, etwa nach Absturz zwischen Slot-Schreiben und WAL-`fsync`) werden bei der Recovery genullt. (3) `batch_window ≤ erase_sla / 2` ist Konfigurationsprüfung; `health()` warnt, wenn die gemessene Purge-Dauer `erase_sla − batch_window` übersteigt.|Entkoppelt Wirkung vom Medium; schließt Waisen; verhindert Konfigurationen, die die SLA unerreichbar machen.|T-CRASH-5, T-CFG-3, BM-ERASE-WA|
|**D-84**|**Verbindliche Rückbauliste (Gate G-B):** Features und Parameter `legacy-wal-key`, `allow_legacy_integrity_key_fallback`, `memory-only-storage`, `cloud-egress-guard`, `kv-encryption`, `experimental-diskann`, `experimental-rabitq`, `experimental-predicate-augmented-search`, `partial-index-rebuild`, `graph`, `fault-injection` (ersetzt durch `FaultVfs` im `testkit`), `block-cache-v2`/`sieve-cache` (eine Cache-Implementierung bleibt), `auto-extraction-opt-out`, `entity-extraction`, `edge-reinforcement-learning`, `reranking`, `adaptive-candidate-pool-sizing`, `onnx`, `sandbox`, `graph-connectivity-health`, `encryption-at-rest`, `docid-128`, `bm25f`, `background-maintenance`, `dibud`, `contract-tests` (nur Dev), Paketalias `contextra-index`; `WasmMergeOperator` samt `block_on`; `contextra-sys/src/acl_win32.rs` und alle `cfg(windows)`. `test-utils`/`testkit` bleiben nur als Dev-Features.|F-06, F-14, F-21, F-22, F-23: jedes dieser Elemente ist ein Schalter für Sicherheit, ein Parallelpfad oder eine tote Auditfläche (P3, P5, P8).|T-CFG-1, T-ARCH-3, T-UNSAFE-1|
|**D-85**|**Dateischlüssel-Vernichtung.** Jede Datendatei (WAL-Segment, SSTable, Textsegment, Vektor-Snapshot, `scope/`) erhält bei Anlage einen zufälligen 32-Byte-Dateischlüssel (OS-CSPRNG), abgelegt als Tresor-Slot Typ `F` (`tag = file_id`, Payload = Schlüssel ⊕ `HKDF-Expand(K_enc, "ctx/fkmask/v1" ‖ file_id, 32)`). Beim Ausscheiden der Datei (Manifest-Swap dauerhaft) wird der `F`-Slot genullt (`sync_data`) und **dann** die Datei entfernt. Ordnung: `F`-Slot dauerhaft **vor** dem ersten dauerhaften Block der Datei (I-9b); Nullung nach dauerhaftem Manifest-Swap, vor `unlink` (I-17). Recovery: Datei ohne Manifest-Eintrag → Slot nullen, entfernen; `F`-Slot ohne Datei → nullen. Beim SSTable-Rewrite werden Nicht-Ziel-Records als Ciphertext-Envelopes unverändert in die neue Datei übernommen und unter dem neuen Dateischlüssel blockverschlüsselt (I-3).|Freigegebene Dateiblöcke enthielten sonst Strukturmetadaten (pseudonyme Tags, Clustergrößen, Postings) unter einem aus `K_master` ableitbaren Schlüssel: für den Inhaber der Passphrase lesbar. Mit Dateischlüssel-Vernichtung sind sie auch für ihn wertlos; die Remanenzfläche schrumpft auf den genullten 64-Byte-Slot. Die Maßnahme wirkt auch dort, wo Überschreiben nicht verlässlich ist (Flash, COW).|T-FK-1..3, T-CRASH-6|
|**D-86**|**Vektor-Arena-Layout:** je Slot ein zusammenhängender, zeroisierbarer Bereich, der **alle** Repräsentationen des Vektors enthält (f32; später SQ8, PQ, binär). Gelöschte Slots werden in allen Repräsentationen überschrieben (D-77). Keine Repräsentation außerhalb der Arena (kein Quantisierungs-Nebenarray).|Spätere Quantisierung darf die Löschbarkeit nicht unterlaufen; später lässt sich das Layout nur mit Formatbruch ändern.|T-VEC-ARENA-1|
|**D-87**|**Kapazitätsgrenzen v1:** `max_docs_per_store = 5·10⁷` (Tresor, D-79), `max_doc_bytes` 4 MiB (≤ 64 MiB), Snapshots je Mandant ≤ 1024, Dimension ≤ 4096, Query-Terme ≤ 256, `k` ≤ 1000. Überschreitung: `Limit::*`, nie stilles Kürzen.|Dokumentierte Produktgrenzen statt stillem Versagen.|T-LIMIT-1, BM-VAULT-OPEN|

### 4.4 Neu in 4.1.0 (Architektur- und Strukturprüfung am HEAD `f8a3bbb24`)

Die Entscheidungen D-88 bis D-92 sind Revisionsentscheidungen dieser Fassung; sie lösen die Befunde F-26 bis F-31 und sind vom Eigentümer der Spezifikation zu bestätigen (Anhang H). Sie ändern kein Datenformat und keine Löschsemantik.

|ID|Entscheidung|Begründung|Test / Beleg|
|---|---|---|---|
|**D-88**|**Matrixkorrektur (5.3).** (1) `engine→sys` ist erlaubt, **ausschließlich** für die Prozesshärtung (D-63); alle anderen `sys`-Aufrufe laufen über `durable-fs` oder `store`. (2) `vector` hängt **weder an `sys` noch an `mvcc`**: `mlock`/`madvise` der Arena über einen Port aus `ports`, implementiert in `engine`; Sichtbarkeit und Snapshot-Widerruf (D-61) bleiben in `engine`/`erasure`, nie im Index. (3) `text→crypto` ist ausdrücklich erlaubt (verschlüsselte Segmente, D-85). (4) `contextra` (Ring 4) erhält eine Matrixzeile: `engine`, `durable-fs`, `types`. (5) `ports` re-exportiert `types` nicht per Glob (AR-10).|F-27. Eine Kante ohne Matrixeintrag ist nicht prüfbar (AR-1); `vector→mvcc` würde dem Index Snapshot-Typen geben, während `erasure` (AR-3) keine Ring-2-Crates kennen darf.|T-ARCH-3, T-ARCH-6|
|**D-89**|**Schnitt der Fassade.** Paketname `contextra` bleibt (D-66). Das Crate enthält **keine Logik** und nur drei Module: `api` (Handles `Contextra`, `Tenant`, `Collection` als Re-Exporte aus `engine`), `wiring` (`system_defaults()`: `RealVfs`, Systemuhr, OS-CSPRNG, Verdrahtung von `FileVault`/`FileKeyring`), `error`. `EngineBuilder` (5.7) liegt in `engine`; die Fassade stellt nur `Contextra::builder()` bereit. **Entfallen:** `query_rewriter`, `collection_profile`, `performance_profile`, `EncryptionMode` (samt `Unencrypted`), Lizenz- und Embedder-Bezüge, Re-Exporte von `DocId`, `chunker`, `TextEmbeddingEngine`, `DriftStatusProvider`, `rank`. Erste Zeile von `README.md` und `//!`: „Öffentliche API und Verdrahtung. Enthält keine Logik.“|F-26. Eine Fassade, die Profile, Rewriter und Builder-Politik enthält, ist eine zweite Engine.|T-API-1, T-ARCH-3, T-ARCH-7|
|**D-90**|**Namensregeln.** (1) Kein Modul, keine Datei und kein Crate heißt `misc`, `util`, `utils`, `common`, `helpers` oder endet auf `_impl`. (2) Kein Modul trägt den Namen seines Crates (`types::types` → `domain`, `error`, `erasure`, `tags`). (3) **Eine** Modulform: `foo.rs` neben `foo/`, kein `mod.rs`. (4) Keine Paketaliase (`contextra-index` entfällt). (5) `engine/src/contextra_impl` heißt `instance`; `wiring.rs` (27 Zeilen) geht in die Fassade (D-89). (6) Bestehende Verstöße sind per Ratchet nur sinkend (5.8).|F-30. Namen sind die erste Dokumentation; ein Name ohne Aussage kostet bei jedem Lesen.|T-ARCH-7|
|**D-91**|**Crate-Kopf und Ringangabe.** Jedes Crate hat in `lib.rs` einen `//!`-Kopf mit Zweck (ein Satz), Ring, erlaubten Abhängigkeiten und Verweis auf den SPEC-Abschnitt. Ringangabe und Abhängigkeiten werden von `tests/arch` gegen 5.1 und 5.3 geprüft, nicht von Hand gepflegt. Verboten in Quelltext und Doku: Zeitstempel („STAND“), Sitzungs- und Agentenkennungen, `ANCHOR[...]`-Marken, absolute Entwicklerpfade.|F-28, F-30. Handgepflegte Angaben veralten ohne Test.|T-ARCH-5, T-LINT-4|
|**D-92**|**Folgedokumente und Werkzeugverläufe.** Die in Anhang E genannten, nicht normativen Dokumente werden **gelöscht**, nicht gekürzt, sobald ihr Inhalt in dieser Datei steht. An ihre Stelle tritt **eine** `AGENTS.md` im Wurzelverzeichnis (höchstens eine Seite, verweist auf `docs/SPEC.md`); `crates/*/AGENTS.md` entfallen. Agenten- und Werkzeugverzeichnisse (`.agents/`, `.gemini/`, `.antigravity/`), `reports/` und `evidence/` gehören nicht in den Quellbaum; Evidenz liegt als Release-Artefakt (11.7, 17.1).|Anhang E nannte „angleichen“; Angleichen erzeugt weiter zwei Wahrheiten. KI-Werkzeuge lesen jede Datei im Baum.|T-TRACE-1, T-ARCH-8|

---

## 5. Gesamtarchitektur

### 5.1 Workspace (Soll)

14 Produktions-Crates, dazu vier Hilfs-Mitglieder. Die Spalte „Ist“ zeigt den am HEAD `25c8f17` geprüften Zustand [V], sofern nicht [M] vermerkt ist.

|Crate|Ring|Rolle|Ist am HEAD|
|---|---|---|---|
|`contextra-types`|0|Reine Typen (IDs, Tags, Schlüssel, Fehler, Löschtypen, typisierter Zeuge). Kein I/O, kein `async`.|vorhanden; enthält Altlasten (`DocId`, `saos`, `tombstone`, `AutoExtractionMode`); `erasure.rs` ist nicht das Soll|
|`contextra-ports`|0|Traits: `StorageEngine`, `TextIndex`, `VectorIndex` (statisch dispatcht, D-03), `Vfs`, `Clock`, `Rng`, `IdGen`, `ErasureLayer`, `ReadGate`, `RecordSealer`, `FileKeyring`, `LedgerSink`, `Metrics`.|vorhanden; Altports `graph`, `checkpoint`, `license`, `attention`, `plugin` bereits entfernt (`c2a864d`); `embedding`, `reranker`, `query_rewriter` noch zu entfernen; `Vfs` nur noch in `ports` (F-29); Glob-Re-Export `pub use contextra_types::*` zu entfernen (D-88); Löschports fehlen|
|`contextra-sys`|1|`unsafe`-Insel: `fsync`, `flush_dir`, `rename_durable`, `mlock`, Prozesshärtung, Dateirechte (POSIX).|vorhanden, 30/30 Lib-Tests [M]; Windows-Code (`acl_win32.rs`) ist zu entfernen (D-84)|
|`contextra-simd`|1|`unsafe`-Insel: f32-Distanzkernel mit Skalar-Referenz.|vorhanden, 13/13 Lib-Tests [M]; u8-VNNI-Pfad zu entfernen|
|`contextra-durable-fs`|1|`Vfs`-Implementierung (`RealVfs`), atomarer Austausch, optionales `scrub` (D-83), Startprobe.|vorhanden; Doppel-`Vfs` behoben (F-29); Startprobe D-56 und `scrub` D-83 nicht nachgewiesen|
|`contextra-crypto`|1|AES-256-GCM-SIV, Argon2id, HKDF, Keyslots, WAL-HMAC, Hash-Ketten, Ed25519, Tags.|vorhanden; 9 Test-/Bench-Ziele kompilieren nicht [M]; Untergrenze zu niedrig, keine Obergrenze (F-11)|
|`contextra-mvcc`|2|Snapshots, SSI, Sequenzen.|vorhanden, 47/47 Lib-Tests [M]; Sentinel-Fehler (F-07)|
|`contextra-store`|2|LSM-Tree (WAL, Memtable, SSTable, Manifest, Kompaktion, Purge-Primitive).|vorhanden (26.996 Zeilen, 124 Testdateien [M]); F-06, F-07, F-09, F-13, F-21|
|`contextra-text`|2|BM25-Segmentindex (9.7).|vorhanden als termgeordneter LSM-Index (Module `bm25`, `inverted`, `morphology`, `tokenizer`, `wand`); Umbau nötig (D-20, D-81)|
|`contextra-vector`|2|HNSW, Arena, Snapshot.|vorhanden (13.624 Zeilen [M]); `std::fs`/`memmap2` 53 Treffer, unverschlüsselt, Feature-Altlasten (F-23)|
|`contextra-rank`|2|RRF-Fusion.|vorhanden (3.440 Zeilen [M], Ziel deutlich kleiner)|
|`contextra-erasure`|3|Löschkern: Saga, Gate, Journal, Tresor (`FileVault`), Messung, Berichtsbau, Ledger.|**leer** (nur README/AGENTS)|
|`contextra-engine`|3|Orchestrierung: `EngineBuilder`, Instanz (`instance`), Collections, Suche, Transaktionen, Verdrahtung. Keine Löschlogik, kein Chunker, keine Extraktion, kein Import/Export.|vorhanden (28.001 Zeilen [V], 20.357 [M]); **kompiliert nicht** [M]; Fremdmodule F-31; `contextra_impl` (F-30)|
|`contextra`|4|Fassade, einzige öffentliche Oberfläche; **keine Logik**; Module `api`, `wiring`, `error` (D-89).|vorhanden (1.558 Zeilen), widerspricht §7 (F-26)|
|`contextra-testkit`|dev|Fixtures, `FaultVfs`, `ReferenceModel`, `ManualClock`, In-Memory-Ports.|vorhanden (Graph-Reste seit `b1637a8` entfernt)|
|`contextra-lab`|dev|Unabhängige Prüfinstanz (12.3), enthält das Scanner-Modul für `ctx-verify scan`.|**fehlt**|
|`ctx-verify`|dev/bin|Eigenständiger Berichtsprüfer und Feldscanner (8.17).|**fehlt**|
|`tests/arch`|dev|Architektur- und Lint-Tests, Rückverfolgbarkeit Spezifikation ↔ Test (15.1).|vorhanden|

`members` = alle obigen Einträge. `default-members` = die 14 Produktions-Crates (ohne `testkit`, `lab`, `ctx-verify`). Pfadabhängigkeiten und `[workspace.dependencies]` auf entfernte Crates entfallen (Reste: F-05).

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
|AR-4|`lab` und `ctx-verify` hängen nicht an `store`, `engine`, `erasure` und nicht an den Produktions-Decodern von `crypto`. Sie haben eigene Implementierungen der Formate (Anhang F) und Primitive (unabhängiger Prüfer).|T-ARCH-3|
|AR-5|`types` ohne `async` und ohne I/O.|T-ARCH-1|
|AR-6|Nur `TenantScopedStorage` ruft den rohen Store.|T-ARCH-2|
|AR-7|Kein Netzwerk-Crate im Abhängigkeitsbaum der Produktionscrates (D-54).|T-ARCH-4|
|AR-8|`unsafe` nur in `sys` und `simd`; alle anderen `#![forbid(unsafe_code)]`.|T-UNSAFE-1|
|AR-9|Jedes `feature = "…"` im Quelltext ist in einem `[features]`-Abschnitt deklariert; kein `#[allow(unexpected_cfgs)]`; kein Feature schwächt ein Sicherheitsmerkmal (D-84).|T-CFG-1|
|AR-10|`ports` re-exportiert Typen aus `types` nur einzeln und nur, wenn sie in Trait-Signaturen vorkommen; kein `pub use …::*`.|T-ARCH-6|
|AR-11|Ringangabe und Abhängigkeiten im `//!`-Kopf jedes Crates entsprechen 5.1 und 5.3 (D-91).|T-ARCH-5|
|AR-12|Die Fassade enthält keine Logik; Abhängigkeiten laut D-88 (4).|T-ARCH-7|
|AR-13|Namens- und Modulregeln D-90; keine Agenten-/Zeitstempelmarken im Quelltext (D-91).|T-ARCH-7, T-LINT-4|

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
|contextra (Ring 4)|✓||||✓||||||||
|engine|✓|✓|✓||✓|✓|✓|✓|✓|✓|✓|✓|

`store` darf `Vfs` nur als Trait aus `ports` nutzen; die konkrete `RealVfs` wird in `engine`/`contextra` verdrahtet. `store`, `text` und `vector` erhalten Dateischlüssel ausschließlich über den Port `FileKeyring` (Implementierung `FileVault` in `erasure`, verdrahtet in `engine`). `contextra-sys` ist für `durable-fs`, `store` und (nur Prozesshärtung) `engine` zulässig; `vector` benötigt `sys` nur über den Port-Wrapper in `engine`. **Ergänzung 4.1.0 (D-88):** `engine` darf `sys` nur für die Prozesshärtung (D-63) nutzen; `vector` nutzt weder `sys` noch `mvcc` (Arena-Sperren über einen Port aus `ports`, implementiert in `engine`). Die Zeile `contextra` gilt zusätzlich zur Kante auf `engine` (die Matrix hat keine Engine-Spalte): Fassade → `engine`, `durable-fs`, `types`. `text` darf `crypto` nutzen.

### 5.4 Laufzeitmodell

- **Runtime:** Tokio multi-thread. Die Bibliothek erzeugt keine eigene Runtime; `EngineBuilder::runtime(Handle)` ist Pflicht, alle Hintergrundtasks laufen darauf. Es werden nur die Tokio-Features nach D-54 verwendet.
- **Cancel-Safety (I-14):** Jede mutierende Operation läuft in einer vom Aufrufer unabhängigen Task (`spawn`); der Aufrufer wartet auf ein `oneshot`. Abbruch des Aufruferfutures ändert den Ausgang nicht.
- **Hintergrundtasks:** WAL-Flusher, Memtable-Flush, Kompaktion, Purge-Scheduler, Vektor-Reparatur und -Rebuild, Textsegment-Merge, Ledger-Rotation. Alle besitzen Stop-Flag und `JoinHandle`; `close()` beendet sie geordnet.
- **Sperren und Barrieren:** (a) Pro Dokument ein asynchroner Lock für Insert, Update, `soft_delete` und `erase(Document)`. (b) **Commit-Barriere** (`RwLock`): jede Gruppen-Commit-Phase hält sie shared, die Gate-Gruppe (`ErasedSet::add_batch`) hält sie exklusiv (D-57, D-78). (c) **Purge-Barrier** (`RwLock`) zwischen Kompaktion (shared) und Purge (exklusiv); die Kompaktion prüft pro Block das Flag `purge_waiting` und bricht kooperativ ab (D-78, I-16). Lockreihenfolge: Dokumentlock → Commit-Barriere → Purge-Barrier → Journal. Keine Sperre über `.await` einer fremden Task (`await_holding_lock = deny`).
- **Keine Blockierung in `Drop`** (14.3).
- **Poisoned-Zustand (D-55):** atomares Flag im Store; setzt jede mutierende Operation auf `Io`.

### 5.5 Datenflüsse

**Schreiben (Insert/Update):** `Collection::insert(DocRef, Document)` → `TenantContext` prüfen → `ReadGate` prüfen (Ziel nicht `Restricted`/`Gated`) → Dokumentlock → `doc_tag`, `cluster_tag` ableiten (D-08, D-70) → Tresor: fehlende Slots der Ahnenkette (`T`, `C`, `S`) und `D` anlegen, DEK ableiten (D-79) → Envelope sealen (`Doc`, `DocMap`, `Manifest`) → Text- und Vektorpuffer vormerken → **eine** Store-Transaktion: Gruppen-Commit-Leader prüft `ReadGate` erneut (D-57), schreibt **vorher** alle neuen Tresor-Slots der Gruppe und führt **einen** `sync_data` des Tresors aus (I-9), danach WAL-Frames, HMAC, `sync_data` WAL (zwei `fsync` je Gruppe, nicht je Dokument) → Memtable → Sichtbarkeit.

**Lesen:** `get(DocRef)` → `doc_tag`, `cluster_tag` → `ReadGate` → Record lesen → DEK (Tresor/LRU) → Envelope öffnen. Fehlt ein Slot der Ahnenkette: `Erased`.

**Suchen:** Text/Vektor liefern `DocHandle`-Treffer im Strom (Bindung nur für Kandidaten entschlüsselt, D-69) → `ReadGate`-Filter mit Nachladen (D-62) → RRF → Dokumente laden.

**Löschen:** `erase` → Validierung, Retention → Journal `Requested` → Gate-Gruppen-Commit (Gate-Log) → Schlüsselvernichtung (höchste Ebene) und Vektor-Surrogat → Journal `KeyDestroyed` → Rückgabe `Gated` → Purge-Fenster → Drain, Flush, WAL-Segmente entfernen, SSTable-Rewrite, Dateischlüssel vernichten, Textsegment-Rewrite, Vektor-Reparatur, `D`-Slots entfernen, Cache-Eviction → Messung → Bericht → Ledger → `Completed` (11.2).

### 5.6 Verzeichnislayout

```
<data>/                       0700
  LOCK                        exklusiver Prozess-Lock (D-50)
  HEADER.a  HEADER.b          je 1024 B: Format-Version, Store-ID, Keyslots (D-52), report_pub; A/B mit Generationszähler
  report.pub                  Kopie des öffentlichen Schlüssels, vom Betreiber extern zu hinterlegen (D-13)
  wal/        <file_id>.wal   HMAC-Kette, verschlüsselt (Dateischlüssel zufällig, D-85)
  sst/        <file_id>.sst   SSTables (Block-AEAD, zufälliger Dateischlüssel), Level-Struktur im Manifest
  MANIFEST/   <file_id>.log   Manifest-Log (Datei-Mengen, Schlüsselbereiche, Seq, WalFloor); abgeleiteter Dateischlüssel
  text/       <tenant_tag>/<coll_tag>/<file_id>.seg   Textsegmente (D-75)
  vector/     <tenant_tag>/<coll_tag>/<file_id>.vsn   Vektor-Snapshots (D-75)
  vault/      vault.dat       Tresor: Slots T, C, S, D, F (11.5, F.3)
  erasure/    journal.log     Löschjournal (Hash-Kette, AEAD)
              gates.log       aktive Gates (Append-Log, D-58)
              scope/          <erasure_id>.scp: Auflösung je laufender Löschung (verschlüsselt)
              report/         <erasure_id>.json: signierte Berichte
  tmp/        Waisen werden beim Start entfernt
```

Außerhalb des Datenverzeichnisses: das Ledger-Exemplar des Betreibers (`LedgerSink`, 11.8). Das Verzeichnis `keys/` aus 3.0.0 entfällt (D-80). Kein Datei- oder Verzeichnisname enthält etwas anderes als Tags, `file_id`, `erasure_id` oder feste Namen (T-RAW-6).

### 5.7 Konfiguration (`EngineBuilder`)

|Parameter|Pflicht|Standard|Prüfung|
|---|---|---|---|
|`data_dir`|ja|–|existiert oder wird mit 0700 angelegt; Dateisystem laut D-56|
|`passphrase`|ja|–|Länge ≥ 12 Bytes|
|`erase_sla`|ja|–|1 min ≤ sla ≤ 30 d (Empfehlung: Abschluss ≤ 24 h, Alarm bei `Pending` > 1 h, 16)|
|**`backup_retention`**|**ja**|–|Dauer 1 d … 30 Jahre; steuert die Ledger-Lebensdauer (D-74); `open` ohne Angabe wird verweigert|
|`runtime` (Handle)|ja|–|–|
|`vfs`, `clock`, `rng`|ja (Fassade: `system_defaults()`)|–|Traits|
|`kdf`|nein|m = 128 MiB, t = 3, p = 4|innerhalb der Grenzen von D-06|
|`erase_return_timeout`|nein|60 s|1 s … 10 min|
|`batch_window`|nein|60 s|0 s … `erase_sla / 2` (D-39)|
|`history_horizon`|nein|24 h|0 … 365 d|
|`max_doc_bytes`|nein|4 MiB|≤ 64 MiB|
|`vector_rebuild_ratio`|nein|0,2|0,05 … 0,5|
|`vector_strict_rebuild`|nein|aus|D-77; Bericht weist `IndexTopologyInference` bei „aus“ aus|
|`commit_epoch`|nein|365 d|30 d … 3 Jahre (rotiert nur Commitments in Journal und Bericht, D-48)|
|`ledger_sink`|nein|Datei außerhalb des Datenverzeichnisses (`<data>/../.contextra-ledger/<store_id>`)|Pfad ≠ Unterpfad von `data_dir`|
|`process_hardening`|nein|`BestEffort`|`Required`/`BestEffort`/`Off` (D-63)|
|`min_free_bytes`|nein|`max(1 GiB, 2 × größte Kandidatendatei)`|≥ 256 MiB|
|Collection-Optionen|bei Anlage|`analyzer = de/1`, `positions = true`|unveränderlich (D-81)|

Fehlende Pflichtparameter oder Werte außerhalb der Grenzen sind `ConfigError` beim `open`. Es gibt keine stillen Korrekturen.

### 5.8 Budgets (CI-blockierend)

|Budget|Wert|Messung|
|---|---|---|
|Erase-Produktcode (`contextra-erasure`, `store/src/erase`, `types/src/erasure`, `crypto/src/vault_cipher.rs`)|≤ 9.000 Zeilen (3.0.0: 7.000; +2.000 für Schlüsselhierarchie, Dateischlüssel-Slots, Ledger-Bereiche, Gate-Gruppen)|`tokei`|
|`unwrap`/`expect`/`panic!`/`let _ =`/`#[allow]` im Produktionscode|nur sinkend (Ratchet); Ziel 0 in allen Kern-Crates ab Gate G-C. Die Ratchet-Datei `governance/ratchet.toml` wird mit WP-0.1 angelegt (F-20).|`governance/ratchet.toml`|
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
|H3|**Produkthaftung** (ProdHaftG-Neufassung nach RL (EU) 2024/2853; Software ist Produkt) **[W]**: vom Bundestag am **08.10.2026** beschlossen (Einspruchsgesetz), anwendbar auf Produkte, die ab **09.12.2026** in Verkehr gebracht werden; für frühere gilt das alte Gesetz; Beweiserleichterungen für Geschädigte. **Reichweite:** Ansprüche stehen natürlichen Personen zu; ersatzfähig sind Personen- und Sachschäden **[W]**, Datenverlust nach der Richtlinie nur für Daten, die nicht zu beruflichen Zwecken genutzt werden (Art. 6 RL (EU) 2024/2853) **[D]**. Für Unternehmen als Kunden greifen daher vorrangig Vertrag und Deliktsrecht (§§ 280, 823 BGB). Freie und Open-Source-Software außerhalb einer Geschäftstätigkeit ist ausgenommen, wer sie in ein Produkt integriert, haftet [W, PwC]; E-L9 unterstellt die strengste Annahme.|Falsches Löschziel, Datenverlust nach Absturz, unbemerkte Korruption|T-ISO, Crash-Matrix, WAL-HMAC, Block-AEAD sind haftungskritisch; Release-Blocker HR-7; Update- und Sicherheitsupdate-Prozess. Ein 1.0.0, das ab 09.12.2026 bereitgestellt wird, unterliegt dem neuen Recht; Aufzeichnungen der Gates (Evidenz) sind Beweismittel. Die B2B-Reichweite prüft G-LEGAL (Frage 13).|T-ISO-*, T-CRASH-*, T-WAL-1|
|H4|**Cyber Resilience Act** (VO (EU) 2024/2847): Meldepflichten für aktiv ausgenutzte Schwachstellen und schwere Vorfälle (Art. 14) gelten **seit 11.09.2026** über die ENISA Single Reporting Platform; übrige Pflichten (u. a. CE-Kennzeichnung, SBOM, Support-Zeitraum) ab **11.12.2027** [W/D]|Bereitstellung eines Produkts mit digitalen Elementen|Meldeprozess (D-67), SBOM (`cargo cyclonedx`) je Release, Meldeweg und Fristen in der Sicherheitsdokumentation, benannter Support-Zeitraum, signierte Releases|T-SUPPLY-1, T-SUPPLY-2|
|H5|DSGVO direkt|Contextra-Seite erhält Daten (Telemetrie, Crash-Upload, Lizenzserver, Escrow)|HR-1..HR-4|T-ARCH-4, T-LOG-1|
|H6|Berufsgeheimnis (§ 203 StGB)|Einsatz bei Kanzlei, Arzt, Steuerberater|Kein Zugriff durch Contextra; Support nur mit Berichten ohne Inhalt; Berichtsvorlage enthält nur Commitments und Tags (P13)|HR-4, T-REPORT-1|
|H7|Verlässlichkeit der Nachweise|Betreiber verlässt sich auf einen falschen Bericht|Fail-closed, typisierte Positivkontrolle, Mutationstests auf `classify`, unabhängiger Verifier und Feldscanner|T-RAW-2, T-PROOF-1, T-MUT-1|
|H8|Krypto-Obsoleszenz (Art. 32)|Algorithmus gebrochen oder Parameter veraltet|Algorithmus-ID und Format-Version in jedem Header; jährliche Neubewertung (Release-Checkliste); Keyslots erlauben KDF-Wechsel|T-FMT-1, T-KEYSLOT-2|
|**H9**|**Aufsichtsrechtliche Wertung der Contextra-Artefakte** (EDPB Guidelines 02/2026 Anonymisierung, Entwurf; Konsultation am 30.10.2026 beendet [W]; relativer Personenbezug nach EuGH C-413/23 P)|Tags, Commitments, Ledger-Bereiche, entkoppelte Postings, Graphtopologie werden als personenbezogen oder identifizierbar gewertet|Konservativ: alle genannten Artefakte gelten als personenbezogen (E-L3, E-L4, E-L15); Datensparsamkeit (P13), Fristen (D-48, D-74), Purge im ersten Fenster (D-69); Restrisiken im Bericht|T-RAW-*, T-KEYCOV-1, T-JRN-3|

### 6.3 Harte Regeln

|ID|Regel|Durchsetzung|
|---|---|---|
|HR-1|Kein Netzwerkzugriff im Produktionsbuild. Keine HTTP-, Socket- oder DNS-Bibliothek im Abhängigkeitsbaum der Produktionscrates.|`deny.toml [bans]` (D-54), `cargo tree -e normal`-Test, Quellscan nach `std::net`, `tokio::net`|
|HR-2|Keine Telemetrie, kein Crash-Upload, kein Phone-Home.|wie HR-1|
|HR-3|**Kein Schlüssel-Escrow, keine Wiederherstellung durch den Hersteller.** Passphrase verloren heißt Daten verloren; dies steht an erster Stelle der Dokumentation. Betreiberseitige zusätzliche Keyslots (z. B. Notfallpassphrase im Tresor des Betreibers) sind Betreiberentscheidung, nicht Hersteller-Escrow.|Doku-Test, Review|
|HR-4|Fehler- und Lognachrichten enthalten nie Inhalte, Nutzerkennungen, Schlüssel oder Pfade mit Personenbezug.|T-LOG-1|
|HR-5|Außenaussagen ausschließlich aus 6.4.|T-LINT-1|
|HR-6|Jede Zeile der Leistungsbeschreibung hat mindestens einen automatisierten Test und einen Evidenzlauf; was nur im Lab belegbar ist, steht als `lab_evidence`, nicht als Produktionszusage.|CI-Job `leistung`|
|HR-7|Ein Test, der zeigt, dass ein Erase Daten außerhalb des Ziels verändert, ist **Release-Blocker ohne Ausnahme**.|G-D, G-H|
|HR-8|Sicherheitslücken werden innerhalb dokumentierter Fristen bearbeitet (kritisch 7 Tage, hoch 30 Tage); aktiv ausgenutzte Schwachstellen werden nach CRA Art. 14 gemeldet; unterstützte Versionen sind benannt.|Sicherheitsdokumentation, Release-Checkliste|
|HR-9|Haftungsausschlüsse der Lizenz (MIT/Apache-2.0) werden **nicht** als Schutz verbucht. Für Vorsatz, grobe Fahrlässigkeit und Produkthaftung gelten zwingende Grenzen.|Prozess|
|HR-10|Löschzusagen im Bericht enthalten nie einen Zeugen ohne Positivkontrolle (typisiert erzwungen, D-72).|T-RAW-2|
|HR-11|Kein Release ohne bestandenes Gate G-H und signierte Artefakte samt SBOM.|15.3|
|HR-12|Kein neuer `unsafe`-Block, keine neue Abhängigkeit, kein neues Feature ohne Nennung des Tests, der es belegt (PR-Vorlage).|PR-Template, `cargo-deny`|
|**HR-13**|Kein Feature, kein Laufzeitparameter und kein Umgebungsschalter schwächt Verschlüsselung, Integrität, Löschmessung oder Mandantenprüfung (P5, D-05, D-84).|T-CFG-1, T-DEF-1|

### 6.4 Zulässige Außenaussage (einzige)

> **DE:** Contextra entfernt die Daten eines Löschziels aus dem Datenverzeichnis, aus den Indexen und aus den gespeicherten Zeitreise-Versionen und weist den Zustand je Schicht messtechnisch nach. Für jedes Ziel wird zusätzlich der zugehörige Schlüssel aus dem Schlüsseltresor entfernt; ob und wie das rechtlich zu werten ist, entscheidet der Betreiber. Nicht erfasst sind externe Sicherungen, Flash-Remanenz, Dateisystem-Journale, Prozessspeicher und externe Verarbeiter. Der Bericht listet diese Restrisiken.
>
> **EN:** Contextra removes the data of an erasure target from the data directory, from its indexes and from stored time-travel versions, and provides per-layer, measured evidence of the resulting state. For each target the associated key is additionally removed from the key vault; whether and how this is to be assessed legally is for the operator to decide. Not covered are external backups, flash remanence, file-system journals, process memory and external processors. The report lists these residual risks.

**Verbotene Begriffe** (Wortlaut-Lint über Quelle, Rustdoc, README, Doku, Berichtsvorlagen, Marketing; deutsch und englisch): „garantiert“/„guaranteed“, „100 %“ (im Zusammenhang mit Löschung), „DSGVO-konform“/„GDPR-compliant“, „rechtskonform“, „unwiederbringlich“/„irrecoverable“, „unwiderruflich“, „informationstheoretisch“, „kryptografisch gelöscht“/„cryptographically erased“ (ohne Verweis auf Bericht und Voraussetzungen), „sicher gelöscht“ ohne Berichtsbezug, „vollständig gelöscht“ ohne Klassenangabe, „zertifiziert“/„certified“, „von einer Aufsichtsbehörde geprüft“, „anonymisiert“ als Beschreibung des Löschergebnisses.

**Lint-Geltungsbereich (T-LINT-1):** Das Lint prüft alle Produktartefakte. **`docs/SPEC.md` ist ausgenommen**, weil sie die Verbotsliste führt und begründet; alle aus ihr in Produktartefakte kopierten Texte (6.4, 16, Anhang D) sind lint-rein formuliert. T-LINT-3 prüft zusätzlich, dass die Begriffe „E1/E2/E3“ (alte Messklassen) und „E0..E4“ (alte Evidenzstufen) nirgends mehr vorkommen.

### 6.5 Rechtsanforderungsmatrix

|#|Anforderung (Orientierung)|Mechanismus|Test|Restrisiko im Bericht|
|---|---|---|---|---|
|R1|Unverzüglich, Monatsfrist (Art. 17 Abs. 1, Art. 12 Abs. 3)|Saga mit Wiederholung, `erase_sla`, Status, Alarm|T-CRASH-1, T-SLA-1|`ReplicaWindow`|
|R2|Unkenntlichmachen, nicht reproduzierbar (SDM-Baustein 60, ErwGr 26)|Klassen K und P, Dateischlüssel-Vernichtung|T-RAW-1..6, T-FK-1..3|`FlashRemanence`, `WrappedKeyRemanence`|
|R3|Einfache OS-Löschbefehle genügen nicht (SDM-Baustein 60)|Rewrite, Dateischlüssel-Vernichtung, Unlink, Fehlen geprüft, Tresor-Slot genullt|T-CRASH-1, T-RAW-1|`FsJournalOrCow`|
|R4|Alle Versionen und Kopien (SDM M60.P14)|Purge aller Versionen, Snapshot-Widerruf, Index-Segmente|T-VER-1|`ExternalCopies`|
|R5|Rechenschaft (Art. 5 Abs. 2, Art. 24)|signierter Bericht, Audit-Kette, Feldscanner|T-PROOF-1, T-REPORT-1|`PseudonymCommitment`|
|R6|Datenschutz durch Technikgestaltung und Voreinstellung (Art. 25)|Verschlüsselung und Mandantenpflicht ohne Opt-out|T-DEF-1, T-API-1|–|
|R7|Sicherheit der Verarbeitung (Art. 32)|Argon2id mit Grenzen, WAL-HMAC, AEAD mit AAD, Keyslots|T-KDF-1/2, T-WAL-1, T-ENC-2|Schlüsselkompromittierung|
|R8|Information von Empfängern (Art. 17 Abs. 2, Art. 19)|Bericht führt `ExternalProcessors`; Checkliste (16); Hook `on_erased` in v1.1|–|Verantwortung Betreiber|
|R9|Aufbewahrungspflichten (Art. 17 Abs. 3), Einschränkung (Art. 18)|`RetentionAttestation`, `Restricted`|T-API-2, T-RESTR-1|Prüfung beim Betreiber|
|R10|Embeddings vorsorglich personenbezogen|Vektoren im Umfang, Snapshot verschlüsselt, Surrogat, Kantenreparatur, Rebuild|T-RAW-3, T-VEC-DEL-*|`ProcessMemory`, `IndexTopologyInference`|
|R11|Nach Restore erneut löschen (SDM M60.S10, M60.P08)|Restore-Guard (Ledger mit Bereichen)|T-RESTORE-1..3|Backup-Inhalte außerhalb|
|R12|Löschbarkeit auf Datensatz- und Gruppenebene mit beherrschbarem Aufwand (SDM M60.D01)|Löschcluster-Schlüsselraum, Segmentindex, Schlüsselhierarchie|BM-ERASE-WA, BM-ERASE-O1|–|
|R13|Datensparsames Löschprotokoll (SDM M60.S05/S06)|Journal-Reduktion, Commitments mit Epochenrotation|T-REPORT-1, T-JRN-3|`PseudonymCommitment`|
|R14|Regeln zu Schlüsseln zu löschender Daten (SDM M60.P09/P10)|Tresor-Protokoll (11.5)|T-VAULT-1..5|`WrappedKeyRemanence`|
|R15|Algorithmenwechsel planbar|Algorithmus-ID in Headern, Keyslots|T-FMT-1, T-KEYSLOT-2|–|
|R16|Werbeaussagen belegbar (§ 5 UWG)|Wortlaut-Lint|T-LINT-1|–|
|R17|Datenschutz-Folgenabschätzung des Betreibers (Art. 35)|Lieferdokument DSFA-Baustein (Anhang G): technische Maßnahmen, Restrisiken, Datenflüsse, Begründung von `backup_retention`; keine Rechtsbewertung|Review|–|
|R18|Meldung von Schwachstellen (CRA Art. 14)|PSIRT-Prozess (D-67)|T-SUPPLY-1|–|
|**R19**|**BDSG § 35** (Einschränkung statt Löschung bei unverhältnismäßigem Aufwand): nach meiner Lesart auf **nicht automatisierte** Verarbeitung beschränkt **[D]**|Contextra stützt sich nicht darauf; physische Löschung macht die Ausnahme entbehrlich. Anwendbarkeit auf RAG-Verarbeitung klärt G-LEGAL (Frage 12).|–|–|
|**R20**|**Aufbewahrungspflichten** (§ 257 HGB, § 147 AO) und GoBD (Unveränderbarkeit) [D]|`RetentionAttestation`/`Restricted`; Zuordnung Fachverfahren → `RetentionCode` im DSFA-Baustein|T-API-2, T-RESTR-1|Prüfung beim Betreiber|
|**R21**|**BSI IT-Grundschutz CON.6** (Löschen und Vernichten) und **ISO/IEC 27001:2022 A.8.10** (Löschung von Informationen), ergänzend A.8.24 (Kryptografie) [D]|Abbildung der Mechanismen auf die Anforderungen im DSFA-Baustein|Review|–|
|**R22**|**NIS2-Umsetzung im deutschen Recht** (wesentliche/wichtige Einrichtungen als Kunden) [D]|Wirkt auf Lieferkette und Betrieb des Kunden; Beitrag: SBOM, Support-Zeitraum, PSIRT. Gehört in den Betriebsbaustein, nicht in den Kern.|T-SUPPLY-1|–|
|**R23**|**§ 203 StGB** (Berufsgeheimnisträger) [D]|Berichte und Support ohne Inhalte|T-REPORT-1, T-LOG-1|–|

### 6.6 Rechtsentscheidungen (technisch konservativ, endgültig)

Jede Frage, die die Technik berührt, ist hier entschieden. Maßstab ist immer die **strengste** Auslegung.

|ID|Frage|Entscheidung|Folge|
|---|---|---|---|
|E-L1|Genügt physische Umschreibung samt Entfernen der Altdateien?|Sie ist Pflichtmaßnahme (Klasse P), wird aber **nie allein** als ausreichend dargestellt. Der Bericht führt `FlashRemanence` und `FsJournalOrCow` immer, wenn zutreffend.|Säule 1 plus 2|
|**E-L2**|Ist die Vernichtung des Schlüssels (Klasse K) Löschung, Anonymisierung oder Einschränkung?|Contextra **behauptet keine** dieser Wertungen. Der Mechanismus läuft immer. Der Bericht weist `key_destroyed` und `key_coverage` je Schicht als Tatsache aus und nennt `WrappedKeyRemanence`. Stand [D, EDPB-Teil W]: keine Behörden- oder Gerichtsaussage erkennt Schlüsselvernichtung als Löschung an; EDPB-Leitlinien 02/2025 (Blockchain) nennen Bedingungen (Algorithmus, Schlüsselkopien) [D]; EDPB Guidelines 02/2026 (Anonymisierung) sind **Entwurf**, die Konsultation endete am 30.10.2026, die Endfassung steht aus; der Entwurf nimmt einen relativen Personenbezug an und verlangt eine fortlaufend geprüfte, dokumentierte Bewertung [W].|Bericht, 6.4|
|**E-L3**|Sind Embeddings, HNSW-Slotnummern, Kanten und blinde Tags personenbezogen?|**Ja, vorsorglich.** Embeddings werden durch Surrogate ersetzt, Snapshots verschlüsselt, Tags im Purge entfernt, Kanten repariert. Bis zum Rebuild weist der Bericht `IndexTopologyInference` aus.|D-77, D-19|
|**E-L4**|Sind Bericht, reduziertes Journal und Ledger personenbezogen?|**Pseudonym ja, daher begrenzt:** nur Commitments, Bereichs-Tags und Zeiten, nie Kennungen. Journal und Bericht: Lebensdauer = Commitment-Epoche (365 d). **Ledger: Lebensdauer = `backup_retention` mit Zweckbindung „Restore-Schutz“**; Begründung im DSFA-Baustein.|D-48, D-74|
|E-L5|Sind aggregierte Textstatistiken personenbezogen?|Sie enthalten keine Dokumentbezüge. Sie werden dennoch **nach jedem Segment-Rewrite exakt neu berechnet**; kein Zähler überlebt die Löschung seines Beitrags.|9.7|
|E-L6|Rolle von Contextra?|Weder Verantwortlicher noch Auftragsverarbeiter, solange HR-1..4 gelten. Dokumentation weist darauf hin; Betreiber prüft selbst.|HR|
|E-L7|Aufbewahrungspflichten?|Contextra führt kein Fristenwerk. Der Betreiber erklärt per `RetentionAttestation`. Bei Pflicht: `Refused(RetentionObligation)`, oder `Restricted` als Gate-Variante ohne Vernichtung.|D-15|
|E-L8|Löschcheckliste und Berichtstext?|Wortlaut ist in Abschnitt 16 und Anhang D **endgültig festgelegt**; sie sind kein Rechtsrat und enthalten diesen Hinweis.|Lint|
|E-L9|ProdHaftG/CRA bei Open-Source-Bibliothek?|Contextra behandelt sich als **dem CRA und dem ProdHaftG unterliegend** (strengste Annahme, auch wenn eine Ausnahme für nicht-kommerzielle freie Software bestehen kann): SBOM, Meldeweg, Support-Zeitraum, Sicherheitsupdates.|H3, H4, D-67|
|E-L10|Wirkung von Haftungsausschlüssen?|Werden nicht als Schutz verbucht (HR-9). Die Leistungsbeschreibung ist schmal und testbar gehalten.|Anhang B|
|E-L11|Reichweite der Signatur?|Die Signatur beweist Integrität und Herkunft aus der Installation, nicht die Wahrheit des Inhalts. Dieser Satz steht im Bericht. Wer die Passphrase hält, kann Berichte erzeugen; eine unabhängige Zeitstempelung (RFC 3161) ist ein optionaler Betreiber-Hook ab v1.1.|D-13|
|E-L12|Liste der `RetentionCode`s?|Festgelegt in 7.3 (Enum ohne Freitext).|T-API-2|
|E-L13|Beanspruchen wir „Anonymisierung“?|**Nein.** Weder API, Bericht noch Dokumentation verwenden den Begriff für das Löschergebnis (Wortlaut-Lint).|T-LINT-1|
|E-L14|Wie wird der Zeitbezug „unverzüglich“ technisch bestimmt?|Contextra misst Zeiten (`requested_at`, `gated_at`, …) aus dem `Clock`-Port und weist `erase_sla`-Verletzungen aus. Die Frist selbst (Monatsfrist, ggf. Verlängerung nach Art. 12 Abs. 3) bestimmt der Betreiber; Empfehlung: Abschluss ≤ 24 h (16).|T-SLA-1|
|**E-L15**|Sind entkoppelte Textpostings (`unlinked`) und reparierte Graphkanten Rückstände mit Personenbezug?|**Vorsorglich ja, bis zum Purge.** Sie werden im ersten Purge-Fenster (Text) bzw. beim nächsten Rebuild (Kanten) beseitigt; der Bericht führt `TextIndexUntilPurge` und `IndexTopologyInference`.|D-69, D-77|
|**E-L16**|Sind Tags in Datei- und Verzeichnisnamen personenbezogen?|Pseudonym (keyed Hash). Sie verraten nur Existenz und Zahl von Mandanten und Collections. Klartextnamen sind verboten (D-75).|T-RAW-6|

### 6.7 Gate G-LEGAL (externe Freigabe, nicht technisch)

Ohne schriftliche Antworten eines Fachanwalts für Datenschutzrecht bzw. der Datenschutzbeauftragten darf das Produkt keine Löschaussage außer der Zulässigen (6.4) treffen. Prüffragen: (1) Qualifikation der Schlüsselvernichtung (Löschung, Anonymisierung, Unzugänglichmachung, Einschränkung) und Haltung der zuständigen Aufsichtsbehörde; (2) relativer Personenbezug nach EuGH C-413/23 P; (3) Wirksamkeitsnachweis, Umfang und Remanenzrisiko des dateibasierten Tresors samt Dateischlüssel-Slots; (4) Embeddings, Slot-Nummern, Graphtopologie; (5) Backups und Restore-Guard-Horizont; (6) Zulässigkeit, Inhalt und Dauer von Commitments in Bericht und Ledger; (7) Rolle des Herstellers bei lokal installierter Software; (8) Aufbewahrungspflichten je Branche; (9) Freigabe der Löschcheckliste (16) und der Berichtstexte; (10) Prüfung jeder Werbeaussage auf UWG/BGB; **(11)** Anwendung des Prüfrasters der EDPB Guidelines 02/2026 (nach Veröffentlichung der Endfassung) auf Tags, Commitments, Ledger-Bereiche, entkoppelte Postings und Graphtopologie; **(12)** Anwendbarkeit von § 35 BDSG auf automatisierte RAG-Verarbeitung; **(13)** Reichweite des neuen ProdHaftG für B2B-Kunden und die Open-Source-Ausnahme; **(14)** Zweckbindung und zulässige Dauer des Ledgers (`backup_retention`). **Der Zeitplan hängt nicht an diesem Gate**: Es blockiert nur die Veröffentlichung von Löschaussagen über 6.4 hinaus, nicht die Entwicklung.

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
    pub async fn lift_restriction(&self, id: ErasureId, att: RetentionAttestation) -> Result<(), Error>;
    pub async fn acknowledge_ledger_reset(&self, att: OperatorAttestation) -> Result<(), Error>;
    pub async fn change_passphrase(&self, old: &Secret, new: &Secret) -> Result<(), Error>;
    pub fn health(&self) -> HealthReport;    // D-64, nicht async, kein I/O
    pub async fn close(self) -> Result<(), Error>;
}
pub struct Tenant;        // an TenantContext gebunden (D-07)
impl Tenant {
    pub async fn collection(&self, name: &str, opts: CollectionOptions) -> Result<Collection, Error>;
}
```

Es gibt keine Funktion ohne `TenantContext` (T-API-1, trybuild). `Secret` ist `zeroize`-on-drop, hat kein `Debug`/`Display`/`Clone`. `CollectionOptions` (Analyzer, Positionen, Vektordimension) sind bei Anlage fest (D-81).

### 7.2 Dokument-API (`Collection`)

```rust
pub struct DocRef { pub subject: Option<SubjectRef>, pub key: DocKey }   // D-70
pub struct DocHandle { /* cluster_tag, doc_tag; opak */ }               // Suchtreffer
pub struct SearchPage { pub hits: Vec<Hit>, pub exhausted: bool }       // D-62
```

|Methode|Wirkung|Fehler (Auszug)|
|---|---|---|
|`insert(DocRef, Document)`|legt an; der Betroffene ist Teil der Identität (D-40, D-70)|`Conflict::IdCollision`, `Conflict::Exists`, `Erased`, `Restricted`, `Limit::*`|
|`update(DocRef, Document)`|neue MVCC-Version desselben `DocRef`|`NotFound`, `Erased`, `Restricted`|
|`get(DocRef, at: Option<SnapshotId>)`|Lesen, optional Zeitreise|`Erased`, `NotFound`, `SnapshotRevoked`|
|`soft_delete(DocRef)`|logisch (Tombstone), **keine Löschaussage** (D-51)|`NotFound`|
|`scan(range)`|Iteration in `(cluster_tag, doc_tag)`-Reihenfolge; `ScanRange::{All, Subject(SubjectRef), After(DocHandle)}` (D-41)|gegatete Elemente werden still übersprungen, nie sichtbar|
|`search_text(query, k, filter)`|BM25, ReadGate-Filter mit Nachladen (D-62); liefert `SearchPage`|`Limit::QueryTooLarge`|
|`search_vector(vec, k, filter)`|HNSW; liefert `SearchPage`|`Dimension`, `Limit::*`|
|`search_hybrid(query, vec, k, filter)`|RRF aus Text und Vektor; liefert `SearchPage`|–|
|`load(DocHandle)`|lädt das Dokument zu einem Treffer|`Erased`, `NotFound`|
|`snapshot()`|MVCC-Snapshot (mandantengebunden, D-61)|`Limit::Snapshots`|

`Document` = Textchunks (gesamt höchstens `max_doc_bytes`), Metadaten (typisierte Schlüssel/Werte, keine Freitext-Nutzerkennung im Schlüsselraum), optionale Vektoren je Chunk. `filter` wirkt in v1 nachgelagert (D-62). Rustdoc von `soft_delete` beginnt mit: „Für Löschpflichten `erase` verwenden.“ (T-API-3).

### 7.3 Lösch-API

```rust
pub struct EraseRequest {
    pub target: EraseTarget,
    pub timing: Timing,                       // Now | Batched
    pub retention: RetentionAttestation,      // Pflicht (D-15)
    pub mode: EraseMode,                      // Erase | Restrict
}
pub enum EraseTarget {
    Document   { tenant: TenantRef, collection: CollectionRef, doc: DocRef },
    Subject    { tenant: TenantRef, subject: SubjectRef },   // Art.-17-Standardfall (D-14)
    Collection { tenant: TenantRef, collection: CollectionRef },
    Tenant     { tenant: TenantRef },                        // Offboarding (D-38)
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

Contextra prüft die Erklärung nicht inhaltlich (E-L7); sie wird signiert im Bericht geführt. Ein `erase` auf ein bereits gegatetes Ziel ist idempotent (gleiche `ErasureId`); ein `Erase` auf ein `Restricted`-Ziel ist zulässig und hebt die Restriktion durch Löschung auf (Journal-Eintrag).

### 7.4 Rückgabe und Status

```rust
pub struct EraseReceipt { pub id: ErasureId, pub status: ErasureStatus, pub gated_at: Timestamp }
pub enum ErasureStatus {
    Requested, Gated, KeyDestroyed, Purging, Verifying, Completed,
    Pending { reason: PendingReason },      // sichtbar, nie still
    Refused(RefusalReason),
}
```

`erase` kehrt zurück, sobald Gate persistiert **und** Schlüssel vernichtet (`Gated`), oder nach `erase_return_timeout` mit `Pending`. `Completed` entsteht nur aus Zeugen aller Schichten (P1). Idempotenz: gleicher `EraseRequest` liefert dieselbe `ErasureId` (Ableitung aus Ziel-Commitment und Attestierung).

### 7.5 Fehlermodell

`Error` ist eine geschlossene Enum mit festen Fehlertexten (HR-4, T-LOG-1): `Config`, `Io` (inkl. Poisoned), `Corrupt`, `Auth` (falsche Passphrase), `Erased`, `Restricted`, `NotFound`, `Conflict`, `Limit` (inkl. `Capacity`, `DiskReserve`, `Snapshots`, `QueryTooLarge`), `SnapshotRevoked`, `Cancelled`, `Refused`, `Internal`. Kein Fehler trägt Nutzerinhalt, Schlüssel oder Klartext-Kennung. `Corrupt` ist immer fatal für das betroffene Objekt (P2), nie „best effort“.

---

## 8. Komponenten (je Crate)

Jede Komponente nennt Verantwortung, öffentliche Verträge und Abgrenzung. Ringe und Kanten: 5.1–5.3.

**8.1 `types`** – IDs, Tags (`[u8;16]`-Newtypes), `KeyBuf` (feste Länge, kein String), Fehler, Löschtypen (`ErasureId`, `EraseTarget`, `LayerWitness`, `ResidualRisk`, `KeyCoverage`). Kein I/O, kein `async`, `DocId(u64)` entfällt (D-08).

**8.2 `ports`** – Traits: `StorageEngine`, `TextIndex`, `VectorIndex`, `Vfs` (**ein** Trait, D-04), `Clock`, `Rng`, `IdGen`, `ErasureLayer`, `ReadGate`, `RecordSealer`, `FileKeyring`, `LedgerSink`, `Metrics`. Altports entfallen (D-24). `ErasureLayer`:

```rust
trait ErasureLayer: Send + Sync {
    fn id(&self) -> LayerId;
    fn resolve(&self, t: &EraseTarget) -> BoxFuture<Result<LayerScope>>;   // Umfang bestimmen
    fn on_gate(&self, s: &LayerScope) -> BoxFuture<Result<()>>;            // Sofortmaßnahmen vor Gated (z. B. Vektor-Surrogat)
    fn purge(&self, s: &LayerScope, cx: &PurgeCx) -> BoxFuture<Result<PurgeOutcome>>;
    fn witness(&self, s: &LayerScope, sc: &Scanner<Armed>) -> BoxFuture<Result<LayerWitness>>;  // nur lesend
    fn key_coverage(&self) -> KeyCoverage;                                 // full | unlinked | none (D-69)
}
trait FileKeyring: Send + Sync {          // D-85
    fn create(&self, kind: FileKind, id: FileId) -> BoxFuture<Result<FileKey>>;
    fn open(&self, id: FileId) -> BoxFuture<Result<FileKey>>;
    fn destroy(&self, id: FileId) -> BoxFuture<Result<()>>;               // Slot nullen, sync, Fehlen prüfen
}
```

**8.3 `sys`** – einzige `unsafe`-Stelle für OS: `fsync`/`fdatasync`/`F_FULLFSYNC`, `flush_dir`, `rename_durable`, `mlock`, `RLIMIT_CORE`, `PR_SET_DUMPABLE`, `madvise`, Dateirechte (POSIX). Jede Funktion mit offener Vorbedingung ist `unsafe fn` (D-29). Panics ausgeschlossen. Windows-Code entfernt (D-84).

**8.4 `simd`** – f32-Distanzkerne (Dot, L2, Cosine) mit Skalar-Referenz; Laufzeit-Dispatch; jeder Kernel hat Cross-Check gegen die Referenz (T-SIMD-1); u8-VNNI entfällt (D-28).

**8.5 `durable-fs`** – `RealVfs`: Datei öffnen, positionelles Lesen/Schreiben, `sync_data`/`sync_all`/`sync_dir`, atomarer Austausch (tmp, fsync, rename, Verzeichnis-fsync), exklusiver Lock (D-50), `scrub` nur nach D-83, Startprobe (D-56). Meldet `fsync`-Fehler unveränderlich nach oben (D-55).

**8.6 `crypto`** – AES-256-GCM-SIV (Records, Keyslots, Block-AEAD; D-80), Argon2id mit Grenzen (D-06), HKDF-SHA256, BLAKE3 keyed, HMAC-SHA256 (WAL-Kette), Ed25519. Nonce-Schema nach D-60. Alle Schlüssel `ZeroizeOnDrop`. Keine Feature-Schalter (D-05).

**8.7 `mvcc`** – Sequenzen, Snapshots, SSI-Validierung. `min_read_snapshot` liefert `Option`; der Sentinel `u64::MAX` (F-07) entfällt: ohne offene Snapshots gilt `current_seq`, nie „alles sichtbar“. Snapshot-Registry nach `tenant_tag` (D-61).

**8.8 `store`** – LSM: WAL (HMAC-Kette, verschlüsselt, nie umgeschrieben, D-73), Memtable, SSTable mit Block-AEAD, Manifest-Log, Kompaktion (kooperativ abbrechend, D-78), **Purge-Primitive** (`drain_range`, `rewrite_without`, `seal_wal_at`, `drop_wal_until`, `unlink_verified`). Nur über `TenantScopedStorage` erreichbar (AR-6). Kennt `Vfs`-Trait, nicht `RealVfs` (5.3). `Poisoned`-Flag (D-55). Gruppen-Commit mit ReadGate-Prüfung (D-57).

**8.9 `text`** – Segmentierter BM25-Index (9.7): Textpuffer, Segmente, Merge, Rewrite je Zielsegment, Analyzer (D-81). Implementiert `TextIndex` und `ErasureLayer`.

**8.10 `vector`** – HNSW mit Arena (D-86), Surrogat, Batch-Kantenreparatur, deterministischer Rebuild (D-77), verschlüsselter Snapshot (D-19) nur über `Vfs`. Kein `std::fs`, kein `memmap2` auf Klartext (F-13). Implementiert `VectorIndex` und `ErasureLayer`.

**8.11 `rank`** – nur RRF (D-26), `k_rrf = 60`, deterministische Tie-Breaks nach `doc_tag`.

**8.12 `erasure`** – Löschkern (11): Saga-Treiber, `ErasedSet`/Gate-Log, Journal, Tresor-Protokoll (`FileVault`, `RecordSealer`, `FileKeyring`), Messung (`Scanner<Armed>`), Berichtsbau, Ledger, Restore-Guard. Hängt nur an `types`, `ports`, `crypto` (AR-3).

**8.13 `engine`** – `EngineBuilder`, Instanz (Modul `instance`, D-90), Verdrahtung: `TenantScopedStorage`, Collections, Transaktionen, Suche, Hintergrundtasks, Startsequenz (14.1). Enthält keine Löschlogik.

**8.14 `contextra`** – Fassade ohne Logik (D-89): Module `api`, `wiring` (`system_defaults()`), `error`; Re-Exports, Doc-Tests. Abhängigkeiten: `engine`, `durable-fs`, `types` (D-88).

**8.15 `testkit`** – `FaultVfs` (Fehler-/Crashinjektion je `fsync`, Write, Rename), `ReferenceModel` (naives In-Memory-Orakel), `ManualClock`, deterministischer `Rng`, In-Memory-Ports.

**8.16 `lab`** – unabhängiges Erasure Lab (12.3): eigener Format-Parser nach Anhang F, eigene Krypto-Aufrufe, keine Abhängigkeit zu `store`, `engine`, `erasure` (AR-4). Enthält das Scanner-Modul für `ctx-verify scan`.

**8.17 `ctx-verify`** – Eigenständige Binärdatei. Zwei Unterbefehle:
- `ctx-verify report <bericht.json> --pub <report.pub> [--ledger <datei>]`: prüft kanonische Serialisierung, Ed25519-Signatur, Hash-Kette der Journal-Reduktion, Konsistenz der Zeugen (jeder Zeuge hat Positivkontrolle), Vollständigkeit der Restrisikoliste gegen Katalog (Anhang D), Zeitordnung. Ausgabe `VALID` / `INVALID(code)`; Exit-Code 0/2.
- `ctx-verify scan <datadir> --passphrase-fd N --target <ziel>`: unabhängige Feldprüfung (D-72) mit eigenen Parsern.
Kein Netzwerk, keine Secrets in Ausgaben.

---

## 9. Datenmodell

### 9.1 Löschcluster-Schlüsselraum (D-31, D-70)

Schlüssel (feste Länge bis auf `sub`, alles blinde Tags, D-09):

```
region(1) ‖ tenant_tag(16) ‖ coll_tag(16) ‖ cluster_tag(16) ‖ doc_tag(16) ‖ family(1) ‖ sub(u32) ‖ !commit_seq(u64)
```

|Region|Code|Inhalt|
|---|---|---|
|`DATA`|0x01|Dokumentfamilien unter Cluster|
|`META`|0x03|Collection-Katalog (`name_tag`), Zähler|
|~~`LOC`~~|~~0x02~~|entfällt (D-70), Code bleibt reserviert|

`cluster_tag` = `subject_tag` bei zugeordneten Dokumenten, sonst `0x00¹² ‖ doc_tag[0..4]`. Ein Subject-Ziel ist damit **ein** Bereich `DATA‖tenant‖coll‖cluster`; ein Dokument-Ziel ein Bereich bis `doc_tag`; ein Collection-Ziel ein Bereich bis `coll_tag`; ein Mandant ein Bereich bis `tenant_tag`. Ein Betroffener über mehrere Collections liegt in je einem Bereich je Collection (`resolve` fasst sie zusammen). `!commit_seq` (bitweises Komplement, big-endian) lässt die neueste Version zuerst sortieren und macht `seq` ohne DEK lesbar (D-74). Matcher: `Range{lo,hi}` fester Länge oder `Exact` (D-10).

### 9.2 Familien

|Family|Code|Inhalt|
|---|---|---|
|`Doc`|0x01|Chunks und Metadaten (Envelope)|
|`DocMap`|0x02|Chunk-Zuordnung, Vektor-Slot-Nummern|
|`Manifest`|0x03|Versionen, Größen, Identität (`DocRef` verschlüsselt)|
|~~`Locator`~~|~~0x04~~|entfällt (D-70)|
|`TextBuf`|0x05|noch nicht segmentierte Textdeltas (9.7)|

### 9.3 Tags

`tag = BLAKE3-keyed(K_idx, label ‖ eingaben)[0..16]` mit festen Labels (Anhang C). `K_idx` wird aus `K_master` abgeleitet; ohne `K_master` sind Tags nicht berechenbar. Eingaben werden längenpräfigiert (`u32`-Länge je Feld), damit `(a‖bc)` und `(ab‖c)` nicht kollidieren. Kollisionen: bei 128 Bit praktisch ausgeschlossen, dennoch geprüft (`Conflict::IdCollision`, D-08).

### 9.4 Record-Envelope (D-32)

```
Envelope(v2) := version(1)=2 ‖ alg_id(1) ‖ doc_tag(16) ‖ nonce(12) ‖ ciphertext ‖ aead_tag(16)
AAD := "ctx/rec/v2" ‖ key_bytes            // key_bytes inkl. !commit_seq-Suffix (9.1)
DEK := HKDF-SHA256(ikm=K_unit, salt=SHA-256("ctx/dek-salt/v2" ‖ salt_T ‖ salt_C ‖ salt_S ‖ salt_D), info="ctx/dek/v2" ‖ doc_tag)
```

Der Wert enthält Nutzerkennungen (`DocRef`, Metadaten) nur im Ciphertext. Block-AEAD der SSTable ist die zweite Hülle. Fehlt ein Slot der Ahnenkette (`T`, `C`, `S`, `D`), ist der DEK unableitbar und der Envelope nicht mehr öffenbar (`Erased`).

### 9.5 Lebenszyklus und MVCC

Insert/Update schreibt neue Version; `soft_delete` schreibt Tombstone. `history_horizon` (D-11) begrenzt Versionen; Kompaktion entfernt Versionen oberhalb `min_read_snapshot` **und** älter als Horizont. Das ist keine Löschaussage (D-51). `erase` entfernt **alle** Versionen: Purge verwirft Memtable-Einträge, flusht, entfernt WAL-Segmente bis zum Flush-Punkt als Ganzes (D-73) und schreibt betroffene SSTables ohne Ziel-Records neu (11.4).

### 9.6 Subject-Zuordnung

`subject_tag = tag("ctx/subj/v1", tenant_tag ‖ subject_ref)`. Teil der Dokumentidentität, damit unveränderlich (D-40, T-SUBJ-2). Dokumente ohne Betroffenen sind nur per `Document`, `Collection`, `Tenant` löschbar. Die Zuordnung eines Dokuments zu einem Betroffenen ist Betreiberpflicht (Contextra erkennt Personenbezug nicht, D-27).

### 9.7 Textindex (D-20, D-49, D-69, D-71, D-81)

- **Textpuffer:** neue und geänderte Dokumente liegen zunächst als `TextBuf`-Records im Cluster des Dokuments (löschbar wie Dokumente, Klasse K `full`).
- **Segmente** (`text/<tenant_tag>/<coll_tag>/<file_id>.seg`, D-75): unveränderlich, Block-AEAD unter zufälligem Dateischlüssel (D-85), Zielgröße adaptiv (D-71b). Inhalt (F.7): Header (`bucket`, `bucket_count`, `analyzer_id`, `analyzer_version`, `lang`, `positions`), Termwörterbuch (Terme **nur** als `term_tag`, nie Klartext), Postings je Term als Delta-codierte `doc_ord`-Listen in 128er-Blöcken mit Block-Max-Metadaten, optional Positionen, Dokumenttabelle (`doc_ord → doclen, binding`), Lösch-Bitmap, Segmentstatistik.
- **Entkopplung (D-69):** Postings tragen nur segmentlokale `doc_ord`. Die Bindung `doc_ord → (cluster_tag, doc_tag)` liegt je Dokument als Envelope unter dem DEK des Dokuments (AAD: Segment-`file_id` ‖ `doc_ord`). Nach Schlüsselvernichtung sind Postings `unlinked`: Struktur sichtbar, Zuordnung nicht herstellbar.
- Ein Dokument liegt zu jedem Zeitpunkt in genau einem Segment oder im Puffer (D-49). Update = Eintrag im alten Segment per Bitmap als gelöscht markieren und neu in den Puffer; Merge baut Segmente neu.
- **Erase:** alle Segmente, die Zieldokumente enthalten (`resolve` über Bindungen und Bucket), werden im **ersten Purge-Fenster** ohne die Zieldokumente neu geschrieben; alle Statistiken (df, Längensummen, Blockmaxima) werden exakt neu berechnet (E-L5); das alte Segment geht durch Dateischlüssel-Vernichtung und Unlink (D-85). Der Bericht führt `segments_rewritten` und `bytes_rewritten`. Aufwand ∝ Σ Größe der berührten Segmente (D-71c, BM-ERASE-WA-TEXT).
- **Analyzer** (D-81): `de/1` und `und/1`; je Collection unveränderlich; Goldenset für Komposita, Umlaute, `ß` (T-TEXT-ANALYZER-1); Phrasensuche über Positionen (T-TEXT-PHRASE-1).
- **Suche:** Query-Terme → Analyzer → `term_tag`, Postings pro Segment mit Block-Max-WAND, BM25 mit globalen Statistiken aus Segmentsummen (`k1 = 1,2`, `b = 0,75`), Kandidatenstrom → Bindung nur für Top-k-Kandidaten entschlüsseln → ReadGate-Filter mit Nachladen (D-62). Fehlt der DEK einer Bindung, ist der Treffer gelöscht und wird übersprungen.
- **Grenzen (ausgewiesen):** Keine Präfix-/Wildcard-Suche (Terme nur als Tags). Wörterbuchangriff gegen `term_tag` setzt `K_idx` voraus (`K_master`); deshalb `TextIndexUntilPurge` bis zum Rewrite.

### 9.8 Vektorindex (D-18, D-19, D-77, D-86)

HNSW (`M = 16`, `efConstruction = 200`, Standard `efSearch = 64`), Arena mit festen Slots (D-86: je Slot alle Repräsentationen zusammenhängend). Slot-Tabelle `slot → (cluster_tag, doc_tag, chunk)` im Speicher; im Snapshot als Envelope unter dem DEK des Dokuments (Vektor **und** Bindung). Löschung nach D-77: (1) vor `Gated` Surrogat in allen Repräsentationen; (2) im Purge-Fenster Batch-Kantenreparatur (ein Durchlauf), Slots in die Freiliste; (3) deterministischer Rebuild nach Quote. Suchstrom: gegatete Kandidaten werden verworfen, `ef` verdoppelt sich bis `ef_max = 4096` (D-62). Snapshot (F.8): Kanten unter dem Dateischlüssel (D-85), Payload je Slot unter DEK. Arena-Seiten: `MADV_DONTDUMP`, `mlock` best effort (D-63).

### 9.9 Dateiformate (Rahmen, D-42)

Jede Datei beginnt mit dem Rahmenkopf aus Anhang F.1: `magic(8) ‖ format_version(u16) ‖ kind(u8) ‖ alg_ids(u8[4]) ‖ file_id(16) ‖ …`. Unbekannte `format_version` → `Corrupt` (fail-closed, P3: keine Legacy-Pfade). Die byte-genauen Layouts stehen in Anhang F dieser Datei und sind normativ; `lab` und `ctx-verify` implementieren sie unabhängig (AR-4).

---

## 10. Kryptografie und Schlüsselverwaltung

### 10.1 Hierarchie

```
passphrase ──Argon2id(slot_salt, m,t,p)──▶ KEK ──unwrap(AES-256-GCM-SIV, AAD nach D-52)──▶ K_master (32 B, zufällig)
K_master ─HKDF─▶ K_enc   ("ctx/enc/v1")   ─HKDF(kind‖file_id)─▶ K_file   (Steuerdateien)
         │                                 ─HKDF("ctx/fkmask/v1"‖file_id)─▶ Maske für F-Slots (D-85)
         ─HKDF─▶ K_idx   ("ctx/idx/v1")   (Tags)
         ─HKDF─▶ K_unit  ("ctx/unit/v1")  ─HKDF(salt = SHA-256(salt_T‖salt_C‖salt_S‖salt_D), info = doc_tag)─▶ DEK
         ─HKDF─▶ K_sign  ("ctx/sign/v1")  (Ed25519-Seed, nur abgeleitet, nie gespeichert, D-80)
         ─HKDF─▶ K_wal   ("ctx/wal/v1")   (HMAC-Kette)
         ─HKDF─▶ K_commit(epoch)          ("ctx/commit/v1" ‖ epoch)
         ─HKDF─▶ K_ledger("ctx/ledger/v1") (Ledger-Authentisierung)
Datendateien: zufälliger Dateischlüssel (OS-CSPRNG), im Tresor als F-Slot (D-85)
```

Domänentrennung durch eindeutige Labels; jedes Label steht in Anhang C. Die Hierarchie hat zwei **unabhängige** Löschanker: die Salts `T/C/S/D` (Inhalte) und die `F`-Slots (Dateistruktur).

### 10.2 Keyslots und HEADER (D-52)

- `HEADER.a` / `HEADER.b`: abwechselnd geschrieben, je mit Generationszähler und Prüfsumme (F.2; `T-HDR-1`: Korruption eines Exemplars führt nie zu Totalverlust).
- 1..4 Slots: `slot_id, anchor_id, kdf_id, m, t, p, slot_salt, nonce, wrapped_master(48 B), status`. Die Parameter werden **vor** der Ableitung gegen D-06 geprüft.
- `change_passphrase`: neuen Slot erzeugen → entpacken und prüfen → HEADER schreiben (A/B, fsync, Verzeichnis-fsync) → alten Slot nullen (O(1)). Restrisiko `OldKeyslotRemanence`.
- Falsche Passphrase: `Auth` ohne Hinweis, welcher Teil falsch war; konstante Zeitgrenze durch Argon2-Kosten; keine Datei wird verändert (FM-10).

### 10.3 Nonces und AAD

Record-Envelope und Keyslot-Wrap: 96 Bit aus dem OS-CSPRNG. Block-AEAD: `0x00000000 ‖ u64(Blockzähler)` je Dateischlüssel (D-60), höchstens 2³² Blöcke je Datei, dann neue Datei. AAD bindet Dateityp, `file_id`, Blockindex (und bei Appendlogs den Vorgänger-MAC), sodass Umordnen und Kreuz-Kopieren scheitern (T-ENC-2). Der Kopf-Tag nutzt den reservierten Nonce-Präfix `0xFFFFFFFF`.

### 10.4 WAL-Integrität

HMAC-Kette über Frames (`mac_n = HMAC(K_wal, mac_{n-1} ‖ len ‖ ct ‖ aead_tag)`, F.4). Recovery verwirft ab erstem fehlerhaften Frame **nur dann** automatisch, wenn der Rest der Datei Nullen/unbeschrieben ist (Torn Write); sonst `Corrupt` und Start verweigert (T-WAL-1). Segmente werden nie umgeschrieben (D-73); die Kette setzt nach `WalFloor` mit `tail_mac` fort (T-WAL-2). Es gibt keinen Downgrade-Schlüssel (D-05).

### 10.5 Algorithmus-Wechsel (R15)

`alg_id` in jedem Header; neue Dateien verwenden den aktuellen Satz, alte bleiben lesbar bis zur Kompaktion; Neubewertung jährlich (Release-Checkliste, H8). `alg_id = 2` ist reserviert (D-80).

---

## 11. Löschkern

### 11.1 Ziele und Umfangsauflösung

Je `EraseTarget` ruft jede `ErasureLayer` `resolve` auf; das Ergebnis (Schlüsselbereiche, `D`-Slot-Mengen, Segmentlisten, Slot-Mengen, Dateilisten) wird verschlüsselt in `erasure/scope/<erasure_id>.scp` persistiert **vor** dem Gate (damit Recovery denselben Umfang kennt). Für `Document` mit kleinem Umfang wird er in den Journal-Eintrag eingebettet (kein extra `fsync`). Der Umfang wird zum Zeitpunkt `gated_seq` fixiert: alles `≤ gated_seq` ist im Umfang; Schreibzugriffe danach werden abgewiesen (D-57). `resolve` ist O(Schichtstruktur), nicht O(Zieldokumente), soweit das Ziel ein Schlüsselbereich ist; zielgrößenabhängige Auflistungen (z. B. alle Textsegmente eines Betroffenen) erfolgen im Purge, nicht vor `Gated`.

### 11.2 Saga (Zustandsmaschine)

```
Requested ─▶ Gated ─▶ KeyDestroyed ─▶ Purging ─▶ Verifying ─▶ Completed
    │           │            │            │            │
    └───────────┴────────────┴─── Pending(reason) ◀────┘      (jeder Schritt idempotent)
Refused (terminal, vor Gate)
```

Schritte und Dauerhaftigkeit (Normalpfad bis `Gated`: 4 `fsync`, D-58; bei gleichzeitigen Anfragen teilen sich diese einen Gate-Log-`fsync`, D-78):

1. Validierung, `RetentionAttestation`, Journal `Requested+Intent` mit eingebettetem Umfang (fsync #1; I-8).
2. Gate: `ErasedSet::add_batch` unter exklusiver Commit-Barriere → Gate-Log-Append mit `gated_seq` (fsync #2; D-57).
3. `on_gate` je Schicht: Vektor-Surrogat (D-77), DEK-Cache evicten; Schlüsselvernichtung: Slot der höchsten Ebene nullen (`T`/`C`/`S`/`D`), `sync_data` (fsync #3), Fehlen lesend prüfen (D-79).
4. Journal `KeyDestroyed` (fsync #4) → Rückgabe `Gated`.
5. **Purge-Fenster** (`batch_window`, D-39; `Timing::Now` sofort), in dieser Reihenfolge:
   a. Snapshots des Mandanten widerrufen (D-61).
   b. **Textsegment-Rewrite im ersten Fenster** (D-69); Dateischlüssel der alten Segmente vernichten (D-85).
   c. Memtable-Drain des Ziels, **Memtable-Flush**, WAL-Segment an Sequenzgrenze versiegeln, **WAL-Segmente bis zum Flush-Punkt entfernen** (`WalFloor`, D-73).
   d. SSTable-Rewrite betroffener Dateien (Nicht-Ziel-Records bytegleich als Envelopes übernommen, neuer Dateischlüssel); nach dauerhaftem Manifest-Swap `F`-Slots der Altdateien nullen, dann `unlink` (I-17, D-85).
   e. Vektor-Batch-Kantenreparatur, Freiliste (D-77); Rebuild nach Quote.
   f. `D`-Slots der Zielmenge entfernen (zielgrößenabhängig), `F`-Waisen prüfen (D-83).
   g. Cache-Eviction (Block-, DEK-, Dokumentcache).
6. Messung (12): Zeugen aller Schichten, Bericht signieren, Ledger-Eintrag (11.8), Journal `Completed`, Gate-Kompaktion.

**Recovery:** Beim Start liest der Löschkern das Journal; jede nicht `Completed`-Löschung wird beim nächsten Schritt fortgesetzt; Gate-Log wird **vor** dem Öffnen von Lesepfaden geladen (fail-closed, I-2). Fehlt der Journal-Eintrag zum Gate, gilt das Gate trotzdem (konservativ) und die Löschung wird vollendet; fehlt das Gate-Log, wird es aus dem Journal rekonstruiert (FM-9). `Pending` trägt `PendingReason` (`Io`, `SlaExceeded`, `LayerFailed(id)`, `WitnessMissing`, `DiskReserve`).

### 11.3 Gate (`ReadGate`)

Gate-Varianten: `Erased` und `Restricted`. Prüfung in: jedem Lesepfad, jedem Suchstrom, `scan`, Zeitreisen, Commit-Phase (Schreibabweisung), Kompaktion (darf gegatete Bereiche nicht „wiederbeleben“). Datenstruktur: sortierte Bereichsmenge im Speicher (Intervallbaum), O(log n) je Prüfung; Persistenz: `gates.log` (AEAD, Hash-Kette). `Restricted` verhindert Lesen/Suchen/Schreiben, vernichtet nichts, wird von der Kompaktion **nie** verworfen (I-18); Aufhebung nur durch `lift_restriction` mit Journal-Eintrag. Die Kompaktion verwirft Bereiche der Variante `Erased` beim Schreiben der Ausgabe (D-78); das ersetzt den Zeugen nicht.

### 11.4 Purge

- **Drain:** Memtable- und Puffer-Einträge im Umfang entfernen.
- **WAL (D-73):** nie umschreiben. Nach Memtable-Flush gilt der Flush-Punkt `F_seq`: Segmente, deren letzte Sequenz `≤ F_seq` ist, werden vollständig entfernt (Dateischlüssel vernichten, `unlink`). Das aktive Segment wird zuvor an einer Sequenzgrenze versiegelt; ein neues Segment beginnt mit `prev_mac = tail_mac` (`WalFloor`-Eintrag im Manifest). Die Dauer ist begrenzt durch die Flush-Zeit; sie ist ein Bestandteil von BM-ERASE-BATCH.
- **Rewrite:** je betroffener SSTable eine neue Datei ohne Zieleinträge (Kandidaten über Dateibereich im Manifest; durch den Cluster-Schlüsselraum meist 1–2 Dateien je Level). Neue Datei: schreiben, `sync_data`, Manifest-Log-Eintrag (fsync), `F`-Slot der alten Datei nullen (fsync), dann Datei entfernen und Verzeichnis-`fsync`. `scrub` nur nach D-83.
- **Verifikation vor Unlink:** neue Datei wird gelesen und gegen Matcher geprüft (MK1, zuletzt: gesamte Messung nach Abschluss).
- **Plattenreserve:** Rewrite reserviert vorab (D-65); fehlt Platz, `Pending(DiskReserve)`, Gate bleibt aktiv.
- **Barrieren:** Purge exklusiv gegen Kompaktion (5.4); die Kompaktion bricht kooperativ ab (I-16). Fehler beim Rewrite lässt die alte Datei unverändert (kein Teilzustand, T-CRASH-2).
- **Mandanten- und Collection-Ziele:** Dateien, die ausschließlich Zielbereiche enthalten, werden ohne Rewrite entfernt (Dateischlüssel vernichten, `unlink`); Randdateien werden umgeschrieben.

### 11.5 Tresor-Protokoll (D-33, D-79, D-85)

`vault/vault.dat`: plattenresidente Open-Addressing-Hash-Tabelle fester 80-Byte-Slots (Layout F.3), Lastfaktor ≤ 0,7, lineares Sondieren; Slot-Position = `BLAKE3(slot_type ‖ tag) mod capacity` (öffentlicher, unkeyed Hash genügt, die Tags sind bereits keyed). Es gibt **keinen** vollständigen RAM-Index; ein LRU-Cache hält heiße Slots. Slot-Typen: `T`, `C`, `S`, `D` (Salts, D-79) und `F` (maskierte Dateischlüssel, D-85).

- **Anlegen:** fehlende Ahnen-Slots (`T`, `C`, `S`) und `D` schreiben und gruppenweise `sync_data` **vor** den WAL-Frames (I-9); `F` vor dem ersten dauerhaften Block der Datei (I-9b). So existiert nie ein Record ohne Schlüssel.
- **Löschen (Vernichten):** Slot auf Nullen setzen, bis auf das Statusbyte `0x02` (Grabstein, hält die Sondierkette), `sync_data`, Slot lesen und **Nullen verifizieren** (T-VAULT-1); bei Fehler `Pending` mit Neuöffnen der Datei (D-55).
- **Ahnenfolge:** Ein Ziel vernichtet genau einen Slot der höchsten Ebene. Darunter liegende `D`-Slots werden vom Purge entfernt (Abfall, kein Pfadbestandteil bis `Gated`).
- **Wachstum:** Rehash in eine neue Datei bei Lastfaktor 0,7 (atomarer Austausch); Grabsteine werden dabei nicht übernommen, die alte Datei geht durch `scrub`-freies Unlink (nach D-83) – die Remanenz alter Slotinhalte ist als `WrappedKeyRemanence` ausgewiesen.
- **Kapazität:** `max_docs_per_store = 5·10⁷` (D-87); Speicherbedarf ≈ 114 B je Dokument bei Lastfaktor 0,7 (≈ 5,7 GB bei Maximum, [A]).
- **Open:** Kopf und Stichproben prüfen, kein Vollscan (BM-VAULT-OPEN); Waisen (`D`/`F` ohne Gegenstück) werden bei der Recovery genullt (D-83, T-CRASH-5).

Restrisiko `WrappedKeyRemanence` (Flash/COW) wird im Bericht geführt (nach D-53 durch Hardware-Anker ersetzbar).

### 11.6 Journal

Hash-Kette, AEAD, `fsync` je Eintrag (Gruppen bei Last). Einträge enthalten nur `ErasureId`, Zustand, Zeit, Commitments (`BLAKE3-keyed(K_commit(epoch), subject ‖ tenant)`), Zähler, Zeugen-Hashes, Retention-Code, eingebetteten Umfang (nur Bereichs-Tags). Nach `Completed` wird das Journal reduziert (Zwischenzustände entfernt, R13). Epochen-Rotation (D-48): nach `commit_epoch` neuer Epochenschlüssel, alte Commitments in Journal und Bericht werden entfernt. **Das Ledger folgt dieser Rotation nicht** (D-74).

### 11.7 Bericht

Kanonisches JSON (F.10: sortierte Schlüssel, keine Gleitkommazahlen, UTF-8 NFC), Ed25519 (D-13). Felder: `report_version`, `store_id`, `erasure_id`, Ziel-Typ (nicht Kennung), Commitment, Zeiten (`requested_at`, `gated_at`, `key_destroyed_at`, `completed_at`), `sla` und Verletzung, Retention-Code, **`field_measurement`** (je Schicht `LayerWitness`: MK1, MK2, MK3-Feld) und **`lab_evidence`** (Hash des Release-Evidenzbündels, D-72), Klassenverband G/K/P mit `key_coverage` je Schicht (12.5), `segments_rewritten`, `bytes_rewritten`, Restrisiken (Katalog Anhang D), Pflichtsatz zur Signatur (E-L11), Hinweis „keine Rechtsberatung“, Prozesshärtungsstatus. **Keine** Inhalte, Schlüssel oder Klartext-Kennungen (P13).

### 11.8 Ledger und Restore-Guard (D-37, D-74)

**Ledger:** abgeschlossene Löschungen als `(commitment, range_set, gated_seq, completed_at)`; `range_set` = Liste von Schlüsselbereichen (`lo`, `hi`, je Tags fester Länge) der Löschung. Datei **außerhalb** des Datenverzeichnisses über `LedgerSink`, Hash-Kette plus HMAC (`K_ledger`, F.9). Lebensdauer: `backup_retention` (nicht `commit_epoch`).

**Erkennung beim Öffnen:**
1. **Zeitregression:** `store.current_seq < max(gated_seq im Ledger)` ⇒ Restore erkannt, ohne Scan.
2. **Wiederauftauchen:** Datei im Zielbereich mit `min_seq ≤ gated_seq` und Record mit `seq ≤ gated_seq` (Dateirestore, Einzelrestore). Records mit `seq > gated_seq` sind legitime Neuanlage und lösen nichts aus.
3. Bei Treffer: Store wird **nicht freigegeben**: Status `Recovering`, automatische Wiederholung der Löschungen (`RestoreReplay`), Bericht mit Vermerk.

**Sequenz:** `next_seq = max(current_seq, ledger_floor) + 1`; sie bleibt über Restores monoton.

**Fehlendes Ledger** (neuer Rechner): `health()` meldet `Degraded: LedgerMissing`; der Betreiber bestätigt per `acknowledge_ledger_reset` (Journal-Eintrag). Ledger-Einträge jenseits `backup_retention` werden entfernt. Zweckbindung „Restore-Schutz“ (E-L4); die Begründung der Dauer gehört in die DSFA des Betreibers (Anhang G).

### 11.9 Rechnung und Gate G0: Warum Löschcluster-Schlüsselraum (Grundlage D-02, D-31, D-70, D-76) **[A/E]**

**Rechnung.** Datenbank mit N = 150 SSTables, ein Dokument mit m = 45 Schlüsseln (Familien × Chunks × Versionen), Schlüssel hash-verteilt: `P(Datei betroffen) = 1 − (1 − 1/N)^m ≈ 1 − e^(−m/N) = 1 − e^(−0,3) ≈ 26 %`. Ein Purge schriebe ≈ 39 von 150 Dateien um (Write Amplification ∝ DB-Größe). Mit Cluster-Schlüsselraum liegt das Ziel pro Level in höchstens 1–2 Dateien (7 Level ⇒ ≤ 14 Dateien, praktisch 3–6). Subject-Ziele skalieren mit der Zielgröße, ein Mandant mit Dateien, die ihn ganz enthalten, ohne Rewrite. Batch (D-39): B Löschungen im Fenster berühren die Vereinigung der Dateimengen, nicht die Summe. Die Locator-Region der Vorfassung hätte diese Rechnung für Subject-Ziele neu eröffnet (D-70).

**Gate G0 (verbindlich, als Prototyp mit D-70 und D-71 gefahren).** BM-ERASE-WA misst Dateien und Bytes je Erase auf 1 GiB und 10 GiB Testbestand für drei Zielarten (Einzeldokument, Subject mit 10³ Dokumenten, Collection). Bestehensschwellen (D-76):

- `bytes_written ≤ 3 × Zielbytes + L_t × 32 MiB` (`L_t` = Zahl der Level mit überlappender Datei, ≤ 7),
- **absolute Obergrenze 512 MiB** je isoliertem Einzeldokument-Erase,
- Steigung `bytes(10 GiB) / bytes(1 GiB) ≤ 1,25` (nicht wachsend mit der DB-Größe),
- Tresor und Gate zählen nicht zu `bytes_written`, sind aber in BM-ERASE-LAT erfasst.

**Fallbackleiter (verbindlich; jede Stufe ist Neufassung der genannten Abschnitte, nie stilles Weiterlaufen):**

|Stufe|Auslöser|Maßnahme|Neu zu fassen|Folge im Bericht|
|---|---|---|---|---|
|0|G0 bestanden|Weiter wie spezifiziert|–|–|
|1|Schwelle gerissen, aber Steigung ≤ 1,25|**Dateigrenzen an Clustergrenzen ausrichten** (Kompaktion schneidet Ausgabedateien an `cluster_tag`-Grenzen) und Zielgröße Level ≥ 1 auf 16 MiB senken|9.1, 11.4, D-76|–|
|2|Steigung > 1,25 oder 512 MiB gerissen|**Partitionierter LSM:** je Mandant (oder Hash-Partition über `tenant_tag`) ein eigener LSM-Baum mit eigener Manifest- und WAL-Kette; Mandanten-Ziele werden Verzeichnis-Löschungen|4.1 (D-02, D-31), 5.6, 9.1, 11.4, F.4, F.5|–|
|3|Stufe 2 reicht nicht für Subject-Ziele|**Schlüsselvernichtung als Primärmittel, Rewrite entkoppelt:** Subject-Purge nur noch lazy bei Kompaktion, Gate und Schlüssel bleiben sofort; Abschluss wird `Completed(KeyOnly)` bis die Kompaktion alle Zieldateien erfasst hat (Frist `erase_sla`)|4.1, 11.2, 11.4, 12.5|`PurgeScalesWithDb` entfällt, `PurgeDeferred` neu|
|4|Eigenbau-LSM nicht tragfähig|**Speicheradapter** (z. B. `redb`) hinter `StorageEngine`; Löschung = Schlüsselvernichtung plus Adapter-Löschung; physische Entfernung skaliert mit der DB|4.1, 5.1, 8.8, 9.1, 11.4, 11.9|`PurgeScalesWithDb` im Bericht|

Der Wechsel der Stufe ist eine Entscheidung des Gates G0 mit Messprotokoll (WP-3.5); Anhang H wird dabei fortgeschrieben.

### 11.10 Mandanten-Offboarding (D-38)

`EraseTarget::Tenant`: Schlüsselvernichtung über den `T`-Slot (O(1), D-79); Bereich `DATA‖tenant_tag` und `META‖tenant_tag`; Dateien, die ausschließlich Mandantendaten enthalten, werden direkt entfernt (Dateischlüssel vernichten, `unlink`); Randdateien umgeschrieben; Verzeichnisse `text/<tenant_tag>/` und `vector/<tenant_tag>/` werden entfernt (D-75); die `D`/`C`/`S`-Slots des Mandanten räumt der Purge ab (Bereichsscan über das Tag-Präfix im Tresor, in Blöcken, abbrechbar). Fremde Mandanten: bitgleich auf Record-Ebene (T-ISO-1, T-TENANT-1), Leser fremder Mandanten bleiben ungestört (D-61).

---

## 12. Messung und Nachweis

### 12.1 Messklassen (D-35, D-72)

|Klasse|Methode|Aussage|Wer|
|---|---|---|---|
|**MK1** Struktur|Strukturscan aller Dateien: Matcher (Bereiche, Tags, `doc_tag`) gegen die entschlüsselte Struktur (Manifeste, SSTable-Indexblöcke, Segmentwörterbücher, Slot-Tabellen) und gegen Rohbytes der Steuerdateien|Kein Eintrag des Ziels in einer Struktur des Datenverzeichnisses außerhalb des Tresors|Produktion (Feld) und Lab|
|**MK2** Rohscan|Rohscan aller Dateien (inkl. `tmp/`, alte Segmente, WAL, Snapshots, `scope/`, Berichte) auf **Klartext-Needles des Betreibers** und bekannte Muster (Tag-Bytes, Envelope-Header, Salt-Bytes)|Kein Klartextfragment, kein Tag-/Envelope-Fragment außerhalb verschlüsselter Blöcke|Produktion (Feld) und Lab|
|**MK3** Schlüsselprobe|**Feld:** Schlüsselabwesenheit: der Slot der höchsten Ebene ist genullt, die Ableitung des DEK scheitert (`Erased`), kein `F`-Slot der Zieldateien existiert. **Lab:** mit dem **vor der Löschung gesicherten** DEK der Fixtur wird versucht, verbliebene Envelopes und Blöcke zu öffnen; Erwartung: nichts öffenbar|Feld: Der Schlüssel ist weg. Lab: Verbliebene Chiffrate sind mit dem alten Schlüssel nicht öffenbar|Feld: Produktion. Lab: nur Lab (der Schlüssel existiert in Produktion nicht mehr)|

Ein Zeuge ohne **Positivkontrolle** (der Scanner findet ein bekanntes, eingestreutes Muster in einer Kontrolldatei) ist ungültig (HR-10, T-RAW-2). Ein Rohscan auf Klartext allein zählt nicht. **Grenze:** In einem verschlüsselten Store ist ein Rohscan auf Tag-Bytes für Dateischichten systematisch leer; MK2 belegt dort „kein Klartext-Leck“, MK1 belegt die Strukturaussage, MK3 die Schlüsselaussage. Die Produktions-Eigenmessung ist Selbstattestierung (gleiche Decoder wie der Purge); sie ist unabhängig nur über `ctx-verify scan` oder das Lab (12.4).

### 12.2 `LayerWitness`

`{layer, class (MK1|MK2|MK3), method_id, scanned_files, scanned_bytes, matches: 0, control_ok: true, duration, hash}`. `LayerWitness` ist nur aus einem `Scanner<Armed>` konstruierbar (D-72); `Scanner<Armed>` entsteht nur durch eine erfolgreiche Positivkontrolle (Typzustandsmuster, `#[must_use]`, Konstruktoren privat). `matches ≠ 0` ⇒ Status `Pending(WitnessMissing)`, nie `Completed`.

### 12.3 Erasure Lab

`contextra-lab` ist ein separates Programm/Crate (AR-4): eigener Parser aller Formate (Anhang F), eigene Krypto-Aufrufe, keine Produktions-Decoder. Funktionen: Fixturen erzeugen (Dokumente mit bekannten Mustern und gesicherten DEKs), Löschung über die öffentliche API auslösen, danach MK1–MK3 unabhängig ausführen, Differenzprüfung (T-DIFF-1: Nicht-Ziel-Daten auf Record-Ebene bitgleich), Restore-Szenarien, Mutationstests (absichtlich gebrochene Purge-Implementierungen müssen erkannt werden, T-MUT-1). Das Lab enthält das Scanner-Modul, das `ctx-verify scan` verwendet.

### 12.4 Erhebung des Messstands

Das Lab läuft in CI (EV3: reproduzierbar auf unabhängigem Runner) und als Teil der Gates G-D/G-H (EV4 mit Mutationsprüfung). Das **Release-Evidenzbündel** (Lab-Protokolle, Mutationsergebnisse, Toolchain-Version, Build-Hash) wird gehasht und im Bericht als `lab_evidence` referenziert. In Produktion führt die Bibliothek MK1, MK2 und MK3-Feld selbst aus (`field_measurement`). Eine Produktaussage über Löschung stützt sich auf EV4 (Lab), nicht auf die Eigenmessung; der Betreiber kann die Feldmessung mit `ctx-verify scan` unabhängig wiederholen.

### 12.5 Klassenverband G, K, P (D-36, D-69)

|Klasse|Aussage im Bericht|Voraussetzung|
|---|---|---|
|**G** (Gate)|Ab Zeitpunkt `gated_at` liefern alle Lesepfade `Erased`|T-GATE-1, T-VER-1|
|**K** (Key)|Schlüssel der höchsten Ebene vernichtet, Fehlen geprüft; **`key_coverage` je Schicht**|T-VAULT-1..4, T-KEYCOV-1|
|**P** (Purge)|Daten im Datenverzeichnis, in Indexen und Versionen entfernt; MK1/MK2/MK3-Zeugen je Schicht|T-RAW-1..6|

**`key_coverage` (D-69):**

|Schicht|Nach `KeyDestroyed`|Danach|
|---|---|---|
|`doc_store`|`full`|–|
|`vector_payload`|`full` (Vektor und Bindung unter DEK)|–|
|`text_postings`|`unlinked`|`TextIndexUntilPurge` bis Segment-Rewrite (erstes Purge-Fenster)|
|`vector_edges`|`none`|`IndexTopologyInference` bis Rebuild (D-77)|
|`file_structure`|`none` bis zur Dateischlüssel-Vernichtung|Strukturdaten der Altdateien sind nach D-85 wertlos|

Jede Klasse wird einzeln als `reached/not_reached` ausgewiesen; es gibt keine zusammenfassende Note und keine Rangfolge (die Aussagen sind unabhängig, P17).

---

## 13. Invarianten und Fehlermodi

### 13.1 Invarianten

|ID|Invariante|
|---|---|
|I-1|Kein Schreibzugriff in einen gegateten Bereich nach persistiertem Gate.|
|I-2|Ein gegateter Bereich ist in jedem Lesepfad `Erased`, auch nach Neustart (Gate-Log wird vor Lesepfaden geladen).|
|I-3|Nicht-Ziel-Daten bleiben auf **Record-Ebene** bitgleich (Ciphertext-Envelopes unverändert; ausgenommen Manifest, Seq und die Dateihülle: Block-AEAD, Dateischlüssel, Offsets). Der WAL wird nie umgeschrieben (D-73).|
|I-4|`Completed` ⇒ alle Schichten haben gültigen Zeugen.|
|I-5|Kein Klartext von Nutzerdaten auf Platte (außerhalb Prozessspeicher).|
|I-6|Jeder Header enthält `format_version` und Algorithmus-IDs.|
|I-7|Hash-Ketten (WAL, Journal, Gate-Log, Ledger) sind lückenlos; Bruch ⇒ `Corrupt`. Ausnahme: dokumentierte Fortsetzung nach `WalFloor`.|
|I-8|Journal-Eintrag `Requested` ist vor dem Gate dauerhaft.|
|I-9|Tresor-Slot (`T`/`C`/`S`/`D`) ist dauerhaft **bevor** ein Record unter dem DEK im WAL dauerhaft wird.|
|I-9b|Der `F`-Slot einer Datei ist dauerhaft **vor** dem ersten dauerhaften Block der Datei.|
|I-10|Ein `fsync`-Fehler wird nie auf demselben Handle wiederholt (D-55).|
|I-11|Kein Fehlertext enthält Nutzerinhalt (HR-4).|
|I-12|`min_read_snapshot` hat keinen „alles sichtbar“-Sentinel.|
|I-13|Ein Dokument gehört zu höchstens einem Betroffenen, unveränderlich (Teil der Identität, D-40).|
|I-14|Mutierende Operationen sind cancel-safe (Spawn + oneshot).|
|I-15|Jede Löschung ist idempotent und nach Neustart fortsetzbar.|
|I-16|Keine Hintergrundoperation hält eine wartende Löschung (Purge-Barrier, Commit-Barriere) länger als 1 s auf (kooperativer Abbruch).|
|I-17|Der `F`-Slot einer ausgeschiedenen Datei wird nach dauerhaftem Manifest-Swap und **vor** `unlink` genullt.|
|I-18|Die Kompaktion verwirft nie Daten eines Bereichs der Variante `Restricted`.|
|I-19|`LayerWitness` ist nur aus `Scanner<Armed>` konstruierbar (D-72).|
|I-20|Die Sequenz ist über Restores monoton (`next_seq = max(current_seq, ledger_floor) + 1`).|

### 13.2 Fehlermodi (FM)

|ID|Fehlermodus|Reaktion|Test|
|---|---|---|---|
|FM-1|Absturz zwischen Gate und Key-Nullen|Recovery nullt Slot, setzt fort|T-CRASH-1|
|FM-2|Absturz im Rewrite|Alte Datei bleibt gültig; Rewrite wird wiederholt; Waisen in `tmp/` entfernt|T-CRASH-2|
|FM-3|Platte voll beim Purge|Gate bleibt, `Pending(DiskReserve)`, Schreibzugriffe `Limit::DiskReserve`|T-CRASH-3|
|FM-4|Abbruch des Aufruferfutures|Task läuft zu Ende|T-CANCEL-1|
|FM-5|`fsync`-Fehler|Datei fatal, `Poisoned`, Neuöffnen|T-FSYNC-1|
|FM-6|Beschädigter HEADER-Slot|zweites Exemplar übernimmt|T-HDR-1|
|FM-7|Korrupter WAL-Frame mitten in der Datei|Start verweigert (`Corrupt`)|T-WAL-1|
|FM-8|Restore aus Backup (Zeitregression oder Dateirestore)|Restore-Guard wiederholt Löschungen|T-RESTORE-1|
|FM-9|Journal vorhanden, Gate-Log fehlt|Gate aus Journal rekonstruiert|T-CRASH-4|
|FM-10|Passphrase falsch|`Auth`, keine Datei wird verändert|T-KDF-1|
|FM-11|Unbekanntes Dateisystem|Start verweigert (`UnsupportedFilesystem`)|T-PLAT-1|
|FM-12|SLA überschritten|Bericht weist Verletzung aus, `health()` meldet|T-SLA-1|
|FM-13|Absturz zwischen Tresor-Slot-Schreiben und WAL-`fsync` (Waise)|Recovery nullt verwaiste `D`/`F`-Slots|T-CRASH-5|
|FM-14|Absturz nach Manifest-Swap, vor `F`-Slot-Nullung oder `unlink`|Recovery nullt `F`-Slot, entfernt Datei|T-CRASH-6|
|FM-15|Kompaktion läuft, während `erase` wartet|Kompaktion bricht kooperativ ab, Teilergebnis verworfen|T-GATEGROUP-1, BM-PURGE-WAIT|
|FM-16|Manipulierte KDF-Parameter im HEADER|`Corrupt` vor Ableitung, keine Ressourcenerschöpfung|T-KDF-2|
|FM-17|Slot in fremden Store kopiert oder Parameter herabgesetzt|Unwrap schlägt wegen AAD fehl (`Auth`)|T-KEYSLOT-4|

---

## 14. Betrieb, Nebenläufigkeit, Plattform

### 14.1 Startsequenz

1. Verzeichnis-Lock (D-50). 2. Startprobe Dateisystem (D-56). 3. HEADER lesen (A/B), KDF-Grenzen prüfen (D-06), Keyslots, Passphrase prüfen. 4. Prozesshärtung (D-63). 5. Orphans in `tmp/` entfernen. 6. Gate-Log laden. 7. Journal lesen, offene Löschungen aufnehmen. 8. Manifest, WAL-Recovery (`WalFloor`). 9. Tresor öffnen; Waisen-Slots (`D`/`F`) und Dateien ohne Manifest-Eintrag bereinigen (D-83, D-85). 10. Ledger prüfen (Restore-Guard, 11.8). 11. Lesepfade freigeben, Hintergrundtasks starten, offene Löschungen fortsetzen. Fehler in 1–10 verhindern den Start.

### 14.2 Nebenläufigkeit

Sperren und Reihenfolge laut 5.4. Gruppen-Commit: Leader sammelt Batches, prüft ReadGate, schreibt Tresor-Slots + `sync_data`, dann WAL + `sync_data` (2 fsync je Gruppe). Gate-Gruppen-Commit (D-78). `loom`-Tests für Commit-Barriere, Gate, Purge-Barrier (T-CONC-1); kooperativer Abbruch der Kompaktion (T-GATEGROUP-1).

### 14.3 Drop und Herunterfahren

`Drop` blockiert nie; es setzt Stop-Flags. Geordneter Shutdown nur über `close().await`. Nicht abgeschlossene Löschungen bleiben im Journal.

### 14.4 Kapazität und Grenzen (D-87)

`max_docs_per_store` 5·10⁷, `max_doc_bytes` 4 MiB (≤ 64 MiB), Snapshots je Mandant ≤ 1024, Dimension ≤ 4096, Query-Terme ≤ 256, `k` ≤ 1000. Überschreitung: `Limit::*`, nie stilles Kürzen. Plattenreserve D-65.

### 14.5 Gesundheitsstatus (D-64)

`HealthReport { state: Ok|Degraded|Poisoned|Recovering, fsync_p50, open_erasures, oldest_open_age, sla_violations, tombstone_ratio, free_bytes, max_purge_barrier_wait, process_hardening, reasons: Vec<DegradedReason> }`. `DegradedReason` u. a. `LedgerMissing`, `PurgeSlow`, `FreeSpaceLow`, `HardeningPartial`. `Metrics`-Namen statisch (`ctx_erase_gated_seconds`, `ctx_fsync_seconds`, …), keine IDs als Labels.

### 14.6 Logging

`tracing`-Fassade mit festen Ereigniscodes, keine Inhalte, keine Kennungen, keine Pfade mit Personenbezug (HR-4, T-LOG-1).

### 14.7 Plattformen (D-56)

Tier 1: Linux x86_64/aarch64, ext4/XFS. Tier 2: macOS APFS (`F_FULLFSYNC`), Linux btrfs/ZFS (Restrisiko `FsJournalOrCow` immer). Nicht unterstützt: Windows, NFS, SMB, FUSE, overlayfs.

### 14.8 Referenzplattform für Benchmarks

Bare-Metal-Linux (x86_64, ≥ 8 Kerne, 32 GiB RAM), ext4 auf NVMe (`fsync_p50` ≤ 1,2 ms gemessen), Kernel ≥ 6.1, Build-Toolchain nach D-82 (Version im Protokoll), `--release`, `lto = "thin"`. Die Hardware wird im Benchmark-Protokoll dokumentiert (P9); Gate-Schwellen sind relativ zu `fsync_p50` der Startprobe (D-30). Ein Lauf ohne Hardware-Protokoll ist ungültig.

---

## 15. Teststrategie, Benchmarks, Gates, Umsetzungsplan

### 15.1 Test- und Nachweis-Framework

**Grundsatz:** Jede ID in dieser Spezifikation (D-, I-, FM-, T-, BM-, L-Zeile) ist maschinell mit mindestens einem Test oder Benchmark verknüpft. „100 % granular“ bedeutet hier **Rückverfolgbarkeit ohne Lücke**, nicht Fehlerfreiheit: Das ist die einzige Form von Vollständigkeit, die ein Dritter prüfen kann.

**Rückverfolgbarkeit (T-TRACE-1).** Jeder Test trägt das Attribut `#[ctx_test(id = "T-…")]` (bzw. `ctx_bench(id = "BM-…")`). `tests/arch` liest `docs/SPEC.md`, extrahiert alle `T-`/`BM-`/`I-`/`FM-`-IDs und prüft: (a) jede referenzierte ID hat ≥ 1 Test oder Benchmark; (b) jeder Test verweist auf eine existierende ID; (c) jede Zeile der Leistungsbeschreibung (Anhang B) ist abgedeckt. Fehlende Abdeckung blockiert CI. Ausnahme nur mit Ablaufdatum in `governance/exceptions.toml`.

**Testpyramide und Methoden:**

|Ebene|Methode|Ziel|
|---|---|---|
|Einheit|Unit- und Property-Tests (`proptest`) gegen `ReferenceModel` (naives In-Memory-Orakel)|Funktionale Korrektheit, MVCC, Matcher, Tags|
|Crash|Crash-Matrix mit `FaultVfs`: Abbruch nach **jedem** `write`/`fsync`/`rename`/`unlink` der Saga und des Commit-Pfads; Fehlerinjektion (`EIO`, `ENOSPC`, Teilschreibvorgänge, `fsync`-Fehler)|T-CRASH-*, T-FSYNC-1, T-WAL-*|
|Nebenläufigkeit|`loom` für Commit-Barriere, Gate, Purge-Barrier; Stresstests mit gleichzeitigen `erase`/`insert`/Kompaktion|T-CONC-1, T-GATEGROUP-1|
|Format|Fuzz (`cargo-fuzz`) aller Parser aus Anhang F, Journal-Replay, KDF-Header; Golden-Dateien je Format-Version|T-FMT-1, T-KDF-2|
|Unabhängige Prüfung|Lab (MK1–MK3), `ctx-verify`, Differenzprüfung|T-RAW-*, T-DIFF-1, T-PROOF-1|
|Mutation|`cargo-mutants` auf `erasure`, `classify`, Matcher, Gate, Tresor; **Überlebende Mutanten in diesen Modulen blockieren G-D**|T-MUT-1|
|Speichersicherheit|`miri` auf `types`, `mvcc`, `simd`-Referenz; `cargo-careful`; Sanitizer-Läufe (`asan`) auf `sys`|T-UNSAFE-1|
|API|`trybuild` (Compile-Fail), Doc-Tests, `cargo-semver-checks`|T-API-*|
|Architektur und Lint|`tests/arch`: Abhängigkeiten, Features, Netzwerkbann, Wortlaut-Lint, Muster-Ratchet|T-ARCH-*, T-CFG-*, T-LINT-*|
|Abdeckung|`cargo-llvm-cov`: Zeilen ≥ 90 % in `erasure`, `crypto`, `store`, `ports`; ≥ 95 % in `erasure`-Entscheidungslogik und `classify`; Zweigabdeckung berichtet|Ratchet, nur steigend|

**Testkatalog (normativ; jede ID ist ein Test oder eine Testfamilie):**

|Gruppe|ID|Prüft|
|---|---|---|
|Isolation|T-ISO-1|Nach `erase` sind alle Nicht-Ziel-Records bitgleich (Record-Ebene, I-3), Mandanten- und Subject-übergreifend|
||T-ISO-2|Matcher: Bereichs- und Exakt-Matcher treffen genau das Ziel (Property-Test, keine variablen Präfixe)|
||T-ISO-3|`IdCollision` wird erkannt (erzwungene Tag-Kollision im Test-Hash)|
||T-KEY-1|Schlüsselkodierung: feste Längen, Sortierung, `!commit_seq`, Längenpräfixe der Tag-Eingaben|
||T-DIFF-1|Lab-Differenz Nicht-Ziel bitgleich bis auf erwartete Manifest-/Seq-/Hüllenänderungen|
||T-DOCREF-1|`DocRef` mit gleichem `key`, anderem Betroffenen ⇒ verschiedene Dokumente; direkte Adressierbarkeit ohne Locator|
||T-SUBJ-1|Subject-Erase entfernt alle Dokumente des Betroffenen über alle Collections|
||T-SUBJ-2|Betroffener ist unveränderlich; Wechsel = neues Dokument + `erase` des alten|
||T-TENANT-1|Mandanten-Offboarding; fremde Mandanten unberührt (inkl. Leser)|
|Gate/Lesen|T-GATE-1|Nach persistiertem Gate: jeder Lesepfad `Erased`; Schreibzugriffe abgewiesen; nichts geschrieben (I-1, I-2)|
||T-VER-1|Zeitreise und Snapshots liefern `Erased` ab Gate|
||T-SNAP-1|Widerruf nur für Snapshots des Mandanten (D-61)|
||T-SCAN-1/2|`scan` überspringt gegatete Elemente; `ScanRange::Subject` ist ein Bereich|
||T-RESTR-1|`Restricted`: Lesen/Schreiben gesperrt, nichts vernichtet, Aufhebung journalisiert|
||T-COMPACT-RESTRICT-1|Kompaktion verwirft `Restricted`-Daten nie (I-18)|
||T-GATEGROUP-1|Gleichzeitige `erase` teilen Barriere und `fsync`; Kompaktion bricht kooperativ ab (I-16)|
|Schlüssel|T-VAULT-1|Slot-Nullung: Nullen verifiziert, Fehlen geprüft, Grabstein hält Sondierkette|
||T-VAULT-2|Schlüsselhierarchie: Vernichtung von `S`/`C`/`T` macht alle darunter liegenden DEKs unableitbar|
||T-VAULT-3|Reihenfolge I-9: nie ein Record ohne Schlüssel (Crash-Matrix)|
||T-VAULT-4|Zeit bis `Gated` unabhängig von der Zielgröße (Korrektheitsteil von BM-ERASE-O1)|
||T-VAULT-5|Tresor: Rehash, Lastfaktor, Kapazitätsgrenze, Open ohne Vollscan|
||T-KEYCOV-1|`key_coverage` je Schicht stimmt mit gemessenem Zustand überein (Lab: Postings `unlinked`, Kanten `none`)|
||T-KEYSLOT-1..4|Slot hinzufügen/entfernen, Passphrasenwechsel O(1), KDF-Wechsel, AAD-Bindung (Kopie/Herabsetzen scheitert)|
||T-HDR-1|A/B-HEADER: Korruption eines Exemplars ohne Totalverlust|
||T-KDF-1|Grenzen D-06, falsche Passphrase ⇒ `Auth`; T-KDF-2: Fuzz auf manipulierte Parameter|
||T-NONCE-1|Nonce-Eindeutigkeit je Dateischlüssel (Zähler), Überlaufgrenze 2³²|
||T-ENC-1..3|Pflichtverschlüsselung, AAD-Bindung (Umordnen/Kreuzkopie), Nonce-Schema|
||T-FMT-1|Jeder Header trägt Version und Algorithmus-IDs; unbekannte Version ⇒ `Corrupt`|
||T-FK-1..3|Dateischlüssel: Anlage vor erstem Block, Vernichtung nach Swap vor `unlink`, freigegebene Blöcke nicht entschlüsselbar|
|Messung|T-RAW-1..6|MK1/MK2/MK3 je Schicht (WAL, SSTable, Text, Vektor, Journal, `tmp/`); T-RAW-3 Embeddings; T-RAW-4 Tags im Index; T-RAW-6 Dateinamen|
||T-CLASS-1|Klassenverband G/K/P je einzeln, keine Gesamtnote|
||T-PROOF-1|`ctx-verify report` akzeptiert gültige, lehnt jede Manipulation ab (Signatur, Kette, fehlende Positivkontrolle, fehlendes Restrisiko)|
||T-REPORT-1|Bericht enthält keine Inhalte/Kennungen; alle zutreffenden Restrisiken|
||T-MUT-1|Absichtlich gebrochene Purge-/Gate-/Tresor-Implementierungen werden vom Lab erkannt|
|Crash|T-CRASH-1|Absturz an jedem Punkt der Saga ⇒ Vollendung oder `Pending`|
||T-CRASH-2|Absturz im Rewrite; T-CRASH-3: Platte voll; T-CRASH-4: Journal ohne Gate-Log|
||T-CRASH-5|Waisen-Slots (`D`/`F`) werden bereinigt; T-CRASH-6: Absturz zwischen Manifest-Swap und `F`-Nullung|
||T-CANCEL-1|Abbruch des Aufruferfutures ändert den Ausgang nicht|
||T-FSYNC-1|`fsync`-Fehler ⇒ Datei fatal, `Poisoned`, nie Wiederholung auf demselben Handle|
||T-WAL-1|Korrupter Frame mitten in der Datei ⇒ Start verweigert; Torn Write am Ende toleriert|
||T-WAL-2|WAL wird nie umgeschrieben; Kette setzt nach `WalFloor` mit `tail_mac` fort|
||T-LOCK-1|Zweiter Prozess scheitert am Verzeichnis-Lock|
|Restore|T-RESTORE-1|Zeitregression erkannt, Löschungen wiederholt|
||T-RESTORE-2|Dateirestore: Wiederauftauchen erkannt; Neuanlage mit `seq > gated_seq` löst nichts aus|
||T-RESTORE-3|Ledger-Lebensdauer = `backup_retention`, nicht `commit_epoch`; Sequenz monoton (I-20)|
||T-JRN-3|Journal-/Bericht-Commitments rotieren mit `commit_epoch`|
|Suche|T-TEXT-1..4|Segmentbildung, Merge, Rewrite je Zielsegment, Statistiken exakt neu (E-L5); T-TEXT-4: Postings `unlinked`|
||T-TEXT-ANALYZER-1|Goldenset Komposita/Umlaute/`ß`; T-TEXT-PHRASE-1: Phrasensuche|
||T-VEC-DEL-1/2|Surrogat sofort, kein Zielvektor-Byte in Arena; Kantenreparatur: keine Referenz auf Zielslots, Recall-Grenze|
||T-VEC-REBUILD-INDEP|Graph nach Rebuild hängt nur von der Menge der lebenden Slots ab|
||T-VEC-ARENA-1|Alle Repräsentationen je Slot zusammenhängend und überschrieben|
||T-HYB-1, T-SEARCH-2|Nachladen bis `k` oder Erschöpfung; `exhausted` korrekt; nie stilles Verkürzen|
||T-RANK-1|RRF deterministisch|
|Architektur|T-ARCH-1..8|Ringe, Matrix, Verbote, Netzwerkbann; **neu:** T-ARCH-5 Ringangabe/Abhängigkeiten im Crate-Kopf gegen 5.1/5.3; T-ARCH-6 Matrix aus `cargo metadata`, kein Glob-Re-Export in `ports`; T-ARCH-7 Fassade ohne Logik, Namens- und Modulregeln (D-90); T-ARCH-8 keine Agenten-/Werkzeugverzeichnisse und keine nicht normativen Dokumente im Baum (D-92)|
||T-CFG-1/2/3|Keine Sicherheitsfeatures/Schalter; `testing` nie im Release; `batch_window ≤ erase_sla/2`|
||T-DEF-1|Keine Konfiguration ohne Verschlüsselung/Integrität|
||T-UNSAFE-1|`unsafe` nur `sys`/`simd`, Safe-Funktionen ohne offene Vorbedingung|
||T-LINT-1/2/3|Wortlaut-Lint; kein `block_on`/kein zweites `Vfs`; alte Begriffe E1–E3/E0–E4 entfernt|
||T-API-1..3|Mandantenpflicht (trybuild), `RetentionAttestation`-Pflicht, `soft_delete`-Doc|
||T-PLAT-1|Startprobe verweigert unbekannte Dateisysteme|
||T-LIMIT-1|Kapazitätsgrenzen liefern `Limit::*`|
||T-TRACE-1|Rückverfolgbarkeit Spezifikation ↔ Test|
|Betrieb|T-SLA-1|`erase` bis `Gated` innerhalb D-30; SLA-Verletzung wird ausgewiesen|
||T-OPS-1/2|`health()` Felder; Plattenreserve|
||T-LOG-1|Keine Inhalte/Kennungen in Logs und Fehlern|
||T-DUMP-1, T-SEC-2|Prozesshärtung (Coredump, Schlüsselseiten)|
||T-SUPPLY-1/2|PSIRT-Prozess, SBOM, Toolchain-Regel D-82|
||T-SIMD-1|Jeder Kernel gleicht gegen die Skalar-Referenz|
||T-CONC-1|`loom`: Barrieren|
|Leistung/Beleg|T-LEIST-1..12|je Zeile L1–L12 (Anhang B) ein Abnahmetest, der die Zusage end-to-end belegt|

### 15.2 Benchmark-Framework

**Harness.** `benchmarks/` enthält (a) `criterion`-Mikrobenchmarks (Distanzkerne, Tag-Berechnung, Envelope seal/open, BM25-Block) und (b) ein eigenes **Makro-Harness** `ctx-bench` (Binary im Workspace, dev-only) für Szenarien auf realen Dateien mit `RealVfs`. Jeder Lauf schreibt ein **Protokoll** (JSON): Commit-Hash, Toolchain, Hardware (CPU, RAM, Kernel, Dateisystem, `fsync_p50`), Seed, Parameter, Rohzeiten (HDR-Histogramm), abgeleitete Kennzahlen. Läufe ohne vollständiges Protokoll sind ungültig (P9). Die Referenzhardware ist 14.8. Regressionsprüfung: Vergleich mit der Basislinie des letzten Release auf **derselben** Hardware; Verschlechterung > 10 % blockiert (Schlüsselwerte: BM-WRITE, BM-GET, BM-ERASE-LAT).

**Korpora.** Synthetischer Generator (deterministisch, Seed) für Dokumente (1 KiB, 8 KiB), Betroffene (Verteilung: 80 % ≤ 10 Dokumente, Schwanz bis 10⁶), deutsche Textkorpora (Wikipedia-DE-Auszug und Rechtstexte, lizenzrein, nur für Benchmark/Goldenset), Vektoren (768 dim, aus offenem Modell vorberechnet, f32).

|ID|Messung|Ziel (Gate-Schwelle, [E])|
|---|---|---|
|BM-ERASE-LAT|`erase` bis `Gated` (Dokument)|p50 ≤ 10 ms (NVMe), allgemein ≤ 6 × `fsync_p50` + 2 ms, p99 ≤ 5 × p50|
|**BM-ERASE-O1**|`erase` bis `Gated` für Ziele mit 10², 10⁴, 10⁶ Dokumenten (Subject, Collection, Mandant)|Zeit(10⁶) / Zeit(10²) ≤ 1,2|
|BM-ERASE-WA|Bytes/Dateien je Purge bei 1/10 GiB, Dokument/Subject/Collection|Gate G0 (11.9, D-76)|
|**BM-ERASE-WA-TEXT**|Bytes/Segmente je Subject-Erase (Betroffener mit 10³ Dokumenten über 12 Monate) bei Segmentzielgrößen 8/32 MiB und `bucket_count` ∈ {1, 4, 16}|Aufwand ≤ 2 × Σ Größe der berührten Segmente; Entscheidung über `bucket_count > 1`: wenn der Median-Betroffene > 25 % der Segmente berührt (D-71d)|
|BM-ERASE-BATCH|1000 Erases im Fenster (inkl. WAL-Flush und Segment-Entfernung)|berührte Dateien ≪ Summe; Fensterdauer ≤ `erase_sla − batch_window`|
|BM-WRITE|Insert-Durchsatz mit Gruppen-Commit (Block-AEAD und Record-Layer)|≥ 20 k Dokumente/s (1 KiB) auf Referenz, Regression ≤ 10 %; Kosten des Block-Layers getrennt ausgewiesen (D-32)|
|**BM-WRITE-DURING-ERASE**|Insert-Durchsatz und p99-Latenz während eines Erase-Stroms (10 Erases/s)|Durchsatz ≥ 70 % der Ruhe, p99 ≤ 3 × Ruhe|
|**BM-PURGE-WAIT**|Wartezeit des Purge auf die Barrier bei laufender Kompaktion|p99 ≤ 1 s (I-16)|
|BM-GET|`get` p99 (warm)|≤ 200 µs|
|BM-TEXT|BM25 top-10 auf 1 M Chunks (de/1, mit Block-Max-WAND)|p99 ≤ 30 ms|
|BM-VEC|HNSW top-10, Recall@10 ≥ 0,95 (1 M × 768)|p99 ≤ 15 ms|
|**BM-VEC-DEL**|Recall@10 nach Löschung von 5 % / 10 % / 20 % der Slots mit Batch-Kantenreparatur; Dauer des Reparaturdurchlaufs|Recall ≥ 0,95; Durchlauf ≤ 60 s je 10⁶ Knoten (speichergebunden)|
|**BM-VEC-REBUILD**|Deterministischer Rebuild 1 M × 768|Dauer dokumentiert; Recall ≥ Ausgangswert − 0,01|
|BM-OPEN|`open` ohne Argon2|≤ 2 s bei 10 GiB|
|**BM-VAULT-OPEN**|Tresor-Open bei 5·10⁷ Slots|≤ 2 s, kein Vollscan|
|**BM-RESTORE-OPEN**|Ledger-Prüfung beim `open` (10⁴ Einträge, Zeitregression)|≤ 1 s ohne Scan|
|BM-KDF|Argon2id-Latenz Standardparameter|0,3 … 1,5 s|

Zielwerte sind bis zum Lauf WP-6.1 Schwellen, keine Messungen (0). Regressionen blockieren.

### 15.3 Gates

|Gate|Bedingung|
|---|---|
|G-A|`cargo check --workspace --all-targets --locked` grün, `cargo fmt --check` grün (F-01..03)|
|G-B|Rückbau vollständig nach D-84 (0 Verweise auf `graph` etc.), Features/Tokio laut D-54, `deny` ohne offene Ausnahmen (F-05, F-06, F-16, F-17, F-23), ein `Vfs`-Trait|
|G0|Speicherentscheid (11.9) als Prototyp mit D-70/D-71; Ergebnis mit Messprotokoll, Stufe der Fallbackleiter festgelegt|
|G-C|Kern-Crates: 0 `unwrap/expect/panic/let _ =/allow`; Clippy `-D warnings`; Sentinel- und Delete-Race-Fixes (F-07, F-08); Pflichtverschlüsselung (F-09, F-21)|
|G-D|Löschkern S0: T-ISO, T-GATE, T-VAULT, T-KEYCOV, T-FK, T-CRASH, T-RAW, T-PROOF, T-RESTORE grün; Lab MK1–MK3; Mutationstests ohne Überlebende; BM-ERASE-LAT, BM-ERASE-O1, BM-ERASE-WA erfüllt; HR-7 erfüllt|
|G-E|S1 Textindex (T-TEXT-*, BM-TEXT, BM-ERASE-WA-TEXT mit Entscheidung D-71d)|
|G-F|Keyslots/HEADER/Plattformprobe (T-KEYSLOT, T-HDR, T-KDF, T-PLAT)|
|G-G|S2 Vektor (T-VEC-*, BM-VEC, BM-VEC-DEL, BM-VEC-REBUILD)|
|G-H|Release: S0–S3 (oder Cut-Linie), alle T-* und BM-*, T-TRACE-1, SBOM, signierte Artefakte, Doku-Lint, Support-Dokumente, Evidenzbündel (HR-11)|
|G-LEGAL|extern (6.7), blockiert nur Aussagen jenseits 6.4|

Reihenfolge: A → B → G0 → C → D → F → E → G → H. G-LEGAL läuft parallel.

### 15.4 Umsetzungsplan (Arbeitspakete)

Aufwände sind Schätzungen **[E]** in Personenwochen (PW) für ein Team mit zwei bis drei erfahrenen Rust-Entwicklern; sie sind keine Zusage. Jedes Arbeitspaket nennt ein Evidenzbündel (D-44: erst rot, dann grün).

|WP|Inhalt|Ergebnis / Evidenz|Gate|PW|
|---|---|---|---|---|
|WP-0.1|Governance: `governance/ratchet.toml` (F-20), `governance/exceptions.toml`, PR-Vorlage (HR-12), `tests/arch`-Grundgerüst, Wortlaut-Lint, T-TRACE-1|Ratchet greift, Lint rot auf Altbestand|–|2|
|WP-0.2|Toolchain-Regel D-82, `rust-toolchain.toml` aktualisieren, MSRV-Matrix, `cargo-semver-checks`, SBOM-Job|CI mit Pinning-Regel, kein `continue-on-error` auf Sicherheitsjobs (F-17)|G-A|1|
|WP-1.1|Rückbau Graph/Altlasten: 100 Verweise (F-05), `AutoExtractionMode`, Ports `embedding`/`reranker`/`query_rewriter`|0 Treffer im Grep|G-B|3|
|WP-1.2|Branch `archive/v0-full` (D-22); Rückbauliste D-84 (Features, `acl_win32.rs`, `WasmMergeOperator`)|Feature-Skript grün (AR-9)|G-B|3|
|WP-1.3|Tokio-Features (D-54), `deny.toml` aktualisieren (Bans, Advisories mit Ablaufdatum, tote `clarify`/`wrappers`)|`cargo tree -e normal` ohne `mio`/`socket2`|G-B|1|
|WP-1.4|Ein `Vfs`: `durable-fs` auf `ports::Vfs`; `store`/`vector` ohne `std::fs` (F-13)|T-LINT-2 grün|G-B|4|
|WP-1.5|Struktur und Namen: Fassade nach D-89 schneiden, `contextra_impl`→`instance`, Modul-/Namensregeln D-90, Crate-Köpfe D-91, Matrixkorrektur D-88 im Code, `tests/arch` T-ARCH-5..8, Folgedokumente und Werkzeugverläufe nach D-92 entfernen|T-ARCH-5..8 grün (Ratchet für Altbestand)|G-B|3|
|WP-2.1|`contextra-engine` und Tests wieder kompilierbar (F-01, F-02), `fmt` (F-03)|`check --all-targets --locked` grün|G-A|6|
|WP-3.1|`types`: `DocRef`, `DocHandle`, Tags mit Längenpräfix, Löschtypen, `KeyCoverage`, `DocId` entfernt|T-KEY-1, T-DOCREF-1|G0|2|
|WP-3.2|`crypto`: D-06 Grenzen, D-52 Slot-AAD, D-60 Nonce-Schema, D-80 Algorithmenliste, `K_sign` abgeleitet|T-KDF-*, T-KEYSLOT-*, T-NONCE-1|G-F|3|
|WP-3.3|`store`: Schlüsselraum 9.1, Envelope v2, WAL ohne Rewrite, `WalFloor`, Sentinel-Fix, Delete-Race-Fix, Pflichtverschlüsselung|T-ISO, T-WAL-*, T-ENC-*|G-C|10|
|WP-3.4|**G0-Prototyp:** Schlüsselraum mit D-70/D-71, SSTable-Rewrite, BM-ERASE-WA/-WA-TEXT auf 1/10 GiB|Messprotokoll, Entscheid über Fallbackstufe|**G0**|4|
|WP-3.5|Fortschreibung dieser Spezifikation nach G0 (Anhang H), falls Stufe > 0|SPEC 4.x|G0|1|
|WP-4.1|`erasure`: Tresor (`FileVault`, F.3), `RecordSealer`/`FileKeyring`, Dateischlüssel-Lebenszyklus|T-VAULT-*, T-FK-*|G-D|8|
|WP-4.2|`erasure`: Gate-Log, Gate-Gruppen-Commit, Saga, Journal, Recovery, Kompaktions-Kooperation|T-GATE-*, T-CRASH-*, T-GATEGROUP-1|G-D|10|
|WP-4.3|Messung: `Scanner<Armed>`, `LayerWitness`, Bericht, Ledger/Restore-Guard|T-RAW-*, T-PROOF-1, T-RESTORE-*|G-D|6|
|WP-4.4|`lab` und `ctx-verify` (unabhängige Parser nach Anhang F), Mutationstests|T-MUT-1, T-DIFF-1|G-D|8|
|WP-5.1|`engine`: `TenantScopedStorage`, Collections, `DocRef`-API, Suchstrom mit Nachladen, Startsequenz 14.1|T-API-*, T-HYB-1|G-D|6|
|WP-5.2|Plattformprobe, Prozesshärtung, `health()`/`Metrics`, Plattenreserve|T-PLAT-1, T-DUMP-1, T-OPS-*|G-F|4|
|WP-6.1|Benchmark-Harness `ctx-bench`, Referenzhardware, erste Basislinien aller BM|Protokolle, Basislinie|G-D|4|
|WP-7.1|`text`: Segmentindex, Analyzer `de/1`/`und/1`, BMW, Positionen, Entkopplung, Rewrite|T-TEXT-*, BM-TEXT, BM-ERASE-WA-TEXT|G-E|12|
|WP-8.1|`vector`: Arena D-86, Surrogat, Kantenreparatur, Rebuild, verschlüsselter Snapshot über `Vfs`|T-VEC-*, BM-VEC*|G-G|12|
|WP-8.2|`rank` (RRF), Hybridsuche|T-RANK-1, T-HYB-1|G-G|2|
|WP-9.1|Doku: README (HR-3, 6.4), Rustdoc, `docs/OPERATIONS.md`, `docs/DSFA-BAUSTEIN.md` (Anhang G), Runbooks, `SECURITY.md`|T-LINT-1, T-LEIST-*|G-H|4|
|WP-9.2|Release: signierte Artefakte, SBOM, Evidenzbündel, Fuzz-/Mutationsläufe, G-LEGAL-Stand|alle Gates|G-H|3|

Kritischer Pfad: WP-1.x → WP-2.1 → WP-3.1..3.4 (G0) → WP-4.x → G-D. Summe ≈ 123 PW [E] (4.1.0: +3 PW für WP-1.5); bei drei Entwicklern ca. 40 Wochen bis G-H. Bei Cut-Linie entfallen WP-8.x (≈ 14 PW) im ersten Release.

---

## 16. Löschcheckliste für Betreiber (Wortlaut endgültig, E-L8)

> **Keine Rechtsberatung.** Diese Liste benennt technische Schritte, die außerhalb von Contextra liegen.

1. Rechtsgrundlage und Aufbewahrungspflichten prüfen; `RetentionAttestation` wahrheitsgemäß setzen.
2. Dokumente beim Anlegen dem Betroffenen zuordnen (`DocRef.subject`); nicht zugeordnete Dokumente sind nur über Dokument, Collection oder Mandant löschbar.
3. `erase` mit passendem Ziel (`Subject` für Betroffene) auslösen; Status bis `Completed` beobachten (`Pending` > 1 h alarmieren).
4. Bericht mit `ctx-verify` prüfen und gemäß Art. 5 Abs. 2 DSGVO aufbewahren (Commitments laufen nach der Commitment-Epoche ab). Bei Bedarf die Feldmessung mit `ctx-verify scan` unabhängig wiederholen.
5. **Externe Sicherungen** (Backups, Snapshots der Plattform) separat bereinigen oder nach Restore die Löschung erneut ausführen (Restore-Guard; Ledger **außerhalb** des Backups halten; `backup_retention` mindestens so lang wie die längste Aufbewahrung der Sicherungen).
6. Kopien beim Betreiber (Logs, Exporte, Caches, Suchindizes anderer Systeme, Embeddings-Dienste) identifizieren und löschen.
7. Empfänger nach Art. 17 Abs. 2 und Art. 19 informieren (`ExternalProcessors` im Bericht).
8. Flash-/Dateisystem-Remanenz bewerten: bei hohem Schutzbedarf Volume-Verschlüsselung mit Schlüsselvernichtung und zertifizierter Medienaussonderung ergänzen.
9. Swap verschlüsseln oder deaktivieren; Coredumps abschalten (Contextra setzt bei `process_hardening` `RLIMIT_CORE=0`).
10. Passphrase-Verwaltung: kein Escrow durch den Hersteller (HR-3); Notfall-Keyslot im eigenen Tresor.
11. Den Bericht auf `key_coverage` und Restrisiken (`TextIndexUntilPurge`, `IndexTopologyInference`) prüfen und deren Bereinigung (Segment-Rewrite, Rebuild) abwarten, bevor die Löschung gegenüber Dritten als abgeschlossen bezeichnet wird.
12. DSFA-Baustein (`docs/DSFA-BAUSTEIN.md`, Anhang G) in die eigene Dokumentation übernehmen; Wirksamkeit regelmäßig prüfen.

---

## 17. Support, Lieferkette, Betriebshandbuch

### 17.1 Lieferung je Release

Signierte Quell- und Binärartefakte (Sigstore oder minisign), SBOM (CycloneDX), `cargo-audit`/`cargo-deny`-Bericht, Changelog mit Format-Version, Evidenzbündel (Gate-Protokolle EV2/EV3, Lab-Läufe EV4, Mutationsergebnisse), reproduzierbarer Build (Toolchain nach D-82 mit Versionsangabe, `Cargo.lock` eingecheckt, `--locked`).

### 17.2 Versionierung und Support

SemVer; Format-Version getrennt (D-66). Je Hauptversion ≥ 5 Jahre Sicherheitsupdates (D-67). Formatänderungen: Migrationspfad nur vorwärts, explizit (`migrate`), nie stilles Upgrade (P3). `cargo-semver-checks` in CI.

### 17.3 Schwachstellenmanagement

`SECURITY.md`: Meldeweg (verschlüsselt), Frühwarnung 24 h und Meldung 72 h nach CRA Art. 14 bei aktiv ausgenutzten Schwachstellen, interne Fristen kritisch 7 d / hoch 30 d (HR-8), Advisory-Ausnahmen in `deny.toml` nur mit Ablaufdatum (F-17). CI ohne `continue-on-error` auf Sicherheitsgates.

### 17.4 Runbooks (Pflichtlieferung)

|Runbook|Inhalt|
|---|---|
|RB-1 Passphrase wechseln|`change_passphrase`, Verifikation, alter Slot|
|RB-2 Löschung bleibt `Pending`|Ursachenliste je `PendingReason`, Platz prüfen, Neustart|
|RB-3 `Poisoned`|Ursache (`fsync`-Fehler) auswerten, Neustart, Dateisystem prüfen|
|RB-4 Restore|Ledger bereitstellen, Replay abwarten, Bericht; `LedgerMissing` und `acknowledge_ledger_reset`|
|RB-5 Plattenvoll|Reserve, Löschung weiter möglich, Platz schaffen|
|RB-6 Offboarding|`Tenant`-Löschung, Nachweis|
|RB-7 Passphrase verloren|Keine Wiederherstellung (HR-3)|
|RB-8 Bericht prüfen|`ctx-verify report` und `scan`, Ablage, Interpretation von `key_coverage`|

### 17.5 Dokumentationspflichten

README beginnt mit HR-3-Hinweis und Abschnitt 6.4; Rustdoc je Lösch-API mit Restrisiken; `docs/DSFA-BAUSTEIN.md`, `docs/OPERATIONS.md`; Wortlaut-Lint über alles (T-LINT-1). Die Byte-Formate stehen in Anhang F dieser Datei (KF-9), nicht in `docs/FORMATS.md`.

---

## Anhang A – Funktionsinventar

|ID|Funktion|Stufe|
|---|---|---|
|FI-F01|Dokument-CRUD mit Mandanten und `DocRef`|S0|
|FI-F02|Zeitreise und Snapshot|S0|
|FI-F03|Startprobe Dateisystem, Plattformstufen|S0|
|FI-F04|`erase` mit Gate, Tresor-Hierarchie, Purge, Bericht|S0|
|FI-F05|Keyslots, Passphrasenwechsel|S0|
|FI-F06|BM25-Suche mit Analyzer `de/1`, Phrasen|S1|
|FI-F07|HNSW-Suche mit Surrogat, Kantenreparatur, Rebuild|S2|
|FI-F08|Hybridsuche (RRF)|S3|
|FI-F09|Restore-Guard, Ledger mit Bereichen|S0|
|FI-F10|Gesundheitsstatus, Metriken|S0|
|FI-F11|Dateischlüssel-Vernichtung (`FileKeyring`)|S0|
|FI-F12|`Restricted` (Art. 18), `lift_restriction`|S0|
|FI-F13|`ctx-verify report` und `scan`, Lab|S0|
|FI-F14|Mandanten-Offboarding in O(1) bis `Gated`|S0|

Fehlermodi: 13.2. Invarianten: 13.1.

## Anhang B – Leistungsbeschreibung (HR-6)

Die Zeilen entsprechen 1.5 und sind einzeln testbar. Jede Zeile gilt nur im Rahmen von 6.4 und 1.6.

|Zeile|Zusage|Tests|Evidenz|
|---|---|---|---|
|L1|Ziel-Isolation (Record-Ebene)|T-ISO-1, T-DIFF-1|EV4 (Lab)|
|L2|ReadGate ab Sperre inkl. Zeitreisen, keine Schreibzugriffe nach Sperre|T-GATE-1, T-VER-1|EV3|
|L3|Schlüsselvernichtung je Ziel (höchste Ebene), Zeit unabhängig von der Zielgröße; `key_coverage` je Schicht|T-VAULT-1..5, T-KEYCOV-1, BM-ERASE-O1|EV4|
|L4|`field_measurement` MK1/MK2/MK3-Feld nach Abschluss; unabhängig wiederholbar mit `ctx-verify scan`; MK1–MK3 im CI-Lab|T-RAW-1..6|Feld: EV2; Lab: EV4|
|L5|Crash-/Abbruchsicherheit|T-CRASH-1..6, T-CANCEL-1, T-FSYNC-1|EV3|
|L6|Bericht mit Restrisiken, signiert, unabhängig prüfbar|T-REPORT-1, T-PROOF-1|EV4|
|L7|Pflichtverschlüsselung|T-DEF-1, T-ENC-1..3, T-FK-1..3|EV3|
|L8|Mandantenpflicht|T-API-1|EV1/EV3|
|L9|Keine Netzwerkfunktion/Telemetrie|T-ARCH-4, T-LOG-1|EV3|
|L10|Restore-Guard (innerhalb `backup_retention`)|T-RESTORE-1..3|EV3|
|L11|Passphrasenwechsel ohne Neuverschlüsselung|T-KEYSLOT-1..4|EV3|
|L12|Plattformprobe, Fail-closed|T-PLAT-1, T-FSYNC-1|EV3|

Nichtleistungen: 1.6. `lab_evidence` wird nur für Zeilen ausgewiesen, deren Evidenz EV4 erreicht hat.

## Anhang C – Tag-Labels und Konstanten

|Zweck|Label|
|---|---|
|`tenant_tag`|`ctx/tenant/v1`|
|`coll_tag`|`ctx/coll/v1`|
|`subject_tag` (= `cluster_tag` bei Zuordnung)|`ctx/subj/v1`|
|`doc_tag`|`ctx/doc/v2`|
|`term_tag`|`ctx/term/v1`|
|`name_tag`|`ctx/name/v1`|
|Commitment|`ctx/commit/v1` (+ Epochenschlüssel)|
|Schlüsselableitung|`ctx/enc/v1`, `ctx/idx/v1`, `ctx/unit/v1`, `ctx/sign/v1`, `ctx/wal/v1`, `ctx/ledger/v1`, `ctx/file/v1`, `ctx/fkmask/v1`, `ctx/dek/v2`, `ctx/dek-salt/v2`|
|Record-AAD|`ctx/rec/v2`|
|Keyslot-AAD|`ctx/slot/v1`|
|Vault-Slot-MAC|`ctx/vault-mac/v1`|
|Header-MAC|`ctx/hdr/v1`|
|Textbindung-AAD|`ctx/tbind/v1`|

Konstanten: Tag 16 B, Salt 32 B, Nonce 12 B, Tresor-Slot 80 B, Segmentziel 8–32 MiB (D-71), SSTable-Ziel Level ≥ 1: 32 MiB, `k_rrf = 60`, `commit_epoch = 365 d`, `ef_max = 4096`, Postingblock 128, Tresor-Lastfaktor ≤ 0,7.

## Anhang D – Berichtsformat und Restrisikokatalog

**Restrisiken (`ResidualRisk`, geschlossen):** Jedes Risiko hat Kennung, Kurztext, Bedingung (wann es zutrifft) und Betreiberhinweis; `ctx-verify report` prüft die Vollständigkeit gegen die Bedingungen.

|Kennung|Kurztext|Bedingung|Betreiberhinweis|
|---|---|---|---|
|`FlashRemanence`|Flash-Medien können Altdaten in nicht adressierbaren Zellen halten|immer (Medium nicht rotierend oder unbekannt)|Volume-Verschlüsselung, Medienaussonderung|
|`FsJournalOrCow`|Dateisystem-Journal oder Copy-on-Write kann Altblöcke halten|Tier 2 (btrfs/ZFS/APFS) immer; ext4/XFS bei Journal-Modus `data=journal`|Dateischlüssel-Vernichtung mindert (D-85); Volume-Verschlüsselung|
|`WrappedKeyRemanence`|Genullter Tresor-Slot oder Altblock kann auf dem Medium verbleiben|immer, solange der Anker `FileVault` ist|Hardware-Anker (D-53) oder Volume-Verschlüsselung|
|`FreedBlockRemanence`|Freigegebene Dateiblöcke bleiben bis zum Überschreiben auf dem Medium|immer; ohne Schlüssel wertlos nach D-85, soweit `F`-Slot genullt|keine weitere Maßnahme erforderlich, außer hohem Schutzbedarf|
|`OldKeyslotRemanence`|Ein alter Keyslot kann nach Passphrasenwechsel auf dem Medium verbleiben|nach `change_passphrase`|Alte Passphrase gilt als kompromittiert; Master-Rotation v1.1|
|`ProcessMemory`|Klartext kann im Prozessspeicher, Swap oder Coredump liegen|`process_hardening ∈ {Off}` oder Teilerfolg; sonst als Hinweis|Swap verschlüsseln, Coredumps abschalten|
|`IndexTopologyInference`|Reparierte HNSW-Kanten sind eine Funktion der Zielnachbarschaft|nach Vektorlöschung bis zum nächsten Rebuild|`vector_strict_rebuild` aktivieren|
|`TextIndexUntilPurge`|Entkoppelte Postings enthalten den Wortschatz des Ziels ohne Zuordnung|zwischen `KeyDestroyed` und Segment-Rewrite|Abschluss abwarten|
|`ReplicaWindow`|Zeit zwischen Anfrage und Abschluss; Kopien in diesem Fenster|immer, Dauer = `completed_at − requested_at`|`erase_sla` klein halten|
|`ExternalCopies`|Kopien außerhalb von Contextra (Backups, Exporte, Logs)|immer|Checkliste 16|
|`ExternalProcessors`|Empfänger und Auftragsverarbeiter außerhalb von Contextra|immer|Information nach Art. 17 Abs. 2, 19|
|`PseudonymCommitment`|Commitments, Bereichs-Tags im Bericht, Ledger sind pseudonym|immer|Aufbewahrungsdauer begründen (DSFA)|
|`PurgeScalesWithDb`|Physischer Aufwand wächst mit der Datenbank|nur Fallbackstufe 4 (11.9)|–|
|`PurgeDeferred`|Physischer Purge erfolgt verzögert bei Kompaktion|nur Fallbackstufe 3 (11.9)|Abschluss abwarten|
|`KeyCompromise`|Wer Passphrase oder `K_master` vor der Löschung besaß, konnte Daten entschlüsseln und kopieren|immer|Schlüsselverwaltung des Betreibers|

**Pflichttexte im Bericht:** (1) Signaturumfang (E-L11): „Die Signatur beweist Integrität und Herkunft aus dieser Installation, nicht die Wahrheit des Inhalts.“ (2) „Keine Rechtsberatung; die rechtliche Wertung obliegt dem Betreiber.“ (3) Außenaussage aus 6.4. (4) Klassen G/K/P je mit `reached` und `key_coverage` je Schicht. (5) `field_measurement` und `lab_evidence` getrennt.

## Anhang E – Anzupassende Folgedokumente

|Dokument|Änderung|
|---|---|
|`docs/SPEC.md`|ersetzt durch diese Fassung 4.1.0|
|`PLAN.md`|durch Verweis auf 15.4 ersetzen (kein eigenes Plandokument)|
|`VETOES.md`|VETO-D01 aufheben, VETO-D01b einführen (Schlüsselvernichtung in v1)|
|`CONSTITUTION.md`, `ARCHITECTURE.md`, `SOURCE_OF_TRUTH.md`, `AGENTS.md`, `crates/*/AGENTS.md`|„keine Ebene B in v1“ streichen; Verweis auf SPEC 4.1.0; **ab 4.1.0 löschen statt angleichen (D-92):** eine Wurzel-`AGENTS.md`, keine `crates/*/AGENTS.md`|
|`SECURITY.md`|CRA-Meldeweg, Fristen, Support-Zeitraum (D-67)|
|`docs/DELETION-DSGVO.md`|an E-L1..E-L16 angleichen; Wortlaut-Lint|
|`docs/FORMATS.md` (falls vorhanden)|löschen; Formate stehen in Anhang F|
|`ADR-001`/`ADR-002`|ADR-002 ersetzt ADR-001 (D-01)|
|`deny.toml`, Workflows|D-54-Bans, keine `continue-on-error` auf Sicherheitsgates, Advisory-Ablaufdaten|
|`rust-toolchain.toml`|nach D-82 aktualisieren|
|`README.md`|HR-3-Hinweis, Außenaussage 6.4|

## Anhang F – Byte-Formate (normativ)

Alle Zahlen big-endian, sofern nicht anders angegeben. `‖` = Konkatenation. Die Formate werden von `lab` und `ctx-verify` unabhängig implementiert (AR-4). Jedes Format hat Golden-Dateien in `tests/golden/` (T-FMT-1).

### F.1 Dateikopf (alle Datendateien und Steuerdateien)

```
offset size  feld
0      8     magic            "CTXFILE\0"
8      2     format_version   u16  (v1 = 1)
10     1     kind             u8   siehe Tabelle
11     4     alg_ids          u8[4] = [aead, kdf_or_0, mac, hash]; v1: [1, 0, 1, 1]
15     16    file_id          zufällig (OS-CSPRNG)
31     8     created_seq      u64
39     1     key_mode         0 = zufälliger Dateischlüssel (F-Slot), 1 = abgeleitet (K_file)
40     8     reserved         0
48     16    header_tag       AEAD-Tag über Bytes [0..48) als AAD, leerer Klartext,
                              Nonce 0xFFFFFFFF ‖ u64(0), Dateischlüssel
64     …     Körper
```

|kind|Datei|key_mode|
|---|---|---|
|1|WAL-Segment|0|
|2|SSTable|0|
|3|Manifest-Log|1|
|4|Textsegment|0|
|5|Vektor-Snapshot|0|
|6|Journal|1|
|7|Gate-Log|1|
|8|Ledger|1|
|9|Scope-Datei|0|
|10|Tresor|1 (Slots tragen eigene MACs)|

Unbekannte `format_version`, `kind`, `alg_ids` ⇒ `Corrupt`. Blöcke im Körper: `Block := u32 len ‖ ciphertext(len) ‖ aead_tag(16)`; Nonce `0x00000000 ‖ u64(Blockindex)`; AAD `kind ‖ file_id ‖ u64(Blockindex)`. Appendlogs ergänzen den Vorgänger-Hash in der AAD.

### F.2 HEADER.a / HEADER.b (1024 Byte)

```
0    8    magic          "CTXHDR\0\1"
8    2    format_version u16
10   1    kind           = 0x20
11   4    alg_ids        u8[4]
15   16   store_id       zufällig
31   8    generation     u64 (A/B: höhere gültige Generation gewinnt)
39   1    slot_count     u8 (1..4)
40   32   report_pub     Ed25519 öffentlicher Schlüssel
72   4×128 slots
584  16   check          SHA-256(Bytes[0..584))[0..16]  (Korruptionserkennung, keine Authentisierung)
600  32   header_mac     HMAC-SHA256(K_hdr, Bytes[0..584)), K_hdr = HKDF(K_master, "ctx/hdr/v1"); nach Unwrap prüfbar
632  392  padding        0
```

Slot (128 Byte): `slot_id(1) ‖ anchor_id(1) ‖ kdf_id(1) ‖ status(1: 0 leer, 1 aktiv, 2 entfernt) ‖ m_kib(u32) ‖ t(u8) ‖ p(u8) ‖ reserved(2) ‖ slot_salt(32) ‖ nonce(12) ‖ wrapped_master(48) ‖ zero(24)`.
`wrapped_master = AES-256-GCM-SIV(KEK, nonce, K_master)` (32 B Klartext + 16 B Tag). `KEK = Argon2id(passphrase, slot_salt, m, t, p) → 32 B`. **AAD des Wraps:** `"ctx/slot/v1" ‖ store_id ‖ slot_id ‖ anchor_id ‖ kdf_id ‖ u32(m) ‖ u8(t) ‖ u8(p) ‖ slot_salt`. Vor der Ableitung: `m ∈ [65536, 4194304]` KiB, `t ∈ [3, 16]`, `p ∈ [1, 16]` (D-06), sonst `Corrupt`. Entfernen eines Slots: alle 128 Bytes auf 0, `status = 2` nur im Statusbyte, `fsync`.

### F.3 Tresor (`vault/vault.dat`)

Kopf nach F.1 (`kind = 10`), danach Metablock (64 B): `capacity(u64, Zweierpotenz) ‖ live(u64) ‖ tombstones(u64) ‖ generation(u64) ‖ zero(32)`, danach `capacity` Slots à 80 Byte, Offset `128 + 64 + i·80` (ausgerichtet, damit ein Slot-Update in einem Schreibvorgang atomar gegenüber Teilschreibvorgängen der Startprobe ist, D-56).

```
Slot (80 B):
0   1   slot_type   0x01 T, 0x02 C, 0x03 S, 0x04 D, 0x05 F  (0x00 = leer)
1   16  tag         tenant_tag | coll_tag | subject_tag | doc_tag | file_id
17  32  payload     T/C/S/D: Salt (zufällig, 32 B)
                    F: Dateischlüssel ⊕ HKDF-Expand(K_enc, "ctx/fkmask/v1" ‖ file_id, 32)
49  1   status      0x00 leer, 0x01 aktiv, 0x02 Grabstein
50  6   gen         u48, Slot-Generation
56  24  mac         HMAC-SHA256(K_vm, Bytes[0..56))[0..24], K_vm = HKDF(K_enc, "ctx/vault-mac/v1")
```

Position: `i = BLAKE3(slot_type ‖ tag)[0..8] mod capacity`, lineares Sondieren; Suche endet bei `status = 0x00`. **Vernichten:** alle 80 Bytes auf 0, danach Byte 49 auf `0x02` (Grabstein); `mac` bleibt 0; Leser ignorieren Grabsteine. Verifikation: Bytes 0..49 und 50..80 sind Null. Wachstum: Rehash in neue Datei bei `(live + tombstones) / capacity > 0,7`. Slot mit ungültigem MAC ⇒ `Corrupt`.

### F.4 WAL und Manifest

**WAL-Segment** (`kind = 1`): Kopf nach F.1, danach Segmentkopf `prev_mac(32) ‖ first_seq(u64)`; dann Frames:

```
Frame := len(u32) ‖ seq(u64) ‖ ct(len) ‖ aead_tag(16) ‖ mac(32)
ct      = AEAD(Dateischlüssel, nonce = 0x00000000 ‖ u64(Framezähler),
               AAD = "wal" ‖ file_id ‖ u64(Framezähler) ‖ prev_mac, Klartext = Batch)
mac_n   = HMAC-SHA256(K_wal, prev_mac ‖ len ‖ seq ‖ ct ‖ aead_tag),   prev_mac_0 = Segmentkopf.prev_mac
```

Das erste Segment beginnt mit `prev_mac = 0³²`. **Torn Write:** Ein unvollständiger oder ungültiger letzter Frame wird verworfen, wenn alle folgenden Bytes der Datei 0 sind; sonst `Corrupt`. `Batch` = Liste `(key, value_envelope | tombstone)` mit `commit_seq`.

**Manifest-Log** (`kind = 3`): Appendlog mit abgeleitetem Dateischlüssel; Eintrag `len(u32) ‖ type(u8) ‖ payload ‖ aead_tag(16) ‖ chain(32)` mit `chain = SHA-256(chain_prev ‖ Eintrag)`. Typen: `0x01 AddFile{file_id, kind, level(u8), lo(65), hi(65), min_seq, max_seq, size}`, `0x02 RemoveFile{file_id}`, `0x03 WalFloor{seq(u64), tail_mac(32), removed_file_ids}`, `0x04 SeqMark{next_seq}`, `0x05 Collection{coll_tag, name_tag, options_envelope}`. `WalFloor` bedeutet: Alle Frames mit `seq ≤ seq` sind in SSTables dauerhaft; Segmente mit letzter Sequenz `≤ seq` sind entfernt; das erste verbleibende Segment hat `prev_mac = tail_mac`. Recovery prüft diese Gleichheit.

### F.5 SSTable (`kind = 2`)

Kopf nach F.1; Datenblöcke (nominal 32 KiB Klartext) mit Einträgen `klen(u16) ‖ key ‖ vlen(u32) ‖ value`, `value` = Envelope v2 (F.6) oder Tombstone (`vlen = 0`), aufsteigend nach `key` (9.1). Danach Indexblock (`key_lo`, Offset, Länge je Datenblock), Bloomblock (Schlüsselhash über `tenant‖coll‖cluster‖doc_tag`-Präfix, 10 Bit/Schlüssel), Statistikblock (`min_seq`, `max_seq`, Zähler), Footer (fest 64 B, nicht verschlüsselt außer Tag): `index_off(u64) ‖ bloom_off(u64) ‖ stats_off(u64) ‖ block_count(u64) ‖ footer_tag(16) ‖ magic(8)`. Beim Rewrite werden Envelopes **byteidentisch** kopiert (I-3).

### F.6 Record-Envelope v2

```
0   1   version   = 2
1   1   alg_id    = 1 (AES-256-GCM-SIV)
2   16  doc_tag
18  12  nonce     96 Bit OS-CSPRNG
30  n   ciphertext
30+n 16 aead_tag
AAD = "ctx/rec/v2" ‖ key_bytes   (key_bytes = vollständiger SSTable-Schlüssel inkl. !commit_seq)
Klartext (Doc): u8 type ‖ Felder; Identität: DocRef (subject?, key) als erstes Feld (für IdCollision-Prüfung)
```

### F.7 Textsegment (`kind = 4`)

Kopf nach F.1, dann Segmentheader-Block: `bucket(u8) ‖ bucket_count(u8) ‖ analyzer_id(u8) ‖ analyzer_version(u16) ‖ lang(u8[3]) ‖ flags(u8: bit0 positions) ‖ doc_count(u32) ‖ term_count(u32) ‖ total_len(u64) ‖ reserved(8)`. Dann:

- **Wörterbuch:** sortierte Einträge `term_tag(16) ‖ df(u32) ‖ posting_off(u64) ‖ posting_len(u32)`.
- **Postings:** Blöcke zu 128 Dokumenten: `doc_ord`-Deltas (VByte), `tf` (VByte), optional Positionen (Delta/VByte); Blockkopf mit `max_doc_ord`, `max_tf_norm(u16)` für Block-Max-WAND.
- **Dokumenttabelle:** je `doc_ord(u32)`: `doclen(u32) ‖ binding_len(u16) ‖ binding_envelope`; `binding_envelope` = Envelope v2 unter dem DEK des Dokuments, Klartext `cluster_tag ‖ doc_tag`, **AAD** = `"ctx/tbind/v1" ‖ file_id ‖ u32(doc_ord)`.
- **Lösch-Bitmap** (`ceil(doc_count/8)` Byte), **Statistikblock**.

### F.8 Vektor-Snapshot (`kind = 5`)

Kopf nach F.1; Header: `dim(u16) ‖ metric(u8) ‖ M(u16) ‖ ef_c(u16) ‖ slot_count(u32) ‖ entry_slot(u32) ‖ max_level(u8)`. **Slot-Abschnitt:** je Slot `slot(u32) ‖ envelope` (Envelope v2 unter dem DEK des Dokuments; Klartext: Vektor in allen Repräsentationen nach D-86 ‖ `cluster_tag ‖ doc_tag ‖ chunk(u32)`). **Graph-Abschnitt:** Adjazenzlisten je Level, nur Slotnummern, unter dem Dateischlüssel (Block-AEAD). Slots auf der Freiliste tragen keinen Envelope.

### F.9 Journal, Gate-Log, Ledger

Alle drei: Appendlogs mit abgeleitetem Dateischlüssel; Eintrag `len(u32) ‖ ct ‖ aead_tag(16) ‖ chain(32)`, `chain = SHA-256(chain_prev ‖ len ‖ ct ‖ aead_tag)`.

- **Journal-Klartext:** `erasure_id(16) ‖ state(u8) ‖ time(u64) ‖ commitment(16) ‖ target_type(u8) ‖ retention_code(u8) ‖ scope_tags(list lo/hi) ‖ counters ‖ witness_hashes`. Zustände: 1 Requested+Intent, 2 Gated, 3 KeyDestroyed, 4 Purging, 5 Verifying, 6 Completed, 7 Pending, 8 Refused.
- **Gate-Log-Klartext:** `Add{gate_id, variant(u8: 1 Erased, 2 Restricted), lo(65), hi(65), gated_seq(u64), erasure_id}` oder `Remove{gate_id}`.
- **Ledger-Eintrag:** `commitment(16) ‖ gated_seq(u64) ‖ completed_at(u64) ‖ range_count(u16) ‖ (lo ‖ hi)*`; zusätzlich `ledger_mac = HMAC(K_ledger, chain)` je Eintrag, damit eine Fälschung außerhalb des Verzeichnisses erkannt wird.

### F.10 Bericht (kanonisches JSON)

Kanonisierung: UTF-8 NFC, Objektschlüssel aufsteigend sortiert, keine Leerzeichen außerhalb von Strings, keine Gleitkommazahlen (Ganzzahlen, Dezimalstrings für Dauern), Zeiten als Ganzzahl-Mikrosekunden UTC. Signatur: Ed25519 über `"ctx/report/v1" ‖ kanonische_Bytes_ohne_signature`; Feld `signature` (Base64url) und `pub_fingerprint` (SHA-256 von `report_pub`, 16 Byte hex). Pflichtfelder siehe 11.7 und Anhang D.

---

## Anhang G – DSFA-Baustein (Lieferdokument, keine Rechtsbewertung)

`docs/DSFA-BAUSTEIN.md` enthält, vom Betreiber zu übernehmen und anzupassen:

1. **Datenflüsse:** Contextra läuft lokal; keine Übermittlung durch Contextra (HR-1..4).
2. **Technische Maßnahmen:** Pflichtverschlüsselung, Keyslots, Schlüsselhierarchie, Dateischlüssel-Vernichtung, Gate, Purge, Messung, Restore-Guard (jeweils mit Verweis auf die Tests).
3. **Restrisiken:** Anhang D, mit Bedingungen.
4. **Begründung von `backup_retention`:** Vorlage mit Feldern: Aufbewahrungsdauer der Sicherungen, Zweckbindung „Restore-Schutz“, Datenarten im Ledger (Commitments, Bereichs-Tags, Sequenzen, Zeiten), Löschfrist (E-L4).
5. **Zuordnung Fachverfahren → `RetentionCode`** (R20).
6. **Abbildung auf BSI CON.6 und ISO/IEC 27001:2022 A.8.10/A.8.24** (R21).
7. **Rollen:** Betreiber als Verantwortlicher (6.1, E-L6).
8. **Hinweis:** Keine Rechtsberatung; Wertung durch den Betreiber.

---

## Anhang H – Änderungen 4.1.0 (Architektur- und Strukturprüfung)

Prüfgrundlage: HEAD `f8a3bbb24`, statisch **[V]**, ohne Build. **Nicht geändert:** Datenformate (Anhang F), Löschsemantik (11), Schlüsselhierarchie (10), Rechtsrahmen (6). **Vom Eigentümer zu bestätigen:** D-88 bis D-92 (Revisionsentscheidungen).

|Befund|Kern der Änderung|Neu/geändert|
|---|---|---|
|A-01 Fassade widerspricht §7|Schnitt `api`/`wiring`/`error`, Entfallliste, Builder nach `engine`|F-26, D-89, 8.14, WP-1.5|
|A-02 Matrix lückenhaft und im Code verletzt|`engine→sys` nur Prozesshärtung; `vector` ohne `sys`/`mvcc`; `text→crypto`; Zeile für `contextra`; kein Glob-Re-Export|F-27, D-88, AR-10, 5.3|
|A-03 Ringangaben handgepflegt und falsch|Kopf-Pflicht, Prüfung gegen 5.1/5.3|F-28, D-91, AR-11, T-ARCH-5|
|A-04 Befunde F-07/F-13 überholt|Korrektur mit Commit-Bezug; Sentinel in `mvcc/snapshot.rs:314`|F-29|
|A-05 Namen ohne Aussage, gemischte Modulform|Namensregeln, `instance`, kein Alias|F-30, D-90, AR-13, T-ARCH-7|
|A-06 Umfang gegenüber Soll nicht beziffert|Zählungen je Rückbaumodul und Restmuster|F-31, D-84 (Ergänzung)|
|A-07 P5 ohne namentliche Ausnahmen|Ausnahmeliste: `process_hardening`, Feature `testing`|P5|
|A-08 Doppelte Wahrheit durch Folgedokumente|Löschen statt Angleichen; eine `AGENTS.md`; Verläufe nicht im Baum|D-92, Anhang E, T-ARCH-8|
|A-09 Aufwand|WP-1.5 (+3 PW)|15.4|

Unverändert aus 4.0.0: R-01 bis R-17 (nachfolgender Abschnitt „Änderungen gegenüber 3.0.0“) bleiben gültig.

**Ende der Spezifikation 4.1.0.**

---

## Anhang H – Änderungen gegenüber 3.0.0 (Review-Abgleich)

|Befund|Kern der Änderung|Neu/geändert|
|---|---|---|
|R-01 Klasse K deckt nur einen Teil|`key_coverage` je Schicht; Aussage über „das System“ entfällt; Postings entkoppelt|D-01, D-69, 12.5, T-KEYCOV-1|
|R-02 Locator-Region verstreut Ziele|Locator entfällt; `DocRef` mit Betroffenem|D-31, D-47, D-70, 9.1|
|R-03 Textsegmente nach Einfügezeit verteilt|Ehrliches P11, adaptive Segmente, Bucket-Haken, Messung|D-49, D-71, BM-ERASE-WA-TEXT|
|R-04 Messklassen in Produktion nicht einlösbar|MK1–MK3, EV0–EV4, `field_measurement` vs. `lab_evidence`, `ctx-verify scan`, `Scanner<Armed>`|D-35, D-72, 12|
|R-05 WAL-Rewrite widerspricht Bitgleichheit|WAL nie umschreiben; `WalFloor`|D-73, I-3, F.4|
|R-06 Restore-Guard|`backup_retention`, Bereiche, `gated_seq`, Zeitregression|D-48, D-74, 11.8|
|R-07 Klartext-Pfade, keine Mandantendimension|Pfade mit Tags|D-75, 5.6|
|R-08 „Final“ vs. G0-Fallback|Status mit Vorbehalten, messbares G0, Fallbackleiter|D-76, 11.9|
|R-09 HNSW Nullen zerstört Navigation|Surrogat, Batch-Kantenreparatur, deterministischer Rebuild|D-18, D-77, D-86|
|R-10 Gate serialisiert Writer, Kompaktion blockiert|Gate-Gruppen-Commit, kooperative Kompaktion|D-57, D-78, I-16, I-18|
|R-11 Tresor O(m), RAM-Index|Schlüsselhierarchie `T/C/S/D`, Hash-Tabelle|D-33, D-79|
|R-12 Krypto-Details|Argon2-Grenzen, Slot-AAD, `K_sign` abgeleitet, Algorithmensatz, Dateischlüssel|D-06, D-52, D-60, D-80, D-85|
|R-13 Rechtsrahmen|ProdHaftG, CRA, EDPB 02/2026, DE-Normen, H9|6.2, 6.5, 6.6, 6.7|
|R-14 Textsuche: Analyzer, Phrasen, Pruning|`de/1`, Positionen, Block-Max|D-81, 9.7|
|R-15 Toolchain-Pin|MSRV getrennt von Build-Toolchain|D-82|
|R-16 Spec-Hygiene|Zählungen, Begriffe, Quellenlage, Formate und Plan in dieser Datei|0, KF-9, 15.4, Anhang F|
|R-17 Betrieb und Leistung|Kapazitätsgrenzen, Benchmarks, Gesundheit|D-83, D-87, 14, 15.2|

Danach neu gegenüber der Review: Dateischlüssel-Vernichtung (D-85), Textbindung unter DEK (9.7), Fallbackleiter mit Stufen (11.9), Rückverfolgbarkeits-Test T-TRACE-1 (15.1), Umsetzungsplan (15.4), Byte-Formate (Anhang F).

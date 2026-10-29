# Abgleichsbericht: Referenzen und Kennungen der Löschkette

**Datum:** 2026-10-29
**Repository:** `tfufuz1/contextra`
**Prüfgegenstand:** `docs/SPEC.md` und `docs/PLAN.md`
**Status:** Bestandsaufnahme aller verwaisten, unvollständig definierten oder mehrdeutigen Kennungen (P-xx, O-xx, REQ-*, AC-*, E-*, H-*, §-Verweise) gem. Auftrag.

---

## 1. Übersicht und Methodik

Dieser Abgleichsbericht dokumentiert alle im Repository verwendeten Kennungen und Querverweise in `docs/SPEC.md` und `docs/PLAN.md`. Ziel ist die Herstellung vollständiger Referenzierbarkeit ohne eigenmächtige Umnummerierungen bestehender, gültiger IDs.

### Kategorien der Befunde:
- **definiert:** Kennung ist an einer normativen Stelle im Dokument oder Katalog definiert.
- **nicht definiert:** Kennung wird verwendet, besitzt aber keine explizite Definition oder zeigt auf ein nicht existierendes Kapitel (z. B. `P-xx`, `§25.1`).
- **Kollision:** Kennung wird für zwei unterschiedliche Sachverhalte verwendet (z. B. `O-12`).

---

## 2. Abgleichstabelle aller Kennungen und Verweise

| Kennung | Datei | Zeile | Fundstelle / Kontext | Status | Vorschlag |
|---|---|---|---|---|---|
| `REQ-CHG-2` | `docs/SPEC.md` | 50 | 14. **Ein PR = ein Arbeitspaket = höchstens ein Crate-Schnitt**; nach jedem Schnitt Build, Modul-Prüfung, Zeilenzählu... | definiert | In Anforderungskatalog / SPEC geführt. |
| `E-3.8` | `docs/SPEC.md` | 438 | \|W9\|**Dokumentation am Code:** `MODULE.md` ≤ 120 Zeilen je Crate, `//!`-Kopf mit Zweck und Invarianten je Datei (im... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `O-02` | `docs/SPEC.md` | 442 | **Zukunftsfähigkeit der Löschkette:** Rechtsauslegung, Fristen und Aufsichtspraxis ändern sich häufiger als Code. Des... | definiert | Dokumentiert in §16. |
| `O-05` | `docs/SPEC.md` | 539 | \|`avv-generator`\|AVV-Vorlagen (Art. 28)\|`experimental`; 196 Code-Z.; **keine Rechtsberatung**\|Vorlagen versionier... | definiert | Dokumentiert in §16. |
| `O-05` | `docs/SPEC.md` | 598 | \|`contextra-avv-generator`\|0\|196\|1\|–\|–\|0/0/0; allow 0\|grün\|rot\|grün\|Vorlagen versionieren; juristische Prü... | definiert | Dokumentiert in §16. |
| `O-01` | `docs/SPEC.md` | 788 | Technische Anforderungen leiten sich ab aus Art. 5 Abs. 1 lit. e und f (Speicherbegrenzung, Integrität/Vertraulichkei... | definiert | Dokumentiert in §16. |
| `O-06` | `docs/SPEC.md` | 788 | Technische Anforderungen leiten sich ab aus Art. 5 Abs. 1 lit. e und f (Speicherbegrenzung, Integrität/Vertraulichkei... | definiert | Dokumentiert in §16. |
| `O-04` | `docs/SPEC.md` | 881 | Nicht durch Contextra kontrollierbar und deshalb **ausgewiesen**, nie behauptet: externe Sicherungen des Betreibers, ... | definiert | Dokumentiert in §16. |
| `E-1` | `docs/SPEC.md` | 919 | \|E-1\|`erasure` hängt nur von `types`, `ports`, `crypto`; nie von `engine`, `db`, `store`, `vector`, `text`, `graph`... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-2` | `docs/SPEC.md` | 920 | \|E-2\|Messung und Reinigung einer Schicht liegen im Crate, das die Daten hält; `erasure` ruft nur `LayerScanner` und... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-3` | `docs/SPEC.md` | 921 | \|E-3\|Die Lesesperre wirkt über `ErasedSetPort` in **jedem** Lesepfad der Besitzer; ein Lesepfad ohne Prüfung ist ei... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-4` | `docs/SPEC.md` | 922 | \|E-4\|Das Berichtsformat (`DeletionProof` V5) liegt genau einmal in `crypto`; `graph` oder `engine` definieren keine... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-5` | `docs/SPEC.md` | 923 | \|E-5\|Zeit, Zufall, IDs, Dateizugriff kommen über `Clock`, `Rng`, `IdGen`, `Vfs`, `DurableLog`; Schlüssel- und Salt-... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-6` | `docs/SPEC.md` | 924 | \|E-6\|Der Koordinator **lehnt den Start ab**, wenn eine Schicht, in der Daten liegen können, weder Scanner noch doku... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-7` | `docs/SPEC.md` | 925 | \|E-7\|Ein Profil ohne `erasure` (z. B. **Search**) behauptet nichts über Löschung; die Fassade stellt dann weder `fo... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-1` | `docs/SPEC.md` | 927 | **Verdrahtung (Pseudocode, nicht kompiliert; Namen sind Richtwerte, normativ sind E-1 bis E-7):** | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-7` | `docs/SPEC.md` | 927 | **Verdrahtung (Pseudocode, nicht kompiliert; Namen sind Richtwerte, normativ sind E-1 bis E-7):** | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-6` | `docs/SPEC.md` | 942 | .build()?;   // Err, wenn Schicht ohne Scanner und ohne Ausschluss (E-6) | definiert | Entscheidungs- / Entkopplungs-ID. |
| `O-02` | `docs/SPEC.md` | 959 | \|Gesetzliche Aufbewahrungspflichten\|Löschung weicht Aufbewahrung: `Restricted{basis, until}`\|Legal-Hold-Mechanismu... | definiert | Dokumentiert in §16. |
| `O-05` | `docs/SPEC.md` | 961 | \|Art. 28 Auftragsverarbeitung\|AVV-Unterlagen\|`avv-generator`\|Golden-Files; juristische Prüfung (O-05)\|`experimen... | definiert | Dokumentiert in §16. |
| `O-06` | `docs/SPEC.md` | 965 | \|VO (EU) 2024/1689 (KI-Verordnung)\|Einordnung je Anwendungsfall\|Integrator (O-06); Bausteine liefern Protokollieru... | definiert | Dokumentiert in §16. |
| `O-11` | `docs/SPEC.md` | 966 | \|VO (EU) 2024/2847 (Cyber Resilience Act)\|Anforderungen an Produkte mit digitalen Elementen, Schwachstellenmanageme... | definiert | Dokumentiert in §16. |
| `O-01` | `docs/SPEC.md` | 1176 | \|O-01\|Rechtliche Einordnung von Embeddings als personenbezogene Daten (vorsorglich ja) und von Backups (Aufsichtsau... | definiert | Dokumentiert in §16. |
| `O-02` | `docs/SPEC.md` | 1177 | \|O-02\|Aufbewahrungsfristen und Legal-Hold-Katalog für Zielbranchen\|externe Rechtsprüfung\| | definiert | Dokumentiert in §16. |
| `O-03` | `docs/SPEC.md` | 1178 | \|O-03\|Marktprüfung der Hypothesen aus §2.1 (Gastronomie, Shop, Kommune)\|Produkt\| | definiert | Dokumentiert in §16. |
| `O-04` | `docs/SPEC.md` | 1179 | \|O-04\|Medien-/Datenträgerlöschung nach anerkannten Normen ist Betreiberpflicht; Abgrenzung im Bericht\|Betreiber/Re... | definiert | Dokumentiert in §16. |
| `O-05` | `docs/SPEC.md` | 1180 | \|O-05\|Juristische Prüfung der AVV-Vorlagen\|externe Rechtsprüfung\| | definiert | Dokumentiert in §16. |
| `O-06` | `docs/SPEC.md` | 1181 | \|O-06\|Einordnung nach EU-KI-Verordnung (VO (EU) 2024/1689) für Integratoren je Anwendungsfall\|Integrator/Recht\| | definiert | Dokumentiert in §16. |
| `O-07` | `docs/SPEC.md` | 1182 | \|O-07\|Hardware-Referenzklassen und Modellgrößen für Benchmarks festschreiben\|Technik\| | definiert | Dokumentiert in §16. |
| `O-08` | `docs/SPEC.md` | 1183 | \|O-08\|Welche `xtask`-Prüfungen sind wirksam (Mutations-/Selbsttest)?\|WP 3.6\| | definiert | Dokumentiert in §16. |
| `O-09` | `docs/SPEC.md` | 1184 | \|O-09\|Notwendigkeit von DiskANN/RaBitQ im Designpunkt ≤ 10⁷ Vektoren\|Benchmark\| | definiert | Dokumentiert in §16. |
| `O-10` | `docs/SPEC.md` | 1185 | \|O-10\|macOS und Windows als unterstützte Entwicklungs-/Edge-Plattformen\|Technik\| | definiert | Dokumentiert in §16. |
| `O-11` | `docs/SPEC.md` | 1186 | \|O-11\|Einordnung der Bibliothek und der Integrator-Produkte nach VO (EU) 2024/2847 (Cyber Resilience Act): Pflichte... | definiert | Dokumentiert in §16. |
| `O-12` | `docs/SPEC.md` | 1187 | \|O-12\|Reproduzierbarkeit der Baseline: Commit 8f8fa7cd im flachen Klon nicht auflösbar; Zielstand für die erneute B... | definiert | Dokumentiert in §16. |
| `O-13` | `docs/SPEC.md` | 1188 | \|O-13\|Ursache der 33 Clippy-Fehlschläge (gemeinsamer Abbruch oder echte Lints) anhand der rohen Logs\|Technik (WP 0... | definiert | Dokumentiert in §16. |
| `O-14` | `docs/SPEC.md` | 1189 | \|O-14\|Welche Tests verursachen die 13 Timeouts (Fixture, echte Last, Deadlock)?\|Technik (WP 0.7)\| | definiert | Dokumentiert in §16. |
| `O-15` | `docs/SPEC.md` | 1190 | \|O-15\|Dokumentationssprache: `rustdoc` englisch oder deutsch; Integrationsleitfäden deutsch\|Produkt (E-3.8)\| | definiert | Dokumentiert in §16. |
| `E-3.8` | `docs/SPEC.md` | 1190 | \|O-15\|Dokumentationssprache: `rustdoc` englisch oder deutsch; Integrationsleitfäden deutsch\|Produkt (E-3.8)\| | definiert | Entscheidungs- / Entkopplungs-ID. |
| `§19` | `docs/SPEC.md` | 1234 | \|§19–§26 Kataloge\|ersetzt durch §1.3, §15, §16; REQ-Katalog wird in WP 0.3 nach `docs/spec/requirements.toml` überf... | nicht definiert | Abschnitt existiert in 3.1 nicht mehr als Hauptkapitel; Verweis prüfen. |
| `§26` | `docs/SPEC.md` | 1234 | \|§19–§26 Kataloge\|ersetzt durch §1.3, §15, §16; REQ-Katalog wird in WP 0.3 nach `docs/spec/requirements.toml` überf... | nicht definiert | Abschnitt existiert in 3.1 nicht mehr als Hauptkapitel; Verweis prüfen. |
| `O-11` | `docs/SPEC.md` | 1242 | \|**3.1**\|**28.10.2026**\|Baseline (E2, teilweise), Metrik-Datei und Scan eingearbeitet (§1.2a, U-15 bis U-27); Proj... | definiert | Dokumentiert in §16. |
| `O-15` | `docs/SPEC.md` | 1242 | \|**3.1**\|**28.10.2026**\|Baseline (E2, teilweise), Metrik-Datei und Scan eingearbeitet (§1.2a, U-15 bis U-27); Proj... | definiert | Dokumentiert in §16. |
| `§19` | `docs/SPEC.md` | 1246 | **Herkunft.** Textauszug aus dem Repo-Stand `docs/SPEC.md` (Fassung 2.5, auf 2.4 aufbauend; SHA-256 der Quelldatei be... | nicht definiert | Abschnitt existiert in 3.1 nicht mehr als Hauptkapitel; Verweis prüfen. |
| `§21` | `docs/SPEC.md` | 1246 | **Herkunft.** Textauszug aus dem Repo-Stand `docs/SPEC.md` (Fassung 2.5, auf 2.4 aufbauend; SHA-256 der Quelldatei be... | nicht definiert | Abschnitt existiert in 3.1 nicht mehr als Hauptkapitel; Verweis prüfen. |
| `§22` | `docs/SPEC.md` | 1246 | **Herkunft.** Textauszug aus dem Repo-Stand `docs/SPEC.md` (Fassung 2.5, auf 2.4 aufbauend; SHA-256 der Quelldatei be... | nicht definiert | Abschnitt existiert in 3.1 nicht mehr als Hauptkapitel; Verweis prüfen. |
| `AC-1` | `docs/SPEC.md` | 1246 | **Herkunft.** Textauszug aus dem Repo-Stand `docs/SPEC.md` (Fassung 2.5, auf 2.4 aufbauend; SHA-256 der Quelldatei be... | definiert | In Abnahmekriterien §12.12 geführt. |
| `REQ-PRF-4` | `docs/SPEC.md` | 1307 | \|A2\|**Eine Typfamilie je Konzept.** Berichts-/Beweistypen existieren nur in `contextra-crypto`.\|Gate (REQ-PRF-4)\| | definiert | In Anforderungskatalog / SPEC geführt. |
| `§19` | `docs/SPEC.md` | 1319 | Spalten: ID · Klasse · Invariante · Mechanismus (Kürzel §1.4) · Anforderung (§19). `INV-SHRED-1` der Fassung 1.3 blei... | nicht definiert | Abschnitt existiert in 3.1 nicht mehr als Hauptkapitel; Verweis prüfen. |
| `REQ-ARCH-003` | `docs/SPEC.md` | 1323 | \|INV-CORE-ISOLATION-1\|K\|Umsetzung von P30: Gate über `cargo metadata` (aufgelöst, **inkl. Default-Features der Ker... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-KEY-001` | `docs/SPEC.md` | 1324 | \|INV-KMS-1\|K\|Jeder Schlüssel stammt von einem `KmsProvider`; kein Modul hält Master-Material außerhalb des Provide... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-001` | `docs/SPEC.md` | 1325 | \|INV-DELETION-WIRE-1\|K\|`forget` löst **alle zehn** Schichten über die Composition Root aus: acht werden gemessen, ... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-MVCC-001` | `docs/SPEC.md` | 1326 | \|INV-SSI-ACTIVE-1\|K\|SSI ist im Produktionspfad aktiv; Read-Sets inkl. Prefix-/Range-Phantomschutz werden bei jedem... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-MVCC-002` | `docs/SPEC.md` | 1327 | \|INV-INDEX-SNAPSHOT-1\|K\|Jeder Indexeintrag trägt `born_seq`/`died_seq`; Suchen am Snapshot `S` sehen nur `born_seq... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-GOV-004` | `docs/SPEC.md` | 1328 | \|INV-DOC-SINGLE-SOURCE-1\|K\|Jede Tatsache (Konstante, Ring, Reifegrad, Format, ID) hat genau eine kanonische Quelle... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-ARCH-015` | `docs/SPEC.md` | 1329 | \|INV-RING0-DETERMINISM-1\|K\|Ring 0 ist deterministisch: kein I/O außerhalb der Ports (insb. `Vfs`), keine Uhr/kein ... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DUR-012` | `docs/SPEC.md` | 1330 | \|INV-NONDET-1\|K\|Ergebnisse nichtdeterministischer Quellen (Embedding-Inferenz, LLM-Ausgabe, Zufall) werden als **W... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DUR-016` | `docs/SPEC.md` | 1331 | \|INV-WAL-SSOT-1\|K\|Jeder persistente **Anwendungszustand** ist aus WAL + Manifest rekonstruierbar; Indexe sind abge... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DUR-011` | `docs/SPEC.md` | 1332 | \|INV-ATOMIC-IDX-1\|K\|Metadaten- und Indexänderung gehören zu **einem** WAL-Batch; Recovery wiederholt Index-Intents... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DUR-011` | `docs/SPEC.md` | 1333 | \|INV-INDEX-APPLY-1\|K\|Indexe werden von **einem** Applier strikt in WAL-Reihenfolge angewendet; HNSW-Level folgen a... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-TEN-001` | `docs/SPEC.md` | 1334 | \|INV-TENANT-1\|K\|Tenant-Ableitung aus der Transaktion; jede Schlüsselkonstruktion enthält das Tenant-Präfix.\|T+P+F... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-IDX-001` | `docs/SPEC.md` | 1335 | \|INV-TENANT-IDX-1\|K\|Vektor- und Text-Indexe sind **physisch je (Tenant, Collection[, Modell-Hash])** getrennt; kei... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-IDX-010` | `docs/SPEC.md` | 1336 | \|INV-KEYSPACE-1\|K\|LSM-Schlüssel, Index-IDs und Dateinamen enthalten nur opake IDs/Hashes, nie personenbezogene Inh... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-007` | `docs/SPEC.md` | 1337 | \|INV-DERIVED-1\|K\|`forget` kaskadiert über die Lineage auf Ableitungen (Chunk-Präfixe, Zusammenfassungen, Konsolidi... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-KEY-003` | `docs/SPEC.md` | 1338 | \|INV-ENCRYPT-REST-1\|K\|Nutzdaten sind at-rest AEAD-verschlüsselt in zwei Ebenen (§12.2): Engine-Schlüssel `K_eng` (... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-KEY-004` | `docs/SPEC.md` | 1339 | \|INV-ENCRYPT-REST-2\|K\|Per `mmap` zero-copy gelesene Dateien sind nur zulässig, wenn `doctor` ein verschlüsseltes V... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-ARCH-016` | `docs/SPEC.md` | 1340 | \|INV-FAILCLOSED-1\|K\|Unklarer Zustand ⇒ Ablehnung, nie Raten.\|C+I\|REQ-ARCH-016\| | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-KEY-002` | `docs/SPEC.md` | 1341 | \|INV-KDF-1\|K\|Passphrasen-Schlüssel über Argon2id mit zufälligem, persistiertem Salt und persistierten Parametern (... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DUR-002` | `docs/SPEC.md` | 1342 | \|INV-FSYNC-1\|K\|Schlägt `fsync`/`fdatasync` fehl, ist der Handle unbestimmt: kein Ack, kein Retry auf demselben Han... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DUR-001` | `docs/SPEC.md` | 1343 | \|INV-DURABILITY-PRIM-1\|K\|Der Commit-Punkt nutzt die plattformgerechte Durability-Primitive (§7.7); „fsync“ ohne di... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DUR-003` | `docs/SPEC.md` | 1344 | \|INV-ENOSPC-1\|K\|Volles Medium hinterlässt keinen Phantom-Eintrag und keinen halb angewendeten Batch; WAL-Segmente ... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DUR-004` | `docs/SPEC.md` | 1345 | \|INV-IO-1\|K\|Lesepfade geben I/O-Fehler als typisierte Fehler zurück. `mmap` nur für vollständig prüfsummenverifizi... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DUR-005` | `docs/SPEC.md` | 1346 | \|INV-FORMAT-1\|K\|Jede persistente Datei trägt Magic und Formatversion; neuere Version ⇒ Öffnen verweigert; Migratio... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-MVCC-004` | `docs/SPEC.md` | 1347 | \|INV-LOCK-ORDER-1\|K\|Vollständige, geordnete Sperrenhierarchie; keine Sperr-Guards über `.await` (§8.2).\|L+G\|REQ-... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-IDX-002` | `docs/SPEC.md` | 1348 | \|INV-HNSW-SLOT-1\|K\|Arena-Slots gelöschter HNSW-Knoten werden nicht wiederverwendet, solange veraltete Kantenrefere... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-IDX-011` | `docs/SPEC.md` | 1349 | \|INV-EMBED-1\|K\|Kein stiller Platzhalter-Embedder; Mock nur hinter `cfg(test)`/`test-utils`; `test-utils` in keinem... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-IDX-011` | `docs/SPEC.md` | 1350 | \|INV-EMBED-2\|K\|Collection persistiert `embedding_model_hash` und Dimension; abweichender Hash wird abgewiesen (§11... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-FAC-003` | `docs/SPEC.md` | 1351 | \|INV-WIRING-2\|K\|Sicherheits-Ports (Liste in `governance/security_ports.toml`, §5.2) haben in Produktion eine **rea... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-023` | `docs/SPEC.md` | 1352 | \|INV-DELETION-PORTS-1\|K\|Die Löschpfad-Ports `LayerScanner`, `Purger`, `ErasureJournal`, `Vfs`, `DurableLog`, `KmsP... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-003` | `docs/SPEC.md` | 1353 | \|INV-PROOF-COVERAGE-1\|K\|Jede der zehn Schichten erscheint in einem Bericht genau einmal als belegt (`Structural`),... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-004` | `docs/SPEC.md` | 1354 | \|INV-WITNESS-MINT-1\|K\|`LayerWitness` entsteht ausschließlich in `ScanSession::finish` (Konstruktoren `pub(crate)`,... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-005` | `docs/SPEC.md` | 1355 | \|INV-PROOF-KEY-1\|K\|Der Signierschlüssel des Berichts stammt aus dem `KeyManager`/`KmsProvider`, nie aus einem frei... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-006` | `docs/SPEC.md` | 1356 | \|INV-CLAIMS-1\|K\|Externe Zusicherungen werden aus dem Bericht (`outcome`, `claims`, `excluded`) **erzeugt**, nie au... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-GOV-001` | `docs/SPEC.md` | 1357 | \|INV-PROVENANCE-1\|K\|Merges sind scope-treu, beschreibend benannt, serialisiert über eine Merge-Queue und auf dem M... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-PLT-001` | `docs/SPEC.md` | 1358 | \|INV-PLATFORM-1\|K\|Durabilitäts- und Löschgarantien gelten **nur auf der Garantieplattform** (§13.7); andere Plattf... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-MCP-002` | `docs/SPEC.md` | 1359 | \|INV-IPC-1\|K\|Genau ein Primärprozess hält das Datenverzeichnis; weitere Clients nutzen ausschließlich lokales IPC ... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-MCP-001` | `docs/SPEC.md` | 1360 | \|INV-STDOUT-1\|K\|`stdout` gehört ausschließlich dem MCP-Protokoll; Logging nach `stderr`/Datei; `clippy::print_stdo... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-MCP-007` | `docs/SPEC.md` | 1361 | \|INV-CANCEL-1\|K\|Anfrageabbruch (`notifications/cancelled`, Timeout, Disconnect) wirkt kooperativ bis in die Engine... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-MCP-005` | `docs/SPEC.md` | 1362 | \|INV-FORGET-GUARD-1\|K\|Breite Löschungen (Selektor, Collection, Tenant, > `forget_confirm_threshold` Dokumente) bra... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-MCP-006` | `docs/SPEC.md` | 1363 | \|INV-TRUST-1\|K\|Jeder Eintrag trägt `trust ∈ {user, agent, external}` und `source`; Recall liefert Inhalt strukturi... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-AUD-002` | `docs/SPEC.md` | 1364 | \|INV-LOG-1\|K\|Logs, Metriken, Fehlertexte und Panic-Meldungen enthalten keine Nutzinhalte, nur IDs und Zähler.\|G+P... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-COG-003` | `docs/SPEC.md` | 1365 | \|INV-RETENTION-1\|K\|Ranking-Abwertung löscht nie. Löschung entsteht ausschließlich durch ausdrückliche Aktion oder ... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-QA-001` | `docs/SPEC.md` | 1366 | \|INV-EVIDENCE-1\|K\|Jede Zusage ist ein Ziel, bis der Mechanismus sie maschinell belegt; ein nicht ausgeführtes oder... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-QA-002` | `docs/SPEC.md` | 1367 | \|INV-ORACLE-1\|K\|Jede Kerngarantie RB1–RB4 hat ein unabhängig entstandenes Prüfmaß.\|P\|REQ-QA-002\| | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-ARCH-014` | `docs/SPEC.md` | 1368 | \|INV-SCAN-1\|K\|Die öffentliche API enthält keine Funktion, keinen Typ und kein Feld, das Schlüsselmaterial (WAL-Int... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DUR-016` | `docs/SPEC.md` | 1369 | \|INV-SIDECAR-1\|K\|Außer WAL, Manifest, SSTables, Blob-Segmenten und Indexdateien existiert persistenter Zustand nur... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DUR-017` | `docs/SPEC.md` | 1370 | \|INV-EMERGENCY-DEL-1\|K\|Ein Betroffener ist in jedem Betriebszustand mindestens **sperrbar**: in `ReadOnly` (ENOSPC... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-030` | `docs/SPEC.md` | 1371 | \|INV-ASOF-ERASE-1\|K\|Historische Lesezugriffe (`as_of`, offene Snapshots, Checkpoint-Lesen) unterliegen der Lesespe... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-FAC-005` | `docs/SPEC.md` | 1372 | \|INV-BOOT-1\|K\|Der Start folgt der festen Phasenfolge B0–B9 (§13.5): Die Lesesperre (`ErasedSet`) ist armiert, bevo... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-KEY-011` | `docs/SPEC.md` | 1373 | \|INV-KEYPURPOSE-1\|K\|Jeder Schlüssel dient genau einem Zweck und entsteht aus dem Zweckregister (§12.3) mit eigenem... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-032` | `docs/SPEC.md` | 1374 | \|INV-RESTORE-HORIZON-1\|K\|`restore_checkpoint` und jeder Import verweigern fail-closed jedes **Schlüssel-Backup** (... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-N20` | `docs/SPEC.md` | 1375 | \|INV-DEL-ORDER-1\|K\|Die Schlüsselvernichtung (Schritt 5) beginnt erst, wenn Klasse P (Siegel 2) für jede Ebene-A-Sc... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-N21` | `docs/SPEC.md` | 1376 | \|INV-NO-RESURRECTION-1\|K\|Beim Start (B6/B7) und nach jedem Restore steht für jede `ErasedSet`-Einheit ab `Frozen` ... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-N22` | `docs/SPEC.md` | 1377 | \|INV-BACKUP-SPLIT-1\|K\|Kein Daten-Backup (Spur D) enthält Schlüsselmaterial; jedes Schlüssel-Backup (Spur K) trägt ... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-PRF-2` | `docs/SPEC.md` | 1378 | \|INV-CLASS-MEASURED-1\|K\|Ein Bericht weist höchstens die Klasse aus (L, P, C*, C1–C3, M), deren Voraussetzungen gem... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-035` | `docs/SPEC.md` | 1379 | \|INV-CLAIM-BIND-1\|K\|Außentexte entstehen nur aus einer festen Satzbibliothek; jeder Satz ist an einen gemessenen W... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-N23` | `docs/SPEC.md` | 1380 | \|INV-SURFACE-1\|K\|Jede persistente Datei- und Strukturklasse des Format-Registers (§7.11) steht in genau einer Zeil... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-PRF-4` | `docs/SPEC.md` | 1381 | \|INV-NO-DUMMY-PROOF-1\|K\|Kein Codepfad erzeugt einen `DeletionProof` ohne Scanner-Witness; `contextra_delete` und `... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-N24` | `docs/SPEC.md` | 1382 | \|INV-EPOCH-SEAL-1\|K\|Die Klassen C1 bis C3 gelten erst nach dem ersten Epoch Seal, der die Löschung umfasst; bis da... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-N16` | `docs/SPEC.md` | 1383 | \|INV-ARTKEY-1\|K\|Jedes Index-Artefakt hat genau einen ArtKey; der alte ArtKey wird bei einem Neuschreiben ohne Lösc... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-N40` | `docs/SPEC.md` | 1384 | \|INV-SEAL-HARDENING-1\|K\|Der Bericht weist `flash_hardening ∈ {None, PassphraseRotated, ModuleBound, ProviderAttest... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-N41` | `docs/SPEC.md` | 1385 | \|INV-HEAP-ZERO-1\|K\|Klartext von Nutzdaten (entschlüsselte Blöcke, Query-Puffer, Memtable-Werte, Arena-Slots, Cache... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-N42` | `docs/SPEC.md` | 1386 | \|INV-UNITDIR-1\|K\|Jede persistente Datei trägt ein unter `K_eng` verschlüsseltes UnitDirectory (Einheit → Blöcke); ... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-SBX-001` | `docs/SPEC.md` | 1387 | \|INV-ADMISSION-1\|R\|(`sandbox`) Ein `AdmittedModule` ist nur über `ModuleVerifier::verify` konstruierbar; SHA-256-D... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-SBX-002` | `docs/SPEC.md` | 1388 | \|INV-APPROVAL-1\|R\|(`sandbox`) `High`-/`Elevated`-Module laufen nicht vor `ApprovalStatus::Approved`.\|T+P+F\|REQ-S... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-SBX-003` | `docs/SPEC.md` | 1389 | \|INV-WASM-DET-1\|R\|(`sandbox`) Gleiches Modul, Eingabe und Fuel liefern auf x86-64 und aarch64 bit-identische Ergeb... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-PRV-002` | `docs/SPEC.md` | 1390 | \|INV-EGRESS-BUDGET-1\|R\|(`privacy`) Jede externe Wirkung ist budgetiert, auditiert und widerrufbar.\|I+P\|REQ-PRV-0... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-EXP-003` | `docs/SPEC.md` | 1391 | \|INV-LICENSE-1/-2/-3\|E\|(`license`) Verstoß/Ablauf/fehlende Aktivierung degradiert auf Basisfunktion, nie Datenverl... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-GOV-004` | `docs/SPEC.md` | 1393 | **Zählung:** `[K]` 64 (53 der Fassung 1.3 minus `INV-SHRED-1` plus neun neue aus 2.3 und drei aus 2.4 [`INV-SEAL-HARD... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-ASYNC-1` | `docs/SPEC.md` | 1410 | \|REQ-ASYNC-1\|Ratchet (Stufe B): `rg "async fn\\|BoxFuture"` in den Ring-0-Crates steigt nie; `cargo tree -p <ring0-... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-ASYNC-2` | `docs/SPEC.md` | 1411 | \|REQ-ASYNC-2\|Gate `async-io` hat 0 Verstöße und prüft Blockieren im async-Kontext zeilenweise (heute heuristisch).\| | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-ASYNC-3` | `docs/SPEC.md` | 1412 | \|REQ-ASYNC-3\|Jede `spawn`-Stelle hat eine Obergrenze (Backpressure); Gate `unbounded-spawn`.\| | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DUR-014` | `docs/SPEC.md` | 1496 | Regeln: (1) Übergänge sind monoton in Richtung `Poisoned`, außer durch Neustart bzw. dokumentierte Selbstheilung `Deg... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-PRF-5` | `docs/SPEC.md` | 1542 | \| Löschbericht (`DeletionProof`) \| `MFDR` \| V5 (explizites Layout, kein `serde`-Derive); V1–V3 nur lesbar; V4 blei... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-FMT-5` | `docs/SPEC.md` | 1543 | \| KeyTable \| `MFKT` \| prüfen \| Blake3 \| neu in 2.3; Ersetzen-Semantik, atomar (REQ-FMT-5), §12.2.9 \| | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-FMT-1` | `docs/SPEC.md` | 1554 | \| REQ-FMT-1 \| Jedes persistente Artefakt beginnt mit Magic (4 Byte), Formatversion (u16), Flags, Längenfeld und trä... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-FMT-2` | `docs/SPEC.md` | 1555 | \| REQ-FMT-2 \| Ein Leser verweigert **fail-closed** jede Version, die er nicht kennt; der Schreiber schreibt nur die... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-FMT-3` | `docs/SPEC.md` | 1556 | \| REQ-FMT-3 \| Für jede unterstützte Version liegt ein **Golden-File** im Repo; Round-Trip-Tests laufen in CI. \| | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-FMT-4` | `docs/SPEC.md` | 1557 | \| REQ-FMT-4 \| Serde-abgeleitete Layouts sind für signierte oder persistente Daten nur zulässig, wenn ein Golden-Fil... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-FMT-5` | `docs/SPEC.md` | 1558 | \| REQ-FMT-5 \| Atomares Ersetzen: temporäre Datei, `fsync`, `rename`, `fsync` des Verzeichnisses (Linux `fdatasync`,... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-FMT-6` | `docs/SPEC.md` | 1559 | \| REQ-FMT-6 \| Migration läuft offline über `contextra migrate` mit Trockenlauf und vorheriger Sicherung; keine stil... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-FMT-7` | `docs/SPEC.md` | 1560 | \| REQ-FMT-7 \| Vor 1.0 sind inkompatible Formatwechsel erlaubt, wenn `migrate` sie abdeckt; ab 1.0 sind On-Disk-Form... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-031` | `docs/SPEC.md` | 1576 | - **Zeitreisen:** `as_of(seq \| zeit)` liest historische Snapshots, solange nicht kompaktiert/gelöscht. **Löschung wi... | definiert | In Anforderungskatalog / SPEC geführt. |
| `O-5` | `docs/SPEC.md` | 1607 | - **ACL-Bitset (RLS):** pro Rolle ein Bitset berechtigter Vektor-IDs; **Prüfung vor Distanzberechnung**. Modus `stric... | definiert | Dokumentiert in §16. |
| `O-6` | `docs/SPEC.md` | 1634 | `Analyzer`-Port: Unicode-Normalisierung (NFKC), Kleinschreibung, Standard-Tokenisierung; Tokenizer ohne Panics. `Germ... | definiert | Dokumentiert in §16. |
| `REQ-IDX-015` | `docs/SPEC.md` | 1646 | **Modellidentität (REQ-IDX-015):** `embedding_model_hash` ist `BLAKE3` über die kanonische Kodierung von `(Gewichts-D... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-011` | `docs/SPEC.md` | 1673 | 5. **Lesesperre:** Treffer, deren Dokument im `ErasedSet` steht, werden entfernt, **bevor** irgendetwas zurückgegeben... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-IDX-009` | `docs/SPEC.md` | 1678 | **RRF** mit `k = 60`; deterministische Tie-Breaker: **Score absteigend, DocId aufsteigend** (REQ-IDX-009). Die Basis-... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-IDX-012` | `docs/SPEC.md` | 1686 | Jedes Dokument wird vor Indexierung über einen `MarkdownChunker` zerlegt: (1) Zielgröße 512 und Obergrenze 1024 Token... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-TEN-005` | `docs/SPEC.md` | 1690 | `max_document_bytes` 16 MiB, `max_chunks_per_document` 4.096, `max_batch_ops` 10.000, `max_collections_per_tenant` 25... | definiert | In Anforderungskatalog / SPEC geführt. |
| `§25.1` | `docs/SPEC.md` | 1704 | > **Rechtshinweis.** Technische Spezifikation, keine Rechtsberatung. **Software kann eine Rechtswirkung weder herbeif... | nicht definiert | Verweis auf §16.1 (Prüfpunkte-Register) oder passenden Unterabschnitt anpassen. |
| `P-01` | `docs/SPEC.md` | 1712 | 3. Es ist **keine höchstrichterliche Entscheidung bekannt**, die Schlüsselvernichtung als Löschung anerkennt (P-01, P... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-01). |
| `P-02` | `docs/SPEC.md` | 1712 | 3. Es ist **keine höchstrichterliche Entscheidung bekannt**, die Schlüsselvernichtung als Löschung anerkennt (P-01, P... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-02). |
| `P-14` | `docs/SPEC.md` | 1718 | Die Spalte „Prüfung 2.3“ trennt, was bei der Quellenprüfung vor 2.3 geprüft wurde, von dem, was bei der Erstellung di... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-14). |
| `P-14` | `docs/SPEC.md` | 1723 | \|EDPB Guidelines 02/2026 Anonymisierung, Entwurf v1.0 (07.07.2026), Konsultation bis 30.10.2026\|Kriterien No Record... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-14). |
| `P-14` | `docs/SPEC.md` | 1729 | \|BayLfD 30. TB (2020) Nr. 12.5\|Zeitversetzte Backup-Löschung zulässig bei technischer Unmöglichkeit, begründeter Lö... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-14). |
| `P-06` | `docs/SPEC.md` | 1748 | \|„Aufsichten in Bayern, Baden-Württemberg und Berlin akzeptieren Backup-Rotation von 30, 60 oder 90 Tagen“\|**Keine ... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-06). |
| `P-05` | `docs/SPEC.md` | 1750 | \|„EDPB 28/2024 stellt fest, dass Embeddings personenbezogen sind“\|**Zu weit.** Die Stellungnahme prüft die Anonymit... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-05). |
| `P-09` | `docs/SPEC.md` | 1753 | \|„Buchungsbelege 10 Jahre, § 203 StGB als Aufbewahrungsnorm“\|**Veraltet bzw. falsch.** Buchungsbelege 8 Jahre seit ... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-09). |
| `P-22` | `docs/SPEC.md` | 1753 | \|„Buchungsbelege 10 Jahre, § 203 StGB als Aufbewahrungsnorm“\|**Veraltet bzw. falsch.** Buchungsbelege 8 Jahre seit ... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-22). |
| `§ 257` | `docs/SPEC.md` | 1753 | \|„Buchungsbelege 10 Jahre, § 203 StGB als Aufbewahrungsnorm“\|**Veraltet bzw. falsch.** Buchungsbelege 8 Jahre seit ... | nicht definiert | Verweis auf §16.1 (Prüfpunkte-Register) oder passenden Unterabschnitt anpassen. |
| `P-01` | `docs/SPEC.md` | 1773 | \|T2 Unkenntlichmachung\|niemand ist mit vertretbarem Aufwand zur Identifizierung in der Lage (SDM, DSB, OLG Dresden ... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-01). |
| `P-02` | `docs/SPEC.md` | 1773 | \|T2 Unkenntlichmachung\|niemand ist mit vertretbarem Aufwand zur Identifizierung in der Lage (SDM, DSB, OLG Dresden ... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-02). |
| `O-12` | `docs/SPEC.md` | 1775 | \|T4 Einschränkung/Aufbewahrung\|Art. 18, Art. 17 Abs. 3 (CEF Issue 4)\|Klasse **L**, `Restricted`, Legal Hold\|Dauer... | Kollision | In C-12.1.3 (T4) auf O-41 umstellen (O-12 nur für Baseline-Reproduzierbarkeit). |
| `REQ-DEL-028` | `docs/SPEC.md` | 1808 | \|LA-10\|Krypto-Agilität: Austauschplan, periodische Neubewertung.\|[B ✔ Analogie]\|§12.2.9, REQ-DEL-028\| | definiert | In Anforderungskatalog / SPEC geführt. |
| `§20` | `docs/SPEC.md` | 1848 | Bis zur bestandenen Konformitätssuite liefern K2 und K3 `CapabilityUnsupported`, **nie** einen Stub, der Erfolg melde... | nicht definiert | Abschnitt existiert in 3.1 nicht mehr als Hauptkapitel; Verweis prüfen. |
| `REQ-DEL-027` | `docs/SPEC.md` | 1848 | Bis zur bestandenen Konformitätssuite liefern K2 und K3 `CapabilityUnsupported`, **nie** einen Stub, der Erfolg melde... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-PORT-2` | `docs/SPEC.md` | 1860 | `DestroyAttestation` enthält Custody, Verfahren (Zeroization), Zeitpunkt (aus `Clock`) und einen Beleg der Implementi... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-FMT-5` | `docs/SPEC.md` | 1870 | 3. schreibt die KeyTable atomar neu (REQ-FMT-5); | definiert | In Anforderungskatalog / SPEC geführt. |
| `H-07` | `docs/SPEC.md` | 1879 | **Auslöser.** Zeitgesteuert (`kek_epoch_len`, Startwert 30 Tage), auf Anforderung (`seal-now`) und automatisch nach e... | definiert | Härtungshinweis in C-12. |
| `O-24` | `docs/SPEC.md` | 1885 | **Kosten.** Re-Wrap ist O(Schlüssel der geänderten Shards + `n_shards`), nicht O(Datenmenge) und nicht O(alle Schlüss... | nicht definiert | Als offener Punkt / Kalibrierungsparameter dokumentiert, Definition in §16 ergänzen. |
| `O-40` | `docs/SPEC.md` | 1885 | **Kosten.** Re-Wrap ist O(Schlüssel der geänderten Shards + `n_shards`), nicht O(Datenmenge) und nicht O(alle Schlüss... | nicht definiert | Als offener Punkt / Kalibrierungsparameter dokumentiert, Definition in §16 ergänzen. |
| `REQ-DEL-N5` | `docs/SPEC.md` | 1892 | 4. Der gewrappte DocDek liegt in der **KeyTable** (Ersetzen-Semantik), nicht in einem Append-Log. Das `KeyRegistry`-S... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-012` | `docs/SPEC.md` | 1893 | 5. Der Key-Lock der Einheit wird über die **gesamte** Saga gehalten; ein konkurrierendes `remember` auf die Einheit w... | definiert | In Anforderungskatalog / SPEC geführt. |
| `H-04` | `docs/SPEC.md` | 1894 | 6. **Entschlüsselte Schlüssel im Speicher** liegen ausschließlich in einem registrierten `KeyCache`, der bei Vernicht... | definiert | Härtungshinweis in C-12. |
| `REQ-DEL-N37` | `docs/SPEC.md` | 1895 | 7. Bestand ohne DocDek (Altformat, Scope-DEK) gilt als `LegacyUnkeyed`: höchstens Klasse P, bis `contextra migrate --... | definiert | In Anforderungskatalog / SPEC geführt. |
| `H-12` | `docs/SPEC.md` | 1895 | 7. Bestand ohne DocDek (Altformat, Scope-DEK) gilt als `LegacyUnkeyed`: höchstens Klasse P, bis `contextra migrate --... | definiert | Härtungshinweis in C-12. |
| `REQ-DEL-N15` | `docs/SPEC.md` | 1901 | - **Header:** `doc_key_id` und Sequenzbezug stehen **innerhalb** des unter `K_eng` verschlüsselten Blockbereichs, nic... | definiert | In Anforderungskatalog / SPEC geführt. |
| `H-03` | `docs/SPEC.md` | 1904 | ##### C-12.2.7 Vernichtungsprotokoll (Destroy-Protokoll, H-03) | definiert | Härtungshinweis in C-12. |
| `REQ-FMT-5` | `docs/SPEC.md` | 1912 | \|D3\|Neue KeyTable-Generation **ohne** den Eintrag schreiben: temporäre Datei, `fsync`, `rename`, `fsync` des Verzei... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-N16` | `docs/SPEC.md` | 1929 | INV-ARTKEY-1: Jedes Index-Artefakt hat genau einen ArtKey; der alte wird bei Merge sofort, bei Löschung nach Siegel 2... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-N4` | `docs/SPEC.md` | 1933 | - **KeyTable** (Magic `MFKT`, §7.2) ist ein eigener Speicherbereich mit Ersetzen-Semantik; in 2.4 in `n_shards` Datei... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-FMT-5` | `docs/SPEC.md` | 1933 | - **KeyTable** (Magic `MFKT`, §7.2) ist ein eigener Speicherbereich mit Ersetzen-Semantik; in 2.4 in `n_shards` Datei... | definiert | In Anforderungskatalog / SPEC geführt. |
| `P-13` | `docs/SPEC.md` | 1935 | - **Krypto-Agilität (REQ-DEL-028, P-13).** Jedes persistierte kryptografische Objekt trägt Algorithmus- und Formatken... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-13). |
| `REQ-DEL-028` | `docs/SPEC.md` | 1935 | - **Krypto-Agilität (REQ-DEL-028, P-13).** Jedes persistierte kryptografische Objekt trägt Algorithmus- und Formatken... | definiert | In Anforderungskatalog / SPEC geführt. |
| `O-28` | `docs/SPEC.md` | 1940 | Jeder Schlüssel dient genau einem Zweck; HKDF-Label `contextra/v1/<zweck>`. Das Register `governance/key_purposes.tom... | nicht definiert | Als offener Punkt / Kalibrierungsparameter dokumentiert, Definition in §16 ergänzen. |
| `REQ-KEY-011` | `docs/SPEC.md` | 1940 | Jeder Schlüssel dient genau einem Zweck; HKDF-Label `contextra/v1/<zweck>`. Das Register `governance/key_purposes.tom... | definiert | In Anforderungskatalog / SPEC geführt. |
| `H-16` | `docs/SPEC.md` | 1955 | Regeln: (1) Labels sind paarweise verschieden; Wiederverwendung eines Schlüssels für zwei Zwecke ist unzulässig. (2) ... | definiert | Härtungshinweis in C-12. |
| `REQ-DEL-N7` | `docs/SPEC.md` | 1967 | \|S4 Caches und Vault\|`CachesAndVault`\|A\|`store`, `engine`, `privacy`\|Block-, Embedding-, RRF-, Antwort-Cache und... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-010` | `docs/SPEC.md` | 1978 | 2. Eine Schicht, in der Daten liegen können und für die kein Scanner existiert, ist `NotMeasured`; der Bericht ist da... | definiert | In Anforderungskatalog / SPEC geführt. |
| `P-05` | `docs/SPEC.md` | 1980 | 4. **Embeddings und Vektorbytes sind personenbezogen** (vorsorglich, P-05): „Text gelöscht, Vektor im Index“ ist kein... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-05). |
| `§21` | `docs/SPEC.md` | 1984 | 8. **Migration der Typen.** Der Code kennt sieben abweichende Varianten (`LsmMemtable`, `SsTableAllLevels`, `HnswInde... | nicht definiert | Abschnitt existiert in 3.1 nicht mehr als Hauptkapitel; Verweis prüfen. |
| `P-18` | `docs/SPEC.md` | 1999 | \|**C3**\|wie C1, externer KMS\|A-FLASH ersetzt durch Anbieterattest (Prüfpflicht des Verantwortlichen, P-18)\|wie C1... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-18). |
| `P-03` | `docs/SPEC.md` | 2011 | \|A-RAM\|Kein Klartext oder Schlüssel liegt in Swap, Dumps, Hibernation (P-03; §12.18)\| | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-03). |
| `H-09` | `docs/SPEC.md` | 2014 | \|A-CLOCK\|Die Systemuhr ist nicht manipuliert (Horizont- und Fristentscheidungen, H-09)\| | definiert | Härtungshinweis in C-12. |
| `O-15` | `docs/SPEC.md` | 2022 | `checkpoint_policy` je Collection: `Rewrite` (**Standard**, ohne den Datensatz neu schreiben, Manifest und Prüfsumme ... | definiert | Dokumentiert in §16. |
| `REQ-DEL-022` | `docs/SPEC.md` | 2030 | **AuditChain:** append-only, hash-verkettet, signierte Checkpoints, tenant-getrennt; enthält Zugriffe, Löschungen, Be... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-008` | `docs/SPEC.md` | 2038 | 3. Eine **Konstante oder ein vom Aufrufer gelieferter Zählerwert** DARF NIE als Messung dienen (REQ-DEL-008). **Es gi... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-009` | `docs/SPEC.md` | 2038 | 3. Eine **Konstante oder ein vom Aufrufer gelieferter Zählerwert** DARF NIE als Messung dienen (REQ-DEL-008). **Es gi... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-010` | `docs/SPEC.md` | 2039 | 4. Fehlt einer Schicht die Fähigkeit zu messen, ist sie `NotMeasured`; fehlende Fähigkeit ist `CapabilityUnsupported`... | definiert | In Anforderungskatalog / SPEC geführt. |
| `AC-1` | `docs/SPEC.md` | 2041 | 6. **Positivkontrolle.** Jeder Scan-Lauf weist an einem absichtlich nicht gelöschten Kontroll-Datensatz nach, dass er... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-1` | `docs/SPEC.md` | 2048 | 3. Positivkontrolle (AC-1) in beiden Sichten. | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-1` | `docs/SPEC.md` | 2070 | control_matches: u64,         // positive Kontrolle (AC-1); > 0 | definiert | In Abnahmekriterien §12.12 geführt. |
| `H-02` | `docs/SPEC.md` | 2086 | **Mint-Härtung (H-02, INV-WITNESS-MINT-1).** Der Zustand im Code ist, dass `LayerWitness::issue_from_scanner` `pub` i... | definiert | Härtungshinweis in C-12. |
| `REQ-FMT-1` | `docs/SPEC.md` | 2090 | **Format.** V5 ersetzt den in 2.2.1 eingefrorenen V4 (E26, ersetzt durch E46). **Der Repo-Stand kennt höchstens `Sign... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-PRF-1` | `docs/SPEC.md` | 2111 | \|REQ-PRF-1\|Der Stub `no_crypto_stubs` (`engine/src/lib.rs:17`) und die leeren Mandanten-Hooks werden **gelöscht**; ... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-PRF-2` | `docs/SPEC.md` | 2112 | \|REQ-PRF-2\|`issue` liefert `Err`, wenn eine Pflichtschicht `remaining > 0` hat oder weder `Structural` noch `Featur... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-PRF-3` | `docs/SPEC.md` | 2113 | \|REQ-PRF-3\|Der Bericht nennt je Schicht Methode, Zahlen, Digest sowie `excluded`; die Außenformulierung wird aus de... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-PRF-4` | `docs/SPEC.md` | 2114 | \|REQ-PRF-4\|`rg "default_test_deletion_proof_key"` im Produktionscode ergibt **0 Treffer** (heute `contextra-mcp/src... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-PRF-5` | `docs/SPEC.md` | 2115 | \|REQ-PRF-5\|V5 mit Golden-File, Migrationspfad und Fuzz-Ziel (ersetzt „entfällt, E26“).\| | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-PRF-6` | `docs/SPEC.md` | 2116 | \|REQ-PRF-6\|`doctor verify-report` prüft offline Signatur, Prüfsumme, Klasse, Siegelkette und Zeitanker (ersetzt „en... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-N9` | `docs/SPEC.md` | 2116 | \|REQ-PRF-6\|`doctor verify-report` prüft offline Signatur, Prüfsumme, Klasse, Siegelkette und Zeitanker (ersetzt „en... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-PRF-7` | `docs/SPEC.md` | 2117 | \|REQ-PRF-7\|Jeder Bericht (alle vier Siegel) wird als Eintrag in die AuditChain geschrieben (Magic, Version, Prüfsum... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-N20` | `docs/SPEC.md` | 2132 | Die Klasse im Bericht gilt für beide Zustände; `Validated` ist ein zusätzliches Feld. Ein Bericht ohne `Validated` im... | definiert | In Anforderungskatalog / SPEC geführt. |
| `O-34` | `docs/SPEC.md` | 2140 | Der Digest des Berichts wird optional über einen qualifizierten Zeitstempel (RFC 3161, eIDAS Art. 41, 42) verankert. ... | nicht definiert | Als offener Punkt / Kalibrierungsparameter dokumentiert, Definition in §16 ergänzen. |
| `P-07` | `docs/SPEC.md` | 2144 | Der Bericht enthält nur Referenz-Hashes und gilt selbst als personenbezogen (EDPB Rn. 52). Aufbewahrung nach `report_... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-07). |
| `H-05` | `docs/SPEC.md` | 2193 | **Ordnungsinvariante (INV-DEL-ORDER-1, H-05).** Schritt 5 DARF NICHT beginnen, bevor Siegel 2 (Klasse P) für **jede E... | definiert | Härtungshinweis in C-12. |
| `REQ-DEL-N26` | `docs/SPEC.md` | 2197 | Bis Schritt 5 ist die Löschung rücknehmbar (`Unfreeze`, Kategorie `Admin`, Begründung im Audit; **im Regulated-Tier z... | definiert | In Anforderungskatalog / SPEC geführt. |
| `H-10` | `docs/SPEC.md` | 2197 | Bis Schritt 5 ist die Löschung rücknehmbar (`Unfreeze`, Kategorie `Admin`, Begründung im Audit; **im Regulated-Tier z... | definiert | Härtungshinweis in C-12. |
| `H-06` | `docs/SPEC.md` | 2205 | - **Keine Wiederauferstehung (INV-NO-RESURRECTION-1, H-06).** Beim Start (B6/B7) und nach jedem Restore prüft die Eng... | definiert | Härtungshinweis in C-12. |
| `H-09` | `docs/SPEC.md` | 2206 | - **Uhr (H-09, A-CLOCK).** Entscheidungen, die von Zeit abhängen (Horizont, Fristen, Karenz, `valid_until`), schlagen... | definiert | Härtungshinweis in C-12. |
| `REQ-DEL-N27` | `docs/SPEC.md` | 2217 | Jedes Siegel ist ein signierter Bericht, in die AuditChain eingetragen und mit dem vorherigen verkettet (REQ-DEL-N27)... | definiert | In Anforderungskatalog / SPEC geführt. |
| `P-09` | `docs/SPEC.md` | 2221 | Bei Aufbewahrungspflicht (Art. 17 Abs. 3) oder Legal Hold geht die Saga nach Schritt 1 in `Restricted{basis, until}`:... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-09). |
| `O-12` | `docs/SPEC.md` | 2221 | Bei Aufbewahrungspflicht (Art. 17 Abs. 3) oder Legal Hold geht die Saga nach Schritt 1 in `Restricted{basis, until}`:... | Kollision | In C-12.1.3 (T4) auf O-41 umstellen (O-12 nur für Baseline-Reproduzierbarkeit). |
| `AC-11` | `docs/SPEC.md` | 2221 | Bei Aufbewahrungspflicht (Art. 17 Abs. 3) oder Legal Hold geht die Saga nach Schritt 1 in `Restricted{basis, until}`:... | definiert | In Abnahmekriterien §12.12 geführt. |
| `REQ-FGT-1` | `docs/SPEC.md` | 2227 | \|REQ-FGT-1\|Jeder Übergang wird **vor** der Ausführung ins durable Journal geschrieben (`fsync` mit Fehlerpropagatio... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-FGT-2` | `docs/SPEC.md` | 2228 | \|REQ-FGT-2\|Beim Start (B7) setzt die Engine jede Saga ohne `Completed` fort oder meldet `Partial{outstanding}`.\| | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-FGT-3` | `docs/SPEC.md` | 2229 | \|REQ-FGT-3\|`Frozen` schreibt den `ErasedSet`-Eintrag durabel **vor** der ersten Löschoperation; ab dann gilt die Le... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-FGT-4` | `docs/SPEC.md` | 2230 | \|REQ-FGT-4\|Fehler von `fsync`, `rename`, `write` in Purgern führen zu `Partial`, nie zu stillem Weiterlaufen.\| | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-FGT-5` | `docs/SPEC.md` | 2231 | \|REQ-FGT-5\|Ein Scanner-Ergebnis mit `remaining > 0` hält die Saga in `PlaintextMeasured`/`HygieneMeasured` mit `Res... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-FGT-6` | `docs/SPEC.md` | 2232 | \|REQ-FGT-6\|Die Saga ist abbrechbar nur **zwischen** Schritten; Abbruch hinterlässt `Partial`.\| | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-FGT-7` | `docs/SPEC.md` | 2233 | \|REQ-FGT-7\|Der `RequestContext` (Mandant, Prinzipal) kommt vom Aufrufer; kein Fallback auf `TenantId::SYSTEM` und k... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-036` | `docs/SPEC.md` | 2234 | \|REQ-DEL-036\|**Fristverfolgung:** Die Saga trägt `requested_at` und Schwellen (§12.19.1); jeder `Partial`-Zustand e... | definiert | In Anforderungskatalog / SPEC geführt. |
| `AC-35` | `docs/SPEC.md` | 2248 | **c) Zweistufige Messung (INV-UNITDIR-1, AC-35).** Der Rohbyte-Vollscan aller Dateien je Löschung wäre O(Datenmenge) ... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-35` | `docs/SPEC.md` | 2256 | Differenzbedingung: Der zweistufige Befund und der volle Rohscan liefern auf denselben Daten dasselbe Urteil; eine Ab... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-37` | `docs/SPEC.md` | 2258 | **d) Koaleszenz.** Löschungen innerhalb von `measure_coalesce_window` (Startwert 60 s) teilen sich einen Messlauf; in... | definiert | In Abnahmekriterien §12.12 geführt. |
| `REQ-DEL-011` | `docs/SPEC.md` | 2276 | - **Lesesperre.** Mit `Frozen` trägt der persistente `ErasedSet` nur Referenz-Hashes. `get`, `search`, `hydrate`, `ex... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-012` | `docs/SPEC.md` | 2277 | - **Key-Lock.** Das Löschen hält den Key-Lock der Einheit über die **gesamte** Saga (REQ-DEL-012); konkurrierendes Sc... | definiert | In Anforderungskatalog / SPEC geführt. |
| `H-08` | `docs/SPEC.md` | 2278 | - **`ErasedSet`-Eintrag (H-08).** `{entry_seq, scope_ref, subject_ref_hash?, unit_ref_hash[], ref_epoch, requested_at... | definiert | Härtungshinweis in C-12. |
| `REQ-RST-1` | `docs/SPEC.md` | 2285 | \|REQ-RST-1\|`ErasedSet` ist durabel, enthält nur Referenz-Hashes (keine Klartext-IDs) und wird in B6 vor Verkehr gel... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-RST-2` | `docs/SPEC.md` | 2286 | \|REQ-RST-2\|`restore_checkpoint` sperrt vor dem ersten Lesezugriff, wendet jeden Eintrag erneut an und schreibt `res... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-013` | `docs/SPEC.md` | 2286 | \|REQ-RST-2\|`restore_checkpoint` sperrt vor dem ersten Lesezugriff, wendet jeden Eintrag erneut an und schreibt `res... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-RST-3` | `docs/SPEC.md` | 2287 | \|REQ-RST-3\|Schlüssel-Backups, deren signiertes `valid_until` überschritten oder deren Alter nicht bestimmbar ist, w... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-RST-4` | `docs/SPEC.md` | 2288 | \|REQ-RST-4\|Checkpoint-Erzeugung und Export prüfen den `ErasedSet` vor dem Schreiben.\| | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-RST-5` | `docs/SPEC.md` | 2289 | \|REQ-RST-5\|Backups außerhalb von Contextra stehen als `ExternalBackups`/`ExternalUnmanaged` im Bericht; die Rücksic... | definiert | In Anforderungskatalog / SPEC geführt. |
| `O-36` | `docs/SPEC.md` | 2293 | **Zweck.** Berichte, `ErasedSet`, `KeyRegistry` und Journal müssen auf Betroffene verweisen, ohne sie zu identifizier... | nicht definiert | Als offener Punkt / Kalibrierungsparameter dokumentiert, Definition in §16 ergänzen. |
| `H-13` | `docs/SPEC.md` | 2293 | **Zweck.** Berichte, `ErasedSet`, `KeyRegistry` und Journal müssen auf Betroffene verweisen, ohne sie zu identifizier... | definiert | Härtungshinweis in C-12. |
| `P-06` | `docs/SPEC.md` | 2302 | 6. `erased_set_retention ≥ key_backup_max_age + Sicherheitszuschlag` (Startwert insgesamt 90 Tage; Obergrenze nach Re... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-06). |
| `P-07` | `docs/SPEC.md` | 2302 | 6. `erased_set_retention ≥ key_backup_max_age + Sicherheitszuschlag` (Startwert insgesamt 90 Tage; Obergrenze nach Re... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-07). |
| `P-16` | `docs/SPEC.md` | 2302 | 6. `erased_set_retention ≥ key_backup_max_age + Sicherheitszuschlag` (Startwert insgesamt 90 Tage; Obergrenze nach Re... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-16). |
| `O-17` | `docs/SPEC.md` | 2304 | Kipptest: Will der Eigentümer auch die Referenz-Epochen streichen, werden `ErasedSet`-Einträge nach `erased_set_reten... | nicht definiert | Als offener Punkt / Kalibrierungsparameter dokumentiert, Definition in §16 ergänzen. |
| `REQ-DEL-035` | `docs/SPEC.md` | 2324 | Das Gate `deletion-honesty` vergleicht jeden **statischen** Außentext (README, MCP-Werkzeugbeschreibungen, `doctor`-T... | definiert | In Anforderungskatalog / SPEC geführt. |
| `AC-19` | `docs/SPEC.md` | 2324 | Das Gate `deletion-honesty` vergleicht jeden **statischen** Außentext (README, MCP-Werkzeugbeschreibungen, `doctor`-T... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-24` | `docs/SPEC.md` | 2324 | Das Gate `deletion-honesty` vergleicht jeden **statischen** Außentext (README, MCP-Werkzeugbeschreibungen, `doctor`-T... | definiert | In Abnahmekriterien §12.12 geführt. |
| `§25.3` | `docs/SPEC.md` | 2328 | Eine Löschung gilt erst als bewiesen, wenn alle gültigen Kriterien grün sind. AC-7, AC-12, AC-14, AC-15 bleiben **unb... | nicht definiert | Verweis auf §16.1 (Prüfpunkte-Register) oder passenden Unterabschnitt anpassen. |
| `AC-7` | `docs/SPEC.md` | 2328 | Eine Löschung gilt erst als bewiesen, wenn alle gültigen Kriterien grün sind. AC-7, AC-12, AC-14, AC-15 bleiben **unb... | nicht definiert (unbelegt) | In §12.12 als historisch unbelegt geführt, Nummer reserviert. |
| `AC-12` | `docs/SPEC.md` | 2328 | Eine Löschung gilt erst als bewiesen, wenn alle gültigen Kriterien grün sind. AC-7, AC-12, AC-14, AC-15 bleiben **unb... | nicht definiert (unbelegt) | In §12.12 als historisch unbelegt geführt, Nummer reserviert. |
| `AC-14` | `docs/SPEC.md` | 2328 | Eine Löschung gilt erst als bewiesen, wenn alle gültigen Kriterien grün sind. AC-7, AC-12, AC-14, AC-15 bleiben **unb... | nicht definiert (unbelegt) | In §12.12 als historisch unbelegt geführt, Nummer reserviert. |
| `AC-15` | `docs/SPEC.md` | 2328 | Eine Löschung gilt erst als bewiesen, wenn alle gültigen Kriterien grün sind. AC-7, AC-12, AC-14, AC-15 bleiben **unb... | nicht definiert (unbelegt) | In §12.12 als historisch unbelegt geführt, Nummer reserviert. |
| `AC-21` | `docs/SPEC.md` | 2328 | Eine Löschung gilt erst als bewiesen, wenn alle gültigen Kriterien grün sind. AC-7, AC-12, AC-14, AC-15 bleiben **unb... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-1` | `docs/SPEC.md` | 2332 | \|AC-1\|Nach Löschung und Bericht enthält **keine** Datei des Datenverzeichnisses (WAL, SSTables, Blob-Segmente, Inde... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-2` | `docs/SPEC.md` | 2333 | \|AC-2\|Wird eine Messung durch eine Konstante ersetzt, schlägt mindestens ein Test fehl.\|Mutation\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-3` | `docs/SPEC.md` | 2334 | \|AC-3\|Absturz an jedem Journalzustand und an jedem Schritt D1 bis D6: danach `Completed` mit vollständigen Messunge... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-4` | `docs/SPEC.md` | 2335 | \|AC-4\|Restore eines Checkpoints vor der Löschung: Betroffener gesperrt, danach erneut gelöscht, `restore_reapplied`... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-5` | `docs/SPEC.md` | 2336 | \|AC-5\|Zwischen `Frozen` und `Completed` liefert kein Lesepfad Daten des Betroffenen, auch parallel.\|loom/Property\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-6` | `docs/SPEC.md` | 2337 | \|AC-6\|Berichts-Negativsuite: Bit-Flip je Feld, Feldvertauschung, Versions-Downgrade, anderer Mandant, falscher Schl... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-8` | `docs/SPEC.md` | 2338 | \|AC-8\|Neustart nach Löschung: gesperrt/gelöscht, Journal konsistent.\|Integration\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-9` | `docs/SPEC.md` | 2339 | \|AC-9\|Mit `ManualClock` und festen Eingaben ist der Bericht bytegleich reproduzierbar.\|Golden-Test\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-10` | `docs/SPEC.md` | 2340 | \|AC-10\|Das Gate `deletion-honesty` erkennt konstante Messwerte, verworfene Prüfergebnisse, unbedingte `Ok(true)`-Ve... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-11` | `docs/SPEC.md` | 2341 | \|AC-11\|Legal Hold ⇒ `Restricted`, keine Löschoperation, **keine Schlüsselvernichtung**, Lesesperre aktiv, `basis` u... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-13` | `docs/SPEC.md` | 2342 | \|AC-13\|Gleichzeitiges `put`/`delete` derselben ID ergibt nach Abschluss nie einen sichtbaren Zustand.\|loom\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-16` | `docs/SPEC.md` | 2343 | \|AC-16\|Checkpoint vor der Löschung: Bericht höchstens Klasse C ohne M mit `RetainedUnderRestoreGuard{count, oldest_... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-17` | `docs/SPEC.md` | 2344 | \|AC-17\|`restore_checkpoint` weist ein Schlüssel-Backup, das `valid_until` überschritten hat oder dessen Alter unbes... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-18` | `docs/SPEC.md` | 2345 | \|AC-18\|Nach Vernichtung von `K_ref[tenant, epoche]` ist kein Eintrag dieser Epoche in `ErasedSet`, `AuditChain` und... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-19` | `docs/SPEC.md` | 2346 | \|AC-19\|Kein statischer Außentext enthält einen Claim, den der Bericht der Standardkonfiguration (K1) nicht ausstell... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-20` | `docs/SPEC.md` | 2347 | \|AC-20\|Mit `ManualClock`: ein `Partial`-Zustand erzeugt nach den drei Schwellen je genau einen Alarm, eine Eskalati... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-21` | `docs/SPEC.md` | 2348 | \|AC-21\|**Epoch Seal.** Nach dem Seal: Re-Wrap aller lebenden Schlüssel, `KEK[e]` vernichtet, keine Datei des Datenv... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-22` | `docs/SPEC.md` | 2349 | \|AC-22\|**Backup-Trennung.** Kein Daten-Backup enthält Schlüsselmaterial (Format- und Literalscan). Backup vor Lösch... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-23` | `docs/SPEC.md` | 2350 | \|AC-23\|**Mixed-Segmente.** `BlockAll`, `RedactAndRekey`, `DeleteAll` verhalten sich nach §12.15; kein Segment wird ... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-24` | `docs/SPEC.md` | 2351 | \|AC-24\|**Claim-Generator.** Jeder Satz der Satzbibliothek ist an einen Messwert gebunden; ein absichtlich überzogen... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-25` | `docs/SPEC.md` | 2352 | \|AC-25\|**Nadelscan.** Eine absichtlich zurückgelassene Wrap-Nonce (Test-Mutation) wird gefunden; ohne Nonce 0 Treff... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-26` | `docs/SPEC.md` | 2353 | \|AC-26\|**Kein Dummy-Bericht.** Kein Codepfad erzeugt einen `DeletionProof` ohne Scanner-Witness; `rg` nach `default... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-27` | `docs/SPEC.md` | 2354 | \|AC-27\|**Keine Wiederauferstehung.** Nach jedem Crash und jedem Restore steht kein lebender Schlüssel einer `Erased... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-28` | `docs/SPEC.md` | 2355 | \|AC-28\|**Destroy-Durabilität.** Stromausfall an jedem Punkt von D1 bis D6 (`FaultVfs`): Schlüssel lebt und Einheit ... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-29` | `docs/SPEC.md` | 2356 | \|AC-29\|**Mirror.** Restore nach Totalverlust: mit aktuellem `ErasedSetMirror` bleibt der Scope gesperrt; ohne aktue... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-30` | `docs/SPEC.md` | 2357 | \|AC-30\|**Padding und Header.** Kein Record trägt `doc_key_id` im Klartext; die Längenverteilung bleibt nach Padmé u... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-31` | `docs/SPEC.md` | 2358 | \|AC-31\|**Rücknahme.** `Unfreeze` nach D3 wird abgewiesen; im Regulated-Tier braucht `Unfreeze` zwei Personen.\|Inte... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-32` | `docs/SPEC.md` | 2359 | \|AC-32\|**Uhr.** Bei Uhrrücksprung verweigern Horizont-, Karenz- und `valid_until`-Entscheidungen fail-closed (`Cloc... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-33` | `docs/SPEC.md` | 2360 | \|AC-33\|**Verification/Validation.** Im Regulated-Tier ist ein Bericht ohne Concurrence-Signatur `ValidationPending`... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-34` | `docs/SPEC.md` | 2361 | \|AC-34\|**Heap-Canary.** Nach Klasse P findet der Prozessspeicher-Scan keinen Canary, kein Embedding-Byte-Muster und... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-35` | `docs/SPEC.md` | 2362 | \|AC-35\|**Zweistufige Messung.** Auf 1.000 seed-reproduzierbaren Szenarien (mit Fehlerinjektion) liefern M1 bis M3 u... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-36` | `docs/SPEC.md` | 2363 | \|AC-36\|**Shard-Seal.** Nach dem Seal enthält keine Datei Wrap-Nonce oder -Blob eines vernichteten Schlüssels; nur g... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-37` | `docs/SPEC.md` | 2364 | \|AC-37\|**Koaleszenz.** n Löschungen in einem Fenster erzeugen n gültige Berichte mit einem Messlauf; keine Einheit ... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-38` | `docs/SPEC.md` | 2365 | \|AC-38\|**Flash-Härtung.** `flash_hardening` im Bericht folgt dem tatsächlichen Ablauf: K1-Seal ohne Passphrasenwech... | definiert | In Abnahmekriterien §12.12 geführt. |
| `REQ-DEL-027` | `docs/SPEC.md` | 2382 | Löschtypen existieren genau einmal (Kantenregel 5, Gate `deletion-types-once`). Ohne Löschpfad-Orchestrator liefert d... | definiert | In Anforderungskatalog / SPEC geführt. |
| `O-24` | `docs/SPEC.md` | 2396 | \|Padding und Re-Wrap kosten Platz und Zeit\|Padmé ≤ 12 %, Re-Wrap O(Schlüssel); Budgets in `budgets.toml` (O-24, O-4... | nicht definiert | Als offener Punkt / Kalibrierungsparameter dokumentiert, Definition in §16 ergänzen. |
| `O-40` | `docs/SPEC.md` | 2396 | \|Padding und Re-Wrap kosten Platz und Zeit\|Padmé ≤ 12 %, Re-Wrap O(Schlüssel); Budgets in `budgets.toml` (O-24, O-4... | nicht definiert | Als offener Punkt / Kalibrierungsparameter dokumentiert, Definition in §16 ergänzen. |
| `§25.1` | `docs/SPEC.md` | 2399 | \|Rechtswertung offen\|kein Produkttext ohne fachkundige Freigabe (§25.1)\| | nicht definiert | Verweis auf §16.1 (Prüfpunkte-Register) oder passenden Unterabschnitt anpassen. |
| `REQ-DEL-037` | `docs/SPEC.md` | 2401 | **Löschinventar (`doctor`, REQ-DEL-037).** Die Software erzwingt keine Löschfristen, macht ihr Fehlen aber sichtbar. ... | definiert | In Anforderungskatalog / SPEC geführt. |
| `O-35` | `docs/SPEC.md` | 2452 | Nach Totalverlust fehlt der `ErasedSet` des Zielsystems. Der `ErasedSetMirror` (Port, Magic der Einträge `MFEM`) schr... | nicht definiert | Als offener Punkt / Kalibrierungsparameter dokumentiert, Definition in §16 ergänzen. |
| `H-11` | `docs/SPEC.md` | 2452 | Nach Totalverlust fehlt der `ErasedSet` des Zielsystems. Der `ErasedSetMirror` (Port, Magic der Einträge `MFEM`) schr... | definiert | Härtungshinweis in C-12. |
| `P-02` | `docs/SPEC.md` | 2456 | CEF Issue 6 und BayLfD 12.5 verlangen Nachverfolgung und Nachlöschung beim Restore sowie Begründung, wenn Backups nic... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-02). |
| `O-38` | `docs/SPEC.md` | 2470 | Die plattformspezifischen Aufrufe sind in Spike S-HRD zu verifizieren (O-38). Bis dahin gilt: Fehlt ein Mittel, melde... | nicht definiert | Als offener Punkt / Kalibrierungsparameter dokumentiert, Definition in §16 ergänzen. |
| `P-11` | `docs/SPEC.md` | 2483 | \|Hygiene (Klasse M)\|intern, P-11\|≤ `purge_rebuild_max_days` (14 Tage)\| | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-11). |
| `P-11` | `docs/SPEC.md` | 2488 | Die Werte sind Zielgrößen. **Contextra sendet nichts nach außen**; Alarme erscheinen in `doctor`, Metrik, CLI, MCP-An... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-11). |
| `P-19` | `docs/SPEC.md` | 2488 | Die Werte sind Zielgrößen. **Contextra sendet nichts nach außen**; Alarme erscheinen in `doctor`, Metrik, CLI, MCP-An... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-19). |
| `P-12` | `docs/SPEC.md` | 2490 | ##### C-12.19.2 `RecipientLedger` (Art. 17 Abs. 2, Art. 19, P-12) | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-12). |
| `O-21` | `docs/SPEC.md` | 2520 | - **CLI:** `init`, `doctor` (inkl. `verify-report`, `export-concept`, `crypto-drill`), `status`, `confirm <token>`, `... | nicht definiert | Als offener Punkt / Kalibrierungsparameter dokumentiert, Definition in §16 ergänzen. |
| `REQ-MCP-004` | `docs/SPEC.md` | 2528 | **Sicherheitspipeline (fünf Stufen je Aufruf, REQ-MCP-004):** | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-MCP-008` | `docs/SPEC.md` | 2529 | 1. Schema- und Größenvalidierung (RPC-Größe, Argumentgrenzen; fuzz-geprüft, REQ-MCP-008). | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-MCP-005` | `docs/SPEC.md` | 2543 | Das Werkzeug antwortet dann mit `ConfirmationRequired { token, preview }`. Der Token ist **nur über einen Kanal außer... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-FAC-001` | `docs/SPEC.md` | 2547 | Die Fassade `contextra` ist der **einzige** Ort, an dem konkrete Implementierungen gewählt und verdrahtet werden (P29... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-FAC-002` | `docs/SPEC.md` | 2552 | 4. führt beim Start die Phasen B0–B9 aus (§13.5), einschließlich eines **Verdrahtungs-Selbsttests**: jeder Sicherheit... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-027` | `docs/SPEC.md` | 2555 | Ohne Löschpfad-Orchestrator liefert dessen Konstruktor `Err(CapabilityUnsupported)` und nie einen Stub, der Erfolg me... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-FAC-005` | `docs/SPEC.md` | 2557 | #### C-13.5 Startsequenz B0–B9 (INV-BOOT-1, REQ-FAC-005) | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-RST-1` | `docs/SPEC.md` | 2569 | \| B6 \| `ErasedSet` laden und Lesesperre armieren (INV-BOOT-1, REQ-RST-1) \| unlesbarem oder manipuliertem `ErasedSe... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-FGT-2` | `docs/SPEC.md` | 2570 | \| B7 \| Löschsagas aus dem `ErasureJournal` fortsetzen oder als `Partial` melden (REQ-FGT-2); unterbrochene Destroy-... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-MCP-010` | `docs/SPEC.md` | 2576 | #### C-13.6 Fehlermodell (REQ-MCP-010, A6) | definiert | In Anforderungskatalog / SPEC geführt. |
| `O-18` | `docs/SPEC.md` | 2595 | \| Windows \| offen (O-18) \| offene oder gemappte Dateien sind nicht löschbar (Handles freigeben, `FILE_SHARE_DELETE... | nicht definiert | Als offener Punkt / Kalibrierungsparameter dokumentiert, Definition in §16 ergänzen. |
| `§25` | `docs/SPEC.md` | 2597 | **Dateisysteme.** Unterstützt sind lokale ext4, xfs, btrfs, zfs, APFS und NTFS. Abgelehnt im persistenten Betrieb sin... | nicht definiert | Verweis auf §16.1 (Prüfpunkte-Register) oder passenden Unterabschnitt anpassen. |
| `REQ-PLT-001` | `docs/SPEC.md` | 2597 | **Dateisysteme.** Unterstützt sind lokale ext4, xfs, btrfs, zfs, APFS und NTFS. Abgelehnt im persistenten Betrieb sin... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-PLT-003` | `docs/SPEC.md` | 2599 | **Durability-Primitive.** Der Commit-Punkt nutzt die plattformgerechte Primitive (§7.7). Löschlogik stützt sich nie a... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DUR-009` | `docs/SPEC.md` | 2607 | **DurabilityMode.** Es gibt genau zwei Modi: `Full` (WAL, HMAC-Kette und AEAD; Standard und einziger Modus für `Compl... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-RST-4` | `docs/SPEC.md` | 2611 | - **Erzeugung:** Ein Checkpoint ist ein konsistenter Snapshot an einem Sequenzstand. Seine Erzeugungszeit steht im si... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-013` | `docs/SPEC.md` | 2613 | - **Restore:** `restore_checkpoint` verifiziert Kette, Manifest und Format, sperrt vor dem ersten Lesezugriff, liest ... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-RST-2` | `docs/SPEC.md` | 2613 | - **Restore:** `restore_checkpoint` verifiziert Kette, Manifest und Format, sperrt vor dem ersten Lesezugriff, liest ... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-RST-3` | `docs/SPEC.md` | 2614 | - **Restore-Horizont (INV-RESTORE-HORIZON-1):** Schlüssel-Backups (Spur K), deren `valid_until` überschritten oder de... | definiert | In Anforderungskatalog / SPEC geführt. |
| `O-14` | `docs/SPEC.md` | 2619 | `contextra-db` stellt Collection-Handles und Transaktionen bereit. Die Logik liegt in `engine`; `db` enthält keine ei... | definiert | Dokumentiert in §16. |
| `§25` | `docs/SPEC.md` | 2619 | `contextra-db` stellt Collection-Handles und Transaktionen bereit. Die Logik liegt in `engine`; `db` enthält keine ei... | nicht definiert | Verweis auf §16.1 (Prüfpunkte-Register) oder passenden Unterabschnitt anpassen. |
| `REQ-DEL-031` | `docs/SPEC.md` | 2676 | \| `snapshot_max_age` \| 15 min (REQ-DEL-031) \| | definiert | In Anforderungskatalog / SPEC geführt. |
| `P-07` | `docs/SPEC.md` | 2682 | \| `erased_set_retention` \| 90 Tage (≥ `key_backup_max_age` + Zuschlag; Pflichtwert ohne Standard in `EnterpriseRegu... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-07). |
| `P-16` | `docs/SPEC.md` | 2682 | \| `erased_set_retention` \| 90 Tage (≥ `key_backup_max_age` + Zuschlag; Pflichtwert ohne Standard in `EnterpriseRegu... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-16). |
| `P-06` | `docs/SPEC.md` | 2683 | \| `key_backup_max_age` \| 30 Tage (≤ `erased_set_retention`) \| Lebensdauer eines Schlüssel-Backups (Spur K, §12.17)... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-06). |
| `H-07` | `docs/SPEC.md` | 2685 | \| `seal_after_deletion_max` \| 24 h (`Maximum`: 1 h; Regulated: ≤ 24 h) \| Epoch Seal nach Löschung (H-07); kurz, we... | definiert | Härtungshinweis in C-12. |
| `O-42` | `docs/SPEC.md` | 2689 | \| `n_shards` \| 256 \| KeyTable-Shards; per Benchmark kalibrieren (O-42) \| | nicht definiert | Als offener Punkt / Kalibrierungsparameter dokumentiert, Definition in §16 ergänzen. |
| `O-15` | `docs/SPEC.md` | 2694 | \| `checkpoint_policy` \| `Rewrite` \| Standard (§12.5.4; O-15) \| | definiert | Dokumentiert in §16. |
| `P-11` | `docs/SPEC.md` | 2695 | \| `checkpoint_hold` \| 25 Tage \| nur für `ExpireAfterHold` unter `Restricted`; interne Zielgröße unter der Monatsfr... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-11). |
| `P-15` | `docs/SPEC.md` | 2695 | \| `checkpoint_hold` \| 25 Tage \| nur für `ExpireAfterHold` unter `Restricted`; interne Zielgröße unter der Monatsfr... | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-15). |
| `P-11` | `docs/SPEC.md` | 2696 | \| `partial_alarm_after` \| 24 Stunden \| Interne Zielgröße (P-11, P-19) \| | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-11). |
| `P-19` | `docs/SPEC.md` | 2696 | \| `partial_alarm_after` \| 24 Stunden \| Interne Zielgröße (P-11, P-19) \| | nicht definiert | Eintrag im Prüfpunkte-Register (§16.1) anlegen (P-19). |
| `AC-1` | `docs/SPEC.md` | 2700 | \| Canary \| mindestens 32 Zufallsbyte als Text \| Eindeutigkeit gegen Zufallstreffer (AC-1) \| | definiert | In Abnahmekriterien §12.12 geführt. |
| `REQ-QA-006` | `docs/SPEC.md` | 2711 | `FaultVfs` (in `testkit`) injiziert seed-reproduzierbar an **jedem** Schreib- und Durability-Punkt: Crash, Torn Write... | definiert | In Anforderungskatalog / SPEC geführt. |
| `§25.3` | `docs/SPEC.md` | 2736 | Ergänzend gilt: AC-1 bis AC-38 aus §12.12 sind Abnahmekriterien für die Löschung. AC-7, AC-12, AC-14 und AC-15 bleibe... | nicht definiert | Verweis auf §16.1 (Prüfpunkte-Register) oder passenden Unterabschnitt anpassen. |
| `AC-1` | `docs/SPEC.md` | 2736 | Ergänzend gilt: AC-1 bis AC-38 aus §12.12 sind Abnahmekriterien für die Löschung. AC-7, AC-12, AC-14 und AC-15 bleibe... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-38` | `docs/SPEC.md` | 2736 | Ergänzend gilt: AC-1 bis AC-38 aus §12.12 sind Abnahmekriterien für die Löschung. AC-7, AC-12, AC-14 und AC-15 bleibe... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-7` | `docs/SPEC.md` | 2736 | Ergänzend gilt: AC-1 bis AC-38 aus §12.12 sind Abnahmekriterien für die Löschung. AC-7, AC-12, AC-14 und AC-15 bleibe... | nicht definiert (unbelegt) | In §12.12 als historisch unbelegt geführt, Nummer reserviert. |
| `AC-12` | `docs/SPEC.md` | 2736 | Ergänzend gilt: AC-1 bis AC-38 aus §12.12 sind Abnahmekriterien für die Löschung. AC-7, AC-12, AC-14 und AC-15 bleibe... | nicht definiert (unbelegt) | In §12.12 als historisch unbelegt geführt, Nummer reserviert. |
| `AC-14` | `docs/SPEC.md` | 2736 | Ergänzend gilt: AC-1 bis AC-38 aus §12.12 sind Abnahmekriterien für die Löschung. AC-7, AC-12, AC-14 und AC-15 bleibe... | nicht definiert (unbelegt) | In §12.12 als historisch unbelegt geführt, Nummer reserviert. |
| `AC-15` | `docs/SPEC.md` | 2736 | Ergänzend gilt: AC-1 bis AC-38 aus §12.12 sind Abnahmekriterien für die Löschung. AC-7, AC-12, AC-14 und AC-15 bleibe... | nicht definiert (unbelegt) | In §12.12 als historisch unbelegt geführt, Nummer reserviert. |
| `AC-28` | `docs/SPEC.md` | 2736 | Ergänzend gilt: AC-1 bis AC-38 aus §12.12 sind Abnahmekriterien für die Löschung. AC-7, AC-12, AC-14 und AC-15 bleibe... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-21` | `docs/SPEC.md` | 2736 | Ergänzend gilt: AC-1 bis AC-38 aus §12.12 sind Abnahmekriterien für die Löschung. AC-7, AC-12, AC-14 und AC-15 bleibe... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-22` | `docs/SPEC.md` | 2736 | Ergänzend gilt: AC-1 bis AC-38 aus §12.12 sind Abnahmekriterien für die Löschung. AC-7, AC-12, AC-14 und AC-15 bleibe... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-25` | `docs/SPEC.md` | 2736 | Ergänzend gilt: AC-1 bis AC-38 aus §12.12 sind Abnahmekriterien für die Löschung. AC-7, AC-12, AC-14 und AC-15 bleibe... | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-27` | `docs/SPEC.md` | 2736 | Ergänzend gilt: AC-1 bis AC-38 aus §12.12 sind Abnahmekriterien für die Löschung. AC-7, AC-12, AC-14 und AC-15 bleibe... | definiert | In Abnahmekriterien §12.12 geführt. |
| `REQ-PRF-6` | `docs/SPEC.md` | 2745 | \| RB2 Löschung \| Canary-Scanner (§17.9), Heap-Canary (§12.7.11) und ein Referenz-Prüfer in anderer Sprache **nur al... | definiert | In Anforderungskatalog / SPEC geführt. |
| `AC-1` | `docs/SPEC.md` | 2757 | \|M-DEL-01\|Memtable-Werte werden nicht zeroisiert\|AC-1, AC-34\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-34` | `docs/SPEC.md` | 2757 | \|M-DEL-01\|Memtable-Werte werden nicht zeroisiert\|AC-1, AC-34\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-1` | `docs/SPEC.md` | 2758 | \|M-DEL-02\|Antwort- oder RRF-Cache wird nicht invalidiert\|AC-1 (entschlüsselte Sicht), AC-5\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-5` | `docs/SPEC.md` | 2758 | \|M-DEL-02\|Antwort- oder RRF-Cache wird nicht invalidiert\|AC-1 (entschlüsselte Sicht), AC-5\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-1` | `docs/SPEC.md` | 2759 | \|M-DEL-03\|SQ8-Code im Arena-Slot wird nicht genullt\|`VectorScanner`-Test, AC-1\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-1` | `docs/SPEC.md` | 2761 | \|M-DEL-05\|alte Text-Segmentdatei bleibt\|AC-1, AC-35\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-35` | `docs/SPEC.md` | 2761 | \|M-DEL-05\|alte Text-Segmentdatei bleibt\|AC-1, AC-35\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-1` | `docs/SPEC.md` | 2762 | \|M-DEL-06\|WAL-Scrub lässt Altsegment\|AC-1, AC-3\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-3` | `docs/SPEC.md` | 2762 | \|M-DEL-06\|WAL-Scrub lässt Altsegment\|AC-1, AC-3\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-35` | `docs/SPEC.md` | 2763 | \|M-DEL-07\|Blob-Rewrite entfällt\|AC-35 (Inventar)\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-25` | `docs/SPEC.md` | 2764 | \|M-DEL-08\|D4 entfällt (alte KeyTable-Generation bleibt)\|AC-25, AC-36\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-36` | `docs/SPEC.md` | 2764 | \|M-DEL-08\|D4 entfällt (alte KeyTable-Generation bleibt)\|AC-25, AC-36\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-5` | `docs/SPEC.md` | 2765 | \|M-DEL-09\|`KeyCache` wird bei D2 nicht invalidiert\|AC-5, AC-28\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-28` | `docs/SPEC.md` | 2765 | \|M-DEL-09\|`KeyCache` wird bei D2 nicht invalidiert\|AC-5, AC-28\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-28` | `docs/SPEC.md` | 2766 | \|M-DEL-10\|D3 ohne Verzeichnis-`fsync`\|AC-28 (Crash-Matrix)\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-31` | `docs/SPEC.md` | 2767 | \|M-DEL-11\|Schritt 5 beginnt vor Siegel 2\|AC-31, Typestate (INV-DEL-ORDER-1)\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-2` | `docs/SPEC.md` | 2768 | \|M-DEL-12\|Scanner liefert konstant 0\|AC-2, AC-26\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-26` | `docs/SPEC.md` | 2768 | \|M-DEL-12\|Scanner liefert konstant 0\|AC-2, AC-26\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-1` | `docs/SPEC.md` | 2769 | \|M-DEL-13\|Positivkontrolle entfällt\|AC-1\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-35` | `docs/SPEC.md` | 2770 | \|M-DEL-14\|Scanner liest nur Teilmenge der Dateien\|Dateilisten-Digest gegen Verzeichnis (AC-35)\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-21` | `docs/SPEC.md` | 2771 | \|M-DEL-15\|`KEK[e]` überlebt den Seal\|AC-21, AC-25\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-25` | `docs/SPEC.md` | 2771 | \|M-DEL-15\|`KEK[e]` überlebt den Seal\|AC-21, AC-25\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-36` | `docs/SPEC.md` | 2772 | \|M-DEL-16\|geänderter Shard wird nicht neu geschrieben\|AC-36\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-4` | `docs/SPEC.md` | 2773 | \|M-DEL-17\|Restore-Guard entfällt\|AC-4, AC-27\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-27` | `docs/SPEC.md` | 2773 | \|M-DEL-17\|Restore-Guard entfällt\|AC-4, AC-27\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-5` | `docs/SPEC.md` | 2774 | \|M-DEL-18\|Lesesperre prüft nur `subject_ref_hash`, nicht `unit_ref_hash`\|AC-5\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-6` | `docs/SPEC.md` | 2776 | \|M-DEL-20\|Klasse C1 ohne Epoch Seal ausgestellt\|AC-6, INV-CLASS-MEASURED-1\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-24` | `docs/SPEC.md` | 2777 | \|M-DEL-21\|Satz ohne Messwert in der Satzbibliothek\|AC-24\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-30` | `docs/SPEC.md` | 2778 | \|M-DEL-22\|Padding abgeschaltet\|AC-30\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-29` | `docs/SPEC.md` | 2779 | \|M-DEL-23\|Mirror-Ausfall blockiert `Frozen`\|Chaos (AC-29)\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-32` | `docs/SPEC.md` | 2780 | \|M-DEL-24\|Uhrrücksprung wird ignoriert\|AC-32\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-35` | `docs/SPEC.md` | 2781 | \|M-DEL-25\|alter ArtKey schon in Schritt 3 vernichtet\|AC-35 (Inventar vor Vernichtung), INV-DEL-ORDER-1\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-38` | `docs/SPEC.md` | 2782 | \|M-DEL-26\|`flash_hardening` aus Konfiguration statt aus Ablauf\|AC-38\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-37` | `docs/SPEC.md` | 2783 | \|M-DEL-27\|Koaleszenz hebt die Klasse einer Einheit über ihre Messung\|AC-37\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `AC-35` | `docs/SPEC.md` | 2784 | \|M-DEL-28\|zweistufige Messung weicht vom Vollscan ab (Scanner übersieht Datei)\|AC-35\| | definiert | In Abnahmekriterien §12.12 geführt. |
| `REQ-DEL-011` | `docs/SPEC.md` | 2794 | Anfrage, Planner, Snapshot-Sequenz, `ScopeConstraint` und ACL-Bitset. Dann Query-Embedding im Blocking-Pool, Vektorsu... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-DEL-021` | `docs/SPEC.md` | 2800 | 1. **Fast-Path.** `Frozen` (`ErasedSet`-Eintrag durabel, Lesesperre, Spiegelung an den Mirror; Siegel 1, Klasse L), s... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-RST-5` | `docs/SPEC.md` | 2807 | Siehe §13.8. Export und Restore unterliegen dem Restore-Horizont (INV-RESTORE-HORIZON-1). Checkpoints im Datenverzeic... | definiert | In Anforderungskatalog / SPEC geführt. |
| `REQ-TEN-004` | `docs/SPEC.md` | 2809 | #### C-24.5 Tenant-Lebenszyklus (REQ-TEN-004) | definiert | In Anforderungskatalog / SPEC geführt. |
| `O-12` | `docs/PLAN.md` | 108 | 6. **Zielcommit festlegen** (O-12) und den flachen Klon vertiefen (`git fetch --unshallow`); alle Zahlen aus 8f8fa7cd... | definiert | Dokumentiert in §16. |
| `§19` | `docs/PLAN.md` | 135 | 3. Anforderungskatalog (alt §19) nach `docs/spec/requirements.toml` überführen, je Anforderung `module`, `test`, `sta... | nicht definiert | Abschnitt existiert in 3.1 nicht mehr als Hauptkapitel; Verweis prüfen. |
| `E-0.5` | `docs/PLAN.md` | 158 | 2. `LayerCleanupProof`: Entscheidung E-0.5 (Test an den gültigen V5-Vertrag anpassen oder, falls der Vertrag den Typ ... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-1.1` | `docs/PLAN.md` | 196 | **Aufgaben:** `get_protected_regex()` darf nicht panicken. Muster einmalig beim Start kompilieren und **Fehler als `C... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-1.5` | `docs/PLAN.md` | 230 | **Aufgaben:** (1) Stellen lokalisieren und prüfen, ob der Code durch WAL v3 (HMAC-Kette) und Record-Verschlüsselung ü... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-2.2` | `docs/PLAN.md` | 268 | **Aufgaben:** Port `RelationIndex` im besitzenden Crate (`graph`) bzw. in `ports` (Entscheidung E-2.2); `engine` ford... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-2.14` | `docs/PLAN.md` | 384 | \|`contextra` (Fassade)\|`license`, `core` hart\|`license` optional (P30), `core` entfällt\|WP 2.8; Port `LicenseGate... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `O-08` | `docs/PLAN.md` | 419 | **Selbsttest (Pflicht):** Ein Meta-Test erzeugt absichtlich ein Crate mit (a) nicht deklarierter Kante, (b) `panic!` ... | definiert | Dokumentiert in §16. |
| `O-07` | `docs/PLAN.md` | 455 | 5. Hardwareklasse festschreiben (O-07); **bis dahin keine absoluten Leistungsversprechen** in Dokumenten. | definiert | Dokumentiert in §16. |
| `O-08` | `docs/PLAN.md` | 465 | 1. Das vorhandene Inventar fortschreiben: je Gate/Prüfung **Zweck, Eingabe, Laufzeit, Fehlalarmquote, letzter wirksam... | definiert | Dokumentiert in §16. |
| `E-1` | `docs/PLAN.md` | 583 | 4. Verdrahtung nur in der Fassade (EP7); bestehende Saga-Reste in `engine`, `mcp`, `crypto` hierhin verlagern. **Schn... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-7` | `docs/PLAN.md` | 583 | 4. Verdrahtung nur in der Fassade (EP7); bestehende Saga-Reste in `engine`, `mcp`, `crypto` hierhin verlagern. **Schn... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-6` | `docs/PLAN.md` | 583 | 4. Verdrahtung nur in der Fassade (EP7); bestehende Saga-Reste in `engine`, `mcp`, `crypto` hierhin verlagern. **Schn... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-7` | `docs/PLAN.md` | 583 | 4. Verdrahtung nur in der Fassade (EP7); bestehende Saga-Reste in `engine`, `mcp`, `crypto` hierhin verlagern. **Schn... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `O-02` | `docs/PLAN.md` | 601 | **Aufgaben:** `RecipientLedger` (Art. 17 Abs. 2, Art. 19), Fristen mit Standard und Obergrenzen (kalibrierbar, Anhang... | definiert | Dokumentiert in §16. |
| `O-01` | `docs/PLAN.md` | 617 | 1. `docs/spec/legal-matrix.toml` aus SPEC §9.12 mit den Feldern `vorgabe`, `anforderung`, `baustein`, `test`, `status... | definiert | Dokumentiert in §16. |
| `O-06` | `docs/PLAN.md` | 617 | 1. `docs/spec/legal-matrix.toml` aus SPEC §9.12 mit den Feldern `vorgabe`, `anforderung`, `baustein`, `test`, `status... | definiert | Dokumentiert in §16. |
| `O-11` | `docs/PLAN.md` | 617 | 1. `docs/spec/legal-matrix.toml` aus SPEC §9.12 mit den Feldern `vorgabe`, `anforderung`, `baustein`, `test`, `status... | definiert | Dokumentiert in §16. |
| `O-01` | `docs/PLAN.md` | 619 | 3. Externe Rechtsprüfung (O-01, O-02, O-05) liefert Eingaben; die Matrix ist **keine** Rechtsauskunft und keine Zerti... | definiert | Dokumentiert in §16. |
| `O-02` | `docs/PLAN.md` | 619 | 3. Externe Rechtsprüfung (O-01, O-02, O-05) liefert Eingaben; die Matrix ist **keine** Rechtsauskunft und keine Zerti... | definiert | Dokumentiert in §16. |
| `O-05` | `docs/PLAN.md` | 619 | 3. Externe Rechtsprüfung (O-01, O-02, O-05) liefert Eingaben; die Matrix ist **keine** Rechtsauskunft und keine Zerti... | definiert | Dokumentiert in §16. |
| `O-02` | `docs/PLAN.md` | 620 | 4. Änderungsverfahren: Rechtsänderung ⇒ Zeile, Ticket, betroffener Baustein, Test; Fristen und Legal-Hold-Kataloge bl... | definiert | Dokumentiert in §16. |
| `E-1.1` | `docs/PLAN.md` | 735 | \|E-1.1\|Tokenizer: Muster-Fehler fail-closed oder Degradationspfad?\|Konstruktorfehler / Betrieb ohne geschützte Mus... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-2.2` | `docs/PLAN.md` | 736 | \|E-2.2\|Wo liegt `RelationIndex`?\|in `graph` / in `ports`\|in `ports`, solange `graph`-freie Signatur; sonst `graph... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-2.4` | `docs/PLAN.md` | 737 | \|E-2.4\|Welche Ports bleiben in `ports`?\|nach Nutzungsanalyse\|nur Querschnitts-Ports\|2.4\| | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-3.1` | `docs/PLAN.md` | 738 | \|E-3.1\|`xtask` behalten oder auf `cargo-make`/`just` auslagern?\|behalten, bereinigt / ersetzen\|behalten, Budget s... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-4.1` | `docs/PLAN.md` | 739 | \|E-4.1\|SSTable-Magic `MFST` (Legacy) weiter lesen?\|lesen+migrieren / ablehnen\|lesen, nur einmalig migrieren, dana... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-5.1` | `docs/PLAN.md` | 740 | \|E-5.1\|Schlüsselverwahrung K1 im Modul oder nur Port?\|K1 Software / nur Port\|K1 Software als Referenz, K2/K3 nur ... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-5.2` | `docs/PLAN.md` | 741 | \|E-5.2\|Standard-Policy für interne Checkpoints\|`Rewrite` / `ExpireNow`\|`Rewrite` (SPEC §9.4)\|5.7\| | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-6.1` | `docs/PLAN.md` | 742 | \|E-6.1\|`adapt` zurückbauen, wenn Benchmark keinen Nutzen zeigt?\|behalten / Archiv-Tag\|zurückbauen (EP12)\|6.3\| | definiert | Entscheidungs- / Entkopplungs-ID. |
| `O-09` | `docs/PLAN.md` | 743 | \|E-6.2\|DiskANN/RaBitQ im Designpunkt ≤ 10⁷ behalten? (O-09)\|behalten / entfernen\|nach Benchmark entscheiden; bis ... | definiert | Dokumentiert in §16. |
| `E-6.2` | `docs/PLAN.md` | 743 | \|E-6.2\|DiskANN/RaBitQ im Designpunkt ≤ 10⁷ behalten? (O-09)\|behalten / entfernen\|nach Benchmark entscheiden; bis ... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `O-10` | `docs/PLAN.md` | 744 | \|E-7.1\|Plattformen: macOS/Windows als Edge unterstützen? (O-10)\|ja / nein\|macOS ohne Garantieaussage, Windows nic... | definiert | Dokumentiert in §16. |
| `E-7.1` | `docs/PLAN.md` | 744 | \|E-7.1\|Plattformen: macOS/Windows als Edge unterstützen? (O-10)\|ja / nein\|macOS ohne Garantieaussage, Windows nic... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-0.5` | `docs/PLAN.md` | 745 | \|E-0.5\|`LayerCleanupProof`: Test an V5-Vertrag anpassen oder Typ im Vertrag führen?\|Test anpassen / Typ einführen\... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-1.5` | `docs/PLAN.md` | 746 | \|E-1.5\|`store`: undefinierte Features `encryption-at-rest`, `wal-integrity`\|Code entfernen / unbedingt kompilieren... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-2.14` | `docs/PLAN.md` | 747 | \|E-2.14\|Lizenzprüfung ohne harte Kante\|Port `LicenseGate` in Fassade / Feature\|Port `LicenseGate`; `license` nie ... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-3.8` | `docs/PLAN.md` | 748 | \|E-3.8\|Dokumentationssprache\|`rustdoc` englisch + Leitfäden deutsch / alles deutsch\|`rustdoc` englisch (Ökosystem... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `O-01` | `docs/PLAN.md` | 750 | Rechts- und Marktfragen (O-01 bis O-06) sind **nicht** durch Entwicklung lösbar und stehen in SPEC §16. | definiert | Dokumentiert in §16. |
| `O-06` | `docs/PLAN.md` | 750 | Rechts- und Marktfragen (O-01 bis O-06) sind **nicht** durch Entwicklung lösbar und stehen in SPEC §16. | definiert | Dokumentiert in §16. |
| `O-01` | `docs/PLAN.md` | 761 | \|R4\|Rechtliche Erwartungen überschreiten technische Aussage\|Haftung, Fehlvertrauen\|Bericht weist Grenzen verpflic... | definiert | Dokumentiert in §16. |
| `O-06` | `docs/PLAN.md` | 761 | \|R4\|Rechtliche Erwartungen überschreiten technische Aussage\|Haftung, Fehlvertrauen\|Bericht weist Grenzen verpflic... | definiert | Dokumentiert in §16. |
| `O-07` | `docs/PLAN.md` | 763 | \|R6\|Benchmarks auf falscher Hardware\|Falsche Leistungsaussagen\|Hardwareklasse festschreiben (O-07); keine absolut... | definiert | Dokumentiert in §16. |
| `O-12` | `docs/PLAN.md` | 769 | \|R12\|Drei Commit-Stände (Baseline, Scan, Klon) werden vermischt\|Falsche Zahlen, nicht reproduzierbare Aussagen\|Zi... | definiert | Dokumentiert in §16. |
| `O-06` | `docs/PLAN.md` | 773 | \|R16\|Rechts- und Regulierungsänderungen (Aufsichtspraxis, KI-Verordnung, Cyber Resilience Act)\|Bausteine veralten\... | definiert | Dokumentiert in §16. |
| `O-11` | `docs/PLAN.md` | 773 | \|R16\|Rechts- und Regulierungsänderungen (Aufsichtspraxis, KI-Verordnung, Cyber Resilience Act)\|Bausteine veralten\... | definiert | Dokumentiert in §16. |
| `O-12` | `docs/PLAN.md` | 797 | 1. **0.1 (Beginn):** Zielcommit festlegen, Klon vertiefen, Rohlogs sichern (O-12). Ohne Logs bleiben Ursachenaussagen... | definiert | Dokumentiert in §16. |
| `E-0.5` | `docs/PLAN.md` | 835 | \|**3.1**\|**28.10.2026**\|Baseline (E2, teilweise), Metrik-Datei und Scan eingearbeitet (U-15 bis U-27); WP 0.5–0.7 ... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-1.5` | `docs/PLAN.md` | 835 | \|**3.1**\|**28.10.2026**\|Baseline (E2, teilweise), Metrik-Datei und Scan eingearbeitet (U-15 bis U-27); WP 0.5–0.7 ... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-2.14` | `docs/PLAN.md` | 835 | \|**3.1**\|**28.10.2026**\|Baseline (E2, teilweise), Metrik-Datei und Scan eingearbeitet (U-15 bis U-27); WP 0.5–0.7 ... | definiert | Entscheidungs- / Entkopplungs-ID. |
| `E-3.8` | `docs/PLAN.md` | 835 | \|**3.1**\|**28.10.2026**\|Baseline (E2, teilweise), Metrik-Datei und Scan eingearbeitet (U-15 bis U-27); WP 0.5–0.7 ... | definiert | Entscheidungs- / Entkopplungs-ID. |

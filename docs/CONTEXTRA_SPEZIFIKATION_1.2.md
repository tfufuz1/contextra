# CONTEXTRA — Gesamtspezifikation

```
Dokument:  Normative Gesamtspezifikation (eigenständig, in sich vollständig)
Fassung:   1.2 (final)
Stand:     2026-10-25
Geltung:   Zielzustand des fertigen Produkts
Änderung:  1.2 gegenüber 1.1 — Verdrahtung und Semantik: Sidecar-Logs neben dem WAL (§7.11), Notlöschpfad (§7.12), Rand-Brücken (§6.11), Zweckregister (§20.2c), Löschschichten-Katalog (§20.4a), Startsequenz B0–B9 (§22.1), Tenant-Lebenszyklus (§24.5), Port-Katalog mit Namensauflösung (Anhang A.1/A.2), Notreserve und Snapshot-Höchstalter (§25.4); 5 neue Invarianten, 10 neue Anforderungen; Abschnittsnummern der Fassung 1.1 unverändert (nur neue Unterabschnitte).
           1.1 gegenüber 1.0 — §5.1/§5.2 (vollständiges Invariantenregister), §27.6 und §33 ergänzt
           1.1 gegenüber 1.0 — §5.1/§5.2 (vollständiges Invariantenregister), §27.6 und §33 ergänzt
```

> **Zweck.** Dieses Dokument ist die einzige Quelle der Soll-Architektur von Contextra: eine lokale, eingebettete Speicher- und Such-Engine für KI-Agenten in reinem Rust, ausgeliefert als Bibliothek und als MCP-Server. Es ist aus sich heraus verständlich und verweist auf keine anderen Dokumente. Alle rechtlichen, fachlichen und architektonischen Grundlagen, die für die Anforderungen nötig sind, stehen in diesem Dokument selbst (insbesondere §20.0 für den rechtlichen Rahmen der Löschung).
>
> **Abgrenzung.** Die Spezifikation beschreibt das Ziel. Sie enthält keine Statusangaben, keine Fehlerlisten und keine Abweichungslisten zwischen Soll und Code; diese gehören in begleitende Arbeitsunterlagen (§28.5). Eine solche Unterlage ist der **Implementierungsplan (Dokument B)**: Er leitet Reihenfolge, Phasen und Abnahmekriterien aus dieser Spezifikation ab, führt keine eigene Anforderung ein und ist für das Verständnis dieses Dokuments nicht erforderlich; bei Widerspruch gilt diese Spezifikation. **Abschnittsnummern sind stabil** (Code, Gates und ADRs verweisen darauf); neue Inhalte erhalten neue Unterabschnitte.
>
> **Rechtlicher Hinweis.** Die Spezifikation enthält keine Rechtsberatung. Rechtliche Wertungen sind als Prüfpunkte geführt (§20.13) und von einer fachkundigen Stelle zu klären, bevor Produkttexte eine Löschwirkung versprechen.
>
> **Beweisprinzip (INV-EVIDENCE-1):** Jede Zusage dieses Dokuments ist ein **Ziel**, kein Befund. Sie gilt erst, wenn der in §27.6 genannte Mechanismus maschinell belegt ist (Evidenzstufe, §27.7). Berichte, Audits, Skriptausgaben und Agentenaussagen sind Hypothesen, keine Belege; sie werden am Code geprüft, bevor sie übernommen werden.
>
> **Schlüsselwörter:** **MUSS**, **DARF NICHT**, **SOLL**, **KANN** sind normativ; ein Satz ohne normatives Schlüsselwort ist Beschreibung. `INV-*`, `P*`, `VETO-*` sind unverletzlich (Vetos tragen maschinenlesbare IDs in der Verfassungsdatei, §28). Jede normative Anforderung hat eine stabile ID (`REQ-…`, §33), eine Klasse, einen Durchsetzungsmechanismus und einen Verweis auf den Abschnitt, in dem sie entsteht.
>
> **Normativitätsklassen** (je Abschnitt im Titel): **`[K]` Kern** gilt uneingeschränkt. **`[R]` Rand** gilt nur bei aktiviertem Feature und DARF Kern-Garantien nie schwächen. **`[E]` Experimentell** ist nicht normativ (Zielbild, keine Release-Zusage, keine Beweislast für den Kern). Zuordnung je Crate und Feature: §6.0.
>
> **Aufbau.** §1–§5 Vision, Philosophie, Grenzen, Invarianten (Register §5.2) · §6 Ringe und Crates · §7–§19 Subsysteme · §20 Kryptografie, Löschung, Nachweis, Recht · §21–§23 Schnittstellen, Fassade, Betrieb · §24 Datenlebenszyklus · §25–§26 Konstanten und Verträge · §27–§28 Qualität und Governance · §29–§30 Erweiterung und Sicherheit · §31–§32 Glossar und Kurzfassung · §33 Anforderungskatalog · §34 Grundsatzentscheidungen · Anhang A Port-Katalog.

---

## 1. Vision `[K]`

**Contextra ist die lokale, eingebettete Speicher- und Gedächtnis-Infrastruktur für KI-Anwendungen: Schlüssel-Wert-Speicher, Vektorindex, Volltext und Graph bilden mit Metadaten, Audit und Kryptografie eine einzige, atomar konsistente Wahrheit (ein WAL), mit kryptografisch verifizierbarer Provenienz und Löschung.** Zugang: Rust-API und MCP (stdio).

**Produktkern in einem Satz:** *Contextra Memory — der lokale Speicher für KI-Agenten, der Informationen nachweisbar vergessen kann.* Ein enges Versprechen lässt sich beweisen, ein breites nur behaupten; deshalb gilt **Tiefe vor Breite** (§3 Punkt 14, §6.8). „Nachweisbar“ ist ehrlich definiert: Der Löschbeweis ist eine signierte Behauptung des löschenden Systems; unabhängige Beweiskraft entsteht erst durch Schlüsselvernichtung mit ausgewiesener Verwahrungsstufe (§20.2a), einen Verifier in anderer Sprache (§20.5) und reproduzierbare Canary-Läufe (§27.8, §27.9). Die Grenzen stehen im Robustheitsvertrag (§4.4).

**Strategischer Hebel:** minimaler, gehärteter, deterministischer Kern; alles Domänenspezifische (Metrik, Merge-Semantik, Filter, Analyzer, Embedder, Ranker) hängt an **Ports** (IoC) — statisch als generisches Rust oder zur Laufzeit als WASM-Plugin `[R]`.

|Eigenschaft|Bedeutung|
|---|---|
|**Isoliert**|Mandanten sind physisch getrennt (Schlüssel, Indexe, Dateien, §9.2); Plugins laufen in Wasmtime ohne Ambient Authority `[R]`.|
|**Deterministisch**|Gleicher Anfangszustand + gleiche WAL-Folge ⇒ bit-identischer Zustand. Nichtdeterministische Quellen (LLM, Embedder, Zufall, Uhr) gehen nur als **aufgezeichnete Werte** ins WAL (INV-NONDET-1).|
|**Zero-Overhead**|Hot-Path generisch monomorphisiert; WASM nur außerhalb des Hot-Paths.|
|**Single-Node**|Ein Primärprozess hält ein Datenverzeichnis; weitere lokale Prozesse sprechen über lokales IPC mit ihm (§21.1). Kein Sharding, kein P2P, keine Replikation.|

**Einsatzgebiete und Designpunkte:** (1) Einzelnutzer-Workstation/Desktop-Agent; (2) Edge/Embedded; (3) regulierter lokaler Betrieb (Recht/Medizin), mehrere Mandanten auf einem Knoten. **Designpunkt:** ≤ 10⁷ Vektoren und ≤ 10² Mandanten je Knoten; darüber nur Experimental-Modi (§9.7). Aussagen zu größeren Lasten sind nicht belegt. **Arbeitsannahme (per Benchmark zu belegen, nicht zugesichert):** Agentengedächtnis umfasst je Betroffenem typischerweise 10³ bis 10⁶ Einträge; Entwurfsentscheidungen (SQ8 mit exaktem Nachranking statt RaBitQ, sichere Standard-Kernel statt Intrinsics, Partitionierung je Betroffenem) stützen sich darauf, bis Messungen am Designpunkt etwas anderes zeigen.

---

## 2. Die fünf Produktmissionen

Jede Mission ist ein **testbarer Systemvertrag** mit Reifegrad.

### M1 — Zustandsbehaftete KI: persistenter Prefix-Cache `[R]`
Attention-KV-Blöcke sind datenbankverwaltete Objekte (§13), tenant-isoliert, per Schlüsselvernichtung löschbar. **Vorbedingung:** Die Bedingungen B1 bis B3 aus §13.1 sind erfüllt: verdrahtete Löschkette, ein `Generator`-Backend mit `external_kv() = true` und ein **Break-even-Gate** mit realem Modell, das belegt, dass Laden eines Prefix schneller ist als sein Prefill. Ohne alle drei wird M1 nicht behauptet.

### M2 — Minimale Kontext-Latenz `[K]`
Messgröße ist **Time-to-Context (TTC)**: Anfrage → fusionierter, berechtigter Kontext, als p50/p99/p99.9 mit Konfidenzintervall je Tier und Last. Mittel: SIMD-Dispatch (Intrinsics nur soweit durch Benchmark belegt, §9.6), Arena-Allokation, Block-Max-WAND, SQ8-Vorstufe mit exaktem Rescoring, Bounded-Compute-Pool, Zero-Copy **ab Block-Cache** (entschlüsselte Blöcke als `Bytes`, §7.7). TTFT ist nur im Pfad mit integriertem Generator `[R]` messbar. Regression bricht die CI (§27.4).

### M3 — Präzises Grounding und Mandantensicherheit `[K]`
Retrieval ist ScopeConstraint-gebunden und **mandanten-/zeilensicher auf Vektorebene**: Index je Tenant (INV-TENANT-IDX-1), ACL-Bitset **vor** Distanzberechnung (§9.4), BM25-Statistiken je Tenant. Domänengewichtung (BM25F, Vokabulare, Kalibrierung) `[R]`.

### M4 — Belegbare Antworten `[K]`
Jedes Ergebnis ist auf einen WAL-Eintrag rückführbar (Lineage Quelle → Chunk → Embedding → Ergebnis). Indexe sind atomar mit den Metadaten synchronisiert (INV-ATOMIC-IDX-1). Provenienz ist kryptografisch verifizierbar (HMAC-Kette, AuditChain, Ed25519). Nicht Belegtes wird als „nicht belegt“ gekennzeichnet; Gelöschtes ist **für die im Beweis ausgewiesenen Schichten** nachweislich gelöscht (§20). Antwort-Grounding-Validierung (`GaspValidator`) ist `[R]` (§15.2).

### M5 — Deterministische Filterung neben den Daten `[R]`
Domänenfilter laufen als Fuel-begrenzte WASM-Module am Daten-Port (Compute-near-Data); gleiches Modul + gleicher Snapshot ⇒ gleiche Treffermenge.

---

## 3. Engineering-Philosophie (verbindlich) `[K]`

1. **Fail-closed, kein Prozessabbruch durch erwartbare Fehler.** Keine stillen Fallbacks; keine `unwrap`/`expect`/`panic!`-Pfade in Produktion; I/O-Fehler sind typisierte Fehler, nie SIGBUS/Abort (INV-IO-1).
2. **Messbare Testtiefe.** Jede `INV-*` hat einen Test, der ihre Verletzung erkennt (Mutation belegt Wirksamkeit); kritische Algorithmen sind property-/modell-/loom-/miri-verifiziert; Schwellen stehen in `governance/ratchet.toml` und dürfen nur steigen. Tests werden aus Spezifikation und Invarianten abgeleitet, nicht aus der Implementierung desselben Erzeugers (§28.3).
3. **Null technische Schuld.** Kein TODO, kein toter Code. _P29:_ Eine Fähigkeit existiert nur, wenn die Composition Root sie verdrahtet **und** ein Integrationstest sie über den echten Pfad beweist. Feature-gated `[R]`/`[E]`-Code ist kein toter Code, solange das Gate ihn klassifiziert.
4. **Gnadenloses Benchmarking mit fremder Baseline.** Jede veröffentlichte Zahl nennt Commit, Modell-Hash, Hardware, Workload und Vergleichsbasis; ohne Baseline ist sie nicht belegt (§26).
5. **Atomare Entwicklungszyklen.** Ein Zyklus = ein Rot→Grün-Schritt, ein Scope, ein verifizierter Commit. Merges laufen serialisiert über eine Merge-Queue, die das **Merge-Ergebnis** testet (§28.2).
6. **Jepsen-artiges Chaos-Testing** mit `FaultVfs` (§27.3).
7. **Deterministische Verdrahtung.** Nur die `contextra`-Fassade instanziiert und verbindet; keine globalen Singletons.
8. **IoC.** Merge, Metrik, Analyzer, Observer, Filter, Embedder, Ranker sind Ports mit Registrierung.
9. **Automatisierte Audits — und die Audits sind selbst Testobjekte** (§27.5, Meta-Gate).
10. **Absolute Verifizierbarkeit.** Jede Behauptung (gelöscht, ausgeliefert, berechtigt, unverändert) ist durch einen offline prüfbaren Beweis belegt oder ausdrücklich als nicht belegt deklariert.
11. **Compiler vor Heuristik.** Was `rustc`, `clippy`, `cargo metadata` oder `cargo public-api` prüfen können, prüfen sie; Text-/Regex-Audits ergänzen nur und sind nie alleinige Grundlage eines Gates.
12. **Bauen oder kaufen — belegt.** Eigenbau eines Standardbausteins (LSM, HNSW, BM25, Block-Cache, SIMD-Kernel, MCP-Protokollschicht) bleibt nur bei belegtem Vorteil in Kerneigenschaften (Löschbeweis, Determinismus, Mandantentrennung) oder Leistung gegenüber der besten fremden Implementierung hinter demselben Port (§26). Kryptografische Primitive werden nie selbst gebaut. **Reife Fremdimplementierungen dienen auch dort als Orakel im Test (Punkt 15), wo sie als Abhängigkeit nicht taugen.**
13. **Evidenz vor Behauptung.** Maßgeblich ist maschinell erzeugte, wiederholbare Evidenz: Befehl, Toolchain-Version, Commit, Exit-Code, Dauer, Ausführungszahlen, Hash (INV-EVIDENCE-1). Ein nicht ausgeführtes oder nicht verfügbares Prüfwerkzeug ist **rot**, nie „bedingt grün“. Jede Garantie ist an eine Evidenzstufe (§27.7) gebunden; `maturity` wird aus ihr berechnet, nicht behauptet.
14. **Tiefe vor Breite.** Der Kern ist geschlossen (§6.8). Eine neue Fähigkeit betritt ihn erst, wenn die Evidenz des bestehenden Kerns die Zielstufe erreicht hat. Experimentelles vergrößert nie die Beweislast, die CI-Zeit oder die Angriffsfläche des Kerns.
15. **Ein unabhängiges Orakel je Kerngarantie.** Jede Garantie G1–G4 (§4.4) hat ein Prüfmaß, das unabhängig entstanden ist: Referenzmodell, Brute-Force, Verifier in anderer Sprache, Canary-Scanner. Ein Orakel wird nicht aus denselben Datenstrukturen und nicht vom selben Erzeuger abgeleitet wie der geprüfte Pfad (INV-ORACLE-1, §27.8).

---

## 4. Geltungsbereich, Vetos, Ausschlüsse `[K]`

### 4.1 Vetos (unverhandelbar)

- **VETO-F10 — Keine mandantenübergreifenden Datenflüsse.** Kein Cache, Index (auch keine Graph-Kante *innerhalb* eines Index), KV-Block, Embedding, Statistik (z. B. IDF) oder Audit-Eintrag überschreitet Tenant-Grenzen. Der Tenant wird aus der Transaktion abgeleitet, nie aus Nutzerparametern vertraut.
- **VETO-OP03 — Kein Echtzeit-Audio/STT** im Kern.
- **VETO-F02 — HNSW-Löschung nur per Tombstone-Pruning**, kein partielles Rewiring des Graphen.
- **Single-Node-Doktrin** — kein Sharding, kein P2P, keine verteilte Replikation.

### 4.2 Scope-Ausschlüsse

Kein Python-First-Design (nur Bindung `py` `[R]`), kein GUI-Kern, keine Cloud-Pflicht, keine Telemetrie nach außen ohne ausdrückliche Freigabe, **kein Agenten-Orchestrator und kein LLM-Router im Kern** (`agent`, `router` sind `[E]`: Agentenschleife und Modellwahl gehören dem MCP-Host).

### 4.3 Ehrliche Grenzen des Löschversprechens

Contextra garantiert Löschung **innerhalb seiner Kontrolle** (§20.3). Zwei Mechanismen wirken zusammen: **kryptografisch** (Vernichtung der Scope-DEK, sofort, wirkt auch auf SSD-Remanenz, Dateisystem-Snapshots und Ciphertext-Backups, sofern diese den Keystore nicht enthalten) und **physisch** (Purge-Kompaktion, WAL-Scrub, Index-Scrub). Nicht garantiert: RAM-Kopien, Swap/Hibernation, Core-Dumps, Kopien des Keystores/Key-Escrows, in Modellparameter eingeflossenes Wissen, extern ausgeleitete Daten, Klartext-Schlüsselraum (daher INV-KEYSPACE-1). Diese Grenzen stehen **signiert im Löschnachweis** (`uncovered_layers`).

**Verbindliche Außenformulierung (INV-CLAIMS-1):** „Nach `forget` bzw. `drop_collection` ist der Scope logisch gelöscht (Tombstone, fsync). Ist sein Schlüssel vernichtet, ist er zusätzlich kryptografisch unlesbar; das weist der Beweis aus. Die physische Bereinigung folgt nach Flush, Purge-Kompaktion und WAL-Bereinigung; erst dann wird ein vollständiger Nachweis für die im Beweis aufgeführten Schichten ausgestellt. Nicht aufgeführte Schichten sind nicht abgedeckt. Für den physischen Datenträger gelten die dokumentierten Rahmenbedingungen (verschlüsseltes Volume, Swap deaktiviert oder verschlüsselt, Backups ohne Keystore).“ Aussagen wie „100 % gelöscht“ DÜRFEN NICHT verwendet werden.

**Beweiskraft.** Der Löschbeweis ist eine **signierte Behauptung** des löschenden Systems, kein unabhängiger Beleg physischer Vernichtung: Der Verifier prüft Signatur, Struktur und ausgewiesene Abdeckung, nicht, ob Bytes tatsächlich fehlen. Unabhängige Beweiskraft entsteht aus dem Zusammenspiel von (a) Schlüsselvernichtung mit im Beweis genannter Verwahrungsstufe (§20.2a), (b) dem Referenz-Verifier in anderer Sprache (§20.5) und (c) öffentlich reproduzierbaren Canary-Läufen über das gesamte Datenverzeichnis (§27.8, §27.9). Rechtsaussagen („DSGVO-konform“) gehören nicht in Produkt- oder Beweistexte; zulässig sind technische Aussagen („Scope-Schlüssel vernichtet, Verwahrungsstufe K1“). Ob Krypto-Shredding eine Löschung im Rechtssinn ist, bewerten Aufsichtsbehörden unterschiedlich; die Verwahrungsstufen sind der technische Hebel dafür.

### 4.4 Robustheitsvertrag `[K]`

Der Robustheitsvertrag ist das Produktversprechen und zugleich das Test-Backlog. **„100 % robust“ wird nie zugesagt.** Zugesagt wird ein **begrenztes Fehlermodell**, jede Zusage durch einen wiederholbaren Test gedeckt (§27.6, §27.7), und **Fail-Closed** für alles außerhalb des Modells (§7.10).

**Fehlermodell (innerhalb der Garantien):**
1. Prozess-Kill (`SIGKILL`) an beliebiger Stelle;
2. Stromausfall mit Verlust nicht geflushter Daten;
3. Teilschreibungen (Torn Writes) auf Sektor- oder Blockebene;
4. Umordnung nicht geflushter Schreibvorgänge;
5. `fsync`-/`fdatasync`-Fehler;
6. volles Medium (`ENOSPC`);
7. Bit-Rot einzelner Blöcke — **erkannt** (Prüfsumme, AEAD-Tag), nicht korrigiert;
8. beliebige, auch bösartige Eingaben (Bytes, Größen, ungültiges UTF-8).

**Garantien:**

|ID|Garantie|Hauptmechanismen|
|---|---|---|
|**G1 Crash/WAL**|Jede bestätigte Transaktion ist nach Recovery vollständig oder gar nicht sichtbar; „bestätigt“ heißt durabel auf dem Medium (§7.3, §7.8); nach einem `fsync`-Fehler wird nie weitergeschrieben (INV-FSYNC-1, §7.10).|WAL-Kette, Recovery, `Vfs`-Port, DST (§27.8)|
|**G2 Löschung/Krypto**|Nach SK und bestätigtem Beweis sind die Inhalte des Scopes im definierten Modell (§4.3, §20) nicht rekonstruierbar; der Beweis nennt belegte und unbelegte Schichten sowie die Verwahrungsstufe.|Scope-DEK, Purge, Witness, Canary-Orakel, Referenz-Verifier|
|**G3 Audit/Provenienz**|Löschungen, Schlüsselwiderrufe, Zugriffe, Egress und Freigaben sind in einer manipulationserkennenden Kette (AuditChain, HMAC-Kette) nachweisbar.|AuditChain, RevocationLog|
|**G4 Isolation**|SSI auf dem Keyspace, Snapshot-Isolation in Indexen (§8); keine mandantenübergreifenden Datenflüsse (VETO-F10); keine internen Schlüssel über die öffentliche API (INV-SCAN-1).|SSI-Validator, Index je Tenant, Property-/Fuzz-Tests|

**Nicht-Ziele (ausdrücklich nicht zugesichert):** SSD-Wear-Leveling-Reste vor/ohne SK; Backups und Dateisystem-Snapshots außerhalb von Contextra; bereits an Cloud-Modelle oder andere Egress-Ziele gesendete Daten; Prozessspeicherabbilder, Swap, Core-Dumps, Hibernation; Fremdprozesse (auch Root) mit Schreibzugriff auf das Datenverzeichnis; Hardware, die einen Flush bestätigt, ohne ihn auszuführen; Dateisysteme außerhalb der Garantieplattform (§23.4).

**Zustandsautomat und Evidenz:** Betriebszustände `Healthy → Degraded → ReadOnly → Poisoned` und ihre Übergänge sind in §7.10 normiert. Jede Garantie verweist über §27.6 auf Mechanismen, über §27.7 auf eine Evidenzstufe und über §27.10 auf **einen** wiederholbaren Befehl (`cargo xtask verify`).

---

## 5. Architekturprinzipien und Invarianten `[K]`

|ID|Prinzip|
|---|---|
|**P1**|Ring-Disziplin: Kanten (normal/build) zeigen auf gleichen oder niedrigeren Ring.|
|**P2**|Unsafe nur in drei Inseln (`simd`, `sys`, `wire`) im **eigenen** Code; jeder handgeschriebene `unsafe`-Block trägt `// SAFETY:` und ist durch Test/Miri gedeckt. Generierter Code ist nur zulässig, wenn er in `@generated`-Dateien isoliert, Generator und Version gepinnt, Ausgabe in der CI reproduzierbar geprüft, nur über sichere Adapter genutzt und von Fuzz/Miri erfasst wird. Edition-2024-Attribute `#[unsafe(no_mangle)]`/`#[unsafe(export_name)]`/`#[unsafe(link_section)]` zählen als `unsafe`. Native Drittabhängigkeiten stehen im Supply-Chain-Inventar (§28.4).|
|**P5**|DAG-Integrität: keine Aufwärtskanten, keine Zyklen (normal+build, **mit aufgelösten Features**); Dev-Aufwärtskanten nur als in `capabilities.toml` deklarierte Ausnahme.|
|**P7**|Zero-Panic im Produktionscode (`unwrap`/`expect`/`panic!`/`todo!`/`unimplemented!`/`unreachable!`, zusätzlich `assert!`, `process::exit`, unchecked Indexierung/Slicing); Lint-Satz in §27.5 auf `deny`. Ergebnisse von `Mutex::lock()`/`RwLock::*` (Poisoning) werden als typisierter Fehler behandelt und lösen einen definierten Gesundheitsübergang aus (§7.10, §8.1), nie `unwrap`. Ratchet-Baselines (`governance/*-baseline.toml`) dürfen nur sinken; das Ziel ist 0, ein Ratchet ist kein Beleg für „null“. Release-Profil: `overflow-checks = true` für Kern-Crates. FFI-Builds (`py`): `panic = "unwind"` und Abfangen an der Grenze; Binaries: `panic = "abort"`.|
|**P24**|Aller **Anwendungszustand** (Dokumente, Metadaten, Indexabsichten, Lineage, Retention-Entscheidungen) geht durch das WAL; kein Seitenpfad schreibt ihn. Die einzigen zulässigen weiteren persistenten Zustände sind die in §7.11 abschließend aufgezählten **Sidecar-Logs** (Schlüssel, Widerruf, Audit, Löschjournal, Löschmenge); jede andere Persistenz ist ein Gate-Fehler (INV-SIDECAR-1).|
|**P26**|Sync-Kern, Async-Schale: Ring-0-Crates (**einschließlich `vector`, `text`, `graph`**) führen **keine Async-Runtime** (`tokio`, `async-std` o. ä.) als Produktions- oder Build-Abhängigkeit und blockieren nie auf einer Runtime. Ihre Algorithmen (Suche, Index, Ranking, Kryptografie) sind synchron implementiert. **Port-Traits dürfen Futures zurückgeben** (`BoxFuture`/AFIT), damit Adapter I/O ausführen können; Implementierungen mit reiner CPU-Arbeit DÜRFEN nicht künstlich `async` sein. Die Runtime lebt in `engine` (Ring 3) bzw. in Adaptern (Ring ≥ 2/3/4); blockierende I/O in `async fn` ist verboten. Das Gate prüft die Runtime-Freiheit der Ring-0-Abhängigkeitsbäume und zählt `async fn` außerhalb von Port-Definitionen (Ratchet, Ziel 0).|
|**P28**|Determinismus: Zeit, Zufall, IDs aus injizierten Ports (`Clock`, `Rng`, `IdGenerator`); `SystemTime::now`, `Instant::now`, `thread_rng`, `std::collections::HashMap/HashSet` (RandomState) sind in Kernlogik per `clippy::disallowed_*` verboten. Ausnahme: Schlüssel-/Salt-Material aus dem System-RNG.|
|**P29**|Wired-or-it-doesn't-exist (§3).|
|**P30**|Kern-Isolation: `[K]`-Crates hängen weder normal noch build (im Release-Default-Feature-Set) von `[R]`/`[E]`-Crates ab; Rand-Funktionen erreichen den Kern nur über Ports, die die Fassade injiziert.|

|ID|Invariante|
|---|---|
|**INV-CORE-ISOLATION-1**|Umsetzung von P30: Gate über `cargo metadata` (aufgelöst, **einschließlich der Default-Features der Kern-Crates**) lehnt jede Kante `[K] → [R]/[E]` ab; Features der Klassen `[R]`/`[E]` sind im Release-Default aus (§6.0).|
|**INV-KMS-1**|Jeder Schlüssel stammt von einem `KmsProvider`; kein Modul hält Master-Material außerhalb des Providers; mindestens die lokale Implementierung ist verdrahtet (INV-WIRING-2). Ableitungen direkt aus Passphrase mit festem oder globalem Salt (Muster `CryptoKey::try_new(passphrase, konstanter_salt)`) sind außerhalb des Keystore-Pfads verboten; Aufruferregel im Gate.|
|**INV-DELETION-WIRE-1**|`forget` löst **alle sieben** Löschschichten über die Composition Root aus. Ein Beweis entsteht nur, wenn jede Schicht durch einen `LayerWitness` belegt oder mit `UncoveredReason` signiert als nicht abgedeckt deklariert ist (Typestate; Auslassen = Kompilierfehler).|
|**INV-SHRED-1**|Kryptografische Löschung ist erst vollzogen, wenn die Vernichtung der Scope-DEK **und** der Eintrag im `RevocationLog` durable (fsync) sind. Erst danach darf ein `DeletionReceipt` „kryptografisch gelöscht“ ausweisen.|
|**INV-SSI-ACTIVE-1**|SSI ist im Produktionspfad aktiv; Read-Sets inkl. Prefix-/Range-Phantomschutz werden bei jedem Commit validiert. Für Index-Lesezugriffe gilt §8 (Snapshot-Isolation, optional konservatives SSI).|
|**INV-INDEX-SNAPSHOT-1**|Jeder Indexeintrag trägt `born_seq`/`died_seq`; Suchen am Snapshot `S` sehen nur Einträge mit `born_seq ≤ S < died_seq`. `as_of` gilt für Indexe nur bis zum Retention-Horizont der Tombstones.|
|**INV-DOC-SINGLE-SOURCE-1**|Jede Tatsache (Konstante, Ring, Reifegrad, Format, ID) hat genau eine kanonische Quelle (`capabilities.toml`, die maschinenlesbare Verfassungsdatei, Code); Abgeleitetes wird generiert oder per Doc-Truth-Gate geprüft; doppelt vergebene IDs sind Gate-Fehler.|
|**INV-RING0-DETERMINISM-1**|Ring 0 ist deterministisch: kein I/O außerhalb der Ports (insbesondere `Vfs`), keine Uhr/kein Zufall ohne Port, kein Ergebnis hängt von Hash-Iterationsreihenfolge, Thread-Scheduling oder paralleler Float-Reduktion in unfixierter Reihenfolge ab.|
|**INV-NONDET-1**|Ergebnisse nichtdeterministischer Quellen (LLM-Ausgabe, Embedding-Inferenz, Zufall) werden als **Werte** ins WAL geschrieben; Recovery und Index-Rebuild lesen sie dort und rufen die Quelle nie erneut auf.|
|**INV-WAL-SSOT-1**|Jeder persistente **Anwendungszustand** ist aus WAL + Manifest rekonstruierbar; Indexe sind abgeleitet. Sicherheitszustand, der bewusst nicht aus dem WAL ableitbar ist (weil S3 das WAL scrubbt), liegt ausschließlich in den Sidecar-Logs des §7.11.|
|**INV-ATOMIC-IDX-1**|Metadaten- und Indexänderung gehören zu **einem** WAL-Batch; Recovery wiederholt Index-Intents idempotent.|
|**INV-INDEX-APPLY-1**|Indexe werden von **einem** Applier strikt in WAL-Reihenfolge angewendet; HNSW-Level folgen aus `hash(doc_id, seed)`, nie aus einem Laufzeit-RNG. Parallelität nur bei Rebuild mit deterministischer Partitionierung.|
|**INV-TENANT-1**|Tenant-Ableitung aus der Transaktion; jede Schlüsselkonstruktion enthält das Tenant-Präfix.|
|**INV-TENANT-IDX-1**|Vektor-, Text- und Graph-Indexe sind **physisch je (Tenant, Collection[, Modell-Hash])** getrennt; es gibt keine tenantübergreifende Kante, Posting-Liste oder Statistik. `drop_collection` = Schlüsselvernichtung + Dateilöschung.|
|**INV-KEYSPACE-1**|LSM-Schlüssel, Index-IDs und Dateinamen enthalten nur opake IDs/Hashes, nie personenbezogene Inhalte (sie überleben Crypto-Shredding).|
|**INV-DERIVED-1**|`forget` kaskadiert über die Lineage auf Ableitungen (Konsolidierungsfakten, Tripel, Chunk-Präfixe, KV-Blöcke, Zusammenfassungen); Ableitungen mit weiteren Quellen werden neu abgeleitet oder als `stale` markiert, nie stehen gelassen.|
|**INV-ENCRYPT-REST-1**|Nutzdaten sind at-rest AEAD-verschlüsselt, in zwei Ebenen (§20.2): Engine-Schlüssel (Struktur/Integrität) und Scope-DEK (Nutzwerte). Verschlüsselung ist Standard; ohne sie weist `doctor` rot aus, und sie ist nur für `EdgeMinimal`/Entwicklung zulässig.|
|**INV-ENCRYPT-REST-2**|Dateien, die per `mmap` zero-copy gelesen werden (Vollpräzisionsvektoren im DiskANN-Modus `[R]`), sind nur zulässig, wenn `doctor` ein verschlüsseltes Volume nachweist; andernfalls läuft der Pfad über einen Page-Decrypt-Cache ohne `mmap`.|
|**INV-FAILCLOSED-1**|Unklarer Zustand ⇒ Ablehnung, nie Raten.|
|**INV-KDF-1**|Aus Passphrasen abgeleitete Schlüssel entstehen über Argon2id mit zufälligem, persistiertem Salt und persistierten Parametern (`KdfHeader`; Untergrenze m ≥ 64 MiB, t ≥ 3, Kalibrierung auf ≥ 250 ms). Reines HKDF auf eine Passphrase und feste/globale Salts sind verboten. Passphrase nie ausschließlich per Umgebungsvariable; Keyfile, OS-Keyring oder interaktive Eingabe stehen bereit.|
|**INV-FSYNC-1**|Schlägt `fsync`/`fdatasync` fehl, ist der Handle unbestimmt: kein Ack, kein Retry auf demselben Handle, WAL-Handle wird vergiftet (Zustand `Poisoned`, §7.10), Weiterbetrieb erst nach Neustart mit Recovery-Replay. Rückschneiden ungeflushter Bytes ersetzt das nicht.|
|**INV-DURABILITY-PRIM-1**|Der Commit-Punkt nutzt die plattformgerechte Durability-Primitive (§7.8); „fsync“ ohne diese ist kein Commit.|
|**INV-ENOSPC-1**|Volles Medium hinterlässt keinen Phantom-Eintrag und keinen halb angewendeten Batch; WAL-Segmente werden vorab reserviert, sodass ENOSPC bei Rotation statt mitten im Record auftritt; Wechsel in einen definierten lesenden Zustand, retrybarer Fehler.|
|**INV-IO-1**|Lesepfade geben I/O-Fehler als typisierte Fehler zurück. `mmap` nur für vollständig prüfsummenverifizierte, unveränderliche, volumenlokale Dateien (§7.7); sonst `pread`.|
|**INV-FORMAT-1**|Jede persistente Datei trägt Magic und Formatversion; neuere Version ⇒ Öffnen verweigert; Migration nur explizit, getestet, mit Vorab-Backup (§7.2). Persistente Formate sind explizit spezifiziert (Feldreihenfolge, Endianness, Längen) und nicht aus `serde`-Derives abgeleitet.|
|**INV-LOCK-ORDER-1**|Vollständige, geordnete Sperrenhierarchie; keine Sperr-Guards über `.await` (§8.1).|
|**INV-HNSW-SLOT-1**|Arena-Slots gelöschter HNSW-Knoten werden nicht wiederverwendet, solange veraltete Kantenreferenzen möglich sind (§9.3).|
|**INV-EMBED-1**|Kein stiller Platzhalter-Embedder; Mock nur hinter `cfg(test)`/`test-utils`; `test-utils` ist in keinem Release-/Default-Feature-Set (§15.3).|
|**INV-EMBED-2**|Collection persistiert `embedding_model_hash` und Dimension; abweichender Hash wird abgewiesen (§15.3).|
|**INV-WIRING-2**|Sicherheits-Ports (Liste in `governance/security_ports.toml`: `KmsProvider`, `KeyCustodian`, `ModuleVerifier`, `EntityRecognizer`, `InjectionDetector`, `GroupRevocationListener`, `Vfs`, `DurableLog`, `ErasureParticipant`, `ErasureJournal` und die Lösch-Scanner; die Löschpfad-Ports bilden INV-DELETION-PORTS-1) haben in Produktion eine **reale** Implementierung; Null-, Test- und Accept-All-Implementierungen (konstantes `true`/`Ok(())`) als Standard sind unzulässig (§22).|
|**INV-DELETION-PORTS-1**|Die Löschpfad-Ports `KeyCustodian`, `ErasureParticipant`, `ErasureJournal`, `Vfs`, `DurableLog` und `KmsProvider` sind Sicherheits-Ports (INV-WIRING-2): Sie haben in Produktion reale Implementierungen, werden in `governance/security_ports.toml` geführt, und `forget` löst alle sieben Löschschichten über sie aus (INV-DELETION-WIRE-1).|
|**INV-PROOF-COVERAGE-1**|Jede der sieben Löschschichten erscheint in einem Beweis genau einmal als belegt oder nicht abgedeckt (§20.5).|
|**INV-WITNESS-MINT-1**|`LayerWitness` nur aus `store`/`vector` ausstellbar; Bypass-Konstruktoren außerhalb von Tests verboten (§20.5).|
|**INV-PROOF-KEY-1**|Signierschlüssel des Beweises stammt aus dem `KeyManager`, nie aus einem frei übergebenen Byte-String (§20.5).|
|**INV-CLAIMS-1**|Externe Zusicherungen werden aus der belegten Garantiestufe erzeugt, nie aus dem Zielzustand (§4.3, §20.7).|
|**INV-PROVENANCE-1**|Merges sind scope-treu, beschreibend benannt, serialisiert über eine Merge-Queue und auf dem Merge-Ergebnis kompilierend (§28.2).|
|**INV-PLATFORM-1**|Vertragliche Durabilitäts- und Löschgarantien gelten **nur auf der Garantieplattform** (Linux, ext4/xfs, x86_64/aarch64; §23.4). Alle anderen Plattformen und Dateisysteme sind Best Effort: `doctor` weist `BestEffort` aus, Löschbeweise tragen `platform_guarantee = false`. Löschpfade und `mmap` haben plattformspezifische Tests, wo eine Plattform unterstützt wird; Funktionen ohne Plattformäquivalent (`mlock`) sind im Betriebshandbuch und in `doctor` deklariert. Löschlogik stützt sich nie auf Unix-Unlink-Semantik offener Dateien.|
|**INV-IPC-1**|Genau ein Primärprozess hält das Datenverzeichnis; weitere Clients nutzen ausschließlich lokales IPC (Unix-Socket/Named Pipe, Peer-Credentials, kein TCP/UDP) (§21.1).|
|**INV-STDOUT-1**|`stdout` gehört ausschließlich dem MCP-Protokoll; Logging geht nach `stderr`/Datei; `clippy::print_stdout` ist in allen Crates des MCP-Binärs `deny`.|
|**INV-CANCEL-1**|Anfrageabbruch (`notifications/cancelled`, Timeout, Disconnect) wirkt kooperativ bis in die Engine; vor dem WAL-Append bricht er ab, danach wird er ignoriert (Commit bleibt atomar).|
|**INV-FORGET-GUARD-1**|Breite Löschungen (Selektor, Collection, Tenant, > `forget_confirm_threshold` Dokumente) benötigen eine Bestätigung über einen Kanal außerhalb des Tool-Aufrufs (§21.5).|
|**INV-TRUST-1**|Jeder Eintrag trägt `trust ∈ {user, agent, external}` und `source`; Recall liefert Inhalt strukturiert als Daten mit Trust-Label, nie als Anweisung (§21.6).|
|**INV-LOG-1**|Logs, Metriken, Fehlertexte und Panic-Meldungen enthalten keine Nutzinhalte, nur IDs und Zähler.|
|**INV-RETENTION-1**|Decay ist nur ein Ranking-Gewicht und löscht nie. Löschung entsteht ausschließlich durch ausdrückliche Aktion, `Working`-TTL (§17.1) oder eine opt-in `RetentionPolicy` mit Dry-run, Karenzzeit und Audit.|

Die Invarianten der Rand-/Experimentell-Abschnitte sowie die bislang nur im Text benannten Invarianten (`INV-EVIDENCE-1`, `INV-ORACLE-1`, `INV-SCAN-1`) stehen in §5.1; das Register in §5.2 führt alle.

### 5.1 Ergänzende Invarianten (Rand, Experimentell und verteilt definierte Kern-Invarianten)

Die folgenden Invarianten gelten wie die Tabelle oben; sie stehen hier als eigene Zeilen, damit jede `INV-*` dieses Dokuments an genau einer Stelle als Tabellenzeile definiert ist. Die Spalte „Herkunft“ weist aus, ob die Aussage unverändert aus dem Fachabschnitt übernommen oder erst gefasst wurde. `[R]`-Invarianten gelten nur bei aktiviertem Feature und schwächen nie eine Kern-Garantie; `[E]`-Invarianten sind nicht normativ (Zielbild).

|ID|Kl.|Invariante|Fachabschnitt|Herkunft|
|---|---|---|---|---|
|**INV-EVIDENCE-1**|`[K]`|Jede Zusage dieses Dokuments ist ein Ziel, bis der Mechanismus aus §27.6 sie maschinell belegt; maßgeblich ist maschinell erzeugte Evidenz (Befehl, Toolchain-Version, Commit, Exit-Code, Dauer, Ausführungszahlen, Hash). Ein nicht ausgeführtes oder nicht verfügbares Prüfwerkzeug ist rot.|Kopf, §3 Punkt 13, §27.7, §27.10|Namen im Text; hier als Zeile geführt|
|**INV-ORACLE-1**|`[K]`|Jede Kerngarantie G1–G4 hat ein Prüfmaß, das unabhängig entstanden ist: nicht aus denselben Datenstrukturen und nicht vom selben Erzeuger wie der geprüfte Pfad.|§3 Punkt 15, §27.8|Namen im Text; hier als Zeile geführt|
|**INV-SCAN-1**|`[K]`|Die öffentliche API enthält keine Funktion, keinen Typ und kein Feld, das Schlüsselmaterial (WAL-Integritätsschlüssel, DEK, KEK, Signierschlüssel) im Klartext zurückgibt; Test-Zugänge liegen hinter `cfg(test)` oder einem Test-Feature außerhalb jedes Release-/Default-Sets.|§30.1|Definition in §30.1; hier als Zeile geführt|
|**INV-SIDECAR-1**|`[K]`|Außer WAL, Manifest, SSTables, Blob-Segmenten und den Indexdateien existiert persistenter Zustand nur in den Sidecar-Logs aus §7.11. Jedes Sidecar-Log trägt Magic und Formatversion, eine HMAC-Kette mit eigenem Zweckschlüssel (§20.2c) und eine Manifest-Marke (Name, letzte Sequenz, Kettenkopf). Wirkungen, die ein WAL-Commit auslöst, stehen als `SidecarIntent` im selben Batch und werden idempotent angewendet; sicherheitskritische Wirkungen folgen „Sidecar vor Wirkung“.|§7.11|
|**INV-EMERGENCY-DEL-1**|`[K]`|Ein Betroffener ist in jedem Betriebszustand mindestens sperrbar: In `ReadOnly` (ENOSPC) sind `forget`, Lesesperre und Schlüsselvernichtung über eine vorab reservierte Notreserve ausführbar; in `Poisoned` ist der Shred-only-Betrieb über unabhängige Sidecar-Handles zulässig, WAL-Schreiben nicht. Fehlt auch das, gilt für den Betroffenen eine prozesslokale Lesesperre (fail-closed) bis zum Neustart.|§7.12|
|**INV-ASOF-ERASE-1**|`[K]`|Historische Lesezugriffe (`as_of`, offene Snapshots, Checkpoint-Lesen) unterliegen der Lesesperre: Ein gelöschter oder gesperrter Betroffener ist in keinem Snapshot lesbar, auch wenn die Altversion physisch noch existiert (S2/S3 `Pending`). Snapshots haben ein Höchstalter `snapshot_max_age`.|§8, §20.9|
|**INV-BOOT-1**|`[K]`|Der Start folgt der festen Phasenfolge B0–B9 (§22.1): Die Lesesperre (`ErasedSet`) ist armiert, bevor irgendein Lesepfad Verkehr annimmt, und jede Phase kann den Start mit benanntem Grund verweigern; es gibt keinen Teilstart.|§22.1|
|**INV-KEYPURPOSE-1**|`[K]`|Jeder Schlüssel dient genau einem Zweck und entsteht aus dem Zweckregister (§20.2c) mit eigenem HKDF-Label; kein Schlüssel wird für zwei Zwecke verwendet, und ein Label ohne Registereintrag ist ein Gate-Fehler.|§20.2c|
|**INV-ADMISSION-1**|`[R]`|Ein `AdmittedModule` ist nur über `ModuleVerifier::verify` konstruierbar (nicht öffentlicher Konstruktor); der SHA-256-Digest wird aus den Bytes berechnet, nie vom Aufrufer geliefert. Unsignierte Module sind nur mit `ApprovalRisk::Low` zulässig und im Audit als `unsigned` geführt.|§14.2|Aus §14.2 übernommen|
|**INV-APPROVAL-1**|`[R]`|Jede Gastausführung mit erweiterten Fähigkeiten durchläuft die Risikoklassifikation `ApprovalRisk`; `High`- und `Elevated`-Module laufen nicht vor `ApprovalStatus::Approved` (`Pending → Approved` oder `Pending → Rejected`, auditiert). Für `EnterpriseRegulated` ist die Freigabe Pflicht.|§14.1|Aus §14.1 übernommen|
|**INV-WASM-DET-1**|`[R]`|Gleiches Modul, gleiche Eingabe und gleiches Fuel liefern auf x86-64 und aarch64 ein bit-identisches Ergebnis; Wall-Clock-/Epoch-Unterbrechung ist nur Sicherheitsnetz und erzeugt fail-closed einen Fehler, nie ein Teilergebnis.|§14.3|Aus §14.3 übernommen|
|**INV-GASP-1**|`[R]`|Ist die Grounding-Validierung aktiviert, wird jede Tatsachenbehauptung gegen mindestens einen abgerufenen Chunk geprüft; der Konfidenz-Score ist kalibriert und an den `ConfigFingerprint` gebunden; unterhalb des Schwellenwerts (Default 0,70) liefert der Aufruf `PolicyViolation(LowConfidenceGrounding)`.|§15.2|Aus §15.2 übernommen|
|**INV-EGRESS-BUDGET-1**|`[R]`|Jede externe Wirkung (Egress, Modellaufruf, Plugin) ist budgetiert, auditiert und widerrufbar.|§19.1|Aus §19.1 übernommen|
|**INV-DECAY-1**|`[R]`|Decay ist ausschließlich ein Ranking-Gewicht in [0, 1] (NaN/∞ ⇒ 0.0) aus Aktivitätsuhr bzw. `Clock`-Port und löscht nie; unterhalb von `DECAY_FLOOR` ist ein Eintrag `dormant` (ausgeblendet, abrufbar, durchsuchbar, Teil aller Löschbeweise).|§17.0b|In §17.0b nur im Titel benannt; Aussage aus Formel und Text gefasst|
|**INV-MEMTYPE-1**|`[R]`|Fehlt `memory_type`, gilt `Semantic`.|§17.1|Aus §17.1 übernommen|
|**INV-MEMTYPE-2**|`[R]`|`Working`-Einträge laufen nach 50 000 Aktivitätsticks, `working_ttl_wall` (Default 4 h, `Clock`-Port) oder Sitzungsende ab, je nachdem, was zuerst eintritt; die Entfernung führt der Reaper (§16.4) über die Löschkette aus.|§17.1, §16.4|Aus §17.1 übernommen|
|**INV-MEMTYPE-3**|`[R]`|Die Verstärkung von `Procedural`-Einträgen ist nutzungs-, nicht zeitbasiert (`StepFloor`).|§17.1|Aus §17.1 übernommen|
|**INV-ROUTING-1**|`[E]`|Bandit-Strategien bleiben aus, bis ein Regret-Vorteil gegenüber `Cascade` im Benchmark-Gate nachgewiesen ist.|§18a|Aus §18a übernommen|
|**INV-ROUTING-2**|`[E]`|Das Routing ist bei gleichem Seed deterministisch (injizierte Zufallsquelle).|§18a|Aus §18a übernommen|
|**INV-ROUTING-3**|`[E]`|Der `ProfileCalibrationState` wird bei Wechsel des Konfigurations-Fingerprints invalidiert; bei `PrecisionMatrixDriftDetected` gilt fail-closed.|§18a|Aus §18a übernommen|
|**INV-LICENSE-1**|`[E]`|Verstoß, Ablauf oder fehlende Aktivierung degradieren auf die definierte Basisfunktion; sie führen niemals zu Datenverlust oder Datensperre (Lesezugriff und Export bleiben).|§23.5|In 1.0 nicht benannt; Aussage aus §23.5 („niemals Datenverlust oder -sperre“) als ID gefasst|
|**INV-LICENSE-2**|`[E]`|Die niedrigste Klasse (`Fast`) ist immer `Ok`.|§23.5|Aus §23.5 übernommen|
|**INV-LICENSE-3**|`[E]`|Eine falsche Installationsbindung liefert `NotActivated`, ohne Informationsleck.|§23.5|Aus §23.5 übernommen|

### 5.2 Invariantenregister (Vollständigkeitsindex)

Das Register führt **alle 67 Invarianten** dieses Dokuments (`[K]` 52, `[R]` 9, `[E]` 6). Jede `INV-*` ist genau einmal als Tabellenzeile definiert (§5 oder §5.1), hat eine Zeile in der Durchsetzungsmatrix (§27.6) und wird von mindestens einer Anforderung in §33 referenziert. Das Doc-Truth-Gate prüft diese drei Beziehungen und die Eindeutigkeit der IDs (REQ-GOV-004). Mechanismus-Kürzel wie in §27.6.

|ID|Kl.|Definition|Mechanismus (§27.6)|Anforderungen (§33)|
|---|---|---|---|---|
|INV-CORE-ISOLATION-1|`[K]`|§5|G|REQ-ARCH-003|
|INV-KMS-1|`[K]`|§5|G + I|REQ-KEY-001|
|INV-DELETION-WIRE-1|`[K]`|§5|T + I|REQ-DEL-001|
|INV-SHRED-1|`[K]`|§5|C + I|REQ-DEL-002|
|INV-SSI-ACTIVE-1|`[K]`|§5|P + C + L|REQ-MVCC-001|
|INV-INDEX-SNAPSHOT-1|`[K]`|§5|P + C + L|REQ-MVCC-002|
|INV-DOC-SINGLE-SOURCE-1|`[K]`|§5|G|REQ-GOV-004|
|INV-RING0-DETERMINISM-1|`[K]`|§5|G + P|REQ-ARCH-015, REQ-DUR-015|
|INV-NONDET-1|`[K]`|§5|P + G|REQ-DUR-012|
|INV-WAL-SSOT-1|`[K]`|§5|C + P|REQ-DUR-011, REQ-DUR-016|
|INV-ATOMIC-IDX-1|`[K]`|§5|C + P|REQ-DUR-011|
|INV-INDEX-APPLY-1|`[K]`|§5|C + P|REQ-DUR-011|
|INV-TENANT-1|`[K]`|§5|T + P + F + G|REQ-TEN-001|
|INV-TENANT-IDX-1|`[K]`|§5|T + P + F + G|REQ-IDX-001|
|INV-KEYSPACE-1|`[K]`|§5|T + P + F + G|REQ-IDX-010|
|INV-DERIVED-1|`[K]`|§5|I + P|REQ-DEL-007|
|INV-ENCRYPT-REST-1|`[K]`|§5|I + G + P|REQ-KEY-003|
|INV-ENCRYPT-REST-2|`[K]`|§5|I + G + P|REQ-KEY-004|
|INV-FAILCLOSED-1|`[K]`|§5|C + I|REQ-ARCH-016|
|INV-KDF-1|`[K]`|§5|I + G + P|REQ-KEY-002|
|INV-FSYNC-1|`[K]`|§5|C|REQ-DUR-002|
|INV-DURABILITY-PRIM-1|`[K]`|§5|C|REQ-DUR-001|
|INV-ENOSPC-1|`[K]`|§5|C|REQ-DUR-003|
|INV-IO-1|`[K]`|§5|G + C|REQ-DUR-004|
|INV-FORMAT-1|`[K]`|§5|G + F|REQ-DUR-005|
|INV-LOCK-ORDER-1|`[K]`|§5|L + G|REQ-MVCC-004|
|INV-HNSW-SLOT-1|`[K]`|§5|L + G|REQ-IDX-002|
|INV-EMBED-1|`[K]`|§5|G + I|REQ-IDX-011|
|INV-EMBED-2|`[K]`|§5|G + I|REQ-IDX-011|
|INV-WIRING-2|`[K]`|§5|G + I|REQ-DUR-015, REQ-DEL-023, REQ-FAC-003|
|INV-DELETION-PORTS-1|`[K]`|§5|G + I|REQ-DEL-023|
|INV-PROOF-COVERAGE-1|`[K]`|§5|T + I|REQ-DEL-003|
|INV-WITNESS-MINT-1|`[K]`|§5|T + G|REQ-DEL-004|
|INV-PROOF-KEY-1|`[K]`|§5|T + P|REQ-DEL-005|
|INV-CLAIMS-1|`[K]`|§5|G|REQ-DEL-006|
|INV-PROVENANCE-1|`[K]`|§5|G|REQ-GOV-001|
|INV-PLATFORM-1|`[K]`|§5|CI-Matrix + C|REQ-PLT-001|
|INV-IPC-1|`[K]`|§5|I + C + G|REQ-MCP-002|
|INV-STDOUT-1|`[K]`|§5|I + C + G|REQ-MCP-001|
|INV-CANCEL-1|`[K]`|§5|I + C + G|REQ-MCP-007|
|INV-FORGET-GUARD-1|`[K]`|§5|I + P|REQ-MCP-005|
|INV-TRUST-1|`[K]`|§5|I + P|REQ-MCP-006|
|INV-LOG-1|`[K]`|§5|G + P|REQ-AUD-002|
|INV-RETENTION-1|`[K]`|§5|P + I|REQ-COG-003|
|INV-EVIDENCE-1|`[K]`|§5.1|G|REQ-QA-001|
|INV-ORACLE-1|`[K]`|§5.1|P|REQ-QA-002|
|INV-SCAN-1|`[K]`|§5.1|G|REQ-ARCH-014|
|INV-SIDECAR-1|`[K]`|§5.1|G + C + I|REQ-DUR-016|
|INV-EMERGENCY-DEL-1|`[K]`|§5.1|C + I|REQ-DUR-017|
|INV-ASOF-ERASE-1|`[K]`|§5.1|P + L|REQ-DEL-030, REQ-DEL-031|
|INV-BOOT-1|`[K]`|§5.1|I + C|REQ-FAC-005|
|INV-KEYPURPOSE-1|`[K]`|§5.1|G + P|REQ-KEY-011|
|INV-ADMISSION-1|`[R]`|§5.1|T + P + F|REQ-SBX-001|
|INV-APPROVAL-1|`[R]`|§5.1|T + P + F|REQ-SBX-002|
|INV-WASM-DET-1|`[R]`|§5.1|T + P + F|REQ-SBX-003|
|INV-GASP-1|`[R]`|§5.1|I + P|REQ-COG-001|
|INV-EGRESS-BUDGET-1|`[R]`|§5.1|I + P|REQ-PRV-002|
|INV-DECAY-1|`[R]`|§5.1|I + P|REQ-COG-003|
|INV-MEMTYPE-1|`[R]`|§5.1|I + P|REQ-COG-002|
|INV-MEMTYPE-2|`[R]`|§5.1|I + P|REQ-COG-002|
|INV-MEMTYPE-3|`[R]`|§5.1|I + P|REQ-COG-002|
|INV-ROUTING-1|`[E]`|§5.1|OFFEN|REQ-EXP-001|
|INV-ROUTING-2|`[E]`|§5.1|OFFEN|REQ-EXP-001|
|INV-ROUTING-3|`[E]`|§5.1|OFFEN|REQ-EXP-001|
|INV-LICENSE-1|`[E]`|§5.1|P|REQ-EXP-003|
|INV-LICENSE-2|`[E]`|§5.1|P|REQ-EXP-003|
|INV-LICENSE-3|`[E]`|§5.1|P|REQ-EXP-003|

---

## 6. Ring-Modell und Crate-Katalog `[K]`

`capabilities.toml` ist die **einzige** Quelle für Ring, Klasse (`class`: `kern`/`rand`/`experimentell`) und `maturity` je Crate sowie für Dev-Ausnahmen; Skripte und Gates lesen nur dort (INV-DOC-SINGLE-SOURCE-1). Richtung: Ring 4 → 3 → 2 → 1 → 0. Kanten innerhalb eines Rings sind nur erlaubt, wenn sie als `may_use` des Crates deklariert und azyklisch sind; die Kantenregeln stehen in §6.10.

### 6.0 Kern, Ränder, Experimentell

|Klasse|Crates|Garantien|
|---|---|---|
|**`[K]` Kern**|`types`, `ports`, `mvcc`, `crypto`, `vector`, `text`, `store`, `checkpoint`, `engine`, `db`, `contextra`, `mcp` (+ Inseln `simd`, `sys`, `wire`)|Durabilität, Isolation, Löschung, Provenienz, Backup/Restore. Gilt für Release 1.0.|
|**`[R]` Rand**|`graph`, `rank`, `adapt`, `kvcache`, `cognition`, `privacy`, `sandbox`, `infer-candle`, `infer-ollama`, `py`|Hinter Feature-Flag/Port (Release-Default aus); Garantien einzeln ausgewiesen.|
|**`[E]` Experimentell**|`router`, `agent`, `infer-onnx`, `license`, `audit-export`, `avv-generator`|Default-off, ohne Garantien, von Release-Aussagen ausgeschlossen; Kandidaten für Auslagerung in eigene Repositories.|

`maturity = "stable"` setzt voraus: (a) alle Sicherheits-Ports des Crates sind produktiv verdrahtet (INV-WIRING-2), (b) alle Gates (§27.5) grün, (c) Fuzz-, Loom- und Miri-Abdeckung gemäß §27.1. Ein Crate mit nur Null-/Test-Implementierungen seiner Sicherheits-Ports ist `experimentell`. Kein `[K]`-Crate hängt von `[R]`/`[E]` ab (P30).

### 6.1 Ring 0 — Fundament (deterministisch, ohne Domänenwissen)

|Crate|Kl.|Verantwortung|
|---|---|---|
|`types`|K|IDs, Tenant/Domain, Zeit, Fehlerklassen, `ValidityWindow`, `MemoryType`, serde-stabile Wire-Typen.|
|~~`core`~~|—|Entfällt: Migrationsfassade mit Deprecation-Re-Exports; keine Logik, keine neuen Abhängigkeiten darauf (Gate); `Clock`/`Rng`/`IdGenerator` leben in `ports`.|
|`ports`|K|Alle Trait-Schnittstellen (Storage, Index, Embedder, Kms, Clock, Rng, IdGen, Observer, Merge, Metric, Analyzer, Egress, `Vfs`, `DurableLog`, `KeyCustodian`, `ErasureParticipant`, `ErasureJournal`, `NotaryPort`; Katalog: Anhang A) sowie die gemeinsam genutzten Datentypen mehrerer Ring-0-Kerne (z. B. Shadow-Typen).|
|`mvcc`|K|SequenceLog, SnapshotRegistry, TxBuffer, ReadSet, SSI-Validator.|
|`crypto`|K|AEAD, KDF, Signaturen, `KmsProvider`, `KeyRegistry`, `EpochKeyStore` (In-Memory-Algebra), DeletionProof, `ResidualReport`, AuditChain, RevocationLog (Kette, Signatur, Merkle, kanonische Kodierung), Offline-Verifier. **I/O-frei und lock-frei:** Persistenz läuft über den Port `DurableLog` (Implementierung in `store`). **Nicht-Inhalt:** kein Wissen über KV-Cache-Segmente, Eviction oder andere Domänenstrukturen des Randes (diese liegen in `kvcache` und nutzen nur die Primitive aus `crypto`), keine Threads, keine Hintergrundworker, kein `parking_lot`/`std::sync`-Zustand in Produktionspfaden (Zustand liegt in `store`/`engine` hinter Ports).|
|`vector`|K|HNSW, SQ8, ACL-Bitset, Metric-Trait, Planner für gefilterte Suche. (RaBitQ/DiskANN `[R]`, §9.7)|
|`text`|K|BM25/BM25F, Block-Max-WAND, `Analyzer`-Port, Standard-Analyzer. (`GermanAnalyzer`, Vokabulare `[R]`, §10)|
|`graph`|R|CSR-Graph, PPR, Leiden, Hyperedges, PathRAG, TL-HFD, Edge-Reinforcement (§11).|
|`rank`|R|Kalibrierung (Platt/Isotonic/Conformal), DiBuD, Explain (§12). RRF liegt im Kern (`engine`).|
|`adapt`|R|Deterministische Parameteradaption (ef, Kompaktions-Planer-Eingaben). Bandit-Mathematik gehört zu `router` `[E]`.|
|**`simd`**|K|SIMD-Kernel, Dispatch AVX-512 > AVX2 > NEON > skalar; skalare Referenz als Orakel.|
|**`sys`**|K|OS-Primitive: mmap, madvise, Durability-Primitive, Dateisperren, Speicherdruck, `mem_lock`/`LockedRegions`, Win32-ACL.|
|**`wire`**|K|Zero-Copy-Dekodierung/Layout-Casts für Wire-/Disk-Formate.|

### 6.2 Ring 1 — Persistenz

|Crate|Kl.|Verantwortung|
|---|---|---|
|`store`|K|LSM: WAL, MemTable, SSTables, Blob-Segmente, Manifest, Kompaktion, Block-Cache, CDC. Implementiert `Vfs` (echtes Dateisystem, Durability-Primitive §7.8, DirLock, atomare Writes), `DurableLog`, `ErasureJournal`, den `ErasureKeyStore` und den Rohdatei-Scanner. `FaultVfs` liegt in `testkit`.|
|`checkpoint`|K|Konsistente Snapshots, Backup-Export (ohne Keystore), verifizierte Wiederherstellung (§24.4).|
|`kvcache`|R|Persistenter Inferenz-KV-Cache (§13).|

### 6.3 Ring 2 — Isolation und Inferenz

|Crate|Kl.|Verantwortung|
|---|---|---|
|`sandbox`|R|Wasmtime-Host (§14).|
|`infer-candle`, `infer-ollama`|R|Lokale Embedder/Reranker/Generatoren (§15).|
|`infer-onnx`|E|ONNX-Runtime-Embedder/Reranker.|

### 6.4 Ring 3 — Orchestrierung

|Crate|Kl.|Verantwortung|
|---|---|---|
|`engine`|K|HybridQueryBuilder, Planner, RRF-Fusion, Bounded-Compute-Pool, Async-Wrapper, `as_of`, Chunking, Retention-Reaper.|
|`db`|K|Collections, Transaktionen, Tenant-Scoping. **Kennt keine Inferenz-Backends**; Embedder kommen als Port von der Fassade.|
|`cognition`|R|Wissensmodell, Konsolidierung, Multi-Step-Retrieval, Gedächtnismodell (§17).|
|`privacy`|R|Egress-Schichten, Surrogat-Vault, Exfiltrationsdetektor (§19).|
|`router`, `agent`|E|SLM/LLM-Routing, Agent-Laufzeit (§18, §18a).|

### 6.5 Ring 4 — Fassade und Schnittstellen

|Crate|Kl.|Verantwortung|
|---|---|---|
|`contextra`|K|**Fassade und einzige Composition Root.** Öffentliche Rust-API.|
|`mcp`|K|MCP-Server (stdio, JSON-RPC 2.0), IPC-Proxy, CLI `init`/`doctor`/`verify-proof`/`confirm` (§21).|
|`py`|R|Dünne Python-Bindung ohne eigene Logik.|
|`license`, `audit-export`, `avv-generator`|E|Lizenz-Gating, Audit-Bundles, AVV-Vorlagen (§20.7, §23.5).|

### 6.6 Werkzeug-Crates

`testkit` (Fixtures, Chaos-Harness), `xtask` (schnelle Gates), `xtask-heavy` (Fuzz/Miri/Mutation/Chaos), `benchmarks` (Suite, Baselines inkl. fremder Referenzen). **Anzahl:** 32 Crates unter `crates/` (12 Ring 0 inkl. 3 Inseln, 3 Ring 1, 4 Ring 2, 6 Ring 3, 6 Ring 4, 1 `testkit`), plus `xtask`, `xtask-heavy`, `benchmarks` außerhalb; mit der Kompatibilitätsfassade `core` 33. Maßgeblich ist `capabilities.toml`.

### 6.7 Unsafe-Inseln (genau drei)

`simd`, `sys`, `wire`. (1) Jede Insel exponiert eine **sichere** API; (2) jeder `unsafe`-Block trägt `// SAFETY:` (Plattformcode wie Win32 eingeschlossen); (3) Miri-/Property-Tests gegen eine sichere Referenz; (4) kein anderes Crate enthält `unsafe` im Produktionscode: `#![forbid(unsafe_code)]` in jedem Nicht-Insel-Crate, `unsafe_code = "deny"` workspace-weit, Testtargets nur für Test-Allokatoren/Fehlerinjektion mit lokalem `allow`; (5) Generierter Code folgt P2; (6) das Gate weist generierten und handgeschriebenen `unsafe`-Code getrennt aus und prüft beide; **Dateien dürfen nicht anhand ihres Namens (`test_*.rs`) von der Prüfung ausgenommen werden** (§27.5).

### 6.8 Der Kern ist geschlossen `[K]`

Ein Crate oder Feature betritt den Kern (§6.0) nur per ADR (§28.6) und erst, wenn die Evidenz des bestehenden Kerns die Zielstufe (§27.7) erreicht hat. `[R]`- und `[E]`-Code DARF die Beweislast, die CI-Zeit oder die Angriffsfläche des Kerns nicht vergrößern (§3 Punkt 14). Prüfbar über REQ-ARCH-003 und REQ-ARCH-012.

### 6.9 Crate-Steckbrief (Ziel) `[K]`

Die Tabelle fasst je Crate Ring, Klasse, Unsafe-Status, erlaubte Abhängigkeiten (`may_use`, Obergrenze) und Zielreifegrad zusammen. Sie **leitet** sich aus der Ringregel (P1), der Kernisolation (P30), §6.1 bis §6.7 und dem Prinzip „Fassade verdrahtet, Kern kennt keine Backends“ (§22) ab. **Maßgeblich bleibt `capabilities.toml`** (INV-DOC-SINGLE-SOURCE-1); eine Abweichung zwischen Tabelle und Datei ist eine Abweichung, die im Prüfbericht geführt wird, keine Änderung dieser Spezifikation. „Ring 0 unten“ bedeutet: nur Crates von Ring 0, die in der Zeile nicht ausgeschlossen sind. Jede Kante MUSS azyklisch sein (P5); gleichrangige Kanten gelten nur, soweit hier genannt.

Zielreifegrad: `[K]` ⇒ `stable` (Release 1.0, nur bei erfüllter Bedingung aus §6.0); `[R]` ⇒ `experimental`, bis alle Gates (§27.5) des Crates mit Evidenz der Zielstufe (§27.7) grün sind; `[E]` ⇒ `experimental` ohne Release-Zusage.

|Crate|Ring|Kl.|Unsafe|`may_use` (Obergrenze)|Zielreife|
|---|---|---|---|---|---|
|`types`|0|K|nein|(keine)|stable|
|`ports`|0|K|nein|`types`|stable|
|`mvcc`|0|K|nein|`types`, `ports`|stable|
|`crypto`|0|K|nein|`types`, `ports` (Persistenz nur über den Port `DurableLog`)|stable|
|`vector`|0|K|nein|`types`, `ports`, `simd`|stable|
|`text`|0|K|nein|`types`, `ports`|stable|
|`graph`|0|R|nein|`types`, `ports`|experimental|
|`rank`|0|R|nein|`types`, `ports`|experimental|
|`adapt`|0|R|nein|`types`, `ports`|experimental|
|`simd`|0|K|**ja**|`types`|stable|
|`sys`|0|K|**ja**|`types`|stable|
|`wire`|0|K|**ja**|`types`|stable|
|`store`|1|K|nein|Ring 0 (`types`, `ports`, `crypto`, `mvcc`, `wire`, `sys`)|stable|
|`checkpoint`|1|K|nein|Ring 0, `store`|stable|
|`kvcache`|1|R|nein|Ring 0, `store`|experimental|
|`sandbox`|2|R|nein|`types`, `ports`|experimental|
|`infer-candle`, `infer-ollama`|2|R|nein|`types`, `ports`|experimental|
|`infer-onnx`|2|E|nein|`types`, `ports`|experimental|
|`engine`|3|K|nein|Ring 0 und 1 ohne `[R]`/`[E]`; keine `infer-*`, kein `privacy`|stable|
|`db`|3|K|nein|Ring 0 und 1 ohne `[R]`/`[E]`, `engine`; keine `infer-*`|stable|
|`cognition`|3|R|nein|Ring 0 und 1, `engine`, `db`; Inferenz nur über Port|experimental|
|`privacy`|3|R|nein|`types`, `ports`, `crypto`|experimental|
|`router`, `agent`|3|E|nein|Ring 0 und 1, `engine`, `db`; Inferenz nur über Port|experimental|
|`contextra`|4|K|nein|alle Crates der Release-Konfiguration (einzige Composition Root, REQ-FAC-001)|stable|
|`mcp`|4|K|nein|`contextra`, `types`, `ports` (kein `unsafe`, keine eigene Logik über Protokoll/Pipeline hinaus; Offline-Verifikation nur über den Fassaden-Re-Export `contextra::offline_verify`)|stable|
|`py`|4|R|nein (FFI-Grenze: `panic = "unwind"`, P7)|`contextra`|experimental|
|`license`, `audit-export`, `avv-generator`|4|E|nein|`types`, `crypto`, `ports`|experimental|
|`testkit`|Werkzeug|—|nein|beliebig, **nur** als `dev-dependency`; nie in Release-/Default-Features (INV-EMBED-1)|—|

Hinweis: `mcp` führt `unsafe` nicht einmal als Ausnahme; die Eingaben kommen unvertraut über stdio (§21, REQ-ARCH-004).

### 6.10 Kantenregeln `[K]`

1. **Fundament.** `types`, `ports` sowie die Inseln `wire`, `sys`, `simd` bilden das Fundament; jedes Crate darf sie nutzen, soweit seine `may_use`-Zeile (§6.9) sie nennt. Typen, die mehrere Ring-0-Kerne teilen (z. B. Shadow-Schnittstellen zwischen `graph` und `adapt`), liegen in `ports`, nicht in einem der Kerne.
2. **Kerne untereinander.** Ring-0-Kerne (`vector`, `text`, `graph`, `rank`, `adapt`, `mvcc`, `crypto`) hängen nur dann voneinander ab, wenn die Kante in `may_use` steht und begründet ist.
3. **Ring 2.** Adapter (`infer-*`, `sandbox`) nutzen nur das Fundament; Kanten zwischen Adaptern nur per `may_use` mit Begründung.
4. **Ring 4.** `mcp` und `py` erreichen Ring 2 und 3 ausschließlich über die Fassade `contextra`; kein direkter Zugriff auf `crypto`, `store` oder Inferenz-Backends. Das Lizenz-Gate wird nur über den Port `LicenseGate` genutzt; die konkrete Verdrahtung liegt im Binary bzw. in der Fassade. Die Fassade re-exportiert den Offline-Verifier (`verify_external`, Quelle: `crypto` + `types`) als `contextra::offline_verify`; `mcp` und `py` nutzen für `verify-proof` ausschließlich diesen Re-Export und halten keine eigene Kante auf `crypto`.
5. **Löschtypen** (Beweis, Schicht-Beleg, Messung, Receipt) existieren genau einmal in `crypto`/`ports`; andere Crates re-exportieren sie. Eine zweite Typfamilie für Löschung, Schlüssel oder KV-Segmente ist unzulässig.
6. **Test-Doubles.** Mocks, Fake-Uhr, Fake-Verwahrer, Fehlerinjektion und Platzhalter-Embedder liegen in `testkit` oder hinter dem Feature `testing`; Test-Hooks (Panic-Auslöser, Schlüsselzugänge für Tests) nur hinter `cfg(test)` oder dem Feature `test-hooks`. Beide Features liegen in keinem Release- oder Default-Set.
7. **Aufwärts-Dev-Kanten** (Dev-Abhängigkeit auf höheren Ring) sind nur als deklarierte Ausnahme in `capabilities.toml` zulässig; Ziel ist, sie nach `testkit` oder in `tests/` des höheren Rings zu verlagern.
8. **Fassadenregel für `db`.** `db` kennt keine Inferenz-Backends; Embedder, Reranker und Generator kommen als Ports von der Fassade.
9. **Port-Besitz.** Jeder Port aus Anhang A (A.1 und A.2) ist in `ports` definiert; ein Trait mit Port-Rolle (Austauschpunkt für Fassade oder Test) außerhalb von `ports` ist unzulässig. Crate-interne Hilfs-Traits ohne Austauschrolle sind davon ausgenommen und im Gate als „intern“ klassifiziert.
10. **Einzel-Besitz der Standardfusion.** RRF-Fusion (§16) und die Basis-Erklärung (Modalität, Score, WAL-Seq) liegen ausschließlich in `engine`; `rank` darf sie ergänzen, aber keine zweite Standardfusion enthalten.

### 6.11 Rand-Brücken: wie `[K]`-Crates `[R]`-Fähigkeiten nutzen `[K]`

Kern-Crates erreichen Rand-Fähigkeiten ausschließlich über Ports, die die Fassade bei aktiviertem Feature injiziert (P30). Fehlt die Fähigkeit, verhält sich der Kern wie in der Spalte „Ohne Rand“ beschrieben; er startet und besteht seine Tests auch dann.

|Fähigkeit|Port (Anhang A)|Anbieter|Ohne Rand|
|---|---|---|---|
|Graph-Modalität|`GraphIndex`|`graph`|Planner lässt die Modalität aus; das Ergebnis weist `modalities_unavailable` aus|
|Reranking|`Reranker`|`infer-*`|entfällt; RRF-Reihenfolge gilt|
|Embeddings|`EmbeddingProvider`|`infer-*`|`EmbeddingBackend = None`: nur Textsuche (§15.3)|
|KV-Prefix-Cache|`KvPrefixStore`|`kvcache`|inaktiv (kein stilles Fallback); Löschschicht `KvCache` ist `FeatureDisabled` (§20.4a)|
|Adaptive Regelung (ef, Latenz)|`AdaptiveController`|`adapt`|feste Startwerte aus §25|
|Kalibrierung, DiBuD|`Calibrator`|`rank`|rohe RRF-Scores, keine „Unsicher“-Ausgabe|
|Egress, Erkennung, Injektion|`EgressChannel`, `EntityRecognizer`, `InjectionDetector`|`privacy`|Egress gesperrt (INV-WIRING-2)|
|WASM-Hooks|`ModuleVerifier`, `FilterModule`|`sandbox`|nicht registrierbar|
|Lizenzprüfung|`LicenseGate`|`license`|keine Prüfung; der Kern kennt kein Lizenzkonzept|
|Multi-Step-Retrieval|`QueryRewriter`|`cognition`|Single-Step-Planner|

Regeln: (1) **Modul-Entfernung:** Der Kern baut und testet ohne jedes `[R]`/`[E]`-Crate (REQ-ARCH-024). (2) Die Fassade hängt von Rand-Crates nur über optionale Features ab, die im Release-Default aus sind (REQ-ARCH-012). (3) Typen, die Kern und Rand teilen (Ergebnis-, Signal-, Graph-ID-Typen), liegen in `types` oder `ports`, nie doppelt in beiden Seiten (§6.10 Regel 1). (4) Ein Kern-Crate, das ein Rand-Crate als Abhängigkeit führt, verletzt INV-CORE-ISOLATION-1 auch dann, wenn die Kante nur für eine Teilfunktion gebraucht wird.

---

## 7. Storage-Engine (`contextra-store`) `[K]`

### 7.1 Datenpfad

`put/delete/merge` → TxBuffer → **WAL-Batch (Group Commit, ein Commit-Punkt)** → MemTable → Flush → SSTable (L0) → Kompaktion (L1…Ln). Werte ≥ `blob_threshold` (Start 64 KiB) gehen als **Blob-Segment** (§7.9) statt durch WAL/LSM-Wertpfad. Lesen: MemTable → immutable MemTables → SSTables (Bloom-gefiltert) mit Block-Cache. Alle Werte sind Dokument-Records (eine Verschlüsselungseinheit, §20.2).

### 7.2 Dateiformate

|Format|Magic|Inhalt|
|---|---|---|
|WAL|`MFW3` (Ops `IndexIntent`/`IndexApplied`)|Records: Sequenznummer, Länge, `scope_id`, `key_id`, Nonce, AEAD-Ciphertext, **HMAC-SHA256-Kette** (jedes Record bindet den Vorgänger-Tag). Segmente vorab reserviert.|
|SSTable|`MFSX`|Datenblöcke, Index-Block, Bloom-Filter, Footer mit Prüfsumme/Tag, AEAD je Block (Engine-Schlüssel); Werte darin je Record unter Scope-DEK.|
|Blob-Segment|`MFBL`|Append-only Werte, Kopf mit Digest; je Segment eigener Schlüsselbezug (§7.9).|
|Manifest|`MFMN`|Versionierte Liste der Live-Dateien je Level, **High-Water-Mark** (höchste bestätigte WAL-Sequenz) gegen Truncation.|
|Keystore|`MFKS`|`KdfHeader` (eigene Kennung `MFKD`), gewrappte Wurzelschlüssel; bei Betriebsart K0 der Integritätswurzelschlüssel im Klartext (§20.2c).|
|Sidecar-Logs|`MFKR`, `MFRV`, `MFAU`, `MFEJ`, `MFEK`, `MFES`|`KeyRegistry`, `RevocationLog`, `AuditChain`, `ErasureJournal`, `ErasureKeyStore`, `ErasedSet` (§7.11); jede Datei mit Magic, Formatversion und HMAC-Kette.|

**Formatversionierung (INV-FORMAT-1):** Magic **und** Formatversion in jeder Datei; neuere als unterstützte Version ⇒ Öffnen verweigert (kein stilles Herabstufen); Downgrade-Angriffe (Ersetzen durch ältere Version) werden abgewiesen. Ältere Versionen öffnet nur ein expliziter, getesteter Migrationspfad: `contextra migrate` erzeugt zuvor ein verifiziertes Backup und bietet `--dry-run`. Pro Release liegt ein **Kompatibilitätskorpus** (Golden-Files je Format und Version) im Repository; die CI öffnet ihn mit dem aktuellen Code. Formate sind **explizit** spezifiziert (Feldreihenfolge, Endianness, Längenpräfixe), nicht aus `serde`-Derives oder Serialisierungsbibliotheken mit versionsabhängigem Layout abgeleitet.

### 7.3 WAL-Garantien

- **Commit-Punkt = durable Flush des Batches** (Primitive je Plattform, §7.8). Ack erst danach.
- **Group Commit:** Mehrere Transaktionen teilen sich einen Flush; Latenz/Durchsatz über `PerformanceProfile`.
- **Kettenprüfung beim Recovery:** Bruch der HMAC-Kette ⇒ Abbruch am letzten gültigen Record (torn tail wird abgeschnitten, Korruption in der Mitte ⇒ fail-closed).
- **Anti-Truncation:** Manifest-High-Water-Mark > WAL-Ende ⇒ Manipulation/Datenverlust wird erkannt, nicht verschwiegen.
- **Index-Atomarität (INV-ATOMIC-IDX-1):** Ein Batch enthält Daten-Ops **und** `IndexIntent{index_id, op}`. `IndexApplied` ist ein **nicht-durabler Hinweis** (kein eigener Flush): Recovery wiederholt Intents idempotent, daher kostet ein Commit genau einen Flush.
- **CDC:** `WalObserver` erhält geordnete, deterministische Ereignisse mit dauerhaftem Cursor (at-least-once, idempotent konsumierbar). **Ereignisse tragen keine Nutzinhalte** (nur `op`, `scope_id`, opake IDs, `seq`), sodass Observer keine Löschkopien halten.
- **fsync-Fehler (INV-FSYNC-1)** und **volles Medium (INV-ENOSPC-1)** wie in §5. Backpressure: erreicht der Rückstau (L0-Dateien, MemTable-Bytes) die Schwellen aus §7.4, werden Schreiber gedrosselt statt Speicher unbegrenzt zu belegen.

### 7.3.1 Recovery-Algorithmus

1. DirLock erwerben. Kann das Dateisystem keine exklusive Sperre garantieren (Netzwerk-/Cloud-Sync-/unbekannte FUSE-Dateisysteme), wird das Öffnen im persistenten Betrieb verweigert; nur `MemoryOnly` läuft ohne Sperre. 2. Keystore und Manifest laden und prüfen. 3. WAL ab letztem Checkpoint ketten-validieren. 4. Records in MemTable replayen; offene IndexIntents vom Applier idempotent anwenden (INV-INDEX-APPLY-1); Embedding-/LLM-Werte stammen aus dem WAL (INV-NONDET-1). 5. High-Water-Mark abgleichen. 6. Sidecar-Logs (§7.11) validieren und gegen ihre Manifest-Marken abgleichen; ausstehende `SidecarIntent`s aus dem WAL idempotent anwenden. 7. `ErasedSet` laden und die Lesesperre armieren (INV-BOOT-1); `ErasureJournal` replayen und offene Löschsagas zur Fortsetzung vormerken (§20.10). 8. Retention-Entscheidungen (WAL-Records, §16.4) in ihren `PendingDeletion`-Zustand überführen. 9. Erst dann Lese-/Schreibverkehr zulassen; Lesen eines durch den `ErasedSet` gesperrten Betroffenen bleibt auch danach verweigert. Poison-Locks werden erkannt und über definierte Wiederherstellung behandelt. Die Checkpoint-Frequenz begrenzt die Recovery-Zeit auf ein je Tier festgelegtes Budget (Messgröße in §26).

### 7.4 Kompaktion

- **Tiered** und **adaptiv** (`CostBasedAdaptivePlanner`): Planer minimiert Write-/Read-/Space-Amplification anhand gemessener Workload-Statistiken.
- Tombstones werden erst auf dem untersten Level bzw. nach Snapshot-Horizont verworfen (MVCC-sicher).
- **Löschstufe S2** erzwingt Purge-Kompaktion betroffener SSTables (§20.3).
- Kompaktion ist budgetiert (I/O-Rate, CPU) und vom `SystemPressureMonitor` drosselbar.
- **Write-Stall-Policy:** Ab `l0_slowdown` (Start 8) werden Schreiber gedrosselt; ab `l0_stop` (Start 20) warten sie höchstens `write_stall_timeout` (Start 5 s) und erhalten danach den retrybaren Fehler `Busy`. Rückstau ist **kein Betriebszustand** (§7.10): Er ändert weder Gesundheitszustand noch Garantien. Werte sind zu kalibrieren und per Benchmark belegt.

### 7.5 MergeOperator (IoC auf Schreibsemantik)

Port `MergeOperator { full_merge, partial_merge }`. Zwei Implementierungsarten: **Rust** (statisch, generisch) und **WASM** (Sandbox). Verbindliche **CRDT-Gesetze** (Assoziativität, Kommutativität, Idempotenz soweit zutreffend), property-getestet für die mitgelieferten Operatoren **LWW-Register**, **G-Counter**, **OR-Set**. Ein WASM-Operator `[R]` wird nur registriert, wenn er die Gesetz-Testsuite besteht und die Determinismus-Konfiguration aus §14.3 erfüllt. Operatoren sind deterministisch (P28).

### 7.6 Caches und Druck

- Block-Cache: ein Standard (**SIEVE**) hinter dem Port `BlockCache`; Alternativen sind Port-Implementierungen, keine Pflicht.
- `SystemPressureMonitor` (über `sys`): bei Speicherdruck werden Caches verkleinert, Kompaktion gedrosselt, KV-Cache evictiert — kontrolliert, nie OOM-Kill.
- Bloom-Filter pro SSTable (konfigurierbare FPR); Prefix-Bloom für Range-Scans.

### 7.7 I/O-Regeln (INV-IO-1)

- **Standard:** SSTable-/WAL-/Blob-Lesen per `pread` mit Prüfsumme; entschlüsselte Blöcke leben als `Bytes` im Block-Cache (Zero-Copy ab dort).
- **`mmap`** nur für unveränderliche, vollständig prüfsummenverifizierte Dateien auf lokalem Volume (`doctor` prüft Dateisystemtyp); Verkürzen/Überschreiben durch Fremdprozesse wird durch DirLock und Rechte ausgeschlossen. Medienfehler bei gemappten Dateien (SIGBUS) sind durch diese Auswahl auf verifizierte, lokale Dateien begrenzt und im Betriebshandbuch deklariert; Pfade ohne diese Voraussetzung nutzen `pread`.
- Verschlüsselte Dateien werden nicht zero-copy gemappt (INV-ENCRYPT-REST-2).

### 7.8 Durability-Primitive je Plattform (INV-DURABILITY-PRIM-1)

|Plattform|Datei-Flush|Verzeichniseintrag (Create/Rename)|
|---|---|---|
|Linux|`fdatasync` (Metadaten ändernd: `fsync`)|`fsync` auf das Elternverzeichnis|
|macOS|`fcntl(F_FULLFSYNC)` (reines `fsync` garantiert keine Durabilität)|`F_FULLFSYNC` auf das Verzeichnis|
|Windows|`FlushFileBuffers`|`MoveFileEx(MOVEFILE_WRITE_THROUGH)`/`ReplaceFile`|

Jede neue oder umbenannte Datei (WAL-Segment, SSTable, Manifest-Wechsel) wird erst nach Flush des Elternverzeichnisses als existent behandelt. Sämtlicher Dateizugriff läuft über den Port `Vfs` (Ring 0 führt keinen direkten Dateizugriff aus); die Implementierung liegt in `store`, die Fehlerinjektion `FaultVfs` in `testkit`, und beide bilden diese Primitive ab.

### 7.9 Blob-Segmente

Große Werte (KV-Blöcke, große Dokumente) liegen in append-only Segmenten; LSM/WAL halten nur Zeiger + BLAKE3-Digest. **Reihenfolge:** Segment durable flushen → WAL-Commit mit Zeiger → Ack. Orphan-Segmente (Crash dazwischen) räumt Recovery per Manifest ab. Jedes Segment trägt einen Schlüsselbezug auf eine Scope-DEK (§20.2); Segmentlöschung ist Dateilöschung plus Schlüsselvernichtung.

### 7.10 Betriebszustände und Gesundheitsübergänge `[K]`

|Zustand|Bedeutung|Eintritt|Verlassen|
|---|---|---|---|
|`Healthy`|Alle Garantien aktiv|nach erfolgreicher Recovery|—|
|`Degraded`|Ein nicht-kritischer Teil ist ausgefallen (z. B. Block-Cache, `[R]`-Feature, Hintergrundkompaktion); Kern-Garantien bleiben|Ausfall einer abgeleiteten Komponente|Komponente gesund oder Neustart|
|`ReadOnly`|Lesen erlaubt, Schreiben abgelehnt (retrybarer Fehler); Löschen über die Notreserve zulässig (§7.12)|`ENOSPC` bei Rotation (INV-ENOSPC-1), `fsync`-Fehler auf einem Sidecar-Handle (§7.12), Lock-Poisoning in Schreibstrukturen mit sicherer Wiederherstellung|Platz frei / Neustart|
|`Poisoned`|Durabilität unbestimmt: kein Ack, kein Schreiben|`fsync`-/`fdatasync`-Fehler (INV-FSYNC-1), unwiederherstellbarer Poison-Lock|ausschließlich Neustart mit Recovery-Replay|

Regeln: (1) Übergänge sind monoton in Richtung `Poisoned`, außer durch Neustart bzw. dokumentierte Selbstheilung `Degraded→Healthy`. (2) Jeder Übergang wird auditiert (ohne Nutzinhalte, INV-LOG-1) und über `doctor` sichtbar. (3) Ein Poisoned-Lock (`Mutex`/`RwLock`) ist ein typisierter Fehler mit definiertem Übergang, nie `unwrap` (P7). (4) Unbekannte Zustände ⇒ fail-closed (INV-FAILCLOSED-1). Rückstau (§7.4) ist kein Zustand, sondern `Busy` am Schreibpfad. Poison-Umfang und Notlöschpfad: §7.12. Prüfung: REQ-DUR-014 (C, I).

### 7.11 Sidecar-Logs: die einzigen Persistenzdomänen neben dem WAL `[K]`

Der WAL ist die einzige Wahrheit für Anwendungszustand (P24). Sicherheitszustand kann nicht aus dem WAL abgeleitet werden, weil S3 WAL-Segmente scrubbt, und muss Löschungen und Schlüsselvernichtungen **überleben**. Er liegt deshalb in genau diesen Sidecar-Logs; die Liste ist abschließend (INV-SIDECAR-1).

|Sidecar|Magic|Inhalt|Primärer Schreiber|Kettenschlüssel (§20.2c)|Im Backup|
|---|---|---|---|---|---|
|`KeyRegistry`|`MFKR`|Schlüsselereignisse: Erzeugen, Wrap, Rotation, Vernichtung|`KmsProvider` über `KeyManager`|`K_sc/keyreg`|ja (nur Ereignisse, keine Schlüssel)|
|`RevocationLog`|`MFRV`|Widerruf von Schlüsseln, Modulen, Berechtigungen|Autorisierung, Löschsaga|`K_sc/revoke`|ja|
|`AuditChain`|`MFAU`|Zugriffe, Löschungen, Egress, Freigaben; tenant-getrennt|Applier (aus `SidecarIntent`) und Sicherheitspfade|`K_sc/audit`|ja|
|`ErasureJournal`|`MFEJ`|Saga-Übergänge (§20.10)|Orchestrator|`K_sc/journal`|nein (Betriebszustand)|
|`ErasureKeyStore`|`MFEK`|gewrappte `DocDek`/`SegKey`/`EpochKek`-Verweise (§20.2b)|`KeyCustodian`-Adapter|`K_sc/keystore`|**nein** (Schlüsselmaterial)|
|`ErasedSet`|`MFES`|Commitments gelöschter/gesperrter Betroffener (§20.11)|Orchestrator|`K_sc/erased`|**nein** (siehe unten)|
|Keystore|`MFKS`|`KdfHeader`, gewrappte Wurzelschlüssel|`KmsProvider` (lokal)|(Integritätswurzel)|**nein** (Schlüsselmaterial)|

Regeln:

1. **Form.** Jedes Sidecar ist append-only, trägt Magic und Formatversion (INV-FORMAT-1), eine HMAC-Kette mit eigenem Zweckschlüssel und einen `fsync` je Eintrag über den Port `DurableLog`. Die Manifest-Marke (Name, letzte Sequenz, Kettenkopf) wird bei jedem Checkpoint und bei jeder Rotation aktualisiert; sie erkennt Tail-Truncation und Ersetzen durch eine ältere Version (§20.6). Das Löschen eines Sidecars bei vorhandenen Daten ist Manipulation (fail-closed).
2. **Intent-Muster (WAL zuerst).** Wirkungen, die ein WAL-Commit auslöst (Audit-Einträge zu Datenoperationen, Lineage-Hinweise), stehen als `SidecarIntent{log, payload_hash}` im selben WAL-Batch (wie `IndexIntent`, INV-ATOMIC-IDX-1). Der Applier schreibt sie nach dem Commit idempotent über `(log, seq)`; Recovery wiederholt offene Intents (§7.3.1 Schritt 6). Ein Commit kostet weiterhin genau einen Flush.
3. **Sidecar vor Wirkung.** Sicherheitskritische Wirkungen, die nicht aus einem WAL-Commit folgen, werden zuerst im Sidecar durabel und erst danach wirksam: Autorisierungswiderruf (`RevocationLog`), Saga-Übergang (`ErasureJournal` vor Ausführung des Schritts), Schlüsselvernichtung (INV-SHRED-1: Vernichtung der Schlüssel **und** `RevocationLog`-Eintrag durabel, dann `DeletionReceipt`). Stürzt der Prozess zwischen Vernichtung und Log-Eintrag ab, setzt Recovery anhand des Journals die idempotente Vernichtung beim Verwahrer fort und trägt den Log-Eintrag nach; ein Receipt entsteht erst danach.
4. **Autorität.** Sidecars sind nicht aus dem WAL rekonstruierbar und deshalb autoritativ; ein beschädigtes Sidecar in der Mitte der Kette ist fail-closed (Start verweigert, `doctor` benennt Datei und Sequenz).
5. **Backup.** Checkpoints enthalten `KeyRegistry`, `RevocationLog` und `AuditChain`, aber weder `MFKS` noch `MFEK` (§24.4). Der `ErasedSet` gehört dem laufenden System: Der Restore-Guard liest den `ErasedSet` des **Zielsystems**; ein Backup, das in ein System ohne `ErasedSet` und ohne Keystore eingespielt wird, ist ohne die zugehörigen Schlüssel ohnehin unlesbar. Nach SK vernichtetes Material bleibt damit auch in älteren Backups unlesbar.
6. **Ausfall.** Ein `fsync`-Fehler auf einem Sidecar-Handle vergiftet genau diesen Handle (INV-FSYNC-1) und führt gemäß §7.12 zu `ReadOnly` bzw. `Poisoned`.

### 7.12 Notlöschpfad und Poison-Umfang `[K]`

Löschen ist eine Schutzfunktion und darf nicht an Betriebszuständen scheitern, die Löschen selbst auslösen (volles Medium, `fsync`-Fehler).

|Ereignis|Zustand|Löschen erlaubt?|
|---|---|---|
|`ENOSPC` bei WAL-Rotation|`ReadOnly`|**Ja.** Tombstone-Batch, Journal-Einträge und `ErasedSet`-Eintrag nutzen die Notreserve (`emergency_reserve_bytes`, Start 64 MiB, mindestens zwei WAL-Segmente plus Journal, bei Start vorab reserviert, §25.2). SK braucht keinen Speicher. S2 (Purge-Kompaktion) braucht Platz: Schlägt sie mit `StorageFull` fehl, bleibt die Saga `Partial{outstanding: S2}` und setzt nach Freigabe fort; der Beweis weist das aus.|
|`fsync`-Fehler am WAL-Handle|`Poisoned`|**Nur Shred-only:** Schlüsselvernichtung beim Verwahrer, `RevocationLog`, `ErasedSet` und `ErasureJournal` über ihre **eigenen, gesunden** Handles; keine WAL-Schreibzugriffe. Der Tombstone-Batch wird nach dem Neustart von der Saga aus dem Journal nachgeholt.|
|`fsync`-Fehler an einem Sidecar-Handle|`ReadOnly` (Funktion fail-closed)|Betroffen ist genau die abhängige Funktion: `RevocationLog` ⇒ keine Autorisierung neuer Schreibvorgänge; `ErasureJournal` ⇒ keine neue Saga, aber Lesesperre im RAM; `KeyRegistry`/`ErasureKeyStore` ⇒ keine Schlüsselerzeugung.|
|Ausfall aller Löschpfad-Handles|`Poisoned`|Prozesslokale Lesesperre für den angefragten Betroffenen (INV-EMERGENCY-DEL-1); Retry nach Neustart.|

Leseverkehr im Zustand `Poisoned` ist nur zulässig, wenn der `ErasedSet` armiert ist (INV-BOOT-1); andernfalls verweigert der Zustand auch das Lesen (fail-closed). Der Poison-Umfang ist pro Persistenzdomäne definiert und monoton wie in §7.10.

---

## 8. MVCC und Serializable Snapshot Isolation (`contextra-mvcc`) `[K]`

- **SequenceLog:** monoton wachsende, WAL-gebundene Commit-Sequenz.
- **SnapshotRegistry:** aktive Snapshots; bestimmt GC-/Tombstone-Horizont. Ein Snapshot ist **immer ein konkreter Sequenzstand, nie `u64::MAX`**.
- **TxBuffer:** privater Schreibpuffer, Read-your-writes.
- **ReadSet:** erfasst Punkt-, **Prefix-/Range-Lesezugriffe (Phantomschutz)**; SSI-Validierung bei Commit erkennt gefährliche Strukturen (rw-Antidependenzen) ⇒ retrybarer `SerializationFailure`.
- **Tenant:** aus der Transaktion abgeleitet; Schlüsselkonstruktion ohne Tenant-Kontext ist typbedingt unmöglich.
- **Kein Saga-/Kompensationsmodell.** Der Normalfall ist atomar über den WAL-Batch; Fehler nach dem Commit-Punkt werden **vorwärts** behoben (Recovery wendet Intents erneut an), nie kompensiert.
- **Zeitreisen:** `as_of(seq | zeit)` liest historische Snapshots, solange nicht kompaktiert/gelöscht. **Löschung wirkt rückwirkend (INV-ASOF-ERASE-1):** Ein Betroffener, der im `ErasedSet` steht oder `Frozen` ist, ist auch in jedem historischen Snapshot unlesbar, selbst wenn S2/S3 noch `Pending` sind und die Altversion physisch existiert. Ein Snapshot hat das Höchstalter `snapshot_max_age` (Start 15 min, §25.4); danach wird er als `SnapshotExpired` (final) ungültig, nicht stillschweigend verlängert. Das begrenzt, wie lange offene Snapshots S2 verzögern können (REQ-DEL-031).

**Isolation je Lesetyp:**

|Lesetyp|Isolation|
|---|---|
|Punkt-/Prefix-/Range-Reads auf dem Keyspace|SSI (Phantomschutz)|
|ANN-, BM25-, Graph-Suchen|Snapshot-Isolation via `born_seq`/`died_seq` (INV-INDEX-SNAPSHOT-1); approximative Suchen haben keinen definierbaren Phantomschutz|
|Optional `serializable_index_reads = true`|Index-Lesen registriert einen Collection-Marker im ReadSet; jeder konkurrierende Schreiber derselben Collection löst Konflikt aus (konservativ, kann Abbrüche erhöhen)|

### 8.1 Sperrenordnung (INV-LOCK-ORDER-1)

Alle Sperren der Engine (`collections`, Key-Sperren, Embedder, Hooks, Konfiguration, Uhr, Metriken, Konsolidierungs-Guard) stehen in **einer** vollständigen, geordneten Hierarchie; nur diese Ordnung gilt beim Erwerb mehrerer Sperren. Mehrschlüssel-Operationen sperren Shards in aufsteigender Reihenfolge und deduplizieren sie; Shard-Hasher nutzen feste Seeds (deterministische Verteilung). Der Sperrtyp je Ebene (Mutex oder RwLock) ist Teil der Dokumentation und durch einen Test festgeschrieben. Guards von `std`/`parking_lot` DÜRFEN NICHT über `.await` gehalten werden (`await_holding_lock = deny`). Für jede nebenläufige Kernstruktur (SnapshotRegistry, TxBuffer, RevocationLog, HNSW-Arena, Group-Commit, Key-Sperren) existiert ein loom-Modell (§27.1).

---

## 9. Vektor-Subsystem (`contextra-vector`) `[K]`

### 9.1 HNSW

Arena-Allokation, Backlinks, **adaptives ef**, Startwerte **m=16, efConstruction=200, efSearch=64**, `MAX_SEARCH_K=1000` (zu kalibrieren). Persistenz: WAL (IndexIntent) + Snapshot der Graphstruktur; Wiederaufbau deterministisch (INV-INDEX-APPLY-1: ein Applier in WAL-Reihenfolge, Level aus `hash(doc_id, seed)`).

### 9.2 Mandantentrennung (INV-TENANT-IDX-1)

Je **(Tenant, Collection, Embedding-Modell-Hash)** existiert ein eigener Index (eigene Datei, eigener Schlüssel, §20.2). Ein gemeinsamer Graph über Mandanten ist unzulässig (Kanten wären mandantenübergreifende Datenflüsse, VETO-F10). Der ACL-Bitset (§9.4) wirkt **innerhalb** eines Tenants (Rolle/Zeile). Konsequenzen: `drop_collection` ist Dateilöschung + Schlüsselvernichtung; Suchlatenz und Recall eines Mandanten hängen nicht von anderen ab.

### 9.3 Löschung (VETO-F02) und Slot-Wiederverwendung

Nur **Tombstone + Pruning**; Knoten werden nicht „weggeflickt“. Löschstufe S1: der Vektor-Payload (Vollpräzision und SQ8-Code) des Knotens wird **sofort im Slot genullt** und der Index-Snapshot neu geschrieben (Routing-Knoten bleibt, kein Rewiring); Pruning entfernt tote Kanten beim Rebuild. Übersteigt der Tombstone-Anteil `tombstone_rebuild_ratio` (Start 0,2), läuft ein deterministischer Rebuild im Hintergrund.

**INV-HNSW-SLOT-1:** Ein Arena-Slot wird erst wiederverwendet, wenn keine eingehende Kante mehr auf ihn zeigt, oder jede Kante trägt eine Generationsmarkierung. Wird der Einstiegsknoten gelöscht, erfolgt die Neuwahl deterministisch.

### 9.4 Gefilterte Suche, ACL und Planner

- **Planner:** Selektivität `s = |erlaubt| / |Index|`. Ist `|erlaubt| ≤ exact_scan_max` (Start 20 000), läuft ein **exakter SIMD-Scan** über die erlaubten IDs (kein Graph, kein Recall-Verlust). Sonst **ACORN-artige gefilterte Traversierung** (Prädikat während der Traversierung, erweiterte Nachbarschaft, ef skaliert mit 1/s bis zu einer Obergrenze).
- **ACL-Bitset (RLS):** Pro Rolle ein Bitset berechtigter Vektor-IDs; **Prüfung vor Distanzberechnung**. Modus `strict`: nur berechtigte Knoten werden bewertet *und* expandiert; fällt dadurch die erreichbare Menge unter `k`, greift der exakte Scan statt einer stillen Unterlieferung.
- Seitenkanäle: Ergebnisse enthalten keine Scores oder Counts nicht berechtigter Knoten; Latenz wird in Klassen gemessen (§26), nicht als konstant zugesichert.

### 9.5 Metriken (IoC)

Generischer `Metric`-Trait mit Zero-Sized-Types (`Cosine`, `L2`, `InnerProduct`, `Hamming` …) ⇒ **monomorphisierter Hot-Path ohne dyn-Dispatch**. **WASM-Metriken** (domänenspezifische Ähnlichkeit) dürfen **ausschließlich im Top-k-Rerank** laufen, nie in der Traversierung.

### 9.6 Quantisierung

**SQ8** (Skalar) ist die RAM-Vorstufe; die besten `ef` Kandidaten werden mit den Originalvektoren **exakt nachgerankt**. SIMD-Dispatch AVX-512 > AVX2 > NEON > skalar; skalar ist Orakel (Bit-/ULP-Toleranz getestet). Das Ergebnis hängt nie vom Cache-Zustand ab. Aussagen zu RAM-Bedarf folgen der Formel `N·D` Byte (SQ8) + Graph (`N·R·4` Byte); Beispiel 1 Mio. × 1536: ≈ 1,5 GB SQ8 + Graph statt ≈ 6,1 GB fp32.

### 9.7 Skalierung über RAM hinaus `[R]`

RaBitQ (1-Bit-Codes mit Fehlerschranke, ≈ 0,19 GB je 1 Mio. × 1536) und ein **DiskANN-artiger Modus** (SSD-residente Vollpräzisionsvektoren per `mmap_readonly`, Codes + Adjazenz im RAM) sind `[R]` und erst relevant, wenn der Workload den Designpunkt (§1) überschreitet. Vorbedingungen: INV-ENCRYPT-REST-2, deterministisches Ergebnis unabhängig vom Hot-Set, `MAX_SEARCH_K` auch im Filterpfad, Filterfaktor `gamma = 0` ⇒ Ergebnis identisch zum ungefilterten Pfad, Tie-Breaker Score absteigend/DocId aufsteigend. Nur die **Platzierung** der Daten ändert sich, kein Rewiring (VETO-F02).

### 9.8 Adaptiver Overfetch

Liefert ein selektiver Filter nach Anwendung weniger als `k` Treffer, MUSS das System in deterministischen Stufen nachladen: `3·k → 6·k → 12·k`, bis entweder `k` Treffer erreicht sind oder eine harte Scan-Obergrenze greift. Abbruch erfolgt ausschließlich über deterministische Zähler, nie über Zeit-/Wall-Clock-Heuristiken (Konsistenz mit P28). **Abnahmekriterium:** Ein Filter mit 1 % Selektivität liefert entweder exakt `k` Treffer oder nachweislich alle im Snapshot vorhandenen passenden Treffer (keine stille Unterlieferung).

---

## 10. Text-Subsystem (`contextra-text`) `[K]`

- **BM25** (Startwerte k1=1.5, b=0.75, per Recall-Benchmark zu kalibrieren), **BM25F** mit Feldgewichten, **Block-Max-WAND**. **Statistiken (IDF, Dokumentlängen) werden je Tenant geführt** (INV-TENANT-IDX-1).
- **`Analyzer`-Port:** Unicode-Normalisierung (NFKC), Kleinschreibung, Standard-Tokenisierung. `GermanAnalyzer` `[R]`: Komposita-Zerlegung (Wörterbuch + Fugen-Heuristik), Lemmatisierung, Umlaute/ß.
- **Fachvokabulare** `[R]` (Recht, Medizin) als versionierte Ressourcenpakete: Synonyme, Abkürzungen, Normbezüge, geschützte Begriffe (nie Stoppwörter). Qualität belegt ein Recall-Benchmark je Paket, nicht eine Eintragszahl.
- Invertierter Index segmentiert, über WAL-IndexIntent konsistent, je Tenant/Collection getrennt; Löschung per Segment-Rewrite (S1); Segmentdateien liegen unter der Scope-DEK.
- **SPLADE** (gelerntes Sparse-Signal) `[E]`: ergänzend zu BM25, Feature-Flag, braucht stabiles ONNX-Backend.

### 10.3 Dokumentformat-Extraktoren `[R]`

PDF-, E-Mail- und DOCX-Ingestion MÜSSEN, sobald aktiviert, zwingend mit Zip-Bomb-Schutz und Größenlimits abgesichert sein und in einem eigenen Adapter-Crate außerhalb des Kerns (Ring 4, `[R]`) laufen; Formatparser in Ring 0–3 sind verboten. Jeder Extraktor ist Fuzz-Ziel (§27.1). Bis zur Aktivierung dieses Adapters bleibt Ingestion normativ auf Text/Markdown beschränkt.

---

## 11. Graph-Subsystem (`contextra-graph`) `[R]`

- **CSR**-Repräsentation (read-optimiert) + Delta-Log für Updates.
- **Personalized PageRank**, **Leiden**-Community-Detection, **Hyperedges** (n-äre Beziehungen), **PathRAG** (Pfadbasiertes Retrieval mit Pruning), **TL-HFD/APPRH** (Hierarchische Diffusion), **Edge-Reinforcement** (Kantengewichte wachsen durch bestätigte Nutzung, mit Decay).
- Bi-temporale Kanten (`ValidityWindow`), tenant-isoliert.

### 11.1 Budgetierte k-Pfad-Diffusion (PathRAG-Innenleben)

Innerhalb von PathRAG MUSS eine **hart knotenbudgetierte, bidirektionale Dijkstra-Pfadsuche** verfügbar sein, die gezielt Speicherexplosion beim Traversieren von Hub-Entitäten mit hohem Ausgangsgrad verhindert. Das Budget begrenzt die Anzahl gleichzeitig offen gehaltener Suchknoten unabhängig von der tatsächlichen Graphdichte; eine Überschreitung führt zu kontrolliertem Abbruch mit Teilresultat, nie zu unbegrenztem Speicherwachstum.

### 11.2 Lovász-Extension für TL-HFD (submodulare Optimierung)

Die hierarchische Diffusion (TL-HFD/APPRH) MUSS auf einer **Lovász-Extension** mit Subgradienten-Berechnung für Hyperkanten aufsetzen (submodulare Optimierungstechnik). Eine `truncate`-Operation begrenzt die Hyperkanten-Teilnehmer auf die Top-x-Werte plus mindestens einen Minimalwert-Teilnehmer, um Degenerierung (Ausschluss aller schwach gewichteten Teilnehmer) zu vermeiden.

Der Graph ist je (Tenant, Collection) getrennt (INV-TENANT-IDX-1).

---

## 12. Fusion und Ranking (`contextra-rank`) `[R]`

RRF (k=60) liegt als Standardfusion im Kern (`engine`, §16). `rank` ergänzt:

- **Kalibrierung:** Platt-Scaling, Isotonic Regression, **Conformal Prediction** (Abdeckungsgarantie, ermöglicht „Unsicher“-Ausgabe).
- **DiBuD** (Diversität unter Budget) gegen redundante Treffer.
- **Explain:** Jeder Treffer liefert Begründung (welche Modalität, welche Scores, welche Quelle/WAL-Seq) – Basis der Provenienz (M4).

---

## 13. Persistenter KV-Cache (`contextra-kvcache`) `[R]` — Mission M1

- **Adressierung:** `kv/<tenant>/<model-hash>/<prefix-hash>/<block-nr>`; `model-hash` bindet Gewichte + Tokenizer + Quantisierung + KV-Layout (nie über Modellwechsel gültig).
- **Prefix-Radix-Tree:** längster gemeinsamer Token-Prefix; nur der Suffix wird neu berechnet. **Blöcke:** 16 Token, unabhängig adressierbar, Copy-on-Write innerhalb eines Tenants.
- **Speicherung:** KV-Tensoren liegen in **Blob-Segmenten** (§7.9), nie im WAL- oder LSM-Wertpfad; der LSM hält je Block `(Segment, Offset, Länge, BLAKE3)`. Reihenfolge: Segment-Flush → Metadaten-Commit. Größenordnung (fp16, 32 Layer, 8 KV-Köpfe × 128): ≈ 2,1 MB je 16-Token-Block, ≈ 131 MB je 1 000 Token Prefix — ohne Wertseparation würde Kompaktion diese Daten wiederholt umschreiben.
- **Fähigkeitsverhandlung:** Nur `Generator`-Backends mit `external_kv() = true` und kompatibler `kv_format_id` dürfen Blöcke laden/liefern; andernfalls ist der Cache inaktiv (kein stilles Fallback).
- **Quantisierung:** KIVI 2-Bit ist ein optionales **Speicherformat** (Dekodierung beim Laden); Qualitätsgrenze per Perplexitäts-Gate. Eine Laufzeitbeschleunigung folgt daraus nur mit Backend-Kernel für quantisierte Attention.
- **Break-even-Gate (Voraussetzung für M1-Aussagen):** Mit einem realen Backend in der CI gilt `t_laden(Prefix) + t_Suffix < t_Prefill(Prefix + Suffix)` für definierte Prefixlängen und Tier-Hardware; das Ergebnis wird mit Baseline veröffentlicht, sonst wird M1 nicht beworben.
- **Eviction:** Attention-Gewicht-basiert plus Plattenquote und Speicherdruck-Kopplung.
- **Isolation und Löschung:** je Tenant eigene Segmentschlüssel; Crypto-Shredding (INV-SHRED-1) macht Segmente unlesbar. `forget` eines Dokuments invalidiert **alle Prefixe, die es enthalten** (Lineage-Index Dokument → Prefix-Hash; INV-DERIVED-1).
- **Consistency:** Blockmetadaten laufen im WAL; Cache ist konsistent mit Modell-Hash und Snapshot.

### 13.1 Bedingungen für die M1-Aussage `[R]`

- **B1** Die Löschkette ist verdrahtet: Lineage-Index Dokument → Prefix-Hash, Invalidierung per `forget`, Schlüsselvernichtung der Segmentschlüssel (INV-SHRED-1, INV-DERIVED-1).
- **B2** Ein `Generator`-Backend mit `external_kv() = true` und kompatibler `kv_format_id` ist verfügbar.
- **B3** Das Break-even-Gate (§13) ist mit realem Modell bestanden und mit Baseline veröffentlicht.

Ohne B1 bis B3 wird M1 nicht behauptet (REQ-KVC-001).

---

## 14. Sandbox und Plugins (`contextra-sandbox`) `[R]`

- **Wasmtime**, WASI preview1, **deny-by-default** (kein FS, kein Netz, keine Uhr, kein Zufall).
- **Fuel-Metering** (deterministische Rechenbudgets), Speicherlimit, Stack-Limit, Timeout-Fallback.
- **Modul-Registry:** signierte Module (Ed25519), Version/Hash-gepinnt, Capability-Manifest; Hot-Swap und **Widerruf** (RevocationLog).
- **Human-Approval-Workflow:** Neue/aktualisierte Module mit erweiterten Rechten benötigen Freigabe; Entscheidung wird auditiert.
- **Einhängepunkte:** MergeOperator, Metric-Rerank, Filter (M5), Observer (CDC), Scorer/Ranker, Transformationsstufen.
- **Determinismus:** Gleiches Modul + gleiche Eingabe + gleiches Fuel ⇒ gleiches Ergebnis; Abweichung ist Defekt.

### 14.1 Risikoklassifikation und Freigabe-Zustandsmaschine (INV-APPROVAL-1)

Jede WASM-Gastausführung mit über die Standard-Sandbox hinausgehenden Fähigkeiten MUSS vor Ausführung eine deterministische Risikoklassifikation `ApprovalRisk` durchlaufen, abgeleitet aus den angeforderten `WasmCapabilities`:

| `ApprovalRisk` | Auslösende Capability |
|---|---|
| **High** | Netzwerkzugriff / Cloud-Egress |
| **Elevated** | Dateisystemzugriff ohne Netzwerkzugriff |
| **Low** | Standard-Sandbox (keine erweiterten Capabilities) |

Eine begleitende `ApprovalStatus`-Zustandsmaschine (`Pending → Approved | Rejected`, auditiert) steuert die Freigabe. `High`- und `Elevated`-Module DÜRFEN NICHT ausgeführt werden, bevor der Zustand `Approved` erreicht ist. Für `EnterpriseRegulated` (§23.1) ist die Freigabe Pflicht.

### 14.2 Admission (INV-ADMISSION-1)

Ein `AdmittedModule` ist nur über `ModuleVerifier::verify` konstruierbar; der Konstruktor ist nicht öffentlich. Der Verifier prüft die Ed25519-Signatur gegen die Registry und berechnet den SHA-256-Digest **selbst aus den Bytes** — ein vom Aufrufer gelieferter oder konstanter Digest (z. B. `[0u8; 32]`) ist unzulässig. Für Produktion existiert mindestens ein realer Verifier; ein Port ohne Produktionsimplementierung blockiert die Aktivierung des Plugin-Pfads (INV-WIRING-2). Module ohne Signatur sind nur im Standard-Sandbox-Profil (`ApprovalRisk::Low`) zulässig; ihr berechneter Digest wird im Audit als `unsigned` geführt.

### 14.3 Wasmtime-Konfiguration (INV-WASM-DET-1)

Determinismus über Hosts hinweg setzt voraus: `cranelift_nan_canonicalization(true)`, relaxed-SIMD und Threads aus, Fuel-Metering an, `ResourceLimiter` (Speicher/Tabellen/Instanzen), Pooling-Allocator für Aufrufe im Kompaktionspfad. Eine Wall-Clock-/Epoch-Unterbrechung ist **nur Sicherheitsnetz**: sie erzeugt einen Fehler (fail-closed, Ergebnis verworfen, auditiert), nie ein Teilergebnis. Die Gesetz-Testsuite (§7.5) läuft auf x86-64 und aarch64 mit bit-identischem Ergebnis. **Versionspolitik:** Wasmtime bleibt auf einer vom Hersteller **unterstützten** Release-Linie (Security-Fixes werden nicht auf alte Linien zurückportiert); eine nicht mehr gepflegte Linie ist ein Release-Blocker (§28.4). Der Hersteller pflegt reguläre Releases nur kurz (Größenordnung zwei Monate) und jede zwölfte Hauptversion als LTS-Linie mit 24 Monaten Pflege; für Contextra gilt: eine **LTS-Linie wird bevorzugt**, und die gepinnte Version wird vor jedem Release gegen die Herstellerseite geprüft (E2-Evidenz im Release-Dossier).

---

## 15. Inferenz-Anbindung

### 15.0 Ports und Backends `[R]`

Ports `Embedder`, `Reranker`, `Generator` (in `ports`, `[K]`). Backends (`[R]`): **Candle** (lokal), **Ollama** (lokale Runtime, nur Loopback; ein nicht-lokaler Endpunkt ist ein Egress-Fall, §19), ONNX `[E]`. Alle Backends liefern `model-hash`; Modellwechsel ⇒ kontrollierte Re-Embedding-Migration. Kein Backend sendet Daten ins Netz außer über Privacy-Egress. Der HTTP-/TLS-Stack ist Feature-gebunden und nicht Teil des Air-Gap-Profils. `db` und `engine` kennen **keine** konkreten Backends; die Fassade injiziert sie.

### 15.1 Cross-Encoder-Reranking `[R]`

Die Abstraktion `CrossEncoderReranker` ist Teil des `Reranker`-Ports und MUSS **unabhängig von jeder C/C++-FFI-Anbindung** nutzbar sein (reine Rust-/ONNX-Backends). FFI-Varianten sind Opt-in. Das Feature `reranking` steuert die Aktivierung (Release-Default aus).

### 15.2 Grounding-Validierung `[R]` (INV-GASP-1)

Contextra erzeugt keine Antworten; es stellt `grounding_check(answer, retrieved_chunks)` bereit (`GaspValidator`: `GroundingValidator` + `ResponseGroundingValidator`). **MUSS, wenn aktiviert:** (1) jede Tatsachenbehauptung wird gegen mindestens einen abgerufenen Chunk geprüft; (2) der Konfidenz-Score ist über einen `IsotonicCalibrator` (§12) kalibriert und an den `ConfigFingerprint` der Modellkonfiguration gebunden (Wechsel invalidiert die Kalibrierung); (3) unterhalb des Schwellenwerts (Default **0,70**) liefert der Aufruf `PolicyViolation(LowConfidenceGrounding)` statt einer Freigabe. Die Auslieferung generierter Antworten liegt beim Host; dieser MUSS für Antworten mit M4-Anspruch den Check aufrufen.

### 15.3 Embedding-Identität `[K]` (INV-EMBED-1, INV-EMBED-2)

Es gibt keinen stillen Platzhalter-Embedder: Mock-/Fake-Embedder existieren nur hinter `cfg(test)` oder dem Feature `test-utils`, das in keinem Release-/Default-Feature-Set liegt (Gate über `cargo metadata`); fehlende oder unbekannte Provider-Konfiguration ist ein Startfehler. `EmbeddingBackend ∈ {Onnx, Candle, Ollama, None}` (`None` = nur Textsuche). Jede Collection persistiert `embedding_model_hash` und Dimension; Einfügen und Suche mit abweichendem Hash werden abgewiesen, auch bei gleicher Dimension; Modellwechsel nur über Re-Embedding-Migration (neuer Index je INV-TENANT-IDX-1, atomarer Umschalt-Commit). Ein Semantik-Selbsttest („Hund“/„Welpe“ näher als „Hund“/„Steuererklärung“) ist festes CI-Gate je Backend. Die Modellkonfiguration wird aus dem Modellverzeichnis geladen, nie durch Defaults ersetzt. Embeddings sind **personenbezogene Daten** (Text ist teilweise rekonstruierbar) und werden von den Löschschichten `VectorIndex` und `CachesAndVault` erfasst (§20.4a).

**Modellidentität (REQ-IDX-015).** `embedding_model_hash` ist `BLAKE3` über die kanonische Kodierung von `(Gewichts-Digest, Tokenizer-Digest, Quantisierungskennung, Pooling-/Normalisierungskonfiguration, Dimension, Präfix-Konvention)`; der Typ `ModelIdentity` liegt in `types`, jedes Backend liefert ihn, und die Fassade vergleicht ihn bei jedem `insert`/`search` mit dem in der Collection persistierten Wert. Zwei Modelle mit gleicher Dimension haben verschiedene Hashes; ein Backend ohne berechenbare Identität (z. B. Remote-Endpunkt ohne Gewichts-Digest) ist als Embedder für persistente Collections unzulässig.

---

## 16. Engine und Anfrageausführung (`contextra-engine`) `[K]`

- **HybridQueryBuilder:** deklarativ: Vektor + Text + [Graph `[R]`] + [Filter `[R]`] + `ScopeConstraint` + `ValidityWindow` + `as_of` + Budget. `ScopeConstraint` (Tenant, Domäne, Projekt, Klassifikation, Gedächtnistyp `[R]`) wird in jede Modalität **vor** der Berechnung gepusht.
- **Planner (deterministisch, regelbasiert):** wählt Modalitäten und Parameter (Text-only, Hybrid, Graph-Expansion `[R]`, Multi-Step `[R]`) aus Anfrage, Budget und Index-Statistik; gleicher Zustand ⇒ gleicher Plan. Lernende Strategien sind `[E]` (§18a).
- **Ausführung:** Modalitätszweige im **Bounded-Compute-Pool**; Ring-0-Async-Wrapper; **RRF-Fusion** (k=60, deterministische Tie-Breaker: Score absteigend, DocId aufsteigend); kooperative Abbruchpunkte (INV-CANCEL-1).
- **Bi-temporal:** Gültigkeitszeit × Transaktionszeit („Was galt am Datum X, wie wir es am Datum Y wussten?“).
- **Provenienz:** Jedes Ergebnis trägt Lineage (WAL-Seq, Quelle, Chunk, Embedding-Version), `trust` und `source` (INV-TRUST-1).
- **Schreibpfad:** Embedding und optionale LLM-Schritte laufen **vor** dem Batch; ihre Ergebnisse werden als Werte ins WAL geschrieben (INV-NONDET-1). Embedding ist begrenzt (`max_embed_batch`), nebenläufig im Pool, bricht bei Abbruch ab.
- **Limits (DoS-Schutz, Startwerte, zu kalibrieren):** `max_document_bytes` 16 MiB, `max_chunks_per_document` 4 096, `max_batch_ops` 10 000, `max_collections_per_tenant` 256, `max_in_flight_per_principal` 8. Überschreitung ⇒ strukturierter, nicht retrybarer Fehler (`LimitExceeded`); einzige Ausnahme ist `max_in_flight_per_principal`, dessen Überschreitung den retrybaren Fehler `Busy` liefert (§21.8). **Quoten (REQ-TEN-005):** je Tenant `tenant_quota_bytes` und `tenant_quota_docs`; in `EnterpriseShared`/`EnterpriseRegulated` sind sie Pflichtwerte, in `PowerUserLocal`/`EdgeMinimal` optional. Überschreitung ⇒ `QuotaExceeded` (final, bis Platz geschaffen ist); die Quote zählt den Platzbedarf, den ein Löschvorgang freigibt, erst nach S2.

### 16.1 Chunking-Spezifikation (MUSS)

Jedes Dokument wird vor Indexierung über einen `MarkdownChunker` zerlegt. Verbindlich: (1) **Zielgröße** und **Obergrenze** je Chunk in Token (konfigurierbar; Startwerte 512 / 1024, per Recall-Benchmark zu kalibrieren); (2) **Overlap** bei überlangen Absätzen im Sliding-Window-Fallback (Startwert 10–15 % der Zielgröße); (3) **atomare Blöcke:** Codeblöcke und Tabellen werden nicht mitten im Block getrennt; überschreitet ein Block die Obergrenze, wird er an Zeilen- bzw. Zeilenblockgrenzen geteilt; (4) **Determinismus:** gleicher Text und gleiche Konfiguration ⇒ identische Chunk-Grenzen und Chunk-IDs; (5) jeder Chunk trägt `source_span` (Bytebereich im Quelldokument) für die Lineage; (6) Kopplung an die kontextuelle Präfixierung (§16.2). Ingestion ist ohne diese Konfiguration nicht aktivierbar.

### 16.2 Kontextuelles Chunk-Präfix `[R]`

Vor BM25-Indexierung und Embedding-Berechnung erzeugt eine `ContextPrefixEngine` je Chunk ein kurzes (1–2 Satz) Kontext-Präfix mittels eines kleinen lokalen Modells (z. B. 2–3B-Parameter-Klasse über Ollama). Parameter: `max_document_chars` (Default **8000**), `max_prefix_tokens` (Default **80**). **MUSS:** Vor dem Prompt-Bau wird XML-Escaping angewendet (Prompt-Injection-Isolation des Dokumentinhalts gegenüber der Präfix-Generierungsanweisung); `truncate_chars` wahrt Unicode-Codepoint- und Wortgrenzen. Batch-Verarbeitung läuft sequenziell (kein paralleles Modell-Fan-out, Determinismus- und Ressourcenschutz-Grund). Literaturbasierte Fehlerreduktionswerte für kontextuelle Präfixierung sind nicht am System selbst gemessen und DÜRFEN NICHT als eigene Contextra-Benchmarkzahl kommuniziert werden. Das Präfix ist eine Ableitung (INV-DERIVED-1) und ein aufgezeichneter LLM-Wert (INV-NONDET-1).

### 16.3 Automatische Tripel-Extraktion (OpenIE) `[R]`

Die automatische Wissensgraph-Befüllung aus Fließtext MUSS über einen konkreten LLM-basierten Open-Information-Extraction-Algorithmus erfolgen: satz-/zeilenweises Text-Chunking, striktes JSON-Prompting (Extraktionsschema: `subject`/`predicate`/`object`/`confidence`/`source_span`), Bereinigung der LLM-Antwort von Markdown-Code-Block-Umrandung vor dem JSON-Parsing, Konfidenzschwellenwert-Filterung der extrahierten Tripel und eine harte Obergrenze an LLM-Aufrufen pro Extraktionszyklus. **MUSS:** Überschreitung der Aufrufobergrenze liefert einen strukturierten Fehler (kein Panic-Pfad, Konsistenz mit P7). Extrahierte Tripel sind Ableitungen mit Quellverweis (INV-DERIVED-1) und aufgezeichnete LLM-Werte (INV-NONDET-1).

### 16.4 Retention-Reaper (INV-RETENTION-1, INV-MEMTYPE-2)

Ein periodischer Hintergrund-Worker entfernt (a) abgelaufene `Working`-Einträge und (b) Einträge, die eine **opt-in `RetentionPolicy`** der Collection nach Karenzzeit (Standard 7 Tage in `PendingDeletion`-Zustand, per `restore` rückholbar) erfasst. Batches sind begrenzt (Start 100 Dokumente/Tick), das Herunterfahren erfolgt per Cancellation. Jede Entscheidung wird **als Record ins WAL geschrieben** (Zeit aus `Clock`, Ursache, Policy-Version), die Entfernung folgt der Löschkette (§20.3). Eine Policy ohne Dry-run-Lauf und Audit-Eintrag ist nicht aktivierbar. Ein Storage-interner TTL-Mechanismus auf LSM-Ebene, falls vorhanden, ist davon abzugrenzen und zu dokumentieren.

---

## 17. Cognition (`contextra-cognition`) `[R]`

- **Wissensmodell:** Fakten, Entitäten, Beziehungen, Episoden mit Vertrauenswert, Herkunft, `trust`.
- **Konsolidierungs-Pipeline:** Episodisch → semantisch; Dedup, Konfliktauflösung (bi-temporal: Fakten werden _beendet_, nicht überschrieben), Zusammenfassung. Stufen idempotent, mit **DLQ**. LLM-Ausgaben werden als Werte aufgezeichnet (INV-NONDET-1); jede Ableitung referenziert ihre Quellen (INV-DERIVED-1).
- **Multi-Step-Retrieval:** §17.0a.

### 17.0a Multi-Step Iterative Retrieval Engine

Das Multi-Step-Retrieval MUSS als `MultiStepEngine` nach dem iterativen Query-Rewriting-Muster implementiert sein (vergleichbar mit iterativer Retrieval-Verstärkung in Reasoning-Workflows):

- Bis zu `max_rounds` (Default **3**) iterative Suchrunden, gesteuert von einem generischen, LLM-agnostischen `QueryRewriter`-Port, der alternative Teil-Anfragen basierend auf den bisherigen Ergebnissen generiert.
- **Abbruchkriterium:** ein `quality_threshold`/`min_quality_hits`-Paar beendet die Iteration, sobald ausreichend hochwertige Treffer vorliegen.
- **Fusion:** RRF (§12) über alle Runden hinweg, nicht nur über die letzte Runde.
- **Latenzschutz (MUSS):** Ein `PidLatencyController`-Guard mit `latency_budget_ms` (Default **100 ms**) verhindert unbegrenzte Latenzausweitung durch zu viele Runden — der Regler bricht die Iteration kontrolliert ab, statt das Budget zu überschreiten.
- Sub-Queries innerhalb der Iteration nutzen bewusst nur den Textkanal (BM25) ohne Vektorberechnung, da sie textuelle Umformulierungen der ursprünglichen Anfrage darstellen, keine neue semantische Suche.

### 17.0b Decay (Ranking-Gewicht, INV-DECAY-1, INV-RETENTION-1)

```
elapsed = activity_now − created_at_activity          // Aktivitätsuhr (s. u.)
falls now < created oder elapsed = 0                   → 1.0
None                                                    → 1.0
Exponential{h}                                          → h = 0 ? 0.0 : 0.5^(elapsed / h)
StepFloor{floor}                                        → elapsed < floor ? 1.0 : 0.5
WallClockExponential{half_life_sekunden}                → 0.5^(elapsed_sekunden / half_life_sekunden)
Ergebnis: NaN/∞ → 0.0, sonst auf [0,1] geklemmt
```

**Aktivitätsuhr:** zählt interaktive Schreibtransaktionen; Bulk-Transaktionen (`bulk = true`, z. B. Import) zählen höchstens einen Tick je Batch. Dadurch altert ein Bestand nicht durch Massenimport. `WallClockExponential` ist die **einzige** wall-clock-basierte Variante (Zeit aus `Clock`).

**Decay löscht nie (INV-RETENTION-1).** Unterschreitet das Gewicht `DECAY_FLOOR = 0.05`, gilt der Eintrag als **dormant**: aus dem Standard-Recall ausgeblendet, mit Flag `include_dormant` abrufbar, weiter durchsuchbar und Teil aller Löschbeweise. Ein `AdaptiveDecayController` steuert die Aggressivität (`kappa = 2.0`, `base_half_life_tx = 10_000`).

### 17.1 `MemoryType` — Vier-Typen-Taxonomie

Jede gespeicherte Erinnerungseinheit trägt einen `MemoryType` mit genau vier Varianten (erweiterbar, aber nicht reduzierbar; kanonischer Metadaten-Key in Kleinschreibung):

| Typ | Bedeutung | Retrieval-Signal | Decay-Verhalten | Lebensdauer |
|---|---|---|---|---|
| **Episodic** (episodisches Gedächtnis) | Erlebnisse, Unterhaltungen, zeitlich verankerte Ereignisse | Zeitliche Nähe + Relevanz zur aktuellen Session | Exponentiell, Halbwertszeit 10.000 Aktivitätsticks — moderate Abnahme | dauerhaft, mit Decay |
| **Semantic** (semantisches Gedächtnis) | Fakten, Konzepte, dauerhaftes Wissen — **Default-Typ** bei fehlender Klassifikation | Inhaltliche Ähnlichkeit (Vektor + BM25) | Keine automatische Decay, gültig bis Widerspruch | dauerhaft |
| **Procedural** (Prozessgedächtnis / prozedurales Gedächtnis) | Abläufe, Tool-Nutzungsmuster, Workflows | Task-Matching (SOLL-Erweiterung: Instruktionskodierung) | Nutzungsbasiert (Aktivierungsschwelle 50 Zugriffe) — wird durch Nutzung verstärkt, nicht durch Zeit geschwächt | dauerhaft, nutzungsgestärkt |
| **Working** (operatives Arbeitsgedächtnis) | Kurzzeit-Kontext der aktuellen Session | nicht für Langzeit-Retrieval-Ranking vorgesehen | Exponentiell, Halbwertszeit 500 Aktivitätsticks — sehr schnelle Abnahme | **Session-scoped**: Ablauf nach 50 000 Aktivitätsticks, 4 h Wall-Clock oder Sitzungsende (INV-MEMTYPE-2) |

Abfragen MÜSSEN gezielt nach `MemoryType` filterbar sein (`ScopeConstraint`, §16). Normative Regeln:

**INV-MEMTYPE-1:** Fehlt `memory_type`, gilt `Semantic`. **INV-MEMTYPE-2:** `Working`-Einträge laufen ab nach 50 000 Aktivitätsticks, `working_ttl_wall` (Default 4 h, Clock-Port) oder Sitzungsende (`end_session`), je nachdem, was zuerst eintritt; Entfernung durch den Reaper (§16.4) über die Löschkette. **INV-MEMTYPE-3:** `Procedural`-Verstärkung ist nutzungs-, nicht zeitbasiert (`StepFloor`).

---

## 18. Agent (`contextra-agent`) `[E]`

Agent-Laufzeit: Tool-Schleife mit hartem Budget (Tokens, Zeit, Aufrufe), DLQ, Audit, nur freigegebene `ToolCategory`. **CLM-Scratchpad:** editierbarer, bei Teilziel leerbarer RAM-Arbeitskontext auf Basis des `VolatileContextVault` (§19.5), nie persistiert, jede Änderung als `ContextEditAuditRecord { tenant_id, task_id, operation, subgoal_index, timestamp_nanos }` über einen `ContextEditAuditSink` auditiert; `ScratchpadCacheInvalidator` (Port in `kvcache`) invalidiert betroffene Prefixe gezielt. Abgrenzung: `MemoryType::Working` ist persistent (WAL, TTL), der Scratchpad ist reiner RAM. **Auslagerung empfohlen:** Agentenschleifen gehören dem MCP-Host (§4.2).

---

## 18a. SLM/LLM-Routing (`contextra-router`, Bandit-Mathematik) `[E]`

Nicht-normatives Zielbild. Das Subsystem wählt ein Inferenz-Backend (`SlmProfile`: Name, Endpunkt, `domain_communities`, Token-Budget, Mindest-Relevanz, Kostenschätzung, Konfigurations-Fingerprint, Transport `StdioMcp`/`HttpCloud`) über `RoutingStrategy ∈ {Cascade (Default), ContextualBandit (LinUCB), FlowCorrectedThompson}`.

- **INV-ROUTING-1:** Bandit-Strategien bleiben aus, bis ein Regret-Vorteil gegenüber `Cascade` im Benchmark-Gate nachgewiesen ist. **INV-ROUTING-2:** deterministisch bei gleichem Seed (injizierte Zufallsquelle). **INV-ROUTING-3:** `ProfileCalibrationState` wird bei Fingerprint-Wechsel invalidiert. Bei `PrecisionMatrixDriftDetected` fail-closed.
- Dispatch: `StdioMcp` ausschließlich stdio-JSON-RPC zum Ziel-Endpunkt; `HttpCloud` ausschließlich über die Egress-Pipeline (§19). Contextra öffnet nie einen Netzwerk-Listener.
- Outcome-Recording (`record_outcome`, Propensity für IPS, Lyapunov-Drift, `recalibrate_conformal`) mit beschränkter Eviction offener Entscheidungen.
- **Auslagerung empfohlen:** Die Modellwahl liegt beim MCP-Host; im Kern wählt nur der deterministische Retrieval-Planner (§16). Nach Auslagerung entfällt die Bandit-Mathematik aus `adapt`.

---

## 19. Privacy (`contextra-privacy`) `[R]`

### 19.1 Egress-Schichten 1–5

1. **Policy-Gate** (darf diese Klassifikation überhaupt raus?)
2. **Erkennung** (PII/Geheimnisse/Mandantendaten)
3. **Surrogat-Ersetzung** (Vault: Platzhalter statt Klardaten)
4. **Budget/Rate-Limit** (Bulk-Exfiltrationsdetektor)
5. **Audit & Signatur** (jede Ausleitung protokolliert)

Jeder ausgehende Aufruf an einen nicht-lokalen Endpunkt (Remote-Embedder, Cloud-LLM, Plugin mit Netz-Capability) durchläuft zwingend alle fünf Schichten, bevor Daten den Prozess verlassen. Die Schichten 2 und 3 benötigen **produktive** Implementierungen (`EntityRecognizer`, `InjectionDetector`); ein Null-Objekt als Produktionsstandard ist unzulässig — ohne reale Erkennung ist Egress gesperrt (INV-WIRING-2). **INV-EGRESS-BUDGET-1:** Jede externe Wirkung (Egress, Modellaufruf, Plugin) ist budgetiert, auditiert und widerrufbar.

### 19.2 Surrogat-Vault und Rehydrator

Reversible Pseudonymisierung: Klartext verlässt das System nie; Antworten externer Modelle werden **lokal** rehydriert. Der Vault ist verschlüsselt (Scope-DEK, §20.2) und tenant-gebunden.

### 19.3 GuardedPayload (Typestate)

Ein Payload kann nur an einen Egress-Kanal übergeben werden, nachdem alle Schichten **typisiert** durchlaufen wurden (`Raw → Classified → Surrogated → Approved → Sendable`). Umgehen ist ein Compile-Fehler.

### 19.4 Bulk-Exfiltrationsdetektor

Erkennt ungewöhnliche Abfragevolumen/Scan-Muster pro Prinzipal, drosselt oder sperrt, alarmiert über Audit.

### 19.5 VolatileContextVault — ephemerer RAM-Tresor

Ephemerer RAM-Puffer für sensitive Kontexte (Basis für den Surrogat-Vault dieses Abschnitts und den CLM-Scratchpad `[E]`, §18):

- `VaultConfig { max_capacity_bytes, attempt_mlock }` — Default-Kapazität 512 MiB.
- `VaultChunk { id, content, modality, captured_tx, label }` — ein einzelner Chunk im Vault.
- `VolatileContextVault { open, ingest, purge, current_size_bytes, len, is_empty, preview_metadata }`.
- **MUSS:** `mlock` wird versucht (über `contextra-sys`: `mem_lock`/`mem_unlock`, wirksam unter Linux, begrenzt unter macOS/Windows; scheitert die Sperrung, meldet `mem_lock` dies wahrheitsgemäß (`false`/`Unsupported`), und `doctor` weist den Zustand aus, statt Erfolg vorzutäuschen (INV-PLATFORM-1)), Inhalte werden beim Verwerfen gezeroized.
- `LockedRegions` (in `contextra-sys`) verwaltet gesperrte Adressbereiche (`lock_slice`, `unlock_all`).
- Fehlerklasse `VaultError ∈ {CapacityExceeded, AlreadyConsumed}`.

---

## 20. Kryptografie, Löschnachweis, Audit (`contextra-crypto`) `[K]`

### 20.0 Rechtlicher und fachlicher Rahmen `[K]`

Dieser Abschnitt fasst zusammen, welche rechtlichen und fachlichen Maßstäbe die Löschfunktionen technisch bedienen müssen. Er ist **keine Rechtsberatung** und trifft **keine Rechtsentscheidung**: Wo die Rechtslage offen ist, bedient die Technik alle plausiblen Auffassungen (Verteidigung in der Tiefe), und der Nachweis behauptet nur, was gemessen wurde. Stand der zugrunde liegenden Recherche ist Oktober 2026; Gesetzestexte und Behördenpapiere sind vor Verwendung in Produkttexten im Volltext zu prüfen (Prüfpunkt P-14).

Legende der Belegstärke: **[R]** verbindliches Recht (Gesetz, Verordnung, EuGH/BGH); **[B]** Auslegung von Behörden (EDPB, DSK, Landesbehörden, Standard-Datenschutzmodell, nicht bindend, aber Maßstab der Aufsichtspraxis); **[P]** Fachliteratur, Praxisempfehlung, technische Norm; **✔** an der Primärquelle geprüft; **◐** nur über Sekundärquelle belegt; **○** allgemeiner Kenntnisstand, nicht verifiziert; **⚠** umstritten oder offen.

**Löschpflicht und Löschbegriff.** Art. 17 Abs. 1 DSGVO verpflichtet den Verantwortlichen, personenbezogene Daten **unverzüglich** zu löschen, wenn einer der Gründe des Absatzes vorliegt; die Pflicht besteht auch ohne Antrag [R]. Die Antwortfrist auf Anträge beträgt grundsätzlich einen Monat (Art. 12 Abs. 3) [R ○]. Die DSGVO definiert „Löschen“ nicht und schreibt keine Methode vor; die Löschung ist selbst eine Verarbeitung und unterliegt Art. 5 und Art. 32 [R]. Das Standard-Datenschutzmodell (Baustein 60) versteht Löschen als **Unkenntlichmachen**: Danach darf keine Person mehr identifizierbar sein, und der Informationsgehalt darf nicht oder nur mit nach allgemeinem Ermessen unwahrscheinlichem Aufwand reproduzierbar sein; die Löschung ist erst vollzogen, wenn auch keine Kopie oder Replikation beim Verantwortlichen oder Auftragsverarbeiter mehr besteht [B ✔]. Technische Unmöglichkeit rechtfertigt keine Verstöße (EDPB-Leitlinien 02/2025, Rn. 50) [B ✔].

**Anonymisierung als Löschung ⚠.** Ob Anonymisierung die Löschpflicht erfüllen kann, ist umstritten: Instanzgerichte und einzelne Behörden bejahen es unter Bedingungen [R-Instanz ◐, B ◐]; die europäischen Aufsichtsbehörden stellen klar, dass bloße Pseudonymisierung oder Maskierung nicht genügt (EDPB-Bericht zur koordinierten Prüfung 2025, angenommen Februar 2026) [B ✔]. Es gibt kein bekanntes höchstrichterliches Urteil dazu.

**Kryptografisches Löschen ⚠.** Verschlüsselung ist eine Sicherheitsmaßnahme (Art. 32) und kann Pseudonymisierung sein (Art. 4 Nr. 5); pseudonymisierte Daten bleiben personenbezogen, solange Zusatzinformationen (hier: Schlüssel) existieren [R ○]. Nach der relativen Betrachtung des Personenbezugs kann es auf die Mittel der jeweiligen Stelle ankommen (EuGH Breyer; EuGH C-413/23 P) [R ○]. Die einzige gefundene Aussage einer Aufsichtsbehörde zur Schlüsselvernichtung steht in den EDPB-Leitlinien 02/2025 (Blockchain, Endfassung Juli 2026): Nach Löschung des Schlüssels sind verschlüsselte Daten unverständlich, solange der Algorithmus nicht gebrochen wird und der Schlüssel nicht kompromittiert war; verschlüsselte personenbezogene Daten bleiben jedoch personenbezogen, und bei unbegrenzter Speicherung wird auch gute Verschlüsselung von der Zeit überholt; verlangt werden ein Notfallplan für den Austausch von Algorithmen und eine periodische Neubewertung (Rn. 51, 85, 96) [B ✔, Übertragung auf RAG ist Analogie]. Für DSK und Landesbehörden wurde keine ausdrückliche Aussage zu Crypto-Shredding gefunden. **Es ist keine gerichtliche Entscheidung bekannt, die Crypto-Shredding als ausreichend anerkennt.** Als technischer Standard kennt NIST SP 800-88 Rev. 2 (September 2025) „Cryptographic Erase“ als Purge-Methode unter strengen Voraussetzungen [P ◐]. Ob Schlüsselvernichtung rechtlich Löschung, Anonymisierung oder Einschränkung ist, bleibt **offen** (P-01); beide Wege laufen auf dieselbe Wirksamkeitsprüfung hinaus: Es müssen **alle** Schlüsselkopien vernichtet und alle Repräsentationen erfasst sein.

**Rückstände.** Die Löschpflicht betrifft grundsätzlich auch Sicherungskopien; das heißt nicht, dass in jedem Backup zum selben Zeitpunkt gelöscht werden muss, aber **nach einer Rücksicherung muss unmittelbar erneut gelöscht werden** und ein Überwachungsprozess dafür bestehen [B ✔]. Konkrete Höchstfristen für Backups sind Praxisempfehlung, keine Behördenvorgabe [P]. Auch Protokolle mit Personenbezug, temporäre Dateien, Caches und Indizes unterliegen der Löschpflicht; das bloße Austragen aus dem Dateisystem genügt oft nicht [B ✔]. RAG-typische Restquellen sind soft-gelöschte Vektoren in Graphindizes, Segment-Merges im Volltextindex, Prompt-Historien, Embedding- und LLM-Kontext-Caches, Swap und Crash-Dumps [P].

**Embeddings ⚠.** Keine Entscheidung stellt verbindlich fest, dass jedes Text-Embedding personenbezogen ist; Behörden behandeln den Inhalt der Vektordatenbank jedoch als Verarbeitungsgegenstand und weisen auf Rekonstruktionsrisiken hin [B ✔]. Wegen der gemeinsamen Speicherung mit dem Chunk-Text, der Verknüpfung über IDs und der Inversionsmöglichkeit behandelt diese Spezifikation Chunks, Vektoren und Indizes **vorsorglich als personenbezogen** (P-05).

**Nachweis und Rechenschaft.** Art. 5 Abs. 2, Art. 24 und Art. 32 verlangen, Löschung verifizieren und nachweisen zu können [R/B ✔]. Der Nachweis steht im Spannungsfeld zur Rückstandslosigkeit („Lösch-Paradoxon“): Ein Nachweis mit Personenbezug braucht eine eigene Rechtsgrundlage und Frist (P-07), weshalb der Nachweis datensparsam sein muss (§20.11).

**Aufbewahrungspflichten.** Art. 17 Abs. 3 nimmt u. a. rechtliche Aufbewahrungspflichten aus; dann tritt Einschränkung (Art. 18) an die Stelle der Löschung, und das Archiv ist vom aktiven Retrieval-Index zu trennen. Wie die Einschränkung eines Index technisch zu verstehen ist, ist ungeregelt ⚠ (P-09).

**Produktaussagen.** Aussagen wie „garantiert“, „DSGVO-konform“ oder „erfüllt Art. 17 automatisch“ bergen wettbewerbs- und vertragsrechtliche Risiken [R ○]. Produkt- und Beweistexte beschreiben deshalb ausschließlich, **was technisch geschieht** (INV-CLAIMS-1, §4.3).

**Technische Anforderungen aus dem Rahmen.** Die folgende Tabelle ordnet jede abgeleitete Anforderung dem Abschnitt zu, der sie umsetzt.

|ID|Technische Anforderung|Status der Rechtslage|Umsetzung|
|---|---|---|---|
|A1|Nach der Löschung ist keine Person identifizierbar und der Informationsgehalt mit vernünftigen Mitteln nicht reproduzierbar; auch Kopien beim Verantwortlichen sind erfasst.|[B ✔]|§20.3, §20.4, §20.8|
|A2|Alle Repräsentationen im Wirkungsbereich: Quelldokument, Chunk, Vektor, Volltextindex, Graph, Cache, Log.|[P]; Embeddings ⚠ (vorsorglich personenbezogen)|§20.4|
|A3|Bei Schlüsselvernichtung: alle Schlüsselkopien (KMS, HSM, Backups des Schlüsselspeichers, Escrow, Swap, Dumps) erfasst oder als nicht abgedeckt deklariert.|⚠ offen|§20.2a, §20.2b, §20.15|
|A4|Schlüsselgranularität (Betroffener, Dokument, Wissensbasis) ohne Kollateralverlust.|[B ✔]|§20.2b|
|A5|Verbleibende Metadaten (IDs, Größen, Zeitstempel, Schlüssel-IDs) identifizieren nicht wieder.|[P]|§20.11|
|A6|Löschung ist verifizierbar und nachweisbar.|[R/B ✔]|§20.5, §20.8|
|A7|Nachweis ist datensparsam (Metadaten statt Inhalt, keine Klartext-IDs).|⚠ umstritten|§20.11|
|A8|Nach Rücksicherung unmittelbare erneute Löschung; Überwachungsprozess.|[B ✔]|§20.9, §24.4|
|A9|Aufbewahrungspflichten: Einschränkung statt Löschung, Archiv getrennt vom Retrieval-Index.|[R] + ⚠ Index-Einschränkung|§20.10, REQ-DEL-018|
|A10|Krypto-Agilität: Notfallplan für Algorithmenwechsel, periodische Neubewertung.|[B ✔ (Analogie)]|§20.1, REQ-DEL-028|
|A11|Unverzüglich; Antwortfrist ein Monat.|[R ○]|§20.10|
|A12|Ehrliche Produktaussagen.|⚠|§4.3, §20.12|
|A13|Löschkonzept: Fristen, Rollen, Kopienzahl und -ort sind dokumentierbar (Audit-Export, `doctor`).|[B ✔]|§20.7, §21.7|
|A14|Empfänger werden informiert (Art. 17 Abs. 2, Art. 19); Auftragsverarbeiter-Löschung ist nachweisbar.|[R ○/◐]|§19, §20.7|

### 20.1 Primitive

**AES-256-GCM-SIV** (nonce-misuse-resistant AEAD), **Argon2id** (Passphrase-KDF, INV-KDF-1), **HKDF** (Ableitung aus hochentropischem Schlüsselmaterial je Zweck/Label, nie direkt aus Passphrasen), **HMAC-SHA256** (WAL-Kette), **Ed25519** (Signaturen), **SHA-256/BLAKE3** (Hashing/Merkle). Zufall für Schlüssel, Nonces und Salts stammt aus dem System-RNG (P28-Ausnahme); Geheimnisse sind `zeroize`-geschützt und werden konstantzeitig verglichen. **Krypto-Agilität (REQ-DEL-028):** Jedes persistierte kryptografische Objekt trägt Algorithmus- und Formatkennung; ein Wechsel von AEAD, KDF, Hash oder Signaturverfahren ist ohne Datenverlust möglich (Re-Wrap bzw. Neuverschlüsselung per Kompaktion), der Austauschplan ist dokumentiert, und die Eignung der Primitive wird periodisch neu bewertet (Prüfpunkt P-13).

### 20.2 Schlüsselhierarchie und KmsProvider (INV-KMS-1, INV-ENCRYPT-REST-1)

```
Passphrase | Keyfile | OS-Keyring ──Argon2id(KdfHeader)──► KEK_root        (KmsProvider)
KEK_root ──HKDF(Label)──► K_eng            WAL-Frames, SSTable-Blöcke (Struktur, Integrität)
KEK_root ──wrap──────────► DEK(scope)       zufällig, 256 Bit, im KeyRegistry-Log
```

- **Scope** = (Tenant, Collection); opt-in `subject_scope` je Collection: (Tenant, Collection, Subject) für Betroffenen-/Akten-genaue Löschung (Kosten: mehr Schlüssel).
- **Verschlüsselungseinheit** ist der Dokument-Record (Nonce 96 Bit zufällig + Tag 128 Bit = 28 Byte Overhead; AD = `scope_id ‖ key_id ‖ doc_id_hash`).
- **Unter der Scope-DEK:** Dokumentwerte, Chunk-Text, Metadatenwerte, Embeddings (Vollpräzision und SQ8), Index-Dateien (HNSW-Snapshot, Text-Segmente, Graph), Blob-Segmente, Surrogat-Vault. **Nur unter `K_eng`:** LSM-Schlüssel (opake IDs, INV-KEYSPACE-1), Manifest, Sequenznummern, Tombstones, Kettenköpfe.
- **Crypto-Shredding** = Vernichtung der gewrappten DEK (RAM-Kopien zeroized) + durabler `RevocationLog`-Eintrag (INV-SHRED-1). Es wirkt auf ganze Scopes. `forget` einzelner Dokumente in einem gemeinsamen Scope ist physisch (S1–S3), außer `subject_scope` ist aktiv oder die Collection trägt `erasure_required`; dann wirkt die Vernichtung dokumentgenau über die Epochen-Hierarchie (§20.2b).
- **Rotation:** Re-Wrap der DEKs unter neuer KEK ohne Datenrewrite (Envelope); Rotation von `K_eng` läuft per Hintergrundkompaktion.
- **KeyRegistry:** append-only, HMAC-verkettet, nicht ohne durables Log konstruierbar (Kodierung, Kette und Signatur liegen in `crypto`, die durable Ablage läuft über den Port `DurableLog`, den `store` mit `fsync` je Eintrag implementiert); ein gelöschtes Log samt Marker bei vorhandenen Daten gilt als Manipulation (fail-closed). `KmsProvider`: erzeugen/ableiten/wrappen/entwrappen/vernichten; **lokaler Default** (Argon2id-geschützter Keystore, INV-KDF-1) plus externe Referenzimplementierung (HSM/Cloud-KMS-Adapter).

### 20.2a Verwahrungsstufen `[K]`

Die Verwahrungsstufe bezeichnet, wie das Wurzelmaterial (`KEK_root`) geschützt und vernichtbar ist. Sie steht im Beweis (§20.5) und bestimmt zulässige Claims (§20.12).

|Stufe|Verwahrung|Vernichtungsnachweis|
|---|---|---|
|**K0**|Keine Schlüsselverwahrung (Verschlüsselung aus)|keiner; kein kryptografischer Claim|
|**K1**|Lokaler, passphrasegeschützter Keystore (Datei, `LocalFile`)|lokale Vernichtung, **Remanenz des Schlüsselmaterials nicht ausgeschlossen**; der Beweis nennt die Vernichtung als ausgeführte Maßnahme, leitet daraus aber **keinen** kryptografischen Claim ab|
|**K2**|Lokal, hardwaregebunden (TPM/Secure Enclave, `LocalSealed`)|hardwaregestützt; Remanenz geringer, aber Backup-/Escrow-Kopien weiter möglich|
|**K3**|Externer KMS/HSM mit Vernichtungsattestierung durch den Betreiber des KMS|Attestierung des Custodians|

Abbildung auf den Verwahrer-Typ (`CustodianKind`): K1 = `LocalFile`, K2 = `LocalSealed`, K3 = `Kms` oder `Hsm`. Die Rechtswirkung der Schlüsselvernichtung ist ausdrücklich offen (Prüfpunkte §20.13).

### 20.2b Dokumentgenaue Schlüsselvernichtung (Epochenmodell) `[K]`

Collections mit `erasure_required = true` verlangen Löschung einzelner Dokumente und Betroffener **ohne Kollateralverlust** (Anforderung A4). Für sie ersetzt folgende Hierarchie die einstufige Scope-DEK aus §20.2; Collections ohne dieses Flag behalten die Scope-DEK und löschen nur als ganzer Scope kryptografisch.

```
Verwahrer-Wurzel (KMS/HSM | LocalSealed | LocalFile)
   └─ TenantKek            je Mandant; nie auf Platte außer gewrappt durch den Verwahrer
        └─ EpochKek[E]     je Löschepoche E; vernichtbar
             ├─ DocDek[d]  je Dokument/Chunk-Gruppe; 256 Bit aus dem System-RNG; gewrappt unter EpochKek[E]
             └─ SegKey[s]  je persistiertem Segment abgeleiteter Schichten; gewrappt unter EpochKek[E]
```

1. **Schlüssel nie neben Chiffretext.** Gewrappte DEKs liegen in einem getrennten `ErasureKeyStore`; der Payload trägt nur eine `KeyRef { tenant, epoch, key_id }` (REQ-DEL-026).
2. **Löschen eines Dokuments ist eine Epochen-Rotation (Re-Wrap, dann Vernichtung):** (a) neue `EpochKek[E+1]` beim Verwahrer anlegen; (b) alle **überlebenden** `DocDek`/`SegKey` aus `E` unter `EpochKek[E+1]` neu wrappen, `fsync`, den Zeiger im `ErasureKeyStore` atomar umschalten; (c) `EpochKek[E]` beim Verwahrer **vernichten** (Vernichtungsattestierung, lokaler Speicher genullt); (d) alte Key-Store-Segmente für `E` physisch entfernen und, falls der Verwahrer lokal ist, die Remanenz als Restrisiko deklarieren (§20.15).
3. **Warum nicht nur den gewrappten DEK löschen?** Ein Append-only-Store und Flash-Wear-Leveling bewahren Altbytes; solange `EpochKek[E]` existiert, sind sie entwrapbar. **Nur die Vernichtung des Wrapping-Schlüssels macht Remanenz unschädlich** (REQ-DEL-025).
4. **Kostenbegrenzung.** Der Re-Wrap-Aufwand wird durch ein konfigurierbares **Batch-Fenster** (nie über die gesetzliche Frist hinaus, §25.6) und eine **Obergrenze der Schlüssel je Epoche** gedeckelt. Während der Rotation schützt die Lesesperre (`Frozen`, §20.9) den Betroffenen.
5. **Abgeleitete Schichten** (HNSW-Snapshot, Postings, Graph-CSR, KV-Cache) werden aus Latenzgründen nicht je Dokument verschlüsselt. Stattdessen liegen persistierte Segmente unter `SegKey[s]`; bei Löschung wird das betroffene Segment **ohne** den Datensatz neu geschrieben (neuer `SegKey` unter `EpochKek[E+1]`), und das alte Segment verliert seinen Schlüssel. Der Aufwand ist durch die Segmentgröße beschränkt.
6. **Startbedingung.** Eine Collection mit `erasure_required = true` startet nur, wenn Verschlüsselung im Ruhezustand aktiv **und** ein Verwahrer konfiguriert ist; andernfalls entsteht `Err(CapabilityUnsupported)`, niemals ein Stub (REQ-DEL-024). Ohne Verschlüsselung gibt es keinen Beweis mit kryptografischem Claim.
7. **Checkpoints und Backups.** Checkpoints sind Hardlink-Klone derselben Dateien. Sind die Daten unter `DocDek`/`SegKey` verschlüsselt, erfasst die Schlüsselvernichtung automatisch alle Klone und Backups ohne Keystore; physisches Löschen erreicht Klone nicht. Das ist der stärkste Hebel der Architektur.

### 20.2c Zweckregister und Domänentrennung `[K]`

Jeder Schlüssel dient genau einem Zweck (INV-KEYPURPOSE-1). Die Ableitung per HKDF nutzt je Zweck ein eigenes Label der Form `contextra/v1/<zweck>`. Das Register `governance/key_purposes.toml` ist die einzige Quelle dieser Labels; das Gate vergleicht sie mit dem Code (REQ-KEY-011).

|Zweck|Label bzw. Herkunft|Ableitung|Wirkung der Vernichtung|
|---|---|---|---|
|Struktur-Schlüssel `K_eng`|`contextra/v1/eng`|HKDF aus `KEK_root`|keine Nutzdaten; nie für Nutzdaten verwendet|
|WAL-Integrität|`contextra/v1/wal`|HKDF aus `K_eng`|keine|
|Sidecar-Ketten|`contextra/v1/sc/keyreg`, `…/revoke`, `…/audit`, `…/journal`, `…/keystore`, `…/erased`|HKDF aus `K_eng`|keine; Ketten sind keine Datenschlüssel|
|Keystore-Integritätswurzel|`contextra/v1/keystore-root`|bei K0 im Klartext, sonst gewrappt|keine|
|Scope-DEK|zufällig, 256 Bit, gewrappt|`KEK_root`|Crypto-Shredding ganzer Scopes (INV-SHRED-1)|
|Epochen-Schlüssel|`TenantKek`, `EpochKek`, `DocDek`, `SegKey`|Epochenmodell (§20.2b)|dokumentgenaue Löschung|
|Beweissignatur|Signierschlüssel des `KeyManager`|`KmsProvider`|keine; Beweise bleiben prüfbar (INV-PROOF-KEY-1)|
|Surrogat-Vault|`contextra/v1/vault`|HKDF aus `K_eng`|Löschschicht „Surrogat-Vault/Caches“ (§20.4a)|

Regeln: (1) Labels sind paarweise verschieden; eine Wiederverwendung eines Schlüssels für zwei Zwecke ist unzulässig. (2) Ein Label ohne Registereintrag ist Gate-Fehler. (3) Sidecar-Ketten (§7.11) und Struktur-Schlüssel überleben jede Schlüsselvernichtung nach Nutzdaten; sie enthalten keine personenbezogenen Nutzinhalte (INV-LOG-1).

### 20.3 Löschstufen (Garantiemodell)

|Stufe|Wirkung|
|---|---|
|**S0 Logisch**|Tombstone/Revokation; für Leser sofort unsichtbar.|
|**SK Kryptografisch**|Scope-DEK vernichtet (ganze Scopes) bzw. `EpochKek` vernichtet nach Re-Wrap der Überlebenden (dokumentgenau, §20.2b). Wirkt auf jede Ciphertext-Kopie, die den Keystore nicht enthält (SSD-Remanenz, Snapshots, Backups).|
|**S1 Indexschichten**|Vektor-Payload im Slot genullt (§9.3), Text-Segment-Rewrite, Graph-/KV-Einträge bereinigt, Abwesenheit verifiziert.|
|**S2 SSTables**|Memtables geflusht; betroffene SSTables per Purge-Kompaktion ohne die Daten neu geschrieben; Alt-Dateien per `durable_remove` entfernt (Windows: Reader-Handles vorher freigeben, `FILE_SHARE_DELETE`). Offene Snapshots verzögern S2 (Status `Pending`), sie werden nicht abgebrochen.|
|**S3 WAL**|WAL-Segmente mit Records des Scopes nach Checkpoint **gescrubbt** (Überschreiben, Flush, Entfernen; nur auf Anforderung eines Löschvorgangs); die Kette bleibt validierbar (Tombstone-Record mit Beweisbindung). Ein Beweis über `WalAllSegments` setzt einen Vollscan der aktiven Segmente mit dem `KeyManager` voraus.|

Für S2/S3 behauptet der Beweis „kein lesbarer Klartext des Scopes“, nicht „Bytes physisch weg“. Nach SK ist der Scope auch dann unlesbar, wenn S2/S3 noch `Pending` sind; der Beweis weist beides getrennt aus.

### 20.4 Sieben Löschschichten (`DeletionLayer`)

Primärdaten (LSM) · Vektorindex · Textindex · Graph · KV-Cache · Surrogat-Vault/Caches · Audit-/Lineage-Referenzen (pseudonymisiert, Hash bleibt). Jede Schicht liefert einen `LayerWitness` (Hash über Vorher-/Nachher-Zustand, Zeit, Verifikator) in einer von zwei Arten: **`Physical`** (Scanner bestätigt Abwesenheit) oder **`CryptoShredded`** (Verweis auf den `KeyDestroyRecord`). Schichten ohne Scanner und ohne vernichteten Schlüssel (`CsrGraph`, `KvCacheSegments`, `EmbeddingCache`, bis ein Scanner existiert) erscheinen als `Uncovered(NoPhysicalScanner)`. Zusätzlich gilt:

- **Embeddings und Vektorbytes sind personenbezogene Daten:** „Text gelöscht, Vektor im Index“ ist keine Löschung.
- **Ableitungen (INV-DERIVED-1):** `forget` kaskadiert über die Lineage auf Konsolidierungsfakten, Tripel, Chunk-Präfixe, Zusammenfassungen und KV-Prefixe; der Beweis führt sie als eigene Einträge.
- **CDC-Ereignisse sind inhaltsfrei** (§7.3), erzeugen also keine Löschkopien bei Observern.
- Der Memtable-Witness ist erst nach `force_flush` belegbar (überschattete Versionen sind bis dahin physisch vorhanden). `Working`-Einträge, die der Reaper entfernt, durchlaufen dieselbe Kette.

### 20.4a Katalog der sieben Löschschichten `[K]`

Die Namen sind die Typnamen in `DeletionLayer` (§20.15). Diese Tabelle ergänzt §20.4 und ist für Verweise aus §6.11 und §15.3 maßgeblich.

|Schicht|Typname|Besitzer und Scanner|Witness-Arten|Ohne Rand (§6.11)|
|---|---|---|---|---|
|Primärdaten (LSM)|`Lsm`|`store`|`Physical`, `CryptoShredded`|—|
|Vektorindex|`VectorIndex`|`vector`|`Physical`, `CryptoShredded`|—|
|Textindex|`TextIndex`|`text`|`Physical`|—|
|Graph|`Graph`|`graph` `[R]`|`Physical`|`FeatureDisabled`, wenn keine Graph-Daten existieren|
|KV-Cache|`KvCache`|`kvcache` `[R]`|`Physical`, `CryptoShredded` (über `SegKey`)|`FeatureDisabled`|
|Surrogat-Vault und Caches|`CachesAndVault`|`privacy`, `store`|`Physical`|—|
|Audit- und Lineage-Referenzen|`AuditLineage`|`engine`, `store`|`Physical` (pseudonymisiert, Hash bleibt)|—|

Regeln: (1) Jede Schicht stellt genau einen Witness-Typ aus; ein Scanner gehört genau einem Crate (§6.10). (2) `FeatureDisabled` ist nur zulässig, wenn der Rand-Feature-Schalter aus ist **und** der Start den fehlenden Datenbestand belegt; sonst gilt `Uncovered(NoPhysicalScanner)` (§20.4). (3) `EmbeddingCache` fällt unter `CachesAndVault`, sobald sein Scanner existiert (§15.3).

### 20.5 DeletionProof (Formatversion 4)

Strukturiert: `subject_hash`, `scope`, `covered_layers` (je Schicht ein `LayerWitness`), **`uncovered_layers`** (`UncoveredLayer { layer, reason }`; ehrlich deklarierte Grenzen: SSD-physisch, Backups, Swap, Modellparameter, Keystore-Kopien), `shred_evidence` (`KeyDestroyRecord { key_id, registry_seq, revocation_seq }`, falls SK), `stage_reached`, Zeitstempel (Clock-Port), Verweis auf AuditChain-Position, **Ed25519-Signatur** über das kanonische Encoding (`SignatureVersion::V4`).

Zusätzlich trägt der Beweis: `outcome` (`Erased`, `Restricted{basis, until}`, `Partial{outstanding}`), `claims` (nur aus Messungen erzeugbar, §20.12), `excluded` (Pflicht, nie leer), `key_destruction` (Vernichtungsattestierungen), `custodian_kind`, `requested_at`/`completed_at`, `clock_source` (`System` oder `Notarized{tsa}`), optional `wal_chain_receipt` und `log_inclusion` (Merkle-Einschluss mit signiertem Kettenkopf), `signer_key_id`. Der `subject_hash` ist ein Commitment (§20.11). Ein Beweis mit `outcome = Erased` entsteht nur, wenn jede Pflichtschicht einen sauberen `ResidualReport` hat; sonst ist das Ergebnis `Partial`.

- **INV-PROOF-COVERAGE-1:** Jede der sieben Schichten erscheint genau einmal in `covered_layers` oder `uncovered_layers`; der **Typestate-Builder** macht das Auslassen zum Kompilierfehler.
- **INV-WITNESS-MINT-1:** `LayerWitness` lebt in `crypto`, hat private Felder, kein `Default`/`Deserialize`; Ausstellung nur über `LayerWitness::issue_from_storage_layer`, aufrufbar ausschließlich aus `store`/`vector`. Durchsetzung zweistufig: Typsystem **und** Gate (Aufruf außerhalb dieser Crates verboten; `DeletionProof::create*`, `LayerCleanupProof::new_after_verified_empty` und vergleichbare Bypass-Konstruktoren außerhalb von `test`/`fuzz`/`benches` verboten). Rust erzwingt „nur Storage kann ausstellen“ nicht rein typbasiert; die Gate-Regel ist Teil der Garantie.
- **INV-PROOF-KEY-1:** Der Signierschlüssel stammt aus dem `KeyManager` (deterministisch, domänengetrennt), nie aus einem frei übergebenen Byte-String. Produktion stellt ausschließlich Formatversion 4 aus; die Altformate v1–v3 sind nur lesbar und prüfbar, werden nie mehr erzeugt und melden `coverage_known = false` (im Audit-Export als „Altformat, nicht beweisend“ gekennzeichnet).
- **Verifikation:** `verify` prüft die Signatur; `coverage()` liefert `covered`, `uncovered`, `coverage_known` getrennt. Eine gültige Signatur bedeutet keine Mindestabdeckung. Prüfung **offline** über `verify-proof` (`verify_external`, nur `crypto` + `types`, kein Datenbankzugriff).

### 20.6 AuditChain und RevocationLog

**AuditChain:** append-only, hash-verkettet, signierte Checkpoints, tenant-getrennt, enthält Zugriffe, Löschungen, Egress, Plugin-Freigaben, Lizenzereignisse. **RevocationLog:** widerrufene Schlüssel/Module/Berechtigungen; wird bei jeder Autorisierung konsultiert (fail-closed). Beide sind durabel (Flush vor Wirksamkeit, Persistenz über den Port `DurableLog`) und crash-sicher; Tail-Truncation ist über einen signierten Kettenkopf und optional einen externen Zeitanker (`NotaryPort`) erkennbar. Beide Dekoder sind Fuzz-Ziele (§27.1); Audit-Inhalte enthalten nur pseudonymisierte Referenzen (INV-LOG-1).

### 20.7 Compliance-Ausleitungen `[E]`

`audit-export` erzeugt offline prüfbare Bundles (Kette + Signaturen + Manifest), `avv-generator` AVV-Vorlagen aus realer Konfiguration/Auditdaten. **INV-CLAIMS-1** gilt für jede Außenaussage (README, MCP-Antworten, Exporte): Zusicherungen entstehen aus der belegten Garantiestufe (Beweisabdeckung, `maturity`), nie aus dem Zielzustand. Generierte Rechtsdokumente sind Vorlagen mit Version, Datum, Garantiestufe und dem Hinweis, dass sie keine Rechtsberatung ersetzen und vor Verwendung juristisch zu prüfen sind; ihre Auslieferung setzt eine dokumentierte juristische Prüfung der Vorlage voraus. Ein BSI-TR-02102-1-Mapping der Primitive (§20.1) mit Markdown-Renderfunktion wird bei jeder Primitiv-Änderung nachgeführt (INV-DOC-SINGLE-SOURCE-1).

### 20.8 Messprinzip `[K]`

1. **Messen vor Behaupten.** Ein Beleg über eine Schicht (`LayerWitness`, in der Messung getragen von einem `ResidualReport`, §20.15) entsteht nur aus einer Messung am **Speicher**, nicht am Lesepfad: Sichtbarkeit (`scan_prefix`, Suche) belegt keine physische Abwesenheit, weil Tombstones und Altversionen unsichtbar sind.
2. Ein Messwert MUSS vom Scanner der Schicht erzeugt werden, trägt Methode (`RawFileScan`, `IndexIntrospection`, `BlackBoxProbe`, `KeyAbsence`), `units_scanned`, `residual_units`, einen Digest über Scan-Parameter und Dateiliste (kein Klartext) und einen Zeitstempel aus dem `Clock`-Port. `units_scanned = 0` ist außer bei `KeyAbsence` nicht konstruierbar.
3. Eine **Konstante oder ein vom Aufrufer gelieferter Zählerwert** DARF NIE als Messung dienen (REQ-DEL-008). Es gibt keinen Fallback- oder Standardschlüssel für Beweise (REQ-DEL-009).
4. Fehlt einer Schicht die Fähigkeit zu messen, ist sie `Uncovered(NoPhysicalScanner)`; fehlende Fähigkeit ist `CapabilityUnsupported`, nie ein Erfolgsstub (REQ-DEL-010).
5. Zwei unabhängige Messungen je Pflichtschicht: Scanner der Schicht **und** Black-Box-Probe durch den Orchestrator über einen internen Prüfzugang.
6. Der Memtable-Beleg ist erst nach `force_flush` belegbar (§20.4).

### 20.9 Lesesperre, Key-Lock und Restore-Guard `[K]`

- **Lesesperre.** Mit Beginn der Löschung (`Frozen`) trägt ein persistenter `ErasedSet` nur Commitments. `get`, `search`, `hydrate`, `export` und Checkpoint-Erzeugung prüfen ihn **vor** Rückgabe (REQ-DEL-011).
- **Key-Lock.** Das Löschen hält den Key-Lock des Dokuments über die **gesamte** Transaktion, nicht nur bis zum Transaktionsbeginn (REQ-DEL-012).
- **Restore-Guard.** `restore_checkpoint` liest den `ErasedSet`, sperrt vor dem ersten Lesezugriff, wendet jede Löschung erneut an und schreibt je Eintrag ein Journal-Ereignis `restore_reapplied` (REQ-DEL-013). Backups außerhalb von Contextra bleiben `ExternalBackups` im Beweis; der Verantwortliche trägt die Rücksicherungsregel.

### 20.10 Löschsaga `[K]`

```
Requested → Authorized → Frozen → LogicallyDeleted → KeysDestroyed/Rotated → PhysicallyPurged
   │             └─(Hold)→ Restricted                                              │
   └→ Rejected                                           Measured → Attested → Logged → Completed
                              (jeder Zustand) → Partial{outstanding} → Retry/Escalate
```

Jeder Übergang wird **vor** der Ausführung in ein durables Journal geschrieben (fsync); alle Schritte sind idempotent. Nach Absturz setzt Recovery fort. Fällt eine Pflichtschicht aus, bleibt `Partial{outstanding}`; es entsteht **kein** erfolgreicher Beweis. Retry mit Backoff über `Clock`. Fristverfolgung (unverzüglich, spätestens ein Monat, Prüfpunkt P-11) ist Teil des Zustands `Completed`.

### 20.11 Datensparsamer Nachweis und opake Schlüssel `[K]`

Beweis und `ErasedSet` enthalten keine Klartext-IDs, keine Inhalte, keine Embeddings. `subject_hash` wird als Commitment mit mandantenspezifischem Geheimnis in der Hand des Verantwortlichen gebildet (HMAC), nicht als ungesalzener Hash niedrig-entropischer Schlüssel. Sammlungen mit `erasure_required` speichern `HMAC(K_idx, id)` im Speicherschlüssel (Kompromiss: keine Prefix-Scans über Klartext-IDs). Die Aufbewahrungsdauer des `ErasedSet` ist konfigurierbar; Rechtsgrundlage und Frist sind Prüfpunkt P-07. Die Commitment-Form ist `HMAC-SHA256(K_commit[tenant], kanonische Subjekt-Referenz)`; `K_commit` liegt in der Hand des Verantwortlichen, nie beim Anbieter, und ist vom Datenschlüssel getrennt.

### 20.12 Claim-Taxonomie `[K]`

|Claim|Voraussetzung (gemessen oder attestiert)|
|---|---|
|`LogicalSuppression`|Lesesperre aktiv, Black-Box-Probe findet nichts|
|`PhysicalPurge{layer}`|Segment neu geschrieben, Altdatei entfernt, Rohdateiscan ohne Canary, `units_scanned > 0`|
|`CryptoErasureAttested`|Verwahrungsstufe K3 **und** Vernichtungsattestierung des Custodians|
|`CryptoErasureLocal`|Verwahrungsstufe K2; Beweis trägt Warnung `KeyMaterialRemanenceNotExcluded`|
|`Restricted`|Aufbewahrungspflicht/Legal Hold; kein Schlüssel vernichtet; Lesesperre aktiv|

Bei Verwahrungsstufe K1 (und K0) entsteht kein kryptografischer Claim; es bleiben `LogicalSuppression` und `PhysicalPurge`. Unzulässig in Beweis und Außentext: „DSGVO-konform“, „garantiert“, „unwiederbringlich“ ohne `CryptoErasureAttested`, „100 % gelöscht“. Zulässig sind technische Beschreibungen (§4.3). Der Beweis führt `excluded` (nie leer): externe Backups, Schlüsselkopien/Escrow, Prozessspeicher/Swap/Dumps, extern ausgeleitete Daten, Modellparameter, Konsolidierungsergebnisse bis zur Lineage-Kaskade.

### 20.13 Rechtliche Prüfpunkte `[K]`

Die Punkte sind **offene Prüfaufträge** an die Projektverantwortlichen, eine Fachanwältin bzw. einen Fachanwalt für Datenschutzrecht oder die Datenschutzbeauftragte bzw. den Datenschutzbeauftragten. Keiner ist durch die Technik entschieden; bis zur Klärung enthalten Produkt- und Beweistexte keine Aussage, die davon abhängt (INV-CLAIMS-1). Die Belegstärken folgen der Legende in §20.0.

|ID|Prüfpunkt|Hintergrund|Status|
|---|---|---|---|
|P-01|Ist Schlüsselvernichtung eine Löschung nach Art. 17, eine Anonymisierung oder nur eine Einschränkung?|Kein Gesetz definiert Löschung; keine Entscheidung erkennt Crypto-Shredding an; einzige Behördenaussage stammt aus dem Blockchain-Kontext (§20.0).|⚠ offen|
|P-02|Entfällt der Personenbezug nach Schlüsselvernichtung für jede Stelle (relativer Personenbezug)?|EuGH-Rechtsprechung zum relativen Personenbezug; für den Verantwortlichen mit Schlüssel bleiben die Daten personenbezogen.|⚠ offen|
|P-03|Welcher Nachweis belegt, dass kein Schlüsselmaterial verbleibt (KMS/HSM, RAM, Swap, Dumps, Backups des Schlüsselspeichers)?|Alle Schlüsselkopien müssen erfasst sein; lokale Verwahrer können Remanenz nicht ausschließen (§20.15).|⚠ offen|
|P-04|Liegen Quelle, Chunks, Vektoren, Volltextindex, Caches, Logs und Prompt-Historie im Wirkungsbereich?|Alle Repräsentationen müssen erfasst oder als nicht abgedeckt ausgewiesen sein (A2).|[P]|
|P-05|Behandlung von Embeddings; die Spezifikation behandelt sie vorsorglich als personenbezogen.|Keine verbindliche Entscheidung; Behörden sehen die Vektordatenbank als Verarbeitungsgegenstand, Inversionsangriffe sind dokumentiert.|⚠ offen|
|P-06|Zulässige Backup-Frist, Wiedereinspiel-Sperre, separater Backup-Schlüssel.|Die Wiedereinspiel-Sperre (Löschung nach Rücksicherung) ist Behördenposition; konkrete Fristen sind Praxisempfehlung.|[B] Sperre; Frist offen|
|P-07|Welche Nachweis-Metadaten dürfen wie lange bleiben; darf eine Lösch-ID personenbezogen sein?|„Lösch-Paradoxon“: Nachweis versus Rückstandslosigkeit; Rechtsgrundlage und Frist des `ErasedSet` sind festzulegen.|⚠ umstritten|
|P-08|Rolle des Anbieters (kein Datenzugriff, keine Telemetrie, kein Key-Recovery).|Einordnung als Auftragsverarbeiter, Hersteller oder keines von beidem hängt davon ab, ob der Anbieter Zugriff auf Daten oder Schlüssel hat; die Architektur schließt Zugriff aus (kein Telemetrie-Pfad, kein Escrow).|◐|
|P-09|Aufbewahrungspflichten; Archiv getrennt vom RAG-Index; Einschränkung statt Löschung.|Art. 17 Abs. 3 und Art. 18; die Einschränkung eines Index ist ungeregelt.|[R] + ⚠|
|P-10|Werbe- und Produktaussagen (Wettbewerbs- und Vertragsrecht).|Pauschalversprechen sind riskant; nur technische Beschreibungen sind zulässig.|○|
|P-11|Löschfrist „unverzüglich“, Monatsfrist, Obergrenze für Batch-Fenster und Epochengröße.|Art. 17 Abs. 1, Art. 12 Abs. 3; die Spezifikation setzt interne Zielgrößen (§25.6), die DSB bestätigen muss.|[R] ○|
|P-12|Empfängerinformation (Art. 17 Abs. 2, Art. 19) und Auftragsverarbeiter-Nachweis.|Contextra kennt Empfänger nur über Egress- und Audit-Daten (§19); die Mitteilung bleibt Sache des Verantwortlichen.|[R] ○/◐|
|P-13|Krypto-Agilität: Notfallplan bei Algorithmenwechsel, periodische Neubewertung.|Aufsichtsposition im Blockchain-Kontext, als Analogie übertragen.|[B] (Analogie)|
|P-14|Verifikation der mit ◐/○ markierten Fundstellen im Volltext (Urteile, Leitlinien, Normen).|Teile der Recherche beruhen auf Sekundärquellen.|offen|

### 20.14 Abnahmekriterien Löschung `[K]`

Die folgenden Kriterien sind die Tests, an denen REQ-DEL-001 bis REQ-DEL-029 gemessen werden. Eine Löschung gilt erst als bewiesen, wenn alle Kriterien AC-1 bis AC-15 grün sind.

|ID|Kriterium|Methode|
|---|---|---|
|AC-1|Nach Löschung und Beweis enthält **keine** Datei des Datenverzeichnisses (WAL, SSTables, Checkpoints, Temp, `.bak`, Keystore, Journal) den Klartext-Canary (≥ 32 Zufallsbyte als Text), dessen Embedding-Bytes, eindeutige Terms oder die Roh-ID. Chiffretext-Reste sind erlaubt.|Scanner im Test + unabhängiger `grep -r`|
|AC-2|Wird eine Messung durch eine Konstante ersetzt, schlägt mindestens ein Test fehl.|Mutation|
|AC-3|Absturz an jedem Journalzustand: danach `Completed` mit vollständigen Messungen oder `Partial`, nie `Completed` ohne Messung.|Crash-Matrix (`FaultVfs`)|
|AC-4|Restore eines Checkpoints vor der Löschung: Betroffener gesperrt, danach erneut gelöscht, `restore_reapplied` im Journal.|Integrationstest|
|AC-5|Zwischen `Frozen` und `Completed` liefert kein Lesepfad Daten des Betroffenen, auch parallel.|Loom/Property|
|AC-6|Beweis-Negativsuite: Bit-Flip je Feld, Feldvertauschung, Versions-Downgrade, Domain-Verwechslung, anderer Mandant, falscher/widerrufener Schlüssel, fehlendes `excluded` ⇒ Fehler mit benanntem Grund.|Unit + Fuzz|
|AC-7|Mit vollständigem Dump aller persistierten Bytes und aller überlebenden Schlüssel lässt sich das Payload eines gelöschten Datensatzes nicht entschlüsseln (gilt für Betroffenen-/Dokument-genaue Schlüsselvernichtung).|Property|
|AC-8|Neustart nach Löschung: gesperrt/gelöscht, Keystore ohne Spur des DEK, Journal konsistent.|Integrationstest|
|AC-9|Mit `ManualClock` und festen Eingaben ist der Beweis bytegleich reproduzierbar.|Golden-Test|
|AC-10|Das Gate `deletion-honesty` erkennt konstante Messwerte, verworfene Verifier-Ergebnisse, unbedingte `Ok(true)`-Verifikation und öffentliche Konstruktoren mit Zählerwerten in einem absichtlich fehlerhaften Beispiel.|Gate-Selbsttest|
|AC-11|Legal Hold ⇒ `Restricted`, keine Schlüsselvernichtung, Lesesperre aktiv, `basis` und `until` im Beweis.|Integrationstest|
|AC-12|Bei `erasure_required` enthält kein Speicherschlüssel die Klartext-ID.|Rohscan|
|AC-13|Gleichzeitiges `put`/`delete` derselben ID ergibt nach Abschluss nie einen sichtbaren Zustand.|Loom|
|AC-14|`CryptoErasureAttested` steht nur bei Verwahrungsstufe K3 mit Attestierung.|Typ-/Property-Test|
|AC-15|Bei Verwahrungsstufe K0/K1 enthält der Beweis keinen kryptografischen Claim; bei K2 trägt er die Warnung `KeyMaterialRemanenceNotExcluded`.|Property-Test|

### 20.15 Löschpfad: Komponenten, Typen und Platzierung `[K]`

**Entwurfsprinzipien des Löschpfads.** (1) *Messen vor Behaupten:* Ein Claim existiert nur, wenn ein Typ, der nur aus einer Messung konstruierbar ist, ihn trägt. (2) *Fail-closed:* keine Löschung ohne Schlüssel, keine Bescheinigung bei Teilausfall, keine Fallbacks. (3) *Verteidigung in der Tiefe:* Sperre → logische Löschung → kryptografische Vernichtung → physische Bereinigung → Residualmessung → Beweis. (4) *Granularität ist Schlüsseldesign* (§20.2b). (5) *Schlüssel nie neben Chiffretext.* (6) *Ehrliche Grenzen:* Die Art des Verwahrers bestimmt den zulässigen Claim. (7) *Kein Architekturverstoß:* Ports in `ports`, reine Logik in `crypto`, Persistenz in `store`, Orchestrierung in `engine`, Handler dünn. (8) *Datensparsame Evidenz.* (9) *Determinismus:* Zeit nur über `Clock`, IDs aus Zählern und Hashes, Schlüsselmaterial nur aus dem System-RNG.

**Platzierung.**

|Komponente|Crate|Ring|Inhalt|
|---|---|---|---|
|IDs und Fehler: `ErasureId`, `KeyEpoch`, `KeyRef`, `SubjectCommitment`, `ErasureError`|`types`|0|reine Datentypen|
|Ports: `KeyCustodian`, `ErasureParticipant`, `ErasureJournal`, `NotaryPort` (optional), `RetentionPolicy`, `ErasedSet`, `Vfs`, `DurableLog`|`ports`|0|nur Traits|
|Reine Krypto- und Kodierungslogik: `EpochKeyStore`, Beweis (kanonische Kodierung, Signatur, Verifikation), `ResidualReport`, Merkle-Log|`crypto`|0|synchron, I/O-frei, lock-frei|
|Persistenz: atomare Dateien, `DurableAppendLog`, Journal- und Key-Store-Dateien, Rohdatei-Scanner|`store`|1|`fsync`, Temp-Datei und Rename, Verzeichnis-`fsync`|
|Participants: LSM/WAL (`store`), HNSW (`vector`), Volltext (`text`), Graph (`graph`), KV (`kvcache`), Checkpoint (`checkpoint`)|je Crate|0/1|implementieren `ErasureParticipant` aus `ports`|
|Orchestrator: Saga, Lesesperre, Restore-Guard, Legal-Hold-Gate|`engine`|3|verdrahtet Participants über Ports; `crypto` ist für den Löschpfad nicht optional|
|Werkzeug `contextra_erase`/`forget`, Python-Bindung|`mcp`, `py` über `contextra`|4|nur Aufruf und JSON-Abbildung; **keine Beweiskonstruktion**|
|Verifier-Bibliothek, Bericht|`audit-export`|4 `[E]`|`verify_receipt_bundle`, Markdown-Rendering|
|Test-Doubles (Fake-Verwahrer, Fake-Uhr, Fehlerinjektion)|`testkit`|—|nur dort|

Löschtypen existieren genau einmal (§6.10 Regel 5). Ohne den Löschpfad-Orchestrator liefert dessen Konstruktor `Err(CapabilityUnsupported)`, nie einen Stub, der Erfolg meldet (REQ-DEL-027).

**Ports (Auszug; vollständiger Katalog in Anhang A).**

```rust
pub trait KeyCustodian: Send + Sync + 'static {
    fn kind(&self) -> CustodianKind;   // Kms | Hsm | LocalSealed | LocalFile
    fn create_kek(&self, scope: KekScope) -> BoxFuture<'_, Result<KekHandle>>;
    fn wrap(&self, kek: &KekHandle, dek: &SecretBytes<32>, aad: &[u8]) -> BoxFuture<'_, Result<WrappedKey>>;
    fn unwrap(&self, kek: &KekHandle, w: &WrappedKey, aad: &[u8]) -> BoxFuture<'_, Result<SecretBytes<32>>>;
    /// Vernichtet die KEK dauerhaft; der Beweiswert der Attestierung hängt von `kind()` ab.
    fn destroy_kek(&self, kek: KekHandle) -> BoxFuture<'_, Result<DestructionAttestation>>;
}
pub trait ErasureParticipant: Send + Sync + 'static {
    fn layer(&self) -> LayerId;                                       // stabile, maschinenlesbare Schicht-ID
    fn prepare(&self, req: &ErasureRequest) -> BoxFuture<'_, Result<()>>;          // sperren/markieren, idempotent
    fn erase(&self, req: &ErasureRequest) -> BoxFuture<'_, Result<LayerWork>>;     // logisch und physisch, idempotent
    fn measure(&self, req: &ErasureRequest) -> BoxFuture<'_, Result<ResidualReport>>; // Messung am Speicher
}
pub trait ErasureJournal: Send + Sync + 'static {                     // Implementierung: store
    fn append(&self, rec: &JournalRecord) -> BoxFuture<'_, Result<JournalPos>>;   // inklusive fsync
    fn replay(&self) -> BoxFuture<'_, Result<Vec<JournalRecord>>>;
}
```

**Messung als Typ.**

```rust
/// Nur über `ResidualReport::from_scan` konstruierbar. Kein `Default`, kein `From<usize>`.
pub struct ResidualReport {
    layer: LayerId,
    method: ScanMethod,          // RawFileScan | IndexIntrospection | BlackBoxProbe | KeyAbsence
    units_scanned: u64,          // > 0, außer bei KeyAbsence
    residual_units: u64,
    evidence_digest: [u8; 32],   // Hash über Scan-Parameter, Dateiliste, Prüfsummen; kein Klartext
    measured_at: UnixNanos,      // aus `Clock`, nicht vom Aufrufer
}
impl ResidualReport {
    pub fn from_scan(s: ScanOutcome, clock: &dyn Clock) -> Result<Self>;   // lehnt units_scanned == 0 ab
    pub fn is_clean(&self) -> bool { self.residual_units == 0 }
}
```

`ScanOutcome` erzeugen nur Scanner. Der Rohdatei-Scanner liest **alle** Dateien des Datenverzeichnisses (WAL, SSTables, Checkpoints, Temp-Dateien, `.bak`, Keystore, Journal) und sucht nach dem Klartext-Canary, dessen Embedding-Bytes, den eindeutigen Terms und der Roh-ID (AC-1). Damit ist der Fehlertyp „konstante 0“ strukturell ausgeschlossen.

**Kanonische Kodierung des Beweises.** Präfix `b"contextra/erasure-receipt/v4\0"`, dann Felder in fester Reihenfolge als `tag:u8 ‖ len:u32le ‖ bytes`; Optionals haben einen eigenen Tag (nie „weglassen“), Listen `count:u32le` plus Elemente; die Versionskennung steht im signierten Text. Kein `bincode` über Enums. Ed25519 ist deterministisch; mit `ManualClock` und festen Eingaben ist der Beweis bytegleich (AC-9). `verify_external(&VerifyingKey, &TrustPolicy) -> VerificationReport` liefert **Gründe** (Signatur, Domäne, Version, Schlüssel-ID widerrufen, Zeitquelle `System` statt `Notarized`, Verwahrer-Art) statt eines `bool`.

**Verwahrer-Realismus.** Eine lokal gespeicherte, durch einen Hauptschlüssel gewrappte `EpochKek` ist nicht sicher vernichtbar: Das Entfernen der Datei hinterlässt Remanenz (Flash, Copy-on-Write-Dateisysteme, Backups), und der überlebende Hauptschlüssel entwrappt sie. Daraus folgt: (1) `CryptoErasureAttested` nur mit `Kms`/`Hsm`, deren `destroy_kek` eine Attestierung liefert; (2) für `LocalSealed` gilt `CryptoErasureLocal` plus Warnung im Beweis; (3) für `LocalFile` entsteht kein kryptografischer Claim; (4) Schlüssel-Backups sind verboten oder haben eine eigene, kurze Rotation (Lebensdauer ≤ Löschfrist) und erscheinen sonst als `ExcludedScope::KeyMaterialBackups`; (5) Speicherreste (Swap, Core-Dumps, Prozessspeicher) bleiben als `ExcludedScope::ProcessMemoryResidue` deklariert; ihre Härtung (`mlock`, `MADV_DONTDUMP`) liegt in der Unsafe-Insel `sys` und ist nicht Teil der Löschgarantie.

**Verhalten der Altbestandteile.** Schlüsseltypen tragen `Zeroizing<[u8; 32]>` und kein `Clone`; ein vergiftetes Lock ergibt `Err`, nie `bool`. Die Altformate des Beweises (v1–v3) werden nur gelesen. Öffentliche Konstruktoren für Schicht-Belege mit Zählerwerten oder Closures existieren nicht. Löschereignisse stehen in der AuditChain als Einträge mit `subject_commitment`, ohne ID. Das Löschwerkzeug der MCP-Schnittstelle ruft ausschließlich die Fassade; seine Antwort enthält `outcome`, `claims`, `excluded` und den Beweis (oder `null` bei `Partial`/`Restricted`), nie einen Nachweis ohne Messung.

### 20.16 Konformität und Risiken des Löschpfads `[K]`

**Konformität zu Architekturregeln.** Der Löschpfad fügt keine Produktionskante hinzu (Ports in `ports`, Participants hängen nur an `ports`, der Orchestrator kennt sie über die Fassade). `crypto` und `engine` bleiben `#![forbid(unsafe_code)]`; Zeit läuft über `Clock`, Schlüsselmaterial ausschließlich über den System-RNG (nie `thread_rng`); Produktionscode enthält kein `unwrap`/`expect`/`panic!`; Participants ohne Fähigkeit liefern `Err(CapabilityUnsupported)`; Test-Doubles liegen nur in `testkit`; Verschlüsselungstypen existieren unabhängig von Features.

|Risiko|Gegenmaßnahme|
|---|---|
|Epochen-Rotation kostet bei großen Epochen Zeit|Epochen-Größenlimit, Batch-Fenster, asynchrone Rotation hinter der Lesesperre; die Frist bleibt über `Frozen` geschützt.|
|Segment-Rewrite abgeleiteter Schichten erzeugt Latenz und Write-Amplification|Segmentgrößen begrenzen; Messung mit der Write-Amplification-Benchmark-Suite (§26).|
|Opake IDs brechen Prefix-Scans|Nur für `erasure_required`-Collections; dokumentierter Kompromiss; ein Blind-Index ist ein späteres Zusatzfeature.|
|Lokale Verwahrer vermitteln falsche Sicherheit|Claim-Taxonomie (§20.12), Warnfeld im Beweis, `CryptoErasureAttested` nur mit Attestierung.|
|Rechtswertung|Kein Produkttext zur Löschwirkung ohne fachkundige Freigabe (§20.13).|

---

## 21. MCP-Schnittstelle und Betriebsmodell (`contextra-mcp`) `[K]`

### 21.0 Transport

Zum Host: **stdio, JSON-RPC 2.0**; `stdout` gehört ausschließlich dem Protokoll (INV-STDOUT-1), Logging nach `stderr`/Datei. Kein TCP/UDP-Listener.

### 21.1 Primär-/Proxy-Betrieb (INV-IPC-1)

MCP-Hosts starten je Anwendung einen eigenen Serverprozess; mehrere gleichzeitige Hosts sind der Normalfall. Daher: Genau ein **Primärprozess** hält DirLock und Engine. Jeder weitere `contextra-mcp`-Start erkennt den Primär (Lock- und Endpoint-Datei) und arbeitet als **stdio↔IPC-Proxy**. IPC = Unix-Domain-Socket (Verzeichnis 0700, Socket 0600) bzw. Named Pipe mit ACL; die Peer-Credentials (`SO_PEERCRED`/`getpeereid`/`GetNamedPipeClientProcessId`) werden geprüft; der Prinzipal ist (OS-Nutzer, Client-Name) und wird über die Konfiguration einem Tenant zugeordnet. Der Primär startet bei Bedarf automatisch (abgelöst), beendet sich nach `idle_timeout` (Start 15 min) ohne Clients, und fällt er aus, übernimmt der nächste Client nach Recovery. Alle Clients teilen dieselben Garantien (SSI, Audit, Budgets).

### 21.2 Werkzeuge

**3-Tool-Fassade** (Standard für Agenten): `memory_remember`, `memory_recall`, `memory_forget`, mit Defaults (Tenant, Scope, Budget) aus der Konfiguration. **Vollwerkzeugsatz** mit `ToolCategory ∈ {Read, Write, Destructive, Admin, Egress, Plugin}`; Agenten erhalten nur freigegebene Kategorien. MCP-Tool-Annotationen (`readOnlyHint`, `destructiveHint`) werden aus der Kategorie erzeugt.

### 21.3 Sicherheitspipeline (5 Stufen je Aufruf)

(1) Schema-/Größenvalidierung, (2) Authentifizierung/Prinzipal, (3) Autorisierung gegen ToolCategory + Scope + RevocationLog, (4) Budget/Rate/Exfiltrationsprüfung, (5) Audit-Eintrag vor/nach Ausführung.

### 21.4 Nebenläufigkeit und Ausgabegrenzen

Mehrere Anfragen laufen gleichzeitig; `notifications/cancelled`, Timeout und Disconnect lösen kooperativen Abbruch aus (INV-CANCEL-1). Je Prinzipal gilt `max_in_flight`; langlaufende Operationen (`forget`, Reindex, Migration) melden `progress`. `recall` begrenzt die Ausgabe auf `max_response_tokens` (Start 4 000) und kennzeichnet Kürzung ausdrücklich.

### 21.5 Forget-Guard (INV-FORGET-GUARD-1)

`memory_forget` mit einzelner ID und `dry_run` als Default ist direkt zulässig. Selektor-Löschungen, Collection-/Tenant-Scope und Löschungen über `forget_confirm_threshold` (Start 10 Dokumente) liefern `ConfirmationRequired { token, preview }`. Der Token ist **nur über einen Kanal außerhalb des Tool-Aufrufs** einlösbar (`contextra confirm <token>` oder OS-Dialog), nie durch das LLM selbst. Schutz gegen promptinjektionsgetriebene Massenlöschung.

### 21.6 Gespeicherte Injektion (INV-TRUST-1)

Abgerufene Inhalte sind **Daten, nie Anweisungen**. Jeder Eintrag trägt `trust ∈ {user, agent, external}` und `source`; Recall liefert `{content, trust, source, lineage}` strukturiert und rahmt Tool-Text mit einem Datenmarker. Von Agenten oder externen Quellen geschriebene Inhalte werden nicht stillschweigend zu `user`-Vertrauen hochgestuft. Ein `InjectionDetector` `[R]` kann beim Schreiben/Lesen prüfen; ohne ihn bleibt der Marker die einzige Schutzschicht und wird so ausgewiesen.

### 21.7 CLI

`init` (Datenverzeichnis, Schlüssel, Konfiguration), `doctor` (Selbstdiagnose: Ringe, Formate, Verschlüsselung an/aus, KDF-Parameter, Dateisystemtyp/Cloud-Sync-Pfad, Durability-Primitive, Swap/`mlock`, Plattformgrenzen, Kette, Kapazitäten, Wasmtime-Linie), `verify-proof` (offline), `confirm`, `migrate`.

### 21.8 Fehlermodell

Strukturierte Fehlercodes; retrybar (`SerializationFailure`, `Busy`, `StorageFull`) vs. final (`Denied`, `Corrupt`, `ConfirmationRequired`).

---

## 22. Fassade und Composition Root (`contextra`) `[K]`

Die Fassade ist der **einzige** Ort, der konkrete Implementierungen wählt und verdrahtet (P29); `mcp` und `py` rufen nur ihre API. Sie:

1. liest Konfiguration/Tier/Profil (Lizenz `[E]`),
2. instanziiert Ports in Ringreihenfolge (`Vfs`-Implementierung aus `store` → `crypto/Kms` → `store` → Indexe → [`kvcache`] → [`sandbox`] → `engine` → [`cognition`] → [`privacy`]); Rand-Module nur bei aktiviertem Feature,
3. registriert Observer (WAL → Indexe/Audit/CDC), Plugins und injiziert Embedder/Reranker/Generator als Ports in `db`/`engine`,
4. führt beim Start Recovery und einen **Verdrahtungs-Selbsttest** aus (jeder Port hat eine Implementierung, jede Invariante ist aktiv; sonst Start verweigert). **INV-WIRING-2:** Sicherheits-Ports aus `governance/security_ports.toml` haben reale Implementierungen; Beobachtungs-Ports (`MetricsSink`, `ReplayProgressSink`) dürfen Null-Objekte nutzen. Das Gate klassifiziert Ports, führt eine begründete Allowlist und erkennt Accept-All-Implementierungen,
5. exponiert die öffentliche Rust-API (`Contextra::open`, `db()`, `query()`, `remember/recall/forget`, `verify`, `checkpoint`, `shutdown`).

---

### 22.1 Startsequenz B0–B9 `[K]`

Der Start folgt dieser festen Reihenfolge (INV-BOOT-1, REQ-FAC-005). Die Schritte 4–9 entsprechen der Speicher-Teilfolge in §7.3.1.

|Phase|Schritt|Verweigert den Start bei|
|---|---|---|
|B0|Konfiguration, Tier und Profil (Lizenz `[E]`)|ungültiger Konfiguration, Tier-Verletzung|
|B1|Datenverzeichnis: Dateisystemtyp (INV-PLATFORM-1), Verzeichnissperre|nicht unterstütztem Dateisystem, belegter Sperre (INV-IPC-1)|
|B2|Schlüssel: `KmsProvider`, `K_eng`, Verwahrer je Scope und Epoche|fehlendem Verwahrer bei `erasure_required` (REQ-DEL-024)|
|B3|Sidecar-Logs validieren: Magic, Kette, Manifest-Marke (§7.11)|gebrochener Kette, fehlender Datei bei vorhandenen Daten|
|B4|WAL- und Manifest-Recovery (§7.3.1, Schritte 1–5)|Lücke in der WAL-Kette (INV-FAILCLOSED-1)|
|B5|Ausstehende `SidecarIntent`s idempotent anwenden (§7.3.1, Schritt 6)|dauerhaft scheiterndem Intent|
|B6|`ErasedSet` laden und Lesesperre armieren (INV-BOOT-1)|unlesbarem oder manipuliertem `ErasedSet`|
|B7|`ErasureJournal` replayen; Retention-Entscheidungen in `PendingDeletion` überführen (§16.4)|gebrochener Journal-Kette|
|B8|Verdrahtungs-Selbsttest der Fassade (Punkt 4 oben)|fehlender Sicherheits-Port-Implementierung (INV-WIRING-2)|
|B9|Verkehr zulassen|—|

Regeln: (1) Kein Teilstart. Scheitert eine Phase, endet der Start mit benanntem Grund. (2) Lesepfade sind erst nach B6 zugelassen; B9 setzt B6 voraus. (3) Jede Phase ist in der Reihenfolge durch das Gate `boot-order` geprüft.

## 23. Bereitstellungsprofile, Plattform, Lizenz

### 23.1 Deployment-Tiers `[K]`

|Tier|Zweck|Kennzeichen|
|---|---|---|
|**EdgeMinimal**|Embedded/Edge|kleine Caches, SQ8, minimale Hintergrundarbeit; Verschlüsselung optional|
|**PowerUserLocal**|Einzelnutzer-Workstation|volle Funktionen, lokale Inferenz|
|**EnterpriseShared**|Mehrere Mandanten, ein Knoten|strikte Tenant-Isolation, Quoten, RLS|
|**EnterpriseRegulated**|Reguliert (Recht/Medizin/Finanz)|externer KMS, vollständiger Audit, Human-Approval Pflicht `[R]`, Verschlüsselung Pflicht|

### 23.2 PerformanceProfile und Durabilitätsmodi `[K]`

- **Compliance:** maximale Durabilität (Flush pro Commit/kleine Gruppen), Audit synchron. **Balanced:** Group Commit, Standard-Audit. **BareMetal:** größere Gruppen, aggressives Prefetch — nie auf Kosten der Korrektheitsinvarianten.
- **`DurabilityMode ∈ {Full, MemoryOnly}`.** `Full` = WAL + HMAC-Kette + AEAD (Standard; einziger Modus für `Compliance`). `MemoryOnly` hat keine Persistenz und läuft ohne Verzeichnissperre. DirLock und Recovery-Prüfungen sind in `Full` nicht abschaltbar. Einen Modus ohne Integritätskette gibt es nicht; insbesondere existiert kein `WalNoHmac`.

### 23.3 Vertrauensklassen `[E]`

Anfragen werden einer **Vertrauensklasse** zugeordnet (nicht zu verwechseln mit Architektur-Ringen): **Fast** (lokal, minimaler Audit), **Sovereign** (lokal, volle Provenienz, kein Egress), **Compliance** (Egress nur über Privacy-Pipeline, signierte Beweise).

### 23.4 Plattformmatrix (INV-PLATFORM-1) `[K]`

|Plattform|Tier|Besonderheiten|
|---|---|---|
|Linux|1|Referenz; `fdatasync`; `mlock` wirksam|
|macOS|2|`F_FULLFSYNC` Pflicht; `mlock` begrenzt; APFS-Snapshots/Time Machine sind externe Kopien (deklariert)|
|Windows|2|Offene/gemappte Dateien sind nicht löschbar (Handles freigeben, `FILE_SHARE_DELETE`); ACL über `sys`; Virenscanner-Ausschluss für das Datenverzeichnis empfohlen (`doctor`-Hinweis)|

Unterstützte Dateisysteme: lokale ext4/xfs/btrfs/zfs/APFS/NTFS. **Abgelehnt im persistenten Betrieb:** Netzwerk-Dateisysteme (NFS/SMB), unbekannte FUSE-Mounts und Cloud-Sync-Ordner (z. B. OneDrive, Dropbox, iCloud Drive) — `doctor` und Start erkennen sie und verweigern. Jede Plattform hat CI-Läufe für die Durability-Primitive (`Vfs`), S2/S3-Löschung und `mmap`.

### 23.5 Lizenz `[E]`

`SignedLicenseGate`: Ed25519-signierte Lizenz (Tier, Features, Ablauf, Bindung), offline prüfbar; Verstoß/Ablauf ⇒ Degradation auf definierte Basisfunktion, **niemals Datenverlust oder -sperre** (Lesezugriff/Export bleibt). Das Gate dient der Vertragskonformität, nicht dem Manipulationsschutz: Bei öffentlich lizenziertem Quelltext ist es entfernbar; wo Durchsetzung nötig ist, liegen lizenzpflichtige Komponenten in einem nicht öffentlich verteilten Artefakt, und die Prüfung wird nicht als Sicherheitsmerkmal beworben. `SignedActivation { ring, installation_id_hash, expires_at_unix, signature }` mit fester Prüfreihenfolge: (1) niedrigste Klasse (`Fast`) immer `Ok` (**INV-LICENSE-2**); (2) keine Aktivierung ⇒ `NotActivated`; (3) Installationsbindung falsch ⇒ `NotActivated` (kein Informationsleck, **INV-LICENSE-3**); (4) ungültige Signatur ⇒ `InvalidSignature`; (5) `clock.now_unix() ≥ expires_at` ⇒ `Expired`; (6) niedrigere aktivierte Klasse als angefragt ⇒ `NotActivated`; (7) sonst `Ok`.

---

## 24. Daten-Lebenszyklus: End-to-End-Abläufe `[K]`

### 24.1 `remember`
Eingabe → Prinzipal → Tenant aus Transaktion → Limits/Privacy-Prüfung → `trust`/`source` → `MemoryType` (Default `Semantic`) → Chunking → Embedding (Port; Ergebnis als Wert, INV-NONDET-1) → **ein WAL-Batch**: Primärdaten + IndexIntents (Vektor, Text, Graph) + Lineage + Audit → durabler Flush (Commit-Punkt) → Applier wendet Indexe an → Ack mit WAL-Seq (Provenienz-Anker). Große Werte: Blob-Segment vor dem Commit (§7.9).

### 24.2 `recall`
Anfrage → Planner → ScopeConstraint/ACL-Bitset → parallele Modalitäten am Snapshot (ACORN-Vektor, BM25F, [Graph]) → [WASM-Filter] → RRF-Fusion [Kalibrierung/DiBuD] → Top-k-Rerank → Ergebnis mit Explain, Lineage, `trust`. Unter Unsicherheit (Conformal `[R]`) wird **explizit unsicher** ausgegeben. Ausgabe auf `max_response_tokens` begrenzt.

### 24.3 `forget`
Autorisierung (Destructive) und Forget-Guard (§21.5) → S0 Tombstone (sofort, durabel) → bei ganzem Scope **SK** (DEK vernichtet, INV-SHRED-1) → **sofortige Rückgabe** eines `DeletionReceipt` mit ehrlichem Teilbeweis und `PendingDeletion`-Handle → Lineage-Kaskade (INV-DERIVED-1) → S1 → `force_flush` → Memtable-Witness → Purge-Kompaktion → SSTable-Witness → WAL-Scrub → WAL-Witness → `DeletionProof` (Formatversion 4) signiert → in AuditChain verankert → `poll_deletion_proof` liefert den vollständigen Beweis (`Pending`, solange offene Snapshots die Kompaktion verzögern). Idempotent und wiederaufnehmbar nach Crash (Aufgabenzustand im WAL). Außenformulierung nach INV-CLAIMS-1.

### 24.4 Checkpoint/Backup
Konsistenter Snapshot an Sequenzstand; Export verschlüsselt, **ohne Keystore** (Schlüssel-Export als getrenntes Key-Escrow); Wiederherstellung wird **verifiziert** (Kette, Manifest, Stichproben-Lesetests, Format-Korpus). Weil Backups nur Ciphertext enthalten, bleibt nach SK vernichtetes Material auch in älteren Backups unlesbar, solange kein Escrow mit der gelöschten DEK existiert; Escrow-Kopien sind deklarierte Grenze (§4.3).

---

### 24.5 Tenant-Lebenszyklus `[K]`

Ein Tenant durchläuft die Zustände `Aktiv`, `Gesperrt` und `Gelöscht` (REQ-TEN-004).

- **Aktiv → Gesperrt** (Administration, Kategorie `Admin`): keine Schreibzugriffe, Lesen nur für Betreiber mit Protokollierung; Schlüssel bleiben erhalten. Rücküberführung nach `Aktiv` ist nur bei unveränderten Schlüsseln zulässig.
- **Gesperrt → Gelöscht** (`erase_tenant`, Kategorie `Destructive`, Löschsaga §20.10): alle Scopes des Tenants erhalten SK; jede Collection durchläuft S1 bis S3; je Scope wird ein `DeletionProof` erzeugt. Der Tenant wird im `ErasedSet` als gesperrt eingetragen, bevor die Saga beginnt (INV-BOOT-1).
- **Gelöscht** ist endgültig. Die Tenant-ID wird nie wiederverwendet; Versuche, einen gelöschten Tenant zu schreiben, liefern `TenantErased` (final).

Quoten je Tenant gelten nach §16 (REQ-TEN-005).

## 25. Konstanten und Startwerte `[K]`

Werte sind **Startwerte** (zu kalibrieren per Benchmark, §26/§27.4), sofern nicht als **fest** markiert. Jede Konstante hat genau eine kanonische Quelle im Code (`const` bzw. Konfigurationsschema); dieses Dokument wird per Doc-Truth-Gate gegen den Code geprüft (INV-DOC-SINGLE-SOURCE-1). Abweichung ist ein Gate-Fehler, nicht ein Dokumentationsfehler.

### 25.1 Formate und Kryptografie (fest)

|Parameter|Wert|
|---|---|
|Magics|WAL `MFW3`, SSTable `MFSX`, Blob-Segment `MFBL`, Manifest `MFMN`, Keystore `MFKS`; je Datei zusätzlich Formatversion (INV-FORMAT-1)|
|WAL-Integrität|HMAC-SHA256-Kette (Vorgänger-Tag im Record) + AEAD|
|AEAD|AES-256-GCM-SIV; Nonce 96 Bit zufällig, Tag 128 Bit (28 Byte Overhead je Record); AD = `scope_id ‖ key_id ‖ doc_id_hash`|
|Schlüssel|`KEK_root` per Argon2id; `K_eng` per HKDF(Label); Scope-DEK 256 Bit zufällig, gewrappt|
|KDF-Untergrenze|Argon2id m ≥ 64 MiB, t ≥ 3, Kalibrierung auf ≥ 250 ms (INV-KDF-1)|
|Signatur / Hash|Ed25519; SHA-256 (Merkle/Digest), BLAKE3 (Blob-Digest)|
|DeletionProof|Formatversion 4 (`SignatureVersion::V4`); sieben Löschschichten; Stufen S0, SK, S1, S2, S3|
|Unsafe-Inseln|3: `simd`, `sys`, `wire`|
|MCP-Transport|stdio, JSON-RPC 2.0 zum Host; lokales IPC (Unix-Socket/Named Pipe) zum Primärprozess; kein TCP/UDP|

### 25.2 Storage

|Parameter|Startwert|
|---|---|
|`blob_threshold`|64 KiB|
|`l0_slowdown` / `l0_stop`|8 / 20 L0-Dateien|
|Block-Cache|SIEVE hinter Port `BlockCache`|
|`DurabilityMode`|`Full` (Standard), `MemoryOnly`|
|Recovery-Zeitbudget|je Tier in `benchmarks/budgets.toml` (§26)|

### 25.3 Index und Retrieval

|Parameter|Startwert|
|---|---|
|HNSW `m` / `efConstruction` / `efSearch`|16 / 200 / 64|
|`MAX_SEARCH_K`|1000 (auch im Filterpfad)|
|`exact_scan_max` (Planner)|20 000 erlaubte IDs|
|`tombstone_rebuild_ratio`|0,2|
|Adaptiver Overfetch|`3·k → 6·k → 12·k`, harte Scan-Obergrenze|
|BM25 `k1` / `b`|1.5 / 0.75|
|RRF `k`|60 (Tie-Breaker: Score absteigend, DocId aufsteigend)|
|KV-Blockgröße|16 Token|
|RAM-Formel SQ8|`N·D` Byte + Graph `N·R·4` Byte|
|Designpunkt|≤ 10⁷ Vektoren, ≤ 10² Mandanten je Knoten|

### 25.4 Engine, Betrieb, MCP

|Parameter|Startwert|
|---|---|
|Chunking|Ziel 512 / Obergrenze 1024 Token, Overlap 10–15 %|
|`max_document_bytes`|16 MiB|
|`max_chunks_per_document`|4 096|
|`max_batch_ops`|10 000|
|`max_collections_per_tenant`|256|
|`max_in_flight_per_principal`|8|
|`max_response_tokens`|4 000|
|`forget_confirm_threshold`|10 Dokumente|
|IPC `idle_timeout`|15 min|
|Retention-Karenzzeit / Reaper-Batch|7 Tage / 100 Dokumente je Tick|

**Notreserve und Snapshots.** `emergency_reserve_bytes` = 64 MiB (mindestens zwei WAL-Segmente plus Journal, bei Start vorab reserviert, §7.12). `snapshot_max_age` = 15 min (§8, REQ-DEL-031).

### 25.5 Kognition und Rand

|Parameter|Startwert|
|---|---|
|`DECAY_FLOOR`|0,05 (dormant, nie Löschung)|
|`AdaptiveDecayController`|`kappa = 2.0`, `base_half_life_tx = 10_000`|
|Halbwertszeit Episodic / Working|10 000 / 500 Aktivitätsticks|
|`Procedural`-Verstärkungsschwelle|50 Zugriffe|
|`Working`-TTL|50 000 Aktivitätsticks, `working_ttl_wall` 4 h, Sitzungsende|
|`VolatileContextVault`|512 MiB Default-Kapazität|
|`ContextPrefixEngine`|`max_document_chars` 8000, `max_prefix_tokens` 80|
|`GaspValidator`-Schwelle|0,70|
|`MultiStepEngine`|`max_rounds` 3, `latency_budget_ms` 100|

### 25.6 Löschpfad

|Parameter|Startwert|Herleitung / Kalibrierung|
|---|---|---|
|Batch-Fenster der Epochen-Rotation|15 min|Interne Zielgröße, deutlich unter „unverzüglich“ und der Monatsfrist (Art. 12 Abs. 3); die Obergrenze ist nie die gesetzliche Frist. Bestätigung durch die DSB: Prüfpunkt P-11.|
|Obergrenze Schlüssel je Epoche `N_epoch`|4096|Re-Wrap-Zeit `N_epoch · L_wrap` MUSS höchstens das halbe Batch-Fenster betragen (`L_wrap` = gemessene Wrap-Latenz des Verwahrers); der Startwert ist per Benchmark je Verwahrer-Art zu kalibrieren.|
|Retry-Backoff der Saga|exponentiell, Start 1 s, Deckel 5 min, Zeit über `Clock`|Begrenzt Last bei Teilausfall; deterministisch testbar mit `ManualClock`.|
|Aufbewahrung des `ErasedSet`|konfigurierbar, Standard **nicht** unbegrenzt|Rechtsgrundlage und Frist offen (P-07); Mindestdauer ist die Restore-Guard-Wirksamkeit gegen vorhandene Backups.|
|Canary|mindestens 32 Zufallsbyte als Text|Eindeutigkeit gegen Zufallstreffer im Rohdatenscan (AC-1).|
|Lebensdauer von Schlüssel-Backups|höchstens die konfigurierte Löschfrist|§20.15 Verwahrer-Realismus.|

---

## 26. Messbare Verträge und Leistungsziele `[K]`

Jeder Vertrag ist ein CI-prüfbares Kriterium. Absolute Zahlen stehen je Tier in `benchmarks/budgets.toml` (eine Quelle); Veröffentlichung folgt §3 Punkt 4: Commit-Hash, Modell-Hash, Hardware, Workload, Vergleichsbasis, Konfidenzintervall.

|Vertrag|Kriterium|
|---|---|
|**Durabilität**|0 verlorene bestätigte Commits über alle injizierten Crash-Punkte (§27.3), je Plattform der Matrix §23.4.|
|**Konsistenz**|0 Divergenz Primärdaten ↔ Indexe nach Recovery (INV-ATOMIC-IDX-1); Index-Rebuild aus dem WAL ist bit-identisch (INV-INDEX-APPLY-1).|
|**Isolation**|0 Cross-Tenant-Leaks in Property- und Fuzz-Tests (Daten, Scores, Statistiken, Timing-Klassen); Latenz eines Mandanten hängt nicht von der Last anderer ab (§9.2).|
|**Time-to-Context (TTC)**|p50/p99/p99.9 mit 95-%-Konfidenzintervall je Tier und Last (kalt/warm, gefiltert/ungefiltert); Regression über Schwelle bricht die CI (§27.4).|
|**ANN-Recall**|Recall@k je Konfiguration ≥ Zielwert (Start: Recall@10 ≥ 0,95 bei Standard-`efSearch`); gefilterte Suche liefert exakt `k` oder alle passenden Treffer (§9.8).|
|**Retrieval-Qualität**|Recall@5 auf einem öffentlichen Gedächtnis-Benchmark (z. B. LoCoMo-10) mit reproduzierbarem Harness; Abweichungen werden per Root-Cause erklärt, nicht umgangen.|
|**Recovery-Zeit**|Zeit bis „Lese-/Schreibverkehr zugelassen“ ≤ Budget je Tier bei definierter WAL-Länge; Messgröße ist die Checkpoint-Frequenz (§7.3.1).|
|**Löschung**|Jeder `forget` liefert sofort einen ehrlichen Teilbeweis und nach Abschluss der Purge-Schritte den vollständigen Beweis (oder `Pending` mit Grund). Nach einem Crash an **jedem** Punkt der Löschkette gilt: Ein Scope mit durablem `KeyDestroyRecord` ist unlesbar (INV-SHRED-1), und die Kette setzt idempotent fort. Die Offline-Verifikation ist erfolgreich oder schlägt deterministisch fehl; der Beweis behauptet nur belegte Schichten.|
|**Write-Pfad**|Durchsatz und Latenz je `PerformanceProfile` inkl. Group-Commit-Kurve; Write-/Space-Amplification der Kompaktion (§7.4) innerhalb des Budgets; kein unbegrenzter Speicherzuwachs unter Dauerlast (Backpressure, §7.3).|
|**Speicher**|RAM-Bedarf folgt der Formel aus §25.3 ± Toleranz; unter Speicherdruck wird verkleinert/gedrosselt, nie OOM-Kill (§7.6).|
|**M1 (Rand)**|Break-even-Gate des KV-Caches mit realem Backend (§13); ohne bestandenes Gate wird M1 nicht behauptet.|
|**Vergleich**|Pro Leistungsaussage mindestens eine fremde Baseline unter identischem Workload (§27.4); ohne Baseline gilt die Zahl als nicht belegt.|

---

## 27. Qualitätssicherung, Tests, Gates `[K]`

### 27.1 Testpyramide und Abdeckungsmatrix

Unit · Property (proptest) · Modell-basiert (Referenzmodell vs. Engine) · Integration über die Fassade (P29) · Fuzz · loom · miri · Mutation · Chaos (§27.3) · Benchmarks (§27.4).

**Fuzz-Matrix.** Jede Eintrittsstelle für nicht vertrauenswürdige oder persistierte Bytes hat mindestens ein Fuzz-Target; ein Gate vergleicht die Matrix mit den Eintrittsstellen im Code:

|Eintrittsstelle|Target|
|---|---|
|WAL|Replay aller Formatversionen, Kettenprüfung, Recovery-Abschnitt|
|SSTable / Blob|Reader, Footer, Index-Block, Bloom-Filter, Blob-Kopf|
|Manifest / Keystore|Manifest-Decoder, `KdfHeader`, `KeyRegistry`-Log|
|Audit / Revokation / Beweis|`AuditChain`- und `RevocationLog`-Dekoder, `verify_external` des DeletionProof|
|Wire|FlatBuffers-/Layout-Decoder (`wire`)|
|MCP / IPC|JSON-RPC, Tool-Argument-Schemas, IPC-Frames des Proxys|
|Text|Analyzer/Tokenizer, Chunker|
|Plugins `[R]`|Modul-Admission (Wasm-Bytes, Signatur-Container)|
|Extraktoren `[R]`|jeder Dokumentformat-Extraktor (§10.3)|

Fuzz- und Miri-Läufe sind **Pflicht-Gate** (nicht optional oder nur nightly); „SKIPPED“ gilt als rot. Fuzz-Korpora liegen im Repository und laufen als Regression bei jedem PR.

**loom** deckt ein **Inventar** nebenläufiger Strukturen ab, nicht nur zuletzt geänderte Dateien: SnapshotRegistry, TxBuffer, ReadSet/SSI-Validator, Group-Commit (Leader/Follower), Applier-Übergabe, RevocationLog, KeyRegistry, Key-Sperren, HNSW-Arena (Slot-Wiederverwendung), Block-Cache. Neue nebenläufige Struktur ohne loom-Modell bricht das Gate.

**miri** läuft für alle Unsafe-Inseln gegen die sichere Referenz und für jeden Test, der Zeiger-/Layout-Logik berührt.

**Mutation:** Überlebende Mutanten in `[K]`-Crates sind Defekte oder begründete Ausnahmen (Liste im Repository); Mutation-Score je Crate steht in `governance/ratchet.toml` und darf nur steigen.

**Testdichte** ist risikoproportional: Unsafe-Inseln, `store`, `crypto` und `engine` (Commit-Pfad) liegen über dem Workspace-Median.

### 27.2 Red→Green-Disziplin

Jede Änderung beginnt mit einem fehlschlagenden Test, der die Anforderung oder Invariante ausdrückt; dann minimale Implementierung; dann Refactoring unter grünem Baum. Tests werden aus Spezifikation und Invarianten abgeleitet (§28.3). Ein Test, der erst nach der Implementierung geschrieben wurde, belegt eine Invariante nur, wenn Mutation seine Wirksamkeit zeigt.

### 27.3 Chaos / Jepsen-Stil

`FaultVfs` injiziert seed-reproduzierbar an **jedem** Schreib- und Durability-Punkt (§7.8): Crash, Torn Write, `fsync`-Fehler (INV-FSYNC-1), ENOSPC (INV-ENOSPC-1), `EMFILE`, Bit-Flips, Reorder von Schreibvorgängen vor dem Flush, Rechtefehler. Weitere Nemesis-Szenarien: Uhrsprünge über den `Clock`-Port, Absturz des IPC-Primärprozesses mit Übernahme (INV-IPC-1), konkurrierende Clients, Windows-Handle-Semantik beim Entfernen (INV-PLATFORM-1), Abbruch während Migration und Löschkette.

Nach jedem Szenario laufen Recovery und der Invarianten-Checker: keine verlorenen Acks, Kette gültig, Indexe konsistent, SSI-History serialisierbar, **nach `forget` ist nichts Gelöschtes lesbar** (Scanner je Schicht), Schlüsselvernichtung durabel oder nicht erfolgt (kein Zwischenzustand). Fehlschläge sind mit Seed exakt replizierbar.

### 27.4 Benchmarks

Baselines sind versioniert im Repository; Vergleich pro Commit; Überschreiten der Schwellen in `benchmarks/budgets.toml` bricht die CI (Startwerte: p50 > 5 %, p99 > 10 % Verschlechterung gegenüber der Baseline). Messmethodik: festgelegte Hardware-Klasse, fixierte CPU-Frequenz/Affinität, Warmup, ≥ 30 Wiederholungen, 95-%-Konfidenzintervall, kalter und warmer Cache getrennt.

**Pflicht-Benchmarks (Mindestumfang, über den Workspace verteilt):** TTC end-to-end je Tier; ANN-Recall/Latenz (ungefiltert, gefiltert, Mandantenmix); BM25/BM25F; RRF-Fusion; Schreibpfad inkl. Group-Commit-Kurve; Recovery-Zeit; Kompaktion (Write-/Space-Amplification); Löschkette (SK, S1–S3 Dauer); Block-Cache; SIMD-Kernel gegen skalare Referenz; KV-Cache-Break-even `[R]`; Speicherbedarf.

**Fremde Baselines unter identischem Workload (Pflicht für Vergleichsaussagen):** KV/LSM gegen RocksDB und redb; ANN gegen hnswlib bzw. faiss und usearch; Text gegen eine etablierte Volltext-Implementierung (z. B. SQLite-FTS5 oder Tantivy); kombiniert gegen eingebettete Vektor-Datenbanken der Zielgruppe (z. B. sqlite-vec, LanceDB). Ein Benchmark, der nur Contextra misst, ist ein Regressions-, kein Vergleichsbenchmark. Bandit-Regret `[E]` wird nur bei aktiviertem `router` gemessen (INV-ROUTING-1).

### 27.5 Gates (xtask / xtask-heavy)

Der Gate-Katalog steht in `governance/gates.toml` (Name, Phase, lokal/CI-Pflicht, Risikoklasse, Besitzer, Fixture). **Compiler vor Heuristik** (§3 Punkt 11): Ring-, Kanten- und Feature-Prüfungen laufen über `cargo metadata` mit aufgelösten Features; öffentliche API über `cargo public-api`; `unsafe` über den Compiler-Lint-Satz. Text-/Regex-Audits sind nur Ergänzung.

**Pflicht-Gates:** Format · Clippy mit Lint-Satz (s. u.) · Ring-/DAG-/Kern-Isolation (P1, P5, P30, INV-CORE-ISOLATION-1) aus `capabilities.toml` · Unsafe-Inventar und SAFETY-Prüfung (generiert/handgeschrieben getrennt, **keine Ausnahme nach Dateinamen**) · Panic-/Determinismus-Baseline als Ratchet (`governance/*-baseline.toml`, darf nur sinken) · Doc-Truth (Konstanten, Ringe, IDs) · Wired-Check (P29, INV-WIRING-2 mit `governance/security_ports.toml`, Accept-All-Erkennung) · `test-utils`/Mock nie in Release-Features (INV-EMBED-1) · Bypass-Konstruktoren der Löschbeweise (INV-WITNESS-MINT-1) · geschützte Pfade (§28.2) · Format-Kompatibilitätskorpus (INV-FORMAT-1) · cargo-deny/-audit/-vet, SBOM (§28.4) · Fuzz-Smoke/-Regression, Miri, loom, Mutation · Chaos-Suite · Benchmark-Regression · Reproducible-Build · Merge-Ergebnis-Build (§28.2).

**Lint-Satz (Workspace, `deny`):** Rust: `unsafe_code`, `unsafe_op_in_unsafe_fn`, `unused_must_use`. Clippy: `undocumented_unsafe_blocks`, `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `unreachable`, `indexing_slicing`, `exit`, `dbg_macro`, `await_holding_lock`, `await_holding_refcell_ref`, `cast_possible_truncation`, `cast_sign_loss`; `disallowed_methods`/`disallowed_types`/`disallowed_macros` für die P28-Liste (`SystemTime::now`, `Instant::now`, `thread_rng`, `HashMap`/`HashSet` mit `RandomState` in Kernlogik) und für `assert!`-Familie in Produktionscode; `print_stdout` im MCP-Binär (INV-STDOUT-1). Ausnahmen nur als begründetes, lokal begrenztes `#[allow]`; das Gate zählt sie (Ziel 0). `overflow-checks = true` für Kern-Crates im Release-Profil (P7).

**Meta-Gate:** Jedes Audit-Skript hat einen Selbsttest mit synthetischem Repository (bekannte Falschmeldungen als Regressionstests) und läuft in der CI. Grenzwerte dürfen nicht auf Dummy-Werte gesetzt werden, die ein Gate faktisch abschalten. Als „generiert“ oder „nicht bewertet“ ausgenommene Kategorien (z. B. generierter `unsafe`-Code) werden separat geprüft und ausgewiesen. Gate-Läufe sind auf einem zweiten, von der Agenten-Infrastruktur unabhängigen Runner ohne Agenten-Cache reproduzierbar; rohe Testprotokolle werden archiviert.

### 27.6 Durchsetzungsmatrix (Invariante → Mechanismus)

**T** = Typsystem · **G** = Gate · **P** = Property/Modelltest · **I** = Integrationstest über Fassade · **C** = Chaos · **F** = Fuzz · **L** = loom · **B** = Benchmark. Jede `INV-*` des Kerns hat mindestens einen Mechanismus, der ihre Verletzung rot färbt; die Matrix führt jede `INV-*` dieses Dokuments einzeln, das Register in §5.2 weist die Zuordnung je ID aus.

|Invariante|Mechanismus|
|---|---|
|INV-CORE-ISOLATION-1, P1, P5, P30|G (`cargo metadata`, aufgelöste Features)|
|INV-KMS-1, INV-WIRING-2, INV-DELETION-PORTS-1|G (Port-Klassifikation, Accept-All-Erkennung) + I (Selbsttest beim Start)|
|INV-DELETION-WIRE-1, INV-PROOF-COVERAGE-1|T (Typestate-Builder) + I|
|INV-WITNESS-MINT-1|T (private Felder) + G (Aufrufer-Regel)|
|INV-PROOF-KEY-1|T + P (Schlüsselableitung deterministisch/domänengetrennt)|
|INV-SHRED-1|C (Crash an jedem Punkt der Kette) + I|
|INV-SSI-ACTIVE-1, INV-INDEX-SNAPSHOT-1|P (Referenzmodell) + C (History-Checker) + L|
|INV-ATOMIC-IDX-1, INV-INDEX-APPLY-1, INV-WAL-SSOT-1|C (Recovery) + P (Rebuild bit-identisch)|
|INV-NONDET-1|P (Replay ohne Quellenaufruf) + G (Quelle hinter Port)|
|INV-RING0-DETERMINISM-1, P28|G (`disallowed_*`) + P (Wiederholbarkeit)|
|INV-TENANT-1, INV-TENANT-IDX-1, INV-KEYSPACE-1|T (Schlüsseltypen) + P/F (Cross-Tenant) + G (Dateinamen/IDs ohne Klartext)|
|INV-DERIVED-1|I (Lineage-Kaskade) + P|
|INV-ENCRYPT-REST-1/-2, INV-KDF-1|I + G (kein Klartext im Datenverzeichnis, Scanner) + P (KDF-Parameter)|
|INV-FSYNC-1, INV-ENOSPC-1, INV-DURABILITY-PRIM-1|C (`FaultVfs` je Plattform)|
|INV-IO-1|G (`mmap`-Nutzung) + C (Medienfehler → typisierter Fehler)|
|INV-FORMAT-1|G (Kompatibilitätskorpus) + F|
|INV-LOCK-ORDER-1, INV-HNSW-SLOT-1|L + G (`await_holding_lock`)|
|INV-EMBED-1/-2|G (Features) + I (Modellwechsel abgewiesen) + CI-Semantiktest|
|INV-CLAIMS-1|G (Zusicherungen aus Garantiestufe generiert)|
|INV-PROVENANCE-1|G (Merge-Queue, §28.2)|
|INV-PLATFORM-1|CI-Matrix §23.4 (Durability-Primitive, S2/S3, mmap) + C|
|INV-IPC-1, INV-CANCEL-1, INV-STDOUT-1|I + C + G (`print_stdout`)|
|INV-FORGET-GUARD-1, INV-TRUST-1|I (Prompt-Injection-Szenarien) + P|
|INV-LOG-1|G (Log-Scanner auf Nutzinhalte) + P|
|INV-DOC-SINGLE-SOURCE-1|G (Doc-Truth, ID-Eindeutigkeit)|
|INV-FAILCLOSED-1|C (Fehlerinjektion an jeder Schnittstelle) + I|
|INV-RETENTION-1|P (Decay löscht nie) + I (Dry-run, Karenz)|
|INV-EVIDENCE-1|G (`cargo xtask verify`, Evidenzdatei je Gate; ein nicht ausgeführtes Gate ist rot)|
|INV-ORACLE-1|P (Referenzmodell, Brute-Force, Verifier anderer Sprache, Canary-Scanner; §27.8)|
|INV-SCAN-1|G (`public-api`-Gate, REQ-ARCH-014)|
|INV-SIDECAR-1|G (Sidecar-Abschluss: keine Persistenz außerhalb §7.11) + C (Crash zwischen WAL-Commit und Sidecar-Anwendung) + I|
|INV-EMERGENCY-DEL-1|C (ENOSPC und `fsync`-Fehler während `forget`) + I (Löschen im Zustand `ReadOnly`/`Poisoned`)|
|INV-ASOF-ERASE-1|P (Referenzmodell: `as_of` nach Löschung) + L (Snapshot gegen laufende Löschung)|
|INV-BOOT-1|I (Start mit fehlender Phase, mit manipuliertem Sidecar, mit unvollständigem `ErasedSet`) + C (Absturz in jeder Phase)|
|INV-KEYPURPOSE-1|G (Zweckregister `governance/key_purposes.toml` gegen HKDF-Labels im Code) + P (Domänentrennung)|
|`[R]`: INV-ADMISSION-1, INV-APPROVAL-1, INV-WASM-DET-1|T + P (x86-64/aarch64 bit-identisch) + F (Admission)|
|`[R]`: INV-GASP-1, INV-MEMTYPE-1/-2/-3, INV-DECAY-1, INV-EGRESS-BUDGET-1|I + P|
|`[E]`: INV-LICENSE-1/-2/-3|P (Prüfreihenfolge, Degradation ohne Datenverlust)|
|`[E]`: INV-ROUTING-1/-2/-3|OFFEN (nicht normativ, REQ-EXP-001)|

### 27.7 Evidenzstufen `[K]`

|Stufe|Bedeutung|Zulässige Aussage|
|---|---|---|
|**E0**|Behauptung (Bericht, Agentenaussage, Dokument)|„Ziel“, „Hypothese“|
|**E1**|Am Code gelesen (Datei:Zeile, Review)|„im Code vorhanden/abwesend“|
|**E2**|Gate-/Skriptlauf auf bekanntem Commit, Protokoll archiviert|„Gate grün/rot auf Commit X“|
|**E3**|Test- oder Chaos-Lauf reproduzierbar auf unabhängigem Runner|„Verhalten belegt“|
|**E4**|E3 plus unabhängiges Orakel (§27.8), Mutation-Wirksamkeit und externer Review|„Garantie belegt“ (Voraussetzung für `maturity = stable` und für Release-Aussagen)|

`maturity` wird aus der niedrigsten Stufe der Mechanismen einer Crate-Garantie berechnet, nicht behauptet (§3 Punkt 13). Berichte und Audits haben höchstens E0 bis E1, bis ein Lauf sie bestätigt.

### 27.8 Orakel und deterministische Simulation `[K]`

Je Kerngarantie G1–G4 (§4.4) existiert ein Prüfmaß, das nicht aus denselben Datenstrukturen und nicht vom selben Erzeuger stammt wie der geprüfte Pfad (INV-ORACLE-1): **G1** Referenzmodell der WAL-Semantik und Crash-Simulation mit `FaultVfs` (seed-reproduzierbar); **G2** Canary-Scanner (§27.9) und Referenz-Verifier in anderer Sprache; **G3** Referenzimplementierung der Kette; **G4** Brute-Force-Referenz für ANN/ACL und History-Checker für SSI. Eine Simulation ist deterministisch: gleicher Seed ⇒ gleiche Fehlerfolge; Fehlschläge sind mit Seed replizierbar.

### 27.9 Canary-Läufe `[K]`

Ein Canary-Lauf legt Daten mit eindeutigen, zufällig erzeugten Markern an (Klartext ≥ 32 Byte, davon abgeleitete Embedding-Bytes, seltene Terms, Roh-IDs), führt die Löschkette aus und durchsucht danach **jede Datei** des Datenverzeichnisses einschließlich Checkpoints, Temp-Dateien, `.bak`, Keystore und Journal. Treffer im Klartext sind Defekte; Chiffretext darf verbleiben. Der Lauf ist öffentlich reproduzierbar (Befehl, Seed, Commit) und ergänzt ein unabhängiges Werkzeug (z. B. `grep -r`) als zweiten Zeugen (AC-1).

### 27.10 Verifikationsbefehl `[K]`

`cargo xtask verify` führt alle Pflicht-Gates (§27.5) in definierter Reihenfolge aus und erzeugt eine maschinenlesbare Evidenzdatei je Gate: Befehl, Toolchain-Version, Commit, Exit-Code, Dauer, Ausführungszahlen (Tests gelaufen/übersprungen), Hash der Protokolle. Ein nicht ausgeführtes Gate ist rot. Jede Garantie in §4.4 verweist auf diesen einen Befehl (INV-EVIDENCE-1, REQ-QA-001).

---

## 28. Governance und Entwicklungsprozess `[K]`

**Verfassung.** Prinzipien (§5) und Vetos (§4.1) sind maschinenlesbar versioniert (Verfassungsdatei mit Prinzip- und Veto-IDs, `capabilities.toml`); Gates erzwingen sie. Änderungen an Vetos oder Invarianten benötigen eine ADR (`docs/decisions/`) mit Begründung und Alternativen sowie eine ausdrückliche, signierte Freigabe.

**Agenten-Entwicklung.** Autonome Agenten arbeiten gegen Aufgaben mit Akzeptanzkriterien in ressourcenbeschränkten Läufen: ein Zyklus = ein Rot→Grün-Schritt, ein Scope. Ein Ergebnis wird nur akzeptiert, wenn alle Gates auf dem **Merge-Ergebnis** grün sind. Keine Zyklusergebnisse mit Teilverdrahtung (P29).

### 28.1 Release-Kriterien (Version 1.0)

Ein Release trägt nur Aussagen, die durch eines dieser Kriterien belegt sind; alle Kriterien wurden **vor** der Messung festgelegt:

1. Alle Pflicht-Gates (§27.5) sind grün auf einem unabhängigen Runner.
2. Mindestens ein schriftlicher Review von `store` (WAL, Recovery, Kompaktion) und `crypto` (Löschbeweis, KeyRegistry, RevocationLog) durch eine Person außerhalb der Agenten-Pipeline.
3. Messbare Verträge aus §26 erfüllt, inkl. reproduzierbarem Recall@5 und Vergleich gegen fremde Baselines.
4. Keine kritischen oder hohen Advisories in erreichbarem Code; Wasmtime auf einer vom Hersteller unterstützten Linie (§28.4).
5. Gepflegtes `CHANGELOG.md` (Keep-a-Changelog); Kompatibilitätskorpus und Migrationstest (§7.2).
6. Juristisch geprüfte Außenformulierung zur Löschung (INV-CLAIMS-1) und zu Compliance-Vorlagen.
7. Soak-Test ohne Panik und Korruption über einen vorab festgelegten Zeitraum (Start: 30 Tage) auf allen Tier-1/2-Plattformen.
8. Kontrollierter Soft-Launch: Nutzer außerhalb des Entwicklungsteams testen die MCP-Schnittstelle, ungefiltertes Feedback.

`[R]`- und `[E]`-Crates sind von Release-Aussagen ausgeschlossen, solange sie nicht ausdrücklich in die Kriterien aufgenommen sind.

### 28.2 Scope-Treue, Merge-Queue und Provenienz (INV-PROVENANCE-1)

Jeder Merge auf `main` trägt eine Aufgaben-ID und einen deklarierten Pfad-Scope und läuft **serialisiert** über eine Merge-Queue, die das **Merge-Ergebnis** (nicht nur den PR-Stand) baut und testet (`cargo check --workspace --all-targets --locked`, Gates). Das Gate blockiert **vor** dem Merge:

- Änderungen außerhalb des deklarierten Scopes;
- Commits, deren Diffstat nicht zum Titel passt (z. B. ein `fix(x)`-Titel mit Löschung hunderter Zeilen in fremden Crates);
- Änderungen an **geschützten Pfaden** (`governance/protected-paths.toml`: CI-Konfiguration, Gates, Audits, Kompatibilitätskorpus, Verfassung) ohne ADR-Verweis und Freigabe;
- das Entfernen von Tests, Audits oder Gates;
- eine Merge-Basis, die älter ist als der letzte Merge in denselben Dateien (stille Rückgängigmachung); `detect_silent_reverts` läuft als Pflicht-Gate auf `Basis..PR`, ein Fund der Stufe HOCH verlangt manuelle Freigabe.

Commit-Nachrichten sind beschreibend (kein generisches „Shell-Commit“). `main` kompiliert zu jedem Zeitpunkt.

### 28.3 Unabhängige Verifikation

Verifikation ist vom Erzeuger getrennt: Tests werden aus Spezifikation und Invarianten abgeleitet (§27.2), Mutation belegt ihre Wirksamkeit, ein Mensch prüft das **rohe** Protokoll, bevor ein Ergebnis als „verifiziert“ gilt. Berichte von Agenten sind Hinweise, keine Belege. Für `store` und `crypto` ist vor 1.0 ein externer Review Pflicht (§28.1).

### 28.4 Supply-Chain-Inventar

- Advisory-Ignorierung (`deny.toml`) nur mit Einzelbegründung, Erreichbarkeitsanalyse und Ablaufdatum; Sammel-Ignores sind unzulässig; kritische/hohe Advisories werden nicht ignoriert.
- Workspace-weite `tokio`-Features `full` sind unzulässig; Feature-Listen gelten je Crate.
- Doppelte Versionen derselben Abhängigkeit (`cargo tree -d`) unterliegen einem Budget; Zuwachs bricht das Gate.
- Nicht mehr gewartete Abhängigkeiten (z. B. Serialisierungs- oder Dateisperr-Crates) werden mit Ersatzplan geführt; persistente Formate hängen nie von ihnen ab (INV-FORMAT-1).
- Native Drittabhängigkeiten (C/C++-Build-Skripte, z. B. über Tokenizer-Bibliotheken) und ihre `unsafe`-Anteile werden inventarisiert; „Pure Rust“ gilt nur für Pfade ohne solche Abhängigkeiten und wird aus dem Inventar erzeugt.
- Sicherheitskritische Laufzeiten (Wasmtime) bleiben auf der jeweils gepflegten Linie (§14.3); eine nicht mehr gepflegte Linie ist ein Release-Blocker.
- Gepinnte Abhängigkeiten, reproduzierbare Builds, SBOM, Lizenzprüfung (deny), cargo-vet-Audit-Trail.

### 28.5 Dokumentationsmodell

Genau eine Quelle je Tatsache (INV-DOC-SINGLE-SOURCE-1): Ring, Reifegrad, Dev-Ausnahmen und Crate-Zuordnung in `capabilities.toml`; Prinzip- und Veto-IDs in der Verfassungsdatei; Konstanten im Code; Architekturentscheidungen in `docs/decisions/`; diese Spezifikation ist die Quelle der **Soll-Architektur**. Status- und Tagesberichte sind generierte Projektionen und nie Architekturquelle. Der Implementierungsplan (Dokument B) ist eine begleitende Arbeitsunterlage: Er darf Status gegen einen benannten Commit führen (Evidenzstufe E1, §27.7), aber weder Soll-Aussagen ändern noch Anforderungen einführen. Abgeleitete Texte werden generiert oder per Doc-Truth-Gate geprüft.

### 28.6 ADR-pflichtige Entscheidungen

Folgende Punkte ändern Garantien oder Formate und dürfen nur per ADR geändert werden: Schlüsselgranularität (`subject_scope`, §20.2); Auslagerung von `router`/`agent`/`license`/`audit-export`/`avv-generator` in eigene Repositories (§6.0); Aktivierung von RaBitQ/DiskANN (§9.7); Anhebung eines Crates in den Kern (§6.0); Änderung der Plattform-Tiers (§23.4); Änderung eines persistenten Formats (INV-FORMAT-1).

---

## 29. Erweiterungsmodell (Ökosystem-Verträge) `[K]`

Erweiterungen hängen ausschließlich an Ports aus `contextra-ports`; **registriert und verdrahtet wird nur durch die Fassade** (P29). Kern-Crates kennen Erweiterungen nicht (P30).

|Erweiterungspunkt|Port|Art|Hot-Path|Klasse|
|---|---|---|---|---|
|Merge-Semantik|`MergeOperator`|Rust / WASM|Schreib-/Kompaktionspfad|K (Rust), R (WASM)|
|Distanzmetrik|`Metric`|Rust generisch / WASM nur Rerank|Rust ja, WASM nein|K / R|
|Text-Analyse|`Analyzer`|Rust|Ingestion/Query|K (Standard), R (`GermanAnalyzer`)|
|Filter / Policy|`FilterModule`|WASM|Vorstufe, nicht Traversierung|R|
|WAL-Observer / CDC|`WalObserver`|Rust / WASM|asynchron, Cursor-basiert, inhaltsfrei|K / R|
|Embedder / Reranker / Generator|`Embedder`, `Reranker`, `Generator`|Rust-Backend|Ingestion/Rerank|R|
|Schlüsselverwaltung|`KmsProvider`|Rust-Adapter (lokal, HSM/Cloud)|Schlüsselpfad|K|
|Block-Cache|`BlockCache`|Rust|ja|K|
|Kompaktionsplaner|`CompactionPlanner`|Rust|Hintergrund|K|
|Zeit / Zufall / IDs|`Clock`, `Rng`, `IdGenerator`|Rust|überall|K|
|Modul-Verifikation|`ModuleVerifier`|Rust|Admission|R|
|Erkennung / Injektion|`EntityRecognizer`, `InjectionDetector`|Rust|Schreib-/Egress-Pfad|R|
|Egress-Kanal|`EgressChannel`|Rust (`GuardedPayload`)|extern|R|
|Routing-Strategie|`RoutingStrategy`|Rust|Entscheidungspfad|E|
|Scratchpad-Invalidierung|`ScratchpadCacheInvalidator`|Rust|Agentenlauf|E|

**Vertrag jeder Erweiterung:** deterministisch (P28), budgetiert, auditiert, widerrufbar, mit **Gesetz-/Konformitätstestsuite**, die vor Registrierung bestanden werden muss. Sicherheits-Ports unterliegen INV-WIRING-2. Port-Signaturen sind semver-geprüft (`cargo public-api`); eine inkompatible Änderung verlangt eine neue Port-Version und eine Migrationsregel.

---

## 30. Sicherheitsmodell `[K]`

**Vertrauensgrenzen:** Prozessgrenze (stdio zum Host, lokales IPC mit Peer-Credentials), Mandantengrenze (Schlüssel + Index + Präfix + ACL), Sandbox-Grenze (WASM) `[R]`, Egress-Grenze (Privacy-Pipeline) `[R]`, Dateisystemgrenze (Datenverzeichnis).

|Angreifer|Gegenmaßnahmen|Restrisiko / Grenze|
|---|---|---|
|Datenträgerdieb, Kopie des Datenverzeichnisses|AEAD at-rest (INV-ENCRYPT-REST-1), Argon2id (INV-KDF-1), opake Schlüsselräume (INV-KEYSPACE-1)|Schutz hängt an der Passphrase-Stärke; Klartext im RAM/Swap (§4.3)|
|Angreifer mit Schreibzugriff auf das Verzeichnis|HMAC-Kette, Manifest-High-Water-Mark, Format-Versionen, KeyRegistry-Manipulationserkennung (fail-closed)|Ohne Verschlüsselung nur Korruptionsschutz; `doctor` weist es rot aus|
|Anderer lokaler Nutzer/Prozess|Unix-Socket 0600/ACL, Peer-Credentials (INV-IPC-1), kein TCP|Gleicher OS-Nutzer ist vertrauenswürdig|
|Bösartiger/kompromittierter MCP-Host oder LLM|3-Tool-Fassade, ToolCategory, Forget-Guard außerhalb des Tool-Kanals (INV-FORGET-GUARD-1), Daten-statt-Anweisung (INV-TRUST-1), Budgets/Exfiltrationsdetektor|Prompt-Injection ist ohne `InjectionDetector` nur durch den Marker begrenzt (wird ausgewiesen)|
|Mandant B gegen Mandant A|Tenant aus Transaktion (INV-TENANT-1), Index je Tenant (INV-TENANT-IDX-1), Statistiken je Tenant, ACL vor Distanz|Timing-Klassen werden gemessen, nicht als konstant zugesichert (§9.4)|
|Bösartiges Plugin `[R]`|Wasmtime ohne Ambient Authority, Fuel, `ResourceLimiter`, signierte Admission (INV-ADMISSION-1), Human-Approval (INV-APPROVAL-1)|Nur auf gepflegter Wasmtime-Linie (§14.3)|
|Supply Chain|deny/audit/vet, SBOM, gepinnte Builds (§28.4)|Native Drittabhängigkeiten sind inventarisiert, nicht eliminiert|
|Exfiltration|5 Egress-Schichten, `GuardedPayload`, Bulk-Detektor `[R]`|Wirksamkeit setzt reale Erkennung voraus (INV-WIRING-2)|

**Fail-closed** gilt bei Korruption, Schlüsselverlust, ungültiger Signatur, Widerruf, unbekannter Formatversion, nicht unterstütztem Dateisystem, fehlender Sicherheits-Port-Implementierung und numerischer Drift (`PrecisionMatrixDriftDetected` `[E]`): Das System verweigert und auditiert.

**Nicht adressiert:** RAM-Forensik und Core-Dumps; kompromittierter Host-Kernel; Seitenkanäle über Hardware; Schlüsselkopien außerhalb des Keystores (Escrow, §24.4); physisches Überschreiben auf dem Datenträger (§4.3).

**Logging:** Keine Nutzinhalte in Logs, Metriken, Fehlertexten oder Panic-Meldungen (INV-LOG-1).

### 30.1 Öffentliche API und interne Schlüssel `[K]`

INV-SCAN-1: Die öffentliche API (`cargo public-api`) DARF keine Funktion, keinen Typ und kein Feld enthalten, das Schlüsselmaterial (WAL-Integritätsschlüssel, DEK, KEK, Signierschlüssel) im Klartext zurückgibt. Test-Zugänge dafür liegen hinter `cfg(test)` oder einem Test-Feature, das in keinem Release-/Default-Set liegt. Mechanismus: `public-api`-Gate (REQ-ARCH-014).

---

## 31. Glossar

|Begriff|Bedeutung|
|---|---|
|**Composition Root**|Einziger Verdrahtungsort (`contextra`-Fassade).|
|**IoC**|Inversion of Control: Der Kern definiert Ports, Erweiterungen werden injiziert.|
|**Ring**|Architekturebene mit gerichteter Abhängigkeit (Ring 4 → 0).|
|**Kern / Rand / Experimentell**|Normativitätsklassen `[K]`/`[R]`/`[E]` (§6.0); Garantien gelten nur für den Kern.|
|**WAL**|Write-Ahead Log; Single Source of Truth.|
|**SSI**|Serializable Snapshot Isolation.|
|**Scope / Scope-DEK**|(Tenant, Collection[, Subject]); zufälliger Datenschlüssel je Scope, gewrappt im `KeyRegistry`-Log.|
|**Crypto-Shredding (SK)**|Löschung durch Vernichtung der Scope-DEK.|
|**Purge-Kompaktion**|Kompaktion, die Daten eines gelöschten Scopes nicht in neue SSTables übernimmt.|
|**LayerWitness**|Beleg einer Löschschicht (`Physical` oder `CryptoShredded`).|
|**UncoveredLayer**|Signiert deklarierte Löschschicht ohne Witness, inkl. Grund.|
|**DeletionReceipt / PendingDeletion**|Sofortige Rückgabe von `forget` mit Teilbeweis bzw. Handle auf die ausstehende Bereinigung.|
|**Typestate**|Zustandsmodell im Typsystem, das illegale Abläufe nicht kompilieren lässt.|
|**ACORN**|Prädikat-bewusste ANN-Traversierung für gefilterte Suche.|
|**RLS**|Row-Level Security (hier auf Vektorebene).|
|**TTC / TTFT**|Time-to-Context (Anfrage → berechtigter Kontext) / Time-to-First-Token.|
|**Aktivitätsuhr**|Zähler interaktiver Schreibtransaktionen für Decay und `Working`-TTL.|
|**Dormant**|Eintrag mit Decay-Gewicht < `DECAY_FLOOR`: ausgeblendet, nicht gelöscht.|
|**Primärprozess / Proxy**|Prozess mit DirLock und Engine / weiterer `contextra-mcp`-Start, der über IPC an den Primär weiterleitet.|
|**Meta-Gate**|Selbsttest der Audit-Skripte (§27.5).|

---

## 32. Kurzfassung für LLM-Kontext (Prompt-Header)

> Contextra ist eine lokale, eingebettete Single-Node-Hybrid-Datenbank für KI-Gedächtnis in Rust: LSM mit einem WAL als einziger Wahrheit, atomar synchronisierte Vektor- (HNSW/SQ8), Text- (BM25/BM25F) und optional Graph-Indexe **je Mandant getrennt**, SSI-Transaktionen, ACL-Prüfung vor der Distanzberechnung, AES-256-GCM-SIV-Verschlüsselung mit Scope-Schlüsseln (Argon2id-Keystore), Löschung in Stufen S0/SK/S1/S2/S3 über sieben Schichten (dokumentgenau über ein Epochenmodell, wenn `erasure_required` gesetzt ist) mit offline prüfbarem DeletionProof (Formatversion 4), der nicht belegte Schichten ehrlich als nicht abgedeckt ausweist, und MCP-Schnittstelle über stdio (`remember`/`recall`/`forget`) mit Primär-/Proxy-Betrieb. Nichtdeterministische Werte (Embeddings, LLM-Ausgaben) werden als Werte ins WAL geschrieben; Recovery ist deterministisch. Normativitätsklassen: Kern `[K]` gilt uneingeschränkt, Rand `[R]` nur bei aktiviertem Feature, Experimentell `[E]` ist nicht normativ. Ringe, Klasse, Reifegrad und IDs stehen ausschließlich in `capabilities.toml` und der Verfassungsdatei; Unsafe nur in `simd`, `sys`, `wire`. Qualität: Messbare Testtiefe (Ratchet), Fuzz-Matrix je Parser, loom-Inventar, Miri, Mutation, Chaos mit `FaultVfs`, Benchmarks gegen fremde Baselines als CI-Gate, Merge-Queue auf dem Merge-Ergebnis. Löschbelege entstehen nur aus Messung am Speicher, nie aus Konstanten, und es gibt keinen Fallback-Signierschlüssel. Aussagen zur Löschung folgen der Formulierung INV-CLAIMS-1; „100 % gelöscht“ ist unzulässig. Single-Node, kein Cross-Tenant, kein Sharding, kein Echtzeit-Audio.

---

## 33. Anforderungskatalog `[K]`

Jede normative Anforderung dieses Dokuments hat eine stabile ID, eine Klasse, einen Durchsetzungsmechanismus (Kürzel nach §27.6: **T** Typsystem, **G** Gate, **P** Property/Modell, **I** Integration über Fassade, **C** Chaos, **F** Fuzz, **L** loom, **B** Benchmark) und den Abschnitt, in dem sie entsteht. „OFFEN“ bedeutet: kein prüfbarer Mechanismus festgelegt; solche Anforderungen gelten nicht als Kern-Zusage. Der Katalog fasst die Invarianten, Prinzipien und Vetos aus §3 bis §5 sowie die Ergänzungen §20.8 bis §20.14 prüfbar zusammen; bei Abweichung zwischen Katalogzeile und Abschnittstext gilt der Abschnittstext.

**Zählung:** 161 Anforderungen: `[K]` 143, `[R]` 13, `[E]` 5. Ohne Mechanismus (OFFEN, ausschließlich `[E]`, damit nicht normativ): 4 (REQ-EXP-001, REQ-EXP-002, REQ-EXP-004, REQ-EXP-005).

|ID|Kl.|Anforderung|Mechanismus|Abschnitt|
|---|---|---|---|---|
|**REQ-ARCH-001**|`[K]`|Kanten (normal/build) MÜSSEN auf niedrigeren Ring zeigen oder, innerhalb eines Rings, als `may_use` deklariert und azyklisch sein (P1, §6.10).|G: cargo metadata gegen capabilities.toml|§5, §6|
|**REQ-ARCH-002**|`[K]`|Der Abhängigkeitsgraph MUSS mit aufgelösten Features azyklisch sein; Dev-Aufwärtskanten nur als deklarierte Ausnahme (P5).|G: Ring-/DAG-Gate|§5|
|**REQ-ARCH-003**|`[K]`|`[K]`-Crates DÜRFEN im Release-Default weder normal noch build von `[R]`/`[E]`-Crates abhängen (P30, INV-CORE-ISOLATION-1).|G: Kern-Isolation über cargo metadata|§5, §6.0|
|**REQ-ARCH-004**|`[K]`|`unsafe` DARF NUR in `simd`, `sys`, `wire` stehen; alle anderen Crates MÜSSEN `#![forbid(unsafe_code)]` tragen (P2).|G: Compiler-Lint + Unsafe-Inventar|§5, §6.7|
|**REQ-ARCH-005**|`[K]`|Jeder handgeschriebene `unsafe`-Block MUSS einen `// SAFETY:`-Kommentar tragen; crate-weite `allow` für `undocumented_unsafe_blocks` DÜRFEN NICHT existieren.|G: clippy deny + Allow-Zähler; P: Miri gegen sichere Referenz|§5 P2, §6.7|
|**REQ-ARCH-006**|`[K]`|Produktionscode MUSS panic-frei sein (`unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `unreachable!`, `assert!`, `process::exit`, unchecked Indexierung); Poisoning ist typisierter Fehler (P7).|G: Lint-Satz §27.5 + Ratchet (Ziel 0)|§5 P7|
|**REQ-ARCH-007**|`[K]`|Jede Persistenz MUSS durch das WAL gehen; kein Seitenpfad schreibt Zustand (P24).|C + I|§5, §7|
|**REQ-ARCH-008**|`[K]`|Ring-0-Crates MÜSSEN synchrone Kern-Algorithmen bieten und DÜRFEN keine Async-Runtime als Produktions-/Build-Abhängigkeit führen; Port-Traits dürfen Futures liefern, `async fn` außerhalb von Port-Definitionen ist ratchet-begrenzt (P26).|G: Abhängigkeitsbaum-Prüfung (Runtime-frei) + Zählung `async fn` (Ratchet, Ziel 0)|§5 P26|
|**REQ-ARCH-009**|`[K]`|Zeit, Zufall und IDs MÜSSEN über `Clock`, `Rng`, `IdGenerator` injiziert werden; `SystemTime::now`, `Instant::now`, `thread_rng`, `HashMap/HashSet` mit RandomState in Kernlogik sind verboten (Ausnahme: Schlüssel-/Salt-Material, P28).|G: clippy `disallowed_*` + P: Wiederholbarkeit|§5 P28|
|**REQ-ARCH-010**|`[K]`|Eine Fähigkeit existiert nur, wenn die Composition Root sie verdrahtet und ein Integrationstest sie über den echten Pfad beweist (P29).|G: Wired-Check + I|§3, §5|
|**REQ-ARCH-011**|`[K]`|Ring, Klasse (`kern`/`rand`/`experimentell`) und `maturity` je Crate MÜSSEN ausschließlich in `capabilities.toml` stehen.|G: Doc-Truth|§6, §28.5|
|**REQ-ARCH-012**|`[K]`|Features der Klassen `[R]`/`[E]` MÜSSEN im Release-Default aus sein.|G: Feature-Matrix über cargo metadata|§6.0|
|**REQ-ARCH-013**|`[K]`|`contextra-core` DARF keine Logik und keine neuen Abhängigkeiten tragen (nur Migrationsfassade).|G|§6.1|
|**REQ-ARCH-014**|`[K]`|Interne Schlüssel DÜRFEN NICHT über die öffentliche API erreichbar sein (INV-SCAN-1, Definition in §30.1).|G: `public-api`-Gate (verbotene Signaturen: Klartext-Schlüsselrückgabe, `*_for_test`-Zugänge in Release-Features)|§27.6, §30|
|**REQ-ARCH-015**|`[K]`|Ring 0 MUSS deterministisch sein: kein I/O außerhalb der Ports, keine Uhr und kein Zufall ohne Port, kein Ergebnis in Abhängigkeit von Hash-Iterationsreihenfolge, Thread-Scheduling oder unfixierter paralleler Float-Reduktion (INV-RING0-DETERMINISM-1).|G (`disallowed_*`, Ring-0-I/O-Scan) + P (Wiederholbarkeit, Reihenfolge-Permutation)|§5, §6.1|
|**REQ-ARCH-016**|`[K]`|Ein unklarer oder unbekannter Zustand MUSS zur Ablehnung führen, nie zu Raten oder Weiterlaufen (INV-FAILCLOSED-1).|C (Fehlerinjektion an jeder Schnittstelle) + I|§5, §7.10|
|**REQ-ARCH-017**|`[K]`|Der Kern DARF keine Echtzeit-Audio- oder STT-Funktion enthalten (VETO-OP03).|G: Deny-Liste über `cargo metadata` (Audio-/STT-Abhängigkeiten) + Review|§4.1|
|**REQ-ARCH-018**|`[K]`|Die Kantenregeln aus §6.10 MÜSSEN gelten: Fundament implizit, Kern-Kanten und Adapter-Kanten nur per `may_use`, Ring 4 erreicht Ring 2/3 nur über die Fassade, `LicenseGate` nur über den Port.|G: Ring-Gate über `cargo metadata` und `capabilities.toml`|§6.10|
|**REQ-ARCH-019**|`[K]`|Mocks, Platzhalter-Embedder, Test-Hooks und Test-Schlüsselzugänge DÜRFEN NICHT Teil der öffentlichen Produktions-API sein; sie liegen hinter `testing`/`test-hooks`/`cfg(test)`, die in keinem Release- oder Default-Feature-Set liegen.|G: `public-api`-Gate + `compile_fail`-Doctests|§6.10, §15.3|
|**REQ-ARCH-020**|`[K]`|Jedes Cargo-Feature MUSS im Code wirken und in der Feature-Matrix getestet sein; ungenutzte (tote) Features DÜRFEN NICHT existieren; Entfernen öffentlicher Features erfolgt nach Deprecation.|G: Feature-Matrix + Feature-Konsistenz|§6.0, §27.5|
|**REQ-ARCH-021**|`[K]`|Jedes Crate MUSS die Workspace-Lints erben (`lints.workspace = true`); Werkzeug-Workspaces mit eigener Lint-Konfiguration sind in `capabilities.toml` deklariert; crate-weite `allow` für `undocumented_unsafe_blocks` oder `missing_safety_doc` DÜRFEN NICHT existieren.|G: Lint-Vererbungs-Gate + Allow-Zähler|§5 P2, §27.5|
|**REQ-ARCH-022**|`[K]`|Die P28-Verbotsliste (`SystemTime::now`, `Instant::now`, `thread_rng`, RandomState-Hashes in Kernlogik) und die `assert!`-Familie MÜSSEN in einer versionierten `clippy.toml` (`disallowed-*`) stehen; zulässige Ausnahmen (z. B. die Uhr-Implementierung, Schlüssel- und Salt-Material, Tests, Benches, Werkzeuge) stehen in einer begründeten, einzeln geführten Allowlist.|G: clippy `disallowed_*` + Allowlist-Prüfung|§5 P28, §27.5|
|**REQ-ARCH-023**|`[K]`|Reifegrad- und Statusangaben in Crate-Dokumentation und README MÜSSEN aus `capabilities.toml` stammen oder dagegen geprüft werden; widersprüchliche Angaben sind Gate-Fehler.|G: Doc-Truth|§6, §28.5|
|**REQ-ARCH-024**|`[K]`|Der Kern MUSS ohne jedes `[R]`- oder `[E]`-Crate bauen und testen; Rand-Fähigkeiten werden nur über Ports mit dem Verhalten „Ohne Rand“ aus §6.11 verdrahtet.|G + I|§6.11|
|**REQ-DUR-001**|`[K]`|Der Commit-Punkt MUSS die plattformgerechte Durability-Primitive nutzen (§7.8); Ack erst danach (INV-DURABILITY-PRIM-1).|C: FaultVfs je Plattform|§7.3, §7.8|
|**REQ-DUR-002**|`[K]`|Nach `fsync`/`fdatasync`-Fehler DARF kein Ack, kein Retry auf demselben Handle erfolgen; der WAL-Handle wird vergiftet (INV-FSYNC-1).|C|§5, §7.10|
|**REQ-DUR-003**|`[K]`|Volles Medium DARF keinen Phantom-Eintrag und keinen halb angewendeten Batch hinterlassen; WAL-Segmente werden vorab reserviert (INV-ENOSPC-1).|C|§5|
|**REQ-DUR-004**|`[K]`|Lesepfade MÜSSEN I/O-Fehler als typisierte Fehler liefern; `mmap` nur für verifizierte, unveränderliche, lokale Dateien (INV-IO-1).|G (mmap-Nutzung) + C|§7.7|
|**REQ-DUR-005**|`[K]`|Jede persistente Datei MUSS Magic und Formatversion tragen; neuere Version ⇒ Öffnen verweigert; Formate explizit spezifiziert (INV-FORMAT-1).|G: Kompatibilitätskorpus + F|§7.2|
|**REQ-DUR-006**|`[K]`|Die WAL-Kette (HMAC-SHA256 + AEAD) und die Manifest-High-Water-Mark MÜSSEN Bruch, Truncation und Rollback erkennen.|C + F|§7.3|
|**REQ-DUR-007**|`[K]`|Recovery MUSS die Reihenfolge aus §7.3.1 einhalten und Verkehr erst danach zulassen.|C + I|§7.3.1|
|**REQ-DUR-008**|`[K]`|Im persistenten Betrieb MUSS das Öffnen auf Netzwerk-, unbekannten FUSE- und Cloud-Sync-Dateisystemen verweigert werden.|I + C|§7.3.1, §23.4|
|**REQ-DUR-009**|`[K]`|Es DARF keinen persistenten Modus ohne Integritätskette geben; DirLock und Recovery-Prüfungen sind in `Full` nicht abschaltbar.|T: Enum-Varianten + G|§23.2|
|**REQ-DUR-010**|`[K]`|Blob-Segmente MÜSSEN vor dem WAL-Commit durabel sein; Orphans räumt Recovery ab.|C|§7.9|
|**REQ-DUR-011**|`[K]`|Metadaten- und Indexänderung MÜSSEN in einem WAL-Batch liegen; ein Applier wendet Indexe in WAL-Reihenfolge an; Rebuild ist bit-identisch (INV-ATOMIC-IDX-1, INV-INDEX-APPLY-1, INV-WAL-SSOT-1).|C + P|§5, §24|
|**REQ-DUR-012**|`[K]`|Ergebnisse nichtdeterministischer Quellen MÜSSEN als Werte ins WAL geschrieben werden; Recovery ruft die Quelle nie erneut auf (INV-NONDET-1).|P: Replay ohne Quellenaufruf + G|§5|
|**REQ-DUR-013**|`[K]`|Schreiber MÜSSEN bei Rückstau gedrosselt werden (`l0_slowdown`, `l0_stop`); kein unbegrenzter Speicherzuwachs.|B + I|§7.3, §7.4|
|**REQ-DUR-014**|`[K]`|Betriebszustände `Healthy→Degraded→ReadOnly→Poisoned` MÜSSEN gemäß §7.10 übergehen.|C + I|§4.4, §7.10|
|**REQ-DUR-015**|`[K]`|Sämtlicher Dateizugriff MUSS über den Port `Vfs` laufen; Ring-0-Crates führen keinen direkten Dateizugriff aus; `Vfs` und `DurableLog` sind produktiv verdrahtet (INV-RING0-DETERMINISM-1, INV-WIRING-2).|G: Ring-0-I/O-Scan + C: `FaultVfs`|§5, §7.8|
|**REQ-DUR-016**|`[K]`|Der Anwendungszustand MUSS aus WAL und Manifest rekonstruierbar sein; Sicherheitszustand liegt ausschließlich in den Sidecar-Logs (§7.11), mit Kette, Manifest-Marke und „Sidecar vor Wirkung“ (INV-SIDECAR-1, INV-WAL-SSOT-1).|G + C + I|§7.11|
|**REQ-DUR-017**|`[K]`|Löschen MUSS in den Zuständen `ReadOnly` und `Poisoned` über die Notreserve und eigene Sidecar-Handles möglich bleiben; ohne jede Möglichkeit gilt die prozesslokale Lesesperre bis zum Neustart (INV-EMERGENCY-DEL-1).|C + I|§7.12|
|**REQ-MVCC-001**|`[K]`|SSI MUSS im Produktionspfad aktiv sein; Read-Sets inkl. Prefix-/Range-Phantomschutz werden bei jedem Commit validiert (INV-SSI-ACTIVE-1).|P: Referenzmodell + C: History-Checker + L|§8|
|**REQ-MVCC-002**|`[K]`|Indexeinträge MÜSSEN `born_seq`/`died_seq` tragen; Suchen sehen nur Einträge mit `born_seq ≤ S < died_seq` (INV-INDEX-SNAPSHOT-1).|P + L|§5, §8|
|**REQ-MVCC-003**|`[K]`|Ein Snapshot MUSS immer ein konkreter Sequenzstand sein, nie `u64::MAX`.|P|§8|
|**REQ-MVCC-004**|`[K]`|Die Sperrenordnung MUSS vollständig und geordnet sein; Guards DÜRFEN NICHT über `.await` gehalten werden (INV-LOCK-ORDER-1).|L + G (`await_holding_lock`)|§8.1|
|**REQ-MVCC-005**|`[K]`|Jede nebenläufige Kernstruktur MUSS ein loom-Modell besitzen (Inventar §27.1).|G: Inventar-Abgleich|§27.1|
|**REQ-IDX-001**|`[K]`|Vektor-, Text- und Graph-Indexe MÜSSEN je (Tenant, Collection[, Modell-Hash]) physisch getrennt sein; keine tenantübergreifende Kante, Posting-Liste oder Statistik (INV-TENANT-IDX-1).|P/F: Cross-Tenant + T|§9.2|
|**REQ-IDX-002**|`[K]`|Ein HNSW-Arena-Slot DARF NICHT wiederverwendet werden, solange eingehende Kanten möglich sind (INV-HNSW-SLOT-1).|L|§9.3|
|**REQ-IDX-003**|`[K]`|HNSW-Löschung MUSS Tombstone-Pruning sein; kein partielles Rewiring (VETO-F02).|G: check_vetoes + P|§4.1, §9.3|
|**REQ-IDX-004**|`[K]`|Der ACL-Bitset MUSS vor der Distanzberechnung greifen; Ergebnisse enthalten keine Scores/Counts nicht berechtigter Knoten.|P|§9.4|
|**REQ-IDX-005**|`[K]`|Gefilterte Suche MUSS exakt `k` oder alle passenden Treffer liefern (Overfetch 3k→6k→12k, nur deterministische Zähler).|P|§9.8|
|**REQ-IDX-006**|`[K]`|SQ8-Kandidaten MÜSSEN exakt nachgerankt werden; das Ergebnis DARF NICHT vom Cache-Zustand abhängen.|P|§9.6|
|**REQ-IDX-007**|`[K]`|SIMD-Kernel MÜSSEN gegen die skalare Referenz innerhalb dokumentierter Toleranz äquivalent sein.|P + Miri|§9.6|
|**REQ-IDX-008**|`[K]`|BM25-Statistiken (IDF, Dokumentlängen) MÜSSEN je Tenant geführt werden.|P|§10|
|**REQ-IDX-009**|`[K]`|RRF-Fusion MUSS k=60 mit Tie-Breaker Score absteigend, DocId aufsteigend verwenden.|P|§16, §25.3|
|**REQ-IDX-010**|`[K]`|LSM-Schlüssel, Index-IDs und Dateinamen DÜRFEN NUR opake IDs/Hashes enthalten (INV-KEYSPACE-1).|G + P|§5|
|**REQ-IDX-011**|`[K]`|Kein stiller Platzhalter-Embedder; `test-utils` in keinem Release-/Default-Feature-Set; Modellhash und Dimension je Collection persistiert (INV-EMBED-1/-2).|G + I|§15.3|
|**REQ-IDX-012**|`[K]`|Chunking MUSS deterministisch sein (gleicher Text + Konfiguration ⇒ gleiche Grenzen und IDs) und `source_span` tragen.|P|§16.1|
|**REQ-IDX-013**|`[R]`|Graph-Indexe MÜSSEN je (Tenant, Collection) getrennt sein; PathRAG-Suche MUSS knotenbudgetiert abbrechen.|P|§11|
|**REQ-IDX-014**|`[R]`|`rank`-Kalibrierung und Explain MÜSSEN deterministisch und reproduzierbar sein.|P|§12|
|**REQ-IDX-015**|`[K]`|`embedding_model_hash` MUSS aus der kanonischen Kodierung der Modellidentität berechnet und bei jedem Insert und jeder Suche gegen die Collection geprüft werden; Backends ohne berechenbare Identität sind für persistente Collections unzulässig (§15.3).|G + I|§15.3|
|**REQ-KEY-001**|`[K]`|Jeder Schlüssel MUSS von einem `KmsProvider` stammen; mindestens die lokale Implementierung ist verdrahtet; Passphrasen-Ableitung mit festem/globalem Salt ist verboten (INV-KMS-1).|G + I: Selbsttest beim Start|§5, §20.2|
|**REQ-KEY-002**|`[K]`|Passphrasen-Schlüssel MÜSSEN über Argon2id mit zufälligem, persistiertem Salt und Parametern (m≥64 MiB, t≥3, ≥250 ms) entstehen (INV-KDF-1).|P + I|§5, §25.1|
|**REQ-KEY-003**|`[K]`|Nutzdaten MÜSSEN at-rest AEAD-verschlüsselt sein; ohne Verschlüsselung weist `doctor` rot aus (INV-ENCRYPT-REST-1).|I + G: Klartext-Scanner|§5|
|**REQ-KEY-004**|`[K]`|Per `mmap` gelesene Dateien MÜSSEN auf verschlüsseltem Volume liegen, sonst Page-Decrypt-Cache (INV-ENCRYPT-REST-2).|G + I|§5|
|**REQ-KEY-005**|`[K]`|AEAD MUSS AES-256-GCM-SIV mit 96-Bit-Nonce, 128-Bit-Tag und AD `scope_id‖key_id‖doc_id_hash` sein.|P: Vektoren + Negativtests|§20.1, §25.1|
|**REQ-KEY-006**|`[K]`|Das `KeyRegistry`-Log MUSS append-only, HMAC-verkettet und durabel sein; Löschung des Logs bei vorhandenen Daten ist Manipulation (fail-closed).|C + F|§20.2|
|**REQ-KEY-007**|`[K]`|KEK-Rotation MUSS ohne Datenrewrite (Re-Wrap) funktionieren.|P + I|§20.2|
|**REQ-KEY-008**|`[K]`|Geheimnisse MÜSSEN `zeroize`-geschützt sein und konstantzeitig verglichen werden; Schlüsseltypen DÜRFEN NICHT `Copy`/`Clone` ohne Zeroisierung sein.|G + P|§20.1|
|**REQ-KEY-009**|`[K]`|Kryptografische Primitive DÜRFEN NICHT selbst gebaut werden.|Review|§3 Punkt 12|
|**REQ-KEY-010**|`[K]`|Im Produktionscode DÜRFEN keine Schlüssel-, Passphrasen- oder Signier-Literale stehen (auch nicht als Test-, Default- oder Fallback-Wert).|G: Literal-Scan `no-literal-secrets`|§20.8|
|**REQ-KEY-011**|`[K]`|Jeder Schlüssel MUSS genau einen Zweck aus dem Zweckregister (§20.2c) haben; Labels sind paarweise verschieden; ein Label ohne Registereintrag ist Gate-Fehler (INV-KEYPURPOSE-1).|G + P|§20.2c|
|**REQ-DEL-001**|`[K]`|`forget` MUSS alle sieben Löschschichten über die Composition Root auslösen (INV-DELETION-WIRE-1).|T: Typestate + I|§5, §20.4|
|**REQ-DEL-002**|`[K]`|Kryptografische Löschung gilt erst als vollzogen, wenn DEK-Vernichtung und `RevocationLog`-Eintrag durabel sind (INV-SHRED-1).|C: Crash an jedem Punkt + I|§5, §24.3|
|**REQ-DEL-003**|`[K]`|Jede der sieben Schichten MUSS in einem Beweis genau einmal als belegt oder nicht abgedeckt erscheinen (INV-PROOF-COVERAGE-1).|T: Typestate-Builder + I|§20.5|
|**REQ-DEL-004**|`[K]`|Ein Schichtbeleg DARF NUR aus der Speicherschicht ausstellbar sein; Bypass-Konstruktoren sind außerhalb von test/fuzz/benches verboten (INV-WITNESS-MINT-1).|T + G: Aufrufer-Regel|§20.5|
|**REQ-DEL-005**|`[K]`|Der Signierschlüssel des Beweises MUSS aus dem `KeyManager` stammen, nie aus einem frei übergebenen Byte-String (INV-PROOF-KEY-1).|T + P|§20.5|
|**REQ-DEL-006**|`[K]`|Externe Zusicherungen MÜSSEN aus der belegten Garantiestufe erzeugt werden; „100 % gelöscht“ ist unzulässig (INV-CLAIMS-1).|G|§4.3, §20.12|
|**REQ-DEL-007**|`[K]`|`forget` MUSS über die Lineage auf Ableitungen kaskadieren (INV-DERIVED-1).|I + P|§5, §20.4|
|**REQ-DEL-008**|`[K]`|Ein Schicht-Messwert DARF NUR vom Scanner der Schicht erzeugt werden, MUSS `units_scanned > 0` haben (außer Schlüsselabwesenheit) und DARF NICHT aus einer Konstante oder einem Aufruferwert stammen.|T + Mutation (Konstante ersetzen ⇒ Test rot) + G (`deletion-honesty`)|§20.8|
|**REQ-DEL-009**|`[K]`|Es DARF keinen Fallback-, Standard- oder Testschlüssel zum Signieren eines Löschbeweises geben; ohne konfigurierten Schlüssel entsteht kein Beweis (Fehler).|G: Literal-Scan + I|§20.8|
|**REQ-DEL-010**|`[K]`|Keine Verifikationsfunktion DARF unbedingt „wahr“ liefern; fehlende Fähigkeit ist `CapabilityUnsupported`, nie ein Stub-Erfolg.|G + P|§20.8, §29|
|**REQ-DEL-011**|`[K]`|Ab `Frozen` DARF kein Lesepfad (`get`, `search`, `hydrate`, `export`, Checkpoint-Erzeugung) Daten des Betroffenen liefern, auch unter Parallelität.|P + L|§20.9|
|**REQ-DEL-012**|`[K]`|Löschen MUSS unter dem Key-Lock über die gesamte Transaktion laufen; konkurrierendes Schreiben DARF kein Wiederauftauchen bewirken.|L|§20.9|
|**REQ-DEL-013**|`[K]`|Nach Wiederherstellung eines Checkpoints MUSS jede bekannte Löschung vor dem ersten Lesezugriff erneut angewendet werden (Restore-Guard).|I|§20.9, §24.4|
|**REQ-DEL-014**|`[K]`|Die Löschkette MUSS als durable, idempotente, wiederaufnehmbare Saga laufen; Absturz in jedem Zustand führt nie zu „abgeschlossen“ ohne Messung.|C: Crash-Matrix|§20.10, §24.3|
|**REQ-DEL-015**|`[K]`|Nachweis und Ledger DÜRFEN NUR Metadaten und Commitments enthalten, keine Klartext-IDs oder Inhalte.|G + P|§20.11|
|**REQ-DEL-016**|`[K]`|Jeder Claim im Beweis MUSS seine Voraussetzungen aus §20.12 erfüllen; `CryptoErasureAttested` nur mit Verwahrungsstufe K3 und Vernichtungsattestierung; bei K0/K1 entsteht kein kryptografischer Claim, bei K2 nur `CryptoErasureLocal` mit Warnung.|T + P|§20.12|
|**REQ-DEL-017**|`[K]`|Der Beweis MUSS offline über `verify_external` (nur `crypto`+`types`) mit benannten Fehlergründen prüfbar sein; ein Verifier in anderer Sprache MUSS existieren.|F + P + Referenz-Verifier|§20.5|
|**REQ-DEL-018**|`[K]`|Bei Aufbewahrungspflicht/Legal Hold MUSS Löschung in `Restricted` übergehen, ohne Schlüsselvernichtung, mit Lesesperre.|I|§20.10|
|**REQ-DEL-019**|`[K]`|Stufe S3 MUSS WAL-Segmente scrubben, ohne die Kette zu brechen.|C|§20.3|
|**REQ-DEL-020**|`[K]`|Jede Löschstufe (S0, SK, S1, S2, S3) MUSS pro Schicht durch eine Messung belegt oder als nicht abgedeckt deklariert sein.|I + Canary-Scan (§27.9)|§20.3, §20.4|
|**REQ-DEL-021**|`[K]`|`forget` MUSS sofort einen ehrlichen Teilbeweis und ein `PendingDeletion`-Handle zurückgeben; Offene Snapshots verzögern S2 (`Pending`), brechen nicht ab.|I|§24.3|
|**REQ-DEL-022**|`[K]`|Der `RevocationLog` MUSS durabel (fsync vor Wirksamkeit), crash-sicher und bei jeder Autorisierung konsultiert sein; Schreiben DARF NICHT den bestehenden Log zerstören.|C + F|§20.6|
|**REQ-DEL-023**|`[K]`|Die Löschpfad-Ports (INV-DELETION-PORTS-1: `KeyCustodian`, `ErasureParticipant`, `ErasureJournal`, `Vfs`, `DurableLog`, `KmsProvider`) MÜSSEN in `governance/security_ports.toml` geführt werden und produktiv verdrahtet sein.|G: Port-Klassifikation + Wired-Check|§5 INV-WIRING-2, §20|
|**REQ-DEL-024**|`[K]`|Eine Collection mit `erasure_required` MUSS die Verschlüsselung im Ruhezustand und einen Verwahrer voraussetzen; andernfalls liefert der Start `CapabilityUnsupported`, nie einen Stub.|I|§20.2b|
|**REQ-DEL-025**|`[K]`|Dokumentgenaue kryptografische Löschung MUSS als Epochen-Rotation (Re-Wrap der Überlebenden, dann Vernichtung der `EpochKek`) erfolgen; das Löschen nur des gewrappten DEK genügt nicht.|P (AC-7) + C|§20.2b|
|**REQ-DEL-026**|`[K]`|Payloads DÜRFEN keine gewrappten Schlüssel tragen, nur eine `KeyRef`; gewrappte Schlüssel liegen im getrennten `ErasureKeyStore`.|G + P: Rohscan|§20.2b|
|**REQ-DEL-027**|`[K]`|Löschtypen MÜSSEN genau einmal existieren; ohne Löschpfad liefert der Orchestrator `CapabilityUnsupported`; ein Stub, der Erfolg meldet, ist unzulässig.|G: Typfamilien-Zählung + `deletion-honesty`|§6.10, §20.15|
|**REQ-DEL-028**|`[K]`|Jedes persistierte kryptografische Objekt MUSS Algorithmus- und Formatkennung tragen; ein Algorithmenwechsel MUSS ohne Datenverlust möglich und der Austauschplan dokumentiert sein (Krypto-Agilität).|P: Migrationstest + Review|§20.1|
|**REQ-DEL-029**|`[K]`|Der Rohdatei-Scanner MUSS alle Dateien des Datenverzeichnisses (WAL, SSTables, Checkpoints, Temp, `.bak`, Keystore, Journal) lesen; ein Scanner, der nur Teilmengen liest, erzeugt keinen sauberen Beleg.|I: Canary-Läufe (§27.9)|§20.15|
|**REQ-DEL-030**|`[K]`|Die Lesesperre MUSS auch für historische Lesezugriffe (`as_of`, offene Snapshots, Checkpoint-Lesen) gelten, auch wenn die Altversion physisch existiert (INV-ASOF-ERASE-1).|P + L|§8, §20.9|
|**REQ-DEL-031**|`[K]`|Snapshots MÜSSEN das Höchstalter `snapshot_max_age` haben; abgelaufene Snapshots werden als `SnapshotExpired` final ungültig und nicht verlängert (INV-ASOF-ERASE-1).|L + I|§8, §25.4|
|**REQ-AUD-001**|`[K]`|Die AuditChain MUSS append-only, hash-verkettet, tenant-getrennt und durabel sein.|P + F|§20.6|
|**REQ-AUD-002**|`[K]`|Logs, Metriken, Fehlertexte und Panic-Meldungen DÜRFEN keine Nutzinhalte enthalten (INV-LOG-1).|G: Log-Scanner + P|§5|
|**REQ-TEN-001**|`[K]`|Der Tenant MUSS aus der Transaktion abgeleitet werden; jede Schlüsselkonstruktion enthält das Tenant-Präfix (INV-TENANT-1).|T + P|§5|
|**REQ-TEN-002**|`[K]`|Es DARF keinen mandantenübergreifenden Datenfluss geben (VETO-F10).|P/F|§4.1|
|**REQ-TEN-003**|`[K]`|Ein vom Aufrufer gelieferter Tenant-Wert (fest verdrahtete Tenant-Konstante im Handler) DARF NICHT als Mandantenzuordnung dienen.|T + G: Literal-Scan|§4.1, §21.1|
|**REQ-TEN-004**|`[K]`|Tenant-Zustände `Aktiv`, `Gesperrt`, `Gelöscht` MÜSSEN gemäß §24.5 übergehen; Tenant-IDs gelöschter Tenants werden nie wiederverwendet.|T + I|§24.5|
|**REQ-TEN-005**|`[K]`|Quoten `tenant_quota_bytes` und `tenant_quota_docs` MÜSSEN in den Enterprise-Profilen gesetzt sein; Überschreitung liefert `QuotaExceeded`.|I + G|§16|
|**REQ-MCP-001**|`[K]`|`stdout` MUSS ausschließlich dem MCP-Protokoll gehören; `print_stdout` ist `deny` im MCP-Binär (INV-STDOUT-1).|G|§21.0|
|**REQ-MCP-002**|`[K]`|Genau ein Primärprozess MUSS das Datenverzeichnis halten; weitere Clients nutzen lokales IPC mit Peer-Credentials, kein TCP/UDP (INV-IPC-1).|I + C|§21.1|
|**REQ-MCP-003**|`[K]`|Die 3-Tool-Fassade (`memory_remember`, `memory_recall`, `memory_forget`) und `ToolCategory`-Annotationen MÜSSEN verfügbar sein.|I|§21.2|
|**REQ-MCP-004**|`[K]`|Jeder Aufruf MUSS die 5-stufige Sicherheitspipeline durchlaufen.|I|§21.3|
|**REQ-MCP-005**|`[K]`|Breite oder selektorbasierte Löschungen MÜSSEN eine Bestätigung über einen Kanal außerhalb des Tool-Aufrufs verlangen; ein vom Aufrufer setzbares Argument genügt nicht (INV-FORGET-GUARD-1).|I: Prompt-Injection-Szenarien|§21.5|
|**REQ-MCP-006**|`[K]`|Recall MUSS Inhalt strukturiert mit `trust` und `source` liefern, nie als Anweisung (INV-TRUST-1).|I + P|§21.6|
|**REQ-MCP-007**|`[K]`|Abbruch MUSS kooperativ bis in die Engine wirken; nach dem WAL-Append wird er ignoriert (INV-CANCEL-1).|I + C|§21.4|
|**REQ-MCP-008**|`[K]`|Eingangs- und Ausgabegrenzen (RPC-Größe, `max_response_tokens`) MÜSSEN erzwungen und fuzz-geprüft sein.|F + I|§21.4|
|**REQ-MCP-009**|`[K]`|Die CLI MUSS `init`, `doctor`, `verify-proof`, `confirm`, `migrate` bereitstellen.|I|§21.7|
|**REQ-MCP-010**|`[K]`|Fehler MÜSSEN strukturiert und als retrybar/final klassifiziert sein.|I|§21.8|
|**REQ-FAC-001**|`[K]`|Nur die Fassade `contextra` DARF konkrete Implementierungen wählen und verdrahten.|G: Wired-Check|§22|
|**REQ-FAC-002**|`[K]`|Beim Start MUSS ein Verdrahtungs-Selbsttest laufen; fehlende Sicherheits-Port-Implementierung ⇒ Start verweigert.|I|§22|
|**REQ-FAC-003**|`[K]`|Sicherheits-Ports MÜSSEN reale Implementierungen haben; Null-, Test- und Accept-All-Implementierungen als Produktionsstandard sind unzulässig (INV-WIRING-2).|G: Port-Klassifikation + Accept-All-Erkennung|§5, §22|
|**REQ-FAC-004**|`[K]`|Mock-Provider DÜRFEN NUR nach ausdrücklicher Konfiguration aktiv sein; unbekannte oder fehlende Provider-Konfiguration ist ein Startfehler.|G + I|§15.3|
|**REQ-FAC-005**|`[K]`|Der Start MUSS die Phasen B0 bis B9 in fester Reihenfolge durchlaufen; die Lesesperre MUSS vor dem ersten Lesepfad armiert sein; jede Phase kann mit benanntem Grund verweigern, Teilstarts sind unzulässig (INV-BOOT-1).|I + C|§22.1|
|**REQ-PLT-001**|`[K]`|Durabilitäts- und Löschgarantien gelten nur auf der Garantieplattform; `doctor` weist `BestEffort` aus, Beweise tragen `platform_guarantee=false` (INV-PLATFORM-1).|CI-Matrix + C|§23.4|
|**REQ-PLT-002**|`[K]`|Die vier Deployment-Tiers und `PerformanceProfile` MÜSSEN ohne Verletzung von Korrektheitsinvarianten wirken.|I|§23.1, §23.2|
|**REQ-PLT-003**|`[K]`|Löschlogik DARF sich nicht auf Unix-Unlink-Semantik offener Dateien stützen.|C: Windows-Handle-Nemesis|§23.4|
|**REQ-PLT-004**|`[K]`|`doctor` MUSS Dateisystemtyp und Cloud-Sync-Pfad nach §23.4 ausweisen und im persistenten Betrieb nicht unterstützte Typen verweigern.|I|§21.7, §23.4|
|**REQ-QA-001**|`[K]`|Jede Zusage MUSS durch maschinell erzeugte Evidenz (Befehl, Toolchain, Commit, Exit-Code, Dauer, Hash) belegt sein; nicht ausgeführtes Prüfwerkzeug ist rot (INV-EVIDENCE-1).|G: `cargo xtask verify` (§27.10)|§3, §27.7|
|**REQ-QA-002**|`[K]`|Jede Kerngarantie G1–G4 MUSS ein unabhängiges Orakel besitzen (INV-ORACLE-1).|P: Referenzmodell / Brute-Force / Verifier anderer Sprache|§3 Punkt 15, §27.8|
|**REQ-QA-003**|`[K]`|Jede Eintrittsstelle für nicht vertrauenswürdige Bytes MUSS ein Fuzz-Target haben; ein Gate gleicht die Matrix mit dem Code ab.|G|§27.1|
|**REQ-QA-004**|`[K]`|Fuzz- und Miri-Läufe sind Pflicht-Gates; „SKIPPED“ gilt als rot.|G|§27.1|
|**REQ-QA-005**|`[K]`|Mutation-Score je `[K]`-Crate MUSS in `governance/ratchet.toml` stehen und darf nur steigen.|G|§27.1|
|**REQ-QA-006**|`[K]`|Chaos-Suite mit `FaultVfs` MUSS an jedem Schreib-/Durability-Punkt seed-reproduzierbar injizieren.|C|§27.3|
|**REQ-QA-007**|`[K]`|Benchmarks MÜSSEN gegen versionierte Baselines und Budgets laufen; Vergleichsaussagen brauchen eine fremde Baseline.|B|§26, §27.4|
|**REQ-QA-008**|`[K]`|Jedes Audit-Skript MUSS einen Selbsttest mit synthetischem Repository haben; Grenzwerte DÜRFEN kein Gate faktisch abschalten.|G: Meta-Gate|§27.5|
|**REQ-QA-009**|`[K]`|Gate-Läufe MÜSSEN auf einem von der Agenten-Infrastruktur unabhängigen Runner reproduzierbar sein; rohe Protokolle werden archiviert.|Review + CI|§27.5|
|**REQ-QA-010**|`[K]`|Tests DÜRFEN NICHT gelöscht oder abgeschwächt werden, um Gates grün zu machen.|G: gate-weakening / test-integrity|§28.2|
|**REQ-GOV-001**|`[K]`|Merges MÜSSEN scope-treu, beschreibend benannt und über eine Merge-Queue auf dem Merge-Ergebnis geprüft sein (INV-PROVENANCE-1).|G|§28.2|
|**REQ-GOV-002**|`[K]`|Änderungen an geschützten Pfaden MÜSSEN einen ADR-Verweis und eine Freigabe tragen.|G: protected-paths|§28.2|
|**REQ-GOV-003**|`[K]`|Änderungen an Garantien oder Formaten aus §28.6 MÜSSEN per ADR erfolgen.|Review + G|§28.6|
|**REQ-GOV-004**|`[K]`|Jede Tatsache MUSS genau eine kanonische Quelle haben; doppelt vergebene IDs sind Gate-Fehler (INV-DOC-SINGLE-SOURCE-1).|G: Doc-Truth (inkl. Konsistenz Register §5.2 ↔ §27.6 ↔ §33: jede `INV-*` einmal definiert, einmal in der Matrix, mindestens einmal referenziert)|§5, §5.2, §28.5|
|**REQ-GOV-005**|`[K]`|Ein 1.0-Release MUSS die Kriterien aus §28.1 erfüllen.|Review|§28.1|
|**REQ-GOV-006**|`[K]`|Advisory-Ignorierungen MÜSSEN einzeln begründet, erreichbarkeitsanalysiert und befristet sein; kritische/hohe Advisories werden nicht ignoriert.|G: cargo-deny/audit/vet|§28.4|
|**REQ-SBX-001**|`[R]`|Ein `AdmittedModule` MUSS nur über `ModuleVerifier::verify` konstruierbar sein; der Digest wird aus den Bytes berechnet (INV-ADMISSION-1).|T + F|§14.2|
|**REQ-SBX-002**|`[R]`|`High`/`Elevated`-Module DÜRFEN NICHT vor `Approved` laufen (INV-APPROVAL-1).|T + I|§14.1|
|**REQ-SBX-003**|`[R]`|Wasmtime-Konfiguration MUSS bit-identische Ergebnisse auf x86-64 und aarch64 liefern (INV-WASM-DET-1).|P|§14.3|
|**REQ-PRV-001**|`[R]`|Egress MUSS alle fünf Schichten typisiert durchlaufen (`GuardedPayload`); ohne reale Erkennung ist Egress gesperrt.|T + I|§19.1, §19.3|
|**REQ-PRV-002**|`[R]`|Jede externe Wirkung MUSS budgetiert, auditiert und widerrufbar sein (INV-EGRESS-BUDGET-1).|I|§19.1|
|**REQ-PRV-003**|`[R]`|`mlock`-Misserfolg MUSS wahrheitsgemäß gemeldet und in `doctor` ausgewiesen werden.|I|§19.5|
|**REQ-COG-001**|`[R]`|`grounding_check` MUSS unterhalb der Schwelle `PolicyViolation(LowConfidenceGrounding)` liefern (INV-GASP-1).|I + P|§15.2|
|**REQ-COG-002**|`[R]`|`MemoryType`-Regeln MÜSSEN gelten (INV-MEMTYPE-1..3).|P|§17.1|
|**REQ-COG-003**|`[R]`|Decay DARF nie löschen; Löschung entsteht nur durch ausdrückliche Aktion, Working-TTL oder opt-in Policy (INV-DECAY-1, INV-RETENTION-1).|P + I|§16.4, §17.0b|
|**REQ-KVC-001**|`[R]`|M1 DARF nur behauptet werden, wenn die Bedingungen B1–B3 aus §13.1 erfüllt sind.|B + G|§2, §13.1|
|**REQ-KVC-002**|`[R]`|`forget` MUSS KV-Prefixe invalidieren, die das Dokument enthalten.|I|§13|
|**REQ-EXP-001**|`[E]`|Bandit-Routing bleibt aus, bis Regret-Vorteil nachgewiesen (INV-ROUTING-1..3).|OFFEN (nicht normativ)|§18a|
|**REQ-EXP-002**|`[E]`|Agent-Laufzeit und CLM-Scratchpad (Auslagerung empfohlen).|OFFEN (nicht normativ)|§18|
|**REQ-EXP-003**|`[E]`|Lizenz-Prüfreihenfolge und Degradation ohne Datenverlust (INV-LICENSE-1..3).|P|§23.5|
|**REQ-EXP-004**|`[E]`|Audit-Export und AVV-Vorlagen mit Rechtsprüfungsvorbehalt.|OFFEN (nicht normativ)|§20.7|
|**REQ-EXP-005**|`[E]`|SPLADE ergänzt BM25 hinter Feature-Flag.|OFFEN (nicht normativ)|§10|

---

## 34. Grundsatzentscheidungen dieser Fassung `[K]`

Dieser Abschnitt hält fest, welche Gestaltungsfragen diese Fassung entschieden hat, mit Begründung und Konsequenz. Die Festlegungen gelten normativ und sind in den genannten Abschnitten umgesetzt. Eine Änderung einer Festlegung ist eine ADR-pflichtige Entscheidung (§28.6).

|ID|Frage|Festlegung|Begründung|Konsequenz|
|---|---|---|---|---|
|F-01|Löschmodell: einstufige Scope-DEK oder Epochen-Hierarchie?|**Beides, gestuft:** Scope-DEK für ganze Scopes (§20.2); Epochen-Hierarchie (`TenantKek` → `EpochKek` → `DocDek`/`SegKey`) für Collections mit `erasure_required` (§20.2b). Ein einziger Beweistyp, `DeletionProof` Formatversion 4, mit zusätzlichen Feldern (§20.5).|Die Scope-DEK ist einfach und schnell, kann aber Einzeldokumente nicht kollateralfrei kryptografisch löschen; das Epochenmodell leistet das (AC-7) zu höheren Schlüsselverwaltungskosten. Nur wer Betroffenenrechte pro Dokument braucht, zahlt dafür.|Zwei Schlüsselpfade, ein Beweisformat; Migration bestehender Formatversionen v1–v3 nur lesend.|
|F-02|Wo liegen Schlüsselregister und Widerrufslog?|`crypto` bleibt I/O-frei (Kodierung, Kette, Signatur); die durable Ablage läuft über den Port `DurableLog`, implementiert in `store`.|Hält Ring 0 deterministisch (INV-RING0-DETERMINISM-1) und testbar ohne Dateisystem.|Neuer Port `DurableLog`; Tail-Anker über signierten Kettenkopf und optional `NotaryPort`.|
|F-03|Eigenes Crate `durable-fs`?|**Nein.** Port `Vfs` in `ports`, Implementierung (Durability-Primitive, DirLock, atomare Writes) in `store`, `FaultVfs` in `testkit`.|Geringster Umbau ohne Verlust der Garantie „kein I/O in Ring 0 außer Ports“; vermeidet ein weiteres Kern-Crate.|32 Crates unter `crates/` (§6.6); `store` ist Hotspot und trägt die I/O-Verantwortung.|
|F-04|Async in Ring-0-Ports (P26)?|P26 verbietet die **Runtime-Abhängigkeit** und blockierende Runtime-Nutzung in Ring 0; Port-Traits dürfen Futures liefern; Algorithmen bleiben synchron.|Adapter brauchen asynchrone Schnittstellen (KMS, Verwahrer, Journal); eine wörtliche Async-Freiheit der Ports würde sie unnötig verdoppeln.|Gate prüft Runtime-Freiheit des Abhängigkeitsbaums und ratchtet `async fn` außerhalb von Port-Definitionen.|
|F-05|Ort und Rang der Spezifikation|Diese Spezifikation ist die einzige Quelle der Soll-Architektur; `capabilities.toml` führt zusätzlich das Feld `class`.|Eine Quelle verhindert Drift (INV-DOC-SINGLE-SOURCE-1).|Abgeleitete Dokumente werden generiert oder per Doc-Truth-Gate geprüft (REQ-ARCH-023).|
|F-06|MCP-Werkzeugoberfläche|3-Tool-Fassade `memory_remember`/`memory_recall`/`memory_forget` als Standard für Agenten **plus** Vollwerkzeugsatz `contextra_*` mit `ToolCategory` (§21.2).|Wenige Werkzeuge halten Agentenkontext klein und das Löschen kontrollierbar; Administration braucht den Vollsatz.|Beide Oberflächen sind spezifiziert; Kategorien steuern die Freigabe.|
|F-07|`DurabilityMode::WalNoHmac`|**Entfällt.** Es gibt nur `Full` und `MemoryOnly` (§23.2).|Ein Modus ohne Integritätskette schwächt G1 und die Sicherheitsannahmen aus §30.|REQ-DUR-009.|
|F-08|Verwahrungsstufen|K0 bis K3 mit Abbildung auf `CustodianKind`; kryptografische Claims nur bei K2 (lokal, mit Warnung) und K3 (attestiert); K0/K1 ohne kryptografischen Claim (§20.2a, §20.12).|Lokal gewrappte Schlüssel sind nicht sicher vernichtbar; der Claim darf nicht mehr behaupten als die Verwahrung hergibt.|AC-14, AC-15.|

**Nicht entschieden und deshalb als Prüfpunkte geführt** sind ausschließlich Rechtsfragen (§20.13, P-01 bis P-14). Sie hängen nicht von der Technik ab, und die Technik bedient alle plausiblen Antworten (§20.0).

---

## Anhang A. Port-Katalog (Ring 0, Crate `ports`) `[K]`

Ports sind die einzigen Schnittstellen zwischen Kern und Außenwelt. Konkrete Implementierungen wählt nur die Fassade (§22). Die Methodenauswahl benennt den Vertrag, nicht die vollständige Signatur.

### A.1 Ports des Fundaments (`ports`)

|Port|Aufgabe|Zentrale Methoden|Sicherheitsport (INV-WIRING-2)|
|---|---|---|---|
|`StorageEngine`|Schlüssel-Wert-Zugriff mit Snapshot- und SSI-Verfolgung|`get`, `get_at_seq`, `get_tracked`, `get_at_seq_tracked`, `scan_prefix_tracked`, `supports_ssi_tracking`, `put`, `put_if_absent`, `put_batch`|nein|
|`Snapshot`|Konkreter Sequenzstand|`seq_no`|nein|
|`VectorIndex`|Vektorsuche|`insert`, `insert_batch`, `search`, `search_at`, `search_filtered`, `all_doc_ids`|nein|
|`TextIndex`|Volltextsuche|`search`, `search_at`, `insert`, `delete`, `commit`, `rollback`, `rollback_to_tx`, `last_tx_id`, `len`, `stats`|nein|
|`GraphIndex`|Graphzugriff `[R]`|`traverse`, `neighbors`, `remove_edge`, `add_bidirectional`, `multi_traverse`, `multi_traverse_at`|nein|
|`HybridSearchProvider`|Hybridsuche über Fusion|`search_hybrid`|nein|
|`EmbeddingProvider`|Embeddings mit Modellidentität (INV-EMBED-1/-2)|`provider_name`, `embed`, `embed_batch`, `embedding_dim`|nein|
|`Reranker`|Cross-Encoder-Reranking `[R]`|`rerank`, `record_implicit_feedback`, `rerank_deadline_ms`|nein|
|`TextGenerator`|Textgenerierung `[R]`|`generate_text`|nein|
|`GroundingValidator`|Grounding-Prüfung `[R]`|`validate_grounding`|ja (`[R]`)|
|`KvPrefixStore`|Prefix-Cache `[R]`|`lookup`, `insert`, `evict`|nein|
|`Checkpoint`, `CheckpointCoordinator`|Snapshots, benannte Checkpoints, Restore|`take_snapshot`, `restore`, `create_named_checkpoint`, `restore_named_checkpoint`, `drop_named_checkpoint`, `list_named_checkpoints`|nein|
|`Clock`|Zeit (Wanduhr und monoton)|`now_unix_nanos`, `monotonic_nanos`|nein|
|`Rng`|Zufall (deterministisch seedbar außer Schlüsselmaterial)|`next_u64`, `fill_bytes`, `next_unit_f64`|nein|
|`IdGen`|ID-Erzeugung|`next_id`|nein|
|`MetricsSink`|Metriken ohne Nutzinhalte (INV-LOG-1)|`record_counter`, `record_gauge`, `record_histogram`|nein (Null-Objekt zulässig)|
|`LicenseGate`|Lizenzprüfung `[E]`|`check_ring`, `authorize`|nein|
|`KmsProvider`|Schlüsselerzeugung, -ableitung, -wrap, -vernichtung|erzeugen, ableiten, wrappen, entwrappen, vernichten|**ja**|
|`KeyCustodian`|Verwahrer für `EpochKek` (§20.2b)|`kind`, `create_kek`, `wrap`, `unwrap`, `destroy_kek`|**ja**|
|`ErasureParticipant`|Löschschicht (§20.15)|`layer`, `prepare`, `erase`, `measure`|**ja**|
|`ErasureJournal`|Durables Saga-Journal|`append`, `replay`|**ja**|
|`Vfs`|Gesamter Dateizugriff mit Durability-Primitiven (§7.8)|Öffnen, Lesen, Schreiben, `sync_file`, `sync_dir`, atomares Umbenennen, Entfernen, Sperren|**ja**|
|`DurableLog`|Append-only-Log mit `fsync` je Eintrag (KeyRegistry, RevocationLog)|`append`, `replay`, `truncate_checked`|**ja**|
|`NotaryPort`|Optionaler externer Zeitanker (z. B. RFC 3161)|Zeitstempel anfordern, Zeitstempel prüfen|nein (optional)|
|`ModuleVerifier`|Zulassung von Plugin-Modulen (§14.2)|`verify`|**ja** `[R]`|
|`EntityRecognizer`, `InjectionDetector`|Egress-Erkennung (§19)|`recognize`, `detect`|**ja** `[R]`|
|`GroupRevocationListener`|Widerruf von Gruppenberechtigungen|`on_revoked`|**ja**|

Zusatzregeln: (1) Jede Port-Implementierung für Produktion ist real; Null-, Test- und Accept-All-Implementierungen sind unzulässig außer bei Beobachtungs-Ports. (2) Test-Doubles (`FaultVfs`, `ManualClock`, Fake-Verwahrer, Platzhalter-Embedder) liegen in `testkit` oder hinter dem Feature `testing` (§6.10). (3) Ports, die Futures zurückgeben, tun dies nach P26.

### A.2 Ports der Ränder und Beobachtungs-Ports `[K]`

Diese Ports sind in `ports` definiert (§6.10 Regel 9), auch wenn ihre Implementierung in einem Rand-Crate liegt. Die Verdrahtung „Ohne Rand“ steht in §6.11.

|Port|Crate der Implementierung|Zentrale Methoden|Sicherheitsport (INV-WIRING-2)|
|---|---|---|---|
|`AdaptiveController`|`adapt` `[R]`|`observe`, `current_params`|nein|
|`Calibrator`|`rank` `[R]`|`calibrate`, `explain`|nein|
|`EgressChannel`|`privacy` `[R]`|`send_checked`|**ja** `[R]`|
|`FilterModule`|`sandbox` `[R]`|`apply`|**ja** `[R]` (nur nach `ModuleVerifier`)|
|`QueryRewriter`|`cognition` `[R]`|`rewrite`|nein|
|`BlockCache`|`store` (Kern), `kvcache` `[R]`|`get`, `insert`, `evict`|nein|
|`MergeOperator`|`store` (Rust- oder WASM-Variante)|`full_merge`, `partial_merge`|nein (WASM-Variante über `ModuleVerifier`)|
|`ReplayProgressSink`|`engine` (Beobachtung)|`report`|nein (Null-Objekt zulässig)|

**Namensauflösung.** Die Körpertexte verwenden zum Teil abweichende Bezeichner. Sie sind wie folgt aufgelöst; Namen ohne Zuordnung sind Gate-Fehler, bis sie einem Port, einem Typ oder einer Schicht zugeordnet sind (INV-DOC-SINGLE-SOURCE-1).

|Im Text verwendet|Auflösung|
|---|---|
|`IdGenerator`|Port `IdGen`|
|`DurableAppendLog`|Port `DurableLog`|
|`GaspValidator`, `ResponseGroundingValidator`|Port `GroundingValidator`|
|`CrossEncoderReranker`|Implementierung von `Reranker`|
|`IsotonicCalibrator`|Implementierung von `Calibrator`|
|`PidLatencyController`|Implementierung von `AdaptiveController`|
|`SignedLicenseGate`|Implementierung von `LicenseGate` (Ed25519)|
|`KvCache`|Implementierung von `KvPrefixStore` und Löschschicht `KvCache` (§20.4a)|
|`EmbeddingCache`|Cache-Schicht unter `CachesAndVault` (§20.4a), kein Port|
|`EpochKeyStore`|In-Memory-Algebra in `crypto` (§20.2b), kein Port|
|`ErasureKeyStore`|Sidecar `MFEK` (§7.11), kein Port|
|`AdmittedModule`, `LayerWitness`|Typen (§14.2, §20.4), kein Port|
|`AdaptiveDecayController`, `ContextPrefixEngine`, `MultiStepEngine`, `ContextEditAuditSink`|**ungeklärt**; Zuordnung vor Abnahme von PH8 festzulegen|

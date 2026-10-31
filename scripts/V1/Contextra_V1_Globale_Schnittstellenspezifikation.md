# Contextra V1 – Globale Schnittstellenspezifikation des SOLL-Endprodukts

| Feld | Wert |
|---|---|
| Dokumentstatus | Normative Spezifikation, Entwurf zur Umsetzung. Alle Entscheidungen sind final getroffen und begründet. |
| Datum | 31.10.2026 |
| Gegenstand | Der fokussierte, datenschutzkonforme Kern („Contextra Core V1“) und seine Erweiterungsschnittstellen für ein getrenntes Enterprise-Repository |
| Analysebasis | Quelltext des Repositorys `tfufuz1/contextra` am Remote-Stand `dba9986` (31.10.2026, flacher Klon, nach Force-Update neu gezogen) |
| Charakter | **Nichts wurde kompiliert, getestet oder gemessen.** Die Sandbox hatte keine Rust-Toolchain. Alle Rust-Signaturen sind Entwurf und nicht compilergeprüft. |
| Rechtlicher Vorbehalt | Keine Rechtsberatung. Die Rechtsaussagen stützen sich teils auf Sekundärquellen und sind vor jeder Außenverwendung am Primärtext zu verifizieren (Abschnitt 3.7). |

Das Dokument ist eigenständig. Es verweist auf kein anderes Dokument. Alle benötigten Ergebnisse, Befunde, Rechtsaussagen und Entscheidungen stehen darin.

---

## 0. Lesehilfe

### 0.1 Normative Schlüsselwörter

| Wort | Bedeutung |
|---|---|
| **MUSS / DARF NICHT** | Zwingende Anforderung. Verstoß ist ein Fehler im Produkt. |
| **SOLL / SOLL NICHT** | Regelfall. Abweichung braucht eine dokumentierte Begründung. |
| **KANN** | Optional. |

### 0.2 Evidenzmarken

| Marke | Bedeutung |
|---|---|
| **[K]** | Am Quelltext des Stands `dba9986` gelesen. Nicht kompiliert, nicht ausgeführt. |
| **[Q]** | Ergebnis früherer Prüfungen (Build- und Testläufe, Audits). Hier nicht erneut geprüft. |
| **[E]** | Entwurfsentscheidung dieses Dokuments. |
| **[S]** | Schätzung oder Zielwert. Keine Messung. Zielwerte sind Vorschläge für Release-Gates. |
| **[?]** | Aus dem Gedächtnis (Fremdbibliothek oder Rechtsfundstelle). Vor Übernahme prüfen. |

### 0.3 Rechtsmarken

| Marke | Bedeutung |
|---|---|
| **[R]** | Verbindliches Recht: Gesetz, Verordnung, EuGH- oder BGH-Urteil. |
| **[R-Instanz]** | Instanzgericht. Bindet nicht über den Einzelfall hinaus. |
| **[B]** | Behördenauslegung (EDPB, DSK, Landesbehörden, Standard-Datenschutzmodell). Nicht bindend, aber Maßstab der Aufsichtspraxis. |
| **[P]** | Literatur, Praxis, technische Norm. |
| **Geprüft** | Am Primärtext oder einer Primärquelle abgeglichen. |
| **Sekundär** | Nur über Sekundärbericht oder Leitsatz belegt. Volltext vor Verwendung prüfen. |
| **Offen / Umstritten** | Keine belastbare Antwort vorhanden. |

---

## 1. Ergebnis in Kürze

1. **Produktzuschnitt.** Contextra Core V1 ist eine eingebettete, ein-Knoten-fähige Bibliothek für verschlüsselte, mandantengetrennte Hybrid-Suche (Vektor plus deutsche Volltextsuche) mit **dokumentgenauer Löschung**, Dokumenten-Import und Compliance-Export. Alles andere (Graph, Agenten, Routing, Lizenz, Cloud, RBAC, TPM/HSM, Hosting) liegt hinter stabilen Erweiterungsschnittstellen im getrennten Enterprise-Repository oder in einem Labor-Repository.
2. **Sechs Außenflächen.** Rust-API (Fassade), Service-Provider-Schnittstellen (Ports), On-Disk-Formate, MCP-Server, Python-Bindings, Kommandozeile. Dazu ein signiertes Löschprotokoll mit externem Prüfer.
3. **Das Rückgrat ist die Löschkette „Erasure by Design“.** Jedes Quelldokument hat einen eigenen zufälligen Schlüssel (`DocDek`). Jeder Chunk hat einen dichten Slot, eine gemeinsame Lebend-Maske (`LiveMask`) sperrt ihn in allen Indizes. Löschen läuft in zwei Takten: Takt 1 (synchron, durabel) sperrt und liefert Klasse **L**; Takt 2 (asynchron, gebündelt) bereinigt Klartext-Strukturen (**P**) und vernichtet den Schlüssel (**C**).
4. **Sichere Voreinstellungen sind nicht abschaltbar.** Verschlüsselung ist immer an. Mandantenpflicht ist Standard. Automatische LLM-Entitätsextraktion existiert im Kern nicht. Es gibt keinen ausgehenden Netzverkehr außer zu einem ausdrücklich konfigurierten, standardmäßig nur lokalen Embedding-Dienst.
5. **Der Speicher wird getauscht.** Der Eigenbau-LSM entfällt aus dem Kern. Ersatz ist ein schmaler Port `KvStore` mit einem `redb`-Adapter [?]. Der Eigenbau-Index (HNSW mit Filter) und die deutsche Volltextsuche bleiben, weil es dafür keine etablierte Lösung mit gleicher Löschbarkeit gibt.
6. **Rechtlich ist die Schlüsselvernichtung als „Löschung“ nach Art. 17 DSGVO offen.** Weder Aufsicht noch Gericht hat sie anerkannt oder abgelehnt. Das Produkt gibt deshalb **nie** ein absolutes Löschversprechen ab, sondern einen **Löschbericht mit Klasse und deklarierten Restrisiken**.
7. **Ehrliche Grenze.** Ohne löschbare Schlüsselablage (TPM, Secure Element, OS-Keystore) bleibt auf Flash-Medien physische Remanenz alter Schlüsselseiten ein deklariertes Restrisiko. Das ist physikalisch und steht in jedem Bericht.
8. **Release-Voraussetzung.** Vor einer Außenaussage zur Löschwirkung sind erforderlich: eine laufende Testsuite, die Gates G-E1 bis G-E12 (Abschnitt 14), ein externes Krypto-Review und die Prüfung durch Fachanwalt oder Datenschutzbeauftragte.

---

## 2. Produktentscheidungen (final)

### 2.1 Zielbild und Abgrenzung

**Zielnutzer:** Softwarehäuser und Systemintegratoren, die lokale LLM-Lösungen für den Mittelstand in Deutschland und Europa bauen. Der **Anwender** (Kunde des Integrators) ist der datenschutzrechtlich Verantwortliche. Der Hersteller des Kerns liefert Software ohne Zugriff auf Daten und Schlüssel.

**Was V1 ist:** eine Bibliothek, die man in den eigenen Prozess einbindet (Rust, Python) oder als Prozess für Agenten betreibt (MCP-Server).

**Was V1 nicht ist:** ein verteiltes System, ein Hosting-Dienst, ein Agenten-Framework, ein Modell-Runtime, eine Lizenzplattform.

### 2.2 Verbindliche Zuordnung: Kern, Enterprise, Labor, gestrichen

Maßstab: Ein Baustein bleibt im Kern, wenn (a) die Datenschutzaussage des Produkts ohne ihn nicht trägt oder (b) es für die Aufgabe keine etablierte Lösung gibt, die dieselbe Löschbarkeit liefert, und (c) er prüfbar ist. Ein Eigenbau ersetzt nur dann eine etablierte Lösung, wenn er bewiesen nicht schwächer ist (Korrektheit, Haltbarkeit, mindestens das 1,5-Fache der Referenz in der rechtfertigenden Kennzahl, Wartbarkeit).

| Baustein (Bestand) | Entscheidung | Ziel | Begründung |
|---|---|---|---|
| `types`, `ports` | **Kern** | Ring 0 | Fundament, 25 bis 30 abhängige Crates [K]. Speicher-Port wird verschlankt. |
| `sys` (mlock, Dateirechte) | **Kern, verschlankt** | Ring 0 (einzige `unsafe`-Insel) | Schutz von Schlüsselmaterial vor Auslagerung. |
| `crypto` (Primitive, KDF, Sealed, Ed25519, Hash-Kette) | **Kern, verschlankt** | Ring 1 | Primitive sind etablierte RustCrypto-Bausteine [K]. Streichen: `kv_*`, `wal_crypto`, `anti_tamper`, `deletion_proof/*` in heutiger Form. |
| Schlüsselverwaltung (Hierarchie, Ledger, `KeyVault`) | **Kern, neu** | Ring 1 (`contextra-keys`) | Heute existiert keine Dokument-Schlüsselvernichtung [K]. |
| `store` (eigener LSM, WAL, SSTable, Compaction) | **Ersetzen** | `contextra-store` = Adapter über `KvStore` | Größtes Risiko, Crash-Durability und Hänger in Tests [Q]. Kein belegbarer Vorteil für Ein-Knoten-RAG [S]. |
| `mvcc` (SSI) | **Streichen**, Basistypen in `types` | – | Ein Schreiber. Leser-Isolation liefert der Speicher. |
| `vector` (HNSW, ACORN-Filter) | **Kern, mit Gate** | Ring 2 | Filterbare Suche und Slot-Nullung sind nötig. DiskANN, RaBitQ, Partial-Rebuild entfallen. |
| `simd` (AVX2/AVX-512/NEON, 223 bis 271 `unsafe`) | **Aus dem Kern** | optional, Labor | u8-Kerne nachweislich falsch [Q]. Kern nutzt sichere portable Kernel. |
| `text` (BM25, deutsche Komposita) | **Kern** | Ring 2 | Alleinstellung, kein Dateizugriff, Postings liegen im verschlüsselten Speicher [K]. |
| `rank` (Fusion) | **Kern, verschlankt** | Ring 2 | RRF und Gewichtung. Ungenutztes streichen. |
| `ingest` | **Kern, verdrahten, isolieren** | Ring 2 | Ohne Import kein RAG. Heute von keinem Crate eingebunden [K]. |
| `privacy` (PII-Muster, Prüfziffern, Egress-Policy) | **Kern, verschlankt** | Ring 2 | Datenminimierung beim Import, Egress fail-closed. Tokenisierungs-Tresor und Injektionsschutz wandern nach Enterprise. |
| `engine` | **Kern, umgebaut** | Ring 3 | Neuer `Eraser`, `LiveMask`, Slot-Katalog, Worker. Harte Kopplung an Graph und Checkpoint entfällt. |
| `db`, `contextra` (Fassaden) | **Kern, zusammengeführt** | Ring 4 (`contextra`) | Eine Fassade statt zwei. |
| `mcp` | **Kern, Transport ersetzen** | Ring 4 (Binary) | Tools und Bestätigungslogik bleiben. Transport über offizielles SDK [?]. |
| `py` | **Kern** | Ring 4 | PyO3-Standardweg. |
| `infer-ollama` | **Kern, als Adapter** | `contextra-embed-ollama` | Lokaler Embedding-Dienst, nur Loopback als Standard. |
| `audit-export` | **Kern** | `contextra-compliance` | Verarbeitungsverzeichnis, TOM-Bericht. |
| `avv-generator` | **Enterprise** | – | Nur nötig, wenn der Integrator hostet. |
| `testkit` | **Kern** | Ring 5 | Konformitätssuiten sind der Schiedsrichter für jeden Austausch. |
| `infer-onnx`, `infer-candle` | **Enterprise/Beitrag** | – | In-Process-Inferenz, `ort` ist Release Candidate. |
| `graph`, `cognition`, `adapt`, `router`, `agent` | **Labor** | `contextra-labs` | Für v1 nicht nötig oder forschungsnah. |
| `checkpoint` (Time-Travel) | **Streichen** | – | Rekonstruierbare Zustände halten gelöschte Daten vor (Konflikt mit Art. 17). |
| `sandbox` (wasmtime) | **Enterprise** | – | Parser-Isolation läuft über Prozess und OS-Sandbox. |
| `license` | **Enterprise** | – | Lizenzprüfung in einer MIT/Apache-Bibliothek schützt nichts. |
| `wire` (FlatBuffers) | **Streichen** | – | 45 bis 47 `unsafe` im generierten Code [Q]. Eine In-Process-Bibliothek braucht kein Zero-Copy-IPC. |
| `kvcache`, `crypto/kv_*` | **Streichen** | – | Der „Tensor“ war ein Textstring, Prefill lief immer [Q]. |
| `durable-fs`, `core`, `bench` (Altbestand) | **Streichen** | – | Nicht kompilierbar, ohne Nutzer. Benchmarks wandern nach `benches/`. |

### 2.3 Folgen der Entscheidungen (Kompromisse)

| Entscheidung | Gewinn | Preis |
|---|---|---|
| Speicher-Tausch | Rund 18.700 Zeilen Eigencode und 33.400 Testzeilen entfallen aus dem Kernpfad [Q]. Crash-Konsistenz kommt aus einer breit eingesetzten Komponente [?]. | Ein-Schreiber-Modell. Kein WAL-HMAC mehr. Migration ohne Altbestand nötig (v0.1.0, keine Produktivinstallationen bekannt). |
| Kein Time-Travel | Löschung wirkt auf alle Zustände. | Keine historische Rekonstruktion. |
| Keine Auto-Extraktion im Kern | Kein Datenabfluss an ein Modell ohne Zutun. | Wissensgraph-Funktionen nur in Labor/Enterprise. |
| Blinde Terme im Textindex | Persistierter Index enthält keine lesbaren Wörter. | Keine Präfix- und Wildcard-Suche auf persistierten Daten. |
| Kein Schlüsselteilen zwischen Dokumenten | „Ein Schlüssel je Dokument“ bleibt wahr. | Kein inhaltsadressierter Chunk-Speicher über Dokumente hinweg. |
| Sync-Ports | Deterministisch testbar, keine `block_on`-Deadlocks, cancel-sicher. | Async-Nutzer rufen über einen Adapter (`spawn_blocking`). |

---

## 3. Rechtsrahmen als Schnittstellenanforderung

Dieser Abschnitt enthält alles, was für die Rechtsgültigkeit des Produkts relevant ist, und übersetzt es in prüfbare Anforderungen. Er ist keine Rechtsberatung.

### 3.1 Was gesichert ist, was umstritten, was offen

| # | Aussage | Status | Fundstelle | Prüfstatus |
|---|---|---|---|---|
| 1 | „Löschen“ und „Vernichten“ sind selbst Verarbeitung und unterliegen den Grundsätzen des Art. 5 und der Sicherheit nach Art. 32. | Gesichert [R] | Art. 4 Nr. 2 DSGVO | Geprüft |
| 2 | Die DSGVO definiert „Löschung“ nicht und schreibt keine Methode vor. Behördlicher Maßstab: Unkenntlichmachen, sodass keine Person mehr identifizierbar und der Informationsgehalt nicht oder nach allgemeinem Ermessen nur unwahrscheinlich reproduzierbar ist; erst vollzogen, wenn keine Kopie mehr beim Verantwortlichen oder Auftragsverarbeiter besteht. | Gesichert (Text), Maßstab [B] | Art. 17 DSGVO; Standard-Datenschutzmodell, Baustein 60 „Löschen und Vernichten“ V1.0a (S. 1 bis 3) | Geprüft |
| 3 | Verschlüsselte Daten bleiben personenbezogen, solange irgendein Schlüssel existiert. | Gesichert | Art. 4 Nr. 5, Erwägungsgrund 26; EDPB-Leitlinien 02/2025 Rn. 51 | Geprüft/Sekundär |
| 4 | **Schlüsselvernichtung ist eine Löschung im Sinne des Art. 17.** | **Offen.** Keine Aufsichts- oder Gerichtsaussage gefunden, die sie anerkennt oder ablehnt. | EDPB 02/2025 nennt nur Bedingungen (Blockchain-Kontext) | Geprüft |
| 5 | Anonymisierung kann die Löschpflicht erfüllen. | **Umstritten.** Instanzgerichte bejahen. Aufsicht kritisiert, dass in der Praxis nur pseudonymisiert oder maskiert wird. Leitlinien ausstehend. | OLG Dresden 03.02.2026, 4 U 292/25; LG München II 14.03.2025, 14 O 1461/24; EDPB-Bericht zur koordinierten Prüfung 2025 (10.02.2026) | Sekundär |
| 6 | Cryptographic Erase ist technisch als Purge-Methode anerkannt, unter strengen Voraussetzungen. Rechtliche Wirkung offen. | Gesichert (technisch) | NIST SP 800-88 Rev. 2 (26.09.2025) | Sekundär |
| 7 | Backups müssen nicht bei jedem Antrag sofort verändert werden. Nach einer Rücksicherung muss unmittelbar erneut gelöscht werden. Eine feste Höchstfrist nennt keine Behörde. Rotationsfristen von 30/60/90 Tagen sind Praxisempfehlung. | Überwiegend akzeptiert [B]; Fristen ungeregelt | SDM Baustein 60 (M60.S10, M60.P08); EDPB-Bericht 2025 (Abschn. zu Backups) | Geprüft/Sekundär |
| 8 | § 35 Abs. 1 BDSG deckt weder automatisierte Verarbeitung (RAG-Indizes, Backups) noch unrechtmäßige Verarbeitung. Zweifel an Unionsrechtskonformität. | Gesichert (Wortlaut) | § 35 Abs. 1 BDSG | Geprüft |
| 9 | Vektor-Embeddings in einer RAG-Datenbank sind personenbezogen. | **Offen.** Vorsorglich so behandeln: gemeinsame Speicherung mit Chunk-Text, Verknüpfung über Kennungen, belegte Rekonstruierbarkeit. | DSK-Orientierungshilfe RAG (10/2025); EDPB-Stellungnahme 28/2024 (betrifft Modelle, nicht Vektordatenbanken) | Geprüft |
| 10 | RAG-Referenzdokumente und Vektoreinträge sind gezielt adressier- und löschbar. Betroffenenrechte im Sprachmodell selbst bleiben ungelöst. | [B] | DSK-Orientierungshilfe RAG | Geprüft |
| 11 | Kein Anspruch der betroffenen Person auf Aushändigung eines individuellen Löschnachweises außerhalb eines Streitfalls. | Erstinstanzlich | AG Lörrach 03.03.2025, 3 C 1099/24 | Sekundär |
| 12 | Der Verantwortliche muss Löschung gegenüber Aufsicht und im Streit **nachweisen können**. | Gesichert | Art. 5 Abs. 2, 24, 32; EuGH C-340/21 (14.12.2023) | Geprüft/Unverifiziert |
| 13 | Löschprotokolle dürfen keine Daten enthalten, für die eine Löschpflicht besteht. Ob und wie lange ein pseudonymer Löschnachweis mit Personenbezug zulässig ist, ist umstritten. | [B]; Reichweite umstritten | SDM M60.S05/S06; Art. 11 DSGVO (Literatur) | Geprüft/Sekundär |
| 14 | Ein Anbieter lokal installierter Software ohne Zugriff auf Klartext, Chunks, Vektoren und Schlüssel ist in der Regel weder Verantwortlicher noch Auftragsverarbeiter. | Überwiegend vertreten, einzelfallabhängig | Art. 4 Nr. 7, 8, Art. 28; EDPB-Leitlinien 07/2020; DSK-Orientierungshilfe RAG (unterstellt On-Premise) | Geprüft/Sekundär |
| 15 | Art. 25 DSGVO bindet den Verantwortlichen, nicht den Hersteller. Die DSK fordert eine Gesetzesänderung. | Gesichert | DSK-Entschließung 12.12.2025 | Geprüft |
| 16 | Aufbewahrungspflichten gehen vor, werden aber je Datum geprüft. Dann Einschränkung (Art. 18) statt Löschung. | Gesichert | Art. 17 Abs. 3 lit. b, e; OLG Dresden 14.12.2021, 4 U 1278/21 | Geprüft/Sekundär |
| 17 | KI-Verordnung Art. 12 (Aufzeichnung) gilt nur für Hochrisiko-Systeme und verlangt Ereignisprotokolle, nicht die Klartextspeicherung von Ein- und Ausgaben. Betreiber-Aufbewahrung mindestens sechs Monate, **sofern Unionsrecht, insbesondere Datenschutzrecht, nichts anderes vorsieht**. | Gesichert (Text) | VO (EU) 2024/1689 Art. 12, 19, 26 Abs. 6 | Unverifiziert |
| 18 | Hochrisiko-Fristen (Anhang III) wurden auf den 02.12.2027 verschoben. | Sekundär | „Digital Omnibus on AI“, VO (EU) 2026/1744 vom 08.07.2026 | Sekundär: Amtsblatt prüfen |
| 19 | Art. 10 KI-Verordnung erfasst RAG-Wissensbasen. | **Offen** | Literatur uneinig | Unverifiziert |
| 20 | Pauschale Werbeaussagen („garantiert“, „DSGVO-konform“, „erfüllt Art. 17 automatisch“) bergen Risiko nach § 5 UWG und § 434 BGB. Einschlägige Rechtsprechung zu Löschversprechen von Software wurde nicht gefunden. | Risiko | § 5, § 8 Abs. 3 UWG; § 434 BGB | Unverifiziert |

### 3.2 Aufbewahrungsfristen (aktuell)

| Unterlagen | Frist | Norm |
|---|---|---|
| Bücher, Aufzeichnungen, Inventare, Jahresabschlüsse, Lageberichte | 10 Jahre | § 147 AO; § 257 HGB |
| **Buchungsbelege** (z. B. Rechnungen) | **8 Jahre** seit 01.01.2025 (Bürokratieentlastungsgesetz IV); Banken, Versicherungen und Wertpapierinstitute weiter 10 Jahre | § 147 Abs. 3 AO; § 257 Abs. 4 HGB |
| Handels- und Geschäftsbriefe, auch E-Mails | 6 Jahre | § 147 AO; § 257 HGB |
| Patientenakte | 10 Jahre nach Abschluss der Behandlung, soweit nicht länger | § 630f Abs. 3 BGB |

§ 203 StGB ist eine Schweigepflicht und keine Aufbewahrungsnorm.

### 3.3 Entscheidungen und häufige Fehlzuordnungen

| Entscheidung | Inhalt | Status |
|---|---|---|
| EuGH C-582/14 (Breyer) und C-413/23 P (EDPS/SRB, 04.09.2025) | Relativer Personenbezug: maßgeblich sind die Mittel, die der Stelle vernünftigerweise zur Re-Identifizierung zur Verfügung stehen. Für den Schlüsselhalter bleiben Daten personenbezogen. | Unverifiziert: Volltext prüfen |
| EuGH C-129/21 (Proximus, 27.10.2022) | Art. 17 Abs. 2: angemessene Maßnahmen, andere Verantwortliche über das Löschverlangen zu informieren. | Unverifiziert |
| EuGH C-340/21 (14.12.2023) | Der Verantwortliche muss die Angemessenheit seiner Sicherheitsmaßnahmen nachweisen. | Unverifiziert |
| EuGH C-40/17 (Fashion ID, 29.07.2019) | Gemeinsame Verantwortlichkeit. | Unverifiziert |
| EuGH C-460/20 (08.12.2022) | **Auslistung** bei offensichtlicher Unrichtigkeit, keine allgemeine Löschpflicht. | Unverifiziert |
| EuGH C-300/21 (04.05.2023) | Art. 82: keine Erheblichkeitsschwelle für immateriellen Schaden. | Unverifiziert |
| OLG Dresden, 03.02.2026, 4 U 292/25 | Anonymisierung als „Minus zur Löschung“, wenn die betroffene Person die Entfernung ihres Personenbezugs verlangt. | Sekundär (Leitsatz) |
| OLG Dresden, 14.12.2021, 4 U 1278/21 | Inkasso, namensgleiche Person: Aufbewahrungspflichten sind je Datum zu prüfen. Enthält die „Minus“-Formel **nicht**. | Sekundär |
| LG München II, 14.03.2025, 14 O 1461/24 | Anonymisierung als Unterfall der Löschung. | Sekundär (Leitsatz) |
| OLG München, 22.03.2022, 18 U 1697/21 Pre | Löschbegriff autonom auszulegen. Gelöschte Daten dürfen nicht erneut verarbeitet werden. | Sekundär |
| AG Lörrach, 03.03.2025, 3 C 1099/24 | Nachbarn, Videoaufnahmen. Löschung ja, Anspruch auf Löschnachweis nein. | Sekundär |
| OLG Dresden, 4 U 940/24 | Der Verantwortliche muss sich die Löschung durch den Auftragsverarbeiter nachweisen lassen. | Sekundär (Datum offen) |
| BGH VI ZR 10/24 (18.11.2024) | Scraping, nach Kenntnisstand Schadensersatz bei Kontrollverlust, **keine** Löschentscheidung. | Unverifiziert |

**Keine Entscheidung billigt Crypto-Shredding. Keine beanstandet ein Löschverfahren wegen technischer Unzulänglichkeit.** Das ist ein Nicht-Fund, kein Beleg der Nichtexistenz.

**Fehlzuordnungen, die in Produktunterlagen nicht vorkommen dürfen:**

- Eine Entscheidung aus 2021 belege „Anonymisierung ist ein Minus zur Löschung“. Richtig ist das Urteil vom 03.02.2026.
- Ein Gericht sehe verschlüsselten Chiffretext als hinreichend anonymisiert an. Nicht belegt.
- Die BSI-Richtlinie TR-02102 empfehle kryptografisches Löschen. Sie regelt Algorithmen und Schlüssellängen, nicht das Löschen.
- Die DSK-Orientierungshilfe zu RAG nenne Crypto-Shredding. Sie erwähnt es nicht.
- Eine Aufsichtsbehörde akzeptiere Backup-Rotation von 30, 60 oder 90 Tagen. Keine Primärquelle.
- Der EDPB-Beschluss zu KI-Modellen (28/2024) belege, dass Vektordatenbank-Embeddings personenbezogen sind. Er betrifft Modelle.
- § 35 BDSG „analog“ rechtfertige das Nicht-Löschen in Backups. Die Norm gilt nur für nicht automatisierte Verarbeitung.

### 3.4 Die EDPB-Aussage zur Schlüsselvernichtung

Die einzige gefundene Aufsichtsaussage steht in den EDPB-Leitlinien 02/2025 (Blockchain, Endfassung 07.07.2026). Sinngemäß: Nach Vernichtung des Schlüssels sind verschlüsselte Daten nur unverständlich, **bis** der Algorithmus gebrochen wird, Entschlüsselungstechniken (etwa Quantencomputer) fortschreiten oder der Schlüssel zuvor kompromittiert oder kopiert wurde. Verschlüsselte personenbezogene Daten bleiben personenbezogen. Bei unbegrenzter Speicherung wird selbst perfekt implementierte Verschlüsselung von der Zeit überholt. Verlangt werden Notfallplan für den Algorithmenwechsel, Risikobewertung unter Einbeziehung von Obsoleszenz und periodische Neubewertung sowie Erasure by Design. Die Übertragung auf RAG-Systeme ist eine Analogie.

### 3.5 Bedingungen, unter denen die Argumentation „Schlüsselvernichtung = Unkenntlichmachen“ trägt

Diese zehn Bedingungen sind die Abnahmekriterien der Löschkette. Spalte „Schnittstelle“ nennt, wo sie umgesetzt und geprüft werden.

| # | Bedingung | Schnittstelle in V1 |
|---|---|---|
| B1 | Schlüssel je löschbarer Einheit (Dokument, Betroffener). Mandanten- oder Collection-Schlüssel allein führt zu Kollateralverlust oder unterbliebener Löschung. | `DocDek`, Abschnitt 6 |
| B2 | Vernichtung **aller** Schlüsselkopien (Schlüsselspeicher, Backups des Schlüsselspeichers, Escrow, Swap, Crash-Dumps). | `KeyVault::destroy_batch`, `ResidualRisk::BackupKeyCopies`, `sys::harden_process` |
| B3 | Kein unverschlüsselter Parallelbestand: Embeddings, Volltextindex, Caches, Protokolle, temporäre Dateien. | `LiveMask`, `LayerPurger`, `Purgeable`-Register, Blind-Terme |
| B4 | Verbleibende Metadaten (IDs, Größen, Zeitstempel, Schlüssel-IDs) identifizieren nicht wieder. | Zufalls-IDs, Padding, Commitments, inhaltsfreie Protokolle |
| B5 | Stand der Technik und Algorithmenwechsel: Risikobewertung, periodische Neubewertung, Migrationsverfahren. | `SuiteId` im Umschlag, `KdfHeader`, `contextra migrate-suite` |
| B6 | Passphrase-Ableitung schließt Brute-Force praktisch aus. | Argon2id mit Mindestparametern, `KdfHeader` |
| B7 | Echte Vernichtung: überschreiben, `zeroize`, kein Auslagern. | `KeyVault`, `sys::lock_memory` |
| B8 | Wartefristen bei Cloud-Schlüsselverwaltung sind dokumentiert (AWS KMS 7 bis 30 Tage, Azure Key Vault 7 bis 90 Tage, Google Cloud KMS standardmäßig 30 Tage laut Fachangaben, vor Verwendung gegen Herstellerdokumentation prüfen). | `VaultCaps::destruction_delay` (Enterprise-Adapter) |
| B9 | Keine Kompromittierung vor Vernichtung (kein Export, keine Kopie in unkontrollierte Systeme). | `SecretString`, kein `Serialize`/`Debug` auf Geheimnissen |
| B10 | Nachweisbarkeit des Schlüssellebenszyklus. | `ErasureReceipt`, `AuditChain` |

### 3.6 Anforderungsmatrix (Recht → Schnittstelle → Gate)

| ID | Anforderung | Rechtsgrundlage | Umsetzung | Gate |
|---|---|---|---|---|
| RQ-01 | Löschung ohne unangemessene Verzögerung, Frist Art. 12 Abs. 3 (ein Monat). Wirksam sofort durch Lesesperre, Klartext binnen Sekunden bis Minuten. | Art. 17 Abs. 1, Art. 12 Abs. 3 | Zwei-Takt-Löschung, `erase_sla` | G-E1, G-E3, G-E12 |
| RQ-02 | Löschung je Dokument und je betroffener Person. | Art. 17 | `erase_document`, `erase_subject` | G-E1, G-E7 |
| RQ-03 | Löschung wirkt in allen Repräsentationen (Chunk, Vektor, Volltext, Cache). | SDM Baustein 60; Art. 5 Abs. 1 lit. e | `LayerCatalog`, `LayerPurger` | G-E5, G-E6 |
| RQ-04 | Nachweis der Löschung ohne Inhalte und Klartext-Kennungen. | Art. 5 Abs. 2, Art. 24; SDM M60.S05/S06 | `ErasureReceipt`, Commitments | G-E5, G-E10 |
| RQ-05 | Backups: nach Restore erneut löschen. | SDM M60.S10/P08 | Restore-Guard, `ErasedSet` | G-E11 |
| RQ-06 | Aufbewahrungspflicht: Einschränkung statt Löschung, Dokument nicht mehr im Retrieval. | Art. 17 Abs. 3, Art. 18 | `restrict`, Zustand `Restricted` | G-E13 |
| RQ-07 | Speicherbegrenzung und Löschfristen. | Art. 5 Abs. 1 lit. e, Art. 30 Abs. 1 lit. f | `CollectionConfig::retention`, Reaper | G-E14 |
| RQ-08 | Datenschutz durch Voreinstellung: Verschlüsselung an, Mandantenpflicht, kein Fremdabfluss. | Art. 25 Abs. 2, Art. 32 Abs. 1 lit. a | Konfigurationsschnittstelle (Abschnitt 5) | G-S1 bis G-S6 |
| RQ-09 | Datenminimierung beim Import. | Art. 5 Abs. 1 lit. c | `PiiPolicy` im Ingest | G-P1 |
| RQ-10 | Protokolle ohne Inhalt, entkoppelt von der Löschpflicht. | Art. 5, 32; KI-VO Art. 12 | `AuditSink`, inhaltsfreie Ereignisse | G-E10 |
| RQ-11 | Verzeichnis der Verarbeitungstätigkeiten, TOM-Beschreibung für DSFA. | Art. 30, Art. 35 | `contextra-compliance` | G-C1 |
| RQ-12 | Kryptografie nach Stand der Technik, Algorithmenwechsel möglich. | Art. 32 Abs. 1 lit. a; EDPB 02/2025 | `SuiteId`, Migration | G-K3 |
| RQ-13 | Keine Anbieter-Verarbeitung: kein Fernzugriff, keine Telemetrie, kein Schlüssel-Escrow, kein Lizenzserver im Kern. | Art. 4 Nr. 7, 8, Art. 28 | `EgressPolicy::LoopbackOnly`, Zero-Phone-Home-Test | G-R1 |
| RQ-14 | Ehrliche Außenkommunikation. | § 5 UWG, § 434 BGB | Wording-Lint, Pflicht-Restrisikoliste im Bericht | G-E10 |

### 3.7 Vor Außenverwendung zu verifizieren (Auftrag an Fachanwalt oder Datenschutzbeauftragte)

1. Rechtliche Qualifikation der Schlüsselvernichtung (Löschung, Anonymisierung, Unzugänglichmachung oder bloße Einschränkung) und die Haltung der zuständigen Aufsicht des Kunden.
2. Entfall des Personenbezugs nach Schlüsselvernichtung für jede Stelle (EuGH C-413/23 P, Volltext).
3. Welche Prüfungen belegen, dass kein Schlüsselmaterial verbleibt (Schlüsselvernichtungs-Audit, Pentest, Zertifizierung)?
4. Wirkungsbereich: Liegen Quelldokument, Chunk, Vektor, Volltextindex, Caches, Protokolle und Kontexte im Wirkungsbereich der Schlüsselvernichtung?
5. Behandlungsstrategie für Embeddings bis zur Klärung.
6. Zulässige Backup-Frist und Restore-Sperre; Verschlüsselung der Backups mit separatem Schlüssel.
7. Welche Metadaten des Löschberichts dürfen wie lange aufbewahrt werden (AG Lörrach, OLG Dresden 4 U 940/24, Art. 11 DSGVO)? Ist eine pseudonyme Lösch-ID personenbezogen?
8. Rolle des Anbieters bei Support, Updates, Schlüsselwiederherstellung und LLM-Anbindung.
9. Zusammenspiel mit § 147 AO, § 257 HGB, § 630f BGB; Trennung von Archiv und RAG-Index.
10. Wording jeder Werbeaussage (UWG, BGB), Haftungsbegrenzung, Kundenhinweise.
11. KI-Verordnung: Rolle als Anbieter, Risikoklasse typischer Einsätze, Fristen laut Amtsblatt, Protokollkonzept (Art. 12, 26 Abs. 6).
12. NIS2 und Cyber Resilience Act: Betroffenheit von Hersteller und Zielkunden. NIS2 Art. 21 Abs. 2 lit. h verlangt Konzepte für Kryptografie und Verschlüsselung, das Wort „Krypto-Agilität“ steht nicht in der Richtlinie. Die Erwartung des Algorithmenwechsels folgt aus Art. 32 DSGVO und EDPB 02/2025. Cyber-Resilience-Act-Meldepflichten ab 11.09.2026, Hauptpflichten ab 11.12.2027 laut Angabe [?].
13. Aktualität: EDPB-Anonymisierungsleitlinien, Endfassung der Leitlinien 01/2025 (Pseudonymisierung), mögliche Neufassung von SDM Baustein 60, Digital Omnibus.
14. Volltextprüfung der nur sekundär belegten Fundstellen: OLG Dresden 4 U 292/25, LG München II 14 O 1461/24, AG Lörrach 3 C 1099/24, OLG Dresden 4 U 940/24, EuGH C-413/23 P, EDPB-Bericht 2025 (Abschnitt Backups).

### 3.8 Zulässige und unzulässige Außenkommunikation

**Unzulässig:** „garantiert DSGVO-konform gelöscht“, „erfüllt Art. 17 automatisch“, „vergisst Daten“, „unwiederbringlich“, „informationstheoretisch unentschlüsselbar“ (AES-256-GCM-SIV ist rechnerisch, nicht informationstheoretisch sicher), jede Zertifizierungs- oder Behördenbilligungsbehauptung ohne Beleg.

**Zulässig (Vorschlag, nach Review):**

> Bei einem Löschaufruf wird das Dokument sofort für alle Lese- und Suchpfade gesperrt. Innerhalb eines konfigurierbaren Zeitfensters werden die Klartext-Strukturen im Arbeitsspeicher bereinigt und der dokumentspezifische Schlüssel vernichtet. Das System stellt einen signierten, datensparsamen Bericht über die gemessenen Zustände aus. Nicht erfasst sind externe Backups, Kopien außerhalb der Anwendung sowie Auslagerungsdateien und Speicherabbilder. Je nach Schlüsselablage kann auf Flash-Medien physisch Restmaterial verbleiben; der Bericht weist das aus. Die rechtliche Bewertung der Löschwirkung und die Einhaltung von Aufbewahrungsfristen verbleiben beim Betreiber.

**Pflichtinhalt jeder Produktbeschreibung:** was technisch geschieht, welche Voraussetzungen gelten (Konfiguration, Schlüsselablage), was nicht erfasst ist, und dass der Anwender die rechtliche Gesamtverantwortung trägt.

### 3.9 Rollenabgrenzung als Designregel

Der Hersteller bleibt außerhalb der Verarbeitung, solange der Kern nichts davon tut: Fernzugriff auf Daten, Telemetrie mit Personenbezug, Hosting, Schlüssel-Escrow oder -Wiederherstellung, Lizenz- und Update-Server mit Nutzerdaten, Weiterleitung an externe LLM-Dienste. **Der Kern tut davon nichts** (Anforderung RQ-13). Jede Enterprise-Funktion, die eines davon tut, ändert die Rolle des Betreibers dieser Funktion und braucht eine eigene Prüfung.

---

## 4. Architektur und Systemgrenzen

### 4.1 Crate-Landkarte des Kerns (SOLL)

Zwanzig Crates in fünf Ringen. Höhere Ringe hängen von niedrigeren ab, nie umgekehrt. `#![forbid(unsafe_code)]` gilt überall außer in `contextra-sys`.

| Ring | Crate | Zweck | Außenfläche |
|---|---|---|---|
| 0 | `contextra-types` | Reine Werttypen: IDs, Fehler, Konfigurationstypen, Klassen, Zustandsmaschine `step`. Kein I/O. | Rust-API (Typen) |
| 0 | `contextra-ports` | Alle Erweiterungsschnittstellen (Traits). Keine I/O-Abhängigkeit, kein `tokio`. | SPI (Abschnitt 9) |
| 0 | `contextra-sys` | Einzige `unsafe`-Insel: `mlock`, Core-Dump- und Swap-Härtung, Dateirechte. | intern |
| 1 | `contextra-crypto` | Primitive (AEAD, KDF, HMAC, Ed25519), Umschlag `Sealed`, Hash-Kette. | intern, Formate |
| 1 | `contextra-keys` | Schlüsselhierarchie, Ledger, `KeyVault`-Implementierung `FileVault`. | SPI-Implementierung |
| 1 | `contextra-store` | `KvStore`-Adapter (Standard: `redb` [?]), Slot-Katalog, `StdVfs`. | SPI-Implementierung |
| 2 | `contextra-vector` | HNSW mit Filter, flacher Exakt-Scan, Slot-Nullung, Shadow-Rebuild. | SPI-Implementierung |
| 2 | `contextra-text` | BM25, deutsche Morphologie, blinde Terme, Segmente. | SPI-Implementierung |
| 2 | `contextra-rank` | Fusion (RRF, gewichtet). | intern |
| 2 | `contextra-ingest` | Extraktion, Chunking, Limits, Worker-Prozess. | SPI und Binary `contextra-ingest-worker` |
| 2 | `contextra-privacy` | PII-Muster mit Prüfziffern, Egress-Policy. | SPI-Implementierung |
| 2 | `contextra-embed-ollama` | Embedding-Adapter für lokalen Ollama-Dienst. | SPI-Implementierung |
| 3 | `contextra-engine` | `Eraser`, `LiveMask`, Katalog, Epochen-Worker, Wiederanlauf, Retention-Reaper. | intern |
| 3 | `contextra-compliance` | Verarbeitungsverzeichnis, TOM-Bericht, Berichtsrenderer. | Rust-API, CLI |
| 4 | `contextra` | **Fassade und Composition Root.** Builder, Handles. | Rust-API |
| 4 | `contextra-mcp` | MCP-Server (Binary). | MCP |
| 4 | `contextra-py` | Python-Bindings. | Python |
| 4 | `contextra-cli` | Kommandozeile `contextra`. | CLI |
| 4 | `contextra-verify` | Eigenständiger Prüfer für Löschberichte und Audit-Ketten. Hängt nur von `types` und Ed25519 ab. | Rust-API, CLI |
| 5 | `contextra-testkit` | Konformitätssuiten, Fehlerinjektion (`FaultVfs`), deterministische Uhr und Zufall. Nur Dev. | Test-API |

### 4.2 Abhängigkeitsregeln (durchsetzbar per Test)

| Regel | Inhalt |
|---|---|
| R1 | Ringe nur abwärts. Die Liste der erlaubten Kanten steht als Datei im Repository, ein Test bricht bei neuer Kante („Ratchet“: die Liste darf nur schrumpfen). |
| R2 | Kein Crate des Kerns hängt von Enterprise oder Labor ab. |
| R3 | `unsafe` nur in `contextra-sys`. Ein Test zählt `unsafe` und scheitert bei Treffern außerhalb. |
| R4 | Kein `tokio` unter `[dependencies]` in Ring 0 bis 2. Async lebt an der Kante (MCP, Python-Runtime). |
| R5 | Dateizugriff nur über `Vfs` oder `KvStore`. Ein Test erkennt direkte `std::fs`-Aufrufe in Ring 1 bis 3 außerhalb der Adapter. |
| R6 | Kein `unwrap`, `expect`, `panic!`, Indexzugriff außerhalb von Tests (Clippy-`deny`). Nicht herabstufbar. |
| R7 | Kein Netzwerkcode außer in `contextra-embed-ollama` (und in Enterprise-Adaptern). Ein Test sucht nach `TcpStream`, `reqwest`, `ureq`, `hyper`. |
| R8 | Jede persistierte Struktur trägt eine Formatversion (Abschnitt 10). |

### 4.3 Kern und Enterprise: der Schnitt

Der Schnitt verläuft ausschließlich über Rust-Traits in `contextra-ports`. Es gibt keine Feature-Flags, die Enterprise-Verhalten im Kern freischalten, und kein dynamisches Laden von Bibliotheken.

| Mechanismus | Festlegung |
|---|---|
| Komposition | Zur Compile-Zeit über den Builder (`with_key_vault`, `with_audit_sink`, …). Das Enterprise-Repository liefert Adapter-Crates und ein eigenes Binary, das den Kern einbindet. |
| Stabilität | `contextra-ports` und `contextra-types` folgen strikt Semver ab 1.0. Öffentliche Enums und Structs sind `#[non_exhaustive]`. Neue Trait-Methoden bekommen eine Standardimplementierung oder erscheinen in einem neuen Trait. |
| Fähigkeiten | Jeder Port liefert `capabilities()`. Der Kern prüft die Fähigkeiten beim Start und schreibt sie in den Löschbericht (z. B. `Effaceable`). |
| Konformität | Jeder Adapter, auch aus dem Enterprise-Repository, MUSS die Konformitätssuite aus `contextra-testkit` bestehen. Ohne bestandene Suite ist der Adapter nicht zulässig. |
| Invarianten | Ein Adapter DARF die Invarianten des Kerns nicht lockern: Lesesperre vor Takt 1, Fail-closed, keine Inhalte in Protokollen, keine Telemetrie. Die Suite enthält Tests dafür. |
| Lizenz | Der Kern bleibt MIT OR Apache-2.0. Er enthält keine Lizenzprüfung. Kommerzielle Funktionen liegen vollständig im Enterprise-Repository. |

---

## 5. Globale Konventionen und Konfigurationsschnittstelle

### 5.1 Synchron statt asynchron

Alle Ports und die Kern-API sind **synchron** [E]. Gründe: deterministische Tests, keine `block_on`-Aufrufe in Bibliotheken (diese verursachen Deadlock-Risiko und verdeckten die heutigen Test-Hänger), klare Cancel-Semantik (ein abgebrochener Future ist der heutige Commit-Fehler, bei dem ein als fehlgeschlagen gemeldeter Commit nach Neustart doch wirksam wird [Q]). Asynchrone Nutzer (MCP-Server, Python) rufen über `spawn_blocking` auf. Ein optionales Crate `contextra-async` liefert diesen Adapter und gehört nicht zum Kern-Release.

### 5.2 Kennungen

| Typ | Darstellung | Vergabe | Bemerkung |
|---|---|---|---|
| `TenantId` | `u64` | Katalog, fortlaufend | Nie aus Namen gehasht. |
| `CollectionId` | `u64` | Mandanten-Katalog, fortlaufend | Heute wird sie per `blake3(name)` auf 64 Bit gekürzt (Kollisionsrisiko) [K]. Künftig vergeben, nicht berechnet. |
| `DocId` | `u128` | CSPRNG, zufällig | **Quelldokument und Löscheinheit.** Nicht erratbar, nicht sortierbar. |
| `ChunkId` | `(DocId, u32)` | Ordnungszahl im Dokument | **Retrievbare Einheit.** |
| `Slot` | `u32` | dicht, je Collection, intern | Verweist in HNSW, Postings, Caches, `LiveMask`. Nie öffentlich. |
| `TicketId` | `u128` | CSPRNG | Pseudonym eines Löschauftrags. |
| `SubjectRef` | Bytes, höchstens 256 | vom Aufrufer | Opaker Verweis auf eine betroffene Person (z. B. Kundennummer). Nie Klartext im persistierten Bestand: nur `blind(subject_ref)`. |
| `SubjectCommitment` | 32 Byte | `HMAC-SHA256(K_commit[tenant, epoch], kanonisch(scope))` | Pseudonym im `ErasedSet` und im Bericht. |
| `TxId` | `u64` | Store | Intern. |

**Entscheidung zur Granularität:** Die Löscheinheit ist das Quelldokument (`DocDek`). Die Index-Einheit ist der Chunk (`Slot`). Ein Löschaufruf für ein Dokument sperrt alle Slots seiner Chunks mit einer Maskenoperation.

### 5.3 Zeit und Zufall

- Alle Zeitstempel kommen vom Port `Clock` (UTC, Nanosekunden seit Unix-Epoche, dazu monotone Zeit). Kein direkter `SystemTime`-Aufruf im Kern.
- Schlüsselmaterial, Nonces und IDs stammen ausschließlich aus dem Betriebssystem-CSPRNG über `Rng`. Ein deterministischer `Rng` existiert nur im Testkit und kann im Release-Build nicht verdrahtet werden (Compile-Fehler ohne `cfg(test)` oder Feature `testing`, das in Release-Profilen verboten ist).

### 5.4 Fehlerkonzept

Ein Fehlertyp `ContextraError` mit stabiler Klasse und stabilem Code. Nachrichten enthalten **nie** Inhalte, Klartext-IDs, Schlüssel oder Pfade zu Geheimnissen.

| `ErrorKind` | Code-Bereich | Bedeutung | Wiederholbar |
|---|---|---|---|
| `InvalidInput` | CTX-1xxx | Ungültige Argumente, Limits überschritten | nein |
| `NotFound` | CTX-2xxx | Unbekannt **oder gelöscht** (kein Existenz-Orakel) | nein |
| `PolicyViolation` | CTX-3xxx | Mandantenverstoß, Egress verweigert, Bestätigung fehlt | nein |
| `Conflict` | CTX-4xxx | Gleichzeitige Änderung, Zustand passt nicht | ja, nach Neulesen |
| `Capacity` | CTX-5xxx | Speicher-, Größen- oder Mengenbudget | ggf. |
| `Io` | CTX-6xxx | Dateisystem, Speicher | ggf. |
| `Corruption` | CTX-7xxx | Prüfsumme, Authentifizierung, Formatfehler | nein, Reparaturpfad |
| `Crypto` | CTX-8xxx | Schlüssel fehlt, falsches Passwort, Suite unbekannt | nein |
| `Unsupported` | CTX-9xxx | Fähigkeit des Adapters fehlt | nein |
| `Timeout` | CTX-10xx | Wartezeit überschritten (Ticket läuft weiter) | ja |
| `Internal` | CTX-99xx | Invariante verletzt. Kein `panic`. | nein, melden |

Regeln:

1. **Kein Existenz-Orakel.** Für gelöschte und nie vorhandene Dokumente liefert die öffentliche API denselben Fehler `NotFound`. Intern und im Audit wird unterschieden.
2. **Fail-closed.** Bei unbekanntem oder nicht lesbarem Zustand wird abgelehnt, nicht geraten.
3. **Keine stillen Standardwerte.** Fehlkonfiguration (fehlender Embedder, fehlende Pflichtwerte) ist ein Fehler beim Start, kein Logeintrag.
4. Ein `Err` aus einer mutierenden Operation bedeutet: **es wurde nichts zugesagt**. Ein `Ok` bedeutet: die Zusage der jeweiligen Klasse gilt.

### 5.5 Geheimnisse

- Passphrasen und Schlüssel liegen in `SecretString` bzw. `SecretBytes` (`Zeroizing`). Diese Typen haben **kein** `Debug` mit Inhalt (stets `REDACTED`), **kein** `Serialize` und **kein** implizites `Clone`.
- Konfigurationsstrukturen, die Geheimnisse enthalten, sind nicht serialisierbar. Geheimnisse kommen über `KeySource`, nie über Umgebungsvariablen als Standardweg. Eine Umgebungsvariable ist nur als ausdrückliche `KeySource::FromEnv` erlaubt und wird nach dem Lesen aus der Prozessumgebung entfernt.
- Schlüsselmaterial wird nur in gesperrtem Speicher gehalten (`mlock`). Core-Dumps und Auslagerung des Prozesses werden unterbunden (`contextra_sys::harden_process`). Bleibende Remanenz in nicht adressierbarem RAM oder Swap bleibt ein **deklariertes Restrisiko**.

### 5.6 Idempotenz, Atomarität, Nebenläufigkeit

- Der Kern ist `Send + Sync`. Es gibt genau einen schreibenden Pfad im Speicher (Ein-Schreiber-Modell), Leser laufen parallel auf konsistenten Lesetransaktionen.
- `erase_*` ist **idempotent** nach Scope: ein zweiter Aufruf für dasselbe Ziel liefert dasselbe Ticket.
- `insert_many` ist atomar: alles oder nichts. Der Speicher ist die Wahrheit, die Indizes sind Ableitungen (Abschnitt 8.4).
- Jeder Schritt der Löschzustandsmaschine ist idempotent und wiederaufsetzbar.

### 5.7 Versionierung und Kompatibilität

- Öffentliche Crates folgen Semver. Persistierte Formate tragen eine eigene Formatversion.
- Ein geöffneter Bestand mit **höherer** Formatversion wird abgelehnt. Mit **niedrigerer** Version wird nur über das explizite Kommando `contextra migrate` gearbeitet, nie still beim Öffnen.
- Löschbezogene Formate (Ledger, Journal, Bericht) werden nie rückwirkend umgedeutet: neue Version, neue Prüfregel.

### 5.8 Beobachtbarkeit

- Logging und Tracing über `tracing`. Es ist verboten, Dokumentinhalte, Anfragetexte, Klartext-IDs, Schlüssel oder Passphrasen zu protokollieren. Ein Test prüft Log-Ausgaben gegen eine Canary-Zeichenfolge.
- Metriken über den Port `MetricsSink` (Standard: keine). Keine Metrik trägt Mandanten-, Dokument- oder Betroffenenbezug als Label. Zulässige Labels: Operation, Ergebnisklasse, Schicht.
- Gesundheitsbericht `HealthReport`: Zustand von Store, Vault, Journal, Worker, Anzahl offener Tickets, ältestes offenes Ticket, Indexzustände, Fähigkeiten der Adapter.

### 5.9 Konfigurationsschnittstelle

**Pflichtwerte ohne Standard** zwingen zu einer bewussten Entscheidung. Fehlt einer, scheitert `build()` mit `InvalidInput`.

| Parameter | Typ | Standard | Grenzen und Hinweise |
|---|---|---|---|
| `data_dir` | Pfad | **Pflicht** | Netzlaufwerke und FUSE nicht unterstützt (dokumentiert). |
| `key_source` | `KeySource` | **Pflicht** | `Passphrase(SecretString)`, `RawKey(SecretBytes32)`, `External(Arc<dyn KeyVault>)`. **Es gibt keinen Klartext-Modus.** |
| `kdf` | `KdfParams` | Argon2id, 64 MiB, t=3, p=4 [?] | Untergrenze 64 MiB und t=3 für neue Bestände. Parameter liegen im `KdfHeader`. Vor Release gegen aktuelle Empfehlungen (RFC 9106, BSI) prüfen. |
| `tenant_mode` | `TenantMode` | `Required` | `Single` legt genau einen Mandanten „default“ an, ist eine ausdrückliche Wahl. |
| `erasure.erase_sla` | `Duration` | **Pflicht** | Zeit bis Klasse C. Überschreitung setzt das Ticket auf `Overdue` und löst einen Alarm aus. Empfohlen kleiner als ein Monat (Art. 12 Abs. 3). |
| `erasure.erased_set_retention` | `Duration` | **Pflicht** | Lebensdauer von `ErasedSet` und `K_commit`. |
| `erasure.backup_max_age` | `Duration` | **Pflicht** | Muss `≤ erased_set_retention` sein, sonst Fehler. |
| `erasure.epoch_window` | `Duration` | 5 s | Takt-2-Zeitfenster. |
| `erasure.epoch_max` | `Duration` | 60 s | Obergrenze bis zum Start von Takt 2. |
| `erasure.epoch_batch` | `usize` | 1.024 | Mengenschwelle je Epoche. |
| `vector.metric` | `DistanceMetric` | `Cosine` | Je Collection fest. |
| `vector.hnsw` | `m`, `ef_construction`, `ef_search` | 16, 200, 64 [S] | Vor Release per Recall-Messung bestätigen. |
| `vector.exact_scan_below` | `usize` | 50.000 [S] | Unterhalb: flacher Exakt-Scan (trivial korrekt, trivial löschbar). |
| `vector.tombstone_mode` | `Strict` oder `Waypoint` | `Strict` | `Waypoint` nur nach Recall-Messung freigegeben (nicht Teil von V1.0). |
| `vector.rebuild_ratio` | `f32` | 0,10 | Anteil toter Knoten, ab dem der Shadow-Rebuild startet. |
| `text.language` | `Language` | `German` | Komposita-Zerlegung aktiv. Wortlisten Recht und Medizin zuschaltbar, Lizenz der Listen dokumentiert. |
| `ingest.isolation` | `Subprocess` oder `InProcess` | `Subprocess` | `InProcess` nur für vertrauenswürdige Eingaben, wird im Bericht ausgewiesen. |
| `ingest.limits` | `IngestLimits` | siehe Abschnitt 11 | Hartes Byte-Limit auf gelesene Daten, nicht auf deklarierte Größen. |
| `pii.policy` | `PiiPolicy` | `Report` | `Off`, `Report` (Zählung in den Warnungen, Text unverändert), `Redact` (Platzhalter), `Reject` (Dokument ablehnen). |
| `egress.policy` | `EgressPolicy` | `LoopbackOnly` | Jeder Adapter mit Netzzugriff deklariert seine Ziele. Alles außerhalb der Policy wird fail-closed verweigert. |
| `audit.enabled` | `bool` | `true` | Inhaltsfreie Ereignisse (Löschung, Schlüsselereignisse, Start, Restore). |
| `audit.access_log` | `bool` | `false` | Lesezugriffe werden nur auf ausdrückliche Anforderung protokolliert, ohne Inhalte. |
| `collection.retention` | `Option<Duration>` | `None` | Löschfrist je Collection. Fehlt sie, meldet der Compliance-Bericht „keine Löschfrist definiert“ als Befund. |
| `process.lock_key_memory` | `bool` | `true` | `mlock` für Schlüsselmaterial. |
| `process.disable_core_dumps` | `bool` | `true` | |
| `limits.max_open_tickets` | `usize` | 100.000 | Schutz vor Überlauf des Journals. |

**Nicht vorhanden, bewusst:** `encryption_mode = None`, `auto_extraction = Enabled`, `tenant_policy = Optional` als Standard, Lizenzgate, Telemetrie, Update-Prüfung.

---

## 6. Schlüssel- und Kryptoschnittstelle

### 6.1 Hierarchie

```
Passphrase ─Argon2id(KdfHeader)─▶ KEK_root           (nur im RAM, mlock)
KEK_root   ─wrap─▶ K_tenant      (zufällig, 32 B, je Mandant)
K_tenant   ─wrap─▶ K_coll        (zufällig, 32 B, je Collection)
K_coll     ─wrap─▶ DocDek        (zufällig, 32 B, je Quelldokument)   ◀── Löscheinheit
```

Alle Zwischenschlüssel sind **zufällig** und liegen nur gewrappt vor. Sie werden nicht aus dem Elternschlüssel abgeleitet, weil Abgeleitetes jederzeit neu berechenbar ist und sich deshalb nicht vernichten lässt.

### 6.2 Schlüsselarten

| Schlüssel | Größe | Herkunft | Ablage | Vernichtung | Zweck |
|---|---|---|---|---|---|
| `KEK_root` | 32 B | Argon2id oder `RawKey` | nur RAM, `mlock` | Ende des Prozesses, `zeroize` | Wrappt `K_tenant`, `K_commit`, Signaturschlüssel |
| `K_tenant` | 32 B | CSPRNG | gewrappt in der Mandantendatei | Überschreiben (`erase_tenant`) | Wrappt `K_coll` |
| `K_coll` | 32 B | CSPRNG | gewrappt in der Collection-Datei | Überschreiben (`erase_collection`) | Wrappt `DocDek`, versiegelt Katalog und Postings |
| `DocDek` | 32 B | CSPRNG | gewrappt im **Ledger** | Überschreiben im Ledger (`erase_document`) | Versiegelt alle Chunks, Vektoren, Blobs, Metadaten und Rebuild-Hilfsdaten eines Dokuments |
| `K_idx` | 32 B | HKDF aus `K_coll`, Label `ctx/idx/v1` | nicht gespeichert | mit `K_coll` | Blinde Terme: `blind(term) = HMAC(K_idx, term)` gekürzt auf 16 B |
| `K_commit[t,e]` | 32 B | CSPRNG, je Mandant und Epoche | gewrappt in der Commit-Datei | nach Ablauf der Aufbewahrung des Berichts | `SubjectCommitment` |
| `K_journal` | 32 B | HKDF aus `KEK_root`, Label `ctx/journal/v1` | nicht gespeichert | nicht vernichtbar (kein Personenbezug) | Versiegelt Journal-Einträge |
| `K_sign` | Ed25519 | CSPRNG | versiegelt unter `KEK_root` | Rotation | Signiert Löschberichte. Der öffentliche Schlüssel wird exportiert. |

Ableitung ist nur dort erlaubt, wo der Schlüssel nicht gelöscht, sondern mit seinem Elternschlüssel verworfen wird (`K_idx`) oder nicht personenbezogen ist (`K_journal`).

### 6.3 Umschlag `Sealed` (Format V1)

```
offset  größe  feld
0       1      format_version      = 0x01
1       1      suite_id            0x01 = AES-256-GCM-SIV, 0x02 = XChaCha20-Poly1305 (reserviert)
2       2      reserved            = 0
4       12     nonce               (zufällig, 96 Bit; bei Suite 0x02: 24 Byte, dann verschiebt sich der Rest)
16      n      ciphertext
16+n    16     tag
```

- Jede Nonce wird **zufällig** gezogen. Das ist unbedenklich, weil je Dokument ein eigener Schlüssel besteht und die Zahl der Nachrichten je Schlüssel klein bleibt. Die heutige Mischung aus 4-Byte-Präfix und Zähler entfällt [K]. AES-256-GCM-SIV ist zusätzlich nonce-missbrauchsresistent.
- **AAD** (zugeordnete Daten) = `"ctx1" ‖ tenant_id ‖ collection_id ‖ kind ‖ slot_oder_doc_ref`. Ein Umschlag lässt sich dadurch nicht an eine andere Stelle kopieren.
- `kind` ∈ {`chunk`, `vector`, `meta`, `blob`, `catalog`, `postings`, `journal`, `ledger_wrap`}.
- Unbekannte `suite_id` → `Crypto`-Fehler, kein Rateversuch.

### 6.4 `KdfHeader` (Format V1)

```
0   1   format_version = 0x01
1   1   kdf_id         0x01 = Argon2id
2   1   argon2_version 0x13
3   1   reserved
4   4   m_cost_kib     (u32 LE)
8   4   t_cost         (u32 LE)
12  4   p_cost         (u32 LE)
16  16  salt           (zufällig)
32  4   header_crc32c
```

Beim Öffnen werden Parameter unterhalb der Untergrenze abgelehnt. Eine Passphrasenänderung (`change_passphrase`) schreibt neuen Salt und neue Parameter und wrappt nur `K_tenant` neu. Es werden **keine Daten** neu verschlüsselt.

### 6.5 Ledger (Format V1)

Das Ledger ist das einzige Objekt, das beim Löschen **überschrieben** wird. Es liegt pro Collection in Bucket-Dateien.

| Eigenschaft | Festlegung |
|---|---|
| Slotgröße | **64 Byte**: `key_gen u32 (4) ‖ nonce (12) ‖ wrapped_dek (32 + 16 Tag = 48)` = 64 |
| Lebend / frei | Ein Slot aus lauter Nullen ist frei oder vernichtet. `key_gen` beginnt bei 1, ein lebender Slot ist nie komplett null. |
| Bucket | 4.096 Slots = 256 KiB, Datei `ledger-{collection_id}-{bucket_no}.ctxl` |
| Bucket-Kopf | 64 Byte: `magic "CTXL"`, `format_version`, `collection_id`, `bucket_no`, `generation u32`, `crc32c` |
| AAD des Wraps | `tenant_id ‖ collection_id ‖ bucket_no ‖ slot_index ‖ key_gen` (Position gebunden) |
| Zugriff | `pread`/`pwrite` auf feste Offsets. Entpacken eines `DocDek` = ein AEAD-Aufruf. |
| Cache | LRU entpackter `DocDek` in `Zeroizing`. `mlock` nur für `KEK_root`, `K_tenant`, `K_coll`, nicht für Millionen `DocDek`. |
| Vernichtung (Takt 2) | Slot mit Nullen überschreiben, ein `fdatasync` je betroffener Bucket-Datei für alle Löschungen der Epoche. |
| Epochen-Rewrite | Betroffene Buckets in eine neue Datei ohne die vernichteten Slots schreiben, `fsync`, `rename`, `fsync` des Verzeichnisses, alte Datei entfernen, Verwerfungshinweis (`FITRIM`/Discard) an das Dateisystem [?]. |
| Append-only-Schlüsselprotokoll | **Verboten.** Ein solches Protokoll wäre eine verbleibende Schlüsselkopie. |

### 6.6 `KeyVault`: Fähigkeitsstufen

| `VaultCaps` | Bedeutung | Eintrag im Bericht |
|---|---|---|
| `Effaceable` | Wurzelschlüssel oder `K_tenant` liegt in TPM, Secure Element oder löschbarem OS-Keystore. Vernichtung dort ist echt. | Klasse C ohne `FlashRemanence` |
| `FileOverwrite` (Standard im Kern) | Überschreiben, Epochen-Rewrite, Verwerfungshinweis. | Klasse C mit `ResidualRisk::FlashRemanence` |
| `MemoryOnly` | Nur Tests. | Im Release-Build nicht zulässig |

Gegen einen Angreifer **ohne** Passphrase (gestohlenes Gerät, abgegriffenes Backup) wirkt die Verschlüsselung unabhängig davon. Ein Betreiber, der die Passphrase kennt, ist durch Flash-Remanenz nicht ausgeschlossen. Diese Aussage gehört in den Bericht, in die Dokumentation und in die DSFA-Hinweise. Eine `Effaceable`-Anbindung (TPM, HSM, Cloud-Schlüsseldienst mit eigener Wartefrist `destruction_delay`) ist ein natürlicher Enterprise-Adapter hinter demselben Port.

### 6.7 Algorithmenwechsel (Krypto-Agilität)

- Jeder Umschlag trägt `suite_id`, jeder Header eine Version. Das Kommando `contextra migrate-suite` versiegelt Dokumente unter der neuen Suite neu, dokumentweise, wiederaufsetzbar, mit eigenem Journal.
- Pflichtartefakt: ein **Notfallplan** (Textvorlage im Repository) mit Auslöser, Verantwortlichem, Ablauf, Frist. Der Compliance-Bericht prüft, dass die Parameter („Algorithmus, Datum der letzten Neubewertung“) gesetzt sind.
- Periodische Neubewertung: Das Produkt erinnert über `HealthReport` an die Neubewertung nach einer konfigurierbaren Frist (Standard 12 Monate).

### 6.8 Kryptografische Bausteine (nur Standardbibliotheken)

`aes-gcm-siv`, `chacha20poly1305`, `argon2`, `hkdf`, `hmac`, `sha2`, `blake3` (Hash-Kette, Katalogprüfsummen), `ed25519-dalek`, `subtle` (konstante Zeit), `zeroize`, `getrandom` [?]. **Keine Eigenkonstruktion.** Versionen und bekannte Sicherheitshinweise werden über `cargo audit`, `cargo deny`, `cargo vet` als Pflicht-Gate geprüft.

---

## 7. Löschschnittstelle („Erasure by Design“)

Die Löschkette ist die zentrale Schnittstelle des Produkts und die Grundlage jeder Rechtsaussage.

### 7.1 Leitprinzipien

| # | Prinzip | Konsequenz |
|---|---|---|
| P1 | **Eine Löscheinheit = ein Quelldokument = ein Schlüssel.** Chunks, Vektoren, Blobs, Metadaten und Hilfsdaten laufen unter demselben zufälligen `DocDek`. | Schlüsselvernichtung wirkt auf alles Persistente eines Dokuments zugleich. |
| P2 | **Eine Identität in allen Indizes: der `Slot`** (dicht, `u32`, je Collection, je Chunk). | Ein Bit sperrt den Chunk in allen Indizes. |
| P3 | **Die `LiveMask` ist die Wahrheit im Lesepfad.** Jede Suche und jeder Abruf prüft sie (O(1)). | Löschen wirkt sofort, ohne Indexumbau. |
| P4 | **Indizes sind Ableitungen.** Persistierte Index-Stände enthalten nichts, was ohne `DocDek` lesbar ist, und müssen bei einer Löschung nie neu geschrieben werden. | Kein Rewrite großer Dateien je Löschung. |
| P5 | **Aufwand wird gebündelt (Epochen).** Teure Schritte laufen für viele Löschungen zusammen. | Amortisierte Kosten je Löschung nahe konstant. |
| P6 | **Zwei Takte.** Takt 1 ist kurz, synchron und durabel. Takt 2 ist asynchron und gebündelt. | Rechtlich und praktisch sofort wirksam sind Sperre (L) und Schlüsselvernichtung (C). Physische Bereinigung der Klartext-Strukturen (P) folgt in Sekunden bis Minuten. |
| P7 | **Scope ist ein Typ, kein String.** | Kein Präfix-Matching auf Zeichenketten. Behebt die Fehlerklasse, bei der heute ein Löschauftrag für ein Dokument alle Daten des Mandanten erfasst [K, Q]. |

### 7.2 Klassen der Löschwirkung

Das System gibt nur die Klasse aus, die messtechnisch belegt ist. Absolute Aussagen („100 % gelöscht“) sind unzulässig.

| Klasse | Name | Aussage | Zeuge (Konstruktion, O(1) bis O(d) je Chunk) |
|---|---|---|---|
| **L** | Logisch gesperrt | Das Dokument ist für alle Lese- und Suchpfade gesperrt. | Journal-Eintrag durabel; `LiveMask`-Bit gesetzt; Lesepfad-Probe liefert „gesperrt“ |
| **P** | Klartext bereinigt | In allen aktiven Indizes und Caches (Ebene A) existiert keine Klartextkopie mehr. | HNSW-Slot nullbytig (fp32 und SQ8); Text-Maske gesetzt; Cache-Register leer; Stichproben-Canary |
| **C** | Persistente Reste unlesbar | Alle persistenten Kopien liegen nur als Chiffrat unter vernichtetem `DocDek`. | `KeyVault::present == false`; Ledger-Slot nullbytig; `open` auf ein echtes Chiffrat des Dokuments liefert `KeyAbsent` |
| **E** | Technisch vollständig | L, P und C sind nachgewiesen, und alle Pflichtschichten des `LayerCatalog` sind gemessen oder als außerhalb des Geltungsbereichs deklariert. | L ∧ P ∧ C ∧ Katalog vollständig |

Klasse E ist eine **technische Aussage innerhalb deklarierter Schranken**, keine rechtliche Feststellung einer Löschung nach Art. 17.

### 7.3 Typen (Entwurf, nicht compilergeprüft)

```rust
// contextra-types (rein, ohne I/O)
#[non_exhaustive]
pub enum ErasureTarget {
    Document(DocId),
    Collection(CollectionId),
    Tenant,
    Subject(SubjectRef),          // gesamter Mandant, über Collections hinweg
}
pub struct ErasureScope { pub tenant: TenantId, pub target: ErasureTarget }

bitflags::bitflags! {             // [?] Crate prüfen
    pub struct ClassSet: u8 { const L = 1; const P = 2; const C = 4; const E = 8; }
}

#[non_exhaustive]
pub enum TicketState {
    Requested, Frozen, PlaintextPurged, PlaintextMeasured,
    KeyDestroyed, KeyMeasured, Completed, Restricted,
}
// Overdue ist kein Zustand, sondern ein Flag (Frist überschritten) am Ticket.

/// Total und ohne I/O. Ungültige Kombinationen ergeben TransitionError.
pub fn step(s: TicketState, e: TicketEvent) -> Result<TicketState, TransitionError>;

#[non_exhaustive]
pub enum ResidualRisk {
    FlashRemanence,
    BackupKeyCopies { until: Timestamp },
    SwapOrDumpPossible,
    ExternalCopies,                // LLM-Anbieter, Exporte, Kopien außerhalb der Anwendung
    EscrowPresent,
    CryptoObsolescence { reviewed_at: Timestamp },
    VaultDestructionDelay { days: u16 },
    UnmeasuredLayer(LayerId),
    CommitmentPseudonymous { until: Timestamp },
    MetadataLeakage,               // Längen, Zählwerte
}
```

### 7.4 Zustandsmaschine

```
Requested → Frozen (L) → PlaintextPurged → PlaintextMeasured (P)
                       → KeyDestroyed → KeyMeasured (C) → Completed (E, falls vollständig)
Frozen → Restricted (Aufbewahrungspflicht) → Release → Requested
```

| Übergang | Bedingung | Wirkung |
|---|---|---|
| `Requested → Frozen` | Journal-Eintrag durabel, `LiveMask`-Bits gesetzt | Klasse L, Lesesperre aktiv |
| `Frozen → PlaintextPurged` | Alle `LayerPurger` haben `purge_slots` gemeldet | Ebene A bereinigt |
| `PlaintextPurged → PlaintextMeasured` | Messung A (O(1) je Slot) bestanden | Klasse P |
| `PlaintextMeasured → KeyDestroyed` | **Nur nach Messung A.** Ledger-Slots überschrieben, `fdatasync` je Bucket | Ebene B entwertet |
| `KeyDestroyed → KeyMeasured` | Schlüssel fehlt; Lesepfad-Probe liefert `KeyAbsent` | Klasse C |
| `KeyMeasured → Completed` | Epochen-Rewrite der Buckets durchgeführt, Bericht signiert | Abschluss, ggf. E |
| `Frozen → Restricted` | `legal_hold` aktiv oder `RetentionPolicy` verlangt Aufbewahrung | Keine Vernichtung, kein Purge. Dokument aus dem Retrieval isoliert. „Gelöscht“ ist in diesem Zustand verboten. |
| `Restricted → Requested` | Frist abgelaufen oder Sperre aufgehoben | Reguläre Löschung läuft an |

**Begründung der Reihenfolge:** `KeyDestroyed` folgt auf die Messung A, weil Ebene-A-Arbeit (z. B. Rebuild-Hilfsdaten) unter Umständen noch entschlüsseln muss.

Jeder Schritt ist **idempotent**. Fehler führen zu Wiederholung mit Backoff (`attempt`, `next_retry_at` im Journal), nicht zu einem Abbruch. Es gibt keinen „Fehlgeschlagen“-Endzustand: Ein Ticket, das die Frist `erase_sla` überschreitet, bekommt das Flag `Overdue` und löst einen Alarm aus.

### 7.5 Takt 1: synchron, kurz, durabel

`Eraser::erase(scope) -> Result<ErasureTicket>`

| Schritt | Verhalten |
|---|---|
| 1 | Scope gegen den Mandanten prüfen (Typ, nicht Zeichenkette). |
| 2 | Katalog: `DocId → [Slot]`, `DocId → KeyRef` (Ledger-Position). O(1) je Dokument. |
| 3 | Ist `legal_hold` aktiv: Eintrag `Restricted` ins Journal, Retrieval-Maske setzen, Ticket `Restricted` zurückgeben. Ende. |
| 4 | `Frozen` ins Journal (Group-Commit: gleichzeitige Löschungen teilen sich einen `fdatasync`, Wartefenster 1 bis 2 ms [S]). Erst nach durablem Flush weiter. |
| 5 | `LiveMask`-Bits setzen und veröffentlichen (lock-frei, `Arc`-Austausch oder atomare Wörter). |
| 6 | Ticket zurückgeben, Klasse L. |

Zusagen:

- **Schlägt der Journal-Flush fehl, wird die Lesesperre nicht gesetzt**, die Funktion liefert `Err`, der Aufrufer weiß, dass nichts zugesagt ist.
- Nach Rückgabe liefert jeder Lesepfad für das Dokument `NotFound` (fail-closed, kein Existenz-Orakel).
- Zielwert: p99 ≤ 5 ms, unabhängig von der Datenmenge [S]. Durchsatz ≥ 5.000 Löschungen/s im Group-Commit [S].
- Offene Lese-Snapshots blockieren die Löschung **nicht**. Das Lesegate gilt auch für Snapshot-Leser (Prüfung beim Materialisieren), und ohne `DocDek` ist ein altes Chiffrat ohnehin nicht zu öffnen.

### 7.6 Takt 2: Epochen, gebündelt

Ein Worker sammelt Tickets, bis das Zeitfenster (`epoch_window`, höchstens `epoch_max`) oder die Mengenschwelle (`epoch_batch`) erreicht ist. Dann in dieser Reihenfolge:

1. **Ebene A (Klartext):** HNSW-Slots nullen, Tombstones setzen. Text-Maske ist bereits gesetzt. Caches über das `Purgeable`-Register invalidieren. RAM-Metadatenindex bereinigen.
2. **Messung A** je Dokument (O(1) bis O(d)).
3. **Schlüsselvernichtung:** Ledger-Slots überschreiben, ein `fdatasync` je betroffener Bucket-Datei, `DocDek` aus dem Cache entfernen und `zeroize`.
4. **Messung C:** `present() == false`, Lesepfad-Probe.
5. **Epochen-Rewrite** der betroffenen Ledger-Buckets, Verwerfungshinweis, `Completed` ins Journal, Bericht signieren.

**Collection- und Mandanten-Löschung sind in der Kryptografie O(1):** Es wird `K_coll` bzw. `K_tenant` vernichtet. Damit ist jeder `DocDek` darunter nicht mehr entwrapbar, ohne ein Dokument anzufassen. Takt 2 gibt danach die RAM-Indizes frei (Arena-Speicher überschreiben).

### 7.7 Ebene A je Schicht

| Schicht | Repräsentation | Löschmechanismus | Messung |
|---|---|---|---|
| Chunk-Text, Blob, Metadaten (persistent) | `Sealed` unter `DocDek` im `KvStore` | Schlüsselvernichtung | Schlüsselabwesenheit, Lesepfad-Probe |
| Vektor persistent | `Sealed` unter `DocDek` | Schlüsselvernichtung | wie oben |
| Vektor im RAM (HNSW-Arena) | Klartext, Slot-adressiert | Slot nullen (fp32 und SQ8), Tombstone | Slot nullbytig, Stichproben-Canary |
| HNSW-Graph (Nachbarlisten) | Slot-IDs, kein Inhalt | bleibt bis zum Shadow-Rebuild. **Keine Umverdrahtung** pro Löschung. Modus `Strict`: toter Knoten wird in der Navigation übersprungen. | Tombstone-Anteil |
| Volltext-Postings | Segmente `blind(term) → [(slot, tf)]` | Eintrag in `LiveMask`. Posting-Zeilen bleiben bis zum Segment-Merge. | Maske, Segment-Audit |
| Volltext-Wörterbuch | `blind(term)`, kein Klartextwort | Merge entfernt Terme mit `df == 0` | Merge-Bericht |
| Forward-Terme (nur für Rebuild) | `Sealed` unter `DocDek` | Schlüsselvernichtung | Schlüsselabwesenheit |
| Katalog `DocId ↔ Slot ↔ KeyRef` | `Sealed` unter `K_coll` | Eintrag entfernen (Epoche), mit `K_coll` bei Collection-Löschung | Katalogprüfung |
| Subject-Index | `blind(subject_ref) → [(Collection, DocId)]`, `Sealed` unter `K_tenant` | Eintrag entfernen | Index-Audit |
| Caches (Embedding, Abfrage, Chunk) | `Purgeable`-Register, Slot-adressiert | Invalidierung | Register-Vollständigkeitstest |
| Protokolle, Audit | nur IDs, Commitments, Zähler | nicht inhaltlich, nichts zu löschen | Log-Scan auf Canary |
| Backups, Swap, Dumps, externe Kopien | außerhalb | nicht adressierbar, **deklariert** | – |

Der Textindex speichert **blinde Terme** (`HMAC(K_idx, term)`, 16 Byte), damit auch die persistierte Struktur kein lesbares Wort enthält. Kosten: ein HMAC je Token beim Indexieren und Abfragen (Größenordnung 100 bis 300 ns [S]).

**HNSW-Slot-Wiederverwendung** erst nach Rebuild: Kein Verweis darf auf einen neuen Besitzer zeigen.

**Shadow-Rebuild:** Ein Hintergrund-Thread baut aus den lebenden Slots einen neuen Index, Schreibvorgänge landen währenddessen in einem Delta-Log, am Ende wird per `Arc`-Austausch umgeschaltet. Es gibt keinen langen Schreib-Mutex. Der Rebuild entfernt zugleich Kanten zu toten Slots und gibt Slots frei.

### 7.8 Lesegate und Wiederanlauf

**Lesegate (Pflicht in jedem Lesepfad):**

1. `LiveMask[slot]` prüfen (schnell, für laufende Instanzen).
2. Katalog auflösen.
3. Erst dann `DocDek` entwrappen und `open`.

Zusätzlich fragt der Lesepfad den persistenten `ErasedSet` (Menge der `SubjectCommitment`s gelöschter Dokumente), der für Restore und Wiederanlauf gebraucht wird.

**Wiederanlauf (`recover()`) vor Freigabe des Lesepfads:** Journal lesen, `LiveMask` und `ErasedSet` füllen, offene Tickets in Takt 2 einreihen. Solange `recover()` nicht abgeschlossen ist, liefern alle Lesepfade `Conflict` (nicht Daten).

| Fall | Verhalten |
|---|---|
| Absturz nach Takt 1 | `recover()` stellt Sperre her, setzt Takt 2 fort. |
| Schritt schlägt fehl (I/O) | Zustand unverändert, Backoff. Kein Fehlschlag-Endzustand. |
| Absturz im Epochen-Rewrite | Neue Bucket-Datei gilt erst nach `rename`. Die alte bleibt bis dahin vollständig. Wiederholbar. |
| Journal nicht durabel | Takt 1 liefert `Err`, keine Sperre. |
| `erase_sla` überschritten | Flag `Overdue`, Alarm. |

### 7.9 Betroffenenrechte und Aufbewahrung

| Recht | Schnittstelle |
|---|---|
| Art. 15 Auskunft | `subject_documents(subject_ref)` liefert Dokumentverweise und Metadaten, `get` liefert Inhalte. |
| Art. 16 Berichtigung | `replace_document(id, neue Fassung)`: Neu einfügen, Altes löschen (Ticket). |
| Art. 17 Löschung | `erase_document`, `erase_subject`, `erase_collection`, `erase_tenant`. |
| Art. 18 Einschränkung | `restrict_document`, `release_restriction`. Dokument ist nicht mehr durchsuchbar und nicht abrufbar, bleibt verschlüsselt gespeichert. |
| Art. 20 Übertragbarkeit | `export_subject(subject_ref)` liefert strukturierte Daten (JSON). |
| Art. 17 Abs. 2 / Art. 19 | Informationspflicht gegenüber Empfängern ist organisatorisch. Der Löschbericht weist `ExternalCopies` aus. |

**Aufbewahrungspflicht:** `legal_hold` am Dokument oder die Entscheidung der `RetentionPolicy` führt zu `Restricted` (Einschränkung, Art. 18) statt zu Vernichtung. Das aufbewahrungspflichtige Original (Archiv) und die RAG-Repräsentationen (Chunk, Vektor, Index) sind getrennte Verarbeitungen mit eigener Grundlage. Eine Aufbewahrungspflicht rechtfertigt nicht automatisch die Weiternutzung im aktiven Retrieval.

**Betroffener:** `SubjectRef` ist ein opaker, vom Aufrufer vergebener Verweis. Der Kern speichert nur `blind(subject_ref)`. `erase_subject` plant alle Dokumente der Person als **ein** Ticketbündel, ein Journal-Eintrag je Dokument im selben Group-Commit.

### 7.10 Backup, Restore, Restore-Guard

- **Daten und Ledger werden getrennt gesichert.** Das Backup-Manifest führt beide getrennt, mit Erstellzeit, Ledger-Generation und Wasserstand des `ErasedSet`. Ein Ledger-Backup enthält gewrappte `DocDek` gelöschter Dokumente und ist damit eine **Schlüsselkopie** (`ResidualRisk::BackupKeyCopies { until }`). Empfehlung: Ledger-Backups mit kürzerer Aufbewahrung als Datenbackups, verschlüsselt mit separatem Schlüssel.
- **Restore-Guard:** `Contextra::restore` liest vor dem ersten Lesezugriff den `ErasedSet` des Zielsystems, sperrt alle gelöschten Dokumente sofort nach, vernichtet wiederhergestellte `DocDek` erneut und wiederholt die Löschzustandsmaschine.
- **Restore-Horizont:** Ein Backup, das älter ist als `erased_set_retention`, wird mit `RestoreBeyondHorizon` abgelehnt, weil der Guard seine Bereinigung nicht mehr garantieren kann. Regel: `backup_max_age ≤ erased_set_retention`.
- `K_commit` hat eine Lebensdauer von mindestens `erased_set_retention`. Bei Epochenwechsel werden alle Epochen im Restore-Horizont geprüft.
- Das Commitment ist solange pseudonym und damit personenbezogen, wie `K_commit` existiert und dem Verantwortlichen die Betroffenen-ID bekannt ist. Es braucht Grundlage, Zweck und Frist. Nach Ablauf wird `K_commit` vernichtet; die Hash-Kette der Audit-Protokolle bleibt verifizierbar.

### 7.11 Der Löschbericht `ErasureReceipt`

Er ersetzt den bisherigen „Löschbeweis“. Der Name „Beweis“ ist im Quelltext, in Dokumenten und in Berichten **verboten** (Wording-Lint).

```rust
pub struct ErasureReceipt {
    pub receipt_version: u16,
    pub ticket: TicketId,                 // Pseudonym, kein Klartext-Bezug
    pub scope_kind: ScopeKind,            // Document | Collection | Tenant | Subject
    pub subject_commitment: [u8; 32],     // HMAC-SHA256, siehe 5.2
    pub commitment_epoch: u32,
    pub class: ClassSet,                  // L | P | C | E
    pub layers: Vec<LayerResult>,         // Schicht, Technik, gemessen/deklariert, Restmenge
    pub residual: Vec<ResidualRisk>,
    pub vault: VaultCaps,
    pub ledger_generation: u32,
    pub suite: SuiteInfo,                 // AEAD, KDF, Parameter
    pub requested_at: Timestamp,          // über Clock-Port
    pub completed_at: Timestamp,
    pub issuer_key_id: [u8; 16],
    pub signature: Ed25519Signature,      // nur diese Ebene ist extern prüfbar
}
```

- Der Bericht enthält **kein** Feld mit Inhalt, Klartext-Kennung, Chunk-Hash oder Dateinamen.
- Signiert wird die kanonische JSON-Darstellung (RFC 8785 [?]) mit dem Präfix `contextra-receipt-v1\n`. Verifiziert wird mit dem öffentlichen Schlüssel, den `contextra-verify` ohne Zugriff auf Daten oder Geheimnisse prüft.
- Der frühere nachträglich gesetzte, nicht signierte Zusatzvermerk (`integrity_warning`) entfällt. Es gibt **keine** Felder außerhalb der Signatur.
- Es gibt **keine** symmetrisch (HMAC) signierte Berichtsebene. Ein Prüfer mit dem geteilten Schlüssel könnte fälschen. Nur Ed25519 ist extern prüfbar.
- Ein qualifizierter Zeitstempel (eIDAS) ist über den optionalen Port `TimestampAuthority` möglich und gehört in den Enterprise-Adapter.

### 7.12 Pflichtschichten (`LayerCatalog`)

Eine Konstanten-Tabelle mit Exhaustivitätstest. Fehlt eine Schicht in der Messung, sinkt die Klasse automatisch (nie E).

| Schicht | Im Geltungsbereich | Messung |
|---|---|---|
| Chunk-Speicher | ja | Schlüsselabwesenheit |
| Vektor persistent | ja | Schlüsselabwesenheit |
| Vektor RAM | ja | Slot nullbytig |
| Text-Postings | ja | Maske |
| Text-Wörterbuch | ja | Merge-Bericht |
| Forward-Terme | ja | Schlüsselabwesenheit |
| Katalog, Subject-Index | ja | Katalogprüfung |
| RAM-Metadatenindex | ja | Register leer |
| Caches | ja | Register leer |
| Ledger | ja | Slot nullbytig |
| Journal, Audit | ja | inhaltsfrei, Canary-Scan |
| Backups, Swap, Dumps, Exporte, Kontexte bei externen LLM-Anbietern | **nein, deklariert** | `ExternalCopies`, `SwapOrDumpPossible`, `BackupKeyCopies` |

### 7.13 Zielwerte [S]

| Kennzahl | Zielwert |
|---|---|
| Takt 1, p50 / p99 | ≤ 1 ms / ≤ 5 ms, unabhängig von N |
| Durchsatz Takt 1 | ≥ 5.000 Löschungen/s |
| Takt 2 bis Klasse C | ≤ 60 s bei Standardfenster |
| Leselatenz bei Massenlöschung | Regression ≤ 5 % |
| Recall@10 nach Löschen von 10 % | ≥ Ausgangswert minus 1 Punkt |
| Ingest-Overhead durch Schlüssel je Dokument | ≤ 5 % |
| Ledger-Overhead | 64 Byte je Dokument |

Alle Werte sind Vorschläge für Gates. Erst Messungen entscheiden, welche bindend werden.

---

## 8. Öffentliche Rust-API (Fassade `contextra`)

Alle Signaturen sind Entwurf. Die Fassade enthält höchstens 20 öffentliche Funktionen je Handle-Typ und hängt nicht direkt von Ring 0/1-Crates ab, ausgenommen die Typen.

### 8.1 Builder und Instanz

```rust
pub struct ContextraBuilder { /* privat */ }

impl ContextraBuilder {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self;                       // Pflicht
    pub fn key_source(self, k: KeySource) -> Self;                          // Pflicht
    pub fn kdf(self, p: KdfParams) -> Self;
    pub fn tenant_mode(self, m: TenantMode) -> Self;                        // Standard: Required
    pub fn erasure(self, c: ErasureConfig) -> Self;                         // erase_sla, retention, backup_max_age: Pflicht
    pub fn embedder(self, e: Arc<dyn EmbeddingProvider>) -> Self;           // Pflicht für Text-Einfügen und -Suche
    pub fn ingest(self, c: IngestConfig) -> Self;
    pub fn pii(self, p: PiiPolicy) -> Self;
    pub fn egress(self, p: EgressPolicy) -> Self;
    // Erweiterungspunkte (Enterprise):
    pub fn key_vault(self, v: Arc<dyn KeyVault>) -> Self;
    pub fn authorization(self, a: Arc<dyn AuthorizationHook>) -> Self;
    pub fn audit_sink(self, s: Arc<dyn AuditSink>) -> Self;
    pub fn proof_signer(self, s: Arc<dyn ProofSigner>) -> Self;
    pub fn retention_policy(self, p: Arc<dyn RetentionPolicy>) -> Self;
    pub fn timestamp_authority(self, t: Arc<dyn TimestampAuthority>) -> Self;
    pub fn metrics(self, m: Arc<dyn MetricsSink>) -> Self;
    pub fn build(self) -> Result<Contextra>;                                // validiert, recover(), dann Lesepfad frei
}

pub enum KeySource {
    Passphrase(SecretString),
    RawKey(SecretBytes32),
    FromEnv { var: String },       // liest und entfernt die Variable
    External,                      // setzt key_vault(...) voraus
}
```

```rust
#[derive(Clone)]                    // billig (Arc)
pub struct Contextra { /* privat */ }

impl Contextra {
    pub fn create_tenant(&self, name: &str) -> Result<TenantId>;            // Admin
    pub fn tenant(&self, id: TenantId) -> Result<TenantHandle>;
    pub fn list_tenants(&self) -> Result<Vec<TenantInfo>>;                  // Admin
    pub fn erase_tenant(&self, id: TenantId) -> Result<ErasureTicket>;
    pub fn change_passphrase(&self, new: SecretString) -> Result<()>;       // wrappt nur K_tenant neu
    pub fn backup(&self, dest: &Path) -> Result<BackupManifest>;
    pub fn restore(src: &Path, b: ContextraBuilder) -> Result<(Contextra, RestoreOutcome)>;
    pub fn tickets(&self) -> Result<Vec<TicketInfo>>;                       // offene und jüngst abgeschlossene
    pub fn ticket(&self, id: TicketId) -> Result<TicketInfo>;
    pub fn health(&self) -> HealthReport;
    pub fn compliance_report(&self) -> Result<ComplianceReport>;            // TOM, Register, Befunde
    pub fn close(self) -> Result<()>;                                       // schließt Worker, persistiert Tickets
}
```

Im Modus `TenantMode::Single` gibt es genau einen Mandanten; `Contextra::default_tenant()` liefert dessen Handle. Ein Zugriff **ohne** Mandantenhandle existiert in der API nicht.

### 8.2 Handles

```rust
pub struct TenantHandle { /* privat */ }
impl TenantHandle {
    pub fn create_collection(&self, cfg: CollectionConfig) -> Result<CollectionHandle>;
    pub fn collection(&self, name: &str) -> Result<CollectionHandle>;
    pub fn list_collections(&self) -> Result<Vec<CollectionInfo>>;
    pub fn erase_collection(&self, name: &str) -> Result<ErasureTicket>;
    pub fn erase_subject(&self, s: &SubjectRef) -> Result<ErasureTicket>;
    pub fn subject_documents(&self, s: &SubjectRef) -> Result<Vec<DocInfo>>;     // Art. 15
    pub fn export_subject(&self, s: &SubjectRef) -> Result<SubjectExport>;        // Art. 20
}

pub struct CollectionHandle { /* privat */ }
impl CollectionHandle {
    pub fn add(&self, src: NewSource) -> Result<AddOutcome>;                      // atomar
    pub fn add_many(&self, srcs: Vec<NewSource>) -> Result<Vec<AddOutcome>>;      // atomar: alles oder nichts
    pub fn ingest(&self, req: IngestInput) -> Result<AddOutcome>;                 // Datei -> Chunks -> add
    pub fn get_chunk(&self, id: ChunkId) -> Result<Chunk>;                        // NotFound auch für gelöscht
    pub fn get_document(&self, id: DocId) -> Result<Document>;
    pub fn search(&self, q: SearchRequest) -> Result<SearchResponse>;
    pub fn replace_document(&self, id: DocId, src: NewSource) -> Result<ReplaceOutcome>;
    pub fn erase_document(&self, id: DocId) -> Result<ErasureTicket>;
    pub fn restrict_document(&self, id: DocId, r: Restriction) -> Result<()>;
    pub fn release_restriction(&self, id: DocId) -> Result<()>;
    pub fn stats(&self) -> Result<CollectionStats>;
    pub fn info(&self) -> CollectionInfo;
}

pub struct ErasureTicket { /* TicketId, Verweis auf Eraser */ }
impl ErasureTicket {
    pub fn id(&self) -> TicketId;
    pub fn status(&self) -> TicketStatus;                                         // nicht blockierend
    pub fn wait(&self, at_least: ClassSet, timeout: Duration) -> Result<ErasureReceipt>;
}
```

`erase_*` kehrt nach Klasse L zurück. `wait(ClassSet::C, …)` blockiert bis Klasse C. `ErasureTicket` ist `Clone` und kann nach Neustart über `Contextra::ticket(id)` wiedergewonnen werden.

### 8.3 Datentypen

```rust
pub struct CollectionConfig {
    pub name: String,                        // 1..=128 Zeichen, [A-Za-z0-9._-]
    pub dimension: usize,                    // muss zum Embedder passen
    pub metric: DistanceMetric,              // Cosine (Standard), Dot, L2
    pub retention: Option<Duration>,         // Löschfrist je Dokument (Art. 5 Abs. 1 lit. e)
    pub embedding_model: ModelFingerprint,   // verhindert Vermischung
    pub language: Language,                  // Standard German
}

pub struct NewSource {
    pub chunks: Vec<NewChunk>,               // 1..=max_chunks
    pub metadata: Metadata,                  // opake Schlüssel/Werte, unter DocDek versiegelt
    pub subjects: Vec<SubjectRef>,           // betroffene Personen
    pub legal_hold: Option<LegalHold>,       // Aufbewahrungspflicht
    pub external_ref: Option<String>,        // Verweis der Anwendung (wird nie als Klartext persistiert)
}
pub struct NewChunk {
    pub text: String,
    pub embedding: Option<Vec<f32>>,         // sonst über den Embedder
    pub locator: Option<Locator>,            // Seite, Bytebereich, Tabellenblatt/Zelle
}
pub struct AddOutcome { pub doc_id: DocId, pub chunks: u32, pub warnings: Vec<IngestWarning> }

pub struct SearchRequest {
    pub query: String,
    pub k: usize,                            // 1..=1000
    pub mode: SearchMode,                    // Hybrid (Standard) | Vector | Text
    pub filter: Option<MetadataFilter>,
    pub query_vector: Option<Vec<f32>>,
    pub fusion: FusionConfig,                // RRF, Gewichte
    pub include_text: bool,                  // Standard true
    pub explain: bool,                       // Standard false
}
pub struct SearchResponse { pub hits: Vec<SearchHit>, pub stats: SearchStats }
pub struct SearchHit {
    pub chunk: ChunkId, pub doc: DocId, pub score: f32,
    pub text: Option<String>, pub locator: Option<Locator>,
    pub metadata: Metadata,
    pub signals: Option<Vec<SignalScore>>,   // nur bei explain
}
pub struct SearchStats { pub candidates: u32, pub took_micros: u64, pub exact_scan: bool, pub index_state: IndexState }

pub enum MetadataFilter {                    // typisiert, ohne Freitext-Ausdruck
    Eq(String, MetaValue), In(String, Vec<MetaValue>),
    Range { key: String, min: Option<MetaValue>, max: Option<MetaValue> },
    Exists(String), And(Vec<MetadataFilter>), Or(Vec<MetadataFilter>), Not(Box<MetadataFilter>),
}
pub enum MetaValue { Str(String), Int(i64), Float(f64), Bool(bool), Time(Timestamp) }

pub struct LegalHold { pub reason: LegalHoldReason, pub until: Option<Timestamp> }
pub struct Restriction { pub reason: RestrictionReason, pub until: Option<Timestamp> }
```

Weitere Regeln:

- Die Suche ist **mandantengebunden durch den Handle**, nicht durch einen Parameter. Ergebnisse enthalten nie gesperrte, eingeschränkte oder gelöschte Chunks.
- Metadaten sind unter `DocDek` versiegelt. Der Metadatenfilter arbeitet auf einem **RAM-Metadatenindex**, der beim Öffnen aus dem Speicher aufgebaut und im `Purgeable`-Register geführt wird.
- `replace_document` fügt zuerst die neue Fassung ein und löscht dann die alte (Ticket). Eine Aktualisierung an Ort und Stelle gibt es nicht, weil dabei Klartext der alten Fassung in Indizes verbliebe.
- Kein öffentlicher Snapshot- oder Time-Travel-Zugriff.

### 8.4 Konsistenzmodell

- **Der Speicher (`KvStore`) ist die Wahrheit.** `add` schreibt Katalog, Chunks, Vektoren, Metadaten, Forward-Terme und Subject-Einträge in **einer** Speichertransaktion. `Ok` bedeutet: durabel festgeschrieben.
- Vektor- und Textindex sind **Ableitungen im Speicher**. Nach dem Festschreiben werden sie aktualisiert; die Aktualisierung ist wiederholbar. Beim Öffnen werden sie aus dem Speicher aufgebaut (parallel entsiegelt). Ein optionaler strukturreiner Index-Snapshot (Nachbarlisten ohne Vektoren, versiegelt unter `K_coll`, geprüft gegen den Speicher-Wasserstand) beschleunigt den Start und ist Teil von V1 erst nach bestandenem Gate G-V4.
- **Es gibt keine persistierten Klartext-Indexdateien.** Eine HNSW-Datei mit Klartextvektoren auf der Platte wäre eine Ebene-B-Verletzung und ist ausgeschlossen.
- Lesende Zugriffe sehen einen konsistenten Zustand der letzten festgeschriebenen Transaktion, gefiltert durch die `LiveMask`.

### 8.5 `AdminHandle` und Betrieb

Operationen, die einen Mandanten oder die Instanz betreffen (`create_tenant`, `erase_tenant`, `change_passphrase`, `backup`, `restore`, `tickets`, `health`, `compliance_report`), liegen auf `Contextra` und sind **keine** Operationen eines Mandantenhandles. Eine Anwendung, die nur den Mandantenhandle weitergibt, kann sie nicht aufrufen. `AuthorizationHook` (Abschnitt 9) wird an jeder Grenze aufgerufen.

---

## 9. Service-Provider-Schnittstellen (Ports)

Alle Ports sind **synchron**, `Send + Sync`, ohne `async`. Jeder Port hat `capabilities()`. Alle Signaturen sind Entwurf.

### 9.1 Überblick

| Port | Zweck | Kern-Implementierung | Enterprise-Erweiterung |
|---|---|---|---|
| `KvStore` | Persistenz, Transaktionen | `redb`-Adapter [?] | SQLite-, verteilte Speicher |
| `ErasureJournal` | Durables Journal mit Group-Commit | Datei, versiegelt, Hash-Kette | verteiltes Journal |
| `KeyVault` | Ledger, Schlüsselvernichtung | `FileVault` (`FileOverwrite`) | TPM, Secure Element, HSM, Cloud-KMS (`Effaceable`) |
| `VectorIndex` | Vektorsuche | HNSW, Exakt-Scan | `usearch`-Adapter u. a. |
| `TextIndex` | Volltextsuche | BM25 mit Komposita | – |
| `EmbeddingProvider` | Vektoren erzeugen | Ollama (lokal) | ONNX, andere Dienste |
| `IngestPort` | Dokument → Chunks | Worker-Prozess | weitere Formate |
| `PiiRecognizer` | Personenbezug erkennen | Muster und Prüfziffern | NER-Modelle |
| `LayerPurger`, `Purgeable` | Ebene-A-Bereinigung | je Index | zusätzliche Schichten |
| `AuthorizationHook` | Zugriffsprüfung an der API-Grenze | erlaubt alles innerhalb des Mandantenhandles | RBAC, SSO |
| `AuditSink` | Inhaltsfreie Ereignisse | Hash-Kette in Datei | SIEM-Export |
| `ProofSigner` | Berichte signieren | Ed25519 mit versiegeltem Schlüssel | HSM-Signatur |
| `TimestampAuthority` | Qualifizierter Zeitstempel | keine | eIDAS-Anbieter |
| `RetentionPolicy` | Aufbewahrungsentscheidung | immer löschen | Regelwerk |
| `Reranker` | Nachsortierung (optional) | keine | Cross-Encoder |
| `Clock`, `Rng`, `Vfs`, `MetricsSink` | Basisdienste | System, OS-CSPRNG, `StdVfs`, keine | – |

### 9.2 `KvStore`

```rust
#[non_exhaustive] pub enum Table { Catalog, Chunks, Vectors, Meta, Postings, Blobs, Subjects, Misc }

pub trait KvStore: Send + Sync + 'static {
    fn capabilities(&self) -> StoreCaps;
    fn read(&self) -> Result<Box<dyn KvRead + '_>>;               // konsistenter Lese-Snapshot
    fn begin_write(&self) -> Result<Box<dyn KvWrite + '_>>;       // genau ein Schreiber, sonst Wartezeit
    fn reclaim(&self) -> Result<ReclaimStats>;                    // gibt gelöschten Platz frei
    fn stats(&self) -> Result<StoreStats>;
}
pub trait KvRead: Send {
    fn get(&self, t: Table, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn scan_prefix(&self, t: Table, prefix: &[u8], limit: usize, after: Option<&[u8]>)
        -> Result<Vec<(Vec<u8>, Vec<u8>)>>;
}
pub trait KvWrite: KvRead {
    fn put(&mut self, t: Table, key: &[u8], value: &[u8]) -> Result<()>;
    fn delete(&mut self, t: Table, key: &[u8]) -> Result<()>;
    fn commit(self: Box<Self>) -> Result<CommitInfo>;             // Ok erst nach durablem Flush
    fn rollback(self: Box<Self>) -> Result<()>;
}
```

**Vertrag:**

1. **Kein Geister-Commit.** Liefert `commit` einen Fehler, ist die Transaktion auch nach Neustart nicht sichtbar. Ein abgebrochener Aufruf ändert nichts.
2. **Schlüssel sind opak.** Sie bestehen nur aus Zahlen, Zufalls-IDs und blinden Termen, nie aus Namen, Klartext-IDs oder Präfixen mit Personenbezug.
3. **Werte sind bereits versiegelt** (`Sealed`). Der Adapter verschlüsselt nicht selbst und sieht nie Klartext von Dokumenten.
4. Abgeschlossene Leser blockieren Schreiber nicht und umgekehrt.
5. `reclaim` ist ein Hinweis zur Platzfreigabe. Die Löschwirkung beruht **nicht** darauf.

**Konformitätssuite** (Testkit): Atomarität, Isolation, Crash-Kill-Matrix (SIGKILL vor und nach jedem `fsync`), Fehlerinjektion über `FaultVfs`, Abbruch eines `commit`, parallele Leser unter Schreiblast, Präfix-Scan-Vollständigkeit.

### 9.3 `ErasureJournal`

```rust
pub trait ErasureJournal: Send + Sync + 'static {
    fn capabilities(&self) -> JournalCaps;
    /// Ok erst nach durablem Flush. Gruppiert intern (Group-Commit).
    fn append(&self, rec: JournalRecord) -> Result<JournalSeq>;
    fn replay(&self, from: JournalSeq) -> Result<Vec<JournalRecord>>;
    fn compact(&self, upto: JournalSeq) -> Result<()>;            // entfernt Nutzdaten abgeschlossener Tickets
}
```

Journal-Einträge sind unter `K_journal` versiegelt und per Hash-Kette verbunden. Sie enthalten Slots, Ledger-Positionen, Commitments und Zähler, **keine** Schlüssel, Inhalte oder Klartext-Kennungen. Nach `Completed` werden die Nutzdaten (Slots, Ledger-Positionen) durch `compact` entfernt; es bleibt eine Zusammenfassung.

### 9.4 `KeyVault`

```rust
pub trait KeyVault: Send + Sync + 'static {
    fn capabilities(&self) -> VaultCaps;                                   // Effaceable | FileOverwrite | MemoryOnly
    fn unlock(&self, source: &KeySource) -> Result<()>;
    fn new_doc_key(&self, tenant: TenantId, coll: CollectionId) -> Result<(KeyRef, DocKey)>;
    fn open_doc_key(&self, r: KeyRef) -> Result<DocKey>;                   // KeyAbsent, wenn vernichtet
    fn present(&self, r: KeyRef) -> Result<bool>;
    fn destroy_batch(&self, refs: &[KeyRef]) -> Result<DestroyStats>;      // idempotent
    fn rotate_epoch(&self, touched: &BucketSet) -> Result<()>;             // Epochen-Rewrite
    fn destroy_collection_key(&self, tenant: TenantId, coll: CollectionId) -> Result<()>;
    fn destroy_tenant_key(&self, tenant: TenantId) -> Result<()>;
}
```

`DocKey` ist `ZeroizeOnDrop`, hat kein `Debug`-Inhalt, kein `Clone`, kein `Serialize`.

### 9.5 `VectorIndex`

```rust
pub trait SlotFilter: Send + Sync { fn allows(&self, s: Slot) -> bool; }   // LiveMask, Metadatenfilter

pub trait VectorIndex: Send + Sync + 'static {
    fn capabilities(&self) -> VectorCaps;
    fn insert_batch(&self, items: &[(Slot, &[f32])]) -> Result<()>;
    fn search(&self, q: &[f32], k: usize, filter: &dyn SlotFilter) -> Result<Vec<ScoredSlot>>;
    fn purge_slots(&self, slots: &[Slot]) -> Result<PurgeStats>;           // nullt Nutzdaten, idempotent
    fn is_zeroed(&self, slots: &[Slot]) -> Result<bool>;                   // unabhängige Messung
    fn rebuild_if_needed(&self) -> Result<RebuildOutcome>;                 // Shadow-Rebuild, Arc-Austausch
    fn stats(&self) -> Result<VectorStats>;
}
```

Vertrag: `search` ruft `filter.allows` **innerhalb** der Traversierung auf (gefilterte Suche), nicht erst nach der Suche; tote Slots erscheinen nie im Ergebnis. `purge_slots` ist idempotent und nullt fp32 und SQ8. Keine Graph-Umverdrahtung je Löschung.

### 9.6 `TextIndex`

```rust
pub trait TextIndex: Send + Sync + 'static {
    fn capabilities(&self) -> TextCaps;
    fn index_batch(&self, items: &[(Slot, &str)]) -> Result<()>;           // Tokenisierung, Komposita, blinde Terme
    fn search(&self, query: &str, k: usize, filter: &dyn SlotFilter) -> Result<Vec<ScoredSlot>>;
    fn mask_slots(&self, slots: &[Slot]) -> Result<PurgeStats>;            // Maske
    fn merge_if_needed(&self) -> Result<MergeOutcome>;                     // entfernt tote Einträge physisch
    fn stats(&self) -> Result<TextStats>;
}
```

Vertrag: Das persistierte Wörterbuch enthält nur blinde Terme. Die BM25-Statistik (`df`, `avgdl`) zählt tote Dokumente bis zum Merge mit. Dieser Fehler ist klein, begrenzt und per Test beschränkt.

### 9.7 `EmbeddingProvider`

```rust
pub trait EmbeddingProvider: Send + Sync + 'static {
    fn capabilities(&self) -> EmbedCaps;
    fn fingerprint(&self) -> ModelFingerprint;                             // Modell, Version, Dimension
    fn dimension(&self) -> usize;
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
    fn egress_targets(&self) -> Vec<EgressTarget>;                         // Pflichtdeklaration
}
```

Vertrag: Ein Provider mit Netzzugriff **muss** seine Ziele deklarieren. Der Kern vergleicht sie beim Start mit der `EgressPolicy` und weigert sich zu starten, wenn ein Ziel nicht erlaubt ist (fail-closed). Der Standard-Adapter spricht nur mit `127.0.0.1`/`::1`. Es gibt **keinen** stillen Rückfall (kein Embedder konfiguriert ⇒ Fehler beim Start, wenn Text-Einfügen oder -Suche verlangt wird).

### 9.8 `IngestPort`

```rust
pub trait IngestPort: Send + Sync + 'static {
    fn capabilities(&self) -> IngestCaps;                                  // unterstützte Formate
    fn ingest(&self, req: &IngestRequest, cfg: &IngestConfig) -> Result<IngestOutput>;
}
```

Details und Worker-Protokoll siehe Abschnitt 11.

### 9.9 `PiiRecognizer`

```rust
pub trait PiiRecognizer: Send + Sync + 'static {
    fn capabilities(&self) -> PiiCaps;                                     // Entitätstypen, Sprachen
    fn scan(&self, text: &str) -> Result<Vec<PiiSpan>>;                    // Typ, Bereich, Konfidenz
}
```

### 9.10 `LayerPurger` und `Purgeable`

```rust
pub trait LayerPurger: Send + Sync {
    fn layer(&self) -> LayerId;
    fn purge_slots(&self, slots: &[Slot]) -> Result<PurgeStats>;           // idempotent
    fn witness(&self, slots: &[Slot]) -> Result<LayerWitness>;             // unabhängig vom Purger
}
pub trait Purgeable: Send + Sync {                                         // jede Struktur mit Slot-adressierten Kopien
    fn layer(&self) -> LayerId;
    fn invalidate(&self, slots: &[Slot]);
    fn is_clean(&self, slots: &[Slot]) -> bool;
}
```

Ein **verpflichtendes Register** erfasst jede Struktur mit Slot-adressierten Kopien (Embedding-Cache, Abfrage-Cache, Chunk-Cache, RAM-Metadatenindex). Ein Test gegen den `LayerCatalog` scheitert, wenn eine Schicht fehlt. Der `LayerWitness` hat keinen öffentlichen Konstruktor außerhalb der Scan-Sitzung (nicht fälschbar durch Aufrufer).

### 9.11 `AuthorizationHook`

```rust
pub trait AuthorizationHook: Send + Sync + 'static {
    fn authorize(&self, ctx: &CallContext, action: Action, scope: &AuthScope) -> Decision;
}
pub enum Action { Read, Search, Write, Erase, Restrict, Admin, Backup, Restore, Export }
pub enum Decision { Allow, Deny(DenyReason) }
```

`CallContext` enthält Mandant, optional einen opaken Principal-Verweis (vom Aufrufer), keine Personendaten. Standardimplementierung: alles innerhalb des Mandantenhandles erlaubt. Die Hook wird an **jeder** öffentlichen Operation aufgerufen. Eine Ablehnung ist `PolicyViolation`.

### 9.12 `AuditSink`

```rust
pub trait AuditSink: Send + Sync + 'static {
    fn record(&self, ev: &AuditEvent) -> Result<()>;
}
pub struct AuditEvent {              // inhaltsfrei
    pub kind: AuditKind,             // Start, Stop, Erase{Requested|Frozen|Completed}, Restrict, Restore, KeyEvent, ConfigChange
    pub at: Timestamp,
    pub ticket: Option<TicketId>,
    pub class: Option<ClassSet>,
    pub counts: Counts,              // Anzahlen, keine IDs
    pub commitment: Option<[u8; 32]>,
}
```

Standard: Hash-verkettete Datei (BLAKE3-Kette), extern prüfbar mit `contextra-verify`. Ereignisse enthalten **keine** Klartext-IDs, Prompts, Antworten oder Inhalte. Das hält die Aufzeichnungspflicht (z. B. Art. 12 KI-Verordnung bei Hochrisiko-Einsatz) und die Löschpflicht vereinbar.

### 9.13 Übrige Ports

```rust
pub trait Clock: Send + Sync + 'static { fn now_unix_nanos(&self) -> u64; fn monotonic_nanos(&self) -> u64; }
pub trait Rng: Send + Sync + 'static { fn fill_bytes(&self, dest: &mut [u8]); }          // nur OS-CSPRNG im Release
pub trait Vfs: Send + Sync + 'static {
    fn read(&self, p: &Path) -> Result<Vec<u8>>;
    fn write_atomic(&self, p: &Path, d: &[u8]) -> Result<()>;
    fn pwrite(&self, p: &Path, off: u64, d: &[u8]) -> Result<()>;
    fn pread(&self, p: &Path, off: u64, len: usize) -> Result<Vec<u8>>;
    fn sync_file(&self, p: &Path) -> Result<()>;
    fn sync_dir(&self, p: &Path) -> Result<()>;
    fn rename(&self, from: &Path, to: &Path) -> Result<()>;
    fn remove_file(&self, p: &Path) -> Result<()>;
    fn list_dir(&self, p: &Path) -> Result<Vec<String>>;
    fn exists(&self, p: &Path) -> Result<bool>;
}
pub trait MetricsSink: Send + Sync + 'static { fn counter(&self, name: &'static str, v: u64, l: &[(&'static str, &'static str)]); /* gauge, histogram */ }
pub trait ProofSigner: Send + Sync + 'static { fn key_id(&self) -> [u8; 16]; fn public_key(&self) -> [u8; 32]; fn sign(&self, msg: &[u8]) -> Result<[u8; 64]>; }
pub trait TimestampAuthority: Send + Sync + 'static { fn stamp(&self, digest: &[u8; 32]) -> Result<TimestampToken>; }
pub trait RetentionPolicy: Send + Sync + 'static { fn check(&self, scope: &ErasureScope, now: Timestamp) -> RetentionDecision; } // Delete | Restrict { until, reason }
pub trait Reranker: Send + Sync + 'static { fn rerank(&self, query: &str, cands: &[Candidate]) -> Result<Vec<Rerank>>; }
```

`Vfs` hat in `contextra-store` eine Produktivimplementierung `StdVfs` und im Testkit `FaultVfs` (Fehlerinjektion, Abbruchpunkte). Heute existiert ein `Vfs`-Trait ohne produktive Implementierung neben einem zweiten in einem nicht kompilierbaren Crate [K]. Künftig gibt es genau einen.

### 9.14 Bestand → SOLL (Ports)

| Bestand [K] | SOLL |
|---|---|
| `StorageEngine` (26 Methoden, MVCC-/SSI-förmig, async) | `KvStore` (5 Methoden plus `KvRead`/`KvWrite`, sync) |
| `VectorIndex` (async, `DocId`, Snapshot-Lesen, Rollback) | `VectorIndex` (sync, `Slot`, gefilterte Suche, `purge_slots`) |
| `TextIndex` (async, `DocId`) | `TextIndex` (sync, `Slot`, Maske, Merge) |
| `EmbeddingProvider`, `TextEmbeddingEngine` (zwei Traits) | ein `EmbeddingProvider` mit Egress-Deklaration |
| `LlmTextGenerator`, `TextGenerator`, `QueryRewriter`, `GroundingValidator` | nicht im Kern (Generierung ist Sache der Anwendung) |
| `Purger`, `LayerScanner`, `ScanSession` | `LayerPurger` mit `witness`, `Purgeable` |
| `Checkpoint`, `CheckpointCoordinator`, `GraphIndex`, `GraphCollectionMutation`, `CommunityResolver` | entfallen (Labor) |
| `LicenseGate`, `PluginManifest` | entfallen (Enterprise) |
| `KvPrefixStore`, `KvBridgeStorage` | entfallen |
| `Clock`, `IdGen`, `Rng`, `Vfs`, `MetricsSink` | `Clock`, `Rng`, `Vfs`, `MetricsSink` (`IdGen` entfällt: IDs kommen aus dem `Rng`) |
| `DriftStatusProvider`, `AttentionExporter`, `Reranker` | `Reranker` optional, Rest entfällt |

---

# Contextra v1 – Spezifikation (SPEC)

| Feld | Wert |
|---|---|
| Status | Zur Freigabe durch den Auftraggeber |
| Spezifikationsversion | 1.0.0-spec.1 |
| Stand | 2026-11-01 |
| Repository | `github.com/tfufuz1/contextra`, `main` @ `76ea331c697d95fae195b1009a737dc2b5d8f486` |
| Geltung | Verbindlich für v1. Ersetzt die Dokumente *Spezifikation v1 Erasure*, *Spezifikation v1 Final*, *Spezifikation und Umsetzungsplan*, *Implementierungsspezifikation SOLL* und *Implementierungsspezifikation Erasure Core* als Arbeitsgrundlage. |
| Verhältnis zu `AGENTS.md` | Teilweise überschrieben durch ADR-001 (Abschnitt 3.3). Bis dahin gilt `AGENTS.md`, soweit nicht im Widerspruch. |
| Begleitdokument | `PLAN.md` (Arbeitspakete, Gates, Reihenfolge) |

> Keine Rechtsberatung. Rechtliche Aussagen sind Arbeitshypothesen für die Prüfung durch Fachanwalt bzw. Datenschutzbeauftragte (Gate G-LEGAL).

---

## 0. Evidenz und Prüfmodus

### 0.1 Kennzeichnung

| Marke | Bedeutung |
|---|---|
| **[V]** | Am Clone-HEAD `76ea331c6` statisch gelesen. Datei und Zeile angegeben. |
| **[U]** | Ergebnis des Build- und Auditlaufs des Auftraggebers (`cargo check --workspace`, `scripts/contextra_full_audit.py --all`). Nicht reproduziert. |
| **[K]** | Aus *Fehlerkatalog Stand 2026-11-01* (Stand `f653f0f73`). Am Clone nicht erneut gelesen. |
| **[D]** | Aus den Dokumenten übernommen, nicht geprüft (insbesondere Rechtssynthese). |
| **[E]** | Entscheidung, Empfehlung oder Schätzung dieser Spezifikation. |

### 0.2 Prüfmodus

Diese Spezifikation wurde ausschließlich **statisch und online** erstellt: Quelltextlektüre, Manifest- und Abhängigkeitsauswertung, Musterzählung per `grep`. Es wurde **nichts gebaut und nichts getestet**. Alle Aussagen über Kompilierbarkeit und Laufzeitverhalten sind [U] oder Erwartungen, bis sie in WP-0.1 gemessen sind.

Zählungen per `grep` enthalten inline-`#[cfg(test)]`-Module und sind deshalb **Obergrenzen** für Produktionscode.

### 0.3 Versionierung der Quellen

Die Quelldokumente beziehen sich auf unterschiedliche Repo-Stände (`9da0eca1b`, `f653f0f73`, `c4e285c`). Der geklonte HEAD ist `76ea331c6`. Die Zahlen der Quellen sind deshalb nur mit Vorsicht übernehmbar. Diese Spezifikation nimmt den Clone-HEAD als Wahrheit (Prinzip P9).

---

## 1. Produkt

### 1.1 Mission

Contextra ist eine **eingebettete Rust-Bibliothek** für lokale Wissens- und Gedächtnisspeicherung von Agenten und RAG-Systemen (Chunks, Volltext, Vektoren, Metadaten, Zuordnung zu betroffenen Personen). Sie löscht personenbezogene Daten, die sie selbst hält, **physisch und gemessen**: im Datenverzeichnis, in allen Indexen, in allen MVCC-Versionen (Zeitreisen) und im Prozessspeicher, soweit steuerbar. Der Löschbericht weist den gemessenen Zustand je Speicherort aus und benennt die Restrisiken.

### 1.2 Zielnutzer

Entwickler und Betreiber von Agenten- und RAG-Systemen im EU-Raum, die Löschpflichten (Art. 17 DSGVO) technisch umsetzen müssen und einen prüfbaren Nachweis brauchen.

### 1.3 Nicht-Ziele v1

- Netzwerkdienst, Server-Socket, Sharding, Replikation, P2P.
- Erzeugung von Embeddings, Aufrufe von LLM-Diensten (Vektoren und Text kommen vom Aufrufer).
- Dateiformat-Parsing (PDF, DOCX, EPUB, ...).
- Graph-Algorithmen und Relationen.
- Schlüsselvernichtung als Löschmechanismus (reserviert, Abschnitt 3, D-01).
- Einschränkung der Verarbeitung (Art. 18) als Speicherzustand.
- Persistente Caches (Embedding-, Query-, KV-Cache).
- Automatische Entitätsextraktion.

### 1.4 Verspricht und verspricht nicht

**Verspricht (prüfbar):**

1. Nach abgeschlossener Löschung enthält kein Datenverzeichnis-File, kein Index und kein Contextra-kontrollierter Speicherbereich einen gemessenen Klartextrest des Ziels. Die Messung ist unabhängig vom Löschmechanismus.
2. Ein Löschauftrag wirkt ausschließlich auf sein typisiertes Ziel. Alle Nicht-Ziel-Daten bleiben bitgleich erhalten (T-ISO-1).
3. Unterbrechungen (Absturz, I/O-Fehler) führen nach Neustart zur Vollendung oder zu sichtbarem Status `Pending`. Stille Teilzustände gibt es nicht.
4. Der Bericht benennt genau, was gemessen wurde, und listet alle Restrisiken.

**Verspricht nicht:**

- „DSGVO-konform“, „rechtskonform“, „garantiert gelöscht“, „unwiederbringlich“, „informationstheoretisch“, „kryptografisch gelöscht“ (Abschnitt 13.2).
- Löschung außerhalb von Contextra (Backups, Volume-Snapshots, Logs fremder Systeme, Kopien beim Betreiber).
- Löschung auf Hardware-Ebene (Flash-Remanenz, Dateisystem-Journale, Swap).

---

## 2. Leitprinzipien (normativ)

| ID | Prinzip | Konkrete Folge |
|---|---|---|
| P1 | **Wirkung vor Behauptung** | Ein Zähler oder Status „gelöscht“ entsteht nur aus einem Zeugen (`LayerWitness`) eines unabhängigen Scanners. |
| P2 | **Fail-closed** | Unbekannter Zustand, korrupte Daten, abgelaufene Fristen: Verweigerung oder `Pending`. Kein Weiterlaufen. |
| P3 | **Ein Weg** | Ein Löschpfad, eine Fehlerbehandlung, ein Format je Konzept. Keine Legacy-Lesepfade. |
| P4 | **Ziel-Isolation** | Jede mutierende Operation hat ein typisiertes Ziel. Nachbarn sind bitgleich unverändert (messbar). |
| P5 | **Keine Schalter für Sicherheit** | Kein `cfg(feature)` auf Verschlüsselung, Integrität, Löschmessung, Mandantenprüfung (AGENTS §2 A1). |
| P6 | **Keine Panics, keine verschluckten Fehler** | Im Produktionscode kein `unwrap`, `expect`, `panic!`, `let _ =` auf Fallible, `unwrap_or_default` auf I/O. |
| P7 | **Determinismus über Ports** | Zeit (`Clock`), Zufall (`Rng`), Dateisystem (`Vfs`), IDs (`IdGen`). Ausnahme: Schlüssel-, Salt- und Nonce-Material aus OsRng. |
| P8 | **Kleinster Kern** | Ein Crate oder Modul ist nur im Kern, wenn ein Kernkonsument es braucht (`cargo tree`-belegt). |
| P9 | **Messen vor Optimieren** | Jedes Performance-Gate hat eine Basislinie auf fester Hardware. Ohne Basislinie kein Gate. |
| P10 | **Verifizierbarkeit vor Eleganz** | Jede Designentscheidung nennt den Test, der sie belegt. Ohne Test keine Entscheidung. |

---

## 3. Entscheidungsregister

### 3.1 Entscheidungen

| ID | Entscheidung | Begründung | Verworfene Alternative | Status |
|---|---|---|---|---|
| **D-01** | **Löschmechanismus v1: physische Purge-Compaction plus Messung.** Schlüsselvernichtung (Klasse C) ist nicht Teil von v1. | Die Rechtswirkung der Schlüsselvernichtung ist offen [D]. Eine Mission „physische Löschung“ darf nicht an dieser Frage hängen. Purge wirkt unabhängig davon und ist messbar. Schlüsselvernichtung braucht außerdem einen löschbaren Anker, den Contextra nicht bieten kann (siehe N10 in Quelldokument Erasure Core). | Zwei-Ebenen-Modell mit DocDek und KeyAnchor (Quelle *Erasure*, *Erasure Core*, AGENTS §4 Ebene B). Entfällt in v1, Schnittstelle bleibt frei (X-1). | [E], Auftraggeber bestätigt |
| **D-02** | **Speicher: eigener LSM-Tree behalten**, hinter dem Port `StorageEngine`. redb ist **Rückfallebene** (Adapter nur bei Ausstieg, Abschnitt 3.2 Gate G0). | Purge-Compaction und atomarer Manifest-Tausch sind LSM-Primitive. Bei redb oder SQLite wüchse die Schreibverstärkung mit der Gesamtdatenbank, nicht mit dem Löschziel [E, nicht gemessen]. Die Begründung „redb kann kein Crypto-Shredding“ trägt nicht und wird nicht verwendet. | redb als Vollersatz (Bericht 3). Würde D-01 ohne Purge-Rewrite nicht erfüllen. | [E], Gate G0 offen |
| **D-03** | **Ports async** (`BoxFuture`), konsistent zu `StorageEngine`. Keine `block_on` im Bibliothekscode. | Synchrone Ports über asynchroner I/O erzwingen `block_on` und sind Ursache der Deadlock-Befunde C-L18/C-L19 [K]. | Synchrone Ports mit Blocking-Adapter. | [E] |
| **D-04** | **Ein `Vfs`-Trait**, in `contextra-ports` definiert, in `contextra-durable-fs` implementiert. Der Store und die Engine nutzen ausschließlich `Vfs`. | Zwei `Vfs`-Traits existieren ([V] `ports/src/vfs.rs:29`, `durable-fs/src/vfs.rs:54`). Der Store nutzt `Vfs` nicht [V: 0 Treffer]. Fehlerinjektion (FaultVfs) und Crash-Tests setzen `Vfs` voraus. | Zwei Traits mit Adapter zwischen beiden. | [E] |
| **D-05** | **Verschlüsselung ist nicht abschaltbar in v1.** Es gibt keinen Klartextmodus im Produktionsbuild. Tests dürfen einen In-Memory-Fixture-Schlüssel nur unter `cfg(test)` nutzen. | Default ist heute unverschlüsselt [V: `contextra/src/builder.rs:147` `encryption_mode: None`]. Ein Opt-out macht jede Erase-Aussage zur Ausnahme. P5 verbietet Schalter. | Default an, Opt-out erlaubt (Final R-SEC-1). Verworfen, weil das Opt-out selbst zum Parallelbestand führt. | [E], Breaking Change |
| **D-06** | **Passphrase nur über Argon2id.** Mindestparameter m ≥ 64 MiB, t ≥ 3, p ≥ 1. Unterschreiten ist Fehler, nicht Warnung. HKDF bleibt nur für Subkey-Ableitung aus dem Master-Key. | HKDF-Ableitung aus Passphrase [V: `crypto/src/crypto.rs:88` `try_new`]. Untergrenze heute 19.456 KiB [V: `crypto/src/kdf.rs:36`]. | Untergrenze 19 MiB beibehalten. | [E] |
| **D-07** | **Mandantenpflicht.** `TenantPolicy::Required` ist der einzige Modus. Operationen ohne `RequestContext` mit Mandanten-ID sind nicht kompilierbar. | Default heute `Optional` [V: `engine/src/lib.rs:207–209`]. VETO-F10 [AGENTS §2 Nr. 7]. | Optional als Default. | [E] |
| **D-08** | **Dokumentidentität: 128 Bit, schlüsselgebunden.** `DocId = BLAKE3-keyed(K_idx, tenant ‖ collection ‖ doc_key)[0..16]`. Beim Insert wird eine Kollision mit anderem `doc_key` als Fehler `IdCollision` abgelehnt. | 64 Bit ohne Schlüssel [V: `types/src/types/domain/ids.rs`, `DocId(pub u64)` ohne Feature] sind Pseudonym und kollisionsanfällig (≈2,7 % bei 10⁹ Dokumenten, Rechnung [E]). Schlüsselbindung verhindert Berechnung aus Nutzerkennung. | 64 Bit mit „ADR-016-Kollisionsprüfung“. ADR-016 ist für DocId nicht belegt [V: nur Decay-Controller]. | [E] |
| **D-09** | **Interne Schlüssel sind blind.** Alle Schlüsselkomponenten außer fester Familienkennung sind keyed Hashes (`K_idx`). Nutzerschlüssel stehen nur verschlüsselt im Wert. Dadurch enthält der Klartext-Index in SSTable-Blöcken keine Nutzerdaten (N2). | Index-Schlüssel werden im Klartext geschrieben [V: `sstable/builder.rs` `flush_block`/`finish`, N2]. Blinde Schlüssel machen Präfix-Matcher überflüssig. | Klartextschlüssel mit Präfix-Matcher (Quelle Erasure Core). | [E], ersetzt Entscheidung E1 der Quelle |
| **D-10** | **Matcher nur exakt.** Präfix-Scans auf Nutzerschlüsseln entfallen. `scan` iteriert die Collection in Schlüsselreihenfolge, nicht nach Nutzer-Präfix. | Präfixlogik ist Ursache von F01/C-L09 (Löschung trifft Mandantendaten) [V: `scanner/purge.rs:36,44`]. Mit D-09 wären Präfixe ohnehin Hashpräfixe ohne fachlichen Wert. | Fixlängen-Präfixe (Quelle Erasure Core 4.4 Nr. 6). | [E], Verhaltensänderung |
| **D-11** | **Zeitreisen bleiben im Umfang.** Purge entfernt alle MVCC-Versionen. Offene Snapshots, die ein betroffenes Ziel berühren, werden **widerrufen** (Fehler `SnapshotRevoked`), nicht abgewartet. | Vorgabe des Auftraggebers. Warten auf Snapshots macht Löschfristen unbegrenzt (C-L17 [K]). Widerruf ist die einzige Variante, die eine feste Frist einhält. | Verzögerter Unlink bis Snapshot-Ende (Quelle Erasure Core 4.5). Verworfen wegen unbegrenzter Restdauer. | [E] |
| **D-12** | **Lesesperre (ReadGate) vor Purge.** Ein Ziel ist ab der persistierten Sperre (`Gated`) für jeden Lesepfad gesperrt, auch für Zeitreise-Lesezugriffe. | Sofortige Wirkung für Nutzer ohne Warten auf den Rewrite. Ohne Gate sind Daten bis zum Rewrite lesbar. | Nur Purge ohne Gate. | [E] |
| **D-13** | **Löschbericht: Ed25519-signiert, kanonisches JSON, Ein Format, keine Legacy-Versionen.** | Externe Prüfbarkeit [D, Synthese]. Symmetrische Beweise sind fälschbar durch den Prüfer [K: B-S10]. V1–V3 werden entfernt. | DeletionProof V1–V3 (HMAC/Legacy). | [E] |
| **D-14** | **Betroffenen-Löschung in v1** über `subject_id` beim Insert. Index `subject_tag → DocIds`. | Ohne Zuordnung kann Contextra eine Person nicht auffinden [D, Final E-SUBJ-1]. | Betroffenen-Löschung erst in v1.1. | [E] |
| **D-15** | **Aufbewahrung: nur Pflichtangabe, kein Scheduler.** `erase` verlangt `RetentionAttestation` (Code-Enum). Der Bericht führt sie. Einschränkung (`Restrict`) entfällt in v1. | Aufbewahrungspflichten sind Betreiberwissen. Eine Pflichtangabe verhindert stilles Löschen, ohne Fristlogik einzuführen [D, Art. 17 Abs. 3]. | `Restrict` in v1 (Quelle Erasure). | [E] |
| **D-16** | **Keine Checkpoints, Forks, Hardlink-Klone in v1.** `contextra-checkpoint` ist aus dem Workspace entfernt. | Jeder Klon ist eine Kopie außerhalb der Löschgrenze. Der Sweep je Klon [K: D-C09, D-C08] wäre fehleranfällig. Zeitreisen bleiben über MVCC erhalten (D-11), nicht über Checkpoints. | Klone mit Sweep (Quelle Erasure Core 4.1 Zeile S12). | [E], Auftraggeber-Entscheidung E-V1-3 |
| **D-17** | **Graph aus v1 entfernt, nicht nur per Feature abgeschaltet.** `contextra-graph` und die Relations-API (`relate`, `__rel:`) entfallen. | Feature-Gates auf Löschpfade sind verboten (P5). Graph-Reparatur pro Löschung ist Veto-Gegenstand (VETO-F02). Nicht im RAG-Kern. | Entkoppeln mit Feature `graph` (Entscheidungsgrundlage W3). Verworfen, weil ein abschaltbarer Graph-Löschpfad denselben Fehler wie `wal-integrity` erzeugt (V-01 [V]). | [E], Owner-Review |
| **D-18** | **Vektorlöschung: Slot-Zeroing plus Batch-Rebuild.** Kein Rewiring der HNSW-Kanten pro Löschung. Rebuild bei Tombstone-Quote ≥ 0,2. | VETO-F02 [VETOES.md]. Kanten enthalten nur Slot-Nummern, keinen Vektorinhalt. | Kantenreparatur pro Löschung (`hnsw/deletion.rs`, 353 Zeilen [V]). | [E] |
| **D-19** | **Vektor-Snapshots und Vektorarena verschlüsselt.** Snapshot-Dateien laufen durch denselben Store-Schlüssel wie SSTables und über `Vfs`. | Vektoren gelten vorsorglich als personenbezogen [D, DSK-OH RAG]. Unverschlüsselte mmap-Snapshots wären ein Parallelbestand. | Snapshots unverschlüsselt (heutiger Zustand, Befund G-03/G-04 [K]). | [E], Implementierung offen |
| **D-20** | **Textindex: Postings als exakte Schlüssel `(term_tag, doc_id)`.** Document Frequency als Zähler je `term_tag`. | Per-Term-Postinglisten über viele Dokumente machen Purge zu Listen-Rewrites [D]. Einzelschlüssel machen Löschung exakt und Matcher trivial. | Postinglisten je Term mit Segment-Rewrite (Quelle Erasure Core 4.7). | [E], Lasttest offen (R-03) |
| **D-21** | **Relations- und Mandanten-Tags sind keyed.** `tenant_tag`, `coll_tag`, `term_tag`, `subject_tag` = BLAKE3-keyed(K_idx, …)[0..16]. | Klartext-Tags würden Mandanten- und Begriffsnamen im Index offenlegen (N5). | Klartextnamen im Schlüssel. | [E] |
| **D-22** | **Entfernung ohne Feature-Gate, mit Archiv.** Entfernte Crates werden in den Branch `archive/v0-full` verschoben, nicht gelöscht. | Historie bleibt auffindbar. Löschung aus dem Hauptzweig macht die Entscheidung prüfbar. | Löschen. | [E] |
| **D-23** | **Lizenz- und DRM-Schicht (`contextra-license`) aus dem Speicherkern entfernt.** | Der Speicher soll keine Lizenzlogik in MIT-Code tragen. Legacy-Lizenzen ohne Installationsbindung sind umgehbar [K: R-03, bestätigt]. Geschäftsmodell ist Auftraggebersache. | Fassade mit `LicenseGate` in v1 (Entscheidungsgrundlage 6.2). | [E], Owner-Entscheidung E-LIC |
| **D-24** | **Ingest, Privacy, Agent, Cognition, Router, Adapt, KV-Cache, Inference, Sandbox, Audit-Export, AVV-Generator aus v1 entfernt.** | Nicht im Kern laut Produktumfang. Ingest parst im Hauptprozess ohne Sandbox [K: B-S25]. Audit-Export hängt an entfernten Beweistypen. AVV-Vorlagen sind nicht fachlich freigegeben [K: G-24/G-25]. | Behalten als Erweiterungen. | [E] |
| **D-25** | **MCP-Server und Python-Bindings sind v1.1-Adapter** auf der stabilen Rust-API. | MCP-Löschwerkzeuge verwerfen `_proof_key` und `_tenant_id` [K: C-L04, bestätigt Zeile `tools_crud.rs`]. Adapter erst nach Freeze der API. | MCP in v1. | [E] |
| **D-26** | **`contextra-rank` auf RRF-Fusion reduziert.** Kalibrierung, Ranking-Strategien und Adaptivität entfallen. | Hybridsuche (HNSW + BM25 + RRF) ist Kern für RAG. Der Rest ist nicht Kern. | Vollständig behalten (Entscheidungsgrundlage 6.2) oder vollständig entfernen. | [E] |
| **D-27** | **Keine automatische Entitäts- oder Metadatenextraktion** in v1. `AutoExtractionMode` entfällt. | Default ist heute `Enabled` [V: `types/src/types/domain/misc.rs:591–597`] und erzeugt ohne Zutun Parallelbestände. Extraktion ist Aufgabe des Aufrufers. | Default `Disabled` mit Feature (AGENTS §5). Verworfen wegen P5. | [E], AGENTS §5 überschrieben |
| **D-28** | **SIMD: u8-VNNI-Kernel entfernt.** Nur f32-Kernel und Skalarpfad. Jeder SIMD-Kernel hat einen Cross-Check-Test gegen Skalar. | Operandentypen des `_mm512_dpbusd_epi32`-Aufrufs sind beide `u8` [V: `simd/src/kernels/avx512.rs:445,565–569`], die Instruktion erwartet u8 × i8. Falsche Distanzen [K: D-V01]. | u8 auf u16 erweitern. Mehr Code für einen Pfad ohne Messgewinn [E]. | [E] |
| **D-29** | **`unsafe` nur in `contextra-simd` und `contextra-sys`.** `contextra-wire` entfällt mit MCP und Python (D-25). | AGENTS §2 Nr. 6 nennt drei Inseln. Eine Insel weniger reduziert Audit-Fläche. | Drei Inseln. | [E], AGENTS-Änderung in ADR-001 |
| **D-30** | **Erase-API synchron mit Frist.** `erase` kehrt nach persistierter Sperre (`Gated`) zurück, nicht nach Ende des Purge. Standardfrist für die Rückgabe 60 s. `erase_sla` (Alarmschwelle für Fertigstellung) ist **Pflichtparameter ohne Standardwert**. | Entscheidungsgrundlage E-10 und Final E-SLA sind vereinbar in dieser Form. Ohne SLA-Wert ist Fail-closed. | Rückgabe erst nach Purge (blockiert Aufrufer unbeschränkt). | [E] |

### 3.2 Gate G0: Speicherentscheidung LSM oder redb

D-02 gilt **vorläufig**. Verbindlich wird sie mit Gate G0 nach Abschluss von WP-2.10 und WP-5.1 (Erasure Lab).

| Prüfung | Vorgabe | Schwelle für Verbleib LSM |
|---|---|---|
| Rohscan nach Erase | identischer Workload (Insert, Update, Erase, Restart, 10⁵ Dokumente) auf LSM und auf redb mit identischer Dokumenthülle | LSM-Rest ≤ redb-Rest; Restmenge nach Purge = 0 Treffer in Positivkontrolle-geprüftem Lauf |
| Schreibverstärkung | geschriebene Bytes je gelöschtem Byte | ≤ 10× Basislinie LSM-ohne-Erase, und ≤ redb-Vergleich |
| Korrektheit | Differential-Test gegen `ReferenceModel` (T-DIFF-1) | 0 Abweichungen über 10⁶ Operationen |
| Crash | Kill-Matrix (T-CRASH-1/2) | 0 verlorene bestätigte Transaktionen |

**Ausstiegsregel:** Verfehlt der LSM die Crash-Matrix (W2.10) zweimal nach einer Frist von vier Wochen oder überschreitet W2.4 seine Zeitbox, startet der redb-Adapter als Spike. Die Erase-Logik (Saga, Gate, Journal, Matcher) bleibt unverändert. Der Purge wird zum Datei-Rewrite der redb-Datei. Die Entscheidung wird mit Zahlen dokumentiert, nicht mit Präferenz.

### 3.3 ADR-001: Änderungen an `AGENTS.md`

`AGENTS.md` ist normativ. Diese Spezifikation ändert sie durch ADR-001 an folgenden Stellen:

| AGENTS-Stelle | Heutiger Wortlaut (Kurz) | Neue Regel |
|---|---|---|
| §1 Löschen | „Ebene B: kryptografische DocDek-Schlüsselvernichtung“ | Ebene B entfällt in v1 (D-01). Löschung = Ebene A (physisch) plus Gate und Messung. |
| §3 Ring-Tabelle | 35 Crates, u. a. `checkpoint`, `ingest`, `privacy` in Ring 1/2 | Ringe laut Abschnitt 5.2 dieser SPEC. |
| §4 Lösch-Saga | Zustände mit `KeyDestroyed` | Zustände laut Abschnitt 8.2. |
| §5 Sicherheits-Defaults | `AutoExtractionMode::Disabled`, `TenantPolicy::Required`, `Verschlüsselung Standard aktiv` mit `EncryptionMode::Required` | Verschlüsselung ohne Opt-out (D-05). `AutoExtraction` entfällt (D-27). |
| §2 Nr. 6 | `unsafe` in `simd`, `sys`, `wire` | `simd` und `sys` (D-29). |

**Widerspruch im Ist-Zustand:** AGENTS §5 beschreibt Defaults, die der Code nicht hat (Abschnitt 4.3). Das ist ein Befund, keine Zustimmung zum Ist-Zustand.

### 3.4 Konflikte zwischen den Quelldokumenten

| # | Konflikt | Auflösung in dieser SPEC | Beleg |
|---|---|---|---|
| K-1 | Entscheidungsgrundlage: 21 Workspace-Mitglieder; Fehlerkatalog: 37 | **37** am Clone | [V] `Cargo.toml` members = 37, default-members = 27 |
| K-2 | Entscheidungsgrundlage V-05: `Collection::delete` gibt Lock nicht frei, „nicht bestätigt“ | **Bestätigt**: Lock wird sofort wieder freigegeben | [V] `engine/src/collection/crud/delete.rs`: `let _guard = …lock_for(id).await; drop(_guard);` |
| K-3 | Final / Entscheidungsgrundlage F6: `scanner` und `hnsw_purger` nicht exportiert, Engine bricht | **Überholt**: Exporte vorhanden | [V] `store/src/lib.rs` `pub mod scanner;`, `vector/src/lib.rs:49` `pub use hnsw::{hnsw_purger, …}` |
| K-4 | Entscheidungsgrundlage: `test-utils` im Store undeklariert | **Nicht bestätigt**; `test-hooks` ist deklariert | [V] `store/Cargo.toml` Features; `encryption-at-rest` (11 Test-Dateien) und `wal-integrity` (7 Gates) sind undeklariert |
| K-5 | Entscheidungsgrundlage N1 „SstableBuilder::create im Produktionscode nicht mehr“ | Aufrufe in `purge.rs` nicht mehr geprüft, Rewrite wird ohnehin neu gebaut (WP-3.8) | [K] |
| K-6 | Fehlerkatalog: Workspace kompiliert nicht (Gruppe A) | **Lib kompiliert**, `--all-targets` mit 26 Fehlern | [U] `cargo check --workspace` fertig ohne Fehler (3 Warnungen); Audit: `check_alle_targets FAIL errors=26` |
| K-7 | Final: DocId 64 Bit mit Kollisionsprüfung nach ADR-016 | Kollisionsprüfung **nicht vorhanden**; ADR-016 betrifft Decay | [V] Suche nach ADR-016: nur Decay-Controller |
| K-8 | AGENTS §5 Default `AutoExtractionMode::Disabled`, `TenantPolicy::Required` | Code: `Enabled`, `Optional` | [V] Abschnitt 4.3 |
| K-9 | Erasure Core / AGENTS: Ebene B mit DocDek | D-01 streicht Ebene B für v1 | Abschnitt 3.1 |
| K-10 | Entscheidungsgrundlage E1 (blinde Nutzer-IDs: v1 nein) vs. Erasure Core E1 (v1 nein) vs. SOLL X-4 (v1.1) | **Ja in v1** (D-09, D-10). Begründung: Klartext-Rest in S4 ist ohne diese Entscheidung nicht ausschließbar. | Abschnitt 3.1 |
| K-11 | Restrict in v1 (Erasure, Erasure Core) vs. v1.1 (Final) vs. Pflichtparameter (SOLL R9) | Pflichtangabe `RetentionAttestation`, kein Restrict (D-15) | Abschnitt 3.1 |
| K-12 | Bericht signiert (Erasure Core, SOLL) vs. Final T-REPORT-2 „falls signiert“ | **Signiert** (D-13) | Abschnitt 3.1 |
| K-13 | Journal im Store (Final) vs. eigene Datei mit Hash-Kette (SOLL 4.10) | Eigene Datei `erasure/journal.log`, verschlüsselt, Hash-Kette, `fsync` je Eintrag (Abschnitt 8.5) | Abschnitt 8.5 |
| K-14 | Aufwand: 13–16 Wochen (Bericht 3) vs. 4–7 Monate Teilzeit (Final) | Eigene Schätzung in PLAN.md, mit Begründung | PLAN.md §6 |

---

## 4. Ausgangszustand (verifiziert)

### 4.1 Workspace und Build

| Punkt | Befund | Beleg |
|---|---|---|
| Mitglieder / Default | 37 / 27. Nicht in default: `privacy`, `bench`, `infer-candle`, `sandbox`, `py`, `audit-export`, `avv-generator`, `license`, `ingest`, `tests/arch` | [V] `Cargo.toml` |
| Toolchain | `1.89.0`, Komponenten `rustfmt`, `clippy` | [V] `rust-toolchain.toml` |
| `cargo check --workspace` | erfolgreich, 3 Warnungen (deprecated `search_filtered_at` im Engine-Hybrid-Query, ungenutzter Import in `ingest`, ungenutztes `PagedChunker`, ungenutzter `PyKeyError`) | [U] |
| `cargo check --workspace --all-targets` | 26 Fehler | [U] Audit `check_alle_targets` |
| `clippy` | Fehler in fast allen Crates, teils >300 pro Crate | [U] Audit `clippy_lib__*` |
| `fmt --check`, `doc` | FAIL | [U] Audit |
| `feature_matrix_hack` | Lauf unvollständig im Log | [U] |
| Audit-Befunde (Auszug) | undefinierte Features (1 Crate), `safe`-Funktionen mit unsafe-Inhalt (2 bekannte Stellen: `mmap_readonly`, `reopen_and_dup2`), 4 Produktivdateien mit `from_entropy` (K14), Default-Members unvollständig | [U] |

### 4.2 Sicherheitsbefunde (Kern, am Clone geprüft)

| ID | Befund | Beleg [V] | Status |
|---|---|---|---|
| **D-C02** | Snapshot-Sentinel: `prune_bound` fällt auf `u64::MAX`, `prune_through` maskiert Tombstone-Bit → `i64::MAX`, `fetch_max` macht es permanent. Danach scheitern SSI-Commits mit „snapshot too old“. | `store/src/lsm/ops/compaction.rs:225–231`; `mvcc/src/ssi.rs:646–652` (`bound_seq & !TOMBSTONE_BIT`, `fetch_max`) | **bestätigt, kritisch** |
| **WAL-Integrität abgeschaltet** | `#![allow(unexpected_cfgs)]` hebt das Workspace-`deny` auf. `wal-integrity` ist nirgends deklariert. Sieben Gates in `wal/replay.rs` (Zeilen 5, 10, 250, 332, 334, 434, 682; Zeile 682 als `not`-Zweig). | `store/src/lib.rs:45`; `store/src/wal/replay.rs`; `Cargo.toml:194` (`unexpected_cfgs = deny`) | **bestätigt, kritisch** |
| **Legacy-WAL akzeptiert** | `WalVersion::V1` (ohne HMAC) wird weiter verarbeitet, mit Hinweis auf öffentlichen Rückfallschlüssel | `store/src/wal/io.rs:43, 267, 284, 294, 415` | bestätigt, hoch |
| **Verschlüsselung Default aus** | `encryption_mode: None`; Store `encryption_passphrase: None` | `contextra/src/builder.rs:147`; `store/src/lsm/config.rs:89,111` | bestätigt, kritisch für Mission |
| **Tenant Default optional** | `#[default] Optional` | `engine/src/lib.rs:207–209` | bestätigt, hoch |
| **AutoExtraction Default an** | `Enabled`, solange Feature `auto-extraction-opt-out` fehlt. Feature in `engine` leer. | `types/src/types/domain/misc.rs:591–597` | bestätigt, hoch |
| **Argon2-Untergrenze** | `MIN_M_COST_KIB = 19456` | `crypto/src/kdf.rs:36` | bestätigt, hoch |
| **HKDF für Passphrase** | `try_new(passphrase, salt)` mit HKDF-SHA256 | `crypto/src/crypto.rs:88` | bestätigt, kritisch |
| **Lock-Bug Delete** | Lock wird vor der Operation freigegeben | `engine/src/collection/crud/delete.rs` | bestätigt, hoch (Race bei Parallel-Write) |
| **Drop ohne Shutdown** | `Drop for LsmStorage` ruft nur `cancel_token.cancel()`; Hintergrundtasks und Verzeichnislock überleben | `store/src/lsm/engine.rs:75–79` | bestätigt, mittel |
| **Follower-Commit nicht cancel-safe** | Timeout-Zweig vorhanden, zweites `rx.await` ohne Timeout; Cancel-Guard nur im Leader-Pfad | `store/src/lsm/ops/write.rs` (Timeout ca. Zeile 524, `rx.await` ca. Zeile 564, `LeaderCancelGuard` ca. Zeile 610) | bestätigt, kritisch (Teilbestätigung, Chaos-Test offen) |
| **Zwei `Vfs`-Traits; Store ohne Vfs** | `ports/src/vfs.rs:29`; `durable-fs/src/vfs.rs:54`; `Vfs` im Store: 0 Treffer | [V] | bestätigt, hoch |
| **Direkte Dateizugriffe im Store** | 83 Treffer `tokio::fs::`/`std::fs::` in 18 Produktionsdateien (grep, inkl. inline Tests) | [V] | bestätigt |
| **Purge ignoriert Ziel** | `LsmPurger::purge(&self, ctx, _target)`; Filter `scan_prefix` auf Mandant | `store/src/scanner/purge.rs:36,44` | bestätigt, kritisch (Löschpfad, entfällt mit Rückbau) |
| **Vorhandene Löschtypen** | `deletion_proof/` (7 Dateien), `kv_shredding.rs`, `kv_segment/`, `kv_cipher.rs`, `revocation_log.rs`, `engine/src/contextra_impl/erasure.rs` (616 Zeilen), `store/src/scanner/` | [V] Existenz | Rückbau (Abschnitt 5) |
| **SIMD u8** | `_mm512_dpbusd_epi32(acc, va, vb)` mit u8/u8; Instruktion erwartet u8×i8 | `simd/src/kernels/avx512.rs:445,565–569` | bestätigt (Muster), falsche Distanzen wahrscheinlich [U nicht getestet] |
| **DocId 64 Bit, unkeyed** | `DocId(pub u64)` ohne Feature `docid-128` | `types/src/types/domain/ids.rs:173` | bestätigt |
| **`expect` im Produktionspfad** | Tokio-Runtime-Erzeugung im Scanner | `store/src/scanner/sstable.rs:35,47` | bestätigt |
| **`HashMap`/`HashSet` mit RandomState** | Manifest-Bereinigung | `store/src/manifest/core.rs:284, 338–342` | bestätigt (P28, Ergebnisse sortiert) |
| **Undeklarierte Feature-Gates unentdeckt** | `feature-matrix` und `cargo-hack-feature-check` laufen mit `continue-on-error: true` | `.github/workflows/rust-ci.yml` (Jobs u. a. `feature-matrix`, `cargo-hack-feature-check`, `supply-chain`, `coverage`, `miri-*`, `asan-*`, `test-py`, `test-cross-platform`) | bestätigt, Ursache für Unentdeckbarkeit |
| **Fehlende Dokumentziele** | `docs/decisions/` fehlt; `docs/PLAN.md` fehlt; `engine/src/collections.rs` fehlt (von Löschdokumenten referenziert) | [V] | bestätigt, hoch (Doc-Drift) |
| **`cargo-deny` Ignorierungen** | 6 Advisories ignoriert (RUSTSEC-2024-0384, -2024-0436, -2026-0176, -2026-0177, -2026-0285, -2026-0205) ohne Ablaufdatum | `deny.toml:9–14` | bestätigt |

### 4.3 Befunde, die überholt sind

| ID | Quelle | Stand am Clone |
|---|---|---|
| F6 | Audit 2 (Build-Blocker `scanner`, `hnsw_purger`) | **überholt**: Exporte vorhanden |
| V-07 | Final (KV-Cache verdrahtet) | `kvcache` ist Workspace-Mitglied, aber nicht in `default-members`; Entfernung in WP-1.2 |
| F-19 | Bericht 3 (`default-members` ≠ `members`) | Unterschied besteht (10 Crates), Bericht veraltet |
| R-04 | Fehlerkatalog (SurrogateVault 16 Bit) | [K] nicht am Clone geprüft; mit `privacy`-Entfernung irrelevant für v1 |

### 4.4 Konsumentenanalyse (Scope-Grundlage)

| Crate | Konsumenten am Clone [V] | Folge |
|---|---|---|
| `contextra-wire` | core, mcp, py, router | entfällt mit MCP/Python/Router (D-25, D-29) |
| `contextra-core` | nur `benchmarks/contextra-bench` | entfällt |
| `contextra-graph` | engine (18 Dateien), cognition, db, agent, router, bench; **store dev-dep** | Relations-API und Graph entfernen (D-17); store-Dev-Abhängigkeit streichen |
| `contextra-checkpoint` | engine (lifecycle, diagnostics), db, agent | Engine-Hooks entfernen (D-16) |
| `contextra-rank` | engine (`fuse_*`, `IsotonicCalibrator`), db, contextra, mcp, py, infer-* | auf RRF reduzieren (D-26) |
| `contextra-privacy` | mcp, router, contextra | entfällt (Fassade entkoppeln) |
| `contextra-license` | mcp, contextra | entfällt (D-23) |
| `contextra-db` | agent, py, router, contextra, bench | prüfen: Inhalt vollständig in Engine/Fassade überführbar (WP-1.1) |
| `contextra-ingest` | nur sich selbst | entfällt, keine Verwender |
| `contextra-testkit` | viele, nur dev | bleibt als Dev-Abhängigkeit |
| `contextra-sys` | db, durable-fs, engine, store, vector, bench | bleibt (unsafe-Insel) |
| `contextra-simd` | simd, vector | bleibt (unsafe-Insel, Kernel-Reduktion D-28) |

Vollständige Entfernung wird durch `cargo tree -p contextra -e normal --prefix none | sort -u` nach jedem Schritt bestätigt (WP-1.1, Abnahme).

---

## 5. Scope v1

### 5.1 Crate-Entscheidungen

| Crate | Entscheidung | Begründung (Verweis) |
|---|---|---|
| `contextra-types` | **behalten**, Löschtypen ohne I/O ergänzen | Typfamilie je Konzept (A2). Keine `async`, keine I/O (T-ARCH-1). |
| `contextra-ports` | **behalten**, `Vfs` konsolidieren | D-04 |
| `contextra-sys` | **behalten** (unsafe-Insel) | fsync-Primitive, `flush_dir`, `rename_durable` |
| `contextra-simd` | **behalten, reduziert** | D-28 |
| `contextra-durable-fs` | **behalten**, Implementierung von `Vfs`; `scrub.rs` und `PendingDeleteJournal` nach Test prüfen | D-04 |
| `contextra-crypto` | **behalten, reduziert**: AEAD, KDF (Argon2id), Schlüsselhierarchie, WAL-HMAC, Hash-Kette, Ed25519-Report | entfernt: `deletion_proof/`, `ed25519_proof.rs`, `kv_*`, `revocation_log.rs` |
| `contextra-mvcc` | **behalten** | Zeitreisen, SSI. D-11, Sentinel-Fix (WP-2.1) |
| `contextra-store` | **behalten, gehärtet** | D-02. Entfernt: `scanner/` (alt), `kv/`. Neu: `erase/`. |
| `contextra-vector` | **behalten, reduziert** | D-18, D-19. Entfernt: `hnsw/deletion.rs`, `hnsw/purge.rs` (alt, ersetzt). |
| `contextra-text` | **behalten, umgebaut** | D-20. Entfernt: `inverted/purge.rs`, `scanner.rs` (alt). |
| `contextra-rank` | **behalten, reduziert auf RRF** | D-26 |
| `contextra-engine` | **behalten, umgebaut** | Saga, Journal, Gate, Subject-Index. Entfernt: `contextra_impl/erasure.rs`, `relate`, Checkpoint- und Graph-Hooks. |
| `contextra-db` | **prüfen (WP-1.1)**, Default: in Fassade überführen | Keine Eigenständigkeit im Kern belegt |
| `contextra` (Fassade) | **behalten** | öffentliche API (Abschnitt 6) |
| `contextra-testkit` | **behalten, dev-only** | Erasure Lab (Abschnitt 9.2) |
| `tests/arch` | **behalten** | Architekturtests T-ARCH-1/2 |
| `contextra-graph` | **entfernen** (Archiv) | D-17 |
| `contextra-checkpoint` | **entfernen** (Archiv) | D-16 |
| `contextra-privacy` | **entfernen** (Archiv) | D-24 |
| `contextra-ingest` | **entfernen** (Archiv) | D-24 |
| `contextra-cognition`, `contextra-adapt`, `contextra-agent`, `contextra-router` | **entfernen** (Archiv) | D-24 |
| `contextra-kvcache` | **entfernen** (Archiv) | D-13, Cache-Verbot |
| `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx` | **entfernen** (Archiv) | D-24 |
| `contextra-sandbox`, `contextra-audit-export`, `contextra-avv-generator` | **entfernen** (Archiv) | D-24 |
| `contextra-license` | **entfernen** (Archiv) | D-23 |
| `contextra-mcp` | **entfernen**, v1.1-Adapter | D-25 |
| `contextra-py` | **entfernen**, v1.1-Adapter | D-25 |
| `contextra-wire`, `contextra-core` | **entfernen** (Archiv) | Konsumentenanalyse 4.4 |
| `benchmarks/contextra-bench` | **entfernen aus Workspace** (Archiv), Store-Benchmarks bleiben in `store/benches` | Nicht kompilierbar [K: A-B06], misst Retrieval-Qualität, nicht den Store |

**Ergebnis:** 13 Produktions-Crates im Workspace (einschließlich `types`, `ports`, `sys`, `simd`, `durable-fs`, `crypto`, `mvcc`, `store`, `vector`, `text`, `rank`, `engine`, `contextra`) plus `testkit` (dev) und `tests/arch`. Gegenüber 37 Mitgliedern entspricht das einer Reduktion um ca. 60 %.

### 5.2 Ringe v1

| Ring | Crates | Regel |
|---|---|---|
| 0 | `types`, `ports` | keine I/O, keine `async` in `types` |
| 1 | `sys` [unsafe], `simd` [unsafe], `durable-fs`, `crypto` | Kein Abhängigkeitspfad nach oben |
| 2 | `mvcc`, `store`, `vector`, `text`, `rank` | `store` kennt weder `vector` noch `text` (LSM-Kern bleibt blind, Quelle Umsetzungsplan) |
| 3 | `engine` | Orchestrierung, Saga, Gate |
| 4 | `contextra` (Fassade) | einzige öffentliche Oberfläche |

Abhängigkeiten zeigen nur nach unten. Verletzung ist Build-Fehler (Architekturtest T-ARCH-3, neu).

### 5.3 Funktionsumfang v1

| Bereich | Funktion |
|---|---|
| Speichern | Collection, Insert, Update, Delete (logisch, mit Löschsemantik siehe 8), Get, Scan (Schlüsselreihenfolge) pro Mandant und Collection |
| Suche | BM25 (Volltext), HNSW (Vektor), RRF (Fusion). Vektoren vom Aufrufer. |
| Betroffene | `subject_id` optional beim Insert, Index `subject_tag → DocIds` |
| Zeitreisen | `get_at_seq`, `scan_at_seq` (Vorgabe) |
| Transaktionen | MVCC, SSI-Commits, atomare Batches |
| Löschung | `erase(EraseTarget)` für Dokument, Betroffener, Collection; `erasure_status`; `verify` |
| Nachweis | `ErasureReport` (signiert), Verifier-Beispiel mit Testvektoren |
| Checkliste | Textexport mit Prüfpunkten außerhalb von Contextra (Abschnitt 13.3) |
| Betrieb | Öffnen mit Passphrase, Recovery beim Start, `close()`, Metriken ohne IDs |

### 5.4 Budgets (CI-Gates)

| Budget | Wert | Messung |
|---|---|---|
| Erase-Produktcode (`store/src/erase`, `engine/src/erasure`, `crypto/src/proof`, `types/src/erasure`) | ≤ 2.500 Zeilen | `tokei` in CI |
| Gesamt-Produktcode Kern (ohne Tests) | Zielreduktion ≥ 40 % gegenüber Baseline WP-0.1 | `tokei` in CI, Baseline in `evidence/` |
| Clippy-Meldungen Kern | 0 | `cargo clippy --workspace --all-targets -- -D warnings` |

---

## 6. Öffentliche API (Rust)

Alle Typen mit `#[non_exhaustive]` außer Konfigurationsbausteinen. Keine öffentlichen `String`-Felder für Schlüssel oder Passphrasen.

```rust
// Öffnen. Pflichtfelder fehlen → Fehler (fail-closed), kein Standardwert.
pub struct EngineConfig { /* privat, über Builder */ }
pub struct EngineBuilder;
impl EngineBuilder {
    pub fn new(data_dir: impl AsRef<Path>) -> Self;
    pub fn passphrase(self, secret: SecretBytes) -> Self;      // Pflicht
    pub fn erase_sla(self, sla: Duration) -> Self;              // Pflicht, kein Standard
    pub fn erase_return_timeout(self, t: Duration) -> Self;     // Standard 60 s
    pub fn batch_window(self, w: Duration) -> Self;             // Standard 60 s
    pub async fn open(self, vfs: Arc<dyn Vfs>, clock: Arc<dyn Clock>) -> Result<Contextra, ContextraError>;
}

pub struct Contextra { /* … */ }
impl Contextra {
    pub async fn collection(&self, ctx: &TenantContext, name: &CollectionName) -> Result<Collection, ContextraError>;
    pub async fn erase(&self, ctx: &TenantContext, req: EraseRequest) -> Result<ErasureHandle, ContextraError>;
    pub async fn erasure_status(&self, id: ErasureId) -> Result<ErasureStatus, ContextraError>;
    pub async fn verify(&self, ctx: &TenantContext, target: &EraseTarget) -> Result<VerificationReport, ContextraError>;
    pub fn erasure_checklist(&self) -> String;
    pub async fn close(self) -> Result<(), ContextraError>;     // explizit, siehe 11.3
}

pub enum EraseTarget {
    Document { collection: CollectionName, doc: DocKey },
    Subject  { subject: SubjectId },
    Collection { collection: CollectionName },
}

pub struct EraseRequest {
    pub target: EraseTarget,
    pub retention: RetentionAttestation,   // Pflicht, D-15
}

pub enum RetentionAttestation {
    /// Keine Aufbewahrungspflicht bekannt. Der Betreiber bestätigt dies.
    NoObligationConfirmed,
    /// Aufbewahrungspflicht geprüft. Die Löschung wird verweigert, solange sie gilt.
    /// Der Code erfasst Gründe als Enum (keine Freitexte, D-15).
    ObligationDeclinesErasure { code: RetentionCode },
}

pub struct ErasureHandle { /* … */ }
impl ErasureHandle {
    pub fn id(&self) -> ErasureId;
    pub async fn status(&self) -> Result<ErasureStatus, ContextraError>;
    pub async fn wait(&self, timeout: Duration) -> Result<ErasureStatus, ContextraError>;
}

pub enum ErasureStatus {
    Gated,                                  // Sperre persistiert, Purge läuft
    Pending { attempt: u32 },               // Fehler in Phase, Wiederholung geplant
    Completed(ErasureReport),               // gemessen, Bericht signiert
    Refused { reason: RefusalReason },      // fail-closed, z. B. NoManifest
}
```

**Regeln:**

- `erase` gibt `Gated` zurück, sobald die Sperre persistiert ist (Latenzziel p50 ≤ 10 ms). Die Fertigstellung erfolgt asynchron.
- `ErasureStatus::Completed` nur nach Messung aller Pflichtschichten (Abschnitt 9).
- `ErasureStatus::Refused` ist terminal und ändert keinen Datenbestand (z. B. `NoManifest` bei Altbestand).
- Ohne `TenantContext` existiert kein Zugriff auf `Collection` (D-07).

### 6.1 Fehlertaxonomie

```rust
#[non_exhaustive]
pub enum ContextraError {
    Config(ConfigError),          // fehlende Pflichtangabe, zu schwache KDF-Parameter
    Auth(AuthError),              // falsche Passphrase, fehlender Mandant
    Integrity(IntegrityError),    // WAL-HMAC, AEAD, Manifest, Journal-Kette
    Io { op: IoOp, source: io::ErrorKind }, // ohne Pfad und ohne Inhalt
    Conflict(ConflictError),      // IdCollision, Schreibkonflikt SSI
    SnapshotRevoked { seq: u64 },
    Erased,                       // Ziel gesperrt (Gate)
    Unsupported(&'static str),
    Closed,
}
```

Fehlermeldungen enthalten **nie** Schlüssel, Inhalte, Nutzer-IDs oder Pfade mit personenbezogenen Namen (VETO-FAILCLOSED, T-LOG-1).

---

## 7. Datenmodell

### 7.1 Schlüsselraum

Jeder Schlüssel ist ein Byte-Array fester Struktur, big-endian, ohne Trennzeichen-Parsing:

```
key   = family:u8 ‖ tenant_tag:[u8;16] ‖ coll_tag:[u8;16] ‖ payload
family ∈ { Doc=0x01, Manifest=0x02, DocMap=0x03, Text=0x04, Df=0x05,
           Subject=0x06, Vec=0x07, Journal=0x08 (außerhalb LSM), Meta=0x09 }
```

| Familie | payload | Wert (verschlüsselt, AEAD) |
|---|---|---|
| `Doc` | `doc_id:[u8;16]` | Dokument-Inhalt, Metadaten, `doc_key` |
| `Manifest` | `doc_id` | Liste aller Schlüssel, die das Dokument besitzt (Abschnitt 7.3) |
| `DocMap` | `doc_id` | `slot:u32`, Version, Subject-Tag |
| `Text` | `term_tag:[u8;16] ‖ doc_id` | `tf:u32` |
| `Df` | `term_tag` | `df:u64` (Zähler) |
| `Subject` | `subject_tag:[u8;16] ‖ doc_id` | leer |
| `Vec` | `slot:u32` | Vektor-Metadaten (Vektorinhalt liegt in der Arena, D-19) |
| `Meta` | `name_tag` | Collection-Metadaten, Format-Version |

Alle `*_tag` sind `BLAKE3-keyed(K_idx, label ‖ Eingabe)[0..16]`, Labels fest definiert (Konstanten-Enum, kein Freitext).

### 7.2 Dokumentidentität

`doc_id = BLAKE3-keyed(K_idx, "ctx/doc/v1" ‖ tenant ‖ collection ‖ doc_key)[0..16]` (D-08).

Insert-Ablauf: Lese `DocMap[doc_id]` → existiert und `doc_key` (entschlüsselt) ≠ neuer Schlüssel → `IdCollision`. Gleich → Update. Der Vergleich erfolgt im Speicher mit `Zeroizing`-Puffern.

### 7.3 Manifest

Pro Dokument ein `Manifest`-Eintrag, in **derselben Transaktion** wie `Doc`, `DocMap`, `Text`, `Df`, `Subject`, `Vec`. Inhalt: Liste aller Schlüssel dieses Dokuments (exakte Bytes, verschlüsselt). Löschung liest das Manifest **vor** jeder Mutation.

- Fehlt das Manifest (Altbestand), antwortet `erase` mit `Refused(NoManifest)`. Es gibt keinen Vollscan-Fallback (P3). Migration: Abschnitt 15, Frage F-LEG.
- Update entfernt im selben Batch die Schlüssel alter Terme, die nicht mehr vorkommen, und aktualisiert das Manifest.

### 7.4 Text und BM25

- Posting `Text(term_tag, doc_id) → tf`.
- `Df(term_tag)` als Zähler, in derselben Transaktion wie die Postings aktualisiert.
- Suche: für jeden Query-Term Präfix-Scan auf `Text(term_tag, *)`. Dies ist der **einzige** erlaubte Präfix-Scan, weil `term_tag` fixe Länge hat (D-10).
- Aggregierte `Df`-Werte enthalten keine Nutzerinhalte, nur Zählwerte. Ob sie personenbezogen sind, ist Prüfpunkt G-LEGAL (Abschnitt 15).

### 7.5 Vektoren

- HNSW-Arena: Slots fester Größe, Vektor in f32. Slot-Zuordnung über `DocMap`.
- Löschung: Vektorbytes des Slots mit `zeroize` überschreiben, Slot in Tombstone-Liste, `DocMap` entfernt.
- Kanten enthalten nur `slot:u32` (keine Vektorinhalte). Bleiben bis zum Rebuild als Restposten (`GraphRebuildPending`, Abschnitt 9.4).
- Snapshot-Dateien: verschlüsselt (D-19), über `Vfs`, bei Löschung neu geschrieben.

### 7.6 Subject-Index

`Subject(subject_tag, doc_id)`. Löschung nach `EraseTarget::Subject` liest die Einträge, bildet die Dokumentliste und führt je Dokument die Dokumentlöschung aus (ein Batch, D-14).

---

## 8. Löschmodell

### 8.1 Begriffe

| Begriff | Definition |
|---|---|
| **Ziel** | Typisierter `EraseTarget`, aufgelöst zu einer Menge von `doc_id`s und einer Menge exakter Schlüssel (Manifeste) |
| **Gate** | Persistierte Sperre (`ErasedSet`-Eintrag). Ab hier liefert jeder Lesepfad `Erased`. |
| **Schicht** | Ort, an dem Ziel-Daten liegen können (Abschnitt 9.4) |
| **Zeuge** | Ergebnis eines unabhängigen Scanners für eine Schicht (`LayerWitness`) |
| **Klasse** | Aussagestufe des Berichts (Abschnitt 9.3) |

### 8.2 Zustandsmaschine

| Zustand | Dauerhafte Wirkung | Übergang | Bei Fehler |
|---|---|---|---|
| `Requested` | Journal-Eintrag mit Ziel-Commitment (verschlüsselt) | Gate setzen | `Pending` |
| `Gated` | `ErasedSet` enthält Commitment, Journal-Eintrag `gated` | Drain | `Pending` (Gate bleibt) |
| `Drained` | Memtables geflusht, versiegelte WAL-Dateien entfernt, Fehlen geprüft | Rewrite | `Pending` |
| `Rewritten` | betroffene SSTables neu, Manifest „ersetze alt durch neu“ committed | Unlink | `Pending` |
| `Unlinked` | alte SSTables entfernt, Fehlen geprüft | Index-Bereinigung (HNSW-Slots, Blockcache) | `Pending` |
| `IndexesCleaned` | Vektor-Zeroing, Blockcache-Eviction | Messung | `Pending` |
| `Verified` | Rohscan ohne Befund, Positivkontrolle gefunden | Bericht | `Pending` (Messung wiederholbar) |
| `Completed` | Bericht signiert, Journal auf Minimalform (ID, Zeit, Ergebnis) reduziert | terminal | – |

**Pending** ist ein Zustand mit `attempt` und `next_retry_at`. Er meldet nie Erfolg. Überschreitung von `erase_sla` erzeugt Alarm (Metrik und Audit-Eintrag), keinen Abbruch (SOLL 4.11).

**Refused** ist terminal und ändert nichts (`NoManifest`, `ObligationDeclinesErasure`, Ziel unbekannt ohne Fehler: `Completed` mit leerer Trefferliste, gemessen).

### 8.3 Gate und Lesepfade

- Das Gate ist der `ErasedSet`-Eintrag. Er wird **vor** `erase` zurückkehrt persistiert (fsync).
- `TenantScopedStorage` ist der einzige Zugang zum Store (T-ARCH-2). Alle Lesepfade prüfen den `ErasedSet`: `get`, `scan`, `get_at_seq`, `scan_at_seq`, Suche (Text, Vektor).
- Treffer liefert `Erased`. Zeitreise-Lesezugriffe werden nicht ausgenommen (D-11, D-12).
- Snapshots, die eine betroffene Sequenz pinnen, werden widerrufen (`SnapshotRevoked`). Widerruf ist ein Journal-Eintrag.
- `ErasedSet` wird beim Start vor Freigabe der Lesepfade geladen (`recover()`, Abschnitt 8.6).

### 8.4 Purge-Compaction

Modul: `contextra-store::erase`.

**Barrier.** Ein `RwLock` trennt Compaction (shared) und Purge (exklusiv pro Purge-Lauf). Reads und Writes sind nicht blockiert. Warten auf die Barrier ist mit Frist versehen; Überschreitung ist `Pending`.

**Drain.**
1. `rotate_and_seal` des aktiven WAL.
2. Flush aller Memtables (inkl. immutable).
3. Versiegelte WAL-Dateien, deren Inhalt vollständig in SSTables liegt, entfernen.
4. Fehlen jeder Datei per `Vfs::exists` prüfen. Eine verbleibende Datei ist `Pending`, nie Warnung.

**Kandidaten.** SSTables, deren Schlüsselbereich mindestens einen Matcher-Schlüssel enthält. Bloom-Filter als Vorfilter. Exakte Bestätigung per Stream.

**Streamender Rewrite.**
1. Builder aus dem Quell-Reader ableiten (`create_like`): gleicher Schlüsselkontext, gleiche Formatversion, Schlüsselkontext aus dem Store-Schlüssel.
2. Iteration über alle Einträge in Schlüsselreihenfolge. Einträge mit Schlüssel ∈ Matcher werden **alle Versionen und Tombstones** verworfen (D-11).
3. Übrige Einträge mit Originalwerten `seq`, `tx_id`, `flags` übernehmen. Keine Neunummerierung (behebt N1).
4. Kein Vollladen. Speicherbedarf ≤ Konstante × Blockgröße.
5. Schreibfehler: Abbruch. Alter Reader bleibt gültig. Temp-Datei wird als Waise markiert und beim Start entfernt.

**Atomarer Tausch.**
1. `tmp` schreiben, `fsync` Datei.
2. Manifest-Eintrag „ersetze {alt} durch {neu}“ anhängen, `fsync` Manifest.
3. `fsync` Verzeichnis.
4. Reader tauschen.
5. Alte Datei entfernen (`Unlink`), danach `fsync` Verzeichnis und Fehlen prüfen.

**Blockcache.** Einträge ersetzter Dateien werden entfernt und mit `zeroize` überschrieben, vor dem Unlink.

**Matcher.** `Matcher = Set<[u8]>` exakter Schlüssel (D-10). Keine Präfixe, keine Stringvergleiche.

### 8.5 Invarianten

| ID | Invariante | Testbeleg |
|---|---|---|
| I-1 | Jede Zustandsänderung wird vor der Aktion im Journal mit `fsync` festgehalten. | T-CRASH-1 |
| I-2 | Rewrite übernimmt `seq`, `tx_id`, `flags` unverändert. | T-VER-1 |
| I-3 | Alter Reader bleibt bis zum Manifest-Commit gültig. | T-CRASH-3 |
| I-4 | Unlink nur nach Manifest-`fsync` und Verzeichnis-`fsync`. | T-CRASH-1 |
| I-5 | Kein Gate ohne persistierten Eintrag; kein Purge ohne Gate. | T-CONC-1 |
| I-6 | Nachbar-Daten bitgleich (NeighborDigest). | T-ISO-1 |
| I-7 | `Completed` nur mit Zeugen aller Pflichtschichten. | T-PROOF-1, T-RAW-1 |
| I-8 | Kein Fehler wird verschluckt. Jede Fallible-Operation liefert `Result`, das propagiert oder als `Pending` persistiert wird. | Lint T-LINT-2 |

### 8.6 Journal und Dateien

Pfad: `<data>/erasure/`.

| Datei | Inhalt | Lebenszyklus |
|---|---|---|
| `journal.log` | Append-only. Pro Eintrag: Länge, Typ, Nutzlast (verschlüsselt), CRC, Hash der Vorgängerkette. `fsync` je Eintrag. | Nach `Completed` auf Minimalform reduziert (ID, Zeit, Ergebnis, Commitment). Keine Zielkennung mehr. |
| `scope/{erasure_id}` | Auflösung des Ziels (Dokument-IDs, Schlüsselliste), verschlüsselt mit `K_journal` | Nach `Completed` entfernt, Fehlen geprüft |
| `erased.set` | sortierte Commitments mit `erased_through_seq`. Atomar ersetzt (Temp, `fsync`, Rename, Verzeichnis-`fsync`). | dauerhaft (Commitment, kein Klartext) |
| `keys/` | Verzeichnis 0700, Dateien 0600 | Start verweigert bei lockereren Rechten (P2) |
| `report/` | signierte Berichte (`ctx-erasure-report/1`) | dauerhaft, enthält keinen Personenbezug |

**Commitments.** `commitment = BLAKE3-keyed(K_commit_epoch, canonical(target))`. `K_commit_epoch` wird pro Epoche rotiert. Der Epochen-Schlüssel liegt im Schlüsselverzeichnis. Lebensdauer ≥ `erased_set_retention` (Frage F-COMMIT).

### 8.7 Wiederanlauf `recover()`

Reihenfolge, vor Freigabe jedes Lesepfads:

1. Dateisystem-Probe: atomares Rename, Verzeichnis-`fsync`. `EIO` → Start verweigert.
2. Schlüsselverzeichnis prüfen (Rechte, Vorhandensein). Fehlt ein Schlüssel → Start verweigert.
3. `erased.set` laden, Gate scharf schalten.
4. Journal lesen, Hash-Kette prüfen. Defekte Kette → Start verweigert (P2).
5. Waisen entfernen (`.tmp`, Ausgabedateien ohne Manifest-Eintrag).
6. Offene Sagas in ihrem Zustand fortsetzen (Idempotenz je Schritt).
7. Erst danach Lesen und Schreiben freigeben.

Unklarer Zustand heißt: Lesen verweigern (P2, INV-FAILCLOSED-1).

### 8.8 Nebenläufigkeit

- Pro Dokument ein Lock, derselbe für Writes und Erase (behebt N7).
- Gate-Setzen und logische Sperre laufen unter diesem Lock.
- Batch-Fenster (Standard 60 s): Aufträge werden zu einem Purge-Lauf zusammengefasst, dedupliziert nach Commitment. `Timing::Now` erzwingt sofortigen Lauf und weist die Zusatzkosten im Bericht aus.
- Rewrite-Kosten skalieren mit der Zahl betroffener SSTables, nicht mit der Gesamtdatenmenge (Gate, Abschnitt 12.4).

### 8.9 Cancel-Safety

Alle mutierenden Operationen laufen in einer vom Aufrufer unabhängigen Task (`tokio::spawn` auf dem Engine-Runtime-Handle). Der Aufrufer hält ein Handle auf das Ergebnis. Abbruch des Aufruferfutures ändert den Ausgang nicht und lässt keinen Zustand halb zurück (behebt D-C01). Ein Test bricht jeden `await`-Punkt ab (T-CANCEL-1).

---

## 9. Nachweis und Messung

### 9.1 Löschbericht `ctx-erasure-report/1`

Kanonisches JSON (Schlüssel sortiert, UTF-8, keine Gleitkommazahlen). Felder:

| Feld | Inhalt |
|---|---|
| `report_id`, `erasure_id` | 128 Bit, OsRng |
| `scope_kind` | `document` \| `subject` \| `collection` |
| `subject_commitment` | Commitment, keine Kennung |
| `retention` | `NoObligationConfirmed` \| `ObligationDeclinesErasure` mit Code (nur bei Ablehnung) |
| `times` | `requested_at`, `gated_at`, `drained_at`, `rewritten_at`, `measured_at` (aus `Clock`) |
| `layers[]` | je Schicht: `layer_id`, `status`, `witness_digest` (BLAKE3 über Scan-Protokoll: Dateizahl, Bytezahl, Needle-Kategorien, `positive_control_found`), `encodings_searched` |
| `store_counters` | Dateien neu geschrieben, Bytes geschrieben, WAL-Segmente entfernt, widerrufene Snapshots (nur Zahlen) |
| `residual_risks[]` | Liste aus 9.4 |
| `provenance` | Contextra-Version, Build-ID, Digest der effektiven Konfiguration (ohne Geheimnisse) |
| `class` | aus `classify` (9.3) |
| `signature` | Ed25519 über `canonical(report ohne signature)` |

Der Bericht enthält nie Dokumentinhalt, nie Schlüssel, nie `doc_id` im Klartext, nie `subject_id`.

### 9.2 Messverfahren

| Komponente | Zweck | Unabhängigkeit |
|---|---|---|
| `ForensicScanner` | Rohscan aller Dateien im Datenverzeichnis nach Needles (UTF-8, UTF-16LE, Base64, Hex), je Datei und Offset | kein SSTable- oder WAL-Parser, nur Bytes |
| `HeapScanner` (Linux) | `/proc/self/mem` nach Needles | eigener Code |
| `CanaryFactory` | hochentropische Needles pro Lauf (Seed protokolliert) | nur Testmandant |
| Needle-Quellen | Dokumentinhalt, `doc_key`, abgeleitete Blind-Schlüssel (`Text`, `DocMap`, `Subject`, `Manifest`) | Needles nur im Speicher, `zeroize` nach Scan |
| `FaultVfs` | Absturz nach Operation *n*, abgerissener Write an Byte *b*, `ENOSPC`, `EIO` bei `fsync` | Testkit |
| `ReferenceModel` | In-Memory-Orakel (BTreeMap) | Testkit |
| `RecordingStorage` | protokolliert jeden geschriebenen Schlüssel je öffentlicher Operation | Testkit |
| `NeighborDigest` | Hash aller Nicht-Ziel-Paare über alle Dateien und Versionen | Testkit |
| `LeakyPurger` | absichtlich fehlerhafter Purger | Selbsttest (T-RAW-2) |

**Positivkontrolle.** Vor jedem Messlauf wird ein Canary im Testmandanten geschrieben und gefunden. Nicht gefunden → Lauf ungültig, kein Befund. Der Canary liegt nie im produktiven Mandanten (C-L20 [K]).

### 9.3 Klassen und `classify`

| Klasse | Bedingung | Aussage im Bericht |
|---|---|---|
| `GateOnly` | Sperre persistiert, Purge nicht abgeschlossen oder nicht gemessen | „Für Lesezugriffe gesperrt. Physische Entfernung nicht abgeschlossen.“ |
| `PhysicalMeasured` | alle Pflichtschichten mit gültigem Zeugen ohne Befund | „Im Datenverzeichnis und in den gemessenen Schichten nicht nachweisbar. Restrisiken laut Liste.“ |
| `Partial(Vec<LayerId>)` | Pflichtschicht ohne gültigen Zeugen | Liste der nicht gemessenen Schichten |

`classify` ist eine reine Funktion `Evidence → Class`. **Monoton:** fehlt ein Zeuge, sinkt die Klasse, sie steigt nie. Es gibt keine Klasse „kryptografisch gelöscht“ oder „garantiert gelöscht“. Die Verbotsliste (13.2) wird über `Display` der Klassen erzwungen.

### 9.4 Schichten und Restrisiken

**Schichten (`LayerId`, exhaustiver Enum):**

| LayerId | Inhalt | Mechanismus | Messung |
|---|---|---|---|
| `WalSealed` | versiegelte WAL-Dateien | Drain, Entfernen | Dateiliste, Rohscan |
| `WalActive` | aktive WAL-Datei | Rotate, Drain | Dateiliste, Rohscan |
| `Memtable` | Memtable (alle Versionen) | Flush, Schlüsselentfernung | Heap-Scan, Rohscan nach Flush |
| `Sstable` | SSTable-Daten und -Index | Rewrite | Rohscan |
| `Manifest` | Manifest-Logs | Austausch | Rohscan |
| `TextPostings` | `Text`, `Df` | Purge über Matcher | Rohscan |
| `Vector` | Arena, Snapshots | Zeroing, Snapshot-Rewrite | Rohscan, HeapScan |
| `SubjectIndex` | `Subject` | Purge | Rohscan |
| `BlockCache` | entschlüsselte Blöcke | Eviction, zeroize | Heap-Scan |
| `Journal` | Journal | Reduktion nach `Completed` | Rohscan (nach Abschluss ohne Zielbezug) |
| `Snapshots` | MVCC-Versionen, Snapshot-Register | Widerruf, Purge | Versionsscan je Sequenz |

**Restrisiken (Pflichtkatalog, jeder Bericht listet alle zutreffenden):**

| Risiko | Gilt wenn |
|---|---|
| `FlashRemanence` | Speichermedium Flash/SSD |
| `FsJournalOrCow` | Dateisystem mit Journal oder Copy-on-Write |
| `ProcessMemory` | Heap, Swap, Coredump möglich |
| `PageCacheTransient` | Seitencache des Betriebssystems bis zur Eviction |
| `ExternalCopies` | immer (Backups, Snapshots, Kopien beim Betreiber) |
| `ExternalProcessors` | Embeddings oder LLMs außerhalb von Contextra |
| `PseudonymCommitment` | solange `K_commit`-Epoche existiert |
| `ReplicaWindow` | Zeitfenster zwischen Gate und Abschluss |
| `UnmeasuredLayer(id)` | Schicht ohne gültigen Zeugen |
| `GraphRebuildPending` | Vektor-Tombstones bis Batch-Rebuild |

Ein Bericht ohne Restrisikoliste ist ungültig (T-REPORT-1).

### 9.5 Verifikation (`verify`)

`verify(ctx, target)` läuft **in einem neuen Prozess**, ohne Zugriff auf Inhalte:

1. Ziel-Identität wird vom Aufrufer übergeben (`doc_key` bzw. `subject_id`). Daraus werden `doc_id`, Blind-Schlüssel und Commitment abgeleitet.
2. `ErasedSet`-Prüfung: Commitment vorhanden.
3. Rohscan nach Blind-Schlüsseln und `doc_key`-Needles.
4. Ergebnis als `VerificationReport` (gleiche Klassen-Logik).

**Grenze (ausdrücklich):** Inhalts-Needles sind nur beim Erase verfügbar. `verify` prüft Schlüssel-Reste und Identitätsreste, nicht Inhaltsreste. Diese Grenze steht im Bericht.

### 9.6 Verifier (extern)

`examples/verify_erasure_report`: prüft Signatur, Feldschema (unbekannte Felder abgelehnt), Klassenableitung aus den Zeugen. Ohne Zugriff auf Contextra-Interna. Testvektoren im Repo. T-PROOF-1 ändert jedes Byte und verlangt Ablehnung.

---

## 10. Sicherheitsbasis

### 10.1 Schlüsselhierarchie

```
Passphrase ──Argon2id(m≥64MiB, t≥3, p≥1, salt[32])──▶ K_master (32 B)
K_master ──HKDF-SHA256(label)──▶ K_enc      (AEAD für Blöcke, Werte, Journal)
                              ▶ K_idx       (BLAKE3-keyed für doc_id, Tags)
                              ▶ K_wal       (HMAC-Kette WAL)
                              ▶ K_commit[e] (Commitments, je Epoche)
                              ▶ K_sign      (Ed25519-Seed, Berichte)
```

- Labels sind Konstanten (Enum). Keine Ableitung aus Freitext.
- Salt und Argon2-Parameter stehen im Header (Format-Version, KDF-ID). Parameter unterhalb der Untergrenze → `Config`-Fehler beim Öffnen.
- `K_master` lebt in `Zeroizing<[u8;32]>`, nicht kopierbar, nicht `Debug`.

### 10.2 AEAD

- AES-256-GCM-SIV (`aes-gcm-siv`, im Lock [V]).
- Nonce: 96 Bit, OsRng, je Datensatz. Limit je Schlüssel ≤ 2³² Datensätze, durchgesetzt über Zähler im Header. Überschreitung → Fehler (Rotation in v1.1).
- AAD: `family ‖ tenant_tag ‖ coll_tag ‖ format_version`. Vertauschung zwischen Familien oder Mandanten schlägt fehl (T-ENC-2).

### 10.3 WAL

- HMAC-Kette über jeden Frame, Sequenzprüfung, CRC zur Erkennung von Torn-Writes.
- Keine Legacy-Formate. `WalVersion::V1` ist Fehler `Integrity::LegacyFormat` (D-05, R-SEC-6).
- Am Dateiende: CRC-Mismatch ist Torn-Write → Abschneiden bis zum letzten gültigen Frame, protokollieren. Mismatch mitten in der Datei → `Integrity`-Fehler, kein Abschneiden (P2).
- Eine einzige Replay-Implementierung (Abschnitt 11.5, behebt N6).

### 10.4 Geheimnisse

- Passphrase: Typ `SecretBytes` (`Zeroizing<Vec<u8>>`), kein `Clone`, kein `Debug`-Ausdruck, kein `Serialize`. Keine `String`-Kopie.
- Schlüssel im Speicher: `Zeroizing`. Heap-Scan-Test T-SEC-2 prüft Entfernung nach Drop.
- `mlock` für Schlüsselpuffer best-effort. Fehlschlag wird im Bericht als `ProcessMemory` ausgewiesen, nicht verschwiegen.

### 10.5 Dateirechte und Start

- Datenverzeichnis: 0700. Schlüsseldateien: 0600. Start verweigert bei abweichenden Rechten (außer `root`-Test-Ausnahme unter `cfg(test)`; Befund K-Prüfung: Tests als Nicht-root).
- Dateisystem-Probe beim Start (8.7).

### 10.6 Lieferkette

- `cargo deny check` und `cargo audit` sind **blockierend**.
- Jede Advisory-Ignorierung benötigt Ablaufdatum und ADR. Die sechs heutigen Ignorierungen [V] werden in WP-6.5 geprüft.
- `Cargo.lock` mit `--locked` in CI.
- Pinning des Toolchains auf `1.89.0`.

---

## 11. Robustheit und Fehlerverhalten

### 11.1 Grundsatz

Jede Fehlerstelle hat genau eine von drei Reaktionen:
1. **Propagieren** als typisierter Fehler.
2. **Persistieren** als `Pending` mit Wiederholung (nur Löschpfad).
3. **Verweigern** (fail-closed) mit Fehlerzustand.

Verboten: `let _ =` auf `Result`, `unwrap_or_default` auf I/O, `.ok()` ohne Begründung, `unwrap`, `expect`, `panic!`, `unreachable!` im Produktionscode.

### 11.2 Lint-Durchsetzung

Workspace-Lints (blockierend in CI):

```toml
[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
indexing_slicing = "deny"
todo = "deny"
unimplemented = "deny"
let_underscore_must_use = "deny"   # neu: verbietet let _ = auf #[must_use]
undocumented_unsafe_blocks = "deny"

[workspace.lints.rust]
unsafe_code = "forbid"            # Ausnahme: simd, sys (eigene Lint-Tabelle)
unexpected_cfgs = "deny"          # ohne #![allow] in irgendeinem Crate
```

`clippy.toml`: `allow-unwrap-in-tests`, `allow-expect-in-tests`, `allow-panic-in-tests`, `allow-indexing-slicing-in-tests` = `true`. Außerhalb von `#[cfg(test)]` gilt das Verbot ohne Ausnahme. Ein `#[allow(clippy::…)]` im Produktionscode ist ein CI-Fehler, außer er ist in `governance/allow-ledger.toml` mit ADR und Ablaufdatum geführt.

### 11.3 Lebenszyklus

- `close(self)` ist der definierte Pfad: Hintergrundtasks beenden, Journal schließen, Verzeichnis-Lock lösen, Ergebnis zurückgeben.
- `Drop` ist best-effort und **blockiert nie**: setzt Stop-Flag, `cancel_token.cancel()`, gibt Locks frei, meldet Fehler per Tracing (behebt D-C08). Kein `block_on` in `Drop`.
- Test: Öffnen, Schreiben, Drop, Neuöffnen im selben Prozess ohne Race (Regressionstest zu D-C08).

### 11.4 Konsistenz bei Schreibvorgängen

- Ein Insert legt `Doc`, `DocMap`, `Manifest`, `Text`, `Df`, `Subject`, `Vec` in **einer** Transaktion an.
- Ein Dokument ohne vollständiges Manifest existiert nicht für Löschung. Beim Start werden Teilinserts (kein Commit) per WAL-Replay verworfen, nicht repariert.
- Commit-Reihenfolge: WAL → Memtable → Sichtbarkeit (wie heute, Sequenzsemantik nach WP-2.2).

### 11.5 WAL-Replay

Eine Implementierung. Die heutige Doppelung (`wal/io.rs`, `wal/replay.rs`) wird auf eine reduziert. Paritätstest T-WAL-2 prüft Äquivalenz gegen die Referenz, bis die Doppelung entfernt ist, danach entfällt der Test.

Ursache des CRC-Fehlers bei Offset 4 im verschlüsselten Fall ist **unbekannt** [K, D-C03/D-C04]. Zeitbox in WP-2.4, danach Entscheidung nach Gate G0 (3.2).

### 11.6 Ressourcengrenzen

| Grenze | Wert (Standard, konfigurierbar) |
|---|---|
| Maximale Dokumentgröße | 4 MiB |
| Maximale Chunk-Länge Text | 64 KiB |
| Vektordimension | 16–4096 (fest pro Collection) |
| Maximale Batchgröße Insert | 1.000 Dokumente |
| Maximale Löschbatchgröße | 10.000 Dokumente pro Lauf |
| Speicherbudget Rewrite | Konstante × Blockgröße (Gate 12.4) |
| `erase_return_timeout` | 60 s |

Überschreitung → `Config`- oder `Conflict`-Fehler, nie stilles Kürzen.

### 11.7 Beobachtbarkeit

- Metriken als Zähler und Histogramme ohne Labels mit IDs, Nutzerdaten oder Pfaden.
- Tracing auf Level `info` ohne Inhalte. `debug` darf keine Schlüssel oder Inhalte enthalten (Lint T-LOG-1 über `tracing`-Makros).
- Audit-Kette (`crypto/audit_chain.rs`) für Zustandsänderungen der Löschung, nur Commitments und Zeiten.

---

## 12. Qualitätssicherung

### 12.1 Blockierende CI-Jobs

Alle Jobs mit `continue-on-error` werden in WP-0.2 geprüft. Für v1 blockierend:

| Job | Prüfung |
|---|---|
| `fmt` | `cargo fmt --all -- --check` |
| `check` | `cargo check --workspace --all-targets --locked` |
| `clippy` | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| `test` | `cargo test --workspace --locked` (sequenziell für Crash-Tests) |
| `feature-check` | `cargo hack check --each-feature` plus Test auf undeklarierte Features (ohne `allow`) |
| `deny` | `cargo deny check` |
| `audit` | `cargo audit` |
| `arch` | `tests/arch` (T-ARCH-1..3, T-LINT-1..2) |
| `wording` | Wortlaut-Lint (13.2) über Quelle, Doku, Berichtsvorlagen |
| `policy` | Zählung Panics, `let _`, `allow`, `std::fs`, `HashMap` im Produktionscode auf Obergrenze gemäß `governance/ratchet.toml` (Zähler nur sinkend) |

Nicht blockierend (Tiefenprüfungen): `miri`, `asan`, `coverage`, Fuzz-Dauerlauf, Benchmarks (eigener Lauf mit Basislinie).

### 12.2 Pflicht-Testmatrix

| ID | Prüft | Typ |
|---|---|---|
| T-ISO-1 | Löschung von A lässt `NeighborDigest` bitgleich (andere Collection, anderer Mandant) | Property, ≥ 10⁴ Seeds |
| T-ISO-2 | Nutzerschlüssel als Präfix voneinander (`alice`/`alice2`, `1`/`10`) | gezielt |
| T-ISO-3 | erzwungene `doc_id`-Kollision: Insert abgelehnt, Löschung trifft nur das Ziel | gezielt mit Test-Hook |
| T-VER-1 | Nach Erase liefern alle Lesepfade `Erased` oder `SnapshotRevoked`: `get`, `scan`, `get_at_seq` über **alle** Sequenzen, `scan_at_seq`, Suche | exhaustiv |
| T-RAW-1 | Rohscan nach jedem Saga-Zustand, Abschluss ohne Befund | Rohscan |
| T-RAW-2 | `LeakyPurger` wird gefunden, sauberes System besteht | Selbsttest |
| T-RAW-3 | HeapScan nach Vektor-, Text- und Cache-Löschung | Rohscan RAM |
| T-CRASH-1 | Absturz nach **jeder** Vfs-Operation der Saga; nach `recover()` terminal oder `Pending`, nie Lesezugriff auf `Requested`/`Gated`-Scope außer `Erased` | exhaustiv, enumeriert |
| T-CRASH-2 | abgerissener Write an jedem Byte von Journal, Manifest, `erased.set` | exhaustiv |
| T-CRASH-3 | `ENOSPC` und `EIO` bei `fsync` im Rewrite → `Pending`, Nachbarn unverändert | gezielt |
| T-CRASH-4 | teilindexiertes Dokument nach Absturz verworfen | gezielt |
| T-CANCEL-1 | Abbruch des Aufruferfutures an jedem `await` (Follower-Pfad D-C01) | exhaustiv |
| T-CONC-1 | Erase parallel zu Put desselben Dokuments; mit offenem Snapshot; Barrier gegen Compaction | loom, Stress |
| T-DIFF-1 | Zufallsfolgen (Insert, Update, Flush, Compaction, Restart, Erase) gegen `ReferenceModel` | Differential, ≥ 10⁶ Operationen |
| T-WAL-1 | Abschneiden an jedem Byte-Offset, verschlüsselt und plain: Open gelingt, Präfix wiederhergestellt | exhaustiv |
| T-WAL-2 | Parität der Replay-Pfade bis Entfernen der Doppelung | Differential |
| T-SEQ-1 | Sequenzsemantik (Sichtbarkeit über `TxEnd`) gegen Referenzmodell | Property |
| T-COV-1 | jeder Schlüssel jeder öffentlichen Operation ist im Katalog der Schichten und im Manifest | RecordingStorage |
| T-ENC-1 | neue SSTable mit regulärem Reader lesbar, gleiche Verschlüsselung | Regression |
| T-ENC-2 | Wert unter Familie/Mandant A mit Schlüsselkontext B entschlüsselt nicht | Sicherheit |
| T-KDF-1 | Parameter unterhalb Untergrenze → `Config`-Fehler | gezielt |
| T-DEF-1 | Standard-Builder erzeugt keine Klartextdatei (Rohscan), Öffnen ohne Passphrase schlägt fehl | gezielt |
| T-SEC-2 | Passphrase und `K_master` nach Drop nicht im Heap (HeapScan) | gezielt |
| T-SLA-1 | `erase_sla` fehlt → Builder-Fehler; Überschreitung → Alarm, kein Abbruch | gezielt |
| T-API-1 | Ohne `TenantContext` kein `Collection` | Compile-Fail-Test (trybuild) |
| T-API-2 | `RetentionAttestation` ist Pflicht; `ObligationDeclinesErasure` verhindert Erase | gezielt |
| T-REPORT-1 | Bericht enthält Restrisikoliste, keinen Klartext (Scan des Berichts) | Property |
| T-PROOF-1 | Änderung **jedes** Bytes des Berichts wird abgelehnt; Testvektoren für externen Verifier | Mutation |
| T-LINT-1 | Wortlaut-Lint: keine verbotenen Begriffe (13.2) | Lint |
| T-LINT-2 | kein `std::fs`, `tokio::fs`, `unwrap`, `expect`, `let _ =` in Erase-Modulen | Lint |
| T-LOG-1 | Log-Ausgaben enthalten keine Needles (Canary-Scan über Tracing-Output) | Property |
| T-ARCH-1 | `types` ohne `async` und ohne I/O | Architektur |
| T-ARCH-2 | nur `TenantScopedStorage` ruft den rohen Store auf | Architektur |
| T-ARCH-3 | Ring-Abhängigkeiten nur nach unten | Architektur |
| T-DROP-1 | Drop ohne `close()` blockiert nicht, Neuöffnen im selben Prozess gelingt | gezielt |
| T-SIMD-1 | jeder SIMD-Kernel gleich Skalar (Zufallsvektoren, alle CPU-Dispatch-Pfade) | Property |
| T-DOCMAP-1 | Zuordnung `doc_id ↔ slot` konsistent nach Erase und Rebuild | Property |

### 12.3 Werkzeuge

| Werkzeug | Ziel |
|---|---|
| `loom` | Barrier, ReadGate, Journal, Gate-Set unter Nebenläufigkeit |
| `cargo-fuzz` | Journal-Parser, SSTable-Index, WAL-Frame, Schlüsselcodec, Report-Parser |
| `cargo-mutants` | Saga-Übergänge, `classify`, Matcher, Barrier. Gate ≥ 95 % der Mutanten getötet. |
| `miri` | Simd-Kernel-Inseln (nicht blockierend, CPU-Pfad) |
| `proptest` | T-ISO, T-DIFF, T-SEQ, T-SIMD |

### 12.4 Leistungsziele

Basislinie in WP-6.1 auf dokumentierter Hardware (CPU-Modell, RAM, Speicher, Kernel, Dateisystem, Mount-Optionen). Gates relativ zur Basislinie.

| Messgröße | Methode | Gate |
|---|---|---|
| Erase-Latenz bis `Gated` (p50/p99) | 10⁴, 10⁶ Dokumente | p50 ≤ 10 ms, p99 ≤ 50 ms |
| Erase bis `Completed` (Dokument) | 10⁴, 10⁶, 10⁷ Dokumente | Anstieg proportional zu betroffenen SSTables, nicht zur Gesamtmenge |
| Batch-Erase 1.000 Dokumente | Durchsatz, Anzahl Rewrites | ausgewiesen, Basislinie + 10 % |
| Schreibverstärkung Purge | geschriebene Bytes je gelöschtem Byte | ≤ Basislinie + 5 %, Zielwert je nach Messung festgelegt |
| RSS-Spitze beim Rewrite | während Rewrite | ≤ Konstante × Blockgröße |
| Lese-Overhead `ErasedSet`-Prüfung | `get`, `scan` mit/ohne | ≤ 3 % p50 |
| Insert/Get/Scan/Suche | `lsm_concurrency_bench`, `wal_bench`, `sstable_bench`, `redb_comparative_bench` | Regression ≤ 5 % gegen Basislinie |
| Suche Hybrid (BM25 + HNSW + RRF) | festes Testkorpus | Recall@10 und p95-Latenz, Basislinie |

Benchmarks laufen mit fixen Parametern, mehrfach, mit Standardabweichung. Ein Lauf ohne Basislinie zählt nicht.

---

## 13. Compliance-Leitplanken

### 13.1 Anforderungsmatrix

Die Anforderungen sind Arbeitsgrundlage für G-LEGAL. Artikelangaben sind Orientierung.

| # | Anforderung (Orientierung) | Mechanismus | Test | Restrisiko |
|---|---|---|---|---|
| R1 | Unverzügliche Löschung, Fristen (Art. 17, 12 Abs. 3) | Saga mit Wiederholung, `erase_sla`, Status | T-CRASH-*, T-SLA-1 | Zeitfenster `ReplicaWindow` |
| R2 | Unkenntlichmachen (SDM-Baustein 60) | Physische Entfernung aller Schichten, Messung | T-RAW-1/3 | `FlashRemanence`, `FsJournalOrCow`, `ProcessMemory` |
| R3 | Einfache Löschbefehle genügen nicht | Rewrite, Entfernen, Fehlen geprüft | T-CRASH-1, T-RAW-1 | wie R2 |
| R4 | Alle Versionen und Zeitreisen | Purge aller Versionen, Snapshot-Widerruf | T-VER-1 | – |
| R5 | Rechenschaft (Art. 5 Abs. 2) | signierter Bericht, Audit-Kette ohne Klartext | T-PROOF-1, T-REPORT-1 | `PseudonymCommitment` |
| R6 | Datenschutz durch Technikgestaltung (Art. 25) | Verschlüsselung ohne Opt-out, Mandantenpflicht, keine Extraktion im Default | T-DEF-1, T-API-1 | – |
| R7 | Sicherheit der Verarbeitung (Art. 32) | Argon2id, WAL-HMAC, AEAD mit AAD | T-KDF-1, T-WAL-1, T-ENC-2 | Schlüsselkompromittierung |
| R8 | Information von Empfängern (Art. 17 Abs. 2, Art. 19) | Bericht führt `ExternalProcessors`, `ExternalCopies`; Hook in v1.1 | – | Verantwortung Betreiber |
| R9 | Aufbewahrungspflichten (Art. 17 Abs. 3) | `RetentionAttestation`, kein Scheduler | T-API-2 | Prüfung beim Betreiber |
| R10 | Embeddings vorsorglich personenbezogen [D] | Vektoren im Löschumfang, Snapshots verschlüsselt | T-RAW-3 | Rekonstruktion aus Resten im RAM |
| R11 | Keine „Einschränkung statt Löschung“ in v1 | entfällt | – | – |
| R12 | Werbeaussagen belegpflichtig (§ 5 UWG) [D] | Wortlaut-Lint, Aussagen nur aus Berichtsklasse | T-LINT-1 | – |
| R13 | Backups | außerhalb; im Bericht ausgewiesen | – | Betreiber der Sicherung |

### 13.2 Außenaussagen

**Zulässig:**

> Contextra entfernt die Daten einer betroffenen Person aus dem Datenverzeichnis, den Indexen und den gespeicherten Zeitreise-Versionen und weist den Zustand je Schicht messtechnisch nach. Nicht erfasst sind externe Sicherungen, Flash-Remanenz, Dateisystem-Journale, Prozessspeicher und externe Verarbeiter.

**Unzulässig (Wortlaut-Lint T-LINT-1, Quelle, Doku, Berichtsvorlagen, Marketing):**

„garantiert“, „DSGVO-konform“, „rechtskonform“, „unwiederbringlich“, „informationstheoretisch gelöscht“, „kryptografisch gelöscht“, „sicher gelöscht“ ohne Berichtsbezug, „vollständig gelöscht“ ohne Klassenangabe.

### 13.3 Löschcheckliste (Textexport)

`erasure_checklist()` liefert einen Text mit Prüfpunkten, die der Betreiber außerhalb von Contextra abarbeitet. Inhalt (Auszug):

- Backups und Snapshots des Datenverzeichnisses: Aufbewahrungsdauer prüfen, Restore-Hinweis beachten (Restore kann gelöschte Daten zurückbringen).
- Anwendungs-, Monitoring- und Tracing-Logs, die IDs oder Inhalte enthalten.
- Eingesetzte Embedding- und Sprachmodelldienste: welche Eingaben dort gespeichert werden.
- Exporte, Berichte, Tickets und Kopien beim Betreiber.
- Swap- und Core-Dump-Konfiguration des Hosts.

Die Checkliste enthält den Hinweis, dass sie **keine Rechtsberatung** ist und vom Betreiber geprüft werden muss. Freigabe durch G-LEGAL (F-CHK).

### 13.4 Rechtsfragen (Gate G-LEGAL)

Ohne schriftliche Antworten darf das Produkt keine Löschaussage außer der Zulässigen (13.2) treffen.

1. Gilt physische Umschreibung mit Entfernen der alten Dateien als Löschung nach Art. 17, auch bei Flash-Medien mit Restrisiko?
2. Sind Embeddings personenbezogene Daten? (Vorsorge: ja [D].)
3. Gelten Slot-Nummern im HNSW und Tags als personenbezogen? Ist ein Graph-Rest bis zum Rebuild zulässig?
4. Dürfen Bericht und reduziertes Journal personenbezogene Bezüge (Commitment, Zeit) enthalten, und wie lange?
5. Sind aggregierte `Df`-Zähler personenbezogen?
6. Rolle von Contextra bei lokal installierter Software: Hersteller, Auftragsverarbeiter, Software ohne Zugriff?
7. Aufbewahrungspflichten je Branche und ihre Wirkung auf die Löschung.
8. Freigabe der Löschcheckliste (13.3).

---

## 14. Grenzen dieser Spezifikation

- **Keine Build- und Testläufe.** Kompilierbarkeit und Verhalten sind [U] oder Erwartung.
- **Nicht gelesen am Clone:** Textindex-Postingformat im Detail, Bloom- und Manifestinhalt (ob Schlüsselbereiche gespeichert werden), Blockcache (ob er entschlüsselte Blöcke hält), HNSW-Snapshot-Format, `kvcache`-Verwendung außerhalb der Entfernungsliste, `contextra-db`-Inhalt (WP-1.1).
- **Nicht nachgezählt:** Testzahlen aus Quelldokumenten. Zeilenzahlen sind `grep`- und `find`-Zählungen mit Obergrenzen.
- **Restrisiken** in 9.4 sind Schlussfolgerungen aus Eigenschaften von Flash und Dateisystemen, keine Messungen.
- **Kollisionswahrscheinlichkeit** der 64-Bit-DocId ist eine Rechnung, keine Messung.
- **Mit dem Clone-HEAD nicht abgeglichen:** die Behauptungen über die Schichtzugehörigkeit von `kvcache` und `graph` im Rückbau-Dokument.

---

## 15. Offene Entscheidungen für den Auftraggeber

| ID | Frage | Empfehlung | Wirkung bei Ablehnung |
|---|---|---|---|
| F-LEG | Existiert Altbestand (ausgelieferte Daten, V1–V3-Beweise, Legacy-WAL)? | Nein annehmen; falls ja: dokumentiertes Offline-Migrationswerkzeug mit Frist, dann Entfernen | Migrationspfad als v1-Scope |
| F-BLOB | Speichert Contextra Originaldokumente? | Nein. Nur Chunks und Metadaten. | Gleiche Löschpflicht, zusätzliche Schicht, Phase 4 |
| F-COMMIT | Lebensdauer `K_commit`-Epoche und `erased.set` | Epoche 365 Tage, danach Rotation; Commitments bleiben im Bericht als Pseudonym | Länger = mehr Pseudonymrest |
| F-LIC | Lizenz- und Vertriebsmodell | Außerhalb des Speicherkerns (D-23) | Lizenzlogik zurück in v1 |
| F-PY | Python-Bindings in v1 oder v1.1 | v1.1 (D-25) | Mehrscope, Verzögerung |
| F-MCP | MCP-Server in v1 oder v1.1 | v1.1 (D-25) | Mehrscope, Löschwerkzeuge erst nach Saga |
| F-GRAPH | Graph-Rückkehr | v1.1, nur mit Ebene-A-Adapter (D-17) | – |
| F-RETENTION | Einschränkung (Art. 18) als Zustand | v1.1 (D-15) | – |
| F-KEYSHRED | Schlüsselvernichtung als Schicht | v1.1 nach G-LEGAL (X-1) | – |
| F-AUDITUSER | Benutzerfreundlichkeit `ObligationDeclinesErasure` (Code-Liste) | Code-Liste gemeinsam mit Fachanwalt festlegen | – |
| F-SLA | Wert von `erase_sla` für Standard-Dokumentation | Vorschlag 24 h bis `Completed`, Alarm nach 1 h Pending | – |
| F-AUTO | Automatische Extraktion im Aufrufer oder außerhalb | Außerhalb | – |

---

## Anhang A: Rückbauliste (Kurzform)

| Pfad | Aktion |
|---|---|
| `crates/contextra-crypto/src/deletion_proof/` | entfernen |
| `crates/contextra-crypto/src/ed25519_proof.rs` | ersetzen durch `proof_v1.rs` (Report) |
| `crates/contextra-crypto/src/{kv_shredding,kv_cipher,revocation_log}.rs`, `kv_segment/` | entfernen |
| `crates/contextra-crypto/src/audit_chain.rs` | behalten |
| `crates/contextra-store/src/scanner/` | entfernen (`scanner/purge.rs`, `scanner/wal.rs` mit Canary im produktiven WAL [K: C-L20]) |
| `crates/contextra-store/src/kv/` | entfernen |
| `crates/contextra-store/src/wal/replay.rs` (7 `wal-integrity`-Gates) | Gates entfernen, Replay vereinheitlichen |
| `crates/contextra-store/src/lib.rs:45` | `#![allow(unexpected_cfgs)]` entfernen |
| `crates/contextra-store/src/lsm/ops/compaction.rs:225–231` | Sentinel-Fix (WP-2.1) |
| `crates/contextra-mvcc/src/ssi.rs:646–652` | Sentinel-Fix, Assertion |
| `crates/contextra-engine/src/contextra_impl/erasure.rs` | entfernen |
| `crates/contextra-engine/src/collection/crud/delete.rs` | umbauen (Lock, Saga-Anbindung) |
| `crates/contextra-vector/src/hnsw/{purge,deletion}.rs` | entfernen, neu als Zeroing (D-18) |
| `crates/contextra-text/src/inverted/{purge,scanner}.rs` | entfernen, neu über Matcher (D-20) |
| `crates/contextra-ports/src/scanner/`, `crates/contextra-types/src/witness.rs` | entfernen, neu in `types/src/erasure` |
| `crates/contextra-mcp/src/{proof_key,proof_key_env}.rs` | entfernen |
| `crates/contextra-simd/src/kernels/avx512.rs` (u8-Pfad) | entfernen (D-28) |
| `crates/contextra-ports/src/vfs.rs` und `crates/contextra-durable-fs/src/vfs.rs` | zusammenführen (D-04) |
| `docs/DELETION-DSGVO.md` | Status „Konzept“, durch diese SPEC ersetzt |
| `docs/SPEC.md` (2.799 Zeilen) | durch diese SPEC ersetzen, alte Fassung nach `archive/` |
| `PLAN.md` (1.397 Zeilen) | durch `PLAN.md` dieser Lieferung ersetzen |
| `docs/decisions/` | anlegen, ADR-001 aufnehmen |
| Workspace `members` / `default-members` | auf Kern laut 5.1 reduzieren |
| `capabilities.toml` | an Ringe 5.2 anpassen |
| `governance/*` | Ratchet-Dateien auf neue Zähler umstellen |

---

## Anhang B: Quellenregister

| Dokument | Basis | Verwendung in dieser SPEC |
|---|---|---|
| Fehlerkatalog Stand 2026-11-01 | `f653f0f73` | Befundliste [K], Priorisierung, D-C01/D-C02/B-S-Befunde |
| Löschpfad: Rückbau und Neuaufbau | `f653f0f73` | Entfernungsumfang, Anforderungen N-01..N-29 (übernommen in 8, 9, 10, 13) |
| Implementierungsspezifikation Erasure Core auf eigenem LSM-Tree | `9da0eca` | Befunde N1–N10 (neu, geprüft), Purge-Algorithmus (übernommen in 8.4) |
| Spezifikation v1 Final | `9da0eca1b` | Produktumfang, Phasen (Vorlage für PLAN.md), Nachweisformat |
| Spezifikation v1 Erasure | `9da0eca1b` | ersetzt durch Final, nur Vergleich |
| Implementierungsspezifikation SOLL (Löschkette, Speicherkern, Verifikation) | `9da0eca1b` | Haftungsrahmen, Anforderungsmatrix R1–R13, Journal-Dateien, Restrisiko-Katalog |
| Spezifikation und Umsetzungsplan | `c4e285c` | Ringe, Graph-Entkopplung (verworfen, D-17), Rückbau-Prüfliste |
| Entscheidungsgrundlage | `c4e285c` | Gates G1–G4 (übernommen als G0), Widerspruchsliste W1–W8 |
| LSM-Audit-Report (vollständig und Kurzfassung) | `9da0eca1b` + lokal | Testbefunde #1–#10 (übernommen in PLAN WP-2.x), Gemini-Teil verworfen (N12 Quelle) |
| `AGENTS.md`, `VETOES.md`, `docs/DELETION-DSGVO.md` | Clone | Normative Regeln, ADR-001 |
| Build-/Auditlog des Auftraggebers | Nutzerlauf | [U]-Befunde |

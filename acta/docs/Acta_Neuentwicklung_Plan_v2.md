# Acta – Neuentwicklung: Plan v2 (Clean Sheet)

*Principal-Rust-Engineering · Stand 2026-11-15 · Ersetzt, wo er widerspricht, `Acta_Implementierungsplan_Final.md` (v1)*

**Prämisse:** Contextra wurde für Air-Gap-Roboter gebaut, dieses Ziel ist verworfen, der Code trägt viele uneinheitliche Entscheidungen und wird als Projekt eingestellt. Acta entsteht **neu aus den Anforderungen**. Contextra ist Ideenquelle und Fehlerkatalog, nie Vorlage, nie Orakel.

**Grenzen:** Dieser Plan ist eine Planung auf Basis statischer Lektüre. Es wurde nichts kompiliert, gemessen oder getestet. Aussagen zu Contextra stammen aus meiner bisherigen Analyse (`50cdc3d`) und sind in v1 belegt. Alles Weitere sind Entwurfsentscheidungen mit benannten Prüfkriterien.

---

## 1. Was sich gegenüber v1 ändert

| v1-Element | Status in v2 | Grund |
| --- | --- | --- |
| Klassen ADOPT/HARDEN für `rank`, `text`, `vector`, `graph`, `ingest`, `simd`, `router`, `adapt`, `prompt-guard` | **gestrichen** | Jede Übernahme importiert Contextras Annahmen (Edge-Hardware, Latenzregelung, Selbstadaptation). Jedes Teil muss neu durch eine Anforderung begründet werden |
| `UPSTREAM.md`, Differentialtests gegen Contextra-Fixtures, Diff pro Iteration (D-13) | **gestrichen** | Ein Orakel, das Contextra heißt, schreibt dessen Verhalten fest. Neue Orakel: RFC-Testvektoren, Referenzimplementierungen, Modelltests, Properties |
| SPEC 3.0 als unantastbare Norm | **geändert:** Zusagen S1–S12 und Rechtsteil bleiben Anforderungen, die **Designteile** der SPEC (Tresor-Slots, KeyV2, SST/WAL-Formate, Saga-Schritte, Ringe, MVCC über P) sind Hypothesen | Die SPEC wurde aus dem Contextra-Code abgeleitet; ihre Formate sind dessen Erbe |
| Port-by-Redesign von `store` (D-15), Zerlegung von `db` (D-17) | **ersetzt** durch Build-vs-Buy-Entscheidung und Neuzuschnitt (Abschnitt 5) | Beides startete bei Contextras Zuschnitt |
| Level-Modell, Hub-Budgets, Trait-Besitz, Fehlerarchitektur, Gates | **bleibt** | unabhängig von Contextra; entstanden aus Architekturprinzipien |
| XChaCha20-Poly1305, HMAC-/SHA-2-Familie, sync Kern, handkodierte Formate | **bleibt als Hypothese** mit neuer Begründung (Abschnitt 6) | Begründung darf nicht „SPEC sagt es“ lauten |
| Kein `tokio` im Kern, Typestate, Typzwang (`Sealed`, `Derived`) | **bleibt** | Sicherheitsmuster, nicht Contextra-spezifisch |

---

## 2. Anforderungen statt Vererbung

### 2.1 Was mit dem Air-Gap-Ziel wegfällt und was neu ist

| Annahme in Contextra (Robotik, Air-Gap) | Konsequenz für Acta (Unternehmens-RAG) |
| --- | --- |
| Kein Netz, ein Gerät, ein Betreiber | Server mit Netzwerk, mehrere Mandanten, Administratoren, Insider-Risiko, Cloud-Betrieb möglich |
| Bedrohung: physische Kaperung, Manipulation des Geräts | Bedrohungen: Fehlbedienung, Insider, kompromittierte Anwendung, Backup-Diebstahl, Cloud-Snapshots, Lieferkette, Prompt-Injection |
| Offline-Inferenz auf Edge-Hardware (SIMD-Handoptimierung, 287 `unsafe`, PID-Latenzregler, Homeostat) | Inferenz lokal **oder** kontrolliert extern; Performance durch Standardbibliotheken, Hardwareoptimierung nur nach Messung |
| Autonomer Dauerbetrieb ohne Operator (Selbstheilung, Kognition, Agent) | Operator vorhanden; Prozessführung, Monitoring und Freigaben sind Produktfunktionen |
| Lizenz-Tiers, Capability-Dateien, Plugin-System | nicht gefordert |
| Eigenes IPC-Format (`wire`), eigener Sandbox-Pfad | Standard-Protokolle (gRPC/HTTP), Prozessisolation nur für Parser |

### 2.2 Anforderungsquellen (in Reihenfolge ihrer Verbindlichkeit)

1. **Recht:** DSGVO Art. 5, 12, 16–19, 25, 32; EDPB/DSK-Praxis laut SPEC §9. Daraus folgen **Eigenschaften**, nicht Mechanismen.
2. **Mission:** die acht Säulen.
3. **Zusagen S1–S12** der SPEC als **Eigenschaften** (z. B. „nach `Gated` liefert kein Lesepfad Klartext“). Wie sie erreicht werden, ist offen.
4. **Betriebsanforderungen:** Mandantenfähigkeit, Mehrbenutzerbetrieb, Backup/Restore, Monitoring, Upgrade ohne Datenverlust.

### 2.3 Bedrohungsmodell (Pflichtartefakt von Phase 0)

| Vertrauenszone | Beispiel | Annahme |
| --- | --- | --- |
| Z1 Acta-Kern | Prozess mit Schlüsselmaterial | vertrauenswürdig, aber fehlerhaft denkbar |
| Z2 Betreiber-Admin | Betriebsteam | **nicht** vertrauenswürdig für Klartext (Trennung Betrieb/Schlüssel) |
| Z3 Speicher/Backup | Disk, Snapshots, Object-Store | feindlich lesbar |
| Z4 externe Verarbeiter | LLM-APIs | außerhalb der Einflusssphäre, standardmäßig gesperrt |
| Z5 Kundendokumente | Quellsysteme | außerhalb |
| Z6 Eingaben | Uploads, Prompts | feindlich (Parser, Injection) |

Aus der Tabelle werden **Sicherheitsziele** abgeleitet (Vertraulichkeit je Zone, Integrität des Löschnachweises, Verfügbarkeit des Löschpfads). Jede spätere Entscheidung nennt die Zone, die sie schützt.

---

## 3. Leitprinzipien

1. **Anforderung vor Mechanismus.** Kein Bauteil ohne Verweis auf Recht, Mission oder Zusage.
2. **So wenig eigener Code wie die Anforderung erlaubt.** Eigenbau nur, wenn eine Anforderung (meist physische Kontrolle für Löschung) Bibliotheken ausschließt. Das ist ein messbares Kriterium (Abschnitt 5).
3. **Fehler unmöglich machen, nicht verbieten.** Typzwang statt Review-Regeln.
4. **Eine Lösung pro Problem.** Ein Hash-Ketten-Typ, ein Zeit-/Zufallsport, ein Fehlerklassifikator.
5. **Kleiner Kern, kleine Oberfläche.** Wenige Crates, harte Größenbudgets, keine Hubs mit Fremdwissen.
6. **Beweisbar vor schnell.** Messen vor Optimieren; `unsafe` nur mit Messwert als Begründung.
7. **Kein Wort über Konformität.** Berichte nennen messbare Klassen und Annahmen (SPEC §1.3 bleibt als Produktregel, weil sie rechtlich begründet ist).

---

## 4. Inspirationsregister (Contextra → Acta)

Regel: Contextra-Code wird **vor** dem Entwurf gelesen, um Probleme zu entdecken, und **nicht** beim Schreiben geöffnet. Jede Idee braucht eine ADR mit Anforderungsbezug. Kryptografie ist strikt Clean-Room (Autor schreibt aus Spezifikation und Testvektoren).

| Bereich | Idee in Contextra | Was Acta daraus macht |
| --- | --- | --- |
| Löschen | Gate vor physischem Purge, Saga mit Journal | **Problem übernommen** (sofortige Sperre, Wiederaufnahme nach Crash), **Lösung neu entwerfen** (Abschnitt 6.4) |
| Schlüssel | Hierarchie aus Salts im Slot-Tresor, Epochenmaskierung | Problem „Backup-Wiederbelebung“ übernommen; Lösung per Spike SP-2 neu bestimmt |
| Store | eigene LSM, 32 015 LOC, async | Problem „Löschung auf Dateiebene“ übernommen; Build-vs-Buy neu (SP-1) |
| Lineage | Herkunft jeder Ableitung | Idee gut; Realisierung (Bitmaps, `doc_ord`) wird neu gegen Alternativen verglichen |
| Zeitreise | MVCC über alles | **verworfen als Default**, siehe 6.5 |
| Indizes | HNSW, BM25/WAND, Graph (Eigenbau) | Anforderung „Index muss Löschung tragen“; Eigenbau nur falls Bibliothek scheitert (SP-3) |
| Chronik/Audit | drei getrennte Ketten (crypto, agent, privacy) | **eine** Kette |
| Fehler | Monolith `ContextraError` | Anti-Muster; Fehler je Crate |
| Hubs, `compat/`, Shims, `.inc` | Zwischenlösungen der Entkopplung | Anti-Muster |
| Duplikate (`Clock` ×10, `types_local`) | gewachsene Seams | Anti-Muster: ein Eigentümer je Seam |
| Router, Adapt, Homeostat, Agent, Cognition, Kvcache, Sandbox, Py, License, Wire | Robotik-/Edge-Funktionen | **nicht im Umfang**; Wiedereinführung nur über Anforderung (Backlog) |

---

## 5. Entscheidungsspikes (Build vs Buy, jeweils zeitlich begrenzt, endet mit ADR)

Jeder Spike hat **Hypothese, Kriterien, Entscheidungsregel**. Das Ergebnis wird gemessen, nicht diskutiert.

### SP-1 Speichersubstrat

**Frage:** Eigenes Log-/Segment-Format oder bestehende eingebettete Engine (z. B. redb, fjall; RocksDB nur mit FFI-Begründung)?
**Hypothese:** Die Last ist *schreiben einmal, lesen häufig, löschen nach Betroffenem*. Das spricht für **append-only versiegelte Segmente, gruppiert nach Betroffenem**, nicht für eine allgemeine LSM mit Random-Updates. Kryptografisches Löschen macht physische Bereinigung zur Hygiene.
**Kriterien (jeweils bestanden/nicht):** (1) Dateien eines Betroffenen oder Mandanten lassen sich **gezielt entfernen oder überschreiben**, ohne globale Compaction. (2) Crash-Verhalten ist unter `FaultVfs` deterministisch testbar. (3) Wert-Umschläge liegen **vor** dem Schreiben des WAL/Logs vor (kein Klartext im Pfad). (4) Kein FFI, oder FFI-Fläche klein und auditierbar. (5) Lizenz, Pflegestatus, Supply-Chain.
**Entscheidungsregel:** Erfüllt eine Bibliothek 1 bis 3, nutzen wir sie. Sonst bauen wir den kleinsten Segmentspeicher, der 1 bis 3 erfüllt (Ziel ≤ 6 000 LOC), **nicht** eine LSM.

### SP-2 Schlüsselverwahrung

**Frage:** Lokaler Tresor mit abgeleiteten DEKs (SPEC-Form) oder **gewrappte DEKs** (DEK pro Dokument, gewrappt unter Betroffenen-Schlüssel, gewrappt unter Mandantenschlüssel, Wurzel im KMS/HSM)?
**Hypothese:** Gewrappte Schlüssel sind einfacher: Löschen = Betroffenen-Schlüssel im Schlüsselspeicher vernichten; Backups enthalten nur gewrappte DEKs und werden nutzlos, **sobald** der Betroffenen-Schlüssel nirgends mehr existiert. Das ersetzt Epochenmaskierung weitgehend durch **Re-Wrap bei Wurzelrotation**.
**Gegenargument:** Millionen Betroffenen-Schlüssel dürfen nicht je ein KMS-Objekt sein (Kosten, Latenz). Lösung: lokaler **Schlüsselspeicher** (append-only, AEAD-versiegelt unter einem KMS-gehaltenen Wurzelschlüssel, mit Rotation). Er ist im Kern ein Tresor, aber ohne Salt-Ableitung und ohne Maskierungs-Spezialformat.
**Kriterien:** (1) Backup-Wiederbelebung ausgeschlossen (Testfall: alter Schlüsselspeicher + alte Daten ergeben nach Wurzelrotation Chiffrat ohne Schlüssel). (2) Löschen ist O(1) und crash-sicher. (3) Rotation ohne Stillstand. (4) Weniger Formate als SPEC-Tresor (Slots, Shards, Masken). (5) Fail-Closed: fehlender Schlüssel ist `Erased`, nie Fallback.
**Entscheidungsregel:** Gewrapptes Modell, falls 1 bis 3 bestehen und es strikt weniger Spezialformate erzeugt. Sonst SPEC-Tresor als Referenz.

### SP-3 Indizes (Text, Vektor)

**Frage:** Bibliotheken (z. B. tantivy mit eigener verschlüsselnder `Directory`; ein ANN-Crate oder ein einfaches Flat-/Segment-Verfahren) oder Eigenbau?
**Hypothese:** Löschstrategie „Segment neu bauen, alten Schlüssel vernichten“ braucht nur zwei Fähigkeiten: *aus Vektoren/Dokumenten ein Segment bauen* und *suchen*. Beides leisten Bibliotheken. Eigenbau von HNSW ist **nicht** begründet, solange Flat-Suche plus unveränderliche Segmente die Latenzziele treffen.
**Kriterien:** (1) Segmente lassen sich über Dateiebene **versiegeln** (Umschlag um die Segmentdatei oder `Directory`-Abstraktion). (2) Kein Klartext bleibt in Temp-Dateien. (3) Rebuild ohne gelöschte Ordinale in vorgegebener Zeit (Zielwert wird in Phase 0 gesetzt). (4) Recall/Latenz auf einem repräsentativen Korpus.
**Entscheidungsregel:** Zuerst Flat/Segmentiert; ANN-Bibliothek erst bei gemessener Latenzverletzung; Eigen-HNSW nur bei Scheitern an Kriterium 1 oder 2.

### SP-4 Laufzeit

**Hypothese:** Kern synchron (Threads), async am Rand. **Neue Begründung (SPEC-unabhängig):** Durabilitätspunkte lassen sich in synchronem Code deterministisch fehlerinjizieren; Cancel-Safety an jedem `fsync` entfällt; Loom ist anwendbar. **Kriterium:** Beibehalten, solange Erase-Latenz (`Gated`) unter Last im Zielbereich bleibt. **Trigger zur Revision:** Messung zeigt Blockierung durch Thread-Pool.

### SP-5 Zeitreise und Versionierung

Siehe 6.5. **Kriterium:** Kann die geforderte Nachvollziehbarkeit allein über die Chronik (Ebene N) erfüllt werden?

---

## 6. Zielarchitektur (Hypothese, durch Spikes zu bestätigen)

### 6.1 Zuschnitt: wenige Crates, keine Hubs

```
L0  acta-base        Newtypes (Tags, Ids), Clock/CryptoRng-Ports, ErrorClass   (≤ 1 500 LOC)
L1  acta-crypto      AEAD, KDF, HMAC, Ed25519, Sealed<P>, Dek (kein Default)   (≤ 4 000 LOC)
    acta-fs          Durable-FS, Scrub, Fault-Injection                        (≤ 2 500 LOC)
    acta-sys         mlock/MADV_DONTDUMP/core-limit, SecureBuf (einziges unsafe) (≤ 1 500 LOC)
L2  acta-erasure-api Trait ErasureLayer, ErasedView, Gate-Lesesicht            (≤ 600 LOC)
L3  acta-keys        Schlüsselspeicher + KeyAnchor (SP-2)
    acta-seg         versiegelter Segmentspeicher (SP-1)
    acta-index       Text-/Vektorsegmente, Lineage, Versiegelung (SP-3)
    acta-pii         PII-Erkennung
    acta-ingest      Parser, Chunking (untrusted, ressourcenbegrenzt)
L4  acta-erasure     Gate, Journal, Saga, Messung, Bericht
    acta-chronicle   Chronik Ebene N, N-Validator, Hash-Kette
L5  acta-engine      Use-Cases (Ingest, Retrieval, Erase, Maintenance)
L6  acta-node        Composition Root, Boot-Selbsttest
L7  acta (API), acta-cli
--  acta-verify      unabhängiger Prüfer, keine Acta-Abhängigkeit
dev acta-testkit, xtask
```

Unterschiede zu v1: kein `acta-types`/`acta-ports`-Hub (nur ein winziger Vertrags-Leaf `acta-erasure-api`), kein `acta-erasure-adapters` (Index- und Speichercrates implementieren `ErasureLayer` direkt, `acta-erasure` kennt sie nicht), kein `acta-mvcc`, kein `acta-checkpoint` als eigenes Crate (Backup-Kopien laufen über die Segmentverwaltung), keine Router-/Adapt-/Kvcache-Crates. Die Level-Regel aus v1 (Kante nur nach strikt unten, geprüft durch `xtask`) bleibt.

### 6.2 Datenebenen

Die Trennung **N (Chronik), P (personenbezogen, versiegelt), D (Ableitungen)** bleibt, weil sie das Kernparadoxon löst (Rechtsfolge, nicht Contextra-Erbe). Ableitungen sind at rest versiegelt (v1 D-11), weil sonst „kein persistierter Klartext von Betroffenen“ nicht gilt.

### 6.3 Typzwang

`Sealed<P>` (Store nimmt kein Klartext-P), `Derived<T>` mit nicht leerer Lineage, Schlüsseltypen ohne `Default`/`Clone`/`Debug`, Tag-Newtypes mit privaten Feldern, `Scanner<Unarmed→Armed>`. Jeweils mit Compile-Fail-Test.

### 6.4 Löschpfad (neu gedacht)

Invarianten statt Schrittnummern: (I1) Sperre ist dauerhaft, bevor irgendetwas Physisches geschieht. (II) Schlüsselvernichtung erst nach Bestätigung aller Schichten, dass flüchtige Kopien geleert sind. (III) Jeder Zwischenzustand ist nach Crash wiederaufnehmbar; es gibt keinen Zustand „fehlgeschlagen“. (IV) Der Nachweis entsteht nur aus einer Messung mit Positivkontrolle. Die Schrittfolge (Absicht → Gate → Quittung → Schlüsselvernichtung → Purge → Rebuild → Messung → Bericht) wird als **Zustandsmaschine mit Journal** modelliert und per **Modelltest** (`proptest`-Zustandsmaschine plus Crash-Injection an jedem `fsync`) gegen I1 bis IV geprüft, nicht gegen eine Contextra-Crashmatrix.

### 6.5 Zeitreise (Vorschlag, SPEC-Änderung)

Nachvollziehbarkeit („auf Basis welcher Normen, Prompts, Modellstände, Grenzwerte“) betrifft Ebene N. Die Chronik ist klein, append-only, bitemporal und hash-verkettet. **Ebene P bekommt keine allgemeine MVCC-Zeitreise**, sondern einfache Versionierung für Berichtigung (Art. 16) und Aufbewahrungsfristen. Damit entfallen `acta-mvcc`, `SnapshotRevoker` und die gesamte „as_of auf P“-Problematik; die Zusage „Löschung gewinnt immer“ gilt trivial.
**Preis:** Keine historische Rekonstruktion personenbezogener Inhalte (rechtlich ohnehin unerwünscht). **Zu bestätigen:** Produktanforderung, ob Kunden eine Zeitreise über Nicht-Personendokumente (Richtlinien als Dokumente) erwarten; diese gehören in Klasse `Norm` → Chronik.

### 6.6 Inferenz und Egress

Ein Vertrag `Embedder`/`Generator` im jeweiligen Verbraucher-Crate (`acta-engine`), Clients als Randcrates. **Egress standardmäßig gesperrt**; Remote nur explizit konfiguriert, im Bericht als Zone Z4 ausgewiesen. Kein eigenes Routing-Framework im Umfang; ein einfacher Strategie-Eintrag in der Konfiguration genügt, bis eine Anforderung mehr verlangt.

---

## 7. Roadmap

Kein Kalender, weil Team und Kapazität unbekannt sind. Jede Phase endet mit einer automatisiert prüfbaren Abnahme.

| Phase | Inhalt | Abnahme |
| --- | --- | --- |
| **P0 Fundament** | Bedrohungsmodell, Anforderungskatalog (Eigenschaften aus Recht und S1–S12), Testbarkeitsstrategie, `xtask` (Level-Gate, Defect-Firewall), CI mit pinned Toolchain, `acta-base`, `acta-fs` mit Fehlerinjektion | Gates grün, jede Zusage hat einen benannten Testtyp |
| **P1 Spikes** | SP-1 bis SP-5 mit Messbericht und ADR | je Spike Entscheidung nach Regel, dokumentiert |
| **P2 Krypto und Schlüssel** | `acta-crypto` (Clean-Room, Testvektoren), `acta-keys` nach SP-2 | `T-NOFALLBACK`, Wiederbelebungstest, Bytes-Grep über Schlüsselspeicher |
| **P3 Walking Skeleton** | Ingest (`Sealed<P>`) → Segmentspeicher → Textindex mit Lineage → synchrone Löschung → Messung → Bericht → `acta-verify` | Ende-zu-Ende-Kanarientest, Crash-Injection an jedem Durabilitätspunkt, unabhängiger Bericht-Check |
| **P4 Breite** | Vektorindex, Retention-Klassen, Mandanten, Restrict/Supersede, Backup/Restore-Schutz | Eigenschaftstests je Zusage |
| **P5 Chronik und Betrieb** | `acta-chronicle`, `acta-engine`, `acta-node`, Boot-Selbsttest, Monitoring | Architektur-Gates, Chronik-Determinismus |
| **P6 Härtung** | Fuzzing aller Parser, externe Krypto-Prüfung, Rechtsfreigabe, Lasttest Erase-Latenz | Audit ohne Acta-Code, Messwerte statt Zielwerte |

---

## 8. Qualitätssicherung

- **Orakel:** RFC-/Draft-Testvektoren (XChaCha20-Poly1305, HKDF, HMAC, Ed25519), unabhängige Referenzimplementierung im Test, Modelltests (Zustandsmaschine Löschpfad), Properties (Round-Trips, Invarianten), Fuzzing (alle Parser und Formate).
- **Gates (`cargo xtask gates`):** Level-Modell, Hub-/Crate-Budgets, kein `tokio` im Kern, kein `Default` für Schlüssel/IDs, kein `unwrap_or_default()`/`let _ =` auf `Result` in Krypto/Speicher/Löschung, kein direkter `std::fs`-Zugriff außerhalb `acta-fs`, keine nicht injizierte Zeit/Zufallsquelle, Wortfilter (SPEC §1.3), `cargo deny`/`cargo audit`.
- **Reviewregel:** Vier Augen bei Krypto, Schlüsselverwahrung, Löschpfad und Plattenformaten.
- **Eigenschaft ohne Test = nicht erfüllt.**

---

## 9. Risiken

| Risiko | Einschätzung | Gegenmaßnahme |
| --- | --- | --- |
| Spikes fallen anders aus als die Hypothesen | mittel | Entscheidungsregeln vorab festgelegt; Spikes zuerst, bevor Code auf ihnen aufbaut |
| Zu wenig Eigenbau-Erfahrung beim Segmentspeicher | hoch | Budget ≤ 6 000 LOC; Fault-Injection von Tag 1; Bibliothek bevorzugen |
| SPEC-Zusagen und neue Designentscheidungen kollidieren (z. B. `as_of` auf P) | mittel | Änderungsliste an SPEC-Autor, Entscheidung vor P3 |
| Krypto-Eigenbau | hoch | nur geprüfte RustCrypto-Bibliotheken, keine eigenen Primitive, externes Audit |
| Rechtliche Fehlaussagen | mittel | Wortfilter, Rechtsfreigabe |
| Umfangsdrift durch Wiedereinführung von Robotik-Features | mittel | Backlog-Regel: nur über Anforderung |

---

## 10. Offene Punkte, die ich von Ihnen brauche

1. **Status der SPEC 3.0:** Anforderungsquelle (Zusagen S1–S12, Rechtsteil) ja; Designteile als Hypothese. Bestätigen Sie das?
2. **Zeitreise:** Ist eine Zeitreise über personenbezogene Inhalte (Ebene P) ein Produktziel oder genügt die Chronik für Ebene N (6.5)?
3. **Zielumgebung:** On-Premise mit HSM/KMS beim Kunden, Cloud-Betrieb oder beides? Das bestimmt SP-2 und das Bedrohungsmodell (Z2, Z3).
4. **Lastprofil:** Größenordnung von Dokumenten, Betroffenen pro Mandant und Löschvorgängen pro Tag, damit Spike-Zielwerte nicht geraten sind.

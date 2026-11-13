# Acta – Spezifikation Fassung 1.0-E

**Konsolidierter Entwurf** aus drei Quellen · Stand 2026-11-12
Quellen: Repository `tfufuz1/contextra` (Klon, HEAD `5f3ccbb`, 39 Crates), `SPEC.md` (Acta v2, Fassung 0.2), `ACTA_SPEC.md` (Acta v0.1).
Begleitdokument: `ACTA_DATEIZUORDNUNG.md` (jede Contextra-Datei mit Zielpfad in Acta oder Verwerfungsgrund).

---

## Inhalt

0. Status, Geltung, Beweisstand
1. Produkt und Zusagen
2. Bedrohungsmodell und Vertrauensdomänen
3. Konflikte der Quellen und Entscheidungen
4. Ebenen und Datenklassen
5. Invarianten und verbotene Muster
6. Kryptografie
7. Custodian (Schlüsseldienst)
8. Speicher (SQLite, Schema)
9. Zeit und Versionierung
10. Datenpfad
11. Suche (RAM-Index)
12. Löschprotokoll
13. Nebenläufigkeit
14. Öffentliche Schnittstelle
15. Konfiguration, Fehler, Beobachtung
16. Betrieb
17. Nachweis und Prüfung
18. Architektur, Crates, Schichtung
19. Code-Standards
20. Tests und Gates
21. Roadmap
22. Restrisiken, offene Entscheidungen
- Anhang A: Konstanten
- Anhang B: Testvektoren
- Anhang C: Zulässiger Wortlaut

---

## 0. Status, Geltung, Beweisstand

### 0.1 Was dieses Dokument ist

Die normative Spezifikation eines verschlüsselten, zeitreisefähigen Wissensspeichers für personenbezogene und normative RAG-Daten. Löschen eines Betroffenen ist das Vernichten seines Schlüssels beim Custodian. Ein Dritter prüft das mit eigenem Code.

Es ersetzt `SPEC.md` 0.2 und `ACTA_SPEC.md` 0.1 vollständig für alle Punkte, zu denen es eine Entscheidung trifft (Kapitel 3). Punkte, die es nicht ändert, gelten aus der jeweiligen Quelle weiter und sind dort referenziert.

### 0.2 Beweisstand

| Aussage | Stand |
|---|---|
| Acta ist gebaut | **nein** |
| Contextra ist gebaut oder getestet (durch mich) | **nein**. In der Arbeitsumgebung ist kein `cargo` verfügbar. Ich habe weder kompiliert noch Tests ausgeführt. |
| Strukturbehauptungen über Contextra | **[V]** am Dateisystem oder am Quelltext geprüft (Methode in 0.3) |
| Übernommene Befunde aus Vorlagen | **[A]** aus `SPEC.md` 19.1 übernommen, nicht erneut geprüft |
| Repo-eigene Aussagen (`docs/SPEC.md` v5) | **[Ü]** übernommen; einzelne Punkte stichprobenartig geprüft |
| Kryptografische Testvektoren Anhang B.2 bis B.7 | **ja**, Vorlage `SPEC.md` B, am 2026-11-06 mit zwei unabhängigen Wegen erzeugt |
| Testvektoren Ebene N (Anhang B.8) | **ja**, am 2026-11-12 mit Standardbibliothek erzeugt (`gen_chronicle_vectors.py`) |
| Zahlen (Latenz, Größe) | **[Ziel]**, ungemessen |
| Kryptografie extern geprüft | **nein** (Gate G4) |
| Rechtswirkung der Vernichtung geklärt | **nein** (Gate G5) |

**Keine Aussage dieses Dokuments belegt Fehlerfreiheit.** Erreichbar sind: ein Modell, dessen Verletzung einen Test rot macht; eine externe Prüfung des kleinen Kerns; ein Auditor, der ohne unseren Code prüft.

### 0.3 Methode der Repo-Analyse

1. Klon, Dateibaum, Zeilenzahlen je Crate (`find`, `wc`).
2. Workspace-Auflösung: Jedes `members`-Verzeichnis auf `Cargo.toml` geprüft (39 von 39 vorhanden).
3. Krypto-Primitive und Abhängigkeiten per `Cargo.toml` und `grep` je Crate.
4. Fehlermuster (`unwrap_or_default`, `from_utf8_lossy`, `[0u8; 32]`, Konstanten als Schlüssel) mit Kontextprüfung (Test- oder Produktionscode per `#[cfg(test)]`-Grenze).
5. CI: `|| true`-Muster in `.github/workflows/*.yml`.
6. Features: `default`-Liste je Crate gegen die Regel R10 aus `docs/SPEC.md`.

Nicht gemacht: Ausführen von Tests, Fuzzing, Benchmarks, `cargo-mutants`, Abhängigkeitsprüfung (`cargo audit`/`deny`).

---

## 1. Produkt und Zusagen

### 1.1 Mission

> **Acta ist ein verschlüsselter Wissensspeicher für personenbezogene und normative RAG-Daten mit Zeitreise. Das Löschen eines Betroffenen besteht darin, dessen Schlüssel bei einem unabhängigen Custodian zu vernichten. Ein Dritter prüft das mit eigenem Code.**

Die Zeitreise (was galt wann, was wusste das System wann) betrifft nur Normen und Konfigurationen ohne Personenbezug. Personenbezogene Daten sind nur innerhalb eines kurzen Horizonts zeitreisefähig. Suche (BM25, exakte Vektorsuche, Hybrid) ist notwendig, aber nicht das Alleinstellungsmerkmal.

### 1.2 Leitgedanke Nachweis

Ein Bericht, den das System über seine eigene Löschung ausstellt, ist Selbstauskunft und beweist nichts. Auch mit Signatur nicht, wenn der Signierschlüssel im selben System liegt. Acta stellt deshalb **keinen Bericht als Beweis** aus. Der Nachweis besteht aus Tatsachen, die nicht vom Acta-Code abhängen:

1. **Vernichtungsbeleg** des Custodians, ausgestellt von einer anderen Partei.
2. **Datenformat**, das ein Auditor in etwa 60 Zeilen eigenem Code lesen kann (Anhang B).
3. **Kanarien-Test** des Auditors auf der echten Konfiguration (17.3).
4. **Chronik-Verankerung** der Löschung als Commitment ohne Personenbezug (9.6).

### 1.3 Zusagen

Jede Zusage hat mindestens einen Test (Kapitel 20). Ohne Test ist sie keine Zusage.

| ID | Zusage | Test |
|---|---|---|
| **Z1** | Ab dem Zeitpunkt, an dem `erase()` mit Zustand ≥ `Gated` zurückkehrt, liefert kein Lesepfad mehr Klartext des Ziels, und es wird nichts mehr in das Ziel geschrieben. Ausnahme: Leser, deren zweite Gate-Prüfung vor dem Gate lag (12.3). | T-DEL-1, T-CONC-1 |
| **Z2** | Alle persistierten Klartextdaten eines Betroffenen liegen in Envelopes, die unter einem Schlüssel versiegelt sind, der nur beim Custodian liegt. Ist er dort vernichtet, ist kein Envelope des Betroffenen entschlüsselbar. Gilt ab Zustand `Destroyed`. | T-DEL-1, T-CANARY-1 |
| **Z3** | Ein Dokument mit mehreren Betroffenen wird unlesbar, sobald einer von ihnen gelöscht wird. | T-MULTI-1 |
| **Z4** | Eine Unterbrechung an jedem benannten Punkt führt nach Neustart zur Vollendung oder zu sichtbarem Zustand `Gated` mit `pending`. Eine bestätigte Sperre geht nie verloren. | T-CRASH-* |
| **Z5** | Jeder Nachweis nennt seine Attestierungsklasse (17.4) und seine Grenzen (Kapitel 22). | T-EVID-1 |
| **Z6** | Kein Rückgabewert behauptet etwas, das nicht gemessen wurde. Nicht implementierte Funktionen liefern `Err(NotImplemented)`. | T-STUB-1 |
| **Z7** | Kein Fehler im Schlüssel-, Zufalls-, Ableitungs- oder Siegelpfad wird zu einem gültigen Wert (kein Nullschlüssel, keine leere Hülle, kein Ersatzschlüssel). | T-NOFALLBACK-*, T-RNG-1, T-SEAL-* |
| **Z8** | **Neu.** Eine Chronik-Abfrage mit identischen Parametern (`valid_at`, `known_seq`) liefert identische Ergebnisse, solange die betroffenen Einträge existieren. | T-CHR-2 |
| **Z9** | **Neu.** Die Chronik enthält keine `subject_ref`, keine Schlüssel und keine Klartexte von Betroffenen. Ein Eintrag mit Betroffenenbezug wird abgelehnt. | T-CHR-1, T-OBS-1 |
| **Z10** | **Neu.** Eine Löschung ist nach Abschluss als Commitment in der Chronik verankert, ohne Verknüpfbarkeit zu Person oder Mandant (9.6). | T-CHR-3 |

### 1.4 Zusagen, die Acta nicht macht

Verboten in allen Attestierungsklassen: „DSGVO-konform“, „garantiert gelöscht“, „unwiederbringlich gelöscht“, „zertifiziert“, „revisionssicher“, „fehlerfrei“, „bewiesen sicher“, „Löschung aus allen Sicherungen“ (Contextra-Regel INV-7 übernommen).

Nicht gelöscht werden durch Acta: Klartext im RAM, Swap, Core-Dumps, Hibernation; Backups des Custodians und des Datenverzeichnisses; Daten bei externen Verarbeitern; Prompts, Antworten, Logs des Aufrufers. Prüfung von Aufbewahrungspflichten (Art. 17 Abs. 3 DSGVO) liegt beim Aufrufer vor `erase()`.

### 1.5 Nicht-Ziele Version 1

Server, Netzwerkprotokoll, Replikation, eigener Speicher-Engine-Bau (LSM, WAL, MVCC/SSI), persistierter Suchindex, HNSW, Embedding-Erzeugung, LLM-Aufrufe, Dateiformat-Parsing, Ingest und Klassifikation, Windows, Schlüssel-Escrow durch den Hersteller, Telemetrie, Prozesshärtung (Core-Dump-Verbot, `mlock`), Einschränkung der Verarbeitung (Art. 18), Berichtigung per Schlüsselrotation, automatische Personenerkennung, Signierschlüssel im Acta-Prozess (D-09), Ed25519-Belege aus Acta (D-10), SIMD-Kernel (D-11), Kontext-Cache (ACTA_SPEC §3).

Zurückgestellt für Version 1.1 oder 2: siehe Kapitel 21.

---

## 2. Bedrohungsmodell und Vertrauensdomänen

```
Domäne A  Anwendung + Acta-Prozess     Klartext nur im RAM
Domäne B  Datenverzeichnis             nur Ciphertext + Pseudonyme + Metadaten (SQLite)
Domäne C  Custodian (Schlüsseldienst)  hält TK, DK, CK, KC; vernichtet; führt eigenes Audit-Log
Domäne D  Auditor                      eigener Code, eigener Zugang zu B und C
```

| ID | Angreifer / Prüfer | Acta MUSS |
|---|---|---|
| W1 | Kopie von B zu beliebigem Zeitpunkt | nach Vernichtung unlesbar halten (Ausnahme R7) |
| W2 | Kopie von B nach Löschung plus alle dann vorhandenen Schlüssel in C | Daten des Gelöschten unlesbar halten |
| W3 | Betreiber behauptet Löschung ohne Löschung | durch Custodian-Beleg und Auditor-Test aufdeckbar machen |
| W4 | Absturz, I/O-Fehler an beliebiger Stelle | fail-closed; keine verlorene bestätigte Sperre |
| W5 | Manipulation von B, Einspielen älterer Stände | nie still falsche Klartexte (AEAD); Manipulation der Löschhistorie erkennbar; Wiedereinspielen per Abgleich mit C und Löschregister auffangen |
| W6 | Zwei Mandanten im selben Prozess | Isolation (I-12), kein gemeinsamer veränderlicher Zustand (I-13) |
| W7 | **Neu.** Angreifer liest Chronik (Domäne B) | Chronik enthält keine Verknüpfung zu Betroffenen (I-19) |
| P | Auditor ohne Vertrauen in Acta-Code | Format, Beleg und Testverfahren so liefern, dass ohne Acta-Code geprüft werden kann |

**Nicht im Modell** (Restrisiken, Kapitel 22): Angreifer mit Zugriff auf laufenden Prozessspeicher (R1); Zugriff auf Backups des Custodians (R2); kompromittierter Custodian vor Vernichtung (R7); kryptografische Brechung (R3); Seitenkanäle (R16).

---

## 3. Konflikte der Quellen und Entscheidungen

Die drei Quellen widersprechen sich an mehreren Stellen. Jede Entscheidung unten ist verbindlich. Sie hat eine ID (D-xx), die im Text referenziert wird.

| ID | Thema | SPEC.md | ACTA_SPEC | Contextra | **Entscheidung** | Begründung |
|---|---|---|---|---|---|---|
| D-01 | Zeitreise | Nicht-Ziel 1.5 | Kern (Ebene N, MVCC) | MVCC, keine Zeitreise im Löschpfad | **Zeitreise nur auf Ebene N (Chronik) unbegrenzt, auf Ebene P innerhalb eines Horizonts** | Erfüllt beide Anforderungen ohne MVCC-Eigenbau |
| D-02 | Speicher-Engine | SQLite, kein Eigenbau | `acta-store` (Contextra-LSM erweitert) | LSM, 31.868 Zeilen Produktionscode | **SQLite** (`bundled`, Pin ≥ 3.40) | Contextra-LSM ist nicht auf Löschsemantik ausgelegt (Befund 19.2) und zu groß für den Kern |
| D-03 | Nebenläufigkeit | Mutex W, RwLock S, Mutex C | MVCC mit SSI | MVCC, TxBuffer, SequenceLog | **SPEC.md-Modell** (eine Schreibsperre, eine SQLite-Verbindung) | Ein Schreiber, SQLite serialisiert ohnehin; SSI erfordert Löschwissen in MVCC (Befund ACTA 6.6) |
| D-04 | Schlüsselhierarchie | DEK aus TK, DK (HKDF), Schlüssel nur beim Custodian | `KeyAnchor` wrappt je Einheit DEK, KEK je Epoche | DEK je Dokument, `AncestorSalts`, Ersatzwert `[0u8;32]` | **SPEC.md-Modell**; `KeyAnchor`, `EpochSeal`, DEK-Wrapping entfallen | Kein gespeicherter DEK, kein Wrapping, kein Schlüssel im Datenverzeichnis (I-3). Ableitung aus Custodian-Schlüsseln ist auditierbar (Anhang B) |
| D-05 | Verschlüsselungsverfahren | XChaCha20-Poly1305, 192-Bit-Nonce | nicht festgelegt | AES-256-GCM-SIV, 96-Bit-Nonce, **Nonce als Parameter** `seal_record_with_nonce` | **XChaCha20-Poly1305** (Anhang B verifiziert). Contextra-AES-GCM-SIV ist die dokumentierte Alternative O2 | 192-Bit-Zufallsnonce; Nonce ist nicht Parameter der öffentlichen API (I-23) |
| D-06 | Löschbeleg | Custodian-Beleg (A1/A2) | Ed25519-signierter `ErasureProof` aus Acta, Checkpoints signiert | Ed25519-Beleg vom Prozess selbst (`DeletionProofKeyPair::generate`) | **Kein Acta-Signierschlüssel**. Beleg kommt vom Custodian, Acta speichert ihn unverändert | Selbstsignatur ist Klasse A0 und darf nicht als Löschnachweis gelten (SPEC 1.2) |
| D-07 | Löschprotokoll | Zustände Active, Gated, Destroyed, Completed + `pending` | `ErasureState` mit `Escalated`, `Restricted`, `Hygiene`, Pending | `saga/core.rs` mit Phasen 6/7/9/10 | **SPEC.md-Zustandsmodell** plus `Escalated` als Ereignis (nicht als Zustand): `pending`-Ursache mit `ErasePastSla` | Vier Zustände sind prüfbar; `Failed` existiert nicht (Wiederholung bis Erfolg) |
| D-08 | Zeitbegriff der Chronik | `valid` × `recorded` (ACTA) | — | — | **Übernommen aus ACTA**; Zeitstempel `recorded` als Sequenz (`commit_seq`) plus Wanduhr-Millisekunden | Sequenz ist die einzige Ordnung, die Acta selbst erzeugt |
| D-09 | Checkpoint-Signatur | — | Ed25519 mit Schlüssel im Prozess | — | **Hashkette und Merkle-Wurzel ohne Signatur**; externe Verankerung über Port `Notary` (RFC 3161) in 1.1 | Signierschlüssel im Prozess widerspricht I-3; Signatur ist ab 2.0 über Custodian-Operation `sign` (Klasse A2) |
| D-10 | Beleg-Signatur allgemein | — | Ed25519-`ErasureProof` | Ed25519 in `deletion_proof`, `report/signing` | **Entfällt** (Folge aus D-06) | — |
| D-11 | SIMD | — | Vektor-SIMD in `acta-vector` (ACTA 6.6) | `contextra-simd` mit AVX2/AVX-512/NEON, 270 `unsafe` (grob gezählt) | **Skalar, 8 feste Akkumulatoren** (SPEC 8.5). SIMD ist Version 1.1 und nur mit Determinismsnachweis | Ergebnisgleichheit über Plattformen (INV-5) ist mit unterschiedlicher Reduktion nicht gegeben |
| D-12 | `unsafe` | Nur in `acta-sys` | Nur in `acta-sys` | Inseln `simd`, `sys`, `wire`, `vector` (teils) | **Kein `unsafe` in Acta-Crates.** `#![forbid(unsafe_code)]` in allen | Mlock, mmap, SIMD sind Nicht-Ziele. SQLite-C bleibt Restrisiko R15 |
| D-13 | Schlüssel-IDs | `ctx2/<store>/t|s/...` | `KeyKind` mit 11 Arten | `KeyPurpose` in `governance/key_purposes.toml` | **`KeyId`-Namensraum aus SPEC 5.4**, erweitert um `c` (Chronik-Schlüssel), `k` (Commitment-Epoche) | Zwei Präfixe je Zweck, Kollisionsfrei |
| D-14 | Ports und Composition Root | `Custodian`, `BlobStore` als Trait | `Wiring` mit Pflichtfeldern in `acta-node` | Traits in `contextra-ports` (Zentral-Brücke) | **Port-Traits im konsumierenden Crate** (R2). **`BlobStore` ist kein Port**: SQLite ist die einzige Engine (D-02). Composition Root ist die Anwendung; `Acta::open(Config)` nimmt Custodian, Registry, Observer, Clock, Rng als Pflichtparameter | Port nur dort, wo Austausch Sinn hat (Custodian, Registry). Keine Dead-Port-Gefahr bei einer Implementierung |
| D-15 | Tenant-Schlüssel und Chronik | — | Chronik-Schlüssel nicht definiert | — | **CK_t** (Chronik-Schlüssel je Mandant) im Custodian. Wird nur mit dem Mandanten vernichtet | Chronik-Payloads sind vertraulich, aber nicht betroffenenbezogen |
| D-16 | Kollektionen als Löschziel | — | `Collection`-Target | — | **Nicht in v1** (1.1, O15) | Komplexität ohne Anforderung im Auftrag |
| D-17 | Ingest und Klassifikation | — | `acta-ingest`, `Classifier`, `Redactor` | `contextra-ingest` (21.140 Zeilen Produktionscode) | **Nicht in v1.** Acta nimmt nur Einheiten entgegen, deren Klasse der Aufrufer erklärt | Klassifikation ist unscharf und gehört nicht in den Löschkern. Vgl. Contextra-Zerlegung |
| D-18 | Löschregister | — | — | — | **Übernommen aus SPEC 13.4**: `ErasureRegistry`, Konfiguration `Required` (Standard) | — |

---

## 4. Ebenen und Datenklassen

### 4.1 Drei Ebenen

```
  Ebene N  CHRONIK     Normen · Richtlinien · Konfig-/Modellstände · Entscheidungsregeln
  (bitemporal,         · Löschcommitments · Checkpoints (Hashketten)
   unbegrenzt)         keine Personendaten (I-19)                        ── Brücke: nur Commitments
                                                                            
  Ebene P  TRESOR      Personenbezogene Dokumente, versioniert innerhalb Horizont
  (löschbar,           Envelopes unter DK/TK · Erase gewinnt             ── Lineage: Dokument → Einheit
   Horizont)           Tabellen docs, doc_versions, subjects, erase_log

  Ebene D  ABLEITUNG   RAM-Index: Katalog, Vektormatrix, Postings, Filter
  (nur RAM,            nie gespeichert, nie Quelle, bei Öffnen neu gebaut
   nie persistiert)
```

Ebene D ist gegenüber ACTA_SPEC verkleinert: Derived-Strukturen werden nicht persistiert. Dadurch entfallen Reparatur, Lineage-Purge und HNSW-Kantenreparatur (SPEC 8.1). Der Preis ist Neuaufbau beim Öffnen (Ziel ≤ 60 s für 10⁵ Dokumente, [Ziel]).

### 4.2 Datenklassen (normativ)

| Klasse | Beispiele | Ebene | Zeitreise | Löschung | Speicherung |
|---|---|---|---|---|---|
| `Norm` | Gesetzestext, interne Richtlinie, Modellversion, Konfigurationsstand | N | unbegrenzt | keine Löschung; Korrektur über `supersedes` | Chronik |
| `Personal` | Kundenstammdaten, Vertrag mit Namen | P | Horizont (Standard 30 Tage, oder bis `retain_until`) | Crypto-Shred + Purge | Tresor |
| `Special` | Art. 9/10 (Gesundheit, Strafurteile) | P | Horizont 0 | Crypto-Shred + Purge, ohne Karenz | Tresor |
| `Evidence` | Löschcommitment, Checkpoint | N | unbegrenzt | nie (keine Personendaten, I-19) | Chronik |
| `Unclassified` | Ergebnis fehlender Klassifikation | P | keine | wie `Personal` | Tresor (BR-2: im Zweifel personenbezogen) |

Acta prüft die Klasse nicht inhaltlich. Der Aufrufer erklärt sie (`Subjects::Personal` bzw. `Subjects::NonPersonal` für `Norm`; Chronik-Eintrag mit `ClassAssertion::NonPersonal`). Fehlklassifikation ist Restrisiko R18. Acta lehnt offensichtliche Verstöße ab (Betroffenen-Handle in Chronik-Text ist nicht prüfbar, Kennzeichnung schon).

### 4.3 Brücke

- **`SubjectRef`** (16 Byte, Zufall, SPEC 7.1): Pseudonym. Zuordnung Person → `SubjectRef` liegt beim Aufrufer (R10).
- **`EraseCommitment`** = HMAC-SHA256(`KC_e`, `"ctx/v2/commit" ‖ 0x00 ‖ tenant_id ‖ subject_ref`) mit Epoche `e`. Enthält keine rückrechenbare Information, solange `KC_e` existiert. Nach Vernichtung von `KC_e` ist das Commitment nicht mehr verknüpfbar (9.6, R19).
- Die Chronik sieht nur Commitments, Manifestwurzeln und Belegbinärwerte, nie `subject_ref`.

---

## 5. Invarianten und verbotene Muster

### 5.1 Invarianten

Übernommen aus SPEC 3.1 (I-1 bis I-18) und ergänzt um:

| ID | Invariante | Prüfung |
|---|---|---|
| **I-1** … **I-18** | wie SPEC 3.1 (Gate-Lesegarantie, fail-closed, keine Schlüssel in B, kein ableitbares Geheimnis, kein Fehler zu Wert, Gate vor Destroy, Destroyed nur nach Gegenprobe, Completed nur nach Destroyed, durable Sperre, monotone Zustände, Lesbarkeitsformel, Isolation, kein globaler Zustand, keine stillen Korrekturen, wahrhaftige Rückgaben, keine Personendaten in Beobachtbarem, Ein Prozess je Verzeichnis, Identifikatoren sind Bytes) | siehe SPEC 3.1 |
| **I-19** | **Chronik-Reinheit.** Chronik-Einträge enthalten keine `subject_ref`. Der Eintragstyp `Norm` und `Evidence` ist ein Typsystem-Argument (`ChronicleKind` ohne Betroffenenfeld). | T-CHR-1 |
| **I-20** | **Zeitreise-Gleichheit.** Gleiche (`valid_at`, `known_seq`, Ergebnisparameter) ergeben gleiche Chronik-Ergebnisse. | T-CHR-2 |
| **I-21** | **Horizont.** Vault-Versionen mit `superseded_ms < now − horizon` werden nicht mehr zurückgegeben (`HorizonExceeded`) und nach Hygiene gelöscht. | T-HOR-1 |
| **I-22** | **Keine Eigenbelege.** Acta hält keinen Signierschlüssel. Kein Ausgabewert ist durch einen Acta-Schlüssel signiert. | Architekturtest T-ARCH-9 |
| **I-23** | **Keine Nonce von außen.** Die AEAD-Nonce wird intern aus `Rng` erzeugt. Kein öffentlicher Parameter. | T-ARCH-10, T-SEAL-1 |
| **I-24** | **Chronik-Hashkette vollständig.** Beim Öffnen wird die Chronik von `genesis_chr` bis zum letzten Eintrag geprüft (9.4). Bruch → `Corrupt(ChronicleChain)`. | T-CHR-4 |

### 5.2 Verbotene Muster

Übernommen aus SPEC 3.2 (V-1 bis V-11) und Contextra:

| ID | Verbot | Durchsetzung |
|---|---|---|
| V-1 | `unwrap`, `expect`, `panic!`, `unreachable!`, `todo!`, `unimplemented!`, Indexzugriff in Produktionscode | Clippy `deny`; `unwrap-gate` aus Contextra |
| V-2 | `let _ =` auf `Result` | Clippy `let_underscore_must_use` = `deny` |
| V-3 | `from_utf8_lossy`, `unwrap_or_default` auf Fehlerpfaden, abschneidende `as`-Casts | `clippy.toml` `disallowed-methods`; `cast_possible_truncation = deny` |
| V-4 | Öffentliche Konstruktoren für Beweistypen | `trybuild` (T-TYPE-1) |
| V-5 | `Default`, `Clone`, `Copy`, `Display`, `Serialize` für Schlüsseltypen | T-TYPE-2 |
| V-6 | Typen gleichen Namens in zwei Crates | T-ARCH-3 |
| V-7 | `static` mit innenveränderlichem Zustand | T-ARCH-4 (`syn`-Scan) |
| V-8 | Cargo-Features, die Format, Durabilität, Verschlüsselung oder Löschverhalten ändern | T-ARCH-5 (Whitelist) |
| V-9 | Pfadabhängigkeiten außerhalb des Workspace, `[patch]`, Git-Abhängigkeiten | `cargo deny` |
| V-10 | Legacy-Pfade, `compat`-Module, Altformate | `grep legacy` in CI (T-ARCH-8) |
| V-11 | Mock im Datenpfad der Gates G1 und G2 | TR-1 |
| **V-12** | **Neu.** Standard-Features, die `workspace`, `test-utils` oder `exp-*` enthalten | T-ARCH-5 (Default-Liste leer oder nur `std`) |
| **V-13** | **Neu.** Konstanten als Ersatz für Hierarchiebestandteile (Contextra `dek.rs:76`, Befund A-3) | Architekturtest T-ARCH-11 (Suche nach `[0u8; N]` in Ableitungspfaden) |
| **V-14** | **Neu.** `CI || true` oder `continue-on-error` in Gate-Jobs | T-ARCH-12 (Workflow-Scan) |

---

## 6. Kryptografie

### 6.1 Primitive

| Zweck | Verfahren | Crate (Vorschlag, Version gepinnt) |
|---|---|---|
| AEAD | XChaCha20-Poly1305 (192-Bit-Nonce, 256-Bit-Schlüssel, 128-Bit-Tag) | RustCrypto `chacha20poly1305` |
| KDF | HKDF-SHA256 (RFC 5869) | `hkdf`, `sha2` |
| Hash | SHA-256 (Hashketten, Manifest, Belegdigest) | `sha2` |
| MAC | HMAC-SHA256 (Commitments) | `hmac` |
| Zufall | OS-CSPRNG | `getrandom` |
| Speicherlöschung | `zeroize` mit `ZeroizeOnDrop` | `zeroize` |

**Verpflichtung vor G4:** Versionen, Advisories, Prüfstatus (`cargo audit`, `cargo deny`). Nicht von mir ausgeführt.

**Ausschluss der Contextra-Primitive** (D-05): AES-256-GCM-SIV ist in `contextra-crypto` verwendet. Acta übernimmt die Crate nicht. Begründung: Nonce-Parameter von außen (I-23) und die Entscheidung für eine auditierbare Standardkonstruktion mit 192-Bit-Zufallsnonce.

### 6.2 Geheimnisse

| Name | Länge | Erzeugung | Ort | Lebensdauer |
|---|---|---|---|---|
| `TK_t` | 32 | CSPRNG | Custodian | bis Mandantenlöschung |
| `DK_s` | 32 | CSPRNG | Custodian | bis Betroffenenlöschung |
| `CK_t` | 32 | CSPRNG | Custodian | bis Mandantenlöschung (D-15) |
| `KC_e` | 32 | CSPRNG | Custodian | Epoche `e`, Vernichtung nach `commitment_linkability` (Standard 365 Tage) |
| `MK` | 32 | Aufrufer | nur RAM, Aufrufer | Prozesslaufzeit |
| `DEK` | 32 | HKDF, nie gespeichert | RAM, innerhalb **einer** Operation | sofort genullt |
| `CDEK` | 32 | HKDF, nie gespeichert | RAM, eine Chronik-Operation | sofort genullt |

Acta schreibt keinen dieser Schlüssel in das Datenverzeichnis (I-3). Keine Zwischenspeicherung von DEK (C-7).

### 6.3 Dokumentschlüssel (normativ, SPEC 4.4 unverändert)

```
IKM  = TK_t ‖ DK_s1 ‖ … ‖ DK_sn                  (n = 0..16; s_i aufsteigend nach subject_ref, bytewise)
PRK  = HKDF-Extract( salt = doc_id (16 B), ikm = IKM )
info = "ctx/v2/dek" ‖ 0x00 ‖ tenant_id (16) ‖ collection_id (16) ‖ n (1) ‖ s_1 ‖ … ‖ s_n ‖ generation (4, BE)
DEK  = HKDF-Expand( PRK, info, L = 32 )
```

Änderung gegenüber Contextra (`dek.rs:76`): Ein fehlender Hierarchiebestandteil führt zu Fehler, nicht zu Nullen. Ein Dokument ohne Betroffene hat `n = 0` und `IKM = TK_t`. Jeder Versuch, einen Bestandteil durch Konstanten zu ersetzen, ist ein Typfehler (V-13).

### 6.4 Chronik-Schlüssel

```
IKM_c = CK_t
PRK_c = HKDF-Extract( salt = entry_id (16 B), ikm = IKM_c )
info_c = "ctx/v2/chr-dek" ‖ 0x00 ‖ tenant_id (16) ‖ seq (8, BE)
CDEK  = HKDF-Expand( PRK_c, info_c, L = 32 )
```

Das Label `ctx/v2/chr-dek` ist eigenständig und unterscheidet Chronik-Envelopes von Dokument-Envelopes. Die Ableitungen teilen keine Eingabe-Kombination.

### 6.5 Envelope E1 (normativ, SPEC 4.5 unverändert, Erweiterung für Chronik)

Alle Zahlen big-endian.

| Offset | Länge | Feld | Regel |
|---|---|---|---|
| 0 | 4 | `magic` | `"CTX2"` |
| 4 | 1 | `version` | `0x01` |
| 5 | 1 | `alg` | `0x01` = XChaCha20-Poly1305 |
| 6 | 1 | `n_subjects` | 0..16 (für Chronik: 0) |
| 7 | 1 | `reserved` | `0x00` |
| 8 | 16 | `tenant_id` | |
| 24 | 16 | `collection_id` | Chronik: 16 × `0x00` |
| 40 | 16 | `doc_id` | Chronik: `entry_id` |
| 56 | 4 | `generation` | Chronik: `1` |
| 60 | 16 × n | `subject_refs` | streng aufsteigend, keine Duplikate |
| 60 + 16n | 24 | `nonce` | aus `Rng`, je Siegeln neu |
| 84 + 16n | 4 | `ct_len` | |
| 88 + 16n | `ct_len` + 16 | `ciphertext ‖ tag` | |

AAD = Bytes 0 bis einschließlich `ct_len`.

**Parserregeln** (jede Verletzung → `Corrupt`, nie Panic, nie stilles Kürzen):

1. Länge genau `88 + 16n + ct_len + 16`.
2. `ct_len ≤ max_doc_bytes + P1_OVERHEAD`, geprüft vor jeder Allokation.
3. `n ≤ 16`, `reserved = 0`, `version` und `alg` bekannt.
4. `subject_refs` streng aufsteigend.
5. Header-IDs stimmen mit Zeilenschlüssel überein (`RowMismatch`).
6. Fehlender Schlüssel beim Custodian → `KeyGone`, nicht `Corrupt`.
7. AEAD-Fehler bei vorhandenen Schlüsseln → `Corrupt(Auth)`, kein Klartext.
8. Fuzz-Ziel T-FUZZ-1.

### 6.6 Klartext P1 (SPEC 4.6 unverändert)

Deterministisch kodierte CBOR-Map (RFC 8949 §4.2): `"v"` (uint, `1`), `"text"` (tstr, ≤ `max_text_bytes`), `"meta"` (flache Map, ≤ 32 Einträge, Schlüssel ≤ 64 B, Wert tstr ≤ 1 KiB, int oder bool), `"emb"` (bstr, `4·dim` Byte, IEEE-754 binary32 little-endian).

`NaN` und `±Inf` sind beim Schreiben unzulässig. Der Nullvektor ist bei `Cosine` unzulässig. Abgeleitete Daten stehen nicht im Envelope.

### 6.7 Chronik-Payload-Klartext

Chronik-Payloads sind CBOR-Maps mit `"v"` (`1`), `"text"`, `"meta"`, nach denselben Regeln wie P1. Kein `"emb"`, kein `"subject"`-Schlüssel. Ein unbekannter Schlüssel → `Corrupt`.

### 6.8 Zufall

```rust
pub trait Rng: Send + Sync {
    fn fill(&self, dest: &mut [u8]) -> Result<(), RngError>;   // niemals Nullen bei Fehler
}
```

Produktion: `getrandom::fill`. Kein Wiederholungs-, Ersatz- oder Nullpfad. `Acta::open` akzeptiert einen `Rng` nur als Parameter der Konfiguration, und nur für Tests `#[cfg(test)]` oder Testkit (Befund B-09).

### 6.9 Kryptoregeln

| ID | Regel |
|---|---|
| **C-1** | Nur XChaCha20-Poly1305 siegelt. Kein XOR, keine Eigenentwicklung. |
| **C-2** | Jeder Siegelvorgang nutzt frische 192-Bit-Zufallsnonce und authentisiert den Header als AAD. |
| **C-3** | Jedes Geheimnis stammt aus dem CSPRNG. Keine Konstanten als Schlüssel, keine Ableitung aus Tags, Pfaden oder Zeit. |
| **C-4** | Kein Fallbackpfad bei Schlüsselfehlern. |
| **C-5** | Jede Funktion im Krypto-Pfad liefert `Result`. |
| **C-6** | Domänentrennung über Labels (Anhang A.2). Jedes Label genau einem Zweck. |
| **C-7** | DEK und CDEK werden nicht zwischengespeichert und nach Operation genullt. |
| **C-8** | Format in Anhang B mit unabhängigem Code reproduzierbar (T-SEAL-1). |
| **C-9** | **Neu.** Kein Hilfsfunktion-Pfad, der Klartext ohne Authentisierung zurückgibt. Auch nicht für Tests in Produktionsmodulen. |

### 6.10 Umgang mit Geheimnissen im RAM

- Typen `Secret<N>` mit `Zeroize + ZeroizeOnDrop`.
- Kein `Clone`, `Copy`, `Default`, `Display`, `Serialize`, `PartialEq`. `Debug` gibt `[REDACTED]` aus (T-TYPE-2).
- HKDF-PRK, IKM, Klartext-Puffer werden vor Verlassen des Gültigkeitsbereichs genullt.
- Klartext verlässt den Prozess nur über die API-Rückgabe (R6).

### 6.11 Fehlerabbildung im Schlüsselpfad

| Situation | Ergebnis | Nie |
|---|---|---|
| Custodian meldet `NotFound`/`Disabled` für benötigten Schlüssel | `KeyGone` | Ersatzschlüssel, Cache, Wiederholung |
| Custodian nicht erreichbar, Timeout, Protokollfehler | `CustodianUnavailable` | Deutung als `KeyGone` oder Erfolg |
| `Rng` scheitert | `Rng` | Nullen, Zeit-Seed |
| HKDF nicht berechenbar (bei L = 32 praktisch ausgeschlossen) | `Crypto` | Nullschlüssel |
| AEAD-Siegeln scheitert | `Crypto` | leerer Envelope (Befund V-C1) |
| AEAD-Öffnen scheitert | `Corrupt(Auth)` | Teilklartext |
| Gate „nicht aktiv“ | `Erased` | Klartext |

### 6.12 Nonce-Sicherheit

Nonce: 192 Bit aus CSPRNG. Kollisionswahrscheinlichkeit nach `q` Siegelungen unter demselben DEK ≈ `q² / 2¹⁹³`. Ein Retry nach Fehler erzeugt eine neue Nonce. Praktisch gilt `q ≤ 2` je DEK.

### 6.13 Prüfliste für die externe Kryptoprüfung (G4)

Übernommen aus SPEC 4.12, ergänzt um:

11. Trennung der Labels `ctx/v2/dek` und `ctx/v2/chr-dek` (6.4): Keine Eingabe lässt sich in beiden Ableitungen gleichen.
12. Wiederverwendung der `"CTX2"`-Kopfzeile für Chronik-Envelopes: Ist eine Verwechslung ausgeschlossen, weil die DEK-Ableitung verschieden ist?
13. Commitment HMAC (4.3) über `tenant_id ‖ subject_ref`: Enumeration bei bekanntem `KC_e` möglich? (Erwartung: ja, solange `KC_e` existiert; daher Vernichtung nach Linkability-Frist.)

### 6.14 Grenzen

Verborgen: Klartext, Metadaten in P1 und Chronik-Payloads, Embeddings. Nicht verborgen: `tenant_id`, `collection_id`, `doc_id`, `subject_refs`, `generation`, Größen, Zeitstempel, Zugriffsmuster (R4). Chronik-Metadaten (`kind`, `valid_from`, `recorded_ms`, `matter`) sind im Klartext (R20).

---

## 7. Custodian (Schlüsseldienst)

### 7.1 Schnittstelle (normativ, SPEC 5.2 mit Erweiterung)

```rust
pub struct KeyId(String);   // Namensraum 7.2, Zeichen [a-z0-9/], Länge ≤ 160, nur über Konstruktoren

pub enum KeyLookup { Present(Secret<32>), Gone }        // Gone = NotFound | Disabled
pub enum KeyStatus { Present, Disabled, Destroyed, Unknown }

pub enum CustodianError {
    NotFound, Disabled,        // nur get()
    Conflict,                  // put() auf vorhandene KeyId
    Unavailable, Denied,
    Protocol(u16),             // 1 = AuthFail (Anhang A)
}

pub struct DestroyReceipt {
    pub custodian_ref: String,
    pub custodian_time_ms: Option<u64>,
    pub scheduled_final_ms: Option<u64>,   // Karenz
    pub raw: Vec<u8>,                      // unverändert gespeichert, nicht interpretiert (A-2)
}

pub enum AttestationClass { A0, A1, A2 }

pub trait Custodian: Send + Sync {
    fn put(&self, id: &KeyId, secret: &Secret<32>) -> Result<(), CustodianError>;
    fn get(&self, id: &KeyId) -> Result<Secret<32>, CustodianError>;
    fn get_many(&self, ids: &[KeyId]) -> Result<Vec<KeyLookup>, CustodianError>;   // Reihenfolge wie ids
    fn destroy(&self, id: &KeyId) -> Result<DestroyReceipt, CustodianError>;      // idempotent
    fn status(&self, id: &KeyId) -> Result<KeyStatus, CustodianError>;
    fn attestation_class(&self) -> AttestationClass;
}
```

### 7.2 Namensraum

```
Mandant:        ctx2/<store_id hex32>/t/<tenant_id hex32>
Betroffener:    ctx2/<store_id hex32>/s/<tenant_id hex32>/<subject_ref hex32>
Chronik-Schl.:  ctx2/<store_id hex32>/c/<tenant_id hex32>
Commitment-Ep.: ctx2/<store_id hex32>/k/<epoch hex8>
```

### 7.3 Vertragsregeln (SPEC 5.3 unverändert)

K-1 `put` nur anlegen. K-2 `get` liefert `NotFound`/`Disabled` nur bei fehlendem Schlüssel; jeder andere Fehler ist `Unavailable`/`Denied`/`Protocol`. K-3 `get_many` alles oder nichts. K-4 `destroy` idempotent, unbekannte ID → `NotFound`. K-5 nach `destroy`: `get` = `NotFound`/`Disabled`, `status` ∈ {`Destroyed`, `Disabled`}. K-6 Karenzfrist mit `scheduled_final_ms`, ausgewiesen (A-3). K-7 kein `Secret` in Logs. K-8 zeitbegrenzte Aufrufe, kein Abbruch.

### 7.4 Anlegen und Adoption (SPEC 5.5 unverändert)

1. `status(id)`: `Destroyed`/`Disabled` → `Erased` (I-10).
2. `Present` und Datenbank unbekannt → Waise (CP-R2), **adoptieren**.
3. `Unknown` → `Rng` → `put` → Commit.

### 7.5 `acta-custodian-local` (Klasse A0, Entwicklung und Einzelplatz)

Crate auf E3. Wie SPEC 5.6, mit folgenden Präzisierungen:

- `MK` kommt vom Aufrufer (Keyring, Hardware oder Passphrase-Ableitung im **eigenen** Code). Acta implementiert keine Passphrase-Ableitung (Contextra-Crate `argon2` bleibt außerhalb).
- Verzeichnis außerhalb des Datenverzeichnisses, kanonische Pfadprüfung.
- Eintrag `<sha256(key_id) hex>.k`: `version(1) ‖ nonce(24) ‖ XChaCha20-Poly1305(MK, nonce, aad = "ctx2/local/" ‖ key_id, secret)`.
- `destroy`: Zufallsüberschreiben, `fsync`, `unlink`, Verzeichnis-`fsync`, Marker `<hash>.gone`, `fsync`.
- **Klasse A0.** Der Nachweis enthält keine Löschaussage.

### 7.6 Externe Adapter

Eigene Crates `acta-custodian-<name>` außerhalb des Workspace-Kerns (V-9 mit Begründung im ADR). Konformitätssuite T-CUST-1 (`acta-conformance`) ist Pflicht. Erster produktiver Adapter (O3): Key-Dienst mit Karenz, signiertem Beleg und Audit-Log (Klasse A2 nach 17.4).

---

## 8. Speicher (SQLite)

### 8.1 Prinzip

Der Speicher sieht nur Envelopes und Metadaten. SQLite wird verwendet, weil die Durabilitätsgarantien dort geprüft sind. Eine eigene Engine ist Nicht-Ziel (D-02).

### 8.2 Pragmas (verbindlich, SPEC 6.2)

| Pragma | Wert | Zweck |
|---|---|---|
| `journal_mode` | `WAL` | atomare Commits |
| `synchronous` | `FULL` | Commit nach `fsync` |
| `fullfsync`, `checkpoint_fullfsync` | `ON` | `F_FULLFSYNC` auf macOS |
| `secure_delete` | `ON` | Hygiene, keine Zusage |
| `locking_mode` | `EXCLUSIVE` | ein Prozess |
| `mmap_size` | `0` | kein `mmap` (D3) |
| `foreign_keys` | `ON` | Referenzen (O9) |
| `trusted_schema` | `OFF` | keine fremden Funktionen |
| `cell_size_check` | `ON` | frühe Beschädigungserkennung |
| `application_id` | `0x43545832` (`CTX2`) | Dateikennung |
| `user_version` | `2` | Schema; abweichend → `Corrupt(Schema)` |

Die Bibliothek ist `bundled` mit gepinnter Version. `#![forbid(unsafe_code)]` gilt für alle Acta-Crates, SQLite-C bleibt Restrisiko R15.

### 8.3 Schema (normativ, Version 2)

Basis: SPEC 6.3. Änderungen sind **kursiv**, begründet durch P1 (eine Wahrheit pro Datenklasse) und Zeitreise.

```sql
CREATE TABLE meta (
  key TEXT PRIMARY KEY, value BLOB NOT NULL
) STRICT;                                       -- 'store_id', 'created_ms', 'schema' = 2, 'commit_seq'

CREATE TABLE tenants (
  tenant_id BLOB PRIMARY KEY CHECK (length(tenant_id) = 16),
  state     INTEGER NOT NULL CHECK (state IN (0,1,2,3)),   -- Active, Gated, Destroyed, Completed
  erase_id  BLOB CHECK (erase_id IS NULL OR length(erase_id) = 16),
  receipt   BLOB,
  pending   INTEGER
) STRICT, WITHOUT ROWID;

CREATE TABLE subjects (
  tenant_id   BLOB NOT NULL CHECK (length(tenant_id) = 16),
  subject_ref BLOB NOT NULL CHECK (length(subject_ref) = 16),
  state       INTEGER NOT NULL CHECK (state IN (0,1,2,3)),
  erase_id    BLOB CHECK (erase_id IS NULL OR length(erase_id) = 16),
  receipt     BLOB,
  pending     INTEGER,
  PRIMARY KEY (tenant_id, subject_ref)
) STRICT, WITHOUT ROWID;

CREATE TABLE collections (
  tenant_id     BLOB NOT NULL CHECK (length(tenant_id) = 16),
  collection_id BLOB NOT NULL CHECK (length(collection_id) = 16),
  name          TEXT NOT NULL,
  dim           INTEGER NOT NULL CHECK (dim BETWEEN 0 AND 4096),
  metric        INTEGER NOT NULL CHECK (metric IN (0,1)),
  analyzer      TEXT NOT NULL,
  filterable    BLOB NOT NULL,                    -- CBOR-Array der Meta-Schlüssel
  PRIMARY KEY (tenant_id, collection_id),
  UNIQUE (tenant_id, name)
) STRICT, WITHOUT ROWID;

-- Kopf je Dokument (ohne Envelope: Envelope nur in doc_versions, P1)
CREATE TABLE docs (
  tenant_id       BLOB NOT NULL CHECK (length(tenant_id) = 16),
  collection_id   BLOB NOT NULL CHECK (length(collection_id) = 16),
  doc_id          BLOB NOT NULL CHECK (length(doc_id) = 16),
  current_gen     INTEGER NOT NULL CHECK (current_gen >= 1),
  tombstone       INTEGER NOT NULL DEFAULT 0 CHECK (tombstone IN (0,1)),
  created_ms      INTEGER NOT NULL,
  updated_ms      INTEGER NOT NULL,
  PRIMARY KEY (tenant_id, collection_id, doc_id)
) STRICT, WITHOUT ROWID;

-- *NEU:* Versionen innerhalb Horizont (Zeitreise Ebene P)
CREATE TABLE doc_versions (
  tenant_id     BLOB NOT NULL,
  collection_id BLOB NOT NULL,
  doc_id        BLOB NOT NULL,
  generation    INTEGER NOT NULL CHECK (generation >= 1),
  created_ms    INTEGER NOT NULL,                  -- Transaktionszeit: ab wann gültig
  superseded_ms INTEGER,                           -- NULL = aktuell
  envelope      BLOB NOT NULL,
  PRIMARY KEY (tenant_id, collection_id, doc_id, generation)
) STRICT, WITHOUT ROWID;
CREATE INDEX doc_versions_by_time ON doc_versions (tenant_id, collection_id, doc_id, created_ms);

CREATE TABLE doc_subjects (
  tenant_id     BLOB NOT NULL,
  collection_id BLOB NOT NULL,
  doc_id        BLOB NOT NULL,
  subject_ref   BLOB NOT NULL,
  PRIMARY KEY (tenant_id, collection_id, doc_id, subject_ref)
) STRICT, WITHOUT ROWID;
CREATE INDEX doc_subjects_by_subject ON doc_subjects (tenant_id, subject_ref, collection_id, doc_id);

-- Löschlog: append-only, Hashkette (SPEC 6.6)
CREATE TABLE erase_log (
  seq          INTEGER PRIMARY KEY,               -- lückenlos ab 1
  kind         INTEGER NOT NULL CHECK (kind IN (1,2,3,4)),   -- GATED, DESTROYED, COMPLETED, RECONCILED
  erase_id     BLOB NOT NULL CHECK (length(erase_id) = 16),
  target_kind  INTEGER NOT NULL CHECK (target_kind IN (1,2)),
  tenant_id    BLOB NOT NULL CHECK (length(tenant_id) = 16),
  subject_ref  BLOB NOT NULL CHECK (length(subject_ref) = 16),
  ts_ms        INTEGER NOT NULL,
  reason       INTEGER NOT NULL,
  aux          BLOB NOT NULL CHECK (length(aux) = 32),
  prev_hash    BLOB NOT NULL CHECK (length(prev_hash) = 32),
  entry_hash   BLOB NOT NULL CHECK (length(entry_hash) = 32)
) STRICT;
CREATE TRIGGER erase_log_no_update BEFORE UPDATE ON erase_log BEGIN SELECT RAISE(ABORT, 'append-only'); END;
CREATE TRIGGER erase_log_no_delete BEFORE DELETE ON erase_log BEGIN SELECT RAISE(ABORT, 'append-only'); END;

CREATE TABLE erase_manifest (
  erase_id        BLOB NOT NULL CHECK (length(erase_id) = 16),
  doc_id          BLOB NOT NULL CHECK (length(doc_id) = 16),
  collection_id   BLOB NOT NULL CHECK (length(collection_id) = 16),
  generation      INTEGER NOT NULL,                -- *NEU:* Versionen werden einzeln gelistet
  envelope_sha256 BLOB NOT NULL CHECK (length(envelope_sha256) = 32),
  PRIMARY KEY (erase_id, collection_id, doc_id, generation)
) STRICT, WITHOUT ROWID;

-- *NEU:* Verknüpfung Löschung → Chronik-Commitment (idempotent, Schlüssel erase_id)
CREATE TABLE erase_chronicle_link (
  erase_id      BLOB PRIMARY KEY CHECK (length(erase_id) = 16),
  chronicle_seq INTEGER NOT NULL
) STRICT, WITHOUT ROWID;

-- *NEU:* Ebene N, bitemporal, Hashkette, append-only
CREATE TABLE chronicle (
  seq          INTEGER PRIMARY KEY,                -- lückenlos ab 1, = commit_seq des Eintrags
  entry_id     BLOB NOT NULL UNIQUE CHECK (length(entry_id) = 16),
  kind         INTEGER NOT NULL CHECK (kind IN (1,2,3,4,5,6,7,8)),
                -- 1 NormVersion, 2 PolicyVersion, 3 ConfigState, 4 ModelVersion,
                -- 5 DecisionRule, 6 ErasureCommitment, 7 ContextReceipt, 8 Checkpoint
  scope        INTEGER NOT NULL CHECK (scope IN (0,1)),         -- 0 Global, 1 Tenant
  tenant_id    BLOB CHECK (tenant_id IS NULL OR length(tenant_id) = 16),
  matter       TEXT NOT NULL,                                    -- z. B. "DSGVO/Art.17"
  valid_from   INTEGER NOT NULL,                                 -- Tage seit 1970-01-01
  valid_to     INTEGER NOT NULL,                                 -- halboffen; 2147483647 = offen
  recorded_ms  INTEGER NOT NULL,
  content_hash BLOB NOT NULL CHECK (length(content_hash) = 32),
  payload      BLOB,                                             -- E1 unter CDEK; NULL bei kind 6, 8
  supersedes   INTEGER REFERENCES chronicle(seq),
  prev_hash    BLOB NOT NULL CHECK (length(prev_hash) = 32),
  entry_hash   BLOB NOT NULL CHECK (length(entry_hash) = 32)
) STRICT;
CREATE TRIGGER chronicle_no_update BEFORE UPDATE ON chronicle BEGIN SELECT RAISE(ABORT, 'append-only'); END;
CREATE TRIGGER chronicle_no_delete BEFORE DELETE ON chronicle BEGIN SELECT RAISE(ABORT, 'append-only'); END;
CREATE INDEX chronicle_by_matter ON chronicle (scope, matter, valid_from, recorded_ms);
```

Hinweis zu Triggern: Sie schützen gegen Programmierfehler, nicht gegen einen Angreifer mit Schreibzugriff (W5).

### 8.4 Durabilitätsregeln (SPEC 6.4 unverändert)

D1 Mutation gilt erst nach `COMMIT` mit `synchronous=FULL`; danach RAM-Sichtbarkeit und Bestätigung. D2 `fsync`-/`SQLITE_IOERR`-Fehler → Handle `Poisoned`, nie Wiederholung auf demselben Handle („fsyncgate“), Recovery beim nächsten Öffnen. D3 kein `mmap`. D4 Linux (ext4, XFS), macOS (APFS); Netz-, FUSE-, Overlay-Dateisysteme abgelehnt. D5 Stromausfall: SQLite-/OS-Garantie; Tests nur Prozessabbrüche. D6 `wal_checkpoint(TRUNCATE)` nach Purge als Hygiene. D7 Acta nutzt weder `VACUUM INTO` noch Backup-API.

### 8.5 Löschen und Hygiene

Nach Purge: `DELETE` der Zeilen, `secure_delete` überschreibt freigegebene Seiten (Hygiene, keine Zusage). Reste im WAL oder in Freiseiten sind Ciphertext unter Schlüsseln, die vernichtet sind (R12).

### 8.6 Zeitstempel

`recorded_ms` aus `Clock` (Unix-Millisekunden). `commit_seq` aus `meta`, monoton, in derselben Transaktion erhöht. Keine Annahme über Wanduhr-Monotonie.

---

## 9. Zeit und Versionierung

### 9.1 Chronik (Ebene N)

Eintragstypen, Zeitachsen und Hashkette siehe 8.3 (Tabelle `chronicle`).

Zwei Zeitachsen:

| Achse | Bedeutung | Quelle |
|---|---|---|
| `valid` = `[valid_from, valid_to)` | Geltungszeit in der Welt | Erfasser (Tage seit Epoche) |
| `recorded` = `seq` + `recorded_ms` | Wissenszeit | Acta (`commit_seq`, `Clock`) |

**Korrekturen überschreiben nie.** Neuer Eintrag mit `supersedes`. Die Abfrage „was wussten wir am T über Gültigkeit am V“ wählt je Matter den Eintrag mit größtem `seq ≤ known_seq`, dessen `[valid_from, valid_to)` `V` enthält, und folgt der `supersedes`-Kette nur, wenn der Nachfolger ebenfalls `seq ≤ known_seq` hat.

### 9.2 Hashkette und Checkpoints

```
genesis_chr = SHA-256("ctx/v2/chr-genesis" ‖ 0x00 ‖ store_id)
entry_hash  = SHA-256("ctx/v2/chr" ‖ 0x00 ‖ seq (8) ‖ kind (1) ‖ valid_from (4, signed BE) ‖ valid_to (4, signed BE)
                      ‖ recorded_ms (8) ‖ content_hash (32) ‖ len(matter) (2) ‖ matter ‖ supersedes_entry_id (16, 0 = keiner) ‖ prev_hash (32))
prev_hash   = entry_hash des Vorgängers; beim ersten Eintrag genesis_chr
```

Checkpoint (kind 8) alle `chronicle_checkpoint_every` Einträge (Standard 1024): `content_hash` = Merkle-Wurzel über `entry_hash` der Periode. **Ohne Signatur** (D-09). Externe Verankerung über Port `Notary` ab 1.1.

Die Kette wird beim Öffnen vollständig geprüft (I-24). Die Grenze entspricht 6.7 in SPEC: Wer die Kette neu berechnen kann, manipuliert unbemerkt. Gegenmittel sind Checkpoints außerhalb von Domäne B (Notary, 1.1) und das Löschregister.

### 9.3 Tresor (Ebene P): Horizont

| Klasse | Horizont (Standard) | Wirkung |
|---|---|---|
| `Personal` + `Contract` | `history_horizon_days` = 30 | Versionen älter als 30 Tage werden in Hygiene gelöscht |
| `Personal` + `LegalObligation{retain_until}` | bis `retain_until` | Speicherbegrenzung, kein Sweeper in v1 (Restrisiko) |
| `Special` | 0 | nur aktuelle Version, alte sofort in Purge |

Die Hygiene-Löschung von Versionen ist **kein** Teil der Löschung; sie verkürzt nur Historie. Eine Zeitreise hinter den Horizont liefert `HorizonExceeded`.

### 9.4 AsOf-Semantik (normativ)

Abfrage `AsOf { valid_at, known_at }` (Transaktionszeit `known_at` in Millisekunden, oder `known_seq`):

| Ziel | Ergebnis |
|---|---|
| Chronik-Eintrag | `Found` nach 9.1, sonst `NotFound` |
| Tresor, lebendig, Version im Horizont | `Found` (Version mit `created_ms ≤ known_at < superseded_ms`) |
| Tresor, lebendig, Version außerhalb Horizont | `HorizonExceeded` |
| Tresor, Betroffener oder Mandant `Gated`/`Destroyed`/`Completed` | `Erased` (**Erase gewinnt**, unabhängig von `known_at`) |
| Ableitung (RAM-Index) | nicht eigenständig; wird aus Tresor-Versionen zum `known_at` aufgebaut oder entfällt |

Außen (Rolle `Reader`) wird `Erased` als `NotFound` gemeldet. Nur `Auditor` und `Controller` unterscheiden (ACTA 8.3).

### 9.5 Berichtigung

`rectify(doc, new_input)` = `update` mit neuer Generation. Die alte Version bleibt im Horizont lesbar (Audit) und wird bei Löschung des Betroffenen mit vernichtet (gleiche DEK-Bindung über `subject_refs`). Art. 16 DSGVO ist damit abgedeckt, solange die Version innerhalb des Horizonts liegt.

---

## 10. Datenpfad

### 10.1 Anlegen

- `create_tenant(tenant_id)`: `TK_t` anlegen/adoptieren, `CK_t` anlegen, Zeile `tenants(Active)`, Commit. Ein `tenant_id` aus 16 Nullbytes → `InvalidInput`.
- `create_collection(tenant, name, spec)`: `collection_id` Zufall; `spec` unveränderlich.
- `register_subject(tenant, subject_ref)`: `DK_s` anlegen/adoptieren, Zeile `subjects(Active)`, Commit. Gelöschter `subject_ref` → `Erased` (I-10).

### 10.2 Schreiben (`put`, `update`, `rectify`)

Eingabe `DocInput { doc_id, subjects: Subjects, text, meta, embedding }`, `Subjects::Personal(&[SubjectRef])` (1..=16 verschiedene) oder `Subjects::NonPersonal`.

| Schritt | Aktion | Fehler |
|---|---|---|
| P0 | Validierung: Limits, `NaN`/`Inf`, Nullvektor, verschiedene Betroffene | `InvalidInput`, `Limit(..)` |
| P1 | Mandant `Active`; alle Betroffenen `Active` und im Schlüsselcache | `Erased`, `UnknownSubject`, `UnknownTenant` |
| P2 | `generation` (1 bei `put`, +1 bei `update`), DEK (6.3), Nonce aus `Rng`, Envelope E1, DEK nullen | `Rng`, `Crypto` |
| P3 | Transaktion: `docs` upsert, `doc_versions` einfügen, bei `update` alte Version `superseded_ms` setzen, `doc_subjects` ersetzen; COMMIT durabel | `Io(Poisoned)` |
| P4 | RAM-Index aktualisieren; Klartextpuffer nullen | — |
| P5 | Rückgabe `DocVersion { generation }` | — |

- `put` auf vorhandene `doc_id` → `Exists`. `update` auf nicht vorhandene → `NotFound`.
- Klartext wird nie geschrieben (I-3, Z2).
- Ältere Versionen unter denselben Schlüsseln bis Löschung (R12).

### 10.3 Chronik-Schreiben (`chronicle_append`)

Eingabe `ChronicleInput { kind, scope, matter, valid, content_text, meta, supersedes }` mit `ClassAssertion::NonPersonal` (Pflicht, kein Standardwert).

| Schritt | Aktion |
|---|---|
| C0 | Validierung: `matter` ≤ 128 Byte, Gültigkeit `valid_from < valid_to`, Payload ≤ `max_text_bytes`, `supersedes` existiert und gehört zum gleichen Matter |
| C1 | `commit_seq` erhöhen, `recorded_ms` aus `Clock`, `entry_id` aus `Rng` |
| C2 | `content_hash = SHA-256(canonical(content_text, meta))` |
| C3 | Payload als E1 unter CDEK (6.4), `CK_t` aus Schlüsselcache |
| C4 | `entry_hash` (9.2), Zeile einfügen, COMMIT durabel |
| C5 | Checkpoint anlegen, falls `seq mod chronicle_checkpoint_every = 0` (gleiche Transaktion) |

Fehler: `Crypto`/`Rng` → Abbruch ohne Zeile; `Io(Poisoned)` → Handle vergiftet (D2).

### 10.4 Lesen (`get`, `chronicle_get`)

`get` folgt SPEC 7.3 (zwei Gate-Prüfungen, Header-Abgleich, Schlüsselcache). `chronicle_get` entschlüsselt nur mit `CK_t`; nach Mandantenlöschung liefert es `Erased`, Einträge ohne Payload (kind 6, 8) bleiben lesbar.

### 10.5 Tombstone

`tombstone(doc)` setzt `docs.tombstone = 1` und entfernt aus dem RAM-Index. **Keine Löschaussage.** Dokumentweises kryptografisches Löschen erreicht der Aufrufer über eigenen `subject_ref` (SPEC 7.4).

### 10.6 Ersetzen und Löschen (`replace_and_erase`)

SPEC 7.5 unverändert: Ersatzdokumente zuerst (durabel), dann `erase(B)`. Absturz dazwischen → Wiederholung mit derselben `erase_id`.

---

## 11. Suche (RAM-Index)

### 11.1 Prinzip

Kein persistierter Index. Beim Öffnen werden Envelopes nicht gesperrter Ziele entschlüsselt und Indizes gebaut. Klartext wird nicht gehalten, nur Terme, Vektoren, Filterwerte (SPEC 8.1).

Begründung gegenüber ACTA 6.3 (`KvPort`, persistierte Ableitungen): Eine persistierte Ableitung ist nach Löschung zu reparieren oder zu purgen. Das ist die Quelle der Komplexität in Contextra (`kv_shredding`, `purge/phase.rs`, `compat/purge.rs` in Text und Vektor). Der RAM-Index vermeidet diese Klasse.

### 11.2 Analyzer `a1` (SPEC 8.3)

NFKC, UAX #29, Kleinschreibung, Token > 64 Byte nicht indiziert (im Report ausgewiesen). Variante `a1-de`: Snowball-Stemmer Deutsch. Keine Stoppwörter, keine Dekomposition, kein Präfix/Regex.

Übernahme aus Contextra: `contextra-text/src/tokenizer.rs` nach Entfernung der Morphologie-Pfade (D-19).

### 11.3 BM25 (SPEC 8.4)

`idf(t) = ln(1 + (N − n_t + 0.5)/(n_t + 0.5))`, `k1 = 1.2`, `b = 0.75`. Anfrageterme deduplizieren, lexikografisch sortieren, Summe in `f64`. Sortierung: Score absteigend (`total_cmp`), dann `doc_id` aufsteigend.

Übernahme aus Contextra: `contextra-text/src/bm25.rs` als Muster für Postings und Score-Struktur. Formel und Reihenfolge werden gegen T-IDX-1 neu geprüft.

### 11.4 Vektorsuche (SPEC 8.5)

Exakt, `Cosine` und `Dot`. Skalarprodukt mit 8 festen Akkumulatoren (`acc[i mod 8]`), Reduktion `((a0+a1)+(a2+a3))+((a4+a5)+(a6+a7))`. Kein SIMD (D-11).

Übernahme: Die Reihenfolge der Akkumulatoren ist neu. `contextra-vector/src/distance/scalar.rs` wird nicht übernommen, weil seine Reduktion nicht spezifiziert ist (Prüfung in AP4).

### 11.5 RRF (SPEC 8.6)

`rrf(d) = Σ_r 1/(60 + rank_r(d))`, Rang ab 1, Summation in fester Reihenfolge (Text, Vektor). Übernahme: `contextra-rank/src/fusion/rrf.rs` (753 Zeilen, davon viele Testhilfen), auf Kern reduziert.

### 11.6 Filter (SPEC 8.7)

`Eq`, `In`, `And`. Nur Schlüssel aus `filterable`. Vorfilter als Bitmap-Schnitt. Höchstens 1024 Werte je Schlüssel.

### 11.7 Gate im Suchpfad (SPEC 8.8)

Kandidatenmenge schließt gesperrte Ziele und Tombstones vor dem Scoring aus. Vor Rückgabe erneute Gate-Prüfung je Treffer. Ergebnis darf kürzer als `k` sein. Kein `exhausted`-Feld (I-15).

### 11.8 Limits (SPEC 8.9)

`k ≤ 1000`, Anfrage ≤ 4 KiB, `dim ≤ 4096`, `max_index_bytes`: Überschreitung beim Öffnen → `Limit(IndexBytes)`, nie Speicherüberlauf.

### 11.9 Entfernen bei Löschung (SPEC 8.10)

Dokumente des Ziels aus Katalog und Postings, Vektorzeilen genullt, Freiliste, Neubau bei > 25 % freien Zeilen.

### 11.10 Skalierung [Ziel, ungemessen]

Aufbau O(N). Obergrenze je Prozess wird in G3 gemessen. Rechenbeispiel: 10⁵ × 768 × 4 B ≈ 307 MB Vektoren. Snapshots unter Epochenschlüssel nur bei gemessenem Bedarf (O4).

---

## 12. Löschprotokoll

### 12.1 Zustände (pur, in `acta-model`)

```
Active ──► Gated ──► Destroyed ──► Completed          (Completed ist terminal, I-10)
```

Zusätzlich `pending: Option<PendingReason>` mit `CustodianUnavailable`, `CustodianInconsistent`, `Io`, `PurgeFailed`, `ErasePastSla`. `pending` ist keine Zustandsachse, sondern die Ursache, warum der nächste Übergang nicht erfolgt ist (D-07).

```rust
pub enum Event { Gate, Destroy, Purged, Reconcile(State) }
pub fn next(state: State, ev: Event) -> Result<State, TransitionError>;   // rein, exhaustiv
```

Proptest: keine unerlaubte Kante, Zustände nie zurück, `Completed` nur über `Destroyed`. Der Zustandsautomat ist die einzige Quelle für Übergänge. Das ist ACTA 7.4 mit der Einschränkung auf vier Zustände.

### 12.2 Zusage je Zustand

| Zustand | Z1 (kein Klartext) | Z2 (Schlüssel vernichtet) | Nachweis |
|---|---|---|---|
| `Active` | nein | nein | — |
| `Gated` | **ja** | nein | `erase_log` GATED |
| `Destroyed` | **ja** | **ja** (bei Karenz: `Disabled`, ausgewiesen) | Beleg + `erase_log` DESTROYED |
| `Completed` | ja | ja | Beleg + Manifest + `erase_log` COMPLETED + Chronik-Commitment |

### 12.3 Gate-Protokoll und Lesegarantie

- **Setzen (E1 bis E2b) unter Schreibsperre W**: E1 Commit durabel, E2a RAM-Gate-Flag (`Release`), E2b Index entfernen, Schlüssel nullen. W wird zwischen E1 und E2b nicht freigegeben.
- **Lesen:** `load(Acquire)` vor Entschlüsselung und nach Entschlüsselung unmittelbar vor Rückgabe (Linearisierungspunkt).
- **Z1-Aussage:** Beginnt die zweite Prüfung nach E2a, liefert sie „nicht aktiv“. Ein Lesevorgang, dessen zweite Prüfung vor E2a lag, darf noch Klartext liefern; dieser liegt beim Aufrufer (R6).

### 12.4 `erase(Subject)` (normativ)

| Schritt | Aktion | Persistenz | CP |
|---|---|---|---|
| **E0** | Validierung. `erase_id` (vom Aufrufer oder `Rng`). Bereits Gated/Destroyed/Completed → Handle des bestehenden Vorgangs. Gleiche `erase_id` auf anderem Ziel → `InvalidInput`. | — | CP-E1 |
| **E0R** | Falls Register konfiguriert: `registry.record(entry)`, Fehler → `Registry`, Zustand unverändert | Register (außerhalb B) | CP-E1R |
| **E1** | **GATE:** Transaktion `state=Gated`, `erase_id`, Log `GATED`; COMMIT durabel | `W` | CP-E2, CP-E3 |
| **E2a** | RAM-Gate-Flag setzen (`Release`) | `W` | CP-E4 |
| **E2b** | Index-Dokumente des Ziels entfernen und nullen; `DK_s` aus Cache nullen | `W` → `S` | CP-E5 |
| *(Modus Gated: Rückkehr)* | | | |
| **E3** | `custodian.destroy(id)`. Fehler → `pending=CustodianUnavailable`/`CustodianInconsistent`. Kein Lock gehalten. | — | CP-E6 |
| **E4** | Verifikation: `status` ∈ {Destroyed, Disabled}; Probe `get` = `NotFound`/`Disabled`; falls Dokumente existieren: Probeöffnen eines Envelopes mit `KeyGone`-Ergebnis (Entscheidung O17). Fehlschlag → `pending=CustodianInconsistent`, Zustand bleibt Gated (I-7) | — | CP-E7 |
| **E5** | Transaktion `state=Destroyed`, `receipt`, Log `DESTROYED` (aux = SHA-256 des Belegs); COMMIT | `W` | CP-E8, CP-E9 |
| *(Modus Destroyed: Rückkehr)* | | | |
| **E6** | Purge in Chunks (≤ `purge_chunk_docs`, 1000): je Chunk `erase_manifest`-Zeilen (`INSERT OR IGNORE`) für **alle** Versionen (`doc_versions`), dann `doc_versions`, `doc_subjects`, `docs`-Zeilen der Dokumente löschen (bei mehrpersonigen Dokumenten alle Zeilen); COMMIT | `W` je Chunk | CP-E10 |
| **E7** | `wal_checkpoint(TRUNCATE)`; Fehler nicht fatal, Health-Hinweis | — | CP-E11 |
| **E8** | `manifest_root` (12.6). Transaktion `state=Completed`, Log `COMPLETED` (aux = Wurzel) | `W` | CP-E12 |
| **E9** | **Neu.** Chronik-Commitment: `chronicle_append(kind 6, scope Global, ClassAssertion::NonPersonal)` mit `content_hash` aus 12.6 und `erase_chronicle_link`-Zeile in derselben Transaktion | `W` | CP-E13 |

**Reihenfolge-Invarianten:** Gate vor Destroy (I-6); Destroyed nach Gegenprobe (I-7); Completed nach Destroyed und vollständigem Purge (I-8); Beleg vor `DESTROYED` in derselben Transaktion; Chronik-Commitment nach `COMPLETED` (E9 ist Idempotenzpunkt via `erase_chronicle_link`).

**Idempotenz:** E3, E4 rein wiederholbar; E6 `INSERT OR IGNORE`; E9 prüft `erase_chronicle_link` vor dem Anlegen.

**Manifest vor Löschen:** Einträge werden in derselben Transaktion wie das Löschen eines Chunks festgeschrieben. Die Menge bleibt nach Gate stabil, weil Schreibzugriffe gesperrt sind (Z1).

### 12.5 `erase(Tenant)`

Wie 12.4 mit Zielen `tenants`. Unterschiede:

- E2b: alle Index-Strukturen des Mandanten, alle `DK_s` und `TK_t` nullen.
- E3/E4: `destroy` für `TK_t` und `CK_t`. Ist `TK_t` vernichtet, ist kein DEK des Mandanten mehr berechenbar.
- E6: zusätzlich Kaskade `destroy(DK_s)` für jeden noch aktiven Betroffenen, Fortschritt je Betroffenem durabel (CP-T1).
- E9: Chronik-Commitment mit `scope = Global` (Mandant bleibt unbekannt).

Commitments im Chronik-Eintrag tragen keine `tenant_id`. Eine Zuordnung zum Mandanten ist nur über `erase_chronicle_link` und `erase_log` möglich, also nur mit Zugriff auf Domäne B.

### 12.6 Manifest und Commitment-Wurzeln

```
manifest_root  = SHA-256("ctx/v2/manifest" ‖ 0x00 ‖ concat over rows sorted by (collection_id, doc_id, generation):
                         doc_id ‖ generation (4, BE) ‖ envelope_sha256)
erase_content  = SHA-256("ctx/v2/erase-commit" ‖ 0x00 ‖ commitment ‖ manifest_root ‖ receipt_sha256)
commitment     = HMAC-SHA256(KC_e, "ctx/v2/commit" ‖ 0x00 ‖ tenant_id ‖ subject_ref)   [Subject]
               = HMAC-SHA256(KC_e, "ctx/v2/commit" ‖ 0x00 ‖ tenant_id ‖ 16×0x00)       [Tenant]
```

`receipt_sha256` = SHA-256 von `DestroyReceipt.raw`. Bei Tenant-Löschung ist das Commitment gegen die Nullmenge gebildet, damit es nicht zwischen Mandant und Betroffenen unterscheidbar ist (Entscheidung O18).

### 12.7 Öffnen und Wiederaufnahme (SPEC 9.6, erweitert)

| Schritt | Aktion |
|---|---|
| O1 | Dateisperre auf `LOCK`; Plattformprobe (Dateisystemtyp, Verhaltensprobe) |
| O2 | SQLite öffnen, Pragmas, `application_id`, `user_version`, `quick_check` |
| O3 | `erase_log`-Kette prüfen; **Chronik-Kette prüfen (I-24)**; Zustände mit Log abgleichen (fortgeschrittener gewinnt) |
| O4 | Zeilen laden; RAM-Gate-Flags für Ziele mit `state ≠ Active` setzen |
| O5 | Custodian-Abgleich (`get_many`): `Gone` → Ziel auf `Gated` mit `RECONCILED`; Transportfehler → Store öffnet nicht |
| O6 | Löschregister abgleichen (falls konfiguriert) |
| O7 | Offene Vorgänge fortsetzen (Worker): Gated → E3; Destroyed → E6; E9 für Completed ohne Link |
| O8 | Indexaufbau: alle Dokumente aktiver Ziele. AEAD-Fehler → Quarantäne, `Degraded`; bei `strict_open` → Abbruch |
| O9 | Betriebsbereit |

Ein Absturz in O1 bis O9 ändert nichts an der Korrektheit des nächsten Öffnens.

### 12.8 Worker (SPEC 10.4)

Ein Thread je Instanz. Führt E3 bis E9 und Kaskaden aus. Hält zwischen Schritten keine Sperre über Custodian-Aufruf oder `fsync`. Backoff 1 s bis 15 min mit Jitter. `erase_sla` ist die Frist bis `Completed` ab Gate (nicht Latenz bis Gated). Überschreitung → `pending=ErasePastSla`, Beobachter, Wiederholung läuft weiter. Keine stille Aufgabe.

### 12.9 Absturzpunkte (SPEC 9.8, erweitert)

Tabelle 9.8 der Quelle unverändert für R1–R3, P1–P4, E1–E12, T1, O1–O4. Neu:

| Punkt | Lage | Erwartung |
|---|---|---|
| CP-E1R | nach Register-Eintrag, vor Gate-Commit | Register kennt Ziel, DB nicht gesperrt → O6 sperrt (RECONCILED, Grund Register) |
| CP-E13 | in der Chronik-Transaktion, vor COMMIT | Ziel `Completed`, Commitment fehlt → O7 holt E9 nach |
| CP-C4 | Chronik: Zeile vor COMMIT | kein Eintrag; Wiederholung ergibt neue `seq` |
| CP-C5 | Chronik: nach Checkpoint-Zeile | Checkpoint in derselben Transaktion; kein Halbzustand |

### 12.10 Was `erase` nicht tut

Keine Löschung von: Backups des Custodians (R2) oder des Datenverzeichnisses (R14), bereits ausgeliefertem Klartext (R6), Daten bei externen Verarbeitern (R5), Swap, Dumps, Hibernation (R1). Keine Prüfung von Aufbewahrungspflichten (R17). **Keine Löschung in der Chronik:** Die Chronik enthält nach Definition keine Personendaten; Commitments werden nach `commitment_linkability` anonym (R19).

---

## 13. Nebenläufigkeit

Synchron, keine Async-Laufzeit (Nicht-Ziel). `Acta: Send + Sync`. Ein Worker-Thread.

| Name | Typ | Schützt |
|---|---|---|
| `W` | `Mutex<()>` | jede Mutation: `create_*`, `register_subject`, `put`, `update`, `rectify`, `chronicle_append`, `tombstone`, Zustandsübergänge, Purge-Chunk |
| `S` | `RwLock<State>` | RAM-Index, Schlüsselcache, Gate-Tabelle |
| `C` | `Mutex<Connection>` | die eine SQLite-Verbindung |
| `Q` | `Mutex<VecDeque<Job>>` + `Condvar` | Worker-Warteschlange |

**Regeln** (je ein Test, T-CONC-4):

1. Ordnung: `W` vor `S`, `W` vor `C`.
2. `S`, `C`, `Q` sind Blattsperren; nie zwei gleichzeitig.
3. Kein Aufruf nach außen (Observer, Clock, Custodian, Registry) unter gehaltener Sperre. Ausnahme: kurzer Custodian-Aufruf in `register_subject`/`create_tenant` unter `W` (zeitbegrenzt, K-8).
4. Commits halten `W` und `C` bis nach `fsync` (D1).
5. Vergiftete Sperre ist fatal: Handle `Poisoned`, nie `into_inner`.

`TargetGate` enthält `AtomicU8` mit Zustandscode. Einträge werden nie auf kleineren Wert gesetzt (I-10).

Gegenüber Contextra entfällt das Sperrschema aus `contextra-mvcc` (SequenceLog, SSI, TxBuffer-Shards) vollständig (D-03).

---

## 14. Öffentliche Schnittstelle

Typen und Funktionen nach SPEC 11 mit folgenden Änderungen:

### 14.1 Neue Typen

```rust
pub struct ChronicleInput<'a> {
    pub kind: ChronicleKind, pub scope: Scope, pub matter: &'a str,
    pub valid: ValidInterval, pub text: Option<&'a str>, pub meta: &'a [(&'a str, MetaValue<'a>)],
    pub supersedes: Option<EntryId>, pub class: ClassAssertion,
}
pub enum ClassAssertion { NonPersonal }                       // einziger Wert; Pflichtfeld, kein Default
pub struct ValidInterval { pub from_day: i32, pub to_day: Option<i32> }   // None = offen
pub enum Scope { Global, Tenant(TenantRef) }
pub enum AsOf { At { valid_at: i32, known_at_ms: u64 }, Seq { valid_at: i32, known_seq: u64 } }
pub enum ChronicleOutcome<T> { Found(T), NotFound, Erased, HorizonExceeded }
pub struct EntryId([u8; 16]);
```

### 14.2 Funktionen (Ergänzungen)

| Funktion | Wirkung | Kapitel |
|---|---|---|
| `Acta::chronicle_append(ChronicleInput) -> Result<EntryId, Error>` | Eintrag mit Hashkette | 10.3 |
| `Acta::chronicle_get(EntryId) -> Result<ChronicleOutcome<ChronicleEntry>, Error>` | Eintrag, entschlüsselt | 10.4 |
| `Acta::chronicle_resolve(matter, AsOf) -> Result<ChronicleOutcome<ChronicleEntry>, Error>` | Gültigkeit nach 9.1 | 9.1 |
| `Acta::chronicle_history(matter, valid_at) -> Result<Vec<ChronicleEntry>, Error>` | Korrekturkette in `seq`-Reihenfolge | 9.1 |
| `Tenant::get_at(DocId, AsOf) -> Result<ChronicleOutcome<Doc>, Error>` | Vault-Version nach 9.4 | 9.4 |
| `Collection::search_at(SearchQuery, AsOf) -> Result<Vec<Hit>, Error>` | Suche gegen Version zum `known_at`; Ergebnis darf kleiner sein | 11.7, 9.4 |

Alle bisherigen Funktionen aus SPEC 11.2 bleiben. `Collection::get` ist die Kurzform von `get_at(now)`.

### 14.3 Nicht vorhanden in v1

`delete`, `restrict`, `lift_restriction`, `change_passphrase`, `report` (als Löschbericht), `acknowledge_ledger_reset`, Backup-Funktionen, `Chronicle::takedown` (1.1, O16), `erase(Collection)` (1.1, O15), `erase(Expired)` (1.1, O19). Keine Stubs (Z6).

### 14.4 Verhalten bei Wiederholung

Wie SPEC 11.3. Ergänzung: `chronicle_append` mit gleichem `supersedes` und gleichem Inhalt ist **nicht** idempotent (jeder Aufruf erzeugt einen Eintrag). Idempotenz ist Aufrufersache über eigene Schlüssel im Payload (Entscheidung O20).

---

## 15. Konfiguration, Fehler, Beobachtung

### 15.1 Konfiguration

`Config::new(data_dir, custodian)`. Kein `Default` ohne Custodian. `validate()` liefert benannte `Config`-Fehler.

| Parameter | Standard | Bereich |
|---|---|---|
| `data_dir` | — | existierend, beschreibbar, Modus 0700 |
| `custodian` | — | `Arc<dyn Custodian>` |
| `registry` | keines | `Option<Arc<dyn ErasureRegistry>>` |
| `registry_policy` | `Required` | `Required`, `BestEffort` |
| `observer` | keiner | `Option<Arc<dyn Observer>>` |
| `clock` | Systemuhr | `Arc<dyn Clock>` |
| `rng` | Systemzufall | `Arc<dyn Rng>` (nur `cfg(test)` oder Testkit-Feature, sonst gesperrt) |
| `open_mode` | `ReadWrite` | `ReadWrite`, `ReadOnlyForensic` |
| `strict_open` | `true` | bool (O6) |
| `erase_return` | `Destroyed` | `Destroyed`, `Gated` |
| `erase_return_timeout` | 60 s | 1 s … 10 min |
| `erase_sla` | 24 h | 1 min … 30 d |
| `purge_chunk_docs` | 1000 | 1 … 10 000 |
| `clock_skew_tolerance` | 5 min | 1 s … 1 h |
| `history_horizon_days` | 30 | 0 … 365 (**neu**; 0 = keine Historie) |
| `chronicle_checkpoint_every` | 1024 | 64 … 65 536 (**neu**) |
| `commitment_epoch_days` | 90 | 7 … 365 (**neu**, O21) |
| `commitment_linkability_days` | 365 | 30 … 3650 (**neu**) |
| `max_doc_bytes` | 4 MiB | 1 MiB … 64 MiB |
| `max_text_bytes` | 1 MiB | 1 KiB … 4 MiB |
| `max_index_bytes` | min(4 GiB, ½ Speicher) | 64 MiB … Speicher |
| `rrf_depth` | 200 | `k` … 1000 |

**Konsistenzregeln:** `erase_return_timeout ≤ erase_sla`; `max_text_bytes ≤ max_doc_bytes`; `local_custodian_dir` und Register außerhalb `data_dir`; `commitment_linkability_days ≥ 2 · commitment_epoch_days`; `history_horizon_days` für `Special` ignoriert (immer 0). Verletzung → `Config`-Fehler.

**T-CONFIG-1:** Jeder Standard mit jedem zulässigen Grenzwert verträglich.

### 15.2 Fehler

```rust
pub enum Error {
    InvalidInput(InvalidInput), Limit(LimitKind), Config(ConfigError),
    Erased, UnknownTenant, UnknownSubject, Exists, NotFound,
    KeyGone, CustodianUnavailable, Custodian(CustodianKind),
    Rng, Crypto, Clock,
    Corrupt(CorruptKind),        // Schema, Db, LogChain, ChronicleChain, Auth, UnknownAlg, RowMismatch, Format
    Io(IoKind),                  // Poisoned, Other
    Locked, Unsupported, Registry(RegistryKind), NotImplemented, Closed,
    ChronicleClass,              // ClassAssertion fehlt oder Verstoß gegen I-19
}
```

`Display` und `Debug` ohne Schlüssel, Klartext, `subject_ref`, `tenant_id`, Pfad (I-16). `#[non_exhaustive]` ist für `Error` gesetzt, für `Corrupt` und `Limit` nicht (exhaustive Matches für Auditoren).

### 15.3 Beobachter

Ereignisse wie SPEC 12.3, ergänzt um `ChronicleCommitted { seq }`, `ChronicleCheckpoint { seq }`, `ChronicleChainBroken` (vor Start). Enthalten nur Zähler, `erase_id`, Zustandsnamen, Sequenzen (I-16).

### 15.4 `health()`

Wie SPEC 12.4. Felder aus gemessenem Zustand. T-HEALTH-1 prüft jedes Feld.

---

## 16. Betrieb

### 16.1 Verzeichnislayout

```
<data_dir>/LOCK           Sperrdatei (leer)
<data_dir>/ctx.sqlite3    Datenbank
<data_dir>/ctx.sqlite3-wal, -shm
```

Keine weiteren Dateien. Insbesondere keine Schlüssel, keine Exporte, keine Berichte (T-ARCH-8).

### 16.2 Sicherungen (Betreiberpflicht, D7)

1. Sicherung nur im geschlossenen Zustand oder per konsistentem Dateisystem-Snapshot.
2. Aufbewahrung der Sicherungen ≤ Löschfrist; Sicherungen des Custodians gleich lang oder kürzer (R2).
3. Nach Restore: Öffnen führt O3, O5, O6 aus. Offene Löschaufträge werden **erneut eingereicht** (idempotent über `erase_id`, R11).

### 16.3 Löschregister

Wie SPEC 13.4. `ErasureRegistry` ist außerhalb B. `registry_policy = Required` ist Standard. Referenzimplementierung `acta-registry-file` (E3) mit `fsync` je Eintrag.

### 16.4 Verlust von Schlüsseln

Verlust von `MK` oder des Custodians macht alle Daten unlesbar (R9). Beabsichtigt, muss in jeder Betreiberdokumentation stehen.

### 16.5 Formatversionen

`application_id` und `user_version = 2` kennzeichnen die Datei. Eine Datei mit anderer Version öffnet nicht. Acta v1 ist das erste Release; daher keine Migration aus Contextra oder Altrepo (V-10). Eine Formatänderung erhöht `version` (Envelope) oder `user_version` und die Spezifikationsversion.

### 16.6 Plattform und Toolchain

Linux (ext4, XFS), macOS (APFS). Rust `1.89.0` in `rust-toolchain.toml` gepinnt (Contextra INV-9 übernommen), Edition 2021, `--locked` in allen Aufrufen.

---

## 17. Nachweis und Prüfung

### 17.1 Materialsammlung (`evidence`)

Acta stellt **keinen Bericht** aus (1.2). `evidence(erase_id)` liefert eine Materialsammlung in UTF-8-JSON, Schlüssel alphabetisch sortiert, Binärwerte als Kleinbuchstaben-Hex, keine Gleitkommazahlen.

| Feld | Inhalt |
|---|---|
| `spec_version` | Fassung dieser Spezifikation |
| `store_id` | 16 Byte hex |
| `erase_id`, `target_kind`, `tenant_id`, `subject_ref` | Pseudonyme (R4); Auditor leitet `KeyId` ab |
| `state`, `pending` | gemessener Zustand |
| `attestation_class` | `A0`, `A1`, `A2` unverändert vom Adapter |
| `log` | `erase_log`-Einträge der `erase_id`, `genesis`, `entry_hash` des letzten |
| `custodian` | `custodian_ref`, `custodian_time_ms`, `scheduled_final_ms`, `karenz`, `receipt_raw` (hex), `receipt_sha256` |
| `manifest` | Zeilen `(collection_id, doc_id, generation, envelope_sha256)` und `manifest_root` (ab Completed) |
| `chronicle` | `seq` und `entry_hash` des Commitments (ab Completed), `commitment` (hex) |
| `clock_skew` | bool |
| `limits_text` | feste Aussage (A.6) |

Das Paket enthält **keine** Felder `verified`, `compliant`, `deleted`, `erased` o. Ä. (A-4). Keine Schlüssel, keine Klartexte.

### 17.2 Prüfverfahren für den Auditor (Domäne D)

Der Auditor braucht: Kopie von B (idealerweise vor und nach Löschung), eigenen Custodian-Zugang, das Paket, eigenen Code (Referenz Anhang B.4, `audit_open.py`, und `audit_chronicle.py` für Kette und Commitment, Anhang B.8).

| Schritt | Prüfung | Belegt |
|---|---|---|
| V1 | `erase_log` und Chronik-Kette aus B neu berechnen | keine Teilmanipulation; Grenze W5 |
| V2 | `KeyId` ableiten; **beim Custodian direkt** `status` und `get` abfragen | Z2 |
| V3 | `receipt_sha256` = SHA-256(`receipt_raw`); `aux` des DESTROYED-Eintrags gleich; `custodian_ref` im Audit-Log des Custodians | Beleg stammt vom Custodian |
| V4 | `manifest_root` neu berechnen; Vergleich mit `aux` des COMPLETED-Eintrags | Manifest vollständig |
| V5 | Vor Purge: SHA-256 jedes Envelopes gegen `envelope_sha256` | Manifest beschreibt Objekte |
| V6 | **Unlesbarkeit:** alle Envelopes der Vorher-Kopie mit allen vorhandenen Schlüsseln öffnen; jeder Versuch MUSS scheitern. Positivkontrolle: derselbe Code öffnet einen Envelope eines nicht gelöschten Betroffenen | Z2, W2 |
| V7 | Zeitabgleich `ts_ms` gegen `custodian_time_ms` | Plausibilität |
| V8 | **Neu.** Commitment: Auditor berechnet `HMAC(KC_e, …)` mit `KC_e` aus Custodian (solange vorhanden) und vergleicht mit Chronik-Eintrag | Z10 |
| V9 | **Neu.** Chronik-Eintrag enthält keine Betroffenen- oder Mandanten-ID (Scan aller Felder nach bekannten Werten) | Z9 |
| V10 | Klasse und `limits_text` gelesen; Außenaussage nicht stärker (17.4) | Z5 |

Positivkontrolle für V6 ist Pflicht (ohne sie beweist Misserfolg nichts).

### 17.3 Kanarien-Test (Auditor, echte Konfiguration)

| Schritt | Aktion |
|---|---|
| K1 | Zufallswerte `marker` (32 Byte) und `subject_ref` |
| K2 | `register_subject(canary)`, `put` eines Dokuments mit `marker` |
| K3 | Kopie von B einschließlich WAL (`kopie_vor`) |
| K4 | Positivkontrolle: mit eigenem Code und Schlüsseln aus C den Envelope öffnen; `marker` erscheint |
| K5 | `erase(canary)` bis mindestens `Destroyed` |
| K6 | Custodian direkt: `get` = `NotFound`/`Disabled`; `status` ∈ {Destroyed, Disabled} |
| K7 | Negativkontrolle: K4 auf `kopie_vor` mit allen vorhandenen Schlüsseln; MUSS scheitern |
| K8 | `marker` (hex und Rohbytes) in B suchen (Dateien, WAL, Freiseiten); kein Treffer |
| K9 | `custodian_ref` im Audit-Log des Custodians wiederfinden |
| K10 | **Neu.** Commitment in Chronik vorhanden und K9-Schritt-analog verifizierbar |

T-CANARY-1 automatisiert K1 bis K8 gegen `LocalCustodian` (A0). Für externe Adapter einmal vor Inbetriebnahme auf dem echten Dienst.

### 17.4 Attestierungsklassen

| Klasse | Bedeutung | Höchste zulässige Außenaussage |
|---|---|---|
| **A0** | Selbstauskunft. Lokaler Schlüsseleintrag, Funktionstest. | „Der lokale Schlüsseleintrag wurde überschrieben und entfernt (Funktionstest, Klasse A0). Das ist keine Löschbestätigung.“ |
| **A1** | Dienstbeleg mit eigener Kennung (`custodian_ref`, `raw`). Karenz ausgewiesen. | C.1 plus „Der Key-Dienst hat die Vernichtung quittiert (Klasse A1).“ |
| **A2** | A1 plus signierter Beleg, Audit-Log des Dienstes, Adapter besteht T-CUST-2 | C.1 plus „Der Beleg ist vom Dienst signiert und im Audit-Log des Dienstes nachvollziehbar (Klasse A2).“ |

**Regeln:** A-1 Klasse aus Adapter unverändert. A-2 `raw` unverändert gespeichert; bei A1/A2 leer → `CustodianInconsistent`. A-3 Karenz ausgewiesen. A-4 nur Tatsachen. A-5 Zeitabgleich mit `clock_skew`-Hinweis, kein Fehler. A-6 `limits_text` nicht abschaltbar.

Der Commitment-Teil der Chronik gilt **nicht** als eigene Klasse. Er ist ein Hinweis auf Verankerung und stärkt die Aussage nicht über die Klasse des Custodian-Belegs hinaus (Entscheidung O22).

### 17.5 Grenzen (A.6 `LIMITS_TEXT`)

Nicht erfasst: Arbeitsspeicher, Auslagerung, Speicherabzüge (R1); Sicherungen von Custodian und Datenverzeichnis (R2, R14); externe Verarbeiter (R5); ausgelieferter Klartext (R6); Schlüsselkopien vor Vernichtung (R7); Karenzfrist (R8); rechtliche Einordnung (R17); Chronik-Fehlklassifikation (R18). Keine Rechtmäßigkeitsaussage.

---

## 18. Architektur, Crates, Schichtung

### 18.1 Crate-Karte

| Crate | Ebene | Zweck | Abhängig von (Kern) | LOC-Budget [Ziel] |
|---|---|---|---|---|
| `acta-ids` | E0 | Typisierte 16-Byte-Identifikatoren, keine Verlustumwandlung | — | 300 |
| `acta-env` | E0 | `Clock`, `Rng` (fallibel), `IdGen` | — | 400 |
| `acta-model` | E1a | Reine Domäne: Zustände, `next()`, Horizont, Klassen, AsOf-Typen | ids | 700 |
| `acta-seal` | E1a | Secret, Envelope E1, P1, DEK/CDEK, AEAD | ids, env | 1500 |
| `acta-store` | E1b | SQLite-Speicher, Schema, Pragmas, `erase_log`-Kette | ids, model | 2000 |
| `acta-index` | E1b | RAM: Analyzer, BM25, exakte Vektorsuche, RRF, Filter | ids, env, model | 2500 |
| `acta-custodian` | E1b | Trait `Custodian`, `KeyId`, Fehler, `DestroyReceipt`, `AttestationClass` | ids, seal | 600 |
| `acta-chronicle` | E2 | Ebene N: Einträge, Hashkette, Checkpoints, Commitments | ids, model, seal, store | 1200 |
| `acta` | E2 | Facade: Öffnen, Tresor, Gate, Löschsaga, Worker, Chronik-API, Evidence | ids, env, model, seal, custodian, store, index, chronicle | 4000 |
| `acta-forensic` | E2 | `RawScanner`, Positivkontrolle, Kanarien-Helfer (auch Auditor-Werkzeug) | ids | 1000 |
| `acta-custodian-local` | E3 | `LocalCustodian` (A0) | custodian, seal | 800 |
| `acta-registry-file` | E3 | Referenz-Löschregister | acta | 500 |
| `acta-conformance` | E3 | Suiten T-CUST-1/2, T-STORE-1 | custodian, store | 800 |
| `acta-testkit` | E3 (nur Tests) | `FaultIo`, `ManualClock`, `ReferenceModel`, Mutanten-Custodians, Integrationstests | alle | 1500 |

Produktionscode gesamt [Ziel]: ≈ 16.000 Zeilen. **Löschkern** = `acta-model`, `acta-seal`, `acta-custodian`, `acta-store`, Modul `acta::erase` und `acta::gate` in `acta`, `acta-forensic`. Budget ≤ 7.000 Zeilen [Ziel] (ACTA_SPEC nannte 6.300, Abweichung wegen SQLite-Schema und Commit-Logik im Store, Entscheidung O23).

**Gegenüber ACTA_SPEC entfallen** v1: `acta-mvcc`, `acta-derive`, `acta-vault` (als eigenes Crate, Inhalt in `acta`), `acta-proof` (Inhalt in `acta::evidence`), `acta-backup` (1.1), `acta-query` (Phase 2), `acta-adapter-*` (nicht nötig, D-14), `acta-node` (D-14), `acta-sys`, `acta-codec` (Inhalt in `acta-seal`).

**Gegenüber Contextra entfallen**: alle 39 Crates als Ganzes; Übernahmen in `ACTA_DATEIZUORDNUNG.md`.

### 18.2 Abhängigkeitsrichtung (Gate G-L1)

```
  E3   custodian-local   registry-file   conformance   testkit
           │                  │               │            │
  E2       │   chronicle   acta   forensic    │            │
           │       │        │                 │            │
  E1b      │   store  index  custodian  ◄─────┘            │
           │     │       │      │                          │
  E1a      │   model   seal ◄────┘                          │
           │     │       │                                  │
  E0       └─── ids ◄── env ◄────────────────────────────────┘
```

Regeln wie Contextra R1 bis R14, übernommen und ergänzt:

| Regel | Inhalt |
|---|---|
| **R1** | Abhängigkeiten nur nach unten. Gleiche Ebene: keine Kante, außer benanntem Feature (R13). |
| **R2** | Port-Traits gehören dem konsumierenden Crate oder dem Vertragscrate (`custodian`). |
| **R3** | Ein Besitzer je Konzept. `Clock`, `Rng` in `env`; Zustände in `model`; Schlüssel-IDs in `custodian`. |
| **R4** | Nur Adapter (E3) nennen konkrete Schlüsseldienste. |
| **R5** | Öffentliche APIs re-exportieren keine Typen Dritter. |
| **R6** | Features rein additiv, keine Verhaltensänderung des Löschpfads (V-8). Default ohne Features (V-12). |
| **R7** | Keine `compat`-Module, keine Altschnittstellen (V-10). |
| **R8** | Kern (E0–E2) enthält kein `tokio`, kein `std::fs` außerhalb `acta-store`, kein `SystemTime`, kein `thread_rng`. |
| **R9** | `#![forbid(unsafe_code)]` in allen Crates (D-12). |
| **R10** | `unwrap`, `expect`, `panic!`, Indexzugriff in Produktionscode `deny` (V-1). |
| **R11** | Dev-Kanten zeigen nur nach unten (INV-8 aus Contextra). Integrationstests der Gates G1 und G2 liegen in `acta-testkit/tests/`. |
| **R12** | `acta-testkit` ist kein Dependency-Ziel eines Produktionscrates. |
| **R13** | Kanten innerhalb einer Ebene nur als Feature, Kantengraph azyklisch. |
| **R14** | Jedes Crate ist Workspace-Mitglied. Ausschluss nur mit ADR. |
| **R15** | **Neu.** Adapter (E3) implementieren Ports; Kern kennt nur Traits. |

Prüfung: `tools/gate_layering.py` aus Contextra, **ohne** `|| true` (V-14).

### 18.3 Ports und Besitz

| Port | Besitzer | Implementierung v1 | Austausch |
|---|---|---|---|
| `Custodian` | `acta-custodian` | `LocalCustodian` (A0) | ja (Adapter, Konformität T-CUST-1) |
| `ErasureRegistry` | `acta` | `FileRegistry` (`acta-registry-file`) | ja |
| `Observer` | `acta` | keiner oder Aufrufer | ja |
| `Clock` | `acta-env` | Systemuhr, `ManualClock` (Testkit) | ja |
| `Rng` | `acta-env` | `getrandom`, Test-RNG (Testkit) | ja (nur Test) |
| `Notary` | `acta-chronicle` | nicht in v1 (1.1) | — |
| `BlobStore` | **kein Port** (D-14) | `acta-store` (SQLite) | nein |

### 18.4 Typestate und Beweistypen

- `GatedTarget`, `DestroyedTarget`, `CompletedTarget` als Typen mit privatem Konstruktor, erzeugt nur von den Funktionen E1, E5, E8.
- `DestroyProof { receipt, status, probe }` nur aus E3/E4 (I-7).
- `PurgeProof { manifest_root }` nur nach vollständigem E6 (I-8).
- `Completed` verlangt `&DestroyProof` und `&PurgeProof`.
- T-TYPE-1 (`trybuild`): diese Typen sind nicht konstruierbar.

Das ist die Umsetzung von V-4 und ACTA 15.6. Sie ersetzt die Contextra-Praxis, Zeugen über öffentliche `const fn` zu erzeugen (Befund B-18).

### 18.5 Repository-Layout

```
acta/
  Cargo.toml  Cargo.lock  rust-toolchain.toml  deny.toml  clippy.toml  justfile
  docs/SPEC.md                    diese Datei (nicht löschen, T-DOC-1)
  docs/decisions/ADR-*.md         Entscheidungen, Format wie Contextra ADR
  crates/<crate>/                 nach 18.1
  crates/acta-testkit/tests/      Integrations- und Absturztests
  fuzz/                           Fuzz-Ziele (Envelope, P1, Chronik, Analyzer)
  audit/                          Referenzprüfer (Python), Vektoren, Anleitung
  tools/                          Gate-Skripte (übernommen aus Contextra, Schritt AP0)
  governance/                     gates.toml, ratchet.toml, protected-paths.toml
  .github/workflows/              CI (ohne `|| true`)
```

---

## 19. Code-Standards

### 19.1 Modularität

- `lib.rs` nur Modul-Doku und Re-Exports. Keine Logik.
- Ein Konzept pro Datei. Datei ≤ 500 Zeilen Produktionscode (Ratchet), Funktion ≤ 60 Zeilen, kognitive Komplexität ≤ 15 (`clippy.toml`).
- Module nach Verantwortung, nicht nach Technik (`gate/read.rs`, nicht `util/`).
- `pub(crate)` ist Standard. `pub` nur für den Vertrag.
- Jede `pub`-Item mit Rustdoc (`#![deny(missing_docs)]` in Vertragscrates `acta`, `acta-custodian`, `acta-model`).

### 19.2 Typen

- Newtypes für jede ID. Keine `String`-Schlüssel außer `KeyId` (validiert).
- Keine booleschen Parameter. Enums mit benannten Varianten.
- Interne Enums exhaustiv (`ErasureState`, `KeyKind`): neue Variante bricht den Build. Öffentliche API-Enums `#[non_exhaustive]`, außer Auditor-relevanten (`Corrupt`).
- Zeit als `u64` Millisekunden oder `i32` Tage, nie `f64`.

### 19.3 Fehler

- `thiserror` pro Crate, Fehlertyp je Schicht, Umwandlung nur an der Grenze (`From` nur in der Facade).
- Keine Fehler als Werte. Jede fehlbare Funktion liefert `Result`.

### 19.4 Determinismus

- `BTreeMap` in Ausgabe- und Hashpfaden. `HashMap` nur für interne Lookups ohne Reihenfolge-Abhängigkeit.
- Sortierung mit Tie-Break nach ID (INV-5).

### 19.5 Speicher und Nebenläufigkeit

- Keine Sperre über `.await` (keine Async im Kern). Keine Sperre über Aufruf nach außen (13).
- `Arc` nur an Port-Grenzen. Innere Strukturen besitzen Daten.
- Geheimnisse: nur `Secret<N>`.

### 19.6 Kommentare und Dokumentation

- Kommentare erklären das Warum (Invariante, Befund, Entscheidung) und verweisen auf ID (I-x, D-x, R-x).
- Keine Compliance-Aussagen in Code oder Doku (INV-7).
- `SAFETY:`-Kommentare entfallen mangels `unsafe`.

### 19.7 Commits

Conventional Commits. Ein Schritt, ein Commit-Paar: Commit A verschiebt, Commit B baut um (INV-3 Contextra). `commit-msg`-Hook fehlt bei Contextra (`.githooks/` enthält nur `pre-commit`, `pre-push`) und wird in Acta angelegt.

---

## 20. Tests und Gates

### 20.1 Testregeln

| Nr. | Regel |
|---|---|
| **TR-1** | Keine Attrappe im Datenpfad der Gates G1 und G2: echte SQLite, echter `LocalCustodian`, echtes Dateisystem. |
| **TR-2** | Mutationsregel: Für jeden Schutzmechanismus ein Mutant, der mindestens einen Test rot macht. |
| **TR-3** | Neustartregel: Löschtests enden mit Neustart in neuem Prozess und Schlüsselprobe. |
| **TR-4** | Jeder Test hat eine Assertion. `#[ignore]` verboten. |
| **TR-5** | Determinismus: feste Seeds, feste Uhr, keine echten Wartezeiten außer in Zeitlimit-Tests. |
| **TR-6** | Rot zuerst: T-WALK-1 wird rot eingecheckt, bevor Implementierung entsteht. |
| **TR-7** | Rückführbarkeit: jeder Test trägt die IDs der Zusagen, Invarianten, Befunde. T-DOC-3 prüft Vollständigkeit. |

### 20.2 Katalog (Auszug, vollständig in der Quelle SPEC 16.2)

| ID | Prüft | Gate |
|---|---|---|
| T-WALK-1 | Walking Skeleton (20.3) | G1 |
| T-SEAL-1 | Known-Answer, Anhang B byteweise | G1 |
| T-SEAL-2 … 4 | Parser, Bindung, Manipulation | G1 |
| T-NOFALLBACK-1 | jeder Custodian-Fehler → keine Ersatzwerte | G1 |
| T-RNG-1 | `Rng`-Fehler → keine Zeile, kein Schlüssel | G1 |
| T-DEL-1 | `erase(Subject)`: keine Klartexte nach Gate, Probe, Neustart | G1 |
| T-DEL-2 | `erase(Tenant)` mit Kaskade | G2 |
| T-MULTI-1 | Mehrpersonen-Dokument | G1 |
| T-ISO-1 | Isolation | G2 |
| T-GATE-1 | Suchpfad und Gate | G2 |
| T-CONC-1 … 5 | Nebenläufigkeit (`loom`) | G2 |
| T-CRASH-`<CP>` | je Absturzpunkt aus 12.9 | G2 |
| T-OPEN-1 | O1 bis O9 | G2 |
| T-RESTORE-1 | Restore mit Custodian-Gone | G2 |
| T-REG-1 | Löschregister | G2 |
| T-LOG-1 | `erase_log`-Kette, Trigger | G2 |
| T-STORE-1 | Speicher-Konformität (Commit überlebt `kill -9`) | G2 |
| T-STORE-2 | Schema, Pragmas, `application_id`, `user_version` | G1 |
| T-POISON-1 | `fsync`-Fehler → Poisoned | G2 |
| T-CUST-1 | Konformitätssuite 1 | G1 |
| T-CUST-2 | A2-Suite | je Adapter |
| T-CANARY-1 | K1 bis K8 automatisiert | G2 |
| T-EVID-1 | Paket, Klasse, `limits_text` | G2 |
| T-STUB-1 | Keine stillen `Ok` | G1 |
| T-HEALTH-1 | `health()` folgt dem Zustand | G2 |
| T-CONFIG-1 | Grenzwertkombinationen | G1 |
| T-LIMIT-1 | Jedes Limit beidseitig | G1 |
| T-OBS-1 | Keine Personendaten in Fehlern, Logs, Ereignissen | G2 |
| T-LEAK-1 | Bytesuche im Datenverzeichnis nach Schlüsseln und Marker | G1 |
| T-TYPE-1 … 3 | Typen, Secret, Bezeichner | G1 |
| T-IDX-1 … 6 | BM25, exakte Vektor, RRF, Filter, Entfernen, Limits | G2 |
| T-WORKER-1 | Backoff, `ErasePastSla` | G2 |
| T-REPLACE-1 | `replace_and_erase` | G2 |
| T-IDEM-1 | Wiederholung von `erase` | G2 |
| T-LOCK-1 | Zweiter Prozess → `Locked` | G2 |
| T-FUZZ-1 … 4 | Envelope, P1, Chronik-Decoder, Analyzer | G2 |
| T-ARCH-1 … 12 | Layering, deny, Typnamen, static, Features, forbid(unsafe), Ignore, Legacy, Doc-Vollständigkeit, Eigenbeleg (T-ARCH-9), Nonce (T-ARCH-10), Konstanten-Ersatz (T-ARCH-11), CI-`|| true` (T-ARCH-12) | G1 |
| T-DOC-1 … 3 | Spezifikation vorhanden, Konstanten, Abdeckung | G1 |
| T-POWER-1 | Nachtlauf, Gerätefehler-Simulation | G3 |
| **T-CHR-1** | Chronik enthält keine `subject_ref`; Eintrag mit Betroffenenbezug abgelehnt (Z9) | G1 |
| **T-CHR-2** | AsOf-Gleichheit (I-20, Z8) | G2 |
| **T-CHR-3** | Commitment nach Completed, idempotent, Verknüpfung nur über `erase_chronicle_link` (Z10) | G2 |
| **T-CHR-4** | Hashkette: Bit-, Lösch-, Umordnungs- und Einfügemanipulation erkannt; `UPDATE`/`DELETE` blockiert (I-24) | G1 |
| **T-HOR-1** | Horizont: Version außerhalb → `HorizonExceeded`, nach Hygiene nicht mehr vorhanden (I-21) | G2 |
| **T-CRASH-CP-E13** | Absturz zwischen Completed und Commitment | G2 |
| **T-ARCH-9** | Kein Acta-Signierschlüssel im Kern (I-22) | G1 |

### 20.3 T-WALK-1 (Walking Skeleton, verbindlich)

1. Datenverzeichnis `D`, Custodian-Verzeichnis `K` außerhalb `D`, echter `LocalCustodian` mit zufälligem `MK`, echte SQLite-Datei.
2. `create_tenant`, `create_collection`, `register_subject(S)`, `put` mit `marker`, `chronicle_append(kind 1)` mit Norm.
3. `get` und `search` finden das Dokument; `chronicle_resolve` liefert die Norm.
4. Neuer Prozess: beide Abfragen identisch (Z8).
5. `erase(Subject(T, S))`, Handle ≥ Destroyed.
6. `get` → `Erased` oder `NotFound`; `search` findet nichts; `register_subject` → `Erased`.
7. Neuer Prozess: Punkt 6 wiederholt; `health().status == Ok`.
8. Schlüsselprobe: `get(key_id(S))` = `NotFound`/`Disabled`; `.k` fehlt, `.gone` existiert.
9. Chronik-Commitment vorhanden (T-CHR-3), Norm aus Punkt 2 unverändert lesbar.
10. Auditor-Referenz `audit_open.py` öffnet den vor Löschung kopierten Envelope mit den jetzt vorhandenen Schlüsseln **nicht**.
11. T-LEAK-1 auf `D`.

### 20.4 Mutationen

Verbindlich (alle müssen sterben): M-1 bis M-25 aus SPEC 16.3, dazu:

| ID | Mutant | Rot in |
|---|---|---|
| **M-26** | Chronik-Eintrag mit `subject_ref` im Payload wird akzeptiert | T-CHR-1 |
| **M-27** | Hashkette ohne `prev_hash` | T-CHR-4 |
| **M-28** | AsOf liest `known_at` aus Wanduhr statt `commit_seq` | T-CHR-2 |
| **M-29** | Commitment mit Klartext-`subject_ref` statt HMAC | T-CHR-3 |
| **M-30** | Horizont nicht erzwungen (alte Version lesbar) | T-HOR-1 |
| **M-31** | `erase` ohne Chronik-Commitment bei Completed | T-CHR-3, T-CRASH-CP-E13 |
| **M-32** | Chronik-Schlüssel `CK_t` nicht vernichtet bei Tenant-Löschung | T-DEL-2 |

Mutationsziel `cargo-mutants` auf `acta-seal`, `acta-model`, `acta-chronicle`, `acta::erase`: nicht überlebende Mutanten ≥ 90 % [Ziel].

### 20.5 Eigenschaften und Fuzzing

- `proptest`: Round-Trip Envelope, Chronik-Kette, Zustandsautomat (`next`), Index-Entfernen gleich Neuaufbau.
- `loom`: Gate-Protokoll und Sperrordnung.
- `miri`: `acta-model`, `acta-seal`, `acta-index` (rein, ohne FFI). `acta-store` nicht (SQLite-FFI).
- Fuzzing: Envelope-Parser, P1-Decoder, Chronik-Decoder, Analyzer, Filter-Kodierung. Kurzläufe in CI, lange nachts. Korpus im Repository.

### 20.6 Gates

| Gate | Inhalt | Bedingung |
|---|---|---|
| **G1** Walking Skeleton | Kern ohne Stub | T-WALK-1 Linux und macOS; alle G1-Tests; M-1 bis M-10, M-12, M-14, M-20, M-26, M-27 tot |
| **G2** Protokoll | Löschprotokoll, Wiederaufnahme, Nebenläufigkeit, Chronik | alle G2-Tests; alle Crash-Punkte; M-1 bis M-32; Fuzz ohne Fund |
| **G3** Messung | Zahlen gemessen | Benchmarkprotokoll; jedes [Ziel] ersetzt oder gestrichen; T-POWER-1 |
| **G4** Krypto-Prüfung | Externer Prüfer für `acta-seal` und Chronik-Ableitungen | Prüfliste 6.13 beantwortet; Funde behoben; `audit`/`deny` ohne offene Advisories |
| **G5** Recht | Einordnung der Vernichtung und Chronik-Speicherung | schriftliche Einschätzung; erst dann Anhang C erweiterbar |

Engineering-Gates zusätzlich (übernommen aus Contextra `governance/gates.toml`, ohne `|| true`):

| Gate | Werkzeug | Herkunft |
|---|---|---|
| E-G1 Layering | `gate_layering.py` | Contextra |
| E-G2 Feature-Matrix | `cargo-hack --each-feature` | Contextra |
| E-G3 Unwrap | `unwrap-gate.yml` | Contextra |
| E-G4 Determinismus | `determinism_guard.py` | Contextra |
| E-G5 Dokumentwahrheit | `doc_lint.py` (Compliance-Wörter, Versionen) | Contextra |
| E-G6 Lizenz/Advisories | `cargo deny`, `cargo audit` | Contextra |
| E-G7 Fuzz | `fuzz-extended.yml` | Contextra |
| E-G8 Mutation | `cargo-mutants` (ohne `continue-on-error`) | Contextra, korrigiert |
| E-G9 Loom | `loom-gate.yml` | Contextra |
| E-G10 Format-Korpus | Golden-Dateien mit `MANIFEST.sha256` | Contextra G-F1 |

---

## 21. Roadmap

### 21.1 Pakete (ersetzt SPEC 17.2 und ACTA 20)

| Paket | Inhalt | Ende |
|---|---|---|
| **AP0** | Repository aus Contextra-Gerüst (Gates, CI, Lints, `docs/SPEC.md`, `tools/`); `acta-ids`, `acta-env`; alle Architekturtests | Architekturtests grün; **T-WALK-1 rot eingecheckt** |
| **AP1** | `acta-model` (Zustände, `next`), `acta-seal` (Secret, Envelope, P1, Ableitungen, AEAD), Anhang-B-Vektoren | T-SEAL-1 bis 4, T-TYPE-*, T-CHR-4 (Hashkette, pur), M-1, M-4 bis M-8 tot |
| **AP2** | `acta-custodian`, `acta-custodian-local`, `acta-conformance` | T-CUST-1, M-9, M-10, M-25 |
| **AP3** | `acta-store`: Schema 2, Pragmas, `erase_log`-Kette, Chronik-Tabelle | T-STORE-1, T-STORE-2, T-LOG-1 |
| **AP4** | `acta-index`: Analyzer, BM25, exakte Vektorsuche (Akkumulatorreihenfolge), RRF, Filter | T-IDX-1 bis 6 |
| **AP5** | `acta-chronicle` und Chronik-API in `acta`: Anhängen, AsOf, Commitment (E9) | T-CHR-1 bis 3, M-26 bis M-29 |
| **AP6** | Kern `acta`: Anlegen, `put`/`get`, Gate, `erase` bis Destroyed | **G1** |
| **AP7** | Worker, O1 bis O9, Purge (mit Versionen), Completed, Kaskade, Register, Evidence, Forensik, Crash-Matrix | **G2** |
| **AP8** | Messungen, Optimierung nach Messung | **G3** |
| **AP9** | Externe Prüfung (G4), Rechtsgutachten (G5), erster A1-Adapter (O3) | Release 1.0 |

### 21.2 Release 1.0

Alle G1 bis G5, Spezifikation als „Fassung 1.0“ mit Messwerten (0.2), mindestens ein produktiver Custodian-Adapter Klasse A1 oder höher mit bestandener T-CUST-1, Kanarientest auf der echten Konfiguration (17.3).

### 21.3 Version 1.1

- `Notary` (RFC 3161) für Checkpoints und Commitment-Wurzeln (D-09).
- `erase(Collection)`, `erase(Expired)`-Sweeper (O15, O19).
- `Restrict`/`Lift` (Art. 18).
- `acta-backup` (Zwei-Spuren-Sicherung aus ACTA 15, nur wenn Bedarf gemessen).
- SIMD-Kernel mit Determinismusnachweis (D-11).
- Index-Snapshots unter Epochenschlüssel (O4), nur bei gemessenem Bedarf.
- `Chronicle::takedown` mit Vier-Augen (O16).

### 21.4 Version 2

- Ingest, Klassifikation, Redaktion (ACTA 4.4, 4.5, D-17) als eigene Crates `acta-ingest`, `acta-classify`, `acta-redact`. Parser aus Contextra (`contextra-ingest`) werden hier evaluiert.
- `ContextRequest`/`ContextBundle` und `ContextReceipt` (ACTA 8.5), LLM-Anbindung über `EgressGuard` (ACTA 8.5, Z4).
- Server, MCP und CLI: `acta-server`, `acta-mcp`, `acta-cli` (ACTA 6.1), Python-Bindings (Contextra `contextra-py` als Vorlage).
- Custodian-Operation `sign` für A2-Belege (D-09).

---

## 22. Restrisiken und offene Entscheidungen

### 22.1 Restrisiken

| ID | Restrisiko | Wirkung | Gegenmaßnahme außerhalb des Kerns |
|---|---|---|---|
| R1 | Klartext im Prozessspeicher, Swap, Dumps, Hibernation | Kopien überleben | Betreiber: verschlüsselter Swap, Dump-Verbot |
| R2 | Sicherungen des Custodians | Vernichtung erreicht sie nicht | Aufbewahrung ≤ Löschfrist |
| R3 | Brechung der Primitive, Quantenrechner, langlebiger Ciphertext | Langzeitvertraulichkeit nicht garantiert | G4, Advisories |
| R4 | Metadaten im Klartext: Pseudonyme, Größen, Zeitstempel, Zugriffsmuster | Existenz und Größe erkennbar | Aufrufer: zufällige Bezeichner |
| R5 | Externe Verarbeiter | `erase` erreicht sie nicht | Aufrufer: Verträge |
| R6 | Ausgelieferter Klartext | bleibt dort | Aufrufer |
| R7 | Kompromittierter Custodian oder Schlüsselkopie vor Vernichtung | später lesbar | Zugriffskontrolle, Audit |
| R8 | Karenzfrist des Dienstes | Wiederherstellung durch Dienst möglich | Ausweisung (A-3) |
| R9 | Verlust von `MK` oder Custodian | alles unlesbar | Eigene Sicherung der Hauptschlüssel |
| R10 | Zuordnung Person → `subject_ref` | Falsche Löschung | Aufrufer (10.1) |
| R11 | Wiedereinspielen eines älteren Datenverzeichnisses im Fenster Gated | Sperre verloren | Löschregister; erneute Einreichung |
| R12 | Ältere Versionen in SQLite-Freiseiten, WAL | Ciphertext unter gleichen Schlüsseln | `replace_and_erase`, Hygiene |
| R13 | Waisenschlüssel nach Abbruch | unschädlich | Adoption (7.4) |
| R14 | Hardware, Firmware falscher `fsync`-Meldung | Reste auf Medium | Medienwahl |
| R15 | Fremdcode mit `unsafe`: SQLite-C, RustCrypto, Standardbibliothek | nicht von `forbid` erfasst | Pins, `audit`, G4 |
| R16 | Seitenkanäle: Zeit, Cache, Dateisystem | Hinweise auf Inhalt | Betreiber |
| R17 | Rechtliche Einordnung | offen | G5, Verantwortlicher |
| **R18** | **Neu.** Fehlklassifikation durch den Aufrufer (Norm mit Personenbezug in der Chronik) | Personendaten dauerhaft in der Chronik | Aufrufer; Acta erzwingt nur die Erklärung `ClassAssertion` |
| **R19** | **Neu.** Verknüpfbarkeit von Commitments bis zur Vernichtung von `KC_e` | Mit `KC_e` und einem Betroffenen-Kandidaten prüfbar | `commitment_linkability_days`; Vernichtung der Epoche |
| **R20** | **Neu.** Chronik-Metadaten im Klartext (`kind`, `matter`, Zeitstempel, `tenant_id` bei Scope Tenant) | Rückschlüsse auf Geschäftsvorgänge | Aufrufer: neutrale Matter-Namen |

### 22.2 Offene Entscheidungen

| ID | Frage | Vorschlag | Status |
|---|---|---|---|
| O1 | Speicher | SQLite | entschieden (D-02) |
| O2 | Chiffre | XChaCha20-Poly1305. Alternative AES-256-GCM-SIV (Contextra) | vorläufig, Vorbehalt G4 |
| O3 | Erster produktiver Custodian | Key-Dienst mit Karenz und signiertem Beleg | offen |
| O4 | Index für mehr als 10⁶ Chunks | Snapshots unter Epochenschlüssel, 1.1 | zurückgestellt |
| O5 | Lesezugriff bei `EXCLUSIVE` | eine Verbindung; Leser warten auf Commits | offen, G3 misst |
| O6 | `strict_open` Standard | `true` | vorläufig |
| O7 | Passphrase für `MK` | beim Aufrufer | offen |
| O8 | Mehrprozess-Replikate | Nicht-Ziel v1 | zurückgestellt |
| O9 | `foreign_keys` mit `REFERENCES` | `doc_subjects → docs` ergänzen; `erase_manifest`, `erase_log` ohne Referenz | vor G1 entscheiden |
| O10 | Zeitquelle | Wanduhr ohne Monotonie, A-5 meldet Abweichung | vorläufig |
| O11 | Quarantäne nur im RAM | ja | vorläufig |
| O12 | Versionstabelle `doc_versions` | eingeführt, Vorlage ACTA 8.2 | entschieden |
| O13 | Commitment-Schema HMAC über Epoche | so, wie 12.6 | vor G1 zu prüfen |
| O14 | Checkpoint-Signatur | ohne Signatur in v1 | entschieden (D-09) |
| O15 | `erase(Collection)` | 1.1 | zurückgestellt |
| O16 | `Chronicle::takedown` | 1.1 mit Vier-Augen | zurückgestellt |
| O17 | E4 mit Probeöffnen | nur wenn Dokumente existieren; Ziel ohne Dokumente nur `get`-Probe | vorläufig |
| O18 | Commitment bei Tenant-Löschung | gegen Nullmenge (12.6) | vor G1 zu prüfen |
| O19 | `erase(Expired)` Sweeper | 1.1 | zurückgestellt |
| O20 | Idempotenz von `chronicle_append` | Aufrufer über Payload-Schlüssel | vorläufig |
| O21 | Epochenlänge `KC_e` | 90 Tage | vorläufig |
| O22 | Rolle des Commitments für die Klasse | keine eigene Klasse, nur Verankerungshinweis | vorläufig |
| O23 | Löschkern-Budget | ≤ 7.000 Zeilen | Ratchet |

### 22.3 Befunde aus dem Contextra-Repository

Verifiziert am Klon (Methode 0.3). Einige Befunde betreffen die Quelle und dienen als Regel, nicht als Schuldzuweisung.

| ID | Befund | Stufe | Fundstelle | Acta-Regel | Test |
|---|---|---|---|---|---|
| **A-1** | `seal_record` ruft `seal_record_with_rng(...).unwrap_or_default()` auf. Bei Fehler entsteht ein **leerer Ciphertext** ohne Fehlersignal. Im Produktionscode, vor `#[cfg(test)]` (Zeile 121). | [V] | `contextra-crypto/src/aead.rs:71` | Z7, C-5, 6.11 | T-SEAL-2, M-8 |
| **A-2** | `derive_dek` ersetzt einen fehlenden Hierarchiebestandteil (`a.s == None`) durch 32 Nullbytes im Hash. Die Ableitung wird still eine andere. | [V] | `contextra-crypto/src/dek.rs:76` | V-13, 6.3 | T-ARCH-11, T-SEAL-3 |
| **A-3** | `seal_record_with_nonce` ist öffentlich und nimmt die Nonce als Parameter. Die Nonce-Disziplin liegt beim Aufrufer. | [V] | `contextra-crypto/src/aead.rs:24` | I-23, C-2 | T-ARCH-10 |
| **A-4** | `String::from_utf8_lossy(file_id)` bei Schlüsselbezeichnern. Verlustbehaftet. | [V] | `contextra-crypto/src/wal_crypto.rs:229` | I-18, V-3 | T-TYPE-3 |
| **A-5** | Ein hartkodierter Legacy-Integritätsschlüssel steht in `contextra-store/src/wal/hmac.rs:33`. **Nur hinter `#[cfg(feature = "legacy-wal-key")]`**, also nicht im Standardpfad. Die Feature-Kante verändert das Verhalten. | [V] | `contextra-store/src/wal/hmac.rs:33` | I-4, V-8 | T-ARCH-5 |
| **A-6** | `get_segments` liefert bei fehlendem Mandanten `unwrap_or_default()`, also eine leere Liste ohne Fehler. Leseseitig, geringeres Risiko. | [V] | `contextra-crypto/src/kv_segment/store.rs:293` | I-5 | T-LIMIT-1 |
| **A-7** | `DeletionProofKeyPair::generate()` erzeugt Ed25519-Schlüssel im Prozess. Belege sind Selbstauskunft (Klasse A0). | [V] | `contextra-erasure/src/deletion_proof.rs:131` | D-06, I-22 | T-ARCH-9 |
| **A-8** | `crypto/Cargo.toml` setzt `default = ["kv-encryption", "workspace"]`, `erasure/Cargo.toml` setzt `default = ["signing", "vault", "async"]`. Die Regel R10 der Quellspezifikation verbietet `workspace` im Default. | [V] | `Cargo.toml` der Crates | V-12 | T-ARCH-5 |
| **A-9** | In `rust-ci.yml` stehen drei `|| true`: Zeile 28 (`cargo check --workspace`), Zeile 43 (`gate_layering.py`), Zeile 57 (`docid-128`-Check). Ein Gate kann damit nicht fehlschlagen. Ich habe das im Klon gelesen; die Ausführung habe ich nicht geprüft. | [V] | `.github/workflows/rust-ci.yml` | V-14 | T-ARCH-12 |
| **A-10** | Die Quellspezifikation nennt `contextra-vault` als ausgeschlossen (`exclude`). Der Crate-Inhalt liegt in `contextra-vault/src/vault.rs` und wird von `contextra-db` re-exportiert. | [V] | `Cargo.toml` (`exclude`), `crates/contextra-db` | R14 | T-ARCH-1 |
| **A-11** | `contextra-engine` ist eine deprecated Fassade mit 15 Zeilen Quelltext. Das Crate zählt in der Matrix trotzdem als E2. | [V] | `crates/contextra-engine/src/lib.rs` | R9 | T-ARCH-7 |
| **A-12** | Cargo steht im Arbeitsumfeld nicht zur Verfügung. Ich habe nichts gebaut oder getestet. Die Aussagen über Kompilierbarkeit stammen aus `docs/SPEC.md` v5 und sind **[Ü]**. | [V] | Umgebung | 0.2 | — |

**Übernommene Befunde** aus `SPEC.md` 19.1 (B-01 bis B-37, betreffen das Altrepo `tfufuz1/acta`) bleiben gültig. Das Altrepo wurde nicht geklont. Die Zuordnung zu Acta-Regeln steht in `SPEC.md` 19.1.

---

## Anhang A – Konstanten (normativ)

Jede Konstante steht im Code genau einmal (T-DOC-2). Die Tabelle wird vom Testprogramm gelesen.

### A.1 Format

| Name | Wert |
|---|---|
| `ENV_MAGIC` | `"CTX2"` |
| `ENV_VERSION` | `0x01` |
| `ENV_ALG_XCHACHA20POLY1305` | `0x01` |
| `ENV_HEADER_FIXED_LEN` | 88 |
| `ENV_SUBJECT_REF_LEN` | 16 |
| `ENV_NONCE_LEN` | 24 |
| `ENV_TAG_LEN` | 16 |
| `ENV_MAX_SUBJECTS` | 16 |
| `P1_OVERHEAD` | 65536 |
| `P1_VERSION` | 1 |
| `P1_EMB_ELEM_LEN` | 4 |
| `ID_LEN` | 16 |
| `SECRET_LEN` | 32 |
| `KEY_ID_MAX_LEN` | 160 |
| `STORE_ID_LEN` | 16 |
| `LOCAL_CUSTODIAN_FILE_VERSION` | `0x01` |
| `CHRONICLE_ENTRY_ID_LEN` | 16 |
| `CHRONICLE_NO_END_DAY` | 2147483647 |

### A.2 Labels (C-6)

| Name | Wert | Zweck |
|---|---|---|
| `LABEL_DEK` | `"ctx/v2/dek"` | DEK-Ableitung (6.3) |
| `LABEL_CHR_DEK` | `"ctx/v2/chr-dek"` | CDEK-Ableitung (6.4) |
| `LABEL_CHR_GENESIS` | `"ctx/v2/chr-genesis"` | Genesis Chronik (9.2) |
| `LABEL_CHR` | `"ctx/v2/chr"` | Eintragshash (9.2) |
| `LABEL_LOG_GENESIS` | `"ctx/v2/log-genesis"` | Genesis `erase_log` (SPEC 6.6) |
| `LABEL_LOG` | `"ctx/v2/log"` | Eintragshash `erase_log` |
| `LABEL_MANIFEST` | `"ctx/v2/manifest"` | Manifestwurzel (12.6) |
| `LABEL_COMMIT` | `"ctx/v2/commit"` | Commitment-HMAC (12.6) |
| `LABEL_ERASE_COMMIT` | `"ctx/v2/erase-commit"` | Chronik-Inhalt Löschung (12.6) |
| `LABEL_LOCAL_AAD` | `"ctx2/local/"` | AAD `LocalCustodian` |
| `KEY_ID_PREFIX` | `"ctx2/"` | Namensraum (7.2) |

### A.3 Zustands- und Ursachencodes

| Name | Wert | Bedeutung |
|---|---|---|
| `STATE_ACTIVE` | 0 | Active |
| `STATE_GATED` | 1 | Gated |
| `STATE_DESTROYED` | 2 | Destroyed |
| `STATE_COMPLETED` | 3 | Completed |
| `PENDING_CUSTODIAN_UNAVAILABLE` | 1 | |
| `PENDING_CUSTODIAN_INCONSISTENT` | 2 | |
| `PENDING_IO` | 3 | |
| `PENDING_PURGE_FAILED` | 4 | |
| `PENDING_ERASE_PAST_SLA` | 5 | |
| `LOG_KIND_GATED` | 1 | |
| `LOG_KIND_DESTROYED` | 2 | |
| `LOG_KIND_COMPLETED` | 3 | |
| `LOG_KIND_RECONCILED` | 4 | |
| `TARGET_SUBJECT` | 1 | |
| `TARGET_TENANT` | 2 | |
| `LOG_REASON_REQUEST` | 0 | |
| `LOG_REASON_STATE_MISMATCH` | 1 | |
| `LOG_REASON_KEY_GONE` | 2 | |
| `LOG_REASON_REGISTRY` | 3 | |
| `CHR_KIND_NORM` | 1 | NormVersion |
| `CHR_KIND_POLICY` | 2 | PolicyVersion |
| `CHR_KIND_CONFIG` | 3 | ConfigState |
| `CHR_KIND_MODEL` | 4 | ModelVersion |
| `CHR_KIND_DECISION` | 5 | DecisionRule |
| `CHR_KIND_ERASURE` | 6 | ErasureCommitment |
| `CHR_KIND_RECEIPT` | 7 | ContextReceipt (Phase 2) |
| `CHR_KIND_CHECKPOINT` | 8 | Checkpoint |
| `CHR_SCOPE_GLOBAL` | 0 | |
| `CHR_SCOPE_TENANT` | 1 | |
| `METRIC_COSINE` | 0 | |
| `METRIC_DOT` | 1 | |
| `PROTOCOL_AUTHFAIL` | 1 | |

### A.4 Datenbank

| Name | Wert |
|---|---|
| `SQLITE_APPLICATION_ID` | `0x43545832` |
| `SQLITE_USER_VERSION` | 2 |
| `SQLITE_MIN_VERSION` | 3.40 (Prüfung offen, 22.3 Hinweis) |
| `META_KEY_STORE_ID`, `META_KEY_CREATED_MS`, `META_KEY_SCHEMA`, `META_KEY_COMMIT_SEQ` | `"store_id"`, `"created_ms"`, `"schema"`, `"commit_seq"` |
| `LOCK_FILE_NAME` | `"LOCK"` |
| `DB_FILE_NAME` | `"ctx.sqlite3"` |
| `DATA_DIR_MODE` | `0700` |

### A.5 Limits und Zahlen

| Name | Wert |
|---|---|
| `LIMIT_K_MAX` | 1000 |
| `LIMIT_QUERY_BYTES` | 4096 |
| `LIMIT_DIM_MAX` | 4096 |
| `LIMIT_FILTER_VALUES_PER_KEY` | 1024 |
| `LIMIT_TOKEN_BYTES` | 64 |
| `LIMIT_META_KEY_BYTES` | 64 |
| `LIMIT_META_VALUE_BYTES` | 1024 |
| `LIMIT_META_ENTRIES` | 32 |
| `LIMIT_SUBJECTS_PER_DOC` | 16 |
| `LIMIT_MATTER_BYTES` | 128 |
| `DEFAULT_MAX_DOC_BYTES` | 4194304 |
| `DEFAULT_MAX_TEXT_BYTES` | 1048576 |
| `DEFAULT_PURGE_CHUNK_DOCS` | 1000 |
| `DEFAULT_RRF_DEPTH` | 200 |
| `RRF_DEPTH_MAX` | 1000 |
| `BM25_K1` | 1.2 |
| `BM25_B` | 0.75 |
| `RRF_K` | 60 |
| `VECTOR_ACCUMULATORS` | 8 |
| `INDEX_REBUILD_FREE_RATIO_PERCENT` | 25 |
| `DEFAULT_ERASE_RETURN_TIMEOUT_S` | 60 |
| `DEFAULT_ERASE_SLA_S` | 86400 |
| `DEFAULT_CLOCK_SKEW_TOLERANCE_S` | 300 |
| `DEFAULT_HISTORY_HORIZON_DAYS` | 30 |
| `DEFAULT_CHRONICLE_CHECKPOINT_EVERY` | 1024 |
| `DEFAULT_COMMITMENT_EPOCH_DAYS` | 90 |
| `DEFAULT_COMMITMENT_LINKABILITY_DAYS` | 365 |
| `BACKOFF_INITIAL_MS` | 1000 |
| `BACKOFF_MAX_MS` | 900000 |
| `REGISTRY_ENTRY_LEN` | 73 |

### A.6 Feste Aussagetexte

`LIMITS_TEXT` (17.5) steht wörtlich im Nachweispaket und wird von T-DOC-2 gegen den Code verglichen:

> Dieser Nachweis erfasst nicht: Arbeitsspeicher, Auslagerung, Speicherabzüge und Ruhezustand (R1); Sicherungen des Key-Dienstes (R2) und des Datenverzeichnisses (R14); externe Verarbeiter (R5); bereits ausgelieferter Klartext (R6); Schlüsselkopien vor der Vernichtung (R7); die Karenzfrist des Dienstes (R8); Chronik-Fehlklassifikation (R18); die rechtliche Einordnung (R17). Der Nachweis nennt seine Attestierungsklasse; er ist keine Bestätigung der Rechtmäßigkeit.

---

## Anhang B – Testvektoren (normativ)

### B.1 Herkunft

- **B.2 bis B.7** (Envelope, DEK, Manifest, Log, Schlüsselbezeichner): unverändert aus `SPEC.md` Anhang B. Erzeugt am 2026-11-06 mit libsodium (PyNaCl 1.6.2) und eigenem HChaCha20 plus `cryptography` 50.0.1. Erzeugungsprogramm `audit/gen_vectors.py`. Referenzprüfer `audit_open.py`.
- **B.8** (Chronik): am 2026-11-12 erzeugt mit `gen_chronicle_vectors.py` (nur `hashlib`, `hmac`, `struct`). Einzelausführung, keine zweite unabhängige Implementierung. Gegenprüfung in AP1 mit `cryptography` (HMAC) und einem zweiten Skript.

Alle Schlüssel in diesem Anhang sind **öffentliche Testwerte**.

### B.2 Chronik (neu, Stand 2026-11-12)

```
store_id       = f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff
genesis_chr    = SHA-256("ctx/v2/chr-genesis" ‖ 0x00 ‖ store_id)
               = 62b529396f1da12c64306612cd470f92303ae33408a139cfa2578889dacd9852

NORM_CONTENT   = SHA-256("Beispielnorm v1 (Testtext)")
               = 5ee245f450573f1745efb24f10c7901dfd2c11c8710acca28807f7a7d6d05ba7

Eintrag 1:     seq = 1, kind = 1 (NormVersion), valid_from = 19000, valid_to = 2147483647,
               recorded_ms = 1762400000000, content_hash = NORM_CONTENT,
               matter = "Policy/Reisekosten", supersedes = 16 × 0x00, prev_hash = genesis_chr
entry_hash     = 3d14bbfc8b552919013d981bffeccaf2eb288c69d712f36dbf0d551863b2a51a
```

Die Eingabe der Hashfunktion ist in 9.2 festgelegt. Die Zahlen `valid_from` und `valid_to` sind vorzeichenbehaftet, big-endian, 4 Byte.

### B.3 Commitment (neu, Stand 2026-11-12)

```
K_COMMIT       = 60 61 62 … 7f                     (öffentlicher Testwert, 32 Byte)
tenant_id      = a1 × 16
subject_ref    = 01 × 16
commitment     = HMAC-SHA256(K_COMMIT, "ctx/v2/commit" ‖ 0x00 ‖ tenant_id ‖ subject_ref)
               = cc667cf8bf8b33840131d4a74223cc66031ca6ba770951815910bcded57b5c47
```

### B.4 Referenzprüfer

`audit_open.py` (SPEC Anhang B.4) öffnet Envelopes. Neu: `audit_chronicle.py` berechnet `genesis_chr`, `entry_hash` und die Kette aus einer exportierten Tabelle und prüft Commitments gegen `K_COMMIT`. Beide Skripte sind Teil von `audit/`.

---

## Anhang C – Zulässiger Wortlaut

### C.1 Standardaussage

> *„Acta vernichtet den Schlüssel des Betroffenen beim Key-Dienst und macht dessen gespeicherte Daten dadurch unlesbar. Ob das im Einzelfall eine Löschung nach Art. 17 DSGVO darstellt, beurteilt der Verantwortliche.“*

### C.2 Zusätze je Klasse

Wie 17.4. Nicht verwendbar in Klasse A0: C.1.

### C.3 Zeitreise-Aussage (neu)

> *„Acta kann für Normen und Konfigurationen ohne Personenbezug den Stand zu einem gewählten Stichtag wiedergeben. Personenbezogene Daten sind nur innerhalb des konfigurierten Versionshorizonts abrufbar.“*

### C.4 Pflichthinweis

Jede Außenaussage über einen einzelnen Satz hinaus MUSS `LIMITS_TEXT` (A.6) enthalten oder darauf verweisen.

### C.5 Nicht zulässig

„DSGVO-konform“, „garantiert gelöscht“, „unwiederbringlich gelöscht“, „zertifiziert“, „fehlerfrei“, „bewiesen sicher“, „Löschung aus allen Sicherungen“, „revisionssicher“, „Zeitreise mit vollständiger Historie personenbezogener Daten“.

---

*Ende der Spezifikation Fassung 1.0-E (Entwurf). Begleitdokument: `ACTA_DATEIZUORDNUNG.md`.*

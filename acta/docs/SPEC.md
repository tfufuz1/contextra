# Acta – Spezifikation

*Wissens- und Entscheidungsplattform mit versionierbarem Unternehmenswissen und entkoppeltem Personendatenschutz*
*Fassung 1.0 · Stand 2026-11-15 · Status: normativ für alle Acta-Repositories*

Normative Schlüsselwörter nach RFC 2119: **MUSS**, **DARF NICHT**, **SOLL**, **KANN**.

Kennzeichnung der Aussagen:

| Marke | Bedeutung |
| --- | --- |
| **[E]** | Verbindliche Architekturentscheidung dieser Spezifikation |
| **[R]** | Rechtliche Anforderung, aus der eine technische Eigenschaft folgt |
| **[Q]** | Messbares Qualitätsziel. Die Zahl ist ein Startwert; freigegeben wird nur nach gemessenem Bericht |
| **[A]** | Annahme über die Umgebung, die im Betrieb zu bestätigen und im Bericht auszuweisen ist |

---

## 1. Produkt

### 1.1 Zweck

Acta ist eine eigenständige Software für **Behörden und mittelständische Unternehmen**, mit der Organisationen ihr Wissen verwalten, ordnen und durch ein Sprachmodell nutzbar machen. Drei Eigenschaften machen Acta besonders:

1. **Belegtes Antworten (RAG).** Jede Antwort stützt sich auf konkrete, zitierte Quellen. Fehlt der Beleg, enthält sich das System.
2. **Versionskontrolle für Wissen („Git für RAG“).** Der gesamte nicht personenbezogene Wissensbestand samt Richtlinien, Prompts, Modellständen und Grenzwerten ist unveränderlich versioniert. Damit lassen sich alte Entscheidungen nachvollziehen und „Wenn-Dann“-Szenarien auf Verzweigungen des Wissens durchspielen.
3. **Entkoppeltes, nachweisbares Löschen personenbezogener Daten.** Personenbezogene Daten liegen getrennt vom versionierten Wissen, sind verschlüsselt und werden durch Schlüsselvernichtung und physische Bereinigung unlesbar gemacht, ohne die Versionsgeschichte zu verändern.

### 1.2 Zielgruppen und Einsatzrahmen

| Merkmal | Festlegung [E] |
| --- | --- |
| Betreiber | Behörden, Kommunen, Mittelstand; Betrieb On-Premise oder in einer vom Kunden kontrollierten EU-Umgebung |
| Mandanten | Mehrere Organisationseinheiten je Installation (z. B. Ämter, Gesellschaften), strikt getrennt |
| Nutzer | Sachbearbeitung, Fachabteilungen, Datenschutzbeauftragte, Administratoren, Revision |
| Inferenz | Lokal oder in vom Kunden kontrollierter Infrastruktur; externe Dienste nur durch ausdrückliche Freigabe |
| Plattform | Linux x86_64 und aarch64; ein Knoten pro Installation (Hochverfügbarkeit siehe 14.5) |
| Netz | Betrieb ohne ausgehende Verbindungen ist der Standard und voll funktionsfähig |

### 1.3 Mission und Umsetzung

| Säule | Umsetzung in Acta | Abschnitt |
| --- | --- | --- |
| Souveränes Unternehmenswissen | On-Premise, Schlüssel beim Kunden, kein Egress im Standard, lokale Inferenz | 4, 12 |
| Transparenz und Nachvollziehbarkeit | Versionierung, Entscheidungsprotokoll, Hash-Kette, unabhängiger Prüfer | 6, 7, 15 |
| Präzision und Kontrolle | Belegpflicht, Hybridsuche mit Reranking, Zugriffskontrolle vor der Suche, Messbarkeit | 9, 10 |
| Effizienz und Ressourcenschonung | Segmentierte Indizes, Rebuild statt Vollversiegelung, Batch-Vernichtung, kleine Binärdatei | 5, 9, 13 |
| Schutz personenbezogener Daten | Ebene P, Schlüsselhierarchie, Lesesperre, versiegelte Ableitungen | 2, 4, 8 |
| Verwaltung und Organisation von Daten | Mandanten, Sammlungen, Metadatenschema, Aufbewahrungsklassen, Löschkonzept | 10, 11 |
| Entscheidungs- und Verwaltungshilfe | Entscheidungsprotokoll, Zeitreise, Szenarien, Berichte | 6, 7, 16 |
| Europäischer Datenschutz | Abläufe zu Art. 5, 12, 16 bis 19, 25, 30, 32 DSGVO, Berichte mit Annahmen | 8, 16, 17 |

### 1.4 Nicht-Ziele (Version 1)

- Kein Dokumentenmanagementsystem für Originale und keine Langzeitarchivierung (Schnittstelle zu vorhandenen Systemen).
- Keine Mehrknoten-Replikation (14.5).
- Keine autonomen Agenten und keine Werkzeugausführung durch das Sprachmodell.
- Keine grafische Oberfläche als Teil dieser Spezifikation; Referenzoberflächen sind eigene Projekte über die API.
- Keine Zertifizierungsaussage (1.6).

### 1.5 Normative Zusagen

Jede Zusage MUSS durch automatisierte Tests belegt sein. Eine Zusage ohne Test gilt als nicht erfüllt.

| ID | Zusage | Primärer Nachweis |
| --- | --- | --- |
| **Z1** | **Sofortige Lesesperre.** Nach Rückkehr einer Löschung im Zustand `Gated` liefert kein Pfad des Systems Klartext des Ziels: nicht Suche, Antwort, Zeitreise, Szenario, Cache, Export oder Sitzung. Schreibzugriffe auf das Ziel werden verweigert. | `T-GATE-*` |
| **Z2** | **Kryptografische Isolation.** Jeder persistierte Klartext personenbezogener Daten und jede daraus abgeleitete Struktur liegt vor dem Schreiben in einem AEAD-Umschlag unter Schlüsseln, die nach der Vernichtung nicht mehr existieren. | `T-CRY-*`, `T-DERIVED-SEALED` |
| **Z3** | **Mehrbetroffenen-Konsistenz.** Betrifft ein Datensatz mehrere Betroffene, wird er unlesbar, sobald einer von ihnen gelöscht wird (Modus `DeleteAll`); Mitbetroffene werden im Bericht deklariert. | `T-MULTI-*` |
| **Z4** | **Crash-Invarianz.** Ein Absturz an jedem Durabilitätspunkt führt nach dem Neustart entweder zur Vollendung der Löschung oder zu `Pending` bei dauerhaft aktiver Sperre. Eine gesetzte Sperre geht nie verloren. | `T-CRASH-*` |
| **Z5** | **Trennung von Wissen und Personendaten.** Die versionierte Wissensebene K enthält keine personenbezogenen Daten. Verweise auf Personendaten sind zufällige Handles ohne Inhaltsbezug. Ein Schreibversuch mit erkanntem Personenbezug scheitert (Fail-Closed). | `T-SEP-*`, `T-ANON-*` |
| **Z6** | **Löschung berührt die Versionskontrolle nicht.** Eine Löschung in Ebene P ändert weder Objekt-IDs noch Commit-Hashes noch die Hash-Kette der Wissensebene. | `T-VCS-ERASE-*` |
| **Z7** | **Zeitreise-Determinismus.** Abfragen der Wissensebene zu einem Stand liefern bei gleichen Parametern byte-identische Inhalte. | `T-TT-*` |
| **Z8** | **Entscheidungsnachvollziehbarkeit.** Zu jeder Antwort existiert ein Entscheidungsprotokoll mit Wissensstand, Prompt-Version, Modellstand, Parametern und Belegen. Nach Löschung einer Person bleibt die Begründungskette erhalten; personenbezogene Eingaben liefern `Erased`. | `T-DEC-*` |
| **Z9** | **Szenario-Isolation.** Ein Szenario verändert weder die Hauptlinie des Wissens noch Ebene P. | `T-SCEN-*` |
| **Z10** | **Fail-Closed-Kryptografie.** Kein Fehler im Schlüssel-, Zufalls-, Ableitungs- oder Siegelpfad führt zu Ersatzwert, Fallback-Schlüssel oder Klartext. Es gibt keine Standardschlüssel. | `T-NOFALLBACK-*`, `T-RNG-*` |
| **Z11** | **Nachweis nur mit Positivkontrolle.** Ein Löschnachweis entsteht ausschließlich aus einer Messung, deren Messverfahren seine Funktionsfähigkeit an einem synthetischen Treffer bewiesen hat. Nicht implementierte Prüfungen liefern `NotImplemented`. | `T-EVID-*` |
| **Z12** | **Restore-Schutz.** Das Einspielen einer alten Sicherung kann gelöschte Daten nicht wiederbeleben. | `T-REST-*` |
| **Z13** | **Herkunftszwang.** Jede abgeleitete Struktur trägt ihre Herkunft; Ableitungen ohne Herkunft sind nicht konstruierbar. | Compile-Fail-Tests |
| **Z14** | **Schlüsselisolation je Dokument.** Zwei Dokumente der Ebene P teilen niemals denselben Datenschlüssel. | Property-Test |
| **Z15** | **Pseudonymitätsfreie Protokolle.** Protokolle enthalten weder Schlüsselmaterial noch Roh-Kennungen von Personen. | `T-LOG-*` |
| **Z16** | **Zugriffskontrolle vor der Suche.** Ein Nutzer erhält nie Treffer, Belege oder Texte, auf die er keinen Zugriff hat; die Prüfung wirkt auf die Kandidatenmenge, nicht erst auf die Ausgabe. | `T-ACL-*` |
| **Z17** | **Belegpflicht.** Jede inhaltliche Aussage einer Antwort trägt einen Quellenverweis auf einen abgerufenen Beleg, oder die Antwort enthält die ausdrückliche Enthaltung „nicht belegt“. | `T-GROUND-*` |
| **Z18** | **Kein Egress im Standard.** Ohne ausdrückliche, protokollierte Freigabe verlässt kein Inhalt die Installation. | `T-EGRESS-*` |

### 1.6 Sprachregeln und Berichtsgrenzen [R]

In Berichten, Oberflächen, Dokumentation und Werbematerial sind **verboten**: „DSGVO-konform“, „DSGVO-zertifiziert“, „100 % gelöscht“, „garantiert vernichtet“, „unwiederbringlich“, „revisionssicher“, „fehlerfrei“, „löscht aus allen Backups“, „perfekte Antworten“. Konformität ist ein Betriebszustand der Organisation, keine Eigenschaft einer Software.

Acta berichtet ausschließlich **messbare Erfüllungsklassen** (8.7) mit `reached` oder `not_reached` und weist die **Annahmen** A-FLASH, A-RAM, A-ANCHOR, A-CRYPTO und A-EXT (17.3) aus. Ein Wortfilter (Gate G-WORD) prüft Quelltext, Texte und Berichtsvorlagen.

---

## 2. Grundmodell

### 2.1 Vier Ebenen

```
┌───────────────────────────────────────────────────────────────────────────┐
│ Ebene K – Wissen (versioniert, unveränderlich, ohne Personenbezug)        │
│   Blobs, Trees, Commits, Branches, Tags, Entscheidungsprotokolle          │
│   Verweise auf Personendaten nur als zufällige PersonalRef-Handles        │
├───────────────────────────────────────────────────────────────────────────┤
│ Ebene P – Personendaten (verschlüsselt, löschbar, nicht Teil der Hashes)  │
│   Datensätze unter Dokument- und Betroffenenschlüsseln                    │
│   Kurzer Versionsverlauf nur für Berichtigung und Fristen                 │
├───────────────────────────────────────────────────────────────────────────┤
│ Ebene D – Ableitungen (Indizes, Caches, Embeddings)                       │
│   Tragen Herkunft (Lineage); aus P abgeleitete Segmente sind versiegelt   │
├───────────────────────────────────────────────────────────────────────────┤
│ Ebene E – Nachweise (Löschjournal, Ledger, Berichte, Schlüsseltresor)     │
└───────────────────────────────────────────────────────────────────────────┘
```

**Kerngedanke [E]:** Versionskontrolle und Löschung sind **konstruktiv entkoppelt**. Die Versionsgeschichte (K) enthält nie Inhalte, die gelöscht werden müssen. Sie enthält an deren Stelle ein **PersonalRef**, ein zufälliges 128-Bit-Handle ohne Ableitung aus dem Inhalt. Der Hash eines Trees oder Commits wird über das Handle gebildet, nie über Personendaten. Die Löschung vernichtet den Schlüssel in Ebene P; das Handle bleibt gültig und löst nach der Löschung zu `Erased` auf.

### 2.2 Datenklassen (Regel R-CLS)

Jeder Schreibvorgang MUSS eine Datenklasse deklarieren. Fehlt sie, bricht die Operation mit `ClassViolation` ab.

| Klasse | Ebene | Beschreibung | Aufbewahrung (Standard) | Löschmechanismus |
| --- | --- | --- | --- | --- |
| `Wissen` | K | Sachinhalte ohne Personenbezug: Vorschriften, Richtlinien, Handbücher, Vorlagen, Konfigurationen, Prompts | unbegrenzt (versioniert) | Takedown (7.6) |
| `Beleg` | K | Entscheidungsprotokolle ohne Personenbezug | unbegrenzt | nicht löschbar außer Takedown |
| `Personal` | P | Personenbezogene Daten | nach Aufbewahrungsklasse | Schlüsselvernichtung und Bereinigung |
| `Besonders` | P | Daten nach Art. 9 und 10 DSGVO | 0 Tage Karenz | sofortige Schlüsselvernichtung, priorisierte Bereinigung |
| `Abgeleitet` | D | Indizes, Caches, Embeddings | keine eigene Frist | herkunftsbasierte Bereinigung, Rebuild |
| `Unklassifiziert` | P | Eingang ohne Entscheidung | Quarantäne, max. 24 h | wie `Personal`; Sperre bei Fristablauf |

### 2.3 Dokumente mit gemischtem Inhalt [E]

Ein Dokument kann Sach- und Personenbezug mischen. Der Ingest wählt je Dokument genau einen Modus:

| Modus | Wirkung | Wann |
| --- | --- | --- |
| `Whole` (Standard bei erkanntem Personenbezug) | Das gesamte Dokument liegt in Ebene P. In K entsteht höchstens ein PersonalRef. | Akten, Fallbearbeitung, Schreiben an Personen |
| `Split` | Personenbezogene Spannen werden entfernt und in P abgelegt; K enthält den bereinigten Text mit Platzhaltern `⟦P:<handle>⟧`. Pflicht: bestandene Prüfung des N-Validators (7.5) | nur bei ausdrücklicher Richtlinie des Mandanten |
| `Open` | Das Dokument ist vollständig `Wissen` | Dokument besitzt bestätigt keinen Personenbezug (Vorschrift, Handbuch) |

**Begründung:** Eine automatische Erkennung kann Personenbezug übersehen. Deshalb ist `Whole` der sichere Standard, und `Split` und `Open` setzen eine bestandene Validierung voraus. Verbleibendes Restrisiko behandelt der Takedown (7.6).

### 2.4 Identifikatoren

| Typ | Größe | Bildung | Zweck |
| --- | --- | --- | --- |
| `TenantTag` | 16 B | HMAC-SHA256(`k_idx`-Teilschlüssel, `acta/tag/v1` ‖ „tenant“ ‖ externe ID), gekürzt | Mandant |
| `SubjectTag` | 16 B | wie oben, Teilschlüssel je Mandant | Betroffener |
| `DocRef` | 16 B | zufällig (`CryptoRng`) | Dokument in Ebene P |
| `PersonalRef` | 16 B | zufällig | Verweis aus K auf Personendaten |
| `DocOrd` | u32 | dicht, monoton je Mandant, nie wiederverwendet | Herkunft (Lineage), nur intern |
| `KOrd` | u64 | dicht, monoton je Installation | Sichtbarkeitsmengen (6.5), nur intern |
| `ObjectId` | 32 B | SHA-256 über kanonische Bytes mit Domänenpräfix | Objekte der Ebene K |

`k_idx` ist ein 256-Bit-Geheimnis ohne Standardwert, geliefert durch den Schlüsselanker (4.4). Ein Start ohne `k_idx` wird verweigert. Tags sind niemals in Ebene K sichtbar.

---

## 3. Systemarchitektur

### 3.1 Prinzipien [E]

1. **Anforderung vor Mechanismus.** Kein Bauteil ohne Zusage oder Produktanforderung.
2. **Eigenbau nur dort, wo physische Kontrolle erforderlich ist** (Segmentspeicher, Schlüsseltresor, Löschkern, Versionskontrolle). Sonst bewährte Bibliotheken.
3. **Fehler unmöglich machen statt verbieten:** Typzwang (`Sealed<P>`, `Derived<T>`, Schlüsseltypen ohne `Default`).
4. **Eine Lösung je Problem:** ein Hash-Ketten-Typ, ein Zeit- und Zufallsport, ein Fehlerklassifikator.
5. **Kleine Crates, harte Größenbudgets, keine Sammel-Crates** mit Fremdwissen.
6. **`unsafe` nur in `acta-sys`.**
7. **Kern synchron.** `tokio` nur an den Rändern (Netzwerk, Inferenzclient), nie in Level 0 bis 6.

### 3.2 Level-Modell [E]

Eine Abhängigkeit `A → B` ist erlaubt genau dann, wenn `level(A) > level(B)`. Die Level stehen in `rings.toml`; das Gate G1 prüft sie gegen `cargo metadata`. Querkanten auf gleichem Level sind verboten.

### 3.3 Crate-Verteilung

Größenbudgets sind harte Obergrenzen für Produktionscode (ohne Tests); Überschreitung erfordert eine dokumentierte Entscheidung.

| Level | Crate | Aufgabe | Budget (LOC) | Interne Abhängigkeiten |
| --- | --- | --- | --- | --- |
| L0 | `acta-base` | Newtypes (Tags, Refs, Ordinale), `DataClass`, `ErrorClass`, `ClassifiedError`, Ports `Clock` und `CryptoRng` (fallibel), kanonische Kodierung (sicheres Lesen/Schreiben von Big-Endian-Feldern), Grenzwerte | 2 500 | – |
| L1 | `acta-crypto` | AEAD (XChaCha20-Poly1305), HKDF, HMAC, Ed25519, `Sealed<P>`, Schlüsseltypen (`Zeroizing`), `HashChain` | 4 000 | base |
| L1 | `acta-fs` | `Vfs`-Port, `DurableVfs` (fsync, atomares Ersetzen, Scrub), `FaultVfs` (Fehlerinjektion) | 2 500 | base |
| L1 | `acta-sys` | Einziges `unsafe`: `mlock`, `MADV_DONTDUMP`, `RLIMIT_CORE`, `SecureBuf`; `SystemClock`, `OsCryptoRng` | 1 200 | base |
| L1 | `acta-gate` | `ReadGate` (Roaring-Bitmap, Epoche), Traits `ErasedView`, `ErasureLayer`, `LayerAck` | 900 | base |
| L1 | `acta-access` | Mandanten, Rollen, Attribute, Richtlinienauswertung (ABAC), Sichtbarkeitsmengen je Gruppe | 3 000 | base |
| L1 | `acta-infer` | Verträge `Embedder`, `Reranker`, `Generator`, `ModelFingerprint`, Kostenbeschreibung | 600 | base |
| L1 | `acta-pii` | Erkennung personenbezogener Muster (Regeln, Wörterbücher, Prüfsummen), N-Validator-Bausteine | 4 000 | base |
| L1 | `acta-ingest` | Parser (nicht vertrauenswürdige Eingaben), Chunking, Ressourcenlimits | 8 000 | base |
| L2 | `acta-ledger` | Append-only-Protokoll mit Hash-Kette, signierten Checkpoints, keyed Commitments | 2 500 | base, crypto, fs |
| L2 | `acta-vault` | Schlüsseltresor, `KeyAnchor`-Trait, `FileAnchor`, Wurzelrotation, Vernichtung | 4 500 | base, crypto, fs |
| L2 | `acta-seg` | Versiegelter Segmentspeicher: Journal, Segmente, Manifest, Aufräumen, Kopienregister | 7 000 | base, crypto, fs, gate |
| L3 | `acta-history` | Versionskontrolle der Ebene K: Objekte, Refs, Sichtbarkeitsmengen, Diff, Merge | 6 000 | base, crypto, ledger, seg, gate |
| L3 | `acta-erasure` | Löschkern: Gate-Journal, Zustandsmaschine, Messung, Bericht, Restore-Schutz | 8 000 | base, crypto, fs, gate, ledger, vault |
| L3 | `acta-text` | Segmentierter Volltextindex (BM25) mit Lineage, DE/EN-Normalisierung | 5 000 | base, crypto, gate, seg |
| L3 | `acta-vec` | Segmentierter Vektorindex (Flat, später ANN-Backend), Lineage | 4 000 | base, crypto, gate, seg |
| L4 | `acta-decision` | Entscheidungsprotokoll, Szenarien (Wenn-Dann), Vergleich | 4 000 | base, history, access, infer |
| L4 | `acta-rag` | Abrufpipeline: Hybridsuche, Fusion, Reranking, Belegprüfung, Bewertungswerkzeug | 7 000 | base, text, vec, history, gate, access, infer |
| L4 | `acta-reports` | Löschbericht (JCS), Verzeichnis nach Art. 30, Fristen- und Aufbewahrungsberichte, Wortfilter | 3 500 | base, erasure, ledger, history |
| L5 | `acta-engine` | Anwendungsfälle: Ingest, Antwort, Löschung, Szenario, Aufbewahrung, Wartung | 12 000 | alle darunter |
| L6 | `acta-node` | Composition Root: Konfiguration, Verdrahtung, Selbsttest beim Start, Hintergrundaufgaben | 3 000 | engine, alle Adapter |
| L6 | `acta-infer-http` | Client für lokale und freigegebene Inferenzdienste (OpenAI-kompatible Schnittstelle); darf `tokio` nutzen | 3 000 | base, infer |
| L7 | `acta-api` | HTTP/JSON-Schnittstelle, Authentifizierung (OIDC, mTLS), Streaming; darf `tokio` nutzen | 6 000 | node |
| L8 | `acta-server` (Binary) | Prozessstart, Signalbehandlung | 500 | api, node |
| L8 | `acta-cli` (Binary) | Administration: Betroffenenanfragen, Szenarien, Berichte, Prüfung | 3 000 | node |
| – | `acta-verify` (Binary) | Unabhängige Prüfung von Löschberichten und Ledger; **keine** interne Abhängigkeit | 800 | – |
| dev | `acta-testkit` | `ManualClock`, `SeededRng`, Fehlerinjektion, Konformitätstests, Testdaten | 5 000 | base, crypto, fs |
| dev | `xtask` | `cargo xtask gates`: Architektur- und Hygieneprüfungen | 3 000 | – |

Zusammen sind es 24 Produktions- und 2 Entwicklungs-Crates; Binaries sind `acta-server`, `acta-cli`, `acta-verify`.

### 3.4 Hub-Regeln [E]

Es gibt **keinen** Sammel-Crate für „Typen“ oder „Ports“. Gemeinsame Verträge sind kleine, einem Zweck gewidmete Crates (`acta-base`, `acta-gate`, `acta-infer`).

| Regel | Inhalt |
| --- | --- |
| HR-1 | Ein Vertrags-Crate hat genau einen Zweck und bleibt unter seinem Budget |
| HR-2 | Ein Trait gehört dem Crate, das ihn **braucht**, wenn der Erbringer in einem höheren Level liegt (Dependency Inversion); liegt der Erbringer tiefer, besitzt der Erbringer ihn |
| HR-3 | Nutzen Erbringer und Nutzer einen Vertrag aus verschiedenen Richtungen, entsteht ein eigenes kleines Vertrags-Crate auf dem niedrigsten tragfähigen Level |
| HR-4 | Ein Fehlertyp je Crate (`thiserror`, `#[non_exhaustive]`, kein `Other(String)`); Aggregation erst ab L5 |
| HR-5 | `default = []`; Features sind additiv und haben keine Rückkante nach oben |
| HR-6 | Keine Alias-Re-Exports, kein `include!`, kein `#[deprecated]`, keine Brücken-Module |
| HR-7 | `acta-node` ist die einzige Stelle, die konkrete Implementierungen verdrahtet; jeder Port hat im Betrieb genau eine Implementierung |
| HR-8 | Fan-in-Limit: Von `acta-engine` hängen höchstens `acta-node` und `acta-cli` ab |

### 3.5 Fehlerarchitektur

```rust
#[non_exhaustive]
pub enum ErrorClass { Transient, Integrity, Crypto, Erased, Policy, Invalid }

pub trait ClassifiedError: std::error::Error {
    fn class(&self) -> ErrorClass;
    fn code(&self) -> &'static str; // stabil, ohne Nutzdaten und Kennungen
}
```

`Integrity` und `Crypto` lösen Vergiftung aus (13.2). `Display` und `code()` DÜRFEN keine Personendaten oder Roh-Kennungen enthalten (Z15).

### 3.6 Laufzeitmodell [E]

Der Kern (Level 0 bis 6) ist **synchron** und nutzt Threads. **Begründung:** Durabilitätspunkte lassen sich in synchronem Code deterministisch durch Fehlerinjektion prüfen; es entfällt der Nachweis der Abbruchsicherheit an jedem `fsync`; Modelltests (Loom) sind direkt anwendbar. Netzwerk und Inferenzclients laufen asynchron und rufen den Kern über einen begrenzten Thread-Pool auf. **Preis:** Brücke am Rand. **Revisionstrigger:** Messung zeigt, dass die Bestätigung der Sperre (`erase_return_timeout`) unter Last verletzt wird.

---

## 4. Kryptografie und Schlüsselverwahrung

### 4.1 Primitive [E]

| Zweck | Verfahren | Bibliothek |
| --- | --- | --- |
| Datensatz-Verschlüsselung | XChaCha20-Poly1305, 256-Bit-Schlüssel, 192-Bit-Zufallsnonce aus `CryptoRng` (kein API-Parameter) | `chacha20poly1305` |
| Schlüsselableitung | HKDF-SHA256 (RFC 5869); Fehler werden propagiert | `hkdf`, `sha2` |
| Passwortbasierte Ableitung (nur `FileAnchor`) | Argon2id, m ≥ 19 456 KiB, t ≥ 2, p ≥ 1 | `argon2` |
| Integrität und Commitments | HMAC-SHA256 | `hmac` |
| Objekt- und Kettenhashes | SHA-256 | `sha2` |
| Signatur (Berichte, Checkpoints) | Ed25519 (RFC 8032), Domänenpräfix `acta/report/v1` bzw. `acta/checkpoint/v1` | `ed25519-dalek` |
| Speicherhygiene | `Zeroizing`, konstante Zeit beim Vergleich | `zeroize`, `subtle` |

**Begründung der Auswahl:** Ein einziger Primitivsatz (SHA-2-Familie, ChaCha, Ed25519) hält den unabhängigen Prüfer klein (≤ 800 LOC) und das Krypto-Audit überschaubar. Die 192-Bit-Zufallsnonce macht zufällige Wahl unproblematisch. **Preis:** XChaCha20-Poly1305 ist kein FIPS-140-Algorithmus. Das Umschlagformat trägt ein Algorithmus-Byte (`alg`); `0x02` ist für AES-256-GCM reserviert und wird nur auf schriftliche Anforderung implementiert. Eigene Primitive sind verboten.

Alle Domänenpräfixe haben die Form `acta/<zweck>/v<N>`.

### 4.2 Schlüsselhierarchie [E]

```
              KeyAnchor (HSM, KMS oder FileAnchor)
                        │  liefert Tresor-Master M_g (Generation g)
              ┌─────────▼──────────┐
              │   Schlüsseltresor   │   Einträge sind einzeln versiegelt unter M_g
              └─────────┬──────────┘
        ┌───────────────┼───────────────────────────┐
   SubjectKey(s)    DocumentKey(d)             SegmentKey(x)
   je Betroffener   je Dokument der Ebene P    je versiegeltem Segment (K, D)
        └───────┬────────┘
                ▼
   DEK(d) = HKDF-SHA256( IKM  = DocumentKey(d) ‖ SubjectKey(s₁) ‖ … ‖ SubjectKey(sₙ),
                         salt = DocRef(d),
                         info = "acta/dek/v1" )          s₁ … sₙ aufsteigend nach SubjectTag
```

**Wirkung:**
- Wird `SubjectKey(sᵢ)` vernichtet, sind **alle** Dokumente dieser Person und alle Dokumente, an denen sie mitbeteiligt ist, unlesbar (Z3, Modus `DeleteAll`, ohne Sonderlogik).
- Wird `DocumentKey(d)` vernichtet, ist genau dieses Dokument unlesbar (Einzellöschung).
- Zwei Dokumente haben nie denselben DEK (Z14).
- Für ein Dokument sind höchstens `max_subjects_per_doc` = 64 Betroffene zulässig; darüber behandelt der Ingest es als Sammelakte unter einem Gruppenschlüssel (`SubjectKey` einer Gruppe).

**Alternative verworfen:** Salts mit Maskierung im Tresor. Das erzeugt zusätzliche Spezialformate und eine eigene Rotationslogik. Die gewählte Ableitung braucht pro Schlüssel einen versiegelten Eintrag und nutzt Wurzelrotation (4.3).

Fehlt ein erforderlicher Schlüssel, ist das Ergebnis `Err(Erased)`. Ist der Zustand nicht eindeutig (Tresor nicht lesbar), ist das Ergebnis `Err(Crypto)` mit Vergiftung. Ein Fallback ist ausgeschlossen (Z10).

### 4.3 Tresor und Wurzelrotation

**Eintrag (Klartext):** `[kind u8][status u8][id 16][key 32]` = 50 Byte; versiegelt zu `[len u16][nonce 24][ciphertext 50][tag 16]` mit AAD `acta/vault/v1 ‖ generation u32 ‖ id`. `kind`: 0x01 `SubjectKey`, 0x02 `DocumentKey`, 0x03 `SegmentKey`, 0x04 `TenantKey` (Reserve). `status`: 0x01 aktiv, 0x02 vernichtet (Schlüssel genullt).

**Vernichtung:** Der Eintrag wird durch einen Grabstein ersetzt (`status = 0x02`, `id` genullt, `key` genullt), die Datei wird synchronisiert, und der Zustand wird direkt vom Datenträger zurückgelesen (Direct-Read-Verifikation). Ein Grabstein trägt keine Kennung, damit er kein dauerhaftes Pseudonym darstellt.

**Wurzelrotation (Backup-Härtung):** Alle `T_ROT` = 24 h oder nach `N_SHRED` = 10 000 Vernichtungen erzeugt der Anker eine neue Generation `M_{g+1}`. Alle lebenden Einträge werden neu versiegelt, atomar in eine neue Tresordatei überführt (`fsync`, `rename`, `fsync` des Verzeichnisses), danach vernichtet der Anker `M_g`. Eine Kopie des Tresors aus Generation ≤ g ist danach nicht mehr lesbar. Journal-Eintrag `RootRotated { generation }`.

**Batch-Vernichtung:** `destroy_many(&[KeyId])` bündelt Schreibvorgänge und `fsync` (Durchsatz bei Fristablauf-Läufen).

**Tresorgröße:** 92 Byte je Eintrag; bei 50 Mio. Dokument- und Personenschlüsseln ca. 4,6 GB. Der Tresor wird sortiert in Shards gehalten und über einen Speicherindex (`KeyId → Offset`) bedient. **[Q]** Startwert: Öffnen des Tresors mit 10 Mio. Einträgen unter 30 s; Messung vor Freigabe.

### 4.4 KeyAnchor

```rust
pub trait KeyAnchor: Send + Sync {
    fn current_generation(&self) -> Result<u32, AnchorError>;
    fn master(&self, generation: u32) -> Result<MasterKey, AnchorError>;   // MasterKey: Zeroizing, kein Clone/Default
    fn rotate(&self) -> Result<u32, AnchorError>;                           // neue Generation
    fn destroy(&self, generation: u32) -> Result<DestroyProof, AnchorError>;
    fn index_secret(&self) -> Result<IndexSecret, AnchorError>;             // k_idx
}
```

Implementierungen: `FileAnchor` (Entwicklung und kleine Installationen; Wurzel durch Passphrase und Argon2id geschützt; Kennzeichnung im Bericht als geringe Vertrauensstufe), `Pkcs11Anchor` und `KmsAnchor` (Produktion; getrennte Crates oder Features, additiv). **[A] A-ANCHOR:** Der Anker kann ein Generationsgeheimnis innerhalb einer deklarierten Frist `anchor_destroy_max` unwiderruflich vernichten. Besitzt ein KMS eine Pflicht-Löschfrist, MUSS `anchor_destroy_max` diese Frist enthalten, und der Bericht weist die tatsächliche Frist aus. `destroy` liefert einen `DestroyProof`, den der Bericht übernimmt.

### 4.5 Umschlag v1

```
┌──────────┬──────────┬──────────────┬────────────┬──────────────────────┬──────────┐
│ ver (u8) │ alg (u8) │ key_class u8 │ nonce (24) │  ciphertext (varlen) │ tag (16) │
│   0x01   │   0x01   │ 0x01 P/0x02 K│            │                      │ Poly1305 │
└──────────┴──────────┴──────────────┴────────────┴──────────────────────┴──────────┘
AAD = "acta/rec/v1" ‖ tenant_tag ‖ class_u8 ‖ keyspace_u16 ‖ record_ref(16) ‖ version_u64be
```

`open` liefert `Zeroizing<Vec<u8>>`. Jeder Fehler (Länge, MAC, Schlüssel) führt ohne unterscheidbares Orakel zu `DecryptionFailed`; ist der Schlüssel nachweislich vernichtet, zu `Erased`. Die Nonce ist **kein** Parameter der öffentlichen Schnittstelle.

### 4.6 Zufall, Zeit, Geheimnisse

`CryptoRng::try_fill(&self, dst) -> Result<(), EntropyError>` ist fehlbar; ein Entropiefehler erzeugt nie einen Umschlag (`T-RNG-*`). `Clock` ist ein Port; `SystemTime::now` und `thread_rng` sind im Kern verboten. Geheimnisse (`Dek`, `MasterKey`, `IndexSecret`, `SubjectKey`, `DocumentKey`) besitzen weder `Default`, `Clone`, `Debug` noch `Serialize`.

---

## 5. Speicher

### 5.1 Segmentspeicher (`acta-seg`) [E]

Der Speicher ist ein **append-only Segmentspeicher**, kein allgemeiner LSM-Baum. **Begründung:** Die Last ist „einmal schreiben, häufig lesen, gezielt nach Betroffenem löschen“; Aktualisierungen in der Fläche kommen nicht vor. Segmente lassen sich als Ganzes versiegeln, gezielt ersetzen oder entfernen.

Zwei Siegelmodi:

| Modus | Schlüssel | Einsatz | Löschwirkung |
| --- | --- | --- | --- |
| `RecordSealed` | DEK je Dokument (4.2) | Ebene P: Datensätze | Schlüsselvernichtung macht Datensätze sofort unlesbar |
| `SegmentSealed` | `SegmentKey` je Segment | Ebene K (Packs) und Ebene D (Indexsegmente) | Ersatzsegment schreiben, alten `SegmentKey` vernichten, Altdatei bereinigen |

**Aufbau:** Journal (`wal`), Segmente (`seg-<n>`), Manifest. Alle Dateizugriffe laufen ausschließlich über `acta-fs`.

- **Journal-Rahmen:** `[magic u8][flags u8][len u32][hdr_crc32c u32][payload][chain_tag 32]`; `chain_tag` ist eine HMAC-SHA256-Kette über alle Rahmen. Nutzlasten der Klasse P liegen ausnahmslos als Umschlag vor; das Journal enthält nie Klartext.
- **Segmentkopf:** Magic `ACTASEG\0`, `format_version`, `partition` (K, P, D), `seal_mode`, `file_id` (16 B Zufall), `created_seq`, Kopf-MAC. Der Fußabschnitt enthält bei P- und D-Segmenten das `source_set` (Roaring-Bitmap der `DocOrd`).
- **Manifest:** atomare Batches `AddSegment`, `RemoveSegment`, `SeqHwm`, `Partition`. Ein fehlgeschlagener Manifest-Schreibvorgang vergiftet den Speicher sofort (Fail-Closed).
- **Partitionen:** K, P und D werden niemals in derselben Datei gemischt (Invariante I-PART).
- **Ordnung der Datensatzschlüssel:** `[kind u8][tenant_tag 16][partition u8][cluster 16][doc_ref 16][!seq u64][flags u8]`. `cluster` ist der `SubjectTag` des Hauptbetroffenen (bei Dokumenten ohne Betroffenen die `DocRef`). **Wirkung:** Alle Daten einer Person liegen zusammen; bei Löschung genügt häufig das Entfernen ganzer Segmente (Datei-Drop) statt eines Umschreibens.
- **Handkodierte Formate:** Alle Plattenformate werden von Hand mit geprüften Feldzugriffen kodiert (`from_be_bytes`, `get(..)`); `serde` und `bincode` sind für Plattenformate verboten. **Begründung:** Ein Plattenformat ist eine über Jahre stabile Schnittstelle und darf nicht aus Strukturlayouts folgen. Jedes Format hat Golden-Tests und ein Fuzz-Target.

**Entscheidungsregel Eigenbau [E]:** Eine bestehende eingebettete Speicher-Engine wird nur dann verwendet, wenn sie (1) gezieltes Entfernen und Überschreiben der Dateien eines Betroffenen ohne globale Verdichtung erlaubt, (2) deterministisch unter `FaultVfs` prüfbar ist, (3) Werte vor dem Schreiben im Journal bereits als Umschlag entgegennimmt und (4) keine FFI-Fläche mitbringt. Das Ergebnis dieser Prüfung wird vor Beginn der Speicherimplementierung als Messbericht dokumentiert; fällt sie negativ aus, gilt diese Spezifikation unverändert (Eigenbau, Budget 7 000 LOC).

### 5.2 Ordinale und Herkunft (Lineage) [E]

1. Jedes Dokument der Ebene P erhält beim Ingest eine `DocOrd` (u32, monoton je Mandant, nie wiederverwendet). Sie ist rein intern, verschwindet mit dem Dokumentschlüssel und erscheint nie in Ebene K, Berichten oder Protokollen.
2. Jede abgeleitete Struktur (Indexsegment, Cache-Eintrag, Embedding-Block) trägt `source_set: RoaringBitmap` der `DocOrd`, aus denen sie entstanden ist.
3. `ReadGate` führt `ErasedOrds: RoaringBitmap` und eine Epoche. `contains(ord)` ist O(1) ohne E/A.
4. Ein Segment ist betroffen genau dann, wenn `source_set ∩ ErasedOrds ≠ ∅`.
5. **Typzwang:** Der Typ `Derived<T>` kapselt Nutzdaten untrennbar mit ihrer `Lineage` (nicht leer). Indizes nehmen Daten nur über `put_derived(Derived<T>)` an. Eine Ableitung ohne Herkunft ist nicht konstruierbar (Z13).

---

## 6. Versionskontrolle der Wissensebene („Git für RAG“)

### 6.1 Objektmodell [E]

Alle Objekte sind unveränderlich und inhaltsadressiert: `ObjectId = SHA-256(Domänenpräfix ‖ kanonische Bytes)`.

| Objekt | Inhalt |
| --- | --- |
| **Blob** | Bytes eines Wissensdokuments oder einer Konfiguration. Präfix `acta/blob/v1`. Keine Personendaten (Z5) |
| **Tree** | Sortierte Einträge `{name, kind, id, meta}`; `kind ∈ {Blob, Tree, PersonalRef}`. Bei `PersonalRef` ist `id` das 16-Byte-Handle, **nie** ein Inhaltshash. Präfix `acta/tree/v1` |
| **Commit** | `{tree, parents[], tx_seq, tx_time, valid_from?, valid_to?, author: ActorRef, kind, note_tag}`. Präfix `acta/commit/v1` |
| **Ref** | Benannter Zeiger auf einen Commit: Branch (`main`, `szenario/<id>`) oder Tag (unveränderlich) |

`ActorRef` ist ein zufälliges Handle auf eine Identität in Ebene P (die Identität von Mitarbeitenden ist selbst personenbezogen und löschbar). `note_tag` ist eine strukturierte Kennung aus einer Whitelist; freier Text wird nicht in Commits gespeichert, sondern als Blob mit Validierung abgelegt.

**Wissensobjekte im Tree:** Dokumente (nach Chunking als Kind-Blobs oder Chunk-Tabelle), Aufbewahrungs- und Sichtbarkeitsmetadaten, Prompt-Vorlagen, Abrufkonfiguration, Modellbeschreibungen (`ModelFingerprint`), Schwellenwerte und Regeln. **Alles, was eine Entscheidung beeinflusst, ist versioniert.**

### 6.2 Globales Protokoll und Integrität

Jede Veränderung der Ebene K (neuer Commit, Ref-Bewegung, Tag, Takedown-Vermerk) wird als Eintrag in das **Ledger** (`acta-ledger`) geschrieben: `entry_hash = SHA-256(prev_hash ‖ kanonischer Eintrag)`. Alle `checkpoint_every` = 1 024 Einträge entsteht ein Ed25519-signierter Checkpoint. Damit lässt sich die Geschichte ohne Zugriff auf Inhalte prüfen und nicht unbemerkt umschreiben. Das Ledger enthält keine Personendaten (Z15).

### 6.3 Zeitachsen [E]

| Achse | Bedeutung | Abfrageparameter |
| --- | --- | --- |
| **Gültigkeitszeit** `valid` | Fachliche Geltung einer Vorschrift oder Regel: `[valid_from, valid_to)` | `valid_at: Date` |
| **Transaktionszeit** `tx` | Wann Acta den Stand erfasst hat; Commit-Folge | `as_of: CommitId \| Zeitpunkt` |
| **Löschzeit** `erase` | Wann Ebene P ein Ziel gesperrt hat; nur im Löschjournal | – |

Beispiel: „Welche Fassung der Satzung galt am 1.3. (`valid_at`) und was wusste das System am 15.3. (`as_of`)?“ Die Antwort ist deterministisch (Z7).

### 6.4 Operationen

| Operation | Bedeutung |
| --- | --- |
| `commit` | Neuer Stand aus Änderungen; Pflichtprüfungen: N-Validator (7.5), Klassendeklaration |
| `branch`, `tag` | Verzweigung für Szenarien, unveränderliche Markierung |
| `checkout(as_of, valid_at)` | Lesender Stand (Sicht), keine Kopie |
| `diff(a, b)` | Unterschied zweier Stände auf Dokument-, Chunk- und Metadatenebene |
| `merge(src, dst)` | Drei-Wege-Zusammenführung; Konflikte pro Pfad; Freigabe nach Vier-Augen-Prinzip konfigurierbar; `PersonalRef`-Einträge werden nach Handle verglichen |
| `retire(branch)` | Szenario-Verzweigung freigeben; nicht mehr erreichbare Objekte werden nach `branch_retention` (Standard 90 Tage, anpinnbar) bereinigt |

### 6.5 Sichtbarkeitsmengen und versionsbewusste Suche [E]

Jeder Wissens-Blob erhält eine `KOrd`. Je Commit gibt es eine **Sichtbarkeitsmenge** (Roaring-Bitmap der `KOrd` im Tree). Sie wird als Basismenge plus Deltas zum Elterncommit gespeichert (Schnappschuss alle 256 Commits). Die Suche arbeitet auf **globalen, unveränderlichen Indexsegmenten** und filtert die Kandidaten mit der Sichtbarkeitsmenge des gewählten Standes. So kostet ein Branch oder Zeitpunkt keinen eigenen Index, und eine Suche „wie am Stichtag“ ist exakt (Z7).

### 6.6 Zeitreise über Personendaten (Ebene P) [E]

Ebene P hat **keine** allgemeine Zeitreise. Sie führt je Dokument eine kurze Versionskette ausschließlich für Berichtigung (Art. 16) und Aufbewahrungsfristen, begrenzt durch `history_horizon` (Standard `Personal`: 30 Tage; `Besonders`: 0). Abfragen mit `as_of` auf P prüfen **zuerst** das Gate; ein gelöschtes Ziel liefert `Erased`, nie Klartext (Z1). **Begründung:** Eine unbegrenzte Historie personenbezogener Inhalte widerspricht dem Zweck der Löschung; Nachvollziehbarkeit gelingt über die Begründungskette in K (7).

### 6.7 Garbage Collection

Erreichbarkeit: Objekte, die von keinem Ref und keinem aufbewahrten Entscheidungsprotokoll erreichbar sind, werden nach `branch_retention` entfernt. Pack-Segmente werden dabei ohne unerreichbare Blobs neu geschrieben (neuer `SegmentKey`, alter wird vernichtet). Die Hash-Kette bleibt unberührt, weil sie nur Commits und Refs enthält.

---

## 7. Entscheidungsprotokoll, Szenarien und Entkopplung

### 7.1 Entscheidungsprotokoll [E]

Zu jeder Antwort und jedem Szenariolauf entsteht ein Eintrag der Klasse `Beleg` in K:

```rust
pub struct DecisionRecord {
    pub id: [u8; 16],               // zufällig
    pub kind: DecisionKind,         // Query | ScenarioRun | Review | Release
    pub commit: ObjectId,           // Wissensstand
    pub valid_at: Option<Date>,
    pub prompt: ObjectId,           // Prompt-Version (Blob in K)
    pub retrieval_cfg: ObjectId,    // Abrufkonfiguration (Blob in K)
    pub model: ModelFingerprint,    // Anbieter, Name, Digest, Parameter, Seed
    pub evidence: Vec<EvidenceRef>, // siehe unten
    pub answer: AnswerRef,
    pub actor: ActorRef,            // Handle, löschbar
    pub scenario: Option<[u8; 16]>,
    pub prev_hash: [u8; 32],        // Kette im Ledger
}
pub enum EvidenceRef { Knowledge { blob: ObjectId, chunk: u32 }, Personal(PersonalRef) }
pub enum AnswerRef  { Open(ObjectId), Sealed(PersonalRef) }
```

- Antworten, die aus rein öffentlichem Wissen entstehen und den N-Validator bestehen, liegen als `Open` in K.
- Antworten, die Personendaten benutzen, liegen als `Sealed` in Ebene P. Ihr Dokumentschlüssel wird über alle beteiligten Betroffenen abgeleitet (4.2). Wird eine beteiligte Person gelöscht, ist die Antwort unlesbar.
- Nach einer Löschung bleibt die Begründungskette **vollständig sichtbar**: Wissensstand, Prompt, Modell, Wissensbelege. Eingaben mit Personenbezug lösen zu `Erased` auf.

### 7.2 Nachvollziehen einer alten Entscheidung

`explain(decision_id)` liefert: Stand der Wissensebene (Commit, `valid_at`), Prompt, Modell und Parameter, jeden Beleg (soweit nicht gelöscht und nicht durch Zugriffsrechte verborgen), die gespeicherte Antwort. Optional `replay(decision_id)`: erneuter Lauf mit denselben Parametern. **Grenze [A]:** Byte-identische Wiederholung ist nur möglich, wenn das Inferenzsystem deterministische Dekodierung unterstützt; die **gespeicherte Antwort** ist der Beleg, der Wiederholungslauf ein Vergleich. Der Bericht weist aus, ob der Wiederholungslauf identisch war.

### 7.3 Szenarien („Wenn-Dann“) [E]

Ein **Szenario** besteht aus: Basis-Commit, Szenario-Branch mit Änderungen (z. B. geänderte Vorschrift, anderer Schwellenwert, anderer Prompt, anderes Modell), einem **Fragenset** (Blob in K) und den Ergebnissen. Ablauf: (1) Branch aus Basis-Commit, (2) Änderungen als Commits, (3) Fragenset gegen Basis und Szenario laufen lassen, (4) Vergleich: Unterschiede in Belegen und Antworten, (5) bei Bedarf `merge` oder `retire`.

Z9 gilt: Ein Szenario schreibt nie in `main` und nie in Ebene P. Personendaten dürfen in einem Szenario nur gelesen werden, wenn Zugriffsrecht und Gate es erlauben; Ergebnisse mit Personenbezug werden `Sealed`. Szenarien verändern keine Zugriffsrechte.

### 7.4 Akteure und Mitarbeitende

Nutzer sind selbst Betroffene. Protokolle enthalten daher `ActorRef`-Handles; die Zuordnung zu Klarnamen liegt in Ebene P unter dem Schlüssel der Person. Eine Löschung oder Auskunft über Mitarbeitende folgt denselben Mechanismen wie jede andere Betroffenenanfrage. **[A]** Personalvertretung und interne Regeln zur Auswertung von Nutzungsdaten liegen beim Betreiber.

### 7.5 N-Validator (Schutz der Ebene K) [E]

Vor jedem Schreiben in K prüft der Validator: (1) **Schema-Whitelist** für strukturierte Felder, (2) **Mustererkennung** (Namen, Anschriften, Kennnummern, E-Mail, Telefon, Kontonummern, Aktenzeichen mit Personenbezug) über `acta-pii`, (3) **Entropie- und Handle-Prüfung** für Kennungen, (4) bei Aggregaten **k-Anonymität** mit `K_MIN` ≥ 5. Ein Treffer über der Schwelle blockiert das Schreiben (Fail-Closed). **Grenze [A]:** Automatische Erkennung ist nicht vollständig; deshalb Standardmodus `Whole` (2.3), Vier-Augen-Freigabe für `Open` und `Split` und Takedown (7.6).

### 7.6 Takedown in Ebene K

Wird in K irrtümlich Personenbezug festgestellt oder ordnet ein Gericht die Entfernung an: (1) Vier-Augen-Freigabe, kryptografisch signiert; (2) Pack-Segment ohne den Blob neu schreiben, `SegmentKey` vernichten; (3) im Ledger einen Vermerk `Redacted { object_id, grund_code }` anfügen. Die Objekt-ID und die Kette bleiben erhalten, der Inhalt nicht. **Restrisiko [A]:** Die Objekt-ID ist ein Hash über den entfernten Inhalt; bei niedriger Entropie ist ein Wörterbuchtest theoretisch möglich. Der Bericht weist das aus.

---

## 8. Löschen

### 8.1 Ziele

| Ziel | Auslöser | Vernichteter Schlüssel | Wirkung |
| --- | --- | --- | --- |
| `Subject` | Art. 17, Widerruf, Zweckfortfall | `SubjectKey` | alle Dokumente der Person und aller Mitbetroffenen (Z3) |
| `Document` | Einzellöschung | `DocumentKey` | ein Dokument in allen Versionen |
| `Collection` | Fachliche Bereinigung | alle `DocumentKey` der Sammlung | Sammlung |
| `Tenant` | Off-Boarding | alle Schlüssel des Mandanten, `SegmentKey`s | gesamter Mandant |
| `Expired` | Fristablauf (Art. 5 Abs. 1 lit. e) | zuständiger Schlüssel | automatisch durch Sweeper |
| `Restrict` | Art. 18 | keiner | Sperre im Gate (`Restricted`), Daten bleiben |
| `Supersede` | Art. 16 | keiner | alte Version bereinigt, Schlüssel bleibt |
| `Actor` | Löschung von Mitarbeitendendaten | `SubjectKey` der Person | Handles in K lösen zu `Erased` auf |

### 8.2 Invarianten [E]

- **I1** Die Sperre ist durabel, bevor etwas Physisches geschieht.
- **I2** Schlüsselvernichtung erst, nachdem alle Schichten bestätigt haben, dass flüchtige Kopien (RAM, Caches, laufende Sitzungen) geleert sind.
- **I3** Jeder Zwischenzustand ist nach einem Absturz wiederaufnehmbar; es gibt keinen Zustand `Failed`.
- **I4** Ein Nachweis entsteht nur aus einer Messung mit Positivkontrolle.
- **I5** Eine gesetzte Sperre wird nie aufgehoben (Ausnahme: dokumentierte Administratorentscheidung mit Vier-Augen-Prinzip nach Fehlauslösung, vor Schlüsselvernichtung).
- **I6** Die Versionsgeschichte in K wird durch keinen Schritt verändert (Z6).

### 8.3 Zustandsautomat

```
Requested ─► Gated ─► KeyDestroyed ─► Purging ─► Verifying ─► Completed
    │
    ├─► Refused           (Aufbewahrungspflicht oder Rechtsvorbehalt vor Sperre)
    ├─► Restricted        (Art. 18)
    └─► Pending(Grund) / Escalated(Grund)   (automatische Wiederaufnahme)
```

### 8.4 Schritte

| Schritt | Inhalt |
| --- | --- |
| **E0** | Validierung und Autorisierung; Prüfung auf Aufbewahrungspflicht, Rechtsvorbehalt (`legal_hold`), Archivierungspflicht (Hook zu vorhandenem Archivsystem); Auflösung Betroffener → Dokumente |
| **E1** | Absicht protokollieren: verschlüsselte Scope-Datei und Journal-Eintrag `Requested` (`fsync`) |
| **E2** | **Sperre setzen:** `ErasedOrds` erweitern, Epoche erhöhen, Gate-Log anfügen (`fsync`, Gruppen-Commit). Ab hier liefert kein Pfad Daten (Z1) |
| **E2a** | **Quittung aller Schichten** (`on_gate`): Indizes, Caches, Prompt-Caches, laufende Sitzungen leeren; fehlt eine Quittung, bleibt der Vorgang in `Pending(LayerNotAcked)` |
| **E2b** | **Nadeln entnehmen:** stratifizierte Klartextfenster (max. `NEEDLE_MAX` = 4 096) im `mlock`-Speicher für die spätere Messung |
| **E3** | **Schlüsselvernichtung:** Grabstein im Tresor, `fsync`, Direct-Read-Verifikation, Journal `KeyDestroyed`. Rückkehr an den Aufrufer mit `Gated`/`KeyDestroyed` |
| **E4** | Ledger-Quittung mit keyed Commitments (keine Roh-Kennungen) |
| **E5** | Physische Bereinigung (8.5) |
| **E6** | Ableitungen neu aufbauen (Rebuild-Segmente, 9.3) |
| **E7** | Messung MK1 bis MK3 mit Positivkontrolle (8.6) |
| **E8** | Bericht erzeugen und signieren; Scope-Datei überschreiben und löschen; Journal `Completed` |
| **E9** | Benachrichtigung der Empfänger (Art. 19); Gate-Verdichtung nach `gate_compaction_delay` |

**Regeln:** E3 darf erst nach durabler E2 und vollständiger E2a ausgeführt werden (R-E2-E3). Die Dauer bis zum Abschluss von E2 ist unabhängig von der Datenmenge (R-O1). Der synchrone Aufruf kehrt nach spätestens `erase_return_timeout` = 500 ms mit `Gated` zurück (oder `Pending(Timeout)`).

### 8.5 Physische Bereinigung (E5), Reihenfolge

a) Leser-Snapshots widerrufen, Kopien lösen · b) Textindex-Segmente ohne Ziel-Ordinale neu schreiben · c) Speicher- und Journalpuffer entleeren, Journal bis zur Sperrfolge recyceln (überschreiben, dann entfernen) · d) Segmente: Datei-Drop bei Betroffenen-Isolation, sonst gefilterter Rewrite · e) Vektorindex: Vektoren nullen, Segmente neu bauen · f) Tresor verdichten · g) Caches invalidieren und nullen · h) Kopienregister aktualisieren, Kopien bereinigen. Reservierung `min_free_bytes` vor Rewrites.

### 8.6 Messung [E]

| Klasse | Prüfbereich | Methode |
| --- | --- | --- |
| **MK1** | Logische Strukturen | Manifest, Indizes, Bitmaps, Kataloge auf Abwesenheit der Ziel-IDs |
| **MK2** | Rohbyte-Scan | Musterscan ohne Formatparser über alle Datendateien, Journalsegmente, Temporärdateien und Heap |
| **MK3** | Schlüsselprüfung | Byteweise Prüfung des Tresor-Eintrags am Datenträger (`VerifiedVia::Direct`) |

**Typestate (Z11):** `Scanner<Unarmed>` → `arm_with_synthetic_positive_control()` → `Scanner<Armed>` → `scan_layer()` → `ScanResult` → `into_witness()` nur bei exakt 0 Treffern.

### 8.7 Berichtsklassen

| Klasse | Aussage |
| --- | --- |
| **G** | Alle Lesepfade gesperrt |
| **K** | Schlüsselmaterial vernichtet (MK3) |
| **P** | Chiffrate und Klartexte physisch entfernt (MK2) |
| **D** | Ableitungen bereinigt (Zeugen der Indexschichten) |
| **C** | Registrierte Kopien bereinigt (Kopienregister) |

Die Klasse **E** („vollständig gelöscht“) gilt genau dann als erreicht, wenn G, K, P, D und C `reached` sind. Der Bericht (JCS, RFC 8785, siehe Anhang B) enthält Klassen, Zeugen, Zeitstempel, Mitbetroffenenzahl und die Annahmen A-FLASH, A-RAM, A-ANCHOR, A-CRYPTO, A-EXT.

### 8.8 Restore-Schutz [E]

Der Tresor trägt seine Generation `g`. Der Anker kennt die gültige Mindestgeneration. Ein Tresor einer vernichteten Generation ist nicht lesbar. Sicherungen des Tresors sind höchstens `T_BACKUP` = 72 h gültig; ältere werden abgewiesen. Nach jedem Einspielen läuft ein **Erase-Replay**: Das Ledger enthält die keyed Commitments aller Löschungen; jede im Ledger vermerkte Löschung, deren Schlüssel im wiederhergestellten Tresor noch aktiv ist, wird erneut vollzogen, bevor das System Anfragen annimmt. **Datensicherungen der Segmente** (ohne Tresor) dürfen beliebig lange aufbewahrt werden: Ohne vernichtete Schlüssel sind sie unlesbar. **[A] A-FLASH:** Medienreste auf Flash und Dateisystem-Journalen sind durch den Schlüsselentzug abgedeckt, nicht durch Überschreiben.

---

## 9. Abgeleitete Daten und Abruf (RAG)

### 9.1 Ingest

1. **Annahme** (nicht vertrauenswürdig): Parser mit Ressourcenlimits (Größe, Tiefe, Zeit, Speicher); Fuzzing für jeden Parser; Isolation in einen Kindprozess ist über die Parser-Schnittstelle (`parse(bytes, limits) -> Result<Chunks>`) vorbereitet.
2. **Klassifikation:** Datenklasse und Modus (2.3) werden vom Mandanten per Richtlinie oder Handentscheidung vorgegeben; `acta-pii` liefert Hinweise.
3. **Chunking:** strukturbewusst (Überschriften, Absätze, Tabellen), Chunk-Größe und Überlappung konfigurierbar, jeder Chunk trägt Quellverweis (Dokument, Seite, Abschnitt).
4. **Ablage:** Wissen → Blob/Tree-Commit in K; Personendaten → versiegelter Datensatz in P mit `DocOrd`; Ableitungen → Indexsegmente.
5. **Idempotenz:** erneuter Ingest desselben Inhalts erzeugt in K keine neuen Objekte (Inhaltsadressierung).

### 9.2 Indizes

- **Text (`acta-text`):** segmentierter BM25-Index, Tokenisierung und Normalisierung für Deutsch und Englisch (Zerlegung von Komposita, Flexion), Phrasen- und Feldsuche.
- **Vektor (`acta-vec`):** zunächst exakte Flat-Suche über unveränderliche Segmente; ein ANN-Backend wird nur eingesetzt, wenn die Flat-Suche die Latenzziele (9.6) auf dem Referenzkorpus nicht erreicht. Der Embedder ist ein austauschbarer Vertrag (`acta-infer`); Embeddings tragen den `ModelFingerprint`; ein Modellwechsel erzeugt neue Segmente, alte bleiben bis zur Ablösung lesbar.
- Eigenbau eines Graph-Index gehört nicht zu Version 1.

### 9.3 Versiegelung der Ableitungen [E]

Jedes persistierte Indexsegment, das aus Ebene-P-Quellen entsteht, ist `SegmentSealed` unter einem eigenen `SegmentKey`. Segmente, die ausschließlich aus `Wissen` bestehen, sind ebenfalls versiegelt (Vertraulichkeit auf dem Medium), unterliegen aber keiner Löschlogik. **Begründung:** Z2 verlangt AEAD für jeden persistierten Klartext von Personen. Terme und Embeddings aus Personendaten sind selbst personenbezogen. Außerdem werden Sicherungen von Indexdateien mit der Vernichtung des alten `SegmentKey` automatisch unlesbar.

`Rebuild` ist die **Löschstrategie** (Standard): sofortige Maske über `ErasedOrds`, Ersatzsegment ohne gelöschte Ordinale unter neuem `SegmentKey`, atomarer Austausch, danach alten `SegmentKey` vernichten und Altdatei bereinigen. Fremdbetroffene bleiben verfügbar. Die Strategie `Sealed` (Segment sofort unlesbar, Rest neu aus Ebene P) ist für statische Archive zulässig, höchstens `segment_max_vectors` = 50 000.

**Preis:** Segmente werden zum Lesen entschlüsselt und im Arbeitsspeicher gehalten (kein Klartext-`mmap`); Speicher in `mlock`-Seiten. **Revisionstrigger:** Ladezeit oder Speicherbedarf verfehlen die Betriebsziele; dann Entschlüsselung blockweise.

### 9.4 Abrufpipeline [E]

```
Frage ─► Normalisierung ─► (optional) Umformulierung
   ─► Kandidatenmenge = Sichtbarkeit(commit/valid_at) ∩ Zugriff(Nutzer) ∩ ¬ErasedOrds ∩ Aufbewahrung
   ─► Textsuche (BM25)  ┐
   ─► Vektorsuche       ├─► Fusion (Reciprocal Rank Fusion)
                         ┘
   ─► Reranking (Cross-Encoder, optional)
   ─► Kontextaufbau (Budget, Deduplizierung, Quellverweise)
   ─► Generierung mit Belegpflicht
   ─► Belegprüfung (jede Aussage → Beleg) ─► Antwort oder Enthaltung
   ─► Entscheidungsprotokoll
```

- **Kandidatenmenge vor der Suche (Z16):** Zugriff, Sichtbarkeit und Sperre wirken als Bitmap-Schnitt auf die Kandidaten; es gibt keine nachträgliche Filterung der Ausgabe.
- **Prompt-Injection:** Abgerufene Inhalte sind Daten, nie Anweisungen. Systemanweisungen und Belege werden strukturell getrennt; Ausgaben werden gegen die Belege geprüft; das Modell führt keine Werkzeuge aus.
- **Enthaltung:** Liegt kein hinreichender Beleg vor (Schwelle aus der Abrufkonfiguration), antwortet das System „nicht belegt“ und nennt, welche Quellen geprüft wurden.

### 9.5 Belegprüfung [E]

Die Antwort wird in Aussagen zerlegt; jede Aussage verweist auf mindestens einen Beleg-Chunk. Eine unabhängige Prüfung (Regelprüfung auf Zitat-Übereinstimmung, optional Entailment-Modell) bestätigt Stützung. Aussagen ohne Stützung werden entfernt oder markiert; ist danach nichts übrig, folgt die Enthaltung (Z17). Prüfmodelle sind austauschbare Verträge und werden im Entscheidungsprotokoll mit Fingerprint geführt.

### 9.6 Qualitätsziele und Bewertung [Q]

Acta liefert ein **Bewertungswerkzeug** (`acta-rag::eval`), das auf einem mandantenspezifischen Prüfkorpus (Fragen, erwartete Quellen, erwartete Enthaltungen) Messwerte erzeugt. Startwerte für die Freigabe:

| Messgröße | Startwert |
| --- | --- |
| Recall@20 der Kandidaten (erwartete Quelle unter den Kandidaten) | ≥ 0,95 |
| Zitiergenauigkeit (zitierte Aussage durch Beleg gestützt) | ≥ 0,98 |
| Korrekte Enthaltung bei fehlendem Beleg | ≥ 0,90 |
| Latenz Abruf bis erstes Token, 95. Perzentil | wird je Referenzkorpus festgelegt und gemessen |
| Bestätigung der Sperre bei Löschung, 99. Perzentil | ≤ 500 ms (R-RET) |

Die Zahlen sind keine Zusage, sondern die **Mindestanforderung für die Freigabe** einer Konfiguration; der Bericht nennt Korpus, Messdatum und Konfigurationsstand (als Commit in K). Eine Konfigurationsänderung ohne neuen Messbericht kann nicht in `main` zusammengeführt werden, wenn die Richtlinie „Bewertung vor Freigabe“ des Mandanten aktiv ist.

---

## 10. Zugriff, Mandanten, Organisation

### 10.1 Mandanten [E]

Jeder Mandant hat eigene Tags, eigene Schlüssel im Tresor, eigene Partitionen und Ordinalräume. Es gibt keine mandantenübergreifende Suche. Off-Boarding ist ein Löschziel (8.1).

### 10.2 Zugriffsmodell

- **Rollen** (z. B. Sachbearbeitung, Fachadministration, Datenschutz, Revision, Betrieb) und **Attribute** (Organisationseinheit, Verschlusssachen-Stufe, Sammlung, Projekt).
- **Schutzstufen** je Dokument (konfigurierbar, z. B. offen, intern, vertraulich, VS-NfD). Acta setzt Stufen technisch durch; eine Zulassung für Verschlusssachen ist nicht Gegenstand dieser Spezifikation.
- **Auswertung:** Richtlinien werden je Nutzer zu einer Bitmap-Menge erlaubter Dokumente aufgelöst und mit Sichtbarkeit und Sperre geschnitten (9.4).
- **Trennung der Pflichten:** Betrieb (Administratoren) hat keinen Zugriff auf Klartext von Ebene P; Schlüssel liegen im Anker, den der Betreiber getrennt verwaltet. Datenschutzfunktion und Revision sind eigene Rollen.
- **Vier-Augen-Prinzip** (konfigurierbar) für: `merge` nach `main`, Takedown, `Open`/`Split`-Freigabe, Aufhebung einer Fehlauslösung, Änderung von Richtlinien.

### 10.3 Verwaltung und Aufbewahrung

- **Sammlungen** mit Metadatenschema (Pflichtfelder, Typen), Verantwortlichen und Aufbewahrungsklasse.
- **Aufbewahrungsklassen** mit Frist, Auslöser (Erstellung, Abschluss, Ereignis), Folge (Löschung, Anbietung an Archiv, Prüfung) und `legal_hold`.
- **Sweeper** erkennt Fristabläufe, plant `Expired`-Löschungen und meldet Konflikte mit Aufbewahrungspflichten.
- **Löschkonzept** als Konfiguration in K: je Datenklasse Frist, Verfahren, Verantwortliche. Änderungen sind versioniert und nachvollziehbar.

---

## 11. Inferenz und Egress

### 11.1 Verträge [E]

`Embedder`, `Reranker` und `Generator` sind Traits in `acta-infer`. Jede Implementierung liefert einen `ModelFingerprint` (Anbieter, Modellname, Digest oder Version, relevante Parameter). Der Fingerprint ist Teil jedes Entscheidungsprotokolls und jeder Embedding-Segmentbeschreibung.

### 11.2 Betriebsformen

| Form | Beschreibung | Standard |
| --- | --- | --- |
| Lokaler Dienst | Inferenzserver im selben Netz (z. B. OpenAI-kompatible Schnittstelle) über `acta-infer-http` | ja |
| Prozessintern | Embedding/Reranking über eine Bibliothek, optional später | nein |
| Freigegebener Dritter | Externer Dienst | nein |

### 11.3 Egress-Kontrolle (Z18)

Ausgehende Verbindungen sind standardmäßig gesperrt. Eine Freigabe nennt Ziel, Zweck, zulässige Datenklassen und Mandanten und ist selbst ein versionierter, vier-Augen-freigegebener Eintrag in K. Ein Egress-Wächter implementiert `Generator`/`Embedder` als **Dekorator**, prüft die Freigabe, blockiert Inhalte der Ebene P (außer freigegeben) und protokolliert jeden Aufruf (ohne Inhalt). Der Bericht weist Zone Z4 (extern) als Annahme A-EXT aus.

---

## 12. Schnittstellen und Betrieb

### 12.1 API [E]

HTTP/JSON mit Streaming (SSE) für Antworten. Authentifizierung über OIDC oder mTLS; Autorisierung durch `acta-access`. Ressourcen: Mandanten, Sammlungen, Dokumente, Commits, Branches, Szenarien, Fragen und Antworten, Entscheidungen (`explain`, `replay`), Betroffenenanfragen (Auskunft, Berichtigung, Einschränkung, Löschung), Berichte, Verwaltung. Eine Versionskennung in der URL; die Schnittstelle ist Teil der Stabilitätszusage ab Version 1.

### 12.2 Betroffenenanfragen

Auskunft (Art. 15) liefert alle zum `SubjectTag` auffindbaren Datensätze unter Beachtung der Rechte Dritter (Mitbetroffene). Berichtigung (Art. 16) als `Supersede`. Einschränkung (Art. 18) als `Restrict`. Löschung (Art. 17) als `Subject`. Fristüberwachung mit `legal_max` = 30 Tage und Eskalation. Alle Anfragen haben Ticket-Handles; das Ticket enthält keine Klarnamen im Ledger.

### 12.3 Selbsttest beim Start (nicht abschaltbar)

Prüft: Konfiguration vollständig (`k_idx` vorhanden), Anker erreichbar, Tresor lesbar und Generation gültig, Erase-Replay ohne offene Punkte, Journal und Manifest konsistent, Positivkontrolle des Scanners, synthetischer Ende-zu-Ende-Lösch-Test auf einem Prüf-Mandanten, Zeit- und Zufallsquelle funktionsfähig. Schlägt ein Punkt fehl, startet der Knoten nicht (Fail-Closed).

### 12.4 Beobachtbarkeit

Metriken (Prometheus-Format) und strukturierte Protokolle **ohne** Personendaten und ohne Roh-Kennungen (Z15). Gesundheitsendpunkte. Alarme bei `Escalated`, `Poisoned`, Ankerausfall, Rotationsverzug, fehlenden Schichtquittungen.

### 12.5 Betrieb, Sicherung, Aktualisierung

- **Sicherung:** Segmente und Ledger beliebig lange; Tresor höchstens `T_BACKUP`; Wiederherstellung nur mit Erase-Replay (8.8). Eine Anleitung und ein Werkzeug (`acta-cli backup`, `restore`) gehören zum Lieferumfang.
- **Aktualisierung:** Jedes Plattenformat trägt eine Versionsnummer. Neuere Formate werden von älterer Software abgelehnt. Migrationen laufen offline mit Vorab-Sicherung und Prüfsummenvergleich.
- **Lieferkette:** reproduzierbare Builds, `cargo vendor` für Offline-Bau, SBOM (CycloneDX), signierte Releases, `cargo deny` und `cargo audit` in der CI.

---

## 13. Nebenläufigkeit, Fehler, Konfiguration

### 13.1 Sperrenordnung

Sperren dürfen nur in aufsteigender Rangfolge genommen werden:

| Rang | Sperre | Haltezeit |
| --- | --- | --- |
| 1 | `CommitBarrier` (Schreiber gemeinsam, Gate-Setzer exklusiv) | nur für `epoch += 1` und Bitmap-Einfügen; nie über E/A |
| 2 | `PurgeBarrier` (Bereinigung exklusiv, Verdichtung gemeinsam) | über eine Bereinigungsphase |
| 3 | `ErasureFence` | ticketbasiert |
| 4 | `RefLock` (Ref-Bewegung in K) | kurz |
| 5 | `SnapshotRegistry` | O(1), nur Speicher |
| 6 | `VaultShardLock` | kurz, keine Blockierung durch Netzwerk |
| 7 | `ReadGateLock` | `RwLock`, Leser ohne E/A |

Die Rangfolge wird durch `OrderedMutex<const RANK: u8>` (Debug-Assertion) und Loom-Modelle geprüft.

### 13.2 Fehlertaxonomie und Vergiftung

- **Transient** (Zeitüberschreitung, Ressourcenmangel): `Pending(Grund)` mit Backoff 1 s, 2 s, 4 s … höchstens 15 min; Sperre bleibt.
- **Integrität und Krypto** (MAC-Fehler, ungültige Prüfsumme, Ableitungsfehler): **Vergiftung**. Der Speicher blockiert Schreibzugriffe und sperrt Lesezugriffe auf Ebene P; Aufhebung nur durch Administratoreingriff und Wiederherstellung.
- **SLA-Überschreitung:** Dauer einer Löschung über `escalate_after` → `Escalated(SlaExceeded)` mit Alarm.

### 13.3 Standardwerte

| Parameter | Wert | Bedeutung |
| --- | --- | --- |
| `erase_return_timeout` | 500 ms | maximale Blockierzeit bis `Gated` |
| `erase_sla` | 24 h | Ziel bis `Completed` |
| `escalate_after` | 72 h | Eskalation |
| `legal_max` | 30 Tage | Frist nach Art. 12 Abs. 3 DSGVO |
| `batch_window` | 60 s | Bündelung nachgelagerter Bereinigung |
| `min_free_bytes` | 1 GiB | Reserve vor Rewrites |
| `T_ROT` / `N_SHRED` | 24 h / 10 000 | Wurzelrotation |
| `T_BACKUP` | 72 h | maximales Alter einer Tresor-Sicherung |
| `K_MIN` | 5 | k-Anonymität für Aggregate in K |
| `NEEDLE_MAX` | 4 096 | Nadeln für die Messung |
| `history_horizon` | `Personal` 30 Tage, `Besonders` 0 | Versionskette in P |
| `checkpoint_every` | 1 024 | Ledger-Einträge je signiertem Checkpoint |
| `branch_retention` | 90 Tage | Aufbewahrung nicht angepinnter Szenario-Branches |
| `segment_max_vectors` | 50 000 | Obergrenze für `Sealed`-Strategie |
| `max_subjects_per_doc` | 64 | Betroffene je Dokument |
| `gate_compaction_delay` | 90 Tage | Vernichtung rotierter Commitment-Schlüssel |
| `boot_selftest` | `true` | nicht abschaltbar |
| `k_idx` | **kein Standardwert** | Start ohne Angabe verweigert |

---

## 14. Entscheidungsregister

| ID | Entscheidung | Begründung | Preis / Kompromiss |
| --- | --- | --- | --- |
| **ADR-01** | Vier Ebenen K, P, D, E; PersonalRef als zufälliges Handle | Versionierung und Löschung konstruktiv entkoppeln (Z5, Z6) | Dokumente mit Personenbezug sind in der Wissenssuche nur über P erreichbar |
| **ADR-02** | Standardmodus `Whole` bei erkanntem Personenbezug | Automatische Erkennung ist unvollständig; sicherer Standard | Weniger Wissen in der Versionsgeschichte |
| **ADR-03** | Zeitreise nur über K, kurze Versionskette in P | Nachvollziehbarkeit braucht Entscheidungsgrundlagen, nicht dauerhafte Personendaten | Keine historische Rekonstruktion von Personendaten |
| **ADR-04** | Sichtbarkeitsmengen (Roaring) je Commit statt Index je Stand | Branches und Stichtage ohne Indexkopien | Rechenaufwand bei jedem Filter; Messung nötig |
| **ADR-05** | XChaCha20-Poly1305, SHA-2-Familie, Ed25519; Algorithmus-Byte | Kleiner Primitivsatz, kleiner Prüfer, sichere Zufallsnonce | Kein FIPS-Algorithmus; AES-GCM nachrüstbar |
| **ADR-06** | DEK = HKDF über Dokument- und Betroffenenschlüssel | Mehrbetroffenen-Löschung ohne Sonderlogik, kein Schlüssel-Wrapping je Datensatz | 92 Byte je Dokument-/Personenschlüssel im Tresor |
| **ADR-07** | Wurzelrotation mit Generationen statt Maskierung je Slot | Ein Mechanismus, einfacher zu prüfen; macht alte Tresor-Kopien unlesbar | Anker muss Vernichtung innerhalb deklarierter Frist leisten |
| **ADR-08** | Append-only Segmentspeicher statt LSM | Lastprofil, gezielte Löschung, einfachere Prüfung | Kein allgemeiner Schlüssel-Wert-Speicher; Eigenbau nach Prüfregel (5.1) |
| **ADR-09** | Ableitungen aus P sind at rest versiegelt | Z2 verlangt AEAD für jeden persistierten Klartext von Personen | Kein Klartext-`mmap`, höherer Speicherbedarf, längerer Kaltstart |
| **ADR-10** | Kern synchron, async nur am Rand | Deterministische Fehlerinjektion, Loom, keine Abbruchsicherheitsnachweise | Thread-Pool-Brücke |
| **ADR-11** | `unsafe` nur in `acta-sys`; keine SIMD-Handoptimierung in Version 1 | Kleine auditierbare Fläche; Messung vor Optimierung | Geringere Spitzenleistung der Vektorsuche |
| **ADR-12** | Flat-Vektorsuche zuerst, ANN nur nach Messung | Löschung per Segment-Rebuild braucht nur Bauen und Suchen | Mehr Rechenzeit bei sehr großen Beständen |
| **ADR-13** | Level-Modell mit `rings.toml` | Strikte Schichtung prüfbar, auch bei Geschwister-Abhängigkeiten | Zusätzliche Konfigurationsdatei |
| **ADR-14** | Keine Sammel-Crates; Fehler je Crate | Verhindert wachsende Hubs und Fremdwissen | Mehr `From`-Code in Aggregatoren |
| **ADR-15** | Deterministische Tags aus `k_idx` für Mandant, Sammlung und Betroffene; zufällige `DocRef` und Handles | Betroffene ohne Zuordnungstabelle auffindbar; Dokumente nicht verknüpfbar | `k_idx`-Verlust ermöglicht Wörterbuchtests auf externe Kennungen; `k_idx` liegt im Anker |
| **ADR-16** | Handkodierte Plattenformate | Stabilität, Prüfbarkeit, kein `unsafe` | Mehr Code |
| **ADR-17** | Egress-Standard gesperrt, Freigabe als versionierter Eintrag | Souveränität, Z18 | Aufwand bei der Einrichtung externer Dienste |
| **ADR-18** | Nutzer sind Betroffene: `ActorRef` in Protokollen | Auskunfts- und Löschrechte auch für Mitarbeitende | Auswertungen nach Person erfordern Auflösung über P |
| **ADR-19** | Belegpflicht mit Enthaltung statt Antwort ohne Beleg | Präzision und Kontrolle | Mehr Enthaltungen bei dünner Wissensbasis |

---

## 15. Teststrategie

### 15.1 Prinzipien

Eine Zusage ohne automatisierten Test gilt als nicht erfüllt. Orakel sind **unabhängig vom System**: Testvektoren aus RFCs und Entwürfen, eine unabhängige Referenzimplementierung im Test, **Modelltests** der Löschzustandsmaschine (zustandsbasiertes Property-Testing mit Absturzinjektion an jedem `fsync`), Properties (Hin- und Rückumwandlung, Invarianten), Fuzzing aller Parser und Formate, Compile-Fail-Tests für Typzwang.

### 15.2 Matrix

| Zusage | Primäre Tests |
| --- | --- |
| Z1 | `T-GATE-1` (alle Lesepfade nach `Gated`: Suche, Antwort, `as_of`, Szenario, Cache, Export), `T-CONC-1` (Loom: Linearisierung des Gates) |
| Z2 | `T-CRY-1` (Bytes-Grep über alle Dateien mit bekannten Klartexten), `T-SEAL-1..4`, `T-DERIVED-SEALED` (Terme und Embeddings nicht im Klartext auf Platte) |
| Z3 | `T-MULTI-1` |
| Z4 | `T-CRASH-*`: Absturz an jedem Durabilitätspunkt von E1 bis E8, Rotation, Rewrite-Austausch |
| Z5 | `T-SEP-1` (Wissensebene enthält keine Muster aus dem Personenkorpus), `T-ANON-1..3` (Aussonderung, Verknüpfbarkeit, Ableitbarkeit) |
| Z6 | `T-VCS-ERASE-1`: Hashes, Commits, Ledger vor und nach Löschung identisch |
| Z7 | `T-TT-*`: gleiche Parameter, byte-identische Ergebnisse; Stichtagsabfragen |
| Z8 | `T-DEC-1`: Begründungskette nach Löschung vollständig; Personeneingaben `Erased` |
| Z9 | `T-SCEN-1`: `main` und Ebene P vor und nach Szenario identisch |
| Z10 | `T-NOFALLBACK-*`, `T-RNG-1` (Entropiefehler → kein Umschlag) |
| Z11 | `T-EVID-1`, `T-STUB-1` |
| Z12 | `T-REST-1..3` (alter Tresor abgewiesen, Erase-Replay) |
| Z13 | Compile-Fail `derived_requires_lineage` |
| Z14 | Property `no_shared_dek` |
| Z15 | `T-LOG-1` (Bytes-Grep über Ledger, Protokolle, Fehlertexte) |
| Z16 | `T-ACL-1..n` (Nutzer ohne Recht erhält weder Treffer noch Hinweise noch Zählwerte) |
| Z17 | `T-GROUND-*` (Aussagen ohne Beleg werden entfernt; Enthaltung) |
| Z18 | `T-EGRESS-1` (kein ausgehender Verkehr im Standard, Netzwerk-Sandbox) |

### 15.3 Architektur-Gates (`cargo xtask gates`)

| Gate | Prüfung |
| --- | --- |
| G1 | Level-Modell gegen `cargo metadata` (negative Fixtures) |
| G2 | Jeder Port genau einmal verdrahtet |
| G3 | Keine nicht injizierte Zeit- oder Zufallsquelle |
| G-CRY | Keine `Default`-Implementierung für Schlüsseltypen; kein `unwrap_or_default()` und kein `let _ =` auf `Result` in `acta-crypto`, `acta-vault`, `acta-seg`, `acta-erasure`, `acta-history` |
| G-FS | Dateizugriffe nur in `acta-fs` |
| G-ERA | Löschungen nur über den Löschkern |
| G-READ | Öffentliche Lesepfade nur über den gegateten Leser |
| G-UNSAFE | `unsafe` nur in `acta-sys`, jeder Block mit `// SAFETY:` |
| G-NOTOKIO | Kein `tokio` in Level 0 bis 6 |
| G-NOSERDE-DISK | Kein `serde`/`bincode` in Plattenformaten |
| G-HUB | Größenbudgets, HR-1 bis HR-8 |
| G-WORD | Wortfilter nach 1.6 |
| G-SUPPLY | Lizenzen, bekannte Schwachstellen, Mindest-Compilerversion (`cargo deny`, `cargo audit`) |
| G-DEF | Alle Crates in `default-members` |

Alle Gates laufen in der CI und lokal. Lints im Produktionscode: `deny(clippy::unwrap_used, expect_used, panic, indexing_slicing, let_underscore_must_use)`.

---

## 16. Berichte und Nachweise

| Bericht | Inhalt | Empfänger |
| --- | --- | --- |
| **Löschbericht** | Klassen G, K, P, D, C, Zeugen, Annahmen, Mitbetroffenenzahl, Zeitstempel; signiert | Verantwortlicher, Aufsicht |
| **Verzeichnis von Verarbeitungstätigkeiten (Art. 30)** | Aus Sammlungen, Datenklassen, Aufbewahrungsklassen, Empfängern und Egress-Freigaben erzeugter Entwurf | Datenschutzfunktion |
| **Auftragsverarbeitung (Art. 28)** | Vorlage aus Konfiguration | Betreiber |
| **Entscheidungsbericht** | `explain` für eine Entscheidung oder ein Szenario mit Vergleich | Fachbereich, Revision |
| **Qualitätsbericht** | Messwerte nach 9.6 mit Korpus und Konfigurationsstand | Fachbereich |
| **Betriebsbericht** | Rotation, Eskalationen, Selbsttest, Vergiftungen | Betrieb |

Alle Berichte verwenden die Sprachregeln aus 1.6. Dieser Text ist keine Rechtsberatung; die Rechtsfreigabe liegt beim Betreiber und beim Hersteller (Abschnitt 17.4).

---

## 17. Rechtliche Einordnung und Betreibergrenzen

### 17.1 Rechtsgrundlagen, aus denen Eigenschaften folgen [R]

- **Art. 17 DSGVO:** Löschung als Aufheben der Personenbeziehbarkeit; kein vorgeschriebenes technisches Verfahren.
- **Art. 5 Abs. 1 lit. c, e:** Datenminimierung, Speicherbegrenzung → Aufbewahrungsklassen, Sweeper, Trennung K/P.
- **Art. 25:** Datenschutz durch Technikgestaltung → Entkopplung als Grundarchitektur.
- **Art. 32:** Sicherheit der Verarbeitung → Verschlüsselung, Zugriffskontrolle, Protokollierung, Selbsttest.
- **Art. 12 Abs. 3, 19:** Fristen und Mitteilung an Empfänger.
- **Art. 30, 28:** Verzeichnis und Auftragsverarbeitung als Berichtsfunktion.

### 17.2 Einordnung der Schlüsselvernichtung [R]

Verschlüsselte Daten bleiben personenbezogen, solange ein Schlüssel existiert. Die Vernichtung aller Schlüssel entzieht dem Verantwortlichen die Mittel zur Wiederherstellung des Personenbezugs; das Restrisiko künftiger Algorithmenbrüche ist zu dokumentieren und durch Kryptoagilität (`alg`-Byte) und physische Bereinigung zu flankieren. Reines logisches Ausblenden (Soft-Delete) genügt für Indizes und Vektoren nicht; Acta kombiniert Sperre, Schlüsselentzug und physische Bereinigung. Auf § 35 BDSG (Ersatz der Löschung durch Sperrung bei unverhältnismäßigem Aufwand) baut Acta nicht auf.

### 17.3 Deklarierte Annahmen

| Kürzel | Annahme |
| --- | --- |
| **A-FLASH** | Wear-Leveling und Dateisystem-Journale können physische Blöcke unkontrolliert vorhalten; dagegen wirkt der Schlüsselentzug |
| **A-RAM** | Flüchtige Reste im Speicher werden durch `mlock`, `Zeroizing`, `MADV_DONTDUMP` und `RLIMIT_CORE = 0` minimiert, sind aber gegen Kernel-Angriffe nicht gesichert |
| **A-ANCHOR** | Der Schlüsselanker vernichtet Generationsgeheimnisse innerhalb der deklarierten Frist `anchor_destroy_max` |
| **A-CRYPTO** | Die eingesetzten Primitive bleiben im geforderten Zeithorizont ungebrochen |
| **A-EXT** | Daten bei externen Verarbeitern oder in Kundendokumenten außerhalb von Acta liegen außerhalb der Einflusssphäre |
| **A-DETECT** | Die automatische Erkennung von Personenbezug ist unvollständig; Standardmodus und Freigabeprozesse kompensieren das |
| **A-LLM** | Sprachmodelle sind nicht deterministisch; die gespeicherte Antwort ist der Beleg |

### 17.4 Freigabepflichten vor dem Produktivbetrieb

(1) Unabhängige kryptografische Prüfung, (2) Rechtsprüfung der Berichtsvorlagen und Sprachregeln, (3) Penetrationstest der API und der Parser, (4) Messbericht nach 9.6 auf dem Kundenkorpus, (5) Wiederherstellungsübung mit Erase-Replay.

---

## 18. Umsetzungsreihenfolge

Prinzip: **dünner vertikaler Durchstich vor horizontaler Breite.** Eine Stufe beginnt erst, wenn die Abnahme der vorherigen grün ist.

| Stufe | Inhalt | Abnahme |
| --- | --- | --- |
| **S0 Fundament** | Workspace, Toolchain 1.89.0 (gepinnt), `rings.toml`, `xtask gates`, CI; `acta-base`, `acta-sys`, `acta-fs` mit `FaultVfs`, `acta-testkit` | Gates grün; Fehlerinjektion funktioniert; negative Gate-Fixtures rot |
| **S1 Krypto und Tresor** | `acta-crypto` (Testvektoren), `acta-vault` mit `FileAnchor`, Wurzelrotation, `destroy_many` | `T-CRY`, `T-NOFALLBACK`, `T-RNG`, Rotation mit Absturz, Bytes-Grep über Tresor |
| **S2 Speicher und Gate** | `acta-gate`, `acta-seg` (Journal, Segmente, Manifest, beide Siegelmodi) | `T-CRASH` für Speicher, `no_shared_dek`, Golden-Formate |
| **S3 Versionskontrolle** | `acta-ledger`, `acta-history` (Objekte, Refs, Sichtbarkeitsmengen, Diff) | `T-TT`, Hash-Ketten-Test, Stichtagsabfragen |
| **S4 Durchstich Löschen** | `acta-erasure` (E0 bis E3, MK2, Bericht), `acta-text` mit Lineage, `acta-verify` | `T-GATE-1`, `T-VCS-ERASE-1`, Crash C-Matrix E1 bis E3, Bericht mit `acta-verify` prüfbar |
| **S5 Abruf und Entscheidungen** | `acta-vec`, `acta-rag`, `acta-decision`, `acta-access`, `acta-infer`, `acta-infer-http` | `T-ACL`, `T-GROUND`, `T-DEC`, Qualitätsbericht auf Referenzkorpus |
| **S6 Vollständiges Löschen** | E4 bis E9, Rebuild, Restore-Schutz, Takedown, Aufbewahrung | `T-REST`, `T-MULTI`, `T-DERIVED-SEALED`, vollständige Crash-Matrix |
| **S7 Szenarien und Merge** | `acta-decision` Szenarien, `merge`, `retire`, Vier-Augen-Prozesse | `T-SCEN`, Szenario-Vergleich |
| **S8 Betrieb** | `acta-engine`, `acta-node`, `acta-api`, `acta-server`, `acta-cli`, Selbsttest, Berichte, Sicherung/Wiederherstellung | Selbsttest, Restore-Übung, `T-EGRESS`, Lasttest |
| **S9 Härtung** | Fuzzing aller Parser, externe Prüfungen (17.4), Dokumentation | Freigabepflichten erfüllt |

Parallele Spuren ab S2: **Ingest und Erkennung** (`acta-ingest`, `acta-pii` mit Fuzzing), **Berichte** (`acta-reports`), **Qualität** (Prüfkorpus, Messwerkzeug).

---

## Anhang A – Kanonische Objektformate (Auszug)

**Commit (kanonische Bytes):** `"acta/commit/v1" ‖ tree(32) ‖ n_parents u8 ‖ parents(32·n) ‖ tx_seq u64be ‖ tx_time i64be ‖ valid_from i32be (Tage, 0 = keiner) ‖ valid_to i32be ‖ kind u8 ‖ author(16) ‖ note_tag u16be`.

**Tree-Eintrag:** `kind u8 ‖ name_len u16be ‖ name (UTF-8, NFC) ‖ id (32 oder 16) ‖ meta_len u16be ‖ meta`. Einträge sind aufsteigend nach Name (Bytevergleich) sortiert; doppelte Namen sind ungültig. `kind = 0x03` (`PersonalRef`) hat `id` von 16 Byte.

**Ledger-Eintrag:** `[seq u64be][kind u8][len u32be][payload][entry_hash 32]` mit `entry_hash = SHA-256("acta/ledger/v1" ‖ prev_hash ‖ seq ‖ kind ‖ len ‖ payload)`.

## Anhang B – Löschbericht (kanonisches JSON, RFC 8785)

```json
{
  "assumptions": ["A-ANCHOR", "A-CRYPTO", "A-EXT", "A-FLASH", "A-RAM"],
  "classes_reached": { "C": true, "D": true, "G": true, "K": true, "P": true },
  "collateral_count": 0,
  "erasure_id": "018f4a2b-8c3d-7e9a-b1c2-d3e4f5a6b7c8",
  "mode": "DeleteAll",
  "notary_key_id": "anchor-key-1",
  "public_fingerprint": "ed25519:…",
  "residual_risks": ["Flash wear-leveling slack blocks retained without key (A-FLASH)"],
  "scope_commitment": "hmac-sha256:…",
  "target_type": "Subject",
  "timestamps": { "completed_at": 0, "gated_at": 0, "key_destroyed_at": 0, "requested_at": 0 },
  "witnesses": [
    { "layer": "Segments", "matches": 0, "method": "ForensicRawScan", "scanned_bytes": 0 },
    { "layer": "Vector",   "matches": 0, "method": "NeedleScan",      "scanned_segments": 0 }
  ]
}
```

Die Signatur wird über `acta/report/v1 ‖ kanonische_bytes_ohne_signature` gebildet und als `"signature": "base64url:…"` angehängt. Werte `0` und `…` sind Platzhalter des Formats. `acta-verify` prüft Signatur, Ledger-Kette und den Tresor-Eintrag am Datenträger ohne Rückgriff auf andere Acta-Crates.

## Anhang C – Verzeichnisstruktur des Repositorys

```
acta/
├─ Cargo.toml                 Workspace (Mitglieder = default-members), resolver = "3"
├─ rust-toolchain.toml        1.89.0, rustfmt, clippy
├─ rings.toml                 Level je Crate (Quelle für Gate G1)
├─ clippy.toml  deny.toml  .cargo/config.toml  (Alias: xtask)
├─ docs/  SPEC.md  adr/  betrieb/  datenschutz/
├─ crates/
│   acta-base  acta-crypto  acta-fs  acta-sys  acta-gate  acta-access
│   acta-infer  acta-pii  acta-ingest  acta-ledger  acta-vault  acta-seg
│   acta-history  acta-erasure  acta-text  acta-vec
│   acta-decision  acta-rag  acta-reports  acta-engine
│   acta-node  acta-infer-http  acta-api  acta-server  acta-cli  acta-testkit
├─ tools/ acta-verify         (eigenständig, ohne interne Abhängigkeiten)
└─ xtask/
```

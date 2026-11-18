# contextra-privacy — Crate-Spezifikation (Datenschutz, DLP & Egress-Shield)

**Ebene:** E0 (Autarkes Fundament / Ring 0)  
**Kern-Closure:** Leer (0 interne Workspace-Abhängigkeiten)  
**Status:** Normativ  
**Version:** 1.0  
**Geltungsbereich:** `crates/contextra-privacy`  
**Bindung:** `docs/SPEC.md` §2 (Architektur), §3 (Datenklassifikation & DSGVO-Kapselung), §13 (Invarianten)

---

## 1. Zweck & Systemrolle

`contextra-privacy` ist das fundamentale Datenschutz- und Data-Loss-Prevention-Crate (DLP) von Contextra. Als autarkes E0-Fundament definiert es die kanonische Datenklassifikation (`DataClass`), stellt den Typestate-Wrapper `Validated<T>` für Klasse-K-Schreibpfade bereit und schützt ausgehende Datenströme (Cloud Egress) vor unerwünschtem Abfluss personenbezogener Daten (PII).

---

## 2. Architektur & Komponenten

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              contextra-privacy                              │
│                                                                             │
│   ┌──────────────────────────┐             ┌────────────────────────────┐   │
│   │        DataClass         │             │        Validated<T>        │   │
│   │   (K, P, D, E Enums)     │             │ (Type-State Payload Guard) │   │
│   └─────────────┬────────────┘             └─────────────┬──────────────┘   │
│                 │                                        │                  │
│                 ▼                                        ▼                  │
│   ┌──────────────────────────┐             ┌────────────────────────────┐   │
│   │     EntityRecognizer     │             │       EgressGateway        │   │
│   │  (Regex PII Detection:   │             │   (Cloud Query Filtering,  │   │
│   │   IBAN, Mail, Tel, PII)  │             │    Shielding & Rehydration)│   │
│   └──────────────────────────┘             └────────────────────────────┘   │
│                                                                             │
│   ┌──────────────────────────┐             ┌────────────────────────────┐   │
│   │      SurrogateVault      │             │      ContextEditAudit      │   │
│   │ (Pseudonymisierung durch │             │ (Kryptographisch verkettete│   │
│   │  Surrogat-Token im RAM)  │             │  Audit-Logs bei Redaction) │   │
│   └──────────────────────────┘             └────────────────────────────┘   │
│                                                                             │
│   ┌──────────────────────────┐             ┌────────────────────────────┐   │
│   │ BulkExfiltrationDetector │             │     ProcessingRegistry     │   │
│   │ (Session Rate-Limiting   │             │ (Verzeichnis der Verarbei- │   │
│   │  gegen Massen-Abfluss)   │             │  tungstätigkeiten / VVT)   │   │
│   └──────────────────────────┘             └────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Datenklassifikation & Typestate (`DataClass` & `Validated<T>`)

### 3.1 Die vier Datenklassen (SPEC §3.1)

`contextra-privacy` ist der **alleinige kanonische Besitzer** der `DataClass`-Definition:

```rust
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DataClass {
    /// Klasse K: Unternehmenswissen (Verträge, Handbücher, Fachwissen). Verschlüsselt mit Collection-KEK.
    K,
    /// Klasse P: Personenbezogene Daten natürlicher Personen. Verschlüsselt mit individuellem SubjectKey.
    P,
    /// Klasse D: Abgeleitete Daten (Vektoren, Postings, Graph-Knoten). Tragen verpflichtend Lineage.
    D,
    /// Klasse E: Löschmaterial (Journal, Ledger, signierte Berichte, Attestierungen).
    E,
}
```

### 3.2 Der `Validated<T>` Typestate-Wrapper

Um zu garantieren, dass **keine unerkannte PII in Klasse-K-Pfade gelangt**, erzwingt das Typsystem einen Type-State:
```rust
pub struct Validated<T>(/* private */ T);

impl<T> Validated<T> {
    pub fn inner(&self) -> &T {
        &self.0
    }
    pub fn into_inner(self) -> T {
        self.0
    }
}
```
- **Privater Konstruktor:** Kann von außen **ausschließlich** über die Funktion `validate()` erzeugt werden.
- **Klasse-K-Schreibpfad:** `db.put_k(...)` akzeptiert ausschließlich `Validated<Payload>`. Ein Versuch, rohe Bytes oder unvalidierte Strings zu übergeben, scheitert bereits zur Compile-Zeit.

```rust
pub fn validate<T: AsRef<[u8]>>(
    payload: T,
    recognizer: &EntityRecognizer,
) -> Result<Validated<T>, PersonalDataFound>;
```

---

## 4. PII-Erkennung & Pseudonymisierung (`EntityRecognizer`, `SurrogateVault`)

### 4.1 `EntityRecognizer`

Regelbasierte Hochgeschwindigkeits-Erkennung sensibler Datenmuster:
- Internationale Bankkontonummern (IBAN) mit Prüfziffern-Validierung
- E-Mail-Adressen (RFC 5322 konform)
- Telefonnummern (E.164 und nationale Formate)
- Steuer-Identifikationsnummern und Sozialversicherungsnummern
- Kreditkartennummern (Luhn-Algorithmus)

### 4.2 `SurrogateVault`

Wird PII in Texten erkannt, die an externe LLMs übermittelt werden sollen:
1. PII-Werte werden durch syntaktisch gleichwertige Surrogat-Token (z. B. `SURROGATE_IBAN_01`) ersetzt.
2. Das Mapping `Surrogat ↔ Originalwert` wird flüchtig im `SurrogateVault` im RAM gehalten.
3. Nach Rückkehr der Antwort aus der Cloud rehydriert der `CloudResponseRehydrator` die Surrogat-Token wieder zu den Originalwerten.
4. Der Tresor unterliegt `ZeroizeOnDrop`.

---

## 5. Cloud-Egress-Schutz & Auditierung

### 5.1 `EgressGateway`

Zentraler Kontrollpunkt vor jedem Netzwerkaufruf an Cloud-Inferenz-Provider:
- Bewertet Anfragen via `EgressPolicy`.
- Blockiert Anfragen, falls unmaskierte PII enthalten ist oder der Mandant externe Cloud-Verarbeitung untersagt.

### 5.2 `ContextEditAudit`

Jede Schwärzung oder Modifikation von Kontexten wird kryptographisch unveränderbar dokumentiert:
- Verkettete SHA-256 / BLAKE3-Prüfsummen (`verify_audit_chain`).
- Enthält Zeitstempel, Regel-ID und Hash des bearbeiteten Bereichs (ohne den Klartext zu speichern).

### 5.3 `BulkExfiltrationDetector`

Sitzungsbasierte Flusskontrolle zur Erkennung automatisierter Exfiltrationsversuche (z. B. Scraping ganzer Datenbanken über MCP-Tools).

---

## 6. Abhängigkeiten

| Art | Bibliothek |
|-----|-----------|
| **Intern** | Keine (0 interne Workspace-Abhängigkeiten) |
| **Extern** | `regex`, `serde`, `serde_json`, `thiserror`, `tracing`, `zeroize` |

---

## 7. Invarianten

1. **`#![forbid(unsafe_code)]`** im gesamten Crate.
2. **Klasse-K-Garantie:** Kein Klartext-K-Schreibvorgang ohne vorherige `Validated<T>`-Zertifizierung.
3. **Keine PII in Logs:** `EntityRecognizer`-Fehler und Audit-Logs enthalten niemals die erkannten sensiblen Klartextwerte, sondern nur Typ und Position.
4. **Zero-Panic-Policy:** Reguläre Ausdrücke und Parser sind defensiv konstruiert und paniken nicht bei bösartigen Eingaben (ReDoS-resistent).

---

## 8. Test-Pflichten

| Test-ID | Beschreibung |
|---------|-------------|
| T-PRV1 | `validate`: Akzeptiert reinen Fachtext und liefert `Validated<T>` |
| T-PRV2 | `validate`: Erkennt IBAN / E-Mail und bricht mit `PersonalDataFound` ab |
| T-PRV3 | `SurrogateVault`: Ersetzt PII durch Surrogat und rehydriert Antwort fehlerfrei |
| T-PRV4 | `ContextEditAudit`: Manipulation an einem Audit-Eintrag bricht die Prüfkette |
| T-PRV5 | `BulkExfiltrationDetector`: Überschreitung des Grenzwerts löst Blockade aus |

# contextra-privacy — Crate-Spezifikation

**Ebene:** E0 (Kern-Closure leer)  
**Bindung:** `docs/SPEC.md` §2, §3

---

## 1 Zweck

Datenschutz-Primitive: Entity-Recognizer, `Validated<T>`, Datenklassifikation.

## 2 Verantwortlichkeit

### 2.1 `Validated<T>`

Typestate-Wrapper, der belegt, dass ein Payload den Entity-Recognizer durchlaufen hat und keine unerwarteten personenbezogenen Daten enthält. Privater Konstruktor — nur über `privacy::validate()` erzeugbar.

```rust
pub struct Validated<T>(/* private */ T);

impl<T> Validated<T> {
    pub fn inner(&self) -> &T { &self.0 }
}

pub fn validate<T: AsRef<[u8]>>(payload: T, recognizer: &EntityRecognizer)
    -> Result<Validated<T>, PersonalDataFound>;
```

### 2.2 `EntityRecognizer`

Regelbasierte Erkennung von PII-Mustern (E-Mail, Telefon, IBAN, Name). Konfigurierbar über Pattern-Listen.

### 2.3 `DataClass`

```rust
#[non_exhaustive]
pub enum DataClass { K, P, D, E }
```

Besitzer der Enum-Definition. Alle anderen Crates importieren aus `privacy`.

## 3 Abhängigkeiten

Keine internen Abhängigkeiten. Nur `regex`.

## 4 Rolle in der Löschkette

- K-Schreibvorgänge nehmen nur `Validated<T>` entgegen → Compile-Fail wenn roher Payload übergeben wird.
- Stellt sicher, dass K-Daten keine unerkannten personenbezogenen Daten enthalten (Best-Effort, kein absoluter Schutz).

# contextra-privacy — Implementierungsspezifikation (Soll-Zustand)

**Ebene:** E0 (Autarkes Fundament / Ring 0)  
**Kern-Closure:** `ids`, `env` (erlaubte E0-Fundamente gemäß SPEC §2.1)  
**Status:** Normativ (Soll-Zustand für Entwickler)  
**Version:** 2.0  
**Geltungsbereich:** `crates/contextra-privacy`  
**Referenzdokumente:** `docs/SPEC.md` §2, §3 (Datenklassifikation & DSGVO-Kapselung), §13; `ANALYSIS-E0-CONTRADICTIONS.md` (D-02, D-03, D-09); `ALTLASTEN-AUDIT.md` (AP-21)  

---

## 1. Zweck & Architekturgarantien

`contextra-privacy` ist das fundamentale Datenschutz-, DLP- (Data Loss Prevention) und Egress-Shield-Crate von Contextra:
- **Kanonische Datenklassifikation (`DataClass`):** Alleiniger kanonischer Besitzer der Klassifikation `K`, `P`, `D`, `E` (SPEC §3.1).
- **Typestate-Schutz für Klasse K (`Validated<T>`):** Compile-Zeit-Garantie, dass keine unerkannte PII in Unternehmenswissens-Pfade (K) gelangt.
- **PII-Erkennung (`EntityRecognizer`):** Hochgeschwindigkeits-Regex- und Regelwerkserkennung von IBANs, E-Mails, Telefonnummern und Steuer-IDs.
- **Flüchtige Pseudonymisierung (`SurrogateVault`):** Ersetzen von PII durch Surrogat-Token vor externer Cloud-Inferenz; Rehydrierung bei Rückkehr.
- **Cloud-Egress-Shield (`EgressGateway`):** Filterung, Auditierung und Schutz vor Massen-Exfiltration (`BulkExfiltrationDetector`).

### Beseitigung von Typ-Dubletten (D-09)
Gemäß SPEC §2.1 dürfen E0-Fundamente `contextra-ids` und `contextra-env` nutzen. `contextra-privacy` importiert `TenantId`, `TenantScoped`, `TenantScopeViolation` und `Clock` direkt aus den kanonischen E0-Crates. Alle lokalen Kopien in `types.rs` werden gelöscht.

### Architekturgarantien (R1–R6, INV-3, INV-4, INV-11)
1. **`#![forbid(unsafe_code)]`:** 100 % speichersicher.
2. **Zero-Panic-Policy:** Keine Panics bei bösartigen oder ungültigen Eingaben.
3. **Leere Default-Features (R3):** `default = []`.
4. **Schutz sensibler Daten:** `SurrogateVault` implementiert `ZeroizeOnDrop`.
5. **Keine PII in Logs (INV-12):** Audit-Logs und Traces enthalten niemals Klartext-PII, sondern nur Regel-IDs, Token-Hashes und Byteranges.

---

## 2. Kanonische Datenklassifikation & Typestate

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
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### 2.1 `DataClass` (Kanonischer Besitzer — SPEC §3.1, D-02)

```rust
/// Die vier normativen Datenklassen des Contextra-Systems (SPEC §3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DataClass {
    /// Klasse K: Unternehmenswissen (Verträge, Handbücher, Richtlinien).
    /// Verschlüsselt mit Collection-KEK. Löschbar nur per Collection-Drop.
    K,

    /// Klasse P: Personenbezogene Daten natürlicher Personen (Name, E-Mail, Interaktionen).
    /// Adressiert über `SubjectRef`, verschlüsselt mit individuellem SubjectKey.
    /// Atomar löschbar in O(1) via Crypto-Shredding.
    P,

    /// Klasse D: Abgeleitete Daten (Embeddings, Postings, Graph-Knoten, Caches).
    /// Tragen verpflichtend `LineageRef` zum Quelldatum für kaskadierte Bereinigung.
    D,

    /// Klasse E: Löschmaterial (Journal, Ledger, signierte Berichte, Attestierungen).
    /// Verschlüsselt mit separatem ErasureKey; eigenständig nach Aufbewahrungsfrist bereinigt.
    E,
}
```

---

### 2.2 Typestate `Validated<T>` für Klasse-K-Schreibpfade (D-03)

Verhindert zur Compile-Zeit, dass unerkannte personenbezogene Daten in Klasse-K-Datenbestände gelangen:

```rust
/// Typestate-Wrapper: Bezeugt, dass der innere Payload erfolgreich gegen
/// PII-Muster geprüft wurde und keine personenbezogenen Daten enthält.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Validated<T>(T);

impl<T> Validated<T> {
    /// Liefert eine Referenz auf die validierten Daten.
    #[inline]
    pub fn inner(&self) -> &T {
        &self.0
    }

    /// Verbraucht den Wrapper und gibt die validierten Daten frei.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }
}

/// Prüft einen Text auf PII. Bei Fund schlägt die Validierung fehl.
pub fn validate_k_payload<T: AsRef<str>>(
    payload: T,
    recognizer: &EntityRecognizer,
) -> Result<Validated<T>, PersonalDataFound> {
    if let Some(mat) = recognizer.find_first_pii(payload.as_ref()) {
        return Err(PersonalDataFound {
            entity_type: mat.entity_type,
            byte_range: mat.start..mat.end,
        });
    }
    Ok(Validated(payload))
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("Personal data detected in Class K payload: entity '{entity_type}' at range {byte_range:?}")]
pub struct PersonalDataFound {
    pub entity_type: &'static str,
    pub byte_range: std::ops::Range<usize>,
}
```

---

### 2.3 `EntityRecognizer` (Regelbasierte PII-Erkennung)

```rust
use regex::Regex;

/// Fundstelle eines erkannten sensitiven Datums.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityMatch {
    pub entity_type: &'static str,
    pub start: usize,
    pub end: usize,
}

pub struct EntityRecognizer {
    iban_regex: Regex,
    email_regex: Regex,
    phone_regex: Regex,
}

impl EntityRecognizer {
    pub fn new() -> Result<Self, PrivacyError> {
        Ok(Self {
            iban_regex: Regex::new(r"\b[A-Z]{2}[0-9]{2}[A-Z0-9]{4}[0-9]{7}([A-Z0-9]?){0,16}\b")
                .map_err(PrivacyError::Regex)?,
            email_regex: Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b")
                .map_err(PrivacyError::Regex)?,
            phone_regex: Regex::new(r"\b(?:\+?[0-9]{1,3})?[-. (]*[0-9]{2,4}[-. )]*[0-9]{3,4}[-. ]*[0-9]{3,9}\b")
                .map_err(PrivacyError::Regex)?,
        })
    }

    pub fn find_first_pii(&self, text: &str) -> Option<EntityMatch> {
        if let Some(m) = self.iban_regex.find(text) {
            return Some(EntityMatch { entity_type: "IBAN", start: m.start(), end: m.end() });
        }
        if let Some(m) = self.email_regex.find(text) {
            return Some(EntityMatch { entity_type: "EMAIL", start: m.start(), end: m.end() });
        }
        if let Some(m) = self.phone_regex.find(text) {
            return Some(EntityMatch { entity_type: "PHONE", start: m.start(), end: m.end() });
        }
        None
    }

    pub fn find_all_pii(&self, text: &str) -> Vec<EntityMatch> {
        let mut matches = Vec::new();
        for m in self.iban_regex.find_iter(text) {
            matches.push(EntityMatch { entity_type: "IBAN", start: m.start(), end: m.end() });
        }
        for m in self.email_regex.find_iter(text) {
            matches.push(EntityMatch { entity_type: "EMAIL", start: m.start(), end: m.end() });
        }
        for m in self.phone_regex.find_iter(text) {
            matches.push(EntityMatch { entity_type: "PHONE", start: m.start(), end: m.end() });
        }
        matches.sort_by_key(|m| m.start);
        matches
    }
}
```

---

### 2.4 Pseudonymisierung & Tresor (`SurrogateVault`)

```rust
use std::collections::HashMap;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Flüchtig im RAM gehaltener Zuordnungsspeicher für Pseudonyme.
/// Implementiert ZeroizeOnDrop zum Schutz vor Speicherabzügen.
#[derive(Default, Zeroize, ZeroizeOnDrop)]
pub struct SurrogateVault {
    surrogate_to_original: HashMap<String, String>,
}

impl SurrogateVault {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, surrogate: String, original: String) {
        self.surrogate_to_original.insert(surrogate, original);
    }

    pub fn rehydrate(&self, text: &str) -> String {
        let mut result = text.to_string();
        for (surrogate, original) in &self.surrogate_to_original {
            result = result.replace(surrogate, original);
        }
        result
    }
}
```

---

### 2.5 Cloud Egress Gateway & GuardedPayload

Compile-Zeit Typestate für Cloud-Anfragen:
```rust
use std::marker::PhantomData;
use zeroize::Zeroize;

pub struct Unsanitized;
pub struct Sanitized;

#[derive(Debug)]
pub struct GuardedPayload<State> {
    payload: String,
    session_id: String,
    _state: PhantomData<State>,
}

impl<S> Drop for GuardedPayload<S> {
    fn drop(&mut self) {
        self.payload.zeroize();
        self.session_id.zeroize();
    }
}

impl GuardedPayload<Unsanitized> {
    pub fn new(payload: String, session_id: String) -> Self {
        Self {
            payload,
            session_id,
            _state: PhantomData,
        }
    }
}

impl GuardedPayload<Sanitized> {
    pub fn from_sanitized(payload: String, session_id: String) -> Self {
        Self {
            payload,
            session_id,
            _state: PhantomData,
        }
    }

    pub fn into_inner(mut self) -> String {
        std::mem::take(&mut self.payload)
    }
}
```

---

## 3. Fehlerbehandlung (`PrivacyError`)

```rust
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum PrivacyError {
    #[error("Personal data detected in Class K write: {0}")]
    PersonalDataDetected(#[from] PersonalDataFound),

    #[error("Regex compilation error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Cloud egress blocked by policy: {0}")]
    EgressBlocked(String),

    #[error("Bulk exfiltration limit exceeded for session {0}")]
    ExfiltrationLimitExceeded(String),

    #[error("Audit chain verification failed: expected hash {expected}, got {actual}")]
    AuditChainCorrupted { expected: String, actual: String },
}
```

---

## 4. `Cargo.toml` Spezifikation

```toml
[package]
name = "contextra-privacy"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Canonical data classification, PII DLP filtering, and Cloud Egress Guard for Contextra"

[lints]
workspace = true

[dependencies]
contextra-ids = { workspace = true }
contextra-env = { workspace = true }
thiserror = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
zeroize = { version = "1.8", features = ["zeroize_derive"] }
regex = "1"
tracing = { workspace = true, optional = true }

[dev-dependencies]
tempfile = { workspace = true }

[features]
default = []
std = ["contextra-ids/std", "contextra-env/std"]
tracing = ["dep:tracing"]
```

---

## 5. Implementierungs-Arbeitspakete für Entwickler

1. **`Cargo.toml` Bereinigung:**
   - Hinzufügen von `contextra-ids` und `contextra-env` zu `[dependencies]`.
   - Entfernen von `tokio`, `scc`, `rand` und `parking_lot` aus den Pflichtabhängigkeiten.
   - `default = []` setzen (R3).
2. **Kanonisches `DataClass` Enum erstellen:**
   - Datei `src/data_class.rs` anlegen und an `lib.rs` exportieren.
3. **`Validated<T>` Pattern implementieren:**
   - `validate_k_payload` implementieren und mit `EntityRecognizer` verdrahten.
4. **Typ-Dubletten in `src/types.rs` löschen:**
   - Lokale `TenantId`, `TenantScoped`, `TenantScopeViolation` und `Clock` entfernen.
   - Durch Re-Exports aus `contextra-ids` und `contextra-env` ersetzen.
5. **Altlasten bereinigen (`AP-21`):**
   - Veraltete Referenz auf `contextra-router` in `src/egress_vault.rs` Zeile 555 entfernen.
6. **Verifikation:**
   - `cargo test -p contextra-privacy`

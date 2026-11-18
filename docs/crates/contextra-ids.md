# contextra-ids — Crate-Spezifikation (Identitätstypen & Referenzen)

**Ebene:** E0 (Autarkes Fundament / Ring 0)  
**Kern-Closure:** Leer (0 interne Workspace-Abhängigkeiten)  
**Status:** Normativ  
**Version:** 1.0  
**Geltungsbereich:** `crates/contextra-ids`  
**Bindung:** `docs/SPEC.md` §2 (Architektur), §3.2 (Kapselung personenbezogener Daten), §4 (Schlüsselhierarchie), §13 (Invarianten)

---

## 1. Zweck & Systemrolle

`contextra-ids` ist der fundamentale Mikro-Kernel für Identifikatoren, Mandanten-IDs, Referenzen und Sequenznummern des gesamten Contextra-Systems. Es besitzt keinerlei interne Workspace-Abhängigkeiten und definiert die kanonischen Repräsentationen für alle Entitäten und Identitäten.

---

## 2. Definierte Identitätstypen & Bitbreiten

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                contextra-ids                                │
│                                                                             │
│   ┌──────────────────────────┐             ┌────────────────────────────┐   │
│   │          DocId           │             │         SubjectRef         │   │
│   │ (64-Bit / 128-Bit BLAKE3)│             │ (128-Bit PII-Zufall, Zero) │   │
│   └──────────────────────────┘             └────────────────────────────┘   │
│   ┌──────────────────────────┐             ┌────────────────────────────┐   │
│   │       PersonalRef        │             │          TenantId          │   │
│   │(128-Bit P-Verweis in K)  │             │ (TenantId(0) = SYSTEM)     │   │
│   └──────────────────────────┘             └────────────────────────────┘   │
│   ┌──────────────────────────┐             ┌────────────────────────────┐   │
│   │       CollectionId       │             │            TxId            │   │
│   │   (Muss > 0 sein)        │             │  (Transaktionssequenz)     │   │
│   └──────────────────────────┘             └────────────────────────────┘   │
│   ┌──────────────────────────┐             ┌────────────────────────────┐   │
│   │        ErasureId         │             │      LayerId / FileId      │   │
│   │ (128-Bit Saga-Zufalls-ID)│             │ (Ebene / Datei-Deskriptor) │   │
│   └──────────────────────────┘             └────────────────────────────┘   │
│   ┌──────────────────────────┐             ┌────────────────────────────┐   │
│   │     TOMBSTONE_BIT        │             │        MAX_SEARCH_K        │   │
│   │     (Bit 63 = 1 << 63)   │             │   (Obergrenze = 1.000)     │   │
│   └──────────────────────────┘             └────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.1 `DocId` (Dokument-Identifikator)

- **Standard:** 64-Bit numerischer Identifikator basierend auf BLAKE3-Hash-Präfix.
- **Konfigurierbar via Feature `docid-128`:** 128-Bit breiter Identifikator für Korpora mit extremer Skalierung.
- **Compile-Time Layout-Check:**
  ```rust
  #[cfg(not(feature = "docid-128"))]
  const _: () = {
      assert!(std::mem::size_of::<DocId>() == 8);
      assert!(std::mem::align_of::<DocId>() == 8);
  };
  #[cfg(feature = "docid-128")]
  const _: () = {
      assert!(std::mem::size_of::<DocId>() == 16);
      assert!(std::mem::align_of::<DocId>() == 16);
  };
  ```
- **Kollisionserkennung:** Eine 64-Bit-Präfixkollision DARF NIEMALS zu einer stillen Überschreibung führen. Die Kollision MUSS erkannt und mit `Err(IdError)` abgewiesen werden.

### 2.2 `SubjectRef` (Personen-Identifikator — SPEC §3.2)

- **128-Bit-Zufalls-ID** zur Identifikation einer natürlichen Person (Betroffener im Sinne von Art. 4 Nr. 1 DSGVO).
- **Fehlbare Erzeugung:** MUSS zwingend über `rand::rngs::OsRng::try_fill_bytes` erzeugt werden.
- **Sicherheits-Eigenschaften:**
  - **Kein `Clone`:** Verhindert das unkontrollierte Duplizieren von Betroffenen-Identitäten im Speicher.
  - **`ZeroizeOnDrop`:** Da `SubjectRef` als Eingabe für die HKDF-Schlüsselableitung (`SubjectKey`) dient, wird das Speicherabbild beim Verlassen des Gültigkeitsbereichs mit Nullen überschrieben.
  - **Geschwärztes `Display`:** Implementiert kein `Display`, das den vollen hexadezimalen Wert ausgibt (Verhinderung von Log-Leaks).

### 2.3 `PersonalRef` (P-Referenz in K-Dokumenten)

- **128-Bit-Zufalls-ID** als loser Zeiger auf personenbezogene Daten innerhalb von Klasse-K-Dokumenten.
- Enthält selbst keine Klartext-PII.
- Wird bei Löschung des Betroffenen zu einem toten Link.

### 2.4 `TenantId` & Mandantenisolation

- `TenantId(0)` ist fest für interne Systemoperationen reserviert (`TenantId::SYSTEM`).
- Normale Mandanten-IDs müssen $> 0$ sein.
- `IdError::TenantSystemReserved` schützt den System-Namensraum vor Fremdnutzung.

### 2.5 Konstanten

- **`TOMBSTONE_BIT = 1 << 63`:** Bit 63 einer Sequenznummer markiert einen MVCC-Löschmarker (Soft-Deletion).
- **`MAX_SEARCH_K = 1_000`:** Obergrenze für $k$-NN und Retrieval-Ergebnismengen.
- **`MAX_REF_LEN = 256`:** Maximale Bytelänge für Referenzbezeichner.

---

## 3. Fehlerbehandlung (`IdError`)

```rust
#[non_exhaustive]
#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
pub enum IdError {
    #[error("Key cannot be empty")]
    EmptyKey,
    #[error("empty identifier")]
    EmptyIdentifier,
    #[error("identifier too long")]
    IdentifierTooLong,
    #[error("TenantId(0) is reserved for TenantId::SYSTEM")]
    TenantSystemReserved,
    #[error("CollectionId cannot be 0")]
    CollectionZero,
    #[error("Tenant scope boundary violation: attempted cross-tenant access between {expected} and {actual}")]
    TenantScopeViolation { expected: String, actual: String },
}
```

---

## 4. Abhängigkeiten

| Art | Bibliothek |
|-----|-----------|
| **Intern** | Keine (0 interne Abhängigkeiten) |
| **Extern** | `blake3`, `rand` 0.8, `serde`, `thiserror`, `zeroize` |

---

## 5. Invarianten

1. **`#![forbid(unsafe_code)]`** im gesamten Crate.
2. **Kein `Clone` für `SubjectRef` und `PersonalRef`.**
3. **`ZeroizeOnDrop` für `SubjectRef`.**
4. **Compile-Time Layout-Prüfung:** `size_of::<DocId>() == 8` bzw. `16`.
5. **Kein `Uuid::new_v4()`:** Keine unkontrollierten Nicht-Deterministischen Zufallsgeneratoren abseits von `OsRng::try_fill_bytes`.

---

## 6. Test-Pflichten

| Test-ID | Beschreibung |
|---------|-------------|
| T-ID1 | `SubjectRef`: Erzeugung via `try_fill_bytes` schlägt bei RNG-Erschöpfung kontrolliert fehl |
| T-ID2 | `SubjectRef`: Speicher wird beim Droppen genullt (`Zeroize`) |
| T-ID3 | `DocId`: BLAKE3-Präfixberechnung ist deterministisch |
| T-ID4 | `TenantId(0)` führt beim normalen Allokieren zu `TenantSystemReserved` |
| T-ID5 | `CollectionId(0)` wird mit `CollectionZero` abgewiesen |
| T-ID6 | Layout-Assertions kompilieren fehlerfrei unter `docid-128` und 64-Bit |

# Phase C: Modularisierung und Architekturgrenzen

## 1. Öffentliche Schnittstellen & Public Field Exposure
- **Öffentliche Felder in Ring 0/1 Structs**: Es wurden **109 Structs** in Ring-0- und Ring-1-Crates identifiziert, deren Felder direkt als `pub` deklariert sind (z. B. `ModelFingerprint`, `MemoryImportance`, `TokenBudget`, `ContextChunk`).
  - *Sicherheits-/Architektur-Einfluss*: Bei Typen mit Geschäfts-Invarianten (z. B. `TokenBudget.limit` oder `MemoryImportance.created_at_tx`) hebelt der direkte Schreibzugriff über `pub`-Felder die Invarianten-Kapselung aus.
- **Re-Export-Kaskaden**: `contextra-ports` und `contextra-types` kaskadieren Re-Exporte über `pub use *`, was Typ-Sichtbarkeiten verschleiert.

---

## 2. Ring-Verletzungen & Port-Nutzung
- **Direct Backend Coupling in Ring 1**: In `crates/contextra-checkpoint/src/orphan.rs` bindet das Crate direkt `std::env::var("CONTEXTRA_ORPHAN_PATH")` ein, anstatt Umgebungskonfigurationen über den Fassaden-Init oder den `Clock`/`Env`-Port zu injizieren.
- **Port-Katalog Einhaltung**: `contextra-ports` definiert die abstrakt geforderten Ports (`Clock`, `Rng`, `Store`, etc.).
  - *Verletzung von `INV-NONDET-1`*: In `contextra-vector/src/hnsw/core_rebuild.rs` werden direkte Aufrufe von `std::time::Instant::now()` getätigt, um Deadlines und Timings zu berechnen, anstatt den injizierten `Clock`-Port zu nutzen.

---

## 3. Composition Root & Globale Zustände
- **Globale Zustände / Statics**: Es wurden **26 statische Zustände** (`static`, `OnceLock`, `LazyLock`, `AtomicU64`) in Ring-0/1-Crates gefunden.
  - `crates/contextra-ports/src/rng.rs:8`: `static ENTROPY_COUNTER: AtomicU64 = AtomicU64::new(1);` (Globale Pseudozufalls-Entropiequelle).
  - `crates/contextra-text/src/tokenizer.rs`: Statische `OnceLock`-Singletons für `STOPWORDS`, `PROTECTED_REGEX`, und `SHARED_SPLITTER`.
  - *Spezifikationsbewertung*: Die Spezifikation verlangt Instanziierung ausschließlich in der Composition Root (`contextra` Fassade) und verbietet globale Singletons mit veränderbarem oder verstecktem Zustand (Prinzip P7).

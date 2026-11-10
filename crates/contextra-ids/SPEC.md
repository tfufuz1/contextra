# contextra-ids — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v2, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `f0eda1cb9`). `cargo` ist in der Analyseumgebung nicht verfügbar. **Es wurde nichts gebaut, getestet oder ausgeführt.**
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-ids/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Identitäts- und Mandantentypen (DocId, EntityId, TxId, TenantId, CollectionId, Referenzen) ohne interne Abhängigkeit. *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | foundation |
| Ebene (Kern-Closure) | 0 |
| Kern-Closure | — |
| Default-Features | `std` |
| Fehlertyp (R3) | `IdError` |
| Brücke (hinter Feature `workspace`, §1.1) | — |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1 = Kern-Closure ⊆ E0, höchstens 4 Mitglieder · E2 = Integration · E3 = Anwendungen und Testwerkzeuge. Der Kern ist der Code mit `--no-default-features` ohne Feature `workspace`.

## 3. Ist-Zustand (tools/)

*Crate existiert im Repo noch nicht. Es wird in S-10/S-11 bzw. im jeweiligen Schritt angelegt.*

## 4. Ist-vs-Ziel-Abgleich [V]

Crate existiert noch nicht. Ist-Vergleich entfällt; Ziel siehe §2.

## 5. Verifizierte Befunde und Abweichungen zu docs/SPEC.md [V]

- Neu: Crate existiert noch nicht. Quellbasis ist `crates/contextra-types/src/types/domain/ids.rs` (`from_key` an Zeilen 225, 267, 348; BLAKE3-Präfix-Funktion `hash_key_u64` ca. Zeile 373). Die Quellstellen wurden gelesen, nicht gebaut.

## 6. Anforderungen

### 6.1 Vertrag (docs/SPEC.md §4.1 bzw. §4.2) [S]

#### 4.1 `contextra-ids`

**Zweck:** alle Identitäts- und Mandantentypen, ohne interne Abhängigkeit.

**Inhalt (Quelle → Ziel)** [E, Zeilenzahlen nicht neu gezählt]:
- `types/src/types/domain/ids.rs`: `DocId`, `EntityId`, `TxId` (inkl. `INTERNAL_BASE`), `TenantId` (inkl. `SYSTEM`), `CollectionId`; `from_key` per BLAKE3-Präfix.
- `types/src/ids.rs`: `FileId`, `StoreId`, `Seq`, `Timestamp`. `ErasureId` und `GateId` gehen **nicht** hierher, sondern nach `erasure` (S-51).
- `types/src/refs.rs`: `CollectionRef`, `DocKey`, `OperatorRef`, `SubjectRef`, `TenantRef`, `MAX_REF_LEN`.
- `types/src/tenant_scope.rs`: `TenantScoped`, `TenantScopeViolation`.
- Konstanten `TOMBSTONE_BIT`, `MAX_SEARCH_K` aus `types/.../misc.rs`.
- Tests `types/.../tests/ids_tests.rs` (473 Z.) ziehen mit.

**Dependencies:** `blake3`, `serde`, `thiserror`. Keine weiteren. `#![forbid(unsafe_code)]`.

**Öffentliche Schnittstelle (Vertrag):**

```rust
// Feature `docid-128` wählt die Breite für das GANZE Workspace-Build (nicht additiv; siehe B03).
#[cfg(not(feature = "docid-128"))] pub struct DocId(pub u64);        // repr(transparent)
#[cfg(feature = "docid-128")]     pub struct DocId(pub u128);       // repr(C, align(16))
pub const ID_WIDTH_BITS: u32 = if cfg!(feature = "docid-128") { 128 } else { 64 };

#[non_exhaustive]
#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
pub enum IdError {
    #[error("Key cannot be empty")] EmptyKey,
    // weitere Varianten 1:1 aus den heutigen InvalidInput-Stellen
    // (Zeilen 14, 64, 98 in ids.rs sowie refs.rs/tenant_scope.rs, per S-10 erhoben):
    // Text der Display-Ausgabe MUSS dem heutigen String entsprechen.
}

impl DocId {
    pub fn from_key(key: &str) -> Result<Self, IdError>;   // BLAKE3[..8] LE (64) bzw. [..16] (128)
    pub fn inner(&self) -> /* u64 | u128 */;
}
impl From<u64> for DocId { /* wie heute: id as _ */ }
```

**Re-Export:** `types` macht `pub use contextra_ids::*;` (Pfad `contextra_types::DocId` bleibt) und enthält `impl From<IdError> for ContextraError` (Variante `InvalidInput(String)`, Text = `IdError::to_string()`).
**Kompatibilitätsbruch (bewusst):** `DocId::from_key` & Co. liefern `Result<_, IdError>`; `?` in Funktionen mit `ContextraError`-Fehlertyp funktioniert über `From`. Direkte Musterabgleiche auf `ContextraError::InvalidInput` an Aufrufstellen sind in S-10 Commit C zu korrigieren.

**Abnahme (zusätzlich zu §8):**
1. `ids_tests.rs` grün in `ids`.
2. Differenztest gegen die **noch bestehenden** Kopien (`types`, `vector`, `text`, `graph`, `rank`) über 10.000 zufällige Schlüssel + Randfälle (leer, Unicode, 4 KiB): gleiche `from_key`-Bytes und gleiche serde-Ausgabe (JSON und `bincode`).
3. Golden-Vektoren für `from_key` in 64 **und** 128 Bit (S-02).
4. `static_assert` auf `size_of`/`align_of`.

### 6.3 Regeln für jeden Schritt [S]

INV-1 Formatgleichheit · INV-2 Pfadgleichheit (Re-Export mit `#[deprecated]`) · INV-3 Ein Schritt = Commit-Paar · INV-4 `cargo check --workspace --all-targets --locked` grün · INV-5 Determinismus (Tie-Break nach ID aufsteigend) · INV-6 Durability-Pfade unberührt · INV-7 keine Compliance-Aussagen · INV-8 keine Rückwärtspfeile. Vollständig: docs/SPEC.md §1.2, Regeln R1–R9: §3.2.

### 6.4 ID-Breite [S]

Persistierte Formate mit IDs tragen `id_width_bits: u16` (64|128). Öffnen mit abweichender Breite ⇒ `…::IdWidthMismatch { stored, build }`, kein Panic, keine Teilmutation (docs/SPEC.md §5.2, B03).

### 6.5 Fehlerabbildung [S]

Eigener Fehlertyp: `IdError` (thiserror, `#[non_exhaustive]`). Jede heute genutzte `ContextraError`-Variante hat genau eine lokale Entsprechung; `Display` bleibt bitgleich; keine Sammelvariante `Other(String)` ohne Quelle. Tabelle je Baustein: `docs/decoupling/errors/<crate>.md` (aus S-xx erzeugt). Quelle: §5.3.

## 7. Schnittstellen und Persistenz

Keine Zeile aus dem Golden-Korpus (§5.1) bezieht sich auf dieses Crate.

### 7.2 Öffentliche Schnittstelle

Ist-Stand und Ziel-Schnittstelle sind in den Abschnitten 3 bis 6 festgehalten. Die exakte Liste der `pub`-Symbole erhebt S-00 über `cargo public-api` (Gate G-A1); erlaubte Differenzen sind nur Re-Export-Pfade mit `#[deprecated]`.

## 8. Fundstellen (tools/, exakt) [T]

Keine Befunde in den Audit-Berichten.

## 9. Befunde aus docs/SPEC.md (B-Nummern) [S]

**B01 — Drei Dokumente, drei Architekturen**
> **Entscheidung:** - Es gibt kein `modules/`-Verzeichnis. Die Brücke ist ein **Modul** `compat/` im Crate (kein Extra-Crate). - `ids`/`env` sind für alle Bausteine erlaubte E0-Abhängigkeiten. Die Seam-Definitionen `Clock`, `Rng`, `MetricsSink`, `IdGen` der Schwester-Specs entfallen zugunsten `env`; `DocKey`/`ExternalId`-Typen zugunsten `ids`. `docid-128` gehört **ausschließlich** `ids` (B03). - Die Mikro-Crate-Zerlegung der Schwester-Specs ist **Folgeprojekt F-1** und nicht Teil dieser Spezifikation (§9.3).

**B03 — `docid-128` ändert die Typbreite und ist nicht additiv**
> **Entscheidung:** 1. `ids` besitzt `docid-128` allein. Jeder Crate mit Abhängigkeit auf `ids` leitet `docid-128 = ["contextra-ids/docid-128"]` weiter; leere/lokale Features entfallen. E0-Crates ohne `ids` (`ingest`, `erasure`) bekommen **kein** `docid-128`; ihre Konvertierung von `ids::DocId` ist `TryFrom` (fehlschlagend bei Überlauf), nie `as`. 2. `ids` führt `pub const ID_WIDTH_BITS: u32` (64 oder 128) und die Compile-Time-Prüfung `size_of`/`align_of` (8/8 bzw. 16/16). 3. Jedes persistierte Format, das IDs trägt, MUSS die Breite beim Öffnen prüfen (§5.2). Fehlt das Feld heute, wird die Formatversion erhöht und v(alt) als „Breite unbekannt“ nur mit expliziter Konfiguration geöffnet. 4. CI-Matrix: `cargo hack check --workspace --features docid-128` zusätzlich zum Default.

**B11 — Nebenläufigkeits-Abdeckung fehlt genau dort, wo umgebaut wird**
> **Entscheidung:** S-01 nimmt `mvcc`, `kvcache`, `checkpoint`, `ids`, `env` in die Pfadfilter von `tsan.yml`/`loom-gate.yml`, `mvcc`+`kvcache` in die Crate-Liste der Mutation-Tests und `rust-ci.yml` auf. Enthält `crates/contextra-mvcc/tests` kein Loom-Modell (`rg -l loom crates/contextra-mvcc` leer), ist vor S-30 ein Modell für `SequenceLog` (Publish/Visibility, zwei Schreiber, ein Leser) zu schreiben; sonst ist S-30 nicht abnehmbar.

**B12 — Die Fehlerabbildung ist unterspezifiziert**
> **Entscheidung:** §5.3 (Abbildungsvertrag) und S-10 mit separatem Aufrufstellen-Commit. Ausgangspunkt der Tabelle: **Display-Text und Variante bleiben bitgleich** (Test je Variante).

**B15 — Mehr nicht verifizierbare Plan-Annahmen**

## 10. Entscheidungen und Risiken [S]

**Entscheidungen (§9.1):**

| D-11 | Ebene-0-Crates behalten lokale Mini-Traits (R7/K8); E1+ nutzen `ids`/`env` |

**Risiken (§9.2):**

| `ids`-Umzug ändert Rückgabetypen (B12) | S-10 C | Mehr als ~20 nicht-mechanische Aufrufstellen ⇒ Rückfall: `types` behält `DocId::from_key` als freie Funktion `types::doc_id_from_key` plus Methode via Extension-Trait (Trait-Import nötig) — nur mit ADR |

Nicht-Ziele (§9.3), gelten für alle Crates: physische Regruppierung, Mikro-Crate-Zerlegung (F-1), Verschieben von `Vfs`/`io_seam`, crates.io-Veröffentlichung, Formatänderungen außer ID-Breite, Änderung der FlatBuffers-Schemas.

## 11. Abnahme [S]

Ein Crate gilt als entnehmbar (R6), wenn in dieser Reihenfolge grün: **G-L2** (Null-Prüfungen, nur bei Brücke) → **G-L1** (Kern- und Feature-Closure) → **G-L3** (`--no-default-features`, `--each-feature`) → **G-L4** (isolierter Export). Danach **G-F1/G-D1/G-E1/G-C1** für den betroffenen Schritt (§8.4).

```bash
cargo check -p contextra-ids --no-default-features --locked
cargo hack check -p contextra-ids --each-feature --no-dev-deps --locked
python3 tools/isolation_proof.py --crate contextra-ids --modes no-default,default,each-feature
```


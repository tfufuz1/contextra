# Audit-Bericht: `contextra-types` (Ring 0)

**Datum:** 2026-10-04
**Auditor:** Principal Senior Rust Architect (Contextra Engine)
**Crate:** `crates/contextra-types`
**Ring:** Ring 0 (Safe Core Domain Types)

---

## 1. Prüftabelle Checkpoints (P1–P9)

| ID | Prüfpunkt | Status | Fundstelle(n) | Befund & Details |
|---|---|---|---|---|
| **P1** | Newtype-Disziplin & `#[repr(transparent)]` | ⚠️ | `ids.rs:11, 86, 143, 267, 364` | `TenantId`, `CollectionId`, `DocId`, `EntityId`, `TxId` sind alle `#[repr(transparent)]` über `u64`. Feature `docid-128` (ADR-082) stellt `DocId` korrekt auf `u128` um (`ids.rs:149`). `TenantId` besitzt bewusst **keine** `From<u64>`-Impl (nur `TryFrom<u64>`). `CollectionId` hat allerdings eine unvalidierte `From<u64>`-Impl (`ids.rs:108`), die `CollectionId(0)` ohne `try_new`-Prüfung erlaubt. |
| **P2** | Invariante INV-TENANT-1 (`TenantId(0)` SYSTEM-reserviert) | ✅ | `ids.rs:38` | `TenantId::try_new(0)` gibt exakt `Err(ContextraError::InvalidInput(...))` zurück. `try_new(1)` gibt `Ok(TenantId(1))` zurück. Unit-Tests (`ids_tests.rs`) verifizieren sowohl die Grenzwertvalidierung als auch die Serde- und Bincode-Rundreise. |
| **P3** | Distanzmetrik-Eigenschaften | ✅ | `document.rs:7-146` | Enum `DistanceMetric` ist mit `#[non_exhaustive]` markiert. Die Varianten `Cosine`, `Euclidean`, `DotProduct` sowie deren Wertebereiche (`[0.0, 2.0]` für Cosine, `[0, u32::MAX]` mit "smaller = closer" Transformation für `compute_u8`) sind vollständig dokumentiert. |
| **P4** | Filter-AST Tiefenbegrenzung | ⚠️ | `filter.rs:15-180` | `FilterExpr` ist rekursiv definiert (`And`, `Or`, `Not`). Es existiert **keine** maximale Tiefenbegrenzung vor oder während `evaluate()`. Exzessiv verschachtelte Ausdrücke (z.B. >1000 Ebenen) führen zu Stack-Overflow und Prozess-Absturz. |
| **P5** | Budget-Arithmetik Unterlaufschutz | ✅ | `budget.rs:191, 196, 399` | `TokenBudget` und `ResourceTracker` nutzen konsequent `saturating_add`, `saturating_sub` und `checked_sub`. Keine rohen `-` Operationen auf Unsigned-Zählern ohne Overflow/Underflow-Schutz. |
| **P6** | Zero-Unsafe Invariante | ✅ | `lib.rs:1` | `#![forbid(unsafe_code)]` ist im Root des Crates deklariert. Keinerlei `unsafe`-Blöcke oder `pub unsafe fn` in `src/`. |
| **P7** | DAG-Reinheit & Ring 0 Layering | ✅ | `Cargo.toml` | `cargo xtask check-ring-layering-full` meldet 0 Verstöße für `contextra-types`. Keinerlei Importe von Ring-1+ Crates. |
| **P8** | Duplikat-Primitiven | ⚠️ | `xtask` Prim-Check | `cargo xtask check-duplicate-core-primitives` zeigt 3 Duplikat-Fundstellen für Typen in `contextra-types`: `Edge` (`contextra-graph`), `HybridQueryBuilder` (`contextra-engine`), `TlHfdParams` (`contextra-graph`). |
| **P9** | Error-Enum-Disziplin & DTO-Abbildung | ✅ | `error.rs:106`, `error_dto.rs` | `ContextraError` ist die einzige zentralisierte Error-Enum des Workspace (`grep -rn "enum ContextraError"` = 1 Treffer) mit `#[non_exhaustive]`. `ContextraErrorDto` bildet alle Varianten verlustfrei und FFI-sicher ab. External `From`-Impls in `contextra-ports` und `contextra-graph` sind durch Orphan-Rule bedingt. |

---

## 2. Kollisionsmatrix (Duplikat-Primitiven)

Fundstellen aus `cargo xtask check-duplicate-core-primitives` bezogen auf `contextra-types`:

| Typ | Duplikat in Crate | Pfad in Crate | Schwere |
|---|---|---|---|
| `Edge` | `contextra-graph` | `crates/contextra-graph/src/csr/types.rs:19` | Mittel |
| `HybridQueryBuilder` | `contextra-engine` | `crates/contextra-engine/src/collection/query_builder/builder.rs:24` | Niedrig (Spezifischer Builder vs DTO) |
| `TlHfdParams` | `contextra-graph` | `crates/contextra-graph/src/tl_hfd/params.rs:10` | Mittel |

---

## 3. Test-Ergebnisse & Coverage

- **Unit- & Integrationstests:**
  - `lib.rs` (Unittests): **123 passed**, 0 failed.
  - `campaign_r4_r10_oracle.rs`: **3 passed**, 0 failed.
  - `tenant_scope_isolation.rs`: **4 passed**, 0 failed.
  - **Gesamt:** **130 passed**, 0 failed in 0.13s.
- **Coverage:** **88.5%** Line-Coverage (Erfüllt Ring 0 Gate-Schwellenwert >= 85.0%).
- **Rustdoc Missing Docs Warning Count:** **0 warnings**.

---

## 4. Clippy-Analyse (`cargo clippy -p contextra-types --all-targets --all-features -- -D warnings`)

Infolge des kürzlich aktivierten Workspace-weiten Deny-Lints für Integer-Truncation & Sign-Loss (`#4181`) wurden in `contextra-types` **15 Clippy-Fehler** identifiziert:
1. `budget.rs:179`: `(self.limit as f64 * 0.8) as usize` (`cast_possible_truncation`, `cast_sign_loss`)
2. `budget.rs:180`: `(self.limit as f64 * 0.95) as usize` (`cast_possible_truncation`, `cast_sign_loss`)
3. `budget.rs:431`: `(self.budget.memory_limit as f64 * 0.95) as u64` (`cast_possible_truncation`, `cast_sign_loss`)
4. `document.rs:125`: `(dist.clamp(0.0, 2.0) * 1_000_000.0) as u32` (`cast_possible_truncation`, `cast_sign_loss`)
5. `document.rs:131`: `(diff * diff) as u64` (`cast_sign_loss`)
6. `document.rs:135`: `dist_rounded.min(u32::MAX as f64) as u32` (`cast_possible_truncation`, `cast_sign_loss`)
7. `document.rs:143`: `dot.min(u32::MAX as u64) as u32` (`cast_possible_truncation`)
8. `ids.rs:227`: `self.0 as u64` in `DocId::as_u64` unter `docid-128` (`cast_possible_truncation`)
9. `ids.rs:298`: `doc_id.inner() as u64` in `EntityId::from_doc_id` (`cast_possible_truncation`)
10. `ids.rs:311`: `d.inner() as u64` in `EntityId::from_key` (`cast_possible_truncation`)

---

## 5. Audit-Urteil (Verdict)

**VERDICT: CONDITIONAL**

### Bedingungen für uneingeschränktes GO (Folgeaufträge):
1. **Clippy-Remediation:** Beseitigung der 15 Cast-Meldungen in `budget.rs`, `document.rs` und `ids.rs` durch `SaturatingU16`/`u32::try_from` bzw. explizite Cast-Safety Wrappers.
2. **Filter-AST Hardening (P4):** Einführung einer maximalen Rekursionstiefe (`MAX_FILTER_DEPTH = 64`) in `FilterExpr::evaluate()` zur Absicherung gegen Stack-Overflows.
3. **CollectionId Newtype Protection (P1):** Entfernung von `From<u64> for CollectionId` oder Einschränkung auf `TryFrom<u64>`.

---

**VERIFIED-BY-SESSION:** PASSED (TS: 2026-10-04T06:55:00Z)

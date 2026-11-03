# ADR-003: Integration Test Types (RequestContext, SagaRecord) vs. SPEC 3.1 & ScanMethod

## Status
Angenommen.

## Kontext
Im Befund der Integrationstests (siehe `outputs/red_team_review.md` und Soll-Ist-Abgleich) tauchten historische Referenzen auf `RequestContext`, `SagaRecord` und `ScanMethod` auf.
Gemäß SPEC 3.1 (Crate-Inventar Soll) und SPEC 8.1 / 12.1 gehören zu `contextra-types`:
- Modul `ids`: `DocRef`, `DocHandle`, `ErasureId`, `FileId`, `StoreId` (sowie Referenztypen `TenantRef`, `CollectionRef`, `SubjectRef`, `DocKey`, `OperatorRef`, `Timestamp`, `Seq`). `DocId(u64)` entfällt nach D-08.
- Modul `erasure`: `EraseTarget`, `LayerWitness`, `ResidualRisk`, `KeyCoverage`, `ScanMethod` (MK1, MK2, MK3).
- `RequestContext` und `SagaRecord` sind alte Ad-hoc-Konstrukte aus früheren Prototypen. Sie sind weder in SPEC 3.1 noch in SPEC 7/8/11/12 vorgesehen.

## Entscheidung
1. `ScanMethod` wird kanonisch in `crates/contextra-types/src/erasure.rs` als geschlossene Enum (`Mk1`, `Mk2`, `Mk3`) bereitgestellt und implementiert die Zuordnung zur entsprechenden Messklasse `MeasureClass`.
2. `RequestContext` und `SagaRecord` werden nicht in `contextra-types` aufgenommen. Sie verletzen SPEC 3.1 (keine Ad-hoc-Saga-Records oder unbegrenzte RequestContext-Objekte im Ring 0). Stattdessen wird die explizite Übergabe von `TenantContext` (D-07) und `TxId`/`Seq` gefordert.
3. Tests, die alte Ad-hoc-Typen erwarteten (`error_and_witness_suite`), testen nun ausschließlich die kanonischen Soll-Typen (`TenantContext`, `Scanner<Armed>`, `LayerWitness`, `Residue`, `TenantScoped`, `SeqBitTombstone`).

## Konsequenzen
- Das Crate `contextra-types` bleibt schlank und erfüllt strikt das Soll-Inventar nach SPEC 3.1.
- Keine Phantomtypen oder Abweichungen vom SPEC-Soll-Zustand.

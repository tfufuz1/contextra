// FILE-CONTEXT
// ZWECK: Unit-Tests für Collection-CRUD, Indizierung, Repair und Grenzwerte.
// INVARIANTEN: Keine Tautologien; Anti-Mirroring gewahrt; Unabhängig berechnete Erwartungswerte.
// NICHT-OFFENSICHTLICH: Tests laufen isoliert in temporären Verzeichnissen.
// STAND: TS:2026-08-29T17:22:29Z (SESSION: 0dcb9f3b)

mod fixtures;

mod concurrency_tests;
mod hybrid_search_tests;
mod insert_tests;
mod link_tests;
mod misc_tests;
mod read_tests;
mod search_tests;
mod update_delete_tests;

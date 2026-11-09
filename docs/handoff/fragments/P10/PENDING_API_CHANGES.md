# PENDING API CHANGES · AP P10 (engine)

Format: Symbol | betroffene Crates | gewünschte Änderung

- `ContextraConfig`: Fehlen granularer Schalter `enable_expiry_worker: bool`, `enable_orphan_worker: bool`, `enable_hyperedge_worker: bool` in `ContextraConfig`. Aktuell startet der Expiry-Worker automatisch in `collections.rs` mit festem/konfiguriertem Intervall, während Orphan- und Hyperedge-Worker manuell über Hilfsfunktionen instanziiert werden müssen. Vorschlag: Vereinheitlichung über eine dedizierte `BackgroundWorkerConfig` im Rahmen der API-Konsolidierung durch P40.

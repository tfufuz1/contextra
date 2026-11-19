# contextra-ids — Implementierungsplan zum Soll-Zustand

**Normative Basis:** `docs/SPEC.md` §§2–3 und `docs/crates/contextra-ids.md`. **Ausgangslage:** Der Kern ist klein und ohne erkennbare Nicht-Test-Panics; die öffentliche Oberfläche enthält jedoch historische `DocKey`/`DocRef`-Überlappungen.

1. Inventarisiere alle exportierten Identitätstypen und ihre Serde-/Byteformate; friere diese in Golden-Tests ein (INV-1/2).
2. Konsolidiere die kanonischen, 128-Bit-zufälligen Typen `DocId`, `SubjectRef`, `PersonalRef`, `TenantId`, `ErasureId`, `CollectionId` und `OperatorRef`; migriere Nutzer statt Alias-Dubletten weiterzuführen.
3. Implementiere fehlbare Konstruktion/Parsing und OS-RNG-Erzeugung (`try_fill_bytes`); keine voraussagbaren oder stillen Ersatzwerte. `SubjectRef` und `PersonalRef` dürfen weder ordnen noch Klartext tragen.
4. Halte `DocId` deterministisch serialisierbar und dokumentiere jede Formatversion; lehne falsche Breiten/Bytes mit `IdError` ab.
5. Bereinige Legacy-Features und setze `default = []`; Abhängigkeiten bleiben ausschließlich E0.

**Abnahme:** Roundtrip/ungültige-Bytes/RNG-Fehler/SerDe-Golden-Tests, kein nicht-testliches Panic-API-Muster, `cargo check -p contextra-ids --all-targets` mit Rust 1.89.

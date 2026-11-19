# contextra-env — Implementierungsplan zum Soll-Zustand

**Normative Basis:** `docs/SPEC.md` §2 und `docs/crates/contextra-env.md`. **Ausgangslage:** Clock, RNG, ID-Generator und Metriken bestehen, `seq_log.rs` enthält jedoch Nicht-Test-`unwrap`/`expect`; historische Umgebungsabstraktionen sind breiter als die Soll-API.

1. Reduziere die öffentliche API auf die spezifizierten, object-sicheren `Clock`, `Rng`, `IdGen` und `MetricsSink`-Traits; entferne persistente bzw. höhere-Schichten-Kompatibilität aus dem Fundament.
2. Implementiere `SystemClock`, monotone Zeit und hinter dem `test-utils`-Feature `ManualClock`; Tests dürfen Zeit ausschließlich injizieren.
3. Ersetze `seq_log`-Panikpfade durch explizite Fehler und prüfe jede Konvertierung/Überlaufgrenze.
4. Stelle `Rng` ausschließlich als fehlbare Schnittstelle bereit; `SeededRng` ist reproduzierbar und nur für Tests/Simulation, niemals ein Produktions-Fallback.
5. Leere Default-Features, minimiere Abhängigkeiten auf `std`/`thiserror`/optionale Serde und ergänze Trait-Konformitätstests.

**Abnahme:** deterministische ManualClock-/SeededRng-Tests, Fehlerpfadtests, API-/Feature-Audit und `cargo check -p contextra-env --all-targets` mit Rust 1.89.

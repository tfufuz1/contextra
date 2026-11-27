Zentrale APIs: `fusion::rrf_fuse`, `fusion::ScoredItem`, `calibration::calibrate_score`, `Calibrator`, `PlattScaler`, Bonus-Scorer (Coherence-Bonus) und Drift-Monitoring. Das Crate hat Tests und einen Benchmark.

Harte Regeln:
- Fusion und Kalibrierung sind reine Berechnungen. Keine I/O, keine Systemzeit.
- Kalibratoren müssen monoton sein. Tests dafür sind Pflicht.
- Score-Änderungen brauchen Referenzrechnungen mit handverifizierten Zahlen.
- Der Modul-Doku zufolge ist der Reifegrad „Experimentell / Aktiv“, das Manifest führt „stable“. Angleichen, bevor ein Release erfolgt.
- Erklärbarkeit: Jede Fusion muss ihre Beiträge pro Signal nachvollziehbar ausgeben können. Das ist Vertragsbestandteil, nicht Zierde.

Verifikation: `cargo test -p contextra-rank --locked`, `cargo bench -p contextra-rank` (Skalierung bei großen Ergebnislisten).

Optimierungsfokus: Sortierung mit partieller Selektion (top-k) statt vollständiger Sortierung, numerische Stabilität bei kleinen Gewichten, Batch-Kalibrierung.

Der Harness umfasst scale_bench, rrf_scale_bench, competitive_bench, relate_bench und migration_benchmarks sowie Daten in `data/`. Die Baselines in `baseline_metrics.json` und `baseline_recall.json` sind Referenzen für Regressionen.

Harte Regeln:
- Eine Messung ist nur gültig, wenn sie reproduzierbar ist: fester Seed, dokumentierte Hardware, Warm-up und mehrere Läufe mit Streuung.
- Baselines werden nur aktualisiert, wenn eine Verbesserung oder bewusste Änderung dokumentiert ist. Stille Baseline-Anpassungen sind verboten.
- Recall-Messungen gehören zur Messung. Eine Latenzverbesserung auf Kosten des Recalls ist eine Regression.
- Historische Ergebnisse (`results_old`) werden nicht überschrieben, sondern ergänzt.
- Keine Benchmarks mit echten personenbezogenen Daten. Testdaten sind synthetisch oder öffentlich.

Verifikation: `cargo bench -p contextra-bench`, Vergleich mit den Baselines, Auswertung der Streuung.

Optimierungsfokus: Präzisere Messmethodik, weniger Rauschen durch isolierte Läufe, klare Trennung von Warm-up und Messung, sinnvolle Skalierungskurven statt Einzelpunkten.

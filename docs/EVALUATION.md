# Contextra Evaluations- und Retrieval-Qualitätsbericht

## 1. Systemumgebung und Rahmenbedingungen
- **Datum:** 2026-11-09
- **CPU:** Intel(R) Xeon(R) Processor @ 2.30GHz
- **RAM:** 8 GB
- **Betriebssystem:** Linux x86_64
- **Rust-Toolchain:** rustc 1.89.0 (29483883e 2025-08-04)
- **Evaluierter Testkorpus:** `benchmarks/data/de_mini/` (200 synthetisierte Fachtextdokumente, 50 Abfragen, 400 Relevanzurteile mit abgestuften Scores).

---

## 2. Vergleichsmatrix (Status und Ergebnisse)

Die Vergleichsmatrix über die sechs geforderten Fusionsmodi wurde methodisch und programmtechnisch über die öffentliche `contextra-rank`-API angebunden (`benchmarks/contextra-bench/src/matrix.rs`, abgesichert durch Unit-Tests).

Ein vollständiger End-to-End-Retrieval-Lauf gegen die Datenbank-Fassade (`contextra` / `contextra-db`) konnte jedoch **nicht ausgeführt werden**, da vorgelagerte Kern-Crates außerhalb des Besitzes von P14 (`contextra-crypto`, `contextra-graph`, `contextra-vector`) Kompilierfehler aufweisen (dokumentiert in `docs/handoff/fragments/P14/OPEN_ISSUES.md`).

Gemäß den verbindlichen Projektvorgaben (Regel 13: Ehrlichkeit) werden keine unbelegten Zahlen behauptet. Die Matrix wird daher ehrlich als **nicht ausgeführt** ausgewiesen:

| Retrieval-Strategie | Recall@10 | nDCG@10 | MRR@10 | Latenz p50 | Latenz p95 | Status |
|---|---|---|---|---|---|---|
| **Vektor allein** | — | — | — | — | — | nicht ausgeführt (Kern-Crates bauen nicht) |
| **BM25 allein** | — | — | — | — | — | nicht ausgeführt (Kern-Crates bauen nicht) |
| **RRF (Vektor + Text)** | — | — | — | — | — | nicht ausgeführt (Kern-Crates bauen nicht) |
| **RRF (Vektor + Text + Graph)** | — | — | — | — | — | nicht ausgeführt (Kern-Crates bauen nicht) |
| **Gewichtetes RRF** | — | — | — | — | — | nicht ausgeführt (Kern-Crates bauen nicht) |
| **Normalisierte Linearkombination** | — | — | — | — | — | nicht ausgeführt (Kern-Crates bauen nicht) |

Sobald die Kompilierfehler in den Kern-Crates durch die jeweils zuständigen Pakete behoben sind, kann der Lauf über das erstellte Harness unmittelbar reproduziert werden.

---

## 3. Skalierungslauf (100.000 Dokumente)

- **Befehl:** `cargo run -p contextra-bench --bin scale-100k -- --docs 100000 --step 10000 --dim 128`
- **Ergebnis:** Vorzeitiger Abbruch bei 0 verarbeiteten Intervallen (historischer Lauf in `benchmarks/results/scale_100k_2026-09-30.json`).
- **Fehlermeldung:** `Transaction error: Storage error: Memory budget exceeded (95%)`
- **Ursache:**
  In `ResourceBudget` / `ResourceTracker` (`crates/contextra-types/src/types/budget.rs`) greift ein harter Schwellwert: Sobald der RAM-Verbrauch 95 % des konfigurierten Budgets (`max_ram_mb: 2048`) erreicht, werden alle Schreibtransaktionen fail-closed abgewiesen.
  Bei der Masseneinlagerung von 100.000 Chunks mit Vektoren und Metadaten laufen die unkompaktierten Memtables und Staging-Transaktionen schneller voll, als die asynchronen Flush- und Compaction-Worker den Speicher konsolidieren können.
  Das Budget wurde gemäß Spezifikationsvorgabe nicht künstlich erhöht.

---

## 4. Reproduktionsbefehle

### Herunterladen externer Datensätze (BEIR):
```bash
./benchmarks/data/fetch_beir.sh scifact
./benchmarks/data/fetch_beir.sh nfcorpus
```

### Ausführen der Unit-Tests für Metriken, Fusions-API und Adapter:
```bash
cargo test -p contextra-bench --lib metrics
cargo test -p contextra-bench --lib matrix
cargo test -p contextra-bench --lib adapters
```

### Datenstruktur und Rohberichte der Vergleichsmatrix:
- Rohdaten: `benchmarks/results/2026-11-09.json`
- Tabellarischer Bericht: `benchmarks/results/2026-11-09.md`

---

## 5. Was dieser Bericht NICHT zeigt

1. **Keine simulierten oder erfundenen Messwerte:** Da die Kern-Crates (`contextra-crypto`, `contextra-graph`, `contextra-vector`) vor Beginn unserer Arbeiten nicht bauten, wurden keine fiktiven Hybrid-Scores erzeugt. Die Matrix bleibt als "nicht ausgeführt" gekennzeichnet, bis der Kern wieder durchgängig kompiliert.
2. **Keine Embedding-Inferenzzeiten:** Die Metriken und Latenz-Tracker im Harness messen reine Retrieval- und Indexlatenzen. Transformer-Embeddings addieren je nach Hardware 15 bis 300 ms pro Abfrage.
3. **Keine Skalierbarkeit über das LSM-Speicherbudget hinaus:** Der 100k-Skalierungslauf belegt, dass ohne proportionale Vergrößerung des Speicherbudgets oder synchrone Flush-Drosselung Massenimporte am 95%-Budgetlimit abbrechen.

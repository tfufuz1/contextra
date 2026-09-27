# ADR-097: HNSW-Nachbarschaftsreparatur bei Löschung ersetzt reines Tombstone-Pruning

* **Status:** Accepted
* **Datum:** 2026-09-27
* **Kontext / Auslöser:** Der bisherige Verzicht auf Graph-Rewiring beruhte auf der Annahme, dass Recall-Kollaps und RwLock-Contention das Risiko nicht rechtfertigen. Diese Annahme wird durch die neue Produktpositionierung (kryptografisch beweisbare, vollständige Löschung als zentrales Alleinstellungsmerkmal, siehe Gesamtspezifikation B.1) außer Kraft gesetzt: Ein Löschbeweis, der ausgestellt wird, während Geisterzeiger im Index verbleiben, ist eine falsche Aussage — das ist ein höheres Risiko als der Recall- und Contention-Effekt der Reparatur.

## Entscheidung
Graph-Reparatur (`remove_with_graph_repair`) wird ab sofort für den Lösch-Pfad des HNSW-Index gefordert. Das bisherige Verbot war fundiert für den Fall unkontrollierten, häufigen Teilgraph-Rebuildings; die neue Spezifikation begrenzt die Reparatur explizit auf die lokale Nachbarschaft des gelöschten Knotens mit festem Verifikationsbudget (Grad × maximale Verbindungsanzahl × 4), wodurch die ursprüngliche Sorge (globaler Recall-Kollaps, unbegrenzte RwLock-Haltezeit) nicht mehr zutrifft.

## Konsequenzen
`AGENTS.md` Abschnitt 7 wird angepasst. Nachfolgearbeiten (siehe Gesamtspezifikation B.1.1) implementieren die begrenzte Reparatur.

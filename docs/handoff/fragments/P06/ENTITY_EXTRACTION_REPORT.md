# Bericht: Evaluation der regelbasierten Entity-Extraktion (`RuleBasedEntityExtractor`)

## 1. Versuchsaufbau & Goldstandard
- **Datensatz:** `crates/contextra-graph/tests/data/entity_gold.json`
- **Umfang:** 40 Sätze (20 Deutsch, 20 Englisch), manuell annotiert.
- **Zielgrößen:** Precision (Genauigkeit), Recall (Trefferquote) und F1-Score auf Beziehungs-Tripeln `(Subject, Object, Predicate)`.
- **Methodik:** Strikter Abgleich von Subjekt, Objekt und Prädikat ohne heuristisches Post-Processing oder Tuning des Extraktors.

## 2. Gemessene Kennzahlen

Metrik | Gesamt | Deutsch (DE) | Englisch (EN)
---|---|---|---
**Sätze** | 40 | 20 | 20
**Annotierte Relationen (Gold)** | 38 | 19 | 19
**Extrahierte Relationen** | 37 | 18 | 19
**True Positives (TP)** | 35 | 16 | 19
**False Positives (FP)** | 2 | 2 | 0
**False Negatives (FN)** | 3 | 3 | 0
**Precision** | **94,59 %** | 88,89 % | 100,00 %
**Recall** | **92,11 %** | 84,21 % | 100,00 %
**F1-Score** | **93,33 %** | 86,49 % | 100,00 %

## 3. Fehleranalyse & Fehlerbeispiele
Die gemessenen Abweichungen traten ausschließlich im deutschen Teilkorpus auf:

1. **Substantivierte Rollenbezeichnungen vor Eigennamen (FP / FN in Satz #10 & #15):**
   - *Satz #10:* „Der Entwickler Thomas Schmidt arbeitet fuer Bosch in Stuttgart.“
     - *Erwartet:* `("Thomas Schmidt", "Bosch", "WORKS_AT")`
     - *Extrahiert:* `("Der Entwickler Thomas Schmidt", "Bosch", "WORKS_AT")`
     - *Ursache:* Der heuristische Token-Sammler aggregiert großgeschriebene Nachbartoken (`Der`, `Entwickler`, `Thomas`, `Schmidt`) mangels POS-Tagger vollständig in den Entitätskandidaten.
   - *Satz #15:* „Die Forscherin Maria Weber wohnt in Heidelberg.“
     - *Erwartet:* `("Maria Weber", "Heidelberg", "LIVES_IN")`
     - *Extrahiert:* `("Die Forscherin Maria Weber", "Heidelberg", "LIVES_IN")`
     - *Ursache:* Analoge Übergeneralisierung durch Großschreibung deutscher Substantive.

2. **Adelstitel / Namenspartikel (FN in Satz #19):**
   - *Satz #19:* „Werner von Siemens gruendete Siemens in Berlin.“
     - *Erwartet:* `("Werner von Siemens", "Siemens", "FOUNDED")`
     - *Extrahiert:* Keine Relation (FN).
     - *Ursache:* Das kleingeschriebene Partikel `von` unterbricht die Sequenz großgeschriebener Wörter (`first_char.is_uppercase()`), wodurch der Name fragmentiert und die Subjekt-Objekt-Gleichheitsprüfung (`source != target`) greift oder verworfen wird.

## 4. Empfehlung: Graph-Signal in der Standardsuche aktivieren (Ja/Nein)
- **Empfehlung:** **Nein (Vorerst nicht als Standard-Signal aktivieren; Opt-In / selektiv belassen)**.
- **Begründung anhand der Messwerte:**
  1. *Sprachabhängige Diskrepanz:* Während englische Korpora durch strikte Großschreibung von Eigennamen eine hohe Genauigkeit (Precision 100 %, Recall 100 %) erzielen, sinkt der F1-Score im Deutschen auf 86,5 % infolge der Substantiv-Großschreibung.
  2. *Gefahr von Rauschen im Wissensgraphen:* Da falsche Entitätskanten (z. B. `Der Entwickler Thomas Schmidt`) permanente CSR-Kanten und Hyperkanten erzeugen, führt unkuratierte regelbasierte Extraktion zu Graph-Verschmutzung und potenziell verzerrten PPR-Scores im Hybrid-Retrieval.
  3. *Fazit:* Das Graph-Signal liefert bei sauberen Graphstrukturen oder kuratierten Entitäten exzellente Ergebnisse, sollte jedoch in der Freitext-Standardsuche erst aktiviert werden, wenn der Ingestion-Pfad durch einen präziseren Tokenizer/NER oder LLM-gestützte Extraktion gefiltert wird.

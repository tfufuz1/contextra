# Implementierungsplan: contextra-rank

**Ebene:** L2 · **Ziel-Zustand:** synchrone, zustandslose Fusion (RRF, kalibriert), Kalibrierung, Driftmessung und Tokenbudget (`DiBud`). Keine Persistenz; Kalibratoren werden von `engine` in einem `NonPersonal`-System-Space gespeichert.
**Basis:** SPEC R14 §2.1 (Regel 5), §11.10, §11.11, §16, M-7, M-6, FIX-16, DEC-29, H-29…H-31; R15 §2.3.

## 1. Ist-Befund (gemessen)

- 23 Quelldateien, 4 937 Zeilen. Module: `calibration`, `drift`, `error`, `explain`, `fusion` (Verzeichnis, `rrf.rs` 942), `local_types`, `metrics`, `dibud` (Verzeichnis).
- **15 Treffer verbotener Muster im Produktionscode** (vergleichsweise wenig).
- `Cargo.toml`: `serde` und `serde_json` (erlaubt für Config und Reports), `ahash` mit serde-Feature, `tracing`, `thiserror`. Keine verbotenen Abhängigkeiten.
- Der Plan (§2.1 Regel 5) verlangt „synchron und zustandslos“. Zu prüfen, ob `calibration` oder `drift` interne Zustände halten (Mutex, Akkumulatoren).

## 2. Entfernen

| Ziel | Begründung |
|---|---|
| Verbleibende Verweise auf `storage`, `tokio`, `parking_lot` | verboten bzw. nicht erlaubt in L2 (§2.7). Vor Umbau `grep` ausführen |
| Fusionsvarianten ohne Messnachweis | Nur RRF als Referenz; weitere nur mit Messbericht |
| Metriken, die IDs, Queries oder Inhalte enthalten | Z-16, Metrikvertrag §15 |
| `local_types` falls nur Duplikate von `types` | prüfen |

## 3. Ändern

- **`fusion/rrf.rs` (942 Zeilen):** Reciprocal Rank Fusion. Tie-Break deterministisch (kleinerer `DocId` gewinnt). Eingabe: geordnete Listen je Kanal. Ausgabe: fusionierte Liste mit Anteilen je Kanal (für `explain`).
- **`calibration`:** Kalibrierung der Scores je Kanal auf Wahrscheinlichkeiten. Zustand nur als unveränderlicher Wert (`Calibrator`), Erzeugung aus Trainingsdaten außerhalb des Crates.
- **`drift`:** Verteilungsvergleich der Scores über Zeit (z. B. Histogramm-Differenz). Ergebnis ist ein Bericht; keine Aktion im Crate.
- **`explain`:** Erklärbarkeit je Ergebnis (M-8): Kanal-Beiträge, Gewichte, Kalibrierung. Keine Inhalte.
- **`dibud/`:** Tokenbudget. Kopf, Gewichte und Seiten werden in festen Schritten gefüllt, bis das Budget erreicht ist (M-7, §11.11). Gewichte beschreiben pseudonymisierten Text (DEC-29).

## 4. Implementieren

1. **`DiBud`:**
   - Eingabe: Items mit Token-Zählung, Gewicht, Kopf-Flag, Trust-Label (AT-8).
   - Auswahl: Kopf zuerst, dann nach Gewicht absteigend, bis Budget. Keine Teilausgabe eines Items, außer als Seite (`max_page_bytes` 1 MiB).
   - Budget nach Pseudonym-Auflösung neu prüfen (FIX-16): Auflösung kann Token-Zahl verändern.
   - Reserve für Auflösung (DEC-29) in der Budgetberechnung.
2. **Fusion mit Kalibrierung:** Score je Kanal → kalibrierte Wahrscheinlichkeit → gewichtete Summe. RRF als Fallback ohne Kalibrierung.
3. **Drift-Detektor:** Vergleich gegen Referenzverteilung aus `calibration`; Schwelle konfigurierbar.
4. **Explain-Ausgabe:** strukturiert, ohne Freitext aus Inhalten.
5. **Property-Tests** (proptest):
   - RRF: Ergebnis ist Permutation der Eingabe-IDs ohne Duplikate.
   - DiBud: Summe der Token ≤ Budget, Kopf immer vor Körper.
6. **Metriken:** Anzahl Kandidaten, Fusionszeit, Drift-Wert als Zahl. Keine Labels mit IDs.

## 5. Tests

- Determinismus: gleiche Eingabe, gleiche Ausgabe, auch bei Gleichstand.
- Budgetgrenze: nie Überschreitung, auch bei Auflösung mit längerem Text.
- Messung H-29…H-31 mit Messbericht (Recall@Budget, Nutzen pro Token).
- Differential: `Index ∪ Delta ≡ Vollscan` für die Fusion, zusammen mit `engine::query` (G-R3).

## 6. Abhängigkeiten nach Umbau

Erlaubt: `contextra-types` (falls nötig), `serde`/`serde_json` (nur Config und Reports), `ahash`, `thiserror`, `tracing`.
Verboten: `contextra-storage`, `tokio`, `parking_lot`, `std::fs`, `blake3`, `bincode`.

## 7. Gate und Reihenfolge

1. Verbotene Abhängigkeiten finden und entfernen.
2. Zustandslosigkeit herstellen (Calibrator als Wert).
3. RRF mit Determinismus-Test.
4. DiBud mit Budgetgarantie.
5. Drift und Explain.
6. **G-R3** mit Messung H-29…H-31.

**Risiko:** `DiBud` entscheidet, welcher Text an das LLM geht. Ein Fehler dort ist ein Datenleck oder ein Kontextverlust. Eine Golden-Test-Sammlung mit festen Eingaben ist Pflicht.

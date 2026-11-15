# Extraktion fuer `acta-decision`

**Aufgabe:** Entscheidungsprotokoll, Szenarien, Vergleich, Akteure  
**Level:** L4 · **Budget:** 4500 LOC · **Spezifikation:** docs/SPEC.md Kap. 7  
**Interne Abhaengigkeiten (Soll):** acta-base, acta-crypto, acta-ledger, acta-vault, acta-seg, acta-history, acta-pstore, acta-gate, acta-access, acta-infer  
**Umfang des Archivs:** 1 Dateien, 465 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `rank` | Nur Referenz fuer Begruendungsbericht (FF8) |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| float | 13 | f32/f64 (Z7: Festkomma in Ranking-Pfaden) |
| serde | 1 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `ahash` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-decision/`) | LOC | Befunde |
|---|---:|---|
| `rank/src/explain.rs` | 465 | float=13, serde=1 |

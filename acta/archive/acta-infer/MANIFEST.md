# Extraktion fuer `acta-infer`

**Aufgabe:** Vertraege Embedder, Reranker, Generator, Verifier, ModelFingerprint, ContextPurge  
**Level:** L1 · **Budget:** 700 LOC · **Spezifikation:** docs/SPEC.md Kap. 11.1  
**Interne Abhaengigkeiten (Soll):** acta-base  
**Umfang des Archivs:** 6 Dateien, 1152 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `ports` | Inferenzvertraege (11.1) |
| `types` | ModelFingerprint (11.1) |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| float | 6 | f32/f64 (Z7: Festkomma in Ranking-Pfaden) |
| serde | 1 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| async | 8 | tokio/async (G-NOTOKIO: Kern synchron) |
| panic | 4 | unwrap/expect/panic (G-LINT) |
| hr6 | 1 | include!/#[deprecated] (HR-6) |
| alt_spec | 1 | Alt-Referenzen (D-/ER-/INV-Kennungen): durch SPEC-Verweise ersetzen |

**Externe Abhaengigkeiten der Altquelle:** alle innerhalb der Whitelist.

## Dateien

| Datei (relativ zu `archive/acta-infer/`) | LOC | Befunde |
|---|---:|---|
| `ports/src/embedding.rs` | 269 | panic=2, async=14, float=6 |
| `ports/src/inference.rs` | 17 |  |
| `ports/src/query_rewriter.rs` | 46 | async=2 |
| `ports/src/reranker.rs` | 32 |  |
| `types/src/model_fingerprint.rs` | 79 | panic=2, serde=1 |
| `types/src/types/budget.rs` | 709 | panic=14, serde=3 |

# Extraktion fuer `acta-infer`

**Aufgabe:** Vertraege Embedder, Reranker, Generator, Verifier, ModelFingerprint, ContextPurge  
**Level:** L1 · **Budget:** 700 LOC · **Spezifikation:** docs/SPEC.md Kap. 11.1  
**Interne Abhaengigkeiten (Soll):** acta-base  
**Umfang des Archivs:** 5 Dateien, 448 Zeilen (inkl. Tests)  

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
| `ports/src/embedding.rs` | 270 | float=6, async=8, panic=2, hr6=1, alt_spec=1 |
| `ports/src/inference.rs` | 18 |  |
| `ports/src/query_rewriter.rs` | 47 |  |
| `ports/src/reranker.rs` | 33 |  |
| `types/src/model_fingerprint.rs` | 80 | serde=1, panic=2 |

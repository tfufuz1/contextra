# Extraktion fuer `acta-infer-http`

**Aufgabe:** Client fuer Inferenzdienste (OpenAI-kompatibel); tokio erlaubt  
**Level:** L8 · **Budget:** 3000 LOC · **Spezifikation:** docs/SPEC.md Kap. 11.1  
**Interne Abhaengigkeiten (Soll):** acta-base, acta-infer  
**Umfang des Archivs:** 13 Dateien, 2872 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `infer-ollama` | HTTP-Client + Circuit Breaker (11.1); auf OpenAI-kompatible Schnittstelle umstellen |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| float | 18 | f32/f64 (Z7: Festkomma in Ranking-Pfaden) |
| serde | 12 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| async | 25 | tokio/async (G-NOTOKIO: Kern synchron) |
| zeit_zufall | 3 | Zeit/Zufall direkt (G3: nur ueber Ports Clock/CryptoRng) |
| panic | 1 | unwrap/expect/panic (G-LINT) |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `futures-util`, `parking_lot`, `reqwest`, `tracing` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-infer-http/`) | LOC | Befunde |
|---|---:|---|
| `infer-ollama/src/client.rs` | 48 | panic=1 |
| `infer-ollama/src/client/circuit_breaker.rs` | 194 |  |
| `infer-ollama/src/client/config.rs` | 65 | serde=1 |
| `infer-ollama/src/client/core.rs` | 1252 | float=4, serde=2, async=20 |
| `infer-ollama/src/client/errors.rs` | 39 |  |
| `infer-ollama/src/client/policy.rs` | 410 | serde=1 |
| `infer-ollama/src/client/validation.rs` | 63 |  |
| `infer-ollama/src/client/wire.rs` | 63 | float=2, serde=2 |
| `infer-ollama/src/embedding.rs` | 156 | float=2 |
| `infer-ollama/src/error.rs` | 190 | float=6, serde=4, zeit_zufall=3 |
| `infer-ollama/src/lib.rs` | 44 |  |
| `infer-ollama/src/query_rewriter.rs` | 303 | serde=2, async=5 |
| `infer-ollama/src/traits.rs` | 45 | float=4 |

# Extraktion fuer `acta-infer-http`

**Aufgabe:** Client fuer Inferenzdienste (OpenAI-kompatibel); tokio erlaubt  
**Level:** L8 · **Budget:** 3000 LOC · **Spezifikation:** docs/SPEC.md Kap. 11.1  
**Interne Abhaengigkeiten (Soll):** acta-base, acta-infer  
**Umfang des Archivs:** 17 Dateien, 5078 Zeilen (inkl. Tests)  

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
| `infer-ollama/src/client.rs` | 47 | panic=1 |
| `infer-ollama/src/client/circuit_breaker.rs` | 193 |  |
| `infer-ollama/src/client/config.rs` | 64 | serde=1 |
| `infer-ollama/src/client/core.rs` | 1251 | async=20, float=4, serde=2 |
| `infer-ollama/src/client/errors.rs` | 38 |  |
| `infer-ollama/src/client/policy.rs` | 409 | serde=1 |
| `infer-ollama/src/client/validation.rs` | 62 |  |
| `infer-ollama/src/client/wire.rs` | 62 | float=2, serde=2 |
| `infer-ollama/src/compat/client_tests.rs` | 1739 |  |
| `infer-ollama/src/embedding.rs` | 155 | async=2, float=2 |
| `infer-ollama/src/error.rs` | 189 | float=6, serde=4 |
| `infer-ollama/src/lib.rs` | 43 |  |
| `infer-ollama/src/model_info.rs` | 339 | panic=14, async=32, serde=1 |
| `infer-ollama/src/query_rewriter.rs` | 302 | async=8, serde=2 |
| `infer-ollama/src/traits.rs` | 44 | float=4 |
| `infer-ollama/tests/fake_server_tests.rs` | 122 | panic=11, async=18 |
| `infer-ollama/tests/property_backoff_tests.rs` | 19 |  |

# Spezifikation `contextra-infer-onnx`

## 1. Zweck & Einordnung
Normativitätsklasse: **Klasse A** (Autarker Baustein, Ring 2).
Stellt Embeddings und Cross-Encoder Reranking über ONNX Runtime oder deterministische Fake-Engines bereit.

## 2. Invarianten

| Invariante | Beschreibung | Test |
|---|---|---|
| `INV-EMBED-1` | Batch-Inferenz liefert identische Ergebnisse wie sequentielle Einzel-Inferenz | `tests/batch_equiv.rs` |
| `INV-EMBED-2` | Modell-Hash-Prüfung verhindert das Laden manipulierter Modelle (Fail-Closed) | `tests/model_security.rs` |
| `INV-EMBED-3` | Sichere Pfadprüfung verhindert Path-Traversal über `..` | `tests/model_security.rs` |
| `INV-EMBED-4` | Dimension-Validierung weist Mismatches ab | `tests/adversarial.rs` |
| `INV-PANIC-0` | Zero-Panic Invariante in Produktionscode | `tools/zero_panic_analyzer.py` |

## 3. Schnittstellen
- `trait Embedder`: `embed`, `embed_batch`, `dimension`.
- `trait Reranker`: `score`.
- Primärer Fehlertyp: `Error` (`thiserror`, `#[non_exhaustive]`).

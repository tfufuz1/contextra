# Spezifikation: `contextra-infer-onnx`

**Ring:** Ring 2 (External Adapters & Inferenz)
**Normativität:** Normativ gemäß `docs/SPEC.md` §1.1, §6.1, §6.2
**Klasse:** A (Vollständig autarker Baustein, B1–B12)
**MSRV:** 1.89.0
**Code-Disziplin:** `#![forbid(unsafe_code)]`, Zero-Panic

---

## 1. Zweck & Systemgrenzen
`contextra-infer-onnx` stellt In-Process Einbettungen und Reranking über ONNX Runtime sowie deterministische Fake-Engines bereit:
- **Autarkie (Klasse A):** 0 interne Abhängigkeiten zu `contextra-*`. Vollkommen unabhängig kompilierbar.
- **Sicherheit:** Strikte Pfadsicherheitsprüfung gegen Directory-Traversal und kryptografische Modellintegritätsprüfung via BLAKE3 / SHA-256.
- **Feature-Schema:** Standalone-Default (`default = []`), `onnx` (aktiviert `ort`, `tokenizers`, `ndarray`), `download` (aktiviert `reqwest`).

---

## 2. Metriken & Werkzeug-Audit-Befunde
*Erfasst via `tools/decouple_inspector.py -c contextra-infer-onnx`:*
- **LOC:** 690 LOC über 7 Rust-Dateien.
- **Typen-Inventar:** 2 Traits, 5 Structs, 2 Enums, 15 öffentliche Funktionen (`pub fn`).
- **Autarkie-Score:** 100/100 (0 interne Abhängigkeiten, 0 Leaks).
- **Zero-Panic-Audit:** 0 panics / unwraps im Produktionscode.

---

## 3. Architektur & Modulkarte

| Modul | Verantwortung | Primäre Typen / Traits |
|---|---|---|
| `traits.rs` | Lokale Trait-Verträge für Einbettung und Reranking | `Embedder`, `Reranker` |
| `onnx.rs` | Inferenz-Engine über `ort` (ONNX Runtime) | `OnnxEmbedder`, `OnnxReranker` |
| `fake.rs` | Deterministische Fake-Engines für Tests ohne native Laufzeit | `FakeEmbedder`, `FakeReranker` |
| `security.rs` | Pfadsicherheit (`verify_path_safety`) und Modell-Hash-Prüfung | `ModelHash`, `SecurityPolicy` |
| `error.rs` | Typisierter Fehlertyp mit `thiserror` | `Error`, `Result<T>` |

---

## 4. Invarianten (INV-ONX-*)

| Invariante | Beschreibung | Verifikation |
|---|---|---|
| `INV-ONX-ZERO-UNSAFE` | `#![forbid(unsafe_code)]` wird ausnahmslos durchgesetzt. | `cargo check -p contextra-infer-onnx` |
| `INV-ONX-EMBED-1` | Batch-Inferenz liefert identische Ergebnisse wie sequentielle Einzel-Inferenz. | `tests/batch_equiv.rs` |
| `INV-ONX-HASH-VERIFY-1` | Modell-Hash-Prüfung verhindert das Laden manipulierter Gewichte Fail-Closed. | `tests/model_security.rs` |
| `INV-ONX-PATH-SAFE-1` | Pfadprüfung weist versuchte Directory-Traversal-Angriffe (`../`) vor Dateizugriffen ab. | `tests/model_security.rs` |
| `INV-ONX-DIM-CHECK-1` | Vektordimensionen werden vor der Weitergabe gegen die Modellkonfiguration validiert. | `tests/adversarial.rs` |

---

## 5. Zielzustand & Baukasten-Integration
Vollständig realisierter Klasse-A-Baustein. Integration in Workspace über optionale Adapterbrücken.

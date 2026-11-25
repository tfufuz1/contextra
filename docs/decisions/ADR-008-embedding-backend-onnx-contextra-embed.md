# ADR-008: Embedding-Backend — ONNX (contextra-embed) → Ollama HTTP (contextra-ollama)

*   **Datum**: 2026-08-22
*   **Status**: ✅ Final (Ersetzt ADR-007 bzgl. lokaler ONNX-Inferenz)
*   **Entscheidung**: Ollama via `contextra-ollama` als primäres Embedding-Backend. `contextra-embed` wird vollständig aus Workspace-Dependencies und Features entfernt.
*   **Alternativen**: ONNX In-Process Embeddings (`contextra-embed`).
*   **Begründung**:
    - Ollama dient im KMU-Desktop-Szenario bereits als LLM-Runtime.
    - Modell-Tausch ohne Code-Änderung (Ollama-Modell-Name konfigurierbar).
    - Apple-Silicon ARM-Optimierung durch Ollama nativ vorhanden.
    - Reduziert C++ Native Build-Komplexität (kein ONNX-Runtime-Vendoring).
*   **Kosten & Konsequenzen**:
    - Höhere Latenz pro Embedding vs. In-Process-ONNX (mitigiert durch parallele Embedding-Batch-Requests in `contextra-ollama`).
    - Harte Laufzeit-Abhängigkeit von lokalem Ollama-Prozess.
    - `contextra-ollama` als shared Crate bereitgestellt für `contextra-tauri`, `contextra-mcp` und `contextra-py`.

---

---

# ADR-V1-0001: Ausrichtung auf lokalen Coding-Kontext-Server v1.0 und Funktionalitäts-Parken

* **Datum:** 2026-10-21
* **Status:** Akzeptiert (Gültig ab v1.0)
* **Kontext / Auslöser:**
  Contextra wird fokussiert und auf einen lokalen Coding-Kontext-Server verkleinert (v1.0: Modell Qwen-Coder, Kontext aus Repository-Index, exakter KV-Präfix-Cache).
  Komplexe, nicht unmittelbar für die Kernfunktion benötigte Subsysteme (wie hybrides Routing, Wissensgraphen, kryptografischer Löschnachweis, Python-Bindings, Agenten-Workflows, Compliance/Privacy-Module und erweiterte Inferenz-Backends) werden vorübergehend geparkt, um eine schlanke, performante und fehlerfreie v1.0-Release zu garantieren.

## Entscheidung

1. **Produktdefinition v1.0:**
   * Contextra dient primär als lokaler MCP-Coding-Kontext-Server (stdio), der Repository-Indizierung, Kontextblöcke mit Herkunftsnachweis, lokale GGUF-Inferenz via Candle (`contextra-infer-candle`, Modell Qwen-Coder) und exakten KV-Präfix-Cache (`contextra-kvcache`) bereitstellt.

2. **Löschverhalten in v1.0:**
   * In v1.0 enthält Contextra **keinen kryptografischen Löschnachweis**.
   * Der Aufruf von `contextra_forget` führt ein Standard-LSM-Tombstone-Löschen aus (`TombstoneOnly`).
   * Kryptografische DeletionProofs und physische Shredding-Nachweise entfallen in v1.0.

3. **Geparkte Subsysteme:**
   * Folgende Module und Crates werden aus der v1.0-Hauptentwicklung geparkt und über Entsprechungen in `docs/restore/<ID>/` bzw. entsprechende Restore-Karten rekonstruierbar gehalten:
     * Routing (`contextra-adapt`)
     * Knowledge Graph & PPR (`contextra-graph`)
     * Memory Consolidation / Cognition (`contextra-cognition`)
     * Persistent Checkpoint Engine (`contextra-checkpoint`)
     * Persistent Agent Workflows (`contextra-agent`)
     * Python Bindings (`contextra-py`)
     * Compliance, Privacy, AVV & License (`contextra-compliance`, `contextra-privacy`, `contextra-avv-generator`, `contextra-license`, `contextra-audit-export`)
     * ONNX-Inferenz (`contextra-infer-onnx`)
     * WASM-Sandbox (`contextra-sandbox`)
     * Benchmarks (`contextra-bench`)
   * Im Arbeitsbaum verbleiben ausschließlich die v1.0-Kernkomponenten (u. a. `contextra-kvcache` und `contextra-infer-candle`).

## Konsequenzen

* **Abweichung von der Spezifikation:**
  * Es besteht eine explizite, geplante Abweichung von `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`.
  * Der kryptografische Löschnachweis (`DeletionProof`) rückt aus dem Kern-Release heraus zurück in die Phase **R-ASSURED (geplant für Version v1.5)**.

* **Wiederherstellungsreihenfolge (Restore-Roadmap):**
  * **v1.1:** Routing (`contextra-adapt`)
  * **v1.2:** Graph & Cognition (`contextra-graph`, `contextra-cognition`)
  * **v1.3:** Rerank
  * **v1.4:** Checkpoint (`contextra-checkpoint`)
  * **v1.5:** Assured (Kryptografischer Löschnachweis & Privacy)
  * **v1.6:** Python & Agent (`contextra-py`, `contextra-agent`)
  * **v1.7:** Compliance & Cloud (`contextra-compliance`, `contextra-audit-export`, `contextra-license`)
  * **v1.8:** Weitere Inferenz (ONNX) & WASM-Sandbox (`contextra-infer-onnx`, `contextra-sandbox`)

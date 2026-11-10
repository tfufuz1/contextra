# PENDING_API_CHANGES.md — Öffentliche API-Änderungen (P05)

| Symbol / Pfad | Betroffene Crates | Gewünschte Änderung / Vorschlag |
|---|---|---|
| `contextra_rank::IsotonicCalibrator` | `contextra-db`, `contextra-infer-candle`, `contextra-mcp`, `contextra-engine` | Späteres Umschalten: Verschieben in Feature `experimental-rank` (sobald Konsumenten das Feature explizit anfordern oder die Nutzung abbauen). |
| `contextra_rank::fusion::apply_resonance_bonus` | `contextra-engine` | Verschieben hinter `experimental-rank`, sobald `contextra-engine` Resonanz optionalisiert oder isoliert. |
| `contextra_rank::fusion::ProvenanceRecord` | `contextra-mcp`, `contextra-engine` | Verschieben hinter `experimental-rank` bzw. kanonische Entkopplung von Explainability. |

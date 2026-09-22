# MemFuse — AI-Assistenten-Kontext (`memfuse-rank`)

## Verifizierter Codestand · Ring 0 (Rank)

> **Für AI-Assistenten:** Diese Datei beschreibt den Crate `memfuse-rank`.
> `memfuse-rank` definiert die Ranking- und Scoring-Funktionen für die Multi-Signal-Fusion (RRF, Reranking).
> Er erzwingt `#![forbid(unsafe_code)]`.

---

## Crate-Topologie

- **Ring 0 Rank**:
  - Ranking- und Scoring-Algorithmen zur Signal-Fusion und Re-Rank-Kaskadierung.

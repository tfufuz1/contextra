# ADR-095: Einzelne ADR-Dateien unter docs/decisions/ statt zentraler DECISIONS.md

*   **Datum**: 2026-09-08
*   **Status**: ✅ Final
*   **Supersedes**: ADR-006, ADR-060
*   **Entscheidung**: Architecture Decision Records (ADRs) werden als einzelne Markdown-Dateien unter `docs/decisions/ADR-NNN-*.md` geführt und nicht mehr in eine zentrale `DECISIONS.md` im Repository-Root zusammengeführt.
*   **Alternativen**: Beibehaltung der zentralen `DECISIONS.md` (ADR-006 / ADR-060).
*   **Begründung**: Die Praxis mit 90+ ADRs hat gezeigt, dass eine einzelne monolitische `DECISIONS.md` bei parallelen Agenten-Sessions zu Merge-Konflikten und Lock-Contention führt. Einzeldateien bieten eine feingranulare Git-Diff-Historie und harmonieren perfekt mit dem im Projekt genutzten Crate-Claiming-Mechanismus (`claim.rs`). Die CI-Gates (`check-adr-deadlines`, `check-consistency`) durchsuchen das Verzeichnis `docs/decisions/*.md` direkt.

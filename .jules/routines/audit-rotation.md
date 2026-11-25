# Routine: Audit-Rotation

- **Takt**: Wöchentlich (Freitag 22:00 UTC)
- **Ziel**: Wöchentliche proaktive Auditierung wechselnder Kern-Crates gemäß `.jules/AUDIT_INTAKE_PROTOCOL.md`.
- **Eingaben**:
  - `.github/workflows/scheduled-audit.yml`
  - `.jules/AUDIT_INTAKE_PROTOCOL.md`
- **Harte Grenzen**: Max. 15 Dateien / 1 Crate pro PR. Scope-Globs: `crates/*/**`, `.jules/pending-audit-task.md`.
- **Stopp-Bedingungen**: Unklarer Audit-Befund erfordert Rücksprache vor Codeänderung.
- **Ergebnisformat**: PR mit Audit-Behebungen und Aktualisierung von `pending-audit-task.md` auf `STATUS: DONE`.
- **Eskalation**: Siehe Architekturdokument §9.3.

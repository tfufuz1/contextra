# Harness Gate: Session Report (`cargo xtask session-report`)

## Summary

`cargo xtask session-report` automatically constructs the mandatory 7-section Pull Request body description at `.jules/local/PR_BODY.md`.

## Required Sections & Fallbacks

The generated PR body includes the following seven sections:
1. **Ziel**: Extracted from `--card` file or `--goal` text. If missing, displays `KEIN BELEG`.
2. **Änderungen**: Extracted directly from `git diff --name-only <base> <head>`. If empty, displays `KEIN BELEG`.
3. **Invarianten berührt**: Extracted from card metadata or defaults to `Keine spezifischen Invarianten berührt.`.
4. **Verifikation**: Formatted results from `--results-dir` JSON gate reports. If missing, displays `KEIN BELEG`.
5. **Out-of-scope Findings**: Extracted from `--findings` file or defaults to `Keine Out-of-scope Findings identifiziert.`.
6. **Risiken/Rollback**: Standard risk assessment and revert strategy.
7. **ADR/Spec-Sync**: Spec trailer sync status.

Standard flags supported: `--root <dir>`, `--base <rev>`, `--head <rev>`, `--json`, `--card <file>`, `--goal <text>`, `--results-dir <dir>`, `--findings <file>`.

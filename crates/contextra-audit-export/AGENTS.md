# Contextra — AI-Assistenten-Kontext (`contextra-audit-export`)

## Verifizierter Codestand · Tooling / Export

> **Für AI-Assistenten:** Diese Datei beschreibt den Crate `contextra-audit-export`.
> `contextra-audit-export` stellt Export-Pipelines für Audit- und Compliance-Protokolle bereit.
> Er erzwingt `#![forbid(unsafe_code)]`.

---

## Crate-Topologie

- **Audit & Compliance Export**:
  - Exportiert kryptographische Belege und Audit-Logs für externe Verifikation.
- **INV-AUDIT-BSI-MAPPING**: BSI TR-02102 Zuordnungstabelle muss alle aktiven Krypto-Primitiven aus der Codebasis vollständig auflisten (`bsi_mapping_table`).
  *Prüfung*: `cargo test -p contextra-audit-export --lib`
- **INV-AUDIT-GDPR-ART30**: Formatiert Verarbeitungsverzeichnisse deterministisch in Markdown und JSON (`render_register_markdown`, `render_register_json`).
  *Prüfung*: `cargo test -p contextra-audit-export --lib`
- **INV-FORBID-UNSAFE**: Strikte Einhaltung von `#![forbid(unsafe_code)]`.
  *Prüfung*: `cargo check -p contextra-audit-export`

## 4. Verboten / Anti-Patterns

- **Kein direkter Festplatten-Export im Core**: Audit-Exporte rendern In-Memory-Strings und überlassen I/O-Schreiben dem Anrufer.
- **Keine fehlende Mandanten-Validierung**: Verarbeitungsverzeichnisse ohne gültige `tenant_id` müssen mit `AuditExportError::InvalidTenant` abgewiesen werden.

## 5. Nebenläufigkeit, Async- und Lock-Regeln

- Alle Export-Methoden arbeiten synchron und zustandslos auf übergebenen In-Memory-Datenstrukturen.
- Keine internen Locks oder Shared-State-Variablen.

## 6. Verifikation

```bash
cargo test -p contextra-audit-export
cargo check -p contextra-audit-export
cargo xtask check-agents-integrity
cargo xtask doctrine-scan --crate contextra-audit-export
cargo xtask check-unsafe-islands
```

## 7. Bekannte Lücken / SOLL

- Stand: 2026-10-13 (Governance Alignment)
- Ring-Klassifikation: `capabilities.toml` ist die verbindliche Single Source of Truth (P12) und führt `contextra-audit-export` in Ring 0.

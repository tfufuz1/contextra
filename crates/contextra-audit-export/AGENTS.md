# AGENTS.md — contextra-audit-export
> Ring 0 · experimental · Quelle: capabilities.toml · Spec: III.21 / K.3

## 1. Zweck
Erzeugt Verarbeitungsverzeichnisse gemäß Art. 30 DSGVO und Audit-Log-Exporte für Contextra.
Exportiert zusammenfassende Audit-Protokolle, Egress-Gateway-Events und kryptographische Löschbelege.
Ordnet kryptographische Primitiven den BSI-Grundschutz- und TR-02102-Richtlinien zu.

## 2. Modul-Karte

| Datei | Verantwortung |
|---|---|
| `lib.rs` | Öffentliche Datenstrukturen (`ProcessingRegisterEntry`, `EgressEventSummary`) und Trait `ProcessingRegisterSource` |
| `bsi_mapping.rs` | `BsiMappingEntry` und Generierung der BSI Grundschutz / TR-02102 Zuordnungstabelle |
| `markdown_template.rs` | Rendering von Art. 30 Verarbeitungsverzeichnissen als Markdown-Dokumente |
| `error.rs` | Fehlerdefinitionen (`AuditExportError`) für Serialisierung und Mandantenvalidierung |
| `testkit.rs` | `InMemoryProcessingRegisterSource` für Testumgebungen |

## 3. Invarianten

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

- `cargo test -p contextra-audit-export`
- `cargo check -p contextra-audit-export`

## 7. Bekannte Lücken / SOLL

- Ring-Diskrepanz: `capabilities.toml` führt Crate in Ring 0, während Spec III.21 es als Ring 4 Utility beschreibt.

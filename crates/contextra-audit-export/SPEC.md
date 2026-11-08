# Spezifikation: `contextra-audit-export`

**Version:** 1.0 (SPEC v4.0, Welle 1)
**Rolle:** Autarkes Export-Werkzeug für Verarbeitungsverzeichnisse (DSGVO Art. 30) und BSI IT-Grundschutz / TR-02102 Zuordnungstabellen
**Baustein-Klasse:** Klasse A (Ring Extension / Autarker Baustein)

---

## 1. Zweck und Abgrenzung

### Zweck
`contextra-audit-export` liest neutrale Verarbeitungs- und Audit-Datensätze und transformiert diese deterministisch in typsichere, standardkonforme Exportformate:
- **JSON** (strukturiert, maschinenlesbar, deterministisch nach Mandant und Zweck sortiert)
- **Markdown** (tabellarisch formatiert mit striktem Tabellen- und Zeichen-Escaping)
- **CSV** (RFC 4180 konform mit robustem Field-Quoting)
- **BSI-Mapping** (kryptographische Zuordnungstabelle nach BSI TR-02102 und BSI IT-Grundschutz CON.1, CON.2, CON.6, OPS.1.1.5)

### Strikte Abgrenzung (§6)
- **Keine Löschlogik:** `contextra-audit-export` erzeugt Verzeichnis- und Audit-Berichte.
- **Keine Löschkette, kein Löschbeweis, keine Saga:** Das Crate enthält weder `Purger`, `LayerScanner`, `WitnessLayer` noch Saga- oder Deletion-Proof-Verifikationslogik. Alle Lösch- und Beweisabläufe verbleiben ausschließlich im Kern `contextra-erasure`.
- **Keine Rechtsberatung:** Die generierten Berichte und Zuordnungstabellen stellen technische Hilfsmittel dar und begründen keine formale Zertifizierung oder Rechtsberatung.

---

## 2. Invarianten

| ID | Invariante | Beschreibung |
|---|---|---|
| **INV-LOG-1** | Keine Nutzinhalte in Fehlermeldungen | Fehler enthalten ausschließlich Feldnamen oder Zähler, niemals sensible Nutzdaten, Passwörter oder Klartext-Payloads. |
| **INV-CLAIMS-1** | Ehrlichkeit und belegte Aussagen | Externe Zusicherungen enthalten keine unbewiesenen Garantien ("garantiert 100%"). Die BSI-Zuordnung weist explizit aus, dass keine formale Zertifizierung vorliegt. |
| **INV-EXPORT-DET-1** | Deterministische Sortierung | Exporte werden strikt nach `(tenant_id, processing_purpose, controller_name)` sortiert. Listenfelder werden lexikographisch stabil sortiert. |
| **INV-ZERO-PANIC** | Zero-Panic in Produktionscode | Alle Parsing-, Formatierungs- und Validierungspfade geben typisierte `Result<T, AuditExportError>` zurück. Keine Panics oder unbegrenztes Unwrapping. |

---

## 3. Schnittstellen & Datentypen

### `TenantId` (Baustein-Eigener Typ / B3)
Autarker 64-Bit Newtype (`TenantId(pub u64)`):
- `TenantId::try_new(id)`: Verhindert `0` (reserviert für `TenantId::SYSTEM`).
- `TenantId::from_raw(id)`: Direkte Konstruktion.
- Feature `types-compat`: Bietet bidirektionale `From`-Konvertierungen zu `contextra_types::TenantId`.

### `ProcessingRegisterEntry`
Enthält alle gemäß Art. 30 Abs. 1 DSGVO vorgeschriebenen Pflichtangaben:
- `controller_contact`: Verantwortlicher, Vertreter, DSB
- `processing_purpose`: Verarbeitungszweck
- `data_subject_categories`: Kategorien betroffener Personen
- `data_categories`: Kategorien personenbezogener Daten
- `recipient_categories`: Empfängerkategorien
- `third_country_transfers`: Drittlandübermittlungen und Garantien (Art. 30 Abs. 1 lit. e)
- `retention_periods`: Löschfristen (Art. 30 Abs. 1 lit. f)
- `technical_organizational_measures`: TOMs (Art. 32 / Art. 30 Abs. 1 lit. g)
- `legal_basis`: Rechtsgrundlage (z. B. Art. 6 Abs. 1 lit. b)
- `egress_events`: Referenzen auf Egress-Gate-Ereignisse
- `deletion_proofs`: Referenzen auf kryptographische Nachweise

### Hauptfunktionen
```rust
pub fn render_register_json(entries: &[ProcessingRegisterEntry]) -> Result<String, AuditExportError>;
pub fn render_register_markdown(entries: &[ProcessingRegisterEntry], export_date: &str) -> Result<String, AuditExportError>;
pub fn render_register_csv(entries: &[ProcessingRegisterEntry]) -> Result<String, AuditExportError>;
pub fn bsi_mapping_table() -> Vec<BsiMappingEntry>;
pub fn render_bsi_mapping_markdown(entries: &[BsiMappingEntry]) -> String;
```

---

## 4. Fehlermodell (`AuditExportError`)

Einziger `thiserror`-Fehlertyp des Bausteins (B4), mit `#[non_exhaustive]` annotiert:
- `Serialization(serde_json::Error)`: JSON-Formatierungsfehler
- `MissingField { field: String }`: Fehlende gesetzliche Pflichtangabe nach Art. 30 DSGVO
- `SourceCollection(String)`: Fehler beim Laden von Datensätzen aus einer Quelle
- `InvalidTenant(String)`: Ungültiger Mandant (z. B. 0)
- `CsvExport(String)`: Formatierungsfehler im CSV-Export

---

## 5. Nebenläufigkeit & Speichergarantien
- Vollkommen thread-safe (`Send + Sync` für alle Typen).
- Rein funktionale Transformationen ohne persistente Locks oder globale Stati.
- Big-O Laufzeit: $O(N \log N)$ Sortierung, $O(N)$ Serialisierung bei $N$ Datensätzen.

# SPEC: `contextra-avv-generator`

```
Dokument:  Baustein-Spezifikation `contextra-avv-generator`
Klasse:    A (Isolierter Baustein, SPEC v4.0 §2, §5.1)
Ring:      0 (Extension / Utility)
Stand:     2026-11-08
Status:    Produktionsreif / Autark
```

## 1. Zweck und Abgrenzung

`contextra-avv-generator` ist eine zustandslose, synchrone Template-Engine zur automatisierten Erzeugung strukturierter Entwürfe für Auftragsverarbeitungsverträge (AVV) nach Art. 28 DSGVO im Markdown-Format.

### Nicht-Ziele (Abgrenzung)
- **Keine Rechtsberatung:** Generierte Dokumente sind rechtlich unverbindliche Vorlagen. Ein Vorbehaltshinweis ist zwingender Bestandteil aller Ausgaben.
- **Keine Lösch- oder Transaktionslogik (§6):** Das Crate führt keine Löschungen durch, mintet keine Nachweise und verwaltet keine Schlüssel. Es formuliert lediglich vertragliche Vereinbarungen (SLAs, TOMs).
- **Keine externen I/O- oder Netzwerkoperationen:** Reines String-Building im Arbeitsspeicher; keine Dateisystem- oder Netzwerkzugriffe.

---

## 2. Baustein-Eigenschaften (Klasse A)

- **B1 / B2 (Keine internen Abhängigkeiten):** 0 Abhängigkeiten zu anderen `contextra-*`-Crates. Lokaler `TenantId`-Typ.
- **B3 (API in Standard- und Eigen-Typen):** Schnittstelle nutzt ausschließlich Standardtypen (`String`, `Vec`, `u32`, `u64`) und `TenantId`, `AvvContext`, `TechnicalMeasure`.
- **B4 (Fehlermodell):** Zentraler Fehlertyp `AvvError` (`#[non_exhaustive]`, `thiserror::Error`).
- **B5 (Isolierter Build):** Baut und testet autark im isolierten Workspace und als exportiertes Verzeichnis (`export_crate.py`).
- **B6 (Feature-Entkopplung):** `default = ["std"]`; `serde` ist optional hinter Feature-Flag `serde`.
- **B8 (Dokumentation & Benchmarks):** Vollständig dokumentiert; Criterion-Benchmark unter `benches/avv_bench.rs`.
- **B9 / B10 (Sicherheit):** `#![forbid(unsafe_code)]`, Zero-Panic in Produktion.
- **B11 (Determinismus):** Reine deterministische String-Formatierung ohne Zeit- oder Zufallsquellen.
- **B12 (Lizenzen):** `LICENSE-MIT`, `LICENSE-APACHE` im Crate enthalten.

---

## 3. Invarianten

| ID | Invariante | Verifikation |
|---|---|---|
| `INV-AVV-DISCLAIMER-1` | Jedes generierte Dokument enthält den rechtlichen Prüfungsvorbehalt (`Entwurf — ersetzt keine Rechtsberatung`). | `test_disclaimer_present_in_header` |
| `INV-AVV-DETERMINISM-1` | Identischer `AvvContext` und gleiches Datum führen stets zu bitgenau identischem Markdown. | `test_deterministic_rendering_with_fixed_date` |
| `INV-AVV-INJECTION-1` | Markdown-Steuerzeichen in Benutzereingaben werden in `escape_markdown` escaped, Zeilenumbrüche maskiert. | `test_markdown_escaping_injection_prevention` |
| `INV-AVV-VALIDATION-1` | Leere Parteinamen, ungültige Lösch-SLAs (nicht in 1..=365) oder leere autorisierte Unterauftragsverarbeiter führen fail-closed zu `AvvError`. | `test_validation_*` |

---

## 4. Fehlermodell (`AvvError`)

```rust
#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub enum AvvError {
    EmptyControllerName,
    EmptyProcessorName,
    EmptyContractDate,
    InvalidDeletionSla { days: u32, reason: &'static str },
    MissingSubprocessors,
    InvalidSubprocessor(String),
    InvalidTechnicalMeasure(String),
    InvalidContext(String),
    RenderError(String),
}
```

---

## 5. Benchmarks

Ausgeführt via Criterion (`benches/avv_bench.rs`):
- `escape_markdown`: Escaping von Strings mit Markdown-Steuerzeichen.
- `render_avv_markdown`: Vollständige Validierung und Generierung eines AVV-Dokuments.

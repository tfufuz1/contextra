# Extraktion fuer `acta-access`

**Aufgabe:** Mandanten, Rollen, Attribute (ABAC), Richtlinienauswertung, Sichtklassen, Zugriffsmengen  
**Level:** L1 · **Budget:** 3000 LOC · **Spezifikation:** docs/SPEC.md Kap. 9.4, 12.5, Z16  
**Interne Abhaengigkeiten (Soll):** acta-base  
**Umfang des Archivs:** 4 Dateien, 748 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `ports` | Nur Referenz fuer Richtlinienschnittstelle; ABAC ist Neubau |
| `types` | Metadatenfilter (`filter.rs`) als Referenz fuer Sichtklassen-/Attributfilter; Mandantenisolations-Tests als Orakel (Z16) |

## Dateien

| Datei (relativ zu `archive/acta-access/`) | LOC | Befunde |
|---|---:|---|
| `ports/src/policy.rs` | 44 |  |
| `types/src/types/filter.rs` | 427 | panic=2, serde=1 |
| `types/tests/filter_expressions_suite.rs` | 209 |  |
| `types/tests/tenant_scope_isolation.rs` | 68 | panic=6 |

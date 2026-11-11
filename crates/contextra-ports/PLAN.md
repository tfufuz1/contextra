# contextra-ports — Umsetzungsplan (Baukasten-Architektur)

> Abgeleitet aus `docs/SPEC.md` v4.1 (Class B Storage-Bundle, Ring 0, Vertragsschicht).

## 1. Status & Entkopplungshistorie

- **S-11 (`env` Extraktion):** Vollständig abgeschlossen. Grundlegende Umwelt-Ports (`Clock`, `Rng`, `IdGen`, `MetricsSink`) sind nach `contextra-env` migriert und werden hier für Rückwärtskompatibilität re-exportiert.
- **S-00 / Baseline:** Statische Prüfung und Auditierung via `tools/` (Zero-Unsafe, Zero-Panic in Produktion bestätigt).
- **Bereinigung von Compiler-Warnungen:** Behebung des doppelten Re-Exports von `KeyMaterialSource` in `src/lib.rs`.
- **Systematische Code-Dokumentation & Kommentierung:** Etablierung des einheitlichen Inline-Kommentierungsschemas mit `ANCHOR`, `SECTION`/`!SECTION`, `NOTE`, `LINK` über alle Module hinweg, um die Wiederverwendbarkeit in Drittprojekten transparent zu machen.

## 2. Modulare Baukasten-Ziele für contextra-ports

1. **Vertragsintegrität:** Bereitstellung modularer Abstraktionsverträge für Storage-, Index-, Inference- und Lifecycle-Subsysteme ohne konkrete Engine-Abhängigkeiten.
2. **Klares Kommentierungssystem:** Vollständige Nachvollziehbarkeit aller Traits, Methoden, Invarianten und Fail-Closed-Verhalten für Entwickler und Coding-Agenten.
3. **Dyn-Kompatibilität:** Sicherstellung, dass alle Port-Traits objekt-sicher (`dyn-compatible`) bleiben, um dynamischen Polymorphismus in Satelliten- und Integrationscrates zu ermöglichen.
4. **Keine unzulässigen Vorwärts- oder Rückwärtsabhängigkeiten:** Striktes Einhalten der Ring-0-Hierarchie (`contextra-types` und `contextra-env`).

## 3. Definition of Done

- `cargo check -p contextra-ports --no-default-features --locked` kompiliert fehlerfrei.
- `cargo test -p contextra-ports --locked` läuft erfolgreich durch.
- `tools/zero_panic_analyzer.py --crate contextra-ports` meldet 0 Produktionsverstöße.
- `tools/determinism_guard.py --crate contextra-ports` meldet 0 unzulässige Befunde.
- `tools/unsafe_boundary_auditor.py --crate contextra-ports` bestätigt striktes `#![forbid(unsafe_code)]`.
- `tools/decouple_inspector.py -c contextra-ports` dokumentiert den modularen Schnittstellenkatalog.

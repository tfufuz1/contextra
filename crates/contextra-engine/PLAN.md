# contextra-engine — Umsetzungs- & Migrationsplan (v4.1 Baukasten-Architektur)

> Abgeleitet aus `docs/SPEC.md` (v3/v4.1 Gesamtspezifikation) und den Entkopplungsleitlinien für Klasse-C-Komponenten.

## 1. Status der Roadmap-Schritte

| Schritt | Inhalt | Status |
|---|---|---|
| **S-00 / S-01** | Tooling & Layering-Gates Baseline | Abgeschlossen |
| **S-02 / S-03** | Golden-Korpus, Differenz-Harness, Metadaten-Bereinigung | Abgeschlossen |
| **S-33** | `rank`: Fusion & Scoring-Entkopplung | Abgeschlossen |
| **S-37** | `store`: Speicherkern-Entkopplung (`StoreError`, `compat/`) | Abgeschlossen |
| **S-50 / S-51** | Lizenz- und Erasure-Ports nach `contextra-license` / `contextra-erasure` | Abgeschlossen |
| **S-60** | Entkopplung Regler (`AdaptiveDecayController`) | Vorbereitet / Bereit |
| **S-61** | Chunker: Extraktion/Parität mit `contextra-ingest` | Vorgemerkt für Wave 5 |
| **S-63** | `cognition` Invertierung (Entfernung von `contextra-engine` aus normal tree) | Abhängigkeit isoliert |
| **S-64** | `contextra-engine` Baustein-Entkopplung (`Backends`-Trait, Dokumentation) | **In Umsetzung** |

## 2. Detaillierte Maßnahmen für `contextra-engine` (S-64)

1. **Backends-Abstraktion (WP-ENG-1):**
   - Etablierung des Trait-Bündels `Backends` mit `Storage`, `Vector`, `Text` und `Graph` in `src/backends.rs`.
   - Bereitstellung von `DefaultBackends` für nahtlose Abwärtskompatibilität ohne Zwang zur manuellen Typbindung.

2. **Beseitigung von DSGVO- und Löschketten-Relikten (E-04):**
   - Isolierung aller Löschketten- und DSGVO-Referenzen im Engine-Kern.
   - Verlagerung spezialisierter Löschlogik in `contextra-erasure` und `contextra-erasure-adapters`.

3. **Einführung des einheitlichen Comment-Anchors-Systems:**
   - Standard-Anker (`ANCHOR`, `NOTE`, `TODO`, `FIXME`, `STUB`, `REVIEW`).
   - Regionen (`SECTION`, `!SECTION`) für hierarchische Strukturierung in IDEs.
   - Link-Anker (`LINK`) für modulübergreifende Querverweise.
   - Ausführliche Erläuterung aller Sperren, Invarianten und Fail-Closed-Pfade für nachfolgende Entwickler.

4. **Qualitätsgates & Validierung:**
   - Zero-Panic-Audit in Produktionspfaden.
   - Determinismus-Audit (`determinism_guard.py`).
   - Einhaltung der Rust 2021-Formatierung.

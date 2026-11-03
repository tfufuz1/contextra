# Quellen der Wahrheit

Abgeleitet aus [`SPEC.md`](SPEC.md), Abschnitt „Quelle der Wahrheit“ und D-98 (Abschnitt 4.6). Bei Widerspruch gilt die SPEC.

## 1. Grundregel

Jede Tatsache hat genau eine normative Quelle: die SPEC. Abgeleitete Dokumente wiederholen Fakten nur, wenn sie für das Verständnis nötig sind, und verweisen dann auf die SPEC-Stelle. Weicht ein abgeleitetes Dokument von der SPEC ab, ist das Dokument fehlerhaft und wird im selben Änderungspaket korrigiert (D-98).

## 2. Rangfolge

| Rang | Quelle | Status |
|---|---|---|
| 1 | `docs/SPEC.md` (Version 5.1.0) | normativ, einzige Quelle |
| 2 | `docs/VETOES.md` | normativ, konkretisiert Verbote aus der SPEC |
| 3 | `docs/CONSTITUTION.md` | abgeleitet, fasst Prinzipien und harte Regeln zusammen |
| 4 | `docs/ARCHITECTURE.md`, `docs/TYPE_REGISTRY.md` | abgeleitet |
| 5 | `docs/COMPILATION.md`, `docs/CONTRIBUTING.md`, `docs/SECURITY.md`, `docs/DELETION-DSGVO.md`, `AGENTS.md`, `README.md`, `crates/*/README.md`, `crates/*/AGENTS.md`, `.agents/` | abgeleitet |
| 6 | Quellcode und grüne Gates (`cargo check --locked`, Tests, Lab-Protokolle) | Beleg, kein Maßstab |

Der Quellcode belegt den Ist-Zustand. Er definiert keinen Soll-Zustand. Grüne Gates belegen die Einhaltung der SPEC, sie ersetzen sie nicht.

## 3. Abgeleitete Dokumente und ihr Pflegestand

Nach D-98 bleiben alle abgeleiteten Dokumente im Repository und werden mit der SPEC gepflegt. Die Tabelle nennt den Stand dieser Revision (SPEC 5.1.0).

| Datei oder Verzeichnis | Stand | Offener Pflegebedarf |
|---|---|---|
| `AGENTS.md`, `README.md` | überarbeitet | – |
| `docs/` (außer SPEC) mit `ARCHITECTURE`, `COMPILATION`, `CONSTITUTION`, `CONTRIBUTING`, `SECURITY`, `SOURCE_OF_TRUTH`, `TYPE_REGISTRY`, `VETOES`, `DELETION-DSGVO`, `CHANGELOG` | überarbeitet | Rechtsstand in `DELETION-DSGVO` bei Änderungen prüfen (G-LEGAL) |
| `docs/IMPLEMENTATION_SPEC_PER_CRATE.md` | nicht überarbeitet | Abgleich mit SPEC 8 |
| `docs/DELETION-DSGVO.md` | überarbeitet | – |
| `crates/*/README.md`, `crates/*/AGENTS.md` | überarbeitet (14 Crates und `contextra-erasure`) | Zweck-Sätze in `lib.rs` (WP-1.5) |
| `.agents/**/*.md` | überarbeitet (40 Dateien) | Skripte und Konfiguration offen (siehe `.agents/README.md`) |
| `.agents/` als Werkzeugverzeichnis | Teil des Repositorys (D-98) | Konsistenz der Pfade und Workflows nach T-DOC-1 |
| `.gemini/`, `.antigravity/` | Teil des Repositorys (D-98) | nicht überarbeitet, Konsistenz prüfen |
| `docs/CHANGELOG.md` | Kopf überarbeitet, Historie erhalten | – |
| `docs/AUDIT_*.md`, `docs/ROOT_CAUSE_ANALYSIS.md` | Prüfprotokolle früherer Audits | nach D-98 (3) nicht im Quellbaum; in den Archivbranch verschieben |
| `docs/cleanup/`, `docs/postmortems/`, `docs/analysis/` | Protokolle und Analysen | nach D-98 (3) nicht im Quellbaum; in den Archivbranch verschieben |
| `docs/explain/protected-paths.toml` | nicht geprüft | Pfadregeln in `.agents/governance/` abgleichen |
| `reports/`, `evidence/` | Evidenz | nach D-98 (3) nicht im Quellbaum; als Release-Artefakt führen |
| `tools/*.py`, `scripts/contextra_full_audit.py` | Einmalwerkzeuge | nicht von D-98 erfasst; Entscheidung des Eigentümers offen |

## 4. Umgang mit Widersprüchen

1. Widerspruch zwischen einem abgeleiteten Dokument und der SPEC: Das Dokument wird im selben Änderungspaket korrigiert.
2. Widerspruch zwischen SPEC und Quellcode: Der Code ist ein Defekt und wird mit einem Arbeitspaket behoben.
3. Widerspruch innerhalb der SPEC: Die SPEC wird über Anhang H korrigiert. Bis dahin gilt die strengere Auslegung (SPEC 6.6).
4. Fehlende Information: Es wird nichts erfunden. Die Lücke wird als Arbeitspaket oder Gate-Bedingung erfasst.

## 5. Prüfung

Die Konsistenz der abgeleiteten Dokumente prüft der Test T-DOC-1 (`contextra-arch-tests`). Er erfasst Zweck-Sätze, Löschbegriffe, Pfad- und Workflow-Verweise sowie Zeitstempel- und Agentenmarken. Die Prüfung auf Sitzungsverläufe, Prüfprotokolle und Evidenz im Quellbaum erfolgt über T-ARCH-8. Beide Tests sind Teil von Gate B.

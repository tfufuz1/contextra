# Evaluierung der `.jules/`-Routinen und Prompts (`docs/status/JULES-VORSCHLAG.md`)

*Soll-Architektur & Governance gemäß Contextra Spezifikation 1.2 (§27, §28, §31, §33)*

Gemäß REQ-GOV-002 werden die Dateien im Verzeichnis `.jules/` selbst im Rahmen dieser Evaluation **nicht** direkt modifiziert. Die folgenden Tabellen und Abschnitte dokumentieren das Verifikationsergebnis aller Routinen und Prompts gegen Spezifikation 1.2.

---

## 1. Routinen-Evaluierung (`.jules/routines/*.md`)

| Routine-Datei | Takt / Ziel | Evaluierung | Empfehlung | Spezifikations-Bezug | Begründung & Anpassungsbedarf |
|---|---|:---:|:---:|:---:|---|
| `.jules/routines/audit-rotation.md` | Wöchentliche proaktive Auditierung wechselnder Kern-Crates | Konform | **behalten** | §27.1, §28.3 | Ergänzt die unabhängige Verifikation und Audit-Protokollierung gemäß §28.3. |
| `.jules/routines/bench-trend.md` | Wöchentliche Performance- und Regressionstrend-Analyse | Konform | **behalten** | §26, §27.4, REQ-QA-007 | Überwacht Latenz/Durchsatz-Budgets aus `benchmarks/budgets.toml` gemäß §27.4. |
| `.jules/routines/chaos-fuzz-langlauf.md` | Nächtlicher Lauf der Chaos-Matrix und Fuzz-Smoke-Tests | Konform | **behalten** | §27.1, §27.3, REQ-QA-003, REQ-QA-004, REQ-QA-006 | Erzwingt kontinuierliches Fuzzing und Fault-Injection mit `FaultVfs` gemäß §27.3. |
| `.jules/routines/dependency-hygiene.md` | Wöchentliche Lizenz-, Vulnerability- und Supply-Chain-Prüfung | Konform | **behalten** | §28.4, REQ-GOV-006 | Verifiziert Abhängigkeiten gegen Advisories und `cargo-deny`/`cargo-vet` gemäß §28.4. |
| `.jules/routines/drift-waechter.md` | Tägliche Früherkennung von Doku-, Typ- und Registrierungs-Drift | Konform | **behalten** | §27.5, §28.5, REQ-GOV-004 | Verhindert Single-Source-of-Truth-Verletzungen (INV-DOC-SINGLE-SOURCE-1) gemäß §28.5. |
| `.jules/routines/flake-jagd.md` | Nächtliches Aufspüren nicht-deterministischer Flakes | Konform | **behalten** | §27.2, §27.8, REQ-QA-002 | Setzt Red->Green-Disziplin und deterministische Simulationen gemäß §27.8 durch. |
| `.jules/routines/unsafe-pruefung.md` | Wöchentliche Verifikation aller Unsafe-Inseln und Miri-Sauberkeit | Korrekturbedarf | **korrigieren** | §5 P2, §27.1, §27.5, REQ-ARCH-004, REQ-ARCH-005 | Grundsätzlich konform. Empfehlung: Bezeichnungen historischer Module an Spezifikation 1.2 anpassen (Verweis auf Vfs statt `durable-fs`). |
| `.jules/routines/unwrap-allow-abbau.md` | Wöchentlicher Abbau verbliebener `unwrap()`-Aufrufe und `#[allow]` | Konform | **behalten** | §5 P7, §27.5, REQ-ARCH-006, REQ-QA-005 | Setzt den Zero-Panic-Ratchet und Ziel-0-Abbau der Clippy-Allowances gemäß §27.5 um. |

---

## 2. Prompts und Prozessdokumente (`.jules/*.md`)

| Datei | Zweck | Evaluierung | Empfehlung | Spezifikations-Bezug | Begründung & Anpassungsbedarf |
|---|---|:---:|:---:|:---:|---|
| `.jules/PREAMBLE.md` | System-Preamble für KI-Agenten-Sessions | Konform | **behalten** | §28.2, REQ-GOV-001 | Definiert unumstößliche Kern-Direktiven und Safety-Grenzen für autonome Agenten. |
| `.jules/JULES_CONTEXT.md` | Zentraler Prozesskontext & Invarianten für Agenten | Korrekturbedarf | **korrigieren** | §20.5, §28.2, §31 | Terminologie-Anpassung erforderlich: Ersetzung von `durable-fs` durch `Vfs` (§31), Klarstellung zu `DeletionProof v4` (§20.5). |
| `.jules/AUDIT_INTAKE_PROTOCOL.md` | Standard-Protokoll zur Erfassung externer Auditergebnisse | Konform | **behalten** | §28.3, REQ-QA-001 | Regelt die strukturierte Übernahme von Audit-Befunden und Nachweisen. |
| `.jules/ISSUE_AUTOMATION_POLICY.md` | Richtlinie für automatisierte Governance-Issues | Konform | **behalten** | §28.2, REQ-GOV-001 | Stellt sicher, dass automatisch erzeugte Issues den Claim- und Task-Kriterien entsprechen. |
| `.jules/SCHEDULED_AUDIT.md` | Instruktionen für periodische CI-Sicherheitsaudits | Konform | **behalten** | §27.5, §28.3 | Regelt den Ablauf automatisierter periodischer Audits. |
| `.jules/SESSION_BOOTSTRAP.md` | Initialisierungsschritte zu Beginn jeder Session | Konform | **behalten** | §28.2, REQ-GOV-001 | Garantiert konsistente Session-Umgebung und Preflight-Checks. |
| `.jules/HARNESS_QUICKSTART.md` | Entwickleranleitung für das `xtask`-Gate-Harness | Konform | **behalten** | §27.5, §27.10, REQ-QA-001 | Beschreibt die Nutzung von `cargo xtask gate-check` und `cargo xtask verify`. |
| `.jules/PROMPTER_RESTRUCTURING_PLAN.md` | Restrukturierungsplan der Agenten-Prompts | Konform | **behalten** | §28.5 | Dokumentiert den Umbau der Prompter-Struktur; kann nach Abschluss archiviert werden. |
| `.jules/contextra-Prompter.html` | Interaktive Web-Benutzeroberfläche für Prompter | Konform | **behalten** | §28.5 | Bietet ein visuelles Dashboard für Agenten-Konfiguration und Prompts. |

---

## 3. Zusamenfassung der vorgeschlagenen Maßnahmen

1. **Beibehalten (15 Elemente)**: Alle 7 Routinen (`audit-rotation`, `bench-trend`, `chaos-fuzz-langlauf`, `dependency-hygiene`, `drift-waechter`, `flake-jagd`, `unwrap-allow-abbau`) sowie 8 Prozessdokumente stimmen in Ziel und Struktur mit Spezifikation 1.2 überein und sollen ohne strukturelle Änderung beibehalten werden.
2. **Korrigieren (2 Elemente)**:
   - `unsafe-pruefung.md`: Ersetzung veralteter Crate-Namen (`durable-fs` -> `Vfs`) im Beschreibungstext zur vollständigen Ausrichtung an Spezifikation §31.
   - `JULES_CONTEXT.md`: Ersetzung historischer Begriffe (`durable-fs`, `DeletionProof v3`) durch die normativen Bezeichnungen aus Spezifikation §20.5 und §31 (`Vfs`, `DeletionProof v4`).
3. **Entfernen (0 Elemente)**: Keine obsoleten oder der Spezifikation widersprechenden Routinen/Prompts identifiziert.

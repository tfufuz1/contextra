# ADR-108: Meta-System-Konsolidierung

* **Status:** Proposed
* **Datum:** 2026-10-25
* **Autoren:** Principal Architecture Governance Engineer (Contextra)
* **Betroffene Komponenten:** `xtask`, `xtask-heavy`, `.github/workflows/`, `governance/`, `docs/`, `.jules/`, `justfile`

---

## 1. Kontext & Problemstellung

Die Befunde und Analysen aus Dokument C (`docs/CONTEXTRA_META_PLAN.md` §1) dokumentieren eine signifikante Entkopplung und Erosion des bestehenden Meta-Systems gegenüber der normativen Gesamtspezifikation (Dokument A, Fassung 1.3):

1. **Widerspruchsregister & Wissens-Entkopplung (Dokument C §1.4):**
   - Veraltete Pfade und Spec-Verweise in Arbeitsanweisungen (`AGENTS.md`, `README.md`, `ARCHITECTURE.md` verweisen auf nicht-existente Pfade wie `docs/spec/`).
   - Abweichende Invarianten- und Klassifikationsangaben zwischen Spezifikation, Crate-Dokumenten und Tooling.
   - Widersprüchliche Regelwerke ohne Durchsetzungskraft durch funktionierende CI-Gates.

2. **Gate-Wildwuchs und unklare Autorität (Dokument C §1.5):**
   - Derzeit existieren 109 Kommandos, 32 Workflows und ein 494 Zeilen altes `justfile` mit massiver funktionaler Überlappung.
   - Weit verbreitete Escape-Marker (`harness:planned`, `*-ignore`) verbergen tote Referenzen und schwächen die Gate-Integrität.
   - Die zentrale `xtask/src/main.rs` (4.635 Zeilen) leidet unter direkter Produktkopplung (`contextra-bench`, `router`, `types`), wodurch Produkt-Build-Fehler sämtliche Governance-Gates blockieren.

3. **Fehlende Single Source of Truth (SSOT):**
   - Reifegrad, Ringe, Invarianten und Governance-Regeln sind über viele Dateien verstreut.
   - Es fehlt eine maschinenlesbare, unbezweifelbare Verfassungsdatei für das Meta-System.

---

## 2. Entscheidung

Es wird das in Dokument C (§2) definierte Zielbild der Meta-System-Konsolidierung schrittweise und strukturiert umgesetzt:

1. **Einführung einer zentralen Verfassungsdatei (`governance/constitution.toml`):**
   - Dient als alleinige Quelle der Wahrheit (SSOT) für Projektdaten, Prinzipien P1–P30, Invarianten, Vetos und Plattform-Annahmen.
2. **Entkopplung & Aufteilung von `xtask`:**
   - Bereinigung von `xtask` zu einem Produkt-unabhängigen, schnellen Governance-Werkzeug.
   - Auslagerung schwerer Läufe (Fuzz, Miri, Mutants, Chaos, Loom, Benchmarks) in einen separaten Workspace `xtask-heavy`.
3. **Kanonischer Gate-Katalog (`governance/gates.toml`):**
   - Reduktion auf max. 34 wohl-definierte, phasenbewusste Gates in den Tiers `pr`, `merge`, `nightly` und `heavy`.
   - Striktes Verbot von Escape-Markern und stillen Streichungen.
4. **Schlanke Kontextkette für Agenten & Entwickler:**
   - Reduzierte `AGENTS.md` (≤ 150 Zeilen) und standardisierte Crate-Karten (≤ 40 Zeilen).
   - Generierung aufgabenbezogener Context-Packs statt des Ladens des gesamten Repository-Bestands.

---

## 3. Betrachtete Alternativen

1. **Status quo behalten:**
   - *Verworfen:* Führt zu fortlaufenden Drift-Problemen, Verwirrung bei Agenten/Entwicklern, unzuverlässigen CI-Läufen und Nichterfüllung der Invariante INV-DOC-SINGLE-SOURCE-1.
2. **Nur dokumentarische Bereinigung:**
   - *Verworfen:* Beseitigt nicht die strukturelle Koppelung in `xtask`, die Escape-Marker in den Skripten und die Redundanz im Workflow-Bestand.
3. **Zielbild Meta-System-Konsolidierung wie in Dokument C (Gewählt):**
   - *Akzeptiert:* Stellt eine saubere Trennung von Belangen, maschinenlesbare Governance, strikte Gate-Durchsetzung und Wartbarkeit sicher.

---

## 4. Konsequenzen

- Alle Meta-Dateien, Skripte, Gates und Workflows werden schrittweise gemäß den Arbeitspaketen MP-01 bis MP-16 konsolidiert und refaktoriert.
- Vor dem Löschen oder Ersetzen bestehender Gates wird deren Disposition transparent im Disposition-Anhang geführt.
- `cargo check --workspace --all-targets --locked` bleibt zu jedem Zeitpunkt auf `main` kompilierbar.

---

## 5. Vorab-Entscheidungen (E-1 bis E-10 aus C §7)

Die folgenden Vorgaben sind als Defaults vorgeschlagen und bedürfen der finalen Freigabe durch den Auftraggeber (Status: *vorgeschlagen, Freigabe durch den Auftraggeber*):

- **E-1:** ADR-097 (Graph-Reparatur) vs. VETO-F02: Spezifikation 1.3 gilt; ADR-097 wird durch ADR-109 als *Superseded* deklariert.
- **E-2:** Prompter-System entfernen und durch `task new --wp` ersetzen: Ja.
- **E-3:** `AI-TAG`/`ANCHOR`-Tags schrittweise abbauen (MP-17): Ja.
- **E-4:** `WORKING_STATE.md` nicht mehr committen (ersetzt durch `xtask status`): Ja.
- **E-5:** GitHub Merge Queue aktivieren (`merge_group` Ruleset): Ja.
- **E-6:** Unabhängiger Runner als getrennter Workflow ohne Cache: Ja.
- **E-7:** Echte `CODEOWNERS`-Handles einsetzen: Durch Auftraggeber bereitgestellt.
- **E-8:** `justfile` auf dünne Aliase (≤ 25 Zeilen) reduzieren: Ja.
- **E-9:** `jules-dispatch` in `xtask-heavy` beibehalten: Ja.
- **E-10:** Tier-Modell (Tier 0–3) durch `class` + `risk_class` in `gates.toml` ersetzen: Ja.

---

## Anhang: Disposition der Gates, Kommandos und Workflows

*Hinweis: Dieser Anhang wird im Laufe der Arbeitspakete MP-01 bis MP-15 fortgeschrieben und von MP-16 mit den finalen Ergebnissen und Prüfbelegen befüllt.*

### Gates & Kommandos
| Alt-Element / Kommando | Status / Disposition | Ersetzt durch / Begründung |
| :--- | :--- | :--- |
| *Wird von MP-02..MP-15 befüllt* | *Ausstehend* | *Wird von MP-16 final attestiert* |

### Workflows (`.github/workflows/`)
| Workflow | Status / Disposition | Ersetzt durch / Begründung |
| :--- | :--- | :--- |
| *Wird von MP-15 befüllt* | *Ausstehend* | *Wird von MP-16 final attestiert* |

# ADR-N08: Generierte Spezifikation und Capability-Manifest (`capabilities.toml`)

* **Status:** Beschlossen / Final (Prinzip P12, Gesamtspezifikation §20.3)
* **Datum:** 2026-09-17
* **Kontext / Auslöser:**
  In komplexen Multi-Crate-Systemen führt die manuelle Pflege von Dokumentation, Feature-Katalogen und Reifegrad-Markern (z.B. Alpha, Beta, Production) häufig zu Diskrepanzen zwischen dem tatsächlichen Code-Zustand und den Spezifikationsdokumenten (Spezifikations-Drift).

  Um dies nachhaltig zu verhindern, führt Prinzip **P12** das Konzept einer maschinell ausgewerteten Single Source of Truth für Feature-Reifegrade und System-Capabilities ein.

---

## 1. Finale Entscheidung

Es wird verabschiedet, dass **`capabilities.toml` im Repository-Root** als die **einzige maßgebliche Quelle (Single Source of Truth)** für Feature-Reifegrade, Crate-Capabilities und Audit-Marker definiert wird.

### 1.1 Struktur von `capabilities.toml`
Die Datei `capabilities.toml` erfasst deklarativ:
- Crate-Namen und zugewiesene Architecture-Ringe (Ring 0 bis Ring 3).
- Feature-Reifegrad (z.B. `Experimental`, `Stable`, `Deprecated`).
- Zugehörige Test-Garantien und Verweise auf CI-Gates (z.B. Coverage, Drift-Check, Benchmarks).

### 1.2 Maschinelle Dokumentations-Generierung (`cargo xtask sync-docs`)
1. **Keine manuelle Synchonisierung:** Reifegrad-Tabellen in `WORKING_STATE.md`, `docs/ARCHITECTURE.md` und `docs/SOURCE_OF_TRUTH.md` dürfen **nicht mehr manuell** editiert werden.
2. **Generierung via Tooling:** `cargo xtask sync-docs` liest `capabilities.toml` sowie Inline-Code-Tags (`AI-TAG`, `ANCHOR`, `REVIEW-PASS`) aus und generiert die entsprechenden Abschnitte in den Dokumentationsdateien automatisiert.
3. **CI-Gate enforcement:** `cargo xtask sync-docs --check` (bzw. `just sync-docs-check`) prüft in der CI-Pipeline, ob die generierten Dokumente exakt mit `capabilities.toml` übereinstimmen. Bei Abweichungen schlägt der Build fehl.

---

## 2. Begründung

- **Vermeidung von Dokumentations-Drift:** Dokumentation spiegelt garantiert den tatsächlichen, in `capabilities.toml` hinterlegten Stand wider.
- **Transparenz:** Entwickler und Product Owner haben eine zentrale, übersichtliche TOML-Datei zur Verwaltung aller Feature-Status.
- **Automatisierbarkeit:** CI-Pipelines können Reifegrad-Marker direkt parsen und z.B. experimentelle Features automatisch in Production-Builds sperren.

---

## 3. Konsequenzen

- Manuelle Änderungen an generierten Abschnitten in den Dokumentationsdateien werden durch CI-Gated-Checks abgelehnt.
- Neue Features oder Statusänderungen müssen zuerst in `capabilities.toml` eingetragen und mittels `cargo xtask sync-docs` in die Dokumente synchronisiert werden.

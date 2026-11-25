# Cargo Audit Integration Guide

Dieses Dokument beschreibt den Zweck, die Nutzung und die CI-Integration von `cargo-audit` im Contextra-Projekt.

---

## 1. Zweck

`cargo-audit` führt eine automatisierte Sicherheitsanalyse aller in `Cargo.lock` festgelegten Workspace-Abhängigkeiten durch.
Dabei werden die verwendeten Crates gegen die [RustSec Advisory Database](https://rustsec.org/) abgeglichen, um bekannte Schwachstellen (CVEs/Advisories), als ungewartet markierte Crates oder Yanks frühzeitig zu erkennen.

---

## 2. Ausführung & Justfile-Integration

Das Projekt stellt dafür ein eigenes, entkoppeltes Just-Recipe zur Verfügung:

```bash
just security-audit
```

Das Recipe prüft zunächst, ob das CLI-Tool `cargo-audit` lokal bzw. in der Umgebung installiert ist.
Ist es installiert, führt es folgenden Befehl aus:

```bash
cargo audit --deny warnings
```

---

## 3. Lokale Installation & CI-Setup

### Lokale Verifikation
Für die lokale Entwicklung und manuelle Überprüfung kann `cargo-audit` wie folgt installiert werden:

```bash
cargo install cargo-audit --locked
```

*Hinweis:* Das automatische Ausführen von `cargo install` innerhalb von Test-Runs oder Just-Recipes wird als Anti-Pattern vermieden, um reproduzierbare, schnelle und offline-fähige Standard-Build-Abläufe zu gewährleisten.

### CI-Integration & Container-Environments
In CI-Runnern, Docker-Images und Dev-Containern (z. B. `.jules/setup/environment_script.sh` oder Nix-Flakes) sollte `cargo-audit` als vorinstalliertes Tooling im CI-Environment / Base Image bereitgestellt werden.

---

## 4. Interpretation von Findings

Wenn `just security-audit` Fehler oder Warnungen ausgibt, sind folgende Typen zu unterscheiden:

1. **Security Vulnerability (Cve/Advisory):**
   Eine bekannte Sicherheitslücke in einer genutzten Crate-Version.
2. **Unmaintained / Warning:**
   Die Crate wird vom Maintainer als nicht mehr gepflegt eingestuft oder weist andere Warnungen auf.
3. **Yanked Crate:**
   Die spezifische Crate-Version wurde auf crates.io zurückgezogen (Yanked).

---

## 5. Eskalationspfad & Behebung bei kritischen Findings

Wenn `just security-audit` fehlschlägt:

1. **Abhängigkeit aktualisieren:**
   Prüfe, ob ein Patch für die betroffene Crate existiert:
   ```bash
   cargo update -p <crate_name>
   ```
2. **Breaking Changes evaluieren:**
   Falls ein Major-Update erforderlich ist, erstelle einen entsprechenden Issue/Branch zur Behebung.
3. **Ausnahme / Whitelisting (nur nach Security-Review):**
   Sollte ein Finding ein False Positive sein oder keine Auswirkung auf das Projekt haben (z. B. nur in nicht genutztem Test-Code) und ein Update nicht sofort möglich sein, kann nach Abstimmung mit dem Security/Architecture-Lead ein temporärer Exclusion-Eintrag in `audit.toml` oder via Ignorier-Flag vorgenommen werden.

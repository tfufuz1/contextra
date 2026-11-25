# Contextra — Supply Chain Trust Framework

## Zweck & Kontext

Dieses Dokument beschreibt die Werkzeuge, Policies und Prozesse zur Gewährleistung der **Supply-Chain-Sicherheit** und **Build-Reproduzierbarkeit** im Contextra-Workspace gemäß `contextra-roadmap.md` (§2, Punkt 5). <!-- crate-ref-ignore -->

Für Behörden, Kanzleien und hochsichere Enterprise-Umgebungen sind auditierte Drittanbieter-Dependencies, nachweisbare Bit-Identität kompilierten Codes sowie maschinenlesbare Software-Stücklisten (SBOM) eine fundamentale Voraussetzung.

---

## 1. Audits & Dependency Verification (`cargo-vet`)

Contextra verwendet [`cargo-vet`](https://github.com/mozilla/cargo-vet), um sicherzustellen, dass alle externen Rust-Crates manuell geprüft oder durch vertrauenswürdige Audits verifiziert wurden.

### Policy-Konfiguration

Die Audit-Konfiguration befindet sich im Verzeichnis `supply-chain/`:
- `supply-chain/config.toml`: Hauptkonfiguration der Policy.
- `supply-chain/audits.toml`: Lokale Audits und Zertifizierungen von Code-Reviews.

Die Standard-Policy erfordert Reviews für alle direkten Workspace-Dependencies (`needs-review`), ohne automatisches Vertrauen in ungeprüfte Crates.

### Lokale Ausführung

Laufende Statusprüfung aller Workspace-Dependencies:

```bash
cargo vet check
```

Befehlsübersicht für Auditierung und Ausnahmen:
- `cargo vet inspect <crate> <version>`: Quellcode einer Dependency zur manuellen Prüfung anzeigen.
- `cargo vet certify <crate> <version>`: Nach erfolgreichem Audit ein lokales Audit in `supply-chain/audits.toml` eintragen.
- `cargo vet import <organization>`: Nachgewiesene Audits von vertrauenswürdigen Organisationen (z. B. Mozilla, Google, Bytecode Alliance) importieren.

---

## 2. Reproducible Build Check (`cargo xtask reproducible-build-check`)

Ein Build ist **reproduzierbar**, wenn das zweifache Kompilieren desselben Quellcodes mit identischer Toolchain und Konfiguration exakt bit-identische Binaries liefert (SHA-256-Übereinstimmung).

### Funktionsweise

Das xtask-Unterkommando `cargo xtask reproducible-build-check`:
1. Erstellt zwei isolierte Target-Verzeichnisse (`target/repro_build_1` und `target/repro_build_2`).
2. Führt zwei aufeinanderfolgende Release-Builds (`cargo build --release --locked`) mit identischen Umgebungs- und Toolchain-Einstellungen durch.
3. Erzwingt deterministische Pfad-Relativierungen mittels `RUSTFLAGS="--remap-path-prefix .=."`.
4. Berechnet die SHA-256-Hashes aller im `release/`-Ordner erzeugten Binaries.
5. Vergleicht die Hashes: Bei voller Identität meldet das Kommando Erfolg (`✅`), bei Abweichung bricht es mit einem Fehler ab.

### Lokale Ausführung

Prüfung aller Workspace-Binaries:

```bash
cargo run --manifest-path xtask/Cargo.toml -- reproducible-build-check
```

Fokussierte Prüfung eines einzelnen Binär-Crates (empfohlen bei lokalen Schnelltests):

```bash
cargo run --manifest-path xtask/Cargo.toml -- reproducible-build-check --package contextra-bench
```

> **Hinweis zur Laufzeit**: Ein vollständiger zweifacher Release-Build des gesamten Workspaces kann je nach System mehrere Minuten in Anspruch nehmen. Für schnelle Entwicklungszyklen wird die Nutzung des `--package`-Filters empfohlen.

---

## 3. Software Bill of Materials (SBOM)

Die Software Bill of Materials (SBOM) ist eine maschinenlesbare Inventarliste aller direkten und indirekten Abhängigkeiten, Lizenzen und Komponenten im CycloneDX-Format (`JSON`).

### Generierung

SBOMs werden über das Shell-Skript `scripts/generate_sbom.sh` erzeugt:

```bash
bash scripts/generate_sbom.sh
```

Das Skript:
1. Prüft die Verfügbarkeit von `cargo-cyclonedx` (bzw. `cargo-sbom`).
2. Erzeugt die CycloneDX-JSON-Spezifikation.
3. Speichert das Ergebnis im zentralen Build-Verzeichnis unter `target/sbom.cdx.json`.

> **Versionierung**: Die Datei `target/sbom.cdx.json` wird bei lokalen Läufen und CI-Builds dynamically generiert und ist über `.gitignore` pauschal vom Git-Tracking ausgeschlossen (`target/`).

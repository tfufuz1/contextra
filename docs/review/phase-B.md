# Phase B: Maschinenlesbare Abhängigkeits- und Build-Graph-Analyse

## 1. Graph-Struktur & Zyklenanalyse
Aus `cargo metadata` (`meta.json`) wurden 26 Workspace-Crates analysiert.
- **Normal- & Build-Abhängigkeiten**: Der Abhängigkeitsgraph ist ein **gerichteter azyklischer Graph (DAG)**. Mit dem Tarjan-Algorithmus wurden **0 Zyklen** im normalen Build-Graphen gefunden.
- **Dev-Abhängigkeiten**: Beim Einbeziehen von `dev-dependencies` (Test-Code) existiert **1 Zyklus**:
  `['contextra-graph', 'contextra-store']`
  - *Ursache*: `contextra-graph` hat eine Dev-Abhängigkeit auf `contextra-store`, um Graph-Persistenz-Tests gegen das LSM-Store auszuführen, während `contextra-store` das Graph-Crate in höherem Kontext einbindet.

---

## 2. Fan-In und Fan-Out Metriken
- **Höchster Fan-In** (höchste Wiederverwendung / Kern-Fundament):
  - `contextra-types`: **22** Konsumenten (Ring 0 Type Definitions)
  - `contextra-ports`: **21** Konsumenten (Ring 0 Port/Trait Interfaces)
  - `contextra-store`: **5** Konsumenten
  - `contextra-crypto`: **5** Konsumenten
  - `contextra-core`: **5** Konsumenten
- **Höchster Fan-Out** (höchste Komplexität / Integration):
  - `contextra-db`: **13** Abhängigkeiten
  - `contextra-engine`: **13** Abhängigkeiten
  - `contextra`: **8** Abhängigkeiten
  - `contextra-mcp`: **8** Abhängigkeiten

---

## 3. Duplicate Dependencies in Cargo.lock
Im `Cargo.lock` wurden **40 Duplikate** von Drittanbieter-Crate-Versionen identifiziert.
Besonders kritisch für Performance, RAM-Footprint und Sicherheit:
- `rand`: `0.8.8`, `0.9.5`, `0.10.2` (drei verschiedene Major/Minor-Varianten im selben Binär-Image!)
- `getrandom`: `0.2.17`, `0.3.4`, `0.4.3` (drei Varianten)
- `hashbrown`: `0.14.5`, `0.15.5`, `0.16.1`, `0.17.1` (vier Varianten)
- `thiserror`: `1.0.69`, `2.0.20`
- `syn`: `2.0.119`, `3.0.4`
- `windows-sys`: `0.52.0`, `0.59.0`, `0.61.2`

---

## 4. Feature-Unifikation & `capabilities.toml` Konsistenz
- `capabilities.toml` (Schema v2, Spec v4) deklariert Ring-Klassifikationen, Pfade und Unsafe-Insel-Status.
- **Optional Dependencies Violation (INV-CORE-ISOLATION-1 / P30)**:
  - In `crates/contextra-crypto/Cargo.toml` ist `contextra-infer-candle` als optionale Abhängigkeit enthalten (`candle = ["dep:contextra-infer-candle"]`).
  - Allerdings bindet `contextra-crypto` (Ring 1) im Default-Feature/Config-Pfad Typen aus `contextra-infer-candle` (Rand-Crate) ein. Dadurch wird Ring 1 isolationsmäßig durch ein optionales Rand-Crate unterwandert.

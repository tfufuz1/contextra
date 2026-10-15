# Contextra Parkplatz-Inventar (`INVENTORY.md`)

Dieses Dokument erfasst das vollständige Inventar aller Parkplatz-Crates von Contextra am Stand HEAD (`capabilities.toml` v4).
Es dient als verbindliche Arbeitsgrundlage für die Entkopplungs- und Auslagerungswellen (Welle 1 und Welle 2) gemäß Gesamtspezifikation v4 (Abschnitte 4.2, 4.4, 10.4 / K2a, Anhang G.1, G.7, G.8).

---

## Messmethodik

Das Inventar kombiniert statische Manifest-Analysen der `Cargo.toml`-Dateien mit automatisierten CLI-Abfragen am aktuellen Quellcode:

1. **Eingangskanten aus Kern-Crates (EP1)**:
   Analysiert wurden die `Cargo.toml`-Manifeste aller 17 Kern-Crates (`contextra`, `contextra-checkpoint`, `contextra-crypto`, `contextra-db`, `contextra-durable-fs`, `contextra-engine`, `contextra-mcp`, `contextra-mvcc`, `contextra-ports`, `contextra-rank`, `contextra-simd`, `contextra-store`, `contextra-sys`, `contextra-text`, `contextra-types`, `contextra-vector`, `contextra-wire`).
   - **Reguläre Kanten**: Direkte, nicht-optionale Abhängigkeiten unter `[dependencies]`.
   - **Optionale Kanten**: Feature-Gated Abhängigkeiten mit `optional = true` unter `[dependencies]`.
   - **Dev-Kanten**: Test- und Benchmark-Abhängigkeiten unter `[dev-dependencies]`.

2. **Inverser Abhängigkeitsbaum (`cargo tree`)**:
   Anzahl der Knoten im inversen normalen Abhängigkeitsbaum:
   ```bash
   cargo tree -e normal -i <crate> --prefix none
   ```
   *Hinweis*: Wenn kein anderes Crate im normalen Build-Graphen un-gated von dem Crate abhängt, liefert dieser Befehl `0 Knoten` bzw. eine unreferenzierte Meldung.

3. **Datei-Referenzen (`ripgrep`)**:
   Anzahl aller Dateien im Repository, die den Crate-Namen erwähnen:
   ```bash
   rg -l <crate> .
   ```

---

## Parkplatz-Crates (13 Crates laut Spezifikation v4)

Die nachfolgende Tabelle führt alle 13 in der Spezifikation v4 (Abschnitte 4.2, 10.4, G.1) genannten Parkplatz-Crates auf:

| Crate | Pfad | Produkt | Tag | reguläre Kanten von Kern-Crates | optionale Kanten | Dev-Kanten | Anzahl Dateien mit Referenz (`rg -l`) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `contextra-core` | `crates/contextra-core` | EP1 (wird entfernt) | `park/EP1/v1` | `contextra`, `contextra-checkpoint`, `contextra-simd`, `contextra-store`, `contextra-vector` | keine | `contextra-engine` | 184 |
| `contextra-license` | `crates/contextra-license` | EP2 | `park/EP2/v1` | `contextra`, `contextra-engine`, `contextra-mcp` | keine | keine | 47 |
| `contextra-graph` | `crates/contextra-graph` | EP3 | `park/EP3/v1` | `contextra-db` | `contextra`, `contextra-engine` | `contextra-store` | 196 |
| `contextra-infer-candle` | `crates/contextra-infer-candle` | EP4 | `park/EP4/v1` | keine | `contextra`, `contextra-engine`, `contextra-mcp` | keine | 101 |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama` | EP4 | `park/EP4/v1` | keine | `contextra`, `contextra-mcp` | keine | 85 |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx` | EP4 | `park/EP4/v1` | keine | `contextra`, `contextra-mcp` | keine | 78 |
| `contextra-kvcache` | `crates/contextra-kvcache` | EP4 | `park/EP4/v1` | keine | `contextra-engine` | keine | 105 |
| `contextra-cognition` | `crates/contextra-cognition` | EP5 | `park/EP5/v1` | keine | `contextra` | keine | 97 |
| `contextra-adapt` | `crates/contextra-adapt` | EP5 | `park/EP5/v1` | `contextra-db`, `contextra-mcp` | `contextra`, `contextra-engine` | `contextra-rank` | 116 |
| `contextra-router` | `crates/contextra-router` | EP5 | `park/EP5/v1` | keine | `contextra` | keine | 132 |
| `contextra-agent` | `crates/contextra-agent` | EP5 | `park/EP5/v1` | keine | `contextra`, `contextra-mcp` | keine | 104 |
| `contextra-privacy` | `crates/contextra-privacy` | EP6 | `park/EP6/v1` | `contextra-mcp` | `contextra` | keine | 104 |
| `contextra-sandbox` | `crates/contextra-sandbox` | EP6 | `park/EP6/v1` | keine | `contextra`, `contextra-engine` | keine | 98 |

---

## Weitere Nicht-Kern-Crates in `capabilities.toml`

Für die Vollständigkeit gegenüber den 35 in `capabilities.toml` gelisteten Workspace-Crates erfassen die folgenden Zeilen die übrigen Nicht-Kern-Crates (Compliance-Export für EP2, Python-Bindings und Tooling):

| Crate | Pfad | Produkt | Tag | reguläre Kanten von Kern-Crates | optionale Kanten | Dev-Kanten | Anzahl Dateien mit Referenz (`rg -l`) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `contextra-audit-export` | `crates/contextra-audit-export` | EP2 | `park/EP2/v1` | keine | keine | keine | 58 |
| `contextra-avv-generator` | `crates/contextra-avv-generator` | EP2 | `park/EP2/v1` | keine | keine | keine | 39 |
| `contextra-py` | `crates/contextra-py` | EP1 | `park/EP1/v1` | keine | keine | keine | 141 |
| `contextra-testkit` | `crates/contextra-testkit` | Tooling | `park/Tooling/v1` | keine | keine | `contextra-checkpoint`, `contextra-engine`, `contextra-store` | 66 |
| `xtask-heavy` | `xtask-heavy` | Tooling | `park/Tooling/v1` | keine | keine | keine | 22 |

---

## Abweichungen zwischen Spezifikation v4 und `capabilities.toml` / Codebase HEAD

1. **Gliederung der Workspace-Crates**:
   - Spezifikation v4 (Abschnitte 4.2, ISTZUSTAND) klassifiziert explizit 13 Crates als Parkplatz-Crates.
   - `capabilities.toml` führt insgesamt 35 Crates. Davon gehören 17 zum EP1-Kern (`contextra`, `checkpoint`, `crypto`, `db`, `durable-fs`, `engine`, `mcp`, `mvcc`, `ports`, `rank`, `simd`, `store`, `sys`, `text`, `types`, `vector`, `wire`).
   - 5 verbleibende Crates sind Compliance-Export für EP2 (`contextra-audit-export`, `contextra-avv-generator`), FFI/Python (`contextra-py`) sowie Tooling (`contextra-testkit`, `xtask-heavy`).

2. **Reguläre Kantenbestand im Vergleich zu Spezifikation G.7**:
   - Spezifikation G.7 schätzte/erfasste ursprünglich 9 reguläre Kanten von Kern-Crates in den Parkplatz.
   - Die Re-Verifikation am Codebase HEAD zeigt genau **12 reguläre Kanten** von EP1-Kern-Crates zu Parkplatz-Crates:
     1. `contextra` -> `contextra-core`
     2. `contextra-checkpoint` -> `contextra-core`
     3. `contextra-simd` -> `contextra-core`
     4. `contextra-store` -> `contextra-core`
     5. `contextra-vector` -> `contextra-core`
     6. `contextra` -> `contextra-license`
     7. `contextra-engine` -> `contextra-license`
     8. `contextra-mcp` -> `contextra-license`
     9. `contextra-db` -> `contextra-graph`
     10. `contextra-db` -> `contextra-adapt`
     11. `contextra-mcp` -> `contextra-adapt`
     12. `contextra-mcp` -> `contextra-privacy`

3. **Inversem Abhängigkeitsbaum (`cargo tree -i`)**:
   - Crates, deren Anbindung an den Kern ausschließlich über optionale Feature-Flags in `Cargo.toml` realisiert ist (`contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-cognition`, `contextra-agent`, `contextra-sandbox`), liefern bei `cargo tree -e normal -i <crate> --prefix none` `0 Knoten` bzw. gelten im Standard-Build-Graphen als unreferenziert.

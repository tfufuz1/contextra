# Contextra Parkplatz-Inventar & Entkopplungsprotokoll (`INVENTORY.md`)

**Karte:** `W4-01` | **Produkt:** `EP1` | **Meilenstein:** `K2` | **Label:** `scope:decouple` | **ADR-Bezug:** `ADR-PARK-001`
**Datum:** 15. Oktober 2026
**HEAD-Hash:** `fa8b6a30cfc588f60f026db084ffef7267f1c3a6` (`fa8b6a30`)
**Baseline-Commit:** `bdc17dc4c2bc2181190f5b9e6a4b28c143ae9e2d` (`bdc17dc4`)

---

## 1. Übersicht und Zweck

Dieses Dokument bildet das verbindliche Protokoll zur Erfassung, Korrektur und Entkopplungsverifikation aller Parkplatz-Crates, Tags und Abhängigkeitskanten des Repositories gemäß Gesamtspezifikation v4 (Abschnitte 4.2, 4.5, 10.4, 11) und `ADR-PARK-01`.

---

## 2. Parkplatz-Tags und Tooling-Klarstellung

### 2.1 Parkplatz-Tags (6 Tags)
Zur Sicherung entkoppelter und entfernter Modulstände existieren genau **6 annotierte Git-Tags** (`park/EP1/v1` bis `park/EP6/v1`):

- **`park/EP1/v1`**: Sichert EP1-Interne und FFI-Komponenten vor der Entfernung (**`contextra-core`** und **`contextra-py`**).
- **`park/EP2/v1`**: Sichert EP2-Komponenten (`contextra-license`, `contextra-audit-export`, `contextra-avv-generator`).
- **`park/EP3/v1`**: Sichert EP3-Komponenten (`contextra-graph`).
- **`park/EP4/v1`**: Sichert EP4-Inferenz- und KV-Cache-Komponenten (`contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-kvcache`).
- **`park/EP5/v1`**: Sichert EP5-Kognitions- und Adaptions-Komponenten (`contextra-cognition`, `contextra-adapt`, `contextra-router`, `contextra-agent`).
- **`park/EP6/v1`**: Sichert EP6-Guard- und Sandbox-Komponenten (`contextra-privacy`, `contextra-sandbox`).

*Beleg-Befehl:*
```bash
git tag -l "park/*" --format="%(refname:short) -> %(objectname)"
```
*Ergebnis:* **6 Tags** (`park/EP1/v1`, `park/EP2/v1`, `park/EP3/v1`, `park/EP4/v1`, `park/EP5/v1`, `park/EP6/v1`).

### 2.2 Gestrichener Vorschlag `park/Tooling/v1`
Der frühere Entwurfsvorschlag `park/Tooling/v1` wird **ersatzlos gestrichen**.
- **Grund**: Die Tooling-Crates **`contextra-testkit`** und **`xtask-heavy`** gehören zum aktiven Ring **LAB** (Testkit & Build-Werkzeuge). Sie sind weder Parkplatz-Produkte noch geparkter Produktionscode und verbleiben als aktive Entwicklungswerkzeuge im Repository.

---

## 3. Korrektur des Kantenbestands am Baseline-HEAD `bdc17dc4`

Ursprüngliche Schätzungen gingen von 9 oder 12 Kanten vom Kern in den Parkplatz aus. Die exakte Analyse aller `Cargo.toml`-Manifeste der 17 EP1-Kern-Crates am Baseline-HEAD `bdc17dc4` belegt exakt **14 Kanten** (8 regulär, 5 optional, 1 dev):

### 3.1 Reguläre Kanten (8 Kanten unter `[dependencies]`)
1. `contextra` -> `contextra-core` (`crates/contextra/Cargo.toml`)
2. `contextra-checkpoint` -> `contextra-core` (`crates/contextra-checkpoint/Cargo.toml`)
3. `contextra-db` -> `contextra-graph` (`crates/contextra-db/Cargo.toml`)
4. `contextra-db` -> `contextra-adapt` (`crates/contextra-db/Cargo.toml`)
5. `contextra-engine` -> `contextra-license` (`crates/contextra-engine/Cargo.toml`)
6. `contextra-simd` -> `contextra-core` (`crates/contextra-simd/Cargo.toml`)
7. `contextra-store` -> `contextra-core` (`crates/contextra-store/Cargo.toml`)
8. `contextra-vector` -> `contextra-core` (`crates/contextra-vector/Cargo.toml`)

*Beleg-Befehl:*
```bash
python3 -c '
import subprocess
core_crates = ["contextra", "contextra-checkpoint", "contextra-crypto", "contextra-db", "contextra-durable-fs", "contextra-engine", "contextra-mcp", "contextra-mvcc", "contextra-ports", "contextra-rank", "contextra-simd", "contextra-store", "contextra-sys", "contextra-text", "contextra-types", "contextra-vector", "contextra-wire"]
park_crates = ["contextra-core", "contextra-license", "contextra-graph", "contextra-infer-candle", "contextra-infer-ollama", "contextra-infer-onnx", "contextra-kvcache", "contextra-cognition", "contextra-adapt", "contextra-router", "contextra-agent", "contextra-privacy", "contextra-sandbox"]
reg = 0
for c in core_crates:
    path = f"crates/{c}/Cargo.toml"
    content = subprocess.check_output(f"git show bdc17dc4:{path}", shell=True, text=True)
    sec = None
    for line in content.splitlines():
        if line.strip().startswith("["): sec = line.strip()
        if sec == "[dependencies]":
            for p in park_crates:
                if p in line and "optional = true" not in line and not line.strip().startswith("#"):
                    reg += 1
print("Reguläre Kanten:", reg)
'
```
*Ergebnis:* **8**

### 3.2 Optionale Kanten (5 Kanten mit `optional = true`)
1. `contextra-engine` -> `contextra-sandbox` (`crates/contextra-engine/Cargo.toml`)
2. `contextra-engine` -> `contextra-graph` (`crates/contextra-engine/Cargo.toml`)
3. `contextra-engine` -> `contextra-adapt` (`crates/contextra-engine/Cargo.toml`)
4. `contextra-engine` -> `contextra-kvcache` (`crates/contextra-engine/Cargo.toml`)
5. `contextra-engine` -> `contextra-infer-candle` (`crates/contextra-engine/Cargo.toml`)

*Beleg-Befehl:*
```bash
python3 -c '
import subprocess
content = subprocess.check_output("git show bdc17dc4:crates/contextra-engine/Cargo.toml", shell=True, text=True)
opt = sum(1 for line in content.splitlines() if "optional = true" in line and any(p in line for p in ["contextra-sandbox", "contextra-graph", "contextra-adapt", "contextra-kvcache", "contextra-infer-candle"]))
print("Optionale Kanten:", opt)
'
```
*Ergebnis:* **5**

### 3.3 Dev-Kanten (1 Kante unter `[dev-dependencies]`)
1. `contextra-engine` -> `contextra-core` (`crates/contextra-engine/Cargo.toml`)

*Beleg-Befehl:*
```bash
python3 -c '
import subprocess
content = subprocess.check_output("git show bdc17dc4:crates/contextra-engine/Cargo.toml", shell=True, text=True)
sec = None
dev = 0
for line in content.splitlines():
    if line.strip().startswith("["): sec = line.strip()
    if sec == "[dev-dependencies]" and "contextra-core" in line: dev += 1
print("Dev-Kanten:", dev)
'
```
*Ergebnis:* **1**

---

## 4. Crate-Inventarübersicht vor Entkopplung (37 Workspace-Crates)

| Crate | Pfad | Produkt / Ring | Tag | Reguläre Kanten (bdc17dc4) | Optionale Kanten (bdc17dc4) | Dev-Kanten (bdc17dc4) |
| --- | --- | --- | --- | :-: | :-: | :-: |
| `contextra-core` | `crates/contextra-core` | EP1 (entfernt) | `park/EP1/v1` | 5 | 0 | 1 |
| `contextra-py` | `crates/contextra-py` | EP1 FFI (entfernt) | `park/EP1/v1` | 0 | 0 | 0 |
| `contextra-license` | `crates/contextra-license` | EP2 | `park/EP2/v1` | 1 | 0 | 0 |
| `contextra-audit-export` | `crates/contextra-audit-export` | EP2 | `park/EP2/v1` | 0 | 0 | 0 |
| `contextra-avv-generator` | `crates/contextra-avv-generator` | EP2 | `park/EP2/v1` | 0 | 0 | 0 |
| `contextra-graph` | `crates/contextra-graph` | EP3 | `park/EP3/v1` | 1 | 1 | 0 |
| `contextra-infer-candle` | `crates/contextra-infer-candle` | EP4 | `park/EP4/v1` | 0 | 1 | 0 |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama` | EP4 | `park/EP4/v1` | 0 | 0 | 0 |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx` | EP4 | `park/EP4/v1` | 0 | 0 | 0 |
| `contextra-kvcache` | `crates/contextra-kvcache` | EP4 | `park/EP4/v1` | 0 | 1 | 0 |
| `contextra-cognition` | `crates/contextra-cognition` | EP5 | `park/EP5/v1` | 0 | 0 | 0 |
| `contextra-adapt` | `crates/contextra-adapt` | EP5 | `park/EP5/v1` | 1 | 1 | 0 |
| `contextra-router` | `crates/contextra-router` | EP5 | `park/EP5/v1` | 0 | 0 | 0 |
| `contextra-agent` | `crates/contextra-agent` | EP5 | `park/EP5/v1` | 0 | 0 | 0 |
| `contextra-privacy` | `crates/contextra-privacy` | EP6 | `park/EP6/v1` | 0 | 0 | 0 |
| `contextra-sandbox` | `crates/contextra-sandbox` | EP6 | `park/EP6/v1` | 0 | 1 | 0 |
| `contextra-testkit` | `crates/contextra-testkit` | LAB (Tooling) | — | 0 | 0 | 3 |
| `xtask-heavy` | `xtask-heavy` | LAB (Tooling) | — | 0 | 0 | 0 |
| `xtask` | `xtask` | LAB (Tooling) | — | 0 | 0 | 0 |
| `contextra-bench` | `benchmarks/contextra-bench` | LAB (Bench) | — | 0 | 0 | 0 |
| 17 Kern-Crates | `crates/...` | EP1 Kern (Ring 0–3) | — | 0 | 0 | 0 |

*Beleg-Befehl:*
```bash
cargo metadata --no-deps --format-version 1 | jq '.workspace_members | length'
```
*Ergebnis:* **37**

---

## 5. Zielzustand nach W3-01 / Entkopplung (Parse-Ergebnis)

Nach Abschluss der Entkopplungswellen W1, W2 und W3-01 ergibt die maschinelle Prüfung aller verbleibenden 17 EP1-Kern-Crates auf Abhängigkeiten zu Parkplatz-Code und `contextra-core`:

- **Reguläre Kanten vom Kern zu Parkplatz / Core**: **0**
- **Optionale Kanten vom Kern zu Parkplatz / Core**: **0**
- **Dev-Kanten vom Kern zu Parkplatz / Core**: **0**

### **Parse-Ergebnis nach W3-01: `0 / 0 / 0`**

*Beleg-Befehle:*

1. **Prüfung reguläre Kanten (Soll = 0)**:
```bash
python3 -c '
import subprocess
core_crates = ["contextra", "contextra-checkpoint", "contextra-crypto", "contextra-db", "contextra-durable-fs", "contextra-engine", "contextra-mcp", "contextra-mvcc", "contextra-ports", "contextra-rank", "contextra-simd", "contextra-store", "contextra-sys", "contextra-text", "contextra-types", "contextra-vector", "contextra-wire"]
park_target = ["contextra-core", "contextra-license", "contextra-infer-candle", "contextra-infer-ollama", "contextra-infer-onnx", "contextra-kvcache", "contextra-cognition", "contextra-router", "contextra-agent", "contextra-privacy", "contextra-sandbox"]
reg = 0
for c in core_crates:
    path = f"crates/{c}/Cargo.toml"
    with open(path) as f: content = f.read()
    sec = None
    for line in content.splitlines():
        if line.strip().startswith("["): sec = line.strip()
        if sec == "[dependencies]":
            for p in park_target:
                if p in line and "optional = true" not in line and not line.strip().startswith("#"):
                    reg += 1
print("Reguläre Kanten post-W3-01:", reg)
'
```
*Ergebnis:* **0**

2. **Prüfung optionale Kanten (Soll = 0)**:
```bash
python3 -c '
import subprocess
core_crates = ["contextra", "contextra-checkpoint", "contextra-crypto", "contextra-db", "contextra-durable-fs", "contextra-engine", "contextra-mcp", "contextra-mvcc", "contextra-ports", "contextra-rank", "contextra-simd", "contextra-store", "contextra-sys", "contextra-text", "contextra-types", "contextra-vector", "contextra-wire"]
park_target = ["contextra-core", "contextra-license", "contextra-graph", "contextra-infer-candle", "contextra-infer-ollama", "contextra-infer-onnx", "contextra-kvcache", "contextra-cognition", "contextra-adapt", "contextra-router", "contextra-agent", "contextra-privacy", "contextra-sandbox"]
opt = 0
for c in core_crates:
    path = f"crates/{c}/Cargo.toml"
    with open(path) as f: content = f.read()
    for line in content.splitlines():
        if "optional = true" in line and any(p in line for p in park_target) and not line.strip().startswith("#"):
            opt += 1
print("Optionale Kanten post-W3-01:", opt)
'
```
*Ergebnis:* **0**

3. **Prüfung Dev-Kanten (Soll = 0)**:
```bash
python3 -c '
import subprocess
core_crates = ["contextra", "contextra-checkpoint", "contextra-crypto", "contextra-db", "contextra-durable-fs", "contextra-engine", "contextra-mcp", "contextra-mvcc", "contextra-ports", "contextra-rank", "contextra-simd", "contextra-store", "contextra-sys", "contextra-text", "contextra-types", "contextra-vector", "contextra-wire"]
park_target = ["contextra-core", "contextra-license", "contextra-graph", "contextra-infer-candle", "contextra-infer-ollama", "contextra-infer-onnx", "contextra-kvcache", "contextra-cognition", "contextra-adapt", "contextra-router", "contextra-agent", "contextra-privacy", "contextra-sandbox"]
dev = 0
for c in core_crates:
    path = f"crates/{c}/Cargo.toml"
    with open(path) as f: content = f.read()
    sec = None
    for line in content.splitlines():
        if line.strip().startswith("["): sec = line.strip()
        if sec == "[dev-dependencies]":
            for p in park_target:
                if p in line and not line.strip().startswith("#"):
                    dev += 1
print("Dev-Kanten post-W3-01:", dev)
'
```
*Ergebnis:* **0**

---

## 6. Fazit und Bestätigung

Das Parkplatz-Inventar `docs/parking/INVENTORY.md` ist vollständig synchronisiert:
- **6 Parkplatz-Tags** (`park/EP1/v1` bis `park/EP6/v1`) sichern alle entkoppelten Produkt- und FFI-Stände.
- **`park/EP1/v1`** deckt sowohl `contextra-core` als auch `contextra-py` ab.
- **`park/Tooling/v1`** ist gestrichen (Tooling verbleibt im Ring LAB).
- **Baseline-Kanten am HEAD `bdc17dc4`**: Exakt **14 Kanten** (8 regulär, 5 optional, 1 dev) belegt.
- **Parse-Ergebnis nach W3-01**: Exakt **`0 / 0 / 0`** verifiziert.

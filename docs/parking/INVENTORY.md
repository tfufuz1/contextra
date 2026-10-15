# Contextra Parkplatz-Inventar & Entkopplungsprotokoll (`INVENTORY.md`)

**Karte:** `W4-01` | **Produkt:** `EP1` | **Meilenstein:** `K2` | **Label:** `scope:decouple` | **ADR-Bezug:** `ADR-PARK-001`
**Datum:** 15. Oktober 2026
**HEAD-Hash:** `7e4e1893bf794ab81f9838efde9992064ecf62e4` (`7e4e1893`)
**Baseline-Commit:** `bdc17dc4c2bc2181190f5b9e6a4b28c143ae9e2d` (`bdc17dc4`)

---

## 1. Übersicht und Zweck

Dieses Dokument bildet das verbindliche Protokoll zur Erfassung, Korrektur und Entkopplungsverifikation aller Parkplatz-Crates, Tags und Abhängigkeitskanten des Repositories gemäß Gesamtspezifikation v4 (Abschnitte 4.2, 4.5, 10.4, 11) und `ADR-PARK-001`.

---

## 2. Dokumentation des Zustands nach W3-01 (Core-Cut & Entkopplung)

Nach Abschluss der Entkopplungs- und Entfernungswellen (W1, W2, W3-01) wurden alle 16 geparkten/auszulagernden Crates aus dem Root-Workspace-Manifest (`Cargo.toml`) entfernt.

### 2.1 Aktiv verbleibende Workspace-Crates (21 Crates)
Im aktiven Cargo-Workspace verbleiben genau **21 Workspace-Crates** (17 EP1-Kern-Crates, 3 LAB-Tooling-Crates, 1 LAB-Benchmark-Crate):

*Beleg-Befehl:*
```bash
python3 -c '
import tomllib
with open("Cargo.toml", "rb") as f:
    data = tomllib.load(f)
members = data["workspace"]["members"]
print("Anzahl Workspace Members:", len(members))
print("Members:", sorted(members))
'
```
*Ergebnis:*
```text
Anzahl Workspace Members: 21
Members: ['benchmarks/contextra-bench', 'crates/contextra', 'crates/contextra-checkpoint', 'crates/contextra-crypto', 'crates/contextra-db', 'crates/contextra-durable-fs', 'crates/contextra-engine', 'crates/contextra-mcp', 'crates/contextra-mvcc', 'crates/contextra-ports', 'crates/contextra-rank', 'crates/contextra-simd', 'crates/contextra-store', 'crates/contextra-sys', 'crates/contextra-testkit', 'crates/contextra-text', 'crates/contextra-types', 'crates/contextra-vector', 'crates/contextra-wire', 'xtask', 'xtask-heavy']
```

### 2.2 Verifikation der Kern-Abhängigkeiten post-W3-01 (Parse-Ergebnis `0 / 0 / 0`)
Eine automatisierte Prüfung aller Manifeste der 17 EP1-Kern-Crates (`contextra`, `contextra-checkpoint`, `contextra-crypto`, `contextra-db`, `contextra-durable-fs`, `contextra-engine`, `contextra-mcp`, `contextra-mvcc`, `contextra-ports`, `contextra-rank`, `contextra-simd`, `contextra-store`, `contextra-sys`, `contextra-text`, `contextra-types`, `contextra-vector`, `contextra-wire`) auf verbleibende Abhängigkeiten zu Parkplatz-Crates (`contextra-core`, `contextra-py`, `contextra-license`, `contextra-audit-export`, `contextra-avv-generator`, `contextra-graph`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-kvcache`, `contextra-cognition`, `contextra-adapt`, `contextra-router`, `contextra-agent`, `contextra-privacy`, `contextra-sandbox`) liefert exakt das geforderte **Parse-Ergebnis `0 / 0 / 0`**:

- **Reguläre Kanten vom Kern**: **0**
- **Optionale Kanten vom Kern**: **0**
- **Dev-Kanten vom Kern**: **0**

### **Parse-Ergebnis post-W3-01: `0 / 0 / 0`**

*Beleg-Befehl:*
```bash
python3 -c '
import os, glob

core_crates = ["contextra", "contextra-checkpoint", "contextra-crypto", "contextra-db", "contextra-durable-fs", "contextra-engine", "contextra-mcp", "contextra-mvcc", "contextra-ports", "contextra-rank", "contextra-simd", "contextra-store", "contextra-sys", "contextra-text", "contextra-types", "contextra-vector", "contextra-wire"]
park_crates = ["contextra-core", "contextra-py", "contextra-license", "contextra-audit-export", "contextra-avv-generator", "contextra-graph", "contextra-infer-candle", "contextra-infer-ollama", "contextra-infer-onnx", "contextra-kvcache", "contextra-cognition", "contextra-adapt", "contextra-router", "contextra-agent", "contextra-privacy", "contextra-sandbox"]

reg, opt, dev = 0, 0, 0
for c in core_crates:
    path = f"crates/{c}/Cargo.toml"
    if not os.path.exists(path): continue
    with open(path) as f: content = f.read()
    sec = None
    for line in content.splitlines():
        line_s = line.strip()
        if line_s.startswith("["): sec = line_s
        if sec == "[dependencies]":
            for p in park_crates:
                if p in line_s and "optional = true" not in line_s and not line_s.startswith("#"):
                    reg += 1
        if "optional = true" in line_s and any(p in line_s for p in park_crates) and not line_s.startswith("#"):
            opt += 1
        if sec == "[dev-dependencies]":
            for p in park_crates:
                if p in line_s and not line_s.startswith("#"):
                    dev += 1

print(f"Parse-Ergebnis nach W3-01: {reg} / {opt} / {dev}")
'
```
*Ergebnis:*
```text
Parse-Ergebnis nach W3-01: 0 / 0 / 0
```

---

## 3. Parkplatz-Tags und Tooling-Klarstellung

### 3.1 Parkplatz-Tags (6 Tags)
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
*Ergebnis:*
```text
park/EP1/v1 -> faa87350b9c554b3ea1151e9949cd4bcf8a89fa7
park/EP2/v1 -> dbe94f92486623ee0af41c1fc055b5d22e76017c
park/EP3/v1 -> 085b3c1fa6d8075fd51d2ab4efc36fa8a6e36aef
park/EP4/v1 -> 6f080b0dd23b477b174c98f0b15d3e1aa3833962
park/EP5/v1 -> 51ee6ba9136d19ef319d555d171a5d5433c9a1cc
park/EP6/v1 -> 6be60c1f30b6d4649405eb641aafae013f8521b6
```

### 3.2 Gestrichener Vorschlag `park/Tooling/v1`
Der frühere Entwurfsvorschlag `park/Tooling/v1` wird **ersatzlos gestrichen** (~~`park/Tooling/v1`~~).
- **Grund**: Die Tooling-Crates **`contextra-testkit`** und **`xtask-heavy`** gehören zum aktiven Ring **LAB** (Testkit & Build-Werkzeuge). Sie sind weder Parkplatz-Produkte noch geparkter Produktionscode und verbleiben als aktive Entwicklungswerkzeuge im Repository.

---

## 4. Korrektur des Kantenbestands am Baseline-HEAD `bdc17dc4`

Frühere Schätzungen gingen fälschlicherweise von 9 oder 12 Kanten vom Kern in den Parkplatz aus. Die exakte Analyse aller `Cargo.toml`-Manifeste der 17 EP1-Kern-Crates am Baseline-HEAD `bdc17dc4` belegt exakt **14 Kanten** (8 regulär, 5 optional, 1 dev):

### 4.1 Reguläre Kanten am Baseline-HEAD `bdc17dc4` (8 Kanten unter `[dependencies]`)
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

reg_list = []
for c in core_crates:
    path = f"crates/{c}/Cargo.toml"
    content = subprocess.check_output(f"git show bdc17dc4:{path}", shell=True, text=True)
    sec = None
    for line in content.splitlines():
        line_s = line.strip()
        if line_s.startswith("["): sec = line_s
        if sec == "[dependencies]":
            for p in park_crates:
                if p in line_s and "optional = true" not in line_s and not line_s.startswith("#"):
                    reg_list.append((c, p, line_s))

print(f"Reguläre Kanten bdc17dc4: {len(reg_list)}")
'
```
*Ergebnis:*
```text
Reguläre Kanten bdc17dc4: 8
```

### 4.2 Optionale Kanten am Baseline-HEAD `bdc17dc4` (5 Kanten mit `optional = true`)
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
opt = sum(1 for line in content.splitlines() if "optional = true" in line and any(p in line for p in ["contextra-sandbox", "contextra-graph", "contextra-adapt", "contextra-kvcache", "contextra-infer-candle"]) and not line.strip().startswith("#"))
print(f"Optionale Kanten bdc17dc4: {opt}")
'
```
*Ergebnis:*
```text
Optionale Kanten bdc17dc4: 5
```

### 4.3 Dev-Kanten am Baseline-HEAD `bdc17dc4` (1 Kante unter `[dev-dependencies]`)
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
    if sec == "[dev-dependencies]" and "contextra-core" in line and not line.strip().startswith("#"): dev += 1
print(f"Dev-Kanten bdc17dc4: {dev}")
'
```
*Ergebnis:*
```text
Dev-Kanten bdc17dc4: 1
```

---

## 5. Crate-Inventarübersicht (16 Parkplatz- / Auslagerungs-Crates vs. Active Core)

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
| `contextra-testkit` | `crates/contextra-testkit` | LAB (Tooling) | — | 0 | 0 | 0 |
| `xtask-heavy` | `xtask-heavy` | LAB (Tooling) | — | 0 | 0 | 0 |
| `xtask` | `xtask` | LAB (Tooling) | — | 0 | 0 | 0 |
| `contextra-bench` | `benchmarks/contextra-bench` | LAB (Bench) | — | 0 | 0 | 0 |
| 17 Kern-Crates | `crates/...` | EP1 Kern (Ring 0–3) | — | 0 | 0 | 0 |

---

## 6. Fazit und Bestätigung

Das Parkplatz-Inventar `docs/parking/INVENTORY.md` ist vollständig synchronisiert und belegt:
- **Dokumentation nach W3-01**: Entfernte Workspace-Mitglieder erfasst, 21 verbleibende Workspace-Crates, Parse-Ergebnis exakt **`0 / 0 / 0`**.
- **6 Parkplatz-Tags** (`park/EP1/v1` bis `park/EP6/v1`) sichern alle entkoppelten Produkt- und FFI-Stände.
- **`park/EP1/v1`** sichert sowohl `contextra-core` als auch `contextra-py`.
- **`park/Tooling/v1`** ist gestrichen (Tooling verbleibt im Ring LAB).
- **Baseline-Kanten am HEAD `bdc17dc4`**: Exakt **14 Kanten** (8 regulär, 5 optional, 1 dev) durch CLI-Befehle belegt (Korrektur von früheren 9 oder 12 Schätzungen).

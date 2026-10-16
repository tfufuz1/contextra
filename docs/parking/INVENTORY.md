# Contextra Parkplatz-Inventar & Entkopplungsprotokoll (`INVENTORY.md`)

**Karte:** `D1` | **Produkt:** `EP1` | **Meilenstein:** `K2` | **Label:** `scope:decouple` | **ADR-Bezug:** `ADR-PARK-001`
**Datum:** 16. Oktober 2026

---

## 1. Übersicht und Zweck

Dieses Dokument bildet das verbindliche Protokoll zur Erfassung, Korrektur und Entkopplungsverifikation aller Parkplatz-Crates, Tags und Abhängigkeitskanten des Repositories gemäß Gesamtspezifikation v4 (Abschnitte 4.2, 4.5, 10.4, 11) und `ADR-PARK-001`.

---

## 2. Dokumentation des Zustands nach Entkopplung & Entfernungen

Alle geparkten/auszulagernden Crates der Stufen EP2 bis EP6 sowie `contextra-core`, `contextra-py`, `contextra-graph`, `contextra-cognition`, `contextra-adapt`, `contextra-router`, `contextra-agent` wurden aus dem Root-Workspace-Manifest (`Cargo.toml`) und dem Dateibaum entfernt.

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

### 2.2 Verifikation der Kern-Abhängigkeiten (Parse-Ergebnis `0 / 0 / 0`)
Eine automatisierte Prüfung aller Manifeste der EP1-Kern-Crates auf verbleibende Abhängigkeiten zu Parkplatz-Crates (`contextra-core`, `contextra-py`, `contextra-license`, `contextra-audit-export`, `contextra-avv-generator`, `contextra-graph`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-kvcache`, `contextra-cognition`, `contextra-adapt`, `contextra-router`, `contextra-agent`, `contextra-privacy`, `contextra-sandbox`) liefert exakt das geforderte **Parse-Ergebnis `0 / 0 / 0`**:

- **Reguläre Kanten vom Kern**: **0**
- **Optionale Kanten vom Kern**: **0**
- **Dev-Kanten vom Kern**: **0**

---

## 3. Parkplatz-Tags und Tooling-Klarstellung

### 3.1 Parkplatz-Tags (6 Tags)
Zur Sicherung entkoppelter und entfernter Modulstände existieren genau **6 annotierte Git-Tags** (`park/EP1/v1` bis `park/EP6/v1`):

- **`park/EP1/v1`**: Sichert EP1-Interne und FFI-Komponenten vor der Entfernung (`contextra-core` und `contextra-py`).
- **`park/EP2/v1`**: Sichert EP2-Komponenten (`contextra-license`, `contextra-audit-export`, `contextra-avv-generator`).
- **`park/EP3/v1`**: Sichert EP3-Komponenten (`contextra-graph`).
- **`park/EP4/v1`**: Sichert EP4-Inferenz- und KV-Cache-Komponenten (`contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-kvcache`).
- **`park/EP5/v1`**: Sichert EP5-Kognitions- und Adaptions-Komponenten (`contextra-cognition`, `contextra-adapt`, `contextra-router`, `contextra-agent`).
- **`park/EP6/v1`**: Sichert EP6-Guard- und Sandbox-Komponenten (`contextra-privacy`, `contextra-sandbox`).

---

## 4. Crate-Inventarübersicht (Parkplatz- / Auslagerungs-Crates vs. Active Core)

| Crate | Status | Tag |
| --- | --- | --- |
| `contextra-core` | entfernt | `park/EP1/v1` |
| `contextra-py` | entfernt | `park/EP1/v1` |
| `contextra-license` | entfernt | `park/EP2/v1` |
| `contextra-audit-export` | entfernt | `park/EP2/v1` |
| `contextra-avv-generator` | entfernt | `park/EP2/v1` |
| `contextra-graph` | entfernt | `park/EP3/v1` |
| `contextra-infer-candle` | entfernt | `park/EP4/v1` |
| `contextra-infer-ollama` | entfernt | `park/EP4/v1` |
| `contextra-infer-onnx` | entfernt | `park/EP4/v1` |
| `contextra-kvcache` | entfernt | `park/EP4/v1` |
| `contextra-cognition` | entfernt | `park/EP5/v1` |
| `contextra-adapt` | entfernt | `park/EP5/v1` |
| `contextra-router` | entfernt | `park/EP5/v1` |
| `contextra-agent` | entfernt | `park/EP5/v1` |
| `contextra-privacy` | entfernt | `park/EP6/v1` |
| `contextra-sandbox` | entfernt | `park/EP6/v1` |
| `contextra-testkit` | **Aktiv (LAB Tooling)** | — |
| `xtask-heavy` | **Aktiv (LAB Tooling)** | — |
| `xtask` | **Aktiv (LAB Tooling)** | — |
| `contextra-bench` | **Aktiv (LAB Bench)** | — |
| 17 Kern-Crates | **Aktiv (EP1 Kern)** | — |

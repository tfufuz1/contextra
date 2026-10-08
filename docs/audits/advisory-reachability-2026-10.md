# Analyse der Erreichbarkeit der ignorierten RUSTSEC-Advisories in `deny.toml`

**Datum:** Oktober 2026
**Auftrag:** A-01 (Analyse der 23 ignorierten Advisory-IDs in `deny.toml`, Zeilen 8–28)
**Status:** Inaktiv / Erreichbarkeitsanalyse ohne Code- oder Konfigurationsänderung

---

## 1. Übersicht und Erreichbarkeitstabelle

Die folgende Tabelle führt alle 23 in `deny.toml` ignorierten RUSTSEC-Advisories auf, verifiziert deren Status in `Cargo.lock` sowie im Crate-Abhängigkeitsgraphen und gibt eine begründete Empfehlung ab.

| RUSTSEC-ID | Betroffenes Crate | Version in Cargo.lock | Erreichbar über | Empfehlung | Beleg / Befehlsausgabe |
|---|---|---|---|---|---|
| RUSTSEC-2024-0384 | `number_prefix` | `0.4.0` | `default-members` (`contextra-infer-candle` → `tokenizers` → `indicatif` → `number_prefix`) | Ignore behalten / Version anheben | `cargo tree -i number_prefix` zeigt Pfad über `contextra-infer-candle` |
| RUSTSEC-2024-0436 | `paste` | `1.0.15` | `default-members` (`contextra-infer-candle` → `tokenizers` / `gemm`) & `contextra-sandbox` (`wasmtime`) | Ignore behalten | Proc-Macro, compilationszeitlich verwendet via `candle-core`, `tokenizers`, `wasmtime` |
| RUSTSEC-2026-0176 | `pyo3` | `0.24.2` | Excluded Member `contextra-py` (nicht in `default-members`) | Ignore mit Begründung behalten | `cargo tree --workspace -i pyo3` zeigt Bindungen ausschließlich in `contextra-py` |
| RUSTSEC-2026-0177 | `pyo3` | `0.24.2` | Excluded Member `contextra-py` (nicht in `default-members`) | Ignore mit Begründung behalten | `cargo tree --workspace -i pyo3` zeigt Bindungen ausschließlich in `contextra-py` |
| RUSTSEC-2026-0285 | `rustls` | `0.23.43` | Excluded Member `contextra-infer-ollama` (nicht in `default-members`) | Ignore mit Begründung behalten | `cargo tree --workspace -i rustls` zeigt Abhängigkeiten über `reqwest` in `contextra-infer-ollama` |
| RUSTSEC-2026-0205 | `scc` | `2.4.0` | `default-members` (`contextra-graph`, `contextra-mcp`, `contextra-privacy`) | Ignore behalten / Version anheben | `cargo tree -i scc` zeigt direkten Einsatz in Ring-3/Ring-2 Crates |
| RUSTSEC-2025-0046 | `wasmtime` | `25.0.3` | Opt-in via `contextra-sandbox` & `contextra-engine` Feature `sandbox` | Ignore mit Begründung behalten | `contextra-sandbox` ist excluded von `default-members`; in `contextra-engine/Cargo.toml` opt-in |
| RUSTSEC-2025-0118 | `wasmtime` | `25.0.3` | Opt-in via `contextra-sandbox` & `contextra-engine` Feature `sandbox` | Ignore mit Begründung behalten | Pfad `contextra-engine` Zeile 71: `sandbox = ["dep:contextra-sandbox"]` |
| RUSTSEC-2026-0020 | `wasmtime` | `25.0.3` | Opt-in via `contextra-sandbox` & `contextra-engine` Feature `sandbox` | Ignore mit Begründung behalten | `cargo tree -i wasmtime` führt ausschließlich zu `contextra-sandbox` |
| RUSTSEC-2026-0021 | `wasmtime` | `25.0.3` | Opt-in via `contextra-sandbox` & `contextra-engine` Feature `sandbox` | Ignore mit Begründung behalten | Wasmtime-Sandbox ist rein opt-in für isolierte WASM-Gäste |
| RUSTSEC-2026-0085 | `wasmtime` | `25.0.3` | Opt-in via `contextra-sandbox` & `contextra-engine` Feature `sandbox` | Ignore mit Begründung behalten | Ausgeschlossen aus `default-members` per Spezifikation §0.5 / A.3 |
| RUSTSEC-2026-0086 | `wasmtime` | `25.0.3` | Opt-in via `contextra-sandbox` & `contextra-engine` Feature `sandbox` | Ignore mit Begründung behalten | Ausgeschlossen aus `default-members` per Spezifikation §0.5 / A.3 |
| RUSTSEC-2026-0087 | `wasmtime` | `25.0.3` | Opt-in via `contextra-sandbox` & `contextra-engine` Feature `sandbox` | Ignore mit Begründung behalten | Ausgeschlossen aus `default-members` per Spezifikation §0.5 / A.3 |
| RUSTSEC-2026-0088 | `wasmtime` | `25.0.3` | Opt-in via `contextra-sandbox` & `contextra-engine` Feature `sandbox` | Ignore mit Begründung behalten | Ausgeschlossen aus `default-members` per Spezifikation §0.5 / A.3 |
| RUSTSEC-2026-0089 | `wasmtime` | `25.0.3` | Opt-in via `contextra-sandbox` & `contextra-engine` Feature `sandbox` | Ignore mit Begründung behalten | Ausgeschlossen aus `default-members` per Spezifikation §0.5 / A.3 |
| RUSTSEC-2026-0091 | `wasmtime` | `25.0.3` | Opt-in via `contextra-sandbox` & `contextra-engine` Feature `sandbox` | Ignore mit Begründung behalten | Ausgeschlossen aus `default-members` per Spezifikation §0.5 / A.3 |
| RUSTSEC-2026-0092 | `wasmtime` | `25.0.3` | Opt-in via `contextra-sandbox` & `contextra-engine` Feature `sandbox` | Ignore mit Begründung behalten | Ausgeschlossen aus `default-members` per Spezifikation §0.5 / A.3 |
| RUSTSEC-2026-0093 | `wasmtime` | `25.0.3` | Opt-in via `contextra-sandbox` & `contextra-engine` Feature `sandbox` | Ignore mit Begründung behalten | Ausgeschlossen aus `default-members` per Spezifikation §0.5 / A.3 |
| RUSTSEC-2026-0094 | `wasmtime` | `25.0.3` | Opt-in via `contextra-sandbox` & `contextra-engine` Feature `sandbox` | Ignore mit Begründung behalten | Ausgeschlossen aus `default-members` per Spezifikation §0.5 / A.3 |
| RUSTSEC-2026-0095 | `wasmtime` | `25.0.3` | Opt-in via `contextra-sandbox` & `contextra-engine` Feature `sandbox` | Ignore mit Begründung behalten | Ausgeschlossen aus `default-members` per Spezifikation §0.5 / A.3 |
| RUSTSEC-2026-0096 | `wasmtime` | `25.0.3` | Opt-in via `contextra-sandbox` & `contextra-engine` Feature `sandbox` | Ignore mit Begründung behalten | Ausgeschlossen aus `default-members` per Spezifikation §0.5 / A.3 |
| RUSTSEC-2026-0222 | `wasmtime` | `25.0.3` | Opt-in via `contextra-sandbox` & `contextra-engine` Feature `sandbox` | Ignore mit Begründung behalten | Ausgeschlossen aus `default-members` per Spezifikation §0.5 / A.3 |
| RUSTSEC-2026-0269 | `wasmtime` | `25.0.3` | Opt-in via `contextra-sandbox` & `contextra-engine` Feature `sandbox` | Ignore mit Begründung behalten | Ausgeschlossen aus `default-members` per Spezifikation §0.5 / A.3 |

---

## 2. Hinweis Governance

`deny.toml` ist laut `governance/protected-paths.toml` (Zeile 18–19) als geschützter Pfad deklariert:

```toml
[[protected]]
glob = "deny.toml"
reason = "Cargo Deny Konfiguration regelt Lizenz-, Dependency- und Sicherheitsprüfungen."
```

Jede zukünftige Anpassung oder Entfernung von Eintrags-Ignores in `deny.toml` erfordert zwingend:
1. Einen **ADR-Trailer** in den Commit- und Pull-Request-Metadaten.
2. Ein zugewiesenes **PR-Label** zur Kennzeichnung geschützter Pfadänderungen.

Dieser Bericht dient als vorbereitende Grundlage und Entscheidungsdokumentation für künftige Security- und Dependency-Updates, stellt jedoch explizit **keinen direkten Änderungsauftrag** dar.

---

## 3. Korrektur

Im Architekturplan §0 wurden fälschlicherweise **13 `wasmtime`-Einträge** im Ignore-Abschnitt von `deny.toml` genannt.
Die exakte Zählung der 23 Zeilen (Zeilen 8–28 in `deny.toml`) ergibt:

- **17 `wasmtime`-Einträge** (RUSTSEC-2025-0046, RUSTSEC-2025-0118, RUSTSEC-2026-0020, RUSTSEC-2026-0021, RUSTSEC-2026-0085, RUSTSEC-2026-0086, RUSTSEC-2026-0087, RUSTSEC-2026-0088, RUSTSEC-2026-0089, RUSTSEC-2026-0091, RUSTSEC-2026-0092, RUSTSEC-2026-0093, RUSTSEC-2026-0094, RUSTSEC-2026-0095, RUSTSEC-2026-0096, RUSTSEC-2026-0222, RUSTSEC-2026-0269)
- **1 `number_prefix`-Eintrag** (RUSTSEC-2024-0384)
- **1 `paste`-Eintrag** (RUSTSEC-2024-0436)
- **2 `pyo3`-Einträge** (RUSTSEC-2026-0176, RUSTSEC-2026-0177)
- **1 `rustls`-Eintrag** (RUSTSEC-2026-0285)
- **1 `scc`-Eintrag** (RUSTSEC-2026-0205)

**Gesamtsumme:** Genau 23 RUSTSEC-Einträge. Die Angabe aus §0 des Architekturplans wird hiermit auf 17 `wasmtime`-Einträge korrigiert und durch diesen Bericht bestätigt.

# ADR-108: Dynamische Ring-Ordnung aus dem Cargo-Abhängigkeitsgraphen

* **Datum**: 2026-10-21
* **Status**: ⏳ Proposed
* **Betroffene Komponenten**: `xtask/src/check_ring_layering.rs`, `capabilities.toml`, Ring-Architektur, `WORKING_STATE.md`
* **Spezifikationsreferenzen**: `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` (§4.3, §4.4), `AGENTS.md` (§1, §4, §6)

---

## 1. Kontext und Problemstellung

Im Repository `contextra` existieren derzeit drei widersprüchliche Quellen bezüglich der Ring- und Layer-Architektur:

1. **`docs/WORKING_STATE.md`**: Generierte Dokumentation mit historischer Layer-Einteilung (Layer 0–10).
2. **`capabilities.toml` & Crate-Manifeste (`Cargo.toml`)**: Das primäre Ring-Modell (`[package.metadata.contextra] ring`), das Crates in die Ringe 0–4 sowie `Tooling` einteilt.
3. **Hartkodierte Ordnungstabellen in `xtask/src/check_ring_layering.rs`**: Funktionen `ring0_crate_order` und `ring3_crate_order`, die manuell gepflegte Ganzzahl-Schichten innerhalb der Ringe definieren.

Die manuell gepflegten Tabellen in `check_ring_layering.rs` weisen schwerwiegende Inkonsistenzen und Tabellenfehler gegenüber der tatsächlichen Abhängigkeitstopologie des Cargo-Graphen auf:

- **Fehlende Crate-Registrierung (`contextra-durable-fs`)**: `contextra-durable-fs` ist eine valide Ring-0-Crate, fehlt jedoch vollständig in der statischen `ring0_crate_order`-Tabelle. Dadurch fällt sie auf den Fallback-Wert `99` zurück und wird fälschlich als höchste Schicht behandelt.
- **Invertierte Relation bei `contextra-sys`**: In der manuellen Ordnungstabelle wird `contextra-sys` auf Stufe `0` eingeordnet, während `contextra-ports` auf Stufe `1` liegt. Der reale Cargo-Graph zeigt jedoch eine direkte Abhängigkeit von `contextra-sys` auf `contextra-ports` (`contextra-sys -> contextra-ports`). Die Tabelle suggeriert somit fälschlich eine Rückwärtskante.
- **Unvollständige Schichtenabbildung**: Abhängigkeiten wie `contextra-crypto -> contextra-durable-fs` und `contextra-vector -> contextra-durable-fs` spiegeln physisch korrekte Kanten wider, werden aber durch die statische Handtabelle unzureichend abgebildet.

Die reale Ring-0-Topologie auf Basis der normalen Cargo-Abhängigkeiten (`cargo metadata --no-deps --format-version 1`) ist absolut azyklisch (DAG). Statische Handtabellen führen zu unnötiger Wartungslast, fehlerhaften Alarmen im Gate-Check und potenzieller Vertuschung architektonischer Kanten.

---

## 2. Entscheidung

Wir entscheiden, die manuellen Ordnungstabellen (`ring0_crate_order` und `ring3_crate_order`) in `xtask/src/check_ring_layering.rs` abzuschaffen. Künftig wird die schichtinterne Ordnung dynamisch aus der **topologischen Tiefe (Topological Level)** des Cargo-Abhängigkeitsgraphen berechnet.

### 2.1 Formales Berechnungsschema

Die Berechnungslogik verwendet das `cargo metadata --no-deps --format-version 1`-JSON und ermittelt für jede Crate innerhalb desselben Rings ihre topologische Ebene (Bottom-Up Depth):

$$\text{Level}(C) = \begin{cases} 0 & \text{falls } C \text{ keine Ring-internen normalen/build Abhängigkeiten hat} \\ 1 + \max_{D \in \text{Deps}(C)} \text{Level}(D) & \text{sonst} \end{cases}$$

- **Filterregeln**:
  - Nur Workspace-interne Crates desselben Rings werden einbezogen.
  - `dev-dependencies` werden bei der Ebenenberechnung ausgeschlossen (sie dienen Integrations- und Unittests).
  - Verwendet werden ausschließlich `normal`- und `build`-Abhängigkeiten.

- **Regel für Kanten-Validierung**:
  - Innerhalb von Ring 0 (und entsprechend Ring 3) darf eine Crate $A$ nur von einer Crate $B$ abhängen, wenn $\text{Level}(A) > \text{Level}(B)$ gilt.
  - Ein Zyklus innerhalb eines Rings führt automatisch zum Fehlschlagen des Gate-Checks.

---

## 3. Berechneter Ist-Zustand für Ring 0

Basierend auf der exakten Auswertung von `cargo metadata --no-deps --format-version 1` ergibt sich für alle 16 Ring-0-Crates die folgende azyklische Topologie:

### 3.1 Topologische Ebenen (Level 0 bis Level 5)

| Crate | Ring | Topologische Ebene (Level) | Auflösung der Abhängigkeiten innerhalb Ring 0 |
| :--- | :--- | :--- | :--- |
| `contextra-types` | Ring 0 | **Level 0** (Blatt) | keine |
| `contextra-wire` | Ring 0 | **Level 0** (Blatt) | keine |
| `contextra-ports` | Ring 0 | **Level 1** | `contextra-types` |
| `contextra-simd` | Ring 0 | **Level 1** | `contextra-types` |
| `contextra-avv-generator` | Ring 0 | **Level 2** | `contextra-ports`, `contextra-types` |
| `contextra-mvcc` | Ring 0 | **Level 2** | `contextra-ports`, `contextra-types` |
| `contextra-rank` | Ring 0 | **Level 2** | `contextra-ports`, `contextra-types` |
| `contextra-sys` | Ring 0 | **Level 2** | `contextra-ports` |
| `contextra-text` | Ring 0 | **Level 2** | `contextra-ports`, `contextra-types` |
| `contextra-adapt` | Ring 0 | **Level 3** | `contextra-ports`, `contextra-sys`, `contextra-types` |
| `contextra-core` | Ring 0 | **Level 3** | `contextra-mvcc`, `contextra-ports`, `contextra-types`, `contextra-wire` |
| `contextra-durable-fs` | Ring 0 | **Level 3** | `contextra-sys` |
| `contextra-crypto` | Ring 0 | **Level 4** | `contextra-durable-fs`, `contextra-ports`, `contextra-types` |
| `contextra-graph` | Ring 0 | **Level 4** | `contextra-adapt`, `contextra-ports`, `contextra-types` |
| `contextra-audit-export` | Ring 0 | **Level 5** | `contextra-crypto`, `contextra-ports`, `contextra-types` |
| `contextra-vector` | Ring 0 | **Level 5** | `contextra-core`, `contextra-crypto`, `contextra-durable-fs`, `contextra-ports`, `contextra-simd`, `contextra-sys`, `contextra-types` |

### 3.2 Vollständige Liste der Ring-0-Abhängigkeitskanten (35 Kanten)

1. `contextra-adapt -> contextra-ports` (normal)
2. `contextra-adapt -> contextra-sys` (normal)
3. `contextra-adapt -> contextra-types` (normal)
4. `contextra-audit-export -> contextra-crypto` (normal)
5. `contextra-audit-export -> contextra-ports` (normal)
6. `contextra-audit-export -> contextra-types` (normal)
7. `contextra-avv-generator -> contextra-ports` (normal)
8. `contextra-avv-generator -> contextra-types` (normal)
9. `contextra-core -> contextra-mvcc` (normal)
10. `contextra-core -> contextra-ports` (normal)
11. `contextra-core -> contextra-types` (normal)
12. `contextra-core -> contextra-wire` (normal)
13. `contextra-crypto -> contextra-durable-fs` (normal)
14. `contextra-crypto -> contextra-ports` (normal)
15. `contextra-crypto -> contextra-types` (normal)
16. `contextra-durable-fs -> contextra-sys` (normal)
17. `contextra-graph -> contextra-adapt` (normal)
18. `contextra-graph -> contextra-ports` (normal)
19. `contextra-graph -> contextra-types` (normal)
20. `contextra-mvcc -> contextra-ports` (normal)
21. `contextra-mvcc -> contextra-types` (normal)
22. `contextra-ports -> contextra-types` (normal)
23. `contextra-rank -> contextra-ports` (normal)
24. `contextra-rank -> contextra-types` (normal)
25. `contextra-simd -> contextra-types` (normal)
26. `contextra-sys -> contextra-ports` (normal)
27. `contextra-text -> contextra-ports` (normal)
28. `contextra-text -> contextra-types` (normal)
29. `contextra-vector -> contextra-core` (normal)
30. `contextra-vector -> contextra-crypto` (normal)
31. `contextra-vector -> contextra-durable-fs` (normal)
32. `contextra-vector -> contextra-ports` (normal)
33. `contextra-vector -> contextra-simd` (normal)
34. `contextra-vector -> contextra-sys` (normal)
35. `contextra-vector -> contextra-types` (normal)

---

## 4. Behandlung bestehender Allowlist-Einträge

In `xtask/src/check_ring_layering.rs` existieren derzeit drei explizite Allowlist-Ausnahmen:

1. **`#CTX-101`**: `contextra-graph` -> `contextra-store` (Phase 1a, Ablauf: `2026-12-31`)
   - *Grund*: Dev-Dependency auf Store für Graph-Integrationstests; wird schrittweise nach `contextra-testkit` isoliert.
2. **`#CTX-102`**: `contextra-infer-onnx` -> `contextra-infer-candle` (Phase 1b, Ablauf: `2026-12-31`)
   - *Grund*: Ausführungsanbieter-Abstraktion in Phase 1b.
3. **`#CTX-103`**: `contextra-infer-ollama` -> `contextra-infer-onnx` (Phase 1b, Ablauf: `2026-12-31`)
   - *Grund*: Dev-Dependency für Benchmarks/Tests; wird in Phase 1b isoliert.

### Governance-Richtlinie für Allowlists:
- **Keine Verlängerung**: Die Ablaufdaten aller drei Allowlist-Einträge verbleiben strikt auf dem **31.12.2026**. Es erfolgt keinerlei Fristverlängerung.
- **Kontinuierlicher Abbau**: Die Ausnahmen dürfen ausschließlich schrittweise abgebaut und nach Bereinigung der Test-Abhängigkeiten entfernt werden. Neuzuweisungen oder Erweiterungen der Allowlist sind unzulässig.

---

## 5. Alternativen

1. **Beibehaltung statischer Handtabellen (Status Quo)**:
   - *Nachteil*: Hohes Risiko veralteter Tabellen, Fehler bei Crate-Hinzufügungen (wie bei `contextra-durable-fs`), falsche Richtungsanzeigen (wie bei `contextra-sys`). Hohe manuelle Wartungslast.
2. **Manuelle Tabellenkorrektur ohne dynamische Berechnung**:
   - *Nachteil*: Behebt Symptome kurzfristig, schützt aber nicht vor zukünftigen Desynchronisationen bei Refactorings.
3. **Dynamische Auswertung per Cargo-Graph (Gewählte Option)**:
   - *Vorteil*: Mathematisch exakt, automatisiert, wartungsfrei, spiegelt immer die tatsächliche Build-Realität wider.

---

## 6. Konsequenzen

### Positiv:
- **Fehlerfreiheit**: Beseitigung aller Tabellenfehler (`contextra-durable-fs` Zuordnung, `contextra-sys` Relationsrichtung).
- **Wartungsfreiheit**: Neue Ring-0- oder Ring-3-Crates sortieren sich ohne manuelle Anpassung in `check_ring_layering.rs` automatisch in die korrekte topologische Ebene ein.
- **Determinismus**: Der Gate-Check basiert auf `cargo metadata`, dem einzigen kanonischen Truth-Store des Rust-Ökosystems.

### Negativ / Risiken:
- Eine Anpassung von `xtask/src/check_ring_layering.rs` berührt geschützte Pfade (`xtask/**`) und bedarf daher einer separaten Freigabe durch das Governance-Team.

---

## 7. Migrationsplan

1. **Phase 1 (Dieser ADR)**:
   - Dokumentation des Soll-Zustands, der mathematischen Formalisierung und der Fehleranalyse in `docs/decisions/ADR-108-ring-order-from-graph.md`.
2. **Phase 2 (Separater PR - xtask Anpassung)**:
   - Nach separater Freigabe der geschützten Pfade: Ersetzen von `ring0_crate_order` und `ring3_crate_order` in `xtask/src/check_ring_layering.rs` durch einen In-Memory DAG-Topological-Sort / Level-Calculator auf Basis der `cargo metadata`-Daten.
3. **Phase 3 (Dokumentations-Sync)**:
   - Aktualisierung von `docs/WORKING_STATE.md`, um veraltete Layer-Bezeichnungen (Layer 0–10) schrittweise auf die Ring-0..4-Nomenklatur und topologischen Level einzunorden.

# Contextra Baukasten: Leitfaden & Orientierung für Implementierer

**Gültigkeit:** Für alle Coding-Agenten und Entwickler, die an der Entkopplung und Umwandlung des Contextra-Retrieval-Kerns in autarke Bausteine arbeiten.
**Normative Grundlagen:**
- `docs/SPEC.md` v3 (normativ verbindlich)
- `docs/decoupling/BAUKASTEN_ARCHITEKTURENTSCHEIDUNG.md` (Architekturprinzipien & Befunde)
- `docs/decoupling/BAUKASTEN_UMSETZUNGSPLAN.md` (Detaillierter Arbeitspaket-Katalog WP-00 bis WP-91)
- `AGENTS.md` (Verbindliche System- und Autonomieregeln)

---

## 1. Das Zielbild: Autarker Baukasten

Contextra wird von einem monolithisch gekoppelten System in ein Set autarker, isoliert lauffähiger Bausteine (**Building Blocks**) transformiert. Jeder Baustein soll ohne unsaubere Abhängigkeiten in Drittprojekte exportiert oder eigenständig weiterverwendet werden können.

### Das 4-Ebenen-Schichtenmodell (Abhängigkeits-Richtung: oben darf nach unten zeigen, niemals umgekehrt)

| Ebene | Crates | Erlaubte zwingende Abhängigkeiten |
|---|---|---|
| **E0a Seams** | `ids`, `env`, `sys`, `wire`, `durable-fs` | **Nichts Internes** (vollständig autark) |
| **E0b Bausteine** | `simd`, `rank`, `adapt`, `privacy`, `prompt-guard`, `erasure`, `sandbox`, `ingest`, `vault`, `license` | Nur E0a (`ids`, `env`) |
| **E1 Capability** | `vector`, `text`, `graph`, `mvcc`, `crypto`, `store`, `checkpoint`, `kvcache`, `infer-ollama`, `infer-candle`, `router` | E0a, E0b (explizite Whitelist je Crate) |
| **E2 Bridge** | `engine`, `db`, `cognition`, `agent` | Beliebig E0 / E1 |
| **E3 Anwendung** | `contextra`, `mcp`, `py`, `testkit`, `bench` | E2 / E1 / E0 |

---

## 2. Die 10 Gebote für Implementierer (Coding-Agenten)

1. **Strikte Datei-Isolation (Scope Guard):**
   - Jedes Arbeitspaket arbeitet STRENG auf seinem Crate-Pfad (z. B. `crates/<crate>/**`).
   - Keine unabgesprochenen Modifikationen an fremden Crates oder globalen Governance-Dateien.
2. **Einheitliche Seams für IDs und Environment (AD-2):**
   - `DocId`, `TxId`, `TenantId`, `EntityId` stammen ausschließlich aus `contextra-ids`.
   - `Clock`, `Rng`, `MetricsSink`, `IdGen` stammen ausschließlich aus `contextra-env`.
   - Keine lokalen Duplikate/Kopien von `Clock`, `Rng` oder `DocId` anlegen.
3. **Eigener Fehlertyp pro Crate (AD-4):**
   - Jedes Crate definiert sein lokales `#[derive(thiserror::Error)] pub enum <Crate>Error`.
   - `ContextraError` DARF NICHT in E0/E1-Crates importiert werden.
   - Konvertierung erfolgt an den Systemrändern (E2/E3) via `ContextraError::wrap(...)` oder `From`-Implementierungen im jeweiligen Crate (`compat/error.rs`).
4. **Ports gehören dem Konsumenten (AD-3):**
   - E0/E1-Bausteine definieren ihren minimalen Vertrag selbst (`contract.rs`).
   - Keine zwingende Abhängigkeit von `contextra-ports` in E0/E1-Kernen.
5. **Konfiguration gehört zum Algorithmus (AD-5):**
   - Algorithmus-Parameter (z. B. HNSW-Parameter, PPR-Config, Metriken) gehören in das jeweilige Crate.
   - `DistanceMetric` gehört zu `contextra-simd` (und wird ggf. re-exportiert).
6. **Reine Kerne, I/O an den Rändern (AD-9):**
   - Kerne sind synchron und deterministisch (`Clock`/`Rng` injiziert).
   - Async- und I/O-Logik befindet sich in Adaptern oder Schicht E2/E3.
7. **`default = []` für E0/E1 (AD-7):**
   - Keine Default-Features, die Workspace-Abhängigkeiten aktivieren. Nur `std` ist als Default zulässig.
8. **Zero Panic & Speichersicherheit:**
   - Kein `.unwrap()`, `.expect()`, `panic!()`, `todo!()` oder `unimplemented!()` in Produktionscode.
   - Unsafe ist verboten (`#![forbid(unsafe_code)]`), außer in den deklarierten Unsafe-Inseln (`simd`, `sys`, `wire`) mit lückenlosem `// SAFETY:`-Nachweis.
9. **Löschketten- und DSGVO-Entkopplung:**
   - DSGVO-, Deletion-Proof- oder Saga-Logik gehört ausschließlich in `contextra-erasure`.
   - Andere Crates binden physische Bereinigungen neutral über `Purger` oder `LayerScanner` an.
10. **Autonomie & Dokumentation:**
    - Nach Planfreigabe autonom arbeiten.
    - Änderungen in den entsprechenden Handoff-Dateien (`docs/handoff/<CRATE>.md` oder `docs/handoff/fragments/`) nachvollziehbar dokumentieren.

---

## 3. Arbeitspaket-Spuren (Zuordnung)

Die Umsetzung ist in 4 parallele Spuren geteilt (siehe `docs/decoupling/BAUKASTEN_UMSETZUNGSPLAN.md` §7):

| Spur | Fokus | Pakete / Crates |
|---|---|---|
| **Spur A: Plattform & Gates** | Build, CI, Tooling, Release | Welle 0, WP-12, WP-13, WP-90, WP-91 |
| **Spur B: Kern-Retrieval** | HNSW, Vektor, Text (WAND/BM25), Graph (PPR) | WP-30 (`mvcc`), WP-32 (`text`), WP-33 (`checkpoint`), WP-34 (`graph`), WP-35 (`rank`), WP-36 (`vector`), WP-80, WP-81 |
| **Spur C: Speicher & Krypto** | LSM-Tree, WAL, Krypto-Primitive | WP-31 (`crypto`), WP-37 (`store`), WP-38 (`kvcache`), WP-70a (`erasure`) |
| **Spur D: Bausteine & Integration** | Middleware, Integration, Fassade | Welle 2 (`ingest`, `privacy`, `prompt-guard`, `vault`, etc.), WP-39 (`infer-*`), WP-50/51 (`ports`/`types`), WP-60–64, WP-70 |

*Wichtig:* Spur B und C stimmen sich an den Schnittstellen `contextra-mvcc` und `contextra-store` verbindlich ab.

---

## 4. Wichtigste Prüfbefehle (Gate-Werkzeuge)

Jeder Implementierer muss vor Abschluss seines Arbeitspakets die Einhaltung der Gates verifizieren:

```bash
# 1. Schichten- und Abhängigkeitsprüfung (G-L1):
python3 tools/gate_layering.py --all

# 2. Null-Prüfung für unerlaubte Bridge-Referenzen außerhalb compat/ (G-L2):
python3 tools/count_bridge_refs.py -c <crate>

# 3. Isolation & Standalone-Build des Bausteins (G-L4):
python3 tools/isolation_proof.py -c <crate> --modes no-default,default

# 4. Feature-Hygiene prüfen:
python3 tools/feature_audit.py --fail-on-unused

# 5. Detaillierten Crate-Statusbericht abrufen:
python3 tools/decouple_inspector.py -c <crate>

# 6. Unsafe- und Zero-Panic-Audit:
python3 tools/zero_panic_analyzer.py -c <crate>
```

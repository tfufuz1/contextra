# Architekturentscheidung & Spezifikationsvervollständigung (Scope-Reset)

**Dokument-ID:** ADR-SCOPE-REDUCTION-001  
**Datum:** 18. November 2026  
**Status:** Akzeptiert (Normativ)  
**Autor:** Principal Rust Engineer  
**Geltungsbereich:** Workspace `contextra`, SPEC v6, `capabilities.toml`

---

## 1. Ausgangslage & Scope-Bereinigung

Im Rahmen der Stabilisierung des Kerns und zur Gewährleistung strikter mathematischer und formaler Korrektheit (INV-1 bis INV-16) wurden 13 experimentelle bzw. periphere Crates vorläufig aus dem Workspace entfernt:

```text
contextra-cognition      contextra-agent          contextra-adapt
contextra-wire           contextra-py             contextra-prompt-guard
contextra-ingest         contextra-infer-onnx     contextra-infer-ollama
contextra-infer-candle   contextra-sandbox        contextra-router
contextra-kv-cache
```

### Begründung des Scope-Resets
1. **Zyklische & transitive Lecks eliminieren:** Zahlreiche dieser Crates verletzten die Ebenenordnung (INV-8) durch ungegitterte Brücken-Referenzen (`compat/`) oder zogen über Default-Features unzulässige E2-Abhängigkeiten (`types`/`ports`) in E0/E1-Closures.
2. **Klassentrennung durchsetzen (INV-13 / §11):** Die Trennung zwischen Unternehmenswissen (Klasse K) und Personendaten (Klasse P) muss im Kernarchitekturmodell lückenlos nachweisbar sein.
3. **Fokus auf die beweisbare Persistenz- & Löschkette:** Der minimale Kern für Hybrid-Search, Transaktionen, Zeitreise und DSGVO-konforme Löschkette (`erasure`) muss autark und isoliert bauen und testen (`cargo test --locked`).

---

## 2. Der konsolidierte Ziel-Workspace

Der aktive Workspace umfasst exakt 23 Crates, hierarchisch gegliedert in Ebenen E0 bis E3:

```mermaid
graph TD
    classDef e0 fill:#e1f5fe,stroke:#0288d1,stroke-width:2px;
    classDef e1 fill:#e8f8f5,stroke:#26a69a,stroke-width:2px;
    classDef e2 fill:#f3e5f5,stroke:#ab47bc,stroke-width:2px;
    classDef e3 fill:#fff3e0,stroke:#fb8c00,stroke-width:2px;

    subgraph E0_Foundation ["E0: Autarke Mikround Primitivkerne"]
        ids["contextra-ids"]:::e0
        env["contextra-env"]:::e0
        sys["contextra-sys (unsafe)"]:::e0
        dfs["contextra-durable-fs"]:::e0
        simd["contextra-simd (unsafe)"]:::e0
        privacy["contextra-privacy"]:::e0
        erasure["contextra-erasure"]:::e0
    end

    subgraph E1_Subsystems ["E1: Autonome Domänenbausteine"]
        mvcc["contextra-mvcc"]:::e1
        crypto["contextra-crypto"]:::e1
        text["contextra-text"]:::e1
        graph["contextra-graph"]:::e1
        rank["contextra-rank"]:::e1
        ckpt["contextra-checkpoint"]:::e1
        vault["contextra-vault"]:::e1
        store["contextra-store (E1b)"]:::e1
        vec["contextra-vector (E1b)"]:::e1
    end

    subgraph E2_Integration ["E2: Integration & Orchestrierung"]
        types["contextra-types"]:::e2
        ports["contextra-ports"]:::e2
        db["contextra-db"]:::e2
        facade["contextra"]:::e2
        engine["contextra-engine (Deprecated Façade)"]:::e2
    end

    subgraph E3_Applications ["E3: Endpunkte & Werkzeuge"]
        mcp["contextra-mcp"]:::e3
        bench["contextra-bench"]:::e3
        testkit["contextra-testkit"]:::e3
    end

    E1_Subsystems --> E0_Foundation
    E2_Integration --> E1_Subsystems
    E2_Integration --> E0_Foundation
    E3_Applications --> E2_Integration
```

---

## 3. Normative Architekturentscheidungen (D-34 bis D-42)

### D-34: Handhabung von `agent` und `cognition`
- **Entscheidung:** Verbleiben bis auf Weiteres außerhalb des Workspace.
- **Bedingung für Reaktivierung:** 
  1. Vollständige Umstellung auf Inversion-of-Control via Ports (`ports::DocumentSource`, `ports::ConsolidationSink`).
  2. Beseitigung aller direkten Abhängigkeiten zu `engine` und `graph`.
  3. Nachweis, dass keine Personendaten (Klasse P) ungefiltert in den kognitiven Graph fließen (G-ER1 / G-PII1).

### D-35: Handhabung von `adapt` und `router`
- **Entscheidung:** Regler (`decay_controller`, `homeostat`, `pid`) verbleiben übergangsweise als interne Module in `contextra-db`.
- **Bedingung für Reaktivierung:**
  1. Durchführung der Differenztests (§6.4, S-60) gegen die Referenz-Implementierung.
  2. Bereinigung der Feature-Namen auf `exp-flow-thompson` und `exp-rie-greedy` (S-23).

### D-36: Handhabung von `wire`
- **Entscheidung:** Einfrierung bis zur Einführung von Schema-Stabilitäts-Gates.
- **Bedingung für Reaktivierung:** FlatBuffers-Schema-Freeze mit CI-Prüfung (`G-F2`, SHA256-Abgleich gegen `schemas/contextra.fbs.sha256`).

### D-37: Handhabung von `py`
- **Entscheidung:** Auslagerung bis zum API-Freeze von `contextra`.
- **Bedingung für Reaktivierung:** Grüne CI-Pipeline für `maturin` mit und ohne `pyo3` sowie Weiterleitung des konsistenten `docid-128`-Features an `contextra-ids`.

### D-38: Handhabung von `prompt-guard`
- **Entscheidung:** Absorbiert in `contextra-mcp` bzw. `contextra-privacy`.
- **Begründung:** Ein eigenes Crate für weniger als 1.000 Zeilen Validierungslogik widerspricht Regel X4.

### D-39: Handhabung von `ingest`
- **Entscheidung:** Reaktivierung erst nach Abschluss von S-61 (Chunker-Paritätsnachweis gegen `contextra-text`).

### D-40: Handhabung der Inferenz-Crates (`infer-onnx`, `infer-ollama`, `infer-candle`)
- **Entscheidung:** Vollständige Entkopplung der Inferenz. Contextra operiert als In-Memory / Embedded Search- und Storage-Engine. Inferenz-Engines werden rein über dynamische Trait-Objekte in `contextra-ports` angebunden.
- **Bedingung für Reaktivierung:**
  1. Jedes Inferenz-Crate muss `--no-default-features` kompilieren.
  2. Keine verbotenen Default-Features (insb. `candle` oder ONNX-Binaries).
  3. Kern-Closure ≤ 1 (nur `env`).

### D-41: Handhabung von `sandbox`
- **Entscheidung:** Temporäre Deaktivierung. Reaktivierung als isoliertes E0-Crate nach grünem `isolation_proof.py`.

### D-42: Handhabung von `kv-cache`
- **Entscheidung:** Keine Reaktivierung vor Abschluss von S-38. Die doppelte KV-Implementierung zwischen `crypto/kv_segment` und `kvcache` wird zugunsten eines einzigen Besitzers aufgelöst.

---

## 4. Reaktivierungs-Gate: G-REACTIVATE (normativ)

Bevor ein entferntes Crate wieder als Workspace-Member aufgenommen werden darf, muss das Gate **G-REACTIVATE** automatisiert erfüllt sein:

```bash
# G-REACTIVATE Prüfprotokoll
1. cargo check -p <crate> --no-default-features --locked
2. python3 tools/gate_bridge_null.py --crate <crate> (Muss 0 ungegitterte Referenzen liefern)
3. python3 tools/feature_audit.py --crate <crate> (Keine verbotenen Defaults gem. R10)
4. python3 tools/isolation_proof.py --crate <crate> (Autarkes Kompilieren in leerem Workspace)
5. Nachweis des Eintrags in capabilities.toml mit level, sublevel und core_deps
```

---

## 5. Sofortige Maßnahmen für den verbleibenden Kern

1. **`contextra-erasure`:** Sofortige Aufnahme in `default-members` und Behebung der Check-Fehler (S-101 / S-102).
2. **`contextra-mcp`:** Sicherstellung der Funktionsfähigkeit ohne `wire`/`prompt-guard` über stdio.
3. **`contextra-engine`:** Verbleibt als reine Re-Export-Façade auf `contextra-db` bis zur endgültigen Deprecation in S-70.
4. **Gates schärfen:** Entfernung aller `|| true` aus `rust-ci.yml` zur Durchsetzung von INV-4.

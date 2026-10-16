# Contextra — Jules Session Bootstrap
> Mascinenausführbare Checkliste. Jede Session MUSS mit dieser
> Sequenz beginnen, bevor Code geschrieben oder Dateien geändert werden.

- **VETOES.md** (Root): Permanent abgelehnte oder eingeschränkt akzeptierte Features.
  Vor jeder neuen Feature-Implementierung mit "F-NN"-Bezeichnung prüfen ob ein
  Eintrag existiert. `just check-vetoes` läuft automatisch, ist aber kein Ersatz
  für manuelles Lesen vor Arbeitsbeginn an physio-*/Nucleation-artigen Features.

## Phase 0 — Session-Identität etablieren, Task-Claiming & Context Pack erzeugen (30 Sekunden)

```bash
cargo xtask session-init --crate <CRATE_NAME> --task "<KURZE_BESCHREIBUNG>" --output-env <!-- harness:planned --> <!-- doc-ref-ignore -->
source .jules/session.env
cargo xtask context-pack --crate <CRATE_NAME> <!-- harness:planned --> <!-- doc-ref-ignore -->
```

Der Befehl `context-pack` erzeugt `.jules/context/CONTEXT_PACK.md` <!-- doc-ref-ignore -->. Dies ist die einzige Datei, die ein Agent zu Sessionbeginn lesen muss.

## Phase 1 — Offene Kritische Issues prüfen (30 Sekunden)

```bash
# BLOCKER und CRITICAL Tags — bei Fund: STOP, zuerst beheben
grep -rn "AI-TAG\[.*\]\[BLOCKER\]\|AI-TAG\[.*\]\[CRITICAL\]" crates/ \
  --include="*.rs" | grep -v "RESOLVED" || echo "  ✅ Keine"

# Offene ANCHORS mit IN-PROGRESS Status
grep -rn "ANCHOR\[.*\] STATUS:IN-PROGRESS" crates/ \
  --include="*.rs" || echo "  (keine)"

# WORKING_STATE.md lesen (autogeneriert, immer aktuell)
head -50 WORKING_STATE.md
```

## Phase 2 — Toolchain verifizieren (30 Sekunden)

```bash
# Verifiziere Build-Grundlage
cargo check --workspace

# Falls cargo nicht im PATH: Rust-Toolchain aktivieren
# source "$HOME/.cargo/env" && cargo check --workspace
```

## Phase 3 — Aufgaben-spezifischen Kontext laden

Lade basierend auf der Aufgabe:

| Aufgabe-Typ | Zu lesende Dateien |
|-------------|-------------------|
| Code in `contextra-store/*` <!-- doc-ref-ignore --> | `crates/contextra-store/AGENTS.md`, `rules/wal_crypto.md`, `rules/async-io.md` |
| Code in `contextra-vector/*` <!-- doc-ref-ignore --> | `crates/contextra-vector/AGENTS.md`, `rules/simd_safety.md` |
| Code in `contextra-db/*` <!-- doc-ref-ignore --> | `crates/contextra-db/AGENTS.md` |
| Neue Dependency | `rules/dependencies.md` → Cargo.lock prüfen → crates.io verifizieren |
| Neue API-Oberfläche | `CONSTITUTION.md`, `docs/TYPE_REGISTRY.md` |
| ADR schreiben | `docs/decisions/` (letzte 5 ADRs lesen), `CONSTITUTION.md §Governance` |
| Tests schreiben | `rules/testing.md`, `rules/test_quality.md` |
| unsafe Code | `rules/simd_safety.md` — NUR in approved files (AGENTS.md §4) |
| Crypto/WAL | `rules/wal_crypto.md` → WAL-First-Regel verifizieren |

## Phase 4 — Pre-Write-Check (vor JEDER Code-Änderung)

```bash
# API-Halluzinations-Schutz: Signatur vor Nutzung verifizieren
grep -n "pub fn <METHODE>" crates/contextra-db/src/collection.rs

# Typ-Dopplungs-Schutz: Typ-Register prüfen
grep "<TYPNAME>" docs/TYPE_REGISTRY.md

# DAG-Prüfung: Keine Layer-Verletzung
```

## Phase 5 — Session-Ende (VOR letztem Commit)

```bash
# 1. Format & Lint (Formatierung erzwingen + Clippy/Check)
cargo fmt --all
just check

# 2. DAG-Integrität & Tech-Debt Audit
just dag-check
just debt-audit

# 3. Tests
just test

# 4. Sync-Docs (generiert WORKING_STATE.md, CHANGELOG, etc.)
just sync-docs

# 5. Finaler Check
just sync-docs-check
```

## Phase 6 — Pre-Submit Gate (BLOCKIEREND — kein Submit ohne ✅)

> **Invariante:** Führe zwingend `cargo xtask jules-submit-gate --crate <DEIN-CRATE>` aus. <!-- harness:planned --> <!-- doc-ref-ignore -->
> Kein submit() vor ✅ SUBMIT GATE BESTANDEN. Bei Fehlschlag: STOP, Fix, Phase 6 erneut durchlaufen.

```bash
# Pre-Submit Gate ausführen (Schritte 6.1–6.5 automatisiert)
cargo xtask jules-submit-gate --crate <DEIN-CRATE> <!-- harness:planned --> <!-- doc-ref-ignore -->
```

> **Regel für Commit-Messages:** Jede in der PR-Beschreibung unter
> "Hinzugefügt" oder "Getestet" genannte Datei MUSS in `git diff --name-only
> origin/main...HEAD` erscheinen. Ausnahmen begründen, nie stillschweigend weglassen.

## Notfall-Eskalation (Prompt-Thrashing)

Wenn derselbe Compiler-Fehler nach 2 Iterationen nicht behoben ist:
1. **STOPP** — keinen weiteren Code schreiben
2. Fehler auf minimales Beispiel reduzieren
3. Fehlermeldung + Diff in Session-Log dokumentieren
4. Entwickler um explizite Instruktion bitten

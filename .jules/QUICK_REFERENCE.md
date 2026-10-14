# Contextra — Jules Quick Reference (QUICK_REFERENCE.md)

> **Nur lesen wenn nötig** — die verbindliche Primärquelle ist `.jules/context/CONTEXT_PACK.md`
> (erzeugt durch `cargo xtask jules start --card <karte>`).
> Stand: 2026-10-14 · Gilt für alle Jules-Sitzungen (v1.1)

---

## 🚦 Session-Lebenszyklus (5 Befehle)

```bash
# 1. Session starten — liest Karte, setzt Claim, erzeugt CONTEXT_PACK.md
cargo xtask jules start --card .jules/tasks/T-2026-XXXX.toml

# 2. Während der Arbeit — Syntax, Scope, Duplikate, Doktrin
cargo xtask jules check

# 3. Akzeptanz-Tests + Architektur-Gates
cargo xtask jules verify

# 4. Finales Submit-Gate + PR-Report
cargo xtask jules submit

# 5. Im Fehlerfall / Abbruch (Fehler-Limit 2 Iterationen!)
cargo xtask jules stop --reason <thrash|blocked|scope-exceeded>
```

---

## 📖 Kontext-Ladereihenfolge (verbindlich)

```
1. .jules/context/CONTEXT_PACK.md     ← Einstiegspunkt IMMER zuerst
2. crates/<crate>/AGENTS.md           ← Bei Crate-Änderung zwingend
3. AGENTS.md (Root)                   ← Nur bei offenen Fragen
4. WORKING_STATE.md                   ← Nur bei offenen Fragen
```

---

## ⚡ Tier 0 — Sofort (build-frei)

```bash
cargo xtask harness-list              # Alle verfügbaren Befehle (verbindlich!)
cargo xtask repo-doctor               # Session-Start-Dashboard
cargo xtask env-attest                # Toolchain-Validierung
cargo xtask fast-diff-lint            # Schneller Doktrin-Linter
cargo xtask workspace-verify          # Workspace-Konsistenz
cargo xtask explain <gate>            # Gate-Erklärung
cargo xtask symbol-exists <pfad>      # Symbol-Existenz prüfen
```

## 🔍 Tier 1 — Workspace-Scan (build-frei, ~2-3s)

```bash
cargo xtask orphan-symbols            # Ungenutzte pub-Symbole
cargo xtask unwired-ports             # Traits ohne Implementierung
cargo xtask stub-impls                # Stub-Implementierungen
cargo xtask doctrine-scan             # Zero-Panic, Unsafe, #[allow]
cargo xtask dependency-graph-audit    # Ring-DAG (P5) + tokio Ring 0 (P26)
cargo xtask check-ring-layering       # Ring-Layering-Kurzcheck
cargo xtask check-ring-layering-full  # Ring-Layering vollständig (alle Features)
cargo xtask determinism-check         # Determinismus via Ports (P28)
cargo xtask check-unsafe-islands      # Unsafe nur in Inseln
cargo xtask check-duplicate-symbols   # Doppelte Symbole
cargo xtask check-result-dropped-io   # Ignorierte I/O-Fehler (INV-DURABILITY-RING)
cargo xtask claim                     # Claim-Integrität
```

## 🏗️ Tier 2 — Build-abhängig

```bash
cargo xtask check-compile             # Fehlerfrei kompilieren
cargo xtask bench-gate --tolerance 0.05  # Benchmark-Gate
cargo xtask feature-matrix            # Feature-Kombinationen
cargo xtask loom-run                  # Concurrency-Tests
cargo xtask fuzz-smoke                # Fuzzing-Smoke
cargo xtask mutants-diff              # Mutation-Testing auf Diff
cargo xtask wal-replay-verify         # WAL-Replay Crash-Konsistenz
cargo xtask blast-radius              # Auswirkungsbereich
cargo xtask risk                      # Risikobewertung
cargo xtask can-merge                 # Merge-Entscheidung
```

---

## 🛡️ Pflicht-APMs vor jeder Änderung (TOP 10)

| APM | Kurzname | Prüfung |
|-----|----------|---------|
| APM-1 | tmp→fsync→rename→fsync_parent | Jede Datei-Erstellung atomic? |
| APM-2 | if/else-if vs. separate if | Filter vor Limit? |
| APM-3 | try_write() für Mutationen | Niemals try_write für garantierte Updates |
| APM-7 | Byte-Slicing → Panic bei ä/ö | .char_indices() statt &s[a..b] |
| APM-12 | Lock-Hierarchie-Inversion | collections → kv_locks → embedder |
| APM-13 | Nonce-/Key-Domain-Separation | Neuer HKDF-Separator pro Zweck |
| APM-16 | NaN/Inf in Scoring | .is_finite() vor Vergleich |
| APM-18 | Check-Then-Act ohne Version | Compare-And-Swap-Semantik |
| APM-29 | Silent Fallback (.ok()) | Error-Propagation statt Swallowing |
| APM-36 | Vektor-Dimension ungeprüft | Am API-Eingang validieren |

Vollständige Liste: `.jules/ANTI_PATTERNS.md`

---

## 🏷️ Tag-Grammatik (Pflicht-Format)

```rust
// AI-TAG[KATEGORIE][SEVERITY] Kurzbeschreibung (ID: AGT-<CRATE>-<8-hex>) (TS: <ISO-8601-UTC>) (SESSION: <HASH>)
// BEFUND: Detaillierte Analyse.
// RISIKO: Auswirkung bei Ausfall.
// EMPFEHLUNG: Konkrete Handlung.

// ANCHOR[TYP:ID] STATUS:OPEN (TS: <ISO-8601-UTC>) (SESSION: <HASH>)
```

---

## 🔒 Invarianten-Kurzreferenz

| ID | Befehl | Regel |
|----|--------|-------|
| P2 | `cargo xtask doctrine-scan` | Kein unwrap/expect/panic in Prod |
| P5 | `cargo xtask dependency-graph-audit` | DAG: nur zu niedrigeren Ringen |
| P12 | `cargo xtask check-ring-capabilities-consistency` | capabilities.toml = Ring-Wahrheit |
| P26 | `cargo xtask check-ring0-async-purity` | Ring 0 kein tokio in Prod |
| P28 | `cargo xtask determinism-check` | Ports: Clock, Rng, IdGenerator |
| INV-TENANT-1 | `cargo test -p contextra-types` | TenantId::try_new(0) → Err |
| INV-WAL-TRUNCATION-1 | `cargo test -p contextra-store` | WAL-Truncation via High-Water-Mark |
| INV-DURABILITY-RING | `cargo test -p contextra` | MemoryOnly ↔ DeletionProof inkompatibel |

---

## ⚠️ Absolut verboten (AGENTS.md §10)

```bash
# ❌ NIEMALS:
sh -c "..."           # Kein sh -c — Argument-Arrays statt String
bash -c "..."         # Kein bash -c für Prozessaufrufe
grep <als Ersatz>     # Nur für gezieltes Lesen, nicht Analyse
python analyse.py     # Kein Ad-hoc-Analyseskript
```

---

## 🔄 Notfall-Eskalation

Gleicher Fehler **2× nicht behebbar** → STOPP:
```bash
cargo xtask jules stop --reason thrash
```
Fehler auf minimales Beispiel reduzieren, in Session-Log dokumentieren.

---

## 📁 Crate-Ladereferenz (Schnell-Index)

Vollständige Tabelle: `.jules/JULES_CONTEXT.md`

| Ring | Crate | Kritischste Invariante |
|------|-------|------------------------|
| 0 | `contextra-types` | INV-TENANT-1, TxId-Allokator |
| 0 | `contextra-crypto` | Nonce-Sicherheit, Zeroize, Domain-Sep |
| 0 | `contextra-vector` | APM-2 Tombstone, APM-3 try_write |
| 0 | `contextra-graph` | CSR-Invariante, Rollback-Vollständigkeit |
| 0 | `contextra-text` | APM-7 UTF-8-Slicing, Compound-Splitter |
| 1 | `contextra-store` | APM-1 fsync-Atomarität, INV-WAL-* |
| 1 | `contextra-checkpoint` | APM-8 spawn() in Drop |
| 3 | `contextra-db` | APM-5 RRF-Post-Score, 2PC-Rollback |
| 4 | `contextra-mcp` | APM-9 FFI-Panic, APM-27 Event-Flooding |

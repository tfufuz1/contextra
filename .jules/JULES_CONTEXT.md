# Contextra — Jules Agent Context
> Version: 3.0 | Stand: 2026-10-06 | Permanent Ambient Context für Jules Sessions (Systemspezifikation v15)
>
> ⚠️ **FRISCHEGARANTIE**: Diese Datei regelt ausschließlich die Session-Prozessführung für Jules.
> Die tatsächlichen Code-Fakten, Crate-Strukturen, Invarianten und Implementierungsstände
> sind gemäß MECE-Prinzip (CONSTITUTION.md §Documentation Model) in folgenden Quellen verankert:
> - **Code-Zustand & Non-Obvious Decisions**: siehe `AGENTS.md`
> - **Dynamischer Projektstatus & Tag-Inventar**: siehe `WORKING_STATE.md`
> - **Architektur-Entscheidungen (ADRs)**: siehe `docs/decisions/README.md` (ADR-Index gemäß ADR-095)

---

## 🎯 Kontext-Ladeordnung

1. **Sitzungsvertrag & Bootstrap**: `.jules/PREAMBLE.md`, `.jules/SESSION_BOOTSTRAP.md`
2. **Aktueller Code-Zustand & Invarianten**: `AGENTS.md` (Verifizierter Code-Befund, Non-Obvious Decisions)
3. **Crate-spezifische Anweisungen**: Crate-spezifische `AGENTS.md` laden
4. **Dynamischer Status & ADRs**: `WORKING_STATE.md`, `docs/decisions/README.md`

---

## 📖 Crate AGENTS.md Laderegel <!-- doc-ref-ignore -->

**MANDATORY FIRST STEP:** Bevor Code in einer Crate bearbeitet wird, MUSS Jules die jeweilige `AGENTS.md` der Crate laden (sofern vorhanden). Sie enthält Modul-Karten, API-Signaturen, Anti-Patterns und Lock-Hierarchien.

| Crate | Pfad für view_file / read |
|---|---|
| `contextra` | `crates/contextra/AGENTS.md` |
| `contextra-adapt` | `crates/contextra-adapt/AGENTS.md` |
| `contextra-agent` | `crates/contextra-agent/AGENTS.md` |
| `contextra-audit-export` | `crates/contextra-audit-export/AGENTS.md` |
| `contextra-avv-generator` | `crates/contextra-avv-generator/AGENTS.md` |
| `contextra-bench` | `benchmarks/contextra-bench/AGENTS.md` |
| `contextra-checkpoint` | `crates/contextra-checkpoint/AGENTS.md` |
| `contextra-cognition` | `crates/contextra-cognition/AGENTS.md` |
| `contextra-core` | `crates/contextra-core/AGENTS.md` |
| `contextra-crypto` | `crates/contextra-crypto/AGENTS.md` |
| `contextra-db` | `crates/contextra-db/AGENTS.md` |
| `contextra-durable-fs` | `crates/contextra-durable-fs/AGENTS.md` |
| `contextra-engine` | `crates/contextra-engine/AGENTS.md` |
| `contextra-graph` | `crates/contextra-graph/AGENTS.md` |
| `contextra-infer-candle` | `crates/contextra-infer-candle/AGENTS.md` |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/AGENTS.md` |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/AGENTS.md` |
| `contextra-kvcache` | `crates/contextra-kvcache/AGENTS.md` |
| `contextra-license` | `crates/contextra-license/AGENTS.md` |
| `contextra-mcp` | `crates/contextra-mcp/AGENTS.md` |
| `contextra-mvcc` | `crates/contextra-mvcc/AGENTS.md` |
| `contextra-ports` | `crates/contextra-ports/AGENTS.md` |
| `contextra-privacy` | `crates/contextra-privacy/AGENTS.md` |
| `contextra-py` | `crates/contextra-py/AGENTS.md` |
| `contextra-rank` | `crates/contextra-rank/AGENTS.md` |
| `contextra-router` | `crates/contextra-router/AGENTS.md` |
| `contextra-sandbox` | `crates/contextra-sandbox/AGENTS.md` |
| `contextra-simd` | `crates/contextra-simd/AGENTS.md` |
| `contextra-store` | `crates/contextra-store/AGENTS.md` |
| `contextra-sys` | `crates/contextra-sys/AGENTS.md` |
| `contextra-testkit` | `crates/contextra-testkit/AGENTS.md` |
| `contextra-text` | `crates/contextra-text/AGENTS.md` |
| `contextra-types` | `crates/contextra-types/AGENTS.md` |
| `contextra-vector` | `crates/contextra-vector/AGENTS.md` |
| `contextra-wire` | `crates/contextra-wire/AGENTS.md` |

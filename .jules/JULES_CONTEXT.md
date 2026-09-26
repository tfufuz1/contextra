# Contextra — Jules Agent Context
> Version: 2.1 | Stand: 2026-09-26 | Permanent Ambient Context für Jules Sessions
>
> ⚠️ **FRISCHEGARANTIE**: Diese Datei regelt ausschließlich die Session-Prozessführung für Jules.
> Die tatsächlichen Code-Fakten, Crate-Strukturen, Invarianten und Implementierungsstände
> sind gemäß MECE-Prinzip (CONSTITUTION.md §Documentation Model) in folgenden Quellen verankert:
> - **Code-Zustand & Non-Obvious Decisions**: siehe `AGENTS.md`
> - **Dynamischer Projektstatus & Tag-Inventar**: siehe `WORKING_STATE.md`
> - **Architektur-Entscheidungen (ADRs)**: siehe `DECISIONS.md` / `docs/decisions/`

---

## 🎯 Kontext-Ladeordnung & Modus Operandi für Sessions

Um Halluzinationen und veraltete Fakten zu vermeiden, gilt für jede Jules-Session folgende Lade- und Nachschlage-Reihenfolge:
1. **System & Arbeitsumgebung**: `.jules/JULES_CONTEXT.md` (Prozessanleitung), `.jules/SESSION_BOOTSTRAP.md`
2. **Aktueller Code-Zustand & Invarianten**: `AGENTS.md` (Verifizierter Code-Befund, Crate-Topologie, Non-Obvious Decisions)
3. **Offene Schulden & Tags**: `WORKING_STATE.md` (Autogenerierter Tag-Bericht)
4. **Verbindliche Architektur-Vorgaben**: `DECISIONS.md` (ADR-Zusammenfassungen)

> 📌 **Hinweis für Prompt-Erstellung**: Die automatische Erstellung von GitHub-Issues durch Workflows/Gates ist deaktiviert.

---

## 📜 Audit-Report & Remediation-Prompts Kontext-Anker (2026-09-26)

- **Audit-Report**: `CONTEXTRA_AUDIT_REPORT.md` dokumentiert den umfassenden System-Audit (Funde in Ring-2-Leaf, Compaction-Speicherbudget, `KvKeyLocks`-Divergenz, `BanditError`-Fehlergrenze, `ConstantTimeEq`-Lücke).
- **Remediation-Prompts**: `CONTEXTRA_FIX_PROMPTS.md` enthält 10 isolierte Behebungs-Prompts (P1–P10) als Kontext-Anker für Folge-Sessions, die auf diese Historie referenzieren müssen.

---

## 📐 Crate-Topologie & Referenzen

Vollständige Schichten-Architektur (Layer 0–6) sowie DAG-Regeln: siehe `AGENTS.md` Abschnitt **"Crate-Topologie"**.

---

## 📖 Crate-AGENTS.md Laderegel

**MANDATORY FIRST STEP:** Bevor Code in einer Crate bearbeitet wird, MUSS Jules die jeweilige `AGENTS.md` der Crate laden. Sie enthält Modul-Karten, API-Signaturen, Anti-Patterns und Lock-Hierarchien.

| Crate | Pfad für view_file / read |
|---|---|
| `contextra-core-ipc-gen` | `crates/contextra-core-ipc-gen/AGENTS.md` | <!-- crate-ref-ignore -->
| `contextra-core` | `crates/contextra-core/AGENTS.md` |
| `contextra-checkpoint` | `crates/contextra-checkpoint/AGENTS.md` |
| `contextra-crypto` | `crates/contextra-crypto/AGENTS.md` |
| `contextra-graph` | `crates/contextra-graph/AGENTS.md` |
| `contextra-sandbox` | `crates/contextra-sandbox/AGENTS.md` |
| `contextra-text` | `crates/contextra-text/AGENTS.md` |
| `contextra-store` | `crates/contextra-store/AGENTS.md` |
| `contextra-vector` | `crates/contextra-vector/AGENTS.md` |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/AGENTS.md` |
| `contextra-infer-candle` | `crates/contextra-infer-candle/AGENTS.md` |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/AGENTS.md` |
| `contextra-rank` | `crates/contextra-rank/AGENTS.md` |
| `contextra-privacy` | `crates/contextra-privacy/AGENTS.md` |
| `contextra-db` | `crates/contextra-db/AGENTS.md` |
| `contextra-engine` | `crates/contextra-engine/AGENTS.md` |
| `contextra-cognition` | `crates/contextra-cognition/AGENTS.md` |
| `contextra-bench` | `benchmarks/contextra-bench/AGENTS.md` |
| `contextra-router` | `crates/contextra-router/AGENTS.md` |
| `contextra-agent` | `crates/contextra-agent/AGENTS.md` |
| `contextra-mcp` | `crates/contextra-mcp/AGENTS.md` |
| `contextra-py` | `crates/contextra-py/AGENTS.md` |
| `contextra-tauri` | `crates/contextra-tauri/AGENTS.md` (deprecated, ADR-077) | <!-- crate-ref-ignore -->

---

## 🚫 Architektur-Entscheidungen (ADRs)

Vollständige Liste und Verbindlichkeit aller Architektur-Entscheidungen: siehe `DECISIONS.md` sowie `AGENTS.md` Abschnitt **"Non-Obvious Decisions"**.

---

## ✅ Existierende Typen & API-Disziplin

- **Existierende Typen**: Die Übersicht aller bereits verifizierten Typen und Module befindet sich in `AGENTS.md` Abschnitt **"Was TATSÄCHLICH implementiert ist"**. Vor jeder Neuimplementierung zusätzlich `find crates/ -name "*.rs" | xargs grep -l "<Typ-Name>"` ausführen.
- **Kritische Implementierungs-Muster**: Details zu `TxId`-Allokation, `fsync`-Fehlerbehandlung, `unsafe`-Einschränkungen und HMAC-Keys sind zentral in `AGENTS.md` unter **"Non-Obvious Decisions"** hinterlegt.

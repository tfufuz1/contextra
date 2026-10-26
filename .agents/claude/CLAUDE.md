# Contextra — Claude / Antigravity-Betriebsanleitung

<!-- doc-stamp: 2026-10-26 | Spec-Fassung: 2.3 -->

> Diese Datei gilt **ausschließlich** für Claude/Antigravity. Für Jules gilt `AGENTS.md §2a` + `.agents/jules/SESSION_BOOTSTRAP.md`.

## ⚠️ Subagenten: Sparsam verwenden

Subagenten nur für klar abgrenzbare, parallele Aufgaben. Sequenziell arbeiten, wo möglich.

- ✅ Subagenten für parallele, unabhängige Dokumentations- oder Analyse-Tasks
- ❌ Kein Aufteilen von Aufgaben, die Daten aus demselben Code-Zustand brauchen
- ✅ Kontingent-bewusst: lieber eine Datei mehr selbst lesen als delegieren
- ✅ Für Coding-Tasks: direkt im Kontext arbeiten

---

## Start

> **`AGENTS.md §2a` ist für Jules — Claude/Antigravity überspringt diesen Abschnitt vollständig.**

```bash
cargo check --workspace
```

Dann: `docs/SPEC.md` §0 lesen (LLM-Schnellstart, 3 Minuten).

---

## Ablauf

1. Task lesen, Ziel verstehen, Quellen der Wahrheit identifizieren.
2. APIs und Dateipfade im Code verifizieren — niemals aus dem Gedächtnis annehmen.
3. Kleinste mögliche Änderung. Kein Over-Engineering.
4. Verifizieren: Gates, Lints, Tests grün (`just check && just test`).
5. PR-Text formulieren (mit WP-Nummer aus PLAN.md).

---

## Kontingent clever nutzen

| Statt… | Besser… |
|---|---|
| Subagent starten um Datei zu lesen | `read_file` direkt aufrufen |
| Subagent für Suche | `grep`/`find` via `run_command` |
| Große Kontextmengen komplett laden | Gezielt nur relevante Abschnitte lesen |
| Ganzes SPEC.md lesen | §0 (Schnellstart) + gezielt §x für die Aufgabe |

---

## Kernregeln (aus AGENTS.md §4, SPEC §0.2)

1. **Zero-Panic:** Kein `unwrap()`, `expect()`, `panic!()` in Produktionspfaden (P7).
2. **Unsafe-Isolierung:** Nur in `contextra-simd`, `contextra-sys`, `contextra-wire`. Stets `// SAFETY:` (P2).
3. **Determinismus:** Zeit/Zufall/IDs nur über Ports `Clock`, `Rng`, `IdGen`. Kein `Instant::now()`, `thread_rng()`, `env::var()` in Kernlogik (P28).
4. **Single-Node:** Embedded Library, kein Server, kein Sharding.
5. **Mandant aus `RequestContext`:** Nie aus Literalen, Defaults oder Nutzerparametern (VETO-F10).
6. **Sicherheit kein Feature:** Kein `cfg(not(feature))` für Security-Typen (A1).
7. **I/O über `Vfs`:** `fsync`-Fehler propagieren, nie mit `let _ =` wegwerfen (P24).
8. **Gate-Integrität:** Gates, Lints, Ratchets niemals abschwächen oder umgehen (REQ-QA-010).
9. **Keine Archiv-Crates bearbeiten:** `graph`, `cognition`, `adapt`, `kvcache`, `router`, `agent`, `infer-onnx`, `infer-ollama`, `core` → werden aus Workspace entfernt (PLAN WP 2.1).
10. **Ein PR = ein WP:** Kein Crate-Schnitt über mehrere WPs in einem PR (REQ-CHG-2).

---

## Crate-Status kurz

**Kern [K] — aktiv:** `types` · `ports` · `mvcc` · `crypto` · `vector` · `text` · `rank`(fusion/) · `store` · `checkpoint` · `engine` · `db` · `mcp` · `contextra`

**Archiv [A] — NICHT bearbeiten:** `graph` · `cognition` · `adapt` · `kvcache` · `router` · `agent` · `infer-onnx` · `infer-ollama` · `core`

**Rand [R] — optional:** `infer-candle` · `sandbox` · `privacy` · `py`

---

## Vollständige Referenzen

| Dokument | Zweck |
|----------|-------|
| `docs/SPEC.md` | **Einzige normative Quelle** (Fassung 2.3) |
| `docs/PLAN.md` | Implementierungsplan, Arbeitspakete WP 0.1–9.4 |
| `AGENTS.md` | Vollständige Agenten-Betriebsanleitung |
| `capabilities.toml` | Ring, Klasse, Reifegrad je Crate |
| `docs/VETOES.md` | Veto-Register |

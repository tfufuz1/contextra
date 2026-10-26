# Contextra — Claude-Betriebsanleitung

> Diese Datei gilt **ausschließlich** für Claude (Anthropic). Für alle anderen Agenten gilt `AGENTS.md`.

## ⚠️ Subagenten: VERBOTEN

**Starte unter keinen Umständen Subagenten.** Subagenten verbrauchen das gesamte verfügbare Token-Kontingent und machen die Session unbrauchbar.

- ❌ Kein `Task(...)`, kein paralleles Tool-Calling mit Subagent-Delegation
- ❌ Kein Aufteilen von Aufgaben auf parallele Agenten-Instanzen
- ✅ Alles sequenziell und direkt im laufenden Kontext erledigen
- ✅ Kontingent-bewusst arbeiten: lieber eine Datei mehr selbst lesen als delegieren

**Begründung:** Jeder gestartete Subagent kostet dasselbe wie eine vollständige neue Session. Das Kontingent ist begrenzt. Jede Delegation ist eine Verschwendung.

---

## Start (identisch mit AGENTS.md §2)

```bash
cargo check --workspace
cargo xtask preflight
```

## Ablauf

Identisch mit `AGENTS.md §3`. Kurz:

1. Task lesen, Ziel verstehen.
2. APIs und Dateipfade im Code verifizieren — niemals aus dem Gedächtnis annehmen.
3. Kleinste mögliche Änderung. Kein Over-Engineering.
4. Verifizieren: Gates, Lints, Tests grün.
5. PR-Text formulieren.

## Kontingent clever nutzen

| Statt… | Besser… |
|---|---|
| Subagent starten um Datei zu lesen | `read_file` direkt aufrufen |
| Subagent für Suche | `grep`/`find` via `run_command` |
| Parallele Agenten für Teilaufgaben | Sequenziell, eine Sache nach der anderen |
| Große Kontextmengen laden | Gezielt nur relevante Abschnitte lesen |

## Kernregeln (aus AGENTS.md §5)

- **Zero-Panic:** Kein `unwrap()`, `expect()`, `panic!()` in Produktionspfaden.
- **Unsafe-Isolierung:** Nur in `contextra-simd`, `contextra-sys`, `contextra-wire`.
- **Single-Node:** Embedded Library, kein Server, kein Sharding.
- **Determinismus:** Zeit/Zufall/IDs nur über injizierte Ports (`Clock`, `Rng`, `IdGenerator`).
- **Gate-Integrität:** Gates, Lints, Ratchets niemals abschwächen oder umgehen (REQ-QA-010).

Vollständige Regeln: `AGENTS.md`, normative Spezifikation: `docs/SPEC.md`.

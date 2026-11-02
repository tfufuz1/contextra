# Routine: Chaos- und Fuzz-Langlauf

- **Takt:** nächtlich, 02:00 UTC (Workflow `chaos.yml`)
- **Ziel:** Ausführung der Crash-Matrix (T-CRASH-*) und der Fuzz-Ziele aus SPEC 15.1
- **Eingaben:** `.github/workflows/chaos.yml`; die Fuzz-Ziele liegen in den Ordnern `fuzz/` der Crates (z. B. `crates/contextra-store/fuzz`, `crates/contextra-vector/fuzz`). Ein Sammelrezept `just fuzz-all` existiert nicht.
- **Harte Grenzen:** höchstens 15 Dateien in einem Crate je PR. Scope: `crates/*/fuzz/**`, `crates/*/tests/**`.
- **Stopp:** Ein Absturz durch Einschränkungen der Hardware- oder Betriebssystemsimulation ist kein Fund im Code.
- **Ergebnis:** Fund mit Reproduktionsdaten als Artefakt. Eine Aufgabe nur, wenn der Fund bestätigt ist.
- **Eskalation:** siehe [`../HARNESS_QUICKSTART.md`](../HARNESS_QUICKSTART.md).

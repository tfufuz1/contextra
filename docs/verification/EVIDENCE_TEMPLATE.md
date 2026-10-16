# Contextra — Belegbericht / Evidence Report

**Datum:** YYYY-MM-DD HH:MM:SS UTC
**Git Commit SHA:** `<COMMIT_SHA>`
**Git Status:** `<CLEAN / DIRTY>`
**Toolchain:** Rust 1.89.0
**Umgebung:** `<UNAME_INFO>`
**Artefaktordner:** `evidence/<TIMESTAMP>-<SHORT_SHA>/`

---

## 1. Übersicht der Garantien & Belege

Die folgende Tabelle dokumentiert die durchgeführten Verifikationsschritte und deren Ergebnisse. Jeder Schritt verweist auf die zugehörige Roh-Logdatei und spezifiziert explizit die Grenzen des jeweiligen Nachweises.

| Garantie | Beleg-Befehl | Ergebnis | Artefaktpfad | Was dieser Beleg NICHT beweist |
| :--- | :--- | :---: | :--- | :--- |
| *Beispiel: Code-Formatierung* | `cargo fmt --check` | PASSED | `step_01_cargo_fmt.log` | Beweist keine logische Korrektheit oder Abwesenheit von Laufzeitfehlern. |
| *Beispiel: Crash-Durability* | `cargo test --test crash_kill_durability --locked` | MISSING | `step_04_crash_kill_durability.log` | Testdatei derzeit noch nicht im Repository implementiert. |
| *Beispiel: Unsafe-Isolierung* | `cargo xtask check-unsafe-islands` | FAILED | `step_09_xtask_check_unsafe_islands.log` | Prüft nur Quellcode-Attribute, keine Binär-Disassemblierung. |

---

## 2. Bekannte Grenzen (Known Limitations)

1. **Keine Hardware-Powerfail-Tests:**
   - Die Durability- und WAL-Recovery-Tests simulieren Prozess-Crashes (`SIGKILL`) auf Software-Ebene.
   - Es werden keine echten Hardware-Power-Loss-Events (PLP / Power-Loss Protection, plötzlicher physikalischer Stromausfall ohne Disk-Sync) geprüft.

2. **Kein externes Crypto-Review:**
   - Die kryptografischen Invarianten (Blake3 Audit Chain, Ed25519 Signaturen, AES-256-GCM-SIV Encrypted Vaults) werden gegen offizielle Testvektoren und interne Referenzmodelle geprüft.
   - Ein unabhängiges Third-Party-Krypto-Audit steht derzeit noch aus.

3. **VM-Benchmark-Varianz:**
   - Benchmarks und Latenzmessungen, die in virtuellen Maschinen (VMs) oder CI/CD-Runnern ausgeführt werden, unterliegen Schwankungen durch CPU-Stealing, Noisy Neighbors und dynamsiches Scheduling.
   - Absolute Durchsatzwerte sind daher nicht als Garantie für Bare-Metal-Produktivumgebungen zu verstehen.

4. **Fehlende/Incomplete Garantie-Tests:**
   - Noch nicht im Codebase existierende Garantie-Tests werden von den Evidence-Skripten als `MISSING` (nicht bestanden) ausgewiesen und nicht künstlich als erfolgreich deklariert.

---

## 3. Artefakt-Manifest & Integrität

Die vollständigen Roh-Logs und maschinenlesbaren Ergebnisse befinden sich im Artefaktordner:

- `env.json`: Vollständige Systemdiagnose (CPU, RAM, Rustc, Cargo, Git Status).
- `results.jsonl`: Zeilenweise Zusammenfassung aller Schritte inkl. Exit-Codes, Dauern und Test-Zählungen.
- `MANIFEST.sha256`: SHA-256 Checksummen aller erzeugten Artefakt-Dateien zur Fälschungssicherheit.

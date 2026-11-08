# SPEC: `contextra-sandbox` (Ring 2, Klasse A)

> **Normativer Status:** Verbindliche Spezifikation gemäß SPEC v4.0 §2, §3, §4, §5.1 (Klasse A Baustein).
> **Sicherheits-Invariante:** `#![forbid(unsafe_code)]` im gesamten Produktions- und Testcode.
> **Ring-Zugehörigkeit:** Ring 2 (WASM Execution Boundary & Isolation Sandbox).

---

## 1. Zweck & Abgrenzung

`contextra-sandbox` stellt eine isolierte, eng begrenzte WASM-Ausführungsumgebung für die `CodeExecution`-Permission sowie für LSM-Tree-Filter- und Merge-Operatoren bereit.

### Ziele
- **Deterministische CPU-Drosselung:** Fuel-Metering verhindert Endlosschleifen (INV-SBX-1).
- **Strikte Speichergrenzen:** WASM-Speicher und Tabellengröße werden durch `wasmtime::ResourceLimiter` drosselbar begrenzt (INV-SBX-2).
- **Least-Privilege-Sicherheit:** Deny-by-default für Dateisystem-, Netzwerk-, PRNG- und Cloud-Egress-Zugriffe.
- **Fail-Closed-Fehlerbehandlung:** Alle Grenzüberschreitungen und Traps enden in typisierten Fehlern (`SandboxError`), ohne Prozessabbrüche oder Panics.
- **Vollständige Entkopplung:** 0 interne Abhängigkeiten; `wasmtime`, `tokio` und `anyhow` liegen strikt hinter dem optionalen Feature `wasm` (B6).

### Nicht-Ziele
- Kein allgemeiner WASM-Compiler (nur Ausführung vorab kompilierter `.wasm`-Binaries).
- Keine Löschketten-, Saga- oder Deletion-Proof-Logik (§6).
- Kein permanenter Dateisystem-Zustand.

---

## 2. Invarianten (DbC)

### INV-SBX-1: Deterministische Fuel-Drosselung & CPU-Budget
Jede WASM-Ausführung erfolgt mit einem konfigurierten Fuel-Budget (`max_fuel`).
- Wenn das Fuel-Budget vor Abschluss der Ausführung erschöpft ist, bricht die Runtime garantiert mit `SandboxError::FuelExhausted { consumed }` ab.
- Wall-Clock-Timeout (`max_wall_clock_ms`) dient als orthogonales Sicherheitsnetz gegen I/O- oder Thread-Hangs und terminiert mit `SandboxError::Timeout`.

### INV-SBX-2: Strikte Speichergrenzen & Memory Growth
WASM-Memory- und Tabellenwachstum werden über den `SandboxState`-Ressourcenbegrenzer kontrolliert.
- Allokationen über `max_memory_pages` (64 KB pro Page) oder `max_table_entries` werden durch `memory_growing` bzw. `table_growing` abgewiesen (`Ok(false)`).
- Jeder Versuch, das Limit zu überschreiten, resultiert in einem kontrollierten Fehler ohne Panics oder unkontrollierte OOMs des Host-Prozesses.

### INV-SBX-3: Capability-Whitelisting (Deny-by-default)
WASM-Gäste besitzen standardmäßig:
- Keinen Zugriff auf das Host-Dateisystem (`allow_filesystem = false`).
- Keinen Zugriff auf Host-Netzwerk-Sockets (`allow_network = false`).
- Keinen Zugriff auf die Host-Uhr (`allow_clock = false`).
- Keinen Zugriff auf unkontrollierte Entropie (`random_seed = None`).
- Keinen Zugriff auf Cloud-Egress (`allow_cloud_egress = false`).
Verletzungen führen zu `SandboxError::CapabilityViolation`.

### INV-WASM-DET-1: Multi-Architektur-Determinismus
Gleiches WASM-Modul, gleiche Eingabebytes und gleicher Fuel-Wert führen über x86-64 und AArch64 hinweg zu bit-identischer Ausgabe (`stdout`) und exakt identischem Fuel-Verbrauch.

### INV-OUTPUT-ZEROIZE: ZeroizeOnDrop für sensible Daten
Die `stdout`-Ausgabe (`WasmOutput.stdout`) implementiert `ZeroizeOnDrop` und wird beim Verlassen des Gültigkeitsbereichs im Speicher genullt (P9).

---

## 3. Sicherheits- und Vulnerabilitäts-Audit

### RUSTSEC-2026-0269 (Wasmtime Trailing Slash Symlink Escape)
- **Status:** Geprüft und abgesichert.
- **Hintergrund:** In Wasmtime vor `36.0.14` führten Trailing Slashes bei Symlink-Auflösungen zu potenziellen Filesystem-Escapes.
- **Auflösung:** `contextra-sandbox` verwendet Wasmtime Version `36.0.19` (gepatcht ab `36.0.14`), womit die Schwachstelle behoben ist.
- **Regressionstest:** `test_abuse_filesystem_trailing_slash_symlink_escape_blocked` in `tests/abuse_test_matrix.rs` stellt sicher, dass Pfadmanipulationen mit Trailing Slashes fail-closed abgewiesen werden.

---

## 4. Fehlermodell (`SandboxError`)

Alle Fehlermöglichkeiten sind im typisierten Enum `SandboxError` (`#[non_exhaustive]`) zusammengefasst:
- `Timeout { timeout_ms }`: Wall-Clock-Timeout überschritten.
- `MemoryExceeded { pages, max_pages }`: Speicherobergrenze erreicht.
- `FuelExhausted { consumed }`: CPU-Fuel-Budget aufgebraucht.
- `CapabilityViolation { capability }`: Unerlaubter Zugriff auf unberechtigte Subsysteme.
- `WasmTrap(String)`: WASM-Trap (z. B. Division durch 0, unreachable).
- `InvalidModule(String)`: Ungültiges WASM-Binary oder Größenüberschreitung.
- `OutputLimitExceeded { stream, limit }`: Stdout/Stderr-Pufferüberlauf.
- `InputTooLarge { len, limit }`: Stdin-Eingabelimit überschritten.
- `ProcessExit { code }`: Nicht-Null-Beendigung von `proc_exit`.
- `Runtime(String)`: Interner Wasmtime-Initialisierungs- oder Ausführungsfehler.

---

## 5. Nebenläufigkeits- & Zustandsmodell

- **Kein Shared State:** Jede Ausführung erhält einen frischen `Store` und eine isolierte Instanz.
- **Wait-Free:** Keine Sperren über `.await`-Pfade (`await_holding_lock = deny`).
- **Thread-Safety:** `WasmExecutor` ist `Send + Sync` und kann nebenläufig für unabhängige Ausführungen genutzt werden.

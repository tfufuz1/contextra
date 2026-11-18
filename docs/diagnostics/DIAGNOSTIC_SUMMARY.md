# Gesamtdiagnose & Systemzustand der Rust-Codebase

> Generiert durch `tools/orchestrate_diagnostics.py`.
> Ausführungszeitpunkt: automatisiert durch Principal Agentic Engineering.

## Übersicht der ausgeführten Diagnose-Module

| Modul | Analysebereich | Status | Dokumentation | Maschinendaten |
|---|---|---|---|---|
| `gate_layering.py` | Layering & Abhängigkeiten (SPEC §2.3 R1-R3) | **BEFUNDE** | [`GATE_LAYERING.md`](GATE_LAYERING.md) | [`gate_layering.json`](gate_layering.json) |
| `gate_dependencies.py` | Gate G1 Closure Compliance (SPEC §2 / capabilities v3) | **PASS** | [`GATE_DEPENDENCIES.md`](GATE_DEPENDENCIES.md) | [`gate_dependencies.json`](gate_dependencies.json) |
| `zero_panic_analyzer.py` | Zero-Panic & Unsafe Indexing (INV-4, R6) | **PASS** | [`ZERO_PANIC_AUDIT.md`](ZERO_PANIC_AUDIT.md) | [`zero_panic_audit.json`](zero_panic_audit.json) |
| `unsafe_boundary_tracer.py` | Unsafe Boundary & Proof Tracer (INV-3, R5) | **FEHLER** | [`UNSAFE_BOUNDARY_AUDIT.md`](UNSAFE_BOUNDARY_AUDIT.md) | [`unsafe_boundary_audit.json`](unsafe_boundary_audit.json) |
| `erasure_leak_prober.py` | DSGVO-Löschung & ReadGate Leaks (SPEC §5, §6) | **FEHLER** | [`ERASURE_LEAK_AUDIT.md`](ERASURE_LEAK_AUDIT.md) | [`erasure_leak_audit.json`](erasure_leak_audit.json) |
| `determinism_guard.py` | Determinismus & Zeitquellen (INV-5) | **PASS** | [`DETERMINISM_GUARD_AUDIT.md`](DETERMINISM_GUARD_AUDIT.md) | [`determinism_guard_audit.json`](determinism_guard_audit.json) |
| `type_registry_sync.py` | Typ-Register & Kollisionen (R4) | **PASS** | [`TYPE_REGISTRY_COLLISIONS.md`](TYPE_REGISTRY_COLLISIONS.md) | [`type_registry_collisions.json`](type_registry_collisions.json) |
| `async_boundary_scanner.py` | Async Boundary & Lock-Across-Await (M8) | **PASS** | [`ASYNC_BOUNDARY_REPORT.md`](ASYNC_BOUNDARY_REPORT.md) | [`async_boundary_report.json`](async_boundary_report.json) |
| `fsync_durability_tracer.py` | Fsync & Durability Tracer (INV-FSYNC-1) | **PASS** | [`FSYNC_DURABILITY_AUDIT.md`](FSYNC_DURABILITY_AUDIT.md) | [`fsync_durability_audit.json`](fsync_durability_audit.json) |
| `port_contract_auditor.py` | Port Contracts & ErasureLayer (SPEC §6, §7) | **PASS** | [`PORT_CONTRACT_AUDIT.md`](PORT_CONTRACT_AUDIT.md) | [`port_contract_audit.json`](port_contract_audit.json) |
| `dead_code_microprobe.py` | Dead Code & Orphan Symbols | **FEHLER** | [`DEAD_CODE_ANALYSIS.md`](DEAD_CODE_ANALYSIS.md) | [`dead_code_analysis.json`](dead_code_analysis.json) |
| `error_topology_mapper.py` | Error Topology & Data Leaks (INV-LOG-1) | **PASS** | [`ERROR_TOPOLOGY_REPORT.md`](ERROR_TOPOLOGY_REPORT.md) | [`error_topology_report.json`](error_topology_report.json) |
| `test_matrix_advisor.py` | Test Matrix & Pyramid Advisor | **PASS** | [`TEST_MATRIX_ADVISOR.md`](TEST_MATRIX_ADVISOR.md) | [`test_matrix_advisor.json`](test_matrix_advisor.json) |

## Schnelleinstieg & Werkzeugverzeichnis für Coding-Agenten

Alle Werkzeuge können einzeln aufgerufen oder via `tools/orchestrate_diagnostics.py` als Gesamtsuite ausgeführt werden:

- **`python3 tools/gate_layering.py`**: Layering & Abhängigkeiten (SPEC §2.3 R1-R3)
- **`python3 tools/gate_dependencies.py`**: Gate G1 Closure Compliance (SPEC §2 / capabilities v3)
- **`python3 tools/zero_panic_analyzer.py`**: Zero-Panic & Unsafe Indexing (INV-4, R6)
- **`python3 tools/unsafe_boundary_tracer.py`**: Unsafe Boundary & Proof Tracer (INV-3, R5)
- **`python3 tools/erasure_leak_prober.py`**: DSGVO-Löschung & ReadGate Leaks (SPEC §5, §6)
- **`python3 tools/determinism_guard.py`**: Determinismus & Zeitquellen (INV-5)
- **`python3 tools/type_registry_sync.py`**: Typ-Register & Kollisionen (R4)
- **`python3 tools/async_boundary_scanner.py`**: Async Boundary & Lock-Across-Await (M8)
- **`python3 tools/fsync_durability_tracer.py`**: Fsync & Durability Tracer (INV-FSYNC-1)
- **`python3 tools/port_contract_auditor.py`**: Port Contracts & ErasureLayer (SPEC §6, §7)
- **`python3 tools/dead_code_microprobe.py`**: Dead Code & Orphan Symbols
- **`python3 tools/error_topology_mapper.py`**: Error Topology & Data Leaks (INV-LOG-1)
- **`python3 tools/test_matrix_advisor.py`**: Test Matrix & Pyramid Advisor

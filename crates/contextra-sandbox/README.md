# contextra-sandbox

WASM Execution Boundary und Isolations-Sandbox.

Normativitätsklasse: **Klasse A Baustein** (Ring 2, `stable`; §5.1, `capabilities.toml`).

`contextra-sandbox` stellt eine vollständig isolierte, eng begrenzte WASM-Ausführungsgrenze bereit (Ring 2).

## Zweck

Führt vorab kompilierte WASM-Binaries für Agenten-Tools und LSM-Merge-Operatoren aus. Erzwingt strikte Fuel- (Rechenschritt), Speicher- und Wall-Clock-Timeout-Grenzen.

## Ring-Zugehörigkeit & Baukasten-Status

- **Ring:** Ring 2 (Blatt-Crate / WASM Execution Boundary)
- **Klasse:** Klasse A Baustein (B1–B12 konform)
- **Modularitäts-Score:** 100/100 autark (0 interne Abhängigkeiten)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`
- **Zero-Panic:** Keine unwrap/expect/panic! im Produktivcode (P2/B9).

## Feature-Flags

- `default = []`: Minimaler Fußabdruck ohne Wasmtime-Kompilierung. Enthält `WasmCapabilities`, `MergeOperatorCapabilities`, `ApprovalRequest`, `ApprovalRisk`, `ApprovalStatus`, `WasmOutput`, `SandboxError`.
- `wasm`: Aktiviert die WASM-Laufzeitumgebung (`WasmExecutor`, `WasmMergeFunction`, WASI Preview 1 Emulation via `wasmtime 36` und `tokio`).

## Abhängigkeitsmenge

- **Workspace-Abhängigkeiten:** keine (vollkommen autarker Blattknoten).
- **Kern-Abhängigkeiten (ohne Features):** `zeroize`, `thiserror`, `tracing`.
- **Optionale Abhängigkeiten (Feature `wasm`):** `wasmtime`, `tokio`, `anyhow`.

## Garantien & Invarianten

1. **Kein Shared Mutable State:** Pro Aufruf wird eine frische `Store`- und `Instance`-Instanz alloziert; kein Zustandsübertrag zwischen Aufrufen.
2. **Deterministische Ressourcendrosselung:**
   - Fuel-Budget (CPU-Ticks) verhindert Endlosschleifen (INV-SBX-1).
   - `ResourceLimiter` drosselt WASM-Speicher und Tabellengröße strikt (INV-SBX-2).
   - Wall-Clock-Timeout als orthogonales Sicherheitsnetz.
3. **Least Privilege & Approval:**
   - Deny-by-default für Dateisystem-, Netzwerk- und Cloud-Egress-Zugriffe.
   - Deterministische Risikoklassifikation (`ApprovalRisk`) und Zustandsautomat (`ApprovalRequest`).
4. **Multi-Architektur-Determinismus (INV-WASM-DET-1):**
   - Bit-identische Ausgaben und deterministischer Fuel-Verbrauch über x86-64 und AArch64.
5. **ZeroizeOnDrop (INV-OUTPUT-ZEROIZE):** Sensitiver Standard-Output (`WasmOutput.stdout`) wird beim Verlassen des Gültigkeitsbereichs sicher genullt (P9).
6. **Vulnerabilitäts-Prüfung:**
   - Schutz vor Symlink-Escapes mit Trailing Slashes (`RUSTSEC-2026-0269`) durch Verwendung von Wasmtime 36.0.19 (gepatcht ab 36.0.14) und validiert via `tests/abuse_test_matrix.rs`.

## Export in Drittprojekte

Der Baustein kann über das Export-Tool direkt in isolierte Drittprojekte kopiert werden:

```bash
python3 tools/export_crate.py contextra-sandbox --out /pfad/zu/deinem/projekt
```

## Minimalbeispiel (Drittprojekt)

```rust
use contextra_sandbox::{WasmCapabilities, ApprovalRequest, ApprovalRisk, classify_risk};

// Risikoprüfung
let caps = WasmCapabilities::default();
assert_eq!(classify_risk(&caps), ApprovalRisk::Low);

// Genehmigungs-Workflow
let mut req = ApprovalRequest::new("req-1".into(), &caps, 1_000, 5_000);
req = req.approve("admin".into(), 2_000).unwrap();
```

Ein vollständiges Ausführungsbeispiel mit WASM-Runtime findet sich unter `examples/standalone_sandbox.rs`.

## Benchmarks

```bash
cargo bench -p contextra-sandbox --bench sandbox_bench
```

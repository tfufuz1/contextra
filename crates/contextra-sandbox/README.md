# contextra-sandbox

WASM Execution Boundary und Isolations-Sandbox.

Normativitätsklasse: **Rand `[R]`** (Ring 2, `stable`; §6.0, `capabilities.toml`).

`contextra-sandbox` stellt eine isolierte, eng begrenzte WASM-Ausführungsgrenze bereit (Ring 2).

## Zweck

Führt vorab kompilierte WASM-Binaries für Agenten-Tools und LSM-Merge-Operatoren aus. Erzwingt strikte Fuel- (Rechenschritt), Speicher- und Wall-Clock-Timeout-Grenzen.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 2 (Blatt-Crate / WASM Execution Boundary)
- **Status:** 🟢 Fertig (Baustein 100/100 autark)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`
- **Zero-Panic:** Keine unwrap/expect/panic! im Produktivcode.

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
   - Wall-Clock-Timeout als deterministisches/sicheres Abbruchkriterium.
3. **Least Privilege & Approval:**
   - Deny-by-default für Dateisystem-, Netzwerk- und Cloud-Egress-Zugriffe.
   - Deterministische Risikoklassifikation (`ApprovalRisk`) und Zustandsautomat (`ApprovalRequest`).
4. **ZeroizeOnDrop:** Sensitiver Standard-Output (`WasmOutput.stdout`) wird beim Verlassen des Gültigkeitsbereichs sicher genullt (P9).

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

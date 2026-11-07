# AGENTS.md — `contextra-sandbox`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-sandbox` stellt eine strikt isolierte WebAssembly (WASM) Ausführungsumgebung (`WasmExecutor`) auf Basis von Wasmtime bereit (Ring 2).
Sie dient als sichere Sandbox für dynamische Berechnungen, untrusted Plugins und MCP-CodeExecution-Befehle (§4.18).
Als autarker Baustein erzwingt sie Default-Deny-Rechte, deterministisches Fuel-Budgeting und strenge Ressourcen-Limits, ohne Seiteneffekte auf den Host-Prozess zu erlauben.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 2 (Sicherheits-Sandbox & WASM Boundary)
- **Klasse:** Erweiterung `[E]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** Keine Workspace-Abhängigkeiten (Spec §6.9, `capabilities.toml`) — maximale Autarkie.

## 3. Garantien & Architektur-Invarianten
- **Strikte Speichersicherheit:** `#![forbid(unsafe_code)]` wird über das gesamte Crate hinweg erzwungen.
- **Default-Deny Capabilities:** Kein Dateisystem-, kein Netzwerk-, kein Umgebungs- und kein Systemuhr-Zugriff im Gast (`WasmCapabilities::default()`).
- **Deterministische Limits:** Strikte Fuel-Obergrenzen (CPU-Fuel), WASM-Stack-Limits (512 KB) und Host-Call-Zähler.
- **Doppelte Begrenzung:** Jede Ausführung wird durch deterministisches Fuel UND ein synchrones Wall-Clock-Timeout beschränkt.
- **State Isolation per Execution:** Jeder `execute()`-Aufruf instantiiert einen frischen `Store` und eine neue `Instance`; keinerlei Zustandsleckagen zwischen Läufen.
- **Zeroize-on-Drop:** Sensibler Host- und Guest-Output (`WasmOutput`) wird beim Verlassen des Scopes genullt.

## 4. Verbotene Muster
- Jegliche Verwendung von `unsafe` im Sandbox-Code.
- Wiederverwendung von WASM-Stores oder -Instanzen über Aufrufgrenzen hinweg.
- Weglassen von Fuel-Budgets oder Timeouts bei Guest-Code-Ausführung.
- Direkte Dateisystem- oder Netzwerkdurchgriffe für den WASM-Gast.

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Exportiert:** `WasmExecutor`, `WasmCapabilities`, `WasmOutput`, `ApprovalRequest`.
- **Bereitgestellt für:** `contextra-mcp`, `contextra-engine`.

## 6. Tests, Sabotage & Qualitätssicherung
- **Resource Exhaustion Tests:** Verifikation des Fuel-Erschöpfungs-Abbruchs bei Endlosschleifen im Guest.
- **Memory Leak & Escape Tests:** Prüfung auf saubere Isolation und Speicherfreigabe nach Guest-Panics.
- **Sabotage-Testing:** Injektion bösartiger WASM-Binaries mit versuchten Host-Calls.

## 7. Offene Punkte & Technische Schulden
- Keine.

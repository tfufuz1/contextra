# ADR-0XX: contextra-sandbox — WASM Ausführungsgrenze & Bedrohungsmodell

* **Status:** Accepted
* **Datum:** 2026-09-13 (Aktualisiert: 2026-09-30)
* **Kontext / Auslöser:**
  Für dynamische Benutzerskripte, unkonfigurierte Merge-Operatoren und MCP-Tool-Ausführungen (`CodeExecution`-Permission) benötigt Contextra eine sichere, isolierte Ausführungsumgebung (Sandbox) im In-Process-Betrieb.
  Sicherheitskritischer Gast-Code darf unter keinen Umständen Wirtsspeicher lesen, Dateisystem-Zugriffe außerhalb freigegebener Scopes durchführen, unbegrenzte CPU-Ressourcen verbrauchen oder das Gesamtsystem zum Absturz bringen.

## Entscheidung

Das Crate **`contextra-sandbox`** stellt die Ausführungsgrenze für isolierten WASM-Gastcode bereit [V].
Als Sandboxing-Backend kommt **`wasmtime`** zum Einsatz. Das Crate erzwingt am Root `#![forbid(unsafe_code)]` [V].
Ausführungen erfordern eine strikte Vorab-Admission (`AdmittedModule` via `ModuleVerifier` / `Ed25519ModuleVerifier`), beschränkte Ressourcen (Fuel-Limiting + Wall-Clock-Timeout) sowie eine deterministische CRDT-Verifikation für Merge-Operatoren (`MergeOperatorRegistry::admit`) [V].

## Alternativen-Evaluierung

### Option A: `wasmtime` (Gewählt)
* **Vorteile:** Offizielles Projekt der Bytecode Alliance; exzellente WebAssembly-Spezifikationskonformität; Pure-Rust-Schnittstelle ohne C-FFI-Zwang; ausgereifte Fuel-Metering- und Memory-Limit-Mechanismen [V].
* **Nachteile:** Moderate Erhöhung von Kompilierzeit und Binärgröße.
* **Bewertung:** Akzeptiert.

### Option B: `wasmer`
* **Vorteile:** Gute Performanz; flexible Compiler-Backends (Cranelift, LLVM, Singlepass).
* **Nachteile:** Komplexere C-FFI-Abhängigkeiten bei bestimmten Backends; historisch wechselhafteres API-Design; höheres Risiko bezüglich Pure-Rust-Kompatibilität.
* **Bewertung:** Verworfen.

### Option C: Prozess-Isolation über Subprozesse & `seccomp`
* **Vorteile:** Native Betriebssystem-Isolation (OS-Level Isolation, namespaces, `seccomp-bpf`).
* **Nachteile:** Stark plattformabhängig (Linux-only, nicht portabel auf Windows/macOS); hoher IPC-Overhead für frequentierte In-Memory Lookups; widerspricht der "Single-Process Embedded Library"-Architektur.
* **Bewertung:** Verworfen.

## Bedrohungsmodell (Threat Model)

1. **Ressourcen-Erschöpfung (Denial of Service):**
   * *Bedrohung:* Endlosschleifen oder speicherintensive Allokationen im Gastcode.
   * *Gegenmaßnahme:* Metering über `wasmtime`-Fuel (`consume_fuel`), harter Wall-Clock-Timeout und strikte Max-Memory-Limits per Instanz (`ResourceLimiter`).
2. **Speicher-Lese-/Schreib-Escapes (Escape from Sandbox):**
   * *Bedrohung:* Out-of-Bounds Memory-Reads/Writes zur Manipulation des Wirtsspeichers.
   * *Gegenmaßnahme:* WebAssembly Linear Memory Isolation; Rust/Wasmtime Safe Memory Boundaries.
3. **Schadcode-Einschleusung (Unverified Code Execution):**
   * *Bedrohung:* Ausführung nicht verifizierter Binärdateien mit bösartigen Seiteneffekten.
   * *Gegenmaßnahme:* Kapselung in `AdmittedModule`; Verifikation über kryptografische Signaturen (`Ed25519ModuleVerifier`) und Policy-Check vor der Instanziierung.
4. **Seiteneffekte & Netzwerk-Exfiltration:**
   * *Bedrohung:* Unbefugter Zugriff auf Host-APIs, Sockets oder lokale Dateien.
   * *Gegenmaßnahme:* Keine System-Calls (WASI) standardmäßig importiert; Host-Funktionen strikt auf anonyme In-Memory-Puffer beschränkt.

## Konsequenzen

* **Echte Execution-Isolation:** Gast-Code läuft sicher und isoliert im Wirtsprozess [V].
* **Determinismus & Sicherheit:** Transparente Zertifizierung und Ressourcenkontrolle [V].

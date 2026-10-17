# ADR-N25: F10 Panic-Isolation an der öffentlichen API (guard_panic und Poisoned-Zustand)

* **Status:** Accepted
* **Datum:** 2026-09-30
* **Kontext / Auslöser:**
  Gemäß der Projektverfassung (CONSTITUTION.md) und der Zero-Panic-Invariante (P7) dürfen öffentliche Bibliotheks-Schnittstellen niemals paniken.
  Treffen jedoch unvorhergesehene Laufzeitfehler, Bugs in Drittanbieter-Dependencies oder abgebrochene Invarianten auf den Ausführungspfad, droht ein Panic-Unwinding über FFI-Grenzen hinweg (z. B. in Python PyO3, C-FFI oder WASM-Gastprozesse). Ein Unwinding über FFI-Schnittstellen ist in Rust undefiniertes Verhalten (Undefined Behavior / Undefined Panic Behavior) und führt zum abrupten Absturz des einbettenden Wirtsprozesses.

## Entscheidung

1. **Panic-Isolation an allen öffentlichen API-Grenzen:**
   Sämtliche öffentlichen API-Einsprungpunkte (Public Facade `contextra`, FFI-Hüllen in `contextra-py`, MCP-Handler in `contextra-mcp`) kapseln ihre interne Ausführung über eine **Panic-Boundary** (`guard_panic` / `std::panic::catch_unwind`) [V].

2. **Überführung in den `Poisoned`-Zustand:**
   Wird eine Panic im internen Kern abgefangen, schlägt der Aufruf sicher mit einem strukturierten `ContextraError::Internal` oder FFI-Fehlercode fehl [V]. Die betroffene Instanz geht unverzüglich in einen unumkehrbaren **`Poisoned` (Vergiftet)** Zustand über [V].

3. **Fail-Closed bei Folgeaufrufen:**
   Sobald eine Instanz den `Poisoned`-Zustand erreicht hat, lehnt sie sämtliche nachfolgenden Lese- und Schreiboperationen sofort ab (`Err(ContextraError::Internal("Instance is poisoned due to a previous panic"))`), um Datenkorruption oder inkonsistente In-Memory-Zustände zu verhindern [V].

## Alternativen

### Option A: Panics über FFI- und Bibliotheksgrenzen durchreichen lassen
* **Vorteile:** Keine `catch_unwind`-Kapselung erforderlich.
* **Nachteile:** Undefiniertes Verhalten bei FFI; harter Prozessabsturz der Wirtsanwendung (z. B. Python-Kernel oder WASM-Host).
* **Bewertung:** Verworfen (verletzt P7 Zero-Panic) [S].

### Option B: Sofortiger Prozess-Abort (`std::process::abort()`)
* **Vorteile:** Einfachste Implementierung.
* **Nachteile:** Zerstört den Wirtsprozess; inakzeptabel für eine eingebettete Bibliothek.
* **Bewertung:** Verworfen [S].

## Konsequenzen

* **Prozess-Sicherheit:** Die Wirtsanwendung stürzt bei internen Unerwarteten Fehlern nicht ab [V].
* **Keine Silent Corruption:** Der `Poisoned`-Zustand garantiert, dass nach einer Panic keine potenziell korrupten In-Memory-Zustände weiterverwendet werden [V].
* **FFI-Sicherheit:** 100% UB-freie FFI-Grenzen für Python und C [V].

## Reversibilitäts-Bedingung

Eine Umstellung erfordert Sprach- und Plattformgarantien von Rust und FFI-Runtimes, die sicheres Panic-Unwinding über beliebige Sprachgrenzen ohne UB garantieren [S].

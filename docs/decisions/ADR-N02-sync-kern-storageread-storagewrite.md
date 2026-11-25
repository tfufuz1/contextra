# ADR-N02: Sync-Kern — StorageRead synchron, StorageWrite asynchron (begrenzter ComputePool)

* **Status:** Final
* **Datum:** 2026-09-22
* **Kontext / Auslöser:**
  Die Ring-0-Speicherkerne (`contextra-vector`, `contextra-text`, `contextra-graph`) bilden das fundamentale Hochleistungs-Fundament für In-Memory Lookups und Traversierungen im Hot-Path.
  Das direkte Mischen von Asynchronitäts-Laufzeiten (`tokio`) in reinen In-Memory-Suchalgorithmen erzeugt unnötigen Runtime-Overhead, erschwert die formale Korrektheitsanalyse und birgt die Gefahr von Thread-Explosionen durch unbegrenzte `spawn_blocking`-Aufrufe bei hoher paralleler Leselast.

  Gleichzeitig erfordern Disk-Persistierungs- und Schreiboperationen (`StorageWrite`, SSTable-/WAL-Flushes) asynchrone I/O-Orchestrierung. Es bedarf einer klaren Trennung zwischen synchronen Kern-Lese-Zugriffen und asynchronen Schreib-Workflows.

## Entscheidungen

1. **Entkopplung der Ring-0-Speicherkerne von `tokio`:**
   Lese-Operationen über die Schnittstelle `StorageRead` innerhalb von Ring 0 werden streng synchron ausgeführt. Die Kern-Suchindizes (`csr.rs`, `inverted.rs`, `diskann.rs`) enthalten keine direkten `tokio`-Abhängigkeiten.

2. **Verlagerung von Persistenz-Aufrufen (`StorageWrite`):**
   Schreib-, Mutations- und Persistierungsoperationen (`StorageWrite`, z. B. `persist_delta()`, SSTable- / WAL-Flushes) werden aus den Ring-0-Modulen heraus gelöst und in die übergeordnete Orchestrierungsschicht (`contextra-engine`, Ring 3) verlagert.

3. **Begrenzter `ComputePool` für CPU-intensive Tasks:**
   Unbegrenzte `tokio::task::spawn_blocking`-Aufrufe bei parallelen Lese- und Indexierungs-Workloads werden durch einen kapazitätsbegrenzten, dedizierten `ComputePool` in `contextra-engine` ersetzt. Dies garantiert harte Obergrenzen für zeitgleiche Thread-Belegungen.

## Konsequenzen

* **Hot-Path Lese-Performanz:** In-Memory-Lese-Pfade in Ring 0 laufen ohne Async-Context-Switches ab und bieten vorhersehbare Microsecond-Latenzen (p99-Budget $\le +3\%$ vs. Frozen Baseline).
* **Zero-`tokio`-Invariante in Ring 0:** `cargo tree -e normal -p contextra-vector`, `-p contextra-text` und `-p contextra-graph` zeigen keine `tokio`-Laufzeitabhängigkeit.
* **Resilienter Ressourcen-Schutz:** Der begrenzte ComputePool schützt den Server vor Thread-Contention und Memory-Pressure bei hoher paralleler Last.

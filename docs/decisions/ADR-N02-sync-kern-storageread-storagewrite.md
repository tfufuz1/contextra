# ADR-N02: Sync-Kern — Synchroner Ring-0-Lese-Pfad und einheitliches StorageEngine-Trait

* **Status:** Accepted (aktualisiert zur Übereinstimmung mit dem Quellcode)
* **Datum:** 2026-09-22 (Aktualisiert: 2026-09-30)
* **Kontext / Auslöser:**
  Die Ring-0-Speicherkerne (`contextra-vector`, `contextra-text`, `contextra-graph`) bilden das fundamentale Hochleistungs-Fundament für In-Memory-Lookups und Traversierungen im Hot-Path.
  Das direkte Mischen von Asynchronitäts-Laufzeiten (`tokio`) in reinen In-Memory-Suchalgorithmen erzeugt unnötigen Runtime-Overhead und erschwert die Korrektheitsanalyse.
  Im ursprünglichen Entwurf wurden getrennte Traits `StorageRead` und `StorageWrite` skizziert. Im Quellcode wird die Speicherschnittstelle hingegen einheitlich über das trait **`StorageEngine`** (`contextra-core` / `contextra-testkit` / `contextra-store`) abstrahiert, welches asynchrone Schreib- und Persistierungsmethoden mittels `BoxFuture` bereitstellt [V].
  Dieses ADR dokumentiert den tatsächlichen Stand der Trait-Architektur.

## Entscheidungen

1. **Synchroner In-Memory-Lesepfad in Ring 0:**
   In-Memory-Lese- und Suchoperationen in den Ring-0-Modulen (`csr.rs`, `inverted.rs`, `hnsw`) werden streng synchron ohne Async-Context-Switches ausgeführt [V]. Ring 0 enthält keine direkte `tokio`-Laufzeitabhängigkeit (Zero-`tokio`-Invariante in Ring 0) [V].

2. **Einheitliches `StorageEngine`-Trait für Persistenz:**
   Statt getrennter `StorageRead`/`StorageWrite`-Traits definiert das System ein zentrales Trait **`StorageEngine`** [V]. Operationen, die physische Festplatten-I/O, WAL-Appends oder SSTable-Flushes durchführen (`put`, `commit`, `rollback`, `scan_prefix_at`), nutzen `BoxFuture` für asynchrone Orchestrierung in den höheren Schichten (`contextra-engine`, Ring 3) [V].

3. **Begrenzter `ComputePool` für CPU-intensive Tasks:**
   Unbegrenzte `tokio::task::spawn_blocking`-Aufrufe bei parallelen Lese- und Indexierungs-Workloads werden durch einen kapazitätsbegrenzten, dedizierten `ComputePool` in `contextra-engine` ersetzt [V].

## Konsequenzen

* **Exakte Code-Übereinstimmung:** Die Dokumentation spiegelt das existierende `StorageEngine`-Trait wider [V].
* **Hot-Path Lese-Performanz:** In-Memory-Lookups im Ring 0 laufen frei von Async-Overhead [V].
* **Kritische Thread-Obergrenze:** Der begrenzte ComputePool verhindert Thread-Explosionen unter hoher paralleler Last [V].

## Reversibilitäts-Bedingung

Eine Aufspaltung des `StorageEngine`-Traits in getrennte Reader/Writer-Traits erfordert eine synchrone Umstellung aller Speicheradapter in Ring 1 und 3 [S].

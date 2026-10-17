# ADR-001: Speicherkern-Architektur — LSM-Tree für Persistenz

* **Datum:** 2026-05-10
* **Status:** Superseded / Refactored (ersetzt und erweitert durch ADR-N20)
* **Kontext / Auslöser:**
  Die Wahl des Speicherkerns für `contextra-store` bestimmt die Schreib- und Leseperformanz, die Crash-Konsistenz und die Fähigkeit zur rechtskonformen physischen Löschung (DSGVO / Multi-Tenant-Isolation).
  Die ursprüngliche 11-zeilige Begründung nannte als Argument ausschließlich den "Schreibdurchsatz" gegenüber B-Tree und SQLite. Dieses ADR wird hiermit durch eine systematische Sechs-Kriterien-Evaluierung (K1–K6) überarbeitet und durch den messbasierten Feature Freeze und Ausstiegspfad in ADR-N20 ergänzt.

## Entscheidung

Die initiale Persistenzarchitektur verwendet eine maßgeschneiderte LSM-Tree-Architektur (Log-Structured Merge-Tree in `contextra-store`) mit unumstößlichem Write-Ahead Log (`contextra-store::wal`) und immutablen SSTables [V].
Aufgrund von Performanz-Unterschieden bei zufälligen Lesezugriffen unterliegt dieser Speicherkern ab sofort einem Feature Freeze und wird gemäß ADR-N20 anhand harter Benchmark-Schwellenwerte gegen `redb` evaluiert.

## Evaluierung nach den sechs Norm-Kriterien (K1–K6)

1. **K1 — Schreibdurchsatz / Commit-Latenz:**
   * **LSM (Option A):** Sehr hoher sequenzieller Schreibdurchsatz durch append-only WAL und MemTable-Inserts [V].
   * **B-Tree / `redb` (Option B):** Sehr gute Commit-Latenz dank Copy-on-Write / ACID-Transaktionen, aber potenziell höherer Random-Write-Overhead [V].
2. **K2 — Random Read Latenz:**
   * **LSM (Option A):** Höhere Latenz bei Punkt-Suchläufen (Point Lookups) durch Suchen über mehrere SSTable-Level hinweg [V].
   * **B-Tree / `redb` (Option B):** Signifikant überlegene Point-Lookup-Latenzen durch direkte $O(\log N)$ B-Tree-In-Page-Lookups [V].
3. **K3 — Range Scan Durchsatz:**
   * **LSM (Option A):** Erfordert Zusammenführen (Merging) mehrerer iterativer SSTable-Reader; anfällig für Latenzspitzen [V].
   * **B-Tree / `redb` (Option B):** Sehr hoher, gleichmäßiger Range-Scan-Durchsatz [V].
4. **K4 — Speichereffizienz & Amplifikation:**
   * **LSM (Option A):** Hohe Write-Amplification (WAF) während Compactions; hervorragende Datenkompression [V].
   * **B-Tree / `redb` (Option B):** Niedrigere WAF im laufenden Betrieb, jedoch freie Seitenstrukturen [V].
5. **K5 — Crash-Konsistenz & Recovery:**
   * **LSM (Option A):** Exzellente Crash-Konsistenz durch sequenzielles WAL-Append mit HMAC-Frames und atomare Manifest-Rolls [V].
   * **B-Tree / `redb` (Option B):** ACID-Sicherheit durch zweiphasiges Root-Page-Fsync [V].
6. **K6 — Multi-Tenant-Löschfähigkeit & Erasure Proofs:**
   * **LSM (Option A):** Nativ unterstützt über `PurgeOrchestrator`, Erasure-Fences und physische SSTable/Blob-Purges [V].
   * **B-Tree / `redb` (Option B):** Erfordert eine dünne Aufsatzschicht für Multi-Tenant Scope-Fences und Löschzeugnisse [V].

## Alternativen-Bewertung

### Option A: Eigener LSM-Tree (`contextra-store`)
* **Pros:** Volle Kontrolle über WAL-HMAC-Header, Erasure-Fences und Zero-Copy-Read-Pfade.
* **Cons:** Hohe Komplexität; Latenznachteile bei Random Reads.
* **Bewertung:** Initial gewählt, nun unter Feature Freeze (siehe ADR-N20).

### Option B: `redb` Pure-Rust B-Tree Engine + Erasure-Schicht
* **Pros:** Pure Rust, kompromisslose ACID-Sicherheit, herausragende Read/Scan-Latenzen.
* **Cons:** Benötigt zusätzliche Abstraktion für Multi-Tenant-Scope-Löschungen.
* **Bewertung:** Als offizieller Ausstiegspfad in ADR-N20 definiert ($1,5\times$-Degradationsschwelle).

### Option C: Externe C++ Storage-Engines (`fjall` / RocksDB)
* **Pros:** Ausgereift und weit verbreitet.
* **Cons:** RocksDB verletzt Pure Rust Policy (ADR-004 / ADR-N03 / C-FFI); `fjall` fehlt noch nötige Erasure-Primitiven.
* **Bewertung:** Verworfen.

## Konsequenzen

* **Vollständige Transparenz:** Die ursprüngliche Begründung wird durch dieses Dokument und ADR-N20 auf eine saubere, messbare Architektur-Basis gestellt [S].

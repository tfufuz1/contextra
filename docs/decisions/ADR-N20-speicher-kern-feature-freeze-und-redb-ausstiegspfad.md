# ADR-N20: F5 Speicherkern — Feature Freeze, Messbasierte Evaluierung und redb-Ausstiegspfad

* **Status:** Accepted
* **Datum:** 2026-09-30
* **Kontext / Auslöser:**
  Der bisherige Beschluss in ADR-001 rechtfertigte den eigenen LSM-Tree (`contextra-store`) in lediglich 11 Zeilen und nannte als Vergleichsoptionen nur B-Tree und SQLite mit dem Argument "Schreibdurchsatz". Eigene empirische Benchmarks zeigten jedoch, dass rein zufällige Lesezugriffe (Random Read) und Range Scans unter dem maßgeschneiderten LSM-Tree Leistungseinbußen gegenüber etablierten Pure-Rust-Speicher-Engines aufweisen können.
  Zur Risikominimierung und zur Sicherstellung der Wartbarkeit erfordert das Projekt eine präzise Sechs-Kriterien-Evaluierung (K1–K6) sowie einen verbindlichen messbasierten Entscheidungs- und Ausstiegspfad (Gesamtspezifikation §F5).

## Entscheidung

1. **Feature Freeze für den eigenen LSM-Speicherkern:**
   Für die Eigenentwicklung des LSM-Trees in `contextra-store` gilt ein **sofortiger Feature Freeze**. Es werden ausschließlich Stabilitäts-, Korrektheits- und Bugfix-Änderungen vorgenommen; keine neuen architektonischen Speichereigenschaften werden ergänzt [V].

2. **Sechs-Kriterien-Evaluierung (K1–K6):**
   Der Speicherkern wird anhand von sechs normativen Kriterien evaluiert:
   * **K1 (Schreibdurchsatz / Commit-Durchsatz):** Latenz und Durchsatz sequenzieller WAL-Appends und MemTable-Inserts.
   * **K2 (Random Read Latenz):** Punktuelle Point-Lookups über SSTables und Blobs.
   * **K3 (Range Scan Durchsatz):** Iteration über geordnete Schlüsselbereiche.
   * **K4 (Speichereffizienz & Speicher-Amplifikation):** Write Amplification Factor (WAF) und Space Amplification.
   * **K5 (Crash-Konsistenz & Recovery):** Deterministische Wiederherstellbarkeit nach unsauberen Shutdowns (Fault-VFS).
   * **K6 (Multi-Tenant-Löschfähigkeit & Erasure Proofs):** Unterstützung für physische Bereichs-Löschungen, Erasure-Fences und Zeugnis-Ausstellung.

3. **Optionen-Evaluierung:**
   * **Option A (Eigener LSM-Tree in `contextra-store`):** Hohe Kontrolle über Erasure Proofs, WAL-Formate (V3/V4) und Zero-Copy-Read-Pfade; jedoch hoher eigener Wartungsaufwand [V].
   * **Option B (`redb` B-Tree Engine mit dedizierter Erasure-Schicht):** Pure Rust, extrem schnelle Random-Reads und ACID-Sicherheit, erfordert eine dünne Aufsatzschicht für Multi-Tenant-Erasure [V].
   * **Option C (`fjall` / RocksDB):** RocksDB verletzt das Pure-Rust-Prinzip (C++ FFI / Unsafe) [S]; `fjall` bietet noch nicht die benötigten In-Memory Erasure Proof Primitives [A].

4. **Messbasierter Ausstiegspfad & Schwelle ($1,5\times$):**
   Falls empirische Benchmarks (`contextra-bench`) zeigen, dass der eigene LSM-Tree gegenüber `redb` eine Leistungsdegradation von **$\ge 1,5\times$** bei Random Reads, Range Scans ODER der p99 Commit-Latenz aufweist, wird automatisch der Ausstiegspfad zu **Option B (`redb` + Erasure-Schicht)** vollzogen [V].

5. **Evaluierungs-Frist:**
   Die finale Messung und Entscheidung erfolgt spätenstens im **nächsten Minor-Release nach Ende des Freeze-Zeitraums**, sofern der Product Owner kein abweichendes Datum festlegt [S].

## Alternativen

### Option A: Unbegrenzte Weiterentwicklung des eigenen LSM-Trees
* **Vorteile:** Volle Flexibilität bei internen Datenstrukturen.
* **Nachteile:** Hohes Wartungsrisiko; Gefahr schleichender Latenz-Degradation.
* **Bewertung:** Verworfen zugunsten des messbasierten Freeze [S].

### Option C: RocksDB oder externe C++ Storage-Engines
* **Vorteile:** Extrem ausgereift und performant.
* **Nachteile:** Verstoß gegen die Sovereign Core Pure Rust Policy (ADR-004 / ADR-N03 / CONSTITUTION.md); C-FFI-Sicherheitsrisiken.
* **Bewertung:** Verworfen [S].

## Konsequenzen

* **Präzise Leistungssignale:** Entscheidungen über den Speicherkern basieren auf harten, reproduzierbaren Benchmark-Zahlen [V].
* **Kein Lock-in:** Der geplante `redb`-Ausstiegspfad sichert die Zukunftsfähigkeit der Architektur ab [V].
* **Ablösung von ADR-001:** Die unzureichende Begründung aus ADR-001 wird vollständig ersetzt [S].

## Reversibilitäts-Bedingung

Der eigene LSM-Tree verbleibt als Standard-Speicherkern, sofern er in allen drei Messkategorien unter der $1,5\times$-Degradationsschwelle im Vergleich zu `redb` bleibt [S].

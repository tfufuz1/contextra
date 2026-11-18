# Contextra

**Souveräne, offline betreibbare Such- und Wissens-Engine mit belegbarem Crypto-Shredding**

Contextra ist eine eingebettete Rust-Bibliothek zur transaktionalen, mehrdimensionalen Suche und strukturierten Wissensverwaltung für sensible Datenbestände in Unternehmen und regulierten Umgebungen.

---

## Kernarchitektur & Fähigkeiten

Contextra vereint drei Suchparadigmen, bitemporale Zeitreise und eine kryptographisch abgesicherte DSGVO-Löschkette in einem monolithischen, hierarchisch geschichteten Workspace:

```
┌──────────────────────── E3: Endpunkte ──────────────────────┐
│  mcp          testkit          bench                        │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────── E2: Integration ────────────────────┐
│  contextra ──► db       types        ports                  │
│  engine (deprecated = pub use db::*)                        │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────── E1: Domäne ─────────────────────────┐
│  store    vector    text    graph    rank                    │
│  mvcc     crypto    checkpoint                              │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────── E0: Fundament ──────────────────────┐
│  ids   env   sys   durable-fs   simd   privacy   erasure    │
└─────────────────────────────────────────────────────────────┘
```

### 1. Hybrid-Suche (Dense + Sparse + Graph)
* **Vektor-Suche (`contextra-vector`):** Hierarchical Navigable Small World (HNSW) Index mit Hardware-beschleunigten SIMD-Kernen (`AVX2`, `AVX-512`, `NEON`) und bitidentischem skalarem IEEE-754 Fallback (INV-5). Skalare Quantisierung (SQ8).
* **Volltext-Suche (`contextra-text`):** Invertierter Index mit BM25 / BM25F Relevanzbewertung und Zerlegung deutscher Komposita.
* **Graph-Navigation (`contextra-graph`):** Kompakter Wissensgraph (Compressed Sparse Row, CSR) für Traversierung und kontextuelle Relationen.
* **Fusion & Reranking (`contextra-rank`):** Reciprocal Rank Fusion (RRF) und Score-Normalisierung mit deterministischem Tie-Breaking nach DocId.

### 2. Speicherkern & Transaktionen
* **LSM-Tree Persistenz (`contextra-store`):** Crash-sicheres Write-Ahead Logging (WAL) mit Gruppen-Commits, Block-Cache und SSTables.
* **Snapshot-Isolation (`contextra-mvcc`):** Serializable Snapshot Isolation (SSI) mit Key-Level Mutexen und Pin-Verwaltung.
* **Crash-Resistenz (`contextra-durable-fs`):** Handle-Poisoning, atomare Dateioperationen und Scrubbing vor dem Freigeben von Blöcken.

### 3. DSGVO-Löschkette & Crypto-Shredding
Contextra unterscheidet vier Datenklassen physisch durch die kryptographische Schlüsselhierarchie (SPEC §3):
* **Klasse K (Unternehmenswissen):** Verschlüsselt mit `CollectionKEK`. Löschung erfolgt atomar durch Collection-Drop.
* **Klasse P (Personenbezogene Daten):** Adressiert über kryptographische `SubjectRef` (128-Bit-Zufalls-ID, `ZeroizeOnDrop`, kein `Clone`). Verschlüsselt mit individuellem `SubjectKey`. Die Löschung eines Betroffenen erfolgt atomar in $\mathcal{O}(1)$ via Crypto-Shredding des Schlüsselslots.
* **Klasse D (Abgeleitete Daten):** Tragen zwingend eine `LineageRef` zum Quelldatum für kaskadierte Bereinigung.
* **Klasse E (Löschmaterial):** Getrennter `ErasureKey` für manipulationssichere Audit-Journale und Hash-verkettete Ledger.

#### Das In-Memory ReadGate
Jeder einzelne Lesepfad (`get`, `scan`, Vektorsuche, Textsuche, Graph-Traversierung, Export) prüft synchron in $\mathcal{O}(1)$ vor jeglicher Entschlüsselung das Fail-Closed `ReadGate`. Für gesperrte Einträge existiert kein Unterschied zu nicht existierenden Daten.

### 4. Bitemporale Zeitreise
* **Transaktionszeit (`as_of`):** Reproduzierbare Abfrage des historischen Systemwissens zu einer Sequenznummer oder Unix-Timestamp.
* **Gültigkeitszeit (`valid_at`):** Filterung nach fachlicher Gültigkeit unabhängig vom Erfassungszeitpunkt.

---

## Schnittstellen & Betrieb

Contextra wird primär als eingebettete Rust-Bibliothek betrieben:

```rust
use contextra::Contextra;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Contextra::open(Path::new("./data")).await?;
    
    // Multi-Index Operationen
    Ok(())
}
```

Zusätzlich steht ein Model Context Protocol Server (`contextra-mcp`) für LLM-Tool-Integration via `stdio` bereit.

---

## Architekturgarantien & Invarianten

1. **`#![forbid(unsafe_code)]`:** Durchgesetzt im gesamten Workspace mit Ausnahme isolierter OS- und SIMD-Primitive (`contextra-sys`, `contextra-simd`).
2. **Zero-Panic-Policy (INV-4):** Kein `unwrap()`, `expect()` oder `panic!()` in Produktivcode.
3. **Schichtendisziplin (R1):** Abhängigkeitskanten verlaufen ausschließlich abwärts (E3 → E2 → E1 → E0).
4. **Keine stillen Fallbacks:** Fehlt ein Schlüssel oder scheitert AEAD, bricht die Operation mit Fehler ab.
5. **INV-7 Compliance-Klarstellung:** Contextra stellt technische Schutzmechanismen bereit (Crypto-Shredding, ReadGate, Ledger). Ob eine Gesamtanwendung datenschutzkonform betrieben wird, ist eine rechtliche Bewertung des jeweiligen Verantwortlichen und keine automatische Eigenschaft der Software.

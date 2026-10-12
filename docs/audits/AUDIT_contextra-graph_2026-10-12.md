# Audit-Bericht: contextra-graph Structure & Invariant Audit

- **Datum:** 2026-10-12
- **Crate:** `contextra-graph` (Ring 0)
- **Auditor:** Principal Senior Rust Architect (Jules Session `bce858f6`)
- **Spec / Referenzen:** K.11, ADR-016, ADR-026, ADR-033, AGT-GRAPH-001

---

## 1. Übersicht: Prüfpunkte P1–P7

| ID | Prüfpunkt | Status / Befund | Einstufung |
|---|---|---|---|
| **P1** | **AGT-GRAPH-001 Durchsetzung** | **Dokumentiert & Debug-Assert / Warnung** (NICHT durch Typ-System erzwungen). No wall-clock callers in `src/`. | 🟡 MEDIUM |
| **P2** | **Bi-Temporale Kanten (ADR-033)** | **Pass**. Traversierungslogik `as_of < valid_to` blendet Kanten bei `current_tx >= valid_to` exakt aus. | 🟢 PASSED |
| **P3** | **CSR-Präfix-Isolation** | **Verletzung in Default-Collection**. Unbound Scan in Default-Collection gibt `__graph:` Systemschlüssel zurück. | 🔴 HIGH |
| **P4** | **GraphEdge-Relation-Sync** | **Pass mit Sync-Fenster**. `relate()` schreibt LSM-KV vor `db_tx.commit()`, Commit führt `add_edge` synchron aus. | 🟡 LOW |
| **P5** | **Hop-Limit bei Traversierung** | **Pass**. `KPathConfig` erzwingt `max_hops` (4), `max_visited_nodes` (1000) und `max_steps`. | 🟢 PASSED |
| **P6** | **PPR L1-Norm-Abbruch (ADR-026)** | **Pass**. `max_iterations` begrenzt Power-Iteration strikt; Logging-Warning bei Nicht-Konvergenz, Best-Effort Rückgabe. | 🟢 PASSED |
| **P7** | **Community-Detection Skalierung** | **Pass**. Leiden-Pass durch `max_iterations` und `sub_pass < 10` geschützt; `O(|E|)`/Pass. keine Obergrenze für N. | 🟡 LOW |

---

## 2. Detaillierte Befunde & Nachweise

### P1: AGT-GRAPH-001 Technisch Durchgesetzt vs. Dokumentiert

**Frage:** Ist AGT-GRAPH-001 nur dokumentiert oder durch das Typ-System erzwungen?

**Antwort:** **Dokumentiert und via `debug_assert!` / `tracing::warn!` geprüft, aber NICHT durch das Typ-System erzwungen.**

- **Code-Scans in `crates/contextra-graph/src/`:**
  ```
  grep -rn "SystemTime::now()\|as_nanos()\|UNIX_EPOCH" crates/contextra-graph/src/ | grep -v "#\[cfg(test)\]"
  ```
  -> **0 Treffer** in Produktionscode. Alle Aufrufe beschränken sich auf Benchmark- und Test-Dateien.

- **Typ-System-Analyse:**
  In `crates/contextra-types/src/types/domain/ids.rs`:
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
  #[repr(transparent)]
  pub struct TxId(pub u64);

  impl TxId {
      pub const fn new(id: u64) -> Self { Self(id) }
      pub fn is_valid_origin(&self) -> bool {
          self.0 <= Self::MAX_COLLECTION_SEQUENCE || self.0 >= Self::INTERNAL_BASE
      }
  }
  ```
  `TxId::new(u64)` ist ein öffentlicher Konstruktor, der jeden beliebigen `u64`-Wert ohne Validierung akzeptiert.

- **Laufzeit-Durchsetzung in `CsrGraph` (`crates/contextra-graph/src/csr/graph_index.rs`):**
  ```rust
  debug_assert!(
      tx != TxId::INVALID && tx.is_valid_origin(),
      "TxId {} verletzt AGT-GRAPH-001 Origin-Invariante",
      tx
  );
  if is_suspicious_tx_id(tx) {
      tracing::warn!(tx_id = tx.inner(), "AGT-GRAPH-001: Verdächtiger oder unallozierter TxId...");
  }
  ```
  In Release-Builds wird ein ungültiger `TxId` nicht mit einem Fehler abgelehnt, sondern nur geloggt (`tracing::warn!`). Das schützt nicht vor kausalen Korruptionen durch Fehlbedienung in Release-Umgebungen.

---

### P2: Bi-Temporale Traversierungs-Logik-Nachweis (ADR-033)

**Implementierung in `crates/contextra-graph/src/csr/visibility.rs`:**

```rust
#[inline]
pub fn is_edge_visible(
    tx_valid_from: Option<TxId>,
    tx_valid_to: Option<TxId>,
    as_of: TxId,
) -> bool {
    tx_valid_from.is_none_or(|vf| vf <= as_of) && tx_valid_to.is_none_or(|vt| as_of < vt)
}
```

**Formale Logik-Verifikation:**

1. **Fall 1: `current_tx < valid_from` (Noch nicht begonnen)**
   - `vf <= as_of` ist `false`. `is_edge_visible` gibt `false` zurück. -> **Ausgeblendet** (Korrekt).

2. **Fall 2: `valid_from <= current_tx < valid_to` (Aktiv gültig)**
   - `vf <= as_of` ist `true` UND `as_of < vt` ist `true`. `is_edge_visible` gibt `true` zurück. -> **Sichtbar** (Korrekt).

3. **Grenzfall 3: `current_tx == valid_to` (Grenzzeitpunkt)**
   - `as_of < vt` mit `as_of == vt` ergibt `vt < vt` -> `false`. `is_edge_visible` gibt `false` zurück. -> **Ausgeblendet** (Korrekt gemäß ADR-033 Spec: `valid_to` ist ein exklusiver Endpunkt).

4. **Fall 4: `current_tx > valid_to` (Abgelaufen / Invalide)**
   - `as_of < vt` ist `false`. `is_edge_visible` gibt `false` zurück. -> **Ausgeblendet** (Korrekt).

---

### P3: Präfix-Isolations-Nachweis (CSR-Präfix-Isolation)

**Befund:** Datenleck von Systemschlüsseln bei ungefilterten Scans über die `default`-Collection.

**Code-Stelle in `crates/contextra-engine/src/collection/crud/read.rs` (`scan_prefix` / `scan`):**

```rust
for (k, v) in batch {
    let key_str = String::from_utf8_lossy(&k).to_string();
    let user_key = if self.name == "default" {
        key_str
    } else {
        let prefix_len = self.prefix.len() + 1;
        if key_str.len() >= prefix_len {
            key_str[prefix_len..].to_string()
        } else {
            key_str
        }
    };

    if let Ok(val) = serde_json::from_slice(&v) {
        results.push((user_key, val));
    }
}
```

**Analyse:**
- Für benannte Collections werden Schlüssel mit Collection-Präfix und `key_type` (z.B. `0` für User-Dokumente) geschützt.
- Für die `default`-Collection werden Schlüssel ohne Präfix im LSM-Key-Space abgelegt.
- `scan_prefix("", None)` oder `scan(Unbounded, Unbounded)` auf der `default`-Collection liest alle LSM-Einträge.
- Während `load_index()` in `crates/contextra-engine/src/collection/mod.rs:686` explizit `if self.name == "default" && k.starts_with(b"__") { continue; }` filtert, **fehlt** diese Filterung in `read.rs` (`scan` und `scan_prefix`).
- Graph-Objekte (`__graph:entity:...`, `__graph:edge:...`, `__graph:community:...`) sind valide JSON-Payloads und werden somit als reguläre User-Dokumente im Scan-Ergebnis zurückgegeben.

---

### P4: GraphEdge-Relation-Synchronisation

**Ablauf in `crates/contextra-engine/src/collection/relate.rs` (`relate_with_provenance`):**

1. `self.storage.put(db_tx.tx_id, &key, &bytes).await` persisitiert Relation als JSON-Dokument in der StorageEngine.
2. `db_tx.stage_graph_entity(...)` und `db_tx.stage_graph_edge(edge)` tragen Mutationen in den Transaktions-Puffer ein.
3. `db_tx.commit().await` ruft in `db_transaction_commit.rs` synchron auf:
   ```rust
   for edge in edges {
       GraphIndex::add_edge(&*self.collection.graph_index, self.tx_id, edge).await?;
   }
   ```

**Zeitfenster-Analyse:**
- Während der Transaktionsvorbereitung (`storage.put`) sieht kein anderer Leser die Kante, da die Sequenznummer uncommitted ist.
- Der In-Memory CSR Graph `graph_index` wird synchron während des `db_tx.commit()`-Passes aktualisiert.
- Nach Crash und Restart baut `load_index` / Graph-Recovery den In-Memory Graph-Index aus den LSM `__graph:` Persistenz-Einträgen wieder auf.

---

### P5: Hop-Limit bei Traversierung (PathRAG / Dijkstra)

**Implementierung in `crates/contextra-graph/src/path_rag/k_path.rs` (`KPathConfig`):**

```rust
pub struct KPathConfig {
    pub k: usize,
    pub max_visited_nodes: usize,
    pub max_hops: usize,
}

impl Default for KPathConfig {
    fn default() -> Self {
        Self {
            k: 3,
            max_visited_nodes: 1000,
            max_hops: 4,
        }
    }
}
```

- Traversierung prüft vor jedem Expansionsschritt `hops < config.max_hops` und bricht bei `visited_set.len() >= config.max_visited_nodes` ab (`budget_exhausted: true`).
- Zusätzlicher Schutzschalter: `max_steps = config.max_hops * 1000 + 100` verhindert Endlosschleifen bei dichten/zyklischen Graphen.

---

### P6: PPR L1-Norm-Abbruch (ADR-026)

**Implementierung in `crates/contextra-graph/src/ppr.rs`:**

- Die Power-Iteration führt Iterationen aus, bis die L1-Norm-Differenz unter `config.convergence_epsilon` fällt ODER `config.max_iterations` erreicht ist.
- Bei Erreichen von `max_iterations` vor Erreichen der Toleranz:
  ```rust
  if !converged && config.warn_on_non_convergence {
      tracing::warn!(
          max_iterations = config.max_iterations,
          convergence_epsilon = epsilon,
          "Personalized PageRank power iteration reached maximum iterations without reaching convergence; returning best-effort rank allocation"
      );
  }
  ```
- Garantiert terminierend, kein Panik-Verhalten, Rückgabe des besten Annäherungsergebnisses (Best-Effort).

---

### P7: Community-Detection Skalierung (Leiden)

**Implementierung in `crates/contextra-graph/src/community.rs` (`detect_communities`):**

- Iterationsobergrenze `config.max_iterations` (Default: 100) schützt den äußeren Konvergenzloop.
- Bipartite Sub-Community Refinement ist pro Phase auf `sub_pass < 10` Iterationen beschränkt.
- **Komplexität:** Pro Leiden-Pass `O(|E|)` auf allgemeinen Graphen, entartet zu `O(N^2)` auf vollständigen/extrem dichten Graphen.
- **Einschränkung:** Es existiert keine explizite Obergrenze für die Knotenzahl `N` vor dem Start der Community-Erkennung. Bei gigantischen In-Memory Graphen wird der Hauptspeicher skaliert.

---

## 3. VERDICT & EVIDENCE

<!-- EVIDENCE-MARKER-START -->
```
VERDICT: CONDITIONAL_PASSED
CRATE: contextra-graph
RING: 0
INVARIANTS_CHECKED:
  - AGT-GRAPH-001: PARTIAL (Enforced via debug_assert & warn; TxId type constructor non-validating)
  - INV-GRAPH-PROV-1: PASSED
  - Bi-Temporal Validity (ADR-033): PASSED
  - CSR Prefix Isolation: FAILED (System key leakage in default collection scan)
  - PPR L1 Convergence (ADR-026): PASSED
  - Hop Limit / PathRAG Budgeting: PASSED
FINDINGS:
  - FINDING-P3-HIGH: scan() and scan_prefix() in contextra-engine/src/collection/crud/read.rs do not filter out '__' or '__graph:' prefixes for the 'default' collection, leaking graph representation keys to public API scans.
  - FINDING-P1-MEDIUM: TxId::new(u64) constructor does not validate origin range; AGT-GRAPH-001 is enforced via debug_assert! in debug mode but only logs tracing::warn! in release mode.
RECOMMENDATIONS:
  - Fix FINDING-P3-HIGH by adding `if self.name == "default" && (user_key.starts_with("__") || user_key.starts_with("__graph:")) { continue; }` filtering in `read.rs`.
  - Consider strict runtime error rejection or fallible constructor for TxId origin validation in release mode.
```
<!-- EVIDENCE-MARKER-END -->

# MCP Tool Semantik & Anfragen-Dokumentation (`contextra-mcp`)

## Übersicht & Zweck

Dieses Dokument beschreibt die exakte Semantik, Eingabeschemata und tatsächlichen Wirkungen aller 15 in `crates/contextra-mcp/src/sandbox.rs` (`TOOL_REGISTRY`) registrierten Model Context Protocol (MCP) Tools. Es dient als technische Spezifikation und Analyse-Fundament vor einer künftigen Zusammenlegung/Umbenennung der Tools in eine konsolidierte Drei-Tool-Fassade.

Die Analyse basiert auf dem Stand des Quellcodes in:
- `crates/contextra-mcp/src/sandbox.rs` (Tool-Registrierung, Schemata, Kategorien)
- `crates/contextra-mcp/src/server_dispatch.rs` (JSON-RPC-Dispatching & `tools/list`)
- `crates/contextra-mcp/src/server_tools.rs` (Handler für CRUD, Relate, Consolidate, Cloud Query, Search)
- `crates/contextra-mcp/src/tools_crud.rs` (Handler für `create_collection`, `drop_collection`, `delete`)
- `crates/contextra-mcp/src/explain.rs` (Handler für `explain`)
- `crates/contextra-mcp/src/plugin_status.rs` (Handler für `plugin_status`)

---

## Einzelanalyse der 15 registrierten MCP-Tools

### 1. `contextra_search`

| Eigenschaft | Details |
|---|---|
| **Exakter Name** | `contextra_search` |
| **Kategorie** | `DatabaseRead` |
| **Eingabeschema (`input_schema`)** | ```json<br>{<br>  "type": "object",<br>  "properties": {<br>    "query": { "type": "string" },<br>    "collection": { "type": "string", "default": "default" },<br>    "k": { "type": "integer", "default": 10 }<br>  },<br>  "required": ["query"]<br>}<br>``` |
| **Tatsächliche Wirkung (Handler-Code)** | Generiert ein Vektorembedding für die Suchanfrage `query` via `self.embedder.embed(query)`. Führt eine hybride Suche (Vektor + BM25 + Graph) auf der Collection via `col.query().text(query).vector(&vec).k(k).execute()` aus. Das Feld `k` wird auf `MAX_SEARCH_K` (100) gedeckelt. Reichert jedes Suchergebnis mit `"content_provenance": "retrieved_untrusted_data"` an und prüft es auf Prompt-Injection. |
| **Diskrepanz zur Doku-Zeile in `sandbox.rs`** | Der Handler akzeptiert zusätzlich das Parameter-Alias `limit` als Alternative zu `k` (`args.get("k").or_else(|| args.get("limit"))`). Das Alias `limit` ist im JSON-Schema in `sandbox.rs` nicht deklariert. |
| **Entsteht ein `DeletionProof`?** | **Nein** |
| **Automatische Collection-Erstellung?** | **Ja** (Ruft `self.db.collection(col_name)` auf, was eine nicht existierende Collection automatisch initialisiert/anlegt). |

---

### 2. `contextra_insert`

| Eigenschaft | Details |
|---|---|
| **Exakter Name** | `contextra_insert` |
| **Kategorie** | `DatabaseWrite` |
| **Eingabeschema (`input_schema`)** | ```json<br>{<br>  "type": "object",<br>  "properties": {<br>    "id": { "type": "string" },<br>    "text": { "type": "string" },<br>    "collection": { "type": "string", "default": "default" },<br>    "metadata": { "type": "object" }<br>  },<br>  "required": ["id", "text"]<br>}<br>``` |
| **Tatsächliche Wirkung (Handler-Code)** | Teilt den übergebenen `text` mittels `MarkdownChunker` (~512 Tokens) in semantische Chunks auf. Für jeden Chunk wird ein Embedding berechnet und mit Metadaten (`source_id`, `chunk_index`, `chunk_total`) in der Collection via `col.insert()` gespeichert. Falls ein optionales `vector`-Array im Argument vorhanden ist, wird das Chunking übersprungen und der Vektor direkt unter der ID gespeichert. |
| **Diskrepanz zur Doku-Zeile in `sandbox.rs`** | 1. Das Schema in `sandbox.rs` fordert `["id", "text"]` als `required`, der Code erlaubt jedoch auch die Übergabe von `vector` anstelle oder zusätzlich zu `text`.<br>2. Der Handler teilt sich die Implementierung in `server_tools.rs` 1:1 mit `contextra_upsert` (`"contextra_insert" \| "contextra_upsert"`). Auf Storage-Ebene schreibt `col.insert` in die LSM-Engine; eine bereits existierende ID wird überschrieben (idempotenter Überschreibvorgang/Upsert). |
| **Entsteht ein `DeletionProof`?** | **Nein** |
| **Automatische Collection-Erstellung?** | **Ja** (`self.db.collection(col_name)` legt die Collection an, falls sie nicht existiert). |

---

### 3. `contextra_get`

| Eigenschaft | Details |
|---|---|
| **Exakter Name** | `contextra_get` |
| **Kategorie** | `DatabaseRead` |
| **Eingabeschema (`input_schema`)** | ```json<br>{<br>  "type": "object",<br>  "properties": {<br>    "id": { "type": "string" },<br>    "collection": { "type": "string", "default": "default" }<br>  },<br>  "required": ["id"]<br>}<br>``` |
| **Tatsächliche Wirkung (Handler-Code)** | Lädt ein Dokument direkt anhand seiner ID via `col.get(id)`. Falls gefunden, wird das JSON-Objekt mit `"content_provenance": "retrieved_untrusted_data"` versehen und durch den Prompt-Injection-Guard verifiziert. Falls nicht vorhanden, wird `null` zurückgegeben. |
| **Diskrepanz zur Doku-Zeile in `sandbox.rs`** | keine (Kurzbeschreibung und Implementierung stimmen überein). |
| **Entsteht ein `DeletionProof`?** | **Nein** |
| **Automatische Collection-Erstellung?** | **Ja** (`self.db.collection(col_name)`). |

---

### 4. `contextra_forget`

| Eigenschaft | Details |
|---|---|
| **Exakter Name** | `contextra_forget` |
| **Kategorie** | `DatabaseWrite` |
| **Eingabeschema (`input_schema`)** | ```json<br>{<br>  "type": "object",<br>  "properties": {<br>    "collection": { "type": "string" },<br>    "id": { "type": "string" },<br>    "confirm": { "type": "boolean" }<br>  },<br>  "required": ["collection", "confirm"]<br>}<br>``` |
| **Tatsächliche Wirkung (Handler-Code)** | Erfordert strikt `confirm: true`. Verzweigt nach Vorhandensein von `id`:<br>1. Wenn `id` angegeben: Löscht einzelnes Dokument via `col.delete(id)` (LSM Tombstone). Gibt `proof: null` und `proof_scope: "collection_only"` zurück.<br>2. Wenn `id` weggelassen: Löscht die gesamte Collection via `self.db.drop_collection(...)`. Erfordert konfigurierten `CONTEXTRA_DELETION_PROOF_KEY`. Stellt einen collection-weiten kryptografischen `DeletionProof` aus und gibt `proof_scope: "collection"` zurück. |
| **Diskrepanz zur Doku-Zeile in `sandbox.rs`** | Die Beschreibung erwähnt, dass ein DeletionProof bei Collection-Drop einen `CONTEXTRA_DELETION_PROOF_KEY` erfordert. Das Verhalten ist im Code exakt so umgesetzt, erfordert jedoch auch bei der Einzel-Dokument-Löschung über `forget` den Parameter `confirm: true` (im Gegensatz zu `contextra_delete`). |
| **Entsteht ein `DeletionProof`?** | **Bedingt:**<br>- **Nein** bei Einzel-Dokument-Löschung (`id` vorhanden -> `proof: null`).<br>- **Ja** bei Collection-Drop (`id` weggelassen -> kryptografischer `DeletionProof`). |
| **Automatische Collection-Erstellung?** | **Ja** (bei Einzel-Dokument-Löschung via `col.delete`). |

---

### 5. `contextra_collections`

| Eigenschaft | Details |
|---|---|
| **Exakter Name** | `contextra_collections` |
| **Kategorie** | `DatabaseRead` |
| **Eingabeschema (`input_schema`)** | ```json<br>{<br>  "type": "object",<br>  "properties": {}<br>}<br>``` |
| **Tatsächliche Wirkung (Handler-Code)** | Ruft `self.db.list_collections()` auf und gibt ein JSON-Objekt mit dem Array aller existierenden Collection-Namen zurück (`{ "collections": [...] }`). |
| **Diskrepanz zur Doku-Zeile in `sandbox.rs`** | keine. |
| **Entsteht ein `DeletionProof`?** | **Nein** |
| **Automatische Collection-Erstellung?** | **Nein** (reine Leseoperation). |

---

### 6. `contextra_consolidate`

| Eigenschaft | Details |
|---|---|
| **Exakter Name** | `contextra_consolidate` |
| **Kategorie** | `DatabaseWrite` |
| **Eingabeschema (`input_schema`)** | ```json<br>{<br>  "type": "object",<br>  "properties": {<br>    "collection": { "type": "string", "default": "default" }<br>  }<br>}<br>``` |
| **Tatsächliche Wirkung (Handler-Code)** | Triggert einen synchronen Gedächtnis-Konsolidierungslauf auf der angegebenen Collection. Liest alle Konversations-Turns aus dem Speicher, führt doppelte Einträge in Tombstones über, aktualisiert Graphkanten und führt optional eine Synthese durch. Gibt Kennzahlen über gescannte Turns, erzeugte Segmente, gesetzte Tombstones und Synthese-Chunks zurück. |
| **Diskrepanz zur Doku-Zeile in `sandbox.rs`** | keine (Verhalten entspricht exakt der Doku-Zeile und den Invarianten). |
| **Entsteht ein `DeletionProof`?** | **Nein** |
| **Automatische Collection-Erstellung?** | **Ja** (`self.db.collection(col_name)`). |

---

### 7. `contextra_cloud_query`

| Eigenschaft | Details |
|---|---|
| **Exakter Name** | `contextra_cloud_query` |
| **Kategorie** | `CloudEgress` |
| **Eingabeschema (`input_schema`)** | ```json<br>{<br>  "type": "object",<br>  "properties": {<br>    "query": { "type": "string" },<br>    "collection": { "type": "string", "default": "default" },<br>    "max_results": { "type": "integer", "default": 10 }<br>  },<br>  "required": ["query"]<br>}<br>``` |
| **Tatsächliche Wirkung (Handler-Code)** | Nimmt eine externe Cloud-Anfrage entgegen, kapselt diese in `TenantScoped` und leitet sie an das `egress_gateway` weiter. Dort werden Egress-Klassifizierung, Abstraktion und Bulk-Exfiltrationsprüfungen durchgeführt. |
| **Diskrepanz zur Doku-Zeile in `sandbox.rs`** | Der Handler akzeptiert zusätzlich das Parameter-Alias `k` als Alternative zu `max_results` (`args.get("max_results").or_else(|| args.get("k"))`). Im Schema ist nur `max_results` angegeben. |
| **Entsteht ein `DeletionProof`?** | **Nein** |
| **Automatische Collection-Erstellung?** | **Abhängig von Egress-Gateway/Cloud-Service** (keine direkte lokale DB-Collection-Erstellung im Handler). |

---

### 8. `contextra_relate`

| Eigenschaft | Details |
|---|---|
| **Exakter Name** | `contextra_relate` |
| **Kategorie** | `DatabaseWrite` |
| **Eingabeschema (`input_schema`)** | ```json<br>{<br>  "type": "object",<br>  "properties": {<br>    "from": { "type": "string" },<br>    "to": { "type": "string" },<br>    "label": { "type": "string" },<br>    "collection": { "type": "string", "default": "default" },<br>    "bidirectional": { "type": "boolean", "default": false }<br>  },<br>  "required": ["from", "to", "label"]<br>}<br>``` |
| **Tatsächliche Wirkung (Handler-Code)** | Prüft `label` auf Prompt-Injection via `injection_guard.detect(label)`. Erstellt eine gerichtete (`col.relate`) oder bidirektionale (`col.relate_bidirectional`) binäre Graph-Beziehung zwischen den Dokumenten `from` und `to`. |
| **Diskrepanz zur Doku-Zeile in `sandbox.rs`** | keine. |
| **Entsteht ein `DeletionProof`?** | **Nein** |
| **Automatische Collection-Erstellung?** | **Ja** (`self.db.collection(col_name)`). |

---

### 9. `contextra_relate_n_ary`

| Eigenschaft | Details |
|---|---|
| **Exakter Name** | `contextra_relate_n_ary` |
| **Kategorie** | `DatabaseWrite` |
| **Eingabeschema (`input_schema`)** | ```json<br>{<br>  "type": "object",<br>  "properties": {<br>    "predicate": { "type": "string" },<br>    "participants": {<br>      "type": "array",<br>      "items": {<br>        "type": "object",<br>        "properties": {<br>          "doc_id": { "type": "string" },<br>          "role": { "type": "string" }<br>        },<br>        "required": ["doc_id", "role"]<br>      },<br>      "minItems": 2,<br>      "maxItems": 64<br>    },<br>    "source_doc_id": { "type": "string" },<br>    "collection": { "type": "string", "default": "default" }<br>  },<br>  "required": ["predicate", "participants"]<br>}<br>``` |
| **Tatsächliche Wirkung (Handler-Code)** | Validiert `predicate` und alle Rollen auf Prompt-Injection. Prüft Teilnehmeranzahl (2 bis 64). Erstellt eine Hyperkanten-Graphbeziehung (`col.relate_n_ary`), die mehrere Teilnehmende mit ihren jeweiligen Rollen verbindet. Gibt die erzeugte `hyperedge_id` zurück. |
| **Diskrepanz zur Doku-Zeile in `sandbox.rs`** | keine. |
| **Entsteht ein `DeletionProof`?** | **Nein** |
| **Automatische Collection-Erstellung?** | **Ja** (`self.db.collection(col_name)`). |

---

### 10. `contextra_explain`

| Eigenschaft | Details |
|---|---|
| **Exakter Name** | `contextra_explain` |
| **Kategorie** | `DatabaseRead` |
| **Eingabeschema (`input_schema`)** | ```json<br>{<br>  "type": "object",<br>  "properties": {<br>    "id": { "type": "string" },<br>    "collection": { "type": "string", "default": "default" }<br>  },<br>  "required": ["id"]<br>}<br>``` |
| **Tatsächliche Wirkung (Handler-Code)** | Liest das Dokument aus der Collection (`col.get(id)`). Falls vorhanden, werden die Provenienzdaten aus den Metadaten extrahiert und eine menschenlesbare Erklärung erzeugt (`prov.explain_human_readable()`). Gibt eine `ExplainResponse` mit `explanation` und `"content_provenance": "retrieved_untrusted_data"` zurück. |
| **Diskrepanz zur Doku-Zeile in `sandbox.rs`** | In `server_dispatch.rs` besitzt `contextra_explain` eine Sonderbehandlung bei der Dispatch-Verzweigung (`handle_explain` anstelle von `call_tool`), um strukturierte `ExplainResponse`-Payloads zu liefern. Semantisch entspricht das der Beschreibung. |
| **Entsteht ein `DeletionProof`?** | **Nein** |
| **Automatische Collection-Erstellung?** | **Ja** (`self.db.collection(col_name)`). |

---

### 11. `contextra_plugin_status`

| Eigenschaft | Details |
|---|---|
| **Exakter Name** | `contextra_plugin_status` |
| **Kategorie** | `DatabaseRead` |
| **Eingabeschema (`input_schema`)** | ```json<br>{<br>  "type": "object",<br>  "properties": {}<br>}<br>``` |
| **Tatsächliche Wirkung (Handler-Code)** | Fragt die Plugin-Registry der Laufzeit ab (`registry.snapshot()`) und gibt Liste aller aktiven Plugins mit Name, Version, Ring-Level und benötigtem Feature-Ring sowie den aktuell aktiven Feature-Ring zurück. |
| **Diskrepanz zur Doku-Zeile in `sandbox.rs`** | keine. |
| **Entsteht ein `DeletionProof`?** | **Nein** |
| **Automatische Collection-Erstellung?** | **Nein** (keine DB-Operation). |

---

### 12. `contextra_upsert`

| Eigenschaft | Details |
|---|---|
| **Exakter Name** | `contextra_upsert` |
| **Kategorie** | `DatabaseWrite` |
| **Eingabeschema (`input_schema`)** | ```json<br>{<br>  "type": "object",<br>  "properties": {<br>    "id": { "type": "string" },<br>    "text": { "type": "string" },<br>    "vector": { "type": "array", "items": { "type": "number" } },<br>    "collection": { "type": "string", "default": "default" },<br>    "metadata": { "type": "object" }<br>  },<br>  "required": ["id"]<br>}<br>``` |
| **Tatsächliche Wirkung (Handler-Code)** | Speichert oder aktualisiert ein Dokument idempotent per Schlüssel. Verwendet denselben Handler-Code wie `contextra_insert` in `server_tools.rs`. Bei Übermittlung von `text` wird automatisches Markdown-Chunking durchgeführt; bei `vector` wird der Vektor direkt gespeichert. |
| **Diskrepanz zur Doku-Zeile in `sandbox.rs`** | 1. Im Schema in `sandbox.rs` ist nur `["id"]` als `required` definiert (im Gegensatz zu `contextra_insert`, das `["id", "text"]` fordert). Der Code verlangt jedoch bei beiden Tools, dass mindestens `text` oder `vector` vorhanden sein muss.<br>2. Der ausführende Code ist in `server_tools.rs` identisch mit `contextra_insert`. Es gibt auf Implementierungsebene keine funktionale Unterscheidung zwischen `insert` und `upsert`. |
| **Entsteht ein `DeletionProof`?** | **Nein** |
| **Automatische Collection-Erstellung?** | **Ja** (`self.db.collection(col_name)`). |

---

### 13. `contextra_delete`

| Eigenschaft | Details |
|---|---|
| **Exakter Name** | `contextra_delete` |
| **Kategorie** | `DatabaseWrite` |
| **Eingabeschema (`input_schema`)** | ```json<br>{<br>  "type": "object",<br>  "properties": {<br>    "id": { "type": "string" },<br>    "collection": { "type": "string", "default": "default" }<br>  },<br>  "required": ["id"]<br>}<br>``` |
| **Tatsächliche Wirkung (Handler-Code)** | Löscht ein einzelnes Dokument aus der Collection und den Vektor-/Graph-Indizes via `col.delete(id)` durch Schreiben eines LSM-Tombstones. Benötigt **kein** `confirm`-Flag. Gibt explizit `{ "ok": true, "collection": col_name, "id": id, "proof": null, "proof_scope": "collection_only" }` zurück. |
| **Diskrepanz zur Doku-Zeile in `sandbox.rs`** | keine. Die Doku-Zeile stellt korrekterweise klar, dass kein DeletionProof für Einzeldokumente ausgestellt wird. |
| **Entsteht ein `DeletionProof`?** | **Nein** (`proof: null`). |
| **Automatische Collection-Erstellung?** | **Ja** (`self.db.collection(col_name)`). |

---

### 14. `contextra_create_collection`

| Eigenschaft | Details |
|---|---|
| **Exakter Name** | `contextra_create_collection` |
| **Kategorie** | `DatabaseWrite` |
| **Eingabeschema (`input_schema`)** | ```json<br>{<br>  "type": "object",<br>  "properties": {<br>    "collection": { "type": "string" },<br>    "deployment_tier": {<br>      "type": "string",<br>      "enum": ["EdgeMinimal", "PowerUserLocal", "EnterpriseShared", "EnterpriseRegulated"],<br>      "default": "PowerUserLocal"<br>    }<br>  },<br>  "required": ["collection"]<br>}<br>``` |
| **Tatsächliche Wirkung (Handler-Code)** | Erstellt eine neue Collection mit dem angegebenen `deployment_tier` (Standard: `PowerUserLocal`). Der Handler unterstützt zusätzlich die Konfiguration von `auto_extraction` (`"enabled"` / `"disabled"`). Initialisiert das Collection-Profil und öffnet die Collection via Contextra DB. |
| **Diskrepanz zur Doku-Zeile in `sandbox.rs`** | 1. Schema-Abweichung: Das Schema in `sandbox.rs` enthält das optionale Feld `auto_extraction` nicht, wohingegen die `tools/list`-Antwort in `server_dispatch.rs` (Zeile 217ff.) `auto_extraction` im Schema deklariert.<br>2. Der Handler in `tools_crud.rs` akzeptiert zusätzlich `name` als Alias für `collection` (`args.get("collection").or_else(|| args.get("name"))`). |
| **Entsteht ein `DeletionProof`?** | **Nein** |
| **Automatische Collection-Erstellung?** | **Ja** (Ziel des Tools ist die explizite Collection-Erstellung). |

---

### 15. `contextra_drop_collection`

| Eigenschaft | Details |
|---|---|
| **Exakter Name** | `contextra_drop_collection` |
| **Kategorie** | `DatabaseWrite` |
| **Eingabeschema (`input_schema`)** | ```json<br>{<br>  "type": "object",<br>  "properties": {<br>    "collection": { "type": "string" },<br>    "confirm": { "type": "boolean" }<br>  },<br>  "required": ["collection", "confirm"]<br>}<br>``` |
| **Tatsächliche Wirkung (Handler-Code)** | Löscht eine gesamte Collection inklusive aller Indizes und Daten auf Disk via `self.db.drop_collection(...)`. Erfordert zwingend `confirm: true` und einen Umgebungsschlüssel `CONTEXTRA_DELETION_PROOF_KEY`. Generiert und gibt einen kryptografisch signierten, auditierbaren collection-weiten `DeletionProof` als JSON zurück. |
| **Diskrepanz zur Doku-Zeile in `sandbox.rs`** | 1. Der Handler in `tools_crud.rs` akzeptiert auch `name` als Alias für `collection`.<br>2. Die Erfordernis der Umgebungsvariable `CONTEXTRA_DELETION_PROOF_KEY` ist im Code hart erzwungen, steht aber nicht in der Kurzbeschreibung in `sandbox.rs` (wohl aber bei `contextra_forget`). |
| **Entsteht ein `DeletionProof`?** | **Ja** (kryptografischer collection-weiter Löschbeweis). |
| **Automatische Collection-Erstellung?** | **Nein** (Löschfunktion für existierende Collections). |

---

## Abgrenzung `insert` vs. `upsert`

In der MCP-Tool-Registrierung (`sandbox.rs`) sind `contextra_insert` und `contextra_upsert` als zwei getrennte Schreibwerkzeuge aufgeführt. Eine genaue Analyse der Handler-Implementierung in `crates/contextra-mcp/src/server_tools.rs` offenbart jedoch folgende technische Realität:

### 1. Handler-Identität im Quellcode
In `server_tools.rs` verzweigt der Match-Block für Tool-Aufrufe wie folgt:
```rust
"contextra_insert" | "contextra_upsert" => {
    // ... gemeinsamer Handler-Code für beide Tools ...
}
```
Beide MCP-Tools führen exakt denselben Quellcode aus. Es gibt im Rust-Backend keinerlei Fallunterscheidung oder abweichende Ausführungslogik zwischen einem `insert`- und einem `upsert`-Call.

### 2. Idempotenzgarantie auf Storage-Ebene
In der zugrundeliegenden Datenbank-Architektur (`contextra-db` / `contextra-store`) werden Dokument-Einträge über `col.insert(id, vector, metadata)` in ein Log-Structured Merge-Tree (LSM) geschrieben:
- Wenn ein Dokument mit einer bestimmten `id` **noch nicht existiert**, wird ein neuer Eintrag mit aktueller Sequenznummer angelegt.
- Wenn ein Dokument mit dieser `id` **bereits existiert**, wird das alte Dokument im LSM nicht fehlerhaft abgelehnt, sondern durch den neuen Eintrag mit einer höheren Sequenznummer überschrieben.

**Ergebnis:** Weder `insert` noch `upsert` besitzen eine reine "Insert-Only"-Garantie (die bei einer bereits existierenden ID mit einem `AlreadyExists`-Fehler abbrechen würde). Beide Tools verhalten sich auf Datenbank-Ebene als **idempotente Upserts** ($O(1)$ Schreibvorgang).

### 3. Differenzen im deklarierten Eingabeschema (`sandbox.rs`)
Obwohl der Handler identisch ist, unterscheiden sich die JSON-Schemata in `sandbox.rs`:
- **`contextra_insert`**:
  - `required`: `["id", "text"]`
  - Gedachtes Paradigma: Automatisches Text-Chunking via `MarkdownChunker` (~512 Tokens) und Auto-Embedding aus dem Klartext.
- **`contextra_upsert`**:
  - `required`: `["id"]`
  - Deklarierte Felder: `id`, `text`, `vector`, `collection`, `metadata`
  - Gedachtes Paradigma: Idempotentes Aktualisieren oder Einfügen, primär für vorgefertigte Vektoren oder Direkt-Einträge.

### 4. Zusammenfassung für die spätere Drei-Tool-Fassade
Die getrennte Existenz von `contextra_insert` und `contextra_upsert` ist historisch bedingt. In der künftigen Drei-Tool-Fassade reicht ein einziges Schreib-Tool (z. B. `write` / `store`), da die Speicher-Engine nativ idempotent arbeitet und sowohl Klartext-Chunking als auch vorgefertigte Vektoren im selben Handler verarbeitet.

---

## Abgrenzung `forget` vs. `delete` vs. `drop_collection`

Ein zentraler Aspekt von Contextra ist die datenschutzkonforme Löschung (GDBR / DS-GVO Compliance) und die Ausstellung kryptografischer Löschnachweise (`DeletionProof`).

### 1. Wirkungsbereich (Scope) & Parameter-Vergleich

| Tool | Primärer Zweck | Erforderliche Parameter | Wirkungsbereich |
|---|---|---|---|
| `contextra_delete` | Einzel-Dokument löschen | `id`, (`collection` optional) | **Einzelnes Dokument** |
| `contextra_drop_collection` | Gesamte Collection löschen | `collection`, `confirm: true` | **Gesamte Collection** |
| `contextra_forget` | Hybrid-Löschwerkzeug | `collection`, `confirm: true`, (`id` optional) | **Dokument ODER Collection** |

- **`contextra_delete`**: Löscht genau ein Dokument anhand der ID. Erfordert kein `confirm`-Flag.
- **`contextra_drop_collection`**: Löscht eine vollständige Collection inklusive aller verknüpften Indizes auf Disk. Erfordert zwingend `confirm: true`.
- **`contextra_forget`**: Funktioniert als Weiche. Wird ein `id`-Parameter übergeben, delegiert es intern an die Einzel-Dokument-Löschung (`col.delete(id)`). Wird kein `id`-Parameter übergeben, delegiert es an den Collection-Drop (`db.drop_collection(...)`). Erfordert in beiden Fällen `confirm: true`.

---

### 2. Kryptografischer Löschnachweis (`DeletionProof`) Matrix

| Tool | Szenario | `DeletionProof` ausgestellt? | Rückgabe-Feld `proof` | Rückgabe-Feld `proof_scope` |
|---|---|---|---|---|
| `contextra_delete` | Einzel-Dokument | **Nein** | `null` | `"collection_only"` |
| `contextra_forget` | Mit `id` (Einzel-Doc) | **Nein** | `null` | `"collection_only"` |
| `contextra_forget` | Ohne `id` (Collection) | **Ja** | `<DeletionProof JSON>` | `"collection"` |
| `contextra_drop_collection` | Collection-Drop | **Ja** | `<DeletionProof JSON>` | `"collection"` |

---

### 3. Technische Begründung aus dem Code (WARUM kein Proof für Einzel-Dokumente?)

Die Unterscheidung, welches Tool einen kryptografischen `DeletionProof` liefert und welches nicht, beruht auf der internen Speicher- und Löscharchitektur von Contextra (`crates/contextra-engine/src/contextra_impl/collections.rs` und `contextra-store`):

1. **Einzel-Dokument-Löschung (`col.delete(id)`)**:
   - Wenn ein einzelnes Dokument gelöscht wird, schreibt die LSM-Storage-Engine lediglich einen **Tombstone** (Löschmarker mit Sequenznummer) in das WAL (Write-Ahead Log) und die MemTable.
   - Der Tombstone bewirkt, dass das Dokument bei Leseanfragen (`get`, `search`) sofort unsichtbar ist ($O(1)$ logische Löschung).
   - Die ursprünglichen physischen Daten verbleiben jedoch in bestehenden unveränderlichen SSTable-Dateien auf der Festplatte, bis zu einem späteren Zeitpunkt eine SstCompaction (Merging pass) stattfindet.
   - **Begründung:** Da bei einer Einzel-Löschung keine sofortige physische Bereinigung/Nullung der Festplattensektoren garantiert werden kann, wäre die Ausstellung eines "kryptografischen Löschnachweises" technisch unwahr. Contextra erzwingt strikte ehrliche Nachweisbarkeit (*Cryptographic Honesty*): Einzel-Löschungen geben daher stets `proof: null` und `proof_scope: "collection_only"` zurück.

2. **Collection-Löschung (`db.drop_collection(...)`)**:
   - Beim Droppen einer gesamten Collection werden die physischen Ordnerstrukturen, SSTable-Dateien, WAL-Segmente und Vektor-/Graph-Indizes im Dateisystem physisch entfernt und gelöscht.
   - Das System liest die Merkle-Tree-Hashes der Collection, berechnet ein HMAC-SHA256 / Ed25519 Signatur-Token unter Verwendung des tenant-spezifischen `CONTEXTRA_DELETION_PROOF_KEY` und erzeugt ein unumstößliches Audit-Zertifikat (`DeletionProof v3`).
   - **Begründung:** Hier wurden die physischen Artefakte der Collection vollständig eliminiert. Der mathematische Löschnachweis kann den verifizierbaren Übergang der Collection in den gelöschten Zustand garantieren.

---

## Fazit & Empfehlung für die Drei-Tool-Fassade

Aus der vorgelegten Analyse ergeben sich klare Leitlinien für das spätere Refactoring-Arbeitspaket:
1. **Suche & Lesen:** `contextra_search`, `contextra_get`, `contextra_explain`, `contextra_collections` und `contextra_plugin_status` können in einer einheitlichen Lese-Fassade (`read` / `query`) gebündelt werden.
2. **Schreiben:** `contextra_insert` und `contextra_upsert` sind bereits auf Code-Ebene identisch und verschmelzen nahtlos in ein Schreib-Tool (`write` / `store`). `contextra_relate`, `contextra_relate_n_ary` und `contextra_consolidate` ordnen sich ebenfalls der Schreib-Kategorie zu.
3. **Löschen & Compliance:** Die Verzweigungslogik von `contextra_forget` zeigt, dass Einzel-Löschungen (`delete`) und Collection-Löschungen (`drop_collection`) aufgrund der kryptografischen DeletionProof-Sicherheitsgarantien klar getrennte Wirkungen haben. In einer künftigen Fassade muss die Aufteilung zwischen ehrlichem Tombstoning (Einzel-Doc) und auditiertem DeletionProof (Collection/Tenant) explizit erhalten bleiben.

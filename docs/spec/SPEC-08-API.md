# SPEC-08 — APIs, Protokolle und Fehlerkatalog

**Version 2.0 · November 2026 · Status: Final**
**Querverweise:** [Index](SPEC-00-INDEX.md) · [Datenmodell](SPEC-02-DATA-MODEL.md) · [Context Engine](SPEC-06-CONTEXT-ENGINE.md)

---

## 1. Rust-API (eingebettet, Referenz)

### 1.1 Repository öffnen und konfigurieren

```rust
let db = Contextra::open(
    Config::at("/var/lib/contextra/myrepo")
        .create_if_missing(true)
        .master_key(Some(key))                    // Verschlüsselung (optional)
        .resource_governor(governor.clone())       // Geteiltes Ressourcen-Budget
        .wal_group_commit_max_wait(Duration::from_millis(1))
        .wal_group_commit_max_batch(64)
        .page_cache_size(512 * 1024 * 1024)       // 512 MiB
        .memtable_budget(128 * 1024 * 1024)        // 128 MiB
        .background_threads(4)
)?;
```

### 1.2 Branch-Operationen

```rust
let main    = db.branch("main")?;
let attempt = main.fork("agent/a1/run-42/step-7")?;           // O(1), < 50 µs

let mut tx = attempt.begin(
    Tx::new()
        .intent("Fehlerursache aus Logs als Fakt ablegen")
        .idempotency_key("run-42/step-7/put-fact-1")
        .durability(Durability::Durable),
);

tx.put(Cell::fact("Registry-Token war abgelaufen")
        .namespace("agent/semantic")
        .trust(Trust::ToolVerified)
        .edge(Edge::derived_from(log_cell))
        .attr("source", "deployment-log-2026-10-14"))?;

let commit = tx.commit().await?;

// Recall
let ctx = attempt.recall(
    Recall::new("Registry Token")
        .budget(Tokens(3_000), TokenizerId::CL100K)
).await?;

// Merge
let merge = main.merge(&attempt, MergePolicy::from_namespace_defaults()).await?;
match merge {
    MergeOutcome::Merged(c)       => { /* fertig */ }
    MergeOutcome::FastForward(c)  => { /* fast-forward */ }
    MergeOutcome::Conflicts(list) => {
        for conflict in &list {
            attempt.resolve(conflict.id, Resolution::TakeOurs)?;
        }
        let c = attempt.commit_resolution().await?;
    }
}

// Aufräumen
attempt.discard()?;  // Branch und MemDelta freigeben
```

### 1.3 Zeitreise

```rust
// Zustand zu einem früheren Commit
let old = db.branch_at("main~3", "exploration/old-state")?;
let old_ctx = old.recall(Recall::new("Registry").budget(Tokens(1000))).await?;

// Diff
let changes = main.diff("main~5", "main")?;
for change in changes {
    println!("{}: {:?}", change.cell_id, change.kind);
}
```

### 1.4 Work Queue (externe Worker)

```rust
// Worker-Seite
let tasks = db.work().lease("worker-1", &[TaskKind::Embed, TaskKind::Summarize], 10, Duration::from_secs(300)).await?;

for task in tasks {
    match &task.task {
        WorkTask::Embed { cells, producer } => {
            let embeddings = compute_embeddings(cells);
            let changeset = ChangeSet::from_embeddings(cells, embeddings, producer);
            db.work().complete(task.task_id, changeset).await?;
        }
        WorkTask::Summarize { cluster, target_level, max_tokens } => {
            let summary = generate_summary(cluster, *max_tokens);
            let changeset = ChangeSet::from_summary(cluster, summary, *target_level);
            db.work().complete(task.task_id, changeset).await?;
        }
        _ => { db.work().reject(task.task_id, "unsupported task type").await?; }
    }
}
```

---

## 2. MCP-Server (Model Context Protocol)

### 2.1 Tool-Übersicht

Wenige, mächtige Tools (spart Tool-Beschreibungs-Token):

| Tool | Zweck | Typische Token für Schema |
|---|---|---|
| `mem_recall` | Recall unter Budget | ~120 |
| `mem_expand` | Handle → feinere/gröbere Stufe | ~40 |
| `mem_write` | Cells ablegen (idempotent) | ~100 |
| `mem_fork` | Neuen Branch erzeugen | ~30 |
| `mem_merge` | Branch in Ziel mergen | ~40 |
| `mem_discard` | Branch verwerfen | ~20 |
| `mem_history` | Zeitreise und Commit-Log | ~50 |
| `mem_diff` | Änderungsübersicht | ~40 |
| `mem_describe` | Schema/Namespaces/Policies | ~30 |

### 2.2 Tool-Schemas (JSON Schema)

#### `mem_recall`

```json
{
  "name": "mem_recall",
  "description": "Retrieves the most relevant context under a token budget. Returns handles that can be expanded.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "query": { "type": "string", "description": "Natural language search query" },
      "budget": { "type": "integer", "default": 4000, "description": "Max tokens in result" },
      "namespaces": {
        "type": "array", "items": { "type": "string" },
        "description": "Namespaces to search (empty = all)"
      },
      "min_trust": {
        "type": "string",
        "enum": ["system", "user", "tool-verified", "tool-unverified", "external"],
        "default": "external"
      },
      "time_range": {
        "type": "object",
        "properties": {
          "from": { "type": "string", "format": "date-time" },
          "to": { "type": "string", "format": "date-time" }
        }
      },
      "branch": { "type": "string", "default": "main" },
      "format": { "type": "string", "enum": ["compact", "xml", "json"], "default": "compact" }
    },
    "required": ["query"]
  }
}
```

#### `mem_expand`

```json
{
  "name": "mem_expand",
  "description": "Expands a handle to show more detail (finer zoom level).",
  "inputSchema": {
    "type": "object",
    "properties": {
      "handle": { "type": "string", "description": "Cell handle (e.g. 'c:k7qd')" },
      "level": { "type": "integer", "default": 0, "description": "Zoom level (0=full, higher=coarser)" },
      "branch": { "type": "string", "default": "main" }
    },
    "required": ["handle"]
  }
}
```

#### `mem_write`

```json
{
  "name": "mem_write",
  "description": "Writes one or more cells. Idempotent with idempotency_key.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "cells": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "content": { "type": "string" },
            "kind": {
              "type": "string",
              "enum": ["episode", "fact", "doc", "tool_result", "plan", "skill", "summary",
                       "profile", "code", "config", "annotation", "observation", "hypothesis"]
            },
            "namespace": { "type": "string", "default": "agent/episodic" },
            "trust": { "type": "string", "default": "tool-unverified" },
            "tags": { "type": "array", "items": { "type": "string" } },
            "edges": {
              "type": "array",
              "items": {
                "type": "object",
                "properties": {
                  "kind": { "type": "string" },
                  "target": { "type": "string", "description": "Handle or CellId" }
                }
              }
            }
          },
          "required": ["content"]
        }
      },
      "intent": { "type": "string", "description": "Why this write is happening" },
      "idempotency_key": { "type": "string", "description": "Prevents duplicate writes on retry" },
      "branch": { "type": "string", "default": "main" },
      "durability": { "type": "string", "enum": ["volatile", "buffered", "durable"], "default": "durable" }
    },
    "required": ["cells"]
  }
}
```

#### `mem_fork` / `mem_merge` / `mem_discard`

```json
{
  "name": "mem_fork",
  "inputSchema": {
    "type": "object",
    "properties": {
      "source": { "type": "string", "default": "main" },
      "name": { "type": "string", "description": "New branch name" },
      "ttl_hours": { "type": "integer", "description": "Auto-cleanup after hours" }
    },
    "required": ["name"]
  }
}
```

```json
{
  "name": "mem_merge",
  "inputSchema": {
    "type": "object",
    "properties": {
      "source": { "type": "string", "description": "Branch to merge from" },
      "target": { "type": "string", "default": "main" },
      "squash": { "type": "boolean", "default": false }
    },
    "required": ["source"]
  }
}
```

```json
{
  "name": "mem_discard",
  "inputSchema": {
    "type": "object",
    "properties": {
      "branch": { "type": "string", "description": "Branch to discard" }
    },
    "required": ["branch"]
  }
}
```

#### `mem_describe`

```json
{
  "name": "mem_describe",
  "description": "Describes the repository schema, namespaces, and capabilities within a token budget.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "budget": { "type": "integer", "default": 800 },
      "branch": { "type": "string", "default": "main" }
    }
  }
}
```

### 2.3 MCP-Server-Konfiguration

```toml
[mcp]
transport = "stdio"              # stdio | sse | streamable-http
max_concurrent_requests = 16
default_branch = "main"
default_budget = 4000
default_format = "compact"
```

---

## 3. gRPC-Service

```protobuf
syntax = "proto3";
package contextra.v1;

service Contextra {
    // Branch-Operationen
    rpc Fork(ForkRequest) returns (ForkResponse);
    rpc Merge(MergeRequest) returns (MergeResponse);
    rpc ListBranches(ListBranchesRequest) returns (ListBranchesResponse);

    // Lese-Operationen
    rpc Recall(RecallRequest) returns (RecallResponse);
    rpc Expand(ExpandRequest) returns (ExpandResponse);
    rpc Window(WindowRequest) returns (WindowResponse);
    rpc Explain(ExplainRequest) returns (ExplainResponse);
    rpc Neighbors(NeighborsRequest) returns (NeighborsResponse);
    rpc Diff(DiffRequest) returns (DiffResponse);

    // Schreib-Operationen
    rpc Commit(CommitRequest) returns (CommitResponse);
    rpc Resolve(ResolveRequest) returns (ResolveResponse);

    // Work Queue
    rpc LeaseTasks(LeaseRequest) returns (LeaseResponse);
    rpc CompleteTasks(CompleteRequest) returns (CompleteResponse);

    // Admin
    rpc Describe(DescribeRequest) returns (DescribeResponse);
    rpc Verify(VerifyRequest) returns (VerifyResponse);
    rpc Stats(StatsRequest) returns (StatsResponse);

    // Streaming
    rpc Subscribe(SubscribeRequest) returns (stream CommitEvent);
}
```

---

## 4. SQL-Zugang (SQLite Virtual Tables)

### 4.1 Schema-Mapping

```sql
-- Jeder Namespace wird als virtuelle Tabelle registriert
CREATE VIRTUAL TABLE episodic USING contextra(
    namespace='agent/episodic',
    branch='main'
);

-- Spalten (automatisch aus Namespace-Schema):
-- Feste Spalten:
--   _handle     TEXT     -- Handle (c:k7qd)
--   _cell_id    BLOB     -- CellId (16 Bytes)
--   _kind       TEXT     -- CellKind
--   _trust      TEXT     -- TrustLabel
--   _created    TEXT     -- ISO 8601
--   _valid_from TEXT     -- ISO 8601 (nullable)
--   _valid_to   TEXT     -- ISO 8601 (nullable)
--   _raw        TEXT     -- Volltext (L0)
--   _zoom_l1    TEXT     -- Summary (L1)
--   _zoom_l2    TEXT     -- Title (L2)
--   _tok_count  INTEGER  -- Token-Zählung (Standard-Tokenizer)
--   _author     TEXT     -- Autor-ID
--   _confidence REAL     -- 0.0–1.0 (nullable)

-- Dynamische Spalten aus Namespace-Policy Attributen:
--   tags        TEXT     -- JSON-Array
--   importance  REAL
--   pinned      INTEGER  -- 0/1
--   <custom>    <type>   -- Policy-definierte Attribute

-- Hidden Columns für Zeitreise:
--   _asof       TEXT HIDDEN  -- Commit-Ref oder Timestamp
--   _branch     TEXT HIDDEN  -- Branch (Default: 'main')
```

### 4.2 Spezielle Abfragen

```sql
-- Recall via SQL (Budget-aware)
SELECT * FROM episodic
WHERE contextra_recall('Deployment Fehler', 4000)
ORDER BY _relevance DESC;

-- Zeitreise
SELECT * FROM episodic WHERE _asof = 'main~3';

-- System-Tabellen
SELECT * FROM vcs_log LIMIT 20;
SELECT * FROM vcs_branches;
SELECT * FROM vcs_diff('main~3', 'main', 'episodic');
SELECT * FROM vcs_conflicts;

-- Commit via SQL
SELECT vcs_commit('Neue Fakten hinzugefügt', '{"run_id": "42"}');
```

### 4.3 Einschränkungen

- SQL `BEGIN…COMMIT` = eine Transaktion im Journal. Versions-Commit ist explizit via `vcs_commit()`.
- **Constraints erzwingt `contextra-table`**, nicht SQLite.
- Kein DDL (`CREATE TABLE`, `ALTER TABLE`) — Schema ist über Namespace-Policies gesteuert.
- Kein `JOIN` über verschiedene Branches (verschiedene Snapshots).

---

## 5. CLI (`ctx`)

```
ctx — Contextra Repository-Werkzeug

USAGE:
    ctx <COMMAND> [OPTIONS]

REPOSITORY:
    ctx init [PATH]              Neues Repository initialisieren
    ctx open [PATH]              Repository öffnen und Status anzeigen
    ctx info                     Repository-Metadaten

BRANCHES:
    ctx branch list              Alle Branches auflisten
    ctx branch create <NAME>     Neuen Branch erzeugen
    ctx branch delete <NAME>     Branch löschen
    ctx branch switch <NAME>     Aktiven Branch wechseln

VERSIONING:
    ctx log [--branch <B>]       Commit-Log
    ctx diff <A> <B>             Änderungen zwischen Commits/Branches
    ctx merge <SRC> [<DST>]      Merge
    ctx tag <NAME> [<COMMIT>]    Tag erzeugen

CONTENT:
    ctx put <FILE> [--ns <NS>]   Datei als Cell(s) importieren
    ctx get <HANDLE>             Cell-Inhalt ausgeben
    ctx recall <QUERY> [--budget N]  Recall ausführen
    ctx list [--ns <NS>]         Cells auflisten

MAINTENANCE:
    ctx verify                   Integritätsprüfung (Hashes, Indizes, GC-Erreichbarkeit)
    ctx gc                       Garbage Collection manuell auslösen
    ctx compact                  Compaction manuell auslösen
    ctx relayout                 Re-Layout manuell auslösen
    ctx stats                    Statistiken (Packs, Extents, Indizes, Cache)

IMPORT/EXPORT:
    ctx ingest <PATH>            Dokument(e) importieren
    ctx export <COMMIT> <PATH>   Zustand exportieren
    ctx dump <PACK>              Pack-Inhalt ausgeben (Debugging)
```

---

## 6. Fehlerkatalog (exhaustive)

Alle Fehler folgen dem Schema:

```json
{
  "error": "<error_code>",
  "message": "<menschenlesbare Beschreibung>",
  "retryable": true|false,
  "hint": "<optionaler Hilfetext>",
  "did_you_mean": ["<vorschlag1>", "..."],
  "suggested_call": { "tool": "<tool>", "args": { ... } }
}
```

### 6.1 Vollständiger Fehlerkatalog

| Code | Message-Template | Retryable | Kategorie |
|---|---|---|---|
| `unknown_handle` | Handle `{h}` existiert auf Branch `{b}` nicht. | nein | Datenmodell |
| `unknown_branch` | Branch `{b}` existiert nicht. | nein | Versionierung |
| `unknown_namespace` | Namespace `{ns}` existiert nicht. | nein | Datenmodell |
| `unknown_commit` | Commit `{id}` nicht gefunden. | nein | Versionierung |
| `branch_exists` | Branch `{b}` existiert bereits. | nein | Versionierung |
| `cell_not_found` | Cell `{id}` nicht in aktuellem Snapshot. | nein | Datenmodell |
| `budget_exceeded` | Angefordertes Budget ({n}) übersteigt Maximum ({max}). | nein | Context Engine |
| `budget_insufficient` | Mindestens {min} Token nötig, Budget ist {budget}. | nein | Context Engine |
| `conflict` | Merge-Konflikt in Cell `{id}`, Feld `{field}`. | nein | Versionierung |
| `rebase_failed` | Rebase fehlgeschlagen: überlappende Änderungen. | ja | Versionierung |
| `idempotent_replay` | Commit mit Key `{k}` bereits ausgeführt (CommitId: {id}). | nein (Erfolg) | Idempotenz |
| `permission_denied` | Operation `{op}` auf `{target}` nicht erlaubt. | nein | Sicherheit |
| `capability_expired` | Capability abgelaufen um {t}. | nein | Sicherheit |
| `capability_insufficient` | Capability erlaubt nicht: {missing_permission}. | nein | Sicherheit |
| `trust_violation` | Cell-Trust `{cell_trust}` unter Minimum `{min_trust}`. | nein | Sicherheit |
| `repo_locked` | Repository bereits von anderem Prozess geöffnet. | ja (nach Warten) | System |
| `repo_poisoned` | Repository nach fsync-Fehler vergiftet. Recovery nötig. | nein | System |
| `wal_corrupted` | WAL-Segment `{seg}` beschädigt ab Offset {off}. | nein | System |
| `pack_corrupted` | Pack `{pack}` CRC-Fehler in Extent {ext}. | nein | System |
| `invalid_input` | Eingabe ungültig: {details}. | nein | Validierung |
| `cell_too_large` | Cell-Größe ({size}) übersteigt Limit ({max}). | nein | Validierung |
| `namespace_read_only` | Namespace `{ns}` ist read-only. | nein | Lifecycle |
| `namespace_archived` | Namespace `{ns}` ist archiviert. | nein | Lifecycle |
| `zoom_level_missing` | Zoom-Level {l} für Cell `{id}` nicht verfügbar. Verfügbar: {available}. | nein | Context Engine |
| `work_task_expired` | Lease für Task `{id}` abgelaufen. | ja (re-lease) | Work Queue |
| `encryption_key_wrong` | Master Key passt nicht zu diesem Repository. | nein | Sicherheit |
| `format_version_unsupported` | Format-Version {v} nicht unterstützt (erwartet: {expected}). | nein | Kompatibilität |
| `internal_error` | Interner Fehler: {details}. | ja | System |
| `storage_full` | Speicher voll: {tier} hat {available} frei, braucht {needed}. | ja (nach Platz schaffen) | System |
| `timeout` | Operation nach {duration} abgebrochen. | ja | System |
| `rate_limited` | Token-Budget erschöpft: {used}/{limit} in {period}. | ja (nach Wartezeit) | Sicherheit |

---

## 7. Weitere Frontends

| Frontend | Crate | Beschreibung |
|---|---|---|
| C-ABI | `contextra-ffi` | C-kompatible Schnittstelle, `cbindgen`-generiert |
| Python | `contextra-py` | PyO3-Bindings, Conda/pip-Paket |
| Node.js | `contextra-node` (geplant) | N-API-Bindings |
| gRPC/HTTP | `contextra-router` | Protobuf + gRPC-Web |

### 7.1 Selbstbeschreibung (`mem_describe`)

Liefert Namespaces, Cell-Typen, Attribute, Merge-Regeln und Beispiele **in einem Token-Budget** (z. B. 800 Token) im Compact-Format:

```
Repository: myrepo (3 namespaces, 142k cells, 847 commits)
Branches: main (HEAD), agent/a1/run-42 (3 commits ahead)

Namespaces:
  agent/episodic  (84k cells, episodic, cl100k+o200k, emb-768-v3)
  agent/semantic  (52k cells, semantic, cl100k, emb-768-v3)
  docs/api        (6k cells, reference, cl100k, emb-768-v3)

Cell types: Episode(62%), Fact(21%), Doc(8%), ToolResult(5%), other(4%)
Trust: user(45%), tool-verified(32%), tool-unverified(18%), external(5%)

Write: mem_write({cells: [{content, kind, namespace, tags}], intent, idempotency_key})
Read:  mem_recall({query, budget, namespaces, min_trust})
```

---

*Ende SPEC-08. Weiter: [SPEC-09 Operations](SPEC-09-OPERATIONS.md)*

# SPEC-08 — APIs, Protokolle und Fehlerkatalog

**Version 2.1 · November 2026 · Status: Final**
**Querverweise:** [Index](SPEC-00-INDEX.md) · [Datenmodell](SPEC-02-DATA-MODEL.md) · [Context Engine](SPEC-06-CONTEXT-ENGINE.md)

---

## 1. Rust-API (eingebettete Referenz, `contextra`)

### 1.1 Repository öffnen & konfigurieren

```rust
let db = Contextra::open(
    Config::at("/var/lib/contextra/myrepo")
        .create_if_missing(true)
        .profile(ResourceProfile::Standard)     // lite | standard | scale
        .master_key(Some(key))                    // Optionale KEK/DEK-Verschlüsselung
        .memory_budget(MemoryBudget::gib(8))      // Partitioniert: fixed/memtable/cache/query/jobs
        .wal_group_commit(Duration::from_millis(1), 64)
        .background_threads(4)
)?;
```

### 1.2 Session-Branch, Typestate-Transaktion, Recall & Merge-Gates

```rust
let main = db.branch("main")?;
// Agenten schreiben nie direkt nach main: O(1)-Fork als Session-Branch
let session = main.fork("sess/planner/run-42")?;

let mut tx = session.begin(Isolation::Snapshot)?;
tx.put_content(CellContent::fact("Registry-Token war abgelaufen").source_span(log_ref, 120..210))?;
// Leichtgewichtiges Meta-Update ändert content_hash NICHT:
tx.tag(fact_id, ["incident", "auth"])?;
let commit_id = tx.commit(
    CommitOpts::new("Fehlerursache aus Logs extrahiert")
        .idempotency_key("run-42/step-7/put-fact-1")
        .durability(Durability::Durable),
)?;

// Hybride Suche unter Budget (MAC-Handle z. B. c:k7qd-3f)
let ctx = session.recall(
    Recall::new("Wie wurde der Deployment-Fehler gelöst?")
        .budget(Tokens(4_000), TokenizerId::O200K)
        .min_trust(TrustTier::ToolVerified)
        .placement(OrderPolicy::CacheFriendly)
        .now(clock.now_hlc())
).await?;

// Citation-Verifikation vor Agenten-Antwort
let verdict = session.verify_citations(&ctx, &draft_claims)?;

// Promotion nach main über Merge-Gates G1..G7
match main.propose_merge(&session, MergePolicy::from_namespace_defaults()).await? {
    MergeOutcome::Merged(c)             => { /* atomarer Commit auf main */ }
    MergeOutcome::FastForward(c)        => { /* fast-forward */ }
    MergeOutcome::GateRejected(report)  => { /* Gate G1..G7 verletzt: detaillierter Report */ }
    MergeOutcome::Conflicts(mr, list)   => {
        // Konflikte leben auf mr/*, niemals auf main!
        for c in &list { mr.resolve(c.id, Resolution::TakeTheirs)?; }
        mr.complete_merge().await?;
    }
}
```

### 1.3 Entkoppelte Mutationen, Retire & Purge

```rust
tx.close_validity(old_fact_id, new_valid_from)?; // Aktualisiert NUR CellMeta + Supersedes-Edge!
tx.retire(obsolete_id, "Doppelt erfasst")?;      // Versionierter Tombstone (kein Löschbeleg)
db.admin_purge(PurgeFilter::by_cell(pii_id))?;   // Best-Effort Tombstone + Retention-Override + Compaction
```

### 1.4 Work Queue (nicht-versionierter Sidecar)

```rust
let tasks = db.work().lease("worker-1", &[WorkKind::Embed, WorkKind::Summarize], 10, Duration::from_secs(300)).await?;
for entry in tasks {
    // Leasing erzeugt KEINEN Commit; erst complete() verbucht das Ergebnis als regulären Commit
    db.work().complete(entry.task_id, worker_result).await?;
}
```

---

## 2. MCP-Server (`contextra-mcp`)

### 2.1 Deklarative Tool-Klassifikation (Fail-Closed)

Jedes MCP-Tool ist statisch klassifiziert in **`Read` | `Write` | `Control`**. Unbekannte Methodennamen werden strikt abgelehnt (`method_not_found`, niemals Standard-Zugriff). Schreibwerkzeuge sind im Read-Only-Modus deaktiviert (`allow_write = false`) und schreiben standardmäßig in den aktiven Session-Branch.

| Tool | Klasse | Zweck | Schema-Budget |
|---|---|---|---|
| `mem_recall` | `Read` | Hybride Suche + Packer (MAC-Handles, `sufficiency`, Warnungen) | ~120 tok |
| `mem_expand` | `Read` | MAC-Handle (`c:k7qd-3f`) verifizieren & Zoom-Stufe nachladen | ~45 tok |
| `mem_verify` | `Read` | Claims deterministisch gegen zitierte Handles & Spannen prüfen | ~55 tok |
| `mem_neighbors` | `Read` | Begrenzte Graph-Nachbarschaft (`max_hops`, `truncated`) | ~45 tok |
| `mem_history` / `mem_diff` | `Read` | Bitemporale Historie (`AS OF` / `VALID AT`) & Diffs | ~65 tok |
| `mem_explain` | `Read` | Ranking-Signale & Filtergründe einer `recall_id` | ~35 tok |
| `mem_describe` | `Read` | Budgetierte Selbstbeschreibung (Namespaces, Klassen, Beispiele) | ~30 tok |
| `mem_write` | `Write` | Cells/Fakten/Kanten mit Pflichtprovenienz (idempotent) | ~110 tok |
| `mem_retire` | `Write` | Cell auf Session-Branch ausblenden (Begründung Pflicht) | ~35 tok |
| `mem_resolve` | `Write` | Konflikt auf `mr/*`-Branch auflösen | ~40 tok |
| `mem_session` | `Control` | `begin`, `savepoint`, `rollback`, `propose_merge`, `discard` | ~60 tok |

### 2.2 Beispiel-Schemas

```json
{
  "name": "mem_expand",
  "description": "Verifies a MAC-tagged handle (e.g. 'c:k7qd-3f') and loads the requested zoom level.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "handle": { "type": "string", "pattern": "^c:[0-9a-hjkmnp-tv-z]{4,}-[0-9a-hjkmnp-tv-z]{2}$" },
      "level": { "type": "integer", "minimum": 0, "maximum": 4, "default": 0 },
      "branch": { "type": "string", "default": "main" }
    },
    "required": ["handle"]
  }
}
```

**MCP-Konfiguration:** Transport `stdio` oder `streamable-http` (Nachrichtenlimit 4 MiB, `max_concurrent_requests = 16`).

---

## 3. gRPC, Arrow Flight & HTTP/JSON (`contextra-server`)

```protobuf
syntax = "proto3";
package contextra.v1;

service Contextra {
    rpc BeginSession(SessionRequest) returns (SessionResponse);
    rpc Savepoint(SavepointRequest) returns (SavepointResponse);
    rpc RollbackTo(RollbackRequest) returns (RollbackResponse);
    rpc ProposeMerge(MergeRequest) returns (MergeResponse);
    rpc ResolveConflict(ResolveRequest) returns (ResolveResponse);

    rpc Recall(RecallRequest) returns (RecallResponse);
    rpc Expand(ExpandRequest) returns (ExpandResponse);
    rpc VerifyCitations(VerifyRequest) returns (VerifyResponse);
    rpc Window(WindowRequest) returns (WindowResponse);
    rpc Neighbors(NeighborsRequest) returns (NeighborsResponse);
    rpc Explain(ExplainRequest) returns (ExplainResponse);
    rpc Diff(DiffRequest) returns (DiffResponse);

    rpc Commit(CommitRequest) returns (CommitResponse);
    rpc Purge(PurgeRequest) returns (PurgeResponse);

    rpc LeaseTasks(LeaseRequest) returns (LeaseResponse);
    rpc CompleteTasks(CompleteRequest) returns (CompleteResponse);

    rpc Describe(DescribeRequest) returns (DescribeResponse);
    rpc VerifyIntegrity(VerifyIntegrityRequest) returns (VerifyIntegrityResponse);
    rpc Subscribe(SubscribeRequest) returns (stream CommitEvent);
}
```

- **Arrow Flight:** Parallel zu gRPC für Bulk-Ingest, Dokument-Export und Embedding-Vektorbatches ohne JSON-/Protobuf-Kopie.
- **HTTP/JSON:** Identischer JSON-Query-Plan wie MCP + `/health` & Prometheus `/metrics`.

---

## 4. Optionales SQL-Frontend (`contextra-sqlite`, Feature `sqlite`)

Ein **optionales** SQLite-Virtual-Table-Frontend (`CREATE VIRTUAL TABLE semantic USING contextra(namespace='agent/semantic', branch='main')`) macht gespeicherte Namespaces über SQL lesbar (`_handle`, `_gid`, `_kind`, `_trust`, `_taint`, `_belief_lcb`, `_valid_from`, `_valid_to`, `_raw`, `_zoom_l1`, `_zoom_l2`, `_tok_count`, `_asof HIDDEN`, `_branch HIDDEN`), inkl. `contextra_recall(query, budget)` und `vcs_log`/`vcs_diff`/`vcs_conflicts`. Keine DDL (`CREATE`/`ALTER`); Joins über verschiedene Branches sind gesperrt.

---

## 5. CLI (`ctx`)

- **Repo & Integrität:** `ctx init`, `ctx info`, `ctx verify`, `ctx scrub`, `ctx stats`
- **Branches & Merge:** `ctx branch list|create|discard`, `ctx log`, `ctx diff <A> <B>`, `ctx merge <SRC> [<DST>]`, `ctx tag`
- **Daten & Suche:** `ctx ingest <PATH> --ns <NS>`, `ctx put`, `ctx get <HANDLE>`, `ctx recall <QUERY> --budget 4000`, `ctx explain <ID>`, `ctx purge`
- **Wartung:** `ctx compact`, `ctx relayout`, `ctx gc`, `ctx snapshot-export`

---

## 6. Vollständiger Fehlerkatalog (LLM-lesbar & Fail-Closed)

Alle Fehler folgen einheitlich diesem Schema. **Invariante (INV-5):** Bei fehlender Berechtigung verraten Fehlermeldungen weder Existenz noch Inhalt geschützter Namespaces/Cells.

```json
{
  "error": "unknown_handle",
  "message": "Handle 'c:k7qe-3f' ist ungültig oder im Snapshot nicht sichtbar.",
  "retryable": false,
  "hint": "MAC-Prüfung fehlgeschlagen. Prüfe einen der vorgeschlagenen Handles.",
  "did_you_mean": ["c:k7qd-3f", "c:k7qb-8m"],
  "suggested_call": { "tool": "mem_expand", "args": { "handle": "c:k7qd-3f" } }
}
```

| Code | Bedeutung | Retryable | Kategorie |
|---|---|---|---|
| `unknown_handle` / `invalid_handle_mac` | Handle ungültig oder MAC-Prüfung fehlgeschlagen (`did_you_mean` bis 3) | nein | Datenmodell |
| `unknown_branch` / `branch_exists` | Branch fehlt / existiert bereits | nein | Versionierung |
| `unknown_namespace` / `forbidden_namespace` | Neutraler Fehler ohne Existenz-Leak (INV-5) | nein | Sicherheit |
| `unknown_commit` / `cell_not_found` | Commit oder Cell im Snapshot nicht vorhanden/sichtbar | nein | Datenmodell |
| `budget_exceeded` / `budget_insufficient` | Budget > Max oder < Mindest-Token | nein | Context Engine |
| `conflict` | Merge-Konflikt auf `mr/*` (`ConflictCell`-ID angegeben) | nein | Versionierung |
| `merge_gate_rejected` | Gate `G1`–`G7` verletzt (Gate-ID + betroffene `CellId`s im Detail) | nach Fix | Versionierung |
| `rebase_failed` | OCC-/SSI-Konflikt bei gleichzeitigem Commit | ja | Transaktion |
| `idempotent_replay` | Key bereits verarbeitet → gibt gespeicherte `CommitId` zurück | (Erfolg) | Idempotenz |
| `idempotency_mismatch` | Gleicher `IdempotencyKey` mit abweichendem Payload-Digest gesendet | nein | Idempotenz |
| `snapshot_expired` / `snapshot_too_old` | Snapshot-Lease (300 s) abgelaufen oder hinter SSI-Prune-Horizont | ja | Transaktion |
| `beyond_retention` | Zeitreise außerhalb Retention (früheste `Seq` angegeben) | nein | Zeitreise |
| `permission_denied` / `capability_expired` | Fehlende Capability oder geschützter Branch (`main`) | nein | Sicherheit |
| `trust_violation` / `egress_blocked` | Trust unter Minimum oder Egress-Guard hat Cloud-Transfer blockiert | nein | Sicherheit |
| `vector_invalid` | NaN/∞/falsche Dimension im Embedding-Vektor | nein | Validierung |
| `backpressure` | Speicher-/Queue-Druckstufe `Critical` (`retry_after_ms` gesetzt) | ja | System |
| `repo_locked` / `repo_poisoned` | Prozess-Lock aktiv / Store nach `fsync`-Fehler vergiftet | je nach Fall | System |
| `wal_corrupted` / `pack_corrupted` | Prüfsumme/BLAKE3 defekt (Index heilt selbst; Truth braucht Restore) | nein | System |
| `work_task_expired` | Lease des Workers abgelaufen (Task zurück in `Pending`) | ja | Work Queue |
| `encryption_key_wrong` / `format_version_unsupported` | Master Key passt nicht / unbekanntes inkompatibles Format-Flag | nein | System |

---

*Ende SPEC-08. Weiter: [SPEC-09 Betrieb und Qualitätssicherung](SPEC-09-OPERATIONS.md)*

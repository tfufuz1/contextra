# SPEC-08 — APIs, Protokolle und Fehlerkatalog

**Version 3.0 · 26. November 2026 · Status: Final**
**Querverweise:** [Index](SPEC-00-INDEX.md) · [Versionierung](SPEC-04-VERSIONING.md) · [Context Engine](SPEC-06-CONTEXT-ENGINE.md) · [Sicherheit](SPEC-07-DURABILITY.md)

---

## 1. Gemeinsame Verträge (alle Oberflächen)

1. **Eine Semantik:** MCP, gRPC, HTTP, CLI und SQL bilden auf dieselben Engine-Operationen ab.
2. **Identität** stammt aus Transport-Authentisierung bzw. Capability, nie aus dem Anfragekörper.
3. **Idempotenz:** Jede Mutation akzeptiert einen Schlüssel; über Netzwerk ist er Pflicht.
4. **Deadlines** und Abbruch werden propagiert; nach Einreihung in das WAL ist ein Commit nicht mehr abbrechbar.
5. **Paginierung** über opake, an Snapshot und Principal gebundene Cursor.
6. **Limits** werden vor Allokation geprüft.
7. **Fehler** folgen §7 und verraten nichts Unberechtigtes (INV-5).
8. **Versionierung:** Protokollversion `v1`; Felder werden nur additiv ergänzt.

---

## 2. Rust-API (`contextra`)

```rust
let db = Contextra::open(
    Config::at("/var/lib/contextra/repo")
        .create_if_missing(true)
        .profile(ResourceProfile::Standard)
        .memory_budget(MemoryBudget::gib(48))
        .encryption(Some(cipher_provider)),
)?;

let main = db.branch("main")?;
let session = main.session("sess/planner/run-42", SessionConfig {
    ttl: Duration::from_secs(86_400), quotas: Quotas::default(), ..Default::default()
})?;

let sp = session.savepoint("before-tool-7")?;
let mut tx = session.begin(Isolation::Snapshot)?;
let fact = tx.put(
    NewCell::fact("Registry-Token war abgelaufen")
        .provenance(Provenance::derived_from(log_cell).span(120..210)),
)?;
tx.tag(fact, ["incident", "auth"])?;
let receipt = tx.commit(
    CommitOpts::new("Fehlerursache aus Logs extrahiert")
        .idempotency(IdemKey::new("run-42", "step-7", "put-fact-1"))
        .durability(Durability::Durable),
)?;

let pack = session.recall(
    Recall::new("Wie wurde der Deployment-Fehler gelöst?")
        .budget(Tokens(4_000), TokenizerAlias::O200K)
        .min_trust(TrustTier::AgentInferred)
        .now(clock.wall_ms()),
)?;
let verdicts = session.verify(&claims, Some(pack.recall_id))?;

match main.propose_merge(&session, MergePolicy::from_namespace_defaults())? {
    MergeOutcome::Merged(c) | MergeOutcome::FastForward(c) => {}
    MergeOutcome::GateRejected(report) => {}
    MergeOutcome::NeedsReview { mr, report } => {}
    MergeOutcome::Conflicts { mr, conflicts } => {
        for id in conflicts {
            db.merge_request(&mr)?.resolve(id, Resolution::TakeTheirs)?;
        }
        db.merge_request(&mr)?.complete()?;   // Gates laufen erneut
    }
}
```

- Die Kern-API ist **synchron** (L0–L4 sind runtime-frei). Das Feature `async` stellt `AsyncContextra` bereit, das auf eigene Pools auslagert.
- `Contextra: Send + Sync + Clone`; `Txn: Send + !Sync`.
- `Error` ist ein `#[non_exhaustive]`-Enum mit `code()`, `retryable()`, `to_wire()`.

### 2.1 Mutationen und Administration

```rust
tx.close_validity(old, valid_to)?;            // nur CellMeta
tx.link(Edge::supersedes(new, old))?;
tx.retire(id, "Doppelt erfasst")?;
db.admin(&cap).purge(PurgeFilter::cells([id]), PurgeOpts::dry_run())?;   // zuerst Vorschau

let tasks = db.work().lease("worker-1", &[WorkKind::Embed], 10, Duration::from_secs(300))?;
for t in tasks { db.work().complete(t.task_id, t.lease_token, result)?; }
```

---

## 3. MCP-Server (`contextra-mcp`)

Transport `stdio` oder `streamable-http`; Nachricht ≤ 4 MiB; ≤ 16 parallele Anfragen.

### 3.1 Klassifikation (fail-closed)

Jedes Tool ist statisch `Read | Write | Control`. Unbekannte Namen ⇒ `method_not_found`. Default `allow_write = false`: `Write`-Tools und mutierende `Control`-Aktionen werden weder gelistet noch ausgeführt.

| Tool | Klasse | Zweck | Schema-Budget [Ziel] |
|---|---|---|---|
| `ctx_recall` | Read | Suche + Packer | ~120 tok |
| `ctx_expand` | Read | Handle → Zoom-Stufe | ~50 tok |
| `ctx_window` | Read | Sequenzfenster | ~45 tok |
| `ctx_verify` | Read | Claims prüfen | ~55 tok |
| `ctx_neighbors` | Read | Graph-Nachbarschaft | ~45 tok |
| `ctx_history` | Read | Bitemporale Historie | ~40 tok |
| `ctx_diff` | Read | Änderungsübersicht | ~35 tok |
| `ctx_explain` | Read | Signale einer `recall_id` | ~35 tok |
| `ctx_describe` | Read | Namespaces, Schemata, Limits | ~30 tok |
| `ctx_remember` | Write | Cells/Fakten mit Provenienz | ~110 tok |
| `ctx_relate` | Write | Kante anlegen | ~45 tok |
| `ctx_retire` | Write | Ausblenden (Begründung Pflicht) | ~35 tok |
| `ctx_resolve` | Write | Konflikt auf `mr/*` lösen | ~40 tok |
| `ctx_session` | Control | `begin · savepoint · rollback · propose · discard · status` | ~60 tok |

### 3.2 Branch- und Sicherheitsregeln

- Ohne `branch` gilt der **aktive Session-Branch**, sonst `main` (nur lesend). Schreibzugriffe ohne Session ⇒ `session_required`; sie fallen nie auf `main` zurück.
- `ctx_session.propose` startet die Gates; ein Agent kann G7 nicht selbst erfüllen. `ctx_resolve` darf Trust nicht anheben.
- `purge`, Katalogänderungen, Capability-Ausstellung sind **nicht** über MCP verfügbar.
- Toolbeschreibungen sind statisch; gespeicherte Inhalte fließen nie in Tooldefinitionen. `ctx_describe` liefert Katalognamen als gekennzeichnete Daten.
- `ctx_remember` ignoriert clientseitig behaupteten `source_root`; Trust wird auf `MaxTrust` begrenzt und die Begrenzung gemeldet.

### 3.3 Schema-Beispiele

```json
{
  "name": "ctx_expand",
  "description": "Load a zoom level for a handle such as c:k7qd-3f.",
  "inputSchema": {
    "type": "object",
    "additionalProperties": false,
    "properties": {
      "handle": { "type": "string",
                  "pattern": "^c:[0-9a-hjkmnp-tv-z]{4,13}-[0-9a-hjkmnp-tv-z]{2}$" },
      "level":  { "type": "integer", "minimum": 0, "maximum": 3, "default": 0 },
      "max_tokens": { "type": "integer", "minimum": 64, "default": 2000 },
      "cursor": { "type": "string" },
      "branch": { "type": "string" }
    },
    "required": ["handle"]
  }
}
```

```json
{
  "name": "ctx_remember",
  "inputSchema": {
    "type": "object", "additionalProperties": false,
    "properties": {
      "items": { "type": "array", "minItems": 1, "maxItems": 64, "items": {
        "type": "object", "additionalProperties": false,
        "properties": {
          "kind": { "enum": ["episode","fact","observation","tool_result","plan","annotation","hypothesis"] },
          "text": { "type": "string" },
          "namespace": { "type": "string" },
          "sources": { "type": "array", "items": { "type": "object",
              "properties": { "handle": {"type":"string"}, "uri": {"type":"string"}, "quote": {"type":"string"} } } },
          "claim": { "type": "object" },
          "tags": { "type": "array", "items": {"type":"string"} }
        },
        "required": ["kind","text","namespace"]
      }},
      "idempotency_key": { "type": "string", "maxLength": 256 }
    },
    "required": ["items","idempotency_key"]
  }
}
```

Fehlt `sources` und ist `kind` nicht `hypothesis`, lehnt die Engine mit `provenance_missing` ab. `quote` wird gegen die Quelle geprüft und bestimmt `GL0`.

---

## 4. gRPC, Arrow Flight, HTTP (`contextra-server`)

```protobuf
syntax = "proto3";
package contextra.v1;

service Contextra {
  rpc BeginSession(SessionRequest)        returns (SessionResponse);
  rpc Savepoint(SavepointRequest)         returns (SavepointResponse);
  rpc RollbackTo(RollbackRequest)         returns (RollbackResponse);
  rpc DiscardSession(DiscardRequest)      returns (DiscardResponse);
  rpc Commit(CommitRequest)               returns (CommitResponse);
  rpc ProposeMerge(MergeRequest)          returns (MergeResponse);
  rpc ResolveConflict(ResolveRequest)     returns (ResolveResponse);
  rpc CompleteMerge(CompleteMergeRequest) returns (MergeResponse);
  rpc SubmitReview(ReviewRequest)         returns (ReviewResponse);

  rpc Recall(RecallRequest)               returns (RecallResponse);
  rpc Expand(ExpandRequest)               returns (ExpandResponse);
  rpc Window(WindowRequest)               returns (WindowResponse);
  rpc Neighbors(NeighborsRequest)         returns (NeighborsResponse);
  rpc VerifyCitations(VerifyRequest)      returns (VerifyResponse);
  rpc Explain(ExplainRequest)             returns (ExplainResponse);
  rpc History(HistoryRequest)             returns (HistoryResponse);
  rpc Diff(DiffRequest)                   returns (DiffResponse);
  rpc Describe(DescribeRequest)           returns (DescribeResponse);

  rpc LeaseTasks(LeaseRequest)            returns (LeaseResponse);
  rpc HeartbeatTasks(HeartbeatRequest)    returns (HeartbeatResponse);
  rpc CompleteTasks(CompleteRequest)      returns (CompleteResponse);

  rpc Purge(PurgeRequest)                 returns (PurgeResponse);
  rpc VerifyIntegrity(VerifyIntegrityRequest) returns (VerifyIntegrityResponse);
  rpc Subscribe(SubscribeRequest)         returns (stream CommitEvent);
}
```

- `SubmitReview`, `Purge`, `VerifyIntegrity` verlangen eigene Rechte (`Reviewer` via Merge+Admin-Caveat bzw. `Admin`).
- `Subscribe` liefert Ereignisse ab einer `seq`, at-least-once, nur sichtbare Änderungen; zu alte Position ⇒ `beyond_retention`. Langsame Konsumenten werden getrennt statt gepuffert.
- **Arrow Flight:** Bulk-Ingest, Export, Embedding-Batches; Ingest ist je Batch atomar mit Idempotenzschlüssel.
- **HTTP/JSON:** derselbe JSON-Plan wie MCP; `/health`, `/metrics` (ohne Inhalte, standardmäßig nur intern).

---

## 5. SQLite-Frontend (optional, Feature `sqlite`)

```sql
CREATE VIRTUAL TABLE semantic USING contextra(namespace='agent/semantic', branch='main');
SELECT _handle, _raw FROM semantic WHERE _asof = 'main~3' LIMIT 20;
```

Spalten: `_handle, _gid, _kind, _trust, _taint, _belief_lcb, _grounding, _valid_from, _valid_to, _raw, _zoom_l1, _zoom_l2, _tok_count, _asof HIDDEN, _branch HIDDEN`. Funktionen `contextra_recall(query, budget)`, `vcs_log`, `vcs_diff`, `vcs_blame`, `vcs_conflicts`. Nur lesend, keine DDL auf Contextra-Objekten, keine branchübergreifenden Joins. Jede Verbindung ist an eine Capability gebunden; ohne sie ist die Tabelle leer bzw. die Verbindung wird abgewiesen.

## 6. CLI (`ctx`) und Python

| Gruppe | Befehle |
|---|---|
| Store | `init · info · stats · verify · scrub · upgrade · backup · restore · snapshot-export` |
| Versionierung | `branch list/create/discard · log · diff · merge · resolve · tag` |
| Daten | `ingest <PATH> --ns · put · get <HANDLE> · recall <Q> --budget · explain · retire · purge` |
| Wartung | `compact · relayout · gc · index rebuild · key rotate` |

Destruktive Befehle unterstützen `--dry-run` und verlangen Bestätigung oder `--yes`. `--json` liefert stabile Ausgaben. `contextra-py` (PyO3, außerhalb `default-members`) bildet die Rust-API ab und gibt den GIL während Engine-Aufrufen frei.

---

## 7. Fehlerkatalog

```json
{
  "error": "unknown_handle",
  "code": "E_HANDLE_UNKNOWN",
  "message": "Handle 'c:k7qe-3f' ist ungültig oder hier nicht sichtbar.",
  "retryable": false,
  "hint": "Prüfe einen vorgeschlagenen Handle oder suche erneut.",
  "did_you_mean": ["c:k7qd-3f"],
  "suggested_call": { "tool": "ctx_expand", "args": { "handle": "c:k7qd-3f" } }
}
```

`error` und `code` sind stabil; Texte dürfen sich ändern. `message` zitiert nur Eingaben des Aufrufers.

| `error` | `code` | Retry | gRPC | HTTP | Bedeutung |
|---|---|---|---|---|---|
| `unknown_handle` | `E_HANDLE_UNKNOWN` | nein | NOT_FOUND | 404 | unbekannt oder nicht sichtbar |
| `invalid_handle_mac` | `E_HANDLE_MAC_INVALID` | nein | INVALID_ARGUMENT | 400 | MAC falsch (vor jedem Lookup) |
| `forbidden_namespace` | `E_FORBIDDEN_NAMESPACE` | nein | PERMISSION_DENIED | 403 | unbekannt **oder** verboten |
| `cell_not_found` | `E_CELL_NOT_FOUND` | nein | NOT_FOUND | 404 | nicht vorhanden **oder** unsichtbar |
| `unknown_branch` | `E_BRANCH_UNKNOWN` | nein | NOT_FOUND | 404 | auch bei fehlender Berechtigung |
| `branch_exists` | `E_BRANCH_EXISTS` | nein | ALREADY_EXISTS | 409 | |
| `unknown_commit` | `E_COMMIT_UNKNOWN` | nein | NOT_FOUND | 404 | |
| `permission_denied` | `E_PERMISSION_DENIED` | nein | PERMISSION_DENIED | 403 | Capability reicht nicht |
| `capability_expired` | `E_CAPABILITY_EXPIRED` | nein | UNAUTHENTICATED | 401 | |
| `session_required` | `E_SESSION_REQUIRED` | nein | FAILED_PRECONDITION | 412 | |
| `quota_exceeded` | `E_QUOTA_EXCEEDED` | nein | RESOURCE_EXHAUSTED | 429 | |
| `budget_exceeded` | `E_BUDGET_EXCEEDED` | nein | INVALID_ARGUMENT | 400 | über Capability-Maximum |
| `budget_insufficient` | `E_BUDGET_TOO_SMALL` | nein | INVALID_ARGUMENT | 400 | Mindestwert angegeben |
| `tokenizer_unknown` | `E_TOKENIZER_UNKNOWN` | nein | INVALID_ARGUMENT | 400 | |
| `limit_exceeded` | `E_LIMIT_EXCEEDED` | nein | INVALID_ARGUMENT | 400 | |
| `vector_invalid` | `E_VECTOR_INVALID` | nein | INVALID_ARGUMENT | 400 | |
| `schema_violation` | `E_SCHEMA_VIOLATION` | nein | INVALID_ARGUMENT | 400 | |
| `provenance_missing` | `E_PROV_MISSING` | nein | INVALID_ARGUMENT | 400 | |
| `trust_violation` | `E_TRUST_INSUFFICIENT` | nein | FAILED_PRECONDITION | 412 | |
| `egress_blocked` | `E_EGRESS_BLOCKED` | nein | PERMISSION_DENIED | 403 | |
| `conflict` | `E_CONFLICT` | nein | ABORTED | 409 | inhaltlicher Konflikt |
| `rebase_failed` | `E_REBASE_FAILED` | ja | ABORTED | 409 | neu beginnen |
| `merge_gate_rejected` | `E_GATE_REJECTED` | nach Fix | FAILED_PRECONDITION | 422 | Gate, betroffene sichtbare Cells |
| `review_required` | `E_REVIEW_REQUIRED` | nein | FAILED_PRECONDITION | 409 | MR offen |
| `idempotency_mismatch` | `E_IDEMPOTENT_MISMATCH` | nein | ALREADY_EXISTS | 409 | |
| `snapshot_expired` | `E_SNAPSHOT_EXPIRED` | ja | ABORTED | 409 | |
| `snapshot_too_old` | `E_SNAPSHOT_TOO_OLD` | ja | ABORTED | 409 | SSI-Horizont |
| `beyond_retention` | `E_BEYOND_RETENTION` | nein | OUT_OF_RANGE | 410 | früheste Position |
| `expunged` | `E_EXPUNGED` | nein | NOT_FOUND | 410 | nur für zuvor Berechtigte |
| `index_incomplete` | `E_INDEX_INCOMPLETE` | ja | UNAVAILABLE | 503 | bei `require_complete` |
| `deadline_exceeded` | `E_DEADLINE` | ja | DEADLINE_EXCEEDED | 504 | Commit-Ausgang ggf. per Key klären |
| `backpressure` | `E_BACKPRESSURE` | ja | RESOURCE_EXHAUSTED | 429 | `retry_after_ms` |
| `work_task_expired` | `E_TASK_EXPIRED` | ja | ABORTED | 409 | |
| `repo_locked` | `E_STORE_LOCKED` | ja | UNAVAILABLE | 503 | |
| `repo_poisoned` | `E_STORE_FAILED` | nein | UNAVAILABLE | 503 | Neustart und Recovery |
| `wal_corrupted` | `E_WAL_CORRUPT` | nein | DATA_LOSS | 500 | Operator |
| `pack_corrupted` | `E_CORRUPT_TRUTH` | nein | DATA_LOSS | 500 | Restore |
| `encryption_key_wrong` | `E_KEY_INVALID` | nein | FAILED_PRECONDITION | 412 | |
| `format_version_unsupported` | `E_FORMAT_UNSUPPORTED` | nein | FAILED_PRECONDITION | 412 | |
| `unsupported_durability` | `E_DURABILITY_UNSUPPORTED` | nein | UNIMPLEMENTED | 501 | `replicated`, unzulässig auf Branch |

**Kein Fehler:** Idempotenter Replay ist eine Erfolgsantwort mit `replayed: true` (informativer Code `I_IDEMPOTENT_REPLAY`). **Orakelschutz:** Unbekannt und verboten sind bei Namespaces, Branches, Cells und Handles ununterscheidbar; `invalid_handle_mac` hängt nur von der Eingabe ab.

---

## 8. Abnahme

- Konformitätstest: dieselbe Szenariensuite über Rust, MCP, gRPC und HTTP mit identischen Fehlercodes.
- `xtask`: jedes Tool klassifiziert, jeder `Error`-Wert auf genau eine Katalogzeile abgebildet.
- Golden-Agententranskripte: Konflikt, Retry, Zeitreise, halluzinierte Handles.
- `buf breaking` für Protobuf.

---

*Weiter: [SPEC-09 Betrieb und Qualitätssicherung](SPEC-09-OPERATIONS.md)*

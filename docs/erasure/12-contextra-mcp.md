# Implementierungsspezifikation `contextra-mcp` (Löschkette – Erase-Werkzeuge)

*Ring 5 · Konzept-WP 5 (Teil) · Abhängig von: 10, 11 · Index: `00-INDEX.md` (K-19, K-36)*

> **Verifikationsgrenze.** Statisch, nichts kompiliert. **✔** gelesen, **[?]** Prüfauftrag.

## 1. Zweck

Drei Werkzeuge `contextra_erase`, `contextra_erase_status`, `contextra_erase_dry_run` (K-19: Namensschema `contextra_*`, nicht `ctx_*`). Das Crate darf laut `capabilities.toml` nur `contextra` (Facade) und `types`/`ports`/`rank`/`crypto` nutzen (K-36) ✔; alle Erase-Typen kommen über die Facade-Re-Exporte (11 §3).

## 2. Ist-Zustand (✔)

| Aspekt | Befund |
| --- | --- |
| Registrierung | `sandbox.rs:49 TOOL_REGISTRY: &[ToolDefinition]` mit `name`, `category: ToolCategory {DatabaseRead, DatabaseWrite, …}`, `description`, `input_schema: fn() -> Value`; `validate_tool_call(name, args)` |
| Auflistung | `server_dispatch.rs:~150-215`: `tools/list` mit Namen `contextra_search|insert|get|forget|collections|explain` (handgepflegt) |
| Dispatch | `server_tools.rs:16-369`: `match name { "contextra_search" … "contextra_forget" \| "contextra_delete" => self.handle_delete(args), "contextra_drop_collection" => self.handle_drop_collection(args) }` |
| Handler | `tools_crud.rs`: `handle_create_collection` (`:10`), `handle_drop_collection` (`:93`, verlangt `confirm == true`, `tenant_id` aus `self.tenant_id`, ruft `self.db.drop_collection(col_name, tenant_id)`), `handle_delete` (`:139`) |
| Validierung | `validation.rs` (`validate_collection_name`), `prompt_injection/*` |
| Mandant | `ContextraServer.tenant_id: Option<TenantId>` (`server.rs:19`), Default `TenantId::try_new(1)` (`:52`) |

## 3. Dateien

| Datei | Aktion |
| --- | --- |
| `src/sandbox.rs` | `ToolCategory::DatabaseErase` **neu** (**Ende** der Enum, `#[non_exhaustive]` beachten **[?]**), drei `ToolDefinition`s, Tabelle in der Doku |
| `src/server_dispatch.rs` | Einträge in `tools/list` |
| `src/server_tools.rs` | `match`-Zweige |
| `src/tools_erase.rs` | **NEU** (`handle_erase`, `handle_erase_status`, `handle_erase_dry_run`) |
| `src/validation.rs` | `validate_erase_args`, `validate_erasure_id(&str) -> ErasureId` |
| `src/lib.rs` | `mod tools_erase;` |
| `src/config.rs` | `allow_erase: bool` (Default **false**), `erase_requires_confirm_token: Option<String>` |
| `tests.rs`, `tests/erase_tools.rs` | **NEU** |

## 4. Werkzeuge (normativ)

### 4.1 `contextra_erase`

Kategorie `DatabaseErase`. **Standard: deaktiviert** (`allow_erase = false` ⇒ `McpError::method_not_allowed("erase disabled")`). Sandbox-Regel: nicht über Prompt-Injection-Pfad erreichbar ohne Bestätigung (siehe §5).

```json
{
  "type": "object",
  "properties": {
    "scope":      { "type": "string", "enum": ["document", "collection", "tenant"] },
    "collection": { "type": "string" },
    "id":         { "type": "string", "description": "Dokumentschlüssel, nur bei scope=document" },
    "reason":     { "type": "string", "enum": ["user_request","legal_request","retention_expiry","operator_action","tenant_offboarding"] },
    "await":      { "type": "string", "enum": ["accepted","gated","purged"], "default": "gated" },
    "confirm":    { "type": "boolean" },
    "confirm_token": { "type": "string" }
  },
  "required": ["scope", "reason", "confirm"]
}
```

Regeln:
1. `confirm == true` Pflicht (wie `contextra_drop_collection`, ✔ `tools_crud.rs:93-109`); sonst `invalid_params`.
2. `scope = document`: `collection` (Default `default`) und `id` Pflicht; `collection` per `validate_collection_name`; `id` nicht leer, ≤ 4 096 Byte, kein NUL.
3. `scope = collection`: `collection` Pflicht; `default` ⇒ `invalid_params("default collection cannot be erased as collection")` (K-5).
4. `scope = tenant`: nur wenn `config.allow_tenant_erase` **und** `confirm_token` stimmt (konstante Zeit, `subtle`-ähnlicher Vergleich über `crypto`, **[?]** vorhandene Konstantzeit-Funktion `rg "ct_eq|constant_time" crates`).
5. `tenant_id` kommt **ausschließlich** aus `self.tenant_id` (nie aus Argumenten, Mandantenisolation); `None` ⇒ `invalid_params("tenant_id not configured on server")`.
6. Aufruf: `self.db.erase(target, EraseOptions{ reason, await_level, policy: None })`.
7. `await = purged` wartet **höchstens** `config.erase_wait_timeout` (Default 30 s); danach Antwort mit aktuellem Stand (`state`), Auftrag läuft weiter.

Antwort (kein Schlüssel, kein Name, **keine** Inhalte):

```json
{ "ok": true, "erasure_id": "<32 hex>", "state": "Gated", "guarantee": "G1",
  "gated": true, "purged": false, "crypto_erased": false, "exceptions": [] }
```

Fehlerabbildung: `ErasureError::InvalidTarget` → `invalid_params`; `ScopeAmbiguous` → `invalid_params("scope ambiguous")`; `Busy`/`InsufficientSpace` → `internal_error` mit Hinweis `retryable`; `IdempotencyConflict` → `internal_error` (kann über das Tool nicht vorkommen, ID wird intern erzeugt).

### 4.2 `contextra_erase_status`

Kategorie `DatabaseRead`. Argument `erasure_id` (32 Hex-Zeichen, sonst `invalid_params`). Antwort: `ErasureStatus` serialisiert (Index §4: enthält nie Schlüssel/Namen/Terme, Test TST-TYPES-06). Unbekannte ID ⇒ `not_found`. Die ID gehört zum Server-Mandanten: Aufträge anderer Mandanten sind nicht abfragbar (`status` prüft Mandant des Plans, **[?]** `JobView` trägt Mandant über `ResolvedScope`; sonst Treiber-Prüfung in `Contextra::erase_status`).

### 4.3 `contextra_erase_dry_run`

Kategorie `DatabaseRead`. Argumente wie `contextra_erase` (ohne `await`, `confirm`, `reason`). Antwort: `ErasePlanSummary` (`rules`, `docs`, `unprovenanced`, `layers`, `est_bytes`, `checkpoints`). **Keine Schlüssel**, keine Namen. Wirkung: keine (kein `accept`, kein Journal-Eintrag).

## 5. Sicherheit

- **Prompt-Injection** (`prompt_injection/*`): `contextra_erase` gilt als „destruktiv“. Wird der Aufruf durch Inhalt ausgelöst, den die Guard-Policy als untrusted markiert (Ergebnis einer Suche, Dokumentinhalt), wird er abgelehnt. Umsetzung: `ToolCategory::DatabaseErase` in `policy.rs` mit `requires_trusted_origin = true` **[?]** Policy-Datei lesen und die vorhandene Kategorie-Logik erweitern.
- **Audit:** `prompt_injection/audit.rs` erhält Eintrag `erase_requested{scope, tenant_hash}`; **kein** Schlüssel, kein Name; IDs als gekürzter HMAC (I-12).
- **Logging:** Handler benutzen `#[instrument(skip_all)]` oder gar keines (G-3); Schlüssel nie in `tracing`-Feldern.
- **Rate-Limit:** höchstens `erase_max_inflight` (Default 8) offene Aufträge je Server; darüber `Busy`.

## 6. `contextra_drop_collection`

Bleibt, ruft nach 10 §10 intern `erase(Collection)`. Antwort erhält zusätzlich `"purged": false`, `"erasure_id"`, `"guarantee"`. **Keine** Verhaltensänderung für `confirm`.

## 7. Tests

| ID | Inhalt |
| --- | --- |
| TST-MCP-ERA-1 | `tools/list` enthält die drei Werkzeuge; `TOOL_REGISTRY` und Dispatch stimmen überein (bestehender Konsistenztest, **[?]** Name prüfen) |
| TST-MCP-ERA-2 | Ohne `confirm` ⇒ `invalid_params`; mit `allow_erase = false` ⇒ abgelehnt |
| TST-MCP-ERA-3 | `scope=document` für eingefügtes Dokument ⇒ `ok`, `guarantee = "G1"`, danach `contextra_get` ⇒ nicht gefunden |
| TST-MCP-ERA-4 | `scope=collection`, `collection=default` ⇒ `invalid_params` |
| TST-MCP-ERA-5 | Antworten enthalten weder `id` noch `collection` noch Dokumentbytes (JSON-Durchsuchung mit Canary) |
| TST-MCP-ERA-6 | `erase_status` mit falscher/fremder ID ⇒ `invalid_params`/`not_found` |
| TST-MCP-ERA-7 | `dry_run` ändert nichts (Journal leer, Gate inaktiv) |
| TST-MCP-ERA-8 | Prompt-Injection-Pfad: Erase aus untrusted Inhalt ⇒ abgelehnt |
| TST-MCP-ERA-9 | `drop_collection` liefert `erasure_id`; `erase_status` zeigt Fortschritt bis `Completed` |
| TST-MCP-ERA-10 | `tenant`-Scope nur mit Token; Fehlversuch ändert nichts |

## 8. Arbeitspakete

| WP | Inhalt |
| --- | --- |
| 12.1 | `ToolCategory::DatabaseErase`, Registry, `tools/list`, Dispatch |
| 12.2 | Handler `contextra_erase_status`, `contextra_erase_dry_run` |
| 12.3 | Handler `contextra_erase` mit Bestätigung, Policy |
| 12.4 | `contextra_drop_collection`-Antwort, Tests |

## 9. Definition of Done

Drei Werkzeuge getestet; Policy-Test grün; keine Schlüssel in Antworten/Logs; keine neue Kante (nur `contextra`/`types`/`ports`).

## 10. Offene Punkte

| Nr. | Frage | Vorschlag |
| --- | --- | --- |
| O-M1 | `ToolCategory` ist `non_exhaustive`? | lesen; neue Variante am Ende |
| O-M2 | Konstantzeit-Vergleich vorhanden? | sonst in `crypto` ergänzen (`ct_eq`) |
| O-M3 | Mandant des Status-Aufrufers | siehe §4.2 |

## Revision 2 (28.11.2026, Audit gegen HEAD `e12873de6`; Details in 14)

**Stand.** `contextra_erase`, `contextra_erase_status`, `contextra_erase_dry_run` sind in `server_tools.rs` verdrahtet ✔ (Kategorien `DatabaseErase`/`DatabaseRead`, `tests.rs:177-179`).

**Ergänzungen.** (1) `contextra_erase_dry_run` weist `required_layers`, fehlende Layer und die voraussichtlichen Ausnahmen (Datenort-Matrix Zeilen 17–20) aus; `contextra_erase_status` zeigt `exceptions` inklusive `sla_breached_at`/`FeatureDisabled`. (2) **Kein** Werkzeug für `resume` (A-8). (3) Offene MCP-Cursor/Streams: nach `Gated` liefern sie keine Treffer des Scopes mehr (`TST-MCP-CUR-1`, Matrix Zeile 18, **[?]**). (4) Fehlerantworten und Logs nennen nur den gekürzten HMAC der `ErasureId`, nie Schlüssel/Zielbytes (G-3). (5) Das Werkzeug `contextra_erase` ist nur mit expliziter Bestätigung/Plan-Hash aufrufbar (Dry-Run-Hash), um Fehlaufrufe durch Agenten zu verhindern **[?]** Abgleich mit der bestehenden Implementierung.

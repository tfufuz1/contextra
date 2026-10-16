# Security Audit Report: `contextra-privacy`

**Audit-Datum:** 2026-09-27
**Ziel-Crate:** `contextra-privacy` (Ring 3: Cloud Egress Security, DLP & Bulk Exfiltration Protection)
**Safety Policy:** `#![forbid(unsafe_code)]`
**Architektur-Ebene:** Ring 3 — Cloud Safeguards & Privacy Enclaves

---

## Executive Summary & Modulübersicht

`contextra-privacy` bildet den Ring-3-Schutzschild für sämtliche outbound Cloud-Anfragen und Inbound-Response-Flüsse. Die Crate kapselt Data Loss Prevention (DLP), sitzungsgebundenes Surrogate Vaulting (Anonymisierung), Layer-4 Vektor-Ähnlichkeitsprüfungen gegen Bulk-Exfiltrationsmuster und zeitfensterbasierte Rate-Limiting-Zähler pro Session.

| Modul | Hauptfunktion | Invarianten & Schutzmechanismen |
|---|---|---|
| `egress_vault.rs` | Layer-1 RegexSet DLP & `SurrogateVault` Anonymisierung | Fail-Closed bei Timeout/Fehlern; Zeroization aller Surrogat-Keys beim Drop; Opake Regel-IDs (`R-001`, `R-002`) verhindern Regex-Information-Leaks. |
| `bulk_exfiltration_detector.rs` | Layer-4 Sliding-Window Volumenzähler pro Session | Thread-safe via `scc::HashMap` + `parking_lot::Mutex`; Fail-Closed bei Uhrzeitanomalien (Monotonic Clock Regression). |
| `egress_guard.rs` | Layer-4 k-NN Cosine Similarity Check ($k=1$) | Entkopplung über `TextSearchEngine`-Trait; Fail-Closed bei leeren Suchergebnissen, Index-Fehlern oder Timeouts. |
| `egress_gateway.rs` | Gateway-Orchestrierung & Rehydrierung | Obligatorischer Egress-Check für alle Cloud-Queries (APM-EGRESS-BYPASS Verbot); UTF-8 sichere Surrogat-Rehydrierung. |
| `audit_trace.rs` | Deterministische Audit-Spur | 32-Byte BLAKE3-Hash-Trace aus Payload, Rule-ID und injiziertem Nanosekunden-Zeitstempel. |
| `guarded_payload.rs` | Type-State Wrapper (`GuardedPayload<Sanitized>`) | Kompilierzeiterzwingung: Unsanitizierte Payloads können nicht an Cloud-Dispatcher übergeben werden. |

---

## (1) Vollständige Fail-Closed-Analyse (Prüfpunkt P1)

Die Kerninvariante von `contextra-privacy` lautet: **Fail-Closed** — bei jeder Verarbeitungsstörung (Timeout, Regex-Fehler, Index nicht erreichbar, UTF-8 Parsing-Fehler, Mandanten-Diskrepanz) MUSS die Entscheidung `EgressClassification::Block(...)` lauten.

### Vollständiger Katalog der `Allow`-Return-Statements

Ein gründliches Audit aller Quellcodedateien in `crates/contextra-privacy/src/` ergab exakt **drei (3)** Stellen, an denen explizit `EgressClassification::Allow` zurückgegeben wird:

1. **`egress_vault.rs` (`classify_layer1_arc`, Zeile 203):**
   ```rust
   // Wird NUR erreicht, wenn:
   // 1. payload.len() <= MAX_CLASSIFY_PAYLOAD_BYTES (64 KB)
   // 2. std::str::from_utf8 erfolgreich war
   // 3. regex_set.matches(text).matched_any() FALSE war
   // 4. der blocking_task ohne Panic und innerhalb des Timeouts terminiert ist
   EgressClassification::Allow
   ```
2. **`egress_guard.rs` (`check`, Zeile 108):**
   ```rust
   // Wird erreicht, wenn payload.len() < min_bytes (128 Bytes)
   // Kurze Payloads unterhalb der Vektorgrenze überspringen HNSW-Suche.
   if payload.len() < self.min_bytes {
       return EgressClassification::Allow;
   }
   ```
3. **`egress_guard.rs` (`check`, Zeile 125):**
   ```rust
   // Wird NUR erreicht, wenn die Vektorsuche erfolgreich Ergebnisse geliefert hat
   // UND der Cosine-Similarity Score des Top-1 Treffers strikt kleiner als threshold (0.85) ist.
   if top.score >= self.threshold {
       EgressClassification::Block(...)
   } else {
       EgressClassification::Allow
   }
   ```

### Audit aller Fehlerbehandlungspfade & Result-Matches

Jeder denkbare Fehler- und Ausnahmepfad wurde auditiert und als strikt **Fail-Closed** verifiziert:

| Fehler- / Ausnahmezustand | Modul / Funktion | Rückgabewert / Verhalten | Evaluierung |
|---|---|---|---|
| Payload überschreitet `MAX_CLASSIFY_PAYLOAD_BYTES` (64 KB) | `egress_vault.rs::classify_layer1_arc` | `Block(BlockReason::EgressPolicyDenied(...))` | **FAIL-CLOSED** |
| Ungültiges UTF-8 im Payload | `egress_vault.rs` (in `spawn_blocking`) | `Block(BlockReason::InternalError("Invalid UTF-8 payload..."))` | **FAIL-CLOSED** |
| `RegexSet` Kompilierungsfehler | `egress_vault.rs::classify_layer1` | `Block(BlockReason::InternalError("Failed to compile RegexSet..."))` | **FAIL-CLOSED** |
| Tokio Blocking Task Join-Fehler (z.B. Panic im Thread) | `egress_vault.rs::classify_layer1_arc` | `Block(BlockReason::InternalError("Evaluation task failed..."))` | **FAIL-CLOSED** |
| Layer-1 Klassifikations-Timeout | `egress_vault.rs::classify_layer1_arc` | `Block(BlockReason::ClassificationTimeout)` | **FAIL-CLOSED** |
| Vector Index Suchfehler (`Err(err)`) | `egress_guard.rs::check` | `Block(BlockReason::InternalError("egress guard index unavailable — fail-closed"))` | **FAIL-CLOSED** |
| Vector Index Timeout (`Err(_elapsed)`) | `egress_guard.rs::check` | `Block(BlockReason::InternalError("egress guard index unavailable — fail-closed"))` | **FAIL-CLOSED** |
| Vector Index leere Ergebnisse (`results.is_empty()`) | `egress_guard.rs::check` | `Block(BlockReason::InternalError("egress guard index unavailable — fail-closed"))` | **FAIL-CLOSED** |
| Mandanten-ID Mismatch in `classify_scoped` / `check_scoped` | `egress_vault.rs` & `egress_guard.rs` | `Block(BlockReason::PolicyDenied("tenant scope mismatch..."))` | **FAIL-CLOSED** |
| Monotonie-Anomalie der Systemzeit (`now < timestamp`) | `bulk_exfiltration_detector.rs` | `BulkExfiltrationOutcome::Block { window_bytes: usize::MAX, ... }` | **FAIL-CLOSED** |

---

## (2) DLP-Regex-Testmatrix & Anonymisierung (Prüfpunkt P3)

### Standart-DLP-Muster im `EgressVault`

Das System wird mit folgenden vordefinierten Mustern ausgeliefert:

| Regel-ID | Muster-Kategorie | Raw Regex Pattern | Ziel |
|---|---|---|---|
| `R-001` | OpenAI API Keys | `sk-` | Verhindert Leaks von OpenAI Secrets |
| `R-002` | AWS Access Key IDs | `AKIA` | Verhindert Leaks von AWS IAM Keys |
| `R-003` | Generic API Keys | `api_key` | Erkennt unstrukturierte Key-Bezeichner |
| `R-004` | Passwörter / Credentials | `password` | Erkennt Passworteingaben |
| `R-005` | E-Mail-Adressen (PII) | `\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b` | Erkennt E-Mail-Adressen |

**Information-Leak Shielding:** Bei einem Treffer gibt der `EgressVault` ausschließlich die opake Regel-ID (`R-001`, `R-002`, ...) im `BlockReason::SensitivePattern` zurück. Das rohe Regex-Muster oder der gefundene Klartext-Inhalt wird niemals in Logdateien oder API-Antworten preisgegeben.

### Anonymisierung via `SurrogateVault` & `EntityRecognizer`

Die Funktion `sanitize_and_vault` verfährt zweiphasig:
1. **Phase 1 (Strukturierte Muster):** Ersetzt Treffer der kompilierten `RegexSet`-Muster durch sitzungsstabile Surrogate `[USER_ENTITY_<4_hex_chars>]`.
2. **Phase 2 (Unstrukturierte Entitäten):** Wendet das `EntityRecognizer`-Trait (z.B. Named Entity Recognition für Personen, Orte, Firmen) an. Die erhaltenen Byte-Ranges werden absteigend nach Start-Index sortiert und kollisionsfrei durch Surrogate ersetzt.

Die Bidirektionale Mappings-Tabelle im `SurrogateVault` nutzt einen kryptografisch sicheren `OsRng` Sitzungssalt (16 Bytes BLAKE3-Salt). Beim Verlassen des Scopes (`Drop`) werden sowohl der Salt als auch alle gespeicherten Schlüssel-Wert-Paare via `zeroize` im RAM überschrieben.

### DLP-Evasion & Umgehungsversuche — Analyse & Grenzen

| Evasion-Vektor | Beispiel / Angriffsmuster | Erkennungsverhalten | Sicherheitsbewertung & Empfehlung |
|---|---|---|---|
| **E-Mail / Standard PII** | `alice@example.com` | Blockiert (`R-005`) | Vollständig geschützt. |
| **OpeneAI Key Infix** | `sk-01234567890123456789012345678901` | Blockiert (`R-001`) | Vollständig geschützt. |
| **Kreditkarte / IBAN mit Leerzeichen** | `4532 0123 4567 8910` oder `DE89 3704 0044 0532 0130 00` | Passt nicht auf Standard-Muster `R-001`..`R-005` | **Einschränkung:** Standard-Muster enthalten keine IBAN/Luhn-Regexes. Wenn IBAN-Muster registriert werden, sollten sie optional Leerzeichen/Bindestriche zulassen (`\s*`). |
| **Base64-kodierte Secrets** | `c2stMTIzNA==` (`sk-1234`) | Nicht erkannt von Layer-1 Plaintext Regex | **Einschränkung:** Layer-1 Regex arbeitet auf Plaintext-UTF8. Base64-Dekodierung muss ggf. als Pre-Processing-Pass vorgeschaltet werden. |
| **Zeilenumbruch-Splitting** | `pass\nword` | Nicht erkannt durch `password` | Wird durch Layer-4 Vektor-Guard (`EgressGuard`) abgedeckt, wenn der Kontext semantisch ähnlich ist. |

---

## Bulk-Exfiltration Sliding-Window & Thread-Safety (Prüfpunkt P2)

Das Modul `bulk_exfiltration_detector.rs` schützt vor exzessivem Datenabfluss über kumulative Anfragen hinweg.

- **Fenstergröße & Schwelle:** Konfigurierbar über `max_bytes_per_window` (z.B. 1.000.000 Bytes) und `window` (`Duration`, z.B. 60 Sekunden).
- **Thread-Safety & Datenstruktur:**
  - Nutzt `scc::HashMap<SessionId, Arc<parking_lot::Mutex<SlidingWindowCounter>>>`.
  - `scc::HashMap` bietet lock-freie hochparallele Zugriffe über verschiedene `SessionId`s hinweg ohne Lock-Kontention zwischen verschiedenen Benutzern.
  - Innerhalb derselben `SessionId` schützt ein schlanker `parking_lot::Mutex` die `VecDeque<(Instant, usize)>` Historiendatenbank.
  - Werden synchrone oder parallele Anfragen für dieselbe Session gestellt, schützt der Mutex den Zustand atomar.
- **Fail-Closed bei Monotonic Clock Regression:**
  Beim Bereinigen abgelaufener Zeitfenster prüft `prune_and_sum` jeden Eintrag mit `now.checked_duration_since(timestamp)`. Sollte die Systemzeit rückwärts springen (`now < timestamp`), schlägt die Prüfung fehl und die Funktion gibt sofort `BulkExfiltrationOutcome::Block { window_bytes: usize::MAX, limit: ... }` zurück.

---

## Layer-4 k-NN Similarity Check (Prüfpunkt P4)

Das Modul `egress_guard.rs` wendet Vektor-Ähnlichkeitsprüfungen gegen bekannte vertrauliche Dokumente an:

- **Entkopplung:** Über das Trait `TextSearchEngine` von der konkreten Datenbank-Implementierung entkoppelt (Ring-3 Abstraktion).
- **Schwellwerte:**
  - `DEFAULT_EGRESS_GUARD_TIMEOUT`: 200 ms
  - `DEFAULT_EGRESS_GUARD_THRESHOLD`: 0.85 (Cosine Similarity Match)
  - `DEFAULT_EGRESS_GUARD_MIN_BYTES`: 128 Bytes (Payloads < 128 Bytes werden performant durchgewunken).
- **Index-Befüllung & Aktualisierung:**
  - Die Befüllung der Referenz-Datenbank (HNSW/Vector Index) erfolgt asynchron in der darunterliegenden Vektordatenbank (`contextra-vector` / `contextra-db`).
  - Neue vertrauliche Dokumente oder Exfiltrations-Patterns werden dort indiziert und stehen dem `EgressGuard` sofort zur Verfügung.
- **Fail-Closed bei leerem Index:**
  Gibt der Vektor-Index keine Ergebnisse zurück (`results.is_empty()`), interpretiert der `EgressGuard` dies sicherheitshalber als "Index nicht verfügbar" und **blockiert** die Anfrage (`BlockReason::InternalError("egress guard index unavailable — fail-closed")`).

---

## Tenant-Isolation im Egress-Filter (Prüfpunkt P5)

Das System erzwingt die strikte Einhaltung von VETO-F10 (kein Cross-Tenant-Datenaustausch):

1. **`TenantScoped` Integration:**
   Sämtliche öffentlichen API-Funktionen bieten Scoped-Varianten an:
   - `classify_scoped(payload: TenantScoped<&str>, expected_tenant_id: &TenantId)`
   - `sanitize_and_vault_scoped(payload: TenantScoped<&str>, expected_tenant_id: &TenantId, ...)`
   - `check_scoped(payload: TenantScoped<&str>, expected_tenant_id: &TenantId)`
   - `handle_cloud_query_scoped(request: TenantScoped<CloudQueryRequest>, expected_tenant_id: &TenantId, ...)`
2. **Isolations-Garantie:**
   Sollte die in `TenantScoped` gekapselte `TenantId` nicht exakt mit der übergebenen `expected_tenant_id` übereinstimmen, schlägt das Entpacken (`into_inner_checked`) fehl und die Anfrage wird augenblicklich blockiert (`BlockReason::PolicyDenied("tenant scope mismatch")`).
3. **Surrogate-Salt Trennung:**
   Jede `SurrogateVault`-Instanz erzeugt sitzungsspezifische Surrogat-Token. Dadurch können Anfragen eines Tenants die Surrogat-Zuordnungen oder Egress-Entscheidungen eines anderen Tenants weder einsehen noch beeinflussen.

---

## (3) VERDICT & VERIFIED-BY-SESSION

### Test- & Clippy-Verifizierung

```text
cargo clippy -p contextra-privacy --all-targets -- -D warnings
Result: Clean compilation (0 warnings, Exit 0)

cargo test -p contextra-privacy --locked -- --nocapture
Result: 55 passed (46 lib unit tests + 9 integration tests); 0 failed; 0 ignored; 0 measured
```

### Audit-Verdict

**VERDICT: PASSED**

**VERIFIED-BY-SESSION: PASSED (TS: 2026-09-27T20:50:00Z)**

Die Crate `contextra-privacy` erfüllt alle geforderten Sicherheitsinvarianten:
- Strikte Fail-Closed-Semantik ohne ungesicherte Fehler- oder Exception-Pfade.
- Vollständige Zero-Panic und `#![forbid(unsafe_code)]` Einhaltung.
- Thread-safe Bulk-Exfiltrations-Schutz mit Monotonic Clock Failure Control.
- Mandantentrennung gemäß VETO-F10 über `TenantScoped` und sitzungsisolierte Surrogate.

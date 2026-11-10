# contextra-vault — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v2, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `f0eda1cb9`). `cargo` ist in der Analyseumgebung nicht verfügbar. **Es wurde nichts gebaut, getestet oder ausgeführt.**
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-vault/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Flüchtiger Chunk-Tresor (aus `db` herausgelöst), baut auf `sys::vault` auf. *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | safety |
| Ebene (Kern-Closure) | 1 |
| Kern-Closure | sys |
| Default-Features | — |
| Fehlertyp (R3) | `VaultError` |
| Brücke (hinter Feature `workspace`, §1.1) | `From` |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1 = Kern-Closure ⊆ E0, höchstens 4 Mitglieder · E2 = Integration · E3 = Anwendungen und Testwerkzeuge. Der Kern ist der Code mit `--no-default-features` ohne Feature `workspace`.

## 3. Ist-Zustand (tools/)

*Crate existiert im Repo noch nicht. Es wird in S-10/S-11 bzw. im jeweiligen Schritt angelegt.*

## 4. Ist-vs-Ziel-Abgleich [V]

Crate existiert noch nicht. Ist-Vergleich entfällt; Ziel siehe §2.

## 5. Verifizierte Befunde und Abweichungen zu docs/SPEC.md [V]

- Keine zusätzlichen Abweichungen in dieser Prüfung festgestellt. Die Aussagen aus docs/SPEC.md sind nur dort übernommen, wo sie oben nicht widersprochen werden.

## 6. Anforderungen

### 6.1 Crate-Spezifikation (docs/SPEC.md §6) [S]

Quelle `db/src/volatile_vault.rs` (592 Z.); `VaultError`; `contextra_types`-Bezug (2×) nur in `compat/`; baut auf `sys::vault`; `db` re-exportiert (`db/lib.rs:108`).

### 6.2 Brückenmuster [S]

Die Brücke liegt ausschließlich in `src/compat/` hinter dem Feature `workspace`. Verbindliche Muster: docs/SPEC.md §4.3 (a–d). R2 gilt: `contextra_types`, `contextra_ports`, `contextra_testkit` dürfen im Kern nirgends vorkommen, auch nicht in Doc-Kommentaren oder `#[cfg(test)]`.

### 6.3 Regeln für jeden Schritt [S]

INV-1 Formatgleichheit · INV-2 Pfadgleichheit (Re-Export mit `#[deprecated]`) · INV-3 Ein Schritt = Commit-Paar · INV-4 `cargo check --workspace --all-targets --locked` grün · INV-5 Determinismus (Tie-Break nach ID aufsteigend) · INV-6 Durability-Pfade unberührt · INV-7 keine Compliance-Aussagen · INV-8 keine Rückwärtspfeile. Vollständig: docs/SPEC.md §1.2, Regeln R1–R9: §3.2.

### 6.5 Fehlerabbildung [S]

Eigener Fehlertyp: `VaultError` (thiserror, `#[non_exhaustive]`). Jede heute genutzte `ContextraError`-Variante hat genau eine lokale Entsprechung; `Display` bleibt bitgleich; keine Sammelvariante `Other(String)` ohne Quelle. Tabelle je Baustein: `docs/decoupling/errors/<crate>.md` (aus S-xx erzeugt). Quelle: §5.3.

## 7. Schnittstellen und Persistenz

Keine Zeile aus dem Golden-Korpus (§5.1) bezieht sich auf dieses Crate.

### 7.2 Öffentliche Schnittstelle

Ist-Stand und Ziel-Schnittstelle sind in den Abschnitten 3 bis 6 festgehalten. Die exakte Liste der `pub`-Symbole erhebt S-00 über `cargo public-api` (Gate G-A1); erlaubte Differenzen sind nur Re-Export-Pfade mit `#[deprecated]`.

## 8. Fundstellen (tools/, exakt) [T]

Keine Befunde in den Audit-Berichten.

## 9. Befunde aus docs/SPEC.md (B-Nummern) [S]

**B16 — Einzelentscheidungen, die der Entwurf offen ließ**
> `contextra-vault` vs. `sys/vault.rs` — Öffentliche Namen gemeinsam: 3; `sys::vault` = gesperrter Speicher (`allocate`, `as_slice`, `is_locked`), `db::volatile_vault` = Chunk-Tresor (`ingest`, `commit`, `SignalModality`) [V] — **Aufbauend, nicht ersetzend:** `contextra-vault` (E1, Kern-Closure …

## 10. Entscheidungen und Risiken [S]

**Entscheidungen (§9.1):**

| D-07 | `vault` baut auf `sys::vault`; `mcp::protocol` und `wire::jsonrpc` bleiben getrennt; `adapt::drift`/`rank::drift` und die beiden `query_rewriter` bleiben getrennt (B16) |

Nicht-Ziele (§9.3), gelten für alle Crates: physische Regruppierung, Mikro-Crate-Zerlegung (F-1), Verschieben von `Vfs`/`io_seam`, crates.io-Veröffentlichung, Formatänderungen außer ID-Breite, Änderung der FlatBuffers-Schemas.

## 11. Abnahme [S]

Ein Crate gilt als entnehmbar (R6), wenn in dieser Reihenfolge grün: **G-L2** (Null-Prüfungen, nur bei Brücke) → **G-L1** (Kern- und Feature-Closure) → **G-L3** (`--no-default-features`, `--each-feature`) → **G-L4** (isolierter Export). Danach **G-F1/G-D1/G-E1/G-C1** für den betroffenen Schritt (§8.4).

```bash
cargo check -p contextra-vault --no-default-features --locked
cargo hack check -p contextra-vault --each-feature --no-dev-deps --locked
python3 tools/isolation_proof.py --crate contextra-vault --modes no-default,default,each-feature
```


# CLAUDE.md — Contextra Workspace Kurzreferenz

**Status:** Normativ (Quelle der Wahrheit: `docs/SPEC.md` v1.0)
**Datum:** 18. November 2026
**Zielgruppe:** KI-Assistenten (Claude, Gemini, GPT u. a.)

---

## Dokumentenhierarchie

1. `docs/SPEC.md` — Gesamtspezifikation (Single Source of Truth)
2. `docs/crates/*.md` — Crate-Spezifikationen (normativ pro Crate)
3. `AGENTS.md` (Wurzel) — Agenten- & Entwicklerrichtlinien
4. `crates/contextra-*/AGENTS.md` — Crate-lokale Agentenrichtlinien
5. `CLAUDE.md` (diese Datei) — Kurzreferenz für AI-Assistenten

---

## Produktdefinition

Contextra ist eine **souveräne, offline betreibbare Such- und Wissens-Engine** als eingebettete Rust-Bibliothek (kein Server). Primäre Schnittstelle: Rust-API; sekundär: MCP über stdio.

**Kernfähigkeiten:** Hybrid-Suche (Vektor + Volltext + Graph), Versionierung/Zeitreise, DSGVO-konforme Löschung mit Crypto-Shredding, keine Telemetrie.

**Nicht-Ziele:** Keine automatisierte Entscheidung über Personen, kein Training mit PII, kein DMS.

---

## Architektur (23 Crates, 4 Ebenen)

```
E3 Endpunkte:    mcp, testkit, bench
E2 Integration:  contextra (Façade), db, engine (deprecated), types, ports
E1 Domäne:       store, vector, text, graph, rank, crypto, mvcc, checkpoint
E0 Fundament:    ids, env, sys (unsafe), durable-fs, simd (unsafe), privacy, erasure
```

**Abhängigkeitsregeln:**
- Kanten nur abwärts (E3→E2→E1→E0), nie aufwärts, keine zirkulären Dev-Deps
- `contextra-vault` ist dormant (`exclude`), 13 Crates sind entfernt (Gate `G-REACTIVATE` nötig)

---

## Entwicklungsrichtlinien (Kurzform)

| Regel | Beschreibung |
|-------|-------------|
| **Unsafe** | `#![forbid(unsafe_code)]` überall außer `sys` und `simd` |
| **Zero-Panic** | Kein `unwrap()`, `expect()`, `panic!()`, `todo!()` in Nicht-Test-Code |
| **Fail-Closed** | Fehlender Schlüssel → `Err`, nie Klartext-Fallback |
| **Zeroize** | Schlüsselmaterial: `ZeroizeOnDrop`, kein `Clone`, kein `Debug` |
| **Determinismus** | Gleiche Eingabe → gleiche Rangfolge, Tie-Break nach `DocId` aufsteigend |
| **Features** | `default`-Features in allen Crates leer |
| **Lints** | `clippy::unwrap_used`, `expect_used`, `panic`, `todo` = `deny` |

---

## Datenklassen

| Klasse | Name | Verschlüsselung | Löschbar pro Betroffenen |
|--------|------|-----------------|-------------------------|
| **K** | Unternehmenswissen | Collection-KEK | Nein (Collection-Drop) |
| **P** | Personenbezogene Daten | SubjectKey → SubjectDocDEK | Ja, atomar O(1) |
| **D** | Abgeleitete Daten | Mit Lineage zum Quelldatum | Ja (kaskadiert) |
| **E** | Löschmaterial | Separater ErasureKey | Nach Aufbewahrungsfrist |

**K-Schreibpfade** akzeptieren nur `Validated<T>` aus `contextra-privacy`.
**P-Daten** werden ausschließlich über `SubjectRef` (128-Bit-Zufall) adressiert.

---

## Löschkette (6 Schritte)

1. **Validierung & Journal** (fsync #1) — Aufbewahrungspflicht prüfen, Intent loggen
2. **Gate setzen** (fsync #2) — `CommitBarrier::exclusive()`, ReadGate blockiert Lesepfade
3. **Schlüsselvernichtung** (fsync #3) — DEK/KEK/SubjectKey nullen, `verify_key_absent`
4. **Physische Bereinigung** — Asynchron: Snapshot-Widerruf → Text → WAL/SSTable → HNSW → Graph → Crypto → Cache
5. **Messung** — Canary-Positivkontrolle, `Scanner<Armed>`, alle `residue_matches == 0`
6. **Bericht & Ledger** — Ed25519-signiert, Hash-verkettetes Ledger außerhalb des Datenverzeichnisses

**ReadGate:** Jeder Lesepfad (`get`, `scan`, `search`, `hybrid_search`, `graph_query`, `export`, `mcp`) prüft synchron O(1) gegen das Gate. Bei `Erased` → sofortiger Abbruch, Fail-Closed.

---

## Schlüsselhierarchie

```
MasterKey (Passphrase → Argon2id → 256 Bit)
  ├── HKDF → TenantKEK(tenant_id)
  │            ├── HKDF → CollectionKEK(collection_id, epoch)
  │            │            ├── HKDF → DocumentDEK (Klasse K)
  │            │            └── HKDF → SubjectKey → SubjectDocDEK (Klasse P)
  │            └── Epochenrotation bei Collection-Neuanlage
  └── HKDF → ErasureKey (Journal, Ledger, Berichte)
```

**Algorithmen:** AES-256-GCM-SIV (AEAD), HKDF-SHA-256, Argon2id, Ed25519 (Berichte), BLAKE3 (IDs)

---

## Toolchain & Build

- **Rust-Toolchain:** 1.89 (in der VM nicht installiert)
- **Rust-Installation:** `crates.io` erreichbar, `static.rust-lang.org` nicht. Nutze `conda-forge` (`conda.anaconda.org` erlaubt) oder `apt` mit `rustc-1.89` / `cargo-1.89`
- **Profil:** Release mit `lto = "fat"`, `codegen-units = 1`, `overflow-checks = true`
- **Audits:** Siehe `reports/` für aktuelle Audit-Berichte

---

## Wichtige Invarianten

| ID | Invariante |
|----|-----------|
| INV-1 | Kein persistiertes Byte ändert sich ohne Formatvermerk |
| INV-3 | `forbid(unsafe_code)` außer `sys`, `simd` |
| INV-4 | Kein `unwrap`/`expect`/`panic!`/`todo!` in Nicht-Test-Code |
| INV-5 | Gleiche Eingabe → gleiche Rangfolge, Tie-Break DocId aufsteigend |
| INV-6 | WAL, Manifest, Flush, Recovery-Verhalten unverändert bei Umbau |
| INV-7 | Keine Compliance-Aussagen. „Gelöscht" nur mit gemessener Stufe |
| INV-8 | Abhängigkeitskanten nur abwärts |
| INV-9 | `Completed` und `Refused` sind terminale Saga-Zustände |
| INV-10 | Gate wird nie zurückgenommen |
| INV-11 | Schlüsselbytes mit `zeroize` genullt vor Drop |
| INV-12 | Logs/Traces enthalten nie Klartextinhalte des Löschziels |

---

## Lieferformat

Wenn Dateien geliefert werden: Verzeichnisstruktur vom Wurzelverzeichnis aus, damit der Nutzer einfach überschreiben kann. Zu löschende Dateien als leere Dateien mit Inhalt `DELETED` liefern.

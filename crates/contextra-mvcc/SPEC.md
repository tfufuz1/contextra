# contextra-mvcc — Spezifikation (v3)

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v3, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `5408826f8` / v3).
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-mvcc/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Multi-Version Concurrency Control (MVCC): Snapshot-Registry, Serializable Snapshot Isolation (SSI), SequenceLog, TxBuffer, Tombstones. *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | foundation / core |
| Ebene (Kern-Closure) | E1a |
| Kern-Closure | `ids`, `env` |
| Default-Features | `workspace` (für Workspace-Konsumenten); `--no-default-features` für Standalone |
| Fehlertyp (R3) | `MvccError` (lokal, stark typisiert) |
| Brücke (hinter Feature `workspace`, §1.1) | `From<MvccError>`, `contextra_types::ContextraError` in `src/compat/` |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1a = Kern-Closure ⊆ E0, höchstens 4 Mitglieder (`ids`, `env`) · E1b = mindestens ein E1a-Mitglied (`store`, `vector`) · E2 = Integration · E3 = Anwendungen und Testwerkzeuge.

## 3. Ist-Zustand (tools/)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | ~3290 Zeilen in 8 Produktionsdateien |
| Version / publish | workspace-vererbt |
| Interne Normal-Abhängigkeiten (Kern) | `contextra-ids`, `contextra-env` |
| Interne Kompatibilitäts-Abhängigkeiten (`workspace`) | `contextra-types`, `contextra-ports` (optional) |
| Ungegatete Brückenreferenzen (`count_bridge_refs.py`) | **0** |
| `#[forbid(unsafe_code)]` | Ja (100 % sicher) |
| zero-panic (Produktionscode) | 0 Unwraps/Expects/Panics im Produktionspfad |
| Fehler-Enum | `MvccError` (lokal, stark typisiert) |
| Kommentierungssystem | Standardisiertes Comment-Anchors-System (`ANCHOR`, `SECTION`, `LINK`, `NOTE`) |

## 4. Kern- und Brückenarchitektur

### 4.1 Kernkomponenten (Ebene E1a)
1. `types`: Re-Export kanonischer Identifikatoren (`DocId`, `TenantId`, `TxId`, `TOMBSTONE_BIT`) aus `contextra-ids`.
2. `seq_log`: Monotone Dokument-Versionsverfolgung für $O(1)$-Sichtbarkeitsabfragen (`SeqLogEntry`, `SequenceLog`).
3. `snapshot`: RAII-geschützte Snapshots (`SnapshotRegistry`, `SnapshotGuard`) mit lock-freiem Watermark-Tracking (`min_active_seqno`).
4. `tx_buffer`: Sharded Staging-Buffer (`TxBuffer`, `IndexOp`) mit Deadlock-Freiheit (`INV-LOCK-ORDER-1`) und DoS-Schutz (Bounded Staging).
5. `ssi`: Serializable Snapshot Isolation (`ReadSet`, `SequenceLogSsiValidator`) mit Write-Skew-Schutz, Phantom-Bereichsscans und konservativer LCP-Vergröberung (`B-16`).
6. `tombstone`: Trait und Referenzimplementierung für Tombstone-Semantikprüfung (`TombstoneSemanticsCheck`, `SeqBitTombstone`).
7. `mvcc_error`: Lokale, geschlossene Fehlerhierarchie (`MvccError`).

### 4.2 Brücke (`compat/`)
- Liegt strikt in `src/compat/` hinter dem Feature `workspace`.
- Implementiert `From<MvccError> for ContextraError`.
- Keine `contextra_types` oder `contextra_ports`-Symbole außerhalb von `src/compat/` oder `#[cfg(feature = "workspace")]`.

## 5. Abnahme & Isolation (G-L1 bis G-L4)

- **G-L1 (Kern-Closure):** `ids`, `env` (E0).
- **G-L2 (Null-Prüfung):** `tools/count_bridge_refs.py -c contextra-mvcc` ergibt exakt 0 ungegatete Treffer.
- **G-L3 (Feature-Matrix):** Kompiliert fehlerfrei sowohl unter `--no-default-features` als auch mit `workspace`.
- **G-L4 (Isolationsnachweis):** Standalone-Kompilation und Testausführung ohne Abhängigkeit von höheren Workspace-Schichten.

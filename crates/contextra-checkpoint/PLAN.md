# contextra-checkpoint — Implementierungsspezifikation

**Ebene:** E1 (Kern, R2)
**Normative Quelle:** `docs/SPEC.md` v4.1, Nachtrag v4.1.1 (maßgeblich bei Widerspruch). **Stand:** 21.11.2026, Abgleich mit SPEC v4.1. Verweist nur auf SPEC-Kapitel und IDs. Ist-Abweichungen sind markiert, ungeklärte Punkte stehen unter „Offene Punkte (Entscheidung nötig)“ in Abschnitt 6.

## 1. Zweck und Ebene
Hartlink-Snapshot und Restore, Ausschluss von `keys/` und `erasure/` (ADR-42), Retention nach `checkpoint_retention` (SPEC 15, 3.1). Ziel-Umfang: 3.000 Zeilen (SPEC 3.1). Ist: 10.901 Zeilen laut Zielspalte (Altbestand).
Abhängigkeiten nach Anhang E: `ports`, `ids`, `env`, `durable-fs`. Dev-Abhängigkeit `testkit` (SPEC 15).

## 2. Vertrag
Implementiert (SPEC 3.4): `Checkpointer`. Ist: `Checkpoint` und `CheckpointCoordinator` in `contextra-ports` (Datei checkpoint.rs), Name abweichend (siehe PLAN von `contextra-ports`, O-1).

## 3. Abhängigkeiten
**Erlaubt (Anhang E):** `ports`, `ids`, `env`, `durable-fs`.
**Ist (Cargo.toml):** erlaubt sind `contextra-ids`, `contextra-env`, `contextra-ports`. **Verboten:** `contextra-types` (R1, REM-30, 9 Dateien). Weitere: `tokio` (**normale Abhängigkeit**, Wrapper in `deny.toml`, Übergang E29), `bytes`, `tracing`, `serde`, `serde_json`, `parking_lot`, `blake3`, `thiserror`.
**Ist-Abweichung:** `contextra-durable-fs` fehlt in Ist, obwohl erlaubt und nötig. Im Quelltext wird `contextra_mvcc` in einer Datei referenziert (R2-Verstoß, Anhang E erlaubt `mvcc` nicht).
**Features:** Ist `default`, `workspace`, `types-compat`, `ports-compat`, `docid-128`. Ist-Abweichung: `workspace`, `types-compat`, `ports-compat` (R7, CMP-10).

## 4. Module
| Datei | Verantwortung (Ist) | Befund |
|---|---|---|
| `src/hardlink_cloner.rs` | Hartlink-Kopie (Snapshot) | Soll |
| `src/manifest.rs` | Manifest | Soll |
| `src/meta.rs`, `src/meta/tests.rs` | Metadaten | Soll, zu prüfen |
| `src/guard.rs`, `src/guard/tests.rs` | Schutz (Ausschluss keys/erasure) | Soll, zu prüfen |
| `src/orphan.rs`, `src/orphan/tests.rs` | verwaiste Dateien | zu prüfen |
| `src/store.rs`, `src/store/tests.rs` | Speicher-Anbindung | zu prüfen; Abhängigkeit von `store` ist verboten |
| `src/snapshot_source.rs` | Snapshot-Quelle | Soll, zu prüfen |
| `src/fs_seam.rs`, `src/vfs.rs` | Dateisystem-Naht | zu prüfen, Verbleib gegen `durable-fs` |
| `src/compat/*` (ports_impl, snapshot_bridge, error, mod) | Kompatibilitätsmodul | **entfernen** (CMP-2, R7, G-11) |
| `src/contract.rs` | Selbstdeklaration | REM-14 |
| `src/error.rs`, `src/lib.rs` | Fehler, Wurzel | Soll |

## 5. Entfernungen
| ID | Gegenstand | Abnahme-Befehl |
|---|---|---|
| CMP-2 | `src/compat/` | `find crates/contextra-checkpoint -type d -name compat` leer |
| REM-30 | Import `contextra-types` (9 Dateien) | `grep -rln "contextra_types" crates/contextra-checkpoint` leer |
| REM-27 | Import `contextra_mvcc` | `grep -rn "contextra_mvcc" crates/contextra-checkpoint` leer |
| CMP-10 | Features `workspace`, `types-compat`, `ports-compat` | `grep -n "compat\|workspace" crates/contextra-checkpoint/Cargo.toml` leer |
| REM-14 | ANCHOR-Dateien (7), `contract.rs` | `grep -rl "ANCHOR\[" crates/contextra-checkpoint` leer |

## 6. Umbau
1. `compat/` auflösen (CMP-2). Die Zieltypen aus `ports` verwenden.
2. `contextra-types` und `contextra-mvcc` lösen (REM-30, REM-27).
3. Abhängigkeit `contextra-durable-fs` ergänzen.
4. `tokio` klären (Punkt 10 CLAUDE.md).
5. `unwrap`/`expect`/`panic!` (128 Treffer) nach R6 abbauen.

**Offene Punkte (Entscheidung nötig):**
- **O-1** `tokio` als normale Abhängigkeit (Punkt 10 CLAUDE.md).
- **O-2** `store.rs`: Zuordnung, da `store` nach Anhang E nicht aus `checkpoint` erreichbar sein darf.
- **O-3** Inhalt von CMP-2 nach SPEC 12.5 bestätigen.

## 7. Tests
- Ausschluss von `keys/` und `erasure/`, Restore mit Gate-Erhalt (SPEC 15).
- Dev-Abhängigkeit `testkit`: in Ist nicht gefunden, prüfen.
- Ist: nicht gezählt.

## 8. Abnahme
| Gate | Ist-Befund | Soll |
|---|---|---|
| G-02 | kein `unsafe`, `forbid` in `lib.rs` | eingehalten |
| G-03 | `contextra-types`, `contextra_mvcc` | verletzt |
| G-09 | 7 Dateien mit ANCHOR, `contract.rs` | verletzt (REM-14) |
| G-11 | `compat/`, Features | verletzt (CMP-2, CMP-10) |
| G-13 | 128 Treffer | Abbau nach R6 |

# contextra-testkit — Crate-Spezifikation (Soll-Zustand, SPEC 2.1)

**Ebene:** E3 (Endpunkt, Erasure Lab)
**Kern-Closure:** `ids`, `env`, `types`, `ports`, `erasure`, `durable-fs`
**Status:** Normativ (Soll-Zustand)
**Version:** 2.1
**Geltungsbereich:** `crates/contextra-testkit`
**Normative Quellen:** `docs/SPEC.md` §5.8, §12, §15, INV-WITNESS-MINT-1, INV-WIRING-2, R2.

---

## 1. Zweck

`contextra-testkit` ist das Erasure Lab und die gemeinsame Testinfrastruktur.

- In-Memory-Doubles der Ports `StorageEngine`, `VectorIndex`, `TextIndex`, `GraphIndex`.
- Fehlerinjektion für I/O, volle Platte und Handle-Poisoning.
- Canary-Erzeugung für die Positivkontrolle `AC-1`.
- Der Layering-Gate-Test `tests/layering.rs` (SPEC §15).
- Synthetische Korpora mit festem Seed.

E0- und E1-Crates binden dieses Crate weder als Abhängigkeit noch als Dev-Abhängigkeit (R2).

## 2. Architekturregeln

1. `#![forbid(unsafe_code)]`. Zero-Panic in Hilfscode, der von Tests als Bibliothek aufgerufen wird. `default = []`.
2. `Scanner<Armed>` wird hier nicht konstruiert. Der einzige Übergang von `Scanner<Unarmed>` zu `Scanner<Armed>` ist `Chain::verify_control` nach bestandener Positivkontrolle (INV-WITNESS-MINT-1).
3. Ein Double, das `purge` oder `witness` mit `Ok` beantwortet, ohne den Canary zu entfernen oder die Messung wirklich auszuführen, ist ein Stub und darf nicht als Schichtregistrierung dienen (INV-WIRING-2).
4. Kein Test gilt als Löschtest, wenn er nur das Gate setzt. Der Katalog verlangt den Durchlauf der Saga einschließlich Purge (SPEC §12). Ein reiner Gate-Test prüft höchstens Klasse L.

## 3. Positivkontrolle

`CanaryPattern` ist 32 Byte. `generate` füllt sie über `OsRng::try_fill_bytes` und liefert `Err`, wenn der RNG scheitert. `fill_bytes` ohne Fehlerpfad wird nicht verwendet.

Der Canary wird über den Schreibpfad der zu messenden Schicht eingefügt, nicht nachträglich in eine Datei gepatcht.

`CanaryScanner::scan_dir` zählt Rohbyte-Treffer. Das Ergebnis ist eine Zahl. Es ist kein `LayerWitness` und kein `Scanner<Armed>`. Ein Test, der nur `scan_dir` aufruft, hat `AC-1` nicht bestanden.

T-DEL-7 ist erfüllt, wenn ein `Scanner<Unarmed>` den eingefügten Canary findet und `Chain::verify_control` danach einen `Scanner<Armed>` erzeugt. Schlägt das Finden fehl, bleibt der Scanner unbewaffnet und der Messlauf ist ungültig.

Ein Compile-Fail-Test zeigt, dass `Scanner<Armed>` außerhalb von `contextra-erasure` nicht konstruierbar ist.

## 4. Doubles

Ein In-Memory-Double speichert, was der Test geschrieben hat, und entfernt es in `purge`, wenn es als Schicht-Double auftritt. `witness` liest den Speicher und meldet `residue_matches`. Ein leerer Speicher ohne vorherigen Canary-Fund ist kein bestandener Lauf.

`FaultVfs` implementiert `Vfs` aus `contextra-durable-fs`, einschließlich `scrub_if_safe` und `enqueue_pending_delete`. Ein injizierter Fehler am Journal-Flush muss beim Aufrufer `JournalNotDurable` ergeben, ohne dass der Double eine Lesesperre setzt.

## 5. Layering-Gate

`crates/contextra-testkit/tests/layering.rs` nutzt `cargo metadata` und prüft SPEC §15:

1. Jedes Crate hat `default = []`.
2. Keine E0- oder E1-Crate hängt von E2 oder E3 ab.
3. Die Domänen-Closure jeder E1-Crate ist eine Teilmenge von E0 und hat höchstens fünf Mitglieder. `contextra-ports` und `contextra-erasure` zählen nicht mit.
4. `[lints] workspace = true` ist gesetzt.
5. Kein Crate aus SPEC §2.2 ist Workspace-Member.
6. Die Gates `surface-coverage`, `claim-generator` und `deletion-honesty` sind im CI-Auftrag referenziert.

`contextra-store` darf in diesem Test nicht von `contextra-crypto` oder `contextra-mvcc` abhängen. `contextra-vector` darf nicht von `contextra-mvcc` abhängen. `contextra-ports` darf nicht von `contextra-types` abhängen.

## 6. `Cargo.toml`

```toml
[package]
name = "contextra-testkit"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Erasure lab, fault injection, and the layering gate"

[lints]
workspace = true

[dependencies]
contextra-ids = { workspace = true }
contextra-env = { workspace = true }
contextra-types = { workspace = true }
contextra-ports = { workspace = true }
contextra-erasure = { workspace = true }
contextra-durable-fs = { workspace = true }
rand = { workspace = true }
thiserror = { workspace = true }

[features]
default = []
```

## 7. Tests

| Kennung | Pflicht |
|---|---|
| T-DEL-7 | Positivkontrolle bewaffnet den Scanner. Ein fehlender Canary bewaffnet ihn nicht |
| INV-WITNESS-MINT-1 | `Scanner::<Armed>::new` ist außerhalb von `contextra-erasure` ein Compile-Fehler |
| T-DEL-23 | Eine Konstante anstelle der Messung lässt den Lab-Test fehlschlagen |
| §15 | `layering.rs` bricht ab, wenn `ports` von `types` abhängt oder `store` von `crypto` abhängt |

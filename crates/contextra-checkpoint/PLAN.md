# contextra-checkpoint — Crate-Spezifikation (Soll-Zustand, SPEC 2.1)

**Ebene:** E1 (Domäne)
**Domänen-Closure:** `ids`, `env`, `durable-fs`
**Verträge:** `erasure` (`CheckpointLayer`). Zählt nicht gegen die fünf (SPEC §15).
**Status:** Normativ (Soll-Zustand)
**Version:** 2.1
**Geltungsbereich:** `crates/contextra-checkpoint`
**Normative Quellen:** `docs/SPEC.md` §5.3 Schritt 8, §5.9 Schicht S6, §5.11, §6.3, §7, INV-12.

---

## 1. Zweck

`contextra-checkpoint` erzeugt Point-in-Time-Abbilder über Hardlinks auf unveränderliche SSTables und eine kopierte Manifest-Datei.

Schicht **S6**. `CheckpointLayer` implementiert `ErasureLayer` aus `contextra-erasure`. Die Messung heißt `CheckpointScanner`.

Ein Checkpoint ist eine Datenkopie im Sinn von Spur D (SPEC §5.11). Er enthält Chiffretext. Er enthält keine `KeyTable`, keinen Keystore, kein `master.salt` und keinen Wrap-Blob. Ein Lauf, der eine dieser Dateien verlinkt oder kopiert, endet mit `CheckpointError::KeyMaterialRefused`.

Die Vernichtung eines `DocDek` macht den Chiffretext in jedem Hardlink unlesbar. Das ersetzt den Sweep nicht. Hardlinks teilen die Blöcke der Quelldatei nicht mehr, sobald die Quelle neu geschrieben wurde. Der Sweep muss jeden Checkpoint selbst bereinigen (SPEC §5.3 Schritt 8).

## 2. Architekturregeln

1. `#![forbid(unsafe_code)]`. Zero-Panic. `default = []`.
2. Keine Abhängigkeit auf `contextra-store`, `contextra-crypto`, `contextra-mvcc` oder ein anderes E1-Crate. Keine Dev-Abhängigkeit auf `contextra-testkit`.
3. Kein `unwrap` und kein `expect` in Beispielen und Bibliothekscode. Metadaten-Fehler sind `Result`.
4. Logs nennen keine Nutzbytes des Löschziels (INV-12).

## 3. Anlegen

`create_checkpoint` legt das Zielverzeichnis an, verlinkt Dateien mit der Endung `sst` und kopiert das Manifest. Die Kopie des Manifests wird mit `fsync` abgeschlossen, danach das Verzeichnis.

`allow_copy_fallback` darf eine SSTable nur dann kopieren, wenn `hard_link` mit einem Fehler scheitert, der kein bereits geschriebenes Ziel hinterlässt. Die Kopie ist Chiffretext derselben Bytes. Ein Fallback, der entschlüsselt, existiert nicht.

Dateinamen aus der Schlüsselverwaltung (`keytable`, `keystore`, `master.salt` und ihre temporären Geschwister) werden nicht übernommen.

Verwaiste Zielverzeichnisse nach einem Crash löscht der `OrphanCleaner` nur, wenn kein Manifest-`fsync` abgeschlossen wurde. Ein abgeschlossener Checkpoint bleibt stehen, bis der Sweep ihn erfasst.

## 4. Policies (SPEC §5.9 S6)

| Policy | Wirkung |
|---|---|
| `Rewrite` | Standard. Jeder Checkpoint, der den Scope enthält, wird ohne den Scope neu geschrieben und ausgetauscht. |
| `ExpireNow` | Der Checkpoint wird gelöscht, sobald kein Leser ihn mehr pinnt. |
| `ExpireAfterHold` | Nur zulässig, solange die Saga `Restricted` ist. Nach `Lifted` oder nach dem Wechsel auf `Frozen` gilt wieder `Rewrite`. |

`ExpireAfterHold` auf einer Saga, die nicht `Restricted` ist, liefert `CheckpointError::PolicyNotApplicable`. Ein Checkpoint gilt erst nach abgeschlossenem `Rewrite` oder nach nachgewiesenem `ExpireNow` als bereinigt. Bis dahin bleibt er ein Residuum im Bericht. Die Klasse M vergibt der Koordinator.

`purge` ist idempotent (T-DEL-14).

## 5. Sweep und Restore

Schritt 8 ruft `CheckpointLayer::purge` unter der Policy der Collection auf. Der Austausch eines neu geschriebenen Checkpoints läuft über `contextra-durable-fs`: `tmp` → `fsync` → `rename` → Verzeichnis-`fsync`. Ersetzte Dateien, die eine offene Zeitreise noch pinnt, gehen in `PendingDelete`.

Restore ist an das aktuelle `ReadGate` gebunden (SPEC §6.3):

1. Vor dem ersten Lesen eines wiederhergestellten Bytes ruft der Restore `ReadGate::check` für jeden Scope des Checkpoints auf.
2. `Erased` verweigert die Wiederherstellung dieses Scopes. Die übrigen Scopes dürfen nur gelesen werden, wenn der Restore-Guard in `contextra-db` `restore_reapplied` geschrieben hat.
3. Ein Checkpoint wird nicht als Quelle einer `KeyTable` eingespielt.

Der Guard selbst liegt in `contextra-db` und `contextra-erasure` (SPEC §5.11). Dieses Crate stellt die Dateien bereit und verweigert den Lesezugriff, wenn das Gate `Erased` oder `Restricted` ohne Audit sagt.

## 6. Fehler (`CheckpointError`)

`#[non_exhaustive]`: `DestinationAlreadyExists`, `HardlinkFailed`, `Io`, `DurableFs`, `KeyMaterialRefused`, `PolicyNotApplicable`, `RestoreBlocked`, `PendingDeleteBlocked`.

## 7. `Cargo.toml`

```toml
[package]
name = "contextra-checkpoint"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Hardlink checkpoints and erasure sweep under Rewrite, ExpireNow, ExpireAfterHold"

[lints]
workspace = true

[dependencies]
contextra-ids = { workspace = true }
contextra-env = { workspace = true }
contextra-durable-fs = { workspace = true }
contextra-erasure = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }

[features]
default = []
std = ["contextra-ids/std", "contextra-env/std"]
```

## 8. Tests

| Kennung | Pflicht |
|---|---|
| T-DEL-14 | Zweites `purge` derselben Policy ist ein No-op |
| T-DEL-17 | Nach `Rewrite` enthält die Checkpoint-Datei den Klartext-Canary nicht |
| T-DEL-18 | Restore eines Checkpoints von vor der Löschung bleibt am Gate gesperrt |
| S6 | Ein Lauf, der `keytable` verlinken würde, liefert `KeyMaterialRefused` |
| Policy | `ExpireAfterHold` außerhalb von `Restricted` liefert `PolicyNotApplicable` |

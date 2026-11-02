# Contextra – Implementierungsspezifikation je Crate (Index)

Dieses Dokument ist die Einstiegsseite der Implementierungsspezifikationen. Es gibt je Crate eine eigene Datei (Tabelle in Abschnitt 4).

## 1. Geltung und Lesart

|Punkt|Festlegung|
|---|---|
|Status|**Abgeleitetes Dokument** im Sinne von D-98. Normativ ist ausschließlich `docs/SPEC.md` (Version 4.3.0). Bei Widerspruch gilt die SPEC; der Widerspruch ist ein Defekt dieses Dokuments.|
|Basis Soll|`docs/SPEC.md` 4.3.0, Soll-Zustand für Release 1.0.0|
|Basis Ist|`main` @ `1454bbec0` (Merge PR #5520, SIMD-VNNI-Rückbau). Die SPEC selbst stützt sich auf `4e208c179`; Abweichungen sind in den Crate-Dateien als Ist-Stand vermerkt.|
|Prüfmodus Ist|Quelltext gelesen, Manifeste ausgewertet, Muster gezählt (**[V]**). **Nicht gebaut, nicht getestet:** Im Analysecontainer gibt es keine Rust-Toolchain. Build-, Test- und Benchmarkwerte stammen aus der SPEC (3.0.0, **[M]**) und sind nicht wiederholt.|
|Marke **[SPEC x.y / D-nn]**|Vorgabe ist wörtlich aus der SPEC abgeleitet; der Anker nennt den Abschnitt oder die Entscheidung.|
|Marke **[E]**|**Entwurf dieses Dokuments.** Die SPEC lässt die Form offen (Modulschnitt, Rust-Signatur, Hilfstyp). Ein [E] ist ein Vorschlag, keine Anforderung. Wer ihn ändert, ändert nur dieses Dokument; wer eine SPEC-Aussage ändern will, ändert zuerst die SPEC (Anhang H).|
|Marke **[Lücke]**|Die SPEC ist an dieser Stelle widersprüchlich, unvollständig oder nicht umsetzbar. Das Dokument benennt den Punkt, schlägt eine Auflösung vor und führt ihn in Abschnitt 7 (alle Lücken gesammelt).|
|Wortlaut|Dieses Dokument vermeidet die Begriffe aus SPEC 6.4 (Wortlaut-Lint T-LINT-1, T-LINT-3). Löschaussagen nach außen bleiben allein die Außenaussage aus SPEC 6.4.|

## 2. Ringe, Reihenfolge, Kanten

```
Ring 4  contextra
Ring 3  contextra-erasure   contextra-engine
Ring 2  contextra-mvcc  contextra-store  contextra-text  contextra-vector  contextra-rank
Ring 1  contextra-sys  contextra-simd  contextra-durable-fs  contextra-crypto
Ring 0  contextra-types  contextra-ports
dev     contextra-testkit  contextra-lab  ctx-verify  tests/arch
```

**Erlaubte Kanten (SPEC 5.3, mit D-88, D-95, D-97):**

|von|nach (Produktionsabhängigkeiten)|
|---|---|
|`types`|keine Workspace-Crates|
|`ports`|`types`|
|`sys`, `simd`|`types`|
|`durable-fs`|`types`, `ports`, `sys`|
|`crypto`|`types`, `ports`|
|`mvcc`|`types`, `ports`|
|`store`|`types`, `ports`, `sys`, `crypto`, `mvcc`|
|`text`|`types`, `ports`, `crypto`|
|`vector`|`types`, `ports`, `simd`, `crypto` (**nicht** `sys`, **nicht** `mvcc`)|
|`rank`|`types`, `ports`|
|`erasure`|`types`, `ports`, `crypto` (**keine** Ring-2-Crates, AR-3)|
|`engine`|alle Produktions-Crates; `sys` nur für Prozesshärtung (D-88)|
|`contextra`|`engine`, `durable-fs`, `types`|
|`lab`, `ctx-verify`|**keine** Produktions-Decoder, nicht `store`, `engine`, `erasure` (AR-4)|

**Umsetzungsreihenfolge (SPEC 15.3):** A → B → G0 → C → D → F → E → G → H; G-LEGAL läuft parallel. Kritischer Pfad: WP-1.x → WP-2.1 → WP-3.1..3.4 (G0) → WP-4.x → G-D.

## 3. Querschnittsregeln für jedes Crate

Diese Regeln gelten für alle Crate-Dateien und werden dort nicht wiederholt, nur referenziert.

|ID|Regel|Quelle|Prüfung|
|---|---|---|---|
|Q-01|`//!`-Kopf in `lib.rs`: Zweck-Satz (wortgleich mit `Cargo.toml` `description` und README-Zeile 1), Ring, erlaubte Abhängigkeiten, SPEC-Abschnitt. Keine handgepflegten Verbrauchslisten.|D-91, D-97, 5.1a|T-ARCH-5, T-DOC-1|
|Q-02|`#![forbid(unsafe_code)]` in allen Crates außer `sys` und `simd`.|D-29, AR-8|T-UNSAFE-1|
|Q-03|Kein `unwrap`, `expect`, `panic!`, `unreachable!`, `todo!`, `let _ =` auf `Result`, kein `unwrap_or_default` auf I/O, keine Indexierung per `[]` im Produktionscode (`indexing_slicing = deny`). Sinkender Ratchet, Ziel 0 in Kern-Crates ab G-C.|P6, 5.8|Clippy `-D warnings`, `governance/ratchet.toml`|
|Q-04|Dateizugriff nur über das Trait `Vfs` aus `ports`. Kein `std::fs`, kein `tokio::fs`, kein `memmap2` außerhalb von `contextra-durable-fs`.|D-04, D-94|T-LINT-2|
|Q-05|Zeit, Zufall, IDs nur über `Clock`, `Rng`, `IdGen`. Ausnahme: Schlüssel-, Salt- und Nonce-Material aus dem OS-CSPRNG.|P7|Clippy `disallowed_methods`, Review|
|Q-06|Kein Feature und kein Laufzeitparameter schwächt Verschlüsselung, Integrität, Löschmessung oder Mandantenprüfung. Jedes `cfg(feature = …)` ist im `[features]`-Abschnitt deklariert; kein `#[allow(unexpected_cfgs)]`.|P5, D-05, HR-13, AR-9|T-CFG-1, T-DEF-1|
|Q-07|Jeder Test trägt `#[ctx_test(id = "T-…")]`, jede Benchmark `#[ctx_bench(id = "BM-…")]`.|15.1|T-TRACE-1|
|Q-08|Namensregeln: kein Modul `misc`, `util`, `utils`, `common`, `helpers`, kein Suffix `_impl`; kein Modul gleich dem Crate-Namen; **eine** Modulform (`foo.rs` neben `foo/`, kein `mod.rs`); keine Paketaliase.|D-90|T-ARCH-7|
|Q-09|Keine Zeitstempel („STAND“), Sitzungs- oder Agentenkennungen, `ANCHOR[…]`, `FILE-CONTEXT`, absolute Entwicklerpfade in Quelltext und Dokumenten.|D-91|T-LINT-4|
|Q-10|Fehler- und Lognachrichten enthalten nie Inhalte, Nutzerkennungen, Schlüssel oder personenbezogene Pfade. Feste Fehlertexte, `tracing` mit festen Ereigniscodes.|HR-4, I-11|T-LOG-1, T-REPORT-1|
|Q-11|`fsync`-Fehler sind für die betroffene Datei fatal; nie Wiederholung auf demselben Handle.|D-55, I-10|T-FSYNC-1|
|Q-12|`Cargo.lock` wird im selben Commit wie jede Manifest-Änderung erneuert; Prüfungen laufen mit `--locked`.|D-93|T-BUILD-1|
|Q-13|Neuer `unsafe`-Block, neue Abhängigkeit oder neues Feature nur mit Nennung des belegenden Tests (PR-Vorlage).|HR-12|`cargo-deny`, PR-Vorlage|
|Q-14|Rot-vor-Grün: Pro Arbeitspaket zuerst ein fehlschlagender Test mit SPEC-ID, dann die Implementierung; Evidenzbündel je WP.|D-44, P10|PR-Prüfung|
|Q-15|Öffentliche Typen `#[non_exhaustive]` (außer Konfigurationsbausteine); einzige öffentliche Oberfläche ist `contextra`.|D-66|T-API-1, `cargo-semver-checks`|
|Q-16|Alle Byte-Layouts sind big-endian (Anhang F). Jeder Header trägt `format_version` und `alg_ids`; unbekannte Werte ⇒ `Corrupt`.|D-42, I-6|T-FMT-1|

## 4. Crate-Dateien

|Nr.|Datei|Crate|Ring|Stufe|Wichtigste WP|Hauptgates|Ist (Zeilen `src`)|
|---|---|---|---|---|---|---|---|
|01|`01-contextra-types.md`|`contextra-types`|0|S0|WP-3.1|G-B, G0|7.304|
|02|`02-contextra-ports.md`|`contextra-ports`|0|S0|WP-1.1, WP-1.4, WP-3.1|G-B, G0|2.668|
|03|`03-contextra-sys.md`|`contextra-sys`|1|S0|WP-1.2, WP-0.3, WP-5.2|G-B, G-C, G-F|2.035|
|04|`04-contextra-simd.md`|`contextra-simd`|1|S2|WP-8.1|G-G|2.797|
|05|`05-contextra-durable-fs.md`|`contextra-durable-fs`|1|S0|WP-1.4, WP-5.2|G-B, G-F|1.677|
|06|`06-contextra-crypto.md`|`contextra-crypto`|1|S0|WP-3.2|G-F, G-C|3.260|
|07|`07-contextra-mvcc.md`|`contextra-mvcc`|2|S0|WP-3.3|G-C|4.195|
|08|`08-contextra-store.md`|`contextra-store`|2|S0|WP-0.3, WP-1.4, WP-3.3, WP-3.4|G-A, G-C, G0, G-D|26.996|
|09|`09-contextra-text.md`|`contextra-text`|2|S1|WP-7.1|G-E|7.151|
|10|`10-contextra-vector.md`|`contextra-vector`|2|S2|WP-8.1|G-G|13.624|
|11|`11-contextra-rank.md`|`contextra-rank`|2|S3|WP-8.2|G-G|3.440|
|12|`12-contextra-erasure.md`|`contextra-erasure`|3|S0|WP-3.1, WP-4.1..4.3|G-D|0|
|13|`13-contextra-engine.md`|`contextra-engine`|3|S0|WP-2.1, WP-5.1, WP-5.2|G-A, G-D|19.235|
|14|`14-contextra.md`|`contextra`|4|S0|WP-1.5|G-B, G-H|1.558|
|15|`15-contextra-testkit.md`|`contextra-testkit`|dev|S0|WP-0.1, WP-4.4|alle|3.347|
|16|`16-contextra-lab.md`|`contextra-lab`|dev|S0|WP-4.4|G-D|0 (fehlt)|
|17|`17-ctx-verify.md`|`ctx-verify`|dev/bin|S0|WP-4.3, WP-4.4|G-D|0 (fehlt)|
|18|`18-tests-arch.md`|`tests/arch`|dev|S0|WP-0.1, WP-1.5|G-B|823|

Die Stufen S0 bis S3 sind die Produktstufen aus SPEC 1.3.

## 5. Gate-Abdeckung (welche Crate-Datei liefert was)

|Gate|Bedingung (Kurz)|Beteiligte Dateien|
|---|---|---|
|G-A|`cargo check --workspace --all-targets --locked` und `fmt` grün|08, 13, 14, 02, 03|
|G-B|Rückbau nach D-84, Tokio-Features D-54, ein `Vfs`|01, 02, 03, 05, 08, 10, 13, 14, 18|
|G0|Speicherentscheid, Messprotokoll (BM-ERASE-WA, -WA-TEXT)|08, 01, 15|
|G-C|0 `unwrap`/`expect`/`panic`; Sentinel- und Delete-Race-Fix; Pflichtverschlüsselung|06, 07, 08, 13|
|G-D|Löschkern S0: T-ISO, T-GATE, T-VAULT, T-KEYCOV, T-FK, T-CRASH, T-RAW, T-PROOF, T-RESTORE; Lab MK1–MK3; Mutationstests|12, 16, 17, 08, 13, 15|
|G-E|Textindex S1|09|
|G-F|Keyslots, HEADER, Plattformprobe|06, 05, 03, 13|
|G-G|Vektorindex S2|10, 04, 11|
|G-H|Release|alle|

## 6. Abweichungen der bestehenden `docs/IMPLEMENTATION_SPEC_PER_CRATE.md`

Die vorhandene Datei im Repository nennt sich „SOLL-Zustand“ und bezieht sich auf **SPEC v2.0.0**. Sie widerspricht der SPEC 4.3.0 an mindestens diesen Stellen; die Dateien dieses Pakets ersetzen sie (Aktualisierung nach D-98 empfohlen):

|Altdatei|Widerspruch zur SPEC 4.3.0|SPEC-Anker|
|---|---|---|
|Abschnitt 1: `Region` mit `Loc=0x02`|Die Region `LOC` entfällt, Code 0x02 bleibt reserviert.|D-70, D-31, 9.1|
|Abschnitt 1: `TxId(u64)`, `Seq(NonZeroU64)`, `SecretBytes` als Löschtyp-Umfeld; `ErasureReport` als Typ in `types`|Löschtypen laut 8.1: `ErasureId`, `EraseTarget`, `LayerWitness`, `ResidualRisk`, `KeyCoverage`; Bericht ist kanonisches JSON (F.10) im Löschkern.|8.1, 11.7, F.10|
|Abschnitt 2: `ErasureLayer` mit `plan`, `purge`, `scan_residue`, `positive_control`|Soll: `id`, `resolve`, `on_gate`, `purge`, `witness`, `key_coverage`; Positivkontrolle ist typisiert (`Scanner<Armed>`).|8.2, D-72, I-19|
|Abschnitt 6: `MIN_M_COST_KIB >= 65536`, `t >= 3`, `p = 1`|Grenzen sind **Bereiche**: `m ∈ [64 MiB, 4 GiB]`, `t ∈ [3, 16]`, `p ∈ [1, 16]`; Standard m = 128 MiB, t = 3, p = 4.|D-06, D-68|
|Abschnitt 6: „Nonce- und AAD-Bindung“, kein Schlüsselhierarchie-Bezug|Dateischlüssel zufällig (F-Slot), Nonce-Schema D-60, Keyslots D-52, `K_sign` nur abgeleitet.|D-52, D-60, D-80, D-85|
|Abschnitt 7: `prune_through` mit `TOMBSTONE_BIT`|Der Sentinel steckt in `min_read_snapshot` (`u64::MAX`); Soll ist `Option`, ohne „alles sichtbar“-Wert. Tombstones sind `vlen = 0` (F.5).|8.7, I-12, F.5|
|Abschnitt 8: `drain()`, `rewrite_file()`, WAL-Dateien „entfernen“|Purge-Primitive: `drain_range`, `rewrite_without`, `seal_wal_at`, `drop_wal_until`, `unlink_verified`. WAL wird **nie** umgeschrieben.|8.8, D-73|
|Abschnitt 12: Saga `Requested → Gated → KeyDestroyed → Drained → Rewritten → Unlinked → IndexesCleaned → Measured → Completed`; Prüfungen „E1/E2/E3“|Saga: `Requested → Gated → KeyDestroyed → Purging → Verifying → Completed` (+ `Pending`, `Refused`). Messklassen heißen **MK1–MK3**; „E1–E3“ ist ein verbotener Altbegriff (T-LINT-3).|11.2, 12.1, D-72|
|Abschnitt 12: „Dokument-DEK-Salts“|Schlüsselhierarchie mit Slots `T`, `C`, `S`, `D`, `F`; Löschen = ein Slot der höchsten Ebene, O(1).|D-79, D-33, 11.5|
|Abschnitt 14: „Pflichtparameter `passphrase`, `erase_sla`, `runtime`, `vfs`, `clock`, `rng`“; `erasure_checklist()` aus „§13.3“|Zusätzlich Pflicht: `data_dir`, `backup_retention`. Löschcheckliste steht in SPEC **16**, nicht 13.3.|5.7, D-74, 16|
|Abschnitt 15: `T-ARCH-1..3` und `T-LINT-1..2`|Zusätzlich T-ARCH-4..8, T-LINT-3..4, T-DOC-1, T-CFG-*, T-BUILD-1.|15.1, D-91..D-98|
|Kopf: „Ring 3: contextra-erasure, contextra-engine“ ohne `lab`, `ctx-verify`|`lab` und `ctx-verify` sind eigene Mitglieder (dev) mit Abhängigkeitsverbot zu Produktions-Decodern.|5.1, AR-4|

## 7. Gefundene Lücken und Unstimmigkeiten in der SPEC (zur Klärung durch den SPEC-Eigentümer)

Bei der Ableitung sind folgende Punkte aufgefallen. Sie blockieren die Arbeit nicht, jede Datei nennt ihre vorläufige Auflösung **[E]**. Empfehlung: Klärung vor Beginn von WP-3.1 und Aufnahme in Anhang H.

|Nr.|Fundstelle|Beobachtung|Vorschlag [E]|Betrifft|
|---|---|---|---|---|
|L-1|8.2 vs. Ist-`Vfs`; D-03|D-03 verlangt für `Vfs` dyn-fähige `BoxFuture`-Methoden. Das Ist-Trait `Vfs`/`VfsFile` ist **synchron** (`std::io::Result`). `RealVfs` müsste async werden, ohne `tokio::fs` (D-54 verbietet Feature `fs`).|`Vfs` async per `BoxFuture`; `RealVfs` führt blockierende Aufrufe über `tokio::task::spawn_blocking` (Feature `rt-multi-thread` genügt).|02, 05, 08, 10|
|L-2|D-03 vs. Ist-`StorageEngine`|Soll: Heißpfad-Ports statisch dispatcht (`async fn` im Trait). Ist: `StorageEngine` ist per `BoxFuture` dyn-fähig und trägt den Kommentar „retained for backward compatibility“.|Umstellung auf `async fn`/`-> impl Future + Send`; Engine generisch.|02, 08, 13|
|L-3|9.4/D-79 vs. 10.1|DEK-Salt: 9.4 und D-79 bilden `SHA-256("ctx/dek-salt/v2" ‖ salt_T ‖ salt_C ‖ salt_S ‖ salt_D)`; 10.1 zeigt `SHA-256(salt_T‖salt_C‖salt_S‖salt_D)` ohne Label.|Maßgeblich D-79/9.4 (Label `ctx/dek-salt/v2`); 10.1 angleichen.|06, 12|
|L-4|F.3|Slot-Offset `128 + 64 + i·80` passt nicht zu „Kopf 64 B + Metablock 64 B“ (Slots begännen bei 128). Außerdem ist ein 80-Byte-Slot kein Vielfaches der 64-Byte-Granularität der Atomaritätsprobe (D-56); Slots können Blockgrenzen überschreiten.|Eine Konstante `VAULT_SLOT_BASE` an **einer** Stelle, Wert per Golden-Datei festlegen; Slot-Vernichtung als zwei Schreibvorgänge mit Verifikation (wie F.3 beschreibt) und Probe auf 80-Byte-Teilschreibvorgänge erweitern.|05, 12, 16, 17|
|L-5|8.1/8.2, 12.2, I-19|`LayerWitness` soll in `types` liegen, nur aus `Scanner<Armed>` konstruierbar, `Scanner<Armed>` nur nach erfolgreicher Positivkontrolle. Ein privater Konstruktor ist über Crate-Grenzen (types ↔ erasure) nicht durchsetzbar.|Typzustandsmuster vollständig in `types` (`Scanner<Unarmed>` → `arm(ControlEvidence)` → `Scanner<Armed>` → `finish(matches)` → `LayerWitness` nur bei `matches == 0`); Aufrufstellen von `arm`/`ControlEvidence::observed` per `tests/arch` auf `erasure::measure` beschränken; Lab-Mutationstest T-MUT-1 belegt Wirksamkeit.|01, 12, 18|
|L-6|15.1, T-TRACE-1|Das Attribut `#[ctx_test(id = …)]` ist ein Attributmakro. Es braucht ein Crate mit `proc-macro = true`; 5.1 führt kein solches Crate.|Hilfs-Crate `contextra-testmacros` (dev) in 5.1 und 5.3 aufnehmen, oder Verfahren per Anhang H ändern.|15, 18|
|L-7|5.4, D-57, D-78|Commit-Barriere (`RwLock`) wird vom Store-Leader (shared) und von `ErasedSet::add_batch` in `erasure` (exklusiv) benutzt; `purge_waiting` von Kompaktion (`store`) und Purge. `erasure` darf `store` nicht kennen (AR-3). Die Port-Liste in 8.2 enthält keinen Port dafür.|Ports `CommitBarrier` und `PurgeBarrier` in `ports`, implementiert in `store`, über `engine` an `erasure` gereicht.|02, 08, 12, 13|
|L-8|9.1|Region `META` (Collection-Katalog, Zähler) hat kein definiertes Schlüsselformat.|Gleicher 78-Byte-Rahmen; `cluster_tag = 0¹⁶`, `doc_tag` = `name_tag`; Familie `Manifest`; per Golden-Datei festschreiben.|01, 08, 13|
|L-9|I-9, 5.5|Der Store-Leader muss „Tresor-Slots + `sync_data`“ vor den WAL-Frames ausführen, kennt den Tresor aber nur über Ports (`RecordSealer`/`FileKeyring`).|Port-Methode `make_slots_durable()` auf `RecordSealer`/`FileKeyring`, vom Leader aufgerufen.|02, 08, 12|
|L-10|7.2 vs. 7.5|`search_vector` kann `Dimension` liefern; das Fehlermodell nennt `Dimension` nicht.|`Limit::Dimension` oder `Config::Dimension` festlegen.|01, 13, 14|
|L-11|8.2 `ErasureLayer::witness`|`witness` bekommt nur `LayerScope` und `Scanner<Armed>`, braucht aber Dateizugriff und für MK1 die Entschlüsselungsfunktion der Schicht.|Schicht hält `Arc<dyn Vfs>` und `Arc<dyn FileKeyring>` aus der Konstruktion; `Scanner` bleibt datenfrei (Zähler).|08, 09, 10, 12|
|L-12|F.4 vs. D-73|`WalFloor` trägt `removed_file_ids`; die Frage, ob die Fortsetzung der Kette über **mehrere** Segmente (mehrere `WalFloor` hintereinander) die Gleichung `prev_mac = tail_mac` erhält, ist nur für den Einzelfall beschrieben.|Recovery prüft Gleichheit gegen den **letzten** `WalFloor`; Test T-WAL-2 mit mehreren Floors.|08|

## 8. Definition of Done je Crate

Ein Crate gilt als abgenommen, wenn alle Punkte erfüllt sind:

1. Alle `Q-*`-Regeln dieses Index erfüllt; `cargo check/clippy/fmt/test --locked` für das Crate grün.
2. Alle in der Crate-Datei in Abschnitt „Tests und Benchmarks“ genannten IDs sind als `#[ctx_test]`/`#[ctx_bench]` vorhanden und grün (T-TRACE-1).
3. Alle Rückbaupositionen aus Abschnitt „Rückbau“ der Crate-Datei sind geschlossen (Grep-Nachweis im PR).
4. Crate-Kopf, `Cargo.toml`-Beschreibung und README-Zeile 1 sind wortgleich (T-ARCH-5, T-DOC-1).
5. Die Abhängigkeiten entsprechen der Matrix 5.3 (T-ARCH-3, T-ARCH-6).
6. Evidenzbündel des Arbeitspakets (roter Lauf, grüner Lauf, Protokoll) liegt dem PR bei (D-44).

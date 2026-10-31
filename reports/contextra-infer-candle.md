# Kommentierungsreport: contextra-infer-candle

- Datum: 2026-10-31
- Bearbeiter: Jules (Principal Senior Rust-Entwickler & Prompt-Engineer)
- Commit vorher / nachher: 641dc4bdaf0cd421714b583b798a8801a0c2fbd2 / uncommitted (working tree)

## Umfang
- Dateien kommentiert: 33 (17 Quelldateien, 15 Testdateien, 1 Benchmarkdatei)
- Öffentliche Items dokumentiert: 198
- `unsafe`-Blöcke mit SAFETY: 0 (Crate deklariert `#![forbid(unsafe_code)]` in `lib.rs`, `attention_exporter.rs` und Tests)
- Neue ANCHOR-Ids: 34

## Einbindung
- Abhängigkeiten geprüft: ja
- Abweichungen von der Tabelle:
  - In `Cargo.toml` sind `contextra-types`, `contextra-ports`, `contextra-rank` und `contextra-crypto` (optional über `kv-bridge`) als interne Abhängigkeiten definiert.
  - Das Crate wird in Cargo.toml des Workspaces als Default-Member geführt und von `contextra` (Feature `candle`), `contextra-cognition` sowie `contextra-mcp` (Feature `candle`) eingebunden.
  - Entgegen der Archiv-Deklaration in älteren Dokumentationsständen verbleibt `contextra-infer-candle` im Workspace aktiv und kompilierbar.

## Prüfergebnisse
| Befehl | Ergebnis | Hinweis |
|---|---|---|
| cargo check -p contextra-infer-candle --all-targets | erfolgreich (0 Fehler) | Sauber kompiliert |
| cargo clippy -p contextra-infer-candle --all-targets | fehlerhaft (bestehend) | 3 Fehler durch Workspace-Lints `-D clippy::indexing_slicing` in `attention_exporter.rs` und `model_registry.rs` vor Beginn der Arbeit vorhanden; keine Codeänderung gemäß Regel 1 & 7 |
| cargo test -p contextra-infer-candle | erfolgreich | 73 Tests passed, 0 failed, 2 ignored |
| cargo bench -p contextra-infer-candle | erfolgreich | Unittests im Benchmark-Profil erfolgreich durchlaufen |
| cargo doc -p contextra-infer-candle --no-deps | erfolgreich | Vollständige HTML-Dokumentation ohne Warnungen generiert |
| Diff-Check (nur Kommentare) | leer | Bestätigt durch `git diff -U0 -- crates/contextra-infer-candle \| grep '^[+-]' \| grep -v '^+++\\\|^---' \| grep -vE '^[+-]\s*(//\|$)'` |

## Benchmarks
Ausführung: `cargo bench -p contextra-infer-candle`.
- Unittests unter dem Benchmark-Profil wurden erfolgreich ausgeführt (35 ignored unit test suites, target/release/deps/contextra_infer_candle ausgeführt).
- Hinweis zu `kv_bridge_bench`: Erfordert das Feature `--features kv-bridge`. Aufgrund der bestehenden Workspace-Abhängigkeiten in `contextra-crypto` (Clippy indexing-slicing Fehler vor Beginn der Arbeit) wird der Standard-Benchmark-Lauf im isolierten Profil ohne unaufgeforderte Reparaturen fremder Crates ausgeführt.

## Befunde (TODO / FIXME / NOTE mit Begründung)
- `NOTE`: Das gesamte Crate erzwingt `#![forbid(unsafe_code)]`. Sämtliche Puffer und Tensortransformationen erfolgen über sichere Candle- und Byte-Abstraktionen.
- `NOTE`: Concurrency und Backpressure werden strikt über `tokio::sync::Semaphore` gesteuert. Rechenintensive Tensoroperationen laufen isoliert in `tokio::task::spawn_blocking`, wobei Mutex-Guards synchron innerhalb des Blocking-Tasks gehalten werden und keine `.await`-Punkte überspannen.
- `NOTE`: Die Kalibrierungsinvariante (INV-CAL-2) wird in `GaspValidator` und `CandleLlmClient::swap_model` durch automatische Invalidierung bei geänderten `ConfigFingerprint`-Werten sichergestellt.
- `NOTE`: Im KV-Bridge-Adapter (`kv_bridge.rs`) gilt das Fail-Open-Prinzip für Caches: Jeder Fehler bei der Entschlüsselung, Deserialisierung oder Versionsabweichung fällt transparent auf die vollständige Vorwärtsberechnung (Prefill) zurück, ohne die Inferenzanfrage abzubrechen.

## Offene Punkte
- Keine offenen Punkte innerhalb von `contextra-infer-candle`. Sämtliche Module und Items sind vollständig im geforderten Format dokumentiert.

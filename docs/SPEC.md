# Contextra – Gesamtspezifikation 4.1 (Konsolidierung auf Ist-Stand)

| Feld | Wert |
|---|---|
| Stand | 31.10.2026 |
| Geprüfter Commit | `3a0b11699` (`origin/main`, „RÜCKSTELLUNG MEISTE CRATES“) |
| Ersetzt | Fassung 4.0 (30.10.2026, Reduktionsfassung). Vollständiger Text in `docs/spec/archiv/SPEC-4.0.md` |
| Vorgänger | Fassung 3.1 (Baukasten-Fassung). Vollständiger Text in `docs/spec/archiv/SPEC-3.1.md` |
| Status | Normativ für Build-Vertrag, Ringmodell und Inventar. Scope-Entscheidungen sind als offen markiert (§8) |
| Belegbasis | Statische Analyse von `Cargo.toml`, `Cargo.lock`, `capabilities.toml`, Quelltext und Git-Historie. **Kein Build, kein Testlauf.** |

---

## 1. Geltung und Hierarchie

1. Diese Datei ist die normative Quelle für Scope, Build-Vertrag und Ringmodell.
2. `capabilities.toml` ist die normative Quelle für Ring, Reifegrad und erlaubte Abhängigkeiten je Crate. Bei Widerspruch zur Spezifikation gilt die Datei, die später geändert wurde, und der Widerspruch wird im Änderungsprotokoll (§10) vermerkt.
3. Archivierte Fassungen (3.1, 4.0) sind nicht bindend. Sie dienen der Nachvollziehbarkeit.
4. Die README beschreibt den Ist-Stand und verweist hierher. Sie enthält keine eigenen Versprechen, die nicht in dieser Datei stehen.

## 2. Befund zum Ist-Stand

Am Commit `3a0b11699` gilt:

| Aussage | Befund | Quelle |
|---|---|---|
| Repo ist auf 14 Crates reduziert | **Falsch.** 35 Crate-Verzeichnisse, 34 Einträge in `capabilities.toml` | `ls crates/`, `capabilities.toml` |
| Workspace ist konsistent | **Falsch.** `xtask` ist Workspace-Mitglied, das Verzeichnis fehlt. Laut `CLAUDE.md` wurde der Gate-Runner `cargo xtask` mit Commit `068261abd` entfernt, die Gates in `governance/gates.toml` sind nicht ausführbar. Der Workspace-Eintrag ist ein Rest dieser Entfernung | `Cargo.toml`, `ls`, `CLAUDE.md` |
| Alle Feature-Kombinationen bauen | **Falsch.** Die Features `onnx` und `adaptive-candidate-pool-sizing` sind leere Feature-Flags ohne aktivierte Abhängigkeit. Der zugehörige Code referenziert `contextra_infer_onnx` und `contextra_adapt`, die nicht deklariert sind. Default-Build ist davon nicht betroffen | `crates/contextra-engine/Cargo.toml`, `src/contextra_impl/lifecycle.rs:188`, `src/collection/query_builder/builder.rs:45` |
| Ring-Modell wird eingehalten (deklarierte Abhängigkeiten) | **Gilt für 34 Crates.** 0 Verletzungen. `contextra-durable-fs` hat keinen Ring-Eintrag und ist nicht geprüft. Die Prüfung umfasst nur `[dependencies]`, nicht die Feature-Verweise aus dem Befund oben | Auswertung `Cargo.toml` gegen `capabilities.toml` |
| Build ist grün | **Nicht geprüft.** README sagt „nicht grün“ | README, Abschnitt Status |

Die Gegenüberstellung von Versionen mit Messung steht in `docs/analysis/BEFUNDE_2026-10-31.md`.

## 3. Produkt und Scope

### 3.1 Kern (unverändert, Fassung 4.0 §2)

Eingebettete Such- und Speicherbibliothek für lokale RAG-Anwendungen. Hybride Suche aus HNSW-Vektoren und BM25-Volltext mit Rangfusion. Mandantentrennung. Verschlüsselung im Ruhezustand, in jedem Build aktiv.

### 3.2 Offene Scope-Entscheidung D-K1

Der Commit `3a0b11699` setzt die Reduktion von 4.0 teilweise zurück. Die Entscheidung über den Umfang ist nicht getroffen. Zwei Optionen, Details in ADR 0009:

- **Option A, Kern v1:** Nur die 14 Crates aus Fassung 4.0 §5.1. Die übrigen 21 Crates werden in `archive/`-Tags verschoben und nicht gelöscht.
- **Option B, Vollumfang:** Alle 35 Crates bleiben im Workspace. Jede Fähigkeit ist über `capabilities.toml` mit Reifegrad versehen. Die Löschkette erhält Status `experimental` und ist nicht als Garantie beworben.

Bis zur Entscheidung gilt: Kein Crate wird gelöscht. Keine Aussage über Löschgarantien wird öffentlich gemacht.

### 3.3 Löschkette

Die Löschkette (Fassung 3.1 §9) ist in Fassung 4.0 aus dem Scope genommen. Die Crates `contextra-store`, `contextra-crypto` (Löschnachweis) und `contextra-engine` (Erasure-Module) enthalten weiterhin entsprechenden Code. Eine Wiederaufnahme als Garantie erfordert einen neuen ADR mit eigenem Scope (D-R5 aus 4.0, übernommen als D-K4).

Die ADRs 0001 bis 0008 sind historisch. Sie gelten nicht als bindend, bis ein neuer ADR sie bestätigt.

## 4. Build-Vertrag

Ein Stand ist nur dann als lauffähig zu bezeichnen, wenn alle folgenden Prüfungen grün sind:

| Nr. | Prüfung | Befehl | Status am Commit |
|---|---|---|---|
| B1 | Workspace-Manifest auflösbar | `cargo metadata --locked --format-version 1 > /dev/null` | Nicht geprüft. Erwartet Fehler wegen `xtask` |
| B2 | Default-Build | `cargo build --workspace --locked` | Nicht geprüft |
| B3 | Alle Features | `cargo build --workspace --locked --all-features` | Erwartet Fehler (`onnx`, `adaptive-candidate-pool-sizing`) |
| B4 | Keine Features | `cargo build --workspace --locked --no-default-features` | Nicht geprüft |
| B5 | Tests | `cargo test --workspace --locked` | Nicht geprüft. 4.729 Testfunktionen gezählt |
| B6 | Lints | `cargo clippy --workspace --locked -- -D warnings` | Nicht geprüft |
| B7 | Format | `cargo fmt --all -- --check` | Nicht geprüft |
| B8 | Toolchain | `rust-toolchain.toml`: 1.89.0 | Festgelegt, nicht installiert in Analyseumgebung |

Ein Feature-Flag ohne aktivierte Abhängigkeit, auf das Code verweist, gilt als Build-Defekt und nicht als Konfigurationsstil. Regel R-B1: Jedes `#[cfg(feature = "x")]`-Symbol, das eine externe Crate nennt, muss in `Cargo.toml` über `dep:` oder ein optionales Paket aktiviert werden.

## 5. Architektur-Invarianten

| Nr. | Invariante | Prüfung | Status |
|---|---|---|---|
| A1 | Crate in Ring N hängt nur von Ringen ≤ N ab | Auswertung `[dependencies]` gegen `capabilities.toml` | Erfüllt für 34 Crates (0 Verletzungen). `contextra-durable-fs` ungeprüft (ohne Eintrag) |
| A2 | Ring 0 enthält kein `tokio` unter `[dependencies]` | `grep` in Ring-0-Manifesten | Erfüllt (nur `dev-dependencies` in `text`) |
| A3 | Ring-0-Crates führen kein Dateisystem-I/O aus | Suche nach `std::fs`, `OpenOptions`, `File::` in `src/` | **Verletzt** in `contextra-vector` (28 Treffer) |
| A4 | Feature-Verweise auf Crates sind deklariert | siehe R-B1 | **Verletzt** (`onnx`, `adaptive-candidate-pool-sizing`) |
| A5 | `unsafe` nur in deklarierten Unsafe-Inseln | `capabilities.toml`: `unsafe_island = true` | Teilweise. `simd` (271 Treffer), `wire` (46), `sys` (21), `infer-candle` (2). `contextra-engine` hat keine Insel deklariert, aber 0 Treffer |
| A6 | Composition Root nur in der Fassade `contextra` | Quelltext-Muster `builder(` | Gilt laut README. Fassade hängt von 12 Crates ab |

Hinweis zu A3: Die Verletzung in `vector` ist bekannt und in Fassung 4.0 nicht als Befund erfasst. Sie ist in PLAN K2 eingeplant.

## 6. Sicherheitsinvarianten

| Nr. | Invariante | Stand |
|---|---|---|
| S1 | Verschlüsselung im Ruhezustand ist standardmäßig aktiv | Laut README, nicht durch Test am Commit bestätigt |
| S2 | Algorithmen: AES-256-GCM-SIV, HMAC für Integrität | Laut Fassung 3.1 §4, im Code `crypto` vorhanden |
| S3 | Keine eigenen kryptografischen Primitive | Eingehalten laut Crate-Beschreibungen |
| S4 | Legacy-Schlüsselpfade (`LegacyActive`, WAL V1) sind nicht beworben | Offen. `LegacyActive` liegt in `contextra-store/src/wal/hmac.rs`. Ob WAL V1 schreibbar bleibt, ist ungeprüft |
| S5 | Löschnachweise werden nur als Garantie beworben, wenn die Löschkette Reifegrad `stable` hat | Erfüllt: Reifegrad `experimental` |

## 7. Crate-Inventar

Die vollständige Tabelle mit Ring, Reifegrad und internen Abhängigkeiten steht in `docs/ARCHITECTURE.md` §3. Diese Spezifikation nennt keine eigenen Zahlen, um Doppelpflege zu vermeiden.

## 8. Entscheidungen

| Nr. | Entscheidung | Status |
|---|---|---|
| D-K1 | Scope: Option A (Kern v1) oder B (Vollumfang) | **Offen**, ADR 0009 |
| D-K2 | Build-Vertrag B1 bis B8 ist Voraussetzung für jeden Merge auf `main` | Vorgeschlagen |
| D-K3 | Feature-Flags ohne deklarierte Abhängigkeit sind Build-Defekt (R-B1) | Vorgeschlagen |
| D-K4 | Löschkette bleibt `experimental`, bis ein ADR den Scope festlegt | Übernommen aus 4.0 (D-R5) |
| D-K5 | `xtask` wird wiederhergestellt oder aus dem Workspace entfernt | **Offen**, PLAN K0.1 |
| D-K6 | README, SPEC, PLAN, ARCHITECTURE werden bei jeder Änderung an Crate-Liste oder Workspace synchron gehalten | Vorgeschlagen |

## 9. Qualitätsregeln

1. Kein Merge mit grünen Tests, aber rotem Build auf einer anderen Feature-Kombination.
2. Kein Crate-Verzeichnis ohne Eintrag in `capabilities.toml` und `Cargo.toml`.
3. Jede Zahl in der Dokumentation ist am Commit reproduzierbar. Befehle stehen in `docs/analysis/BEFUNDE_2026-10-31.md`.
4. Aussagen zu Stabilität oder Sicherheit verweisen auf einen Test, eine ADR oder eine Messung.

## 10. Änderungsprotokoll

| Fassung | Datum | Änderung |
|---|---|---|
| 3.1 | 28.10.2026 | Baukasten-Fassung, 33 Crates. Archiv: `spec/archiv/SPEC-3.1.md` |
| 4.0 | 30.10.2026 | Reduktionsfassung, Kern v1 mit 14 Crates. Archiv: `spec/archiv/SPEC-4.0.md` |
| 4.1 | 31.10.2026 | Konsolidierung auf Commit `3a0b11699`. Korrektur: 35 Crates am Ist-Stand, `xtask` fehlt, Feature-Defekte A4, Verletzung A3, Scope D-K1 offen. Keine Build- oder Testergebnisse |

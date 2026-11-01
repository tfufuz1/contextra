# AGENTS.md — `contextra` (Fassade)

> **Normative Vorgabe gemäß [docs/SPEC.md §8.14](file:///home/freddy/Projekte/contextra/docs/SPEC.md) und [PLAN.md](file:///home/freddy/Projekte/contextra/PLAN.md)**

## 1. Rolle & Zweck
`contextra` ist die einzige öffentliche Fassade und Composition Root von Contextra. Sie re-exportiert ausschließlich die autorisierte öffentliche API (SPEC §7) für Endanwender und kapselt die Assemblierung der internen Ringe 0 bis 3.

## 2. Ring, Abhängigkeiten & Verbote
- **Ring:** Ring 4 (Fassade, Composition Root)
- **Erlaubte Abhängigkeiten:** `contextra-types`, `contextra-ports`, `contextra-crypto`, `contextra-durable-fs`, `contextra-engine`.
- **Verbotene Abhängigkeiten:**
  - Kein Crate im Workspace darf von `contextra` abhängen (Invariante AR-1).
  - Keine Abhängigkeit zu externen Netzwerk-, Server- oder ML-Frameworks.
- **Verbotene Muster:**
  - **Strikt kein `unsafe`** (`#![forbid(unsafe_code)]`).
  - Keine Implementierungsdetails oder interne Traits in der öffentlichen API sichtbar.
  - Kein unvollständiger `EngineBuilder` ohne Passphrase oder Tokio-Handle.

## 3. Öffentliche API & Module (Soll-Zustand laut SPEC §7, §8.14)

- **`EngineBuilder`:**
  - Konfiguriert und öffnet die Datenbank:
    ```rust
    let db = EngineBuilder::new(data_dir)
        .with_passphrase(passphrase) // Pflicht! Argon2id KDF
        .with_runtime_handle(tokio_handle)
        .open()
        .await?;
    ```
  - `system_defaults()` stellt `RealVfs`, `SystemClock` und den CSPRNG `OsRng` bereit.
- **Collections & Dokumente:**
  - `db.collection("name")` liefert `Collection` mit `insert`, `get`, `scan`, `delete`, `search_hybrid`.
- **Löschen & Verifikation (SPEC §7.3):**
  - `db.erase(target, retention)` stößt die Löschung an und liefert `ErasureReceipt` mit Status `Gated`.
  - `db.erasure_status(erasure_id)` fragt den Fortschritt der Saga ab.
  - `contextra::verify(report, target)` prüft einen signierten Löschbericht unabhängig.

> [!IMPORTANT]
> **Bereinigung veralteter Module & Features (SPEC §8.14):**
> Entfernen aller Abhängigkeiten und Features zu `license`, `privacy`, `audit-export`, `avv-generator`, `infer-candle` sowie veralteter Profile.

## 4. Kritische Invarianten
1. **Sicherheit per Default (D-05, D-07):** Verschlüsselung ist standardmäßig aktiv und erfordert eine Passphrase; Mandantentrennung ist verpflichtend.
2. **Zero-Panic-Doktrin:** Alle öffentlichen Funktionen liefern `Result<T, ContextraError>`; kein Aufruf von `unwrap()` oder `panic!`.
3. **Keine Blockierung in Drop:** Das Beenden erfolgt asynchron und kooperativ über `db.close().await?`.

## 5. Crate-Arbeitspakete zum SOLL-Zustand (PLAN.md)

- **[ ] WP-1.1 & WP-1.2: Veraltete Fassaden-Features & Crates entfernen**
  - Streichen der Features `audit-export`, `avv-generator`, `candle` und Bereinigung von `Cargo.toml`.
- **[ ] WP-2.5: Sicherheitsdefaults im `EngineBuilder` erzwingen (D-05, D-06)**
  - Passphrase mit Argon2id-Ableitung als zwingenden Konstruktor-Parameter etablieren.
- **[ ] WP-3.11: Öffentliche `erase()`- und `erasure_status()`-API bereitstellen**
  - Integration der Löschauftrags-Schnittstelle in `Collection` und `Contextra`.
- **[ ] WP-5.3: Öffentliche `verify()`-API bereitstellen**
  - Bereitstellung der unabhängigen Berichts- und Residuenprüfung.
- **[ ] WP-7.4: Release-Candidate v1.0.0 Abnahme**
  - Überprüfung aller Doc-Tests und Beispiele.

## 6. Verifikation & Beispiele
- **T-API-1:** `trybuild`-Tests zur Absicherung der stabilen öffentlichen API-Signatur.
- **Beispielprogramme im Repository:**
  - `examples/quickstart.rs`
  - `examples/collections.rs`
  - `examples/hybrid_search.rs`
  - `examples/erase_and_verify.rs` (neu für Lösch- und Nachweispfad)
  - `examples/governance_fixtures.rs`

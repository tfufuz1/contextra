# AGENTS.md — `contextra` (Fassade)

> **Normative Vorgabe gemäß [docs/SPEC.md §8.14](file:///home/freddy/Projekte/contextra/docs/SPEC.md) und [PLAN.md](file:///home/freddy/Projekte/contextra/PLAN.md)**

## 1. Rolle & Zweck
`contextra` ist die einzige öffentliche Fassade und Composition Root von Contextra. Sie re-exportiert ausschließlich die autorisierte öffentliche API (SPEC §7) für Endanwender und kapselt die Assemblierung der internen Ringe 0 bis 3.

## 2. Ring, Abhängigkeiten & Verbote
- **Ring:** Ring 4 (Fassade, Composition Root)
- **Erlaubte Abhängigkeiten:** `contextra-types`, `contextra-ports`, `contextra-crypto`, `contextra-store`, `contextra-engine`, `contextra-rank`.
- **Verbotene Abhängigkeiten:**
  - Kein Crate im Workspace darf von `contextra` abhängen (Invariante AR-1).
  - Keine Abhängigkeit zu externen Netzwerk-, Server- oder ML-Frameworks im Produktionsbaum.
- **Verbotene Muster:**
  - **Strikt kein `unsafe`** (`#![forbid(unsafe_code)]`).
  - Keine Implementierungsdetails oder interne Traits in der öffentlichen API sichtbar.
  - Keine Rechtsbehauptungen in Dokumentation oder Fehlermeldungen (sachliche, messbare Beschreibungen nachprüfbarer Eigenschaften).

## 3. Öffentliche API & Module (Soll-Zustand laut SPEC §7, §8.14)

- **`ContextraBuilder` / `builder`:**
  - Konfiguriert und öffnet die Datenbank:
    ```rust
    let db = builder(768)
        .with_storage_path(data_dir)
        .with_encryption_passphrase(passphrase) // Pflicht gemäß D-05
        .build()
        .await?;
    ```
  - `OsRngAdapter` stellt den kryptografisch sicheren CSPRNG aus Betriebssystem-Entropie bereit.
- **Collections & Dokumente:**
  - `db.collection("name")` liefert `Collection` mit `insert`, `get`, `scan`, `delete`.
- **Löschen & Verifikation (SPEC §7.3):**
  - Physikalische Löschung und Nachweisprüfung erfolgen über die entsprechenden Engine- und Store-Schnittstellen.

## 4. Kritische Invarianten
1. **Sicherheit per Default (D-05):** Verschlüsselung ist standardmäßig aktiv und erfordert eine Passphrase; unverschlüsselter Betrieb wird abgewiesen.
2. **Zero-Panic-Doktrin:** Alle öffentlichen Funktionen liefern `Result<T, ContextraError>`; kein Aufruf von `unwrap()` oder `panic!`.
3. **Keine Blockierung in Drop:** Das Beenden erfolgt kooperativ und ressourcenschonend über `db.close().await?`.

## 5. Verifikation & Tests
- **T-API-1:** `trybuild`-Tests zur Absicherung der stabilen öffentlichen API-Signatur.

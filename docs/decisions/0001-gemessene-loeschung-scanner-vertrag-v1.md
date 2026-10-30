# ADR 0001: Gemessene Löschung, Scanner-Vertrag V1, Ausschlüsse

> **Status-Update 30.10.2026:** Ausgesetzt (D-R5). Die Scanner-Verträge gehören zur entfernten Löschkette.

- **Status**: Akzeptiert
- **Datum**: 2026-10-28
- **Kontext**: Welle 4 - Löschkette W4-P10, SPEC §12.7 ("Erledigung & Konsolidierung der gemessenen Löschung")

## Kontext und Problemstellung
In bisherigen Versionen existierten in `contextra-crypto` deprecated Konstruktoren (`LayerCleanupProof::new_after_verified_empty` und `verify_and_create`), die manuell deklarierte Schichtlöschbeweise ohne verifizierte Messung erlaubten. Dies verletzte die Invariante **INV-CLAIMS-1** und **INV-WITNESS-MINT-1**, da Löschnachweise ohne gemessenen Befund erzeugt werden konnten.

## Entscheidungen

1. **Vollständiges Entfernen veralteter Konstruktoren**:
   - Die Funktionen `LayerCleanupProof::new_after_verified_empty` und `verify_and_create` wurden ersatzlos gestrichen.
   - `LayerCleanupProof` wird nur noch als Typ für Bincode-Kompatibilität alter Formate vorgehalten.

2. **Scanner-Vertrag V1 & Unabhängige Messung**:
   - Gültige Löschnachweise werden ausschließlich über `DeletionProof::issue` unter Verwendung von fälschungssicheren `LayerWitness`-Messungen erzeugt, die in `ScanSession::finish` gemessen wurden.
   - Versucht ein Aufrufer, einen Nachweis mit verbliebenen Restdaten (`remaining > 0`) oder ohne gemessenen Befund (`ScanMethod::NotMeasured`) auszustellen, schlägt `DeletionProof::issue` fail-closed mit `Err` fehl.

3. **Integritäts-Gates in CI / xtask**:
   - Gate `no-dummy-proof`: Blockiert Test-Schlüssel (`default_test_deletion_proof_key`) sowie `new_after_verified_empty` und `verify_and_create` im Produktionscode.
   - Gate `deletion-honesty`: Stellt sicher, dass MCP/Tool-Antworten den Schlüssel `"proof"` niemals ohne echten Aufruf von `erase_*` / `drop_collection` zurückgeben.
   - Gate `scanner-coverage`: Erzwingt `LayerScanner`-Implementierungen für alle Pflicht-Schichten (`LsmMemtable`, `SsTableAllLevels`, `WalAllSegments`).

4. **Transparenz über Ausschlüsse (`ExcludedScope`)**:
   - Speicherschichten außerhalb des physischen Storage-Zugriffs (wie Fine-Tuning LLM-Zusammenfassungen `ConsolidatedAndDistilled` und LLM-Parameter `LlmParameterMemory`) werden explizit als `ExcludedScope` ausgewiesen und niemals als "100% gelöscht" behauptet.

## Konsequenzen
- **Positiv**: Löschnachweise basieren ausnahmslos auf echten, unabhängigen Nachlöschungsmessungen. Stubs und Fakes im Produktionscode werden durch automatisierte Gates ausgeschlossen.
- **Negativ**: Tests und Beispiele müssen mit `contextra-testkit` oder echten Engine-Löschungen arbeiten, um valide Witnesses zu erhalten.

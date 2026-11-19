# contextra-erasure — Implementierungsplan zum Soll-Zustand

**Normative Basis:** `docs/SPEC.md` §§5–7, 10–11 und `docs/crates/contextra-erasure.md`. **Ausgangslage:** 80+ Module überlappen mehrfach (chain/saga/journal/gate/vault); 42 Nicht-Test-Panikmuster verhindern eine beweisbare, kanonische Löschsaga.

1. Führe die Module auf `saga`, `journal`, `gate`, `vault`, `measure`, `report`, `recovery`, `restricted` und `error` zurück; definiere einen einzigen Eigentümer je State, Record und Gate.
2. Implementiere die neun durablen Schritte exakt: Intent+Attestation, Gate, logische Kaskade, A-Messung/Positivkontrolle, idempotentes Key-Shredding, B-Messung, Epoch-Seal, terminierte Hygiene, signierter Report. `Completed`/`Refused` sind terminal, ein Gate nie reversibel.
3. Stelle `ReadGate` als O(1), fail-closed, mit Dekker-Fence zwischen Journal und LiveMask bereit; unterstütze Subject/Doc/Collection/Derived Targets und Single-Flight.
4. Kanonisiere `ErasureLayer`, Witness und Fehlerpolitik: Schlüssel-/Gate-Fehler stoppen vor Erfolgsaussage; Messung und Report beschreiben Stufe/Grenzen, niemals pauschale Compliance.
5. Entferne Altketten/Kompatibilitätsbrücken erst nach Recovery-Migration; ersetze jede Panic-Stelle und zeroize Vault-/Keymaterial.

**Abnahme:** T-DEL-1–13, Crash an jedem Seal, Idempotenz-/Parallelitäts-/Fence-Tests, Canary-Positivkontrolle und Report-Signaturprüfung.

# AGENTS – contextra-erasure

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md). Bei Widerspruch gilt die SPEC.

**Zweck:** Löschkern: Saga, Gate, Journal, Tresor, Messung, signierter Bericht.

**Ring und Abhängigkeiten:** Ring 3. Erlaubt: `types`, `ports`, `crypto` (AR-3).

**Verboten:** Abhängigkeiten zu `store`, `text`, `vector`, `engine` (AR-3); `lab`- oder `ctx-verify`-Code im Produktionscrate (AR-4); Löschung ohne Zeugen (P1); Berichte mit Inhalten, Schlüsseln oder Klartext-Kennungen (P13)

**Entscheidungen:** D-01, D-13, D-33, D-35, D-37, D-39, D-48, D-55, D-57, D-58, D-61, D-72, D-73, D-74, D-78, D-79, D-85, D-96, D-98

**Tests:** T-GATE-1, T-GATEGROUP-1, T-VAULT-1 bis T-VAULT-5, T-KEYCOV-1, T-FK-1 bis T-FK-3, T-RAW-1 bis T-RAW-6, T-PROOF-1, T-RESTORE-1 bis T-RESTORE-3, T-MUT-1, T-CRASH-*, T-RESTR-1, T-CLASS-1, T-JRN-3

**Befunde zu prüfen:** F-33, F-04

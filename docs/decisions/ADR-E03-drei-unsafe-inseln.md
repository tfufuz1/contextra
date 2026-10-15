# ADR-E03: Genau drei Unsafe-Inseln: `sys`, `simd`, `wire`. `kvcache` b

* **Status:** Akzeptiert (aus Spezifikation §13 E03)
* **Datum:** 2026-10-15

## Entscheidung
Genau drei Unsafe-Inseln: `sys`, `simd`, `wire`. `kvcache` bleibt `forbid(unsafe_code)`; ausgerichtete Allokation als sichere Fassade in `sys`.

## Begründung
Auditierbarkeit; kein `unsafe` in Ring 1.

## Verworfen
`unsafe` in `kvcache`/`vector`/`store`

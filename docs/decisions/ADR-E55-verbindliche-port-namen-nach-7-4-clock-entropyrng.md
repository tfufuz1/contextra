# ADR-E55: Verbindliche Port-Namen nach §7.4: `Clock`, `EntropyRng`, `S

* **Status:** Akzeptiert (aus Spezifikation §13 E55)
* **Datum:** 2026-10-15

## Entscheidung
Verbindliche Port-Namen nach §7.4: `Clock`, `EntropyRng`, `SeededRng`, `IdGenerator`, `SystemRng` (Impl). `Rng` entfällt.

## Begründung
Der Begriff `Rng` ist im Code mehrdeutig (Entropie gegen Test). Die Trennung ist sicherheitsrelevant (§5.13).

## Verworfen
Generischer Name `Rng`

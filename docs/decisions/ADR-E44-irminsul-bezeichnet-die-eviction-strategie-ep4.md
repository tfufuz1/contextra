# ADR-E44: `Irminsul` bezeichnet die Eviction-Strategie (EP4). Position

* **Status:** Akzeptiert (aus Spezifikation §13 E44)
* **Datum:** 2026-10-15

## Entscheidung
`Irminsul` bezeichnet die Eviction-Strategie (EP4). Positionsunabhängiges Caching ist Kandidat `PIC`, nur mandantengebunden und nur nach ADR.

## Begründung
Derselbe Name steht in ADVANCED §1 und in der Recherche für Verschiedenes. Mandantenübergreifende Treffer verletzen §9.4 (Timing-Seitenkanal).

## Verworfen
Mandantenübergreifender Inhalts-Cache (Recherche Feld M)

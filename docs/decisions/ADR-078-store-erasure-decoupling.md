# ADR-078: Entkopplung der Löschlogik aus contextra-store und Port-Ausnahme in scanner/

## Status
Akzeptiert (SPEC §6.1, E-04, DEBT-01, WP-STO-1)

## Kontext
Im Zuge der Transformation des Contextra-Systems in einen modularen Baukasten (SPEC v4.1, E-04) müssen datenhaltende Persistenz- und Storage-Kerne von rechtlichen, regulatorischen und Lösch-Saga-Domänenlogiken befreit werden.
Das Modul  enthielt Lösch- und Crypto-Shredding-Logik, die von keinem internen Modul des Storage-Kerns genutzt wurde, sondern fachlich an  gebunden war.

Zudem existierten in  Verweise auf  und .

## Entscheidungen
1. **Entfernung von  aus :**
   Das Modul  (, , ) wird vollständig aus  entfernt und in  angesiedelt, wo die zugehörigen kryptographischen Komponenten (, ) liegen.
   Die Tests  und  ziehen nach  um.

2. **Typisierte Fehlerbehandlung statt Panic:**
   Der bisherige Fallback  in  wird durch eine typisierte Fehlerrückgabe () ersetzt (Zero-Panic-Doktrin).

3. **Neutralisierung von :**
   Die Methode  wird als neutrale Schnittstelle eingeführt. Die alte Signatur  bleibt für eine Übergangsphase als -Weiterleitung erhalten und wird in  (DEBT-01) mit Ablauf-Meilenstein M8 / v0.2.0 geführt.

4. **Ausnahme für Port-Namen in :**
   Die Port-Typen in  (, , ) verbleiben temporär in , bis das dedizierte Adapter-Crate  angelegt wird (SPEC §6.3, E-04). Dies stellt die einzige dokumentierte Ausnahme dar.

5. **Keine -Importe im Store:**
   Sämtliche Importe von  im Store-Crate (Produktionscode und Tests) werden eliminiert.

## Konsequenzen
-  ist frei von -Abhängigkeiten und verbotenen Löschbegriffen.
-  hostet  und  autark.
- Kein Funktionsverlust (Funktionsinventar-Diff belegt Konsistenz).

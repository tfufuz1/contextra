---
name: acta-spec-check
description: "Prüft Änderungen am Acta-Quelltext, an Dokumentation oder Oberflächentexten gegen docs/SPEC.md, insbesondere Wortfilter (SPEC 1.6, Liste A und Regel B), Hub-Regeln und Zusagen. Verwenden vor jedem Abschluss einer Code- oder Textänderung und bei Fragen, ob eine Formulierung oder ein Designentscheid zulässig ist."
---

# SPEC-Prüfung für Änderungen

## 1. Wortfilter (G-WORD)

Die folgenden Formulierungen sind in Quelltext, Dokumentation, Berichten und Oberflächen verboten (Groß-/Kleinschreibung ignoriert):

- DSGVO-konform
- DSGVO-zertifiziert
- erfüllt Art. 17 automatisch
- 100 % gelöscht
- garantiert gelöscht
- garantiert vernichtet
- unwiederbringlich
- vergisst Daten
- revisionssicher
- fehlerfrei
- löscht aus allen Backups
- perfekte Antworten

Außerdem verboten: jede Behauptung einer Zertifizierung oder behördlichen Billigung.

Suche:
```
rg -i -n "dsgvo-konform|dsgvo-zertifiziert|erfüllt art. 17 automatisch|100 % gelöscht|garantiert gelöscht|garantiert vernichtet|unwiederbringlich|vergisst daten|revisionssicher|fehlerfrei|löscht aus allen backups|perfekte antworten" crates xtask
```

**Regel B** (nur Berichtsvorlagen, Oberflächentexte, Exportvorlagen): Das Wort „gelöscht“ steht nur zusammen mit der erreichten Erfüllungsklasse, etwa „Klasse K erreicht“. Berichte geben nur `reached` oder `not_reached` aus und weisen die Annahmen A-FLASH, A-RAM, A-ANCHOR, A-CRYPTO, A-EXT, A-INFER, A-CLOCK, A-DETECT, A-ARCH und A-FS aus (SPEC 20.3).

Die Gate-Prüfung deckt Liste A ab. Regel B ist nur über Berichtsvorlagen und Oberflächentexte zu prüfen. Das Gate prüft sie noch nicht. Also manuell kontrollieren.

## 2. Architektur

Vor jeder neuen Abhängigkeit prüfen:
- Level-Regel: `level(A) > level(B)` (SPEC 3.2, `rings.toml`).
- HR-1 bis HR-9 (SPEC 3.4), insbesondere HR-6 (keine Alias-Re-Exports, kein `include!`) und HR-9 (`acta-seg` kennt keine Schlüssel).
- Whitelist SPEC 3.8. Neue Crates brauchen ADR-Eintrag.
- Synchroner Kern: kein `tokio` unterhalb Level 8.

## 3. Lebenszyklen und Zusagen

- Eine neue Funktion muss in SPEC Kap. 19 einen Lebenszyklus haben (L01 bis L17). Ohne Eintrag darf sie nicht implementiert werden.
- Zusagen Z1 bis Z19 (SPEC 1.5) betreffen Verhalten, nicht Implementierungsdetails. Eine Änderung, die eine Zusage verletzt, ist ein Defekt. Die Zusage ist maßgeblich (SPEC 0.4 Regel 2).
- Ein Fehlerpfad muss die Klasse aus `ErrorClass` tragen (`Transient`, `Integrity`, `Crypto`, `Erased`, `Policy`, `Invalid`).
- `Display` und `code()` von Fehlern dürfen keine Personendaten oder Roh-Kennungen enthalten (Z15).

## 4. Typsicherheit (SPEC 3.1 Prinzip 3)

- Schlüsseltypen ohne `Default`, `Clone`, `Debug`, `Serialize`.
- Versiegelte Daten als `Sealed<P>`, abgeleitete Daten als `Derived<T>`, geprüfte Eingaben als `Validated<T>`.
- Keine Geheimnisse in `Debug`-Ausgaben.

## 5. Ausgabe

Zu jeder Prüfung nennen:
- welche Regel geprüft wurde (Wortfilter, Level, HR, Lebenszyklus, Zusage),
- Dateipfad und Zeile bei Befund,
- ob der Befund behoben oder bewusst offen gelassen wurde (mit Begründung und ADR-Verweis).

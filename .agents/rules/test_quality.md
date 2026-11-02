# Testqualität

Abgeleitet aus SPEC 15.1, P10, D-35, D-72, HR-10.

- Jeder Test prüft eine benannte Eigenschaft mit SPEC-ID. Tests ohne Bezug werden nicht gemergt.
- Ein Test, der nur „kein Panic“ prüft, belegt keine Zusage. Die Assertion muss das erwartete Ergebnis angeben.
- Positivkontrolle: Ein Scanner-Zeuge ist nur gültig, wenn er ein eingestreutes Muster in einer Kontrolldatei findet (HR-10, T-RAW-2).
- Mutationstests auf `erasure`, Klassifikation, Matcher, Gate und Tresor. Überlebende Mutanten blockieren Gate G-D (T-MUT-1).
- Abdeckung: Zeilen ≥ 90 % in `erasure`, `crypto`, `store`, `ports`; ≥ 95 % in der Entscheidungslogik von `erasure` und der Klassifikation (SPEC 15.1). Der Wert steigt nur.
- Golden-Dateien je Format-Version (T-FMT-1, Anhang F).
- Fuzz-Ziele für alle Parser aus Anhang F und das KDF-Header-Format (T-KDF-2).

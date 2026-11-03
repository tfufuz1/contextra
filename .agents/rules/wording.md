# Wortlaut über Löschung

Abgeleitet aus SPEC 1.6, 6.4, 16, Anhang D, HR-5, E-L2, E-L13, T-LINT-1, T-LINT-3, T-DOC-1.

- **Einzige zulässige Außenaussage:** der Wortlaut in SPEC 6.4 (DE und EN). Er gilt für Quelltext, Rustdoc, Kommentare, Fehlermeldungen, Testnamen, README, Berichtsvorlagen und Commit-Texte.
- **Verbotene Begriffe** stehen in SPEC 6.4. Sie werden in Produktartefakten nicht zitiert, auch nicht verneint; das Wortlaut-Lint prüft alle Produktartefakte außer `docs/SPEC.md`.
- **Erlaubte Fachsprache:** Gate, Schlüsselvernichtung (mit `key_coverage` und Verweis auf den Bericht), Purge, Messklassen MK1 bis MK3, Evidenzstufen EV0 bis EV4, Klassen G/K/P, `field_measurement`, `lab_evidence`, Restrisiko.
- **Zeit- und Größenangaben** nur so, wie die SPEC sie trifft: O(1) gilt bis `Gated` (P16, D-30); der Purge skaliert mit dem Ziel (P11). Keine Aussagen wie „sofort gelöscht“ oder „unabhängig von der Systemlast“.
- **Rechtliche Wertung** trifft der Betreiber (E-L2). Contextra behauptet weder Löschung, Anonymisierung noch Einschränkung als Rechtsfolge.
- **Berichte** nennen, was gemessen wurde, und listen alle zutreffenden Restrisiken (Anhang D). Die Signatur beweist Integrität und Herkunft, nicht die Wahrheit des Inhalts (E-L11).
- **`soft_delete`** ist ein Tombstone; in der Doku steht an erster Stelle, dass für Löschpflichten `erase` zu verwenden ist (D-51).
- **Fehlt eine Schicht** im Code, steht sie im Bericht als „nicht erfasst“.

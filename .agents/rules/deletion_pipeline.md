# Lösch-Pipeline

Abgeleitet aus SPEC 1.5, 6.4, 11.1 bis 11.8, 13.1 (I-1 bis I-20), D-01, D-30, D-39, D-57, D-58, D-73, D-79, D-85.

## Saga

`Requested` → `Gated` → `KeyDestroyed` → `Purging` → `Verifying` → `Completed`, jederzeit `Pending(reason)`. Jeder Schritt ist idempotent und nach Neustart fortsetzbar (I-15).

## Reihenfolge bis `Gated` (4 `fsync`, D-58)

1. Validierung, `RetentionAttestation`, Journal `Requested` mit Umfang (fsync 1).
2. Gate-Eintrag in `gates.log` unter exklusiver Commit-Barriere (fsync 2).
3. Vektor-Surrogat und Cache-Eviction. Schlüssel der höchsten Ebene nullen (fsync 3).
4. Journal `KeyDestroyed` (fsync 4). Rückgabe `Gated`.

## Purge-Fenster

Snapshots des Mandanten widerrufen; Textsegmente neu schreiben; Memtable leeren und flushen; WAL-Segmente bis zum Flush-Punkt entfernen (nie umschreiben, D-73); SSTables ohne Zielrecords neu schreiben; Dateischlüssel nach dauerhaftem Manifest-Swap nullen, dann `unlink`; Vektor-Kanten reparieren; `D`-Slots entfernen.

## Regeln

- Nach `Gated` wird nichts mehr in das Ziel geschrieben (I-1). Lesen liefert `Erased` (I-2).
- `Completed` nur mit gültigen Zeugen aller Schichten (I-4, P1).
- Ein Zeuge ohne Positivkontrolle ist ungültig (HR-10).
- `fsync`-Fehler sind fatal für die Datei (D-55). Die Löschung bleibt `Pending`, bis der Neustart und die Prüfung abgeschlossen sind.
- Ein Test, der zeigt, dass ein Erase Daten außerhalb des Ziels verändert, ist Release-Blocker (HR-7).
- Der Bericht nennt, was gemessen wurde, und listet die zutreffenden Restrisiken (SPEC Anhang D). Außenaussagen nur nach SPEC 6.4.

# D-76 (ENTWURF, nicht freigegeben): Obergrenze Cluster-Rewrite / Write Amplification

**Status:** Entwurf. Bis zur Freigabe: nur messen, kein Gate (Auftrag §6).
**Befund B8:** `D-76` war im Repo nicht definiert.

## 1. Definition (Vorschlag)
- `R_max`: maximale Rewrite-Bytes je Cluster-Purge = `c_r * Σ Bytes betroffener Tabellen` (Startwert c_r = 1.5, Kalibrierung nach P0-Messung).
- `WA_max`: maximaler WA-Faktor je Zeitfenster `W` (Startwert W = 60 s, gleitend).
- `WA = (WAL + Flush + Compaction Bytes) / User-Bytes`, je Level und gesamt.

## 2. Messmethode
- Zaehler je Level in `stats.rs`: `wal_bytes`, `flush_bytes`, `compaction_read/write_bytes`, `user_bytes`.
- Fenster: Ringpuffer mit Delta-Differenzen, keine Locks im Hot-Path (Atomics).
- Messpunkte: nach fsync der jeweiligen Datei, nicht beim Schreibaufruf.

## 3. Verhalten bei Ueberschreitung (zur Entscheidung)
| Option | Wirkung | Risiko |
|---|---|---|
| A drosseln (`throttle.rs`) | Purge verlangsamt, endet spaeter | Loeschfrist (DSGVO) verlaengert sich |
| B abbrechen (`StoreError`) | Kein Teilzustand, Retry durch Aufrufer | Purge kann dauerhaft scheitern |
| C hybrid: drosseln bis `T_max`, dann abbrechen | Beides begrenzt | zwei Parameter |

Empfehlung: C. Loeschpflicht darf nicht still an der WA-Grenze haengen (INV-7: nur gemessene Stufe).

## 4. Offene Fragen an den Entscheider
1. Gilt `WA_max` pro Store oder pro Tenant?
2. Hat Purge Vorrang vor Foreground-Latenz (Δp99-Ziel 10 %)?
3. Wert fuer `T_max`.

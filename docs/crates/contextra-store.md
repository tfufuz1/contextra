# contextra-store — Crate-Spezifikation

**Ebene:** E1b (Kern-Closure: `ids`, `env`, `sys`, `durable-fs`, `crypto`)  
**Bindung:** `docs/SPEC.md` §2, §3, §4, §5, §6, §7

---

## 1 Zweck & Domäne

`contextra-store` ist die primäre Persistenz-Engine auf Basis einer Log-Structured Merge-Tree (LSM-Tree) Architektur:
- Write-Ahead Log (WAL) für synchrone Durability vor Bestätigung
- In-Memory MemTable und Immutable MemTables
- Persistente SSTables (SST) mit Block-Indexen und Bloom-Filtern
- Zweistufige Kompaktions-Engine zur Bereinigung und Konsolidierung

## 2 Verschlüsselung & Datenkapselung

- **Block-Level & Record-Level Encryption:** Payloads werden vor dem Schreiben in WAL und MemTable versiegelt (`AEAD` via `contextra-crypto`).
- **Kein Klartext auf der Platte (T-DEL-1):** SSTables und WAL-Logs enthalten ausschließlich Chiffretext für Payloads von K- und P-Klassen.
- **Header & Metadaten:** Schlüsselpräfixe enthalten keine personenbezogenen Klartext-Daten.

## 3 Löschkette & ErasureLayer-Implementierung

`contextra-store` implementiert `ErasureLayer` mit den Prioritäten 4c (WAL-Drain) und 4d (SSTable-Rewrite):

1. **`resolve(target)`:**
   - Ermittelt alle betroffenen SSTable-Dateien anhand der KeyRange-Metadaten und Bloom-Filter.
   - Prüft aktuelle WAL-Segmente.
2. **`on_gate(scope)`:**
   - Überträgt die Gate-Sequenz auf den Write-Controller. Neue Schreibvorgänge dürfen für betroffene Schlüssel nicht mehr angenommen werden.
3. **`purge(scope, cx)`:**
   - **Schritt 4c (WAL-Drain):** MemTable wird sofort als Immutable markiert und auf Platte geflusht. Der alte WAL-Bereich wird verworfen.
   - **Schritt 4d (SSTable-Rewrite / Kompaktion):** Führt ein gezieltes Rewrite der betroffenen SSTables unter Ausschluss der gelöschten Key-Ranges durch. Die neuen SSTables werden atomar umbenannt (`contextra-durable-fs`). Die alten SSTable-Dateien werden genullt und gelöscht.
4. **`witness(scope, scanner)`:**
   - Scannt physisch alle SSTables und WAL-Dateien im Datenverzeichnis mittels Positivkontrollen-Mustersuche.
   - Verifiziert: `residue_matches == 0`.

## 4 Koordination mit PurgeBarrier & ReadGate

- Alle Kompaktionsläufe müssen die `PurgeBarrier` beachten.
- Der Store-Lesepfad (`get`, `scan`) konsultiert vor der Entschlüsselung synchron den `ReadGate`. Liefert dieser `Erased`, bricht der Read sofort mit `KeyNotFound` ab (Fail-Closed, Z-E1).

## 5 Invarianten

- `#![forbid(unsafe_code)]`.
- WAL-fsync vor Schreibquittierung.
- Idempotente Wiederherstellung aus Crash-Zuständen.

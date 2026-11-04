# ADR-SL-1: Atomarer SSTable-Tausch im Manifest-Log

## Status
Vorschlag (Änderungsantrag zu SPEC 5.1.0 Anhang F.4, WP-W.3, Gate G0).

## Kontext
In SPEC 5.1.0 Anhang F.4 definiert das Manifest-Log (`kind = 3`) die Eintragstypen zur Verwaltung aktiver Dateien:
- `0x01 AddFile{file_id, kind, level(u8), lo(65), hi(65), min_seq, max_seq, size}`
- `0x02 RemoveFile{file_id}`
- `0x03 WalFloor{seq(u64), tail_mac(32), removed_file_ids}`
- `0x04 SeqMark{next_seq}`
- `0x05 Collection{coll_tag, name_tag, options_envelope}`
- `0x06 TextCommit{add, remove, floor}`

Für den Textindex existiert mit `TextCommit (0x06)` bereits ein atomarer Verbundtyp, der das Hinzufügen neuer Segmente, das Entfernen alter Segmente und das Nachziehen des Floors in einem einzigen Append-Eintrag bündelt. Für das LSM-Speichersystem (SSTables) kennt F.4 dagegen ausschließlich isolierte `AddFile`- und `RemoveFile`-Einträge.

Wenn während einer Kompaktion oder eines physischen Löschdurchlaufs (Purge-Rewrite betroffener SSTables gemäß SPEC 11.2 Schritt 5d und D-85) alte SSTables durch neu geschriebene SSTables ersetzt werden, müssen im Manifest die neuen Dateien registriert und die alten Dateien deregistriert werden. Werden hierfür zwei getrennte Manifest-Einträge (`AddFile` gefolgt von `RemoveFile`) geschrieben, führt ein Systemabsturz (Torn Write oder Stromausfall) zwischen den beiden Einträgen dazu, dass sowohl die neuen als auch die alten SSTable-Dateien im Manifest aktiv bleiben.

Während ein solcher Zustand bei regulärer Kompaktion lediglich redundante Daten und erhöhten Speicherverbrauch bewirkt, stellt er im Purge-Pfad einen kritischen Löschfehler dar: Die alte SSTable enthält noch unbereinigte Zielrecords des gelöschten Objekts. Bei einem Neustart liest die Engine weiterhin aus beiden Dateien, wodurch bereits als gelöscht deklarierte Daten wieder sichtbar werden oder im LSM-Baum verbleiben.

## Entscheidung (Vorschlag)
1. Es wird ein neuer, atomarer Manifest-Eintragstyp `0x07 SstSwap` in Anhang F.4 eingeführt:
   ```
   0x07 SstSwap {
       add: Vec<FileDescriptor>,
       remove: Vec<FileId>,
   }
   ```
   wobei `FileDescriptor` die Metadaten analog zu `AddFile` kapselt (`file_id`, `kind`, `level`, `lo`, `hi`, `min_seq`, `max_seq`, `size`).
2. Kompaktion und Purge-Rewrite (SPEC 11.2 Schritt 5d) MÜSSEN den Tausch von SSTables verbindlich über `0x07 SstSwap` als einzelnen atomaren Manifest-Eintrag abbilden.
3. Bis zur Verabschiedung und Implementierung dieses Vorschlags darf kein Purge-Rewrite über zwei getrennte Einträge als abgeschlossen gewertet werden.

## Alternativen
1. *Beibehaltung von zwei getrennten Einträgen (`AddFile` gefolgt von `RemoveFile`):*
   Verworfen. Verletzt die Atomaritätsgarantie bei Absturz. Ein Absturz nach `AddFile` lässt die veraltete Datei aktiv im Manifest, was im Löschpfad Zielrecords persistiert hinterlässt.
2. *Umgekehrte Reihenfolge (`RemoveFile` vor `AddFile`):*
   Verworfen. Bei einem Absturz nach `RemoveFile` und vor `AddFile` sind Daten verloren, die in der neuen Datei lagen, da die neue Datei noch nicht referenziert ist. Führt zu unbeabsichtigtem Datenverlust unbeteiligter Records.
3. *Zweistufiges Commit-Protokoll im Manifest (Prepare/Commit-Marker):*
   Verworfen. Erhöht die Protokollkomplexität, verdoppelt Fsync-Aufwände auf dem Manifest-Log und erfordert komplexe Rollback-Heuristiken während des Startvorgangs.

## Sicherheitsargument (Löschintegrität)
Gemäß SPEC 2 (Prinzipien P1, P11), SPEC 11.2 und D-85 dürfen nach Abschluss eines Purge-Laufs keine Reste gelöschter Schlüssel in aktiven SSTables verbleiben. Durch die atomare Ersetzung in einem einzigen Appendlog-Eintrag `0x07 SstSwap` mit deterministischer SHA-256-Kette (`chain = SHA-256(chain_prev ‖ Eintrag)`) wird verhindert, dass eine Zwischenphase beobachtet oder nach Recovery wirksam werden kann. Entweder ist der Tausch vollständig im Manifest persistiert (Altdateien deregistriert, Neudateien registriert) oder er ist nicht wirksam, woraufhin die Altdatei noch aktiv ist, aber der Purge-Saga-Schritt als nicht vollendet gilt und wiederholt werden muss.

## Vorsichtigere Lesart (Fail-Closed)
Bis zur Umsetzung von `0x07 SstSwap` gilt: Der LSM-Speicher darf einen SSTable-Tausch im Löschpfad nicht als zwei unkoordinierte Manifest-Einträge ausführen. Findet die Recovery ein Manifest vor, bei dem im Rahmen eines Purge-Laufs redundante SSTables derselben Sequenz- und Schlüsselbereiche vorliegen, MUSS die Engine den Start verweigern (`StorageHealth::Poisoned` oder Startabbruch), um das Wiederaufleben gelöschter Records zu verhindern.

## Betroffene SPEC-Stellen und Formulierungsvorschlag

### 1. SPEC Anhang F.4 (Manifest-Log):
Im Abschnitt **Manifest-Log (`kind = 3`)** wird die Liste der Typen ergänzt:
```diff
--- a/docs/SPEC.md
+++ b/docs/SPEC.md
@@ -F.4
 Typen: `0x01 AddFile{file_id, kind, level(u8), lo(65), hi(65), min_seq, max_seq, size}`,
 `0x02 RemoveFile{file_id}`, `0x03 WalFloor{seq(u64), tail_mac(32), removed_file_ids}`,
 `0x04 SeqMark{next_seq}`, `0x05 Collection{coll_tag, name_tag, options_envelope}`,
-`0x06 TextCommit{add, remove, floor}` (D-121 Abs. 7).
+`0x06 TextCommit{add, remove, floor}` (D-121 Abs. 7),
+`0x07 SstSwap{add: [FileDescriptor], remove: [file_id]}`.
```

### 2. SPEC Abschnitt 11.2 (Saga):
Schritt 5d wird wie folgt präzisiert:
„d. SSTable-Rewrite betroffener Dateien (Nicht-Ziel-Records als Envelopes übernommen, neuer Dateischlüssel); atomarer Manifest-Swap über `0x07 SstSwap`, danach `F`-Slots der Altdateien nullen, dann `unlink` (I-17, D-85).“

## Betroffene Komponenten und Tests
- `crates/contextra-store`: Manifest-Writer und Manifest-Replay müssen `SstSwap (0x07)` unterstützen.
- `crates/contextra-lab`: Unabhängiger Format-Parser (`contextra-lab/src/format/wal_manifest.rs`) muss Typ `0x07` parsen.
- Tests:
  - `T-FMT-1`: Formatverifikation für Manifest-Eintrag `0x07`.
  - `T-CRASH-2`: Crash-Injektion während SSTable-Kompaktion und Purge-Swap.
  - `T-CRASH-6`: Abbruch an jedem Teilschritt der SSTable-Ersetzung; Verifikation, dass niemals alte und neue Datei gleichzeitig aktiv bleiben.

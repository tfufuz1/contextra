# 2. Löschung

## 2.1 Zustände eines Dokuments [E]

| Zustand | Bedeutung | Lesen | Schlüssel vorhanden |
|---|---|---|---|
| `Live` | regulär | ja | ja |
| `Tombstoned` | logisch entfernt, Schlüssel noch vorhanden (Übergangszustand) | nein | ja |
| `KeyRevoked` | RecordDek widerrufen, Daten noch auf dem Datenträger | nein (`Erased`) | nein |
| `Purged` | Indizes und Caches bereinigt | nein (`Erased`) | nein |
| `Measured` | Messung mit Positivkontrolle durchgeführt | nein (`Erased`) | nein |

**Regel [E]:** Ein Dokument ist ab `KeyRevoked` unlesbar. Ab diesem Zustand gilt: Auch ein Restart oder eine Compaction ändert daran nichts.

## 2.2 Löschablauf [E]

Der Ablauf MUSS in dieser Reihenfolge erfolgen. Jeder Schritt ist wiederholbar.

1. **Transaktion öffnen** (bestehender Pfad in `delete_op`).
2. **RecordDek widerrufen** (`KeyRegistry::revoke_record`) und den Widerruf durabel machen (`fsync` des Revocation-Logs). Ab diesem Punkt ist das Dokument `KeyRevoked`.
3. **Tombstone schreiben** für `user_key` und `doc_key` (bestehender Pfad).
4. **Indizes bereinigen**: Volltext (`stage_text_delete`), Vektor (`index.delete`), Graph (`stage_graph_entity_delete`). Jeder Index MUSS den Eintrag entfernen oder nur verschlüsselt halten.
5. **Transaktion abschließen** (`commit`). Bei Fehler: Rollback, und der Schlüssel bleibt widerrufen. Ein Widerruf wird nie zurückgenommen, weil das Dokument sonst wieder lesbar wäre.
6. **Messung** (2.4).
7. **Bericht** (2.5).

**Begründung der Reihenfolge.** Der Widerruf steht vor dem Tombstone. Stürzt das System zwischen 2 und 3 ab, ist das Dokument trotzdem unlesbar. Beim Neustart MUSS der Widerruf Vorrang vor dem Stand der Tombstones haben.

## 2.3 Modi [E]

Das Profil bestimmt den Modus. Der Modus ist keine reine Deklaration: Der Löschpfad MUSS ihn auswerten.

- `CryptoShred` (Standard für Compliance-Stufen): Ablauf nach 2.2.
- `TombstoneOnly`: nur Schritt 1, 3, 5 bis 7. Der Bericht MUSS ausweisen, dass Klasse K nicht erreicht wurde. Dieser Modus DARF NICHT mit einem aktiven Löschnachweis kombiniert werden (Profilvalidierung, `INV-COLLECTION-PROFILE-1`).

**[I]:** Der Modus wird heute nicht ausgewertet. Die Umsetzung dieses Kapitels setzt das um.

## 2.4 Messung und Positivkontrolle [E]

Eine Messung ohne Positivkontrolle gilt als ungültig.

- **Canary**: Vor der Messung wird ein Dokument mit einem bekannten Byte-Muster über denselben Pfad eingefügt (Insert, Flush, Compaction).
- **Positivkontrolle**: Das Messverfahren MUSS in einer Testdatei ein eingebettetes Muster finden. Damit ist belegt, dass der Scanner funktioniert, bevor er ein Nichtvorkommen als Ergebnis meldet.
- **Negativmessung**: Nach dem Löschen MUSS der Canary in keiner Datei des Datenverzeichnisses im Klartext vorkommen.
- **Lesepfad**: Eine Leseabfrage auf das Dokument MUSS `Erased` liefern.
- **Gesamtzahl der Dateien**: Die Messung zählt alle Dateien, auch WAL, Manifest, SSTables, Indexdateien und Logs.

Ergebnisse je Schicht:

| Schicht | Verfahren | Art |
|---|---|---|
| Lesepfad | Leseabfrage nach Löschung | vollständig |
| WAL, SSTables, Manifest | Byte-Suche über alle Dateien | vollständig |
| Volltextindex | Suche nach dem Muster in Posting-Listen | vollständig, nach Messung festzulegen |
| Vektorindex | Suche nach Vektor-ID und Payload | vollständig, nach Messung festzulegen |
| Temporäre Dateien | Byte-Suche | Stichprobe, ausgewiesen |
| RAM, Swap, Flash, Backups, externe Kopien | nicht messbar | deklariert (`A`, Kapitel 6 in README) |

Nicht implementierte Messungen liefern `NotImplemented`, nie `reached`.

## 2.5 Erfüllungsklassen [E]

| Klasse | Name | Erreicht, wenn |
|---|---|---|
| **L** | logisch entfernt | Tombstone geschrieben, Lesepfad liefert `Erased` |
| **K** | Schlüssel vernichtet | RecordDek widerrufen und durabel, Positivkontrolle bestanden, Negativmessung auf WAL und SSTables ohne Klartext-Treffer |
| **I** | Indizes bereinigt | Messung der Indizes ohne Treffer mit Positivkontrolle |
| **M** | vollständig gemessen | Klasse K und I, alle Schichten gemessen oder deklariert |

Der Bericht nennt je Klasse `reached` oder `not_reached` mit Zeitpunkt und Annahmen.

**Wortregel [E]:** „gelöscht“ steht nur mit Klasse K oder höher im Bericht. Bei L heißt es „logisch entfernt“.

## 2.6 Mehrere Dokumente und Collections [E]

- **Einzeldokument**: Widerruf des RecordDek dieses Dokuments.
- **Collection**: Widerruf des KEK der Gruppe. Alle RecordDeks der Gruppe werden damit unlesbar. Der Löschablauf für die einzelnen Dokumente läuft danach zur Bereinigung der Indizes und Messung weiter.
- **Mandant**: Widerruf aller Gruppen des Mandanten. Der Mandant MUSS danach im Bericht als `TenantRevoked` erscheinen.

## 2.7 Fehlerverhalten [E]

- Transiente Fehler (etwa Schreibfehler im Index): Wiederholung mit Backoff. Der Zustand bleibt erhalten.
- Ein Fehler nach dem Widerruf (Schritt 2): Das Dokument bleibt `KeyRevoked`. Die Bereinigung wird nachgeholt.
- Fehler beim Widerruf selbst: Die Operation schlägt fehl. Das Dokument bleibt `Live`, und der Aufrufer erhält einen Fehler.
- Keine Stille: Jeder Fehler im Löschpfad MUSS im Bericht erscheinen.

## 2.8 Testkatalog [E]

Die Testnamen MÜSSEN diesen Kennungen entsprechen.

| Kennung | Prüft | Quelle |
|---|---|---|
| `T-DEL-1` | Nach Insert und Flush kein Klartext-Canary in irgendeiner Datei des Datenverzeichnisses | `deletion_e2e.rs` (Erweiterung) |
| `T-DEL-2` | Nach Löschung, Flush und Compaction kein Klartext-Canary; Lesepfad liefert `Erased` | `deletion_e2e.rs`, bestehender Test `test_tdel_1_canary_bytes_erased_after_compaction` bleibt unverändert |
| `T-DEL-3` | Löschnachweis schlägt mit falschem Schlüssel fehl; der bisherige Test `test_tdel_3_drop_collection_e2e` wird umbenannt, damit die Kennung passt | `deletion_e2e.rs` |
| `T-DEL-4` | Nach Löschung und Neustart bleibt das Dokument `Erased` | neu |
| `T-DEL-5` | `CryptoShred` führt Widerruf aus; `TombstoneOnly` weist Klasse K als nicht erreicht aus | neu, `collection_profile_matrix.rs` ergänzen |
| `T-DEL-6` | Nach Löschung findet weder Volltext- noch Vektorsuche das Dokument | neu |
| `T-DEL-7` | Positivkontrolle: Der Scanner findet ein eingebettetes Muster | neu |
| `T-DEL-8` | Drop einer Collection widerruft den KEK; alle Dokumente der Collection sind danach `Erased` | neu |

## 2.9 Zustand [I]

- `T-DEL-1` schlägt im Stand vom 15.11.2026 fehl.
- `T-DEL-2` bis `T-DEL-8` sind nicht vorhanden. `T-DEL-3` existiert in anderer Form (siehe README, Kapitel 5).
- `kv_delete_mode` wird nicht ausgewertet.
- `T-DEL-1` läuft mit `ContextraConfig::default()`, also unverschlüsselt. Der Fehlschlag belegt daher Klartext auf dem Datenträger, nicht nur einen fehlenden Tombstone.

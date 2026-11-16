# 1. Schlüsselmodell

## 1.1 Hierarchie [E]

Jeder Payload wird mit einem Record-Schlüssel (DEK) verschlüsselt. Der DEK wird vom Gruppen-Schlüssel (KEK) der Collection umhüllt. Der KEK wird aus einem Master-Schlüssel abgeleitet.

```
MasterKey ──HKDF──> GroupKek(group_id, epoch)
GroupKek  ──wrap──> RecordDek(group_id, record_id)
RecordDek ──AEAD──> Payload(doc_id)
```

Grundlage im Code: `crates/contextra-erasure/src/kv_shredding.rs` (`derive_subkey`, `GroupEpochKey`, `RecordDek`, `KeyRegistry`).

Zuordnung für Contextra:
- `group_id` ist die Collection innerhalb eines Mandanten.
- `record_id` ist das Dokument (`doc_id`), numerisch abgebildet.
- Die Bindung aus Mandant, Collection und Dokument MUSS in die Ableitung eingehen (Additional Data). So kann ein Chiffretext nicht in einen anderen Mandanten oder eine andere Collection verschoben werden.

## 1.2 Bindung [E]

- **Eindeutigkeit**: Zwei Dokumentversionen DÜRFEN NICHT denselben Record-Schlüssel teilen.
- **Neue Version, neuer Schlüssel**: Jede Änderung eines Dokuments, die den Payload ersetzt, erzeugt einen neuen RecordDek. Der alte RecordDek unterliegt dem Löschablauf aus `02_loeschung.md`.
- **Gruppen-Schlüssel**: Das Droppen einer Collection vernichtet den KEK der Gruppe. Damit werden alle RecordDeks der Gruppe unlesbar. Das ist O(1) und unabhängig von der Anzahl der Dokumente.

## 1.3 Fail-closed [E]

- Fehlt ein RecordDek oder KEK, MUSS die Operation mit einem Fehler abbrechen. Sie DARF NICHT auf Klartext zurückfallen.
- Unterscheidbare Fehler:
  - `Erased`: Der Schlüssel ist nachweislich vernichtet (Register oder Revocation-Log).
  - `Integrity` oder `Crypto`: Der Zustand ist nicht eindeutig, etwa weil die Schlüsseldatei nicht lesbar ist.
- Die Reihenfolge ist: zuerst prüfen, ob das Dokument als gelöscht registriert ist, danach den Schlüssel suchen. So erscheint ein gelöschtes Dokument nie als Integritätsfehler.
- Entschlüsselungsfehler (Länge, Tag, falscher Schlüssel) DÜRFEN NICHT unterscheidbar sein. Nach außen gilt ein einheitlicher Fehler.

## 1.4 Geheimnisse im Speicher [E]

- Schlüsselmaterial besitzt weder `Clone`, `Debug` noch `Serialize`. Ausnahme: Ein Test-Modul kann es unter `#[cfg(test)]` gesondert erzeugen.
- Schlüsselmaterial wird beim Verwerfen auf null gesetzt (`ZeroizeOnDrop`).
- Die Entschlüsselung liefert einen Puffer mit vorab reservierter Kapazität. Ein Wachsen des Puffers würde Restkopien erzeugen.

## 1.5 Algorithmen und Nonces [E]

- AEAD ist AES-256-GCM-SIV (`contextra-crypto`), wie in `kv_shredding.rs` verwendet.
- Nonces DÜRFEN NICHT aus einem Zähler stammen, der nach einem Absturz zurückgesetzt werden kann. Sie SOLLEN zufällig aus dem Betriebssystem stammen.
- Ein Fehler des Zufallsgenerators DARF NICHT zu einem Chiffretext führen (fehlbarer Zufall, `Result`).

## 1.6 Zustand [I]

- `kv_shredding.rs` implementiert Gruppen-KEK und Record-DEK mit Widerruf (`revoke_group`, `revoke_record`).
- `contextra-db` nutzt diese Schlüsselverwaltung im Schreib- und Löschpfad nicht.
- Die Standardkonfiguration ist unverschlüsselt (`encryption_passphrase: None` in `ContextraConfig::default()`). Der T-DEL-1-Test läuft mit dieser Konfiguration.
- Die optionale Verschlüsselung über `EncryptionMode::Passphrase` schützt den gesamten Speicher mit einem Schlüssel. Ein einzelnes Dokument wird damit nicht unlesbar. Sie ersetzt also keine Löschung pro Dokument und ist nicht Teil der Lösung.
- Das ist die Lücke, die `02_loeschung.md` schließt.

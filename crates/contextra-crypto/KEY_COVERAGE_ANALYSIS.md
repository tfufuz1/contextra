# Schlüsselbaum- und Vernichtbarkeitsanalyse (T/C/S/D)

Gemäß SPEC §10.1, D-01, D-69, D-79 und Anhang F:

## Ableitung vs. Wrapping im Schlüsselbaum

1. **Master-Ebene (Wrapping via KEK):**
   - Der `MasterKey` wird im HEADER über einen Keyslot mittels `AES-256-GCM-SIV` und AAD (`ctx/slot/v1 ‖ store_id ‖ slot_id ‖ ...`) verpackt (Wrapped Master, 48 Byte).
   - Der KEK entsteht aus Passphrase und Slot-Salt über `Argon2id` (m ∈ [64 MiB, 4 GiB], t ∈ [3, 16], p ∈ [1, 16]).
   - Bei Vernichtung des Keyslots (Überschreiben mit Nullen, Status=2) ist der `MasterKey` aus der Passphrase nicht mehr herstellbar.

2. **Hierarchie-Ebene (HKDF-Ableitung aus K_master):**
   - Typisierte Schlüssel (`EncKey`, `IdxKey`, `UnitKey`, `SignSeed`, `WalKey`, `LedgerKey`, `HdrKey`) werden per HKDF-Expand mit festen, injektiven info-Labels abgeleitet.
   - Einweg-Eigenschaft von HKDF-SHA256 (RFC 5869): Aus abgeleiteten Schlüsseln kann `K_master` nicht zurückgerechnet werden.

3. **Dokumenten- und Daten-Ebene (T/C/S/D Ableitung per D-79):**
   - `DEK` wird deterministisch aus `UnitKey`, `AncestorSalts` (T, C, S, D) und `doc_tag` abgeleitet:
     `salt = SHA-256("ctx/dek-salt/v2" ‖ salt_T ‖ salt_C ‖ salt_S ‖ salt_D)`
     `DEK = HKDF-Expand(K_unit, "ctx/dek/v2" ‖ salt ‖ doc_tag)`
   - Die Salts für T (Tenant), C (Collection), S (Subject, optional), D (Document) liegen als 32-Byte-Zufalls-Payloads in den 80-Byte-Slots des Tresors (`vault/vault.dat`, F.3).

## Vernichtungs- und Herleitbarkeitstabelle (T-KEYCOV / T-VAULT)

| Vernichteter Slot | Was wird unlesbar? | Was bleibt herleitbar? | Schicht-Auswirkung (`key_coverage`) | Testverweis |
|---|---|---|---|---|
| **D-Slot (Dokument)** | DEK des konkreten Dokuments | Nichts für dieses Dokument. Alle anderen Dokumente (D' ≠ D) derselben Collection/Tenant bleiben lesbar. | `doc_store: full`<br>`vector_payload: full`<br>`text_postings: unlinked`<br>`vector_edges: none` | `T-KEYCOV-1`, `T-VAULT-2`, `test_cr07_dek_derivation_and_record_envelope_v2_t_enc_1_t_vault_2` |
| **S-Slot (Betroffener)** | Alle DEKs aller Dokumente dieses Subjects über alle Collections | Dokumente anderer Betroffener oder ohne Betroffenenbezug | `doc_store: full`<br>`vector_payload: full`<br>`text_postings: unlinked`<br>`vector_edges: none` | `T-KEYCOV-1`, `T-VAULT-3` |
| **C-Slot (Collection)** | Alle DEKs aller Dokumente dieser Collection | Andere Collections desselben Mandanten | `doc_store: full`<br>`vector_payload: full`<br>`text_postings: unlinked`<br>`vector_edges: none` | `T-KEYCOV-1`, `T-VAULT-4` |
| **T-Slot (Mandant)** | Alle DEKs sämtlicher Dokumente und Collections des Mandanten | Andere Mandanten (vollständige Mandantentrennung) | `doc_store: full`<br>`vector_payload: full`<br>`text_postings: unlinked`<br>`vector_edges: none` | `T-KEYCOV-1`, `T-VAULT-5` |
| **F-Slot (Datei)** | Dateischlüssel `K_file` für Blöcke/Graphen | Dateien anderer Dateischlüssel | `file_structure: unreadable`<br>`vector_edges: full` (nach Dateilöschung) | `T-FK-1..3` |

### Begriffliche Klarstellung (SPEC §6.4 & Invarianten)
- **`full`**: Payload-Inhalt ist kryptografisch unlesbar (DEK nicht mehr herleitbar).
- **`unlinked`**: Posting-Strukturen verbleiben bis zum Purge-Fenster ohne Bindung an Dokument/Betroffenen (Bindungs-Envelope unter DEK zerstört).
- **`none`**: Kanten/Strukturen werden durch Purge und Dateischlüssel-Vernichtung beseitigt; bis dahin Schutz durch Gate.

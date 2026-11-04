# ADR-SL-4: Reihenfolge Tresor- und WAL-Recovery sowie Formatklärung des WAL-Körpers

## Status
Vorschlag (Änderungsantrag zu SPEC 5.1.0 Abschnitt 14.1 und Anhang F.4, WP-W.3, Gate G0).

## Kontext
Zwei zusammenhängende Fragen bezüglich Startsequenz und WAL-Format sind in SPEC 5.1.0 unzureichend abgestimmt:

1. **Reihenfolge von Tresor-Öffnung und WAL-Recovery in SPEC 14.1:**
   In SPEC 14.1 lautet die Startsequenz:
   - Schritt 8: „Manifest, WAL-Recovery (`WalFloor`).“
   - Schritt 9: „Tresor öffnen: 16 Köpfe und Stichprobe, kein Vollscan (D-111 Abs. 8); Dateien ohne Manifest-Eintrag bereinigen (D-85); Waisen-Slots über `vault_scrub` (D-111).“
   Gemäß SPEC F.1 hat jedes WAL-Segment `kind = 1` und `key_mode = 0` (zufälliger Dateischlüssel im `F`-Slot des Tresors). Um die AEAD-verschlüsselten Frames des WAL-Segments während der WAL-Recovery (Schritt 8) entschlüsseln und über die HMAC-Kette verifizieren zu können, MUSS der Dateischlüssel über seinen `F`-Slot im Tresor nachgeschlagen werden. Da der Tresor in Schritt 8 laut 14.1 jedoch noch nicht geöffnet ist, kann Schritt 8 nicht ausgeführt werden.

2. **Verhältnis von Anhang F.1 zu Anhang F.4 beim WAL-Körper:**
   SPEC Anhang F.1 beschreibt den generischen Dateiaufbau aller Daten- und Steuerdateien. Dort heißt es:
   „Blöcke im Körper: `Block := u32 len ‖ ciphertext(len) ‖ aead_tag(16)`; Nonce `0x00000000 ‖ u64(Blockindex)`; AAD `kind ‖ file_id ‖ u64(Blockindex)`. Appendlogs ergänzen den Vorgänger-Hash in der AAD.“
   SPEC Anhang F.4 beschreibt dagegen spezifisch das WAL-Segment:
   „Kopf nach F.1, danach Segmentkopf `prev_mac(32) ‖ first_seq(u64)`; dann Frames:
   `Frame := len(u32) ‖ seq(u64) ‖ ct(len) ‖ aead_tag(16) ‖ mac(32)`
   `ct = AEAD(Dateischlüssel, nonce = 0x00000000 ‖ u64(Framezähler), AAD = "wal" ‖ file_id ‖ u64(Framezähler) ‖ prev_mac, Klartext = Batch)`
   `mac_n = HMAC-SHA256(K_wal, prev_mac ‖ len ‖ seq ‖ ct ‖ aead_tag)`“
   Es fehlte eine ausdrückliche Klarstellung, dass der Körper eines WAL-Segments ausschließlich aus Frames gemäß F.4 besteht und nicht in generische F.1-Blöcke verschachtelt ist, während der F.1-Kopf (64 Byte mit Magic `CTXFILE\0` und `header_tag`) vollumfänglich gilt.

## Entscheidung (Vorschlag)
1. **Anpassung der Startsequenz in SPEC 14.1:**
   - Schritt 8 wird aufgeteilt: Das Öffnen des Tresors (Lesen der 16 Shard-Köpfe, Metablöcke und Stichprobenvalidierung gemäß D-111) wird VOR die Manifest- und WAL-Recovery verschoben (neuer Schritt 8a).
   - Die WAL-Recovery wird als Schritt 8b ausgeführt, wobei sie auf den geöffneten Tresor für `F`-Slot-Lookups der WAL-Segmente zugreifen kann.
   - Die Waisenbereinigung (`vault_scrub`, Auffinden von unreferenzierten `F`-Slots) und Bereinigung von Dateien ohne Manifest-Eintrag (D-85) bleibt verbindlich NACH der Manifest- und WAL-Recovery (Schritt 9), da erst nach Replay des Manifests und des WAL feststeht, welche Dateien und Slots aktiv sind.
2. **Klarstellung des Formats in SPEC Anhang F.4:**
   - Es wird explizit festgehalten: Der Dateikopf eines WAL-Segments folgt strikt F.1 (Bytes 0..63 mit `header_tag` verifiziert über den im `F`-Slot hinterlegten Dateischlüssel).
   - Der Körper des WAL-Segments (ab Byte 64) besteht aus dem Segmentkopf (`prev_mac ‖ first_seq`) gefolgt von einer linearen Sequenz von Frames nach F.4. Er enthält KEINE generischen F.1-Blöcke.

## Alternativen
1. *F-Slots für WAL-Segmente nicht im Tresor, sondern im Manifest speichern:*
   Verworfen. Verletzt SPEC 10.1 (Schlüsselhierarchie D-79), da Dateischlüssel einheitlich über den Tresor verwaltet werden, um bei Schlüsselvernichtung dieselben atomaren Primitive zu nutzen.
2. *WAL mit abgeleitetem Schlüssel (`key_mode = 1`) betreiben:*
   Verworfen. Verletzt SPEC Anhang F.1 Tabelle (`kind = 1` hat `key_mode = 0`) und schwächt die kryptografische Trennung einzelner Segmente sowie die O(1)-Vernichtung von Altsegmenten.
3. *Vollständige Tresor-Recovery samt Waisenbereinigung vor Schritt 8:*
   Verworfen. Vor Replay von Manifest und WAL kann die Waisenbereinigung noch nicht wissen, ob ein `F`-Slot zu einem Segment gehört, das im WAL noch referenziert wird. Ein verfrühtes Scrubben könnte aktive Segmente beschädigen.

## Sicherheitsargument (Löschintegrität)
Durch das Öffnen der Tresor-Shards vor der WAL-Recovery wird sichergestellt, dass die kryptografische Verifikation der Segment-Header (`header_tag`) und jedes einzelnen Frames lückenlos und fail-closed durchgeführt werden kann. Ein Segment, dessen `F`-Slot fehlt oder dessen `header_tag` nicht zum Schlüssel passt, führt sofort zum kontrollierten Startabbruch.
Dass die Waisenbereinigung erst nach der WAL-Recovery erfolgt, garantiert, dass `F`-Slots von im Purge-Prozess befindlichen Dateien nicht vorzeitig gelöscht werden, bevor das WAL seinen Zustand (`WalFloor`) deklariert hat.

## Vorsichtigere Lesart (Fail-Closed)
Bis zur redaktionellen Anpassung von SPEC 14.1 gilt für Implementierungen:
Der Tresor MUSS schreibgeschützt (Read-Only) für Lookups geöffnet werden, bevor WAL-Segmente gelesen werden. Eine Modifikation oder Waisenbereinigung des Tresors darf vor Abschluss der WAL-Recovery unter keinen Umständen stattfinden. Schlägt der `F`-Slot-Lookup für ein im Manifest oder Verzeichnis vorgefundenes aktives WAL-Segment fehl, ist der Start unverzüglich mit `StorageHealth::Poisoned` abzubrechen.

## Betroffene SPEC-Stellen und Formulierungsvorschlag

### 1. SPEC Abschnitt 14.1 (Startsequenz):
Schritte 8 und 9 werden wie folgt präzisiert:
```diff
--- a/docs/SPEC.md
+++ b/docs/SPEC.md
@@ -14.1
-8. Manifest, WAL-Recovery (`WalFloor`). 9. Tresor öffnen: 16 Köpfe und Stichprobe, kein Vollscan (D-111 Abs. 8); Dateien ohne Manifest-Eintrag bereinigen (D-85); Waisen-Slots über `vault_scrub` (D-111).
+8. Tresor öffnen: 16 Köpfe und Stichprobe, kein Vollscan (D-111 Abs. 8). Danach Manifest, WAL-Recovery (`WalFloor`, Entschlüsselung über `F`-Slots im Tresor). 9. Bereinigung: Dateien ohne Manifest-Eintrag bereinigen (D-85); Waisen-Slots über `vault_scrub` (D-111).
```

### 2. SPEC Anhang F.4 (WAL-Segment):
Der einleitende Satz wird präzisiert:
```diff
--- a/docs/SPEC.md
+++ b/docs/SPEC.md
@@ -F.4
-**WAL-Segment** (`kind = 1`): Kopf nach F.1, danach Segmentkopf `prev_mac(32) ‖ first_seq(u64)`; dann Frames:
+**WAL-Segment** (`kind = 1`): Kopf (Bytes 0..63) nach F.1 mit `key_mode = 0`. Der Körper besteht nicht aus F.1-Blöcken, sondern beginnt unmittelbar mit dem Segmentkopf `prev_mac(32) ‖ first_seq(u64)`, gefolgt von Frames:
```

## Betroffene Komponenten und Tests
- `crates/contextra-engine`: Startsequenz in `wiring.rs` / `engine.rs`.
- `crates/contextra-store`: WAL-Segment-Decoder und Recovery (`wal/replay.rs`, `wal/io.rs`).
- `crates/contextra-erasure`: Tresor-Initialisierung (`vault/open.rs`, `vault/scrub.rs`).
- `crates/contextra-lab`: Format-Parser für Anhang F.1 und F.4.
- Tests:
  - `T-VAULT-8`: Tresor-Startprüfung und zeitlich entkoppelte Waisenbereinigung.
  - `T-WAL-1`: WAL-Segment-Parsing mit F.1-Header und F.4-Frames.
  - `T-CRASH-5`: Crash-Szenarien an der Schnittstelle von Tresor-Schreibvorgang und WAL-Recovery.

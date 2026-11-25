# Spezifikation: `contextra-wire` (SPEC v4.0, Klasse A)

> **Crate:** `contextra-wire`
> **Klasse:** A (Isolierter Protokoll-Baustein, SPEC §2, §5.1)
> **Ring:** Ring 0 (Unsafe-Insel FlatBuffers FFI)
> **Normativitätsstatus:** Verbindlich
> **Stand:** 2026-11-08

---

## 1. Zweck & Abgrenzung
`contextra-wire` ist ein zustandsloser, hochperformanter Protokoll- und Serialisierungsbaustein:
- **Aufgabe:** Konvertierung eingehender Bytestreams in typisierte, verifizierte Sichten ohne unnötige Allokationen (Zero-Copy) und Bereitstellung eines gehärteten JSON-RPC-2.0-Parsers.
- **Nicht-Ziele (SPEC §2, §6):** Keine Speicherung, kein I/O-Handling, keine Ausführung von RPC-Methoden, keine Kenntnis von Transaktionslogik, Mandanten-IDs (`TenantId`), Sagas, Deletion-Proofs oder DSGVO-Löschketten.

---

## 2. Invarianten

| Invariante | Name | Beschreibung |
|---|---|---|
| `INV-WIRE-FAILCLOSED` | Fail-Closed Verifikation | Kein Byte eines externen FlatBuffers-Puffers darf gelesen werden, bevor die vollständige Verifikation (`verification.rs`) erfolgreich war. Ungültige Puffer werden deterministisch abgelehnt. |
| `INV-WIRE-ZEROCOPY` | Zero-Copy Puffer-Adapter | `WireBuffer` kapselt `bytes::Bytes` ohne Zeiger- oder Lebenszeit-Transmutationen. Kopiervorgänge auf dem Lesepfad sind `O(1)`. |
| `INV-WIRE-ZEROPANIC` | Zero-Panic in Produktion | Handgeschriebener Produktionscode enthält strikt kein `unwrap()`, `expect()`, `panic!()` oder ungeprüftes Indexing. Fehler werden über `Result<T, Error>` gemeldet. |
| `INV-WIRE-DOSDEFENSE` | DoS-Abwehr (JSON-RPC) | Begrenzung von Nachrichtengröße (`max_message_size`), Schachtelungstiefe (`max_depth`) und Batch-Größe (`max_batch_size`). Duplicate Keys werden standardmäßig abgewiesen. |
| `INV-WIRE-ISOLATION` | Unsafe-Isolation | Maschinell erzeugter FlatBuffers-Code liegt isoliert in `src/generated.rs` / `src/contextra_generated.rs` (`@generated`). Handgeschriebener Code erzwingt `#![deny(unsafe_code)]`. |

---

## 3. Konstanten & Standard-Grenzwerte

### FlatBuffers Verifikation (`FlatBufferConfig`)
- `DEFAULT_MAX_MESSAGE_SIZE`: `10 * 1024 * 1024` Bytes (10 MiB)
- `DEFAULT_MAX_DEPTH`: `64` Ebenen
- `DEFAULT_MAX_TABLES`: `1_000_000` Tabellen

### JSON-RPC 2.0 Parser (`JsonRpcConfig`)
- `DEFAULT_MAX_MESSAGE_SIZE`: `10 * 1024 * 1024` Bytes (10 MiB)
- `DEFAULT_MAX_DEPTH`: `64` Ebenen
- `DEFAULT_MAX_BATCH_SIZE`: `100` Anfragen
- `DEFAULT_REJECT_DUPLICATE_KEYS`: `true`

---

## 4. Fehlermodell (B4)
Die Crate exponiert einen zentralen, erweiterbaren Fehlertyp:

```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("flatbuffers error: {0}")]
    FlatBuffer(#[from] FlatBufferError),

    #[error("json-rpc error: {0}")]
    JsonRpc(#[from] JsonRpcError),

    #[error("invalid input: {0}")]
    InvalidInput(&'static str),
}

pub type Result<T> = core::result::Result<T, Error>;
```

- `FlatBufferError`: `EmptyBuffer`, `BufferTooLarge`, `VerificationFailed(String)`, `IdentifierMismatch`.
- `JsonRpcError`: Standard-Codes (`-32700` Parse error, `-32600` Invalid request, `-32601` Method not found, `-32602` Invalid params, `-32603` Internal error).

---

## 5. Concurrency- & Thread-Safety-Garantien
- **Zustandslos:** Alle Parsing- und Verifikationsfunktionen sind zustandslos und wiedereintrittsfähig (`pure functions`).
- **Send + Sync:** Alle Typen (`WireBuffer`, `JsonRpcRequest`, `JsonRpcResponse`, `FlatBufferConfig`, `JsonRpcConfig`) implementieren `Send` und `Sync`.
- **Lock-frei:** Keine Mutexe, keine atomaren Sperren, kein Sperren über `.await`.
- **Laufzeit-Unabhängigkeit:** Kein Bezug zu Tokio, Futures oder Netzwerk-Stacks.

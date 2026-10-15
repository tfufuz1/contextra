# ADR-E75: Ack-Schlüssel für den Downgrade-Schutz per HKDF aus dem Inte

* **Status:** Akzeptiert (aus Spezifikation §13 E75)
* **Datum:** 2026-10-15

## Entscheidung
Ack-Schlüssel für den Downgrade-Schutz per HKDF aus dem Integritätsschlüssel (Info `contextra/ack/v1`).

## Begründung
Der Schlüssel war unspezifiziert (FE-15, O-4). Eine Ableitung braucht keinen neuen Speicherort.

## Verworfen
Eigenes Signaturschlüsselpaar für den Ack

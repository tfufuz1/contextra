# PENDING_API_CHANGES (Welle 2 - P20 - WP-14)

Format: Symbol | betroffene Crates | gewünschte Änderung

`Contextra::erase_document` | `contextra-engine` (Konsumenten zuvor in P10-P13 migriert) | Vollständig entfernt aus Engine; Nachweislogik liegt in `contextra-erasure`.
`Contextra::erase_collection` | `contextra-engine` (Konsumenten zuvor in P10-P13 migriert) | Vollständig entfernt aus Engine; Nachweislogik liegt in `contextra-erasure`.
`contextra_impl::erasure` | `contextra-engine` | Modul archiviert nach `archive/contextra-engine-erasure/`.
`summary_from_witness` | `contextra-engine` | Re-Export aus Engine entfernt.
`ErasureReport`, `ErasureFailure`, `LayerSummary`, `UNMEASURED_ERASURE_LAYERS` | `contextra-engine` | Aus Engine entfernt; gehören nach `contextra-erasure`.
`ProofKeyProvider` | `contextra-engine` | Trait aus Engine entfernt; liegt in `contextra-erasure` bzw. `contextra-ports`.
`Contextra::with_proof_key_provider` | `contextra-engine` | Aus Engine entfernt.

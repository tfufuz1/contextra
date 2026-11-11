# Findings während der Entkopplung von contextra-mvcc

Keine internen Fehler in contextra-mvcc.
Vorbestehender Fehler auf `main` außerhalb von BESITZ:
- `crates/contextra-store/src/sstable/builder.rs:621:3: unclosed delimiter`
Liegt nicht in contextra-mvcc und wurde vorschriftsmäßig unberührt gelassen.

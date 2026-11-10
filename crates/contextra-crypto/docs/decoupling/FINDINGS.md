# FINDINGS: contextra-crypto Entkopplung

## Vorbefunde / Auffälligkeiten (Stand: origin/main)

1. **Golden-Corpus Prüfsummen-Diskrepanz `dek_wrapping.bin`:**
   - In `tests/golden/MANIFEST.sha256` war für `dek_wrapping.bin` der Hash `984d29b3dca6c095d7d98c42376eb4e41b3dc47bdefefec121278bcb73a9dd70` eingetragen.
   - Die tatsächliche Datei `dek_wrapping.bin` hat den Hash `5943f46fee4bcb0ccbb84d182970665298b7b3c837e474e7ab5ddc7b1bb5eeea`.
   - Die Datei selbst wurde in Commit 3e7a41133 eingecheckt und unverändert gelassen.
   - Dies führte bei `cargo test -p contextra-crypto --test golden_corpus` zu einem Fehlschlag bei `test_golden_corpus_files_exist_and_match_manifest`.
   - Die Datei darf per Regel H2 nicht still verändert werden.

2. **Lange Testlaufzeiten in Debug-Profil:**
   - Tests mit 1 Million Nonces (`test_1m_nonce_uniqueness_stress` und `test_nonce_uniqueness_smoke_1m`) sowie 200k Nonces (`test_h3_nonce_uniqueness_200k_encryptions`) dauern im Debug-Profil mehrere Minuten und laufen in den Timeout. Im Release-Profil laufen sie in unter 1 Sekunde durch.

3. **Bestehende Clippy-Fehler (Baseline):**
   - 146 bestehende Clippy-Fehler in `contextra-crypto` (z. B. `cast_possible_truncation`, `doc_lazy_continuation`, `indexing_slicing`).

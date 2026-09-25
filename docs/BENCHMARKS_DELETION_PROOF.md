# Cryptographic Deletion Proof Latency Benchmarks

*Datum der Messung: 2026-09-25*
*Commit:* `7e6ee5396a84102fc5357c8489b558bd8d2a9d17`
*Environment: Linux x86_64, 4 CPU Cores (Intel Xeon @ 2.30GHz), 7.8 GiB RAM (Jules Sandbox VM)*

---

## WICHTIGER HINWEIS — DECKUNGSGRENZE DER KENNZAHL

Dieser Benchmark misst **AUSSCHLIESSLICH** die kryptografische Verarbeitungszeit:
- Erzeugung des Löschbeweises (`DeletionProof::create_with_wal_receipt` inkl. Blake3-Hashing der Schlüsselliste & HMAC-SHA256 Signatur)
- Signaturverifikation (`DeletionProof::verify`)
- Audit-Export (`DeletionProof::export_for_audit`)

Er misst **NICHT** die vorgelagerte physische Bereinigung der Storage-Layer (LSM-Compaction, WAL-Truncation, HNSW-Purge, CSR-Invalidierung).

---

## 0. Status-Matrix

| Claim / Metrik | Quelle (Datei / Befehl) | Status | Befund / Anmerkung |
|---|---|---|---|
| **deletion_proof_creation_latency (1 key)** | `cargo bench -p contextra-crypto --bench deletion_proof_latency_bench` | `reproduziert` | p50 = 2.50 µs, p95 = 2.51 µs |
| **deletion_proof_creation_latency (100 keys)** | `cargo bench -p contextra-crypto --bench deletion_proof_latency_bench` | `reproduziert` | p50 = 15.41 µs, p95 = 15.53 µs |
| **deletion_proof_creation_latency (10,000 keys)** | `cargo bench -p contextra-crypto --bench deletion_proof_latency_bench` | `reproduziert` | p50 = 2.30 ms, p95 = 2.30 ms |
| **deletion_proof_creation_latency (1,000,000 keys)** | `cargo bench -p contextra-crypto --bench deletion_proof_latency_bench` | `reproduziert` | p50 = 1.06 s, p95 = 1.07 s |
| **deletion_proof_verification_latency (1 key)** | `cargo bench -p contextra-crypto --bench deletion_proof_latency_bench` | `reproduziert` | p50 = 2.32 µs, p95 = 2.33 µs |
| **deletion_proof_verification_latency (100 keys)** | `cargo bench -p contextra-crypto --bench deletion_proof_latency_bench` | `reproduziert` | p50 = 2.36 µs, p95 = 2.38 µs |
| **deletion_proof_verification_latency (10,000 keys)** | `cargo bench -p contextra-crypto --bench deletion_proof_latency_bench` | `reproduziert` | p50 = 2.45 µs, p95 = 2.58 µs |
| **deletion_proof_verification_latency (1,000,000 keys)** | `cargo bench -p contextra-crypto --bench deletion_proof_latency_bench` | `reproduziert` | p50 = 2.32 µs, p95 = 2.32 µs |
| **deletion_proof_full_issuance_latency (1 key)** | `cargo bench -p contextra-crypto --bench deletion_proof_latency_bench` | `reproduziert` | p50 = 7.86 µs, p95 = 7.89 µs |
| **deletion_proof_full_issuance_latency (100 keys)** | `cargo bench -p contextra-crypto --bench deletion_proof_latency_bench` | `reproduziert` | p50 = 20.67 µs, p95 = 20.74 µs |
| **deletion_proof_full_issuance_latency (10,000 keys)** | `cargo bench -p contextra-crypto --bench deletion_proof_latency_bench` | `reproduziert` | p50 = 2.30 ms, p95 = 2.31 ms |
| **deletion_proof_full_issuance_latency (1,000,000 keys)** | `cargo bench -p contextra-crypto --bench deletion_proof_latency_bench` | `reproduziert` | p50 = 1.11 s, p95 = 1.12 s |

---

## 1. Verifizierter Messlauf

*Protokollierter Messlauf:*
- **Commit:** `7e6ee5396a84102fc5357c8489b558bd8d2a9d17`
- **Datum:** 2026-09-25
- **Hardware:** Intel(R) Xeon(R) Processor @ 2.30GHz (4 vCPUs), 7.8 GiB RAM, Linux x86_64 (Jules Sandbox VM)
- **Befehl:** `cargo bench -p contextra-crypto --bench deletion_proof_latency_bench`

### 1.1 `deletion_proof_creation_latency` (Erzeugung inkl. Blake3 Hash & HMAC Signatur)

| Anz. Gelöschter Schlüssel | Latenz p50 | Latenz p95 | Status |
|---|---|---|---|
| **1** | 2.50 µs | 2.51 µs | `reproduziert` |
| **100** | 15.41 µs | 15.53 µs | `reproduziert` |
| **10,000** | 2.30 ms | 2.30 ms | `reproduziert` |
| **1,000,000** | 1.06 s | 1.07 s | `reproduziert` |

### 1.2 `deletion_proof_verification_latency` (Signaturverifikation in O(1))

| Anz. Gelöschter Schlüssel | Latenz p50 | Latenz p95 | Status |
|---|---|---|---|
| **1** | 2.32 µs | 2.33 µs | `reproduziert` |
| **100** | 2.36 µs | 2.38 µs | `reproduziert` |
| **10,000** | 2.45 µs | 2.58 µs | `reproduziert` |
| **1,000,000** | 2.32 µs | 2.32 µs | `reproduziert` |

### 1.3 `deletion_proof_full_issuance_latency` (Erzeugung + Verifikation + Audit-Export)

| Anz. Gelöschter Schlüssel | Latenz p50 | Latenz p95 | Status |
|---|---|---|---|
| **1** | 7.86 µs | 7.89 µs | `reproduziert` |
| **100** | 20.67 µs | 20.74 µs | `reproduziert` |
| **10,000** | 2.30 ms | 2.31 ms | `reproduziert` |
| **1,000,000** | 1.11 s | 1.12 s | `reproduziert` |

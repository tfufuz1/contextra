# W1-12 SPECIFICATION DELTA & RESIDUAL RISK DOCUMENTATION

**Session:** W1-12 — contextra-testkit, Benchmarks und Beweis-Infrastruktur
**Stand:** 2026-10-22

---

## 1. APIs & Capabilities Introduced

### 1.1 `RawSweep` Builder & Positive Control (`testkit::raw_sweep`)
- **API:**
  ```rust
  pub struct RawSweep<'a>;
  impl<'a> RawSweep<'a> {
      pub fn new(data_dir: &'a Path) -> Self;
      pub fn with_marker(self, marker: impl Into<Vec<u8>>) -> Self;
      pub fn with_markers(self, markers: impl IntoIterator<Item = Vec<u8>>) -> Self;
      pub fn run(self) -> io::Result<OrcSweepReport>;
      pub fn assert_detects_planted(data_dir: &Path, marker: &[u8]) -> io::Result<()>;
  }
  ```
- **Invariants:** Streams files in 1-MiB chunk windows with overlap equal to max marker length minus 1. Deduplicates matches across chunk boundaries. Evaluates raw, hex, base64, and UTF-16LE encoding variants.

### 1.2 `ProcessImageProbe` (`testkit::process_image_probe`)
- **API:**
  ```rust
  pub struct ProcessImageProbe;
  impl ProcessImageProbe {
      pub fn scan_self(marker: &[u8]) -> Result<ProbeReport, ProbeError>;
  }
  ```
- **Behavior:**
  - **Linux:** Parses `/proc/self/maps` for readable virtual memory ranges (`r--p`/`rw-p`). Reads `/proc/self/mem` using `std::fs::File`, `Seek`, and `Read` without `unsafe` code. Counts per-region I/O errors into `ProbeReport.region_errors` without aborting.
  - **Non-Linux:** Returns `Err(ProbeError::Unsupported)` explicitly (fail-closed, no quiet `Ok`).

### 1.3 `CrashMatrix` Runner (`testkit::crash_matrix`)
- **API:**
  ```rust
  pub struct CrashMatrix;
  impl CrashMatrix {
      pub fn new(failpoints: &'static [&'static str]) -> Self;
      pub fn with_seed(self, seed: u64) -> Self;
      pub fn record_hit(failpoint: &str);
      pub async fn run<F, Fut, E>(&self, scenario_runner: F) -> Result<CrashMatrixReport, CrashMatrixError>;
  }
  ```
- **Invariants:** Verifies every registered failpoint is triggered during scenario execution. Fails with `CrashMatrixError::DeadFailpoints` if any failpoint is never triggered ("dead failpoint").

### 1.4 `ModelStore` MVCC Oracle (`testkit::model_store`)
- **API:**
  ```rust
  pub struct ModelStore;
  impl ModelStore {
      pub fn new() -> Self;
      pub fn get_at(&self, seq: u64, key: &[u8]) -> Option<Vec<u8>>;
      pub fn apply_batch(&mut self, ops: Vec<RefOp>) -> u64;
      pub fn purge_scope(&mut self, as_of_seq: u64, scope_prefix: &[u8]);
  }
  ```

---

## 2. Residual Risk Documentation (`ResidualRisk`)

| Residual Risk Term | Scope / Limitation | Explanation / Boundary |
| :--- | :--- | :--- |
| **`ResidualRisk::SwapSpace`** | Process Memory Inspection | `ProcessImageProbe` scans the active virtual memory maps of `/proc/self/mem`. Unmapped pages that were paged out to OS swap space by the kernel are not directly readable via `/proc/self/mem` until faulted back in. |
| **`ResidualRisk::KernelPageCache`** | Storage Data Scans | `RawSweep` scans physical files in the data directory. OS kernel page cache buffers associated with dirty file pages or unlinked inode buffers remain under OS kernel authority. |
| **`ResidualRisk::ExternalProcesses`** | Process Boundary Isolation | Memory of external OS processes is isolated by OS process boundaries and is strictly outside the self-probing scope of `ProcessImageProbe`. |

---

## 3. Decisions & Non-Breaking Invariants
- `contextra-testkit` maintains `#![forbid(unsafe_code)]` at crate level.
- `EntropyRng` in `contextra-testkit` serves as a type-separated test RNG source and never implements `fill_secret`.

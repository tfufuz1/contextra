//! WASI preview1 Host Function Implementations (§4.18).

use wasmtime::{Caller, Linker};

use crate::executor::SandboxState;

/// Error thrown on WASI `proc_exit`.
#[derive(Debug, Clone)]
pub struct ProcessExitError {
    pub code: i32,
}

impl std::fmt::Display for ProcessExitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Process exited with code {}", self.code)
    }
}

impl std::error::Error for ProcessExitError {}

/// Error thrown when stdout or stderr exceeds `max_output_bytes`.
#[derive(Debug, Clone)]
pub struct OutputLimitExceededError {
    pub stream: &'static str,
    pub limit: usize,
}

impl std::fmt::Display for OutputLimitExceededError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Output stream {} limit exceeded ({} bytes)",
            self.stream, self.limit
        )
    }
}

impl std::error::Error for OutputLimitExceededError {}

const ERRNO_SUCCESS: i32 = 0;
const ERRNO_ACCES: i32 = 2;
const ERRNO_BADF: i32 = 8;
const ERRNO_INVAL: i32 = 28;
const ERRNO_NOSYS: i32 = 52;
const ERRNO_NOTSUP: i32 = 58;

/// Registers WASI preview1 host functions into the provided Linker.
pub(crate) fn register(linker: &mut Linker<SandboxState>) -> Result<(), anyhow::Error> {
    // fd_read (fd 0 = stdin)
    linker.func_wrap(
        "wasi_snapshot_preview1",
        "fd_read",
        move |mut caller: Caller<'_, SandboxState>,
              fd: i32,
              iovs_ptr: i32,
              iovs_len: i32,
              nread_ptr: i32|
              -> i32 {
            if fd != 0 {
                return ERRNO_BADF;
            }
            if iovs_ptr < 0 || iovs_len < 0 || nread_ptr < 0 {
                return ERRNO_INVAL;
            }

            let memory = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(mem)) => mem,
                _ => return ERRNO_INVAL,
            };

            let mem_slice = memory.data(&caller);
            let iovs_start = iovs_ptr as usize;
            let iovs_count = iovs_len as usize;

            let iovs_bytes = match iovs_count.checked_mul(8) {
                Some(bytes) => bytes,
                None => return ERRNO_INVAL,
            };

            let iovs_end = match iovs_start.checked_add(iovs_bytes) {
                Some(end) => end,
                None => return ERRNO_INVAL,
            };

            if iovs_end > mem_slice.len() {
                return ERRNO_INVAL;
            }

            // Extract iovecs: (buf_ptr, buf_len)
            let mut iovecs = Vec::with_capacity(iovs_count);
            for i in 0..iovs_count {
                let offset = iovs_start + i * 8;
                let iov_buf = match mem_slice.get(offset..offset + 8) {
                    Some(slice) => slice,
                    None => return ERRNO_INVAL,
                };
                let buf_ptr = u32::from_le_bytes(match iov_buf[0..4].try_into() {
                    Ok(arr) => arr,
                    Err(_) => return ERRNO_INVAL,
                }) as usize;
                let buf_len = u32::from_le_bytes(match iov_buf[4..8].try_into() {
                    Ok(arr) => arr,
                    Err(_) => return ERRNO_INVAL,
                }) as usize;
                iovecs.push((buf_ptr, buf_len));
            }

            let nread_offset = nread_ptr as usize;
            let nread_end = match nread_offset.checked_add(4) {
                Some(end) => end,
                None => return ERRNO_INVAL,
            };

            if nread_end > mem_slice.len() {
                return ERRNO_INVAL;
            }

            // Copy available stdin slice into a local buffer to drop borrow on caller
            let available_input = {
                let state = caller.data();
                let stdin_input = &state.stdin_input;
                let current_pos = state.stdin_pos;
                if current_pos < stdin_input.len() {
                    stdin_input[current_pos..].to_vec()
                } else {
                    Vec::new()
                }
            };

            let mut input_offset = 0;
            let mut total_read: u32 = 0;

            // Fill memory buffers
            for (buf_ptr, buf_len) in iovecs {
                if buf_len == 0 {
                    continue;
                }
                let buf_end = match buf_ptr.checked_add(buf_len) {
                    Some(end) => end,
                    None => return ERRNO_INVAL,
                };

                let remaining_input = &available_input[input_offset..];
                if remaining_input.is_empty() {
                    break;
                }

                let to_copy = std::cmp::min(buf_len, remaining_input.len());
                let src_bytes = &remaining_input[..to_copy];

                let mem_slice_mut = memory.data_mut(&mut caller);
                if buf_end > mem_slice_mut.len() {
                    return ERRNO_INVAL;
                }

                if let Some(dest) = mem_slice_mut.get_mut(buf_ptr..buf_ptr + to_copy) {
                    dest.copy_from_slice(src_bytes);
                } else {
                    return ERRNO_INVAL;
                }

                input_offset += to_copy;
                total_read = match total_read.checked_add(to_copy as u32) {
                    Some(sum) => sum,
                    None => return ERRNO_INVAL,
                };
            }

            // Update stdin position
            caller.data_mut().stdin_pos += input_offset;

            // Write nread result to guest memory
            let mem_slice_mut = memory.data_mut(&mut caller);
            if let Some(dest) = mem_slice_mut.get_mut(nread_offset..nread_end) {
                dest.copy_from_slice(&total_read.to_le_bytes());
            } else {
                return ERRNO_INVAL;
            }

            ERRNO_SUCCESS
        },
    )?;

    // fd_write (fd 1 = stdout, fd 2 = stderr)
    linker.func_wrap(
        "wasi_snapshot_preview1",
        "fd_write",
        move |mut caller: Caller<'_, SandboxState>,
              fd: i32,
              iovs_ptr: i32,
              iovs_len: i32,
              nwritten_ptr: i32|
              -> Result<i32, wasmtime::Error> {
            if fd != 1 && fd != 2 {
                return Ok(ERRNO_BADF);
            }

            if iovs_ptr < 0 || iovs_len < 0 || nwritten_ptr < 0 {
                return Ok(ERRNO_INVAL);
            }

            let memory = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(mem)) => mem,
                _ => return Ok(ERRNO_INVAL),
            };

            let mem_slice = memory.data(&caller);
            let iovs_start = iovs_ptr as usize;
            let iovs_count = iovs_len as usize;

            let iovs_bytes = match iovs_count.checked_mul(8) {
                Some(bytes) => bytes,
                None => return Ok(ERRNO_INVAL),
            };

            let iovs_end = match iovs_start.checked_add(iovs_bytes) {
                Some(end) => end,
                None => return Ok(ERRNO_INVAL),
            };

            if iovs_end > mem_slice.len() {
                return Ok(ERRNO_INVAL);
            }

            let mut total_written: u32 = 0;
            let allow_write = if fd == 1 {
                caller.data().allow_stdout
            } else {
                caller.data().allow_stderr
            };
            let max_output_bytes = caller.data().max_output_bytes;

            for i in 0..iovs_count {
                let offset = iovs_start + i * 8;
                let iov_buf = match mem_slice.get(offset..offset + 8) {
                    Some(slice) => slice,
                    None => return Ok(ERRNO_INVAL),
                };

                let buf_ptr = u32::from_le_bytes(match iov_buf[0..4].try_into() {
                    Ok(arr) => arr,
                    Err(_) => return Ok(ERRNO_INVAL),
                }) as usize;
                let buf_len = u32::from_le_bytes(match iov_buf[4..8].try_into() {
                    Ok(arr) => arr,
                    Err(_) => return Ok(ERRNO_INVAL),
                }) as usize;

                let buf_end = match buf_ptr.checked_add(buf_len) {
                    Some(end) => end,
                    None => return Ok(ERRNO_INVAL),
                };

                if buf_end > mem_slice.len() {
                    return Ok(ERRNO_INVAL);
                }

                if allow_write && buf_len > 0 {
                    if let Some(slice) = mem_slice.get(buf_ptr..buf_end) {
                        if fd == 1 {
                            let mut guard = caller
                                .data()
                                .stdout_buf
                                .lock()
                                .map_err(|e| wasmtime::Error::msg(e.to_string()))?;
                            if guard.len().saturating_add(slice.len()) > max_output_bytes {
                                return Err(wasmtime::Error::from(OutputLimitExceededError {
                                    stream: "stdout",
                                    limit: max_output_bytes,
                                }));
                            }
                            guard.extend_from_slice(slice);
                        } else if fd == 2 {
                            let mut guard = caller
                                .data()
                                .stderr_buf
                                .lock()
                                .map_err(|e| wasmtime::Error::msg(e.to_string()))?;
                            if guard.len().saturating_add(slice.len()) > max_output_bytes {
                                return Err(wasmtime::Error::from(OutputLimitExceededError {
                                    stream: "stderr",
                                    limit: max_output_bytes,
                                }));
                            }
                            guard.extend_from_slice(slice);
                        }
                    }
                }

                total_written = match total_written.checked_add(buf_len as u32) {
                    Some(sum) => sum,
                    None => return Ok(ERRNO_INVAL),
                };
            }

            let nwritten_offset = nwritten_ptr as usize;
            let nwritten_end = match nwritten_offset.checked_add(4) {
                Some(end) => end,
                None => return Ok(ERRNO_INVAL),
            };

            let mem_slice_mut = memory.data_mut(&mut caller);
            if nwritten_end > mem_slice_mut.len() {
                return Ok(ERRNO_INVAL);
            }

            if let Some(dest) = mem_slice_mut.get_mut(nwritten_offset..nwritten_end) {
                dest.copy_from_slice(&total_written.to_le_bytes());
            } else {
                return Ok(ERRNO_INVAL);
            }

            Ok(ERRNO_SUCCESS)
        },
    )?;

    // proc_exit
    linker.func_wrap(
        "wasi_snapshot_preview1",
        "proc_exit",
        |_: Caller<'_, SandboxState>, code: i32| -> Result<(), wasmtime::Error> {
            Err(wasmtime::Error::from(ProcessExitError { code }))
        },
    )?;

    // clock_time_get
    linker.func_wrap(
        "wasi_snapshot_preview1",
        "clock_time_get",
        move |mut caller: Caller<'_, SandboxState>,
              clock_id: u32,
              _precision: u64,
              time_ptr: i32|
              -> i32 {
            if !caller.data().allow_clock {
                return ERRNO_ACCES;
            }
            if clock_id != 1 {
                return ERRNO_NOTSUP;
            }
            if time_ptr < 0 {
                return ERRNO_INVAL;
            }

            let time_offset = time_ptr as usize;
            let time_end = match time_offset.checked_add(8) {
                Some(end) => end,
                None => return ERRNO_INVAL,
            };

            let nanos = caller.data().start_instant.elapsed().as_nanos() as u64;

            let memory = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(mem)) => mem,
                _ => return ERRNO_INVAL,
            };

            let mem_slice_mut = memory.data_mut(&mut caller);
            if time_end > mem_slice_mut.len() {
                return ERRNO_INVAL;
            }

            if let Some(dest) = mem_slice_mut.get_mut(time_offset..time_end) {
                dest.copy_from_slice(&nanos.to_le_bytes());
                ERRNO_SUCCESS
            } else {
                ERRNO_INVAL
            }
        },
    )?;

    // random_get
    linker.func_wrap(
        "wasi_snapshot_preview1",
        "random_get",
        move |mut caller: Caller<'_, SandboxState>, buf_ptr: i32, buf_len: i32| -> i32 {
            if buf_ptr < 0 || buf_len < 0 {
                return ERRNO_INVAL;
            }

            let buf_offset = buf_ptr as usize;
            let len = buf_len as usize;

            let buf_end = match buf_offset.checked_add(len) {
                Some(end) => end,
                None => return ERRNO_INVAL,
            };

            let mut rng_state = match caller.data().rng_state {
                Some(state) => state,
                None => return ERRNO_NOSYS,
            };

            let memory = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(mem)) => mem,
                _ => return ERRNO_INVAL,
            };

            let mem_slice_mut = memory.data_mut(&mut caller);
            if buf_end > mem_slice_mut.len() {
                return ERRNO_INVAL;
            }

            // Deterministic SplitMix64 PRNG
            let mut generated = Vec::with_capacity(len);
            while generated.len() < len {
                rng_state = rng_state.wrapping_add(0x9e3779b97f4a7c15);
                let mut z = rng_state;
                z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
                let word = z ^ (z >> 31);
                let bytes = word.to_le_bytes();
                let to_take = std::cmp::min(8, len - generated.len());
                generated.extend_from_slice(&bytes[..to_take]);
            }

            if let Some(dest) = mem_slice_mut.get_mut(buf_offset..buf_end) {
                dest.copy_from_slice(&generated);
                caller.data_mut().rng_state = Some(rng_state);
                ERRNO_SUCCESS
            } else {
                ERRNO_INVAL
            }
        },
    )?;

    // args_sizes_get
    linker.func_wrap(
        "wasi_snapshot_preview1",
        "args_sizes_get",
        move |mut caller: Caller<'_, SandboxState>, argc_ptr: i32, argv_buf_size_ptr: i32| -> i32 {
            if argc_ptr < 0 || argv_buf_size_ptr < 0 {
                return ERRNO_INVAL;
            }

            let memory = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(mem)) => mem,
                _ => return ERRNO_INVAL,
            };

            let argc_offset = argc_ptr as usize;
            let argc_end = match argc_offset.checked_add(4) {
                Some(end) => end,
                None => return ERRNO_INVAL,
            };

            let buf_size_offset = argv_buf_size_ptr as usize;
            let buf_size_end = match buf_size_offset.checked_add(4) {
                Some(end) => end,
                None => return ERRNO_INVAL,
            };

            let mem_slice_mut = memory.data_mut(&mut caller);
            if argc_end > mem_slice_mut.len() || buf_size_end > mem_slice_mut.len() {
                return ERRNO_INVAL;
            }

            if argc_end > mem_slice_mut.len() || buf_size_end > mem_slice_mut.len() {
                return ERRNO_INVAL;
            }

            if let Some(c_dest) = mem_slice_mut.get_mut(argc_offset..argc_end) {
                c_dest.copy_from_slice(&0u32.to_le_bytes());
            } else {
                return ERRNO_INVAL;
            }

            if let Some(b_dest) = mem_slice_mut.get_mut(buf_size_offset..buf_size_end) {
                b_dest.copy_from_slice(&0u32.to_le_bytes());
            } else {
                return ERRNO_INVAL;
            }

            ERRNO_SUCCESS
        },
    )?;

    // args_get
    linker.func_wrap(
        "wasi_snapshot_preview1",
        "args_get",
        move |mut caller: Caller<'_, SandboxState>, argv_ptr: i32, argv_buf_ptr: i32| -> i32 {
            if argv_ptr < 0 || argv_buf_ptr < 0 {
                return ERRNO_INVAL;
            }

            let memory = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(mem)) => mem,
                _ => return ERRNO_INVAL,
            };

            let mem_slice = memory.data(&caller);
            if (argv_ptr as usize) > mem_slice.len() || (argv_buf_ptr as usize) > mem_slice.len() {
                return ERRNO_INVAL;
            }

            ERRNO_SUCCESS
        },
    )?;

    // environ_sizes_get
    linker.func_wrap(
        "wasi_snapshot_preview1",
        "environ_sizes_get",
        move |mut caller: Caller<'_, SandboxState>,
              environc_ptr: i32,
              environ_buf_size_ptr: i32|
              -> i32 {
            if environc_ptr < 0 || environ_buf_size_ptr < 0 {
                return ERRNO_INVAL;
            }

            let memory = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(mem)) => mem,
                _ => return ERRNO_INVAL,
            };

            let envc_offset = environc_ptr as usize;
            let envc_end = match envc_offset.checked_add(4) {
                Some(end) => end,
                None => return ERRNO_INVAL,
            };

            let buf_size_offset = environ_buf_size_ptr as usize;
            let buf_size_end = match buf_size_offset.checked_add(4) {
                Some(end) => end,
                None => return ERRNO_INVAL,
            };

            let mem_slice_mut = memory.data_mut(&mut caller);
            if envc_end > mem_slice_mut.len() || buf_size_end > mem_slice_mut.len() {
                return ERRNO_INVAL;
            }

            if let Some(c_dest) = mem_slice_mut.get_mut(envc_offset..envc_end) {
                c_dest.copy_from_slice(&0u32.to_le_bytes());
            } else {
                return ERRNO_INVAL;
            }

            if let Some(b_dest) = mem_slice_mut.get_mut(buf_size_offset..buf_size_end) {
                b_dest.copy_from_slice(&0u32.to_le_bytes());
            } else {
                return ERRNO_INVAL;
            }

            ERRNO_SUCCESS
        },
    )?;

    // environ_get
    linker.func_wrap(
        "wasi_snapshot_preview1",
        "environ_get",
        move |mut caller: Caller<'_, SandboxState>,
              environ_ptr: i32,
              environ_buf_ptr: i32|
              -> i32 {
            if environ_ptr < 0 || environ_buf_ptr < 0 {
                return ERRNO_INVAL;
            }

            let memory = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(mem)) => mem,
                _ => return ERRNO_INVAL,
            };

            let mem_slice = memory.data(&caller);
            if (environ_ptr as usize) > mem_slice.len()
                || (environ_buf_ptr as usize) > mem_slice.len()
            {
                return ERRNO_INVAL;
            }

            ERRNO_SUCCESS
        },
    )?;

    Ok(())
}

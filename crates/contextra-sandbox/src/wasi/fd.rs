// FILE-CONTEXT
// STAND: 2026-10-30T00:00:00Z
// ZWECK: WASI fd_read and fd_write implementations

use super::{OutputLimitExceededError, ERRNO_BADF, ERRNO_INVAL, ERRNO_SUCCESS};
use crate::executor::SandboxState;
use wasmtime::{Caller, Linker};

#[allow(clippy::indexing_slicing)]
pub(crate) fn register_fd(linker: &mut Linker<SandboxState>) -> Result<(), anyhow::Error> {
    // fd_read (fd 0 = stdin)
    linker.func_wrap(
        "wasi_snapshot_preview1",
        "fd_read",
        move |mut caller: Caller<'_, SandboxState>,
              fd: i32,
              iovs_ptr: i32,
              iovs_len: i32,
              nread_ptr: i32|
              -> Result<i32, wasmtime::Error> {
            let state = caller.data_mut();
            if let Err(e) = state.check_and_inc_host_call() {
                return Err(wasmtime::Error::from(e));
            }

            if fd != 0 {
                return Ok(ERRNO_BADF);
            }
            let iovs_start = match usize::try_from(iovs_ptr) {
                Ok(v) => v,
                Err(_) => return Ok(ERRNO_INVAL),
            };
            let iovs_count = match usize::try_from(iovs_len) {
                Ok(v) => v,
                Err(_) => return Ok(ERRNO_INVAL),
            };

            let memory = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(mem)) => mem,
                _ => return Ok(ERRNO_INVAL),
            };

            let mem_slice = memory.data(&caller);

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

            let mut iovecs = Vec::with_capacity(iovs_count);
            for i in 0..iovs_count {
                let offset = iovs_start + i * 8;
                let iov_buf = match mem_slice.get(offset..offset + 8) {
                    Some(slice) => slice,
                    None => return Ok(ERRNO_INVAL),
                };
                let buf_ptr_bytes: [u8; 4] = match iov_buf.get(0..4).and_then(|s| s.try_into().ok())
                {
                    Some(arr) => arr,
                    None => return Ok(ERRNO_INVAL),
                };
                let buf_len_bytes: [u8; 4] = match iov_buf.get(4..8).and_then(|s| s.try_into().ok())
                {
                    Some(arr) => arr,
                    None => return Ok(ERRNO_INVAL),
                };
                let buf_ptr = match usize::try_from(u32::from_le_bytes(buf_ptr_bytes)) {
                    Ok(v) => v,
                    Err(_) => return Ok(ERRNO_INVAL),
                };
                let buf_len = match usize::try_from(u32::from_le_bytes(buf_len_bytes)) {
                    Ok(v) => v,
                    Err(_) => return Ok(ERRNO_INVAL),
                };
                iovecs.push((buf_ptr, buf_len));
            }

            let nread_offset = match usize::try_from(nread_ptr) {
                Ok(v) => v,
                Err(_) => return Ok(ERRNO_INVAL),
            };
            let nread_end = match nread_offset.checked_add(4) {
                Some(end) => end,
                None => return Ok(ERRNO_INVAL),
            };

            if nread_end > mem_slice.len() {
                return Ok(ERRNO_INVAL);
            }

            let available_input = {
                let state = caller.data();
                let stdin_input = &state.stdin_input;
                let current_pos = state.stdin_pos;
                if current_pos < stdin_input.len() {
                    stdin_input.get(current_pos..).unwrap_or_default().to_vec()
                } else {
                    Vec::new()
                }
            };

            let mut input_offset = 0;
            let mut total_read: u32 = 0;

            for (buf_ptr, buf_len) in iovecs {
                if buf_len == 0 {
                    continue;
                }
                let buf_end = match buf_ptr.checked_add(buf_len) {
                    Some(end) => end,
                    None => return Ok(ERRNO_INVAL),
                };

                let remaining_input = available_input.get(input_offset..).unwrap_or_default();
                if remaining_input.is_empty() {
                    break;
                }

                let to_copy = std::cmp::min(buf_len, remaining_input.len());
                let src_bytes = match remaining_input.get(..to_copy) {
                    Some(slice) => slice,
                    None => return Ok(ERRNO_INVAL),
                };

                let mem_slice_mut = memory.data_mut(&mut caller);
                if buf_end > mem_slice_mut.len() {
                    return Ok(ERRNO_INVAL);
                }

                if let Some(dest) = mem_slice_mut.get_mut(buf_ptr..buf_ptr + to_copy) {
                    dest.copy_from_slice(src_bytes);
                } else {
                    return Ok(ERRNO_INVAL);
                }

                input_offset += to_copy;
                let to_copy_u32 = match u32::try_from(to_copy) {
                    Ok(v) => v,
                    Err(_) => return Ok(ERRNO_INVAL),
                };
                total_read = match total_read.checked_add(to_copy_u32) {
                    Some(sum) => sum,
                    None => return Ok(ERRNO_INVAL),
                };
            }

            caller.data_mut().stdin_pos += input_offset;

            let mem_slice_mut = memory.data_mut(&mut caller);
            if let Some(dest) = mem_slice_mut.get_mut(nread_offset..nread_end) {
                dest.copy_from_slice(&total_read.to_le_bytes());
            } else {
                return Ok(ERRNO_INVAL);
            }

            Ok(ERRNO_SUCCESS)
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
            let state = caller.data_mut();
            if let Err(e) = state.check_and_inc_host_call() {
                return Err(wasmtime::Error::from(e));
            }

            if fd != 1 && fd != 2 {
                return Ok(ERRNO_BADF);
            }

            let iovs_start = match usize::try_from(iovs_ptr) {
                Ok(v) => v,
                Err(_) => return Ok(ERRNO_INVAL),
            };
            let iovs_count = match usize::try_from(iovs_len) {
                Ok(v) => v,
                Err(_) => return Ok(ERRNO_INVAL),
            };

            let memory = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(mem)) => mem,
                _ => return Ok(ERRNO_INVAL),
            };

            let mem_slice = memory.data(&caller);

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

                let buf_ptr_bytes: [u8; 4] = match iov_buf.get(0..4).and_then(|s| s.try_into().ok())
                {
                    Some(arr) => arr,
                    None => return Ok(ERRNO_INVAL),
                };
                let buf_len_bytes: [u8; 4] = match iov_buf.get(4..8).and_then(|s| s.try_into().ok())
                {
                    Some(arr) => arr,
                    None => return Ok(ERRNO_INVAL),
                };

                let buf_ptr = match usize::try_from(u32::from_le_bytes(buf_ptr_bytes)) {
                    Ok(v) => v,
                    Err(_) => return Ok(ERRNO_INVAL),
                };
                let buf_len = match usize::try_from(u32::from_le_bytes(buf_len_bytes)) {
                    Ok(v) => v,
                    Err(_) => return Ok(ERRNO_INVAL),
                };

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

                let buf_len_u32 = match u32::try_from(buf_len) {
                    Ok(v) => v,
                    Err(_) => return Ok(ERRNO_INVAL),
                };
                total_written = match total_written.checked_add(buf_len_u32) {
                    Some(sum) => sum,
                    None => return Ok(ERRNO_INVAL),
                };
            }

            let nwritten_offset = match usize::try_from(nwritten_ptr) {
                Ok(v) => v,
                Err(_) => return Ok(ERRNO_INVAL),
            };
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

    Ok(())
}

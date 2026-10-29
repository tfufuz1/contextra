// FILE-CONTEXT
// STAND: 2026-10-30T00:00:00Z
// ZWECK: WASI proc_exit, clock_time_get, random_get, args, environ implementations

use super::{ProcessExitError, ERRNO_ACCES, ERRNO_INVAL, ERRNO_NOSYS, ERRNO_NOTSUP, ERRNO_SUCCESS};
use crate::executor::SandboxState;
use wasmtime::{Caller, Linker};

pub(crate) fn register_sys(linker: &mut Linker<SandboxState>) -> Result<(), anyhow::Error> {
    // proc_exit
    linker.func_wrap(
        "wasi_snapshot_preview1",
        "proc_exit",
        |mut caller: Caller<'_, SandboxState>, code: i32| -> Result<(), wasmtime::Error> {
            let _ = caller.data_mut().check_and_inc_host_call();
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
              -> Result<i32, wasmtime::Error> {
            let state = caller.data_mut();
            if let Err(e) = state.check_and_inc_host_call() {
                return Err(wasmtime::Error::from(e));
            }

            if !caller.data().allow_clock {
                return Ok(ERRNO_ACCES);
            }
            if clock_id != 1 {
                return Ok(ERRNO_NOTSUP);
            }
            let time_offset = match usize::try_from(time_ptr) {
                Ok(v) => v,
                Err(_) => return Ok(ERRNO_INVAL),
            };

            let time_end = match time_offset.checked_add(8) {
                Some(end) => end,
                None => return Ok(ERRNO_INVAL),
            };

            let nanos_u128 = caller.data().start_instant.elapsed().as_nanos();
            let nanos = match u64::try_from(nanos_u128) {
                Ok(v) => v,
                Err(_) => u64::MAX,
            };

            let memory = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(mem)) => mem,
                _ => return Ok(ERRNO_INVAL),
            };

            let mem_slice_mut = memory.data_mut(&mut caller);
            if time_end > mem_slice_mut.len() {
                return Ok(ERRNO_INVAL);
            }

            if let Some(dest) = mem_slice_mut.get_mut(time_offset..time_end) {
                dest.copy_from_slice(&nanos.to_le_bytes());
                Ok(ERRNO_SUCCESS)
            } else {
                Ok(ERRNO_INVAL)
            }
        },
    )?;

    // random_get
    linker.func_wrap(
        "wasi_snapshot_preview1",
        "random_get",
        move |mut caller: Caller<'_, SandboxState>,
              buf_ptr: i32,
              buf_len: i32|
              -> Result<i32, wasmtime::Error> {
            let state = caller.data_mut();
            if let Err(e) = state.check_and_inc_host_call() {
                return Err(wasmtime::Error::from(e));
            }

            let buf_offset = match usize::try_from(buf_ptr) {
                Ok(v) => v,
                Err(_) => return Ok(ERRNO_INVAL),
            };
            let len = match usize::try_from(buf_len) {
                Ok(v) => v,
                Err(_) => return Ok(ERRNO_INVAL),
            };

            let buf_end = match buf_offset.checked_add(len) {
                Some(end) => end,
                None => return Ok(ERRNO_INVAL),
            };

            let mut rng_state = match caller.data().rng_state {
                Some(state) => state,
                None => return Ok(ERRNO_NOSYS),
            };

            let memory = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(mem)) => mem,
                _ => return Ok(ERRNO_INVAL),
            };

            let mem_slice_mut = memory.data_mut(&mut caller);
            if buf_end > mem_slice_mut.len() {
                return Ok(ERRNO_INVAL);
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
                if let Some(slice) = bytes.get(..to_take) {
                    generated.extend_from_slice(slice);
                }
            }

            if let Some(dest) = mem_slice_mut.get_mut(buf_offset..buf_end) {
                dest.copy_from_slice(&generated);
                caller.data_mut().rng_state = Some(rng_state);
                Ok(ERRNO_SUCCESS)
            } else {
                Ok(ERRNO_INVAL)
            }
        },
    )?;

    // args_sizes_get
    linker.func_wrap(
        "wasi_snapshot_preview1",
        "args_sizes_get",
        move |mut caller: Caller<'_, SandboxState>,
              argc_ptr: i32,
              argv_buf_size_ptr: i32|
              -> Result<i32, wasmtime::Error> {
            let state = caller.data_mut();
            if let Err(e) = state.check_and_inc_host_call() {
                return Err(wasmtime::Error::from(e));
            }

            let argc_offset = match usize::try_from(argc_ptr) {
                Ok(v) => v,
                Err(_) => return Ok(ERRNO_INVAL),
            };
            let buf_size_offset = match usize::try_from(argv_buf_size_ptr) {
                Ok(v) => v,
                Err(_) => return Ok(ERRNO_INVAL),
            };

            let memory = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(mem)) => mem,
                _ => return Ok(ERRNO_INVAL),
            };

            let argc_end = match argc_offset.checked_add(4) {
                Some(end) => end,
                None => return Ok(ERRNO_INVAL),
            };

            let buf_size_end = match buf_size_offset.checked_add(4) {
                Some(end) => end,
                None => return Ok(ERRNO_INVAL),
            };

            let mem_slice_mut = memory.data_mut(&mut caller);
            if argc_end > mem_slice_mut.len() || buf_size_end > mem_slice_mut.len() {
                return Ok(ERRNO_INVAL);
            }

            if let Some(c_dest) = mem_slice_mut.get_mut(argc_offset..argc_end) {
                c_dest.copy_from_slice(&0u32.to_le_bytes());
            } else {
                return Ok(ERRNO_INVAL);
            }

            if let Some(b_dest) = mem_slice_mut.get_mut(buf_size_offset..buf_size_end) {
                b_dest.copy_from_slice(&0u32.to_le_bytes());
            } else {
                return Ok(ERRNO_INVAL);
            }

            Ok(ERRNO_SUCCESS)
        },
    )?;

    // args_get
    linker.func_wrap(
        "wasi_snapshot_preview1",
        "args_get",
        move |mut caller: Caller<'_, SandboxState>,
              argv_ptr: i32,
              argv_buf_ptr: i32|
              -> Result<i32, wasmtime::Error> {
            let state = caller.data_mut();
            if let Err(e) = state.check_and_inc_host_call() {
                return Err(wasmtime::Error::from(e));
            }

            let argv_offset = match usize::try_from(argv_ptr) {
                Ok(v) => v,
                Err(_) => return Ok(ERRNO_INVAL),
            };
            let argv_buf_offset = match usize::try_from(argv_buf_ptr) {
                Ok(v) => v,
                Err(_) => return Ok(ERRNO_INVAL),
            };

            let memory = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(mem)) => mem,
                _ => return Ok(ERRNO_INVAL),
            };

            let mem_slice = memory.data(&caller);
            if argv_offset > mem_slice.len() || argv_buf_offset > mem_slice.len() {
                return Ok(ERRNO_INVAL);
            }

            Ok(ERRNO_SUCCESS)
        },
    )?;

    // environ_sizes_get
    linker.func_wrap(
        "wasi_snapshot_preview1",
        "environ_sizes_get",
        move |mut caller: Caller<'_, SandboxState>,
              environc_ptr: i32,
              environ_buf_size_ptr: i32|
              -> Result<i32, wasmtime::Error> {
            let state = caller.data_mut();
            if let Err(e) = state.check_and_inc_host_call() {
                return Err(wasmtime::Error::from(e));
            }

            let envc_offset = match usize::try_from(environc_ptr) {
                Ok(v) => v,
                Err(_) => return Ok(ERRNO_INVAL),
            };
            let buf_size_offset = match usize::try_from(environ_buf_size_ptr) {
                Ok(v) => v,
                Err(_) => return Ok(ERRNO_INVAL),
            };

            let memory = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(mem)) => mem,
                _ => return Ok(ERRNO_INVAL),
            };

            let envc_end = match envc_offset.checked_add(4) {
                Some(end) => end,
                None => return Ok(ERRNO_INVAL),
            };

            let buf_size_end = match buf_size_offset.checked_add(4) {
                Some(end) => end,
                None => return Ok(ERRNO_INVAL),
            };

            let mem_slice_mut = memory.data_mut(&mut caller);
            if envc_end > mem_slice_mut.len() || buf_size_end > mem_slice_mut.len() {
                return Ok(ERRNO_INVAL);
            }

            if let Some(c_dest) = mem_slice_mut.get_mut(envc_offset..envc_end) {
                c_dest.copy_from_slice(&0u32.to_le_bytes());
            } else {
                return Ok(ERRNO_INVAL);
            }

            if let Some(b_dest) = mem_slice_mut.get_mut(buf_size_offset..buf_size_end) {
                b_dest.copy_from_slice(&0u32.to_le_bytes());
            } else {
                return Ok(ERRNO_INVAL);
            }

            Ok(ERRNO_SUCCESS)
        },
    )?;

    // environ_get
    linker.func_wrap(
        "wasi_snapshot_preview1",
        "environ_get",
        move |mut caller: Caller<'_, SandboxState>,
              environ_ptr: i32,
              environ_buf_ptr: i32|
              -> Result<i32, wasmtime::Error> {
            let state = caller.data_mut();
            if let Err(e) = state.check_and_inc_host_call() {
                return Err(wasmtime::Error::from(e));
            }

            let env_offset = match usize::try_from(environ_ptr) {
                Ok(v) => v,
                Err(_) => return Ok(ERRNO_INVAL),
            };
            let env_buf_offset = match usize::try_from(environ_buf_ptr) {
                Ok(v) => v,
                Err(_) => return Ok(ERRNO_INVAL),
            };

            let memory = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(mem)) => mem,
                _ => return Ok(ERRNO_INVAL),
            };

            let mem_slice = memory.data(&caller);
            if env_offset > mem_slice.len() || env_buf_offset > mem_slice.len() {
                return Ok(ERRNO_INVAL);
            }

            Ok(ERRNO_SUCCESS)
        },
    )?;

    Ok(())
}

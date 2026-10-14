use contextra_ports::erasure::{ErasureHooks, HookToken};

fn main() {}

fn test_double_finish(hooks: &dyn ErasureHooks, token: HookToken) {
    let _ = hooks.finish(token);
    let _ = hooks.finish(token);
}

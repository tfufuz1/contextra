pub const EXIT_PASS: i32 = 0;
pub const EXIT_GATE_VIOLATION: i32 = 1;
pub const EXIT_ERROR: i32 = 2;

pub fn exit_with_code(code: i32) -> ! {
    std::process::exit(code);
}

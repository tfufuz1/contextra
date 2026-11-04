use contextra_sys::secure_zero_volatile;

fn main() {
    let mut secret = [0x5A; 256];
    secure_zero_volatile(&mut secret);
    for b in secret.iter() {
        if *b != 0 {
            std::process::exit(1);
        }
    }
}

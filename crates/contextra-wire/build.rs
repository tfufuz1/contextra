#![allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]

fn main() {
    println!("cargo:rerun-if-changed=../../schemas/contextra.fbs");
}

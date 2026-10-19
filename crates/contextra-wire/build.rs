#![allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]

use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../../schemas/contextra.fbs");
}

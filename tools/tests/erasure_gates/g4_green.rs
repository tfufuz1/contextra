use std::path::Path;

// std::fs ist hier verboten
fn a(vfs: &dyn Vfs, offs: u64) {
    vfs.remove_file(Path::new("std::fs"));
}

use std::fs::File;
use std::io::Write;
use tempfile::NamedTempFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut temp = NamedTempFile::new()?;
    temp.write_all(b"contextra-sys standalone mmap payload")?;
    let file = File::open(temp.path())?;
    let mmap = contextra_sys::mmap_readonly(&file)?;
    assert_eq!(&mmap[..], b"contextra-sys standalone mmap payload");
    println!("Successfully read {} bytes via mmap_readonly", mmap.len());
    Ok(())
}

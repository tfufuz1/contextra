//! Unterkommando zur Prüfung der Reproduzierbarkeit von Workspace-Builds.
//!
//! Baut den Workspace (oder ein spezifiziertes Package) zweimal nacheinander in isolierten
//! Target-Verzeichnissen (`target/repro_build_1` und `target/repro_build_2`) mit identischen
//! `RUSTFLAGS` und Toolchain-Einstellungen und vergleicht die SHA-256-Hashes der resultierenden
//! Binaries. Bei Abweichung: Fehler mit Diff-Hinweis.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

pub fn run_reproducible_build_check(args: &[String]) -> bool {
    let start = Instant::now();
    println!("=== Running xtask reproducible-build-check ===");

    // Parse command line arguments for optional target package filter
    let mut target_package = None;
    let mut i = 0;
    while i < args.len() {
        if let Some(val) = args[i].strip_prefix("--package=") {
            target_package = Some(val.to_string());
        } else if let Some(val) = args[i].strip_prefix("--crate=") {
            target_package = Some(val.to_string());
        } else if (args[i] == "--package" || args[i] == "-p" || args[i] == "--crate")
            && i + 1 < args.len()
        {
            target_package = Some(args[i + 1].clone());
            i += 1;
        }
        i += 1;
    }

    let root_dir = crate::find_root_dir();
    let target_dir = root_dir.join("target");
    let dir1 = target_dir.join("repro_build_1");
    let dir2 = target_dir.join("repro_build_2");

    // Clean build dirs if they exist
    let _ = fs::remove_dir_all(&dir1);
    let _ = fs::remove_dir_all(&dir2);

    println!("Executing Run 1 in isolated target dir: {}", dir1.display());
    if !execute_build(&root_dir, &dir1, target_package.as_deref()) {
        eprintln!("❌ [GATE-REPRODUCIBLE-BUILD]: Build Run 1 failed");
        return false;
    }

    println!("Executing Run 2 in isolated target dir: {}", dir2.display());
    if !execute_build(&root_dir, &dir2, target_package.as_deref()) {
        eprintln!("❌ [GATE-REPRODUCIBLE-BUILD]: Build Run 2 failed");
        return false;
    }

    println!("Comparing binary SHA-256 hashes between Run 1 and Run 2...");
    let binaries1 = collect_executable_hashes(&dir1.join("release"));
    let binaries2 = collect_executable_hashes(&dir2.join("release"));

    if binaries1.is_empty() {
        println!("⚠️ No release binaries were generated in target directory.");
        println!(
            "✅ [GATE-REPRODUCIBLE-BUILD]: Check completed in {:.2}s (no binaries produced)",
            start.elapsed().as_secs_f64()
        );
        return true;
    }

    let mut mismatch = false;
    for (rel_path, hash1) in &binaries1 {
        match binaries2.get(rel_path) {
            Some(hash2) => {
                if hash1 == hash2 {
                    println!("  ✅ {:<30} SHA-256 match: {}", rel_path, hash1);
                } else {
                    eprintln!("  ❌ {:<30} SHA-256 MISMATCH!", rel_path);
                    eprintln!("     Run 1: {}", hash1);
                    eprintln!("     Run 2: {}", hash2);
                    mismatch = true;
                }
            }
            None => {
                eprintln!("  ❌ {:<30} Missing in Run 2!", rel_path);
                mismatch = true;
            }
        }
    }

    for rel_path in binaries2.keys() {
        if !binaries1.contains_key(rel_path) {
            eprintln!("  ❌ {:<30} Present in Run 2 but missing in Run 1!", rel_path);
            mismatch = true;
        }
    }

    if mismatch {
        eprintln!("❌ [GATE-REPRODUCIBLE-BUILD]: SHA-256 mismatch detected between builds! Builds are not byte-for-byte reproducible.");
        false
    } else {
        println!(
            "✅ [GATE-REPRODUCIBLE-BUILD]: All {} binaries produced byte-for-byte identical output ({:.2}s)",
            binaries1.len(),
            start.elapsed().as_secs_f64()
        );
        true
    }
}

fn execute_build(root_dir: &Path, target_dir: &Path, package: Option<&str>) -> bool {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(root_dir);
    cmd.arg("build");
    cmd.arg("--release");
    cmd.arg("--locked");
    cmd.arg("--target-dir");
    cmd.arg(target_dir);

    if let Some(pkg) = package {
        cmd.arg("--package");
        cmd.arg(pkg);
    } else {
        cmd.arg("--workspace");
    }

    // Pass deterministic RUSTFLAGS
    cmd.env("RUSTFLAGS", "--remap-path-prefix .=.");

    let status = cmd.status();
    match status {
        Ok(st) => st.success(),
        Err(e) => {
            eprintln!("Failed to spawn cargo build: {}", e);
            false
        }
    }
}

fn collect_executable_hashes(release_dir: &Path) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    if !release_dir.is_dir() {
        return map;
    }

    if let Ok(entries) = fs::read_dir(release_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && is_executable_file(&path) {
                if let Ok(bytes) = fs::read(&path) {
                    let sha256_hash = compute_sha256(&bytes);
                    let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                    map.insert(file_name, sha256_hash);
                }
            }
        }
    }
    map
}

fn is_executable_file(path: &Path) -> bool {
    let file_name = path.file_name().unwrap_or_default().to_string_lossy();

    // Skip internal build artifacts
    if file_name.ends_with(".d")
        || file_name.ends_with(".rlib")
        || file_name.ends_with(".rmeta")
        || file_name.ends_with(".so")
        || file_name.ends_with(".dylib")
        || file_name.ends_with(".dll")
        || file_name.ends_with(".a")
        || file_name.ends_with(".o")
        || file_name.ends_with(".pdb")
        || file_name.starts_with("lib")
    {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(path) {
            return metadata.permissions().mode() & 0o111 != 0;
        }
    }

    #[cfg(windows)]
    {
        return file_name.ends_with(".exe");
    }

    #[allow(unreachable_code)]
    true
}

fn compute_sha256(bytes: &[u8]) -> String {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    let k: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef4a9f7, 0xc67178f2,
    ];

    let bit_len = (bytes.len() as u64) * 8;
    let mut padded = bytes.to_vec();
    padded.push(0x80);
    while (padded.len() % 64) != 56 {
        padded.push(0x00);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut h_val = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h_val
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(k[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h_val = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(h_val);
    }

    format!(
        "{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}",
        h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]
    )
}

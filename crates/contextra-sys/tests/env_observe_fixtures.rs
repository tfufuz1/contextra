use contextra_sys::{
    cow_fs_from_fs_kind_and_options, observe, parse_core_pattern, parse_dm_uuid, parse_limits,
    parse_mountinfo, parse_proc_swaps, parse_rotational, EnvFsKind, EnvOsKind, EnvTri,
};
use std::path::Path;

#[test]
fn test_parse_mountinfo_fixtures() {
    let fixture = r#"
36 35 98:0 / /mnt/btrfs rw,relatime shared:1 - btrfs /dev/sdb1 rw,discard=async,space_cache=v2
37 35 98:1 / /mnt/ext4 rw,relatime shared:2 - ext4 /dev/sda2 rw,errors=remount-ro
38 35 98:2 / /mnt/xfs rw,relatime shared:3 - xfs /dev/sdc1 rw,attr2,inline,logbufs=8,logbsize=32k,reflink,noquota
39 35 98:3 / /mnt/xfs_noreflink rw,relatime shared:4 - xfs /dev/sdc2 rw,noreflink
"#;

    let (fs1, disc1) = parse_mountinfo(fixture, Path::new("/mnt/btrfs/data"));
    assert_eq!(fs1, EnvFsKind::Btrfs);
    assert_eq!(disc1, EnvTri::Yes);

    let (fs2, disc2) = parse_mountinfo(fixture, Path::new("/mnt/ext4/sub/dir"));
    assert_eq!(fs2, EnvFsKind::Ext4);
    assert_eq!(disc2, EnvTri::No);

    let (fs3, disc3) = parse_mountinfo(fixture, Path::new("/mnt/xfs/test"));
    assert_eq!(fs3, EnvFsKind::Xfs);
    assert_eq!(disc3, EnvTri::No);

    // Malformed line handling
    let malformed = "invalid mountinfo string without formatting";
    let (fs_err, disc_err) = parse_mountinfo(malformed, Path::new("/mnt/test"));
    assert_eq!(fs_err, EnvFsKind::Unknown);
    assert_eq!(disc_err, EnvTri::Unknown);
}

#[test]
fn test_cow_fs_evaluation_rules() {
    assert_eq!(
        cow_fs_from_fs_kind_and_options(EnvFsKind::Btrfs, "rw,discard"),
        EnvTri::Yes
    );
    assert_eq!(
        cow_fs_from_fs_kind_and_options(EnvFsKind::Zfs, "rw"),
        EnvTri::Yes
    );
    assert_eq!(
        cow_fs_from_fs_kind_and_options(EnvFsKind::Apfs, "rw"),
        EnvTri::Yes
    );

    assert_eq!(
        cow_fs_from_fs_kind_and_options(EnvFsKind::Ext4, "rw"),
        EnvTri::No
    );
    assert_eq!(
        cow_fs_from_fs_kind_and_options(EnvFsKind::Tmpfs, "rw"),
        EnvTri::No
    );
    assert_eq!(
        cow_fs_from_fs_kind_and_options(EnvFsKind::Ntfs, "rw"),
        EnvTri::No
    );

    assert_eq!(
        cow_fs_from_fs_kind_and_options(EnvFsKind::Xfs, "rw,reflink,attr2"),
        EnvTri::Yes
    );
    assert_eq!(
        cow_fs_from_fs_kind_and_options(EnvFsKind::Xfs, "rw,noreflink,attr2"),
        EnvTri::No
    );
    assert_eq!(
        cow_fs_from_fs_kind_and_options(EnvFsKind::Xfs, "rw,attr2"),
        EnvTri::Unknown
    );

    assert_eq!(
        cow_fs_from_fs_kind_and_options(EnvFsKind::Other, "rw"),
        EnvTri::Unknown
    );
    assert_eq!(
        cow_fs_from_fs_kind_and_options(EnvFsKind::Unknown, "rw"),
        EnvTri::Unknown
    );
}

#[test]
fn test_parse_proc_swaps_fixtures() {
    let active_swap = r#"
Filename				Type		Size		Used		Priority
/dev/dm-1                               partition	2097148		0		-2
"#;
    assert_eq!(parse_proc_swaps(active_swap), EnvTri::Yes);

    let empty_swap = r#"
Filename				Type		Size		Used		Priority
"#;
    assert_eq!(parse_proc_swaps(empty_swap), EnvTri::No);

    assert_eq!(parse_proc_swaps(""), EnvTri::No);

    let malformed_swap = r#"
Filename				Type		Size		Used		Priority
invalid_line_without_numbers
"#;
    assert_eq!(parse_proc_swaps(malformed_swap), EnvTri::Unknown);
}

#[test]
fn test_parse_limits_fixtures() {
    let limits_enabled = r#"
Limit                     Soft Limit           Hard Limit           Units
Max cpu time              unlimited            unlimited            seconds
Max file size             unlimited            unlimited            bytes
Max data size             unlimited            unlimited            bytes
Max stack size            8388608              unlimited            bytes
Max core file size        unlimited            unlimited            bytes
Max resident set          unlimited            unlimited            bytes
"#;
    assert_eq!(parse_limits(limits_enabled), EnvTri::Yes);

    let limits_disabled = r#"
Limit                     Soft Limit           Hard Limit           Units
Max core file size        0                    unlimited            bytes
"#;
    assert_eq!(parse_limits(limits_disabled), EnvTri::No);

    let limits_malformed = "Max core file size";
    assert_eq!(parse_limits(limits_malformed), EnvTri::Unknown);

    assert_eq!(parse_limits(""), EnvTri::Unknown);
}

#[test]
fn test_parse_core_pattern_fixtures() {
    let pipe_pat = "|/usr/lib/systemd/systemd-coredump %P %u %g %s %t %c %h";
    assert_eq!(parse_core_pattern(pipe_pat), EnvTri::Yes);

    let simple_pat = "core";
    assert_eq!(parse_core_pattern(simple_pat), EnvTri::Yes);

    assert_eq!(parse_core_pattern("   \n\t "), EnvTri::Unknown);
}

#[test]
fn test_parse_dm_uuid_fixtures() {
    let crypt_uuid = "CRYPT-LUKS2-66b03914a1e9447e852a4f49b10c85ee-dm_crypt_0\n";
    assert_eq!(parse_dm_uuid(crypt_uuid), EnvTri::Yes);

    let lvm_uuid = "LVM-8f2e71c9b20d4f3a8b2c1d0e9f8a7b6c5d4e3f2a1b";
    assert_eq!(parse_dm_uuid(lvm_uuid), EnvTri::No);

    assert_eq!(parse_dm_uuid(""), EnvTri::Unknown);
    assert_eq!(parse_dm_uuid("unknown_format"), EnvTri::Unknown);
}

#[test]
fn test_parse_rotational_fixtures() {
    assert_eq!(parse_rotational("1\n"), EnvTri::Yes);
    assert_eq!(parse_rotational("0"), EnvTri::No);
    assert_eq!(parse_rotational("invalid"), EnvTri::Unknown);
}

#[test]
fn test_observe_tempdir_no_panic() {
    let temp_dir = std::env::temp_dir();
    let obs = observe(&temp_dir);

    assert!(matches!(
        obs.os,
        EnvOsKind::Linux | EnvOsKind::MacOs | EnvOsKind::Windows | EnvOsKind::Other
    ));

    assert!(matches!(
        obs.cow_fs,
        EnvTri::Yes | EnvTri::No | EnvTri::Unknown
    ));
    assert!(matches!(
        obs.discard_enabled,
        EnvTri::Yes | EnvTri::No | EnvTri::Unknown
    ));
    assert!(matches!(
        obs.block_encrypted,
        EnvTri::Yes | EnvTri::No | EnvTri::Unknown
    ));
    assert!(matches!(
        obs.rotational,
        EnvTri::Yes | EnvTri::No | EnvTri::Unknown
    ));
    assert!(matches!(
        obs.swap_active,
        EnvTri::Yes | EnvTri::No | EnvTri::Unknown
    ));
    assert!(matches!(
        obs.core_dumps_possible,
        EnvTri::Yes | EnvTri::No | EnvTri::Unknown
    ));
}

#[test]
fn test_u8_encoding_values() {
    assert_eq!(EnvTri::Unknown.to_u8(), 0);
    assert_eq!(EnvTri::Yes.to_u8(), 1);
    assert_eq!(EnvTri::No.to_u8(), 2);

    assert_eq!(EnvOsKind::Unknown.to_u8(), 0);
    assert_eq!(EnvOsKind::Linux.to_u8(), 1);
    assert_eq!(EnvOsKind::MacOs.to_u8(), 2);
    assert_eq!(EnvOsKind::Windows.to_u8(), 3);
    assert_eq!(EnvOsKind::Other.to_u8(), 4);

    assert_eq!(EnvFsKind::Unknown.to_u8(), 0);
    assert_eq!(EnvFsKind::Ext4.to_u8(), 1);
    assert_eq!(EnvFsKind::Xfs.to_u8(), 2);
    assert_eq!(EnvFsKind::Btrfs.to_u8(), 3);
    assert_eq!(EnvFsKind::Zfs.to_u8(), 4);
    assert_eq!(EnvFsKind::Tmpfs.to_u8(), 5);
    assert_eq!(EnvFsKind::Apfs.to_u8(), 6);
    assert_eq!(EnvFsKind::Ntfs.to_u8(), 7);
    assert_eq!(EnvFsKind::Other.to_u8(), 8);
}

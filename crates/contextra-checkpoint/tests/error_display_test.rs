use contextra_checkpoint::CheckpointError;
use contextra_types::ContextraError;

#[test]
fn test_checkpoint_error_display_parity_with_contextra_error() {
    let cases: Vec<(CheckpointError, ContextraError)> = vec![
        (
            CheckpointError::Internal("test logic err".into()),
            ContextraError::Internal("test logic err".into()),
        ),
        (
            CheckpointError::InvalidInput("bad input".into()),
            ContextraError::InvalidInput("bad input".into()),
        ),
        (
            CheckpointError::CheckpointNotFound,
            ContextraError::CheckpointNotFound,
        ),
        (
            CheckpointError::Transaction("tx failed".into()),
            ContextraError::Transaction("tx failed".into()),
        ),
        (
            CheckpointError::Serialization("json err".into()),
            ContextraError::Serialization("json err".into()),
        ),
        (
            CheckpointError::CrossDeviceLink {
                source_path: "/dev/sda1/a.sst".into(),
                target_path: "/dev/sdb1/a.sst".into(),
            },
            ContextraError::CrossDeviceLink {
                source_path: "/dev/sda1/a.sst".into(),
                target_path: "/dev/sdb1/a.sst".into(),
            },
        ),
        (
            CheckpointError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found")),
            ContextraError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found")),
        ),
    ];

    for (ckpt_err, ctx_err) in cases {
        assert_eq!(
            ckpt_err.to_string(),
            ctx_err.to_string(),
            "Display strings must match bit-identical"
        );
        let converted_ctx: ContextraError = ckpt_err.into();
        assert_eq!(converted_ctx.to_string(), ctx_err.to_string());
    }
}

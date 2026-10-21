use contextra_mcp::ipc::server::IpcServer;
use contextra_mcp::config::IpcConfig;
use std::path::Path;

#[tokio::test]
async fn windows_ipc_unsupported_returns_err() {
    let server = IpcServer::new(Path::new("dummy_pipe"), IpcConfig::default());
    let handler = |_s: String| {
        Box::pin(async { Ok(None) }) as contextra_ports::BoxFuture<'static, Result<Option<String>, contextra_types::ContextraError>>
    };

    #[cfg(windows)]
    {
        let res = server.run_windows_listener(handler).await;
        assert!(
            res.is_err(),
            "run_windows_listener MUST return Err(ContextraError::Unsupported)"
        );
        match res.unwrap_err() {
            contextra_types::ContextraError::Unsupported(msg) => {
                assert!(msg.contains("Windows Named Pipe IPC"));
            }
            other => panic!("Expected Unsupported error, got {other:?}"),
        }
    }

    #[cfg(not(windows))]
    {
        let _ = server;
        let _ = handler;
    }
}

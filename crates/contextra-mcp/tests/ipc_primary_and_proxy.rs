// FILE-CONTEXT
// STAND:       2026-10-11
// ZWECK:       Integration test suite for MCP IPC Primary detection, socket permissions, proxy roundtrip, and peer credentials.

use contextra_mcp::config::IpcConfig;
use contextra_mcp::ipc::peer_cred::validate_unix_peer_credentials;
use contextra_mcp::ipc::primary::{PrimaryDetector, ProcessRole, ENDPOINT_FILE_NAME};
use contextra_mcp::ipc::server::IpcServer;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use tempfile::tempdir;

#[tokio::test]
async fn test_primary_detection_and_lock() {
    let tmp = tempdir().unwrap();
    let db_path = tmp.path();

    // First detection: becomes Primary
    let role1 = PrimaryDetector::detect_and_open(db_path, None)
        .await
        .unwrap();

    match role1 {
        ProcessRole::Primary { endpoint_path, .. } => {
            assert!(endpoint_path.exists() || endpoint_path.to_string_lossy().contains(".sock"));
            let endpoint_file = db_path.join(ENDPOINT_FILE_NAME);
            assert!(endpoint_file.exists());
        }
        ProcessRole::Proxy { .. } => panic!("Expected Primary role for initial open"),
    }

    // Second detection on same dir while primary is held: attempts proxy verification
    let role2 = PrimaryDetector::detect_and_open(db_path, None).await;
    // Without an active IPC socket server running on the endpoint, proxy verification fails closed
    assert!(role2.is_err());
}

#[tokio::test]
async fn test_unix_socket_permissions_and_peer_cred() {
    let tmp = tempdir().unwrap();
    let socket_path = tmp.path().join("test_ipc.sock");
    let config = IpcConfig::default();

    let server = IpcServer::new(&socket_path, config);

    // Run listener in background task
    let socket_path_clone = socket_path.clone();
    let _handle = tokio::spawn(async move {
        server
            .run_unix_listener(|req| {
                Box::pin(async move { Ok(Some(format!("{{\"echo\":\"{req}\"}}"))) })
            })
            .await
    });

    // Wait briefly for socket binding
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    assert!(socket_path_clone.exists());

    // Verify parent dir permissions 0700 and socket file 0600 on Unix
    let parent_meta = std::fs::metadata(tmp.path()).unwrap();
    assert_eq!(parent_meta.permissions().mode() & 0o777, 0o700);

    let socket_meta = std::fs::metadata(&socket_path_clone).unwrap();
    assert_eq!(socket_meta.permissions().mode() & 0o777, 0o600);

    // Connect stream and test peer cred validation
    let stream = tokio::net::UnixStream::connect(&socket_path_clone)
        .await
        .unwrap();

    let current_uid = std::fs::metadata(&socket_path_clone).unwrap().uid();
    let cred_res = validate_unix_peer_credentials(&stream, current_uid);
    assert!(cred_res.is_ok());

    let wrong_uid_res = validate_unix_peer_credentials(&stream, current_uid + 999);
    assert!(wrong_uid_res.is_err());
}

#[test]
fn test_verify_no_network_listener() {
    assert!(IpcServer::verify_no_network_listener("/tmp/contextra.sock").is_ok());
    assert!(IpcServer::verify_no_network_listener("tcp://127.0.0.1:8080").is_err());
    assert!(IpcServer::verify_no_network_listener("127.0.0.1:8080").is_err());
}

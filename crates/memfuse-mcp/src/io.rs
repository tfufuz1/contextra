use tokio::io::AsyncBufReadExt;

/// Maximum allowed single JSON-RPC message size via stdio (4 MB).
pub const MAX_RPC_BYTES: usize = 4 * 1024 * 1024;
/// Maximum allowed search query length in bytes (64 KB).
pub const MAX_SEARCH_QUERY_BYTES: usize = 64 * 1024;

// AI-TAG[SMELL][RESOLVED] Missing inactivity timeout on stdio read_line_bounded (ID: AGT-MCP-782aa62e) (TS: 2026-09-13T14:29:07Z) (SESSION: 23626761)
// BEFUND: read_line_bounded caps byte size at 4MB but has no idle read timeout.
// RISIKO: Slowloris-style partial request streams can hold task handles open indefinitely.
// EMPFEHLUNG: Wrap read_line_bounded invocations with tokio::time::timeout in run_stdio loop.

/// Reads a single line from an async reader into `buf` up to `max_bytes`.
/// If the line exceeds `max_bytes`, consumes and discards the remainder of the line without allocating memory and returns `InvalidData`.
pub async fn read_line_bounded<R: tokio::io::AsyncBufRead + Unpin>(
    reader: &mut R,
    buf: &mut String,
    max_bytes: usize,
) -> std::io::Result<usize> {
    buf.clear();
    let mut raw_bytes = Vec::new();

    loop {
        let available = reader.fill_buf().await?;
        if available.is_empty() {
            break;
        }

        let (done, used) = if let Some(i) = available.iter().position(|&b| b == b'\n') {
            (true, i + 1)
        } else {
            (false, available.len())
        };

        if raw_bytes.len() + used > max_bytes {
            reader.consume(used);
            // Drain remaining line from reader to avoid leaving unconsumed bytes
            if !done {
                loop {
                    let avail = reader.fill_buf().await?;
                    if avail.is_empty() {
                        break;
                    }
                    if let Some(pos) = avail.iter().position(|&b| b == b'\n') {
                        reader.consume(pos + 1);
                        break;
                    } else {
                        let len = avail.len();
                        reader.consume(len);
                    }
                }
            }
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Message size limit exceeded ({max_bytes} bytes limit)"),
            ));
        }

        raw_bytes.extend_from_slice(&available[..used]);
        reader.consume(used);

        if done {
            break;
        }
    }

    if raw_bytes.is_empty() {
        return Ok(0);
    }

    match String::from_utf8(raw_bytes) {
        Ok(s) => {
            let len = s.len();
            *buf = s;
            Ok(len)
        }
        Err(e) => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid UTF-8: {e}"),
        )),
    }
}

//! Channel types distinguishing host stdio vs local IPC transport channels.

/// Marker indicating the source channel through which a request entered the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Channel {
    /// Request originated from host stdio JSON-RPC transport.
    HostStdio,
    /// Request originated from local IPC (Unix Domain Socket / Named Pipe) proxy transport.
    LocalIpc,
}

impl Channel {
    /// Returns true if the channel is local IPC.
    #[must_use]
    pub fn is_ipc(&self) -> bool {
        matches!(self, Self::LocalIpc)
    }

    /// Returns true if the channel is host stdio.
    #[must_use]
    pub fn is_host_stdio(&self) -> bool {
        matches!(self, Self::HostStdio)
    }
}

/// Configures channel bounds for framing and buffer capacities.
pub fn configure_channel_bounds(
    max_line_bytes: usize,
) -> Result<usize, contextra_types::ContextraError> {
    if max_line_bytes == 0 {
        return Err(contextra_types::ContextraError::InvalidInput(
            "Channel line limit must be greater than 0".into(),
        ));
    }
    Ok(max_line_bytes)
}

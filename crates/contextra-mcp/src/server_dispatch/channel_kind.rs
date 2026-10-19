#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChannelKind {
    #[default]
    HostStdio,
    LocalIpc,
}

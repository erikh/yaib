use crate::config::CommandItem;
use anyhow::Result;
use tokio::{io::AsyncReadExt, net::UnixListener, sync::mpsc::UnboundedSender};

pub const SOCKET_PATH: &str = "/tmp/yaib.sock";

pub async fn manage_unix_socket(blocks: UnboundedSender<CommandItem>) -> Result<()> {
    let _ = std::fs::remove_file(SOCKET_PATH);
    let listener = UnixListener::bind(SOCKET_PATH)?;

    while let Ok((mut stream, _)) = listener.accept().await {
        let mut v = Vec::new();
        stream.read_to_end(&mut v).await?;
        drop(stream);
        if let Ok(block) = serde_json::from_slice(&v) {
            blocks.send(block)?;
        }
    }

    Ok(())
}

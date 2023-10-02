use crate::bar::Block;
use anyhow::Result;
use tokio::{io::AsyncReadExt, net::UnixListener, sync::mpsc::UnboundedSender};

pub async fn manage_unix_socket(blocks: UnboundedSender<Block>) -> Result<()> {
    let _ = std::fs::remove_file("/tmp/yaib.sock");
    let listener = UnixListener::bind("/tmp/yaib.sock")?;

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

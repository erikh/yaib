use anyhow::Result;
use std::path::PathBuf;
use yaib::{bar::Bar, config::Config};

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load(PathBuf::from(
        "/home/erikh/src/github.com/erikh/yaib/test.yaml",
    ))?;
    let bar = Bar::default();
    let mut b = bar.clone();
    let (s, r) = tokio::sync::mpsc::unbounded_channel();
    let c = config.clone();

    tokio::spawn(async move { b.emit_status(c, std::io::stdout(), r).await.unwrap() });

    loop {
        config.launch_collectors(s.clone()).await?;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

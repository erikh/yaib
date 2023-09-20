use anyhow::Result;
use std::path::PathBuf;
use yaib::{bar::Bar, config::Config};

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load(PathBuf::from("test.yaml"))?;
    let bar = Bar::default();
    let mut b = bar.clone();
    let (s, r) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move { b.emit_status(std::io::stdout(), r).await.unwrap() });
    loop {
        config.launch_collectors(s.clone()).await?;
        tokio::time::sleep(std::time::Duration::new(1, 0)).await;
    }
}

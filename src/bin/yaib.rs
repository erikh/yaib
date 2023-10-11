use anyhow::Result;
use std::path::PathBuf;
use tokio::{
    io::AsyncWriteExt,
    sync::mpsc::{unbounded_channel, UnboundedReceiver},
};
use yaib::{
    bar::{Bar, Block},
    config::Config,
    input::manage_clicks,
    state::ProtectedState,
    unix::{manage_unix_socket, SOCKET_PATH},
};

async fn manage_errors(mut r: UnboundedReceiver<Result<()>>) {
    while let Some(error) = r.recv().await {
        if let Err(error) = error {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    }
}

fn config_file() -> PathBuf {
    std::env::var("YAIB_CONFIG")
        .map(|x| x.into())
        .unwrap_or_else(|_| {
            dirs::config_local_dir()
                .map(|x| x.join("yaib"))
                .unwrap_or(dirs::home_dir().unwrap_or("/".into()))
                .join("yaib.config.yaml")
        })
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args();

    if let Some(cmd) = args.nth(1) {
        if cmd == "write-block" {
            if let Some(s) = args.next() {
                let _: Block = serde_json::from_str(&s)?; // just test that it parses
                let mut stream = tokio::net::UnixStream::connect(SOCKET_PATH).await?;
                stream.write_all(s.as_bytes()).await?;
                drop(stream);
            }

            return Ok(());
        }
    }

    let mut config = Config::load(config_file())?;
    let (s_collection, r_collection) = unbounded_channel();
    let (s_result, r_result) = unbounded_channel();
    let (s_blocks, r_blocks) = unbounded_channel();
    let c = config.clone();
    let state = ProtectedState::default();
    let mut bar = Bar::new(state.clone());

    tokio::spawn(async move { manage_unix_socket(s_blocks).await });
    tokio::spawn(async move {
        bar.emit_status(c, std::io::stdout(), r_collection, r_blocks)
            .await
            .unwrap()
    });
    tokio::spawn(async move { manage_errors(r_result).await });
    let c = config.clone();
    tokio::spawn(async move { manage_clicks(state, c).await });

    loop {
        config
            .launch_collectors(s_collection.clone(), s_result.clone())
            .await?;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

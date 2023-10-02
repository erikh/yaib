use anyhow::Result;
use std::path::PathBuf;
use tokio::{
    io::AsyncReadExt,
    sync::mpsc::{unbounded_channel, UnboundedReceiver},
};
use yaib::{
    bar::{Bar, NAME_PAGE_DOWN, NAME_PAGE_UP},
    config::Config,
    state::ProtectedState,
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

async fn manage_clicks(state: ProtectedState, config: Config) {
    let mut v = Vec::with_capacity(4096);
    while let Ok(_) = tokio::io::stdin().read_buf(&mut v).await {
        let mut lock = state.lock().await;

        if v.len() > 2 && v[0] as char == '[' && v[1] as char == '\n' {
            v = v[1..v.len()].to_vec();
        }

        if !v.is_empty() && v[v.len() - 1] as char == '\n' {
            if v[0] as char == ',' || v[0] as char == '[' {
                v = v[1..v.len() - 1].to_vec();
            } else {
                v = v[0..v.len() - 1].to_vec();
            }
        } else {
            continue;
        }

        if let Ok(click) = serde_json::from_slice::<yaib::bar::Click>(&v) {
            match click.name.as_str() {
                NAME_PAGE_UP => {
                    if lock.page < config.pages().len() - 1 {
                        lock.page += 1
                    }
                }
                NAME_PAGE_DOWN => {
                    if lock.page > 0 {
                        lock.page -= 1
                    }
                }
                _ => {
                    if lock.opened.contains(&click.name) {
                        let mut v = Vec::new();
                        for i in &lock.opened {
                            if *i != click.name {
                                v.push(i.clone())
                            }
                        }
                        lock.opened.clear();
                        lock.opened.append(&mut v);
                    } else {
                        lock.opened.push(click.name);
                    }
                }
            }
            v = Vec::new();
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load(config_file())?;
    let (s_collection, r_collection) = unbounded_channel();
    let (s_result, r_result) = unbounded_channel();
    let c = config.clone();
    let state = ProtectedState::default();
    let mut bar = Bar::new(state.clone());

    tokio::spawn(async move {
        bar.emit_status(c, std::io::stdout(), r_collection)
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

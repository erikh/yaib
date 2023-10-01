use anyhow::Result;
use std::path::PathBuf;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use yaib::{bar::Bar, config::Config, state::ProtectedState};

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

async fn manage_clicks(state: ProtectedState) {
    let mut first = true;
    let mut v = Vec::with_capacity(4096);
    while let Ok(_) = tokio::io::stdin().read_buf(&mut v).await {
        if first {
            if v.len() > 1 {
                v = v[1..v.len()].to_vec();
                first = false;
                continue;
            } else {
                continue;
            }
        }

        if v[v.len() - 1] as char == '\n' {
            if v.len() > 1 && !first {
                v = v[1..v.len() - 1].to_vec();
            }
        } else {
            continue;
        }

        if let Ok(click) = serde_json::from_slice::<yaib::bar::Click>(&v) {
            let mut lock = state.lock().await;
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
            drop(lock);
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
    tokio::spawn(async move { manage_clicks(state).await });

    loop {
        config
            .launch_collectors(s_collection.clone(), s_result.clone())
            .await?;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

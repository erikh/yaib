use crate::{
    bar::{Click, NAME_PAGE_DOWN, NAME_PAGE_UP},
    config::Config,
    state::ProtectedState,
};
use tokio::io::AsyncReadExt;

pub async fn manage_clicks(state: ProtectedState, config: Config) {
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

        if let Ok(click) = serde_json::from_slice::<Click>(&v) {
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

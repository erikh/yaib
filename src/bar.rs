use crate::{collectors::Collection, config::Config};
use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

pub const NAME_PAGE_UP: &str = "yaib-page-up";
pub const NAME_PAGE_DOWN: &str = "yaib-page-down";

#[derive(Debug, Clone, Default)]
pub struct Bar {
    state: BTreeMap<String, Block>,
    internal_state: crate::state::ProtectedState,
}

impl Bar {
    pub fn new(internal_state: crate::state::ProtectedState) -> Self {
        Self {
            state: BTreeMap::default(),
            internal_state,
        }
    }

    async fn add_page_blocks(&self, v: &mut Vec<Block>, pages: usize) {
        let page = self.internal_state.lock().await.page;

        if page != pages {
            v.push(Block {
                name: Some(NAME_PAGE_UP.to_string()),
                full_text: "▲".to_string(),
                ..Default::default()
            });
        }

        if page != 0 {
            v.push(Block {
                name: Some(NAME_PAGE_DOWN.to_string()),
                full_text: "▼".to_string(),
                ..Default::default()
            });
        }
    }

    pub async fn write_blocks(
        &self,
        mut w: impl std::io::Write + Send + 'static,
        mut data: UnboundedReceiver<Vec<Block>>,
    ) -> Result<()> {
        while let Some(v) = data.recv().await {
            serde_json::to_writer(&mut w, &v)?;
            w.write_all(",\n".as_bytes())?;
            w.flush()?;
        }

        Ok(())
    }

    pub async fn emit_status(
        &mut self,
        config: Config,
        mut w: impl std::io::Write + Send + 'static,
        mut data: UnboundedReceiver<Collection>,
    ) -> Result<()> {
        serde_json::to_writer(
            &mut w,
            &Header {
                version: 1,
                click_events: Some(true),
                ..Default::default()
            },
        )?;
        w.write_all("\n[\n".as_bytes())?;
        w.flush()?;

        let (s, r) = unbounded_channel();
        let obj = self.clone();
        tokio::spawn(async move { obj.write_blocks(w, r).await.unwrap() });

        let mut last_send = chrono::Local::now() - config.update_interval();
        let mut last_sent = Vec::new();

        while let Some(collection) = data.recv().await {
            let block = collection.to_block(self.internal_state.clone()).await?;
            self.state.insert(collection.name(), block);

            let now = chrono::Local::now();
            if last_send + config.update_interval() < now {
                let mut v = Vec::new();
                let items = &config.pages()[self.internal_state.lock().await.page].items();
                for item in items {
                    if let Some(block) = self.state.get(&item.name) {
                        v.push(block.clone())
                    }
                }

                self.add_page_blocks(&mut v, config.pages().len() - 1).await;

                if !last_sent.eq(&v) {
                    s.send(v.clone())?;
                    last_send = now;
                    last_sent = v;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Header {
    version: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_signal: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cont_signal: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    click_events: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Block {
    pub full_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_width: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urgent: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub separator: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub separator_block_width: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markup: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_top: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_bottom: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_left: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_right: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Click {
    pub name: String,
    pub instance: Option<String>,
    pub x: u16,
    pub y: u16,
    pub button: u16,
    pub relative_x: u16,
    pub relative_y: u16,
    pub output_x: u16,
    pub output_y: u16,
    pub width: u16,
    pub height: u16,
    pub modifiers: Vec<String>,
}

use crate::{collectors::Collection, config::Config};
use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

#[derive(Debug, Clone, Default)]
pub struct Bar {
    state: BTreeMap<String, Block>,
}

impl Bar {
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
            let block = collection.to_block();
            self.state.insert(collection.name(), block);

            let now = chrono::Local::now();
            if last_send + config.update_interval() < now {
                let mut v = Vec::new();
                let items = &config.pages()[0].items();
                for item in items {
                    if let Some(block) = self.state.get(&item.name) {
                        v.push(block.clone())
                    }
                }

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
    name: String,
    instance: String,
    x: u16,
    y: u16,
    button: u16,
}

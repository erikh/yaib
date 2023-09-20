use crate::collectors::Collection;
use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tokio::sync::mpsc::UnboundedReceiver;

#[derive(Debug, Clone, Default)]
pub struct Bar {
    state: BTreeMap<String, Block>,
}

impl Bar {
    pub async fn emit_status(
        &mut self,
        mut w: impl std::io::Write,
        mut data: UnboundedReceiver<Collection>,
    ) -> Result<()> {
        while let Some(collection) = data.recv().await {
            let block = collection.to_block();
            self.state.insert(collection.name(), block);
            writeln!(w, "{:?}", self.state)?;
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    pub separator: bool,
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

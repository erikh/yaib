use crate::bar::Block;
use anyhow::Result;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub struct Collection {
    name: String,
    collection_type: CollectionType,
}

impl Collection {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn collection_type(&self) -> CollectionType {
        self.collection_type.clone()
    }

    pub fn to_block(&self) -> Block {
        let mut block = Block::default();
        match &self.collection_type {
            CollectionType::Static(s) => block.full_text = s.clone(),
            CollectionType::Time(t) => block.full_text = t.format("%m/%d %H:%M").to_string(),
            _ => {}
        }

        block
    }
}

// every edit to this must mirror a ModuleType
#[derive(Debug, Clone)]
pub enum CollectionType {
    Static(String),
    CPU { count: usize, usage: f64 },
    Disk { total: usize, usage: usize },
    Memory { total: usize, usage: usize },
    Load(f64, f64, f64),
    Time(chrono::NaiveDateTime),
    Volume(usize),
}

pub async fn collect_static(
    s: UnboundedSender<Collection>,
    name: String,
    content: String,
) -> Result<()> {
    Ok(s.send(Collection {
        name,
        collection_type: CollectionType::Static(content),
    })?)
}

pub async fn collect_time(s: UnboundedSender<Collection>, name: String) -> Result<()> {
    Ok(s.send(Collection {
        name,
        collection_type: CollectionType::Time(chrono::Local::now().naive_local()),
    })?)
}

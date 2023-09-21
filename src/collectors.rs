use crate::bar::Block;
use anyhow::Result;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub struct Collection {
    name: String,
    value: Option<String>,
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
            CollectionType::Static => block.full_text = self.value.clone().unwrap(),
            CollectionType::Time(t) => {
                block.full_text = t
                    .format(&self.value.clone().unwrap_or("%m/%d %H:%M".to_string()))
                    .to_string()
            }
            CollectionType::Load(one, five, fifteen) => {
                let format = self.value.clone().unwrap_or("%1, %5, %15".to_string());
                let format = format.replace("%1", &one.to_string());
                let format = format.replace("%5", &five.to_string());
                let format = format.replace("%15", &fifteen.to_string());
                block.full_text = format
            }
            _ => {}
        }

        block.name = Some(self.name());

        block
    }
}

// every edit to this must mirror a ModuleType
#[derive(Debug, Clone)]
pub enum CollectionType {
    Static,
    CPU { count: usize, usage: f64 },
    Disk { total: usize, usage: usize },
    Memory { total: usize, usage: usize },
    Load(f64, f64, f64),
    Time(chrono::DateTime<chrono::Local>),
    Volume(usize),
}

pub async fn collect_static(
    s: UnboundedSender<Collection>,
    name: String,
    value: String,
) -> Result<()> {
    Ok(s.send(Collection {
        name,
        collection_type: CollectionType::Static,
        value: Some(value),
    })?)
}

pub async fn collect_time(
    s: UnboundedSender<Collection>,
    name: String,
    value: Option<String>,
    now: chrono::DateTime<chrono::Local>,
) -> Result<()> {
    Ok(s.send(Collection {
        name,
        collection_type: CollectionType::Time(now),
        value,
    })?)
}

pub async fn collect_load(
    s: UnboundedSender<Collection>,
    name: String,
    value: Option<String>,
) -> Result<()> {
    let avg = mprober_lib::load_average::get_load_average()?;

    Ok(s.send(Collection {
        name,
        collection_type: CollectionType::Load(avg.one, avg.five, avg.fifteen),
        value,
    })?)
}

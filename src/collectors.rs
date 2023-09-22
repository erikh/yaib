use crate::bar::Block;
use anyhow::{anyhow, Result};
use pretty_bytes::converter::convert;
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
            CollectionType::CPU { count, usage } => {
                let format = self
                    .value
                    .clone()
                    .unwrap_or("cpus: %count, usage: %usage".to_string());
                let format = format.replace("%count", &count.to_string());
                let format = format.replace("%usage", &format!("{:.2}", usage));
                block.full_text = format
            }
            CollectionType::Memory {
                total,
                usage,
                swap_total,
                swap_usage,
            } => {
                let format = self
                    .value
                    .clone()
                    .unwrap_or("total: %total, usage: %usage".to_string());
                let format = format.replace("%total", &convert(*total as f64));
                let format = format.replace("%usage", &convert(*usage as f64));
                let format = format.replace("%swap_total", &convert(*swap_total as f64));
                let format = format.replace("%swap_usage", &convert(*swap_usage as f64));
                block.full_text = format
            }
            CollectionType::Disk { total, usage } => {
                let format = "total: %total, usage: %usage";
                let format = format.replace("%total", &convert(*total as f64));
                let format = format.replace("%usage", &convert(*usage as f64));
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
    CPU {
        count: usize,
        usage: f64,
    },
    Disk {
        total: usize,
        usage: usize,
    },
    Memory {
        total: usize,
        usage: usize,
        swap_total: usize,
        swap_usage: usize,
    },
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

pub async fn collect_cpu(
    s: UnboundedSender<Collection>,
    name: String,
    value: Option<String>,
) -> Result<()> {
    let avg = mprober_lib::cpu::get_all_cpu_utilization_in_percentage(
        false,
        std::time::Duration::from_millis(100),
    )?;

    let count = avg.len();
    let avg = avg.iter().fold(0.0, |acc, item| item + acc) / count as f64;

    Ok(s.send(Collection {
        name,
        collection_type: CollectionType::CPU {
            count,
            usage: avg * 100.0,
        },
        value,
    })?)
}

pub async fn collect_memory(
    s: UnboundedSender<Collection>,
    name: String,
    value: Option<String>,
) -> Result<()> {
    let mem = mprober_lib::memory::free()?;

    Ok(s.send(Collection {
        name,
        collection_type: CollectionType::Memory {
            total: mem.mem.total,
            usage: mem.mem.used,
            swap_total: mem.swap.total,
            swap_usage: mem.swap.used,
        },
        value,
    })?)
}

pub async fn collect_disk(
    s: UnboundedSender<Collection>,
    name: String,
    value: Option<String>,
) -> Result<()> {
    if value.is_none() {
        return Err(anyhow!(
            "Value must be provided and must point at a mount point"
        ));
    }

    let stat = nix::sys::statvfs::statvfs(&std::path::PathBuf::from(&value.clone().unwrap()))?;

    Ok(s.send(Collection {
        name,
        collection_type: CollectionType::Disk {
            total: (stat.blocks() * stat.block_size()) as usize,
            usage: ((stat.blocks() - stat.blocks_available()) * stat.block_size()) as usize,
        },
        value,
    })?)
}

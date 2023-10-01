use crate::{
    bar::Block,
    config::ConfigItem,
    formatter::{Format, Rules},
};
use anyhow::{anyhow, Result};
use pretty_bytes::converter::convert;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub struct Collection {
    name: String,
    value: Option<String>,
    format: Option<String>,
    collection_type: CollectionType,
    item: crate::config::ConfigItem,
}

impl Collection {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn collection_type(&self) -> CollectionType {
        self.collection_type.clone()
    }

    fn get_formatter(&self) -> Format {
        let pair = match self.collection_type {
            CollectionType::Static => (self.value.clone().unwrap(), Rules::default()),
            CollectionType::Time(t) => (
                t.format(&self.format.clone().unwrap_or("%m/%d %H:%M".to_string()))
                    .to_string(),
                Rules::default(),
            ),
            CollectionType::Load(one, five, fifteen) => (
                self.format.clone().unwrap_or("%1, %5, %15".to_string()),
                vec![
                    ("%1", one.to_string()),
                    ("%5", five.to_string()),
                    ("%15", fifteen.to_string()),
                ],
            ),
            CollectionType::CPU { count, usage } => (
                self.format
                    .clone()
                    .unwrap_or("cpus: %count, usage: %usage".to_string()),
                vec![
                    ("%count", count.to_string()),
                    ("%usage", format!("{:.2}", usage)),
                ],
            ),
            CollectionType::Memory {
                total,
                usage,
                swap_total,
                swap_usage,
            } => (
                self.format
                    .clone()
                    .unwrap_or("total: %total, usage: %usage".to_string()),
                vec![
                    ("%total", convert(total as f64)),
                    ("%usage", convert(usage as f64)),
                    ("%swap_total", convert(swap_total as f64)),
                    ("%swap_usage", convert(swap_usage as f64)),
                    (
                        "%pct",
                        format!("{:.1}", (usage as f64 / total as f64) * 100.0),
                    ),
                    (
                        "%pct_swap",
                        format!("{:.1}", (swap_usage as f64 / swap_total as f64) * 100.0),
                    ),
                ],
            ),
            CollectionType::Disk { total, usage } => (
                self.format
                    .clone()
                    .unwrap_or("total: %total, usage: %usage".to_string()),
                vec![
                    ("%total", convert(total as f64)),
                    ("%usage", convert(usage as f64)),
                    (
                        "%pct",
                        format!("{:.1}", (usage as f64 / total as f64) * 100.0),
                    ),
                ],
            ),
        };
        Format::new(pair.0, pair.1)
    }

    pub async fn to_block(&self, state: crate::state::ProtectedState) -> Block {
        let mut block = Block::default();

        let pct = match self.collection_type {
            CollectionType::Static => 0,
            CollectionType::CPU { count: _, usage } => usage.floor() as u64,
            CollectionType::Disk { total, usage } => {
                ((usage as f64 / total as f64) * 100.0).floor() as u64
            }
            CollectionType::Load(one, ..) => {
                ((one / num_cpus::get() as f64) * 100.0).floor() as u64
            }
            CollectionType::Memory { total, usage, .. } => {
                ((usage as f64 / total as f64) * 100.0).floor() as u64
            }
            CollectionType::Time(..) => 0,
        };

        let urgency = if let Some(colors) = &self.item.urgency_colors {
            if let Some(urgency) = self.item.urgency {
                if pct > urgency.0.into() {
                    if pct > urgency.1.into() {
                        if pct > urgency.2.into() {
                            Some(colors.2.clone())
                        } else {
                            Some(colors.1.clone())
                        }
                    } else {
                        Some(colors.0.clone())
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        if let Some(urgency) = urgency {
            block.color = Some(urgency);
        }

        block.name = Some(self.name());

        if let Some(icon) = &self.item.icon {
            if state.lock().await.opened.contains(&self.name()) {
                block.full_text = self.get_formatter().format();
            } else {
                block.full_text = icon.clone()
            }
        } else {
            block.full_text = self.get_formatter().format();
        }

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
}

pub async fn collect_static(s: UnboundedSender<Collection>, item: ConfigItem) -> Result<()> {
    let clone = item.clone();
    Ok(s.send(Collection {
        name: item.name,
        collection_type: CollectionType::Static,
        value: item.value,
        format: item.format,
        item: clone,
    })?)
}

pub async fn collect_time(s: UnboundedSender<Collection>, item: ConfigItem) -> Result<()> {
    let clone = item.clone();
    Ok(s.send(Collection {
        name: item.name,
        collection_type: CollectionType::Time(chrono::Local::now()),
        value: item.value,
        format: item.format,
        item: clone,
    })?)
}

pub async fn collect_load(s: UnboundedSender<Collection>, item: ConfigItem) -> Result<()> {
    let avg = mprober_lib::load_average::get_load_average()?;
    let clone = item.clone();

    Ok(s.send(Collection {
        name: item.name,
        collection_type: CollectionType::Load(avg.one, avg.five, avg.fifteen),
        value: item.value,
        format: item.format,
        item: clone,
    })?)
}

pub async fn collect_cpu(s: UnboundedSender<Collection>, item: ConfigItem) -> Result<()> {
    let avg = mprober_lib::cpu::get_all_cpu_utilization_in_percentage(
        false,
        std::time::Duration::from_millis(100),
    )?;

    let count = avg.len();
    let avg = avg.iter().fold(0.0, |acc, item| item + acc) / count as f64;
    let clone = item.clone();

    Ok(s.send(Collection {
        name: item.name,
        collection_type: CollectionType::CPU {
            count,
            usage: avg * 100.0,
        },
        value: item.value,
        format: item.format,
        item: clone,
    })?)
}

pub async fn collect_memory(s: UnboundedSender<Collection>, item: ConfigItem) -> Result<()> {
    let mem = mprober_lib::memory::free()?;
    let clone = item.clone();

    Ok(s.send(Collection {
        name: item.name,
        collection_type: CollectionType::Memory {
            total: mem.mem.total,
            usage: mem.mem.used,
            swap_total: mem.swap.total,
            swap_usage: mem.swap.used,
        },
        value: item.value,
        format: item.format,
        item: clone,
    })?)
}

pub async fn collect_disk(s: UnboundedSender<Collection>, item: ConfigItem) -> Result<()> {
    let clone = item.clone();
    if let Some(value) = item.value {
        let vols = mprober_lib::volume::get_volumes()?;
        let mut target: Option<mprober_lib::volume::Volume> = None;

        for vol in vols {
            if vol.points.contains(&value) {
                target = Some(vol);
                break;
            }
        }

        if let Some(target) = target {
            Ok(s.send(Collection {
                name: item.name,
                collection_type: CollectionType::Disk {
                    total: target.size as usize,
                    usage: target.used as usize,
                },
                value: Some(value),
                format: item.format,
                item: clone,
            })?)
        } else {
            Err(anyhow!("Volume could not be found"))
        }
    } else {
        Err(anyhow!(
            "Value must be provided and must point at a mount point"
        ))
    }
}

use crate::collectors::*;
use anyhow::{anyhow, Result};
use chrono::Duration;
use fancy_duration::FancyDuration;
use serde_derive::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pages: Vec<ConfigPage>,
    update_interval: Option<FancyDuration<Duration>>,
}

impl Config {
    pub fn load(filename: std::path::PathBuf) -> Result<Self> {
        let mut io = std::fs::OpenOptions::new();
        io.read(true);
        let r = io.open(filename)?;
        Ok(serde_yaml::from_reader(r)?)
    }

    pub async fn launch_collectors(
        &self,
        s: UnboundedSender<Collection>,
        result: UnboundedSender<Result<()>>,
    ) -> Result<()> {
        for page in &self.pages {
            page.launch_collectors(s.clone(), result.clone()).await?;
        }

        Ok(())
    }

    pub fn pages(&self) -> Vec<ConfigPage> {
        self.pages.clone()
    }

    pub fn update_interval(&self) -> chrono::Duration {
        self.update_interval
            .clone()
            .unwrap_or(FancyDuration(chrono::Duration::seconds(1)))
            .duration()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigPage(Vec<ConfigItem>);

impl ConfigPage {
    pub fn items(&self) -> Vec<ConfigItem> {
        self.0.clone()
    }

    pub async fn launch_collectors(
        &self,
        s: UnboundedSender<Collection>,
        result: UnboundedSender<Result<()>>,
    ) -> Result<()> {
        for item in &self.0 {
            item.launch_collector(s.clone(), result.clone()).await?;
        }

        Ok(())
    }
}

// every edit to this must mirror a CollectionType
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum ModuleType {
    #[serde(rename = "static")]
    Static,
    #[serde(rename = "cpu")]
    CPU,
    #[serde(rename = "disk", alias = "hdd")]
    Disk,
    #[serde(rename = "memory", alias = "ram")]
    Memory,
    #[serde(rename = "load", alias = "load_average")]
    Load,
    #[serde(rename = "time", alias = "clock")]
    #[default]
    Time,
}

impl From<CollectionType> for ModuleType {
    fn from(value: CollectionType) -> Self {
        match value {
            CollectionType::Static => Self::Static,
            CollectionType::CPU { .. } => Self::CPU,
            CollectionType::Disk { .. } => Self::Disk,
            CollectionType::Memory { .. } => Self::Memory,
            CollectionType::Load(..) => Self::Load,
            CollectionType::Time(..) => Self::Time,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigItem {
    pub name: String,
    #[serde(rename = "type")]
    pub typ: ModuleType,
    pub value: Option<String>,
    pub format: Option<String>,
    pub urgency: Option<(u8, u8, u8)>,
    pub urgency_colors: Option<(String, String, String)>,
    pub icon: Option<String>,
}

async fn spawn(
    s: UnboundedSender<Result<()>>,
    f: impl std::future::Future<Output = Result<()>> + Send + 'static,
) -> Result<()> {
    Ok(s.send(tokio::spawn(f).await?)?)
}

impl ConfigItem {
    pub async fn launch_collector(
        &self,
        s: UnboundedSender<Collection>,
        result: UnboundedSender<Result<()>>,
    ) -> Result<()> {
        let clone = self.clone();
        match self.typ {
            ModuleType::Static => {
                if self.value.is_some() {
                    tokio::spawn(spawn(result, collect_static(s, clone)));
                } else {
                    return Err(anyhow!(
                        "Static self.clone() '{}' must have a value",
                        self.clone().name
                    ));
                }
            }
            ModuleType::Time => {
                tokio::spawn(spawn(result, collect_time(s, clone)));
            }
            ModuleType::Load => {
                tokio::spawn(spawn(result, collect_load(s, clone)));
            }
            ModuleType::CPU => {
                tokio::spawn(spawn(result, collect_cpu(s, clone)));
            }
            ModuleType::Memory => {
                tokio::spawn(spawn(result, collect_memory(s, clone)));
            }
            ModuleType::Disk => {
                tokio::spawn(spawn(result, collect_disk(s, clone)));
            }
        }

        Ok(())
    }
}

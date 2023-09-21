use crate::collectors::{collect_load, collect_static, collect_time, Collection, CollectionType};
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

    pub async fn launch_collectors(&self, s: UnboundedSender<Collection>) -> Result<()> {
        for page in &self.pages {
            page.launch_collectors(s.clone()).await?;
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

    pub async fn launch_collectors(&self, s: UnboundedSender<Collection>) -> Result<()> {
        let now = chrono::Local::now();
        for item in &self.0 {
            item.launch_collector(s.clone(), now).await?;
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
    #[serde(rename = "volume", alias = "audio")]
    Volume,
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
            CollectionType::Volume(..) => Self::Volume,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigItem {
    pub name: String,
    #[serde(rename = "type")]
    pub typ: ModuleType,
    pub value: Option<String>,
}

impl ConfigItem {
    pub async fn launch_collector(
        &self,
        s: UnboundedSender<Collection>,
        now: chrono::DateTime<chrono::Local>,
    ) -> Result<()> {
        match self.typ {
            ModuleType::Static => {
                if let Some(value) = &self.value {
                    tokio::spawn(collect_static(s, self.name.clone(), value.clone()));
                } else {
                    return Err(anyhow!("Static item '{}' must have a value", self.name));
                }
            }
            ModuleType::Time => {
                tokio::spawn(collect_time(s, self.name.clone(), self.value.clone(), now));
            }
            ModuleType::Load => {
                tokio::spawn(collect_load(s, self.name.clone(), self.value.clone()));
            }
            _ => {}
        }

        Ok(())
    }
}

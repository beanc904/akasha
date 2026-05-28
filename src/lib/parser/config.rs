/// Just using std file read is ok.
/// If there is something wrong with config read,
/// the whole process can not be initialized currectly.
use std::{
    error::Error,
    fs::{self},
    path::Path,
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MihomoConfig {
    #[serde(rename = "proxy-groups")]
    pub proxy_groups: Vec<ProxyGroup>,
}

#[derive(Debug, Deserialize)]
pub struct ProxyGroup {
    pub name: String,

    #[allow(unused)]
    #[serde(rename = "type")]
    pub group_type: String,

    pub proxies: Vec<String>,
}

impl MihomoConfig {
    /// Read config from [`path`]
    pub fn from_file<P>(path: P) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let contents = fs::read_to_string(path)?;
        Ok(serde_yaml::from_str(&contents)?)
    }

    /// Get the [`ProxyGroup`]s' names
    pub fn groups_name(&self) -> Vec<String> {
        self.proxy_groups
            .iter()
            .map(|proxy_group| proxy_group.name.to_string())
            .collect()
    }

    /// Get all the [`ProxyGroup::proxies`] details
    pub fn groups_proxies(&self) -> Vec<Vec<String>> {
        self.proxy_groups
            .iter()
            .map(|proxy_group| proxy_group.proxies.clone())
            .collect()
    }

    /// Get the count of [`ProxyGroup`]s
    pub fn group_count(&self) -> usize {
        self.proxy_groups.len()
    }
}

#[derive(Debug, Deserialize)]
pub struct AkashaConfig {
    #[serde(rename = "subscription-link")]
    subscription_link: String,
    #[serde(rename = "test-url")]
    test_url: Option<String>,
}

impl AkashaConfig {
    pub fn from_file<P>(path: P) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let contents = fs::read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }

    pub fn subscription_link(&self) -> String {
        self.subscription_link.clone()
    }

    pub fn test_url(&self) -> String {
        self.test_url
            .clone()
            .unwrap_or("http://cp.cloudflare.com/generate_204".to_string())
    }
}

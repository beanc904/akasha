/// Just using std file read is ok.
/// If there is something wrong with config read,
/// the whole process can not be initialized currectly.
use std::{error::Error, fs::File, io::Read};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MihomoConfig {
    #[serde(rename = "proxy-groups")]
    pub proxy_groups: Vec<ProxyGroup>,
}

#[derive(Debug, Deserialize)]
pub struct ProxyGroup {
    pub name: String,

    #[serde(rename = "type")]
    pub group_type: String,

    pub proxies: Vec<String>,
}

impl MihomoConfig {
    pub fn new(path: &'static str) -> Result<Self, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    pub fn get_proxy_groups_namevec(&self) -> Vec<String> {
        let mut vec = vec![];
        for proxy_group in self.proxy_groups.iter() {
            vec.push(proxy_group.name.clone());
        }
        vec
    }

    pub fn get_proxy_groups_proxies(&self) -> Vec<Vec<String>> {
        let mut groups = vec![];
        for proxy_group in self.proxy_groups.iter() {
            groups.push(proxy_group.proxies.clone());
        }
        groups
    }
}

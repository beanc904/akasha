use std::{error::Error, path::Path};

use chrono::prelude::*;
use http::{HeaderMap, header::USER_AGENT};
use reqwest::Client;
use tokio::{fs::File, io::AsyncWriteExt};

const CLASH_UA: &'static str = "clash-verge/v2.4.0";

#[derive(Debug)]
pub struct SubscriptionInfo {
    url: String,
    headers: HeaderMap,
    config_text: String,
    update_time: DateTime<Local>,
}

/// Unit: byte
#[derive(Debug)]
pub struct Usage {
    pub upload: u64,
    pub download: u64,
    pub total: u64,
    pub expire: Option<String>,
}

impl SubscriptionInfo {
    pub async fn new<S>(url: S) -> Result<SubscriptionInfo, Box<dyn Error + Send + Sync>>
    where
        S: AsRef<str>,
    {
        let url = url.as_ref();

        let client = Client::new();
        let resp = client.get(url).header(USER_AGENT, CLASH_UA).send().await?;

        let headers = resp.headers().clone();
        let text = resp.text().await?;

        Ok(SubscriptionInfo {
            url: url.to_string(),
            headers,
            config_text: text,
            update_time: Local::now(),
        })
    }

    pub async fn update(&mut self) -> Result<(), Box<dyn Error>> {
        let client = Client::new();
        let resp = client
            .get(&self.url)
            .header(USER_AGENT, CLASH_UA)
            .send()
            .await?;
        self.headers = resp.headers().clone();
        self.config_text = resp.text().await?;
        self.update_time = Local::now();

        Ok(())
    }

    pub fn get_updatetime(&self) -> DateTime<Local> {
        self.update_time.clone()
    }

    pub fn get_text(&self) -> String {
        self.config_text.clone()
    }

    pub async fn create_config<P>(&self, path: P) -> Result<(), Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let mut file = File::create(path).await?;
        file.write_all(self.config_text.as_bytes()).await?;
        file.flush().await?;
        Ok(())
    }

    pub fn parse_usage(&self) -> Option<Usage> {
        if let Some(header_value) = self.headers.get("subscription-userinfo") {
            let header_str = header_value.to_str().unwrap();

            let mut upload = 0;
            let mut download = 0;
            let mut total = 0;
            let mut expire: Option<String> = None;

            for part in header_str.split(';') {
                let mut kv = part.trim().splitn(2, '=');
                let key = kv.next().unwrap_or("").trim();
                let value = kv.next().unwrap_or("").trim();

                match key {
                    "upload" => upload = value.parse().unwrap_or(0),
                    "download" => download = value.parse().unwrap_or(0),
                    "total" => total = value.parse().unwrap_or(0),
                    "expire" => {
                        if !value.is_empty() {
                            expire = Some(value.to_string());
                        }
                    }
                    _ => {}
                }
            }

            Some(Usage {
                upload,
                download,
                total,
                expire,
            })
        } else {
            None
        }
    }
}

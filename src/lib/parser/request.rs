use std::{error::Error, path::Path};

use http::{HeaderMap, header::USER_AGENT};
use reqwest::Client;
use tokio::{fs::File, io::AsyncWriteExt};

#[derive(Debug)]
pub struct SubscriptionInfo {
    headers: HeaderMap,
    config_text: String,
}

/// Unit: byte
#[derive(Debug)]
pub struct UserInfo {
    pub upload: u64,
    pub download: u64,
    pub total: u64,
    pub expire: u64,
}

impl SubscriptionInfo {
    pub async fn new<S>(url: S) -> Result<SubscriptionInfo, Box<dyn Error>>
    where
        S: AsRef<str>,
    {
        let url = url.as_ref();

        let client = Client::new();
        let resp = client
            .get(url)
            .header(USER_AGENT, "clash-verge/v2.4.0")
            .send()
            .await?;

        let headers = resp.headers().clone();
        let text = resp.text().await?;

        Ok(SubscriptionInfo {
            headers,
            config_text: text,
        })
    }

    pub fn get_text(&self) -> String {
        self.config_text.clone()
    }

    pub async fn create_config(&self, path: Box<Path>) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(path).await?;
        file.write_all(self.config_text.as_bytes()).await?;
        file.flush().await?;
        Ok(())
    }

    pub fn parse_userinfo(&self) -> Option<UserInfo> {
        let header_value = self
            .headers
            .get("subscription-userinfo")
            .unwrap()
            .to_str()
            .unwrap();

        let mut upload = 0;
        let mut download = 0;
        let mut total = 0;
        let mut expire = 0;

        for part in header_value.split(';') {
            let mut kv = part.trim().split('=');
            let key = kv.next()?;
            let value = kv.next()?.parse::<u64>().ok()?;

            match key {
                "upload" => upload = value,
                "download" => download = value,
                "total" => total = value,
                "expire" => expire = value,
                _ => {}
            }
        }

        Some(UserInfo {
            upload,
            download,
            total,
            expire,
        })
    }
}

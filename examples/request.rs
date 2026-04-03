use std::{env, error::Error};

use dotenvy::dotenv;
use reqwest::header::USER_AGENT;
use tokio::{fs::File, io::AsyncWriteExt};

#[derive(Debug)]
struct SubscriptionInfo {
    upload: u64,
    download: u64,
    total: u64,
    expire: u64,
}

fn parse_subscription_userinfo(header_value: &str) -> Option<SubscriptionInfo> {
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

    Some(SubscriptionInfo {
        upload,
        download,
        total,
        expire,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let url = env::var("SUBSCRIPTION_LINK").unwrap();

    let client = reqwest::Client::new();

    let resp = client
        .get(url)
        .header(USER_AGENT, "clash-verge/v2.4.0")
        .send()
        .await?;

    // resp.text() will take the ownership of resp,
    // so use .clone() here.
    let headers = resp.headers().clone();

    let yaml_text = resp.text().await?;

    println!("===== YAML content =====");
    let mut file = File::create("config.yaml").await?;
    file.write_all(yaml_text.as_bytes()).await?;
    file.flush().await?;

    // Parse subscription-userinfo
    if let Some(value) = headers.get("subscription-userinfo") {
        let value_str = value.to_str()?;

        println!("\n===== the origin traffic info =====");
        println!("{}", value_str);

        if let Some(info) = parse_subscription_userinfo(value_str) {
            println!("\n===== after parse =====");
            println!("Upload   : {} MB", info.upload / 1024 / 1024);
            println!("Download : {} MB", info.download / 1024 / 1024);
            println!("Total    : {} MB", info.total / 1024 / 1024);
            println!("Expire   : {}", info.expire);

            let used = info.upload + info.download;
            let remain = info.total.saturating_sub(used);

            println!("Used     : {} MB", used / 1024 / 1024);
            println!("Remain   : {} MB", remain / 1024 / 1024);
        }
    } else {
        println!("\n Do not file subscription-userinfo.");
    }

    Ok(())
}

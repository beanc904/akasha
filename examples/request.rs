use std::env;

use akasha::parser::request::SubscriptionInfo;
use color_eyre::eyre::Result;
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let url = env::var("SUBSCRIPTION_LINK").unwrap();
    let subscription = SubscriptionInfo::new(url).await.unwrap();

    // let config_yaml = subscription.get_text();
    let update_time = subscription.get_updatetime();
    let usage = subscription.parse_usage();

    println!("Profile update time: {:?}", update_time);
    println!("{:?}", usage);

    Ok(())
}

use std::env;
use std::error::Error;

use akasha::parser::config::MihomoConfig;
use dotenvy::dotenv;

use akasha::client as ac;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    dotenv().ok();

    let sock_path = env::var("AKASHA_SOCKET_PATH").unwrap_or("/tmp/akasha/mihomo.sock".to_string());

    let mihomo = ac::Builder::new()
        .protocol(ac::Protocol::LocalSocket)
        .socket_path(sock_path)
        .pool_config(
            ac::IpcPoolConfigBuilder::new()
                .min_connections(0)
                .max_connections(20)
                .idle_timeout(std::time::Duration::from_millis(500))
                .health_check_interval(std::time::Duration::from_secs(10))
                .build(),
        )
        .build();

    let mihomo_config = MihomoConfig::new("config/config.yaml").unwrap();
    let proxies = mihomo_config.get_proxy_groups_proxies()[1].clone();

    if let Ok(mi) = &mihomo {
        let m = mi.read().await;
        let delay = m
            .delay_group("节点选择", "https://www.gstatic.com/generate_204", 5000)
            .await;
        for proxy in proxies.iter() {
            let selected_delay = if let Ok(delay) = &delay {
                Some(delay[proxy])
            } else {
                None
            };
            println!("Proxy name: {} >> Delay is: {:?}", proxy, selected_delay);
        }
        println!("Debug: {:?}", delay);
    }

    if let Ok(mi) = &mihomo {
        let m = mi.read().await;
        let proxy_name = "节点选择";
        let delay = m
            .delay_proxy_by_name(proxy_name, "https://www.gstatic.com/generate_204", 5000)
            .await;
        println!(
            "Single Proxy node name: {} >> Delay is: {:?}",
            proxy_name, delay
        );
    }

    Ok(())
}

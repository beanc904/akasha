use std::env;
use std::error::Error;

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

    if let Ok(mi) = &mihomo {
        let m = mi.read().await;
        let version = m.get_version().await;
        println!("Mihomo Version: {:?}", version);
    }

    Ok(())
}

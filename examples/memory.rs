use std::env;
use std::error::Error;

use dotenvy::dotenv;
use serde_json::Value;
use tokio::sync::mpsc;

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
        .build()
        .unwrap();

    let (tx, mut rx) = mpsc::channel::<Value>(64);

    let handle1;
    let handle2;

    handle1 = tokio::spawn(ac::ws_memory(mihomo, tx));

    handle2 = tokio::spawn(async move {
        println!(">>> Starting... >>>");
        while let Some(msg) = rx.recv().await {
            println!("Kernel Memory Occpuy: {}", msg);
        }
        println!("<<< Ending... <<<");
    });
    let _ = tokio::join!(handle1, handle2);

    Ok(())
}

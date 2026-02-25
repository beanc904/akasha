use std::env;
use std::error::Error;

use dotenvy::dotenv;

use akasha::client as ac;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    dotenv().ok();

    let sock_path = env::var("AKASHA_SOCKET_PATH").unwrap_or("/tmp/akasha/mihomo.sock".to_string());
    let args: Vec<String> = env::args().collect();

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
        println!("Mihomo version: {:?}", version);
    }

    {
        use serde_json::Value;
        use tokio::sync::mpsc;

        let (tx, mut rx) = mpsc::channel::<Value>(64);

        // invoke ws_traffic，input Sender
        let mihomo_clone = mihomo.unwrap().clone();
        let handle1;
        let handle2;

        if args.len() > 1 {
            match args[1].as_str() {
                "--memory" => {
                    handle1 = tokio::spawn(ac::ws_memory(mihomo_clone, tx));

                    handle2 = tokio::spawn(async move {
                        println!(">>> Starting... >>>");
                        while let Some(msg) = rx.recv().await {
                            println!("Kernel memory occpuy: {}", msg);
                        }
                        println!("<<< Ending... <<<");
                    });
                    let _ = tokio::join!(handle1, handle2);
                }
                "--traffic" => {
                    handle1 = tokio::spawn(ac::commands::ws_traffic(mihomo_clone, tx));

                    handle2 = tokio::spawn(async move {
                        println!(">>> Starting... >>>");
                        while let Some(msg) = rx.recv().await {
                            println!("Kernel traffic information: {}", msg);
                        }
                        println!("<<< Ending... <<<");
                    });
                    let _ = tokio::join!(handle1, handle2);
                }
                _ => {
                    println!("No such argument.");
                }
            }
        } else {
            println!("Run it without argument.");
        }
    }

    Ok(())
}

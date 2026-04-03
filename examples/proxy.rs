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

    // // Show the traffic chain.
    // if let Ok(mi) = &mihomo {
    //     let m = mi.read().await;
    //     let conn = m.get_connections().await;
    //     if let Ok(con) = conn {
    //         println!("Connection: {:?}", con);
    //     }
    // }

    // Check the proxy node information.
    if let Ok(mi) = &mihomo {
        let m = mi.read().await;
        let proxy = m.get_proxy_by_name("节点选择").await;
        if let Ok(proxy) = proxy {
            match proxy.now {
                Some(now) => {
                    if now == "自动选择" {
                        let detail = m.get_proxy_by_name("自动选择").await;
                        println!("Now the chosen proxy is: {}", detail.unwrap().now.unwrap());
                    } else {
                        println!("Now the chosen proxy is: {}", now);
                    }
                }
                None => eprintln!("Something wrong with the proxy."),
            }
        }
    }

    // Change the proxy node.
    if let Ok(mi) = &mihomo {
        let m = mi.read().await;
        let res = m.select_node_for_group("节点选择", "🇸🇬|新加坡-直连").await;
        if let Ok(_) = res {
            let proxy = m.get_proxy_by_name("节点选择").await;
            println!("Now the chosen proxy is: {}", proxy.unwrap().now.unwrap());
        }
    }

    Ok(())
}

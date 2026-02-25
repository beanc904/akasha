use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{RwLock, mpsc::Sender};

use crate::client::{error::Result, mihomo::Mihomo, models::*};

pub async fn update_controller(
    // state: State<'_, RwLock<Mihomo>>,
    mihomo: Arc<RwLock<Mihomo>>,
    host: Option<String>,
    port: Option<u16>,
) -> Result<()> {
    let mut mihomo = mihomo.write().await;
    mihomo.update_external_host(host);
    mihomo.update_external_port(port);
    Ok(())
}

pub async fn update_secret(mihomo: Arc<RwLock<Mihomo>>, secret: Option<String>) -> Result<()> {
    mihomo.write().await.update_secret(secret);
    Ok(())
}

pub async fn get_version(mihomo: Arc<RwLock<Mihomo>>) -> Result<MihomoVersion> {
    mihomo.read().await.get_version().await
}

pub async fn flush_fakeip(mihomo: Arc<RwLock<Mihomo>>) -> Result<()> {
    mihomo.read().await.flush_fakeip().await
}

pub async fn flush_dns(mihomo: Arc<RwLock<Mihomo>>) -> Result<()> {
    mihomo.read().await.flush_dns().await
}

// ANCHOR: connections
pub async fn get_connections(mihomo: Arc<RwLock<Mihomo>>) -> Result<Connections> {
    mihomo.read().await.get_connections().await
}

pub async fn close_all_connections(mihomo: Arc<RwLock<Mihomo>>) -> Result<()> {
    mihomo.read().await.close_all_connections().await
}

pub async fn close_connections(mihomo: Arc<RwLock<Mihomo>>, connection_id: String) -> Result<()> {
    mihomo.read().await.close_connection(&connection_id).await
}
// ANCHOR_END: connections

// ANCHOR: groups
pub async fn get_groups(mihomo: Arc<RwLock<Mihomo>>) -> Result<Groups> {
    mihomo.read().await.get_groups().await
}

pub async fn get_group_by_name(mihomo: Arc<RwLock<Mihomo>>, group_name: String) -> Result<Proxy> {
    mihomo.read().await.get_group_by_name(&group_name).await
}

pub async fn delay_group(
    mihomo: Arc<RwLock<Mihomo>>,
    group_name: String,
    test_url: String,
    timeout: u32,
    keep_fixed: bool,
) -> Result<HashMap<String, u32>> {
    let fixed = if keep_fixed {
        mihomo
            .read()
            .await
            .get_group_by_name(&group_name)
            .await?
            .fixed
    } else {
        None
    };
    log::debug!("delay group, fixed: {fixed:?}");
    let res = mihomo
        .read()
        .await
        .delay_group(&group_name, &test_url, timeout)
        .await?;
    if keep_fixed
        && let Some(fixed) = fixed
        && !fixed.is_empty()
    {
        mihomo
            .read()
            .await
            .select_node_for_group(&group_name, &fixed)
            .await?;
    }
    Ok(res)
}
// ANCHOR_END: groups

// ANCHOR: providers
pub async fn get_proxy_providers(mihomo: Arc<RwLock<Mihomo>>) -> Result<ProxyProviders> {
    mihomo.read().await.get_proxy_providers().await
}

pub async fn get_proxy_provider_by_name(
    mihomo: Arc<RwLock<Mihomo>>,
    provider_name: String,
) -> Result<ProxyProvider> {
    mihomo
        .read()
        .await
        .get_proxy_provider_by_name(&provider_name)
        .await
}

pub async fn update_proxy_provider(
    mihomo: Arc<RwLock<Mihomo>>,
    provider_name: String,
) -> Result<()> {
    mihomo
        .read()
        .await
        .update_proxy_provider(&provider_name)
        .await
}

pub async fn healthcheck_proxy_provider(
    mihomo: Arc<RwLock<Mihomo>>,
    provider_name: String,
) -> Result<()> {
    mihomo
        .read()
        .await
        .healthcheck_proxy_provider(&provider_name)
        .await
}

pub async fn healthcheck_node_in_provider(
    mihomo: Arc<RwLock<Mihomo>>,
    provider_name: String,
    proxy_name: String,
    test_url: String,
    timeout: u32,
) -> Result<ProxyDelay> {
    mihomo
        .read()
        .await
        .healthcheck_node_in_provider(&provider_name, &proxy_name, &test_url, timeout)
        .await
}
// ANCHOR_END: providers

// ANCHOR: proxies
pub async fn get_proxies(mihomo: Arc<RwLock<Mihomo>>) -> Result<Proxies> {
    mihomo.read().await.get_proxies().await
}

pub async fn get_proxy_by_name(mihomo: Arc<RwLock<Mihomo>>, proxy_name: String) -> Result<Proxy> {
    mihomo.read().await.get_proxy_by_name(&proxy_name).await
}

pub async fn select_node_for_group(
    mihomo: Arc<RwLock<Mihomo>>,
    group_name: String,
    node: String,
) -> Result<()> {
    mihomo
        .read()
        .await
        .select_node_for_group(&group_name, &node)
        .await
}

pub async fn unfixed_proxy(mihomo: Arc<RwLock<Mihomo>>, group_name: String) -> Result<()> {
    mihomo.read().await.unfixed_proxy(&group_name).await
}

pub async fn delay_proxy_by_name(
    mihomo: Arc<RwLock<Mihomo>>,
    proxy_name: String,
    test_url: String,
    timeout: u32,
) -> Result<ProxyDelay> {
    mihomo
        .read()
        .await
        .delay_proxy_by_name(&proxy_name, &test_url, timeout)
        .await
}
// ANCHOR_END: proxies

// ANCHOR: rules
pub async fn get_rules(mihomo: Arc<RwLock<Mihomo>>) -> Result<Rules> {
    mihomo.read().await.get_rules().await
}

pub async fn get_rule_providers(mihomo: Arc<RwLock<Mihomo>>) -> Result<RuleProviders> {
    mihomo.read().await.get_rule_providers().await
}

pub async fn update_rule_provider(
    mihomo: Arc<RwLock<Mihomo>>,
    provider_name: String,
) -> Result<()> {
    mihomo
        .read()
        .await
        .update_rule_provider(&provider_name)
        .await
}
// ANCHOR_END: rules

// ANCHOR: runtime config
pub async fn get_base_config(mihomo: Arc<RwLock<Mihomo>>) -> Result<BaseConfig> {
    mihomo.read().await.get_base_config().await
}

pub async fn reload_config(
    mihomo: Arc<RwLock<Mihomo>>,
    force: bool,
    config_path: String,
) -> Result<()> {
    mihomo.read().await.reload_config(force, &config_path).await
}

pub async fn patch_base_config(mihomo: Arc<RwLock<Mihomo>>, data: serde_json::Value) -> Result<()> {
    mihomo.read().await.patch_base_config(&data).await
}

pub async fn update_geo(mihomo: Arc<RwLock<Mihomo>>) -> Result<()> {
    mihomo.read().await.update_geo().await
}

pub async fn restart(mihomo: Arc<RwLock<Mihomo>>) -> Result<()> {
    mihomo.read().await.restart().await
}
// ANCHOR_END: runtime config

// ANCHOR: upgrade
pub async fn upgrade_core(
    mihomo: Arc<RwLock<Mihomo>>,
    channel: CoreUpdaterChannel,
    force: bool,
) -> Result<()> {
    mihomo.read().await.upgrade_core(channel, force).await
}

pub async fn upgrade_ui(mihomo: Arc<RwLock<Mihomo>>) -> Result<()> {
    mihomo.read().await.upgrade_ui().await
}

pub async fn upgrade_geo(mihomo: Arc<RwLock<Mihomo>>) -> Result<()> {
    mihomo.read().await.upgrade_geo().await
}
// ANCHOR_END: upgrade

// ANCHOR: mihomo websocket
pub async fn ws_traffic(
    mihomo: Arc<RwLock<Mihomo>>,
    sender: Sender<serde_json::Value>,
) -> Result<ConnectionId> {
    mihomo
        .read()
        .await
        .ws_traffic(move |data| {
            let _ = sender.try_send(data);
        })
        .await
}

pub async fn ws_memory(
    mihomo: Arc<RwLock<Mihomo>>,
    sender: Sender<serde_json::Value>,
) -> Result<ConnectionId> {
    mihomo
        .read()
        .await
        .ws_memory(move |data| {
            // let sender_clone = sender.clone();
            // tokio::spawn(async move {
            //     let _ = sender_clone.send(data).await;
            // });
            let _ = sender.try_send(data);
        })
        .await
}

pub async fn ws_connections(
    mihomo: Arc<RwLock<Mihomo>>,
    sender: Sender<serde_json::Value>,
) -> Result<ConnectionId> {
    mihomo
        .read()
        .await
        .ws_connections(move |data| {
            let _ = sender.try_send(data);
        })
        .await
}

pub async fn ws_logs(
    mihomo: Arc<RwLock<Mihomo>>,
    level: LogLevel,
    sender: Sender<serde_json::Value>,
) -> Result<ConnectionId> {
    mihomo
        .read()
        .await
        .ws_logs(level, move |data| {
            let _ = sender.try_send(data);
        })
        .await
}

// mihomo 的 websocket 应该只读取数据，没必要发送数据
// #[command]
// pub async fn ws_send(
//     mihomo: Arc<RwLock<Mihomo>>,
//     id: u32,
//     message: WebSocketMessage,
// ) -> Result<()> {
//     mihomo.read().await.send(id, message).await
// }

pub async fn ws_disconnect(
    mihomo: Arc<RwLock<Mihomo>>,
    id: ConnectionId,
    force_timeout: Option<u64>,
) -> Result<()> {
    mihomo.read().await.disconnect(id, force_timeout).await
}

pub async fn clear_all_ws_connections(mihomo: Arc<RwLock<Mihomo>>) -> Result<()> {
    mihomo.write().await.clear_all_ws_connections().await
}
// ANCHOR_END: mihomo websocket

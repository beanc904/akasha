mod ui;
mod utils;

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;

use aka_logger::{AkaLogger, LogStore, LoggerConfig};
use akasha::client::mihomo::Mihomo;
use akasha::parser::request::SubscriptionInfo;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use log::LevelFilter;
use ratatui::DefaultTerminal;
use ratatui::widgets::ListState;
use serde_json::Value;
use sysproxy::Sysproxy;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{Duration, interval};

use akasha::client as ac;
use akasha::parser::config::{AkashaConfig, MihomoConfig};

use crate::pkginfo::PkgInfo;

include!("app/statetypes.rs");

pub struct App {
    /// Is the application running?
    running: bool,
    // Event stream.
    event_stream: EventStream,
    // Mihomo handle.
    mihomo: Arc<RwLock<Mihomo>>,
    // Remember to fix it later!!!
    akasha_config: AkashaConfig,
    // Package informations.
    pkginfo: PkgInfo,
    sidebar_status: SidebarStatus,
    dashboard_status: DashboardStatus,
    proxies_status: ProxiesStatus,
    logs_status: LogsStatus,

    // ANCHOR: traffic data
    /// The touple signature is (tick, up, down, upTotal, downTotal). (unit: bps)
    ///
    /// The original ws_traffic data is:
    /// Object {"data": String("{\"up\":0,\"down\":0,\"upTotal\":0,\"downTotal\":0}\n"), "type": String("Text")}
    traffic_data: VecDeque<(f64, f64, f64, f64, f64)>,
    /// The original ws_memory data is:
    /// {"data":"{\"inuse\":41844736,\"oslimit\":0}\n","type":"Text"} (unit: b)
    memory_inuse: f64,
    /// It is the unit frame of traffic monitor, and also the x_axis.
    tick: f64,
    // ANCHOR_END: traffic data
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        let pkginfo = PkgInfo::new();
        let mihomo_config = MihomoConfig::new(pkginfo.get_mihomo_config()).unwrap();
        let akasha_config = AkashaConfig::new(pkginfo.get_akasha_config()).unwrap();
        App {
            running: true,
            event_stream: EventStream::default(),
            mihomo: ac::Builder::new()
                .protocol(ac::Protocol::LocalSocket)
                .socket_path(pkginfo.get_mihomo_socket().to_str().unwrap())
                .pool_config(
                    ac::IpcPoolConfigBuilder::new()
                        .min_connections(0)
                        .max_connections(20)
                        .idle_timeout(std::time::Duration::from_millis(500))
                        .health_check_interval(std::time::Duration::from_secs(10))
                        .build(),
                )
                .build()
                .unwrap(),
            sidebar_status: SidebarStatus {
                list_state: ListState::default().with_selected(Some(0)),
                list_items: vec![
                    "Dashboard",
                    "Proxies",
                    "Profiles",
                    "Connections",
                    "Rules",
                    "Logs",
                    "Test",
                    "Settings",
                ],
                current_page: CurrentPage::Dashboard,
            },
            dashboard_status: DashboardStatus {
                scrollbar_pos: 0,
                viewport_height: 0,
                titles: vec![
                    "Profiles",
                    "CurrentNode",
                    "NetworkSettings",
                    "ProxyMode",
                    "TrafficStats",
                    "WebsiteTests",
                    "IpInformation",
                    "ClashInfo",
                    "SystemInfo",
                ],
                sublabels: vec![
                    vec!["From: ", "Update Time: ", "Used / Total: "],
                    vec!["Selected: ", "Delay: "],
                    vec!["System Proxy: ", "Tun Mode: "],
                    vec!["Mode: "],
                    vec![
                        "Upload Speed: ",
                        "Download Speed: ",
                        "Uploaded: ",
                        "Downloaded: ",
                        "Active Connections: ",
                        "Core Usage: ",
                    ],
                    vec!["Apple: ", "GitHub: ", "Google: ", "YouTube: "],
                    vec![
                        "IP: ",
                        "ASN: ",
                        "ISP: ",
                        "ORG: ",
                        "Location: ",
                        "Timezone: ",
                    ],
                    vec![
                        "Core Version: ",
                        "System Proxy Address: ",
                        "Mixed Port: ",
                        "Uptime: ",
                        "Rules Count: ",
                    ],
                    vec![
                        "OS Info: ",
                        "Auto Launch: ",
                        "Running Mode: ",
                        "Last Check Update: ",
                        "Akasha Version: ",
                    ],
                ],
                subscription_info: None,
                selected_node_delay: 0,
                sysproxy: None,
            },
            proxies_status: ProxiesStatus {
                group_state: ListState::default().with_selected(Some(0)),
                group_items: mihomo_config
                    .get_proxy_groups_namevec()
                    .into_iter()
                    .map(|name| (name, 0))
                    .collect(),
                proxy_state: ListState::default().with_selected(Some(0)),
                proxy_items: mihomo_config.get_proxy_groups_proxies(),
                proxy_focus: false,
                delay: vec![None; mihomo_config.get_num_of_groups()],
                delay_mpsc: mpsc::channel::<(Option<HashMap<String, u32>>, usize)>(64),
            },
            logs_status: LogsStatus {
                log_state: AkaLogger::init(LoggerConfig {
                    buf_capacity: 10,
                    level: LevelFilter::Info,
                    with_stdout: true,
                    log_path: PathBuf::from("debug/info.log").into_boxed_path(),
                }),
                scrollbar_pos: (0, 0),
                step_len: 3,
            },
            traffic_data: VecDeque::default(),
            memory_inuse: 0f64,
            tick: 0f64,
            akasha_config,
            pkginfo,
        }
    }

    // ANCHOR: key handler events
    async fn enter_handler(&mut self) {
        match self.sidebar_status.current_page {
            CurrentPage::Dashboard => todo!(),
            CurrentPage::Proxies => self.proxies_status.enter_handler(self.mihomo.clone()).await,
            CurrentPage::Profiles => todo!(),
            CurrentPage::Connections => todo!(),
            CurrentPage::Rules => todo!(),
            CurrentPage::Logs => todo!(),
            CurrentPage::Test => todo!(),
            CurrentPage::Settings => todo!(),
        }
    }

    fn esc_handler(&mut self) {
        match self.sidebar_status.current_page {
            CurrentPage::Dashboard => todo!(),
            CurrentPage::Proxies => self.proxies_status.esc_handler(),
            CurrentPage::Profiles => todo!(),
            CurrentPage::Connections => todo!(),
            CurrentPage::Rules => todo!(),
            CurrentPage::Logs => todo!(),
            CurrentPage::Test => todo!(),
            CurrentPage::Settings => todo!(),
        }
    }

    fn l_handler(&mut self) {
        match self.sidebar_status.current_page {
            CurrentPage::Dashboard => todo!(),
            CurrentPage::Proxies => self.proxies_status.l_handler(),
            CurrentPage::Profiles => todo!(),
            CurrentPage::Connections => todo!(),
            CurrentPage::Rules => todo!(),
            CurrentPage::Logs => self.logs_status.l_handler(),
            CurrentPage::Test => todo!(),
            CurrentPage::Settings => todo!(),
        }
    }

    fn h_handler(&mut self) {
        match self.sidebar_status.current_page {
            CurrentPage::Dashboard => todo!(),
            CurrentPage::Proxies => self.proxies_status.h_handler(),
            CurrentPage::Profiles => todo!(),
            CurrentPage::Connections => todo!(),
            CurrentPage::Rules => todo!(),
            CurrentPage::Logs => self.logs_status.h_handler(),
            CurrentPage::Test => todo!(),
            CurrentPage::Settings => todo!(),
        }
    }

    fn j_handler(&mut self) {
        match self.sidebar_status.current_page {
            CurrentPage::Dashboard => self.dashboard_status.j_handler(),
            CurrentPage::Proxies => self.proxies_status.tab_switch(true),
            CurrentPage::Profiles => todo!(),
            CurrentPage::Connections => todo!(),
            CurrentPage::Rules => todo!(),
            CurrentPage::Logs => self.logs_status.j_handler(),
            CurrentPage::Test => todo!(),
            CurrentPage::Settings => todo!(),
        }
    }

    fn k_handler(&mut self) {
        match self.sidebar_status.current_page {
            CurrentPage::Dashboard => self.dashboard_status.k_handler(),
            CurrentPage::Proxies => self.proxies_status.tab_switch(false),
            CurrentPage::Profiles => todo!(),
            CurrentPage::Connections => todo!(),
            CurrentPage::Rules => todo!(),
            CurrentPage::Logs => self.logs_status.k_handler(),
            CurrentPage::Test => todo!(),
            CurrentPage::Settings => todo!(),
        }
    }

    async fn d_handler(&mut self) {
        match self.sidebar_status.current_page {
            CurrentPage::Dashboard => todo!(),
            CurrentPage::Proxies => self.proxies_status.d_handler(self.mihomo.clone()).await,
            CurrentPage::Profiles => todo!(),
            CurrentPage::Connections => todo!(),
            CurrentPage::Rules => todo!(),
            CurrentPage::Logs => todo!(),
            CurrentPage::Test => todo!(),
            CurrentPage::Settings => todo!(),
        }
    }

    async fn p_handler(&mut self) {
        match &mut self.dashboard_status.sysproxy {
            Some(sysproxy) => {
                sysproxy.enable = !sysproxy.enable;
                sysproxy.host = "127.0.0.1".into();
                sysproxy.port = 7890;

                sysproxy.set_system_proxy().unwrap();
                log::info!("Switched the system proxy status: {:?}", sysproxy);
            }
            None => log::error!("Something wrong with [`Sysproxy`] getting."),
        }
    }
    // ANCHOR_END: key handler events

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.running = true;
        // Renderint interval
        let mut ticker = interval(Duration::from_millis(1000 / 24));

        // ANCHOR: sidebar
        // ANCHOR: start the thread of traffic monitor
        let (tx_traffic, mut rx_traffic) = mpsc::channel::<Value>(64);
        let (tx_memory, mut rx_memory) = mpsc::channel::<Value>(64);
        let mihomo_traffic = self.mihomo.clone();
        let mihomo_memory = self.mihomo.clone();
        tokio::spawn(ac::ws_traffic(mihomo_traffic, tx_traffic));
        tokio::spawn(ac::ws_memory(mihomo_memory, tx_memory));
        // ANCHOR_END: start the thread of traffic monitor
        // ANCHOR_END: sidebar

        // ANCHOR: dashboard
        // ANCHOR: Initialize the subscription information.
        let (tx_subscription, mut rx_subscription) = mpsc::channel::<Option<SubscriptionInfo>>(64);
        if self.dashboard_status.subscription_info.is_none() {
            let url = self.akasha_config.subscription_link.clone();
            tokio::spawn(async move {
                let sub_info = SubscriptionInfo::new(url).await;
                let bundle = sub_info.ok();
                let _ = tx_subscription.send(bundle).await;
            });
        }
        // ANCHOR_END: Initialize the subscription information.

        // ANCHOR: setup the selected node delay info getter
        let (tx_node_delay, mut rx_node_delay) = mpsc::channel::<u32>(64);
        let mihomo_node_delay = self.mihomo.clone();
        let proxy_name = self.proxies_status.get_selected_node().clone();
        let test_url = self.akasha_config.test_url.clone().unwrap();
        let timeout = 5000;
        let mut ticker_node_delay_task = interval(Duration::from_secs(5));
        tokio::spawn(async move {
            loop {
                ticker_node_delay_task.tick().await;
                let mi = mihomo_node_delay.read().await;
                let delay = mi
                    .delay_proxy_by_name(&proxy_name, &test_url, timeout)
                    .await;
                let _ = tx_node_delay.send(delay.unwrap().delay).await;
            }
        });
        // ANCHOR_END: setup the selected node delay info getter

        // ANCHOR: setup the system proxy status server
        let (tx_sysproxy, mut rx_sysproxy) = mpsc::channel::<Sysproxy>(64);
        let mut ticker_sysproxy_task = interval(Duration::from_secs(5));
        tokio::spawn(async move {
            loop {
                ticker_sysproxy_task.tick().await;
                match Sysproxy::get_system_proxy() {
                    Ok(sysproxy) => {
                        let _ = tx_sysproxy.send(sysproxy).await;
                    }
                    Err(_) => {
                        log::info!("Something wrong with sysproxy getting.");
                    }
                }
            }
        });
        // ANCHOR_END: setup the system proxy status server
        // ANCHOR_END: dashboard

        // ANCHOR: proxies
        // ANCHOR: setup the group and proxy selected status in ui
        let (tx_proxies, mut rx_proxies) = mpsc::channel::<Vec<usize>>(64);
        let mihomo_proxies = self.mihomo.clone();
        let proxy_group = self.proxies_status.group_items.clone();
        let proxy_items = self.proxies_status.proxy_items.clone();
        // Set the interval of the proxy async check task.
        let mut ticker_proxy_task = interval(Duration::from_secs(5));
        tokio::spawn(async move {
            loop {
                ticker_proxy_task.tick().await;
                let mut selected_proxy: Vec<usize> = vec![];
                let mi = mihomo_proxies.read().await;
                for (i, group) in proxy_group.iter().enumerate() {
                    // Because there is a loop using of mihomo,
                    // via function from mihomo itself will be better.
                    let proxy = mi.get_proxy_by_name(group.0.as_str()).await;
                    if let Ok(proxy) = proxy {
                        match proxy.now {
                            Some(now) => {
                                let index =
                                    proxy_items[i].iter().position(|name| name == &now).unwrap();
                                selected_proxy.push(index);
                            }
                            None => {}
                        }
                    }
                }
                let _ = tx_proxies.send(selected_proxy).await;
            }
        });
        // ANCHOR_END: setup the group and proxy selected status in ui
        // ANCHOR_END: proxies

        while self.running {
            tokio::select! {
                _ = ticker.tick() => {
                    // Initialize subscription
                    if let Ok(bundle) = rx_subscription.try_recv() {
                        self.dashboard_status.subscription_info = bundle;
                    }

                    // traffic monitor thread recv
                    if let Ok(value) = rx_traffic.try_recv() {
                        log::trace!("Traffic try_recv(): {:?}", value);
                        let inner_json_data = value["data"].as_str().unwrap().trim();
                        let data: Value = serde_json::from_str(inner_json_data).unwrap();
                        let up = data["up"].as_f64().unwrap();
                        let down = data["down"].as_f64().unwrap();
                        let up_total = data["upTotal"].as_f64().unwrap();
                        let down_total = data["downTotal"].as_f64().unwrap();

                        self.tick += 1.0;

                        if self.traffic_data.len() >= 1024 {
                            self.traffic_data.pop_front();
                        }

                        self.traffic_data.push_back((self.tick, up, down, up_total, down_total));
                    }

                    // memory inuse thread recv
                    if let Ok(value) = rx_memory.try_recv() {
                        log::trace!("Memory try_recv(): {:?}", value);
                        let inner_json_data = value["data"].as_str().unwrap().trim();
                        let data: Value = serde_json::from_str(inner_json_data).unwrap();
                        self.memory_inuse = data["inuse"].as_f64().unwrap();
                    }

                    if let Ok(value) = rx_node_delay.try_recv() {
                        log::info!("Selected node delay: {}", value);
                        self.dashboard_status.selected_node_delay = value;
                    }

                    if let Ok(value) = rx_sysproxy.try_recv() {
                        log::trace!("Sysproxy try_recv(): {:?}", value);
                        self.dashboard_status.sysproxy = Some(value);
                    }

                    // proxies selected status thread recv
                    if let Ok(value) = rx_proxies.try_recv() {
                        log::trace!("Proxies Groups selected status: {:?}", value);
                        if value.len() == self.proxies_status.group_items.len() {
                            for (i, index) in value.iter().enumerate() {
                                self.proxies_status.group_items[i].1 = *index;
                            }
                        } else {
                            panic!("There is something wrong with group size.");
                        }
                    }

                    // proxies delay info thread recv
                    if let Ok(value) = self.proxies_status.delay_mpsc.1.try_recv() {
                        // // The index here maybe different from the index of 'd' press time.
                        // let group_index = self.proxies_status.group_state.selected().unwrap();
                        let (msg, group_index) = value;
                        self.proxies_status.delay[group_index] = msg;
                    }

                    terminal.draw(|frame| crate::app::ui::draw(&mut self, frame))?;
                    // self.handle_crossterm_events().await?;
                }
                maybe_event = self.event_stream.next() => {
                    if let Some(Ok(evt)) = maybe_event {
                        self.handle_crossterm_events(evt).await?;
                    }
                }
            }
        }

        // let _ = tokio::join!(handle_server, handle_client);
        Ok(())
    }

    /// Reads the crossterm events and updates the state of [`App`].
    async fn handle_crossterm_events(&mut self, evt: Event) -> color_eyre::Result<()> {
        // let event = self.event_stream.next().fuse().await;
        // match event {
        match evt {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key).await,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    async fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Tab) => self.sidebar_status.sidebar_next(),
            (_, KeyCode::BackTab) => self.sidebar_status.sidebar_previous(),
            (_, KeyCode::Char('j')) => self.j_handler(),
            (_, KeyCode::Char('k')) => self.k_handler(),
            (_, KeyCode::Char('h')) => self.h_handler(),
            (_, KeyCode::Char('l')) => self.l_handler(),
            (_, KeyCode::Enter) => self.enter_handler().await,
            (_, KeyCode::Esc) => self.esc_handler(),
            (_, KeyCode::Char('d')) => self.d_handler().await,
            (_, KeyCode::Char('p')) => self.p_handler().await,
            // Add other key handlers here.
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}

use std::borrow::Cow;
use std::collections::VecDeque;
use std::sync::Arc;

use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use ratatui::DefaultTerminal;
use ratatui::widgets::ListState;
use serde_json::Value;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{Duration, interval};

use akasha::client as ac;
use akasha::parser::config::MihomoConfig;

use crate::pkginfo::PkgInfo;

pub enum CurrentPage {
    Dashboard,
    Proxies,
    Profiles,
    Connections,
    Rules,
    Logs,
    Test,
    Settings,
}

pub enum DashboardTab {
    Profile,
    CurrentNode,
    NetworkSettings,
    ProxyMode,
    TrafficStats,
    WebsiteTests,
    IpInformation,
    ClashInfo,
    SystemInfo,
}

// #[derive(Debug, Default)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    // Event stream.
    pub event_stream: EventStream,
    // Mihomo handle.
    pub mihomo: Arc<RwLock<ac::mihomo::Mihomo>>,
    // Mihomo config.yaml
    pub mihomo_config: MihomoConfig,
    // Package informations.
    pub pkginfo: PkgInfo,
    // Sidebar selective status.
    pub sidebar_status: (ListState, Vec<&'static str>, CurrentPage),
    // Main area select tab.
    pub dashboard_status: (ListState, Vec<&'static str>, DashboardTab),
    // (0, 1, 2, 3)
    // 1: the state of proxy group 2: the names of each proxy group
    // 3: the state of group details 4: the detailed events of each group
    // 5: is focus of the selected proxy group
    pub proxies_status: (ListState, Vec<String>, ListState, Vec<Vec<String>>, bool),
    pub selected_proxy: (usize, usize),

    // ANCHOR: traffic data
    /// The touple signature is (tick, up, down, upTotal, downTotal). (unit: bps)
    ///
    /// The original ws_traffic data is:
    /// Object {"data": String("{\"up\":0,\"down\":0,\"upTotal\":0,\"downTotal\":0}\n"), "type": String("Text")}
    pub traffic_data: VecDeque<(f64, f64, f64, f64, f64)>,
    /// The original ws_memory data is:
    /// {"data":"{\"inuse\":41844736,\"oslimit\":0}\n","type":"Text"} (unit: b)
    pub memory_inuse: f64,
    /// It is the unit frame of traffic monitor, and also the x_axis.
    pub tick: f64,
    // ANCHOR_END: traffic data
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        let pkginfo = PkgInfo::new();
        let mihomo_config = MihomoConfig::new("resources/config.yaml").unwrap();
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
            sidebar_status: (
                ListState::default().with_selected(Some(0)),
                vec![
                    "Dashboard",
                    "Proxies",
                    "Profiles",
                    "Connections",
                    "Rules",
                    "Logs",
                    "Test",
                    "Settings",
                ],
                CurrentPage::Dashboard,
            ),
            dashboard_status: (
                ListState::default().with_selected(Some(0)),
                vec![
                    "Profile",
                    "Current Node",
                    "Network Settings",
                    "Proxy Mode",
                    "Traffic Stats",
                    "Website Tests",
                    "IP Information",
                    "Clash Info",
                    "System Info",
                ],
                DashboardTab::Profile,
            ),
            proxies_status: (
                ListState::default().with_selected(Some(0)),
                mihomo_config.get_proxy_groups_namevec(),
                ListState::default().with_selected(Some(0)),
                mihomo_config.get_proxy_groups_proxies(),
                false,
            ),
            selected_proxy: (0, 0),
            traffic_data: VecDeque::default(),
            memory_inuse: 0f64,
            tick: 0f64,
            mihomo_config,
            pkginfo,
        }
    }

    #[allow(dead_code)]
    fn get_selected_groupproxy(&self) -> (String, String) {
        let group_index = self.selected_proxy.0;
        let proxy_index = self.selected_proxy.1;
        let group_name = self.proxies_status.1[group_index].clone();
        let proxy_name = self.proxies_status.3[group_index][proxy_index].clone();
        (group_name, proxy_name)
    }

    /// You must use it after finishing selecting the current page.
    /// Sync the status of enumeration and liststate.
    fn update_liststate_status(&mut self) {
        match self.sidebar_status.0.selected() {
            Some(0) => self.sidebar_status.2 = CurrentPage::Dashboard,
            Some(1) => self.sidebar_status.2 = CurrentPage::Proxies,
            Some(2) => self.sidebar_status.2 = CurrentPage::Profiles,
            Some(3) => self.sidebar_status.2 = CurrentPage::Connections,
            Some(4) => self.sidebar_status.2 = CurrentPage::Rules,
            Some(5) => self.sidebar_status.2 = CurrentPage::Logs,
            Some(6) => self.sidebar_status.2 = CurrentPage::Test,
            Some(7) => self.sidebar_status.2 = CurrentPage::Settings,
            Some(_) => log::error!("Switching over array bound!"),
            None => log::error!("There is something wrong with switching page."),
        }

        match self.dashboard_status.0.selected() {
            Some(0) => self.dashboard_status.2 = DashboardTab::Profile,
            Some(1) => self.dashboard_status.2 = DashboardTab::CurrentNode,
            Some(2) => self.dashboard_status.2 = DashboardTab::NetworkSettings,
            Some(3) => self.dashboard_status.2 = DashboardTab::ProxyMode,
            Some(4) => self.dashboard_status.2 = DashboardTab::TrafficStats,
            Some(5) => self.dashboard_status.2 = DashboardTab::WebsiteTests,
            Some(6) => self.dashboard_status.2 = DashboardTab::IpInformation,
            Some(7) => self.dashboard_status.2 = DashboardTab::ClashInfo,
            Some(8) => self.dashboard_status.2 = DashboardTab::SystemInfo,
            Some(_) => log::error!("Switching over array bound!"),
            None => log::error!("There is something wrong with switching dashboard tab."),
        }
    }

    fn liststate_switch<'a, S>(is_next: bool, state: &mut ListState, items: &Vec<S>)
    where
        S: Into<Cow<'a, str>>,
    {
        match is_next {
            true => {
                // Switch to the next tab
                let index = match state.selected() {
                    Some(i) => {
                        if i >= items.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                state.select(Some(index));
            }
            false => {
                // Switch to the previous tab
                let index = match state.selected() {
                    Some(i) => {
                        if i == 0 {
                            items.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                state.select(Some(index));
            }
        }
    }

    pub fn sidebar_next(&mut self) {
        App::liststate_switch(true, &mut self.sidebar_status.0, &self.sidebar_status.1);
        self.update_liststate_status();
    }

    pub fn sidebar_previous(&mut self) {
        App::liststate_switch(false, &mut self.sidebar_status.0, &self.sidebar_status.1);
        self.update_liststate_status();
    }

    pub fn tab_switch(&mut self, is_next: bool) {
        match self.sidebar_status.2 {
            CurrentPage::Dashboard => {
                App::liststate_switch(
                    is_next,
                    &mut self.dashboard_status.0,
                    &self.dashboard_status.1,
                );
                self.update_liststate_status();
            }
            CurrentPage::Proxies => {
                if !self.proxies_status.4 {
                    // Now the focus is the groups list.
                    App::liststate_switch(
                        is_next,
                        &mut self.proxies_status.0,
                        &self.proxies_status.1,
                    );
                    // // It seems that it does not need to update enumeration.
                    // self.update_liststate_status();
                    // Reset the selected proxy item, each time switch the groups tab.
                    self.proxies_status.2.select(Some(0));
                } else {
                    // Now the focus is the details list.
                    let index = self.proxies_status.0.selected().unwrap();
                    App::liststate_switch(
                        is_next,
                        &mut self.proxies_status.2,
                        &self.proxies_status.3[index],
                    );
                }
            }
            CurrentPage::Profiles => todo!(),
            CurrentPage::Connections => todo!(),
            CurrentPage::Rules => todo!(),
            CurrentPage::Logs => todo!(),
            CurrentPage::Test => todo!(),
            CurrentPage::Settings => todo!(),
        }
    }

    pub fn enter_handler(&mut self) {
        match self.sidebar_status.2 {
            CurrentPage::Dashboard => todo!(),
            CurrentPage::Proxies => {
                if self.proxies_status.4 {
                    // Cursor at details
                    self.selected_proxy.0 = self.proxies_status.0.selected().unwrap();
                    self.selected_proxy.1 = self.proxies_status.2.selected().unwrap();
                } else {
                    // Cursor at tabs
                    self.proxies_status.4 = true;
                }
            }
            CurrentPage::Profiles => todo!(),
            CurrentPage::Connections => todo!(),
            CurrentPage::Rules => todo!(),
            CurrentPage::Logs => todo!(),
            CurrentPage::Test => todo!(),
            CurrentPage::Settings => todo!(),
        }
    }

    pub fn esc_handler(&mut self) {
        match self.sidebar_status.2 {
            CurrentPage::Dashboard => todo!(),
            CurrentPage::Proxies => {
                self.proxies_status.4 = false;
            }
            CurrentPage::Profiles => todo!(),
            CurrentPage::Connections => todo!(),
            CurrentPage::Rules => todo!(),
            CurrentPage::Logs => todo!(),
            CurrentPage::Test => todo!(),
            CurrentPage::Settings => todo!(),
        }
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.running = true;
        // Renderint interval
        let mut ticker = interval(Duration::from_millis(1000 / 24));

        // ANCHOR: start the thread of traffic monitor
        let (tx_traffic, mut rx_traffic) = mpsc::channel::<Value>(64);
        let (tx_memory, mut rx_memory) = mpsc::channel::<Value>(64);
        let mihomo_traffic = self.mihomo.clone();
        let mihomo_memory = self.mihomo.clone();
        tokio::spawn(ac::ws_traffic(mihomo_traffic, tx_traffic));
        tokio::spawn(ac::ws_memory(mihomo_memory, tx_memory));
        // ANCHOR_END: start the thread of traffic monitor

        while self.running {
            tokio::select! {
                _ = ticker.tick() => {
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

                    terminal.draw(|frame| crate::ui::draw(&mut self, frame))?;
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
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Tab) => self.sidebar_next(),
            (_, KeyCode::BackTab) => self.sidebar_previous(),
            (_, KeyCode::Char('j')) => self.tab_switch(true),
            (_, KeyCode::Char('k')) => self.tab_switch(false),
            (_, KeyCode::Enter) => self.enter_handler(),
            (_, KeyCode::Esc) => self.esc_handler(),
            // Add other key handlers here.
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}

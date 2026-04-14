use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use akasha::client::mihomo::Mihomo;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use ratatui::DefaultTerminal;
use ratatui::widgets::ListState;
use serde_json::Value;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{Duration, interval};

use akasha::client as ac;
use akasha::parser::config::{AkashaConfig, MihomoConfig};

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

fn liststate_switch(is_next: bool, state: &mut ListState, items_len: usize) {
    match is_next {
        true => {
            // Switch to the next tab
            let index = match state.selected() {
                Some(i) => {
                    if i >= items_len - 1 {
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
                        items_len - 1
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

pub struct SidebarStatus {
    pub list_state: ListState,
    pub list_items: Vec<&'static str>,
    pub current_page: CurrentPage,
}

impl SidebarStatus {
    /// You must use it after finishing selecting the current page.
    /// Sync the status of enumeration and liststate.
    fn update_liststate_status(&mut self) {
        match self.list_state.selected() {
            Some(0) => self.current_page = CurrentPage::Dashboard,
            Some(1) => self.current_page = CurrentPage::Proxies,
            Some(2) => self.current_page = CurrentPage::Profiles,
            Some(3) => self.current_page = CurrentPage::Connections,
            Some(4) => self.current_page = CurrentPage::Rules,
            Some(5) => self.current_page = CurrentPage::Logs,
            Some(6) => self.current_page = CurrentPage::Test,
            Some(7) => self.current_page = CurrentPage::Settings,
            Some(_) => log::error!("Switching over array bound!"),
            None => log::error!("There is something wrong with switching page."),
        }
    }

    fn sidebar_next(&mut self) {
        liststate_switch(true, &mut self.list_state, self.list_items.len());
        self.update_liststate_status();
    }

    fn sidebar_previous(&mut self) {
        liststate_switch(false, &mut self.list_state, self.list_items.len());
        self.update_liststate_status();
    }
}

pub struct ProxiesStatus {
    pub group_state: ListState,
    pub group_items: Vec<(String, usize)>,
    pub proxy_state: ListState,
    pub proxy_items: Vec<Vec<String>>,
    pub proxy_focus: bool,
    pub delay: Vec<Option<HashMap<String, u32>>>,
    pub delay_mpsc: (
        mpsc::Sender<(Option<HashMap<String, u32>>, usize)>,
        mpsc::Receiver<(Option<HashMap<String, u32>>, usize)>,
    ),
}

impl ProxiesStatus {
    fn tab_switch(&mut self, is_next: bool) {
        if !self.proxy_focus {
            // Now the focus is the groups list.
            liststate_switch(is_next, &mut self.group_state, self.group_items.len());
            // // It seems that it does not need to update enumeration.
            // self.update_liststate_status();
            // Reset the selected proxy item, each time switch the groups tab.
            self.proxy_state.select(Some(0));
        } else {
            // Now the focus is the details list.
            let index = self.group_state.selected().unwrap();
            liststate_switch(
                is_next,
                &mut self.proxy_state,
                self.proxy_items[index].len(),
            );
        }
    }

    async fn enter_handler(&mut self, mihomo: &Arc<RwLock<Mihomo>>) {
        if self.proxy_focus {
            // Cursor at details
            let index_group = self.group_state.selected().unwrap();
            let index_proxy = self.proxy_state.selected().unwrap();
            self.group_items[index_group].1 = index_proxy;

            let name_group = &self.group_items[index_group].0;
            let name_proxy = &self.proxy_items[index_group][index_proxy];

            let mihomo = mihomo.clone();
            let mi = mihomo.read().await;

            let _ = mi.select_node_for_group(name_group, name_proxy).await;
        } else {
            // Cursor at tabs
            self.proxy_focus = true;
        }
    }

    fn esc_handler(&mut self) {
        self.proxy_focus = false;
    }

    fn l_handler(&mut self) {
        self.proxy_focus = true;
    }

    fn h_handler(&mut self) {
        self.proxy_focus = false;
    }

    async fn d_handler(&mut self, mihomo: &Arc<RwLock<Mihomo>>) {
        let mihomo = mihomo.clone();
        let tx_delay = self.delay_mpsc.0.clone();
        let group_index = self.group_state.selected().unwrap();
        let group_name = self.group_items[group_index].0.clone();
        let test_url = "https://www.gstatic.com/generate_204".to_string();
        let timeout = 5000;
        let keep_fixed = true;
        tokio::spawn(async move {
            let delay =
                ac::delay_group(mihomo, group_name.clone(), test_url, timeout, keep_fixed).await;
            if let Ok(delay) = delay {
                let _ = tx_delay.send((Some(delay), group_index)).await;
            } else {
                log::error!("Encountered some problems with {} delay test.", group_name);
            }
        });
    }
}

// #[derive(Debug, Default)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    // Event stream.
    pub event_stream: EventStream,
    // Mihomo handle.
    pub mihomo: Arc<RwLock<Mihomo>>,
    // Mihomo config.yaml
    pub mihomo_config: MihomoConfig,
    pub akasha_config: AkashaConfig,
    // Package informations.
    pub pkginfo: PkgInfo,
    pub sidebar_status: SidebarStatus,
    pub proxies_status: ProxiesStatus,

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
    pub debug: String,
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
            traffic_data: VecDeque::default(),
            memory_inuse: 0f64,
            tick: 0f64,
            mihomo_config,
            akasha_config,
            pkginfo,
            debug: String::new(),
        }
    }

    // ANCHOR: key handler events
    pub async fn enter_handler(&mut self) {
        match self.sidebar_status.current_page {
            CurrentPage::Dashboard => todo!(),
            CurrentPage::Proxies => self.proxies_status.enter_handler(&self.mihomo).await,
            CurrentPage::Profiles => todo!(),
            CurrentPage::Connections => todo!(),
            CurrentPage::Rules => todo!(),
            CurrentPage::Logs => todo!(),
            CurrentPage::Test => todo!(),
            CurrentPage::Settings => todo!(),
        }
    }

    pub fn esc_handler(&mut self) {
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

    pub fn l_handler(&mut self) {
        match self.sidebar_status.current_page {
            CurrentPage::Dashboard => todo!(),
            CurrentPage::Proxies => self.proxies_status.l_handler(),
            CurrentPage::Profiles => todo!(),
            CurrentPage::Connections => todo!(),
            CurrentPage::Rules => todo!(),
            CurrentPage::Logs => todo!(),
            CurrentPage::Test => todo!(),
            CurrentPage::Settings => todo!(),
        }
    }

    pub fn h_handler(&mut self) {
        match self.sidebar_status.current_page {
            CurrentPage::Dashboard => todo!(),
            CurrentPage::Proxies => self.proxies_status.h_handler(),
            CurrentPage::Profiles => todo!(),
            CurrentPage::Connections => todo!(),
            CurrentPage::Rules => todo!(),
            CurrentPage::Logs => todo!(),
            CurrentPage::Test => todo!(),
            CurrentPage::Settings => todo!(),
        }
    }

    pub async fn d_hander(&mut self) {
        match self.sidebar_status.current_page {
            CurrentPage::Dashboard => todo!(),
            CurrentPage::Proxies => self.proxies_status.d_handler(&self.mihomo).await,
            CurrentPage::Profiles => todo!(),
            CurrentPage::Connections => todo!(),
            CurrentPage::Rules => todo!(),
            CurrentPage::Logs => todo!(),
            CurrentPage::Test => todo!(),
            CurrentPage::Settings => todo!(),
        }
    }
    // ANCHOR_END: key handler events

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
            (_, KeyCode::Char('j')) => self.proxies_status.tab_switch(true),
            (_, KeyCode::Char('k')) => self.proxies_status.tab_switch(false),
            (_, KeyCode::Char('h')) => self.h_handler(),
            (_, KeyCode::Char('l')) => self.l_handler(),
            (_, KeyCode::Enter) => self.enter_handler().await,
            (_, KeyCode::Esc) => self.esc_handler(),
            (_, KeyCode::Char('d')) => self.d_hander().await,
            // Add other key handlers here.
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}

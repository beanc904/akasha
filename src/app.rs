mod ui;
mod utils;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use aka_logger::{AkaLogger, LogStore, LoggerConfig};
use akasha::client::mihomo::Mihomo;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use log::LevelFilter;
use ratatui::DefaultTerminal;
use ratatui::widgets::ListState;
use sysproxy::Sysproxy;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{Duration, interval};

use akasha::client as ac;
use akasha::parser::config::{AkashaConfig, MihomoConfig};

use crate::app::ui::components::dashboard::Dashboard;
use crate::app::ui::components::proxies::Proxies;
use crate::app::ui::components::sidebar::Sidebar;
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
    sysproxy: Option<Sysproxy>,

    // Components
    sidebar: Sidebar,
    dashboard: Dashboard,
    proxies: Proxies,
    // sidebar_status: SidebarStatus,
    // dashboard_status: DashboardStatus,
    // proxies_status: ProxiesStatus,
    logs_status: LogsStatus,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        let pkginfo = PkgInfo::new();
        let mihomo_config = MihomoConfig::from_file(pkginfo.get_mihomo_config()).unwrap();
        let akasha_config = AkashaConfig::from_file(pkginfo.get_akasha_config()).unwrap();
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
            sidebar: Sidebar::new(pkginfo.get_name(), pkginfo.get_version()),
            sysproxy: None,
            dashboard: Dashboard::new(),
            proxies: Proxies::new(&mihomo_config.proxy_groups),
            // proxies_status: ProxiesStatus {
            //     group_state: ListState::default().with_selected(Some(0)),
            //     group_items: mihomo_config
            //         .groups_name()
            //         .into_iter()
            //         .map(|name| (name, 0))
            //         .collect(),
            //     proxy_state: ListState::default().with_selected(Some(0)),
            //     proxy_items: mihomo_config.groups_proxies(),
            //     proxy_focus: false,
            //     delay: vec![None; mihomo_config.group_count()],
            //     delay_mpsc: mpsc::channel::<(Option<HashMap<String, u32>>, usize)>(64),
            // },
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
            akasha_config,
            pkginfo,
        }
    }

    // ANCHOR: key handler events
    async fn enter_handler(&mut self) {
        match self.sidebar.current_page() {
            CurrentPage::Dashboard => todo!(),
            CurrentPage::Proxies => self.proxies.enter_handler(self.mihomo.clone()).await,
            CurrentPage::Profiles => todo!(),
            CurrentPage::Connections => todo!(),
            CurrentPage::Rules => todo!(),
            CurrentPage::Logs => todo!(),
            CurrentPage::Test => todo!(),
            CurrentPage::Settings => todo!(),
        }
    }

    fn esc_handler(&mut self) {
        match self.sidebar.current_page() {
            CurrentPage::Dashboard => todo!(),
            CurrentPage::Proxies => self.proxies.esc_handler(),
            CurrentPage::Profiles => todo!(),
            CurrentPage::Connections => todo!(),
            CurrentPage::Rules => todo!(),
            CurrentPage::Logs => todo!(),
            CurrentPage::Test => todo!(),
            CurrentPage::Settings => todo!(),
        }
    }

    fn l_handler(&mut self) {
        match self.sidebar.current_page() {
            CurrentPage::Dashboard => todo!(),
            CurrentPage::Proxies => self.proxies.l_handler(),
            CurrentPage::Profiles => todo!(),
            CurrentPage::Connections => todo!(),
            CurrentPage::Rules => todo!(),
            CurrentPage::Logs => self.logs_status.l_handler(),
            CurrentPage::Test => todo!(),
            CurrentPage::Settings => todo!(),
        }
    }

    fn h_handler(&mut self) {
        match self.sidebar.current_page() {
            CurrentPage::Dashboard => todo!(),
            CurrentPage::Proxies => self.proxies.h_handler(),
            CurrentPage::Profiles => todo!(),
            CurrentPage::Connections => todo!(),
            CurrentPage::Rules => todo!(),
            CurrentPage::Logs => self.logs_status.h_handler(),
            CurrentPage::Test => todo!(),
            CurrentPage::Settings => todo!(),
        }
    }

    fn j_handler(&mut self) {
        match self.sidebar.current_page() {
            CurrentPage::Dashboard => self.dashboard.j_handler(),
            // CurrentPage::Dashboard => self.dashboard_status.j_handler(),
            CurrentPage::Proxies => self.proxies.j_handler(),
            CurrentPage::Profiles => todo!(),
            CurrentPage::Connections => todo!(),
            CurrentPage::Rules => todo!(),
            CurrentPage::Logs => self.logs_status.j_handler(),
            CurrentPage::Test => todo!(),
            CurrentPage::Settings => todo!(),
        }
    }

    fn k_handler(&mut self) {
        match self.sidebar.current_page() {
            CurrentPage::Dashboard => self.dashboard.k_handler(),
            // CurrentPage::Dashboard => self.dashboard_status.k_handler(),
            CurrentPage::Proxies => self.proxies.k_handler(),
            CurrentPage::Profiles => todo!(),
            CurrentPage::Connections => todo!(),
            CurrentPage::Rules => todo!(),
            CurrentPage::Logs => self.logs_status.k_handler(),
            CurrentPage::Test => todo!(),
            CurrentPage::Settings => todo!(),
        }
    }

    async fn d_handler(&mut self) {
        match self.sidebar.current_page() {
            CurrentPage::Dashboard => todo!(),
            CurrentPage::Proxies => {
                self.proxies
                    .d_handler(self.mihomo.clone(), self.akasha_config.test_url())
                    .await
            }
            CurrentPage::Profiles => todo!(),
            CurrentPage::Connections => todo!(),
            CurrentPage::Rules => todo!(),
            CurrentPage::Logs => todo!(),
            CurrentPage::Test => todo!(),
            CurrentPage::Settings => todo!(),
        }
    }

    async fn p_handler(&mut self) {
        match &mut self.sysproxy {
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

        // // ANCHOR: proxies
        // // ANCHOR: setup the group and proxy selected status in ui
        // let (tx_proxies, mut rx_proxies) = broadcast::channel::<Vec<usize>>(64);
        // let rx_proxies_dash = tx_proxies.subscribe();
        // let mihomo_proxies = self.mihomo.clone();
        // let proxy_group = self.proxies_status.group_items.clone();
        // let proxy_items = self.proxies_status.proxy_items.clone();
        // // Set the interval of the proxy async check task.
        // let mut ticker_proxy_task = interval(Duration::from_secs(5));
        // tokio::spawn(async move {
        //     loop {
        //         ticker_proxy_task.tick().await;
        //         let mut selected_proxy: Vec<usize> = vec![];
        //         let mi = mihomo_proxies.read().await;
        //         for (i, group) in proxy_group.iter().enumerate() {
        //             // Because there is a loop using of mihomo,
        //             // via function from mihomo itself will be better.
        //             let proxy = mi.get_proxy_by_name(group.0.as_str()).await;
        //             if let Ok(proxy) = proxy {
        //                 match proxy.now {
        //                     Some(now) => {
        //                         let index =
        //                             proxy_items[i].iter().position(|name| name == &now).unwrap();
        //                         selected_proxy.push(index);
        //                     }
        //                     None => {}
        //                 }
        //             }
        //         }
        //         let _ = tx_proxies.send(selected_proxy).unwrap();
        //     }
        // });
        // // ANCHOR_END: setup the group and proxy selected status in ui
        // // ANCHOR_END: proxies
        let (mut rx_proxies, rx_proxies_dash) = self.proxies.launch_server(self.mihomo.clone());

        let (mut rx_traffic, mut rx_memory) = Sidebar::launch_server(self.mihomo.clone());
        let (mut rx_subscription, mut rx_delay) =
            self.dashboard
                .launch_server(&self.akasha_config, self.mihomo.clone(), rx_proxies_dash);

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

        while self.running {
            tokio::select! {
                _ = ticker.tick() => {
                    self.sidebar.sync_client(&mut rx_traffic, &mut rx_memory);
                    self.dashboard.sync_client(&mut rx_subscription, &mut rx_delay);

                    if let Ok(value) = rx_sysproxy.try_recv() {
                        log::trace!("Sysproxy try_recv(): {:?}", value);
                        self.sysproxy = Some(value);
                    }

                    self.proxies.sync_client(&mut rx_proxies);
                    // // proxies selected status thread recv
                    // if let Ok(value) = rx_proxies.try_recv() {
                    //     log::trace!("Proxies Groups selected status: {:?}", value);
                    //     if value.len() == self.proxies_status.group_items.len() {
                    //         for (i, index) in value.iter().enumerate() {
                    //             self.proxies_status.group_items[i].1 = *index;
                    //         }
                    //     } else {
                    //         panic!("There is something wrong with group size.");
                    //     }
                    // }

                    // // proxies delay info thread recv
                    // if let Ok(value) = self.proxies_status.delay_mpsc.1.try_recv() {
                    //     // // The index here maybe different from the index of 'd' press time.
                    //     // let group_index = self.proxies_status.group_state.selected().unwrap();
                    //     let (msg, group_index) = value;
                    //     self.proxies_status.delay[group_index] = msg;
                    // }
                    self.proxies.update();

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
            (_, KeyCode::Tab) => self.sidebar.tab_next(),
            (_, KeyCode::BackTab) => self.sidebar.tab_pre(),
            (_, KeyCode::Char('j')) => self.j_handler(),
            (_, KeyCode::Char('k')) => self.k_handler(),
            (_, KeyCode::Char('h')) => self.h_handler(),
            (_, KeyCode::Char('l')) => self.l_handler(),
            (_, KeyCode::Enter) => self.enter_handler().await,
            (_, KeyCode::Esc) => self.esc_handler(),
            (_, KeyCode::Char('d')) => self.d_handler().await,
            (_, KeyCode::Char('p')) => self.p_handler().await,
            (_, KeyCode::Char(c @ '1'..='8')) => {
                let idx = (c as u8 - b'1') as usize;
                self.sidebar.tab_switch(idx);
            }
            // Add other key handlers here.
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}

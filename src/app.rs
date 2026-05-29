mod ui;

use std::path::PathBuf;
use std::sync::Arc;

use aka_logger::{AkaLogger, LoggerConfig};
use akasha::client::mihomo::Mihomo;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use log::LevelFilter;
use ratatui::DefaultTerminal;
use sysproxy::Sysproxy;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{Duration, interval};

use akasha::client as ac;
use akasha::parser::config::{AkashaConfig, MihomoConfig};

use crate::app::ui::components::dashboard::Dashboard;
use crate::app::ui::components::logs::Logs;
use crate::app::ui::components::proxies::Proxies;
use crate::app::ui::components::sidebar::Sidebar;
use crate::pkginfo::PkgInfo;

enum CurrentPage {
    Dashboard,
    Proxies,
    Profiles,
    Connections,
    Rules,
    Logs,
    Test,
    Settings,
}

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
    logs: Logs,
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
            logs: Logs::new(AkaLogger::init(LoggerConfig {
                buf_capacity: 10,
                level: LevelFilter::Info,
                with_stdout: true,
                log_path: PathBuf::from("debug/info.log").into_boxed_path(),
            })),
            akasha_config,
            pkginfo,
        }
    }

    // // ANCHOR: key handler events
    // async fn enter_handler(&mut self) {
    //     match self.sidebar.current_page() {
    //         CurrentPage::Dashboard => todo!(),
    //         CurrentPage::Proxies => self.proxies.enter_handler(self.mihomo.clone()).await,
    //         CurrentPage::Profiles => todo!(),
    //         CurrentPage::Connections => todo!(),
    //         CurrentPage::Rules => todo!(),
    //         CurrentPage::Logs => todo!(),
    //         CurrentPage::Test => todo!(),
    //         CurrentPage::Settings => todo!(),
    //     }
    // }

    // fn esc_handler(&mut self) {
    //     match self.sidebar.current_page() {
    //         CurrentPage::Dashboard => todo!(),
    //         CurrentPage::Proxies => self.proxies.esc_handler(),
    //         CurrentPage::Profiles => todo!(),
    //         CurrentPage::Connections => todo!(),
    //         CurrentPage::Rules => todo!(),
    //         CurrentPage::Logs => todo!(),
    //         CurrentPage::Test => todo!(),
    //         CurrentPage::Settings => todo!(),
    //     }
    // }

    // fn l_handler(&mut self) {
    //     match self.sidebar.current_page() {
    //         CurrentPage::Dashboard => todo!(),
    //         CurrentPage::Proxies => self.proxies.l_handler(),
    //         CurrentPage::Profiles => todo!(),
    //         CurrentPage::Connections => todo!(),
    //         CurrentPage::Rules => todo!(),
    //         CurrentPage::Logs => todo!(),
    //         CurrentPage::Test => todo!(),
    //         CurrentPage::Settings => todo!(),
    //     }
    // }

    // fn h_handler(&mut self) {
    //     match self.sidebar.current_page() {
    //         CurrentPage::Dashboard => todo!(),
    //         CurrentPage::Proxies => self.proxies.h_handler(),
    //         CurrentPage::Profiles => todo!(),
    //         CurrentPage::Connections => todo!(),
    //         CurrentPage::Rules => todo!(),
    //         CurrentPage::Logs => todo!(),
    //         CurrentPage::Test => todo!(),
    //         CurrentPage::Settings => todo!(),
    //     }
    // }

    // fn j_handler(&mut self) {
    //     match self.sidebar.current_page() {
    //         CurrentPage::Dashboard => self.dashboard.j_handler(),
    //         CurrentPage::Proxies => self.proxies.j_handler(),
    //         CurrentPage::Profiles => todo!(),
    //         CurrentPage::Connections => todo!(),
    //         CurrentPage::Rules => todo!(),
    //         CurrentPage::Logs => self.logs.j_handler(),
    //         CurrentPage::Test => todo!(),
    //         CurrentPage::Settings => todo!(),
    //     }
    // }

    // fn k_handler(&mut self) {
    //     match self.sidebar.current_page() {
    //         CurrentPage::Dashboard => self.dashboard.k_handler(),
    //         CurrentPage::Proxies => self.proxies.k_handler(),
    //         CurrentPage::Profiles => todo!(),
    //         CurrentPage::Connections => todo!(),
    //         CurrentPage::Rules => todo!(),
    //         CurrentPage::Logs => self.logs.k_handler(),
    //         CurrentPage::Test => todo!(),
    //         CurrentPage::Settings => todo!(),
    //     }
    // }

    // async fn d_handler(&mut self) {
    //     match self.sidebar.current_page() {
    //         CurrentPage::Dashboard => todo!(),
    //         CurrentPage::Proxies => {
    //             self.proxies
    //                 .d_handler(self.mihomo.clone(), self.akasha_config.test_url())
    //                 .await
    //         }
    //         CurrentPage::Profiles => todo!(),
    //         CurrentPage::Connections => todo!(),
    //         CurrentPage::Rules => todo!(),
    //         CurrentPage::Logs => todo!(),
    //         CurrentPage::Test => todo!(),
    //         CurrentPage::Settings => todo!(),
    //     }
    // }

    // async fn p_handler(&mut self) {
    //     match &mut self.sysproxy {
    //         Some(sysproxy) => {
    //             sysproxy.enable = !sysproxy.enable;
    //             sysproxy.host = "127.0.0.1".into();
    //             sysproxy.port = 7890;

    //             sysproxy.set_system_proxy().unwrap();
    //             log::info!("Switched the system proxy status: {:?}", sysproxy);
    //         }
    //         None => log::error!("Something wrong with [`Sysproxy`] getting."),
    //     }
    // }
    // // ANCHOR_END: key handler events

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.running = true;
        // Renderint interval
        let mut ticker = interval(Duration::from_millis(1000 / 24));

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
            (_, KeyCode::Char(c @ '1'..='8')) => {
                let idx = (c as u8 - b'1') as usize;
                self.sidebar.tab_switch(idx);
            }
            // (_, KeyCode::Char('j')) => self.j_handler(),
            // (_, KeyCode::Char('k')) => self.k_handler(),
            // (_, KeyCode::Char('h')) => self.h_handler(),
            // (_, KeyCode::Char('l')) => self.l_handler(),
            // (_, KeyCode::Enter) => self.enter_handler().await,
            // (_, KeyCode::Esc) => self.esc_handler(),
            // (_, KeyCode::Char('d')) => self.d_handler().await,
            // (_, KeyCode::Char('p')) => self.p_handler().await,

            // Add other key handlers here.
            _ => match self.sidebar.current_page() {
                CurrentPage::Dashboard => self.dashboard.handle_key_event(key),
                CurrentPage::Proxies => {
                    self.proxies
                        .handle_key_event(key, self.mihomo.clone(), self.akasha_config.test_url())
                        .await
                }
                CurrentPage::Profiles => {}
                CurrentPage::Connections => {}
                CurrentPage::Rules => {}
                CurrentPage::Logs => self.logs.handle_key_event(key),
                CurrentPage::Test => {}
                CurrentPage::Settings => {}
            },
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}

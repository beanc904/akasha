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

use crate::pkginfo::PkgInfo;

pub enum CurrentPage {
    Home,
    Proxies,
    Profiles,
    Connections,
    Rules,
    Logs,
    Test,
    Settings,
}

// #[derive(Debug, Default)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    // Event stream.
    pub event_stream: EventStream,
    // Mihomo handle.
    pub mihomo: Arc<RwLock<ac::mihomo::Mihomo>>,
    // Package informations.
    pub pkginfo: PkgInfo,
    // Sidebar selective status.
    pub sidebar_state: ListState,
    pub sidebar_items: Vec<&'static str>,
    pub current_page: CurrentPage,

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
        let mut liststate = ListState::default();
        liststate.select(Some(0));
        let pkginfo = PkgInfo::new();
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
            pkginfo,
            sidebar_state: liststate,
            sidebar_items: vec![
                "Home",
                "Proxies",
                "Profiles",
                "Connections",
                "Rules",
                "Logs",
                "Test",
                "Settings",
            ],
            current_page: CurrentPage::Home,
            traffic_data: VecDeque::default(),
            memory_inuse: 0f64,
            tick: 0f64,
        }
    }

    /// You must use it after finishing selecting the current page.
    fn set_current_page(&mut self) {
        match self.sidebar_state.selected() {
            Some(0) => self.current_page = CurrentPage::Home,
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

    pub fn sidebar_next(&mut self) {
        let index = match self.sidebar_state.selected() {
            Some(i) => {
                if i >= self.sidebar_items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.sidebar_state.select(Some(index));
        self.set_current_page();
    }

    pub fn sidebar_previous(&mut self) {
        let index = match self.sidebar_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.sidebar_items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.sidebar_state.select(Some(index));
        self.set_current_page();
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
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Tab) => self.sidebar_next(),
            (_, KeyCode::BackTab) => self.sidebar_previous(),
            // Add other key handlers here.
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}

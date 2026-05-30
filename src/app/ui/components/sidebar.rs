mod sidebar_logo;
mod sidebar_monitor;
mod sidebar_tab;

use std::sync::Arc;

use akasha::client::mihomo::Mihomo;
use ratatui::prelude::*;
use serde_json::Value;
use sysproxy::Sysproxy;
use tokio::sync::{RwLock, mpsc};

use crate::app::{CurrentPage, ui::components::Component};

use self::sidebar_logo::SidebarLogo;
use self::sidebar_monitor::SidebarMonitor;
use self::sidebar_tab::SidebarTab;

pub struct Sidebar {
    logo: SidebarLogo,
    tab: SidebarTab,
    monitor: SidebarMonitor,
}

impl Sidebar {
    pub fn new<S>(logo: S, version: S) -> Self
    where
        S: AsRef<str>,
    {
        Self {
            logo: SidebarLogo::new(logo, version),
            tab: SidebarTab::new(),
            monitor: SidebarMonitor::new(),
        }
    }

    /// Remember to use update before draw the components
    pub fn update(&mut self, sysproxy: &Option<Sysproxy>) {
        // Update logo status
        self.logo.update_mode(sysproxy);
    }

    // Transfer functions
    pub fn current_page(&self) -> &CurrentPage {
        self.tab.current_page()
    }

    pub fn tab_next(&mut self) {
        self.tab.tab_next();
    }

    pub fn tab_pre(&mut self) {
        self.tab.tab_previous();
    }

    pub fn tab_switch(&mut self, index: usize) {
        self.tab.tab_switch(index);
    }

    pub fn launch_server(
        mihomo: Arc<RwLock<Mihomo>>,
    ) -> (mpsc::Receiver<Value>, mpsc::Receiver<Value>) {
        let (tx_traffic, rx_traffic) = mpsc::channel::<Value>(64);
        let (tx_memory, rx_memory) = mpsc::channel::<Value>(64);
        let mihomo_traffic = mihomo.clone();
        let mihomo_memory = mihomo.clone();
        tokio::spawn(akasha::client::ws_traffic(mihomo_traffic, tx_traffic));
        tokio::spawn(akasha::client::ws_memory(mihomo_memory, tx_memory));
        (rx_traffic, rx_memory)
    }

    pub fn sync_client(
        &mut self,
        rx_traffic: &mut mpsc::Receiver<Value>,
        rx_memory: &mut mpsc::Receiver<Value>,
    ) {
        if let Ok(value) = rx_traffic.try_recv() {
            log::trace!("Traffic try_recv(): {:?}", value);
            let inner_json_data = value["data"].as_str().unwrap().trim();
            let data: Value = serde_json::from_str(inner_json_data).unwrap();
            let up = data["up"].as_f64().unwrap();
            let down = data["down"].as_f64().unwrap();
            let up_total = data["upTotal"].as_f64().unwrap();
            let down_total = data["downTotal"].as_f64().unwrap();

            self.monitor.push_back(up, down, up_total, down_total);
        }

        if let Ok(value) = rx_memory.try_recv() {
            log::trace!("Memory try_recv(): {:?}", value);
            let inner_json_data = value["data"].as_str().unwrap().trim();
            let data: Value = serde_json::from_str(inner_json_data).unwrap();
            let inuse = data["inuse"].as_f64().unwrap();

            self.monitor.memory_inuse(inuse);
        }
    }
}

impl Component for Sidebar {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(area);

        self.logo.draw(frame, layout[0]);
        self.tab.draw(frame, layout[1]);
        self.monitor.draw(frame, layout[2]);
    }
}

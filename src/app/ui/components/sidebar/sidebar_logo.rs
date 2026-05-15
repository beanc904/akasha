use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use sysproxy::Sysproxy;

use crate::app::ui::components::Component;

pub struct SidebarLogo {
    logo: String,
    version: String,
    mode: ProxyMode,
}

impl SidebarLogo {
    pub fn new<S>(logo: S, version: S) -> Self
    where
        S: AsRef<str>,
    {
        Self {
            logo: logo.as_ref().to_string(),
            version: version.as_ref().to_string(),
            mode: ProxyMode::Off,
        }
    }

    pub fn update_mode(&mut self, sysproxy: &Option<Sysproxy>) {
        self.mode = match sysproxy {
            Some(Sysproxy { enable, .. }) => match enable {
                true => ProxyMode::Proxy,
                false => ProxyMode::Off,
            },
            None => ProxyMode::Off,
        };
    }
}

#[allow(dead_code)]
enum ProxyMode {
    Off,
    Proxy,
    Tun,
}

impl Component for SidebarLogo {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let para = Paragraph::new(format!("{} v{}", self.logo.to_uppercase(), self.version))
            .block(Block::new().borders(Borders::ALL))
            .fg(match self.mode {
                ProxyMode::Off => Color::default(),
                ProxyMode::Proxy => Color::Green,
                ProxyMode::Tun => Color::Blue,
            });

        frame.render_widget(para, area);
    }
}

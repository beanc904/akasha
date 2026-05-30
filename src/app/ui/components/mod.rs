mod dashboard;
mod logs;
mod proxies;
mod sidebar;

pub use self::{dashboard::Dashboard, logs::Logs, proxies::Proxies, sidebar::Sidebar};

use ratatui::{Frame, layout::Rect};

pub(super) trait Component {
    fn draw(&mut self, frame: &mut Frame, area: Rect);
}

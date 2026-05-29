use ratatui::{Frame, layout::Rect};

pub mod dashboard;
pub mod logs;
pub mod proxies;
pub mod sidebar;

pub trait Component {
    fn draw(&mut self, frame: &mut Frame, area: Rect);
}

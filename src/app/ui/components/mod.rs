use ratatui::{Frame, layout::Rect};

pub mod sidebar;

pub trait Component {
    #[allow(unused)]
    fn draw(&mut self, frame: &mut Frame, area: Rect) {}
}

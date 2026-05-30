use std::sync::Arc;

use aka_logger::LogStore;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders},
};

use crate::app::ui::{components::Component, widgets::ScrollView};

pub struct Logs {
    logstore: Arc<LogStore>,
    scrollview: ScrollView,
}

impl Logs {
    pub fn new(store: Arc<LogStore>) -> Self {
        Self {
            logstore: store,
            scrollview: ScrollView::new(2),
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Char('j')) => self.scrollview.j_handler(),
            (_, KeyCode::Char('k')) => self.scrollview.k_handler(),
            _ => {}
        }
    }
}

impl Component for Logs {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        // Consider using the below later.
        // ```
        // logs.lines
        //     .iter()
        //     .skip(scroll)
        //     .take(height)
        // ```
        let binding = self.logstore.get_inner();
        let logs = binding.read().unwrap();
        let logs: Vec<Line> = logs.iter().map(|log| Line::raw(log)).collect();

        let root_block = Block::default().borders(Borders::ALL).title(" Logs ");
        let root_inner = root_block.inner(area);
        frame.render_widget(root_block, area);
        // self.scrolltext.content(logs);
        self.scrollview.render(frame, root_inner, logs);
    }
}

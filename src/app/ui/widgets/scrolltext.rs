use ratatui::{
    prelude::*,
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
};

pub struct ScrollText {
    step: usize,
    content: Vec<Line<'static>>,
    content_length: usize,
    viewport_height: usize,
    state: ScrollbarState,
}

impl ScrollText {
    pub fn new(step: usize, content: Vec<Line<'static>>) -> Self {
        let content_length = content.len();
        Self {
            step,
            content,
            content_length,
            viewport_height: 0,
            state: ScrollbarState::default(),
        }
    }

    pub fn content(&mut self, content: Vec<Line<'static>>) {
        self.content_length = content.len();
        self.content = content;
    }

    // pub fn viewport_height(&mut self, height: usize) {
    //     self.viewport_height = height;
    // }

    fn pos_max(&self) -> usize {
        self.content_length.saturating_sub(self.viewport_height) + 2
    }

    fn reset_state(&mut self, length: usize) {
        self.state = self.state.content_length(length);
    }

    fn position(&mut self, pos: usize) {
        self.state = self.state.position(pos);
    }

    pub fn j_handler(&mut self) {
        let max = self.pos_max();
        let step = self.step;
        let pos = self.state.get_position();
        if pos >= max {
            self.position(max);
        } else {
            self.position(pos + step);
        }
    }

    pub fn k_handler(&mut self) {
        let step = self.step;
        let pos = self.state.get_position();
        if pos == 0 || (pos as i32 - step as i32) < 0 {
            self.position(0);
        } else {
            self.position(pos - step);
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.viewport_height = area.height as usize;
        self.reset_state(self.pos_max());

        let [para_area, scrollbar_area] =
            Layout::horizontal([Constraint::Min(0), Constraint::Length(1)]).areas(area);

        let paragraph = Paragraph::new(self.content.clone())
            .scroll((self.state.get_position() as u16, 0))
            .wrap(Wrap { trim: true });
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);

        frame.render_widget(paragraph, para_area);
        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut self.state);
    }
}

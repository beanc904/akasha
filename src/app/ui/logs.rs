use std::rc::Rc;

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::app::App;

pub(super) fn render_logs(app: &App, frame: &mut Frame, layout_root: &Rc<[Rect]>) {
    let root_block = Block::default().borders(Borders::ALL).title(" Logs ");
    let root_inner = root_block.inner(layout_root[1]);
    frame.render_widget(root_block, layout_root[1]);

    let [para_area, v_scrollbar_area] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(1)]).areas(root_inner);
    let [inner_area, h_scrollbar_area] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(para_area);

    let all = app.logs_status.get_all();
    let mut log_max_len = 0;
    let logs: Vec<Line> = all
        .iter()
        .map(|log| {
            let len = log.len();
            if len > log_max_len {
                log_max_len = len;
            };
            Line::raw(log)
        })
        .collect();

    let v_pos = app.logs_status.get_scrollbar_pos().0;
    let h_pos = app.logs_status.get_scrollbar_pos().1;
    let v_scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    let h_scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom);
    let mut v_scrollbar_state = ScrollbarState::new(logs.len()).position(v_pos);
    let mut h_scrollbar_state = ScrollbarState::new(log_max_len).position(if h_pos < log_max_len {
        h_pos
    } else {
        log_max_len
    });

    let paragraph = Paragraph::new(logs).scroll((v_pos as u16, h_pos as u16));

    frame.render_widget(paragraph, inner_area);
    frame.render_stateful_widget(v_scrollbar, v_scrollbar_area, &mut v_scrollbar_state);
    frame.render_stateful_widget(h_scrollbar, h_scrollbar_area, &mut h_scrollbar_state);
}

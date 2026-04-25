use std::rc::Rc;

use ratatui::prelude::*;
use ratatui::widgets::Wrap;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

pub(super) fn render_dashboard(app: &App, frame: &mut Frame, layout_root: &Rc<[Rect]>) {
    let root_block = Block::default().borders(Borders::ALL).title(" Dashboard ");
    let root_inner = root_block.inner(layout_root[1]);
    frame.render_widget(root_block, layout_root[1]);

    let title_lines: Vec<Line> = app
        .dashboard_status
        .titles
        .iter()
        .map(|title| Line::from(format!(">>>{}<<<", title)).bg(Color::DarkGray))
        .collect();

    // ANCHOR: getting time and usage info
    let time_txt = app.dashboard_status.get_updatetime();
    let usage_txt = app.dashboard_status.get_usage();
    // ANCHOR_END: getting time and usage info

    // ANCHOR: getting selected node and delay info
    let selected_txt = app.proxies_status.get_selected_node();
    let mihomo = app.mihomo.clone();
    let node_delay_txt = app.proxies_status.get_selected_node_delay(mihomo);
    // ANCHOR_END: getting selected node and delay info

    let underline_style = Style::new().underlined();
    let profiles_txt = vec![
        title_lines[0].clone(),
        Line::from(vec![
            Span::styled("From: ", underline_style),
            Span::raw(&app.akasha_config.subscription_link),
        ]),
        Line::from(vec![
            Span::styled("Update Time: ", underline_style),
            Span::raw(time_txt),
        ]),
        Line::from(vec![
            Span::styled("Used / Total: ", underline_style),
            Span::raw(usage_txt),
        ]),
    ];
    let currentnode_txt = vec![
        title_lines[1].clone(),
        Line::from(vec![
            Span::styled("Selected: ", underline_style),
            Span::raw(selected_txt),
        ]),
        Line::from(vec![
            Span::styled("Delay: ", underline_style),
            Span::raw("delay err"),
        ]),
    ];

    let mut txt = Vec::new();
    txt.extend(profiles_txt);
    txt.extend(currentnode_txt);
    let paragraph = Paragraph::new(txt).wrap(Wrap { trim: true });

    frame.render_widget(paragraph, root_inner);
}

use std::borrow::Cow;
use std::rc::Rc;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::widgets::{Scrollbar, ScrollbarState, Wrap};

use crate::app::App;

pub(super) fn render_dashboard(app: &App, frame: &mut Frame, layout_root: &Rc<[Rect]>) {
    let root_block = Block::default().borders(Borders::ALL).title(" Dashboard ");
    let root_inner = root_block.inner(layout_root[1]);
    let [para_area, scrollbar_area] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(1)]).areas(root_inner);
    frame.render_widget(root_block, layout_root[1]);

    let title_lines: Vec<Line> = app
        .dashboard_status
        .titles
        .iter()
        .map(|title| Line::from(format!(">>> {} <<<", title)).bg(Color::DarkGray))
        .collect();

    // ANCHOR: getting time and usage info
    let time_txt = app.dashboard_status.get_updatetime();
    let usage_txt = app.dashboard_status.get_usage();
    // ANCHOR_END: getting time and usage info

    // ANCHOR: getting selected node and delay info
    let selected_txt = app.proxies_status.get_selected_node();
    // let mihomo = app.mihomo.clone();
    // let node_delay_txt = app.proxies_status.get_selected_node_delay(mihomo);
    let delay = app.dashboard_status.selected_node_delay;
    let node_delay = Span::raw(format!("{} ms", delay)).style(Style::default().fg(if delay == 0 {
        Color::Red
    } else {
        if delay < 250 {
            Color::Green
        } else if delay < 500 {
            Color::Blue
        } else {
            Color::LightRed
        }
    }));
    // ANCHOR_END: getting selected node and delay info

    let label_style = Style::new().underlined();
    let labels = &app.dashboard_status.sublabels;
    let profiles_txt = vec![
        title_lines[0].clone(),
        one_line(
            labels[0][0],
            &app.akasha_config.subscription_link,
            label_style,
        ),
        one_line(labels[0][1], time_txt, label_style),
        one_line(labels[0][2], usage_txt, label_style),
    ];
    let currentnode_txt = vec![
        title_lines[1].clone(),
        one_line(labels[1][0], selected_txt, label_style),
        // one_line("Delay: ", node_delay, underline_style),
        Line::from(vec![Span::styled(labels[1][1], label_style), node_delay]),
    ];
    let networksettings_txt = vec![
        title_lines[2].clone(),
        one_line(labels[2][0], "xxx", label_style),
        one_line(labels[2][1], "xxx", label_style),
    ];
    let proxymode_txt = vec![
        title_lines[3].clone(),
        one_line(labels[3][0], "xxx", label_style),
    ];
    let trafficstats_txt = vec![
        title_lines[4].clone(),
        one_line(labels[4][0], "xxx", label_style),
        one_line(labels[4][1], "xxx", label_style),
        one_line(labels[4][2], "xxx", label_style),
        one_line(labels[4][3], "xxx", label_style),
        one_line(labels[4][4], "xxx", label_style),
        one_line(labels[4][5], "xxx", label_style),
    ];
    let websitetests_txt = vec![
        title_lines[5].clone(),
        one_line(labels[5][0], "xxx", label_style),
        one_line(labels[5][1], "xxx", label_style),
        one_line(labels[5][2], "xxx", label_style),
        one_line(labels[5][3], "xxx", label_style),
    ];
    let ipinfo_txt = vec![
        title_lines[6].clone(),
        one_line(labels[6][0], "xxx", label_style),
        one_line(labels[6][1], "xxx", label_style),
        one_line(labels[6][2], "xxx", label_style),
        one_line(labels[6][3], "xxx", label_style),
        one_line(labels[6][4], "xxx", label_style),
        one_line(labels[6][5], "xxx", label_style),
    ];
    let clashinfo_txt = vec![
        title_lines[7].clone(),
        one_line(labels[7][0], "xxx", label_style),
        one_line(labels[7][1], "xxx", label_style),
        one_line(labels[7][2], "xxx", label_style),
        one_line(labels[7][3], "xxx", label_style),
        one_line(labels[7][4], "xxx", label_style),
    ];
    let sysinfo_txt = vec![
        title_lines[8].clone(),
        one_line(labels[8][0], "xxx", label_style),
        one_line(labels[8][1], "xxx", label_style),
        one_line(labels[8][2], "xxx", label_style),
        one_line(labels[8][3], "xxx", label_style),
        one_line(labels[8][4], "xxx", label_style),
    ];

    let mut txts = Vec::new();
    txts.extend(profiles_txt);
    txts.extend(currentnode_txt);
    txts.extend(networksettings_txt);
    txts.extend(proxymode_txt);
    txts.extend(trafficstats_txt);
    txts.extend(websitetests_txt);
    txts.extend(ipinfo_txt);
    txts.extend(clashinfo_txt);
    txts.extend(sysinfo_txt);

    let mut txt_height = 0;
    for txt in &txts {
        txt_height += txt.iter().len();
    }
    let pos = app.dashboard_status.scrollbar_pos;
    let scrollbar = Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight);
    let mut scrollbar_state = ScrollbarState::new(txt_height).position(pos);
    let paragraph = Paragraph::new(txts)
        .scroll((pos as u16, 0))
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, para_area);
    frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
}

fn one_line<'a, T, P, S>(label: T, content: P, label_style: S) -> Line<'a>
where
    T: Into<Cow<'a, str>>,
    P: Into<Cow<'a, str>>,
    S: Into<Style>,
{
    Line::from(vec![
        Span::styled(label.into(), label_style),
        Span::raw(content.into()),
    ])
}

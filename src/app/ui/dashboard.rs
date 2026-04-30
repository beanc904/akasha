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
        .map(|title| Line::from(format!(">>>{}<<<", title)).bg(Color::DarkGray))
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
    let profiles_txt = vec![
        title_lines[0].clone(),
        one_line("From: ", &app.akasha_config.subscription_link, label_style),
        one_line("Update Time: ", time_txt, label_style),
        one_line("Used / Total: ", usage_txt, label_style),
    ];
    let currentnode_txt = vec![
        title_lines[1].clone(),
        one_line("Selected: ", selected_txt, label_style),
        // one_line("Delay: ", node_delay, underline_style),
        Line::from(vec![Span::styled("Delay: ", label_style), node_delay]),
    ];
    let networksettings_txt = vec![
        title_lines[2].clone(),
        one_line("System Proxy: ", "xxx", label_style),
        one_line("Tun Mode: ", "xxx", label_style),
    ];
    let proxymode_txt = vec![
        title_lines[3].clone(),
        one_line("Mode: ", "xxx", label_style),
    ];
    let trafficstats_txt = vec![
        title_lines[4].clone(),
        one_line("Upload Speed: ", "xxx", label_style),
        one_line("Download Speed: ", "xxx", label_style),
        one_line("Uploaded: ", "xxx", label_style),
        one_line("Downloaded: ", "xxx", label_style),
        one_line("Active Connections: ", "xxx", label_style),
        one_line("Core Usage: ", "xxx", label_style),
    ];
    let websitetests_txt = vec![
        title_lines[5].clone(),
        one_line("Apple: ", "xxx", label_style),
        one_line("GitHub: ", "xxx", label_style),
        one_line("Google: ", "xxx", label_style),
        one_line("YouTube: ", "xxx", label_style),
    ];
    let ipinfo_txt = vec![
        title_lines[6].clone(),
        one_line("IP: ", "xxx", label_style),
        one_line("ASN: ", "xxx", label_style),
        one_line("ISP: ", "xxx", label_style),
        one_line("ORG: ", "xxx", label_style),
        one_line("Location: ", "xxx", label_style),
        one_line("Timezone: ", "xxx", label_style),
    ];
    let clashinfo_txt = vec![
        title_lines[7].clone(),
        one_line("Core Version: ", "xxx", label_style),
        one_line("System Proxy Address: ", "xxx", label_style),
        one_line("Mixed Port: ", "xxx", label_style),
        one_line("Uptime: ", "xxx", label_style),
        one_line("Rules Count", "xxx", label_style),
    ];
    let sysinfo_txt = vec![
        title_lines[8].clone(),
        one_line("OS Info: ", "xxx", label_style),
        one_line("Auto Launch: ", "xxx", label_style),
        one_line("Running Mode: ", "xxx", label_style),
        one_line("Last Check Update: ", "xxx", label_style),
        one_line("Verge Version: ", "xxx", label_style),
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

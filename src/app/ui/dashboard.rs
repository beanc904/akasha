use std::rc::Rc;

use ratatui::prelude::*;
use ratatui::widgets::Wrap;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

pub(super) fn render_dashboard(app: &App, frame: &mut Frame, layout_root: &Rc<[Rect]>) {
    let root_block = Block::default().borders(Borders::ALL).title(" Dashboard ");
    let root_inner = root_block.inner(layout_root[1]);
    frame.render_widget(root_block, layout_root[1]);

    let titles = [
        " Profiles ",
        " CurrentNode ",
        " NetworkSettings ",
        " ProxyMode ",
        " TrafficStats ",
        " WebsiteTests ",
        " IpInformation ",
        " ClashInfo ",
        " SystemInfo ",
    ];
    let title_lines: Vec<Line> = titles
        .iter()
        .map(|title| Line::from(format!(">>>{}<<<", title)).bg(Color::DarkGray))
        .collect();

    let (time_txt, usage_txt) = match &app.subscription_info {
        Some(subscription) => {
            let time = subscription.get_updatetime();
            let usage = subscription.parse_usage();
            let time_txt = format!("{:?}", time);
            let usage_txt = match usage {
                Some(usage) => format!(
                    "{} MB / {} MB",
                    (usage.download + usage.upload) / 1024 / 1024,
                    usage.total / 1024 / 1024
                ),
                None => format!("usage err"),
            };
            (time_txt, usage_txt)
        }
        None => {
            let time_txt = format!("time err");
            let usage_txt = format!("usage err");
            (time_txt, usage_txt)
        }
    };

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
            Span::raw("test text"),
        ]),
        Line::from(vec![
            Span::styled("Delay: ", underline_style),
            Span::raw("test text"),
        ]),
    ];

    let mut txt = Vec::new();
    txt.extend(profiles_txt);
    txt.extend(currentnode_txt);
    let paragraph = Paragraph::new(txt).wrap(Wrap { trim: true });

    frame.render_widget(paragraph, root_inner);
}

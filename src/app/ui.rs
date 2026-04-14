use ratatui::prelude::*;
use ratatui::widgets::{Axis, Chart, Dataset, Scrollbar, ScrollbarState};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use unicode_width::UnicodeWidthStr;

use crate::app::{App, CurrentPage};

/// Renders the user interface.
///
/// This is where you add new widgets. See the following resources for more information:
/// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
/// - <https://github.com/ratatui/ratatui/tree/master/examples>
pub fn draw(app: &mut App, frame: &mut Frame) {
    // let title = Line::from("Ratatui Simple Template")
    //     .bold()
    //     .blue()
    //     .centered();
    // let text = "Hello, Ratatui!\n\n\
    //     Created using https://github.com/ratatui/templates\n\
    //     Press `Esc`, `Ctrl-C` or `q` to stop running.";
    // frame.render_widget(
    //     Paragraph::new(text)
    //         .block(Block::bordered().title(title))
    //         .centered(),
    //     frame.area(),
    // )

    // ANCHOR: setup layout
    let layout_root = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Length(25), Constraint::Min(0)])
        .split(frame.area());
    let layout_sidebar = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(3),
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(layout_root[0]);
    let layout_monitor = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Min(0), Constraint::Length(3)])
        .split(layout_sidebar[2]);
    // ANCHOR_END: setup layout

    // ANCHOR: setup traffic monitor chart and dataset
    // The origin unit b to kb.
    let up_set: Vec<(f64, f64)> = app
        .traffic_data
        .iter()
        .map(|&(tick, up, _, _, _)| (tick, up / 1024f64))
        .collect();
    let down_set: Vec<(f64, f64)> = app
        .traffic_data
        .iter()
        .map(|&(tick, _, down, _, _)| (tick, down / 1024f64))
        .collect();
    // The origin unit: b
    let up_speed = app
        .traffic_data
        .back()
        .unwrap_or(&(0f64, 0f64, 0f64, 0f64, 0f64))
        .1;
    let down_speed = app
        .traffic_data
        .back()
        .unwrap_or(&(0f64, 0f64, 0f64, 0f64, 0f64))
        .2;
    // let up_total = app.traffic_data.back().unwrap().3;
    // let down_total = app.traffic_data.back().unwrap().4;
    let datasets = vec![
        Dataset::default()
            .name("up speed")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Cyan))
            .data(&up_set),
        Dataset::default()
            .name("down speed")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::LightRed))
            .data(&down_set),
    ];
    let chart_traffic_monitor = Chart::new(datasets)
        .block(
            Block::default()
                .title(" Traffic Monitor ")
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .title("T")
                .bounds([app.tick - 60f64, app.tick])
                .labels([
                    format!("{:.0}", app.tick - 60.0).bold(),
                    format!("{:.0}", app.tick - 30.0).into(),
                    format!("{:.0}", app.tick).into(),
                ]),
        )
        .y_axis(
            Axis::default()
                .title("KB/s")
                .bounds([0.0, 10000.0])
                .labels(["0".bold(), "5k".into(), "10k".into()]),
        );
    let up_speed_text = if up_speed <= 1024f64 {
        format!("Up Speed: {:.0} B/s", up_speed)
    } else if up_speed <= 1024f64 * 1024f64 {
        format!("Up Speed: {:.2} KB/s", up_speed / 1024f64)
    } else {
        format!("Up Speed: {:.2} MB/s", up_speed / 1024f64 / 1024f64)
    }
    .cyan()
    .bold();
    let down_speed_text = if down_speed <= 1024f64 {
        format!("Down Speed: {:.0} B/s", down_speed)
    } else if down_speed <= 1024f64 * 1024f64 {
        format!("Down Speed: {:.2} KB/s", down_speed / 1024f64)
    } else {
        format!("Down Speed: {:.2} MB/s", down_speed / 1024f64 / 1024f64)
    }
    .light_red()
    .bold();
    let traffic_info = vec![
        Line::from(up_speed_text),
        Line::from(down_speed_text),
        Line::from(format!(
            "Memory Inuse: {:.1} MB",
            app.memory_inuse / 1024f64 / 1024f64
        )),
    ];
    // ANCHOR_END: setup traffic monitor chart and dataset

    let sidebar_items: Vec<ListItem> = app
        .sidebar_status
        .list_items
        .iter()
        .map(|i| ListItem::new(*i).style(Style::default().fg(Color::White)))
        .collect();

    let sidebar = List::new(sidebar_items)
        .block(
            Block::default()
                .title(" Menu(Tab/BackTab) ")
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .bg(Color::White)
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    let selected_tab_style = Style::default()
        .bg(Color::White)
        .fg(Color::DarkGray)
        .add_modifier(Modifier::BOLD);
    match app.sidebar_status.current_page {
        CurrentPage::Dashboard => {
            let root_block = Block::default().borders(Borders::ALL).title(" Dashboard ");
            let root_inner = root_block.inner(layout_root[1]);
            frame.render_widget(root_block, layout_root[1]);

            let titles = [
                " SakuraCat ",
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

            let paragraph = Paragraph::new(vec![
                title_lines[0].clone(),
                Line::from(format!("From: {}", app.akasha_config.subscription_link)),
                Line::from(format!("Update Time: {}", "2026-04-13 09:27")),
                Line::from(format!("Used/Total: {}", "3.20GB / 260GB")),
                title_lines[1].clone(),
                Line::from(format!("Selected: {}", "")),
                Line::from(vec!["Delay: ".into(), "ms".into()]),
                title_lines[2].clone(),
                title_lines[3].clone(),
                title_lines[4].clone(),
                title_lines[5].clone(),
                title_lines[6].clone(),
                title_lines[7].clone(),
                title_lines[8].clone(),
            ]);

            frame.render_widget(paragraph, root_inner);
        }
        CurrentPage::Proxies => {
            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Proxies(Enter/Esc) ");
            let block_inner = block.inner(layout_root[1]);
            let mainwindow = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Length(20), Constraint::Min(0)])
                .split(block_inner);
            frame.render_widget(block, layout_root[1]);

            let selected_proxy_style = Style::default()
                .bg(Color::Yellow)
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD);
            let tab_items: Vec<ListItem> = app
                .proxies_status
                .group_items
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let index = app.proxies_status.group_items[i].1;
                    let selected_proxy_name = &app.proxies_status.proxy_items[i][index];
                    ListItem::new(format!("{}\n({})", item.0, selected_proxy_name))
                        .style(Style::default())
                })
                .collect();
            let tab = List::new(tab_items)
                .block(
                    Block::default()
                        .title(if app.proxies_status.proxy_focus {
                            " Tabs(J/K) ".fg(Color::White)
                        } else {
                            " Tabs(J/K) ".fg(Color::Yellow)
                        })
                        .borders(Borders::RIGHT),
                )
                .highlight_style(selected_tab_style)
                .highlight_symbol(" * ");
            frame.render_stateful_widget(tab, mainwindow[0], &mut app.proxies_status.group_state);

            let [list_area, scrollbar_area] =
                Layout::horizontal([Constraint::Min(0), Constraint::Length(1)])
                    .areas(mainwindow[1]);
            let group_index = app.proxies_status.group_state.selected().unwrap();
            let current_group_delay = &app.proxies_status.delay[group_index];
            let proxies_item: Vec<ListItem> = app.proxies_status.proxy_items[group_index]
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let mut style = Style::default();
                    if i == app.proxies_status.group_items[group_index].1 {
                        style = selected_proxy_style;
                    }
                    // ListItem::new(item.clone()).style(style)
                    // let delay = Some(0);
                    let delay = if let Some(delay) = current_group_delay {
                        if let Some(info) = delay.get(item) {
                            Some(*info as i32)
                        } else {
                            Some(-1)
                        }
                    } else {
                        None
                    };
                    make_item(item, delay, list_area.width).style(style)
                })
                .collect();
            let proxies = List::new(proxies_item)
                .block(Block::default().title(if app.proxies_status.proxy_focus {
                    " Proxies(J/K) ".fg(Color::Yellow)
                } else {
                    " Proxies(J/K) ".fg(Color::White)
                }))
                .highlight_style(selected_tab_style)
                .highlight_symbol(" > ")
                .scroll_padding(2);
            let scrollbar = Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight);
            let mut scrollbar_state =
                ScrollbarState::new(app.proxies_status.proxy_items[group_index].len())
                    .position(app.proxies_status.proxy_state.selected().unwrap_or(0));
            frame.render_stateful_widget(proxies, list_area, &mut app.proxies_status.proxy_state);
            frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
        }
        CurrentPage::Profiles => {
            frame.render_widget(
                Paragraph::new("Now it is selecting profiles.")
                    .block(Block::new().borders(Borders::ALL)),
                layout_root[1],
            );
        }
        CurrentPage::Connections => {
            frame.render_widget(
                Paragraph::new("Now it is selecting connections.")
                    .block(Block::new().borders(Borders::ALL)),
                layout_root[1],
            );
        }
        CurrentPage::Rules => {
            frame.render_widget(
                Paragraph::new("Now it is selecting rules.")
                    .block(Block::new().borders(Borders::ALL)),
                layout_root[1],
            );
        }
        CurrentPage::Logs => {
            frame.render_widget(
                Paragraph::new("Now it is selecting logs.")
                    .block(Block::new().borders(Borders::ALL)),
                layout_root[1],
            );
        }
        CurrentPage::Test => {
            frame.render_widget(
                Paragraph::new(format!("Here is the debug information:\n{}", app.debug))
                    .block(Block::new().borders(Borders::ALL)),
                layout_root[1],
            );
        }
        CurrentPage::Settings => {
            frame.render_widget(
                Paragraph::new(format!("The authors is {}.", app.pkginfo.get_authors()))
                    .block(Block::new().borders(Borders::ALL)),
                layout_root[1],
            );
        }
    }
    frame.render_widget(
        Paragraph::new(format!(
            "{} v{}",
            app.pkginfo.get_name().to_uppercase(),
            app.pkginfo.get_version()
        ))
        .block(Block::new().borders(Borders::ALL)),
        layout_sidebar[0],
    );
    frame.render_stateful_widget(
        sidebar,
        layout_sidebar[1],
        &mut app.sidebar_status.list_state,
    );
    frame.render_widget(chart_traffic_monitor, layout_monitor[0]);
    frame.render_widget(Paragraph::new(traffic_info), layout_monitor[1]);
}

fn make_item<'a>(name: &'a String, value: Option<i32>, width: u16) -> ListItem<'a> {
    let mut timeout = false;
    let left = name;
    let right = match &value {
        Some(value) => {
            if *value == -1 {
                timeout = true;
            }
            format!("{}ms", value)
        }
        None => format!(""),
    };

    let left_width = UnicodeWidthStr::width(left.as_str());
    let right_width = UnicodeWidthStr::width(right.as_str());

    let space_count = width as i32 - left_width as i32 - right_width as i32 - 3;
    let spaces = " ".repeat(space_count.max(1) as usize);

    ListItem::new(Line::from(vec![
        Span::raw(left.to_string()),
        Span::raw(spaces),
        Span::raw(right).style(Style::default().fg(if !timeout {
            match &value {
                Some(value) => {
                    if *value < 250 {
                        Color::Green
                    } else if *value < 500 {
                        Color::Blue
                    } else {
                        Color::LightRed
                    }
                }
                None => Color::Red,
            }
        } else {
            Color::Red
        })),
    ]))
}

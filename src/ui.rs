use ratatui::prelude::*;
use ratatui::widgets::{Axis, Chart, Dataset};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

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
        .1
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

    match app.sidebar_status.2 {
        CurrentPage::Dashboard => {
            let block = Block::default().borders(Borders::ALL).title(" Dashboard ");
            let block_inner = block.inner(layout_root[1]);
            let mainwindow = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Length(20), Constraint::Min(0)])
                .split(block_inner);
            frame.render_widget(block, layout_root[1]);

            let tab_items: Vec<ListItem> = app
                .dashboard_status
                .1
                .iter()
                .map(|i| ListItem::new(*i).style(Style::default().fg(Color::Blue)))
                .collect();
            let tab = List::new(tab_items)
                .block(
                    Block::default()
                        .title(" Tabs(J/K) ")
                        .borders(Borders::RIGHT),
                )
                .highlight_style(
                    Style::default()
                        .bg(Color::White)
                        .fg(Color::Red)
                        .add_modifier(Modifier::ITALIC),
                )
                .highlight_symbol(" * ");
            frame.render_stateful_widget(tab, mainwindow[0], &mut app.dashboard_status.0);
        }
        CurrentPage::Proxies => {
            let block = Block::default().borders(Borders::ALL).title(" Proxies ");
            let block_inner = block.inner(layout_root[1]);
            let mainwindow = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Length(20), Constraint::Min(0)])
                .split(block_inner);
            frame.render_widget(block, layout_root[1]);

            let tab_items: Vec<ListItem> = app
                .proxies_status
                .1
                .iter()
                .map(|i| ListItem::new(i.clone()).style(Style::default().fg(Color::Blue)))
                .collect();
            let tab = List::new(tab_items)
                .block(
                    Block::default()
                        .title(" Tabs(J/K) ")
                        .borders(Borders::RIGHT),
                )
                .highlight_style(
                    Style::default()
                        .bg(Color::White)
                        .fg(Color::Red)
                        .add_modifier(Modifier::ITALIC),
                )
                .highlight_symbol(" * ");
            frame.render_stateful_widget(tab, mainwindow[0], &mut app.proxies_status.0);
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
                Paragraph::new("Now it is selecting test.")
                    .block(Block::new().borders(Borders::ALL)),
                layout_root[1],
            );
        }
        CurrentPage::Settings => {
            frame.render_widget(
                Paragraph::new("Now it is selecting settings.")
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
    frame.render_stateful_widget(sidebar, layout_sidebar[1], &mut app.sidebar_status.0);
    frame.render_widget(chart_traffic_monitor, layout_monitor[0]);
    frame.render_widget(Paragraph::new(traffic_info), layout_monitor[1]);
}

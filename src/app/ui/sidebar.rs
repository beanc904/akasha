use std::rc::Rc;

use ratatui::prelude::*;
use ratatui::widgets::{Axis, Chart, Dataset, List, ListItem, Paragraph};
use ratatui::widgets::{Block, Borders};

use crate::app::App;

pub(super) fn render_sidebar(app: &mut App, frame: &mut Frame, layout_root: &Rc<[Rect]>) {
    // ANCHOR: setup layout
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

    // ANCHOR: setup sidebar
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

    // ANCHOR_END: setup sidebar

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

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
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(layout_root[0]);
    // ANCHOR_END: setup layout

    let datasets = vec![
        Dataset::default()
            .name("mihomo traffic")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Cyan))
            .data(app.data.make_contiguous()),
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
        .y_axis(Axis::default().title("KB/s").bounds([0.0, 2000.0]).labels([
            "0".bold(),
            "1k".into(),
            "2k".into(),
        ]));

    let sidebar_items: Vec<ListItem> = app
        .sidebar_items
        .iter()
        .map(|i| ListItem::new(*i).style(Style::default().fg(Color::White)))
        .collect();

    let sidebar = List::new(sidebar_items)
        .block(Block::default().title(" Menu ").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::White)
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    frame.render_widget(
        Paragraph::new("Left").block(Block::new().borders(Borders::ALL)),
        layout_root[0],
    );
    match app.current_page {
        CurrentPage::Home => {
            frame.render_widget(
                Paragraph::new("Now it is selecting home.")
                    .block(Block::new().borders(Borders::ALL)),
                layout_root[1],
            );
        }
        CurrentPage::Proxies => {
            frame.render_widget(
                Paragraph::new("Now it is selecting proxies.")
                    .block(Block::new().borders(Borders::ALL)),
                layout_root[1],
            );
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
    frame.render_stateful_widget(sidebar, layout_sidebar[1], &mut app.sidebar_state);
    frame.render_widget(chart_traffic_monitor, layout_sidebar[2]);
}

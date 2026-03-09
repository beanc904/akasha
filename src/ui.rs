use ratatui::prelude::*;
use ratatui::widgets::{Axis, Chart, Dataset};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

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
        .constraints(vec![Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(frame.area());
    let layout_sidebar = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(10),
            Constraint::Percentage(60),
            Constraint::Percentage(30),
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
                .title("Mihomo Traffic Monitor")
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .title("time")
                .bounds([app.tick - 120f64, app.tick])
                .labels([
                    format!("{:.0}", app.tick - 120.0).bold(),
                    format!("{:.0}", app.tick - 60.0).into(),
                    format!("{:.0}", app.tick).into(),
                ]),
        )
        .y_axis(Axis::default().title("KB/s").bounds([0.0, 500.0]).labels([
            "0".bold(),
            "250".into(),
            "500".into(),
        ]));

    frame.render_widget(
        Paragraph::new("Left").block(Block::new().borders(Borders::ALL)),
        layout_root[0],
    );
    frame.render_widget(
        Paragraph::new("Right").block(Block::new().borders(Borders::ALL)),
        layout_root[1],
    );
    frame.render_widget(
        Paragraph::new("Akasha v0.0.1").block(Block::new().borders(Borders::ALL)),
        layout_sidebar[0],
    );
    frame.render_widget(
        Paragraph::new("Up").block(Block::new().borders(Borders::ALL).title("Sidebar")),
        layout_sidebar[1],
    );
    frame.render_widget(chart_traffic_monitor, layout_sidebar[2]);
}

mod dashboard;
mod logs;
mod proxies;
use dashboard::*;
use logs::*;
use proxies::*;

pub mod components;
use components::Component;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::{App, CurrentPage};

/// Renders the user interface.
///
/// This is where you add new widgets. See the following resources for more information:
/// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
/// - <https://github.com/ratatui/ratatui/tree/master/examples>
pub fn draw(app: &mut App, frame: &mut Frame) {
    // ANCHOR: setup layout
    let layout_root = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Length(25), Constraint::Min(0)])
        .split(frame.area());
    // ANCHOR_END: setup layout

    app.sidebar.update(&app.dashboard_status.sysproxy);
    app.sidebar.draw(frame, layout_root[0]);

    match app.sidebar.current_page() {
        CurrentPage::Dashboard => {
            render_dashboard(app, frame, &layout_root);
        }
        CurrentPage::Proxies => {
            render_proxies(app, frame, &layout_root);
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
            render_logs(app, frame, &layout_root);
        }
        CurrentPage::Test => {
            frame.render_widget(
                Paragraph::new(format!("Here is the debug information:\n{}", "debug"))
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
}

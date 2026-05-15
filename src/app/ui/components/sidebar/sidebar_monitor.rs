use std::collections::VecDeque;

use ratatui::{
    prelude::*,
    widgets::{Axis, Block, Borders, Chart, Dataset, Paragraph},
};

use crate::app::ui::components::Component;

const KB: f64 = 1024.0;
const MB: f64 = 1024.0 * 1024.0;

pub struct SidebarMonitor {
    /// It is the unit frame of traffic monitor, and also the x_axis.
    tick: f64,
    /// The touple signature is (tick, up, down, upTotal, downTotal). (unit: bps)
    ///
    /// The original ws_traffic data is:
    /// Object {"data": String("{\"up\":0,\"down\":0,\"upTotal\":0,\"downTotal\":0}\n"), "type": String("Text")}
    traffic: VecDeque<(f64, f64, f64, f64, f64)>,
    /// The original ws_memory data is:
    /// {"data":"{\"inuse\":41844736,\"oslimit\":0}\n","type":"Text"} (unit: b)
    memory_inuse: f64,
}

impl SidebarMonitor {
    pub fn new() -> Self {
        Self {
            tick: 0f64,
            traffic: VecDeque::default(),
            memory_inuse: 0f64,
        }
    }

    // pub(super) fn update_tick(&mut self, tick: f64) {
    //     self.tick = tick;
    // }

    pub(super) fn push_back(&mut self, up: f64, down: f64, up_total: f64, down_total: f64) {
        self.tick += 1.0;

        if self.traffic.len() >= 1024 {
            self.traffic.pop_front();
        }

        self.traffic
            .push_back((self.tick, up, down, up_total, down_total));
    }

    pub(super) fn memory_inuse(&mut self, inuse: f64) {
        self.memory_inuse = inuse;
    }
}

impl Component for SidebarMonitor {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        // The origin unit b to kb.
        let up_set: Vec<(f64, f64)> = self
            .traffic
            .iter()
            .map(|&(tick, up, _, _, _)| (tick, up / 1024f64))
            .collect();
        let down_set: Vec<(f64, f64)> = self
            .traffic
            .iter()
            .map(|&(tick, _, down, _, _)| (tick, down / 1024f64))
            .collect();
        // The origin unit: b
        let up_speed = self
            .traffic
            .back()
            .unwrap_or(&(0f64, 0f64, 0f64, 0f64, 0f64))
            .1;
        let down_speed = self
            .traffic
            .back()
            .unwrap_or(&(0f64, 0f64, 0f64, 0f64, 0f64))
            .2;

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

        let widget_chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title(" Traffic Monitor ")
                    .borders(Borders::ALL),
            )
            .x_axis(
                Axis::default()
                    .title("T")
                    .bounds([self.tick - 60f64, self.tick])
                    .labels([
                        format!("{:.0}", self.tick - 60.0).bold(),
                        format!("{:.0}", self.tick - 30.0).into(),
                        format!("{:.0}", self.tick).into(),
                    ]),
            )
            .y_axis(
                Axis::default()
                    .title("KB/s")
                    .bounds([0.0, 10000.0])
                    .labels(["0".bold(), "5k".into(), "10k".into()]),
            );

        let up_speed = make_speed_text("Up", up_speed).cyan().bold();
        let down_speed = make_speed_text("Down", down_speed).light_red().bold();
        let widget_info = Paragraph::new(vec![
            Line::from(up_speed),
            Line::from(down_speed),
            Line::from(format!("Memory Inuse: {:.1} MB", self.memory_inuse / MB)),
        ]);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(0), Constraint::Length(3)])
            .split(area);
        frame.render_widget(widget_chart, layout[0]);
        frame.render_widget(widget_info, layout[1]);
    }
}

fn make_speed_text(kind: &'static str, speed: f64) -> String {
    let (value, unit) = match speed {
        v if v < KB => (speed, "B/s"),
        v if v < MB => (speed / KB, "KB/s"),
        _ => (speed / MB, "MB/s"),
    };
    if unit == "B/s" {
        format!("{} Speed: {:.0} {}", kind, value, unit)
    } else {
        format!("{} Speed: {:.0} {}", kind, value, unit)
    }
}

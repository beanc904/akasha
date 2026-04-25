use std::rc::Rc;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::widgets::{Scrollbar, ScrollbarState};
use unicode_width::UnicodeWidthStr;

use crate::app::App;

pub(super) fn render_proxies(app: &mut App, frame: &mut Frame, layout_root: &Rc<[Rect]>) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Proxies(Enter/Esc) ");
    let block_inner = block.inner(layout_root[1]);
    let mainwindow = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Length(20), Constraint::Min(0)])
        .split(block_inner);
    frame.render_widget(block, layout_root[1]);

    let selected_tab_style = Style::default()
        .bg(Color::White)
        .fg(Color::DarkGray)
        .add_modifier(Modifier::BOLD);
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
            ListItem::new(format!("{}\n({})", item.0, selected_proxy_name)).style(Style::default())
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
        Layout::horizontal([Constraint::Min(0), Constraint::Length(1)]).areas(mainwindow[1]);
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

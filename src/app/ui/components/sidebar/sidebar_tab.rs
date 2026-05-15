use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::app::{CurrentPage, ui::components::Component};

pub struct SidebarTab {
    title: &'static str,
    items: Vec<&'static str>,
    state: ListState,
    selected: CurrentPage,
}

impl SidebarTab {
    pub fn new() -> Self {
        Self {
            title: " Menu(Tab/BackTab) ",
            items: vec![
                "Dashboard",
                "Proxies",
                "Profiles",
                "Connections",
                "Rules",
                "Logs",
                "Test",
                "Settings",
            ],
            state: ListState::default().with_selected(Some(0)),
            selected: CurrentPage::Dashboard,
        }
    }

    pub(super) fn current_page(&self) -> &CurrentPage {
        &self.selected
    }

    /// You must use it after finishing selecting the current page.
    /// Sync the status of enumeration and liststate.
    fn sync_state(&mut self) {
        match self.state.selected() {
            Some(0) => self.selected = CurrentPage::Dashboard,
            Some(1) => self.selected = CurrentPage::Proxies,
            Some(2) => self.selected = CurrentPage::Profiles,
            Some(3) => self.selected = CurrentPage::Connections,
            Some(4) => self.selected = CurrentPage::Rules,
            Some(5) => self.selected = CurrentPage::Logs,
            Some(6) => self.selected = CurrentPage::Test,
            Some(7) => self.selected = CurrentPage::Settings,
            Some(_) => panic!("Switching over array bound!"),
            None => panic!("There is something wrong with switching page."),
        }
    }

    pub(super) fn tab_next(&mut self) {
        np_switch(true, &mut self.state, self.items.len());
        self.sync_state();
    }

    pub(super) fn tab_previous(&mut self) {
        np_switch(false, &mut self.state, self.items.len());
        self.sync_state();
    }

    pub(super) fn tab_switch(&mut self, index: usize) {
        self.state.select(Some(index));
        self.sync_state();
    }
}

impl Component for SidebarTab {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                ListItem::new(format!("({}) {}", idx + 1, item))
                    .style(Style::default().fg(Color::White))
            })
            .collect();
        let widget = List::new(items)
            .block(Block::default().title(self.title).borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .bg(Color::White)
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(widget, area, &mut self.state);
    }
}

fn np_switch(is_next: bool, state: &mut ListState, items_len: usize) {
    match is_next {
        true => {
            // Switch to the next tab
            let index = match state.selected() {
                Some(i) => {
                    if i >= items_len - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            state.select(Some(index));
        }
        false => {
            // Switch to the previous tab
            let index = match state.selected() {
                Some(i) => {
                    if i == 0 {
                        items_len - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            state.select(Some(index));
        }
    }
}

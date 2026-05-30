use std::{collections::HashMap, sync::Arc, time::Duration};

use akasha::{client::mihomo::Mihomo, parser::config::ProxyGroup};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{
        Block, Borders, List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
};
use tokio::{
    sync::{RwLock, broadcast, mpsc},
    time::interval,
};
use unicode_width::UnicodeWidthStr;

use crate::app::ui::components::Component;

enum Focus {
    Group,
    Proxy,
}

struct GroupItem {
    name: String,
    subitems: Vec<ProxyItem>,
    current_idx: usize,
}

struct ProxyItem {
    name: String,
    delay: Option<i32>,
}

/// Field [`state`] contains the liststate data structure of (group, proxy)
pub struct Proxies {
    groups: Vec<GroupItem>,
    state: (ListState, ListState),
    focus: Focus,
    rx_delay: Option<mpsc::Receiver<(HashMap<String, u32>, usize)>>,
}

impl Proxies {
    pub fn new(proxy_groups: &Vec<ProxyGroup>) -> Self {
        let groups = proxy_groups
            .iter()
            .map(|group| {
                let proxies = (&group.proxies)
                    .iter()
                    .map(|proxy| ProxyItem {
                        name: proxy.clone(),
                        delay: None,
                    })
                    .collect();
                GroupItem {
                    name: group.name.clone(),
                    subitems: proxies,
                    current_idx: 0,
                }
            })
            .collect();

        Self {
            groups,
            state: (
                ListState::default().with_selected(Some(0)),
                ListState::default().with_selected(Some(0)),
            ),
            focus: Focus::Group,
            rx_delay: None,
        }
    }

    /// Need to delete after stable edition.
    pub fn current_proxy(&self) -> String {
        let current_idx = self.groups[0].current_idx;
        self.groups[0].subitems[current_idx].name.clone()
    }

    pub async fn handle_key_event(
        &mut self,
        key: KeyEvent,
        mihomo: Arc<RwLock<Mihomo>>,
        test_url: String,
    ) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Enter) => self.enter_handler(mihomo).await,
            (_, KeyCode::Esc | KeyCode::Char('h')) => self.focus = Focus::Group,
            (_, KeyCode::Char('l')) => self.focus = Focus::Proxy,
            (_, KeyCode::Char('j')) => self.j_handler(),
            (_, KeyCode::Char('k')) => self.k_handler(),
            (_, KeyCode::Char('d')) => self.d_handler(mihomo, test_url).await,
            _ => {}
        }
    }

    async fn enter_handler(&mut self, mihomo: Arc<RwLock<Mihomo>>) {
        match self.focus {
            Focus::Group => self.focus = Focus::Proxy,
            Focus::Proxy => {
                let group_idx = self.state.0.selected().unwrap();
                let proxy_idx = self.state.1.selected().unwrap();
                let group_name = self.groups[group_idx].name.clone();
                let node = self.groups[group_idx].subitems[proxy_idx].name.clone();
                let _ = akasha::client::select_node_for_group(mihomo, group_name, node);
            }
        }
    }

    fn j_handler(&mut self) {
        match self.focus {
            Focus::Group => {
                np_switch(true, &mut self.state.0, self.groups.len());
                self.state.1.select(Some(0));
            }
            Focus::Proxy => {
                let groups_idx = self.state.0.selected().unwrap();
                np_switch(true, &mut self.state.1, self.groups[groups_idx].subitems.len());
            }
        }
    }

    fn k_handler(&mut self) {
        match self.focus {
            Focus::Group => {
                np_switch(false, &mut self.state.0, self.groups.len());
                self.state.1.select(Some(0));
            }
            Focus::Proxy => {
                let groups_idx = self.state.0.selected().unwrap();
                np_switch(false, &mut self.state.1, self.groups[groups_idx].subitems.len());
            }
        }
    }

    async fn d_handler(&mut self, mihomo: Arc<RwLock<Mihomo>>, test_url: String) {
        let (tx_delay, rx_delay) = mpsc::channel::<(HashMap<String, u32>, usize)>(64);
        let group_idx = self.state.0.selected().unwrap();
        let group_name = self.groups[group_idx].name.clone();
        let timeout = 5000;
        let keep_fixed = true;
        tokio::spawn(async move {
            let delay =
                akasha::client::delay_group(mihomo, group_name, test_url, timeout, keep_fixed)
                    .await;
            if let Ok(delay) = delay {
                let _ = tx_delay.send((delay, group_idx)).await;
            } else {
                log::error!("Encountered some problems with index:{group_idx} delay test.");
            }
        });
        self.rx_delay = Some(rx_delay);
    }

    pub fn launch_server(
        &self,
        mihomo: Arc<RwLock<Mihomo>>,
    ) -> (broadcast::Receiver<Vec<String>>, broadcast::Receiver<Vec<String>>) {
        let (tx_proxies, rx_proxies) = broadcast::channel(64);
        let rx_proxies_dash = tx_proxies.subscribe();

        let groups_name: Vec<String> = self.groups.iter().map(|group| group.name.clone()).collect();
        let mut ticker = interval(Duration::from_secs(5));
        tokio::spawn(async move {
            loop {
                ticker.tick().await;
                let mut current_proxy: Vec<String> = vec![];
                let mi = mihomo.read().await;
                for group_name in &groups_name {
                    let proxy = mi.get_proxy_by_name(group_name).await;
                    if let Ok(proxy) = proxy {
                        if let Some(current) = proxy.now {
                            current_proxy.push(current);
                        }
                    } else {
                        panic!("Encountered error when get the current proxy.");
                    }
                }
                let _ = tx_proxies.send(current_proxy);
            }
        });

        (rx_proxies, rx_proxies_dash)
    }

    pub fn sync_client(&mut self, rx_proxies: &mut broadcast::Receiver<Vec<String>>) {
        if let Ok(value) = rx_proxies.try_recv() {
            log::trace!("Proxies Groups selected status: {:?}", value);
            if value.len() == self.groups.len() {
                self.groups.iter_mut().enumerate().for_each(|(idx, group)| {
                    group.current_idx = group
                        .subitems
                        .iter()
                        .position(|proxy| proxy.name == value[idx])
                        .unwrap();
                });
            } else {
                panic!("There is something wrong with group size.");
            }
        }
    }

    pub fn update(&mut self) {
        // Update delay information
        if let Some(rx_delay) = &mut self.rx_delay {
            if let Ok(value) = rx_delay.try_recv() {
                let (delay, group_idx) = value;
                self.groups[group_idx]
                    .subitems
                    .iter_mut()
                    .for_each(|proxy| {
                        proxy.delay = match delay.get(&proxy.name) {
                            Some(num) => Some(*num as i32),
                            None => Some(-1),
                        }
                    });
            }
        }
    }
}

impl Component for Proxies {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let root_block = Block::default()
            .borders(Borders::ALL)
            .title(" Proxies(Enter/Esc) ");
        let root_inner = root_block.inner(area);
        let [groups_area, proxies_area, scroll_area] = Layout::horizontal([
            Constraint::Length(20),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas(root_inner);

        let groups_items: Vec<ListItem> = self
            .groups
            .iter()
            .map(|group| {
                let current_idx = group.current_idx;
                ListItem::new(format!("{}\n({})", group.name, group.subitems[current_idx].name))
            })
            .collect();
        let groups_widget = List::new(groups_items)
            .block(
                Block::default()
                    .title(match self.focus {
                        Focus::Group => " Tabs(J/K) ".fg(Color::Yellow),
                        Focus::Proxy => " Tabs(J/K) ".fg(Color::White),
                    })
                    .borders(Borders::RIGHT),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::White)
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(" * ");

        let groups_state_idx = self.state.0.selected().unwrap();
        let current_idx = self.groups[groups_state_idx].current_idx;
        let proxies_items: Vec<ListItem> = self.groups[groups_state_idx]
            .subitems
            .iter()
            .enumerate()
            .map(|(idx, proxy)| {
                make_item(&proxy.name, &proxy.delay, proxies_area.width).style(
                    if idx == current_idx {
                        Style::default()
                            .bg(Color::Yellow)
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    },
                )
            })
            .collect();
        let proxies_widget = List::new(proxies_items)
            .block(Block::default().title(match self.focus {
                Focus::Group => " Proxies(J/K) ".fg(Color::White),
                Focus::Proxy => " Proxies(J/K) ".fg(Color::Yellow),
            }))
            .highlight_style(
                Style::default()
                    .bg(Color::White)
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(" > ")
            .scroll_padding(2);

        let scrollbar_widget = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        let mut scroll_state = ScrollbarState::new(self.groups[groups_state_idx].subitems.len())
            .position(self.state.1.selected().unwrap());

        frame.render_widget(root_block, area);
        frame.render_stateful_widget(groups_widget, groups_area, &mut self.state.0);
        frame.render_stateful_widget(proxies_widget, proxies_area, &mut self.state.1);
        frame.render_stateful_widget(scrollbar_widget, scroll_area, &mut scroll_state);
    }
}

/// We have to distinguish the the difference between [`None`] and "timeout".
/// So, i decide to let:
///
/// `value = None` to express do not make delay test
/// `value = -1` to express delay test return [`timeout`]
fn make_item<'a>(name: &'a String, value: &Option<i32>, width: u16) -> ListItem<'a> {
    let left = name;
    let right = match &value {
        Some(value) => format!("{}ms", value),
        None => format!(""),
    };

    let left_width = UnicodeWidthStr::width(left.as_str());
    let right_width = UnicodeWidthStr::width(right.as_str());

    let space_count = width as i32 - left_width as i32 - right_width as i32 - 3;
    let spaces = " ".repeat(space_count.max(1) as usize);

    ListItem::new(Line::from(vec![
        Span::raw(left.to_string()),
        Span::raw(spaces),
        Span::raw(right).style(Style::default().fg(match value {
            Some(-1) => Color::Red,
            Some(0..250) => Color::Green,
            Some(250..500) => Color::Blue,
            Some(_) => Color::Yellow,
            None => Color::Red,
        })),
    ]))
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

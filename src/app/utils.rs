use std::sync::Arc;

use akasha::client as ac;
use akasha::client::mihomo::Mihomo;
use ratatui::widgets::ListState;
use tokio::sync::RwLock;

use crate::app::{DashboardStatus, LogsStatus, ProxiesStatus};

impl DashboardStatus {
    pub(super) fn get_updatetime(&self) -> String {
        match &self.subscription_info {
            Some(subscription) => {
                let update_time = subscription.get_updatetime();
                format!("{:?}", update_time)
            }
            None => {
                format!("time err")
            }
        }
    }

    pub(super) fn get_usage(&self) -> String {
        match &self.subscription_info {
            Some(subscription) => {
                let usage = subscription.parse_usage();
                if let Some(usage) = usage {
                    format!(
                        "{} MB / {} MB",
                        (usage.download + usage.upload) / 1024 / 1024,
                        usage.total / 1024 / 1024
                    )
                } else {
                    format!("usage err")
                }
            }
            None => {
                format!("usage err")
            }
        }
    }

    pub(super) fn j_handler(&mut self) {
        let max = self.get_posmax();
        let step = 1;
        let pos = &mut self.scrollbar_pos;
        if *pos >= max {
            *pos = max;
        } else {
            *pos += step;
        }
    }

    pub(super) fn k_handler(&mut self) {
        // let max = 43 - 1;
        let step = 1;
        let pos = &mut self.scrollbar_pos;
        if *pos == 0 || (*pos as i32 - step as i32) < 0 {
            *pos = 0;
        } else {
            *pos -= step;
        }
    }

    pub(super) fn get_posmax(&self) -> usize {
        let para_lines: usize = self.sublabels.iter().map(|row| row.len()).sum();
        let title_lines = self.titles.len();
        let total_lines = para_lines + title_lines;
        total_lines - self.viewport_height as usize + 2
    }
}

impl ProxiesStatus {
    pub(super) fn tab_switch(&mut self, is_next: bool) {
        if !self.proxy_focus {
            // Now the focus is the groups list.
            liststate_switch(is_next, &mut self.group_state, self.group_items.len());
            // // It seems that it does not need to update enumeration.
            // self.update_liststate_status();
            // Reset the selected proxy item, each time switch the groups tab.
            self.proxy_state.select(Some(0));
        } else {
            // Now the focus is the details list.
            let index = self.group_state.selected().unwrap();
            liststate_switch(
                is_next,
                &mut self.proxy_state,
                self.proxy_items[index].len(),
            );
        }
    }

    pub(super) async fn enter_handler(&mut self, mihomo: Arc<RwLock<Mihomo>>) {
        if self.proxy_focus {
            // Cursor at details
            let index_group = self.group_state.selected().unwrap();
            let index_proxy = self.proxy_state.selected().unwrap();
            self.group_items[index_group].1 = index_proxy;

            let name_group = &self.group_items[index_group].0;
            let name_proxy = &self.proxy_items[index_group][index_proxy];

            let _ =
                ac::select_node_for_group(mihomo, name_group.to_string(), name_proxy.to_string())
                    .await;
        } else {
            // Cursor at tabs
            self.proxy_focus = true;
        }
    }

    pub(super) fn esc_handler(&mut self) {
        self.proxy_focus = false;
    }

    pub(super) fn l_handler(&mut self) {
        self.proxy_focus = true;
    }

    pub(super) fn h_handler(&mut self) {
        self.proxy_focus = false;
    }

    pub(super) async fn d_handler(&mut self, mihomo: Arc<RwLock<Mihomo>>, test_url: String) {
        let tx_delay = self.delay_mpsc.0.clone();
        let group_index = self.group_state.selected().unwrap();
        let group_name = self.group_items[group_index].0.clone();
        // let test_url = "https://www.gstatic.com/generate_204".to_string();
        // let test_url = "http://cp.cloudflare.com/generate_204".to_string();
        let timeout = 5000;
        let keep_fixed = true;
        tokio::spawn(async move {
            let delay =
                ac::delay_group(mihomo, group_name.clone(), test_url, timeout, keep_fixed).await;
            if let Ok(delay) = delay {
                let _ = tx_delay.send((Some(delay), group_index)).await;
            } else {
                log::error!("Encountered some problems with {} delay test.", group_name);
            }
        });
    }

    pub(super) fn get_selected_node(&self) -> String {
        if let Some(item) = self.group_items.get(0) {
            let idx = item.1;
            self.proxy_items[0][idx].clone()
        } else {
            format!("selected err")
        }
    }
}

impl LogsStatus {
    pub(super) fn get_inner(&self) -> Arc<std::sync::RwLock<Vec<String>>> {
        self.log_state.get_inner()
    }

    pub(super) fn get_all_len(&self) -> usize {
        self.log_state.all_len()
    }

    pub(super) fn get_scrollbar_pos(&self) -> (usize, usize) {
        self.scrollbar_pos
    }

    pub(super) fn j_handler(&mut self) {
        let max = self.get_all_len() - 1;
        let step = self.step_len;
        let pos = &mut self.scrollbar_pos.0;
        if *pos >= max {
            *pos = 0;
        } else {
            *pos += step;
        }
    }

    pub(super) fn k_handler(&mut self) {
        let max = self.get_all_len() - 1;
        let step = self.step_len;
        let pos = &mut self.scrollbar_pos.0;
        if *pos == 0 || (*pos as i32 - step as i32) < 0 {
            *pos = max - step;
        } else {
            *pos -= step;
        }
    }

    pub(super) fn h_handler(&mut self) {
        let step = self.step_len;
        let pos = &mut self.scrollbar_pos.1;
        if *pos != 0 {
            *pos -= step;
        }
    }

    pub(super) fn l_handler(&mut self) {
        let step = self.step_len;
        let pos = &mut self.scrollbar_pos.1;
        *pos += step;
    }
}

fn liststate_switch(is_next: bool, state: &mut ListState, items_len: usize) {
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

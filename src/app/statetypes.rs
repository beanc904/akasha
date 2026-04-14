enum CurrentPage {
    Dashboard,
    Proxies,
    Profiles,
    Connections,
    Rules,
    Logs,
    Test,
    Settings,
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

struct SidebarStatus {
    list_state: ListState,
    list_items: Vec<&'static str>,
    current_page: CurrentPage,
}

impl SidebarStatus {
    /// You must use it after finishing selecting the current page.
    /// Sync the status of enumeration and liststate.
    fn update_liststate_status(&mut self) {
        match self.list_state.selected() {
            Some(0) => self.current_page = CurrentPage::Dashboard,
            Some(1) => self.current_page = CurrentPage::Proxies,
            Some(2) => self.current_page = CurrentPage::Profiles,
            Some(3) => self.current_page = CurrentPage::Connections,
            Some(4) => self.current_page = CurrentPage::Rules,
            Some(5) => self.current_page = CurrentPage::Logs,
            Some(6) => self.current_page = CurrentPage::Test,
            Some(7) => self.current_page = CurrentPage::Settings,
            Some(_) => log::error!("Switching over array bound!"),
            None => log::error!("There is something wrong with switching page."),
        }
    }

    fn sidebar_next(&mut self) {
        liststate_switch(true, &mut self.list_state, self.list_items.len());
        self.update_liststate_status();
    }

    fn sidebar_previous(&mut self) {
        liststate_switch(false, &mut self.list_state, self.list_items.len());
        self.update_liststate_status();
    }
}

struct ProxiesStatus {
    group_state: ListState,
    group_items: Vec<(String, usize)>,
    proxy_state: ListState,
    proxy_items: Vec<Vec<String>>,
    proxy_focus: bool,
    delay: Vec<Option<HashMap<String, u32>>>,
    delay_mpsc: (
        mpsc::Sender<(Option<HashMap<String, u32>>, usize)>,
        mpsc::Receiver<(Option<HashMap<String, u32>>, usize)>,
    ),
}

impl ProxiesStatus {
    fn tab_switch(&mut self, is_next: bool) {
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

    async fn enter_handler(&mut self, mihomo: &Arc<RwLock<Mihomo>>) {
        if self.proxy_focus {
            // Cursor at details
            let index_group = self.group_state.selected().unwrap();
            let index_proxy = self.proxy_state.selected().unwrap();
            self.group_items[index_group].1 = index_proxy;

            let name_group = &self.group_items[index_group].0;
            let name_proxy = &self.proxy_items[index_group][index_proxy];

            let mihomo = mihomo.clone();
            let _ =
                ac::select_node_for_group(mihomo, name_group.to_string(), name_proxy.to_string())
                    .await;
        } else {
            // Cursor at tabs
            self.proxy_focus = true;
        }
    }

    fn esc_handler(&mut self) {
        self.proxy_focus = false;
    }

    fn l_handler(&mut self) {
        self.proxy_focus = true;
    }

    fn h_handler(&mut self) {
        self.proxy_focus = false;
    }

    async fn d_handler(&mut self, mihomo: &Arc<RwLock<Mihomo>>) {
        let mihomo = mihomo.clone();
        let tx_delay = self.delay_mpsc.0.clone();
        let group_index = self.group_state.selected().unwrap();
        let group_name = self.group_items[group_index].0.clone();
        let test_url = "https://www.gstatic.com/generate_204".to_string();
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
}

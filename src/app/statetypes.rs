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

struct LogsStatus {
    log_state: Arc<LogStore>,
    scrollbar_pos: (usize, usize),
    step_len: usize,
}

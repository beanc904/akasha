use std::{borrow::Cow, sync::Arc, time::Duration};

use akasha::{
    client::mihomo::Mihomo,
    parser::{config::AkashaConfig, request::SubscriptionInfo},
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders},
};
use sysproxy::Sysproxy;
use tokio::{
    sync::{RwLock, broadcast, mpsc, mpsc::Receiver},
    time::interval,
};

use crate::app::ui::widgets::scrolltext::ScrollText;

// const KB: u64 = 1024;
const MB: u64 = 1024 * 1024;

pub struct Dashboard {
    sections: [Vec<&'static str>; 9],
    subscription_info: Option<SubscriptionInfo>,
    current_node: Option<String>,
    node_delay: u32,
    sysproxy: Option<Sysproxy>,
    scrolltext: ScrollText,
}

impl Dashboard {
    pub fn new() -> Self {
        let sections = [
            vec!["Profiles", "From: ", "Update Time: ", "Used / Total: "],
            vec!["CurrentNode", "Selected: ", "Delay: "],
            vec!["NetworkSettings", "System Proxy: ", "Tun Mode: "],
            vec!["ProxyMode", "Mode: "],
            vec![
                "TrafficStats",
                "Upload Speed: ",
                "Download Speed: ",
                "Uploaded: ",
                "Downloaded: ",
                "Active Connections: ",
                "Core Usage: ",
            ],
            vec![
                "WebsiteTests",
                "Apple: ",
                "GitHub: ",
                "Google: ",
                "YouTube: ",
            ],
            vec![
                "IpInformation",
                "IP: ",
                "ASN: ",
                "ISP: ",
                "ORG: ",
                "Location: ",
                "Timezone: ",
            ],
            vec![
                "ClashInfo",
                "Core Version: ",
                "System Proxy Address: ",
                "Mixed Port: ",
                "Uptime: ",
                "Rules Count: ",
            ],
            vec![
                "SystemInfo",
                "OS Info: ",
                "Auto Launch: ",
                "Running Mode: ",
                "Last Check Update: ",
                "Akasha Version: ",
            ],
        ];

        let content = sections
            .iter()
            .map(|section| {
                section
                    .iter()
                    .enumerate()
                    .map(|(idx, sec)| {
                        if idx == 0 {
                            Line::from(format!(">>> {} <<<", sec)).bg(Color::DarkGray)
                        } else {
                            Line::from(sec.to_string()).underlined()
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>();

        Self {
            sections,
            subscription_info: None,
            current_node: None,
            node_delay: 0,
            sysproxy: None,
            scrolltext: ScrollText::new(2, content),
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Char('j')) => self.scrolltext.j_handler(),
            (_, KeyCode::Char('k')) => self.scrolltext.k_handler(),
            _ => {}
        }
    }

    pub fn update(&mut self, sysproxy: Option<Sysproxy>, current_node: String) {
        self.sysproxy = sysproxy;
        self.current_node = Some(current_node);
    }

    // pub fn j_handler(&mut self) {
    //     self.scrolltext.j_handler();
    // }

    // pub fn k_handler(&mut self) {
    //     self.scrolltext.k_handler();
    // }

    fn get_updatetime(&self) -> String {
        match &self.subscription_info {
            Some(time) => format!("{:?}", time.get_updatetime()),
            None => format!("time err"),
        }
    }

    fn get_usage(&self) -> String {
        let err = format!("usage err");
        match &self.subscription_info {
            Some(ss) => match ss.parse_usage() {
                Some(usage) => format!(
                    "{} MB / {} MB",
                    (usage.download + usage.upload) / MB,
                    usage.total / MB
                ),
                None => err,
            },
            None => err,
        }
    }

    pub fn launch_server(
        &self,
        akasha_config: &AkashaConfig,
        mihomo: Arc<RwLock<Mihomo>>,
        mut rx_proxies: broadcast::Receiver<Vec<String>>,
    ) -> (Receiver<Option<SubscriptionInfo>>, Receiver<u32>) {
        // ANCHOR: Initialize the subscription information.
        let (tx_subscription, rx_subscription) = mpsc::channel::<Option<SubscriptionInfo>>(64);
        if self.subscription_info.is_none() {
            let url = akasha_config.subscription_link();
            tokio::spawn(async move {
                let sub_info = SubscriptionInfo::new(url).await;
                let bundle = sub_info.ok();
                let _ = tx_subscription.send(bundle).await;
            });
        }
        // ANCHOR_END: Initialize the subscription information.

        // ANCHOR: setup the selected node delay info getter
        let (tx_node_delay, rx_node_delay) = mpsc::channel::<u32>(64);
        let test_url = akasha_config.test_url();
        let timeout = 5000;
        let mut ticker_node_delay_task = interval(Duration::from_secs(5));
        tokio::spawn(async move {
            loop {
                ticker_node_delay_task.tick().await;
                let mi = mihomo.read().await;
                if let Ok(value) = rx_proxies.try_recv() {
                    let delay = mi.delay_proxy_by_name(&value[0], &test_url, timeout).await;
                    let _ = tx_node_delay.send(delay.unwrap().delay).await;
                }
            }
        });
        // ANCHOR_END: setup the selected node delay info getter

        (rx_subscription, rx_node_delay)
    }

    pub fn sync_client(
        &mut self,
        rx_subscription: &mut Receiver<Option<SubscriptionInfo>>,
        rx_delay: &mut Receiver<u32>,
    ) {
        // Initialize subscription
        if let Ok(bundle) = rx_subscription.try_recv() {
            self.subscription_info = bundle;
        }

        if let Ok(value) = rx_delay.try_recv() {
            log::info!("Selected node delay: {}", value);
            self.node_delay = value;
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect, link: String) {
        let label_style = Style::new().underlined();

        // ANCHOR: getting time and usage info
        let time_txt = self.get_updatetime();
        let usage_txt = self.get_usage();
        // ANCHOR_END: getting time and usage info

        // ANCHOR: getting selected node and delay info
        let selected_txt = self.current_node.clone().unwrap_or("xxx".to_string());
        // let mihomo = app.mihomo.clone();
        // let node_delay_txt = app.proxies_status.get_selected_node_delay(mihomo);
        let node_delay = Span::raw(format!("{} ms", self.node_delay)).style(Style::default().fg(
            match self.node_delay {
                0 => Color::Red,
                1..250 => Color::Green,
                250..500 => Color::Blue,
                _ => Color::Yellow,
            },
        ));
        // ANCHOR_END: getting selected node and delay info

        // ANCHOR: getting system proxy status
        let sysproxy = &self.sysproxy;
        let (sysproxy_txt, sysproxy_style) = match sysproxy {
            Some(Sysproxy { enable, .. }) => {
                let style = Style::default();
                if *enable {
                    ("ON", style.fg(Color::Green))
                } else {
                    ("OFF", style.fg(Color::Red))
                }
            }
            None => ("none", Style::default()),
        };
        let system_proxy = Span::raw(sysproxy_txt).style(sysproxy_style);
        // ANCHOR_END: getting system proxy status

        let profiles_section = vec![
            Line::from(format!(">>> {} <<<", self.sections[0][0])).bg(Color::DarkGray),
            one_line(self.sections[0][1], link, label_style),
            one_line(self.sections[0][2], time_txt, label_style),
            one_line(self.sections[0][3], usage_txt, label_style),
        ];
        let currentnode_section = vec![
            Line::from(format!(">>> {} <<<", self.sections[1][0])).bg(Color::DarkGray),
            one_line(self.sections[1][1], selected_txt, label_style),
            // one_line("Delay: ", node_delay, underline_style),
            Line::from(vec![
                Span::styled(self.sections[1][2], label_style),
                node_delay,
            ]),
        ];
        let networksettings_section = vec![
            Line::from(format!(">>> {} <<<", self.sections[2][0])).bg(Color::DarkGray),
            // one_line(self.sections[2][0], sysproxy_txt, label_style),
            Line::from(vec![
                Span::styled(self.sections[2][1], label_style),
                system_proxy,
            ]),
            one_line(self.sections[2][2], "xxx", label_style),
        ];
        let proxymode_section = vec![
            Line::from(format!(">>> {} <<<", self.sections[3][0])).bg(Color::DarkGray),
            one_line(self.sections[3][1], "xxx", label_style),
        ];
        let trafficstats_section = vec![
            Line::from(format!(">>> {} <<<", self.sections[4][0])).bg(Color::DarkGray),
            one_line(self.sections[4][1], "xxx", label_style),
            one_line(self.sections[4][2], "xxx", label_style),
            one_line(self.sections[4][3], "xxx", label_style),
            one_line(self.sections[4][4], "xxx", label_style),
            one_line(self.sections[4][5], "xxx", label_style),
            one_line(self.sections[4][6], "xxx", label_style),
        ];
        let websitetests_section = vec![
            Line::from(format!(">>> {} <<<", self.sections[5][0])).bg(Color::DarkGray),
            one_line(self.sections[5][1], "xxx", label_style),
            one_line(self.sections[5][2], "xxx", label_style),
            one_line(self.sections[5][3], "xxx", label_style),
            one_line(self.sections[5][4], "xxx", label_style),
        ];
        let ipinfo_section = vec![
            Line::from(format!(">>> {} <<<", self.sections[6][0])).bg(Color::DarkGray),
            one_line(self.sections[6][1], "xxx", label_style),
            one_line(self.sections[6][2], "xxx", label_style),
            one_line(self.sections[6][3], "xxx", label_style),
            one_line(self.sections[6][4], "xxx", label_style),
            one_line(self.sections[6][5], "xxx", label_style),
            one_line(self.sections[6][6], "xxx", label_style),
        ];
        let clashinfo_section = vec![
            Line::from(format!(">>> {} <<<", self.sections[7][0])).bg(Color::DarkGray),
            one_line(self.sections[7][1], "xxx", label_style),
            one_line(self.sections[7][2], "xxx", label_style),
            one_line(self.sections[7][3], "xxx", label_style),
            one_line(self.sections[7][4], "xxx", label_style),
            one_line(self.sections[7][5], "xxx", label_style),
        ];
        let sysinfo_section = vec![
            Line::from(format!(">>> {} <<<", self.sections[8][0])).bg(Color::DarkGray),
            one_line(self.sections[8][1], "xxx", label_style),
            one_line(self.sections[8][2], "xxx", label_style),
            one_line(self.sections[8][3], "xxx", label_style),
            one_line(self.sections[8][4], "xxx", label_style),
            one_line(self.sections[8][5], "xxx", label_style),
        ];

        let content = [
            profiles_section,
            currentnode_section,
            networksettings_section,
            proxymode_section,
            trafficstats_section,
            websitetests_section,
            ipinfo_section,
            clashinfo_section,
            sysinfo_section,
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let root_block = Block::default().borders(Borders::ALL).title(" Dashboard ");
        let root_inner = root_block.inner(area);
        frame.render_widget(root_block, area);
        self.scrolltext.content(content);
        // Please modify the [`self.scrolltext.content`] first.
        self.scrolltext.render(frame, root_inner);
    }
}

fn one_line<'a, T, P, S>(label: T, content: P, label_style: S) -> Line<'a>
where
    T: Into<Cow<'a, str>>,
    P: Into<Cow<'a, str>>,
    S: Into<Style>,
{
    Line::from(vec![
        Span::styled(label.into(), label_style),
        Span::raw(content.into()),
    ])
}

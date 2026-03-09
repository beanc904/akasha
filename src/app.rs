use std::collections::VecDeque;
use std::sync::Arc;

use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use ratatui::DefaultTerminal;
use serde_json::Value;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{Duration, interval};

use akasha::client as ac;

// #[derive(Debug, Default)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    // Event stream.
    pub event_stream: EventStream,
    // Mihomo handle.
    pub mihomo: Arc<RwLock<ac::mihomo::Mihomo>>,

    // ANCHOR: demo
    pub data: VecDeque<(f64, f64)>,
    pub tick: f64,
    // ANCHOR_END: demo
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        // Self::default()
        App {
            running: true,
            event_stream: EventStream::default(),
            mihomo: ac::Builder::new()
                .protocol(ac::Protocol::LocalSocket)
                .socket_path("/tmp/mihomo.sock".to_string())
                .pool_config(
                    ac::IpcPoolConfigBuilder::new()
                        .min_connections(0)
                        .max_connections(20)
                        .idle_timeout(std::time::Duration::from_millis(500))
                        .health_check_interval(std::time::Duration::from_secs(10))
                        .build(),
                )
                .build()
                .unwrap(),
            data: VecDeque::default(),
            tick: 0f64,
        }
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.running = true;

        // ANCHOR: chart demo
        let (tx, mut rx) = mpsc::channel::<Value>(64);
        let mihomo_clone = self.mihomo.clone();
        tokio::spawn(ac::ws_traffic(mihomo_clone, tx));
        // let handle_client = tokio::spawn(async move {
        //     while let Some(msg) = rx.recv().await {
        //         // print!("Traffic information: {:?}\r\n", msg);
        //         let inner_json_str = msg["data"].as_str().unwrap().trim();
        //         let inner_value: serde_json::Value = serde_json::from_str(inner_json_str).unwrap();
        //         let up = inner_value["up"].as_f64().unwrap();
        //         // let down = inner_value["down"].as_i64().unwrap();

        //         self.tick += 0.02;

        //         if self.data.len() >= 1024 {
        //             self.data.pop_front();
        //         }

        //         self.data.push_back((self.tick, up));
        //     }
        // });
        let mut ticker = interval(Duration::from_millis(200));
        // ANCHOR_END: chart demo

        while self.running {
            tokio::select! {
                _ = ticker.tick() => {
                    // ANCHOR: chart demo
                    // let value = (self.tick.sin() * 200.0 + 250.0).abs();
                    // self.tick += 0.02;

                    // if self.data.len() >= 1024 {
                    //     self.data.pop_front();
                    // }

                    // self.data.push_back((self.tick, value));
                    // ANCHOR_END: chart demo
                    if let Some(msg) = rx.recv().await {
                        // print!("Traffic information: {:?}\r\n", msg);
                        let inner_json_str = msg["data"].as_str().unwrap().trim();
                        let inner_value: serde_json::Value = serde_json::from_str(inner_json_str).unwrap();
                        let up = inner_value["up"].as_f64().unwrap();
                        // let down = inner_value["down"].as_i64().unwrap();

                        self.tick += 1.0;

                        if self.data.len() >= 1024 {
                            self.data.pop_front();
                        }

                        self.data.push_back((self.tick, up));
                    }

                    terminal.draw(|frame| crate::ui::draw(&mut self, frame))?;
                    // self.handle_crossterm_events().await?;
                }
                maybe_event = self.event_stream.next() => {
                    if let Some(Ok(evt)) = maybe_event {
                        self.handle_crossterm_events(evt).await?;
                    }
                }
            }
        }

        // let _ = tokio::join!(handle_server, handle_client);
        Ok(())
    }

    /// Reads the crossterm events and updates the state of [`App`].
    async fn handle_crossterm_events(&mut self, evt: Event) -> color_eyre::Result<()> {
        // let event = self.event_stream.next().fuse().await;
        // match event {
        match evt {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}

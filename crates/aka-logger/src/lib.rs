pub mod export;
pub mod logger;
pub mod store;

use log::LevelFilter;
#[cfg(feature = "export")]
use std::path::Path;
use std::sync::Arc;

pub use store::LogStore;

#[cfg(feature = "export")]
pub struct LoggerConfig {
    pub buf_capacity: usize,
    pub level: LevelFilter,
    pub with_stdout: bool,
    pub log_path: Box<Path>,
}

#[cfg(not(feature = "export"))]
pub struct LoggerConfig {
    pub buf_capacity: usize,
    pub level: LevelFilter,
    pub with_stdout: bool,
}

pub struct AkaLogger;

impl AkaLogger {
    pub fn init(config: LoggerConfig) -> Arc<LogStore> {
        #[cfg(feature = "export")]
        let store = Arc::new(LogStore::new(config.buf_capacity, config.log_path));
        #[cfg(not(feature = "export"))]
        let store = Arc::new(LogStore::new(config.buf_capacity));

        let logger = logger::InnerLogger {
            store: (*store).clone(),
            level: config.level,

            #[cfg(feature = "env-forward")]
            env: env_logger::Builder::new().build(),
        };

        log::set_boxed_logger(Box::new(logger)).unwrap();
        log::set_max_level(config.level);

        store
    }
}

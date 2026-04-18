use crate::store::LogStore;
use log::{LevelFilter, Log, Metadata, Record};

pub struct InnerLogger {
    pub(crate) store: LogStore,
    pub(crate) level: LevelFilter,

    #[cfg(feature = "env-forward")]
    pub(crate) env: env_logger::Logger,
}

impl Log for InnerLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let line = format!("[{} {}] {}", record.level(), record.target(), record.args());

        self.store.push(line);

        #[cfg(feature = "env-forward")]
        self.env.log(record);
    }

    fn flush(&self) {}
}

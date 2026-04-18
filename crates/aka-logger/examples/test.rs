#[cfg(feature = "export")]
use std::path::PathBuf;

use aka_logger::{AkaLogger, LoggerConfig};
use log::LevelFilter;

fn main() {
    #[cfg(feature = "export")]
    let store = AkaLogger::init(LoggerConfig {
        buf_capacity: 10,
        level: LevelFilter::Error,
        with_stdout: true,
        log_path: PathBuf::from("error").into_boxed_path(),
    });
    #[cfg(not(feature = "export"))]
    let store = AkaLogger::init(LoggerConfig {
        buf_capacity: 10,
        level: LevelFilter::Error,
        with_stdout: true,
    });

    generate_test_logs();

    let mut logs = store.get_inner().read().unwrap().clone();
    let buffer = store.buf_all();
    logs.extend(buffer);
    println!(">>> All log is: <<<");
    println!("{:?}", logs);
    println!("length of logs: {:?}", logs.len());
    println!(">>> Log end <<<");
    let logs_buf = store.buf_all();
    println!(">>> All buf log is: <<<");
    println!("{:?}", logs_buf);
    println!("length of buffer: {:?}", logs_buf.len());
    println!(">>> Buffer Log end <<<");
}

/// There are 46 logs in total.
fn generate_test_logs() {
    log::trace!("initializing tracing subsystem");
    log::debug!("loading configuration from /etc/app/config.toml");
    log::info!("application started");

    log::info!("connecting to database");
    log::debug!("db host=127.0.0.1 port=5432");
    log::warn!("database response slow: 120ms");
    log::error!("database connection retry #1 failed");

    log::info!("retrying connection");
    log::info!("database connected successfully");

    log::trace!("spawning worker threads");
    log::debug!("worker[0] initialized");
    log::debug!("worker[1] initialized");
    log::debug!("worker[2] initialized");

    log::info!("processing request id=1001");
    log::debug!("parsing payload size=512 bytes");
    log::trace!("payload content validated");
    log::info!("request id=1001 completed");

    log::info!("processing request id=1002");
    log::warn!("request id=1002 took longer than expected");
    log::info!("request id=1002 completed");

    log::info!("processing request id=1003");
    log::error!("request id=1003 failed: timeout");

    log::debug!("cleaning temporary files");
    log::trace!("tmp file /tmp/a removed");
    log::trace!("tmp file /tmp/b removed");

    log::info!("starting background sync task");
    log::debug!("sync interval set to 30s");
    log::warn!("sync delayed due to network instability");

    log::info!("user login attempt user=admin");
    log::error!("login failed: invalid password");
    log::info!("user login attempt user=guest");
    log::info!("login success user=guest");

    log::debug!("updating cache entries");
    log::trace!("cache key=abc123 refreshed");
    log::trace!("cache key=xyz789 refreshed");

    log::info!("writing data to disk");
    log::warn!("disk usage at 85%");
    log::error!("disk write failed: no space left");

    log::info!("rotating logs");
    log::debug!("old logs archived");

    log::info!("shutting down worker threads");
    log::debug!("worker[0] stopped");
    log::debug!("worker[1] stopped");
    log::debug!("worker[2] stopped");

    log::info!("application shutting down");
    log::trace!("cleanup complete");
}

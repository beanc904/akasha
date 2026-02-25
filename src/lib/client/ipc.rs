use std::{
    collections::VecDeque,
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::{Arc, OnceLock},
    task::{Context, Poll},
    time::{Duration, Instant},
};

use async_trait::async_trait;
use http::{
    Version,
    header::{CONTENT_LENGTH, CONTENT_TYPE},
};
use httparse::EMPTY_HEADER;
use pin_project::pin_project;
use reqwest::RequestBuilder;
use tokio::net::UnixStream;
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader},
    sync::{Mutex, Semaphore, SemaphorePermit},
    time::timeout,
};

use crate::client::error::{Error, Result};

#[pin_project(project = WrapStreamProj)]
pub enum WrapStream {
    #[cfg(unix)]
    Unix(#[pin] UnixStream),
}

impl WrapStream {
    pub fn is_available(&self) -> Result<bool> {
        match self {
            #[cfg(unix)]
            WrapStream::Unix(s) => {
                use std::io::ErrorKind;

                let mut buf = [0u8; 1];
                match s.try_read(&mut buf) {
                    Ok(n) => Ok(n != 0),
                    Err(e) if e.kind() == ErrorKind::WouldBlock => Ok(true),
                    Err(_) => Err(Error::ConnectionLost),
                }
            }
        }
    }

    pub async fn readable(&self) -> std::io::Result<()> {
        match self {
            #[cfg(unix)]
            WrapStream::Unix(s) => s.readable().await,
        }
    }
    pub async fn writable(&self) -> std::io::Result<()> {
        match self {
            #[cfg(unix)]
            WrapStream::Unix(s) => s.writable().await,
        }
    }
}

impl AsyncRead for WrapStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.project() {
            #[cfg(unix)]
            WrapStreamProj::Unix(s) => s.poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for WrapStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match self.project() {
            #[cfg(unix)]
            WrapStreamProj::Unix(s) => s.poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.project() {
            #[cfg(unix)]
            WrapStreamProj::Unix(s) => s.poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.project() {
            #[cfg(unix)]
            WrapStreamProj::Unix(s) => s.poll_shutdown(cx),
        }
    }
}

pub async fn connect_to_socket(socket_path: &str) -> Result<WrapStream> {
    #[cfg(unix)]
    {
        if !std::path::Path::new(socket_path).exists() {
            log::error!("socket path is not exists: {socket_path}");
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("socket path: {socket_path} not found"),
            )));
        }
        Ok(WrapStream::Unix(UnixStream::connect(socket_path).await?))
    }
}

fn generate_socket_request(req: reqwest::Request) -> Result<String> {
    let method = req.method().as_str();
    let mut path = req.url().path().to_string();
    if let Some(query) = req.url().query() {
        path.push_str(&format!("?{query}"));
    }
    let request_line = format!("{method} {path} HTTP/1.1\r\n");

    // add header information
    let mut headers = String::new();
    let missing_content_length =
        req.headers().contains_key(CONTENT_TYPE) && !req.headers().contains_key(CONTENT_LENGTH);
    let body = req
        .body()
        .and_then(|b| b.as_bytes())
        .map(|b| String::from_utf8_lossy(b).to_string())
        .unwrap_or_default();
    for (name, value) in req.headers() {
        if let Ok(value) = value.to_str() {
            headers.push_str(&format!("{name}: {value}\r\n"));
            if name == CONTENT_TYPE && missing_content_length {
                headers.push_str(&format!("{}: {}\r\n", CONTENT_LENGTH, body.len()));
            }
        }
    }

    // splice completed request, format: request line + header + blank line + body
    let raw = format!("{request_line}{headers}\r\n{body}");

    Ok(raw)
}

fn generate_socket_response(header: String, body: String) -> Result<reqwest::Response> {
    log::debug!("parsing socket response");
    let mut headers = [EMPTY_HEADER; 16];
    let mut res = httparse::Response::new(&mut headers);
    let response_str = format!("{header}{body}");
    // println!("response str: {response_str:?}");
    let raw_response = response_str.as_bytes();
    match res.parse(raw_response) {
        Ok(httparse::Status::Complete(_)) => {
            let mut res_builder = http::Response::builder()
                .version(Version::HTTP_11)
                .status(res.code.unwrap_or(400));
            for header in res.headers.iter() {
                let header_name = header.name;
                let header_value = str::from_utf8(header.value).unwrap_or_default();
                res_builder = res_builder.header(header_name, header_value);
            }
            // {
            //     use std::io::Write;
            //     let mut file = std::fs::File::create("body.json")?;
            //     file.write_all(body.as_bytes())?;
            // }
            let response = res_builder.body(body.to_string())?;
            Ok(reqwest::Response::from(response))
        }
        Ok(httparse::Status::Partial) => {
            log::error!("Partial response, need more data");
            Err(Error::HttpParseError(
                "Partial response, need more data".to_string(),
            ))
        }
        Err(e) => {
            log::error!("Failed to parse response: {e}");
            Err(Error::HttpParseError(format!(
                "Failed to parse response: {e}"
            )))
        }
    }
}

async fn read_header(reader: &mut BufReader<&mut WrapStream>) -> Result<String> {
    let mut header = String::new();
    loop {
        let mut line = String::new();
        if let Ok(size) = reader.read_line(&mut line).await
            && size == 0
        {
            return Err(Error::HttpParseError("no response".to_string()));
        }
        header.push_str(&line);
        if line == "\r\n" {
            break;
        }
    }
    log::debug!("read header done: {header:?}");

    Ok(header)
}

async fn read_chunked_data(reader: &mut BufReader<&mut WrapStream>) -> Result<String> {
    let mut body = Vec::new();
    loop {
        // read chunk size
        let mut size_line = String::new();
        reader.read_line(&mut size_line).await?;
        let size_line = size_line.trim();
        if size_line.is_empty() {
            continue;
        }
        let chunk_size = usize::from_str_radix(size_line, 16)
            .map_err(|e| Error::HttpParseError(format!("Failed to parse chunk size: {e}")))?;

        if chunk_size == 0 {
            // read out the last CRLF
            let mut end = String::new();
            reader.read_line(&mut end).await?;
            break;
        }

        // read chunk data
        let mut chunk_data = vec![0u8; chunk_size];
        reader.read_exact(&mut chunk_data).await?;
        body.extend_from_slice(&chunk_data);

        // read off the end CRLF
        let mut crlf = String::new();
        reader.read_line(&mut crlf).await?;
    }
    log::debug!("read chunked data done");
    Ok(String::from_utf8(body)?)
}

// ----------------------------------------------------------------
//                       Connection Pool
// ----------------------------------------------------------------

// connection pool configuration
#[derive(Debug, Clone)]
pub struct IpcPoolConfig {
    /// Minimum connection size, default: `3`
    pub min_connections: usize,
    /// Maximum connection size, default: `10`
    pub max_connections: usize,
    /// Idle timeout, default: `60s`
    pub idle_timeout: Duration,
    /// Health check interval, default: `60s`
    pub health_check_interval: Duration,
    /// Reject policy,
    /// default: `New`
    /// (Create new ipc connection directly
    /// without waiting for the available connection pool)
    pub reject_policy: RejectPolicy,
}

impl Default for IpcPoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 3,
            max_connections: 20,
            idle_timeout: Duration::from_secs(60),
            health_check_interval: Duration::from_secs(60),
            reject_policy: RejectPolicy::New,
        }
    }
}

pub struct IpcPoolConfigBuilder {
    min_connections: usize,
    max_connections: usize,
    idle_timeout: Duration,
    health_check_interval: Duration,
    reject_policy: RejectPolicy,
}

impl Default for IpcPoolConfigBuilder {
    fn default() -> Self {
        Self {
            min_connections: 3,
            max_connections: 20,
            idle_timeout: Duration::from_secs(60),
            health_check_interval: Duration::from_secs(60),
            reject_policy: RejectPolicy::New,
        }
    }
}

impl IpcPoolConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn min_connections(mut self, min_connections: usize) -> Self {
        self.min_connections = min_connections;
        self
    }

    pub fn max_connections(mut self, max_connections: usize) -> Self {
        self.max_connections = max_connections;
        self
    }

    pub fn idle_timeout(mut self, idle_timeout: Duration) -> Self {
        self.idle_timeout = idle_timeout;
        self
    }

    pub fn health_check_interval(mut self, health_check_interval: Duration) -> Self {
        self.health_check_interval = health_check_interval;
        self
    }

    pub fn reject_policy(mut self, reject_policy: RejectPolicy) -> Self {
        self.reject_policy = reject_policy;
        self
    }

    pub fn build(self) -> IpcPoolConfig {
        IpcPoolConfig {
            min_connections: self.min_connections,
            max_connections: self.max_connections,
            idle_timeout: self.idle_timeout,
            health_check_interval: self.health_check_interval,
            reject_policy: self.reject_policy,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub enum RejectPolicy {
    /// Create new ipc connection directly without waiting for available connection pool
    #[default]
    New,
    /// Reject directly when connection pool is unavailable
    Reject,
    /// Timeout of waiting for connection pool becoming available
    Timeout(Duration),
    /// Keep waiting for connection pool becoming available
    Wait,
}

// IPC connector
struct IpcConnection {
    stream: WrapStream,
    last_used: Instant,
}

impl Deref for IpcConnection {
    type Target = WrapStream;

    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}

impl DerefMut for IpcConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream
    }
}

impl IpcConnection {
    fn new(stream: WrapStream) -> Self {
        Self {
            stream,
            last_used: Instant::now(),
        }
    }

    fn is_valid(&mut self) -> bool {
        self.stream.is_available().unwrap_or_default()
    }
}

// IPC connection pool
#[derive(Clone)]
pub struct IpcConnectionPool {
    connections: Arc<Mutex<VecDeque<IpcConnection>>>,
    semaphore: Arc<Semaphore>,
    config: IpcPoolConfig,
}

static CONNECTION_POOL: OnceLock<IpcConnectionPool> = OnceLock::new();

impl IpcConnectionPool {
    fn new(config: IpcPoolConfig) -> Self {
        let pool = IpcConnectionPool {
            semaphore: Arc::new(Semaphore::new(config.max_connections)),
            config,
            connections: Arc::new(Mutex::new(VecDeque::new())),
        };
        // start thread of clear idle connection task
        pool.start_clear_idle_conns_task();
        pool
    }

    /// Initialize global instance
    pub fn init(config: IpcPoolConfig) -> Result<()> {
        CONNECTION_POOL
            .set(IpcConnectionPool::new(config))
            .map_err(|_| Error::ConnectionPoolInitFailed)
    }

    pub fn global() -> Result<&'static Self> {
        CONNECTION_POOL
            .get()
            .ok_or(Error::ConnectionPoolNotInitialized)
    }

    /// Start thread of clear idle connection task
    fn start_clear_idle_conns_task(&self) {
        let pool = self.clone();

        // tauri::async_runtime::spawn()
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(pool.config.health_check_interval);
            loop {
                interval.tick().await;
                pool.cleanup_idle_connections().await;
            }
        });
    }

    /// Clear idle connections
    async fn cleanup_idle_connections(&self) {
        log::debug!("cleanup idle connections");
        let now = Instant::now();

        // remove timeout idle connections but keep minimum connections
        let min_to_keep = self.config.min_connections;

        let mut connections = self.connections.lock().await;
        let before = connections.len();
        if before < min_to_keep {
            log::debug!("connections less than min_to_keep, skip cleanup");
            return;
        }

        let mut kept_connections = 0;
        connections.retain(|conn| {
            if kept_connections < min_to_keep
                || now.duration_since(conn.last_used) <= self.config.idle_timeout
            {
                kept_connections += 1;
                true
            } else {
                false
            }
        });
        log::debug!(
            "cleanup connections done, before: {}, after: {}",
            before,
            connections.len()
        );
    }

    async fn get_connection<'a>(
        &'a self,
        socket_path: &str,
    ) -> Result<(IpcConnection, SemaphorePermit<'a>)> {
        log::debug!("get connection from pool");
        // ensure to gain semaphore permit
        let permit = self.acquire_permit().await?;
        // start to create connection
        let conn = self.acquire_or_create_connection(socket_path).await?;
        Ok((conn, permit))
    }

    async fn acquire_permit<'a>(&'a self) -> Result<SemaphorePermit<'a>> {
        log::debug!("acquire permit");
        match self.semaphore.try_acquire() {
            Ok(permit) => Ok(permit),
            Err(_) => match self.config.reject_policy {
                RejectPolicy::New => {
                    log::debug!("max permit has acquire, add permit");
                    self.semaphore.add_permits(1);
                    match self.semaphore.acquire().await {
                        Ok(permit) => Ok(permit),
                        Err(_) => {
                            log::error!("failed to acquire permit, forget permit");
                            self.semaphore.forget_permits(1);
                            Err(Error::ConnectionFailed)
                        }
                    }
                }
                RejectPolicy::Reject => Err(Error::ConnectionPoolFull),
                RejectPolicy::Timeout(timeout_duration) => {
                    let acquire_future = self.semaphore.acquire();
                    match timeout(timeout_duration, acquire_future).await {
                        Ok(Ok(permit)) => Ok(permit),
                        Ok(Err(_)) => Err(Error::ConnectionPoolFull),
                        Err(e) => Err(Error::Timeout(e)),
                    }
                }
                RejectPolicy::Wait => {
                    let acquire_future = self.semaphore.acquire().await;
                    match acquire_future {
                        Ok(permit) => Ok(permit),
                        Err(_) => Err(Error::ConnectionPoolFull),
                    }
                }
            },
        }
    }

    async fn acquire_or_create_connection(&self, socket_path: &str) -> Result<IpcConnection> {
        // get connection from pool and check it timeliness
        if let Some(mut conn) = self.connections.lock().await.pop_front() {
            log::debug!("get connection from pool successfully");
            if !conn.is_valid() {
                // if the connection fails, re-establish the connection.
                log::warn!(
                    "connection from pool is not available, drop it and create new connection"
                );
                drop(conn);
                return Self::create_connection(socket_path).await;
            }
            return Ok(conn);
        }

        let conn = Self::create_connection(socket_path).await?;
        Ok(conn)
    }

    async fn create_connection(socket_path: &str) -> Result<IpcConnection> {
        log::trace!(
            "creating connection, available permits: {}",
            Self::global()?.semaphore.available_permits()
        );
        match connect_to_socket(socket_path).await {
            Ok(stream) => Ok(IpcConnection::new(stream)),
            Err(_) => Err(Error::ConnectionFailed),
        }
    }

    async fn release_connection(&self, mut connection: IpcConnection) {
        let mut connections = self.connections.lock().await;
        log::debug!(
            "release connections, pool length: {}, available permits: {}",
            connections.len(),
            self.semaphore.available_permits()
        );
        connection.last_used = Instant::now();

        if self.semaphore.available_permits() >= self.config.max_connections {
            self.semaphore.forget_permits(1);
            drop(connection);
        } else {
            log::debug!("push connection to pool");
            connections.push_back(connection);
        }
    }

    pub async fn clear_pool(&self) {
        let mut connections = self.connections.lock().await;
        for conn in connections.drain(..) {
            drop(conn);
        }
        connections.clear();
    }
}

impl Drop for IpcConnectionPool {
    fn drop(&mut self) {
        // // tauri::async_runtime::block_on();
        // // I think the akasha main is start with #[tokio::main],
        // // so the whole programme is running in the tokio runtime.
        // tokio::runtime::Handle::current().block_on(async move {
        //     // tokio::runtime::Runtime::new()
        //     // .unwrap()
        //     // .block_on(async move {
        //     log::debug!("IpcConnectionPool is being dropped");
        //     self.clear_pool().await;
        // });
        // match tokio::runtime::Handle::try_current() {
        //     Ok(handle) => {
        //         handle.spawn(async move {
        //         });
        //     }
        //     Err(_) => tokio::runtime::Runtime::new().unwrap().block_on(async {
        //         log::debug!("IpcConnectionPool is being dropped");
        //         self.clear_pool().await;
        //     }),
        // }

        // try_lock() is synchronous, do not need runtime
        if let Ok(mut connections) = self.connections.try_lock() {
            log::debug!("IpcConnectionPool is being dropped via try_lock");
            connections.drain(..);
            connections.clear();
        } else {
            // If fail to get the lock, means there are some other
            // synchronous tasks are operating pool at the same time.
            // In this case, you could consider spawning a background
            // task to pick up and clean up any remaining issues.
            let connections_arc = self.connections.clone();
            tokio::spawn(async move {
                let mut conns = connections_arc.lock().await;
                conns.clear();
            });
        }
    }
}

#[async_trait]
pub trait LocalSocket {
    async fn send_by_local_socket(self, socket_path: &str) -> Result<reqwest::Response>;
}

#[async_trait]
impl LocalSocket for RequestBuilder {
    async fn send_by_local_socket(self, socket_path: &str) -> Result<reqwest::Response> {
        let request = self.build()?;
        let timeout = request.timeout().cloned();

        let process = async move {
            let pool = IpcConnectionPool::global()?;
            let (mut conn, _permit) = pool.get_connection(socket_path).await?;
            // let mut stream = connect_to_socket(socket_path).await?;
            log::debug!("building socket request");
            let req_str = generate_socket_request(request)?;
            log::debug!("request string: {req_str:?}");
            conn.writable().await?;
            log::debug!("send request");
            conn.write_all(req_str.as_bytes()).await?;
            log::debug!("wait for response");
            conn.readable().await?;

            let mut reader = BufReader::new(&mut conn.stream);

            // read and parse header
            let header = read_header(&mut reader).await?;
            // parse Content-Length, judge if it is chunked response
            let mut content_length: Option<usize> = None;
            let mut is_chunked = false;
            for line in header.lines() {
                if let Some(v) = line.to_lowercase().strip_prefix("content-length: ") {
                    content_length = Some(v.trim().parse()?);
                }
                if line.to_lowercase().contains("transfer-encoding: chunked") {
                    is_chunked = true;
                }
            }

            // read body
            let body = if is_chunked {
                read_chunked_data(&mut reader).await?
            } else if let Some(content_length) = content_length {
                log::debug!("content length: {content_length}");
                let mut body_buf = vec![0u8; content_length];
                reader.read_exact(&mut body_buf).await?;
                String::from_utf8_lossy(&body_buf).to_string()
            } else {
                // use blank body
                String::new()
            };
            log::debug!("receive response success");
            // reader.shutdown().await?;
            pool.release_connection(conn).await;
            generate_socket_response(header, body)
        };

        match timeout {
            Some(duration) => {
                log::debug!("Timeout duration: {:?}", duration);
                tokio::time::timeout(duration, process).await?
            }
            None => {
                log::debug!("No timeout specified");
                process.await
            }
        }
    }
}

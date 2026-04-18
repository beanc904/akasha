use std::collections::VecDeque;
#[cfg(feature = "export")]
use std::fs::File;
#[cfg(feature = "export")]
use std::io::Write;
#[cfg(feature = "export")]
use std::path::Path;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct LogStore {
    buffer: Arc<RwLock<VecDeque<String>>>,
    inner: Arc<RwLock<Vec<String>>>,
    capacity: usize,
    #[cfg(feature = "export")]
    log_path: Box<Path>,
}

impl LogStore {
    #[cfg(not(feature = "export"))]
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(VecDeque::with_capacity(capacity))),
            inner: Arc::new(RwLock::new(Vec::new())),
            capacity,
        }
    }

    #[cfg(feature = "export")]
    pub fn new<P>(capacity: usize, log_path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            buffer: Arc::new(RwLock::new(VecDeque::with_capacity(capacity))),
            inner: Arc::new(RwLock::new(Vec::new())),
            capacity,
            log_path: log_path.as_ref().into(),
        }
    }

    pub fn push(&self, line: String) {
        let mut buf = self.buffer.write().unwrap();
        let mut inner = self.inner.write().unwrap();

        if buf.len() >= self.capacity {
            let buffer = buf.drain(..).collect::<Vec<String>>();
            inner.extend_from_slice(&buffer);

            #[cfg(feature = "export")]
            {
                let mut file = File::options()
                    .append(true)
                    .create(true)
                    .open(&self.log_path)
                    .unwrap();
                for line in buffer {
                    writeln!(file, "{}", line).unwrap();
                }
            }
        }

        buf.push_back(line);
    }

    pub fn buf_tail(&self, n: usize) -> Vec<String> {
        let buf = self.buffer.read().unwrap();
        buf.iter().rev().take(n).cloned().collect()
    }

    pub fn buf_all(&self) -> Vec<String> {
        self.buffer.read().unwrap().iter().cloned().collect()
    }

    pub fn buf_clear(&self) {
        self.buffer.write().unwrap().clear();
    }

    pub fn tail(&self, n: usize) -> Vec<String> {
        let inner = self.inner.read().unwrap();
        let start_idx = inner.len() - n;
        let end_idx = inner.len() + 1;
        inner.get(start_idx..end_idx).unwrap().into()
    }

    pub fn all(&self) -> Vec<String> {
        let buf = self.buffer.read().unwrap().clone();
        let mut inner = self.inner.read().unwrap().clone();
        inner.extend(buf);
        inner
    }

    pub fn all_len(&self) -> usize {
        let buf_len = self.buffer.read().unwrap().len();
        let inner_len = self.inner.read().unwrap().len();
        buf_len + inner_len
    }
}

#[cfg(feature = "export")]
impl Drop for LogStore {
    fn drop(&mut self) {
        let buf = self.buffer.write().unwrap();
        if buf.len() < self.capacity {
            let mut file = File::options()
                .append(true)
                .create(true)
                .open(&self.log_path)
                .unwrap();
            for line in buf.iter().cloned().collect::<Vec<String>>() {
                writeln!(file, "{}", line).unwrap();
            }
        }
    }
}

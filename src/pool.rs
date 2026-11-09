use std::sync::{Arc, Mutex};

pub struct BufferPool {
    pool: Arc<Mutex<Vec<Vec<u8>>>>,
    buffer_size: usize,
}

impl BufferPool {
    pub fn new(capacity: usize, buffer_size: usize) -> Self {
        let mut pool_vec = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            pool_vec.push(Vec::with_capacity(buffer_size));
        }
        Self {
            pool: Arc::new(Mutex::new(pool_vec)),
            buffer_size,
        }
    }

    pub fn acquire(&self) -> PooledBuffer {
        let mut pool = self.pool.lock().unwrap();
        let mut buf = pool.pop().unwrap_or_else(|| Vec::with_capacity(self.buffer_size));
        buf.clear();
        PooledBuffer {
            buf: Some(buf),
            pool: self.pool.clone(), // Sada radi jer je Arc!
        }
    }
}

pub struct PooledBuffer {
    pub buf: Option<Vec<u8>>,
    pool: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl Drop for PooledBuffer {
    fn drop(&mut self) {
        if let Some(buf) = self.buf.take() {
            if let Ok(mut pool) = self.pool.lock() {
                if pool.len() < 128 {
                    pool.push(buf);
                }
            }
        }
    }
}
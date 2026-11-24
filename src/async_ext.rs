use crate::errors::NetworkError;
use crate::manager::ConnectionCommand;
use crate::pool::{BufferPool, PooledBuffer};
use tokio::sync::mpsc;

pub type BoundedCommandSender = mpsc::Sender<ConnectionCommand>;
pub type BoundedCommandReceiver = mpsc::Receiver<ConnectionCommand>;

/// Creates a bounded command channel to enforce backpressure under heavy load.
pub fn bounded_command_channel(capacity: usize) -> (BoundedCommandSender, BoundedCommandReceiver) {
    let cap = capacity.max(1);
    mpsc::channel(cap)
}

/// Sends a command with backpressure; returns ConnectionClosed if receiver is dropped.
pub async fn send_command_backpressured(
    tx: &BoundedCommandSender,
    cmd: ConnectionCommand,
) -> Result<(), NetworkError> {
    tx.send(cmd)
        .await
        .map_err(|_| NetworkError::ConnectionClosed)
}

/// Thread-safe helper around BufferPool for acquiring buffers without holding locks across `.await`.
pub struct SharedBufferPool {
    pool: BufferPool,
}

impl SharedBufferPool {
    pub fn new(capacity: usize, buffer_size: usize) -> Self {
        Self {
            pool: BufferPool::new(capacity, buffer_size),
        }
    }

    pub fn acquire(&self) -> PooledBuffer {
        self.pool.acquire()
    }
}
use tokio::sync::mpsc;
use std::collections::HashMap;
use std::net::SocketAddr;

#[derive(Debug)]
pub enum ConnectionCommand {
    Register { addr: SocketAddr, tx: mpsc::Sender<Vec<u8>> },
    Unregister { addr: SocketAddr },
    SendTo { addr: SocketAddr, data: Vec<u8> },
    Broadcast { data: Vec<u8> },
}

pub struct ConnectionManager {
    receiver: mpsc::Receiver<ConnectionCommand>,
    connections: HashMap<SocketAddr, mpsc::Sender<Vec<u8>>>,
}

impl ConnectionManager {
    pub fn new(receiver: mpsc::Receiver<ConnectionCommand>) -> Self {
        Self {
            receiver,
            connections: HashMap::new(),
        }
    }

    /// Pokreće actor petlju koja asinhrono upravlja svim konekcijama
    pub async fn run(&mut self) {
        while let Some(cmd) = self.receiver.recv().await {
            match cmd {
                ConnectionCommand::Register { addr, tx } => {
                    self.connections.insert(addr, tx);
                }
                ConnectionCommand::Unregister { addr } => {
                    self.connections.remove(&addr);
                }
                ConnectionCommand::SendTo { addr, data } => {
                    if let Some(tx) = self.connections.get(&addr) {
                        let _ = tx.send(data).await;
                    }
                }
                ConnectionCommand::Broadcast { data } => {
                    for tx in self.connections.values() {
                        let _ = tx.send(data.clone()).await;
                    }
                }
            }
        }
    }
}
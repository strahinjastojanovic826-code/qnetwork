use crate::errors::NetworkError;
use crate::packet::QuatPacket;
use crate::socket::QuatUdpSocket;
use std::net::SocketAddr;

pub struct ReliableUdpSocket {
    socket: QuatUdpSocket,
    next_seq: u32,
}

impl ReliableUdpSocket {
    pub fn bind(addr: &str) -> Result<Self, NetworkError> {
        let socket = QuatUdpSocket::bind(addr)?;
        Ok(Self { socket, next_seq: 0 })
    }

    pub fn send_reliable(&mut self, packet: &QuatPacket, target_addr: &str) -> Result<(), NetworkError> {
        // Ovde bi dodao sekvencni broj u payload ili zaglavlje pre slanja
        self.socket.send_to(packet, target_addr)?;
        self.next_seq += 1;
        Ok(())
    }

    pub fn recv_reliable(&mut self) -> Result<(QuatPacket, SocketAddr), NetworkError> {
        self.socket.recv_from()
    }
}
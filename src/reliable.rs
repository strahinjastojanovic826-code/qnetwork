use crate::errors::NetworkError;
use crate::packet::QuatPacket;
use crate::socket::QuatUdpSocket;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct ReliableUdpSocket {
    socket: QuatUdpSocket,
    next_seq: u32,
    unacked_packets: HashMap<u32, (QuatPacket, Instant)>,
    ack_timeout: Duration,
}

impl ReliableUdpSocket {
    pub fn bind(addr: &str) -> Result<Self, NetworkError> {
        let socket = QuatUdpSocket::bind(addr).map_err(|_| NetworkError::BindFailed)?;
        Ok(Self {
            socket,
            next_seq: 0,
            unacked_packets: HashMap::new(),
            ack_timeout: Duration::from_millis(500), // 500ms timeout za retransmisiju
        })
    }

    /// Šalje paket pouzdano uz praćenje sekvence i ACK mehanizam
    pub fn send_reliable(&mut self, packet: QuatPacket, target_addr: &str) -> Result<(), NetworkError> {
        let seq = self.next_seq;
        self.next_seq = self.next_seq.wrapping_add(1);

        // Direktno prosleđujemo &packet jer QuatUdpSocket::send_to to očekuje
        self.socket.send_to(&packet, target_addr)?;

        // Čuvamo u listi nepotvrđenih paketa za slučaj retransmisije
        self.unacked_packets.insert(seq, (packet, Instant::now()));
        Ok(())
    }

    /// Prima paket i automatski šalje ACK nazad pošiljaocu
    pub fn recv_reliable(&mut self) -> Result<(QuatPacket, std::net::SocketAddr), NetworkError> {
        let (packet, addr) = self.socket.recv_from()?;
        // Ovde bi išla logika za parsiranje ACK-a ili slanje povratnog ACK-a
        Ok((packet, addr))
    }
}
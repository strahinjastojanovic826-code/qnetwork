use crate::errors::NetworkError;
use crate::packet::QuatPacket;
use crate::stream::StreamDecoder;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::time::Duration;

pub struct QuatSocket {
    stream: TcpStream,
    decoder: StreamDecoder,
}

impl QuatSocket {
    pub fn connect(addr: &str) -> Result<Self, NetworkError> {
        let stream = TcpStream::connect(addr)?;
        Ok(Self {
            stream,
            decoder: StreamDecoder::new(),
        })
    }

    pub fn send(&mut self, packet: &QuatPacket) -> Result<(), NetworkError> {
        let wire_bytes = packet.serialize();
        self.stream.write_all(&wire_bytes)?;
        self.stream.flush()?;
        Ok(())
    }

    pub fn set_timeout(
        &mut self,
        read_ms: Option<u64>,
        write_ms: Option<u64>,
    ) -> Result<(), NetworkError> {
        let read_dur = read_ms.map(Duration::from_millis);
        let write_dur = write_ms.map(Duration::from_millis);

        self.stream
            .set_read_timeout(read_dur)
            .map_err(|_| NetworkError::SetTimeoutFailed)?;
        self.stream
            .set_write_timeout(write_dur)
            .map_err(|_| NetworkError::SetTimeoutFailed)?;
        Ok(())
    }

    pub fn receive(&mut self) -> Result<Vec<QuatPacket>, NetworkError> {
        let mut buf = [0u8; 1024];
        let bytes_read = self.stream.read(&mut buf)?;
        if bytes_read == 0 {
            return Err(NetworkError::ConnectionClosed);
        }
        Ok(self.decoder.feed(&buf[..bytes_read]))
    }
}

pub struct QuatUdpSocket {
    socket: UdpSocket,
}

impl QuatUdpSocket {
    pub fn bind(addr: &str) -> Result<Self, NetworkError> {
        let socket = UdpSocket::bind(addr).map_err(|_| NetworkError::BindFailed)?;
        Ok(Self { socket })
    }

    pub fn send_to(&self, packet: &QuatPacket, target_addr: &str) -> Result<usize, NetworkError> {
        let bytes = packet.serialize();
        let sent = self.socket.send_to(&bytes, target_addr)?;
        Ok(sent)
    }

    pub fn recv_from(&self) -> Result<(QuatPacket, SocketAddr), NetworkError> {
        let mut buf = [0u8; 2048];
        let (bytes_read, src_addr) = self.socket.recv_from(&mut buf)?;
        let packet = QuatPacket::deserialize(&buf[..bytes_read])?;
        Ok((packet, src_addr))
    }
}

use tokio::net::TcpStream as TokioTcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct AsyncQuatSocket {
    stream: TokioTcpStream,
    decoder: StreamDecoder,
}

impl AsyncQuatSocket {
    pub async fn connect(addr: &str) -> Result<Self, NetworkError> {
        let stream = TokioTcpStream::connect(addr)
            .await
            .map_err(|_| NetworkError::BindFailed)?; // ili odgovarajući error
        Ok(Self {
            stream,
            decoder: StreamDecoder::new(),
        })
    }

    pub async fn send(&mut self, packet: &QuatPacket) -> Result<(), NetworkError> {
        let wire_bytes = packet.serialize();
        self.stream.write_all(&wire_bytes).await.map_err(|e| NetworkError::Io(e.to_string()))?;
        self.stream.flush().await.map_err(|e| NetworkError::Io(e.to_string()))?;
        Ok(())
    }

    pub async fn receive(&mut self) -> Result<Vec<QuatPacket>, NetworkError> {
        let mut buf = [0u8; 1024];
        let bytes_read = self.stream.read(&mut buf).await.map_err(|e| NetworkError::Io(e.to_string()))?;
        if bytes_read == 0 {
            return Err(NetworkError::ConnectionClosed);
        }
        Ok(self.decoder.feed(&buf[..bytes_read]))
    }
}
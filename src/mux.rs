use crate::errors::NetworkError;
use crate::packet::QuatPacket;
use crate::quat::{Quat, QuatBuffer};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuatChannel {
    Control = 0,
    Data = 1,
    Priority = 2,
    Custom = 3,
}

impl QuatChannel {
    pub fn to_quat(self) -> Quat {
        match self {
            QuatChannel::Control => Quat::Q0,
            QuatChannel::Data => Quat::Q1,
            QuatChannel::Priority => Quat::Q2,
            QuatChannel::Custom => Quat::Q3,
        }
    }

    pub fn from_quat(quat: Quat) -> Self {
        match quat {
            Quat::Q0 => QuatChannel::Control,
            Quat::Q1 => QuatChannel::Data,
            Quat::Q2 => QuatChannel::Priority,
            Quat::Q3 => QuatChannel::Custom,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketMode {
    Raw,
    Fec,
    RleCompressed,
}

pub struct QuatMuxFrame {
    pub channel: QuatChannel,
    pub mode: PacketMode,
    pub packet: QuatPacket,
}

impl QuatMuxFrame {
    pub fn create(
        channel: QuatChannel,
        mode: PacketMode,
        raw_buffer: &QuatBuffer,
    ) -> Self {
        let mut final_buffer = QuatBuffer::new();
        final_buffer.push(channel.to_quat());

        let mode_quat = match mode {
            PacketMode::Raw => Quat::Q0,
            PacketMode::Fec => Quat::Q1,
            PacketMode::RleCompressed => Quat::Q2,
        };
        final_buffer.push(mode_quat);

        match mode {
            PacketMode::Raw => {
                for &q in raw_buffer.iter() {
                    final_buffer.push(q);
                }
            }
            PacketMode::RleCompressed => {
                let compressed_bytes = raw_buffer.compress_rle();
                for byte in compressed_bytes {
                    if let Ok(q) = Quat::try_from((byte >> 6) & 0b11) { final_buffer.push(q); }
                    if let Ok(q) = Quat::try_from((byte >> 4) & 0b11) { final_buffer.push(q); }
                    if let Ok(q) = Quat::try_from((byte >> 2) & 0b11) { final_buffer.push(q); }
                    if let Ok(q) = Quat::try_from(byte & 0b11) { final_buffer.push(q); }
                }
            }
            PacketMode::Fec => {
                for &q in raw_buffer.iter() {
                    final_buffer.push(q);
                }
            }
        }

        QuatMuxFrame {
            channel,
            mode,
            packet: QuatPacket::new(final_buffer),
        }
    }

    pub fn unpack(&self) -> Result<(QuatChannel, PacketMode, QuatBuffer), NetworkError> {
        let quats = &self.packet.payload;
        if quats.len() < 2 {
            return Err(NetworkError::PackageTooShort);
        }

        let channel = QuatChannel::from_quat(quats[0]);
        let mode = match quats[1] {
            Quat::Q0 => PacketMode::Raw,
            Quat::Q1 => PacketMode::Fec,
            Quat::Q2 => PacketMode::RleCompressed,
            _ => PacketMode::Raw,
        };

        let mut payload_buf = QuatBuffer::new();
        for &q in quats[2..].iter() {
            payload_buf.push(q);
        }

        Ok((channel, mode, payload_buf))
    }
}
use crate::packet::QuatPacket;
use std::collections::VecDeque;
use std::convert::TryInto;

pub struct StreamDecoder {
    buffer: VecDeque<u8>,
}

impl StreamDecoder {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
        }
    }

    pub fn feed(&mut self, chunk: &[u8]) -> Vec<QuatPacket> {
        self.buffer.extend(chunk);
        let mut packets = Vec::new();

        loop {
            if self.buffer.len() < 10 {
                break;
            }

            if self.buffer[0] != b'Q' || self.buffer[1] != b'4' {
                self.buffer.pop_front();
                continue;
            }

            let slice = self.buffer.make_contiguous();
            let quat_count = u32::from_be_bytes(slice[2..6].try_into().unwrap()) as usize;

            let payload_bytes_len = (quat_count + 3) / 4;
            let total_frame_len = 6 + payload_bytes_len + 4;

            if self.buffer.len() < total_frame_len {
                break;
            }

            let frame_bytes: Vec<u8> = self.buffer.drain(0..total_frame_len).collect();
            if let Ok(packet) = QuatPacket::deserialize(&frame_bytes) {
                packets.push(packet);
            }
        }

        packets
    }
}
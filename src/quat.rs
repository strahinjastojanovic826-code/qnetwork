use crate::errors::NetworkError;
use std::convert::TryFrom;
use std::ops::{Deref, DerefMut};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quat {
    Q0 = 0b00,
    Q1 = 0b01,
    Q2 = 0b10,
    Q3 = 0b11,
}

impl TryFrom<u8> for Quat {
    type Error = NetworkError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(Quat::Q0),
            1 => Ok(Quat::Q1),
            2 => Ok(Quat::Q2),
            3 => Ok(Quat::Q3),
            _ => Err(NetworkError::InsufficientQuats),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct QuatBuffer {
    pub quats: Vec<Quat>,
}

impl QuatBuffer {
    pub fn new() -> Self {
        Self { quats: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            quats: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, quat: Quat) {
        self.quats.push(quat);
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut packed = Vec::with_capacity((self.quats.len() + 3) / 4);
        for chunk in self.quats.chunks(4) {
            let mut byte = 0u8;
            for (i, &q) in chunk.iter().enumerate() {
                let shift = 6 - (i * 2);
                byte |= (q as u8) << shift;
            }
            packed.push(byte);
        }
        packed
    }

    pub fn unpack(packed: &[u8], total_quats: usize) -> Result<Self, NetworkError> {
        let expected_bytes = total_quats
            .checked_add(3)
            .ok_or(NetworkError::OverflowPayloadCalculation)?
            / 4;

        if packed.len() < expected_bytes {
            return Err(NetworkError::PackageSmallerThanHeader);
        }

        let mut quats = Vec::with_capacity(total_quats);
        let mut read = 0;

        for &byte in packed {
            for i in 0..4 {
                if read >= total_quats {
                    break;
                }
                let shift = 6 - (i * 2);
                let val = (byte >> shift) & 0b11;
                quats.push(Quat::try_from(val)?);
                read += 1;
            }
        }

        Ok(Self { quats })
    }

    pub fn pack_with_fec(&self) -> Vec<u8> {
        let mut encoded = Vec::with_capacity(self.quats.len());
        for chunk in self.quats.chunks(2) {
            let mut nibble = 0u8;
            if let Some(&q0) = chunk.get(0) {
                nibble |= (q0 as u8) << 2;
            }
            if let Some(&q1) = chunk.get(1) {
                nibble |= q1 as u8;
            }

            let d0 = (nibble >> 3) & 1;
            let d1 = (nibble >> 2) & 1;
            let d2 = (nibble >> 1) & 1;
            let d3 = nibble & 1;

            let p1 = d0 ^ d1 ^ d3;
            let p2 = d0 ^ d2 ^ d3;
            let p3 = d1 ^ d2 ^ d3;

            let code = (p1 << 6) | (p2 << 5) | (d0 << 4) | (p3 << 3) | (d1 << 2) | (d2 << 1) | d3;
            encoded.push(code);
        }
        encoded
    }

    pub fn decode_fec_byte(code: u8) -> u8 {
        let p1 = (code >> 6) & 1;
        let p2 = (code >> 5) & 1;
        let d0 = (code >> 4) & 1;
        let p3 = (code >> 3) & 1;
        let d1 = (code >> 2) & 1;
        let d2 = (code >> 1) & 1;
        let d3 = code & 1;

        let s1 = p1 ^ d0 ^ d1 ^ d3;
        let s2 = p2 ^ d0 ^ d2 ^ d3;
        let s3 = p3 ^ d1 ^ d2 ^ d3;

        let error_pos = (s3 << 2) | (s2 << 1) | s1;
        let mut corrected = code;
        if error_pos > 0 && error_pos <= 7 {
            corrected ^= 1 << (7 - error_pos);
        }

        let cd0 = (corrected >> 4) & 1;
        let cd1 = (corrected >> 2) & 1;
        let cd2 = (corrected >> 1) & 1;
        let cd3 = corrected & 1;

        (cd0 << 3) | (cd1 << 2) | (cd2 << 1) | cd3
    }

    pub fn compress_rle(&self) -> Vec<u8> {
        let mut compressed = Vec::new();
        if self.quats.is_empty() {
            return compressed;
        }

        let mut current_quat = self.quats[0];
        let mut count: u8 = 0;

        for &q in &self.quats {
            if q == current_quat && count < 255 {
                count += 1;
            } else {
                compressed.push(current_quat as u8);
                compressed.push(count);
                current_quat = q;
                count = 1;
            }
        }
        compressed.push(current_quat as u8);
        compressed.push(count);

        compressed
    }

    pub fn decompress_rle(data: &[u8]) -> Result<Self, NetworkError> {
        if data.len() % 2 != 0 {
            return Err(NetworkError::InvalidRleStream);
        }

        let total_quats: usize = data.chunks_exact(2).map(|c| c[1] as usize).sum();
        let mut buf = QuatBuffer::with_capacity(total_quats);

        for chunk in data.chunks_exact(2) {
            let quat = Quat::try_from(chunk[0])?;
            let count = chunk[1];
            for _ in 0..count {
                buf.push(quat);
            }
        }

        Ok(buf)
    }
}

impl Deref for QuatBuffer {
    type Target = [Quat];
    fn deref(&self) -> &Self::Target {
        &self.quats
    }
}

impl DerefMut for QuatBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.quats
    }
}
use crate::errors::NetworkError;
use crate::quat::QuatBuffer;
use std::convert::TryInto;

pub struct QuatPacket {
    pub payload: QuatBuffer,
}

impl QuatPacket {
    pub fn new(payload: QuatBuffer) -> Self {
        Self { payload }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let packed_payload = self.payload.pack();
        let quat_count = self.payload.len() as u32;
        let mut frame = Vec::with_capacity(2 + 4 + packed_payload.len() + 4);

        frame.extend_from_slice(b"Q4");
        frame.extend_from_slice(&quat_count.to_be_bytes());
        frame.extend_from_slice(&packed_payload);

        let checksum = Self::calculate_checksum(&frame);
        frame.extend_from_slice(&checksum.to_be_bytes());

        frame
    }

    pub fn generate_parity_packet(packets: &[QuatPacket]) -> QuatPacket {
        let max_len = packets.iter().map(|p| p.payload.len()).max().unwrap_or(0);
        let mut parity_data = QuatBuffer::new();
        
        for i in 0..max_len {
            let mut xor_acc: u8 = 0;
            for p in packets {
                if let Some(q) = p.payload.get(i) {
                    // Konverzija quat-a u u8 radi XOR operacije
                    xor_acc ^= *q as u8;
                }
            }
            if let Ok(q) = crate::quat::Quat::try_from(xor_acc & 0b11) {
                parity_data.push(q);
            }
        }
        QuatPacket::new(parity_data)
    }

    pub fn calculate_crc32(data: &[u8]) -> u32 {
        let mut crc: u32 = 0xFFFFFFFF;
        for &byte in data {
            crc ^= u32::from(byte);
            for _ in 0..8 {
                if (crc & 1) != 0 {
                    crc = (crc >> 1) ^ 0xEDB88320;
                } else {
                    crc >>= 1;
                }
            }
        }
        !crc
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, NetworkError> {
        if bytes.len() < 10 {
            return Err(NetworkError::PackageTooShort);
        }

        if &bytes[0..2] != b"Q4" {
            return Err(NetworkError::IncorrectMagicHeader);
        }

        let quat_count = u32::from_be_bytes(
            bytes[2..6]
                .try_into()
                .map_err(|_| NetworkError::PackageTooShort)?,
        ) as usize;

        let payload_bytes_len = quat_count
            .checked_add(3)
            .ok_or(NetworkError::OverflowPayloadCalculation)?
            / 4;

        let expected_total_len = 6usize
            .checked_add(payload_bytes_len)
            .and_then(|v| v.checked_add(4))
            .ok_or(NetworkError::TotalLengthExceedsLimits)?;

        if bytes.len() < expected_total_len {
            return Err(NetworkError::PackageSmallerThanHeader);
        }

        let checksum_start = 6 + payload_bytes_len;
        let received_checksum = u32::from_be_bytes(
            bytes[checksum_start..checksum_start + 4]
                .try_into()
                .map_err(|_| NetworkError::PackageTooShort)?,
        );

        let calculated_checksum = Self::calculate_checksum(&bytes[0..checksum_start]);
        if received_checksum != calculated_checksum {
            return Err(NetworkError::ChecksumMismatch);
        }

        let payload = QuatBuffer::unpack(&bytes[6..checksum_start], quat_count)?;
        Ok(Self { payload })
    }

    fn calculate_checksum(data: &[u8]) -> u32 {
        let mut acc: u32 = 0x811C9DC5;
        for &byte in data {
            acc ^= byte as u32;
            acc = acc.wrapping_mul(0x01000193);
        }
        acc
    }
}
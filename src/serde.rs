use crate::errors::NetworkError;
use crate::quat::{Quat, QuatBuffer};
use std::convert::TryFrom;

pub trait QuatSerde {
    fn to_quats(&self) -> QuatBuffer;
    fn from_quats(buf: &QuatBuffer) -> Result<Self, NetworkError>
    where
        Self: Sized;
}

impl QuatSerde for u32 {
    fn to_quats(&self) -> QuatBuffer {
        let mut buf = QuatBuffer::with_capacity(16);
        for i in (0..16).rev() {
            let val = ((self >> (i * 2)) & 0b11) as u8;
            if let Ok(quat) = Quat::try_from(val) {
                buf.push(quat);
            }
        }
        buf
    }

    fn from_quats(buf: &QuatBuffer) -> Result<Self, NetworkError> {
        if buf.len() < 16 {
            return Err(NetworkError::InsufficientQuats);
        }

        let mut val = 0u32;
        for &q in buf.iter().take(16) {
            val = (val << 2) | (q as u32);
        }
        Ok(val)
    }
}
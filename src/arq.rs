use crate::errors::NetworkError;
use crate::fec::QuatFecEncoder;
use crate::quat::QuatBuffer;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ArqFrame {
    pub seq: u32,
    /// FEC-encoded payload bytes produced from QuatBuffer.
    pub fec_payload: Vec<u8>,
    pub orig_quat_count: usize,
}

#[derive(Debug, Clone)]
struct PendingEntry {
    frame: ArqFrame,
    sent_at: Instant,
    retries: u32,
}

pub struct ReliableArqEngine {
    next_seq: u32,
    ack_timeout: Duration,
    max_retries: u32,
    fec: QuatFecEncoder,
    pending: HashMap<u32, PendingEntry>,
}

impl ReliableArqEngine {
    pub fn new(block_size: usize, ack_timeout: Duration, max_retries: u32) -> Self {
        Self {
            next_seq: 0,
            ack_timeout,
            max_retries,
            fec: QuatFecEncoder::new(block_size),
            pending: HashMap::new(),
        }
    }

    /// Prepares an ARQ frame with FEC protection and tracks it until ACKed.
    pub fn prepare_send(&mut self, payload: &QuatBuffer) -> ArqFrame {
        let seq = self.next_seq;
        self.next_seq = self.next_seq.wrapping_add(1);

        let fec_payload = payload.pack_with_fec();
        let frame = ArqFrame {
            seq,
            fec_payload,
            orig_quat_count: payload.quats.len(),
        };

        self.pending.insert(
            seq,
            PendingEntry {
                frame: frame.clone(),
                sent_at: Instant::now(),
                retries: 0,
            },
        );

        frame
    }

    /// Marks a sequence number as acknowledged and removes it from pending retries.
    pub fn on_ack(&mut self, seq: u32) -> bool {
        self.pending.remove(&seq).is_some()
    }

    /// Collects frames that exceeded `ack_timeout` and schedules exponential backoff.
    pub fn collect_retransmissions(&mut self) -> Result<Vec<ArqFrame>, NetworkError> {
        let now = Instant::now();
        let mut to_retry = Vec::new();

        for entry in self.pending.values_mut() {
            let effective_timeout = self.ack_timeout.saturating_mul(1_u32 << entry.retries.min(6));
            if now.duration_since(entry.sent_at) >= effective_timeout {
                if entry.retries >= self.max_retries {
                    return Err(NetworkError::SetTimeoutFailed);
                }
                entry.retries += 1;
                entry.sent_at = now;
                to_retry.push(entry.frame.clone());
            }
        }

        Ok(to_retry)
    }

    /// Decodes FEC bytes back into a QuatBuffer and returns the sequence number to ACK.
    pub fn receive_frame(&self, frame: &ArqFrame) -> Result<(u32, QuatBuffer), NetworkError> {
        let _ = &self.fec;
        let mut decoded_nibbles = Vec::with_capacity(frame.fec_payload.len());
        for &code in &frame.fec_payload {
            decoded_nibbles.push(QuatBuffer::decode_fec_byte(code));
        }

        // Each FEC byte carries a 4-bit nibble -> two 2-bit quats (high 2 bits, low 2 bits).
        let mut raw_packed = Vec::with_capacity((decoded_nibbles.len() + 1) / 2);
        for pair in decoded_nibbles.chunks(2) {
            let hi = pair[0] & 0x0F;
            let lo = pair.get(1).copied().unwrap_or(0) & 0x0F;
            raw_packed.push((hi << 4) | lo);
        }

        let unpacked = QuatBuffer::unpack(&raw_packed, frame.orig_quat_count)?;
        Ok((frame.seq, unpacked))
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}
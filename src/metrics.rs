use std::sync::atomic::{AtomicU64, Ordering};

/// Industrial-grade metrics collector for qnetwork
pub struct NetworkMetrics {
    pub quats_sent: AtomicU64,
    pub quats_received: AtomicU64,
    pub packets_sent: AtomicU64,
    pub packets_received: AtomicU64,
    pub error_count: AtomicU64,
    pub crc_mismatches: AtomicU64,
}

impl NetworkMetrics {
    pub fn new() -> Self {
        Self {
            quats_sent: AtomicU64::new(0),
            quats_received: AtomicU64::new(0),
            packets_sent: AtomicU64::new(0),
            packets_received: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            crc_mismatches: AtomicU64::new(0),
        }
    }

    pub fn inc_quats_sent(&self, val: u64) {
        self.quats_sent.fetch_add(val, Ordering::Relaxed);
    }

    pub fn inc_quats_received(&self, val: u64) {
        self.quats_received.fetch_add(val, Ordering::Relaxed);
    }

    pub fn record_packet_sent(&self) {
        self.packets_sent.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_packet_received(&self) {
        self.packets_received.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_error(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_crc_mismatch(&self) {
        self.crc_mismatches.fetch_add(1, Ordering::Relaxed);
        self.record_error();
    }

    /// Generates an immutable snapshot of current metrics for monitoring/logging
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            quats_sent: self.quats_sent.load(Ordering::Relaxed),
            quats_received: self.quats_received.load(Ordering::Relaxed),
            packets_sent: self.packets_sent.load(Ordering::Relaxed),
            packets_received: self.packets_received.load(Ordering::Relaxed),
            error_count: self.error_count.load(Ordering::Relaxed),
            crc_mismatches: self.crc_mismatches.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MetricsSnapshot {
    pub quats_sent: u64,
    pub quats_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub error_count: u64,
    pub crc_mismatches: u64,
}
use crate::metrics::MetricsSnapshot;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct ExtendedTelemetry {
    pub retransmissions: AtomicU64,
    pub dropped_packets: AtomicU64,
    pub pool_hits: AtomicU64,
    pub pool_misses: AtomicU64,
    pub last_rtt_us: AtomicU64,
    pub total_rtt_us: AtomicU64,
    pub rtt_samples: AtomicU64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TelemetryReport {
    pub base: MetricsSnapshot,
    pub retransmissions: u64,
    pub dropped_packets: u64,
    pub pool_hits: u64,
    pub pool_misses: u64,
    pub last_rtt_us: u64,
    pub avg_rtt_us: u64,
}

impl ExtendedTelemetry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_retransmission(&self) {
        self.retransmissions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_dropped_packet(&self) {
        self.dropped_packets.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_pool_hit(&self) {
        self.pool_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_pool_miss(&self) {
        self.pool_misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_rtt_micros(&self, rtt_us: u64) {
        self.last_rtt_us.store(rtt_us, Ordering::Relaxed);
        self.total_rtt_us.fetch_add(rtt_us, Ordering::Relaxed);
        self.rtt_samples.fetch_add(1, Ordering::Relaxed);
    }

    pub fn report(&self, base: MetricsSnapshot) -> TelemetryReport {
        let samples = self.rtt_samples.load(Ordering::Relaxed);
        let total_rtt = self.total_rtt_us.load(Ordering::Relaxed);
        let avg_rtt_us = if samples == 0 { 0 } else { total_rtt / samples };

        TelemetryReport {
            base,
            retransmissions: self.retransmissions.load(Ordering::Relaxed),
            dropped_packets: self.dropped_packets.load(Ordering::Relaxed),
            pool_hits: self.pool_hits.load(Ordering::Relaxed),
            pool_misses: self.pool_misses.load(Ordering::Relaxed),
            last_rtt_us: self.last_rtt_us.load(Ordering::Relaxed),
            avg_rtt_us,
        }
    }

    /// Formats metrics in Prometheus / OpenMetrics exposition text format.
    pub fn to_prometheus(&self, prefix: &str, base: MetricsSnapshot) -> String {
        let r = self.report(base);
        format!(
            "# TYPE {p}_quats_sent_total counter\n{p}_quats_sent_total {qs}\n\
             # TYPE {p}_quats_received_total counter\n{p}_quats_received_total {qr}\n\
             # TYPE {p}_packets_sent_total counter\n{p}_packets_sent_total {ps}\n\
             # TYPE {p}_packets_received_total counter\n{p}_packets_received_total {pr}\n\
             # TYPE {p}_errors_total counter\n{p}_errors_total {err}\n\
             # TYPE {p}_crc_mismatches_total counter\n{p}_crc_mismatches_total {crc}\n\
             # TYPE {p}_retransmissions_total counter\n{p}_retransmissions_total {ret}\n\
             # TYPE {p}_dropped_packets_total counter\n{p}_dropped_packets_total {drp}\n\
             # TYPE {p}_pool_hits_total counter\n{p}_pool_hits_total {ph}\n\
             # TYPE {p}_pool_misses_total counter\n{p}_pool_misses_total {pm}\n\
             # TYPE {p}_rtt_last_microseconds gauge\n{p}_rtt_last_microseconds {lrtt}\n\
             # TYPE {p}_rtt_avg_microseconds gauge\n{p}_rtt_avg_microseconds {artt}\n",
            p = prefix,
            qs = r.base.quats_sent,
            qr = r.base.quats_received,
            ps = r.base.packets_sent,
            pr = r.base.packets_received,
            err = r.base.error_count,
            crc = r.base.crc_mismatches,
            ret = r.retransmissions,
            drp = r.dropped_packets,
            ph = r.pool_hits,
            pm = r.pool_misses,
            lrtt = r.last_rtt_us,
            artt = r.avg_rtt_us,
        )
    }
}
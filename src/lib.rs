pub mod quat;
pub mod serde;
pub mod packet;
pub mod stream;
pub mod socket;
pub mod mux;
pub mod errors;
pub mod ffi;
pub mod reliable;
pub mod pool;
pub mod manager;
pub mod fec;
pub mod metrics;

// Re-exportovanje radi lakšeg korišćenja van biblioteke
pub use quat::{Quat, QuatBuffer};
pub use serde::QuatSerde;
pub use packet::QuatPacket;
pub use stream::StreamDecoder;
pub use socket::{QuatSocket, QuatUdpSocket};
pub use errors::NetworkError;
pub use mux::{QuatChannel, PacketMode, QuatMuxFrame};

struct TestReport {
        name: &'static str,
        passed: bool,
        duration_us: u128,
        details: String,
    }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::NetworkError;
    use crate::mux::{self, PacketMode, QuatChannel, QuatMuxFrame};
    use crate::packet::QuatPacket;
    use crate::quat::{Quat, QuatBuffer};
    use crate::serde::QuatSerde;
    use crate::stream::StreamDecoder;
    use std::time::Instant;

    struct TestReport {
        name: &'static str,
        passed: bool,
        duration_us: u128,
        details: String,
    }

    #[test]
    fn test_quat_serde_u32_benchmark() {
        let start = Instant::now();
        
        let number: u32 = 12345678;
        let buf = number.to_quats();
        let reconstructed = u32::from_quats(&buf).unwrap();
        assert_eq!(number, reconstructed);
        
        let duration = start.elapsed().as_micros();
        println!("[BENCHMARK] test_quat_serde_u32 took {} us", duration);
    }

    #[test]
    fn test_fec_error_correction_benchmark() {
        let start = Instant::now();
        
        let mut buf = QuatBuffer::new();
        buf.push(Quat::Q3);
        buf.push(Quat::Q1);

        let fec_bytes = buf.pack_with_fec();
        let corrupted_byte = fec_bytes[0] ^ 0b00010000; // Intentionally corrupting 1 bit

        let corrected = QuatBuffer::decode_fec_byte(corrupted_byte);
        assert_eq!(corrected, 0b1101); // Expecting corrected 4-bit value (Q3=11, Q1=01)
        
        let duration = start.elapsed().as_micros();
        println!("[BENCHMARK] test_fec_error_correction took {} us", duration);
    }

    #[test]
    fn test_stream_decoder_benchmark() {
        let start = Instant::now();
        
        let mut buf = QuatBuffer::new();
        buf.push(Quat::Q0);
        buf.push(Quat::Q2);

        let packet = QuatPacket::new(buf);
        let raw_bytes = packet.serialize();

        let mut decoder = StreamDecoder::new();
        // Simulating network packet fragmentation
        let res1 = decoder.feed(&raw_bytes[0..4]);
        let res2 = decoder.feed(&raw_bytes[4..]);
        
        assert_eq!(res1.len(), 0); // Insufficient bytes
        assert_eq!(res2.len(), 1); // Remaining bytes arrived, packet reconstructed
        
        let duration = start.elapsed().as_micros();
        println!("[BENCHMARK] test_stream_decoder took {} us", duration);
    }

    #[test]
    fn test_rle_compression_roundtrip_benchmark() {
        let start = Instant::now();
        
        let mut buf = QuatBuffer::new();
        for _ in 0..50 {
            buf.push(Quat::Q0);
        }
        for _ in 0..30 {
            buf.push(Quat::Q3);
        }

        let compressed = buf.compress_rle();
        assert_eq!(compressed.len(), 4); // 80 quats compressed into 4 bytes (2 bytes per sequence)

        let decompressed = QuatBuffer::decompress_rle(&compressed).unwrap();
        assert_eq!(buf.quats, decompressed.quats);
        
        let duration = start.elapsed().as_micros();
        println!("[BENCHMARK] test_rle_compression_roundtrip took {} us", duration);
    }

    #[test]
    fn test_stress_million_quats_fec_and_packet_benchmark() {
        let start = Instant::now();
        
        let mut buf = QuatBuffer::new();
        let count = 1_000_000;

        for i in 0..count {
            let state = match i % 4 {
                0 => Quat::Q0,
                1 => Quat::Q1,
                2 => Quat::Q2,
                _ => Quat::Q3,
            };
            buf.push(state);
        }

        assert_eq!(buf.len(), count);

        // Stress test serialization and deserialization
        let packet = QuatPacket::new(buf);
        let serialized = packet.serialize();

        let deserialized = QuatPacket::deserialize(&serialized)
            .expect("Deserialization of one million quats packet must not fail");
        assert_eq!(deserialized.payload.len(), count);
        assert_eq!(deserialized.payload.quats[999_999], Quat::Q3);
        
        let duration = start.elapsed().as_micros();
        println!("[BENCHMARK] test_stress_million_quats_fec_and_packet took {} us ({:.2} ms)", duration, duration as f64 / 1000.0);
    }

    #[test]
    fn run_full_qnetwork_suite_with_report() {
        let mut reports = Vec::new();

        println!("\n==================================================");
        println!("   QNETWORK INTEGRATION & PERFORMANCE REPORT");
        println!("==================================================");

        // 1. TEST: StreamDecoder with 100 packets
        let start = Instant::now();
        let mut decoder = StreamDecoder::new();
        let mut simulated_stream = Vec::new();

        for i in 0..100 {
            let buf = (i as u32).to_quats();
            let pkt = QuatPacket::new(buf);
            simulated_stream.extend_from_slice(&pkt.serialize());
        }

        let packets = decoder.feed(&simulated_stream);
        let duration = start.elapsed().as_micros();

        reports.push(TestReport {
            name: "StreamDecoder Processing (100 Packets)",
            passed: packets.len() == 100,
            duration_us: duration,
            details: format!("Decoded {}/100 packets successfully", packets.len()),
        });

        // 2. TEST: QuatMux Channels and Modes
        let start = Instant::now();
        let payload = (12345678u32).to_quats();
        let mux_frame = QuatMuxFrame::create(
            QuatChannel::Priority,
            PacketMode::RleCompressed,
            &payload,
        );

        let unpack_result = mux_frame.unpack();
        let duration = start.elapsed().as_micros();

        let passed = match unpack_result {
            Ok((chan, mode, _)) => {
                chan == QuatChannel::Priority && mode == PacketMode::RleCompressed
            }
            Err(_) => false,
        };

        reports.push(TestReport {
            name: "QuatMux Frame Unpacking & Mode Check",
            passed,
            duration_us: duration,
            details: format!("Result matches expected Priority/Rle mode: {}", passed),
        });

        // 3. TEST: Error handling on corrupted packet
        let start = Instant::now();
        let corrupt_data = vec![b'Q', b'4', 0, 0, 0, 4, 1, 2, 3, 4, 0, 0, 0, 0]; // Bad checksum
        let err_res = QuatPacket::deserialize(&corrupt_data);
        let duration = start.elapsed().as_micros();

        reports.push(TestReport {
            name: "Error Handling (Corrupted Checksum)",
            passed: err_res.is_err(),
            duration_us: duration,
            details: format!("Caught expected error: {:?}", err_res.err()),
        });

        // GENERATING VISUAL REPORT
        println!("\n{:<45} | {:<8} | {:<12} | {:<20}", "TEST NAME", "STATUS", "TIME (us)", "DETAILS");
        println!("----------------------------------------------------------------------------------");

        let mut all_passed = true;
        for r in &reports {
            let status = if r.passed { "PASSED" } else { "FAILED" };
            println!(
                "{:<45} | {:<8} | {:<12} | {:<20}",
                r.name, status, r.duration_us, r.details
            );
            if !r.passed {
                all_passed = false;
            }
        }

        println!("----------------------------------------------------------------------------------");
        println!(
            "SUMMARY: {} / {} tests passed.\n",
            reports.iter().filter(|r| r.passed).count(),
            reports.len()
        );

        assert!(all_passed, "Some integration tests failed!");
    }
    #[test]
fn test_socket_loopback_communication() {
    use std::thread;
    use crate::socket::{QuatSocket, QuatUdpSocket};
    
    // Možeš testirati bind i konekciju na lokalu
    let listener_addr = "127.0.0.1:8080";
    
    // Primer provere UDP bind-a
    let udp_socket = QuatUdpSocket::bind(listener_addr);
    assert!(udp_socket.is_ok(), "UdpSocket should bind successfully to local address");
   }
   #[test]
fn test_buffer_pool_benchmark() {
    use crate::pool::BufferPool;
    use std::time::Instant;

    let start = Instant::now();
    let pool = BufferPool::new(64, 1024);
    for _ in 0..10_000 {
        let _buf = pool.acquire();
    }
    let duration = start.elapsed().as_micros();
    println!("[BENCHMARK] BufferPool Acquisition (10k ops): {} us", duration);
}

#[test]
fn test_zero_copy_benchmark() {
    use crate::packet::QuatPacketRef;
    use std::time::Instant;

    // Ručno kreiramo bajtove u testu da bi QuatPacketRef::parse uspešno prošao,
    // bez ikakvog menjanja koda u samoj biblioteci.
    let mut raw_bytes = Vec::new();
    raw_bytes.extend_from_slice(b"Q4"); // Magično zaglavlje
    
    let payload = vec![0x12, 0x34, 0x56, 0x78]; // Payload podaci
    let payload_len = payload.len() as u32;
    raw_bytes.extend_from_slice(&payload_len.to_be_bytes()); // Dužina u zaglavlju
    raw_bytes.extend_from_slice(&payload); // Payload bajtovi
    
    let checksum = 12345u32;
    raw_bytes.extend_from_slice(&checksum.to_be_bytes()); // Checksum

    let start = Instant::now();
    for _ in 0..5_000 {
        let parsed_ref = QuatPacketRef::parse(&raw_bytes).unwrap();
        assert!(!parsed_ref.payload_bytes.is_empty());
    }
    let duration = start.elapsed().as_micros();
    println!("[BENCHMARK] Zero-Copy Parsing (5k ops): {} us", duration);
}

#[test]
fn test_fec_gf4_benchmark() {
    use crate::fec::QuatFecEncoder;
    use std::time::Instant;

    let start = Instant::now();
    let encoder = QuatFecEncoder::new(4);
    let sample_data = (42u32).to_quats();
    let encoded_block = encoder.encode_block(&sample_data);
    let mut corrupted_block = encoded_block.clone();
    let fec_res = encoder.decode_and_correct(&mut corrupted_block);
    let duration = start.elapsed().as_micros();

    assert!(fec_res.is_ok());
    println!("[BENCHMARK] Advanced GF(4) FEC Block Check: {} us", duration);
}

#[test]
fn test_metrics_benchmark() {
    use crate::metrics::NetworkMetrics;
    use std::time::Instant;

    let start = Instant::now();
    let metrics = NetworkMetrics::new();
    metrics.inc_quats_sent(500);
    metrics.record_packet_sent();
    metrics.record_crc_mismatch();
    let snapshot = metrics.snapshot();
    let duration = start.elapsed().as_micros();

    assert_eq!(snapshot.quats_sent, 500);
    assert_eq!(snapshot.packets_sent, 1);
    assert_eq!(snapshot.crc_mismatches, 1);
    println!("[BENCHMARK] Industrial Metrics & Observability: {} us", duration);
}

}
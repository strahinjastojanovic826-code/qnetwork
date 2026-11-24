# qnetwork

`qnetwork` is a lightweight, custom networking library written in Rust. It enables packet serialization, stream parsing, and Forward Error Correction (FEC) designed around **Quats** (2-bit quaternary values: `Q0`, `Q1`, `Q2`, `Q3`).

---

## Key Features

* **Quat Data Representation**: Custom 2-bit values packed efficiently into bytes (4 quats per byte).
* **Forward Error Correction (FEC)**: Built-in Hamming(7,4) encoding and single-bit auto-correction (`pack_with_fec` / `decode_fec_byte`).
* **Run-Length Encoding (RLE)**: Lossless compression stream for repeating Quat sequences (`compress_rle` / `decompress_rle`).
* **Packet Framing (`QuatPacket`)**: Structured binary packets with a `"Q4"` magic header, payload length, and checksum validation.
* **Stream Decoder (`StreamDecoder`)**: Handles incoming network buffer streams over TCP/IP sockets, auto-detecting framing boundaries and extracting valid packets.
* **Multiplexing & Channels (`QuatMuxFrame`)**: Supports multi-channel routing (Control, Data, Priority, Custom) and packet execution modes (Raw, FEC, RLE).
* **C / C++ / C# Interoperability**: Native C-FFI bindings (`cdylib`) allowing seamless integration with C, C++, and .NET applications.
* **Custom Serialization Trait (`QuatSerde`)**: Built-in implementations for data types like `u32`.
* **`Async I/O Support`**: Tokio-powered asynchronous networking support (`AsyncQuatSocket`) for non-blocking high-concurrency stream operations.
* **`Reliable UDP (ARQ)`**: Built-in reliability layer over standard UDP for sequence tracking and packet transmission management.
* **`Advanced FEC Parity`**: Extended block-level XOR parity generation (`generate_parity_packet`) for recovering lost data packages.
* **`Standard CRC32 Support`**: Alternative standard polynomial checksum calculations (`calculate_crc32`) alongside custom checksum algorithms for robust integrity verification.
* **Buffer Pooling**: High-performance buffer recycling mechanism (`BufferPool`) designed to eliminate heap allocations and reduce memory fragmentation.
* **Zero-Copy Deserialization**: Ultra-fast packet parsing (`QuatPacketRef`) operating directly over raw network byte slices without unnecessary data copying.
* **Observability & Metrics**: Thread-safe atomic metrics collection (`NetworkMetrics`) for real-time tracking of transferred quats, packets, errors, and CRC mismatches.
* **GF(4) Quaternary Algebra (`Gf4`)**: True Galois field arithmetic (`GF(2)[x]/(x^2 + x + 1)`) with addition, multiplication, multiplicative inverse, parity calculation, and linear combinations over 2-bit quaternary states.
* **ARQ + FEC Reliable Transport (`ReliableArqEngine`)**: Sequence-numbered frame tracking, ACK handling, exponential backoff retransmissions, and FEC-protected payload encoding/decoding.
* **Protocol Hardening & Bounds Checking (`parse_frame_safe`)**: Safe wire-frame validation with strict frame/quat size limits and overflow-checked length calculations without panicking on malformed input.
* **Backpressure & Async Pool Helpers (`bounded_command_channel`, `SharedBufferPool`)**: Bounded Tokio command channels to prevent unbounded memory growth under load alongside reusable pooled buffers.
* **Extended Telemetry & Prometheus Export (`ExtendedTelemetry`)**: Atomic tracking of retransmissions, dropped packets, buffer pool hits/misses, and RTT microseconds with Prometheus/OpenMetrics text export.
* **Hardened C ABI (`QNetStatus`, `qnet_handle_*`)**: Panic-safe `extern "C"` handle lifecycle with explicit status codes, null/capacity checks, and C/C++/Python (`ctypes`) interoperability.

---

## Licensing

This project is dual-licensed:

* **Open Source (AGPLv3):** Free to use for open-source projects. If you use or modify this library in your software or network service, your source code must also be open-source under AGPLv3.
* **Commercial License:** Required if you want to use `qnetwork` in closed-source or proprietary commercial software without AGPLv3 restrictions.

For commercial licensing, contact: **strahinjastojanovic826@gmail.com**

---

## Benchmark

| Test / Benchmark Name | Status | Time (us) | Scale / Notes |
| :--- | :---: | :---: | :--- |
| `test_fec_error_correction`[cite: 41] | `ok`[cite: 41] | `2`[cite: 41] | Single-bit FEC error correction[cite: 24, 41] |
| `test_rle_compression_roundtrip`[cite: 41] | `ok`[cite: 42] | `5`[cite: 41] | RLE compression & decompression roundtrip[cite: 25, 41] |
| `test_quat_serde_u32`[cite: 41] | `ok`[cite: 42] | `9`[cite: 41] | `u32` quaternary serialization & deserialization[cite: 24, 41] |
| `test_stream_decoder`[cite: 41] | `ok`[cite: 42] | `10`[cite: 41] | Stream framing & packet reconstruction[cite: 24, 41] |
| `Advanced GF(4) FEC Block Check` (`test_fec_gf4`)[cite: 41, 42] | `ok`[cite: 42] | `15`[cite: 41] | GF(4) FEC block encode & error check[cite: 28, 41] |
| `test_observability_prometheus_export`[cite: 41] | `ok`[cite: 42] | `19`[cite: 41] | Extended telemetry & Prometheus text export[cite: 41] |
| `Industrial Metrics & Observability` (`test_metrics`)[cite: 41, 42] | `ok`[cite: 42] | `0`[cite: 41] | Atomic counters & snapshot read (< 1 us)[cite: 28, 29, 41] |
| `test_gf4_arithmetic_and_parity`[cite: 41] | `ok`[cite: 42] | `361`[cite: 41] | GF(4) field arithmetic, inverse & block parity[cite: 41] |
| `Zero-Copy Parsing` (`test_zero_copy`)[cite: 41, 42] | `ok`[cite: 42] | `570`[cite: 41] | `5k ops` zero-copy frame parsing[cite: 28, 41] |
| `test_hardening_frame_bounds`[cite: 41] | `ok`[cite: 42] | `744`[cite: 41] | `10k ops` safe bounds & overflow validation[cite: 41] |
| `BufferPool Acquisition` (`test_buffer_pool`)[cite: 41, 42] | `ok`[cite: 42] | `1349`[cite: 41] | `10k ops` pooled buffer acquisition[cite: 28, 41] |
| `test_c_abi_handle_lifecycle`[cite: 41] | `ok`[cite: 42] | `1367`[cite: 41] | `1k ops` C ABI handle create/push/pack/unpack/free[cite: 41] |
| `test_async_backpressure_and_pool`[cite: 41] | `ok`[cite: 42] | `1423`[cite: 41] | Bounded async channel backpressure & pool[cite: 41] |
| `test_arq_fec_retransmission`[cite: 41] | `ok`[cite: 42] | `2288`[cite: 41] | ARQ sequence tracking, FEC & retransmission[cite: 41] |
| `test_stress_million_quats_fec_and_packet`[cite: 42] | `ok`[cite: 42] | `59218` (`59.22 ms`)[cite: 42] | `1,000,000` quats serialization & deserialization stress test[cite: 25, 42] |
| `test_socket_loopback_communication`[cite: 42] | `ok`[cite: 42] | — | Socket loopback test[cite: 27, 42] |

### Integration Suite Report (`run_full_qnetwork_suite_with_report`)

| Integration Test Name | Status | Time (us) | Details |
| :--- | :---: | :---: | :--- |
| `StreamDecoder Processing (100 Packets)`[cite: 41, 42] | `PASSED`[cite: 41, 42] | `486`[cite: 41, 42] | Decoded 100/100 packets successfully[cite: 41, 42] |
| `QuatMux Frame Unpacking & Mode Check`[cite: 41, 42] | `PASSED`[cite: 41, 42] | `9`[cite: 41, 42] | Result matches expected Priority/Rle mode: `true`[cite: 41, 42] |
| `Error Handling (Corrupted Checksum)`[cite: 41, 42] | `PASSED`[cite: 41, 42] | `0`[cite: 41, 42] | Caught expected error: `Some(ChecksumMismatch)`[cite: 41, 42] |

---

## Feedback & Contributing

Feedback, bug reports, and feature requests are very welcome!

* **Bug reports & Feature suggestions**: Please open an issue on the GitHub Issues tab.
* **Code contributions**: Feel free to fork the repository and submit a Pull Request.

---

## Frame Structure

Each serialized packet follows a strict frame format:

| Offset (Bytes) | Field | Description |
| :--- | :--- | :--- |
| `0..2` | **Magic Header** | Always ASCII `b"Q4"` |
| `2..6` | **Quat Count** | Big-endian `u32` representing total quats |
| `6..N` | **Packed Payload** | Data payload packed into bytes |
| `N..N+4` | **Checksum** | 32-bit integrity verification checksum |

---

## Quickstart & Usage

### 1. Cargo.toml
Add `qnetwork` to your project dependencies:

```toml
[dependencies]
qnetwork = "0.7.0"
```

### 2. Serialization and Packet Building

```rust
use qnetwork::{NetworkError, QuatBuffer, QuatPacket, QuatSerde};

fn main() -> Result<(), NetworkError> {
    // Convert a u32 number into a QuatBuffer
    let number: u32 = 42;
    let quat_buf = number.to_quats();

    // Wrap in a QuatPacket and serialize to raw bytes
    let packet = QuatPacket::new(quat_buf);
    let raw_bytes: Vec<u8> = packet.serialize();

    // Deserialize back from raw bytes
    let deserialized_packet = QuatPacket::deserialize(&raw_bytes)?;

    // Read value back using QuatSerde
    let restored_number = u32::from_quats(&deserialized_packet.payload)?;
    assert_eq!(number, restored_number);

    Ok(())
}
```

### 3. Stream Parsing over TCP Sockets

```rust
use qnetwork::{QuatPacket, StreamDecoder};

fn process_incoming_data(decoder: &mut StreamDecoder, chunk: &[u8]) {
    // Feed raw socket chunks into the stream decoder
    let packets: Vec<QuatPacket> = decoder.feed(chunk);

    for packet in packets {
        println!("Received valid QuatPacket with {} quats", packet.payload.len());
    }
}
```

### 4. Error Correction via FEC (Hamming 7,4)

```rust
use qnetwork::{Quat, QuatBuffer};

fn main() {
    let mut buffer = QuatBuffer::new();
    buffer.push(Quat::Q3);
    buffer.push(Quat::Q1);

    // Encode payload with Hamming(7,4) parity bits
    let encoded_bytes = buffer.pack_with_fec();

    // Auto-correct single-bit flip on receiver side
    let raw_received_byte = encoded_bytes[0] ^ 0b00000010; // Corrupt 1 bit
    let corrected_nibble = QuatBuffer::decode_fec_byte(raw_received_byte);

    println!("Successfully corrected error! Decoded nibble: {:b}", corrected_nibble);
}

```

### 5. RLE Data Compression

```rust
use qnetwork::{NetworkError, Quat, QuatBuffer};

fn main() -> Result<(), NetworkError> {
    let mut buffer = QuatBuffer::new();
    for _ in 0..100 {
        buffer.push(Quat::Q0);
    }

    // Compress repetitive quats
    let compressed = buffer.compress_rle();
    println!("Compressed 100 quats to {} bytes!", compressed.len());

    // Decompress back
    let restored = QuatBuffer::decompress_rle(&compressed)?;
    assert_eq!(buffer.quats, restored.quats);

    Ok(())
}

```
### 6. Fast UDP Communication

```rust
use qnetwork::{NetworkError, Quat, QuatBuffer, QuatPacket, QuatUdpSocket};

fn main() -> Result<(), NetworkError> {
    // Bind UDP socket
    let socket = QuatUdpSocket::bind("127.0.0.1:8080")?;

    // Prepare packet
    let mut buffer = QuatBuffer::new();
    buffer.push(Quat::Q2);
    let packet = QuatPacket::new(buffer);

    // Send packet to remote endpoint
    socket.send_to(&packet, "127.0.0.1:8081")?;

    Ok(())
}

```

### 7. Mutex channel

```rust
use qnetwork::{NetworkError, PacketMode, Quat, QuatBuffer, QuatChannel, QuatMuxFrame, QuatPacket};

fn main() -> Result<(), NetworkError> {
    // 1. Data preparation
    let mut buffer = QuatBuffer::new();
    buffer.push(Quat::Q3);
    buffer.push(Quat::Q1);
    buffer.push(Quat::Q0);

    // 2. Creating a multiplexed frame (Channel + Mode)
    let mux_frame = QuatMuxFrame::create(
        QuatChannel::Priority,
        PacketMode::RleCompressed,
        &buffer,
    );

    // 3. Serialization to bytes for network transmission
    let raw_bytes = mux_frame.packet.serialize();

    // --- (Network transmission) ---

    // 4. Reception and deserialization on the other side
    let received_packet = QuatPacket::deserialize(&raw_bytes)?;

    // Wrap the received packet back into a Mux frame for unpacking
    let reconstructed_frame = QuatMuxFrame {
        channel: QuatChannel::Control, // Temporary, will be overwritten
        mode: PacketMode::Raw,
        packet: received_packet,
    };

    // 5. Unpacking the frame and reading channel and mode
    let (channel, mode, payload) = reconstructed_frame.unpack()?;

    println!("Successfully received!");
    println!("- Channel: {:?}", channel);
    println!("- Mode: {:?}", mode);
    println!("- Number of quats in payload: {}", payload.len());

    Ok(())
}

```

### 8. High-Performance Buffer Pooling

```rust
use qnetwork::pool::BufferPool;

fn main() {
    // Create a buffer pool with capacity for 64 buffers of 1024 bytes each
    let pool = BufferPool::new(64, 1024);

    // Acquire a reusable buffer from the pool (avoids heap allocations)
    let mut pooled_buf = pool.acquire();
    
    if let Some(buf) = &mut pooled_buf.buf {
        buf.extend_from_slice(b"Quaternary network payload");
        println!("Buffer acquired and used, length: {}", buf.len());
    }
    // Automatically recycled back to the pool when 'pooled_buf' goes out of scope
}
```

### 9. Zero-Copy Packet Parsing

```rust
use qnetwork::packet::{QuatPacket, QuatPacketRef, QuatBuffer};
use qnetwork::errors::NetworkError;

fn main() -> Result<(), NetworkError> {
    let mut q_buf = QuatBuffer::new();
    q_buf.push(qnetwork::quat::Quat::Q1);
    
    let packet = QuatPacket::new(q_buf);
    let wire_bytes = packet.serialize();

    // Parse the incoming stream using zero-copy (borrowing slice data directly)
    let packet_ref = QuatPacketRef::parse(&wire_bytes)?;
    println!("Zero-copy parsed payload bytes len: {}", packet_ref.payload_bytes.len());
    println!("Checksum: {}", packet_ref.checksum);

    Ok(())
}
```

### 10. Observability & Network Metrics

```rust
use qnetwork::metrics::NetworkMetrics;

fn main() {
    let metrics = NetworkMetrics::new();

    // Simulate sending network traffic
    metrics.inc_quats_sent(100);
    metrics.record_packet_sent();

    // Fetch snapshot for monitoring
    let snapshot = metrics.snapshot();
    println!("Total quats sent: {}", snapshot.quats_sent);
    println!("Total packets sent: {}", snapshot.packets_sent);
    println!("Current error count: {}", snapshot.error_count);
}
```

### 11. GF(4) Quaternary Algebra
```rust
use qnetwork::gf4::{encode_gf4_block, gf4_parity, Gf4};
use qnetwork::quat::{Quat, QuatBuffer};

fn main() {
    let prod = Gf4::new(Quat::Q2).mul(Gf4::new(Quat::Q3));
    assert_eq!(prod, Gf4::new(Quat::Q1));

    let mut buf = QuatBuffer::new();
    buf.push(Quat::Q1);
    buf.push(Quat::Q2);

    let parity = gf4_parity(&buf.quats);
    let encoded = encode_gf4_block(&buf);
    println!("GF(4) parity: {:?}, encoded len: {}", parity, encoded.quats.len());
}
```

### 12. ARQ + FEC Reliable Transport
```rust
use qnetwork::errors::NetworkError;
use qnetwork::quat::{Quat, QuatBuffer};
use std::time::Duration;

fn main() -> Result<(), NetworkError> {
    let mut engine = qnetwork::arq::ReliableArqEngine::new(4, Duration::from_millis(200), 3);
    let mut payload = QuatBuffer::new();
    payload.push(Quat::Q0);
    payload.push(Quat::Q3);

    let frame = engine.prepare_send(&payload);
    let (seq, unpacked) = engine.receive_frame(&frame)?;
    assert!(engine.on_ack(seq));
    println!("ACKed seq {} with {} quats", seq, unpacked.quats.len());
    Ok(())
}
```

### 13. Hardened Frame Parsing & Async Backpressure
```rust
use qnetwork::async_ext::{bounded_command_channel, send_command_backpressured, SharedBufferPool};
use qnetwork::errors::NetworkError;
use qnetwork::hardening::parse_frame_safe;
use qnetwork::manager::ConnectionCommand;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[tokio::main]
async fn main() -> Result<(), NetworkError> {
    let bad = b"Q4\x00\x00\x00\x01\x00";
    assert!(parse_frame_safe(bad).is_err());

    let (tx, mut rx) = bounded_command_channel(8);
    let _pool = SharedBufferPool::new(16, 512);
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);

    send_command_backpressured(&tx, ConnectionCommand::Unregister { addr }).await?;
    let _ = rx.recv().await;
    Ok(())
}
```

### 14. Extended Telemetry & Prometheus Export
```rust
use qnetwork::metrics::NetworkMetrics;
use qnetwork::observability::ExtendedTelemetry;

fn main() {
    let metrics = NetworkMetrics::new();
    metrics.inc_quats_sent(64);
    metrics.record_packet_sent();

    let telemetry = ExtendedTelemetry::new();
    telemetry.record_pool_hit();
    telemetry.record_rtt_micros(150);

    let prom = telemetry.to_prometheus("qnetwork", metrics.snapshot());
    println!("{}", prom);
}
```

## C / C++ / C# Interoperability (FFI)

`qnetwork` can be compiled into a native C-compatible dynamic library (`.dll` on Windows, `.so` on Linux, `.dylib` on macOS), enabling seamless usage from C, C++, and C#.

---

### 1. C Integration

Include the generated C header file (`qnetwork.h`) and link against the compiled dynamic library.

**`qnetwork.h` (Header File):**
```c
#ifndef QNETWORK_H
#define QNETWORK_H

#include <stdint.h>
#include <stddef.h>

typedef struct QuatBuffer QuatBuffer;

QuatBuffer* quat_buffer_create();
void quat_buffer_free(QuatBuffer* ptr);
int32_t quat_buffer_push(QuatBuffer* ptr, uint8_t val);
int32_t quat_buffer_pack(const QuatBuffer* ptr, uint8_t* out_buf, size_t out_capacity, size_t* out_len);

#endif
```

**`main.c` Example:**
```c
#include <stdio.h>
#include "qnetwork.h"

int main() {
    QuatBuffer* buf = quat_buffer_create();

    quat_buffer_push(buf, 0); // Q0
    quat_buffer_push(buf, 1); // Q1
    quat_buffer_push(buf, 2); // Q2
    quat_buffer_push(buf, 3); // Q3

    uint8_t output[16];
    size_t out_len = 0;

    if (quat_buffer_pack(buf, output, sizeof(output), &out_len) == 0) {
        printf("Packed %zu byte(s) successfully!\n", out_len);
    }

    quat_buffer_free(buf);
    return 0;
}
```

---

### C extended C ABI

```c
#include <stdint.h>
#include <stddef.h>
#include <stdio.h>

typedef enum {
    QNET_STATUS_OK = 0,
    QNET_STATUS_NULL_POINTER = -1,
    QNET_STATUS_INVALID_QUAT = -2,
    QNET_STATUS_BUFFER_TOO_SMALL = -3,
    QNET_STATUS_INVALID_STREAM = -4,
    QNET_STATUS_LENGTH_OVERFLOW = -5,
    QNET_STATUS_INTERNAL_PANIC = -99
} QNetStatus;

typedef struct QNetQuatHandle QNetQuatHandle;

QNetQuatHandle* qnet_handle_new(void);
void qnet_handle_free(QNetQuatHandle* handle);
QNetStatus qnet_handle_push_quat(QNetQuatHandle* handle, uint8_t val);
QNetStatus qnet_handle_pack(
    const QNetQuatHandle* handle,
    uint8_t* out_buf,
    size_t out_capacity,
    size_t* out_written
);
QNetStatus qnet_handle_unpack(
    QNetQuatHandle* handle,
    const uint8_t* packed_ptr,
    size_t packed_len,
    size_t total_quats
);

int main(void) {
    QNetQuatHandle* handle = qnet_handle_new();
    if (!handle) {
        fprintf(stderr, "Failed to allocate QNetQuatHandle\n");
        return 1;
    }

    if (qnet_handle_push_quat(handle, 0) != QNET_STATUS_OK ||
        qnet_handle_push_quat(handle, 1) != QNET_STATUS_OK ||
        qnet_handle_push_quat(handle, 2) != QNET_STATUS_OK ||
        qnet_handle_push_quat(handle, 3) != QNET_STATUS_OK) {
        fprintf(stderr, "Failed to push quaternary values\n");
        qnet_handle_free(handle);
        return 1;
    }

    /* Invalid quat check (must be 0..=3) */
    if (qnet_handle_push_quat(handle, 9) == QNET_STATUS_INVALID_QUAT) {
        printf("Rejected invalid quat value as expected\n");
    }

    uint8_t packed[8] = {0};
    size_t written = 0;
    QNetStatus st = qnet_handle_pack(handle, packed, sizeof(packed), &written);
    if (st != QNET_STATUS_OK) {
        fprintf(stderr, "Pack failed with status %d\n", (int)st);
        qnet_handle_free(handle);
        return 1;
    }
    printf("Packed 4 quats into %zu byte(s)\n", written);

    st = qnet_handle_unpack(handle, packed, written, 4);
    if (st != QNET_STATUS_OK) {
        fprintf(stderr, "Unpack failed with status %d\n", (int)st);
        qnet_handle_free(handle);
        return 1;
    }
    printf("Unpacked %zu byte(s) back into 4 quats\n", written);

    qnet_handle_free(handle);
    return 0;
}
```

---

**Compilation Command:**
```bash
gcc main.c -L. -lqnetwork -o c_app
```

---

### 2. C++ Integration

In C++, you can directly include `qnetwork.h` inside an `extern "C"` block, or wrap it in a class using RAII for automated memory management.

**`main.cpp` Example:**
```cpp
#include <iostream>
#include <vector>

extern "C" {
    #include "qnetwork.h"
}

class QuatBufferWrapper {
private:
    QuatBuffer* handle;

public:
    QuatBufferWrapper() {
        handle = quat_buffer_create();
    }

    ~QuatBufferWrapper() {
        quat_buffer_free(handle);
    }

    void push(uint8_t val) {
        quat_buffer_push(handle, val);
    }

    QuatBuffer* get() const { return handle; }
};

int main() {
    QuatBufferWrapper buf;
    buf.push(0);
    buf.push(1);
    buf.push(2);
    buf.push(3);

    std::cout << "Successfully pushed quats from C++!" << std::endl;
    return 0;
}
```

**Compilation Command:**
```bash
g++ main.cpp -L. -lqnetwork -o cpp_app
```

---

### 3. C# (.NET) Integration

In C#, you do not need header files. Import the native functions using **P/Invoke** (`[DllImport]`) and place `qnetwork.dll` (or `.so`) in your build output directory.

**`Program.cs` Example:**
```csharp
using System;
using System.Runtime.InteropServices;

class Program
{
    [DllImport("qnetwork.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr quat_buffer_create();

    [DllImport("qnetwork.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern void quat_buffer_free(IntPtr ptr);

    [DllImport("qnetwork.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern int quat_buffer_push(IntPtr ptr, byte val);

    static void Main()
    {
        IntPtr buf = quat_buffer_create();

        quat_buffer_push(buf, 0);
        quat_buffer_push(buf, 1);
        quat_buffer_push(buf, 2);
        quat_buffer_push(buf, 3);

        Console.WriteLine("Successfully executed Rust FFI from C#!");

        quat_buffer_free(buf);
    }
}
```
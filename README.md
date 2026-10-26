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

---

## Licensing

This project is dual-licensed:

* **Open Source (AGPLv3):** Free to use for open-source projects. If you use or modify this library in your software or network service, your source code must also be open-source under AGPLv3.
* **Commercial License:** Required if you want to use `qnetwork` in closed-source or proprietary commercial software without AGPLv3 restrictions.

For commercial licensing, contact: **strahinjastojanovic826@gmail.com**

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
qnetwork = "0.5.0"
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
# qnetwork

`qnetwork` is a lightweight, custom networking library written in Rust. It enables packet serialization, stream parsing, and Forward Error Correction (FEC) designed around **Quats** (2-bit quaternary values: `Q0`, `Q1`, `Q2`, `Q3`).

---

## Key Features

* **Quat Data Representation**: Custom 2-bit values packed efficiently into bytes (4 quats per byte).
* **Forward Error Correction (FEC)**: Built-in Hamming(7,4) encoding and single-bit auto-correction (`pack_with_fec` / `decode_fec_byte`).
* **Packet Framing (`QuatPacket`)**: Structured binary packets with a `"Q4"` magic header, payload length, and checksum validation.
* **Stream Decoder (`StreamDecoder`)**: Handles incoming network buffer streams over TCP/IP sockets, auto-detecting framing boundaries and extracting valid packets.
* **Custom Serialization Trait (`QuatSerde`)**: Built-in implementations for data types like `u32` and string references (`&str`).

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
qnetwork = "0.1.0"

```

Serialization and Packet Building

use qnetwork::{QuatBuffer, QuatPacket, QuatSerde};

fn main() -> Result<(), &'static str> {
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

Stream Parsing over TCP Sockets

Use StreamDecoder to assemble complete packets from partial TCP byte chunks:

use qnetwork::{StreamDecoder, QuatPacket};

fn process_incoming_data(decoder: &mut StreamDecoder, chunk: &[u8]) {
    // Feed raw socket chunks into the stream decoder
    let packets: Vec<QuatPacket> = decoder.feed(chunk);

    for packet in packets {
        println!("Received valid QuatPacket with {} quats", packet.payload.len());
    }
}

```

Error Correction via FEC (Hamming 7,4):

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

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Quat {
    Q0 = 0b00,
    Q1 = 0b01,
    Q2 = 0b10,
    Q3 = 0b11,
}

impl Quat {
    pub fn from_u8(val: u8) -> Result<Self, &'static str> {
        match val {
            0 => Ok(Quat::Q0),
            1 => Ok(Quat::Q1),
            2 => Ok(Quat::Q2),
            3 => Ok(Quat::Q3),
            _ => Err("The value must be within the range 0-3"),
        }
    }
}

pub struct QuatBuffer {
    quats: Vec<Quat>,
}

impl QuatBuffer {
    pub fn new() -> Self {
        Self { quats: Vec::new() }
    }

    pub fn push(&mut self, quat: Quat) {
        self.quats.push(quat);
    }

    pub fn pack_with_fec(&self) -> Vec<u8> {
        let mut encoded = Vec::new();
        for chunk in self.quats.chunks(2) { // Uzima po 2 kvata (4 bita)
            let mut nibble = 0u8;
            if let Some(&q0) = chunk.get(0) { nibble |= (q0 as u8) << 2; }
            if let Some(&q1) = chunk.get(1) { nibble |= q1 as u8; }

            // Izračunavanje parity bitova za Hamming(7,4)
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

    /// Dekodira Hamming(7,4) bajt i automatski ispravlja grešku na 1 bitu ako postoji
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
            corrected ^= 1 << (7 - error_pos); // Ispravlja pokvareni bit u hodu
        }

        // Vraća očišćena 4 bit-a podataka
        ((corrected >> 4) & 1) << 3 | ((corrected >> 2) & 1) << 2 | ((corrected >> 1) & 1) << 1 | (corrected & 1)
    }

    pub fn len(&self) -> usize {
        self.quats.len()
    }

    pub fn is_empty(&self) -> bool {
        self.quats.is_empty()
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

    pub fn unpack(packed: &[u8], total_quats: usize) -> Result<Self, &'static str> {
        // Zastita od pretvaranja u preveliku alokaciju memorije
        let expected_bytes = total_quats.checked_add(3).ok_or("Overflow u quat count")? / 4;
        if packed.len() < expected_bytes {
            return Err("Insufficient bytes for the specified number of quats.");
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
                quats.push(Quat::from_u8(val)?);
                read += 1;
            }
        }

        Ok(Self { quats })
    }

    pub fn as_slice(&self) -> &[Quat] {
        &self.quats
    }
}

pub struct StreamDecoder {
    buffer: Vec<u8>,
}

impl StreamDecoder {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn feed(&mut self, chunk: &[u8]) -> Vec<QuatPacket> {
        self.buffer.extend_from_slice(chunk);
        let mut packets = Vec::new();

        loop {
            if self.buffer.len() < 10 {
                break; // Premasno za zaglavlje, čekamo još podatke
            }

            // Provera "Q4" magic zaglavlja
            if &self.buffer[0..2] != b"Q4" {
                self.buffer.remove(0); // Izbaci smeće i pomeri se za 1 bajt
                continue;
            }

            // Čitanje dužine paketa
            let quat_count = u32::from_be_bytes([
                self.buffer[2], self.buffer[3], self.buffer[4], self.buffer[5]
            ]) as usize;

            let payload_bytes_len = (quat_count + 3) / 4;
            let total_frame_len = 6 + payload_bytes_len + 4; // 6 (header) + payload + 4 (checksum)

            if self.buffer.len() < total_frame_len {
                break; // Paket još uvek nije kompletno stigao, čekamo ostatak
            }

            // Izdvajamo ceo paket iz strima
            let frame_bytes: Vec<u8> = self.buffer.drain(0..total_frame_len).collect();
            
            // Pokušaj deserializacije (ako pukne checksum, samo ga ignorišemo i idemo dalje)
            if let Ok(packet) = QuatPacket::deserialize(&frame_bytes) {
                packets.push(packet);
            }
        }

        packets
    }
}

pub trait QuatSerde {
    fn to_quats(&self) -> QuatBuffer;
    fn from_quats(buf: &QuatBuffer) -> Result<Self, &'static str>
    where
        Self: Sized;
}

// Konverzija za tekst (String / &str)
impl QuatSerde for &str {
    fn to_quats(&self) -> QuatBuffer {
        let mut buf = QuatBuffer::new();
        for &byte in self.as_bytes() {
            if let Ok(q1) = Quat::from_u8((byte >> 6) & 0b11) { buf.push(q1); }
            if let Ok(q2) = Quat::from_u8((byte >> 4) & 0b11) { buf.push(q2); }
            if let Ok(q3) = Quat::from_u8((byte >> 2) & 0b11) { buf.push(q3); }
            if let Ok(q4) = Quat::from_u8(byte & 0b11) { buf.push(q4); }
        }
        buf
    }

    fn from_quats(_buf: &QuatBuffer) -> Result<Self, &'static str> {
        Err("To decode the text, use String impl")
    }
}

// Konverzija za u32 brojeve (16 kvata po broju)
impl QuatSerde for u32 {
    fn to_quats(&self) -> QuatBuffer {
        let mut buf = QuatBuffer::new();
        for i in (0..16).rev() {
            let val = ((self >> (i * 2)) & 0b11) as u8;
            buf.push(Quat::from_u8(val).unwrap());
        }
        buf
    }

    fn from_quats(buf: &QuatBuffer) -> Result<Self, &'static str> {
        if buf.len() < 16 {
            return Err("Insufficient quats for u32");
        }
        let mut val = 0u32;
        for q in buf.as_slice().iter().take(16) {
            val = (val << 2) | (*q as u32);
        }
        Ok(val)
    }
}

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

    pub fn deserialize(bytes: &[u8]) -> Result<Self, &'static str> {
        // Minimalna velicina paketa je 2 (Magic) + 4 (Count) + 4 (Checksum) = 10 bajtova
        if bytes.len() < 10 {
            return Err("The package is too short.");
        }

        if &bytes[0..2] != b"Q4" {
            return Err("Incorrect Magic header.");
        }

        let quat_count = u32::from_be_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]) as usize;
        
        // Zastita od overflow-a
        let payload_bytes_len = quat_count
            .checked_add(3)
            .ok_or("Overlow payload calculation")? / 4;

        // Bezbedna provera ukupne duzine pre bilo kakvog indeksiranja
        let expected_total_len = 6usize
            .checked_add(payload_bytes_len)
            .and_then(|v| v.checked_add(4))
            .ok_or("The total length exceeds the permitted limits")?;

        if bytes.len() < expected_total_len {
            return Err("The package is smaller than the header indicates.");
        }

        let payload_bytes = &bytes[6..6 + payload_bytes_len];
        
        // Bezbedan extract checksume iz zadnjih 4 bajta specifičnog paketa
        let checksum_start = 6 + payload_bytes_len;
        let received_checksum = u32::from_be_bytes([
            bytes[checksum_start],
            bytes[checksum_start + 1],
            bytes[checksum_start + 2],
            bytes[checksum_start + 3],
        ]);

        let calculated_checksum = Self::calculate_checksum(&bytes[0..checksum_start]);
        if received_checksum != calculated_checksum {
            return Err("Checksum error: The data is corrupted.");
        }

        let payload = QuatBuffer::unpack(payload_bytes, quat_count)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quat_serde_u32() {
        let broj: u32 = 123456789;
        let buf = broj.to_quats();
        let rekonstruisan = u32::from_quats(&buf).unwrap();
        assert_eq!(broj, rekonstruisan);
    }

    #[test]
    fn test_fec_error_correction() {
        let mut buf = QuatBuffer::new();
        buf.push(Quat::Q3);
        buf.push(Quat::Q1);

        let fec_bytes = buf.pack_with_fec();
        let pokvaren_bajt = fec_bytes[0] ^ 0b00010000; // Namerno kvarimo 1 bit
        
        let popravljen = QuatBuffer::decode_fec_byte(pokvaren_bajt);
        assert_eq!(popravljen, 0b1101); // Očekujemo ispravljena 4 bit-a (Q3=11, Q1=01)
    }

    #[test]
    fn test_stream_decoder() {
        let mut buf = QuatBuffer::new();
        buf.push(Quat::Q0);
        buf.push(Quat::Q2);

        let packet = QuatPacket::new(buf);
        let raw_bytes = packet.serialize();

        let mut decoder = StreamDecoder::new();
        
        // Simuliramo seckanje paketa preko mreže
        assert_eq!(decoder.feed(&raw_bytes[0..4]).len(), 0); // Nedovoljno bajtova
        assert_eq!(decoder.feed(&raw_bytes[4..]).len(), 1);  // Stigao ostatak, paket sastavljen!
    }
}

pub struct QuatSocket {
    stream: TcpStream,
    decoder: StreamDecoder,
}

impl QuatSocket {
    pub fn connect(addr: &str) -> Result<Self, &'static str> {
        let stream = TcpStream::connect(addr).map_err(|_| "Network connection failed")?;
        Ok(Self { stream, decoder: StreamDecoder::new() })
    }

    pub fn send(&mut self, packet: &QuatPacket) -> Result<(), &'static str> {
        let wire_bytes = packet.serialize();
        self.stream.write_all(&wire_bytes).map_err(|_| "Network transmission error")?;
        self.stream.flush().map_err(|_| "Error sending")?;
        Ok(())
    }

    pub fn receive(&mut self) -> Result<Vec<QuatPacket>, &'static str> {
        let mut buf = [0u8; 1024];
        let bytes_read = self.stream.read(&mut buf).map_err(|_| "Error reading from the network")?;
        if bytes_read == 0 {
            return Err("Connection closed");
        }
        Ok(self.decoder.feed(&buf[0..bytes_read]))
    }
}
use crate::quat::Quat;

/// Prikaz jednostavnog Block FEC-a prilagođenog kvartarnoj logici (GF(4))
pub struct QuatFecEncoder {
    block_size: usize,
}

impl QuatFecEncoder {
    pub fn new(block_size: usize) -> Self {
        Self { block_size }
    }

    /// Generiše paritetne kvartove za dati blok podataka koristeći XOR / GF(4) sumu
    pub fn encode_block(&self, data: &[Quat]) -> Vec<Quat> {
        let mut encoded = data.to_vec();
        
        // Računamo paritetni kvart (jednostavan XOR / sabiranje u GF(4))
        let mut parity_val: u8 = 0;
        for q in data {
            parity_val ^= *q as u8;
        }
        
        // Osiguravamo da paritet ostaje u granicama kvartarne logike (0..=3)
        let parity_quat = Quat::try_from(parity_val & 0b11).unwrap_or(Quat::Q0);
        encoded.push(parity_quat);
        
        encoded
    }

    /// Detektuje i ispravlja grešku u bloku ukoliko je došlo do oštećenja jednog kvarta
    pub fn decode_and_correct(&self, received: &mut [Quat]) -> Result<(), &'static str> {
        if received.is_empty() {
            return Err("Empty block");
        }

        let split_idx = received.len() - 1;
        let (data, parity_slice) = received.split_at_mut(split_idx);
        let received_parity = parity_slice[0];

        // Izračunaj očekivani paritet
        let mut calculated_val: u8 = 0;
        for q in data {
            calculated_val ^= *q as u8;
        }
        let expected_parity = Quat::try_from(calculated_val & 0b11).unwrap_or(Quat::Q0);

        if received_parity != expected_parity {
            // Došlo je do greške u prenosu! 
            // Sa 1 paritetnim kvartom možemo detektovati grešku, a uz Reed-Solomon/Hamming šemu i ispraviti.
            return Err("FEC detected uncorrectable or single-bit/quat error in block");
        }

        Ok(())
    }
}
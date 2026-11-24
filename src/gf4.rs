use crate::quat::{Quat, QuatBuffer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gf4(pub Quat);

impl Gf4 {
    #[inline]
    pub const fn new(q: Quat) -> Self {
        Self(q)
    }

    #[inline]
    pub const fn from_bits(bits: u8) -> Self {
        match bits & 0b11 {
            0 => Self(Quat::Q0),
            1 => Self(Quat::Q1),
            2 => Self(Quat::Q2),
            _ => Self(Quat::Q3),
        }
    }

    #[inline]
    pub const fn bits(self) -> u8 {
        self.0 as u8
    }

    /// Addition in GF(4) is bitwise XOR over the 2-bit representation.
    #[inline]
    pub const fn add(self, rhs: Self) -> Self {
        Self::from_bits(self.bits() ^ rhs.bits())
    }

    /// Multiplication in GF(4) = GF(2)[x] / (x^2 + x + 1), where x^2 = x + 1.
    #[inline]
    pub const fn mul(self, rhs: Self) -> Self {
        let a = self.bits();
        let b = rhs.bits();

        let a1 = (a >> 1) & 1;
        let a0 = a & 1;
        let b1 = (b >> 1) & 1;
        let b0 = b & 1;

        let x2_coeff = a1 & b1;
        let x1_coeff = (a1 & b0) ^ (a0 & b1) ^ x2_coeff;
        let x0_coeff = (a0 & b0) ^ x2_coeff;

        Self::from_bits((x1_coeff << 1) | x0_coeff)
    }

    /// Multiplicative inverse in GF(4) for non-zero elements (Q0 has no inverse).
    #[inline]
    pub const fn inv(self) -> Option<Self> {
        match self.bits() {
            0 => None,
            1 => Some(Self(Quat::Q1)),
            2 => Some(Self(Quat::Q3)),
            _ => Some(Self(Quat::Q2)),
        }
    }
}

/// Computes a GF(4) parity quat across a slice of Quats.
pub fn gf4_parity(quats: &[Quat]) -> Quat {
    let acc = quats
        .iter()
        .fold(Gf4::new(Quat::Q0), |state, &q| state.add(Gf4::new(q)));
    acc.0
}

/// Linear combination in GF(4): sum_i (weights[i] * data[i]).
pub fn gf4_linear_combine(data: &[Quat], weights: &[Gf4]) -> Quat {
    let mut acc = Gf4::new(Quat::Q0);
    for (q, &w) in data.iter().zip(weights.iter()) {
        acc = acc.add(Gf4::new(*q).mul(w));
    }
    acc.0
}

/// Encodes a QuatBuffer with an appended GF(4) parity quat.
pub fn encode_gf4_block(buf: &QuatBuffer) -> QuatBuffer {
    let mut out = QuatBuffer::with_capacity(buf.quats.len() + 1);
    out.quats.extend_from_slice(&buf.quats);
    out.quats.push(gf4_parity(&buf.quats));
    out
}
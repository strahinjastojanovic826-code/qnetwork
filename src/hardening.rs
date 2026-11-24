use crate::errors::NetworkError;

pub const MAX_SAFE_QUAT_COUNT: usize = 1_048_576; // 1M quats (~256 KB packed)
pub const MAX_SAFE_FRAME_BYTES: usize = 4 * 1024 * 1024; // 4 MiB

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckedFrameHeader<'a> {
    pub quat_count: usize,
    pub payload_bytes: &'a [u8],
    pub checksum: u32,
}

/// Safely parses a `Q4` wire frame without panicking on truncated or malicious input.
pub fn parse_frame_safe<'a>(bytes: &'a [u8]) -> Result<CheckedFrameHeader<'a>, NetworkError> {
    if bytes.len() < 10 {
        return Err(NetworkError::PackageTooShort);
    }
    if bytes.len() > MAX_SAFE_FRAME_BYTES {
        return Err(NetworkError::TotalLengthExceedsLimits);
    }
    if &bytes[0..2] != b"Q4" {
        return Err(NetworkError::IncorrectMagicHeader);
    }

    let count_bytes: [u8; 4] = bytes[2..6]
        .try_into()
        .map_err(|_| NetworkError::PackageTooShort)?;
    let quat_count = u32::from_be_bytes(count_bytes) as usize;

    if quat_count > MAX_SAFE_QUAT_COUNT {
        return Err(NetworkError::TotalLengthExceedsLimits);
    }

    let payload_len = quat_count
        .checked_add(3)
        .ok_or(NetworkError::OverflowPayloadCalculation)?
        / 4;

    let payload_end = 6usize
        .checked_add(payload_len)
        .ok_or(NetworkError::TotalLengthExceedsLimits)?;

    let frame_end = payload_end
        .checked_add(4)
        .ok_or(NetworkError::TotalLengthExceedsLimits)?;

    if frame_end > MAX_SAFE_FRAME_BYTES {
        return Err(NetworkError::TotalLengthExceedsLimits);
    }
    if bytes.len() < frame_end {
        return Err(NetworkError::PackageSmallerThanHeader);
    }

    let csum_bytes: [u8; 4] = bytes[payload_end..frame_end]
        .try_into()
        .map_err(|_| NetworkError::PackageTooShort)?;
    let checksum = u32::from_be_bytes(csum_bytes);

    Ok(CheckedFrameHeader {
        quat_count,
        payload_bytes: &bytes[6..payload_end],
        checksum,
    })
}
//! # Safety Contract for Legacy FFI Functions
//! Callers passing raw pointers (`ptr`, `out_buf`, `out_len`) into `quat_buffer_*`
//! functions in this module must ensure they are non-null (where required), properly
//! aligned, valid for the expected lifetime/size, not aliased by conflicting mutable
//! references, and freed once via `quat_buffer_free`. The static string returned by
//! `qnetwork_version` must not be freed.

use crate::quat::{Quat, QuatBuffer};
use std::convert::TryFrom;
use std::os::raw::c_char;
use crate::errors::NetworkError;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::slice;

// Generiramo C-kompatibilne pokazivače/strukture
#[repr(C)]
pub struct CQuatBuffer {
    pub ptr: *mut u8,
    pub len: usize,
}

#[unsafe(no_mangle)]
pub extern "C" fn quat_buffer_create() -> *mut QuatBuffer {
    Box::into_raw(Box::new(QuatBuffer::new()))
}

#[unsafe(no_mangle)]
pub extern "C" fn quat_buffer_free(ptr: *mut QuatBuffer) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(ptr);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn quat_buffer_push(ptr: *mut QuatBuffer, val: u8) -> i32 {
    if ptr.is_null() {
        return -1;
    }
    let buffer = unsafe { &mut *ptr };
    if let Ok(quat) = Quat::try_from(val) {
        buffer.push(quat);
        0 // Uspeh
    } else {
        -2 // Neispravan quat
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn quat_buffer_pack(
    ptr: *const QuatBuffer,
    out_buf: *mut u8,
    out_capacity: usize,
    out_len: *mut usize,
) -> i32 {
    if ptr.is_null() || out_buf.is_null() || out_len.is_null() {
        return -1;
    }

    let buffer = unsafe { &*ptr };
    let packed = buffer.pack();

    if packed.len() > out_capacity {
        return -3; // Premali izlazni međuspremnik
    }

    unsafe {
        std::ptr::copy_nonoverlapping(packed.as_ptr(), out_buf, packed.len());
        *out_len = packed.len();
    }

    0
}

/// Helper function to check library version or status from Python/C
#[unsafe(no_mangle)]
pub extern "C" fn qnetwork_version() -> *const c_char {
    // Vraća string ka verziji biblioteke bezbedan za C
    b"1.0.0\0".as_ptr() as *const c_char
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QNetStatus {
    Ok = 0,
    NullPointer = -1,
    InvalidQuat = -2,
    BufferTooSmall = -3,
    InvalidStream = -4,
    LengthOverflow = -5,
    InternalPanic = -99,
}

impl From<NetworkError> for QNetStatus {
    fn from(err: NetworkError) -> Self {
        match err {
            NetworkError::InsufficientQuats => Self::InvalidQuat,
            NetworkError::InvalidRleStream => Self::InvalidStream,
            NetworkError::OverflowPayloadCalculation
            | NetworkError::TotalLengthExceedsLimits => Self::LengthOverflow,
            NetworkError::PackageSmallerThanHeader => Self::BufferTooSmall,
            _ => Self::InvalidStream,
        }
    }
}

#[repr(C)]
pub struct QNetQuatHandle {
    inner: QuatBuffer,
}

/// Allocates a new quaternary buffer handle on the heap. Returns null on panic.
#[unsafe(no_mangle)]
pub extern "C" fn qnet_handle_new() -> *mut QNetQuatHandle {
    catch_unwind(|| {
        Box::into_raw(Box::new(QNetQuatHandle {
            inner: QuatBuffer::new(),
        }))
    })
    .unwrap_or(std::ptr::null_mut())
}

/// Frees a handle previously created by `qnet_handle_new`. Safe on null pointer.
///
/// # Safety
/// `handle` must either be null or a valid pointer returned by `qnet_handle_new`
/// that has not already been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn qnet_handle_free(handle: *mut QNetQuatHandle) {
    if handle.is_null() {
        return;
    }
    let _ = catch_unwind(AssertUnwindSafe(|| unsafe {
        drop(Box::from_raw(handle));
    }));
}

/// Pushes a single quaternary value (`0..=3`) into the buffer.
///
/// # Safety
/// `handle` must point to a live `QNetQuatHandle` allocated by `qnet_handle_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn qnet_handle_push_quat(handle: *mut QNetQuatHandle, val: u8) -> QNetStatus {
    if handle.is_null() {
        return QNetStatus::NullPointer;
    }
    catch_unwind(AssertUnwindSafe(|| {
        let quat = match Quat::try_from(val) {
            Ok(q) => q,
            Err(_) => return QNetStatus::InvalidQuat,
        };
        let h = unsafe { &mut *handle };
        h.inner.push(quat);
        QNetStatus::Ok
    }))
    .unwrap_or(QNetStatus::InternalPanic)
}

/// Packs quats (4 quats = 1 byte, 75% wire size reduction vs 1 byte per quat) into `out_buf`.
/// Writes required or written byte count to `out_written`.
///
/// # Safety
/// `handle`, `out_buf`, and `out_written` must be valid pointers, and `out_buf`
/// must have capacity of at least `out_capacity` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn qnet_handle_pack(
    handle: *const QNetQuatHandle,
    out_buf: *mut u8,
    out_capacity: usize,
    out_written: *mut usize,
) -> QNetStatus {
    if handle.is_null() || out_buf.is_null() || out_written.is_null() {
        return QNetStatus::NullPointer;
    }
    catch_unwind(AssertUnwindSafe(|| {
        let h = unsafe { &*handle };
        let packed = h.inner.pack();
        if packed.len() > out_capacity {
            unsafe { *out_written = packed.len() };
            return QNetStatus::BufferTooSmall;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(packed.as_ptr(), out_buf, packed.len());
            *out_written = packed.len();
        }
        QNetStatus::Ok
    }))
    .unwrap_or(QNetStatus::InternalPanic)
}

/// Unpacks `packed_len` bytes into `total_quats` quaternary values (`0..=3`).
///
/// # Safety
/// `handle` must point to a live `QNetQuatHandle`, and `packed_ptr` must be valid
/// for reads of `packed_len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn qnet_handle_unpack(
    handle: *mut QNetQuatHandle,
    packed_ptr: *const u8,
    packed_len: usize,
    total_quats: usize,
) -> QNetStatus {
    if handle.is_null() || (packed_len > 0 && packed_ptr.is_null()) {
        return QNetStatus::NullPointer;
    }
    catch_unwind(AssertUnwindSafe(|| {
        let slice = if packed_len == 0 {
            &[]
        } else {
            unsafe { slice::from_raw_parts(packed_ptr, packed_len) }
        };
        match QuatBuffer::unpack(slice, total_quats) {
            Ok(buf) => {
                let h = unsafe { &mut *handle };
                h.inner = buf;
                QNetStatus::Ok
            }
            Err(e) => QNetStatus::from(e),
        }
    }))
    .unwrap_or(QNetStatus::InternalPanic)
}
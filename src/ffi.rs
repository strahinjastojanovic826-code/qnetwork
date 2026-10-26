use crate::quat::{Quat, QuatBuffer};
use std::convert::TryFrom;
use std::os::raw::c_char;

// Generiramo C-kompatibilne pokazivače/strukture
#[repr(C)]
pub struct CQuatBuffer {
    pub ptr: *mut u8,
    pub len: usize,
}

#[no_mangle]
pub extern "C" fn quat_buffer_create() -> *mut QuatBuffer {
    Box::into_raw(Box::new(QuatBuffer::new()))
}

#[no_mangle]
pub extern "C" fn quat_buffer_free(ptr: *mut QuatBuffer) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(ptr);
        }
    }
}

#[no_mangle]
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

#[no_mangle]
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
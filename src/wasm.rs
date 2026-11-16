//! Interfaz WebAssembly "a mano" (sin wasm-bindgen): cinco funciones exportadas.
//!
//! Protocolo desde JavaScript:
//!   1. `ptr = mpp_alloc(n)` y copiar los bytes del .mpp en la memoria del modulo.
//!   2. `estado = mpp_convert(ptr, n, modo)`  (modo 0 = xlsx, 1 = json, 2 = json completo).
//!   3. Leer `mpp_result_len()` bytes desde `mpp_result_ptr()`:
//!      con estado 0 es el resultado; con estado 1 es el mensaje de error en UTF-8.
//!   4. `mpp_free(ptr, n)`.

use std::alloc::{alloc, dealloc, Layout};

static mut RESULT: Vec<u8> = Vec::new();

#[no_mangle]
pub extern "C" fn mpp_alloc(len: usize) -> *mut u8 {
    if len == 0 {
        return std::ptr::null_mut();
    }
    match Layout::from_size_align(len, 1) {
        Ok(layout) => unsafe { alloc(layout) },
        Err(_) => std::ptr::null_mut(),
    }
}

/// # Safety
/// `ptr` debe venir de `mpp_alloc(len)` y no haberse liberado antes.
#[no_mangle]
pub unsafe extern "C" fn mpp_free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len != 0 {
        if let Ok(layout) = Layout::from_size_align(len, 1) {
            dealloc(ptr, layout);
        }
    }
}

/// # Safety
/// `ptr` debe apuntar a `len` bytes validos dentro de la memoria del modulo.
#[no_mangle]
pub unsafe extern "C" fn mpp_convert(ptr: *const u8, len: usize, mode: u32) -> u32 {
    let input: &[u8] = if ptr.is_null() || len == 0 { &[] } else { std::slice::from_raw_parts(ptr, len) };
    let result = crate::read_mpp(input).map(|p| match mode {
        0 => crate::xlsx::workbook(&p),
        1 => crate::json::project_json(&p, false).into_bytes(),
        _ => crate::json::project_json(&p, true).into_bytes(),
    });
    let (status, bytes) = match result {
        Ok(b) => (0, b),
        Err(e) => (1, e.into_bytes()),
    };
    // WebAssembly aqui es de un solo hilo: no hay acceso concurrente a RESULT.
    *std::ptr::addr_of_mut!(RESULT) = bytes;
    status
}

#[no_mangle]
pub extern "C" fn mpp_result_ptr() -> *const u8 {
    unsafe { (*std::ptr::addr_of!(RESULT)).as_ptr() }
}

#[no_mangle]
pub extern "C" fn mpp_result_len() -> usize {
    unsafe { (*std::ptr::addr_of!(RESULT)).len() }
}

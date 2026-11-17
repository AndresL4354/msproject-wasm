//! Interfaz WebAssembly "a mano" (sin wasm-bindgen), con el mismo protocolo que mpp2xlsx:
//!   1. `ptr = doc_alloc(n)` y copiar los bytes del documento en la memoria del modulo.
//!   2. `estado = doc_convert(ptr, n, modo)`. El byte bajo de `modo` es la operacion y los bits
//!      8 a 31, su parametro (`operacion | (parametro << 8)`); es un reparto comun a todas las
//!      operaciones, tambien las que este modulo aun no implementa:
//!
//!      | operacion | salida                          | parametro                              |
//!      |-----------|---------------------------------|----------------------------------------|
//!      | 0         | JSON                            | (no usa)                               |
//!      | 1         | JSON completo                   | (no usa)                               |
//!      | 2         | Markdown                        | (reservada; aqui da error)             |
//!      | 3         | fragmentos en JSON              | (reservada; aqui da error)             |
//!      | 4         | JSON con las imagenes           | lado minimo: omite los bytes de las    |
//!      | 5         | JSON completo con las imagenes  | imagenes con un lado menor a eso       |
//!
//!      Cualquier otra operacion da el error "Modo de conversion desconocido.".
//!   3. Leer `doc_result_len()` bytes desde `doc_result_ptr()`:
//!      con estado 0 es el JSON en UTF-8 o, con imagenes, `[largo del JSON: u32 LE][JSON][bytes de
//!      las imagenes]` (cada imagen del JSON trae su `offset` y `length` en ese bloque final);
//!      con estado 1 es el mensaje de error en UTF-8.
//!   4. `doc_free(ptr, n)`.

use std::alloc::{alloc, dealloc, Layout};

static mut RESULT: Vec<u8> = Vec::new();

#[no_mangle]
pub extern "C" fn doc_alloc(len: usize) -> *mut u8 {
    if len == 0 {
        return std::ptr::null_mut();
    }
    match Layout::from_size_align(len, 1) {
        Ok(layout) => unsafe { alloc(layout) },
        Err(_) => std::ptr::null_mut(),
    }
}

/// # Safety
/// `ptr` debe venir de `doc_alloc(len)` y no haberse liberado antes.
#[no_mangle]
pub unsafe extern "C" fn doc_free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len != 0 {
        if let Ok(layout) = Layout::from_size_align(len, 1) {
            dealloc(ptr, layout);
        }
    }
}

/// # Safety
/// `ptr` debe apuntar a `len` bytes validos dentro de la memoria del modulo.
#[no_mangle]
pub unsafe extern "C" fn doc_convert(ptr: *const u8, len: usize, mode: u32) -> u32 {
    let input: &[u8] = if ptr.is_null() || len == 0 { &[] } else { std::slice::from_raw_parts(ptr, len) };
    // Byte bajo: operacion; bits 8 a 31: su parametro (ver el encabezado de este archivo).
    let (op, param) = (mode & 0xFF, mode >> 8);
    let result = match op {
        0 | 1 => crate::to_json(input, op == 1).map(String::into_bytes),
        4 | 5 => crate::read_with_images(input).map(|doc| {
            let (json, blob) = crate::doc::to_json_with_images(&doc, op == 5, param);
            let mut out = Vec::with_capacity(4 + json.len() + blob.len());
            out.extend_from_slice(&(json.len() as u32).to_le_bytes());
            out.extend_from_slice(json.as_bytes());
            out.extend_from_slice(&blob);
            out
        }),
        _ => Err("Modo de conversion desconocido.".to_string()),
    };
    let (status, bytes) = match result {
        Ok(b) => (0, b),
        Err(e) => (1, e.into_bytes()),
    };
    // WebAssembly aqui es de un solo hilo: no hay acceso concurrente a RESULT.
    *std::ptr::addr_of_mut!(RESULT) = bytes;
    status
}

#[no_mangle]
pub extern "C" fn doc_result_ptr() -> *const u8 {
    unsafe { (*std::ptr::addr_of!(RESULT)).as_ptr() }
}

#[no_mangle]
pub extern "C" fn doc_result_len() -> usize {
    unsafe { (*std::ptr::addr_of!(RESULT)).len() }
}

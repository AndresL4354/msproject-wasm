//! Interfaz WebAssembly "a mano" (sin wasm-bindgen), con el mismo protocolo que mpp2xlsx:
//!   1. `ptr = doc_alloc(n)` y copiar los bytes del documento en la memoria del modulo.
//!   2. `estado = doc_convert(ptr, n, modo, mb, paginas, caracteres)`.
//!      - `modo`: el byte bajo es la operacion y los bits 8 a 31 su parametro
//!        (`operacion | (parametro << 8)`):
//!
//!      | operacion | salida                     | parametro                               |
//!      |-----------|----------------------------|-----------------------------------------|
//!      | 0         | JSON                       | -                                       |
//!      | 1         | JSON completo              | -                                       |
//!      | 2         | Markdown                   | -                                       |
//!      | 3         | fragmentos en JSON         | tamano maximo en caracteres (0 = 2000)  |
//!      | 4         | JSON con imagenes          | lado minimo de las imagenes             |
//!      | 5         | JSON completo con imagenes | lado minimo de las imagenes             |
//!
//!      Este modulo implementa 0 a 3; 4 y 5 quedan reservadas para la extraccion de imagenes.
//!      Una operacion que el modulo no conoce da error ("Modo de conversion desconocido."), nunca
//!      otra salida.
//!      - `mb`, `paginas`, `caracteres`: limites de recursos de esta conversion (datos
//!        descomprimidos en MB, paginas de PDF, caracteres de texto); 0 = valor por defecto. Van
//!        en cada llamada y no quedan para la siguiente. Quien llama con solo tres argumentos (un
//!        cargador anterior) recibe ceros: los valores por defecto.
//!   3. Leer `doc_result_len()` bytes desde `doc_result_ptr()`:
//!      con estado 0 es el JSON; con estado 1 es el mensaje de error, ambos en UTF-8.
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
pub unsafe extern "C" fn doc_convert(ptr: *const u8, len: usize, mode: u32, mb: u32, pages: u32, chars: u32) -> u32 {
    let input: &[u8] = if ptr.is_null() || len == 0 { &[] } else { std::slice::from_raw_parts(ptr, len) };
    let limits = crate::budget::Limits::from_units(mb, pages, chars);
    let read = || crate::read_with_limits(input, limits);
    let result = match mode & 0xff {
        op @ (0 | 1) => read().map(|d| crate::doc::to_json(&d, op == 1)),
        2 => read().map(|d| crate::markdown::to_markdown(&d)),
        3 => read().map(|d| {
            let size = match (mode >> 8) as usize {
                0 => crate::markdown::DEFAULT_CHUNK,
                n => n,
            };
            crate::markdown::chunks_json(&d, &crate::markdown::chunks(&d, size))
        }),
        // 4 y 5 (imagenes) y cualquier otra: no se adivina otra salida.
        _ => Err("Modo de conversion desconocido.".to_string()),
    };
    let (status, bytes) = match result {
        Ok(s) => (0, s.into_bytes()),
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

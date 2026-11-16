//! mpp2xlsx: convierte archivos de Microsoft Project (.mpp) a Excel (.xlsx) y JSON.
//!
//! - Sin dependencias externas: lector OLE2, lector MPP, escritor XLSX y ZIP propios.
//! - Funciona igual como binario nativo y como WebAssembly (navegador o funcion serverless).
//!
//! ```no_run
//! let bytes = std::fs::read("plan.mpp").unwrap();
//! let xlsx = mpp2xlsx::mpp_to_xlsx(&bytes).unwrap();
//! std::fs::write("plan.xlsx", xlsx).unwrap();
//! ```

pub mod cfb;
pub mod fasthash;
pub mod json;
pub mod model;
pub mod mpp;
pub mod schema;
#[cfg(target_arch = "wasm32")]
mod wasm;
pub mod xlsx;
mod zip;

pub use model::Project;

/// Lee un .mpp y devuelve el modelo del proyecto.
pub fn read_mpp(bytes: &[u8]) -> Result<Project, String> {
    mpp::read(bytes)
}

/// Convierte un .mpp en un libro de Excel (.xlsx).
pub fn mpp_to_xlsx(bytes: &[u8]) -> Result<Vec<u8>, String> {
    Ok(xlsx::workbook(&read_mpp(bytes)?))
}

/// Convierte un .mpp en JSON (tareas, recursos, asignaciones y propiedades).
pub fn mpp_to_json(bytes: &[u8]) -> Result<String, String> {
    Ok(json::project_json(&read_mpp(bytes)?, false))
}

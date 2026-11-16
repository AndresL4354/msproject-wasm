//! doc2json: convierte documentos Word (.docx), PDF, PowerPoint (.pptx) y Excel (.xlsx) a JSON.
//!
//! - Sin dependencias externas: descompresor, lector ZIP, lector XML y lector PDF propios.
//! - Funciona igual como binario nativo y como WebAssembly (navegador o funcion serverless).
//! - Todos los formatos salen con la misma forma: metadatos, bloques de contenido y texto plano.
//!
//! ```no_run
//! let bytes = std::fs::read("informe.pdf").unwrap();
//! let json = doc2json::to_json(&bytes, false).unwrap();
//! std::fs::write("informe.json", json).unwrap();
//! ```

pub mod doc;
pub mod docx;
pub mod inflate;
pub mod json;
pub mod pdf;
pub mod pptx;
pub mod symbols;
#[cfg(target_arch = "wasm32")]
mod wasm;
pub mod xlsx;
pub mod xml;
pub mod zipread;

pub use doc::Document;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Format {
    Docx,
    Pdf,
    Pptx,
    Xlsx,
}

/// Reconoce el formato por el contenido, no por la extension.
pub fn detect(bytes: &[u8]) -> Result<Format, String> {
    let head = &bytes[..bytes.len().min(1024)];
    if head.windows(5).any(|w| w == b"%PDF-") {
        return Ok(Format::Pdf);
    }
    if bytes.starts_with(b"PK\x03\x04") || bytes.starts_with(b"PK\x05\x06") {
        return Ok(office_kind(bytes));
    }
    if bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]) {
        return Err("Es un documento de Office antiguo (.doc, .xls o .ppt) o protegido con contrasena. Abralo y guardelo sin contrasena como .docx, .xlsx o .pptx.".into());
    }
    if bytes.is_empty() {
        return Err("El archivo esta vacio.".into());
    }
    Err("Formato no reconocido: se esperaba un .docx, .pdf, .pptx o .xlsx.".into())
}

/// Tipo de paquete Office segun la parte principal que declara (o, si no la declara, segun sus
/// carpetas). Un ZIP que no es de Office se trata como Word, que dara el error adecuado.
fn office_kind(bytes: &[u8]) -> Format {
    let Ok(zip) = zipread::ZipArchive::open(bytes) else {
        // ZIP danado o incompleto: los nombres de las partes estan en claro en cada entrada, asi
        // el error puede hablar del formato correcto.
        let has = |name: &[u8]| bytes.windows(name.len()).any(|w| w == name);
        return if has(b"ppt/presentation.xml") {
            Format::Pptx
        } else if has(b"xl/workbook.xml") {
            Format::Xlsx
        } else {
            Format::Docx
        };
    };
    let main = docx::relationships(&zip, "_rels/.rels").into_iter().find(|r| r.1 == "officeDocument").map(|r| docx::resolve("", &r.2));
    let by_path = |p: &str| {
        if p.starts_with("ppt/") {
            Some(Format::Pptx)
        } else if p.starts_with("xl/") {
            Some(Format::Xlsx)
        } else if p.starts_with("word/") {
            Some(Format::Docx)
        } else {
            None
        }
    };
    if let Some(f) = main.as_deref().and_then(by_path) {
        return f;
    }
    if zip.find("ppt/presentation.xml").is_some() {
        Format::Pptx
    } else if zip.find("xl/workbook.xml").is_some() {
        Format::Xlsx
    } else {
        Format::Docx
    }
}

/// Lee un documento Word, PDF, PowerPoint o Excel.
pub fn read(bytes: &[u8]) -> Result<Document, String> {
    match detect(bytes)? {
        Format::Docx => docx::read(bytes),
        Format::Pdf => pdf::read(bytes),
        Format::Pptx => pptx::read(bytes),
        Format::Xlsx => xlsx::read(bytes),
    }
}

/// Convierte un documento a JSON. Con `full` se incluyen estilos, formato de los tramos de
/// texto y, en PDF, las lineas de cada pagina con su posicion.
pub fn to_json(bytes: &[u8], full: bool) -> Result<String, String> {
    Ok(doc::to_json(&read(bytes)?, full))
}

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

pub mod budget;
pub mod doc;
pub mod docx;
pub mod inflate;
pub mod json;
pub mod markdown;
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
    let zip = match zipread::ZipArchive::open(bytes) {
        Ok(zip) => zip,
        Err(_) => {
            // ZIP danado o incompleto: los nombres de las partes estan en claro en cada entrada,
            // asi el error puede hablar del formato correcto.
            let has = |name: &[u8]| bytes.windows(name.len()).any(|w| w == name);
            return if has(b"ppt/presentation.xml") {
                Format::Pptx
            } else if has(b"xl/workbook.xml") {
                Format::Xlsx
            } else {
                Format::Docx
            };
        }
    };
    // La parte principal declarada cuenta solo si existe: un paquete con una relacion rota sigue
    // reconociendose por sus carpetas, como lo hace cada lector.
    let main = docx::relationships(&zip, "_rels/.rels", docx::MAX_TARGET)
        .into_iter()
        .find(|r| r.1 == "officeDocument")
        .map(|r| docx::resolve("", &r.2))
        .filter(|p| zip.find(p).is_some());
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
    if zip.find("word/document.xml").is_some() {
        Format::Docx
    } else if zip.find("ppt/presentation.xml").is_some() {
        Format::Pptx
    } else if zip.find("xl/workbook.xml").is_some() {
        Format::Xlsx
    } else {
        Format::Docx
    }
}

/// Lee un documento Word, PDF, PowerPoint o Excel con los limites de recursos por defecto
/// (`budget::Limits`).
pub fn read(bytes: &[u8]) -> Result<Document, String> {
    read_with_limits(bytes, budget::Limits::default())
}

/// Lee un documento Word, PDF, PowerPoint o Excel sin pasar los limites dados: datos
/// descomprimidos en total, paginas y caracteres de texto. Al alcanzar uno, entrega lo leido hasta
/// ahi con un aviso en `warnings`. Si el presupuesto de descompresion se agota antes de llegar al
/// contenido, falla con un error que lo dice (en vez del error generico del lector).
pub fn read_with_limits(bytes: &[u8], limits: budget::Limits) -> Result<Document, String> {
    // Reconocer un paquete Office ya descomprime sus relaciones: tambien cuenta en el presupuesto.
    budget::begin(limits);
    let result = detect(bytes).and_then(|format| match format {
        Format::Docx => docx::read(bytes),
        Format::Pdf => pdf::read(bytes),
        Format::Pptx => pptx::read(bytes),
        Format::Xlsx => xlsx::read(bytes),
    });
    let spent = budget::end();
    let mut d = match result {
        Ok(d) => d,
        Err(e) if spent.decompressed_hit => return Err(budget::decompressed_error(limits.max_decompressed, &e)),
        Err(e) => return Err(e),
    };
    if spent.decompressed_hit {
        d.warnings.push(budget::decompressed_warning(limits.max_decompressed));
    }
    if limit_text(&mut d, limits.max_chars) || spent.text_hit {
        d.warnings.push(format!("El documento tiene mas texto que el limite de {} caracteres: se entrega solo la primera parte.", limits.max_chars));
    }
    Ok(d)
}

/// Deja como maximo `max` caracteres de texto: primero el contenido, despues las partes
/// secundarias (encabezados, pies, notas, comentarios) y al final los metadatos de texto y los
/// marcadores del PDF, en ese orden. En los bloques cuentan tambien el texto alternativo de las
/// imagenes y la etiqueta de notas y comentarios (numero o autor). Las paginas de un PDF repiten
/// el texto del contenido, asi que se cuentan aparte, con su propio maximo. Devuelve si se
/// descarto algo.
fn limit_text(d: &mut Document, max: usize) -> bool {
    let mut left = max;
    let mut cut = limit_blocks(&mut d.content, &mut left);
    for (_, blocks) in d.extras.iter_mut() {
        cut |= limit_blocks(blocks, &mut left);
    }
    for (_, m) in d.meta.iter_mut() {
        if let doc::Meta::Text(s) = m {
            cut |= limit_str(s, &mut left);
        }
    }
    // Los metadatos y marcadores vacios no existian: son los que no cupieron.
    d.meta.retain(|(_, m)| !matches!(m, doc::Meta::Text(s) if s.is_empty()));
    for o in d.outline.iter_mut() {
        cut |= limit_str(&mut o.title, &mut left);
    }
    d.outline.retain(|o| !o.title.is_empty());
    let mut left = max;
    for p in d.pages.iter_mut() {
        let n = p.text.chars().count();
        if n > left {
            p.text = p.text.chars().take(left).collect();
            p.lines.clear();
            cut = true;
        }
        left = left.saturating_sub(n);
    }
    cut
}

/// Deja en `s` lo que quepa en `left` caracteres y lo descuenta. Devuelve si se descarto algo.
fn limit_str(s: &mut String, left: &mut usize) -> bool {
    let n = s.chars().count();
    if n <= *left {
        *left -= n;
        return false;
    }
    *s = s.chars().take(*left).collect();
    *left = 0;
    true
}

/// Caracteres de un bloque: su texto plano (con la numeracion delante), o el texto alternativo
/// si es una imagen, mas su etiqueta (numero de la nota o autor del comentario).
fn block_chars(b: &doc::Block) -> usize {
    let text = if b.kind == doc::Kind::Image { b.text.chars().count() } else { doc::block_text(b).chars().count() };
    text + b.label.as_deref().map_or(0, |l| l.chars().count())
}

/// Deja en `blocks` como maximo `left` caracteres (contados con `block_chars`) y descuenta lo
/// que queda. El bloque que cruza el limite se corta por dentro (la etiqueta, el texto o las
/// filas de una tabla) y los siguientes se descartan.
fn limit_blocks(blocks: &mut Vec<doc::Block>, left: &mut usize) -> bool {
    for i in 0..blocks.len() {
        let n = block_chars(&blocks[i]);
        if n <= *left {
            *left -= n;
            continue;
        }
        let mut room = std::mem::take(left);
        blocks.truncate(i + 1);
        let b = &mut blocks[i];
        // La etiqueta (el autor de un comentario) va antes que el texto.
        if let Some(l) = b.label.as_mut() {
            limit_str(l, &mut room);
        }
        let keep = if b.kind == doc::Kind::Table {
            let mut used = 0usize;
            let n = b.rows.iter().take_while(|r| {
                used += r.iter().map(|c| c.text.chars().count() + 1).sum::<usize>();
                used <= room
            });
            let n = n.count();
            b.rows.truncate(n);
            n > 0
        } else {
            // La numeracion o vineta ("1.1 ") tambien cuenta.
            let marker = b.marker.as_deref().filter(|m| !m.is_empty()).map_or(0, |m| m.chars().count() + 1);
            b.text = b.text.chars().take(room.saturating_sub(marker)).collect();
            b.runs.clear();
            // Una imagen queda aunque su texto alternativo no quepa: ella misma no es texto.
            b.kind == doc::Kind::Image || !b.text.is_empty()
        };
        if !keep {
            blocks.pop();
        }
        return true;
    }
    false
}

/// Convierte un documento a JSON. Con `full` se incluyen estilos, formato de los tramos de
/// texto y, en PDF, las lineas de cada pagina con su posicion.
pub fn to_json(bytes: &[u8], full: bool) -> Result<String, String> {
    Ok(doc::to_json(&read(bytes)?, full))
}

/// Convierte un documento a Markdown (tablas GFM, notas al pie, marcas de pagina en PDF).
pub fn to_markdown(bytes: &[u8]) -> Result<String, String> {
    Ok(markdown::to_markdown(&read(bytes)?))
}

/// Convierte un documento en fragmentos de hasta `max_chars` caracteres, en JSON, con la ruta
/// de titulos y las paginas de cada uno. `0` usa el tamano por defecto.
pub fn to_chunks_json(bytes: &[u8], max_chars: usize) -> Result<String, String> {
    let d = read(bytes)?;
    let size = if max_chars == 0 { markdown::DEFAULT_CHUNK } else { max_chars };
    Ok(markdown::chunks_json(&d, &markdown::chunks(&d, size)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc::{Block, Kind};

    fn item(marker: &str, text: &str) -> Block {
        Block { kind: Kind::ListItem, marker: Some(marker.into()), text: text.into(), ..Block::default() }
    }

    fn total(d: &Document) -> usize {
        d.content.iter().chain(d.extras.iter().flat_map(|(_, b)| b.iter())).map(|b| doc::block_text(b).chars().count()).sum()
    }

    #[test]
    fn el_limite_cuenta_la_numeracion_y_sigue_en_las_notas() {
        // "1.1 abcdef" son 10 caracteres; la nota, 5.
        let mut d = Document { content: vec![item("1.1", "abcdef")], extras: vec![("footnotes", vec![Block { text: "nota1".into(), ..Block::default() }])], ..Document::default() };
        assert!(!limit_text(&mut d, 15));
        assert!(limit_text(&mut d, 12));
        assert_eq!(total(&d), 12);
        assert_eq!(d.extras[0].1[0].text, "no");
        // Con 6 caracteres solo cabe "1.1 ab"; la nota se descarta.
        assert!(limit_text(&mut d, 6));
        assert_eq!((doc::block_text(&d.content[0]).as_str(), d.extras[0].1.len()), ("1.1 ab", 0));
        // Si no cabe ni la numeracion, el bloque se quita.
        assert!(limit_text(&mut d, 3));
        assert!(d.content.is_empty());
    }
}

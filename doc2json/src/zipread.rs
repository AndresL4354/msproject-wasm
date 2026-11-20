//! Lector ZIP minimo: directorio central y entradas almacenadas o comprimidas con DEFLATE.
//! Suficiente para abrir un .docx (que es un ZIP con archivos XML dentro).

use crate::inflate::inflate;
use std::collections::HashMap;

pub struct Entry {
    pub name: String,
    method: u16,
    packed: usize,
    pub size: usize,
    offset: usize,
}

pub struct ZipArchive<'a> {
    data: &'a [u8],
    pub entries: Vec<Entry>,
    /// Indice de cada nombre en minusculas (ASCII) a su primera entrada: `find` no recorre la
    /// lista, que en un archivo malicioso puede tener 65.535 entradas.
    index: HashMap<String, usize>,
}

/// Clave con que `find` busca una parte: sin barras iniciales y en minusculas (ASCII). Dos
/// rutas con la misma clave son la misma parte; sirve para no leer dos veces la misma.
pub fn part_key(name: &str) -> String {
    name.trim_start_matches('/').to_ascii_lowercase()
}

fn u16_at(d: &[u8], o: usize) -> usize {
    match d.get(o..o + 2) {
        Some(b) => u16::from_le_bytes([b[0], b[1]]) as usize,
        None => 0,
    }
}

fn u32_at(d: &[u8], o: usize) -> usize {
    match d.get(o..o + 4) {
        Some(b) => u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize,
        None => 0,
    }
}

impl<'a> ZipArchive<'a> {
    pub fn open(data: &'a [u8]) -> Result<ZipArchive<'a>, String> {
        // El registro de fin de directorio esta al final, seguido como mucho de un comentario.
        let min = data.len().saturating_sub(22 + 65535);
        let mut eocd = None;
        let mut i = data.len().saturating_sub(22);
        loop {
            if data.get(i..i + 4) == Some(&[0x50, 0x4B, 0x05, 0x06]) {
                eocd = Some(i);
                break;
            }
            if i == 0 || i <= min {
                break;
            }
            i -= 1;
        }
        let eocd = eocd.ok_or("El archivo no es un ZIP valido (no se encontro el directorio).")?;
        let count = u16_at(data, eocd + 10);
        let mut pos = u32_at(data, eocd + 16);
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            if data.get(pos..pos + 4) != Some(&[0x50, 0x4B, 0x01, 0x02]) {
                break;
            }
            let name_len = u16_at(data, pos + 28);
            let name = String::from_utf8_lossy(data.get(pos + 46..pos + 46 + name_len).unwrap_or(&[])).replace('\\', "/");
            entries.push(Entry {
                name,
                method: u16_at(data, pos + 10) as u16,
                packed: u32_at(data, pos + 20),
                size: u32_at(data, pos + 24),
                offset: u32_at(data, pos + 42),
            });
            pos += 46 + name_len + u16_at(data, pos + 30) + u16_at(data, pos + 32);
        }
        let mut index = HashMap::with_capacity(entries.len());
        for (i, e) in entries.iter().enumerate() {
            // Vale la primera entrada con ese nombre. Al nombre de la entrada no se le quitan las
            // barras iniciales: una entrada "/a.xml" no la encuentra ninguna busqueda.
            index.entry(e.name.to_ascii_lowercase()).or_insert(i);
        }
        Ok(ZipArchive { data, entries, index })
    }

    /// Entrada de una parte, sin distinguir mayusculas (ASCII) y sin las barras iniciales.
    pub fn find(&self, name: &str) -> Option<&Entry> {
        self.index.get(&part_key(name)).map(|&i| &self.entries[i])
    }

    /// Posicion en el archivo de los datos de una entrada (tras su encabezado local).
    fn data_start(&self, e: &Entry) -> Option<usize> {
        let o = e.offset;
        if self.data.get(o..o + 4) != Some(&[0x50, 0x4B, 0x03, 0x04]) {
            return None;
        }
        Some(o + 30 + u16_at(self.data, o + 26) + u16_at(self.data, o + 28))
    }

    /// Posicion de los datos de una parte. Varias entradas del directorio central, con nombres
    /// distintos, pueden apuntar a los mismos datos: si la posicion coincide, son la misma parte.
    pub fn data_at(&self, name: &str) -> Option<usize> {
        self.data_start(self.find(name)?)
    }

    /// Datos de una entrada tal como estan en el archivo (comprimidos o no).
    fn raw(&self, e: &Entry) -> Option<&'a [u8]> {
        let start = self.data_start(e)?;
        self.data.get(start..start.checked_add(e.packed)?)
    }

    pub fn read(&self, name: &str) -> Option<Vec<u8>> {
        let e = self.find(name)?;
        let raw = self.raw(e)?;
        match e.method {
            0 => Some(raw.to_vec()),
            8 => Some(inflate(raw).data),
            _ => None,
        }
    }

    /// Lee una entrada como texto UTF-8 (los XML de un .docx lo son; se tolera UTF-16 con BOM).
    /// Una parte almacenada sin comprimir tambien se descuenta del presupuesto de la lectura, como
    /// una comprimida (que descuenta `inflate`): asi leer muchas veces los mismos datos, citados
    /// desde muchas hojas o diapositivas, tampoco queda gratis.
    pub fn read_text(&self, name: &str) -> Option<String> {
        let e = self.find(name)?;
        let raw = self.raw(e)?;
        let d = match e.method {
            0 => {
                let n = crate::budget::room(raw.len());
                crate::budget::spend(n, n < raw.len());
                raw[..n].to_vec()
            }
            8 => inflate(raw).data,
            _ => return None,
        };
        Some(if d.starts_with(&[0xFF, 0xFE]) || d.starts_with(&[0xFE, 0xFF]) {
            let be = d[0] == 0xFE;
            let units: Vec<u16> = d[2..].chunks_exact(2).map(|c| if be { u16::from_be_bytes([c[0], c[1]]) } else { u16::from_le_bytes([c[0], c[1]]) }).collect();
            String::from_utf16_lossy(&units)
        } else {
            let d = d.strip_prefix(&[0xEF, 0xBB, 0xBF][..]).unwrap_or(&d);
            String::from_utf8_lossy(d).into_owned()
        })
    }
}
